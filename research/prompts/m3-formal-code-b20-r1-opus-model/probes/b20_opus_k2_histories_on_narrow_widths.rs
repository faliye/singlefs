//! 攻方 m3-formal-code-b20-r1-opus 的副本探针（harness 档，只放进攻方自己拷出的副本里跑）：
//! 崩溃注入那一种跑法（随机历史、不逐步跑 checker、空间准入判着）在窄盘几档上，崩溃态之后的可写挂载交回的现行那一版
//! 落进 20-1 发布臂的哪一支；落进「第一个文件 nlink = 0」那一支的，再跑 20-1 的臂与会话建 inode，比结局与之后的池级 checker。
//! 崩溃态取层 0 的枚举域：mkfs 之后的每一段，段内写的每个子集（不含整段，整段等于下一段的空集），外加全部持久那一个；
//! 状态数照 singlefs_harness::memory_pool::closed_form_state_count 算，超过 B20_STATE_CAP 的历史整段不跑（只报数）。
//! 环境变量：B20_WIDTH（384 / 256 / 240 / 4096 = 4 GiB）、B20_WEIGHTS（deleting / unitwall / broad）、B20_SEED_FROM、B20_SEED_COUNT、B20_OPS、
//! B20_STATE_CAP（缺省 1000000）、B20_ESTIMATE_ONLY（1 = 只算状态数）、B20_BOUNDARY_ONLY（1 = 只取每段的空集，即「之前的段整段持久、
//! 这一段一个都没落」那几个段边界态，外加全部持久那一个；这不是层 0 的枚举域，报告里单列、不当崩溃态覆盖）。

mod common_admission;

use common_admission::{checker_violations_on, plain_devices_on};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::mount::{mount_writable_with_space_admission, DeletionQueueContinuationOutcome};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::records::NLINK_OF_AN_INODE_BEING_DELETED;
use singlefs_core::transaction::{publish_new_inodes, publish_overwrite, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::history::{
    execute_history_with, generate_history_with_weights, GenerationWeights, HistoryDeviceWidth,
    HistoryExecution, HistorySeed, PerStepChecker,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments_with_stream_indexes, MemoryPool, SparseBlockDevice,
};
use singlefs_harness::model_comparison::refusal_reason_of_publish_error;
use singlefs_harness::SharedStream;
use std::collections::BTreeMap;

const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_100;

fn variable(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(default)
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)], device_size_in_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes,
    }
}

#[test]
fn b20_opus_k2_branch_of_the_publish_arm_over_every_crash_state_of_narrow_histories() {
    let width = match variable("B20_WIDTH", 384) {
        256 => HistoryDeviceWidth::UnitAreaOf256Slots,
        240 => HistoryDeviceWidth::UnitAreaOf240Slots,
        4096 => HistoryDeviceWidth::FourGibibytes,
        _ => HistoryDeviceWidth::UnitAreaOf384Slots,
    };
    let weights = match std::env::var("B20_WEIGHTS").as_deref() {
        Ok("unitwall") => GenerationWeights::TOWARD_THE_UNIT_AREA_WALL,
        Ok("broad") => GenerationWeights::BROAD,
        _ => GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES,
    };
    let seed_from = variable("B20_SEED_FROM", 0);
    let seed_count = variable("B20_SEED_COUNT", 1);
    let operations = usize::try_from(variable("B20_OPS", 12)).expect("步数");
    let state_cap = variable("B20_STATE_CAP", 1_000_000);
    let estimate_only = variable("B20_ESTIMATE_ONLY", 0) == 1;
    let boundary_only = variable("B20_BOUNDARY_ONLY", 0) == 1;
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: width,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let parameters = width.parameters();
    let mut tally: BTreeMap<String, u64> = BTreeMap::new();
    let mut states_total = 0u64;
    for seed in seed_from..seed_from + seed_count {
        let history = generate_history_with_weights(HistorySeed(seed), operations, &weights);
        let stream = SharedStream::retaining_contents();
        let run = execute_history_with(&history, execution, &stream, &mut |_| {});
        let unlinks = history
            .operations
            .iter()
            .filter(|operation| format!("{operation:?}") == "Unlink")
            .count();
        let (writes, segments, stream_indexes) =
            writes_and_segments_with_stream_indexes(&stream.retained_operations(), &width.fixed_geometry());
        let candidate: Vec<Vec<usize>> = segments
            .iter()
            .filter(|segment| {
                segment
                    .iter()
                    .all(|write| stream_indexes[*write] >= run.operations_written_by_make_filesystem)
            })
            .cloned()
            .collect();
        let states = closed_form_state_count(&candidate);
        let largest_segment = candidate.iter().map(Vec::len).max().unwrap_or(0);
        println!(
            "HISTORY seed={seed} ending={:?} unlinks={unlinks} writes={} candidate_segments={} largest_segment={largest_segment} states={states}",
            std::mem::discriminant(&run.ending),
            writes.len(),
            candidate.len()
        );
        if states > state_cap && !boundary_only {
            *tally.entry("history skipped over the state cap".to_string()).or_insert(0) += 1;
            continue;
        }
        states_total += states;
        if estimate_only {
            continue;
        }
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], width.device_bytes());
        let mut applied = 0usize;
        let mut judge_state = |image: &MemoryPool, tally: &mut BTreeMap<String, u64>, label: &str| {
            let mut devices = plain_devices_on(image);
            let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula);
            let mounted = match mounted {
                Err(_) => {
                    *tally.entry("mount refused".to_string()).or_insert(0) += 1;
                    return;
                }
                Ok(mounted) => mounted,
            };
            let short = matches!(
                mounted.output.deletion_queue_continuation_outcome,
                DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. }
            );
            let branch = match &mounted.current {
                PoolVersion::WithoutFile(_) => "first file (tree table 0 entries)",
                PoolVersion::WithTreesAndNoFile(_) => "new inode (with trees and no file)",
                PoolVersion::WithFile(version) => match version.inode_record {
                    Some(record) if record.nlink != NLINK_OF_AN_INODE_BEING_DELETED => "overwrite (first file live)",
                    Some(_) => "new inode (first file nlink 0)",
                    None => "new inode (first file not in the tree)",
                },
            };
            *tally.entry(format!("branch {branch}{}", if short { " / continuation short" } else { "" })).or_insert(0) += 1;
            // 一一对应那一格：带文件的一版上，20-1 的判法（活着）与实现写入口（publish_overwrite 现调一次）各说什么。
            if let PoolVersion::WithFile(version) = &mounted.current {
                let live = version
                    .inode_record
                    .is_some_and(|record| record.nlink != NLINK_OF_AN_INODE_BEING_DELETED);
                let mut probe_devices = plain_devices_on(image);
                let probe = mount_writable_with_space_admission(&parameters, &mut probe_devices, SpaceAdmission::JudgedByTheFormula)
                    .expect("同一份镜像现挂第二次");
                let content = vec![0x5a_u8; 120];
                let mut probe_allocator = probe.allocator;
                let probe_version = probe.current.version_of_trees().expect("有树").clone();
                let mut writer = PoolWriter::new(&parameters, probe_devices.as_mut_slice());
                let entrance = match publish_overwrite(
                    &mut writer,
                    &mut probe_allocator,
                    &probe_version,
                    FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
                    probe.output.instance,
                ) {
                    Ok(_) => "overwrite published".to_string(),
                    Err(error) => format!("overwrite refused {}", format!("{error:?}").split([' ', '(']).next().unwrap_or("")),
                };
                *tally.entry(format!("mapping live={live} / {entrance}")).or_insert(0) += 1;
            }
            if branch != "new inode (first file nlink 0)" {
                return;
            }
            // 20-1 的臂：直接建一个 inode。
            let instance = mounted.output.instance;
            let mut allocator = mounted.allocator;
            let current = mounted.current.version_of_trees().expect("有树").clone();
            let outcome = {
                let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
                match publish_new_inodes(&mut writer, &mut allocator, &current, 1, FIXED_WRITE_TIME_SECONDS, instance) {
                    Ok(_) => "published".to_string(),
                    Err(error) => format!("refused {:?}", refusal_reason_of_publish_error(&error)),
                }
            };
            let violations = checker_violations_on(&image_of(&devices, image.device_size_in_bytes));
            *tally.entry(format!("nlink0 arm 20-1: {outcome}; checker red {}", !violations.is_empty())).or_insert(0) += 1;
            // 会话建 inode（先走空）。
            let mut devices = plain_devices_on(image);
            let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula)
                .expect("同一份镜像现挂第二次");
            let (_output, mut session) = MountedSession::of_the_mount(mounted);
            let session_outcome = match session.publish_user_change(
                &mut devices,
                UserChange::NewInodes { count: 1, write_time_seconds: FIXED_WRITE_TIME_SECONDS },
            ) {
                Ok(published) => format!("published {} versions", published.versions_published.len()),
                Err(refusal) => format!("refused {}", format!("{refusal:?}").split('(').next().unwrap_or("")),
            };
            let violations = checker_violations_on(&image_of(&devices, image.device_size_in_bytes));
            *tally.entry(format!("nlink0 arm session: {session_outcome}; checker red {}", !violations.is_empty())).or_insert(0) += 1;
            println!("  NLINK0 state {label}");
        };
        for (segment_index, segment) in candidate.iter().enumerate() {
            let first = segment[0];
            base.apply_writes(&writes[applied..first]);
            applied = first;
            let length = segment.len();
            let masks = if boundary_only { 1 } else { (1u64 << length) - 1 };
            for mask in 0..masks {
                let mut image = base.clone();
                for (within, write) in segment.iter().enumerate() {
                    if mask & (1 << within) != 0 {
                        image.apply_writes(std::slice::from_ref(&writes[*write]));
                    }
                }
                judge_state(&image, &mut tally, &format!("seed={seed} segment={segment_index} mask={mask:#x}"));
            }
        }
        base.apply_writes(&writes[applied..]);
        judge_state(&base, &mut tally, &format!("seed={seed} all persisted"));
    }
    println!("STATES_TOTAL {states_total}");
    for (key, count) in &tally {
        println!("TALLY {count:>8}  {key}");
    }
}

//! m3-code-r1 攻方腿 Z2 ③：合法崩溃状态上的可写挂载会不会被新判法（落后支缺根那一半）拒成落后支。
//! 每条流：起点之后 1–2 次发布（覆盖写 / 准入抬 F / 管理员回退 / 建一个 inode / 崩了再可写挂载），录下最后那次的写流，
//! 用 checker 档 `enumerate_layer0_in_state_slices`（每一段都展开 `full_expansion`，原地覆写取三态）走遍它的崩溃状态，
//! 每个状态上 checker 由枚举器自己判，另在观察者里把那个状态落成盘面、可写挂载一次，按拒因分类。
//! 状态数先用 `closed_form_state_count` 算，超过 M3R1_STATE_LIMIT（默认 10⁶）的流不跑、只打出来。
//! M3R1_ENUMERATE=0 时只打每条流的段长与状态数（估时用）。M3R1_FLOWS 逗号分隔只跑点名的流。

#[path = "../../singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use common_admission::{content_of, plain_devices_on, start_plain, OVERWRITE_BYTES};
use singlefs_checker_tier::crash::{
    enumerate_layer0_in_state_slices, full_expansion, Layer0ObserverCounts, Layer0Parallelism, Layer0SliceLength,
    Layer0WorkerThreadsSource,
};
use singlefs_checker_tier::layer0_progress::Layer0Resume;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor_to_the_admission_ceiling, roll_back_by_a_forward_publish, RollbackTarget,
    ShadowLedger,
};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::RecoveryReport;
use singlefs_core::transaction::{FirstFile, PoolVersion};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, PublishedVersion, SparseBlockDevice,
};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

fn clone_session(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
    }
}

type Contents = BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>;

/// 起点之后跑一个动作：交回是否做成、之后的内容。
fn act(
    action: &str,
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>,
    session: &mut MountedSession,
    contents: &Contents,
    content_now: &[u8],
    time: u64,
) -> (bool, Vec<u8>) {
    match action {
        "overwrite" => {
            let content = content_of(OVERWRITE_BYTES, time);
            let ok = session.publish_user_change(devices, UserChange::Overwrite(FirstFile { content: &content, write_time_seconds: time })).is_ok();
            (ok, if ok { content } else { content_now.to_vec() })
        }
        "create_inode" => (session.publish_user_change(devices, UserChange::NewInodes { count: 1, write_time_seconds: time }).is_ok(), content_now.to_vec()),
        "raise_floor" => {
            let PoolVersion::WithFile(current) = &mut session.current else { return (false, content_now.to_vec()) };
            (raise_rollback_floor_to_the_admission_ceiling(parameters, devices, &mut session.allocator, current, ShadowLedger::On).is_ok(), content_now.to_vec())
        }
        "rollback" => {
            let current_root = *session.current.root();
            let Some((instance, txg)) = contents.keys().filter(|(_, txg)| *txg >= current_root.rollback_floor && *txg < current_root.checkpoint_txg).min_by_key(|(_, txg)| *txg).copied() else {
                return (false, content_now.to_vec());
            };
            let PoolVersion::WithFile(current) = &mut session.current else { return (false, content_now.to_vec()) };
            let ok = roll_back_by_a_forward_publish(parameters, devices, &mut session.allocator, current, RollbackTarget { instance, checkpoint_txg: txg }).is_ok();
            (ok, if ok { contents[&(instance, txg)].clone() } else { content_now.to_vec() })
        }
        "crash_remount" => match mount_writable(parameters, devices) {
            Ok(mounted) => {
                let (output, new_session) = MountedSession::of_the_mount(mounted);
                *session = new_session;
                (true, contents.get(&(output.effective_root.instance, output.effective_root.checkpoint_txg)).cloned().unwrap_or_else(|| content_now.to_vec()))
            }
            Err(_) => (false, content_now.to_vec()),
        },
        other => panic!("没有这个动作 {other}"),
    }
}

fn materialize(image: &CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    for (write, persisted) in image.writes.iter().zip(&image.persisted) {
        if *persisted {
            pool.apply_writes(std::slice::from_ref(write));
        }
    }
    pool
}

fn class_of(outcome: &Result<(), String>) -> String {
    match outcome {
        Ok(()) => "Ok".to_string(),
        Err(text) => {
            let head: String = text.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            if text.contains("BehindTheSelectedVersionAndItsRootNotInItsRingSlot") {
                format!("{head}/BehindTheSelectedVersionAndItsRootNotInItsRingSlot")
            } else if text.contains("BehindTheSelectedVersionAndMissingItsUnits") {
                format!("{head}/BehindTheSelectedVersionAndMissingItsUnits")
            } else {
                head
            }
        }
    }
}

#[test]
fn every_crash_state_of_the_last_publish_then_one_writable_mount() {
    let enumerate = std::env::var("M3R1_ENUMERATE").map_or(true, |text| text != "0");
    let limit: u64 = std::env::var("M3R1_STATE_LIMIT").ok().and_then(|text| text.parse().ok()).unwrap_or(1_000_000);
    let threads: usize = std::env::var("M3R1_THREADS").ok().and_then(|text| text.parse().ok()).unwrap_or(8);
    let only: Option<Vec<String>> = std::env::var("M3R1_FLOWS").ok().map(|text| text.split(',').map(str::to_string).collect());
    let mut pool = start_plain(HistoryDeviceWidth::UnitAreaOf384Slots);
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 1");
    pool.overwrite(OVERWRITE_BYTES).expect("覆盖写 2");
    let parameters = pool.parameters.clone();
    let start_image = pool.image();
    let start_session = pool.session.take().expect("会话");
    let start_contents = pool.content_of_each_root.clone();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let kinds = ["overwrite", "raise_floor", "rollback", "create_inode", "crash_remount"];
    let mut flows: Vec<Vec<&str>> = kinds.iter().map(|kind| vec![*kind]).collect();
    for first in kinds {
        for last in kinds {
            flows.push(vec![first, last]);
        }
    }
    let mut grand_total = 0u64;
    let mut flagged = Vec::new();
    for flow in flows {
        let name = flow.join("+");
        if only.as_ref().is_some_and(|only| !only.contains(&name)) {
            continue;
        }
        let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&start_image)
            .into_iter()
            .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, SharedStream::retaining_contents())))
            .collect();
        let mut session = clone_session(&start_session);
        let mut contents = start_contents.clone();
        let mut content_now = pool.content_of_the_current_version.clone();
        let mut time = pool.write_time_seconds;
        let mut done = true;
        let mut base = start_image.clone();
        let stream = SharedStream::retaining_contents();
        for (index, action) in flow.iter().enumerate() {
            time += 1;
            if index + 1 == flow.len() {
                base = MemoryPool { devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(), device_size_in_bytes: start_image.device_size_in_bytes };
                devices = plain_devices_on(&base)
                    .into_iter()
                    .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
                    .collect();
            }
            let newest_txg_before = session.current.root().checkpoint_txg;
            let (ok, content) = act(action, &parameters, &mut devices, &mut session, &contents, &content_now, time);
            if !ok {
                done = false;
                break;
            }
            content_now = content;
            // 这一步写下的每条根（含抬 F 推的、可写挂载的写行与暖机）都记上这一步之后的内容：层 0 的 oracle 按它判读回什么。
            let image_now = MemoryPool { devices: devices.iter().map(|(identity, device)| (*identity, device.wrapped_device().image.clone())).collect(), device_size_in_bytes: start_image.device_size_in_bytes };
            let fsid = parameters.filesystem_identifier;
            for (_, root) in singlefs_core::recovery::readable_roots_with_ring_slots(&image_now, &parameters.region_devices, &parameters.geometry, &fsid) {
                if root.checkpoint_txg > newest_txg_before {
                    contents.entry((root.instance, root.checkpoint_txg)).or_insert_with(|| content_now.clone());
                }
            }
        }
        if !done {
            println!("FLOW {name} 做不成，跳过");
            continue;
        }
        let operations = stream.retained_operations();
        let (writes, segments) = writes_and_segments(&operations, &geometry);
        let state_count = closed_form_state_count(&segments);
        let sizes: Vec<usize> = segments.iter().map(Vec::len).collect();
        let Some(root_index) = writes.iter().rposition(|write| write.kind == StepKind::RootRecordFua) else {
            println!("FLOW {name} 流里没有根槽写，跳过");
            continue;
        };
        println!("FLOW {name} writes={} segments={sizes:?} closed_form_state_count={state_count}", writes.len());
        if !enumerate {
            continue;
        }
        if state_count > limit {
            println!("FLOW {name} 状态数 {state_count} 超过 {limit}，不跑");
            continue;
        }
        let versions: Vec<PublishedVersion> = contents
            .iter()
            .map(|((instance, checkpoint_txg), content)| PublishedVersion { instance: *instance, checkpoint_txg: *checkpoint_txg, content: content.clone() })
            .collect();
        let mut classes: BTreeMap<String, u64> = BTreeMap::new();
        let started = std::time::Instant::now();
        let tally = enumerate_layer0_in_state_slices(
            &base,
            &writes,
            &segments,
            root_index,
            &versions,
            &full_expansion,
            Layer0Parallelism {
                worker_threads: NonZeroUsize::new(threads).expect("正"),
                worker_threads_source: Layer0WorkerThreadsSource::GivenByCaller,
                slice_length: Layer0SliceLength::ScaledToWorkerThreads,
            },
            Some(&mut |image: &CrashImage<'_>, _report: &RecoveryReport, _counts: &mut Layer0ObserverCounts| {
                let crashed = materialize(image);
                let mut devices = plain_devices_on(&crashed);
                let outcome = mount_writable(&parameters, &mut devices).map(|_| ()).map_err(|error| format!("{error:?}"));
                let class = class_of(&outcome);
                if class.contains("Behind") {
                    flagged.push(format!("FLOW {name} persisted={:?} outcome={}", image.persisted.iter().map(|p| u8::from(*p)).collect::<Vec<_>>(), outcome.as_ref().err().map(|e| e.chars().take(300).collect::<String>()).unwrap_or_default()));
                }
                *classes.entry(class).or_default() += 1;
            }),
            &Layer0Resume::NoProgressFile,
        );
        grand_total += tally.states;
        println!(
            "FLOW {name} states={} violations={} first_violation={:?} record_root_without_record={} record_claimed_state_missing_unit={} elapsed={:.1}s mount_outcomes={classes:?}",
            tally.states, tally.violations, tally.first_violation, tally.record_root_without_record, tally.record_claimed_state_missing_unit, started.elapsed().as_secs_f64()
        );
    }
    println!("TOTAL states={grand_total} flagged={}", flagged.len());
    for line in flagged.iter().take(50) {
        println!("FLAG {line}");
    }
}

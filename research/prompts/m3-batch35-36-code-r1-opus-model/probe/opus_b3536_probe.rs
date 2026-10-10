//! m3-batch35-36-code-r1 攻方（Opus）探针：只在攻方的仓副本里编，不入库。每条用例打印以 `OPUS ` 开头的判定行，不断言。
//! 装置照 `tests/common_unwitnessed_publish/mod.rs`（384 槽小盘、第一个文件之后覆盖写 k 次、录一次发布 R，崩在 R 的根槽落盘之后、轮换之前）。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports, reason = "攻方原型")]

#[path = "common_unwitnessed_publish/mod.rs"]
mod common_unwitnessed_publish;

use std::collections::{BTreeMap, BTreeSet};

use common_unwitnessed_publish::*;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, scan_journal, JournalPolicy,
    RecoveryOutcome, RootRingSlotReading,
};
use singlefs_harness::memory_pool::{writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

type Key = (InstanceGeneration, CheckpointTxg);
type Range = (DeviceIdentity, u64, u64);

fn k(key: Key) -> String {
    format!("({},{})", key.0 .0, key.1 .0)
}

fn kind_of(text: &str) -> String {
    text.split([' ', '(', '{']).next().unwrap_or("").to_string()
}

fn short_mount_error(error: &MountError) -> String {
    match error {
        MountError::NewerStateStillUnreadableAfterOneReread(still) => format!("Refused:{}", kind_of(&format!("{still:?}"))),
        other => format!("Err:{}", kind_of(&format!("{other:?}"))),
    }
}

/// 环里全部自证过的根的 (实例, txg)，按 (txg, 实例) 排。
fn ring_keys(image: &MemoryPool) -> Vec<Key> {
    let Ok(configuration) = choose_system_configuration(image) else { return Vec::new() };
    let sizes = &configuration.immutable.sizes;
    let mut keys: Vec<Key> = every_root_ring_slot(sizes)
        .into_iter()
        .filter_map(|ring_slot| match read_root_ring_slot(image, &configuration.immutable.region_devices, sizes, &configuration.immutable.filesystem_identifier, ring_slot) {
            RootRingSlotReading::SelfVerified(root) => Some((root.instance, root.checkpoint_txg)),
            RootRingSlotReading::Bad(_) => None,
        })
        .collect();
    keys.sort_by_key(|key| (key.1, key.0));
    keys
}

fn ranges_of(writes: &[&RetainedWrite]) -> Vec<Range> {
    writes.iter().map(|w| (w.device, w.offset.0, w.length_in_bytes())).collect()
}

/// R 写出的单元，按字节相同归成一个单元的几份（每个单元一组：两块盘各一份）。
fn units_of_r(recorded: &RecordedPublish) -> Vec<Vec<&RetainedWrite>> {
    let mut groups: Vec<Vec<&RetainedWrite>> = Vec::new();
    for write in recorded.unit_writes() {
        if let Some(group) = groups.iter_mut().find(|g| g[0].bytes() == write.bytes() && g.iter().all(|o| o.device != write.device)) {
            group.push(write);
        } else {
            groups.push(vec![write]);
        }
    }
    groups
}

/// R 的根记录直接指着的单元：一组的第一份落在哪条根指针上（T 树表、M 中央映射树根、I 实例表、A 分配记录树根，都不是写 -）。
fn root_pointer_of(recorded: &RecordedPublish, group: &[&RetainedWrite]) -> &'static str {
    let root = singlefs_core::root_record::RootRecord::parse_slot(
        recorded.root_slot_write().bytes().expect("根槽写带字节"),
        &recorded.pool.parameters.filesystem_identifier,
    )
    .expect("R 的根自证");
    let hits = |pointer: &singlefs_core::pointer::NodePointer| {
        pointer.locations.iter().any(|location| group.iter().any(|w| location.device == w.device && location.slot.to_device_offset() == w.offset))
    };
    if hits(&root.tree_table) {
        "T"
    } else if hits(&root.mapping_root) {
        "M"
    } else if hits(&root.instance_table) {
        "I"
    } else if hits(&root.allocation_record_tree_root) {
        "A"
    } else {
        "-"
    }
}

/// 单元头第 6 字节的类码（1 数据、2 节点、3 打包），读不出写 ?
fn class_of(write: &RetainedWrite) -> String {
    write.bytes().and_then(|b| b.get(6).copied()).map(|c| c.to_string()).unwrap_or_else(|| "?".to_string())
}

fn recover_label(image: &MemoryPool, recorded: &RecordedPublish) -> String {
    let report = recover(image, JournalPolicy::Consult);
    let name = |key: Key| if key == recorded.key { "R".to_string() } else if key == recorded.previous_key { "P".to_string() } else { k(key) };
    let content_name = |content: &[u8]| if content == recorded.content.as_slice() { "R" } else if content == recorded.previous_content.as_slice() { "P" } else { "OTHER" };
    match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => format!("rec=FileRead({} content={})", name(*root), content_name(content)),
        RecoveryOutcome::NoFile { root } => format!("rec=NoFile({})", name(*root)),
        RecoveryOutcome::Failed { root, failure } => format!("rec=Failed({} {})", root.map(name).unwrap_or_default(), kind_of(&format!("{failure:?}"))),
    }
}

fn checker_names(image: &MemoryPool) -> BTreeSet<String> {
    checker_violations_on(image).into_iter().map(|l| l.split(':').next().unwrap_or("").to_string()).collect()
}

fn publish_kind(code: &str) -> PublishRecordedAsR {
    match code {
        "ow" => PublishRecordedAsR::Overwrite,
        "sw3" => PublishRecordedAsR::SequentialWriteOfThreeDataUnits,
        other => panic!("{other}"),
    }
}

/// 一次可写挂载 M1（`hidden` 那几段读回全零）之后：所选那一版、C 隔离的槽数、写行那次的根落盘之前 R 的单元被写掉几份、
/// 那一刻的池级 checker 与恢复；再在 M1 之后放开用户动作（o 覆盖写、s 顺序写两个数据单元，长度 ≤ depth），每个结点看
/// R 的根还在不在环里、R 的单元被写掉几份。交回一行汇总。
fn mount_and_sweep(recorded: &RecordedPublish, crash: &MemoryPool, hidden: &[Range], depth: usize, tag: &str) -> String {
    let stream = SharedStream::retaining_contents();
    let ranges = RangesReadingBackAllZero(hidden.to_vec());
    let mut recorded_devices = recorded_devices_reading_back_all_zero_on(crash, &ranges, &stream);
    let mounted = match mount_writable(&recorded.pool.parameters, &mut recorded_devices) {
        Ok(mounted) => mounted,
        Err(error) => return format!("OPUS {tag} M1={}", short_mount_error(&error)),
    };
    let effective = (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg);
    let isolated: Vec<u64> = mounted.allocator.devices.iter().map(|d| d.isolated_slots()).collect();
    let (mount_writes, _s, _i) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
    let row_root = mount_writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("写行那次有根槽写");
    let mut before_row_root = crash.clone();
    before_row_root.apply_writes(&mount_writes[..row_root]);
    let overwritten_before_row_root = units_of_r_overwritten_on(recorded, &before_row_root).len();
    let chk_before = checker_names(&before_row_root);
    let rec_before = recover_label(&before_row_root, recorded);
    // 用户动作那一段：同一份盘面、同一组读回全零的段，另挂一次（不录），接着发。
    let mut devices = devices_reading_back_all_zero_on(crash, &ranges);
    let mounted_again = mount_writable(&recorded.pool.parameters, &mut devices).expect("同一份盘面第二次挂得上");
    let image = image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
    let pool = pool_after_the_mount(recorded, &image, mounted_again);
    let mut stack: Vec<(String, PoolUnderTest<SparseBlockDevice>)> = vec![(String::new(), pool)];
    let mut worst = (0usize, String::new());
    let mut nodes = 0usize;
    while let Some((sequence, node)) = stack.pop() {
        nodes += 1;
        let image = node.image();
        let in_ring = ring_keys(&image).contains(&recorded.key);
        let overwritten = units_of_r_overwritten_on(recorded, &image).len();
        if in_ring && overwritten > worst.0 {
            worst = (overwritten, if sequence.is_empty() { "-".to_string() } else { sequence.clone() });
        }
        if sequence.len() < depth {
            for op in ['o', 's', 'n'] {
                let mut child = clone_pool(&node);
                let outcome = match op {
                    'o' => child.overwrite(OVERWRITE_BYTES).map(|_| ()),
                    'n' => child.create_one_inode().map(|_| ()),
                    _ => child.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity()).map(|_| ()),
                };
                if outcome.is_ok() {
                    stack.push((format!("{sequence}{op}"), child));
                }
            }
        }
    }
    format!(
        "OPUS {tag} M1=Ok effective={} isolated={isolated:?} before_row_root:r_copies_overwritten={overwritten_before_row_root} chk={chk_before:?} {rec_before} sweep(depth={depth},nodes={nodes}):max_r_copies_overwritten_while_r_in_ring={} at={}",
        if effective == recorded.previous_key { "P".to_string() } else if effective == recorded.key { "R".to_string() } else { k(effective) },
        worst.0,
        if worst.1.is_empty() { "-" } else { &worst.1 }
    )
}

fn clone_pool(pool: &PoolUnderTest<SparseBlockDevice>) -> PoolUnderTest<SparseBlockDevice> {
    let session = pool.session();
    PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: plain_devices_on(&pool.image()),
        session: Some(MountedSession {
            allocator: session.allocator.clone(),
            current: session.current.clone(),
            instance: session.instance,
            shadow_ledger: session.shadow_ledger,
            parameters_and_device_table: session.parameters_and_device_table.clone(),
        }),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: pool.content_of_each_root.clone(),
    }
}

fn env_list(name: &str, default: &str) -> Vec<String> {
    std::env::var(name).unwrap_or_else(|_| default.to_string()).split(',').map(str::to_string).collect()
}

/// C 与 C_ACCOUNT 的盲区扫描：R 的哪个单元读回全零（不藏、某个单元两份、某个单元一份）× R 的哪几条记录读回全零（全部子集）×
/// R 的根槽藏不藏（OPUS_ROOT_HIDDEN=true/false 只跑一半），每格可写挂载一次并扫用户动作（o 覆盖写、s 顺序写、n 建 inode）。`vis` 记 R 在 M1 那一刻看得见什么：根读得出（root）、有读得出的记录（rec）、都没有（none）。
#[test]
fn opus_c_blind_spot_sweep() {
    let depth: usize = std::env::var("OPUS_DEPTH").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    let single_copy = std::env::var("OPUS_SINGLE_COPY").is_ok();
    for kind in env_list("OPUS_KINDS", "ow,sw3") {
        for overwrites_before in env_list("OPUS_KS", "5").iter().map(|v| v.parse::<u64>().expect("OPUS_KS")) {
            let recorded = record_one_publish_after_overwrites(overwrites_before, publish_kind(&kind));
            let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let units = units_of_r(&recorded);
            let record_count = recorded.record_count();
            println!(
                "OPUS CBINFO kind={kind} k={overwrites_before} R={} P={} units={} records={record_count} classes={:?}",
                k(recorded.key),
                k(recorded.previous_key),
                units.len(),
                units.iter().enumerate().map(|(i, g)| format!("u{i}:c{}x{}:{}", class_of(g[0]), g.len(), root_pointer_of(&recorded, g))).collect::<Vec<_>>()
            );
            let mut unit_choices: Vec<(String, Vec<Range>)> = vec![("none".to_string(), Vec::new())];
            for (index, group) in units.iter().enumerate() {
                unit_choices.push((format!("u{index}c{}", class_of(group[0])), ranges_of(group)));
                if single_copy {
                    for (copy, write) in group.iter().enumerate() {
                        unit_choices.push((format!("u{index}c{}copy{copy}", class_of(group[0])), ranges_of(&[*write])));
                    }
                }
            }
            for (unit_name, unit_ranges) in &unit_choices {
                for record_mask in 0u32..(1 << record_count) {
                    for root_hidden in [false, true].into_iter().filter(|hidden| std::env::var("OPUS_ROOT_HIDDEN").map_or(true, |only| only == hidden.to_string())) {
                        let mut hidden = unit_ranges.clone();
                        for ordinal in 1..=record_count {
                            if record_mask & (1 << (ordinal - 1)) != 0 {
                                hidden.extend(ranges_of(&recorded.record_writes_of_the_record(ordinal)));
                            }
                        }
                        if root_hidden {
                            hidden.extend(ranges_of(&[recorded.root_slot_write()]));
                        }
                        let all_records_hidden = record_mask == (1 << record_count) - 1;
                        let visibility = if !root_hidden { "root" } else if !all_records_hidden { "rec" } else { "none" };
                        let tag = format!(
                            "CB kind={kind} k={overwrites_before} unit={unit_name} rec_hidden={record_mask:0width$b} root_hidden={root_hidden} vis={visibility}",
                            width = record_count
                        );
                        println!("{}", mount_and_sweep(&recorded, &crash, &hidden, depth, &tag));
                    }
                }
            }
        }
    }
}

/// H1 的最小形：照入库用例 `a_power_loss_before_the_row_root_of_the_mount_still_reads_back_a_candidate_root_whose_mapping_root_read_back_zero_at_that_mount`
/// （K4-1 那一形：M1 里只有 R 的中央映射树根两份读回全零）原样，只多藏一样：R 那一条记录（两块盘各一份）。打印两形各自 M1 落到哪、
/// C 隔离了几个槽、写行那次的根落盘之前 R 的单元被写掉的（盘, 偏移）、那一刻 R 在不在环里、池级 checker 与恢复。
#[test]
fn opus_h1_the_k4_1_form_with_the_record_of_r_also_reading_back_zero() {
    let recorded = record_one_publish_after_overwrites(5, PublishRecordedAsR::Overwrite);
    let root_of_r = singlefs_core::root_record::RootRecord::parse_slot(
        recorded.root_slot_write().bytes().expect("根槽写带字节"),
        &recorded.pool.parameters.filesystem_identifier,
    )
    .expect("R 的根自证");
    let mapping_root_writes: Vec<&RetainedWrite> = recorded
        .unit_writes()
        .into_iter()
        .filter(|write| root_of_r.mapping_root.locations.iter().any(|l| l.device == write.device && l.slot.to_device_offset() == write.offset))
        .collect();
    let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    for (form, hidden) in [
        ("K4-1:mapping_root", ranges_of(&mapping_root_writes)),
        ("H1:mapping_root+record", {
            let mut ranges = ranges_of(&mapping_root_writes);
            ranges.extend(ranges_of(&recorded.record_writes_of_the_record(1)));
            ranges
        }),
    ] {
        let stream = SharedStream::retaining_contents();
        let mut devices = recorded_devices_reading_back_all_zero_on(&crash, &RangesReadingBackAllZero(hidden.clone()), &stream);
        let mounted = match mount_writable(&recorded.pool.parameters, &mut devices) {
            Ok(mounted) => mounted,
            Err(error) => {
                println!("OPUS H1 form={form} M1={}", short_mount_error(&error));
                continue;
            }
        };
        let effective = (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg);
        let isolated: Vec<u64> = mounted.allocator.devices.iter().map(|d| d.isolated_slots()).collect();
        let (mount_writes, _s, _i) = writes_and_segments_with_stream_indexes(&stream.retained_operations(), &fixed_geometry_of_the_384_slot_small_devices());
        let row_root = mount_writes.iter().position(|w| w.kind == StepKind::RootRecordFua).expect("写行那次有根槽写");
        let mut before_row_root = crash.clone();
        before_row_root.apply_writes(&mount_writes[..row_root]);
        let overwritten = units_of_r_overwritten_on(&recorded, &before_row_root);
        println!(
            "OPUS H1 form={form} M1=Ok effective={} isolated={isolated:?} hidden_ranges={} before_row_root:r_in_ring={} r_copies_overwritten={} at={:?} chk={:?} {}",
            if effective == recorded.previous_key { "P" } else if effective == recorded.key { "R" } else { "other" },
            hidden.len(),
            ring_keys(&before_row_root).contains(&recorded.key),
            overwritten.len(),
            overwritten.iter().map(|(d, o)| format!("d{}@{}", d.0, o)).collect::<Vec<_>>(),
            checker_names(&before_row_root),
            recover_label(&before_row_root, &recorded)
        );
    }
}

/// W4：R 没见证（崩在根槽之后、轮换之前），P 已确认（它那次轮换写的系统配置槽在每块盘上都是较新的那一槽）。读的那一刻读回全零：
/// R 的单元（设计里那一退）、每块盘较新的那个系统配置槽（P 那次轮换写的）、两样都藏。P 自己的根、记录、单元都读得出。
/// 打印恢复、只读挂载、可写挂载各落到哪一版（R、P、Pm = P 之前那一版）与读出的内容是哪一版的。
#[test]
fn opus_w4_f2_with_the_newer_system_configuration_slots_of_a_confirmed_version_reading_back_zero() {
    for kind in env_list("OPUS_KINDS", "ow,sw3") {
        for overwrites_before in env_list("OPUS_KS", "3,5").iter().map(|v| v.parse::<u64>().expect("OPUS_KS")) {
            let recorded = record_one_publish_after_overwrites(overwrites_before, publish_kind(&kind));
            let crash = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
            let p = recorded.previous_key;
            let pm = ring_keys(&crash).into_iter().filter(|key| (key.1, key.0) < (p.1, p.0)).max_by_key(|key| (key.1, key.0)).expect("P 之前一版");
            let configuration = choose_system_configuration(&crash).expect("系统配置");
            let spacing = u64::from(configuration.immutable.sizes.fixed_structure_slot_spacing);
            let mut newer_slots: Vec<Range> = Vec::new();
            let mut tails = Vec::new();
            for (device, sparse) in &crash.devices {
                let mut best: Option<(u64, u64, u64)> = None;
                for offset in [0, spacing] {
                    let bytes = sparse.read(DeviceOffsetInBytes(offset), 4096);
                    if let Ok(parsed) = singlefs_core::system_configuration::SystemConfiguration::parse_slot(&bytes) {
                        let candidate = (parsed.quantities.slot_generation, offset, parsed.quantities.journal_tail);
                        if best.map_or(true, |b| candidate.0 > b.0) {
                            best = Some(candidate);
                        }
                        tails.push(format!("d{}@{}:gen{}:tail{}", device.0, offset, parsed.quantities.slot_generation, parsed.quantities.journal_tail));
                    }
                }
                let (_, offset, _) = best.expect("每块盘有自证过的槽");
                newer_slots.push((*device, offset, 4096));
            }
            let name = |key: Key| if key == recorded.key { "R".to_string() } else if key == p { "P".to_string() } else if key == pm { "Pm".to_string() } else { k(key) };
            let contents = &recorded.pool.content_of_each_root;
            let content_name = |content: &[u8]| contents.iter().find(|(_, c)| c.as_slice() == content).map(|(key, _)| name(*key)).unwrap_or_else(|| "OTHER".to_string());
            println!("OPUS W4INFO kind={kind} k={overwrites_before} R={} P={} Pm={} slots={tails:?}", k(recorded.key), k(p), k(pm));
            for (form, hidden) in [
                ("units_of_R", ranges_of(&recorded.unit_writes())),
                ("newer_cfg_slots", newer_slots.clone()),
                ("units_of_R+newer_cfg_slots", {
                    let mut ranges = ranges_of(&recorded.unit_writes());
                    ranges.extend(newer_slots.iter().copied());
                    ranges
                }),
            ] {
                let ranges = RangesReadingBackAllZero(hidden.clone());
                let devices = devices_reading_back_all_zero_on(&crash, &ranges);
                let report = recover(&devices, JournalPolicy::Consult);
                let rec = match &report.outcome {
                    RecoveryOutcome::FileRead { root, content } => format!("rec=FileRead({} content={})", name(*root), content_name(content)),
                    RecoveryOutcome::NoFile { root } => format!("rec=NoFile({})", name(*root)),
                    RecoveryOutcome::Failed { root, failure } => format!("rec=Failed({} {})", root.map(name).unwrap_or_default(), kind_of(&format!("{failure:?}"))),
                };
                let ro = match mount_read_only(&devices) {
                    Ok(mounted) => format!("ro=Ok({})", name((mounted.effective_root.instance, mounted.effective_root.checkpoint_txg))),
                    Err(error) => format!("ro=Err({})", kind_of(&format!("{error:?}"))),
                };
                let mut devices_of_the_mount = devices_reading_back_all_zero_on(&crash, &ranges);
                let rw = match mount_writable(&recorded.pool.parameters, &mut devices_of_the_mount) {
                    Ok(mounted) => format!("rw=Ok({})", name((mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg))),
                    Err(error) => format!("rw={}", short_mount_error(&error)),
                };
                println!("OPUS W4 kind={kind} k={overwrites_before} hidden={form} {rec} {ro} {rw}");
            }
        }
    }
}

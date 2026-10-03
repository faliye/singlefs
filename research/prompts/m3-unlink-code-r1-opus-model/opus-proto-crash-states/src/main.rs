//! m3-unlink-code-r1 opus 攻方腿原型 E6：挂载续做被落得下那一判拒、推一串抬 F、接着走——那一段（续做的上一步落盘之后、推的那一串、
//! 推完之后续做的下一步）的每个崩溃状态：枚举器自己恢复、跑池级 checker（oracle 不给版本：这一段每个根下面都没有文件，读出内容就是违例）；
//! 之后每个状态再可写挂载一次（批大小 1、准入判着），挂载之后的镜像跑池级 checker、看队列走没走空。
//! 流只在这里自己造（窄盘、准入关着造盘面，同仓里 `admission_raises_the_floor_before_refusing.rs` 那条用例的盘面），不从层 0 用例里拷。
#![allow(dead_code)]

#[path = "/tmp/claude-1000/m3-unlink-code-r1-opus/tree/crates/singlefs-harness/tests/common_admission/mod.rs"]
mod common_admission;

use common_admission::{
    checker_violations_on, plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest, OVERWRITE_BYTES,
};
use singlefs_checker_tier::crash::{
    enumerate_layer0_in_state_slices, full_expansion, layer0_state_count_with_torn_in_place_overwrites,
    Layer0Parallelism,
};
use singlefs_checker_tier::layer0_progress::Layer0Resume;
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{mount_writable_with_test_only_switches_and_deletion_switches, MountError, ShadowLedger};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{publish_unlink_intent, publish_unlink_step, PoolVersion, PoolWriter, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::{DeletionSwitches, UnlinkBatchDataUnitsCappedForTests, UnlinkBatchSize};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{closed_form_state_count, writes_and_segments, MemoryPool, RetainedWrite, SparseBlockDevice};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::collections::BTreeMap;

const STATES_AT_MOST: u64 = 1_000_000;

fn switches_of_one() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: UnlinkBatchSize::CappedForTests(UnlinkBatchDataUnitsCappedForTests::new(1).unwrap()),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

fn crashed_pool(unit_area: u64, overwrites: usize, units: usize, batches: usize) -> PoolUnderTest<SparseBlockDevice> {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            unit_area,
        );
    for _ in 0..overwrites {
        pool.overwrite(OVERWRITE_BYTES).unwrap();
    }
    pool.sequential_write((units - 1) * data_unit_payload_capacity() + 100).unwrap();
    {
        let PoolUnderTest { session, devices, parameters, .. } = &mut pool;
        let session = session.as_mut().unwrap();
        let mut current = session.current.version_of_trees().unwrap().clone();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        current = publish_unlink_intent(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), session.instance).unwrap();
        for _ in 0..batches {
            current = publish_unlink_step(&mut writer, &mut session.allocator, &current, InodeNumber(FIRST_INODE_NUMBER), switches_of_one().batch_size, session.instance).unwrap();
        }
    }
    pool.session = None;
    pool
}

fn mount_with_switches(parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters, image: &MemoryPool) -> (Result<singlefs_core::mount::Mounted, MountError>, MemoryPool) {
    let mut devices = plain_devices_on(image);
    let result = mount_writable_with_test_only_switches_and_deletion_switches(
        parameters, &mut devices, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches_of_one());
    let after = MemoryPool {
        devices: devices.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: image.device_size_in_bytes,
    };
    (result, after)
}

fn apply(base: &MemoryPool, writes: &[RetainedWrite]) -> MemoryPool {
    let mut image = base.clone();
    for write in writes {
        write.contents.apply_to(image.devices.get_mut(&write.device).unwrap(), write.offset);
    }
    image
}

fn run_case(label: &str, unit_area: u64, overwrites: usize, units: usize, batches: usize) {
    let pool = crashed_pool(unit_area, overwrites, units, batches);
    let parameters = pool.parameters.clone();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    };
    let base_before_mount = pool.image();
    // 录这一次挂载的整条流。
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = plain_devices_on(&base_before_mount)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &parameters, &mut recorded, SpaceAdmission::JudgedByTheFormula, ShadowLedger::On, switches_of_one());
    let mounted = match mounted {
        Ok(m) => m,
        Err(e) => { println!("E6 {label} uncrashed mount failed: {e:?}"); return; }
    };
    let output = &mounted.output;
    let ops = stream.retained_operations();
    let (all_writes, all_segments) = writes_and_segments(&ops, &geometry);
    println!(
        "E6 {label} uncrashed mount: continuation_steps={} continuation_raises={:?} whole_mount_stream writes={} segments={} closed_form_states={}",
        output.deletion_queue_continuation.len(),
        output.floor_raises_pushed_by_the_continuation.iter().map(|r| r.publishes.len()).collect::<Vec<_>>(),
        all_writes.len(), all_segments.len(), closed_form_state_count(&all_segments)
    );
    if output.floor_raises_pushed_by_the_continuation.is_empty() {
        println!("E6 {label} no raise in the continuation; skip");
        return;
    }
    // 每次发布一次根槽写：按 txg 排出挂载里每次发布，与流里的根槽写一一对应。
    let mut publish_txgs: Vec<(u64, &'static str)> = Vec::new();
    publish_txgs.push((output.row_publish.root().checkpoint_txg.0, "row"));
    for w in &output.warm_up_publishes { publish_txgs.push((w.root().checkpoint_txg.0, "warm")); }
    for r in output.space_admission.floor_raises() { for p in &r.publishes { publish_txgs.push((p.root.checkpoint_txg.0, "mount-raise")); } }
    for s in &output.deletion_queue_continuation { publish_txgs.push((s.root.checkpoint_txg.0, "step")); }
    for r in &output.floor_raises_pushed_by_the_continuation { for p in &r.publishes { publish_txgs.push((p.root.checkpoint_txg.0, "cont-raise")); } }
    publish_txgs.sort();
    let root_ops: Vec<usize> = ops.iter().enumerate().filter(|(_, op)| geometry.classify(&op.operation) == StepKind::RootRecordFua).map(|(i, _)| i).collect();
    assert_eq!(root_ops.len(), publish_txgs.len(), "每次发布一次根槽写");
    let first_raise = publish_txgs.iter().position(|(_, k)| *k == "cont-raise").unwrap();
    let last_raise = publish_txgs.iter().rposition(|(_, k)| *k == "cont-raise").unwrap();
    // 段：续做里推的那一串之前那一步的根槽写之后起，到那一串之后的下一步（续做的下一步）根槽写与它后面的系统配置槽写为止。
    let start = root_ops[first_raise - 1] + 1;
    let include_next_step = std::env::var("OPUS_INCLUDE_NEXT_STEP").is_ok();
    let mut end = if include_next_step && last_raise + 1 < root_ops.len() { root_ops[last_raise + 1] + 1 } else { root_ops[last_raise] + 1 };
    while end < ops.len() && matches!(geometry.classify(&ops[end].operation), StepKind::SystemConfigurationSlot) { end += 1; }
    let kinds_in_piece: Vec<&str> = publish_txgs[first_raise - 1..=(if include_next_step { last_raise + 1 } else { last_raise }).min(publish_txgs.len() - 1)].iter().map(|(_, k)| *k).collect();
    let (before_writes, _) = writes_and_segments(&ops[..start], &geometry);
    let base = apply(&base_before_mount, &before_writes);
    let (writes, segments) = writes_and_segments(&ops[start..end], &geometry);
    let judged_root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).unwrap();
    let states = layer0_state_count_with_torn_in_place_overwrites(&base, &writes, &segments, &full_expansion);
    println!("E6 {label} piece publishes={kinds_in_piece:?} ops=[{start},{end}) writes={} segments={} closed_form={} states_with_torn={states}", writes.len(), segments.len(), closed_form_state_count(&segments));
    if states > STATES_AT_MOST { println!("E6 {label} piece too large; skip"); return; }
    let dump_dir = std::env::var("OPUS_DUMP_DIR").unwrap_or_else(|_| "/tmp/claude-1000/m3-unlink-code-r1-opus/e6-dump".to_string());
    let phase = std::env::var("OPUS_PHASE").unwrap_or_else(|_| "both".to_string());
    if phase == "remount" {
        let range: Vec<usize> = std::env::var("OPUS_RANGE").unwrap().split(',').map(|v| v.parse().unwrap()).collect();
        let bytes = std::fs::read(format!("{dump_dir}/table.bin")).unwrap();
        let mut loaded: Vec<RetainedWrite> = Vec::new();
        let mut at = 0usize;
        let mut index = 0usize;
        while at < bytes.len() {
            let device = u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap()); at += 4;
            let offset = u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()); at += 8;
            let tag = bytes[at]; at += 1;
            let length = u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()) as usize; at += 8;
            let contents = if tag == 0 { let c = bytes[at..at + length].to_vec(); at += length; singlefs_harness::memory_pool::WrittenContents::Bytes(c) } else { singlefs_harness::memory_pool::WrittenContents::Zeros { length: length as u64 } };
            let kind = StepKind::UnitWrite; let _ = index;
            loaded.push(RetainedWrite { device: DeviceIdentity(device), kind, is_force_unit_access: false, offset: singlefs_core::address::DeviceOffsetInBytes(offset), contents });
            index += 1;
        }
        let file = std::env::var("OPUS_PERSISTED_FILE").unwrap();
        let text = std::fs::read_to_string(format!("{dump_dir}/{file}")).unwrap();
        let persisted_sets: Vec<Vec<bool>> = text.lines().skip(range[0]).take(range[1] - range[0]).map(|l| l.chars().map(|c| c == '1').collect()).collect();
        println!("E6 {label} remount states [{},{}) of the dump (table {} writes)", range[0], range[0] + persisted_sets.len(), loaded.len());
        remount_all(&label, &base, &loaded, &persisted_sets, &parameters);
        return;
    }
    let mut table: Option<Vec<RetainedWrite>> = None;
    let mut persisted_sets: Vec<Vec<bool>> = Vec::new();
    let mut outcomes: BTreeMap<String, u64> = BTreeMap::new();
    let mut collect = |image: &singlefs_harness::memory_pool::CrashImage<'_>, report: &singlefs_core::recovery::RecoveryReport, _c: &mut singlefs_checker_tier::crash::Layer0ObserverCounts| {
        table.get_or_insert_with(|| image.writes.to_vec());
        persisted_sets.push(image.persisted.clone());
        let kind = match &report.outcome {
            singlefs_core::recovery::RecoveryOutcome::NoFile { .. } => "NoFile".to_string(),
            singlefs_core::recovery::RecoveryOutcome::FileRead { .. } => "FileRead".to_string(),
            singlefs_core::recovery::RecoveryOutcome::Failed { failure, .. } => format!("Failed({failure:?})").chars().take(80).collect(),
        };
        *outcomes.entry(kind).or_default() += 1;
    };
    let started = std::time::Instant::now();
    let shard_text = std::env::var("OPUS_SHARD").ok();
    let tally = match &shard_text {
        None => enumerate_layer0_in_state_slices(&base, &writes, &segments, judged_root_index, &[], &full_expansion,
            Layer0Parallelism::from_environment(), Some(&mut collect), &Layer0Resume::NoProgressFile),
        Some(text) => {
            use singlefs_checker_tier::layer0_progress::*;
            let parts: Vec<u32> = text.split('/').map(|v| v.parse().unwrap()).collect();
            let progress_dir = format!("{dump_dir}/progress");
            std::fs::create_dir_all(&progress_dir).unwrap();
            let resume = Layer0Resume::RunOneShardKeepingProgressFile(Layer0ShardRun {
                progress: Layer0ProgressFileSettings {
                    directory: progress_dir.into(),
                    input_fingerprint: Layer0ProgressFileNamePart::new("opuse6").unwrap(),
                    stream_name: Layer0ProgressFileNamePart::new("continuationraise").unwrap(),
                    start: Layer0ResumeStart::ResumeFromTheProgressFile,
                    after_completion: Layer0ProgressFileAfterCompletion::Deleted,
                },
                shard: Layer0ShardOfShards::new(parts[0], std::num::NonZeroU32::new(parts[1]).unwrap()).unwrap(),
                toolchain: Layer0ToolchainIdentity { rustc_version_lines: "opus".into(), cargo_version: "opus".into(), target_triple: "opus".into() },
            });
            match singlefs_checker_tier::crash::enumerate_layer0_in_state_slices_or_one_shard(&base, &writes, &segments, judged_root_index, &[], &full_expansion,
                Layer0Parallelism::from_environment(), Some(&mut collect), &resume) {
                singlefs_checker_tier::crash::Layer0EnumerationOutcome::OneShardWrittenToItsLedger(written) => written.tally_of_this_shard,
                singlefs_checker_tier::crash::Layer0EnumerationOutcome::WholeStream(t) => t,
            }
        }
    };
    println!("E6 {label} enumerated states={} oracle_violations={} first={:?} checker_violated_states={} checker_first={:?} recovery_outcomes={outcomes:?} secs={:.1}",
        tally.states, tally.violations, tally.first_violation, tally.checker_violated_states.values().sum::<u64>(), tally.checker_first_violation, started.elapsed().as_secs_f64());
    let table = table.unwrap();
    if phase == "enumerate" {
        std::fs::create_dir_all(&dump_dir).unwrap();
        let mut table_bytes: Vec<u8> = Vec::new();
        for w in &table {
            table_bytes.extend_from_slice(&w.device.0.to_le_bytes());
            table_bytes.extend_from_slice(&w.offset.0.to_le_bytes());
            match w.contents.as_bytes() {
                Some(b) => { table_bytes.push(0); table_bytes.extend_from_slice(&(b.len() as u64).to_le_bytes()); table_bytes.extend_from_slice(b); }
                None => { table_bytes.push(1); table_bytes.extend_from_slice(&w.contents.length_in_bytes().to_le_bytes()); }
            }
        }
        std::fs::write(format!("{dump_dir}/table.bin"), table_bytes).unwrap();
        let lines: Vec<String> = persisted_sets.iter().map(|p| p.iter().map(|b| if *b { '1' } else { '0' }).collect()).collect();
        let shard_suffix = shard_text.clone().unwrap_or_default().replace('/', "of");
        std::fs::write(format!("{dump_dir}/persisted{shard_suffix}.txt"), lines.join("\n")).unwrap();
        println!("E6 {label} dumped table={} states={} to {dump_dir}", table.len(), persisted_sets.len());
        return;
    }
    remount_all(&label, &base, &table, &persisted_sets, &parameters);
}

fn remount_all(label: &str, base: &MemoryPool, table: &[RetainedWrite], persisted_sets: &[Vec<bool>], parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters) {
    let threads = Layer0Parallelism::from_environment().worker_threads.get();
    let chunk = persisted_sets.len().div_ceil(threads).max(1);
    let results: Vec<(String, Option<String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = persisted_sets.chunks(chunk).map(|part| {
            let base = base; let table = table; let parameters = parameters;
            scope.spawn(move || {
                part.iter().map(|persisted| {
                    let image = apply(base, &table.iter().zip(persisted).filter(|(_, p)| **p).map(|(w, _)| w.clone()).collect::<Vec<_>>());
                    let (result, after) = mount_with_switches(parameters, &image);
                    match result {
                        Ok(m) => {
                            let queue = match &m.current { PoolVersion::WithFile(v) | PoolVersion::WithTreesAndNoFile(v) => v.deleted_inodes.len(), PoolVersion::WithoutFile(_) => 0 };
                            let red = checker_violations_on(&after);
                            (format!("Mounted(queue={queue}, raises={})", m.output.floor_raises_pushed_by_the_continuation.len()), red.first().cloned())
                        }
                        Err(MountError::Publish(f)) => (format!("MountError::Publish({})", format!("{:?}", f.cause).chars().take(40).collect::<String>()), checker_violations_on(&after).first().cloned()),
                        Err(e) => {
                            let detail = if std::env::var("OPUS_DETAIL").is_ok() {
                                let ro = singlefs_core::mounted_read::mount_read_only(&image).map(|m| (m.effective_root.instance.0, m.effective_root.checkpoint_txg.0)).map_err(|e| format!("{e:?}").chars().take(60).collect::<String>());
                                let (second, _) = mount_with_switches(parameters, &image);
                                let second = match second { Ok(_) => "second-mount-ok".to_string(), Err(e2) => format!("second-mount-err {}", format!("{e2:?}").chars().take(60).collect::<String>()) };
                                let bits: String = persisted.iter().map(|b| if *b { '1' } else { '0' }).collect();
                                Some(format!("DETAIL ro={ro:?} {second} persisted={bits} full_error={}", format!("{e:?}").chars().take(600).collect::<String>()))
                            } else { None };
                            (format!("{e:?}").chars().take(80).collect(), detail)
                        }
                    }
                }).collect::<Vec<_>>()
            })
        }).collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let mut by_kind: BTreeMap<String, u64> = BTreeMap::new();
    let mut reds = 0; let mut first_red = None;
    for (kind, red) in &results {
        *by_kind.entry(kind.clone()).or_default() += 1;
        if let Some(r) = red {
            if r.starts_with("DETAIL") { println!("E6 {label} {kind} {r}"); continue; }
            reds += 1; first_red.get_or_insert(r.clone());
        }
    }
    println!("E6 {label} remount outcomes={by_kind:?} post_mount_checker_red={reds} first={first_red:?}");
}

fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    match which.as_str() {
        "c1" => run_case("C1 narrow256 P2 U14 crash-after-intent", 256, 2, 14, 0),
        "c1b" => run_case("C1b narrow256 P2 U14 crash-after-batch1", 256, 2, 14, 1),
        other => {
            let parts: Vec<u64> = other.split(',').filter_map(|p| p.parse().ok()).collect();
            if parts.len() == 4 { run_case(&format!("custom{parts:?}"), parts[0], parts[1] as usize, parts[2] as usize, parts[3] as usize); }
            else { println!("usage: c1 | c1b | unit_area,overwrites,units,batches"); }
        }
    }
}

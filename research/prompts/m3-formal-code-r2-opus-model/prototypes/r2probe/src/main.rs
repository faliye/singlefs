//! m3-formal-code-r2 攻方原型（只在冻结副本的拷贝上编、跑；模型目录里留一份源码）。
//! 模式：
//!   sizes <历史…>                     每段历史窗口的写数、段数、closed_form_state_count、含撕裂第三态的层 0 全量态数
//!   k2 <历史> <线程> [起 止] [二次]     窗口里层 0 全量枚举的每个崩溃状态上照崩溃注入的接法跑挂载透明（只挂载、挂载加一次发布）
//!                                     与拒绝合法；「二次」= all 时每个状态再把那次挂载的写流按层 0 全量枚举成二次崩溃状态逐个判
//!   k3slot                            全池没有可用槽 0 时，checker 与恢复逐档找槽 1 的结局（手放的几种槽）
//!   k3mkfs                            新池新建文件那条路上 mkfs 占几步、清零几次（crates 今天的 mkfs）
//! 历史全是本原型手写的（不取自任何层 0 用例）；枚举用 `judge_layer0_state_range` + `full_expansion`（与 `enumerate_layer0_versions`
//! 同一份计划：每段都展开、原地覆写取第三态），状态数用 `layer0_state_count_with_torn_in_place_overwrites` 与 `closed_form_state_count`。
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Mutex;

use singlefs_checker::image::{chosen_system_configurations, system_configuration_slot_readings, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    full_expansion, judge_layer0_state_range, layer0_state_count_with_torn_in_place_overwrites,
    layer0_state_ranges_by_segment,
};
use singlefs_checker_tier::crash_injection::{
    materialized_crash_image, mount_and_publish_once_on_the_crash_image, CrashPoint, WritableMountOutcome,
    WritableMountPhase,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::make_filesystem::{make_filesystem, MakeFilesystemParameters};
use singlefs_core::recovery::{choose_system_configuration, readable_roots, PoolReader, RecoveryOutcome};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::{SystemImmutableSizes, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker, RollbackTargetChoice,
    StepPosition,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, newest_persisted_root, writes_and_segments_with_stream_indexes, CrashImage, MemoryPool,
    PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::model::{crash_recovery_disagreement, ModelRootKey};
use singlefs_harness::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use singlefs_harness::mount_transparency::{
    judge_mount_transparency, judge_writable_mount_refusal, read_recovery_fields,
    ComparedVersion, DiskAfterTheRefusal, FieldOutcome, FloorRaisedByTheMount, FloorRaisesAtTheSecondCrash,
    MountTransparency, MountTransparencyAction, ReadOnlyReadBackOfTheCrashImage, RecoveryFieldsRead,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

fn small(fill_seed: u64) -> ContentChoice {
    ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 200 }, fill_seed }
}
fn big(fill_seed: u64) -> ContentChoice {
    ContentChoice { length: ContentLength::OneByteBelowDataUnitPayloadCapacity, fill_seed }
}

struct Spec {
    starting_point: HistoryStartingPoint,
    operations: Vec<HistoryOperation>,
    /// None = 只有 mkfs 当基线；Some(k) = 起点与前 k 步当基线，之后的步是窗口。
    prefix: Option<usize>,
    width: HistoryDeviceWidth,
}

fn spec(id: &str) -> Spec {
    use HistoryOperation as O;
    let w = HistoryDeviceWidth::UnitAreaOf384Slots;
    let p = |starting_point, operations, prefix| Spec { starting_point, operations, prefix, width: w };
    if let Some(rest) = id.strip_prefix("wrap") {
        // wrap<N>：N 次覆盖写转过根环之后，窗口是再一次覆盖写。
        let n: usize = rest.parse().expect("N");
        let mut ops: Vec<HistoryOperation> = (0..n).map(|i| O::PublishOverwrite(small(30 + i as u64))).collect();
        ops.push(O::PublishOverwrite(small(99)));
        return p(HistoryStartingPoint::AfterFirstFile, ops, Some(n));
    }
    if let Some(rest) = id.strip_prefix("fm256m") {
        // fm256m<N>：256 槽小盘上 N 次满单元覆盖写当基线，窗口是一次重挂（挂载自己推抬 F 的那一形）。
        let n: usize = rest.parse().expect("N");
        let mut ops: Vec<HistoryOperation> = (0..n).map(|i| O::PublishOverwrite(big(200 + i as u64))).collect();
        ops.push(O::CloseAndMountWritable);
        return Spec { starting_point: HistoryStartingPoint::AfterFirstFile, operations: ops, prefix: Some(n), width: HistoryDeviceWidth::UnitAreaOf256Slots };
    }
    if let Some(rest) = id.strip_prefix("rfx") {
        // rfx<N>-<s>：N 次覆盖写当基线，窗口是一次抬 F（steps_above_current_floor = s）；rfxm 再接一次重挂当窗口。
        let (n, st) = rest.split_once('-').expect("N-s");
        let n: usize = n.parse().expect("N");
        let st: u64 = st.parse().expect("s");
        let mut ops: Vec<HistoryOperation> = (0..n).map(|i| O::PublishOverwrite(small(40 + i as u64))).collect();
        ops.push(O::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: st }));
        return p(HistoryStartingPoint::AfterFirstFile, ops, Some(n));
    }
    if let Some(rest) = id.strip_prefix("rfm") {
        let (n, st) = rest.split_once('-').expect("N-s");
        let n: usize = n.parse().expect("N");
        let st: u64 = st.parse().expect("s");
        let mut ops: Vec<HistoryOperation> = (0..n).map(|i| O::PublishOverwrite(small(40 + i as u64))).collect();
        ops.push(O::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: st }));
        ops.push(O::CloseAndMountWritable);
        return p(HistoryStartingPoint::AfterFirstFile, ops, Some(n + 1));
    }
    match id {
        "ow" => p(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7))], Some(0)),
        "mnt" => p(HistoryStartingPoint::AfterFirstFile, vec![O::CloseAndMountWritable], Some(0)),
        "mkmnt" => p(HistoryStartingPoint::AfterMakeFilesystem, vec![O::CloseAndMountWritable], None),
        "first" => p(HistoryStartingPoint::AfterMakeFilesystem, vec![O::CloseAndMountWritable, O::PublishFirstFile(small(5))], Some(1)),
        "rb" => p(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 })], Some(1)),
        "rbm" => p(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::CloseAndMountWritable, O::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 2 })], Some(2)),
        "rf" => p(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::PublishOverwrite(small(8)), O::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 1 })], Some(2)),
        "rfmnt" => p(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::PublishOverwrite(small(8)), O::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 2 }), O::CloseAndMountWritable], Some(3)),
        "ab" => p(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::CrashRecoveryAbandoningTheNewestRoot], Some(1)),
        "ab2" => p(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::CrashRecoveryAbandoningTheNewestRoot, O::PublishOverwrite(small(9))], Some(2)),
        "abf" => p(HistoryStartingPoint::AfterMakeFilesystem, vec![O::CloseAndMountWritable, O::PublishFirstFile(small(5)), O::CrashRecoveryAbandoningTheNewestRoot], Some(2)),
        "abf2" => p(HistoryStartingPoint::AfterMakeFilesystem, vec![O::CloseAndMountWritable, O::PublishFirstFile(small(5)), O::CrashRecoveryAbandoningTheNewestRoot, O::PublishFirstFile(small(6))], Some(3)),
        "cold" => p(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::ColdStartRecover, O::CloseAndMountWritable], Some(2)),
        "puw" => p(HistoryStartingPoint::AfterMakeFilesystem, vec![O::CloseAndMountWritable, O::PublishWithoutUnits], Some(1)),
        other => panic!("未知历史 {other}"),
    }
}

fn execution_of(width: HistoryDeviceWidth) -> HistoryExecution {
    HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: width, space_admission: SpaceAdmission::JudgedByTheFormula }
}

struct Prepared {
    width: HistoryDeviceWidth,
    parameters: MakeFilesystemParameters,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    versions: Vec<PublishedVersion>,
    judged_root_index: usize,
    committed: BTreeMap<ModelRootKey, Option<Vec<u8>>>,
    prefix_newest: Option<ModelRootKey>,
}

fn newest_readable_root(pool: &MemoryPool) -> Option<ModelRootKey> {
    let sc = choose_system_configuration(pool).ok()?;
    let im = &sc.immutable;
    readable_roots(pool, &im.region_devices, &im.sizes, &im.filesystem_identifier)
        .iter()
        .map(|r| model_root_key(r.instance, r.checkpoint_txg))
        .max()
}

fn prepare(id: &str) -> Prepared {
    if id == "um" || id == "umow" || id.starts_with("fms") {
        return prepare_unmount_then_mount(id);
    }
    let s = spec(id);
    let h = GeneratedHistory { seed: HistorySeed(0), starting_point: s.starting_point, operations: s.operations.clone() };
    let parameters = s.width.parameters();
    let size = s.width.device_bytes();
    let stream = SharedStream::retaining_contents();
    let mut marks: Vec<usize> = Vec::new();
    let mut committed: BTreeMap<ModelRootKey, Option<Vec<u8>>> = BTreeMap::new();
    let run = execute_history_with(&h, execution_of(s.width), &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed.insert(key, file.map(|c| c.to_vec()));
        }
        marks.push(stream.operation_count());
    });
    for (key, file) in &run.committed_versions_at_the_end {
        committed.insert(*key, file.clone());
    }
    eprintln!("history {id}: ending {:?}", run.ending);
    for (i, o) in run.outcomes.iter().enumerate() {
        let text = format!("{o:?}");
        eprintln!("  step {i}: {}", text.chars().take(300).collect::<String>());
    }
    let ops = stream.retained_operations();
    let cut = match s.prefix {
        None => run.operations_written_by_make_filesystem,
        // marks[0] 是起点之后那一次；marks[k] 是第 k 步之后。
        Some(k) => marks[k],
    };
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], size);
    base.apply(&ops[..cut]);
    let geometry = s.width.fixed_geometry();
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&ops[cut..], &geometry);
    let Some(judged_root_index) = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua) else {
        panic!("history {id}: 窗口里没有根槽写（那一步没写根）");
    };
    let versions = committed
        .iter()
        .filter_map(|(key, content)| {
            content.as_ref().map(|c| PublishedVersion {
                instance: InstanceGeneration(u32::try_from(key.instance.0).expect("实例")),
                checkpoint_txg: CheckpointTxg(key.checkpoint_txg.0),
                content: c.clone(),
            })
        })
        .collect();
    let prefix_newest = newest_readable_root(&base);
    Prepared { width: s.width, parameters, base, writes, segments, versions, judged_root_index, committed, prefix_newest }
}

/// um：起点之后（umow 再加一次覆盖写）不录，挂载、正常卸载（打卸载记号），再录一次可写挂载当窗口——R1 的那一版是卸载写的。
fn prepare_unmount_then_mount(id: &str) -> Prepared {
    use singlefs_core::mount::{mount_writable_with_space_admission, unmount};
    use singlefs_core::mounted_session::MountedSession;
    use singlefs_harness::RecordingBlockDevice;
    // fms<N>：256 槽小盘、前缀 N 次满单元覆盖写关掉空间准入跑（只供测试的开关，r5 攻方 fms 同一种造法），不卸载；
    // 窗口是一次判准入的可写挂载（挂载自己推抬 F 的那一形）。
    let fms: Option<usize> = id.strip_prefix("fms").map(|n| n.parse().expect("N"));
    let width = if fms.is_some() { HistoryDeviceWidth::UnitAreaOf256Slots } else { HistoryDeviceWidth::UnitAreaOf384Slots };
    let operations = match fms {
        Some(n) => (0..n).map(|i| HistoryOperation::PublishOverwrite(big(300 + i as u64))).collect(),
        None if id == "umow" => vec![HistoryOperation::PublishOverwrite(small(7))],
        None => vec![],
    };
    let h = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterFirstFile, operations };
    let parameters = width.parameters();
    let size = width.device_bytes();
    let stream = SharedStream::retaining_contents();
    let mut committed: BTreeMap<ModelRootKey, Option<Vec<u8>>> = BTreeMap::new();
    let mut prefix_execution = execution_of(width);
    if fms.is_some() {
        prefix_execution.space_admission = SpaceAdmission::SkippedByTheTestOnlySwitch;
    }
    let run = execute_history_with(&h, prefix_execution, &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed.insert(key, file.map(|c| c.to_vec()));
        }
    });
    eprintln!("history {id}: ending {:?}", run.ending);
    let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], size);
    pool.apply(&stream.retained_operations());
    let to_devices = |pool: &MemoryPool| -> Vec<(DeviceIdentity, SparseBlockDevice)> {
        pool.devices.iter().map(|(i, sectors)| {
            let mut d = SparseBlockDevice::new(size, PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size));
            d.image = sectors.clone();
            (*i, d)
        }).collect()
    };
    let base = if fms.is_some() {
        pool.clone()
    } else {
        let mut devices = to_devices(&pool);
        let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula).expect("挂载");
        let (_, mut session) = MountedSession::of_the_mount(mounted);
        let unmounted = unmount(&parameters, &mut devices, &mut session.allocator, &mut session.current, session.shadow_ledger).expect("卸载");
        match &unmounted {
            singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(r) => eprintln!("  unmount: F={:?} publishes={}", r.rollback_floor, r.publishes.len()),
            _ => eprintln!("  unmount: 没写字节"),
        }
        MemoryPool { devices: devices.into_iter().map(|(i, d)| (i, d.image)).collect(), device_size_in_bytes: size }
    };
    // 卸载之后的读回内容就是卸载之前的现行内容：基线上每条可读根都记成那一份（这段只挂载卸载，不改内容）。
    let content = match &singlefs_core::recovery::recover(&base, singlefs_core::recovery::JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => Some(content.clone()),
        _ => None,
    };
    if let Ok(sc) = choose_system_configuration(&base) {
        let im = &sc.immutable;
        for r in readable_roots(&base, &im.region_devices, &im.sizes, &im.filesystem_identifier) {
            committed.insert(model_root_key(r.instance, r.checkpoint_txg), content.clone());
        }
    }
    let window = SharedStream::retaining_contents();
    let mut recording: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = to_devices(&base)
        .into_iter()
        .map(|(i, d)| (i, RecordingBlockDevice::with_shared_stream(i, d, window.clone())))
        .collect();
    let mounted = mount_writable_with_space_admission(&parameters, &mut recording, SpaceAdmission::JudgedByTheFormula).expect("窗口挂载");
    eprintln!("  window mount: instance={:?} floor_raises={}", mounted.output.instance, mounted.output.space_admission.floor_raises().len());
    drop(mounted);
    let geometry = width.fixed_geometry();
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&window.retained_operations(), &geometry);
    let judged_root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("窗口挂载写根");
    let mut final_pool = base.clone();
    final_pool.apply_writes(&writes);
    if let Ok(sc) = choose_system_configuration(&final_pool) {
        let im = &sc.immutable;
        for r in readable_roots(&final_pool, &im.region_devices, &im.sizes, &im.filesystem_identifier) {
            committed.insert(model_root_key(r.instance, r.checkpoint_txg), content.clone());
        }
    }
    let versions = committed.iter().filter_map(|(key, c)| c.as_ref().map(|c| PublishedVersion {
        instance: InstanceGeneration(u32::try_from(key.instance.0).expect("实例")),
        checkpoint_txg: CheckpointTxg(key.checkpoint_txg.0),
        content: c.clone(),
    })).collect();
    let prefix_newest = newest_readable_root(&base);
    Prepared { width, parameters, base, writes, segments, versions, judged_root_index, committed, prefix_newest }
}

#[derive(Default)]
struct Tally {
    states: u64,
    second_states: u64,
    classes: BTreeMap<String, u64>,
    examples: BTreeMap<String, Vec<String>>,
}

impl Tally {
    fn add(&mut self, class: String, example: impl FnOnce() -> String) {
        *self.classes.entry(class.clone()).or_default() += 1;
        let e = self.examples.entry(class).or_default();
        if e.len() < 2 {
            e.push(example());
        }
    }
    fn merge(&mut self, other: Tally) {
        self.states += other.states;
        self.second_states += other.second_states;
        for (k, v) in other.classes {
            *self.classes.entry(k).or_default() += v;
        }
        for (k, ex) in other.examples {
            let e = self.examples.entry(k).or_default();
            for x in ex {
                if e.len() < 2 {
                    e.push(x);
                }
            }
        }
    }
}

fn record_transparency(tally: &mut Tally, label: &str, t: &MountTransparency, example: &dyn Fn() -> String) {
    match t {
        MountTransparency::NotJudged(reason) => tally.add(format!("{label} not_judged={}", reason.name()), || String::new()),
        MountTransparency::Judged(verdicts) => {
            tally.add(format!("{label} judged"), || String::new());
            for v in verdicts {
                match &v.outcome {
                    FieldOutcome::Holds => {}
                    FieldOutcome::NotJudged(r) => tally.add(format!("{label} field_not_judged={}", r.name()), || String::new()),
                    FieldOutcome::Violated { expected, observed } => {
                        let name = v.field.violation_name(v.basis);
                        tally.add(format!("{label} RED {name}"), || format!("{} | expected {expected} | observed {observed}", example()));
                    }
                }
            }
        }
    }
}

fn floor_raises_at(floor: FloorRaisedByTheMount, phase: WritableMountPhase) -> FloorRaisesAtTheSecondCrash {
    match (floor, phase) {
        (FloorRaisedByTheMount::RaisedTo(_), WritableMountPhase::WarmUp) => FloorRaisesAtTheSecondCrash::SomeMayHaveLanded,
        _ => FloorRaisesAtTheSecondCrash::NoneLandedYet,
    }
}

/// 与崩溃注入 `writable_mount_phase_of_each_write` 同一个分段（那一份是私有的，照它的规则重写）。
fn phase_of_each_write(writes: &[RetainedWrite]) -> Vec<WritableMountPhase> {
    let mut phase = WritableMountPhase::Acquisition;
    let mut row_root = false;
    writes
        .iter()
        .map(|w| {
            phase = match (phase, w.kind) {
                (WritableMountPhase::Acquisition, StepKind::SystemConfigurationSlot) => WritableMountPhase::Acquisition,
                (WritableMountPhase::Acquisition, _) => WritableMountPhase::RowPublish,
                (WritableMountPhase::RowPublish, StepKind::SystemConfigurationSlot) => WritableMountPhase::RowPublish,
                (WritableMountPhase::RowPublish, _) => if row_root { WritableMountPhase::WarmUp } else { WritableMountPhase::RowPublish },
                (WritableMountPhase::WarmUp, _) => WritableMountPhase::WarmUp,
            };
            if phase == WritableMountPhase::RowPublish && w.kind == StepKind::RootRecordFua {
                row_root = true;
            }
            phase
        })
        .collect()
}

fn key_of(fields: &RecoveryFieldsRead) -> String {
    let v = fields.compared_version.as_ref().map(|v| (v.root.instance.0, v.root.checkpoint_txg.0, v.root.tree_identifier_watermark, v.root.rollback_floor.0));
    format!("eff={:?} cmp(inst,txg,W,F)={v:?} applied={} above={} sc_inst={:?} ring_inst={:?} ring_txg={:?} rec_txg={:?} F={:?}",
        fields.report.effective_root.map(|(i, t)| (i.0, t.0)), fields.report.journal.prefix_applied, fields.report.journal.above_water,
        fields.highest_system_configuration_instance.map(|i| i.0), fields.highest_root_instance_in_the_ring.map(|i| i.0),
        fields.highest_root_txg_in_the_ring.map(|t| t.0), fields.highest_record_checkpoint_txg_in_the_ring.map(|t| t.0),
        fields.effective_rollback_floor.map(|t| t.0))
}

static SECOND_SPENT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn judge_k2(ordinal: u64, image: &CrashImage<'_>, p: &Prepared, committed: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>, second: bool, tally: &mut Tally) {
    tally.states += 1;
    let crash = materialized_crash_image(image);
    let r1 = read_recovery_fields(&crash, ComparedVersion::TheRecoveredVersion);
    let read_back = observed_read_back_after_a_crash(&r1.report);
    let newest = newest_persisted_root(image.writes, &image.persisted)
        .map(|(t, i)| model_root_key(i, t))
        .max(p.prefix_newest);
    let disagreement = crash_recovery_disagreement(committed, newest, &read_back);
    let r1_class = match &r1.report.outcome {
        RecoveryOutcome::FileRead { .. } => "file",
        RecoveryOutcome::NoFile { .. } => "no_file",
        RecoveryOutcome::Failed { .. } => "failed",
    };
    tally.add(format!("R1 outcome={r1_class} rebuilt={} o1={}", r1.report.journal.prefix_applied > 0, if disagreement.is_some() { "red" } else { "green" }), || String::new());
    let crash_point = CrashPoint { segment_index: 0, persisted_within_the_segment: Vec::new(), step: StepPosition::StartingPoint, operation_kind: None };
    let execution = execution_of(p.width);
    let mount_run = mount_and_publish_once_on_the_crash_image(&crash, &p.parameters, execution, HistorySeed(0), &crash_point);
    let geometry = p.width.fixed_geometry();
    let (mount_writes, mount_segments, _) = writes_and_segments_with_stream_indexes(&mount_run.mount_operations, &geometry);
    let example = || format!("ordinal={ordinal} R1: {}", key_of(&r1));
    match &mount_run.outcome {
        WritableMountOutcome::MountRefused { member, refusal, .. } => {
            let disk = if mount_run.pool_after == crash { DiskAfterTheRefusal::ByteForByteUnchanged } else { DiskAfterTheRefusal::Changed };
            let ro = if disagreement.is_none() { ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet } else { ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet };
            let verdict = judge_writable_mount_refusal(&crash, r1.report.effective_root, refusal, disk, ro);
            let short: String = member.chars().take(90).collect();
            tally.add(format!("MOUNT refused verdict={verdict:?} member={short}"), example);
            return;
        }
        WritableMountOutcome::PublishRefused { member, .. } => {
            let short: String = member.chars().take(90).collect();
            tally.add(format!("MOUNT ok publish_refused={short}"), || String::new());
        }
        WritableMountOutcome::Published => {}
    }
    let floor = mount_run.floor_raised_by_the_mount.expect("挂载做成就有交回值");
    tally.add(format!("MOUNT floor={floor:?}"), || String::new());
    let mut after_mount = crash.clone();
    after_mount.apply_writes(&mount_writes);
    let r2a = read_recovery_fields(&after_mount, ComparedVersion::TheRecoveredVersion);
    let t = judge_mount_transparency(&r1, &r2a, MountTransparencyAction::MountAlone(floor));
    record_transparency(tally, "ALONE", &t, &|| format!("{} | R2: {}", example(), key_of(&r2a)));
    if matches!(mount_run.outcome, WritableMountOutcome::Published) {
        let r2b = read_recovery_fields(&mount_run.pool_after, ComparedVersion::TheVersionBeforeTheRecoveredOne);
        let t = judge_mount_transparency(&r1, &r2b, MountTransparencyAction::MountAndOnePublish(floor));
        record_transparency(tally, "PUBLISH", &t, &|| format!("{} | R2: {}", example(), key_of(&r2b)));
    }
    if !second {
        return;
    }
    if std::env::var("R2_ONLY_RAISED").is_ok() && floor == FloorRaisedByTheMount::NotRaised {
        return;
    }
    let phases = phase_of_each_write(&mount_writes);
    let Some(root_index) = mount_writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua) else { return };
    let (ranges, _total2) = layer0_state_ranges_by_segment(&crash, &mount_writes, &mount_segments, &full_expansion);
    let total2 = layer0_state_count_with_torn_in_place_overwrites(&crash, &mount_writes, &mount_segments, &full_expansion);
    let cap: u64 = std::env::var("R2_SECOND_CAP").ok().and_then(|v| v.parse().ok()).unwrap_or(5000);
    let sizes: Vec<usize> = mount_segments.iter().map(Vec::len).collect();
    if total2 > cap {
        tally.add(format!("SECOND skipped_over_cap segments={sizes:?} states={total2}"), || String::new());
        return;
    }
    let budget: u64 = std::env::var("R2_SECOND_BUDGET").ok().and_then(|v| v.parse().ok()).unwrap_or(u64::MAX);
    let before = SECOND_SPENT.fetch_add(total2, std::sync::atomic::Ordering::SeqCst);
    if before + total2 > budget {
        SECOND_SPENT.fetch_sub(total2, std::sync::atomic::Ordering::SeqCst);
        tally.add(format!("SECOND skipped_over_budget segments={sizes:?} states={total2} floor={floor:?}"), || format!("ordinal={ordinal}"));
        return;
    }
    tally.add(format!("SECOND enumerated segments={sizes:?} states={total2} floor={floor:?}"), || format!("ordinal={ordinal}"));
    let _ = judge_layer0_state_range(&crash, &mount_writes, &mount_segments, root_index, &[], &full_expansion, 0..total2, &mut |o2, image2, _j| {
        tally.second_states += 1;
        // 这个状态起头在哪一段：状态序号落在哪一段的区间里；段内第一个没持久的原写定相位，都持久了（撕裂那一态）取段首写。
        let seg = ranges.iter().position(|r| r.contains(&o2));
        let phase = match seg {
            None => return, // 全持久那个终态：挂载做完了，不是挂载途中
            Some(s) => {
                let segment = &mount_segments[s];
                let first_withheld = segment.iter().find(|w| !image2.persisted.get(**w).copied().unwrap_or(true));
                phases[*first_withheld.unwrap_or(&segment[0])]
            }
        };
        let r2 = read_recovery_fields(image2, ComparedVersion::TheRecoveredVersion);
        let t = judge_mount_transparency(&r1, &r2, MountTransparencyAction::SecondCrashInsideTheMount(floor_raises_at(floor, phase)));
        let label = format!("SECOND[{}]", phase.name());
        record_transparency(tally, &label, &t, &|| format!("{} | second={o2} R2: {}", example(), key_of(&r2)));
    });
}

fn run_k2(id: &str, threads: usize, range: Option<(u64, u64)>, second: bool) {
    let started = std::time::Instant::now();
    let p = prepare(id);
    let total = layer0_state_count_with_torn_in_place_overwrites(&p.base, &p.writes, &p.segments, &full_expansion);
    let (from, to) = range.unwrap_or((0, total));
    let to = to.min(total);
    // R2_FIRST_PER_SEGMENT=k：第一次崩溃只取窗口每一段状态区间里的前 k 个序号（缩的是历史条数；留下的每个第一次崩溃状态，
    // 二次崩溃照层 0 全量枚举）。不设就是 [from, to) 整段。
    let picked: Option<Vec<u64>> = std::env::var("R2_FIRST_PER_SEGMENT").ok().and_then(|v| v.parse::<u64>().ok()).map(|k| {
        let (ranges, _) = layer0_state_ranges_by_segment(&p.base, &p.writes, &p.segments, &full_expansion);
        let mut v: Vec<u64> = ranges.iter().flat_map(|r| r.start..r.end.min(r.start + k)).filter(|o| (from..to).contains(o)).collect();
        v.sort_unstable();
        v
    });
    if let Some(v) = &picked {
        println!("first_states_picked={} per_segment={}", v.len(), std::env::var("R2_FIRST_PER_SEGMENT").unwrap_or_default());
    }
    println!("history={id} mode=k2 second={second} writes={} segments={} closed_form={} with_torn={total} range={from}..{to}", p.writes.len(), p.segments.len(), closed_form_state_count(&p.segments));
    let merged = Mutex::new(Tally::default());
    let chunk = (to - from).div_ceil(threads as u64).max(1);
    std::thread::scope(|scope| {
        for t in 0..threads as u64 {
            let a = from + t * chunk;
            let b = (from + (t + 1) * chunk).min(to);
            if a >= b {
                continue;
            }
            let p = &p;
            let merged = &merged;
            let picked = &picked;
            scope.spawn(move || {
                let mut tally = Tally::default();
                let committed: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = p.committed.iter().map(|(k, v)| (*k, v.as_ref().map(|c| Rc::from(c.as_slice())))).collect();
                match picked {
                    None => {
                        let _ = judge_layer0_state_range(&p.base, &p.writes, &p.segments, p.judged_root_index, &p.versions, &full_expansion, a..b, &mut |o, image, _j| {
                            judge_k2(o, image, p, &committed, second, &mut tally);
                        });
                    }
                    Some(v) => {
                        for (i, o) in v.iter().enumerate() {
                            if i as u64 % threads as u64 != t {
                                continue;
                            }
                            let _ = judge_layer0_state_range(&p.base, &p.writes, &p.segments, p.judged_root_index, &p.versions, &full_expansion, *o..*o + 1, &mut |o, image, _j| {
                                judge_k2(o, image, p, &committed, second, &mut tally);
                            });
                        }
                    }
                }
                merged.lock().expect("锁").merge(tally);
            });
        }
    });
    let tally = merged.into_inner().expect("锁");
    println!("states_judged={} second_states={} elapsed_secs={:.1}", tally.states, tally.second_states, started.elapsed().as_secs_f64());
    for (k, c) in &tally.classes {
        println!("{k} count={c}");
        for x in &tally.examples[k] {
            if !x.is_empty() {
                println!("    e.g. {x}");
            }
        }
    }
}

fn sizes(ids: &[String]) {
    for id in ids {
        let p = prepare(id);
        let total = layer0_state_count_with_torn_in_place_overwrites(&p.base, &p.writes, &p.segments, &full_expansion);
        println!("history={id} writes={} segments={} closed_form={} with_torn={total} committed={}", p.writes.len(), p.segments.len(), closed_form_state_count(&p.segments), p.committed.len());
    }
}

// ───────── K3：mkfs 占几步（与 E142 导出、E170 导出同一条路：e142_parameters(512, 512)、两块 4 GiB 内存稀疏盘） ─────────
fn run_k3mkfs() {
    use singlefs_harness::scenario::{e142_parameters, run_new_pool_file_creation};
    use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice};
    let parameters = e142_parameters(512, 512);
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|n| {
            let identity = DeviceIdentity(n);
            (identity, RecordingBlockDevice::with_shared_stream(identity, SparseBlockDevice::new(singlefs_format::TEST_IMAGE_DEFAULT_BYTES, PhysicalBlockSizeInBytes(512)), stream.clone()))
        })
        .collect();
    let run = run_new_pool_file_creation(&parameters, &mut devices, &stream, |_, _| {}).expect("整条路跑得通");
    let ops = stream.retained_operations();
    let mkfs = &ops[..run.mkfs_operation_count];
    let zero_fills: Vec<String> = mkfs
        .iter()
        .filter(|o| o.operation.kind == RecordedOperationKind::WriteZeroes)
        .map(|o| format!("d{}@{}+{}", o.operation.device.0, o.operation.offset.0, o.operation.length))
        .collect();
    let barriers = mkfs.iter().filter(|o| o.operation.kind == RecordedOperationKind::Barrier).count();
    println!("name=k3_mkfs mkfs_operations={} zero_fills={} barriers={} zero_fill_ranges={}", run.mkfs_operation_count, zero_fills.len(), barriers, zero_fills.join(","));
}

// ───────── K3：全池没有可用槽 0 时 checker 与恢复逐档找槽 1 的结局 ─────────
const K3_DEVICE_BYTES: u64 = 4 << 30;
const K3_FSID: [u8; 16] = *b"r2probe-k3-slots";
const SLOT: usize = SYSTEM_CONFIGURATION_SLOT_BYTES as usize;
const OFF_FORMAT_VERSION: usize = 4;
const OFF_INCOMPAT0: usize = 6;
const OFF_ENCRYPTION_TYPE: usize = 219;
const OFF_IN_FLIGHT_LIMIT: usize = 345;
const OFF_REGIONS: usize = 361;
const OFF_SLOTS_PER_REGION: usize = 362;
const OFF_SPACING: usize = 429;
const OFF_GENERATION: usize = 147;

fn k3_pool() -> MemoryPool {
    let parameters = MakeFilesystemParameters {
        filesystem_identifier: K3_FSID,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 8192,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(8192),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    };
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|n| (DeviceIdentity(n), SparseBlockDevice::new(K3_DEVICE_BYTES, PhysicalBlockSizeInBytes(512))))
        .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    MemoryPool { devices: devices.into_iter().map(|(i, d)| (i, d.image)).collect(), device_size_in_bytes: K3_DEVICE_BYTES }
}

fn reseal(slot: &mut [u8]) {
    let digest = wide_checksum_with_field_zeroed(slot, SLOT, SYSTEM_CONFIGURATION_CHECKSUM_OFFSET);
    slot[SYSTEM_CONFIGURATION_CHECKSUM_OFFSET..SYSTEM_CONFIGURATION_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
}

fn read_slot(image: &MemoryPool, device: u32, offset: u64) -> Vec<u8> {
    PoolReader::read(image, DeviceIdentity(device), DeviceOffsetInBytes(offset), SLOT).expect("读得到")
}

fn write_slot(image: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
    image.devices.get_mut(&DeviceIdentity(device)).expect("盘").write(DeviceOffsetInBytes(offset), bytes);
}

fn put_u32(b: &mut [u8], at: usize, v: u32) {
    b[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn put_u64(b: &mut [u8], at: usize, v: u64) {
    b[at..at + 8].copy_from_slice(&v.to_le_bytes());
}
fn get_u32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().expect("4"))
}
fn get_u64(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().expect("8"))
}

/// 一种手放的槽：在 mkfs 写下的槽 1（偏移 8192）上改几格、重封校验和，放到 `offset`（两块盘各放一份）。
struct Planted {
    name: &'static str,
    offset: u64,
    edit: fn(&mut [u8]),
}

fn outcome_line(image: &MemoryPool) -> String {
    let recovery = match choose_system_configuration(image) {
        Ok(sc) => format!("recovery=chose(dev{},spacing{},gen{})", sc.immutable.this_device.0, sc.immutable.sizes.fixed_structure_slot_spacing, sc.quantities.slot_generation),
        Err(e) => format!("recovery=refused({})", format!("{e:?}").chars().take(70).collect::<String>()),
    };
    let checker_chosen: Vec<String> = chosen_system_configurations(image)
        .into_iter()
        .map(|(d, c)| format!("dev{d}:{}", c.map_or("none".to_string(), |(v, g)| format!("spacing{},gen{}", g.slot_spacing, v.slot_generation))))
        .collect();
    let readings: Vec<String> = system_configuration_slot_readings(image)
        .into_iter()
        .map(|(d, s)| format!("dev{d}:slot1={}", format!("{:?}", s[1]).chars().take(48).collect::<String>()))
        .collect();
    let red: Vec<&str> = check_pool_image(image)
        .into_iter()
        .filter_map(|(inv, v)| matches!(v, InvariantVerdict::Violated(_)).then_some(inv))
        .collect();
    format!("{recovery} checker_chose=[{}] checker_red={red:?} readings=[{}]", checker_chosen.join(" "), readings.join(" "))
}

fn run_k3slot() {
    let pristine = k3_pool();
    let genuine = read_slot(&pristine, 0, 8192);
    println!("name=k3_genuine spacing={} gen={} in_flight={} regions={} slots_per_region={}", get_u32(&genuine, OFF_SPACING), get_u64(&genuine, OFF_GENERATION), get_u32(&genuine, OFF_IN_FLIGHT_LIMIT), genuine[OFF_REGIONS], genuine[OFF_SLOTS_PER_REGION]);
    let variants: Vec<Planted> = vec![
        Planted { name: "control_nothing_planted", offset: 0, edit: |_| {} },
        Planted { name: "S17_spacing_equals_offset_higher_gen", offset: 12288, edit: |b| { b[OFF_SLOTS_PER_REGION] = 17; put_u32(b, OFF_SPACING, 12288); put_u64(b, OFF_GENERATION, 9); } },
        Planted { name: "S17_spacing_8192_at_12288", offset: 12288, edit: |b| { b[OFF_SLOTS_PER_REGION] = 17; } },
        Planted { name: "S17_spacing_8192_at_4096_below_genuine", offset: 4096, edit: |b| { b[OFF_SLOTS_PER_REGION] = 17; } },
        Planted { name: "inflight_plus1_spacing_8192_at_12288", offset: 12288, edit: |b| { let v = get_u32(b, OFF_IN_FLIGHT_LIMIT); put_u32(b, OFF_IN_FLIGHT_LIMIT, v + 1); } },
        Planted { name: "inflight_plus1_spacing_equals_offset_gen0", offset: 12288, edit: |b| { let v = get_u32(b, OFF_IN_FLIGHT_LIMIT); put_u32(b, OFF_IN_FLIGHT_LIMIT, v + 1); put_u32(b, OFF_SPACING, 12288); put_u64(b, OFF_GENERATION, 0); } },
        Planted { name: "inflight_plus1_spacing_equals_offset_higher_gen", offset: 12288, edit: |b| { let v = get_u32(b, OFF_IN_FLIGHT_LIMIT); put_u32(b, OFF_IN_FLIGHT_LIMIT, v + 1); put_u32(b, OFF_SPACING, 12288); put_u64(b, OFF_GENERATION, 9); } },
        Planted { name: "R4_spacing_equals_offset_higher_gen", offset: 12288, edit: |b| { b[OFF_REGIONS] = 4; put_u32(b, OFF_SPACING, 12288); put_u64(b, OFF_GENERATION, 9); } },
        Planted { name: "R4_spacing_8192_at_12288", offset: 12288, edit: |b| { b[OFF_REGIONS] = 4; } },
        Planted { name: "incompat_unknown_and_format_version_2_at_12288", offset: 12288, edit: |b| { b[OFF_INCOMPAT0] |= 0x80; b[OFF_FORMAT_VERSION] = 2; } },
        Planted { name: "incompat_unknown_and_encryption_1_at_12288", offset: 12288, edit: |b| { b[OFF_INCOMPAT0] |= 0x80; b[OFF_ENCRYPTION_TYPE] = 1; } },
        Planted { name: "format_version_2_spacing_8192_at_12288", offset: 12288, edit: |b| { b[OFF_FORMAT_VERSION] = 2; } },
        Planted { name: "equal_gen_spacing_equals_offset_at_12288", offset: 12288, edit: |b| { put_u32(b, OFF_SPACING, 12288); } },
    ];
    for v in &variants {
        let mut image = pristine.clone();
        for d in 0..2u32 {
            write_slot(&mut image, d, 0, &vec![0u8; SLOT]);
            if v.offset != 0 {
                let mut slot = read_slot(&pristine, d, 8192);
                (v.edit)(&mut slot);
                reseal(&mut slot);
                write_slot(&mut image, d, v.offset, &slot);
            }
        }
        println!("name=k3_slot variant={} {}", v.name, outcome_line(&image));
    }
}

/// 对照：槽 0 还可用的普通读法（不逐档）上同几种槽。盘 0 的槽 0 换成改过的那一份（世代号抬到 9），别的不动。
fn run_k3slot0() {
    let pristine = k3_pool();
    let edits: Vec<(&str, fn(&mut [u8]))> = vec![
        ("slot0_dev0_S17_gen9", |b| { b[OFF_SLOTS_PER_REGION] = 17; put_u64(b, OFF_GENERATION, 9); }),
        ("slot0_dev0_inflight_plus1_gen9", |b| { let v = get_u32(b, OFF_IN_FLIGHT_LIMIT); put_u32(b, OFF_IN_FLIGHT_LIMIT, v + 1); put_u64(b, OFF_GENERATION, 9); }),
        ("slot0_dev0_inflight_plus1_gen0", |b| { let v = get_u32(b, OFF_IN_FLIGHT_LIMIT); put_u32(b, OFF_IN_FLIGHT_LIMIT, v + 1); put_u64(b, OFF_GENERATION, 0); }),
        ("slot0_dev0_R4_gen9", |b| { b[OFF_REGIONS] = 4; put_u64(b, OFF_GENERATION, 9); }),
        ("slot0_dev0_incompat_unknown_and_format_version_2", |b| { b[OFF_INCOMPAT0] |= 0x80; b[OFF_FORMAT_VERSION] = 2; }),
    ];
    for (name, edit) in edits {
        let mut image = pristine.clone();
        let mut slot = read_slot(&pristine, 0, 0);
        edit(&mut slot);
        reseal(&mut slot);
        write_slot(&mut image, 0, 0, &slot);
        println!("name=k3_slot0 variant={name} {}", outcome_line(&image));
    }
}

/// k2one：窗口里第 `ordinal` 个第一次崩溃状态，起一次挂载，把那次挂载的写流按层 0 全量枚举成二次崩溃状态，
/// 序号段 [from2, to2) 分给 `threads` 个线程判（一个第一次崩溃状态的二次崩溃太多时分几次跑）。
fn run_k2one(id: &str, ordinal: u64, threads: usize, from2: u64, to2: Option<u64>) {
    let started = std::time::Instant::now();
    let p = prepare(id);
    let mut crash: Option<MemoryPool> = None;
    let _ = judge_layer0_state_range(&p.base, &p.writes, &p.segments, p.judged_root_index, &p.versions, &full_expansion, ordinal..ordinal + 1, &mut |_o, image, _j| {
        crash = Some(materialized_crash_image(image));
    });
    let crash = crash.expect("这个序号有状态");
    let r1 = read_recovery_fields(&crash, ComparedVersion::TheRecoveredVersion);
    let crash_point = CrashPoint { segment_index: 0, persisted_within_the_segment: Vec::new(), step: StepPosition::StartingPoint, operation_kind: None };
    let mount_run = mount_and_publish_once_on_the_crash_image(&crash, &p.parameters, execution_of(p.width), HistorySeed(0), &crash_point);
    let Some(floor) = mount_run.floor_raised_by_the_mount else {
        println!("history={id} ordinal={ordinal} mount_refused");
        return;
    };
    let geometry = p.width.fixed_geometry();
    let (mount_writes, mount_segments, _) = writes_and_segments_with_stream_indexes(&mount_run.mount_operations, &geometry);
    let phases = phase_of_each_write(&mount_writes);
    let root_index = mount_writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("挂载写根");
    let (ranges, total2) = layer0_state_ranges_by_segment(&crash, &mount_writes, &mount_segments, &full_expansion);
    let to2 = to2.unwrap_or(total2).min(total2);
    let sizes: Vec<usize> = mount_segments.iter().map(Vec::len).collect();
    println!("history={id} ordinal={ordinal} floor={floor:?} R1: {} mount_segments={sizes:?} second_total={total2} range={from2}..{to2}", key_of(&r1));
    for (i, seg) in mount_segments.iter().enumerate() {
        let kinds: Vec<&str> = seg.iter().map(|w| mount_writes[*w].kind.name()).collect();
        println!("  mount_segment {i}: states={:?} phase_of_first={} kinds={kinds:?}", ranges[i], phases[seg[0]].name());
    }
    let merged = Mutex::new(Tally::default());
    let chunk = (to2 - from2).div_ceil(threads as u64).max(1);
    std::thread::scope(|scope| {
        for t in 0..threads as u64 {
            let a = from2 + t * chunk;
            let b = (from2 + (t + 1) * chunk).min(to2);
            if a >= b {
                continue;
            }
            let (crash, mount_writes, mount_segments, phases, ranges, r1, merged) = (&crash, &mount_writes, &mount_segments, &phases, &ranges, &r1, &merged);
            scope.spawn(move || {
                let mut tally = Tally::default();
                let _ = judge_layer0_state_range(crash, mount_writes, mount_segments, root_index, &[], &full_expansion, a..b, &mut |o2, image2, _j| {
                    tally.second_states += 1;
                    let Some(s) = ranges.iter().position(|r| r.contains(&o2)) else { return };
                    let segment = &mount_segments[s];
                    let first_withheld = segment.iter().find(|w| !image2.persisted.get(**w).copied().unwrap_or(true));
                    let phase = phases[*first_withheld.unwrap_or(&segment[0])];
                    let r2 = read_recovery_fields(image2, ComparedVersion::TheRecoveredVersion);
                    let t = judge_mount_transparency(r1, &r2, MountTransparencyAction::SecondCrashInsideTheMount(floor_raises_at(floor, phase)));
                    let label = format!("SECOND[{}] seg={s}", phase.name());
                    record_transparency(&mut tally, &label, &t, &|| format!("second={o2} R2: {}", key_of(&r2)));
                });
                merged.lock().expect("锁").merge(tally);
            });
        }
    });
    let tally = merged.into_inner().expect("锁");
    println!("second_states={} elapsed_secs={:.1}", tally.second_states, started.elapsed().as_secs_f64());
    for (k, c) in &tally.classes {
        println!("{k} count={c}");
        for x in &tally.examples[k] {
            if !x.is_empty() {
                println!("    e.g. {x}");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("sizes") => sizes(&args[2..]),
        Some("k3mkfs") => run_k3mkfs(),
        Some("k3slot") => run_k3slot(),
        Some("k3slot0") => run_k3slot0(),
        Some("k2one") => {
            let ordinal: u64 = args[3].parse().expect("序号");
            let threads: usize = args[4].parse().expect("线程");
            let from2: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
            let to2: Option<u64> = args.get(6).and_then(|s| s.parse().ok());
            run_k2one(&args[2], ordinal, threads, from2, to2);
        }
        Some("k2") => {
            let id = &args[2];
            let threads: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
            let range = match (args.get(4).and_then(|s| s.parse().ok()), args.get(5).and_then(|s| s.parse().ok())) {
                (Some(a), Some(b)) => Some((a, b)),
                _ => None,
            };
            let second = args.get(6).is_some_and(|s| s == "all");
            run_k2(id, threads, range, second);
        }
        _ => eprintln!("用法见文件头"),
    }
}

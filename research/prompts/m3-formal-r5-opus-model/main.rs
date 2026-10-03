//! m3-formal-r5 攻方腿原型（只在冻结副本的拷贝上编、跑；不进仓，模型目录里留一份源码）。
//! 枚举：`singlefs_checker_tier::crash::judge_layer0_state_range` + `full_expansion`，0..全量状态数（层 0 同一枚举域：
//! 每段都展开、原地覆写取第三态；与 `enumerate_layer0_versions` 同一份计划，只是逐状态把镜像与层 0 判定交回来）。
//! 历史是本原型手写的几段（不取自任何层 0 用例）。
//! 模式：
//!   scan <历史…>           只跑历史，打印每一步的结局与恢复读回（不枚举崩溃状态）
//!   sizes <历史>           状态数
//!   u3 <历史> <线程> [上限] 每个崩溃状态上：O1 判官全四条、只取 ③、RefFS-Q 替身、层 0 的判定，另起第二截（可写挂载 + 一次发布）跑记录核对器与 checker
//!   u5 <历史> <线程> [上限] 每个崩溃状态上：R1 = 只读恢复；动作 mount、mount+write 之后 R2；逐字段比（含第一轮之外的字段）
//! 副本变异：环境变量 R5_MUTATION=keep_tree_table 时 core 的 replay_journal 施加记录不换树表指针（只在这份副本的 recovery.rs 里）。
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Mutex;

use singlefs_checker::walk::check_pool_image;
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker_tier::crash::{
    full_expansion, judge_layer0_state_range, layer0_state_count_with_torn_in_place_overwrites,
    Layer0StateJudgement,
};
use singlefs_checker_tier::crash_injection::{
    materialized_crash_image, mount_and_publish_once_on_the_crash_image, CrashPoint,
    HistoryThenWritableMountRecords, WritableMountOutcome,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{mount_writable_with_space_admission, unmount};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{
    allocation_records_under_root, choose_root, choose_system_configuration,
    effective_rollback_floor, instance_table_of_root, readable_roots, recover, replay_journal,
    scan_journal, verified_system_configuration_slots, walk_to_file, JournalPolicy, PoolReader,
    RecoveryOutcome,
};
use singlefs_core::root_record::{RootRecord, UnmountMarker};
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth,
    HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker,
    RollbackTargetChoice, StepPosition,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, newest_persisted_root, writes_and_segments_with_stream_indexes,
    CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::model::{crash_recovery_disagreement, ModelRootKey, ObservedReadBack};
use singlefs_harness::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, RetainedOperation, SharedStream};

fn small(fill_seed: u64) -> ContentChoice {
    ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 200 }, fill_seed }
}
fn big(fill_seed: u64) -> ContentChoice {
    ContentChoice { length: ContentLength::OneByteBelowDataUnitPayloadCapacity, fill_seed }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Custom {
    Plain,
    /// 历史跑完之后（不录）挂载、正常卸载，再录一次可写挂载：枚举的是卸载之后那次挂载。
    UnmountThenMount,
    /// 前缀历史关掉空间准入跑（只供测试的开关，与 harness 用例 admission_formula_on_the_mounted_state 同一种造法），再录一次判准入的可写挂载。
    AdmissionSkippedPrefixThenMount,
}

struct Spec {
    starting_point: HistoryStartingPoint,
    operations: Vec<HistoryOperation>,
    /// None = 只有 mkfs 当基线；Some(k) = 起点与前 k 步当基线。
    prefix: Option<usize>,
    width: HistoryDeviceWidth,
    custom: Custom,
}

fn spec(id: &str) -> Spec {
    use HistoryOperation as O;
    let w = HistoryDeviceWidth::UnitAreaOf384Slots;
    let plain = |starting_point, operations, prefix| Spec { starting_point, operations, prefix, width: w, custom: Custom::Plain };
    for (prefix, skipped) in [("fmm", false), ("fms", true)] {
        if let Some(rest) = id.strip_prefix(prefix) {
            // fmm<宽>-<N> / fms<宽>-<N>：起点之后重挂一次、N 次小覆盖写，再枚举一次可写挂载；fms 的前缀关掉准入。
            let (width, n) = rest.split_once('-').expect("<宽>-<N>");
            let width = match width { "256" => HistoryDeviceWidth::UnitAreaOf256Slots, "240" => HistoryDeviceWidth::UnitAreaOf240Slots, _ => w };
            let n: usize = n.parse().expect("N");
            let mut operations = vec![O::CloseAndMountWritable];
            operations.extend(std::iter::repeat_n(O::PublishOverwrite(small(11)), n));
            if skipped {
                return Spec { starting_point: HistoryStartingPoint::AfterFirstFile, operations, prefix: None, width, custom: Custom::AdmissionSkippedPrefixThenMount };
            }
            operations.push(O::CloseAndMountWritable);
            return Spec { starting_point: HistoryStartingPoint::AfterFirstFile, operations, prefix: Some(n + 1), width, custom: Custom::Plain };
        }
    }
    if let Some(n) = id.strip_prefix("fm") {
        // fm<宽>-<N>：起点之后 N 次满单元覆盖写当基线，再枚举一次可写挂载。宽 256 / 240 / 384。
        let (width, n) = n.split_once('-').expect("fm<宽>-<N>");
        let width = match width {
            "256" => HistoryDeviceWidth::UnitAreaOf256Slots,
            "240" => HistoryDeviceWidth::UnitAreaOf240Slots,
            _ => w,
        };
        let n: u64 = n.parse().expect("N");
        let mut operations: Vec<HistoryOperation> = (0..n).map(|i| O::PublishOverwrite(big(100 + i))).collect();
        operations.push(O::CloseAndMountWritable);
        return Spec { starting_point: HistoryStartingPoint::AfterFirstFile, operations, prefix: Some(usize::try_from(n).expect("n")), width, custom: Custom::Plain };
    }
    match id {
        // 回退那次发布（第一轮 s3 同形）：覆盖写之后回退到环里次新那条。
        "rb" => plain(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 })], Some(1)),
        // 同一个回退目标选法，覆盖写之后先重挂再回退：环里次新那条是暖机的根。
        "rbm" => plain(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::CloseAndMountWritable, O::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 })], Some(2)),
        "rbm3" => plain(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::CloseAndMountWritable, O::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 3 })], Some(2)),
        "s1" => plain(HistoryStartingPoint::AfterMakeFilesystem, vec![O::CloseAndMountWritable], None),
        "s2" => plain(HistoryStartingPoint::AfterFirstFile, vec![O::CloseAndMountWritable], Some(0)),
        "s5" => plain(HistoryStartingPoint::AfterMakeFilesystem, vec![O::CloseAndMountWritable, O::PublishWithoutUnits], None),
        "um" => Spec { starting_point: HistoryStartingPoint::AfterFirstFile, operations: vec![], prefix: Some(0), width: w, custom: Custom::UnmountThenMount },
        "ow" => plain(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7))], Some(0)),
        other => panic!("未知历史 {other}"),
    }
}

fn execution_of(width: HistoryDeviceWidth) -> HistoryExecution {
    HistoryExecution { per_step_checker: PerStepChecker::Skipped, device_width: width, space_admission: SpaceAdmission::JudgedByTheFormula }
}

struct Prepared {
    width: HistoryDeviceWidth,
    parameters: MakeFilesystemParameters,
    /// 基线之前（mkfs 起）整条流的写，全部持久：第二截交记录核对器的历史写表的前半。
    prefix_writes: Vec<RetainedWrite>,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    versions: Vec<PublishedVersion>,
    judged_root_index: usize,
    /// 模型提交过的每一版（崩溃注入同一种攒法：每步之后并一次，再并历史停下时的那一份）。
    committed: BTreeMap<ModelRootKey, Option<Vec<u8>>>,
    /// 基线里盘上最新的根（全部持久）。
    prefix_newest: Option<(CheckpointTxg, InstanceGeneration)>,
    before: Obs,
    after: Obs,
}

fn devices_of(pool: &MemoryPool, parameters: &MakeFilesystemParameters) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    pool.devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(pool.device_size_in_bytes, PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size));
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect()
}

fn pool_of(devices: Vec<(DeviceIdentity, SparseBlockDevice)>, size: u64) -> MemoryPool {
    MemoryPool { devices: devices.into_iter().map(|(i, d)| (i, d.image)).collect(), device_size_in_bytes: size }
}

fn recording_devices_of(pool: &MemoryPool, parameters: &MakeFilesystemParameters, stream: &SharedStream) -> Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> {
    devices_of(pool, parameters)
        .into_iter()
        .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
        .collect()
}

fn pool_of_recording(devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>, size: u64) -> MemoryPool {
    MemoryPool { devices: devices.into_iter().map(|(i, d)| (i, d.into_inner_and_operations().0.image)).collect(), device_size_in_bytes: size }
}

fn newest_readable_root(pool: &MemoryPool) -> Option<(CheckpointTxg, InstanceGeneration)> {
    let sc = choose_system_configuration(pool).ok()?;
    let im = &sc.immutable;
    readable_roots(pool, &im.region_devices, &im.sizes, &im.filesystem_identifier)
        .iter()
        .map(|r| (r.checkpoint_txg, r.instance))
        .max()
}

fn prepare(id: &str) -> Prepared {
    let s = spec(id);
    let h = GeneratedHistory { seed: HistorySeed(0), starting_point: s.starting_point, operations: s.operations.clone() };
    let parameters = s.width.parameters();
    let size = s.width.device_bytes();
    let stream = SharedStream::retaining_contents();
    let mut marks: Vec<usize> = Vec::new();
    let mut committed: BTreeMap<ModelRootKey, Option<Vec<u8>>> = BTreeMap::new();
    let mut history_execution = execution_of(s.width);
    if s.custom == Custom::AdmissionSkippedPrefixThenMount {
        history_execution.space_admission = SpaceAdmission::SkippedByTheTestOnlySwitch;
    }
    let run = execute_history_with(&h, history_execution, &stream, &mut |observation| {
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
        eprintln!("  step {i}: {}", short(&format!("{o:?}"), 400));
    }
    let ops = stream.retained_operations();
    let mut unmount_span: Vec<singlefs_harness::RecordedEntrySpan> = Vec::new();
    let (prefix_ops, window_ops, base): (Vec<RetainedOperation>, Vec<RetainedOperation>, MemoryPool) = match s.custom {
        Custom::Plain => {
            let cut = match s.prefix {
                None => run.operations_written_by_make_filesystem,
                Some(k) => marks[k],
            };
            let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], size);
            base.apply(&ops[..cut]);
            (ops[..cut].to_vec(), ops[cut..].to_vec(), base)
        }
        Custom::UnmountThenMount => {
            let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], size);
            pool.apply(&ops);
            // 挂载、正常卸载（录进前缀流），再录一次可写挂载（枚举它）。
            let prefix_stream = SharedStream::retaining_contents();
            let mut devices = recording_devices_of(&pool, &parameters, &prefix_stream);
            let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula).expect("挂载");
            let (_, mut session) = MountedSession::of_the_mount(mounted);
            let unmount_starts = ops.len() + prefix_stream.operation_count();
            let unmounted = unmount(&parameters, &mut devices, &mut session.allocator, &mut session.current, session.shadow_ledger).expect("卸载");
            match &unmounted { singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(r) => eprintln!("  unmount: F={:?} publishes={}", r.rollback_floor, r.publishes.len()), _ => eprintln!("  unmount: 没写字节") }
            unmount_span.push(singlefs_harness::RecordedEntrySpan { entry: singlefs_harness::RecordedPublishEntry::Unmount, operations: unmount_starts..ops.len() + prefix_stream.operation_count() });
            let pool = pool_of_recording(devices, size);
            let window_stream = SharedStream::retaining_contents();
            let mut devices = recording_devices_of(&pool, &parameters, &window_stream);
            let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula).expect("挂载");
            eprintln!("  window mount: instance={:?} admission={}", mounted.output.instance, short(&format!("{:?}", mounted.output.space_admission), 200));
            drop(mounted);
            let mut all_prefix = ops.clone();
            all_prefix.extend(prefix_stream.retained_operations());
            (all_prefix, window_stream.retained_operations(), pool)
        }
        Custom::AdmissionSkippedPrefixThenMount => {
            let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], size);
            pool.apply(&ops);
            let window_stream = SharedStream::retaining_contents();
            let mut devices = recording_devices_of(&pool, &parameters, &window_stream);
            let mounted = mount_writable_with_space_admission(&parameters, &mut devices, SpaceAdmission::JudgedByTheFormula).expect("挂载");
            eprintln!("  window mount: instance={:?} admission={}", mounted.output.instance, short(&format!("{:?}", mounted.output.space_admission), 300));
            drop(mounted);
            (ops.clone(), window_stream.retained_operations(), pool)
        }
    };
    let geometry = s.width.fixed_geometry();
    let (prefix_writes, _, _) = singlefs_harness::memory_pool::writes_and_segments_with_stream_indexes_and_entries(&prefix_ops, &unmount_span, &geometry);
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&window_ops, &geometry);
    let judged_root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("至少一次根槽写");
    let mut final_pool = base.clone();
    final_pool.apply_writes(&writes);
    let before = observe(&base);
    let after = observe(&final_pool);
    // 自定义历史的模型版本：基线与终态池里每一条可读根，内容取终态读回的（这段只挂载卸载，不改内容）。
    if s.custom != Custom::Plain {
        for pool in [&base, &final_pool] {
            let sc = choose_system_configuration(pool).expect("系统配置");
            let im = &sc.immutable;
            for r in readable_roots(pool, &im.region_devices, &im.sizes, &im.filesystem_identifier) {
                committed.insert(model_root_key(r.instance, r.checkpoint_txg), after.content.clone());
            }
        }
    }
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
    eprintln!("  prefix_ops={} window_ops={} prefix_newest={prefix_newest:?}", prefix_ops.len(), window_ops.len());
    eprintln!("  before: {}\n  after: {}", describe(&before), describe(&after));
    Prepared { width: s.width, parameters, prefix_writes, base, writes, segments, versions, judged_root_index, committed, prefix_newest, before, after }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Obs {
    outcome: String,
    content: Option<Vec<u8>>,
    chosen: Option<RootRecord>,
    effective: Option<RootRecord>,
    prefix_applied: usize,
    max_applied_transaction: u64,
    immutable_without_this_device: Option<String>,
    this_device: Option<u32>,
    quantities: Option<String>,
    /// 每块盘两槽里全部自证过的系统配置的实例代号最大值（取号读法的那个 max）。
    sysconf_instance_max: Option<u32>,
    ring_instance_max: Option<u32>,
    rows: Option<Vec<(u32, u64, u64)>>,
    allocated: Option<BTreeSet<(u32, u64, u64)>>,
    floor: Option<u64>,
    content_one_version_before: Option<Option<Vec<u8>>>,
}

fn observe(reader: &dyn PoolReader) -> Obs {
    let report = recover(reader, JournalPolicy::Consult);
    let (outcome, content) = match &report.outcome {
        RecoveryOutcome::NoFile { .. } => ("no_file".to_string(), None),
        RecoveryOutcome::FileRead { content, .. } => ("file".to_string(), Some(content.clone())),
        RecoveryOutcome::Failed { failure, .. } => (format!("failed:{failure:?}"), None),
    };
    let mut obs = Obs {
        outcome, content, chosen: None, effective: None,
        prefix_applied: report.journal.prefix_applied,
        max_applied_transaction: report.journal.maximum_applied_transaction,
        immutable_without_this_device: None, this_device: None, quantities: None,
        sysconf_instance_max: None, ring_instance_max: None,
        rows: None, allocated: None, floor: None, content_one_version_before: None,
    };
    let Ok(sc) = choose_system_configuration(reader) else { return obs };
    let mut im = sc.immutable;
    obs.this_device = Some(im.this_device.0);
    im.this_device = DeviceIdentity(u32::MAX);
    obs.immutable_without_this_device = Some(format!("{im:?}"));
    obs.quantities = Some(format!("{:?}", sc.quantities));
    let im = &sc.immutable;
    let spacing = u64::from(im.sizes.fixed_structure_slot_spacing);
    obs.sysconf_instance_max = reader
        .device_identities()
        .into_iter()
        .flat_map(|d| verified_system_configuration_slots(reader, d, spacing, &im.filesystem_identifier))
        .map(|c| c.quantities.journal_instance.0)
        .max();
    obs.floor = Some(effective_rollback_floor(reader, &im.region_devices, &im.sizes, &im.filesystem_identifier).0);
    let roots = readable_roots(reader, &im.region_devices, &im.sizes, &im.filesystem_identifier);
    obs.ring_instance_max = roots.iter().map(|r| r.instance.0).max();
    let Some(chosen) = choose_root(reader, &sc) else { return obs };
    obs.chosen = Some(chosen);
    let records = scan_journal(reader, &sc);
    if let Ok((_, effective)) = replay_journal(reader, &chosen, im.sizes.journal_ring_bytes, &records, true) {
        obs.effective = Some(effective);
        obs.rows = instance_table_of_root(reader, &effective).as_ref().map(|t| {
            t.rows.iter().map(|r| (r.instance.0, r.selected_root_txg.0, r.applied_transaction_high_water)).collect()
        });
        obs.allocated = allocation_records_under_root(reader, &effective).ok().map(|a| {
            a.iter().filter(|r| !r.is_released).map(|r| (r.device.0, r.slot.0, r.generation.0)).collect()
        });
        if let Some(before) = roots.iter().find(|r| r.instance == effective.instance && r.checkpoint_txg.0 + 1 == effective.checkpoint_txg.0) {
            let mut fallbacks = 0;
            obs.content_one_version_before = walk_to_file(reader, before, &mut fallbacks).ok();
        }
    }
    obs
}

fn short(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn key(r: &RootRecord) -> (u64, u32) {
    (r.checkpoint_txg.0, r.instance.0)
}

fn describe(o: &Obs) -> String {
    format!(
        "outcome={} len={:?} chosen={:?} eff={:?} marker={:?} F_root={:?} prefix={} W={} rows={:?} floor={:?} sc_inst={:?} ring_inst={:?} q={:?}",
        short(&o.outcome, 60), o.content.as_ref().map(Vec::len), o.chosen.map(|r| key(&r)), o.effective.map(|r| key(&r)),
        o.effective.map(|r| r.unmount_marker), o.effective.map(|r| r.rollback_floor.0), o.prefix_applied, o.max_applied_transaction,
        o.rows, o.floor, o.sysconf_instance_max, o.ring_instance_max, o.quantities.as_deref().map(|q| short(q, 160))
    )
}

#[derive(Default)]
struct Tally {
    states: u64,
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
        for (k, v) in other.classes { *self.classes.entry(k).or_default() += v; }
        for (k, ex) in other.examples {
            let e = self.examples.entry(k).or_default();
            for x in ex { if e.len() < 2 { e.push(x); } }
        }
    }
}

fn rc_committed(p: &Prepared) -> BTreeMap<ModelRootKey, Option<Rc<[u8]>>> {
    p.committed.iter().map(|(k, v)| (*k, v.as_ref().map(|c| Rc::from(c.as_slice())))).collect()
}

fn red(b: bool) -> &'static str {
    if b { "red" } else { "green" }
}

/// u3：一个崩溃状态上各判官的判定。
fn judge_u3(image: &CrashImage<'_>, judgement: &Layer0StateJudgement, p: &Prepared, committed: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>, stage2: bool, tally: &mut Tally) {
    tally.states += 1;
    let report = recover(image, JournalPolicy::Consult);
    let read_back = observed_read_back_after_a_crash(&report);
    let newest = newest_persisted_root(image.writes, &image.persisted).max(p.prefix_newest);
    let newest_key = newest.map(|(t, i)| model_root_key(i, t));
    let o1 = crash_recovery_disagreement(committed, newest_key, &read_back);
    let (chosen, content): (Option<ModelRootKey>, Option<Option<&[u8]>>) = match &read_back {
        ObservedReadBack::FileRead { root, content } => (Some(*root), Some(Some(content.as_slice()))),
        ObservedReadBack::NoFile { root } => (Some(*root), Some(None)),
        ObservedReadBack::Failed { .. } => (None, None),
    };
    // 只取 ③：盘上已持久的最新根槽不许比择到的根新（走读失败按 ① 记红，与 O1 同）。
    let c3 = match (chosen, newest_key) { (None, _) => true, (Some(c), Some(n)) => c < n, (Some(_), None) => false };
    // RefFS-Q 替身：这一窗只有一个操作（或只有不改内容的挂载），允许集合 = {窗前, 窗后}。
    let reffs = match content {
        None => true,
        Some(c) => {
            let ok_before = p.before.outcome != "failed" && c == p.before.content.as_deref();
            let ok_after = p.after.outcome != "failed" && c == p.after.content.as_deref();
            !(ok_before || ok_after)
        }
    };
    let layer0_items = judgement.red_items();
    let rebuilt = report.journal.prefix_applied > 0;
    let mut stage2_text = String::new();
    if stage2 && (rebuilt || std::env::var("R5_STAGE2_ALL").is_ok()) {
        let crash = materialized_crash_image(image);
        let crash_point = CrashPoint { segment_index: 0, persisted_within_the_segment: Vec::new(), step: StepPosition::StartingPoint, operation_kind: None };
        let mount_run = mount_and_publish_once_on_the_crash_image(&crash, &p.parameters, execution_of(p.width), HistorySeed(0), &crash_point);
        let geometry = p.width.fixed_geometry();
        let (mount_writes, _, _) = writes_and_segments_with_stream_indexes(&mount_run.mount_operations, &geometry);
        let (publish_writes, _, _) = writes_and_segments_with_stream_indexes(&mount_run.publish_operations, &geometry);
        let mut history_writes = p.prefix_writes.clone();
        history_writes.extend(image.writes.iter().cloned());
        let mut persisted = vec![true; p.prefix_writes.len()];
        persisted.extend(image.persisted.iter().copied());
        let records = HistoryThenWritableMountRecords::new(&history_writes, &persisted, report.effective_root, &mount_writes, &publish_writes);
        let crash_image = CrashImage { base: &crash, writes: &[], persisted: Vec::new() };
        let rc = records.record_check_after_the_writable_mount(&crash_image, &mount_run.pool_after);
        let violated: Vec<&str> = check_pool_image(&mount_run.pool_after)
            .into_iter()
            .filter_map(|(inv, v)| matches!(v, InvariantVerdict::Violated(_)).then_some(inv))
            .collect();
        let outcome = match &mount_run.outcome {
            WritableMountOutcome::MountRefused { member, .. } => &format!("mount_refused:{}", short(member, 70)),
            WritableMountOutcome::PublishRefused { member, .. } => &format!("publish_refused:{}", short(member, 70)),
            WritableMountOutcome::Published => "published",
        };
        stage2_text = format!(" stage2={outcome} s2_root_without_record={} s2_claimed_missing={} s2_checker={violated:?}", rc.root_without_record, rc.claimed_state_missing_unit);
    }
    let class = format!(
        "rebuilt={rebuilt} o1={} c3={} reffs={} conj(reffs&c3)={} layer0={layer0_items:?}{stage2_text}",
        red(o1.is_some()), red(c3), red(reffs), red(reffs || c3)
    );
    tally.add(class, || format!("read_back={} o1={:?}", short(&format!("{read_back:?}"), 120), o1.map(|d| short(&format!("{d:?}"), 300))));
}

const U5_ACTIONS: [&str; 2] = ["mount", "mount+write"];

fn post_content() -> Vec<u8> {
    (0..321u32).map(|i| u8::try_from(i % 251).expect("小") ^ 0x5a).collect()
}

fn act(crash: &MemoryPool, parameters: &MakeFilesystemParameters, action: &str) -> (String, Option<MemoryPool>, String) {
    let mut devices = devices_of(crash, parameters);
    let mounted = match mount_writable_with_space_admission(parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => {
            // 拒的合法性（D23 已定项 14「仍为真就在取号之前拒可写挂载、盘上逐字节不变」）：拒之后盘上逐字节同不同。
            let unchanged = pool_of(devices, crash.device_size_in_bytes) == *crash;
            return (format!("mount_refused[disk_unchanged={unchanged}]:{}", short(&format!("{e:?}"), 60)), None, String::new());
        }
    };
    let floor_raises = mounted.output.space_admission.floor_raises().len();
    let admission = format!("floor_raises={floor_raises}");
    let size = crash.device_size_in_bytes;
    match action {
        "mount" => ("ok".into(), Some(pool_of(devices, size)), admission),
        "mount+write" => {
            let content = post_content();
            let file = FirstFile { content: &content, write_time_seconds: 1_000 };
            let (output, mut session) = MountedSession::of_the_mount(mounted);
            let result = match &session.current {
                PoolVersion::WithFile(_) => session.publish_user_change(&mut devices, UserChange::Overwrite(file)).map(|_| ()).map_err(|e| short(&format!("{e:?}"), 90)),
                PoolVersion::WithoutFile(current) => {
                    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
                    publish_first_file(&mut writer, &mut session.allocator, &current.root, file, output.instance, &current.record_bytes).map(|_| ()).map_err(|e| short(&format!("{e:?}"), 90))
                }
            };
            match result {
                Ok(()) => ("ok".into(), Some(pool_of(devices, size)), admission),
                Err(e) => (format!("write_refused:{e}"), Some(pool_of(devices, size)), admission),
            }
        }
        other => panic!("未知动作 {other}"),
    }
}

fn v(same: bool) -> &'static str {
    if same { "same" } else { "diff" }
}

/// D18 已定项 11「恢复写的行」：在 R1 那一版的表上，给 [max(R1 所选根实例, 1), 新实例) 每个实例写一行，后写覆盖。
fn expected_rows(r1: &Obs, new_instance: u32) -> Option<Vec<(u32, u64, u64)>> {
    let chosen = r1.chosen?;
    let effective = r1.effective?;
    let mut rows: BTreeMap<u32, (u64, u64)> = r1.rows.clone()?.into_iter().map(|(i, t, w)| (i, (t, w))).collect();
    let i1 = chosen.instance.0;
    for i in i1.max(1)..new_instance {
        let row = if i == i1 { (effective.checkpoint_txg.0, r1.max_applied_transaction) } else { (0, 0) };
        rows.insert(i, row);
    }
    Some(rows.into_iter().map(|(i, (t, w))| (i, t, w)).collect())
}

fn compare(r1: &Obs, r2: &Obs, action: &str) -> Vec<(String, &'static str)> {
    let mut out: Vec<(String, &'static str)> = Vec::new();
    let mut push = |f: &str, s: &'static str| out.push((f.to_string(), s));
    // 比的对象：mount 下是 R2 现行（暖机最后一版），mount+write 下是挂载后那次发布之前那一版。
    let compared_content = if action == "mount+write" { r2.content_one_version_before.clone() } else { Some(r2.content.clone()) };
    push("posix.content_of_the_compared_version", v(compared_content == Some(r1.content.clone()) && !r1.outcome.starts_with("failed")));
    push("sysconf.immutable_without_this_device", v(r1.immutable_without_this_device == r2.immutable_without_this_device));
    push("sysconf.this_device", v(r1.this_device == r2.this_device));
    push("effective_floor", v(r1.floor == r2.floor));
    if let (Some(a), Some(b)) = (&r1.effective, &r2.effective) {
        let marker_class = match a.unmount_marker {
            UnmountMarker::WrittenByTheUnmountSequence => "r1_marker=unmount",
            UnmountMarker::NotWrittenByTheUnmountSequence => "r1_marker=not_unmount",
        };
        out.push((format!("eff.unmount_marker[{marker_class}]"), v(a.unmount_marker == b.unmount_marker)));
        out.push(("eff.unmount_marker_r2_is_not_unmount".to_string(), v(b.unmount_marker == UnmountMarker::NotWrittenByTheUnmountSequence)));
        out.push(("eff.rollback_floor".to_string(), v(a.rollback_floor == b.rollback_floor)));
        // 候选关系：R2 那一版根记录里的 F = R1 的 F 生效值（系统配置与根两处取大，D16 已定项 1「生效」）。
        out.push(("eff.rollback_floor_r2_root_equals_r1_effective_floor".to_string(), v(r1.floor == Some(b.rollback_floor.0))));
        out.push(("eff.tree_identifier_watermark".to_string(), v(a.tree_identifier_watermark == b.tree_identifier_watermark)));
        // 新实例代号 = max(系统配置自证槽的实例代号, 根环里全部根的实例代号) + 1（D18 已定项 11 取号读法，D23 已定项 16）。
        let expected_instance = r1.sysconf_instance_max.max(r1.ring_instance_max).map(|m| m + 1);
        out.push(("instance.r2_is_max_sysconf_ring_plus_one".to_string(), v(expected_instance == Some(b.instance.0))));
        let rows_ok = expected_rows(r1, b.instance.0).is_some_and(|e| Some(e) == r2.rows);
        out.push(("rows.r2_equals_d18_item11_rule".to_string(), v(rows_ok)));
    } else {
        out.push(("eff.*".to_string(), "na"));
    }
    if let (Some(a1), Some(a2)) = (&r1.allocated, &r2.allocated) {
        let missing = a1.difference(a2).count();
        out.push((format!("alloc.r1_allocated_not_in_r2={missing}"), "count"));
    }
    out
}

fn judge_u5(ordinal: u64, image: &CrashImage<'_>, p: &Prepared, tally: &mut Tally) {
    tally.states += 1;
    let crash = materialized_crash_image(image);
    let torn = image.persisted.iter().enumerate().any(|(i, q)| *q && i >= p.writes.len());
    let r1 = observe(&crash);
    let class = format!("torn={torn} r1_outcome={} r1_rebuilt={}", r1.outcome.split(':').next().unwrap_or(""), r1.prefix_applied > 0);
    tally.add(format!("CLASS {class}"), || String::new());
    for action in U5_ACTIONS {
        let (outcome, after, admission) = act(&crash, &p.parameters, action);
        tally.add(format!("ACTION {action} | {class} -> {} {admission}", short(&outcome, 90)), || String::new());
        let Some(after) = after else { continue };
        if outcome != "ok" { continue; }
        let r2 = observe(&after);
        for (field, verdict) in compare(&r1, &r2, action) {
            tally.add(format!("FIELD action={action} {admission} field={field} verdict={verdict}"), || {
                format!("ordinal={ordinal} {class}\n      R1: {}\n      R2: {}", describe(&r1), describe(&r2))
            });
        }
    }
}

fn scan(ids: &[String]) {
    for id in ids {
        let p = prepare(id);
        println!("history={id} writes={} segments={} closed_form={}", p.writes.len(), p.segments.len(), closed_form_state_count(&p.segments));
        println!("  before: {}", describe(&p.before));
        println!("  after:  {}", describe(&p.after));
        println!("  committed versions: {}", p.committed.len());
        for (k, c) in &p.committed {
            println!("    ({}, {}) len={:?} head={:?}", k.checkpoint_txg.0, k.instance.0, c.as_ref().map(Vec::len), c.as_ref().map(|c| c.iter().take(4).copied().collect::<Vec<u8>>()));
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("sizes");
    if mode == "scan" {
        scan(&args[2..]);
        return;
    }
    let id = args.get(2).map(String::as_str).unwrap_or("rb");
    let threads: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    let limit: Option<u64> = args.get(4).and_then(|s| s.parse().ok());
    let started = std::time::Instant::now();
    let p = prepare(id);
    let total = layer0_state_count_with_torn_in_place_overwrites(&p.base, &p.writes, &p.segments, &full_expansion);
    println!(
        "history={id} mode={mode} mutation={:?} writes={} segments={} closed_form_two_state={} with_torn={} prepare_secs={:.2}",
        std::env::var("R5_MUTATION").ok(), p.writes.len(), p.segments.len(), closed_form_state_count(&p.segments), total, started.elapsed().as_secs_f64()
    );
    if mode == "sizes" {
        for (i, s) in p.segments.iter().enumerate() {
            let kinds: Vec<&str> = s.iter().map(|w| p.writes[*w].kind.name()).collect();
            println!("  segment {i}: {} writes {:?}", s.len(), kinds);
        }
        return;
    }
    let stage2 = mode == "u3s2";
    let end = limit.map_or(total, |l| l.min(total));
    let start: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
    let merged = Mutex::new(Tally::default());
    let chunk = (end - start).div_ceil(threads as u64).max(1);
    std::thread::scope(|scope| {
        for t in 0..threads as u64 {
            let from = start + t * chunk;
            let to = (start + (t + 1) * chunk).min(end);
            if from >= to { continue; }
            let p = &p;
            let merged = &merged;
            scope.spawn(move || {
                let mut tally = Tally::default();
                let committed = rc_committed(p);
                judge_layer0_state_range(&p.base, &p.writes, &p.segments, p.judged_root_index, &p.versions, &full_expansion, from..to, &mut |ordinal, image, judgement| {
                    match mode {
                        "u3" | "u3s2" => judge_u3(image, judgement, p, &committed, stage2, &mut tally),
                        "u5" => judge_u5(ordinal, image, p, &mut tally),
                        other => panic!("未知模式 {other}"),
                    }
                });
                merged.lock().expect("锁").merge(tally);
            });
        }
    });
    let tally = merged.into_inner().expect("锁");
    println!("states_judged={} of {} range={start}..{end} elapsed_secs={:.1}", tally.states, total, started.elapsed().as_secs_f64());
    for (k, c) in &tally.classes {
        println!("{k} count={c}");
        for x in &tally.examples[k] {
            if !x.is_empty() { println!("    e.g. {x}"); }
        }
    }
}

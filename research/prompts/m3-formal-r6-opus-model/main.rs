//! m3-formal-r6 攻方腿原型（只在冻结副本的拷贝上编、跑；模型目录里留一份源码）。
//! 枚举：`singlefs_checker_tier::crash::judge_layer0_state_range` + `full_expansion`，0..全量状态数（层 0 同一枚举域：
//! 每段都展开、原地覆写取第三态）。历史是本原型手写的几段（不取自任何层 0 用例）。
//! 模式：
//!   sizes <历史>                     每段写的种类与状态数
//!   lo <历史> <线程> [上限] [起点]    每个崩溃状态：恢复读回、O1 判官全四条、RefFS-Q 在四种读法 × 两种崩溃时刻归属下的判定
//!   tgt <历史> <线程> [上限] [起点]   每个崩溃状态：恢复 → 可写挂载 → 一次改内容的发布 → 对根环里每一条可读根试回退（用户动作放开扫）
//!   w1 <过滤串> <线程> [上限] [起点]  旧池上重做 mkfs，mkfs 本身录成写流，逐态：实现择哪份系统配置、恢复结局、checker
//! 副本变异：R6_MUTATION=rollback_not_durable（r6_patch_copy.py 装的）。
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::Mutex;

use singlefs_checker::image::{system_configuration_slot_readings, InvariantVerdict, SystemConfigurationSlotReading};
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{full_expansion, judge_layer0_state_range, layer0_state_count_with_torn_in_place_overwrites};
use singlefs_checker_tier::crash_injection::materialized_crash_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters};
use singlefs_core::mount::{mount_writable_with_space_admission, roll_back_by_a_forward_publish, RollbackError, RollbackTarget};
use singlefs_core::mounted_session::{MountedSession, UserChange};
use singlefs_core::recovery::{choose_system_configuration, readable_roots, recover, JournalPolicy, PoolReader, RecoveryOutcome};
use singlefs_core::root_ring::RootRingSlotsPerRegion;
use singlefs_core::system_configuration::SystemImmutableSizes;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion, PoolWriter,
};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, GeneratedHistory, HistoryDeviceWidth, HistoryExecution,
    HistoryOperation, HistorySeed, HistoryStartingPoint, PerStepChecker, RollbackTargetChoice, StepPosition,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, newest_persisted_root, writes_and_segments, writes_and_segments_with_stream_indexes,
    CrashImage, MemoryPool, PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::model::{crash_recovery_disagreement, ModelRootKey};
use singlefs_harness::model_comparison::{model_root_key, observed_read_back_after_a_crash};
use singlefs_harness::segments::{FixedGeometry, StepKind};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

fn small(fill_seed: u64) -> ContentChoice {
    ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 200 }, fill_seed }
}

/// 一步的种类（RefFS 的读法只看这几样）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    /// mkfs 起点（没有文件、不是发布调用）。
    Mkfs,
    /// 发布调用（起点那次第一个文件、第一个文件、覆盖写、零单元发布）。
    Publish,
    Rollback,
    /// 可写挂载、冷启动等不改 POSIX 内容的步。
    Other,
}

fn kind_of(op: &HistoryOperation) -> Kind {
    match op {
        HistoryOperation::PublishFirstFile(_) | HistoryOperation::PublishOverwrite(_) | HistoryOperation::PublishWithoutUnits => Kind::Publish,
        HistoryOperation::RollBackWhileMounted(_) => Kind::Rollback,
        _ => Kind::Other,
    }
}

struct Spec {
    starting_point: HistoryStartingPoint,
    operations: Vec<HistoryOperation>,
    /// 基线切在第几个位置之后（位置 0 = 起点，位置 p = 第 p 步操作；与 r5probe 的 prefix 同义）。
    prefix: usize,
    /// 窗里前几段整段并进基线（缩历史长度：基线切在那一步自己的某道屏障之后；留下的那一截的崩溃状态照全取）。
    skip_segments: usize,
}

fn spec(id: &str) -> Spec {
    use HistoryOperation as O;
    let rb1 = O::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 });
    match id {
        // 覆盖写（基线）→ 回退到环里次新那条（第一个文件那一版）→ 覆盖写；枚举回退与之后那次覆盖写两窗。
        "rbw" => Spec { starting_point: HistoryStartingPoint::AfterFirstFile, operations: vec![O::PublishOverwrite(small(7)), rb1, O::PublishOverwrite(small(9))], prefix: 1, skip_segments: 0 },
        // 同上，只枚举回退之后那次覆盖写那一窗（回退一窗在 rbw 里）。
        "rbw3" => Spec { starting_point: HistoryStartingPoint::AfterFirstFile, operations: vec![O::PublishOverwrite(small(7)), rb1, O::PublishOverwrite(small(9))], prefix: 2, skip_segments: 0 },
        // 覆盖写（基线）→ 回退到环里次新那条 → 重挂（关掉会话、可写挂载）；枚举回退与重挂两窗。
        "rbm" => Spec { starting_point: HistoryStartingPoint::AfterFirstFile, operations: vec![O::PublishOverwrite(small(7)), rb1, O::CloseAndMountWritable], prefix: 1, skip_segments: 0 },
        // 两次覆盖写，枚举两窗（first_overwrite_not_durable 的对照历史）。
        "ow2" => Spec { starting_point: HistoryStartingPoint::AfterFirstFile, operations: vec![O::PublishOverwrite(small(7)), O::PublishOverwrite(small(9))], prefix: 0, skip_segments: 0 },
        // mkfs 之后挂载、发第一个文件；枚举第一个文件那一窗（回退目标可不可命名用它）。
        "ff" => Spec { starting_point: HistoryStartingPoint::AfterMakeFilesystem, operations: vec![O::CloseAndMountWritable, O::PublishFirstFile(small(5))], prefix: 1, skip_segments: 0 },
        // 同 ff，基线切在第一个文件那次发布的单元屏障之后：只枚举记录、根槽、系统配置轮换三段。
        "fft" => Spec { starting_point: HistoryStartingPoint::AfterMakeFilesystem, operations: vec![O::CloseAndMountWritable, O::PublishFirstFile(small(5))], prefix: 1, skip_segments: 1 },
        other => panic!("未知历史 {other}"),
    }
}

fn execution() -> HistoryExecution {
    HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    }
}

struct Prepared {
    parameters: MakeFilesystemParameters,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    versions: Vec<PublishedVersion>,
    judged_root_index: usize,
    committed: BTreeMap<ModelRootKey, Option<Vec<u8>>>,
    prefix_newest: Option<(CheckpointTxg, InstanceGeneration)>,
    /// 每个位置（0 = 起点，p = 第 p 步）的种类与做完之后整池恢复读回的内容。
    kinds: Vec<Kind>,
    contents: Vec<Option<Vec<u8>>>,
    /// 窗里每个写落在哪个位置（按录制流下标与每一步跑完时流里的步数定）。
    write_position: Vec<usize>,
    /// 基线之后第一个位置。
    first_window_position: usize,
    /// 每一段的终结者（切开它的那道屏障或那次 FUA 写）是哪个位置发的；最后一段没有终结者，记最后一个位置。
    segment_terminator_position: Vec<usize>,
}

fn content_of(pool: &dyn PoolReader) -> Option<Vec<u8>> {
    match recover(pool, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { content, .. } => Some(content),
        _ => None,
    }
}

fn prepare(id: &str) -> Prepared {
    let s = spec(id);
    let width = HistoryDeviceWidth::UnitAreaOf384Slots;
    let h = GeneratedHistory { seed: HistorySeed(0), starting_point: s.starting_point, operations: s.operations.clone() };
    let parameters = width.parameters();
    let size = width.device_bytes();
    let stream = SharedStream::retaining_contents();
    let mut marks: Vec<usize> = Vec::new();
    let mut positions: Vec<String> = Vec::new();
    let mut committed: BTreeMap<ModelRootKey, Option<Vec<u8>>> = BTreeMap::new();
    let run = execute_history_with(&h, execution(), &stream, &mut |observation| {
        for (key, file) in observation.model.committed_versions() {
            committed.insert(key, file.map(|c| c.to_vec()));
        }
        marks.push(stream.operation_count());
        positions.push(format!("{:?}", observation.position));
    });
    for (key, file) in &run.committed_versions_at_the_end {
        committed.insert(*key, file.clone());
    }
    eprintln!("history {id}: ending {:?} positions {positions:?} marks {marks:?} mkfs_ops {}", run.ending, run.operations_written_by_make_filesystem);
    for (i, o) in run.outcomes.iter().enumerate() {
        eprintln!("  step {i}: {}", short(&format!("{o:?}"), 300));
    }
    assert_eq!(marks.len(), s.operations.len() + 1, "每个位置一条登记（起点 + 每步）");
    let ops = stream.retained_operations();
    let mut cut = marks[s.prefix];
    if s.skip_segments > 0 {
        let geometry = width.fixed_geometry();
        let (_, segments, stream_indexes) = writes_and_segments_with_stream_indexes(&ops[cut..], &geometry);
        cut += stream_indexes[segments[s.skip_segments][0]];
    }
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], size);
    base.apply(&ops[..cut]);
    let geometry = width.fixed_geometry();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(&ops[cut..], &geometry);
    let write_position: Vec<usize> = stream_indexes
        .iter()
        .map(|local| {
            let global = cut + local;
            marks.iter().position(|m| *m > global).expect("窗里的写都在最后一步跑完之前")
        })
        .collect();
    let judged_root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("至少一次根槽写");
    let window = &ops[cut..];
    let position_of_global = |global: usize| marks.iter().position(|m| *m > global).unwrap_or(marks.len() - 1);
    let segment_terminator_position: Vec<usize> = segments
        .iter()
        .enumerate()
        .map(|(k, segment)| {
            let last = *segment.last().expect("段不空");
            let last_local = stream_indexes[last];
            if window[last_local].operation.kind == singlefs_harness::RecordedOperationKind::WriteForceUnitAccess {
                return write_position[last];
            }
            let until = segments.get(k + 1).map_or(window.len(), |next| stream_indexes[next[0]]);
            match (last_local + 1..until).find(|i| window[*i].operation.kind == singlefs_harness::RecordedOperationKind::Barrier) {
                Some(i) => position_of_global(cut + i),
                None => marks.len() - 1,
            }
        })
        .collect();
    eprintln!("  segment_terminator_position {segment_terminator_position:?}");
    let mut kinds = vec![match s.starting_point {
        HistoryStartingPoint::AfterFirstFile => Kind::Publish,
        HistoryStartingPoint::AfterMakeFilesystem => Kind::Mkfs,
    }];
    kinds.extend(s.operations.iter().map(kind_of));
    let contents: Vec<Option<Vec<u8>>> = marks
        .iter()
        .map(|m| {
            let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], size);
            pool.apply(&ops[..*m]);
            content_of(&pool)
        })
        .collect();
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
    let prefix_newest = {
        let sc = choose_system_configuration(&base).expect("基线的系统配置");
        let im = &sc.immutable;
        readable_roots(&base, &im.region_devices, &im.sizes, &im.filesystem_identifier).iter().map(|r| (r.checkpoint_txg, r.instance)).max()
    };
    eprintln!("  kinds {kinds:?} contents {:?} window_writes {} write_position {:?}", contents.iter().map(|c| c.as_ref().map(Vec::len)).collect::<Vec<_>>(), writes.len(), write_position);
    Prepared { parameters, base, writes, segments, versions, judged_root_index, committed, prefix_newest, kinds, contents, write_position, first_window_position: s.prefix + 1, segment_terminator_position }
}

fn short(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

#[derive(Default)]
struct Tally {
    states: u64,
    classes: BTreeMap<String, u64>,
}

impl Tally {
    fn add(&mut self, class: String) {
        *self.classes.entry(class).or_default() += 1;
    }
    fn merge(&mut self, other: Tally) {
        self.states += other.states;
        for (k, v) in other.classes {
            *self.classes.entry(k).or_default() += v;
        }
    }
}

/// 内容的名字：位置 p 做完之后的内容第一次出现在哪个位置就叫 `v@p`；没有文件叫 `nofile`；别的叫 `other(len)`。
fn label(p: &Prepared, content: &Option<Vec<u8>>) -> String {
    if content.is_none() {
        return "nofile".to_string();
    }
    match p.contents.iter().position(|c| c == content) {
        Some(i) => format!("v@{i}"),
        None => format!("other({})", content.as_ref().map_or(0, Vec::len)),
    }
}

/// RefFS-Q 的四种读法（报告「回退之后又崩溃」一节）：
/// R0 = 条款字面、lo 只由 fsync 抬（这几段历史没有 fsync ⇒ lo 停在历史开头），边界只取发布调用；
/// R1 = 发布调用返回也抬 lo，回退不是边界；R1b = 回退是边界（j 可落在它末尾）但不抬 lo；R2 = 回退与发布调用一样，是边界、做成就抬 lo。
#[derive(Clone, Copy, Debug)]
enum Reading {
    R0,
    R1,
    R1b,
    R2,
}

const READINGS: [Reading; 4] = [Reading::R0, Reading::R1, Reading::R1b, Reading::R2];

fn raises_lo(r: Reading, k: Kind) -> bool {
    match r {
        Reading::R0 => false,
        Reading::R1 | Reading::R1b => k == Kind::Publish,
        Reading::R2 => matches!(k, Kind::Publish | Kind::Rollback),
    }
}

fn is_boundary(r: Reading, k: Kind) -> bool {
    match r {
        Reading::R0 | Reading::R1 => k == Kind::Publish,
        Reading::R1b | Reading::R2 => matches!(k, Kind::Publish | Kind::Rollback),
    }
}

/// 位置 h 那一步在飞（没交回）时 RefFS-Q 的允许集合：{S_lo} ∪ {S_p : lo < p ≤ h，p 是边界}。lo 之前没有抬 lo 的位置时 S_lo = 历史开头（没有文件）。
fn allowed(p: &Prepared, r: Reading, h: usize) -> BTreeSet<Option<Vec<u8>>> {
    let lo = (0..h).rev().find(|q| raises_lo(r, p.kinds[*q]));
    let mut set = BTreeSet::new();
    set.insert(lo.and_then(|q| p.contents[q].clone()));
    let from = lo.map_or(0, |q| q + 1);
    for q in from..=h.min(p.kinds.len() - 1) {
        if is_boundary(r, p.kinds[q]) {
            set.insert(p.contents[q].clone());
        }
    }
    set
}

/// 三种崩溃时刻归属：fw = 崩溃注入今天的写法（段里第一个被扣下的写落在哪一步，`StreamMarks::step_containing`）；
/// lp = 崩溃时刻不早于任何一个已持久写的发出时刻（已持久的写里最晚那一个落在哪一步）。撕裂镜像的那几份按同盘同偏移同长度对回当前段里的写；
/// se = 崩溃时刻可以晚到切开当前段的那道屏障（或那次 FUA 写）完成之前，取发它的那一步（最严、对守约的实现不多判：守约的实现一步交回之前自己发完切段的屏障）。
fn attributions(p: &Prepared, image: &CrashImage<'_>) -> (usize, usize, usize) {
    let n = p.writes.len();
    // 撕裂态里原写可能记成持久、撕裂镜像盖在它后面：那时没有扣下的原写，当前段取最后一段（撕裂只落在当前段）。
    let first_withheld = (0..n).find(|i| !image.persisted[*i]);
    let fw = first_withheld.map_or_else(|| *p.write_position.last().expect("窗里有写"), |i| p.write_position[i]);
    let current_segment = match first_withheld {
        Some(i) => p.segments.iter().find(|s| s.contains(&i)).expect("每个写在一段里"),
        None => p.segments.last().expect("至少一段"),
    };
    let mut lp = (0..n).filter(|i| image.persisted[*i]).map(|i| p.write_position[i]).max().unwrap_or(p.first_window_position);
    for (k, persisted) in image.persisted.iter().enumerate().skip(n) {
        if !*persisted {
            continue;
        }
        let w = &image.writes[k];
        for i in current_segment {
            let o = &p.writes[*i];
            if o.device == w.device && o.offset == w.offset && o.length_in_bytes() == w.length_in_bytes() {
                lp = lp.max(p.write_position[*i]);
            }
        }
    }
    let segment_index = match first_withheld {
        Some(i) => p.segments.iter().position(|s| s.contains(&i)).expect("每个写在一段里"),
        None => p.segments.len() - 1,
    };
    (fw, lp.max(p.first_window_position), p.segment_terminator_position[segment_index])
}

fn judge_lo(image: &CrashImage<'_>, judgement: &singlefs_checker_tier::crash::Layer0StateJudgement, p: &Prepared, committed: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>, tally: &mut Tally) {
    tally.states += 1;
    let report = &judgement.consulted;
    let read_back = observed_read_back_after_a_crash(report);
    let content = match &report.outcome {
        RecoveryOutcome::FileRead { content, .. } => Some(content.clone()),
        _ => None,
    };
    let failed = matches!(report.outcome, RecoveryOutcome::Failed { .. });
    let newest = newest_persisted_root(image.writes, &image.persisted).max(p.prefix_newest);
    let o1 = crash_recovery_disagreement(committed, newest.map(|(t, i)| model_root_key(i, t)), &read_back);
    let (fw, lp, se) = attributions(p, image);
    let mut verdicts = String::new();
    for (name, h) in [("fw", fw), ("lp", lp), ("se", se)] {
        for r in READINGS {
            let red = failed || !allowed(p, r, h).contains(&content);
            verdicts.push_str(&format!(" {r:?}.{name}={}", if red { "red" } else { "green" }));
        }
    }
    let rebuilt = report.journal.prefix_applied > 0;
    tally.add(format!(
        "rec={} rebuilt={rebuilt} h_fw={fw} h_lp={lp} h_se={se} o1={}{verdicts}",
        if failed { "failed".to_string() } else { label(p, &content) },
        if o1.is_some() { "red" } else { "green" }
    ));
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

fn pool_of(devices: &[(DeviceIdentity, SparseBlockDevice)], size: u64) -> MemoryPool {
    MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(), device_size_in_bytes: size }
}

fn variant(debug: String) -> String {
    debug.split(|c: char| c == '{' || c == '(' || c == ' ').next().unwrap_or("").to_string()
}

/// tgt：崩溃 → 恢复 → 可写挂载 → 一次改内容的发布（已确认）→ 根环里每一条可读根各试一次回退（用户选哪条放开扫）。
fn judge_tgt(image: &CrashImage<'_>, p: &Prepared, tally: &mut Tally) {
    tally.states += 1;
    let crash = materialized_crash_image(image);
    let size = crash.device_size_in_bytes;
    let rec = label(p, &content_of(&crash));
    let mut devices = devices_of(&crash, &p.parameters);
    let mounted = match mount_writable_with_space_admission(&p.parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => {
            tally.add(format!("rec={rec} mount_refused:{}", variant(format!("{e:?}"))));
            return;
        }
    };
    let (output, mut session) = MountedSession::of_the_mount(mounted);
    let c3: Vec<u8> = (0..333u32).map(|i| u8::try_from(i % 251).expect("小") ^ 0x3c).collect();
    let file = FirstFile { content: &c3, write_time_seconds: 2_000 };
    let current = match &session.current {
        PoolVersion::WithFile(_) => match session.publish_user_change(&mut devices, UserChange::Overwrite(file)) {
            Ok(_) => match &session.current {
                PoolVersion::WithFile(t) => t.clone(),
                PoolVersion::WithoutFile(_) => panic!("覆盖写之后带文件"),
            },
            Err(e) => {
                tally.add(format!("rec={rec} c3_refused:{}", variant(format!("{e:?}"))));
                return;
            }
        },
        PoolVersion::WithoutFile(without) => {
            let mut writer = PoolWriter::new(&p.parameters, devices.as_mut_slice());
            match publish_first_file(&mut writer, &mut session.allocator, &without.root, file, output.instance, &without.record_bytes) {
                Ok(t) => t,
                Err(e) => {
                    tally.add(format!("rec={rec} c3_refused:{}", variant(format!("{e:?}"))));
                    return;
                }
            }
        }
    };
    let c3_key = (current.root.checkpoint_txg, current.root.instance);
    let after_c3 = pool_of(&devices, size);
    let sc = choose_system_configuration(&after_c3).expect("系统配置");
    let im = &sc.immutable;
    let roots: BTreeSet<(CheckpointTxg, InstanceGeneration)> =
        readable_roots(&after_c3, &im.region_devices, &im.sizes, &im.filesystem_identifier).iter().map(|r| (r.checkpoint_txg, r.instance)).collect();
    for (txg, instance) in roots.iter().rev() {
        let key = (*txg, *instance);
        let origin = if key == c3_key {
            "c3_confirmed"
        } else if *instance == output.instance {
            "continuation_mount"
        } else if p.committed.contains_key(&model_root_key(*instance, *txg)) {
            "original_run"
        } else {
            "other"
        };
        let mut devs = devices_of(&after_c3, &p.parameters);
        let mut allocator = session.allocator.clone();
        let mut cur = current.clone();
        let result = match roll_back_by_a_forward_publish(&p.parameters, &mut devs, &mut allocator, &mut cur, RollbackTarget { instance: *instance, checkpoint_txg: *txg }) {
            Ok(_) => {
                let after = content_of(&pool_of(&devs, size));
                let l = if after.as_deref() == Some(c3.as_slice()) { "c3".to_string() } else { label(p, &after) };
                // 已确认发布：起点（若是发布）与基线里的发布调用、c3。在飞那一步（窗里那次发布）没有交回。
                let confirmed: Vec<Option<Vec<u8>>> = (0..p.first_window_position)
                    .filter(|q| p.kinds[*q] == Kind::Publish)
                    .map(|q| p.contents[q].clone())
                    .chain(std::iter::once(Some(c3.clone())))
                    .collect();
                let nameable = if key == c3_key {
                    "by_root"
                } else if confirmed.contains(&after) {
                    "by_content_only"
                } else {
                    "unnameable"
                };
                format!("ok:{l}:{nameable}")
            }
            Err(RollbackError::TargetNotACandidate { exclusion, .. }) => format!("err:TargetNotACandidate:{exclusion:?}"),
            Err(e) => format!("err:{}", variant(format!("{e:?}"))),
        };
        tally.add(format!("rec={rec} target_origin={origin} -> {result}"));
    }
}

// ---------------------------------------------------------------- w1：旧池上重做 mkfs，mkfs 本身录成写流
/// 单元区 384 槽那一档小盘（与 m3-c651-r1 攻方原型同一档）：环 6 MiB，盘 = (1408 + 384) 槽。
const W1_JOURNAL_RING_BYTES: u64 = 6 << 20;
const W1_DEVICE_BYTES: u64 = (1408 + 384) * 16384;
const NEW_FSID: [u8; 16] = *b"c651-new-pool-B!";
const OLD_FSID: [u8; 16] = *b"c651-old-pool-A!";
const WRITE_TIME: u64 = 1_788_000_000;

type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn w1_parameters(fsid: [u8; 16], io_min: u32) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: fsid,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: io_min,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(io_min),
            journal_ring_bytes: W1_JOURNAL_RING_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

fn fresh_device() -> SparseBlockDevice {
    SparseBlockDevice::new(W1_DEVICE_BYTES, PhysicalBlockSizeInBytes(512))
}

fn clone_device(device: &SparseBlockDevice) -> SparseBlockDevice {
    let mut copy = fresh_device();
    copy.image = device.image.clone();
    copy
}

fn w1_content(seed: u64) -> Vec<u8> {
    (0..3000u64).map(|i| u8::try_from((i * 137 + seed * 11 + 3) % 251).expect("<256")).collect()
}

/// 旧池的用户动作：0 = 只 mkfs；1 = 取号 + 暖机；2 + k = 再发第一个文件与 k 次覆盖写。交回最后一版的内容。
fn make_and_use(devices: &mut Devices, parameters: &MakeFilesystemParameters, usage: u32, seed: u64) -> Option<Vec<u8>> {
    let genesis = make_filesystem(parameters, devices).expect("mkfs");
    if usage == 0 {
        return None;
    }
    let mut allocator = allocator_after_make_filesystem(parameters, devices, &genesis);
    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
    if usage == 1 {
        return None;
    }
    let first = w1_content(seed);
    let mut previous = publish_first_file(&mut writer, &mut allocator, warmed.roots.last().expect("两代根"), FirstFile { content: &first, write_time_seconds: WRITE_TIME }, instance, &warmed.last_record_bytes).expect("第一个文件");
    let mut last = first;
    for k in 0..usage - 2 {
        let c = w1_content(seed + 1 + u64::from(k));
        previous = publish_overwrite(&mut writer, &mut allocator, &previous, FirstFile { content: &c, write_time_seconds: WRITE_TIME + 1 + u64::from(k) }, instance).expect("覆盖写");
        last = c;
    }
    Some(last)
}

struct W1History {
    name: String,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    old_last: Option<Vec<u8>>,
    old_versions_contents: Vec<Vec<u8>>,
}

fn w1_histories(filter: &str) -> Vec<W1History> {
    let mut out = Vec::new();
    for (x, y) in [(65536u32, 512u32), (8192, 512), (512, 8192), (512, 65536), (65536, 8192)] {
        for u in [1u32, 5] {
            for reuse in ["disk0", "both"] {
                for same_fsid in [false, true] {
                    let name = format!("W:x={x}:y={y}:u={u}:reuse={reuse}:fsid={}", if same_fsid { "same" } else { "diff" });
                    if !name.contains(filter) {
                        continue;
                    }
                    let old_parameters = w1_parameters(if same_fsid { NEW_FSID } else { OLD_FSID }, x);
                    let mut old: Devices = vec![(DeviceIdentity(0), fresh_device()), (DeviceIdentity(1), fresh_device())];
                    let old_last = make_and_use(&mut old, &old_parameters, u, 7);
                    let old_versions_contents: Vec<Vec<u8>> = if u >= 2 { (0..u - 1).map(|k| w1_content(7 + u64::from(k))).collect() } else { Vec::new() };
                    let devices: Devices = if reuse == "both" {
                        old
                    } else {
                        vec![(DeviceIdentity(0), clone_device(&old[0].1)), (DeviceIdentity(1), fresh_device())]
                    };
                    let base = MemoryPool { devices: devices.iter().map(|(i, d)| (*i, d.image.clone())).collect(), device_size_in_bytes: W1_DEVICE_BYTES };
                    let stream = SharedStream::retaining_contents();
                    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = devices
                        .iter()
                        .map(|(i, d)| (*i, RecordingBlockDevice::with_shared_stream(*i, clone_device(d), stream.clone())))
                        .collect();
                    let p = w1_parameters(NEW_FSID, y);
                    make_filesystem(&p, &mut recorded).expect("新 mkfs");
                    let operations = stream.retained_operations();
                    let fixed = FixedGeometry {
                        fixed_structure_slot_spacing: p.geometry.fixed_structure_slot_spacing,
                        journal_ring_bytes: p.geometry.journal_ring_bytes,
                        root_ring_slots_per_region: p.geometry.root_ring_slots_per_region,
                    };
                    let (writes, segments) = writes_and_segments(&operations, &fixed);
                    let judged_root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
                    out.push(W1History { name, base, writes, segments, judged_root_index, old_last, old_versions_contents });
                }
            }
        }
    }
    out
}

fn fsid_name(fsid: &[u8; 16]) -> &'static str {
    if *fsid == NEW_FSID { "new" } else if *fsid == OLD_FSID { "old" } else { "other" }
}

fn core_choice(reader: &dyn PoolReader) -> String {
    match choose_system_configuration(reader) {
        Ok(c) => format!("{}@{}g{}d{}", fsid_name(&c.immutable.filesystem_identifier), c.immutable.sizes.fixed_structure_slot_spacing, c.quantities.slot_generation, c.immutable.this_device.0),
        Err(e) => format!("Err:{}", variant(format!("{e:?}"))),
    }
}

fn slot_label(r: &SystemConfigurationSlotReading) -> String {
    match r {
        SystemConfigurationSlotReading::Usable(view, geometry) => format!("{}@{}g{}", fsid_name(&view.filesystem_identifier), geometry.slot_spacing, view.slot_generation),
        SystemConfigurationSlotReading::CarriesAValueThisReaderRefuses { offset_in_bytes, .. } => format!("refused@{offset_in_bytes}"),
        SystemConfigurationSlotReading::NotSelfVerified => "x".to_string(),
        SystemConfigurationSlotReading::SelfVerifiedButNotChoosable => "nc".to_string(),
    }
}

fn w1_main(filter: &str, threads: usize, limit: Option<u64>, start: u64) {
    let mkfs_clears = std::env::var("C651_MKFS_CLEARS").is_ok();
    let started = std::time::Instant::now();
    let mut total = 0u64;
    for h in w1_histories(filter) {
        let states = layer0_state_count_with_torn_in_place_overwrites(&h.base, &h.writes, &h.segments, &full_expansion);
        let end = limit.map_or(states, |l| l.min(states));
        assert!(end - start <= 200_000, "{}：单次调用 {} 态超过 20 万态的上限", h.name, end - start);
        let shape: Vec<usize> = h.segments.iter().map(Vec::len).collect();
        let kinds: Vec<&str> = h.writes.iter().map(|w| w.kind.name()).collect();
        println!("name=history w1={mkfs_clears} h={} segments={shape:?} closed_form_two_state={} states={states} range={start}..{end} kinds={kinds:?}", h.name, closed_form_state_count(&h.segments));
        let n = h.writes.len();
        let merged: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());
        let chunk = (end - start).div_ceil(threads as u64).max(1);
        std::thread::scope(|scope| {
            for t in 0..threads as u64 {
                let from = start + t * chunk;
                let to = (start + (t + 1) * chunk).min(end);
                if from >= to {
                    continue;
                }
                let h = &h;
                let merged = &merged;
                scope.spawn(move || {
                    let mut signatures: BTreeMap<String, u64> = BTreeMap::new();
                    let judged = judge_layer0_state_range(&h.base, &h.writes, &h.segments, h.judged_root_index, &[], &full_expansion, from..to, &mut |_ordinal, image, judgement| {
                        let torn_zero = image.persisted.iter().enumerate().skip(n).any(|(k, q)| *q && image.writes[k].kind == StepKind::ZeroFill);
                        let first_withheld = (0..n).find(|i| !image.persisted[*i]).unwrap_or(n);
                        let segment = h.segments.iter().position(|s| s.contains(&first_withheld)).unwrap_or(h.segments.len());
                        let readings = system_configuration_slot_readings(image);
                        let all_slot0_unusable = readings.iter().all(|(_, r)| !matches!(r[0], SystemConfigurationSlotReading::Usable(..)));
                        let checker_slots: Vec<String> = readings.iter().map(|(_, r)| format!("{}/{}", slot_label(&r[0]), slot_label(&r[1]))).collect();
                        let outcome = match &judgement.consulted.outcome {
                            RecoveryOutcome::NoFile { root } => format!("NoFile(i{},t{})", root.0 .0, root.1 .0),
                            RecoveryOutcome::FileRead { root, content } => {
                                let which = if Some(content) == h.old_last.as_ref() { "old_last" } else if h.old_versions_contents.contains(content) { "old_older" } else { "other" };
                                format!("FileRead(i{},t{}):{which}", root.0 .0, root.1 .0)
                            }
                            RecoveryOutcome::Failed { failure, .. } => format!("Failed:{}", variant(format!("{failure:?}"))),
                        };
                        let red: Vec<&str> = judgement.pool_checker.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(i, _)| *i).collect();
                        let key = format!(
                            "seg={segment} torn_zero={torn_zero} all_slot0_unusable={all_slot0_unusable} core={} outcome={outcome} checker_slots={} checker_red={red:?}",
                            core_choice(image),
                            checker_slots.join("|")
                        );
                        *signatures.entry(key).or_insert(0) += 1;
                    });
                    assert_eq!(judged, states);
                    let mut m = merged.lock().expect("锁");
                    for (k, v) in signatures {
                        *m.entry(k).or_insert(0) += v;
                    }
                });
            }
        });
        total += end - start;
        for (key, count) in merged.into_inner().expect("锁") {
            println!("name=sig w1={mkfs_clears} h={} count={count} {key}", h.name);
        }
    }
    println!("name=done w1={mkfs_clears} states={total} elapsed_seconds={:.1}", started.elapsed().as_secs_f64());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("sizes");
    if mode == "w1" {
        let threads: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
        let limit: Option<u64> = args.get(4).and_then(|s| s.parse().ok());
        let start: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
        w1_main(args.get(2).map(String::as_str).unwrap_or(""), threads, limit, start);
        return;
    }
    if mode == "w1sizes" {
        for h in w1_histories(args.get(2).map(String::as_str).unwrap_or("")) {
            let states = layer0_state_count_with_torn_in_place_overwrites(&h.base, &h.writes, &h.segments, &full_expansion);
            let shape: Vec<usize> = h.segments.iter().map(Vec::len).collect();
            println!("name=w1size h={} segments={shape:?} closed_form_two_state={} states={states}", h.name, closed_form_state_count(&h.segments));
        }
        return;
    }
    let id = args.get(2).map(String::as_str).unwrap_or("rbw");
    let threads: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    let started = std::time::Instant::now();
    let p = prepare(id);
    let total = layer0_state_count_with_torn_in_place_overwrites(&p.base, &p.writes, &p.segments, &full_expansion);
    println!(
        "history={id} mode={mode} mutation={:?} writes={} segments={} closed_form_two_state={} with_torn={} prepare_secs={:.2}",
        std::env::var("R6_MUTATION").ok(), p.writes.len(), p.segments.len(), closed_form_state_count(&p.segments), total, started.elapsed().as_secs_f64()
    );
    if mode == "sizes" {
        for (i, s) in p.segments.iter().enumerate() {
            let kinds: Vec<String> = s.iter().map(|w| format!("{}@{}", p.writes[*w].kind.name(), p.write_position[*w])).collect();
            println!("  segment {i}: {} writes {:?}", s.len(), kinds);
        }
        return;
    }
    let limit: Option<u64> = args.get(4).and_then(|s| s.parse().ok());
    let end = limit.map_or(total, |l| l.min(total));
    let start: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
    assert!(end - start <= 200_000, "单次调用不超过 20 万态");
    let merged = Mutex::new(Tally::default());
    let chunk = (end - start).div_ceil(threads as u64).max(1);
    std::thread::scope(|scope| {
        for t in 0..threads as u64 {
            let from = start + t * chunk;
            let to = (start + (t + 1) * chunk).min(end);
            if from >= to {
                continue;
            }
            let p = &p;
            let merged = &merged;
            scope.spawn(move || {
                let mut tally = Tally::default();
                let committed: BTreeMap<ModelRootKey, Option<Rc<[u8]>>> = p.committed.iter().map(|(k, v)| (*k, v.as_ref().map(|c| Rc::from(c.as_slice())))).collect();
                judge_layer0_state_range(&p.base, &p.writes, &p.segments, p.judged_root_index, &p.versions, &full_expansion, from..to, &mut |_ordinal, image, judgement| match mode {
                    "lo" => judge_lo(image, judgement, p, &committed, &mut tally),
                    "tgt" => judge_tgt(image, p, &mut tally),
                    other => panic!("未知模式 {other}"),
                });
                merged.lock().expect("锁").merge(tally);
            });
        }
    });
    let tally = merged.into_inner().expect("锁");
    println!("states_judged={} of {} range={start}..{end} elapsed_secs={:.1}", tally.states, total, started.elapsed().as_secs_f64());
    for (k, c) in &tally.classes {
        println!("{k} count={c}");
    }
}

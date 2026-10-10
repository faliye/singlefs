//! m3-formal-c655-c687-r1 攻方腿原型 c655probe（只在冻结副本的拷贝上编、跑；模型目录里留一份源码）。
//! 历史全是本原型手写的几段（不取自任何层 0 用例）；崩溃状态用 `singlefs_checker_tier::crash::judge_layer0_state_range` + `full_expansion`
//! 枚举（层 0 同一枚举域：每段都展开、段内整写子集逐个、原地覆写多一态），每个状态都跑 `judge_crash_image`（含池级 checker）。
//! 模式：
//!   sizes <历史>                       窗口的写数、段数、状态数
//!   y1 <历史> <线程> [上限] [起点]       每个第一次崩溃态：R1；只挂载 → R2a；挂载加一次发布 → R2b；表外字段逐个比；真判官（judge_mount_transparency）复判；I-5.5 判定
//!   y1s2 <历史> <第一次崩溃态序号,…> <线程>  选定的第一次崩溃态上，挂载那一段写流的每个二次崩溃态：R2 与 R1 逐字段比；I-5.5 判定
//!   fallback <盘宽> <形状,…>            mkfs 之后 N 次可写挂载（每次后接 K 次零单元发布）再写第一个文件：提交内生块回落次数、I-5.5 判定、落点
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    full_expansion, judge_layer0_state_range, layer0_state_count_with_torn_in_place_overwrites,
    Layer0StateJudgement,
};
use singlefs_checker_tier::crash_injection::{
    materialized_crash_image, mount_and_publish_once_on_the_crash_image, CrashPoint,
    WritableMountOutcome,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::PROBE_COMMIT_GENERATED_FALLBACKS;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, probe_tree_table_entries_of_root, readable_roots,
    recover, replay_journal, scan_journal, verified_system_configuration_slots, JournalPolicy,
    PoolReader, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory,
    HistoryDeviceWidth, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, RollbackTargetChoice, StepPosition,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, writes_and_segments_with_stream_indexes, CrashImage, MemoryPool,
    PublishedVersion, RetainedWrite,
};
use singlefs_harness::model::ModelRootKey;
use singlefs_harness::mount_transparency::{
    judge_mount_transparency, read_recovery_fields, ComparedVersion, FloorRaisesAtTheSecondCrash,
    MountTransparency, MountTransparencyAction,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const I55: &str = "I-5.5";

/// 各段累计的微秒（只为估时）：挂载加发布、observe、真判官读字段、挂载之后的 checker。
static PHASE_MICROS: [std::sync::atomic::AtomicU64; 4] = [const { std::sync::atomic::AtomicU64::new(0) }; 4];

fn timed<T>(phase: usize, f: impl FnOnce() -> T) -> T {
    let t = std::time::Instant::now();
    let x = f();
    PHASE_MICROS[phase].fetch_add(u64::try_from(t.elapsed().as_micros()).unwrap_or(u64::MAX), Ordering::Relaxed);
    x
}

fn small(fill_seed: u64) -> ContentChoice {
    ContentChoice { length: ContentLength::InsideOneDataUnit { selector: 200 }, fill_seed }
}

struct Spec {
    starting_point: HistoryStartingPoint,
    operations: Vec<HistoryOperation>,
    /// None = 只有 mkfs 当基线；Some(k) = 起点与前 k 步当基线，之后那几步是枚举的窗口。
    prefix: Option<usize>,
    width: HistoryDeviceWidth,
    /// 只枚举窗口里第 a 段到第 b 段之前（前面几段整段持久、并进基线；b 段起不算这段历史）：缩历史长度，留下的段每个崩溃态都枚举。
    segments: Option<(usize, usize)>,
}

fn spec(id: &str) -> Spec {
    use HistoryOperation as O;
    // 盘宽默认 4 GiB；C655_WIDTH=384 / 256 / 240 换小盘（几何取样，准入照式子判）。
    let w = std::env::var("C655_WIDTH").map_or(HistoryDeviceWidth::FourGibibytes, |name| width_of(&name));
    let s = |starting_point, operations: Vec<HistoryOperation>, prefix| Spec { starting_point, operations, prefix, width: w, segments: None };
    let (id, segments) = match id.split_once('@') {
        Some((id, range)) => {
            let (a, b) = range.split_once('-').expect("历史@a-b");
            (id, Some((a.parse().expect("a"), b.parse().expect("b"))))
        }
        None => (id, None),
    };
    let mut chosen = match id {
        // mkfs 之后第一次可写挂载（R1 落在 mkfs 那一版或挂载写到一半）。
        "s1" => s(HistoryStartingPoint::AfterMakeFilesystem, vec![O::CloseAndMountWritable], None),
        // 第一个文件之后一次覆盖写。
        "ow" => s(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7))], Some(0)),
        // 第一个文件之后重挂一次（多实例：R1 可以落在上一个实例那一版）。
        "s2" => s(HistoryStartingPoint::AfterFirstFile, vec![O::CloseAndMountWritable], Some(0)),
        // unlink 第一个文件（意图那次 + 分批走空）：崩溃态里有 deleted_inodes 不空的那几版。
        "ul" => s(HistoryStartingPoint::AfterFirstFile, vec![O::Unlink], Some(0)),
        // unlink 之后建一个 inode。
        "ci" => s(HistoryStartingPoint::AfterFirstFile, vec![O::Unlink, O::CreateInode], Some(1)),
        // 覆盖写之后回退到次新那条（谱系树记一条）。
        "rb" => s(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 })], Some(1)),
        // 三次覆盖写之后抬 F。
        "fr" => s(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::PublishOverwrite(small(8)), O::PublishOverwrite(small(9)), O::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 2 })], Some(3)),
        // 回退之后的那一版上再覆盖写一次（R1 带谱系记录、挂载在它上面）。
        "rbow" => s(HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7)), O::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 }), O::PublishOverwrite(small(9))], Some(2)),
        other => panic!("未知历史 {other}"),
    };
    chosen.segments = segments;
    chosen
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
}

fn prepare(id: &str) -> Prepared {
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
        eprintln!("  step {i}: {}", short(&format!("{o:?}"), 300));
    }
    let ops = stream.retained_operations();
    let cut = match s.prefix { None => run.operations_written_by_make_filesystem, Some(k) => marks[k] };
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], size);
    base.apply(&ops[..cut]);
    let geometry = s.width.fixed_geometry();
    let (mut writes, mut segments, stream_indexes) = writes_and_segments_with_stream_indexes(&ops[cut..], &geometry);
    if let Some((a, b)) = s.segments {
        let from = cut + stream_indexes[segments[a][0]];
        let to = if b < segments.len() { cut + stream_indexes[segments[b][0]] } else { ops.len() };
        base.apply(&ops[cut..from]);
        let (w, g, _) = writes_and_segments_with_stream_indexes(&ops[from..to], &geometry);
        writes = w;
        segments = g;
        eprintln!("  window narrowed to segments {a}..{b}: stream ops {from}..{to}");
    }
    let judged_root_index = writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).unwrap_or(0);
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
    Prepared { width: s.width, parameters, base, writes, segments, versions, judged_root_index }
}

fn short(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// 一次只读恢复连同表外字段（拿来比的那一版、系统配置）。
#[derive(Clone, Debug)]
struct Obs {
    outcome: String,
    effective: Option<RootRecord>,
    prefix_applied: usize,
    /// 拿来比的那一版（恢复落到的，或它之前那一条可读根）。
    compared: Option<RootRecord>,
    /// 拿来比的那一版根槽里算法、nonce、MAC 那 29 字节（偏移 428..457）；那一版不在根环里（由记录施加出来）时 None。
    compared_alg_nonce_mac: Option<Vec<u8>>,
    /// 拿来比的那一版树表：种类 → (树 ID, 根指针, 诞生 txg, 头 ID)。
    entries: Option<BTreeMap<u16, (u64, Vec<u8>, u64, u64)>>,
    /// 每块盘每个自证过的系统配置槽：(盘, 世代号, journal_tail, journal 实例代号, F)。
    sc_slots: Vec<(u32, u64, u64, u32, u64)>,
    /// 择到的那一份：(世代号, journal_tail, journal 实例代号, F)。
    sc_chosen: Option<(u64, u64, u32, u64)>,
    /// 环里全部自证过的记录里最大的计数器。
    max_record_counter: Option<u64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Compared {
    Recovered,
    Before,
}

fn ring_alg_nonce_mac(reader: &dyn PoolReader, root: &RootRecord) -> Option<Vec<u8>> {
    let sc = choose_system_configuration(reader).ok()?;
    let im = &sc.immutable;
    let target = target_for_publish(root.checkpoint_txg, im.sizes.root_ring_slots_per_region);
    let device = im.region_devices[usize::try_from(target.region).expect("区域")];
    let offset = slot_offset(target, im.sizes.fixed_structure_slot_spacing);
    let bytes = reader.read(device, offset, usize::try_from(im.sizes.physical_block_size).expect("槽宽"))?;
    let parsed = RootRecord::parse_slot(&bytes, &im.filesystem_identifier)?;
    (parsed == *root).then(|| bytes[428..457].to_vec())
}

fn observe(reader: &dyn PoolReader, compared: Compared) -> Obs {
    let report = recover(reader, JournalPolicy::Consult);
    let outcome = match &report.outcome {
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::FileRead { .. } => "file".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("failed:{}", short(&format!("{failure:?}"), 60)),
    };
    let mut obs = Obs { outcome, effective: None, prefix_applied: report.journal.prefix_applied, compared: None, compared_alg_nonce_mac: None, entries: None, sc_slots: Vec::new(), sc_chosen: None, max_record_counter: None };
    let Ok(sc) = choose_system_configuration(reader) else { return obs };
    obs.sc_chosen = Some((sc.quantities.slot_generation, sc.quantities.journal_tail, sc.quantities.journal_instance.0, sc.quantities.rollback_floor.0));
    let im = &sc.immutable;
    let spacing = u64::from(im.sizes.fixed_structure_slot_spacing);
    for device in reader.device_identities() {
        for slot in verified_system_configuration_slots(reader, device, spacing, &im.filesystem_identifier) {
            obs.sc_slots.push((device.0, slot.quantities.slot_generation, slot.quantities.journal_tail, slot.quantities.journal_instance.0, slot.quantities.rollback_floor.0));
        }
    }
    let Some(chosen) = choose_root(reader, &sc) else { return obs };
    let records = scan_journal(reader, &sc);
    obs.max_record_counter = records.values().map(|r| r.counter).max();
    let Ok((_, effective)) = replay_journal(reader, &chosen, im.sizes.journal_ring_bytes, &records, true) else { return obs };
    obs.effective = Some(effective);
    let target = match compared {
        Compared::Recovered => Some(effective),
        Compared::Before => readable_roots(reader, &im.region_devices, &im.sizes, &im.filesystem_identifier)
            .into_iter()
            .filter(|r| (r.checkpoint_txg, r.instance) < (effective.checkpoint_txg, effective.instance))
            .max_by_key(|r| (r.checkpoint_txg, r.instance)),
    };
    let Some(target) = target else { return obs };
    obs.compared = Some(target);
    obs.compared_alg_nonce_mac = ring_alg_nonce_mac(reader, &target);
    obs.entries = probe_tree_table_entries_of_root(reader, &target).ok().map(|entries| {
        entries.into_iter().map(|(kind, tree, root, birth, head)| (kind, (tree, root, birth, head))).collect()
    });
    obs
}

fn hex_short(bytes: &[u8]) -> String {
    let nonzero = bytes.iter().any(|b| *b != 0);
    if !nonzero { return "zero".to_string(); }
    // 指针里的位置条目：头部 50 之后 (设备 4, 槽 6, 校验和 4) × 2
    if bytes.len() == 86 {
        let slot = |at: usize| u64::from_le_bytes([bytes[at + 4], bytes[at + 5], bytes[at + 6], bytes[at + 7], bytes[at + 8], bytes[at + 9], 0, 0]);
        return format!("ptr(slot {} / {})", slot(50), slot(64));
    }
    format!("{}B nonzero", bytes.len())
}

fn sc_max_gen(o: &Obs) -> Option<u64> {
    o.sc_slots.iter().map(|s| s.1).max()
}

fn kind_name(kind: u16) -> &'static str {
    match kind { 1 => "extent", 2 => "inode", 3 => "allocation", 4 => "accounting", 6 => "livelist", 7 => "sparse", 8 => "deadlist", 9 => "deleted_inodes", 10 => "tombstone", 11 => "lineage", 12 => "logged_ops", _ => "?" }
}

/// R2 那一版与 R1 那一版的关系。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Relation {
    WrittenByTheMount,
    TheVersionOfTheFirstRecovery,
}

/// 表外字段逐个比：(字段, 判定)。判定是 same / diff 或一个取值类。
fn compare(r1: &Obs, r2: &Obs, relation: Relation, new_instance: Option<u32>, mount_sc_writes: Option<usize>) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let (Some(a), Some(b)) = (r1.compared, r2.compared) else {
        out.push(("compared".into(), format!("missing r1={} r2={}", r1.compared.is_some(), r2.compared.is_some())));
        return out;
    };
    let v = |same: bool| if same { "same".to_string() } else { "diff".to_string() };
    out.push(("root.mapping_root".into(), v(a.mapping_root == b.mapping_root)));
    out.push(("root.allocation_record_tree_root".into(), format!("{} (r1 {} -> r2 {})", v(a.allocation_record_tree_root == b.allocation_record_tree_root),
        if a.allocation_record_tree_root == singlefs_core::pointer::NodePointer::empty_root() { "empty" } else { "nonzero" },
        if b.allocation_record_tree_root == singlefs_core::pointer::NodePointer::empty_root() { "empty" } else { "nonzero" })));
    out.push(("root.alg_nonce_mac".into(), match (&r1.compared_alg_nonce_mac, &r2.compared_alg_nonce_mac) {
        (Some(x), Some(y)) => format!("{} r1={} r2={}", v(x == y), hex_short(x), hex_short(y)),
        (x, y) => format!("not_in_ring r1={} r2={}", x.is_some(), y.is_some()),
    }));
    match (&r1.entries, &r2.entries) {
        (Some(e1), Some(e2)) => {
            for kind in [3u16, 4, 6, 7, 8, 9, 10, 11, 12] {
                let x = e1.get(&kind).map(|e| e.1.clone());
                let y = e2.get(&kind).map(|e| e.1.clone());
                let shape = |p: &Option<Vec<u8>>| match p { None => "absent".to_string(), Some(p) => if p.iter().all(|b| *b == 0) { "zero".into() } else { "nonzero".into() } };
                out.push((format!("tt.{}.root", kind_name(kind)), format!("{} (r1 {} -> r2 {})", v(x == y), shape(&x), shape(&y))));
            }
        }
        (x, y) => out.push(("tt.*".into(), format!("entries r1={} r2={}", x.is_some(), y.is_some()))),
    }
    out.push(("sc.max_slot_generation_delta".into(), match (sc_max_gen(r1), sc_max_gen(r2)) {
        (Some(x), Some(y)) => format!("{:+}", i128::from(y) - i128::from(x)),
        _ => "missing".into(),
    }));
    if let (Some(x), Some(y), Some(n)) = (sc_max_gen(r1), sc_max_gen(r2), mount_sc_writes) {
        out.push(("sc.max_slot_generation_delta_vs_mount_sc_writes".into(), format!("delta {:+} writes {n}", i128::from(y) - i128::from(x))));
    }
    if let (Some(c2), Some(m)) = (r2.sc_chosen, r2.max_record_counter) {
        out.push(("sc.chosen.journal_tail_minus_r2_max_record_counter".into(), format!("{:+}", i128::from(c2.1) - i128::from(m))));
    }
    if let (Some(c1), Some(c2)) = (r1.sc_chosen, r2.sc_chosen) {
        out.push(("sc.chosen.journal_tail".into(), format!("{} delta {:+}", v(c1.1 == c2.1), i128::from(c2.1) - i128::from(c1.1))));
        out.push(("sc.chosen.journal_instance".into(), format!("{} r2_is_new_instance={} r2_is_r2_root_instance={}", v(c1.2 == c2.2), Some(c2.2) == new_instance, c2.2 == b.instance.0)));
        out.push(("sc.chosen.rollback_floor".into(), format!("{} r2_le_r2_root_F={}", v(c1.3 == c2.3), c2.3 <= b.rollback_floor.0)));
    }
    let tag = match relation { Relation::WrittenByTheMount => "mount_version", Relation::TheVersionOfTheFirstRecovery => "v1" };
    out.into_iter().map(|(f, verdict)| (format!("{tag} {f}"), verdict)).collect()
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
            let x = example();
            if !x.is_empty() { e.push(x); }
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

fn verdict_class(verdicts: &[(&'static str, InvariantVerdict)], invariant: &str) -> String {
    match verdicts.iter().find(|(name, _)| *name == invariant).map(|(_, v)| v) {
        None => "absent".into(),
        Some(InvariantVerdict::Holds) => "holds".into(),
        Some(InvariantVerdict::Violated(d)) => format!("VIOLATED:{}", short(d, 160)),
        Some(InvariantVerdict::NotApplicable(r)) => format!("not_applicable:{}", short(r, 80)),
        Some(InvariantVerdict::PartiallyJudged(d)) => format!("partially:{}", short(d, 160)),
    }
}

fn red_list(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<&'static str> {
    verdicts.iter().filter(|(_, v)| matches!(v, InvariantVerdict::Violated(_))).map(|(n, _)| *n).collect()
}

fn describe(o: &Obs) -> String {
    let key = |r: &Option<RootRecord>| r.map(|r| (r.checkpoint_txg.0, r.instance.0));
    let entries = o.entries.as_ref().map(|e| {
        [9u16, 10, 11, 3].iter().map(|k| format!("{}={}", kind_name(*k), e.get(k).map_or("absent".to_string(), |x| hex_short(&x.1)))).collect::<Vec<_>>().join(",")
    });
    format!(
        "outcome={} eff={:?} cmp={:?} prefix={} map={} art={} anm={:?} tt[{}] sc_chosen(gen,tail,inst,F)={:?} sc_slots={:?}",
        short(&o.outcome, 50), key(&o.effective), key(&o.compared), o.prefix_applied,
        o.compared.map_or("-".into(), |r| format!("{:?}", short(&format!("{:?}", r.mapping_root.locations), 60))),
        o.compared.map_or("-".into(), |r| (r.allocation_record_tree_root == singlefs_core::pointer::NodePointer::empty_root()).then_some("empty").unwrap_or("nonzero").to_string()),
        o.compared_alg_nonce_mac.as_deref().map(hex_short), entries.unwrap_or_default(), o.sc_chosen, o.sc_slots
    )
}

fn crash_point() -> CrashPoint {
    CrashPoint { segment_index: 0, persisted_within_the_segment: Vec::new(), step: StepPosition::StartingPoint, operation_kind: None }
}

fn r1_tags(r1: &Obs) -> String {
    let queue = r1.entries.as_ref().and_then(|e| e.get(&9)).map_or("no_entries", |e| if e.1.iter().all(|b| *b == 0) { "queue_empty" } else { "queue_nonempty" });
    format!("r1={} {queue} r1_rebuilt={}", r1.outcome.split(':').next().unwrap_or(""), r1.prefix_applied > 0)
}

fn judge_y1(ordinal: u64, image: &CrashImage<'_>, judgement: &Layer0StateJudgement, p: &Prepared, tally: &mut Tally) {
    tally.states += 1;
    tally.add(format!("I55 first_crash {}", verdict_class(&judgement.pool_checker, I55)), || format!("ordinal={ordinal} red={:?}", red_list(&judgement.pool_checker)));
    let crash = materialized_crash_image(image);
    let r1 = observe(&crash, Compared::Recovered);
    let tags = r1_tags(&r1);
    tally.add(format!("CLASS {tags}"), String::new);
    let run = timed(0, || mount_and_publish_once_on_the_crash_image(&crash, &p.parameters, execution_of(p.width), HistorySeed(ordinal), &crash_point()));
    if let WritableMountOutcome::MountRefused { member, .. } = &run.outcome {
        tally.add(format!("MOUNT refused {}", short(member, 80)), String::new);
        return;
    }
    let geometry = p.width.fixed_geometry();
    let (mount_writes, _, _) = writes_and_segments_with_stream_indexes(&run.mount_operations, &geometry);
    let mut after_mount = crash.clone();
    after_mount.apply_writes(&mount_writes);
    let mount_sc = mount_writes.iter().filter(|w| w.kind == StepKind::SystemConfigurationSlot).count();
    let floor = run.floor_raised_by_the_mount.expect("挂载做成");
    let continuation = run.user_visible_tree_root_pointers_of_the_continuation.as_ref().expect("挂载做成");
    let fr1 = timed(2, || read_recovery_fields(&crash, ComparedVersion::TheRecoveredVersion));
    let new_instance = fr1.highest_system_configuration_instance.max(fr1.highest_root_instance_in_the_ring).map(|i| i.0 + 1);
    let fr2a = timed(2, || read_recovery_fields(&after_mount, ComparedVersion::TheRecoveredVersion));
    let real = judge_mount_transparency(&fr1, &fr2a, MountTransparencyAction::MountAlone(floor), continuation);
    tally.add(format!("REAL mount {}", real_class(&real)), String::new);
    let r2a = timed(1, || observe(&after_mount, Compared::Recovered));
    for (field, verdict) in compare(&r1, &r2a, Relation::WrittenByTheMount, new_instance, Some(mount_sc)) {
        tally.add(format!("FIELD action=mount {tags} {field} = {verdict}"), || format!("ordinal={ordinal}\n      R1: {}\n      R2: {}", describe(&r1), describe(&r2a)));
    }
    if std::env::var("C655_CHECK_AFTER_MOUNT").is_ok() {
        let after = timed(3, || check_pool_image(&after_mount));
        tally.add(format!("I55 after_mount {}", verdict_class(&after, I55)), || format!("ordinal={ordinal} red={:?}", red_list(&after)));
    }
    if let WritableMountOutcome::Published = run.outcome {
        let fr2b = read_recovery_fields(&run.pool_after, ComparedVersion::TheVersionBeforeTheRecoveredOne);
        let real = judge_mount_transparency(&fr1, &fr2b, MountTransparencyAction::MountAndOnePublish(floor), continuation);
        tally.add(format!("REAL mount+publish {}", real_class(&real)), String::new);
        let r2b = observe(&run.pool_after, Compared::Before);
        for (field, verdict) in compare(&r1, &r2b, Relation::WrittenByTheMount, new_instance, None) {
            tally.add(format!("FIELD action=mount+publish {tags} {field} = {verdict}"), || format!("ordinal={ordinal}\n      R1: {}\n      R2: {}", describe(&r1), describe(&r2b)));
        }
    } else {
        tally.add("MOUNT published=false".into(), String::new);
    }
}

fn real_class(real: &MountTransparency) -> String {
    match real {
        MountTransparency::NotJudged(reason) => format!("not_judged:{}", reason.name()),
        MountTransparency::Judged(_) => {
            let violated: Vec<String> = real.violated_fields().iter().map(|v| format!("{:?}", v.field)).collect();
            if violated.is_empty() { "judged_all_held_or_not_judged".into() } else { format!("VIOLATED {violated:?}") }
        }
    }
}

fn judge_second_crashes(first_ordinal: u64, image: &CrashImage<'_>, p: &Prepared, threads: usize, tally: &mut Tally) {
    let crash = materialized_crash_image(image);
    let r1 = observe(&crash, Compared::Recovered);
    let tags = r1_tags(&r1);
    let run = mount_and_publish_once_on_the_crash_image(&crash, &p.parameters, execution_of(p.width), HistorySeed(first_ordinal), &crash_point());
    if let WritableMountOutcome::MountRefused { member, .. } = &run.outcome {
        tally.add(format!("MOUNT refused {}", short(member, 80)), String::new);
        return;
    }
    let geometry = p.width.fixed_geometry();
    let (mount_writes, mount_segments, _) = writes_and_segments_with_stream_indexes(&run.mount_operations, &geometry);
    let judged = mount_writes.iter().rposition(|w| w.kind == StepKind::RootRecordFua).expect("挂载写过根槽");
    let total = layer0_state_count_with_torn_in_place_overwrites(&crash, &mount_writes, &mount_segments, &full_expansion);
    println!("first_crash_ordinal={first_ordinal} {tags} mount_writes={} mount_segments={} closed_form_two_state={} second_crash_states={total}", mount_writes.len(), mount_segments.len(), closed_form_state_count(&mount_segments));
    let fr1 = read_recovery_fields(&crash, ComparedVersion::TheRecoveredVersion);
    let new_instance = fr1.highest_system_configuration_instance.max(fr1.highest_root_instance_in_the_ring).map(|i| i.0 + 1);
    let continuation = run.user_visible_tree_root_pointers_of_the_continuation.as_ref().expect("挂载做成");
    let merged = Mutex::new(Tally::default());
    let chunk = total.div_ceil(threads as u64).max(1);
    std::thread::scope(|scope| {
        for t in 0..threads as u64 {
            let (from, to) = (t * chunk, ((t + 1) * chunk).min(total));
            if from >= to { continue; }
            let (crash, mount_writes, mount_segments, merged, r1, fr1, tags, run) = (&crash, &mount_writes, &mount_segments, &merged, &r1, &fr1, &tags, &run);
            scope.spawn(move || {
                let mut local = Tally::default();
                let _ = judge_layer0_state_range(crash, mount_writes, mount_segments, judged, &p.versions, &full_expansion, from..to, &mut |ordinal, image2, judgement| {
                    local.states += 1;
                    local.add(format!("I55 second_crash {}", verdict_class(&judgement.pool_checker, I55)), || format!("first={first_ordinal} second={ordinal} red={:?}", red_list(&judgement.pool_checker)));
                    let crash2 = materialized_crash_image(image2);
                    let r2 = observe(&crash2, Compared::Recovered);
                    let relation = if r2.effective.map(|r| (r.checkpoint_txg, r.instance)) == r1.effective.map(|r| (r.checkpoint_txg, r.instance)) { Relation::TheVersionOfTheFirstRecovery } else { Relation::WrittenByTheMount };
                    let persisted = &image2.persisted[..mount_writes.len()];
                    let floor_raises = run.floor_raises_in_the_mount_writes.clone().map_or(FloorRaisesAtTheSecondCrash::BoundaryNotLocatedInTheMountWrites, |f| f.at_a_second_crash(persisted));
                    let fr2 = read_recovery_fields(&crash2, ComparedVersion::TheRecoveredVersion);
                    let real = judge_mount_transparency(fr1, &fr2, MountTransparencyAction::SecondCrashInsideTheMount(floor_raises), continuation);
                    local.add(format!("REAL second_crash {}", real_class(&real)), String::new);
                    for (field, verdict) in compare(r1, &r2, relation, new_instance, Some(persisted.iter().zip(mount_writes.iter()).filter(|(q, w)| **q && w.kind == StepKind::SystemConfigurationSlot).count())) {
                        local.add(format!("FIELD action=second_crash {tags} {field} = {verdict}"), || format!("first={first_ordinal} second={ordinal}\n      R1: {}\n      R2: {}", describe(r1), describe(&r2)));
                    }
                });
                merged.lock().expect("锁").merge(local);
            });
        }
    });
    tally.merge(merged.into_inner().expect("锁"));
}

fn width_of(name: &str) -> HistoryDeviceWidth {
    match name { "4g" => HistoryDeviceWidth::FourGibibytes, "384" => HistoryDeviceWidth::UnitAreaOf384Slots, "256" => HistoryDeviceWidth::UnitAreaOf256Slots, "240" => HistoryDeviceWidth::UnitAreaOf240Slots, other => panic!("未知盘宽 {other}") }
}

/// fallback <盘宽> m<N>z<K>,…：mkfs 之后 N 次（关会话、可写挂载，再接 K 次零单元发布），最后写第一个文件。
fn fallback(width_name: &str, shapes: &str) {
    use HistoryOperation as O;
    let width = width_of(width_name);
    for shape in shapes.split(',') {
        let (n, rest) = shape.trim_start_matches('m').split_once('z').expect("m<N>z<K>[o<J>]");
        let (k, j) = rest.split_once('o').map_or((rest, "0"), |(k, j)| (k, j));
        let (n, k, j): (usize, usize, usize) = (n.parse().expect("N"), k.parse().expect("K"), j.parse().expect("J"));
        let mut operations = Vec::new();
        for _ in 0..n {
            operations.push(O::CloseAndMountWritable);
            operations.extend(std::iter::repeat_n(O::PublishWithoutUnits, k));
        }
        operations.push(O::PublishFirstFile(small(5)));
        // 阳性对照用：第一个文件之后再接 J 次满单元覆盖写（C655_SKIP_ADMISSION=1 时关掉准入，只供证计数器会动）。
        operations.extend((0..j).map(|i| O::PublishOverwrite(ContentChoice { length: ContentLength::OneByteBelowDataUnitPayloadCapacity, fill_seed: 100 + i as u64 })));
        let h = GeneratedHistory { seed: HistorySeed(0), starting_point: HistoryStartingPoint::AfterMakeFilesystem, operations };
        let stream = SharedStream::retaining_contents();
        PROBE_COMMIT_GENERATED_FALLBACKS.store(0, Ordering::Relaxed);
        let mut fallbacks_after_each_step: Vec<u64> = Vec::new();
        let mut execution = execution_of(width);
        if std::env::var("C655_SKIP_ADMISSION").is_ok() {
            execution.space_admission = SpaceAdmission::SkippedByTheTestOnlySwitch;
        }
        let run = execute_history_with(&h, execution, &stream, &mut |_observation| {
            fallbacks_after_each_step.push(PROBE_COMMIT_GENERATED_FALLBACKS.load(Ordering::Relaxed));
        });
        let total = PROBE_COMMIT_GENERATED_FALLBACKS.load(Ordering::Relaxed);
        // 写第一个文件那一步之前（起点之后第 n × (1 + k) 步）的累计：观察者在起点与每一步之后各调一次。
        let before_first_file = fallbacks_after_each_step.get(n * (1 + k)).copied().unwrap_or(0);
        let after_first_file = fallbacks_after_each_step.get(n * (1 + k) + 1).copied().unwrap_or(total);
        let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], width.device_bytes());
        pool.apply(&stream.retained_operations());
        let verdicts = check_pool_image(&pool);
        let last = run.outcomes.last().map(|o| short(&format!("{o:?}"), 140)).unwrap_or_default();
        let refused = run.outcomes.iter().filter(|o| { let d = format!("{o:?}"); d.contains("Refused") || d.contains("Error") || d.contains("refused") }).count();
        let placement = newest_root_placements(&pool);
        println!("name=fallback width={width_name} shape={shape} steps={} ending={} fallbacks_total={total} fallbacks_in_first_file={} steps_with_refusal={refused} i55={} red={:?} {placement} last={last}",
            run.outcomes.len(), short(&format!("{:?}", run.ending), 60), after_first_file - before_first_file, verdict_class(&verdicts, I55), red_list(&verdicts));
    }
}

/// 最新那条根的树表单元落点与 extent 树第一片数据（相对单元区起始槽号 U）。
fn newest_root_placements(pool: &MemoryPool) -> String {
    let Ok(sc) = choose_system_configuration(pool) else { return "no_sc".into() };
    let Some(root) = choose_root(pool, &sc) else { return "no_root".into() };
    let u = { let b = pool.read(DeviceIdentity(0), DeviceOffsetInBytes(0), 4096).unwrap_or_default(); if b.len() >= 425 { u64::from_le_bytes(b[417..425].try_into().expect("8")) } else { 0 } };
    let tt: Vec<u64> = root.tree_table.locations.iter().map(|l| l.slot.0.saturating_sub(u)).collect();
    format!("U={u} newest=({}, {}) tree_table_rel_U={tt:?}", root.checkpoint_txg.0, root.instance.0)
}

/// 阳性对照：与 harness 用例 `commit_generated_units_fall_back_when_the_segment_runs_out` 同一种造法（内存稀疏盘：mkfs、取号、暖机、新池新建文件，
/// 再把开放段剩下的槽占满、每个全空段占掉段首一槽），之后一次覆盖写必须走回落、计数器必须动。
fn fallback_control() {
    use singlefs_core::address::SlotNumber;
    use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
    use singlefs_core::block_device::PhysicalBlockSizeInBytes;
    use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
    use singlefs_core::transaction::{acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter};
    use singlefs_harness::memory_pool::SparseBlockDevice;
    let width = HistoryDeviceWidth::FourGibibytes;
    let parameters = width.parameters();
    let bytes = width.device_bytes();
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = (0..2u32)
        .map(|d| (DeviceIdentity(d), SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512))))
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(vec![DeviceFreeMap::new(DeviceIdentity(0), bytes), DeviceFreeMap::new(DeviceIdentity(1), bytes)]);
    allocator.mark_format_time_units(Placement { slot: INSTANCE_TABLE_SLOT, span: 2 }, Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 });
    let content: Vec<u8> = (0..300u32).map(|i| (i % 251) as u8).collect();
    let mut pool = PoolWriter::new(&parameters, &mut devices);
    let instance = acquire_instance(&mut pool).expect("取号");
    let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
    let first = publish_first_file(&mut pool, &mut allocator, warm.roots.last().expect("根"), FirstFile { content: &content, write_time_seconds: 1 }, instance, &warm.last_record_bytes).expect("第一个文件");
    let before = PROBE_COMMIT_GENERATED_FALLBACKS.load(Ordering::Relaxed);
    let open = allocator.open_segment().expect("开放段开着");
    let start = allocator.devices[0].unit_area_start().slot().0;
    for device_map in &mut allocator.devices {
        for slot in open.0..open.0 + singlefs_format::CLUSTER_SEGMENT_SLOTS {
            if device_map.is_free(SlotNumber(slot)) { device_map.mark_allocated(SlotNumber(slot), 1); }
        }
        let full_segments = device_map.unit_area_slots() / singlefs_format::CLUSTER_SEGMENT_SLOTS;
        for segment_index in 0..full_segments {
            if device_map.segment_is_empty(usize::try_from(segment_index).expect("段号")) {
                device_map.mark_allocated(SlotNumber(start + segment_index * singlefs_format::CLUSTER_SEGMENT_SLOTS), 1);
            }
        }
    }
    let second: Vec<u8> = (0..4100u32).map(|i| (i % 249) as u8).collect();
    let result = publish_overwrite(&mut pool, &mut allocator, &first, FirstFile { content: &second, write_time_seconds: 2 }, instance);
    let after = PROBE_COMMIT_GENERATED_FALLBACKS.load(Ordering::Relaxed);
    println!("name=fallback-control publish_ok={} fallbacks_before={before} fallbacks_after={after}", result.is_ok());
}

fn print_tally(tally: &Tally) {
    for (k, c) in &tally.classes {
        println!("{k} count={c}");
        for x in &tally.examples[k] {
            println!("    e.g. {x}");
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("sizes");
    let started = std::time::Instant::now();
    if mode == "fallback-control" {
        fallback_control();
        return;
    }
    if mode == "fallback" {
        fallback(&args[2], &args[3]);
        println!("elapsed_secs={:.1}", started.elapsed().as_secs_f64());
        return;
    }
    let id = args.get(2).map(String::as_str).unwrap_or("ow");
    let p = prepare(id);
    let total = layer0_state_count_with_torn_in_place_overwrites(&p.base, &p.writes, &p.segments, &full_expansion);
    println!("history={id} mode={mode} writes={} segments={} closed_form_two_state={} with_torn={total} prepare_secs={:.2}",
        p.writes.len(), p.segments.len(), closed_form_state_count(&p.segments), started.elapsed().as_secs_f64());
    match mode {
        "sizes" => {
            for (i, s) in p.segments.iter().enumerate() {
                let kinds: Vec<&str> = s.iter().map(|w| p.writes[*w].kind.name()).collect();
                println!("  segment {i}: {} writes {:?}", s.len(), kinds);
            }
        }
        "y1" => {
            let threads: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
            let end = args.get(4).and_then(|s| s.parse().ok()).map_or(total, |l: u64| l.min(total));
            let start: u64 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
            let merged = Mutex::new(Tally::default());
            let chunk = (end - start).div_ceil(threads as u64).max(1);
            std::thread::scope(|scope| {
                for t in 0..threads as u64 {
                    let (from, to) = (start + t * chunk, (start + (t + 1) * chunk).min(end));
                    if from >= to { continue; }
                    let (p, merged) = (&p, &merged);
                    scope.spawn(move || {
                        let mut tally = Tally::default();
                        let _ = judge_layer0_state_range(&p.base, &p.writes, &p.segments, p.judged_root_index, &p.versions, &full_expansion, from..to, &mut |ordinal, image, judgement| {
                            judge_y1(ordinal, image, judgement, p, &mut tally);
                        });
                        merged.lock().expect("锁").merge(tally);
                    });
                }
            });
            let tally = merged.into_inner().expect("锁");
            println!("states_judged={} of {total} range={start}..{end} elapsed_secs={:.1}", tally.states, started.elapsed().as_secs_f64());
            let ms = |i: usize| PHASE_MICROS[i].load(Ordering::Relaxed) as f64 / 1000.0 / (tally.states.max(1) as f64);
            println!("per_state_ms mount_and_publish={:.0} observe_r2a={:.0} read_recovery_fields_x2={:.0} checker_after_mount={:.0}", ms(0), ms(1), ms(2), ms(3));
            print_tally(&tally);
        }
        "y1s2" => {
            let ordinals: Vec<u64> = args[3].split(',').map(|s| if s == "last" { total - 1 } else { s.parse().expect("序号") }).collect();
            let threads: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
            let mut tally = Tally::default();
            for ordinal in ordinals {
                let _ = judge_layer0_state_range(&p.base, &p.writes, &p.segments, p.judged_root_index, &p.versions, &full_expansion, ordinal..ordinal + 1, &mut |ordinal, image, _judgement| {
                    judge_second_crashes(ordinal, image, &p, threads, &mut tally);
                });
            }
            println!("second_crash_states_judged={} elapsed_secs={:.1}", tally.states, started.elapsed().as_secs_f64());
            print_tally(&tally);
        }
        other => panic!("未知模式 {other}"),
    }
}

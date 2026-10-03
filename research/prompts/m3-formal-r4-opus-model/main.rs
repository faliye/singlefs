//! m3-formal-r4 攻方腿原型（只在冻结副本的拷贝上编、跑；不进仓）。
//! 每个崩溃状态（层 0 同一枚举域：每段都展开、原地覆写取第三态）上：
//!   R1 = 崩溃镜像上的只读恢复（择系统配置、择根、扫环、重放、走读），逐字段取出；
//!   对每个用户动作 A：在崩溃镜像的副本上起可写挂载、做 A，再只读恢复一次得 R2，与 R1 逐字段比。
//! 历史是本原型手写的几段（不取自任何层 0 用例）。
use std::collections::BTreeMap;
use std::sync::Mutex;

use singlefs_checker_tier::crash::{
    full_expansion, judge_layer0_state_range, layer0_state_count_with_torn_in_place_overwrites,
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
    root_is_abandoned_by_the_instance_table, scan_journal, tree_table_has_no_entries, walk_to_file,
    JournalPolicy, PoolReader, RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::transaction::{publish_first_file, FirstFile, PoolVersion, PoolWriter};
use singlefs_harness::history::{
    execute_history_with, ContentChoice, ContentLength, FloorTargetChoice, GeneratedHistory,
    HistoryDeviceWidth, HistoryExecution, HistoryOperation, HistorySeed, HistoryStartingPoint,
    PerStepChecker, RollbackTargetChoice,
};
use singlefs_harness::memory_pool::{
    closed_form_state_count, newest_persisted_root, writes_and_segments_with_stream_indexes, CrashImage, MemoryPool,
    PublishedVersion, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const WIDTH: HistoryDeviceWidth = HistoryDeviceWidth::UnitAreaOf384Slots;

fn small(fill_seed: u64) -> ContentChoice {
    ContentChoice {
        length: ContentLength::InsideOneDataUnit { selector: 200 },
        fill_seed,
    }
}

fn history(id: &str) -> GeneratedHistory {
    use HistoryOperation as O;
    let (starting_point, operations) = match id {
        "h0" => (HistoryStartingPoint::AfterFirstFile, vec![]),
        "h1" => (HistoryStartingPoint::AfterFirstFile, vec![O::PublishOverwrite(small(7))]),
        "h2" => (HistoryStartingPoint::AfterFirstFile, vec![O::CloseAndMountWritable]),
        "h3" => (
            HistoryStartingPoint::AfterMakeFilesystem,
            vec![O::CloseAndMountWritable, O::PublishFirstFile(small(7))],
        ),
        "h4" => (
            HistoryStartingPoint::AfterFirstFile,
            vec![
                O::PublishOverwrite(small(7)),
                O::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 }),
            ],
        ),
        "h5" => (
            HistoryStartingPoint::AfterFirstFile,
            vec![O::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 2 })],
        ),
        "h6" => (
            HistoryStartingPoint::AfterFirstFile,
            vec![O::PublishOverwrite(small(7)), O::PublishOverwrite(small(9))],
        ),
        // 以下几段带一个固定前缀（不枚举、不摆崩溃状态，只当基线），只在前缀之后的那几步上枚举。
        "s1" => (HistoryStartingPoint::AfterMakeFilesystem, vec![O::CloseAndMountWritable]),
        "s2" => (HistoryStartingPoint::AfterFirstFile, vec![O::CloseAndMountWritable]),
        "s3" => (
            HistoryStartingPoint::AfterFirstFile,
            vec![
                O::PublishOverwrite(small(7)),
                O::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest: 1 }),
            ],
        ),
        "s5" => (
            HistoryStartingPoint::AfterMakeFilesystem,
            vec![O::CloseAndMountWritable, O::PublishWithoutUnits],
        ),
        "s6" => (
            HistoryStartingPoint::AfterFirstFile,
            vec![O::CloseAndMountWritable, O::CloseAndMountWritable],
        ),
        "s7" => (
            HistoryStartingPoint::AfterFirstFile,
            vec![O::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 1 })],
        ),
        // 根环转满之后的可写挂载：起点之后 24 次覆盖写都当固定前缀，只在挂载那一步上枚举。
        "s10" => {
            let mut operations: Vec<HistoryOperation> = (0..24).map(|i| O::PublishOverwrite(small(100 + i))).collect();
            operations.push(O::CloseAndMountWritable);
            (HistoryStartingPoint::AfterFirstFile, operations)
        }
        other => panic!("未知历史 {other}"),
    };
    GeneratedHistory { seed: HistorySeed(0), starting_point, operations }
}

/// 固定前缀：None = 只有 mkfs 当基线；Some(0) = 起点那一步也当基线；Some(k) = 起点与前 k 步当基线。
fn fixed_prefix(id: &str) -> Option<usize> {
    match id {
        "s2" | "s7" => Some(0),
        "s3" | "s6" => Some(1),
        "s10" => Some(24),
        _ => None,
    }
}

struct Prepared {
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    versions: Vec<PublishedVersion>,
    judged_root_index: usize,
    parameters: MakeFilesystemParameters,
    /// 基线（固定前缀之后、枚举那几步之前）与全部写都落之后，只读恢复读回的内容：语义级「按前缀丢」在这一步上的允许集合。
    before: Obs,
    after: Obs,
}

fn prepare(id: &str) -> Prepared {
    let h = history(id);
    let stream = SharedStream::retaining_contents();
    let execution = HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: WIDTH,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    };
    let mut marks: Vec<usize> = Vec::new();
    let run = execute_history_with(&h, execution, &stream, &mut |_| marks.push(stream.operation_count()));
    eprintln!("history {id}: ending {:?}", run.ending);
    for (i, o) in run.outcomes.iter().enumerate() {
        eprintln!("  step {i}: {o:?}");
    }
    let ops = stream.retained_operations();
    let mkfs = match fixed_prefix(id) {
        None => run.operations_written_by_make_filesystem,
        Some(k) => marks[k],
    };
    eprintln!("  marks {marks:?} mkfs_ops={} base_ops={mkfs}", run.operations_written_by_make_filesystem);
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], WIDTH.device_bytes());
    base.apply(&ops[..mkfs]);
    let (writes, segments, _) = writes_and_segments_with_stream_indexes(&ops[mkfs..], &WIDTH.fixed_geometry());
    let versions = run
        .committed_versions_at_the_end
        .iter()
        .filter_map(|(key, content)| {
            content.as_ref().map(|c| PublishedVersion {
                instance: InstanceGeneration(u32::try_from(key.instance.0).expect("实例")),
                checkpoint_txg: CheckpointTxg(key.checkpoint_txg.0),
                content: c.clone(),
            })
        })
        .collect();
    let judged_root_index = writes
        .iter()
        .rposition(|w| w.kind == StepKind::RootRecordFua)
        .expect("至少一次根槽写");
    let before = observe(&base);
    let mut final_pool = base.clone();
    final_pool.apply_writes(&writes);
    let after = observe(&final_pool);
    eprintln!("  before: {}\n  after: {}", describe(&before), describe(&after));
    Prepared { base, writes, segments, versions, judged_root_index, parameters: WIDTH.parameters(), before, after }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Obs {
    outcome: String,
    content: Option<Vec<u8>>,
    chosen: Option<RootRecord>,
    effective: Option<RootRecord>,
    prefix_applied: usize,
    max_applied_transaction: u64,
    quantities: Option<String>,
    rows: Option<Vec<(u32, u64, u64)>>,
    allocations: Option<Vec<String>>,
    floor: Option<u64>,
    ring: Vec<(u64, u32)>,
    candidates: Vec<(u64, u32)>,
    /// 环里与走到的那条根同实例、txg 小一的那条根读回的内容（挂载之后那次发布之前的那一版：暖机的最后一版）。
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
        outcome,
        content,
        chosen: None,
        effective: None,
        prefix_applied: report.journal.prefix_applied,
        max_applied_transaction: report.journal.maximum_applied_transaction,
        quantities: None,
        rows: None,
        allocations: None,
        floor: None,
        ring: Vec::new(),
        candidates: Vec::new(),
        content_one_version_before: None,
    };
    let Ok(sc) = choose_system_configuration(reader) else { return obs };
    obs.quantities = Some(format!("{:?}", sc.quantities));
    let im = &sc.immutable;
    let floor = effective_rollback_floor(reader, &im.region_devices, &im.sizes, &im.filesystem_identifier);
    obs.floor = Some(floor.0);
    let roots = readable_roots(reader, &im.region_devices, &im.sizes, &im.filesystem_identifier);
    let mut ring: Vec<(u64, u32)> = roots.iter().map(|r| (r.checkpoint_txg.0, r.instance.0)).collect();
    ring.sort_unstable();
    ring.dedup();
    obs.ring = ring;
    let Some(chosen) = choose_root(reader, &sc) else { return obs };
    obs.chosen = Some(chosen);
    let records = scan_journal(reader, &sc);
    if let Ok((_, effective)) = replay_journal(reader, &chosen, im.sizes.journal_ring_bytes, &records, true) {
        obs.effective = Some(effective);
        let table = instance_table_of_root(reader, &effective);
        obs.rows = table.as_ref().map(|t| {
            t.rows.iter().map(|r| (r.instance.0, r.selected_root_txg.0, r.applied_transaction_high_water)).collect()
        });
        obs.allocations = allocation_records_under_root(reader, &effective).ok().map(|mut a| {
            a.sort_unstable();
            a.iter().map(|r| format!("{r:?}")).collect()
        });
        let mut candidates: Vec<(u64, u32)> = roots
            .iter()
            .filter(|r| table.as_ref().is_none_or(|t| !root_is_abandoned_by_the_instance_table(r, t)))
            .filter(|r| r.checkpoint_txg >= floor)
            .filter(|r| matches!(tree_table_has_no_entries(reader, r), Ok(false)))
            .map(|r| (r.checkpoint_txg.0, r.instance.0))
            .collect();
        candidates.sort_unstable();
        candidates.dedup();
        obs.candidates = candidates;
        if let Some(before) = roots.iter().find(|r| {
            r.instance == effective.instance && r.checkpoint_txg.0 + 1 == effective.checkpoint_txg.0
        }) {
            let mut fallbacks = 0;
            obs.content_one_version_before = walk_to_file(reader, before, &mut fallbacks).ok();
        }
    }
    obs
}

fn devices_of(pool: &MemoryPool, parameters: &MakeFilesystemParameters) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    pool.devices
        .iter()
        .map(|(identity, sectors)| {
            let mut device = SparseBlockDevice::new(
                pool.device_size_in_bytes,
                PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
            );
            device.image = sectors.clone();
            (*identity, device)
        })
        .collect()
}

fn pool_of(devices: Vec<(DeviceIdentity, SparseBlockDevice)>, size: u64) -> MemoryPool {
    MemoryPool { devices: devices.into_iter().map(|(i, d)| (i, d.image)).collect(), device_size_in_bytes: size }
}

const ACTIONS: [&str; 4] = ["mount", "mount+write", "mount+unmount", "mount+mount"];

fn post_content() -> Vec<u8> {
    (0..321u32).map(|i| u8::try_from(i % 251).expect("小") ^ 0x5a).collect()
}

/// 在 `crash` 的副本上做动作 `action`，交回（结局说明，之后的池）。
fn act(crash: &MemoryPool, parameters: &MakeFilesystemParameters, action: &str) -> (String, Option<MemoryPool>) {
    let mut devices = devices_of(crash, parameters);
    let mounted = match mount_writable_with_space_admission(parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
        Ok(m) => m,
        Err(e) => return (format!("mount_refused:{}", short(&format!("{e:?}"))), None),
    };
    let size = crash.device_size_in_bytes;
    match action {
        "mount" => ("ok".into(), Some(pool_of(devices, size))),
        "mount+write" => {
            let content = post_content();
            let file = FirstFile { content: &content, write_time_seconds: 1_000 };
            let (output, mut session) = MountedSession::of_the_mount(mounted);
            let result = match &session.current {
                PoolVersion::WithFile(_) => session
                    .publish_user_change(&mut devices, UserChange::Overwrite(file))
                    .map(|_| ())
                    .map_err(|e| short(&format!("{e:?}"))),
                PoolVersion::WithoutFile(current) => {
                    let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
                    publish_first_file(&mut writer, &mut session.allocator, &current.root, file, output.instance, &current.record_bytes)
                        .map(|_| ())
                        .map_err(|e| short(&format!("{e:?}")))
                }
            };
            match result {
                Ok(()) => ("ok".into(), Some(pool_of(devices, size))),
                Err(e) => (format!("write_refused:{e}"), Some(pool_of(devices, size))),
            }
        }
        "mount+unmount" => {
            let (_, mut session) = MountedSession::of_the_mount(mounted);
            match unmount(parameters, &mut devices, &mut session.allocator, &mut session.current, session.shadow_ledger) {
                Ok(_) => ("ok".into(), Some(pool_of(devices, size))),
                Err(e) => (format!("unmount_refused:{}", short(&format!("{e:?}"))), Some(pool_of(devices, size))),
            }
        }
        "mount+mount" => {
            drop(mounted);
            match mount_writable_with_space_admission(parameters, &mut devices, SpaceAdmission::JudgedByTheFormula) {
                Ok(_) => ("ok".into(), Some(pool_of(devices, size))),
                Err(e) => (format!("second_mount_refused:{}", short(&format!("{e:?}"))), Some(pool_of(devices, size))),
            }
        }
        other => panic!("未知动作 {other}"),
    }
}

fn short(s: &str) -> String {
    s.chars().take(160).collect()
}

fn key(r: &RootRecord) -> (u64, u32) {
    (r.checkpoint_txg.0, r.instance.0)
}

fn verdict(same: bool) -> &'static str {
    if same { "same" } else { "diff" }
}

/// R1 与 R2 逐字段比：交回（字段，same / diff / na）。
fn compare(r1: &Obs, r2: &Obs, action: &str) -> Vec<(&'static str, &'static str)> {
    let mut out = Vec::new();
    let expected_content = if action == "mount+write" { Some(post_content()) } else { r1.content.clone() };
    out.push(("posix.content", verdict(r2.content == expected_content && r2.outcome == if action == "mount+write" { "file".to_string() } else { r1.outcome.clone() })));
    out.push(("recover.outcome_kind", verdict(r1.outcome == r2.outcome)));
    if action == "mount+write" {
        out.push((
            "posix.content_of_the_version_before_the_post_mount_publish",
            verdict(r2.content_one_version_before == Some(r1.content.clone()) && r1.outcome != "failed"),
        ));
    }
    match (&r1.effective, &r2.effective) {
        (Some(a), Some(b)) => {
            out.push(("eff.instance", verdict(a.instance == b.instance)));
            out.push(("eff.checkpoint_txg", verdict(a.checkpoint_txg == b.checkpoint_txg)));
            out.push(("eff.unmount_marker", verdict(a.unmount_marker == b.unmount_marker)));
            out.push(("eff.tree_table", verdict(a.tree_table == b.tree_table)));
            out.push(("eff.tree_identifier_watermark", verdict(a.tree_identifier_watermark == b.tree_identifier_watermark)));
            out.push(("eff.rollback_floor", verdict(a.rollback_floor == b.rollback_floor)));
            out.push(("eff.instance_table", verdict(a.instance_table == b.instance_table)));
            out.push(("eff.mapping_root", verdict(a.mapping_root == b.mapping_root)));
            out.push(("eff.allocation_record_tree_root", verdict(a.allocation_record_tree_root == b.allocation_record_tree_root)));
            let expected_row = (a.instance.0, a.checkpoint_txg.0, r1.max_applied_transaction);
            let rows_relation = match (&r1.rows, &r2.rows) {
                (Some(x), Some(y)) => {
                    y.len() >= x.len() && y[..x.len()] == x[..] && y[x.len()..].contains(&expected_row)
                }
                _ => false,
            };
            out.push(("rows.r2_is_r1_plus_row_for_r1_effective", verdict(rows_relation)));
            out.push(("r1_effective_is_a_candidate_in_r2", verdict(r2.candidates.contains(&key(a)))));
            out.push(("r1_effective_in_r2_ring", verdict(r2.ring.contains(&key(a)))));
        }
        _ => out.push(("eff.*", "na")),
    }
    out.push(("chosen.key", verdict(r1.chosen.map(|r| key(&r)) == r2.chosen.map(|r| key(&r)))));
    out.push(("journal.prefix_applied", verdict(r1.prefix_applied == r2.prefix_applied)));
    out.push(("sysconf.quantities", verdict(r1.quantities == r2.quantities)));
    out.push(("rows", verdict(r1.rows == r2.rows)));
    out.push(("allocation_records", verdict(r1.allocations == r2.allocations)));
    out.push(("effective_floor", verdict(r1.floor == r2.floor)));
    out.push(("ring.r1_subset_of_r2", verdict(r1.ring.iter().all(|k| r2.ring.contains(k)))));
    out.push(("candidates", verdict(r1.candidates == r2.candidates)));
    out.push(("candidates.r1_subset_of_r2", verdict(r1.candidates.iter().all(|k| r2.candidates.contains(k)))));
    out
}

#[derive(Default)]
struct Tally {
    states: u64,
    classes: BTreeMap<String, u64>,
    action_outcomes: BTreeMap<(String, String), u64>,
    fields: BTreeMap<(String, String, String), (u64, Vec<String>)>,
}

impl Tally {
    fn merge(&mut self, other: Tally) {
        self.states += other.states;
        for (k, v) in other.classes { *self.classes.entry(k).or_default() += v; }
        for (k, v) in other.action_outcomes { *self.action_outcomes.entry(k).or_default() += v; }
        for (k, (c, ex)) in other.fields {
            let e = self.fields.entry(k).or_default();
            e.0 += c;
            for x in ex { if e.1.len() < 3 { e.1.push(x); } }
        }
    }
}

fn describe(o: &Obs) -> String {
    format!(
        "outcome={} len={:?} chosen={:?} eff={:?} marker={:?} F_root={:?} prefix={} W={} rows={:?} floor={:?} ring={:?} cand={:?} q={:?}",
        o.outcome,
        o.content.as_ref().map(Vec::len),
        o.chosen.map(|r| key(&r)),
        o.effective.map(|r| key(&r)),
        o.effective.map(|r| r.unmount_marker),
        o.effective.map(|r| r.rollback_floor.0),
        o.prefix_applied,
        o.max_applied_transaction,
        o.rows,
        o.floor,
        o.ring,
        o.candidates,
        o.quantities
    )
}

fn judge_one(ordinal: u64, image: &CrashImage<'_>, p: &Prepared, tally: &mut Tally) {
    let original_writes = p.writes.len();
    let parameters = &p.parameters;
    tally.states += 1;
    let crash = singlefs_checker_tier::crash_injection::materialized_crash_image(image);
    let torn = image.persisted.iter().enumerate().any(|(i, p)| *p && i >= original_writes);
    let r1 = observe(&crash);
    // 语义级：这一步（枚举的那几步）之前与之后两种 POSIX 内容；R1 读回的是哪一种。
    let semantic = match (r1.content == p.before.content && r1.outcome == p.before.outcome, r1.content == p.after.content && r1.outcome == p.after.outcome) {
        (true, true) => "before=after",
        (true, false) => "before",
        (false, true) => "after",
        (false, false) => "neither",
    };
    // 盘级下界（崩溃注入判据 ③ / 层 0 RecoveredToAnOlderRoot）：盘上已持久的最新根槽。它是不是这几步最后写出的那条根，
    // 是的话「读回之前那一版的内容」被语义级允许集合放过、被盘级下界拒。
    let newest = newest_persisted_root(image.writes, &image.persisted);
    let final_root = p.after.chosen.map(|r| (r.checkpoint_txg, r.instance));
    let lower_bound = if newest.is_some() && newest == final_root && p.before.content != p.after.content {
        "final_root_slot_persisted_and_contents_differ"
    } else {
        "other"
    };
    *tally.classes.entry(format!("SEM r1={semantic} lower_bound={lower_bound}")).or_default() += 1;
    let class = format!(
        "torn={torn} r1_outcome={} r1_applied_records={}",
        r1.outcome.split(':').next().unwrap_or(""),
        r1.prefix_applied > 0
    );
    *tally.classes.entry(class.clone()).or_default() += 1;
    for action in ACTIONS {
        let (outcome, after) = act(&crash, parameters, action);
        let outcome_class = outcome.chars().take(90).collect::<String>();
        *tally.action_outcomes.entry((action.to_string(), format!("{class} -> {outcome_class}"))).or_default() += 1;
        let Some(after) = after else { continue };
        if outcome != "ok" { continue; }
        let r2 = observe(&after);
        for (field, v) in compare(&r1, &r2, action) {
            let entry = tally.fields.entry((action.to_string(), field.to_string(), v.to_string())).or_default();
            entry.0 += 1;
            if v == "diff" && entry.1.len() < 3 {
                entry.1.push(format!("ordinal={ordinal} {class}\n      R1: {}\n      R2: {}", describe(&r1), describe(&r2)));
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("sizes");
    let id = args.get(2).map(String::as_str).unwrap_or("h1");
    let threads: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
    let limit: Option<u64> = args.get(4).and_then(|s| s.parse().ok());
    let started = std::time::Instant::now();
    let p = prepare(id);
    let total = layer0_state_count_with_torn_in_place_overwrites(&p.base, &p.writes, &p.segments, &full_expansion);
    println!(
        "history={id} writes={} segments={} closed_form_two_state={} with_torn={} prepare_secs={:.2}",
        p.writes.len(),
        p.segments.len(),
        closed_form_state_count(&p.segments),
        total,
        started.elapsed().as_secs_f64()
    );
    if mode == "sizes" {
        for (i, s) in p.segments.iter().enumerate() {
            let kinds: Vec<&str> = s.iter().map(|w| p.writes[*w].kind.name()).collect();
            println!("  segment {i}: {} writes {:?}", s.len(), kinds);
        }
        return;
    }
    let end = limit.map_or(total, |l| l.min(total));
    let merged = Mutex::new(Tally::default());
    let chunk = end.div_ceil(threads as u64).max(1);
    std::thread::scope(|scope| {
        for t in 0..threads as u64 {
            let from = t * chunk;
            let to = ((t + 1) * chunk).min(end);
            if from >= to { continue; }
            let p = &p;
            let merged = &merged;
            scope.spawn(move || {
                let mut tally = Tally::default();
                judge_layer0_state_range(
                    &p.base, &p.writes, &p.segments, p.judged_root_index, &p.versions, &full_expansion, from..to,
                    &mut |ordinal, image, _judgement| judge_one(ordinal, image, p, &mut tally),
                );
                merged.lock().expect("锁").merge(tally);
            });
        }
    });
    let tally = merged.into_inner().expect("锁");
    println!("states_judged={} of {} elapsed_secs={:.1}", tally.states, total, started.elapsed().as_secs_f64());
    for (k, v) in &tally.classes { println!("CLASS {k} count={v}"); }
    for ((a, o), v) in &tally.action_outcomes { println!("ACTION {a} | {o} | count={v}"); }
    for ((a, f, v), (c, ex)) in &tally.fields {
        println!("FIELD action={a} field={f} verdict={v} count={c}");
        for x in ex { println!("    e.g. {x}"); }
    }
}

use super::*;

// ============================================================================================
// 产物行
// ============================================================================================

/// 产物行写到哪：第一段与合并直接打到标准输出；第二、三段的一片先收在内存里，整片做完再原子地写成片文件（登记 E32、5.5「续跑」）。
enum EmitterSink {
    StandardOutput,
    CollectedLines(Vec<String>),
}

pub(super) struct Emitter {
    pub(super) emitted: u64,
    sink: EmitterSink,
}

impl Emitter {
    pub(super) fn to_standard_output() -> Self {
        Emitter {
            emitted: 0,
            sink: EmitterSink::StandardOutput,
        }
    }

    pub(super) fn collecting_lines() -> Self {
        Emitter {
            emitted: 0,
            sink: EmitterSink::CollectedLines(Vec::new()),
        }
    }

    fn write_line(&mut self, line: String) {
        match &mut self.sink {
            EmitterSink::StandardOutput => println!("{line}"),
            EmitterSink::CollectedLines(lines) => lines.push(line),
        }
    }

    pub(super) fn emit(&mut self, body: &str) {
        self.emitted += 1;
        self.write_line(format!("E7RESULT {body}"));
    }

    pub(super) fn finish(&mut self) {
        self.emitted += 1;
        let line = format!("E7RESULT name=done emitted={}", self.emitted);
        self.write_line(line);
    }

    /// 收在内存里的行（打到标准输出的那一种交回空表）。
    pub(super) fn into_collected_lines(self) -> Vec<String> {
        match self.sink {
            EmitterSink::StandardOutput => Vec::new(),
            EmitterSink::CollectedLines(lines) => lines,
        }
    }
}

fn mebibytes_text(bytes: u64) -> String {
    assert_eq!(bytes % MEBIBYTE, 0, "盘字节数落在 MiB 整数倍上");
    (bytes / MEBIBYTE).to_string()
}

pub(super) fn optional_mebibytes_text(bytes: Option<u64>) -> String {
    bytes.map_or_else(|| "not_found".to_string(), mebibytes_text)
}

pub(super) fn list_of_mebibytes_text(points: &[u64]) -> String {
    if points.is_empty() {
        return "none".to_string();
    }
    points
        .iter()
        .map(|bytes| mebibytes_text(*bytes))
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn optional_number_text(value: Option<u64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| number.to_string())
}

pub(super) fn mount_text(outcome: &StageOutcome<MountReport>) -> String {
    outcome.text(|report| report.class.name().to_string())
}

fn available_text(outcome: &StageOutcome<MountReport>) -> String {
    match outcome {
        StageOutcome::Succeeded(report) => report.available_minimum_bytes.to_string(),
        StageOutcome::Failed(_) | StageOutcome::NotReached => "none".to_string(),
    }
}

fn tally_text(reads_and_writes: &ReadsAndWrites) -> String {
    format!(
        "r{}/{},w{}/{}",
        reads_and_writes.reads.calls,
        reads_and_writes.reads.bytes,
        reads_and_writes.writes.calls,
        reads_and_writes.writes.bytes
    )
}

pub(super) fn emit_trial(emitter: &mut Emitter, item: &str, trial: &Trial) {
    emitter.emit(&format!(
        "name=h4_trial item={item} device_mebibytes={} make_filesystem={} empty_mount={} first_file={} unmount={} file_mount={} empty_available_minimum_bytes={} file_available_minimum_bytes={}",
        mebibytes_text(trial.device_bytes),
        trial.make_filesystem.text(|()| "ok".to_string()),
        mount_text(&trial.empty_mount),
        trial.first_file.text(|()| "ok".to_string()),
        trial.unmount.text(|()| "ok".to_string()),
        mount_text(&trial.file_mount),
        available_text(&trial.empty_mount),
        available_text(&trial.file_mount),
    ));
}

/// 一份读数所在的位置：哪个项、哪种状态、哪块盘（字节数）、哪个几何。
struct ReadingLabel<'label> {
    pub(super) item: &'label str,
    pub(super) state: StateKind,
    pub(super) device_bytes: u64,
    pub(super) journal_ring_bytes: u64,
    pub(super) slots_per_region: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StateKind {
    Empty,
    File,
}

impl StateKind {
    pub(super) fn name(self) -> &'static str {
        match self {
            StateKind::Empty => "empty",
            StateKind::File => "file",
        }
    }
}

fn parts_per_million(part: u64, total: u64) -> u64 {
    if total == 0 {
        return 0;
    }
    u64::try_from(u128::from(part) * 1_000_000 / u128::from(total)).expect("百万分比装得进 u64")
}

/// 一份读数的各行：读数头、逐盘各项（含准入预留、比例、六项份额、主导）、对 K30–K32 与 G7 的核。返回盘 0 的主导判定。
fn emit_reading(
    emitter: &mut Emitter,
    label: &ReadingLabel<'_>,
    report: &MountReport,
    snapshot: &ReadingSnapshot,
) -> (Dominance, [(&'static str, u64); 6]) {
    let item = label.item;
    let state = label.state.name();
    let entry = snapshot.entry.name();
    emitter.emit(&format!(
        "name=h4_reading item={item} state={state} entry={entry} device_mebibytes={} class={} instance_rows_of_the_switch_reserve={} replicas={} checkpoint_reserve_pool_bytes={} pending_delete_bytes={} committed_reservation_bytes={} retirement_tree_internal_levels={}",
        mebibytes_text(label.device_bytes),
        report.class.name(),
        snapshot.instance_rows_of_the_switch_reserve,
        snapshot.replicas,
        snapshot.checkpoint_reserve_pool_bytes,
        snapshot.pending_delete_bytes,
        snapshot.committed_reservation_bytes,
        optional_number_text(snapshot.retirement_tree_internal_levels),
    ));
    let mut dominance_of_the_first_device = None;
    for terms in &snapshot.devices {
        let items = terms.admission_reserve_items();
        let reserve_bytes = terms.admission_reserve_bytes();
        let dominance = dominance_of(&items);
        let shares = items
            .iter()
            .map(|(name, bytes)| {
                format!(
                    "share_permille_{name}={}",
                    share_permille(*bytes, reserve_bytes)
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        emitter.emit(&format!(
            "name=h4_reading_device item={item} state={state} entry={entry} device={} capacity_bytes={} allocated_bytes={} unreclaimable_bytes={} mount_time_commitment_bytes={} abandoned_root_exclusive_bytes={} pending_delete_share_bytes={} committed_reservation_share_bytes={} checkpoint_reserve_pool_share_bytes={} ring_window_bytes={} admission_reserve_bytes={} available_bytes={} reserve_over_capacity_parts_per_million={} {shares} dominance={}",
            terms.device,
            terms.capacity_bytes,
            terms.allocated_bytes,
            terms.unreclaimable_bytes,
            terms.mount_time_commitment_bytes,
            terms.abandoned_root_exclusive_bytes,
            terms.pending_delete_share_bytes,
            terms.committed_reservation_share_bytes,
            terms.checkpoint_reserve_pool_share_bytes,
            terms.ring_window_bytes,
            reserve_bytes,
            terms.available_bytes,
            parts_per_million(reserve_bytes, terms.capacity_bytes),
            dominance.verdict_text(),
        ));
        if dominance_of_the_first_device.is_none() {
            dominance_of_the_first_device = Some((dominance, items));
        }
    }
    let checks = checks_of_a_reading(
        snapshot,
        label.device_bytes,
        label.journal_ring_bytes,
        label.slots_per_region,
    );
    let ring_window_if_deducted_bytes = ring_window_by_the_clauses(
        label.device_bytes,
        label.journal_ring_bytes,
        label.slots_per_region,
        snapshot.retirement_tree_internal_levels.unwrap_or(0),
    )
    .ring_window_bytes();
    emitter.emit(&format!(
        "name=h4_reading_checks item={item} state={state} entry={entry} pending_delete_is_zero={} checkpoint_cost_blocks_by_the_pool={} checkpoint_cost_blocks_by_the_commitment={} mount_time_commitment_matches_the_clauses={} ring_window_matches_the_clauses={} ring_window_by_the_clauses_if_deducted_bytes={ring_window_if_deducted_bytes} capacity_by_the_local_geometry={} all_hold={}",
        checks.pending_delete_is_zero,
        optional_number_text(checks.checkpoint_cost_blocks_by_the_pool),
        checks
            .checkpoint_cost_blocks_by_the_commitment
            .iter()
            .map(|blocks| optional_number_text(*blocks))
            .collect::<Vec<_>>()
            .join(","),
        checks.mount_time_commitment_by_the_clauses,
        checks.ring_window_by_the_clauses,
        checks.capacity_by_the_local_geometry,
        checks.all_hold(),
    ));
    dominance_of_the_first_device.expect("两块盘")
}

fn emit_mount_tally(emitter: &mut Emitter, item: &str, state: &str, report: &MountReport) {
    for (device, tally) in report.mount_tallies.iter().enumerate() {
        emitter.emit(&format!(
            "name=h4_mount_tally item={item} state={state} device={device} system_configuration={} root_ring={} journal_ring={} unit_area={}",
            tally_text(&tally.system_configuration),
            tally_text(&tally.root_ring),
            tally_text(&tally.journal_ring),
            tally_text(&tally.unit_area),
        ));
    }
    emitter.emit(&format!(
        "name=h4_instance_rows item={item} state={state} rows_by_the_allocator={} rows_read_back_from_the_disk={}",
        report.instance_rows_by_the_allocator,
        optional_number_text(report.instance_rows_read_back_from_the_disk),
    ));
}

/// 一个入口的读数：入口、主导判定、准入预留六项（项名，字节）。
type DominanceOfAnEntry = (ReadingEntry, Dominance, [(&'static str, u64); 6]);

/// 一种状态（空池 / 有文件）上的全部读数行，返回每个入口的主导判定与各份读数的核是否全过。
struct StateOutcome {
    dominance_by_entry: Vec<DominanceOfAnEntry>,
    checks_hold: bool,
}

fn emit_state(
    emitter: &mut Emitter,
    label: &ReadingLabel<'_>,
    report: &MountReport,
) -> StateOutcome {
    let mut dominance_by_entry = Vec::new();
    let mut checks_hold = true;
    for snapshot in &report.readings {
        let (dominance, reserve_items) = emit_reading(emitter, label, report, snapshot);
        let checks = checks_of_a_reading(
            snapshot,
            label.device_bytes,
            label.journal_ring_bytes,
            label.slots_per_region,
        );
        checks_hold &= checks.all_hold();
        dominance_by_entry.push((snapshot.entry, dominance, reserve_items));
    }
    emit_mount_tally(emitter, label.item, label.state.name(), report);
    StateOutcome {
        dominance_by_entry,
        checks_hold,
    }
}

/// 一个项跑完留给汇总的东西：各状态各入口的主导判定、设备有效性的红数（T2、T3、PC-堆，决定退出码）。
pub(super) struct ItemOutcome {
    pub(super) dominance_points: Vec<DominancePoint>,
    pub(super) invalid_device_conditions: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DominancePoint {
    pub(super) item: String,
    pub(super) state: &'static str,
    pub(super) entry: ReadingEntry,
    pub(super) dominance: Dominance,
    pub(super) reserve_items: [(&'static str, u64); 6],
}

/// 要读数的一种状态与它所在的那次尝试（空池读第一次挂载，有文件读第二次挂载）；尝试没有（最小盘没找到）就是 `None`。
pub(super) struct StateToRead<'trial> {
    pub(super) state: StateKind,
    pub(super) trial: Option<&'trial Trial>,
}

pub(super) fn emit_states_of_a_trial(
    emitter: &mut Emitter,
    item: &str,
    journal_ring_bytes: u64,
    slots_per_region: u64,
    states: [StateToRead<'_>; 2],
) -> (Vec<DominancePoint>, bool) {
    let mut points = Vec::new();
    let mut checks_hold = true;
    for StateToRead { state, trial } in states {
        let state_name = state.name();
        let Some(trial) = trial else {
            emitter.emit(&format!(
                "name=h4_reading item={item} state={state_name} reading=not_run reason=minimum_disk_not_found"
            ));
            continue;
        };
        let outcome = match state {
            StateKind::Empty => &trial.empty_mount,
            StateKind::File => &trial.file_mount,
        };
        let StageOutcome::Succeeded(report) = outcome else {
            emitter.emit(&format!(
                "name=h4_reading item={item} state={state_name} reading=not_run reason=mount_did_not_succeed"
            ));
            continue;
        };
        let label = ReadingLabel {
            item,
            state,
            device_bytes: trial.device_bytes,
            journal_ring_bytes,
            slots_per_region,
        };
        let state_outcome = emit_state(emitter, &label, report);
        checks_hold &= state_outcome.checks_hold;
        for (entry, dominance, reserve_items) in state_outcome.dominance_by_entry {
            points.push(DominancePoint {
                item: item.to_string(),
                state: state_name,
                entry,
                dominance,
                reserve_items,
            });
        }
    }
    (points, checks_hold)
}

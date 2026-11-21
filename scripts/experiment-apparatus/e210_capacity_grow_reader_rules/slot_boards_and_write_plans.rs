//! 槽与棋盘、判定与臂和读者、写计划与每一步的结果、崩溃点与起始棋盘。

use e7_index_bench::{u32_from_usize, usize_from_u64};
use super::*;

/// 一个系统配置槽的内容：读不出（撕裂或全零，不自证），或自证过（槽世代号、表、意图标记）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Slot {
    Unreadable,
    SelfVerified {
        generation: u64,
        table: Table,
        carries_intent_marker: bool,
    },
}

/// 盘面：两块盘各两个槽。
pub(super) type Board = [[Slot; SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE]; DISK_COUNT];

/// 读者对一个（盘面，探测值）的决定：可写挂载做不做成，只读挂载做不做成以及点名哪几块盘。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct Decision {
    pub(super) writable_mount_made: bool,
    pub(super) read_only_mount_made: bool,
    pub(super) named_by_read_only: DiskSet,
}

/// 整池拒：可写与只读都拒，不点名。
pub(super) const WHOLE_POOL_REFUSED: Decision = Decision {
    writable_mount_made: false,
    read_only_mount_made: false,
    named_by_read_only: NOBODY_NAMED,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Arm {
    ChainThenLowest,
    HighestTable,
    OldTableWhileIntentMarkerIsOnTheDisk,
    LowestToReadHighestToProbeWithAbandon,
    OnlyIfEveryTableIsEqual,
}

/// 臂表：对照读者与自检都遍历它，加臂时一起改。
pub(super) const ARMS: [Arm; 5] = [
    Arm::ChainThenLowest,
    Arm::HighestTable,
    Arm::OldTableWhileIntentMarkerIsOnTheDisk,
    Arm::LowestToReadHighestToProbeWithAbandon,
    Arm::OnlyIfEveryTableIsEqual,
];

pub(super) fn arm_name(arm: Arm) -> &'static str {
    match arm {
        Arm::ChainThenLowest => "chain_then_lowest",
        Arm::HighestTable => "highest_table",
        Arm::OldTableWhileIntentMarkerIsOnTheDisk => "old_table_while_intent_marker",
        Arm::LowestToReadHighestToProbeWithAbandon => "lowest_to_read_highest_to_probe",
        Arm::OnlyIfEveryTableIsEqual => "baseline_only_if_every_table_is_equal",
    }
}

/// 三条对照读者：放行一切臂（必漏）、一律拒臂（必误拒）、按类判官臂（阴性对照，答案由构造给出）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ControlReader {
    AllowEverything,
    RejectEveryWritableMount,
    NameWhatTheClassOverruns,
}

pub(super) const CONTROL_READERS: [ControlReader; 3] = [
    ControlReader::AllowEverything,
    ControlReader::RejectEveryWritableMount,
    ControlReader::NameWhatTheClassOverruns,
];

pub(super) fn control_reader_name(control: ControlReader) -> &'static str {
    match control {
        ControlReader::AllowEverything => "allow_everything",
        ControlReader::RejectEveryWritableMount => "reject_every_writable_mount",
        ControlReader::NameWhatTheClassOverruns => "name_what_the_class_overruns",
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Reader {
    Arm(Arm),
    Control(ControlReader),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WriteContent {
    /// 变大要写的新表（探测值那张）。
    NewTable,
    /// 变大开始时的生效表加意图标记（丙的标记轮）。
    TableInForceBeforeGrowthWithIntentMarker,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum PlanStep {
    Write { disk: usize, content: WriteContent },
    PoolBarrier,
}

/// 一次变大的可写挂载的系统配置写序：开头那道取号前的池屏障（前面没有写）、可选的标记轮、`new_table_rounds` 轮新表；
/// 每轮逐盘各一次写、轮后一道池屏障（`transaction.rs:1334` 起的 `write_the_witnessed_acquisition`：外层按轮、内层逐盘、每轮后一次屏障）。
pub(super) fn acquisition_write_plan(
    with_intent_marker_round: bool,
    new_table_rounds: usize,
) -> Vec<PlanStep> {
    let mut steps = vec![PlanStep::PoolBarrier];
    let mut round_contents = Vec::new();
    if with_intent_marker_round {
        round_contents.push(WriteContent::TableInForceBeforeGrowthWithIntentMarker);
    }
    for _round in 0..new_table_rounds {
        round_contents.push(WriteContent::NewTable);
    }
    for content in &round_contents {
        for disk in 0..DISK_COUNT {
            steps.push(PlanStep::Write {
                disk,
                content: *content,
            });
        }
        steps.push(PlanStep::PoolBarrier);
    }
    steps
}

/// 每条臂一次变大的可写挂载用的写序：丙多一轮带意图标记的标记轮，其余就是取号写两轮。
pub(super) fn write_plan_of(arm: Arm) -> Vec<PlanStep> {
    match arm {
        Arm::OldTableWhileIntentMarkerIsOnTheDisk => {
            acquisition_write_plan(true, ACQUISITION_WRITE_ROUNDS)
        }
        Arm::ChainThenLowest
        | Arm::HighestTable
        | Arm::LowestToReadHighestToProbeWithAbandon
        | Arm::OnlyIfEveryTableIsEqual => acquisition_write_plan(false, ACQUISITION_WRITE_ROUNDS),
    }
}

/// 数写序里的槽写个数与「前面有写」的池屏障个数：开头那道前面没有写的屏障、连着的屏障都不算
/// （`transaction.rs:574`–`:648` 的 `perform_commit_step`：屏障只在 `has_writes_since_barrier` 为真时才发）。
pub(super) fn count_writes_and_pool_barriers(steps: &[PlanStep]) -> (usize, usize) {
    let mut writes = 0;
    let mut barriers = 0;
    let mut writes_since_the_last_barrier = 0;
    for step in steps {
        match step {
            PlanStep::Write { .. } => {
                writes += 1;
                writes_since_the_last_barrier += 1;
            }
            PlanStep::PoolBarrier => {
                if writes_since_the_last_barrier > 0 {
                    barriers += 1;
                }
                writes_since_the_last_barrier = 0;
            }
        }
    }
    (writes, barriers)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct PlannedWrite {
    disk: usize,
    content: WriteContent,
}

/// 按池屏障切持久段（D13（验证路线） 已定项 4）：当前段里有写才被屏障关掉，前面没有写的屏障不记。
pub(super) fn persistence_segments(steps: &[PlanStep]) -> Vec<Vec<PlannedWrite>> {
    let mut segments = Vec::new();
    let mut open_segment: Vec<PlannedWrite> = Vec::new();
    for step in steps {
        match step {
            PlanStep::Write { disk, content } => open_segment.push(PlannedWrite {
                disk: *disk,
                content: *content,
            }),
            PlanStep::PoolBarrier => {
                if !open_segment.is_empty() {
                    segments.push(std::mem::take(&mut open_segment));
                }
            }
        }
    }
    if !open_segment.is_empty() {
        segments.push(open_segment);
    }
    segments
}

/// 这块盘下一次写系统配置槽的（世代号，槽下标）：世代号 = 本盘自证槽最大世代号 + 1（两槽都读不出从 1 起），槽 = 世代号 mod 槽数
/// （`transaction.rs:666`–`:670` 的文档、`:686`–`:694`、`:739`–`:743`；算世代号那一读是写那一刻读盘，已发的写都看得见）。
pub(super) fn next_write_target(board: &Board, disk: usize) -> (u64, usize) {
    let newest_generation = board[disk]
        .iter()
        .filter_map(|slot| match slot {
            Slot::SelfVerified { generation, .. } => Some(*generation),
            Slot::Unreadable => None,
        })
        .max()
        .unwrap_or(0);
    let generation = newest_generation + 1;
    (
        generation,
        usize_from_u64(generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE as u64),
    )
}

fn slot_content_of_write(
    content: WriteContent,
    table_before_growth: Table,
    table_after_growth: Table,
    generation: u64,
) -> Slot {
    match content {
        WriteContent::NewTable => Slot::SelfVerified {
            generation,
            table: table_after_growth,
            carries_intent_marker: false,
        },
        WriteContent::TableInForceBeforeGrowthWithIntentMarker => Slot::SelfVerified {
            generation,
            table: table_before_growth,
            carries_intent_marker: true,
        },
    }
}
/// 一次系统配置槽写在崩溃状态里取的态。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WriteOutcome {
    DidNotPersist,
    Persisted,
    TornSoNeitherOldNorNewIsReadable,
}

/// 一次写可取的态，次序固定（没持久、持久、新旧都读不出）：系统配置槽写是原地覆写、槽 4096 字节长于一个扇区，所以多取第三态。
pub(super) fn outcomes_of_one_slot_write() -> Vec<WriteOutcome> {
    let mut outcomes = vec![WriteOutcome::DidNotPersist, WriteOutcome::Persisted];
    if SYSTEM_CONFIGURATION_SLOT_BYTES > CRASH_REPLAY_SECTOR_BYTES {
        outcomes.push(WriteOutcome::TornSoNeitherOldNorNewIsReadable);
    }
    outcomes
}

fn apply_outcome(
    board: &mut Board,
    disk: usize,
    slot_index: usize,
    persisted_content: Slot,
    outcome: WriteOutcome,
) {
    match outcome {
        WriteOutcome::DidNotPersist => {}
        WriteOutcome::Persisted => board[disk][slot_index] = persisted_content,
        WriteOutcome::TornSoNeitherOldNorNewIsReadable => {
            board[disk][slot_index] = Slot::Unreadable
        }
    }
}

/// 把序号按进制拆成各次写的态，第一次写是最高位（字典序）。
fn decode_outcomes(
    state_number: usize,
    write_count: usize,
    possible_outcomes: &[WriteOutcome],
) -> Vec<WriteOutcome> {
    let base = possible_outcomes.len();
    let mut remaining = state_number;
    let mut outcomes = vec![possible_outcomes[0]; write_count];
    for position in (0..write_count).rev() {
        outcomes[position] = possible_outcomes[remaining % base];
        remaining /= base;
    }
    outcomes
}

/// 一个断电点：盘面，是不是「终点已用」（全部写与屏障都完成、之后的发布已把单元写进新增的盘区），
/// 是不是「最后一个持久段全部持久」那一点。
pub(super) struct CrashPoint {
    pub(super) board: Board,
    pub(super) is_end_of_use: bool,
    pub(super) is_last_segment_fully_persisted: bool,
}

/// 写序的全部断电点：前面各持久段全持久、当前持久段里每次写各取一态、之后各段全没持久；次序 = 持久段先后、段内按态的字典序；
/// 最后加一个「终点已用」。盘面相同的状态不去重。
pub(super) fn crash_points(
    start: &Board,
    steps: &[PlanStep],
    table_before_growth: Table,
    table_after_growth: Table,
) -> Vec<CrashPoint> {
    let segments = persistence_segments(steps);
    let possible_outcomes = outcomes_of_one_slot_write();
    let mut points = Vec::new();
    let mut board_before_segment = *start;
    for (segment_index, segment) in segments.iter().enumerate() {
        let mut board_with_whole_segment_persisted = board_before_segment;
        let mut targets = Vec::new();
        for planned in segment {
            let (generation, slot_index) =
                next_write_target(&board_with_whole_segment_persisted, planned.disk);
            let content = slot_content_of_write(
                planned.content,
                table_before_growth,
                table_after_growth,
                generation,
            );
            board_with_whole_segment_persisted[planned.disk][slot_index] = content;
            targets.push((planned.disk, slot_index, content));
        }
        let state_count = possible_outcomes.len().pow(u32_from_usize(segment.len()));
        for state_number in 0..state_count {
            let outcomes = decode_outcomes(state_number, segment.len(), &possible_outcomes);
            let mut board = board_before_segment;
            for (target, outcome) in targets.iter().zip(outcomes.iter()) {
                apply_outcome(&mut board, target.0, target.1, target.2, *outcome);
            }
            let is_last_segment_fully_persisted = segment_index + 1 == segments.len()
                && outcomes
                    .iter()
                    .all(|outcome| *outcome == WriteOutcome::Persisted);
            points.push(CrashPoint {
                board,
                is_end_of_use: false,
                is_last_segment_fully_persisted,
            });
        }
        board_before_segment = board_with_whole_segment_persisted;
    }
    points.push(CrashPoint {
        board: board_before_segment,
        is_end_of_use: true,
        is_last_segment_fully_persisted: false,
    });
    points
}

/// 变大前的盘面：每盘两槽都自证、都装 `table`；`newest_slot_of_each_disk` 给每盘世代号最大的槽下标（槽 = 世代号 mod 2，
/// 最新槽在槽 1 时两槽世代号是 2、3，在槽 0 时是 4、3）。
pub(super) fn start_board(newest_slot_of_each_disk: [usize; DISK_COUNT], table: Table) -> Board {
    let mut board = [[Slot::Unreadable; SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE]; DISK_COUNT];
    for disk in 0..DISK_COUNT {
        let newest_slot = newest_slot_of_each_disk[disk];
        let older_slot = 1 - newest_slot;
        let newest_generation: u64 = if newest_slot == 1 { 3 } else { 4 };
        board[disk][newest_slot] = Slot::SelfVerified {
            generation: newest_generation,
            table,
            carries_intent_marker: false,
        };
        board[disk][older_slot] = Slot::SelfVerified {
            generation: newest_generation - 1,
            table,
            carries_intent_marker: false,
        };
    }
    board
}

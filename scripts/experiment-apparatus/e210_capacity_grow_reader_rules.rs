//! E210（盘变大改容量表的读者规则四臂）：独立手写的计数模型，不与 `crates/` 共用代码（不是入库装置）。
//!
//! 跑前登记 `research/prompts/e210-preregistration.md`；岔路单 `research/prompts/m4-capacity-grow-r1-forks.md` 三行。
//! 命令行参数选一段产物：
//! - `write-counts-and-root-level`：岔路单行 2（每条读者规则一次变大的可写挂载写几次系统配置槽、几道屏障）与行 3（变大跨过分配记录树
//!   根层级边界时，一点不记与记到边界为止两种做法下用得上的整槽数）。
//! - `reader-rules-on-histories`：岔路单行 1（四条读者规则加基线在同一组历史上的漏放行数与误拒数，各带分母）。
//!
//! 臂的名字（登记里的甲、乙、丙、丁与基线臂）：
//! - 甲 `ChainThenLowest`：全部有效槽的表成链就取逐项最小，不成链整池拒。
//! - 乙 `HighestTable`：生效表取各有效槽逐项最大。
//! - 丙 `OldTableWhileIntentMarkerIsOnTheDisk`：先写一轮带意图标记的槽、再写两轮新表；盘面上有带标记的槽取其中逐项最小，没有取逐项最大。
//! - 丁 `LowestToReadHighestToProbeWithAbandon`：生效表逐项最小，比探测值用逐项最大，未写齐的变大可以写回旧表放弃。
//! - 基线臂 `OnlyIfEveryTableIsEqual`：表全相等才认，否则整池拒。
//!
//! 本地常量抄自 `crates/singlefs-format/src/lib.rs`（行号写在各常量旁，登记第十一节停机条款 1 逐个核）；写序、世代号与屏障口径照
//! `crates/singlefs-core/src/transaction.rs` 的取号写两轮。模型里没有随机源、时钟与并发，同一份装置跑两次标准输出逐字节相同。

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use e7_index_bench::Emitter;

include!("heavy_threads_default/threads.rs");

/// 线程数只影响挂钟、不影响任何一行输出（并行单位的结果按单元次序合并）。`SINGLEFS_THREAD_CAP` 没设取登记的重活默认并行度。
fn worker_thread_count() -> usize {
    default_heavy_threads(include_str!("../../.claude/heavy-thread-default"))
}

/// `crates/singlefs-format/src/lib.rs:14` 的 `SLOT_BYTES`。
const SLOT_BYTES: u64 = 16384;
/// 同文件 `:163` 的 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`。
const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
/// 同文件 `:169` 的 `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`。
const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// 同文件 `:279` 的 `SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE`（那边是 u64，这里当数组下标用，取 usize）。
const SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: usize = 2;
/// 同文件 `:286` 的 `SYSTEM_CONFIGURATION_SLOT_BYTES`。
const SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
/// 同文件 `:321` 的 `UNIT_AREA_START_SLOT`。
const UNIT_AREA_START_SLOT: u64 = 50176;
/// 崩溃点重放的扇区取 512 字节（D13（验证路线） 已定项 4）：长于一个扇区的原地覆写多取第三态「新旧都读不出」。
const CRASH_REPLAY_SECTOR_BYTES: u64 = 512;
/// `crates/singlefs-core/src/transaction.rs:1313` 的 `ACQUISITION_WRITE_ROUNDS` 有两个元素：取号写两轮。
const ACQUISITION_WRITE_ROUNDS: usize = 2;

/// 池里两块盘（设备身份 0、1）。
const DISK_COUNT: usize = 2;
/// 「只有一块盘缩小或变大」的用户动作里动的那块盘的设备身份。
const DEVICE_THAT_CHANGES_ALONE: usize = 0;
/// 用户动作「再变大」每盘加的整槽数。
const GROW_STEP_SLOTS: u64 = 100;

/// 逐盘容量表：按设备身份排的两项，每盘一项，单位是整槽（`SLOT_BYTES`）。
type Table = [u64; DISK_COUNT];
/// 一个盘的集合，下标是设备身份。
type DiskSet = [bool; DISK_COUNT];
const NOBODY_NAMED: DiskSet = [false; DISK_COUNT];

/// 一个系统配置槽的内容：读不出（撕裂或全零，不自证），或自证过（槽世代号、表、意图标记）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Slot {
    Unreadable,
    SelfVerified {
        generation: u64,
        table: Table,
        carries_intent_marker: bool,
    },
}

/// 盘面：两块盘各两个槽。
type Board = [[Slot; SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE]; DISK_COUNT];

/// 读者对一个（盘面，探测值）的决定：可写挂载做不做成，只读挂载做不做成以及点名哪几块盘。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Decision {
    writable_mount_made: bool,
    read_only_mount_made: bool,
    named_by_read_only: DiskSet,
}

/// 整池拒：可写与只读都拒，不点名。
const WHOLE_POOL_REFUSED: Decision = Decision {
    writable_mount_made: false,
    read_only_mount_made: false,
    named_by_read_only: NOBODY_NAMED,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    ChainThenLowest,
    HighestTable,
    OldTableWhileIntentMarkerIsOnTheDisk,
    LowestToReadHighestToProbeWithAbandon,
    OnlyIfEveryTableIsEqual,
}

/// 臂表：对照读者与自检都遍历它，加臂时一起改。
const ARMS: [Arm; 5] = [
    Arm::ChainThenLowest,
    Arm::HighestTable,
    Arm::OldTableWhileIntentMarkerIsOnTheDisk,
    Arm::LowestToReadHighestToProbeWithAbandon,
    Arm::OnlyIfEveryTableIsEqual,
];

fn arm_name(arm: Arm) -> &'static str {
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
enum ControlReader {
    AllowEverything,
    RejectEveryWritableMount,
    NameWhatTheClassOverruns,
}

const CONTROL_READERS: [ControlReader; 3] = [
    ControlReader::AllowEverything,
    ControlReader::RejectEveryWritableMount,
    ControlReader::NameWhatTheClassOverruns,
];

fn control_reader_name(control: ControlReader) -> &'static str {
    match control {
        ControlReader::AllowEverything => "allow_everything",
        ControlReader::RejectEveryWritableMount => "reject_every_writable_mount",
        ControlReader::NameWhatTheClassOverruns => "name_what_the_class_overruns",
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Reader {
    Arm(Arm),
    Control(ControlReader),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WriteContent {
    /// 变大要写的新表（探测值那张）。
    NewTable,
    /// 变大开始时的生效表加意图标记（丙的标记轮）。
    TableInForceBeforeGrowthWithIntentMarker,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PlanStep {
    Write { disk: usize, content: WriteContent },
    PoolBarrier,
}

/// 一次变大的可写挂载的系统配置写序：开头那道取号前的池屏障（前面没有写）、可选的标记轮、`new_table_rounds` 轮新表；
/// 每轮逐盘各一次写、轮后一道池屏障（`transaction.rs:1334` 起的 `write_the_witnessed_acquisition`：外层按轮、内层逐盘、每轮后一次屏障）。
fn acquisition_write_plan(
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
fn write_plan_of(arm: Arm) -> Vec<PlanStep> {
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
fn count_writes_and_pool_barriers(steps: &[PlanStep]) -> (usize, usize) {
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
struct PlannedWrite {
    disk: usize,
    content: WriteContent,
}

/// 按池屏障切持久段（D13（验证路线） 已定项 4）：当前段里有写才被屏障关掉，前面没有写的屏障不记。
fn persistence_segments(steps: &[PlanStep]) -> Vec<Vec<PlannedWrite>> {
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
/// （`transaction.rs:666`–`:670` 的文档、`:686`–`:694`、`:739`–`:743`；算世代号那一读是写那一刻读盘，所以前面已发的写都看得见）。
fn next_write_target(board: &Board, disk: usize) -> (u64, usize) {
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
        (generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE as u64) as usize,
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
enum WriteOutcome {
    DidNotPersist,
    Persisted,
    TornSoNeitherOldNorNewIsReadable,
}

/// 一次写可取的态，次序固定（没持久、持久、新旧都读不出）：系统配置槽写是原地覆写、槽 4096 字节长于一个扇区，所以多取第三态。
fn outcomes_of_one_slot_write() -> Vec<WriteOutcome> {
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
struct CrashPoint {
    board: Board,
    is_end_of_use: bool,
    is_last_segment_fully_persisted: bool,
}

/// 写序的全部断电点：前面各持久段全持久、当前持久段里每次写各取一态、之后各段全没持久；次序 = 持久段先后、段内按态的字典序；
/// 最后加一个「终点已用」。盘面相同的状态不去重。
fn crash_points(
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
        let state_count = possible_outcomes.len().pow(segment.len() as u32);
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
fn start_board(newest_slot_of_each_disk: [usize; DISK_COUNT], table: Table) -> Board {
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
/// 历史族：池里更早已经变大过并写入过（早尺寸、前尺寸、后尺寸三个尺寸），或从没变大过（早尺寸 = 前尺寸）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Family {
    GrownBefore,
    NeverGrownBefore,
}

/// 形状：三个尺寸各两项（依设备身份 0、1，整槽数）。变大前盘面装前表；本次变大要写的新表是后尺寸那张。
#[derive(Clone, Copy, Debug)]
struct Shape {
    family: Family,
    early: Table,
    front: Table,
    back: Table,
}

fn registered_shapes() -> Vec<Shape> {
    vec![
        Shape {
            family: Family::GrownBefore,
            early: [100, 100],
            front: [200, 200],
            back: [300, 300],
        },
        Shape {
            family: Family::GrownBefore,
            early: [100, 100],
            front: [200, 200],
            back: [300, 200],
        },
        Shape {
            family: Family::GrownBefore,
            early: [100, 100],
            front: [200, 100],
            back: [200, 300],
        },
        Shape {
            family: Family::NeverGrownBefore,
            early: [100, 100],
            front: [100, 100],
            back: [200, 200],
        },
        Shape {
            family: Family::NeverGrownBefore,
            early: [100, 100],
            front: [100, 100],
            back: [200, 100],
        },
    ]
}

/// 另一组保持每个设备身份上大小次序的整数（登记第八节「尺寸取另一组整数」）：同一设备身份上的几个不同尺寸按从小到大依次换成下表的数。
const ALTERNATIVE_NUMBERS_BY_RANK: [[u64; 3]; DISK_COUNT] = [[10, 20, 30], [90, 95, 99]];

fn alternative_shapes() -> Vec<Shape> {
    registered_shapes()
        .iter()
        .map(|shape| {
            let mut alternative = *shape;
            for position in 0..DISK_COUNT {
                let mut distinct_sizes = vec![
                    shape.early[position],
                    shape.front[position],
                    shape.back[position],
                ];
                distinct_sizes.sort();
                distinct_sizes.dedup();
                let rank_of = |size: u64| {
                    distinct_sizes
                        .iter()
                        .position(|candidate| *candidate == size)
                        .expect("size is one of the three")
                };
                alternative.early[position] =
                    ALTERNATIVE_NUMBERS_BY_RANK[position][rank_of(shape.early[position])];
                alternative.front[position] =
                    ALTERNATIVE_NUMBERS_BY_RANK[position][rank_of(shape.front[position])];
                alternative.back[position] =
                    ALTERNATIVE_NUMBERS_BY_RANK[position][rank_of(shape.back[position])];
            }
            alternative
        })
        .collect()
}

/// 点位：「满」= 已用落点上界取前尺寸（终点已用取后尺寸），「少」= 单元只在最早那一段（取早尺寸，只在族 D）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Footprint {
    Full,
    Few,
}

fn landing_of(footprint: Footprint, is_end_of_use: bool, shape: &Shape) -> Table {
    match (footprint, is_end_of_use) {
        (Footprint::Full, true) => shape.back,
        (Footprint::Full, false) => shape.front,
        (Footprint::Few, _) => shape.early,
    }
}

/// 断电之后用户的动作。动作之后每块盘的大小就是探测值。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UserAction {
    LeaveAlone,
    ShrinkEveryDiskToFrontSize,
    ShrinkOnlyTheChangingDeviceToFrontSize,
    ShrinkEveryDiskToEarlySize,
    ShrinkOnlyTheChangingDeviceToEarlySize,
    GrowEveryDisk,
    GrowOnlyTheChangingDevice,
    /// 把一块盘整盘换回它在早尺寸时的镜像：两槽都自证、都装早表，盘大小 = 早尺寸。
    ReplaceDiskWithEarlyImage(usize),
    /// 把一块盘整盘换回它在前尺寸写入之后的镜像（只在族 D）。
    ReplaceDiskWithFrontImage(usize),
}

fn actions_of(family: Family) -> Vec<UserAction> {
    let mut actions = vec![UserAction::LeaveAlone];
    if family == Family::GrownBefore {
        actions.push(UserAction::ShrinkEveryDiskToFrontSize);
        actions.push(UserAction::ShrinkOnlyTheChangingDeviceToFrontSize);
    }
    actions.push(UserAction::ShrinkEveryDiskToEarlySize);
    actions.push(UserAction::ShrinkOnlyTheChangingDeviceToEarlySize);
    actions.push(UserAction::GrowEveryDisk);
    actions.push(UserAction::GrowOnlyTheChangingDevice);
    for disk in 0..DISK_COUNT {
        actions.push(UserAction::ReplaceDiskWithEarlyImage(disk));
    }
    if family == Family::GrownBefore {
        for disk in 0..DISK_COUNT {
            actions.push(UserAction::ReplaceDiskWithFrontImage(disk));
        }
    }
    actions
}

fn mirrored_disk(table: Table) -> [Slot; SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE] {
    [Slot::SelfVerified {
        generation: 1,
        table,
        carries_intent_marker: false,
    }; SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE]
}

/// 断电之后的盘面与用户动作之后的探测值（每块盘此刻的整槽数）。
fn board_and_probe_after_user_action(
    action: UserAction,
    shape: &Shape,
    board_at_power_loss: &Board,
) -> (Board, Table) {
    let mut board = *board_at_power_loss;
    let mut probe_sizes = shape.back;
    match action {
        UserAction::LeaveAlone => {}
        UserAction::ShrinkEveryDiskToFrontSize => probe_sizes = shape.front,
        UserAction::ShrinkOnlyTheChangingDeviceToFrontSize => {
            probe_sizes[DEVICE_THAT_CHANGES_ALONE] = shape.front[DEVICE_THAT_CHANGES_ALONE]
        }
        UserAction::ShrinkEveryDiskToEarlySize => probe_sizes = shape.early,
        UserAction::ShrinkOnlyTheChangingDeviceToEarlySize => {
            probe_sizes[DEVICE_THAT_CHANGES_ALONE] = shape.early[DEVICE_THAT_CHANGES_ALONE]
        }
        UserAction::GrowEveryDisk => {
            for disk in 0..DISK_COUNT {
                probe_sizes[disk] += GROW_STEP_SLOTS;
            }
        }
        UserAction::GrowOnlyTheChangingDevice => {
            probe_sizes[DEVICE_THAT_CHANGES_ALONE] += GROW_STEP_SLOTS
        }
        UserAction::ReplaceDiskWithEarlyImage(disk) => {
            probe_sizes[disk] = shape.early[disk];
            board[disk] = mirrored_disk(shape.early);
        }
        UserAction::ReplaceDiskWithFrontImage(disk) => {
            probe_sizes[disk] = shape.front[disk];
            board[disk] = mirrored_disk(shape.front);
        }
    }
    (board, probe_sizes)
}
/// 一条历史（观测）：用户动作之后的盘面、探测值、已用落点上界、它落在写序的哪个断电点。
#[derive(Clone, Copy, Debug)]
struct HistoryObservation {
    family: Family,
    board: Board,
    probe_sizes: Table,
    landing: Table,
    crash_point_index: usize,
    is_early_image: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum CanonicalSlot {
    Unreadable,
    Table {
        table: Table,
        carries_intent_marker: bool,
    },
}

type CanonicalBoard = [[CanonicalSlot; SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE]; DISK_COUNT];
/// 观测类的键：盘面的规范形（每块盘两槽内容的无序对，槽内容取读不出或表加意图标记，不含世代号与槽号）加探测值。
type ClassKey = (CanonicalBoard, Table);

fn canonical_board_of(board: &Board) -> CanonicalBoard {
    let mut canonical =
        [[CanonicalSlot::Unreadable; SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE]; DISK_COUNT];
    for disk in 0..DISK_COUNT {
        for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
            canonical[disk][slot_index] = match board[disk][slot_index] {
                Slot::Unreadable => CanonicalSlot::Unreadable,
                Slot::SelfVerified {
                    table,
                    carries_intent_marker,
                    ..
                } => CanonicalSlot::Table {
                    table,
                    carries_intent_marker,
                },
            };
        }
        canonical[disk].sort();
    }
    canonical
}

fn class_key_of(observation: &HistoryObservation) -> ClassKey {
    (
        canonical_board_of(&observation.board),
        observation.probe_sizes,
    )
}

/// 有单元的落点越过这块盘此刻的盘尾的盘（严格大于：落点正好在盘尾的整槽内不算越界）。
fn beyond_the_end(landing: Table, probe_sizes: Table) -> DiskSet {
    let mut exceeds = NOBODY_NAMED;
    for disk in 0..DISK_COUNT {
        exceeds[disk] = landing[disk] > probe_sizes[disk];
    }
    exceeds
}

fn disks_without_a_valid_slot(board: &Board) -> usize {
    board
        .iter()
        .filter(|slots| slots.iter().all(|slot| *slot == Slot::Unreadable))
        .count()
}

/// 一条臂的写序生成的全部历史，加按观测类算出的必点名集。
struct HistorySet {
    observations: Vec<HistoryObservation>,
    /// 必点名集 `N*(键)`：这个观测类里全部历史的越界的并集。
    necessarily_named_by_class: BTreeMap<ClassKey, DiskSet>,
    /// 用户动作之后某块盘一个有效槽都没有的历史数（出模型，预期 0）；这样的历史不进 `observations`。
    out_of_model_histories: usize,
    raw_state_count: usize,
    distinct_board_count: usize,
    crash_point_count: usize,
    last_segment_fully_persisted_index: usize,
}

impl HistorySet {
    fn class_union_of(&self, observation: &HistoryObservation) -> DiskSet {
        *self
            .necessarily_named_by_class
            .get(&class_key_of(observation))
            .expect("every observation belongs to a class built from the same observations")
    }
}

/// 一条臂的历史集：族、形状、点位、断电点、用户动作的笛卡尔积（挂载种类不进历史元组，每条历史各有可写与只读两次挂载）。
fn build_history_set(
    steps: &[PlanStep],
    newest_slot_of_each_disk: [usize; DISK_COUNT],
    shapes: &[Shape],
) -> HistorySet {
    let mut observations = Vec::new();
    let mut out_of_model_histories = 0;
    let mut raw_state_count = 0;
    let mut distinct_board_count = 0;
    let mut crash_point_count = 0;
    let mut last_segment_fully_persisted_index = 0;
    for (shape_index, shape) in shapes.iter().enumerate() {
        let start = start_board(newest_slot_of_each_disk, shape.front);
        let points = crash_points(&start, steps, shape.front, shape.back);
        if shape_index == 0 {
            raw_state_count = points.len() - 1;
            distinct_board_count = points
                .iter()
                .filter(|point| !point.is_end_of_use)
                .map(|point| canonical_board_of(&point.board))
                .collect::<BTreeSet<_>>()
                .len();
            crash_point_count = points.len();
            last_segment_fully_persisted_index = points
                .iter()
                .position(|point| point.is_last_segment_fully_persisted)
                .expect("the write plan has a state where its last persistence segment is fully persisted");
        }
        let footprints: &[Footprint] = if shape.family == Family::GrownBefore {
            &[Footprint::Full, Footprint::Few]
        } else {
            &[Footprint::Full]
        };
        for footprint in footprints {
            for (crash_point_index, point) in points.iter().enumerate() {
                if *footprint == Footprint::Few && point.is_end_of_use {
                    continue;
                }
                let landing = landing_of(*footprint, point.is_end_of_use, shape);
                for action in actions_of(shape.family) {
                    let (board, probe_sizes) =
                        board_and_probe_after_user_action(action, shape, &point.board);
                    if disks_without_a_valid_slot(&board) > 0 {
                        out_of_model_histories += 1;
                        continue;
                    }
                    observations.push(HistoryObservation {
                        family: shape.family,
                        board,
                        probe_sizes,
                        landing,
                        crash_point_index,
                        is_early_image: matches!(action, UserAction::ReplaceDiskWithEarlyImage(_)),
                    });
                }
            }
        }
    }
    let mut necessarily_named_by_class: BTreeMap<ClassKey, DiskSet> = BTreeMap::new();
    for observation in &observations {
        let beyond = beyond_the_end(observation.landing, observation.probe_sizes);
        let class_union = necessarily_named_by_class
            .entry(class_key_of(observation))
            .or_insert(NOBODY_NAMED);
        for disk in 0..DISK_COUNT {
            class_union[disk] = class_union[disk] || beyond[disk];
        }
    }
    HistorySet {
        observations,
        necessarily_named_by_class,
        out_of_model_histories,
        raw_state_count,
        distinct_board_count,
        crash_point_count,
        last_segment_fully_persisted_index,
    }
}
fn valid_tables_of(board: &Board) -> Vec<Table> {
    let mut tables = Vec::new();
    for slots in board {
        for slot in slots {
            if let Slot::SelfVerified { table, .. } = slot {
                tables.push(*table);
            }
        }
    }
    tables
}

fn marked_tables_of(board: &Board) -> Vec<Table> {
    let mut tables = Vec::new();
    for slots in board {
        for slot in slots {
            if let Slot::SelfVerified {
                table,
                carries_intent_marker: true,
                ..
            } = slot
            {
                tables.push(*table);
            }
        }
    }
    tables
}

fn componentwise_minimum(tables: &[Table]) -> Table {
    let mut lowest = tables[0];
    for table in tables {
        for disk in 0..DISK_COUNT {
            lowest[disk] = lowest[disk].min(table[disk]);
        }
    }
    lowest
}

fn componentwise_maximum(tables: &[Table]) -> Table {
    let mut highest = tables[0];
    for table in tables {
        for disk in 0..DISK_COUNT {
            highest[disk] = highest[disk].max(table[disk]);
        }
    }
    highest
}

/// 全部表两两逐项可比（一张的每一项都不大于另一张，或都不小于）。
fn tables_form_a_chain(tables: &[Table]) -> bool {
    tables.iter().all(|left| {
        tables.iter().all(|right| {
            let left_not_above_right = (0..DISK_COUNT).all(|disk| left[disk] <= right[disk]);
            let right_not_above_left = (0..DISK_COUNT).all(|disk| right[disk] <= left[disk]);
            left_not_above_right || right_not_above_left
        })
    })
}

fn all_tables_equal(tables: &[Table]) -> bool {
    tables.iter().all(|table| *table == tables[0])
}

/// 所有臂共用的外壳（登记第一节读法第 5 条）：变小的盘 = 探测值小于比较表的盘；没有变小的盘可写挂载做成，
/// 有就拒（D12（目标介质） 已定项 4）；只读挂载做成并点名变小的盘。
fn standard_outcome(comparison_table: Table, probe_sizes: Table) -> Decision {
    let mut named = NOBODY_NAMED;
    for disk in 0..DISK_COUNT {
        named[disk] = probe_sizes[disk] < comparison_table[disk];
    }
    Decision {
        writable_mount_made: named == NOBODY_NAMED,
        read_only_mount_made: true,
        named_by_read_only: named,
    }
}

/// 一条臂对（盘面，探测值）的决定：纯函数，只看有效槽的表与意图标记，不看槽号与世代号。盘面上至少有一个有效槽。
fn arm_decision(arm: Arm, board: &Board, probe_sizes: Table) -> Decision {
    let valid_tables = valid_tables_of(board);
    let marked_tables = marked_tables_of(board);
    match arm {
        Arm::ChainThenLowest => {
            if !tables_form_a_chain(&valid_tables) {
                return WHOLE_POOL_REFUSED;
            }
            let table_in_force = componentwise_minimum(&valid_tables);
            standard_outcome(table_in_force, probe_sizes)
        }
        Arm::HighestTable => {
            let table_in_force = componentwise_maximum(&valid_tables);
            standard_outcome(table_in_force, probe_sizes)
        }
        Arm::OldTableWhileIntentMarkerIsOnTheDisk => {
            let table_in_force = if marked_tables.is_empty() {
                componentwise_maximum(&valid_tables)
            } else {
                componentwise_minimum(&marked_tables)
            };
            standard_outcome(table_in_force, probe_sizes)
        }
        Arm::LowestToReadHighestToProbeWithAbandon => {
            let lowest_table = componentwise_minimum(&valid_tables);
            let comparison_table = componentwise_maximum(&valid_tables);
            let outcome = standard_outcome(comparison_table, probe_sizes);
            let tables_differ = !all_tables_equal(&valid_tables);
            let probe_reaches_lowest_table =
                (0..DISK_COUNT).all(|disk| probe_sizes[disk] >= lowest_table[disk]);
            let may_abandon = tables_differ && probe_reaches_lowest_table;
            Decision {
                writable_mount_made: outcome.writable_mount_made || may_abandon,
                ..outcome
            }
        }
        Arm::OnlyIfEveryTableIsEqual => {
            if !all_tables_equal(&valid_tables) {
                return WHOLE_POOL_REFUSED;
            }
            standard_outcome(valid_tables[0], probe_sizes)
        }
    }
}

/// 对照读者的决定。按类判官臂读的是这条历史所在观测类的必点名集：非空则可写拒，空则做成，只读点名它。
fn control_decision(
    control: ControlReader,
    observation: &HistoryObservation,
    set: &HistorySet,
) -> Decision {
    match control {
        ControlReader::AllowEverything => Decision {
            writable_mount_made: true,
            read_only_mount_made: true,
            named_by_read_only: NOBODY_NAMED,
        },
        ControlReader::RejectEveryWritableMount => Decision {
            writable_mount_made: false,
            read_only_mount_made: true,
            named_by_read_only: NOBODY_NAMED,
        },
        ControlReader::NameWhatTheClassOverruns => {
            let necessarily_named = set.class_union_of(observation);
            Decision {
                writable_mount_made: necessarily_named == NOBODY_NAMED,
                read_only_mount_made: true,
                named_by_read_only: necessarily_named,
            }
        }
    }
}

fn reader_decision(reader: Reader, observation: &HistoryObservation, set: &HistorySet) -> Decision {
    match reader {
        Reader::Arm(arm) => arm_decision(arm, &observation.board, observation.probe_sizes),
        Reader::Control(control) => control_decision(control, observation, set),
    }
}

/// 漏放行（可写）：可写挂载做成，而有单元的落点越过某块盘此刻的盘尾。
fn is_leak_through_writable_mount(decision: Decision, beyond: DiskSet) -> bool {
    decision.writable_mount_made && beyond != NOBODY_NAMED
}

/// 漏放行（只读）：只读挂载做成，而越界的盘里有没被点名的。
fn is_leak_through_read_only_mount(decision: Decision, beyond: DiskSet) -> bool {
    decision.read_only_mount_made
        && (0..DISK_COUNT).any(|disk| beyond[disk] && !decision.named_by_read_only[disk])
}

/// 误拒（岔路单字面读法）：可写挂载被拒，而没有任何落点越界。
fn is_literal_misrefusal(decision: Decision, beyond: DiskSet) -> bool {
    !decision.writable_mount_made && beyond == NOBODY_NAMED
}

/// 误拒（可避免读法）：可写挂载被拒，而这条历史所在的观测类里没有任何历史越界（换一条读者规则本可以放行）。
fn is_avoidable_misrefusal(decision: Decision, named_by_the_whole_class: DiskSet) -> bool {
    !decision.writable_mount_made && named_by_the_whole_class == NOBODY_NAMED
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ReadingKind {
    LeakThroughWritableMount,
    LeakThroughReadOnlyMount,
    LiteralMisrefusal,
    AvoidableMisrefusal,
    WritableMountRefused,
}

const READING_KIND_COUNT: usize = 5;
const ALL_READINGS: [ReadingKind; READING_KIND_COUNT] = [
    ReadingKind::LeakThroughWritableMount,
    ReadingKind::LeakThroughReadOnlyMount,
    ReadingKind::LiteralMisrefusal,
    ReadingKind::AvoidableMisrefusal,
    ReadingKind::WritableMountRefused,
];

fn reading_name(reading: ReadingKind) -> &'static str {
    match reading {
        ReadingKind::LeakThroughWritableMount => "leak_writable",
        ReadingKind::LeakThroughReadOnlyMount => "leak_read_only",
        ReadingKind::LiteralMisrefusal => "misrefusal_literal",
        ReadingKind::AvoidableMisrefusal => "misrefusal_avoidable",
        ReadingKind::WritableMountRefused => "writable_refused",
    }
}

/// 一条历史上各个读数是否成立，次序同 `ALL_READINGS`。
fn reading_flags(
    decision: Decision,
    observation: &HistoryObservation,
    set: &HistorySet,
) -> [bool; READING_KIND_COUNT] {
    let own_beyond_the_end = beyond_the_end(observation.landing, observation.probe_sizes);
    let named_by_the_whole_class = set.class_union_of(observation);
    [
        is_leak_through_writable_mount(decision, own_beyond_the_end),
        is_leak_through_read_only_mount(decision, own_beyond_the_end),
        is_literal_misrefusal(decision, own_beyond_the_end),
        is_avoidable_misrefusal(decision, named_by_the_whole_class),
        !decision.writable_mount_made,
    ]
}

/// 一个读者在一条臂的历史集上的全部读数。
#[derive(Clone, PartialEq, Eq, Debug)]
struct ReaderReadings {
    counts: [usize; READING_KIND_COUNT],
    counts_on_early_image_histories: [usize; READING_KIND_COUNT],
    /// 每个断电点序号上各读数的计数（点位「满」与「少」、全部形状与动作加在一起）。
    trajectory: Vec<[usize; READING_KIND_COUNT]>,
    classes: usize,
    classes_with_differing_decisions: usize,
    whole_pool_refusals: usize,
}

fn readings_of_reader(reader: Reader, set: &HistorySet) -> ReaderReadings {
    let mut counts = [0usize; READING_KIND_COUNT];
    let mut counts_on_early_image_histories = [0usize; READING_KIND_COUNT];
    let mut trajectory = vec![[0usize; READING_KIND_COUNT]; set.crash_point_count];
    let mut first_decision_of_class: BTreeMap<ClassKey, Decision> = BTreeMap::new();
    let mut classes_with_differing_decisions: BTreeSet<ClassKey> = BTreeSet::new();
    let mut whole_pool_refusals = 0;
    for observation in &set.observations {
        let decision = reader_decision(reader, observation, set);
        if decision == WHOLE_POOL_REFUSED {
            whole_pool_refusals += 1;
        }
        let key = class_key_of(observation);
        match first_decision_of_class.get(&key).copied() {
            None => {
                first_decision_of_class.insert(key, decision);
            }
            Some(first_decision) => {
                if first_decision != decision {
                    classes_with_differing_decisions.insert(key);
                }
            }
        }
        let flags = reading_flags(decision, observation, set);
        for reading in ALL_READINGS {
            let index = reading as usize;
            if flags[index] {
                counts[index] += 1;
                trajectory[observation.crash_point_index][index] += 1;
                if observation.is_early_image {
                    counts_on_early_image_histories[index] += 1;
                }
            }
        }
    }
    ReaderReadings {
        counts,
        counts_on_early_image_histories,
        trajectory,
        classes: first_decision_of_class.len(),
        classes_with_differing_decisions: classes_with_differing_decisions.len(),
        whole_pool_refusals,
    }
}

/// 互相独立的单位按次序切片并行：每个线程从共用的计数器领下一个单位，结果按单元次序合并，输出与线程数无关。
fn parallel_map<Item: Sync, Output: Send>(
    items: &[Item],
    threads: usize,
    work: impl Fn(&Item) -> Output + Sync,
) -> Vec<Output> {
    let next_item = AtomicUsize::new(0);
    let thread_count = threads_used_for(items.len(), threads);
    let mut collected: Vec<(usize, Output)> = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _worker in 0..thread_count {
            handles.push(scope.spawn(|| {
                let mut finished = Vec::new();
                loop {
                    let index = next_item.fetch_add(1, Ordering::SeqCst);
                    if index >= items.len() {
                        break;
                    }
                    finished.push((index, work(&items[index])));
                }
                finished
            }));
        }
        let mut all_finished = Vec::new();
        for handle in handles {
            all_finished.extend(handle.join().expect("a worker thread panicked"));
        }
        all_finished
    });
    collected.sort_by_key(|(index, _)| *index);
    collected.into_iter().map(|(_, output)| output).collect()
}

/// 这一批单位实际用几个线程：配置的线程数与单位数取小，至少 1。
fn threads_used_for(unit_count: usize, configured_threads: usize) -> usize {
    configured_threads.min(unit_count).max(1)
}
/// 一条臂的历史集加四个读者（臂自己与三条对照读者）的读数。
struct ArmAnalysis {
    arm: Arm,
    set: HistorySet,
    arm_readings: ReaderReadings,
    control_readings: Vec<(ControlReader, ReaderReadings)>,
}

/// 并行单位：（臂下标，读者）。每条臂一个自己的读者单位，加三条对照读者各一个，对照读者循环遍历臂表里的每一条臂。
fn reader_units() -> Vec<(usize, Reader)> {
    let mut units: Vec<(usize, Reader)> = ARMS
        .iter()
        .enumerate()
        .map(|(arm_index, arm)| (arm_index, Reader::Arm(*arm)))
        .collect();
    let arms_under_control: &[Arm] = &ARMS;
    for (arm_index, _arm) in arms_under_control.iter().enumerate() {
        for control in CONTROL_READERS {
            units.push((arm_index, Reader::Control(control)));
        }
    }
    units
}

/// 每块盘世代号最大的槽下标的四个组合：主格取两块盘都在槽 1，另三个只在不变性核对里跑。
const NEWEST_SLOTS_OF_THE_MAIN_GRID: [usize; DISK_COUNT] = [1, 1];
const NEWEST_SLOT_COMBINATIONS_BESIDES_THE_MAIN_GRID: [[usize; DISK_COUNT]; 3] =
    [[0, 0], [0, 1], [1, 0]];

fn analyse_arms(
    newest_slot_of_each_disk: [usize; DISK_COUNT],
    shapes: &[Shape],
    threads: usize,
) -> Vec<ArmAnalysis> {
    let sets = parallel_map(&ARMS, threads, |arm| {
        build_history_set(&write_plan_of(*arm), newest_slot_of_each_disk, shapes)
    });
    let units = reader_units();
    let readings = parallel_map(&units, threads, |unit| {
        readings_of_reader(unit.1, &sets[unit.0])
    });
    let mut analyses = Vec::new();
    for (arm_index, set) in sets.into_iter().enumerate() {
        let mut arm_readings = None;
        let mut control_readings = Vec::new();
        for (unit, unit_readings) in units.iter().zip(readings.iter()) {
            if unit.0 != arm_index {
                continue;
            }
            match unit.1 {
                Reader::Arm(_) => arm_readings = Some(unit_readings.clone()),
                Reader::Control(control) => control_readings.push((control, unit_readings.clone())),
            }
        }
        analyses.push(ArmAnalysis {
            arm: ARMS[arm_index],
            set,
            arm_readings: arm_readings.expect("every arm has its own reader unit"),
            control_readings,
        });
    }
    analyses
}

fn analysis_of(analyses: &[ArmAnalysis], arm: Arm) -> &ArmAnalysis {
    analyses
        .iter()
        .find(|analysis| analysis.arm == arm)
        .expect("every arm in the arm table has an analysis")
}

fn control_readings_of(analysis: &ArmAnalysis, control: ControlReader) -> &ReaderReadings {
    &analysis
        .control_readings
        .iter()
        .find(|(candidate, _)| *candidate == control)
        .expect("every arm has all three control readers")
        .1
}

/// 两份分析的全部读数（臂自己、三条对照读者，含轨迹与类计数）逐项相同。
fn analyses_agree(left: &ArmAnalysis, right: &ArmAnalysis) -> bool {
    left.arm_readings == right.arm_readings && left.control_readings == right.control_readings
}

fn zero_or_count(count: usize) -> String {
    if count == 0 {
        "zero".to_string()
    } else {
        count.to_string()
    }
}

fn describe_table(table: Table) -> String {
    format!("{},{}", table[0], table[1])
}

fn describe_disk_set(disks: DiskSet) -> String {
    let named: Vec<String> = (0..DISK_COUNT)
        .filter(|disk| disks[*disk])
        .map(|disk| disk.to_string())
        .collect();
    format!("{{{}}}", named.join(","))
}

fn describe_decision(decision: Decision) -> String {
    let writable = if decision.writable_mount_made {
        "made"
    } else {
        "refused"
    };
    let read_only = if decision.read_only_mount_made {
        "made"
    } else {
        "refused"
    };
    format!(
        "writable={writable} read_only={read_only} named={}",
        describe_disk_set(decision.named_by_read_only)
    )
}

/// 登记第四节第 7 条的元组数与状态数（按每条臂的写序）：（原始状态数，去重后盘面数，可写元组数，族 D 可写元组数，族 S 可写元组数）。
fn expected_history_counts(arm: Arm) -> (usize, usize, usize, usize, usize) {
    match arm {
        Arm::OldTableWhileIntentMarkerIsOnTheDisk => (27, 25, 2207, 1815, 392),
        Arm::ChainThenLowest
        | Arm::HighestTable
        | Arm::LowestToReadHighestToProbeWithAbandon
        | Arm::OnlyIfEveryTableIsEqual => (18, 17, 1487, 1221, 266),
    }
}

fn grown_family_history_count(set: &HistorySet) -> usize {
    set.observations
        .iter()
        .filter(|observation| observation.family == Family::GrownBefore)
        .count()
}
fn history_space_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    analyses
        .iter()
        .map(|analysis| {
            let (writes, barriers) = count_writes_and_pool_barriers(&write_plan_of(analysis.arm));
            let writable_tuples = analysis.set.observations.len();
            let grown_family = grown_family_history_count(&analysis.set);
            format!(
                "name=history_space arm={} writes={writes} pool_barriers={barriers} raw_states={} distinct_boards={} crash_points={} \
                 writable_tuples={writable_tuples} read_only_tuples={writable_tuples} grown_family_tuples={grown_family} \
                 never_grown_family_tuples={} out_of_model_histories={}",
                arm_name(analysis.arm),
                analysis.set.raw_state_count,
                analysis.set.distinct_board_count,
                analysis.set.crash_point_count,
                writable_tuples - grown_family,
                zero_or_count(analysis.set.out_of_model_histories),
            )
        })
        .collect()
}

fn reading_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    let mut lines = Vec::new();
    for analysis in analyses {
        let denominator = analysis.set.observations.len();
        for reading in ALL_READINGS {
            let count = analysis.arm_readings.counts[reading as usize];
            lines.push(format!(
                "name=reading arm={} reading={} count={count} denominator={denominator} exists={}",
                arm_name(analysis.arm),
                reading_name(reading),
                count >= 1
            ));
        }
    }
    lines
}

fn control_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    let mut lines = Vec::new();
    for analysis in analyses {
        let denominator = analysis.set.observations.len();
        for control in CONTROL_READERS {
            let counts = control_readings_of(analysis, control).counts;
            lines.push(format!(
                "name=control arm={} reader={} leak_writable={} leak_read_only={} misrefusal_literal={} misrefusal_avoidable={} \
                 writable_denominator={denominator} read_only_denominator={denominator}",
                arm_name(analysis.arm),
                control_reader_name(control),
                counts[ReadingKind::LeakThroughWritableMount as usize],
                counts[ReadingKind::LeakThroughReadOnlyMount as usize],
                counts[ReadingKind::LiteralMisrefusal as usize],
                counts[ReadingKind::AvoidableMisrefusal as usize],
            ));
        }
    }
    lines
}

/// 每个读者（臂自己与三条对照读者）在每个观测类里的决定是否全相同：不同的类数必须是 0。
fn class_consistency_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    let mut lines = Vec::new();
    for analysis in analyses {
        let mut readers: Vec<(String, &ReaderReadings)> =
            vec![("arm".to_string(), &analysis.arm_readings)];
        for control in CONTROL_READERS {
            readers.push((
                control_reader_name(control).to_string(),
                control_readings_of(analysis, control),
            ));
        }
        for (reader_name, readings) in readers {
            lines.push(format!(
                "name=class_consistency arm={} reader={reader_name} classes={} classes_with_differing_decisions={}",
                arm_name(analysis.arm),
                readings.classes,
                zero_or_count(readings.classes_with_differing_decisions),
            ));
        }
    }
    lines
}

fn literal_minus_avoidable_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    analyses
        .iter()
        .map(|analysis| {
            let literal = analysis.arm_readings.counts[ReadingKind::LiteralMisrefusal as usize];
            let avoidable = analysis.arm_readings.counts[ReadingKind::AvoidableMisrefusal as usize];
            format!(
                "name=literal_minus_avoidable arm={} literal={literal} avoidable={avoidable} difference={}",
                arm_name(analysis.arm),
                literal as i64 - avoidable as i64
            )
        })
        .collect()
}

/// 轨迹：每条臂每个读数按断电点序号各报一个计数，连同峰值、为正的断电点个数与两个期末值（登记第八节）。
fn trajectory_lines(analyses: &[ArmAnalysis]) -> Vec<String> {
    let mut lines = Vec::new();
    for analysis in analyses {
        for reading in ALL_READINGS {
            let per_point: Vec<usize> = analysis
                .arm_readings
                .trajectory
                .iter()
                .map(|counts| counts[reading as usize])
                .collect();
            let peak = *per_point
                .iter()
                .max()
                .expect("the trajectory has at least one crash point");
            let peak_at = per_point
                .iter()
                .position(|count| *count == peak)
                .expect("the peak is one of the counts");
            let positive_points = per_point.iter().filter(|count| **count > 0).count();
            let counts_text = per_point
                .iter()
                .map(|count| count.to_string())
                .collect::<Vec<String>>()
                .join(",");
            lines.push(format!(
                "name=trajectory arm={} reading={} counts={counts_text} peak={peak} peak_at={peak_at} positive_points={positive_points} of={} \
                 end_of_use={} last_segment_fully_persisted={}",
                arm_name(analysis.arm),
                reading_name(reading),
                per_point.len(),
                per_point[analysis.set.crash_point_count - 1],
                per_point[analysis.set.last_segment_fully_persisted_index],
            ));
        }
    }
    lines
}
fn push_invariance_lines(
    lines: &mut Vec<String>,
    mismatches: &mut usize,
    variant_name: &str,
    main_analyses: &[ArmAnalysis],
    variant_analyses: &[ArmAnalysis],
) {
    for main in main_analyses {
        let agrees = analyses_agree(main, analysis_of(variant_analyses, main.arm));
        if !agrees {
            *mismatches += 1;
        }
        lines.push(format!(
            "name=invariance arm={} variant={variant_name} equal_to_main={agrees}",
            arm_name(main.arm)
        ));
    }
}

/// 不变性核对（登记第八节与第十节作废条款）：世代号奇偶的另三个组合与另一组保序整数上，全部读数必须与主格逐项相同。
fn invariance_lines(main_analyses: &[ArmAnalysis], threads: usize) -> (Vec<String>, usize) {
    let mut lines = Vec::new();
    let mut mismatches = 0;
    for combination in NEWEST_SLOT_COMBINATIONS_BESIDES_THE_MAIN_GRID {
        let variant_analyses = analyse_arms(combination, &registered_shapes(), threads);
        let variant_name = format!("newest_slots_{}_{}", combination[0], combination[1]);
        push_invariance_lines(
            &mut lines,
            &mut mismatches,
            &variant_name,
            main_analyses,
            &variant_analyses,
        );
    }
    let alternative_analyses = analyse_arms(
        NEWEST_SLOTS_OF_THE_MAIN_GRID,
        &alternative_shapes(),
        threads,
    );
    push_invariance_lines(
        &mut lines,
        &mut mismatches,
        "alternative_integers",
        main_analyses,
        &alternative_analyses,
    );
    (lines, mismatches)
}

/// 登记第四节第 8 至 13 条的手推结局：装置读数与手推对不上时先查装置，再记「手推被推翻」（第十节第 9 条）。
struct HandDerivation {
    claim: &'static str,
    holds: bool,
    reading: String,
}

fn hand_derivations(analyses: &[ArmAnalysis]) -> Vec<HandDerivation> {
    let count_of = |arm: Arm, reading: ReadingKind| {
        analysis_of(analyses, arm).arm_readings.counts[reading as usize]
    };
    let early_count_of = |arm: Arm, reading: ReadingKind| {
        analysis_of(analyses, arm)
            .arm_readings
            .counts_on_early_image_histories[reading as usize]
    };
    let literal_per_arm: Vec<String> = ARMS
        .iter()
        .map(|arm| {
            format!(
                "{}={}",
                arm_name(*arm),
                count_of(*arm, ReadingKind::LiteralMisrefusal)
            )
        })
        .collect();
    let chain_early_writable =
        early_count_of(Arm::ChainThenLowest, ReadingKind::LeakThroughWritableMount);
    let chain_early_read_only =
        early_count_of(Arm::ChainThenLowest, ReadingKind::LeakThroughReadOnlyMount);
    let highest_writable = count_of(Arm::HighestTable, ReadingKind::LeakThroughWritableMount);
    let highest_read_only = count_of(Arm::HighestTable, ReadingKind::LeakThroughReadOnlyMount);
    let highest_avoidable = count_of(Arm::HighestTable, ReadingKind::AvoidableMisrefusal);
    let probe_early_writable = early_count_of(
        Arm::LowestToReadHighestToProbeWithAbandon,
        ReadingKind::LeakThroughWritableMount,
    );
    let baseline_avoidable = count_of(
        Arm::OnlyIfEveryTableIsEqual,
        ReadingKind::AvoidableMisrefusal,
    );
    let chain_whole_pool_refusals = analysis_of(analyses, Arm::ChainThenLowest)
        .arm_readings
        .whole_pool_refusals;
    vec![
        HandDerivation {
            claim: "literal_misrefusal_on_every_arm",
            holds: ARMS.iter().all(|arm| count_of(*arm, ReadingKind::LiteralMisrefusal) >= 1),
            reading: literal_per_arm.join(","),
        },
        HandDerivation {
            claim: "chain_then_lowest_leaks_on_early_image",
            holds: chain_early_writable >= 1 && chain_early_read_only >= 1,
            reading: format!("early_image_leak_writable={chain_early_writable},early_image_leak_read_only={chain_early_read_only}"),
        },
        HandDerivation {
            claim: "highest_table_has_no_leak",
            holds: highest_writable == 0 && highest_read_only == 0,
            reading: format!("leak_writable={highest_writable},leak_read_only={highest_read_only}"),
        },
        HandDerivation {
            claim: "highest_table_misrefuses_avoidably",
            holds: highest_avoidable >= 1,
            reading: format!("misrefusal_avoidable={highest_avoidable}"),
        },
        HandDerivation {
            claim: "lowest_to_read_highest_to_probe_leaks_on_early_image",
            holds: probe_early_writable >= 1,
            reading: format!("early_image_leak_writable={probe_early_writable}"),
        },
        HandDerivation {
            claim: "baseline_misrefuses_avoidably",
            holds: baseline_avoidable >= 1,
            reading: format!("misrefusal_avoidable={baseline_avoidable}"),
        },
        HandDerivation {
            claim: "chain_then_lowest_never_refuses_the_whole_pool",
            holds: chain_whole_pool_refusals == 0,
            reading: format!("whole_pool_refusals={chain_whole_pool_refusals}"),
        },
    ]
}

fn describe_board(board: &Board) -> String {
    let mut disks = Vec::new();
    for (disk, slots) in board.iter().enumerate() {
        let described: Vec<String> = slots
            .iter()
            .map(|slot| match slot {
                Slot::Unreadable => "unreadable".to_string(),
                Slot::SelfVerified {
                    table,
                    carries_intent_marker,
                    ..
                } => {
                    format!(
                        "table({}){}",
                        describe_table(*table),
                        if *carries_intent_marker {
                            "+marker"
                        } else {
                            ""
                        }
                    )
                }
            })
            .collect();
        disks.push(format!("disk{disk}:[{}]", described.join(";")));
    }
    disks.join("|")
}

/// 手推说某条臂在「换回早镜像」上漏放行时，打印这样的第一条历史的盘面、探测值、落点上界与该臂的决定。
fn first_early_image_leak_witness(claim: &str, analysis: &ArmAnalysis) -> String {
    for observation in &analysis.set.observations {
        if !observation.is_early_image {
            continue;
        }
        let decision = reader_decision(Reader::Arm(analysis.arm), observation, &analysis.set);
        let beyond = beyond_the_end(observation.landing, observation.probe_sizes);
        if is_leak_through_writable_mount(decision, beyond)
            || is_leak_through_read_only_mount(decision, beyond)
        {
            return format!(
                "name=hand_derivation_witness claim={claim} board={} probe_sizes={} landing={} decision={}",
                describe_board(&observation.board),
                describe_table(observation.probe_sizes),
                describe_table(observation.landing),
                describe_decision(decision).replace(' ', ",")
            );
        }
    }
    format!("name=hand_derivation_witness claim={claim} witness=none")
}
/// 判决行：每一项检查各占一行；字段取 `true` 或 `zero` 是过，`false` 或大于 0 的违例计数是没过。
fn verdict_lines(
    analyses: &[ArmAnalysis],
    invariance_mismatches: usize,
    derivations: &[HandDerivation],
) -> Vec<String> {
    let allow_everything_leaks_on_every_arm = analyses.iter().all(|analysis| {
        let counts = control_readings_of(analysis, ControlReader::AllowEverything).counts;
        counts[ReadingKind::LeakThroughWritableMount as usize] > 0
            && counts[ReadingKind::LeakThroughReadOnlyMount as usize] > 0
    });
    let reject_every_writable_mount_misrefuses_on_every_arm = analyses.iter().all(|analysis| {
        let counts = control_readings_of(analysis, ControlReader::RejectEveryWritableMount).counts;
        counts[ReadingKind::LiteralMisrefusal as usize] > 0
            && counts[ReadingKind::AvoidableMisrefusal as usize] > 0
    });
    let negative_control_violations: usize = analyses
        .iter()
        .map(|analysis| {
            let counts =
                control_readings_of(analysis, ControlReader::NameWhatTheClassOverruns).counts;
            counts[ReadingKind::LeakThroughWritableMount as usize]
                + counts[ReadingKind::LeakThroughReadOnlyMount as usize]
                + counts[ReadingKind::AvoidableMisrefusal as usize]
        })
        .sum();
    let differing_class_violations: usize = analyses
        .iter()
        .map(|analysis| {
            analysis.arm_readings.classes_with_differing_decisions
                + analysis
                    .control_readings
                    .iter()
                    .map(|(_, readings)| readings.classes_with_differing_decisions)
                    .sum::<usize>()
        })
        .sum();
    let at_least_one_arm_literal_exceeds_avoidable = analyses.iter().any(|analysis| {
        analysis.arm_readings.counts[ReadingKind::LiteralMisrefusal as usize]
            > analysis.arm_readings.counts[ReadingKind::AvoidableMisrefusal as usize]
    });
    let out_of_model_violations: usize = analyses
        .iter()
        .map(|analysis| analysis.set.out_of_model_histories)
        .sum();
    let history_count_mismatches: usize = analyses
        .iter()
        .map(|analysis| {
            let (raw_states, distinct_boards, writable_tuples, grown_family, never_grown_family) =
                expected_history_counts(analysis.arm);
            let grown_family_observed = grown_family_history_count(&analysis.set);
            let observed = [
                analysis.set.raw_state_count,
                analysis.set.distinct_board_count,
                analysis.set.observations.len(),
                grown_family_observed,
                analysis.set.observations.len() - grown_family_observed,
            ];
            let expected = [
                raw_states,
                distinct_boards,
                writable_tuples,
                grown_family,
                never_grown_family,
            ];
            (0..observed.len())
                .filter(|position| observed[*position] != expected[*position])
                .count()
        })
        .sum();
    let hand_derivations_overturned = derivations
        .iter()
        .filter(|derivation| !derivation.holds)
        .count();
    let candidate_readings: Vec<[usize; 4]> = ARMS[..4]
        .iter()
        .map(|arm| {
            let counts = analysis_of(analyses, *arm).arm_readings.counts;
            [
                counts[ReadingKind::LeakThroughWritableMount as usize],
                counts[ReadingKind::LeakThroughReadOnlyMount as usize],
                counts[ReadingKind::LiteralMisrefusal as usize],
                counts[ReadingKind::AvoidableMisrefusal as usize],
            ]
        })
        .collect();
    let any_two_of_the_four_candidate_arms_differ = candidate_readings
        .iter()
        .any(|readings| *readings != candidate_readings[0]);
    vec![
        format!(
            "name=verdict check=positive_controls allow_everything_leaks_on_every_arm={allow_everything_leaks_on_every_arm} \
             reject_every_writable_mount_misrefuses_on_every_arm={reject_every_writable_mount_misrefuses_on_every_arm}"
        ),
        format!("name=verdict check=negative_controls negative_control_violations={}", zero_or_count(negative_control_violations)),
        format!("name=verdict check=decision_is_function_of_observation differing_class_violations={}", zero_or_count(differing_class_violations)),
        format!("name=verdict check=literal_minus_avoidable at_least_one_arm_literal_exceeds_avoidable={at_least_one_arm_literal_exceeds_avoidable}"),
        format!("name=verdict check=invariance invariance_mismatches={}", zero_or_count(invariance_mismatches)),
        format!("name=verdict check=out_of_model out_of_model_violations={}", zero_or_count(out_of_model_violations)),
        format!("name=verdict check=history_counts history_count_mismatches={}", zero_or_count(history_count_mismatches)),
        format!("name=verdict check=hand_derivations hand_derivations_overturned={}", zero_or_count(hand_derivations_overturned)),
        format!("name=verdict check=arms_distinguishable any_two_of_the_four_candidate_arms_differ={any_two_of_the_four_candidate_arms_differ}"),
    ]
}

/// 岔路单行 1：四条候选读者规则加基线臂在同一组历史上的读数，各带分母（命令行参数 `reader-rules-on-histories`）。
fn reader_rules_on_histories_lines(threads: usize) -> Vec<String> {
    let main_analyses = analyse_arms(NEWEST_SLOTS_OF_THE_MAIN_GRID, &registered_shapes(), threads);
    let unit_count = reader_units().len();
    let mut lines = vec![format!(
        "name=config segment=reader-rules-on-histories configured_threads={threads} history_set_phase_threads={} reader_phase_threads={} \
         arms={} reader_units={unit_count} shapes={}",
        threads_used_for(ARMS.len(), threads),
        threads_used_for(unit_count, threads),
        ARMS.len(),
        registered_shapes().len()
    )];
    lines.extend(history_space_lines(&main_analyses));
    lines.extend(reading_lines(&main_analyses));
    lines.extend(control_lines(&main_analyses));
    lines.extend(class_consistency_lines(&main_analyses));
    lines.extend(literal_minus_avoidable_lines(&main_analyses));
    lines.extend(trajectory_lines(&main_analyses));
    let (invariance, invariance_mismatches) = invariance_lines(&main_analyses, threads);
    lines.extend(invariance);
    let derivations = hand_derivations(&main_analyses);
    for derivation in &derivations {
        lines.push(format!(
            "name=hand_derivation claim={} holds={} reading={}",
            derivation.claim, derivation.holds, derivation.reading
        ));
    }
    lines.push(first_early_image_leak_witness(
        "chain_then_lowest_leaks_on_early_image",
        analysis_of(&main_analyses, Arm::ChainThenLowest),
    ));
    lines.push(first_early_image_leak_witness(
        "lowest_to_read_highest_to_probe_leaks_on_early_image",
        analysis_of(&main_analyses, Arm::LowestToReadHighestToProbeWithAbandon),
    ));
    lines.push("name=not_run_after_sufficient item=first_five_witnesses_and_distribution_by_action_and_persistence_segment".to_string());
    lines.push("name=not_run_after_sufficient item=read_only_refusal_count_class_count_and_class_size_histogram".to_string());
    lines.push("name=not_run_this_segment item=existence_judgments_on_the_geometry_sensitivity_sampling_points reason=stability_not_dispatched".to_string());
    lines.push("name=not_run_this_segment item=discrimination_self_proof_of_the_geometry_sensitivity reason=stability_not_dispatched".to_string());
    lines.extend(verdict_lines(
        &main_analyses,
        invariance_mismatches,
        &derivations,
    ));
    lines
}
/// 参照：一次不变大的可写挂载的取号写（条款本身的写序，D22（单元原子性怎么合成） 已定项 8「取号写两轮：每轮逐盘一写、轮后一道池屏障」）。
const REFERENCE_WRITES: usize = ACQUISITION_WRITE_ROUNDS * DISK_COUNT;
const REFERENCE_BARRIERS: usize = ACQUISITION_WRITE_ROUNDS;

/// 岔路单行 2 的读数：每条臂一次变大的可写挂载写几次系统配置槽、几道池屏障，与参照比。
fn write_count_lines() -> Vec<String> {
    let mut lines = Vec::new();
    for arm in ARMS {
        let (writes, barriers) = count_writes_and_pool_barriers(&write_plan_of(arm));
        lines.push(format!(
            "name=write_count arm={} writes={writes} pool_barriers={barriers} reference_writes={REFERENCE_WRITES} \
             reference_pool_barriers={REFERENCE_BARRIERS} extra_writes={} extra_pool_barriers={} more_than_the_reference={}",
            arm_name(arm),
            writes as i64 - REFERENCE_WRITES as i64,
            barriers as i64 - REFERENCE_BARRIERS as i64,
            writes > REFERENCE_WRITES || barriers > REFERENCE_BARRIERS
        ));
    }
    // 行 2 的对照：每盘每轮一写、轮后一道屏障的四轮写序，读数必须是 8 与 4。
    let (control_writes, control_barriers) =
        count_writes_and_pool_barriers(&acquisition_write_plan(false, 4));
    lines.push(format!(
        "name=write_count_control plan=four_rounds writes={control_writes} pool_barriers={control_barriers} expected_writes=8 expected_pool_barriers=4 \
         matches={}",
        control_writes == 8 && control_barriers == 4
    ));
    lines
}

/// 层级 `level` 的一格罩多少整槽：叶罩 812 槽、每上一层乘 169（`allocation_record_tree.rs:289`–`:293` 的 `span_in_slots_at_level`）。
fn span_in_slots(level: u32) -> u64 {
    ALLOCATION_RECORD_TREE_LEAF_SLOTS * ALLOCATION_RECORD_TREE_INTERNAL_FANOUT.pow(level)
}

/// 最小的根层级 R ≥ 1，使各盘 ⌈盘上整槽数 ÷ 层级 R−1 的一格罩的槽数⌉ 之和不超过根的扇出
/// （`allocation_record_tree.rs:96`–`:120` 的 `AllocationRecordTreeGeometry::of_devices`；这里按定义另写，不抄它）。
fn root_level_of(slots_of_each_disk: &[u64]) -> u32 {
    let mut level = 1;
    loop {
        let cells_needed: u64 = slots_of_each_disk
            .iter()
            .map(|slots| slots.div_ceil(span_in_slots(level - 1)))
            .sum();
        if cells_needed <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT {
            return level;
        }
        level += 1;
    }
}

/// 两块等大的盘时根层级 `level` 的上界（每盘整槽数）：两盘各 ⌊扇出 ÷ 2⌋ 格。
fn equal_disks_boundary_slots(level: u32) -> u64 {
    (ALLOCATION_RECORD_TREE_INTERNAL_FANOUT / 2) * span_in_slots(level - 1)
}

fn is_better_split(candidate: Table, current: Table) -> bool {
    let candidate_total = candidate[0] + candidate[1];
    let current_total = current[0] + current[1];
    candidate_total > current_total
        || (candidate_total == current_total
            && candidate[0].abs_diff(candidate[1]) < current[0].abs_diff(current[1]))
}

/// 乙「记到边界为止」在两块盘上的摊法（登记第一节未定项第 2 条）：每盘记录值不低于旧记录、不高于探测值、根层级不变
/// （各盘格数之和不超过扇出）的前提下取整槽总数最大；总数相同取两盘之差最小；再相同取先遇到的（盘 0 格数少的那一组）。
fn best_total_split(old_records: Table, new_probe: Table) -> Table {
    let span = span_in_slots(root_level_of(&old_records) - 1);
    let mut best: Option<Table> = None;
    for cells_of_disk_zero in old_records[0].div_ceil(span)..=ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
    {
        let cells_of_disk_one = ALLOCATION_RECORD_TREE_INTERNAL_FANOUT - cells_of_disk_zero;
        if cells_of_disk_one < old_records[1].div_ceil(span) {
            break;
        }
        let candidate = [
            new_probe[0].min(cells_of_disk_zero * span),
            new_probe[1].min(cells_of_disk_one * span),
        ];
        if candidate[0] < old_records[0] || candidate[1] < old_records[1] {
            continue;
        }
        best = match best {
            Some(current) if !is_better_split(candidate, current) => Some(current),
            _ => Some(candidate),
        };
    }
    best.expect(
        "when the probe is not below the old records, the old records themselves are a candidate",
    )
}

/// 乙按盘号先后贪心的变体（登记第一节未定项第 2 条附带并列报）：盘 0 先取满（扣下盘 1 旧记录要的格数），盘 1 保旧记录。
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "只在单测里钉登记第七节乙的逐盘按盘号贪心变体与它的变异"
    )
)]
fn greedy_split(old_records: Table, new_probe: Table) -> Table {
    let span = span_in_slots(root_level_of(&old_records) - 1);
    let cells_of_disk_zero = ALLOCATION_RECORD_TREE_INTERNAL_FANOUT - old_records[1].div_ceil(span);
    [new_probe[0].min(cells_of_disk_zero * span), old_records[1]]
}

/// 一个行 3 取样点：旧记录与新探测（每盘整槽数）下两种做法变大之后用得上的整槽数。
struct RootLevelPoint {
    old_records: Table,
    new_probe: Table,
    old_level: u32,
    new_level: u32,
    /// 甲「一点不记」：根层级变了就保持旧记录，没变用新探测。
    chain_total: u64,
    /// 乙「记到边界为止」：根层级变了就记总数最大的一组（不超过边界），没变用新探测。
    highest_split: Table,
}

fn root_level_point(old_records: Table, new_probe: Table) -> RootLevelPoint {
    let old_level = root_level_of(&old_records);
    let new_level = root_level_of(&new_probe);
    let level_changes = old_level != new_level;
    RootLevelPoint {
        old_records,
        new_probe,
        old_level,
        new_level,
        chain_total: if level_changes {
            old_records[0] + old_records[1]
        } else {
            new_probe[0] + new_probe[1]
        },
        highest_split: if level_changes {
            best_total_split(old_records, new_probe)
        } else {
            new_probe
        },
    }
}

/// 单元区整槽数：表项（绝对整槽数）减单元区起点。
fn unit_area_slots(absolute_slots: u64) -> u64 {
    absolute_slots - DISK_COUNT as u64 * UNIT_AREA_START_SLOT
}
/// 行 3 取样点：边界之下、跨一道、跨两道（登记第六节），加行 3 的合成对照点（答案手算得出）与变异要的两个边界加一点。
/// 旧尺寸每盘 60000 槽，根层级 1，单元区起点之外有 9824 槽。
const ROOT_LEVEL_POINTS: [(&str, Table, Table); 6] = [
    ("below_boundary", [60000, 60000], [68208, 68208]),
    ("crosses_one_boundary", [60000, 60000], [262144, 262144]),
    (
        "crosses_two_boundaries",
        [60000, 60000],
        [67108864, 67108864],
    ),
    (
        "synthetic_control_crossing",
        [68208, 68208],
        [262144, 262144],
    ),
    (
        "one_over_the_boundary_on_one_disk",
        [60000, 60000],
        [68209, 68208],
    ),
    (
        "one_over_the_boundary_on_both_disks",
        [60000, 60000],
        [68209, 68209],
    ),
];

/// 登记第七节第二张表里这六个点的（甲总数，乙总数）：草稿算术脚本 `arith.py` 的输出（甲 = 层级变了取旧记录总数）。
const DRAFT_TOTALS_OF_ROOT_LEVEL_POINTS: [(u64, u64); 6] = [
    (136416, 136416),
    (120000, 137228),
    (120000, 137228),
    (136416, 137228),
    (136417, 136417),
    (120000, 136417),
];

/// 登记第三节第 8 条：`allocation_record_tree.rs` 的单元测试断言的根层级（每盘整槽数，两块等大盘 → 根层级，测试名）。
const CRATES_ROOT_LEVEL_READINGS: [(u64, u32, &str); 4] = [
    (
        262144,
        2,
        "two_four_gibibyte_devices_give_a_tree_of_height_three",
    ),
    (
        67108864,
        3,
        "two_one_tebibyte_devices_give_a_tree_of_height_four",
    ),
    (
        68208,
        1,
        "half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary",
    ),
    (
        68209,
        2,
        "half_a_slot_at_the_end_of_each_device_does_not_move_the_root_level_across_its_boundary",
    ),
];

const DRAFT_LEVEL_SPANS: [u64; 4] = [812, 137228, 23191532, 3919368908];
const DRAFT_EQUAL_DISK_BOUNDARIES: [u64; 3] = [68208, 11527152, 1948088688];

fn root_level_point_line(label: &str, point: &RootLevelPoint) -> String {
    let old_total = point.old_records[0] + point.old_records[1];
    let new_total = point.new_probe[0] + point.new_probe[1];
    let highest_total = point.highest_split[0] + point.highest_split[1];
    let level_changes = point.old_level != point.new_level;
    format!(
        "name=root_level_point label={label} old_records={} new_probe={} old_root_level={} new_root_level={} root_level_changes={level_changes} \
         chain_total_slots={} highest_total_slots={highest_total} highest_split={} highest_minus_chain_slots={} \
         chain_total_unit_area_slots={} highest_total_unit_area_slots={} highest_minus_chain_unit_area_slots={} \
         growth_available_slots={} highest_keeps_fraction_of_the_growth={:.4} diverges={}",
        describe_table(point.old_records),
        describe_table(point.new_probe),
        point.old_level,
        point.new_level,
        point.chain_total,
        describe_table(point.highest_split),
        highest_total as i64 - point.chain_total as i64,
        unit_area_slots(point.chain_total),
        unit_area_slots(highest_total),
        highest_total as i64 - point.chain_total as i64,
        new_total - old_total,
        (highest_total as i64 - point.chain_total as i64) as f64 / (new_total - old_total) as f64,
        highest_total > point.chain_total
    )
}

/// 岔路单行 2 与行 3 的读数（命令行参数 `write-counts-and-root-level`）。
fn write_counts_and_root_level_lines(threads: usize) -> Vec<String> {
    let mut lines = vec![format!(
        "name=config segment=write-counts-and-root-level configured_threads={threads} root_level_point_phase_threads={} root_level_points={}",
        threads_used_for(ROOT_LEVEL_POINTS.len(), threads),
        ROOT_LEVEL_POINTS.len()
    )];
    lines.extend(write_count_lines());
    for (level, span) in DRAFT_LEVEL_SPANS.iter().enumerate() {
        lines.push(format!(
            "name=level_span level={level} slots={} draft_slots={span}",
            span_in_slots(level as u32)
        ));
    }
    for (boundary_index, draft_boundary) in DRAFT_EQUAL_DISK_BOUNDARIES.iter().enumerate() {
        let level = boundary_index as u32 + 1;
        lines.push(format!(
            "name=level_boundary root_level={level} equal_disks_up_to_slots={} draft_slots={draft_boundary} bytes_per_disk={}",
            equal_disks_boundary_slots(level),
            equal_disks_boundary_slots(level) * SLOT_BYTES
        ));
    }
    for (slots, crates_level, crates_test) in CRATES_ROOT_LEVEL_READINGS {
        let level = root_level_of(&[slots, slots]);
        lines.push(format!(
            "name=crates_cross_check per_disk_slots={slots} root_level={level} crates_root_level={crates_level} crates_test={crates_test} matches={}",
            level == crates_level
        ));
    }
    let points = parallel_map(&ROOT_LEVEL_POINTS, threads, |point| {
        root_level_point(point.1, point.2)
    });
    for (sampling_point, point) in ROOT_LEVEL_POINTS.iter().zip(points.iter()) {
        lines.push(root_level_point_line(sampling_point.0, point));
    }
    lines.push(
        "name=not_run_after_sufficient item=publication_rotation_write_count_after_each_publish"
            .to_string(),
    );
    lines.push("name=not_run_after_sufficient item=abandon_branch_write_count_of_lowest_to_read_highest_to_probe".to_string());
    lines.push("name=not_run_after_sufficient item=highest_greedy_per_disk_variant_and_the_one_disk_grows_across_one_boundary_point".to_string());
    lines.extend(write_counts_and_root_level_verdict_lines(&points));
    lines
}

fn write_counts_and_root_level_verdict_lines(points: &[RootLevelPoint]) -> Vec<String> {
    let arms_that_write_more: Vec<&str> = ARMS
        .iter()
        .filter(|arm| {
            let (writes, barriers) = count_writes_and_pool_barriers(&write_plan_of(**arm));
            writes > REFERENCE_WRITES || barriers > REFERENCE_BARRIERS
        })
        .map(|arm| arm_name(*arm))
        .collect();
    let crates_reading_mismatches = CRATES_ROOT_LEVEL_READINGS
        .iter()
        .filter(|(slots, crates_level, _)| root_level_of(&[*slots, *slots]) != *crates_level)
        .count();
    let span_mismatches = (0..DRAFT_LEVEL_SPANS.len())
        .filter(|level| span_in_slots(*level as u32) != DRAFT_LEVEL_SPANS[*level])
        .count();
    let boundary_mismatches = (0..DRAFT_EQUAL_DISK_BOUNDARIES.len())
        .filter(|index| {
            equal_disks_boundary_slots(*index as u32 + 1) != DRAFT_EQUAL_DISK_BOUNDARIES[*index]
        })
        .count();
    let total_mismatches = points
        .iter()
        .zip(DRAFT_TOTALS_OF_ROOT_LEVEL_POINTS.iter())
        .filter(|(point, draft)| {
            point.chain_total != draft.0
                || point.highest_split[0] + point.highest_split[1] != draft.1
        })
        .count();
    let synthetic_control = &points[3];
    let synthetic_control_total =
        synthetic_control.highest_split[0] + synthetic_control.highest_split[1];
    let synthetic_control_matches_hand_values = synthetic_control.chain_total == 136416
        && synthetic_control_total == 137228
        && synthetic_control_total - synthetic_control.chain_total == 812;
    let non_crossing_points_have_zero_difference = points
        .iter()
        .filter(|point| point.old_level == point.new_level)
        .all(|point| point.highest_split[0] + point.highest_split[1] == point.chain_total);
    let (control_writes, control_barriers) =
        count_writes_and_pool_barriers(&acquisition_write_plan(false, 4));
    vec![
        format!(
            "name=verdict check=write_counts reference_writes={REFERENCE_WRITES} reference_pool_barriers={REFERENCE_BARRIERS} \
             four_round_control_matches={} arms_that_write_more_than_the_reference={}",
            control_writes == 8 && control_barriers == 4,
            if arms_that_write_more.is_empty() { "none".to_string() } else { arms_that_write_more.join(",") }
        ),
        format!("name=verdict check=root_level_cross_check crates_reading_mismatches={}", zero_or_count(crates_reading_mismatches)),
        format!(
            "name=verdict check=independent_arithmetic draft_mismatches={}",
            zero_or_count(span_mismatches + boundary_mismatches + total_mismatches)
        ),
        format!(
            "name=verdict check=root_level_points synthetic_control_matches_hand_values={synthetic_control_matches_hand_values} \
             non_crossing_points_have_zero_difference={non_crossing_points_have_zero_difference}"
        ),
    ]
}

fn main() {
    let threads = worker_thread_count();
    let lines = match std::env::args().nth(1).as_deref() {
        Some("write-counts-and-root-level") => write_counts_and_root_level_lines(threads),
        Some("reader-rules-on-histories") => reader_rules_on_histories_lines(threads),
        _ => {
            eprintln!("用法：e210-capacity-grow-reader-rules write-counts-and-root-level | reader-rules-on-histories");
            std::process::exit(2);
        }
    };
    let mut emitter = Emitter::new();
    let mut output_text = String::new();
    for line in &lines {
        output_text.push_str(&emitter.emit_raw(line));
        output_text.push('\n');
    }
    output_text.push_str(&emitter.finish());
    println!("{output_text}");
}
#[cfg(test)]
mod tests {
    use super::*;

    fn slot_of(table: Table) -> Slot {
        Slot::SelfVerified {
            generation: 1,
            table,
            carries_intent_marker: false,
        }
    }

    fn marked_slot_of(table: Table) -> Slot {
        Slot::SelfVerified {
            generation: 1,
            table,
            carries_intent_marker: true,
        }
    }

    fn board_of(disk_zero: [Slot; 2], disk_one: [Slot; 2]) -> Board {
        [disk_zero, disk_one]
    }

    fn decision(writable_mount_made: bool, read_only_mount_made: bool, named: DiskSet) -> Decision {
        Decision {
            writable_mount_made,
            read_only_mount_made,
            named_by_read_only: named,
        }
    }

    fn made_naming(named: DiskSet) -> Decision {
        decision(true, true, named)
    }

    /// 登记第七节盘面小输入用例 1：混合盘面，每盘一槽 (200, 200)、一槽 (300, 300)，探测 (200, 200)。
    #[test]
    fn mixed_board_at_the_probe_is_made_by_chain_and_by_probe_arm_and_refused_by_highest_and_by_the_baseline(
    ) {
        let board = board_of(
            [slot_of([200, 200]), slot_of([300, 300])],
            [slot_of([200, 200]), slot_of([300, 300])],
        );
        let probe = [200, 200];
        assert_eq!(
            arm_decision(Arm::ChainThenLowest, &board, probe),
            made_naming([false, false])
        );
        assert_eq!(
            arm_decision(Arm::HighestTable, &board, probe),
            decision(false, true, [true, true])
        );
        assert_eq!(
            arm_decision(Arm::OldTableWhileIntentMarkerIsOnTheDisk, &board, probe),
            decision(false, true, [true, true])
        );
        assert_eq!(
            arm_decision(Arm::LowestToReadHighestToProbeWithAbandon, &board, probe),
            made_naming([true, true])
        );
        assert_eq!(
            arm_decision(Arm::OnlyIfEveryTableIsEqual, &board, probe),
            WHOLE_POOL_REFUSED
        );
    }

    /// 小输入用例 2：四槽都 (300, 300)，探测 (200, 200)：五条臂一样，可写拒、只读点名两块盘。
    #[test]
    fn single_table_board_below_its_table_is_refused_writable_and_names_both_disks_on_every_arm() {
        let board = board_of(
            [slot_of([300, 300]), slot_of([300, 300])],
            [slot_of([300, 300]), slot_of([300, 300])],
        );
        for arm in ARMS {
            assert_eq!(
                arm_decision(arm, &board, [200, 200]),
                decision(false, true, [true, true]),
                "{}",
                arm_name(arm)
            );
        }
    }

    /// 第七节第一张表最后一行：表 (200, 200)、探测 (199, 200) 可写拒、只读点名盘 0；探测 (201, 200) 可写做成、只读不点名。
    #[test]
    fn shrunk_disk_is_refused_writable_and_named_read_only_while_grown_disk_is_made_and_not_named_on_every_arm(
    ) {
        let board = board_of(
            [slot_of([200, 200]), slot_of([200, 200])],
            [slot_of([200, 200]), slot_of([200, 200])],
        );
        for arm in ARMS {
            assert_eq!(
                arm_decision(arm, &board, [199, 200]),
                decision(false, true, [true, false]),
                "{}",
                arm_name(arm)
            );
            assert_eq!(
                arm_decision(arm, &board, [201, 200]),
                made_naming([false, false]),
                "{}",
                arm_name(arm)
            );
        }
    }

    /// 小输入用例 3：每盘一槽 (200, 200) 带标记、一槽 (300, 300)，探测 (200, 200)：丙取标记槽的表而做成，乙拒，甲不看标记也做成。
    #[test]
    fn marked_slot_makes_the_intent_marker_arm_take_the_old_table_where_the_highest_table_arm_refuses(
    ) {
        let board = board_of(
            [marked_slot_of([200, 200]), slot_of([300, 300])],
            [marked_slot_of([200, 200]), slot_of([300, 300])],
        );
        let probe = [200, 200];
        assert_eq!(
            arm_decision(Arm::OldTableWhileIntentMarkerIsOnTheDisk, &board, probe),
            made_naming([false, false])
        );
        assert_eq!(
            arm_decision(Arm::HighestTable, &board, probe),
            decision(false, true, [true, true])
        );
        assert_eq!(
            arm_decision(Arm::ChainThenLowest, &board, probe),
            made_naming([false, false])
        );
    }

    /// 小输入用例 4：盘 0 两槽 (200, 100)、盘 1 两槽 (100, 200)，探测 (200, 200)：不成链，甲与基线臂整池拒，乙、丙、丁做成。
    #[test]
    fn tables_that_do_not_form_a_chain_make_the_chain_arm_and_the_baseline_refuse_the_whole_pool() {
        let board = board_of(
            [slot_of([200, 100]), slot_of([200, 100])],
            [slot_of([100, 200]), slot_of([100, 200])],
        );
        let probe = [200, 200];
        assert_eq!(
            arm_decision(Arm::ChainThenLowest, &board, probe),
            WHOLE_POOL_REFUSED
        );
        assert_eq!(
            arm_decision(Arm::OnlyIfEveryTableIsEqual, &board, probe),
            WHOLE_POOL_REFUSED
        );
        assert_eq!(
            arm_decision(Arm::HighestTable, &board, probe),
            made_naming([false, false])
        );
        assert_eq!(
            arm_decision(Arm::OldTableWhileIntentMarkerIsOnTheDisk, &board, probe),
            made_naming([false, false])
        );
        assert_eq!(
            arm_decision(Arm::LowestToReadHighestToProbeWithAbandon, &board, probe),
            made_naming([false, false])
        );
    }

    /// 小输入用例 5：盘 0 两槽 (100, 100)、盘 1 两槽 (200, 200)（盘 0 被换回早镜像），探测 (100, 200)，落点上界 (200, 200)：
    /// 甲可写与只读都漏放行；乙、丙拒可写、只读点名盘 0；丁可写做成（放弃）、只读点名盘 0，可写那一半漏放行。
    #[test]
    fn swapped_in_old_disk_leaks_through_the_chain_arm_both_ways_and_through_the_probe_arm_writable_only(
    ) {
        let board = board_of(
            [slot_of([100, 100]), slot_of([100, 100])],
            [slot_of([200, 200]), slot_of([200, 200])],
        );
        let probe = [100, 200];
        let beyond = beyond_the_end([200, 200], probe);
        assert_eq!(beyond, [true, false]);
        let chain = arm_decision(Arm::ChainThenLowest, &board, probe);
        assert_eq!(chain, made_naming([false, false]));
        assert!(
            is_leak_through_writable_mount(chain, beyond)
                && is_leak_through_read_only_mount(chain, beyond)
        );
        assert_eq!(
            arm_decision(Arm::HighestTable, &board, probe),
            decision(false, true, [true, false])
        );
        assert_eq!(
            arm_decision(Arm::OldTableWhileIntentMarkerIsOnTheDisk, &board, probe),
            decision(false, true, [true, false])
        );
        let probe_arm = arm_decision(Arm::LowestToReadHighestToProbeWithAbandon, &board, probe);
        assert_eq!(probe_arm, made_naming([true, false]));
        assert!(
            is_leak_through_writable_mount(probe_arm, beyond)
                && !is_leak_through_read_only_mount(probe_arm, beyond)
        );
        assert_eq!(
            arm_decision(Arm::OnlyIfEveryTableIsEqual, &board, probe),
            WHOLE_POOL_REFUSED
        );
    }

    /// 小输入用例 7：盘 0 一槽 (100, 100) 带标记、另一槽 (300, 300)；盘 1 一槽 (200, 200) 带标记、另一槽 (300, 300)；探测 (100, 100)：
    /// 丙取各带标记槽的表逐项最小 (100, 100) 而做成；乙取 (300, 300) 而拒；甲取 (100, 100) 而做成。
    #[test]
    fn two_marked_slots_make_the_intent_marker_arm_take_their_lowest_table() {
        let board = board_of(
            [marked_slot_of([100, 100]), slot_of([300, 300])],
            [marked_slot_of([200, 200]), slot_of([300, 300])],
        );
        let probe = [100, 100];
        assert_eq!(
            arm_decision(Arm::OldTableWhileIntentMarkerIsOnTheDisk, &board, probe),
            made_naming([false, false])
        );
        assert_eq!(
            arm_decision(Arm::HighestTable, &board, probe),
            decision(false, true, [true, true])
        );
        assert_eq!(
            arm_decision(Arm::ChainThenLowest, &board, probe),
            made_naming([false, false])
        );
    }

    /// 丁的放弃要求探测值不低于生效表（逐项最小）：每盘一槽 (200, 200)、一槽 (300, 300)，探测 (199, 300) 低于 (200, 200)，放弃不成，可写拒。
    #[test]
    fn probe_arm_refuses_abandoning_when_the_probe_is_below_the_lowest_table() {
        let board = board_of(
            [slot_of([200, 200]), slot_of([300, 300])],
            [slot_of([200, 200]), slot_of([300, 300])],
        );
        assert_eq!(
            arm_decision(
                Arm::LowestToReadHighestToProbeWithAbandon,
                &board,
                [199, 300]
            ),
            decision(false, true, [true, false])
        );
        assert_eq!(
            arm_decision(
                Arm::LowestToReadHighestToProbeWithAbandon,
                &board,
                [200, 300]
            ),
            made_naming([true, false])
        );
    }
    /// 丁的放弃分支去掉「有效槽的表不全相同」是等价变异（登记第九节）：表全相同时探测值低于表必然低于生效表，放弃的另一个条件已不成立。
    /// 把等价性写成测试：枚举全部表全相同的盘面与探测值。
    #[test]
    fn abandoning_is_never_reachable_when_every_table_is_the_same() {
        for table in [[100, 100], [200, 200], [300, 100], [100, 300]] {
            let board = board_of(
                [slot_of(table), slot_of(table)],
                [slot_of(table), slot_of(table)],
            );
            for first_probe in [table[0] - 1, table[0], table[0] + 1] {
                for second_probe in [table[1] - 1, table[1], table[1] + 1] {
                    let probe = [first_probe, second_probe];
                    let outcome = standard_outcome(table, probe);
                    let probe_reaches_lowest_table =
                        (0..DISK_COUNT).all(|disk| probe[disk] >= table[disk]);
                    assert!(
                        outcome.writable_mount_made || !probe_reaches_lowest_table,
                        "table {table:?} probe {probe:?}"
                    );
                    assert_eq!(
                        arm_decision(Arm::LowestToReadHighestToProbeWithAbandon, &board, probe),
                        outcome
                    );
                }
            }
        }
    }

    /// 小输入用例 6：同一份落点上界 (200, 200) 与探测下，做成不点名、做成点名、拒三种结局 × 越界空 / 非空 / 类里有越界，
    /// 六个谓词各给出手算的 0 或 1。
    #[test]
    fn predicates_give_the_hand_computed_answers_on_every_outcome_and_overrun_combination() {
        let landing = [200, 200];
        assert_eq!(
            beyond_the_end(landing, [200, 200]),
            [false, false],
            "落点正好在盘尾不算越界"
        );
        assert_eq!(beyond_the_end(landing, [199, 200]), [true, false]);
        assert_eq!(beyond_the_end(landing, [199, 199]), [true, true]);
        let made_unnamed = made_naming([false, false]);
        let made_named = made_naming([true, false]);
        let refused = decision(false, true, [false, false]);
        let nobody = [false, false];
        let disk_zero = [true, false];
        assert!(!is_leak_through_writable_mount(made_unnamed, nobody));
        assert!(is_leak_through_writable_mount(made_unnamed, disk_zero));
        assert!(is_leak_through_writable_mount(made_named, disk_zero));
        assert!(!is_leak_through_writable_mount(refused, disk_zero));
        assert!(!is_leak_through_read_only_mount(made_unnamed, nobody));
        assert!(is_leak_through_read_only_mount(made_unnamed, disk_zero));
        assert!(!is_leak_through_read_only_mount(made_named, disk_zero));
        assert!(is_leak_through_read_only_mount(made_named, [true, true]));
        assert!(!is_leak_through_read_only_mount(
            WHOLE_POOL_REFUSED,
            disk_zero
        ));
        assert!(is_literal_misrefusal(refused, nobody));
        assert!(!is_literal_misrefusal(refused, disk_zero));
        assert!(!is_literal_misrefusal(made_unnamed, nobody));
        assert!(is_avoidable_misrefusal(refused, nobody));
        assert!(
            !is_avoidable_misrefusal(refused, disk_zero),
            "类里有越界的历史，拒不算可避免"
        );
        assert!(!is_avoidable_misrefusal(made_unnamed, nobody));
    }

    fn plan_counts(with_intent_marker_round: bool, new_table_rounds: usize) -> (usize, usize) {
        count_writes_and_pool_barriers(&acquisition_write_plan(
            with_intent_marker_round,
            new_table_rounds,
        ))
    }

    /// 取号写 = 2 轮 × 2 盘 = 4 次槽写，轮后各一道池屏障，开头那道前面没有写的屏障不算：2 道（D22（单元原子性怎么合成） 已定项 8）。
    #[test]
    fn acquisition_write_plan_of_two_rounds_has_four_writes_and_two_barriers_after_writes() {
        assert_eq!(plan_counts(false, 2), (2 + 2, 1 + 1));
        assert_eq!(
            acquisition_write_plan(false, 2).len(),
            1 + (2 + 1) + (2 + 1),
            "开头一道屏障，之后每轮两次写加一道屏障"
        );
    }

    /// 丙的写序多一轮标记轮：6 次槽写、3 道屏障（行 2 的读数）。
    #[test]
    fn plan_with_the_intent_marker_round_has_six_writes_and_three_barriers() {
        assert_eq!(plan_counts(true, 2), (2 + 2 + 2, 1 + 1 + 1));
    }

    /// 行 2 的对照：每盘每轮一写、轮后一道屏障的四轮写序是 8 次写、4 道屏障；开头前面没有写的那一道不记。
    #[test]
    fn four_round_control_plan_has_eight_writes_and_four_barriers() {
        assert_eq!(plan_counts(false, 4), (4 + 4, 1 + 1 + 1 + 1));
        assert_eq!(
            count_writes_and_pool_barriers(&[PlanStep::PoolBarrier, PlanStep::PoolBarrier]),
            (0, 0)
        );
    }

    /// 每条臂的写序读数：除丙之外是 4 与 2，丙是 6 与 3。
    #[test]
    fn write_plan_of_every_arm_has_the_registered_write_and_barrier_counts() {
        for arm in ARMS {
            let expected = if arm == Arm::OldTableWhileIntentMarkerIsOnTheDisk {
                (6, 3)
            } else {
                (4, 2)
            };
            assert_eq!(
                count_writes_and_pool_barriers(&write_plan_of(arm)),
                expected,
                "{}",
                arm_name(arm)
            );
        }
    }

    /// 槽世代号规则（D22 已定项 16）：从两槽世代号都是 1 起连写，第一次世代号 2 落槽 0，第二次世代号 3 落槽 1，第三次世代号 4 落槽 0。
    #[test]
    fn slot_generation_rule_writes_slot_zero_then_slot_one_then_slot_zero_from_the_formatted_pair()
    {
        let mut board = board_of(
            [slot_of([1, 1]), slot_of([1, 1])],
            [slot_of([1, 1]), slot_of([1, 1])],
        );
        let mut written = Vec::new();
        for _write in 0..3 {
            let (generation, slot_index) = next_write_target(&board, 0);
            board[0][slot_index] = Slot::SelfVerified {
                generation,
                table: [1, 1],
                carries_intent_marker: false,
            };
            written.push((generation, slot_index));
        }
        assert_eq!(written, vec![(2, 0), (3, 1), (4, 0)]);
        let torn_board = board_of(
            [Slot::Unreadable, Slot::Unreadable],
            [slot_of([1, 1]), slot_of([1, 1])],
        );
        assert_eq!(
            next_write_target(&torn_board, 0),
            (1, 1),
            "两槽都读不出时世代号从 1 起"
        );
    }

    /// 持久段与原始状态数（D13（验证路线） 已定项 4）：每段两次写、每次取三态，四写序 18 = 9 + 9，六写序 27 = 9 + 9 + 9。
    #[test]
    fn raw_crash_state_counts_are_eighteen_for_the_four_write_plan_and_twenty_seven_for_the_six_write_plan(
    ) {
        let segment_lengths = |steps: &[PlanStep]| {
            persistence_segments(steps)
                .iter()
                .map(|segment| segment.len())
                .collect::<Vec<usize>>()
        };
        assert_eq!(
            segment_lengths(&acquisition_write_plan(false, 2)),
            vec![2, 2]
        );
        assert_eq!(
            segment_lengths(&acquisition_write_plan(true, 2)),
            vec![2, 2, 2]
        );
        let start = start_board([1, 1], [200, 200]);
        let four_write_points = crash_points(
            &start,
            &acquisition_write_plan(false, 2),
            [200, 200],
            [300, 300],
        );
        let six_write_points = crash_points(
            &start,
            &acquisition_write_plan(true, 2),
            [200, 200],
            [300, 300],
        );
        assert_eq!(four_write_points.len(), 9 + 9 + 1, "原始状态加终点已用");
        assert_eq!(six_write_points.len(), 9 + 9 + 9 + 1);
        assert_eq!(
            four_write_points
                .iter()
                .filter(|point| point.is_end_of_use)
                .count(),
            1
        );
        assert_eq!(outcomes_of_one_slot_write().len(), 3);
    }

    /// 第三态「新旧都读不出」：第一持久段两次写都撕裂（态序号 2 × 3 + 2 = 8）时，盘 0 与盘 1 取号落的槽都读不出，另一槽仍装前表。
    #[test]
    fn torn_write_leaves_the_target_slot_unreadable_and_the_other_slot_with_the_front_table() {
        let start = start_board([1, 1], [200, 200]);
        let points = crash_points(
            &start,
            &acquisition_write_plan(false, 2),
            [200, 200],
            [300, 300],
        );
        let board = points[8].board;
        for disk in 0..DISK_COUNT {
            assert_eq!(
                board[disk][0],
                Slot::Unreadable,
                "世代号 4 落槽 0，撕裂后读不出，旧内容也没了"
            );
            assert_eq!(
                board[disk][1],
                Slot::SelfVerified {
                    generation: 3,
                    table: [200, 200],
                    carries_intent_marker: false
                }
            );
        }
        let nothing_persisted = points[0].board;
        assert_eq!(nothing_persisted, start, "序号 0 = 没有任何写持久");
    }
    /// 层级跨度（登记第七节）：S_0 = 812，S_1 = 137228，S_2 = 23191532，S_3 = 3919368908。
    #[test]
    fn level_spans_are_the_draft_values() {
        assert_eq!(
            [
                span_in_slots(0),
                span_in_slots(1),
                span_in_slots(2),
                span_in_slots(3)
            ],
            [812, 137228, 23191532, 3919368908]
        );
        assert_eq!(137228, 812 * 169);
    }

    /// 两块等大盘的根层级边界（每盘整槽数）：层级 1 到 68208（= 84 × 812）、层级 2 到 11527152、层级 3 到 1948088688；边界上根层级不变，加一槽就进下一层。
    #[test]
    fn equal_disk_boundaries_sit_exactly_where_the_next_root_level_starts() {
        assert_eq!(
            [
                equal_disks_boundary_slots(1),
                equal_disks_boundary_slots(2),
                equal_disks_boundary_slots(3)
            ],
            [68208, 11527152, 1948088688]
        );
        assert_eq!(68208, 84 * 812);
        for (level, boundary) in [(1, 68208u64), (2, 11527152), (3, 1948088688)] {
            assert_eq!(root_level_of(&[boundary, boundary]), level);
            assert_eq!(root_level_of(&[boundary + 1, boundary + 1]), level + 1);
        }
    }

    /// 与 `crates/` 钉死读数对拍的四个输入（`allocation_record_tree.rs` 的三条单元测试）：每盘 262144 槽 → 层级 2，每盘 67108864 槽 → 层级 3，
    /// 每盘 68208 槽 → 层级 1，每盘 68209 槽 → 层级 2。
    #[test]
    fn root_level_arithmetic_agrees_with_the_four_readings_pinned_in_the_crates() {
        assert_eq!(root_level_of(&[262144, 262144]), 2);
        assert_eq!(root_level_of(&[67108864, 67108864]), 3);
        assert_eq!(root_level_of(&[68208, 68208]), 1);
        assert_eq!(root_level_of(&[68209, 68209]), 2);
        assert_eq!(CRATES_ROOT_LEVEL_READINGS.len(), 4);
    }

    /// 六个取样点（登记第四节第 6 条表）：（旧层级，新层级，甲总数，乙总数，乙的逐盘摊法）。
    #[test]
    fn root_level_points_have_the_draft_levels_totals_and_splits() {
        let expected: [(u32, u32, u64, u64, Table); 6] = [
            (1, 1, 136416, 136416, [68208, 68208]),
            (1, 2, 120000, 137228, [68208, 69020]),
            (1, 3, 120000, 137228, [68208, 69020]),
            (1, 2, 136416, 137228, [68208, 69020]),
            (1, 1, 136417, 136417, [68209, 68208]),
            (1, 2, 120000, 136417, [68208, 68209]),
        ];
        for (sampling_point, expected_point) in ROOT_LEVEL_POINTS.iter().zip(expected.iter()) {
            let point = root_level_point(sampling_point.1, sampling_point.2);
            assert_eq!(
                (
                    point.old_level,
                    point.new_level,
                    point.chain_total,
                    point.highest_split[0] + point.highest_split[1],
                    point.highest_split
                ),
                *expected_point,
                "{}",
                sampling_point.0
            );
        }
    }

    /// 行 3 的对照点：旧 (68208, 68208) → 新 (262144, 262144)：甲 136416 = 2 × 68208，乙 137228 = 169 × 812，差 812；必不跨边界的点两做法差是 0。
    #[test]
    fn synthetic_crossing_point_gives_136416_and_137228_and_a_difference_of_812_while_a_non_crossing_point_gives_zero(
    ) {
        let crossing = root_level_point([68208, 68208], [262144, 262144]);
        let highest_total = crossing.highest_split[0] + crossing.highest_split[1];
        assert_eq!(crossing.chain_total, 2 * 68208);
        assert_eq!(highest_total, 169 * 812);
        assert_eq!(highest_total, crossing.chain_total + 812);
        let non_crossing = root_level_point([60000, 60000], [68208, 68208]);
        assert_eq!(
            non_crossing.highest_split[0] + non_crossing.highest_split[1],
            non_crossing.chain_total
        );
    }

    /// 乙的逐盘按盘号贪心变体（第七节）：旧 (60000, 60000) → 新 (262144, 262144)：盘 0 取到 77140、盘 1 不动，总数 137140，
    /// 总数最大的读法是 137228，差 88；边界加一（两盘）贪心 128209、总数最大 136417。
    #[test]
    fn greedy_per_disk_variant_gives_137140_against_the_best_total_137228_and_128209_one_over_the_boundary(
    ) {
        assert_eq!(
            greedy_split([60000, 60000], [262144, 262144]),
            [77140, 60000]
        );
        assert_eq!(77140 + 60000, 137140);
        assert_eq!(95 * 812, 77140);
        assert_eq!(137228 - 137140, 88);
        assert_eq!(
            best_total_split([60000, 60000], [262144, 262144]),
            [68208, 69020]
        );
        assert_eq!(greedy_split([60000, 60000], [68209, 68209]), [68209, 60000]);
        assert_eq!(68209 + 60000, 128209);
        assert_eq!(
            best_total_split([60000, 60000], [68209, 68209]),
            [68208, 68209]
        );
        assert_eq!(
            best_total_split([60000, 60000], [262144, 60000]),
            [77140, 60000]
        );
    }

    /// 单元区整槽数 = 表项 − 单元区起点，两种口径下乙减甲的差相同。
    #[test]
    fn unit_area_slots_subtract_the_unit_area_start_from_each_entry() {
        assert_eq!(unit_area_slots(136416), 136416 - 2 * 50176);
        assert_eq!(
            unit_area_slots(137228) - unit_area_slots(120000),
            137228 - 120000
        );
    }

    /// 历史空间的元组数（登记第四节第 7 条）：四写序可写 1487（族 D 1221、族 S 266）、只读同数，合计 2974；
    /// 六写序可写 2207（族 D 1815、族 S 392），合计 4414；原始状态数 18 / 27，去重后盘面数 17 / 25；全部在模型内。
    #[test]
    fn history_space_has_the_registered_tuple_and_state_counts_for_every_arm() {
        assert_eq!(1221 * 2 + 266 * 2, 2974);
        assert_eq!(1815 * 2 + 392 * 2, 4414);
        for arm in ARMS {
            let set = build_history_set(
                &write_plan_of(arm),
                NEWEST_SLOTS_OF_THE_MAIN_GRID,
                &registered_shapes(),
            );
            let six_write = arm == Arm::OldTableWhileIntentMarkerIsOnTheDisk;
            let grown_family = grown_family_history_count(&set);
            let observed = (
                set.raw_state_count,
                set.distinct_board_count,
                set.crash_point_count,
                set.observations.len(),
                grown_family,
                set.observations.len() - grown_family,
                set.out_of_model_histories,
            );
            let expected = if six_write {
                (27, 25, 28, 2207, 1815, 392, 0)
            } else {
                (18, 17, 19, 1487, 1221, 266, 0)
            };
            assert_eq!(observed, expected, "{}", arm_name(arm));
        }
    }

    /// 世代号奇偶的四个组合都读到同样的原始状态数与去重后盘面数。
    #[test]
    fn raw_and_distinct_board_counts_do_not_depend_on_which_slot_holds_the_newest_generation() {
        let combinations = [NEWEST_SLOTS_OF_THE_MAIN_GRID, [0, 0], [0, 1], [1, 0]];
        for newest_slots in combinations {
            let four_write = build_history_set(
                &acquisition_write_plan(false, 2),
                newest_slots,
                &registered_shapes(),
            );
            let six_write = build_history_set(
                &acquisition_write_plan(true, 2),
                newest_slots,
                &registered_shapes(),
            );
            assert_eq!(
                (four_write.raw_state_count, four_write.distinct_board_count),
                (18, 17),
                "{newest_slots:?}"
            );
            assert_eq!(
                (six_write.raw_state_count, six_write.distinct_board_count),
                (27, 25),
                "{newest_slots:?}"
            );
        }
    }
    fn main_analyses_with(threads: usize) -> Vec<ArmAnalysis> {
        analyse_arms(NEWEST_SLOTS_OF_THE_MAIN_GRID, &registered_shapes(), threads)
    }

    /// 阳性对照（对每一条臂的历史集都跑）：放行一切臂的漏放行（可写与只读）> 0；一律拒臂的字面误拒与可避免误拒 > 0。
    #[test]
    fn allow_everything_leaks_and_reject_every_writable_mount_misrefuses_on_the_history_set_of_every_arm(
    ) {
        for analysis in &main_analyses_with(4) {
            let allow_everything =
                control_readings_of(analysis, ControlReader::AllowEverything).counts;
            assert!(
                allow_everything[ReadingKind::LeakThroughWritableMount as usize] > 0,
                "{}",
                arm_name(analysis.arm)
            );
            assert!(
                allow_everything[ReadingKind::LeakThroughReadOnlyMount as usize] > 0,
                "{}",
                arm_name(analysis.arm)
            );
            let reject_every_writable_mount =
                control_readings_of(analysis, ControlReader::RejectEveryWritableMount).counts;
            assert!(
                reject_every_writable_mount[ReadingKind::LiteralMisrefusal as usize] > 0,
                "{}",
                arm_name(analysis.arm)
            );
            assert!(
                reject_every_writable_mount[ReadingKind::AvoidableMisrefusal as usize] > 0,
                "{}",
                arm_name(analysis.arm)
            );
        }
    }

    /// 阴性对照（对每一条臂的历史集都跑）：按类判官臂的漏放行（可写与只读）与可避免误拒都是 0。
    #[test]
    fn class_oracle_has_no_leak_and_no_avoidable_misrefusal_on_the_history_set_of_every_arm() {
        for analysis in &main_analyses_with(4) {
            let class_oracle =
                control_readings_of(analysis, ControlReader::NameWhatTheClassOverruns).counts;
            assert_eq!(
                class_oracle[ReadingKind::LeakThroughWritableMount as usize],
                0,
                "{}",
                arm_name(analysis.arm)
            );
            assert_eq!(
                class_oracle[ReadingKind::LeakThroughReadOnlyMount as usize],
                0,
                "{}",
                arm_name(analysis.arm)
            );
            assert_eq!(
                class_oracle[ReadingKind::AvoidableMisrefusal as usize],
                0,
                "{}",
                arm_name(analysis.arm)
            );
        }
    }

    /// 决定是观测的函数：每个读者（臂自己与三条对照读者）在每个观测类里的决定全相同，不同的类数是 0；类确实建出来了。
    #[test]
    fn every_reader_decides_the_same_within_each_observation_class_on_every_arm() {
        for analysis in &main_analyses_with(4) {
            assert!(analysis.arm_readings.classes > 0);
            assert_eq!(
                analysis.arm_readings.classes_with_differing_decisions,
                0,
                "{}",
                arm_name(analysis.arm)
            );
            assert_eq!(analysis.control_readings.len(), CONTROL_READERS.len());
            for (control, readings) in &analysis.control_readings {
                assert_eq!(
                    readings.classes_with_differing_decisions,
                    0,
                    "{} {}",
                    arm_name(analysis.arm),
                    control_reader_name(*control)
                );
            }
        }
    }

    /// 观测类的算法生效：有类里既有越界成员、又有自己不越界的成员（单一表 T_后、探测 = 前尺寸：终点未用与终点已用同类），
    /// 所以字面误拒读数减可避免误拒读数在至少一条臂上 ≥ 1。
    #[test]
    fn some_class_mixes_overrunning_and_non_overrunning_histories_so_literal_exceeds_avoidable_on_some_arm(
    ) {
        let analyses = main_analyses_with(4);
        let set = &analysis_of(&analyses, Arm::HighestTable).set;
        let mixed_class_exists = set.observations.iter().any(|observation| {
            beyond_the_end(observation.landing, observation.probe_sizes) == NOBODY_NAMED
                && set.class_union_of(observation) != NOBODY_NAMED
        });
        assert!(mixed_class_exists);
        assert!(analyses.iter().any(|analysis| {
            analysis.arm_readings.counts[ReadingKind::LiteralMisrefusal as usize]
                > analysis.arm_readings.counts[ReadingKind::AvoidableMisrefusal as usize]
        }));
    }

    /// 并行单位：五条臂各一个自己的读者单位，加三条对照读者在每一条臂上各一个，共 5 + 15 = 20 个，每条臂都有三条对照。
    #[test]
    fn every_arm_gets_its_own_reader_unit_and_all_three_control_readers() {
        let units = reader_units();
        assert_eq!(units.len(), 5 + 5 * 3);
        for arm_index in 0..ARMS.len() {
            let own = units
                .iter()
                .filter(|unit| unit.0 == arm_index && matches!(unit.1, Reader::Arm(_)))
                .count();
            let controls = units
                .iter()
                .filter(|unit| unit.0 == arm_index && matches!(unit.1, Reader::Control(_)))
                .count();
            assert_eq!((own, controls), (1, 3), "{}", arm_name(ARMS[arm_index]));
        }
        for analysis in &main_analyses_with(4) {
            assert_eq!(
                analysis.control_readings.len(),
                3,
                "{}",
                arm_name(analysis.arm)
            );
        }
    }

    /// 读者规则不看槽号与世代号也不看数值，只比大小次序：世代号奇偶的另三个组合与另一组保序整数的全部读数与主格逐项相同。
    #[test]
    fn readings_do_not_change_with_the_slot_parity_or_with_the_alternative_integers() {
        let main_analyses = main_analyses_with(4);
        let (lines, mismatches) = invariance_lines(&main_analyses, 4);
        assert_eq!(mismatches, 0);
        assert_eq!(lines.len(), 4 * ARMS.len());
        assert!(lines
            .iter()
            .all(|line| line.ends_with("equal_to_main=true")));
    }

    /// 每条臂每个读数的轨迹加起来等于该读数的计数；终点已用那一点与最后持久段全持久那一点的下标在写序里是预期的位置。
    #[test]
    fn trajectory_sums_to_the_counts_and_marks_the_end_of_use_and_the_last_fully_persisted_point() {
        for analysis in &main_analyses_with(4) {
            for reading in ALL_READINGS {
                let summed: usize = analysis
                    .arm_readings
                    .trajectory
                    .iter()
                    .map(|counts| counts[reading as usize])
                    .sum();
                assert_eq!(
                    summed,
                    analysis.arm_readings.counts[reading as usize],
                    "{}",
                    arm_name(analysis.arm)
                );
            }
            let six_write = analysis.arm == Arm::OldTableWhileIntentMarkerIsOnTheDisk;
            let (crash_points, last_fully_persisted) = if six_write { (28, 22) } else { (19, 13) };
            assert_eq!(
                (
                    analysis.set.crash_point_count,
                    analysis.set.last_segment_fully_persisted_index
                ),
                (crash_points, last_fully_persisted)
            );
        }
    }

    /// 同一份装置同一条命令跑两次、线程数 1 与线程数多时，除配置行里的线程数之外逐字相同（确定性；多线程与单线程对拍）。
    #[test]
    fn output_lines_are_identical_across_runs_and_across_thread_counts_except_the_configured_threads(
    ) {
        let reader_rules_single = reader_rules_on_histories_lines(1);
        assert_eq!(
            reader_rules_single[1..],
            reader_rules_on_histories_lines(1)[1..]
        );
        assert_eq!(
            reader_rules_single[1..],
            reader_rules_on_histories_lines(8)[1..]
        );
        let write_counts_single = write_counts_and_root_level_lines(1);
        assert_eq!(
            write_counts_single[1..],
            write_counts_and_root_level_lines(8)[1..]
        );
    }

    /// 行 2 与行 3 的输出：每条臂一行写数（丙 6 与 3，其余 4 与 2）、四轮写序对照读数 8 与 4，四个与 `crates/` 对拍的输入，六个取样点，判决行全过。
    #[test]
    fn write_counts_and_root_level_output_has_the_registered_lines() {
        let lines = write_counts_and_root_level_lines(4);
        let count_of = |needle: &str| lines.iter().filter(|line| line.contains(needle)).count();
        assert_eq!(count_of("name=write_count arm="), 5);
        assert_eq!(
            count_of("arm=old_table_while_intent_marker writes=6 pool_barriers=3 "),
            1
        );
        assert_eq!(count_of(" writes=4 pool_barriers=2 "), 4);
        assert_eq!(count_of("name=write_count_control plan=four_rounds writes=8 pool_barriers=4 expected_writes=8 expected_pool_barriers=4 matches=true"), 1);
        assert_eq!(count_of("name=crates_cross_check "), 4);
        assert_eq!(count_of("name=root_level_point "), 6);
        assert_eq!(count_of("matches=false"), 0);
        assert_eq!(count_of("name=not_run_after_sufficient item="), 3);
        for verdict in [
            "name=verdict check=write_counts reference_writes=4 reference_pool_barriers=2 four_round_control_matches=true arms_that_write_more_than_the_reference=old_table_while_intent_marker",
            "name=verdict check=root_level_cross_check crates_reading_mismatches=zero",
            "name=verdict check=independent_arithmetic draft_mismatches=zero",
            "name=verdict check=root_level_points synthetic_control_matches_hand_values=true non_crossing_points_have_zero_difference=true",
        ] {
            assert_eq!(count_of(verdict), 1, "{verdict}");
        }
    }

    /// 行 1 的输出：每条臂一行历史空间、五个读数各一行、三条对照各一行（五条臂各自的三条）、判决行里的对照与不变性与计数都过。
    #[test]
    fn reader_rules_output_has_every_arm_and_every_control_and_the_passing_verdicts() {
        let lines = reader_rules_on_histories_lines(4);
        let count_of = |needle: &str| lines.iter().filter(|line| line.contains(needle)).count();
        assert_eq!(count_of("name=history_space arm="), 5);
        assert_eq!(count_of("name=reading arm="), 5 * 5);
        assert_eq!(count_of("name=control arm="), 5 * 3);
        for control in CONTROL_READERS {
            assert_eq!(
                count_of(&format!(" reader={} ", control_reader_name(control))),
                5 + 5,
                "对照读者的 control 行与 class_consistency 行各五条"
            );
        }
        assert_eq!(count_of("name=trajectory arm="), 5 * 5);
        assert_eq!(count_of("name=hand_derivation claim="), 7);
        assert_eq!(count_of("name=hand_derivation_witness claim="), 2);
        assert_eq!(count_of("name=not_run_after_sufficient item="), 2);
        assert_eq!(count_of("name=not_run_this_segment item="), 2);
        for verdict in [
            "name=verdict check=positive_controls allow_everything_leaks_on_every_arm=true reject_every_writable_mount_misrefuses_on_every_arm=true",
            "name=verdict check=negative_controls negative_control_violations=zero",
            "name=verdict check=decision_is_function_of_observation differing_class_violations=zero",
            "name=verdict check=literal_minus_avoidable at_least_one_arm_literal_exceeds_avoidable=true",
            "name=verdict check=invariance invariance_mismatches=zero",
            "name=verdict check=out_of_model out_of_model_violations=zero",
            "name=verdict check=history_counts history_count_mismatches=zero",
        ] {
            assert_eq!(count_of(verdict), 1, "{verdict}");
        }
    }
}

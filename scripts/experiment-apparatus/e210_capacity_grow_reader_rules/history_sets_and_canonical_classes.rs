//! 历史的观测、规范化的槽与棋盘与类键、历史集的构造、有效表与带标记表、逐项最小最大与成链的判定。

use super::*;

/// 一条历史（观测）：用户动作之后的盘面、探测值、已用落点上界、它落在写序的哪个断电点。
#[derive(Clone, Copy, Debug)]
pub(super) struct HistoryObservation {
    pub(super) family: Family,
    pub(super) board: Board,
    pub(super) probe_sizes: Table,
    pub(super) landing: Table,
    pub(super) crash_point_index: usize,
    pub(super) is_early_image: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum CanonicalSlot {
    Unreadable,
    Table {
        table: Table,
        carries_intent_marker: bool,
    },
}

pub(super) type CanonicalBoard = [[CanonicalSlot; SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE]; DISK_COUNT];
/// 观测类的键：盘面的规范形（每块盘两槽内容的无序对，槽内容取读不出或表加意图标记，不含世代号与槽号）加探测值。
pub(super) type ClassKey = (CanonicalBoard, Table);

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

pub(super) fn class_key_of(observation: &HistoryObservation) -> ClassKey {
    (
        canonical_board_of(&observation.board),
        observation.probe_sizes,
    )
}

/// 有单元的落点越过这块盘此刻的盘尾的盘（严格大于：落点正好在盘尾的整槽内不算越界）。
pub(super) fn beyond_the_end(landing: Table, probe_sizes: Table) -> DiskSet {
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
pub(super) struct HistorySet {
    pub(super) observations: Vec<HistoryObservation>,
    /// 必点名集 `N*(键)`：这个观测类里全部历史的越界的并集。
    necessarily_named_by_class: BTreeMap<ClassKey, DiskSet>,
    /// 用户动作之后某块盘一个有效槽都没有的历史数（出模型，预期 0）；这样的历史不进 `observations`。
    pub(super) out_of_model_histories: usize,
    pub(super) raw_state_count: usize,
    pub(super) distinct_board_count: usize,
    pub(super) crash_point_count: usize,
    pub(super) last_segment_fully_persisted_index: usize,
}

impl HistorySet {
    pub(super) fn class_union_of(&self, observation: &HistoryObservation) -> DiskSet {
        *self
            .necessarily_named_by_class
            .get(&class_key_of(observation))
            .expect("every observation belongs to a class built from the same observations")
    }
}

/// 一条臂的历史集：族、形状、点位、断电点、用户动作的笛卡尔积（挂载种类不进历史元组，每条历史各有可写与只读两次挂载）。
pub(super) fn build_history_set(
    steps: &[PlanStep],
    newest_slot_of_each_disk: [usize; DISK_COUNT],
    numbering: Numbering,
) -> HistorySet {
    let shapes = shapes_in(numbering);
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
                        board_and_probe_after_user_action(action, shape, &point.board, numbering);
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
pub(super) fn valid_tables_of(board: &Board) -> Vec<Table> {
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

pub(super) fn marked_tables_of(board: &Board) -> Vec<Table> {
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

pub(super) fn componentwise_minimum(tables: &[Table]) -> Table {
    let mut lowest = tables[0];
    for table in tables {
        for disk in 0..DISK_COUNT {
            lowest[disk] = lowest[disk].min(table[disk]);
        }
    }
    lowest
}

pub(super) fn componentwise_maximum(tables: &[Table]) -> Table {
    let mut highest = tables[0];
    for table in tables {
        for disk in 0..DISK_COUNT {
            highest[disk] = highest[disk].max(table[disk]);
        }
    }
    highest
}

/// 全部表两两逐项可比（一张的每一项都不大于另一张，或都不小于）。
pub(super) fn tables_form_a_chain(tables: &[Table]) -> bool {
    tables.iter().all(|left| {
        tables.iter().all(|right| {
            let left_not_above_right = (0..DISK_COUNT).all(|disk| left[disk] <= right[disk]);
            let right_not_above_left = (0..DISK_COUNT).all(|disk| right[disk] <= left[disk]);
            left_not_above_right || right_not_above_left
        })
    })
}

pub(super) fn all_tables_equal(tables: &[Table]) -> bool {
    tables.iter().all(|table| *table == tables[0])
}

/// 所有臂共用的外壳（登记第一节读法第 5 条）：变小的盘 = 探测值小于比较表的盘；没有变小的盘可写挂载做成，
/// 有就拒（D12（目标介质） 已定项 4）；只读挂载做成并点名变小的盘。
pub(super) fn standard_outcome(comparison_table: Table, probe_sizes: Table) -> Decision {
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

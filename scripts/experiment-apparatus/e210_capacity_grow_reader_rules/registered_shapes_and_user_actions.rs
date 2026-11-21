//! 家族与形状、登记的形状与编号、盘变大的尺寸、足迹与着陆、用户动作与动作后的棋盘和探测。

use super::*;

/// 历史族：池里更早已经变大过并写入过（早尺寸、前尺寸、后尺寸三个尺寸），或从没变大过（早尺寸 = 前尺寸）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Family {
    GrownBefore,
    NeverGrownBefore,
}

/// 形状：三个尺寸各两项（依设备身份 0、1，整槽数）。变大前盘面装前表；本次变大要写的新表是后尺寸那张。
#[derive(Clone, Copy, Debug)]
pub(super) struct Shape {
    pub(super) family: Family,
    pub(super) early: Table,
    pub(super) front: Table,
    pub(super) back: Table,
}

pub(super) fn registered_shapes() -> Vec<Shape> {
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

/// 另一组整数（登记第八节「尺寸取另一组整数」：早 (10, 90)、前 (20, 95)、后 (30, 99)）：每块盘一张换数表，主格的每个整数换成一个新整数，
/// 表按主格整数严格递增，所以同一块盘上主格的大小次序与相等关系换数之后都不变；同一个主格整数在所有形状里换成同一个数。
/// 主格里「再变大」的探测值（后尺寸加 100）最大到 400，表里多给一档。
pub(super) const ALTERNATIVE_NUMBER_OF_MAIN_NUMBER: [[(u64, u64); 4]; DISK_COUNT] = [
    [(100, 10), (200, 20), (300, 30), (400, 40)],
    [(100, 90), (200, 95), (300, 99), (400, 120)],
];

fn alternative_number(disk: usize, main_number: u64) -> u64 {
    ALTERNATIVE_NUMBER_OF_MAIN_NUMBER[disk]
        .iter()
        .find(|(main, _)| *main == main_number)
        .expect("every main number the model produces is in the table")
        .1
}

fn main_number_of_alternative(disk: usize, alternative: u64) -> u64 {
    ALTERNATIVE_NUMBER_OF_MAIN_NUMBER[disk]
        .iter()
        .find(|(_, candidate)| *candidate == alternative)
        .expect("every alternative number the model produces is in the table")
        .0
}

/// 整数的三种写法。主格用登记的 100、200、300；另两种都把形状里的整数按换数表换成另一组，差别只在「再变大」的探测值：
/// 一种把探测值（主格里后尺寸加 100）也按换数表换（大小次序与相等关系整体不变），
/// 一种在换过的后尺寸上直接加 100（登记第八节原文「再变大每盘各加 100 的数不变」的读法）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Numbering {
    Main,
    OrderPreservingOnEveryNumber,
    OrderPreservingWithFixedGrowthStep,
}

pub(super) fn numbering_name(numbering: Numbering) -> &'static str {
    match numbering {
        Numbering::Main => "main",
        Numbering::OrderPreservingOnEveryNumber => {
            "alternative_integers_order_preserving_on_every_number"
        }
        Numbering::OrderPreservingWithFixedGrowthStep => {
            "alternative_integers_with_fixed_growth_step"
        }
    }
}

/// 五个形状在某种写法下的整数。
pub(super) fn shapes_in(numbering: Numbering) -> Vec<Shape> {
    let replaced = |table: Table| -> Table {
        let mut replaced_table = table;
        for disk in 0..DISK_COUNT {
            replaced_table[disk] = alternative_number(disk, table[disk]);
        }
        replaced_table
    };
    registered_shapes()
        .into_iter()
        .map(|shape| match numbering {
            Numbering::Main => shape,
            Numbering::OrderPreservingOnEveryNumber
            | Numbering::OrderPreservingWithFixedGrowthStep => Shape {
                family: shape.family,
                early: replaced(shape.early),
                front: replaced(shape.front),
                back: replaced(shape.back),
            },
        })
        .collect()
}

/// 用户动作「再变大」之后这块盘的探测值，输入是这块盘此刻的整数（后尺寸，已按写法换过数）。
pub(super) fn grown_size(numbering: Numbering, disk: usize, size: u64) -> u64 {
    match numbering {
        Numbering::Main | Numbering::OrderPreservingWithFixedGrowthStep => size + GROW_STEP_SLOTS,
        Numbering::OrderPreservingOnEveryNumber => alternative_number(
            disk,
            main_number_of_alternative(disk, size) + GROW_STEP_SLOTS,
        ),
    }
}

/// 点位：「满」= 已用落点上界取前尺寸（终点已用取后尺寸），「少」= 单元只在最早那一段（取早尺寸，只在族 D）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Footprint {
    Full,
    Few,
}

pub(super) fn landing_of(footprint: Footprint, is_end_of_use: bool, shape: &Shape) -> Table {
    match (footprint, is_end_of_use) {
        (Footprint::Full, true) => shape.back,
        (Footprint::Full, false) => shape.front,
        (Footprint::Few, _) => shape.early,
    }
}

/// 断电之后用户的动作。动作之后每块盘的大小就是探测值。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum UserAction {
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

pub(super) fn actions_of(family: Family) -> Vec<UserAction> {
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
pub(super) fn board_and_probe_after_user_action(
    action: UserAction,
    shape: &Shape,
    board_at_power_loss: &Board,
    numbering: Numbering,
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
            for (disk, probe_size) in probe_sizes.iter_mut().enumerate().take(DISK_COUNT) {
                *probe_size = grown_size(numbering, disk, *probe_size);
            }
        }
        UserAction::GrowOnlyTheChangingDevice => {
            probe_sizes[DEVICE_THAT_CHANGES_ALONE] = grown_size(
                numbering,
                DEVICE_THAT_CHANGES_ALONE,
                probe_sizes[DEVICE_THAT_CHANGES_ALONE],
            )
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

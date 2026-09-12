//! E141：切换预留的挂载准入自证 —— C126（切换预留的最坏量没有口径） 欠的判别力自证。
//!
//! 预注册 `research/prompts/e141-preregistration.md`。被测对象逐字取自 D28（挂载期承诺量） 已定项 1 与已定项 3：
//! 一次切换的最坏量 = 副本数 × 32768 × max(1, ⌈(rows0 + N_switch) / 每片行数⌉) 字节，
//! 每片行数 = ⌊(32768 − 135) / 行宽⌋ − 1，预留 = N_switch × 一次切换的最坏量；两份副本各落一块盘，按设备算。
//! 挂载判定（D2（RAID 条带策略） 已定项 13「实例切换的预留拿得到」）：每块盘上
//! 容量 − 已分配 − 不可回收 − defer 待释放 ≥ 这块盘的份额。纯算术，确定性。

use e7_index_bench::Emitter;

/// 实例表单元的字节数：码 3 单元（D18（块里携带什么信息） 已定项 11）。
const INSTANCE_TABLE_UNIT_BYTES: u64 = 32768;
/// 码 3 头，含 nonce / MAC 预留位（D18（块里携带什么信息） 已定项 11 / 14）。
const PACKED_RECORD_HEADER_BYTES: u64 = 135;
/// 块：16 KiB。
const BLOCK_BYTES: u64 = 16384;
/// 实例表单元的副本数：根记录里实例表单元指针的两个位置条目（D22（单元原子性怎么合成） 已定项 7）。
const INSTANCE_TABLE_COPIES: u64 = 2;
/// 一次挂载允许的实例切换次数 N_switch（D23（journal 的角色与格式） 已定项 14）。
const SWITCHES_PER_MOUNT: u64 = 3;
/// 链指针还是 59 宽时的实例表行宽（2026-09-13 之前 D18（块里携带什么信息） 已定项 11 的值）。
const ROW_BYTES_WITH_59_WIDE_POINTER: u64 = 64;
/// 2026-09-13 起的实例表行宽：链指针记录 1 + 1 + 83 + 3（C304（实例表链指针记录装不下 83 宽的指针） 收口）。
const ROW_BYTES_WITH_83_WIDE_POINTER: u64 = 88;
/// 第一个可运行目标跑两块盘（D2（RAID 条带策略） 已定项 9）。
const DEVICE_COUNT: usize = 2;
const DEVICE_CAPACITY_BLOCKS: u64 = 20000;
const UNRECLAIMABLE_BLOCKS_PER_DEVICE: u64 = 7;
const DEFERRED_RELEASE_BLOCKS_PER_DEVICE: u64 = 11;
/// 判据 1 的池里另一块盘比份额多出来的空闲。
const SLACK_BLOCKS_ON_OTHER_DEVICE: u64 = 100;
/// 判据 3 的池里盘 0 比份额多出来的空闲。
const DEVICE_DIMENSION_SLACK_BLOCKS: u64 = 50;
const ROWS_AT_MOUNT_SAMPLES: [u64; 7] = [0, 366, 367, 505, 506, 1014, 4064];
const ROW_BYTES_SAMPLES: [u64; 2] = [ROW_BYTES_WITH_59_WIDE_POINTER, ROW_BYTES_WITH_83_WIDE_POINTER];
/// 判据 4 的手算表（跑前登记里写死）：(rows0, 行宽, 总预留的 16 KiB 块数)。
const HAND_COMPUTED_TOTAL_BLOCKS: [(u64, u64, u64); 14] = [
    (0, 64, 12), (366, 64, 12), (367, 64, 12), (505, 64, 12), (506, 64, 24), (1014, 64, 36), (4064, 64, 108),
    (0, 88, 12), (366, 88, 12), (367, 88, 24), (505, 88, 24), (506, 88, 24), (1014, 88, 36), (4064, 88, 144),
];

/// 一片实例表单元装几条 64 字节级的记录（含链指针记录）。
fn records_per_page(row_bytes: u64) -> u64 {
    (INSTANCE_TABLE_UNIT_BYTES - PACKED_RECORD_HEADER_BYTES) / row_bytes
}

/// 一片装几行数据：链指针记录恒为一片的最后一条，每片减一条。
fn rows_per_page(row_bytes: u64) -> u64 {
    records_per_page(row_bytes) - 1
}

/// 装下这么多行要几片；mkfs 写出的空表也占一片。
fn pages_for_rows(rows: u64, rows_per_page: u64) -> u64 {
    rows.div_ceil(rows_per_page).max(1)
}

fn unit_blocks() -> u64 {
    INSTANCE_TABLE_UNIT_BYTES / BLOCK_BYTES
}

/// D28（挂载期承诺量） 已定项 3：每块盘的份额（16 KiB 块）= N_switch × 32768 × 片数(rows0 + N_switch) / 16384。
fn reserve_share_per_device_blocks(rows_at_mount: u64, row_bytes: u64) -> u64 {
    let pages = pages_for_rows(rows_at_mount + SWITCHES_PER_MOUNT, rows_per_page(row_bytes));
    SWITCHES_PER_MOUNT * pages * unit_blocks()
}

fn reserve_total_blocks(rows_at_mount: u64, row_bytes: u64) -> u64 {
    reserve_share_per_device_blocks(rows_at_mount, row_bytes) * INSTANCE_TABLE_COPIES
}

/// 判据 5 的错法：取样点要分得开它们与 D28（挂载期承诺量） 已定项 3 的式子。
#[derive(Clone, Copy, Debug)]
enum WrongForm {
    /// C126 第一轮材料的甲：逐次累加、+1、每片 509。
    FirstRoundCandidate,
    /// 每片不减链指针记录。
    NoChainPointerPerPage,
    /// 行宽不随 C304 重算，恒 64。
    FixedRowWidth,
    /// 逐次累加代替 N_switch × 最坏一次。
    SummedInsteadOfWorstTimesSwitches,
}

const WRONG_FORMS: [WrongForm; 4] = [
    WrongForm::FirstRoundCandidate,
    WrongForm::NoChainPointerPerPage,
    WrongForm::FixedRowWidth,
    WrongForm::SummedInsteadOfWorstTimesSwitches,
];

impl WrongForm {
    fn name(self) -> &'static str {
        match self {
            WrongForm::FirstRoundCandidate => "first_round_candidate",
            WrongForm::NoChainPointerPerPage => "no_chain_pointer_per_page",
            WrongForm::FixedRowWidth => "fixed_row_width",
            WrongForm::SummedInsteadOfWorstTimesSwitches => "summed_instead_of_worst_times_switches",
        }
    }

    fn total_blocks(self, rows_at_mount: u64, row_bytes: u64) -> u64 {
        match self {
            WrongForm::FirstRoundCandidate => (1..=SWITCHES_PER_MOUNT)
                .map(|switch_number| INSTANCE_TABLE_COPIES * unit_blocks() * (rows_at_mount + switch_number + 1).div_ceil(records_per_page(ROW_BYTES_WITH_59_WIDE_POINTER)))
                .sum(),
            WrongForm::NoChainPointerPerPage => {
                let pages = pages_for_rows(rows_at_mount + SWITCHES_PER_MOUNT, records_per_page(row_bytes));
                SWITCHES_PER_MOUNT * pages * unit_blocks() * INSTANCE_TABLE_COPIES
            }
            WrongForm::FixedRowWidth => reserve_total_blocks(rows_at_mount, ROW_BYTES_WITH_59_WIDE_POINTER),
            WrongForm::SummedInsteadOfWorstTimesSwitches => (1..=SWITCHES_PER_MOUNT)
                .map(|switch_number| INSTANCE_TABLE_COPIES * unit_blocks() * pages_for_rows(rows_at_mount + switch_number, rows_per_page(row_bytes)))
                .sum(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MountDecision {
    Writable,
    ReadOnly,
}

impl MountDecision {
    fn name(self) -> &'static str {
        match self {
            MountDecision::Writable => "writable",
            MountDecision::ReadOnly => "read_only",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ReserveArm {
    /// 被测：D28（挂载期承诺量） 已定项 3，按设备。
    SettledPerDevice,
    /// 阳性对照：预留强制为 0。
    PositiveControlZeroReserve,
    /// 对照：两块盘的空闲加起来比总预留。
    PoolScalar,
}

#[derive(Clone, Copy, Debug)]
struct DeviceState {
    capacity_blocks: u64,
    allocated_blocks: u64,
    unreclaimable_blocks: u64,
    deferred_release_blocks: u64,
}

impl DeviceState {
    /// D28（挂载期承诺量） 已定项 1 按设备求和那四项：容量 − 已分配 − 不可回收 − defer 待释放。
    fn free_before_mount_commitment_blocks(&self) -> u64 {
        self.capacity_blocks - self.allocated_blocks - self.unreclaimable_blocks - self.deferred_release_blocks
    }
}

fn device_with_free(free_blocks: u64) -> DeviceState {
    let allocated_blocks = DEVICE_CAPACITY_BLOCKS - free_blocks - UNRECLAIMABLE_BLOCKS_PER_DEVICE - DEFERRED_RELEASE_BLOCKS_PER_DEVICE;
    DeviceState {
        capacity_blocks: DEVICE_CAPACITY_BLOCKS,
        allocated_blocks,
        unreclaimable_blocks: UNRECLAIMABLE_BLOCKS_PER_DEVICE,
        deferred_release_blocks: DEFERRED_RELEASE_BLOCKS_PER_DEVICE,
    }
}

fn pool_with_frees(device_zero_free_blocks: u64, device_one_free_blocks: u64) -> [DeviceState; DEVICE_COUNT] {
    [device_with_free(device_zero_free_blocks), device_with_free(device_one_free_blocks)]
}

/// 紧的那块盘空闲 = tight_free_blocks，另一块多 SLACK_BLOCKS_ON_OTHER_DEVICE。
fn pool_with_tight_device(tight_device: usize, tight_free_blocks: u64, share_blocks: u64) -> [DeviceState; DEVICE_COUNT] {
    let other_free_blocks = share_blocks + SLACK_BLOCKS_ON_OTHER_DEVICE;
    match tight_device {
        0 => pool_with_frees(tight_free_blocks, other_free_blocks),
        1 => pool_with_frees(other_free_blocks, tight_free_blocks),
        other => panic!("只有两块盘，紧盘编号 {other} 不存在"),
    }
}

fn mount_decision(arm: ReserveArm, devices: &[DeviceState; DEVICE_COUNT], rows_at_mount: u64, row_bytes: u64) -> MountDecision {
    let share_per_device = match arm {
        ReserveArm::SettledPerDevice | ReserveArm::PoolScalar => reserve_share_per_device_blocks(rows_at_mount, row_bytes),
        ReserveArm::PositiveControlZeroReserve => 0,
    };
    let is_reserve_obtainable = match arm {
        ReserveArm::SettledPerDevice | ReserveArm::PositiveControlZeroReserve => {
            devices.iter().all(|device| device.free_before_mount_commitment_blocks() >= share_per_device)
        }
        ReserveArm::PoolScalar => {
            devices.iter().map(DeviceState::free_before_mount_commitment_blocks).sum::<u64>() >= share_per_device * INSTANCE_TABLE_COPIES
        }
    };
    if is_reserve_obtainable {
        MountDecision::Writable
    } else {
        MountDecision::ReadOnly
    }
}

fn sampling_points() -> Vec<(u64, u64)> {
    let mut points = Vec::new();
    for row_bytes in ROW_BYTES_SAMPLES {
        for rows_at_mount in ROWS_AT_MOUNT_SAMPLES {
            points.push((rows_at_mount, row_bytes));
        }
    }
    points
}

fn hand_computed_total_blocks(rows_at_mount: u64, row_bytes: u64) -> u64 {
    HAND_COMPUTED_TOTAL_BLOCKS
        .iter()
        .find(|(rows, width, _)| *rows == rows_at_mount && *width == row_bytes)
        .map(|(_, _, total)| *total)
        .expect("手算表覆盖全部 14 个取样点")
}

fn main() {
    let mut emitter = Emitter::new();
    let mut criterion_one_bad_pairs = 0u64;
    let mut criterion_two_control_failures = 0u64;
    let mut criterion_three_settled_writable = 0u64;
    let mut criterion_three_scalar_read_only = 0u64;
    let mut criterion_four_mismatches = 0u64;

    for (rows_at_mount, row_bytes) in sampling_points() {
        let share = reserve_share_per_device_blocks(rows_at_mount, row_bytes);
        let total = reserve_total_blocks(rows_at_mount, row_bytes);
        let expected_total = hand_computed_total_blocks(rows_at_mount, row_bytes);
        if total != expected_total || share * INSTANCE_TABLE_COPIES != total {
            criterion_four_mismatches += 1;
        }
        println!("{}", emitter.emit_raw(&format!(
            "name=reserve rows_at_mount={rows_at_mount} row_bytes={row_bytes} records_per_page={} rows_per_page={} pages={} total_blocks={total} share_per_device_blocks={share} expected_total_blocks={expected_total}",
            records_per_page(row_bytes), rows_per_page(row_bytes), pages_for_rows(rows_at_mount + SWITCHES_PER_MOUNT, rows_per_page(row_bytes))
        )));

        for tight_device in 0..DEVICE_COUNT {
            let at_share = pool_with_tight_device(tight_device, share, share);
            let below_share = pool_with_tight_device(tight_device, share - 1, share);
            let settled_at_share = mount_decision(ReserveArm::SettledPerDevice, &at_share, rows_at_mount, row_bytes);
            let settled_below_share = mount_decision(ReserveArm::SettledPerDevice, &below_share, rows_at_mount, row_bytes);
            let control_at_share = mount_decision(ReserveArm::PositiveControlZeroReserve, &at_share, rows_at_mount, row_bytes);
            let control_below_share = mount_decision(ReserveArm::PositiveControlZeroReserve, &below_share, rows_at_mount, row_bytes);
            if settled_at_share != MountDecision::Writable || settled_below_share != MountDecision::ReadOnly {
                criterion_one_bad_pairs += 1;
            }
            if control_at_share != MountDecision::Writable || control_below_share != MountDecision::Writable {
                criterion_two_control_failures += 1;
            }
            println!("{}", emitter.emit_raw(&format!(
                "name=flip rows_at_mount={rows_at_mount} row_bytes={row_bytes} tight_device={tight_device} share_per_device_blocks={share} at_share={} below_share={} control_at_share={} control_below_share={}",
                settled_at_share.name(), settled_below_share.name(), control_at_share.name(), control_below_share.name()
            )));
        }

        let one_device_short = pool_with_frees(share + DEVICE_DIMENSION_SLACK_BLOCKS, share - 1);
        let settled = mount_decision(ReserveArm::SettledPerDevice, &one_device_short, rows_at_mount, row_bytes);
        let scalar = mount_decision(ReserveArm::PoolScalar, &one_device_short, rows_at_mount, row_bytes);
        if settled == MountDecision::Writable {
            criterion_three_settled_writable += 1;
        }
        if scalar == MountDecision::ReadOnly {
            criterion_three_scalar_read_only += 1;
        }
        println!("{}", emitter.emit_raw(&format!(
            "name=device_dimension rows_at_mount={rows_at_mount} row_bytes={row_bytes} device_zero_free={} device_one_free={} settled={} pool_scalar={}",
            share + DEVICE_DIMENSION_SLACK_BLOCKS, share - 1, settled.name(), scalar.name()
        )));
    }

    let mut criterion_five_blind_forms = 0u64;
    for form in WRONG_FORMS {
        let mut agreeing_points = 0u64;
        let mut first_difference = String::from("none");
        for (rows_at_mount, row_bytes) in sampling_points() {
            let settled_total = reserve_total_blocks(rows_at_mount, row_bytes);
            let wrong_total = form.total_blocks(rows_at_mount, row_bytes);
            if settled_total == wrong_total {
                agreeing_points += 1;
            } else if first_difference == "none" {
                first_difference = format!("{rows_at_mount}:{row_bytes}:settled={settled_total}:wrong={wrong_total}");
            }
        }
        let point_count = sampling_points().len() as u64;
        if agreeing_points == point_count {
            criterion_five_blind_forms += 1;
        }
        println!("{}", emitter.emit_raw(&format!(
            "name=sensitivity form={} points={point_count} agreeing_points={agreeing_points} first_difference={first_difference}",
            form.name()
        )));
    }

    println!("{}", emitter.emit_raw(&format!(
        "name=verdict crit1_bad_pairs={criterion_one_bad_pairs} crit2_control_failures={criterion_two_control_failures} crit3_settled_writable={criterion_three_settled_writable} crit3_scalar_read_only={criterion_three_scalar_read_only} crit4_mismatches={criterion_four_mismatches} crit5_blind_forms={criterion_five_blind_forms}"
    )));
    println!("{}", emitter.finish());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_per_page_is_508_at_row_width_64_and_369_at_row_width_88() {
        assert_eq!(records_per_page(64), 509, "⌊(32768 − 135) / 64⌋");
        assert_eq!(rows_per_page(64), 508, "链指针记录恒为一片的最后一条，每片减一");
        assert_eq!(records_per_page(88), 370, "⌊(32768 − 135) / 88⌋");
        assert_eq!(rows_per_page(88), 369);
    }

    #[test]
    fn empty_table_still_occupies_one_page() {
        assert_eq!(pages_for_rows(0, 508), 1, "mkfs 写出一片空表：零行也占一片");
        assert_eq!(pages_for_rows(508, 508), 1);
        assert_eq!(pages_for_rows(509, 508), 2);
    }

    #[test]
    fn first_transaction_geometry_reserve_is_twelve_blocks_six_per_device() {
        assert_eq!(reserve_total_blocks(0, 64), 12, "3 × 2 × 32768 字节 = 12 个 16 KiB 块");
        assert_eq!(reserve_share_per_device_blocks(0, 64), 6);
    }

    #[test]
    fn reserve_totals_match_hand_computed_table() {
        for (rows_at_mount, row_bytes, expected_total) in HAND_COMPUTED_TOTAL_BLOCKS {
            assert_eq!(reserve_total_blocks(rows_at_mount, row_bytes), expected_total, "rows0={rows_at_mount} 行宽={row_bytes}");
            assert_eq!(reserve_share_per_device_blocks(rows_at_mount, row_bytes) * 2, expected_total, "每块盘恰好一半");
        }
        assert_eq!(reserve_total_blocks(4064, 64), 108, "C126 行里点名要钉的那一格");
        assert_eq!(reserve_total_blocks(4064, 88), 144);
    }

    #[test]
    fn settled_arm_flips_exactly_at_the_share_on_either_device() {
        for (rows_at_mount, row_bytes) in sampling_points() {
            let share = reserve_share_per_device_blocks(rows_at_mount, row_bytes);
            for tight_device in 0..DEVICE_COUNT {
                let at_share = pool_with_tight_device(tight_device, share, share);
                let below_share = pool_with_tight_device(tight_device, share - 1, share);
                assert_eq!(mount_decision(ReserveArm::SettledPerDevice, &at_share, rows_at_mount, row_bytes), MountDecision::Writable,
                    "rows0={rows_at_mount} 行宽={row_bytes} 紧盘={tight_device}：空闲 = 份额要挂得成可写");
                assert_eq!(mount_decision(ReserveArm::SettledPerDevice, &below_share, rows_at_mount, row_bytes), MountDecision::ReadOnly,
                    "rows0={rows_at_mount} 行宽={row_bytes} 紧盘={tight_device}：空闲 = 份额 − 1 要只读");
            }
        }
    }

    #[test]
    fn zero_reserve_control_mounts_both_pools_writable() {
        for (rows_at_mount, row_bytes) in sampling_points() {
            let share = reserve_share_per_device_blocks(rows_at_mount, row_bytes);
            for tight_device in 0..DEVICE_COUNT {
                let below_share = pool_with_tight_device(tight_device, share - 1, share);
                assert_eq!(mount_decision(ReserveArm::PositiveControlZeroReserve, &below_share, rows_at_mount, row_bytes), MountDecision::Writable,
                    "预留置 0 时少 1 块的池也该可写，否则装置不是在测预留");
            }
        }
    }

    #[test]
    fn pool_scalar_misses_the_short_device_that_settled_arm_catches() {
        for (rows_at_mount, row_bytes) in sampling_points() {
            let share = reserve_share_per_device_blocks(rows_at_mount, row_bytes);
            let one_device_short = pool_with_frees(share + DEVICE_DIMENSION_SLACK_BLOCKS, share - 1);
            assert_eq!(mount_decision(ReserveArm::SettledPerDevice, &one_device_short, rows_at_mount, row_bytes), MountDecision::ReadOnly);
            assert_eq!(mount_decision(ReserveArm::PoolScalar, &one_device_short, rows_at_mount, row_bytes), MountDecision::Writable);
        }
    }

    #[test]
    fn every_wrong_form_disagrees_at_a_sampling_point() {
        assert_eq!(WrongForm::FirstRoundCandidate.total_blocks(506, 64), 16, "4 × (1 + 1 + 2)");
        assert_eq!(WrongForm::NoChainPointerPerPage.total_blocks(506, 64), 12, "每片 509 时 509 行仍是一片");
        assert_eq!(WrongForm::FixedRowWidth.total_blocks(367, 88), 12, "行宽恒 64 时 370 行是一片");
        assert_eq!(WrongForm::SummedInsteadOfWorstTimesSwitches.total_blocks(506, 64), 16, "4 × (1 + 1 + 2)");
        for form in WRONG_FORMS {
            assert_eq!(form.total_blocks(0, 64), 12, "rows0 = 0 那一格分不开任何错法（{}）", form.name());
            let differs_somewhere = sampling_points().into_iter().any(|(rows_at_mount, row_bytes)| form.total_blocks(rows_at_mount, row_bytes) != reserve_total_blocks(rows_at_mount, row_bytes));
            assert!(differs_somewhere, "取样点分不开 {}", form.name());
        }
    }
}

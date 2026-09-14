//! E141：切换预留的挂载准入自证 —— C126（切换预留的最坏量没有口径） 欠的判别力自证。
//!
//! 预注册 `research/prompts/e141-preregistration.md`。被测对象逐字取自 D28（挂载期承诺量） 已定项 1 与已定项 3：
//! 一次切换的最坏量 = 副本数 × 32768 × max(1, ⌈(rows0 + N_switch) / 每片行数⌉) 字节，
//! 每片行数 = ⌊(32768 − 135) / 行宽⌋ − 1，预留 = (N_switch + 1) × 一次切换的最坏量；两份副本各落一块盘，按设备算。
//! 挂载判定（D2（RAID 条带策略） 已定项 13「实例切换的预留拿得到」）：每块盘上
//! 容量 − 已分配 − 不可回收 − defer 待释放 ≥ 这块盘的份额。纯算术，确定性。
//!
//! 2026-09-13 加暖机那一半（D16（发布语义） 已定项 8 + D28（挂载期承诺量） 已定项 3 的连带）：一次切换还要加
//! 「至多 R = 3 次空发布 × 现算的 c_max」，c_max 按 D28（挂载期承诺量） 已定项 4 每次发布现算、装置把它当输入扫两档
//! （E148（提交固定点按两棵记录树重算）：第一个事务规模 4 块、池规模 9 块）。第一个可运行目标两块盘互为镜像
//! （E142（第一个事务的干跑） 每个单元写到每块盘），所以空发布的 c_max 块每块盘各占一份，份额按设备加。
//!
//! 2026-09-14 预留多一份（C329（写行那次发布之前推抬 F 的空发布没有检查） 三轮三方后用户定案）：每次可写挂载都写行，
//! 写行那次是新实例的第一次发布、不推抬 F 的空发布，它的元数据走切换预留 ⇒ 份数 = N_switch + 1，链重写与暖机两半都乘这个份数；
//! 片数仍按 rows0 + N_switch 算（rows0 从这一天起含写行那次要写的行）。

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
/// 写行那次发布那一份（C329（写行那次发布之前推抬 F 的空发布没有检查） 2026-09-14 用户定案）。
const ROW_WRITING_PUBLISH_SHARES: u64 = 1;
/// 预留的份数 = N_switch + 写行那次发布那一份。
const RESERVED_SHARES_PER_MOUNT: u64 = SWITCHES_PER_MOUNT + ROW_WRITING_PUBLISH_SHARES;
/// 每次切换至多推几次空发布（暖机，D16（发布语义） 已定项 8：第一版几何至多 R = 3）。
const EMPTY_PUBLISHES_PER_SWITCH: u64 = 3;
/// c_max 的两档取样：E148（提交固定点按两棵记录树重算） 第一个事务规模 4 块、池规模 9 块；它是现算的量，装置当输入扫。
const CHECKPOINT_COST_SAMPLES: [u64; 2] = [4, 9];
/// 暖机份额的手算表（跑前写死；2026-09-14 第三次跑之前按 N_switch + 1 份重算）：(c_max, 每块盘的暖机块数 = (N_switch + 1) × R × c_max)。
const HAND_COMPUTED_WARM_UP_SHARE_BLOCKS: [(u64, u64); 2] = [(4, 48), (9, 108)];
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
/// 判据 4 的手算表（跑前登记里写死；2026-09-14 第三次跑之前按 N_switch + 1 份重算，片数不变）：(rows0, 行宽, 总预留的 16 KiB 块数)。
const HAND_COMPUTED_TOTAL_BLOCKS: [(u64, u64, u64); 14] = [
    (0, 64, 16), (366, 64, 16), (367, 64, 16), (505, 64, 16), (506, 64, 32), (1014, 64, 48), (4064, 64, 144),
    (0, 88, 16), (366, 88, 16), (367, 88, 32), (505, 88, 32), (506, 88, 32), (1014, 88, 48), (4064, 88, 192),
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

/// D28（挂载期承诺量） 已定项 3 链重写那一半：每块盘的份额（16 KiB 块）= (N_switch + 1) × 32768 × 片数(rows0 + N_switch) / 16384。
fn chain_rewrite_share_per_device_blocks(rows_at_mount: u64, row_bytes: u64) -> u64 {
    let pages = pages_for_rows(rows_at_mount + SWITCHES_PER_MOUNT, rows_per_page(row_bytes));
    RESERVED_SHARES_PER_MOUNT * pages * unit_blocks()
}

/// 暖机那一半（D16（发布语义） 已定项 8）：每块盘 (N_switch + 1) × R × c_max 块，镜像两块盘各占一份。
fn warm_up_share_per_device_blocks(checkpoint_cost_blocks: u64) -> u64 {
    RESERVED_SHARES_PER_MOUNT * EMPTY_PUBLISHES_PER_SWITCH * checkpoint_cost_blocks
}

/// 每块盘的切换预留份额 = 链重写 + 暖机。
fn reserve_share_per_device_blocks(rows_at_mount: u64, row_bytes: u64, checkpoint_cost_blocks: u64) -> u64 {
    chain_rewrite_share_per_device_blocks(rows_at_mount, row_bytes) + warm_up_share_per_device_blocks(checkpoint_cost_blocks)
}

fn chain_rewrite_total_blocks(rows_at_mount: u64, row_bytes: u64) -> u64 {
    chain_rewrite_share_per_device_blocks(rows_at_mount, row_bytes) * INSTANCE_TABLE_COPIES
}

fn reserve_total_blocks(rows_at_mount: u64, row_bytes: u64, checkpoint_cost_blocks: u64) -> u64 {
    reserve_share_per_device_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks) * INSTANCE_TABLE_COPIES
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
    /// 漏掉暖机那一半（2026-09-13 之前的式子）。
    NoWarmUp,
    /// 暖机只按池级一份算，不按镜像每块盘各一份。
    WarmUpPoolLevelOnly,
    /// 漏掉写行那次发布那一份：只按 N_switch 份算（2026-09-14 之前的式子）。
    NoRowWritingPublishShare,
}

const WRONG_FORMS: [WrongForm; 7] = [
    WrongForm::FirstRoundCandidate,
    WrongForm::NoChainPointerPerPage,
    WrongForm::FixedRowWidth,
    WrongForm::SummedInsteadOfWorstTimesSwitches,
    WrongForm::NoWarmUp,
    WrongForm::WarmUpPoolLevelOnly,
    WrongForm::NoRowWritingPublishShare,
];

impl WrongForm {
    fn name(self) -> &'static str {
        match self {
            WrongForm::FirstRoundCandidate => "first_round_candidate",
            WrongForm::NoChainPointerPerPage => "no_chain_pointer_per_page",
            WrongForm::FixedRowWidth => "fixed_row_width",
            WrongForm::SummedInsteadOfWorstTimesSwitches => "summed_instead_of_worst_times_switches",
            WrongForm::NoWarmUp => "no_warm_up",
            WrongForm::WarmUpPoolLevelOnly => "warm_up_pool_level_only",
            WrongForm::NoRowWritingPublishShare => "no_row_writing_publish_share",
        }
    }

    /// 错法的总预留；前四种只错链重写那一半（份数与被测相同，逐次累加的从写行那次那一份算起）、暖机照对，第五、六种只错暖机那一半，第七种两半都少一份。
    fn total_blocks(self, rows_at_mount: u64, row_bytes: u64, checkpoint_cost_blocks: u64) -> u64 {
        let warm_up_total = warm_up_share_per_device_blocks(checkpoint_cost_blocks) * INSTANCE_TABLE_COPIES;
        match self {
            WrongForm::FirstRoundCandidate => (0..RESERVED_SHARES_PER_MOUNT)
                .map(|share_index| INSTANCE_TABLE_COPIES * unit_blocks() * (rows_at_mount + share_index + 1).div_ceil(records_per_page(ROW_BYTES_WITH_59_WIDE_POINTER)))
                .sum::<u64>() + warm_up_total,
            WrongForm::NoChainPointerPerPage => {
                let pages = pages_for_rows(rows_at_mount + SWITCHES_PER_MOUNT, records_per_page(row_bytes));
                RESERVED_SHARES_PER_MOUNT * pages * unit_blocks() * INSTANCE_TABLE_COPIES + warm_up_total
            }
            WrongForm::FixedRowWidth => chain_rewrite_total_blocks(rows_at_mount, ROW_BYTES_WITH_59_WIDE_POINTER) + warm_up_total,
            WrongForm::SummedInsteadOfWorstTimesSwitches => (0..RESERVED_SHARES_PER_MOUNT)
                .map(|share_index| INSTANCE_TABLE_COPIES * unit_blocks() * pages_for_rows(rows_at_mount + share_index, rows_per_page(row_bytes)))
                .sum::<u64>() + warm_up_total,
            WrongForm::NoWarmUp => chain_rewrite_total_blocks(rows_at_mount, row_bytes),
            WrongForm::WarmUpPoolLevelOnly => chain_rewrite_total_blocks(rows_at_mount, row_bytes) + warm_up_share_per_device_blocks(checkpoint_cost_blocks),
            WrongForm::NoRowWritingPublishShare => {
                let pages = pages_for_rows(rows_at_mount + SWITCHES_PER_MOUNT, rows_per_page(row_bytes));
                (SWITCHES_PER_MOUNT * pages * unit_blocks() + SWITCHES_PER_MOUNT * EMPTY_PUBLISHES_PER_SWITCH * checkpoint_cost_blocks) * INSTANCE_TABLE_COPIES
            }
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

fn mount_decision(arm: ReserveArm, devices: &[DeviceState; DEVICE_COUNT], rows_at_mount: u64, row_bytes: u64, checkpoint_cost_blocks: u64) -> MountDecision {
    let share_per_device = match arm {
        ReserveArm::SettledPerDevice | ReserveArm::PoolScalar => reserve_share_per_device_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks),
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

fn sampling_points() -> Vec<(u64, u64, u64)> {
    let mut points = Vec::new();
    for checkpoint_cost_blocks in CHECKPOINT_COST_SAMPLES {
        for row_bytes in ROW_BYTES_SAMPLES {
            for rows_at_mount in ROWS_AT_MOUNT_SAMPLES {
                points.push((rows_at_mount, row_bytes, checkpoint_cost_blocks));
            }
        }
    }
    points
}

fn hand_computed_warm_up_share_blocks(checkpoint_cost_blocks: u64) -> u64 {
    HAND_COMPUTED_WARM_UP_SHARE_BLOCKS
        .iter()
        .find(|(cost, _)| *cost == checkpoint_cost_blocks)
        .map(|(_, share)| *share)
        .expect("手算表覆盖两档 c_max")
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

    for (rows_at_mount, row_bytes, checkpoint_cost_blocks) in sampling_points() {
        let chain_share = chain_rewrite_share_per_device_blocks(rows_at_mount, row_bytes);
        let warm_up_share = warm_up_share_per_device_blocks(checkpoint_cost_blocks);
        let share = reserve_share_per_device_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks);
        let total = reserve_total_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks);
        let expected_total = hand_computed_total_blocks(rows_at_mount, row_bytes) + hand_computed_warm_up_share_blocks(checkpoint_cost_blocks) * INSTANCE_TABLE_COPIES;
        if total != expected_total || share * INSTANCE_TABLE_COPIES != total || share != chain_share + warm_up_share {
            criterion_four_mismatches += 1;
        }
        println!("{}", emitter.emit_raw(&format!(
            "name=reserve rows_at_mount={rows_at_mount} row_bytes={row_bytes} checkpoint_cost_blocks={checkpoint_cost_blocks} reserved_shares={RESERVED_SHARES_PER_MOUNT} records_per_page={} rows_per_page={} pages={} chain_share_per_device_blocks={chain_share} warm_up_share_per_device_blocks={warm_up_share} total_blocks={total} share_per_device_blocks={share} expected_total_blocks={expected_total}",
            records_per_page(row_bytes), rows_per_page(row_bytes), pages_for_rows(rows_at_mount + SWITCHES_PER_MOUNT, rows_per_page(row_bytes))
        )));

        for tight_device in 0..DEVICE_COUNT {
            let at_share = pool_with_tight_device(tight_device, share, share);
            let below_share = pool_with_tight_device(tight_device, share - 1, share);
            let settled_at_share = mount_decision(ReserveArm::SettledPerDevice, &at_share, rows_at_mount, row_bytes, checkpoint_cost_blocks);
            let settled_below_share = mount_decision(ReserveArm::SettledPerDevice, &below_share, rows_at_mount, row_bytes, checkpoint_cost_blocks);
            let control_at_share = mount_decision(ReserveArm::PositiveControlZeroReserve, &at_share, rows_at_mount, row_bytes, checkpoint_cost_blocks);
            let control_below_share = mount_decision(ReserveArm::PositiveControlZeroReserve, &below_share, rows_at_mount, row_bytes, checkpoint_cost_blocks);
            if settled_at_share != MountDecision::Writable || settled_below_share != MountDecision::ReadOnly {
                criterion_one_bad_pairs += 1;
            }
            if control_at_share != MountDecision::Writable || control_below_share != MountDecision::Writable {
                criterion_two_control_failures += 1;
            }
            println!("{}", emitter.emit_raw(&format!(
                "name=flip rows_at_mount={rows_at_mount} row_bytes={row_bytes} checkpoint_cost_blocks={checkpoint_cost_blocks} tight_device={tight_device} share_per_device_blocks={share} at_share={} below_share={} control_at_share={} control_below_share={}",
                settled_at_share.name(), settled_below_share.name(), control_at_share.name(), control_below_share.name()
            )));
        }

        let one_device_short = pool_with_frees(share + DEVICE_DIMENSION_SLACK_BLOCKS, share - 1);
        let settled = mount_decision(ReserveArm::SettledPerDevice, &one_device_short, rows_at_mount, row_bytes, checkpoint_cost_blocks);
        let scalar = mount_decision(ReserveArm::PoolScalar, &one_device_short, rows_at_mount, row_bytes, checkpoint_cost_blocks);
        if settled == MountDecision::Writable {
            criterion_three_settled_writable += 1;
        }
        if scalar == MountDecision::ReadOnly {
            criterion_three_scalar_read_only += 1;
        }
        println!("{}", emitter.emit_raw(&format!(
            "name=device_dimension rows_at_mount={rows_at_mount} row_bytes={row_bytes} checkpoint_cost_blocks={checkpoint_cost_blocks} device_zero_free={} device_one_free={} settled={} pool_scalar={}",
            share + DEVICE_DIMENSION_SLACK_BLOCKS, share - 1, settled.name(), scalar.name()
        )));
    }

    let mut criterion_five_blind_forms = 0u64;
    for form in WRONG_FORMS {
        let mut agreeing_points = 0u64;
        let mut first_difference = String::from("none");
        for (rows_at_mount, row_bytes, checkpoint_cost_blocks) in sampling_points() {
            let settled_total = reserve_total_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks);
            let wrong_total = form.total_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks);
            if settled_total == wrong_total {
                agreeing_points += 1;
            } else if first_difference == "none" {
                first_difference = format!("{rows_at_mount}:{row_bytes}:{checkpoint_cost_blocks}:settled={settled_total}:wrong={wrong_total}");
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
    fn first_transaction_geometry_chain_rewrite_is_sixteen_blocks_eight_per_device() {
        assert_eq!(chain_rewrite_total_blocks(0, 64), 16, "(3 + 1) × 2 × 32768 字节 = 16 个 16 KiB 块");
        assert_eq!(chain_rewrite_share_per_device_blocks(0, 64), 8);
    }

    /// 暖机那一半：(N_switch + 1) × R × c_max，两档 c_max 手算 48 / 108，每块盘各一份。
    #[test]
    fn warm_up_share_is_reserved_shares_times_empty_publishes_times_checkpoint_cost() {
        assert_eq!(warm_up_share_per_device_blocks(4), (3 + 1) * 3 * 4);
        assert_eq!(warm_up_share_per_device_blocks(9), 108, "D16 已定项 8 逐字「池规模 9 块 ⇒ 27 块」是一次切换的量，三次切换加写行那次一份共 108");
        for (checkpoint_cost_blocks, expected_share) in HAND_COMPUTED_WARM_UP_SHARE_BLOCKS {
            assert_eq!(warm_up_share_per_device_blocks(checkpoint_cost_blocks), expected_share);
        }
        assert_eq!(reserve_share_per_device_blocks(0, 88, 9), 8 + 108, "第一个事务几何、池规模 c_max：每块盘 116 块");
        assert_eq!(reserve_total_blocks(0, 88, 9), 232);
    }

    #[test]
    fn chain_rewrite_totals_match_hand_computed_table() {
        for (rows_at_mount, row_bytes, expected_total) in HAND_COMPUTED_TOTAL_BLOCKS {
            assert_eq!(chain_rewrite_total_blocks(rows_at_mount, row_bytes), expected_total, "rows0={rows_at_mount} 行宽={row_bytes}");
            assert_eq!(chain_rewrite_share_per_device_blocks(rows_at_mount, row_bytes) * 2, expected_total, "每块盘恰好一半");
        }
        assert_eq!(chain_rewrite_total_blocks(4064, 64), 144, "C126 行里点名要钉的那一格（N_switch 份时是 108）");
        assert_eq!(chain_rewrite_total_blocks(4064, 88), 192);
    }

    #[test]
    fn settled_arm_flips_exactly_at_the_share_on_either_device() {
        for (rows_at_mount, row_bytes, checkpoint_cost_blocks) in sampling_points() {
            let share = reserve_share_per_device_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks);
            for tight_device in 0..DEVICE_COUNT {
                let at_share = pool_with_tight_device(tight_device, share, share);
                let below_share = pool_with_tight_device(tight_device, share - 1, share);
                assert_eq!(mount_decision(ReserveArm::SettledPerDevice, &at_share, rows_at_mount, row_bytes, checkpoint_cost_blocks), MountDecision::Writable,
                    "rows0={rows_at_mount} 行宽={row_bytes} c_max={checkpoint_cost_blocks} 紧盘={tight_device}：空闲 = 份额要挂得成可写");
                assert_eq!(mount_decision(ReserveArm::SettledPerDevice, &below_share, rows_at_mount, row_bytes, checkpoint_cost_blocks), MountDecision::ReadOnly,
                    "rows0={rows_at_mount} 行宽={row_bytes} c_max={checkpoint_cost_blocks} 紧盘={tight_device}：空闲 = 份额 − 1 要只读");
            }
        }
    }

    #[test]
    fn zero_reserve_control_mounts_both_pools_writable() {
        for (rows_at_mount, row_bytes, checkpoint_cost_blocks) in sampling_points() {
            let share = reserve_share_per_device_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks);
            for tight_device in 0..DEVICE_COUNT {
                let below_share = pool_with_tight_device(tight_device, share - 1, share);
                assert_eq!(mount_decision(ReserveArm::PositiveControlZeroReserve, &below_share, rows_at_mount, row_bytes, checkpoint_cost_blocks), MountDecision::Writable,
                    "预留置 0 时少 1 块的池也该可写，否则装置不是在测预留");
            }
        }
    }

    #[test]
    fn pool_scalar_misses_the_short_device_that_settled_arm_catches() {
        for (rows_at_mount, row_bytes, checkpoint_cost_blocks) in sampling_points() {
            let share = reserve_share_per_device_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks);
            let one_device_short = pool_with_frees(share + DEVICE_DIMENSION_SLACK_BLOCKS, share - 1);
            assert_eq!(mount_decision(ReserveArm::SettledPerDevice, &one_device_short, rows_at_mount, row_bytes, checkpoint_cost_blocks), MountDecision::ReadOnly);
            assert_eq!(mount_decision(ReserveArm::PoolScalar, &one_device_short, rows_at_mount, row_bytes, checkpoint_cost_blocks), MountDecision::Writable);
        }
    }

    #[test]
    fn every_wrong_form_disagrees_at_a_sampling_point() {
        assert_eq!(WrongForm::FirstRoundCandidate.total_blocks(506, 64, 4), 20 + 96, "4 × (1 + 1 + 1 + 2) + 暖机 96");
        assert_eq!(WrongForm::NoChainPointerPerPage.total_blocks(506, 64, 4), 16 + 96, "每片 509 时 509 行仍是一片");
        assert_eq!(WrongForm::FixedRowWidth.total_blocks(367, 88, 4), 16 + 96, "行宽恒 64 时 370 行是一片");
        assert_eq!(WrongForm::SummedInsteadOfWorstTimesSwitches.total_blocks(506, 64, 4), 20 + 96, "4 × (1 + 1 + 1 + 2)");
        assert_eq!(WrongForm::NoWarmUp.total_blocks(0, 64, 9), 16, "漏暖机：只剩链重写 16");
        assert_eq!(WrongForm::WarmUpPoolLevelOnly.total_blocks(0, 64, 9), 16 + 108, "池级一份：少算一块盘的 108");
        assert_eq!(WrongForm::NoRowWritingPublishShare.total_blocks(0, 64, 4), 12 + 72, "漏写行那一份：2026-09-14 之前的 N_switch 份");
        for form in WRONG_FORMS {
            let differs_somewhere = sampling_points().into_iter().any(|(rows_at_mount, row_bytes, checkpoint_cost_blocks)| form.total_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks) != reserve_total_blocks(rows_at_mount, row_bytes, checkpoint_cost_blocks));
            assert!(differs_somewhere, "取样点分不开 {}", form.name());
        }
        for form in [WrongForm::FirstRoundCandidate, WrongForm::NoChainPointerPerPage, WrongForm::FixedRowWidth, WrongForm::SummedInsteadOfWorstTimesSwitches] {
            assert_eq!(form.total_blocks(0, 64, 4), 16 + 96, "rows0 = 0 那一格分不开链重写的任何错法（{}）", form.name());
        }
    }
}

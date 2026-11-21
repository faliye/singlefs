//! 本地常量、它们和 crates 的回比与锚点那一段。

use super::*;

/// 本地常量：槽 16 KiB（D3（空间分配） 已定项 10 ①：「段长 64 个 16 KiB 槽」）。
pub(super) const SLOT_BYTES_LOCAL: u64 = 16384;
/// 本地常量：聚簇段 64 槽（D3（空间分配） 已定项 10 ①）。
pub(super) const CLUSTER_SEGMENT_SLOTS_LOCAL: u64 = 64;
/// 本地常量：默认 768 MiB 环的单元区起始槽号 U（D3（空间分配） 已定项 10 ④：「默认 768 MiB 环是 50176」）。
const UNIT_AREA_START_SLOT_LOCAL: u64 = 50176;
/// 本地常量：默认 journal 环 768 MiB（D3（空间分配） 已定项 10 ④）。
pub(super) const DEFAULT_JOURNAL_RING_BYTES_LOCAL: u64 = 805_306_368;
/// 本地常量：mkfs 的每区槽数 S = 8（`.claude/kb/layout/01-first-txn.md`「每区槽数 S」那一行）。
const ROOT_RING_SLOTS_PER_REGION_LOCAL: u64 = 8;
/// 本地常量：分配记录树层级 0 每位置罩 812 槽，内部扇出 169（D28（挂载期承诺量） 已定项 4：S_L = 812 × 169^L）。
pub(super) const ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL: u64 = 812;
pub(super) const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL: u64 = 169;
/// 本地常量：mkfs 的实例表落 U、U + 1，树表第 0 版落 U + 2（D3（空间分配） 已定项 8 射程）。
const INSTANCE_TABLE_OFFSET_FROM_UNIT_AREA_START_LOCAL: u64 = 0;
const TREE_TABLE_OFFSET_FROM_UNIT_AREA_START_LOCAL: u64 = 2;
/// 本地常量：一个数据单元两槽（D3（空间分配） 已定项 8 第 1 条：起点 32768 对齐的两槽）。
pub(super) const DATA_UNIT_SLOTS_LOCAL: u64 = 2;
/// 一个数据单元的字节数：两槽。
pub(super) const DATA_UNIT_BYTES_LOCAL: usize = 32768;
/// 固定写入时间（只进 inode 记录，不影响落点与槽数）。
pub(super) const FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 单元数不少于这个数的写，把发布用时打到标准错误（只给人看进度，不进产物）。
pub(super) const LOGGED_WRITE_UNITS: usize = 5000;
/// 本装置的固定文件系统身份（与别的 eNNN 装置各自独立取一份）。
pub(super) const E181_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x38, 0x31, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
/// 第一个文件的字节数（E142 的 3000 字节，一个数据单元装得下；A3、A4 钉的 t1@50180 就是这个文件）。
pub(super) const FIRST_FILE_BYTES: usize = 3000;
pub(super) const GIBIBYTE: u64 = 1 << 30;

/// 开写前的本地常量回比（登记第五节「要读 `crates/` 常量的量改成本地常量加回比断言」）：对不上返回一句话，作废。
pub(super) fn check_local_constants_against_the_crates() -> Result<(), String> {
    let comparisons: [(&str, u64, u64); 13] = [
        ("槽字节数", SLOT_BYTES_LOCAL, singlefs_format::SLOT_BYTES),
        (
            "聚簇段槽数",
            CLUSTER_SEGMENT_SLOTS_LOCAL,
            singlefs_format::CLUSTER_SEGMENT_SLOTS,
        ),
        (
            "单元区起始槽号",
            UNIT_AREA_START_SLOT_LOCAL,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
        (
            "默认环字节数",
            DEFAULT_JOURNAL_RING_BYTES_LOCAL,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "每区槽数 S（mkfs 默认）",
            ROOT_RING_SLOTS_PER_REGION_LOCAL,
            RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM.count(),
        ),
        (
            "分配记录树叶罩槽数",
            ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL,
            singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        ),
        (
            "分配记录树内部扇出",
            ALLOCATION_RECORD_TREE_INTERNAL_FANOUT_LOCAL,
            singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
        ),
        (
            "实例表落点",
            UNIT_AREA_START_SLOT_LOCAL + INSTANCE_TABLE_OFFSET_FROM_UNIT_AREA_START_LOCAL,
            INSTANCE_TABLE_SLOT.0,
        ),
        (
            "树表第 0 版落点",
            UNIT_AREA_START_SLOT_LOCAL + TREE_TABLE_OFFSET_FROM_UNIT_AREA_START_LOCAL,
            TREE_TABLE_GENESIS_SLOT.0,
        ),
        (
            "数据单元槽数",
            DATA_UNIT_SLOTS_LOCAL,
            singlefs_format::DATA_UNIT_BYTES / singlefs_format::SLOT_BYTES,
        ),
        (
            "数据单元字节数",
            u64::try_from(DATA_UNIT_BYTES_LOCAL).expect("字节数"),
            singlefs_format::DATA_UNIT_BYTES,
        ),
        (
            "小几何的单元区起点",
            SMALL_GEOMETRY_UNIT_AREA_START_SLOT,
            UnitAreaStart::following_the_journal_ring(SMALL_GEOMETRY_JOURNAL_RING_BYTES)
                .expect("小环末尾在段边界上")
                .slot()
                .0,
        ),
        (
            "默认环的单元区起点",
            UNIT_AREA_START_SLOT_LOCAL,
            UnitAreaStart::following_the_journal_ring(DEFAULT_JOURNAL_RING_BYTES_LOCAL)
                .expect("默认环末尾在段边界上")
                .slot()
                .0,
        ),
    ];
    let mut mismatches = Vec::new();
    for (name, local, crate_value) in comparisons {
        emit(&format!(
            "name=constant_check item={name} local={local} crate={crate_value} equal={}",
            local == crate_value
        ));
        if local != crate_value {
            mismatches.push(format!("{name}：本地 {local}，crates {crate_value}"));
        }
    }
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(format!("本地常量回比不等：{}", mismatches.join("；")))
    }
}

/// 登记第七节的钉绝对值的断言：A1（G默认 本地 K1 = 529）、A3（H1 之后第一个文件的数据单元在 50180、每块盘最低落点 50182）、
/// A4（H1 所选那一版账里记录在盘 0 上的最大末槽 = 50253）、B1（单元区槽数与整段数）、B2（K1 逐层罩到的位置数）。
/// 对不上走 Y7「条款可能错」：返回 `Err`，停机。
pub(super) fn run_anchors() -> Result<(), String> {
    let pool = build_pool_after_the_first_file(
        GEOMETRY_DEFAULT,
        CodeTwoTreeNodeCapacities::FromTheNodeFormat,
    );
    let unit_areas: Vec<(u64, u64)> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            (
                device_map.unit_area_start().slot().0,
                device_map.unit_area_slots(),
            )
        })
        .collect();
    let root_level_read = pool
        .current
        .position_addressed_tree_heights_read_from_the_root_node_headers()
        .allocation_record_tree
        - 1;
    let local_allocation_record_tree_nodes = local_allocation_record_tree_nodes(
        u8::try_from(root_level_read).expect("层级"),
        &unit_areas,
    );
    let level_zero_positions = (unit_areas[0].0 + unit_areas[0].1 - 1)
        / ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL
        - unit_areas[0].0 / ALLOCATION_RECORD_TREE_LEAF_SLOTS_LOCAL
        + 1;
    emit(&format!(
        "name=anchor_allocation_record_tree_nodes local_allocation_record_tree_nodes={local_allocation_record_tree_nodes} expected_allocation_record_tree_nodes=529 root_level_read_from_the_header={root_level_read} level_zero_positions_per_device={level_zero_positions} expected_level_zero_positions=262 unit_areas={unit_areas:?}"
    ));
    let first_file_slot = pool
        .current
        .units
        .iter()
        .find(|unit| unit.identity == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .map(|unit| unit.slot.0);
    let lowest_slots: Vec<Option<u64>> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            device_map
                .lowest_user_data_slot(pool.allocator.cluster_segments())
                .map(|slot| slot.0)
        })
        .collect();
    emit(&format!(
        "name=anchor_first_file_slot_and_lowest_slots first_file_data_unit_slot={first_file_slot:?} expected=Some(50180) lowest_user_data_slot_per_device={lowest_slots:?} expected_lowest=Some(50182)"
    ));
    let largest_end_on_device_zero = pool
        .current
        .allocation_records
        .iter()
        .filter(|record| record.device == DeviceIdentity(0))
        .map(|record| record.slot.0 + u64::from(record.span_slots))
        .max();
    emit(&format!(
        "name=anchor_largest_record_end_on_device_zero largest_record_end_on_device_zero={largest_end_on_device_zero:?} expected=Some(50253)"
    ));
    let unit_area_slots_of = |geometry: Geometry| -> Vec<u64> {
        geometry
            .device_bytes
            .iter()
            .map(|device_bytes| device_bytes / SLOT_BYTES_LOCAL - UNIT_AREA_START_SLOT_LOCAL)
            .collect()
    };
    let default_areas = unit_area_slots_of(GEOMETRY_DEFAULT);
    let narrow_areas = unit_area_slots_of(GEOMETRY_NARROW);
    emit(&format!(
        "name=anchor_unit_area_slots_and_whole_segments default_unit_area_slots={default_areas:?} default_whole_segments={} narrow_unit_area_slots={narrow_areas:?} narrow_whole_segments={} narrow_tail_slots={}",
        default_areas[0] / CLUSTER_SEGMENT_SLOTS_LOCAL,
        narrow_areas[0] / CLUSTER_SEGMENT_SLOTS_LOCAL,
        narrow_areas[0] % CLUSTER_SEGMENT_SLOTS_LOCAL,
    ));
    let mut failures = Vec::new();
    if local_allocation_record_tree_nodes != 529 || level_zero_positions != 262 {
        failures.push(format!(
            "分配记录树节点数：{local_allocation_record_tree_nodes}（应 529），层级 0 每盘 {level_zero_positions}（应 262）"
        ));
    }
    if first_file_slot != Some(50180) || lowest_slots != vec![Some(50182), Some(50182)] {
        failures.push(format!(
            "第一个文件的落点与各盘最低落点：{first_file_slot:?} {lowest_slots:?}"
        ));
    }
    if largest_end_on_device_zero != Some(50253) {
        failures.push(format!(
            "盘 0 上记录的最大末槽：{largest_end_on_device_zero:?}"
        ));
    }
    if default_areas != vec![211_968, 211_968]
        || narrow_areas != vec![146_465, 146_465]
        || default_areas[0] / CLUSTER_SEGMENT_SLOTS_LOCAL != 3312
        || narrow_areas[0] / CLUSTER_SEGMENT_SLOTS_LOCAL != 2288
        || narrow_areas[0] % CLUSTER_SEGMENT_SLOTS_LOCAL != 33
    {
        failures.push(format!(
            "单元区槽数与整段数：{default_areas:?} {narrow_areas:?}"
        ));
    }
    let real_areas: Vec<u64> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.unit_area_slots())
        .collect();
    if real_areas != default_areas {
        failures.push(format!(
            "单元区槽数：分配器的单元区槽数 {real_areas:?} 与按字节数算的 {default_areas:?} 不等"
        ));
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("；"))
    }
}

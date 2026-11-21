//! 本地常量与它们和实现的回比。

use super::*;

// 下面这些常量抄自 kb 的登记位（值与出处在每条的注释里），不引 `crates/` 的同名常量：装置与实现不共用一个数，
// `local_constants_against_the_implementation` 逐个回比，两边一起错时红（登记第五节「本地常量」表）。这一条挡不住 kb 与实现一起取错。
/// 槽字节：D3 已定项 10 ①「64 个 16 KiB 槽」；实现侧 `singlefs_format::SLOT_BYTES`。
pub(super) const SLOT_BYTES: u64 = 16384;
/// 数据单元载荷：D18 已定项 16（32768 − 头 134）；实现侧 `singlefs_core::unit::data_unit_payload_capacity()`。
pub(super) const DATA_UNIT_PAYLOAD_BYTES: u64 = 32634;
/// 数据单元字节：D18 已定项 3；实现侧 `singlefs_format::DATA_UNIT_BYTES`。
pub(super) const DATA_UNIT_BYTES: u64 = 32768;
/// 聚簇段槽数：D3 已定项 10 ①；实现侧 `singlefs_format::CLUSTER_SEGMENT_SLOTS`。
pub(super) const CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// 默认环下单元区起始槽号：D3 已定项 10 ④；实现侧 `singlefs_format::UNIT_AREA_START_SLOT`。
pub(super) const DEFAULT_UNIT_AREA_START_SLOT: u64 = 50176;
/// 默认 journal 环长：D23 已定项 19 ③；实现侧 `singlefs_format::JOURNAL_RING_DEFAULT_BYTES`。
pub(super) const DEFAULT_JOURNAL_RING_BYTES: u64 = 805_306_368;
/// journal 环起点槽号：D23 已定项 19；实现侧 `singlefs_format::JOURNAL_RING_START_SLOT`。
pub(super) const JOURNAL_RING_START_SLOT: u64 = 1024;
/// 一条 journal 记录字节：D23 已定项 19；实现侧 `singlefs_format::JOURNAL_RECORD_BYTES`。
pub(super) const JOURNAL_RECORD_BYTES: u64 = 4096;
/// mkfs 的每区槽数 S：D22 已定项 2 参数表与已定项 16 第 3 句（默认 min(8, …)，这个装置的槽距下取 8）；实现侧 `singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`。
const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;
/// 根环区数：D22 已定项 2；实现侧 `singlefs_format::ROOT_RING_REGIONS`。
const ROOT_RING_REGIONS: u64 = 3;
/// 一次准入的发布数 B = 4 + 2 k_tol（k_tol = 2）：D16 已定项 1「准入」一行；实现侧 `singlefs_core::mount::PUBLISHES_PER_ADMISSION_AT_MOST`。
const PUBLISHES_PER_ADMISSION_AT_MOST: u64 = 8;
/// 下段叶罩的数据单元数：D8 已定项 14；实现侧 `singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS`。
pub(super) const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
/// extent 树内部扇出：D8 已定项 14；实现侧 `singlefs_format::EXTENT_TREE_INTERNAL_FANOUT`。
pub(super) const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
/// 复制数：第一版两块盘互为镜像（登记 1.2 副本数固定 2）。
pub(super) const DEVICE_COUNT: u64 = 2;
/// 数据单元头里的魔数与类标签：`singlefs_core::unit::{UNIT_MAGIC, UNIT_CLASS_DATA}`（只用来认「这一次写是数据单元的写」）。
pub(super) const UNIT_MAGIC_BYTES: [u8; 4] = *b"SFSU";
pub(super) const UNIT_CLASS_DATA_CODE: u8 = 1;
/// 单元头里类标签的字节下标（魔数 4 + 版本 2）。
pub(super) const UNIT_CLASS_BYTE_INDEX: usize = 6;
pub(super) const SECTOR_BYTES: usize = 512;
pub(super) const PHYSICAL_BLOCK_BYTES: u32 = 512;
pub(super) const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
pub(super) const STOP_EXIT_CODE: i32 = 4;
/// 登记 5.1 的 T 探测上界用的记录数：会话没法按「第 N 条」指定，装置从拒绝里读（`records_of_the_publish`、`record_limit`）。
pub(super) const PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS: u64 = 120_000;

/// 本地常量逐个与 `crates/` 回比；对不上就停（登记第十一节作废条款：本地常量与实现侧回比有一条不等）。交回比过的个数。
pub(super) fn local_constants_against_the_implementation() -> usize {
    let pairs: [(&str, u64, u64); 14] = [
        ("SLOT_BYTES", SLOT_BYTES, singlefs_format::SLOT_BYTES),
        (
            "DATA_UNIT_PAYLOAD_BYTES",
            DATA_UNIT_PAYLOAD_BYTES,
            u64::try_from(singlefs_core::unit::data_unit_payload_capacity()).expect("载荷字节数"),
        ),
        (
            "DATA_UNIT_BYTES",
            DATA_UNIT_BYTES,
            singlefs_format::DATA_UNIT_BYTES,
        ),
        (
            "CLUSTER_SEGMENT_SLOTS",
            CLUSTER_SEGMENT_SLOTS,
            singlefs_format::CLUSTER_SEGMENT_SLOTS,
        ),
        (
            "DEFAULT_UNIT_AREA_START_SLOT",
            DEFAULT_UNIT_AREA_START_SLOT,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
        (
            "DEFAULT_JOURNAL_RING_BYTES",
            DEFAULT_JOURNAL_RING_BYTES,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "JOURNAL_RING_START_SLOT",
            JOURNAL_RING_START_SLOT,
            singlefs_format::JOURNAL_RING_START_SLOT,
        ),
        (
            "JOURNAL_RECORD_BYTES",
            JOURNAL_RECORD_BYTES,
            singlefs_format::JOURNAL_RECORD_BYTES,
        ),
        (
            "ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM",
            ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
        ),
        (
            "ROOT_RING_REGIONS",
            ROOT_RING_REGIONS,
            singlefs_format::ROOT_RING_REGIONS,
        ),
        (
            "PUBLISHES_PER_ADMISSION_AT_MOST",
            PUBLISHES_PER_ADMISSION_AT_MOST,
            u64::try_from(singlefs_core::mount::PUBLISHES_PER_ADMISSION_AT_MOST)
                .expect("一次准入的发布数"),
        ),
        (
            "EXTENT_TREE_LOWER_LEAF_DATA_UNITS",
            EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
            singlefs_format::EXTENT_TREE_LOWER_LEAF_DATA_UNITS,
        ),
        (
            "EXTENT_TREE_INTERNAL_FANOUT",
            EXTENT_TREE_INTERNAL_FANOUT,
            singlefs_format::EXTENT_TREE_INTERNAL_FANOUT,
        ),
        (
            "UNIT_CLASS_DATA_CODE",
            u64::from(UNIT_CLASS_DATA_CODE),
            u64::from(singlefs_core::unit::UNIT_CLASS_DATA),
        ),
    ];
    for (name, local, implementation) in pairs {
        if local != implementation {
            stop_with(
                "local_constant_differs_from_the_implementation",
                &format!("constant={name} local={local} implementation={implementation}"),
            );
        }
    }
    assert_eq!(
        UNIT_MAGIC_BYTES,
        singlefs_core::unit::UNIT_MAGIC,
        "数据单元魔数本地值与实现不同"
    );
    pairs.len() + 1
}

#[cfg(test)]
mod tests;

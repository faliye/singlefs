//! 本地常量：格式常量与条款里的数（第七节 B 类用），与它们和实现的回比（停机条款 S1）。

use super::*;

// 下面这些常量抄自 `crates/singlefs-format/src/lib.rs` 与 `crates/singlefs-core/src/admission.rs`（各自对应的常量名在各自的注释里），
// 写成本地常量：装置与实现不共用一个数，两边一起错时 `local_constants_against_the_implementation` 逐个回比会红
// （登记 5.0「装置读 `crates/` 的这些常量时一律写成本地常量加回比断言」）。这一条挡不住 kb 与实现一起取错。
/// `singlefs-format/src/lib.rs` 的 `SLOT_BYTES`。
pub(super) const SLOT_BYTES: u64 = 16384;
/// `singlefs-format/src/lib.rs` 的 `DATA_UNIT_BYTES`。
pub(super) const DATA_UNIT_BYTES: u64 = 32768;
/// `singlefs-format/src/lib.rs` 的 `ROOT_RING_REGIONS`。
pub(super) const ROOT_RING_REGIONS: u64 = 3;
/// `singlefs-format/src/lib.rs` 的 `ROOT_RING_SLOTS_PER_REGION_MINIMUM`。
pub(super) const ROOT_RING_SLOTS_PER_REGION_MINIMUM: u64 = 4;
/// `singlefs-format/src/lib.rs` 的 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM`。
pub(super) const ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = 16;
/// `singlefs-format/src/lib.rs` 的 `ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM`。
pub(super) const ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM: u64 = 8;
/// `singlefs-format/src/lib.rs` 的 `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`；登记 5.4 的产品批大小 K = 144。
pub(super) const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
/// `singlefs-format/src/lib.rs` 的 `EXTENT_TREE_INTERNAL_FANOUT`。
const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
/// `singlefs-format/src/lib.rs` 的 `ALLOCATION_RECORD_TREE_LEAF_SLOTS`。
pub(super) const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
/// `singlefs-format/src/lib.rs` 的 `ALLOCATION_RECORD_TREE_INTERNAL_FANOUT`。
pub(super) const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
/// `singlefs-format/src/lib.rs` 的 `JOURNAL_RING_START_SLOT`。
pub(super) const JOURNAL_RING_START_SLOT: u64 = 1024;
/// `singlefs-format/src/lib.rs` 的 `JOURNAL_RING_DEFAULT_BYTES`。
pub(super) const JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
/// `singlefs-format/src/lib.rs` 的 `CLUSTER_SEGMENT_SLOTS`。
const CLUSTER_SEGMENT_SLOTS: u64 = 64;
/// `singlefs-format/src/lib.rs` 的 `JOURNAL_RECORD_BYTES`（一条 journal 记录的字节数）。
pub(super) const JOURNAL_RECORD_BYTES: u64 = 4096;
/// `singlefs-format/src/lib.rs` 的 `JOURNAL_SAFETY_FACTOR`（F：在飞记录数上限 = 环槽数 ÷ F）。
pub(super) const JOURNAL_SAFETY_FACTOR: u64 = 3;
/// `singlefs-core/src/admission.rs` 的 `INSTANCE_SWITCHES_ALLOWED_PER_MOUNT`（N_switch）。
pub(super) const INSTANCE_SWITCHES_PER_MOUNT: u64 = 3;
/// `singlefs-core/src/admission.rs` 的 `WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH`（R）。
pub(super) const WARM_UP_EMPTY_PUBLISHES_PER_SWITCH: u64 = 3;
/// `singlefs-core/src/admission.rs` 的 `INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE`。
const INSTANCE_ROWS_PER_PAGE: u64 = 369;
/// `TransactionUnit::InstanceTable.span_slots()`（实例表一片 32768 字节）。
pub(super) const INSTANCE_TABLE_PAGE_SLOTS: u64 = 2;

/// 第一版两块盘互为镜像（登记 1.2「副本数」：固定 2，不扫）。
pub(super) const DEVICE_COUNT: u64 = 2;
/// 登记 5.4 只供测试的最小批。
pub(super) const SMALLEST_BATCH_DATA_UNITS: u64 = 1;
/// 登记 5.7 P0 的 journal 环：1 MiB；P1 的环从它起往大取（登记第十二节修订第 15 条），不小于它。
pub(super) const SMALL_JOURNAL_RING_BYTES: u64 = 1 << 20;
/// mkfs 收的 journal 环按 1 MiB 一档：单元区起点 = 1024 + ⌈环长 ÷ 16384⌉ 要落 64 槽边界（`allocator.rs` 的
/// `following_the_journal_ring`），同一档里的环长起点相同、盘的布局相同（修订第 15 条取档的上端）。
pub(super) const JOURNAL_RING_STEP_BYTES: u64 = CLUSTER_SEGMENT_SLOTS * SLOT_BYTES;
/// 修订第 15 条「先定环」那一步把环放大的次数上限（放大一次就直接跳到装得下那次写的记录数的一档，再放大只在记录数随环涨时发生）。
pub(super) const JOURNAL_RING_ENLARGEMENTS_AT_MOST: usize = 4;
/// 登记 5.7 P2 的 journal 环：6 MiB（`crates/singlefs-verification/src/history.rs` 窄盘先例）。
pub(super) const NARROW_JOURNAL_RING_BYTES: u64 = 6 << 20;
/// 登记 5.7 P2 的单元区槽数。
pub(super) const NARROW_UNIT_AREA_SLOTS: u64 = 384;
/// 装置单测用的 P2 宽盘：S = 4 时环窗口真扣（每盘 364 槽）之后，384 槽连第一个文件都写不进；
/// 照「准入可用与放大前一样」的先例放大到 790 槽，重挂载后的可用与 384 槽在没有环窗口时只差墓碑每对象那 41 字节。
/// 登记 5.7 的 P2 取样点不随它变。
#[cfg(test)]
pub(super) const NARROW_UNIT_AREA_SLOTS_WITH_THE_RING_WINDOW_RESERVED: u64 = 790;
/// 登记 5.7 P4：4 GiB。
pub(super) const PRODUCT_DEVICE_BYTES: u64 = 4 << 30;
/// 盘的物理块大小：harness 的内存稀疏盘都按 512 建。
pub(super) const PHYSICAL_BLOCK_BYTES: u32 = 512;
/// 第一个文件的写时刻（只要固定，取哪个数不影响任何量）。
pub(super) const WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 停机条款不过的退出码。
pub(super) const STOP_EXIT_CODE: i32 = 4;
/// 一片 panic 时装置的退出码：与 Rust 主线程 panic 的退出码相同。
pub(super) const PANIC_EXIT_CODE: i32 = 101;
/// 主工作区那份不带补丁时的退出码（登记修订第 2 条）。
pub(super) const DELETE_PATH_PATCH_ABSENT_EXIT_CODE: i32 = 3;
/// 梯子末端没饱和时按 ×2 续加的次数上限（登记 8.4：直到饱和或撞上沿；上沿之外再给一个有限的界，让循环有上界）。
pub(super) const LADDER_EXTENSIONS_AT_MOST: usize = 6;
/// 8.3 差额敏感性只算到 δ = 3。
pub(super) const DELTA_AT_MOST: u64 = 3;
/// B 臂一次准入的发布数上限（D16 已定项 1「B = 4 + 2 k_tol」，k_tol = 2）。
pub(super) const PUBLISHES_IN_ONE_ADMISSION_AT_MOST: u64 = 8;

/// 本地常量逐个与 `crates/` 回比；对不上就停（第十一节「装置与 `crates/` 对不上」）。交回比过的个数。
pub(super) fn local_constants_against_the_implementation() -> usize {
    let pairs: [(&str, u64, u64); 19] = [
        ("SLOT_BYTES", SLOT_BYTES, singlefs_format::SLOT_BYTES),
        (
            "DATA_UNIT_BYTES",
            DATA_UNIT_BYTES,
            singlefs_format::DATA_UNIT_BYTES,
        ),
        (
            "ROOT_RING_REGIONS",
            ROOT_RING_REGIONS,
            singlefs_format::ROOT_RING_REGIONS,
        ),
        (
            "ROOT_RING_SLOTS_PER_REGION_MINIMUM",
            ROOT_RING_SLOTS_PER_REGION_MINIMUM,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MINIMUM,
        ),
        (
            "ROOT_RING_SLOTS_PER_REGION_MAXIMUM",
            ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_MAXIMUM,
        ),
        (
            "ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM",
            ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
            singlefs_format::ROOT_RING_SLOTS_PER_REGION_AT_MAKE_FILESYSTEM,
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
            "ALLOCATION_RECORD_TREE_LEAF_SLOTS",
            ALLOCATION_RECORD_TREE_LEAF_SLOTS,
            singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS,
        ),
        (
            "ALLOCATION_RECORD_TREE_INTERNAL_FANOUT",
            ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
            singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT,
        ),
        (
            "JOURNAL_RING_START_SLOT",
            JOURNAL_RING_START_SLOT,
            singlefs_format::JOURNAL_RING_START_SLOT,
        ),
        (
            "JOURNAL_RING_DEFAULT_BYTES",
            JOURNAL_RING_DEFAULT_BYTES,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "CLUSTER_SEGMENT_SLOTS",
            CLUSTER_SEGMENT_SLOTS,
            singlefs_format::CLUSTER_SEGMENT_SLOTS,
        ),
        (
            "JOURNAL_RECORD_BYTES",
            JOURNAL_RECORD_BYTES,
            singlefs_format::JOURNAL_RECORD_BYTES,
        ),
        (
            "JOURNAL_SAFETY_FACTOR",
            JOURNAL_SAFETY_FACTOR,
            singlefs_format::JOURNAL_SAFETY_FACTOR,
        ),
        (
            "INSTANCE_SWITCHES_ALLOWED_PER_MOUNT",
            INSTANCE_SWITCHES_PER_MOUNT,
            INSTANCE_SWITCHES_ALLOWED_PER_MOUNT,
        ),
        (
            "WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH",
            WARM_UP_EMPTY_PUBLISHES_PER_SWITCH,
            WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH,
        ),
        (
            "INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE",
            INSTANCE_ROWS_PER_PAGE,
            INSTANCE_ROWS_PER_INSTANCE_TABLE_PAGE,
        ),
        (
            "InstanceTable.span_slots",
            INSTANCE_TABLE_PAGE_SLOTS,
            TransactionUnit::InstanceTable.span_slots(),
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
    pairs.len()
}

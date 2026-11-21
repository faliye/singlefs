//! 本地常量与它们和 crates 的回比（停机条款 S1）。

use super::*;

// ───────────────────────────── 本地常量（S1 回比） ─────────────────────────────

/// 16 KiB 槽（登记 S1：`singlefs_format::SLOT_BYTES` 16384）。
const LOCAL_SLOT_BYTES: u64 = 16_384;
/// 索引节点单元长（S1：`NODE_BYTES` 16384）。
pub(super) const LOCAL_NODE_BYTES: u64 = 16_384;
/// 数据单元与打包单元长（S1：`DATA_UNIT_BYTES` 32768）。
pub(super) const LOCAL_DATA_UNIT_BYTES: u64 = 32_768;
/// 一条 journal 记录（S1：`JOURNAL_RECORD_BYTES` 4096）。
pub(super) const LOCAL_JOURNAL_RECORD_BYTES: u64 = 4_096;
/// journal 环起点槽（S1：`JOURNAL_RING_START_SLOT` 1024）。
const LOCAL_JOURNAL_RING_START_SLOT: u64 = 1_024;
/// journal 环默认字节数（S1：`JOURNAL_RING_DEFAULT_BYTES` 805306368）。
const LOCAL_JOURNAL_RING_DEFAULT_BYTES: u64 = 805_306_368;
/// 单元区起点槽（S1：`UNIT_AREA_START_SLOT` 50176）。
const LOCAL_UNIT_AREA_START_SLOT: u64 = 50_176;
/// 崩溃点重放用的扇区字节数（S1：`singlefs_verification::memory_pool::SECTOR_BYTES` 512）。
pub(super) const LOCAL_SECTOR_BYTES: u64 = 512;
/// 五种 key 宽（S1：extent 24、inode 8、分配 10、记账 22、映射 27）与树表 8。
pub(super) const LOCAL_KEY_WIDTH_EXTENT: usize = 24;
pub(super) const LOCAL_KEY_WIDTH_INODE: usize = 8;
pub(super) const LOCAL_KEY_WIDTH_ALLOCATION: usize = 10;
pub(super) const LOCAL_KEY_WIDTH_ACCOUNTING: usize = 22;
pub(super) const LOCAL_KEY_WIDTH_MAPPING: usize = 27;
const LOCAL_KEY_WIDTH_TREE_TABLE: usize = 8;

/// journal 环在盘上的起止（字节）：起点 1024 × 16384，终点 = 单元区起点 50176 × 16384。
pub(super) const RING_START_BYTES: u64 = LOCAL_JOURNAL_RING_START_SLOT * LOCAL_SLOT_BYTES;
pub(super) const UNIT_AREA_START_BYTES: u64 = LOCAL_UNIT_AREA_START_SLOT * LOCAL_SLOT_BYTES;

/// 登记第一节「接近 1 / 接近状态数」与 G3「很小」的门槛：0.5。
pub(super) const RATIO_THRESHOLD_NUMERATOR: u64 = 1;
pub(super) const RATIO_THRESHOLD_DENOMINATOR: u64 = 2;
/// 第一段 G3 够判：s_lo、s_hi 在 0.5 同一侧且离 0.5 至少 0.1（5.5）。
pub(super) const SHARE_MARGIN_FOR_ENOUGH: f64 = 0.1;
/// D1-stride 的步长（5.5：素数 2311）。
pub(super) const STRIDE_STEP: u64 = 2_311;
/// 对照样本里大段各取序号最小的这么多个（5.5）。
pub(super) const CONTROL_SAMPLE_PER_LARGE_SEGMENT: usize = 4_096;
/// PC3：覆盖写固定脚本流段 18（第 1 次登记的段 14、第 2 次重跑登记的段 16，同一批写）里序号最小的 1000 个状态、每个计时区各加 2 ms 忙等（5.3；重跑登记 I-6）。
pub(super) const TIMING_CONTROL_STATES: usize = 1_000;
pub(super) const TIMING_CONTROL_BUSY_WAIT: Duration = Duration::from_millis(2);
/// PC3 的判据：加忙等的那个区多出 ≥ 1.8 s，其余每个区变化 < 1 s。
pub(super) const TIMING_CONTROL_MINIMUM_ADDED_NANOSECONDS: u128 = 1_800_000_000;
pub(super) const TIMING_CONTROL_MAXIMUM_OTHER_CHANGE_NANOSECONDS: u128 = 1_000_000_000;
/// 容量（A6）：单卡最大 32607 MiB、16 GB 那一档里最小 16303 MiB、五卡合计 97843 MiB。
pub(super) const MEBIBYTE: u128 = 1 << 20;
pub(super) const LARGEST_CARD_MEBIBYTES: u128 = 32_607;
pub(super) const SMALLEST_SIXTEEN_GIGABYTE_CARD_MEBIBYTES: u128 = 16_303;
pub(super) const FIVE_CARDS_MEBIBYTES: u128 = 97_843;
pub(super) const CARD_COUNT: u128 = 5;
/// 外推点：问题单的 10¹⁰，另报覆盖写固定脚本流整条流的全量状态数（第四节 4.2；第 3 次重跑登记 A7：取号写两轮之后 80 段）。
pub(super) const EXTRAPOLATED_STATES: u128 = 10_000_000_000;
pub(super) const SECOND_STREAM_FULL_STATES: u128 = 1_678_377_220;
/// A7：覆盖写固定脚本流整条流最大一段 2²⁸ − 1（重跑登记 A7：28 个单元写的段，C577 之前是 3² · 2²⁸ − 1 的 30 写段）。
pub(super) const SECOND_STREAM_LARGEST_SEGMENT_STATES: u128 = 268_435_455;
/// 容量的 95% 到 100% 之间记「临界」（第六节 G5 表下）。
pub(super) const CRITICAL_BAND_PERCENT: u128 = 95;
/// G5 的字节口径（第六节）：读集每次调用 21 字节，集合每个元素 20 字节，每状态三个键各 16 字节，每个不同键一条 20 字节。
pub(super) const READ_SET_BYTES_PER_CALL: u64 = 21;
pub(super) const SET_BYTES_PER_ELEMENT: u64 = 20;
pub(super) const KEY_BYTES: u64 = 16;
pub(super) const KEYS_STORED_PER_STATE: u64 = 3;
pub(super) const DISTINCT_KEY_ENTRY_BYTES: u64 = 20;
/// 轨迹检查点（8.1）：段内每 2²⁰ 个状态一个；D1-stride 按取样状态每 2¹² 个一个。
pub(super) const TRAJECTORY_STATES_PER_CHECKPOINT: u64 = 1 << 20;
pub(super) const TRAJECTORY_STRIDE_SAMPLES_PER_CHECKPOINT: u64 = 1 << 12;
/// 一片几个状态：工作线程按片领活。
pub(super) const STATES_PER_SLICE: usize = 256;
/// 线程数的环境变量（登记 5.6）。
pub(super) const THREADS_ENVIRONMENT_VARIABLE: &str = "E161_THREADS";

/// S1：本地常量与 crates 同名常量逐条回比；交回（名字, 本地值, crates 值）。
pub(super) fn local_constants_against_the_crates() -> Vec<(&'static str, u64, u64)> {
    let width =
        |schema: singlefs_checker::KeySchema| u64::try_from(schema.width()).expect("key 宽");
    let local = |value: usize| u64::try_from(value).expect("key 宽");
    vec![
        ("slot_bytes", LOCAL_SLOT_BYTES, singlefs_format::SLOT_BYTES),
        ("node_bytes", LOCAL_NODE_BYTES, singlefs_format::NODE_BYTES),
        (
            "data_unit_bytes",
            LOCAL_DATA_UNIT_BYTES,
            singlefs_format::DATA_UNIT_BYTES,
        ),
        (
            "journal_record_bytes",
            LOCAL_JOURNAL_RECORD_BYTES,
            singlefs_format::JOURNAL_RECORD_BYTES,
        ),
        (
            "journal_ring_start_slot",
            LOCAL_JOURNAL_RING_START_SLOT,
            singlefs_format::JOURNAL_RING_START_SLOT,
        ),
        (
            "journal_ring_default_bytes",
            LOCAL_JOURNAL_RING_DEFAULT_BYTES,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        ),
        (
            "unit_area_start_slot",
            LOCAL_UNIT_AREA_START_SLOT,
            singlefs_format::UNIT_AREA_START_SLOT,
        ),
        (
            "sector_bytes",
            LOCAL_SECTOR_BYTES,
            singlefs_verification::memory_pool::SECTOR_BYTES,
        ),
        (
            "key_width_extent",
            local(LOCAL_KEY_WIDTH_EXTENT),
            width(singlefs_checker::KEY_SCHEMA_EXTENT),
        ),
        (
            "key_width_inode",
            local(LOCAL_KEY_WIDTH_INODE),
            width(singlefs_checker::KEY_SCHEMA_INODE),
        ),
        (
            "key_width_allocation",
            local(LOCAL_KEY_WIDTH_ALLOCATION),
            width(singlefs_checker::KEY_SCHEMA_ALLOCATION),
        ),
        (
            "key_width_accounting",
            local(LOCAL_KEY_WIDTH_ACCOUNTING),
            width(singlefs_checker::KEY_SCHEMA_ACCOUNTING),
        ),
        (
            "key_width_mapping",
            local(LOCAL_KEY_WIDTH_MAPPING),
            width(singlefs_checker::KEY_SCHEMA_MAPPING),
        ),
        (
            "key_width_tree_table",
            local(LOCAL_KEY_WIDTH_TREE_TABLE),
            width(singlefs_checker::KEY_SCHEMA_TREE_TABLE),
        ),
    ]
}

#[cfg(test)]
mod tests;

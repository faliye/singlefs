//! 本地常量（停机条款 S1 回比的那几个）、两条流的名字与各文件的内容。

// 单测（`tests` 子模块）拿第一条流的段长表核闭式与写数：那张表定义在根里，这一份只在测试构建里引。
#[cfg(test)]
use super::LOCAL_FIRST_STREAM_SEGMENT_LENGTHS;

// ───────────────────────────── 本地常量（登记 5.3、S1） ─────────────────────────────
//
// 下面这一批与 `e170_specification_executor_export.rs`「本地常量（登记 5.7、S1）」那一节同一批、同样的值与出处注释；`baseline` 开头与 crates 的同名公开常量或现算值回比，
// 不直接引（决定几何与工作量的数写成本地常量：引了就等于装置与实现共用一个数，两边一起错而逐格相等）。
// 这些常量只在基线那一趟回比：坏实现改了写流，这些数在坏实现那一趟本来就会变，照数记进产物。

/// 盘数：`tests/common/mod.rs` 的 `formatted_recorded_images` 建两块盘（`for device_number in 0..2u32`）。
pub(super) const LOCAL_DEVICE_COUNT: usize = 2;
/// 每块盘的字节数：`tests/common/mod.rs` 的 `IMAGE_BYTES = 4 << 30`。
pub(super) const LOCAL_DEVICE_BYTES: u64 = 4_294_967_296;
/// 物理块与 io_min：`tests/common/mod.rs` 里 `physical_block_size: 512`、`minimum_input_output_bytes: 512`。
pub(super) const LOCAL_PHYSICAL_BLOCK_BYTES: u32 = 512;
pub(super) const LOCAL_MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
/// 导出格式的扇区宽（登记 5.3 `sector_bytes`（512））：与 `singlefs_verification::memory_pool::SECTOR_BYTES` 回比。
pub(super) const LOCAL_SECTOR_BYTES: u64 = 512;
/// 固定 fsid：`tests/common/mod.rs` 的 `E142_FILESYSTEM_IDENTIFIER`。
pub(super) const LOCAL_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
/// 固定写时刻：`tests/common/mod.rs` 的 `FIXED_WRITE_TIME_SECONDS = 1_788_000_000`。
pub(super) const LOCAL_FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 第一个文件的字节数：`tests/common/mod.rs` 的 `FILE_BYTES = 3000`；内容 `index % 251`（`file_content`）。
pub(super) const LOCAL_FIRST_FILE_BYTES: usize = 3000;
const LOCAL_FIRST_FILE_CONTENT_MODULUS: usize = 251;
/// 流一的写数：取号两轮各 2 + 暖机两次各 5 + A 29。
pub(super) const LOCAL_FIRST_STREAM_WRITES: usize = 43;
/// 流一的根槽写数：暖机两次各一条 + A 一条。
pub(super) const LOCAL_FIRST_STREAM_ROOT_WRITES: usize = 3;
/// 流一全量与甲二（每段按甲二展开）的状态数：同一份测试文件的 `FULL_STATES` 与 `IN_PLACE_SUBSETS_STATES`。
pub(super) const LOCAL_FIRST_STREAM_FULL_STATES: u64 = 16_777_268;
pub(super) const LOCAL_FIRST_STREAM_IN_PLACE_SUBSETS_STATES: u64 = 54;
/// 流一每次写只取两态时的全量闭式：同一份文件的 `FULL_STATES_WITH_TWO_STATES_PER_WRITE`（单测拿段长表回算它）。
pub(super) const LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM: u64 = 16_777_243;

/// 流二的段长（80 段）：每次发布是单元段（暖机 txg 1、2 没有）、记录段 2、根槽段 1、轮换段 2；取号（流开头与 B 之后重开可写挂载各一次）两轮各一段 2；
/// 抬 F 与卸载开头先写新 F 一段 2。单元段的写数：A、B、C、回退 D 各 24（回退 D 含谱系容器与谱系树根，D18（块里携带什么信息） 已定项 24 第 5 条），
/// 写行 18，写行之后的两次暖机与抬 F、卸载推的空发布各 16，五次覆盖写与 E 各 28。
pub(super) const LOCAL_SECOND_STREAM_SEGMENT_LENGTHS: [usize; 80] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
    16, // 取号到写行之后的暖机
    2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2,
    28, // C、回退 D、覆盖写
    2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1,
    2, // 抬 F、E、卸载
    16, 2, 1, 2, // 卸载第二次空发布与 txg 19 的轮换
];
/// 流二的写数：段长之和，到 D 199 + 五次覆盖写 5 × 33 + 抬 F 2 + 2 × 21 + E 33 + 卸载 2 + 2 × 21 = 485。
pub(super) const LOCAL_SECOND_STREAM_WRITES: usize = 485;
/// 流二的根槽写数：`tests/crash_enumeration_fixed_script_stream.rs` 里这条流的根槽写数断言（`=> 19`），19 次发布各一条。
pub(super) const LOCAL_SECOND_STREAM_ROOT_WRITES: usize = 19;
/// 流二全量与甲二的状态数：同一份测试文件的 `FULL_STATES_THROUGH_THE_UNMOUNT` 与 `IN_PLACE_SUBSETS_STATES_THROUGH_THE_UNMOUNT`。
pub(super) const LOCAL_SECOND_STREAM_FULL_STATES: u64 = 1_678_377_220;
pub(super) const LOCAL_SECOND_STREAM_IN_PLACE_SUBSETS_STATES: u64 = 294;
/// 流二每次写只取两态时的全量闭式：同一份文件的 `FULL_STATES_WITH_TWO_STATES_PER_WRITE_THROUGH_THE_UNMOUNT`（单测拿段长表回算它）。
pub(super) const LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM: u64 = 1_678_377_095;
/// E 的数据单元落点：同一份测试文件里 E 那一步的断言（`50176`）。
pub(super) const LOCAL_SECOND_STREAM_REUSED_VERSION_DATA_UNIT_SLOT: u64 = 50_176;
/// 抬 F 推的空发布数：同一份测试文件里抬 F 那一步的断言（`raised.publishes.len(), 2`）。
pub(super) const LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES: usize = 2;
/// 卸载推的两次空发布的 txg：同一份测试文件里卸载那一步的断言（`vec![CheckpointTxg(18), CheckpointTxg(19)]`）。
pub(super) const LOCAL_SECOND_STREAM_UNMOUNT_TXGS: [u64; 2] = [18, 19];

/// 取样：批 1 每段内点的分母、批 2 的分母、批 3 的分母（登记 5.4）。
pub(super) const FIRST_BATCH_DIVISOR: u64 = 7;
pub(super) const SECOND_BATCH_DIVISOR: u64 = 14;
pub(super) const THIRD_BATCH_DIVISOR: u64 = 23;
/// 批 1 在带撕裂写的段里另取几个带「1」的状态、每段最多扫几个序号（登记 5.4）。
pub(super) const TORN_STATES_PER_SEGMENT: usize = 4;
pub(super) const TORN_SCAN_ORDINALS_PER_SEGMENT: u64 = 100_000;

/// 卸载那次调用在 `calls.tsv` 里的标签：模型没有卸载的答（登记第一节读法 6），这次调用不对拍。
pub(super) const UNMOUNT_CALL_LABEL: &str = "unmount";

/// 两条流的名字（登记 5.3 的目录名）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StreamName {
    NewPoolFileCreation,
    FixedScriptReuseAfterRaisingFloorThenUnmount,
}

impl StreamName {
    pub(super) fn directory_name(self) -> &'static str {
        match self {
            Self::NewPoolFileCreation => "new_pool_file_creation",
            Self::FixedScriptReuseAfterRaisingFloorThenUnmount => {
                "fixed_script_through_e_then_unmount"
            }
        }
    }
}

pub(super) const STREAM_NAMES: [StreamName; 2] = [
    StreamName::NewPoolFileCreation,
    StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount,
];

// ───────────────────────────── 流二的内容生成式（照抄测试文件） ─────────────────────────────

/// `tests/crash_enumeration_fixed_script_stream.rs` 的 `SECOND_FILE_BYTES` 与 `second_content`。
const SECOND_FILE_BYTES: usize = 4100;

pub(super) fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 同一份文件第 131–143 行。
const THIRD_FILE_BYTES: usize = 2500;

pub(super) fn later_content(seed: usize) -> Vec<u8> {
    (0..3000 + seed)
        .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
        .collect()
}

pub(super) fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

/// 第一个文件的内容按本地常量现算，与验证包库的 `scenario::first_file_content()` 回比（S1）。
pub(super) fn local_first_file_content() -> Vec<u8> {
    (0..LOCAL_FIRST_FILE_BYTES)
        .map(|index| {
            u8::try_from(index % LOCAL_FIRST_FILE_CONTENT_MODULUS).expect("模 251 小于 256")
        })
        .collect()
}

#[cfg(test)]
mod tests;

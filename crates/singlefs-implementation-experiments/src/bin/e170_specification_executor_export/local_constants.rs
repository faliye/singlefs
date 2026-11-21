//! 本地常量（登记 5.7、停机条款 S1）：盘、段长表与写数、取样批的除数，与两条流的名字。

// ───────────────────────────── 本地常量（登记 5.7、S1） ─────────────────────────────

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
pub(super) const LOCAL_FIRST_FILE_CONTENT_MODULUS: usize = 251;

/// 取号写的轮数：D22（单元原子性怎么合成） 已定项 8「取号写两轮」——每轮逐盘写一次系统配置槽、轮后一道池屏障，
/// 于是每次取号在段序列里是两个各 [`LOCAL_DEVICE_COUNT`] 写的段。
#[cfg(test)]
const LOCAL_INSTANCE_ACQUISITION_WRITE_ROUNDS: usize = 2;

/// 流一的段长（12 段）：取号两轮各一段（2、2），暖机两次各是记录段 2、根槽段 1、轮换段 2，A 是 24 个单元写一段、记录 2、根槽 1、轮换 2
/// （C577（发布返回之前、系统配置轮换之后加一道屏障）：轮换自成一段）。单测按这几块拼出整张表回比。
pub(super) const LOCAL_FIRST_STREAM_SEGMENT_LENGTHS: [usize; 12] =
    [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
/// 流一的写数：取号两轮各 2 + 暖机两次各 5 + A 29（24 个单元写、两份记录、根槽、两块盘的系统配置槽轮换）。
pub(super) const LOCAL_FIRST_STREAM_WRITES: usize = 43;
/// 流一的根槽写数：暖机两次各一条 + A 一条。
pub(super) const LOCAL_FIRST_STREAM_ROOT_WRITES: usize = 3;
/// 流一全量的状态数：系统配置槽段（取号两轮、三次轮换，共五段）取三态、各 3² − 1，其余段各 2^n − 1，再加全部持久那一个：
/// 1 + 5 × 8 + 3 × 3 + 3 × 1 + (2²⁴ − 1) = 16777268。甲二（单元段只取全不落）：1 + 5 × 8 + 1 + 3 × 3 + 3 × 1 = 54。
pub(super) const LOCAL_FIRST_STREAM_FULL_STATES: u64 = 16_777_268;
pub(super) const LOCAL_FIRST_STREAM_IN_PLACE_SUBSETS_STATES: u64 = 54;
/// 流一每次写只取两态时的全量闭式 1 + Σ(2^n − 1)：1 + (3 × 5 + 3 × 3 + 1 × 3) + (2²⁴ − 1) = 16777243。
pub(super) const LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM: u64 = 16_777_243;

/// 流二的段长（80 段）：每次发布是单元段（暖机 txg 1、2 没有）、记录段 2、根槽段 1、轮换段 2；取号（流开头与 B 之后重开可写挂载各一次）两轮各一段 2；
/// 抬 F 与卸载开头先写新 F 一段 2。单元段的写数：A、B、C、回退 D 各 24（回退 D 含谱系容器与谱系树根，D18（块里携带什么信息） 已定项 24 第 5 条），
/// 写行 18，写行之后的两次暖机与抬 F、卸载推的空发布各 16，五次覆盖写与 E 各 28。单测按这几块拼出整张表回比。
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
/// 流二的根槽写数：19 次发布各一条。
pub(super) const LOCAL_SECOND_STREAM_ROOT_WRITES: usize = 19;
/// 流二全量的状态数：25 个系统配置槽段（两次取号各两轮 4、19 次轮换、抬 F 与卸载先写新 F 2）各取三态，比两态闭式各多 3² − 2² = 5：
/// 1678377095 + 25 × 5 = 1678377220。甲二：1 + 25 × 8 + 17 个单元段各 1 + 19 个记录段各 3 + 19 个根槽段各 1 = 294。
pub(super) const LOCAL_SECOND_STREAM_FULL_STATES: u64 = 1_678_377_220;
pub(super) const LOCAL_SECOND_STREAM_IN_PLACE_SUBSETS_STATES: u64 = 294;
/// 流二每次写只取两态时的全量闭式 1 + Σ(2^n − 1)，单测拿段长表回算它。
pub(super) const LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM: u64 = 1_678_377_095;
/// E 的数据单元落点：`tests/crash_enumeration_fixed_script_stream.rs` 第 363 行（`50176`）。
pub(super) const LOCAL_SECOND_STREAM_REUSED_VERSION_DATA_UNIT_SLOT: u64 = 50_176;
/// 抬 F 推的空发布数：同一份测试文件第 351 行（`raised.publishes.len(), 2`）。
pub(super) const LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES: usize = 2;
/// 卸载推的两次空发布的 txg：同一份测试文件第 381 行（`vec![CheckpointTxg(18), CheckpointTxg(19)]`）。
pub(super) const LOCAL_SECOND_STREAM_UNMOUNT_TXGS: [u64; 2] = [18, 19];

/// 取样：批 1 每段内点的分母、批 2 的分母、批 3 的分母（登记 5.4）。
pub(super) const FIRST_BATCH_DIVISOR: u64 = 7;
pub(super) const SECOND_BATCH_DIVISOR: u64 = 14;
pub(super) const THIRD_BATCH_DIVISOR: u64 = 23;
/// 批 1 在带撕裂写的段里另取几个带「1」的状态、每段最多扫几个序号（登记 5.4）。
pub(super) const TORN_STATES_PER_SEGMENT: usize = 4;
pub(super) const TORN_SCAN_ORDINALS_PER_SEGMENT: u64 = 100_000;

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

#[cfg(test)]
mod tests;

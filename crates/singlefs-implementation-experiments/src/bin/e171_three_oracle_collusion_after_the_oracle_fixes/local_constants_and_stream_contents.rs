//! 本地常量（停机条款 S1 回比的那几个）、三条流的名字与各文件的内容。

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

// 流一、流二、流三的段长表、写数、全量 / 甲二 / 闭式态数与下面 Q5 那一形的四个下标，按 E171（三oracle合谋测试） 第 2 次重跑登记
// `research/prompts/e171-r2-prereg.md` 修订 2（停机 S20 只有乙类不过）重钉：取 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants.out`
// 里各行的 `crates=` 那一栏。今天的 crates 取号写两轮（D22（单元原子性怎么合成） 已定项 8 落地：每轮两盘各一写、轮后一道池屏障，各成一段），
// 每次取号比旧钉值多一段 2 写：流一、流三各多一段（下标 2），流二取号两次各多一段（下标 2 与 17）；Q5 那一形的下标跟着后推 4。
/// 流一的段长：`tests/crash_enumeration_new_pool_file_creation_stream.rs` 的 `prepare` 里的段长断言；上面那份产物第 36 行 `crates=`。
pub(super) const LOCAL_FIRST_STREAM_SEGMENT_LENGTHS: [usize; 12] =
    [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
/// 流一的写数：同一份文件 `prepare` 里的写数断言（取号两轮各 2 + 暖机两次各 5 + A 29）；产物第 37 行 `crates=43`。
pub(super) const LOCAL_FIRST_STREAM_WRITES: usize = 43;
/// 流一的根槽写数：暖机两次各一条 + A 一条。
pub(super) const LOCAL_FIRST_STREAM_ROOT_WRITES: usize = 3;
/// 流一全量与甲二（每段按甲二展开）的状态数：同一份测试文件里全量与甲二那两个态数常量；产物第 44、45 行 `crates=`。
pub(super) const LOCAL_FIRST_STREAM_FULL_STATES: u64 = 16_777_268;
pub(super) const LOCAL_FIRST_STREAM_IN_PLACE_SUBSETS_STATES: u64 = 54;
/// 流一每次写只取两态时的全量闭式：同一份文件的 `FULL_STATES_WITH_TWO_STATES_PER_WRITE`（单测拿段长表回算它）；产物第 46 行 `crates=`。
pub(super) const LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM: u64 = 16_777_243;

/// 流二的段长（80 段）：取今天 crates 现算的那一份，即这个装置 `stream-constants` 的 S1 `fixed_script_through_e_then_unmount_segment_lengths`
/// 的 `crates=`（`research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants.out` 第 55 行）。
/// 与上一版钉值（78 段，出处 `research/results/e170-r2-o3-spec-executor-2026-10-04-s2b-today-export.out` 里 `step=z5` 那一行的 `crates=`）
/// 只差两次取号各多出的一段 2 写（下标 2 与 17）；回退 D 那次向前发布的 24（多出的 4 次写是那次发布追加的谱系记录，D18（块里携带什么信息） 已定项 24 第 5 条）照旧。
pub(super) const LOCAL_SECOND_STREAM_SEGMENT_LENGTHS: [usize; 80] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
    16, // 取号到写行之后的暖机
    2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2,
    28, // C、回退 D、覆盖写
    2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1,
    2, // 抬 F、E、卸载
    16, 2, 1, 2, // 卸载第二次空发布与 txg 19 的轮换
];
/// 流二的写数：S1 `fixed_script_through_e_then_unmount_writes` 的 `crates=485`（同一份产物第 56 行；上一版 481，两次取号各多 2 写）。
pub(super) const LOCAL_SECOND_STREAM_WRITES: usize = 485;
/// 流二的根槽写数：同一份文件第 482 行（`=> 19`）。
pub(super) const LOCAL_SECOND_STREAM_ROOT_WRITES: usize = 19;
/// 流二全量与甲二的状态数：S1 `fixed_script_through_e_then_unmount_full_states` 的 `crates=1678377220`、甲二那一条的 `crates=294`
/// （同一份产物第 60、61 行；上一版 1_678_377_204、278）。
pub(super) const LOCAL_SECOND_STREAM_FULL_STATES: u64 = 1_678_377_220;
pub(super) const LOCAL_SECOND_STREAM_IN_PLACE_SUBSETS_STATES: u64 = 294;
/// 流二每次写只取两态时的全量闭式：S1 `fixed_script_through_e_then_unmount_two_state_closed_form` 的 `crates=1678377095`
/// （同一份产物第 62 行；上一版 1_678_377_089）。单测拿段长表回算它。
pub(super) const LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM: u64 = 1_678_377_095;
/// E 的数据单元落点：同一份文件第 318 行（`50176`）。
pub(super) const LOCAL_SECOND_STREAM_REUSED_VERSION_DATA_UNIT_SLOT: u64 = 50_176;
/// 抬 F 推的空发布数：同一份文件第 306 行（`raised.publishes.len(), 2`）。
pub(super) const LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES: usize = 2;
/// 卸载推的两次空发布的 txg：同一份文件第 336 行（`vec![CheckpointTxg(18), CheckpointTxg(19)]`）。
pub(super) const LOCAL_SECOND_STREAM_UNMOUNT_TXGS: [u64; 2] = [18, 19];

// 流三（登记第一节读法 2、5.3「要读 crates/ 常量的量」）：没有用例文件可抄，取本装置 `stream-constants` 模式在冻结之前的主工作区上
// 现算的那一份（`crates=` 那一栏；输出与命令在 E171 第 1 次重跑第一段报告「本地常量怎么来的」一节），之后每一趟基线与
// crates 现算的回比（S1）。
// 流三的段长表、写数、全量状态数与闭式在登记修订 6 第 6.2 节第 1 条装上根环表之后重取（txg 26 起照可再分配谓词回收，
// 分配记录跟着变）：取草稿副本上 `stream-constants` 的 `crates=` 那一栏，对照在登记修订 7。
// 段长表另按登记修订 12 重钉；之后段长表、写数、全量 / 甲二态数与闭式一起按第 2 次重跑登记修订 2 重钉（出处在各常量自己的注释里）。
// K 另有推的值 234（登记第七节 A7：删完之后 inode 树没有记录、一容器 233 条），不进回比，照实测记。

/// 一片 inode 叶容器装几条记录：`crates/singlefs-format/src/lib.rs` 的 `INODE_LEAF_RECORDS = 233`
/// （D8（核心索引结构） 已定项 6「一容器 233 条」，⌊(32768 − 136) ÷ 140⌋）。
pub(super) const LOCAL_INODE_LEAF_RECORDS: u64 = 233;
/// 每块盘的系统配置槽数与一槽的字节数（读法 8 (i)「少于两槽」的门槛、阳性对照「见证」重编的槽长）：
/// `crates/singlefs-format/src/lib.rs` 的 `SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE = 2`、`SYSTEM_CONFIGURATION_SLOT_BYTES = 4096`
/// （D22（单元原子性怎么合成） 已定项 8「每盘两槽」）。
pub(super) const LOCAL_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: usize = 2;
pub(super) const LOCAL_SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
/// 流三的段长（960 段）：取今天 crates 现算的那一份，即这个装置 `stream-constants` 的 S1 `first_file_unlinked_then_inodes_across_one_leaf_segment_lengths`
/// 的 `crates=`（`research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants.out` 第 72 行）。
/// 与第 1 次重跑登记修订 12 那一份（959 段）只差取号多出的一段 2 写（下标 2），其后各段后移一位；修订 12 那一次的下标 15、43
/// （20 → 24、24 → 20，见第 1 次重跑登记修订 12）今天落在下标 16、44。写数、根槽写数在下面两行。
pub(super) const LOCAL_THIRD_STREAM_SEGMENT_LENGTHS: &[usize] = &[
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2,
    1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1,
    2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2,
    20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20,
    2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2,
    1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1,
    2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2,
    20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20,
    2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2,
    1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1,
    2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2,
    20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20,
    2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2,
    1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1,
    2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2,
    20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20,
    2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2,
    1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1,
    2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2,
    20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20,
    2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2,
    1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1,
    2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2,
    20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20,
    2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2,
    1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1,
    2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2,
    20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20,
    2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2,
    1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1,
    2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2,
    20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20,
    2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2,
    1, 2,
];
/// 流三的写数：同一份产物第 73 行 `crates=5982`（上一版 5980）。
pub(super) const LOCAL_THIRD_STREAM_WRITES: usize = 5982;
pub(super) const LOCAL_THIRD_STREAM_ROOT_WRITES: usize = 240;
/// 流三全量、甲二的状态数与每次写只取两态时的闭式：同一份产物第 77、78、79 行 `crates=`（上一版 362_809_947、3127、362_808_742）。
pub(super) const LOCAL_THIRD_STREAM_FULL_STATES: u64 = 362_809_955;
pub(super) const LOCAL_THIRD_STREAM_IN_PLACE_SUBSETS_STATES: u64 = 3135;
pub(super) const LOCAL_THIRD_STREAM_TWO_STATE_CLOSED_FORM: u64 = 362_808_745;
/// 流三的 K：叶容器第一次变成两片的那一次建 inode 发布是第几次（从 1 数）；流三建 K + 1 次。
pub(super) const LOCAL_THIRD_STREAM_SPLIT_PUBLISH: usize = 234;
/// 流三删第一个文件那一段的调用数：意图一次加走空的步数（第一个文件 3000 字节、一个数据单元，推的是意图一次加最后一批一次）。
pub(super) const LOCAL_THIRD_STREAM_UNLINK_CALLS: usize = 2;
/// 流三建 inode 的次数上限：叶容器到这么多次还没变成两片，就停下记 S1 不过（防跑不完；233 条一片，两倍余量）。
pub(super) const THIRD_STREAM_INODE_CREATION_LIMIT: usize = 2 * 233 + 2;
/// 流三走空 `deleted_inodes` 的步数上限（第一个文件一个数据单元，意图之后一步就走空；余量防跑不完）。
pub(super) const THIRD_STREAM_UNLINK_STEP_LIMIT: usize = 16;

/// Q5 那一形（读法 7）里已确认的那次调用与在途的那次调用（流二，从 0 数：第 0、1 次是取号、暖机）。
pub(super) const CONFIRMED_LOSS_CONFIRMED_CALL: usize = 2;
pub(super) const CONFIRMED_LOSS_IN_FLIGHT_CALL: usize = 3;

// Q5 那一形在基线上的几样（登记第一节读法 7、5.3）：同上，取 `stream-constants` 现算的那一份。下标都是流二 mkfs 之后录制流里的下标。
// 下面四个按第 2 次重跑登记修订 2 重钉：取 `research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants.out`
// 第 25–28 行的 `crates=`；取号多出一轮（两次写加一道屏障）在它们之前，四个下标都比上一版（50, 51, 55, 56；52；81；57）后推 4。
/// 第 2 次调用（第一个文件那次发布）里删掉的屏障。
pub(super) const LOCAL_CONFIRMED_LOSS_REMOVED_BARRIERS: &[usize] = &[54, 55, 59, 60];
/// 第 2 次调用写根槽的那次 FUA 写（改成普通写）。
pub(super) const LOCAL_CONFIRMED_LOSS_ROOT_WRITE: usize = 56;
/// 截断处：第 3 次调用发出的第一道屏障（主取样点截在它之前）与第 2 次调用的返回点（方向相反的取样点）。
pub(super) const LOCAL_CONFIRMED_LOSS_TRUNCATION: usize = 85;
pub(super) const LOCAL_CONFIRMED_LOSS_REVERSE_TRUNCATION: usize = 61;
/// 主取样点末段的写数、可撕裂写数、全枚举态数（末段每个写的取法数之积，A10）。
pub(super) const LOCAL_CONFIRMED_LOSS_LAST_SEGMENT_WRITES: usize = 29;
pub(super) const LOCAL_CONFIRMED_LOSS_LAST_SEGMENT_TEARABLE_WRITES: usize = 2;
pub(super) const LOCAL_CONFIRMED_LOSS_STATES: u64 = 1_207_959_552;
// 主取样点按等价类判（登记修订 2 第 2.1 节候选二、修订 3）的起点分法（类核对 ③ 细分之前）：键里是末段中不属于第 3 次调用的写（K），
// 类里是第 3 次调用的写（C）。值照第 2.1 节「末段是什么」与「Q5-等价类」两段：K 5 个写、取法 2³ × 3² = 72 种，C 24 个写，
// 类数 72 × 2 = 144，「C 一个没落」那一类各 1 态、「C 至少落一个」那一类各 2²⁴ − 1 = 16 777 215 态，要照原路径判的态 72 + 72 = 144。
/// K 的取法数。
pub(super) const LOCAL_CONFIRMED_LOSS_KEY_CHOICES: u64 = 72;
/// C 的写数。
pub(super) const LOCAL_CONFIRMED_LOSS_IN_FLIGHT_WRITES: usize = 24;
/// 类数。
pub(super) const LOCAL_CONFIRMED_LOSS_CLASSES: u64 = 144;
/// 「C 至少落一个」那一类的态数。
pub(super) const LOCAL_CONFIRMED_LOSS_STATES_OF_A_CLASS_WITH_IN_FLIGHT_WRITES_LANDED: u64 =
    16_777_215;
/// 要照原路径判的态数（每种取法的代表加「C 全落」那一态）。
pub(super) const LOCAL_CONFIRMED_LOSS_JUDGED_STATES: u64 = 144;
/// 「等价类对拍」那一形末段里属于第 3 次调用的写数（修订 5 第 5.2 节丙：k = 6）。
pub(super) const LOCAL_EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES: usize = 6;
/// 「等价类对拍」那一形的态数：末段各写取法数之积，K 的 72 种取法 × C 那 6 个整写各 2 种（修订 5 第 5.2 节，72 × 2⁶ = 4 608）。
pub(super) const LOCAL_EQUIVALENCE_PAIRING_STATES: u64 =
    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2 * 2 * 2 * 2;
/// c1…c6 的镜像对数（修订 5 第 5.2 节：两两同偏移、同长度、分属盘 0 与盘 1，三对的偏移互不相同）。
pub(super) const LOCAL_EQUIVALENCE_PAIRING_MIRRORED_PAIRS: usize = 3;
/// 阳性对照「判态路 panic 单列」`pool_checker` 那条注入的态数：c3 落、c4 没落，K 的 72 种取法 × c1、c2、c5、c6 各 2 种（72 × 2⁴ = 1 152）。
pub(super) const LOCAL_PIECE_PANIC_CONTROL_POOL_CHECKER_STATES: u64 =
    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2 * 2;
/// 同上 `ignored_recovery` 那条：c3 没落、c4 落（72 × 2⁴ = 1 152）。
pub(super) const LOCAL_PIECE_PANIC_CONTROL_IGNORED_RECOVERY_STATES: u64 =
    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2 * 2;
/// 同上 `record_checker` 那条：c5 落、c6 没落、(c3,c4) 同落或同不落，72 × c1、c2 各 2 种 × 2（72 × 2² × 2 = 576）。
pub(super) const LOCAL_PIECE_PANIC_CONTROL_RECORD_CHECKER_STATES: u64 =
    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2;
/// `confirmed-loss-form` 一趟判得完的态数上限：每态推 5–50 毫秒，上限这么多约是一天以内。方向相反的取样点全枚举，超过的不跑、
/// 打一行 `not_run` 退 3 交主 agent；主取样点按等价类判（登记修订 2 第 2.1 节候选二、修订 3），这个上限管的是类核对 ③ 细分之后
/// 要照原路径判的态数（每种取法的代表加「C 全落」那一态），超过走停机 S13（主取样点全枚举是 1 207 959 552 态，第 1 次重跑第一段跑不了）。
pub(super) const CONFIRMED_LOSS_STATE_LIMIT: u64 = 10_000_000;

/// 取样：批 1 每段内点的分母、批 2 的分母、批 3 的分母（登记 5.4）。
pub(super) const FIRST_BATCH_DIVISOR: u64 = 7;
pub(super) const SECOND_BATCH_DIVISOR: u64 = 14;
pub(super) const THIRD_BATCH_DIVISOR: u64 = 23;
/// 批 1 在带撕裂写的段里另取几个带「1」的状态、每段最多扫几个序号（登记 5.4）。
pub(super) const TORN_STATES_PER_SEGMENT: usize = 4;
pub(super) const TORN_SCAN_ORDINALS_PER_SEGMENT: u64 = 100_000;

/// 卸载那次调用在 `calls.tsv` 里的标签：模型照 `IdealModel::answer_unmount` 答（登记第一节读法 1 ①），与别的调用一样对拍。
pub(super) const UNMOUNT_CALL_LABEL: &str = "unmount";

/// 三条流的名字（登记 5.3 的目录名；流三是第 1 次重跑加的，读法 2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StreamName {
    NewPoolFileCreation,
    FixedScriptReuseAfterRaisingFloorThenUnmount,
    FirstFileUnlinkedThenInodesAcrossOneLeaf,
}

impl StreamName {
    pub(super) fn directory_name(self) -> &'static str {
        match self {
            Self::NewPoolFileCreation => "new_pool_file_creation",
            Self::FixedScriptReuseAfterRaisingFloorThenUnmount => {
                "fixed_script_through_e_then_unmount"
            }
            Self::FirstFileUnlinkedThenInodesAcrossOneLeaf => {
                "first_file_unlinked_then_inodes_across_one_leaf"
            }
        }
    }
    /// `--streams` 里写的号：1、2、3。
    pub(super) fn number(self) -> u8 {
        match self {
            Self::NewPoolFileCreation => 1,
            Self::FixedScriptReuseAfterRaisingFloorThenUnmount => 2,
            Self::FirstFileUnlinkedThenInodesAcrossOneLeaf => 3,
        }
    }
}

pub(super) const STREAM_NAMES: [StreamName; 3] = [
    StreamName::NewPoolFileCreation,
    StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount,
    StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf,
];

/// `--streams 1,2,3` 那样的号表换成流名（按号升序、去重）；认不出的号交回 `None`。
pub(super) fn stream_names_of_the_numbers(text: &str) -> Option<Vec<StreamName>> {
    let mut chosen: Vec<StreamName> = Vec::new();
    for token in text.split(',') {
        let stream = STREAM_NAMES
            .iter()
            .copied()
            .find(|stream| stream.number().to_string() == token)?;
        if !chosen.contains(&stream) {
            chosen.push(stream);
        }
    }
    chosen.sort();
    (!chosen.is_empty()).then_some(chosen)
}

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

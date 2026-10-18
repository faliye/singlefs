//! E171（三oracle合谋测试） 第 1 次重跑：入库装置（登记 `research/prompts/e171-r1-prereg.md`，5.3「装置写在哪」）。
//!
//! 由 `e171_three_oracle_collusion.rs`（sha256 37b5b27c…，流二钉值 481、已知坏镜像 43 条那一份）照抄起头，不 `#[path]` 引它；
//! 那份留着不动（它是 s4 冻结树读数的装置）。改五处、加两样（登记 5.3）：
//! ① 卸载调理想模型的卸载答（`IdealModel::answer_unmount`，对拍用 `model_comparison::observed_unmount`）；
//! ② 每次成功调用的交回值与调用之后装置手里的实现对象，`{:?}` 文本的 sha256 记进 `call-outcomes.tsv` 新加的两列与 `call-return-values.tsv`（读法 4）；
//! ③ 调用 panic 与交回错误同样截断照判，态上的恢复 panic 不截断（读法 5）；
//! ④ O1 崩溃一列并列 RefFS 子列（读法 1，`oracle-verdicts.tsv` 新加的几列）；
//! ⑤ 流三 `first_file_unlinked_then_inodes_across_one_leaf`（读法 2）；
//! 加：`--writable-mount-probe`（Q4，读法 8，`writable-mount-probe.tsv`）与 `confirmed-loss-form` 模式（Q5，读法 7）。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness、format、replay-prefix 与 reference-filesystem。
//!
//!   e171_three_oracle_collusion_after_the_oracle_fixes baseline --output-directory <目录> [--streams 1,2,3] [--writable-mount-probe]
//!   e171_three_oracle_collusion_after_the_oracle_fixes row --output-directory <目录> [--streams 1,2,3] [--writable-mount-probe]
//!   e171_three_oracle_collusion_after_the_oracle_fixes confirmed-loss-form --output-directory <目录> [--truncate-at-the-return-point]
//!   e171_three_oracle_collusion_after_the_oracle_fixes stream-constants [--streams 1,2,3]
//!   baseline 与 row 之后可以加 `--positive-control <O1-对拍|O1-对拍-卸载|O1-崩溃判官|RefFS|O2|交回值|内存对象|panic|见证>`
//!   或 `--disable-oracle <O1|O2>`。
//!
//! `baseline`：回比本地常量（S1）、三批取样、写导出文件；`row`：不回比常量、只取批 1 与批 2。`stream-constants`：只驱动选中的流、
//! 打 S1 回比与 Q5 造法在基线上的几样（填本地常量用，不写导出、不判态）。`confirmed-loss-form`：流二驱到第 3 次调用（从 0 数）、
//! 照读法 7 造「已确认的丢了」那一形；方向相反的取样点末段全枚举，主取样点先过「等价类对拍」、再按等价类判（登记修订 2 第 2.1 节
//! 候选二、修订 3），每个判过的态算 O1 三个子列与附带的 O2。目录已存在就拒绝。
//! 标准输出：`E7RESULT` 行（与跑几次无关）与 `E7TIMING` 行（耗时）；收尾 `name=done emitted=N`。
//! 退出码：0 正常；2 命令行不对或输出目录已存在；3 停机条款 S1、S3、S5、S9、S13 没过，或 `confirmed-loss-form` 那一趟 Q5 没跑
//! （判决行带 `q5_not_run=`）（产物照打，交主 agent）。

/// 崩溃点重放取样过崩溃判定库的项（`crash_state_enumeration::sample_crash_states_on_the_cpu`）：项名与设点文件（这份装置自己）。
const CRASH_POINT_REPLAY_SAMPLING_ITEM: CrashPointReplayItem<'static> = CrashPointReplayItem {
    name: "e171_three_oracle_collusion_after_the_oracle_fixes",
    setting_file:
        "crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs",
};

/// 这一次取样的项名：装置一趟按不同的段、不同的序号区间取样几十次，每次是一条不同的流，项名各给一个（库里同一个项名下只登一份清单，
/// 几次共用会撞；块键按内容算，下一趟照样复用判过的块）。
fn sampling_item_name_of_this_call() -> String {
    static NEXT_SAMPLING_NUMBER: std::sync::atomic::AtomicU64 =
        std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        CRASH_POINT_REPLAY_SAMPLING_ITEM.name,
        NEXT_SAMPLING_NUMBER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    )
}

#[path = "../../../singlefs-harness/tests/common/mod.rs"]
mod common;

// RefFS（`crates/singlefs-reference-filesystem`，D13（验证路线） 已定项 3）：checker 档包直接依赖它（登记修订 2 第 2.3 节），
// 类型与 harness 翻译层用的是同一份；翻译层的 `events_of` 交不出本装置三条流的内容，所以照那份映射在本装置里交事件，
// 要直接用 RefFS 的类型。它自己的单测归它自己的包（harness 档），不再编进本装置。
use singlefs_reference_filesystem as reference_filesystem;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    check_records, classified_oracle_violation_for_versions, crash_point_replay_plan_state_count,
    crash_state_enumeration_tables, full_expansion, quick_tier_expansion,
    CrashPointReplayOracleViolation, CrashStateEnumerationTables, CrashStateJudgement,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
    CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE, CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER,
    CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER, RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT,
    RECORD_CHECKER_ROOT_WITHOUT_RECORD,
};
use singlefs_checker_tier::crash_facts::{
    extract_flow_facts, persisted_writes_of_state, SegmentFact, TornImageFact,
    MAXIMUM_ENUMERATED_WRITES,
};
use singlefs_checker_tier::crash_injection::{
    materialized_crash_image, writable_mount_alone_on, WritableMountAlone,
};
use singlefs_checker_tier::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InodeNumber, InstanceGeneration,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator, RootRingOccupancy};
use singlefs_core::journal::JournalRecordPlaceInPublish;
use singlefs_core::make_filesystem::{
    root_ring_occupancy_after_make_filesystem, MakeFilesystemOutput, INSTANCE_TABLE_SLOT,
    TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::recovery::{
    choose_system_configuration, scan_journal, verified_system_configuration_slots, PoolReader,
    RecoveryOutcome, RecoveryReport,
};
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_new_inodes, publish_overwrite,
    publish_unlink_intent, publish_unlink_step, warm_up, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput, WarmUpOutput, FIRST_INODE_NUMBER,
};
use singlefs_core::unlink::UnlinkBatchSize;
use singlefs_harness::history::StepPosition;
use singlefs_harness::memory_pool::{
    newest_persisted_root, writes_and_segments_with_stream_indexes_and_entries, CrashImage,
    MemoryPool, PublishedVersion, RecordCheck, RetainedWrite, WrittenContents,
};
use singlefs_harness::model::{
    crash_recovery_disagreement, IdealModel, ModelAnswer, ModelDeviceIdentity, ModelDisagreement,
    ModelPoolGeometry, ModelRootKey, ObservedEffect, ObservedFile, ObservedOutcome,
    ObservedReadBack,
};
use singlefs_harness::model_comparison::{
    model_root_key, observed_mount_carrying_to_zero_unit_publishes,
    observed_read_back_after_a_crash, observed_root_of_file_version, observed_unmount,
    refusal_reason_of_mount_error, refusal_reason_of_publish_error,
    refusal_reason_of_rollback_error, reported_ceiling_of_mount_error,
    root_ring_slot_still_bad_after_one_reread_of_mount_error,
    root_ring_slot_still_bad_after_one_reread_of_rollback_error,
};
use singlefs_harness::mount_transparency::{
    judge_writable_mount_refusal_of_a_crash_state, refusal_reason_on_the_d23_list,
    DiskAfterTheRefusal, ReadFaultsInTheCrashState, ReadOnlyReadBackOfTheCrashImage,
    RefusalConditionNotMet, RefusalReasonOnTheD23List, WritableMountRefusalVerdict,
};
use singlefs_harness::reference_filesystem_translation::{
    crash_moment_of_a_segment_closed_at, segment_closing_stream_indexes,
    ReferenceFileSystemCrashMoment,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, sha256_hexadecimal, DIGEST_BYTES};
use singlefs_harness::{
    RecordedEntrySpan, RecordedOperationKind, RecordedPublishEntry, RetainedOperation, SharedStream,
};

// ───────────────────────────── 本地常量（登记 5.3、S1） ─────────────────────────────
//
// 下面这一批与 `e170_o3_spec_executor_export.rs`「本地常量（登记 5.7、S1）」那一节同一批、同样的值与出处注释；`baseline` 开头与 crates 的同名公开常量或现算值回比，
// 不直接引（决定几何与工作量的数写成本地常量：引了就等于装置与实现共用一个数，两边一起错而逐格相等）。
// 这些常量只在基线那一趟回比：坏实现改了写流，这些数在坏实现那一趟本来就会变，照数记进产物。

/// 盘数：`tests/common/mod.rs` 的 `formatted_recorded_images` 建两块盘（`for device_number in 0..2u32`）。
const LOCAL_DEVICE_COUNT: usize = 2;
/// 每块盘的字节数：`tests/common/mod.rs` 的 `IMAGE_BYTES = 4 << 30`。
const LOCAL_DEVICE_BYTES: u64 = 4_294_967_296;
/// 物理块与 io_min：`tests/common/mod.rs` 里 `physical_block_size: 512`、`minimum_input_output_bytes: 512`。
const LOCAL_PHYSICAL_BLOCK_BYTES: u32 = 512;
const LOCAL_MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
/// 导出格式的扇区宽（登记 5.3 `sector_bytes`（512））：与 `singlefs_harness::memory_pool::SECTOR_BYTES` 回比。
const LOCAL_SECTOR_BYTES: u64 = 512;
/// 固定 fsid：`tests/common/mod.rs` 的 `E142_FILESYSTEM_IDENTIFIER`。
const LOCAL_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
/// 固定写时刻：`tests/common/mod.rs` 的 `FIXED_WRITE_TIME_SECONDS = 1_788_000_000`。
const LOCAL_FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 第一个文件的字节数：`tests/common/mod.rs` 的 `FILE_BYTES = 3000`；内容 `index % 251`（`file_content`）。
const LOCAL_FIRST_FILE_BYTES: usize = 3000;
const LOCAL_FIRST_FILE_CONTENT_MODULUS: usize = 251;

/// 流一的段长：`tests/crash_enumeration_new_pool_file_creation_stream.rs` 的 `prepare` 里的段长断言。
const LOCAL_FIRST_STREAM_SEGMENT_LENGTHS: [usize; 11] = [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
/// 流一的写数：同一份文件 `prepare` 里的写数断言（取号 2 + 暖机两次各 5 + A 29）。
const LOCAL_FIRST_STREAM_WRITES: usize = 41;
/// 流一的根槽写数：暖机两次各一条 + A 一条。
const LOCAL_FIRST_STREAM_ROOT_WRITES: usize = 3;
/// 流一全量与快档的状态数：同一份文件的 `FULL_STATES`、`QUICK_TIER_STATES`。
const LOCAL_FIRST_STREAM_FULL_STATES: u64 = 16_777_260;
const LOCAL_FIRST_STREAM_QUICK_TIER_STATES: u64 = 46;
/// 流一每次写只取两态时的全量闭式：同一份文件的 `FULL_STATES_WITH_TWO_STATES_PER_WRITE`（单测拿段长表回算它）。
const LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM: u64 = 16_777_240;

/// 流二的段长（78 段）：取今天 crates 现算的那一份，即这个装置 `baseline` 的 S1 `fixed_script_through_e_then_unmount_segment_lengths`
/// 的 `crates=`（`research/results/e170-r2-o3-spec-executor-2026-10-04-s2b-today-export.out` 里 `step=z5`、`check=fixed_script_through_e_then_unmount_segment_lengths` 那一行）。与
/// `tests/crash_enumeration_fixed_script_stream.rs` 的 `prepare` 里「固定脚本到 E 再正常卸载的段序列」那条断言只差第 33 段（回退 D 那次向前发布）：20 → 24，
/// 多出的 4 次写是那次发布追加的谱系记录（D18（块里携带什么信息） 已定项 24 第 5 条）。
const LOCAL_SECOND_STREAM_SEGMENT_LENGTHS: [usize; 78] = [
    2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
    16, // 取号到写行之后的暖机
    2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2,
    28, // C、回退 D、覆盖写
    2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1,
    2, // 抬 F、E、卸载
    16, 2, 1, 2, // 卸载第二次空发布与 txg 19 的轮换
];
/// 流二的写数：S1 `fixed_script_through_e_then_unmount_writes` 的 `crates=481`（同一份产物第 439 行）；测试文件第 448 行
/// `191 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21)` = 477 不含回退 D 那次的 4 次谱系记录写。
const LOCAL_SECOND_STREAM_WRITES: usize = 481;
/// 流二的根槽写数：同一份文件第 482 行（`=> 19`）。
const LOCAL_SECOND_STREAM_ROOT_WRITES: usize = 19;
/// 流二全量的状态数：S1 `fixed_script_through_e_then_unmount_full_states` 的 `crates=1678377204`（同一份产物第 443 行；
/// 测试文件第 100 行 `FULL_STATES_THROUGH_THE_UNMOUNT` 还是 1_662_648_564，差 2^24 − 2^20）。快档：测试文件第 114 行
/// `QUICK_TIER_STATES_THROUGH_THE_UNMOUNT`，与 S1 `crates=278` 相同。
const LOCAL_SECOND_STREAM_FULL_STATES: u64 = 1_678_377_204;
const LOCAL_SECOND_STREAM_QUICK_TIER_STATES: u64 = 278;
/// 流二每次写只取两态时的全量闭式：S1 `fixed_script_through_e_then_unmount_two_state_closed_form` 的 `crates=1678377089`
/// （同一份产物第 445 行；测试文件第 96 行还是 1_662_648_449）。单测拿段长表回算它。
const LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM: u64 = 1_678_377_089;
/// E 的数据单元落点：同一份文件第 318 行（`50176`）。
const LOCAL_SECOND_STREAM_E_DATA_UNIT_SLOT: u64 = 50_176;
/// 抬 F 推的空发布数：同一份文件第 306 行（`raised.publishes.len(), 2`）。
const LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES: usize = 2;
/// 卸载推的两次空发布的 txg：同一份文件第 336 行（`vec![CheckpointTxg(18), CheckpointTxg(19)]`）。
const LOCAL_SECOND_STREAM_UNMOUNT_TXGS: [u64; 2] = [18, 19];

// 流三（登记第一节读法 2、5.3「要读 crates/ 常量的量」）：没有用例文件可抄，取本装置 `stream-constants` 模式在冻结之前的主工作区上
// 现算的那一份（`crates=` 那一栏；2026-10-04 那一次的输出与命令在 E171 第 1 次重跑第一段报告「本地常量怎么来的」一节），之后每一趟基线与
// crates 现算的回比（S1）。
// 流三的段长表、写数、全量状态数与闭式在登记修订 6 第 6.2 节第 1 条装上根环表之后重取（txg 26 起照可再分配谓词回收，
// 分配记录跟着变）：2026-10-05 草稿副本上 `stream-constants` 的 `crates=` 那一栏，新旧对照在登记修订 7。
// 段长表另按登记修订 12 重钉（出处在段长表自己的注释里）；写数、全量状态数与闭式仍是修订 7 那一份。
// K 另有推的值 234（登记第七节 A7：删完之后 inode 树没有记录、一容器 233 条），不进回比，照实测记。

/// 一片 inode 叶容器装几条记录：`crates/singlefs-format/src/lib.rs` 的 `INODE_LEAF_RECORDS = 233`
/// （D8（核心索引结构） 已定项 6「一容器 233 条」，⌊(32768 − 136) ÷ 140⌋）。
const LOCAL_INODE_LEAF_RECORDS: u64 = 233;
/// 每块盘的系统配置槽数与一槽的字节数（读法 8 (i)「少于两槽」的门槛、阳性对照「见证」重编的槽长）：
/// `crates/singlefs-format/src/lib.rs` 的 `SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE = 2`、`SYSTEM_CONFIGURATION_SLOT_BYTES = 4096`
/// （D22（单元原子性怎么合成） 已定项 8「每盘两槽」）。
const LOCAL_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE: usize = 2;
const LOCAL_SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096;
/// 流三的段长（959 段）：取今天 crates 现算的那一份，即这个装置 `stream-constants` 的 S1 `first_file_unlinked_then_inodes_across_one_leaf_segment_lengths`
/// 的 `crates=`（`research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-13-s1-stream-constants-revision-12-before-the-repin.out` 里那一行）。
/// 与登记修订 7 那一份（2026-10-05 草稿副本的 `crates=`）只差下标 15（20 → 24）与下标 43（24 → 20），段数、写数、全量状态数与闭式不变；
/// 为什么变、哪一处只在副本上核过，见登记修订 12。写数、根槽写数在下面两行。
const LOCAL_THIRD_STREAM_SEGMENT_LENGTHS: &[usize] = &[
    2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 26, 2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1,
    2, 20, 2, 1, 2, 20, 2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2,
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
    1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1, 2, 20, 2, 1,
    2,
];
const LOCAL_THIRD_STREAM_WRITES: usize = 5980;
const LOCAL_THIRD_STREAM_ROOT_WRITES: usize = 240;
/// 流三全量、快档的状态数与每次写只取两态时的闭式。
const LOCAL_THIRD_STREAM_FULL_STATES: u64 = 362_809_947;
const LOCAL_THIRD_STREAM_QUICK_TIER_STATES: u64 = 3127;
const LOCAL_THIRD_STREAM_TWO_STATE_CLOSED_FORM: u64 = 362_808_742;
/// 流三的 K：叶容器第一次变成两片的那一次建 inode 发布是第几次（从 1 数）；流三建 K + 1 次。
const LOCAL_THIRD_STREAM_SPLIT_PUBLISH: usize = 234;
/// 流三删第一个文件那一段的调用数：意图一次加走空的步数（第一个文件 3000 字节、一个数据单元，推的是意图一次加最后一批一次）。
const LOCAL_THIRD_STREAM_UNLINK_CALLS: usize = 2;
/// 流三建 inode 的次数上限：叶容器到这么多次还没变成两片，就停下记 S1 不过（防跑不完；233 条一片，两倍余量）。
const THIRD_STREAM_INODE_CREATION_LIMIT: usize = 2 * 233 + 2;
/// 流三走空 `deleted_inodes` 的步数上限（第一个文件一个数据单元，意图之后一步就走空；余量防跑不完）。
const THIRD_STREAM_UNLINK_STEP_LIMIT: usize = 16;

/// Q5 那一形（读法 7）里已确认的那次调用与在途的那次调用（流二，从 0 数：第 0、1 次是取号、暖机）。
const CONFIRMED_LOSS_CONFIRMED_CALL: usize = 2;
const CONFIRMED_LOSS_IN_FLIGHT_CALL: usize = 3;

// Q5 那一形在基线上的几样（登记第一节读法 7、5.3）：同上，取 `stream-constants` 现算的那一份。下标都是流二 mkfs 之后录制流里的下标。
/// 第 2 次调用（第一个文件那次发布）里删掉的屏障。
const LOCAL_CONFIRMED_LOSS_REMOVED_BARRIERS: &[usize] = &[50, 51, 55, 56];
/// 第 2 次调用写根槽的那次 FUA 写（改成普通写）。
const LOCAL_CONFIRMED_LOSS_ROOT_WRITE: usize = 52;
/// 截断处：第 3 次调用发出的第一道屏障（主取样点截在它之前）与第 2 次调用的返回点（方向相反的取样点）。
const LOCAL_CONFIRMED_LOSS_TRUNCATION: usize = 81;
const LOCAL_CONFIRMED_LOSS_REVERSE_TRUNCATION: usize = 57;
/// 主取样点末段的写数、可撕裂写数、全枚举态数（末段每个写的取法数之积，A10）。
const LOCAL_CONFIRMED_LOSS_LAST_SEGMENT_WRITES: usize = 29;
const LOCAL_CONFIRMED_LOSS_LAST_SEGMENT_TEARABLE_WRITES: usize = 2;
const LOCAL_CONFIRMED_LOSS_STATES: u64 = 1_207_959_552;
// 主取样点按等价类判（登记修订 2 第 2.1 节候选二、修订 3）的起点分法（类核对 ③ 细分之前）：键里是末段中不属于第 3 次调用的写（K），
// 类里是第 3 次调用的写（C）。值照第 2.1 节「末段是什么」与「Q5-等价类」两段：K 5 个写、取法 2³ × 3² = 72 种，C 24 个写，
// 类数 72 × 2 = 144，「C 一个没落」那一类各 1 态、「C 至少落一个」那一类各 2²⁴ − 1 = 16 777 215 态，要照原路径判的态 72 + 72 = 144。
/// K 的取法数。
const LOCAL_CONFIRMED_LOSS_KEY_CHOICES: u64 = 72;
/// C 的写数。
const LOCAL_CONFIRMED_LOSS_IN_FLIGHT_WRITES: usize = 24;
/// 类数。
const LOCAL_CONFIRMED_LOSS_CLASSES: u64 = 144;
/// 「C 至少落一个」那一类的态数。
const LOCAL_CONFIRMED_LOSS_STATES_OF_A_CLASS_WITH_IN_FLIGHT_WRITES_LANDED: u64 = 16_777_215;
/// 要照原路径判的态数（每种取法的代表加「C 全落」那一态）。
const LOCAL_CONFIRMED_LOSS_JUDGED_STATES: u64 = 144;
/// 「等价类对拍」那一形末段里属于第 3 次调用的写数（修订 5 第 5.2 节丙：k = 6）。
const LOCAL_EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES: usize = 6;
/// 「等价类对拍」那一形的态数：末段各写取法数之积，K 的 72 种取法 × C 那 6 个整写各 2 种（修订 5 第 5.2 节，72 × 2⁶ = 4 608）。
const LOCAL_EQUIVALENCE_PAIRING_STATES: u64 =
    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2 * 2 * 2 * 2;
/// c1…c6 的镜像对数（修订 5 第 5.2 节：两两同偏移、同长度、分属盘 0 与盘 1，三对的偏移互不相同）。
const LOCAL_EQUIVALENCE_PAIRING_MIRRORED_PAIRS: usize = 3;
/// 阳性对照「判态路 panic 单列」`pool_checker` 那条注入的态数：c3 落、c4 没落，K 的 72 种取法 × c1、c2、c5、c6 各 2 种（72 × 2⁴ = 1 152）。
const LOCAL_PIECE_PANIC_CONTROL_POOL_CHECKER_STATES: u64 =
    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2 * 2;
/// 同上 `ignored_recovery` 那条：c3 没落、c4 落（72 × 2⁴ = 1 152）。
const LOCAL_PIECE_PANIC_CONTROL_IGNORED_RECOVERY_STATES: u64 =
    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2 * 2;
/// 同上 `record_checker` 那条：c5 落、c6 没落、(c3,c4) 同落或同不落，72 × c1、c2 各 2 种 × 2（72 × 2² × 2 = 576）。
const LOCAL_PIECE_PANIC_CONTROL_RECORD_CHECKER_STATES: u64 =
    LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2 * 2;
/// `confirmed-loss-form` 一趟判得完的态数上限：每态推 5–50 毫秒，上限这么多约是一天以内。方向相反的取样点全枚举，超过的不跑、
/// 打一行 `not_run` 退 3 交主 agent；主取样点按等价类判（登记修订 2 第 2.1 节候选二、修订 3），这个上限管的是类核对 ③ 细分之后
/// 要照原路径判的态数（每种取法的代表加「C 全落」那一态），超过走停机 S13（主取样点全枚举是 1 207 959 552 态，第 1 次重跑第一段跑不了）。
const CONFIRMED_LOSS_STATE_LIMIT: u64 = 10_000_000;

/// 取样：批 1 每段内点的分母、批 2 的分母、批 3 的分母（登记 5.4）。
const FIRST_BATCH_DIVISOR: u64 = 7;
const SECOND_BATCH_DIVISOR: u64 = 14;
const THIRD_BATCH_DIVISOR: u64 = 23;
/// 批 1 在带撕裂写的段里另取几个带「1」的状态、每段最多扫几个序号（登记 5.4）。
const TORN_STATES_PER_SEGMENT: usize = 4;
const TORN_SCAN_ORDINALS_PER_SEGMENT: u64 = 100_000;

/// 卸载那次调用在 `calls.tsv` 里的标签：模型照 `IdealModel::answer_unmount` 答（登记第一节读法 1 ①），与别的调用一样对拍。
const UNMOUNT_CALL_LABEL: &str = "unmount";

/// 三条流的名字（登记 5.3 的目录名；流三是第 1 次重跑加的，读法 2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum StreamName {
    NewPoolFileCreation,
    FixedScriptThroughEThenUnmount,
    FirstFileUnlinkedThenInodesAcrossOneLeaf,
}

impl StreamName {
    fn directory_name(self) -> &'static str {
        match self {
            Self::NewPoolFileCreation => "new_pool_file_creation",
            Self::FixedScriptThroughEThenUnmount => "fixed_script_through_e_then_unmount",
            Self::FirstFileUnlinkedThenInodesAcrossOneLeaf => {
                "first_file_unlinked_then_inodes_across_one_leaf"
            }
        }
    }
    /// `--streams` 里写的号：1、2、3。
    fn number(self) -> u8 {
        match self {
            Self::NewPoolFileCreation => 1,
            Self::FixedScriptThroughEThenUnmount => 2,
            Self::FirstFileUnlinkedThenInodesAcrossOneLeaf => 3,
        }
    }
}

const STREAM_NAMES: [StreamName; 3] = [
    StreamName::NewPoolFileCreation,
    StreamName::FixedScriptThroughEThenUnmount,
    StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf,
];

/// `--streams 1,2,3` 那样的号表换成流名（按号升序、去重）；认不出的号交回 `None`。
fn stream_names_of_the_numbers(text: &str) -> Option<Vec<StreamName>> {
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

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 同一份文件第 131–143 行。
const THIRD_FILE_BYTES: usize = 2500;

fn later_content(seed: usize) -> Vec<u8> {
    (0..3000 + seed)
        .map(|index| u8::try_from((index * 5 + seed) % 251).expect("小于 256"))
        .collect()
}

fn third_content() -> Vec<u8> {
    (0..THIRD_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 11) % 253).expect("小于 256"))
        .collect()
}

/// 第一个文件的内容按本地常量现算，与 `common::file_content()` 回比（S1）。
fn local_first_file_content() -> Vec<u8> {
    (0..LOCAL_FIRST_FILE_BYTES)
        .map(|index| {
            u8::try_from(index % LOCAL_FIRST_FILE_CONTENT_MODULUS).expect("模 251 小于 256")
        })
        .collect()
}

// ───────────────────────────── 驱动方记的调用（calls.tsv、call-outcomes.tsv） ─────────────────────────────

/// 一次调用的结局：成功；实现交回一个错误（成员名取 `Debug` 文本的第一个标识符）；实现 panic（实现自崩，登记第一节读法 5）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum CallOutcome {
    Succeeded,
    ReturnedError(String),
    Panicked(String),
    /// Q5 那一形（读法 7）里截断处还没返回的那次调用：实现在装置里照常跑完了，那一形的写流截在它发出的第一道屏障之前，它在途。
    InFlightAtTheTruncation,
}

impl CallOutcome {
    /// `calls.tsv` 最后一列：E170（O3规约执行器第一版） 登记 5.3 写的 `ok` / `err`；panic 那次调用也写 `err`
    /// （第 1 次重跑登记第一节读法 5：panic 与报错同样截断照判，那份格式只有这两个词）。
    fn calls_file_word(&self) -> &'static str {
        match self {
            Self::Succeeded => "ok",
            Self::ReturnedError(_) | Self::Panicked(_) | Self::InFlightAtTheTruncation => "err",
        }
    }
    /// `call-outcomes.tsv` 里的结局列：成功是 `ok`，错误是 `err:<成员>`，panic 是 `panic`（消息另写结果行）。
    fn outcome_word(&self) -> String {
        match self {
            Self::Succeeded => "ok".to_string(),
            Self::ReturnedError(member) => format!("err:{member}"),
            Self::Panicked(_) => "panic".to_string(),
            Self::InFlightAtTheTruncation => "in_flight".to_string(),
        }
    }
}

/// 理想模型对一次调用的判定（O1-对拍，登记 5.3「调用与模型的对应」那张表）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum ModelVerdictOfCall {
    /// 模型这一步不交答：取号、暖机（两次都返回之后一起推一步）。
    NotModelled,
    /// 对得上，模型往前走。
    Agrees,
    /// 对不上的第一格；模型停在这里，此后的调用不比（读法 6）。
    Disagrees {
        aspect: &'static str,
        model_answer: String,
        implementation_answer: String,
    },
    /// 前面有一次对不上（或前面有一次调用出了错或 panic），这次不比。
    SkippedAfterTheModelStopped,
    /// 这次调用 panic：实现自崩、不比，不红不绿（读法 5）；模型停在这次调用。
    NotComparedBecauseTheImplementationCrashed,
    /// 调用成功了，而胶水造不出交给模型的观测（`model_comparison::model_unit_role` 里模型还不罩的几种单元在这一版里出现，胶水 panic）：
    /// 不比、模型停在这次调用。读法 6 的三处来源之外的一处，停机 S5 记它（第一段流三跨一片叶那一次撞上过；理想模型那件修理之后
    /// 胶水按片记 inode 叶容器的角色、不再断言，登记修订 2 第 2.2 节：合入后的树上再出现就是 S5）。
    ModelCannotTakeTheObservation { what: String },
}

impl ModelVerdictOfCall {
    fn word(&self) -> String {
        match self {
            Self::NotModelled => "not_modelled".to_string(),
            Self::Agrees => "agrees".to_string(),
            Self::Disagrees { aspect, .. } => format!("disagrees:{aspect}"),
            Self::SkippedAfterTheModelStopped => "skipped_after_the_model_stopped".to_string(),
            Self::NotComparedBecauseTheImplementationCrashed => {
                "implementation_crashed_not_compared".to_string()
            }
            Self::ModelCannotTakeTheObservation { .. } => {
                "model_cannot_take_the_observation".to_string()
            }
        }
    }
    /// O1-对拍判过这次调用（读法 5「可判的调用」：`agrees` 或 `disagrees`；`not_modelled` 不算）。
    fn was_judged_by_the_comparison(&self) -> bool {
        match self {
            Self::Agrees | Self::Disagrees { .. } => true,
            Self::NotModelled
            | Self::SkippedAfterTheModelStopped
            | Self::NotComparedBecauseTheImplementationCrashed
            | Self::ModelCannotTakeTheObservation { .. } => false,
        }
    }
}

/// 一次成功调用的交回值（读法 4）：`{:?}` 文本的 sha256，与顶层字段逐个的 (字段名, sha256)（字段名照 `{:?}` 文本切出，
/// 元组成员与没有名字的写 `#<序>`）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReturnValueDigest {
    whole: String,
    fields: Vec<(String, String)>,
}

/// 驱动方调的一次 API：标签、它发出的第一条操作的流序下标、返回点（返回时录制流的长度，都从 mkfs 之后数）、结局、模型的判定、
/// 成功时的交回值摘要、这次调用之后装置手里由实现交出的对象各自的 (名字, `{:?}` 的 sha256)（读法 4，只记、不进触达）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedCall {
    label: &'static str,
    first_operation: usize,
    return_point: usize,
    outcome: CallOutcome,
    model: ModelVerdictOfCall,
    return_value: Option<ReturnValueDigest>,
    memory_objects: Vec<(&'static str, String)>,
    /// 发布调用交给实现的内容（RefFS 子列要它：建第一个文件或覆盖成哪几个字节，读法 1）；别的调用没有。
    published_content: Option<PublishedContent>,
    /// 胶水交给模型的观测 `{:?}` 的 sha256（几份按次序用 `+` 接起来）；这次调用没交观测（取号、暖机、panic）是 `None`。
    observation_sha256: Option<String>,
}

/// 发布调用改了什么内容：第一个文件建成什么、覆盖成什么。
#[derive(Clone, Debug, PartialEq, Eq)]
enum PublishedContent {
    FirstFile(Vec<u8>),
    Overwrite(Vec<u8>),
}

/// 切 `{:?}` 文本的顶层字段：第一个开括号（`{`、`(`、`[`）之内、深度 1 上按逗号切，引号里的不算；每一段有 `名字: ` 起头的取名字，
/// 没有的写 `#<序>`。没有括号的（单元成员、数）只有一段 `#0`，就是整份文本。
fn top_level_fields_of_debug_text(text: &str) -> Vec<(String, String)> {
    let Some(open) = text.find(['{', '(', '[']) else {
        return vec![("#0".to_string(), text.to_string())];
    };
    let mut pieces: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut inside_quotes = false;
    let mut escaped = false;
    for character in text[open..].chars() {
        if inside_quotes {
            current.push(character);
            match (escaped, character) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => inside_quotes = false,
                (false, _) => {}
            }
            continue;
        }
        match character {
            '{' | '(' | '[' => {
                depth += 1;
                if depth > 1 {
                    current.push(character);
                }
            }
            '}' | ')' | ']' => {
                depth = depth.saturating_sub(1);
                if depth >= 1 {
                    current.push(character);
                } else {
                    break;
                }
            }
            ',' if depth == 1 => pieces.push(std::mem::take(&mut current)),
            '"' => {
                inside_quotes = true;
                current.push(character);
            }
            _ => current.push(character),
        }
    }
    if !current.trim().is_empty() {
        pieces.push(current);
    }
    pieces
        .iter()
        .enumerate()
        .map(|(position, piece)| {
            let piece = piece.trim();
            let named = piece.split_once(": ").filter(|(name, _)| {
                !name.is_empty()
                    && name
                        .chars()
                        .all(|character| character.is_alphanumeric() || character == '_')
            });
            match named {
                Some((name, value)) => (name.to_string(), value.to_string()),
                None => (format!("#{position}"), piece.to_string()),
            }
        })
        .collect()
}

/// 交回值的摘要：整份 `{:?}` 文本与顶层字段逐个取 sha256。
fn return_value_digest_of_debug_text(text: &str) -> ReturnValueDigest {
    ReturnValueDigest {
        whole: sha256_hexadecimal(text.as_bytes()),
        fields: top_level_fields_of_debug_text(text)
            .into_iter()
            .map(|(name, value)| (name, sha256_hexadecimal(value.as_bytes())))
            .collect(),
    }
}

/// 交回值的 `{:?}` 文本：`Debug` 有的照打。
fn debug_text<Value: std::fmt::Debug>(value: &Value) -> String {
    format!("{value:?}")
}

/// `mount::Unmounted` 与它里面的 `UnmountRaisedTheFloor` 不带 `Debug`：照 `#[derive(Debug)]` 会打出的样子逐个字段拼（字段名与次序照
/// `crates/singlefs-core/src/mount.rs` 里的声明），每个字段的值用它自己的 `Debug`。
fn unmounted_debug_text(unmounted: &Unmounted) -> String {
    match unmounted {
        Unmounted::FloorRaisedToTheCurrentVersion(raised) => format!(
            "FloorRaisedToTheCurrentVersion(UnmountRaisedTheFloor {{ rollback_floor: {:?}, system_configuration_writes_before_the_first_publish: {:?}, publishes: {:?}, reclaimed: {:?}, abandoned_roots_unreadable: {:?} }})",
            raised.rollback_floor,
            raised.system_configuration_writes_before_the_first_publish,
            raised.publishes,
            raised.reclaimed,
            raised.abandoned_roots_unreadable
        ),
        Unmounted::NothingWrittenOnAVersionWithoutFile { current } => {
            format!("NothingWrittenOnAVersionWithoutFile {{ current: {current:?} }}")
        }
    }
}

/// 一条停机条款 S1 的回比：本地值与 crates 那一侧的值，两边按文本比。
#[derive(Clone, Debug, PartialEq, Eq)]
struct StopCheck {
    name: String,
    local: String,
    crates: String,
}

impl StopCheck {
    fn new(name: &str, local: impl std::fmt::Debug, crates: impl std::fmt::Debug) -> Self {
        Self {
            name: name.to_string(),
            local: format!("{local:?}"),
            crates: format!("{crates:?}"),
        }
    }
    fn holds(&self) -> bool {
        self.local == self.crates
    }
}

/// 阳性对照要改的那一处（登记 5.2 那张表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PositiveControl {
    /// 流二第一次 `publish` 的观测里那一版的文件内容末字节翻一位，再交 `judge_and_advance`。
    ComparisonOfTheCalls,
    /// 流二卸载那一次交给模型的观测里，第一条根的 F 加一（16-2 的观测 `observed_unmount` 是那几次带新 F 的空发布的根，
    /// `ObservedRoot::rollback_floor` 就是卸载要比的新 F；第 1 次重跑登记第十二节修订写明）。
    ComparisonOfTheUnmount,
    /// 每条流批 1 里第一个「实现读回了文件」的态，读回内容末字节翻一位，再交 `crash_recovery_disagreement`。
    CrashJudge,
    /// 同一个态（每条流批 1 里第一个读回了文件的态），交给 RefFS 的读回内容末字节翻一位。
    ReferenceFileSystem,
    /// 流一全持久态叠已知坏镜像族第一条，再交 `check_pool_image`。
    PoolChecker,
    /// 流二第 2 次调用（从 0 数）的交回值在记之前复制一份、把那一版根的 checkpoint_txg 加一（只改记下的那份，不改交给胶水的）。
    ReturnValue,
    /// 流二第 2 次调用之后记分配器摘要之前，在副本上多分配一槽（只改记下的那份）。
    MemoryObject,
    /// 流二第 3 次调用（从 0 数）的闭包里先 panic（装置这一侧注入，不改实现）。
    PanicInTheCall,
    /// 基线流一全持久态：每块盘两槽系统配置的 journal_tail 改成所选那一版末条记录的计数器加一、重编、自证过，再起可写挂载探针。
    Witness,
}

impl PositiveControl {
    fn parse(word: &str) -> Option<Self> {
        match word {
            "O1-对拍" => Some(Self::ComparisonOfTheCalls),
            "O1-对拍-卸载" => Some(Self::ComparisonOfTheUnmount),
            "O1-崩溃判官" => Some(Self::CrashJudge),
            "RefFS" => Some(Self::ReferenceFileSystem),
            "O2" => Some(Self::PoolChecker),
            "交回值" => Some(Self::ReturnValue),
            "内存对象" => Some(Self::MemoryObject),
            "panic" => Some(Self::PanicInTheCall),
            "见证" => Some(Self::Witness),
            _ => None,
        }
    }
}

/// 阳性对照「交回值」「内存对象」改的那一次调用、「panic」注入的那一次调用（流二，从 0 数）。
const RETURN_VALUE_CONTROL_CALL: usize = 2;
const PANIC_CONTROL_CALL: usize = 3;

/// 关掉的那条 oracle（Q3-核，登记第一节读法 9）：关掉的列不算、写成 `off`，别的两列照常。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DisabledOracle {
    ComparisonAndCrashJudge,
    PoolChecker,
}

/// 一次跑的开关：阳性对照、关一条、可写挂载探针（Q4，读法 8）。
#[derive(Clone, Copy, Debug, Default)]
struct RunSwitches {
    positive_control: Option<PositiveControl>,
    disabled_oracle: Option<DisabledOracle>,
    writable_mount_probe: bool,
}

/// 实现 panic 的消息：`String` 或 `&str`，别的写 `non_text_panic`。
fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_else(|| "non_text_panic".to_string())
}

/// 错误值 `Debug` 文本的第一个标识符（`Variant(..)`、`Variant { .. }`、`Variant`）：成员名。
fn error_member_of(error: &dyn std::fmt::Debug) -> String {
    let text = format!("{error:?}");
    text.chars()
        .take_while(|character| character.is_alphanumeric() || *character == '_')
        .collect()
}

/// 一条流驱动时手里的东西：录制流、调用表、理想模型与它提交过的版本目录。每次调用实现的入口都经 [`StreamDriver::call`]：
/// 记返回点、结局与交回值，错误或 panic 之后这条流不再往下驱动（两种同样截断照判，读法 5）。
struct StreamDriver {
    name: StreamName,
    stream: SharedStream,
    mkfs_operation_count: usize,
    switches: RunSwitches,
    calls: Vec<RecordedCall>,
    model: IdealModel,
    /// 每一步之后把模型根环里的每一版并进来（崩溃注入的做法）：O1-崩溃判官拿它问「恢复到的这一版是不是模型提交过的」。
    committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    /// 模型不再往前走的那次调用：第一次对不上的、出了错或 panic 的调用（读法 6）。
    model_stopped_at_call: Option<usize>,
    /// 驱动没出错、也没 panic 却走不下去的那一处（可写挂载交回的现行版本不带文件，后面的脚本接不上）：这条流截在那里照判。
    stopped_without_an_error: Option<String>,
    /// 从这一次调用（含）起模型不比、不推：Q5 那一形只让模型跟到第 2 次调用（读法 7「模型跟到第 2 次调用」），第 3 次调用在途。
    model_not_asked_from_call: Option<usize>,
    halted: bool,
    comparison_control_applied: bool,
}

impl StreamDriver {
    fn new(
        name: StreamName,
        stream: SharedStream,
        mkfs_operation_count: usize,
        switches: RunSwitches,
    ) -> Self {
        let parameters = common::parameters();
        let model = IdealModel::after_make_filesystem_on_a_journal_ring_of(
            ModelPoolGeometry {
                devices: (0..LOCAL_DEVICE_COUNT)
                    .map(|device_number| {
                        ModelDeviceIdentity(u32::try_from(device_number).expect("设备号"))
                    })
                    .collect(),
                device_size_in_bytes: LOCAL_DEVICE_BYTES,
            },
            parameters.geometry.journal_ring_bytes,
        );
        let mut driver = Self {
            name,
            stream,
            mkfs_operation_count,
            switches,
            calls: Vec::new(),
            model,
            committed_versions: BTreeMap::new(),
            model_stopped_at_call: None,
            stopped_without_an_error: None,
            model_not_asked_from_call: None,
            halted: false,
            comparison_control_applied: false,
        };
        driver.merge_committed_versions();
        driver
    }

    /// O1 开着：关一条（Q3-核）关掉 O1 时模型一步都不推、对拍与崩溃判官都不算；Q5 那一形里第 3 次调用起模型也不问。
    fn model_is_enabled(&self) -> bool {
        self.switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge)
            && self
                .model_not_asked_from_call
                .is_none_or(|first_call_not_asked| self.calls.len() <= first_call_not_asked)
    }

    fn operations_so_far(&self) -> usize {
        self.stream.operation_count() - self.mkfs_operation_count
    }

    fn merge_committed_versions(&mut self) {
        for (key, file) in self.model.committed_versions() {
            self.committed_versions.insert(key, file);
        }
    }

    fn stop_the_model_at(&mut self, call_index: usize) {
        self.model_stopped_at_call.get_or_insert(call_index);
    }

    /// 调一次实现的入口并记一行：调之前与返回时各读一次流长；成功时照 `describe` 取交回值的 `{:?}` 文本记摘要（读法 4）。
    /// 交回 `Err(Some(错误))` 是实现交回了错误（调用方照错误造拒绝的观测），`Err(None)` 是 panic：这次调用记 `panic`、O1-对拍记
    /// 「实现自崩、不比」，模型停在这里，这条流截在这次调用、照导出照判（读法 5）。阳性对照「panic」在流二第 3 次调用的闭包里先 panic。
    fn call<Output, Failure: std::fmt::Debug>(
        &mut self,
        label: &'static str,
        describe: fn(&Output) -> String,
        run: impl FnOnce() -> Result<Output, Failure>,
    ) -> Result<Output, Option<Failure>> {
        let first_operation = self.operations_so_far();
        let call_index = self.calls.len();
        let inject_a_panic = self.switches.positive_control
            == Some(PositiveControl::PanicInTheCall)
            && self.name == StreamName::FixedScriptThroughEThenUnmount
            && call_index == PANIC_CONTROL_CALL;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert!(
                !inject_a_panic,
                "阳性对照 panic：装置在流二第 3 次调用的闭包里先 panic"
            );
            run()
        }));
        let return_point = self.operations_so_far();
        let (outcome, model, return_value, returned) = match result {
            Ok(Ok(output)) => {
                let digest = return_value_digest_of_debug_text(&describe(&output));
                (
                    CallOutcome::Succeeded,
                    ModelVerdictOfCall::NotModelled,
                    Some(digest),
                    Ok(output),
                )
            }
            Ok(Err(failure)) => (
                CallOutcome::ReturnedError(error_member_of(&failure)),
                ModelVerdictOfCall::NotModelled,
                None,
                Err(Some(failure)),
            ),
            Err(payload) => (
                CallOutcome::Panicked(panic_text(&*payload)),
                ModelVerdictOfCall::NotComparedBecauseTheImplementationCrashed,
                None,
                Err(None),
            ),
        };
        if outcome != CallOutcome::Succeeded {
            self.halted = true;
            self.stop_the_model_at(call_index);
        }
        self.calls.push(RecordedCall {
            label,
            first_operation,
            return_point,
            outcome,
            model,
            return_value,
            memory_objects: Vec::new(),
            published_content: None,
            observation_sha256: None,
        });
        returned
    }

    /// 刚记的这次发布调用交给实现的内容（RefFS 子列用）。
    fn note_the_published_content_of_the_last_call(&mut self, content: PublishedContent) {
        let call_index = self.calls.len() - 1;
        self.calls[call_index].published_content = Some(content);
    }

    /// 刚记的这次调用成功了，而胶水造不出交给模型的观测：不比、模型停在这次调用（[`ModelVerdictOfCall::ModelCannotTakeTheObservation`]）。
    /// 模型已经停在更早的调用上、或 O1 关掉了，照 [`Self::judge`] 那样不动。
    fn note_that_the_model_cannot_take_the_observation(&mut self, what: String) {
        if !self.model_is_enabled() {
            return;
        }
        let call_index = self.calls.len() - 1;
        if self
            .model_stopped_at_call
            .is_some_and(|stopped| stopped < call_index)
        {
            self.calls[call_index].model = ModelVerdictOfCall::SkippedAfterTheModelStopped;
            return;
        }
        self.calls[call_index].model = ModelVerdictOfCall::ModelCannotTakeTheObservation { what };
        self.stop_the_model_at(call_index);
    }

    /// 阳性对照「交回值」：刚记的这次调用的交回值摘要换成改过的那份的（只改记下的那份）。
    fn replace_the_recorded_return_value_of_the_last_call(&mut self, text: &str) {
        let call_index = self.calls.len() - 1;
        self.calls[call_index].return_value = Some(return_value_digest_of_debug_text(text));
    }

    /// 刚记的这次调用之后装置手里的实现对象（名字, `{:?}` 文本）：记各自的 sha256。
    fn record_the_memory_objects_after_the_last_call(
        &mut self,
        objects: Vec<(&'static str, String)>,
    ) {
        let call_index = self.calls.len() - 1;
        self.calls[call_index].memory_objects = objects
            .into_iter()
            .map(|(name, text)| (name, sha256_hexadecimal(text.as_bytes())))
            .collect();
    }

    /// 取号、暖机两次都返回之后，模型推一步（`acquire_and_warm_up_in_the_make_filesystem_process`）；两次不比。
    fn advance_the_model_past_the_acquire_and_the_warm_up(&mut self) {
        if self.model_is_enabled() && self.model_stopped_at_call.is_none() {
            self.model
                .acquire_and_warm_up_in_the_make_filesystem_process();
            self.merge_committed_versions();
        }
    }

    /// 取号或暖机交回了错误（刚记的这次调用）：这两条流不带故障，理想模型对这两次调用只有一种走法（往前走，`model.rs` 里没有 `answer_*`
    /// 可问拒绝），所以实现拒了就是「模型说该成、实现拒了」，记成 O1-对拍的红（登记第一节读法 5 末句：一次调用返回错误而模型说该成功，
    /// 不算自崩，是 O1-对拍的红）。不记的话这次调用的 O1-对拍一栏留在 `not_modelled`、之后的态记不适用，一行这样的坏实现三列都不红
    /// （代码轮第三轮判决 K4-c）。`failure` 是 `None`（panic，实现自崩）不记；O1 关掉（Q3-核）时不记，那一列写 `off`。
    fn record_a_refusal_the_model_only_goes_forward_on<Failure: std::fmt::Debug>(
        &mut self,
        failure: Option<Failure>,
    ) {
        let Some(failure) = failure else {
            return;
        };
        if !self.model_is_enabled() {
            return;
        }
        let call_index = self.calls.len() - 1;
        self.calls[call_index].model = ModelVerdictOfCall::Disagrees {
            aspect:
                singlefs_harness::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
                    .name(),
            model_answer: "往前走".to_string(),
            implementation_answer: format!("拒了:{}", error_member_of(&failure)),
        };
        self.stop_the_model_at(call_index);
    }

    /// 拿刚记的这次调用的结局与模型的答案比（与 `history.rs` 的 `judge_by_model` 同一种比法：`answer` 出答案、`judge_and_advance` 比并推）。
    /// 对得上，模型往前走并把它根环里的版本并进目录；对不上，记第一格并让模型停在这次调用。同一次调用可以交几次观测
    /// （可写挂载之后准入推的抬 F）：前面一处对不上，后面不再比。
    fn judge(
        &mut self,
        answer: impl FnOnce(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
        mut observed: ObservedOutcome,
    ) {
        let call_index = self.calls.len() - 1;
        // 胶水造出的观测（阳性对照改之前那一份）记摘要：比对器拿它分「模型看见了而判同」与「交回值不同的字段胶水没交给模型」（第六节 Q2-观测）。
        // 同一次调用交几份观测（可写挂载之后准入推的抬 F）的，按交的次序接起来。
        let observation_digest = sha256_hexadecimal(debug_text(&observed).as_bytes());
        let recorded = &mut self.calls[call_index].observation_sha256;
        *recorded = Some(match recorded.take() {
            None => observation_digest,
            Some(earlier) => format!("{earlier}+{observation_digest}"),
        });
        if !self.model_is_enabled() {
            return;
        }
        if self
            .model_stopped_at_call
            .is_some_and(|stopped| stopped < call_index)
        {
            self.calls[call_index].model = ModelVerdictOfCall::SkippedAfterTheModelStopped;
            return;
        }
        if matches!(
            self.calls[call_index].model,
            ModelVerdictOfCall::Disagrees { .. }
        ) {
            return;
        }
        if self.switches.positive_control == Some(PositiveControl::ComparisonOfTheCalls)
            && self.name == StreamName::FixedScriptThroughEThenUnmount
            && self.calls[call_index].label == "publish"
            && !self.comparison_control_applied
        {
            if let ObservedOutcome::Succeeded(ObservedEffect::Publishes { roots, .. }) =
                &mut observed
            {
                if let Some(ObservedFile::Decoded(content)) =
                    roots.first_mut().map(|root| &mut root.file)
                {
                    if let Some(last_byte) = content.last_mut() {
                        *last_byte ^= 1;
                        self.comparison_control_applied = true;
                    }
                }
            }
        }
        let verdict =
            answer(&self.model).and_then(|answer| self.model.judge_and_advance(&answer, &observed));
        match verdict {
            Ok(_) => {
                self.calls[call_index].model = ModelVerdictOfCall::Agrees;
                self.merge_committed_versions();
            }
            Err(disagreement) => {
                self.calls[call_index].model = ModelVerdictOfCall::Disagrees {
                    aspect: disagreement.aspect.name(),
                    model_answer: disagreement.model_answer,
                    implementation_answer: disagreement.implementation_answer,
                };
                self.stop_the_model_at(call_index);
            }
        }
    }

    /// 实现交回错误时交给模型的观测：成员映射成的理由、拒之前做完几次发布、录制流在这一步里有没有多出写或屏障。
    fn refusal(
        &self,
        error: &dyn std::fmt::Debug,
        reason: singlefs_harness::model::ObservedRefusalReason,
        publishes_completed: usize,
        stream_length_before: usize,
        reported_ceiling: Option<singlefs_harness::model::ModelCheckpointTxg>,
        slot_still_bad: Option<singlefs_harness::model::ModelRingPosition>,
    ) -> ObservedOutcome {
        ObservedOutcome::Refused {
            member: error_member_of(error),
            reason,
            publishes_completed,
            wrote_anything: self.stream.operation_count() != stream_length_before,
            reported_ceiling,
            reported_root_ring_slot_still_bad_after_one_reread: slot_still_bad,
        }
    }
}

// ───────────────────────────── 驱动三条流（登记 5.3：不 `.expect`、跟着推模型） ─────────────────────────────

fn version(instance: u32, checkpoint_txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
        content,
    }
}

/// 一次调用之后装置手里由实现交出的对象（读法 4「内存对象」）：分配器、取到的实例代号、暖机输出、现行那一版、可写挂载的输出，
/// 各自的 `{:?}` 文本（没有的写 `None`）。次序固定，比对器按名字比。
fn memory_objects_of(
    allocator: &PoolAllocator,
    instance: Option<InstanceGeneration>,
    warm_up: Option<&WarmUpOutput>,
    current: Option<&TransactionOutput>,
    mount_output_text: Option<&str>,
) -> Vec<(&'static str, String)> {
    vec![
        ("allocator", debug_text(allocator)),
        ("instance", debug_text(&instance)),
        ("warm_up_output", debug_text(&warm_up)),
        ("current_version", debug_text(&current)),
        (
            "mount_output",
            mount_output_text.unwrap_or("None").to_string(),
        ),
    ]
}

/// 流三的一次发布（[`LiveStream::step_third_stream_publish`] 的 `publish`）：经 `PoolWriter` 调实现的发布函数，交回那一版或发布错误。
type ThirdStreamPublish<'closure> = dyn Fn(
        &mut PoolWriter<'_, common::Recorded>,
        &mut PoolAllocator,
        &TransactionOutput,
        InstanceGeneration,
    ) -> Result<TransactionOutput, singlefs_core::transaction::PublishError>
    + 'closure;

/// 一条流驱动时的池：只做过 mkfs 的两块文件镜像（`FormattedPool` 管镜像文件，丢掉就删）、分配器、现行那一版、崩溃点重放手写 oracle 的版本表。
/// 流一照抄 `tests/common/mod.rs` 的 `build_pool` 的取号、暖机、新池新建文件三步，流二照抄
/// `tests/crash_enumeration_fixed_script_stream.rs` 的 `prepare` 里 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支，
/// 流三接着流一删第一个文件、再一次建一个 inode 直到 inode 树的叶容器第一次变成两片、再多建一次（读法 2）；每一步都经 [`StreamDriver::call`]。
struct LiveStream {
    driver: StreamDriver,
    formatted: common::FormattedPool,
    allocator: PoolAllocator,
    instance: Option<InstanceGeneration>,
    warm_up: Option<WarmUpOutput>,
    output: Option<TransactionOutput>,
    /// 可写挂载交回的 `MountOutput` 的 `{:?}` 文本：算进之后每次调用的内存对象（读法 4）。
    mount_output_text: Option<String>,
    versions: Vec<PublishedVersion>,
    checks: Vec<StopCheck>,
    /// 暖机返回时整条录制流（含 mkfs）的长度。
    warm_up_operation_count: usize,
    /// 流三：删第一个文件那一段的调用数（意图一次加走空的每一步）、建 inode 发布的次数、叶容器第一次变成两片的那一次（从 1 数，K）。
    unlink_calls: usize,
    inode_creations: usize,
    split_publish: Option<usize>,
}

/// S1 启动核对 `<流>_allocator_carries_the_root_ring_of_make_filesystem`（登记修订 6 第 6.2 节第 1 条）：分配器装着的根环表与
/// `root_ring_occupancy_after_make_filesystem` 给这次 mkfs 算出的那一份 `{:?}` 文本相同。`RootRingOccupancy` 只派生 `Clone, Debug`，
/// 所以按文本比；两边记的是文本的 sha256（文本长、带空白，结果行里只放摘要），没装记 `None`。
fn allocator_carries_the_root_ring_of_make_filesystem_check(
    name: StreamName,
    allocator: &PoolAllocator,
    genesis: &MakeFilesystemOutput,
) -> StopCheck {
    let digest_of_the_debug_text =
        |occupancy: &RootRingOccupancy| sha256_hexadecimal(debug_text(occupancy).as_bytes());
    StopCheck::new(
        &format!(
            "{}_allocator_carries_the_root_ring_of_make_filesystem",
            name.directory_name()
        ),
        allocator
            .root_ring_occupancy()
            .map(digest_of_the_debug_text),
        Some(digest_of_the_debug_text(
            &root_ring_occupancy_after_make_filesystem(&common::parameters(), genesis),
        )),
    )
}

impl LiveStream {
    fn start(name: StreamName, tag: &str, switches: RunSwitches) -> Self {
        let formatted = common::format_pool(tag);
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), LOCAL_DEVICE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), LOCAL_DEVICE_BYTES),
        ]);
        allocator.mark_format_time_units(
            Placement {
                slot: INSTANCE_TABLE_SLOT,
                span: 2,
            },
            Placement {
                slot: TREE_TABLE_GENESIS_SLOT,
                span: 1,
            },
        );
        // 登记修订 6 第 6.2 节第 1 条：mkfs 同一个进程里接着写的会话照 core 部署的那一种，装上 mkfs 刚写下的根环表
        // （转过根环之后照 D16（发布语义） 已定项 1 的可再分配谓词回收）；空闲图照旧用 `DeviceFreeMap::new`（S1 回比 `build_pool`、S2 比 E170）。
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
            &common::parameters(),
            &formatted.genesis,
        ));
        let root_ring_check = allocator_carries_the_root_ring_of_make_filesystem_check(
            name,
            &allocator,
            &formatted.genesis,
        );
        let driver = StreamDriver::new(
            name,
            formatted.stream.clone(),
            formatted.mkfs_operation_count,
            switches,
        );
        Self {
            driver,
            formatted,
            allocator,
            instance: None,
            warm_up: None,
            output: None,
            mount_output_text: None,
            versions: Vec::new(),
            checks: vec![root_ring_check],
            warm_up_operation_count: 0,
            unlink_calls: 0,
            inode_creations: 0,
            split_publish: None,
        }
    }

    /// 刚记的这次调用之后的内存对象（[`memory_objects_of`]）。阳性对照「内存对象」：流二第 2 次调用之后，分配器那一项换成在副本上多分配一槽的那份。
    fn record_memory_objects(&mut self) {
        let call_index = self.driver.calls.len() - 1;
        let mut objects = memory_objects_of(
            &self.allocator,
            self.instance,
            self.warm_up.as_ref(),
            self.output.as_ref(),
            self.mount_output_text.as_deref(),
        );
        if self.driver.switches.positive_control == Some(PositiveControl::MemoryObject)
            && self.driver.name == StreamName::FixedScriptThroughEThenUnmount
            && call_index == RETURN_VALUE_CONTROL_CALL
        {
            let mut allocator_with_one_more_slot = self.allocator.clone();
            let generation = self.output.as_ref().map_or(CheckpointTxg(1), |output| {
                CheckpointTxg(output.root.checkpoint_txg.0 + 1)
            });
            let _ = allocator_with_one_more_slot.allocate_user_data(generation);
            objects[0].1 = debug_text(&allocator_with_one_more_slot);
        }
        self.driver
            .record_the_memory_objects_after_the_last_call(objects);
    }

    /// 取号、暖机两次调用；两次都返回之后模型推一步。两次调用共用一个 `PoolWriter`（与 `build_pool` 同一条写流，S1 逐条比）。
    fn step_acquire_and_warm_up(&mut self) -> bool {
        let parameters = common::parameters();
        let genesis = self.formatted.genesis.clone();
        let devices = self
            .formatted
            .devices
            .as_mut()
            .expect("format_pool 交回开着的设备");
        let mut pool = PoolWriter::new(&parameters, devices);
        let acquired = self
            .driver
            .call("acquire", debug_text::<InstanceGeneration>, || {
                acquire_instance(&mut pool)
            });
        let acquired_instance = acquired.as_ref().ok().copied();
        self.driver
            .record_the_memory_objects_after_the_last_call(memory_objects_of(
                &self.allocator,
                acquired_instance,
                None,
                None,
                None,
            ));
        let instance = match acquired {
            Ok(instance) => instance,
            Err(failure) => {
                self.driver
                    .record_a_refusal_the_model_only_goes_forward_on(failure);
                return false;
            }
        };
        self.checks.push(StopCheck::new(
            "first_acquired_instance_is_one",
            InstanceGeneration(1),
            instance,
        ));
        let warmed_result = self.driver.call("warm_up", debug_text::<WarmUpOutput>, || {
            warm_up(&mut pool, &genesis.root, instance)
        });
        self.driver
            .record_the_memory_objects_after_the_last_call(memory_objects_of(
                &self.allocator,
                Some(instance),
                warmed_result.as_ref().ok(),
                None,
                None,
            ));
        let warmed = match warmed_result {
            Ok(warmed) => warmed,
            Err(failure) => {
                self.driver
                    .record_a_refusal_the_model_only_goes_forward_on(failure);
                return false;
            }
        };
        self.warm_up_operation_count = self.driver.stream.operation_count();
        self.driver
            .advance_the_model_past_the_acquire_and_the_warm_up();
        self.instance = Some(instance);
        self.warm_up = Some(warmed);
        true
    }

    /// 新池新建文件（`publish_first_file`）：按模型的 `answer_publish_first_file` 比。阳性对照「交回值」改的就是流二这一次（第 2 次调用）。
    fn step_publish_first_file(&mut self) -> bool {
        let parameters = common::parameters();
        let content = local_first_file_content();
        let instance = self.instance.expect("取号成功才走到这一步");
        let warmed = self.warm_up.as_ref().expect("暖机成功才走到这一步");
        let devices = self
            .formatted
            .devices
            .as_mut()
            .expect("format_pool 交回开着的设备");
        let mut pool = PoolWriter::new(&parameters, devices);
        let stream_length_before = self.driver.stream.operation_count();
        let result = self
            .driver
            .call("publish", debug_text::<TransactionOutput>, || {
                publish_first_file(
                    &mut pool,
                    &mut self.allocator,
                    warmed.roots.last().expect("暖机两代根"),
                    FirstFile {
                        content: &content,
                        write_time_seconds: LOCAL_FIXED_WRITE_TIME_SECONDS,
                    },
                    instance,
                    &warmed.last_record_bytes,
                )
            });
        let call_index = self.driver.calls.len() - 1;
        self.driver
            .note_the_published_content_of_the_last_call(PublishedContent::FirstFile(
                content.clone(),
            ));
        if let Ok(output) = &result {
            if self.driver.switches.positive_control == Some(PositiveControl::ReturnValue)
                && self.driver.name == StreamName::FixedScriptThroughEThenUnmount
                && call_index == RETURN_VALUE_CONTROL_CALL
            {
                let mut recorded_copy = output.clone();
                recorded_copy.root.checkpoint_txg =
                    CheckpointTxg(recorded_copy.root.checkpoint_txg.0 + 1);
                self.driver
                    .replace_the_recorded_return_value_of_the_last_call(&debug_text(
                        &recorded_copy,
                    ));
            }
        }
        match result {
            Ok(output) => {
                self.driver.judge(
                    |model| model.answer_publish_first_file(&content),
                    ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                        roots: vec![observed_root_of_file_version(&output)],
                        reported_ceiling: None,
                    }),
                );
                self.output = Some(output);
                self.record_memory_objects();
                true
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_publish_error(&error),
                    0,
                    stream_length_before,
                    None,
                    None,
                );
                self.driver
                    .judge(|model| model.answer_publish_first_file(&content), observed);
                self.record_memory_objects();
                false
            }
            Err(None) => {
                self.record_memory_objects();
                false
            }
        }
    }

    /// 覆盖写（`publish_overwrite`，写时刻取本地常量 + 60）：按模型的 `answer_publish_overwrite` 比。
    fn step_overwrite(&mut self, content: &[u8], instance: InstanceGeneration) -> bool {
        let parameters = common::parameters();
        let previous = self.output.as_ref().expect("上一版带文件才走到这一步");
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let stream_length_before = self.driver.stream.operation_count();
        let result = self
            .driver
            .call("publish", debug_text::<TransactionOutput>, || {
                publish_overwrite(
                    &mut writer,
                    &mut self.allocator,
                    previous,
                    FirstFile {
                        content,
                        write_time_seconds: LOCAL_FIXED_WRITE_TIME_SECONDS + 60,
                    },
                    instance,
                )
            });
        self.driver
            .note_the_published_content_of_the_last_call(PublishedContent::Overwrite(
                content.to_vec(),
            ));
        match result {
            Ok(output) => {
                self.driver.judge(
                    |model| model.answer_publish_overwrite(content),
                    ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                        roots: vec![observed_root_of_file_version(&output)],
                        reported_ceiling: None,
                    }),
                );
                self.output = Some(output);
                self.record_memory_objects();
                true
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_publish_error(&error),
                    0,
                    stream_length_before,
                    None,
                    None,
                );
                self.driver
                    .judge(|model| model.answer_publish_overwrite(content), observed);
                self.record_memory_objects();
                false
            }
            Err(None) => {
                self.record_memory_objects();
                false
            }
        }
    }

    /// 进程重开之后的可写挂载（取号、写行、暖机）：按模型的 `answer_mount_writable` 比；准入不够时写行之后推的几串抬 F 逐串再比
    /// （`history.rs` 的 `settle_mount` 同一种比法）。
    fn step_mount_writable(&mut self) -> bool {
        if self.driver.model_is_enabled() {
            self.driver.model.close_session();
        }
        let parameters = common::parameters();
        let stream_length_before = self.driver.stream.operation_count();
        let formatted = &mut self.formatted;
        let result = self.driver.call(
            "mount_writable",
            debug_text::<singlefs_core::mount::Mounted>,
            || {
                let mut devices = formatted.reopen_recorded();
                let mounted = mount_writable(&parameters, &mut devices);
                formatted.devices = Some(devices);
                mounted
            },
        );
        match result {
            Ok(mounted) => {
                let (observed, _carried_to_zero_unit_publishes) =
                    observed_mount_carrying_to_zero_unit_publishes(&mounted);
                self.driver.judge(
                    |model| Ok(model.answer_mount_writable()),
                    ObservedOutcome::Succeeded(observed),
                );
                for raised in mounted.output.space_admission.floor_raises() {
                    let raised_floor = raised
                        .publishes
                        .first()
                        .expect("抬 F 做成至少推一次空发布")
                        .root
                        .rollback_floor;
                    self.driver.judge(
                        |model| {
                            model.answer_raise_rollback_floor(
                                singlefs_harness::model::ModelCheckpointTxg(raised_floor.0),
                            )
                        },
                        ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                            roots: raised
                                .publishes
                                .iter()
                                .map(observed_root_of_file_version)
                                .collect(),
                            reported_ceiling: Some(singlefs_harness::model::ModelCheckpointTxg(
                                raised.ceiling.0,
                            )),
                        }),
                    );
                }
                self.mount_output_text = Some(debug_text(&mounted.output));
                self.allocator = mounted.allocator;
                self.output = mounted.current.into_file_version();
                self.record_memory_objects();
                if self.output.is_none() {
                    self.driver.halted = true;
                    self.driver.stopped_without_an_error =
                        Some("mount_writable_returned_a_version_without_a_file".to_string());
                }
                self.output.is_some()
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_mount_error(&error),
                    0,
                    stream_length_before,
                    None,
                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
                );
                self.driver
                    .judge(|model| Ok(model.answer_mount_writable()), observed);
                self.record_memory_objects();
                false
            }
            Err(None) => {
                self.record_memory_objects();
                false
            }
        }
    }

    /// 同一个会话里挂着的时候回退到 A 的根（一次向前发布）：按模型的 `answer_rollback_while_mounted` 比。
    fn step_roll_back(&mut self) -> bool {
        let target = RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        };
        let target_key = model_root_key(target.instance, target.checkpoint_txg);
        let parameters = common::parameters();
        let output = self.output.as_mut().expect("上一版带文件才走到这一步");
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let stream_length_before = self.driver.stream.operation_count();
        let result = self.driver.call(
            "roll_back",
            debug_text::<singlefs_core::mount::RolledBack>,
            || {
                roll_back_by_a_forward_publish(
                    &parameters,
                    devices,
                    &mut self.allocator,
                    &mut *output,
                    target,
                )
            },
        );
        let observed = match &result {
            Ok(_rolled_back) => ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                roots: vec![observed_root_of_file_version(output)],
                reported_ceiling: None,
            }),
            Err(Some(error)) => self.driver.refusal(
                error,
                refusal_reason_of_rollback_error(error),
                0,
                stream_length_before,
                None,
                root_ring_slot_still_bad_after_one_reread_of_rollback_error(error),
            ),
            Err(None) => {
                self.record_memory_objects();
                return false;
            }
        };
        self.driver.judge(
            |model| model.answer_rollback_while_mounted(target_key),
            observed,
        );
        self.record_memory_objects();
        result.is_ok()
    }

    /// 抬 F 到 txg 11（`raise_rollback_floor`，影子账开）：按模型的 `answer_raise_rollback_floor` 比（拒与成都要与实现报的上限相等）。
    fn step_raise_floor(&mut self) -> bool {
        let new_floor = CheckpointTxg(11);
        let parameters = common::parameters();
        let mut current = self.output.clone().expect("上一版带文件才走到这一步");
        let txg_before_the_raise = current.root.checkpoint_txg.0;
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let stream_length_before = self.driver.stream.operation_count();
        let result = self.driver.call(
            "raise_floor",
            debug_text::<singlefs_core::mount::RaisedFloor>,
            || {
                raise_rollback_floor(
                    &parameters,
                    devices,
                    &mut self.allocator,
                    &mut current,
                    new_floor,
                    ShadowLedger::On,
                )
            },
        );
        let publishes_completed = usize::try_from(
            current
                .root
                .checkpoint_txg
                .0
                .saturating_sub(txg_before_the_raise),
        )
        .expect("一次抬 F 至多推根环区域数那么多次");
        self.output = Some(current);
        let floor = singlefs_harness::model::ModelCheckpointTxg(new_floor.0);
        let raised_ok = match result {
            Ok(raised) => {
                self.checks.push(StopCheck::new(
                    "second_stream_raise_floor_publishes",
                    LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES,
                    raised.publishes.len(),
                ));
                let observed = ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: raised
                        .publishes
                        .iter()
                        .map(observed_root_of_file_version)
                        .collect(),
                    reported_ceiling: Some(singlefs_harness::model::ModelCheckpointTxg(
                        raised.ceiling.0,
                    )),
                });
                self.driver
                    .judge(|model| model.answer_raise_rollback_floor(floor), observed);
                true
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_mount_error(&error),
                    publishes_completed,
                    stream_length_before,
                    reported_ceiling_of_mount_error(&error),
                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
                );
                self.driver
                    .judge(|model| model.answer_raise_rollback_floor(floor), observed);
                false
            }
            Err(None) => false,
        };
        self.record_memory_objects();
        raised_ok
    }

    /// 正常卸载（`unmount`，入口记在录制流里）：按模型的 `answer_unmount` 比（16-2 加的卸载答；观测 `observed_unmount` 是那几次带新 F
    /// 的空发布的根）。阳性对照「O1-对拍-卸载」把观测里第一条根的 F 加一。
    fn step_unmount(&mut self) -> bool {
        let mut current = PoolVersion::WithFile(self.output.clone().expect("上一版带文件"));
        let stream = self.driver.stream.clone();
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let stream_length_before = self.driver.stream.operation_count();
        let result = self
            .driver
            .call(UNMOUNT_CALL_LABEL, unmounted_debug_text, || {
                stream.record_entry(RecordedPublishEntry::Unmount, || {
                    unmount(
                        &common::parameters(),
                        devices,
                        &mut self.allocator,
                        &mut current,
                        ShadowLedger::On,
                    )
                })
            });
        self.output = current.into_file_version();
        let unmounted_ok = match result {
            Ok(unmounted) => {
                let mut observed = observed_unmount(&unmounted);
                if self.driver.switches.positive_control
                    == Some(PositiveControl::ComparisonOfTheUnmount)
                    && self.driver.name == StreamName::FixedScriptThroughEThenUnmount
                {
                    if let ObservedEffect::Publishes { roots, .. } = &mut observed {
                        if let Some(first_root) = roots.first_mut() {
                            first_root.rollback_floor = singlefs_harness::model::ModelCheckpointTxg(
                                first_root.rollback_floor.0 + 1,
                            );
                        }
                    }
                }
                self.driver.judge(
                    |model| model.answer_unmount(),
                    ObservedOutcome::Succeeded(observed),
                );
                match unmounted {
                    Unmounted::FloorRaisedToTheCurrentVersion(raised) => {
                        self.checks.push(StopCheck::new(
                            "second_stream_unmount_txgs",
                            LOCAL_SECOND_STREAM_UNMOUNT_TXGS.to_vec(),
                            unmounted_txgs_of(&raised),
                        ));
                        true
                    }
                    Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => false,
                }
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_mount_error(&error),
                    0,
                    stream_length_before,
                    reported_ceiling_of_mount_error(&error),
                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
                );
                self.driver.judge(|model| model.answer_unmount(), observed);
                false
            }
            Err(None) => false,
        };
        self.record_memory_objects();
        unmounted_ok
    }

    /// 流三的一次发布（意图、走空的一步、建一个 inode）：经 `PoolWriter` 调 `publish`，按模型的答比（观测与 `history.rs` 的
    /// `settle_user_change_comparing_every_version_published` 同一种：那一版的根 `observed_root_of_file_version`）。成功时换上现行那一版。
    fn step_third_stream_publish(
        &mut self,
        label: &'static str,
        answer: fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
        publish: &ThirdStreamPublish<'_>,
    ) -> bool {
        let parameters = common::parameters();
        let instance = self.instance.expect("取号成功才走到这一步");
        let previous = self.output.as_ref().expect("上一版接得上才走到这一步");
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let stream_length_before = self.driver.stream.operation_count();
        let result = self
            .driver
            .call(label, debug_text::<TransactionOutput>, || {
                publish(&mut writer, &mut self.allocator, previous, instance)
            });
        let published = match result {
            Ok(output) => {
                // 胶水造观测时 panic（`model_comparison::model_unit_role` 里模型还不罩的几种单元：extent 上段根之下的节点、记账树根之下的节点、
                // 墓碑树第二片起的容器与根之下的节点）：接住，记「模型接不了这份观测」、模型停在这次调用。跨出一片 inode 叶容器不再走到这里
                // （理想模型那件修理之后胶水按片记角色，不再断言；登记修订 2 第 2.2 节）。
                let observed_root = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    observed_root_of_file_version(&output)
                }));
                match observed_root {
                    Ok(root) => self.driver.judge(
                        answer,
                        ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                            roots: vec![root],
                            reported_ceiling: None,
                        }),
                    ),
                    Err(payload) => self
                        .driver
                        .note_that_the_model_cannot_take_the_observation(panic_text(&*payload)),
                }
                self.output = Some(output);
                true
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_publish_error(&error),
                    0,
                    stream_length_before,
                    None,
                    None,
                );
                self.driver.judge(answer, observed);
                false
            }
            Err(None) => false,
        };
        self.record_memory_objects();
        published
    }

    /// 流三在第一个文件之后的那一段（读法 2）：删第一个文件——意图一次，再一次一调走空 `deleted_inodes`；之后一次建一个 inode，
    /// 直到 inode 树的叶容器第一次变成两片（那一次是第 K 次），再多建一次。叶容器数读交回值的 `inode_leaf_containers`。
    fn run_the_third_stream_after_the_first_file(&mut self) {
        let write_time_seconds = LOCAL_FIXED_WRITE_TIME_SECONDS + 60;
        let first_inode = InodeNumber(FIRST_INODE_NUMBER);
        if !self.step_third_stream_publish(
            "unlink_intent",
            IdealModel::answer_unlink_intent,
            &|writer, allocator, previous, instance| {
                publish_unlink_intent(
                    writer,
                    allocator,
                    previous,
                    first_inode,
                    write_time_seconds,
                    instance,
                )
            },
        ) {
            return;
        }
        self.unlink_calls = 1;
        // 迭代上界：第一个文件一个数据单元，意图之后一步就是最后一批；留 16 步的余量，到了还没走空就停下记 S1。
        for _ in 0..THIRD_STREAM_UNLINK_STEP_LIMIT {
            let Some(inode) = self
                .output
                .as_ref()
                .and_then(|output| output.deleted_inodes.first().copied())
            else {
                break;
            };
            if !self.step_third_stream_publish(
                "unlink_step",
                IdealModel::answer_deletion_step,
                &|writer, allocator, previous, instance| {
                    publish_unlink_step(
                        writer,
                        allocator,
                        previous,
                        inode,
                        UnlinkBatchSize::OneExtentLowerLeaf,
                        instance,
                    )
                },
            ) {
                return;
            }
            self.unlink_calls += 1;
        }
        // 迭代上界 THIRD_STREAM_INODE_CREATION_LIMIT；跨轮带的是建了几次与第一次变成两片是哪一次；出口：出错、或分裂之后多建了一次。
        while self.inode_creations < THIRD_STREAM_INODE_CREATION_LIMIT {
            if !self.step_third_stream_publish(
                "create_inode",
                IdealModel::answer_create_inode,
                &|writer, allocator, previous, instance| {
                    publish_new_inodes(writer, allocator, previous, 1, write_time_seconds, instance)
                },
            ) {
                return;
            }
            self.inode_creations += 1;
            let leaf_containers = self
                .output
                .as_ref()
                .map_or(0, |output| output.inode_leaf_containers.len());
            if self.split_publish.is_none() && leaf_containers >= 2 {
                self.split_publish = Some(self.inode_creations);
            }
            if self
                .split_publish
                .is_some_and(|split| self.inode_creations == split + 1)
            {
                break;
            }
        }
        let leaf_containers_at_the_end = self
            .output
            .as_ref()
            .map_or(0, |output| output.inode_leaf_containers.len());
        self.checks.extend([
            StopCheck::new(
                "third_stream_unlink_calls",
                LOCAL_THIRD_STREAM_UNLINK_CALLS,
                self.unlink_calls,
            ),
            StopCheck::new(
                "third_stream_split_publish",
                Some(LOCAL_THIRD_STREAM_SPLIT_PUBLISH),
                self.split_publish,
            ),
            StopCheck::new(
                "third_stream_inode_creations_are_the_split_publish_plus_one",
                self.split_publish.map(|split| split + 1),
                Some(self.inode_creations),
            ),
            StopCheck::new(
                "third_stream_leaf_containers_at_the_end",
                2usize,
                leaf_containers_at_the_end,
            ),
        ]);
    }
}

fn unmounted_txgs_of(raised: &UnmountRaisedTheFloor) -> Vec<u64> {
    raised
        .publishes
        .iter()
        .map(|publish| publish.root.checkpoint_txg.0)
        .collect()
}

impl LiveStream {
    /// 整条流的脚本：任一步出错或 panic 就停（这条流不再往下驱动、截在那里照判）。流一是取号、暖机、新池新建文件；流二接着照
    /// `tests/crash_enumeration_fixed_script_stream.rs` 的脚本走到 E 再正常卸载；流三接着删第一个文件、建 inode 跨一片叶。
    /// 每一步成功之后才往崩溃点重放手写 oracle 的版本表里记一版（流三只记第一个文件那一版：之后的版本没有文件，手写版本表写不出，那张表只进附带的 `red_items`）。
    fn run_the_script(&mut self) {
        if !self.step_acquire_and_warm_up() || !self.step_publish_first_file() {
            return;
        }
        match self.driver.name {
            StreamName::NewPoolFileCreation => return,
            StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf => {
                self.versions
                    .push(version(1, 3, local_first_file_content()));
                self.run_the_third_stream_after_the_first_file();
                return;
            }
            StreamName::FixedScriptThroughEThenUnmount => {}
        }
        self.versions
            .push(version(1, 3, local_first_file_content()));
        if !self.step_overwrite(&second_content(), InstanceGeneration(1)) {
            return;
        }
        self.versions.push(version(1, 4, second_content()));
        // 进程重开：取号 2、写行、暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1），文件还是第二次的内容。
        if !self.step_mount_writable() {
            return;
        }
        for checkpoint_txg in 5..=7 {
            self.versions
                .push(version(2, checkpoint_txg, second_content()));
        }
        if !self.step_overwrite(&third_content(), InstanceGeneration(2)) {
            return;
        }
        self.versions.push(version(2, 8, third_content()));
        // 同一个会话里挂着的时候回退到 A 的根：一次向前发布 D（txg 9 落盘 0），文件回到第一次的内容。
        if !self.step_roll_back() {
            return;
        }
        self.versions
            .push(version(2, 9, local_first_file_content()));
        let mut latest = Vec::new();
        for (checkpoint_txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
            latest = later_content(seed);
            if !self.step_overwrite(&latest, InstanceGeneration(2)) {
                return;
            }
            self.versions
                .push(version(2, checkpoint_txg, latest.clone()));
        }
        if !self.step_raise_floor() {
            return;
        }
        for checkpoint_txg in 15..=16 {
            self.versions
                .push(version(2, checkpoint_txg, latest.clone()));
        }
        if !self.step_overwrite(&later_content(31), InstanceGeneration(2)) {
            return;
        }
        let reuse_slot = self
            .output
            .as_ref()
            .and_then(|output| output.data_pointers.first())
            .and_then(|pointer| pointer.locations.first())
            .map(|location| location.slot.0);
        self.checks.push(StopCheck::new(
            "second_stream_e_data_unit_slot",
            Some(LOCAL_SECOND_STREAM_E_DATA_UNIT_SLOT),
            reuse_slot,
        ));
        self.versions.push(version(2, 17, later_content(31)));
        if !self.step_unmount() {
            return;
        }
        for checkpoint_txg in 18..=19 {
            self.versions
                .push(version(2, checkpoint_txg, later_content(31)));
        }
    }

    /// Q5 那一形（读法 7）要的流二前四次调用：取号、暖机、第一个文件（第 2 次调用，模型跟到这一次）、第一次覆盖写（第 3 次调用，
    /// 实现照常跑完、写流照录，模型不问——那一形里它在途）。与基线流二前四次调用逐条同一个驱动。
    fn run_the_second_stream_through_the_first_overwrite(&mut self) {
        assert_eq!(self.driver.name, StreamName::FixedScriptThroughEThenUnmount);
        if !self.step_acquire_and_warm_up() || !self.step_publish_first_file() {
            return;
        }
        self.versions
            .push(version(1, 3, local_first_file_content()));
        self.driver.model_not_asked_from_call = Some(CONFIRMED_LOSS_IN_FLIGHT_CALL);
        let _ = self.step_overwrite(&second_content(), InstanceGeneration(1));
    }
}

/// 一条流驱动完之后的样子：mkfs 之后的基镜像、mkfs 之后的录制流、写表与段、写表每项在录制流里的下标、调用表、被判的根、版本表、
/// 模型提交过的版本目录、S1 回比，以及驱动怎么结束的。
struct PreparedStream {
    name: StreamName,
    base: MemoryPool,
    operations: Vec<RetainedOperation>,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    stream_indexes: Vec<usize>,
    calls: Vec<RecordedCall>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
    committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    /// 模型不再往前走的那次调用（`None`：整条流的每次调用都比过了）。
    model_stopped_at_call: Option<usize>,
    checks: Vec<StopCheck>,
    ending: StreamEnding,
}

/// 驱动怎么结束的：每一步都成功；某一步交回错误、或 panic（这条流都在那里截断，照常导出、照判，读法 5）；
/// 驱动没出错却接不下去（可写挂载交回的现行版本不带文件，同样截断照判）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum StreamEnding {
    Completed,
    StoppedByAnError { call_index: usize, member: String },
    StoppedByAPanic { call_index: usize, message: String },
    StoppedWithoutAnError { what: String },
}

/// 驱动怎么结束的：看最后一次调用的结局与驱动记下的「接不下去」。
fn ending_of_the_driver(driver: &StreamDriver) -> StreamEnding {
    if let Some(what) = &driver.stopped_without_an_error {
        return StreamEnding::StoppedWithoutAnError { what: what.clone() };
    }
    match driver.calls.last().map(|call| &call.outcome) {
        Some(CallOutcome::ReturnedError(member)) => StreamEnding::StoppedByAnError {
            call_index: driver.calls.len() - 1,
            member: format!("err:{member}"),
        },
        Some(CallOutcome::Panicked(message)) => StreamEnding::StoppedByAPanic {
            call_index: driver.calls.len() - 1,
            message: message.clone(),
        },
        Some(CallOutcome::Succeeded | CallOutcome::InFlightAtTheTruncation) | None => {
            StreamEnding::Completed
        }
    }
}

/// 驱动一条流并取出它的基镜像、写表与段。`check_against_the_crates` 只在基线那一趟开：流一整条流逐条与 `build_pool` 比（S1）。
fn prepare_stream(
    name: StreamName,
    switches: RunSwitches,
    check_against_the_crates: bool,
) -> PreparedStream {
    let tag = format!("e171r1-{}", name.directory_name());
    let mut live = LiveStream::start(name, &tag, switches);
    live.run_the_script();
    prepared_stream_of_the_live_stream(name, live, check_against_the_crates)
}

/// 驱动完的一条流取出基镜像、写表与段（[`prepare_stream`] 与 Q5 那一形共用）。
fn prepared_stream_of_the_live_stream(
    name: StreamName,
    mut live: LiveStream,
    check_against_the_crates: bool,
) -> PreparedStream {
    let ending = ending_of_the_driver(&live.driver);
    let all_operations = live.formatted.retained_operations();
    let mkfs_operation_count = live.formatted.mkfs_operation_count;
    // 照抄 `tests/crash_enumeration_fixed_script_stream.rs` 的 `writes_and_segments_after_mkfs`：入口段（卸载入口发的那几段）一起交给切段（C557）。
    let entry_spans: Vec<RecordedEntrySpan> = live
        .formatted
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - mkfs_operation_count
                ..span.operations.end - mkfs_operation_count,
        })
        .collect();
    let operations = all_operations[mkfs_operation_count..].to_vec();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations,
        &entry_spans,
        &common::geometry(),
    );
    let base = live.formatted.memory_pool_after_mkfs();
    let mut checks = std::mem::take(&mut live.checks);
    if check_against_the_crates {
        checks.extend(stream_checks_against_the_crates(
            name,
            &live,
            &all_operations,
            &operations,
            &writes,
            &segments,
            &stream_indexes,
            &base,
        ));
    }
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .unwrap_or(usize::MAX);
    let versions = if name == StreamName::NewPoolFileCreation && judged_root_index != usize::MAX {
        // 流一的版本表照 `tests/crash_enumeration_new_pool_file_creation_stream.rs` 的 `single_version`。
        let (instance, checkpoint_txg) =
            singlefs_harness::memory_pool::root_identity_of_write(&writes[judged_root_index]);
        vec![PublishedVersion {
            instance,
            checkpoint_txg,
            content: local_first_file_content(),
        }]
    } else {
        std::mem::take(&mut live.versions)
    };
    PreparedStream {
        name,
        base,
        operations,
        writes,
        segments,
        stream_indexes,
        calls: live.driver.calls.clone(),
        judged_root_index,
        versions,
        committed_versions: std::mem::take(&mut live.driver.committed_versions),
        model_stopped_at_call: live.driver.model_stopped_at_call,
        checks,
        ending,
    }
}

/// S1 里跟流有关的几条（只在基线那一趟）：段长表、写数、根槽写数、盘数与盘字节数对本地常量；流一整条流逐条与 `build_pool` 比。
/// 与 `e170_o3_spec_executor_export.rs` 的 `prepare_stream` 里那一批 S1 回比同一批。
#[allow(
    clippy::too_many_arguments,
    reason = "这几样是 prepare_stream 驱动完之后手里同时有的，拆成结构体只会多一层搬运"
)]
fn stream_checks_against_the_crates(
    name: StreamName,
    live: &LiveStream,
    all_operations: &[RetainedOperation],
    operations: &[RetainedOperation],
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    stream_indexes: &[usize],
    base: &MemoryPool,
) -> Vec<StopCheck> {
    let mkfs_operation_count = live.formatted.mkfs_operation_count;
    let label = name.directory_name();
    let non_barrier_operations: Vec<usize> = operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| retained.operation.kind != RecordedOperationKind::Barrier)
        .map(|(stream_index, _)| stream_index)
        .collect();
    let (local_segment_lengths, local_writes, local_root_writes): (&[usize], usize, usize) =
        match name {
            StreamName::NewPoolFileCreation => (
                &LOCAL_FIRST_STREAM_SEGMENT_LENGTHS,
                LOCAL_FIRST_STREAM_WRITES,
                LOCAL_FIRST_STREAM_ROOT_WRITES,
            ),
            StreamName::FixedScriptThroughEThenUnmount => (
                &LOCAL_SECOND_STREAM_SEGMENT_LENGTHS,
                LOCAL_SECOND_STREAM_WRITES,
                LOCAL_SECOND_STREAM_ROOT_WRITES,
            ),
            StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf => (
                LOCAL_THIRD_STREAM_SEGMENT_LENGTHS,
                LOCAL_THIRD_STREAM_WRITES,
                LOCAL_THIRD_STREAM_ROOT_WRITES,
            ),
        };
    let root_write_count = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .count();
    let mut checks = vec![
        StopCheck::new(
            &format!("{label}_write_order_is_the_non_barrier_operation_order"),
            true,
            non_barrier_operations == stream_indexes,
        ),
        StopCheck::new(
            &format!("{label}_segment_lengths"),
            local_segment_lengths.to_vec(),
            segments.iter().map(Vec::len).collect::<Vec<usize>>(),
        ),
        StopCheck::new(&format!("{label}_writes"), local_writes, writes.len()),
        StopCheck::new(
            &format!("{label}_root_writes"),
            local_root_writes,
            root_write_count,
        ),
        StopCheck::new(
            &format!("{label}_device_count"),
            LOCAL_DEVICE_COUNT,
            base.devices.len(),
        ),
        StopCheck::new(
            &format!("{label}_device_bytes"),
            LOCAL_DEVICE_BYTES,
            base.device_size_in_bytes,
        ),
    ];
    if name == StreamName::NewPoolFileCreation {
        // 流一的驱动是 `build_pool` 的照抄：整条流（含 mkfs）逐条与 `build_pool` 那一份比。
        let reference = common::build_pool("e171-first-stream-reference");
        checks.push(StopCheck::new(
            "new_pool_file_creation_operations_equal_build_pool",
            true,
            reference.retained_operations() == all_operations,
        ));
        checks.push(StopCheck::new(
            "new_pool_file_creation_mkfs_operations_equal_build_pool",
            mkfs_operation_count,
            reference.mkfs_operation_count,
        ));
        checks.push(StopCheck::new(
            "new_pool_file_creation_warm_up_return_point_equal_build_pool",
            live.warm_up_operation_count,
            reference.warm_up_operation_count,
        ));
    }
    checks
}
// ───────────────────────────── 取样（登记 5.4） ─────────────────────────────

/// 批 1 在一个非空区间 [a, b) 里取的点：a、b − 1 与 a + ⌊i·(b − 1 − a)/7⌋（i = 1..6）。
fn first_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    let mut points = vec![first, state_range.end - 1];
    for step in 1..FIRST_BATCH_DIVISOR {
        points.push(first + step * span / FIRST_BATCH_DIVISOR);
    }
    points
}

/// 批 2：a + ⌊(2i + 1)·(b − 1 − a)/14⌋（i = 0..6）。
fn second_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    (0..SECOND_BATCH_DIVISOR / 2)
        .map(|step| first + (2 * step + 1) * span / SECOND_BATCH_DIVISOR)
        .collect()
}

/// 批 3：a + ⌊i·(b − 1 − a)/23⌋（i = 0..23）。
fn third_batch_points(state_range: &std::ops::Range<u64>) -> Vec<u64> {
    let first = state_range.start;
    let span = state_range.end - 1 - first;
    (0..=THIRD_BATCH_DIVISOR)
        .map(|step| first + step * span / THIRD_BATCH_DIVISOR)
        .collect()
}

/// 带撕裂写的那一段扫了几个序号、取到几个带「1」的状态（登记 5.4「扫不到 4 个的照实报个数」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TornScan {
    segment_index: usize,
    scanned: u64,
    found: usize,
}

/// 三批的状态序号（各自升序、互不重叠）与撕裂扫描的记录。
struct SampledOrdinals {
    batches: [BTreeSet<u64>; 3],
    torn_scans: Vec<TornScan>,
}

fn sample_ordinals(
    state_ranges_by_segment: &[std::ops::Range<u64>],
    state_count: u64,
    segment_has_a_torn_write: &[bool],
    landing_vector_of_ordinal: &dyn Fn(u64) -> String,
) -> SampledOrdinals {
    let non_empty = || {
        state_ranges_by_segment
            .iter()
            .filter(|range| !range.is_empty())
    };
    let mut first: BTreeSet<u64> = non_empty().flat_map(first_batch_points).collect();
    first.insert(state_count - 1);
    let mut torn_scans = Vec::new();
    for (segment_index, state_range) in state_ranges_by_segment.iter().enumerate() {
        if state_range.is_empty() || !segment_has_a_torn_write[segment_index] {
            continue;
        }
        let scan_end = state_range
            .end
            .min(state_range.start + TORN_SCAN_ORDINALS_PER_SEGMENT);
        let mut found = 0usize;
        let mut scanned = 0u64;
        for ordinal in state_range.start..scan_end {
            if found == TORN_STATES_PER_SEGMENT {
                break;
            }
            scanned += 1;
            if landing_vector_of_ordinal(ordinal).contains('1') && first.insert(ordinal) {
                found += 1;
            }
        }
        torn_scans.push(TornScan {
            segment_index,
            scanned,
            found,
        });
    }
    let second: BTreeSet<u64> = non_empty()
        .flat_map(second_batch_points)
        .filter(|ordinal| !first.contains(ordinal))
        .collect();
    let third: BTreeSet<u64> = non_empty()
        .flat_map(third_batch_points)
        .filter(|ordinal| !first.contains(ordinal) && !second.contains(ordinal))
        .collect();
    SampledOrdinals {
        batches: [first, second, third],
        torn_scans,
    }
}

// ───────────────────────────── 落地向量与镜像摘要（登记 5.3） ─────────────────────────────

/// 崩溃状态序号 → 持久掩码：由枚举表现造，掩码按 `crash_facts::persisted_writes_of_state`（事实表与 GPU 判器共用的那一份，与崩溃点重放
/// `CrashStatePlan` 同一套数字次序）算，不走 `extract_flow_facts`——那一个给 GPU 判器抽事实，枚举写表多于 2048 次写的流（流三）它当场 panic。
/// 回调里崩溃点重放给的持久集合与这一份逐态比（`crash_facts_mask` 那一行），两份是两段代码。
struct StateMasks {
    recorded_write_count: usize,
    enumerated_write_count: usize,
    is_tearable: Vec<bool>,
    torn_images: Vec<TornImageFact>,
    segments: Vec<SegmentFact>,
    state_count: u64,
}

impl StateMasks {
    fn of(tables: &CrashStateEnumerationTables, segments: &[Vec<usize>]) -> Self {
        let write_index = |index: usize| u32::try_from(index).expect("写表下标装得进 u32");
        Self {
            recorded_write_count: tables.is_tearable_by_write.len(),
            enumerated_write_count: tables.enumerated_writes.len(),
            is_tearable: tables.is_tearable_by_write.clone(),
            torn_images: tables
                .torn_images
                .iter()
                .map(|row| TornImageFact {
                    write_index: write_index(row.write_index),
                    torn_image_index: write_index(row.torn_image_index),
                    replays: row
                        .replays
                        .iter()
                        .map(|(replay, replayed)| (write_index(*replay), write_index(*replayed)))
                        .collect(),
                })
                .collect(),
            segments: segments
                .iter()
                .zip(&tables.state_ranges_by_segment)
                .map(|(segment, states)| SegmentFact {
                    writes: segment.iter().map(|index| write_index(*index)).collect(),
                    states: states.clone(),
                    // 这一装置的流都不分片（C790 的分片只在崩溃放量按段登记的段上）
                    slice_start: None,
                })
                .collect(),
            state_count: tables.state_count,
        }
    }

    fn persisted_writes_of_state(&self, ordinal: u64) -> Vec<bool> {
        persisted_writes_of_state(
            self.recorded_write_count,
            self.enumerated_write_count,
            &self.is_tearable,
            &self.torn_images,
            &self.segments,
            self.state_count,
            ordinal,
        )
    }
}

/// `extract_flow_facts` 数出的状态总数（S1 那一条）：枚举写表多于 GPU 掩码宽（2048）的流抽不了，交 `None`。
fn flow_facts_state_count(
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
) -> Option<u64> {
    (tables.enumerated_writes.len() <= MAXIMUM_ENUMERATED_WRITES).then(|| {
        extract_flow_facts(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        )
        .state_count
    })
}

/// 把枚举写表上的持久集合折回按写序号的三值向量：原写持久记 `2`；原写没持久而它的撕裂镜像持久记 `1`；都没有记 `0`。
fn landing_vector_of(
    persisted: &[bool],
    recorded_write_count: usize,
    torn_image_index_by_write: &BTreeMap<usize, usize>,
) -> String {
    (0..recorded_write_count)
        .map(|write_index| {
            if persisted[write_index] {
                '2'
            } else if torn_image_index_by_write
                .get(&write_index)
                .is_some_and(|torn_image_index| persisted[*torn_image_index])
            {
                '1'
            } else {
                '0'
            }
        })
        .collect()
}

/// 镜像摘要的一个扇区：全零的不进摘要；进的依次是盘身份 u32、扇区号 u64、那 512 字节（小端）。
fn append_sector_to_digest_message(
    message: &mut Vec<u8>,
    device: u32,
    sector: u64,
    sector_bytes: &[u8],
) {
    if sector_bytes.iter().all(|byte| *byte == 0) {
        return;
    }
    message.extend_from_slice(&device.to_le_bytes());
    message.extend_from_slice(&sector.to_le_bytes());
    message.extend_from_slice(sector_bytes);
}

/// 从 `CrashImage` 直接算镜像摘要：每块盘（身份升序）上可能不全为零的扇区——基镜像写过的扇区与这一态里持久了的带字节的写罩住的扇区——
/// 合成连续的区间按升序读（读走 `CrashImage` 自己的 `PoolReader::read`），全零的扇区跳过。
fn crash_image_digest(image: &CrashImage<'_>) -> [u8; DIGEST_BYTES] {
    let mut message = Vec::new();
    let mut device_identities = image.device_identities();
    device_identities.sort_unstable();
    for device in device_identities {
        let mut sector_ranges: Vec<(u64, u64)> = image
            .base
            .devices
            .get(&device)
            .map(|sparse| {
                sparse
                    .written_sectors()
                    .keys()
                    .map(|sector| (*sector, sector + 1))
                    .collect()
            })
            .unwrap_or_default();
        for (write, is_persisted) in image.writes.iter().zip(&image.persisted) {
            let carries_bytes = match write.contents {
                WrittenContents::Bytes(_) => true,
                WrittenContents::Zeros { .. } => false,
            };
            if *is_persisted && carries_bytes && write.device == device {
                sector_ranges.push((
                    write.offset.0 / LOCAL_SECTOR_BYTES,
                    (write.offset.0 + write.length_in_bytes()).div_ceil(LOCAL_SECTOR_BYTES),
                ));
            }
        }
        for (first_sector, end_sector) in merged_sector_ranges(sector_ranges) {
            let bytes = PoolReader::read(
                image,
                device,
                DeviceOffsetInBytes(first_sector * LOCAL_SECTOR_BYTES),
                usize::try_from((end_sector - first_sector) * LOCAL_SECTOR_BYTES)
                    .expect("区间长装得进 usize"),
            )
            .expect("崩溃镜像在盘内、按扇区对齐的区间读得出");
            for (sector_offset, sector_bytes) in bytes
                .chunks_exact(usize::try_from(LOCAL_SECTOR_BYTES).expect("512"))
                .enumerate()
            {
                let sector = first_sector + u64::try_from(sector_offset).expect("扇区下标");
                append_sector_to_digest_message(&mut message, device.0, sector, sector_bytes);
            }
        }
    }
    sha256_digest(&message)
}

/// 扇区区间按起点排好、重叠或相接的合并。
fn merged_sector_ranges(mut sector_ranges: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
    sector_ranges.sort_unstable();
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (first_sector, end_sector) in sector_ranges {
        match merged.last_mut() {
            Some(last) if first_sector <= last.1 => last.1 = last.1.max(end_sector),
            Some(_) | None => merged.push((first_sector, end_sector)),
        }
    }
    merged
}

// ───────────────────────────── 中性格式：写出与读回重建（登记 5.3、S4） ─────────────────────────────

const BASE_SECTORS_FILE: &str = "base-sectors.bin";
const OPERATIONS_FILE: &str = "ops.tsv";
const OPERATIONS_PAYLOAD_FILE: &str = "ops-payload.bin";
const CALLS_FILE: &str = "calls.tsv";
const TORN_FILE: &str = "torn.tsv";
const TORN_PAYLOAD_FILE: &str = "torn-payload.bin";
const STATES_FILE: &str = "states.tsv";
const IMPLEMENTATION_VERDICTS_FILE: &str = "impl-verdicts.tsv";
const QUICK_TIER_FILE: &str = "quick-tier.tsv";
const MANIFEST_FILE: &str = "manifest.tsv";
/// manifest 记 sha256 的那几个文件，按这个次序。
const FILES_LISTED_IN_THE_MANIFEST: [&str; 9] = [
    BASE_SECTORS_FILE,
    OPERATIONS_FILE,
    OPERATIONS_PAYLOAD_FILE,
    CALLS_FILE,
    TORN_FILE,
    TORN_PAYLOAD_FILE,
    STATES_FILE,
    IMPLEMENTATION_VERDICTS_FILE,
    QUICK_TIER_FILE,
];

fn operation_kind_name(kind: RecordedOperationKind) -> &'static str {
    match kind {
        RecordedOperationKind::Write => "write",
        RecordedOperationKind::WriteForceUnitAccess => "write_fua",
        RecordedOperationKind::WriteZeroes => "write_zeroes",
        RecordedOperationKind::Barrier => "barrier",
    }
}

/// 读回来的一条操作：盘身份、种类、偏移、长度、在 `ops-payload.bin` 里的起点（清零与屏障没有）。
struct NeutralOperation {
    device: u32,
    kind: RecordedOperationKind,
    offset: u64,
    length: u64,
    payload_start: Option<usize>,
}

/// 从导出目录读回一条流：基镜像扇区、操作、两份载荷、撕裂表。只读这几个文件，不碰内存里的 `CrashImage`。
struct NeutralStream {
    base_sectors: BTreeMap<(u32, u64), Vec<u8>>,
    operations: Vec<NeutralOperation>,
    payload: Vec<u8>,
    torn_bytes_by_write: BTreeMap<usize, (usize, usize)>,
    torn_payload: Vec<u8>,
}

fn read_file(directory: &Path, file_name: &str) -> Vec<u8> {
    std::fs::read(directory.join(file_name))
        .unwrap_or_else(|error| panic!("读 {}：{error}", directory.join(file_name).display()))
}

fn read_text_lines(directory: &Path, file_name: &str) -> Vec<String> {
    String::from_utf8(read_file(directory, file_name))
        .expect("导出的 tsv 是 UTF-8")
        .lines()
        .map(str::to_string)
        .collect()
}

fn parse_number<Number: std::str::FromStr>(field: &str, what: &str) -> Number {
    field
        .parse()
        .unwrap_or_else(|_| panic!("{what} 解析不了：{field:?}"))
}

fn read_neutral_stream(directory: &Path) -> NeutralStream {
    let base_bytes = read_file(directory, BASE_SECTORS_FILE);
    let record_bytes = 4 + 8 + usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
    assert!(
        base_bytes.len().is_multiple_of(record_bytes),
        "base-sectors.bin 是整条记录"
    );
    let base_sectors = base_bytes
        .chunks_exact(record_bytes)
        .map(|record| {
            let device = u32::from_le_bytes(record[0..4].try_into().expect("4 字节"));
            let sector = u64::from_le_bytes(record[4..12].try_into().expect("8 字节"));
            ((device, sector), record[12..].to_vec())
        })
        .collect();
    let operations = read_text_lines(directory, OPERATIONS_FILE)
        .iter()
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 7, "ops.tsv 一行七列：{line:?}");
            let kind = match fields[2] {
                "write" => RecordedOperationKind::Write,
                "write_fua" => RecordedOperationKind::WriteForceUnitAccess,
                "write_zeroes" => RecordedOperationKind::WriteZeroes,
                "barrier" => RecordedOperationKind::Barrier,
                unknown => panic!("ops.tsv 里认不出的种类 {unknown:?}"),
            };
            NeutralOperation {
                device: parse_number(fields[1], "盘身份"),
                kind,
                offset: parse_number(fields[3], "偏移"),
                length: parse_number(fields[4], "长度"),
                payload_start: (fields[5] != "-").then(|| parse_number(fields[5], "载荷起点")),
            }
        })
        .collect();
    let torn_bytes_by_write = read_text_lines(directory, TORN_FILE)
        .iter()
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 4, "torn.tsv 一行四列：{line:?}");
            (
                parse_number(fields[0], "写序号"),
                (
                    parse_number(fields[1], "撕裂字节起点"),
                    parse_number(fields[2], "撕裂字节长度"),
                ),
            )
        })
        .collect();
    NeutralStream {
        base_sectors,
        operations,
        payload: read_file(directory, OPERATIONS_PAYLOAD_FILE),
        torn_bytes_by_write,
        torn_payload: read_file(directory, TORN_PAYLOAD_FILE),
    }
}

/// 从中性格式按落地向量重建一态的镜像（按流序叠：`2` 放新字节、`1` 放 `torn-payload.bin` 里那一段、`0` 不动），算镜像摘要。
fn rebuilt_image_digest(stream: &NeutralStream, landing_vector: &str) -> [u8; DIGEST_BYTES] {
    let sector_bytes = usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
    let mut sectors = stream.base_sectors.clone();
    let mut landings = landing_vector.chars();
    let mut write_index = 0usize;
    for operation in &stream.operations {
        if operation.kind == RecordedOperationKind::Barrier {
            continue;
        }
        let landing = landings.next().expect("落地向量与写数一样长");
        let first_sector = operation.offset / LOCAL_SECTOR_BYTES;
        let length = usize::try_from(operation.length).expect("写长装得进 usize");
        let bytes: Option<&[u8]> = match (landing, operation.kind) {
            ('0', _) => None,
            ('2', RecordedOperationKind::WriteZeroes) => {
                let end_sector = (operation.offset + operation.length) / LOCAL_SECTOR_BYTES;
                let covered: Vec<(u32, u64)> = sectors
                    .range((operation.device, first_sector)..(operation.device, end_sector))
                    .map(|(key, _)| *key)
                    .collect();
                for key in covered {
                    sectors.remove(&key);
                }
                None
            }
            ('2', _) => {
                let start = operation.payload_start.expect("带字节的写有载荷起点");
                Some(&stream.payload[start..start + length])
            }
            ('1', _) => {
                let (start, torn_length) = stream.torn_bytes_by_write[&write_index];
                assert_eq!(torn_length, length, "撕裂那一段罩整次写");
                Some(&stream.torn_payload[start..start + torn_length])
            }
            (unknown, _) => panic!("落地向量里认不出的字符 {unknown:?}"),
        };
        if let Some(bytes) = bytes {
            for (sector_offset, chunk) in bytes.chunks_exact(sector_bytes).enumerate() {
                let sector = first_sector + u64::try_from(sector_offset).expect("扇区下标");
                sectors.insert((operation.device, sector), chunk.to_vec());
            }
        }
        write_index += 1;
    }
    assert!(landings.next().is_none(), "落地向量比写数长");
    let mut message = Vec::new();
    for ((device, sector), bytes) in &sectors {
        append_sector_to_digest_message(&mut message, *device, *sector, bytes);
    }
    sha256_digest(&message)
}

fn root_text(root: Option<(InstanceGeneration, CheckpointTxg)>) -> String {
    match root {
        Some((instance, checkpoint_txg)) => format!("{},{}", instance.0, checkpoint_txg.0),
        None => "none".to_string(),
    }
}

/// outcome 的种类；走不下去的那一种带上失败的名字（空白换成下划线，不破坏 tsv 与结果行）。
fn outcome_text(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::NoFile { .. } => "no_file".to_string(),
        RecoveryOutcome::FileRead { .. } => "file_read".to_string(),
        RecoveryOutcome::Failed { failure, .. } => format!("failed:{failure:?}")
            .chars()
            .map(|character| {
                if character.is_whitespace() {
                    '_'
                } else {
                    character
                }
            })
            .collect(),
    }
}

fn list_text(items: &[String]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(",")
    }
}

fn sha256_text(digest: &[u8; DIGEST_BYTES]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// 一个状态上实现的判定（登记第一节读法 6：「实现都放行」= 池级 checker 一条 Violated 都没有，且记录核对器两项都为假）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ImplementationVerdict {
    consulted_root: String,
    consulted_outcome: String,
    ignored_root: String,
    ignored_outcome: String,
    red_items: Vec<String>,
    pool_checker_red: Vec<String>,
    /// 记录核对器的两项（`true` / `false`；逐件重做那一路里那一件 panic 写 `panic`、没跑写 `not_run`）。
    root_without_record: String,
    claimed_state_missing_unit: String,
}

impl ImplementationVerdict {
    /// 回调链 panic 了的那一态（修订 5 第 5.1 节第 4 条）：各格取各件自己的结果，panic 的件写 `panic`，没跑的写 `not_run`。
    /// `red_items` 照 [`CrashStateJudgement::red_items`] 的次序与写法列跑完了的那几件判红的项，panic 与没跑的件记成
    /// `panic:<项>`、`not_run:<项>`（不以 `<项>:` 起头，不进按项数红的计数）。
    fn of_the_pieces(
        consulted: &PieceOutcome<RecoveryReport>,
        ignored: &PieceOutcome<RecoveryReport>,
        crash_point_replay_oracles: &PieceOutcome<CrashPointReplayOraclesOfThePieces>,
        pool_checker: &PieceOutcome<Vec<(&'static str, InvariantVerdict)>>,
        record_checker: &PieceOutcome<RecordCheck>,
    ) -> Self {
        let mut red_items = Vec::new();
        let oracles: [(&str, Result<Option<&CrashPointReplayOracleViolation>, &str>); 2] =
            match crash_point_replay_oracles {
                PieceOutcome::Done(CrashPointReplayOraclesOfThePieces {
                    consulted: consulted_oracle,
                    ignored: ignored_oracle,
                }) => [
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
                        consulted_oracle
                            .as_ref()
                            .map(Option::as_ref)
                            .ok_or("not_run"),
                    ),
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE,
                        ignored_oracle.as_ref().map(Option::as_ref).ok_or("not_run"),
                    ),
                ],
                PieceOutcome::Panicked(_) => [
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
                        Err("panic"),
                    ),
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE,
                        Err("panic"),
                    ),
                ],
                PieceOutcome::NotRun => [
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE,
                        Err("not_run"),
                    ),
                    (
                        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE,
                        Err("not_run"),
                    ),
                ],
            };
        for (pass, oracle) in oracles {
            match oracle {
                Ok(Some(violation)) => {
                    red_items.push(format!("{pass}:{}", violation.kind.name()));
                }
                Ok(None) => {}
                Err(marker) => red_items.push(format!("{marker}:{pass}")),
            }
        }
        match pool_checker {
            PieceOutcome::Done(verdicts) => red_items.extend(
                violated_invariant_names(verdicts)
                    .into_iter()
                    .map(|invariant| {
                        format!("{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{invariant}")
                    }),
            ),
            PieceOutcome::Panicked(_) => {
                red_items.push(format!("panic:{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}"));
            }
            PieceOutcome::NotRun => {
                red_items.push(format!(
                    "not_run:{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}"
                ));
            }
        }
        match record_checker {
            PieceOutcome::Done(records) => {
                if records.root_without_record {
                    red_items.push(format!(
                        "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_ROOT_WITHOUT_RECORD}"
                    ));
                }
                if records.claimed_state_missing_unit {
                    red_items.push(format!(
                        "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
                    ));
                }
            }
            PieceOutcome::Panicked(_) => {
                red_items.push(format!(
                    "panic:{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}"
                ));
            }
            PieceOutcome::NotRun => {
                red_items.push(format!(
                    "not_run:{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}"
                ));
            }
        }
        Self {
            consulted_root: consulted.cell_text(|report| root_text(report.effective_root)),
            consulted_outcome: consulted.cell_text(|report| outcome_text(&report.outcome)),
            ignored_root: ignored.cell_text(|report| root_text(report.effective_root)),
            ignored_outcome: ignored.cell_text(|report| outcome_text(&report.outcome)),
            red_items,
            pool_checker_red: match pool_checker {
                PieceOutcome::Done(verdicts) => violated_invariant_names(verdicts),
                PieceOutcome::Panicked(_) => vec!["panic".to_string()],
                PieceOutcome::NotRun => vec!["not_run".to_string()],
            },
            root_without_record: record_checker
                .cell_text(|records| records.root_without_record.to_string()),
            claimed_state_missing_unit: record_checker
                .cell_text(|records| records.claimed_state_missing_unit.to_string()),
        }
    }
    fn of(judgement: &CrashStateJudgement) -> Self {
        let report_root = |report: &RecoveryReport| root_text(report.effective_root);
        Self {
            consulted_root: report_root(&judgement.consulted),
            consulted_outcome: outcome_text(&judgement.consulted.outcome),
            ignored_root: report_root(&judgement.ignored),
            ignored_outcome: outcome_text(&judgement.ignored.outcome),
            red_items: judgement.red_items(),
            pool_checker_red: judgement
                .pool_checker
                .iter()
                .filter(|(_, verdict)| match verdict {
                    InvariantVerdict::Violated(_) => true,
                    InvariantVerdict::Holds
                    | InvariantVerdict::NotApplicable(_)
                    | InvariantVerdict::PartiallyJudged(_) => false,
                })
                .map(|(invariant, _)| (*invariant).to_string())
                .collect(),
            root_without_record: judgement.records.root_without_record.to_string(),
            claimed_state_missing_unit: judgement.records.claimed_state_missing_unit.to_string(),
        }
    }
}

// ───────────────────────────── 每个取样状态上的 oracle（O1 三个子列、O2；O3 在 Python 一侧） ─────────────────────────────

/// O1-模型崩溃判官在一个状态上的结论（登记第一节读法 1、6）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum CrashJudgeVerdict {
    /// 这一态有写落在模型停下的那次调用（对拍判出不同、交回错误、panic，读法 6 的三处）或之后的调用里。
    NotApplicable,
    Green,
    /// 红：模型那一侧说的话（`ModelDisagreement::implementation_answer`，空白换成下划线）。
    Red {
        what: String,
    },
    /// 关一条（Q3-核）关掉了 O1：这一列不算。
    Off,
    /// 这一态上看 journal 那一遍恢复 panic（读法 5「态上的恢复 panic」；修订 5 第 5.1 节第 3 条收窄成只随这一遍）：
    /// O1 的两个崩溃子列在这一态记不适用。
    RecoveryPanicked,
    /// 这一态上 O1-模型崩溃判官自己 panic（修订 5 第 5.1 节第 3 条）：不红不绿，与弃权、不适用分开计。
    JudgePanicked,
}

impl CrashJudgeVerdict {
    fn word(&self) -> &'static str {
        match self {
            Self::NotApplicable => "na",
            Self::Green => "green",
            Self::Red { .. } => "red",
            Self::Off => "off",
            Self::RecoveryPanicked => "recovery_panic",
            Self::JudgePanicked => "judge_panic",
        }
    }
}

/// 一遍恢复报告的摘要（触达判法 ③，登记第一节读法 4）：择到的根、结局种类、读回内容的 sha256、施加的记录条数。
#[derive(Clone, Debug, PartialEq, Eq)]
struct RecoverySummary {
    root: String,
    outcome: String,
    read_back_sha256: String,
    records_applied: usize,
}

impl RecoverySummary {
    fn of(report: &RecoveryReport) -> Self {
        Self {
            root: root_text(report.effective_root),
            outcome: outcome_text(&report.outcome),
            read_back_sha256: match &report.outcome {
                RecoveryOutcome::FileRead { content, .. } => sha256_hexadecimal(content),
                RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. } => "-".to_string(),
            },
            records_applied: report.journal.prefix_applied,
        }
    }
    /// 恢复 panic 的那一态：四格都写 `recovery_panic`。
    fn of_a_panicking_recovery() -> Self {
        Self {
            root: "recovery_panic".to_string(),
            outcome: "recovery_panic".to_string(),
            read_back_sha256: "-".to_string(),
            records_applied: 0,
        }
    }
    fn text(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.root, self.outcome, self.read_back_sha256, self.records_applied
        )
    }
}

/// 池级 checker 判红的不变量名（`Violated` 才算；`NotApplicable` 不算红）。
fn violated_invariant_names(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<String> {
    verdicts
        .iter()
        .filter(|(_, verdict)| match verdict {
            InvariantVerdict::Violated(_) => true,
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => false,
        })
        .map(|(invariant, _)| (*invariant).to_string())
        .collect()
}

// ───── O1-RefFS 子列（读法 1、停机 S9）：照 `reference_filesystem_translation.rs` 文件头第 1–18 行与 `events_of` 那几臂交事件 ─────

/// RefFS 翻这一条流停在哪一步、为什么（名字与 `reference_filesystem_translation::TranslationStopReason::name` 交出的一字不差，
/// 比对器按名字认弃权种类）。
#[allow(
    clippy::enum_variant_names,
    reason = "变体名逐个照抄 harness `TranslationStopReason` 的同名臂（注释与 `events_of` 按这几个名字对照），不另起一套"
)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum ReferenceFileSystemStop {
    /// `events_of` 的 `RollBackWhileMounted` 那一臂：回退没返回 Ok。
    TheRollbackDidNotReportItsTarget,
    /// 同一臂：回退报的目标根在 m 表里查不到。
    TheRollbackTargetRootHasNoVersion,
    /// `events_of` 的 `Unlink` 那一臂：意图那次或走空的一步写了盘（返回 Ok、报错、没返回都算），RefFS 第一版没有删除。
    TheReferenceFileSystemHasNoUnlinkInTheFirstVersion,
    /// `apply_event` 里 `submit` 拒了：RefFS 不收这一步。
    TheReferenceFileSystemDoesNotTakeTheOperation,
}

impl ReferenceFileSystemStop {
    fn name(&self) -> &'static str {
        match self {
            Self::TheRollbackDidNotReportItsTarget => "回退没返回 Ok、没报目标根",
            Self::TheRollbackTargetRootHasNoVersion => "回退目标根在 m 表里查不到",
            Self::TheReferenceFileSystemHasNoUnlinkInTheFirstVersion => "RefFS 第一版没有删除",
            Self::TheReferenceFileSystemDoesNotTakeTheOperation => "RefFS 不收这一步",
        }
    }
}

/// 一次调用在 RefFS 里交的事件（`events_of` 的 `Submit`；本装置的流里没有冷启动恢复与抛弃最新根的那次挂载）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReferenceFileSystemStep {
    position: StepPosition,
    events: Vec<(
        reference_filesystem::Operation,
        reference_filesystem::OperationOutcome,
    )>,
    /// 这一步跑完时录制流（mkfs 之后）里有几步：调用的返回点；停下的那一步取 `usize::MAX`（同 `TranslatedStep::without_events`）。
    stream_length_after_the_step: usize,
}

/// 一条流翻成 RefFS 的输入：每一步的事件、翻译停在哪一步、每一段由哪一道屏障关上（m 表只在翻译时要，查回退目标用，不留）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReferenceFileSystemStream {
    steps: Vec<ReferenceFileSystemStep>,
    stopped: Option<(StepPosition, ReferenceFileSystemStop)>,
    segment_closing_stream_indexes: Vec<Option<usize>>,
}

/// 一次调用翻成的 RefFS 事件，或这一步翻不出（[`ReferenceFileSystemStop`]）。臂与 `events_of` 一一对应（停机 S9，逐臂写在注释里）。
/// 交回值的第二样：这一步出了新版本时，这一步写的根里是不是只有最后一条映到新版本（发布与回退是；`NewVersionWrittenBy`）。
fn reference_filesystem_events_of_a_call(
    call: &RecordedCall,
    versions_of_the_roots: &BTreeMap<ModelRootKey, reference_filesystem::VersionNumber>,
) -> Result<
    (
        Vec<(
            reference_filesystem::Operation,
            reference_filesystem::OperationOutcome,
        )>,
        bool,
    ),
    ReferenceFileSystemStop,
> {
    use reference_filesystem::{FileContent, Operation, OperationOutcome};
    // 发布调用的结局：返回 Ok 是「成」；交回错误、panic、在途都按「写了之后报错」交——`events_of` 的 `publish_with_content`：
    // ReturnedAnErrorAfterItMayHaveWritten 与 NeverReturned 都交 FailedAfterWriting（本装置的流停在出错的那一步，同
    // `note_the_step_the_history_stopped_in` 把入口报错按「写了之后报错」补上那一条）。
    let publish_outcome = match call.outcome {
        CallOutcome::Succeeded => OperationOutcome::Succeeded,
        CallOutcome::ReturnedError(_)
        | CallOutcome::Panicked(_)
        | CallOutcome::InFlightAtTheTruncation => OperationOutcome::FailedAfterWriting,
    };
    match call.label {
        // `events_of`：起点 AfterMakeFilesystem 交 nothing（取号、暖机是 mkfs 那个进程里的起点）；CloseAndMountWritable、
        // RaiseRollbackFloor、CreateInode 交 nothing（不改内容，写的根映到现行版本）；卸载与抬 F 同一类（文件头第 9 行「抬 F、卸载」）。
        "acquire" | "warm_up" | "mount_writable" | "raise_floor" | UNMOUNT_CALL_LABEL
        | "create_inode" => Ok((Vec::new(), false)),
        // `events_of`：PublishFirstFile / PublishOverwrite → 内容改动（成）+ 发布调用（照结局）。
        "publish" => {
            let change = match &call.published_content {
                Some(PublishedContent::FirstFile(content)) => {
                    Operation::CreateTheFirstFile(FileContent(content.clone()))
                }
                Some(PublishedContent::Overwrite(content)) => {
                    Operation::OverwriteTheFile(FileContent(content.clone()))
                }
                None => panic!("发布调用都记了交给实现的内容：{call:?}"),
            };
            Ok((
                vec![
                    (change, OperationOutcome::Succeeded),
                    (Operation::ExplicitPublish, publish_outcome),
                ],
                true,
            ))
        }
        // `events_of`：RollBackWhileMounted 返回 Ok → 回退到 m（成），m 在 m 表里查；没返回 Ok → TheRollbackDidNotReportItsTarget。
        // 本装置的回退目标写死是 A 的根（实例 1、txg 3，`step_roll_back`）。
        "roll_back" => match call.outcome {
            CallOutcome::Succeeded => {
                let target_root = model_root_key(InstanceGeneration(1), CheckpointTxg(3));
                let version = versions_of_the_roots
                    .get(&target_root)
                    .copied()
                    .ok_or(ReferenceFileSystemStop::TheRollbackTargetRootHasNoVersion)?;
                Ok((
                    vec![(
                        Operation::RollBackToVersion(version),
                        OperationOutcome::Succeeded,
                    )],
                    true,
                ))
            }
            CallOutcome::ReturnedError(_)
            | CallOutcome::Panicked(_)
            | CallOutcome::InFlightAtTheTruncation => {
                Err(ReferenceFileSystemStop::TheRollbackDidNotReportItsTarget)
            }
        },
        // `events_of`：Unlink 写了盘（返回 Ok、写了之后报错、没返回）→ TheReferenceFileSystemHasNoUnlinkInTheFirstVersion。
        // 本装置的 unlink 调用要么返回 Ok、要么在出错或 panic 那一步停（按写了之后报错算），没有「在任何写之前拒」那一臂。
        "unlink_intent" | "unlink_step" => {
            Err(ReferenceFileSystemStop::TheReferenceFileSystemHasNoUnlinkInTheFirstVersion)
        }
        unknown => panic!("RefFS 映射里没有这种调用：{unknown}"),
    }
}

/// 一条流翻成 RefFS 的输入（`ReferenceFileSystemTranslation::translate` 的同一套走法：每一步交事件、攒 m 表、记每段由哪一道屏障关上）。
/// `operations` 是 mkfs 之后的录制流，与调用的返回点同一个原点；根槽写取写表里种类是根槽的那几次（与 `translate` 按几何认根槽写同一组）。
fn reference_filesystem_stream_of(
    calls: &[RecordedCall],
    operations: &[RetainedOperation],
    writes: &[RetainedWrite],
    stream_indexes: &[usize],
) -> ReferenceFileSystemStream {
    let root_writes: Vec<(usize, ModelRootKey)> = writes
        .iter()
        .zip(stream_indexes)
        .filter(|(write, _)| write.kind == StepKind::RootRecordFua)
        .map(|(write, stream_index)| {
            let (instance, checkpoint_txg) =
                singlefs_harness::memory_pool::root_identity_of_write(write);
            (*stream_index, model_root_key(instance, checkpoint_txg))
        })
        .collect();
    let mut model = reference_filesystem::ReferenceFileSystem::on_a_freshly_formatted_pool();
    let mut versions_of_the_roots = BTreeMap::new();
    let mut steps = Vec::new();
    let mut stopped = None;
    let mut stream_length_before_the_step = 0usize;
    for (call_index, call) in calls.iter().enumerate() {
        let position = StepPosition::Operation(call_index);
        let version_before_the_step = model.newest_version();
        let applied = reference_filesystem_events_of_a_call(call, &versions_of_the_roots).and_then(
            |(events, only_the_last_root_is_new)| {
                for (operation, outcome) in &events {
                    model.submit(operation.clone(), *outcome).map_err(|_| {
                        ReferenceFileSystemStop::TheReferenceFileSystemDoesNotTakeTheOperation
                    })?;
                }
                Ok((events, only_the_last_root_is_new))
            },
        );
        let (events, only_the_last_root_is_new) = match applied {
            Ok(applied) => applied,
            Err(stop) => {
                stopped = Some((position, stop));
                steps.push(ReferenceFileSystemStep {
                    position,
                    events: Vec::new(),
                    stream_length_after_the_step: usize::MAX,
                });
                break;
            }
        };
        let version_after_the_step = model.newest_version();
        let roots_of_the_step: Vec<ModelRootKey> = root_writes
            .iter()
            .filter(|(stream_index, _)| {
                (stream_length_before_the_step..call.return_point).contains(stream_index)
            })
            .map(|(_, root)| *root)
            .collect();
        for (index_in_the_step, root) in roots_of_the_step.iter().enumerate() {
            let is_the_last_root = index_in_the_step + 1 == roots_of_the_step.len();
            let version = if only_the_last_root_is_new && !is_the_last_root {
                version_before_the_step
            } else {
                version_after_the_step
            };
            versions_of_the_roots.insert(*root, version);
        }
        stream_length_before_the_step = call.return_point;
        steps.push(ReferenceFileSystemStep {
            position,
            events,
            stream_length_after_the_step: call.return_point,
        });
    }
    ReferenceFileSystemStream {
        steps,
        stopped,
        segment_closing_stream_indexes: segment_closing_stream_indexes(operations),
    }
}

/// RefFS 对一个崩溃态的票（读法 1：弃权不算红、也不算绿，单列种类）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum ReferenceFileSystemVoteOfAState {
    Green {
        allowed: String,
    },
    Red {
        reason: String,
        allowed: String,
    },
    Abstained {
        kind: &'static str,
    },
    /// 这一态上恢复 panic：读回没有，RefFS 不判（同 O1-模型崩溃判官，读法 5）。
    RecoveryPanicked,
    /// 这一态上 RefFS 自己 panic（修订 5 第 5.1 节第 3 条）：不红不绿，与弃权分开计。
    JudgePanicked,
    /// 关一条（Q3-核）关掉了 O1：RefFS 子列一起不算。
    Off,
}

impl ReferenceFileSystemVoteOfAState {
    fn word(&self) -> &'static str {
        match self {
            Self::Green { .. } => "green",
            Self::Red { .. } => "red",
            Self::Abstained { .. } => "abstained",
            Self::RecoveryPanicked => "recovery_panic",
            Self::JudgePanicked => "judge_panic",
            Self::Off => "off",
        }
    }
}

/// 允许集合的下标写成 `S_0,S_2`。
fn allowed_state_indexes_text(allowed: &reference_filesystem::AllowedSet) -> String {
    allowed
        .allowed
        .iter()
        .map(|state| format!("S_{}", state.index.0))
        .collect::<Vec<String>>()
        .join(",")
}

/// 崩在第 `segment_index` 段里（序号越过最后一段的那个全持久态交 `None`）的崩溃落在哪一步：关上这一段的那道屏障由哪一步发出
/// （`crash_moment_of_a_segment_closed_at`，H1-收严的归属；段没关上的、全持久态，算在全部步都返回之后）。
fn reference_filesystem_crash_moment(
    stream: &ReferenceFileSystemStream,
    segment_index: Option<usize>,
) -> ReferenceFileSystemCrashMoment {
    let closing_stream_index = segment_index
        .and_then(|segment| stream.segment_closing_stream_indexes.get(segment).copied())
        .flatten();
    let stream_length_after_each_step: Vec<(StepPosition, usize)> = stream
        .steps
        .iter()
        .map(|step| (step.position, step.stream_length_after_the_step))
        .collect();
    crash_moment_of_a_segment_closed_at(closing_stream_index, &stream_length_after_each_step)
}

fn crash_moment_text(moment: ReferenceFileSystemCrashMoment) -> String {
    match moment {
        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::Operation(call_index)) => {
            format!("call_{call_index}")
        }
        ReferenceFileSystemCrashMoment::InsideStep(StepPosition::StartingPoint) => {
            "starting_point".to_string()
        }
        ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep => "after_every_step".to_string(),
    }
}

/// 一个崩溃态上 RefFS 的票（`TranslatedHistory::judge_a_crash` 的同一套走法）：从头交事件到崩溃那一步，hi 取那一步最后进序列的操作号
/// （那一步一个都没进是「都返回过之后」的号）；翻译停在崩溃之前或崩溃落在停下的那一步里弃权；读回失败判红；读回不在允许集合里判红。
// gpu-exempt: reference-filesystem
fn reference_filesystem_vote_of_a_state(
    stream: &ReferenceFileSystemStream,
    moment: ReferenceFileSystemCrashMoment,
    read_back: &ObservedReadBack,
) -> ReferenceFileSystemVoteOfAState {
    use reference_filesystem::{CrashNotAccepted, FileContent, VisibleState};
    let mut model = reference_filesystem::ReferenceFileSystem::on_a_freshly_formatted_pool();
    let mut crash_falls_in = None;
    for step in &stream.steps {
        if let Some((_, stop)) = stream
            .stopped
            .as_ref()
            .filter(|(position, _)| *position == step.position)
        {
            return ReferenceFileSystemVoteOfAState::Abstained { kind: stop.name() };
        }
        let mut last_entered = None;
        for (operation, outcome) in &step.events {
            match model
                .submit(operation.clone(), *outcome)
                .expect("建翻译时同一串事件交给同一个起点的 RefFS 收过")
            {
                reference_filesystem::SubmittedOperation::Entered(number) => {
                    last_entered = Some(number);
                }
                reference_filesystem::SubmittedOperation::DidNotHappen => {}
            }
        }
        if moment == ReferenceFileSystemCrashMoment::InsideStep(step.position) {
            crash_falls_in =
                Some(last_entered.unwrap_or_else(|| model.crash_after_every_operation_returned()));
            break;
        }
    }
    let crash_falls_in =
        crash_falls_in.unwrap_or_else(|| model.crash_after_every_operation_returned());
    let recovered = match read_back {
        ObservedReadBack::FileRead { content, .. } => {
            VisibleState::OneFile(FileContent(content.clone()))
        }
        ObservedReadBack::NoFile { .. } => VisibleState::NoFile,
        ObservedReadBack::Failed { what } => {
            return ReferenceFileSystemVoteOfAState::Red {
                reason: format!("恢复失败、没有读出的状态（{what}）"),
                allowed: "-".to_string(),
            };
        }
    };
    match model.crash_and_recover(crash_falls_in, recovered) {
        Ok(allowed) => ReferenceFileSystemVoteOfAState::Green {
            allowed: allowed_state_indexes_text(&allowed),
        },
        Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { allowed, recovered }) => {
            ReferenceFileSystemVoteOfAState::Red {
                reason: format!(
                    "崩溃落在第 {} 个操作里，读出 {}",
                    crash_falls_in.0,
                    match recovered {
                        VisibleState::NoFile => "没有文件".to_string(),
                        VisibleState::OneFile(FileContent(bytes)) =>
                            format!("文件 {} 字节（末字节 {:?}）", bytes.len(), bytes.last()),
                    }
                ),
                allowed: allowed_state_indexes_text(&allowed),
            }
        }
        Err(CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion { .. }) => {
            ReferenceFileSystemVoteOfAState::Abstained {
                kind: "写后报错之后又起了操作",
            }
        }
        Err(refusal @ CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery { .. }) => {
            panic!("hi 取的是这一步最后进序列的操作号或「都返回过之后」那个号，不会越界：{refusal:?}")
        }
    }
}

// ───── 可写挂载探针（Q4，读法 8、停机 S10、S11） ─────

/// 见证判据计数那一支（读法 8 (ii)）为真的是哪一句（D23（journal 的角色与格式） 已定项 14「可写挂载读到的更新状态读不出」那一段的三句）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WitnessCounterSentenceThatHolds {
    /// 所选那一版那次发布带末条标志的记录读得出，c_见证 大于它的计数器。
    AboveTheLastRecordOfTheSelectedVersion,
    /// 末条读不出，计数器等于 c_见证 的记录读得出、它的 (实例代号, checkpoint_txg) 大于所选那一版。
    RecordAtTheWitnessedCounterIsNewer,
    /// 两条都读不出，按真。
    NeitherRecordReadable,
}

/// 一个崩溃态上见证判据的两支（读法 8，装置在崩溃镜像上自己算，不调第二截与 mount 的私有函数）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct WitnessCriterionOnTheCrashImage {
    /// (i) 判不出那一支：某块盘自证过的系统配置槽少于两槽（`recovery::verified_system_configuration_slots` 现数）。
    some_device_has_fewer_than_two_verified_slots: bool,
    /// c_见证：全部自证过的槽的 journal_tail 取最大（一槽都没有是 0）。
    witnessed_counter: u64,
    /// (ii) 计数那一支：每块盘两槽都自证过、c_见证 ≠ 0、三句里哪一句为真（都不为真是 `None`）。
    counter_sentence: Option<WitnessCounterSentenceThatHolds>,
}

/// 在崩溃镜像上算见证判据的两支。`selected_version` 是只读恢复（看 journal）落到的那一版。
fn witness_criterion_on_the_crash_image(
    crash_image: &MemoryPool,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> WitnessCriterionOnTheCrashImage {
    let Ok(system_configuration) = choose_system_configuration(crash_image) else {
        // 一份可择的系统配置都没有：自证过的槽数必然少于两槽，记 (i)。
        return WitnessCriterionOnTheCrashImage {
            some_device_has_fewer_than_two_verified_slots: true,
            witnessed_counter: 0,
            counter_sentence: None,
        };
    };
    let slot_spacing = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let verified_by_device: Vec<Vec<SystemConfiguration>> = crash_image
        .device_identities()
        .into_iter()
        .map(|device| {
            verified_system_configuration_slots(
                crash_image,
                device,
                slot_spacing,
                &system_configuration.immutable.filesystem_identifier,
            )
        })
        .collect();
    let some_device_has_fewer_than_two_verified_slots = verified_by_device
        .iter()
        .any(|verified| verified.len() < LOCAL_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE);
    let witnessed_counter = verified_by_device
        .iter()
        .flatten()
        .map(|slot| slot.quantities.journal_tail)
        .max()
        .unwrap_or(0);
    let counter_sentence = if some_device_has_fewer_than_two_verified_slots
        || witnessed_counter == 0
    {
        None
    } else {
        selected_version.and_then(|selected| {
            let records = scan_journal(crash_image, &system_configuration);
            let last_record_counter = records
                .values()
                .filter(|record| {
                    (record.instance, record.checkpoint_txg) == selected
                        && record.place_in_publish
                            == JournalRecordPlaceInPublish::LastRecordOfThePublish
                })
                .map(|record| record.counter)
                .max();
            if let Some(last_record_counter) = last_record_counter {
                return (witnessed_counter > last_record_counter).then_some(
                    WitnessCounterSentenceThatHolds::AboveTheLastRecordOfTheSelectedVersion,
                );
            }
            let record_at_the_witnessed_counter = records
                .values()
                .filter(|record| record.counter == witnessed_counter)
                .map(|record| (record.instance, record.checkpoint_txg))
                .max();
            match record_at_the_witnessed_counter {
                Some(record) => (record > selected)
                    .then_some(WitnessCounterSentenceThatHolds::RecordAtTheWitnessedCounterIsNewer),
                None => Some(WitnessCounterSentenceThatHolds::NeitherRecordReadable),
            }
        })
    };
    WitnessCriterionOnTheCrashImage {
        some_device_has_fewer_than_two_verified_slots,
        witnessed_counter,
        counter_sentence,
    }
}

/// 一个崩溃态上可写挂载探针看到的（`writable-mount-probe.tsv` 一行）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct WritableMountProbe {
    criterion: WitnessCriterionOnTheCrashImage,
    selected_version: String,
    /// `mounted`、`refused`。
    mount: &'static str,
    /// 拒的错误成员（`{:?}` 的第一个标识符）与它在不在 D23 已定项 14 那张单子上（不在写 `-`）。
    refusal_member: String,
    refusal_on_the_d23_list: String,
    /// 第二截的判定：`legal`、`illegal:<没满足的条件>`、`reason_not_on_the_d23_list`；挂上了写 `-`。
    second_crop: String,
    /// 拒的理由是系统配置见证（单子上那一条）。
    refused_for_the_system_configuration_witness: bool,
}

/// 第二截判拒绝时交的「这一态没注入读故障」：本装置的崩溃态都是崩溃点重放截断出来的，没有注入任何读故障（读法 8）。
const READ_FAULTS_OF_A_CRASH_STATE: ReadFaultsInTheCrashState =
    ReadFaultsInTheCrashState::NoneInjected;

/// 一个崩溃态上的可写挂载探针（读法 8；入参逐项照崩溃注入第二截那一处取，停机 S10）：崩溃镜像落成内存池
/// （`crash_injection::materialized_crash_image`，第二截同一个函数），在副本上起一次可写挂载（`crash_injection::writable_mount_alone_on`，
/// 空间准入照公式判，同第二截默认的 `SpaceAdmission::JudgedByTheFormula`）；拒了就用第二截同一个调用
/// （`judge_writable_mount_refusal_of_a_crash_state`）判合不合法：恢复落到的版本取看 journal 那一遍的 `effective_root`，盘上同不同比拒之后的池与崩溃镜像，
/// 只读读出在不在允许集合里取 O1-模型崩溃判官红没红（第二截取同一次崩溃上模型判的结论），读故障交「没注入」。
fn writable_mount_probe_on(
    crash_image_materialized: &MemoryPool,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
    crash_judge: &CrashJudgeVerdict,
) -> WritableMountProbe {
    let criterion =
        witness_criterion_on_the_crash_image(crash_image_materialized, selected_version);
    let read_only_read_back = match crash_judge {
        CrashJudgeVerdict::Red { .. } => ReadOnlyReadBackOfTheCrashImage::OutsideTheAllowedSet,
        CrashJudgeVerdict::Green
        | CrashJudgeVerdict::NotApplicable
        | CrashJudgeVerdict::Off
        | CrashJudgeVerdict::RecoveryPanicked
        | CrashJudgeVerdict::JudgePanicked => ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
    };
    let mounted = writable_mount_alone_on(
        crash_image_materialized,
        &common::parameters(),
        SpaceAdmission::JudgedByTheFormula,
    );
    let (mount, refusal_member, refusal_on_the_d23_list, second_crop, refused_for_the_witness) =
        match mounted {
            WritableMountAlone::Mounted { .. } => (
                "mounted",
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
                false,
            ),
            WritableMountAlone::Refused {
                refusal,
                pool_after,
                ..
            } => {
                let on_the_list = refusal_reason_on_the_d23_list(&refusal);
                let disk = if pool_after == *crash_image_materialized {
                    DiskAfterTheRefusal::ByteForByteUnchanged
                } else {
                    DiskAfterTheRefusal::Changed
                };
                let verdict = judge_writable_mount_refusal_of_a_crash_state(
                    crash_image_materialized,
                    selected_version,
                    &refusal,
                    disk,
                    read_only_read_back,
                    READ_FAULTS_OF_A_CRASH_STATE,
                );
                let second_crop = match verdict {
                    WritableMountRefusalVerdict::Legal(_) => "legal".to_string(),
                    WritableMountRefusalVerdict::Illegal {
                        conditions_not_met, ..
                    } => format!(
                        "illegal:{}",
                        conditions_not_met
                            .iter()
                            .map(refusal_condition_text)
                            .collect::<Vec<String>>()
                            .join(",")
                    ),
                    WritableMountRefusalVerdict::ReasonNotOnTheD23List => {
                        "reason_not_on_the_d23_list".to_string()
                    }
                };
                (
                    "refused",
                    error_member_of(&*refusal),
                    on_the_list.map_or_else(|| "-".to_string(), |reason| format!("{reason:?}")),
                    second_crop,
                    on_the_list
                        == Some(
                            RefusalReasonOnTheD23List::NewerPublishWitnessedBySystemConfiguration,
                        ),
                )
            }
        };
    WritableMountProbe {
        criterion,
        selected_version: root_text(selected_version),
        mount,
        refusal_member,
        refusal_on_the_d23_list,
        second_crop,
        refused_for_the_system_configuration_witness: refused_for_the_witness,
    }
}

/// 第二截没满足的条件写成一个词（见证那一条带是哪一支）。
fn refusal_condition_text(condition: &RefusalConditionNotMet) -> String {
    match condition {
        RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage => {
            "criterion_false_when_recomputed".to_string()
        }
        RefusalConditionNotMet::DiskChangedByTheRefusedMount => "disk_changed".to_string(),
        RefusalConditionNotMet::ReadOnlyReadBackOutsideTheAllowedSet => {
            "read_only_read_back_outside_the_allowed_set".to_string()
        }
        RefusalConditionNotMet::WitnessCriterionTrueOnACrashImageWithoutReadFaults(branch) => {
            format!("witness_criterion_true_without_read_faults:{branch:?}")
        }
        RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults => {
            "system_configuration_unreadable_after_the_read_stage_criterion_true_without_read_faults"
                .to_string()
        }
        RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults => {
            "no_valid_root_in_the_root_ring_criterion_true_without_read_faults".to_string()
        }
    }
}

/// 阳性对照「见证」（登记 5.2）：每块盘两槽系统配置的 journal_tail 改成所选那一版末条记录的计数器加一，用 core 的系统配置编码函数
/// （`SystemConfiguration::to_slot`）重编、自证过，写回这份镜像。交回改了几槽（自证不过的槽不动）。
fn raise_the_witnessed_counter_past_the_selected_version(
    pool: &mut MemoryPool,
    selected_version: Option<(InstanceGeneration, CheckpointTxg)>,
) -> usize {
    let Ok(system_configuration) = choose_system_configuration(&*pool) else {
        return 0;
    };
    let Some(selected) = selected_version else {
        return 0;
    };
    let last_record_counter = scan_journal(&*pool, &system_configuration)
        .values()
        .filter(|record| {
            (record.instance, record.checkpoint_txg) == selected
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .map(|record| record.counter)
        .max()
        .unwrap_or(0);
    let slot_spacing = u64::from(
        system_configuration
            .immutable
            .sizes
            .fixed_structure_slot_spacing,
    );
    let slot_bytes = usize::try_from(LOCAL_SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut rewritten = 0usize;
    for device in pool.devices.values_mut() {
        for slot_index in 0..u64::try_from(LOCAL_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).expect("2")
        {
            let offset = DeviceOffsetInBytes(slot_index * slot_spacing);
            let Ok(mut slot) = SystemConfiguration::parse_slot(&device.read(offset, slot_bytes))
            else {
                continue;
            };
            slot.quantities.journal_tail = last_record_counter + 1;
            device.write(offset, &slot.to_slot());
            rewritten += 1;
        }
    }
    rewritten
}

// ───── 一个取样态上的 oracle 与它的记录 ─────

/// 一个取样状态上看到的：批号、崩溃状态序号、段号、落地向量、镜像摘要、`judge_crash_image` 的判定（`impl-verdicts.tsv`）、
/// O1 三个子列与 O2 的结论与各自耗时、触达的最后一次调用、可写挂载探针（开了才有）。看 journal 那一遍恢复 panic 的态 `recovery_panic` 有值
/// （修订 5 第 5.1 节第 5 条：含义收窄，回调链上别的件 panic 不算）；判态路上 panic 了的件记在 `piece_panics`。
struct StateObservation {
    batch: u8,
    ordinal: u64,
    segment_index: usize,
    landing_vector: String,
    image_digest: String,
    verdict: ImplementationVerdict,
    judge_nanoseconds: u128,
    mask_matches_the_facts: bool,
    /// 这一态里持久或撕裂的写属于的最后一次调用（一个写都没落的态是 None）。
    touched_call: Option<usize>,
    crash_judge: CrashJudgeVerdict,
    reference_filesystem: ReferenceFileSystemVoteOfAState,
    /// RefFS 的崩溃时刻（`call_<k>` 或 `after_every_step`）。
    reference_filesystem_crash_moment: String,
    /// O2 直接调 `check_pool_image` 判红的不变量（关掉 O2 时 None）。
    pool_checker_red: Option<Vec<String>>,
    /// S3：直接调的结果与 `judge_crash_image(...).pool_checker` 判红的不变量集合相同。
    pool_checker_matches_the_judgement: bool,
    consulted: RecoverySummary,
    ignored: RecoverySummary,
    /// 看 journal 那一遍恢复落到的版本（实例代号, checkpoint_txg）；Q5 的「已确认的丢了」按它判。
    consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
    probe: Option<WritableMountProbe>,
    recovery_panic: Option<String>,
    o1_nanoseconds: u128,
    o2_nanoseconds: u128,
    recovery_nanoseconds: u128,
    /// 看 journal 那一遍的整份恢复报告：只在 Q5 主取样点按等价类判的那一趟留（类核对 ①、④ 与「等价类对拍」逐字段比它），
    /// 别的趟与恢复 panic 的态是 None。
    consulted_report: Option<RecoveryReport>,
    /// 回调链 panic 了的态上，逐件重做时 panic 的件（[`JudgingPiece`] 的次序）；回调链没 panic 的态是空的（修订 5 第 5.1 节第 5 条）。
    piece_panics: Vec<JudgingPiece>,
    /// 正常那一路（`judge_crash_state_range` 的回调链）panic 了的态是那一次 panic 的消息（不进产物），没 panic 的态是 None。
    /// 有值而 `piece_panics` 是空的态数进 `unattributed_panic_states=`（[`unattributed_panic_states_of`]）。
    judging_path_panic: Option<String>,
}

/// 判一态那条回调链上的一件（修订 5 第 5.1 节第 2 条）：回调链 panic 了，装置按这个次序在同一个镜像上逐件重做、每件各包一层 `catch_unwind`。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum JudgingPiece {
    /// `recover(image, JournalPolicy::Consult)`。
    ConsultedRecovery,
    /// `recover(image, JournalPolicy::Ignore)`。
    IgnoredRecovery,
    /// 被判那次根槽写落没落（`image.persisted[judged_root_index]`）。
    RootPersisted,
    /// 两份 `classified_oracle_violation_for_versions`（连同它们吃的「盘上已持久的最新根」）。
    CrashPointReplayOracles,
    /// `check_pool_image`（`judge_crash_image` 里那次与装置直调那次是同一函数、同一镜像，算一件）。
    PoolChecker,
    /// `check_records(image, consulted.effective_root)`。
    RecordChecker,
    /// O1-模型崩溃判官本身（连同它吃的读回）。
    CrashJudge,
    /// RefFS 一票本身（连同它吃的读回）。
    ReferenceFileSystem,
    /// 计时那一遍恢复。
    TimingRecovery,
    /// 可写挂载探针（开了才有）。
    Probe,
}

/// 判态路上的件数（[`JudgingPiece::ALL`] 的长度）。
const JUDGING_PIECE_COUNT: usize = 10;

impl JudgingPiece {
    const ALL: [JudgingPiece; JUDGING_PIECE_COUNT] = [
        JudgingPiece::ConsultedRecovery,
        JudgingPiece::IgnoredRecovery,
        JudgingPiece::RootPersisted,
        JudgingPiece::CrashPointReplayOracles,
        JudgingPiece::PoolChecker,
        JudgingPiece::RecordChecker,
        JudgingPiece::CrashJudge,
        JudgingPiece::ReferenceFileSystem,
        JudgingPiece::TimingRecovery,
        JudgingPiece::Probe,
    ];

    /// 在 [`JudgingPiece::ALL`] 里的下标。
    fn position(self) -> usize {
        JudgingPiece::ALL
            .iter()
            .position(|piece| *piece == self)
            .expect("ALL 列全了每一件")
    }

    fn name(self) -> &'static str {
        match self {
            JudgingPiece::ConsultedRecovery => "consulted_recovery",
            JudgingPiece::IgnoredRecovery => "ignored_recovery",
            JudgingPiece::RootPersisted => "root_persisted",
            JudgingPiece::CrashPointReplayOracles => "layer0_oracles",
            JudgingPiece::PoolChecker => "pool_checker",
            JudgingPiece::RecordChecker => "record_checker",
            JudgingPiece::CrashJudge => "crash_judge",
            JudgingPiece::ReferenceFileSystem => "reference_filesystem",
            JudgingPiece::TimingRecovery => "timing_recovery",
            JudgingPiece::Probe => "probe",
        }
    }
}

/// 逐件重做时一件的结局：做完了、panic 了（panic 的消息）、没跑（它吃的那一遍恢复 panic 了，或开关没开）。
enum PieceOutcome<Value> {
    Done(Value),
    Panicked(String),
    NotRun,
}

impl<Value> PieceOutcome<Value> {
    fn done(&self) -> Option<&Value> {
        match self {
            PieceOutcome::Done(value) => Some(value),
            PieceOutcome::Panicked(_) | PieceOutcome::NotRun => None,
        }
    }

    /// `impl-verdicts.tsv` 里这一件的格：做完了照 `cell` 写，panic 写 `panic`，没跑写 `not_run`。
    fn cell_text(&self, cell: impl FnOnce(&Value) -> String) -> String {
        match self {
            PieceOutcome::Done(value) => cell(value),
            PieceOutcome::Panicked(_) => "panic".to_string(),
            PieceOutcome::NotRun => "not_run".to_string(),
        }
    }
}

/// 在一个 `catch_unwind` 里做一件；`injected` 是这一件时先 panic（阳性对照「判态路 panic 单列」的注入，[`PiecePanicInjection`]）。
fn judging_piece_outcome<Value>(
    piece: JudgingPiece,
    injected: Option<JudgingPiece>,
    work: impl FnOnce() -> Value,
) -> PieceOutcome<Value> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert!(
            injected != Some(piece),
            "修订 5 阳性对照「判态路 panic 单列」注入：{} 这一件 panic",
            piece.name()
        );
        work()
    })) {
        Ok(value) => PieceOutcome::Done(value),
        Err(payload) => PieceOutcome::Panicked(panic_text(&*payload)),
    }
}

/// 逐件重做时两份崩溃点重放 oracle 的结局：各自吃的那一遍恢复 panic 了是 None（记 `not_run`），跑了是它交回的（没有违例是 `Some(None)`）。
struct CrashPointReplayOraclesOfThePieces {
    consulted: Option<Option<CrashPointReplayOracleViolation>>,
    ignored: Option<Option<CrashPointReplayOracleViolation>>,
}

/// 一件 panic 了就记下它的名字（[`StateObservation::piece_panics`]）。
fn note_a_piece_panic<Value>(
    piece_panics: &mut Vec<JudgingPiece>,
    piece: JudgingPiece,
    outcome: &PieceOutcome<Value>,
) {
    if let PieceOutcome::Panicked(_) = outcome {
        piece_panics.push(piece);
    }
}

/// 一组态上各件 panic 的态数，按 [`JudgingPiece::ALL`] 的次序每件一格（0 也在）。
fn piece_panic_states_of<'observation>(
    observations: impl IntoIterator<Item = &'observation StateObservation>,
) -> [usize; JUDGING_PIECE_COUNT] {
    let mut states = [0usize; JUDGING_PIECE_COUNT];
    for observation in observations {
        for piece in &observation.piece_panics {
            states[piece.position()] += 1;
        }
    }
    states
}

/// 正常那一路 `catch_unwind` 接到 panic、逐件重做十件一件都没 panic 的态数（`unattributed_panic_states=`，
/// 登记「主 agent 认定：27-2 装置交回」第 2 条：甲那条管子漏了一件）。
fn unattributed_panic_states_of<'observation>(
    observations: impl IntoIterator<Item = &'observation StateObservation>,
) -> usize {
    observations
        .into_iter()
        .filter(|observation| {
            observation.judging_path_panic.is_some() && observation.piece_panics.is_empty()
        })
        .count()
}

/// `name=batch`、`name=verdict` 两种行的 `piece_panic_states=` 字段：`<件>:<态数>` 逗号连起来，0 的件不写，全 0 写 `-`。
fn piece_panic_states_text(states: &[usize; JUDGING_PIECE_COUNT]) -> String {
    let nonzero: Vec<String> = JudgingPiece::ALL
        .iter()
        .zip(states)
        .filter(|(_, count)| **count > 0)
        .map(|(piece, count)| format!("{}:{count}", piece.name()))
        .collect();
    if nonzero.is_empty() {
        "-".to_string()
    } else {
        nonzero.join(",")
    }
}

/// `name=q5_piece_panics` 那一行：十件各一个字段（修订 5 第 5.1 节第 5 条），再带 `unattributed_panic_states=`。
fn q5_piece_panics_line(sampling_point: &str, observations: &[StateObservation]) -> String {
    let states = piece_panic_states_of(observations);
    let fields: Vec<String> = JudgingPiece::ALL
        .iter()
        .zip(states)
        .map(|(piece, count)| format!("{}={count}", piece.name()))
        .collect();
    format!(
        "name=q5_piece_panics sampling_point={sampling_point} judged_states={} {} unattributed_panic_states={}",
        observations.len(),
        fields.join(" "),
        unattributed_panic_states_of(observations)
    )
}

/// 一组态里有 [`unattributed_panic_states_of`] 数到的态（登记「主 agent 认定：27-2 装置交回」第 2 条）：这一趟的 Q5 不判甲乙、不进 S13，
/// 打 `q5_not_run=unattributed_panic` 的判决行、收尾，交回退出码 3（交主 agent）；一态都没有交 `None`，调用方接着判。
fn stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
    lines: &mut ResultLines,
    failed_checks: usize,
    observations: &[StateObservation],
) -> Option<i32> {
    let unattributed_panic_states = unattributed_panic_states_of(observations);
    if unattributed_panic_states == 0 {
        return None;
    }
    lines.emit_result_line(&format!(
        "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=unattributed_panic piece_panic_states={} unattributed_panic_states={unattributed_panic_states}",
        piece_panic_states_text(&piece_panic_states_of(observations))
    ));
    lines.finish();
    Some(3)
}

/// 阳性对照「判态路 panic 单列」的注入（修订 5 第 5.1 节）：按落地串选态，选中的态正常那一路在回调里先 panic（逼走逐件重做那一路），
/// 逐件重做时点名的那一件 panic。只经装置内部造（[`piece_panic_control_injection`] 与内联用例），命令行不开放；不改实现。
struct PiecePanicInjection {
    rules: Vec<PiecePanicInjectionRule>,
}

/// 一条注入：落地串满足全部条件的态，正常那一路 panic、逐件重做时 `piece` 那一件 panic。各条选中的态互不相交
/// （造的人保证；[`PiecePanicInjection::rule_of`] 取第一条）。
struct PiecePanicInjectionRule {
    /// `None`：逐件重做时一件都不 panic，只有正常那一路 panic（造 `unattributed_panic_states=` 数的那种态，只有内联用例造）。
    piece: Option<JudgingPiece>,
    conditions: Vec<InjectedLandingCondition>,
}

/// 落地串上的一个条件（写表下标；「落」= 那一位不是 `0`）。
enum InjectedLandingCondition {
    Landed(usize),
    NotLanded(usize),
    BothOrNeitherLanded(usize, usize),
}

impl PiecePanicInjection {
    /// 这一态（落地串）选中的那一条注入；没有一条选中交 `None`。
    fn rule_of(&self, landing_vector: &str) -> Option<&PiecePanicInjectionRule> {
        let landed = |write_index: usize| {
            landing_vector
                .as_bytes()
                .get(write_index)
                .is_some_and(|landing| *landing != b'0')
        };
        self.rules.iter().find(|rule| {
            rule.conditions.iter().all(|condition| match condition {
                InjectedLandingCondition::Landed(write_index) => landed(*write_index),
                InjectedLandingCondition::NotLanded(write_index) => !landed(*write_index),
                InjectedLandingCondition::BothOrNeitherLanded(first, second) => {
                    landed(*first) == landed(*second)
                }
            })
        })
    }
}

/// 一次观测的状态：阳性对照里「批 1 第一个读回了文件的态」只翻一次（O1-崩溃判官与 RefFS 各一次）、「见证」只改一次。
struct ObservationControls {
    switches: RunSwitches,
    crash_judge_control_pending: bool,
    reference_filesystem_control_pending: bool,
    witness_control_pending: bool,
    consulted_reports: ConsultedReports,
}

/// 观测时留不留看 journal 那一遍的整份恢复报告（[`StateObservation::consulted_report`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConsultedReports {
    Dropped,
    Kept,
}

/// 把写表每一项落在哪一次调用里算出来：写在录制流里的下标落在 `first_operation` 不超过它的最后一次调用里。
fn call_index_of_each_write(stream: &PreparedStream) -> Vec<Option<usize>> {
    stream
        .stream_indexes
        .iter()
        .map(|operation_index| {
            stream
                .calls
                .iter()
                .rposition(|call| call.first_operation <= *operation_index)
        })
        .collect()
}

/// O1-模型崩溃判官在一个状态上的判定：交法照 `crash_injection.rs` 的第一截——「盘上已持久的最新根槽」由 `newest_persisted_root`
/// 从写流数出（只看写表里的原写，不含撕裂镜像），读回由 `observed_read_back_after_a_crash` 从看 journal 那一遍恢复报告造，
/// 模型那一侧给每一步之后并起来的版本目录，交给 `crash_recovery_disagreement`（S8：三处调用都在这个函数里）。
fn crash_judge_of_the_state(
    stream: &PreparedStream,
    image: &CrashImage<'_>,
    read_back: &ObservedReadBack,
) -> CrashJudgeVerdict {
    let newest_persisted_root_key =
        newest_persisted_root(&stream.writes, &image.persisted[..stream.writes.len()])
            .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
    crash_judge_verdict(
        &stream.committed_versions,
        newest_persisted_root_key,
        read_back,
    )
}

/// 模型的 `crash_recovery_disagreement` 判一态读回的版本：有不同记红（带实现那一侧说的话），没有不同记绿。
fn crash_judge_verdict(
    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    newest_persisted_root_key: Option<ModelRootKey>,
    read_back: &ObservedReadBack,
) -> CrashJudgeVerdict {
    match crash_recovery_disagreement(committed_versions, newest_persisted_root_key, read_back) {
        Some(disagreement) => CrashJudgeVerdict::Red {
            what: disagreement
                .implementation_answer
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("_"),
        },
        None => CrashJudgeVerdict::Green,
    }
}

/// 一态上 O1-崩溃判官适用不适用（读法 6）：这一态里有写落在模型停下的那次调用（含）之后的调用里，模型的版本目录里没有那些调用写出的版本，不适用。
fn crash_judge_is_applicable(
    touched_call: Option<usize>,
    model_stopped_at_call: Option<usize>,
) -> bool {
    !touched_call.is_some_and(|call| model_stopped_at_call.is_some_and(|stopped| call >= stopped))
}

/// 读回内容末字节翻一位（阳性对照 O1-崩溃判官、RefFS）；读回里没有文件时交回 false、不动。
fn flip_the_last_byte_of_a_read_back(read_back: &mut ObservedReadBack) -> bool {
    if let ObservedReadBack::FileRead { content, .. } = read_back {
        if let Some(last_byte) = content.last_mut() {
            *last_byte ^= 1;
            return true;
        }
    }
    false
}

/// 按序号升序判一串连续的序号段；某一段判到一半 panic（`judge_run` 交回 panic 的那个序号）时，把那个序号记成恢复 panic 的态、
/// 从它的下一个接着判（态上的恢复 panic 不截断，读法 5：各态是互不依赖的镜像，截断会让判了几态取决于判的次序）。
/// 迭代上界：每一轮要么判完一段、要么至少吃掉一个序号；跨轮带的是还没判的那一截。
fn judge_runs_continuing_past_panicking_states(
    runs: &[std::ops::Range<u64>],
    judge_run: &mut dyn FnMut(std::ops::Range<u64>) -> Result<(), u64>,
    note_a_panicking_state: &mut dyn FnMut(u64),
) {
    for run in runs {
        let mut remaining = run.clone();
        while !remaining.is_empty() {
            match judge_run(remaining.clone()) {
                Ok(()) => break,
                Err(panicking_ordinal) => {
                    assert!(
                        remaining.contains(&panicking_ordinal),
                        "panic 的序号 {panicking_ordinal} 在这一段 {remaining:?} 里"
                    );
                    note_a_panicking_state(panicking_ordinal);
                    remaining = panicking_ordinal + 1..remaining.end;
                }
            }
        }
    }
}

/// 观测要的、与这一条流绑在一起的几样：流、枚举表、按序号现算掩码的事实表、撕裂镜像的下标、RefFS 的翻译。
struct ObservationContext<'stream> {
    stream: &'stream PreparedStream,
    tables: &'stream CrashStateEnumerationTables,
    facts: &'stream StateMasks,
    torn_image_index_by_write: &'stream BTreeMap<usize, usize>,
    reference_filesystem: &'stream ReferenceFileSystemStream,
    /// 写表每一项落在哪一次调用里（[`call_index_of_each_write`]，整条流算一次）。
    call_of_write: Vec<Option<usize>>,
    /// 判态路 panic 的注入：跑法里只有阳性对照「判态路 panic 单列」那一遍有，别的都是 None；内联用例 U13–U16 另经
    /// [`ConfirmedLossPiecePanicInjections`] 交给 Q5 的对拍、阳性对照与取样点导出（登记修订 6 第 6.2 节第 3 条）。
    piece_panic_injection: Option<&'stream PiecePanicInjection>,
}

impl ObservationContext<'_> {
    fn segment_of_ordinal(&self, ordinal: u64) -> usize {
        self.tables
            .state_ranges_by_segment
            .partition_point(|state_range| state_range.end <= ordinal)
    }
    /// RefFS 崩溃时刻要的段号：序号越过最后一段的那个全持久态交 `None`。
    fn reference_filesystem_moment_of(
        &self,
        segment_index: usize,
    ) -> ReferenceFileSystemCrashMoment {
        let segment = (segment_index < self.stream.segments.len()).then_some(segment_index);
        reference_filesystem_crash_moment(self.reference_filesystem, segment)
    }
}

/// 一个判完了的态（`judge_crash_image` 交回了判定）上的全部 oracle（O1 三个子列、O2、探针）。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
fn observation_of_a_judged_state(
    context: &ObservationContext<'_>,
    batch: u8,
    ordinal: u64,
    image: &CrashImage<'_>,
    judgement: &CrashStateJudgement,
    controls: &mut ObservationControls,
) -> StateObservation {
    let stream = context.stream;
    let switches = controls.switches;
    let landing_vector = landing_vector_of(
        &image.persisted,
        stream.writes.len(),
        context.torn_image_index_by_write,
    );
    // 阳性对照「判态路 panic 单列」（修订 5 第 5.1 节）：选中的态在回调里先 panic，逼走逐件重做那一路。
    assert!(
        context
            .piece_panic_injection
            .and_then(|injection| injection.rule_of(&landing_vector))
            .is_none(),
        "修订 5 阳性对照「判态路 panic 单列」注入：正常那一路在这一态的回调里先 panic"
    );
    let touched_call = landing_vector
        .chars()
        .zip(&context.call_of_write)
        .filter(|(landing, _)| *landing != '0')
        .filter_map(|(_, call)| *call)
        .max();
    let verdict = ImplementationVerdict::of(judgement);
    let segment_index = context.segment_of_ordinal(ordinal);
    let model_is_enabled =
        switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge);
    let read_back = observed_read_back_after_a_crash(&judgement.consulted);
    let crash_judge_started = Instant::now();
    let crash_judge = if !model_is_enabled {
        CrashJudgeVerdict::Off
    } else if !crash_judge_is_applicable(touched_call, stream.model_stopped_at_call) {
        CrashJudgeVerdict::NotApplicable
    } else {
        let mut judged_read_back = read_back.clone();
        if controls.crash_judge_control_pending
            && batch == 1
            && flip_the_last_byte_of_a_read_back(&mut judged_read_back)
        {
            controls.crash_judge_control_pending = false;
        }
        crash_judge_of_the_state(stream, image, &judged_read_back)
    };
    let moment = context.reference_filesystem_moment_of(segment_index);
    let reference_filesystem = if model_is_enabled {
        let mut judged_read_back = read_back.clone();
        if controls.reference_filesystem_control_pending
            && batch == 1
            && flip_the_last_byte_of_a_read_back(&mut judged_read_back)
        {
            controls.reference_filesystem_control_pending = false;
        }
        reference_filesystem_vote_of_a_state(
            context.reference_filesystem,
            moment,
            &judged_read_back,
        )
    } else {
        ReferenceFileSystemVoteOfAState::Off
    };
    let o1_nanoseconds = crash_judge_started.elapsed().as_nanos();
    let pool_checker_control = switches.positive_control == Some(PositiveControl::PoolChecker)
        && stream.name == StreamName::NewPoolFileCreation
        && ordinal == context.tables.state_count - 1;
    // O2 逐状态计时量的是 CPU 池级 checker 本身（豁免截名 experiment-measures-the-cpu-judge-itself）：CPU 这一遍只计时；
    // 判红的名单走两段（同一张盘作一张给定状态表交引擎，阳性对照那一态是改坏的那一张），不在计时里
    let pool_checker_started = Instant::now();
    let pool_checker_timed_on_the_cpu =
        (switches.disabled_oracle != Some(DisabledOracle::PoolChecker)).then(|| {
            if pool_checker_control {
                pool_checker_on_the_first_known_bad_image(image)
            } else {
                check_pool_image(image)
            }
        });
    let o2_nanoseconds = pool_checker_started.elapsed().as_nanos();
    let image_digest = sha256_text(&crash_image_digest(image));
    // None 有两种：O2 这路 oracle 关着；或引擎这一趟只录没判（占位，判归编进 gpu 特性的那一趟）——两种都不与判决比
    let pool_checker_red = pool_checker_timed_on_the_cpu.and_then(|_| {
        pool_checker_red_names_of_a_sampled_state_through_the_given_states_engine(
            stream.name,
            ordinal,
            &image_digest,
            image,
            pool_checker_control,
        )
    });
    let pool_checker_matches_the_judgement = pool_checker_red
        .as_ref()
        .is_none_or(|direct| *direct == verdict.pool_checker_red)
        || pool_checker_control;
    let recovery_started = Instant::now();
    let _ =
        singlefs_core::recovery::recover(image, singlefs_core::recovery::JournalPolicy::Consult);
    let recovery_nanoseconds = recovery_started.elapsed().as_nanos();
    let probe = switches.writable_mount_probe.then(|| {
        let mut materialized = materialized_crash_image(image);
        let mut selected_version = judgement.consulted.effective_root;
        if controls.witness_control_pending
            && stream.name == StreamName::NewPoolFileCreation
            && ordinal == context.tables.state_count - 1
        {
            controls.witness_control_pending = false;
            raise_the_witnessed_counter_past_the_selected_version(
                &mut materialized,
                selected_version,
            );
            selected_version = singlefs_core::recovery::recover(
                &materialized,
                singlefs_core::recovery::JournalPolicy::Consult,
            )
            .effective_root;
        }
        writable_mount_probe_on(&materialized, selected_version, &crash_judge)
    });
    StateObservation {
        batch,
        ordinal,
        segment_index,
        landing_vector,
        image_digest,
        verdict,
        judge_nanoseconds: o1_nanoseconds + o2_nanoseconds + recovery_nanoseconds,
        mask_matches_the_facts: context.facts.persisted_writes_of_state(ordinal) == image.persisted,
        touched_call,
        crash_judge,
        reference_filesystem,
        reference_filesystem_crash_moment: crash_moment_text(moment),
        pool_checker_red,
        pool_checker_matches_the_judgement,
        consulted: RecoverySummary::of(&judgement.consulted),
        ignored: RecoverySummary::of(&judgement.ignored),
        consulted_root: judgement.consulted.effective_root,
        probe,
        recovery_panic: None,
        o1_nanoseconds,
        o2_nanoseconds,
        recovery_nanoseconds,
        consulted_report: match controls.consulted_reports {
            ConsultedReports::Dropped => None,
            ConsultedReports::Kept => Some(judgement.consulted.clone()),
        },
        piece_panics: Vec::new(),
        judging_path_panic: None,
    }
}

/// 回调链 panic 了的那一态（`judge_crash_image` 里 panic，或判这一态的 oracle 里 panic）：镜像按事实表的掩码现造，照修订 5 第 5.1 节第 2 条
/// 的次序逐件重做、每件各包一层 `catch_unwind`（[`JudgingPiece`]），panic 了记下件名、接着做下一件。O1 两个崩溃子列只随看 journal 那遍恢复：
/// 它 panic 两列都记恢复 panic（读法 5），没 panic 两列照正常那一路的函数从这一遍的恢复报告算，O1-模型崩溃判官或 RefFS 自己 panic 那一列记
/// `judge_panic`；别的件 panic 不改这两列（第 3 条）。O2 照判，panic 记 `pool_checker_panic`、按红计（第 4 条）。
/// 阳性对照的翻读回、见证改写与 O2 的坏镜像都只在正常那一路做，这一路不做。`judging_path_panic` 是正常那一路 panic 的消息，
/// 记进 [`StateObservation::judging_path_panic`]、不进产物。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
fn observation_of_a_panicking_state(
    context: &ObservationContext<'_>,
    batch: u8,
    ordinal: u64,
    switches: RunSwitches,
    consulted_reports: ConsultedReports,
    judging_path_panic: String,
) -> StateObservation {
    let stream = context.stream;
    let persisted = context.facts.persisted_writes_of_state(ordinal);
    let image = CrashImage {
        base: &stream.base,
        writes: &context.tables.enumerated_writes,
        persisted,
    };
    let landing_vector = landing_vector_of(
        &image.persisted,
        stream.writes.len(),
        context.torn_image_index_by_write,
    );
    let touched_call = landing_vector
        .chars()
        .zip(&context.call_of_write)
        .filter(|(landing, _)| *landing != '0')
        .filter_map(|(_, call)| *call)
        .max();
    let segment_index = context.segment_of_ordinal(ordinal);
    let injected = context
        .piece_panic_injection
        .and_then(|injection| injection.rule_of(&landing_vector))
        .and_then(|rule| rule.piece);
    let mut piece_panics: Vec<JudgingPiece> = Vec::new();
    let consulted = judging_piece_outcome(JudgingPiece::ConsultedRecovery, injected, || {
        singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Consult)
    });
    note_a_piece_panic(
        &mut piece_panics,
        JudgingPiece::ConsultedRecovery,
        &consulted,
    );
    let ignored = judging_piece_outcome(JudgingPiece::IgnoredRecovery, injected, || {
        singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Ignore)
    });
    note_a_piece_panic(&mut piece_panics, JudgingPiece::IgnoredRecovery, &ignored);
    let root_persisted = judging_piece_outcome(JudgingPiece::RootPersisted, injected, || {
        image.persisted[stream.judged_root_index]
    });
    note_a_piece_panic(
        &mut piece_panics,
        JudgingPiece::RootPersisted,
        &root_persisted,
    );
    // 两份崩溃点重放 oracle 各吃自己那一遍恢复：那一遍 panic 了，那一份记 `not_run`；两遍都 panic 了这一件整个不跑。
    let crash_point_replay_oracles = if consulted.done().is_none() && ignored.done().is_none() {
        PieceOutcome::NotRun
    } else {
        judging_piece_outcome(JudgingPiece::CrashPointReplayOracles, injected, || {
            let newest_persisted = newest_persisted_root(image.writes, &image.persisted);
            let oracle_of = |report: &RecoveryReport| {
                classified_oracle_violation_for_versions(
                    &report.outcome,
                    report.effective_root,
                    newest_persisted,
                    &stream.versions,
                )
            };
            CrashPointReplayOraclesOfThePieces {
                consulted: consulted.done().map(oracle_of),
                ignored: ignored.done().map(oracle_of),
            }
        })
    };
    note_a_piece_panic(
        &mut piece_panics,
        JudgingPiece::CrashPointReplayOracles,
        &crash_point_replay_oracles,
    );
    let pool_checker_started = Instant::now();
    let pool_checker = judging_piece_outcome(JudgingPiece::PoolChecker, injected, || {
        check_pool_image(&image)
    });
    let o2_nanoseconds = pool_checker_started.elapsed().as_nanos();
    note_a_piece_panic(&mut piece_panics, JudgingPiece::PoolChecker, &pool_checker);
    let record_checker = match consulted.done() {
        Some(report) => judging_piece_outcome(JudgingPiece::RecordChecker, injected, || {
            check_records(&image, report.effective_root)
        }),
        None => PieceOutcome::NotRun,
    };
    note_a_piece_panic(
        &mut piece_panics,
        JudgingPiece::RecordChecker,
        &record_checker,
    );
    // 两个崩溃子列只随看 journal 那遍恢复 panic（第 3 条）：别的件 panic 不改这两列。
    let columns_follow_a_recovery_panic = matches!(consulted, PieceOutcome::Panicked(_));
    let model_is_enabled =
        switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge);
    let moment = context.reference_filesystem_moment_of(segment_index);
    let crash_judge_started = Instant::now();
    let crash_judge = match consulted.done() {
        _ if columns_follow_a_recovery_panic => CrashJudgeVerdict::RecoveryPanicked,
        None => CrashJudgeVerdict::RecoveryPanicked,
        Some(_) if !model_is_enabled => CrashJudgeVerdict::Off,
        Some(_) if !crash_judge_is_applicable(touched_call, stream.model_stopped_at_call) => {
            CrashJudgeVerdict::NotApplicable
        }
        Some(report) => {
            let outcome = judging_piece_outcome(JudgingPiece::CrashJudge, injected, || {
                crash_judge_of_the_state(stream, &image, &observed_read_back_after_a_crash(report))
            });
            note_a_piece_panic(&mut piece_panics, JudgingPiece::CrashJudge, &outcome);
            match outcome {
                PieceOutcome::Done(verdict) => verdict,
                PieceOutcome::Panicked(_) | PieceOutcome::NotRun => {
                    CrashJudgeVerdict::JudgePanicked
                }
            }
        }
    };
    let reference_filesystem = match consulted.done() {
        _ if columns_follow_a_recovery_panic => ReferenceFileSystemVoteOfAState::RecoveryPanicked,
        None => ReferenceFileSystemVoteOfAState::RecoveryPanicked,
        Some(_) if !model_is_enabled => ReferenceFileSystemVoteOfAState::Off,
        Some(report) => {
            let outcome =
                judging_piece_outcome(JudgingPiece::ReferenceFileSystem, injected, || {
                    reference_filesystem_vote_of_a_state(
                        context.reference_filesystem,
                        moment,
                        &observed_read_back_after_a_crash(report),
                    )
                });
            note_a_piece_panic(
                &mut piece_panics,
                JudgingPiece::ReferenceFileSystem,
                &outcome,
            );
            match outcome {
                PieceOutcome::Done(vote) => vote,
                PieceOutcome::Panicked(_) | PieceOutcome::NotRun => {
                    ReferenceFileSystemVoteOfAState::JudgePanicked
                }
            }
        }
    };
    let o1_nanoseconds = crash_judge_started.elapsed().as_nanos();
    let recovery_started = Instant::now();
    let timing_recovery = judging_piece_outcome(JudgingPiece::TimingRecovery, injected, || {
        let _ = singlefs_core::recovery::recover(
            &image,
            singlefs_core::recovery::JournalPolicy::Consult,
        );
    });
    let recovery_nanoseconds = recovery_started.elapsed().as_nanos();
    note_a_piece_panic(
        &mut piece_panics,
        JudgingPiece::TimingRecovery,
        &timing_recovery,
    );
    let probe = match (switches.writable_mount_probe, consulted.done()) {
        (true, Some(report)) => {
            let outcome = judging_piece_outcome(JudgingPiece::Probe, injected, || {
                writable_mount_probe_on(
                    &materialized_crash_image(&image),
                    report.effective_root,
                    &crash_judge,
                )
            });
            note_a_piece_panic(&mut piece_panics, JudgingPiece::Probe, &outcome);
            match outcome {
                PieceOutcome::Done(probe) => Some(probe),
                PieceOutcome::Panicked(_) | PieceOutcome::NotRun => None,
            }
        }
        (true, None) | (false, _) => None,
    };
    let pool_checker_red = (switches.disabled_oracle != Some(DisabledOracle::PoolChecker)).then(
        || match &pool_checker {
            PieceOutcome::Done(verdicts) => violated_invariant_names(verdicts),
            PieceOutcome::Panicked(_) | PieceOutcome::NotRun => {
                vec!["pool_checker_panic".to_string()]
            }
        },
    );
    let verdict = ImplementationVerdict::of_the_pieces(
        &consulted,
        &ignored,
        &crash_point_replay_oracles,
        &pool_checker,
        &record_checker,
    );
    StateObservation {
        batch,
        ordinal,
        segment_index,
        landing_vector,
        image_digest: sha256_text(&crash_image_digest(&image)),
        verdict,
        judge_nanoseconds: o1_nanoseconds + o2_nanoseconds + recovery_nanoseconds,
        mask_matches_the_facts: true,
        touched_call,
        crash_judge,
        reference_filesystem,
        reference_filesystem_crash_moment: crash_moment_text(moment),
        pool_checker_red,
        pool_checker_matches_the_judgement: true,
        consulted: consulted.done().map_or_else(
            RecoverySummary::of_a_panicking_recovery,
            RecoverySummary::of,
        ),
        ignored: ignored.done().map_or_else(
            RecoverySummary::of_a_panicking_recovery,
            RecoverySummary::of,
        ),
        consulted_root: consulted.done().and_then(|report| report.effective_root),
        probe,
        recovery_panic: match &consulted {
            PieceOutcome::Panicked(message) => Some(message.clone()),
            PieceOutcome::Done(_) | PieceOutcome::NotRun => None,
        },
        o1_nanoseconds,
        o2_nanoseconds,
        recovery_nanoseconds,
        consulted_report: match (consulted_reports, consulted.done()) {
            (ConsultedReports::Kept, Some(report)) => Some(report.clone()),
            (ConsultedReports::Kept, None) | (ConsultedReports::Dropped, _) => None,
        },
        piece_panics,
        judging_path_panic: Some(judging_path_panic),
    }
}

/// 按序号升序把取样的状态切成连续的几段，每段调一次 `judge_crash_state_range`：回调里给的那个 `CrashImage` 上算全部 oracle
/// （[`observation_of_a_judged_state`]）。某个状态上 panic：记成恢复 panic 的态、接着判它后面的（[`judge_runs_continuing_past_panicking_states`]）。
fn observe_sampled_states(
    context: &ObservationContext<'_>,
    sampled: &SampledOrdinals,
    switches: RunSwitches,
    consulted_reports: ConsultedReports,
) -> Vec<StateObservation> {
    let mut batch_of_ordinal: BTreeMap<u64, u8> = BTreeMap::new();
    for (batch_offset, batch) in sampled.batches.iter().enumerate() {
        for ordinal in batch {
            let batch_number = u8::try_from(batch_offset + 1).expect("三批");
            assert!(
                batch_of_ordinal.insert(*ordinal, batch_number).is_none(),
                "三批互不重叠"
            );
        }
    }
    let mut runs: Vec<std::ops::Range<u64>> = Vec::new();
    for ordinal in batch_of_ordinal.keys().copied() {
        match runs.last_mut() {
            Some(run) if run.end == ordinal => run.end += 1,
            Some(_) | None => runs.push(ordinal..ordinal + 1),
        }
    }
    let stream = context.stream;
    let mut controls = ObservationControls {
        switches,
        crash_judge_control_pending: switches.positive_control == Some(PositiveControl::CrashJudge),
        reference_filesystem_control_pending: switches.positive_control
            == Some(PositiveControl::ReferenceFileSystem),
        witness_control_pending: switches.positive_control == Some(PositiveControl::Witness),
        consulted_reports,
    };
    let mut observations: Vec<StateObservation> = Vec::new();
    let mut panicking_states: Vec<(u64, String)> = Vec::new();
    // panic 的序号与消息在 `judge_run` 里记（它手里有 panic 的消息）；块之后只核每一个都记到了。`judge_run` 借着
    // `observations` 与 `panicking_states`，放进这一块，块结束借用就还回来。
    let mut panicking_ordinals_noted: Vec<u64> = Vec::new();
    {
        let mut judge_run = |run: std::ops::Range<u64>| -> Result<(), u64> {
            let mut next_ordinal = run.start;
            let judged =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    sample_crash_states_on_the_cpu(
                CrashPointReplayItem {
                    name: &sampling_item_name_of_this_call(),
                    setting_file: CRASH_POINT_REPLAY_SAMPLING_ITEM.setting_file,
                },
                    &stream.base,
                    &stream.writes,
                    &stream.segments,
                    stream.judged_root_index,
                    &stream.versions,
                    &full_expansion,
                    run.clone(),
                    &mut |ordinal, image, judgement| {
                        observations.push(observation_of_a_judged_state(
                            context,
                            batch_of_ordinal[&ordinal],
                            ordinal,
                            image,
                            judgement,
                            &mut controls,
                        ));
                        next_ordinal = ordinal + 1;
                    },
                )
        .judged()
        .expect("实验装置不分只录、只判两趟跑：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE")
                }));
            match judged {
                Ok(state_count_of_the_stream) => {
                    assert_eq!(
                        state_count_of_the_stream, context.tables.state_count,
                        "judge_crash_state_range 交回的整条流状态数与枚举表的相同"
                    );
                    Ok(())
                }
                Err(payload) => {
                    panicking_states.push((next_ordinal, panic_text(&*payload)));
                    Err(next_ordinal)
                }
            }
        };
        judge_runs_continuing_past_panicking_states(&runs, &mut judge_run, &mut |ordinal| {
            panicking_ordinals_noted.push(ordinal);
        });
    }
    assert_eq!(
        panicking_ordinals_noted,
        panicking_states
            .iter()
            .map(|(ordinal, _)| *ordinal)
            .collect::<Vec<u64>>(),
        "panic 的态两处记的一样"
    );
    // 正常那一路 panic 的消息不进产物：逐件重做时各件自己的结局才是这一态的判定（修订 5 第 5.1 节第 2 条）。
    for (ordinal, message_of_the_judging_path) in panicking_states {
        observations.push(observation_of_a_panicking_state(
            context,
            batch_of_ordinal[&ordinal],
            ordinal,
            switches,
            consulted_reports,
            message_of_the_judging_path,
        ));
    }
    observations.sort_by_key(|observation| (observation.batch, observation.ordinal));
    observations
}

// ───────────────────────────── 一条流导出（登记 5.3 的十个文件，照抄 E170 的写出） ─────────────────────────────

/// 写出的一个文件：名字、字节数、sha256；带耗时的两个文件另算一份去掉耗时之后的 sha256（登记第十二节修订 V2b）。
struct WrittenFile {
    name: &'static str,
    bytes: usize,
    sha256: String,
    sha256_without_timing: Option<String>,
}

/// `impl-verdicts.tsv` 每行去掉最后一列（耗时）；`manifest.tsv` 去掉记 `impl-verdicts.tsv` 摘要的那一行。其余文件原样。
fn contents_without_timing(file_name: &str, contents: &[u8]) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(contents).ok()?;
    let kept: Vec<String> = match file_name {
        IMPLEMENTATION_VERDICTS_FILE => text
            .lines()
            .map(|line| {
                line.rsplit_once('\t')
                    .map_or(line, |(without_timing, _)| without_timing)
                    .to_string()
            })
            .collect(),
        MANIFEST_FILE => text
            .lines()
            .filter(|line| !line.starts_with(&format!("sha256:{IMPLEMENTATION_VERDICTS_FILE}\t")))
            .map(str::to_string)
            .collect(),
        _ => return None,
    };
    Some(
        kept.iter()
            .flat_map(|line| format!("{line}\n").into_bytes())
            .collect(),
    )
}

fn write_export_file(directory: &Path, name: &'static str, contents: &[u8]) -> WrittenFile {
    std::fs::write(directory.join(name), contents)
        .unwrap_or_else(|error| panic!("写 {}：{error}", directory.join(name).display()));
    WrittenFile {
        name,
        bytes: contents.len(),
        sha256: sha256_hexadecimal(contents),
        sha256_without_timing: contents_without_timing(name, contents)
            .map(|kept| sha256_hexadecimal(&kept)),
    }
}

/// `base-sectors.bin` 的内容（登记 5.3）：基镜像里每个不全为零的扇区，盘身份 u32、扇区号 u64、512 字节。
/// 第一段写导出、第三段核导出与重驱动的流相同（`bad-states` 模式）共用这一份。
fn base_sectors_file_bytes(stream: &PreparedStream) -> Vec<u8> {
    let mut base_sectors = Vec::new();
    for (device, sparse) in &stream.base.devices {
        for (sector, bytes) in sparse.written_sectors() {
            if bytes.iter().all(|byte| *byte == 0) {
                continue;
            }
            base_sectors.extend_from_slice(&device.0.to_le_bytes());
            base_sectors.extend_from_slice(&sector.to_le_bytes());
            base_sectors.extend_from_slice(bytes);
        }
    }
    base_sectors
}

/// `ops.tsv` 的文本与 `ops-payload.bin` 的字节（登记 5.3），同上两种模式共用。
fn operations_file_text_and_payload(stream: &PreparedStream) -> (String, Vec<u8>) {
    let mut operations_text = String::new();
    let mut payload = Vec::new();
    for (stream_index, retained) in stream.operations.iter().enumerate() {
        let operation = &retained.operation;
        let (payload_start, content_digest) = match operation.kind {
            RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                let contents = retained
                    .contents
                    .as_ref()
                    .expect("流开了内容保留，普通写与 FUA 写带字节");
                let start = payload.len();
                payload.extend_from_slice(contents);
                (start.to_string(), sha256_hexadecimal(contents))
            }
            RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier => {
                ("-".to_string(), "-".to_string())
            }
        };
        operations_text.push_str(&format!(
            "{stream_index}\t{}\t{}\t{}\t{}\t{payload_start}\t{content_digest}\n",
            operation.device.0,
            operation_kind_name(operation.kind),
            operation.offset.0,
            operation.length,
        ));
    }
    (operations_text, payload)
}

/// `calls.tsv` 的文本（E170 登记 5.3）：调用序号、名字、发出的第一条 op 的流序下标、返回点、`ok` / `err`。
fn calls_file_text(stream: &PreparedStream) -> String {
    stream
        .calls
        .iter()
        .enumerate()
        .map(|(call_index, call)| {
            format!(
                "{call_index}\t{}\t{}\t{}\t{}\n",
                call.label,
                call.first_operation,
                call.return_point,
                call.outcome.calls_file_word()
            )
        })
        .collect()
}

/// tsv 里的自由文本：制表符与换行换成空格。
fn cell_text(text: &str) -> String {
    text.replace(['\t', '\n', '\r'], " ")
}

/// `call-outcomes.tsv`：每次调用一行——序号、名字、第一条 op、返回点、结局（`ok` / `err:<成员>` / `panic` / `in_flight`）、O1-对拍的判定、
/// 对不上时模型与实现各说了什么、panic 的消息，加第 1 次重跑的两列：交回值的 sha256（没有交回值写 `-`）、内存对象的合摘要
/// （各对象 `名字:sha256` 逗号接起来的 sha256）。不含耗时（两次基线逐字节相同，作废条款 Z1）。
fn call_outcomes_file_text(stream: &PreparedStream) -> String {
    stream
        .calls
        .iter()
        .enumerate()
        .map(|(call_index, call)| {
            let (model_answer, implementation_answer) = match &call.model {
                ModelVerdictOfCall::Disagrees {
                    model_answer,
                    implementation_answer,
                    ..
                } => (cell_text(model_answer), cell_text(implementation_answer)),
                ModelVerdictOfCall::NotModelled
                | ModelVerdictOfCall::Agrees
                | ModelVerdictOfCall::SkippedAfterTheModelStopped
                | ModelVerdictOfCall::NotComparedBecauseTheImplementationCrashed => {
                    ("-".to_string(), "-".to_string())
                }
                ModelVerdictOfCall::ModelCannotTakeTheObservation { what } => {
                    ("-".to_string(), cell_text(what))
                }
            };
            let panic_message = match &call.outcome {
                CallOutcome::Panicked(text) => cell_text(text),
                CallOutcome::Succeeded
                | CallOutcome::ReturnedError(_)
                | CallOutcome::InFlightAtTheTruncation => "-".to_string(),
            };
            format!(
                "{call_index}\t{}\t{}\t{}\t{}\t{}\t{model_answer}\t{implementation_answer}\t{panic_message}\t{}\t{}\n",
                call.label,
                call.first_operation,
                call.return_point,
                call.outcome.outcome_word(),
                call.model.word(),
                call.return_value
                    .as_ref()
                    .map_or_else(|| "-".to_string(), |value| value.whole.clone()),
                memory_objects_digest(&call.memory_objects),
            )
        })
        .collect()
}

fn memory_objects_text(memory_objects: &[(&'static str, String)]) -> String {
    if memory_objects.is_empty() {
        return "-".to_string();
    }
    memory_objects
        .iter()
        .map(|(name, digest)| format!("{name}:{digest}"))
        .collect::<Vec<String>>()
        .join(",")
}

fn memory_objects_digest(memory_objects: &[(&'static str, String)]) -> String {
    if memory_objects.is_empty() {
        return "-".to_string();
    }
    sha256_hexadecimal(memory_objects_text(memory_objects).as_bytes())
}

/// `call-return-values.tsv`（读法 4，登记 5.3 第 2 处）：每次调用一行——流名、调用序号、标签、交回值的 sha256、交回值顶层字段逐个
/// `名字:sha256`（逗号接起来）、内存对象逐个 `名字:sha256`、胶水交给模型的观测的 sha256。没有交回值（出错、panic、在途）的两格写 `-`，
/// 没交观测的写 `-`。
fn call_return_values_file_text(stream: &PreparedStream) -> String {
    stream
        .calls
        .iter()
        .enumerate()
        .map(|(call_index, call)| {
            let (whole, fields) = match &call.return_value {
                None => ("-".to_string(), "-".to_string()),
                Some(value) => (
                    value.whole.clone(),
                    value
                        .fields
                        .iter()
                        .map(|(name, digest)| format!("{name}:{digest}"))
                        .collect::<Vec<String>>()
                        .join(","),
                ),
            };
            format!(
                "{}\t{call_index}\t{}\t{whole}\t{fields}\t{}\t{}\n",
                stream.name.directory_name(),
                call.label,
                memory_objects_text(&call.memory_objects),
                call.observation_sha256.as_deref().unwrap_or("-")
            )
        })
        .collect()
}

/// 驱动怎么结束的，写成一个词（`stream-status.tsv` 与结果行共用）。
fn ending_text(ending: &StreamEnding) -> String {
    match ending {
        StreamEnding::Completed => "completed".to_string(),
        StreamEnding::StoppedByAnError { call_index, member } => {
            format!("stopped_by_an_error:call_{call_index}:{member}")
        }
        StreamEnding::StoppedByAPanic {
            call_index,
            message,
        } => format!(
            "stopped_by_a_panic:call_{call_index}:{}",
            field_text(&cell_text(message))
        ),
        StreamEnding::StoppedWithoutAnError { what } => {
            format!("stopped_without_an_error:{}", field_text(what))
        }
    }
}

/// `stream-status.tsv`：驱动怎么结束的、模型停在哪次调用、被判的根写在不在（`key<TAB>value`）。
fn stream_status_file_text(stream: &PreparedStream) -> String {
    format!(
        "ending\t{}\nmodel_stopped_at_call\t{}\njudged_root_index\t{}\nwrites\t{}\ncalls\t{}\n",
        ending_text(&stream.ending),
        stream
            .model_stopped_at_call
            .map_or_else(|| "none".to_string(), |call| call.to_string()),
        if stream.judged_root_index == usize::MAX {
            "none".to_string()
        } else {
            stream.judged_root_index.to_string()
        },
        stream.writes.len(),
        stream.calls.len()
    )
}

/// `oracle-verdicts.tsv`：每个取样状态一行（次序与 `states.tsv` 相同）——状态号、批号、崩溃状态序号、段号、触达的最后一次调用、
/// O1-模型崩溃判官（`green` / `red` / `na` / `off` / `recovery_panic`）与红的内容、O2（同）与判红的不变量、O2 直接调与
/// `judge_crash_image` 内那一遍是否一致、看 journal 那一遍恢复的摘要（根、结局、读回 sha256、施加记录数）、不看 journal 那一遍的摘要、
/// 崩溃点重放的 `red_items`（附带），加第 1 次重跑的五列：O1-RefFS 的票（`green` / `red` / `abstained` / `recovery_panic` / `off`）、
/// 弃权种类或红的理由、RefFS 的崩溃时刻、允许集合的下标、看 journal 那一遍恢复 panic 的消息（没有写 `-`），加修订 5 第 5.1 节第 5 条的
/// `piece_panics`：回调链 panic 了的态上逐件重做时 panic 的件名，按 [`JudgingPiece`] 的次序逗号连起来（没有写 `-`）。
fn oracle_verdicts_file_text(observations: &[StateObservation]) -> String {
    observations
        .iter()
        .enumerate()
        .map(|(state_number, observation)| {
            let (o2_word, o2_items) = match &observation.pool_checker_red {
                None => ("off", "-".to_string()),
                Some(items) if items.is_empty() => ("green", "-".to_string()),
                Some(items) => ("red", items.join(",")),
            };
            let o1_detail = match &observation.crash_judge {
                CrashJudgeVerdict::Red { what } => cell_text(what),
                CrashJudgeVerdict::Green
                | CrashJudgeVerdict::NotApplicable
                | CrashJudgeVerdict::Off
                | CrashJudgeVerdict::RecoveryPanicked
                | CrashJudgeVerdict::JudgePanicked => "-".to_string(),
            };
            let (reference_filesystem_detail, allowed) = match &observation.reference_filesystem {
                ReferenceFileSystemVoteOfAState::Green { allowed } => ("-".to_string(), allowed.clone()),
                ReferenceFileSystemVoteOfAState::Red { reason, allowed } => {
                    (field_text(&cell_text(reason)), allowed.clone())
                }
                ReferenceFileSystemVoteOfAState::Abstained { kind } => {
                    (field_text(kind), "-".to_string())
                }
                ReferenceFileSystemVoteOfAState::RecoveryPanicked
                | ReferenceFileSystemVoteOfAState::JudgePanicked
                | ReferenceFileSystemVoteOfAState::Off => ("-".to_string(), "-".to_string()),
            };
            format!(
                "{state_number}\t{}\t{}\t{}\t{}\t{}\t{o1_detail}\t{o2_word}\t{o2_items}\t{}\t{}\t{}\t{}\t{}\t{reference_filesystem_detail}\t{}\t{allowed}\t{}\t{}\n",
                observation.batch,
                observation.ordinal,
                observation.segment_index,
                observation
                    .touched_call
                    .map_or_else(|| "-".to_string(), |call| call.to_string()),
                observation.crash_judge.word(),
                observation.pool_checker_matches_the_judgement,
                observation.consulted.text(),
                observation.ignored.text(),
                field_text(&list_text(&observation.verdict.red_items)),
                observation.reference_filesystem.word(),
                observation.reference_filesystem_crash_moment,
                observation
                    .recovery_panic
                    .as_deref()
                    .map_or_else(|| "-".to_string(), |message| field_text(&cell_text(message))),
                piece_panics_text(&observation.piece_panics),
            )
        })
        .collect()
}

/// 一态的 `piece_panics` 列：件名按 [`JudgingPiece`] 的次序逗号连起来，没有写 `-`。
fn piece_panics_text(piece_panics: &[JudgingPiece]) -> String {
    if piece_panics.is_empty() {
        "-".to_string()
    } else {
        piece_panics
            .iter()
            .map(|piece| piece.name())
            .collect::<Vec<&str>>()
            .join(",")
    }
}

/// `writable-mount-probe.tsv`（读法 8，登记 5.3 第 6 样）：每个探过的态一行——流名、批号、崩溃状态序号、状态号、(i) 判不出那一支、
/// (ii) 计数那一支（为真写是哪一句，假写 `false`）、c_见证、所选版本、挂载结局、拒的成员、单子上的理由、第二截判定。
fn writable_mount_probe_file_text(stream: StreamName, observations: &[StateObservation]) -> String {
    observations
        .iter()
        .enumerate()
        .filter_map(|(state_number, observation)| {
            observation.probe.as_ref().map(|probe| {
                format!(
                    "{}\t{}\t{}\t{state_number}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    stream.directory_name(),
                    observation.batch,
                    observation.ordinal,
                    probe
                        .criterion
                        .some_device_has_fewer_than_two_verified_slots,
                    probe
                        .criterion
                        .counter_sentence
                        .map_or_else(|| "false".to_string(), |sentence| format!("{sentence:?}")),
                    probe.criterion.witnessed_counter,
                    probe.selected_version,
                    probe.mount,
                    probe.refusal_member,
                    probe.refusal_on_the_d23_list,
                    probe.second_crop,
                )
            })
        })
        .collect()
}

const ORACLE_VERDICTS_FILE: &str = "oracle-verdicts.tsv";
const CALL_OUTCOMES_FILE: &str = "call-outcomes.tsv";
const CALL_RETURN_VALUES_FILE: &str = "call-return-values.tsv";
const STREAM_STATUS_FILE: &str = "stream-status.tsv";
const WRITABLE_MOUNT_PROBE_FILE: &str = "writable-mount-probe.tsv";

/// 按 E170 登记 5.3 写一条流的十个文件，manifest 最后写（它记前九个的 sha256）；之后再写本实验自己的几个：
/// `oracle-verdicts.tsv`、`writable-mount-probe.tsv`（开了探针才有）、`call-outcomes.tsv`、`call-return-values.tsv`、`stream-status.tsv`
/// （不进 manifest：manifest 与 E170 导出装置写的同一份逐字节可比，停机 S2）。
fn write_stream_files(
    directory: &Path,
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    observations: &[StateObservation],
    full_states: u64,
    quick_tier_states: u64,
) -> Vec<WrittenFile> {
    let base_sectors = base_sectors_file_bytes(stream);
    let (operations_text, payload) = operations_file_text_and_payload(stream);
    let calls_text = calls_file_text(stream);
    let mut torn_text = String::new();
    let mut torn_payload = Vec::new();
    let mut torn_rows: Vec<(usize, usize)> = tables
        .torn_images
        .iter()
        .map(|row| (row.write_index, row.torn_image_index))
        .collect();
    torn_rows.sort_unstable();
    for (write_index, torn_image_index) in torn_rows {
        let torn_bytes = tables.enumerated_writes[torn_image_index]
            .bytes()
            .expect("撕裂镜像是带字节的写");
        torn_text.push_str(&format!(
            "{write_index}\t{}\t{}\t{}\n",
            torn_payload.len(),
            torn_bytes.len(),
            sha256_hexadecimal(torn_bytes)
        ));
        torn_payload.extend_from_slice(torn_bytes);
    }
    let mut states_text = String::new();
    let mut verdicts_text = String::new();
    for (state_number, observation) in observations.iter().enumerate() {
        states_text.push_str(&format!(
            "{state_number}\t{}\t{}\t{}\t{}\t{}\n",
            observation.ordinal,
            observation.segment_index,
            observation.batch,
            observation.landing_vector,
            observation.image_digest
        ));
        let verdict = &observation.verdict;
        verdicts_text.push_str(&format!(
            "{state_number}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            verdict.consulted_root,
            verdict.consulted_outcome,
            verdict.ignored_root,
            verdict.ignored_outcome,
            list_text(&verdict.red_items),
            list_text(&verdict.pool_checker_red),
            verdict.root_without_record,
            verdict.claimed_state_missing_unit,
            observation.judge_nanoseconds
        ));
    }
    let quick_tier_text =
        format!("quick_tier_states\t{quick_tier_states}\nfull_states\t{full_states}\n");
    let mut files = vec![
        write_export_file(directory, BASE_SECTORS_FILE, &base_sectors),
        write_export_file(directory, OPERATIONS_FILE, operations_text.as_bytes()),
        write_export_file(directory, OPERATIONS_PAYLOAD_FILE, &payload),
        write_export_file(directory, CALLS_FILE, calls_text.as_bytes()),
        write_export_file(directory, TORN_FILE, torn_text.as_bytes()),
        write_export_file(directory, TORN_PAYLOAD_FILE, &torn_payload),
        write_export_file(directory, STATES_FILE, states_text.as_bytes()),
        write_export_file(
            directory,
            IMPLEMENTATION_VERDICTS_FILE,
            verdicts_text.as_bytes(),
        ),
        write_export_file(directory, QUICK_TIER_FILE, quick_tier_text.as_bytes()),
    ];
    let device_list: Vec<String> = stream
        .base
        .devices
        .keys()
        .map(|device| device.0.to_string())
        .collect();
    let mut manifest_text = format!(
        "stream\t{}\ndevices\t{}\ndevice_bytes\t{}\nsector_bytes\t{LOCAL_SECTOR_BYTES}\nops\t{}\nwrites\t{}\ncalls\t{}\nsampled_states\t{}\n",
        stream.name.directory_name(),
        device_list.join(","),
        stream.base.device_size_in_bytes,
        stream.operations.len(),
        stream.writes.len(),
        stream.calls.len(),
        observations.len(),
    );
    for (listed_name, file) in FILES_LISTED_IN_THE_MANIFEST.iter().zip(&files) {
        assert_eq!(*listed_name, file.name, "manifest 的次序与写出的次序相同");
        manifest_text.push_str(&format!("sha256:{}\t{}\n", file.name, file.sha256));
    }
    files.push(write_export_file(
        directory,
        MANIFEST_FILE,
        manifest_text.as_bytes(),
    ));
    files.push(write_export_file(
        directory,
        ORACLE_VERDICTS_FILE,
        oracle_verdicts_file_text(observations).as_bytes(),
    ));
    if observations
        .iter()
        .any(|observation| observation.probe.is_some())
    {
        files.push(write_export_file(
            directory,
            WRITABLE_MOUNT_PROBE_FILE,
            writable_mount_probe_file_text(stream.name, observations).as_bytes(),
        ));
    }
    files
        .extend(write_the_files_that_exist_even_when_the_stream_is_not_exported(directory, stream));
    files
}

/// 没有根槽写可判、或流水线里别处 panic 的流不导出状态；这三个文件照写（比对器靠它们认出「出错截断」「panic 截断」与交回值）。
fn write_the_files_that_exist_even_when_the_stream_is_not_exported(
    directory: &Path,
    stream: &PreparedStream,
) -> Vec<WrittenFile> {
    vec![
        write_export_file(
            directory,
            CALL_OUTCOMES_FILE,
            call_outcomes_file_text(stream).as_bytes(),
        ),
        write_export_file(
            directory,
            CALL_RETURN_VALUES_FILE,
            call_return_values_file_text(stream).as_bytes(),
        ),
        write_export_file(
            directory,
            STREAM_STATUS_FILE,
            stream_status_file_text(stream).as_bytes(),
        ),
    ]
}

/// 一条流导出之后的样子。`not_exported` 有值：这条流没有导出状态（没有根槽写可判、或流水线里别处 panic），
/// 只有 `call-outcomes.tsv`、`call-return-values.tsv` 与 `stream-status.tsv`（能写出来的话）。
struct StreamExport {
    name: StreamName,
    checks: Vec<StopCheck>,
    calls: Vec<RecordedCall>,
    ending: Option<StreamEnding>,
    model_stopped_at_call: Option<usize>,
    not_exported: Option<String>,
    operation_count: usize,
    write_count: usize,
    segment_count: usize,
    tearable_write_count: usize,
    full_states: u64,
    quick_tier_states: u64,
    torn_scans: Vec<TornScan>,
    observations: Vec<StateObservation>,
    files: Vec<WrittenFile>,
    rebuilt_digest_mismatches: usize,
    rebuilt_states: usize,
    /// RefFS 一侧切的段数（`segment_closing_stream_indexes`，A9 与装置切的段数比）与翻译停在哪一步（没停写 `-`）。
    reference_filesystem_segments: usize,
    reference_filesystem_stopped: String,
    elapsed_nanoseconds: u128,
    /// 驱动这条流（含每次调用与推模型、O1-对拍）、判一批状态、写出导出文件各花多久（装置进程里 `Instant` 计）。
    drive_nanoseconds: u128,
    observe_nanoseconds: u128,
    write_nanoseconds: u128,
}

impl StreamExport {
    /// 什么都没导出的一条流：流水线在驱动之后、导出之前 panic 时用它占位。
    fn not_exported(name: StreamName, why: String, started: Instant) -> Self {
        Self {
            name,
            checks: Vec::new(),
            calls: Vec::new(),
            ending: None,
            model_stopped_at_call: None,
            not_exported: Some(why),
            operation_count: 0,
            write_count: 0,
            segment_count: 0,
            tearable_write_count: 0,
            full_states: 0,
            quick_tier_states: 0,
            torn_scans: Vec::new(),
            observations: Vec::new(),
            files: Vec::new(),
            rebuilt_digest_mismatches: 0,
            rebuilt_states: 0,
            reference_filesystem_segments: 0,
            reference_filesystem_stopped: "-".to_string(),
            elapsed_nanoseconds: started.elapsed().as_nanos(),
            drive_nanoseconds: 0,
            observe_nanoseconds: 0,
            write_nanoseconds: 0,
        }
    }
}

/// 基线与坏实现那一趟的差别：基线回比本地常量、取三批；坏实现那一趟不回比（写流与段表本来就会变）、只取批 1 与批 2。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunKind {
    Baseline,
    BadImplementationRow,
}

/// 判哪些态：基线与逐行那一趟照登记 5.4 取批（逐行那一趟不取批 3）；Q5 那一形取末段整段全枚举再加最后那个全持久态（读法 7），都记在批 1；
/// Q5 主取样点按等价类判（登记修订 2 第 2.1 节候选二、修订 3）只判点名的那几态（各类的代表与「C 全落」那一态），记在批 1、留整份恢复报告。
#[derive(Clone, Debug, PartialEq, Eq)]
enum StateSelection {
    RegisteredBatches { third_batch: bool },
    WholeLastSegmentAndTheFullyPersistedState,
    ListedStatesKeepingTheConsultedReports(BTreeSet<u64>),
}

fn export_stream(
    name: StreamName,
    output_root: &Path,
    run_kind: RunKind,
    switches: RunSwitches,
) -> StreamExport {
    let started = Instant::now();
    let is_baseline = run_kind == RunKind::Baseline;
    let stream = prepare_stream(name, switches, is_baseline);
    let drive_nanoseconds = started.elapsed().as_nanos();
    let directory = output_root.join(name.directory_name());
    let selection = StateSelection::RegisteredBatches {
        third_batch: is_baseline,
    };
    export_prepared_stream(
        &stream,
        &directory,
        is_baseline,
        switches,
        selection,
        started,
        drive_nanoseconds,
        None,
    )
}

/// 一条驱动完的流导出：枚举、取样、判、写导出文件、S4 重建核对。`check_against_the_crates` 只在基线那一趟开（S1 的状态数那几条）。
/// `piece_panic_injection` 交给判态的 [`ObservationContext`]：只有 Q5 主取样点、反向取样点那一次导出可能不空（登记修订 6 第 6.2 节第 3 条，
/// 跑法里也是空，只有内联用例 U15、U16 交），别的调用处一律交空。
#[allow(
    clippy::too_many_arguments,
    reason = "八个参数各有名字、各有类型，凑不成一个带不变量的概念（code-discipline.md「参数不按个数限」）"
)]
fn export_prepared_stream(
    stream: &PreparedStream,
    directory: &Path,
    check_against_the_crates: bool,
    switches: RunSwitches,
    selection: StateSelection,
    started: Instant,
    drive_nanoseconds: u128,
    piece_panic_injection: Option<&PiecePanicInjection>,
) -> StreamExport {
    std::fs::create_dir_all(directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
    let name = stream.name;
    // 回比只在基线那一趟：坏实现那一趟本来就会让取号、txg、落点、写数与段表变，那些不是停机条款。
    let mut checks = if check_against_the_crates {
        stream.checks.clone()
    } else {
        Vec::new()
    };
    let mut export = StreamExport::not_exported(name, String::new(), started);
    export.not_exported = None;
    export.calls = stream.calls.clone();
    export.ending = Some(stream.ending.clone());
    export.model_stopped_at_call = stream.model_stopped_at_call;
    export.operation_count = stream.operations.len();
    export.write_count = stream.writes.len();
    export.segment_count = stream.segments.len();
    export.drive_nanoseconds = drive_nanoseconds;
    if stream.judged_root_index == usize::MAX {
        export.files =
            write_the_files_that_exist_even_when_the_stream_is_not_exported(directory, stream);
        export.checks = checks;
        export.not_exported = Some("no_root_write_to_judge".to_string());
        export.elapsed_nanoseconds = started.elapsed().as_nanos();
        return export;
    }
    let tables = crash_state_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let full_states = crash_point_replay_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let quick_tier_states = crash_point_replay_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &quick_tier_expansion,
    );
    let facts = StateMasks::of(&tables, &stream.segments);
    if check_against_the_crates {
        checks.extend(state_count_checks_against_the_crates(
            name,
            stream,
            &tables,
            flow_facts_state_count(stream, &tables),
            full_states,
            quick_tier_states,
        ));
    }
    let torn_image_index_by_write: BTreeMap<usize, usize> = tables
        .torn_images
        .iter()
        .map(|row| (row.write_index, row.torn_image_index))
        .collect();
    let segment_has_a_torn_write: Vec<bool> = stream
        .segments
        .iter()
        .map(|segment| {
            segment
                .iter()
                .any(|write_index| tables.is_tearable_by_write[*write_index])
        })
        .collect();
    let recorded_write_count = stream.writes.len();
    let sampled = match &selection {
        StateSelection::RegisteredBatches { third_batch } => {
            let mut sampled = sample_ordinals(
                &tables.state_ranges_by_segment,
                tables.state_count,
                &segment_has_a_torn_write,
                &|ordinal| {
                    landing_vector_of(
                        &facts.persisted_writes_of_state(ordinal),
                        recorded_write_count,
                        &torn_image_index_by_write,
                    )
                },
            );
            if !*third_batch {
                sampled.batches[2].clear();
            }
            sampled
        }
        StateSelection::WholeLastSegmentAndTheFullyPersistedState => {
            let last_segment = tables
                .state_ranges_by_segment
                .last()
                .cloned()
                .unwrap_or(0..0);
            let mut whole: BTreeSet<u64> = last_segment.collect();
            whole.insert(tables.state_count - 1);
            SampledOrdinals {
                batches: [whole, BTreeSet::new(), BTreeSet::new()],
                torn_scans: Vec::new(),
            }
        }
        StateSelection::ListedStatesKeepingTheConsultedReports(listed) => SampledOrdinals {
            batches: [listed.clone(), BTreeSet::new(), BTreeSet::new()],
            torn_scans: Vec::new(),
        },
    };
    let consulted_reports = match &selection {
        StateSelection::RegisteredBatches { .. }
        | StateSelection::WholeLastSegmentAndTheFullyPersistedState => ConsultedReports::Dropped,
        StateSelection::ListedStatesKeepingTheConsultedReports(_) => ConsultedReports::Kept,
    };
    let reference_filesystem = reference_filesystem_stream_of(
        &stream.calls,
        &stream.operations,
        &stream.writes,
        &stream.stream_indexes,
    );
    export.reference_filesystem_segments =
        reference_filesystem.segment_closing_stream_indexes.len();
    export.reference_filesystem_stopped = reference_filesystem.stopped.as_ref().map_or_else(
        || "-".to_string(),
        |(position, stop)| {
            format!(
                "{}:{}",
                crash_moment_text(ReferenceFileSystemCrashMoment::InsideStep(*position)),
                field_text(stop.name())
            )
        },
    );
    let context = ObservationContext {
        stream,
        tables: &tables,
        facts: &facts,
        torn_image_index_by_write: &torn_image_index_by_write,
        reference_filesystem: &reference_filesystem,
        call_of_write: call_index_of_each_write(stream),
        piece_panic_injection,
    };
    let observe_started = Instant::now();
    let observations = observe_sampled_states(&context, &sampled, switches, consulted_reports);
    let observe_nanoseconds = observe_started.elapsed().as_nanos();
    let write_started = Instant::now();
    let files = write_stream_files(
        directory,
        stream,
        &tables,
        &observations,
        full_states,
        quick_tier_states,
    );
    // S4：只读刚写出的文件，按 states.tsv 的落地向量重建，摘要与 states.tsv 里（直接从 CrashImage 算的）那一列比。
    let neutral = read_neutral_stream(directory);
    let mut rebuilt_states = 0usize;
    let mut rebuilt_digest_mismatches = 0usize;
    for line in read_text_lines(directory, STATES_FILE) {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 6, "states.tsv 一行六列：{line:?}");
        rebuilt_states += 1;
        if sha256_text(&rebuilt_image_digest(&neutral, fields[4])) != fields[5] {
            rebuilt_digest_mismatches += 1;
        }
    }
    export.write_nanoseconds = write_started.elapsed().as_nanos();
    export.observe_nanoseconds = observe_nanoseconds;
    export.checks = checks;
    export.tearable_write_count = torn_image_index_by_write.len();
    export.full_states = full_states;
    export.quick_tier_states = quick_tier_states;
    export.torn_scans = sampled.torn_scans;
    export.observations = observations;
    export.files = files;
    export.rebuilt_digest_mismatches = rebuilt_digest_mismatches;
    export.rebuilt_states = rebuilt_states;
    export.elapsed_nanoseconds = started.elapsed().as_nanos();
    export
}

/// S1 里跟状态数有关的几条（只在基线那一趟）：全量与快档状态数、每次写只取两态时的闭式、枚举表与 `crash_facts` 数的总数，
/// 对本地常量。与 `e170_o3_spec_executor_export.rs` 的 `export_stream` 里那一批 S1 回比同一批。
fn state_count_checks_against_the_crates(
    name: StreamName,
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    flow_facts_state_count: Option<u64>,
    full_states: u64,
    quick_tier_states: u64,
) -> Vec<StopCheck> {
    let label = name.directory_name();
    let (local_full, local_quick, local_closed_form) = match name {
        StreamName::NewPoolFileCreation => (
            LOCAL_FIRST_STREAM_FULL_STATES,
            LOCAL_FIRST_STREAM_QUICK_TIER_STATES,
            LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM,
        ),
        StreamName::FixedScriptThroughEThenUnmount => (
            LOCAL_SECOND_STREAM_FULL_STATES,
            LOCAL_SECOND_STREAM_QUICK_TIER_STATES,
            LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM,
        ),
        StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf => (
            LOCAL_THIRD_STREAM_FULL_STATES,
            LOCAL_THIRD_STREAM_QUICK_TIER_STATES,
            LOCAL_THIRD_STREAM_TWO_STATE_CLOSED_FORM,
        ),
    };
    vec![
        StopCheck::new(&format!("{label}_full_states"), local_full, full_states),
        StopCheck::new(
            &format!("{label}_quick_tier_states"),
            local_quick,
            quick_tier_states,
        ),
        StopCheck::new(
            &format!("{label}_two_state_closed_form"),
            local_closed_form,
            singlefs_harness::memory_pool::closed_form_state_count(&stream.segments),
        ),
        StopCheck::new(
            &format!("{label}_enumeration_tables_state_count"),
            full_states,
            tables.state_count,
        ),
        match flow_facts_state_count {
            Some(state_count) => StopCheck::new(
                &format!("{label}_flow_facts_state_count"),
                full_states,
                state_count,
            ),
            // 枚举写表多于 GPU 掩码宽：`extract_flow_facts` 抽不了（它断言 ≤ 2048），这一条换成「确实多于」的回比。
            None => StopCheck::new(
                &format!("{label}_flow_facts_not_extracted_because_the_enumerated_writes_exceed_the_mask_width"),
                true,
                tables.enumerated_writes.len() > MAXIMUM_ENUMERATED_WRITES,
            ),
        },
    ]
}

// ───────────────────────────── 结果行、V2 与 main ─────────────────────────────

/// `E7RESULT` 行计数（收尾 `name=done emitted=N`，N 把 done 自己也算进去）；`E7TIMING` 行不计数、不进入库产物。
struct ResultLines {
    emitted: u64,
    /// 收集模式（只在内联用例里用，修订 5 第 5.7 节 U8）：结果行收进这里、不打到标准输出；None 时照常打。
    collected: Option<Vec<String>>,
}

impl ResultLines {
    fn emit_result_line(&mut self, body: &str) {
        self.emitted += 1;
        match &mut self.collected {
            Some(collected) => collected.push(body.to_string()),
            None => println!("E7RESULT {body}"),
        }
    }
    fn timing(body: &str) {
        println!("E7TIMING {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        let body = format!("name=done emitted={}", self.emitted);
        match &mut self.collected {
            Some(collected) => collected.push(body),
            None => println!("E7RESULT {body}"),
        }
    }
}

/// 结果行里的值不许带空白（按空格切字段）。
fn field_text(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

/// S1 里与流无关的几条：本地常量对 `tests/common/mod.rs` 与 harness 的同名公开常量或现算值。
fn stream_independent_checks() -> Vec<StopCheck> {
    let parameters = common::parameters();
    vec![
        StopCheck::new("device_bytes", LOCAL_DEVICE_BYTES, common::IMAGE_BYTES),
        StopCheck::new(
            "physical_block_bytes",
            LOCAL_PHYSICAL_BLOCK_BYTES,
            parameters.geometry.physical_block_size,
        ),
        StopCheck::new(
            "minimum_input_output_bytes",
            LOCAL_MINIMUM_INPUT_OUTPUT_BYTES,
            parameters.geometry.minimum_input_output_bytes,
        ),
        StopCheck::new(
            "sector_bytes",
            LOCAL_SECTOR_BYTES,
            singlefs_harness::memory_pool::SECTOR_BYTES,
        ),
        StopCheck::new(
            "filesystem_identifier",
            LOCAL_FILESYSTEM_IDENTIFIER,
            common::E142_FILESYSTEM_IDENTIFIER,
        ),
        StopCheck::new(
            "fixed_write_time_seconds",
            LOCAL_FIXED_WRITE_TIME_SECONDS,
            common::FIXED_WRITE_TIME_SECONDS,
        ),
        StopCheck::new(
            "first_file_bytes",
            LOCAL_FIRST_FILE_BYTES,
            common::FILE_BYTES,
        ),
        StopCheck::new(
            "first_file_content",
            true,
            local_first_file_content() == common::file_content(),
        ),
    ]
}

/// 中位取排好序之后下标 ⌊(n − 1)/2⌋ 那一个，p95 取下标 ⌈0.95·n⌉ − 1 那一个。
fn timing_summary(mut nanoseconds: Vec<u128>) -> (u128, u128, u128, u128) {
    if nanoseconds.is_empty() {
        return (0, 0, 0, 0);
    }
    nanoseconds.sort_unstable();
    let count = nanoseconds.len();
    let total: u128 = nanoseconds.iter().sum();
    let p95_index = (count * 95).div_ceil(100) - 1;
    (
        total,
        nanoseconds[(count - 1) / 2],
        nanoseconds[p95_index],
        nanoseconds[count - 1],
    )
}

// ───────────────────────────── 已知坏镜像族（登记 5.5 第三族）：照抄 `checker_known_bad_images.rs` ─────────────────────────────
//
// 那份用例是 O2 自己的回归语料：每一份都照 O2 的判据造来让 O2 红。`known_bad_images` 与它新用到的辅助函数
// `move_the_data_unit_of_the_first_file_two_slots_further` 照抄 sha256 e426cdff6c0cd4feebb8d493940c077243017a4d493523e719d7ae68dbf35736
// 那一份（第 755–1326 行、第 1328–1375 行；E170 第 2 次重跑登记修订 2 读法 18）。其余导入、常量与辅助函数照抄自更早的
// sha256 e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820 那一份，与 e426cdff… 里同名条目去空白、去收尾逗号之后逐个相同
// （`use` 两行不比）；散在 `known_bad_images` 之后的那几个在 e426cdff… 里的行号写在模块末尾的注释里。它的 `#[test]` 没有抄。照抄、不 `#[path]` 引：
// 那个文件有自己的 `mod common;` 与全部测试，私有的 `known_bad_images` 也引不到。条目名与条数在 `known-bad-images` 模式里与原文件逐条比（停机 S4、断言 A8）。
#[allow(
    dead_code,
    unused_imports,
    clippy::all,
    clippy::pedantic,
    reason = "照抄 checker_known_bad_images.rs 的辅助函数与坏镜像表，删了没人调的 verdict：那份文件里别的族用的辅助函数在这里没人调，抄来的写法也不照本文件的 lint 改"
)]
mod known_bad_images_copied_from_the_harness_test {
    use super::common::{
        abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool,
        copy_each_root_into_the_new_root_segment_of_its_journal_records, crash_state_devices,
        format_pool, geometry, memory_pool_of_sparse_devices, parameters,
        publish_overwrite_in_process, rewrite_journal_record_and_rechain_the_records_after_it,
        BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES,
    };
    use singlefs_checker::image::{
        InvariantVerdict, ACCOUNTING_ROWS_CARRY_THE_GENERATION_OF_THEIR_VERSION,
        DEVICE_COUNTS_OF_THE_CHOSEN_SYSTEM_CONFIGURATIONS_AGREE,
        FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
        INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
        INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
        JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
        NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
        PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES, REFERENCE_INTERVALS_ARE_EXACT,
        ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES, ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
        SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
        SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
        SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
        SYSTEM_CONFIGURATION_SLOT_GENERATIONS_FOLLOW_THE_ROTATION,
        TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
        UNIT_BIRTH_NOT_LATER_THAN_A_CANDIDATE_ROOT_REFERENCING_IT,
    };
    use singlefs_checker::walk::check_pool_image;
    use singlefs_core::address::{
        CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    };
    use singlefs_core::allocation_record_tree::AllocationRecordTreeNodePosition;
    use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
    use singlefs_core::block_device::{BlockDevice, PhysicalBlockSizeInBytes};
    use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
    use singlefs_core::journal::{
        back_chain_of, record_offset, JournalRecord, JournalRecordOrdinalWithinPublish,
        JournalRecordPlaceInPublish,
    };
    use singlefs_core::make_filesystem::{
        allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
        INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
    };
    use singlefs_core::mount::{
        mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, InstanceRow,
        MountError, RollbackTarget, ShadowLedger, Unmounted,
    };
    use singlefs_core::recovery::{
        instance_table_chain_of_root, recover, JournalPolicy, PoolReader, RecoveryFailure,
        RecoveryOutcome,
    };
    use singlefs_core::system_configuration::SystemImmutableSizes;
    use singlefs_core::transaction::{
        acquire_instance, publish_first_file, publish_instance_table_on_version_without_file,
        publish_overwrite, publish_sequential_write, publish_without_units, warm_up, FirstFile,
        InstanceTableOnlyPublishPlan, InstanceTableRewrite, PoolVersion, PoolWriter,
        TransactionOutput, TransactionUnit, ZeroUnitPublishPlan,
    };
    use singlefs_format::{
        ALLOCATION_RECORD_TREE_LEAF_SLOTS, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES,
        JOURNAL_SAFETY_FACTOR, NODE_BYTES, TREE_IDENTIFIER_EXTENT,
    };
    use singlefs_harness::history::{allocation_statistic_mechanism, AllocationStatisticMechanism};
    use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
    use singlefs_harness::segments::StepKind;
    use singlefs_harness::{RetainedOperation, SharedStream};

    const SLOT: u64 = 16384;
    const DEVICES: [u32; 2] = [0, 1];
    /// 写完新池新建文件之后单元区里的单元：(起点槽, 字节数)。分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，
    /// A 写出它的五个节点——两块盘各自的叶 61（50245、50246）、两块盘各自的第 1 层节点 0（50247、50248）与根（50249），先叶后根。
    const UNITS: [(u64, usize); 14] = [
        (50176, 32768),
        (50178, 16384),
        (50180, 32768),
        (50240, 16384),
        (50242, 32768),
        (50244, 16384),
        (50245, 16384),
        (50246, 16384),
        (50247, 16384),
        (50248, 16384),
        (50249, 16384),
        (50250, 16384),
        (50251, 16384),
        (50252, 16384),
    ];
    /// 再覆盖写一次（发布 B，txg 4）之后单元区里的单元：A 的十四个槽仍在盘上（根环里 A 的根还引用着它们），加上 B 写出的十二个单元
    /// （落点由 `overwrite_in_one_instance.rs` 的验收钉住）。
    const UNITS_AFTER_OVERWRITE: [(u64, usize); 26] = [
        (50176, 32768),
        (50178, 16384),
        (50180, 32768),
        (50182, 32768),
        (50240, 16384),
        (50242, 32768),
        (50244, 16384),
        (50245, 16384),
        (50246, 16384),
        (50247, 16384),
        (50248, 16384),
        (50249, 16384),
        (50250, 16384),
        (50251, 16384),
        (50252, 16384),
        (50253, 16384),
        (50254, 32768),
        (50256, 16384),
        (50257, 16384),
        (50258, 16384),
        (50259, 16384),
        (50260, 16384),
        (50261, 16384),
        (50262, 16384),
        (50263, 16384),
        (50264, 16384),
    ];
    /// 发布 B 写出的分配记录树两片叶（盘 0 的叶 61、盘 1 的叶 61；I-3.9、I-3.10、I-5.4 那几份坏镜像改的就是它们）与树表（I-9.14 那一份改它）。
    /// 每片叶只装它那块盘上的记录：一个单元两盘各一份落点，两条记录分在两片叶里。
    const ALLOCATION_LEAVES_AFTER_OVERWRITE: [u64; 2] = [50257, 50258];
    const TREE_TABLE_AFTER_OVERWRITE: u64 = 50264;
    const INSTANCE_TABLE: u64 = 50176;
    /// mkfs 种下的第 0 版树表单元：第一个文件版本换下它之后只有第 0 代根还引用。
    const GENESIS_TREE_TABLE: u64 = 50178;
    const DATA_UNIT: u64 = 50180;
    const EXTENT_ROOT: u64 = 50240;
    const INODE_LEAF: u64 = 50242;
    const INODE_ROOT: u64 = 50244;
    /// 新池新建文件（A）写出的分配记录树两片叶（盘 0、盘 1 各自的叶 61）：发布 B 之后只有 A 的根还（经 A 的第 1 层节点）指着它们。
    const ALLOCATION_LEAVES: [u64; 2] = [50245, 50246];
    const ACCOUNTING_ROOT: u64 = 50250;
    const TREE_TABLE: u64 = 50252;

    fn read(pool: &MemoryPool, device: u32, offset: u64, length: usize) -> Vec<u8> {
        PoolReader::read(
            pool,
            DeviceIdentity(device),
            DeviceOffsetInBytes(offset),
            length,
        )
        .expect("读")
    }

    fn write(pool: &mut MemoryPool, device: u32, offset: u64, bytes: &[u8]) {
        pool.devices
            .get_mut(&DeviceIdentity(device))
            .expect("盘")
            .write(DeviceOffsetInBytes(offset), bytes);
    }

    /// 按类重封：先算载荷 CRC，再算头校验和（码 1：头 105、CRC 在 101；码 3：头 107、CRC 在 89；码 2：头 86 + 2k、CRC 在 76 + 2k）。
    fn reseal_unit(bytes: &mut [u8]) {
        let (header_end, payload_crc_offset) = match bytes[6] {
            1 => (105usize, 101usize),
            3 => (107, 89),
            _ => (
                86 + 2 * usize::from(bytes[51]),
                76 + 2 * usize::from(bytes[51]),
            ),
        };
        let payload_crc = crc32_castagnoli(&bytes[header_end..]);
        bytes[payload_crc_offset..payload_crc_offset + 4]
            .copy_from_slice(&payload_crc.to_le_bytes());
        singlefs_core::unit::seal_header_checksum(bytes, header_end);
    }

    /// 根环全部槽：(盘, 偏移)。区域 r 在盘 [0, 1, 0][r] 上，起点 64 × 16384 + r × 3 MiB，槽距 4096。
    fn root_slots() -> Vec<(u32, u64)> {
        (0..3u64)
            .flat_map(|region| {
                (0..8u64).map(move |slot| {
                    (
                        [0u32, 1, 0][usize::try_from(region).expect("区域")],
                        64 * SLOT + region * 3 * (1 << 20) + slot * 4096,
                    )
                })
            })
            .collect()
    }

    fn location_pattern(device: u32, slot: u64, checksum: u32) -> Vec<u8> {
        [
            device.to_le_bytes().to_vec(),
            slot.to_le_bytes()[..6].to_vec(),
            checksum.to_le_bytes().to_vec(),
        ]
        .concat()
    }

    fn replace_all(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> bool {
        let mut changed = false;
        let mut index = 0;
        while index + needle.len() <= haystack.len() {
            if haystack[index..index + needle.len()] == *needle {
                haystack[index..index + needle.len()].copy_from_slice(replacement);
                changed = true;
                index += needle.len();
            } else {
                index += 1;
            }
        }
        changed
    }

    /// 一个单元（两盘同槽）的整单元校验和从 old 变成 new：把引用它的位置条目逐个补上，被补的单元重封、再往上补，直到根槽。
    /// `units` 是这份镜像单元区里的单元表：写完新池新建文件的是 `UNITS`，再覆盖写一次的是 `UNITS_AFTER_OVERWRITE`。
    fn propagate(pool: &mut MemoryPool, units: &[(u64, usize)], mut changes: Vec<(u64, u32, u32)>) {
        while let Some((slot, old, new)) = changes.pop() {
            if old == new {
                continue;
            }
            for (container, length) in units.iter().copied() {
                if container == slot {
                    continue;
                }
                let mut bytes = read(pool, 0, container * SLOT, length);
                let mut touched = false;
                for device in DEVICES {
                    touched |= replace_all(
                        &mut bytes,
                        &location_pattern(device, slot, old),
                        &location_pattern(device, slot, new),
                    );
                }
                if touched {
                    let before = crc32_castagnoli(&read(pool, 0, container * SLOT, length));
                    reseal_unit(&mut bytes);
                    for device in DEVICES {
                        write(pool, device, container * SLOT, &bytes);
                    }
                    changes.push((container, before, crc32_castagnoli(&bytes)));
                }
            }
            for (device, offset) in root_slots() {
                let mut bytes = read(pool, device, offset, 512);
                let mut touched = false;
                for location_device in DEVICES {
                    touched |= replace_all(
                        &mut bytes,
                        &location_pattern(location_device, slot, old),
                        &location_pattern(location_device, slot, new),
                    );
                }
                if touched {
                    let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
                    bytes[138..170].copy_from_slice(&digest);
                    write(pool, device, offset, &bytes);
                }
            }
        }
        copy_each_root_into_the_new_root_segment_of_its_journal_records(pool);
    }

    /// journal 记录头里 `checkpoint_txg`（8 字节）的偏移（D23（journal 的角色与格式） 已定项 4 / 已定项 17 的字段表）。
    const JOURNAL_RECORD_CHECKPOINT_TXG_OFFSET: usize = 26;

    /// 只改一块盘上的那一份单元（另一块盘留着原样）：重封被改那一份的两道校验和，再把它的新整单元校验和补进引用它的
    /// **那一块盘那一条**位置条目——位置条目逐盘各带一个整单元校验和（D4（校验和位置），指针 86 / 88 里位置条目 14 × 2），
    /// 所以补完两份各自仍与自己那条位置条目对得上，I-2.1（校验和与内容匹配） 不跟着红；父单元两盘同内容，它自己的整单元校验和
    /// 往上补那一段照旧走 `propagate`。
    fn mutate_one_device_copy_of_unit(
        pool: &mut MemoryPool,
        units: &[(u64, usize)],
        device: u32,
        slot: u64,
        change: impl Fn(&mut Vec<u8>),
    ) {
        let length = units
            .iter()
            .find(|(start, _)| *start == slot)
            .expect("登记过的单元")
            .1;
        let before = read(pool, device, slot * SLOT, length);
        let mut bytes = before.clone();
        change(&mut bytes);
        reseal_unit(&mut bytes);
        write(pool, device, slot * SLOT, &bytes);
        let (old, new) = (crc32_castagnoli(&before), crc32_castagnoli(&bytes));
        let mut parents_changed = Vec::new();
        for (container, container_length) in units.iter().copied() {
            if container == slot {
                continue;
            }
            let container_before = read(pool, 0, container * SLOT, container_length);
            let mut container_bytes = container_before.clone();
            if replace_all(
                &mut container_bytes,
                &location_pattern(device, slot, old),
                &location_pattern(device, slot, new),
            ) {
                reseal_unit(&mut container_bytes);
                for each_device in DEVICES {
                    write(pool, each_device, container * SLOT, &container_bytes);
                }
                parents_changed.push((
                    container,
                    crc32_castagnoli(&container_before),
                    crc32_castagnoli(&container_bytes),
                ));
            }
        }
        for (root_device, offset) in root_slots() {
            let mut root_bytes = read(pool, root_device, offset, 512);
            if replace_all(
                &mut root_bytes,
                &location_pattern(device, slot, old),
                &location_pattern(device, slot, new),
            ) {
                let digest = wide_checksum_with_field_zeroed(&root_bytes, 512, 138);
                root_bytes[138..170].copy_from_slice(&digest);
                write(pool, root_device, offset, &root_bytes);
            }
        }
        propagate(pool, units, parents_changed);
    }

    /// 改一个单元（两盘一起改）：`reseal` 为真时重封它的两道校验和；新的整单元校验和沿引用链补上去。
    fn mutate_unit_of(
        pool: &mut MemoryPool,
        units: &[(u64, usize)],
        slot: u64,
        change: impl Fn(&mut Vec<u8>),
        reseal: bool,
    ) {
        let length = units
            .iter()
            .find(|(start, _)| *start == slot)
            .expect("登记过的单元")
            .1;
        let before = read(pool, 0, slot * SLOT, length);
        let mut bytes = before.clone();
        change(&mut bytes);
        if reseal {
            reseal_unit(&mut bytes);
        }
        for device in DEVICES {
            write(pool, device, slot * SLOT, &bytes);
        }
        propagate(
            pool,
            units,
            vec![(slot, crc32_castagnoli(&before), crc32_castagnoli(&bytes))],
        );
    }

    /// 写完新池新建文件那份镜像上改一个单元。
    fn mutate_unit(pool: &mut MemoryPool, slot: u64, change: impl Fn(&mut Vec<u8>), reseal: bool) {
        mutate_unit_of(pool, &UNITS, slot, change, reseal);
    }

    /// 分配记录树的两片叶（两块盘各一片）逐片改、各自重封，新的整单元校验和沿引用链补上去（叶 → 那块盘的第 1 层节点 → 根 → 树表 → 根槽）；
    /// `change` 返回它在那一片叶里改了几条记录，这里交回两片合起来的条数。
    fn mutate_allocation_leaves(
        pool: &mut MemoryPool,
        units: &[(u64, usize)],
        leaves: [u64; 2],
        change: impl Fn(&mut Vec<u8>) -> usize,
    ) -> usize {
        let changed_records = std::cell::Cell::new(0usize);
        for leaf in leaves {
            mutate_unit_of(
                pool,
                units,
                leaf,
                |bytes| changed_records.set(changed_records.get() + change(bytes)),
                true,
            );
        }
        changed_records.get()
    }

    /// journal 记录头里这三个字段的偏移（D23（journal 的角色与格式） 已定项 17 的字段表；记录头自证校验和在 46、罩整条 4096）。
    const JOURNAL_RECORD_INSTANCE_OFFSET: usize = 16;
    const JOURNAL_RECORD_TRANSACTION_OFFSET: usize = 78;

    /// 这几份镜像上环里的记录条数上界：读到这个计数器还没读完就说明镜像变了，断言会响（别让列表被悄悄截断）。
    const RECORDS_READ_BACK_AT_MOST: u64 = 64;

    /// 一块盘的 journal 环里从计数器 1 起连着的记录，读成 (计数器, 实例代号, 事务号)；读到没有 magic 的那一格就停。
    /// 计数器与环槽一一对应（`record_offset`），所以这里按计数器取偏移，不整条环扫。
    fn journal_records_on_device(pool: &MemoryPool, device: u32) -> Vec<(u64, u32, u64)> {
        let mut records = Vec::new();
        for counter in 1..=RECORDS_READ_BACK_AT_MOST {
            let offset = singlefs_core::journal::record_offset(
                counter,
                singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            )
            .0;
            let bytes = read(pool, device, offset, 4096);
            if bytes[..4] != singlefs_core::journal::JOURNAL_MAGIC {
                break;
            }
            records.push((
                counter,
                u32::from_le_bytes(
                    bytes[JOURNAL_RECORD_INSTANCE_OFFSET..JOURNAL_RECORD_INSTANCE_OFFSET + 4]
                        .try_into()
                        .expect("4 字节"),
                ),
                get_u64(&bytes, JOURNAL_RECORD_TRANSACTION_OFFSET),
            ));
        }
        assert!(
            u64::try_from(records.len()).expect("条数") < RECORDS_READ_BACK_AT_MOST,
            "环里的记录多到读满了上界 {RECORDS_READ_BACK_AT_MOST}：这几份镜像上不该有这么多"
        );
        records
    }

    /// 一条 journal 记录的事务号改成 `transaction`，头校验和重封（罩整条 4096，字段在 46）；两盘各一份都改——
    /// 事务号重号是写者算错号，不是一块盘的字节坏了，两盘会写出同一条记录。
    fn set_transaction_number_of_journal_record(
        pool: &mut MemoryPool,
        counter: u64,
        expected_transaction: u64,
        transaction: u64,
    ) {
        let offset = singlefs_core::journal::record_offset(
            counter,
            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
        )
        .0;
        for device in DEVICES {
            let mut record = read(pool, device, offset, 4096);
            assert_eq!(
                get_u64(&record, JOURNAL_RECORD_TRANSACTION_OFFSET),
                expected_transaction,
                "盘 {device} 计数器 {counter} 那条记录的事务号"
            );
            set_u64(&mut record, JOURNAL_RECORD_TRANSACTION_OFFSET, transaction);
            let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
            record[46..78].copy_from_slice(&digest);
            write(pool, device, offset, &record);
        }
    }

    /// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）：加密类型在整槽校验和（155，宽 32）、自身 MAC（16）、nonce 水位（12）、
    /// KDF 标识（4）之后，偏移 219；第一版只读 0（不加密）。
    const SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET: usize = 219;
    /// 登记过的加密算法，第一版读者不读加密的卷：带它的系统配置槽自证过也整池拒绝挂载。
    const ENCRYPTION_TYPE_THE_FIRST_VERSION_DOES_NOT_READ: u8 = 1;
    /// I-7.13（系统配置池级字段在读者收的范围里） 那份坏镜像改的那一槽：盘 1 的槽 0。
    const DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT: u32 = 1;
    const OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT: u64 = 0;
    /// 系统配置字段表（`.claude/kb/layout/01-first-txn.md` 一）几何段里几个 16 KiB 槽号与环长的偏移：journal 环起点 325、环长 333、
    /// 根环起点 371、单元区起始槽号 417。checker 与实现今天各自按这几个偏移读；这里照字段表各写一份。
    const SYSTEM_CONFIGURATION_JOURNAL_RING_START_SLOT_OFFSET: usize = 325;
    const SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET: usize = 333;
    const SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET: usize = 371;
    const SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET: usize = 417;
    /// 系统配置自举头里的本盘设备号（4 字节）：fsid（偏移 102，宽 16）之后 20 字节、再 1 字节之后，偏移 139。
    const SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_OFFSET: usize = 139;
    /// 第一版 mkfs 写进这几格的值（字段表那一列）：journal 环从槽 1024 起、默认环长 768 MiB、根环从槽 64 起、单元区从槽 50176 起。
    const JOURNAL_RING_START_SLOT_OF_THE_POOL: u64 = 1024;
    const JOURNAL_RING_BYTES_OF_THE_POOL: u64 = 805_306_368;
    const ROOT_RING_BASE_SLOT_OF_THE_POOL: u64 = 64;
    const UNIT_AREA_START_SLOT_OF_THE_POOL: u64 = 50176;
    /// 聚簇段 64 槽（D3（空间分配） 已定项 10 ①）：单元区起始槽号要落在它的整数倍上。
    const CLUSTER_SEGMENT_SLOTS_OF_THE_POOL: u64 = 64;

    /// 系统配置槽重封：自证校验和在 155，罩整槽 4096。
    fn mutate_system_configuration_slot(
        pool: &mut MemoryPool,
        device: u32,
        slot_offset: u64,
        change: impl Fn(&mut Vec<u8>),
    ) {
        let mut bytes = read(pool, device, slot_offset, 4096);
        change(&mut bytes);
        let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 155);
        bytes[155..187].copy_from_slice(&digest);
        write(pool, device, slot_offset, &bytes);
    }

    fn set_u64(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
    fn set_u16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn get_u32(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
    }

    fn set_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    /// 两块盘对调盘体：每块盘两个系统配置槽里的本盘设备号改成另一块盘的身份（盘 0 的两槽写 1、盘 1 的两槽写 0），整槽重封，
    /// 别的字节不动。与盘体真对调不同的只有这一格：根环区域、单元都还在原来的盘上，所以别的不变量照判成立。
    fn swap_the_own_device_numbers_of_the_two_devices(image: &mut MemoryPool) {
        let spacing = u64::from(parameters().geometry.fixed_structure_slot_spacing);
        for (device, own_device_number_written) in [(0u32, 1u32), (1, 0)] {
            for slot_offset in [0, spacing] {
                mutate_system_configuration_slot(image, device, slot_offset, |bytes| {
                    set_u32(
                        bytes,
                        SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_OFFSET,
                        own_device_number_written,
                    );
                });
            }
        }
    }

    /// I-7.13（系统配置池级字段在读者收的范围里） 的几何那五格（实审 Y4 的 A、B、C、D、E，用户 2026-09-27 定「读字段，不等于常量就整池拒」）：
    /// 盘 1 槽 0 的一个几何字段改掉、整槽重封，别的三槽照原样。每项带 checker 该报的那个 `Verdict` 成员名（违例说明里带着它）。
    fn known_bad_images_of_the_system_configuration_geometry() -> Vec<(&'static str, Mutation)> {
        let on_the_refused_slot = |change: fn(&mut Vec<u8>)| -> Mutation {
            Box::new(move |image: &mut MemoryPool| {
                mutate_system_configuration_slot(
                    image,
                    DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
                    OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
                    change,
                );
            })
        };
        vec![
            // A：单元区起始槽号往后挪一段（50176 → 50240），环长不动：不是环长现算的那个槽。
            (
                "UnitAreaStartNotTheSlotAfterTheJournalRing",
                on_the_refused_slot(|bytes| {
                    set_u64(
                        bytes,
                        SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET,
                        UNIT_AREA_START_SLOT_OF_THE_POOL + CLUSTER_SEGMENT_SLOTS_OF_THE_POOL,
                    );
                }),
            ),
            // D：环长少一槽（768 MiB − 16 KiB），单元区起始槽号跟着改成它现算的 50175（两者自洽）：不在聚簇段边界上。
            (
                "UnitAreaStartOffTheClusterSegmentBoundary",
                on_the_refused_slot(|bytes| {
                    set_u64(
                        bytes,
                        SYSTEM_CONFIGURATION_JOURNAL_RING_BYTES_OFFSET,
                        JOURNAL_RING_BYTES_OF_THE_POOL - SLOT,
                    );
                    set_u64(
                        bytes,
                        SYSTEM_CONFIGURATION_UNIT_AREA_START_SLOT_OFFSET,
                        UNIT_AREA_START_SLOT_OF_THE_POOL - 1,
                    );
                }),
            ),
            // B：journal 环起点写 1023：实现按常量 1024 写记录、扫环，按字段扫的读者判的是错位 16 KiB 的另一个环。
            (
                "JournalRingStartSlotNotTheFirstVersionConstant",
                on_the_refused_slot(|bytes| {
                    set_u64(
                        bytes,
                        SYSTEM_CONFIGURATION_JOURNAL_RING_START_SLOT_OFFSET,
                        JOURNAL_RING_START_SLOT_OF_THE_POOL - 1,
                    );
                }),
            ),
            // C：根环起点写 65：实现按常量 64 写根，按字段读的读者读的是另一个根环。
            (
                "RootRingBaseSlotNotTheFirstVersionConstant",
                on_the_refused_slot(|bytes| {
                    set_u64(
                        bytes,
                        SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET,
                        ROOT_RING_BASE_SLOT_OF_THE_POOL + 1,
                    );
                }),
            ),
            // E：根环起点写 0：先于槽距判，报起点、不报「槽 1 不在根环起点之前」。
            (
                "RootRingBaseSlotNotTheFirstVersionConstant",
                on_the_refused_slot(|bytes| {
                    set_u64(bytes, SYSTEM_CONFIGURATION_ROOT_RING_BASE_SLOT_OFFSET, 0);
                }),
            ),
        ]
    }

    /// 树表单元里第 index 条条目（条目区从 115 + 2 × 8 = 131 起，每条 200）。
    fn tree_table_entry_offset(index: usize) -> usize {
        131 + 200 * index
    }

    /// 一个码 2 节点的自描述三段（D18（块里携带什么信息） 已定项 7：key 宽 1 字节在 51、条目数 2 字节在 82 + 2k、
    /// 条目宽 2 字节在 84 + 2k；声明长度 2 字节在 8，条目区从 86 + 2k + 29 起）改成「条目宽 = key 宽、只装一条条目」：
    /// 条目数改 1、声明长度改 k、largest_key 贴到那一条条目的 key 上、条目区之后的字节清零。
    ///
    /// 这么改是为了让**除 I-1.10（码 2 条目宽等于字段表宽） 之外的每一道都还成立**：声明长度 = 条目数 × 条目宽、
    /// 条目宽 ≥ key 宽（`index_node_view` 判的那两道）、key 区间贴紧首末条目（I-1.1（块头自述逻辑地址））、
    /// 声明长度之后的补齐为零且在载荷 CRC 内（I-2.3（补齐字节为零且参与校验和））。
    /// 盘上留下的就是一个处处自洽、唯独条目宽与那棵树的字段表不符的节点——
    /// I-1.10（码 2 条目宽等于字段表宽） 判的正是这一格，判不出来它，走读下一步就按字段表的固定偏移切这条 k 字节的条目、当场越界
    /// （panic 面普查 R13）。
    fn narrow_the_entry_width_to_the_key_width(bytes: &mut [u8]) {
        let key_width = usize::from(bytes[51]);
        let key_width_field = u16::try_from(key_width).expect("key 宽装得进 u16");
        let entries_start = 86 + 2 * key_width + 29;
        let first_key: Vec<u8> = bytes[entries_start..entries_start + key_width].to_vec();
        set_u16(bytes, 8, key_width_field);
        set_u16(bytes, 82 + 2 * key_width, 1);
        set_u16(bytes, 84 + 2 * key_width, key_width_field);
        bytes[52 + key_width..52 + 2 * key_width].copy_from_slice(&first_key);
        for byte in &mut bytes[entries_start + key_width..] {
            *byte = 0;
        }
    }

    /// 码 2 节点头里出生序号那 4 字节的偏移**扣掉 2k**（D18（块里携带什么信息） 已定项 7 的字段表：
    /// 诞生代号 52 + 2k、fsid 60 + 2k、写序 68 + 2k、出生序号 72 + 2k；k 是节点自述的 key 宽，在偏移 51）。
    const INDEX_NODE_BIRTH_SEQUENCE_OFFSET_BEFORE_THE_KEYS: usize = 72;
    /// 码 1 数据单元头里诞生代号那 8 字节的偏移（D18（块里携带什么信息） 已定项 7 的字段表）。
    const DATA_UNIT_BIRTH_TXG_OFFSET: usize = 75;
    /// extent 树上段叶（key 宽 24）里第一条条目内联的那条数据指针中诞生代号那 8 字节的偏移：
    /// 条目区从 115 + 2 × 24 = 163 起，条目 = key (0, inode 8, 0) 24 + 标签 1（2 = 单单元内联）+ 数据指针 88
    /// （D8（核心索引结构） 已定项 14），指针里诞生代号在 42。
    const EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET: usize = 163 + 24 + 1 + 42;
    /// 一条树表条目里诞生 txg 那 8 字节的偏移：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2 + 根指针 86 + `previous_snapshot_txg` 8
    /// （D8（核心索引结构） 已定项 8 的字段表）。
    const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;
    /// 分配记录树叶（key 宽 10）：条目数在 82 + 2 × 10、条目区从 115 + 2 × 10 起，每条 20 = key (设备 4, 槽号 6) +
    /// value (跨度段 2，最高位是已释放标志, 分配代或释放代 8)。
    const ALLOCATION_NODE_ENTRY_COUNT_OFFSET: usize = 102;
    const ALLOCATION_NODE_ENTRY_START: usize = 135;
    const ALLOCATION_RECORD_BYTES: usize = 20;
    const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;

    fn get_u64(bytes: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
    }

    /// 中央映射树（根兼叶，key 宽 27、条目 55）里唯一那条码 1 条目：key 的出生 txg（key 偏移 9）从 `from` 改成 `to`。
    /// 码 1 排在最前（类标签 1 最小），头里的最小 key 就是它的 key，跟着改；改完仍按 key 升序。
    fn rewrite_the_data_unit_mapping_key_birth_txg(bytes: &mut [u8], from: u64, to: u64) {
        const MAPPING_KEY_BIRTH_TXG_OFFSET: usize = 9;
        const HEADER_SMALLEST_KEY_OFFSET: usize = 52;
        let key_width = usize::from(bytes[51]);
        assert_eq!(key_width, 27, "映射 key 宽 27");
        let entries_start = 115 + 2 * key_width;
        assert_eq!(bytes[entries_start], 1, "第一条是码 1 那一条");
        assert_eq!(
            bytes[entries_start..entries_start + key_width],
            bytes[HEADER_SMALLEST_KEY_OFFSET..HEADER_SMALLEST_KEY_OFFSET + key_width],
            "头里的最小 key 就是第一条的 key"
        );
        for key_start in [entries_start, HEADER_SMALLEST_KEY_OFFSET] {
            assert_eq!(
                get_u64(bytes, key_start + MAPPING_KEY_BIRTH_TXG_OFFSET),
                from,
                "码 1 映射 key 的出生 txg 本来是 {from}"
            );
            set_u64(bytes, key_start + MAPPING_KEY_BIRTH_TXG_OFFSET, to);
        }
    }

    /// 实审 B2 新立的「中央映射条目的 key 与它指的单元头相符」：I-1.11（映射 key 与单元头相符）。
    const MAPPING_KEY_INVARIANT: &str =
        singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER;

    /// 中央映射树（根兼叶，key 宽 27、条目 55）里指着 `slot` 那个码 2 节点的那条条目：key 尾段的出生序号（key 偏移 21，4 字节）+1。
    /// 头里的最小 / 最大 key 是它时跟着改；改完仍按 key 严格升序（码 2 的 key 按出生树分开，这一条与左右邻不同出生树）。
    fn raise_the_mapping_key_tail_of_the_node_at(bytes: &mut [u8], slot: u64) {
        const MAPPING_KEY_TAIL_OFFSET: usize = 21;
        const MAPPING_ENTRY_BYTES: usize = 55;
        const MAPPING_ENTRY_FIRST_LOCATION_SLOT_OFFSET: usize = 27 + 4;
        const HEADER_SMALLEST_KEY_OFFSET: usize = 52;
        let key_width = usize::from(bytes[51]);
        assert_eq!(key_width, 27, "映射 key 宽 27");
        let header_largest_key_offset = HEADER_SMALLEST_KEY_OFFSET + key_width;
        let entry_count = usize::from(u16::from_le_bytes([
            bytes[82 + 2 * key_width],
            bytes[83 + 2 * key_width],
        ]));
        let entries_start = 115 + 2 * key_width;
        let entry_start = (0..entry_count)
            .map(|index| entries_start + index * MAPPING_ENTRY_BYTES)
            .find(|start| {
                bytes[*start] == 2
                    && get_u64(bytes, start + MAPPING_ENTRY_FIRST_LOCATION_SLOT_OFFSET)
                        & 0xFFFF_FFFF_FFFF
                        == slot
            })
            .expect("映射里有指着这个码 2 节点的那一条");
        let old_key = bytes[entry_start..entry_start + key_width].to_vec();
        let tail = entry_start + MAPPING_KEY_TAIL_OFFSET;
        let birth_sequence = u32::from_le_bytes(bytes[tail..tail + 4].try_into().expect("4 字节"));
        bytes[tail..tail + 4].copy_from_slice(&(birth_sequence + 1).to_le_bytes());
        let new_key = bytes[entry_start..entry_start + key_width].to_vec();
        for header_key in [HEADER_SMALLEST_KEY_OFFSET, header_largest_key_offset] {
            if bytes[header_key..header_key + key_width] == old_key[..] {
                bytes[header_key..header_key + key_width].copy_from_slice(&new_key);
            }
        }
    }

    /// 分配记录树叶里，把代字段等于 `from` 的已释放记录改成 `to`；返回改了几条。
    fn rewrite_released_generations(bytes: &mut [u8], from: u64, to: u64) -> usize {
        let count = usize::from(u16::from_le_bytes([
            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
        ]));
        let mut rewritten = 0;
        for index in 0..count {
            let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
            let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
            if span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0
                && get_u64(bytes, record + 12) == from
            {
                set_u64(bytes, record + 12, to);
                rewritten += 1;
            }
        }
        rewritten
    }

    /// 分配记录树叶里槽号是 `slot` 的记录（两盘各一条）跨度改成 `span_slots`，已释放标志照留；返回改了几条。
    fn set_allocation_record_span(bytes: &mut [u8], slot: u64, span_slots: u16) -> usize {
        let count = usize::from(u16::from_le_bytes([
            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
        ]));
        let mut rewritten = 0;
        for index in 0..count {
            let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
            let mut slot_bytes = [0u8; 8];
            slot_bytes[..6].copy_from_slice(&bytes[record + 4..record + 10]);
            if u64::from_le_bytes(slot_bytes) == slot {
                let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
                set_u16(
                    bytes,
                    record + 10,
                    (span_field & ALLOCATION_RECORD_RELEASED_FLAG) | span_slots,
                );
                rewritten += 1;
            }
        }
        rewritten
    }

    /// 一份坏镜像：它要让哪一条判违例，以及怎么从干净镜像改出来。
    type Mutation = Box<dyn Fn(&mut MemoryPool)>;

    fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {
        let leaf_head_checksum = crc32_castagnoli(&read(clean, 0, INODE_LEAF * SLOT, 16384));
        let stray_node = {
            let mut stray = read(clean, 0, EXTENT_ROOT * SLOT, 16384);
            set_u64(&mut stray, 42, 65624);
            reseal_unit(&mut stray);
            stray
        };
        let orphan_with_instance_two = {
            let mut orphan = read(clean, 0, DATA_UNIT * SLOT, 32768);
            orphan[91..95].copy_from_slice(&2u32.to_le_bytes());
            reseal_unit(&mut orphan);
            orphan
        };
        let orphan_for_second_sentence = orphan_with_instance_two.clone();
        let record_with_instance_two = {
            let offset = singlefs_core::journal::record_offset(
                3,
                singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            )
            .0;
            let mut record = read(clean, 0, offset, 4096);
            record[16..20].copy_from_slice(&2u32.to_le_bytes());
            let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
            record[46..78].copy_from_slice(&digest);
            record
        };
        vec![
            // 第 0 代根（暖机 txg 1、2）引用的 mkfs 树表单元（50178，A 换下之后只有它们引用）内容改掉再重封：那条候选根指着的块被复用。
            (
                "I-7.4",
                Box::new(|image: &mut MemoryPool| {
                    let mut unit = read(image, 0, GENESIS_TREE_TABLE * SLOT, 16384);
                    set_u64(&mut unit, 42, 77);
                    reseal_unit(&mut unit);
                    for device in DEVICES {
                        write(image, device, GENESIS_TREE_TABLE * SLOT, &unit);
                    }
                }),
            ),
            // 同一个单元的头抹成零、不重封：从第 0 代根出发的遍历读不到它。
            (
                "I-4.8",
                Box::new(|image: &mut MemoryPool| {
                    let erased = vec![0u8; 16384];
                    for device in DEVICES {
                        write(image, device, GENESIS_TREE_TABLE * SLOT, &erased);
                    }
                }),
            ),
            // 数据单元五元组里的锚点偏移改成 32768（extent key 的偏移是 0）。
            (
                "I-1.1",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, DATA_UNIT, |bytes| set_u64(bytes, 67, 32768), true)
                }),
            ),
            // extent 树根头里的树 ID 改成 99。
            (
                "I-1.3",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, EXTENT_ROOT, |bytes| set_u64(bytes, 42, 99), true)
                }),
            ),
            // inode 叶容器头里的 fsid 改掉。
            (
                "I-1.4",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 81, 0xDEAD), true)
                }),
            ),
            // 盘 1 被择的那个系统配置槽里 fsid 改成别的池的（一块外来的旧盘）；单元头照旧。
            (
                "I-1.4",
                Box::new(|image: &mut MemoryPool| {
                    mutate_system_configuration_slot(image, 1, 4096, |bytes| bytes[102] ^= 0xFF)
                }),
            ),
            // 码 3 容器类身份段的首字节改成 1。
            (
                "I-1.6",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, INODE_LEAF, |bytes| bytes[42] = 1, true)
                }),
            ),
            // 类型 2 容器的记录宽改成 139（登记的是 140）。
            (
                "I-1.7",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, INODE_LEAF, |bytes| set_u16(bytes, 71, 139), true)
                }),
            ),
            // 记账树根自述的条目宽缩到 key 宽 22（登记的字段表宽 34），别的自描述字段跟着调成自洽的一份。
            (
                "I-1.10",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        ACCOUNTING_ROOT,
                        |bytes| narrow_the_entry_width_to_the_key_width(bytes),
                        true,
                    )
                }),
            ),
            // 数据单元盘 0 那一份翻一个载荷字节，不补任何校验和。
            (
                "I-2.1",
                Box::new(|image: &mut MemoryPool| {
                    let mut bytes = read(image, 0, DATA_UNIT * SLOT, 32768);
                    bytes[200] ^= 1;
                    write(image, 0, DATA_UNIT * SLOT, &bytes);
                }),
            ),
            // 数据单元补齐区写一个非零字节，载荷 CRC 跟着重算（校验和对、补齐不为 0）。
            (
                "I-2.3",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, DATA_UNIT, |bytes| bytes[32000] = 1, true)
                }),
            ),
            // 数据单元头里翻一个字节，不重封头校验和（整单元校验和照样补上去，I-2.1 不先挡）。
            (
                "I-2.4",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, DATA_UNIT, |bytes| bytes[60] ^= 1, false)
                }),
            ),
            // 树表里 extent 树那条的根指针，两条位置条目对调（盘 1 在前）。
            (
                "I-2.5",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        TREE_TABLE,
                        |bytes| {
                            let pointer = tree_table_entry_offset(0) + 14;
                            let first: Vec<u8> = bytes[pointer + 50..pointer + 64].to_vec();
                            let second: Vec<u8> = bytes[pointer + 64..pointer + 78].to_vec();
                            bytes[pointer + 50..pointer + 64].copy_from_slice(&second);
                            bytes[pointer + 64..pointer + 78].copy_from_slice(&first);
                        },
                        true,
                    );
                }),
            ),
            // 记账树里盘 0 的「已分配」多记 16384。
            (
                "I-3.1",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        ACCOUNTING_ROOT,
                        |bytes| adjust_accounting(bytes, 1, 0, 16384),
                        true,
                    )
                }),
            ),
            // 树表里分配记录树那条的根指针，盘 0 那条位置条目改指槽 50243（压在 inode 叶容器的第二个槽上）。
            (
                "I-5.1",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        TREE_TABLE,
                        |bytes| {
                            let location = tree_table_entry_offset(2) + 14 + 50 + 4;
                            bytes[location..location + 6]
                                .copy_from_slice(&50243u64.to_le_bytes()[..6]);
                        },
                        true,
                    );
                }),
            ),
            // 记账树里盘 0 的「空闲」多记 16384。
            (
                "I-5.2",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        ACCOUNTING_ROOT,
                        |bytes| adjust_accounting(bytes, 2, 0, 16384),
                        true,
                    )
                }),
            ),
            // 根环全部槽抹成 0。
            (
                "I-7.1",
                Box::new(|image: &mut MemoryPool| {
                    for (device, offset) in root_slots() {
                        write(image, device, offset, &[0u8; 512]);
                    }
                }),
            ),
            // inode 叶容器两份都改坏一个字节（不补校验和）：最新的根走不完。
            (
                "I-7.2",
                Box::new(|image: &mut MemoryPool| {
                    for device in DEVICES {
                        let mut bytes = read(image, device, INODE_LEAF * SLOT, 32768);
                        bytes[5000] ^= 1;
                        write(image, device, INODE_LEAF * SLOT, &bytes);
                    }
                }),
            ),
            // 盘 0 被择的那个系统配置槽（槽 1，世代 5）里逐区域设备改成 [0, 0, 0]。
            (
                "I-7.6",
                Box::new(|image: &mut MemoryPool| {
                    mutate_system_configuration_slot(image, 0, 4096, |bytes| {
                        bytes[379..391].copy_from_slice(&[0u8; 12])
                    })
                }),
            ),
            // 盘 1 槽 0 的加密类型改成 1、重算整槽校验和：这一槽自证过而带这一版读者不收的值，实现整池拒绝挂载
            // （A3-checker-2 第四节第 1 条，别的三槽照原样）。
            (
                SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
                Box::new(|image: &mut MemoryPool| {
                    mutate_system_configuration_slot(
                        image,
                        DEVICE_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
                        OFFSET_OF_THE_REFUSED_SYSTEM_CONFIGURATION_SLOT,
                        |bytes| {
                            bytes[SYSTEM_CONFIGURATION_ENCRYPTION_TYPE_OFFSET] =
                                ENCRYPTION_TYPE_THE_FIRST_VERSION_DOES_NOT_READ
                        },
                    )
                }),
            ),
            // 两块盘四个系统配置槽里的本盘设备号互换、整槽重封（实审 Y2-b 越格线索：盘体对调之后的池可写挂载因本盘设备号拒）。
            (
                SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
                Box::new(swap_the_own_device_numbers_of_the_two_devices),
            ),
            // C322（取号那一步的屏障怎么放没有条款） 的撞号镜像：没人引用的槽 50302 上放一份写序实例代号 2 的数据单元（头与载荷
            // 校验和都过），两盘系统配置都还是实例 1 ⇒ ① 红（旧的「各盘相等且 ≥ 根环」在这里判成立）。
            (
                "I-7.7",
                Box::new(move |image: &mut MemoryPool| {
                    write(image, 0, 50302 * SLOT, &orphan_with_instance_two)
                }),
            ),
            // journal 环里一个空槽放一条实例代号 2 的记录（头校验和重封）⇒ ① 红：扫描方向要读 journal 环。
            (
                "I-7.7",
                Box::new(move |image: &mut MemoryPool| {
                    write(
                        image,
                        0,
                        singlefs_core::journal::record_offset(
                            9,
                            singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                        )
                        .0,
                        &record_with_instance_two,
                    )
                }),
            ),
            // 盘 0 较新那一槽的实例代号改成 2，再放那份实例代号 2 的孤儿 ⇒ 各盘最大号 [2, 1] 不等而较大者有东西带着，② 红（① 成立）。
            (
                "I-7.7",
                Box::new(move |image: &mut MemoryPool| {
                    mutate_system_configuration_slot(image, 0, 4096, |bytes| {
                        bytes[477..481].copy_from_slice(&2u32.to_le_bytes())
                    });
                    write(image, 0, 50302 * SLOT, &orphan_for_second_sentence);
                }),
            ),
            // 在没人引用的槽 50300 上放一个树 ID 65624（高于水位 65548）的码 2 节点（头校验和过、诞生代号 3）。
            (
                "I-7.8",
                Box::new(move |image: &mut MemoryPool| write(image, 0, 50300 * SLOT, &stray_node)),
            ),
            // 树表里 inode 树那条的根指针改指 inode 叶容器（码 3），校验和取那个容器前 16 KiB 的 CRC。
            (
                "I-9.1",
                Box::new(move |image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        TREE_TABLE,
                        |bytes| {
                            let pointer = tree_table_entry_offset(1) + 14;
                            for location in [pointer + 50, pointer + 64] {
                                bytes[location + 4..location + 10]
                                    .copy_from_slice(&INODE_LEAF.to_le_bytes()[..6]);
                                bytes[location + 10..location + 14]
                                    .copy_from_slice(&leaf_head_checksum.to_le_bytes());
                            }
                        },
                        true,
                    );
                }),
            ),
            // inode 树根那条内部条目的身份引用里容器号改成 2（子头里是 1）。
            (
                "I-9.2",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, INODE_ROOT, |bytes| set_u64(bytes, 131 + 18, 2), true)
                }),
            ),
            // 容器号改成 5，而容器里那条记录的 inode 是 1。
            (
                "I-9.4",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 53, 5), true)
                }),
            ),
            // inode 记录的预留段（记录偏移 139）写一个非零字节。
            (
                "I-9.7",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, INODE_LEAF, |bytes| bytes[136 + 139] = 1, true)
                }),
            ),
            // inode 记录的对象出生代改成 2（数据单元五元组里是 3）。
            (
                "I-9.10",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(image, INODE_LEAF, |bytes| set_u64(bytes, 136 + 8, 2), true)
                }),
            ),
            // inode 记录的 blocks（记录偏移 48）写回按分到一个 32 KiB 单元算的 64，而 size 3000 ⇒ ⌈3000 ÷ 512⌉ = 6。
            (
                "I-9.15",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        INODE_LEAF,
                        |bytes| set_u64(bytes, 136 + INODE_RECORD_BLOCKS_OFFSET, 64),
                        true,
                    )
                }),
            ),
            // 树表里 livelist 与稀疏旁表两条互换种类：盘上仍按树 ID 升序（I-1.1 不红），按发号次序 livelist 的号 65542 大过稀疏旁表的号 65541。
            (
                "I-9.16",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        TREE_TABLE,
                        |bytes| swap_the_kinds_of_the_livelist_and_sparse_side_table_entries(bytes),
                        true,
                    )
                }),
            ),
            // 实例表里多一行 (1, 3, 0)：行的实例代号等于挂载根的实例 1，不「低于」；链指针记录仍在末尾、行仍唯一，红的只有那一半。
            (
                "I-3.8",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        INSTANCE_TABLE,
                        |bytes| {
                            let chain_record: Vec<u8> = bytes[136..224].to_vec();
                            let mut row = vec![0u8; 88];
                            row[1..5].copy_from_slice(&1u32.to_le_bytes());
                            row[5..13].copy_from_slice(&3u64.to_le_bytes());
                            bytes[136..224].copy_from_slice(&row);
                            bytes[224..312].copy_from_slice(&chain_record);
                            set_u16(bytes, 69, 2);
                            set_u16(bytes, 8, 176);
                        },
                        true,
                    );
                }),
            ),
            // 盘 0 环内偏移 4096 那条记录（w4，暖机第二条空发布）的反向链字段改成 0，头校验和重封：
            // 它不是本实例第一条（本实例第一条是 w1、计数器 1），链值必须等于 CRC32C(w1 的 487 字节头)。
            (
                "I-8.6",
                Box::new(|image: &mut MemoryPool| {
                    let offset = singlefs_core::journal::record_offset(
                        2,
                        singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                    )
                    .0;
                    let mut record = read(image, 0, offset, 4096);
                    let back_chain_field =
                        JOURNAL_RECORD_BACK_CHAIN_OFFSET..JOURNAL_RECORD_BACK_CHAIN_OFFSET + 4;
                    assert_ne!(
                        u32::from_le_bytes(
                            record[back_chain_field.clone()].try_into().expect("4 字节")
                        ),
                        0,
                        "w4 的反向链本来是 CRC32C(w1 的头)，非 0"
                    );
                    record[back_chain_field].copy_from_slice(&0u32.to_le_bytes());
                    let digest = wide_checksum_with_field_zeroed(&record, 4096, 46);
                    record[46..78].copy_from_slice(&digest);
                    write(image, 0, offset, &record);
                }),
            ),
            // inode 叶容器**盘 1 那一份**的载荷改一个字节、重封载荷校验和与头校验和，类身份段与写序一个字节不动：
            // 两份归并成同一组（归并键逐字节相同）而载荷不同。盘 1 那条位置条目跟着补上 ⇒ I-2.1 不先挡。
            (
                "I-1.8",
                Box::new(|image: &mut MemoryPool| {
                    mutate_one_device_copy_of_unit(image, &UNITS, 1, INODE_LEAF, |bytes| {
                        assert_eq!(bytes[20000], 0, "记录区之后的补齐本来是 0");
                        bytes[20000] = 1;
                    });
                }),
            ),
            // extent 树根（码 2，key 宽 24）头里的出生序号 +1，引用它的树表条目里那条指针一个字节不动：
            // 头里的出生身份与引用它的指针记的不符 ⇒ I-1.2（块头写序已发布） 红。
            // 出生序号不是已发布谓词的输入（谓词只读诞生代号与写序），所以 I-4.2（无被引用未提交块） 不跟着红。
            // 中央映射里 extent 根那条 key 的尾段就是它的出生序号（D19（块指针的结构与宽度预算） 已定项 6）：跟着 +1，
            // 不跟着改就是映射 key 与单元头不符、映射 key 那一条（I-1.11（映射 key 与单元头相符）） 一起红。
            (
                "I-1.2",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        EXTENT_ROOT,
                        |bytes| {
                            let offset = INDEX_NODE_BIRTH_SEQUENCE_OFFSET_BEFORE_THE_KEYS
                                + 2 * usize::from(bytes[51]);
                            let birth_sequence = u32::from_le_bytes(
                                bytes[offset..offset + 4].try_into().expect("4 字节"),
                            );
                            bytes[offset..offset + 4]
                                .copy_from_slice(&(birth_sequence + 1).to_le_bytes());
                        },
                        true,
                    );
                    mutate_unit(
                        image,
                        MAPPING_ROOT,
                        |bytes| raise_the_mapping_key_tail_of_the_node_at(bytes, EXTENT_ROOT),
                        true,
                    );
                }),
            ),
            // 中央映射里数据单元那条 key 的出生 txg 从 3 改成 4，数据单元一个字节不动：映射 key 与它指的单元头里的诞生代号不符
            // ⇒ 映射 key 那一条（I-1.11（映射 key 与单元头相符）） 红。
            (
                MAPPING_KEY_INVARIANT,
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        MAPPING_ROOT,
                        |bytes| rewrite_the_data_unit_mapping_key_birth_txg(bytes, 3, 4),
                        true,
                    );
                }),
            ),
            // 数据单元头里的诞生代号从 3 改成 4（最新的根就是 txg 3、实例 1），引用它的 extent 记录里那条数据指针的
            // 诞生代号跟着改成 4：指针与头仍然一致（I-1.2（块头写序已发布） 不跟着红），而这个**被引用**的块按
            // C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的已发布谓词判不成已发布（i == 挂载根实例 ⇒ 要 b ≤ 挂载根的 txg）
            // ⇒ I-4.2（无被引用未提交块） 红：盘上留下的正是「提交了的根指着一个还没提交的块」。
            // 分配记录树里那个落点的分配代也跟着改成 4（两盘各一条）：记录与单元同源，不跟着改就是 I-3.10（已分配记录的分配代等于
            // 它罩住的单元的诞生代号） 一起红，判别力算不到 I-4.2 头上。
            (
                "I-4.2",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        DATA_UNIT,
                        |bytes| {
                            assert_eq!(
                                get_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET),
                                3,
                                "数据单元的诞生代号本来是 3"
                            );
                            set_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET, 4);
                        },
                        true,
                    );
                    mutate_unit(
                        image,
                        EXTENT_ROOT,
                        |bytes| {
                            assert_eq!(
                                get_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET),
                                3,
                                "extent 记录里那条数据指针的诞生代号本来是 3"
                            );
                            set_u64(bytes, EXTENT_RECORD_DATA_POINTER_BIRTH_TXG_OFFSET, 4);
                        },
                        true,
                    );
                    // 中央映射里那个数据单元的 key 带着它的诞生代号（D19（块指针的结构与宽度预算） 已定项 6）：跟着改成 4，
                    // 不跟着改就是映射 key 与单元头的出生身份不符、I-1.2 一起红（实审 B1 起 checker 拿映射 key 比单元头）。
                    mutate_unit(
                        image,
                        MAPPING_ROOT,
                        |bytes| rewrite_the_data_unit_mapping_key_birth_txg(bytes, 3, 4),
                        true,
                    );
                    assert_eq!(
                        mutate_allocation_leaves(image, &UNITS, ALLOCATION_LEAVES, |bytes| {
                            rewrite_unreleased_allocation_generation(bytes, DATA_UNIT, 3, 4)
                        }),
                        2,
                        "数据单元两盘各一条未释放记录（两片叶各一条），分配代本来是 3"
                    );
                }),
            ),
            // 记账里的 inode 号水位从 2 改成 1：树里最大的 key 就是 1，水位不再严格大于它（I-9.6）。
            // 别的行一个字节都不动 ⇒ I-3.1 / I-5.2 那两条照旧绿。
            (
                "I-9.6",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        ACCOUNTING_ROOT,
                        |bytes| set_accounting(bytes, 12, 0xFFFF_FFFF, 1),
                        true,
                    );
                }),
            ),
            // inode 根那条条目的分隔 key 从 1 改成 2（连着把节点头的 key 区间也改成 [2, 2]，
            // 不然 I-1.1（key 区间罩住条目） 先红、这一格就成了搭车）：分隔 key 2 大于这个孩子的最小 key 1 ⇒ I-9.12 红。
            (
                "I-9.12",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        INODE_ROOT,
                        |bytes| {
                            set_u64(bytes, 131, 2);
                            set_u64(bytes, 52, 2);
                            set_u64(bytes, 60, 2);
                        },
                        true,
                    );
                }),
            ),
            // 类型 2 容器的记录数与声明长度都改成 0。
            (
                "I-9.13",
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        INODE_LEAF,
                        |bytes| {
                            set_u16(bytes, 69, 0);
                            set_u16(bytes, 8, 0);
                            bytes[136..276].fill(0);
                        },
                        true,
                    );
                }),
            ),
            // I-3.5：A 的数据单元（50180）头里的诞生代号 3 → 2，重封、补到根：A 引用、暖机那一版（txg 2）不引用的单元诞生代号不大于 2，
            // 也不在整棵继承下来的用户可见树里（extent 树根指针出生 txg 3）。指针、映射 key 与分配代还记 3，I-1.2、I-1.11、I-3.10 跟着红，
            // 所以放在这一张表里（只判「它红」，不判「只红它」）；按各个组成部分的几份在
            // `checker_judges_reference_intervals_and_every_new_root_segment_field.rs`。
            (
                REFERENCE_INTERVALS_ARE_EXACT,
                Box::new(|image: &mut MemoryPool| {
                    mutate_unit(
                        image,
                        DATA_UNIT,
                        |bytes| {
                            assert_eq!(
                                get_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET),
                                3,
                                "A 的数据单元生在 txg 3"
                            );
                            set_u64(bytes, DATA_UNIT_BIRTH_TXG_OFFSET, 2);
                        },
                        true,
                    );
                }),
            ),
            // I-5.5（判决 `research/prompts/c673-clauses-r1-main-verification.md` 第二节 K2）：A 那一版是从 mkfs 水位第一次建树的那一版，
            // 它唯一的数据单元（50180 = 单元区起始槽号 50176 + 4）搬到再往后两槽，每块盘上落点最小的数据单元就不在 U + 4。
            // 分配记录与记账照旧记着 50180，这份镜像上不只红 I-5.5；这一格只要它判违例。
            (
                FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
                Box::new(move_the_data_unit_of_the_first_file_two_slots_further),
            ),
        ]
    }

    /// 新池新建文件写出的那个数据单元（`DATA_UNIT`，32 KiB）原样搬到往后两槽（写完新池新建文件那份镜像上空着，下一个 32 KiB 对齐的槽对）：
    /// 两盘各写一份；`UNITS` 里引用它的单元（extent 树根里的数据指针、中央映射叶）把位置条目的槽号改成新的、重封，
    /// 新的整单元校验和沿引用链补到根槽。50180 那两份原样留着：journal 记录的点名项仍指着它。
    fn move_the_data_unit_of_the_first_file_two_slots_further(image: &mut MemoryPool) {
        const DATA_UNIT_BYTES: usize = 32768;
        let moved_to = DATA_UNIT + 2;
        assert!(
            UNITS.iter().all(|(start, _)| *start != moved_to),
            "槽 {moved_to} 在写完新池新建文件的镜像上空着"
        );
        let data_unit = read(image, 0, DATA_UNIT * SLOT, DATA_UNIT_BYTES);
        for device in DEVICES {
            write(image, device, moved_to * SLOT, &data_unit);
        }
        let checksum = crc32_castagnoli(&data_unit);
        let mut parents_changed = Vec::new();
        for (container, length) in UNITS.iter().copied() {
            if container == DATA_UNIT {
                continue;
            }
            let before = read(image, 0, container * SLOT, length);
            let mut bytes = before.clone();
            let mut touched = false;
            for device in DEVICES {
                touched |= replace_all(
                    &mut bytes,
                    &location_pattern(device, DATA_UNIT, checksum),
                    &location_pattern(device, moved_to, checksum),
                );
            }
            if touched {
                reseal_unit(&mut bytes);
                for device in DEVICES {
                    write(image, device, container * SLOT, &bytes);
                }
                parents_changed.push((
                    container,
                    crc32_castagnoli(&before),
                    crc32_castagnoli(&bytes),
                ));
            }
        }
        assert!(
            !parents_changed.is_empty(),
            "写完新池新建文件的镜像上有单元指着那个数据单元"
        );
        propagate(image, &UNITS, parents_changed);
    }

    /// 发布 B（txg 4）之后的干净镜像：A 的十二个单元（十四个槽）在这次发布里改写成已释放、释放代 4，树表是 B 写的那一版，
    /// A 那一版仍被根环里 A 的根指着。I-3.9（释放代落在停止引用它的那一格区间里）与 I-9.14（树表条目的诞生 txg 跨根不变）
    /// 要的跨根比较到这里才有内容：写完新池新建文件的那份镜像上只有 A 的根有树表条目（mkfs 种的第 0 版树表是空的、
    /// 两次暖机空发布不写树表），I-9.14 在那份镜像上报「不适用」。

    // 以下是散在原文件 `known_bad_images` 之后、被上面用到的常量与辅助函数，在 e426cdff… 那一份里的行号：3191 1629 3299 3381 3366 3239 3736 3233 3235 3236 3234 3231

    /// inode 记录里 `blocks` 的偏移（D8（核心索引结构） 已定项 6 的偏移表：size 在 40，blocks 在 48）。
    const INODE_RECORD_BLOCKS_OFFSET: usize = 48;

    /// 记录头里反向链那 4 字节的偏移：事务段 78 + 事务号 8 + 提交标记 1 + 本次发布内序号 4（D23（journal 的角色与格式） 已定项 4 / 已定项 8）。
    const JOURNAL_RECORD_BACK_CHAIN_OFFSET: usize = 91;

    /// 新池新建文件写出的中央映射树根（bump 次序：分配记录树五个节点、记账树、映射树、树表）。
    const MAPPING_ROOT: u64 = 50251;

    /// 记账树叶里 (统计量, 设备) 那一行的值加 delta（条目区从 115 + 2 × 22 = 159 起，每条 34：统计量 2、树 8、设备 4、代 8、值 8、seq 4）。
    fn adjust_accounting(bytes: &mut [u8], statistic: u16, device: u32, delta: u64) {
        let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
        for index in 0..count {
            let row = 159 + 34 * index;
            if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
                && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == device
            {
                let value = u64::from_le_bytes(bytes[row + 22..row + 30].try_into().expect("8"));
                bytes[row + 22..row + 30].copy_from_slice(&(value + delta).to_le_bytes());
                return;
            }
        }
        panic!("记账树里没有 ({statistic}, {device}) 这一行");
    }

    /// 记账树叶里 (统计量, 设备) 那一行的值改成 value（偏移口径同 `adjust_accounting`）。
    fn set_accounting(bytes: &mut [u8], statistic: u16, device: u32, value: u64) {
        let count = usize::from(u16::from_le_bytes([bytes[82 + 44], bytes[83 + 44]]));
        for index in 0..count {
            let row = 159 + 34 * index;
            if u16::from_le_bytes([bytes[row], bytes[row + 1]]) == statistic
                && u32::from_le_bytes(bytes[row + 10..row + 14].try_into().expect("4")) == device
            {
                bytes[row + 22..row + 30].copy_from_slice(&value.to_le_bytes());
                return;
            }
        }
        panic!("记账树里没有 ({statistic}, {device}) 这一行");
    }

    /// 树表单元里 livelist 与稀疏旁表两条互换种类（条目偏移 10 的 2 字节）；树 ID、根指针不动。
    fn swap_the_kinds_of_the_livelist_and_sparse_side_table_entries(tree_table: &mut [u8]) {
        for (index, kind_before, kind_after) in [
            (
                LIVELIST_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_LIVELIST,
                TREE_KIND_SPARSE_SIDE_TABLE,
            ),
            (
                SPARSE_SIDE_TABLE_TREE_TABLE_ENTRY_INDEX,
                TREE_KIND_SPARSE_SIDE_TABLE,
                TREE_KIND_LIVELIST,
            ),
        ] {
            let kind_offset = tree_table_entry_offset(index) + TREE_TABLE_ENTRY_KIND_OFFSET;
            assert_eq!(
                u16::from_le_bytes([tree_table[kind_offset], tree_table[kind_offset + 1]]),
                kind_before,
                "树表第 {index} 条的种类"
            );
            set_u16(tree_table, kind_offset, kind_after);
        }
    }

    /// 分配记录树叶里槽号是 `slot`、**未带**已释放标志、分配代等于 `from` 的记录（两盘各一条）改成 `to`；返回改了几条。
    fn rewrite_unreleased_allocation_generation(
        bytes: &mut [u8],
        slot: u64,
        from: u64,
        to: u64,
    ) -> usize {
        let count = usize::from(u16::from_le_bytes([
            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET],
            bytes[ALLOCATION_NODE_ENTRY_COUNT_OFFSET + 1],
        ]));
        let mut rewritten = 0;
        for index in 0..count {
            let record = ALLOCATION_NODE_ENTRY_START + ALLOCATION_RECORD_BYTES * index;
            let mut slot_bytes = [0u8; 8];
            slot_bytes[..6].copy_from_slice(&bytes[record + 4..record + 10]);
            let span_field = u16::from_le_bytes([bytes[record + 10], bytes[record + 11]]);
            if u64::from_le_bytes(slot_bytes) == slot
                && span_field & ALLOCATION_RECORD_RELEASED_FLAG == 0
                && get_u64(bytes, record + 12) == from
            {
                set_u64(bytes, record + 12, to);
                rewritten += 1;
            }
        }
        rewritten
    }

    /// 新池新建文件那一版树表里十一条按发号次序排：livelist 是第 4 条（从 0 数，号 65541）、稀疏旁表第 5 条（号 65542）。
    const LIVELIST_TREE_TABLE_ENTRY_INDEX: usize = 4;

    const TREE_KIND_LIVELIST: u16 = 6;

    const TREE_KIND_SPARSE_SIDE_TABLE: u16 = 7;

    const SPARSE_SIDE_TABLE_TREE_TABLE_ENTRY_INDEX: usize = 5;

    /// 树表条目里种类那 2 字节的偏移（树 ID 8 + 条目长度 2 之后，D8（核心索引结构） 已定项 8）。
    const TREE_TABLE_ENTRY_KIND_OFFSET: usize = 10;

    /// 对外的口子：第 755 行的 `known_bad_images` 是私有的，这里转一手（类型就是第 753 行的 `Mutation`）。
    pub(super) fn entries_of_the_known_bad_images(
        clean: &MemoryPool,
    ) -> Vec<(&'static str, Box<dyn Fn(&mut MemoryPool)>)> {
        known_bad_images(clean)
    }
}

// ───────────────────────────── 阳性对照 O2：已知坏镜像族第一条（登记 5.2） ─────────────────────────────

/// 把一个崩溃镜像物化成 `MemoryPool`：基镜像拷一份，按写表次序施加持久了的写。
fn materialized_memory_pool(image: &CrashImage<'_>) -> MemoryPool {
    let mut pool = image.base.clone();
    let persisted_writes: Vec<RetainedWrite> = image
        .writes
        .iter()
        .zip(&image.persisted)
        .filter(|(_, is_persisted)| **is_persisted)
        .map(|(write, _)| write.clone())
        .collect();
    pool.apply_writes(&persisted_writes);
    pool
}

/// 阳性对照 O2：把已知坏镜像族的第一条改法施加在这一态的物化镜像上，再交 `check_pool_image`（只为计时，名单走两段）。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
fn pool_checker_on_the_first_known_bad_image(
    image: &CrashImage<'_>,
) -> Vec<(&'static str, InvariantVerdict)> {
    check_pool_image(&first_known_bad_image_of(image))
}

/// 这一态的物化镜像上施加已知坏镜像族的第一条改法之后的那张盘（阳性对照 O2 判的那一张）。
fn first_known_bad_image_of(image: &CrashImage<'_>) -> MemoryPool {
    let clean = materialized_memory_pool(image);
    let entries =
        known_bad_images_copied_from_the_harness_test::entries_of_the_known_bad_images(&clean);
    let (_, apply_the_first_entry) = entries
        .first()
        .expect("已知坏镜像族至少一条（照抄的那一份 43 条）");
    let mut bad = clean.clone();
    apply_the_first_entry(&mut bad);
    bad
}

/// 这份装置判取样那几态 O2 名单的给定状态引擎（进程里一份，第一次用时起；用户 2026-10-14 定：O2 的名单按判到位与违例位能两段，
/// 计时与按件 panic 留 CPU）：每张要判的盘作一张给定状态表，录入段写进家目录下给定状态表那一类的判定库，判段从库里读回判
/// （配了 GPU 且编进了 `gpu` 特性在卡上判，否则由 CPU 判器判），判过的复用。产物只收这份装置自己打的行：引擎不调 `finish`，不打收尾行。
/// 照第一段装置（`e171_three_oracle_collusion.rs`）的同名函数。
///
/// # Panics
/// 引擎起不了（配了 GPU 卡用不上、结论版本与编译时对不上……都不退回 CPU）。
fn given_states_engine_of_this_experiment(
) -> &'static singlefs_checker_tier::crash_amplification::GivenStatesEngine {
    static ENGINE: std::sync::OnceLock<
        singlefs_checker_tier::crash_amplification::GivenStatesEngine,
    > = std::sync::OnceLock::new();
    ENGINE.get_or_init(|| {
        singlefs_checker_tier::crash_amplification::GivenStatesEngine::start_from_environment(
            singlefs_checker_tier::crash_amplification::GivenStatesItem {
                item_name: "e171_three_oracle_collusion_after_the_oracle_fixes::fixed".to_string(),
            },
            &singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    })
}

/// 取样的一态上 O2 判红的不变量名（按 `IMPLEMENTED_INVARIANTS` 的次序，与 [`violated_invariant_names`] 同一个次序），走两段：
/// 这一态的物化镜像（阳性对照那一态是改坏的那一张，[`first_known_bad_image_of`]）作一张给定状态表（基线就是那张盘、写表空、一个空持久集合）
/// 交 [`given_states_engine_of_this_experiment`]。表名带流、序号与镜像摘要（同名的表内容就相同，判过的复用）。
///
/// 交回 `None` 是这一趟没判名单：引擎只录了这张表、交占位结论（配置开了 GPU 而这份二进制没编进 `gpu` 特性，或设了只录那一段），
/// 判归编进特性的那一趟；引擎开跑时已打了本次未跑那一行（`given_states_not_run_line_without_the_gpu_feature`），
/// 这里不退回 CPU 判，也不把占位当 panic 记进按件 panic 的计数。
///
/// # Panics
/// 引擎判不了这张表（不退回别处判）；这张表归别的机器判（实验装置不分片）。
fn pool_checker_red_names_of_a_sampled_state_through_the_given_states_engine(
    stream_name: StreamName,
    ordinal: u64,
    image_digest: &str,
    image: &CrashImage<'_>,
    pool_checker_control: bool,
) -> Option<Vec<String>> {
    use singlefs_checker_tier::crash_given_states::{
        GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
    };
    let (table_name, judged_image) = if pool_checker_control {
        (
            format!(
                "sampled-state/{}/ordinal={ordinal}/image={image_digest}/first-known-bad-entry",
                stream_name.directory_name()
            ),
            first_known_bad_image_of(image),
        )
    } else {
        (
            format!(
                "sampled-state/{}/ordinal={ordinal}/image={image_digest}",
                stream_name.directory_name()
            ),
            materialized_memory_pool(image),
        )
    };
    let judged = given_states_engine_of_this_experiment()
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: table_name.clone(),
            base: &judged_image,
            writes: &[],
            segments: &[],
            seams: &[],
            persisted_sets: vec![Vec::new()],
        })
        .unwrap_or_else(|reason| {
            panic!("给定状态表 {table_name} 判不了（不退回别处判）：{reason}")
        });
    match judged.source {
        GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
        GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => return None,
        GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost => panic!(
            "实验装置不分片：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_SHARE（{table_name} 归别的机器判）"
        ),
    }
    let [conclusion] = judged.states.as_slice() else {
        panic!(
            "{table_name}：一个状态一份结论，交回 {} 份",
            judged.states.len()
        );
    };
    Some(
        conclusion
            .pool_checker_violations()
            .into_iter()
            .map(|(invariant, _)| invariant.to_string())
            .collect(),
    )
}

/// 一条流各项计数（[`report_stream`] 交回、`run_dump` 合起来写判决行）。
#[derive(Clone, Copy, Debug, Default)]
struct StreamTotals {
    failed_checks: usize,
    rebuilt_digest_mismatches: usize,
    mask_mismatches: usize,
    s3_mismatches: usize,
    comparison_red_calls: usize,
    crash_judge_red_states: usize,
    reference_filesystem_red_states: usize,
    pool_checker_red_states: usize,
    recovery_panic_states: usize,
    /// 各件 panic 的态数（修订 5 第 5.1 节第 5 条，按 [`JudgingPiece::ALL`] 的次序）。
    piece_panic_states: [usize; JUDGING_PIECE_COUNT],
    /// 正常那一路 panic、逐件一件都没 panic 的态数（[`unattributed_panic_states_of`]）。
    unattributed_panic_states: usize,
    probed_states: usize,
    /// (ii) 为真的态、其中第二截判合法的、(i) 为真的态（Q4）。
    counter_branch_states: usize,
    counter_branch_states_judged_legal: usize,
    undecidable_branch_states: usize,
    /// Q4-对账（停机 S11）：拒因是系统配置见证而 (i)(ii) 都不为真的态；(ii) 为真而挂上了的态。
    witness_refusals_without_either_branch: usize,
    counter_branch_states_that_mounted: usize,
}

/// 一条流的结果行（`E7RESULT` 与 `E7TIMING`）；交回各项计数。
fn report_stream(lines: &mut ResultLines, export: &StreamExport) -> StreamTotals {
    let label = export.name.directory_name();
    let mut totals = StreamTotals::default();
    for check in &export.checks {
        totals.failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream={label} check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let ending = export
        .ending
        .as_ref()
        .map_or_else(|| "pipeline_panicked".to_string(), ending_text);
    lines.emit_result_line(&format!(
        "name=stream stream={label} ending={ending} exported={} not_exported_why={} operations={} writes={} segments={} tearable_writes={} calls={} full_states={} quick_tier_states={} sampled_states={} model_stopped_at_call={} reference_filesystem_segments={} reference_filesystem_stopped={} comparison_judged_calls_before_the_first_panic={}",
        export.not_exported.is_none(),
        export
            .not_exported
            .as_deref()
            .map_or_else(|| "-".to_string(), field_text),
        export.operation_count,
        export.write_count,
        export.segment_count,
        export.tearable_write_count,
        export.calls.len(),
        export.full_states,
        export.quick_tier_states,
        export.observations.len(),
        export
            .model_stopped_at_call
            .map_or_else(|| "none".to_string(), |call| call.to_string()),
        export.reference_filesystem_segments,
        export.reference_filesystem_stopped,
        export
            .calls
            .iter()
            .take_while(|call| !matches!(call.outcome, CallOutcome::Panicked(_)))
            .filter(|call| call.model.was_judged_by_the_comparison())
            .count(),
    ));
    for (call_index, call) in export.calls.iter().enumerate() {
        totals.comparison_red_calls +=
            usize::from(matches!(call.model, ModelVerdictOfCall::Disagrees { .. }));
        lines.emit_result_line(&format!(
            "name=call stream={label} call={call_index} label={} first_operation={} return_point={} status={} o1_comparison={} return_value_sha256={} memory_objects_sha256={}",
            call.label,
            call.first_operation,
            call.return_point,
            field_text(&call.outcome.outcome_word()),
            field_text(&call.model.word()),
            call.return_value
                .as_ref()
                .map_or_else(|| "-".to_string(), |value| value.whole.clone()),
            memory_objects_digest(&call.memory_objects),
        ));
    }
    for scan in &export.torn_scans {
        lines.emit_result_line(&format!(
            "name=torn_scan stream={label} segment={} scanned={} found={} wanted={TORN_STATES_PER_SEGMENT}",
            scan.segment_index, scan.scanned, scan.found
        ));
    }
    for batch in 1..=3u8 {
        let in_batch: Vec<&StateObservation> = export
            .observations
            .iter()
            .filter(|observation| observation.batch == batch)
            .collect();
        report_batch(lines, label, batch, &in_batch, &mut totals);
    }
    totals.mask_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.mask_matches_the_facts)
        .count();
    totals.s3_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.pool_checker_matches_the_judgement)
        .count();
    totals.rebuilt_digest_mismatches = export.rebuilt_digest_mismatches;
    lines.emit_result_line(&format!(
        "name=s4 stream={label} rebuilt_states={} digest_mismatches={}",
        export.rebuilt_states, export.rebuilt_digest_mismatches
    ));
    lines.emit_result_line(&format!(
        "name=crash_facts_mask stream={label} states={} mismatches={}",
        export.observations.len(),
        totals.mask_mismatches
    ));
    lines.emit_result_line(&format!(
        "name=s3 stream={label} states={} o2_direct_differs_from_judge_crash_image={}",
        export.observations.len(),
        totals.s3_mismatches
    ));
    report_files_and_timing(lines, label, export);
    totals
}

/// 一批的计数行（O1 三个子列、O2、恢复 panic、记录核对器与手写版本表的附带计数）、探针那几样，与阶段耗时。
fn report_batch(
    lines: &mut ResultLines,
    label: &str,
    batch: u8,
    in_batch: &[&StateObservation],
    totals: &mut StreamTotals,
) {
    let count_where = |predicate: &dyn Fn(&StateObservation) -> bool| {
        in_batch
            .iter()
            .filter(|observation| predicate(observation))
            .count()
    };
    let crash_judge_is =
        |word: &'static str| count_where(&|observation| observation.crash_judge.word() == word);
    let reference_filesystem_is = |word: &'static str| {
        count_where(&|observation| observation.reference_filesystem.word() == word)
    };
    let o2_red = count_where(&|observation| {
        observation
            .pool_checker_red
            .as_ref()
            .is_some_and(|items| !items.is_empty())
    });
    totals.crash_judge_red_states += crash_judge_is("red");
    totals.reference_filesystem_red_states += reference_filesystem_is("red");
    totals.pool_checker_red_states += o2_red;
    totals.recovery_panic_states +=
        count_where(&|observation| observation.recovery_panic.is_some());
    let piece_panic_states = piece_panic_states_of(in_batch.iter().copied());
    for (total, states) in totals.piece_panic_states.iter_mut().zip(piece_panic_states) {
        *total += states;
    }
    totals.unattributed_panic_states += unattributed_panic_states_of(in_batch.iter().copied());
    let segments: BTreeSet<usize> = in_batch
        .iter()
        .map(|observation| observation.segment_index)
        .collect();
    lines.emit_result_line(&format!(
        "name=batch stream={label} batch={batch} states={} segments_touched={} states_with_a_torn_write={} o1_crash_judge_red={} o1_crash_judge_green={} o1_crash_judge_not_applicable={} o1_crash_judge_off={} o1_reference_filesystem_red={} o1_reference_filesystem_green={} o1_reference_filesystem_abstained={} o1_reference_filesystem_off={} recovery_panic_states={} o2_red={o2_red} o2_matches_judgement_false={} judge_crash_image_pool_checker_red={} record_checker_red_states={} consulted_oracle_red_states={} ignored_oracle_red_states={} consulted_effective_root_none={} consulted_failed_outcomes={} piece_panic_states={}",
        in_batch.len(),
        segments.len(),
        count_where(&|observation| observation.landing_vector.contains('1')),
        crash_judge_is("red"),
        crash_judge_is("green"),
        crash_judge_is("na"),
        crash_judge_is("off"),
        reference_filesystem_is("red"),
        reference_filesystem_is("green"),
        reference_filesystem_is("abstained"),
        reference_filesystem_is("off"),
        count_where(&|observation| observation.recovery_panic.is_some()),
        count_where(&|observation| !observation.pool_checker_matches_the_judgement),
        count_where(&|observation| !observation.verdict.pool_checker_red.is_empty()),
        count_where(&|observation| observation.verdict.red_items.iter().any(|item| item.starts_with("record_checker:"))),
        count_where(&|observation| observation.verdict.red_items.iter().any(|item| item.starts_with("journal_consulted_oracle:"))),
        count_where(&|observation| observation.verdict.red_items.iter().any(|item| item.starts_with("journal_ignored_oracle:"))),
        count_where(&|observation| observation.verdict.consulted_root == "none"),
        count_where(&|observation| observation.verdict.consulted_outcome.starts_with("failed:")),
        piece_panic_states_text(&piece_panic_states),
    ));
    let abstention_kinds: BTreeMap<&'static str, usize> =
        in_batch
            .iter()
            .fold(BTreeMap::new(), |mut kinds, observation| {
                if let ReferenceFileSystemVoteOfAState::Abstained { kind } =
                    &observation.reference_filesystem
                {
                    *kinds.entry(*kind).or_insert(0) += 1;
                }
                kinds
            });
    for (kind, states) in abstention_kinds {
        lines.emit_result_line(&format!(
            "name=reference_filesystem_abstentions stream={label} batch={batch} kind={} states={states}",
            field_text(kind)
        ));
    }
    let probes: Vec<(&StateObservation, &WritableMountProbe)> = in_batch
        .iter()
        .filter_map(|observation| {
            observation
                .probe
                .as_ref()
                .map(|probe| (*observation, probe))
        })
        .collect();
    if !probes.is_empty() {
        let probe_count_where = |predicate: &dyn Fn(&WritableMountProbe) -> bool| {
            probes.iter().filter(|(_, probe)| predicate(probe)).count()
        };
        let counter_branch =
            |probe: &WritableMountProbe| probe.criterion.counter_sentence.is_some();
        let undecidable = |probe: &WritableMountProbe| {
            probe
                .criterion
                .some_device_has_fewer_than_two_verified_slots
        };
        let counter_branch_legal =
            probe_count_where(&|probe| counter_branch(probe) && probe.second_crop == "legal");
        let without_either = probe_count_where(&|probe| {
            probe.refused_for_the_system_configuration_witness
                && !counter_branch(probe)
                && !undecidable(probe)
        });
        let counter_branch_mounted =
            probe_count_where(&|probe| counter_branch(probe) && probe.mount == "mounted");
        totals.probed_states += probes.len();
        totals.counter_branch_states += probe_count_where(&counter_branch);
        totals.counter_branch_states_judged_legal += counter_branch_legal;
        totals.undecidable_branch_states += probe_count_where(&undecidable);
        totals.witness_refusals_without_either_branch += without_either;
        totals.counter_branch_states_that_mounted += counter_branch_mounted;
        lines.emit_result_line(&format!(
            "name=probe_batch stream={label} batch={batch} probed_states={} mounted={} refused={} refused_for_the_witness={} undecidable_branch={} undecidable_branch_judged_legal={} counter_branch={} counter_branch_judged_legal={counter_branch_legal} counter_branch_judged_illegal={} witness_refusals_without_either_branch={without_either} counter_branch_mounted={counter_branch_mounted}",
            probes.len(),
            probe_count_where(&|probe| probe.mount == "mounted"),
            probe_count_where(&|probe| probe.mount == "refused"),
            probe_count_where(&|probe| probe.refused_for_the_system_configuration_witness),
            probe_count_where(&undecidable),
            probe_count_where(&|probe| undecidable(probe) && probe.second_crop == "legal"),
            probe_count_where(&counter_branch),
            probe_count_where(&|probe| counter_branch(probe) && probe.second_crop.starts_with("illegal")),
        ));
        for (observation, probe) in probes
            .iter()
            .filter(|(_, probe)| counter_branch(probe) || undecidable(probe))
        {
            lines.emit_result_line(&format!(
                "name=probe_state stream={label} batch={batch} ordinal={} segment={} landing_sha256={} undecidable_branch={} counter_branch={} witnessed_counter={} selected_version={} mount={} refusal={} on_the_d23_list={} second_crop={}",
                observation.ordinal,
                observation.segment_index,
                sha256_hexadecimal(observation.landing_vector.as_bytes()),
                undecidable(probe),
                probe.criterion.counter_sentence.map_or_else(|| "false".to_string(), |sentence| format!("{sentence:?}")),
                probe.criterion.witnessed_counter,
                probe.selected_version,
                probe.mount,
                field_text(&probe.refusal_member),
                field_text(&probe.refusal_on_the_d23_list),
                field_text(&probe.second_crop),
            ));
        }
    }
    report_stage_times(label, batch, in_batch);
}

/// 一批的阶段耗时（`E7TIMING`，不进入库产物）。
fn report_stage_times(label: &str, batch: u8, in_batch: &[&StateObservation]) {
    for (stage, pick) in [
        (
            "o1",
            (|observation: &StateObservation| observation.o1_nanoseconds)
                as fn(&StateObservation) -> u128,
        ),
        ("o2", |observation| observation.o2_nanoseconds),
        ("recovery", |observation| observation.recovery_nanoseconds),
    ] {
        let (total, median, p95, maximum) = timing_summary(
            in_batch
                .iter()
                .map(|observation| pick(observation))
                .collect(),
        );
        ResultLines::timing(&format!(
            "name=stage_time stream={label} batch={batch} stage={stage} states={} total_ns={total} median_ns={median} p95_ns={p95} max_ns={maximum}",
            in_batch.len()
        ));
    }
}

/// 写出的每个文件一行（带耗时的两个文件只在结果行里给去掉耗时之后的摘要）与这条流的分段耗时。
fn report_files_and_timing(lines: &mut ResultLines, label: &str, export: &StreamExport) {
    for file in &export.files {
        match &file.sha256_without_timing {
            None => lines.emit_result_line(&format!(
                "name=file stream={label} file={} bytes={} sha256={}",
                file.name, file.bytes, file.sha256
            )),
            Some(without_timing) => {
                lines.emit_result_line(&format!(
                    "name=file stream={label} file={} sha256_without_timing={without_timing}",
                    file.name
                ));
                ResultLines::timing(&format!(
                    "name=file_with_timing stream={label} file={} bytes={} sha256={}",
                    file.name, file.bytes, file.sha256
                ));
            }
        }
    }
    ResultLines::timing(&format!(
        "name=stream_time stream={label} elapsed_ns={} drive_ns={} observe_ns={} write_ns={}",
        export.elapsed_nanoseconds,
        export.drive_nanoseconds,
        export.observe_nanoseconds,
        export.write_nanoseconds
    ));
}

/// 基线上 O1 不适用与不比的来源（登记第十一节 S5、第一节读法 6）：卸载由模型答之后，基线上模型一次都不许停（三条流都是），
/// 每次调用要么对拍过（`agrees`）、要么是取号暖机那两次（`not_modelled`）。按 `StopCheck` 的形式回比：期望（本地）与实际（这一趟）。
fn s5_checks_of_the_baseline(export: &StreamExport) -> Vec<StopCheck> {
    let label = export.name.directory_name();
    vec![
        StopCheck::new(
            &format!("{label}_o1_model_never_stops"),
            None::<usize>,
            export.model_stopped_at_call,
        ),
        StopCheck::new(
            &format!("{label}_every_call_is_judged_or_is_the_acquire_or_the_warm_up"),
            true,
            export.calls.iter().all(|call| match call.model {
                ModelVerdictOfCall::Agrees => true,
                ModelVerdictOfCall::NotModelled => {
                    call.label == "acquire" || call.label == "warm_up"
                }
                ModelVerdictOfCall::Disagrees { .. }
                | ModelVerdictOfCall::SkippedAfterTheModelStopped
                | ModelVerdictOfCall::NotComparedBecauseTheImplementationCrashed
                | ModelVerdictOfCall::ModelCannotTakeTheObservation { .. } => false,
            }),
        ),
    ]
}

/// 逐行那一趟（坏实现、阳性对照；不关 oracle）的 S5：O1 不适用只许来自读法 6 那三处（对拍判出不同之后、调用交回错误之后、调用 panic 之后），
/// 都按「不合的个数」回比 0：① 交回了错误的调用没有一次留在「没问模型」（`not_modelled`）——取号、暖机出错那一形（代码轮第三轮判决 K4-c）；
/// 模型停下之后的调用（`skipped_after_the_model_stopped`）不数；② O1-模型崩溃判官的不适用只来自模型停在那三处之一的那次调用。
fn s5_checks_of_a_bad_implementation_row(
    label: &str,
    calls: &[RecordedCall],
    model_stopped_at_call: Option<usize>,
    not_applicable_states: usize,
) -> Vec<StopCheck> {
    let errored_calls_the_comparison_did_not_judge = calls
        .iter()
        .filter(|call| matches!(call.outcome, CallOutcome::ReturnedError(_)))
        .filter(|call| matches!(call.model, ModelVerdictOfCall::NotModelled))
        .count();
    let model_stopped_at_one_of_the_three_sources = model_stopped_at_call.is_none_or(|stopped| {
        calls.get(stopped).is_some_and(|call| {
            matches!(call.model, ModelVerdictOfCall::Disagrees { .. })
                || matches!(
                    call.outcome,
                    CallOutcome::ReturnedError(_) | CallOutcome::Panicked(_)
                )
        })
    });
    let not_applicable_states_with_another_source = if model_stopped_at_one_of_the_three_sources {
        0
    } else {
        not_applicable_states
    };
    vec![
        StopCheck::new(
            &format!("{label}_o1_every_errored_call_is_judged_by_the_comparison"),
            0usize,
            errored_calls_the_comparison_did_not_judge,
        ),
        StopCheck::new(
            &format!("{label}_o1_not_applicable_only_after_a_disagreement_an_error_or_a_panic"),
            0usize,
            not_applicable_states_with_another_source,
        ),
    ]
}

/// 这一趟、这一条流要判的 S5（登记第十一节 S5）。基线那一趟判 [`s5_checks_of_the_baseline`]：只判不带对照、不关 oracle、流导出了的基线。
/// 逐行那一趟判 [`s5_checks_of_a_bad_implementation_row`]：阳性对照照判（它让模型停在翻过的那次调用，是对拍的红）；关掉 O1 不判；
/// 流水线 panic（`ending` 是 `None`）的流不判。
fn s5_checks_to_judge(
    export: &StreamExport,
    run_kind: RunKind,
    switches: RunSwitches,
) -> Vec<StopCheck> {
    match run_kind {
        RunKind::Baseline => {
            if export.not_exported.is_none()
                && switches.positive_control.is_none()
                && switches.disabled_oracle.is_none()
            {
                s5_checks_of_the_baseline(export)
            } else {
                Vec::new()
            }
        }
        RunKind::BadImplementationRow => {
            if export.ending.is_some() && switches.disabled_oracle.is_none() {
                let not_applicable_states = export
                    .observations
                    .iter()
                    .filter(|observation| {
                        observation.crash_judge == CrashJudgeVerdict::NotApplicable
                    })
                    .count();
                s5_checks_of_a_bad_implementation_row(
                    export.name.directory_name(),
                    &export.calls,
                    export.model_stopped_at_call,
                    not_applicable_states,
                )
            } else {
                Vec::new()
            }
        }
    }
}

/// 停机 S9 的两样（每一趟都判，导出了的流才判）：RefFS 一侧切的段数与装置切的段数相等（第七节 A9）；翻译停下的理由只许是
/// 本装置走得到的那几种之一（[`ReferenceFileSystemStop`]，停在回退上说明映射或 m 表漏了东西）。
fn s9_checks_of_a_stream(export: &StreamExport) -> Vec<StopCheck> {
    if export.not_exported.is_some() {
        return Vec::new();
    }
    let label = export.name.directory_name();
    let stopped_on_a_rollback = export.reference_filesystem_stopped.contains(
        field_text(ReferenceFileSystemStop::TheRollbackTargetRootHasNoVersion.name()).as_str(),
    ) || export.reference_filesystem_stopped.contains(
        field_text(ReferenceFileSystemStop::TheReferenceFileSystemDoesNotTakeTheOperation.name())
            .as_str(),
    );
    vec![
        StopCheck::new(
            &format!("{label}_reference_filesystem_segments_equal_the_device_segments"),
            export.segment_count,
            export.reference_filesystem_segments,
        ),
        StopCheck::new(
            &format!("{label}_reference_filesystem_translation_does_not_stop_on_a_missing_root_or_a_refused_operation"),
            false,
            stopped_on_a_rollback,
        ),
    ]
}

struct CommandLine {
    run_kind: RunKind,
    output_directory: PathBuf,
    switches: RunSwitches,
    streams: Vec<StreamName>,
}

struct ConfirmedLossCommandLine {
    output_directory: PathBuf,
    truncate_at_the_return_point: bool,
}

enum ParsedCommand {
    Dump(CommandLine),
    StreamConstants(Vec<StreamName>),
    ConfirmedLossForm(ConfirmedLossCommandLine),
}

fn parse_command_line(arguments: &[String]) -> Option<ParsedCommand> {
    let mut words = arguments.iter();
    let mode = words.next()?.as_str();
    let mut output_directory = None;
    let mut switches = RunSwitches::default();
    let mut streams = STREAM_NAMES.to_vec();
    let mut truncate_at_the_return_point = false;
    while let Some(word) = words.next() {
        match word.as_str() {
            "--output-directory" => output_directory = Some(PathBuf::from(words.next()?)),
            "--streams" => streams = stream_names_of_the_numbers(words.next()?)?,
            "--writable-mount-probe" => switches.writable_mount_probe = true,
            "--truncate-at-the-return-point" => truncate_at_the_return_point = true,
            "--positive-control" => {
                switches.positive_control = Some(PositiveControl::parse(words.next()?)?);
            }
            "--disable-oracle" => {
                switches.disabled_oracle = Some(match words.next()?.as_str() {
                    "O1" => DisabledOracle::ComparisonAndCrashJudge,
                    "O2" => DisabledOracle::PoolChecker,
                    _ => return None,
                });
            }
            _ => return None,
        }
    }
    match mode {
        "baseline" | "row" => Some(ParsedCommand::Dump(CommandLine {
            run_kind: if mode == "baseline" {
                RunKind::Baseline
            } else {
                RunKind::BadImplementationRow
            },
            output_directory: output_directory?,
            switches,
            streams,
        })),
        "stream-constants" => Some(ParsedCommand::StreamConstants(streams)),
        "confirmed-loss-form" => Some(ParsedCommand::ConfirmedLossForm(ConfirmedLossCommandLine {
            output_directory: output_directory?,
            truncate_at_the_return_point,
        })),
        _ => None,
    }
}

/// S1 里第 1 次重跑新加的、与流无关的几条：一片 inode 叶容器的记录数、每盘系统配置槽数与槽长，对 `singlefs_format` 的同名公开常量。
fn stream_independent_checks_of_the_first_rerun() -> Vec<StopCheck> {
    vec![
        StopCheck::new(
            "inode_leaf_records",
            LOCAL_INODE_LEAF_RECORDS,
            singlefs_format::INODE_LEAF_RECORDS,
        ),
        StopCheck::new(
            "system_configuration_slots_per_device",
            u64::try_from(LOCAL_SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE).expect("2"),
            singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE,
        ),
        StopCheck::new(
            "system_configuration_slot_bytes",
            LOCAL_SYSTEM_CONFIGURATION_SLOT_BYTES,
            singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES,
        ),
    ]
}

fn run_dump(command_line: &CommandLine) -> i32 {
    let run_kind = command_line.run_kind;
    let switches = command_line.switches;
    let mut lines = ResultLines {
        emitted: 0,
        collected: None,
    };
    lines.emit_result_line(&format!(
        "name=config segment=s1 mode={} streams={} sector_bytes={LOCAL_SECTOR_BYTES} first_batch_divisor={FIRST_BATCH_DIVISOR} second_batch_divisor={SECOND_BATCH_DIVISOR} third_batch_divisor={THIRD_BATCH_DIVISOR} torn_states_per_segment={TORN_STATES_PER_SEGMENT} torn_scan_ordinals_per_segment={TORN_SCAN_ORDINALS_PER_SEGMENT} third_batch_sampled={} positive_control={} disabled_oracle={} writable_mount_probe={}",
        if run_kind == RunKind::Baseline { "baseline" } else { "row" },
        command_line
            .streams
            .iter()
            .map(|stream| stream.number().to_string())
            .collect::<Vec<String>>()
            .join(","),
        run_kind == RunKind::Baseline,
        switches
            .positive_control
            .map_or_else(|| "none".to_string(), |control| format!("{control:?}")),
        switches
            .disabled_oracle
            .map_or_else(|| "none".to_string(), |oracle| format!("{oracle:?}")),
        switches.writable_mount_probe,
    ));
    let mut failed_checks = 0usize;
    if run_kind == RunKind::Baseline {
        let mut independent = stream_independent_checks();
        independent.extend(stream_independent_checks_of_the_first_rerun());
        independent.extend(confirmed_loss_form_checks_against_the_crates());
        for check in independent {
            failed_checks += usize::from(!check.holds());
            lines.emit_result_line(&format!(
                "name=s1 stream=independent check={} local={} crates={} holds={}",
                check.name,
                field_text(&check.local),
                field_text(&check.crates),
                check.holds()
            ));
        }
    }
    let mut totals = StreamTotals::default();
    let mut streams_not_exported = 0usize;
    let mut s5_failed = 0usize;
    let mut s9_failed = 0usize;
    for name in &command_line.streams {
        let started = Instant::now();
        let output_root = command_line.output_directory.clone();
        let exported = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            export_stream(*name, &output_root, run_kind, switches)
        }));
        let mut export = exported.unwrap_or_else(|payload| {
            StreamExport::not_exported(
                *name,
                format!("pipeline_panicked:{}", panic_text(&*payload)),
                started,
            )
        });
        streams_not_exported += usize::from(export.not_exported.is_some());
        let s5 = s5_checks_to_judge(&export, run_kind, switches);
        s5_failed += s5.iter().filter(|check| !check.holds()).count();
        let s9 = s9_checks_of_a_stream(&export);
        s9_failed += s9.iter().filter(|check| !check.holds()).count();
        export.checks.extend(s5);
        export.checks.extend(s9);
        let stream_totals = report_stream(&mut lines, &export);
        add_stream_totals(&mut totals, &stream_totals);
    }
    failed_checks += totals.failed_checks;
    lines.emit_result_line(&format!(
        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} crash_facts_mask_mismatches={} s3_o2_judgement_mismatches={} s9_failed_checks={s9_failed} streams_not_exported={streams_not_exported} o1_comparison_red_calls={} o1_crash_judge_red_states={} o1_reference_filesystem_red_states={} o2_red_states={} recovery_panic_states={} probed_states={} counter_branch_states={} counter_branch_states_judged_legal={} undecidable_branch_states={} witness_refusals_without_either_branch={} counter_branch_states_that_mounted={} piece_panic_states={} unattributed_panic_states={}",
        totals.rebuilt_digest_mismatches,
        totals.mask_mismatches,
        totals.s3_mismatches,
        totals.comparison_red_calls,
        totals.crash_judge_red_states,
        totals.reference_filesystem_red_states,
        totals.pool_checker_red_states,
        totals.recovery_panic_states,
        totals.probed_states,
        totals.counter_branch_states,
        totals.counter_branch_states_judged_legal,
        totals.undecidable_branch_states,
        totals.witness_refusals_without_either_branch,
        totals.counter_branch_states_that_mounted,
        piece_panic_states_text(&totals.piece_panic_states),
        totals.unattributed_panic_states,
    ));
    lines.finish();
    let s11_hit = totals.witness_refusals_without_either_branch > 0
        || totals.counter_branch_states_that_mounted > 0;
    let stop_clause_hit = failed_checks > 0
        || totals.rebuilt_digest_mismatches > 0
        || totals.s3_mismatches > 0
        || s9_failed > 0
        || s11_hit
        || (run_kind == RunKind::Baseline && (s5_failed > 0 || streams_not_exported > 0));
    if stop_clause_hit {
        3
    } else {
        0
    }
}

fn add_stream_totals(totals: &mut StreamTotals, stream: &StreamTotals) {
    totals.failed_checks += stream.failed_checks;
    totals.rebuilt_digest_mismatches += stream.rebuilt_digest_mismatches;
    totals.mask_mismatches += stream.mask_mismatches;
    totals.s3_mismatches += stream.s3_mismatches;
    totals.comparison_red_calls += stream.comparison_red_calls;
    totals.crash_judge_red_states += stream.crash_judge_red_states;
    totals.reference_filesystem_red_states += stream.reference_filesystem_red_states;
    totals.pool_checker_red_states += stream.pool_checker_red_states;
    totals.recovery_panic_states += stream.recovery_panic_states;
    for (total, states) in totals
        .piece_panic_states
        .iter_mut()
        .zip(stream.piece_panic_states)
    {
        *total += states;
    }
    totals.unattributed_panic_states += stream.unattributed_panic_states;
    totals.probed_states += stream.probed_states;
    totals.counter_branch_states += stream.counter_branch_states;
    totals.counter_branch_states_judged_legal += stream.counter_branch_states_judged_legal;
    totals.undecidable_branch_states += stream.undecidable_branch_states;
    totals.witness_refusals_without_either_branch += stream.witness_refusals_without_either_branch;
    totals.counter_branch_states_that_mounted += stream.counter_branch_states_that_mounted;
}

// ───────────────────────────── Q5「已确认的丢了」那一形（读法 7） ─────────────────────────────

/// 照读法 7 改过的流二写流：第 2 次调用里第一条 journal 记录写之后、返回点之前的每一道屏障删掉，这次调用写根槽的那次 FUA 写改成普通写，
/// 截在 `truncation`（主取样点：第 3 次调用发出的第一道屏障；方向相反的取样点：第 2 次调用的返回点）。写的字节一个不改。
/// 下标都是 mkfs 之后录制流里的（改之前的）下标。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ConfirmedLossShape {
    removed_barriers: Vec<usize>,
    root_writes_made_plain: Vec<usize>,
    truncation: usize,
    reverse_truncation: usize,
}

/// 在驱动到第 3 次调用的流二上算读法 7 的造法下标（不改任何东西）。
fn confirmed_loss_shape_of(
    operations: &[RetainedOperation],
    calls: &[RecordedCall],
) -> ConfirmedLossShape {
    let geometry = common::geometry();
    let confirmed = &calls[CONFIRMED_LOSS_CONFIRMED_CALL];
    let in_flight = &calls[CONFIRMED_LOSS_IN_FLIGHT_CALL];
    let inside_the_confirmed_call = confirmed.first_operation..confirmed.return_point;
    let first_journal_record = inside_the_confirmed_call
        .clone()
        .find(|index| geometry.classify(&operations[*index].operation) == StepKind::JournalRecord)
        .expect("第一个文件那次发布写 journal 记录");
    let removed_barriers = (first_journal_record + 1..confirmed.return_point)
        .filter(|index| operations[*index].operation.kind == RecordedOperationKind::Barrier)
        .collect();
    let root_writes_made_plain = inside_the_confirmed_call
        .filter(|index| {
            operations[*index].operation.kind == RecordedOperationKind::WriteForceUnitAccess
                && geometry.classify(&operations[*index].operation) == StepKind::RootRecordFua
        })
        .collect();
    let truncation = (in_flight.first_operation..operations.len())
        .find(|index| operations[*index].operation.kind == RecordedOperationKind::Barrier)
        .unwrap_or(operations.len());
    ConfirmedLossShape {
        removed_barriers,
        root_writes_made_plain,
        truncation,
        reverse_truncation: confirmed.return_point,
    }
}

/// 改过、截过的写流；`at_the_return_point` 选方向相反的取样点。交回 (写流, 改之前的下标 → 改之后的下标；删掉的屏障没有)。
fn confirmed_loss_operations(
    operations: &[RetainedOperation],
    shape: &ConfirmedLossShape,
    at_the_return_point: bool,
) -> (Vec<RetainedOperation>, Vec<Option<usize>>) {
    let end = if at_the_return_point {
        shape.reverse_truncation
    } else {
        shape.truncation
    };
    let mut kept = Vec::new();
    let mut new_index_of = Vec::new();
    for (index, retained) in operations.iter().enumerate().take(end) {
        if shape.removed_barriers.contains(&index) {
            new_index_of.push(None);
            continue;
        }
        let mut changed = retained.clone();
        if shape.root_writes_made_plain.contains(&index) {
            changed.operation.kind = RecordedOperationKind::Write;
        }
        new_index_of.push(Some(kept.len()));
        kept.push(changed);
    }
    (kept, new_index_of)
}

/// 驱动流二到第 3 次调用（模型跟到第 2 次调用），交回驱动完的流（[`LiveStream::run_the_second_stream_through_the_first_overwrite`]）。
fn second_stream_through_the_first_overwrite(switches: RunSwitches) -> LiveStream {
    let mut live = LiveStream::start(
        StreamName::FixedScriptThroughEThenUnmount,
        "e171r1-confirmed-loss-form",
        switches,
    );
    live.run_the_second_stream_through_the_first_overwrite();
    live
}

/// Q5 那一形截在哪里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConfirmedLossTruncation {
    /// 主取样点：第 3 次调用发出的第一道屏障之前（读法 7）。
    BeforeTheInFlightBarrier,
    /// 方向相反的取样点：第 2 次调用的返回点（读法 7、第八节）。
    AtTheConfirmedReturnPoint,
    /// 「等价类对拍」那一形：第 3 次调用发出的前这么多个写之后（登记修订 2 第 2.1 节候选二最后一条，即候选三的造法）。
    AfterTheFirstInFlightWrites(usize),
}

/// 第 3 次调用发出的第 `writes_kept` 个写之后的下标（按发出次序数写，屏障不数）；主取样点截断之前第 3 次调用发出的写不够这么多个，
/// 截在主取样点的截断处（那里第 3 次调用的写已经全留着）。
fn truncation_after_the_first_writes_of_the_in_flight_call(
    operations: &[RetainedOperation],
    calls: &[RecordedCall],
    shape: &ConfirmedLossShape,
    writes_kept: usize,
) -> usize {
    let in_flight = &calls[CONFIRMED_LOSS_IN_FLIGHT_CALL];
    let mut writes_seen = 0usize;
    for (index, retained) in operations
        .iter()
        .enumerate()
        .take(shape.truncation)
        .skip(in_flight.first_operation)
    {
        match retained.operation.kind {
            RecordedOperationKind::Write
            | RecordedOperationKind::WriteForceUnitAccess
            | RecordedOperationKind::WriteZeroes => writes_seen += 1,
            RecordedOperationKind::Barrier => {}
        }
        if writes_seen == writes_kept {
            return index + 1;
        }
    }
    shape.truncation
}

/// Q5 那一形的流：改过、截过的写流重新切段，调用的下标照改之后的写流换，第 3 次调用记「在途」、返回点取截断处。
fn confirmed_loss_prepared_stream(
    live: LiveStream,
    truncation: ConfirmedLossTruncation,
) -> (PreparedStream, ConfirmedLossShape) {
    let mkfs_operation_count = live.formatted.mkfs_operation_count;
    let all_operations = live.formatted.retained_operations();
    let operations = &all_operations[mkfs_operation_count..];
    let mut shape = confirmed_loss_shape_of(operations, &live.driver.calls);
    let at_the_return_point = match truncation {
        ConfirmedLossTruncation::BeforeTheInFlightBarrier => false,
        ConfirmedLossTruncation::AtTheConfirmedReturnPoint => true,
        ConfirmedLossTruncation::AfterTheFirstInFlightWrites(writes_kept) => {
            shape.truncation = truncation_after_the_first_writes_of_the_in_flight_call(
                operations,
                &live.driver.calls,
                &shape,
                writes_kept,
            );
            false
        }
    };
    let (kept, new_index_of) = confirmed_loss_operations(operations, &shape, at_the_return_point);
    let remap = |old: usize| -> usize {
        new_index_of
            .iter()
            .take(old)
            .filter(|new_index| new_index.is_some())
            .count()
    };
    let mut calls: Vec<RecordedCall> = live
        .driver
        .calls
        .iter()
        .take(if at_the_return_point {
            CONFIRMED_LOSS_IN_FLIGHT_CALL
        } else {
            CONFIRMED_LOSS_IN_FLIGHT_CALL + 1
        })
        .cloned()
        .collect();
    for call in &mut calls {
        call.first_operation = remap(call.first_operation);
        call.return_point = remap(call.return_point);
    }
    if let Some(in_flight) = calls.get_mut(CONFIRMED_LOSS_IN_FLIGHT_CALL) {
        in_flight.outcome = CallOutcome::InFlightAtTheTruncation;
        in_flight.return_point = kept.len();
        in_flight.return_value = None;
    }
    let (writes, segments, stream_indexes) =
        writes_and_segments_with_stream_indexes_and_entries(&kept, &[], &common::geometry());
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .unwrap_or(usize::MAX);
    let prepared = PreparedStream {
        name: StreamName::FixedScriptThroughEThenUnmount,
        base: live.formatted.memory_pool_after_mkfs(),
        operations: kept,
        writes,
        segments,
        stream_indexes,
        calls,
        judged_root_index,
        versions: live.versions.clone(),
        committed_versions: live.driver.committed_versions.clone(),
        model_stopped_at_call: live.driver.model_stopped_at_call,
        checks: Vec::new(),
        ending: ending_of_the_driver(&live.driver),
    };
    (prepared, shape)
}

/// 末段每个写的取法数之积（整写 2 种，可撕裂的 3 种；第七节 A10），与末段写数、可撕裂写数。
fn last_segment_product(
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
) -> (usize, usize, u64) {
    let last_segment = stream.segments.last().cloned().unwrap_or_default();
    let tearable = last_segment
        .iter()
        .filter(|write_index| tables.is_tearable_by_write[**write_index])
        .count();
    let product = last_segment.iter().fold(1u64, |product, write_index| {
        product
            * if tables.is_tearable_by_write[*write_index] {
                3
            } else {
                2
            }
    });
    (last_segment.len(), tearable, product)
}

/// S1 里 Q5 造法在基线上的几样（5.3）：驱动一遍流二前四次调用，算造法下标、末段写数、可撕裂写数与全枚举态数，对本地常量。
fn confirmed_loss_form_checks_against_the_crates() -> Vec<StopCheck> {
    let live = second_stream_through_the_first_overwrite(RunSwitches::default());
    let (prepared, shape) =
        confirmed_loss_prepared_stream(live, ConfirmedLossTruncation::BeforeTheInFlightBarrier);
    let tables = crash_state_enumeration_tables(
        &prepared.base,
        &prepared.writes,
        &prepared.segments,
        &full_expansion,
    );
    let (writes, tearable, product) = last_segment_product(&prepared, &tables);
    // 等价类的起点分法（候选二）：只看末段每个写属于哪次调用与各自的进制，不跑恢复（细分要跑恢复，归 `confirmed-loss-form` 那一趟）。
    let digits = LastSegmentDigits::of(&prepared, &tables);
    let partition = ConfirmedLossPartition::before_any_refinement(
        &call_index_of_each_write(&prepared),
        &digits,
        CONFIRMED_LOSS_IN_FLIGHT_CALL,
    );
    let key_choices = partition.keyed_assignments(&digits);
    let states_of_a_landed_class = partition.states_with_the_in_flight_writes_landed(&digits);
    // 「等价类对拍」那一形的三样（修订 5 第 5.2 节 S1 新加三条）。
    let (pairing_stream, _) = confirmed_loss_prepared_stream(
        second_stream_through_the_first_overwrite(RunSwitches::default()),
        ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
            EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
        ),
    );
    let pairing_tables = crash_state_enumeration_tables(
        &pairing_stream.base,
        &pairing_stream.writes,
        &pairing_stream.segments,
        &full_expansion,
    );
    let pairing_facts = equivalence_pairing_shape_facts(&pairing_stream, &pairing_tables);
    vec![
        StopCheck::new(
            "equivalence_pairing_in_flight_writes",
            LOCAL_EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES,
            pairing_facts.in_flight_writes.len(),
        ),
        StopCheck::new(
            "equivalence_pairing_states",
            LOCAL_EQUIVALENCE_PAIRING_STATES,
            pairing_facts.states,
        ),
        StopCheck::new(
            "equivalence_pairing_mirrored_pairs",
            LOCAL_EQUIVALENCE_PAIRING_MIRRORED_PAIRS,
            pairing_facts.mirrored_pairs,
        ),
        StopCheck::new(
            "confirmed_loss_key_choices",
            LOCAL_CONFIRMED_LOSS_KEY_CHOICES,
            key_choices,
        ),
        StopCheck::new(
            "confirmed_loss_in_flight_writes",
            LOCAL_CONFIRMED_LOSS_IN_FLIGHT_WRITES,
            partition.class_positions.len(),
        ),
        StopCheck::new(
            "confirmed_loss_classes",
            LOCAL_CONFIRMED_LOSS_CLASSES,
            key_choices * 2,
        ),
        StopCheck::new(
            "confirmed_loss_states_of_a_class_with_in_flight_writes_landed",
            LOCAL_CONFIRMED_LOSS_STATES_OF_A_CLASS_WITH_IN_FLIGHT_WRITES_LANDED,
            states_of_a_landed_class,
        ),
        StopCheck::new(
            "confirmed_loss_judged_states",
            LOCAL_CONFIRMED_LOSS_JUDGED_STATES,
            partition.judged_state_count(&digits),
        ),
        // 各类态数之和（「C 一个没落」各 1 态加「C 至少落一个」各那么多态）等于全枚举态数：写成加法（第七节 A10）。
        StopCheck::new(
            "confirmed_loss_class_states_sum_to_the_states",
            LOCAL_CONFIRMED_LOSS_STATES,
            key_choices * (1 + states_of_a_landed_class),
        ),
        StopCheck::new(
            "confirmed_loss_removed_barriers",
            LOCAL_CONFIRMED_LOSS_REMOVED_BARRIERS.to_vec(),
            shape.removed_barriers.clone(),
        ),
        StopCheck::new(
            "confirmed_loss_root_write_made_plain",
            vec![LOCAL_CONFIRMED_LOSS_ROOT_WRITE],
            shape.root_writes_made_plain.clone(),
        ),
        StopCheck::new(
            "confirmed_loss_truncation",
            LOCAL_CONFIRMED_LOSS_TRUNCATION,
            shape.truncation,
        ),
        StopCheck::new(
            "confirmed_loss_reverse_truncation",
            LOCAL_CONFIRMED_LOSS_REVERSE_TRUNCATION,
            shape.reverse_truncation,
        ),
        StopCheck::new(
            "confirmed_loss_last_segment_writes",
            LOCAL_CONFIRMED_LOSS_LAST_SEGMENT_WRITES,
            writes,
        ),
        StopCheck::new(
            "confirmed_loss_last_segment_tearable_writes",
            LOCAL_CONFIRMED_LOSS_LAST_SEGMENT_TEARABLE_WRITES,
            tearable,
        ),
        StopCheck::new(
            "confirmed_loss_states",
            LOCAL_CONFIRMED_LOSS_STATES,
            product,
        ),
        StopCheck::new(
            "confirmed_loss_states_equal_the_last_segment_and_the_fully_persisted_state",
            product,
            tables
                .state_ranges_by_segment
                .last()
                .map_or(0, |range| range.end - range.start)
                + 1,
        ),
    ]
}

/// `confirmed-loss-form` 模式：造那一形、每态算 O1 三个子列与附带的 O2，导出中性格式（`<输出>/confirmed_loss_form/<流二目录名>/`），
/// 打 Q5 的计数（读法 7、第六节 Q5-有效 / Q5-判 / Q5-阴性、第八节轨迹）。方向相反的取样点末段全枚举；主取样点按等价类判
/// （登记修订 2 第 2.1 节候选二、修订 3）：先过「等价类对拍」与阳性对照「判态路 panic 单列」，再按类判主取样点，类核对 ①–④ 或对拍不过走停机 S13。
/// 次序照修订 5 第 5.3 节：S1 → 造主取样点（或反向取样点）那一形 → 造对拍那一形 → [`judge_the_confirmed_loss_shapes`]（先看两形的根写）。
fn run_confirmed_loss_form(command_line: &ConfirmedLossCommandLine) -> i32 {
    let mut lines = ResultLines {
        emitted: 0,
        collected: None,
    };
    let truncation = if command_line.truncate_at_the_return_point {
        ConfirmedLossTruncation::AtTheConfirmedReturnPoint
    } else {
        ConfirmedLossTruncation::BeforeTheInFlightBarrier
    };
    let sampling_point = if command_line.truncate_at_the_return_point {
        "reverse_at_the_return_point_of_call_2"
    } else {
        "main_before_the_first_barrier_of_call_3"
    };
    lines.emit_result_line(&format!(
        "name=config segment=s1 mode=confirmed-loss-form sampling_point={sampling_point}"
    ));
    let mut failed_checks = 0usize;
    for check in confirmed_loss_form_checks_against_the_crates() {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream=confirmed_loss_form check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let started = Instant::now();
    let live = second_stream_through_the_first_overwrite(RunSwitches::default());
    let (prepared, shape) = confirmed_loss_prepared_stream(live, truncation);
    let drive_nanoseconds = started.elapsed().as_nanos();
    // 方向相反的取样点没有对拍那一形（修订 5 第 5.3 节：反向取样点只在导出之前先判它自己那一形）。
    let pairing = (!command_line.truncate_at_the_return_point).then(|| {
        confirmed_loss_prepared_stream(
            second_stream_through_the_first_overwrite(RunSwitches::default()),
            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
                EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
            ),
        )
        .0
    });
    judge_the_confirmed_loss_shapes(
        &mut lines,
        command_line,
        ConfirmedLossShapes {
            sampling_point,
            main: prepared,
            shape,
            drive_nanoseconds,
            pairing,
            // 跑法里三格一律空、命令行不开放（登记修订 6 第 6.2 节第 3 条；修订 5 第 5.1 节「注入只经装置内部的开关进」）。
            piece_panic_injections: ConfirmedLossPiecePanicInjections {
                pairing: None,
                piece_panic_control: None,
                sampling_point_export: None,
            },
        },
        failed_checks,
    )
}

/// Q5 那一次判态要的两形：主取样点（或反向取样点）那一形与它的造法下标、驱动它的耗时，与对拍那一形（反向取样点没有）；
/// 另带判态路 panic 的三格注入（跑法里都空）。
struct ConfirmedLossShapes {
    sampling_point: &'static str,
    main: PreparedStream,
    shape: ConfirmedLossShape,
    drive_nanoseconds: u128,
    pairing: Option<PreparedStream>,
    piece_panic_injections: ConfirmedLossPiecePanicInjections,
}

/// Q5 判决路的注入入口（登记修订 6 第 6.2 节第 3 条；判决 b29 第二节 L1 末行、攻方改法 A1、A2）：
/// [`judge_the_confirmed_loss_shapes`] 里收 unattributed 那四个调用点前面的三处判态各一格，可空。
/// 只有内联用例 U13–U16 交非空的，跑法（[`run_confirmed_loss_form`]）一律交三格空。
struct ConfirmedLossPiecePanicInjections {
    /// 对拍那一遍：空时与 [`equivalence_pairing_of`] 同义。
    pairing: Option<PiecePanicInjection>,
    /// 阳性对照「判态路 panic 单列」那一遍：空时照 [`piece_panic_control_injection`] 造；不空时**替换**内置的那三条规则。
    piece_panic_control: Option<PiecePanicInjection>,
    /// 主取样点或反向取样点那一次导出（[`export_prepared_stream`] 交给判态的 [`ObservationContext`]）。
    sampling_point_export: Option<PiecePanicInjection>,
}

/// 两形里头一个没有可判的根槽写的（`judged_root_index` 是 `usize::MAX`）：`main` 或 `pairing`（修订 5 第 5.3 节）；都有交 `None`。
fn shape_without_a_root_write(shapes: &ConfirmedLossShapes) -> Option<&'static str> {
    if shapes.main.judged_root_index == usize::MAX {
        return Some("main");
    }
    if shapes
        .pairing
        .as_ref()
        .is_some_and(|pairing| pairing.judged_root_index == usize::MAX)
    {
        return Some("pairing");
    }
    None
}

/// `run_confirmed_loss_form` 判完 S1 之后的那一段：先看两形的根写（修订 5 第 5.3 节），再对拍、阳性对照「判态路 panic 单列」、
/// 主取样点的分类与判态（反向取样点全枚举）；结果行打进 `lines`，交回退出码（0 或 3）。
fn judge_the_confirmed_loss_shapes(
    lines: &mut ResultLines,
    command_line: &ConfirmedLossCommandLine,
    shapes: ConfirmedLossShapes,
    mut failed_checks: usize,
) -> i32 {
    let sampling_point = shapes.sampling_point;
    if let Some(shape_name) = shape_without_a_root_write(&shapes) {
        // 那一形截出来的写流里没有可判的根槽写：对拍、阳性对照与类核对都不跑，不是 S13；Q5 记「不够判」、不进判定。
        lines.emit_result_line(&format!(
            "name=q5 sampling_point={sampling_point} not_run=no_root_write shape={shape_name}"
        ));
        lines.emit_result_line(&format!(
            "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=no_root_write piece_panic_states=- unattributed_panic_states=0"
        ));
        lines.finish();
        return 3;
    }
    if let Some(pairing_stream) = &shapes.pairing {
        // 装置先过「等价类对拍」再跑主取样点（修订 2 第 2.1 节「定之前与定之后」末句）。
        let pairing = equivalence_pairing_with_injected_piece_panics(
            pairing_stream,
            shapes.piece_panic_injections.pairing.as_ref(),
        );
        let in_flight_writes_kept = call_index_of_each_write(pairing_stream)
            .iter()
            .filter(|call| **call == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL))
            .count();
        lines.emit_result_line(&format!(
            "name=q5_equivalence_pairing in_flight_writes_kept={in_flight_writes_kept} states={} product_of_the_choices={} class_states={} classes={} mismatching_states={} crash_judge_not_applicable_states={} {} s13={}",
            pairing.states,
            pairing.product_of_the_choices,
            pairing.class_states,
            pairing.classes,
            pairing.mismatching_states,
            pairing
                .observations
                .iter()
                .filter(|observation| observation.crash_judge == CrashJudgeVerdict::NotApplicable)
                .count(),
            equivalence_check_fields(&pairing.classification, &pairing.checks),
            pairing.stops_at_s13()
        ));
        lines.emit_result_line(&format!(
            "name=q5_partition sampling_point=pairing {}",
            partition_fields(&pairing.classification, &pairing.checks)
        ));
        lines.emit_result_line(&q5_piece_panics_line("pairing", &pairing.observations));
        // S1（修订 5 第 5.2 节）：对拍那一行打的实际截的 k 与分法里的 C 写数相等。
        let kept_check = StopCheck::new(
            "equivalence_pairing_in_flight_writes_kept_equal_the_partition",
            in_flight_writes_kept,
            pairing.classification.in_flight_writes,
        );
        failed_checks += usize::from(!kept_check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream=confirmed_loss_form check={} local={} crates={} holds={}",
            kept_check.name,
            field_text(&kept_check.local),
            field_text(&kept_check.crates),
            kept_check.holds()
        ));
        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
            lines,
            failed_checks,
            &pairing.observations,
        ) {
            return exit_code;
        }
        if pairing.stops_at_s13() {
            lines.emit_result_line(&format!(
                "name=verdict s1_failed_checks={failed_checks} s13=equivalence_pairing q5_not_run=equivalence_pairing_failed piece_panic_states=- unattributed_panic_states=0"
            ));
            lines.finish();
            return 3;
        }
        let control = piece_panic_control_of(
            pairing_stream,
            &pairing,
            shapes.piece_panic_injections.piece_panic_control.as_ref(),
        );
        lines.emit_result_line(&format!(
            "name=q5_piece_panic_control states={} pool_checker={} ignored_recovery={} record_checker={} other_pieces={} consulted_recovery={} consulted_recovery_of_the_plain_pairing={} o1_columns_differing_from_the_plain_pairing={} mismatching_states={} s13={} holds={}",
            control.states,
            control.piece_panic_states[JudgingPiece::PoolChecker.position()],
            control.piece_panic_states[JudgingPiece::IgnoredRecovery.position()],
            control.piece_panic_states[JudgingPiece::RecordChecker.position()],
            control.other_piece_panic_states(),
            control.piece_panic_states[JudgingPiece::ConsultedRecovery.position()],
            control.consulted_recovery_panics_of_the_plain_pairing,
            control.o1_columns_differing_from_the_plain_pairing,
            control.mismatching_states,
            control.stops_at_s13,
            control.holds()
        ));
        lines.emit_result_line(&q5_piece_panics_line(
            "piece_panic_control",
            &control.observations,
        ));
        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
            lines,
            failed_checks,
            &control.observations,
        ) {
            return exit_code;
        }
        if !control.holds() {
            // 失败 F9（修订 5 第 5.7 节）：甲那条管子接错了，主取样点不跑、不记成「O1 盲」。
            lines.emit_result_line(&format!(
                "name=verdict s1_failed_checks={failed_checks} s13=false q5_not_run=piece_panic_control_failed piece_panic_states=- unattributed_panic_states=0"
            ));
            lines.finish();
            return 3;
        }
    }
    let started = Instant::now();
    let prepared = shapes.main;
    let shape = shapes.shape;
    let drive_nanoseconds = shapes.drive_nanoseconds;
    let confirmed_root = prepared
        .writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .map(|index| {
            singlefs_harness::memory_pool::root_identity_of_write(&prepared.writes[index])
        });
    let in_flight_returned = prepared
        .calls
        .get(CONFIRMED_LOSS_IN_FLIGHT_CALL)
        .is_some_and(|call| call.outcome != CallOutcome::InFlightAtTheTruncation);
    lines.emit_result_line(&format!(
        "name=q5_shape sampling_point={sampling_point} removed_barriers={} root_writes_made_plain={} truncation={} operations_kept={} confirmed_root={} confirmed_call_returned_before_the_truncation={} in_flight_call_returned={in_flight_returned}",
        field_text(&format!("{:?}", shape.removed_barriers)),
        field_text(&format!("{:?}", shape.root_writes_made_plain)),
        if command_line.truncate_at_the_return_point { shape.reverse_truncation } else { shape.truncation },
        prepared.operations.len(),
        root_text(confirmed_root),
        prepared.calls[CONFIRMED_LOSS_CONFIRMED_CALL].outcome == CallOutcome::Succeeded
            && prepared.calls[CONFIRMED_LOSS_CONFIRMED_CALL].return_point <= prepared.operations.len(),
    ));
    let directory = command_line
        .output_directory
        .join("confirmed_loss_form")
        .join(StreamName::FixedScriptThroughEThenUnmount.directory_name());
    let observation_tables = ObservationTables::of(&prepared);
    let (last_segment_writes, tearable, product) =
        last_segment_product(&prepared, &observation_tables.tables);
    let last_segment: Vec<usize> = prepared.segments.last().cloned().unwrap_or_default();
    let (export, totals, stops_at_s13) = if command_line.truncate_at_the_return_point {
        // 方向相反的取样点照读法 7 全枚举（修订 2 的五个候选都不改它）；态数超上限不跑。
        if product > CONFIRMED_LOSS_STATE_LIMIT {
            lines.emit_result_line(&format!(
                "name=q5 sampling_point={sampling_point} not_run=too_many_states states={product} limit={CONFIRMED_LOSS_STATE_LIMIT}"
            ));
            lines.emit_result_line(&format!(
                "name=verdict s1_failed_checks={failed_checks} q5_not_run=too_many_states piece_panic_states=- unattributed_panic_states=0"
            ));
            lines.finish();
            return 3;
        }
        let mut export = export_prepared_stream(
            &prepared,
            &directory,
            false,
            RunSwitches::default(),
            StateSelection::WholeLastSegmentAndTheFullyPersistedState,
            started,
            drive_nanoseconds,
            shapes.piece_panic_injections.sampling_point_export.as_ref(),
        );
        let s9 = s9_checks_of_a_stream(&export);
        export.checks.extend(s9);
        let totals = report_stream(lines, &export);
        lines.emit_result_line(&q5_piece_panics_line("reverse", &export.observations));
        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
            lines,
            failed_checks + totals.failed_checks,
            &export.observations,
        ) {
            return exit_code;
        }
        let rows: Vec<ConfirmedLossTallyRow<'_>> = export
            .observations
            .iter()
            .map(|observation| ConfirmedLossTallyRow::of_a_judged_state(observation, &last_segment))
            .collect();
        report_the_confirmed_loss(
            lines,
            sampling_point,
            &rows,
            confirmed_root,
            (last_segment_writes, tearable, product),
            "",
        );
        (export, totals, false)
    } else {
        let context = observation_tables.context(&prepared);
        let digits = LastSegmentDigits::of(&prepared, &observation_tables.tables);
        let classification = classify_the_last_segment(
            &context,
            &digits,
            CONFIRMED_LOSS_IN_FLIGHT_CALL,
            CONFIRMED_LOSS_STATE_LIMIT,
        );
        if classification.stopped_at_the_state_limit {
            lines.emit_result_line(&format!(
                "name=q5_equivalence sampling_point={sampling_point} {} s13={}",
                equivalence_check_fields(&classification, &EquivalenceCheckCounts::default()),
                equivalence_stops_at_s13(&classification, &EquivalenceCheckCounts::default())
            ));
            lines.emit_result_line(&format!(
                "name=q5_partition sampling_point={sampling_point} {}",
                partition_fields(&classification, &EquivalenceCheckCounts::default())
            ));
            lines.emit_result_line(&format!(
                "name=verdict s1_failed_checks={failed_checks} s13=state_limit q5_not_run=too_many_states_after_refinement piece_panic_states=- unattributed_panic_states=0"
            ));
            lines.finish();
            return 3;
        }
        let judged = classification.judged_ordinals(&digits);
        let judged_states = judged.len();
        let mut export = export_prepared_stream(
            &prepared,
            &directory,
            false,
            RunSwitches::default(),
            StateSelection::ListedStatesKeepingTheConsultedReports(judged),
            started,
            drive_nanoseconds,
            shapes.piece_panic_injections.sampling_point_export.as_ref(),
        );
        let s9 = s9_checks_of_a_stream(&export);
        export.checks.extend(s9);
        let totals = report_stream(lines, &export);
        lines.emit_result_line(&q5_piece_panics_line("main", &export.observations));
        if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
            lines,
            failed_checks + totals.failed_checks,
            &export.observations,
        ) {
            return exit_code;
        }
        if let Some(why) = &export.not_exported {
            // 第二道（第一道是修订 5 第 5.3 节的 [`shape_without_a_root_write`]）：这一形截出来的写流里没有可判的根槽写
            // （`export_prepared_stream` 记 `no_root_write_to_judge`）：类核对没走到，不是 S13；记 Q5 没跑、不进判定
            // （登记「主 agent 认定：25-3a 装置返工交回」第 6 条）。
            lines.emit_result_line(&format!(
                "name=q5 sampling_point={sampling_point} not_run=no_root_write not_exported_why={}",
                field_text(why)
            ));
            lines.emit_result_line(&format!(
                "name=verdict s1_failed_checks={} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states=0 s13=false q5_not_run=no_root_write piece_panic_states={} unattributed_panic_states={}",
                failed_checks + totals.failed_checks,
                totals.rebuilt_digest_mismatches,
                totals.s3_mismatches,
                piece_panic_states_text(&totals.piece_panic_states),
                totals.unattributed_panic_states
            ));
            lines.finish();
            return 3;
        } else {
            let evaluation = evaluate_the_classes(
                &context,
                &digits,
                &classification,
                &export.observations,
                CONFIRMED_LOSS_IN_FLIGHT_CALL,
            );
            lines.emit_result_line(&format!(
                "name=q5_equivalence sampling_point={sampling_point} {} s13={}",
                equivalence_check_fields(&classification, &evaluation.checks),
                equivalence_stops_at_s13(&classification, &evaluation.checks)
            ));
            lines.emit_result_line(&format!(
                "name=q5_partition sampling_point={sampling_point} {}",
                partition_fields(&classification, &evaluation.checks)
            ));
            for class in evaluation.classes.values() {
                let lost = confirmed_root
                    .and_then(|confirmed| {
                        lost_the_confirmed_version(class.consulted_root, confirmed)
                    })
                    .map_or_else(|| "none".to_string(), |lost| lost.to_string());
                lines.emit_result_line(&format!(
                    "name=q5_class sampling_point={sampling_point} keyed_landing={} in_flight_writes={} states={} consulted_root={} lost_the_confirmed_version={lost} crash_judge={} reference_filesystem={}",
                    class.keyed_landing,
                    class.key.in_flight_writes_landed.word(),
                    class.states,
                    root_text(class.consulted_root),
                    class.columns.crash_judge.word(),
                    class.columns.reference_filesystem.word()
                ));
            }
            let rows: Vec<ConfirmedLossTallyRow<'_>> = evaluation
                .classes
                .values()
                .map(ConfirmedLossTallyRow::of_a_class)
                .collect();
            report_the_confirmed_loss(
                lines,
                sampling_point,
                &rows,
                confirmed_root,
                (last_segment_writes, tearable, product),
                &format!(" derived_by=equivalence_classes judged_states={judged_states}"),
            );
            let stops = equivalence_stops_at_s13(&classification, &evaluation.checks);
            (export, totals, stops)
        }
    };
    failed_checks += totals.failed_checks;
    lines.emit_result_line(&format!(
        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={} s3_o2_judgement_mismatches={} states={} s13={stops_at_s13} piece_panic_states={} unattributed_panic_states={}",
        totals.rebuilt_digest_mismatches,
        totals.s3_mismatches,
        export.observations.len(),
        piece_panic_states_text(&totals.piece_panic_states),
        totals.unattributed_panic_states
    ));
    lines.finish();
    if failed_checks > 0
        || totals.rebuilt_digest_mismatches > 0
        || totals.s3_mismatches > 0
        || stops_at_s13
    {
        3
    } else {
        0
    }
}

/// 这一态恢复落到的版本早于第 2 次调用写的那一版（按 (checkpoint_txg, 实例代号) 比，读法 7「已确认的丢了」）；恢复没落到任何版本的交 `None`。
fn lost_the_confirmed_version(
    consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
    confirmed_root: (InstanceGeneration, CheckpointTxg),
) -> Option<bool> {
    consulted_root.map(|(instance, checkpoint_txg)| {
        (checkpoint_txg, instance) < (confirmed_root.1, confirmed_root.0)
    })
}

/// Q5 计数的一行：全枚举时是一个判过的态，按等价类判时是一类。恢复落到的版本、两个崩溃子列，与这一行的态按「末段里落了几个写」
/// 排开的态数（键是落了几个写，值是态数；全枚举的一态只有一格、值是 1）。
struct ConfirmedLossTallyRow<'row> {
    /// 这一行罩几态（全枚举的一态是 1，等价类的一类是这一类的态数）。
    states: u64,
    consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
    crash_judge: &'row CrashJudgeVerdict,
    reference_filesystem: &'row ReferenceFileSystemVoteOfAState,
    states_by_writes_landed_in_the_last_segment: BTreeMap<usize, u64>,
}

impl<'row> ConfirmedLossTallyRow<'row> {
    fn of_a_judged_state(observation: &'row StateObservation, last_segment: &[usize]) -> Self {
        let landed = last_segment
            .iter()
            .filter(|write_index| {
                observation
                    .landing_vector
                    .as_bytes()
                    .get(**write_index)
                    .is_some_and(|landing| *landing != b'0')
            })
            .count();
        Self {
            states: 1,
            consulted_root: observation.consulted_root,
            crash_judge: &observation.crash_judge,
            reference_filesystem: &observation.reference_filesystem,
            states_by_writes_landed_in_the_last_segment: BTreeMap::from([(landed, 1)]),
        }
    }

    fn of_a_class(class: &'row ConfirmedLossClass) -> Self {
        Self {
            states: class.states,
            consulted_root: class.consulted_root,
            crash_judge: &class.columns.crash_judge,
            reference_filesystem: &class.columns.reference_filesystem,
            states_by_writes_landed_in_the_last_segment: class
                .states_by_writes_landed_in_the_last_segment
                .clone(),
        }
    }
}

/// Q5 的计数行（第六节 Q5-有效、Q5-判、Q5-阴性；第七节 A10；第八节轨迹：按末段里落了几个写排开；修订 5 第 5.1 节第 7 条另加两列各自
/// `judge_panic` 的态数）。每一行按它罩的态数计
/// （全枚举的一态计 1，等价类的一类计这一类的态数）；`derived_by` 接在 `name=q5` 那一行末尾（按等价类判时写 `derived_by=…`）。
fn report_the_confirmed_loss(
    lines: &mut ResultLines,
    sampling_point: &str,
    rows: &[ConfirmedLossTallyRow<'_>],
    confirmed_root: Option<(InstanceGeneration, CheckpointTxg)>,
    (last_segment_writes, tearable, product): (usize, usize, u64),
    derived_by: &str,
) {
    let Some(confirmed_root) = confirmed_root else {
        lines.emit_result_line(&format!(
            "name=q5 sampling_point={sampling_point} confirmed_root=none{derived_by}"
        ));
        return;
    };
    let lost: Vec<&ConfirmedLossTallyRow<'_>> = rows
        .iter()
        .filter(|row| lost_the_confirmed_version(row.consulted_root, confirmed_root) == Some(true))
        .collect();
    let not_lost: Vec<&ConfirmedLossTallyRow<'_>> = rows
        .iter()
        .filter(|row| lost_the_confirmed_version(row.consulted_root, confirmed_root) == Some(false))
        .collect();
    fn states_of(
        selected: &[&ConfirmedLossTallyRow<'_>],
        predicate: &dyn Fn(&ConfirmedLossTallyRow<'_>) -> bool,
    ) -> u64 {
        selected
            .iter()
            .filter(|row| predicate(row))
            .map(|row| row.states)
            .sum()
    }
    fn reference_filesystem_red(row: &ConfirmedLossTallyRow<'_>) -> bool {
        row.reference_filesystem.word() == "red"
    }
    fn model_red(row: &ConfirmedLossTallyRow<'_>) -> bool {
        row.crash_judge.word() == "red"
    }
    fn both_green(row: &ConfirmedLossTallyRow<'_>) -> bool {
        row.reference_filesystem.word() == "green" && row.crash_judge.word() == "green"
    }
    fn every_row(_: &ConfirmedLossTallyRow<'_>) -> bool {
        true
    }
    let states: u64 = rows.iter().map(|row| row.states).sum();
    let without_a_root: u64 = rows
        .iter()
        .filter(|row| row.consulted_root.is_none())
        .map(|row| row.states)
        .sum();
    lines.emit_result_line(&format!(
        "name=q5 sampling_point={sampling_point} states={states} last_segment_writes={last_segment_writes} last_segment_tearable_writes={tearable} product_of_the_choices={product} a10_states_equal_the_product={} lost_the_confirmed_version={} reference_filesystem_red={} model_crash_judge_red={} both_green={} reference_filesystem_abstained={} reference_filesystem_judge_panic={} model_crash_judge_judge_panic={} not_lost_states={} not_lost_o1_red={} states_without_a_root={without_a_root}{derived_by}",
        states == product,
        states_of(&lost, &every_row),
        states_of(&lost, &reference_filesystem_red),
        states_of(&lost, &model_red),
        states_of(&lost, &both_green),
        states_of(&lost, &|row| row.reference_filesystem.word() == "abstained"),
        states_of(&lost, &|row| row.reference_filesystem.word() == "judge_panic"),
        states_of(&lost, &|row| row.crash_judge.word() == "judge_panic"),
        states_of(&not_lost, &every_row),
        states_of(&not_lost, &|row| reference_filesystem_red(row) || model_red(row)),
    ));
    let mut by_landed: BTreeMap<usize, (u64, u64, u64)> = BTreeMap::new();
    for row in &lost {
        for (landed, landed_states) in &row.states_by_writes_landed_in_the_last_segment {
            let entry = by_landed.entry(*landed).or_insert((0, 0, 0));
            entry.0 += landed_states;
            if reference_filesystem_red(row) {
                entry.1 += landed_states;
            }
            if model_red(row) {
                entry.2 += landed_states;
            }
        }
    }
    for (landed, (lost_states, reference_filesystem_red_states, model_red_states)) in by_landed {
        lines.emit_result_line(&format!(
            "name=q5_trajectory sampling_point={sampling_point} writes_landed_in_the_last_segment={landed} lost_states={lost_states} reference_filesystem_red={reference_filesystem_red_states} model_crash_judge_red={model_red_states}"
        ));
    }
}

// ───── Q5 主取样点按等价类判（登记修订 2 第 2.1 节候选二；修订 3 定它） ─────
//
// 态集合与读法 7 原句相同（末段整写子集全枚举、可撕裂的写另加撕裂态）；每态的 O1 两个崩溃子列与 RefFS 一票不逐态算，由等价类的代表推出。
// 两态等价 ⇔ 键里那几个写（第 2 次调用的写 K，加上类核对 ③ 细分移进来的第 3 次调用的写）取法相同，且留在类里的第 3 次调用的写（C）
// 「至少落了一个」这一位相同。代表是「C 一个都没落」那一态；「C 至少落一个」那一类按代表推，另把「C 全落」那一态照原路径判一遍对照（类核对 ④）。

/// 「等价类对拍」那一形截到第 3 次调用发出的前几个写之后：k = 6 = C 的前三对镜像写，72 × 2⁶ = 4 608 态（修订 5 第 5.2 节丙；
/// 修订 2 第 2.1 节候选二原来是 k = 2、72 × 4 = 288 态，只罩一个单元的两份镜像）。
const EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT: usize = 6;

/// [`ObservationContext`] 借的几张表，由一条流现算（与 [`export_prepared_stream`] 里同一套算法；按等价类判与「等价类对拍」用）。
struct ObservationTables {
    tables: CrashStateEnumerationTables,
    facts: StateMasks,
    torn_image_index_by_write: BTreeMap<usize, usize>,
    reference_filesystem: ReferenceFileSystemStream,
}

impl ObservationTables {
    fn of(stream: &PreparedStream) -> Self {
        let tables = crash_state_enumeration_tables(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        );
        let facts = StateMasks::of(&tables, &stream.segments);
        let torn_image_index_by_write = tables
            .torn_images
            .iter()
            .map(|row| (row.write_index, row.torn_image_index))
            .collect();
        let reference_filesystem = reference_filesystem_stream_of(
            &stream.calls,
            &stream.operations,
            &stream.writes,
            &stream.stream_indexes,
        );
        Self {
            tables,
            facts,
            torn_image_index_by_write,
            reference_filesystem,
        }
    }

    fn context<'tables>(
        &'tables self,
        stream: &'tables PreparedStream,
    ) -> ObservationContext<'tables> {
        ObservationContext {
            stream,
            tables: &self.tables,
            facts: &self.facts,
            torn_image_index_by_write: &self.torn_image_index_by_write,
            reference_filesystem: &self.reference_filesystem,
            call_of_write: call_index_of_each_write(stream),
            piece_panic_injection: None,
        }
    }
}

/// 末段的写按崩溃点重放的混合进制排开（`crash.rs` 的 `CrashStatePlan::assign_landings`）：段内序号 = Σ 数字 × 权，末段第一个写是最低位，
/// 进制是它能取几态（整写 2、可撕裂的 3），数字 0 是「没落」、最大是「持久」（可撕裂的写 1 是撕裂）。末段之后没有段，
/// 段内序号取满（每位取最大）就是最后那个全持久态，所以末段与全持久态合起来恰是 `first_ordinal..first_ordinal + 组合数`。
struct LastSegmentDigits {
    first_ordinal: u64,
    write_indexes: Vec<usize>,
    radixes: Vec<u64>,
    weights: Vec<u64>,
}

impl LastSegmentDigits {
    fn of(stream: &PreparedStream, tables: &CrashStateEnumerationTables) -> Self {
        let write_indexes = stream.segments.last().cloned().unwrap_or_default();
        let first_ordinal = tables
            .state_ranges_by_segment
            .last()
            .map_or(0, |state_range| state_range.start);
        let radixes: Vec<u64> = write_indexes
            .iter()
            .map(|write_index| {
                if tables.is_tearable_by_write[*write_index] {
                    3
                } else {
                    2
                }
            })
            .collect();
        let mut weights = Vec::with_capacity(radixes.len());
        let mut weight = 1u64;
        for radix in &radixes {
            weights.push(weight);
            weight = weight
                .checked_mul(*radix)
                .expect("末段的组合数装得进 u64（崩溃点重放的计划同样要求）");
        }
        assert_eq!(
            first_ordinal + weight,
            tables.state_count,
            "末段是最后一段：末段的组合数加上它的起点，恰是整条流的态数（最后那个全持久态是段内序号取满的那一个）"
        );
        Self {
            first_ordinal,
            write_indexes,
            radixes,
            weights,
        }
    }

    /// 末段第 `position` 个写在 `ordinal` 那一态上的数字。
    fn digit_of(&self, ordinal: u64, position: usize) -> u64 {
        ((ordinal - self.first_ordinal) / self.weights[position]) % self.radixes[position]
    }

    /// 这几个位置上的写各取几态、全部组合的个数。
    fn product_of_the_radixes(&self, positions: &[usize]) -> u64 {
        positions
            .iter()
            .try_fold(1u64, |product, position| {
                product.checked_mul(self.radixes[*position])
            })
            .expect("末段的组合数装得进 u64")
    }
}

/// 留在类里的第 3 次调用的写（C）在一态上落没落。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum InFlightWritesLanded {
    NoneLanded,
    AtLeastOneLanded,
}

impl InFlightWritesLanded {
    fn word(self) -> &'static str {
        match self {
            Self::NoneLanded => "none",
            Self::AtLeastOneLanded => "at_least_one",
        }
    }
}

/// 一态落在哪一类：键里那几个写的取法（按键里的位置排的混合进制序号）与 C 落没落。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ConfirmedLossClassKey {
    keyed_assignment: u64,
    in_flight_writes_landed: InFlightWritesLanded,
}

/// 等价关系的分法：键里的写（K 加上细分移进来的 C 写）与留在类里的 C 写，都是末段里的位置（[`LastSegmentDigits`] 的下标），升序。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ConfirmedLossPartition {
    keyed_positions: Vec<usize>,
    class_positions: Vec<usize>,
}

impl ConfirmedLossPartition {
    /// 细分之前的起点分法：键 = 末段里不属于 `in_flight_call` 的写，类里 = 属于它的写（`call_of_write` 是 [`call_index_of_each_write`] 的交回）。
    fn before_any_refinement(
        call_of_write: &[Option<usize>],
        digits: &LastSegmentDigits,
        in_flight_call: usize,
    ) -> Self {
        let (class_positions, keyed_positions): (Vec<usize>, Vec<usize>) =
            (0..digits.write_indexes.len()).partition(|position| {
                call_of_write[digits.write_indexes[*position]] == Some(in_flight_call)
            });
        Self {
            keyed_positions,
            class_positions,
        }
    }

    /// 键里那几个写有几种取法（每种取法两类：C 一个没落、C 至少落一个；C 一个都不剩时只有前一类）。
    fn keyed_assignments(&self, digits: &LastSegmentDigits) -> u64 {
        digits.product_of_the_radixes(&self.keyed_positions)
    }

    /// 键里第 `keyed_assignment` 种取法、C 一个都没落的那一态（这一种取法的代表）。
    fn representative_ordinal(&self, digits: &LastSegmentDigits, keyed_assignment: u64) -> u64 {
        let mut remaining = keyed_assignment;
        let mut ordinal = digits.first_ordinal;
        for position in &self.keyed_positions {
            ordinal += (remaining % digits.radixes[*position]) * digits.weights[*position];
            remaining /= digits.radixes[*position];
        }
        assert_eq!(
            remaining, 0,
            "取法序号 {keyed_assignment} 超出键里那几个写的组合数"
        );
        ordinal
    }

    /// 同一种取法、C 全落的那一态。
    fn fully_landed_ordinal(&self, digits: &LastSegmentDigits, keyed_assignment: u64) -> u64 {
        self.representative_ordinal(digits, keyed_assignment)
            + self
                .class_positions
                .iter()
                .map(|position| (digits.radixes[*position] - 1) * digits.weights[*position])
                .sum::<u64>()
    }

    /// 一态落在哪一类。
    fn class_of_ordinal(&self, digits: &LastSegmentDigits, ordinal: u64) -> ConfirmedLossClassKey {
        let mut keyed_assignment = 0u64;
        let mut place = 1u64;
        for position in &self.keyed_positions {
            keyed_assignment += digits.digit_of(ordinal, *position) * place;
            place *= digits.radixes[*position];
        }
        let in_flight_writes_landed = if self
            .class_positions
            .iter()
            .any(|position| digits.digit_of(ordinal, *position) != 0)
        {
            InFlightWritesLanded::AtLeastOneLanded
        } else {
            InFlightWritesLanded::NoneLanded
        };
        ConfirmedLossClassKey {
            keyed_assignment,
            in_flight_writes_landed,
        }
    }

    /// 「C 至少落一个」那一类的态数：C 每个写的取法数之积，减去一个都没落的那一态（C 都只取两态时是 2 的 C 写数次方减一）。
    fn states_with_the_in_flight_writes_landed(&self, digits: &LastSegmentDigits) -> u64 {
        digits.product_of_the_radixes(&self.class_positions) - 1
    }

    /// 「C 至少落一个」那一类按落了几个 C 写排开的态数（下标 j 是落了几个，第 0 格是 0）：Π (1 + (进制 − 1)·x) 的各项系数去掉常数项，
    /// C 都只取两态时第 j 格是 C(C 写数, j)。撕裂算落了（与 `q5_trajectory` 按落地串数「不是 0」同一口径）。
    fn states_by_in_flight_writes_landed(&self, digits: &LastSegmentDigits) -> Vec<u64> {
        let mut coefficients = vec![1u64];
        for position in &self.class_positions {
            let landed_choices = digits.radixes[*position] - 1;
            let mut next = vec![0u64; coefficients.len() + 1];
            for (landed, states) in coefficients.iter().enumerate() {
                next[landed] += states;
                next[landed + 1] += states * landed_choices;
            }
            coefficients = next;
        }
        coefficients[0] = 0;
        coefficients
    }

    /// 要照原路径判的态数：每种取法的代表，加上 C 还剩写时同一取法的「C 全落」那一态。
    fn judged_state_count(&self, digits: &LastSegmentDigits) -> u64 {
        let per_assignment = if self.class_positions.is_empty() {
            1
        } else {
            2
        };
        self.keyed_assignments(digits)
            .checked_mul(per_assignment)
            .expect("要判的态数装得进 u64")
    }
}

/// 恢复读过的一段字节（盘、起点、终点，都按字节；终点不含），与这一段是从哪来的。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecordedReadRange {
    device: DeviceIdentity,
    start_in_bytes: u64,
    end_in_bytes: u64,
    source: RecordedReadSource,
}

/// 一段记读从哪来：一次 `read`，或 journal 位置提示记的整个环（修订 5 第 5.4 节 U3：类核对 ③ 照旧用全部段，用例分得出提示那一段）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordedReadSource {
    Read,
    JournalRecordOffsetsHint,
}

/// 类核对 ③ 的读者：包在崩溃镜像外面、只转发、只记（做法照 `e161_crash_state_dedup_and_time_split.rs` 的 `RecordingReader`，
/// 本装置另写一份、不引它）。答案一律原样交回。每一次读记它的字节段；journal 位置提示的答由环里落了的写定，记成读了整个环
/// （宽了只会多细分，不会漏）；盘身份与盘的字节数只看基镜像，与哪几个写落了无关，不记。
struct ReadRecordingPoolReader<'reader, 'image> {
    image: &'reader CrashImage<'image>,
    read_ranges: RefCell<Vec<RecordedReadRange>>,
}

impl PoolReader for ReadRecordingPoolReader<'_, '_> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        PoolReader::device_identities(self.image)
    }

    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        PoolReader::device_size_in_bytes(self.image, device)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        self.read_ranges.borrow_mut().push(RecordedReadRange {
            device,
            start_in_bytes: offset.0,
            end_in_bytes: offset.0 + u64::try_from(length).expect("读长装得进 u64"),
            source: RecordedReadSource::Read,
        });
        PoolReader::read(self.image, device, offset, length)
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.read_ranges.borrow_mut().push(RecordedReadRange {
            device,
            start_in_bytes: ring_start.0,
            end_in_bytes: ring_start.0 + ring_bytes,
            source: RecordedReadSource::JournalRecordOffsetsHint,
        });
        PoolReader::journal_record_offsets_hint(self.image, device, ring_start, ring_bytes)
    }
}

/// 看 journal 那一遍恢复在一态上的结局：交回了报告，或 panic 了（panic 的消息）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum ConsultedRecovery {
    Reported(Box<RecoveryReport>),
    Panicked(String),
}

impl ConsultedRecovery {
    /// 一个判过的态上记下的那一遍（只在留了整份报告的那一趟有）：恢复 panic 的态是 `Panicked`；没留报告的交 `None`。
    fn of_the_observation(observation: &StateObservation) -> Option<Self> {
        match (&observation.consulted_report, &observation.recovery_panic) {
            (Some(report), None) => Some(Self::Reported(Box::new(report.clone()))),
            (None, Some(message)) => Some(Self::Panicked(message.clone())),
            (None, None) | (Some(_), Some(_)) => None,
        }
    }

    fn effective_root(&self) -> Option<(InstanceGeneration, CheckpointTxg)> {
        match self {
            Self::Reported(report) => report.effective_root,
            Self::Panicked(_) => None,
        }
    }
}

/// 在一态的崩溃镜像上经 [`ReadRecordingPoolReader`] 跑看 journal 的那一遍恢复：交回恢复的结局与它读过的字节段
/// （panic 时读段记到 panic 那一刻为止：同样的读答同样的字节，恢复在同一处 panic）。
fn consulted_recovery_recording_the_reads(
    image: &CrashImage<'_>,
) -> (ConsultedRecovery, Vec<RecordedReadRange>) {
    let reader = ReadRecordingPoolReader {
        image,
        read_ranges: RefCell::new(Vec::new()),
    };
    let recovered = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        singlefs_core::recovery::recover(&reader, singlefs_core::recovery::JournalPolicy::Consult)
    }));
    let recovery = match recovered {
        Ok(report) => ConsultedRecovery::Reported(Box::new(report)),
        Err(payload) => ConsultedRecovery::Panicked(panic_text(&*payload)),
    };
    (recovery, reader.read_ranges.into_inner())
}

/// 读过的字节段与一个写的字节段（同一块盘上）相交。
fn recorded_reads_overlap_the_write(
    read_ranges: &[RecordedReadRange],
    write: &RetainedWrite,
) -> bool {
    let write_start = write.offset.0;
    let write_end = write_start + write.length_in_bytes();
    read_ranges.iter().any(|read| {
        read.device == write.device
            && read.start_in_bytes < write_end
            && write_start < read.end_in_bytes
    })
}

/// 类核对 ③ 细分到不动点之后的分法，与最后一遍在每个代表上的恢复（键是代表的序号）。
struct ConfirmedLossClassification {
    partition: ConfirmedLossPartition,
    /// 原来的 C 写数（末段里属于第 3 次调用的写）。
    in_flight_writes: usize,
    refinements: usize,
    /// 细分移进键的 C 写，末段里的位置。
    moved_positions: Vec<usize>,
    representative_recoveries: BTreeMap<u64, ConsultedRecovery>,
    /// 停在上限时最后一遍还与读段相交的 C 写个数（到了不动点是 0）。
    overlapping_positions_left: usize,
    /// 细分之后要判的态数超过上限（停机 S13）。
    stopped_at_the_state_limit: bool,
    /// 最后那个分法要照原路径判的态数（停在上限时是超过上限的那一个分法的）。
    judged_states_planned: u64,
}

impl ConfirmedLossClassification {
    /// 要照原路径判的态：每种取法的代表，加上 C 还剩写时同一取法的「C 全落」那一态。
    fn judged_ordinals(&self, digits: &LastSegmentDigits) -> BTreeSet<u64> {
        let mut judged = BTreeSet::new();
        for keyed_assignment in 0..self.partition.keyed_assignments(digits) {
            judged.insert(
                self.partition
                    .representative_ordinal(digits, keyed_assignment),
            );
            if !self.partition.class_positions.is_empty() {
                judged.insert(
                    self.partition
                        .fully_landed_ordinal(digits, keyed_assignment),
                );
            }
        }
        judged
    }
}

/// 分类并细分（类核对 ③）：起点是键 = 末段里不属于 `in_flight_call` 的写、类里 = 属于它的写；每一遍在每种取法的代表上经只转发、只记的读者
/// 跑看 journal 的恢复，读段与类里某个写相交就把那个写移进键、重做，直到一遍里一个都不相交（不动点）。移进键之后要判的态数超过
/// `state_limit` 就停（停机 S13）。迭代上界：每一遍不是到不动点就是至少移走一个写，最多 C 写数 + 1 遍。
fn classify_the_last_segment(
    context: &ObservationContext<'_>,
    digits: &LastSegmentDigits,
    in_flight_call: usize,
    state_limit: u64,
) -> ConfirmedLossClassification {
    let stream = context.stream;
    let partition = ConfirmedLossPartition::before_any_refinement(
        &context.call_of_write,
        digits,
        in_flight_call,
    );
    let mut classification = ConfirmedLossClassification {
        in_flight_writes: partition.class_positions.len(),
        partition,
        refinements: 0,
        moved_positions: Vec::new(),
        representative_recoveries: BTreeMap::new(),
        overlapping_positions_left: 0,
        stopped_at_the_state_limit: false,
        judged_states_planned: 0,
    };
    loop {
        classification.judged_states_planned = classification.partition.judged_state_count(digits);
        if classification.judged_states_planned > state_limit {
            classification.stopped_at_the_state_limit = true;
            return classification;
        }
        let mut overlapping: BTreeSet<usize> = BTreeSet::new();
        let mut recoveries = BTreeMap::new();
        for keyed_assignment in 0..classification.partition.keyed_assignments(digits) {
            let representative = classification
                .partition
                .representative_ordinal(digits, keyed_assignment);
            let image = CrashImage {
                base: &stream.base,
                writes: &context.tables.enumerated_writes,
                persisted: context.facts.persisted_writes_of_state(representative),
            };
            let (recovery, read_ranges) = consulted_recovery_recording_the_reads(&image);
            for position in &classification.partition.class_positions {
                let write = &stream.writes[digits.write_indexes[*position]];
                if recorded_reads_overlap_the_write(&read_ranges, write) {
                    overlapping.insert(*position);
                }
            }
            recoveries.insert(representative, recovery);
        }
        classification.representative_recoveries = recoveries;
        if overlapping.is_empty() {
            return classification;
        }
        let mut refined = classification.partition.clone();
        refined
            .class_positions
            .retain(|position| !overlapping.contains(position));
        refined.keyed_positions.extend(overlapping.iter().copied());
        refined.keyed_positions.sort_unstable();
        if refined.judged_state_count(digits) > state_limit {
            classification.judged_states_planned = refined.judged_state_count(digits);
            classification.overlapping_positions_left = overlapping.len();
            classification.stopped_at_the_state_limit = true;
            return classification;
        }
        classification.partition = refined;
        classification.moved_positions.extend(overlapping);
        classification.moved_positions.sort_unstable();
        classification.refinements += 1;
    }
}

/// O1 两个崩溃子列与 RefFS 一票，只由修订 2 第 2.1 节「为什么这样分得开」列的四样输入算：看 journal 那一遍的恢复（读回由它出）、
/// 盘上已持久的最新根、触达的最后一次调用、段号（RefFS 的崩溃时刻）。等价类按它推；类核对 ① 拿它在判过的每一态上重算，
/// 与 [`observation_of_a_judged_state`] 那一路比（那一路多吃一样输入就对不上）。恢复 panic 的两列都记恢复 panic（读法 5）。
fn crash_columns_of_the_four_inputs(
    context: &ObservationContext<'_>,
    recovery: &ConsultedRecovery,
    newest_persisted_root_key: Option<ModelRootKey>,
    touched_call: Option<usize>,
    segment_index: usize,
) -> (CrashJudgeVerdict, ReferenceFileSystemVoteOfAState) {
    match recovery {
        ConsultedRecovery::Panicked(_) => (
            CrashJudgeVerdict::RecoveryPanicked,
            ReferenceFileSystemVoteOfAState::RecoveryPanicked,
        ),
        ConsultedRecovery::Reported(report) => {
            // 判官自己 panic 那一列记 `judge_panic`（修订 5 第 5.1 节第 3 条；与逐件重做那一路同一读法），读回连同判官一起包。
            let crash_judge =
                if crash_judge_is_applicable(touched_call, context.stream.model_stopped_at_call) {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        crash_judge_verdict(
                            &context.stream.committed_versions,
                            newest_persisted_root_key,
                            &observed_read_back_after_a_crash(report),
                        )
                    }))
                    .unwrap_or(CrashJudgeVerdict::JudgePanicked)
                } else {
                    CrashJudgeVerdict::NotApplicable
                };
            let reference_filesystem =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    reference_filesystem_vote_of_a_state(
                        context.reference_filesystem,
                        context.reference_filesystem_moment_of(segment_index),
                        &observed_read_back_after_a_crash(report),
                    )
                }))
                .unwrap_or(ReferenceFileSystemVoteOfAState::JudgePanicked);
            (crash_judge, reference_filesystem)
        }
    }
}

/// 一态上「盘上已持久的最新根」（[`crash_judge_of_the_state`] 同一种数法：只看写表里的原写）。
fn newest_persisted_root_key_of_the_state(
    context: &ObservationContext<'_>,
    ordinal: u64,
) -> Option<ModelRootKey> {
    let stream = context.stream;
    let persisted = context.facts.persisted_writes_of_state(ordinal);
    newest_persisted_root(&stream.writes, &persisted[..stream.writes.len()])
        .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg))
}

/// 一类的触达调用：「C 一个没落」那一类是代表自己的；「C 至少落一个」那一类再算上类里那几个写的调用。
fn touched_call_of_the_class(
    representative_touched_call: Option<usize>,
    in_flight_writes_landed: InFlightWritesLanded,
    call_of_the_class_writes: Option<usize>,
) -> Option<usize> {
    match in_flight_writes_landed {
        InFlightWritesLanded::NoneLanded => representative_touched_call,
        InFlightWritesLanded::AtLeastOneLanded => {
            representative_touched_call.max(call_of_the_class_writes)
        }
    }
}

/// 一类的恢复与两个崩溃子列（「C 一个没落」那一类就是代表照原路径判出的；「C 至少落一个」那一类由代表推出）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ClassColumns {
    recovery: ConsultedRecovery,
    crash_judge: CrashJudgeVerdict,
    reference_filesystem: ReferenceFileSystemVoteOfAState,
}

impl ClassColumns {
    fn of_the_observation(observation: &StateObservation) -> Option<Self> {
        ConsultedRecovery::of_the_observation(observation).map(|recovery| Self {
            recovery,
            crash_judge: observation.crash_judge.clone(),
            reference_filesystem: observation.reference_filesystem.clone(),
        })
    }
}

/// 一类（`name=q5_class` 一行，Q5 计数的一行）。
struct ConfirmedLossClass {
    key: ConfirmedLossClassKey,
    /// 键里那几个写在代表上的落地串（字符同 `states.tsv`：0 没落、1 撕裂、2 持久），按末段里的位置排。
    keyed_landing: String,
    states: u64,
    states_by_writes_landed_in_the_last_segment: BTreeMap<usize, u64>,
    columns: ClassColumns,
    consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
}

/// 类核对的各项计数（`name=q5_equivalence` 一行；任一项不是 0 走停机 S13）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct EquivalenceCheckCounts {
    /// ①：判过的态上，两个崩溃子列与 RefFS 一票照原路径算的，与只由四样输入重算的对不上的态数。
    input_list_mismatches: usize,
    /// ②：C 的写经 `geometry.classify` 是根槽写、系统配置槽写或 journal 记录写的个数。
    in_flight_writes_of_a_forbidden_kind: usize,
    /// ④ 前一半：代表上经读者跑的那一遍恢复，与崩溃点重放判同一态交的 `judgement.consulted` 不同的代表数。
    recording_report_mismatches: usize,
    /// ④ 后一半：「C 全落」那一态照原路径判的恢复、两个崩溃子列与 RefFS 一票，与它所在的类推出的不同的态数。
    derived_class_mismatches: usize,
    /// 代表与「C 全落」那一态的落地串不是分法说的那样（代表上类里的写不全是 0、「C 全落」上不全是持久，或代表不在它那一类）的态数。
    class_landing_mismatches: usize,
    /// C 的写经 `geometry.classify` 各是哪一种（逐个列，按种类数）。
    in_flight_write_kinds: BTreeMap<&'static str, usize>,
}

/// 各类与类核对 ①、②、④ 的计数（③ 在 [`classify_the_last_segment`] 里）。
struct ConfirmedLossClassEvaluation {
    classes: BTreeMap<ConfirmedLossClassKey, ConfirmedLossClass>,
    checks: EquivalenceCheckCounts,
}

/// 由判过的态（每种取法的代表与「C 全落」那一态，都照原路径判、留了整份恢复报告）推出各类，并做类核对 ①、②、④。
///
/// # Panics
/// 判过的态里缺某种取法的代表或「C 全落」那一态（调用方没照 [`ConfirmedLossClassification::judged_ordinals`] 判）。
fn evaluate_the_classes(
    context: &ObservationContext<'_>,
    digits: &LastSegmentDigits,
    classification: &ConfirmedLossClassification,
    observations: &[StateObservation],
    in_flight_call: usize,
) -> ConfirmedLossClassEvaluation {
    let stream = context.stream;
    let partition = &classification.partition;
    let observation_of_ordinal: BTreeMap<u64, &StateObservation> = observations
        .iter()
        .map(|observation| (observation.ordinal, observation))
        .collect();
    let mut checks = EquivalenceCheckCounts::default();
    let geometry = common::geometry();
    for write_index in digits.write_indexes.iter().copied() {
        if context.call_of_write[write_index] != Some(in_flight_call) {
            continue;
        }
        let kind =
            geometry.classify(&stream.operations[stream.stream_indexes[write_index]].operation);
        *checks.in_flight_write_kinds.entry(kind.name()).or_insert(0) += 1;
        match kind {
            StepKind::RootRecordFua
            | StepKind::SystemConfigurationSlot
            | StepKind::JournalRecord => {
                checks.in_flight_writes_of_a_forbidden_kind += 1;
            }
            StepKind::ZeroFill | StepKind::UnitWrite | StepKind::Barrier => {}
        }
    }
    for observation in observations {
        let Some(recovery) = ConsultedRecovery::of_the_observation(observation) else {
            continue;
        };
        let recomputed = crash_columns_of_the_four_inputs(
            context,
            &recovery,
            newest_persisted_root_key_of_the_state(context, observation.ordinal),
            observation.touched_call,
            observation.segment_index,
        );
        if recomputed
            != (
                observation.crash_judge.clone(),
                observation.reference_filesystem.clone(),
            )
        {
            checks.input_list_mismatches += 1;
        }
    }
    let call_of_the_class_writes = partition
        .class_positions
        .iter()
        .filter_map(|position| context.call_of_write[digits.write_indexes[*position]])
        .max();
    let keyed_landed_and_landing = |observation: &StateObservation| -> (usize, String) {
        let keyed_landing: String = partition
            .keyed_positions
            .iter()
            .map(|position| {
                char::from(observation.landing_vector.as_bytes()[digits.write_indexes[*position]])
            })
            .collect();
        let landed = keyed_landing
            .chars()
            .filter(|landing| *landing != '0')
            .count();
        (landed, keyed_landing)
    };
    let class_landings_are = |observation: &StateObservation, expected: char| -> bool {
        partition.class_positions.iter().all(|position| {
            char::from(observation.landing_vector.as_bytes()[digits.write_indexes[*position]])
                == expected
        })
    };
    let mut classes = BTreeMap::new();
    for keyed_assignment in 0..partition.keyed_assignments(digits) {
        let representative_ordinal = partition.representative_ordinal(digits, keyed_assignment);
        let representative = observation_of_ordinal
            .get(&representative_ordinal)
            .copied()
            .expect("每种取法的代表都照原路径判过（judged_ordinals）");
        let none_key = ConfirmedLossClassKey {
            keyed_assignment,
            in_flight_writes_landed: InFlightWritesLanded::NoneLanded,
        };
        if partition.class_of_ordinal(digits, representative_ordinal) != none_key
            || !class_landings_are(representative, '0')
        {
            checks.class_landing_mismatches += 1;
        }
        let recording_recovery = classification
            .representative_recoveries
            .get(&representative_ordinal)
            .expect("类核对 ③ 的最后一遍在每个代表上都跑过");
        let representative_columns = ClassColumns::of_the_observation(representative)
            .expect("按等价类判的那一趟留了整份恢复报告");
        if *recording_recovery != representative_columns.recovery {
            checks.recording_report_mismatches += 1;
        }
        let (keyed_landed, keyed_landing) = keyed_landed_and_landing(representative);
        classes.insert(
            none_key,
            ConfirmedLossClass {
                key: none_key,
                keyed_landing: keyed_landing.clone(),
                states: 1,
                states_by_writes_landed_in_the_last_segment: BTreeMap::from([(keyed_landed, 1)]),
                consulted_root: representative.consulted_root,
                columns: representative_columns,
            },
        );
        if partition.class_positions.is_empty() {
            continue;
        }
        let in_flight_writes_landed_in_the_class = InFlightWritesLanded::AtLeastOneLanded;
        let derived_touched_call = touched_call_of_the_class(
            representative.touched_call,
            in_flight_writes_landed_in_the_class,
            call_of_the_class_writes,
        );
        let (crash_judge, reference_filesystem) = crash_columns_of_the_four_inputs(
            context,
            recording_recovery,
            newest_persisted_root_key_of_the_state(context, representative_ordinal),
            derived_touched_call,
            representative.segment_index,
        );
        let derived = ClassColumns {
            recovery: recording_recovery.clone(),
            crash_judge,
            reference_filesystem,
        };
        let fully_landed_ordinal = partition.fully_landed_ordinal(digits, keyed_assignment);
        let fully_landed = observation_of_ordinal
            .get(&fully_landed_ordinal)
            .copied()
            .expect("每种取法的「C 全落」那一态都照原路径判过（judged_ordinals）");
        let some_key = ConfirmedLossClassKey {
            keyed_assignment,
            in_flight_writes_landed: InFlightWritesLanded::AtLeastOneLanded,
        };
        let highest_digit_landing = '2';
        if partition.class_of_ordinal(digits, fully_landed_ordinal) != some_key
            || !class_landings_are(fully_landed, highest_digit_landing)
        {
            checks.class_landing_mismatches += 1;
        }
        if ClassColumns::of_the_observation(fully_landed).as_ref() != Some(&derived) {
            checks.derived_class_mismatches += 1;
        }
        let states_by_writes_landed_in_the_last_segment = partition
            .states_by_in_flight_writes_landed(digits)
            .into_iter()
            .enumerate()
            .filter(|(_, states)| *states > 0)
            .map(|(in_flight_landed, states)| (keyed_landed + in_flight_landed, states))
            .collect();
        classes.insert(
            some_key,
            ConfirmedLossClass {
                key: some_key,
                keyed_landing,
                states: partition.states_with_the_in_flight_writes_landed(digits),
                states_by_writes_landed_in_the_last_segment,
                consulted_root: derived.recovery.effective_root(),
                columns: derived,
            },
        );
    }
    ConfirmedLossClassEvaluation { classes, checks }
}

/// 类核对 ①–④ 任一不过，或细分到上限：停机 S13（修订 2 第 2.1 节）。
fn equivalence_stops_at_s13(
    classification: &ConfirmedLossClassification,
    checks: &EquivalenceCheckCounts,
) -> bool {
    classification.stopped_at_the_state_limit
        || classification.overlapping_positions_left > 0
        || checks.input_list_mismatches > 0
        || checks.in_flight_writes_of_a_forbidden_kind > 0
        || checks.recording_report_mismatches > 0
        || checks.derived_class_mismatches > 0
        || checks.class_landing_mismatches > 0
}

/// `name=q5_equivalence` 那一行（与对拍那一行）的类核对计数：①–④ 各自的计数与 ③ 停下时还相交的写数（修订 5 第 5.5 节第 2 条：
/// 这两种行只放类核对的计数与判定）。`s13=` 由调用方接在后面：主取样点那一行是 [`equivalence_stops_at_s13`]，对拍那一行是
/// [`EquivalencePairing::stops_at_s13`]（再并上逐态不同与各类态数之和不等于组合数，第 5.5 节第 3 条）。
fn equivalence_check_fields(
    classification: &ConfirmedLossClassification,
    checks: &EquivalenceCheckCounts,
) -> String {
    format!(
        "input_list_mismatches={} in_flight_writes_of_a_forbidden_kind={} read_overlaps_left={} recording_report_mismatches={} derived_class_mismatches={} class_landing_mismatches={}",
        checks.input_list_mismatches,
        checks.in_flight_writes_of_a_forbidden_kind,
        classification.overlapping_positions_left,
        checks.recording_report_mismatches,
        checks.derived_class_mismatches,
        checks.class_landing_mismatches,
    )
}

/// `name=q5_partition` 那一行（主取样点与对拍各一行）的分法字段：C 的写数与经 `geometry.classify` 逐个列的种类（类核对 ②）、键里与类里的写
/// （末段里的位置，A10 比对器按它现算各类态数）、要照原路径判的态数与上限，加修订 5 第 5.5 节第 2 条从类核对那一行挪来的细分次数、
/// 移进键的写与停没停在上限（`stopped_at_the_state_limit` 经 `s13=` 进 S13，这一行的字段不直接当触发观测）。
fn partition_fields(
    classification: &ConfirmedLossClassification,
    checks: &EquivalenceCheckCounts,
) -> String {
    let kinds: Vec<String> = checks
        .in_flight_write_kinds
        .iter()
        .map(|(kind, count)| format!("{kind}:{count}"))
        .collect();
    format!(
        "in_flight_writes={} in_flight_write_kinds={} keyed_last_segment_positions={} class_last_segment_positions={} judged_states_planned={} state_limit={CONFIRMED_LOSS_STATE_LIMIT} refinements={} moved_last_segment_positions={} stopped_at_the_state_limit={}",
        classification.in_flight_writes,
        if kinds.is_empty() { "-".to_string() } else { kinds.join(",") },
        field_text(&format!("{:?}", classification.partition.keyed_positions)),
        field_text(&format!("{:?}", classification.partition.class_positions)),
        classification.judged_states_planned,
        classification.refinements,
        field_text(&format!("{:?}", classification.moved_positions)),
        classification.stopped_at_the_state_limit
    )
}

/// 「等价类对拍」（修订 2 第 2.1 节候选二「用例钉住等价关系」，每一跑也做一遍）：末段与全持久态全枚举、逐态照原路径判，再按等价类推，
/// 逐态比恢复、两个崩溃子列与 RefFS 一票。
struct EquivalencePairing {
    states: u64,
    product_of_the_choices: u64,
    /// 各类态数之和（A10：必须等于末段每个写的取法数之积）。
    class_states: u64,
    classes: usize,
    mismatching_states: usize,
    classification: ConfirmedLossClassification,
    checks: EquivalenceCheckCounts,
    observations: Vec<StateObservation>,
}

impl EquivalencePairing {
    /// 对拍有一态不同、各类态数之和不等于组合数，或类核对不过：停机 S13。
    fn stops_at_s13(&self) -> bool {
        self.mismatching_states > 0
            || self.class_states != self.product_of_the_choices
            || equivalence_stops_at_s13(&self.classification, &self.checks)
    }
}

/// 在一条 Q5 那一形的流上做「等价类对拍」。跑法里 Q5 那一处自修订 6 起改调 [`equivalence_pairing_with_injected_piece_panics`]
/// （注入为空时同义），这个只剩内联用例在用。
#[cfg(test)]
fn equivalence_pairing_of(stream: &PreparedStream) -> EquivalencePairing {
    equivalence_pairing_with_injected_piece_panics(stream, None)
}

/// 同 [`equivalence_pairing_of`]，`piece_panic_injection` 给了的，判态时照它注入（阳性对照「判态路 panic 单列」，修订 5 第 5.1 节）。
fn equivalence_pairing_with_injected_piece_panics(
    stream: &PreparedStream,
    piece_panic_injection: Option<&PiecePanicInjection>,
) -> EquivalencePairing {
    let observation_tables = ObservationTables::of(stream);
    let mut context = observation_tables.context(stream);
    context.piece_panic_injection = piece_panic_injection;
    let context = context;
    let digits = LastSegmentDigits::of(stream, &observation_tables.tables);
    let product_of_the_choices =
        digits.product_of_the_radixes(&(0..digits.write_indexes.len()).collect::<Vec<usize>>());
    let classification = classify_the_last_segment(
        &context,
        &digits,
        CONFIRMED_LOSS_IN_FLIGHT_CALL,
        CONFIRMED_LOSS_STATE_LIMIT,
    );
    let every_state: BTreeSet<u64> =
        (digits.first_ordinal..observation_tables.tables.state_count).collect();
    let observations = observe_sampled_states(
        &context,
        &SampledOrdinals {
            batches: [every_state, BTreeSet::new(), BTreeSet::new()],
            torn_scans: Vec::new(),
        },
        RunSwitches::default(),
        ConsultedReports::Kept,
    );
    if classification.stopped_at_the_state_limit {
        return EquivalencePairing {
            states: u64::try_from(observations.len()).expect("态数"),
            product_of_the_choices,
            class_states: 0,
            classes: 0,
            mismatching_states: 0,
            classification,
            checks: EquivalenceCheckCounts::default(),
            observations,
        };
    }
    let evaluation = evaluate_the_classes(
        &context,
        &digits,
        &classification,
        &observations,
        CONFIRMED_LOSS_IN_FLIGHT_CALL,
    );
    let mismatching_states = observations
        .iter()
        .filter(|observation| {
            let key = classification
                .partition
                .class_of_ordinal(&digits, observation.ordinal);
            evaluation.classes.get(&key).map(|class| &class.columns)
                != ClassColumns::of_the_observation(observation).as_ref()
        })
        .count();
    EquivalencePairing {
        states: u64::try_from(observations.len()).expect("态数"),
        product_of_the_choices,
        class_states: evaluation.classes.values().map(|class| class.states).sum(),
        classes: evaluation.classes.len(),
        mismatching_states,
        classification,
        checks: evaluation.checks,
        observations,
    }
}

/// 「等价类对拍」那一形的三样（修订 5 第 5.2 节 S1 新加三条、U9）：末段里属于第 3 次调用的写（c1…c_k，按发出次序，写表下标）、
/// 末段各写取法数之积、c1…c_k 按发出次序两两成对（同偏移、同长度、分属两块盘）且各对偏移互不相同的对数。
struct EquivalencePairingShapeFacts {
    in_flight_writes: Vec<usize>,
    states: u64,
    mirrored_pairs: usize,
}

fn equivalence_pairing_shape_facts(
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
) -> EquivalencePairingShapeFacts {
    let call_of_write = call_index_of_each_write(stream);
    let in_flight_writes: Vec<usize> = stream
        .segments
        .last()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|write_index| call_of_write[*write_index] == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL))
        .collect();
    let (_, _, states) = last_segment_product(stream, tables);
    let mut pair_offsets: BTreeSet<u64> = BTreeSet::new();
    let mut mirrored_pairs = 0usize;
    for pair in in_flight_writes.chunks(2) {
        if let [first, second] = pair {
            let first_write = &stream.writes[*first];
            let second_write = &stream.writes[*second];
            if first_write.offset == second_write.offset
                && first_write.length_in_bytes() == second_write.length_in_bytes()
                && first_write.device != second_write.device
                && pair_offsets.insert(first_write.offset.0)
            {
                mirrored_pairs += 1;
            }
        }
    }
    EquivalencePairingShapeFacts {
        in_flight_writes,
        states,
        mirrored_pairs,
    }
}

/// 阳性对照「判态路 panic 单列」的注入（修订 5 第 5.1 节）：c1…c6 是对拍流末段里第 3 次调用的前 6 个写（三对镜像）；
/// `pool_checker`：c3 落、c4 没落；`ignored_recovery`：c3 没落、c4 落；`record_checker`：c5 落、c6 没落，且 (c3,c4) 同落或同不落。
/// 三条选中的态互不相交。C 的写不够 6 个交 None（阳性对照不成立，走 F9）。
fn piece_panic_control_injection(in_flight_writes: &[usize]) -> Option<PiecePanicInjection> {
    let [_, _, third, fourth, fifth, sixth] = in_flight_writes.get(..6)? else {
        return None;
    };
    Some(PiecePanicInjection {
        rules: vec![
            PiecePanicInjectionRule {
                piece: Some(JudgingPiece::PoolChecker),
                conditions: vec![
                    InjectedLandingCondition::Landed(*third),
                    InjectedLandingCondition::NotLanded(*fourth),
                ],
            },
            PiecePanicInjectionRule {
                piece: Some(JudgingPiece::IgnoredRecovery),
                conditions: vec![
                    InjectedLandingCondition::NotLanded(*third),
                    InjectedLandingCondition::Landed(*fourth),
                ],
            },
            PiecePanicInjectionRule {
                piece: Some(JudgingPiece::RecordChecker),
                conditions: vec![
                    InjectedLandingCondition::Landed(*fifth),
                    InjectedLandingCondition::NotLanded(*sixth),
                    InjectedLandingCondition::BothOrNeitherLanded(*third, *fourth),
                ],
            },
        ],
    })
}

/// 一态上 O1 两个崩溃子列与看 journal 那遍恢复的结局（阳性对照「判态路 panic 单列」逐态比的三样）。
fn crash_columns_and_consulted_recovery(
    observation: &StateObservation,
) -> (
    &CrashJudgeVerdict,
    &ReferenceFileSystemVoteOfAState,
    Option<ConsultedRecovery>,
) {
    (
        &observation.crash_judge,
        &observation.reference_filesystem,
        ConsultedRecovery::of_the_observation(observation),
    )
}

/// 注入那一遍对不注入那一遍，两个崩溃子列或看 journal 那遍恢复的结局不同的态数（按崩溃状态序号配对；不注入那一遍里没有的序号也算不同）。
fn o1_columns_differing(plain: &[StateObservation], injected: &[StateObservation]) -> usize {
    let plain_of_ordinal: BTreeMap<u64, &StateObservation> = plain
        .iter()
        .map(|observation| (observation.ordinal, observation))
        .collect();
    injected
        .iter()
        .filter(|observation| {
            plain_of_ordinal
                .get(&observation.ordinal)
                .is_none_or(|plain_observation| {
                    crash_columns_and_consulted_recovery(plain_observation)
                        != crash_columns_and_consulted_recovery(observation)
                })
        })
        .count()
}

/// 阳性对照「判态路 panic 单列」那一遍（`name=q5_piece_panic_control` 一行）。
struct PiecePanicControl {
    states: u64,
    piece_panic_states: [usize; JUDGING_PIECE_COUNT],
    consulted_recovery_panics_of_the_plain_pairing: usize,
    o1_columns_differing_from_the_plain_pairing: usize,
    mismatching_states: usize,
    stops_at_s13: bool,
    observations: Vec<StateObservation>,
}

impl PiecePanicControl {
    /// 注入的三件与看 journal 那遍恢复之外的件 panic 的态数之和。
    fn other_piece_panic_states(&self) -> usize {
        JudgingPiece::ALL
            .iter()
            .zip(self.piece_panic_states)
            .filter(|(piece, _)| {
                !matches!(
                    piece,
                    JudgingPiece::ConsultedRecovery
                        | JudgingPiece::PoolChecker
                        | JudgingPiece::IgnoredRecovery
                        | JudgingPiece::RecordChecker
                )
            })
            .map(|(_, states)| states)
            .sum()
    }

    /// 恰为修订 5 第 5.1 节的精确数（不符走 F9）。
    fn holds(&self) -> bool {
        let states_of = |piece: JudgingPiece| {
            u64::try_from(self.piece_panic_states[piece.position()]).expect("态数装得进 u64")
        };
        self.states == LOCAL_EQUIVALENCE_PAIRING_STATES
            && states_of(JudgingPiece::PoolChecker) == LOCAL_PIECE_PANIC_CONTROL_POOL_CHECKER_STATES
            && states_of(JudgingPiece::IgnoredRecovery)
                == LOCAL_PIECE_PANIC_CONTROL_IGNORED_RECOVERY_STATES
            && states_of(JudgingPiece::RecordChecker)
                == LOCAL_PIECE_PANIC_CONTROL_RECORD_CHECKER_STATES
            && self.other_piece_panic_states() == 0
            && self.piece_panic_states[JudgingPiece::ConsultedRecovery.position()]
                == self.consulted_recovery_panics_of_the_plain_pairing
            && self.o1_columns_differing_from_the_plain_pairing == 0
            && self.mismatching_states == 0
            && !self.stops_at_s13
    }
}

/// 在对拍那一条流上照 [`piece_panic_control_injection`] 注入、再做一遍对拍，与不注入那一遍（`plain`）逐态比。
/// `replacing_the_built_in_rules` 不空时用它替换内置的那三条规则（登记修订 6 第 6.2 节第 3 条，只有内联用例 U14 交）。
fn piece_panic_control_of(
    stream: &PreparedStream,
    plain: &EquivalencePairing,
    replacing_the_built_in_rules: Option<&PiecePanicInjection>,
) -> PiecePanicControl {
    let tables = crash_state_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let facts = equivalence_pairing_shape_facts(stream, &tables);
    let consulted_recovery_panics_of_the_plain_pairing =
        piece_panic_states_of(&plain.observations)[JudgingPiece::ConsultedRecovery.position()];
    let built_in_injection = piece_panic_control_injection(&facts.in_flight_writes);
    let Some(injection) = replacing_the_built_in_rules.or(built_in_injection.as_ref()) else {
        return PiecePanicControl {
            states: 0,
            piece_panic_states: [0; JUDGING_PIECE_COUNT],
            consulted_recovery_panics_of_the_plain_pairing,
            o1_columns_differing_from_the_plain_pairing: 0,
            mismatching_states: 0,
            stops_at_s13: false,
            observations: Vec::new(),
        };
    };
    let injected = equivalence_pairing_with_injected_piece_panics(stream, Some(injection));
    PiecePanicControl {
        states: injected.states,
        piece_panic_states: piece_panic_states_of(&injected.observations),
        consulted_recovery_panics_of_the_plain_pairing,
        o1_columns_differing_from_the_plain_pairing: o1_columns_differing(
            &plain.observations,
            &injected.observations,
        ),
        mismatching_states: injected.mismatching_states,
        stops_at_s13: injected.stops_at_s13(),
        observations: injected.observations,
    }
}

/// `stream-constants` 模式：只驱动、只回比（S1 的流相关几条与状态数几条、与流无关的几条、Q5 造法那几样），不写导出、不判态。
fn run_stream_constants(streams: &[StreamName]) -> i32 {
    let mut lines = ResultLines {
        emitted: 0,
        collected: None,
    };
    lines.emit_result_line("name=config segment=s1 mode=stream-constants");
    let mut failed_checks = 0usize;
    let mut checks = stream_independent_checks();
    checks.extend(stream_independent_checks_of_the_first_rerun());
    checks.extend(confirmed_loss_form_checks_against_the_crates());
    for name in streams {
        let stream = prepare_stream(*name, RunSwitches::default(), true);
        let tables = crash_state_enumeration_tables(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        );
        let full_states = tables.state_count;
        let quick_tier_states = crash_point_replay_plan_state_count(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &quick_tier_expansion,
        );
        checks.extend(stream.checks.iter().cloned());
        checks.extend(state_count_checks_against_the_crates(
            *name,
            &stream,
            &tables,
            flow_facts_state_count(&stream, &tables),
            full_states,
            quick_tier_states,
        ));
        lines.emit_result_line(&format!(
            "name=stream_constants stream={} ending={} calls={} writes={} segments={} reference_filesystem_segments={}",
            name.directory_name(),
            ending_text(&stream.ending),
            stream.calls.len(),
            stream.writes.len(),
            stream.segments.len(),
            segment_closing_stream_indexes(&stream.operations).len()
        ));
    }
    for check in checks {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    lines.emit_result_line(&format!("name=verdict s1_failed_checks={failed_checks}"));
    lines.finish();
    if failed_checks > 0 {
        3
    } else {
        0
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(parsed) = parse_command_line(&arguments) else {
        eprintln!(
            "用法：e171_three_oracle_collusion_after_the_oracle_fixes baseline|row --output-directory <目录> [--streams 1,2,3] [--writable-mount-probe] [--positive-control <名> | --disable-oracle O1|O2]；confirmed-loss-form --output-directory <目录> [--truncate-at-the-return-point]；stream-constants [--streams 1,2,3]；收到 {arguments:?}"
        );
        std::process::exit(2);
    };
    let output_directory = match &parsed {
        ParsedCommand::Dump(command_line) => Some(&command_line.output_directory),
        ParsedCommand::ConfirmedLossForm(command_line) => Some(&command_line.output_directory),
        ParsedCommand::StreamConstants(_) => None,
    };
    if let Some(directory) = output_directory.filter(|directory| directory.exists()) {
        eprintln!(
            "输出目录 {} 已存在：装置不覆盖已有的文件，换一个目录",
            directory.display()
        );
        std::process::exit(2);
    }
    let exit_code = match &parsed {
        ParsedCommand::Dump(command_line) => run_dump(command_line),
        ParsedCommand::StreamConstants(streams) => run_stream_constants(streams),
        ParsedCommand::ConfirmedLossForm(command_line) => run_confirmed_loss_form(command_line),
    };
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 单测里判池级 checker 的给定状态引擎（用户 2026-10-13 定「能走两段的都走两段」）：每张要判的盘作一张给定状态表（基线就是那张盘、写表空、
    /// 一个空持久集合），录入段写进家目录下给定状态表那一类的判定库，判段从库里读回判（配了 GPU 且编进了 `gpu` 特性在卡上判，否则由 CPU 判器判），
    /// 判过的复用。交回每张盘上池级 checker 判到而没违例的不变量与违例的不变量；只录那一趟、表归别的机器判的那一趟交 `None`（用例当场返回）。
    ///
    /// # Panics
    /// 引擎起不了、判不了某一张（不退回别处判）；GPU 判不了某一张（第三轮正文 H6 判红）。
    fn pool_checker_holds_and_violations_through_the_given_states_engine(
        images: &[(String, MemoryPool)],
    ) -> Option<Vec<(Vec<&'static str>, Vec<&'static str>)>> {
        use singlefs_checker_tier::crash_amplification::{GivenStatesEngine, GivenStatesItem};
        use singlefs_checker_tier::crash_given_states::{
            GivenStatesConclusionsSource, GivenStatesJudge, GivenStatesOfATable,
        };
        let engine = GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: "e171_three_oracle_collusion_after_the_oracle_fixes::tests::fixed".to_string(),
            },
            &singlefs_checker_tier::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"));
        let mut judged_images = Vec::with_capacity(images.len());
        let mut every_table_judged_here = true;
        for (table_name, image) in images {
            let judged = engine
                .judge_the_given_states(&GivenStatesOfATable {
                    table_name: table_name.clone(),
                    base: image,
                    writes: &[],
                    segments: &[],
                    seams: &[],
                    persisted_sets: vec![Vec::new()],
                })
                .unwrap_or_else(|reason| {
                    panic!("给定状态表 {table_name} 判不了（不退回别处判）：{reason}")
                });
            match judged.source {
                GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
                GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
                | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => {
                    every_table_judged_here = false;
                }
            }
            let [conclusion] = judged.states.as_slice() else {
                panic!(
                    "{table_name}：一个状态一份结论，交回 {} 份",
                    judged.states.len()
                );
            };
            let invariants_with = |bits: u128| -> Vec<&'static str> {
                singlefs_checker::image::IMPLEMENTED_INVARIANTS
                    .iter()
                    .enumerate()
                    .filter(|(position, _)| bits & (1u128 << position) != 0)
                    .map(|(_, invariant)| *invariant)
                    .collect()
            };
            judged_images.push((
                invariants_with(
                    conclusion.words.pool_checker_judged_bits
                        & !conclusion.words.pool_checker_violated_bits,
                ),
                invariants_with(conclusion.words.pool_checker_violated_bits),
            ));
        }
        assert_eq!(
            engine.finish().red_items(),
            Vec::<String>::new(),
            "GPU 判不了的盘照第三轮正文 H6 判红"
        );
        every_table_judged_here.then_some(judged_images)
    }

    /// 空消息的 sha256（`python3 -c "import hashlib; print(hashlib.sha256(b'').hexdigest())"` 现算）。
    const EMPTY_MESSAGE_SHA256: &str =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    /// 盘 1、扇区 2、512 个 0xAB 那一条的摘要（`struct.pack('<IQ', 1, 2) + b'\xab' * 512` 喂 hashlib 现算）。
    const ONE_SECTOR_OF_0XAB_SHA256: &str =
        "c71cae1b3fc678e4cabfa21d575c728edd44a4b372b3389ce763856273abecbf";

    #[test]
    fn operation_kind_name_covers_every_recorded_operation_kind() {
        assert_eq!(operation_kind_name(RecordedOperationKind::Write), "write");
        assert_eq!(
            operation_kind_name(RecordedOperationKind::WriteForceUnitAccess),
            "write_fua"
        );
        assert_eq!(
            operation_kind_name(RecordedOperationKind::WriteZeroes),
            "write_zeroes"
        );
        assert_eq!(
            operation_kind_name(RecordedOperationKind::Barrier),
            "barrier"
        );
    }

    #[test]
    fn a_torn_landing_folds_to_one_and_a_persisted_write_to_two() {
        // 写表：0 持久、1 没持久、2 原写没持久而撕裂镜像（枚举写表下标 3）持久。
        let persisted = vec![true, false, false, true];
        let torn_image_index_by_write = BTreeMap::from([(2usize, 3usize)]);
        assert_eq!(
            landing_vector_of(&persisted, 3, &torn_image_index_by_write),
            "201",
            "撕裂那一态折回 1，不是 2"
        );
        let persisted_with_the_torn_image_absent = vec![true, false, false, false];
        assert_eq!(
            landing_vector_of(
                &persisted_with_the_torn_image_absent,
                3,
                &torn_image_index_by_write
            ),
            "200"
        );
    }

    #[test]
    fn an_all_zero_sector_does_not_enter_the_image_digest() {
        let mut message = Vec::new();
        append_sector_to_digest_message(&mut message, 0, 7, &[0u8; 512]);
        assert!(message.is_empty(), "全零扇区不进摘要");
        assert_eq!(sha256_text(&sha256_digest(&message)), EMPTY_MESSAGE_SHA256);
        append_sector_to_digest_message(&mut message, 1, 2, &[0xAB; 512]);
        assert_eq!(message.len(), 4 + 8 + 512);
        assert_eq!(
            sha256_text(&sha256_digest(&message)),
            ONE_SECTOR_OF_0XAB_SHA256
        );
    }

    #[test]
    fn batch_points_follow_the_registered_formulas_on_a_hundred_state_range() {
        let state_range = 10u64..110;
        assert_eq!(
            first_batch_points(&state_range)
                .into_iter()
                .collect::<BTreeSet<u64>>(),
            BTreeSet::from([10, 24, 38, 52, 66, 80, 94, 109])
        );
        assert_eq!(
            second_batch_points(&state_range),
            vec![17, 31, 45, 59, 73, 87, 101]
        );
        let third = third_batch_points(&state_range);
        assert_eq!(third.len(), 24);
        assert_eq!(third[0], 10);
        assert_eq!(third[1], 14, "10 + ⌊99/23⌋");
        assert_eq!(third[23], 109);
    }

    #[test]
    fn batch_points_on_short_ranges_collapse_onto_the_first_batch() {
        let single = 5u64..6;
        assert_eq!(
            first_batch_points(&single)
                .into_iter()
                .collect::<BTreeSet<u64>>(),
            BTreeSet::from([5])
        );
        let eight = 0u64..8;
        assert_eq!(
            first_batch_points(&eight)
                .into_iter()
                .collect::<BTreeSet<u64>>(),
            (0..8).collect::<BTreeSet<u64>>()
        );
        assert_eq!(second_batch_points(&eight), vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn sampled_batches_are_disjoint_and_include_the_last_state_and_up_to_four_torn_states() {
        // 两段：[0, 8) 带撕裂写，序号为奇数的状态带 1；[8, 108) 不带。总数 109，最后那个是 108。
        let ranges = vec![0u64..8, 8..108];
        let landing = |ordinal: u64| {
            if ordinal % 2 == 1 {
                "21".to_string()
            } else {
                "20".to_string()
            }
        };
        let sampled = sample_ordinals(&ranges, 109, &[true, false], &landing);
        let [first, second, third] = &sampled.batches;
        assert!(first.contains(&108), "最后那个全持久状态进批 1");
        assert!(first.is_disjoint(second) && first.is_disjoint(third) && second.is_disjoint(third));
        assert_eq!(
            sampled.torn_scans,
            vec![TornScan {
                segment_index: 0,
                scanned: 8,
                found: 0
            }],
            "[0, 8) 的点批 1 已经全取了，扫完 8 个一个新的都没有"
        );
    }

    #[test]
    fn torn_scan_stops_after_four_new_states() {
        let ranges: Vec<std::ops::Range<u64>> = std::iter::once(0u64..1000).collect();
        let landing = |ordinal: u64| {
            if ordinal % 3 == 2 {
                "1".to_string()
            } else {
                "0".to_string()
            }
        };
        let sampled = sample_ordinals(&ranges, 1001, &[true], &landing);
        // 批 1 的格点 0、142、285、428、570、713、856、999；带 1 的是 2、5、8、11（都不在格点上）。
        assert_eq!(sampled.torn_scans[0].found, 4);
        assert_eq!(sampled.torn_scans[0].scanned, 12);
        for ordinal in [2u64, 5, 8, 11] {
            assert!(sampled.batches[0].contains(&ordinal));
        }
    }

    /// 三条流本地段长表的闭式 1 + Σ(2^n − 1) 与写数，与本地抄来的闭式、写数逐个相等（改段长表里任何一个数，这里都红）。
    #[test]
    fn local_segment_tables_reproduce_the_closed_forms_and_write_counts_copied_from_the_tests() {
        let closed_form = |lengths: &[usize]| -> u64 {
            1 + lengths
                .iter()
                .map(|length| (1u64 << length) - 1)
                .sum::<u64>()
        };
        assert_eq!(
            closed_form(&LOCAL_FIRST_STREAM_SEGMENT_LENGTHS),
            LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM
        );
        assert_eq!(
            LOCAL_FIRST_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
            LOCAL_FIRST_STREAM_WRITES
        );
        assert_eq!(
            closed_form(&LOCAL_SECOND_STREAM_SEGMENT_LENGTHS),
            LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM
        );
        assert_eq!(
            LOCAL_SECOND_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
            LOCAL_SECOND_STREAM_WRITES
        );
        assert_eq!(
            closed_form(LOCAL_THIRD_STREAM_SEGMENT_LENGTHS),
            LOCAL_THIRD_STREAM_TWO_STATE_CLOSED_FORM
        );
        assert_eq!(
            LOCAL_THIRD_STREAM_SEGMENT_LENGTHS.iter().sum::<usize>(),
            LOCAL_THIRD_STREAM_WRITES
        );
        assert_eq!(LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM, 1_678_377_089);
        assert_eq!(LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM, 16_777_240);
    }

    fn bytes_write(device: u32, offset: u64, bytes: Vec<u8>) -> RetainedWrite {
        RetainedWrite {
            device: DeviceIdentity(device),
            kind: StepKind::SystemConfigurationSlot,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(offset),
            contents: WrittenContents::Bytes(bytes),
        }
    }

    fn neutral(
        device: u32,
        kind: RecordedOperationKind,
        offset: u64,
        length: u64,
        payload_start: Option<usize>,
    ) -> NeutralOperation {
        NeutralOperation {
            device,
            kind,
            offset,
            length,
            payload_start,
        }
    }

    /// 小池上同一个状态两条路算摘要：`CrashImage` 直接读（带撕裂镜像的枚举写表）与中性格式按落地向量重建，两份逐字节相同；
    /// 落地向量按写序号是「2 2 0 1」。
    #[test]
    fn rebuilding_from_the_neutral_format_matches_the_crash_image_digest_on_a_small_pool() {
        let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], 1 << 20);
        base.apply_writes(&[
            bytes_write(0, 0, vec![0x11; 512]),
            bytes_write(1, 8192, vec![0x22; 1024]),
        ]);
        let written_at_4096 = vec![0x33; 1024];
        let zeroes_over_the_base_sector = RetainedWrite {
            device: DeviceIdentity(0),
            kind: StepKind::ZeroFill,
            is_force_unit_access: false,
            offset: DeviceOffsetInBytes(0),
            contents: WrittenContents::Zeros { length: 512 },
        };
        let not_persisted = vec![0x44; 512];
        let overwrite_new = vec![0x55; 1024];
        let torn = [vec![0x55; 512], vec![0x22; 512]].concat();
        let writes = vec![
            bytes_write(0, 4096, written_at_4096.clone()),
            zeroes_over_the_base_sector,
            bytes_write(1, 0, not_persisted.clone()),
            bytes_write(1, 8192, overwrite_new.clone()),
            bytes_write(1, 8192, torn.clone()),
        ];
        let image = CrashImage {
            base: &base,
            writes: &writes,
            persisted: vec![true, true, false, false, true],
        };
        let landing = landing_vector_of(&image.persisted, 4, &BTreeMap::from([(3usize, 4usize)]));
        assert_eq!(landing, "2201");
        let payload = [written_at_4096, not_persisted, overwrite_new].concat();
        let stream = NeutralStream {
            base_sectors: BTreeMap::from([
                ((0u32, 0u64), vec![0x11; 512]),
                ((1, 16), vec![0x22; 512]),
                ((1, 17), vec![0x22; 512]),
            ]),
            operations: vec![
                neutral(0, RecordedOperationKind::Write, 4096, 1024, Some(0)),
                neutral(0, RecordedOperationKind::Barrier, 0, 0, None),
                neutral(0, RecordedOperationKind::WriteZeroes, 0, 512, None),
                neutral(1, RecordedOperationKind::Write, 0, 512, Some(1024)),
                neutral(
                    1,
                    RecordedOperationKind::WriteForceUnitAccess,
                    8192,
                    1024,
                    Some(1536),
                ),
            ],
            payload,
            torn_bytes_by_write: BTreeMap::from([(3usize, (0usize, 1024usize))]),
            torn_payload: torn,
        };
        let from_the_image = crash_image_digest(&image);
        assert_eq!(rebuilt_image_digest(&stream, &landing), from_the_image);
        assert_ne!(
            rebuilt_image_digest(&stream, "2202"),
            from_the_image,
            "撕裂那一态记成持久，重建出来的是新字节，摘要必须不同"
        );
    }

    #[test]
    fn contents_without_timing_drops_only_the_timing_column_and_the_verdicts_digest_line() {
        assert_eq!(
            contents_without_timing(
                IMPLEMENTATION_VERDICTS_FILE,
                b"0\ta\tb\t123\n1\tc\td\t4567\n"
            ),
            Some(b"0\ta\tb\n1\tc\td\n".to_vec())
        );
        assert_eq!(
            contents_without_timing(
                MANIFEST_FILE,
                b"stream\tx\nsha256:states.tsv\taa\nsha256:impl-verdicts.tsv\tbb\n"
            ),
            Some(b"stream\tx\nsha256:states.tsv\taa\n".to_vec())
        );
        assert_eq!(contents_without_timing(STATES_FILE, b"0\t1\n"), None);
    }

    #[test]
    fn timing_summary_takes_the_lower_middle_and_the_ninety_fifth_percentile() {
        assert_eq!(timing_summary(vec![]), (0, 0, 0, 0));
        assert_eq!(timing_summary(vec![4, 1, 3, 2]), (10, 2, 4, 4));
        let hundred: Vec<u128> = (1..=100).collect();
        assert_eq!(timing_summary(hundred), (5050, 50, 95, 100));
    }

    #[test]
    fn merged_sector_ranges_join_overlapping_and_touching_ranges_only() {
        assert_eq!(
            merged_sector_ranges(vec![(10, 12), (0, 2), (2, 3), (11, 15), (20, 21)]),
            vec![(0, 3), (10, 15), (20, 21)]
        );
    }

    /// 池级 checker 的判定里只有 `Violated` 算 O2 红（X6：`Violated` 当 `Holds`，这里红）。
    #[test]
    fn only_a_violated_invariant_counts_as_a_red_pool_checker_verdict() {
        let verdicts: Vec<(&'static str, InvariantVerdict)> = vec![
            ("I-1.1", InvariantVerdict::Violated("坏".to_string())),
            ("I-1.2", InvariantVerdict::Holds),
            ("I-1.3", InvariantVerdict::NotApplicable("没有对象")),
        ];
        assert_eq!(
            violated_invariant_names(&verdicts),
            vec!["I-1.1".to_string()]
        );
    }

    /// 模型不认的版本读回来记红、认的记绿（X4：崩溃判官的结果丢掉，这里红）。
    #[test]
    fn a_read_back_of_a_version_the_model_never_committed_is_red_and_a_committed_one_is_green() {
        let committed_key = model_root_key(InstanceGeneration(1), CheckpointTxg(3));
        let committed = BTreeMap::from([(committed_key, Some(Rc::<[u8]>::from(vec![7u8, 8, 9])))]);
        let green = ObservedReadBack::FileRead {
            root: committed_key,
            content: vec![7, 8, 9],
        };
        assert_eq!(
            crash_judge_verdict(&committed, Some(committed_key), &green),
            CrashJudgeVerdict::Green
        );
        let flipped_last_byte = ObservedReadBack::FileRead {
            root: committed_key,
            content: vec![7, 8, 8],
        };
        assert!(matches!(
            crash_judge_verdict(&committed, Some(committed_key), &flipped_last_byte),
            CrashJudgeVerdict::Red { .. }
        ));
        let never_committed = ObservedReadBack::NoFile {
            root: model_root_key(InstanceGeneration(9), CheckpointTxg(99)),
        };
        assert!(matches!(
            crash_judge_verdict(&committed, None, &never_committed),
            CrashJudgeVerdict::Red { .. }
        ));
    }

    /// 模型停在第 3 次调用：第 2 次及之前的调用里写的态适用，第 3 次及之后的不适用；一个写都没落的态与模型没停的流都适用
    /// （X14：流二每一态都记不适用，这里红）。
    #[test]
    fn the_crash_judge_is_not_applicable_only_from_the_call_where_the_model_stopped() {
        assert!(crash_judge_is_applicable(Some(2), Some(3)));
        assert!(!crash_judge_is_applicable(Some(3), Some(3)));
        assert!(!crash_judge_is_applicable(Some(5), Some(3)));
        assert!(crash_judge_is_applicable(None, Some(0)));
        assert!(crash_judge_is_applicable(Some(5), None));
    }

    fn unit_text(_: &()) -> String {
        "()".to_string()
    }

    /// 模型没有会话时问发布第一个文件：答案本身就是一格对不上，驱动把它记成 `Disagrees`、模型停在这次调用，下一次调用不比
    /// （X5：`judge_and_advance` 的不同丢掉，这里红）。
    #[test]
    fn a_comparison_the_model_cannot_answer_is_recorded_as_a_disagreement_and_stops_the_model() {
        let mut driver = StreamDriver::new(
            StreamName::NewPoolFileCreation,
            SharedStream::retaining_contents(),
            0,
            RunSwitches::default(),
        );
        let refused: Result<(), &str> = Err("拒");
        assert!(driver.call("publish", unit_text, || refused).is_err());
        driver.judge(
            |model| model.answer_publish_first_file(b"x"),
            ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                roots: Vec::new(),
                reported_ceiling: None,
            }),
        );
        assert!(matches!(
            driver.calls[0].model,
            ModelVerdictOfCall::Disagrees { .. }
        ));
        assert_eq!(driver.model_stopped_at_call, Some(0));
        let accepted: Result<(), &str> = Ok(());
        assert!(driver.call("publish", unit_text, || accepted).is_ok());
        driver.judge(
            |model| model.answer_publish_first_file(b"x"),
            ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                roots: Vec::new(),
                reported_ceiling: None,
            }),
        );
        assert_eq!(
            driver.calls[1].model,
            ModelVerdictOfCall::SkippedAfterTheModelStopped
        );
    }

    /// 模型这一步要求成功、实现却交回一个说不出理由的错误：`judge_and_advance` 自己判出一格对不上，驱动记成 `Disagrees` 并让模型停在这次调用
    /// （X5：`judge_and_advance` 的不同丢掉，这里红）。
    #[test]
    fn an_implementation_refusal_the_model_does_not_allow_is_recorded_as_a_disagreement() {
        let mut driver = StreamDriver::new(
            StreamName::NewPoolFileCreation,
            SharedStream::retaining_contents(),
            0,
            RunSwitches::default(),
        );
        driver.advance_the_model_past_the_acquire_and_the_warm_up();
        let refused: Result<(), &str> = Err("拒");
        assert!(driver.call("publish", unit_text, || refused).is_err());
        let observed = driver.refusal(
            &"拒",
            singlefs_harness::model::ObservedRefusalReason::Unexplained,
            0,
            0,
            None,
            None,
        );
        driver.judge(|model| model.answer_publish_first_file(b"x"), observed);
        assert!(
            matches!(driver.calls[0].model, ModelVerdictOfCall::Disagrees { .. }),
            "模型要求成功、实现拒了：{:?}",
            driver.calls[0].model
        );
        assert_eq!(driver.model_stopped_at_call, Some(0));
    }

    /// 单测里造一个说得出成员名的错误：`error_member_of` 取 `Debug` 文本的第一个标识符，`&str` 的 `Debug` 文本以引号起头、取不到。
    #[derive(Debug)]
    enum RefusedForTheTest {
        DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
    }

    /// 取号交回错误：这条流不带故障，模型对取号只有「往前走」一种走法，驱动把它记成「模型说该成、实现拒了」，模型停在这次调用
    /// （代码轮第三轮判决 K4-c；装置一侧变异：记红丢掉，这里红）。
    #[test]
    fn a_refused_acquire_on_a_stream_without_faults_is_recorded_as_a_comparison_red_and_stops_the_model(
    ) {
        let mut driver = StreamDriver::new(
            StreamName::NewPoolFileCreation,
            SharedStream::retaining_contents(),
            0,
            RunSwitches::default(),
        );
        let refused_acquire: Result<(), RefusedForTheTest> = Err(
            RefusedForTheTest::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
        );
        let acquire_failure = driver
            .call("acquire", unit_text, || refused_acquire)
            .expect_err("取号被拒");
        driver.record_a_refusal_the_model_only_goes_forward_on(acquire_failure);
        assert_eq!(
            driver.calls[0].model,
            ModelVerdictOfCall::Disagrees {
                aspect: "模型说该成、实现拒了",
                model_answer: "往前走".to_string(),
                implementation_answer: "拒了:DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness".to_string(),
            }
        );
        assert_eq!(
            driver.calls[0].model.word(),
            "disagrees:模型说该成、实现拒了"
        );
        assert_eq!(driver.model_stopped_at_call, Some(0));
        // 暖机也一样：取号成功之后暖机被拒，记在第二次调用上，第一次调用不动。
        let mut after_a_good_acquire = StreamDriver::new(
            StreamName::NewPoolFileCreation,
            SharedStream::retaining_contents(),
            0,
            RunSwitches::default(),
        );
        let accepted: Result<(), RefusedForTheTest> = Ok(());
        assert!(after_a_good_acquire
            .call("acquire", unit_text, || accepted)
            .is_ok());
        let refused_warm_up: Result<(), RefusedForTheTest> = Err(
            RefusedForTheTest::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
        );
        let warm_up_failure = after_a_good_acquire
            .call("warm_up", unit_text, || refused_warm_up)
            .expect_err("暖机被拒");
        after_a_good_acquire.record_a_refusal_the_model_only_goes_forward_on(warm_up_failure);
        assert_eq!(
            after_a_good_acquire.calls[0].model,
            ModelVerdictOfCall::NotModelled
        );
        assert_eq!(
            after_a_good_acquire.calls[1].model.word(),
            "disagrees:模型说该成、实现拒了"
        );
        assert_eq!(after_a_good_acquire.model_stopped_at_call, Some(1));
    }

    /// 取号 panic：与交回错误同样截断（读法 5），结局记 `panic`、O1-对拍记「实现自崩、不比」（不红不绿），模型停在这次调用，
    /// 这条流的结局是 panic 截断（照导出照判）；`record_a_refusal_the_model_only_goes_forward_on` 收到 `None` 不记红。
    #[test]
    fn a_panic_in_the_acquire_is_an_implementation_crash_and_not_a_comparison_red() {
        let mut driver = StreamDriver::new(
            StreamName::NewPoolFileCreation,
            SharedStream::retaining_contents(),
            0,
            RunSwitches::default(),
        );
        let failure = driver
            .call("acquire", unit_text, || -> Result<(), RefusedForTheTest> {
                panic!("取号自崩")
            })
            .expect_err("panic 交回 Err(None)");
        assert!(failure.is_none());
        driver.record_a_refusal_the_model_only_goes_forward_on(failure);
        assert_eq!(
            driver.calls[0].model,
            ModelVerdictOfCall::NotComparedBecauseTheImplementationCrashed
        );
        assert!(!driver.calls[0].model.was_judged_by_the_comparison());
        assert_eq!(driver.calls[0].outcome.calls_file_word(), "err");
        assert_eq!(driver.model_stopped_at_call, Some(0));
        assert_eq!(
            ending_of_the_driver(&driver),
            StreamEnding::StoppedByAPanic {
                call_index: 0,
                message: "取号自崩".to_string()
            }
        );
    }

    /// 阳性对照「panic」只在流二第 3 次调用（从 0 数）的闭包里先 panic：之前的调用照跑，第 3 次记 panic（不调实现）。
    #[test]
    fn the_panic_control_panics_only_in_the_fourth_call_of_the_second_stream() {
        let switches = RunSwitches {
            positive_control: Some(PositiveControl::PanicInTheCall),
            disabled_oracle: None,
            writable_mount_probe: false,
        };
        let mut driver = StreamDriver::new(
            StreamName::FixedScriptThroughEThenUnmount,
            SharedStream::retaining_contents(),
            0,
            switches,
        );
        let mut implementation_ran = Vec::new();
        for call_index in 0..=PANIC_CONTROL_CALL {
            let _ = driver.call("publish", unit_text, || -> Result<(), RefusedForTheTest> {
                implementation_ran.push(call_index);
                Ok(())
            });
        }
        assert_eq!(implementation_ran, vec![0, 1, 2]);
        assert!(matches!(
            driver.calls[PANIC_CONTROL_CALL].outcome,
            CallOutcome::Panicked(_)
        ));
        let mut first_stream = StreamDriver::new(
            StreamName::NewPoolFileCreation,
            SharedStream::retaining_contents(),
            0,
            switches,
        );
        for _ in 0..=PANIC_CONTROL_CALL {
            assert!(first_stream
                .call("publish", unit_text, || -> Result<(), RefusedForTheTest> {
                    Ok(())
                })
                .is_ok());
        }
    }

    /// 关一条（Q3-核）关掉 O1 时每次调用都不比：被拒的取号也不记红，O1-对拍那一栏留在 `not_modelled`。
    #[test]
    fn with_the_model_switched_off_a_refused_acquire_is_not_a_comparison_red() {
        let mut driver = StreamDriver::new(
            StreamName::NewPoolFileCreation,
            SharedStream::retaining_contents(),
            0,
            RunSwitches {
                positive_control: None,
                disabled_oracle: Some(DisabledOracle::ComparisonAndCrashJudge),
                writable_mount_probe: false,
            },
        );
        let refused: Result<(), RefusedForTheTest> = Err(
            RefusedForTheTest::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
        );
        let failure = driver
            .call("acquire", unit_text, || refused)
            .expect_err("取号被拒");
        driver.record_a_refusal_the_model_only_goes_forward_on(failure);
        assert_eq!(driver.calls[0].model, ModelVerdictOfCall::NotModelled);
    }

    /// 真池上走一遍取号、暖机：O1-对拍那一栏没有红（照抄原装置那一条：核心改坏取号逐盘核时这一条红，装置不记红时这一条不红）。
    #[test]
    fn an_acquire_and_a_warm_up_on_a_fresh_pool_leave_the_comparison_column_without_a_red() {
        let mut live = LiveStream::start(
            StreamName::NewPoolFileCreation,
            "e171r1-unit-acquire-and-warm-up-comparison",
            RunSwitches::default(),
        );
        let _ = live.step_acquire_and_warm_up();
        let calls_red_in_the_comparison: Vec<&'static str> = live
            .driver
            .calls
            .iter()
            .filter(|call| matches!(call.model, ModelVerdictOfCall::Disagrees { .. }))
            .map(|call| call.label)
            .collect();
        assert!(
            calls_red_in_the_comparison.is_empty(),
            "取号或暖机被拒、O1-对拍记了红：{calls_red_in_the_comparison:?}"
        );
    }

    /// 真池上取号、暖机的绝对值：两次调用都成功、模型都不比、模型没停、第一次取到的号是 1；两次调用都记了交回值与五样内存对象。
    #[test]
    fn an_acquire_and_a_warm_up_on_a_fresh_pool_succeed_and_the_model_is_not_asked() {
        let mut live = LiveStream::start(
            StreamName::NewPoolFileCreation,
            "e171r1-unit-acquire-and-warm-up-absolute",
            RunSwitches::default(),
        );
        assert!(live.step_acquire_and_warm_up());
        let labels: Vec<&'static str> = live.driver.calls.iter().map(|call| call.label).collect();
        assert_eq!(labels, vec!["acquire", "warm_up"]);
        assert!(live
            .driver
            .calls
            .iter()
            .all(|call| call.outcome == CallOutcome::Succeeded
                && call.model == ModelVerdictOfCall::NotModelled
                && call.return_value.is_some()
                && call.memory_objects.len() == 5));
        assert_eq!(live.driver.model_stopped_at_call, None);
        assert_eq!(live.instance, Some(InstanceGeneration(1)));
    }

    fn recorded_call_for_the_test(
        label: &'static str,
        outcome: CallOutcome,
        model: ModelVerdictOfCall,
    ) -> RecordedCall {
        RecordedCall {
            label,
            first_operation: 0,
            return_point: 0,
            outcome,
            model,
            return_value: None,
            memory_objects: Vec::new(),
            published_content: None,
            observation_sha256: None,
        }
    }

    fn disagreement_for_the_test() -> ModelVerdictOfCall {
        ModelVerdictOfCall::Disagrees {
            aspect: "模型说该成、实现拒了",
            model_answer: "往前走".to_string(),
            implementation_answer: "拒了:X".to_string(),
        }
    }

    fn stop_check_values(checks: &[StopCheck]) -> Vec<(&str, &str, bool)> {
        checks
            .iter()
            .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
            .collect()
    }

    /// 逐行那一趟的 S5：交回了错误而 O1-对拍没问模型（`not_modelled`）的调用、与不适用来自读法 6 那三处之外的来源，各自回比 0。
    #[test]
    fn the_row_s5_counts_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source(
    ) {
        let errored = |text: &str| CallOutcome::ReturnedError(text.to_string());
        // 取号出错、O1-对拍没记红：第一样 1 处不合；模型停在出错的那次调用（读法 6 的来源之一），第二样 0。
        let silent = [recorded_call_for_the_test(
            "acquire",
            errored("X"),
            ModelVerdictOfCall::NotModelled,
        )];
        let silent_checks = s5_checks_of_a_bad_implementation_row("s", &silent, Some(0), 3);
        assert_eq!(
            silent_checks
                .iter()
                .map(|check| check.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "s_o1_every_errored_call_is_judged_by_the_comparison",
                "s_o1_not_applicable_only_after_a_disagreement_an_error_or_a_panic"
            ]
        );
        assert_eq!(
            stop_check_values(&silent_checks),
            vec![("0", "1", false), ("0", "0", true)]
        );
        // 取号出错、记成对拍的红：两样都 0。
        let red = [recorded_call_for_the_test(
            "acquire",
            errored("X"),
            disagreement_for_the_test(),
        )];
        assert!(s5_checks_of_a_bad_implementation_row("s", &red, Some(0), 9)
            .iter()
            .all(StopCheck::holds));
        // 对拍在发布上判出不同、模型停在那里，之后的可写挂载出错（`skipped_after_the_model_stopped`）：两样都 0（原登记修订 4 那一形）。
        let error_after_the_stop = [
            recorded_call_for_the_test(
                "publish",
                CallOutcome::Succeeded,
                disagreement_for_the_test(),
            ),
            recorded_call_for_the_test(
                "mount_writable",
                errored("Recovery"),
                ModelVerdictOfCall::SkippedAfterTheModelStopped,
            ),
        ];
        assert_eq!(
            stop_check_values(&s5_checks_of_a_bad_implementation_row(
                "s",
                &error_after_the_stop,
                Some(0),
                6
            )),
            vec![("0", "0", true), ("0", "0", true)]
        );
        // 发布出错而模型说拒得对（`agrees`）、之后有 5 个不适用的态：读法 6「调用交回错误之后（含那一次）」是合规来源，两样都 0。
        let allowed_refusal = [
            recorded_call_for_the_test(
                "acquire",
                CallOutcome::Succeeded,
                ModelVerdictOfCall::NotModelled,
            ),
            recorded_call_for_the_test(
                "warm_up",
                CallOutcome::Succeeded,
                ModelVerdictOfCall::NotModelled,
            ),
            recorded_call_for_the_test("publish", errored("Y"), ModelVerdictOfCall::Agrees),
        ];
        assert_eq!(
            stop_check_values(&s5_checks_of_a_bad_implementation_row(
                "s",
                &allowed_refusal,
                Some(2),
                5
            )),
            vec![("0", "0", true), ("0", "0", true)]
        );
        // 卸载由模型答之后，模型停在一次对拍判过、成功的卸载上不再是合规来源：5 个不适用的态记 5 处不合。
        let stopped_on_an_agreeing_unmount = [
            recorded_call_for_the_test(
                "publish",
                CallOutcome::Succeeded,
                ModelVerdictOfCall::Agrees,
            ),
            recorded_call_for_the_test(
                UNMOUNT_CALL_LABEL,
                CallOutcome::Succeeded,
                ModelVerdictOfCall::Agrees,
            ),
        ];
        assert_eq!(
            stop_check_values(&s5_checks_of_a_bad_implementation_row(
                "s",
                &stopped_on_an_agreeing_unmount,
                Some(1),
                5
            )),
            vec![("0", "0", true), ("0", "5", false)]
        );
        // panic 截断：模型停在 panic 的那次调用，合规；panic 那次不算「出了错没问模型」。
        let panicked = [
            recorded_call_for_the_test(
                "publish",
                CallOutcome::Succeeded,
                ModelVerdictOfCall::Agrees,
            ),
            recorded_call_for_the_test(
                "publish",
                CallOutcome::Panicked("炸".to_string()),
                ModelVerdictOfCall::NotComparedBecauseTheImplementationCrashed,
            ),
        ];
        assert!(
            s5_checks_of_a_bad_implementation_row("s", &panicked, Some(1), 4)
                .iter()
                .all(StopCheck::holds)
        );
        // 模型没停：没有不适用的态，合规。
        let no_stop = [recorded_call_for_the_test(
            "publish",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::Agrees,
        )];
        assert!(
            s5_checks_of_a_bad_implementation_row("s", &no_stop, None, 0)
                .iter()
                .all(StopCheck::holds)
        );
    }

    /// 哪一趟判哪一份 S5：逐行那一趟判「交回了错误的调用都记了对拍的红」那一份，基线那一趟判「模型一次都不停」那一份；
    /// 关掉 O1、流水线 panic 的流不判；基线带阳性对照、流没导出也不判。
    #[test]
    fn the_row_run_judges_the_row_s5_and_the_baseline_run_judges_the_baseline_s5() {
        let started = Instant::now();
        let silent_acquire_stopped_by_the_error = || {
            let mut export =
                StreamExport::not_exported(StreamName::NewPoolFileCreation, String::new(), started);
            export.not_exported = None;
            export.ending = Some(StreamEnding::StoppedByAnError {
                call_index: 0,
                member: "err:X".to_string(),
            });
            export.calls = vec![recorded_call_for_the_test(
                "acquire",
                CallOutcome::ReturnedError("X".to_string()),
                ModelVerdictOfCall::NotModelled,
            )];
            export.model_stopped_at_call = Some(0);
            export
        };
        let export = silent_acquire_stopped_by_the_error();
        let row_checks = s5_checks_to_judge(
            &export,
            RunKind::BadImplementationRow,
            RunSwitches::default(),
        );
        assert_eq!(
            row_checks
                .iter()
                .map(|check| (check.name.as_str(), check.holds()))
                .collect::<Vec<_>>(),
            vec![
                ("new_pool_file_creation_o1_every_errored_call_is_judged_by_the_comparison", false),
                ("new_pool_file_creation_o1_not_applicable_only_after_a_disagreement_an_error_or_a_panic", true)
            ]
        );
        let with_the_model_off = RunSwitches {
            positive_control: None,
            disabled_oracle: Some(DisabledOracle::ComparisonAndCrashJudge),
            writable_mount_probe: false,
        };
        assert!(
            s5_checks_to_judge(&export, RunKind::BadImplementationRow, with_the_model_off)
                .is_empty()
        );
        let mut pipeline_panicked = silent_acquire_stopped_by_the_error();
        pipeline_panicked.ending = None;
        assert!(s5_checks_to_judge(
            &pipeline_panicked,
            RunKind::BadImplementationRow,
            RunSwitches::default()
        )
        .is_empty());
        // panic 截断的流照判（读法 5：panic 与报错同样截断照判）。
        let mut panicked = silent_acquire_stopped_by_the_error();
        panicked.ending = Some(StreamEnding::StoppedByAPanic {
            call_index: 0,
            message: "炸".to_string(),
        });
        assert_eq!(
            s5_checks_to_judge(
                &panicked,
                RunKind::BadImplementationRow,
                RunSwitches::default()
            )
            .len(),
            2
        );
        let mut clean =
            StreamExport::not_exported(StreamName::NewPoolFileCreation, String::new(), started);
        clean.not_exported = None;
        clean.ending = Some(StreamEnding::Completed);
        clean.calls = vec![
            recorded_call_for_the_test(
                "acquire",
                CallOutcome::Succeeded,
                ModelVerdictOfCall::NotModelled,
            ),
            recorded_call_for_the_test(
                "warm_up",
                CallOutcome::Succeeded,
                ModelVerdictOfCall::NotModelled,
            ),
            recorded_call_for_the_test(
                "publish",
                CallOutcome::Succeeded,
                ModelVerdictOfCall::Agrees,
            ),
            recorded_call_for_the_test(
                UNMOUNT_CALL_LABEL,
                CallOutcome::Succeeded,
                ModelVerdictOfCall::Agrees,
            ),
        ];
        let baseline_checks = s5_checks_to_judge(&clean, RunKind::Baseline, RunSwitches::default());
        assert_eq!(
            baseline_checks
                .iter()
                .map(|check| (check.name.as_str(), check.holds()))
                .collect::<Vec<_>>(),
            vec![
                ("new_pool_file_creation_o1_model_never_stops", true),
                (
                    "new_pool_file_creation_every_call_is_judged_or_is_the_acquire_or_the_warm_up",
                    true
                )
            ]
        );
        // 卸载留在 not_modelled（没改调模型的卸载答）在基线上不过。
        clean.calls[3].model = ModelVerdictOfCall::NotModelled;
        assert!(!s5_checks_to_judge(&clean, RunKind::Baseline, RunSwitches::default())[1].holds());
        clean.not_exported = Some("x".to_string());
        assert!(s5_checks_to_judge(&clean, RunKind::Baseline, RunSwitches::default()).is_empty());
    }

    /// 交回值的摘要：整份与顶层字段逐个；字段名照 `{:?}` 文本切（嵌套括号、引号里的逗号不切）；两份只差一个字段，整份摘要不同、只那个字段的摘要不同
    /// （V2：摘要取成常量，这里红）。
    #[test]
    fn a_return_value_digest_names_every_top_level_field_and_changes_with_one_field() {
        let text = r#"TransactionOutput { root: RootRecord { checkpoint_txg: CheckpointTxg(3) }, record_bytes: [1, 2, 3], name: "a, b" }"#;
        let changed = r#"TransactionOutput { root: RootRecord { checkpoint_txg: CheckpointTxg(4) }, record_bytes: [1, 2, 3], name: "a, b" }"#;
        let digest = return_value_digest_of_debug_text(text);
        let changed_digest = return_value_digest_of_debug_text(changed);
        assert_eq!(
            digest
                .fields
                .iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            vec!["root", "record_bytes", "name"]
        );
        assert_eq!(digest.whole, sha256_hexadecimal(text.as_bytes()));
        assert_ne!(digest.whole, changed_digest.whole);
        assert_ne!(digest.fields[0].1, changed_digest.fields[0].1);
        assert_eq!(digest.fields[1..], changed_digest.fields[1..]);
        assert_eq!(
            top_level_fields_of_debug_text("InstanceGeneration(1)"),
            vec![("#0".to_string(), "1".to_string())]
        );
        assert_eq!(
            top_level_fields_of_debug_text("None"),
            vec![("#0".to_string(), "None".to_string())]
        );
    }

    /// 内存对象五样、次序固定、各自是 `{:?}` 文本（V3：不记分配器那一项，这里红）。
    #[test]
    fn the_memory_objects_are_the_allocator_the_instance_the_warm_up_the_current_version_and_the_mount_output(
    ) {
        let allocator = PoolAllocator::new(vec![DeviceFreeMap::new(DeviceIdentity(0), 1 << 30)]);
        let objects = memory_objects_of(
            &allocator,
            Some(InstanceGeneration(2)),
            None,
            None,
            Some("挂载"),
        );
        assert_eq!(
            objects.iter().map(|(name, _)| *name).collect::<Vec<_>>(),
            vec![
                "allocator",
                "instance",
                "warm_up_output",
                "current_version",
                "mount_output"
            ]
        );
        assert_eq!(objects[0].1, format!("{allocator:?}"));
        assert_eq!(objects[1].1, "Some(InstanceGeneration(2))");
        assert_eq!(objects[4].1, "挂载");
    }

    /// 态上的恢复 panic 不截断（读法 5；V5：退回「交回 Err、后面的态不判」，这里红）：一段 [3, 9) 在序号 5 上 panic，
    /// 5 记成恢复 panic 的态、6、7、8 照判；另一段 [20, 22) 照判。
    #[test]
    fn a_state_whose_recovery_panics_is_noted_and_the_states_after_it_are_still_judged() {
        let mut judged: Vec<u64> = Vec::new();
        let mut judge_run = |run: std::ops::Range<u64>| -> Result<(), u64> {
            for ordinal in run {
                if ordinal == 5 {
                    return Err(ordinal);
                }
                judged.push(ordinal);
            }
            Ok(())
        };
        let mut panicking: Vec<u64> = Vec::new();
        judge_runs_continuing_past_panicking_states(
            &[3..9, 20..22],
            &mut judge_run,
            &mut |ordinal| {
                panicking.push(ordinal);
            },
        );
        assert_eq!(judged, vec![3, 4, 6, 7, 8, 20, 21]);
        assert_eq!(panicking, vec![5]);
    }

    fn content_of(byte: u8) -> Vec<u8> {
        vec![byte; 10]
    }

    fn calls_for_the_reference_filesystem(third_call_outcome: CallOutcome) -> Vec<RecordedCall> {
        let call = |label: &'static str,
                    first_operation: usize,
                    return_point: usize,
                    outcome: CallOutcome,
                    content: Option<PublishedContent>| RecordedCall {
            label,
            first_operation,
            return_point,
            outcome,
            model: ModelVerdictOfCall::NotModelled,
            return_value: None,
            memory_objects: Vec::new(),
            published_content: content,
            observation_sha256: None,
        };
        vec![
            call("acquire", 0, 2, CallOutcome::Succeeded, None),
            call("warm_up", 2, 12, CallOutcome::Succeeded, None),
            call(
                "publish",
                12,
                40,
                CallOutcome::Succeeded,
                Some(PublishedContent::FirstFile(content_of(1))),
            ),
            call(
                "publish",
                40,
                70,
                third_call_outcome,
                Some(PublishedContent::Overwrite(content_of(2))),
            ),
        ]
    }

    /// 70 次写都在盘 0 上，下标 39 是一道屏障（关上第 0 段）：段由第 2 次调用关上。
    fn operations_closing_one_segment_at_39() -> Vec<RetainedOperation> {
        (0..70usize)
            .map(|index| RetainedOperation {
                operation: singlefs_harness::RecordedOperation {
                    device: DeviceIdentity(0),
                    kind: if index == 39 {
                        RecordedOperationKind::Barrier
                    } else {
                        RecordedOperationKind::Write
                    },
                    offset: DeviceOffsetInBytes(0),
                    length: 0,
                    content_hash: 0,
                },
                contents: None,
            })
            .collect()
    }

    /// RefFS 的崩溃时刻按「关上这一段的那道屏障由哪一步发出」归属（V7：归到下一次调用，这里红）：第 0 段由第 2 次调用关上，
    /// hi 是第 2 次调用里最后进序列的操作（发布调用，第 2 个），下界 lo 还是 0；读回「没有文件」（第 2 次调用之前那一版）判绿。
    /// RefFS 判出不在允许集合里的记红（V6：票恒为绿，这里红）。
    #[test]
    fn a_crash_in_the_segment_the_publish_closes_falls_inside_that_publish() {
        let stream = reference_filesystem_stream_of(
            &calls_for_the_reference_filesystem(CallOutcome::Succeeded),
            &operations_closing_one_segment_at_39(),
            &[],
            &[],
        );
        assert_eq!(stream.segment_closing_stream_indexes.len(), 2);
        let moment = reference_filesystem_crash_moment(&stream, Some(0));
        assert_eq!(
            moment,
            ReferenceFileSystemCrashMoment::InsideStep(StepPosition::Operation(2))
        );
        let no_file = ObservedReadBack::NoFile {
            root: model_root_key(InstanceGeneration(1), CheckpointTxg(2)),
        };
        assert_eq!(
            reference_filesystem_vote_of_a_state(&stream, moment, &no_file),
            ReferenceFileSystemVoteOfAState::Green {
                allowed: "S_0,S_2".to_string()
            }
        );
        let after_everything = reference_filesystem_crash_moment(&stream, None);
        assert_eq!(
            after_everything,
            ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep
        );
        assert!(matches!(
            reference_filesystem_vote_of_a_state(&stream, after_everything, &no_file),
            ReferenceFileSystemVoteOfAState::Red { .. }
        ));
    }

    /// 截断处在途的那次覆盖写按「写了之后报错」交（V8：记成「成」，下界抬到它之后，这里红）：崩溃在全部步之后，读回第 2 次调用那一版
    /// （第一个文件的内容）在允许集合里。
    #[test]
    fn an_overwrite_in_flight_at_the_truncation_leaves_the_confirmed_version_allowed() {
        let stream = reference_filesystem_stream_of(
            &calls_for_the_reference_filesystem(CallOutcome::InFlightAtTheTruncation),
            &operations_closing_one_segment_at_39(),
            &[],
            &[],
        );
        let read_back_of_the_confirmed_version = ObservedReadBack::FileRead {
            root: model_root_key(InstanceGeneration(1), CheckpointTxg(3)),
            content: content_of(1),
        };
        assert_eq!(
            reference_filesystem_vote_of_a_state(
                &stream,
                ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
                &read_back_of_the_confirmed_version
            ),
            ReferenceFileSystemVoteOfAState::Green {
                allowed: "S_2,S_4".to_string()
            }
        );
    }

    /// unlink 写了盘：翻译停在那一步，崩在它里面或之后的弃权，种类是「RefFS 第一版没有删除」（第七节 A5）。
    #[test]
    fn the_reference_filesystem_abstains_from_the_unlink_on() {
        let mut calls = calls_for_the_reference_filesystem(CallOutcome::Succeeded);
        calls[3].label = "unlink_intent";
        calls[3].published_content = None;
        let stream = reference_filesystem_stream_of(
            &calls,
            &operations_closing_one_segment_at_39(),
            &[],
            &[],
        );
        assert_eq!(
            reference_filesystem_vote_of_a_state(
                &stream,
                ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
                &ObservedReadBack::NoFile {
                    root: model_root_key(InstanceGeneration(1), CheckpointTxg(4))
                }
            ),
            ReferenceFileSystemVoteOfAState::Abstained {
                kind: "RefFS 第一版没有删除"
            }
        );
    }

    /// 新池新建文件之后的整份镜像（`tests/common` 的 `build_pool`，与流一全持久态同一份）与它只读恢复落到的版本。
    fn first_file_pool_and_its_selected_version(
        tag: &str,
    ) -> (MemoryPool, Option<(InstanceGeneration, CheckpointTxg)>) {
        let pool = common::build_pool(tag).memory_pool();
        let selected = singlefs_core::recovery::recover(
            &pool,
            singlefs_core::recovery::JournalPolicy::Consult,
        )
        .effective_root;
        (pool, selected)
    }

    /// 见证判据的两支：干净的池上两支都假、c_见证 是所选那一版末条记录的计数器；一块盘一槽系统配置撕裂（槽里一个字节翻一位、自证不过）记 (i)、
    /// 不记 (ii)；每块盘两槽的 journal_tail 抬过所选那一版的末条（阳性对照「见证」的造法）记 (ii) 第一句。
    #[test]
    fn a_torn_system_configuration_slot_is_the_undecidable_branch_and_a_raised_tail_is_the_counter_branch(
    ) {
        let (clean, selected) =
            first_file_pool_and_its_selected_version("e171r1-unit-witness-branches");
        let clean_criterion = witness_criterion_on_the_crash_image(&clean, selected);
        assert!(!clean_criterion.some_device_has_fewer_than_two_verified_slots);
        assert_eq!(clean_criterion.counter_sentence, None);
        let mut torn = clean.clone();
        let device = torn.devices.get_mut(&DeviceIdentity(1)).expect("盘 1");
        let spacing = u64::from(common::parameters().geometry.fixed_structure_slot_spacing);
        let mut slot_bytes = device.read(DeviceOffsetInBytes(spacing), 4096);
        slot_bytes[100] ^= 1;
        device.write(DeviceOffsetInBytes(spacing), &slot_bytes);
        let torn_criterion = witness_criterion_on_the_crash_image(&torn, selected);
        assert!(torn_criterion.some_device_has_fewer_than_two_verified_slots);
        assert_eq!(torn_criterion.counter_sentence, None);
        let mut raised = clean.clone();
        assert_eq!(
            raise_the_witnessed_counter_past_the_selected_version(&mut raised, selected),
            4
        );
        let raised_criterion = witness_criterion_on_the_crash_image(&raised, selected);
        assert_eq!(
            raised_criterion.counter_sentence,
            Some(WitnessCounterSentenceThatHolds::AboveTheLastRecordOfTheSelectedVersion)
        );
    }

    /// 每块盘两槽的 journal_tail 抬过所选那一版的末条之后，再撕盘 1 的一槽：只记 (i)、不记 (ii)，另三槽抬过的 tail 还在、c_见证 不变
    /// （登记修订 2 第 2.4 节；V11：判不出那一支并进 (ii)，三句的第一句为真，这里红）。
    #[test]
    fn a_torn_slot_beside_raised_tails_is_only_the_undecidable_branch() {
        let (clean, selected) =
            first_file_pool_and_its_selected_version("e171r1-unit-witness-torn-beside-raised");
        let mut raised = clean.clone();
        assert_eq!(
            raise_the_witnessed_counter_past_the_selected_version(&mut raised, selected),
            4
        );
        let raised_criterion = witness_criterion_on_the_crash_image(&raised, selected);
        let mut torn = raised.clone();
        let device = torn.devices.get_mut(&DeviceIdentity(1)).expect("盘 1");
        let spacing = u64::from(common::parameters().geometry.fixed_structure_slot_spacing);
        let mut slot_bytes = device.read(DeviceOffsetInBytes(spacing), 4096);
        slot_bytes[100] ^= 1;
        device.write(DeviceOffsetInBytes(spacing), &slot_bytes);
        let torn_criterion = witness_criterion_on_the_crash_image(&torn, selected);
        assert!(
            torn_criterion.some_device_has_fewer_than_two_verified_slots,
            "盘 1 撕了一槽，自证过的少于两槽：{torn_criterion:?}"
        );
        assert_eq!(
            torn_criterion.counter_sentence, None,
            "判不出那一支为真时计数那一支不判：{torn_criterion:?}"
        );
        assert_eq!(
            torn_criterion.witnessed_counter, raised_criterion.witnessed_counter,
            "另三槽抬过的 tail 还在，c_见证 与撕之前那份相同（取样点敏感）"
        );
    }

    /// 流二第 3 次调用 panic（阳性对照 panic 的造法）：整条流照常导出、取样态照判，不退回「整条流不导出」
    /// （登记修订 2 第 2.4 节；V4：panic 那一支退回不导出，这里红）。
    #[test]
    fn a_stream_stopped_by_a_panic_is_exported_and_its_sampled_states_are_judged() {
        let switches = RunSwitches {
            positive_control: Some(PositiveControl::PanicInTheCall),
            disabled_oracle: None,
            writable_mount_probe: false,
        };
        let stream = prepare_stream(StreamName::FixedScriptThroughEThenUnmount, switches, false);
        let directory =
            std::env::temp_dir().join(format!("e171r1-unit-panic-export-{}", std::process::id()));
        let export = export_prepared_stream(
            &stream,
            &directory,
            false,
            switches,
            StateSelection::RegisteredBatches { third_batch: false },
            Instant::now(),
            0,
            None,
        );
        std::fs::remove_dir_all(&directory)
            .unwrap_or_else(|error| panic!("删 {}：{error}", directory.display()));
        assert!(
            matches!(
                stream.ending,
                StreamEnding::StoppedByAPanic { call_index: 3, .. }
            ),
            "{:?}",
            stream.ending
        );
        assert_eq!(export.not_exported, None, "panic 截断的流照常导出");
        assert!(!export.observations.is_empty(), "取样态照判");
        assert_eq!(
            stream.calls[..PANIC_CONTROL_CALL]
                .iter()
                .filter(|call| call.model.was_judged_by_the_comparison())
                .count(),
            1,
            "panic 之前对拍判过的调用恰 1 次（第 2 次调用）"
        );
    }

    /// 阳性对照「见证」那一态上的探针：可写挂载拒、拒因是系统配置见证、第二截判不合法、没满足的条件是 C670 那一条（第七节 A4；
    /// V10：「没注入读故障」照「注入了」交，第二截判合法，这里红）。干净的池上可写挂载挂上。
    #[test]
    fn the_witness_control_is_refused_for_the_witness_and_judged_illegal_by_the_second_crop() {
        let (clean, selected) =
            first_file_pool_and_its_selected_version("e171r1-unit-witness-probe");
        let clean_probe = writable_mount_probe_on(&clean, selected, &CrashJudgeVerdict::Green);
        assert_eq!(clean_probe.mount, "mounted");
        let mut raised = clean.clone();
        raise_the_witnessed_counter_past_the_selected_version(&mut raised, selected);
        let probe = writable_mount_probe_on(&raised, selected, &CrashJudgeVerdict::Green);
        assert_eq!(probe.mount, "refused");
        assert!(
            probe.refused_for_the_system_configuration_witness,
            "{probe:?}"
        );
        assert!(
            probe.second_crop.starts_with("illegal:")
                && probe
                    .second_crop
                    .contains("witness_criterion_true_without_read_faults"),
            "{probe:?}"
        );
    }

    /// Q5 那一形的写流（读法 7；V12：根槽 FUA 不改成普通写，这里红）：删掉的屏障不在、根槽那次写改成普通写、截在截断处、写的字节一个不改。
    #[test]
    fn the_confirmed_loss_form_drops_the_barriers_makes_the_root_write_plain_and_truncates() {
        let operations: Vec<RetainedOperation> = (0..10usize)
            .map(|index| RetainedOperation {
                operation: singlefs_harness::RecordedOperation {
                    device: DeviceIdentity(0),
                    kind: match index {
                        3 | 5 | 8 => RecordedOperationKind::Barrier,
                        6 => RecordedOperationKind::WriteForceUnitAccess,
                        _ => RecordedOperationKind::Write,
                    },
                    offset: DeviceOffsetInBytes(u64::try_from(index).expect("下标") * 4096),
                    length: 4096,
                    content_hash: u64::try_from(index).expect("下标"),
                },
                contents: Some(vec![u8::try_from(index).expect("下标"); 4]),
            })
            .collect();
        let shape = ConfirmedLossShape {
            removed_barriers: vec![3, 5],
            root_writes_made_plain: vec![6],
            truncation: 8,
            reverse_truncation: 7,
        };
        let (kept, new_index_of) = confirmed_loss_operations(&operations, &shape, false);
        assert_eq!(kept.len(), 6);
        assert_eq!(
            new_index_of,
            vec![
                Some(0),
                Some(1),
                Some(2),
                None,
                Some(3),
                None,
                Some(4),
                Some(5)
            ]
        );
        assert_eq!(kept[4].operation.kind, RecordedOperationKind::Write);
        assert_eq!(kept[4].contents, operations[6].contents);
        assert!(kept
            .iter()
            .all(|retained| retained.operation.kind != RecordedOperationKind::Barrier));
        let (reverse, _) = confirmed_loss_operations(&operations, &shape, true);
        assert_eq!(reverse.len(), 5);
    }

    /// 内联用例的对拍流截到第 3 次调用发出的前几个写（修订 5 第 5.2 节：U1–U6 与已有的两条留在 k = 2，288 态；装置那一跑是
    /// [`EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT`]）。
    const IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS: usize = 2;

    /// 「等价类对拍」那一形的流（流二同一种造法），截到第 3 次调用发出的前 `in_flight_writes_kept` 个写之后。
    fn equivalence_pairing_stream_keeping(in_flight_writes_kept: usize) -> PreparedStream {
        confirmed_loss_prepared_stream(
            second_stream_through_the_first_overwrite(RunSwitches::default()),
            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(in_flight_writes_kept),
        )
        .0
    }

    /// 「等价类对拍」那一形的流，截到第 3 次调用发出的前 2 个写之后（72 × 4 = 288 态）。
    fn equivalence_pairing_stream() -> PreparedStream {
        equivalence_pairing_stream_keeping(IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS)
    }

    /// 等价关系（登记修订 2 第 2.1 节候选二「用例钉住等价关系」）：288 态那一形上全枚举逐态判与按等价类推，每态的恢复报告、两个崩溃子列
    /// 与 RefFS 一票逐态相同，类核对 ①–④ 都过；模型照常那一遍与 `model_stopped_at_call` 设成第 3 次调用那一遍各做一遍（V17：键去掉
    /// 「C 至少落一个」那一位，第二遍里「C 落了」的态模型崩溃判官不适用、代表上适用，这里红）。各类态数之和等于末段每个写的取法数之积，
    /// 主取样点上（不判态，只分类）是 72 × 2²⁴（V19：「C 至少落一个」那一类的态数写成 2 的 C 写数次方，这里红）。
    #[test]
    fn equivalence_classes_reproduce_every_state_with_and_without_the_model_stopped_at_the_in_flight_call(
    ) {
        let mut stream = equivalence_pairing_stream();
        let pairing = equivalence_pairing_of(&stream);
        assert_eq!(pairing.states, 288, "72 × 2² 态全枚举");
        assert_eq!(
            pairing.class_states, pairing.product_of_the_choices,
            "各类态数之和等于末段每个写的取法数之积（A10）"
        );
        assert_eq!(pairing.classes, 144, "72 种取法 × C 落没落");
        assert_eq!(
            pairing.classification.refinements, 0,
            "C 的写不在代表的读段里"
        );
        assert_eq!(
            pairing.checks,
            EquivalenceCheckCounts {
                input_list_mismatches: 0,
                in_flight_writes_of_a_forbidden_kind: 0,
                recording_report_mismatches: 0,
                derived_class_mismatches: 0,
                class_landing_mismatches: 0,
                in_flight_write_kinds: BTreeMap::from([("unit_write", 2)]),
            },
            "类核对 ①、②、④ 都过"
        );
        assert_eq!(pairing.mismatching_states, 0, "模型照常：逐态相同");
        assert!(!pairing.stops_at_s13());
        stream.model_stopped_at_call = Some(CONFIRMED_LOSS_IN_FLIGHT_CALL);
        let stopped = equivalence_pairing_of(&stream);
        let not_applicable_states = stopped
            .observations
            .iter()
            .filter(|observation| observation.crash_judge == CrashJudgeVerdict::NotApplicable)
            .count();
        assert_eq!(
            not_applicable_states,
            288 - 72,
            "模型停在第 3 次调用：C 落了的态（每种取法 3 态）模型崩溃判官不适用，代表上适用（取样点敏感）"
        );
        assert_eq!(
            stopped.mismatching_states, 0,
            "模型停在第 3 次调用：逐态相同"
        );
        assert!(!stopped.stops_at_s13());
        let (main, _) = confirmed_loss_prepared_stream(
            second_stream_through_the_first_overwrite(RunSwitches::default()),
            ConfirmedLossTruncation::BeforeTheInFlightBarrier,
        );
        let main_tables = ObservationTables::of(&main);
        let main_context = main_tables.context(&main);
        let main_digits = LastSegmentDigits::of(&main, &main_tables.tables);
        let (class_positions, keyed_positions): (Vec<usize>, Vec<usize>) =
            (0..main_digits.write_indexes.len()).partition(|position| {
                main_context.call_of_write[main_digits.write_indexes[*position]]
                    == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL)
            });
        let main_partition = ConfirmedLossPartition {
            keyed_positions,
            class_positions,
        };
        assert_eq!(
            main_partition.keyed_assignments(&main_digits)
                * (1 + main_partition.states_with_the_in_flight_writes_landed(&main_digits)),
            LOCAL_CONFIRMED_LOSS_STATES,
            "主取样点各类态数之和是 72 × 2²⁴"
        );
    }

    /// 类核对 ③ 细分（登记修订 2 第 2.1 节；V18：③ 恒判不相交，这里红）：288 态那一形上把第 3 次调用的第 1 个写挪到第 2 次调用第一个
    /// journal 记录写的同一个落点（手造写流），代表上的恢复读到它的字节段，那个写移进键（细分一次）；那个写落没落，恢复有不同的态
    /// （取样点敏感）；细分之后按等价类推的与全枚举逐态相同。
    #[test]
    fn an_in_flight_write_over_a_journal_record_of_the_confirmed_call_is_moved_into_the_key() {
        let mut stream = equivalence_pairing_stream();
        let call_of_write = call_index_of_each_write(&stream);
        let last_segment = stream.segments.last().cloned().expect("末段");
        let journal_record_write = last_segment
            .iter()
            .copied()
            .find(|write_index| {
                call_of_write[*write_index] == Some(CONFIRMED_LOSS_CONFIRMED_CALL)
                    && stream.writes[*write_index].kind == StepKind::JournalRecord
            })
            .expect("第 2 次调用在末段里写 journal 记录");
        let first_in_flight_position = last_segment
            .iter()
            .position(|write_index| {
                call_of_write[*write_index] == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL)
            })
            .expect("末段里有第 3 次调用的写");
        let first_in_flight_write = last_segment[first_in_flight_position];
        stream.writes[first_in_flight_write].device = stream.writes[journal_record_write].device;
        stream.writes[first_in_flight_write].offset = stream.writes[journal_record_write].offset;
        let pairing = equivalence_pairing_of(&stream);
        assert_eq!(pairing.states, 288);
        assert_eq!(
            pairing.classification.moved_positions,
            vec![first_in_flight_position],
            "压在 journal 记录上的那个写移进键"
        );
        assert_eq!(pairing.classification.refinements, 1);
        let tables = ObservationTables::of(&stream);
        let digits = LastSegmentDigits::of(&stream, &tables.tables);
        let weight = digits.weights[first_in_flight_position];
        let observation_of_ordinal: BTreeMap<u64, &StateObservation> = pairing
            .observations
            .iter()
            .map(|observation| (observation.ordinal, observation))
            .collect();
        let differing_when_that_write_lands = pairing
            .observations
            .iter()
            .filter(|observation| {
                digits.digit_of(observation.ordinal, first_in_flight_position) == 1
            })
            .filter(|observation| {
                observation_of_ordinal[&(observation.ordinal - weight)].consulted_report
                    != observation.consulted_report
            })
            .count();
        assert!(
            differing_when_that_write_lands > 0,
            "那个写落没落，有恢复不同的态"
        );
        assert_eq!(pairing.mismatching_states, 0, "细分之后逐态相同");
        assert!(!pairing.stops_at_s13());
    }

    /// 流三照读法 2 驱动：删第一个文件（意图一次、走空一步）、一次建一个 inode 到叶容器第一次变成两片，再多建一次
    /// （V9：停在叶容器变成两片的那一次（第 K 次），少建之后那一次，这里红）。取号、暖机之外的每次调用都对拍过、对得上，变成两片的那一次与之后那一次也是
    /// （理想模型那件修理之后胶水按片记叶容器的角色，登记修订 2 第 2.2 节）：没有一次「模型接不了这份观测」，模型一次都不停。
    #[test]
    fn the_third_stream_stops_one_creation_after_the_inode_leaf_splits() {
        let mut live = LiveStream::start(
            StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf,
            "e171r1-unit-third-stream",
            RunSwitches::default(),
        );
        live.run_the_script();
        let split = live.split_publish.expect("叶容器变成了两片");
        assert_eq!(live.inode_creations, split + 1);
        assert_eq!(
            live.output
                .as_ref()
                .map(|output| output.inode_leaf_containers.len()),
            Some(2)
        );
        assert_eq!(live.unlink_calls, 2);
        assert!(live
            .driver
            .calls
            .iter()
            .all(|call| call.outcome == CallOutcome::Succeeded));
        assert!(!live.driver.calls.iter().any(|call| {
            matches!(
                call.model,
                ModelVerdictOfCall::ModelCannotTakeTheObservation { .. }
            )
        }));
        assert_eq!(live.driver.model_stopped_at_call, None);
        assert!(live.driver.calls.iter().all(|call| {
            call.model == ModelVerdictOfCall::Agrees
                || call.label == "acquire"
                || call.label == "warm_up"
        }));
    }

    /// 流二的卸载经模型的卸载答对拍：基线上那一次 `agrees`；阳性对照「O1-对拍-卸载」把观测里第一条根的 F 加一，恰那一次 `disagrees`
    /// （V1：卸载那一步不调模型的卸载答，这里红）。
    #[test]
    fn the_unmount_is_compared_with_the_model_and_its_control_turns_only_the_unmount_red() {
        let mut clean = LiveStream::start(
            StreamName::FixedScriptThroughEThenUnmount,
            "e171r1-unit-unmount-clean",
            RunSwitches::default(),
        );
        clean.run_the_script();
        let unmount_of = |live: &LiveStream| -> ModelVerdictOfCall {
            live.driver
                .calls
                .iter()
                .find(|call| call.label == UNMOUNT_CALL_LABEL)
                .expect("流二走到了卸载")
                .model
                .clone()
        };
        assert_eq!(unmount_of(&clean), ModelVerdictOfCall::Agrees);
        assert_eq!(clean.driver.model_stopped_at_call, None);
        let mut controlled = LiveStream::start(
            StreamName::FixedScriptThroughEThenUnmount,
            "e171r1-unit-unmount-control",
            RunSwitches {
                positive_control: Some(PositiveControl::ComparisonOfTheUnmount),
                disabled_oracle: None,
                writable_mount_probe: false,
            },
        );
        controlled.run_the_script();
        assert!(matches!(
            unmount_of(&controlled),
            ModelVerdictOfCall::Disagrees { .. }
        ));
        assert_eq!(
            controlled
                .driver
                .calls
                .iter()
                .filter(|call| matches!(call.model, ModelVerdictOfCall::Disagrees { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn an_error_member_is_the_first_identifier_of_its_debug_text() {
        #[derive(Debug)]
        #[allow(
            dead_code,
            reason = "字段只经 Debug 文本读，测的就是 Debug 文本的第一个标识符"
        )]
        enum Failure {
            Plain,
            WithFields { reason: u8 },
            Tuple(u8),
        }
        assert_eq!(error_member_of(&Failure::Plain), "Plain");
        assert_eq!(
            error_member_of(&Failure::WithFields { reason: 1 }),
            "WithFields"
        );
        assert_eq!(error_member_of(&Failure::Tuple(2)), "Tuple");
    }

    #[test]
    fn a_call_outcome_writes_ok_or_err_into_the_calls_file_and_a_word_into_the_outcomes_file() {
        assert_eq!(CallOutcome::Succeeded.calls_file_word(), "ok");
        assert_eq!(
            CallOutcome::ReturnedError("NoSpace".to_string()).calls_file_word(),
            "err"
        );
        assert_eq!(
            CallOutcome::ReturnedError("NoSpace".to_string()).outcome_word(),
            "err:NoSpace"
        );
        assert_eq!(
            CallOutcome::Panicked("炸".to_string()).outcome_word(),
            "panic"
        );
        assert_eq!(
            CallOutcome::Panicked("炸".to_string()).calls_file_word(),
            "err"
        );
        assert_eq!(
            CallOutcome::InFlightAtTheTruncation.outcome_word(),
            "in_flight"
        );
    }

    #[test]
    fn stream_numbers_select_the_streams_in_order_and_reject_unknown_numbers() {
        assert_eq!(
            stream_names_of_the_numbers("3,1"),
            Some(vec![
                StreamName::NewPoolFileCreation,
                StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf
            ])
        );
        assert_eq!(stream_names_of_the_numbers("4"), None);
        assert_eq!(stream_names_of_the_numbers(""), None);
    }

    /// 修订 5 第 5.4 节 U1、U4、U5、U6 的共同起点「S0」：k = 2 的对拍流上逐态全判 288 态、分好类（[`equivalence_pairing_of`]）。
    struct UntamperedPairing {
        stream: PreparedStream,
        tables: ObservationTables,
        pairing: EquivalencePairing,
    }

    /// 类核对 ①、②、④ 两半、落地串核对五个计数（按这个次序）。
    fn five_class_check_counts(checks: &EquivalenceCheckCounts) -> [usize; 5] {
        [
            checks.input_list_mismatches,
            checks.in_flight_writes_of_a_forbidden_kind,
            checks.recording_report_mismatches,
            checks.derived_class_mismatches,
            checks.class_landing_mismatches,
        ]
    }

    /// 照 [`evaluate_the_classes`] 在 S0 的（可能篡改过的）观测与分法上重做类核对。
    fn class_checks_of(fixture: &UntamperedPairing) -> EquivalenceCheckCounts {
        let context = fixture.tables.context(&fixture.stream);
        let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
        evaluate_the_classes(
            &context,
            &digits,
            &fixture.pairing.classification,
            &fixture.pairing.observations,
            CONFIRMED_LOSS_IN_FLIGHT_CALL,
        )
        .checks
    }

    /// 造 S0，并断言不篡改时类核对五个计数全 0、不停在 S13（证明之后那一处篡改才是让它红的东西）。
    fn untampered_pairing() -> UntamperedPairing {
        let stream = equivalence_pairing_stream();
        let tables = ObservationTables::of(&stream);
        let pairing = equivalence_pairing_of(&stream);
        let fixture = UntamperedPairing {
            stream,
            tables,
            pairing,
        };
        let checks = class_checks_of(&fixture);
        assert_eq!(
            five_class_check_counts(&checks),
            [0; 5],
            "不篡改时类核对五个计数全 0"
        );
        assert!(
            !equivalence_stops_at_s13(&fixture.pairing.classification, &checks),
            "不篡改时不停在 S13"
        );
        fixture
    }

    /// S0 里崩溃状态序号是 `ordinal` 的那一态在观测里的下标。
    fn observation_index_of(fixture: &UntamperedPairing, ordinal: u64) -> usize {
        fixture
            .pairing
            .observations
            .iter()
            .position(|observation| observation.ordinal == ordinal)
            .expect("对拍逐态全判：每个序号都有观测")
    }

    /// 末段里第一个属于第 3 次调用的写（c1）：写表下标与它在末段里的位置。
    fn first_in_flight_write_of(stream: &PreparedStream) -> (usize, usize) {
        let call_of_write = call_index_of_each_write(stream);
        let last_segment = stream.segments.last().cloned().expect("末段");
        let position = last_segment
            .iter()
            .position(|write_index| {
                call_of_write[*write_index] == Some(CONFIRMED_LOSS_IN_FLIGHT_CALL)
            })
            .expect("末段里有第 3 次调用的写");
        (last_segment[position], position)
    }

    /// U1（修订 5 第 5.4 节，KM2；V20：类核对 ① 不计数，这里红）：S0，代表 0 的观测里 O1-模型崩溃判官换掉，① 数到这一态、停在 S13。
    #[test]
    fn the_input_list_check_counts_a_state_whose_columns_do_not_follow_the_four_inputs() {
        let mut fixture = untampered_pairing();
        let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
        let representative = fixture
            .pairing
            .classification
            .partition
            .representative_ordinal(&digits, 0);
        let index = observation_index_of(&fixture, representative);
        let tampered = match fixture.pairing.observations[index].crash_judge {
            CrashJudgeVerdict::Green => CrashJudgeVerdict::Red {
                what: "tampered".to_string(),
            },
            CrashJudgeVerdict::NotApplicable
            | CrashJudgeVerdict::Red { .. }
            | CrashJudgeVerdict::Off
            | CrashJudgeVerdict::RecoveryPanicked
            | CrashJudgeVerdict::JudgePanicked => CrashJudgeVerdict::Green,
        };
        fixture.pairing.observations[index].crash_judge = tampered;
        let checks = class_checks_of(&fixture);
        assert_eq!(checks.input_list_mismatches, 1, "① 数到代表 0 那一态");
        assert!(equivalence_stops_at_s13(
            &fixture.pairing.classification,
            &checks
        ));
    }

    /// U2（修订 5 第 5.4 节，KM1；V21：类核对 ② 不计数，这里红）：c1 那条操作的盘与偏移改成第 2 次调用末段里第一个 journal 记录写的，
    /// 写表不动（③ 看不见它）：② 数到它、停在 S13。
    #[test]
    fn an_in_flight_write_classified_as_a_journal_record_is_a_forbidden_kind() {
        let mut stream = equivalence_pairing_stream();
        let call_of_write = call_index_of_each_write(&stream);
        let last_segment = stream.segments.last().cloned().expect("末段");
        let journal_record_write = last_segment
            .iter()
            .copied()
            .find(|write_index| {
                call_of_write[*write_index] == Some(CONFIRMED_LOSS_CONFIRMED_CALL)
                    && stream.writes[*write_index].kind == StepKind::JournalRecord
            })
            .expect("第 2 次调用在末段里写 journal 记录");
        let (first_in_flight_write, _) = first_in_flight_write_of(&stream);
        let journal_record_operation = stream.operations
            [stream.stream_indexes[journal_record_write]]
            .operation
            .clone();
        let in_flight_operation_index = stream.stream_indexes[first_in_flight_write];
        stream.operations[in_flight_operation_index]
            .operation
            .device = journal_record_operation.device;
        stream.operations[in_flight_operation_index]
            .operation
            .offset = journal_record_operation.offset;
        assert_eq!(
            common::geometry().classify(&stream.operations[in_flight_operation_index].operation),
            StepKind::JournalRecord,
            "改过的那条操作经 geometry.classify 是 journal 记录写"
        );
        let pairing = equivalence_pairing_of(&stream);
        assert_eq!(pairing.checks.in_flight_writes_of_a_forbidden_kind, 1);
        assert_eq!(
            pairing.checks.in_flight_write_kinds,
            BTreeMap::from([("journal_record", 1), ("unit_write", 1)])
        );
        assert_eq!(
            pairing.classification.refinements, 0,
            "写表没动，③ 看不见它，只有 ② 抓"
        );
        assert!(equivalence_stops_at_s13(
            &pairing.classification,
            &pairing.checks
        ));
    }

    /// 与写对齐的粒度（U3 把 c1 挪到环尾时按它向下对齐）。
    const WRITE_ALIGNMENT_IN_BYTES: u64 = 4096;

    /// U3（修订 5 第 5.4 节，KM7；V22：journal 提示不记整环，这里红）：c1 挪到盘 0 journal 环尾、只有提示那一段罩得到的位置，
    /// ③ 经提示那一段把它移进键（细分一次），细分之后逐态相同。
    #[test]
    fn an_in_flight_write_in_an_unread_journal_ring_slot_is_moved_into_the_key_through_the_hint() {
        let mut stream = equivalence_pairing_stream();
        let (first_in_flight_write, first_in_flight_position) = first_in_flight_write_of(&stream);
        let length = stream.writes[first_in_flight_write].length_in_bytes();
        let (device_zero, landing_offset) = {
            let tables = ObservationTables::of(&stream);
            let context = tables.context(&stream);
            let digits = LastSegmentDigits::of(&stream, &tables.tables);
            let classification = classify_the_last_segment(
                &context,
                &digits,
                CONFIRMED_LOSS_IN_FLIGHT_CALL,
                CONFIRMED_LOSS_STATE_LIMIT,
            );
            let reads_of_the_representative = |keyed_assignment: u64| {
                let image = CrashImage {
                    base: &stream.base,
                    writes: &tables.tables.enumerated_writes,
                    persisted: tables.facts.persisted_writes_of_state(
                        classification
                            .partition
                            .representative_ordinal(&digits, keyed_assignment),
                    ),
                };
                consulted_recovery_recording_the_reads(&image).1
            };
            let ring_hint = reads_of_the_representative(0)
                .into_iter()
                .filter(|read| read.source == RecordedReadSource::JournalRecordOffsetsHint)
                .min_by_key(|read| read.device.0)
                .expect("代表 0 的恢复经提示记了 journal 环");
            let landing_offset = (ring_hint.end_in_bytes - length) / WRITE_ALIGNMENT_IN_BYTES
                * WRITE_ALIGNMENT_IN_BYTES;
            for keyed_assignment in 0..classification.partition.keyed_assignments(&digits) {
                assert!(
                    reads_of_the_representative(keyed_assignment)
                        .iter()
                        .filter(|read| read.source == RecordedReadSource::Read
                            && read.device == ring_hint.device)
                        .all(|read| read.end_in_bytes <= landing_offset
                            || landing_offset + length <= read.start_in_bytes),
                    "前提：第 {keyed_assignment} 种取法的代表上来自 read 的段不罩 [{landing_offset}, {})，只有提示那一段罩得到",
                    landing_offset + length
                );
            }
            (ring_hint.device, landing_offset)
        };
        stream.writes[first_in_flight_write].device = device_zero;
        stream.writes[first_in_flight_write].offset = DeviceOffsetInBytes(landing_offset);
        let pairing = equivalence_pairing_of(&stream);
        assert_eq!(
            pairing.classification.moved_positions,
            vec![first_in_flight_position],
            "环尾那个写经提示那一段移进键"
        );
        assert_eq!(pairing.classification.refinements, 1);
        assert_eq!(pairing.mismatching_states, 0, "细分之后逐态相同");
    }

    /// U4（修订 5 第 5.4 节，KM4；V23：类核对 ④ 前一半不计数，这里红）：S0，代表 0 的记读那一遍恢复换成 panic，④ 前一半数到它。
    #[test]
    fn a_recording_recovery_that_differs_from_the_crash_point_replay_report_is_counted() {
        let mut fixture = untampered_pairing();
        let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
        let representative = fixture
            .pairing
            .classification
            .partition
            .representative_ordinal(&digits, 0);
        fixture
            .pairing
            .classification
            .representative_recoveries
            .insert(
                representative,
                ConsultedRecovery::Panicked("tampered".to_string()),
            );
        let checks = class_checks_of(&fixture);
        assert_eq!(checks.recording_report_mismatches, 1, "④ 前一半数到代表 0");
        assert!(equivalence_stops_at_s13(
            &fixture.pairing.classification,
            &checks
        ));
    }

    /// U5（修订 5 第 5.4 节，KM3；V24：类核对 ④ 后一半不计数，这里红）：S0，全落 0 的观测里 RefFS 一票换掉，④ 后一半数到它（① 同时数到）。
    #[test]
    fn a_fully_landed_state_that_differs_from_its_derived_class_is_counted() {
        let mut fixture = untampered_pairing();
        let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
        let fully_landed = fixture
            .pairing
            .classification
            .partition
            .fully_landed_ordinal(&digits, 0);
        let index = observation_index_of(&fixture, fully_landed);
        let tampered = match fixture.pairing.observations[index].reference_filesystem {
            ReferenceFileSystemVoteOfAState::RecoveryPanicked => {
                ReferenceFileSystemVoteOfAState::Green {
                    allowed: String::new(),
                }
            }
            ReferenceFileSystemVoteOfAState::Green { .. }
            | ReferenceFileSystemVoteOfAState::Red { .. }
            | ReferenceFileSystemVoteOfAState::Abstained { .. }
            | ReferenceFileSystemVoteOfAState::JudgePanicked
            | ReferenceFileSystemVoteOfAState::Off => {
                ReferenceFileSystemVoteOfAState::RecoveryPanicked
            }
        };
        fixture.pairing.observations[index].reference_filesystem = tampered;
        let checks = class_checks_of(&fixture);
        assert_eq!(checks.derived_class_mismatches, 1, "④ 后一半数到全落 0");
        assert_eq!(checks.input_list_mismatches, 1, "① 同时数到这一态");
    }

    /// U6（修订 5 第 5.4 节，KM5、KM6；V25、V26：代表、全落那一态的落地串核对恒过，V27：S13 不看落地串核对，这里红）：S0，
    /// (a) 代表 0 的落地串里 c1 那一位改成 `2`；(b) 还原后全落 0 的落地串里 c1 那一位改成 `0`。各自只有落地串核对数到 1、停在 S13。
    #[test]
    fn a_landing_string_off_the_partition_is_counted_and_stops_at_s13() {
        let mut fixture = untampered_pairing();
        let digits = LastSegmentDigits::of(&fixture.stream, &fixture.tables.tables);
        let (first_in_flight_write, _) = first_in_flight_write_of(&fixture.stream);
        let representative = fixture
            .pairing
            .classification
            .partition
            .representative_ordinal(&digits, 0);
        let fully_landed = fixture
            .pairing
            .classification
            .partition
            .fully_landed_ordinal(&digits, 0);
        for (ordinal, tampered_landing) in [(representative, "2"), (fully_landed, "0")] {
            let index = observation_index_of(&fixture, ordinal);
            let original = fixture.pairing.observations[index].landing_vector.clone();
            fixture.pairing.observations[index]
                .landing_vector
                .replace_range(
                    first_in_flight_write..first_in_flight_write + 1,
                    tampered_landing,
                );
            let checks = class_checks_of(&fixture);
            assert_eq!(
                five_class_check_counts(&checks),
                [0, 0, 0, 0, 1],
                "序号 {ordinal} 的 c1 那一位改成 {tampered_landing}：只有落地串核对数到它"
            );
            assert!(
                equivalence_stops_at_s13(&fixture.pairing.classification, &checks),
                "S13 只能从落地串核对来"
            );
            fixture.pairing.observations[index].landing_vector = original;
        }
    }

    /// U7（修订 5 第 5.7 节，第 5.1 节甲；V28：一件 panic 两列都记恢复 panic，V29：`pool_checker` 那一件 panic 不记件名，这里红）：
    /// k = 4 的对拍流（1 152 态，c1…c4 两对镜像），注入 `pool_checker`（c3 落、c4 没落）、`ignored_recovery`（c3 没落、c4 落）、
    /// `record_checker`（c1 没落、c2 落、(c3,c4) 同落或同不落）：各件的态数恰为 288、288、144，两个崩溃子列与看 journal 那遍恢复逐态不变。
    #[test]
    fn a_panic_off_the_consulted_recovery_leaves_the_two_crash_columns_to_the_four_inputs() {
        let stream = equivalence_pairing_stream_keeping(4);
        let tables = ObservationTables::of(&stream);
        let facts = equivalence_pairing_shape_facts(&stream, &tables.tables);
        let [first, second, third, fourth] = facts.in_flight_writes[..] else {
            panic!(
                "k = 4 的对拍流末段里第 3 次调用恰有 4 个写：{:?}",
                facts.in_flight_writes
            );
        };
        let injection = PiecePanicInjection {
            rules: vec![
                PiecePanicInjectionRule {
                    piece: Some(JudgingPiece::PoolChecker),
                    conditions: vec![
                        InjectedLandingCondition::Landed(third),
                        InjectedLandingCondition::NotLanded(fourth),
                    ],
                },
                PiecePanicInjectionRule {
                    piece: Some(JudgingPiece::IgnoredRecovery),
                    conditions: vec![
                        InjectedLandingCondition::NotLanded(third),
                        InjectedLandingCondition::Landed(fourth),
                    ],
                },
                PiecePanicInjectionRule {
                    piece: Some(JudgingPiece::RecordChecker),
                    conditions: vec![
                        InjectedLandingCondition::NotLanded(first),
                        InjectedLandingCondition::Landed(second),
                        InjectedLandingCondition::BothOrNeitherLanded(third, fourth),
                    ],
                },
            ],
        };
        let plain = equivalence_pairing_of(&stream);
        assert_eq!(
            piece_panic_states_of(&plain.observations),
            [0; JUDGING_PIECE_COUNT],
            "不注入那一遍各件 panic 都是 0（前提）"
        );
        let injected = equivalence_pairing_with_injected_piece_panics(&stream, Some(&injection));
        let two_choices_each_of_two_writes =
            usize::try_from(LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2 * 2).expect("288");
        let two_choices_of_one_write =
            usize::try_from(LOCAL_CONFIRMED_LOSS_KEY_CHOICES * 2).expect("144");
        let mut expected = [0usize; JUDGING_PIECE_COUNT];
        expected[JudgingPiece::PoolChecker.position()] = two_choices_each_of_two_writes;
        expected[JudgingPiece::IgnoredRecovery.position()] = two_choices_each_of_two_writes;
        expected[JudgingPiece::RecordChecker.position()] = two_choices_of_one_write;
        assert_eq!(
            piece_panic_states_of(&injected.observations),
            expected,
            "注入的三件各 288、288、144 态，别的件 0"
        );
        assert_eq!(
            o1_columns_differing(&plain.observations, &injected.observations),
            0,
            "逐态两个崩溃子列与看 journal 那遍恢复的结局与不注入那一遍相同"
        );
        assert!(injected
            .observations
            .iter()
            .all(|observation| observation.recovery_panic.is_none()));
        assert_eq!(injected.mismatching_states, 0);
        assert!(!injected.stops_at_s13());
        let pool_checker_panicking: Vec<&StateObservation> = injected
            .observations
            .iter()
            .filter(|observation| {
                observation
                    .piece_panics
                    .contains(&JudgingPiece::PoolChecker)
            })
            .collect();
        assert_eq!(pool_checker_panicking.len(), two_choices_each_of_two_writes);
        assert!(pool_checker_panicking.iter().all(|observation| {
            observation.pool_checker_red == Some(vec!["pool_checker_panic".to_string()])
        }));
    }

    /// U8 用的两形：主取样点那一形与对拍那一形（照 `run_confirmed_loss_form` 造）。
    fn confirmed_loss_shapes_for_the_test() -> ConfirmedLossShapes {
        let (main, shape) = confirmed_loss_prepared_stream(
            second_stream_through_the_first_overwrite(RunSwitches::default()),
            ConfirmedLossTruncation::BeforeTheInFlightBarrier,
        );
        ConfirmedLossShapes {
            sampling_point: "main_before_the_first_barrier_of_call_3",
            main,
            shape,
            drive_nanoseconds: 0,
            pairing: Some(equivalence_pairing_stream_keeping(
                EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT,
            )),
            piece_panic_injections: ConfirmedLossPiecePanicInjections {
                pairing: None,
                piece_panic_control: None,
                sampling_point_export: None,
            },
        }
    }

    /// U8（修订 5 第 5.7 节，第 5.3 节；V30：根写的前置判删掉、退回先对拍，这里红）：主取样点那一形、对拍那一形各模拟一次没有根写
    /// （只把 `judged_root_index` 设成 `usize::MAX`，写流不变），都在对拍之前退 3、记 `q5_not_run=no_root_write`。
    #[test]
    fn a_shape_without_a_root_write_stops_before_the_pairing() {
        let command_line = ConfirmedLossCommandLine {
            output_directory: std::env::temp_dir().join("e171r1-unit-no-root-write-not-exported"),
            truncate_at_the_return_point: false,
        };
        enum ShapeWithoutARootWrite {
            Main,
            Pairing,
        }
        for shape_without_a_root in [
            ShapeWithoutARootWrite::Main,
            ShapeWithoutARootWrite::Pairing,
        ] {
            let mut shapes = confirmed_loss_shapes_for_the_test();
            let shape_name = match shape_without_a_root {
                ShapeWithoutARootWrite::Main => {
                    shapes.main.judged_root_index = usize::MAX;
                    "main"
                }
                ShapeWithoutARootWrite::Pairing => {
                    shapes
                        .pairing
                        .as_mut()
                        .expect("主取样点那一跑有对拍那一形")
                        .judged_root_index = usize::MAX;
                    "pairing"
                }
            };
            let mut lines = ResultLines {
                emitted: 0,
                collected: Some(Vec::new()),
            };
            let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
            let collected = lines.collected.expect("收集模式");
            assert_eq!(exit_code, 3, "{shape_name}：退 3");
            assert_eq!(
                collected
                    .iter()
                    .filter(|line| line.starts_with("name=q5 ")
                        && line.contains(" not_run=no_root_write")
                        && line.contains(&format!(" shape={shape_name}")))
                    .count(),
                1,
                "{shape_name}：一行 name=q5 … not_run=no_root_write shape={shape_name}：{collected:?}"
            );
            assert!(
                collected
                    .iter()
                    .any(|line| line.starts_with("name=verdict ")
                        && line.contains(" s13=false")
                        && line.contains(" q5_not_run=no_root_write")),
                "{shape_name}：判决行带 s13=false 与 q5_not_run=no_root_write：{collected:?}"
            );
            assert!(
                !collected.iter().any(|line| {
                    line.starts_with("name=q5_equivalence_pairing")
                        || line.starts_with("name=q5_piece_panic_control")
                        || line.starts_with("name=q5_equivalence")
                }),
                "{shape_name}：对拍、阳性对照与类核对一行都没打：{collected:?}"
            );
        }
    }

    /// U9（修订 5 第 5.7 节，第 5.2 节丙；V31：k 由 6 改回 2，这里红）：k = [`EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT`] 的对拍流，
    /// 末段各写取法数之积 4 608、第 3 次调用的写 6 个、c1…c6 三对镜像（偏移互不相同）。只造末段的进制，不判态。
    #[test]
    fn the_pairing_keeps_the_first_three_mirrored_pairs_of_the_in_flight_call() {
        let stream = equivalence_pairing_stream_keeping(EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES_KEPT);
        let tables = crash_state_enumeration_tables(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        );
        let facts = equivalence_pairing_shape_facts(&stream, &tables);
        let digits = LastSegmentDigits::of(&stream, &tables);
        assert_eq!(
            digits.product_of_the_radixes(&(0..digits.write_indexes.len()).collect::<Vec<usize>>()),
            LOCAL_EQUIVALENCE_PAIRING_STATES,
            "末段各写取法数之积是 72 × 2⁶"
        );
        assert_eq!(facts.states, LOCAL_EQUIVALENCE_PAIRING_STATES);
        assert_eq!(
            facts.in_flight_writes.len(),
            LOCAL_EQUIVALENCE_PAIRING_IN_FLIGHT_WRITES,
            "末段里属于第 3 次调用的写 6 个"
        );
        assert_eq!(
            facts.mirrored_pairs, LOCAL_EQUIVALENCE_PAIRING_MIRRORED_PAIRS,
            "c1…c6 两两同偏移、同长度、分属两块盘，三对的偏移互不相同"
        );
    }

    /// 登记「主 agent 认定：27-2 装置交回」第 2 条（变异「unattributed_panic_states 恒 0」在这里红）：对拍流（k = 2）末段第一态上注入一条
    /// 不点名件的规则，只让正常那一路的回调 panic；逐件重做十件一件都不 panic。`unattributed_panic_states=` 数到 1、
    /// `name=q5_piece_panics` 行带它，Q5 走 `q5_not_run=unattributed_panic` 那个出口：判决行 `s13=false`、退 3。
    #[test]
    fn a_judging_path_panic_that_no_piece_repeats_stops_the_q5_as_unattributed() {
        let stream = equivalence_pairing_stream();
        let observation_tables = ObservationTables::of(&stream);
        let injection = PiecePanicInjection {
            rules: vec![PiecePanicInjectionRule {
                piece: None,
                conditions: Vec::new(),
            }],
        };
        let mut context = observation_tables.context(&stream);
        context.piece_panic_injection = Some(&injection);
        let digits = LastSegmentDigits::of(&stream, &observation_tables.tables);
        let observations = observe_sampled_states(
            &context,
            &SampledOrdinals {
                batches: [
                    BTreeSet::from([digits.first_ordinal]),
                    BTreeSet::new(),
                    BTreeSet::new(),
                ],
                torn_scans: Vec::new(),
            },
            RunSwitches::default(),
            ConsultedReports::Dropped,
        );
        assert_eq!(observations.len(), 1, "只判末段第一态");
        assert!(
            observations[0].judging_path_panic.is_some(),
            "正常那一路照注入 panic 了"
        );
        assert_eq!(
            piece_panic_states_of(&observations),
            [0; JUDGING_PIECE_COUNT],
            "逐件重做十件一件都没 panic"
        );
        assert_eq!(unattributed_panic_states_of(&observations), 1);
        let piece_panics_line = q5_piece_panics_line("pairing", &observations);
        assert!(
            piece_panics_line.ends_with(" unattributed_panic_states=1"),
            "{piece_panics_line}"
        );
        let mut lines = ResultLines {
            emitted: 0,
            collected: Some(Vec::new()),
        };
        let exit_code = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(
            &mut lines,
            0,
            &observations,
        );
        let collected = lines.collected.expect("收集模式");
        assert_eq!(exit_code, Some(3), "退 3：{collected:?}");
        assert_eq!(
            collected
                .iter()
                .filter(|line| line.starts_with("name=verdict ")
                    && line.contains(" s13=false")
                    && line.contains(" q5_not_run=unattributed_panic ")
                    && line.ends_with(" unattributed_panic_states=1"))
                .count(),
            1,
            "一行判决带 s13=false、q5_not_run=unattributed_panic 与 unattributed_panic_states=1：{collected:?}"
        );
    }

    // ─────────── 登记修订 6 第 6.5 节：U10–U12（根环表）与 U13–U16（Q5 判决路四个出口） ───────────

    /// U11 截停的那一次：流三建到第 30 个 inode（调查报告第 4 节 N = 30 同形，最新根 txg 35），环已转过 24 槽。
    const INODE_CREATIONS_PAST_ONE_TURN_OF_THE_ROOT_RING: usize = 30;
    /// U11 取样点敏感的那一条：末次发布的最新根 txg 至少这么大（txg 26 起的那条根才盖掉 mkfs 之后第一条还引用旧单元的根，发布不到
    /// txg 26 时不装根环表也是绿，`.claude/rules/mutation-sampling.md` 第三类）。
    const LATEST_ROOT_TXG_PAST_ONE_TURN_OF_THE_ROOT_RING: u64 = 30;
    /// 第 31 批 core 断言（`crates/singlefs-core/src/allocator.rs` 的 `PoolAllocator::record_root_written_by_this_process`，
    /// `expect` 的消息）里独有的一段原文；U12 只认这一段。
    const CORE_ASSERTION_TEXT_OF_AN_ALLOCATOR_WITHOUT_A_ROOT_RING: &str =
        "分配器没装根环表却记了一条根";

    /// 修订 6 之前的造法（旧冻结树装置 `LiveStream::start` 第 1069–1083 行同形）：`PoolAllocator::new` 加 `mark_format_time_units`，
    /// 不装根环表。只在 U12 里用。
    fn allocator_without_the_root_ring_as_before_revision_6() -> PoolAllocator {
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), LOCAL_DEVICE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), LOCAL_DEVICE_BYTES),
        ]);
        allocator.mark_format_time_units(
            Placement {
                slot: INSTANCE_TABLE_SLOT,
                span: 2,
            },
            Placement {
                slot: TREE_TABLE_GENESIS_SLOT,
                span: 1,
            },
        );
        allocator
    }

    /// U11 用的截停帮手（只在用例里用）：流三照 [`LiveStream::run_the_script`] 与 [`LiveStream::run_the_third_stream_after_the_first_file`]
    /// 的次序走取号、暖机、第一个文件、删第一个文件（意图、再一步一步走空），再一次建一个 inode、建到第 `inode_creations` 次为止。
    /// 任一次调用不成功就停在那里交回（用例判每次调用都成功）。迭代上界：走空那一段 `THIRD_STREAM_UNLINK_STEP_LIMIT`，建 inode `inode_creations`。
    fn third_stream_through_inode_creations(inode_creations: usize) -> LiveStream {
        let mut live = LiveStream::start(
            StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf,
            "e171r1-unit-third-stream-past-one-turn-of-the-root-ring",
            RunSwitches::default(),
        );
        if !live.step_acquire_and_warm_up() || !live.step_publish_first_file() {
            return live;
        }
        let write_time_seconds = LOCAL_FIXED_WRITE_TIME_SECONDS + 60;
        let first_inode = InodeNumber(FIRST_INODE_NUMBER);
        if !live.step_third_stream_publish(
            "unlink_intent",
            IdealModel::answer_unlink_intent,
            &|writer, allocator, previous, instance| {
                publish_unlink_intent(
                    writer,
                    allocator,
                    previous,
                    first_inode,
                    write_time_seconds,
                    instance,
                )
            },
        ) {
            return live;
        }
        for _ in 0..THIRD_STREAM_UNLINK_STEP_LIMIT {
            let Some(inode) = live
                .output
                .as_ref()
                .and_then(|output| output.deleted_inodes.first().copied())
            else {
                break;
            };
            if !live.step_third_stream_publish(
                "unlink_step",
                IdealModel::answer_deletion_step,
                &|writer, allocator, previous, instance| {
                    publish_unlink_step(
                        writer,
                        allocator,
                        previous,
                        inode,
                        UnlinkBatchSize::OneExtentLowerLeaf,
                        instance,
                    )
                },
            ) {
                return live;
            }
        }
        for _ in 0..inode_creations {
            if !live.step_third_stream_publish(
                "create_inode",
                IdealModel::answer_create_inode,
                &|writer, allocator, previous, instance| {
                    publish_new_inodes(writer, allocator, previous, 1, write_time_seconds, instance)
                },
            ) {
                return live;
            }
            live.inode_creations += 1;
        }
        live
    }

    /// U10（登记修订 6 第 6.5 节；V32：装根环表那一句删掉，三条流都由 `Some` 变 `None`、S1 启动核对不过，这里红；与 core 带不带
    /// 第 31 批断言无关）：三条流各 `LiveStream::start` 一次、不驱动，分配器都装着根环表，`{:?}` 文本与
    /// `root_ring_occupancy_after_make_filesystem` 给这次 mkfs 现算的那一份相同；第 6.2 节第 1 条那条 S1 在三条流上都过。
    #[test]
    fn every_live_stream_starts_with_the_root_ring_of_make_filesystem() {
        for name in STREAM_NAMES {
            let label = name.directory_name();
            let live = LiveStream::start(
                name,
                &format!("e171r1-unit-root-ring-at-the-start-{label}"),
                RunSwitches::default(),
            );
            let installed = live
                .allocator
                .root_ring_occupancy()
                .unwrap_or_else(|| panic!("{label}：分配器没装根环表"));
            assert_eq!(
                format!("{installed:?}"),
                format!(
                    "{:?}",
                    root_ring_occupancy_after_make_filesystem(
                        &common::parameters(),
                        &live.formatted.genesis
                    )
                ),
                "{label}：装的是 mkfs 刚写下的那一份根环表"
            );
            let check_name = format!("{label}_allocator_carries_the_root_ring_of_make_filesystem");
            let root_ring_checks: Vec<&StopCheck> = live
                .checks
                .iter()
                .filter(|check| check.name == check_name)
                .collect();
            assert_eq!(
                root_ring_checks.len(),
                1,
                "{label}：start 里记了恰一条 {check_name}"
            );
            assert!(
                root_ring_checks[0].holds(),
                "{label}：S1 启动核对过：local={} crates={}",
                root_ring_checks[0].local,
                root_ring_checks[0].crates
            );
        }
    }

    /// U11（登记修订 6 第 6.5 节；V32 下两种 core 都红：带第 31 批断言的 core 在第一个文件那次发布 panic，「每次调用都成功」红；
    /// 不带断言的 core 不回收，txg 26 起 I-3.1 红）：流三驱动到第 30 次建 inode 为止，每次调用都成功；取号、暖机之后的每次发布之后，
    /// 对整条流施加出的镜像（全部写落地）判 I-3.1 都成立；末次发布的最新根 txg 至少 30（根环已转过 24 槽，取样点敏感）。
    #[test]
    fn the_third_stream_past_one_turn_of_the_root_ring_holds_i_3_1() {
        let live =
            third_stream_through_inode_creations(INODE_CREATIONS_PAST_ONE_TURN_OF_THE_ROOT_RING);
        let unsuccessful_calls: Vec<(usize, &'static str, &CallOutcome)> = live
            .driver
            .calls
            .iter()
            .enumerate()
            .filter(|(_, call)| call.outcome != CallOutcome::Succeeded)
            .map(|(call_index, call)| (call_index, call.label, &call.outcome))
            .collect();
        assert!(
            unsuccessful_calls.is_empty(),
            "每次调用都成功：{unsuccessful_calls:?}"
        );
        assert_eq!(
            live.inode_creations, INODE_CREATIONS_PAST_ONE_TURN_OF_THE_ROOT_RING,
            "建 inode 建到了截停那一次"
        );
        let labels_before_the_publishes: Vec<&'static str> = live
            .driver
            .calls
            .iter()
            .take(2)
            .map(|call| call.label)
            .collect();
        assert_eq!(
            labels_before_the_publishes,
            vec!["acquire", "warm_up"],
            "前两次调用是取号、暖机，不经分配器"
        );
        let operations = live.formatted.retained_operations();
        let mkfs_operation_count = live.formatted.mkfs_operation_count;
        let mut image = live.formatted.memory_pool_after_mkfs();
        // 跨轮带的是镜像上已经施加到 mkfs 之后第几条操作；每一轮只补施加这一次调用发出的那一截。
        let mut operations_applied_after_mkfs = 0usize;
        let mut images_after_each_publish: Vec<(String, MemoryPool)> = Vec::new();
        for (publish_index, call) in live.driver.calls.iter().skip(2).enumerate() {
            image.apply(
                &operations[mkfs_operation_count + operations_applied_after_mkfs
                    ..mkfs_operation_count + call.return_point],
            );
            operations_applied_after_mkfs = call.return_point;
            images_after_each_publish.push((
                format!("third-stream/after-publish-{publish_index}-{}", call.label),
                image.clone(),
            ));
        }
        let Some(judged_images) = pool_checker_holds_and_violations_through_the_given_states_engine(
            &images_after_each_publish,
        ) else {
            return;
        };
        let mut publishes_judged = 0usize;
        for (call, (held, violated)) in live.driver.calls.iter().skip(2).zip(&judged_images) {
            assert!(
                held.contains(&"I-3.1"),
                "第 {publishes_judged} 次发布（{}）之后 I-3.1 成立：判到而成立的 {held:?}，违例的 {violated:?}",
                call.label
            );
            publishes_judged += 1;
        }
        assert_eq!(
            publishes_judged,
            live.driver.calls.len() - 2,
            "取号、暖机之后的每次发布都判过"
        );
        let latest_root_txg = live
            .output
            .as_ref()
            .expect("每次调用都成功，末次发布交回了那一版")
            .root
            .checkpoint_txg;
        assert!(
            latest_root_txg.0 >= LATEST_ROOT_TXG_PAST_ONE_TURN_OF_THE_ROOT_RING,
            "末次发布的最新根 txg 至少 {LATEST_ROOT_TXG_PAST_ONE_TURN_OF_THE_ROOT_RING}：{latest_root_txg:?}"
        );
    }

    /// U12（登记修订 6 第 6.5 节；只在带第 31 批 core 断言的 `crates/` 上成立，修订 6 树的 core 不带、那棵树不编用例；第 31 批给
    /// core 断言写的变异「删断言」下第 2 次调用成功，这里红）：`LiveStream::start` 之后把分配器换成修订 6 之前的造法（不装根环表），
    /// 驱动流一三次调用：取号、暖机成功，第一个文件那次发布在 core 记根时 panic，消息带那条断言独有的一段原文。
    #[test]
    fn a_live_stream_without_the_root_ring_panics_in_core_at_its_first_root() {
        let mut live = LiveStream::start(
            StreamName::NewPoolFileCreation,
            "e171r1-unit-without-the-root-ring",
            RunSwitches::default(),
        );
        live.allocator = allocator_without_the_root_ring_as_before_revision_6();
        assert!(
            live.allocator.root_ring_occupancy().is_none(),
            "前提：旧造法不装根环表"
        );
        live.run_the_script();
        let calls = &live.driver.calls;
        let labels: Vec<&'static str> = calls.iter().map(|call| call.label).collect();
        assert_eq!(
            labels,
            vec!["acquire", "warm_up", "publish"],
            "流一三次调用"
        );
        assert_eq!(calls[0].outcome, CallOutcome::Succeeded, "取号成功");
        assert_eq!(calls[1].outcome, CallOutcome::Succeeded, "暖机成功");
        let CallOutcome::Panicked(message) = &calls[2].outcome else {
            panic!("第一个文件那次发布在 core 里 panic：{:?}", calls[2].outcome);
        };
        assert!(
            message.contains(CORE_ASSERTION_TEXT_OF_AN_ALLOCATOR_WITHOUT_A_ROOT_RING),
            "panic 的是第 31 批那条 core 断言：{message}"
        );
    }

    /// U13–U16 用的注入：一条不点名件的规则（`piece: None`），条件是写表下标 `write_index` 那个写落了（「落」= 落地串那一位不是 `0`）。
    fn an_unattributed_panic_where_the_write_landed(write_index: usize) -> PiecePanicInjection {
        PiecePanicInjection {
            rules: vec![PiecePanicInjectionRule {
                piece: None,
                conditions: vec![InjectedLandingCondition::Landed(write_index)],
            }],
        }
    }

    /// U15、U16 用的注入：一条不带条件、不点名件的规则（选中全部判的态）。
    fn an_unattributed_panic_in_every_state() -> PiecePanicInjection {
        PiecePanicInjection {
            rules: vec![PiecePanicInjectionRule {
                piece: None,
                conditions: Vec::new(),
            }],
        }
    }

    /// 对拍流（k = 2）的 c1（末段里第 3 次调用发出的第一个写）与末段里 c1 落了的态数：用例自己按末段的混合进制逐态数，不读装置的计数。
    fn first_in_flight_write_and_the_states_where_it_landed(
        stream: &PreparedStream,
    ) -> (usize, usize) {
        let tables = crash_state_enumeration_tables(
            &stream.base,
            &stream.writes,
            &stream.segments,
            &full_expansion,
        );
        let first_in_flight_write = *equivalence_pairing_shape_facts(stream, &tables)
            .in_flight_writes
            .first()
            .expect("对拍流末段里有第 3 次调用发出的写");
        let digits = LastSegmentDigits::of(stream, &tables);
        let position = digits
            .write_indexes
            .iter()
            .position(|write_index| *write_index == first_in_flight_write)
            .expect("c1 在末段里");
        let states_where_it_landed = (digits.first_ordinal..tables.state_count)
            .filter(|ordinal| digits.digit_of(*ordinal, position) != 0)
            .count();
        (first_in_flight_write, states_where_it_landed)
    }

    /// U13–U16 用的两形：主那一形取 `truncation` 截出的流二前四次调用，对拍那一形照 `pairing` 交，注入照 `piece_panic_injections` 交。
    fn confirmed_loss_shapes_with_injections(
        sampling_point: &'static str,
        truncation: ConfirmedLossTruncation,
        pairing: Option<PreparedStream>,
        piece_panic_injections: ConfirmedLossPiecePanicInjections,
    ) -> ConfirmedLossShapes {
        let (main, shape) = confirmed_loss_prepared_stream(
            second_stream_through_the_first_overwrite(RunSwitches::default()),
            truncation,
        );
        ConfirmedLossShapes {
            sampling_point,
            main,
            shape,
            drive_nanoseconds: 0,
            pairing,
            piece_panic_injections,
        }
    }

    /// U13–U16 的共同断言：退 3；倒数第二行（末行是 `name=done`）是判决行，带 `s13=false`、`q5_not_run=unattributed_panic` 与
    /// `unattributed_panic_states=<用例现算的数>`，判决行只有这一行。
    fn assert_the_q5_stopped_as_unattributed(
        exit_code: i32,
        collected: &[String],
        expected_unattributed_panic_states: usize,
    ) {
        assert_eq!(exit_code, 3, "退 3：{collected:?}");
        assert!(
            collected
                .last()
                .is_some_and(|line| line.starts_with("name=done ")),
            "末行是 name=done：{collected:?}"
        );
        let verdict_lines: Vec<&String> = collected
            .iter()
            .filter(|line| line.starts_with("name=verdict "))
            .collect();
        assert_eq!(verdict_lines.len(), 1, "判决行恰一行：{collected:?}");
        assert_eq!(
            collected
                .iter()
                .rposition(|line| line.starts_with("name=verdict ")),
            Some(collected.len() - 2),
            "判决行就在 name=done 之前：{collected:?}"
        );
        let verdict = verdict_lines[0];
        assert!(
            verdict.contains(" s13=false ")
                && verdict.contains(" q5_not_run=unattributed_panic ")
                && verdict.ends_with(&format!(
                    " unattributed_panic_states={expected_unattributed_panic_states}"
                )),
            "判决行带 s13=false、q5_not_run=unattributed_panic 与 unattributed_panic_states={expected_unattributed_panic_states}：{verdict}"
        );
    }

    /// U13（登记修订 6 第 6.5 节；V33：对拍之后收 unattributed 那一处整块删掉，往下走到阳性对照、k = 2 上内置规则造不出、判决行换成
    /// `piece_panic_control_failed`，这里红）：主那一形与对拍都是 k = 2 的流，对拍那一遍注入一条不点名件的规则（条件「c1 落」），
    /// 在对拍那一处就停：退 3、判决行 `q5_not_run=unattributed_panic`、数等于 c1 落了的态数；阳性对照一行都没打。
    #[test]
    fn an_unattributed_panic_in_the_pairing_stops_the_q5_at_the_pairing() {
        let pairing = equivalence_pairing_stream();
        let (first_in_flight_write, states_where_it_landed) =
            first_in_flight_write_and_the_states_where_it_landed(&pairing);
        let shapes = confirmed_loss_shapes_with_injections(
            "main_before_the_first_barrier_of_call_3",
            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
                IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS,
            ),
            Some(pairing),
            ConfirmedLossPiecePanicInjections {
                pairing: Some(an_unattributed_panic_where_the_write_landed(
                    first_in_flight_write,
                )),
                piece_panic_control: None,
                sampling_point_export: None,
            },
        );
        let command_line = ConfirmedLossCommandLine {
            output_directory: std::env::temp_dir().join(format!(
                "e171r1-unit-unattributed-in-the-pairing-{}",
                std::process::id()
            )),
            truncate_at_the_return_point: false,
        };
        let mut lines = ResultLines {
            emitted: 0,
            collected: Some(Vec::new()),
        };
        let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
        let collected = lines.collected.expect("收集模式");
        assert_the_q5_stopped_as_unattributed(exit_code, &collected, states_where_it_landed);
        assert!(
            !collected
                .iter()
                .any(|line| line.starts_with("name=q5_piece_panic_control")),
            "停在对拍那一处，阳性对照一行都没打：{collected:?}"
        );
    }

    /// U14（登记修订 6 第 6.5 节；V34：阳性对照之后收 unattributed 那一处整块删掉，走到 F9 那一支、判决行换成
    /// `piece_panic_control_failed`，这里红）：同 U13 的两形，对拍不注入；阳性对照那一格交同一条规则（替换内置规则），
    /// 在阳性对照那一处就停：退 3、数等于 c1 落了的态数；阳性对照那一行打了，主取样点那一形一行都没打。
    #[test]
    fn an_unattributed_panic_in_the_piece_panic_control_stops_the_q5_at_the_control() {
        let pairing = equivalence_pairing_stream();
        let (first_in_flight_write, states_where_it_landed) =
            first_in_flight_write_and_the_states_where_it_landed(&pairing);
        let shapes = confirmed_loss_shapes_with_injections(
            "main_before_the_first_barrier_of_call_3",
            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
                IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS,
            ),
            Some(pairing),
            ConfirmedLossPiecePanicInjections {
                pairing: None,
                piece_panic_control: Some(an_unattributed_panic_where_the_write_landed(
                    first_in_flight_write,
                )),
                sampling_point_export: None,
            },
        );
        let command_line = ConfirmedLossCommandLine {
            output_directory: std::env::temp_dir().join(format!(
                "e171r1-unit-unattributed-in-the-control-{}",
                std::process::id()
            )),
            truncate_at_the_return_point: false,
        };
        let mut lines = ResultLines {
            emitted: 0,
            collected: Some(Vec::new()),
        };
        let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
        let collected = lines.collected.expect("收集模式");
        assert_the_q5_stopped_as_unattributed(exit_code, &collected, states_where_it_landed);
        assert_eq!(
            collected
                .iter()
                .filter(|line| line.starts_with("name=q5_piece_panic_control "))
                .count(),
            1,
            "阳性对照那一行打了：{collected:?}"
        );
        assert!(
            !collected
                .iter()
                .any(|line| line.starts_with("name=q5_shape")),
            "停在阳性对照那一处，主取样点那一形一行都没打：{collected:?}"
        );
    }

    /// U15（登记修订 6 第 6.5 节；V35：反向取样点导出之后收 unattributed 那一处整块删掉，往下打出 `report_the_confirmed_loss` 那几行
    /// 与不带 `q5_not_run` 的判决行，这里红）：反向取样点那一形（对拍为空），导出那一格交一条不带条件的不点名件规则，导出之后就停：
    /// 退 3、数等于末段各写取法数之积罩的态加全持久态（用例现算）；`report_the_confirmed_loss` 的计数行一行都没打。
    #[test]
    fn an_unattributed_panic_at_the_reverse_sampling_point_stops_before_the_tally() {
        let shapes = confirmed_loss_shapes_with_injections(
            "reverse_at_the_return_point_of_call_2",
            ConfirmedLossTruncation::AtTheConfirmedReturnPoint,
            None,
            ConfirmedLossPiecePanicInjections {
                pairing: None,
                piece_panic_control: None,
                sampling_point_export: Some(an_unattributed_panic_in_every_state()),
            },
        );
        let judged_states = {
            let tables = crash_state_enumeration_tables(
                &shapes.main.base,
                &shapes.main.writes,
                &shapes.main.segments,
                &full_expansion,
            );
            let last_segment = shapes.main.segments.last().cloned().unwrap_or_default();
            let product_of_the_choices: u64 = last_segment
                .iter()
                .map(|write_index| {
                    if tables.is_tearable_by_write[*write_index] {
                        3u64
                    } else {
                        2u64
                    }
                })
                .product();
            let first_ordinal_of_the_last_segment = tables
                .state_ranges_by_segment
                .last()
                .map_or(0, |state_range| state_range.start);
            let mut states: BTreeSet<u64> = (first_ordinal_of_the_last_segment
                ..first_ordinal_of_the_last_segment + product_of_the_choices)
                .collect();
            states.insert(tables.state_count - 1);
            states.len()
        };
        let output_directory = std::env::temp_dir().join(format!(
            "e171r1-unit-unattributed-at-the-reverse-sampling-point-{}",
            std::process::id()
        ));
        let command_line = ConfirmedLossCommandLine {
            output_directory: output_directory.clone(),
            truncate_at_the_return_point: true,
        };
        let mut lines = ResultLines {
            emitted: 0,
            collected: Some(Vec::new()),
        };
        let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
        std::fs::remove_dir_all(&output_directory)
            .unwrap_or_else(|error| panic!("删 {}：{error}", output_directory.display()));
        let collected = lines.collected.expect("收集模式");
        assert_the_q5_stopped_as_unattributed(exit_code, &collected, judged_states);
        assert!(
            !collected
                .iter()
                .any(|line| line.starts_with("name=q5 sampling_point=")),
            "停在计数之前，report_the_confirmed_loss 的计数行一行都没打：{collected:?}"
        );
    }

    /// U16（登记修订 6 第 6.5 节；V36：主取样点导出之后收 unattributed 那一处整块删掉，往下走到类核对与判决，这里红）：主那一形取
    /// k = 2 的流、对拍为空（只为走到主取样点导出之后那一处，跑法里不会这么交），导出那一格交同 U15 的规则，导出之后就停：
    /// 退 3、数等于分法交的判态数（用例从 `classify_the_last_segment` 的 `judged_ordinals` 现数）；类核对那一行没打。
    #[test]
    fn an_unattributed_panic_at_the_main_sampling_point_stops_before_the_classes_are_judged() {
        let shapes = confirmed_loss_shapes_with_injections(
            "main_before_the_first_barrier_of_call_3",
            ConfirmedLossTruncation::AfterTheFirstInFlightWrites(
                IN_FLIGHT_WRITES_KEPT_IN_THE_TESTS,
            ),
            None,
            ConfirmedLossPiecePanicInjections {
                pairing: None,
                piece_panic_control: None,
                sampling_point_export: Some(an_unattributed_panic_in_every_state()),
            },
        );
        let judged_states = {
            let observation_tables = ObservationTables::of(&shapes.main);
            let context = observation_tables.context(&shapes.main);
            let digits = LastSegmentDigits::of(&shapes.main, &observation_tables.tables);
            let classification = classify_the_last_segment(
                &context,
                &digits,
                CONFIRMED_LOSS_IN_FLIGHT_CALL,
                CONFIRMED_LOSS_STATE_LIMIT,
            );
            assert!(
                !classification.stopped_at_the_state_limit,
                "k = 2 的流分类没撞态数上限"
            );
            classification.judged_ordinals(&digits).len()
        };
        let output_directory = std::env::temp_dir().join(format!(
            "e171r1-unit-unattributed-at-the-main-sampling-point-{}",
            std::process::id()
        ));
        let command_line = ConfirmedLossCommandLine {
            output_directory: output_directory.clone(),
            truncate_at_the_return_point: false,
        };
        let mut lines = ResultLines {
            emitted: 0,
            collected: Some(Vec::new()),
        };
        let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);
        std::fs::remove_dir_all(&output_directory)
            .unwrap_or_else(|error| panic!("删 {}：{error}", output_directory.display()));
        let collected = lines.collected.expect("收集模式");
        assert_the_q5_stopped_as_unattributed(exit_code, &collected, judged_states);
        assert!(
            !collected
                .iter()
                .any(|line| line.starts_with("name=q5_equivalence sampling_point=")),
            "停在类核对之前，name=q5_equivalence sampling_point= 一行都没打：{collected:?}"
        );
    }
}

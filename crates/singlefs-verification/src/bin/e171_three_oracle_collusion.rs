// task-server-gpu-arguments: baseline row known-bad-images
// 三个模式的判段都经 `given_states_engine_of_this_experiment`（`crash_amplification::GivenStatesEngine`）判盘：baseline 与 row 判取样态的 O2，
// known-bad-images 判已知坏镜像与 E170（O3规约执行器第一版）的坏状态；带 gpu 编好的二进制直接交统一任务服务时，由服务按这一行清场、分卡。
// 录入段（`--stage record`）不起引擎、不碰卡，判段（`--stage judge`）才在卡上判；这一行按模式词认，分不出段。
//! E171（三oracle合谋测试） 第一段：入库装置。
//!
//! 登记：`research/prompts/e171-preregistration.md` 5.3（装置写在哪）、5.5（坏镜像表）、5.6（阳性与阴性对照）、5.8「第一段」、
//! 第十一节停机 S1–S6 与 S8。它回答问题单 Q1–Q3 里 O1、O2 两列的那一半：两条崩溃点重放流（新池新建文件、固定脚本到 E 再正常卸载）上，
//! 每次 API 调用之后拿理想模型的答案与实现的结局比（O1-对拍），每个取样的崩溃状态上拿实现只读恢复读回的那一版问理想模型
//! （O1-崩溃判官），再拿池级 checker 判这一态的崩溃镜像（O2）；O3（Python 执行器）读本装置照 E170（O3规约执行器第一版） 登记 5.3 写出的
//! 中性格式，不在这个文件里。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness、format 与 replay-prefix。
//! 两条流的驱动、取样、中性格式的写出与读回照抄 `e170_specification_executor_export.rs`（不 `#[path]` 引它），改三处（登记 5.3）：
//! ① 驱动里的 API 调用不 `.expect`：结局（成功 / 错误成员 / panic）记进 `calls.tsv` 的 `ok` / `err` 列与 `call-outcomes.tsv`，
//! 出错之后这条流不再往下驱动；② 跟着每次调用推理想模型，记 O1-对拍；③ 每个取样的状态算 O1-崩溃判官与 O2，写 `oracle-verdicts.tsv`。
//!
//!   e171_three_oracle_collusion baseline --output-directory <目录>
//!   e171_three_oracle_collusion row --output-directory <目录>
//!   e171_three_oracle_collusion known-bad-images --dump-directory <基线导出目录> --output-directory <目录> [--e170-bad-rows]
//!   以上三种之后都可以加 `--positive-control <O1-对拍|O1-崩溃判官|O2>` 或 `--disable-oracle <O1|O2>`（只对 baseline 与 row）
//!   以及 `--sampling-threads <正整数>`：baseline 与 row 判取样态、known-bad-images 判已知坏镜像与 E170 坏状态的线程数，不给取每台默认重活线程数，都再按内存上限收（`sampling_threads_of_this_run`）；
//!   写 1 是一条线程按序号走完的那一趟，产物（`E7RESULT` 行与去掉耗时之后的文件）与几条线程的逐字相同
//!   以及 `--stage record|judge`（三种模式都认，取值照 `crash_library::RunStage` 的写法）：`record` 只跑录入段，`judge` 只跑判段，不给就两段先后跑。
//!
//! 两段（D13（验证路线） 已定项 15「判器上 GPU」；`.claude/rules/verification.md`「一个门禁：两种跑法」按需领卡那一句）：
//! 录入段只在 CPU 上跑：驱动、取样、O1、导出文件都在这一段，O2 要判的盘一张一张作给定状态表录进判定库（prune-server 里那一份，
//! 给定状态表那一类、按表的摘要），不起给定状态引擎、不碰卡；这一段的观测与结果行要的东西写成一份交接，放在输出目录里
//! （[`HAND_OFF_TO_THE_JUDGE_STAGE_FILE`]）。判段不再驱动、不再取样：读回交接，凭摘要从库里读回每张盘、交引擎判，写判完才有的产物
//! （`oracle-verdicts.tsv`、`known-bad-o2.tsv`、`e170-bad-rows-o2.tsv`，坏镜像模式的另两个产物也在这一段写），打全部 `E7RESULT` 行，删掉交接。
//! 不给 `--stage` 时两段先后跑，交接在内存里交，产物与结果行与分两趟跑的逐字节相同（去掉耗时之后）。
//! 判器的结论只进产物与结果行，不回头定取样与驱动：三种模式都分得开。
//! 段不用 `SINGLEFS_CRASH_AMPLIFICATION_STAGE` 点：取样那一步（`sample_crash_states_on_the_cpu`）照那个变量选段，设了它取样就只录不判。
//!
//! `baseline`：回比本地常量（S1）、三批取样、写十个导出文件加 `oracle-verdicts.tsv` 与 `call-outcomes.tsv`；`row`：同上，只是
//! 不回比常量（坏实现那一趟本来就会让写流与段表变）、只取批 1 与批 2；S5 在这一趟也判：返回了错误的调用（卸载除外）都记成 O1-对拍的红
//! （取号、暖机出错也是），不适用只来自卸载与对拍判出不同之后（`not_applicable_source_checks_of_a_bad_implementation_row`）。目录已存在就拒绝。
//! 标准输出：`E7RESULT` 行（与跑几次无关）与 `E7TIMING` 行（耗时）；收尾 `name=done emitted=N`。
//! 退出码：0 正常（只跑录入段的那一趟录完就是 0，停机条款归判段）；2 命令行不对、要录的输出目录已存在、只跑判段而录入段没跑过
//! （或交接与这一趟的参数对不上）；3 停机条款 S1、S3、S4、S5 没过（产物照打，交主 agent）。

/// 崩溃点重放取样过崩溃判定库的项（`crash_state_enumeration::sample_crash_states_on_the_cpu`）：项名与设点文件（这份装置自己）。
const CRASH_POINT_REPLAY_SAMPLING_ITEM: CrashPointReplayItem<'static> = CrashPointReplayItem {
    name: "e171_three_oracle_collusion",
    setting_file: "crates/singlefs-verification/src/bin/e171_three_oracle_collusion.rs",
};

/// 这一次取样的项名：装置一趟按不同的段、不同的序号区间取样几十次，每次是一条不同的流，项名各给一个（库里同一个项名下只登一份清单，
/// 几次共用会撞；块键按内容算，下一趟照样复用判过的块）。号按调用的次序发：`observe_sampled_states` 在起线程之前按连续段的次序取好，
/// 几条线程与一条线程的那一趟每个连续段拿到同一个项名、录入键相同。
fn sampling_item_name_of_this_call() -> String {
    static NEXT_SAMPLING_NUMBER: std::sync::atomic::AtomicU64 =
        std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        CRASH_POINT_REPLAY_SAMPLING_ITEM.name,
        NEXT_SAMPLING_NUMBER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    )
}

#[path = "../../tests/common/mod.rs"]
mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::make_filesystem::{
    root_ring_occupancy_after_make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::recovery::{PoolReader, RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput, WarmUpOutput,
};
use singlefs_verification::crash::{
    crash_point_replay_plan_state_count, crash_state_enumeration_tables, full_expansion,
    in_place_subsets_with_copy_on_write_none_or_all_expansion, CrashStateEnumerationTables,
    CrashStateJudgement,
};
use singlefs_verification::crash_amplification::RecordedGivenStatesTableWithItsColumns;
use singlefs_verification::crash_facts::{extract_flow_facts, FlowFacts};
use singlefs_verification::crash_library::{LibraryOpenInThisProcess, RunStage};
use singlefs_verification::crash_state_enumeration::{
    sample_crash_states_on_the_cpu, CrashPointReplayItem,
};
use singlefs_verification::known_bad_images::known_bad_images;
use singlefs_verification::memory_pool::{
    newest_persisted_root, writes_and_segments_with_stream_indexes_and_entries, CrashImage,
    MemoryPool, PublishedVersion, RetainedWrite, WrittenContents,
};
use singlefs_verification::model::{
    crash_recovery_disagreement, IdealModel, ModelAnswer, ModelDeviceIdentity, ModelDisagreement,
    ModelPoolGeometry, ModelRootKey, ObservedEffect, ObservedFile, ObservedOutcome,
    ObservedReadBack,
};
use singlefs_verification::model_comparison::{
    model_root_key, observed_mount_carrying_to_zero_unit_publishes,
    observed_read_back_after_a_crash, observed_root_of_file_version, refusal_reason_of_mount_error,
    refusal_reason_of_publish_error, refusal_reason_of_rollback_error,
    reported_ceiling_of_mount_error, root_ring_slot_still_bad_after_one_reread_of_mount_error,
    root_ring_slot_still_bad_after_one_reread_of_rollback_error,
};
use singlefs_verification::segments::StepKind;
use singlefs_verification::sha256::{sha256_digest, sha256_hexadecimal, DIGEST_BYTES};
use singlefs_verification::{
    RecordedEntrySpan, RecordedOperationKind, RecordedPublishEntry, RetainedOperation, SharedStream,
};

// ───────────────────────────── 本地常量（登记 5.3、S1） ─────────────────────────────
//
// 下面这一批与 `e170_specification_executor_export.rs`「本地常量（登记 5.7、S1）」那一节同一批、同样的值与出处注释；`baseline` 开头与 crates 的同名公开常量或现算值回比，
// 不直接引（决定几何与工作量的数写成本地常量：引了就等于装置与实现共用一个数，两边一起错而逐格相等）。
// 这些常量只在基线那一趟回比：坏实现改了写流，这些数在坏实现那一趟本来就会变，照数记进产物。

/// 盘数：`tests/common/mod.rs` 的 `formatted_recorded_images` 建两块盘（`for device_number in 0..2u32`）。
const LOCAL_DEVICE_COUNT: usize = 2;
/// 每块盘的字节数：`tests/common/mod.rs` 的 `IMAGE_BYTES = 4 << 30`。
const LOCAL_DEVICE_BYTES: u64 = 4_294_967_296;
/// 物理块与 io_min：`tests/common/mod.rs` 里 `physical_block_size: 512`、`minimum_input_output_bytes: 512`。
const LOCAL_PHYSICAL_BLOCK_BYTES: u32 = 512;
const LOCAL_MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
/// 导出格式的扇区宽（登记 5.3 `sector_bytes`（512））：与 `singlefs_verification::memory_pool::SECTOR_BYTES` 回比。
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

/// 流一的段长（12 段）：取号两轮各一段（2、2；D22（单元原子性怎么合成） 已定项 8「取号写两轮」），暖机两次各是记录段 2、根槽段 1、轮换段 2，
/// A 是 24 个单元写一段、记录 2、根槽 1、轮换 2。`tests/crash_enumeration_new_pool_file_creation_stream.rs` 的段长断言是同一张表。
const LOCAL_FIRST_STREAM_SEGMENT_LENGTHS: [usize; 12] = [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
/// 流一的写数：取号两轮各 2 + 暖机两次各 5 + A 29。
const LOCAL_FIRST_STREAM_WRITES: usize = 43;
/// 流一的根槽写数：暖机两次各一条 + A 一条。
const LOCAL_FIRST_STREAM_ROOT_WRITES: usize = 3;
/// 流一全量与甲二（每段按甲二展开）的状态数：同一份测试文件的 `FULL_STATES` 与 `IN_PLACE_SUBSETS_STATES`。
const LOCAL_FIRST_STREAM_FULL_STATES: u64 = 16_777_268;
const LOCAL_FIRST_STREAM_IN_PLACE_SUBSETS_STATES: u64 = 54;
/// 流一每次写只取两态时的全量闭式：同一份文件的 `FULL_STATES_WITH_TWO_STATES_PER_WRITE`（单测拿段长表回算它）。
const LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM: u64 = 16_777_243;

/// 流二的段长（80 段）：每次发布是单元段（暖机 txg 1、2 没有）、记录段 2、根槽段 1、轮换段 2；取号（流开头与 B 之后重开可写挂载各一次）两轮各一段 2；
/// 抬 F 与卸载开头先写新 F 一段 2。单元段的写数：A、B、C、回退 D 各 24（回退 D 含谱系容器与谱系树根，D18（块里携带什么信息） 已定项 24 第 5 条），
/// 写行 18，写行之后的两次暖机与抬 F、卸载推的空发布各 16，五次覆盖写与 E 各 28。
const LOCAL_SECOND_STREAM_SEGMENT_LENGTHS: [usize; 80] = [
    2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
    16, // 取号到写行之后的暖机
    2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2,
    28, // C、回退 D、覆盖写
    2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1,
    2, // 抬 F、E、卸载
    16, 2, 1, 2, // 卸载第二次空发布与 txg 19 的轮换
];
/// 流二的写数：段长之和，到 D 199 + 五次覆盖写 5 × 33 + 抬 F 2 + 2 × 21 + E 33 + 卸载 2 + 2 × 21 = 485。
const LOCAL_SECOND_STREAM_WRITES: usize = 485;
/// 流二的根槽写数：`tests/crash_enumeration_fixed_script_stream.rs` 里这条流的根槽写数断言（`=> 19`），19 次发布各一条。
const LOCAL_SECOND_STREAM_ROOT_WRITES: usize = 19;
/// 流二全量与甲二的状态数：同一份测试文件的 `FULL_STATES_THROUGH_THE_UNMOUNT` 与 `IN_PLACE_SUBSETS_STATES_THROUGH_THE_UNMOUNT`。
const LOCAL_SECOND_STREAM_FULL_STATES: u64 = 1_678_377_220;
const LOCAL_SECOND_STREAM_IN_PLACE_SUBSETS_STATES: u64 = 294;
/// 流二每次写只取两态时的全量闭式：同一份文件的 `FULL_STATES_WITH_TWO_STATES_PER_WRITE_THROUGH_THE_UNMOUNT`（单测拿段长表回算它）。
const LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM: u64 = 1_678_377_095;
/// E 的数据单元落点：同一份测试文件里 E 那一步的断言（`50176`）。
const LOCAL_SECOND_STREAM_REUSED_VERSION_DATA_UNIT_SLOT: u64 = 50_176;
/// 抬 F 推的空发布数：同一份测试文件里抬 F 那一步的断言（`raised.publishes.len(), 2`）。
const LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES: usize = 2;
/// 卸载推的两次空发布的 txg：同一份测试文件里卸载那一步的断言（`vec![CheckpointTxg(18), CheckpointTxg(19)]`）。
const LOCAL_SECOND_STREAM_UNMOUNT_TXGS: [u64; 2] = [18, 19];

/// 取样：批 1 每段内点的分母、批 2 的分母、批 3 的分母（登记 5.4）。
const FIRST_BATCH_DIVISOR: u64 = 7;
const SECOND_BATCH_DIVISOR: u64 = 14;
const THIRD_BATCH_DIVISOR: u64 = 23;
/// 批 1 在带撕裂写的段里另取几个带「1」的状态、每段最多扫几个序号（登记 5.4）。
const TORN_STATES_PER_SEGMENT: usize = 4;
const TORN_SCAN_ORDINALS_PER_SEGMENT: u64 = 100_000;

/// 卸载那次调用在 `calls.tsv` 里的标签：模型没有卸载的答（登记第一节读法 6），这次调用不对拍。
const UNMOUNT_CALL_LABEL: &str = "unmount";

/// 两条流的名字（登记 5.3 的目录名）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StreamName {
    NewPoolFileCreation,
    FixedScriptReuseAfterRaisingFloorThenUnmount,
}

impl StreamName {
    fn directory_name(self) -> &'static str {
        match self {
            Self::NewPoolFileCreation => "new_pool_file_creation",
            Self::FixedScriptReuseAfterRaisingFloorThenUnmount => {
                "fixed_script_through_e_then_unmount"
            }
        }
    }
}

const STREAM_NAMES: [StreamName; 2] = [
    StreamName::NewPoolFileCreation,
    StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount,
];

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
}

impl CallOutcome {
    /// `calls.tsv` 最后一列：E170（O3规约执行器第一版） 登记 5.3 写的 `ok` / `err`；panic 的流不导出，这里不会写到 panic。
    fn calls_file_word(&self) -> &'static str {
        match self {
            Self::Succeeded => "ok",
            Self::ReturnedError(_) | Self::Panicked(_) => "err",
        }
    }
    /// `call-outcomes.tsv` 里的结局列：成功是 `ok`，错误是 `err:<成员>`，panic 是 `panic`（消息另写结果行）。
    fn outcome_word(&self) -> String {
        match self {
            Self::Succeeded => "ok".to_string(),
            Self::ReturnedError(member) => format!("err:{member}"),
            Self::Panicked(_) => "panic".to_string(),
        }
    }
}

/// 理想模型对一次调用的判定（O1-对拍，登记 5.3「调用与模型的对应」那张表）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum ModelVerdictOfCall {
    /// 模型这一步不交答：取号、暖机（两次都返回之后一起推一步）与卸载（`crates/singlefs-verification/src/model.rs` 里已有 `answer_unmount`，本装置还没改调它，卸载这一步照旧不问模型；改调归 E171 第 1 次重跑）。
    NotModelled,
    /// 对得上，模型往前走。
    Agrees,
    /// 对不上的第一格；模型停在这里，此后的调用不比（读法 6）。
    Disagrees {
        aspect: String,
        model_answer: String,
        implementation_answer: String,
    },
    /// 前面有一次对不上（或前面有一次调用出了错），这次不比。
    SkippedAfterTheModelStopped,
}

impl ModelVerdictOfCall {
    fn word(&self) -> String {
        match self {
            Self::NotModelled => "not_modelled".to_string(),
            Self::Agrees => "agrees".to_string(),
            Self::Disagrees { aspect, .. } => format!("disagrees:{aspect}"),
            Self::SkippedAfterTheModelStopped => "skipped_after_the_model_stopped".to_string(),
        }
    }
}

/// 驱动方调的一次 API：标签、它发出的第一条操作的流序下标、返回点（返回时录制流的长度，都从 mkfs 之后数）、结局、模型的判定。
#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedCall {
    label: String,
    first_operation: usize,
    return_point: usize,
    outcome: CallOutcome,
    model: ModelVerdictOfCall,
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

/// 阳性对照要改的那一处观测（登记 5.6「坏实现」行的阳性一栏）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PositiveControl {
    /// 流二第一次 `publish` 的观测里那一版的文件内容末字节翻一位，再交 `judge_and_advance`。
    ComparisonOfTheCalls,
    /// 批 1 里第一个「实现读回了文件」的态，读回内容末字节翻一位，再交 `crash_recovery_disagreement`。
    CrashJudge,
    /// 流一全持久态叠已知坏镜像族第一条，再交 `check_pool_image`。
    PoolChecker,
}

impl PositiveControl {
    fn parse(word: &str) -> Option<Self> {
        match word {
            "O1-对拍" => Some(Self::ComparisonOfTheCalls),
            "O1-崩溃判官" => Some(Self::CrashJudge),
            "O2" => Some(Self::PoolChecker),
            _ => None,
        }
    }
}

/// 关掉的那条 oracle（Q3-核，登记第一节读法 9）：关掉的列不算、写成 `off`，别的两列照常。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DisabledOracle {
    ComparisonAndCrashJudge,
    PoolChecker,
}

/// 一次跑的开关：阳性对照、关一条、判取样态与坏镜像的线程数（不进产物，`None` 取每台默认重活线程数）。
#[derive(Clone, Copy, Debug, Default)]
struct RunSwitches {
    positive_control: Option<PositiveControl>,
    disabled_oracle: Option<DisabledOracle>,
    sampling_threads: Option<NonZeroUsize>,
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
/// 记返回点与结局，入口 panic 记「实现自崩」，错误或 panic 之后这条流不再往下驱动。
struct StreamDriver {
    name: StreamName,
    stream: SharedStream,
    mkfs_operation_count: usize,
    switches: RunSwitches,
    calls: Vec<RecordedCall>,
    model: IdealModel,
    /// 每一步之后把模型根环里的每一版并进来（崩溃注入的做法）：O1-崩溃判官拿它问「恢复到的这一版是不是模型提交过的」。
    committed_versions: BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    /// 模型不再往前走的那次调用：第一次对不上的、没有对应答的 `unmount`、出了错或 panic 的调用（读法 6）。
    model_stopped_at_call: Option<usize>,
    implementation_crash: Option<String>,
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
            implementation_crash: None,
            halted: false,
            comparison_control_applied: false,
        };
        driver.merge_committed_versions();
        driver
    }

    /// O1 开着：关一条（Q3-核）关掉 O1 时模型一步都不推、对拍与崩溃判官都不算。
    fn model_is_enabled(&self) -> bool {
        self.switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge)
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

    /// 调一次实现的入口并记一行：调之前与返回时各读一次流长。交回 `Err(Some(错误))` 是实现交回了错误（调用方照错误造拒绝的观测），
    /// `Err(None)` 是 panic。
    fn call<Output, Failure: std::fmt::Debug>(
        &mut self,
        label: &'static str,
        run: impl FnOnce() -> Result<Output, Failure>,
    ) -> Result<Output, Option<Failure>> {
        let first_operation = self.operations_so_far();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(run));
        let return_point = self.operations_so_far();
        let call_index = self.calls.len();
        let (outcome, returned) = match result {
            Ok(Ok(output)) => (CallOutcome::Succeeded, Ok(output)),
            Ok(Err(failure)) => (
                CallOutcome::ReturnedError(error_member_of(&failure)),
                Err(Some(failure)),
            ),
            Err(payload) => {
                let text = panic_text(&*payload);
                self.implementation_crash = Some(format!("call_{call_index}_{label}:{text}"));
                (CallOutcome::Panicked(text), Err(None))
            }
        };
        if outcome != CallOutcome::Succeeded {
            self.halted = true;
            self.stop_the_model_at(call_index);
        }
        self.calls.push(RecordedCall {
            label: label.to_string(),
            first_operation,
            return_point,
            outcome,
            model: ModelVerdictOfCall::NotModelled,
        });
        returned
    }

    /// 刚记的这次调用模型没有对应的答（卸载）：模型停在这里。
    fn note_that_the_model_has_no_answer_for_the_last_call(&mut self) {
        let call_index = self.calls.len() - 1;
        self.stop_the_model_at(call_index);
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
    /// 不算自崩，是 O1-对拍的红）。不记的话这次调用的 O1-对拍一栏留在 `not_modelled`、之后的态记不适用，一行这样的坏实现三列都不红。
    /// `failure` 是 `None`（panic，实现自崩）不记；O1 关掉（Q3-核）时不记，那一列写 `off`。
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
                singlefs_verification::model::ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
                    .name()
                    .to_string(),
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
        if matches!(
            self.calls[call_index].model,
            ModelVerdictOfCall::Disagrees { .. }
        ) {
            return;
        }
        if self.switches.positive_control == Some(PositiveControl::ComparisonOfTheCalls)
            && self.name == StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount
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
                    aspect: disagreement.aspect.name().to_string(),
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
        reason: singlefs_verification::model::ObservedRefusalReason,
        publishes_completed: usize,
        stream_length_before: usize,
        reported_ceiling: Option<singlefs_verification::model::ModelCheckpointTxg>,
        slot_still_bad: Option<singlefs_verification::model::ModelRingPosition>,
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

// ───────────────────────────── 驱动两条流（登记 5.3：不 `.expect`、跟着推模型） ─────────────────────────────

fn version(instance: u32, checkpoint_txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
        content,
    }
}

/// 一条流驱动时的池：只做过 mkfs 的两块文件镜像（`FormattedPool` 管镜像文件，丢掉就删）、分配器、现行那一版、崩溃点重放手写 oracle 的版本表。
/// 流一照抄 `tests/common/mod.rs` 的 `build_pool` 的取号、暖机、新池新建文件三步，流二照抄
/// `tests/crash_enumeration_fixed_script_stream.rs` 的 `prepare` 里 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支；
/// 每一步都经 [`StreamDriver::call`]。
struct LiveStream {
    driver: StreamDriver,
    formatted: common::FormattedPool,
    allocator: PoolAllocator,
    instance: Option<InstanceGeneration>,
    warm_up: Option<WarmUpOutput>,
    output: Option<TransactionOutput>,
    versions: Vec<PublishedVersion>,
    checks: Vec<StopCheck>,
    /// 暖机返回时整条录制流（含 mkfs）的长度。
    warm_up_operation_count: usize,
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
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
            &common::parameters(),
            &formatted.genesis,
        ));
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
            versions: Vec::new(),
            checks: Vec::new(),
            warm_up_operation_count: 0,
        }
    }

    /// 取号、暖机两次调用；两次都返回之后模型推一步。
    fn step_acquire_and_warm_up(&mut self) -> bool {
        let parameters = common::parameters();
        let genesis = self.formatted.genesis.clone();
        let devices = self
            .formatted
            .devices
            .as_mut()
            .expect("format_pool 交回开着的设备");
        let mut pool = PoolWriter::new(&parameters, devices);
        let instance = match self.driver.call("acquire", || acquire_instance(&mut pool)) {
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
        let warmed = match self
            .driver
            .call("warm_up", || warm_up(&mut pool, &genesis.root, instance))
        {
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

    /// 新池新建文件（`publish_first_file`）：按模型的 `answer_publish_first_file` 比。
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
        let result = self.driver.call("publish", || {
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
                false
            }
            Err(None) => false,
        }
    }

    /// 覆盖写（`publish_overwrite`，写时刻取本地常量 + 60）：按模型的 `answer_publish_overwrite` 比。
    fn step_overwrite(&mut self, content: &[u8], instance: InstanceGeneration) -> bool {
        let parameters = common::parameters();
        let previous = self.output.as_ref().expect("上一版带文件才走到这一步");
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let stream_length_before = self.driver.stream.operation_count();
        let result = self.driver.call("publish", || {
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
                false
            }
            Err(None) => false,
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
        let result = self.driver.call("mount_writable", || {
            let mut devices = formatted.reopen_recorded();
            let mounted = mount_writable(&parameters, &mut devices);
            formatted.devices = Some(devices);
            mounted
        });
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
                                singlefs_verification::model::ModelCheckpointTxg(raised_floor.0),
                            )
                        },
                        ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                            roots: raised
                                .publishes
                                .iter()
                                .map(observed_root_of_file_version)
                                .collect(),
                            reported_ceiling: Some(
                                singlefs_verification::model::ModelCheckpointTxg(raised.ceiling.0),
                            ),
                        }),
                    );
                }
                self.allocator = mounted.allocator;
                self.output = mounted.current.into_file_version();
                if self.output.is_none() {
                    self.driver.halted = true;
                    self.driver.implementation_crash =
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
                false
            }
            Err(None) => false,
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
        let result = self.driver.call("roll_back", || {
            roll_back_by_a_forward_publish(
                &parameters,
                devices,
                &mut self.allocator,
                &mut *output,
                target,
            )
        });
        match result {
            Ok(_rolled_back) => {
                let observed = ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: vec![observed_root_of_file_version(output)],
                    reported_ceiling: None,
                });
                self.driver.judge(
                    |model| model.answer_rollback_while_mounted(target_key),
                    observed,
                );
                true
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_rollback_error(&error),
                    0,
                    stream_length_before,
                    None,
                    root_ring_slot_still_bad_after_one_reread_of_rollback_error(&error),
                );
                self.driver.judge(
                    |model| model.answer_rollback_while_mounted(target_key),
                    observed,
                );
                false
            }
            Err(None) => false,
        }
    }

    /// 抬 F 到 txg 11（`raise_rollback_floor`，影子账开）：按模型的 `answer_raise_rollback_floor` 比（拒与成都要与实现报的上限相等）。
    fn step_raise_floor(&mut self) -> bool {
        let new_floor = CheckpointTxg(11);
        let parameters = common::parameters();
        let mut current = self.output.clone().expect("上一版带文件才走到这一步");
        let txg_before_the_raise = current.root.checkpoint_txg.0;
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let stream_length_before = self.driver.stream.operation_count();
        let result = self.driver.call("raise_floor", || {
            raise_rollback_floor(
                &parameters,
                devices,
                &mut self.allocator,
                &mut current,
                new_floor,
                ShadowLedger::On,
            )
        });
        let publishes_completed = usize::try_from(
            current
                .root
                .checkpoint_txg
                .0
                .saturating_sub(txg_before_the_raise),
        )
        .expect("一次抬 F 至多推根环区域数那么多次");
        self.output = Some(current);
        let floor = singlefs_verification::model::ModelCheckpointTxg(new_floor.0);
        match result {
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
                    reported_ceiling: Some(singlefs_verification::model::ModelCheckpointTxg(
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
        }
    }

    /// 正常卸载（`unmount`，入口记在录制流里）：模型没有卸载的答，不比；模型停在这里。
    fn step_unmount(&mut self) -> bool {
        let mut current = PoolVersion::WithFile(std::sync::Arc::new(
            self.output.clone().expect("上一版带文件"),
        ));
        let stream = self.driver.stream.clone();
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let result = self.driver.call(UNMOUNT_CALL_LABEL, || {
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
        self.driver
            .note_that_the_model_has_no_answer_for_the_last_call();
        self.output = current.into_file_version();
        match result {
            Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
                self.checks.push(StopCheck::new(
                    "second_stream_unmount_txgs",
                    LOCAL_SECOND_STREAM_UNMOUNT_TXGS.to_vec(),
                    unmounted_txgs_of(&raised),
                ));
                true
            }
            Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) | Err(_) => false,
        }
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
    /// 整条流的脚本：任一步出错或 panic 就停（这条流不再往下驱动）。流一是取号、暖机、新池新建文件；流二接着照
    /// `tests/crash_enumeration_fixed_script_stream.rs` 的脚本走到 E 再正常卸载。每一步成功之后才往崩溃点重放手写 oracle 的版本表里记一版。
    fn run_the_script(&mut self) {
        if !self.step_acquire_and_warm_up() || !self.step_publish_first_file() {
            return;
        }
        if self.driver.name == StreamName::NewPoolFileCreation {
            return;
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
            Some(LOCAL_SECOND_STREAM_REUSED_VERSION_DATA_UNIT_SLOT),
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
    /// 模型的版本目录的内容：取样态分几条线程判，这份不用 `Rc`，每条线程照它造自己那一份（[`committed_versions_for_this_thread`]）。
    committed_version_contents: BTreeMap<ModelRootKey, Option<Vec<u8>>>,
    /// 模型不再往前走的那次调用（`None`：整条流的每次调用都比过了）。
    model_stopped_at_call: Option<usize>,
    checks: Vec<StopCheck>,
    ending: StreamEnding,
}

/// 驱动怎么结束的：每一步都成功；某一步交回错误（这条流在那里截断，仍照常判）；某一步 panic（实现自崩，这条流不导出）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum StreamEnding {
    Completed,
    StoppedByAnError { call_index: usize, member: String },
    ImplementationCrashed { what: String },
}

/// 驱动一条流并取出它的基镜像、写表与段。`check_against_the_crates` 只在基线那一趟开：流一整条流逐条与 `build_pool` 比（S1）。
fn prepare_stream(
    name: StreamName,
    switches: RunSwitches,
    check_against_the_crates: bool,
) -> PreparedStream {
    let tag = format!("e171-{}", name.directory_name());
    let mut live = LiveStream::start(name, &tag, switches);
    live.run_the_script();
    let ending = match (&live.driver.implementation_crash, live.driver.halted) {
        (Some(what), _) => StreamEnding::ImplementationCrashed { what: what.clone() },
        (None, true) => {
            let call_index = live.driver.calls.len() - 1;
            StreamEnding::StoppedByAnError {
                call_index,
                member: live.driver.calls[call_index].outcome.outcome_word(),
            }
        }
        (None, false) => StreamEnding::Completed,
    };
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
            singlefs_verification::memory_pool::root_identity_of_write(&writes[judged_root_index]);
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
        committed_version_contents: std::mem::take(&mut live.driver.committed_versions)
            .into_iter()
            .map(|(key, file)| (key, file.map(|content| content.to_vec())))
            .collect(),
        model_stopped_at_call: live.driver.model_stopped_at_call,
        checks,
        ending,
    }
}

/// S1 里跟流有关的几条（只在基线那一趟）：段长表、写数、根槽写数、盘数与盘字节数对本地常量；流一整条流逐条与 `build_pool` 比。
/// 与 `e170_specification_executor_export.rs` 的 `prepare_stream` 里那一批 S1 回比同一批。
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
            StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => (
                &LOCAL_SECOND_STREAM_SEGMENT_LENGTHS,
                LOCAL_SECOND_STREAM_WRITES,
                LOCAL_SECOND_STREAM_ROOT_WRITES,
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
const IN_PLACE_SUBSETS_FILE: &str = "in-place-subsets.tsv";
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
    IN_PLACE_SUBSETS_FILE,
];

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
            let kind = RecordedOperationKind::ALL
                .into_iter()
                .find(|candidate| candidate.name() == fields[2])
                .unwrap_or_else(|| panic!("ops.tsv 里认不出的种类 {:?}", fields[2]));
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
    root_without_record: bool,
    claimed_state_missing_unit: bool,
}

impl ImplementationVerdict {
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
            root_without_record: judgement.records.root_without_record,
            claimed_state_missing_unit: judgement.records.claimed_state_missing_unit,
        }
    }
}

// ───────────────────────────── 每个取样状态上的三条 oracle（O1-崩溃判官、O2；O3 在 Python 一侧） ─────────────────────────────

/// O1-崩溃判官在一个状态上的结论（登记第一节读法 1、6）。
#[derive(Clone, Debug, PartialEq, Eq)]
enum CrashJudgeVerdict {
    /// 这一态有写落在模型没有答的调用里，或落在对拍判出不同（或出错）之后的调用里（读法 6）。
    NotApplicable,
    Green,
    /// 红：模型那一侧说的话（`ModelDisagreement::implementation_answer`，空白换成下划线）。
    Red {
        what: String,
    },
    /// 关一条（Q3-核）关掉了 O1：这一列不算。
    Off,
}

impl CrashJudgeVerdict {
    fn word(&self) -> &'static str {
        match self {
            Self::NotApplicable => "na",
            Self::Green => "green",
            Self::Red { .. } => "red",
            Self::Off => "off",
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
    fn text(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}",
            self.root, self.outcome, self.read_back_sha256, self.records_applied
        )
    }
}

// ───────────────────────────── O2 的盘两段：录入段只录、判段只判录下的 ─────────────────────────────

/// 录入段录下的一张要判的盘：表名与这张表录下的那一份的摘要（判段凭摘要读回）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct RecordedTableReference {
    table_name: String,
    digest: [u8; DIGEST_BYTES],
}

/// O2 要判的盘交给谁：录入段只录（只在 CPU 上，不碰给定状态引擎与卡），判段只判录下的。
/// 产品是 [`GivenStatesTablesInTheLibrary`]；单测换成内存里的替身，数录与判各几次。
trait PoolCheckerTablesOfTheTwoStages: Sync {
    /// 录入段：这张盘作一张给定状态表录下，交回判段读回它要的引用。
    fn record_a_table(&self, table_name: &str, pool: &MemoryPool) -> RecordedTableReference;

    /// 判段：读回录下的那张盘判，交回池级 checker 判红的不变量名（按 `IMPLEMENTED_INVARIANTS` 的次序，与 `check_pool_image` 交回的次序相同）。
    ///
    /// # Errors
    /// 交接与录下的对不上：照交接里的摘要读不回那张盘（没录过、读回的字节摘要不是它、解不出），一句话点名那张表与出路（重跑录入段）。
    ///
    /// # Panics
    /// 引擎判不了这张盘（起不了、判的时候失败、库读写不了；不退回别处判）。
    fn judge_a_recorded_table(&self, table: &RecordedTableReference)
        -> Result<Vec<String>, String>;
}

/// O2 两段的产品那一种。录入段：这张盘作一张给定状态表（基线就是那张盘、写表空、一个空持久集合），照给定状态引擎录入段写进库的那一种字节
/// （`RecordedGivenStatesTableWithItsColumns`，两栏不填）按摘要写进这个进程的判定库（prune-server 里那一份；库里已有逐字节相同的那一份就不再写），
/// 不起引擎。判段：照摘要从库里读回，交 [`given_states_engine_of_this_experiment`] 判（引擎在判段第一次判时才起：配了 GPU 且编进了 `gpu` 特性在卡上判，
/// 否则由 CPU 判器判；同一份字节引擎照录入键命中、不再写，判过的复用）。
struct GivenStatesTablesInTheLibrary;

impl GivenStatesTablesInTheLibrary {
    /// 这个进程的判定库（第一次用时开；与引擎开的是同一份，`LibraryOpenInThisProcess::open_or_share`）。
    ///
    /// # Panics
    /// 库开不了（连不上 prune-server 时 `open_or_share` 照它的退出码退出进程）。
    fn library() -> &'static LibraryOpenInThisProcess {
        static LIBRARY: std::sync::OnceLock<LibraryOpenInThisProcess> = std::sync::OnceLock::new();
        LIBRARY.get_or_init(|| {
            LibraryOpenInThisProcess::open_or_share()
                .unwrap_or_else(|refusal| panic!("开不了判定库：{refusal}"))
        })
    }
}

impl PoolCheckerTablesOfTheTwoStages for GivenStatesTablesInTheLibrary {
    fn record_a_table(&self, table_name: &str, pool: &MemoryPool) -> RecordedTableReference {
        use singlefs_verification::crash_given_states::GivenStatesOfATable;
        let table = GivenStatesOfATable {
            table_name: table_name.to_string(),
            base: pool,
            writes: &[],
            segments: &[],
            seams: &[],
            persisted_sets: vec![Vec::new()],
        };
        let recorded_bytes = RecordedGivenStatesTableWithItsColumns::of((&table).into()).to_bytes();
        let digest = sha256_digest(&recorded_bytes);
        let stored = Self::library()
            .lock()
            .read_given_states_table(&digest)
            .unwrap_or_else(|error| panic!("给定状态表 {table_name} 读不了库：{error:?}"));
        if stored.as_deref() != Some(recorded_bytes.as_slice()) {
            Self::library()
                .lock()
                .store_given_states_table(&digest, &recorded_bytes)
                .unwrap_or_else(|error| panic!("给定状态表 {table_name} 写不进库：{error:?}"));
        }
        RecordedTableReference {
            table_name: table_name.to_string(),
            digest,
        }
    }

    fn judge_a_recorded_table(
        &self,
        table: &RecordedTableReference,
    ) -> Result<Vec<String>, String> {
        use singlefs_verification::crash_given_states::{
            GivenStatesConclusionsSource, GivenStatesJudge,
        };
        let table_name = &table.table_name;
        let recorded_bytes = Self::library()
            .lock()
            .read_given_states_table(&table.digest)
            .unwrap_or_else(|error| panic!("给定状态表 {table_name} 读不了库：{error:?}"))
            .ok_or_else(|| {
                format!(
                    "交接里的给定状态表 {table_name} 不在库里：先跑录入段（{STAGE_ARGUMENT} record，或不给 {STAGE_ARGUMENT}、两段先后跑）"
                )
            })?;
        if sha256_digest(&recorded_bytes) != table.digest {
            return Err(format!(
                "交接里的给定状态表 {table_name} 从库里读回的字节摘要不是交接里录的那一个：重跑录入段（{STAGE_ARGUMENT} record）"
            ));
        }
        let recorded = RecordedGivenStatesTableWithItsColumns::from_bytes(&recorded_bytes)
            .ok_or_else(|| {
                format!(
                    "交接里的给定状态表 {table_name} 从库里读回的字节解不出：重跑录入段（{STAGE_ARGUMENT} record）"
                )
            })?;
        let judged = recorded
            .with_the_table(|table_read_back| {
                given_states_engine_of_this_experiment()
                    .judge_the_given_states_with_its_columns(table_read_back)
            })
            .unwrap_or_else(|reason| {
                panic!("给定状态表 {table_name} 判不了（不退回别处判）：{reason}")
            });
        match judged.source {
            GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost => {}
            GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost
            | GivenStatesConclusionsSource::PlaceholderOfATableRecordedOnly => panic!(
                "装置的判段不分片、引擎不分段：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE 与 SINGLEFS_CRASH_AMPLIFICATION_SHARE（{table_name}）"
            ),
        }
        let [conclusion] = judged.states.as_slice() else {
            panic!(
                "{table_name}：一个状态一份结论，交回 {} 份",
                judged.states.len()
            );
        };
        Ok(conclusion
            .pool_checker_violations()
            .into_iter()
            .map(|(invariant, _)| invariant.to_string())
            .collect())
    }
}

/// 这份装置判段判盘的给定状态引擎（进程里一份，判段第一次判时起；录入段不起它）。产物只收这份装置自己打的行：引擎不调 `finish`，不打收尾行。
///
/// # Panics
/// 引擎起不了（配了 GPU 卡用不上、结论版本与编译时对不上……都不退回 CPU）。
fn given_states_engine_of_this_experiment(
) -> &'static singlefs_verification::crash_amplification::GivenStatesEngine {
    static ENGINE: std::sync::OnceLock<
        singlefs_verification::crash_amplification::GivenStatesEngine,
    > = std::sync::OnceLock::new();
    ENGINE.get_or_init(|| {
        singlefs_verification::crash_amplification::GivenStatesEngine::start_from_environment(
            singlefs_verification::crash_amplification::GivenStatesItem {
                item_name: "e171_three_oracle_collusion::fixed".to_string(),
            },
            &singlefs_verification::crash_point_replay_progress::CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
        .unwrap_or_else(|refusal| panic!("起不了给定状态表的引擎：{refusal}"))
    })
}

/// 只有 `Violated` 算判红的那几条的名字。O2 的名单走两段（[`PoolCheckerTablesOfTheTwoStages`]），
/// 这个函数只给单测用：登记第九节 X6 那条（`only_a_violated_invariant_counts_as_a_red_pool_checker_verdict`）与不上卡的判法（`pool_checker_red_names_of_a_given_image_on_the_cpu`）。
#[cfg(test)]
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

/// 一个取样状态上看到的：批号、崩溃状态序号、段号、落地向量、镜像摘要、`judge_crash_image` 的判定（`impl-verdicts.tsv`）、
/// 三条 oracle 的结论与各自耗时、触达的最后一次调用。
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
    /// O2：关掉、录入段录下了这一态要判的盘、判段判完。
    pool_checker: PoolCheckerOfAState,
    consulted: RecoverySummary,
    ignored: RecoverySummary,
    crash_judge_nanoseconds: u128,
    pool_checker_nanoseconds: u128,
    recovery_nanoseconds: u128,
}

/// 一个取样态上 O2 走到哪一步。
#[derive(Clone, Debug, PartialEq, Eq)]
enum PoolCheckerOfAState {
    /// 关一条（Q3-核）关掉了 O2：这一列写 `off`，S3 不比。
    Off,
    /// 录入段把这一态要判的盘录下了，判段还没判。
    RecordedForTheJudgeStage(RecordedTableReference),
    /// 判段判完：判红的不变量名，与 S3（判出的名单与 `judge_crash_image(...).pool_checker` 判红的不变量集合相同；O2 阳性对照那一趟不比，记相同）。
    Judged {
        red: Vec<String>,
        matches_the_judgement: bool,
    },
}

impl StateObservation {
    /// O2 判红的不变量（关掉 O2 时 None）。
    ///
    /// # Panics
    /// 这一态录下的盘判段还没判（`oracle-verdicts.tsv` 与结果行都在判段判完之后才写）。
    fn pool_checker_red(&self) -> Option<&[String]> {
        match &self.pool_checker {
            PoolCheckerOfAState::Off => None,
            PoolCheckerOfAState::Judged { red, .. } => Some(red.as_slice()),
            PoolCheckerOfAState::RecordedForTheJudgeStage(table) => panic!(
                "{} 判段还没判：O2 的名单在判段判完之后才读",
                table.table_name
            ),
        }
    }

    /// S3：O2 判出的名单与 `judge_crash_image(...).pool_checker` 判红的不变量集合相同（关掉 O2 时不比，记相同）。
    ///
    /// # Panics
    /// 同 [`StateObservation::pool_checker_red`]。
    fn pool_checker_matches_the_judgement(&self) -> bool {
        match &self.pool_checker {
            PoolCheckerOfAState::Off => true,
            PoolCheckerOfAState::Judged {
                matches_the_judgement,
                ..
            } => *matches_the_judgement,
            PoolCheckerOfAState::RecordedForTheJudgeStage(table) => {
                panic!("{} 判段还没判：S3 在判段判完之后才比", table.table_name)
            }
        }
    }
}

/// 一次观测的状态：阳性对照里「批 1 第一个读回了文件的态」只翻一次。「第一个」要一条线程按序号走才说得清，
/// 这个阳性对照那一趟只开一条线程（[`sampling_threads_given_the_positive_control`]）。
struct ObservationControls {
    switches: RunSwitches,
    crash_judge_control_pending: AtomicBool,
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

/// O1-崩溃判官在一个状态上的判定：交法照 `crash_injection.rs` 的 `inject_crashes_into_history_with_every_switch` 里问理想模型那一段——「盘上已持久的最新根槽」由 `newest_persisted_root`
/// 从写流数出（只看写表里的原写，不含撕裂镜像），读回由 `observed_read_back_after_a_crash` 从看 journal 那一遍恢复报告造，
/// 模型那一侧给每一步之后并起来的版本目录，交给 `crash_recovery_disagreement`（S8：三处调用都在这个函数里）。
fn crash_judge_of_the_state(
    stream: &PreparedStream,
    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    image: &CrashImage<'_>,
    consulted: &RecoveryReport,
    batch: u8,
    controls: &ObservationControls,
) -> CrashJudgeVerdict {
    let newest_persisted_root_key =
        newest_persisted_root(&stream.writes, &image.persisted[..stream.writes.len()])
            .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
    let mut read_back = observed_read_back_after_a_crash(consulted);
    if controls.crash_judge_control_pending.load(Ordering::SeqCst) && batch == 1 {
        if let ObservedReadBack::FileRead { content, .. } = &mut read_back {
            if let Some(last_byte) = content.last_mut() {
                *last_byte ^= 1;
                controls
                    .crash_judge_control_pending
                    .store(false, Ordering::SeqCst);
            }
        }
    }
    crash_judge_verdict(committed_versions, newest_persisted_root_key, &read_back)
}

/// 这条线程自己的一份模型版本目录：`crash_recovery_disagreement` 收 `Rc`，`Rc` 不跨线程，照流里存的内容造。
fn committed_versions_for_this_thread(
    stream: &PreparedStream,
) -> BTreeMap<ModelRootKey, Option<Rc<[u8]>>> {
    stream
        .committed_version_contents
        .iter()
        .map(|(key, content)| (*key, content.as_deref().map(Rc::<[u8]>::from)))
        .collect()
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

/// 服务交给这一趟的内存上限（字节）所在的环境变量；没设（不经服务起）就不按内存收线程。
const MEMORY_CAP_ENVIRONMENT_VARIABLE: &str = "TASK_SERVER_MEMORY_CAP_BYTES";

/// 内存上限里留给取样线程之外的（两条流的驱动、枚举表、事实表、写文件、引擎与库的连接）：只开一条线程的 E171 取样段，
/// 整个进程的峰值不到 1.5 GiB（`REPLAY_ROW` 的在跑行），这里按它留出余量。
const MEBIBYTES_OUTSIDE_THE_SAMPLING_THREADS: u64 = 2048;

/// 一条取样线程要留的内存：录入段的流字节、读回的流与它的计划、撕裂表、物化的盘、表的字节与读回的那一份、判表与打包判表各一份，
/// 固定脚本流的写流载荷约 7.2 MiB、撕裂载荷约 0.2 MiB（`ops-payload.bin`、`torn-payload.bin`），每样几份，这个数按它们估。
/// 判已知坏镜像与 E170 坏状态的线程同一份数：手里是一张盘的两三份拷贝（已知坏镜像只在新池新建文件流上，载荷约 0.5 MiB；E170 坏状态重建的盘不大于固定脚本流的载荷）、
/// 算摘要的消息、给定状态表的录入字节与读回的那一份、判表与打包判表，没有比取样线程多出来的一样。
const MEBIBYTES_OF_A_SAMPLING_THREAD: u64 = 256;

/// 取样线程的栈：与判定在主线程上跑时同样大（`std::thread::spawn` 缺省的栈比主线程小，恢复与 checker 不该因为换了线程换一种栈）。
const STACK_BYTES_OF_A_SAMPLING_THREAD: usize = 8 * 1024 * 1024;

/// 这一趟判取样态、判坏镜像开几条线程：`--sampling-threads` 给了照它，没给取每台默认重活线程数；再按内存上限与阳性对照收。
///
/// # Panics
/// 同 [`singlefs_verification::heavy_threads::default_heavy_threads`]；服务交的内存上限不是数。
fn sampling_threads_of_this_run(switches: RunSwitches) -> NonZeroUsize {
    let requested = switches
        .sampling_threads
        .unwrap_or_else(singlefs_verification::heavy_threads::default_heavy_threads);
    let memory_cap_bytes =
        singlefs_verification::numeric_environment_variable::number_from_the_variable_value_panicking_when_unreadable::<u64>(
            MEMORY_CAP_ENVIRONMENT_VARIABLE,
            singlefs_verification::numeric_environment_variable::process_environment_variable(
                MEMORY_CAP_ENVIRONMENT_VARIABLE,
            )
            .as_deref(),
        );
    sampling_threads_given_the_positive_control(
        sampling_threads_within_the_memory_cap(requested, memory_cap_bytes),
        switches,
    )
}

/// 按内存上限收线程数：上限扣掉 [`MEBIBYTES_OUTSIDE_THE_SAMPLING_THREADS`]，余下的放得下几条 [`MEBIBYTES_OF_A_SAMPLING_THREAD`] 就开几条，
/// 不多于要的、至少一条。线程数只定快慢，不进产物。
fn sampling_threads_within_the_memory_cap(
    requested: NonZeroUsize,
    memory_cap_bytes: Option<u64>,
) -> NonZeroUsize {
    let Some(memory_cap_bytes) = memory_cap_bytes else {
        return requested;
    };
    let mebibytes_for_the_threads =
        (memory_cap_bytes / (1024 * 1024)).saturating_sub(MEBIBYTES_OUTSIDE_THE_SAMPLING_THREADS);
    let threads_that_fit =
        usize::try_from(mebibytes_for_the_threads / MEBIBYTES_OF_A_SAMPLING_THREAD)
            .unwrap_or(usize::MAX);
    NonZeroUsize::new(requested.get().min(threads_that_fit)).unwrap_or(NonZeroUsize::MIN)
}

/// O1-崩溃判官的阳性对照要翻「批 1 第一个读回了文件的态」，先后由序号定：这一趟只开一条线程，别的趟照给的线程数。
fn sampling_threads_given_the_positive_control(
    threads: NonZeroUsize,
    switches: RunSwitches,
) -> NonZeroUsize {
    if switches.positive_control == Some(PositiveControl::CrashJudge) {
        NonZeroUsize::MIN
    } else {
        threads
    }
}

/// 把 `run_count` 个连续段（坏镜像模式里一条算一段）分给至多 `threads` 条线程判，交回的结局按连续段的次序排，与哪条线程先判完无关：线程按次序领下一个连续段，
/// 判完放回它自己的位置。某个连续段的结局叫停（`stops_the_later_runs`）之后不再领新的，交回的到次序上第一个叫停的连续段为止（含它）：
/// 领的次序就是连续段的次序，比它靠前的都领过、都判完了，所以这一份与一条线程按次序走到那里停下的那一份相同；领了而排在它后面的丢掉。
///
/// # Panics
/// `outcome_of_the_run` panic（它自己要接住连续段里的 panic）；起不了线程。
fn outcomes_of_the_runs_in_their_order<Outcome: Send>(
    run_count: usize,
    threads: NonZeroUsize,
    stops_the_later_runs: &(dyn Fn(&Outcome) -> bool + Sync),
    outcome_of_the_run: &(dyn Fn(usize) -> Outcome + Sync),
) -> Vec<Outcome> {
    let next_run = AtomicUsize::new(0);
    let later_runs_are_stopped = AtomicBool::new(false);
    let outcomes: Vec<Mutex<Option<Outcome>>> =
        (0..run_count).map(|_run| Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads.get().min(run_count))
            .map(|_worker| {
                let next_run = &next_run;
                let later_runs_are_stopped = &later_runs_are_stopped;
                let outcomes = &outcomes;
                std::thread::Builder::new()
                    .name("e171-judging".to_string())
                    .stack_size(STACK_BYTES_OF_A_SAMPLING_THREAD)
                    .spawn_scoped(scope, move || loop {
                        if later_runs_are_stopped.load(Ordering::SeqCst) {
                            break;
                        }
                        let run = next_run.fetch_add(1, Ordering::SeqCst);
                        if run >= run_count {
                            break;
                        }
                        let outcome = outcome_of_the_run(run);
                        if stops_the_later_runs(&outcome) {
                            later_runs_are_stopped.store(true, Ordering::SeqCst);
                        }
                        *outcomes[run].lock().unwrap_or_else(PoisonError::into_inner) =
                            Some(outcome);
                    })
                    .expect("起得了判取样态的线程")
            })
            .collect();
        for worker in workers {
            worker
                .join()
                .expect("连续段里的 panic 都在 outcome_of_the_run 里接住了");
        }
    });
    let mut outcomes_in_their_order = Vec::with_capacity(run_count);
    for outcome_of_a_run in outcomes {
        let outcome = outcome_of_a_run
            .into_inner()
            .unwrap_or_else(PoisonError::into_inner)
            .expect("次序上第一个叫停的连续段之前的都领过、判完了：领的次序就是连续段的次序");
        let stops = stops_the_later_runs(&outcome);
        outcomes_in_their_order.push(outcome);
        if stops {
            break;
        }
    }
    outcomes_in_their_order
}

/// 一件里 panic 时接住的载荷：交回主线程，排到它的次序上再原样抛出。
type CaughtPanicPayload = Box<dyn std::any::Any + Send>;

/// [`outcomes_of_the_runs_in_their_order`] 按一件一件判、接住每件里的 panic：次序上第一个 panic 的件之后不再领新的，它之前的都判完了。
/// 交回的 `Err` 只可能是最后一个，主线程把它之前的结局照次序收完再 [`std::panic::resume_unwind`]：打出的行与一条线程走到那里 panic 时相同。
///
/// # Panics
/// 起不了线程。
fn outcomes_up_to_the_earliest_panic<Outcome: Send>(
    item_count: usize,
    threads: NonZeroUsize,
    outcome_of_the_item: &(dyn Fn(usize) -> Outcome + Sync),
) -> Vec<Result<Outcome, CaughtPanicPayload>> {
    outcomes_of_the_runs_in_their_order(
        item_count,
        threads,
        &|outcome: &Result<Outcome, CaughtPanicPayload>| outcome.is_err(),
        &|item| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| outcome_of_the_item(item)))
        },
    )
}

/// 一张给定的盘上池级 checker 判红的不变量名：表名、那张盘。单测的 O2 两段替身（`PoolCheckerTablesKeptInMemory`）照它判读回的盘；
/// 产品的判段是 [`GivenStatesTablesInTheLibrary`]。
#[cfg(test)]
type PoolCheckerRedNamesOfAGivenImage<'judge> =
    dyn Fn(&str, &MemoryPool) -> Vec<String> + Sync + 'judge;

/// 录入段录下一个取样态上 O2 要判的盘：流名、序号、镜像摘要、这一态的镜像、这一态是不是 O2 阳性对照改坏的那一张。
/// 产品路径是 [`pool_checker_table_of_a_sampled_state`]；由参数给，线程对拍能在单测里换别的录法。
type PoolCheckerTableOfASampledState<'record> =
    dyn Fn(StreamName, u64, &str, &CrashImage<'_>, bool) -> RecordedTableReference + Sync + 'record;

/// 取样中途某个状态上实现 panic（实现自崩）：那一句（`<panic 的消息>:after_ordinal_<崩之前最后判完的序号>`），与崩之前判完的那几态（按序号升序）。
/// 那几态录下的盘判段照样判：引擎在其中一态上失败时，两段之前在取样回调里先撞上的是引擎那一次（[`judge_the_recorded_pool_checker_tables`]）。
struct ObservationsUpToACrashInAState {
    crash: String,
    observations: Vec<StateObservation>,
}

/// 一个连续段判完的结局：判完的那几态（连续段中途 panic 时是 panic 之前判完的），与 `sample_crash_states_on_the_cpu` 交回的整条流状态数或 panic 的消息。
struct ObservationsOfARun {
    observations: Vec<StateObservation>,
    ending: Result<u64, String>,
}

/// 按序号升序把取样的状态切成连续的几段，每段调一次 `judge_crash_state_range`：回调里给的那个 `CrashImage` 折回落地向量、算摘要，
/// 再算 O1-崩溃判官，O2 只在 CPU 上计时、要判的盘交 `pool_checker_table` 录下（录入段：判归判段，[`judge_the_recorded_pool_checker_tables`]），
/// 并单独计恢复的耗时。回调的持久集合另与 `crash_facts` 按序号现算的掩码比。
/// 这几段分给 `threads` 条线程（[`outcomes_of_the_runs_in_their_order`]），每态各交各的表（表名、表的内容与一条线程时相同，判定库的键不变），
/// 结果按（批号、序号）排好：除了耗时几列，与 `threads` 是几无关。
/// 某个状态上实现 panic（实现自崩）：交回 `Err`，那一句带次序上第一个 panic 的态之前最后一个判完的序号，另带 panic 之前判完的那几态。
#[allow(
    clippy::too_many_arguments,
    reason = "前五个是这条流的几样输入，其余是开关、线程数与录 O2 要判的盘的办法"
)]
// gpu-exempt: experiment-measures-the-cpu-judge-itself
fn observe_sampled_states(
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    facts: &FlowFacts,
    torn_image_index_by_write: &BTreeMap<usize, usize>,
    sampled: &SampledOrdinals,
    switches: RunSwitches,
    threads: NonZeroUsize,
    pool_checker_table: &PoolCheckerTableOfASampledState<'_>,
) -> Result<Vec<StateObservation>, ObservationsUpToACrashInAState> {
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
    let ordinals: Vec<u64> = batch_of_ordinal.keys().copied().collect();
    let mut runs: Vec<std::ops::Range<u64>> = Vec::new();
    for ordinal in ordinals {
        match runs.last_mut() {
            Some(run) if run.end == ordinal => run.end += 1,
            Some(_) | None => runs.push(ordinal..ordinal + 1),
        }
    }
    let recorded_write_count = stream.writes.len();
    let call_of_write = call_index_of_each_write(stream);
    let model_is_enabled =
        switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge);
    let pool_checker_is_enabled = switches.disabled_oracle != Some(DisabledOracle::PoolChecker);
    let controls = ObservationControls {
        switches,
        crash_judge_control_pending: AtomicBool::new(
            switches.positive_control == Some(PositiveControl::CrashJudge),
        ),
    };
    let sampling_item_names: Vec<String> = runs
        .iter()
        .map(|_run| sampling_item_name_of_this_call())
        .collect();
    let observe_a_run = |run_position: usize| -> ObservationsOfARun {
        let committed_versions = committed_versions_for_this_thread(stream);
        let mut observations_of_the_run: Vec<StateObservation> = Vec::new();
        let judged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            sample_crash_states_on_the_cpu(
                CrashPointReplayItem {
                    name: &sampling_item_names[run_position],
                    setting_file: CRASH_POINT_REPLAY_SAMPLING_ITEM.setting_file,
                },
                &stream.base,
                &stream.writes,
                &stream.segments,
                stream.judged_root_index,
                &stream.versions,
                &full_expansion,
                runs[run_position].clone(),
                &mut |ordinal, image, judgement| {
                    let batch = batch_of_ordinal[&ordinal];
                    let landing_vector = landing_vector_of(
                        &image.persisted,
                        recorded_write_count,
                        torn_image_index_by_write,
                    );
                    let touched_call = landing_vector
                        .chars()
                        .zip(&call_of_write)
                        .filter(|(landing, _)| *landing != '0')
                        .filter_map(|(_, call)| *call)
                        .max();
                    let verdict = ImplementationVerdict::of(judgement);
                    let crash_judge_started = Instant::now();
                    let crash_judge = if !model_is_enabled {
                        CrashJudgeVerdict::Off
                    } else if !crash_judge_is_applicable(touched_call, stream.model_stopped_at_call)
                    {
                        CrashJudgeVerdict::NotApplicable
                    } else {
                        crash_judge_of_the_state(
                            stream,
                            &committed_versions,
                            image,
                            &judgement.consulted,
                            batch,
                            &controls,
                        )
                    };
                    let crash_judge_nanoseconds = crash_judge_started.elapsed().as_nanos();
                    let pool_checker_control = controls.switches.positive_control
                        == Some(PositiveControl::PoolChecker)
                        && stream.name == StreamName::NewPoolFileCreation
                        && ordinal == tables.state_count - 1;
                    // O2 逐状态计时量的是 CPU 池级 checker 本身（豁免截名 experiment-measures-the-cpu-judge-itself）：CPU 这一遍只计时；
                    // 判红的名单走两段（同一张盘作一张给定状态表录下、判段交引擎判，阳性对照那一态是改坏的那一张），不在计时里
                    let pool_checker_started = Instant::now();
                    let pool_checker_timed_on_the_cpu = pool_checker_is_enabled.then(|| {
                        if pool_checker_control {
                            pool_checker_on_the_first_known_bad_image(image)
                        } else {
                            check_pool_image(image)
                        }
                    });
                    let pool_checker_nanoseconds = pool_checker_started.elapsed().as_nanos();
                    let image_digest = sha256_text(&crash_image_digest(image));
                    let pool_checker = if pool_checker_timed_on_the_cpu.is_some() {
                        PoolCheckerOfAState::RecordedForTheJudgeStage(pool_checker_table(
                            stream.name,
                            ordinal,
                            &image_digest,
                            image,
                            pool_checker_control,
                        ))
                    } else {
                        PoolCheckerOfAState::Off
                    };
                    let recovery_started = Instant::now();
                    let _ = singlefs_core::recovery::recover(
                        image,
                        singlefs_core::recovery::JournalPolicy::Consult,
                    );
                    let recovery_nanoseconds = recovery_started.elapsed().as_nanos();
                    observations_of_the_run.push(StateObservation {
                        batch,
                        ordinal,
                        segment_index: tables
                            .state_ranges_by_segment
                            .partition_point(|state_range| state_range.end <= ordinal),
                        landing_vector,
                        image_digest,
                        verdict,
                        judge_nanoseconds: crash_judge_nanoseconds
                            + pool_checker_nanoseconds
                            + recovery_nanoseconds,
                        mask_matches_the_facts: facts.persisted_writes_of_state(ordinal)
                            == image.persisted,
                        touched_call,
                        crash_judge,
                        pool_checker,
                        consulted: RecoverySummary::of(&judgement.consulted),
                        ignored: RecoverySummary::of(&judgement.ignored),
                        crash_judge_nanoseconds,
                        pool_checker_nanoseconds,
                        recovery_nanoseconds,
                    });
                },
            )
            .judged()
            .expect("实验装置不分只录、只判两趟跑：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE")
        }));
        ObservationsOfARun {
            observations: observations_of_the_run,
            ending: judged.map_err(|payload| panic_text(&*payload)),
        }
    };
    eprintln!(
        "E171_SAMPLED_STATES stream={} runs={} threads={threads}",
        stream.name.directory_name(),
        runs.len()
    );
    let mut observations: Vec<StateObservation> = Vec::new();
    for observed in outcomes_of_the_runs_in_their_order(
        runs.len(),
        threads,
        &|observed: &ObservationsOfARun| observed.ending.is_err(),
        &observe_a_run,
    ) {
        observations.extend(observed.observations);
        match observed.ending {
            Ok(state_count_of_the_stream) => assert_eq!(
                state_count_of_the_stream, tables.state_count,
                "judge_crash_state_range 交回的整条流状态数与枚举表的相同"
            ),
            Err(what) => {
                let crash = format!(
                    "{what}:after_ordinal_{}",
                    observations
                        .last()
                        .map_or_else(|| "none".to_string(), |last| last.ordinal.to_string())
                );
                return Err(ObservationsUpToACrashInAState {
                    crash,
                    observations,
                });
            }
        }
    }
    observations.sort_by_key(|observation| (observation.batch, observation.ordinal));
    Ok(observations)
}

/// 判段判完一条流录下的盘。
enum JudgedTablesOfAStream {
    /// 录下的盘都判完了（这条流没有录下的盘也算）。
    AllJudged,
    /// 引擎判某一态的盘时失败（panic）：`what` 与两段之前取样回调里接住它时的那一句相同（`<panic 的消息>:after_ordinal_<按序号它前面那一态>`），
    /// 这条流照两段之前记成某个状态上实现自崩（[`StreamExport::record_an_engine_failure_in_a_state`]）。
    EngineFailedInAState { what: String },
}

/// 判段：录入段录下了盘的每个取样态按序号升序（两段之前取样回调判它们的次序）交 `tables` 判，分给 `threads` 条线程
/// （[`outcomes_up_to_the_earliest_panic`]），换成判完的名单；S3 照判出的名单与 `judge_crash_image` 那一遍判红的不变量比
/// （O2 阳性对照那一趟不比，记相同）。关掉 O2 的态不动。引擎在序号上第一个失败的态停下，交回
/// [`JudgedTablesOfAStream::EngineFailedInAState`]（它之前的态判完了，它与之后的不再往下走）。
///
/// # Errors
/// 交接与录下的对不上（[`PoolCheckerTablesOfTheTwoStages::judge_a_recorded_table`] 的 `Err`）：那一句原样交回。
fn judge_the_recorded_pool_checker_tables(
    observations: &mut [StateObservation],
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    threads: NonZeroUsize,
    switches: RunSwitches,
) -> Result<JudgedTablesOfAStream, String> {
    let mut positions_in_the_ordinal_order: Vec<usize> = (0..observations.len()).collect();
    positions_in_the_ordinal_order.sort_by_key(|position| observations[*position].ordinal);
    let recorded_tables: Vec<(usize, RecordedTableReference)> = positions_in_the_ordinal_order
        .into_iter()
        .filter_map(|position| match &observations[position].pool_checker {
            PoolCheckerOfAState::RecordedForTheJudgeStage(table) => Some((position, table.clone())),
            PoolCheckerOfAState::Off | PoolCheckerOfAState::Judged { .. } => None,
        })
        .collect();
    let judged_in_the_ordinal_order =
        outcomes_up_to_the_earliest_panic(recorded_tables.len(), threads, &|item| {
            tables.judge_a_recorded_table(&recorded_tables[item].1)
        });
    let pool_checker_control_is_on =
        switches.positive_control == Some(PositiveControl::PoolChecker);
    let mut ordinal_judged_last: Option<u64> = None;
    for ((position, _table), judged) in recorded_tables.iter().zip(judged_in_the_ordinal_order) {
        let red = match judged {
            Ok(red_or_refusal) => red_or_refusal?,
            Err(payload) => {
                return Ok(JudgedTablesOfAStream::EngineFailedInAState {
                    what: format!(
                        "{}:after_ordinal_{}",
                        panic_text(&*payload),
                        ordinal_judged_last
                            .map_or_else(|| "none".to_string(), |ordinal| ordinal.to_string())
                    ),
                });
            }
        };
        let observation = &mut observations[*position];
        let matches_the_judgement =
            red == observation.verdict.pool_checker_red || pool_checker_control_is_on;
        observation.pool_checker = PoolCheckerOfAState::Judged {
            red,
            matches_the_judgement,
        };
        ordinal_judged_last = Some(observation.ordinal);
    }
    Ok(JudgedTablesOfAStream::AllJudged)
}

// ───────────────────────────── 一条流导出（登记 5.3 的十个文件，照抄 E170 的写出） ─────────────────────────────

/// 写出的一个文件：名字、字节数、sha256；带耗时的两个文件另算一份去掉耗时之后的 sha256（登记第十二节修订 V2b）。
struct WrittenFile {
    name: String,
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
        name: name.to_string(),
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
            RecordedOperationKind::WriteZeroes
            | RecordedOperationKind::Discard
            | RecordedOperationKind::Barrier => ("-".to_string(), "-".to_string()),
        };
        operations_text.push_str(&format!(
            "{stream_index}\t{}\t{}\t{}\t{}\t{payload_start}\t{content_digest}\n",
            operation.device.0,
            operation.kind.name(),
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

/// `call-outcomes.tsv`：每次调用一行——序号、名字、第一条 op、返回点、结局（`ok` / `err:<成员>` / `panic`）、O1-对拍的判定、
/// 对不上时模型与实现各说了什么。不含耗时（两次基线逐字节相同，作废条款 Z1）。
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
                | ModelVerdictOfCall::SkippedAfterTheModelStopped => {
                    ("-".to_string(), "-".to_string())
                }
            };
            let panic_message = match &call.outcome {
                CallOutcome::Panicked(text) => cell_text(text),
                CallOutcome::Succeeded | CallOutcome::ReturnedError(_) => "-".to_string(),
            };
            format!(
                "{call_index}\t{}\t{}\t{}\t{}\t{}\t{model_answer}\t{implementation_answer}\t{panic_message}\n",
                call.label,
                call.first_operation,
                call.return_point,
                call.outcome.outcome_word(),
                call.model.word()
            )
        })
        .collect()
}

/// `stream-status.tsv`：驱动怎么结束的、模型停在哪次调用、被判的根写在不在（`key<TAB>value`）。
fn stream_status_file_text(stream: &PreparedStream) -> String {
    let ending = match &stream.ending {
        StreamEnding::Completed => "completed".to_string(),
        StreamEnding::StoppedByAnError { call_index, member } => {
            format!("stopped_by_an_error:call_{call_index}:{member}")
        }
        StreamEnding::ImplementationCrashed { what } => {
            format!(
                "implementation_crashed:{}",
                cell_text(what).replace(' ', "_")
            )
        }
    };
    format!(
        "ending\t{ending}\nmodel_stopped_at_call\t{}\njudged_root_index\t{}\nwrites\t{}\ncalls\t{}\n",
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
/// O1-崩溃判官（`green` / `red` / `na` / `off`）与红的内容、O2（同）与判红的不变量、O2 直接调与 `judge_crash_image` 内那一遍是否一致、
/// 看 journal 那一遍恢复的摘要（根、结局、读回 sha256、施加记录数）、不看 journal 那一遍的摘要、崩溃点重放的 `red_items`（附带：记录核对器与手写版本表的 oracle）。
fn oracle_verdicts_file_text(observations: &[StateObservation]) -> String {
    observations
        .iter()
        .enumerate()
        .map(|(state_number, observation)| {
            let (pool_checker_word, pool_checker_items) = match observation.pool_checker_red() {
                None => ("off", "-".to_string()),
                Some([]) => ("green", "-".to_string()),
                Some(items) => ("red", items.join(",")),
            };
            let crash_judge_detail = match &observation.crash_judge {
                CrashJudgeVerdict::Red { what } => cell_text(what),
                CrashJudgeVerdict::Green
                | CrashJudgeVerdict::NotApplicable
                | CrashJudgeVerdict::Off => "-".to_string(),
            };
            format!(
                "{state_number}\t{}\t{}\t{}\t{}\t{}\t{crash_judge_detail}\t{pool_checker_word}\t{pool_checker_items}\t{}\t{}\t{}\t{}\n",
                observation.batch,
                observation.ordinal,
                observation.segment_index,
                observation
                    .touched_call
                    .map_or_else(|| "-".to_string(), |call| call.to_string()),
                observation.crash_judge.word(),
                observation.pool_checker_matches_the_judgement(),
                observation.consulted.text(),
                observation.ignored.text(),
                field_text(&list_text(&observation.verdict.red_items)),
            )
        })
        .collect()
}

const ORACLE_VERDICTS_FILE: &str = "oracle-verdicts.tsv";
const CALL_OUTCOMES_FILE: &str = "call-outcomes.tsv";
const STREAM_STATUS_FILE: &str = "stream-status.tsv";

/// 按 E170 登记 5.3 写一条流的十个文件，manifest 最后写（它记前九个的 sha256）；之后再写本实验自己的三个：
/// `oracle-verdicts.tsv`、`call-outcomes.tsv`、`stream-status.tsv`（不进 manifest：manifest 与 E170 导出装置写的同一份逐字节可比，停机 S2）。
/// 装置分两段写（[`write_stream_files_before_the_judge_stage`]、[`write_the_oracle_verdicts_file`]）；这一份一趟写完，给单测比两段写的。
#[cfg(test)]
fn write_stream_files(
    directory: &Path,
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    observations: &[StateObservation],
    full_states: u64,
    in_place_subsets_states: u64,
) -> Vec<WrittenFile> {
    let mut files = write_stream_files_before_the_judge_stage(
        directory,
        stream,
        tables,
        observations,
        full_states,
        in_place_subsets_states,
    );
    write_the_oracle_verdicts_file(directory, observations, &mut files);
    files
}

/// 判段：写这条流的 `oracle-verdicts.tsv`（O2 那几列判完才有），排进写出的文件里 manifest 之后那一位（与一趟写完时的次序相同）。
///
/// # Panics
/// `files` 里没有 manifest（这条流没导出：没导出的流不写 `oracle-verdicts.tsv`）；O2 录下的盘还没判（[`StateObservation::pool_checker_red`]）。
fn write_the_oracle_verdicts_file(
    directory: &Path,
    observations: &[StateObservation],
    files: &mut Vec<WrittenFile>,
) {
    let written = write_export_file(
        directory,
        ORACLE_VERDICTS_FILE,
        oracle_verdicts_file_text(observations).as_bytes(),
    );
    let position_after_the_manifest = files
        .iter()
        .position(|file| file.name == MANIFEST_FILE)
        .expect("导出了的流写过 manifest")
        + 1;
    files.insert(position_after_the_manifest, written);
}

/// 录入段写得了的那几个导出文件：[`write_stream_files`] 里除 `oracle-verdicts.tsv` 之外的，次序相同（O2 那几列要等判段）。
fn write_stream_files_before_the_judge_stage(
    directory: &Path,
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    observations: &[StateObservation],
    full_states: u64,
    in_place_subsets_states: u64,
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
    let in_place_subsets_text =
        format!("in_place_subsets_states\t{in_place_subsets_states}\nfull_states\t{full_states}\n");
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
        write_export_file(
            directory,
            IN_PLACE_SUBSETS_FILE,
            in_place_subsets_text.as_bytes(),
        ),
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
        assert_eq!(
            *listed_name,
            file.name.as_str(),
            "manifest 的次序与写出的次序相同"
        );
        manifest_text.push_str(&format!("sha256:{}\t{}\n", file.name, file.sha256));
    }
    files.push(write_export_file(
        directory,
        MANIFEST_FILE,
        manifest_text.as_bytes(),
    ));
    files
        .extend(write_the_files_that_exist_even_when_the_stream_is_not_exported(directory, stream));
    files
}

/// [`write_the_files_that_exist_even_when_the_stream_is_not_exported`] 写的两个文件，按它写的次序。
const FILES_WRITTEN_EVEN_WHEN_THE_STREAM_IS_NOT_EXPORTED: [&str; 2] =
    [CALL_OUTCOMES_FILE, STREAM_STATUS_FILE];

/// 驱动里实现自崩、或没有根槽写可判的流不导出状态；这两个文件照写（比对器靠它们认出「实现自崩」与「出错截断」）。
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
            STREAM_STATUS_FILE,
            stream_status_file_text(stream).as_bytes(),
        ),
    ]
}

/// 一条流导出之后的样子。`not_exported` 有值：这条流没有导出状态（驱动里实现 panic、没有根槽写可判、某个状态上实现 panic、
/// 或流水线里别处 panic），只有 `call-outcomes.tsv` 与 `stream-status.tsv`（能写出来的话）。
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
    in_place_subsets_states: u64,
    torn_scans: Vec<TornScan>,
    observations: Vec<StateObservation>,
    files: Vec<WrittenFile>,
    rebuilt_digest_mismatches: usize,
    rebuilt_states: usize,
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
            in_place_subsets_states: 0,
            torn_scans: Vec::new(),
            observations: Vec::new(),
            files: Vec::new(),
            rebuilt_digest_mismatches: 0,
            rebuilt_states: 0,
            elapsed_nanoseconds: started.elapsed().as_nanos(),
            drive_nanoseconds: 0,
            observe_nanoseconds: 0,
            write_nanoseconds: 0,
        }
    }

    /// 判段里引擎判这条流某一态的盘时失败（`what` 见 [`JudgedTablesOfAStream::EngineFailedInAState`]）：照两段之前取样回调里接住它时那样，
    /// 这条流记成某个状态上实现自崩（`implementation_crashed_in_a_state:<what>`），盖过录入段记的那一句；`directory`（这条流的导出目录）里只留
    /// 照写的两个文件（[`FILES_WRITTEN_EVEN_WHEN_THE_STREAM_IS_NOT_EXPORTED`]，录入段写的别的导出文件删掉），取样之后才有的几样清零。
    ///
    /// # Panics
    /// 录入段写的导出文件删不掉。
    fn record_an_engine_failure_in_a_state(&mut self, directory: &Path, what: &str) {
        for file in &self.files {
            if !FILES_WRITTEN_EVEN_WHEN_THE_STREAM_IS_NOT_EXPORTED.contains(&file.name.as_str()) {
                let path = directory.join(&file.name);
                std::fs::remove_file(&path)
                    .unwrap_or_else(|error| panic!("删 {}：{error}", path.display()));
            }
        }
        self.files.retain(|file| {
            FILES_WRITTEN_EVEN_WHEN_THE_STREAM_IS_NOT_EXPORTED.contains(&file.name.as_str())
        });
        self.not_exported = Some(format!("implementation_crashed_in_a_state:{what}"));
        self.tearable_write_count = 0;
        self.full_states = 0;
        self.in_place_subsets_states = 0;
        self.torn_scans = Vec::new();
        self.observations = Vec::new();
        self.rebuilt_digest_mismatches = 0;
        self.rebuilt_states = 0;
        self.observe_nanoseconds = 0;
        self.write_nanoseconds = 0;
    }
}

/// 基线与坏实现那一趟的差别：基线回比本地常量、取三批；坏实现那一趟不回比（写流与段表本来就会变）、只取批 1 与批 2。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunKind {
    Baseline,
    BadImplementationRow,
}

/// 录入段导出一条流：驱动、取样、写录入段写得了的导出文件（[`write_stream_files_before_the_judge_stage`]）、S4 回读；O2 要判的盘录进 `pool_checker_tables`，不判。
/// 某个状态上实现自崩的流只写两个照写的文件，崩之前判完的那几态留在 `observations` 里交判段（[`judge_the_dump`] 判完清掉）。
fn export_stream(
    name: StreamName,
    output_root: &Path,
    run_kind: RunKind,
    switches: RunSwitches,
    pool_checker_tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> StreamExport {
    let started = Instant::now();
    let is_baseline = run_kind == RunKind::Baseline;
    let stream = prepare_stream(name, switches, is_baseline);
    let drive_nanoseconds = started.elapsed().as_nanos();
    let label = name.directory_name();
    let directory = output_root.join(label);
    std::fs::create_dir_all(&directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
    // 回比只在基线那一趟：坏实现那一趟本来就会让取号、txg、落点、写数与段表变，那些不是停机条款。
    let mut checks = if is_baseline {
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
    let unexported_reason = match &stream.ending {
        StreamEnding::ImplementationCrashed { what } => Some(format!(
            "implementation_crashed_in_a_call:{}",
            what.replace(' ', "_")
        )),
        StreamEnding::Completed | StreamEnding::StoppedByAnError { .. }
            if stream.judged_root_index == usize::MAX =>
        {
            Some("no_root_write_to_judge".to_string())
        }
        StreamEnding::Completed | StreamEnding::StoppedByAnError { .. } => None,
    };
    if let Some(reason) = unexported_reason {
        export.files =
            write_the_files_that_exist_even_when_the_stream_is_not_exported(&directory, &stream);
        export.checks = checks;
        export.not_exported = Some(reason);
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
    let in_place_subsets_states = crash_point_replay_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &in_place_subsets_with_copy_on_write_none_or_all_expansion,
    );
    let facts = extract_flow_facts(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    if is_baseline {
        checks.extend(state_count_checks_against_the_crates(
            name,
            &stream,
            &tables,
            &facts,
            full_states,
            in_place_subsets_states,
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
    if !is_baseline {
        sampled.batches[2].clear();
    }
    let observe_started = Instant::now();
    let observations = match observe_sampled_states(
        &stream,
        &tables,
        &facts,
        &torn_image_index_by_write,
        &sampled,
        switches,
        sampling_threads_of_this_run(switches),
        &|stream_name, ordinal, image_digest, image, pool_checker_control| {
            pool_checker_table_of_a_sampled_state(
                pool_checker_tables,
                stream_name,
                ordinal,
                image_digest,
                image,
                pool_checker_control,
            )
        },
    ) {
        Ok(observations) => observations,
        Err(ObservationsUpToACrashInAState {
            crash,
            observations,
        }) => {
            export.files = write_the_files_that_exist_even_when_the_stream_is_not_exported(
                &directory, &stream,
            );
            export.checks = checks;
            export.not_exported = Some(format!("implementation_crashed_in_a_state:{crash}"));
            // 崩之前判完的那几态录下的盘留给判段判（引擎在其中一态上失败时那一句盖过这一句），判完清掉
            export.observations = observations;
            export.elapsed_nanoseconds = started.elapsed().as_nanos();
            return export;
        }
    };
    let observe_nanoseconds = observe_started.elapsed().as_nanos();
    let write_started = Instant::now();
    let files = write_stream_files_before_the_judge_stage(
        &directory,
        &stream,
        &tables,
        &observations,
        full_states,
        in_place_subsets_states,
    );
    // S4：只读刚写出的文件，按 states.tsv 的落地向量重建，摘要与 states.tsv 里（直接从 CrashImage 算的）那一列比。
    let neutral = read_neutral_stream(&directory);
    let mut rebuilt_states = 0usize;
    let mut rebuilt_digest_mismatches = 0usize;
    for line in read_text_lines(&directory, STATES_FILE) {
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
    export.in_place_subsets_states = in_place_subsets_states;
    export.torn_scans = sampled.torn_scans;
    export.observations = observations;
    export.files = files;
    export.rebuilt_digest_mismatches = rebuilt_digest_mismatches;
    export.rebuilt_states = rebuilt_states;
    export.elapsed_nanoseconds = started.elapsed().as_nanos();
    export
}

/// S1 里跟状态数有关的几条（只在基线那一趟）：全量与甲二状态数、每次写只取两态时的闭式、枚举表与 `crash_facts` 数的总数，
/// 对本地常量。与 `e170_specification_executor_export.rs` 的 `export_stream` 里那一批 S1 回比同一批。
fn state_count_checks_against_the_crates(
    name: StreamName,
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    facts: &FlowFacts,
    full_states: u64,
    in_place_subsets_states: u64,
) -> Vec<StopCheck> {
    let label = name.directory_name();
    let (local_full, local_in_place_subsets, local_closed_form) = match name {
        StreamName::NewPoolFileCreation => (
            LOCAL_FIRST_STREAM_FULL_STATES,
            LOCAL_FIRST_STREAM_IN_PLACE_SUBSETS_STATES,
            LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM,
        ),
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => (
            LOCAL_SECOND_STREAM_FULL_STATES,
            LOCAL_SECOND_STREAM_IN_PLACE_SUBSETS_STATES,
            LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM,
        ),
    };
    vec![
        StopCheck::new(&format!("{label}_full_states"), local_full, full_states),
        StopCheck::new(
            &format!("{label}_in_place_subsets_states"),
            local_in_place_subsets,
            in_place_subsets_states,
        ),
        StopCheck::new(
            &format!("{label}_two_state_closed_form"),
            local_closed_form,
            singlefs_verification::memory_pool::closed_form_state_count_of_write_index_segments(
                &stream.segments,
            ),
        ),
        StopCheck::new(
            &format!("{label}_enumeration_tables_state_count"),
            full_states,
            tables.state_count,
        ),
        StopCheck::new(
            &format!("{label}_flow_facts_state_count"),
            full_states,
            facts.state_count,
        ),
    ]
}

// ───────────────────────────── 结果行、V2 与 main ─────────────────────────────

/// `E7RESULT` 行计数（收尾 `name=done emitted=N`，N 把 done 自己也算进去）；`E7TIMING` 行不计数、不进入库产物。
struct ResultLines<'output> {
    emitted: u64,
    output: &'output mut dyn std::io::Write,
}

impl ResultLines<'_> {
    fn emit_result_line(&mut self, body: &str) {
        self.emitted += 1;
        writeln!(self.output, "E7RESULT {body}").expect("写得了结果行");
    }
    fn timing(body: &str) {
        println!("E7TIMING {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        writeln!(self.output, "E7RESULT name=done emitted={}", self.emitted).expect("写得了结果行");
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
            singlefs_verification::memory_pool::SECTOR_BYTES,
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
    let ninety_fifth_percentile_index = (count * 95).div_ceil(100) - 1;
    (
        total,
        nanoseconds[(count - 1) / 2],
        nanoseconds[ninety_fifth_percentile_index],
        nanoseconds[count - 1],
    )
}

// ───────────────────────────── 坏镜像：镜像物化、补丁与摘要（登记 5.5） ─────────────────────────────

/// 一处字节补丁：在底态镜像上把 (盘, 偏移) 起的这几个字节换掉（与 E170 的 `bad-patches.bin` 同一种补丁）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct BytePatch {
    device: u32,
    offset: u64,
    bytes: Vec<u8>,
}

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
    let entries = known_bad_images(&clean);
    let (_, apply_the_first_entry) = entries.first().expect("已知坏镜像表至少一条");
    let mut bad = clean.clone();
    apply_the_first_entry(&mut bad);
    bad
}

/// 录入段录下取样的一态上 O2 要判的盘（O2 的名单按判到位与违例位能两段，计时与按件 panic 留 CPU）：
/// 这一态的物化镜像（阳性对照那一态是改坏的那一张，[`first_known_bad_image_of`]）作一张给定状态表交 `tables` 录下。
/// 表名带流、序号与镜像摘要（同名的表内容就相同，判过的复用）。
fn pool_checker_table_of_a_sampled_state(
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    stream_name: StreamName,
    ordinal: u64,
    image_digest: &str,
    image: &CrashImage<'_>,
    pool_checker_control: bool,
) -> RecordedTableReference {
    if pool_checker_control {
        tables.record_a_table(
            &format!(
                "sampled-state/{}/ordinal={ordinal}/image={image_digest}/first-known-bad-entry",
                stream_name.directory_name()
            ),
            &first_known_bad_image_of(image),
        )
    } else {
        tables.record_a_table(
            &format!(
                "sampled-state/{}/ordinal={ordinal}/image={image_digest}",
                stream_name.directory_name()
            ),
            &materialized_memory_pool(image),
        )
    }
}

/// 一个 `MemoryPool` 的镜像摘要（E170 登记 5.3 的算法）：每块盘（身份升序）、每个不全为零的扇区（扇区号升序）依次喂进盘身份 u32、扇区号 u64、512 字节。
fn memory_pool_digest(pool: &MemoryPool) -> [u8; DIGEST_BYTES] {
    let mut message = Vec::new();
    for (device, sparse) in &pool.devices {
        for (sector, bytes) in sparse.written_sectors() {
            append_sector_to_digest_message(&mut message, device.0, *sector, bytes);
        }
    }
    sha256_digest(&message)
}

/// 从中性格式按落地向量重建一态的 `MemoryPool`（按流序叠，规则与 [`rebuilt_image_digest`] 相同；两处各算各的摘要，S4 互比）。
fn rebuilt_memory_pool(stream: &NeutralStream, landing_vector: &str) -> MemoryPool {
    let mut pool =
        MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], LOCAL_DEVICE_BYTES);
    for ((device, sector), bytes) in &stream.base_sectors {
        pool.devices
            .get_mut(&DeviceIdentity(*device))
            .expect("基镜像的盘在池里")
            .write(DeviceOffsetInBytes(sector * LOCAL_SECTOR_BYTES), bytes);
    }
    let mut landings = landing_vector.chars();
    let mut write_index = 0usize;
    for operation in &stream.operations {
        if operation.kind == RecordedOperationKind::Barrier {
            continue;
        }
        let landing = landings.next().expect("落地向量与写数一样长");
        let length = usize::try_from(operation.length).expect("写长装得进 usize");
        let device = pool
            .devices
            .get_mut(&DeviceIdentity(operation.device))
            .expect("写的盘在池里");
        match (landing, operation.kind) {
            ('0', _) => {}
            ('2', RecordedOperationKind::WriteZeroes) => {
                device.zero_fill(DeviceOffsetInBytes(operation.offset), operation.length);
            }
            ('2', _) => {
                let start = operation.payload_start.expect("带字节的写有载荷起点");
                device.write(
                    DeviceOffsetInBytes(operation.offset),
                    &stream.payload[start..start + length],
                );
            }
            ('1', _) => {
                let (start, torn_length) = stream.torn_bytes_by_write[&write_index];
                assert_eq!(torn_length, length, "撕裂那一段罩整次写");
                device.write(
                    DeviceOffsetInBytes(operation.offset),
                    &stream.torn_payload[start..start + torn_length],
                );
            }
            (unknown, _) => panic!("落地向量里认不出的字符 {unknown:?}"),
        }
        write_index += 1;
    }
    assert!(landings.next().is_none(), "落地向量比写数长");
    pool
}

/// 把一串补丁施加到 `MemoryPool` 上：补丁的起点与长度不一定按扇区对齐（E170 的 P 行只翻一个字节），
/// 所以先读出罩住它的整扇区、把补丁字节叠上去、再整扇区写回。
fn apply_patches(pool: &mut MemoryPool, patches: &[BytePatch]) {
    let sector_bytes = LOCAL_SECTOR_BYTES;
    for patch in patches {
        let device = pool
            .devices
            .get_mut(&DeviceIdentity(patch.device))
            .expect("补丁的盘在池里");
        let patch_length = u64::try_from(patch.bytes.len()).expect("补丁长装得进 u64");
        let first_sector_start = patch.offset / sector_bytes * sector_bytes;
        let end = (patch.offset + patch_length).div_ceil(sector_bytes) * sector_bytes;
        let mut covered = device.read(
            DeviceOffsetInBytes(first_sector_start),
            usize::try_from(end - first_sector_start).expect("罩住补丁的扇区长装得进 usize"),
        );
        let start_in_covered =
            usize::try_from(patch.offset - first_sector_start).expect("补丁起点在罩住的扇区里");
        covered[start_in_covered..start_in_covered + patch.bytes.len()]
            .copy_from_slice(&patch.bytes);
        device.write(DeviceOffsetInBytes(first_sector_start), &covered);
    }
}

/// 两份镜像逐 512 字节扇区的差（缺的扇区按全零），相邻的差异扇区并成一段补丁；补丁带「改坏之后」的字节。
fn sector_difference_as_patches(clean: &MemoryPool, bad: &MemoryPool) -> Vec<BytePatch> {
    let sector_bytes = usize::try_from(LOCAL_SECTOR_BYTES).expect("512");
    let zero_sector = vec![0u8; sector_bytes];
    let mut patches: Vec<BytePatch> = Vec::new();
    for (device, bad_sparse) in &bad.devices {
        let clean_sparse = clean.devices.get(device).expect("两份镜像的盘相同");
        let sectors: BTreeSet<u64> = bad_sparse
            .written_sectors()
            .keys()
            .chain(clean_sparse.written_sectors().keys())
            .copied()
            .collect();
        let mut open: Option<(u64, BytePatch)> = None;
        for sector in sectors {
            let bad_bytes = bad_sparse
                .written_sectors()
                .get(&sector)
                .unwrap_or(&zero_sector);
            let clean_bytes = clean_sparse
                .written_sectors()
                .get(&sector)
                .unwrap_or(&zero_sector);
            if bad_bytes == clean_bytes {
                continue;
            }
            match &mut open {
                Some((last_sector, patch)) if *last_sector + 1 == sector => {
                    patch.bytes.extend_from_slice(bad_bytes);
                    *last_sector = sector;
                }
                Some(_) | None => {
                    if let Some((_, finished)) = open.take() {
                        patches.push(finished);
                    }
                    open = Some((
                        sector,
                        BytePatch {
                            device: device.0,
                            offset: sector * LOCAL_SECTOR_BYTES,
                            bytes: bad_bytes.clone(),
                        },
                    ));
                }
            }
        }
        if let Some((_, finished)) = open.take() {
            patches.push(finished);
        }
    }
    patches
}

// ───────────────────────────── `known-bad-images` 模式：已知坏镜像与 E170 的 P 行、盲测族上的 O2（登记 5.5） ─────────────────────────────

/// 补丁文本：`设备:偏移:在补丁文件里的起点:长度`，逗号隔开（E170 的 `bad-states.tsv` 同一种写法）；没有补丁写 `-`。
fn patch_list_text(patches: &[BytePatch], patches_file: &mut Vec<u8>) -> String {
    if patches.is_empty() {
        return "-".to_string();
    }
    patches
        .iter()
        .map(|patch| {
            let start = patches_file.len();
            patches_file.extend_from_slice(&patch.bytes);
            format!(
                "{}:{}:{start}:{}",
                patch.device,
                patch.offset,
                patch.bytes.len()
            )
        })
        .collect::<Vec<String>>()
        .join(",")
}

/// 读 E170 的补丁文本回成补丁。
fn patches_of_a_patch_list_text(list: &str, patches_file: &[u8]) -> Vec<BytePatch> {
    if list == "-" {
        return Vec::new();
    }
    list.split(',')
        .map(|item| {
            let fields: Vec<&str> = item.split(':').collect();
            assert_eq!(fields.len(), 4, "补丁四段：{item:?}");
            let start: usize = parse_number(fields[2], "补丁起点");
            let length: usize = parse_number(fields[3], "补丁长度");
            BytePatch {
                device: parse_number(fields[0], "补丁的盘"),
                offset: parse_number(fields[1], "补丁偏移"),
                bytes: patches_file[start..start + length].to_vec(),
            }
        })
        .collect()
}

/// 录入段录下的已知坏镜像表一条：判段写 `known-bad-o2.tsv` 那一行与结果行要的（目标不变量、补丁几处、补丁叠回之后的摘要与直接改坏的相同不相同、
/// 两张要判的盘）。补丁与摘要进 `known-bad-states.tsv`、`known-bad-patches.bin`，录入段就拼好。
struct RecordedKnownBadEntry {
    target: String,
    patch_count: usize,
    digest_equals_the_direct_one: bool,
    direct: RecordedTableReference,
    patched: RecordedTableReference,
}

/// 录入段录完已知坏镜像表一条交回主线程的：交给判段的那一份，与按条目的次序拼 `known-bad-states.tsv`、`known-bad-patches.bin` 要的补丁与摘要。
struct KnownBadEntryOfTheRecordStage {
    hand_off: RecordedKnownBadEntry,
    patches: Vec<BytePatch>,
    patched_digest: String,
}

/// 录入段录已知坏镜像表的一条：施加在干净镜像的拷贝上，直接改坏的那张录下，再取补丁（逐扇区的差）叠回干净镜像、叠完的那张录下。
/// 表的改法（`Box<dyn Fn>`）不跨线程，这条线程照干净镜像现取一份表；干净镜像定了，表的条数、次序与改法就定了。
fn record_a_known_bad_entry(
    clean: &MemoryPool,
    entry_index: usize,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> KnownBadEntryOfTheRecordStage {
    let entries = known_bad_images(clean);
    let (target, apply) = &entries[entry_index];
    let mut bad = clean.clone();
    apply(&mut bad);
    let direct = tables.record_a_table(&format!("known-bad/entry-{entry_index}/direct"), &bad);
    let patches = sector_difference_as_patches(clean, &bad);
    let mut patched = clean.clone();
    apply_patches(&mut patched, &patches);
    let patched_table =
        tables.record_a_table(&format!("known-bad/entry-{entry_index}/patched"), &patched);
    let patched_digest = sha256_text(&memory_pool_digest(&patched));
    let digest_equals_the_direct_one = patched_digest == sha256_text(&memory_pool_digest(&bad));
    KnownBadEntryOfTheRecordStage {
        hand_off: RecordedKnownBadEntry {
            target: (*target).to_string(),
            patch_count: patches.len(),
            digest_equals_the_direct_one,
            direct,
            patched: patched_table,
        },
        patches,
        patched_digest,
    }
}

/// 录入段录下的 E170 坏状态一行：判段写 `e170-bad-rows-o2.tsv` 那一行与结果行要的（流、行号、底态摘要、两个摘要与 E170 写的相同不相同、
/// E170 池级 checker 那一列、底态与叠完补丁的两张要判的盘）。
struct RecordedE170BadRow {
    label: String,
    row_identifier: String,
    base_digest: String,
    digests_equal: bool,
    e170_pool_checker_column: String,
    base: RecordedTableReference,
    patched: RecordedTableReference,
}

/// 录入段录 E170 坏状态的一行：从中性格式重建底态、录下底态，叠补丁、录下叠完的那张，两个摘要与 E170 写的两个摘要比。
fn record_an_e170_bad_row(
    label: &str,
    neutral: &NeutralStream,
    patches_file: &[u8],
    state_row: &str,
    verdict_row: &str,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> RecordedE170BadRow {
    let state_fields: Vec<&str> = state_row.split('\t').collect();
    assert_eq!(
        state_fields.len(),
        5,
        "bad-states.tsv 一行五列：{state_row:?}"
    );
    let verdict_fields: Vec<&str> = verdict_row.split('\t').collect();
    assert_eq!(
        verdict_fields.len(),
        10,
        "bad-impl-verdicts.tsv 一行十列：{verdict_row:?}"
    );
    let row_identifier = state_fields[0];
    let mut pool = rebuilt_memory_pool(neutral, state_fields[1]);
    let base_digest = sha256_text(&memory_pool_digest(&pool));
    // 不打补丁的底态上的 O2（登记 5.6 坏镜像行的阴性：F、Mk、Mk⁺、Mk⁻ 各一张，必须不红）。
    let base = tables.record_a_table(
        &format!("e170-bad-row/{label}/{row_identifier}/base"),
        &pool,
    );
    apply_patches(
        &mut pool,
        &patches_of_a_patch_list_text(state_fields[4], patches_file),
    );
    let patched_digest = sha256_text(&memory_pool_digest(&pool));
    let patched = tables.record_a_table(
        &format!("e170-bad-row/{label}/{row_identifier}/patched"),
        &pool,
    );
    let digests_equal = base_digest == state_fields[2] && patched_digest == state_fields[3];
    RecordedE170BadRow {
        label: label.to_string(),
        row_identifier: row_identifier.to_string(),
        base_digest,
        digests_equal,
        e170_pool_checker_column: verdict_fields[6].to_string(),
        base,
        patched,
    }
}

/// E170 坏状态一行判完的结局：`e170-bad-rows-o2.tsv` 的那一行与结果行，按行的次序在主线程上收。
struct JudgedE170BadRow {
    row_failed: bool,
    pool_checker_file_row: String,
    result_line: String,
}

/// 判段：E170 坏状态一行的底态与叠完补丁的那张判完之后，判红的不变量与 E170 池级 checker 那一列比（S3），拼产物那一行与结果行。
fn judged_e170_bad_row(
    row: &RecordedE170BadRow,
    base_red: &[String],
    red_by_the_pool_checker: &[String],
) -> JudgedE170BadRow {
    let RecordedE170BadRow {
        label,
        row_identifier,
        base_digest,
        digests_equal,
        e170_pool_checker_column,
        ..
    } = row;
    let pool_checker_text_of_the_row = list_text(red_by_the_pool_checker);
    let pool_checker_equals_e170 = pool_checker_text_of_the_row == *e170_pool_checker_column;
    JudgedE170BadRow {
        row_failed: !(*digests_equal && pool_checker_equals_e170),
        pool_checker_file_row: format!(
            "{label}\t{row_identifier}\t{}\t{pool_checker_text_of_the_row}\t{e170_pool_checker_column}\t{pool_checker_equals_e170}\t{digests_equal}\t{}\t{base_digest}\n",
            if red_by_the_pool_checker.is_empty() { "green" } else { "red" },
            list_text(base_red)
        ),
        result_line: format!(
            "name=e170_bad_row stream={label} row={row_identifier} o2={} o2_red={} e170_pool_checker={} o2_equals_e170={pool_checker_equals_e170} digests_equal={digests_equal} base_o2_red={}",
            if red_by_the_pool_checker.is_empty() { "green" } else { "red" },
            field_text(&pool_checker_text_of_the_row),
            field_text(e170_pool_checker_column),
            field_text(&list_text(base_red))
        ),
    }
}

/// `known-bad-images` 模式录入段构造要判的盘的三步，按两段之前打结果行的次序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KnownBadImagesStep {
    /// 从导出目录重建干净镜像、两条 S4 回比、数表的条数、录下干净镜像那张盘。
    CleanImage,
    /// 已知坏镜像表逐条构造、录下。
    KnownBadEntries,
    /// E170 坏状态逐流、逐行构造、录下（只在带 `--e170-bad-rows` 时）。
    E170BadRows,
}

/// 录入段构造要判的盘时 panic 了：在哪一步、panic 的消息。判段照两段之前的次序打到那一步之前，再以同一句 panic。
struct PanicOfTheRecordStage {
    step: KnownBadImagesStep,
    message: String,
}

/// 录下了的干净镜像：已知坏镜像表的条数与干净镜像那张盘。
struct RecordedCleanImage {
    entry_count: usize,
    table: RecordedTableReference,
}

/// 录入段交给判段的 `known-bad-images` 那一趟：已知坏镜像表在流一全持久态上（干净镜像的两条 S4 回比、表的条数与干净镜像那张盘、每一条录下的，
/// 与按条目的次序拼好的 `known-bad-states.tsv`、`known-bad-patches.bin`），与带 `--e170-bad-rows` 时 E170 坏状态两条流按行的次序录下的每一行（不带是空的）。
/// 构造时 panic 了的，各样录到 panic 之前为止（`clean` 没录下是 `None`），`panic_of_the_record_stage` 记着在哪一步、那一句。
struct RecordedKnownBadImages {
    clean_image_checks: Vec<StopCheck>,
    clean: Option<RecordedCleanImage>,
    entries: Vec<RecordedKnownBadEntry>,
    states_text: String,
    patches_file: Vec<u8>,
    e170_bad_rows: Vec<RecordedE170BadRow>,
    panic_of_the_record_stage: Option<PanicOfTheRecordStage>,
}

/// 录入段在 `step` 这一步 panic 过（`panic_of_the_record_stage`）：判段以同一句 panic（与两段之前构造到那里 panic 时一样，之后的结果行与产物都没有）。
fn panic_again_if_the_record_stage_panicked_at(
    panic_of_the_record_stage: Option<&PanicOfTheRecordStage>,
    step: KnownBadImagesStep,
) {
    if let Some(panic) =
        panic_of_the_record_stage.filter(|recorded_panic| recorded_panic.step == step)
    {
        std::panic::panic_any(panic.message.clone());
    }
}

/// `known-bad-images` 模式的录入段：从导出目录重建流一全持久态（干净镜像），回比两个摘要（S4），已知坏镜像表每一条（一条怎么录见
/// [`record_a_known_bad_entry`]）与带 `--e170-bad-rows` 时 E170 坏状态每一行（[`record_an_e170_bad_row`]）要判的盘录进 `tables`，不判。
/// 条目与行分给 [`sampling_threads_of_this_run`] 条线程录（每条各录各的表，表名、内容与一条线程时相同，判定库的键不变）；补丁文件的起点与
/// 录下的次序都按条目、按行的次序在主线程上收，与线程数无关。构造时 panic 了（次序上第一个 panic 的条目或行）：接住，录到它之前为止，
/// 记下在哪一步、那一句（[`PanicOfTheRecordStage`]），交给判段照两段之前的次序打到那里再 panic。
fn record_the_known_bad_images(
    command_line: &KnownBadImagesCommandLine,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> RecordedKnownBadImages {
    let mut recorded = RecordedKnownBadImages {
        clean_image_checks: Vec::new(),
        clean: None,
        entries: Vec::new(),
        states_text: String::new(),
        patches_file: Vec::new(),
        e170_bad_rows: Vec::new(),
        panic_of_the_record_stage: None,
    };
    let mut step = KnownBadImagesStep::CleanImage;
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        record_the_known_bad_images_step_by_step(command_line, tables, &mut recorded, &mut step);
    }));
    if let Err(payload) = caught {
        recorded.panic_of_the_record_stage = Some(PanicOfTheRecordStage {
            step,
            message: panic_text(&*payload),
        });
    }
    recorded
}

/// [`record_the_known_bad_images`] 的三步，录下的随录随放进 `recorded`，走到哪一步记在 `step`（panic 时那两样留到 panic 之前）。
fn record_the_known_bad_images_step_by_step(
    command_line: &KnownBadImagesCommandLine,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    recorded: &mut RecordedKnownBadImages,
    step: &mut KnownBadImagesStep,
) {
    let threads = sampling_threads_of_this_run(command_line.switches);
    let stream_directory = command_line
        .dump_directory
        .join(StreamName::NewPoolFileCreation.directory_name());
    let neutral = read_neutral_stream(&stream_directory);
    let write_count = neutral
        .operations
        .iter()
        .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
        .count();
    let all_persisted = "2".repeat(write_count);
    let clean = rebuilt_memory_pool(&neutral, &all_persisted);
    let clean_digest = sha256_text(&memory_pool_digest(&clean));
    let reference_digest = sha256_text(&memory_pool_digest(
        &common::build_pool("e171-known-bad").memory_pool(),
    ));
    let states_digest_of_the_full_state = read_text_lines(&stream_directory, STATES_FILE)
        .iter()
        .map(|line| {
            line.split('\t')
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
        .find(|fields| fields[4] == all_persisted)
        .map(|fields| fields[5].clone());
    recorded.clean_image_checks = vec![
        StopCheck::new(
            "known_bad_clean_image_digest_equals_build_pool",
            &reference_digest,
            &clean_digest,
        ),
        StopCheck::new(
            "known_bad_clean_image_digest_equals_the_full_state_of_the_dump",
            Some(&clean_digest),
            states_digest_of_the_full_state.as_ref(),
        ),
    ];
    let entry_count = known_bad_images(&clean).len();
    recorded.clean = Some(RecordedCleanImage {
        entry_count,
        table: tables.record_a_table("known-bad/clean", &clean),
    });
    *step = KnownBadImagesStep::KnownBadEntries;
    eprintln!("E171_KNOWN_BAD_IMAGES entries={entry_count} threads={threads}");
    for (entry_index, recorded_entry) in
        outcomes_up_to_the_earliest_panic(entry_count, threads, &|entry_index| {
            record_a_known_bad_entry(&clean, entry_index, tables)
        })
        .into_iter()
        .enumerate()
    {
        let KnownBadEntryOfTheRecordStage {
            hand_off,
            patches,
            patched_digest,
        } = match recorded_entry {
            Ok(entry_of_the_record_stage) => entry_of_the_record_stage,
            Err(payload) => std::panic::resume_unwind(payload),
        };
        recorded.states_text.push_str(&format!(
            "{entry_index}\t{all_persisted}\t{clean_digest}\t{patched_digest}\t{}\n",
            patch_list_text(&patches, &mut recorded.patches_file)
        ));
        recorded.entries.push(hand_off);
    }
    *step = KnownBadImagesStep::E170BadRows;
    if command_line.with_e170_bad_rows {
        record_the_e170_bad_rows_of_both_streams(
            &command_line.dump_directory,
            threads,
            tables,
            &mut recorded.e170_bad_rows,
        );
    }
}

/// E170 的 `bad-states` 模式造的坏状态（P 行、N 行、盲测族）：每一行读 `bad-states.tsv` 的落地向量与补丁、`bad-impl-verdicts.tsv` 里池级 checker
/// 那一列，录下要判的两张盘（一行怎么录见 [`record_an_e170_bad_row`]），按流、按行的次序放进 `rows`，与线程数无关。一条流的行分给 `threads` 条线程录；
/// 次序上第一个 panic 的行原样抛出，它之前的行都放进去了。只读 `bad-states.tsv`、`bad-patches.bin`、`bad-impl-verdicts.tsv`，不读期望文件。
fn record_the_e170_bad_rows_of_both_streams(
    dump_directory: &Path,
    threads: NonZeroUsize,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    rows: &mut Vec<RecordedE170BadRow>,
) {
    for name in STREAM_NAMES {
        let label = name.directory_name();
        let stream_directory = dump_directory.join(label);
        let neutral = read_neutral_stream(&stream_directory);
        let patches_file = read_file(&stream_directory, "bad-patches.bin");
        let state_rows = read_text_lines(&stream_directory, "bad-states.tsv");
        let verdict_rows = read_text_lines(&stream_directory, "bad-impl-verdicts.tsv");
        assert_eq!(
            state_rows.len(),
            verdict_rows.len(),
            "bad-states.tsv 与 bad-impl-verdicts.tsv 同行数"
        );
        eprintln!(
            "E171_E170_BAD_ROWS stream={label} rows={} threads={threads}",
            state_rows.len()
        );
        for recorded in
            outcomes_up_to_the_earliest_panic(state_rows.len(), threads, &|row_position| {
                record_an_e170_bad_row(
                    label,
                    &neutral,
                    &patches_file,
                    &state_rows[row_position],
                    &verdict_rows[row_position],
                    tables,
                )
            })
        {
            match recorded {
                Ok(row) => rows.push(row),
                Err(payload) => std::panic::resume_unwind(payload),
            }
        }
    }
}

/// `known-bad-images` 模式的判段：录下的盘按录下的次序（干净镜像、每一条的直接改坏与叠完补丁、每一行 E170 坏状态的底态与叠完补丁）
/// 分给 [`sampling_threads_of_this_run`] 条线程判，结果行与产物照一趟写完时的次序拼：两边判红的不变量集合要相等（S4）、目标不变量要在里面（S4）；
/// E170 坏状态每一行判红的不变量与 E170 池级 checker 那一列比（S3）。判器在次序上第一个 panic 的那张盘照原样抛出，抛出之前的结果行与产物照写
/// （与一条线程判到那里 panic 时相同）。录入段构造时 panic 过的，打到那一步之前，再以同一句 panic（[`panic_again_if_the_record_stage_panicked_at`]）：
/// 录下的盘都在 panic 之前，判器在其中一张上 panic 的先抛。结果行写进 `result_lines_output`，产物写进命令行给的输出目录。交回退出码。
///
/// # Errors
/// 交接与录下的对不上（[`PoolCheckerTablesOfTheTwoStages::judge_a_recorded_table`] 的 `Err`）：一个文件都不写、一行结果行都不打就交回那一句。
fn judge_the_known_bad_images(
    command_line: &KnownBadImagesCommandLine,
    recorded: RecordedKnownBadImages,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    result_lines_output: &mut dyn std::io::Write,
) -> Result<i32, String> {
    let RecordedKnownBadImages {
        clean_image_checks,
        clean,
        entries,
        states_text,
        patches_file,
        e170_bad_rows,
        panic_of_the_record_stage,
    } = recorded;
    let recorded_tables: Vec<&RecordedTableReference> = clean
        .iter()
        .map(|recorded_clean_image| &recorded_clean_image.table)
        .chain(
            entries
                .iter()
                .flat_map(|entry| [&entry.direct, &entry.patched]),
        )
        .chain(
            e170_bad_rows
                .iter()
                .flat_map(|row| [&row.base, &row.patched]),
        )
        .collect();
    let mut judged_in_the_recorded_order = Vec::with_capacity(recorded_tables.len());
    for judged in outcomes_up_to_the_earliest_panic(
        recorded_tables.len(),
        sampling_threads_of_this_run(command_line.switches),
        &|position| tables.judge_a_recorded_table(recorded_tables[position]),
    ) {
        judged_in_the_recorded_order.push(match judged {
            Ok(red_or_refusal) => Ok(red_or_refusal?),
            Err(payload) => Err(payload),
        });
    }
    let mut judged_in_the_recorded_order = judged_in_the_recorded_order.into_iter();
    let mut lines = ResultLines {
        emitted: 0,
        output: result_lines_output,
    };
    lines.emit_result_line(&format!(
        "name=config segment=s1 mode=known-bad-images e170_bad_rows={}",
        command_line.with_e170_bad_rows
    ));
    let mut known_bad_failed = 0usize;
    for check in &clean_image_checks {
        known_bad_failed += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s4 check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let Some(RecordedCleanImage { entry_count, .. }) = clean else {
        let panic = panic_of_the_record_stage
            .expect("录入段没录下干净镜像，只可能是构造干净镜像那一步 panic 了");
        std::panic::panic_any(panic.message);
    };
    lines.emit_result_line(&format!("name=known_bad_entry_count count={entry_count}"));
    let mut red_names_of_the_next_table = || match judged_in_the_recorded_order
        .next()
        .expect("录下的每张盘都判过一次：判的次序就是录下的次序，panic 的那张之后不再往下走")
    {
        Ok(red) => red,
        Err(payload) => std::panic::resume_unwind(payload),
    };
    let clean_red = red_names_of_the_next_table();
    lines.emit_result_line(&format!(
        "name=known_bad_clean_image invariants_red={}",
        field_text(&list_text(&clean_red))
    ));
    let mut pool_checker_text = String::new();
    for (entry_index, entry) in entries.iter().enumerate() {
        let direct_red = red_names_of_the_next_table();
        let patched_red = red_names_of_the_next_table();
        let target = &entry.target;
        let digest_equals_the_direct_one = entry.digest_equals_the_direct_one;
        let sets_equal = direct_red == patched_red;
        let target_is_red = patched_red.iter().any(|name| name == target);
        known_bad_failed +=
            usize::from(!(sets_equal && digest_equals_the_direct_one && target_is_red));
        pool_checker_text.push_str(&format!(
            "{entry_index}\t{target}\t{}\t{}\t{sets_equal}\t{target_is_red}\n",
            list_text(&patched_red),
            list_text(&direct_red)
        ));
        lines.emit_result_line(&format!(
            "name=known_bad_entry index={entry_index} target={target} o2={} patched_red={} direct_red={} sets_equal={sets_equal} digest_equals_the_direct_one={digest_equals_the_direct_one} target_is_red={target_is_red} patches={}",
            if patched_red.is_empty() { "green" } else { "red" },
            field_text(&list_text(&patched_red)),
            field_text(&list_text(&direct_red)),
            entry.patch_count
        ));
    }
    panic_again_if_the_record_stage_panicked_at(
        panic_of_the_record_stage.as_ref(),
        KnownBadImagesStep::KnownBadEntries,
    );
    std::fs::create_dir_all(&command_line.output_directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", command_line.output_directory.display()));
    for (file_name, contents) in [
        ("known-bad-states.tsv", states_text.into_bytes()),
        ("known-bad-patches.bin", patches_file),
        ("known-bad-o2.tsv", pool_checker_text.into_bytes()),
    ] {
        let written = write_export_file(&command_line.output_directory, file_name, &contents);
        lines.emit_result_line(&format!(
            "name=file file={file_name} bytes={} sha256={}",
            written.bytes, written.sha256
        ));
    }
    let e170_rows_failed = if command_line.with_e170_bad_rows {
        let mut failed = 0usize;
        let mut e170_pool_checker_text = String::new();
        for row in &e170_bad_rows {
            let base_red = red_names_of_the_next_table();
            let red_by_the_pool_checker = red_names_of_the_next_table();
            let judged = judged_e170_bad_row(row, &base_red, &red_by_the_pool_checker);
            failed += usize::from(judged.row_failed);
            e170_pool_checker_text.push_str(&judged.pool_checker_file_row);
            lines.emit_result_line(&judged.result_line);
        }
        panic_again_if_the_record_stage_panicked_at(
            panic_of_the_record_stage.as_ref(),
            KnownBadImagesStep::E170BadRows,
        );
        let written = write_export_file(
            &command_line.output_directory,
            "e170-bad-rows-o2.tsv",
            e170_pool_checker_text.as_bytes(),
        );
        lines.emit_result_line(&format!(
            "name=file file=e170-bad-rows-o2.tsv bytes={} sha256={}",
            written.bytes, written.sha256
        ));
        failed
    } else {
        0
    };
    lines.emit_result_line(&format!(
        "name=verdict known_bad_entries_failing_s4={known_bad_failed} e170_bad_rows_failing_s3_or_s4={e170_rows_failed}"
    ));
    lines.finish();
    if known_bad_failed > 0 || e170_rows_failed > 0 {
        Ok(3)
    } else {
        Ok(0)
    }
}

/// 一条流的结果行（`E7RESULT` 与 `E7TIMING`）；交回 (S1 没过的条数, S4 对不上的态数, 掩码对不上的态数, S3 对不上的态数,
/// O1-对拍判红的调用数, O1-崩溃判官判红的态数, O2 判红的态数)。
fn report_stream(lines: &mut ResultLines<'_>, export: &StreamExport) -> [usize; 7] {
    let label = export.name.directory_name();
    let mut failed_checks = 0usize;
    for check in &export.checks {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream={label} check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let ending_text = match &export.ending {
        Some(StreamEnding::Completed) => "completed".to_string(),
        Some(StreamEnding::StoppedByAnError { call_index, member }) => {
            format!(
                "stopped_by_an_error:call_{call_index}:{}",
                field_text(member)
            )
        }
        Some(StreamEnding::ImplementationCrashed { what }) => {
            format!("implementation_crashed:{}", field_text(what))
        }
        None => "pipeline_panicked".to_string(),
    };
    lines.emit_result_line(&format!(
        "name=stream stream={label} ending={ending_text} exported={} not_exported_why={} operations={} writes={} segments={} tearable_writes={} calls={} full_states={} in_place_subsets_states={} sampled_states={} model_stopped_at_call={}",
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
        export.in_place_subsets_states,
        export.observations.len(),
        export
            .model_stopped_at_call
            .map_or_else(|| "none".to_string(), |call| call.to_string())
    ));
    let mut red_calls = 0usize;
    for (call_index, call) in export.calls.iter().enumerate() {
        red_calls += usize::from(matches!(call.model, ModelVerdictOfCall::Disagrees { .. }));
        lines.emit_result_line(&format!(
            "name=call stream={label} call={call_index} label={} first_operation={} return_point={} status={} o1_comparison={}",
            call.label,
            call.first_operation,
            call.return_point,
            field_text(&call.outcome.outcome_word()),
            field_text(&call.model.word()),
        ));
    }
    for scan in &export.torn_scans {
        lines.emit_result_line(&format!(
            "name=torn_scan stream={label} segment={} scanned={} found={} wanted={TORN_STATES_PER_SEGMENT}",
            scan.segment_index, scan.scanned, scan.found
        ));
    }
    let mut crash_judge_red = 0usize;
    let mut pool_checker_red = 0usize;
    for batch in 1..=3u8 {
        let in_batch: Vec<&StateObservation> = export
            .observations
            .iter()
            .filter(|observation| observation.batch == batch)
            .collect();
        let count_where = |predicate: &dyn Fn(&StateObservation) -> bool| {
            in_batch
                .iter()
                .filter(|observation| predicate(observation))
                .count()
        };
        let crash_judge_is =
            |word: &'static str| count_where(&|observation| observation.crash_judge.word() == word);
        let red_by_the_pool_checker = count_where(&|observation| {
            observation
                .pool_checker_red()
                .is_some_and(|items| !items.is_empty())
        });
        crash_judge_red += crash_judge_is("red");
        pool_checker_red += red_by_the_pool_checker;
        let segments: BTreeSet<usize> = in_batch
            .iter()
            .map(|observation| observation.segment_index)
            .collect();
        lines.emit_result_line(&format!(
            "name=batch stream={label} batch={batch} states={} segments_touched={} states_with_a_torn_write={} o1_crash_judge_red={} o1_crash_judge_green={} o1_crash_judge_not_applicable={} o1_crash_judge_off={} o2_red={red_by_the_pool_checker} o2_matches_judgement_false={} judge_crash_image_pool_checker_red={} record_checker_red_states={} consulted_oracle_red_states={} ignored_oracle_red_states={} consulted_effective_root_none={} consulted_failed_outcomes={}",
            in_batch.len(),
            segments.len(),
            count_where(&|observation| observation.landing_vector.contains('1')),
            crash_judge_is("red"),
            crash_judge_is("green"),
            crash_judge_is("na"),
            crash_judge_is("off"),
            count_where(&|observation| !observation.pool_checker_matches_the_judgement()),
            count_where(&|observation| !observation.verdict.pool_checker_red.is_empty()),
            count_where(&|observation| observation
                .verdict
                .red_items
                .iter()
                .any(|item| item.starts_with("record_checker:"))),
            count_where(&|observation| observation
                .verdict
                .red_items
                .iter()
                .any(|item| item.starts_with("journal_consulted_oracle:"))),
            count_where(&|observation| observation
                .verdict
                .red_items
                .iter()
                .any(|item| item.starts_with("journal_ignored_oracle:"))),
            count_where(&|observation| observation.verdict.consulted_root == "none"),
            count_where(&|observation| observation.verdict.consulted_outcome.starts_with("failed:")),
        ));
        for (stage, pick) in [
            (
                "o1",
                (|observation: &StateObservation| observation.crash_judge_nanoseconds)
                    as fn(&StateObservation) -> u128,
            ),
            ("o2", |observation| observation.pool_checker_nanoseconds),
            ("recovery", |observation| observation.recovery_nanoseconds),
        ] {
            let (total, median, ninety_fifth_percentile, maximum) = timing_summary(
                in_batch
                    .iter()
                    .map(|observation| pick(observation))
                    .collect(),
            );
            ResultLines::timing(&format!(
                "name=stage_time stream={label} batch={batch} stage={stage} states={} total_ns={total} median_ns={median} p95_ns={ninety_fifth_percentile} max_ns={maximum}",
                in_batch.len()
            ));
        }
    }
    let mask_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.mask_matches_the_facts)
        .count();
    let pool_checker_direct_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.pool_checker_matches_the_judgement())
        .count();
    lines.emit_result_line(&format!(
        "name=s4 stream={label} rebuilt_states={} digest_mismatches={}",
        export.rebuilt_states, export.rebuilt_digest_mismatches
    ));
    lines.emit_result_line(&format!(
        "name=crash_facts_mask stream={label} states={} mismatches={mask_mismatches}",
        export.observations.len()
    ));
    lines.emit_result_line(&format!(
        "name=s3 stream={label} states={} o2_direct_differs_from_judge_crash_image={pool_checker_direct_mismatches}",
        export.observations.len()
    ));
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
    [
        failed_checks,
        export.rebuilt_digest_mismatches,
        mask_mismatches,
        pool_checker_direct_mismatches,
        red_calls,
        crash_judge_red,
        pool_checker_red,
    ]
}

/// 基线上 O1 不适用的来源只有两种（登记第十一节 S5）：模型没有答的 `unmount`，与对拍判出不同之后。基线上不许有对拍判出不同，
/// 所以模型只许停在流二的 `unmount` 那一次调用（流一一次都不停）。这两条按 `StopCheck` 的形式回比：期望（本地）与实际（这一趟）。
fn not_applicable_source_checks_of_the_baseline(export: &StreamExport) -> Vec<StopCheck> {
    let label = export.name.directory_name();
    let expected_stop = match export.name {
        StreamName::NewPoolFileCreation => None,
        StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => export
            .calls
            .iter()
            .rposition(|call| call.label == UNMOUNT_CALL_LABEL),
    };
    vec![
        StopCheck::new(
            &format!("{label}_o1_model_stops_only_at_the_unmount"),
            expected_stop,
            export.model_stopped_at_call,
        ),
        StopCheck::new(
            &format!("{label}_every_call_is_judged_or_is_the_unmount"),
            true,
            export.calls.iter().all(|call| {
                matches!(
                    call.model,
                    ModelVerdictOfCall::Agrees | ModelVerdictOfCall::NotModelled
                )
            }),
        ),
    ]
}

/// 逐行那一趟（坏实现、阳性对照；不关 oracle）的 S5：与基线那一趟（[`not_applicable_source_checks_of_the_baseline`]）同一条停机条款（登记第十一节 S5：
/// 「不适用只许来自 `unmount` 与对拍判出不同之后」）。坏实现可以让对拍判出不同、让模型停在那里（那是 O1-对拍的红），所以这一趟不要求模型
/// 只停在 `unmount`；要求两样（登记第一节读法 5、6），都按「不合的个数」回比 0：
/// ① 返回了错误的调用（卸载除外：模型没有卸载的答）没有一次留在「没问模型」（`not_modelled`）——取号、暖机出错那一形；
/// 模型因对拍判出不同停下之后的调用（`skipped_after_the_model_stopped`）不数：读法 6 原文「一次对拍判出不同之后模型不再往前走」，
/// 那是合规来源（登记修订 4）；② O1-崩溃判官的不适用只来自模型停在卸载、或停在对拍判出不同的那次调用。
fn not_applicable_source_checks_of_a_bad_implementation_row(
    label: &str,
    calls: &[RecordedCall],
    model_stopped_at_call: Option<usize>,
    not_applicable_states: usize,
) -> Vec<StopCheck> {
    let errored_calls_the_comparison_did_not_judge = calls
        .iter()
        .filter(|call| call.label != UNMOUNT_CALL_LABEL)
        .filter(|call| matches!(call.outcome, CallOutcome::ReturnedError(_)))
        .filter(|call| matches!(call.model, ModelVerdictOfCall::NotModelled))
        .count();
    let model_stopped_at_the_unmount_or_at_a_disagreement =
        model_stopped_at_call.is_none_or(|stopped| {
            calls.get(stopped).is_some_and(|call| {
                call.label == UNMOUNT_CALL_LABEL
                    || matches!(call.model, ModelVerdictOfCall::Disagrees { .. })
            })
        });
    let not_applicable_states_with_another_source =
        if model_stopped_at_the_unmount_or_at_a_disagreement {
            0
        } else {
            not_applicable_states
        };
    vec![
        StopCheck::new(
            &format!("{label}_o1_every_errored_call_but_the_unmount_is_judged_by_the_comparison"),
            0usize,
            errored_calls_the_comparison_did_not_judge,
        ),
        StopCheck::new(
            &format!("{label}_o1_not_applicable_only_after_the_unmount_or_a_disagreement"),
            0usize,
            not_applicable_states_with_another_source,
        ),
    ]
}

/// 这一趟、这一条流要判的 S5（登记第十一节 S5）。基线那一趟判 [`not_applicable_source_checks_of_the_baseline`]：只判不带对照、不关 oracle、流导出了的基线
/// （阳性对照会让模型在翻过的那次调用停下，关掉 O1 时每次调用都不比）。逐行那一趟判 [`not_applicable_source_checks_of_a_bad_implementation_row`]：阳性对照照判
/// （它让模型停在翻过的那次调用，是对拍的红）；关掉 O1 不判；驱动没走完整（实现 panic、流水线 panic）的流走自崩那一类，不判；
/// 状态上实现 panic 的流驱动是走完了的，照判（那时没有观测，不适用的态数是 0）。
fn not_applicable_source_checks_to_judge(
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
                not_applicable_source_checks_of_the_baseline(export)
            } else {
                Vec::new()
            }
        }
        RunKind::BadImplementationRow => {
            let drive_ran_to_its_end = matches!(
                export.ending,
                Some(StreamEnding::Completed | StreamEnding::StoppedByAnError { .. })
            );
            if drive_ran_to_its_end && switches.disabled_oracle.is_none() {
                let not_applicable_states = export
                    .observations
                    .iter()
                    .filter(|observation| {
                        observation.crash_judge == CrashJudgeVerdict::NotApplicable
                    })
                    .count();
                not_applicable_source_checks_of_a_bad_implementation_row(
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

struct CommandLine {
    run_kind: RunKind,
    output_directory: PathBuf,
    switches: RunSwitches,
}

struct KnownBadImagesCommandLine {
    dump_directory: PathBuf,
    output_directory: PathBuf,
    with_e170_bad_rows: bool,
    switches: RunSwitches,
}

enum ParsedCommand {
    Dump(CommandLine),
    KnownBadImages(KnownBadImagesCommandLine),
}

fn parse_command_line(arguments: &[String]) -> Option<ParsedCommand> {
    let mut words = arguments.iter();
    let mode = words.next()?.as_str();
    let mut output_directory = None;
    let mut dump_directory = None;
    let mut with_e170_bad_rows = false;
    let mut switches = RunSwitches::default();
    while let Some(word) = words.next() {
        match word.as_str() {
            "--output-directory" => output_directory = Some(PathBuf::from(words.next()?)),
            "--dump-directory" => dump_directory = Some(PathBuf::from(words.next()?)),
            "--e170-bad-rows" => with_e170_bad_rows = true,
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
            "--sampling-threads" => {
                switches.sampling_threads = Some(words.next()?.parse::<NonZeroUsize>().ok()?);
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
        })),
        "known-bad-images" => Some(ParsedCommand::KnownBadImages(KnownBadImagesCommandLine {
            dump_directory: dump_directory?,
            output_directory: output_directory?,
            with_e170_bad_rows,
            switches,
        })),
        _ => None,
    }
}

// ───────────────────────────── 两段的交接与选段 ─────────────────────────────

/// 点段的参数（取值照 `crash_library::RunStage` 的写法：`record` 只跑录入段、`judge` 只跑判段，不给就两段先后跑）。
const STAGE_ARGUMENT: &str = "--stage";

/// 只跑录入段那一趟留给判段的交接，放在输出目录里：判段读完、写完产物之后删掉；两段先后跑的那一趟在内存里交、不落这个文件。
/// 里面没有判定（O2 录下的只是盘的摘要，盘在判定库里）：是录入段的观测与结果行要的东西。
const HAND_OFF_TO_THE_JUDGE_STAGE_FILE: &str = "hand-off-to-the-judge-stage.bin";

/// 交接字节的头：换了布局就换头，判段只认这一版。
const HAND_OFF_MAGIC: &[u8; 8] = b"E171HO01";

/// 只跑录入段那一趟录完的退出码（停机条款归判段）。
const EXIT_CODE_OF_A_RECORD_ONLY_RUN: i32 = 0;

/// 交接里的一样东西怎么写成字节、怎么读回：整数小端定长，串与列表先写长度，枚举先写一个字节的成员号。
trait HandOffField: Sized {
    fn put_into(&self, bytes: &mut Vec<u8>);
    /// 字节不够、成员号认不得、串不是 UTF-8 交回 `None`。
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self>;
}

/// 从交接字节里按次序往下读。
struct HandOffReader<'bytes> {
    bytes: &'bytes [u8],
    position: usize,
}

impl<'bytes> HandOffReader<'bytes> {
    fn take_bytes(&mut self, count: usize) -> Option<&'bytes [u8]> {
        let end = self.position.checked_add(count)?;
        let taken = self.bytes.get(self.position..end)?;
        self.position = end;
        Some(taken)
    }

    fn is_at_the_end(&self) -> bool {
        self.position == self.bytes.len()
    }
}

impl HandOffField for u8 {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self);
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        reader.take_bytes(1).map(|taken| taken[0])
    }
}

impl HandOffField for bool {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.push(u8::from(*self));
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(false),
            1 => Some(true),
            2..=u8::MAX => None,
        }
    }
}

impl HandOffField for u64 {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.to_le_bytes());
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        Some(u64::from_le_bytes(reader.take_bytes(8)?.try_into().ok()?))
    }
}

impl HandOffField for u128 {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.to_le_bytes());
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        Some(u128::from_le_bytes(reader.take_bytes(16)?.try_into().ok()?))
    }
}

impl HandOffField for usize {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        u64::try_from(*self)
            .expect("usize 装得进 u64")
            .put_into(bytes);
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        usize::try_from(u64::take_from(reader)?).ok()
    }
}

impl HandOffField for String {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        self.len().put_into(bytes);
        bytes.extend_from_slice(self.as_bytes());
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        let length = usize::take_from(reader)?;
        String::from_utf8(reader.take_bytes(length)?.to_vec()).ok()
    }
}

impl HandOffField for [u8; DIGEST_BYTES] {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(self);
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        reader.take_bytes(DIGEST_BYTES)?.try_into().ok()
    }
}

impl<Element: HandOffField> HandOffField for Option<Element> {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            None => bytes.push(0),
            Some(value) => {
                bytes.push(1);
                value.put_into(bytes);
            }
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(None),
            1 => Element::take_from(reader).map(Some),
            2..=u8::MAX => None,
        }
    }
}

impl<Element: HandOffField> HandOffField for Vec<Element> {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        self.len().put_into(bytes);
        for element in self {
            element.put_into(bytes);
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        let count = usize::take_from(reader)?;
        (0..count)
            .map(|_element_position| Element::take_from(reader))
            .collect()
    }
}

/// 结构体的交接：字段按写出的次序逐个写、逐个读（构造时每个字段都要写出来，漏了一个编不过）。
macro_rules! hand_off_fields_of_a_struct {
    ($structure:ident { $($field:ident),+ $(,)? }) => {
        impl HandOffField for $structure {
            fn put_into(&self, bytes: &mut Vec<u8>) {
                $( self.$field.put_into(bytes); )+
            }
            fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
                Some(Self {
                    $( $field: HandOffField::take_from(reader)?, )+
                })
            }
        }
    };
}

hand_off_fields_of_a_struct!(StopCheck {
    name,
    local,
    crates
});
hand_off_fields_of_a_struct!(RecordedCall {
    label,
    first_operation,
    return_point,
    outcome,
    model
});
hand_off_fields_of_a_struct!(TornScan {
    segment_index,
    scanned,
    found
});
hand_off_fields_of_a_struct!(ImplementationVerdict {
    consulted_root,
    consulted_outcome,
    ignored_root,
    ignored_outcome,
    red_items,
    pool_checker_red,
    root_without_record,
    claimed_state_missing_unit
});
hand_off_fields_of_a_struct!(RecoverySummary {
    root,
    outcome,
    read_back_sha256,
    records_applied
});
hand_off_fields_of_a_struct!(StateObservation {
    batch,
    ordinal,
    segment_index,
    landing_vector,
    image_digest,
    verdict,
    judge_nanoseconds,
    mask_matches_the_facts,
    touched_call,
    crash_judge,
    pool_checker,
    consulted,
    ignored,
    crash_judge_nanoseconds,
    pool_checker_nanoseconds,
    recovery_nanoseconds
});
hand_off_fields_of_a_struct!(WrittenFile {
    name,
    bytes,
    sha256,
    sha256_without_timing
});
hand_off_fields_of_a_struct!(StreamExport {
    name,
    checks,
    calls,
    ending,
    model_stopped_at_call,
    not_exported,
    operation_count,
    write_count,
    segment_count,
    tearable_write_count,
    full_states,
    in_place_subsets_states,
    torn_scans,
    observations,
    files,
    rebuilt_digest_mismatches,
    rebuilt_states,
    elapsed_nanoseconds,
    drive_nanoseconds,
    observe_nanoseconds,
    write_nanoseconds
});
hand_off_fields_of_a_struct!(RecordedTableReference { table_name, digest });
hand_off_fields_of_a_struct!(RecordedDump {
    stream_independent_checks,
    exports
});
hand_off_fields_of_a_struct!(RecordedKnownBadEntry {
    target,
    patch_count,
    digest_equals_the_direct_one,
    direct,
    patched
});
hand_off_fields_of_a_struct!(RecordedE170BadRow {
    label,
    row_identifier,
    base_digest,
    digests_equal,
    e170_pool_checker_column,
    base,
    patched
});
hand_off_fields_of_a_struct!(RecordedCleanImage { entry_count, table });
hand_off_fields_of_a_struct!(PanicOfTheRecordStage { step, message });
hand_off_fields_of_a_struct!(RecordedKnownBadImages {
    clean_image_checks,
    clean,
    entries,
    states_text,
    patches_file,
    e170_bad_rows,
    panic_of_the_record_stage
});

impl HandOffField for KnownBadImagesStep {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.push(match self {
            Self::CleanImage => 0,
            Self::KnownBadEntries => 1,
            Self::E170BadRows => 2,
        });
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::CleanImage),
            1 => Some(Self::KnownBadEntries),
            2 => Some(Self::E170BadRows),
            3..=u8::MAX => None,
        }
    }
}

impl HandOffField for StreamName {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        bytes.push(match self {
            Self::NewPoolFileCreation => 0,
            Self::FixedScriptReuseAfterRaisingFloorThenUnmount => 1,
        });
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::NewPoolFileCreation),
            1 => Some(Self::FixedScriptReuseAfterRaisingFloorThenUnmount),
            2..=u8::MAX => None,
        }
    }
}

impl HandOffField for CallOutcome {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Succeeded => bytes.push(0),
            Self::ReturnedError(member) => {
                bytes.push(1);
                member.put_into(bytes);
            }
            Self::Panicked(text) => {
                bytes.push(2);
                text.put_into(bytes);
            }
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::Succeeded),
            1 => Some(Self::ReturnedError(String::take_from(reader)?)),
            2 => Some(Self::Panicked(String::take_from(reader)?)),
            3..=u8::MAX => None,
        }
    }
}

impl HandOffField for ModelVerdictOfCall {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::NotModelled => bytes.push(0),
            Self::Agrees => bytes.push(1),
            Self::Disagrees {
                aspect,
                model_answer,
                implementation_answer,
            } => {
                bytes.push(2);
                aspect.put_into(bytes);
                model_answer.put_into(bytes);
                implementation_answer.put_into(bytes);
            }
            Self::SkippedAfterTheModelStopped => bytes.push(3),
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::NotModelled),
            1 => Some(Self::Agrees),
            2 => Some(Self::Disagrees {
                aspect: String::take_from(reader)?,
                model_answer: String::take_from(reader)?,
                implementation_answer: String::take_from(reader)?,
            }),
            3 => Some(Self::SkippedAfterTheModelStopped),
            4..=u8::MAX => None,
        }
    }
}

impl HandOffField for StreamEnding {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Completed => bytes.push(0),
            Self::StoppedByAnError { call_index, member } => {
                bytes.push(1);
                call_index.put_into(bytes);
                member.put_into(bytes);
            }
            Self::ImplementationCrashed { what } => {
                bytes.push(2);
                what.put_into(bytes);
            }
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::Completed),
            1 => Some(Self::StoppedByAnError {
                call_index: usize::take_from(reader)?,
                member: String::take_from(reader)?,
            }),
            2 => Some(Self::ImplementationCrashed {
                what: String::take_from(reader)?,
            }),
            3..=u8::MAX => None,
        }
    }
}

impl HandOffField for CrashJudgeVerdict {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::NotApplicable => bytes.push(0),
            Self::Green => bytes.push(1),
            Self::Red { what } => {
                bytes.push(2);
                what.put_into(bytes);
            }
            Self::Off => bytes.push(3),
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::NotApplicable),
            1 => Some(Self::Green),
            2 => Some(Self::Red {
                what: String::take_from(reader)?,
            }),
            3 => Some(Self::Off),
            4..=u8::MAX => None,
        }
    }
}

impl HandOffField for PoolCheckerOfAState {
    fn put_into(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Off => bytes.push(0),
            Self::RecordedForTheJudgeStage(table) => {
                bytes.push(1);
                table.put_into(bytes);
            }
            Self::Judged {
                red,
                matches_the_judgement,
            } => {
                bytes.push(2);
                red.put_into(bytes);
                matches_the_judgement.put_into(bytes);
            }
        }
    }
    fn take_from(reader: &mut HandOffReader<'_>) -> Option<Self> {
        match u8::take_from(reader)? {
            0 => Some(Self::Off),
            1 => Some(Self::RecordedForTheJudgeStage(
                RecordedTableReference::take_from(reader)?,
            )),
            2 => Some(Self::Judged {
                red: Vec::take_from(reader)?,
                matches_the_judgement: bool::take_from(reader)?,
            }),
            3..=u8::MAX => None,
        }
    }
}

/// 交接的全部字节：头、这一趟的参数（判段照它核两段是不是同一条命令）、录入段交出的那一份。
fn hand_off_bytes<HandOff: HandOffField>(run_arguments: &[String], hand_off: &HandOff) -> Vec<u8> {
    let mut bytes = HAND_OFF_MAGIC.to_vec();
    run_arguments.to_vec().put_into(&mut bytes);
    hand_off.put_into(&mut bytes);
    bytes
}

/// 读回交接：头对得上、参数与这一趟的相同、录入段交出的那一份解得出、末尾没有多出的字节。
///
/// # Errors
/// 上面四样有一样不成，一句话说哪一样、出路是什么。
fn hand_off_of_the_bytes<HandOff: HandOffField>(
    bytes: &[u8],
    run_arguments: &[String],
) -> Result<HandOff, String> {
    let mut reader = HandOffReader { bytes, position: 0 };
    if reader.take_bytes(HAND_OFF_MAGIC.len()) != Some(HAND_OFF_MAGIC.as_slice()) {
        return Err(format!(
            "交接的头不是 {}：不是这一版装置的录入段写的，重跑录入段（{STAGE_ARGUMENT} record）",
            String::from_utf8_lossy(HAND_OFF_MAGIC)
        ));
    }
    let recorded_arguments = Vec::<String>::take_from(&mut reader)
        .ok_or_else(|| "交接里的参数解不出：重跑录入段".to_string())?;
    if recorded_arguments != run_arguments {
        return Err(format!(
            "录入段那一趟的参数是 {recorded_arguments:?}，这一趟是 {run_arguments:?}：判段照录入段那一趟的参数起（只换 {STAGE_ARGUMENT}）"
        ));
    }
    let hand_off = HandOff::take_from(&mut reader)
        .ok_or_else(|| "交接里录入段交出的那一份解不出：重跑录入段".to_string())?;
    if !reader.is_at_the_end() {
        return Err("交接末尾多出字节：重跑录入段".to_string());
    }
    Ok(hand_off)
}

/// 照 `stage` 跑两段，交回退出码：
/// - 只跑录入段：`record` 交出交接，写进输出目录里的 [`HAND_OFF_TO_THE_JUDGE_STAGE_FILE`]，不判，交回 [`EXIT_CODE_OF_A_RECORD_ONLY_RUN`]；
/// - 只跑判段：不跑 `record`，从那个文件读回交接交 `judge`，判完删掉那个文件（`judge` 拒了或 panic 了就留着，可以只重跑判段）；
/// - 两段先后跑：`record` 交出的交接在内存里写成字节、读回，交 `judge`（与分两趟同一种交法，不落文件）。
///
/// # Errors
/// 只跑判段而读不到交接（录入段没跑过：报错句点名先跑录入段）、交接读回不成（[`hand_off_of_the_bytes`]）、`judge` 拒了
/// （交接里录的盘与库里的对不上）；交接写不进、删不掉。
fn run_in_stages<HandOff: HandOffField>(
    stage: RunStage,
    run_arguments: &[String],
    output_directory: &Path,
    record: impl FnOnce() -> HandOff,
    judge: impl FnOnce(HandOff) -> Result<i32, String>,
) -> Result<i32, String> {
    let hand_off_path = output_directory.join(HAND_OFF_TO_THE_JUDGE_STAGE_FILE);
    match stage {
        RunStage::RecordOnly => {
            let bytes = hand_off_bytes(run_arguments, &record());
            std::fs::create_dir_all(output_directory)
                .map_err(|error| format!("建 {}：{error}", output_directory.display()))?;
            std::fs::write(&hand_off_path, bytes.as_slice())
                .map_err(|error| format!("交接写不进 {}：{error}", hand_off_path.display()))?;
            Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN)
        }
        RunStage::JudgeOnly => {
            let bytes = std::fs::read(&hand_off_path).map_err(|error| {
                format!(
                    "读不到录入段留下的交接 {}（{error}）：先跑录入段（{STAGE_ARGUMENT} record，或不给 {STAGE_ARGUMENT}、两段先后跑）",
                    hand_off_path.display()
                )
            })?;
            let hand_off = hand_off_of_the_bytes(&bytes, run_arguments)?;
            let exit_code = judge(hand_off)?;
            std::fs::remove_file(&hand_off_path)
                .map_err(|error| format!("交接删不掉 {}：{error}", hand_off_path.display()))?;
            Ok(exit_code)
        }
        RunStage::RecordThenJudge => {
            let bytes_handed_over_in_memory = hand_off_bytes(run_arguments, &record());
            judge(hand_off_of_the_bytes(
                &bytes_handed_over_in_memory,
                run_arguments,
            )?)
        }
    }
}

/// 从命令行里取出 `--stage <段>`（至多一次，取值照 `RunStage::of_the_variable_value`：`record`、`judge`），交回段与其余的参数
/// （进交接、两段核是不是同一条命令的那一份）。写错、给了两次交回 `None`。
fn stage_and_the_run_arguments(arguments: &[String]) -> Option<(RunStage, Vec<String>)> {
    let mut stage_word: Option<&str> = None;
    let mut run_arguments = Vec::new();
    let mut words = arguments.iter();
    while let Some(word) = words.next() {
        if word == STAGE_ARGUMENT {
            if stage_word.is_some() {
                return None;
            }
            stage_word = Some(words.next()?.as_str());
        } else {
            run_arguments.push(word.clone());
        }
    }
    let stage = RunStage::of_the_variable_value(stage_word).ok()?;
    Some((stage, run_arguments))
}

/// `known-bad-images` 模式照 `stage` 跑（[`run_in_stages`]：录入段 [`record_the_known_bad_images`]，判段 [`judge_the_known_bad_images`]）。
///
/// # Errors
/// 同 [`run_in_stages`]。
fn run_known_bad_images(
    command_line: &KnownBadImagesCommandLine,
    stage: RunStage,
    run_arguments: &[String],
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    result_lines_output: &mut dyn std::io::Write,
) -> Result<i32, String> {
    run_in_stages(
        stage,
        run_arguments,
        &command_line.output_directory,
        || record_the_known_bad_images(command_line, tables),
        |recorded| judge_the_known_bad_images(command_line, recorded, tables, result_lines_output),
    )
}

/// 录入段交给判段的 `baseline`、`row` 那一趟：与流无关的 S1 回比（只在基线那一趟有）与两条流各自导出之后的样子（O2 录下了、没判）。
struct RecordedDump {
    stream_independent_checks: Vec<StopCheck>,
    exports: Vec<StreamExport>,
}

/// `baseline`、`row` 的录入段：与流无关的 S1 回比（只在基线那一趟），两条流各导出一遍（[`export_stream`]：驱动、取样、写录入段写得了的导出文件，
/// O2 要判的盘录进 `tables`）；流水线里 panic 的那条流记成没导出。
fn record_the_dump(
    command_line: &CommandLine,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
) -> RecordedDump {
    let stream_independent_checks = if command_line.run_kind == RunKind::Baseline {
        stream_independent_checks()
    } else {
        Vec::new()
    };
    let exports = STREAM_NAMES
        .iter()
        .map(|name| {
            let started = Instant::now();
            let exported = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                export_stream(
                    *name,
                    &command_line.output_directory,
                    command_line.run_kind,
                    command_line.switches,
                    tables,
                )
            }));
            exported.unwrap_or_else(|payload| {
                StreamExport::not_exported(
                    *name,
                    format!("pipeline_panicked:{}", panic_text(&*payload)),
                    started,
                )
            })
        })
        .collect();
    RecordedDump {
        stream_independent_checks,
        exports,
    }
}

/// `baseline`、`row` 的判段：每条流录下的盘交 `tables` 判（[`judge_the_recorded_pool_checker_tables`]）；引擎在某一态上失败的流照两段之前记成
/// 某个状态上实现自崩（[`StreamExport::record_an_engine_failure_in_a_state`]），导出了的流写 `oracle-verdicts.tsv`，没导出的流清掉留给判段判的观测；
/// 再照一趟写完时的次序打全部结果行（S5 在这里判），交回退出码。结果行写进 `result_lines_output`。
///
/// # Errors
/// 交接与录下的对不上（[`PoolCheckerTablesOfTheTwoStages::judge_a_recorded_table`] 的 `Err`）：判段一个文件都不写、一行结果行都不打就交回那一句。
fn judge_the_dump(
    command_line: &CommandLine,
    recorded: RecordedDump,
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    result_lines_output: &mut dyn std::io::Write,
) -> Result<i32, String> {
    let run_kind = command_line.run_kind;
    let switches = command_line.switches;
    let RecordedDump {
        stream_independent_checks,
        mut exports,
    } = recorded;
    let threads = sampling_threads_of_this_run(switches);
    let judged_streams = exports
        .iter_mut()
        .map(|export| {
            judge_the_recorded_pool_checker_tables(
                &mut export.observations,
                tables,
                threads,
                switches,
            )
        })
        .collect::<Result<Vec<JudgedTablesOfAStream>, String>>()?;
    for (export, judged) in exports.iter_mut().zip(judged_streams) {
        let directory = command_line
            .output_directory
            .join(export.name.directory_name());
        match judged {
            JudgedTablesOfAStream::AllJudged => {}
            JudgedTablesOfAStream::EngineFailedInAState { what } => {
                export.record_an_engine_failure_in_a_state(&directory, &what);
            }
        }
        if export.not_exported.is_none() {
            write_the_oracle_verdicts_file(&directory, &export.observations, &mut export.files);
        } else {
            export.observations = Vec::new();
        }
    }
    let mut lines = ResultLines {
        emitted: 0,
        output: result_lines_output,
    };
    lines.emit_result_line(&format!(
        "name=config segment=s1 mode={} streams={} sector_bytes={LOCAL_SECTOR_BYTES} first_batch_divisor={FIRST_BATCH_DIVISOR} second_batch_divisor={SECOND_BATCH_DIVISOR} third_batch_divisor={THIRD_BATCH_DIVISOR} torn_states_per_segment={TORN_STATES_PER_SEGMENT} torn_scan_ordinals_per_segment={TORN_SCAN_ORDINALS_PER_SEGMENT} third_batch_sampled={} positive_control={} disabled_oracle={}",
        if run_kind == RunKind::Baseline { "baseline" } else { "row" },
        STREAM_NAMES.len(),
        run_kind == RunKind::Baseline,
        switches
            .positive_control
            .map_or_else(|| "none".to_string(), |control| format!("{control:?}")),
        switches
            .disabled_oracle
            .map_or_else(|| "none".to_string(), |oracle| format!("{oracle:?}")),
    ));
    let mut failed_checks = 0usize;
    for check in &stream_independent_checks {
        failed_checks += usize::from(!check.holds());
        lines.emit_result_line(&format!(
            "name=s1 stream=both check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let mut totals = [0usize; 7];
    let mut streams_not_exported = 0usize;
    let mut not_applicable_source_checks_failed = 0usize;
    for mut export in exports {
        streams_not_exported += usize::from(export.not_exported.is_some());
        let not_applicable_source_checks =
            not_applicable_source_checks_to_judge(&export, run_kind, switches);
        not_applicable_source_checks_failed += not_applicable_source_checks
            .iter()
            .filter(|check| !check.holds())
            .count();
        export.checks.extend(not_applicable_source_checks);
        let stream_totals = report_stream(&mut lines, &export);
        for (total, value) in totals.iter_mut().zip(stream_totals) {
            *total += value;
        }
    }
    failed_checks += totals[0];
    let [_, rebuilt_digest_mismatches, mask_mismatches, pool_checker_direct_mismatches, red_calls, crash_judge_red, pool_checker_red] =
        totals;
    lines.emit_result_line(&format!(
        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={rebuilt_digest_mismatches} crash_facts_mask_mismatches={mask_mismatches} s3_o2_judgement_mismatches={pool_checker_direct_mismatches} streams_not_exported={streams_not_exported} o1_comparison_red_calls={red_calls} o1_crash_judge_red_states={crash_judge_red} o2_red_states={pool_checker_red}"
    ));
    lines.finish();
    let stop_clause_hit = failed_checks > 0
        || rebuilt_digest_mismatches > 0
        || pool_checker_direct_mismatches > 0
        || (run_kind == RunKind::Baseline
            && (not_applicable_source_checks_failed > 0 || streams_not_exported > 0));
    if stop_clause_hit {
        Ok(3)
    } else {
        Ok(0)
    }
}

/// `baseline`、`row` 照 `stage` 跑（[`run_in_stages`]：录入段 [`record_the_dump`]，判段 [`judge_the_dump`]）。
///
/// # Errors
/// 同 [`run_in_stages`]。
fn run_dump(
    command_line: &CommandLine,
    stage: RunStage,
    run_arguments: &[String],
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    result_lines_output: &mut dyn std::io::Write,
) -> Result<i32, String> {
    run_in_stages(
        stage,
        run_arguments,
        &command_line.output_directory,
        || record_the_dump(command_line, tables),
        |recorded| judge_the_dump(command_line, recorded, tables, result_lines_output),
    )
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some((stage, parsed, run_arguments)) =
        stage_and_the_run_arguments(&arguments).and_then(|(stage, run_arguments)| {
            parse_command_line(&run_arguments).map(|parsed| (stage, parsed, run_arguments))
        })
    else {
        eprintln!(
            "用法：e171_three_oracle_collusion baseline|row --output-directory <目录> [--positive-control O1-对拍|O1-崩溃判官|O2 | --disable-oracle O1|O2] [--sampling-threads <正整数>]，或 known-bad-images --dump-directory <基线导出目录> --output-directory <目录> [--e170-bad-rows] [--sampling-threads <正整数>]；都可以加 {STAGE_ARGUMENT} record|judge；收到 {arguments:?}"
        );
        std::process::exit(2);
    };
    let output_directory = match &parsed {
        ParsedCommand::Dump(command_line) => &command_line.output_directory,
        ParsedCommand::KnownBadImages(command_line) => &command_line.output_directory,
    };
    if stage.records() && output_directory.exists() {
        eprintln!(
            "输出目录 {} 已存在：装置不覆盖已有的文件，换一个目录（只跑判段的那一趟才接着录入段那一趟的目录：{STAGE_ARGUMENT} judge）",
            output_directory.display()
        );
        std::process::exit(2);
    }
    let tables = GivenStatesTablesInTheLibrary;
    let ran = match &parsed {
        ParsedCommand::Dump(command_line) => run_dump(
            command_line,
            stage,
            &run_arguments,
            &tables,
            &mut std::io::stdout(),
        ),
        ParsedCommand::KnownBadImages(command_line) => run_known_bad_images(
            command_line,
            stage,
            &run_arguments,
            &tables,
            &mut std::io::stdout(),
        ),
    };
    match ran {
        Ok(0) => {}
        Ok(exit_code) => std::process::exit(exit_code),
        Err(refusal) => {
            eprintln!("{refusal}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 空消息的 sha256（`python3 -c "import hashlib; print(hashlib.sha256(b'').hexdigest())"` 现算）。
    const EMPTY_MESSAGE_SHA256: &str =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    /// 盘 1、扇区 2、512 个 0xAB 那一条的摘要（`struct.pack('<IQ', 1, 2) + b'\xab' * 512` 喂 hashlib 现算）。
    const ONE_SECTOR_OF_0XAB_SHA256: &str =
        "c71cae1b3fc678e4cabfa21d575c728edd44a4b372b3389ce763856273abecbf";

    #[test]
    fn torn_landing_folds_to_one_and_a_persisted_write_to_two() {
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

    /// 两条流本地段长表的闭式 1 + Σ(2^n − 1) 与写数，与本地抄来的闭式、写数逐个相等（改段长表里任何一个数，这里都红）。
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
        assert_eq!(LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM, 1_678_377_095);
        assert_eq!(LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM, 16_777_243);
    }

    /// 两条流驱动完，`baseline` 回比的段长表、写数、根槽写数、盘数与盘字节数（`prepare_stream` 那一批 S1）逐条成立：
    /// 本地表与现在录到的流分叉时这里红，用不着跑一趟 `baseline`。闭式与写数都对而段的次序不对的表只有这条抓得到。
    #[test]
    fn stop_checks_of_both_streams_hold_against_the_streams_recorded_now() {
        for name in STREAM_NAMES {
            let stream = prepare_stream(name, RunSwitches::default(), true);
            assert_eq!(
                stream.ending,
                StreamEnding::Completed,
                "{} 没有驱动到头",
                name.directory_name()
            );
            let broken: Vec<String> = stream
                .checks
                .iter()
                .filter(|check| !check.holds())
                .map(|check| {
                    format!(
                        "{} local={} crates={}",
                        check.name, check.local, check.crates
                    )
                })
                .collect();
            assert!(
                broken.is_empty(),
                "{} 的 S1 没过：{broken:?}",
                name.directory_name()
            );
        }
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

    /// 两块盘上各改一个扇区的两份镜像：补丁要两块盘都带上，叠回干净镜像之后摘要与改坏的那份相同（X15：补丁提取只取盘 0 的差，这里红）。
    #[test]
    fn the_sector_difference_between_two_images_carries_the_changed_sectors_of_every_device() {
        let devices = [DeviceIdentity(0), DeviceIdentity(1)];
        let mut clean = MemoryPool::with_devices(&devices, 1 << 20);
        clean.apply_writes(&[bytes_write(0, 0, vec![0x11; 512])]);
        let mut bad = clean.clone();
        bad.apply_writes(&[
            bytes_write(0, 512, vec![0x44; 512]),
            bytes_write(1, 1024, vec![0x55; 1024]),
        ]);
        let patches = sector_difference_as_patches(&clean, &bad);
        assert_eq!(
            patches,
            vec![
                BytePatch {
                    device: 0,
                    offset: 512,
                    bytes: vec![0x44; 512]
                },
                BytePatch {
                    device: 1,
                    offset: 1024,
                    bytes: vec![0x55; 1024]
                },
            ],
            "盘 1 上相邻的两个扇区并成一段补丁"
        );
        let mut patched = clean.clone();
        apply_patches(&mut patched, &patches);
        assert_eq!(
            sha256_text(&memory_pool_digest(&patched)),
            sha256_text(&memory_pool_digest(&bad))
        );
        assert_ne!(
            sha256_text(&memory_pool_digest(&clean)),
            sha256_text(&memory_pool_digest(&bad)),
            "改坏之前与之后摘要不同"
        );
    }

    /// E170 的 P 行只翻一个字节：补丁不按扇区对齐，施加时读出罩住它的整扇区、只换那一个字节、再整扇区写回。
    #[test]
    fn one_byte_patch_inside_a_sector_changes_only_that_byte() {
        let devices = [DeviceIdentity(0), DeviceIdentity(1)];
        let mut pool = MemoryPool::with_devices(&devices, 1 << 20);
        pool.apply_writes(&[bytes_write(1, 512, vec![0x11; 1024])]);
        apply_patches(
            &mut pool,
            &[BytePatch {
                device: 1,
                offset: 700,
                bytes: vec![0xEE],
            }],
        );
        let read_back = pool.devices[&DeviceIdentity(1)].read(DeviceOffsetInBytes(512), 1024);
        assert_eq!(read_back[700 - 512], 0xEE);
        assert!(read_back
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 700 - 512 || *byte == 0x11));
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
    fn read_back_of_a_version_the_model_never_committed_is_red_and_a_committed_one_is_green() {
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

    /// 模型没有会话时问发布第一个文件：答案本身就是一格对不上，驱动把它记成 `Disagrees`、模型停在这次调用，下一次调用不比
    /// （X5：`judge_and_advance` 的不同丢掉，这里红）。
    #[test]
    fn comparison_the_model_cannot_answer_is_recorded_as_a_disagreement_and_stops_the_model() {
        let mut driver = StreamDriver::new(
            StreamName::NewPoolFileCreation,
            SharedStream::retaining_contents(),
            0,
            RunSwitches::default(),
        );
        let refused: Result<(), &str> = Err("拒");
        assert!(driver.call("publish", || refused).is_err());
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
        assert!(driver.call("publish", || accepted).is_ok());
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
    /// （X5：`judge_and_advance` 的不同丢掉，这里红；上一条测的是模型连答案都给不出的那一路）。
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
        assert!(driver.call("publish", || refused).is_err());
        let observed = driver.refusal(
            &"拒",
            singlefs_verification::model::ObservedRefusalReason::Unexplained,
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
    /// （登记第一节读法 5 末句；装置一侧变异：记红丢掉，这里红）。
    #[test]
    fn refused_acquire_on_a_stream_without_faults_is_recorded_as_a_comparison_red_and_stops_the_model(
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
            .call("acquire", || refused_acquire)
            .expect_err("取号被拒");
        driver.record_a_refusal_the_model_only_goes_forward_on(acquire_failure);
        assert_eq!(
            driver.calls[0].model,
            ModelVerdictOfCall::Disagrees {
                aspect: "模型说该成、实现拒了".to_string(),
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
        assert!(after_a_good_acquire.call("acquire", || accepted).is_ok());
        let refused_warm_up: Result<(), RefusedForTheTest> = Err(
            RefusedForTheTest::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
        );
        let warm_up_failure = after_a_good_acquire
            .call("warm_up", || refused_warm_up)
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

    /// 取号 panic 是实现自崩（读法 5），不是对拍的红：这一栏留在 `not_modelled`，这条流走自崩那一类、不导出。
    #[test]
    fn panic_in_the_acquire_is_an_implementation_crash_and_not_a_comparison_red() {
        let mut driver = StreamDriver::new(
            StreamName::NewPoolFileCreation,
            SharedStream::retaining_contents(),
            0,
            RunSwitches::default(),
        );
        let failure = driver
            .call("acquire", || -> Result<(), RefusedForTheTest> {
                panic!("取号自崩")
            })
            .expect_err("panic 交回 Err(None)");
        assert!(failure.is_none());
        driver.record_a_refusal_the_model_only_goes_forward_on(failure);
        assert_eq!(driver.calls[0].model, ModelVerdictOfCall::NotModelled);
        assert!(driver.implementation_crash.is_some());
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
                sampling_threads: None,
            },
        );
        let refused: Result<(), RefusedForTheTest> = Err(
            RefusedForTheTest::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness,
        );
        let failure = driver.call("acquire", || refused).expect_err("取号被拒");
        driver.record_a_refusal_the_model_only_goes_forward_on(failure);
        assert_eq!(driver.calls[0].model, ModelVerdictOfCall::NotModelled);
    }

    /// 真池上走一遍取号、暖机：O1-对拍那一栏没有红。照攻方 K4-c 的造法（`crates/singlefs-core/src/transaction.rs` 取号逐盘核 `<` 写成 `<=`，
    /// 不带故障的盘上每次取号都拒），核心一改这一条就红，因为装置把被拒的取号记成了 O1-对拍的红；装置不记红（回到修之前）时这一条不红。
    /// 只断言对拍一栏、不断言取号成功（断言取号成功的是下一条，那一条在装置不记红时也红，分不出装置记没记）。
    #[test]
    fn an_acquire_and_a_warm_up_on_a_fresh_pool_leave_the_comparison_column_without_a_red() {
        let mut live = LiveStream::start(
            StreamName::NewPoolFileCreation,
            "e171-unit-acquire-and-warm-up-comparison",
            RunSwitches::default(),
        );
        let _ = live.step_acquire_and_warm_up();
        let calls_red_in_the_comparison: Vec<&str> = live
            .driver
            .calls
            .iter()
            .filter(|call| matches!(call.model, ModelVerdictOfCall::Disagrees { .. }))
            .map(|call| call.label.as_str())
            .collect();
        assert!(
            calls_red_in_the_comparison.is_empty(),
            "取号或暖机被拒、O1-对拍记了红：{calls_red_in_the_comparison:?}"
        );
    }

    /// 真池上取号、暖机的绝对值：两次调用都成功、模型都不比（这一步模型不交答）、模型没停、第一次取到的号是 1。
    #[test]
    fn an_acquire_and_a_warm_up_on_a_fresh_pool_succeed_and_the_model_is_not_asked() {
        let mut live = LiveStream::start(
            StreamName::NewPoolFileCreation,
            "e171-unit-acquire-and-warm-up-absolute",
            RunSwitches::default(),
        );
        assert!(live.step_acquire_and_warm_up());
        let labels: Vec<&str> = live
            .driver
            .calls
            .iter()
            .map(|call| call.label.as_str())
            .collect();
        assert_eq!(labels, vec!["acquire", "warm_up"]);
        assert!(live
            .driver
            .calls
            .iter()
            .all(|call| call.outcome == CallOutcome::Succeeded
                && call.model == ModelVerdictOfCall::NotModelled));
        assert_eq!(live.driver.model_stopped_at_call, None);
        assert_eq!(live.instance, Some(InstanceGeneration(1)));
    }

    fn recorded_call_for_the_test(
        label: &'static str,
        outcome: CallOutcome,
        model: ModelVerdictOfCall,
    ) -> RecordedCall {
        RecordedCall {
            label: label.to_string(),
            first_operation: 0,
            return_point: 0,
            outcome,
            model,
        }
    }

    fn disagreement_for_the_test() -> ModelVerdictOfCall {
        ModelVerdictOfCall::Disagrees {
            aspect: "模型说该成、实现拒了".to_string(),
            model_answer: "往前走".to_string(),
            implementation_answer: "拒了:X".to_string(),
        }
    }

    /// 逐行那一趟的 S5：返回了错误而 O1-对拍没问模型（`not_modelled`）的调用、与不适用来自卸载和对拍不同之外的来源，各自回比 0，
    /// 不是 0 就是停机。绝对值：期望写 `0`，实际写不合的个数。
    #[test]
    fn the_row_not_applicable_source_checks_count_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source(
    ) {
        let errored = |text: &str| CallOutcome::ReturnedError(text.to_string());
        // 取号出错、O1-对拍没记红（修之前的样子）：第一样 1 处不合；模型停在这次调用、来源不是卸载也不是对拍不同，但没有不适用的态，第二样 0。
        let silent = [recorded_call_for_the_test(
            "acquire",
            errored("X"),
            ModelVerdictOfCall::NotModelled,
        )];
        let silent_acquire_checks =
            not_applicable_source_checks_of_a_bad_implementation_row("s", &silent, Some(0), 0);
        assert_eq!(
            silent_acquire_checks
                .iter()
                .map(|check| check.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "s_o1_every_errored_call_but_the_unmount_is_judged_by_the_comparison",
                "s_o1_not_applicable_only_after_the_unmount_or_a_disagreement"
            ]
        );
        assert_eq!(
            silent_acquire_checks
                .iter()
                .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
                .collect::<Vec<_>>(),
            vec![("0", "1", false), ("0", "0", true)]
        );
        // 取号出错、记成对拍的红：两样都 0，不适用的态来自对拍不同，合规。
        let red = [recorded_call_for_the_test(
            "acquire",
            errored("X"),
            disagreement_for_the_test(),
        )];
        let recorded_red_checks =
            not_applicable_source_checks_of_a_bad_implementation_row("s", &red, Some(0), 9);
        assert!(recorded_red_checks.iter().all(StopCheck::holds));
        // 对拍在发布上判出不同、模型停在那里，之后的可写挂载出错（`skipped_after_the_model_stopped`，第 108 行那一形）：登记读法 6 的合规来源，
        // 两样都 0（登记修订 4）。
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
        let error_after_the_stop_checks = not_applicable_source_checks_of_a_bad_implementation_row(
            "s",
            &error_after_the_stop,
            Some(0),
            6,
        );
        assert_eq!(
            error_after_the_stop_checks
                .iter()
                .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
                .collect::<Vec<_>>(),
            vec![("0", "0", true), ("0", "0", true)]
        );
        // 发布出错而模型说拒得对（`agrees`）、之后有 5 个不适用的态：不是 O1-对拍的红，来源不在登记的两种里，第二样 5 处不合。
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
        let allowed_refusal_checks = not_applicable_source_checks_of_a_bad_implementation_row(
            "s",
            &allowed_refusal,
            Some(2),
            5,
        );
        assert_eq!(
            allowed_refusal_checks
                .iter()
                .map(|check| (check.local.as_str(), check.crates.as_str(), check.holds()))
                .collect::<Vec<_>>(),
            vec![("0", "0", true), ("0", "5", false)]
        );
        // 模型停在卸载：不适用的态合规；卸载出错也不算（模型没有卸载的答）。
        let unmount_stop = [
            recorded_call_for_the_test(
                "publish",
                CallOutcome::Succeeded,
                ModelVerdictOfCall::Agrees,
            ),
            recorded_call_for_the_test(
                UNMOUNT_CALL_LABEL,
                errored("Z"),
                ModelVerdictOfCall::NotModelled,
            ),
        ];
        let unmount_stop_checks = not_applicable_source_checks_of_a_bad_implementation_row(
            "s",
            &unmount_stop,
            Some(1),
            7,
        );
        assert!(unmount_stop_checks.iter().all(StopCheck::holds));
        // 模型没停：没有不适用的态，合规。
        let no_stop = [recorded_call_for_the_test(
            "publish",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::Agrees,
        )];
        let no_stop_checks =
            not_applicable_source_checks_of_a_bad_implementation_row("s", &no_stop, None, 0);
        assert!(no_stop_checks.iter().all(StopCheck::holds));
    }

    /// 哪一趟判哪一份 S5：逐行那一趟判「返回了错误的调用都记了对拍的红」那一份，基线那一趟判原来那一份；
    /// 关掉 O1、驱动没走完整的流不判；基线带阳性对照、流没导出也不判。
    #[test]
    fn the_row_run_judges_the_row_not_applicable_source_checks_and_the_baseline_run_judges_the_baseline_ones(
    ) {
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
        let row_checks = not_applicable_source_checks_to_judge(
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
                (
                    "new_pool_file_creation_o1_every_errored_call_but_the_unmount_is_judged_by_the_comparison",
                    false
                ),
                (
                    "new_pool_file_creation_o1_not_applicable_only_after_the_unmount_or_a_disagreement",
                    true
                )
            ]
        );
        // 阳性对照不挡它（对拍的红正是它要看的）；关掉 O1 不判。
        let with_the_comparison_control = RunSwitches {
            positive_control: Some(PositiveControl::ComparisonOfTheCalls),
            disabled_oracle: None,
            sampling_threads: None,
        };
        assert_eq!(
            not_applicable_source_checks_to_judge(
                &export,
                RunKind::BadImplementationRow,
                with_the_comparison_control
            )
            .len(),
            2
        );
        let with_the_model_off = RunSwitches {
            positive_control: None,
            disabled_oracle: Some(DisabledOracle::ComparisonAndCrashJudge),
            sampling_threads: None,
        };
        assert!(not_applicable_source_checks_to_judge(
            &export,
            RunKind::BadImplementationRow,
            with_the_model_off
        )
        .is_empty());
        // 驱动没走完整（实现 panic）与流水线 panic（`ending` 是 `None`）的流不判，走自崩那一类。
        let mut crashed = silent_acquire_stopped_by_the_error();
        crashed.ending = Some(StreamEnding::ImplementationCrashed {
            what: "x".to_string(),
        });
        assert!(not_applicable_source_checks_to_judge(
            &crashed,
            RunKind::BadImplementationRow,
            RunSwitches::default()
        )
        .is_empty());
        crashed.ending = None;
        assert!(not_applicable_source_checks_to_judge(
            &crashed,
            RunKind::BadImplementationRow,
            RunSwitches::default()
        )
        .is_empty());
        // 基线那一趟判原来那一份：干净的流一两样都过；带阳性对照、流没导出的不判。
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
        ];
        let baseline_checks = not_applicable_source_checks_to_judge(
            &clean,
            RunKind::Baseline,
            RunSwitches::default(),
        );
        assert_eq!(
            baseline_checks
                .iter()
                .map(|check| (check.name.as_str(), check.holds()))
                .collect::<Vec<_>>(),
            vec![
                (
                    "new_pool_file_creation_o1_model_stops_only_at_the_unmount",
                    true
                ),
                (
                    "new_pool_file_creation_every_call_is_judged_or_is_the_unmount",
                    true
                )
            ]
        );
        let with_the_pool_checker_control = RunSwitches {
            positive_control: Some(PositiveControl::PoolChecker),
            disabled_oracle: None,
            sampling_threads: None,
        };
        assert!(not_applicable_source_checks_to_judge(
            &clean,
            RunKind::Baseline,
            with_the_pool_checker_control
        )
        .is_empty());
        clean.not_exported = Some("x".to_string());
        assert!(not_applicable_source_checks_to_judge(
            &clean,
            RunKind::Baseline,
            RunSwitches::default()
        )
        .is_empty());
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
    fn patch_list_text_round_trips_through_the_patches_file() {
        let patches = vec![
            BytePatch {
                device: 1,
                offset: 4096,
                bytes: vec![1, 2, 3],
            },
            BytePatch {
                device: 0,
                offset: 0,
                bytes: vec![9; 5],
            },
        ];
        let mut patches_file = Vec::new();
        let list = patch_list_text(&patches, &mut patches_file);
        assert_eq!(list, "1:4096:0:3,0:0:3:5");
        assert_eq!(patches_of_a_patch_list_text(&list, &patches_file), patches);
        assert_eq!(patch_list_text(&[], &mut Vec::new()), "-");
    }

    #[test]
    fn call_outcome_writes_ok_or_error_into_the_calls_file_and_a_word_into_the_outcomes_file() {
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
    }

    /// 同时判的线程数：对拍里「几条线程」那一趟与连续段的单测用的，不是产品路径的线程数（产品路径 [`sampling_threads_of_this_run`]）。
    const SEVERAL_THREADS: NonZeroUsize = NonZeroUsize::new(4).expect("4 不是 0");

    /// 线程对拍每批取几个取样态：够几条线程同时领到不同的连续段，一趟不跑太久。
    const SAMPLED_STATES_OF_EACH_BATCH_IN_THE_PAIRING: usize = 12;

    /// 让几个连续段按 `finishing_order` 的次序判完：轮到谁谁才交回，还没轮到的等着。要这几个连续段同时在跑（线程数不少于连续段数）。
    struct RunsFinishingInAGivenOrder {
        finishing_order: Vec<usize>,
        finished: std::sync::Mutex<usize>,
        one_finished: std::sync::Condvar,
    }

    impl RunsFinishingInAGivenOrder {
        fn new(finishing_order: &[usize]) -> Self {
            Self {
                finishing_order: finishing_order.to_vec(),
                finished: std::sync::Mutex::new(0),
                one_finished: std::sync::Condvar::new(),
            }
        }

        fn wait_for_the_turn_of(&self, run: usize) {
            let mut finished = self.finished.lock().expect("这把锁里不 panic");
            while self.finishing_order[*finished] != run {
                finished = self.one_finished.wait(finished).expect("这把锁里不 panic");
            }
            *finished += 1;
            self.one_finished.notify_all();
        }
    }

    #[test]
    fn outcomes_come_back_in_run_order_when_the_later_runs_finish_earlier() {
        let turns = RunsFinishingInAGivenOrder::new(&[3, 2, 1, 0]);
        let outcomes = outcomes_of_the_runs_in_their_order(
            4,
            SEVERAL_THREADS,
            &|_outcome: &usize| false,
            &|run| {
                turns.wait_for_the_turn_of(run);
                run
            },
        );
        assert_eq!(outcomes, vec![0, 1, 2, 3], "按连续段的次序，不按判完的次序");
    }

    #[test]
    fn on_one_thread_no_run_after_a_stopping_run_is_taken() {
        let taken = std::sync::Mutex::new(Vec::new());
        let outcomes = outcomes_of_the_runs_in_their_order(
            5,
            NonZeroUsize::MIN,
            &|outcome: &usize| *outcome == 2,
            &|run| {
                taken.lock().expect("这把锁里不 panic").push(run);
                run
            },
        );
        assert_eq!(outcomes, vec![0, 1, 2], "到叫停的连续段为止，含它");
        assert_eq!(
            *taken.lock().expect("这把锁里不 panic"),
            vec![0, 1, 2],
            "叫停之后不再领新的"
        );
    }

    #[test]
    fn the_earliest_stopping_run_ends_the_outcomes_when_a_later_one_stopped_earlier() {
        let turns = RunsFinishingInAGivenOrder::new(&[3, 0, 2, 1]);
        let outcomes = outcomes_of_the_runs_in_their_order(
            4,
            SEVERAL_THREADS,
            &|outcome: &usize| *outcome == 1 || *outcome == 3,
            &|run| {
                turns.wait_for_the_turn_of(run);
                run
            },
        );
        assert_eq!(
            outcomes,
            vec![0, 1],
            "次序上第一个叫停的是连续段 1，判完得早的连续段 3 与排在后面的都丢掉"
        );
    }

    #[test]
    fn the_memory_cap_limits_the_sampling_threads_and_at_least_one_is_left() {
        const MEBIBYTE: u64 = 1024 * 1024;
        let requested = NonZeroUsize::new(31).expect("31 不是 0");
        let threads_under = |memory_cap_mebibytes: u64| {
            sampling_threads_within_the_memory_cap(requested, Some(memory_cap_mebibytes * MEBIBYTE))
                .get()
        };
        assert_eq!(
            sampling_threads_within_the_memory_cap(requested, None).get(),
            31,
            "没有上限照要的开"
        );
        assert_eq!(threads_under(1024 * 1024), 31, "上限放得下时不多于要的");
        assert_eq!(threads_under(9216), 28, "(9216 - 2048) / 256");
        assert_eq!(threads_under(4096), 8, "(4096 - 2048) / 256");
        assert_eq!(threads_under(1024), 1, "连留给线程之外的都不够时仍开一条");
    }

    #[test]
    fn only_the_positive_control_of_the_crash_judge_takes_one_thread() {
        let switches_of = |positive_control| RunSwitches {
            positive_control,
            ..RunSwitches::default()
        };
        assert_eq!(
            sampling_threads_given_the_positive_control(
                SEVERAL_THREADS,
                switches_of(Some(PositiveControl::CrashJudge))
            ),
            NonZeroUsize::MIN,
            "批 1 第一个读回了文件的态要一条线程按序号走才说得清"
        );
        for positive_control in [
            None,
            Some(PositiveControl::ComparisonOfTheCalls),
            Some(PositiveControl::PoolChecker),
        ] {
            assert_eq!(
                sampling_threads_given_the_positive_control(
                    SEVERAL_THREADS,
                    switches_of(positive_control)
                ),
                SEVERAL_THREADS,
                "{positive_control:?}"
            );
        }
    }

    #[test]
    fn the_sampling_threads_argument_is_parsed_and_zero_is_refused() {
        let arguments_with = |threads: &str| {
            [
                "baseline",
                "--output-directory",
                "目录",
                "--sampling-threads",
                threads,
            ]
            .map(str::to_string)
        };
        let Some(ParsedCommand::Dump(command_line)) = parse_command_line(&arguments_with("1"))
        else {
            panic!("baseline 带 --sampling-threads 1 要认");
        };
        assert_eq!(
            command_line.switches.sampling_threads,
            Some(NonZeroUsize::MIN)
        );
        assert!(
            parse_command_line(&arguments_with("0")).is_none(),
            "0 条线程不认"
        );
    }

    /// O2 两段的单测替身：录下的盘按引用存在内存里（判段从这里读回），判交给 `judge`；数录与判各几次。
    struct PoolCheckerTablesKeptInMemory<'judge> {
        pools: Mutex<BTreeMap<[u8; DIGEST_BYTES], MemoryPool>>,
        judge: &'judge PoolCheckerRedNamesOfAGivenImage<'judge>,
        tables_recorded: AtomicUsize,
        tables_judged: AtomicUsize,
    }

    impl<'judge> PoolCheckerTablesKeptInMemory<'judge> {
        fn judging_with(judge: &'judge PoolCheckerRedNamesOfAGivenImage<'judge>) -> Self {
            Self {
                pools: Mutex::new(BTreeMap::new()),
                judge,
                tables_recorded: AtomicUsize::new(0),
                tables_judged: AtomicUsize::new(0),
            }
        }

        fn tables_recorded(&self) -> usize {
            self.tables_recorded.load(Ordering::SeqCst)
        }

        fn tables_judged(&self) -> usize {
            self.tables_judged.load(Ordering::SeqCst)
        }
    }

    impl PoolCheckerTablesOfTheTwoStages for PoolCheckerTablesKeptInMemory<'_> {
        fn record_a_table(&self, table_name: &str, pool: &MemoryPool) -> RecordedTableReference {
            self.tables_recorded.fetch_add(1, Ordering::SeqCst);
            let mut message = table_name.as_bytes().to_vec();
            message.extend_from_slice(&memory_pool_digest(pool));
            let table = RecordedTableReference {
                table_name: table_name.to_string(),
                digest: sha256_digest(&message),
            };
            self.pools
                .lock()
                .expect("这把锁里不 panic")
                .insert(table.digest, pool.clone());
            table
        }

        fn judge_a_recorded_table(
            &self,
            table: &RecordedTableReference,
        ) -> Result<Vec<String>, String> {
            self.tables_judged.fetch_add(1, Ordering::SeqCst);
            let pool = self
                .pools
                .lock()
                .expect("这把锁里不 panic")
                .get(&table.digest)
                .cloned()
                .ok_or_else(|| format!("{} 没录过：先跑录入段", table.table_name))?;
            Ok((self.judge)(&table.table_name, &pool))
        }
    }

    /// 新池新建文件流的几样输入（写法同 `export_stream`），取样每批只留序号最大的 [`SAMPLED_STATES_OF_EACH_BATCH_IN_THE_PAIRING`] 个：
    /// 批 1 留着全持久态（整条流最大的序号），按序号排与按（批号、序号）排的次序不同，对拍看得出结果排没排好。
    struct StreamToSampleInThePairing {
        stream: PreparedStream,
        tables: CrashStateEnumerationTables,
        facts: FlowFacts,
        torn_image_index_by_write: BTreeMap<usize, usize>,
        sampled: SampledOrdinals,
    }

    impl StreamToSampleInThePairing {
        fn new_pool_file_creation() -> Self {
            let stream = prepare_stream(
                StreamName::NewPoolFileCreation,
                RunSwitches::default(),
                true,
            );
            let tables = crash_state_enumeration_tables(
                &stream.base,
                &stream.writes,
                &stream.segments,
                &full_expansion,
            );
            let facts = extract_flow_facts(
                &stream.base,
                &stream.writes,
                &stream.segments,
                &full_expansion,
            );
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
            let mut sampled = sample_ordinals(
                &tables.state_ranges_by_segment,
                tables.state_count,
                &segment_has_a_torn_write,
                &|ordinal| {
                    landing_vector_of(
                        &facts.persisted_writes_of_state(ordinal),
                        stream.writes.len(),
                        &torn_image_index_by_write,
                    )
                },
            );
            for batch in &mut sampled.batches {
                *batch = batch
                    .iter()
                    .rev()
                    .copied()
                    .take(SAMPLED_STATES_OF_EACH_BATCH_IN_THE_PAIRING)
                    .collect();
            }
            Self {
                stream,
                tables,
                facts,
                torn_image_index_by_write,
                sampled,
            }
        }

        fn observations_on(
            &self,
            threads: NonZeroUsize,
            pool_checker_table: &PoolCheckerTableOfASampledState<'_>,
        ) -> Result<Vec<StateObservation>, ObservationsUpToACrashInAState> {
            observe_sampled_states(
                &self.stream,
                &self.tables,
                &self.facts,
                &self.torn_image_index_by_write,
                &self.sampled,
                RunSwitches::default(),
                threads,
                pool_checker_table,
            )
        }

        /// 录入段：取样、O2 要判的盘录进 `tables`（产品的录法 [`pool_checker_table_of_a_sampled_state`]）；实现自崩交回那一句。
        fn observations_recorded_in(
            &self,
            threads: NonZeroUsize,
            tables: &dyn PoolCheckerTablesOfTheTwoStages,
        ) -> Result<Vec<StateObservation>, String> {
            self.observations_on(
                threads,
                &|stream_name, ordinal, image_digest, image, pool_checker_control| {
                    pool_checker_table_of_a_sampled_state(
                        tables,
                        stream_name,
                        ordinal,
                        image_digest,
                        image,
                        pool_checker_control,
                    )
                },
            )
            .map_err(|crashed| crashed.crash)
        }

        /// 录入段录完、判段判完的观测（判段与录入段同样开 `threads` 条线程）。
        fn observations_recorded_and_judged_in(
            &self,
            threads: NonZeroUsize,
            tables: &dyn PoolCheckerTablesOfTheTwoStages,
        ) -> Vec<StateObservation> {
            let mut observations = self
                .observations_recorded_in(threads, tables)
                .expect("这条流上实现不 panic");
            match judge_the_recorded_pool_checker_tables(
                &mut observations,
                tables,
                threads,
                RunSwitches::default(),
            ) {
                Ok(JudgedTablesOfAStream::AllJudged) => observations,
                Ok(JudgedTablesOfAStream::EngineFailedInAState { what }) => {
                    panic!("这条流上判器不失败：{what}")
                }
                Err(refusal) => panic!("录下的盘都读得回：{refusal}"),
            }
        }

        /// 这一份观测写出的每个文件：名字与去掉耗时之后的摘要（`E7RESULT name=file` 行报的那一个，`states.tsv`、`oracle-verdicts.tsv` 的行序进摘要）。
        fn files_written_from(
            &self,
            observations: &[StateObservation],
            threads: NonZeroUsize,
        ) -> Vec<(String, String)> {
            let directory = std::env::temp_dir().join(format!(
                "e171-sampled-states-pairing-{}-{threads}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            std::fs::create_dir_all(&directory)
                .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
            let files = write_stream_files(
                &directory,
                &self.stream,
                &self.tables,
                observations,
                self.tables.state_count,
                0,
            );
            std::fs::remove_dir_all(&directory)
                .unwrap_or_else(|error| panic!("删 {}：{error}", directory.display()));
            files
                .into_iter()
                .map(|file| (file.name, file.sha256_without_timing.unwrap_or(file.sha256)))
                .collect()
        }

        /// 一条线程与几条线程各录、各判一遍：观测按（批号、序号）排好，写出的文件逐个相同。
        fn assert_several_threads_write_the_same_files_as_one_thread(
            &self,
            tables: &dyn PoolCheckerTablesOfTheTwoStages,
        ) {
            let on_one_thread = self.observations_recorded_and_judged_in(NonZeroUsize::MIN, tables);
            let on_several_threads =
                self.observations_recorded_and_judged_in(SEVERAL_THREADS, tables);
            for observations in [&on_one_thread, &on_several_threads] {
                assert!(
                    observations
                        .windows(2)
                        .all(|pair| (pair[0].batch, pair[0].ordinal)
                            < (pair[1].batch, pair[1].ordinal)),
                    "观测按（批号、序号）严格升序"
                );
            }
            assert_eq!(
                self.files_written_from(&on_several_threads, SEVERAL_THREADS),
                self.files_written_from(&on_one_thread, NonZeroUsize::MIN),
                "几条线程与一条线程写出的文件（去掉耗时之后）逐个相同"
            );
        }
    }

    #[test]
    fn sampled_states_on_several_threads_write_the_same_files_as_on_one_thread() {
        StreamToSampleInThePairing::new_pool_file_creation()
            .assert_several_threads_write_the_same_files_as_one_thread(
                &PoolCheckerTablesKeptInMemory::judging_with(
                    &pool_checker_red_names_of_a_given_image_on_the_cpu,
                ),
            );
    }

    /// 同一条对拍走产品路径：每态一张给定状态表录进判定库、判段交引擎，表名与表的内容与产品路径相同。
    #[cfg(feature = "gpu")]
    #[test]
    fn sampled_states_through_the_given_states_engine_on_several_threads_write_the_same_files_as_on_one_thread(
    ) {
        StreamToSampleInThePairing::new_pool_file_creation()
            .assert_several_threads_write_the_same_files_as_one_thread(
                &GivenStatesTablesInTheLibrary,
            );
    }

    /// 录入段取样：O2 开着时每个取样态录下一张盘、一张都不判（判归判段）。
    #[test]
    fn recording_the_sampled_states_records_a_table_for_every_state_and_judges_none() {
        let stream_to_sample = StreamToSampleInThePairing::new_pool_file_creation();
        let tables = PoolCheckerTablesKeptInMemory::judging_with(
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        );
        let observations = stream_to_sample
            .observations_recorded_in(SEVERAL_THREADS, &tables)
            .expect("这条流上实现不 panic");
        assert!(!observations.is_empty(), "取样取到了态");
        assert_eq!(tables.tables_judged(), 0, "录入段一张盘都不判");
        assert_eq!(
            tables.tables_recorded(),
            observations.len(),
            "每个取样态录下一张盘"
        );
        assert!(
            observations.iter().all(|observation| matches!(
                observation.pool_checker,
                PoolCheckerOfAState::RecordedForTheJudgeStage(_)
            )),
            "录入段交出的观测里 O2 都是录下了、没判"
        );
    }

    #[test]
    fn panic_in_a_state_reports_the_same_text_on_several_threads_as_on_one_thread() {
        const PANIC_MESSAGE: &str = "线程对拍里造的实现自崩";
        let stream_to_sample = StreamToSampleInThePairing::new_pool_file_creation();
        let sampled_in_order: Vec<u64> = stream_to_sample
            .sampled
            .batches
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<u64>>()
            .into_iter()
            .collect();
        let position_of_the_panic = sampled_in_order.len() / 2;
        let ordinal_of_the_panic = sampled_in_order[position_of_the_panic];
        let panicking_at_that_state: &PoolCheckerTableOfASampledState<'_> =
            &move |_stream_name, ordinal, _image_digest, _image, _pool_checker_control| {
                assert_ne!(ordinal, ordinal_of_the_panic, "{PANIC_MESSAGE}");
                RecordedTableReference {
                    table_name: String::new(),
                    digest: [0; DIGEST_BYTES],
                }
            };
        let expected_ending = format!(
            ":after_ordinal_{}",
            sampled_in_order[position_of_the_panic - 1]
        );
        for threads in [NonZeroUsize::MIN, SEVERAL_THREADS] {
            let refusal = stream_to_sample
                .observations_on(threads, panicking_at_that_state)
                .err()
                .expect("那一态上 panic，这一趟要交回 Err")
                .crash;
            assert!(
                refusal.contains(PANIC_MESSAGE) && refusal.ends_with(&expected_ending),
                "{threads} 条线程：{refusal}，要以 {expected_ending} 收尾"
            );
        }
    }

    #[test]
    fn the_sampling_threads_argument_reaches_the_known_bad_images_mode() {
        let arguments = [
            "known-bad-images",
            "--dump-directory",
            "导出目录",
            "--output-directory",
            "目录",
            "--sampling-threads",
            "3",
        ]
        .map(str::to_string);
        let Some(ParsedCommand::KnownBadImages(command_line)) = parse_command_line(&arguments)
        else {
            panic!("known-bad-images 带 --sampling-threads 3 要认");
        };
        assert_eq!(
            command_line.switches.sampling_threads,
            NonZeroUsize::new(3),
            "坏镜像模式的线程数取同一个参数"
        );
    }

    /// 不上卡的给定盘 O2 名单：同一张盘直接判（线程对拍比的是分线程，不是两段；两段的那一条在带 gpu 特性的对拍里）。
    // gpu-exempt: experiment-measures-the-cpu-judge-itself
    fn pool_checker_red_names_of_a_given_image_on_the_cpu(
        _table_name: &str,
        pool: &MemoryPool,
    ) -> Vec<String> {
        violated_invariant_names(&check_pool_image(pool))
    }

    /// 造出来的 E170 坏状态每条流各几行带补丁（取已知坏镜像表开头几条的改法）。
    const PATCHED_E170_BAD_ROWS_OF_EACH_STREAM: usize = 4;

    /// 造出来的 E170 坏状态每条流各几行不带补丁、落在同一张没落任何写的底态上（几条线程同时判到同一张底态）。
    const UNPATCHED_E170_BAD_ROWS_OF_EACH_STREAM: usize = 2;

    /// 第 `row_position` 行造出来的 E170 坏状态的行号。
    fn e170_bad_row_identifier(row_position: usize) -> String {
        format!("row-{row_position}")
    }

    /// `known-bad-images` 模式走一趟留下的：退出码或 panic 的消息、结果行的全部字节、输出目录里每个文件的全部字节。
    #[derive(Debug, PartialEq, Eq)]
    struct KnownBadImagesModeOutcome {
        ending: Result<i32, String>,
        result_lines: String,
        files: BTreeMap<String, Vec<u8>>,
    }

    /// 临时目录里一份 `known-bad-images` 模式要读的导出：两条流的目录都放新池新建文件流的中性格式（模式只按落地向量重建，不看它是哪条流），
    /// `states.tsv` 一行全持久态，E170 坏状态由已知坏镜像表开头几条的补丁与不带补丁的几行组成，每行的两个摘要与 E170 池级 checker 那一列由这里现算。
    struct DumpDirectoryForKnownBadImages {
        directory: PathBuf,
        known_bad_entry_count: usize,
    }

    impl DumpDirectoryForKnownBadImages {
        fn new(scenario: &str) -> Self {
            let directory = std::env::temp_dir().join(format!(
                "e171-known-bad-images-{scenario}-{}",
                std::process::id()
            ));
            let stream = prepare_stream(
                StreamName::NewPoolFileCreation,
                RunSwitches::default(),
                true,
            );
            let tables = crash_state_enumeration_tables(
                &stream.base,
                &stream.writes,
                &stream.segments,
                &full_expansion,
            );
            let mut known_bad_entry_count = 0;
            for name in STREAM_NAMES {
                let stream_directory = directory.join(name.directory_name());
                std::fs::create_dir_all(&stream_directory)
                    .unwrap_or_else(|error| panic!("建 {}：{error}", stream_directory.display()));
                write_stream_files(
                    &stream_directory,
                    &stream,
                    &tables,
                    &[],
                    tables.state_count,
                    0,
                );
                known_bad_entry_count =
                    Self::write_the_full_state_and_the_e170_bad_rows(&stream_directory);
            }
            Self {
                directory,
                known_bad_entry_count,
            }
        }

        /// 交回已知坏镜像表的条数。
        fn write_the_full_state_and_the_e170_bad_rows(stream_directory: &Path) -> usize {
            let neutral = read_neutral_stream(stream_directory);
            let write_count = neutral
                .operations
                .iter()
                .filter(|operation| operation.kind != RecordedOperationKind::Barrier)
                .count();
            let all_persisted = "2".repeat(write_count);
            let none_landed = "0".repeat(write_count);
            let clean = rebuilt_memory_pool(&neutral, &all_persisted);
            std::fs::write(
                stream_directory.join(STATES_FILE),
                format!(
                    "0\t0\t0\t1\t{all_persisted}\t{}\n",
                    sha256_text(&memory_pool_digest(&clean))
                ),
            )
            .unwrap_or_else(|error| panic!("写 {STATES_FILE}：{error}"));
            let entries = known_bad_images(&clean);
            let patches_of_the_entry = |entry_index: usize| {
                let mut bad = clean.clone();
                (entries[entry_index].1)(&mut bad);
                sector_difference_as_patches(&clean, &bad)
            };
            let rows: Vec<(&str, Vec<BytePatch>)> = (0..PATCHED_E170_BAD_ROWS_OF_EACH_STREAM)
                .map(|entry_index| (all_persisted.as_str(), patches_of_the_entry(entry_index)))
                .chain(
                    (0..UNPATCHED_E170_BAD_ROWS_OF_EACH_STREAM)
                        .map(|_row| (none_landed.as_str(), Vec::new())),
                )
                .collect();
            let mut states_text = String::new();
            let mut verdicts_text = String::new();
            let mut patches_file = Vec::new();
            for (row_position, (landing_vector, patches)) in rows.iter().enumerate() {
                let mut pool = rebuilt_memory_pool(&neutral, landing_vector);
                let base_digest = sha256_text(&memory_pool_digest(&pool));
                apply_patches(&mut pool, patches);
                let row_identifier = e170_bad_row_identifier(row_position);
                states_text.push_str(&format!(
                    "{row_identifier}\t{landing_vector}\t{base_digest}\t{}\t{}\n",
                    sha256_text(&memory_pool_digest(&pool)),
                    patch_list_text(patches, &mut patches_file)
                ));
                let mut verdict_fields = vec!["-".to_string(); 10];
                verdict_fields[6] = list_text(&pool_checker_red_names_of_a_given_image_on_the_cpu(
                    &row_identifier,
                    &pool,
                ));
                verdicts_text.push_str(&verdict_fields.join("\t"));
                verdicts_text.push('\n');
            }
            for (file_name, contents) in [
                ("bad-states.tsv", states_text.into_bytes()),
                ("bad-patches.bin", patches_file),
                ("bad-impl-verdicts.tsv", verdicts_text.into_bytes()),
            ] {
                std::fs::write(stream_directory.join(file_name), contents)
                    .unwrap_or_else(|error| panic!("写 {file_name}：{error}"));
            }
            entries.len()
        }

        fn command_line_on(
            &self,
            threads: NonZeroUsize,
            output_directory: PathBuf,
        ) -> KnownBadImagesCommandLine {
            KnownBadImagesCommandLine {
                dump_directory: self.directory.clone(),
                output_directory,
                with_e170_bad_rows: true,
                switches: RunSwitches {
                    sampling_threads: Some(threads),
                    ..RunSwitches::default()
                },
            }
        }

        /// 两段先后跑一趟，O2 两段交内存里的替身（判交给 `pool_checker_red_names`）。
        fn run_on(
            &self,
            threads: NonZeroUsize,
            pool_checker_red_names: &PoolCheckerRedNamesOfAGivenImage<'_>,
        ) -> KnownBadImagesModeOutcome {
            self.run_on_with_the_tables(
                threads,
                &PoolCheckerTablesKeptInMemory::judging_with(pool_checker_red_names),
            )
        }

        fn run_on_with_the_tables(
            &self,
            threads: NonZeroUsize,
            tables: &dyn PoolCheckerTablesOfTheTwoStages,
        ) -> KnownBadImagesModeOutcome {
            let output_directory = self.directory.join(format!("output-on-{threads}-threads"));
            let KnownBadImagesModeOutcome {
                ending,
                result_lines,
                ..
            } = self.run_stage_on(
                RunStage::RecordThenJudge,
                threads,
                &output_directory,
                tables,
            );
            KnownBadImagesModeOutcome {
                ending,
                result_lines,
                files: Self::read_and_remove(&output_directory),
            }
        }

        /// 照 `stage` 跑一趟，交回退出码（或 panic 的消息、拒绝的那一句）与结果行；输出目录留着（[`Self::read_and_remove`] 读）。
        fn run_stage_on(
            &self,
            stage: RunStage,
            threads: NonZeroUsize,
            output_directory: &Path,
            tables: &dyn PoolCheckerTablesOfTheTwoStages,
        ) -> KnownBadImagesModeOutcome {
            let command_line = self.command_line_on(threads, output_directory.to_path_buf());
            let mut result_lines = Vec::new();
            let ending = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_known_bad_images(
                    &command_line,
                    stage,
                    &KNOWN_BAD_IMAGES_RUN_ARGUMENTS.map(str::to_string),
                    tables,
                    &mut result_lines,
                )
            }))
            .map_err(|payload| panic_text(&*payload))
            .and_then(|ran| ran);
            KnownBadImagesModeOutcome {
                ending,
                result_lines: String::from_utf8(result_lines).expect("结果行是 UTF-8"),
                files: BTreeMap::new(),
            }
        }

        /// 输出目录里每个文件的全部字节（没有这个目录是空的），读完删掉这个目录。
        fn read_and_remove(output_directory: &Path) -> BTreeMap<String, Vec<u8>> {
            let mut files = BTreeMap::new();
            if output_directory.exists() {
                for entry in std::fs::read_dir(output_directory)
                    .unwrap_or_else(|error| panic!("读 {}：{error}", output_directory.display()))
                {
                    let path = entry.expect("读得了输出目录的一项").path();
                    let contents = std::fs::read(&path)
                        .unwrap_or_else(|error| panic!("读 {}：{error}", path.display()));
                    files.insert(
                        path.file_name()
                            .expect("输出目录里的文件有名字")
                            .to_string_lossy()
                            .into_owned(),
                        contents,
                    );
                }
                std::fs::remove_dir_all(output_directory)
                    .unwrap_or_else(|error| panic!("删 {}：{error}", output_directory.display()));
            }
            files
        }
    }

    impl Drop for DumpDirectoryForKnownBadImages {
        fn drop(&mut self) {
            if let Err(error) = std::fs::remove_dir_all(&self.directory) {
                eprintln!("删 {}：{error}", self.directory.display());
            }
        }
    }

    /// 一条线程与几条线程各走一趟模式：停机条款都过，结果行的字节与产物文件的字节逐个相同。
    fn assert_the_known_bad_images_mode_leaves_the_same_on_several_threads_as_on_one_thread(
        scenario: &str,
        tables: &dyn PoolCheckerTablesOfTheTwoStages,
    ) {
        let dump = DumpDirectoryForKnownBadImages::new(scenario);
        let on_one_thread = dump.run_on_with_the_tables(NonZeroUsize::MIN, tables);
        let on_several_threads = dump.run_on_with_the_tables(SEVERAL_THREADS, tables);
        assert_eq!(
            on_one_thread.ending,
            Ok(0),
            "停机条款都过：{}",
            on_one_thread.result_lines
        );
        assert_eq!(
            on_one_thread
                .result_lines
                .lines()
                .filter(|line| line.contains("name=known_bad_entry "))
                .count(),
            dump.known_bad_entry_count,
            "每条已知坏镜像一行结果行：{}",
            on_one_thread.result_lines
        );
        assert_eq!(
            on_one_thread
                .result_lines
                .lines()
                .filter(|line| line.contains("name=e170_bad_row "))
                .count(),
            (PATCHED_E170_BAD_ROWS_OF_EACH_STREAM + UNPATCHED_E170_BAD_ROWS_OF_EACH_STREAM)
                * STREAM_NAMES.len(),
            "每行 E170 坏状态一行结果行：{}",
            on_one_thread.result_lines
        );
        assert_eq!(
            on_one_thread.files.keys().collect::<Vec<&String>>(),
            [
                "e170-bad-rows-o2.tsv",
                "known-bad-o2.tsv",
                "known-bad-patches.bin",
                "known-bad-states.tsv"
            ]
            .map(str::to_string)
            .iter()
            .collect::<Vec<&String>>(),
            "这个模式写的四个产物文件"
        );
        assert!(
            on_one_thread.result_lines.ends_with(&format!(
                "E7RESULT name=done emitted={}\n",
                on_one_thread.result_lines.lines().count()
            )),
            "收尾行写进同一份输出、数得对：{}",
            on_one_thread.result_lines
        );
        assert_eq!(
            on_several_threads, on_one_thread,
            "几条线程与一条线程的退出码、结果行、产物文件逐字相同"
        );
    }

    #[test]
    fn known_bad_images_mode_on_several_threads_prints_and_writes_the_same_as_on_one_thread() {
        assert_the_known_bad_images_mode_leaves_the_same_on_several_threads_as_on_one_thread(
            "cpu-pairing",
            &PoolCheckerTablesKeptInMemory::judging_with(
                &pool_checker_red_names_of_a_given_image_on_the_cpu,
            ),
        );
    }

    /// 同一条对拍走产品路径：每张盘一张给定状态表录进判定库、判段交引擎，表名与表的内容与产品路径相同。
    #[cfg(feature = "gpu")]
    #[test]
    fn known_bad_images_mode_through_the_given_states_engine_on_several_threads_prints_and_writes_the_same_as_on_one_thread(
    ) {
        assert_the_known_bad_images_mode_leaves_the_same_on_several_threads_as_on_one_thread(
            "engine-pairing",
            &GivenStatesTablesInTheLibrary,
        );
    }

    #[test]
    fn judged_tables_keep_their_names_on_several_threads() {
        let dump = DumpDirectoryForKnownBadImages::new("table-names");
        let judged_table_names = Mutex::new(Vec::new());
        let outcome = dump.run_on(SEVERAL_THREADS, &|table_name, pool| {
            judged_table_names
                .lock()
                .expect("这把锁里不 panic")
                .push(table_name.to_string());
            pool_checker_red_names_of_a_given_image_on_the_cpu(table_name, pool)
        });
        assert_eq!(outcome.ending, Ok(0));
        let mut expected_table_names = vec!["known-bad/clean".to_string()];
        for entry_index in 0..dump.known_bad_entry_count {
            expected_table_names.push(format!("known-bad/entry-{entry_index}/direct"));
            expected_table_names.push(format!("known-bad/entry-{entry_index}/patched"));
        }
        for name in STREAM_NAMES {
            for row_position in
                0..PATCHED_E170_BAD_ROWS_OF_EACH_STREAM + UNPATCHED_E170_BAD_ROWS_OF_EACH_STREAM
            {
                let row_identifier = e170_bad_row_identifier(row_position);
                let label = name.directory_name();
                expected_table_names.push(format!("e170-bad-row/{label}/{row_identifier}/base"));
                expected_table_names.push(format!("e170-bad-row/{label}/{row_identifier}/patched"));
            }
        }
        expected_table_names.sort_unstable();
        let mut judged_table_names = judged_table_names.into_inner().expect("这把锁里不 panic");
        judged_table_names.sort_unstable();
        assert_eq!(
            judged_table_names, expected_table_names,
            "每张表判一次、表名不随线程变（表名进录入摘要，判定库的键不变）"
        );
    }

    /// 同时在判的调用要等到几次才算「同时」。
    const SIMULTANEOUS_JUDGE_CALLS_LOOKED_FOR: usize = 2;

    /// 表名以这个前缀起头的判定，等多久等不到另一次同时的调用就不再等（一条线程走完那一趟只等这一回）。
    const SECONDS_A_JUDGE_CALL_WAITS_FOR_ANOTHER: u64 = 30;

    /// 同时在判的调用数：`in_flight` 是此刻几次，`most_in_flight` 是到此刻最多几次，`gave_up` 是等过一回没等到。
    #[derive(Default)]
    struct JudgeCallsInFlight {
        in_flight: usize,
        most_in_flight: usize,
        gave_up: bool,
    }

    /// 让表名带某个前缀的判定互相等到有 [`SIMULTANEOUS_JUDGE_CALLS_LOOKED_FOR`] 次同时在判：几条线程时必等得到，一条线程时等够时限一回就不再等。
    struct RendezvousOfJudgeCalls {
        table_name_prefix: &'static str,
        calls: Mutex<JudgeCallsInFlight>,
        one_arrived: std::sync::Condvar,
    }

    impl RendezvousOfJudgeCalls {
        fn new(table_name_prefix: &'static str) -> Self {
            Self {
                table_name_prefix,
                calls: Mutex::new(JudgeCallsInFlight::default()),
                one_arrived: std::sync::Condvar::new(),
            }
        }

        fn judge(&self, table_name: &str, pool: &MemoryPool) -> Vec<String> {
            if table_name.starts_with(self.table_name_prefix) {
                let deadline = Instant::now()
                    + std::time::Duration::from_secs(SECONDS_A_JUDGE_CALL_WAITS_FOR_ANOTHER);
                let mut calls = self.calls.lock().expect("这把锁里不 panic");
                calls.in_flight += 1;
                calls.most_in_flight = calls.most_in_flight.max(calls.in_flight);
                self.one_arrived.notify_all();
                while calls.most_in_flight < SIMULTANEOUS_JUDGE_CALLS_LOOKED_FOR && !calls.gave_up {
                    let Some(time_left) = deadline.checked_duration_since(Instant::now()) else {
                        calls.gave_up = true;
                        break;
                    };
                    calls = self
                        .one_arrived
                        .wait_timeout(calls, time_left)
                        .expect("这把锁里不 panic")
                        .0;
                }
                calls.in_flight -= 1;
            }
            pool_checker_red_names_of_a_given_image_on_the_cpu(table_name, pool)
        }

        fn most_in_flight(&self) -> usize {
            self.calls.lock().expect("这把锁里不 panic").most_in_flight
        }
    }

    /// 带 `--sampling-threads` 走一趟模式（线程数照产品路径 `sampling_threads_of_this_run` 取），表名带这个前缀的判定要有几次同时在判。
    fn assert_the_tables_with_this_prefix_are_judged_at_once(
        scenario: &str,
        table_name_prefix: &'static str,
    ) {
        let dump = DumpDirectoryForKnownBadImages::new(scenario);
        let rendezvous = RendezvousOfJudgeCalls::new(table_name_prefix);
        let outcome = dump.run_on(SEVERAL_THREADS, &|table_name, pool| {
            rendezvous.judge(table_name, pool)
        });
        assert_eq!(outcome.ending, Ok(0));
        let threads_of_this_run = sampling_threads_of_this_run(RunSwitches {
            sampling_threads: Some(SEVERAL_THREADS),
            ..RunSwitches::default()
        });
        let expected_at_once = SIMULTANEOUS_JUDGE_CALLS_LOOKED_FOR.min(threads_of_this_run.get());
        assert!(
            rendezvous.most_in_flight() >= expected_at_once,
            "{table_name_prefix} 的表最多 {} 次同时在判，这一趟开 {threads_of_this_run} 条线程，至少要 {expected_at_once} 次",
            rendezvous.most_in_flight()
        );
    }

    #[test]
    fn known_bad_entries_are_judged_on_several_threads_at_once() {
        assert_the_tables_with_this_prefix_are_judged_at_once(
            "entries-at-once",
            "known-bad/entry-",
        );
    }

    #[test]
    fn e170_bad_rows_are_judged_on_several_threads_at_once() {
        assert_the_tables_with_this_prefix_are_judged_at_once("rows-at-once", "e170-bad-row/");
    }

    const PANIC_MESSAGE_OF_THE_JUDGE: &str = "坏镜像线程对拍里造的判器崩";

    /// 走到这张表就 panic 的判器，别的表照判。
    fn judge_panicking_at<'table>(
        panicking_table_name: &'table str,
    ) -> impl Fn(&str, &MemoryPool) -> Vec<String> + Sync + 'table {
        move |table_name, pool| {
            assert_ne!(
                table_name, panicking_table_name,
                "{PANIC_MESSAGE_OF_THE_JUDGE}"
            );
            pool_checker_red_names_of_a_given_image_on_the_cpu(table_name, pool)
        }
    }

    #[test]
    fn panic_on_a_known_bad_entry_prints_the_same_lines_on_several_threads_as_on_one_thread() {
        const ENTRY_OF_THE_PANIC: usize = 3;
        let dump = DumpDirectoryForKnownBadImages::new("known-bad-entry-panic");
        let panicking_table_name = format!("known-bad/entry-{ENTRY_OF_THE_PANIC}/patched");
        let judge = judge_panicking_at(&panicking_table_name);
        let on_one_thread = dump.run_on(NonZeroUsize::MIN, &judge);
        let on_several_threads = dump.run_on(SEVERAL_THREADS, &judge);
        assert!(
            on_one_thread
                .ending
                .as_ref()
                .is_err_and(|text| text.contains(PANIC_MESSAGE_OF_THE_JUDGE)),
            "判器的 panic 原样抛出来：{:?}",
            on_one_thread.ending
        );
        assert!(
            on_one_thread.result_lines.contains(&format!(
                "name=known_bad_entry index={} ",
                ENTRY_OF_THE_PANIC - 1
            )) && !on_one_thread
                .result_lines
                .contains(&format!("name=known_bad_entry index={ENTRY_OF_THE_PANIC} ")),
            "结果行打到 panic 的前一条为止：{}",
            on_one_thread.result_lines
        );
        assert!(on_one_thread.files.is_empty(), "产物在判完之后才写");
        assert_eq!(
            on_several_threads, on_one_thread,
            "几条线程与一条线程的 panic 消息、结果行、产物文件逐字相同"
        );
    }

    #[test]
    fn panic_on_an_e170_bad_row_prints_and_writes_the_same_on_several_threads_as_on_one_thread() {
        const ROW_OF_THE_PANIC: usize = 2;
        let dump = DumpDirectoryForKnownBadImages::new("e170-bad-row-panic");
        let label = StreamName::NewPoolFileCreation.directory_name();
        let row_identifier = e170_bad_row_identifier(ROW_OF_THE_PANIC);
        let panicking_table_name = format!("e170-bad-row/{label}/{row_identifier}/base");
        let judge = judge_panicking_at(&panicking_table_name);
        let on_one_thread = dump.run_on(NonZeroUsize::MIN, &judge);
        let on_several_threads = dump.run_on(SEVERAL_THREADS, &judge);
        assert!(
            on_one_thread
                .ending
                .as_ref()
                .is_err_and(|text| text.contains(PANIC_MESSAGE_OF_THE_JUDGE)),
            "判器的 panic 原样抛出来：{:?}",
            on_one_thread.ending
        );
        let row_line_start = |row_position: usize| {
            format!(
                "name=e170_bad_row stream={label} row={} ",
                e170_bad_row_identifier(row_position)
            )
        };
        assert!(
            on_one_thread
                .result_lines
                .contains(&row_line_start(ROW_OF_THE_PANIC - 1))
                && !on_one_thread
                    .result_lines
                    .contains(&row_line_start(ROW_OF_THE_PANIC)),
            "结果行打到 panic 的前一行为止：{}",
            on_one_thread.result_lines
        );
        assert_eq!(
            on_one_thread.files.keys().collect::<Vec<&String>>(),
            [
                "known-bad-o2.tsv",
                "known-bad-patches.bin",
                "known-bad-states.tsv"
            ]
            .map(str::to_string)
            .iter()
            .collect::<Vec<&String>>(),
            "已知坏镜像的产物已经写了，E170 坏状态那一份在判完之后才写"
        );
        assert_eq!(
            on_several_threads, on_one_thread,
            "几条线程与一条线程的 panic 消息、结果行、产物文件逐字相同"
        );
    }

    #[test]
    fn no_known_bad_entry_after_a_panicking_one_is_judged_on_one_thread() {
        const ENTRY_OF_THE_PANIC: usize = 3;
        let dump = DumpDirectoryForKnownBadImages::new("no-entry-after-the-panic");
        let panicking_table_name = format!("known-bad/entry-{ENTRY_OF_THE_PANIC}/direct");
        let judge = judge_panicking_at(&panicking_table_name);
        let judged_table_names = Mutex::new(Vec::new());
        let outcome = dump.run_on(NonZeroUsize::MIN, &|table_name, pool| {
            judged_table_names
                .lock()
                .expect("这把锁里不 panic")
                .push(table_name.to_string());
            judge(table_name, pool)
        });
        assert!(outcome.ending.is_err(), "判器 panic 了这一趟要 panic");
        assert_eq!(
            judged_table_names
                .into_inner()
                .expect("这把锁里不 panic")
                .last(),
            Some(&panicking_table_name),
            "panic 的那张表之后一张都不再判"
        );
    }

    /// 坏镜像模式这几条单测的参数（进交接；两段核是不是同一条命令）。
    const KNOWN_BAD_IMAGES_RUN_ARGUMENTS: [&str; 1] = ["known-bad-images"];

    /// 只跑录入段：每张要判的盘都录下、一张都不判，不打结果行，输出目录里只留交接。
    #[test]
    fn the_record_stage_of_the_known_bad_images_mode_judges_no_table_and_leaves_only_the_hand_off()
    {
        let dump = DumpDirectoryForKnownBadImages::new("record-stage-only");
        let tables = PoolCheckerTablesKeptInMemory::judging_with(
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        );
        let output_directory = dump.directory.join("output-of-the-record-stage");
        let recorded = dump.run_stage_on(
            RunStage::RecordOnly,
            SEVERAL_THREADS,
            &output_directory,
            &tables,
        );
        let files = DumpDirectoryForKnownBadImages::read_and_remove(&output_directory);
        assert_eq!(recorded.ending, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
        assert_eq!(tables.tables_judged(), 0, "录入段一张盘都不判");
        let e170_bad_rows = (PATCHED_E170_BAD_ROWS_OF_EACH_STREAM
            + UNPATCHED_E170_BAD_ROWS_OF_EACH_STREAM)
            * STREAM_NAMES.len();
        assert_eq!(
            tables.tables_recorded(),
            1 + 2 * dump.known_bad_entry_count + 2 * e170_bad_rows,
            "干净镜像一张、已知坏镜像每条两张、E170 坏状态每行两张都录下"
        );
        assert!(
            recorded.result_lines.is_empty(),
            "录入段不打结果行：{}",
            recorded.result_lines
        );
        assert_eq!(
            files.keys().collect::<Vec<&String>>(),
            vec![&HAND_OFF_TO_THE_JUDGE_STAGE_FILE.to_string()],
            "录入段在输出目录里只留交接"
        );
    }

    /// 分两趟跑（录入段一趟、判段一趟）与两段先后跑一趟：退出码、结果行、产物文件逐字节相同（判完删掉交接）；
    /// 判段一张盘都不再录（不再重建、不再驱动），录下的每张盘判段判一次。
    #[test]
    fn the_known_bad_images_mode_in_two_runs_prints_and_writes_the_same_as_in_one_run() {
        let dump = DumpDirectoryForKnownBadImages::new("two-runs");
        let in_one_run = dump.run_on(
            SEVERAL_THREADS,
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        );
        let tables = PoolCheckerTablesKeptInMemory::judging_with(
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        );
        let output_directory = dump.directory.join("output-in-two-runs");
        let recorded = dump.run_stage_on(
            RunStage::RecordOnly,
            SEVERAL_THREADS,
            &output_directory,
            &tables,
        );
        assert_eq!(recorded.ending, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
        let tables_recorded_by_the_record_stage = tables.tables_recorded();
        let judged = dump.run_stage_on(
            RunStage::JudgeOnly,
            SEVERAL_THREADS,
            &output_directory,
            &tables,
        );
        let in_two_runs = KnownBadImagesModeOutcome {
            ending: judged.ending,
            result_lines: judged.result_lines,
            files: DumpDirectoryForKnownBadImages::read_and_remove(&output_directory),
        };
        assert_eq!(
            tables.tables_recorded(),
            tables_recorded_by_the_record_stage,
            "判段一张盘都不再录：不重跑流程"
        );
        assert_eq!(
            tables.tables_judged(),
            tables_recorded_by_the_record_stage,
            "录下的每张盘判段判一次"
        );
        assert_eq!(
            in_one_run.ending,
            Ok(0),
            "停机条款都过：{}",
            in_one_run.result_lines
        );
        assert_eq!(
            in_two_runs, in_one_run,
            "分两趟跑与两段先后跑一趟的退出码、结果行、产物文件逐字节相同"
        );
    }

    /// 只跑判段而录入段没跑过：拒，报错句点名先跑录入段，一张盘都不录、不判，不打结果行。
    #[test]
    fn the_judge_stage_without_a_record_stage_refuses_and_names_the_record_stage() {
        let dump = DumpDirectoryForKnownBadImages::new("judge-without-record");
        let tables = PoolCheckerTablesKeptInMemory::judging_with(
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        );
        let output_directory = dump.directory.join("output-never-recorded");
        let judged = dump.run_stage_on(
            RunStage::JudgeOnly,
            SEVERAL_THREADS,
            &output_directory,
            &tables,
        );
        let refusal = judged.ending.expect_err("录入段没跑过就只跑判段，要拒");
        assert!(
            refusal.contains("先跑录入段") && refusal.contains(&format!("{STAGE_ARGUMENT} record")),
            "报错句点名先跑录入段：{refusal}"
        );
        assert_eq!(tables.tables_recorded(), 0, "判段不补录");
        assert_eq!(tables.tables_judged(), 0, "没有交接就一张都不判");
        assert!(
            judged.result_lines.is_empty(),
            "拒了就不打结果行：{}",
            judged.result_lines
        );
    }

    /// 判段的参数与录入段那一趟的不同（两趟只许换 `--stage`）：拒，点名两趟的参数，不判。
    #[test]
    fn the_judge_stage_refuses_a_hand_off_recorded_under_other_arguments() {
        let output_directory = std::env::temp_dir().join(format!(
            "e171-hand-off-under-other-arguments-{}",
            std::process::id()
        ));
        let recorded = run_in_stages(
            RunStage::RecordOnly,
            &["row".to_string()],
            &output_directory,
            || "录入段交出的".to_string(),
            |_hand_off: String| -> Result<i32, String> { panic!("只跑录入段不判") },
        );
        let hand_offs_judged = AtomicUsize::new(0);
        let judged = run_in_stages(
            RunStage::JudgeOnly,
            &["baseline".to_string()],
            &output_directory,
            || -> String { panic!("只跑判段不录") },
            |_hand_off: String| {
                hand_offs_judged.fetch_add(1, Ordering::SeqCst);
                Ok(0)
            },
        );
        std::fs::remove_dir_all(&output_directory)
            .unwrap_or_else(|error| panic!("删 {}：{error}", output_directory.display()));
        assert_eq!(recorded, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
        let refusal = judged.expect_err("参数不同要拒");
        assert!(
            refusal.contains("[\"row\"]") && refusal.contains("[\"baseline\"]"),
            "点名两趟的参数：{refusal}"
        );
        assert_eq!(hand_offs_judged.load(Ordering::SeqCst), 0, "拒了就不判");
    }

    /// 新池新建文件流的录入段（取样照 [`StreamToSampleInThePairing`] 收小，与 `export_stream` 同一种填法）：导出文件写进 `output_root`
    /// 下这条流的目录，O2 要判的盘录进 `tables`，交回 `baseline` 那一趟的交接。
    fn recorded_dump_of_the_pairing_stream(
        stream_to_sample: &StreamToSampleInThePairing,
        output_root: &Path,
        tables: &dyn PoolCheckerTablesOfTheTwoStages,
    ) -> RecordedDump {
        let started = Instant::now();
        let name = StreamName::NewPoolFileCreation;
        let directory = output_root.join(name.directory_name());
        std::fs::create_dir_all(&directory)
            .unwrap_or_else(|error| panic!("建 {}：{error}", directory.display()));
        let observations = stream_to_sample
            .observations_recorded_in(SEVERAL_THREADS, tables)
            .expect("这条流上实现不 panic");
        let stream = &stream_to_sample.stream;
        let mut export = StreamExport::not_exported(name, String::new(), started);
        export.not_exported = None;
        export.checks = stream.checks.clone();
        export.calls = stream.calls.clone();
        export.ending = Some(stream.ending.clone());
        export.model_stopped_at_call = stream.model_stopped_at_call;
        export.operation_count = stream.operations.len();
        export.write_count = stream.writes.len();
        export.segment_count = stream.segments.len();
        export.tearable_write_count = stream_to_sample.torn_image_index_by_write.len();
        export.full_states = stream_to_sample.tables.state_count;
        export.torn_scans = stream_to_sample.sampled.torn_scans.clone();
        export.files = write_stream_files_before_the_judge_stage(
            &directory,
            stream,
            &stream_to_sample.tables,
            &observations,
            stream_to_sample.tables.state_count,
            0,
        );
        export.observations = observations;
        RecordedDump {
            stream_independent_checks: stream_independent_checks(),
            exports: vec![export],
        }
    }

    /// 一个目录里每个文件（名字 → 去掉耗时之后的字节，[`contents_without_timing`]；不带耗时的原样）。
    fn files_without_timing_under(directory: &Path) -> BTreeMap<String, Vec<u8>> {
        std::fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("读 {}：{error}", directory.display()))
            .map(|entry| {
                let path = entry.expect("读得了目录的一项").path();
                let name = path
                    .file_name()
                    .expect("目录里的文件有名字")
                    .to_string_lossy()
                    .into_owned();
                let contents = std::fs::read(&path)
                    .unwrap_or_else(|error| panic!("读 {}：{error}", path.display()));
                let kept = contents_without_timing(&name, &contents).unwrap_or(contents);
                (name, kept)
            })
            .collect()
    }

    /// `baseline` 的交接：新池新建文件流（取样收小）录入段一趟、判段一趟，与两段先后跑一趟，退出码与结果行逐字节相同、判完删掉交接；
    /// 两趟写出的文件（去掉耗时之后）都与不经交接、一趟写完（`write_stream_files`）的逐个相同。
    #[test]
    fn a_dump_handed_off_to_the_judge_stage_prints_the_same_as_in_one_run_and_writes_the_same_as_without_a_hand_off(
    ) {
        let stream_to_sample = StreamToSampleInThePairing::new_pool_file_creation();
        let tables = PoolCheckerTablesKeptInMemory::judging_with(
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        );
        let label = StreamName::NewPoolFileCreation.directory_name();
        let root = std::env::temp_dir().join(format!("e171-dump-hand-off-{}", std::process::id()));
        let run_arguments = ["baseline".to_string()];
        let command_line_on = |output_directory: PathBuf| CommandLine {
            run_kind: RunKind::Baseline,
            output_directory,
            switches: RunSwitches::default(),
        };
        let in_one_run_directory = root.join("in-one-run");
        let mut in_one_run_lines = Vec::new();
        let in_one_run = run_in_stages(
            RunStage::RecordThenJudge,
            &run_arguments,
            &in_one_run_directory,
            || {
                recorded_dump_of_the_pairing_stream(
                    &stream_to_sample,
                    &in_one_run_directory,
                    &tables,
                )
            },
            |recorded_dump| {
                judge_the_dump(
                    &command_line_on(in_one_run_directory.clone()),
                    recorded_dump,
                    &tables,
                    &mut in_one_run_lines,
                )
            },
        );
        let in_two_runs_directory = root.join("in-two-runs");
        let recorded = run_in_stages(
            RunStage::RecordOnly,
            &run_arguments,
            &in_two_runs_directory,
            || {
                recorded_dump_of_the_pairing_stream(
                    &stream_to_sample,
                    &in_two_runs_directory,
                    &tables,
                )
            },
            |_recorded: RecordedDump| -> Result<i32, String> { panic!("只跑录入段不判") },
        );
        let mut in_two_runs_lines = Vec::new();
        let in_two_runs = run_in_stages(
            RunStage::JudgeOnly,
            &run_arguments,
            &in_two_runs_directory,
            || -> RecordedDump { panic!("只跑判段不再取样") },
            |recorded_dump| {
                judge_the_dump(
                    &command_line_on(in_two_runs_directory.clone()),
                    recorded_dump,
                    &tables,
                    &mut in_two_runs_lines,
                )
            },
        );
        let hand_off_is_left = in_two_runs_directory
            .join(HAND_OFF_TO_THE_JUDGE_STAGE_FILE)
            .exists();
        let without_a_hand_off_directory = root.join("without-a-hand-off");
        std::fs::create_dir_all(&without_a_hand_off_directory).unwrap_or_else(|error| {
            panic!("建 {}：{error}", without_a_hand_off_directory.display())
        });
        let observations =
            stream_to_sample.observations_recorded_and_judged_in(NonZeroUsize::MIN, &tables);
        write_stream_files(
            &without_a_hand_off_directory,
            &stream_to_sample.stream,
            &stream_to_sample.tables,
            &observations,
            stream_to_sample.tables.state_count,
            0,
        );
        let files_in_one_run = files_without_timing_under(&in_one_run_directory.join(label));
        let files_in_two_runs = files_without_timing_under(&in_two_runs_directory.join(label));
        let files_without_a_hand_off = files_without_timing_under(&without_a_hand_off_directory);
        std::fs::remove_dir_all(&root)
            .unwrap_or_else(|error| panic!("删 {}：{error}", root.display()));
        assert_eq!(recorded, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
        assert!(in_one_run.is_ok(), "两段先后跑那一趟没拒：{in_one_run:?}");
        assert_eq!(in_two_runs, in_one_run, "两趟的退出码相同");
        assert_eq!(
            String::from_utf8(in_two_runs_lines).expect("结果行是 UTF-8"),
            String::from_utf8(in_one_run_lines).expect("结果行是 UTF-8"),
            "分两趟跑与两段先后跑一趟的结果行逐字节相同"
        );
        assert!(!hand_off_is_left, "判段判完删掉交接");
        assert_eq!(
            files_in_two_runs, files_without_a_hand_off,
            "经交接分两趟写出的文件与不经交接一趟写完的逐个相同"
        );
        assert_eq!(
            files_in_one_run, files_without_a_hand_off,
            "两段先后跑写出的文件与不经交接一趟写完的逐个相同"
        );
    }

    /// 判段里引擎判一态的盘时失败的那句消息（不带空白：结果行里的 `field_text` 去掉空白）。
    const ENGINE_FAILURE_MESSAGE: &str = "两段对拍里造的引擎失败";

    /// 判段里引擎判新池新建文件流（取样照 [`StreamToSampleInThePairing`] 收小）某一态的盘时失败：分两趟跑（录入段一趟、判段一趟）、两段先后跑一趟，
    /// 与两段之前的退出码、结果行逐字节相同，这条流的目录里都只留照写的两个文件（去掉耗时之后逐字节相同）。两段之前的那一份照不分两段的 `export_stream`
    /// 在取样回调里接住实现自崩那一支的填法造：`not_exported` 是 `implementation_crashed_in_a_state:<消息>:after_ordinal_<按序号它前面那一态>`、
    /// 只写照写的两个文件、取样之后才有的几样是零；打结果行的是两段之前与之后同一份 `report_stream`。
    #[test]
    fn an_engine_failure_in_a_state_prints_the_same_in_two_runs_in_one_run_and_as_before_the_two_stages(
    ) {
        let stream_to_sample = StreamToSampleInThePairing::new_pool_file_creation();
        let sampled_in_order: Vec<u64> = stream_to_sample
            .sampled
            .batches
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<u64>>()
            .into_iter()
            .collect();
        let position_of_the_failure = sampled_in_order.len() / 2;
        let table_name_part_of_the_failure =
            format!("/ordinal={}/", sampled_in_order[position_of_the_failure]);
        let judge_failing_at_that_state = |table_name: &str, pool: &MemoryPool| -> Vec<String> {
            if table_name.contains(&table_name_part_of_the_failure) {
                panic!("{ENGINE_FAILURE_MESSAGE}");
            }
            pool_checker_red_names_of_a_given_image_on_the_cpu(table_name, pool)
        };
        let tables = PoolCheckerTablesKeptInMemory::judging_with(&judge_failing_at_that_state);
        let name = StreamName::NewPoolFileCreation;
        let label = name.directory_name();
        let root = std::env::temp_dir().join(format!("e171-engine-failure-{}", std::process::id()));
        let run_arguments = ["baseline".to_string()];
        let command_line_on = |output_directory: PathBuf| CommandLine {
            run_kind: RunKind::Baseline,
            output_directory,
            switches: RunSwitches::default(),
        };
        let in_one_run_directory = root.join("in-one-run");
        let mut in_one_run_lines = Vec::new();
        let in_one_run = run_in_stages(
            RunStage::RecordThenJudge,
            &run_arguments,
            &in_one_run_directory,
            || {
                recorded_dump_of_the_pairing_stream(
                    &stream_to_sample,
                    &in_one_run_directory,
                    &tables,
                )
            },
            |recorded_dump| {
                judge_the_dump(
                    &command_line_on(in_one_run_directory.clone()),
                    recorded_dump,
                    &tables,
                    &mut in_one_run_lines,
                )
            },
        );
        let in_two_runs_directory = root.join("in-two-runs");
        let recorded = run_in_stages(
            RunStage::RecordOnly,
            &run_arguments,
            &in_two_runs_directory,
            || {
                recorded_dump_of_the_pairing_stream(
                    &stream_to_sample,
                    &in_two_runs_directory,
                    &tables,
                )
            },
            |_recorded: RecordedDump| -> Result<i32, String> { panic!("只跑录入段不判") },
        );
        let mut in_two_runs_lines = Vec::new();
        let in_two_runs = run_in_stages(
            RunStage::JudgeOnly,
            &run_arguments,
            &in_two_runs_directory,
            || -> RecordedDump { panic!("只跑判段不再取样") },
            |recorded_dump| {
                judge_the_dump(
                    &command_line_on(in_two_runs_directory.clone()),
                    recorded_dump,
                    &tables,
                    &mut in_two_runs_lines,
                )
            },
        );
        let before_directory = root.join("before-the-two-stages");
        let before_stream_directory = before_directory.join(label);
        std::fs::create_dir_all(&before_stream_directory)
            .unwrap_or_else(|error| panic!("建 {}：{error}", before_stream_directory.display()));
        let stream = &stream_to_sample.stream;
        let mut crashed_before_the_two_stages = StreamExport::not_exported(
            name,
            format!(
                "implementation_crashed_in_a_state:{ENGINE_FAILURE_MESSAGE}:after_ordinal_{}",
                sampled_in_order[position_of_the_failure - 1]
            ),
            Instant::now(),
        );
        crashed_before_the_two_stages.checks = stream.checks.clone();
        crashed_before_the_two_stages.calls = stream.calls.clone();
        crashed_before_the_two_stages.ending = Some(stream.ending.clone());
        crashed_before_the_two_stages.model_stopped_at_call = stream.model_stopped_at_call;
        crashed_before_the_two_stages.operation_count = stream.operations.len();
        crashed_before_the_two_stages.write_count = stream.writes.len();
        crashed_before_the_two_stages.segment_count = stream.segments.len();
        crashed_before_the_two_stages.files =
            write_the_files_that_exist_even_when_the_stream_is_not_exported(
                &before_stream_directory,
                stream,
            );
        let mut before_lines = Vec::new();
        let before = judge_the_dump(
            &command_line_on(before_directory.clone()),
            RecordedDump {
                stream_independent_checks: stream_independent_checks(),
                exports: vec![crashed_before_the_two_stages],
            },
            &PoolCheckerTablesKeptInMemory::judging_with(
                &pool_checker_red_names_of_a_given_image_on_the_cpu,
            ),
            &mut before_lines,
        );
        let hand_off_is_left = in_two_runs_directory
            .join(HAND_OFF_TO_THE_JUDGE_STAGE_FILE)
            .exists();
        let files_in_one_run = files_without_timing_under(&in_one_run_directory.join(label));
        let files_in_two_runs = files_without_timing_under(&in_two_runs_directory.join(label));
        let files_before = files_without_timing_under(&before_stream_directory);
        std::fs::remove_dir_all(&root)
            .unwrap_or_else(|error| panic!("删 {}：{error}", root.display()));
        let before_lines = String::from_utf8(before_lines).expect("结果行是 UTF-8");
        assert_eq!(recorded, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
        assert_eq!(before, Ok(3), "基线那一趟有流没导出，停机：{before_lines}");
        assert_eq!(in_one_run, before, "两段先后跑与两段之前的退出码相同");
        assert_eq!(in_two_runs, before, "分两趟跑与两段之前的退出码相同");
        assert_eq!(
            String::from_utf8(in_one_run_lines).expect("结果行是 UTF-8"),
            before_lines,
            "两段先后跑与两段之前的结果行逐字节相同"
        );
        assert_eq!(
            String::from_utf8(in_two_runs_lines).expect("结果行是 UTF-8"),
            before_lines,
            "分两趟跑与两段之前的结果行逐字节相同"
        );
        assert!(!hand_off_is_left, "判段判完删掉交接");
        assert_eq!(
            files_before.keys().collect::<Vec<&String>>(),
            FILES_WRITTEN_EVEN_WHEN_THE_STREAM_IS_NOT_EXPORTED
                .map(str::to_string)
                .iter()
                .collect::<Vec<&String>>(),
            "两段之前这条流的目录里只有照写的两个文件"
        );
        assert_eq!(
            files_in_one_run, files_before,
            "两段先后跑这条流的目录与两段之前的逐个相同"
        );
        assert_eq!(
            files_in_two_runs, files_before,
            "分两趟跑这条流的目录与两段之前的逐个相同"
        );
    }

    /// 坏镜像模式里录入段构造某一行 E170 坏状态时 panic（`bad-states.tsv` 那一行少一列）：两段先后跑一趟、分两趟跑，与两段之前的 panic 消息、
    /// 结果行、产物文件逐字节相同（分两趟跑的输出目录另留着交接）。两段之前（`e170_bad_rows_on_both_streams` 逐行构造、判，panic 的那一行原样抛出）
    /// 结果行打到 panic 的前一行为止，已知坏镜像的三个产物照写，E170 那一份不写：期望照同一份导出没改坏时那一趟的结果行截到 panic 那一行之前、
    /// 产物去掉 E170 那一份。
    #[test]
    fn a_panic_building_an_e170_bad_row_prints_the_same_in_two_runs_in_one_run_and_as_before_the_two_stages(
    ) {
        const ROW_OF_THE_PANIC: usize = 2;
        let dump = DumpDirectoryForKnownBadImages::new("e170-bad-row-building-panic");
        let unbroken = dump.run_on(
            SEVERAL_THREADS,
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        );
        assert_eq!(
            unbroken.ending,
            Ok(0),
            "没改坏的导出停机条款都过：{}",
            unbroken.result_lines
        );
        let label = StreamName::NewPoolFileCreation.directory_name();
        let stream_directory = dump.directory.join(label);
        let mut state_rows = read_text_lines(&stream_directory, "bad-states.tsv");
        let (without_the_patches, _patches) = state_rows[ROW_OF_THE_PANIC]
            .rsplit_once('\t')
            .expect("bad-states.tsv 一行五列");
        state_rows[ROW_OF_THE_PANIC] = without_the_patches.to_string();
        std::fs::write(
            stream_directory.join("bad-states.tsv"),
            state_rows
                .iter()
                .map(|row| format!("{row}\n"))
                .collect::<String>(),
        )
        .unwrap_or_else(|error| panic!("写 bad-states.tsv：{error}"));
        let first_line_not_printed_before = format!(
            "E7RESULT name=e170_bad_row stream={label} row={} ",
            e170_bad_row_identifier(ROW_OF_THE_PANIC)
        );
        let lines_before = unbroken.result_lines[..unbroken
            .result_lines
            .find(&first_line_not_printed_before)
            .expect("没改坏的那一趟打了 panic 那一行")]
            .to_string();
        let mut files_before = unbroken.files;
        assert!(
            files_before.remove("e170-bad-rows-o2.tsv").is_some(),
            "没改坏的那一趟写了 E170 那一份产物"
        );
        let in_one_run = dump.run_on(
            SEVERAL_THREADS,
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        );
        let tables = PoolCheckerTablesKeptInMemory::judging_with(
            &pool_checker_red_names_of_a_given_image_on_the_cpu,
        );
        let output_directory = dump.directory.join("output-in-two-runs");
        let recorded = dump.run_stage_on(
            RunStage::RecordOnly,
            SEVERAL_THREADS,
            &output_directory,
            &tables,
        );
        let judged = dump.run_stage_on(
            RunStage::JudgeOnly,
            SEVERAL_THREADS,
            &output_directory,
            &tables,
        );
        let mut files_in_two_runs =
            DumpDirectoryForKnownBadImages::read_and_remove(&output_directory);
        let hand_off_is_left = files_in_two_runs
            .remove(HAND_OFF_TO_THE_JUDGE_STAGE_FILE)
            .is_some();
        assert!(
            in_one_run
                .ending
                .as_ref()
                .is_err_and(|text| text.contains("bad-states.tsv 一行五列")),
            "构造那一行时的 panic 原样抛出来：{:?}",
            in_one_run.ending
        );
        assert_eq!(
            in_one_run.result_lines, lines_before,
            "两段先后跑与两段之前的结果行逐字节相同（打到 panic 的前一行为止）"
        );
        assert_eq!(
            in_one_run.files, files_before,
            "两段先后跑与两段之前的产物逐字节相同（E170 那一份不写）"
        );
        assert_eq!(recorded.ending, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
        assert_eq!(
            judged.ending, in_one_run.ending,
            "分两趟跑与两段先后跑 panic 的消息相同"
        );
        assert_eq!(
            judged.result_lines, lines_before,
            "分两趟跑与两段之前的结果行逐字节相同"
        );
        assert_eq!(
            files_in_two_runs, files_before,
            "分两趟跑与两段之前的产物逐字节相同"
        );
        assert!(hand_off_is_left, "判段 panic 了，交接留着");
    }

    /// 判段读到的交接里录的盘在判段这一边读不回（判段换了一份没录过它们的库）：拒（退非 0），报错句点名那张表与出路，
    /// 一行结果行都不打、一个产物都不写，交接留着。
    #[test]
    fn the_judge_stage_refuses_a_hand_off_whose_tables_cannot_be_read_back() {
        let dump = DumpDirectoryForKnownBadImages::new("tables-not-read-back");
        let output_directory = dump.directory.join("output-judged-on-another-library");
        let recorded = dump.run_stage_on(
            RunStage::RecordOnly,
            SEVERAL_THREADS,
            &output_directory,
            &PoolCheckerTablesKeptInMemory::judging_with(
                &pool_checker_red_names_of_a_given_image_on_the_cpu,
            ),
        );
        let judged = dump.run_stage_on(
            RunStage::JudgeOnly,
            SEVERAL_THREADS,
            &output_directory,
            &PoolCheckerTablesKeptInMemory::judging_with(
                &pool_checker_red_names_of_a_given_image_on_the_cpu,
            ),
        );
        let files = DumpDirectoryForKnownBadImages::read_and_remove(&output_directory);
        assert_eq!(recorded.ending, Ok(EXIT_CODE_OF_A_RECORD_ONLY_RUN));
        let refusal = judged.ending.expect_err("交接里录的盘读不回，要拒");
        assert!(
            refusal.contains("known-bad/clean") && refusal.contains("先跑录入段"),
            "报错句点名录入次序上第一张读不回的表与出路：{refusal}"
        );
        assert!(
            judged.result_lines.is_empty(),
            "拒了就不打结果行：{}",
            judged.result_lines
        );
        assert_eq!(
            files.keys().collect::<Vec<&String>>(),
            vec![&HAND_OFF_TO_THE_JUDGE_STAGE_FILE.to_string()],
            "一个产物都不写，交接留着"
        );
    }
}

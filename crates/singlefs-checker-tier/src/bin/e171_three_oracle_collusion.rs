//! E171（三oracle合谋测试） 第一段：入库装置。
//!
//! 登记：`research/prompts/e171-preregistration.md` 5.3（装置写在哪）、5.5（坏镜像表）、5.6（阳性与阴性对照）、5.8「第一段」、
//! 第十一节停机 S1–S6 与 S8。它回答问题单 Q1–Q3 里 O1、O2 两列的那一半：两条层 0 流（新池新建文件、固定脚本到 E 再正常卸载）上，
//! 每次 API 调用之后拿理想模型的答案与实现的结局比（O1-对拍），每个取样的崩溃状态上拿实现只读恢复读回的那一版问理想模型
//! （O1-崩溃判官），再拿池级 checker 判这一态的崩溃镜像（O2）；O3（Python 执行器）读本装置照 E170（O3规约执行器第一版） 登记 5.3 写出的
//! 中性格式，不在这个文件里。
//!
//! 只读、只驱动、只观测：不改 `crates/singlefs-core`、`crates/singlefs-checker`、harness、format 与 replay-prefix。
//! 两条流的驱动、取样、中性格式的写出与读回照抄 `e170_o3_spec_executor_export.rs`（不 `#[path]` 引它），改三处（登记 5.3）：
//! ① 驱动里的 API 调用不 `.expect`：结局（成功 / 错误成员 / panic）记进 `calls.tsv` 的 `ok` / `err` 列与 `call-outcomes.tsv`，
//! 出错之后这条流不再往下驱动；② 跟着每次调用推理想模型，记 O1-对拍；③ 每个取样的状态算 O1-崩溃判官与 O2，写 `oracle-verdicts.tsv`。
//!
//!   e171_three_oracle_collusion baseline --output-directory <目录>
//!   e171_three_oracle_collusion row --output-directory <目录>
//!   e171_three_oracle_collusion known-bad-images --dump-directory <基线导出目录> --output-directory <目录> [--e170-bad-rows]
//!   以上三种之后都可以加 `--positive-control <O1-对拍|O1-崩溃判官|O2>` 或 `--disable-oracle <O1|O2>`（只对 baseline 与 row）
//!
//! `baseline`：回比本地常量（S1）、三批取样、写十个导出文件加 `oracle-verdicts.tsv` 与 `call-outcomes.tsv`；`row`：同上，只是
//! 不回比常量（坏实现那一趟本来就会让写流与段表变）、只取批 1 与批 2；S5 在这一趟也判：返回了错误的调用（卸载除外）都记成 O1-对拍的红
//! （取号、暖机出错也是），不适用只来自卸载与对拍判出不同之后（`s5_checks_of_a_bad_implementation_row`）。目录已存在就拒绝。
//! 标准输出：`E7RESULT` 行（与跑几次无关）与 `E7TIMING` 行（耗时）；收尾 `name=done emitted=N`。
//! 退出码：0 正常；2 命令行不对或输出目录已存在；3 停机条款 S1、S3、S4、S5 没过（产物照打，交主 agent）。

#[path = "../../../singlefs-harness/tests/common/mod.rs"]
mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_checker_tier::crash::{
    full_expansion, judge_layer0_state_range, layer0_enumeration_tables, layer0_plan_state_count,
    quick_tier_expansion, Layer0EnumerationTables, Layer0StateJudgement,
};
use singlefs_checker_tier::crash_facts::{extract_flow_facts, FlowFacts};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::make_filesystem::{INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget,
    ShadowLedger, UnmountRaisedTheFloor, Unmounted,
};
use singlefs_core::recovery::{PoolReader, RecoveryOutcome, RecoveryReport};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion,
    PoolWriter, TransactionOutput, WarmUpOutput,
};
use singlefs_harness::memory_pool::{
    newest_persisted_root, writes_and_segments_with_stream_indexes_and_entries, CrashImage,
    MemoryPool, PublishedVersion, RetainedWrite, WrittenContents,
};
use singlefs_harness::model::{
    crash_recovery_disagreement, IdealModel, ModelAnswer, ModelDeviceIdentity, ModelDisagreement,
    ModelPoolGeometry, ModelRootKey, ObservedEffect, ObservedFile, ObservedOutcome,
    ObservedReadBack,
};
use singlefs_harness::model_comparison::{
    model_root_key, observed_mount_carrying_to_zero_unit_publishes,
    observed_read_back_after_a_crash, observed_root_of_file_version, refusal_reason_of_mount_error,
    refusal_reason_of_publish_error, refusal_reason_of_rollback_error,
    reported_ceiling_of_mount_error, root_ring_slot_still_bad_after_one_reread_of_mount_error,
    root_ring_slot_still_bad_after_one_reread_of_rollback_error,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::sha256::{sha256_digest, sha256_hexadecimal, DIGEST_BYTES};
use singlefs_harness::{
    RecordedEntrySpan, RecordedOperationKind, RecordedPublishEntry, RetainedOperation, SharedStream,
};

// ───────────────────────────── 本地常量（登记 5.3、S1） ─────────────────────────────
//
// 下面这一批与 `e170_o3_spec_executor_export.rs` 第 78–140 行同一批、同样的值与出处注释；`baseline` 开头与 crates 的同名公开常量或现算值回比，
// 不直接引（决定几何与工作量的数写成本地常量：引了就等于装置与实现共用一个数，两边一起错而逐格相等）。
// 这些常量只在基线那一趟回比：坏实现改了写流，这些数在坏实现那一趟本来就会变，照数记进产物。

/// 盘数：`tests/common/mod.rs` 第 152–184 行 `formatted_recorded_images` 建两块盘（`for device_number in 0..2u32`）。
const LOCAL_DEVICE_COUNT: usize = 2;
/// 每块盘的字节数：`tests/common/mod.rs` 第 29 行 `IMAGE_BYTES = 4 << 30`。
const LOCAL_DEVICE_BYTES: u64 = 4_294_967_296;
/// 物理块与 io_min：`tests/common/mod.rs` 第 54–55 行 `physical_block_size: 512`、`minimum_input_output_bytes: 512`。
const LOCAL_PHYSICAL_BLOCK_BYTES: u32 = 512;
const LOCAL_MINIMUM_INPUT_OUTPUT_BYTES: u32 = 512;
/// 导出格式的扇区宽（登记 5.3 `sector_bytes`（512））：与 `singlefs_harness::memory_pool::SECTOR_BYTES` 回比。
const LOCAL_SECTOR_BYTES: u64 = 512;
/// 固定 fsid：`tests/common/mod.rs` 第 33–35 行 `E142_FILESYSTEM_IDENTIFIER`。
const LOCAL_FILESYSTEM_IDENTIFIER: [u8; 16] = [
    0x5f, 0x53, 0x46, 0x53, 0x2d, 0x45, 0x31, 0x34, 0x32, 0x2d, 0x30, 0x30, 0x30, 0x31, 0x2d, 0x00,
];
/// 固定写时刻：`tests/common/mod.rs` 第 37 行 `FIXED_WRITE_TIME_SECONDS = 1_788_000_000`。
const LOCAL_FIXED_WRITE_TIME_SECONDS: u64 = 1_788_000_000;
/// 第一个文件的字节数：`tests/common/mod.rs` 第 31 行 `FILE_BYTES = 3000`；内容 `index % 251`（第 71–75 行 `file_content`）。
const LOCAL_FIRST_FILE_BYTES: usize = 3000;
const LOCAL_FIRST_FILE_CONTENT_MODULUS: usize = 251;

/// 流一的段长：`tests/crash_enumeration_new_pool_file_creation_stream.rs` 第 466 行。
const LOCAL_FIRST_STREAM_SEGMENT_LENGTHS: [usize; 11] = [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2];
/// 流一的写数：同一份文件第 471 行（取号 2 + 暖机两次各 5 + A 29）。
const LOCAL_FIRST_STREAM_WRITES: usize = 41;
/// 流一的根槽写数：暖机两次各一条 + A 一条。
const LOCAL_FIRST_STREAM_ROOT_WRITES: usize = 3;
/// 流一全量与快档的状态数：同一份文件第 446 行 `FULL_STATES`、第 450 行 `QUICK_TIER_STATES`。
const LOCAL_FIRST_STREAM_FULL_STATES: u64 = 16_777_260;
const LOCAL_FIRST_STREAM_QUICK_TIER_STATES: u64 = 46;
/// 流一每次写只取两态时的全量闭式：同一份文件第 442 行 `FULL_STATES_WITH_TWO_STATES_PER_WRITE`（单测拿段长表回算它）。
const LOCAL_FIRST_STREAM_TWO_STATE_CLOSED_FORM: u64 = 16_777_240;

/// 流二的段长（78 段）：`tests/crash_enumeration_fixed_script_stream.rs` 第 438–441 行。
const LOCAL_SECOND_STREAM_SEGMENT_LENGTHS: [usize; 78] = [
    2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 18, 2, 1, 2, 16, 2, 1, 2,
    16, // 取号到写行之后的暖机
    2, 1, 2, 24, 2, 1, 2, 20, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2, 28, 2, 1, 2,
    28, // C、回退 D、覆盖写
    2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1, 2, 16, 2, 1, 2, 28, 2, 1, 2, 2, 16, 2, 1,
    2, // 抬 F、E、卸载
    16, 2, 1, 2, // 卸载第二次空发布与 txg 19 的轮换
];
/// 流二的写数：同一份文件第 448 行 `191 + 5 * 33 + (2 + 2 * 21) + 33 + (2 + 2 * 21)`。
const LOCAL_SECOND_STREAM_WRITES: usize = 477;
/// 流二的根槽写数：同一份文件第 482 行（`=> 19`）。
const LOCAL_SECOND_STREAM_ROOT_WRITES: usize = 19;
/// 流二全量与快档的状态数：同一份文件第 100 行 `FULL_STATES_THROUGH_THE_UNMOUNT`、第 114 行 `QUICK_TIER_STATES_THROUGH_THE_UNMOUNT`。
const LOCAL_SECOND_STREAM_FULL_STATES: u64 = 1_662_648_564;
const LOCAL_SECOND_STREAM_QUICK_TIER_STATES: u64 = 278;
/// 流二每次写只取两态时的全量闭式：同一份文件第 96 行（单测拿段长表回算它）。
const LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM: u64 = 1_662_648_449;
/// E 的数据单元落点：同一份文件第 318 行（`50176`）。
const LOCAL_SECOND_STREAM_E_DATA_UNIT_SLOT: u64 = 50_176;
/// 抬 F 推的空发布数：同一份文件第 306 行（`raised.publishes.len(), 2`）。
const LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES: usize = 2;
/// 卸载推的两次空发布的 txg：同一份文件第 336 行（`vec![CheckpointTxg(18), CheckpointTxg(19)]`）。
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
    FixedScriptThroughEThenUnmount,
}

impl StreamName {
    fn directory_name(self) -> &'static str {
        match self {
            Self::NewPoolFileCreation => "new_pool_file_creation",
            Self::FixedScriptThroughEThenUnmount => "fixed_script_through_e_then_unmount",
        }
    }
}

const STREAM_NAMES: [StreamName; 2] = [
    StreamName::NewPoolFileCreation,
    StreamName::FixedScriptThroughEThenUnmount,
];

// ───────────────────────────── 流二的内容生成式（照抄测试文件） ─────────────────────────────

/// `tests/crash_enumeration_fixed_script_stream.rs` 第 50–56 行。
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
    /// 模型这一步不交答：取号、暖机（两次都返回之后一起推一步）与卸载（`crates/singlefs-harness/src/model.rs` 里已有 `answer_unmount`，本装置还没改调它，卸载这一步照旧不问模型；改调归 E171 第 1 次重跑）。
    NotModelled,
    /// 对得上，模型往前走。
    Agrees,
    /// 对不上的第一格；模型停在这里，此后的调用不比（读法 6）。
    Disagrees {
        aspect: &'static str,
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
    label: &'static str,
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

/// 一次跑的开关：阳性对照、关一条。
#[derive(Clone, Copy, Debug, Default)]
struct RunSwitches {
    positive_control: Option<PositiveControl>,
    disabled_oracle: Option<DisabledOracle>,
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
            label,
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

// ───────────────────────────── 驱动两条流（登记 5.3：不 `.expect`、跟着推模型） ─────────────────────────────

fn version(instance: u32, checkpoint_txg: u64, content: Vec<u8>) -> PublishedVersion {
    PublishedVersion {
        instance: InstanceGeneration(instance),
        checkpoint_txg: CheckpointTxg(checkpoint_txg),
        content,
    }
}

/// 一条流驱动时的池：只做过 mkfs 的两块文件镜像（`FormattedPool` 管镜像文件，丢掉就删）、分配器、现行那一版、层 0 手写 oracle 的版本表。
/// 流一照抄 `tests/common/mod.rs` 第 677 行 `build_pool` 的取号、暖机、新池新建文件三步，流二照抄
/// `tests/crash_enumeration_fixed_script_stream.rs` 第 218 行起的 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支；
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
        let floor = singlefs_harness::model::ModelCheckpointTxg(new_floor.0);
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
        }
    }

    /// 正常卸载（`unmount`，入口记在录制流里）：模型没有卸载的答，不比；模型停在这里。
    fn step_unmount(&mut self) -> bool {
        let mut current = PoolVersion::WithFile(self.output.clone().expect("上一版带文件"));
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
    /// `tests/crash_enumeration_fixed_script_stream.rs` 的脚本走到 E 再正常卸载。每一步成功之后才往层 0 手写 oracle 的版本表里记一版。
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
    // 照抄 `tests/crash_enumeration_fixed_script_stream.rs` 第 198–216 行：入口段（卸载入口发的那几段）一起交给切段（C557）。
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
        // 流一的版本表照 `tests/crash_enumeration_new_pool_file_creation_stream.rs` 第 391–399 行 `single_version`。
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
/// 与 `e170_o3_spec_executor_export.rs` 第 550–636 行同一批。
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
    root_without_record: bool,
    claimed_state_missing_unit: bool,
}

impl ImplementationVerdict {
    fn of(judgement: &Layer0StateJudgement) -> Self {
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
                    InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => false,
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

/// 池级 checker 判红的不变量名（`Violated` 才算；`NotApplicable` 不算红）。
fn violated_invariant_names(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<String> {
    verdicts
        .iter()
        .filter(|(_, verdict)| match verdict {
            InvariantVerdict::Violated(_) => true,
            InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_) => false,
        })
        .map(|(invariant, _)| (*invariant).to_string())
        .collect()
}

/// 一个取样状态上看到的：批号、层 0 序号、段号、落地向量、镜像摘要、`judge_crash_image` 的判定（`impl-verdicts.tsv`）、
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
    /// O2 直接调 `check_pool_image` 判红的不变量（关掉 O2 时 None）。
    pool_checker_red: Option<Vec<String>>,
    /// S3：直接调的结果与 `judge_crash_image(...).pool_checker` 判红的不变量集合相同。
    pool_checker_matches_the_judgement: bool,
    consulted: RecoverySummary,
    ignored: RecoverySummary,
    o1_nanoseconds: u128,
    o2_nanoseconds: u128,
    recovery_nanoseconds: u128,
}

/// 一次观测的状态：阳性对照里「批 1 第一个读回了文件的态」只翻一次。
struct ObservationControls {
    switches: RunSwitches,
    crash_judge_control_pending: bool,
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

/// O1-崩溃判官在一个状态上的判定：交法照 `crash_injection.rs` 第 1083–1107 行——「盘上已持久的最新根槽」由 `newest_persisted_root`
/// 从写流数出（只看写表里的原写，不含撕裂镜像），读回由 `observed_read_back_after_a_crash` 从看 journal 那一遍恢复报告造，
/// 模型那一侧给每一步之后并起来的版本目录，交给 `crash_recovery_disagreement`（S8：三处调用都在这个函数里）。
fn crash_judge_of_the_state(
    stream: &PreparedStream,
    image: &CrashImage<'_>,
    consulted: &RecoveryReport,
    batch: u8,
    controls: &mut ObservationControls,
) -> CrashJudgeVerdict {
    let newest_persisted_root_key =
        newest_persisted_root(&stream.writes, &image.persisted[..stream.writes.len()])
            .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
    let mut read_back = observed_read_back_after_a_crash(consulted);
    if controls.crash_judge_control_pending && batch == 1 {
        if let ObservedReadBack::FileRead { content, .. } = &mut read_back {
            if let Some(last_byte) = content.last_mut() {
                *last_byte ^= 1;
                controls.crash_judge_control_pending = false;
            }
        }
    }
    crash_judge_verdict(
        &stream.committed_versions,
        newest_persisted_root_key,
        &read_back,
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

/// 按序号升序把取样的状态切成连续的几段，每段调一次 `judge_layer0_state_range`：回调里给的那个 `CrashImage` 折回落地向量、算摘要，
/// 再算 O1-崩溃判官与 O2（直接调 `check_pool_image`），并单独计恢复的耗时。回调的持久集合另与 `crash_facts` 按序号现算的掩码比。
/// 某个状态上实现 panic（实现自崩）：交回 `Err`，带最后一个判完的序号。
fn observe_sampled_states(
    stream: &PreparedStream,
    tables: &Layer0EnumerationTables,
    facts: &FlowFacts,
    torn_image_index_by_write: &BTreeMap<usize, usize>,
    sampled: &SampledOrdinals,
    switches: RunSwitches,
) -> Result<Vec<StateObservation>, String> {
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
    let mut controls = ObservationControls {
        switches,
        crash_judge_control_pending: switches.positive_control == Some(PositiveControl::CrashJudge),
    };
    let mut observations: Vec<StateObservation> = Vec::new();
    for run in runs {
        let judged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            judge_layer0_state_range(
                &stream.base,
                &stream.writes,
                &stream.segments,
                stream.judged_root_index,
                &stream.versions,
                &full_expansion,
                run,
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
                            image,
                            &judgement.consulted,
                            batch,
                            &mut controls,
                        )
                    };
                    let o1_nanoseconds = crash_judge_started.elapsed().as_nanos();
                    let pool_checker_started = Instant::now();
                    let pool_checker_red = pool_checker_is_enabled.then(|| {
                        let direct = if controls.switches.positive_control
                            == Some(PositiveControl::PoolChecker)
                            && stream.name == StreamName::NewPoolFileCreation
                            && ordinal == tables.state_count - 1
                        {
                            pool_checker_on_the_first_known_bad_image(image)
                        } else {
                            check_pool_image(image)
                        };
                        violated_invariant_names(&direct)
                    });
                    let o2_nanoseconds = pool_checker_started.elapsed().as_nanos();
                    let pool_checker_matches_the_judgement = pool_checker_red
                        .as_ref()
                        .is_none_or(|direct| *direct == verdict.pool_checker_red)
                        || controls.switches.positive_control == Some(PositiveControl::PoolChecker);
                    let recovery_started = Instant::now();
                    let _ = singlefs_core::recovery::recover(
                        image,
                        singlefs_core::recovery::JournalPolicy::Consult,
                    );
                    let recovery_nanoseconds = recovery_started.elapsed().as_nanos();
                    observations.push(StateObservation {
                        batch,
                        ordinal,
                        segment_index: tables
                            .state_ranges_by_segment
                            .partition_point(|state_range| state_range.end <= ordinal),
                        landing_vector,
                        image_digest: sha256_text(&crash_image_digest(image)),
                        verdict,
                        judge_nanoseconds: o1_nanoseconds + o2_nanoseconds + recovery_nanoseconds,
                        mask_matches_the_facts: facts.persisted_writes_of_state(ordinal)
                            == image.persisted,
                        touched_call,
                        crash_judge,
                        pool_checker_red,
                        pool_checker_matches_the_judgement,
                        consulted: RecoverySummary::of(&judgement.consulted),
                        ignored: RecoverySummary::of(&judgement.ignored),
                        o1_nanoseconds,
                        o2_nanoseconds,
                        recovery_nanoseconds,
                    });
                },
            )
        }));
        match judged {
            Ok(state_count_of_the_stream) => assert_eq!(
                state_count_of_the_stream, tables.state_count,
                "judge_layer0_state_range 交回的整条流状态数与枚举表的相同"
            ),
            Err(payload) => {
                return Err(format!(
                    "{}:after_ordinal_{}",
                    panic_text(&*payload),
                    observations
                        .last()
                        .map_or_else(|| "none".to_string(), |last| last.ordinal.to_string())
                ));
            }
        }
    }
    observations.sort_by_key(|observation| (observation.batch, observation.ordinal));
    Ok(observations)
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

/// `oracle-verdicts.tsv`：每个取样状态一行（次序与 `states.tsv` 相同）——状态号、批号、层 0 序号、段号、触达的最后一次调用、
/// O1-崩溃判官（`green` / `red` / `na` / `off`）与红的内容、O2（同）与判红的不变量、O2 直接调与 `judge_crash_image` 内那一遍是否一致、
/// 看 journal 那一遍恢复的摘要（根、结局、读回 sha256、施加记录数）、不看 journal 那一遍的摘要、层 0 的 `red_items`（附带：记录核对器与手写版本表的 oracle）。
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
                | CrashJudgeVerdict::Off => "-".to_string(),
            };
            format!(
                "{state_number}\t{}\t{}\t{}\t{}\t{}\t{o1_detail}\t{o2_word}\t{o2_items}\t{}\t{}\t{}\t{}\n",
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
            )
        })
        .collect()
}

const ORACLE_VERDICTS_FILE: &str = "oracle-verdicts.tsv";
const CALL_OUTCOMES_FILE: &str = "call-outcomes.tsv";
const STREAM_STATUS_FILE: &str = "stream-status.tsv";

/// 按 E170 登记 5.3 写一条流的十个文件，manifest 最后写（它记前九个的 sha256）；之后再写本实验自己的三个：
/// `oracle-verdicts.tsv`、`call-outcomes.tsv`、`stream-status.tsv`（不进 manifest：manifest 与 E170 导出装置写的同一份逐字节可比，停机 S2）。
fn write_stream_files(
    directory: &Path,
    stream: &PreparedStream,
    tables: &Layer0EnumerationTables,
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
    files
        .extend(write_the_files_that_exist_even_when_the_stream_is_not_exported(directory, stream));
    files
}

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
    quick_tier_states: u64,
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
            quick_tier_states: 0,
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
}

/// 基线与坏实现那一趟的差别：基线回比本地常量、取三批；坏实现那一趟不回比（写流与段表本来就会变）、只取批 1 与批 2。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunKind {
    Baseline,
    BadImplementationRow,
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
    let tables = layer0_enumeration_tables(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let full_states = layer0_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &full_expansion,
    );
    let quick_tier_states = layer0_plan_state_count(
        &stream.base,
        &stream.writes,
        &stream.segments,
        &quick_tier_expansion,
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
    ) {
        Ok(observations) => observations,
        Err(what) => {
            export.files = write_the_files_that_exist_even_when_the_stream_is_not_exported(
                &directory, &stream,
            );
            export.checks = checks;
            export.not_exported = Some(format!("implementation_crashed_in_a_state:{what}"));
            export.elapsed_nanoseconds = started.elapsed().as_nanos();
            return export;
        }
    };
    let observe_nanoseconds = observe_started.elapsed().as_nanos();
    let write_started = Instant::now();
    let files = write_stream_files(
        &directory,
        &stream,
        &tables,
        &observations,
        full_states,
        quick_tier_states,
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
/// 对本地常量。与 `e170_o3_spec_executor_export.rs` 第 1298–1334 行同一批。
fn state_count_checks_against_the_crates(
    name: StreamName,
    stream: &PreparedStream,
    tables: &Layer0EnumerationTables,
    facts: &FlowFacts,
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
        StopCheck::new(
            &format!("{label}_flow_facts_state_count"),
            full_states,
            facts.state_count,
        ),
    ]
}

// ───────────────────────────── 结果行、V2 与 main ─────────────────────────────

/// `E7RESULT` 行计数（收尾 `name=done emitted=N`，N 把 done 自己也算进去）；`E7TIMING` 行不计数、不进入库产物。
struct ResultLines {
    emitted: u64,
}

impl ResultLines {
    fn result(&mut self, body: &str) {
        self.emitted += 1;
        println!("E7RESULT {body}");
    }
    fn timing(body: &str) {
        println!("E7TIMING {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
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
// 那份用例是 O2 自己的回归语料：每一份都照 O2 的判据造来让 O2 红。下面这个模块逐行照抄它的导入（第 7–68 行）、常量与辅助函数（第 70–745 行）、
// `known_bad_images`（第 747–1294 行），再抄 `known_bad_images` 用到而散在第 1295 行之后的几个常量与辅助函数（行号在模块末尾的注释里）。
// 文件 sha256 e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820；它的 `#[test]` 没有抄。照抄、不 `#[path]` 引：
// 那个文件有自己的 `mod common;` 与全部测试，私有的 `known_bad_images` 也引不到。条目名与条数在 `known-bad-images` 模式里与原文件逐条比（停机 S4、断言 A8）。
#[allow(
    dead_code,
    unused_imports,
    clippy::all,
    clippy::pedantic,
    reason = "逐行照抄 checker_known_bad_images.rs 的辅助函数与坏镜像表：那份文件里别的族用的辅助函数在这里没人调，抄来的写法也不照本文件的 lint 改"
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
        INODE_CHANGE_COUNT_BETWEEN_OBJECT_BIRTH_AND_CONTAINER_BIRTH,
        INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
        JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE, JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
        NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
        PREVIOUS_SNAPSHOT_TXG_OF_THE_TREE_TABLE_ENTRIES,
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

    fn verdict(pool: &MemoryPool, invariant: &str) -> InvariantVerdict {
        check_pool_image(pool)
            .into_iter()
            .find(|(identifier, _)| *identifier == invariant)
            .expect("清单里有")
            .1
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
            // 它不是本实例第一条（本实例第一条是 w1、计数器 1），链值必须等于 CRC32C(w1 的 311 字节头)。
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
        ]
    }

    /// 发布 B（txg 4）之后的干净镜像：A 的十二个单元（十四个槽）在这次发布里改写成已释放、释放代 4，树表是 B 写的那一版，
    /// A 那一版仍被根环里 A 的根指着。I-3.9（释放代落在停止引用它的那一格区间里）与 I-9.14（树表条目的诞生 txg 跨根不变）
    /// 要的跨根比较到这里才有内容：写完新池新建文件的那份镜像上只有 A 的根有树表条目（mkfs 种的第 0 版树表是空的、
    /// 两次暖机空发布不写树表），I-9.14 在那份镜像上报「不适用」。

    // 以下是散在原文件第 1295 行之后、被上面用到的常量与辅助函数，原文件行号：3071 1543 3179 3261 3246 3119 3573 3113 3115 3116 3114 3111

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

    /// 对外的口子：第 747 行的 `known_bad_images` 是私有的，这里转一手（类型就是第 745 行的 `Mutation`）。
    pub(super) fn entries_of_the_known_bad_images(
        clean: &MemoryPool,
    ) -> Vec<(&'static str, Box<dyn Fn(&mut MemoryPool)>)> {
        known_bad_images(clean)
    }
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

/// 阳性对照 O2：把已知坏镜像族的第一条改法施加在这一态的物化镜像上，再交 `check_pool_image`。
fn pool_checker_on_the_first_known_bad_image(
    image: &CrashImage<'_>,
) -> Vec<(&'static str, InvariantVerdict)> {
    let clean = materialized_memory_pool(image);
    let entries =
        known_bad_images_copied_from_the_harness_test::entries_of_the_known_bad_images(&clean);
    let (_, apply_the_first_entry) = entries
        .first()
        .expect("已知坏镜像族至少一条（S4：装置数出的条数要是 41）");
    let mut bad = clean.clone();
    apply_the_first_entry(&mut bad);
    check_pool_image(&bad)
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

// ───────────────────────────── `known-bad-images` 模式：41 条已知坏镜像与 E170 的 P 行、盲测族上的 O2（登记 5.5） ─────────────────────────────

/// 原用例文件的 sha256（登记第三节、A8）：变了先重数条目再比，不照抄的那一份当作没变。
const KNOWN_BAD_TEST_FILE_SHA256: &str =
    "e8543908a3cb08024037773ffc62997a23e0b1d87742e9ea897fa9c8145be820";
/// `known_bad_images(clean)` 的条目数（登记第七节 A8：第 747–1294 行里 `Box::new(` 41 处，字面名 38 条、常量名 3 条）。
const KNOWN_BAD_ENTRY_COUNT: usize = 41;

/// 原用例文件里 `known_bad_images` 一函数里每条的目标不变量名，按文件次序：十二格缩进的一行写字面名（`"I-x.y",`）或常量名。
fn entry_names_in_the_original_test_file(text: &str) -> Vec<String> {
    let mut inside = false;
    let mut names = Vec::new();
    for line in text.lines() {
        if line.starts_with("fn known_bad_images(clean: &MemoryPool)") {
            inside = true;
            continue;
        }
        if inside && line == "}" {
            break;
        }
        if !inside || !line.starts_with("            ") || line.starts_with("             ") {
            continue;
        }
        let token = line.trim().trim_end_matches(',');
        if let Some(literal) = token
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
        {
            if literal.starts_with("I-") {
                names.push(literal.to_string());
            }
        } else if !token.is_empty()
            && token
                .chars()
                .all(|character| character.is_ascii_uppercase() || character == '_')
        {
            names.push(match token {
                "SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS" => {
                    singlefs_checker::image::SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS
                        .to_string()
                }
                "SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY" => {
                    singlefs_checker::image::SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY
                        .to_string()
                }
                "MAPPING_KEY_INVARIANT" => {
                    singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER.to_string()
                }
                unresolved => format!("unresolved_constant:{unresolved}"),
            });
        }
    }
    names
}

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

/// 41 条已知坏镜像在流一全持久态上的 O2：每一条施加在干净镜像的拷贝上，一边直接判（`MemoryPool`），一边取补丁（逐扇区的差）再叠回干净镜像判，
/// 两边判红的不变量集合要相等（S4）、目标不变量要在里面（S4）。交回（停机条款没过的条数）。
fn known_bad_images_on_the_first_stream(
    dump_directory: &Path,
    output_directory: &Path,
    lines: &mut ResultLines,
) -> usize {
    let mut failed = 0usize;
    let stream_directory = dump_directory.join(StreamName::NewPoolFileCreation.directory_name());
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
    for check in [
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
    ] {
        failed += usize::from(!check.holds());
        lines.result(&format!(
            "name=s4 check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let test_file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../singlefs-harness/tests/checker_known_bad_images.rs");
    let test_file_text = std::fs::read_to_string(&test_file_path)
        .unwrap_or_else(|error| panic!("读 {}：{error}", test_file_path.display()));
    let test_file_sha256 = sha256_hexadecimal(test_file_text.as_bytes());
    let entries =
        known_bad_images_copied_from_the_harness_test::entries_of_the_known_bad_images(&clean);
    let names_here: Vec<&str> = entries.iter().map(|(name, _)| *name).collect();
    let names_in_the_file = entry_names_in_the_original_test_file(&test_file_text);
    for check in [
        StopCheck::new(
            "known_bad_test_file_sha256_is_the_copied_one",
            KNOWN_BAD_TEST_FILE_SHA256,
            &test_file_sha256,
        ),
        StopCheck::new(
            "known_bad_entry_count_here",
            KNOWN_BAD_ENTRY_COUNT,
            entries.len(),
        ),
        StopCheck::new(
            "known_bad_entry_count_in_the_original_file",
            KNOWN_BAD_ENTRY_COUNT,
            names_in_the_file.len(),
        ),
        StopCheck::new(
            "known_bad_entry_names_equal_the_original_file_in_order",
            &names_in_the_file,
            &names_here,
        ),
    ] {
        failed += usize::from(!check.holds());
        lines.result(&format!(
            "name=s4 check={} local={} crates={} holds={}",
            check.name,
            field_text(&check.local),
            field_text(&check.crates),
            check.holds()
        ));
    }
    let clean_red = violated_invariant_names(&check_pool_image(&clean));
    lines.result(&format!(
        "name=known_bad_clean_image invariants_red={}",
        field_text(&list_text(&clean_red))
    ));
    let mut states_text = String::new();
    let mut patches_file: Vec<u8> = Vec::new();
    let mut o2_text = String::new();
    for (entry_index, (target, apply)) in entries.iter().enumerate() {
        let mut bad = clean.clone();
        apply(&mut bad);
        let direct_red = violated_invariant_names(&check_pool_image(&bad));
        let patches = sector_difference_as_patches(&clean, &bad);
        let mut patched = clean.clone();
        apply_patches(&mut patched, &patches);
        let patched_red = violated_invariant_names(&check_pool_image(&patched));
        let patched_digest = sha256_text(&memory_pool_digest(&patched));
        let digest_equals_the_direct_one = patched_digest == sha256_text(&memory_pool_digest(&bad));
        let sets_equal = direct_red == patched_red;
        let target_is_red = patched_red.iter().any(|name| name == target);
        failed += usize::from(!(sets_equal && digest_equals_the_direct_one && target_is_red));
        states_text.push_str(&format!(
            "{entry_index}\t{all_persisted}\t{clean_digest}\t{patched_digest}\t{}\n",
            patch_list_text(&patches, &mut patches_file)
        ));
        o2_text.push_str(&format!(
            "{entry_index}\t{target}\t{}\t{}\t{sets_equal}\t{target_is_red}\n",
            list_text(&patched_red),
            list_text(&direct_red)
        ));
        lines.result(&format!(
            "name=known_bad_entry index={entry_index} target={target} o2={} patched_red={} direct_red={} sets_equal={sets_equal} digest_equals_the_direct_one={digest_equals_the_direct_one} target_is_red={target_is_red} patches={}",
            if patched_red.is_empty() { "green" } else { "red" },
            field_text(&list_text(&patched_red)),
            field_text(&list_text(&direct_red)),
            patches.len()
        ));
    }
    std::fs::create_dir_all(output_directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", output_directory.display()));
    for (file_name, contents) in [
        ("known-bad-states.tsv", states_text.into_bytes()),
        ("known-bad-patches.bin", patches_file),
        ("known-bad-o2.tsv", o2_text.into_bytes()),
    ] {
        let written = write_export_file(output_directory, file_name, &contents);
        lines.result(&format!(
            "name=file file={file_name} bytes={} sha256={}",
            written.bytes, written.sha256
        ));
    }
    failed
}

/// E170 的 `bad-states` 模式造的坏状态（P 行、N 行、盲测族）上的 O2：每一行读 `bad-states.tsv` 的落地向量与补丁，从中性格式重建底态、叠补丁，
/// 底态与补丁之后的镜像摘要与 E170 写的两个摘要逐行比，再调 `check_pool_image`，判红的不变量与 E170 `bad-impl-verdicts.tsv` 里池级 checker 那一列逐行比（S3）。
/// 交回（对不上的行数）。只读 `bad-states.tsv`、`bad-patches.bin`、`bad-impl-verdicts.tsv`，不读期望文件。
fn e170_bad_rows_on_both_streams(
    dump_directory: &Path,
    output_directory: &Path,
    lines: &mut ResultLines,
) -> usize {
    let mut failed = 0usize;
    let mut o2_text = String::new();
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
        for (state_row, verdict_row) in state_rows.iter().zip(&verdict_rows) {
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
            let row_id = state_fields[0];
            let mut pool = rebuilt_memory_pool(&neutral, state_fields[1]);
            let base_digest = sha256_text(&memory_pool_digest(&pool));
            // 不打补丁的底态上的 O2（登记 5.6 坏镜像行的阴性：F、Mk、Mk⁺、Mk⁻ 各一张，必须不红）。
            let base_red = violated_invariant_names(&check_pool_image(&pool));
            apply_patches(
                &mut pool,
                &patches_of_a_patch_list_text(state_fields[4], &patches_file),
            );
            let patched_digest = sha256_text(&memory_pool_digest(&pool));
            let o2_red = violated_invariant_names(&check_pool_image(&pool));
            let o2_text_of_the_row = list_text(&o2_red);
            let e170_pool_checker_column = verdict_fields[6];
            let digests_equal = base_digest == state_fields[2] && patched_digest == state_fields[3];
            let o2_equals_e170 = o2_text_of_the_row == e170_pool_checker_column;
            failed += usize::from(!(digests_equal && o2_equals_e170));
            o2_text.push_str(&format!(
                "{label}\t{row_id}\t{}\t{o2_text_of_the_row}\t{e170_pool_checker_column}\t{o2_equals_e170}\t{digests_equal}\t{}\t{base_digest}\n",
                if o2_red.is_empty() { "green" } else { "red" },
                list_text(&base_red)
            ));
            lines.result(&format!(
                "name=e170_bad_row stream={label} row={row_id} o2={} o2_red={} e170_pool_checker={} o2_equals_e170={o2_equals_e170} digests_equal={digests_equal} base_o2_red={}",
                if o2_red.is_empty() { "green" } else { "red" },
                field_text(&o2_text_of_the_row),
                field_text(e170_pool_checker_column),
                field_text(&list_text(&base_red))
            ));
        }
    }
    std::fs::create_dir_all(output_directory)
        .unwrap_or_else(|error| panic!("建 {}：{error}", output_directory.display()));
    let written = write_export_file(output_directory, "e170-bad-rows-o2.tsv", o2_text.as_bytes());
    lines.result(&format!(
        "name=file file=e170-bad-rows-o2.tsv bytes={} sha256={}",
        written.bytes, written.sha256
    ));
    failed
}

/// 一条流的结果行（`E7RESULT` 与 `E7TIMING`）；交回 (S1 没过的条数, S4 对不上的态数, 掩码对不上的态数, S3 对不上的态数,
/// O1-对拍判红的调用数, O1-崩溃判官判红的态数, O2 判红的态数)。
fn report_stream(lines: &mut ResultLines, export: &StreamExport) -> [usize; 7] {
    let label = export.name.directory_name();
    let mut failed_checks = 0usize;
    for check in &export.checks {
        failed_checks += usize::from(!check.holds());
        lines.result(&format!(
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
    lines.result(&format!(
        "name=stream stream={label} ending={ending_text} exported={} not_exported_why={} operations={} writes={} segments={} tearable_writes={} calls={} full_states={} quick_tier_states={} sampled_states={} model_stopped_at_call={}",
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
            .map_or_else(|| "none".to_string(), |call| call.to_string())
    ));
    let mut red_calls = 0usize;
    for (call_index, call) in export.calls.iter().enumerate() {
        red_calls += usize::from(matches!(call.model, ModelVerdictOfCall::Disagrees { .. }));
        lines.result(&format!(
            "name=call stream={label} call={call_index} label={} first_operation={} return_point={} status={} o1_comparison={}",
            call.label,
            call.first_operation,
            call.return_point,
            field_text(&call.outcome.outcome_word()),
            field_text(&call.model.word()),
        ));
    }
    for scan in &export.torn_scans {
        lines.result(&format!(
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
        let o2_red = count_where(&|observation| {
            observation
                .pool_checker_red
                .as_ref()
                .is_some_and(|items| !items.is_empty())
        });
        crash_judge_red += crash_judge_is("red");
        pool_checker_red += o2_red;
        let segments: BTreeSet<usize> = in_batch
            .iter()
            .map(|observation| observation.segment_index)
            .collect();
        lines.result(&format!(
            "name=batch stream={label} batch={batch} states={} segments_touched={} states_with_a_torn_write={} o1_crash_judge_red={} o1_crash_judge_green={} o1_crash_judge_not_applicable={} o1_crash_judge_off={} o2_red={o2_red} o2_matches_judgement_false={} judge_crash_image_pool_checker_red={} record_checker_red_states={} consulted_oracle_red_states={} ignored_oracle_red_states={} consulted_effective_root_none={} consulted_failed_outcomes={}",
            in_batch.len(),
            segments.len(),
            count_where(&|observation| observation.landing_vector.contains('1')),
            crash_judge_is("red"),
            crash_judge_is("green"),
            crash_judge_is("na"),
            crash_judge_is("off"),
            count_where(&|observation| !observation.pool_checker_matches_the_judgement),
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
    let mask_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.mask_matches_the_facts)
        .count();
    let s3_mismatches = export
        .observations
        .iter()
        .filter(|observation| !observation.pool_checker_matches_the_judgement)
        .count();
    lines.result(&format!(
        "name=s4 stream={label} rebuilt_states={} digest_mismatches={}",
        export.rebuilt_states, export.rebuilt_digest_mismatches
    ));
    lines.result(&format!(
        "name=crash_facts_mask stream={label} states={} mismatches={mask_mismatches}",
        export.observations.len()
    ));
    lines.result(&format!(
        "name=s3 stream={label} states={} o2_direct_differs_from_judge_crash_image={s3_mismatches}",
        export.observations.len()
    ));
    for file in &export.files {
        match &file.sha256_without_timing {
            None => lines.result(&format!(
                "name=file stream={label} file={} bytes={} sha256={}",
                file.name, file.bytes, file.sha256
            )),
            Some(without_timing) => {
                lines.result(&format!(
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
        s3_mismatches,
        red_calls,
        crash_judge_red,
        pool_checker_red,
    ]
}

/// 基线上 O1 不适用的来源只有两种（登记第十一节 S5）：模型没有答的 `unmount`，与对拍判出不同之后。基线上不许有对拍判出不同，
/// 所以模型只许停在流二的 `unmount` 那一次调用（流一一次都不停）。这两条按 `StopCheck` 的形式回比：期望（本地）与实际（这一趟）。
fn s5_checks_of_the_baseline(export: &StreamExport) -> Vec<StopCheck> {
    let label = export.name.directory_name();
    let expected_stop = match export.name {
        StreamName::NewPoolFileCreation => None,
        StreamName::FixedScriptThroughEThenUnmount => export
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

/// 逐行那一趟（坏实现、阳性对照；不关 oracle）的 S5：与基线那一趟（[`s5_checks_of_the_baseline`]）同一条停机条款（登记第十一节 S5：
/// 「不适用只许来自 `unmount` 与对拍判出不同之后」）。坏实现可以让对拍判出不同、让模型停在那里（那是 O1-对拍的红），所以这一趟不要求模型
/// 只停在 `unmount`；要求两样（登记第一节读法 5、6），都按「不合的个数」回比 0：
/// ① 返回了错误的调用（卸载除外：模型没有卸载的答）没有一次留在「没问模型」（`not_modelled`）——取号、暖机出错那一形（代码轮第三轮判决
/// K4-c）；模型因对拍判出不同停下之后的调用（`skipped_after_the_model_stopped`）不数：读法 6 原文「一次对拍判出不同之后模型不再往前走」，
/// 那是合规来源（登记修订 4）；② O1-崩溃判官的不适用只来自模型停在卸载、或停在对拍判出不同的那次调用。
fn s5_checks_of_a_bad_implementation_row(
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

/// 这一趟、这一条流要判的 S5（登记第十一节 S5）。基线那一趟判 [`s5_checks_of_the_baseline`]：只判不带对照、不关 oracle、流导出了的基线
/// （阳性对照会让模型在翻过的那次调用停下，关掉 O1 时每次调用都不比）。逐行那一趟判 [`s5_checks_of_a_bad_implementation_row`]：阳性对照照判
/// （它让模型停在翻过的那次调用，是对拍的红）；关掉 O1 不判；驱动没走完整（实现 panic、流水线 panic）的流走自崩那一类，不判；
/// 状态上实现 panic 的流驱动是走完了的，照判（那时没有观测，不适用的态数是 0）。
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

struct CommandLine {
    run_kind: RunKind,
    output_directory: PathBuf,
    switches: RunSwitches,
}

struct KnownBadImagesCommandLine {
    dump_directory: PathBuf,
    output_directory: PathBuf,
    with_e170_bad_rows: bool,
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
        })),
        _ => None,
    }
}

fn run_known_bad_images(command_line: &KnownBadImagesCommandLine) -> i32 {
    let mut lines = ResultLines { emitted: 0 };
    lines.result(&format!(
        "name=config segment=s1 mode=known-bad-images e170_bad_rows={}",
        command_line.with_e170_bad_rows
    ));
    let known_bad_failed = known_bad_images_on_the_first_stream(
        &command_line.dump_directory,
        &command_line.output_directory,
        &mut lines,
    );
    let e170_rows_failed = if command_line.with_e170_bad_rows {
        e170_bad_rows_on_both_streams(
            &command_line.dump_directory,
            &command_line.output_directory,
            &mut lines,
        )
    } else {
        0
    };
    lines.result(&format!(
        "name=verdict known_bad_entries_failing_s4={known_bad_failed} e170_bad_rows_failing_s3_or_s4={e170_rows_failed}"
    ));
    lines.finish();
    if known_bad_failed > 0 || e170_rows_failed > 0 {
        3
    } else {
        0
    }
}

fn run_dump(command_line: &CommandLine) -> i32 {
    let run_kind = command_line.run_kind;
    let switches = command_line.switches;
    let mut lines = ResultLines { emitted: 0 };
    lines.result(&format!(
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
    if run_kind == RunKind::Baseline {
        for check in stream_independent_checks() {
            failed_checks += usize::from(!check.holds());
            lines.result(&format!(
                "name=s1 stream=both check={} local={} crates={} holds={}",
                check.name,
                field_text(&check.local),
                field_text(&check.crates),
                check.holds()
            ));
        }
    }
    let mut totals = [0usize; 7];
    let mut streams_not_exported = 0usize;
    let mut s5_failed = 0usize;
    for name in STREAM_NAMES {
        let started = Instant::now();
        let output_root = command_line.output_directory.clone();
        let exported = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            export_stream(name, &output_root, run_kind, switches)
        }));
        let mut export = exported.unwrap_or_else(|payload| {
            StreamExport::not_exported(
                name,
                format!("pipeline_panicked:{}", panic_text(&*payload)),
                started,
            )
        });
        streams_not_exported += usize::from(export.not_exported.is_some());
        let s5 = s5_checks_to_judge(&export, run_kind, switches);
        s5_failed += s5.iter().filter(|check| !check.holds()).count();
        export.checks.extend(s5);
        let stream_totals = report_stream(&mut lines, &export);
        for (total, value) in totals.iter_mut().zip(stream_totals) {
            *total += value;
        }
    }
    failed_checks += totals[0];
    let [_, rebuilt_digest_mismatches, mask_mismatches, s3_mismatches, red_calls, crash_judge_red, pool_checker_red] =
        totals;
    lines.result(&format!(
        "name=verdict s1_failed_checks={failed_checks} s4_digest_mismatches={rebuilt_digest_mismatches} crash_facts_mask_mismatches={mask_mismatches} s3_o2_judgement_mismatches={s3_mismatches} streams_not_exported={streams_not_exported} o1_comparison_red_calls={red_calls} o1_crash_judge_red_states={crash_judge_red} o2_red_states={pool_checker_red}"
    ));
    lines.finish();
    let stop_clause_hit = failed_checks > 0
        || rebuilt_digest_mismatches > 0
        || s3_mismatches > 0
        || (run_kind == RunKind::Baseline && (s5_failed > 0 || streams_not_exported > 0));
    if stop_clause_hit {
        3
    } else {
        0
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(parsed) = parse_command_line(&arguments) else {
        eprintln!(
            "用法：e171_three_oracle_collusion baseline|row --output-directory <目录> [--positive-control O1-对拍|O1-崩溃判官|O2 | --disable-oracle O1|O2]，或 known-bad-images --dump-directory <基线导出目录> --output-directory <目录> [--e170-bad-rows]；收到 {arguments:?}"
        );
        std::process::exit(2);
    };
    let output_directory = match &parsed {
        ParsedCommand::Dump(command_line) => &command_line.output_directory,
        ParsedCommand::KnownBadImages(command_line) => &command_line.output_directory,
    };
    if output_directory.exists() {
        eprintln!(
            "输出目录 {} 已存在：装置不覆盖已有的文件，换一个目录",
            output_directory.display()
        );
        std::process::exit(2);
    }
    let exit_code = match &parsed {
        ParsedCommand::Dump(command_line) => run_dump(command_line),
        ParsedCommand::KnownBadImages(command_line) => run_known_bad_images(command_line),
    };
    if exit_code != 0 {
        std::process::exit(exit_code);
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
        assert_eq!(LOCAL_SECOND_STREAM_TWO_STATE_CLOSED_FORM, 1_662_648_449);
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
    fn a_one_byte_patch_inside_a_sector_changes_only_that_byte() {
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
    /// （代码轮第三轮判决 K4-c，登记第一节读法 5 末句；装置一侧变异：记红丢掉，这里红）。
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
            .call("acquire", || refused_acquire)
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
    fn a_panic_in_the_acquire_is_an_implementation_crash_and_not_a_comparison_red() {
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

    /// 真池上取号、暖机的绝对值：两次调用都成功、模型都不比（这一步模型不交答）、模型没停、第一次取到的号是 1。
    #[test]
    fn an_acquire_and_a_warm_up_on_a_fresh_pool_succeed_and_the_model_is_not_asked() {
        let mut live = LiveStream::start(
            StreamName::NewPoolFileCreation,
            "e171-unit-acquire-and-warm-up-absolute",
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
            label,
            first_operation: 0,
            return_point: 0,
            outcome,
            model,
        }
    }

    fn disagreement_for_the_test() -> ModelVerdictOfCall {
        ModelVerdictOfCall::Disagrees {
            aspect: "模型说该成、实现拒了",
            model_answer: "往前走".to_string(),
            implementation_answer: "拒了:X".to_string(),
        }
    }

    /// 逐行那一趟的 S5：返回了错误而 O1-对拍没问模型（`not_modelled`）的调用、与不适用来自卸载和对拍不同之外的来源，各自回比 0，
    /// 不是 0 就是停机（代码轮第三轮判决 K4-c 丙）。绝对值：期望写 `0`，实际写不合的个数。
    #[test]
    fn the_row_s5_counts_the_errored_calls_left_unjudged_and_the_not_applicable_states_from_another_source(
    ) {
        let errored = |text: &str| CallOutcome::ReturnedError(text.to_string());
        // 取号出错、O1-对拍没记红（修之前的样子）：第一样 1 处不合；模型停在这次调用、来源不是卸载也不是对拍不同，但没有不适用的态，第二样 0。
        let silent = [recorded_call_for_the_test(
            "acquire",
            errored("X"),
            ModelVerdictOfCall::NotModelled,
        )];
        let silent_acquire_checks = s5_checks_of_a_bad_implementation_row("s", &silent, Some(0), 0);
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
        let recorded_red_checks = s5_checks_of_a_bad_implementation_row("s", &red, Some(0), 9);
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
        let error_after_the_stop_checks =
            s5_checks_of_a_bad_implementation_row("s", &error_after_the_stop, Some(0), 6);
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
        let allowed_refusal_checks =
            s5_checks_of_a_bad_implementation_row("s", &allowed_refusal, Some(2), 5);
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
        let unmount_stop_checks =
            s5_checks_of_a_bad_implementation_row("s", &unmount_stop, Some(1), 7);
        assert!(unmount_stop_checks.iter().all(StopCheck::holds));
        // 模型没停：没有不适用的态，合规。
        let no_stop = [recorded_call_for_the_test(
            "publish",
            CallOutcome::Succeeded,
            ModelVerdictOfCall::Agrees,
        )];
        let no_stop_checks = s5_checks_of_a_bad_implementation_row("s", &no_stop, None, 0);
        assert!(no_stop_checks.iter().all(StopCheck::holds));
    }

    /// 哪一趟判哪一份 S5（代码轮第三轮判决 K4-c 丙）：逐行那一趟判「返回了错误的调用都记了对拍的红」那一份，基线那一趟判原来那一份；
    /// 关掉 O1、驱动没走完整的流不判；基线带阳性对照、流没导出也不判。
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
        };
        assert_eq!(
            s5_checks_to_judge(
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
        };
        assert!(
            s5_checks_to_judge(&export, RunKind::BadImplementationRow, with_the_model_off)
                .is_empty()
        );
        // 驱动没走完整（实现 panic）与流水线 panic（`ending` 是 `None`）的流不判，走自崩那一类。
        let mut crashed = silent_acquire_stopped_by_the_error();
        crashed.ending = Some(StreamEnding::ImplementationCrashed {
            what: "x".to_string(),
        });
        assert!(s5_checks_to_judge(
            &crashed,
            RunKind::BadImplementationRow,
            RunSwitches::default()
        )
        .is_empty());
        crashed.ending = None;
        assert!(s5_checks_to_judge(
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
        let baseline_checks = s5_checks_to_judge(&clean, RunKind::Baseline, RunSwitches::default());
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
        };
        assert!(
            s5_checks_to_judge(&clean, RunKind::Baseline, with_the_pool_checker_control).is_empty()
        );
        clean.not_exported = Some("x".to_string());
        assert!(s5_checks_to_judge(&clean, RunKind::Baseline, RunSwitches::default()).is_empty());
    }

    /// 原用例文件里一个函数的条目名按文件次序取：字面名与常量名都认，更深缩进的行不认。
    #[test]
    fn entry_names_are_read_in_file_order_from_the_twelve_space_indented_lines_of_the_function() {
        let text = "fn known_bad_images(clean: &MemoryPool) -> Vec<(&'static str, Mutation)> {\n    vec![\n        (\n            \"I-7.4\",\n            Box::new(|image| {\n                let x = 1;\n                \"I-9.9\";\n            }),\n        ),\n        (\n            MAPPING_KEY_INVARIANT,\n            Box::new(|image| {}),\n        ),\n    ]\n}\nfn other() {\n            \"I-0.0\",\n}\n";
        assert_eq!(
            entry_names_in_the_original_test_file(text),
            vec![
                "I-7.4".to_string(),
                singlefs_checker::image::MAPPING_KEY_MATCHES_THE_UNIT_HEADER.to_string()
            ]
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
    fn a_patch_list_text_round_trips_through_the_patches_file() {
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
    }
}

//! E175（产品路径上 inode 号水位退回与重发号）的装置。仓副本装置：只放在执行员草稿目录里那份仓副本的
//! `crates/singlefs-harness/tests/` 下，主工作区的 `crates/` 不动（跑前登记 `research/prompts/e175-preregistration.md`
//! 第五节「装置写在哪」）。产品路径 = 一次都不调 `PoolWriter::set_inode_number_watermark_source`、不加 `SharedFaultPlan` 注入。
//!
//! 臂由跑的一方在副本里施加补丁之后用 `E175_ARM` 告诉装置（`P`、`V1`、`V2`、`V717`）；装置不读源码判臂，
//! 施加前后的 `sha256sum` 由跑的一方记进产物。
//!
//! 三样读法（登记第六节）：S1 账 = `recovery::readable_roots` 列可读根、`recovery::rebuild_version` 重建每条根那一版；
//! S2 = `singlefs_checker::walk::check_pool_image`；S3 被崩溃恢复抛弃的根两条路（路 1 现行实例表、路 2 步账）。
//!
//! 弄坏开关（`E175_BREAK`，只给快档自证用，平常跑时不设；跑大规模的用例见到它就拒）：
//!   ledger-ignores-birth      S1 账按号去重、不看出生代        打开后 V1 臂上 `rollback_one_step_then_create_on_v1_reissues_three_on_the_same_timeline` 红
//!   every-root-abandoned      S3 给发号事件分类时每条持有旧出生代的根都判成被崩溃恢复抛弃（Q1 / Q3b 与 Q9 用的那一处；
//!                             择现行那一版与两路比对不动）    打开后同一条红（Q1 由 1 变 0、Q3b 由 0 变 1）
//!   q9-always-still-present   Q9 分类一律判「还在」            打开后 `q9_classification_says_gone_when_no_holder_is_readable` 红
//!
//! 跑大规模（`#[ignore]` 的那一条）读这几个环境变量：`E175_ARM`、`E175_FAMILY`（第一段 D、F3a、F2；第二段 F1 四组
//! `F1-broad`、`F1-reuse-after-raising-the-floor`、`F1-deleting-files-of-several-lower-leaves`、
//! `F1-newest-system-configuration-slot-reading-zeros` 与第八节几何点 `G-long`、`G-small`、`G-draw`、`G-long-F1`）、`E175_RESULT_FILE`、
//! `E175_WORKER_THREADS`、`E175_TIME_BUDGET_SECONDS`（到点不再领新的历史，领了的跑完；同一条命令再跑就是续跑）、
//! 可选 `E175_INDICES`（逗号分隔，只跑这几段：F6 复跑与试跑用）、可选 `E175_INDEX_LIMIT`（只跑序号小于它的：试跑用）。
//!
//! 代码标记（登记第十二节修订 2 第 2 条）：`E175_CODE_BASELINE` = 冻结副本清单 `baseline-frozen-crates.sha` 的 sha256。
//! 大规模那一条在新结果文件的第一行写 `# code_baseline=<值>`（不是 `E7RESULT` 行，复跑比对不看它），续写已有文件时第一行
//! 必须是同一个值，不同就拒（两份代码混用）；没设时写 `not-given`（只给 `scripts/experiment-apparatus/replay.sh` 的 `driver_e175` 复跑用，
//! 产物一律设）。汇总那一条要求每份输入的第一行都是同一个 `code_baseline`、且等于 `E175_CODE_BASELINE`，
//! 汇总文件第一行写同一个文件头，每一行汇总行末尾带 `code_baseline=<值>`。

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::instance_table::InstanceTableRecords;
use singlefs_core::journal::{JournalRecord, JournalRecordPlaceInPublish};
use singlefs_core::recovery::{
    choose_system_configuration, instance_table_of_root, publish_is_abandoned_by_the_instance_table,
    readable_roots, rebuild_version, scan_journal, RebuiltVersion,
};
use singlefs_core::root_record::RootRecord;
use singlefs_harness::history::{
    execute_history_with, AppliedEffect, ContentChoice, ContentLength, FloorTargetChoice,
    GeneratedHistory, GenerationWeights, HistoryDeviceWidth, HistoryEnding, HistoryExecution,
    HistoryOperation, HistoryOperationKind, HistorySeed, HistoryStartingPoint,
    NamedGenerationWeights, OverwriteContentDraw, PerStepChecker, RollbackTargetChoice,
    RollbackTargetDraw, StepObservation, StepOutcome, StepPosition,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::SharedStream;

/// 种子基，抄自 `crates/singlefs-harness/src/history.rs` 第 107 行（登记第五节「装置要读 crates/ 的常量」）；
/// `seed_base_and_first_inode_number_match_the_crates` 拿它与实现侧同名常量回比。
const SEED_BASE_COPIED_FROM_THE_HISTORY_MODULE: u64 = 7_463_871_032_432_355_113;
/// 第一个 inode 号，抄自 `crates/singlefs-core/src/transaction.rs` 第 138 行，同一条用例回比。
const FIRST_INODE_NUMBER_COPIED_FROM_THE_TRANSACTION_MODULE: u64 = 1;
/// D8（核心索引结构） 已定项 6「新池新建文件写」：一条记录（inode 1、出生代 3、……）；记账里一条「inode 号水位 = 2」。
/// 条款给的锚点，不读 `crates/`（登记第七节 A1、A2）。
const FIRST_FILE_INODE_NUMBER_IN_THE_CLAUSE: u64 = 1;
const FIRST_FILE_OBJECT_BIRTH_IN_THE_CLAUSE: u64 = 3;
const WATERMARK_AFTER_THE_FIRST_FILE_IN_THE_CLAUSE: u64 = 2;
/// 被数的两条不变量的编号（`singlefs_checker::image` 里的名字；`invariant_names_match_the_checker` 回比 I-9.29）。
const ONE_OBJECT_BIRTH_PER_INODE_NUMBER_NAME: &str = "I-9.29";
const WATERMARK_ABOVE_BOTH_HIGHEST_NUMBERS_NAME: &str = "I-9.6";

/// F3a 的规模（登记第五节「历史族」）：中段长 0..=4（P 臂）、0..=3（V 臂），字母表 11 个。
const F3A_ALPHABET_SIZE: u64 = 11;
const F3A_LONGEST_MIDDLE_ON_THE_PRODUCT_ARM: u32 = 4;
const F3A_LONGEST_MIDDLE_ON_THE_VARIANT_ARMS: u32 = 3;
/// F2 的规模：P 臂 256 种子 × 60 步，V 臂前 64 个种子。
const F2_SEEDS_ON_THE_PRODUCT_ARM: u64 = 256;
const F2_SEEDS_ON_THE_VARIANT_ARMS: u64 = 64;
const F2_OPERATIONS_PER_HISTORY: usize = 60;
/// F1 的规模（登记第五节 F1 那一行）：每组 P 臂 128 种子 × 30 步，V 臂每组前 32 个种子。
const F1_SEEDS_ON_THE_PRODUCT_ARM: u64 = 128;
const F1_SEEDS_ON_THE_VARIANT_ARMS: u64 = 32;
const F1_OPERATIONS_PER_HISTORY: usize = 30;
/// 第八节几何点的规模：G-long 与 G-long-F1 每段 150 步；G-long P 64 种子、V1 32 种子；G-small、G-draw P 各 64 种子；
/// G-long-F1 P 32 种子。表里写「不跑」的那几格登记 0 段。
const LONG_GEOMETRY_OPERATIONS_PER_HISTORY: usize = 150;
const G_LONG_SEEDS_ON_THE_PRODUCT_ARM: u64 = 64;
const G_LONG_SEEDS_ON_THE_ROLLBACK_VARIANT_ARM: u64 = 32;
const G_SMALL_SEEDS_ON_THE_PRODUCT_ARM: u64 = 64;
const G_DRAW_SEEDS_ON_THE_PRODUCT_ARM: u64 = 64;
const G_LONG_F1_SEEDS_ON_THE_PRODUCT_ARM: u64 = 32;

/// 被测的那一臂（登记第五节「臂」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arm {
    Product,
    RollbackTakesTheTargetsWatermark,
    PublishDoesNotAddTheNewInodes,
    RollbackTakesOnlyTheRingsWatermark,
}

impl Arm {
    fn from_environment() -> Arm {
        let value = std::env::var("E175_ARM")
            .expect("E175_ARM 没设：跑的一方施加补丁之后设 P、V1、V2 或 V717");
        match value.as_str() {
            "P" => Arm::Product,
            "V1" => Arm::RollbackTakesTheTargetsWatermark,
            "V2" => Arm::PublishDoesNotAddTheNewInodes,
            "V717" => Arm::RollbackTakesOnlyTheRingsWatermark,
            other => panic!("E175_ARM 认不得「{other}」：只认 P、V1、V2、V717"),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Arm::Product => "P",
            Arm::RollbackTakesTheTargetsWatermark => "V1",
            Arm::PublishDoesNotAddTheNewInodes => "V2",
            Arm::RollbackTakesOnlyTheRingsWatermark => "V717",
        }
    }
}

/// 弄坏开关（文件头那张表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Breakage {
    ledger_ignores_birth: bool,
    every_root_abandoned: bool,
    q9_always_still_present: bool,
}

impl Breakage {
    const NONE: Breakage = Breakage {
        ledger_ignores_birth: false,
        every_root_abandoned: false,
        q9_always_still_present: false,
    };

    fn from_environment() -> Breakage {
        match std::env::var("E175_BREAK").ok().as_deref() {
            None | Some("") => Breakage::NONE,
            Some("ledger-ignores-birth") => Breakage { ledger_ignores_birth: true, ..Breakage::NONE },
            Some("every-root-abandoned") => Breakage { every_root_abandoned: true, ..Breakage::NONE },
            Some("q9-always-still-present") => {
                Breakage { q9_always_still_present: true, ..Breakage::NONE }
            }
            Some(other) => panic!("E175_BREAK 认不得「{other}」：见文件头那张表"),
        }
    }
}

/// 历史族（登记第五节「历史族」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    /// 定向，钉值用：D1、D2(1..=6)、D3，序号 0 是 D1、1..=6 是 D2(k)、7 是 D3。
    Directed,
    /// 定向穷举：前缀 `[Unlink, CreateInode, CreateInode]`、中段取字母表上的序列、后缀 `[CreateInode, CreateInode]`。
    DirectedExhaustive,
    /// 带回退的随机历史：登记那组比重，种子 = 种子基 + 序号。
    RandomWithRollbacks,
    /// F1 现有随机历史：一组现有命名比重，种子 = 种子基 + 序号，30 步。
    ExistingRandom(ExistingWeightGroup),
    /// 第八节几何敏感性的一个点（G0 就是 F2 本身）。
    Geometry(GeometryPoint),
}

/// F1 的四组现有命名比重（登记第五节 F1 那一行）：都会在 unlink 之后抽 `CreateInode`；另四组不抽它，不跑。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExistingWeightGroup {
    Broad,
    ReuseAfterRaisingTheFloor,
    DeletingFilesOfSeveralLowerLeaves,
    NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
}

impl ExistingWeightGroup {
    const ALL: [ExistingWeightGroup; 4] = [
        ExistingWeightGroup::Broad,
        ExistingWeightGroup::ReuseAfterRaisingTheFloor,
        ExistingWeightGroup::DeletingFilesOfSeveralLowerLeaves,
        ExistingWeightGroup::NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
    ];

    fn weights(self) -> GenerationWeights {
        match self {
            ExistingWeightGroup::Broad => GenerationWeights::BROAD,
            ExistingWeightGroup::ReuseAfterRaisingTheFloor => GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
            ExistingWeightGroup::DeletingFilesOfSeveralLowerLeaves => {
                GenerationWeights::DELETING_FILES_OF_SEVERAL_LOWER_LEAVES
            }
            ExistingWeightGroup::NewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS
            }
        }
    }

    fn label(self) -> &'static str {
        match self {
            ExistingWeightGroup::Broad => "F1-broad",
            ExistingWeightGroup::ReuseAfterRaisingTheFloor => "F1-reuse-after-raising-the-floor",
            ExistingWeightGroup::DeletingFilesOfSeveralLowerLeaves => "F1-deleting-files-of-several-lower-leaves",
            ExistingWeightGroup::NewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                "F1-newest-system-configuration-slot-reading-zeros"
            }
        }
    }
}

/// 第八节几何敏感性的四个点（登记第八节那张表；G0 是第一段的 F2）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GeometryPoint {
    /// G-long：F2 比重，150 步。
    LongerRandomWithRollbacks,
    /// G-small：F2 比重，60 步，`UnitAreaOf384Slots`。
    RandomWithRollbacksOnSmallDevices,
    /// G-draw：F2 比重换成 `RecentUniformOrBeyond` 回退抽法，60 步。
    RandomWithRollbacksDrawingRecentUniformOrBeyond,
    /// G-long-F1：`BROAD`，150 步。
    LongerBroad,
}

impl GeometryPoint {
    const ALL: [GeometryPoint; 4] = [
        GeometryPoint::LongerRandomWithRollbacks,
        GeometryPoint::RandomWithRollbacksOnSmallDevices,
        GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond,
        GeometryPoint::LongerBroad,
    ];

    fn label(self) -> &'static str {
        match self {
            GeometryPoint::LongerRandomWithRollbacks => "G-long",
            GeometryPoint::RandomWithRollbacksOnSmallDevices => "G-small",
            GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond => "G-draw",
            GeometryPoint::LongerBroad => "G-long-F1",
        }
    }
}

impl Family {
    fn from_environment() -> Family {
        let value = std::env::var("E175_FAMILY").expect("E175_FAMILY 没设：见文件头列的族名");
        Family::from_label(&value).unwrap_or_else(|| panic!("E175_FAMILY 认不得「{value}」：见文件头列的族名"))
    }

    /// 每一族，按结果行里的族名次序：D、F3a、F2、F1 四组、几何四点。
    fn every_family() -> Vec<Family> {
        [Family::Directed, Family::DirectedExhaustive, Family::RandomWithRollbacks]
            .into_iter()
            .chain(ExistingWeightGroup::ALL.into_iter().map(Family::ExistingRandom))
            .chain(GeometryPoint::ALL.into_iter().map(Family::Geometry))
            .collect()
    }

    fn from_label(label: &str) -> Option<Family> {
        Family::every_family().into_iter().find(|family| family.label() == label)
    }

    fn label(self) -> &'static str {
        match self {
            Family::Directed => "D",
            Family::DirectedExhaustive => "F3a",
            Family::RandomWithRollbacks => "F2",
            Family::ExistingRandom(group) => group.label(),
            Family::Geometry(point) => point.label(),
        }
    }

    /// 这一族在这一臂上登记的段数。
    fn registered_history_count(self, arm: Arm) -> u64 {
        match (self, arm) {
            (Family::Directed, _) => DIRECTED_HISTORY_COUNT,
            (Family::DirectedExhaustive, Arm::Product) => {
                f3a_sequences_up_to_middle_length(F3A_LONGEST_MIDDLE_ON_THE_PRODUCT_ARM)
            }
            (
                Family::DirectedExhaustive,
                Arm::RollbackTakesTheTargetsWatermark
                | Arm::PublishDoesNotAddTheNewInodes
                | Arm::RollbackTakesOnlyTheRingsWatermark,
            ) => f3a_sequences_up_to_middle_length(F3A_LONGEST_MIDDLE_ON_THE_VARIANT_ARMS),
            (Family::RandomWithRollbacks, Arm::Product) => F2_SEEDS_ON_THE_PRODUCT_ARM,
            (
                Family::RandomWithRollbacks,
                Arm::RollbackTakesTheTargetsWatermark
                | Arm::PublishDoesNotAddTheNewInodes
                | Arm::RollbackTakesOnlyTheRingsWatermark,
            ) => F2_SEEDS_ON_THE_VARIANT_ARMS,
            (Family::ExistingRandom(_), Arm::Product) => F1_SEEDS_ON_THE_PRODUCT_ARM,
            (
                Family::ExistingRandom(_),
                Arm::RollbackTakesTheTargetsWatermark
                | Arm::PublishDoesNotAddTheNewInodes
                | Arm::RollbackTakesOnlyTheRingsWatermark,
            ) => F1_SEEDS_ON_THE_VARIANT_ARMS,
            (Family::Geometry(GeometryPoint::LongerRandomWithRollbacks), Arm::Product) => {
                G_LONG_SEEDS_ON_THE_PRODUCT_ARM
            }
            // V717 有条件跑（Q6 > 0 的族），规模照这一族 V1 那一格。
            (
                Family::Geometry(GeometryPoint::LongerRandomWithRollbacks),
                Arm::RollbackTakesTheTargetsWatermark | Arm::RollbackTakesOnlyTheRingsWatermark,
            ) => G_LONG_SEEDS_ON_THE_ROLLBACK_VARIANT_ARM,
            (Family::Geometry(GeometryPoint::RandomWithRollbacksOnSmallDevices), Arm::Product) => {
                G_SMALL_SEEDS_ON_THE_PRODUCT_ARM
            }
            (Family::Geometry(GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond), Arm::Product) => {
                G_DRAW_SEEDS_ON_THE_PRODUCT_ARM
            }
            (Family::Geometry(GeometryPoint::LongerBroad), Arm::Product) => G_LONG_F1_SEEDS_ON_THE_PRODUCT_ARM,
            // 第八节那张表写「不跑」的格。
            (Family::Geometry(GeometryPoint::LongerRandomWithRollbacks), Arm::PublishDoesNotAddTheNewInodes)
            | (
                Family::Geometry(
                    GeometryPoint::RandomWithRollbacksOnSmallDevices
                    | GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond
                    | GeometryPoint::LongerBroad,
                ),
                Arm::RollbackTakesTheTargetsWatermark
                | Arm::PublishDoesNotAddTheNewInodes
                | Arm::RollbackTakesOnlyTheRingsWatermark,
            ) => 0,
        }
    }

    /// 这一族每段几步（D、F3a 的步数由操作表定，这里不用）。
    fn operations_per_random_history(self) -> Option<usize> {
        match self {
            Family::Directed | Family::DirectedExhaustive => None,
            Family::RandomWithRollbacks
            | Family::Geometry(
                GeometryPoint::RandomWithRollbacksOnSmallDevices
                | GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond,
            ) => Some(F2_OPERATIONS_PER_HISTORY),
            Family::ExistingRandom(_) => Some(F1_OPERATIONS_PER_HISTORY),
            Family::Geometry(GeometryPoint::LongerRandomWithRollbacks | GeometryPoint::LongerBroad) => {
                Some(LONG_GEOMETRY_OPERATIONS_PER_HISTORY)
            }
        }
    }

    /// 这一族跑在什么盘上（登记第五节「每一步怎么跑」；只有 G-small 换盘宽）。
    fn execution(self) -> HistoryExecution {
        match self {
            Family::Geometry(GeometryPoint::RandomWithRollbacksOnSmallDevices) => EXECUTION_ON_SMALL_DEVICES,
            Family::Directed
            | Family::DirectedExhaustive
            | Family::RandomWithRollbacks
            | Family::ExistingRandom(_)
            | Family::Geometry(
                GeometryPoint::LongerRandomWithRollbacks
                | GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond
                | GeometryPoint::LongerBroad,
            ) => EXECUTION,
        }
    }
}

/// D 族：D1 一段、D2(k) 六段（k = 1..=6）、D3 一段。
const DIRECTED_HISTORY_COUNT: u64 = 8;
const LARGEST_DIRECTED_ROLLBACK_INDEX: u64 = 6;

fn ring_root(index_from_newest: u64) -> HistoryOperation {
    HistoryOperation::RollBackWhileMounted(RollbackTargetChoice::RingRoot { index_from_newest })
}

/// D 族第 `index` 段的操作表与名字（登记第五节 D 那一行）。
fn directed_history(index: u64) -> (String, Vec<HistoryOperation>) {
    match index {
        0 => ("D1".to_string(), Vec::new()),
        rollback_index @ 1..=LARGEST_DIRECTED_ROLLBACK_INDEX => (
            format!("D2({rollback_index})"),
            vec![
                HistoryOperation::Unlink,
                HistoryOperation::CreateInode,
                HistoryOperation::CreateInode,
                ring_root(rollback_index),
                HistoryOperation::CreateInode,
            ],
        ),
        7 => (
            "D3".to_string(),
            vec![
                HistoryOperation::Unlink,
                HistoryOperation::CreateInode,
                HistoryOperation::CloseAndMountWritable,
                HistoryOperation::CreateInode,
                HistoryOperation::ColdStartRecover,
                HistoryOperation::CloseAndMountWritable,
                HistoryOperation::CreateInode,
            ],
        ),
        other => panic!("D 族只有 {DIRECTED_HISTORY_COUNT} 段，序号 {other} 越界"),
    }
}

/// F3a 中段字母表（登记第五节，11 个，次序就是编号次序）。
fn f3a_middle_alphabet() -> [HistoryOperation; 11] {
    [
        HistoryOperation::CreateInode,
        ring_root(1),
        ring_root(2),
        ring_root(3),
        ring_root(5),
        ring_root(8),
        HistoryOperation::CloseAndMountWritable,
        HistoryOperation::ColdStartRecover,
        HistoryOperation::RaiseRollbackFloor(FloorTargetChoice { steps_above_current_floor: 1 }),
        HistoryOperation::Unlink,
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed: 5,
        }),
    ]
}

/// 中段长 0..=`longest` 的序列一共几条：Σ 11^L。
fn f3a_sequences_up_to_middle_length(longest: u32) -> u64 {
    (0..=longest).map(|length| F3A_ALPHABET_SIZE.pow(length)).sum()
}

/// F3a 第 `index` 条的中段（字母表编号，按长度从短到长、同长按 11 进制从小到大排）。
fn f3a_middle_letters(index: u64) -> Vec<u64> {
    let mut remaining = index;
    let mut length = 0_u32;
    // 迭代上界：每轮 remaining 至少减 1（11^L ≥ 1），减到落进某一长度那一段就出循环。
    loop {
        let sequences_of_this_length = F3A_ALPHABET_SIZE.pow(length);
        if remaining < sequences_of_this_length {
            break;
        }
        remaining -= sequences_of_this_length;
        length += 1;
    }
    let mut letters_most_significant_first = Vec::new();
    for position in (0..length).rev() {
        let place_value = F3A_ALPHABET_SIZE.pow(position);
        letters_most_significant_first.push(remaining / place_value);
        remaining %= place_value;
    }
    letters_most_significant_first
}

fn f3a_operations(index: u64) -> Vec<HistoryOperation> {
    let alphabet = f3a_middle_alphabet();
    let mut operations = vec![
        HistoryOperation::Unlink,
        HistoryOperation::CreateInode,
        HistoryOperation::CreateInode,
    ];
    for letter in f3a_middle_letters(index) {
        operations.push(alphabet[usize::try_from(letter).expect("字母表编号 < 11")]);
    }
    operations.push(HistoryOperation::CreateInode);
    operations.push(HistoryOperation::CreateInode);
    operations
}

/// F2 的比重（登记第五节那张表；三张操作表各和为 100，`f2_weight_tables_each_sum_to_one_hundred` 钉住）。
static F2_STARTING_POINTS: [(HistoryStartingPoint, u64); 1] = [(HistoryStartingPoint::AfterFirstFile, 1)];
static F2_WITH_SESSION_CLOSED: [(HistoryOperationKind, u64); 2] = [
    (HistoryOperationKind::CloseAndMountWritable, 90),
    (HistoryOperationKind::ColdStartRecover, 10),
];
static F2_WITH_SESSION_OPEN_WITHOUT_FILE: [(HistoryOperationKind, u64); 5] = [
    (HistoryOperationKind::PublishFirstFile, 70),
    (HistoryOperationKind::CloseAndMountWritable, 10),
    (HistoryOperationKind::RollBackWhileMounted, 10),
    (HistoryOperationKind::ColdStartRecover, 5),
    (HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot, 5),
];
static F2_WITH_SESSION_OPEN_WITH_FILE: [(HistoryOperationKind, u64); 7] = [
    (HistoryOperationKind::Unlink, 40),
    (HistoryOperationKind::RollBackWhileMounted, 20),
    (HistoryOperationKind::PublishOverwrite, 15),
    (HistoryOperationKind::CloseAndMountWritable, 10),
    (HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot, 5),
    (HistoryOperationKind::RaiseRollbackFloor, 5),
    (HistoryOperationKind::ColdStartRecover, 5),
];
static F2_AFTER_THE_FILE_WAS_DELETED: [(HistoryOperationKind, u64); 7] = [
    (HistoryOperationKind::CreateInode, 35),
    (HistoryOperationKind::RollBackWhileMounted, 35),
    (HistoryOperationKind::CloseAndMountWritable, 10),
    (HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot, 5),
    (HistoryOperationKind::ColdStartRecover, 5),
    (HistoryOperationKind::RaiseRollbackFloor, 5),
    (HistoryOperationKind::PublishOverwrite, 5),
];

/// `named` 借 `DeletingFilesOfSeveralLowerLeaves` 当标签（登记第五节）：复现靠这张表加种子，不靠环境变量。
const F2_WEIGHTS: GenerationWeights = GenerationWeights {
    name: "E175 F2：偏重回退与建 inode",
    named: NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves,
    starting_points: &F2_STARTING_POINTS,
    with_session_closed: &F2_WITH_SESSION_CLOSED,
    with_session_open_without_file: &F2_WITH_SESSION_OPEN_WITHOUT_FILE,
    with_session_open_with_file: &F2_WITH_SESSION_OPEN_WITH_FILE,
    with_session_open_after_the_file_was_deleted: Some(&F2_AFTER_THE_FILE_WAS_DELETED),
    rollback_targets: RollbackTargetDraw::FourthNewestRootHalfTheTime,
    overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
};

/// G-draw（登记第八节）：F2 那组比重，只把回退目标的抽法换成 `RecentUniformOrBeyond`（偏向最近四条与环外）。
const F2_WEIGHTS_DRAWING_RECENT_UNIFORM_OR_BEYOND: GenerationWeights = GenerationWeights {
    name: "E175 G-draw：偏重回退与建 inode，回退目标照第一版抽法",
    named: NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves,
    starting_points: &F2_STARTING_POINTS,
    with_session_closed: &F2_WITH_SESSION_CLOSED,
    with_session_open_without_file: &F2_WITH_SESSION_OPEN_WITHOUT_FILE,
    with_session_open_with_file: &F2_WITH_SESSION_OPEN_WITH_FILE,
    with_session_open_after_the_file_was_deleted: Some(&F2_AFTER_THE_FILE_WAS_DELETED),
    rollback_targets: RollbackTargetDraw::RecentUniformOrBeyond,
    overwrite_contents: OverwriteContentDraw::WithinOneDataUnitOrJustBeyond,
};

/// 一个随机族的一段：种子 = 种子基 + 序号，标签写种子。
fn random_history_item(family: Family, index: u64, weights: &GenerationWeights) -> HistoryItem {
    let seed = HistorySeed(SEED_BASE_COPIED_FROM_THE_HISTORY_MODULE.wrapping_add(index));
    let operation_count = family.operations_per_random_history().expect("随机族才走这里");
    HistoryItem {
        family,
        index,
        label: format!("seed{}", seed.0),
        history: singlefs_harness::history::generate_history_with_weights(seed, operation_count, weights),
    }
}

/// 一段要跑的历史：族、序号、给人看的标签（D 的名字、F3a 的中段字母、F2 的种子）与历史本身。
struct HistoryItem {
    family: Family,
    index: u64,
    label: String,
    history: GeneratedHistory,
}

fn history_item(family: Family, index: u64) -> HistoryItem {
    let seed_base = SEED_BASE_COPIED_FROM_THE_HISTORY_MODULE;
    match family {
        Family::Directed => {
            let (name, operations) = directed_history(index);
            HistoryItem {
                family,
                index,
                label: name,
                history: GeneratedHistory {
                    seed: HistorySeed(seed_base),
                    starting_point: HistoryStartingPoint::AfterFirstFile,
                    operations,
                },
            }
        }
        Family::DirectedExhaustive => {
            let letters: Vec<String> =
                f3a_middle_letters(index).iter().map(u64::to_string).collect();
            HistoryItem {
                family,
                index,
                label: format!("middle[{}]", letters.join(",")),
                history: GeneratedHistory {
                    seed: HistorySeed(seed_base),
                    starting_point: HistoryStartingPoint::AfterFirstFile,
                    operations: f3a_operations(index),
                },
            }
        }
        Family::RandomWithRollbacks => {
            let seed = HistorySeed(seed_base.wrapping_add(index));
            HistoryItem {
                family,
                index,
                label: format!("seed{}", seed.0),
                history: singlefs_harness::history::generate_history_with_weights(
                    seed,
                    F2_OPERATIONS_PER_HISTORY,
                    &F2_WEIGHTS,
                ),
            }
        }
        Family::ExistingRandom(group) => random_history_item(family, index, &group.weights()),
        Family::Geometry(
            GeometryPoint::LongerRandomWithRollbacks | GeometryPoint::RandomWithRollbacksOnSmallDevices,
        ) => random_history_item(family, index, &F2_WEIGHTS),
        Family::Geometry(GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond) => {
            random_history_item(family, index, &F2_WEIGHTS_DRAWING_RECENT_UNIFORM_OR_BEYOND)
        }
        Family::Geometry(GeometryPoint::LongerBroad) => {
            random_history_item(family, index, &GenerationWeights::BROAD)
        }
    }
}

/// 操作的短写（结果行里的操作表，凭它与族、序号能单独复现那一段）。
fn operation_code(operation: &HistoryOperation) -> String {
    match operation {
        HistoryOperation::PublishFirstFile(content) => format!("first({:?}/{})", content.length, content.fill_seed),
        HistoryOperation::PublishOverwrite(content) => format!("overwrite({:?}/{})", content.length, content.fill_seed),
        HistoryOperation::PublishWithoutUnits => "empty".to_string(),
        HistoryOperation::CloseAndMountWritable => "mount".to_string(),
        HistoryOperation::RollBackWhileMounted(target) => format!("rollback({target:?})"),
        HistoryOperation::RaiseRollbackFloor(choice) => format!("raise({})", choice.steps_above_current_floor),
        HistoryOperation::ColdStartRecover => "cold".to_string(),
        HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => "crash".to_string(),
        HistoryOperation::Unlink => "unlink".to_string(),
        HistoryOperation::CreateInode => "create".to_string(),
        HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(content) => {
            format!("powerloss({:?}/{})", content.length, content.fill_seed)
        }
        HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
            "mount-zero-slot".to_string()
        }
    }
    .replace(' ', "")
}

/// 这一步是不是挂载 / 恢复类（关掉会话再起；Q5 的窗口到它为止，不论结局）。
fn operation_ends_the_session(kind: HistoryOperationKind) -> bool {
    match kind {
        HistoryOperationKind::CloseAndMountWritable
        | HistoryOperationKind::ColdStartRecover
        | HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot
        | HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable
        | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => true,
        HistoryOperationKind::PublishFirstFile
        | HistoryOperationKind::PublishOverwrite
        | HistoryOperationKind::PublishWithoutUnits
        | HistoryOperationKind::RollBackWhileMounted
        | HistoryOperationKind::RaiseRollbackFloor
        | HistoryOperationKind::Unlink
        | HistoryOperationKind::CreateInode => false,
    }
}

/// 一条根的身份：(实例代号, checkpoint_txg)。
type RootIdentity = (u32, u64);

/// S1 对一条根读到的那一版。
#[derive(Clone, Debug, PartialEq, Eq)]
enum VersionFacts {
    /// 树表 0 条：没有 inode 树，也没有记账水位。
    WithoutTrees,
    WithTrees { watermark: u64, inode_births: Vec<(u64, u64)> },
    /// 重建报错（登记第六节 S1：记「读不出」另数，不当成没有记录）。
    Unreadable(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RootFacts {
    identity: RootIdentity,
    rollback_floor: u64,
    version: VersionFacts,
}

impl RootFacts {
    fn watermark(&self) -> Option<u64> {
        match &self.version {
            VersionFacts::WithTrees { watermark, .. } => Some(*watermark),
            VersionFacts::WithoutTrees | VersionFacts::Unreadable(_) => None,
        }
    }

    fn inode_births(&self) -> &[(u64, u64)] {
        match &self.version {
            VersionFacts::WithTrees { inode_births, .. } => inode_births,
            VersionFacts::WithoutTrees | VersionFacts::Unreadable(_) => &[],
        }
    }
}

fn root_identity(root: &RootRecord) -> RootIdentity {
    (root.instance.0, root.checkpoint_txg.0)
}

/// 重建用的那条记录：同实例、同 txg、带「本次发布末条」标志的；没有就取环里计数器最大的那条顶着
/// （与 `mount::record_standing_for_the_version` 同一个取法，重建出来的版本只用它的 jsn 与事务号，这里都不用）。
fn record_standing_for(
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
    root: &RootRecord,
) -> Option<JournalRecord> {
    records
        .values()
        .filter(|record| {
            record.instance == root.instance
                && record.checkpoint_txg == root.checkpoint_txg
                && record.place_in_publish == JournalRecordPlaceInPublish::LastRecordOfThePublish
        })
        .max_by_key(|record| record.counter)
        .or_else(|| records.values().max_by_key(|record| record.counter))
        .cloned()
}

fn version_facts_of(
    image: &MemoryPool,
    root: &RootRecord,
    records: &BTreeMap<(InstanceGeneration, u64), JournalRecord>,
) -> VersionFacts {
    match rebuild_version(image, root, record_standing_for(records, root)) {
        Ok(RebuiltVersion::WithoutFile) => VersionFacts::WithoutTrees,
        Ok(RebuiltVersion::WithFile(version) | RebuiltVersion::WithTreesAndNoFile(version)) => {
            let mut inode_births: Vec<(u64, u64)> = version
                .inode_leaf_containers
                .iter()
                .flat_map(|container| container.contents.records.iter())
                .map(|record| (record.inode, record.object_birth.0))
                .collect();
            inode_births.sort_unstable();
            VersionFacts::WithTrees { watermark: version.inode_number_watermark(), inode_births }
        }
        Err(failure) => VersionFacts::Unreadable(format!("{failure:?}").replace(' ', "")),
    }
}

/// S3 路 1：一条根按现行实例表判是不是被崩溃恢复抛弃的。
fn abandoned_by_the_table(identity: RootIdentity, table: &InstanceTableRecords) -> bool {
    publish_is_abandoned_by_the_instance_table(
        InstanceGeneration(identity.0),
        CheckpointTxg(identity.1),
        table,
    )
}

/// S3 路 2 的步账：每次挂载做成，新实例接的那一版（模型记的所选根 = 那一步之前模型根环里最新的那条）之后、
/// 同一个旧实例里 txg 更大的根记为被抛弃。键是旧实例，值是接的那一版的 txg。
fn abandoned_by_the_step_ledger(identity: RootIdentity, rows: &BTreeMap<u32, u64>) -> bool {
    rows.get(&identity.0).is_some_and(|selected_txg| identity.1 > *selected_txg)
}

/// Q9 的分类（登记第六节 Q9 行）：持有旧出生代的根里有一条这一步可读、不被抛弃、txg ≥ 现行 F ⇒ 还在。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OlderBirthStillInTheCandidateSet {
    Still,
    Gone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HolderAtThisStep {
    readable_now: bool,
    abandoned_now: bool,
    txg: u64,
}

fn classify_q9(
    holders: &[HolderAtThisStep],
    current_rollback_floor: u64,
    breakage: Breakage,
) -> OlderBirthStillInTheCandidateSet {
    if breakage.q9_always_still_present {
        return OlderBirthStillInTheCandidateSet::Still;
    }
    if holders.iter().any(|holder| {
        holder.readable_now && !holder.abandoned_now && holder.txg >= current_rollback_floor
    }) {
        OlderBirthStillInTheCandidateSet::Still
    } else {
        OlderBirthStillInTheCandidateSet::Gone
    }
}

/// 一次发号碰上账里同号的另一个出生代（Q1 或 Q3b 的一个事件）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReissueEvent {
    position: String,
    inode: u64,
    new_birth: u64,
    older_births: Vec<u64>,
    /// 持有旧出生代的根里至少一条不被崩溃恢复抛弃（Q1）；否则是跨被抛弃时间线（Q3b）。
    same_timeline: bool,
    q9: OlderBirthStillInTheCandidateSet,
    /// 同一步 S2 对 I-9.29 的判定（四栏之一）。
    one_birth_per_number_at_this_step: VerdictClass,
}

/// S2 一条不变量在一步上的判定落在哪一栏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VerdictClass {
    Holds,
    NotApplicable,
    PartiallyJudged,
    Violated,
    /// checker 交回的列表里没有这一条（读不到 ≠ 读到 0，另数）。
    Missing,
}

impl VerdictClass {
    fn of(verdict: &InvariantVerdict) -> VerdictClass {
        match verdict {
            InvariantVerdict::Holds => VerdictClass::Holds,
            InvariantVerdict::NotApplicable(_) => VerdictClass::NotApplicable,
            InvariantVerdict::PartiallyJudged(_) => VerdictClass::PartiallyJudged,
            InvariantVerdict::Violated(_) => VerdictClass::Violated,
        }
    }

    fn label(self) -> &'static str {
        match self {
            VerdictClass::Holds => "holds",
            VerdictClass::NotApplicable => "not_applicable",
            VerdictClass::PartiallyJudged => "partially_judged",
            VerdictClass::Violated => "violated",
            VerdictClass::Missing => "missing",
        }
    }
}

/// 四栏加「缺」的步数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct VerdictColumns {
    holds: u64,
    not_applicable: u64,
    partially_judged: u64,
    violated: u64,
    missing: u64,
}

impl VerdictColumns {
    fn count(&mut self, class: VerdictClass) {
        match class {
            VerdictClass::Holds => self.holds += 1,
            VerdictClass::NotApplicable => self.not_applicable += 1,
            VerdictClass::PartiallyJudged => self.partially_judged += 1,
            VerdictClass::Violated => self.violated += 1,
            VerdictClass::Missing => self.missing += 1,
        }
    }

    fn render(&self, prefix: &str) -> String {
        format!(
            "{prefix}_holds={} {prefix}_not_applicable={} {prefix}_partially_judged={} {prefix}_violated_steps={} {prefix}_missing={}",
            self.holds, self.not_applicable, self.partially_judged, self.violated, self.missing
        )
    }
}

/// 一段历史里各量的计数（登记第六节「量」那张表；名字照表里的量号）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct HistoryCounts {
    steps_observed: u64,
    issuances: u64,
    issuances_at_operations: u64,
    q1_same_timeline_reissues: u64,
    q3b_reissues_across_abandoned_timelines: u64,
    q9_still: u64,
    q9_gone: u64,
    q3_regression_steps: u64,
    q3_regression_peak: u64,
    q3_regression_final: u64,
    watermark_peak: u64,
    watermark_final: u64,
    steps_whose_current_version_has_no_watermark: u64,
    q4_rollbacks_across_issued_numbers: u64,
    q5_windows_with_a_creation_after_such_a_rollback: u64,
    q6_rollbacks_whose_ring_maximum_is_below_the_current: u64,
    q6_rollbacks_judged: u64,
    q7_mounts_landing_on_a_lower_watermark: u64,
    q11_creations_not_issuing_the_previous_watermark: u64,
    rollbacks_applied: u64,
    rollback_targets_not_in_the_previous_ring: u64,
    creations_applied: u64,
    mounts_or_recoveries_applied: u64,
    one_birth_per_number: VerdictColumns,
    watermark_above_both_highest_numbers: VerdictColumns,
    s1_root_observations: u64,
    s1_unreadable_root_observations: u64,
    s1_rebuilds: u64,
    s1_same_identity_with_different_facts: u64,
    s3_path_disagreements: u64,
    s3_table_unreadable_steps: u64,
    f3_still_but_one_birth_holds: u64,
    f3_still_but_one_birth_partially_judged: u64,
    f3_violated_without_two_births_in_the_ledger: u64,
    system_configuration_unchosen_steps: u64,
}

/// 一步的样子，留给下一步比（回退、建 inode、挂载那几个量都拿「那一步之前」比）。
#[derive(Clone, Debug)]
struct StepView {
    readable: Vec<RootFacts>,
    abandoned: BTreeMap<RootIdentity, bool>,
    current: Option<RootFacts>,
}

/// 每一步 S2 对两条被数的不变量的判定，D 族的钉值断言要按步看。
#[derive(Clone, Debug, PartialEq, Eq)]
struct StepVerdicts {
    position: String,
    one_birth_per_number: VerdictClass,
    watermark_above_both_highest_numbers: VerdictClass,
}

/// 一次做成的建 inode 发了哪几个号（D 族 A3–A6 按它钉）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct CreationIssue {
    position: String,
    issued_numbers: Vec<u64>,
    previous_watermark: Option<u64>,
}

/// 一段历史的账（S1 账、S3 两路、S2 判定与各量）：执行器每一步之后的观察者喂它。
struct HistoryAccount {
    breakage: Breakage,
    /// 根键（根记录的全部字段）→ 读到的那一版：同一个根键只重建一次。
    root_cache: HashMap<String, RootFacts>,
    /// 号 → 出生代（弄坏开关 ledger-ignores-birth 打开时一律记 0）→ 持有过它的根。
    births: BTreeMap<u64, BTreeMap<u64, BTreeSet<RootIdentity>>>,
    /// 见过的每条根的水位（Q3 的 M 按它取）。
    watermarks_seen: BTreeMap<RootIdentity, Option<u64>>,
    /// S3 路 2 的步账（[`abandoned_by_the_step_ledger`]）。
    rows_by_steps: BTreeMap<u32, u64>,
    previous: Option<StepView>,
    previous_model_newest: Option<RootIdentity>,
    open_q5_windows: u64,
    counts: HistoryCounts,
    events: Vec<ReissueEvent>,
    first_violation: Option<String>,
    first_s3_disagreement: Option<String>,
    step_verdicts: Vec<StepVerdicts>,
    creation_issues: Vec<CreationIssue>,
    first_q11_detail: Option<String>,
    /// 每一步之后现行那一版的水位与 inode 树（D 族 A1、A2 按起点那一格钉）。
    current_trajectory: Vec<(String, Option<u64>, Vec<(u64, u64)>)>,
    /// 每次做成的回退：目标那一版的身份与 S1 读到的水位（D 族 A5 k ≥ 2 那几格按它钉）。
    rollback_targets: Vec<(RootIdentity, Option<u64>)>,
}

/// 这一步读到的盘面（S1）与 S3 两路的判定。
struct ReadStep {
    readable: Vec<RootFacts>,
    table: Option<InstanceTableRecords>,
}

impl HistoryAccount {
    fn new(breakage: Breakage) -> HistoryAccount {
        HistoryAccount {
            breakage,
            root_cache: HashMap::new(),
            births: BTreeMap::new(),
            watermarks_seen: BTreeMap::new(),
            rows_by_steps: BTreeMap::new(),
            previous: None,
            previous_model_newest: None,
            open_q5_windows: 0,
            counts: HistoryCounts::default(),
            events: Vec::new(),
            first_violation: None,
            first_s3_disagreement: None,
            step_verdicts: Vec::new(),
            creation_issues: Vec::new(),
            first_q11_detail: None,
            current_trajectory: Vec::new(),
            rollback_targets: Vec::new(),
        }
    }

    /// S1：列可读根、按根键缓存重建、按 (txg, 实例) 升序排、同一身份只留一条；再读最新那条根的实例表（S3 路 1 用）。
    fn read_step(&mut self, image: &MemoryPool) -> Option<ReadStep> {
        let Ok(system_configuration) = choose_system_configuration(image) else {
            self.counts.system_configuration_unchosen_steps += 1;
            return None;
        };
        let roots = readable_roots(
            image,
            &system_configuration.immutable.region_devices,
            &system_configuration.immutable.sizes,
            &system_configuration.immutable.filesystem_identifier,
        );
        let mut journal_records: Option<BTreeMap<(InstanceGeneration, u64), JournalRecord>> = None;
        let mut by_identity: BTreeMap<(u64, u32), (RootFacts, RootRecord)> = BTreeMap::new();
        for root in &roots {
            let key = format!("{root:?}");
            let facts = if let Some(cached) = self.root_cache.get(&key) {
                cached.clone()
            } else {
                let records = journal_records
                    .get_or_insert_with(|| scan_journal(image, &system_configuration));
                let facts = RootFacts {
                    identity: root_identity(root),
                    rollback_floor: root.rollback_floor.0,
                    version: version_facts_of(image, root, records),
                };
                self.counts.s1_rebuilds += 1;
                self.root_cache.insert(key, facts.clone());
                facts
            };
            let order = (facts.identity.1, facts.identity.0);
            match by_identity.get(&order) {
                Some((kept, _)) if *kept != facts => self.counts.s1_same_identity_with_different_facts += 1,
                Some(_) => {}
                None => {
                    by_identity.insert(order, (facts, *root));
                }
            }
        }
        let table = by_identity
            .values()
            .last()
            .and_then(|(_, newest)| instance_table_of_root(image, newest));
        if table.is_none() && !by_identity.is_empty() {
            self.counts.s3_table_unreadable_steps += 1;
        }
        let readable: Vec<RootFacts> = by_identity.into_values().map(|(facts, _)| facts).collect();
        self.counts.s1_root_observations += u64::try_from(readable.len()).expect("根环至多几十条");
        self.counts.s1_unreadable_root_observations += u64::try_from(
            readable
                .iter()
                .filter(|facts| matches!(facts.version, VersionFacts::Unreadable(_)))
                .count(),
        )
        .expect("根环至多几十条");
        Some(ReadStep { readable, table })
    }

    /// S3 路 1（现行实例表）。读不出实例表时按不被抛弃算（步数另记）。
    fn abandoned_by_path_one(&self, identity: RootIdentity, table: Option<&InstanceTableRecords>) -> bool {
        table.is_some_and(|table| abandoned_by_the_table(identity, table))
    }

    fn abandoned_by_path_two(&self, identity: RootIdentity) -> bool {
        abandoned_by_the_step_ledger(identity, &self.rows_by_steps)
    }

    /// 给发号事件分类（Q1 / Q3b、Q9）时持有旧出生代的根按 S3 判；弄坏开关 every-root-abandoned 打开时每条都判被抛弃。
    fn holder_abandoned(&self, identity: RootIdentity, table: Option<&InstanceTableRecords>) -> bool {
        self.breakage.every_root_abandoned || self.abandoned_by_path_one(identity, table)
    }
}

impl HistoryAccount {
    /// 执行器每一步之后（起点之后也一次）调它。
    fn observe(&mut self, observation: &StepObservation<'_>) {
        let position = match observation.position {
            StepPosition::StartingPoint => "start".to_string(),
            StepPosition::Operation(step_index) => format!("op{step_index}"),
        };
        self.counts.steps_observed += 1;
        let applied_effect = match observation.outcome {
            Some(StepOutcome::Applied(effect)) => Some(effect),
            Some(StepOutcome::Refused { .. } | StepOutcome::NotApplicable(_)) | None => None,
        };
        // 路 2 的步账先记：这一步做成的挂载接的是那一步之前模型根环里最新的那条根。
        if let (Some(AppliedEffect::Mounted { .. }), Some(chosen)) =
            (applied_effect, self.previous_model_newest)
        {
            self.rows_by_steps.insert(chosen.0, chosen.1);
        }
        let model_newest = observation
            .model
            .committed_versions()
            .iter()
            .map(|(key, _)| key)
            .max()
            .map(|key| {
                (
                    u32::try_from(key.instance.0).expect("模型的实例代号来自盘上 4 字节"),
                    key.checkpoint_txg.0,
                )
            });
        let Some(read) = self.read_step(observation.image) else {
            self.previous_model_newest = model_newest;
            return;
        };
        let table = read.table.as_ref();
        let mut abandoned: BTreeMap<RootIdentity, bool> = BTreeMap::new();
        for facts in &read.readable {
            let by_path_one = self.abandoned_by_path_one(facts.identity, table);
            if table.is_some() && by_path_one != self.abandoned_by_path_two(facts.identity) {
                self.counts.s3_path_disagreements += 1;
                if self.first_s3_disagreement.is_none() {
                    self.first_s3_disagreement = Some(format!(
                        "{position}:root{:?}:path1={by_path_one}:rows={:?}",
                        facts.identity, self.rows_by_steps
                    ));
                }
            }
            abandoned.insert(facts.identity, by_path_one);
            self.watermarks_seen.insert(facts.identity, facts.watermark());
        }
        let current = read
            .readable
            .iter()
            .rev()
            .find(|facts| !abandoned[&facts.identity])
            .cloned();
        let current_rollback_floor = current.as_ref().map_or(0, |facts| facts.rollback_floor);
        self.current_trajectory.push((
            position.clone(),
            current.as_ref().and_then(RootFacts::watermark),
            current.as_ref().map(|facts| facts.inode_births().to_vec()).unwrap_or_default(),
        ));
        let readable_identities: BTreeSet<RootIdentity> =
            read.readable.iter().map(|facts| facts.identity).collect();
        let mut events_of_this_step: Vec<ReissueEvent> = Vec::new();
        for facts in &read.readable {
            for &(inode, birth) in facts.inode_births() {
                let birth_key = if self.breakage.ledger_ignores_birth { 0 } else { birth };
                let older: Vec<(u64, BTreeSet<RootIdentity>)> = self
                    .births
                    .get(&inode)
                    .map(|births| {
                        births
                            .iter()
                            .filter(|(older_birth, _)| **older_birth != birth_key)
                            .map(|(older_birth, holders)| (*older_birth, holders.clone()))
                            .collect()
                    })
                    .unwrap_or_default();
                let is_new = !self
                    .births
                    .get(&inode)
                    .is_some_and(|births| births.contains_key(&birth_key));
                if is_new {
                    self.counts.issuances += 1;
                    if matches!(observation.position, StepPosition::Operation(_)) {
                        self.counts.issuances_at_operations += 1;
                    }
                    if !older.is_empty() {
                        let holders: Vec<RootIdentity> =
                            older.iter().flat_map(|(_, holders)| holders.iter().copied()).collect();
                        let same_timeline = holders
                            .iter()
                            .any(|holder| !self.holder_abandoned(*holder, table));
                        let views: Vec<HolderAtThisStep> = holders
                            .iter()
                            .map(|holder| HolderAtThisStep {
                                readable_now: readable_identities.contains(holder),
                                abandoned_now: self.holder_abandoned(*holder, table),
                                txg: holder.1,
                            })
                            .collect();
                        events_of_this_step.push(ReissueEvent {
                            position: position.clone(),
                            inode,
                            new_birth: birth,
                            older_births: older.iter().map(|(older_birth, _)| *older_birth).collect(),
                            same_timeline,
                            q9: classify_q9(&views, current_rollback_floor, self.breakage),
                            one_birth_per_number_at_this_step: VerdictClass::Missing,
                        });
                    }
                }
                self.births
                    .entry(inode)
                    .or_default()
                    .entry(birth_key)
                    .or_default()
                    .insert(facts.identity);
            }
        }
        self.judge_with_the_checker(observation.image, &position, &mut events_of_this_step);
        self.count_watermark_regression(current.as_ref(), table);
        self.count_operation_specific(observation, applied_effect, &position, current.as_ref());
        self.events.extend(events_of_this_step);
        self.previous = Some(StepView { readable: read.readable, abandoned, current });
        self.previous_model_newest = model_newest;
    }
}

impl HistoryAccount {
    /// S2：这一步的镜像交给池级 checker；数 I-9.29、I-9.6 的四栏，记第一个 Violated，核第十节 F3 两形。
    fn judge_with_the_checker(
        &mut self,
        image: &MemoryPool,
        position: &str,
        events_of_this_step: &mut [ReissueEvent],
    ) {
        let mut one_birth_per_number = VerdictClass::Missing;
        let mut watermark_above_both_highest_numbers = VerdictClass::Missing;
        for (invariant, verdict) in check_pool_image(image) {
            if let InvariantVerdict::Violated(detail) = &verdict {
                if self.first_violation.is_none() {
                    let first_line = detail.lines().next().unwrap_or("").replace(' ', "_");
                    self.first_violation = Some(format!("{position}:{invariant}:{first_line}"));
                }
            }
            if invariant == ONE_OBJECT_BIRTH_PER_INODE_NUMBER_NAME {
                one_birth_per_number = VerdictClass::of(&verdict);
            }
            if invariant == WATERMARK_ABOVE_BOTH_HIGHEST_NUMBERS_NAME {
                watermark_above_both_highest_numbers = VerdictClass::of(&verdict);
            }
        }
        self.counts.one_birth_per_number.count(one_birth_per_number);
        self.counts
            .watermark_above_both_highest_numbers
            .count(watermark_above_both_highest_numbers);
        for event in events_of_this_step.iter_mut() {
            event.one_birth_per_number_at_this_step = one_birth_per_number;
            if event.same_timeline && event.q9 == OlderBirthStillInTheCandidateSet::Still {
                match one_birth_per_number {
                    VerdictClass::Holds => self.counts.f3_still_but_one_birth_holds += 1,
                    VerdictClass::PartiallyJudged => {
                        self.counts.f3_still_but_one_birth_partially_judged += 1;
                    }
                    VerdictClass::NotApplicable | VerdictClass::Violated | VerdictClass::Missing => {}
                }
            }
        }
        if one_birth_per_number == VerdictClass::Violated
            && !self.births.values().any(|births| births.len() >= 2)
        {
            self.counts.f3_violated_without_two_births_in_the_ledger += 1;
        }
        self.step_verdicts.push(StepVerdicts {
            position: position.to_string(),
            one_birth_per_number,
            watermark_above_both_highest_numbers,
        });
    }

    /// Q3：现行那一版的水位 W 与到这一步为止不被抛弃的根的水位最大值 M 比，深度 = M − W（W ≥ M 记 0）。
    fn count_watermark_regression(&mut self, current: Option<&RootFacts>, table: Option<&InstanceTableRecords>) {
        let Some(watermark) = current.and_then(RootFacts::watermark) else {
            self.counts.steps_whose_current_version_has_no_watermark += 1;
            return;
        };
        let highest_seen = self
            .watermarks_seen
            .iter()
            .filter(|(identity, _)| !self.abandoned_by_path_one(**identity, table))
            .filter_map(|(_, seen)| *seen)
            .max()
            .unwrap_or(watermark);
        let depth = highest_seen.saturating_sub(watermark);
        if depth > 0 {
            self.counts.q3_regression_steps += 1;
        }
        self.counts.q3_regression_peak = self.counts.q3_regression_peak.max(depth);
        self.counts.q3_regression_final = depth;
        self.counts.watermark_peak = self.counts.watermark_peak.max(watermark);
        self.counts.watermark_final = watermark;
    }

    /// 拿「那一步之前」比的几个量：Q4、Q5、Q6（回退）、Q11（建 inode）、Q7（挂载 / 恢复）。
    fn count_operation_specific(
        &mut self,
        observation: &StepObservation<'_>,
        applied_effect: Option<&AppliedEffect>,
        position: &str,
        current: Option<&RootFacts>,
    ) {
        let Some(operation) = observation.operation else { return };
        let previous_current_watermark = self
            .previous
            .as_ref()
            .and_then(|previous| previous.current.as_ref())
            .and_then(RootFacts::watermark);
        match applied_effect {
            Some(AppliedEffect::RolledBack { target, .. }) => {
                self.counts.rollbacks_applied += 1;
                let target_identity = (target.instance.0, target.checkpoint_txg.0);
                let previous_readable = self.previous.as_ref().map_or(&[][..], |previous| &previous.readable[..]);
                let target_watermark = match previous_readable.iter().find(|facts| facts.identity == target_identity) {
                    Some(facts) => facts.watermark(),
                    None => {
                        self.counts.rollback_targets_not_in_the_previous_ring += 1;
                        None
                    }
                };
                self.rollback_targets.push((target_identity, target_watermark));
                if let (Some(target_watermark), Some(before)) = (target_watermark, previous_current_watermark) {
                    if target_watermark < before {
                        self.counts.q4_rollbacks_across_issued_numbers += 1;
                        self.open_q5_windows += 1;
                    }
                }
                if let Some(before) = previous_current_watermark {
                    self.counts.q6_rollbacks_judged += 1;
                    let ring_highest = previous_readable.iter().filter_map(RootFacts::watermark).max();
                    if ring_highest.is_some_and(|ring_highest| ring_highest < before) {
                        self.counts.q6_rollbacks_whose_ring_maximum_is_below_the_current += 1;
                    }
                }
            }
            Some(AppliedEffect::Mounted { .. } | AppliedEffect::Recovered { .. }) => {
                self.counts.mounts_or_recoveries_applied += 1;
                let previous_highest = self.previous.as_ref().and_then(|previous| {
                    previous
                        .readable
                        .iter()
                        .filter(|facts| !previous.abandoned.get(&facts.identity).copied().unwrap_or(false))
                        .filter_map(RootFacts::watermark)
                        .max()
                });
                if let (Some(after), Some(previous_highest)) = (current.and_then(RootFacts::watermark), previous_highest) {
                    if after < previous_highest {
                        self.counts.q7_mounts_landing_on_a_lower_watermark += 1;
                    }
                }
            }
            Some(AppliedEffect::Published { .. } | AppliedEffect::RaisedFloor { .. }) | None => {}
        }
        if operation.kind() == HistoryOperationKind::CreateInode && applied_effect.is_some() {
            self.counts.creations_applied += 1;
            let before: BTreeSet<(u64, u64)> = self
                .previous
                .as_ref()
                .and_then(|previous| previous.current.as_ref())
                .map(|facts| facts.inode_births().iter().copied().collect())
                .unwrap_or_default();
            let issued: Vec<(u64, u64)> = current
                .map(|facts| facts.inode_births().iter().copied().filter(|pair| !before.contains(pair)).collect())
                .unwrap_or_default();
            let issued_numbers: Vec<u64> = issued.iter().map(|(inode, _)| *inode).collect();
            if issued_numbers.len() != 1 || Some(issued_numbers[0]) != previous_current_watermark {
                self.counts.q11_creations_not_issuing_the_previous_watermark += 1;
                if self.first_q11_detail.is_none() {
                    self.first_q11_detail = Some(format!(
                        "{position}:issued={issued:?}:previous_watermark={previous_current_watermark:?}"
                    ).replace(' ', ""));
                }
            }
            self.creation_issues.push(CreationIssue {
                position: position.to_string(),
                issued_numbers,
                previous_watermark: previous_current_watermark,
            });
            self.counts.q5_windows_with_a_creation_after_such_a_rollback += self.open_q5_windows;
            self.open_q5_windows = 0;
        }
        if operation_ends_the_session(operation.kind()) {
            self.open_q5_windows = 0;
        }
    }
}

/// 一段历史跑完的结果：账、结局与挂钟（挂钟在装置进程里自己计，登记第五节「预计挂钟」）。
struct HistoryEvaluation {
    family: Family,
    index: u64,
    label: String,
    operations: Vec<HistoryOperation>,
    ending: String,
    account: HistoryAccount,
    elapsed: Duration,
}

/// 执行器的结局写成一个不带空格的词：跑完、已知红第几条、新发现（第几步、什么签名）。
fn ending_label(ending: &HistoryEnding) -> String {
    match ending {
        HistoryEnding::Completed => "completed".to_string(),
        HistoryEnding::KnownRed { form, observation } => {
            format!("known_red:{form}:{:?}", observation.position)
        }
        HistoryEnding::NewFinding { signature, observation } => {
            format!("new_finding:{:?}:{signature:?}", observation.position)
        }
    }
    .chars()
    .map(|character| if character.is_whitespace() { '_' } else { character })
    .collect()
}

/// 登记第五节「每一步怎么跑、怎么看」：4 GiB 的盘、判空间准入、执行器自己的 checker 不跑（装置在观察者里自己调、判红不停）。
const EXECUTION: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::FourGibibytes,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

/// 登记第八节 G-small：同上，只把盘换成单元区 384 槽的小盘。
const EXECUTION_ON_SMALL_DEVICES: HistoryExecution = HistoryExecution {
    per_step_checker: PerStepChecker::Skipped,
    device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
    space_admission: SpaceAdmission::JudgedByTheFormula,
};

fn evaluate_history(item: &HistoryItem, breakage: Breakage) -> HistoryEvaluation {
    let started = Instant::now();
    let mut account = HistoryAccount::new(breakage);
    let execution = item.family.execution();
    let run = execute_history_with(&item.history, execution, &SharedStream::new(), &mut |observation| {
        account.observe(observation);
    });
    HistoryEvaluation {
        family: item.family,
        index: item.index,
        label: item.label.clone(),
        operations: item.history.operations.clone(),
        ending: ending_label(&run.ending),
        account,
        elapsed: started.elapsed(),
    }
}

impl HistoryEvaluation {
    fn same_timeline_reissues(&self) -> u64 {
        u64::try_from(self.account.events.iter().filter(|event| event.same_timeline).count())
            .expect("事件数装得进 u64")
    }

    fn reissues_across_abandoned_timelines(&self) -> u64 {
        u64::try_from(self.account.events.iter().filter(|event| !event.same_timeline).count())
            .expect("事件数装得进 u64")
    }

    fn q9_count(&self, class: OlderBirthStillInTheCandidateSet) -> u64 {
        u64::try_from(
            self.account
                .events
                .iter()
                .filter(|event| event.same_timeline && event.q9 == class)
                .count(),
        )
        .expect("事件数装得进 u64")
    }

    /// 结果行（一段历史一行，末尾 `line_end=ok`；D 族不带挂钟，它的产物要逐字节复跑）。
    fn history_line(&self, arm: Arm) -> String {
        let counts = &self.account.counts;
        let operations: Vec<String> = self.operations.iter().map(operation_code).collect();
        let timing = match self.family {
            Family::Directed => String::new(),
            Family::DirectedExhaustive
            | Family::RandomWithRollbacks
            | Family::ExistingRandom(_)
            | Family::Geometry(_) => {
                format!(" elapsed_ns={}", self.elapsed.as_nanos())
            }
        };
        format!(
            "E7RESULT name=history family={} arm={} index={} label={} ending={} steps_observed={} issuances={} issuances_at_operations={} \
q1={} q3b={} q9_still={} q9_gone={} q2_one_birth_violated={} {} {} \
q3_steps={} q3_peak={} q3_final={} watermark_peak={} watermark_final={} no_watermark_steps={} \
q4={} q5={} q6={} q6_judged={} q7={} q10_issuances_at_operations={} q11={} \
rollbacks_applied={} rollback_targets_not_in_previous_ring={} creations_applied={} mounts_or_recoveries_applied={} \
s1_root_observations={} s1_unreadable={} s1_rebuilds={} s1_identity_conflicts={} s3_disagreements={} s3_table_unreadable_steps={} \
f3_still_but_holds={} f3_still_but_partial={} f3_violated_without_two_births={} system_configuration_unchosen_steps={} \
first_violation={} first_s3_disagreement={} first_q11={} operations=[{}]{} line_end=ok",
            self.family.label(),
            arm.label(),
            self.index,
            self.label,
            self.ending,
            counts.steps_observed,
            counts.issuances,
            counts.issuances_at_operations,
            self.same_timeline_reissues(),
            self.reissues_across_abandoned_timelines(),
            self.q9_count(OlderBirthStillInTheCandidateSet::Still),
            self.q9_count(OlderBirthStillInTheCandidateSet::Gone),
            u64::from(counts.one_birth_per_number.violated > 0),
            counts.one_birth_per_number.render("i9_29"),
            counts.watermark_above_both_highest_numbers.render("i9_6"),
            counts.q3_regression_steps,
            counts.q3_regression_peak,
            counts.q3_regression_final,
            counts.watermark_peak,
            counts.watermark_final,
            counts.steps_whose_current_version_has_no_watermark,
            counts.q4_rollbacks_across_issued_numbers,
            counts.q5_windows_with_a_creation_after_such_a_rollback,
            counts.q6_rollbacks_whose_ring_maximum_is_below_the_current,
            counts.q6_rollbacks_judged,
            counts.q7_mounts_landing_on_a_lower_watermark,
            counts.issuances_at_operations,
            counts.q11_creations_not_issuing_the_previous_watermark,
            counts.rollbacks_applied,
            counts.rollback_targets_not_in_the_previous_ring,
            counts.creations_applied,
            counts.mounts_or_recoveries_applied,
            counts.s1_root_observations,
            counts.s1_unreadable_root_observations,
            counts.s1_rebuilds,
            counts.s1_same_identity_with_different_facts,
            counts.s3_path_disagreements,
            counts.s3_table_unreadable_steps,
            counts.f3_still_but_one_birth_holds,
            counts.f3_still_but_one_birth_partially_judged,
            counts.f3_violated_without_two_births_in_the_ledger,
            counts.system_configuration_unchosen_steps,
            self.account.first_violation.as_deref().unwrap_or("none"),
            self.account.first_s3_disagreement.as_deref().unwrap_or("none"),
            self.account.first_q11_detail.as_deref().unwrap_or("none"),
            operations.join(","),
            timing,
        )
    }

    /// 事件行：每个 Q1 / Q3b 事件一行，排在它那段历史的结果行前面。
    fn event_lines(&self, arm: Arm) -> Vec<String> {
        self.account
            .events
            .iter()
            .map(|event| {
                let older: Vec<String> = event.older_births.iter().map(u64::to_string).collect();
                format!(
                    "E7RESULT name=event family={} arm={} index={} kind={} position={} inode={} new_birth={} older_births=[{}] q9={:?} i9_29_at_step={}",
                    self.family.label(),
                    arm.label(),
                    self.index,
                    if event.same_timeline { "q1_same_timeline" } else { "q3b_across_abandoned" },
                    event.position,
                    event.inode,
                    event.new_birth,
                    older.join(","),
                    event.q9,
                    event.one_birth_per_number_at_this_step.label(),
                )
            })
            .collect()
    }
}

/// 按区间切片并行跑（登记第五节「跑法」）：工作线程按序号递增领活，到点（`deadline`）就不再领；领了的跑完。
/// 领到的序号恒是一段前缀，合并按序号次序交给 `emit`，与单线程逐字相同（`one_thread_and_three_threads_give_the_same_lines` 对拍）。
fn evaluate_items(
    items: &[HistoryItem],
    worker_threads: usize,
    deadline: Option<Instant>,
    breakage: Breakage,
    mut emit: impl FnMut(HistoryEvaluation),
) {
    assert!(worker_threads >= 1, "至少一个工作线程");
    let next_position = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel::<(usize, HistoryEvaluation)>();
    std::thread::scope(|scope| {
        for _ in 0..worker_threads {
            let sender = sender.clone();
            let next_position = &next_position;
            scope.spawn(move || loop {
                if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                    break;
                }
                let position = next_position.fetch_add(1, Ordering::SeqCst);
                let Some(item) = items.get(position) else { break };
                let evaluation = evaluate_history(item, breakage);
                if sender.send((position, evaluation)).is_err() {
                    break;
                }
            });
        }
        drop(sender);
        let mut pending: BTreeMap<usize, HistoryEvaluation> = BTreeMap::new();
        let mut next_to_emit = 0_usize;
        for (position, evaluation) in receiver {
            pending.insert(position, evaluation);
            while let Some(evaluation) = pending.remove(&next_to_emit) {
                emit(evaluation);
                next_to_emit += 1;
            }
        }
        assert!(
            pending.is_empty(),
            "领到的序号应是一段前缀，合并之后不该剩下：剩 {:?}（某个工作线程 panic 了？）",
            pending.keys().collect::<Vec<_>>()
        );
    });
}

fn required_environment(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} 没设（文件头列了跑大规模要的环境变量）"))
}

fn positive_number_from_environment(name: &str) -> u64 {
    let value = required_environment(name);
    let number: u64 = value.parse().unwrap_or_else(|_| panic!("{name}=「{value}」不是正整数"));
    assert!(number > 0, "{name} 要是正整数，收到 0");
    number
}

/// 结果文件里已有的、写完整了的结果行（带 `line_end=ok`）的序号：续跑跳过它们。
fn indices_already_in(path: &std::path::Path, family: Family, arm: Arm) -> BTreeSet<u64> {
    let Ok(text) = std::fs::read_to_string(path) else { return BTreeSet::new() };
    let family_field = format!(" family={} ", family.label());
    let arm_field = format!(" arm={} ", arm.label());
    text.lines()
        .filter(|line| line.starts_with("E7RESULT name=history ") && line.ends_with(" line_end=ok"))
        .filter(|line| line.contains(&family_field) && line.contains(&arm_field))
        .filter_map(|line| {
            line.split(' ')
                .find_map(|field| field.strip_prefix("index="))
                .and_then(|index| index.parse().ok())
        })
        .collect()
}

/// 结果文件与汇总文件的第一行（文件头），不是 `E7RESULT` 行。
fn code_baseline_header(code_baseline: &str) -> String {
    format!("# code_baseline={code_baseline}")
}

/// 一份结果文件第一行记的代码标记；第一行不是文件头时交回 `None`。
fn code_baseline_of_result_text(text: &str) -> Option<&str> {
    text.lines().next().and_then(|first_line| first_line.strip_prefix("# code_baseline="))
}

/// 几份结果文件的代码标记必须全有、全相同（登记第十一节「两份代码混用」）；交回那一个值，否则交回说明。
fn single_code_baseline_of_result_texts(texts: &[(String, String)]) -> Result<String, String> {
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (path, text) in texts {
        let Some(code_baseline) = code_baseline_of_result_text(text) else {
            return Err(format!("{path} 第一行不是 `# code_baseline=` 文件头"));
        };
        seen.entry(code_baseline.to_string()).or_default().push(path.clone());
    }
    match seen.len() {
        1 => Ok(seen.into_keys().next().expect("len 是 1")),
        0 => Err("没有输入".to_string()),
        _ => Err(format!("输入来自不同的代码：{seen:?}")),
    }
}

/// 大规模那一条用的代码标记：没设时 `not-given`（文件头写明）。
fn code_baseline_from_environment_or_not_given() -> String {
    std::env::var("E175_CODE_BASELINE").unwrap_or_else(|_| "not-given".to_string())
}

#[test]
#[ignore = "大规模：跑登记的一族一臂，参数取环境变量（文件头）；快档不跑"]
fn run_one_family_on_one_arm_into_the_result_file() {
    let breakage = Breakage::from_environment();
    assert_eq!(breakage, Breakage::NONE, "大规模不许开弄坏开关（E175_BREAK 只给快档自证用）");
    let arm = Arm::from_environment();
    let family = Family::from_environment();
    let path = std::path::PathBuf::from(required_environment("E175_RESULT_FILE"));
    let worker_threads = usize::try_from(positive_number_from_environment("E175_WORKER_THREADS"))
        .expect("线程数装得进 usize");
    let budget = Duration::from_secs(positive_number_from_environment("E175_TIME_BUDGET_SECONDS"));
    let registered = family.registered_history_count(arm);
    assert!(
        registered > 0,
        "{} 在 {} 臂上登记 0 段（登记第八节那张表写「不跑」的格）",
        family.label(),
        arm.label()
    );
    let explicit_indices: Option<Vec<u64>> = std::env::var("E175_INDICES").ok().map(|list| {
        list.split(',').map(|index| index.trim().parse().expect("E175_INDICES 是逗号分隔的序号")).collect()
    });
    let limit = std::env::var("E175_INDEX_LIMIT")
        .ok()
        .map(|limit| limit.parse::<u64>().expect("E175_INDEX_LIMIT 是正整数"));
    let wanted: Vec<u64> = match &explicit_indices {
        Some(indices) => indices.clone(),
        None => (0..limit.map_or(registered, |limit| limit.min(registered))).collect(),
    };
    assert!(wanted.iter().all(|index| *index < registered), "序号越过这一族在这一臂登记的 {registered} 段");
    let done = indices_already_in(&path, family, arm);
    let items: Vec<HistoryItem> = wanted
        .iter()
        .filter(|index| !done.contains(index))
        .map(|index| history_item(family, *index))
        .collect();
    let existing_text = std::fs::read_to_string(&path).unwrap_or_default();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .unwrap_or_else(|error| panic!("开不了结果文件 {}：{error}", path.display()));
    let code_baseline = code_baseline_from_environment_or_not_given();
    if existing_text.is_empty() {
        file.write_all(format!("{}\n", code_baseline_header(&code_baseline)).as_bytes()).expect("写结果文件");
    } else {
        assert_eq!(
            code_baseline_of_result_text(&existing_text),
            Some(code_baseline.as_str()),
            "续写的结果文件 {} 第一行的代码标记与这一次的 E175_CODE_BASELINE 不同（两份代码混用）",
            path.display()
        );
    }
    if !existing_text.is_empty() && !existing_text.ends_with('\n') {
        // 上一趟写到一半被打断的那一行没有 line_end=ok，续跑不认它；补一个换行把它隔开。
        file.write_all(b"\n").expect("写结果文件");
    }
    let started = Instant::now();
    let mut evaluated = 0_u64;
    evaluate_items(&items, worker_threads, Some(started + budget), Breakage::NONE, |evaluation| {
        let mut block = String::new();
        for line in evaluation.event_lines(arm) {
            block.push_str(&line);
            block.push('\n');
        }
        block.push_str(&evaluation.history_line(arm));
        block.push('\n');
        file.write_all(block.as_bytes()).expect("写结果文件");
        file.flush().expect("刷结果文件");
        evaluated += 1;
    });
    let now_done = indices_already_in(&path, family, arm);
    let remaining = wanted.iter().filter(|index| !now_done.contains(index)).count();
    let chunk_line = format!(
        "E7TIMING name=chunk family={} arm={} worker_threads={worker_threads} evaluated={evaluated} remaining={remaining} elapsed_ns={}\n",
        family.label(),
        arm.label(),
        started.elapsed().as_nanos()
    );
    file.write_all(chunk_line.as_bytes()).expect("写结果文件");
    let complete_marker = format!("E7RESULT name=family_complete family={} arm={} ", family.label(), arm.label());
    let whole_family = explicit_indices.is_none() && limit.is_none_or(|limit| limit >= registered);
    if whole_family && remaining == 0 && !existing_text.contains(&complete_marker) {
        let line = format!("{complete_marker}histories={}\n", now_done.len());
        file.write_all(line.as_bytes()).expect("写结果文件");
    }
    println!("E175 {} {} evaluated={evaluated} remaining={remaining}", family.label(), arm.label());
}

/// 结果行里一个字段的值。
fn field_of<'line>(line: &'line str, name: &str) -> Option<&'line str> {
    let prefix = format!("{name}=");
    line.split(' ').find_map(|field| field.strip_prefix(prefix.as_str()))
}

/// 不进汇总求和的字段：身份与标签、挂钟（挂钟另起一行）。
const FIELDS_NOT_SUMMED: [&str; 3] = ["index", "elapsed_ns", "label"];

/// 汇总（登记第五节「产物」的汇总表）：按（族、臂）收结果行，每个数值字段报合计与「大于 0 的段数」，`*_peak` 另报最大值；
/// 同一（族、臂、序号）出现两行时内容相同记重复、不同记冲突（登记第十一节「合并不确定」作废那一条）；没写完的行另数。
fn summarize(text: &str) -> Vec<String> {
    let mut groups: BTreeMap<(String, String), BTreeMap<u64, String>> = BTreeMap::new();
    let mut duplicates: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut conflicts: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut events: BTreeMap<(String, String, String), u64> = BTreeMap::new();
    let mut complete: BTreeSet<(String, String)> = BTreeSet::new();
    let mut unfinished_lines = 0_u64;
    for line in text.lines() {
        let family = field_of(line, "family").unwrap_or("").to_string();
        let arm = field_of(line, "arm").unwrap_or("").to_string();
        if line.starts_with("E7RESULT name=family_complete ") {
            complete.insert((family, arm));
        } else if line.starts_with("E7RESULT name=event ") {
            let kind = field_of(line, "kind").unwrap_or("").to_string();
            *events.entry((family, arm, kind)).or_default() += 1;
        } else if line.starts_with("E7RESULT name=history ") {
            if !line.ends_with(" line_end=ok") {
                unfinished_lines += 1;
                continue;
            }
            let index: u64 = field_of(line, "index").and_then(|index| index.parse().ok()).expect("结果行带序号");
            let without_timing: String = line
                .split(' ')
                .filter(|field| !field.starts_with("elapsed_ns="))
                .collect::<Vec<_>>()
                .join(" ");
            let key = (family, arm);
            let group = groups.entry(key.clone()).or_default();
            match group.get(&index) {
                Some(kept) if *kept == without_timing => *duplicates.entry(key).or_default() += 1,
                Some(_) => *conflicts.entry(key).or_default() += 1,
                None => {
                    group.insert(index, without_timing);
                }
            }
        } else if line.starts_with("E7RESULT ") && !line.starts_with("E7RESULT name=done ") {
            unfinished_lines += 1;
        }
    }
    let mut output = Vec::new();
    for ((family, arm), lines) in &groups {
        let mut sums: BTreeMap<String, u64> = BTreeMap::new();
        let mut positive_segments: BTreeMap<String, u64> = BTreeMap::new();
        let mut peaks: BTreeMap<String, u64> = BTreeMap::new();
        let mut endings: BTreeMap<String, u64> = BTreeMap::new();
        let mut segments_issuing_at_least_two = 0_u64;
        for line in lines.values() {
            let ending = field_of(line, "ending").unwrap_or("").split(':').next().unwrap_or("").to_string();
            *endings.entry(ending).or_default() += 1;
            if field_of(line, "issuances_at_operations").and_then(|value| value.parse::<u64>().ok()).unwrap_or(0) >= 2 {
                segments_issuing_at_least_two += 1;
            }
            for field in line.split(' ').skip(1) {
                let Some((name, value)) = field.split_once('=') else { continue };
                if FIELDS_NOT_SUMMED.contains(&name) {
                    continue;
                }
                let Ok(number) = value.parse::<u64>() else { continue };
                *sums.entry(name.to_string()).or_default() += number;
                if number > 0 {
                    *positive_segments.entry(name.to_string()).or_default() += 1;
                }
                if name.ends_with("_peak") {
                    let peak = peaks.entry(name.to_string()).or_default();
                    *peak = (*peak).max(number);
                }
            }
        }
        let key = (family.clone(), arm.clone());
        let mut line = format!(
            "E7RESULT name=summary family={family} arm={arm} histories={} complete={} duplicates={} conflicts={} q10_segments_issuing_at_least_two={segments_issuing_at_least_two}",
            lines.len(),
            if complete.contains(&key) { "yes" } else { "no" },
            duplicates.get(&key).copied().unwrap_or(0),
            conflicts.get(&key).copied().unwrap_or(0),
        );
        for (ending, count) in &endings {
            line.push_str(&format!(" ending_{ending}={count}"));
        }
        for (name, sum) in &sums {
            line.push_str(&format!(" sum_{name}={sum} segments_{name}={}", positive_segments.get(name).copied().unwrap_or(0)));
        }
        for (name, peak) in &peaks {
            line.push_str(&format!(" max_{name}={peak}"));
        }
        output.push(line);
    }
    for ((family, arm, kind), count) in &events {
        output.push(format!("E7RESULT name=event_count family={family} arm={arm} kind={kind} events={count}"));
    }
    output.push(format!("E7RESULT name=unfinished_lines count={unfinished_lines}"));
    let emitted = output.len() + 1;
    output.push(format!("E7RESULT name=done emitted={emitted}"));
    output
}

#[test]
#[ignore = "汇总：读 E175_SUMMARY_INPUTS（逗号分隔的结果文件）写到 E175_SUMMARY_FILE；快档不跑"]
fn summarize_the_result_files() {
    let mut text = String::new();
    let mut texts: Vec<(String, String)> = Vec::new();
    for path in required_environment("E175_SUMMARY_INPUTS").split(',') {
        let file_text = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("读不了 {path}：{error}"));
        text.push_str(&file_text);
        if !text.ends_with('\n') {
            text.push('\n');
        }
        texts.push((path.to_string(), file_text));
    }
    let code_baseline = single_code_baseline_of_result_texts(&texts).unwrap_or_else(|reason| panic!("{reason}"));
    assert_eq!(code_baseline, required_environment("E175_CODE_BASELINE"), "输入的代码标记与 E175_CODE_BASELINE 不同");
    let mut output = code_baseline_header(&code_baseline) + "\n";
    for line in summarize(&text) {
        output.push_str(&format!("{line} code_baseline={code_baseline}\n"));
    }
    let target = required_environment("E175_SUMMARY_FILE");
    assert!(!std::path::Path::new(&target).exists(), "{target} 已存在：汇总写新文件，不覆盖");
    std::fs::write(&target, output).expect("写汇总文件");
}

// ───────────── 快档：常量回比、规模、纯函数、D 族钉值（A1–A7）、多线程与单线程对拍 ─────────────

#[test]
fn result_files_from_two_code_baselines_or_without_a_header_are_refused_by_the_summary() {
    let header_a = code_baseline_header("aaaa") + "\nE7RESULT name=history family=F2 arm=P index=0 line_end=ok\n";
    let header_b = code_baseline_header("bbbb") + "\nE7RESULT name=history family=F2 arm=P index=1 line_end=ok\n";
    let same = [("a1".to_string(), header_a.clone()), ("a2".to_string(), header_a.clone())];
    assert_eq!(single_code_baseline_of_result_texts(&same), Ok("aaaa".to_string()));
    let mixed = [("a1".to_string(), header_a.clone()), ("b1".to_string(), header_b)];
    assert!(single_code_baseline_of_result_texts(&mixed).is_err(), "两份代码混用必须拒");
    let headless = [("a1".to_string(), header_a), ("bare".to_string(), "E7RESULT name=history\n".to_string())];
    assert!(single_code_baseline_of_result_texts(&headless).is_err(), "没有文件头必须拒");
}

#[test]
fn seed_base_and_first_inode_number_match_the_crates() {
    assert_eq!(
        SEED_BASE_COPIED_FROM_THE_HISTORY_MODULE,
        singlefs_harness::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        "本地抄的种子基与 history.rs 第 107 行不同"
    );
    assert_eq!(
        FIRST_INODE_NUMBER_COPIED_FROM_THE_TRANSACTION_MODULE,
        singlefs_core::transaction::FIRST_INODE_NUMBER,
        "本地抄的第一个 inode 号与 transaction.rs 第 138 行不同"
    );
}

#[test]
fn invariant_names_match_the_checker() {
    assert_eq!(
        ONE_OBJECT_BIRTH_PER_INODE_NUMBER_NAME,
        singlefs_checker::image::ONE_OBJECT_BIRTH_PER_INODE_NUMBER
    );
}

/// A7：F3a P 臂 16 105 段、V 臂 1 464 段、P 臂步数 143 335（登记第七节那一行的命令输出 `16105 1464 143335`）。
#[test]
fn f3a_scale_matches_the_registered_counts() {
    assert_eq!(Family::DirectedExhaustive.registered_history_count(Arm::Product), 16_105);
    assert_eq!(
        Family::DirectedExhaustive.registered_history_count(Arm::RollbackTakesTheTargetsWatermark),
        1_464
    );
    let product_steps: u64 = (0..16_105_u64)
        .map(|index| u64::try_from(f3a_operations(index).len()).expect("几步"))
        .sum();
    assert_eq!(product_steps, 143_335);
    assert_eq!(f3a_middle_letters(0), Vec::<u64>::new());
    assert_eq!(f3a_middle_letters(1), vec![0]);
    assert_eq!(f3a_middle_letters(11), vec![10]);
    assert_eq!(f3a_middle_letters(12), vec![0, 0]);
    assert_eq!(f3a_middle_letters(16_104), vec![10, 10, 10, 10]);
}

#[test]
fn f2_weight_tables_each_sum_to_one_hundred_and_generate_sixty_steps() {
    for table in [
        &F2_WITH_SESSION_OPEN_WITHOUT_FILE[..],
        &F2_WITH_SESSION_OPEN_WITH_FILE[..],
        &F2_AFTER_THE_FILE_WAS_DELETED[..],
        &F2_WITH_SESSION_CLOSED[..],
    ] {
        assert_eq!(table.iter().map(|(_, weight)| weight).sum::<u64>(), 100);
    }
    let item = history_item(Family::RandomWithRollbacks, 0);
    assert_eq!(item.history.operations.len(), F2_OPERATIONS_PER_HISTORY);
    assert_eq!(item.history.starting_point, HistoryStartingPoint::AfterFirstFile);
}

/// V5 的靶：持有旧出生代的根已不在可读根里 ⇒ 已转出；一条可读、不被抛弃、在 F 之上 ⇒ 还在。
#[test]
fn q9_classification_says_gone_when_no_holder_is_readable() {
    let breakage = Breakage::from_environment();
    let gone = [HolderAtThisStep { readable_now: false, abandoned_now: false, txg: 9 }];
    assert_eq!(classify_q9(&gone, 0, breakage), OlderBirthStillInTheCandidateSet::Gone);
    let below_the_floor = [HolderAtThisStep { readable_now: true, abandoned_now: false, txg: 4 }];
    assert_eq!(classify_q9(&below_the_floor, 5, breakage), OlderBirthStillInTheCandidateSet::Gone);
    let still = [HolderAtThisStep { readable_now: true, abandoned_now: false, txg: 5 }];
    assert_eq!(classify_q9(&still, 5, breakage), OlderBirthStillInTheCandidateSet::Still);
}

fn issued_numbers(evaluation: &HistoryEvaluation) -> Vec<Vec<u64>> {
    evaluation
        .account
        .creation_issues
        .iter()
        .map(|issue| issue.issued_numbers.clone())
        .collect()
}

/// A1、A2：第一个文件之后（D1 的起点），S1 读到的现行那一版 inode 树恰好 {(1, 3)}、水位恰好 2（四条臂都成立：
/// V1 只改回退、V2 只在有新建 inode 的发布上少加）。
#[test]
fn first_file_alone_reads_inode_one_born_at_three_and_watermark_two() {
    let evaluation = evaluate_history(&history_item(Family::Directed, 0), Breakage::from_environment());
    assert_eq!(evaluation.ending, "completed");
    let (position, watermark, births) = &evaluation.account.current_trajectory[0];
    assert_eq!(position, "start");
    assert_eq!(
        births,
        &vec![(FIRST_FILE_INODE_NUMBER_IN_THE_CLAUSE, FIRST_FILE_OBJECT_BIRTH_IN_THE_CLAUSE)],
        "A1"
    );
    assert_eq!(*watermark, Some(WATERMARK_AFTER_THE_FIRST_FILE_IN_THE_CLAUSE), "A2");
    assert_eq!(evaluation.account.counts.issuances, 1, "起点只发了 inode 1");
}

/// D2(k) 在这一臂上的钉值（A3–A6）：一条带参数的函数，k = 1..=6 各一条用例。
fn rollback_then_create_matches_the_arm(rollback_index: u64) {
    let arm = Arm::from_environment();
    let evaluation =
        evaluate_history(&history_item(Family::Directed, rollback_index), Breakage::from_environment());
    let account = &evaluation.account;
    let first = WATERMARK_AFTER_THE_FIRST_FILE_IN_THE_CLAUSE;
    let issued = issued_numbers(&evaluation);
    println!(
        "D2({rollback_index}) arm={} ending={} issued={issued:?} rollbacks_applied={} rollback_targets={:?} q1={} q3b={}",
        arm.label(),
        evaluation.ending,
        account.counts.rollbacks_applied,
        account.rollback_targets,
        evaluation.same_timeline_reissues(),
        evaluation.reissues_across_abandoned_timelines()
    );
    assert!(issued.len() >= 2, "D2({rollback_index}) 前两次建 inode 都要做成：{issued:?}");
    match arm {
        Arm::Product | Arm::RollbackTakesOnlyTheRingsWatermark | Arm::RollbackTakesTheTargetsWatermark => {
            assert_eq!(issued[..2], [vec![first], vec![first + 1]], "A4：前两次发 2、3");
        }
        Arm::PublishDoesNotAddTheNewInodes => {
            assert_eq!(issued[..2], [vec![first], vec![first]], "A6：水位不加新建个数，两次都发 2");
        }
    }
    let rollback_applied = account.counts.rollbacks_applied == 1;
    if rollback_index == 1 {
        assert!(rollback_applied, "D2(1) 回退到第一次建 inode 那一版，要做成");
    }
    if rollback_index == 1 {
        assert_eq!(issued.len(), 3, "D2(1) 回退之后那次建 inode 要做成（目标那一版第一个文件已删完）：{issued:?}");
    }
    // 回退到第一个文件还活着的那一版时，之后那次建 inode 前提不满足、执行器不调（只在第一个文件已删完的一版上建），
    // A3 / A5 这一格没有样本；只在它做成时钉。
    if rollback_applied && issued.len() == 3 {
        match arm {
            Arm::Product | Arm::RollbackTakesOnlyTheRingsWatermark => {
                assert_eq!(issued[2], vec![first + 2], "A3：回退取 max，不退回去，发 4");
            }
            Arm::RollbackTakesTheTargetsWatermark => {
                let target_watermark = account.rollback_targets[0]
                    .1
                    .expect("回退目标那一版 S1 读得到水位");
                assert_eq!(issued[2], vec![target_watermark], "A5：发的号 = 目标那一版的水位");
                if rollback_index == 1 {
                    assert_eq!(target_watermark, first + 1, "A5：目标是第一次建 inode 那一版，水位 3");
                }
            }
            Arm::PublishDoesNotAddTheNewInodes => {
                assert_eq!(issued[2], vec![first], "水位一直是 2");
            }
        }
    }
    match arm {
        Arm::Product | Arm::RollbackTakesOnlyTheRingsWatermark => {
            assert_eq!(evaluation.same_timeline_reissues(), 0, "P 臂上 D2({rollback_index}) 不该重发号");
        }
        Arm::RollbackTakesTheTargetsWatermark => {
            if rollback_index == 1 {
                assert_eq!(evaluation.same_timeline_reissues(), 1, "A5：Q1 恰好 1");
                assert_eq!(evaluation.reissues_across_abandoned_timelines(), 0, "A5：Q3b 恰好 0");
                let event = account.events.iter().find(|event| event.same_timeline).expect("Q1 那个事件");
                assert_eq!((event.inode, event.position.as_str()), (first + 1, "op4"), "A5：第三次建发的 3");
                assert_eq!(event.q9, OlderBirthStillInTheCandidateSet::Still, "A5：Q9 判「还在」");
                assert_eq!(event.one_birth_per_number_at_this_step, VerdictClass::Violated, "A5：那一步 I-9.29 红");
            }
        }
        Arm::PublishDoesNotAddTheNewInodes => {
            if rollback_index == 1 {
                let at_second_creation = account
                    .events
                    .iter()
                    .filter(|event| event.same_timeline && event.position == "op2")
                    .count();
                assert_eq!(at_second_creation, 1, "A6：第二次建那一步 Q1 恰好 1");
                let after_first_creation = account
                    .step_verdicts
                    .iter()
                    .find(|verdicts| verdicts.position == "op1")
                    .expect("第一次建之后那一步有判定");
                assert_eq!(
                    after_first_creation.watermark_above_both_highest_numbers,
                    VerdictClass::Violated,
                    "A6：第一次建之后那一步 I-9.6 红"
                );
            }
        }
    }
}

#[test]
fn rollback_one_step_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(1);
}
#[test]
fn rollback_two_steps_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(2);
}
#[test]
fn rollback_three_steps_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(3);
}
#[test]
fn rollback_four_steps_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(4);
}
#[test]
fn rollback_five_steps_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(5);
}
#[test]
fn rollback_six_steps_then_create_matches_the_arm() {
    rollback_then_create_matches_the_arm(6);
}

/// A4（D3）：建、挂载、建、冷启动、挂载、建：P、V1、V717 依次发 2、3、4；V2 三次都发 2。
#[test]
fn creations_across_mounts_and_a_cold_start_match_the_arm() {
    let arm = Arm::from_environment();
    let evaluation = evaluate_history(&history_item(Family::Directed, 7), Breakage::from_environment());
    let first = WATERMARK_AFTER_THE_FIRST_FILE_IN_THE_CLAUSE;
    let issued = issued_numbers(&evaluation);
    println!("D3 arm={} ending={} issued={issued:?}", arm.label(), evaluation.ending);
    let expected = match arm {
        Arm::Product | Arm::RollbackTakesTheTargetsWatermark | Arm::RollbackTakesOnlyTheRingsWatermark => {
            vec![vec![first], vec![first + 1], vec![first + 2]]
        }
        Arm::PublishDoesNotAddTheNewInodes => vec![vec![first], vec![first], vec![first]],
    };
    assert_eq!(issued, expected, "A4（D3）");
    assert!(evaluation.account.counts.mounts_or_recoveries_applied >= 2, "两次挂载要做成");
}

/// 多线程与单线程对拍（登记第十一节「合并不确定」）：F3a 前 12 段，1 个线程与 3 个线程的结果行（去掉挂钟）逐字相同。
#[test]
fn one_thread_and_three_threads_give_the_same_lines() {
    let items: Vec<HistoryItem> = (0..12).map(|index| history_item(Family::DirectedExhaustive, index)).collect();
    let lines_with = |worker_threads: usize| {
        let mut lines = Vec::new();
        evaluate_items(&items, worker_threads, None, Breakage::NONE, |evaluation| {
            let line = evaluation.history_line(Arm::Product);
            lines.push(
                line.split(' ').filter(|field| !field.starts_with("elapsed_ns=")).collect::<Vec<_>>().join(" "),
            );
            lines.extend(evaluation.event_lines(Arm::Product));
        });
        lines
    };
    assert_eq!(lines_with(1), lines_with(3));
}

/// 汇总的判法：同一（族、臂、序号）两行内容相同记重复、不同记冲突，没写完的行另数，`done` 行的条数把自己算进去。
#[test]
fn summary_counts_duplicates_conflicts_and_unfinished_lines() {
    let text = "\
E7RESULT name=history family=F2 arm=P index=0 label=a q1=0 q3_peak=2 elapsed_ns=5 line_end=ok
E7RESULT name=history family=F2 arm=P index=0 label=a q1=0 q3_peak=2 elapsed_ns=9 line_end=ok
E7RESULT name=history family=F2 arm=P index=1 label=b q1=3 q3_peak=1 elapsed_ns=5 line_end=ok
E7RESULT name=history family=F2 arm=P index=1 label=b q1=4 q3_peak=1 elapsed_ns=5 line_end=ok
E7RESULT name=history family=F2 arm=P index=2 label=c q1=0
E7RESULT name=event family=F2 arm=P index=1 kind=q1_same_timeline
E7RESULT name=family_complete family=F2 arm=P histories=2
";
    let output = summarize(text);
    assert_eq!(
        output[0],
        "E7RESULT name=summary family=F2 arm=P histories=2 complete=yes duplicates=1 conflicts=1 q10_segments_issuing_at_least_two=0 ending_=2 sum_q1=3 segments_q1=1 sum_q3_peak=3 segments_q3_peak=2 max_q3_peak=2"
    );
    assert_eq!(output[1], "E7RESULT name=event_count family=F2 arm=P kind=q1_same_timeline events=1");
    assert_eq!(output[2], "E7RESULT name=unfinished_lines count=1");
    assert_eq!(output[3], "E7RESULT name=done emitted=4");
    assert_eq!(output.len(), 4);
}

// ───────────── 快档：第二段（F1 四组、第八节几何四点）的规模、比重与每族一段的取样 ─────────────

/// 第二段的段数与步数（登记第五节 F1 那一行、第八节那张表）：
/// `python3 -c "print(4*128*30, 4*32*30, 64*150+64*60+64*60+32*150, 32*150)"` 输出 `15360 3840 22080 4800`。
#[test]
fn second_segment_scales_match_the_registered_counts() {
    let product = Arm::Product;
    let rollback_variant = Arm::RollbackTakesTheTargetsWatermark;
    let publish_variant = Arm::PublishDoesNotAddTheNewInodes;
    let steps_on = |families: &[Family], arm: Arm| -> u64 {
        families
            .iter()
            .map(|family| {
                let per_history = family.operations_per_random_history().expect("随机族");
                family.registered_history_count(arm) * u64::try_from(per_history).expect("几步")
            })
            .sum()
    };
    let existing: Vec<Family> = ExistingWeightGroup::ALL.into_iter().map(Family::ExistingRandom).collect();
    let geometry: Vec<Family> = GeometryPoint::ALL.into_iter().map(Family::Geometry).collect();
    for family in &existing {
        assert_eq!(family.registered_history_count(product), 128, "{}", family.label());
        assert_eq!(family.registered_history_count(rollback_variant), 32, "{}", family.label());
        assert_eq!(family.registered_history_count(publish_variant), 32, "{}", family.label());
    }
    let counts = |point: GeometryPoint| {
        let family = Family::Geometry(point);
        (
            family.registered_history_count(product),
            family.registered_history_count(rollback_variant),
            family.registered_history_count(publish_variant),
        )
    };
    assert_eq!(counts(GeometryPoint::LongerRandomWithRollbacks), (64, 32, 0), "G-long");
    assert_eq!(counts(GeometryPoint::RandomWithRollbacksOnSmallDevices), (64, 0, 0), "G-small");
    assert_eq!(counts(GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond), (64, 0, 0), "G-draw");
    assert_eq!(counts(GeometryPoint::LongerBroad), (32, 0, 0), "G-long-F1");
    assert_eq!(steps_on(&existing, product), 15_360);
    assert_eq!(steps_on(&existing, rollback_variant), 3_840);
    assert_eq!(steps_on(&existing, publish_variant), 3_840);
    assert_eq!(steps_on(&geometry, product), 22_080);
    assert_eq!(steps_on(&geometry, rollback_variant), 4_800);
    for family in existing.iter().chain(geometry.iter()) {
        let item = history_item(*family, 0);
        assert_eq!(
            Some(item.history.operations.len()),
            family.operations_per_random_history(),
            "{} 序号 0 生成的步数",
            family.label()
        );
    }
}

/// F1 四组就是 `crates/` 的四组命名比重，都在 unlink 之后抽 `CreateInode`；几何点只改登记第八节点名的那一个旋钮；族名各不相同、反查得回。
#[test]
fn second_segment_families_change_only_the_registered_knob() {
    let named = [
        NamedGenerationWeights::Broad,
        NamedGenerationWeights::ReuseAfterRaisingTheFloor,
        NamedGenerationWeights::DeletingFilesOfSeveralLowerLeaves,
        NamedGenerationWeights::NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
    ];
    for (group, named) in ExistingWeightGroup::ALL.into_iter().zip(named) {
        assert_eq!(group.weights(), named.weights(), "{}", group.label());
        let after_deletion = group
            .weights()
            .with_session_open_after_the_file_was_deleted
            .expect("F1 四组都有「删完之后」那张表");
        assert!(
            after_deletion.iter().any(|(kind, weight)| *kind == HistoryOperationKind::CreateInode && *weight > 0),
            "{} 要抽 CreateInode",
            group.label()
        );
    }
    let drawing = F2_WEIGHTS_DRAWING_RECENT_UNIFORM_OR_BEYOND;
    assert_eq!(drawing.rollback_targets, RollbackTargetDraw::RecentUniformOrBeyond);
    assert_ne!(F2_WEIGHTS.rollback_targets, drawing.rollback_targets);
    assert_eq!(drawing.starting_points, F2_WEIGHTS.starting_points);
    assert_eq!(drawing.with_session_closed, F2_WEIGHTS.with_session_closed);
    assert_eq!(drawing.with_session_open_without_file, F2_WEIGHTS.with_session_open_without_file);
    assert_eq!(drawing.with_session_open_with_file, F2_WEIGHTS.with_session_open_with_file);
    assert_eq!(
        drawing.with_session_open_after_the_file_was_deleted,
        F2_WEIGHTS.with_session_open_after_the_file_was_deleted
    );
    assert_eq!(drawing.overwrite_contents, F2_WEIGHTS.overwrite_contents);
    for family in Family::every_family() {
        let expected_width = if family == Family::Geometry(GeometryPoint::RandomWithRollbacksOnSmallDevices) {
            HistoryDeviceWidth::UnitAreaOf384Slots
        } else {
            HistoryDeviceWidth::FourGibibytes
        };
        assert_eq!(family.execution().device_width, expected_width, "{}", family.label());
        assert_eq!(family.execution().per_step_checker, PerStepChecker::Skipped);
        assert_eq!(family.execution().space_admission, SpaceAdmission::JudgedByTheFormula);
        assert_eq!(Family::from_label(family.label()), Some(family));
    }
    let labels: BTreeSet<&str> = Family::every_family().into_iter().map(Family::label).collect();
    assert_eq!(labels.len(), 11);
    assert_eq!(Family::from_label("F1"), None);
}

/// 第二段每一族在序号 0 上走一遍（变异取样第四类：快档的取样点要覆盖大规模会跑到的每一族），结果行写得完整、两条被数的不变量每一步都有判定。
/// 一条带参数的函数，每族一条用例。
fn first_history_of_the_family_yields_a_complete_line(family: Family) {
    let arm = Arm::from_environment();
    let item = history_item(family, 0);
    let evaluation = evaluate_history(&item, Breakage::NONE);
    let counts = &evaluation.account.counts;
    println!(
        "{} arm={} ending={} steps_observed={} issuances={} creations_applied={} rollbacks_applied={} q1={} q11={} s3_disagreements={}",
        family.label(),
        arm.label(),
        evaluation.ending,
        counts.steps_observed,
        counts.issuances,
        counts.creations_applied,
        counts.rollbacks_applied,
        evaluation.same_timeline_reissues(),
        counts.q11_creations_not_issuing_the_previous_watermark,
        counts.s3_path_disagreements
    );
    assert!(evaluation.history_line(arm).ends_with(" line_end=ok"));
    assert!(counts.steps_observed >= 1, "起点那一步要观察到");
    assert_eq!(counts.one_birth_per_number.missing, 0, "I-9.29 每一步都要有判定");
    assert_eq!(counts.watermark_above_both_highest_numbers.missing, 0, "I-9.6 每一步都要有判定");
}

#[test]
fn first_history_of_broad_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::ExistingRandom(ExistingWeightGroup::Broad));
}
#[test]
fn first_history_of_reuse_after_raising_the_floor_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::ExistingRandom(
        ExistingWeightGroup::ReuseAfterRaisingTheFloor,
    ));
}
#[test]
fn first_history_of_deleting_files_of_several_lower_leaves_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::ExistingRandom(
        ExistingWeightGroup::DeletingFilesOfSeveralLowerLeaves,
    ));
}
#[test]
fn first_history_of_newest_system_configuration_slot_reading_zeros_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::ExistingRandom(
        ExistingWeightGroup::NewestSystemConfigurationSlotOfEachDeviceReadingZeros,
    ));
}
#[test]
fn first_history_of_g_long_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::Geometry(GeometryPoint::LongerRandomWithRollbacks));
}
#[test]
fn first_history_of_g_small_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::Geometry(
        GeometryPoint::RandomWithRollbacksOnSmallDevices,
    ));
}
#[test]
fn first_history_of_g_draw_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::Geometry(
        GeometryPoint::RandomWithRollbacksDrawingRecentUniformOrBeyond,
    ));
}
#[test]
fn first_history_of_g_long_f1_yields_a_complete_line() {
    first_history_of_the_family_yields_a_complete_line(Family::Geometry(GeometryPoint::LongerBroad));
}

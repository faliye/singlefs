//! 判器一侧的结论写法（崩溃点重放合并第三轮判决 J8、第三轮正文 H8「改结论的几处（红项编码、恢复字、GPU 结论换红项）搬进判器一侧的文件」）：
//! 一个状态判完之后，存进判定库的那几样怎么从判定换出来——红项的编码（[`encoded_verdict_of_the_red_items`]）、恢复字（两遍各自落到的根、
//! 走读结局与失败点，[`recovery_words_of_the_cross_check_words`]；对拍比的那几个字 [`CrossCheckWords`] 的 CPU 一侧
//! [`cross_check_words_of_cpu_judgement`]）、失败点码表（[`gpu_failure_point_code`]，GPU 内核里的常量照它生成）、计数向量
//! （[`StateCountsOfABlock`]：逐条不变量判到 / 违例 / 不适用、按发布分的态数，存成 [`BlockCountVector`]）。
//!
//! 不挂特性：没编进 `gpu` 的二进制判的 CPU 结论照样带恢复字（失败点码表不在 `gpu` 特性里）。这一份在两个判器的闭包里：
//! CPU 判器（`crate::crash`）里的 [`CrashStateJudgement`] 的方法写在这里（`impl` 头点名它，闭包按 `impl` 头收进来），
//! GPU 判器（`crate::crash_judge_gpu`）引它；改这里一个字，GPU 判器与每份设点文件的 CPU 判器版本都换。
//! 流水线（`crate::crash_amplification`）与判定库（`crate::verdict_store`）只调这里：判定库只管这几样的字节布局，不解释它们。

use std::collections::BTreeMap;

use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_core::address::{CheckpointTxg, InstanceGeneration};
use singlefs_core::recovery::{RecoveryFailure, RecoveryOutcome};

use crate::crash::{
    CrashPointReplayOracleViolationKind, CrashPointReplayPublishOfState, CrashStateJudgement,
    CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER, CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER,
    RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT, RECORD_CHECKER_ROOT_WITHOUT_RECORD,
};

// ============================== 红项的编码 ==============================

/// 一个状态的判定向量，按 [`encoded_verdict_of_the_red_items`] 的编码存成字节（判定库不解释它）；两份字节相同就是同一个判定向量。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EncodedVerdictVector(pub Vec<u8>);

/// 判定向量里红项之间的分隔：一项一行。
const RED_ITEM_SEPARATOR: &str = "\n";

/// 一个状态存进库的判定向量：红项（`CrashStateJudgement::red_items` 那种写法）逐项、换行接起来的字节；绿的状态是空向量。
#[must_use]
pub fn encoded_verdict_of_the_red_items(items: &[String]) -> EncodedVerdictVector {
    EncodedVerdictVector(items.join(RED_ITEM_SEPARATOR).into_bytes())
}

/// [`encoded_verdict_of_the_red_items`] 的逆：判定向量里的红项，按存的次序；空向量（绿的状态）是一项都没有。
/// 字节不是 UTF-8 的那几段按有损的读法读（判定向量都是这一份编出来的，读不出说明库坏了，对照那一边照样判成不同）。
#[must_use]
pub fn red_items_of_the_encoded_verdict(vector: &EncodedVerdictVector) -> Vec<String> {
    if vector.0.is_empty() {
        return Vec::new();
    }
    String::from_utf8_lossy(&vector.0)
        .split(RED_ITEM_SEPARATOR)
        .map(str::to_string)
        .collect()
}

// ============================== 恢复字 ==============================

/// 每个状态的恢复字有几个：看 journal 与不看 journal 两遍，各自落到的根两个字（实例代号、checkpoint_txg 低 32 位）、结局、失败点。
pub const RECOVERY_WORDS_PER_STATE: usize = 8;

/// 一个状态的恢复字（判定向量里的红项之外，第二轮判决 L1-6、L1-8）：存进恢复字旁表、不进判定向量编号；八个字的次序见
/// [`recovery_words_of_the_cross_check_words`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecoveryWordsOfState(pub [u32; RECOVERY_WORDS_PER_STATE]);

/// 一个状态在对拍里比的那几个字里，存进恢复字旁表的八个：两遍各自落到的根（实例代号、checkpoint_txg 低 32 位）、走读结局、失败点
/// （看 journal 那一遍在前；第二轮判决 L1-6、L1-8，第三轮判决 J6）。红项在判定向量里，判到位进计数向量，不在这八个字里。
#[must_use]
pub fn recovery_words_of_the_cross_check_words(words: &CrossCheckWords) -> RecoveryWordsOfState {
    RecoveryWordsOfState([
        words.consulted_pass_landed_root.instance,
        words.consulted_pass_landed_root.checkpoint_txg_low,
        words.consulted_pass_outcome,
        words.consulted_pass_failure_point,
        words.ignored_pass_landed_root.instance,
        words.ignored_pass_landed_root.checkpoint_txg_low,
        words.ignored_pass_outcome,
        words.ignored_pass_failure_point,
    ])
}

/// 两遍恢复（看 journal 与不看 journal）落到的根、结局码或失败点有没有不同：崩溃点重放计数 `journal_differing_states` 的判据，CPU 现判
/// （`crash` 的取样路径、E161 装置）与从库里的恢复字重建（`crash_state_enumeration`）都按这八个字比，GPU 判过的块重建出同一个数。
/// 读到的内容不进恢复字，两遍都读到文件、根相同而内容不同的状态不算（那是 oracle 的红，另计）。
#[must_use]
pub fn recovery_passes_differ(
    consulted: &singlefs_core::recovery::RecoveryReport,
    ignored: &singlefs_core::recovery::RecoveryReport,
) -> bool {
    let (consulted_outcome, consulted_failure_point) =
        outcome_and_failure_point_of(&consulted.outcome);
    let (ignored_outcome, ignored_failure_point) = outcome_and_failure_point_of(&ignored.outcome);
    landed_root_words_of(consulted.effective_root) != landed_root_words_of(ignored.effective_root)
        || consulted_outcome != ignored_outcome
        || consulted_failure_point != ignored_failure_point
}

impl CrashStateJudgement {
    /// CPU 判器这个状态的恢复字（[`cross_check_words_of_cpu_judgement`] 换出来的那几个字里的八个）：GPU 判器同一个状态的那一份由
    /// `crate::crash_judge_gpu` 从结论字换出来，两份逐字比（第三轮判决 J6 的对照）。
    #[must_use]
    pub fn recovery_words(&self) -> RecoveryWordsOfState {
        recovery_words_of_the_cross_check_words(&cross_check_words_of_cpu_judgement(self))
    }
}

// ============================== 计数向量 ==============================

/// 一条不变量在这一块里判到（成立或违例）、违例、不适用各几个状态。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InvariantStateCounts {
    pub judged: u64,
    pub violated: u64,
    pub not_applicable: u64,
}

/// 每块旁存的计数向量（第三轮判决 J7）：逐条不变量判到 / 违例 / 不适用、各观察者计数、按发布分的态数，键都是名字；
/// 复用时与结论一起读回，闭式、按发布分的态数、逐条计数、观察者计数的断言每一趟照判（`crate::crash_amplification` 收尾那一步）。
/// 「根槽已持久」的计数不在这里：各项按自己的被判根从块的持久集合事后数（第三轮判决 M2-2 甲）。字节布局在判定库
/// （`crate::verdict_store::count_vector_bytes`）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockCountVector {
    pub per_invariant: BTreeMap<String, InvariantStateCounts>,
    pub per_observer: BTreeMap<String, u64>,
    pub states_by_publish: BTreeMap<String, u64>,
}

/// 一块（或它的一段）里逐状态数出来的计数（第三轮判决 J7）：逐条不变量判到与违例的状态数、每个状态归哪次发布
/// （`crate::crash::CrashPointReplayPublishOfState::name`）。不适用的到存块时按状态数补：每个状态每条不变量要么判到（成立或违例）、要么不适用
/// （连同部分没判，与崩溃点重放计数同一口径：`crash::CrashPointReplayTally::checker_not_applicable_states`；GPU 那一份「判到位没置」也是这一格，
/// 判到位的口径照 `crate::crash_judge_gpu` 文件头）。观察者计数不在这里数，观察者那一份结论另存（[`BlockCountVector::per_observer`]）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateCountsOfABlock {
    judged_states_of_each_invariant: BTreeMap<&'static str, InvariantStateCounts>,
    states_by_publish: BTreeMap<String, u64>,
}

impl StateCountsOfABlock {
    /// CPU 判器一个状态的池级 checker 判定记进来。
    pub fn count_the_cpu_judgement(&mut self, judgement: &CrashStateJudgement) {
        for (invariant, verdict) in &judgement.pool_checker {
            let counts = self
                .judged_states_of_each_invariant
                .entry(*invariant)
                .or_default();
            match verdict {
                InvariantVerdict::Holds => counts.judged += 1,
                InvariantVerdict::Violated(_) => {
                    counts.judged += 1;
                    counts.violated += 1;
                }
                InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {}
            }
        }
    }

    /// GPU 判器一个状态的判到位与违例位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）记进来。
    pub fn count_the_gpu_bits(&mut self, judged_bits: u128, violated_bits: u128) {
        for (position, invariant) in IMPLEMENTED_INVARIANTS.iter().enumerate() {
            let bit = 1u128 << position;
            if judged_bits & bit == 0 {
                continue;
            }
            let counts = self
                .judged_states_of_each_invariant
                .entry(*invariant)
                .or_default();
            counts.judged += 1;
            if violated_bits & bit != 0 {
                counts.violated += 1;
            }
        }
    }

    /// 一个状态归哪次发布记进来（一个状态记一次）。
    pub fn count_the_publish_of_a_state(&mut self, publish: CrashPointReplayPublishOfState) {
        self.count_states_of_a_publish(publish, 1);
    }

    /// 同一次发布的 `states` 个状态一起记进来（GPU 判器那一路按块的序号区间与各段归哪次发布现算，`crate::crash_amplification`）。
    pub fn count_states_of_a_publish(
        &mut self,
        publish: CrashPointReplayPublishOfState,
        states: u64,
    ) {
        if states > 0 {
            *self.states_by_publish.entry(publish.name()).or_insert(0) += states;
        }
    }

    /// 并上另一段的。
    pub fn absorb(&mut self, other: Self) {
        for (invariant, other_counts) in other.judged_states_of_each_invariant {
            let counts = self
                .judged_states_of_each_invariant
                .entry(invariant)
                .or_default();
            counts.judged += other_counts.judged;
            counts.violated += other_counts.violated;
        }
        for (publish, states) in other.states_by_publish {
            *self.states_by_publish.entry(publish).or_insert(0) += states;
        }
    }

    /// `state_count` 个状态的一块的计数向量：清单里每条不变量一行，不适用 = 状态数 − 判到；按发布分的态数照记的；观察者计数空着
    /// （判器那一份结论不带观察者，观察者结论另存）。
    ///
    /// # Panics
    /// 某条判到的状态数比块里的状态多（记的时候一个状态只记一次，是这里的 bug）。
    #[must_use]
    pub fn into_count_vector(self, state_count: u64) -> BlockCountVector {
        let per_invariant = IMPLEMENTED_INVARIANTS
            .iter()
            .map(|invariant| {
                let counts = self
                    .judged_states_of_each_invariant
                    .get(invariant)
                    .copied()
                    .unwrap_or_default();
                (
                    (*invariant).to_string(),
                    InvariantStateCounts {
                        judged: counts.judged,
                        violated: counts.violated,
                        not_applicable: state_count.checked_sub(counts.judged).expect(
                            "一个状态每条不变量至多记一次判到：判到的状态数不超过块里的状态数",
                        ),
                    },
                )
            })
            .collect();
        BlockCountVector {
            per_invariant,
            per_observer: BTreeMap::new(),
            states_by_publish: self.states_by_publish,
        }
    }
}

// ============================== 失败点码表与对拍比的那几个字 ==============================

/// 走读结局的码（第 4、6 字；内核里的 `OUTCOME_*`）。
pub const RECOVERY_OUTCOME_CODE_NO_FILE: u32 = 0;
pub const RECOVERY_OUTCOME_CODE_FILE_READ: u32 = 1;
pub const RECOVERY_OUTCOME_CODE_FAILED: u32 = 2;

/// 红位。
pub mod verdict_bits {
    /// 看 journal 那一遍 oracle 的七类从这一位起（与 `CrashPointReplayOracleViolationKind::EVERY_KIND` 同序）。
    pub const CONSULTED_ORACLE_FIRST: u32 = 0;
    /// 不看 journal 那一遍从这一位起。
    pub const IGNORED_ORACLE_FIRST: u32 = 8;
    pub const ROOT_WITHOUT_RECORD: u32 = 16;
    pub const CLAIMED_STATE_MISSING_UNIT: u32 = 17;
    /// 给定状态表那一栏写了恢复自称的那一版（[`super::ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish`]），
    /// 看 journal 那一遍恢复落到的根却不是它（只有给定状态表那一路置；崩溃点重放那一路恒为 0）。
    pub const CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY: u32 = 18;
    pub const UNDECIDABLE: u32 = 31;
}

/// 失败点码一张表的单一来源：每一行一个常量名与它的码，生成 Rust 常量与 [`GPU_FAILURE_POINTS`]；GPU 内核里的 WGSL 常量（名字去掉 `GPU_` 前缀）
/// 由 `crate::crash_judge_gpu::failure_point_wgsl_constants` 照 [`GPU_FAILURE_POINTS`] 生成。不挂 `gpu` 特性（没编进卡的 CPU 结论照样带失败点）。
macro_rules! gpu_failure_points {
    ($($constant:ident = $code:literal,)*) => {
        $(pub const $constant: u32 = $code;)*
        /// 全部失败点码（常量名、码），按码升序：`crate::crash_judge_gpu::failure_point_wgsl_constants` 照它生成内核里的常量。
        pub const GPU_FAILURE_POINTS: &[(&str, u32)] = &[$((stringify!($constant), $constant),)*];
    };
}

gpu_failure_points! {
    GPU_FAILURE_POINT_NONE = 0,
    GPU_FAILURE_POINT_NO_VALID_SYSTEM_CONFIGURATION = 1,
    GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED = 2,
    GPU_FAILURE_POINT_SYSTEM_CONFIGURATIONS_DISAGREE = 3,
    GPU_FAILURE_POINT_NO_DEVICE_IN_THE_POOL = 4,
    GPU_FAILURE_POINT_ROOT_RING_SLOTS_PER_REGION_OUT_OF_RANGE = 5,
    GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED = 6,
    GPU_FAILURE_POINT_DEVICE_ENDS_BEFORE_THE_UNIT_AREA_START = 7,
    GPU_FAILURE_POINT_NO_VALID_ROOT = 8,
    GPU_FAILURE_POINT_UNIT_UNREADABLE = 9,
    GPU_FAILURE_POINT_UNIT_MALFORMED = 10,
    GPU_FAILURE_POINT_INVARIANT_VIOLATED = 11,
    GPU_FAILURE_POINT_ENTRY_NARROWER_THAN_ITS_FIELD_TABLE = 12,
    GPU_FAILURE_POINT_INODE_NUMBER_WATERMARK_ROW_MISSING_FROM_THE_ACCOUNTING_TREE = 13,
    GPU_FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY = 14,
    GPU_FAILURE_POINT_MAPPING_MISS = 15,
    GPU_FAILURE_POINT_MAPPING_STILL_UNREADABLE = 16,
    GPU_FAILURE_POINT_ROOT_PUBLISH_CARRIES_MORE_THAN_ONE_LAST_RECORD_FLAG_WHOSE_ANCHOR_IS_UNDECIDED = 17,
}

/// CPU 恢复的一个失败在 GPU 结论里的失败点码（第 5、7 字）：`RecoveryFailure` 每个成员一个码，成员带的细节（盘、槽、哪一处）不进码。
/// 恢复没失败（没有文件、读到文件）的那一遍失败点是 [`GPU_FAILURE_POINT_NONE`]。
#[must_use]
pub fn gpu_failure_point_code(failure: &RecoveryFailure) -> u32 {
    match failure {
        RecoveryFailure::NoValidSystemConfiguration { .. } => {
            GPU_FAILURE_POINT_NO_VALID_SYSTEM_CONFIGURATION
        }
        RecoveryFailure::SystemConfigurationIncompatBitsNotRecognized { .. } => {
            GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_INCOMPAT_BITS_NOT_RECOGNIZED
        }
        RecoveryFailure::SystemConfigurationsDisagree => {
            GPU_FAILURE_POINT_SYSTEM_CONFIGURATIONS_DISAGREE
        }
        RecoveryFailure::NoDeviceInThePool => GPU_FAILURE_POINT_NO_DEVICE_IN_THE_POOL,
        RecoveryFailure::RootRingSlotsPerRegionOutOfRange { .. } => {
            GPU_FAILURE_POINT_ROOT_RING_SLOTS_PER_REGION_OUT_OF_RANGE
        }
        RecoveryFailure::SystemConfigurationValueRefused { .. } => {
            GPU_FAILURE_POINT_SYSTEM_CONFIGURATION_VALUE_REFUSED
        }
        RecoveryFailure::DeviceEndsBeforeTheUnitAreaStart { .. } => {
            GPU_FAILURE_POINT_DEVICE_ENDS_BEFORE_THE_UNIT_AREA_START
        }
        RecoveryFailure::NoValidRoot => GPU_FAILURE_POINT_NO_VALID_ROOT,
        RecoveryFailure::UnitUnreadable { .. } => GPU_FAILURE_POINT_UNIT_UNREADABLE,
        RecoveryFailure::UnitMalformed { .. } => GPU_FAILURE_POINT_UNIT_MALFORMED,
        RecoveryFailure::InvariantViolated { .. } => GPU_FAILURE_POINT_INVARIANT_VIOLATED,
        RecoveryFailure::EntryNarrowerThanItsFieldTable { .. } => {
            GPU_FAILURE_POINT_ENTRY_NARROWER_THAN_ITS_FIELD_TABLE
        }
        RecoveryFailure::InodeNumberWatermarkRowMissingFromTheAccountingTree => {
            GPU_FAILURE_POINT_INODE_NUMBER_WATERMARK_ROW_MISSING_FROM_THE_ACCOUNTING_TREE
        }
        RecoveryFailure::AllocationRecordOutsideThePoolGeometry { .. } => {
            GPU_FAILURE_POINT_ALLOCATION_RECORD_OUTSIDE_THE_POOL_GEOMETRY
        }
        RecoveryFailure::MappingMiss { .. } => GPU_FAILURE_POINT_MAPPING_MISS,
        RecoveryFailure::MappingStillUnreadable { .. } => GPU_FAILURE_POINT_MAPPING_STILL_UNREADABLE,
        RecoveryFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
            ..
        } => GPU_FAILURE_POINT_ROOT_PUBLISH_CARRIES_MORE_THAN_ONE_LAST_RECORD_FLAG_WHOSE_ANCHOR_IS_UNDECIDED,
    }
}

/// 一个状态在对拍里比的那几个字（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 第三节 J6，用户 2026-10-10 定失败点这一版就比）：
/// 红项（两遍 oracle 与记录核对器的红位、池级 checker 的违例位）、两遍各自落到的根（实例代号、checkpoint_txg 低 32 位；没落到根时两个都是 `u32::MAX`）、
/// 两遍的走读结局与失败点（[`gpu_failure_point_code`]）、池级 checker 逐条判到位、GPU 判不了。GPU 那一侧由 `crate::crash_judge_gpu::cross_check_words_of_gpu_verdict`（挂 `gpu` 特性）、
/// CPU 那一侧由 [`cross_check_words_of_cpu_judgement`] 换出来，两份经 [`cross_check_words_agree_in_the_cross_check`] 去掉内核不置位的那几位之后逐字段相等才算对上
/// （CPU 那一侧判不了恒为假：GPU 判不了的状态对不上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrossCheckWords {
    /// 结论第 0 字去掉判不了那一位：两遍 oracle 各七类（[`verdict_bits`]）、记录核对器两条。
    pub red_bits: u32,
    /// 池级 checker 的违例位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`；[`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS`]
    /// 那几位 CPU 一侧照判定置、GPU 内核恒 0，对拍经 [`cross_check_words_agree_in_the_cross_check`] 去掉再比）。
    pub pool_checker_violated_bits: u128,
    pub consulted_pass_landed_root: LandedRootWords,
    pub ignored_pass_landed_root: LandedRootWords,
    pub consulted_pass_outcome: u32,
    pub ignored_pass_outcome: u32,
    pub consulted_pass_failure_point: u32,
    pub ignored_pass_failure_point: u32,
    /// 池级 checker 逐条判到位（成立或违例；表里那几位同违例位）。
    pub pool_checker_judged_bits: u128,
    /// 池级 checker 判 I-1.13 的结局码（[`SCAN_REBUILD_OUTCOMES`]：成立、违例、报不适用的哪一种理由；没走到是
    /// [`SCAN_REBUILD_OUTCOME_NOT_REACHED`]）：GPU 那一侧是结论第 24 字（`crate::crash_judge_gpu` 的 `SCAN_REBUILD_OUTCOME_WORD`），CPU 那一侧由
    /// [`scan_rebuild_outcome_of_the_pool_checker_verdicts`] 从判定换出来，两路逐个比（C821 交主 agent 判的第 2 条：结局码并进对拍字）。
    /// 判定库不存它（同判到位）：从库里读回的给定状态这一字是 [`SCAN_REBUILD_OUTCOME_NOT_REACHED`]。
    pub scan_rebuild_outcome: u32,
    pub gpu_could_not_judge: bool,
}

/// 一遍恢复落到的根在结论里的两个字：实例代号、checkpoint_txg 低 32 位；没落到根时两个都是 `u32::MAX`。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LandedRootWords {
    pub instance: u32,
    pub checkpoint_txg_low: u32,
}

impl LandedRootWords {
    /// 没落到根（择不到系统配置或根、重放在施加任何记录之前停下）。
    pub const NOT_LANDED: Self = Self {
        instance: u32::MAX,
        checkpoint_txg_low: u32::MAX,
    };
}

/// 一遍恢复的报告换成落到的根那两个字：`effective_root`（施加 journal 之后实际走的那条根；退回见证到的那一版时是退到的那一版）。
fn landed_root_words_of(
    effective_root: Option<(
        singlefs_core::address::InstanceGeneration,
        singlefs_core::address::CheckpointTxg,
    )>,
) -> LandedRootWords {
    effective_root.map_or(LandedRootWords::NOT_LANDED, |(instance, checkpoint_txg)| {
        LandedRootWords {
            instance: instance.0,
            checkpoint_txg_low: u32::try_from(checkpoint_txg.0 & 0xFFFF_FFFF).expect("低 32 位"),
        }
    })
}

/// 走读结局的码与失败点码（第 4 / 5 字或第 6 / 7 字）。
fn outcome_and_failure_point_of(outcome: &RecoveryOutcome) -> (u32, u32) {
    match outcome {
        RecoveryOutcome::NoFile { .. } => (RECOVERY_OUTCOME_CODE_NO_FILE, GPU_FAILURE_POINT_NONE),
        RecoveryOutcome::FileRead { .. } => {
            (RECOVERY_OUTCOME_CODE_FILE_READ, GPU_FAILURE_POINT_NONE)
        }
        RecoveryOutcome::Failed { failure, .. } => (
            RECOVERY_OUTCOME_CODE_FAILED,
            gpu_failure_point_code(failure),
        ),
    }
}

/// oracle 一类违例在红位里的位置（与 `CrashPointReplayOracleViolationKind::EVERY_KIND` 同序）。
fn oracle_kind_position(kind: CrashPointReplayOracleViolationKind) -> u32 {
    u32::try_from(
        CrashPointReplayOracleViolationKind::EVERY_KIND
            .iter()
            .position(|candidate| *candidate == kind)
            .expect("七类之一"),
    )
    .expect("七类，装得进 u32")
}

/// 进了 GPU 判器的覆盖表（`crate::crash_judge_gpu` 的 `POOL_CHECKER_COVERED_INVARIANT_INDEXES`）而内核不置位的不变量
/// （`IMPLEMENTED_INVARIANTS` 的下标，升序）：GPU 与 CPU 对拍对这几条不比。CPU 一侧照实判、照实置位：两个构造函数
/// （[`cross_check_words_of_cpu_judgement`]、[`cross_check_words_of_a_given_state`]）经 [`pool_checker_violated_and_judged_bits`]
/// 照置这几条的判到位与违例位，红项与计数向量照记；只在对拍比较处（[`pool_checker_bits_agree_in_the_cross_check`]、
/// [`cross_check_words_agree_in_the_cross_check`]）把这几位从两边都去掉再比。
/// 今天是空表：原来只有下标 92 的 I-1.13（扫描重建与遍历方向的索引相同，用户 2026-10-13 定「先进覆盖表但内核不置位，记欠账」，
/// `.claude/kb/checks-owed.md` C821（GPU 判器没有扫描重建内核）），用户 2026-10-18 定「补 GPU 扫描重建内核」（两路结论逐位相同、导入比较不改）之后
/// 扫描内核源码里的第三个入口（`crash_judge_checker_scan_rebuild.wgsl` 的 `scan_rebuild_main`）照判、置位，对拍逐位比（`crate::crash_judge_gpu` 的
/// `KERNEL_INVARIANT_INDEX_OF_THE_SCAN_REBUILD_JUDGE` 编译期守着它不再进这张表）。表与比较处那两个函数留着：以后再有进了覆盖表而内核不置位的一条，
/// 登记在这里，比较处照旧只去掉它。下标 73 的 I-4.4 不在这里：池级 checker 起初对它恒报不适用、两边都不置位，
/// 里程碑四改判 logged_ops 条目之后引用与快照那一截照判、置位。下标 95–98 的 I-3.15、I-5.6、I-3.16、I-1.14（判决
/// `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3 的 N2、N3、N5、N6）也不在这里：扫描内核主入口判 I-3.16、引用与快照那一截之后判另三条与
/// I-4.4，照 `singlefs_checker::walk` 那几个函数判，对拍逐位比（`crate::crash_judge_gpu` 的
/// `KERNEL_INVARIANT_INDEXES_OF_THE_COMPACTION_AND_RELOCATION_JUDGES` 编译期守着它们不进这张表）。
/// 下标 93、94 的 I-3.6（deadlist 紧界）与 I-3.7（活快照可枚举）与并进下标 65 的 I-3.5 ③ ④ 与逆向（判决
/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5）不在这里：扫描内核源码里的引用与快照那一截
/// （`crash_judge_checker_scan.wgsl` 的 `references_and_snapshots_main`）照 `singlefs_checker::walk` 的 `judge_snapshots_and_deadlists_of_the_versions`
/// 判、置位，对拍逐位比（D13（验证路线） 已定项 15：判器整套上 GPU、罩全部池级 checker 不变量）。
/// 下标 102 的 I-9.40（extent 上段形状由有条目的 inode 号集合定，判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2）
/// 也不在这里：扫描内核的删除态扫描照判、置位，对拍逐位比（`crate::crash_judge_gpu` 的 `KERNEL_INVARIANT_INDEX_OF_THE_UPPER_SHAPE_JUDGE`
/// 编译期守着它不进这张表；池级 checker 那一批交回时是过渡的 `[92, 102]`，与扫描重建那一份合成空表）。
pub const POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS: &[usize] = &[];

// 表里的下标都在池级 checker 的清单里、按下标升序、一条不重（挂 `gpu` 特性时另核都在覆盖表里，那一条在 `crate::crash_judge_gpu`）。
const _: () = {
    let mut position = 0;
    while position < POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS.len() {
        assert!(
            POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS[position]
                < IMPLEMENTED_INVARIANTS.len(),
            "内核不置位的下标在池级 checker 的清单里"
        );
        assert!(
            position == 0
                || POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS[position - 1]
                    < POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS[position],
            "内核不置位的下标升序、一条不重"
        );
        position += 1;
    }
};

/// [`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS`] 里那几条的位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）：
/// 对拍比较处从 GPU 与 CPU 两边都去掉的那几位。
pub const POOL_CHECKER_BITS_LEFT_OUT_OF_THE_CROSS_CHECK: u128 = {
    let mut bits = 0u128;
    let mut position = 0;
    while position < POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS.len() {
        bits |= 1u128 << POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS[position];
        position += 1;
    }
    bits
};

/// 池级 checker 的一组位（违例位或判到位，位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）在对拍里比的那一部分：
/// 去掉 [`POOL_CHECKER_BITS_LEFT_OUT_OF_THE_CROSS_CHECK`]，别的位原样。
#[must_use]
pub fn pool_checker_bits_compared_by_the_cross_check(bits: u128) -> u128 {
    bits & !POOL_CHECKER_BITS_LEFT_OUT_OF_THE_CROSS_CHECK
}

/// GPU 与 CPU 同一种池级 checker 位（违例位对违例位、判到位对判到位）在对拍里对不对得上：
/// 两边各经 [`pool_checker_bits_compared_by_the_cross_check`] 去掉表里那几位，再逐位比。
#[must_use]
pub fn pool_checker_bits_agree_in_the_cross_check(gpu_bits: u128, cpu_bits: u128) -> bool {
    pool_checker_bits_compared_by_the_cross_check(gpu_bits)
        == pool_checker_bits_compared_by_the_cross_check(cpu_bits)
}

/// 一份对拍字在对拍里比的那一份：违例位与判到位经 [`pool_checker_bits_compared_by_the_cross_check`] 去掉表里那几位，别的字段原样。
#[must_use]
pub fn cross_check_words_compared_by_the_cross_check(words: CrossCheckWords) -> CrossCheckWords {
    let mut compared = words;
    compared.pool_checker_violated_bits =
        pool_checker_bits_compared_by_the_cross_check(words.pool_checker_violated_bits);
    compared.pool_checker_judged_bits =
        pool_checker_bits_compared_by_the_cross_check(words.pool_checker_judged_bits);
    compared
}

/// GPU 与 CPU 同一个状态的对拍字对不对得上：两份各经 [`cross_check_words_compared_by_the_cross_check`] 去掉表里那几位，再逐字段比。
#[must_use]
pub fn cross_check_words_agree_in_the_cross_check(
    gpu_words: &CrossCheckWords,
    cpu_words: &CrossCheckWords,
) -> bool {
    cross_check_words_compared_by_the_cross_check(*gpu_words)
        == cross_check_words_compared_by_the_cross_check(*cpu_words)
}

/// 池级 checker 一份判定的违例位与判到位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`；判到 = 成立或违例，不适用与部分没判都不算）。
/// [`POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS`] 里那几条照判定置位、不跳过（对拍不比只在比较处做）。
///
/// # Panics
/// 判定里有一条不在 `IMPLEMENTED_INVARIANTS` 里。
#[must_use]
pub fn pool_checker_violated_and_judged_bits(
    verdicts: &[(&'static str, InvariantVerdict)],
) -> (u128, u128) {
    verdicts.iter().fold(
        (0u128, 0u128),
        |(violated, judged), (invariant, verdict)| {
            let bit = 1u128
                << IMPLEMENTED_INVARIANTS
                    .iter()
                    .position(|candidate| candidate == invariant)
                    .expect("池级 checker 报的不变量在清单里");
            match verdict {
                InvariantVerdict::Violated(_) => (violated | bit, judged | bit),
                InvariantVerdict::Holds => (violated, judged | bit),
                InvariantVerdict::NotApplicable(_) | InvariantVerdict::PartiallyJudged(_) => {
                    (violated, judged)
                }
            }
        },
    )
}

/// CPU 判器一个状态的判定换成对拍比的那几个字（池级 checker 取判定里那一份；旋钮值编进内核的对拍用例拿带同一组旋钮值的入口重判之后
/// 自己换掉违例位与判到位）。
#[must_use]
pub fn cross_check_words_of_cpu_judgement(judgement: &CrashStateJudgement) -> CrossCheckWords {
    let mut red_bits = 0u32;
    if let Some(violation) = &judgement.consulted_oracle {
        red_bits |=
            1 << (verdict_bits::CONSULTED_ORACLE_FIRST + oracle_kind_position(violation.kind));
    }
    if let Some(violation) = &judgement.ignored_oracle {
        red_bits |=
            1 << (verdict_bits::IGNORED_ORACLE_FIRST + oracle_kind_position(violation.kind));
    }
    if judgement.records.root_without_record {
        red_bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
    }
    if judgement.records.claimed_state_missing_unit {
        red_bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
    }
    let (pool_checker_violated_bits, pool_checker_judged_bits) =
        pool_checker_violated_and_judged_bits(&judgement.pool_checker);
    let (consulted_pass_outcome, consulted_pass_failure_point) =
        outcome_and_failure_point_of(&judgement.consulted.outcome);
    let (ignored_pass_outcome, ignored_pass_failure_point) =
        outcome_and_failure_point_of(&judgement.ignored.outcome);
    CrossCheckWords {
        red_bits,
        pool_checker_violated_bits,
        consulted_pass_landed_root: landed_root_words_of(judgement.consulted.effective_root),
        ignored_pass_landed_root: landed_root_words_of(judgement.ignored.effective_root),
        consulted_pass_outcome,
        ignored_pass_outcome,
        consulted_pass_failure_point,
        ignored_pass_failure_point,
        pool_checker_judged_bits,
        scan_rebuild_outcome: scan_rebuild_outcome_of_the_pool_checker_judgement(
            &judgement.pool_checker,
        ),
        gpu_could_not_judge: false,
    }
}

// ============================== 扫描重建那一截判 I-1.13 的结局码（对拍字的第一项之外那一字） ==============================
//
// 放在判器一侧、不挂 `gpu` 特性：对拍字 [`CrossCheckWords`] 带 I-1.13 的结局码（GPU 结论第 24 字，`crate::crash_judge_gpu` 的
// `SCAN_REBUILD_OUTCOME_WORD`），CPU 一侧由池级 checker 的判定换码（[`scan_rebuild_outcome_of_the_pool_checker_verdicts`]），没编进 `gpu` 的
// 二进制判的 CPU 结论也要这张表；`crate::crash_judge_gpu` 照旧转出去，内核里的常量在那边照这张表生成。

/// 两个名字逐字节相同（编译期比 `IMPLEMENTED_INVARIANTS` 里的名字用）。
pub(crate) const fn invariant_names_are_equal(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut position = 0;
    while position < left.len() {
        if left[position] != right[position] {
            return false;
        }
        position += 1;
    }
    true
}

/// CPU 池级 checker 判 I-1.13 判成的那一种（对拍拿它与 GPU 的结局码比）：不适用带理由原文，原文逐字等于
/// `singlefs_checker::walk` 与 `singlefs_checker::rebuild` 报的那一句。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanRebuildVerdictOfThePoolChecker {
    Holds,
    Violated,
    NotApplicable(&'static str),
}

/// 扫描重建那一截判 I-1.13 的一种结局：WGSL 里的常量名（整名写在串字面量里：判法单元的键收 WGSL 外名的着落时认得它）、结局码、
/// CPU 池级 checker 判成的那一种。
#[derive(Clone, Copy, Debug)]
pub struct ScanRebuildOutcome {
    pub constant_name: &'static str,
    pub code: u32,
    pub verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker,
}

/// 走读内核没判完（判不了、交接段无效）时那一字：结论缓冲的初值，走读内核在不是整池交回（码 27 到 30 那四种）的状态上也写这个值；
/// 扫描重建那一截判到一半判不了（判不了那一字已置）时内核也写这个值。
pub const SCAN_REBUILD_OUTCOME_NOT_REACHED: u32 = u32::MAX;

/// 遍历方向的索引已被别的不变量判损坏时报不适用的那七条（`singlefs_checker::walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`
/// 的次序；扫描重建那一截照这个次序取第一条违例了的，次序挪了这里编译不过）。I-9.32 是多文件删除那一批照 I-9.34 的先例加的（判决
/// `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M5 之后删除中不再整张报不适用，退役记录坏了的那一形露出来）。
const INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED_IN_THE_KERNEL: [&str; 7] = [
    "I-1.11", "I-1.12", "I-4.2", "I-5.1", "I-9.34", "I-9.32", "I-9.39",
];

const _: () = {
    let in_the_pool_checker =
        singlefs_checker::walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED;
    assert!(
        in_the_pool_checker.len()
            == INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED_IN_THE_KERNEL.len(),
        "池级 checker 认遍历方向索引已损坏的不变量条数变了：改扫描重建内核 scan_rebuild_outcome_of_the_state 与这张表"
    );
    let mut position = 0;
    while position < in_the_pool_checker.len() {
        assert!(
            invariant_names_are_equal(
                in_the_pool_checker[position].0,
                INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED_IN_THE_KERNEL[position]
            ),
            "池级 checker 认遍历方向索引已损坏的不变量次序变了：改扫描重建内核 scan_rebuild_outcome_of_the_state 与这张表"
        );
        position += 1;
    }
};

/// 遍历方向的索引已被第 `position` 条（[`INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED_IN_THE_KERNEL`] 的次序）判损坏时那一句不适用。
const fn damaged_walked_index_reason(position: usize) -> &'static str {
    singlefs_checker::walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED[position].1
}

/// 扫描重建那一截判 I-1.13 的全部结局：成立、违例，与 CPU 池级 checker 报不适用的每一种理由——`walk.rs` 的
/// `check_pool_image_given_the_container_capacity_knobs` 走到扫描重建之前整池交回的四句（带越界值整池拒、journal 环几何三格不等整池拒、
/// 一块盘都择不到系统配置、根环里一条自证过的根都没有），再是 `judge_scan_rebuild_agrees_with_the_walked_index` 依次：实例表读不全、
/// 遍历方向的索引已判损坏的七条、`IndexTableTheWalkDoesNotReadInFull` 的七种、`rebuild::ScanRebuildNotDecided` 的九种
/// （「删除中」那一种随判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M5 删掉：删除中的对象只对它自己两侧都不比）。
/// 理由原文逐字抄那两份源码里的（四句早退的串、`walk` 与 `rebuild` 里那两个枚举都不出池级 checker 的 crate）：那边改了一个字、这边没跟上，对拍判
/// [`scan_rebuild_outcome_of_the_pool_checker_verdicts`] 认不得那一句；`crate::crash_judge_gpu` 的单测
/// `every_reason_the_pool_checker_can_give_for_the_scan_rebuild_is_a_row_of_the_outcome_table` 扫池级 checker 的源码，逐句核两边相等。
/// 整池交回的四句由走读内核在 `judge_the_pool_from_its_system_configuration_slots` 那几处早退写进结局字，别的由扫描重建那一截写。
pub const SCAN_REBUILD_OUTCOMES: [ScanRebuildOutcome; 30] = [
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_HOLDS",
        code: 0,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::Holds,
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_VIOLATED",
        code: 1,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::Violated,
    },
    // 码 27 到 30：池级 checker 走到扫描重建之前整池交回的四句（每条不变量都报这一句不适用，I-1.13 也在里面），走读内核在同几处早退写结局字
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_A_SYSTEM_CONFIGURATION_VALUE_THE_READER_DOES_NOT_ACCEPT",
        code: 27,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "池里有一槽自证过的系统配置带这一版读者不收的池级值：实现整池拒绝挂载，这不是一个挂得上的镜像",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_POOL_REFUSED_FOR_UNEQUAL_JOURNAL_RING_GEOMETRY_FIELDS",
        code: 28,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "池里有一槽可择的系统配置 journal 环几何三个字段（在飞上限、最坏占用、F）有一格不等：实现整池拒绝挂载，这不是一个挂得上的镜像",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_NO_DEVICE_YIELDS_A_VALID_SYSTEM_CONFIGURATION",
        code: 29,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "池里没有一块盘交得出有效的系统配置：这不是一个挂得上的镜像",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_ROOT_RING_HOLDS_NO_SELF_VERIFIED_ROOT",
        code: 30,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "根环里一条自证过的根都没有：没有最新根，遍历方向那三张表无从读、已发布谓词也没有输入",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_INSTANCE_TABLE_NOT_READ_IN_FULL",
        code: 2,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            singlefs_checker::walk::SCAN_REBUILD_INSTANCE_TABLE_NOT_READ_IN_FULL,
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_1_11",
        code: 3,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            damaged_walked_index_reason(0),
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_1_12",
        code: 4,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            damaged_walked_index_reason(1),
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_4_2",
        code: 5,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            damaged_walked_index_reason(2),
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_5_1",
        code: 6,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            damaged_walked_index_reason(3),
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_9_34",
        code: 7,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            damaged_walked_index_reason(4),
        ),
    },
    // 码 26：多文件删除那一批加的（原来的码不挪，码 23 那一种「删除中」随判决 M5 删掉，不复用）
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_9_32",
        code: 26,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            damaged_walked_index_reason(5),
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_WALKED_INDEX_DAMAGED_BY_I_9_39",
        code: 8,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            damaged_walked_index_reason(6),
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_TREE_TABLE_NOT_READ_IN_FULL",
        code: 9,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "最新根的树表读不出、解不开、条目窄于字段表或同一种树出现两条：遍历方向那三张表读不全",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_INODE_TREE_NOT_READ_IN_FULL",
        code: 10,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "最新根的 inode 树读不全：遍历方向的 inode 记录集合读不全",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_EXTENT_TREE_NOT_READ_IN_FULL",
        code: 11,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "最新根的 extent 树读不全：遍历方向的每个 inode 的数据单元集合读不全",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_CENTRAL_MAPPING_TREE_NOT_READ_IN_FULL",
        code: 12,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "最新根的中央映射树读不全：遍历方向的中央映射读不全",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_EXTENT_TREE_NAMES_ONE_POSITION_TWICE",
        code: 13,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "最新根的 extent 树里同一个 (inode, 偏移) 有两条：遍历方向的数据单元集合不是一张表",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_CENTRAL_MAPPING_TREE_NAMES_ONE_KEY_TWICE",
        code: 14,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "最新根的中央映射树里同一个 key 有两条：遍历方向的中央映射不是一张表",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_DEADLIST_TREE_NOT_READ_IN_FULL",
        code: 15,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "最新根的 deadlist 树读不全：被活快照钉住、映射条目留存的那几个单元认不全，遍历方向的中央映射里哪些是当前版本分不出",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_DATA_UNIT_VERSIONS_TIED_ON_THE_WRITE_ORDER",
        code: 16,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "扫描重建：码 1 同一对象身份上两版写序逐字节相同，C113 定案 P3 择写序最大者择不出，码 1 的全序键停在 C464（I-1.8 对码 1 的全序键写窄了）",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_PACKED_RECORD_VERSIONS_TIED_ON_THE_TOTAL_ORDER_KEY",
        code: 17,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "扫描重建：码 3 同一对象身份上两组 (诞生代号, 实例代号, 事务号) 相同，I-1.8 ② 不成立，择不出当前版本",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_COPIES_OF_ONE_VERSION_CARRY_DIFFERENT_PAYLOADS",
        code: 18,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "扫描重建：同一版的副本头里的载荷校验和不同，I-1.8 ① 判损坏、不择",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_CURRENT_PACKED_CONTAINER_DECODES_FROM_NO_COPY",
        code: 19,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "扫描重建：当前版本那片码 3 容器（墓碑、inode、退役、谱系）没有一份副本解得开",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_LINEAGE_RECORDS_FORM_NO_PARENT_CHAIN",
        code: 20,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "扫描重建：谱系记录接不成一条父链（有一条 T_old ≥ T_rb，或同一个 T_rb 有两条），父链断了按谱系择现行怎么认没有条款，第一版不定",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_RETIRED_LEAF_CONTAINER_HAS_A_VERSION_BORN_AFTER_ITS_DEATH",
        code: 21,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "扫描重建：谱系上的退役记录点名的 inode 叶容器有一版诞生代号晚于死亡 txg，退役树与那一版谁算数没有条款（I-9.34），第一版不定",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_TOMBSTONE_RANGE_OTHER_THAN_THE_WHOLE_OBJECT",
        code: 22,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "扫描重建：墓碑区间不是整对象 [0, 2⁶⁴ − 1]，第一版只写整对象（D18 已定项 24 第 2 条），区间墓碑怎么压 key 没有条款",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_TWO_OBJECTS_CLAIM_ONE_DATA_UNIT_POSITION",
        code: 24,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "扫描重建：两个不同对象身份的当前版本数据单元落在同一个 (inode, 偏移) 上，按哪一个认没有条款",
        ),
    },
    ScanRebuildOutcome {
        constant_name: "SCAN_REBUILD_OUTCOME_INODE_NUMBER_IN_TWO_CURRENT_CONTAINERS",
        code: 25,
        verdict_of_the_pool_checker: ScanRebuildVerdictOfThePoolChecker::NotApplicable(
            "扫描重建：同一个 inode 号出现在两片当前版本的 inode 叶容器里，按哪一条认没有条款",
        ),
    },
];

/// CPU 池级 checker 一份判定里 I-1.13 换成的结局码：判定里没有它（池级 checker 没走到最后）是 [`SCAN_REBUILD_OUTCOME_NOT_REACHED`]；
/// 成立、违例各一个码；不适用按理由原文对 [`SCAN_REBUILD_OUTCOMES`] 里的那一句。
///
/// # Errors
/// I-1.13 判成「部分没判」（池级 checker 不这样判它），或不适用的理由原文不是表里任何一句（池级 checker 那边改了理由、表没跟上）。
pub fn scan_rebuild_outcome_of_the_pool_checker_verdicts(
    verdicts: &[(&'static str, InvariantVerdict)],
) -> Result<u32, String> {
    let Some((_, verdict)) = verdicts.iter().find(|(invariant, _)| {
        *invariant == singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX
    }) else {
        return Ok(SCAN_REBUILD_OUTCOME_NOT_REACHED);
    };
    let wanted = match verdict {
        InvariantVerdict::Holds => ScanRebuildVerdictOfThePoolChecker::Holds,
        InvariantVerdict::Violated(_) => ScanRebuildVerdictOfThePoolChecker::Violated,
        InvariantVerdict::NotApplicable(reason) => {
            ScanRebuildVerdictOfThePoolChecker::NotApplicable(reason)
        }
        InvariantVerdict::PartiallyJudged(detail) => {
            return Err(format!(
                "池级 checker 把 I-1.13 判成部分没判（{detail}）：扫描重建那一截没有这种结局"
            ))
        }
    };
    SCAN_REBUILD_OUTCOMES
        .iter()
        .find(|outcome| outcome.verdict_of_the_pool_checker == wanted)
        .map(|outcome| outcome.code)
        .ok_or_else(|| {
            format!("池级 checker 判 I-1.13 的「{wanted:?}」不在扫描重建的结局表里：表里的理由原文没跟上池级 checker")
        })
}

/// 结局码的名字（判红时写进说明）；表外的码写成码本身。
#[must_use]
pub fn scan_rebuild_outcome_name(code: u32) -> String {
    if code == SCAN_REBUILD_OUTCOME_NOT_REACHED {
        return "SCAN_REBUILD_OUTCOME_NOT_REACHED".to_string();
    }
    SCAN_REBUILD_OUTCOMES
        .iter()
        .find(|outcome| outcome.code == code)
        .map_or_else(
            || format!("表外的码 {code}"),
            |outcome| outcome.constant_name.to_string(),
        )
}

/// 对拍字里 CPU 一侧 I-1.13 的结局码（[`CrossCheckWords::scan_rebuild_outcome`]）：[`scan_rebuild_outcome_of_the_pool_checker_verdicts`] 换出来的那一个。
///
/// # Panics
/// 池级 checker 把 I-1.13 判成部分没判、或报了表外的一句不适用理由：表的理由原文没跟上池级 checker（单测
/// `crate::crash_judge_gpu` 的 `every_scan_rebuild_outcome_has_its_own_code_generated_into_the_kernels_and_read_back_from_the_pool_checker`
/// 逐条核表里每一句换得回；池级 checker 新加一句理由而表没跟上时在这里停下，不把那个状态记成哪一种结局）。
#[must_use]
pub fn scan_rebuild_outcome_of_the_pool_checker_judgement(
    pool_checker: &[(&'static str, InvariantVerdict)],
) -> u32 {
    scan_rebuild_outcome_of_the_pool_checker_verdicts(pool_checker).unwrap_or_else(|reason| {
        panic!("池级 checker 判 I-1.13 的结局都在扫描重建的结局表里（表与池级 checker 的理由原文同步）：{reason}")
    })
}

// ============================== 给定状态（随机历史两项）的对拍字与红项 ==============================

/// 红位里两遍 oracle 的那十四位（看 journal 那一遍七类、不看那一遍七类，[`verdict_bits`]）：给定状态表那一路两边都不判不比
/// （用户 2026-10-10 定「第一截与第三截都上 GPU」：随机历史两项的 CPU 判法今天不跑崩溃点重放的 oracle，也不给期望版本表）。
#[must_use]
pub fn oracle_red_bits() -> u32 {
    CrashPointReplayOracleViolationKind::EVERY_KIND
        .iter()
        .fold(0u32, |bits, kind| {
            let position = oracle_kind_position(*kind);
            bits | (1 << (verdict_bits::CONSULTED_ORACLE_FIRST + position))
                | (1 << (verdict_bits::IGNORED_ORACLE_FIRST + position))
        })
}

/// 对拍字去掉两遍 oracle 的红位（[`oracle_red_bits`]）：给定状态表那一路 GPU 一侧的结论字先过它再比、再换红项。
#[must_use]
pub fn cross_check_words_without_the_oracle(words: CrossCheckWords) -> CrossCheckWords {
    let mut without_the_oracle = words;
    without_the_oracle.red_bits &= !oracle_red_bits();
    without_the_oracle
}

/// 给定状态的一个状态在 CPU 上判出的四样（看 journal 与不看 journal 两遍恢复、池级 checker、记录核对器；判法在
/// `crate::crash_injection::judge_a_given_state_on_the_cpu`）换成对拍比的那几个字（崩溃点重放合并第三轮判决 J6 的 CPU 一侧；
/// 不从 [`CrashStateJudgement`] 来，那一份带 oracle，这一路不判 oracle：红位里 oracle 那十四位恒为 0）。`claimed_version` 是这个状态那一栏
/// 恢复自称的那一版（[`ClaimedVersionOfAGivenState`]）：写了而看 journal 那一遍落到的根不是它，置
/// [`verdict_bits::CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY`]。
#[must_use]
pub fn cross_check_words_of_a_given_state(
    consulted: &singlefs_core::recovery::RecoveryReport,
    ignored: &singlefs_core::recovery::RecoveryReport,
    pool_checker: &[(&'static str, InvariantVerdict)],
    records: singlefs_harness::memory_pool::RecordCheck,
    claimed_version: ClaimedVersionOfAGivenState,
) -> CrossCheckWords {
    let mut red_bits = 0u32;
    if records.root_without_record {
        red_bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
    }
    if records.claimed_state_missing_unit {
        red_bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
    }
    if claimed_version.is_not_the_version_landed_on(consulted.effective_root) {
        red_bits |= 1 << verdict_bits::CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY;
    }
    let (pool_checker_violated_bits, pool_checker_judged_bits) =
        pool_checker_violated_and_judged_bits(pool_checker);
    let (consulted_pass_outcome, consulted_pass_failure_point) =
        outcome_and_failure_point_of(&consulted.outcome);
    let (ignored_pass_outcome, ignored_pass_failure_point) =
        outcome_and_failure_point_of(&ignored.outcome);
    CrossCheckWords {
        red_bits,
        pool_checker_violated_bits,
        consulted_pass_landed_root: landed_root_words_of(consulted.effective_root),
        ignored_pass_landed_root: landed_root_words_of(ignored.effective_root),
        consulted_pass_outcome,
        ignored_pass_outcome,
        consulted_pass_failure_point,
        ignored_pass_failure_point,
        pool_checker_judged_bits,
        scan_rebuild_outcome: scan_rebuild_outcome_of_the_pool_checker_judgement(pool_checker),
        gpu_could_not_judge: false,
    }
}

/// 一个给定状态的结论里记录核对器那两条（对拍字的红位第 16、17 位）。
#[must_use]
pub fn record_check_of_the_cross_check_words(
    words: &CrossCheckWords,
) -> singlefs_harness::memory_pool::RecordCheck {
    singlefs_harness::memory_pool::RecordCheck {
        root_without_record: words.red_bits & (1 << verdict_bits::ROOT_WITHOUT_RECORD) != 0,
        claimed_state_missing_unit: words.red_bits
            & (1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT)
            != 0,
    }
}

/// 一个给定状态存进库的红项：池级 checker 违例按 `IMPLEMENTED_INVARIANTS` 的次序逐条，再是记录核对器两条（与
/// `CrashStateJudgement::red_items` 同名；oracle 不判，不出项）。GPU 与 CPU 两侧都从对拍字换出来，同一个状态两侧的项逐项同序。
#[must_use]
pub fn red_items_of_a_given_state(words: &CrossCheckWords) -> Vec<String> {
    let mut items: Vec<String> = IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
        .filter(|(position, _)| words.pool_checker_violated_bits & (1u128 << position) != 0)
        .map(|(_, invariant)| format!("{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{invariant}"))
        .collect();
    let records = record_check_of_the_cross_check_words(words);
    if records.root_without_record {
        items.push(format!(
            "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_ROOT_WITHOUT_RECORD}"
        ));
    }
    if records.claimed_state_missing_unit {
        items.push(format!(
            "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
        ));
    }
    if claimed_version_is_not_landed_on_in_the_cross_check_words(words) {
        items.push(format!(
            "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY}"
        ));
    }
    items
}

/// 一个给定状态的结论里「那一栏写的恢复自称的那一版，看 journal 那一遍没落到它」那一位（对拍字的红位第 18 位）。
#[must_use]
pub fn claimed_version_is_not_landed_on_in_the_cross_check_words(words: &CrossCheckWords) -> bool {
    words.red_bits & (1 << verdict_bits::CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY) != 0
}

/// 红项里 [`verdict_bits::CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY`] 那一项的名字（接在记录核对器那一遍的头后面，与
/// `crate::crash::RECORD_CHECKER_ROOT_WITHOUT_RECORD` 一族同一种写法；只有给定状态表那一路出这一项）。
pub const RECORD_CHECKER_CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY: &str =
    "claimed_version_not_landed_on_by_the_recovery";

/// 从库里读回的一个给定状态（红项与恢复字）还原出的对拍字：池级 checker 判到位与 GPU 判不了没存（判到的计数在块的计数向量里），读回是 0、假。
///
/// # Errors
/// 红项不是 [`red_items_of_a_given_state`] 写得出的那几种（库坏了、或这一块不是给定状态那一路存的）。
pub fn cross_check_words_of_a_stored_given_state(
    items: &[String],
    recovery_words: RecoveryWordsOfState,
) -> Result<CrossCheckWords, String> {
    let mut red_bits = 0u32;
    let mut pool_checker_violated_bits = 0u128;
    for item in items {
        let record_item = item
            .strip_prefix(CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER)
            .and_then(|rest| rest.strip_prefix(':'));
        let pool_checker_item = item
            .strip_prefix(CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER)
            .and_then(|rest| rest.strip_prefix(':'));
        match (record_item, pool_checker_item) {
            (Some(RECORD_CHECKER_ROOT_WITHOUT_RECORD), None) => {
                red_bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
            }
            (Some(RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT), None) => {
                red_bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
            }
            (Some(RECORD_CHECKER_CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY), None) => {
                red_bits |= 1 << verdict_bits::CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY;
            }
            (None, Some(invariant)) => {
                let position = IMPLEMENTED_INVARIANTS
                    .iter()
                    .position(|candidate| *candidate == invariant)
                    .ok_or_else(|| format!("库里给定状态的红项点名了清单外的不变量：「{item}」"))?;
                pool_checker_violated_bits |= 1u128 << position;
            }
            (Some(_) | None, Some(_) | None) => {
                return Err(format!(
                    "库里给定状态的红项不是池级 checker 或记录核对器那几种：「{item}」"
                ));
            }
        }
    }
    let [consulted_instance, consulted_txg_low, consulted_pass_outcome, consulted_pass_failure_point, ignored_instance, ignored_txg_low, ignored_pass_outcome, ignored_pass_failure_point] =
        recovery_words.0;
    Ok(CrossCheckWords {
        red_bits,
        pool_checker_violated_bits,
        consulted_pass_landed_root: LandedRootWords {
            instance: consulted_instance,
            checkpoint_txg_low: consulted_txg_low,
        },
        ignored_pass_landed_root: LandedRootWords {
            instance: ignored_instance,
            checkpoint_txg_low: ignored_txg_low,
        },
        consulted_pass_outcome,
        ignored_pass_outcome,
        consulted_pass_failure_point,
        ignored_pass_failure_point,
        pool_checker_judged_bits: 0,
        scan_rebuild_outcome: SCAN_REBUILD_OUTCOME_NOT_REACHED,
        gpu_could_not_judge: false,
    })
}

// ============================== 给定状态恢复自称的那一版（D13（验证路线） 已定项 7 第二截） ==============================

/// 一个给定状态上，记录核对器第二条判据（恢复自称新态而单元缺席）认的「恢复自称的那一版」从哪来（用户 2026-10-14 弹窗定「照字面改：
/// 表里记下根的来路」）。给定状态表每个状态一栏（`crate::crash_given_states::ClaimedVersionsOfTheStates`），跟着表录进判定库、进块键、
/// 进 GPU 判表（`crate::crash_judge_tables` 的 `claimed_versions` 那一张）；CPU 判器（`crate::crash_given_states::judge_a_given_state_on_the_cpu`）
/// 与 GPU 恢复内核（`crash_judge_recovery.wgsl` 的 `check_records`）都照它取第二条判据的那一版。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClaimedVersionOfAGivenState {
    /// 看 journal 那一遍恢复在这个状态上落到的那一版：第一截、第三截，与第二截里第一次崩溃那份镜像那个状态（两份崩溃后镜像都是只读恢复
    /// 不改盘的那一份，恢复落到哪一版，自称的就是哪一版）。
    LandedOnByTheConsultedRecovery,
    /// 第二截里挂载与之后那一次发布之后的池那个状态（D13（验证路线） 已定项 7：「恢复自称的那一版取挂载与那次发布写出的最新那条根，
    /// 挂载一条根都没写出时退回第一次崩溃落到的那一版」）：录入段从挂载与那次发布的写表里取最后一个根槽写写出的根；一个都没有时是第一次崩溃之后
    /// 看 journal 那一遍恢复落到的那一版，那一遍恢复失败是 None。这个状态上看 journal 那一遍落到的根不是它，判红
    /// （[`verdict_bits::CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY`]）。
    NewestRootWrittenByTheMountAndItsPublish(Option<(InstanceGeneration, CheckpointTxg)>),
}

impl ClaimedVersionOfAGivenState {
    /// 第二条判据照哪一版判：这一栏写了就是写的那一版，没写就是看 journal 那一遍恢复落到的 `landed_on`。
    #[must_use]
    pub fn version_the_record_checker_takes(
        self,
        landed_on: Option<(InstanceGeneration, CheckpointTxg)>,
    ) -> Option<(InstanceGeneration, CheckpointTxg)> {
        match self {
            Self::LandedOnByTheConsultedRecovery => landed_on,
            Self::NewestRootWrittenByTheMountAndItsPublish(newest_root_written) => {
                newest_root_written
            }
        }
    }

    /// 这一栏写了恢复自称的那一版，而看 journal 那一遍恢复落到的 `landed_on` 不是它（实例代号与 checkpoint_txg 都比，一边落到根、一边没有也算不是）。
    #[must_use]
    pub fn is_not_the_version_landed_on(
        self,
        landed_on: Option<(InstanceGeneration, CheckpointTxg)>,
    ) -> bool {
        match self {
            Self::LandedOnByTheConsultedRecovery => false,
            Self::NewestRootWrittenByTheMountAndItsPublish(newest_root_written) => {
                newest_root_written != landed_on
            }
        }
    }
}

// ============================== 池级 checker 违例细节里点名的被抛弃根 ==============================
//
// 放在判器一侧：GPU 判器（`crate::crash_judge_gpu`）、CPU 给定状态判法（`crate::crash_given_states`）与判定库（`crate::verdict_store`）都用它。
// 放在 `crate::crash_given_states` 时，GPU 判器引它就把那一份（给定状态表的形态、拼镜像引可写挂载那几份实现文件）连同经 `impl` 头点名
// 那一份类型的流水线（`crate::crash_amplification`）拉进判器的判法闭包（`crate::crash_identity::tests` 那三条判闭包的用例红在这里）。

/// 池级 checker 判 I-7.4（近 K 代块未被复用）时违例细节里点名的被抛弃根那一半认的那一条（`singlefs_checker::walk` 的
/// `judge_blocks_referenced_by_abandoned_roots`：细节以「被抛弃的根（实例 X、txg Y）」起头）。
pub const INVARIANT_WHOSE_DETAIL_NAMES_ABANDONED_ROOTS: &str = "I-7.4";

/// 池级 checker 的违例细节里点名一条被抛弃的根时那一段的开头（后面接实例代号、「、txg 」、checkpoint_txg、「）」）。
const ABANDONED_ROOT_NAMED_IN_THE_DETAIL_PREFIX: &str = "被抛弃的根（实例 ";
/// 点名的那一段里实例代号与 checkpoint_txg 之间的那几个字。
const ABANDONED_ROOT_NAMED_IN_THE_DETAIL_TXG_PREFIX: &str = "、txg ";

/// 池级 checker 判 I-7.4 时违例细节里点名的一条被抛弃的根（E158 装置 `checker_reading_of` 要的那一样；用户 2026-10-14 弹窗定
/// 「GPU 判器加细节输出」）：CPU 判器从细节里读出、GPU 判器在走读内核里记下（`crate::crash_judge_gpu` 的
/// `abandoned_roots_named_by_the_pool_checker_of_the_gpu_verdict`）、判定库按状态存（`crate::verdict_store` 的点名的被抛弃根旁表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbandonedRootNamedByThePoolChecker {
    pub instance: InstanceGeneration,
    pub checkpoint_txg: CheckpointTxg,
}

/// 池级 checker 一条违例细节里点名的被抛弃根，按细节里的次序、重复的照留：每一处「被抛弃的根（实例 X、txg Y）」读出一条，
/// 数字读不出的那一处跳过。池级 checker 只记一条不变量的第一处违例说明，I-7.4 的第一处违例是被抛弃根那一半判的才点名根，
/// 是最新根、候选根或施加出来的版本判的就一条都不点名。
#[must_use]
pub fn abandoned_roots_named_in_a_pool_checker_detail(
    detail: &str,
) -> Vec<AbandonedRootNamedByThePoolChecker> {
    let mut named = Vec::new();
    let mut rest = detail;
    // 迭代上界：细节里这一段的出现次数（每一圈 `rest` 至少往后挪过一个开头）
    while let Some(start) = rest.find(ABANDONED_ROOT_NAMED_IN_THE_DETAIL_PREFIX) {
        let after_the_prefix = &rest[start + ABANDONED_ROOT_NAMED_IN_THE_DETAIL_PREFIX.len()..];
        let instance_digits: String = after_the_prefix
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        let after_the_instance = &after_the_prefix[instance_digits.len()..];
        let txg_digits: String = after_the_instance
            .strip_prefix(ABANDONED_ROOT_NAMED_IN_THE_DETAIL_TXG_PREFIX)
            .map(|after_the_txg_prefix| {
                after_the_txg_prefix
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect()
            })
            .unwrap_or_default();
        if let (Ok(instance), Ok(checkpoint_txg)) =
            (instance_digits.parse::<u32>(), txg_digits.parse::<u64>())
        {
            named.push(AbandonedRootNamedByThePoolChecker {
                instance: InstanceGeneration(instance),
                checkpoint_txg: CheckpointTxg(checkpoint_txg),
            });
        }
        rest = after_the_instance;
    }
    named
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 池级 checker 被抛弃根那一半的两种说明（引用的单元被重新分配或抹头、树表指针全零走不完）都读出点名的那一条根；别的说明一条都不读出。
    #[test]
    fn the_abandoned_root_is_read_from_both_details_of_the_abandoned_root_half_and_from_no_other_detail(
    ) {
        let reused = "被抛弃的根（实例 1、txg 5）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：盘 0 槽 1234 校验和对不上";
        let all_zero = "被抛弃的根（实例 2、txg 8）走不完：根记录里树表指针全零，core 的根记录读者照它读盘 0 槽 0、读不出（这条根本身损坏，不只是它引用的单元被重新分配或抹头）：";
        assert_eq!(
            abandoned_roots_named_in_a_pool_checker_detail(reused),
            [AbandonedRootNamedByThePoolChecker {
                instance: InstanceGeneration(1),
                checkpoint_txg: CheckpointTxg(5),
            }]
        );
        assert_eq!(
            abandoned_roots_named_in_a_pool_checker_detail(all_zero),
            [AbandonedRootNamedByThePoolChecker {
                instance: InstanceGeneration(2),
                checkpoint_txg: CheckpointTxg(8),
            }]
        );
        assert_eq!(
            abandoned_roots_named_in_a_pool_checker_detail(
                "最新根引用的单元校验和对不上：盘 1 槽 9"
            ),
            []
        );
    }

    /// 恢复自称的那一版那一栏：没写的照看 journal 那一遍落到的判、恒不判红；写了的照写的判，与落到的根实例代号或 checkpoint_txg 有一样不同、
    /// 一边落到根一边没有都判红，相同不判红。红了出一项记录核对器的红项，从库里读回的红项还原出同一位。
    #[test]
    fn a_claimed_version_other_than_the_one_landed_on_takes_the_record_checker_and_reddens_its_own_bit(
    ) {
        let newest = Some((InstanceGeneration(1), CheckpointTxg(7)));
        let older = Some((InstanceGeneration(1), CheckpointTxg(6)));
        let other_instance = Some((InstanceGeneration(2), CheckpointTxg(7)));
        let not_written = ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery;
        let written = ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(newest);
        assert_eq!(not_written.version_the_record_checker_takes(older), older);
        assert_eq!(written.version_the_record_checker_takes(older), newest);
        assert!(!not_written.is_not_the_version_landed_on(older));
        assert!(!not_written.is_not_the_version_landed_on(None));
        assert!(!written.is_not_the_version_landed_on(newest));
        for landed_on in [older, other_instance, None] {
            assert!(
                written.is_not_the_version_landed_on(landed_on),
                "写的是 {newest:?}、落到 {landed_on:?}：判红"
            );
        }
        assert!(
            ClaimedVersionOfAGivenState::NewestRootWrittenByTheMountAndItsPublish(None)
                .is_not_the_version_landed_on(older),
            "写的那一版是没有根、落到了一条根：判红"
        );
        let words = CrossCheckWords {
            red_bits: 1 << verdict_bits::CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY,
            pool_checker_violated_bits: 0,
            consulted_pass_landed_root: LandedRootWords::NOT_LANDED,
            ignored_pass_landed_root: LandedRootWords::NOT_LANDED,
            consulted_pass_outcome: RECOVERY_OUTCOME_CODE_FAILED,
            ignored_pass_outcome: RECOVERY_OUTCOME_CODE_FAILED,
            consulted_pass_failure_point: GPU_FAILURE_POINT_NO_VALID_ROOT,
            ignored_pass_failure_point: GPU_FAILURE_POINT_NO_VALID_ROOT,
            pool_checker_judged_bits: 0,
            scan_rebuild_outcome: SCAN_REBUILD_OUTCOME_NOT_REACHED,
            gpu_could_not_judge: false,
        };
        let items = red_items_of_a_given_state(&words);
        assert_eq!(
            items,
            [format!(
                "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_VERSION_NOT_LANDED_ON_BY_THE_RECOVERY}"
            )]
        );
        let read_back = cross_check_words_of_a_stored_given_state(
            &items,
            recovery_words_of_the_cross_check_words(&words),
        )
        .expect("这一项认得");
        assert!(claimed_version_is_not_landed_on_in_the_cross_check_words(
            &read_back
        ));
        assert_eq!(read_back.red_bits, words.red_bits);
    }

    /// 给定状态那一路的红项从对拍字换出、存进库，读回来还原出同样的红位、违例位与恢复字（判到位不存，读回是 0）；
    /// oracle 那十四位在对拍之前去掉（两侧都不判不比）。
    #[test]
    fn a_given_state_read_back_from_its_red_items_and_recovery_words_keeps_its_red_bits() {
        let words = CrossCheckWords {
            red_bits: (1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT)
                | (1 << verdict_bits::CONSULTED_ORACLE_FIRST)
                | (1 << (verdict_bits::IGNORED_ORACLE_FIRST + 6)),
            pool_checker_violated_bits: (1u128 << 0) | (1u128 << 2),
            consulted_pass_landed_root: LandedRootWords {
                instance: 3,
                checkpoint_txg_low: 9,
            },
            ignored_pass_landed_root: LandedRootWords::NOT_LANDED,
            consulted_pass_outcome: RECOVERY_OUTCOME_CODE_FILE_READ,
            ignored_pass_outcome: RECOVERY_OUTCOME_CODE_FAILED,
            consulted_pass_failure_point: GPU_FAILURE_POINT_NONE,
            ignored_pass_failure_point: GPU_FAILURE_POINT_NO_VALID_ROOT,
            pool_checker_judged_bits: (1u128 << 0) | (1u128 << 1) | (1u128 << 2),
            scan_rebuild_outcome: SCAN_REBUILD_OUTCOMES[0].code,
            gpu_could_not_judge: false,
        };
        let without_the_oracle = cross_check_words_without_the_oracle(words);
        assert_eq!(
            without_the_oracle.red_bits,
            1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT,
            "两遍 oracle 的位都去掉，记录核对器那一位留着"
        );
        let items = red_items_of_a_given_state(&without_the_oracle);
        assert_eq!(
            items,
            vec![
                format!(
                    "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{}",
                    IMPLEMENTED_INVARIANTS[0]
                ),
                format!(
                    "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{}",
                    IMPLEMENTED_INVARIANTS[2]
                ),
                format!(
                    "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
                ),
            ]
        );
        let read_back = cross_check_words_of_a_stored_given_state(
            &items,
            recovery_words_of_the_cross_check_words(&without_the_oracle),
        )
        .expect("这一路写的红项读得回");
        let mut without_the_judged_bits = without_the_oracle;
        without_the_judged_bits.pool_checker_judged_bits = 0;
        without_the_judged_bits.scan_rebuild_outcome = SCAN_REBUILD_OUTCOME_NOT_REACHED;
        assert_eq!(
            read_back, without_the_judged_bits,
            "判到位与 I-1.13 的结局码不存，别的逐字段相同"
        );
        assert!(
            cross_check_words_of_a_stored_given_state(
                &["journal_consulted_oracle:no_root_chosen".to_string()],
                recovery_words_of_the_cross_check_words(&without_the_oracle),
            )
            .is_err(),
            "oracle 的红项不是这一路写得出的"
        );
    }

    /// J7：一块的计数向量清单里每条不变量一行，没判到的状态（不适用、部分没判，连同池级 checker 没报的）都记进不适用：
    /// 不适用 = 状态数 − 判到。几段并起来按条相加。
    #[test]
    fn the_count_vector_of_a_block_puts_every_state_not_judged_on_an_invariant_under_not_applicable(
    ) {
        let first_invariant = IMPLEMENTED_INVARIANTS[0];
        let second_invariant = IMPLEMENTED_INVARIANTS[1];
        let mut first_piece = StateCountsOfABlock::default();
        first_piece.judged_states_of_each_invariant.insert(
            first_invariant,
            InvariantStateCounts {
                judged: 2,
                violated: 1,
                not_applicable: 0,
            },
        );
        let mut second_piece = StateCountsOfABlock::default();
        second_piece.judged_states_of_each_invariant.insert(
            first_invariant,
            InvariantStateCounts {
                judged: 1,
                violated: 0,
                not_applicable: 0,
            },
        );
        first_piece.absorb(second_piece);
        let count_vector = first_piece.into_count_vector(5);
        assert_eq!(
            count_vector.per_invariant.len(),
            IMPLEMENTED_INVARIANTS.len(),
            "清单里每条不变量一行"
        );
        assert_eq!(
            count_vector.per_invariant.get(first_invariant),
            Some(&InvariantStateCounts {
                judged: 3,
                violated: 1,
                not_applicable: 2,
            }),
            "两段相加：判到 3、违例 1，五个状态里另两个不适用"
        );
        assert_eq!(
            count_vector.per_invariant.get(second_invariant),
            Some(&InvariantStateCounts {
                judged: 0,
                violated: 0,
                not_applicable: 5,
            }),
            "一个状态都没判到的那一条五个都不适用"
        );
    }

    /// J7：按发布分的态数逐状态记、几段并起来相加，进计数向量；没记过的发布不出现（不写 0）。
    #[test]
    fn the_count_vector_of_a_block_carries_the_states_of_each_publish_counted_state_by_state() {
        let mut first_piece = StateCountsOfABlock::default();
        first_piece.count_the_publish_of_a_state(CrashPointReplayPublishOfState::AfterTheLastRoot);
        first_piece.count_the_publish_of_a_state(CrashPointReplayPublishOfState::AfterTheLastRoot);
        let mut second_piece = StateCountsOfABlock::default();
        second_piece
            .count_the_publish_of_a_state(CrashPointReplayPublishOfState::EveryWritePersisted);
        second_piece.count_states_of_a_publish(CrashPointReplayPublishOfState::AfterTheLastRoot, 3);
        second_piece
            .count_states_of_a_publish(CrashPointReplayPublishOfState::EveryWritePersisted, 0);
        first_piece.absorb(second_piece);
        let count_vector = first_piece.into_count_vector(6);
        assert_eq!(
            count_vector.states_by_publish,
            BTreeMap::from([
                ("after_the_last_root".to_string(), 5),
                ("every_write_persisted".to_string(), 1),
            ]),
            "两段相加：最后一次根槽写之后 2 + 3 个，全部持久那一个；记 0 个的不出现"
        );
        assert!(
            count_vector.per_observer.is_empty(),
            "判器那一份不带观察者计数"
        );
    }

    /// J8、M4 第 8 条：失败点码表不挂 `gpu` 特性；恢复字八个字的次序是看 journal 那一遍落到的根两个字、结局、失败点，
    /// 再是不看 journal 那一遍的同四样（这一份用例在不编 `gpu` 特性的库单测里也跑）。
    #[test]
    fn the_recovery_words_of_a_state_are_the_two_passes_roots_outcomes_and_failure_points_without_the_gpu_feature(
    ) {
        let words = CrossCheckWords {
            red_bits: 0,
            pool_checker_violated_bits: 0,
            consulted_pass_landed_root: LandedRootWords {
                instance: 2,
                checkpoint_txg_low: 7,
            },
            ignored_pass_landed_root: LandedRootWords::NOT_LANDED,
            consulted_pass_outcome: RECOVERY_OUTCOME_CODE_FILE_READ,
            ignored_pass_outcome: RECOVERY_OUTCOME_CODE_FAILED,
            consulted_pass_failure_point: GPU_FAILURE_POINT_NONE,
            ignored_pass_failure_point: gpu_failure_point_code(&RecoveryFailure::NoValidRoot),
            pool_checker_judged_bits: 0,
            scan_rebuild_outcome: SCAN_REBUILD_OUTCOME_NOT_REACHED,
            gpu_could_not_judge: false,
        };
        assert_eq!(
            recovery_words_of_the_cross_check_words(&words),
            RecoveryWordsOfState([
                2,
                7,
                RECOVERY_OUTCOME_CODE_FILE_READ,
                GPU_FAILURE_POINT_NONE,
                u32::MAX,
                u32::MAX,
                RECOVERY_OUTCOME_CODE_FAILED,
                GPU_FAILURE_POINT_NO_VALID_ROOT,
            ])
        );
        assert_ne!(
            gpu_failure_point_code(&RecoveryFailure::NoValidRoot),
            gpu_failure_point_code(&RecoveryFailure::NoDeviceInThePool),
            "两个成员两个码"
        );
    }

    /// 一遍择不到根的恢复（搭对拍字的构造函数用：这几条用例只看池级 checker 那两种位）。
    fn recovery_report_that_chose_no_root() -> singlefs_core::recovery::RecoveryReport {
        singlefs_core::recovery::RecoveryReport {
            outcome: RecoveryOutcome::Failed {
                root: None,
                failure: RecoveryFailure::NoValidRoot,
            },
            effective_root: None,
            journal: singlefs_core::recovery::JournalScanReport::default(),
            mapping_fallbacks: 0,
            devices_without_a_mountable_system_configuration_slot: Vec::new(),
        }
    }

    /// CPU 一侧两个构造函数换出的对拍字：崩溃点重放那一份（[`cross_check_words_of_cpu_judgement`]）与给定状态那一份
    /// （[`cross_check_words_of_a_given_state`]），池级 checker 取同一份判定。
    fn cross_check_words_of_both_cpu_constructors(
        pool_checker: Vec<(&'static str, InvariantVerdict)>,
    ) -> [CrossCheckWords; 2] {
        let judgement = CrashStateJudgement {
            consulted: recovery_report_that_chose_no_root(),
            ignored: recovery_report_that_chose_no_root(),
            root_persisted: false,
            consulted_oracle: None,
            ignored_oracle: None,
            pool_checker: pool_checker.clone(),
            records: singlefs_harness::memory_pool::RecordCheck::default(),
        };
        [
            cross_check_words_of_cpu_judgement(&judgement),
            cross_check_words_of_a_given_state(
                &recovery_report_that_chose_no_root(),
                &recovery_report_that_chose_no_root(),
                &pool_checker,
                singlefs_harness::memory_pool::RecordCheck::default(),
                ClaimedVersionOfAGivenState::LandedOnByTheConsultedRecovery,
            ),
        ]
    }

    /// 扫描重建那一条（I-1.13）在池级 checker 清单里的下标。
    fn scan_rebuild_invariant_position() -> usize {
        IMPLEMENTED_INVARIANTS
            .iter()
            .position(|candidate| {
                *candidate == singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX
            })
            .expect("I-1.13 在池级 checker 的清单里")
    }

    /// C821 的口径：I-1.13 判成立时 CPU 两个构造函数换出的判到位第 92 位是 1、违例位第 92 位是 0（GPU 扫描重建那一截照同一口径置位，
    /// 对拍逐位比）。
    #[test]
    fn the_cpu_cross_check_words_set_the_scan_rebuild_judged_bit_when_the_scan_rebuild_invariant_holds(
    ) {
        let scan_rebuild_bit = 1u128 << scan_rebuild_invariant_position();
        for words in cross_check_words_of_both_cpu_constructors(vec![(
            singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
            InvariantVerdict::Holds,
        )]) {
            assert_eq!(
                words.pool_checker_judged_bits, scan_rebuild_bit,
                "I-1.13 成立：判到位只有第 92 位（{words:?}）"
            );
            assert_eq!(
                words.pool_checker_violated_bits, 0,
                "I-1.13 成立：违例位一位都不置（{words:?}）"
            );
        }
    }

    /// C821 的口径：I-1.13 判违例时 CPU 两个构造函数换出的违例位与判到位第 92 位都是 1。
    #[test]
    fn the_cpu_cross_check_words_set_the_scan_rebuild_violated_and_judged_bits_when_the_scan_rebuild_invariant_is_violated(
    ) {
        let scan_rebuild_bit = 1u128 << scan_rebuild_invariant_position();
        for words in cross_check_words_of_both_cpu_constructors(vec![(
            singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
            InvariantVerdict::Violated("扫描重建与遍历方向对不上".to_string()),
        )]) {
            assert_eq!(
                (
                    words.pool_checker_violated_bits,
                    words.pool_checker_judged_bits
                ),
                (scan_rebuild_bit, scan_rebuild_bit),
                "I-1.13 违例：违例位与判到位都只有第 92 位（{words:?}）"
            );
        }
    }

    /// 补上扫描重建内核之后对拍比较处不再去掉第 92 位（C821）：两组位只差第 92 位判对不上（GPU 没置、CPU 置了与反过来各一次），
    /// 两边都置第 92 位而别的位相同判对得上。
    #[test]
    fn the_cross_check_comparison_of_pool_checker_bits_compares_the_scan_rebuild_bit() {
        let scan_rebuild_bit = 1u128 << scan_rebuild_invariant_position();
        let shared_bits = (1u128 << 0) | (1u128 << 40);
        assert!(
            !pool_checker_bits_agree_in_the_cross_check(
                shared_bits,
                shared_bits | scan_rebuild_bit
            ),
            "只差第 92 位（GPU 没置、CPU 置了）判对不上"
        );
        assert!(
            !pool_checker_bits_agree_in_the_cross_check(
                shared_bits | scan_rebuild_bit,
                shared_bits
            ),
            "只差第 92 位（GPU 置了、CPU 没置）判对不上"
        );
        assert!(
            pool_checker_bits_agree_in_the_cross_check(
                shared_bits | scan_rebuild_bit,
                shared_bits | scan_rebuild_bit
            ),
            "两边都置第 92 位、别的位相同判对得上"
        );
    }

    /// 对拍字的比较不再去掉违例位与判到位的第 92 位：CPU 判 I-1.13 违例、GPU 那一份只在第 92 位上不同时判对不上。
    #[test]
    fn the_cross_check_words_differing_only_in_the_scan_rebuild_bits_disagree() {
        let [cpu_words, _] = cross_check_words_of_both_cpu_constructors(vec![
            (IMPLEMENTED_INVARIANTS[0], InvariantVerdict::Holds),
            (
                singlefs_checker::image::SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
                InvariantVerdict::Violated("扫描重建与遍历方向对不上".to_string()),
            ),
        ]);
        assert!(
            cross_check_words_agree_in_the_cross_check(&cpu_words, &cpu_words),
            "同一份对拍字判对得上"
        );
        let mut gpu_words_without_the_scan_rebuild_violation = cpu_words;
        gpu_words_without_the_scan_rebuild_violation.pool_checker_violated_bits = 0;
        assert!(
            !cross_check_words_agree_in_the_cross_check(
                &gpu_words_without_the_scan_rebuild_violation,
                &cpu_words
            ),
            "违例位只差第 92 位判对不上"
        );
        let mut gpu_words_without_the_scan_rebuild_judgement = cpu_words;
        gpu_words_without_the_scan_rebuild_judgement.pool_checker_judged_bits = 1;
        assert!(
            !cross_check_words_agree_in_the_cross_check(
                &gpu_words_without_the_scan_rebuild_judgement,
                &cpu_words
            ),
            "判到位只差第 92 位判对不上"
        );
    }

    /// 内核不置位那张表是空的（C821 还清：I-1.13 由扫描重建那一截判，清单只往末尾追加，I-1.13 照旧是下标 92）：对拍不去掉任何一位；
    /// 只读快照那一批的 I-3.6（下标 93）、I-3.7（下标 94）由引用与快照那一截判；多文件删除那一批新立的 I-9.40（下标 102）由扫描内核的
    /// 删除态扫描判（判决 `research/prompts/m3-multi-file-unlink-r1-main-verification.md` 第三节 M2，不留成「进覆盖表、内核不置位」的欠账）。
    /// 表里又登记回一条时这里红。
    #[test]
    fn no_invariant_is_left_unset_by_the_gpu_kernels() {
        let position_of = |invariant: &str| {
            IMPLEMENTED_INVARIANTS
                .iter()
                .position(|candidate| *candidate == invariant)
                .expect("在池级 checker 的清单里")
        };
        assert!(
            POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS.is_empty(),
            "内核不置位的表是空的，实际 {POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS:?}"
        );
        assert_eq!(
            scan_rebuild_invariant_position(),
            92,
            "I-1.13 是清单第 93 条（下标 92），内核里的 INVARIANT_I_1_13 照这个下标置位"
        );
        assert_eq!(
            position_of(
                singlefs_checker::image::EXTENT_UPPER_SEGMENT_IS_THE_CANONICAL_SHAPE_OF_THE_INODES_WITH_AN_ENTRY
            ),
            102,
            "I-9.40 是清单第 103 条（下标 102），扫描内核删除态扫描里的 INVARIANT_I_9_40 照这个下标置位、不进内核不置位的表"
        );
        assert_eq!(
            (
                position_of(
                    singlefs_checker::image::DEADLIST_ENTRIES_ARE_BORN_NO_LATER_THAN_THE_PREVIOUS_SNAPSHOT_OF_THEIR_SIDE
                ),
                position_of(singlefs_checker::image::LIVE_SNAPSHOTS_ARE_ENUMERABLE)
            ),
            (93, 94),
            "I-3.6、I-3.7 是清单第 94、95 条（下标 93、94），内核里的 INVARIANT_I_3_6、INVARIANT_I_3_7 照这两个下标置位"
        );
        assert_eq!(
            POOL_CHECKER_BITS_LEFT_OUT_OF_THE_CROSS_CHECK, 0,
            "对拍不去掉任何一位"
        );
    }

    /// 里程碑四自建 GC 判的那五条（I-4.4 下标 73，I-3.15、I-5.6、I-3.16、I-1.14 下标 95–98）对拍逐位比：GPU 与 CPU 只在其中一位上不同，
    /// 判器级的比较判对不上（那一位列进内核不置位的表、比较处把它从两边去掉时这里红）。
    #[test]
    fn the_compaction_and_relocation_invariants_are_compared_bit_by_bit_in_the_cross_check() {
        let position_of = |invariant: &str| {
            IMPLEMENTED_INVARIANTS
                .iter()
                .position(|candidate| *candidate == invariant)
                .expect("在池级 checker 的清单里")
        };
        for invariant in [
            singlefs_checker::image::INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
            singlefs_checker::image::UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
            singlefs_checker::image::COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
            singlefs_checker::image::FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
            singlefs_checker::image::UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
        ] {
            let bit = 1u128 << position_of(invariant);
            assert!(
                !pool_checker_bits_agree_in_the_cross_check(bit, 0),
                "{invariant} 的违例位只在 GPU 一侧置了，对拍判对不上"
            );
            assert!(
                !pool_checker_bits_agree_in_the_cross_check(0, bit),
                "{invariant} 的违例位只在 CPU 一侧置了，对拍判对不上"
            );
        }
    }

    /// 红项编码：逐项换行接起来存，读回来逐项相同、次序不变；绿的状态是空向量，读回一项都没有。
    #[test]
    fn the_red_items_read_back_from_their_encoding_item_by_item_in_order() {
        let items = vec![
            "pool_checker:I-3.1".to_string(),
            "record_checker:root_without_record".to_string(),
        ];
        let encoded = encoded_verdict_of_the_red_items(&items);
        assert_eq!(red_items_of_the_encoded_verdict(&encoded), items);
        assert_eq!(encoded_verdict_of_the_red_items(&[]).0, Vec::<u8>::new());
        assert!(
            red_items_of_the_encoded_verdict(&encoded_verdict_of_the_red_items(&[])).is_empty()
        );
    }

    // ---------------- 扫描重建结局表与池级 checker 源码里 I-1.13 能报的理由逐句相等 ----------------

    /// 判定累加器（`singlefs_checker::image::Judgements`）上第一个实参点名一条不变量的四种调用。
    #[derive(Clone, Copy, Debug)]
    enum JudgementsMethodNamingAnInvariant {
        Judge,
        NotApplicable,
        PartOfTheRangeNotJudged,
        PartiallyJudged,
    }

    impl JudgementsMethodNamingAnInvariant {
        const EVERY_METHOD: [Self; 4] = [
            Self::Judge,
            Self::NotApplicable,
            Self::PartOfTheRangeNotJudged,
            Self::PartiallyJudged,
        ];

        fn name(self) -> &'static str {
            match self {
                Self::Judge => "judge",
                Self::NotApplicable => "not_applicable",
                Self::PartOfTheRangeNotJudged => "part_of_the_range_not_judged",
                Self::PartiallyJudged => "partially_judged",
            }
        }
    }

    /// 源码里一处调用的一个实参：去掉首尾空白之后的原文与它在源码里的字节起点。
    struct CallArgument {
        start: usize,
        text: String,
    }

    /// 池级 checker 的一份源码（`crates/singlefs-checker/src/` 下），截到测试模块之前（测试里造的判定不是池级 checker 能报的）。
    fn pool_checker_source_before_its_tests(file_name: &str) -> String {
        let path = format!(
            "{}/../singlefs-checker/src/{file_name}",
            env!("CARGO_MANIFEST_DIR")
        );
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("读得到池级 checker 的源码 {path}：{error}"));
        match source.find("\n#[cfg(test)]\nmod tests {") {
            Some(end) => source[..end].to_string(),
            None => source,
        }
    }

    /// `//` 注释换成同样字节数的空格（字节位置不变，注释里提到的名字不算源码里的一处）；串字面量里的字不动。
    ///
    /// # Panics
    /// 源码里有这个简单读法认不全的写法（块注释、原始串、`'"'` 字符字面量）。
    fn source_with_line_comments_blanked(source: &str) -> String {
        for unsupported in ["/*", "r#\"", "'\"'"] {
            assert!(
                !source.contains(unsupported),
                "池级 checker 源码里出现了 {unsupported:?}：这份扫描只认行注释与普通串字面量，要先补读法"
            );
        }
        let mut blanked = String::with_capacity(source.len());
        let mut inside_a_string_literal = false;
        let mut inside_a_line_comment = false;
        let mut the_previous_character_escapes = false;
        let mut characters = source.chars().peekable();
        while let Some(character) = characters.next() {
            if inside_a_line_comment {
                if character == '\n' {
                    inside_a_line_comment = false;
                    blanked.push('\n');
                } else {
                    blanked.push_str(&" ".repeat(character.len_utf8()));
                }
                continue;
            }
            if inside_a_string_literal {
                blanked.push(character);
                if the_previous_character_escapes {
                    the_previous_character_escapes = false;
                } else if character == '\\' {
                    the_previous_character_escapes = true;
                } else if character == '"' {
                    inside_a_string_literal = false;
                }
                continue;
            }
            if character == '/' && characters.peek() == Some(&'/') {
                inside_a_line_comment = true;
                blanked.push(' ');
                continue;
            }
            if character == '"' {
                inside_a_string_literal = true;
            }
            blanked.push(character);
        }
        blanked
    }

    /// 从 `open` 那个开括号（`(`、`[`、`{` 之一）起，配对的闭括号的字节位置；串字面量里的括号不算。
    fn position_of_the_matching_bracket(source: &str, open: usize) -> usize {
        let mut depth = 0usize;
        let mut inside_a_string_literal = false;
        let mut the_previous_character_escapes = false;
        for (offset, character) in source[open..].char_indices() {
            if inside_a_string_literal {
                if the_previous_character_escapes {
                    the_previous_character_escapes = false;
                } else if character == '\\' {
                    the_previous_character_escapes = true;
                } else if character == '"' {
                    inside_a_string_literal = false;
                }
                continue;
            }
            match character {
                '"' => inside_a_string_literal = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return open + offset;
                    }
                }
                _ => {}
            }
        }
        panic!("第 {open} 字节起的括号没有配对的闭括号");
    }

    /// 源码里 `.<方法名>(` 的每一处调用：调用的字节起点与按顶层逗号切开的实参（括号、串字面量里的逗号不切）。
    fn method_calls_in(source: &str, method_name: &str) -> Vec<(usize, Vec<CallArgument>)> {
        let call_opening = format!(".{method_name}(");
        source
            .match_indices(call_opening.as_str())
            .map(|(call_start, _)| {
                let open = call_start + call_opening.len() - 1;
                let close = position_of_the_matching_bracket(source, open);
                let mut arguments = Vec::new();
                let mut argument_start = open + 1;
                let mut depth = 0usize;
                let mut inside_a_string_literal = false;
                let mut the_previous_character_escapes = false;
                let mut push_the_argument_ending_at =
                    |end: usize, arguments_so_far: &mut Vec<CallArgument>| {
                        let raw = &source[argument_start..end];
                        let text = raw.trim();
                        if !text.is_empty() {
                            arguments_so_far.push(CallArgument {
                                start: argument_start + (raw.len() - raw.trim_start().len()),
                                text: text.to_string(),
                            });
                        }
                        argument_start = end + 1;
                    };
                for (offset, character) in source[open + 1..close].char_indices() {
                    if inside_a_string_literal {
                        if the_previous_character_escapes {
                            the_previous_character_escapes = false;
                        } else if character == '\\' {
                            the_previous_character_escapes = true;
                        } else if character == '"' {
                            inside_a_string_literal = false;
                        }
                        continue;
                    }
                    match character {
                        '"' => inside_a_string_literal = true,
                        '(' | '[' | '{' => depth += 1,
                        ')' | ']' | '}' => depth -= 1,
                        ',' if depth == 0 => {
                            push_the_argument_ending_at(open + 1 + offset, &mut arguments);
                        }
                        _ => {}
                    }
                }
                push_the_argument_ending_at(close, &mut arguments);
                (call_start, arguments)
            })
            .collect()
    }

    /// 一个串字面量实参（`"…"`）里的字；不是串字面量是 `None`。
    fn string_literal_contents(argument: &str) -> Option<String> {
        let contents = argument.strip_prefix('"')?.strip_suffix('"')?;
        assert!(
            !contents.contains('\\') && !contents.contains('"'),
            "串字面量 {argument} 里有转义或引号：这份扫描不展开转义"
        );
        Some(contents.to_string())
    }

    /// 一段源码里每个串字面量的字，按出现次序。
    fn string_literals_in(source: &str) -> Vec<String> {
        let mut literals = Vec::new();
        let mut rest = source;
        while let Some(open) = rest.find('"') {
            let after_open = &rest[open + 1..];
            let close = after_open.find('"').expect("串字面量有收尾的引号");
            literals.push(after_open[..close].to_string());
            rest = &after_open[close + 1..];
        }
        literals
    }

    /// 源码里 `const <名字>: &str =` 那个串常量的值。
    fn string_constant_in(source: &str, constant_name: &str) -> Option<String> {
        let declaration = format!("const {constant_name}: &str =");
        let after = &source[source.find(declaration.as_str())? + declaration.len()..];
        let literal_end = after.find(';')?;
        string_literal_contents(after[..literal_end].trim())
    }

    /// 调用所在的那个函数从哪个字节起（往前找最近的一个顶格 `fn`）。
    fn start_of_the_function_enclosing(source: &str, position: usize) -> usize {
        ["\nfn ", "\npub fn ", "\npub(crate) fn "]
            .iter()
            .filter_map(|opening| source[..position].rfind(opening))
            .max()
            .expect("调用在某个顶格的函数里")
    }

    /// 第一个实参点不点名 I-1.13：直接写那个常量或 `"I-1.13"`，或是遍历 `IMPLEMENTED_INVARIANTS` 那一圈的 `invariant`
    /// （往前最近的一处 `invariant` 绑定就是那一圈的头）。别的写法不点名它——别处另写到 I-1.13 的，
    /// [`every_reason_the_pool_checker_can_give_for_the_scan_rebuild_is_a_row_of_the_outcome_table`] 另查、判红。
    fn first_argument_names_the_scan_rebuild_invariant(
        source: &str,
        call_start: usize,
        first_argument: &str,
    ) -> bool {
        if first_argument == "SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX"
            || first_argument == "\"I-1.13\""
        {
            return true;
        }
        if first_argument != "invariant" {
            return false;
        }
        // 只在调用所在的函数里找绑定：函数参数叫 invariant 的，调用方传进来的那个常量另由「别处另写到 I-1.13」那一查管
        let within_the_function =
            &source[start_of_the_function_enclosing(source, call_start)..call_start];
        let latest_loop_start = match (
            within_the_function.rfind("for invariant in "),
            within_the_function.rfind("let invariant"),
        ) {
            (Some(loop_start), Some(binding_start)) => {
                (loop_start > binding_start).then_some(loop_start)
            }
            (Some(loop_start), None) => Some(loop_start),
            (None, Some(_) | None) => None,
        };
        latest_loop_start.is_some_and(|loop_start| {
            let header = &within_the_function[loop_start..];
            header.starts_with("for invariant in crate::image::IMPLEMENTED_INVARIANTS {")
                || header.starts_with("for invariant in IMPLEMENTED_INVARIANTS {")
        })
    }

    /// 理由枚举的 `fn reason` 在池级 checker 的哪一份源码里。
    #[derive(Clone, Copy, Debug)]
    enum PoolCheckerSourceFile {
        Walk,
        Rebuild,
    }

    /// I-1.13 那几处 `not_applicable` / `part_of_the_range_not_judged` 的理由实参认成的一句或几句原文。认法：串字面量；
    /// `walk.rs` 里的串常量；绑在遍历方向已判损坏那张表上的 `reason`（那张表的第二栏）；`IndexTableTheWalkDoesNotReadInFull` 与
    /// `rebuild::ScanRebuildNotDecided` 的 `reason()`（各自 `fn reason` 里的每一句，绑定在同一个函数的 `Err(…)` 里）。
    ///
    /// # Panics
    /// 认不得的实参写法：池级 checker 多了一种理由的来源，在这里补认法。
    fn reasons_of_the_argument(
        walk: &str,
        rebuild: &str,
        call_start: usize,
        reason_argument: &str,
    ) -> Vec<String> {
        const REASON_ENUMS_OF_THE_POOL_CHECKER: [(&str, &str, PoolCheckerSourceFile); 2] = [
            (
                "table_not_read_in_full",
                "IndexTableTheWalkDoesNotReadInFull",
                PoolCheckerSourceFile::Walk,
            ),
            (
                "not_decided",
                "ScanRebuildNotDecided",
                PoolCheckerSourceFile::Rebuild,
            ),
        ];
        if let Some(literal) = string_literal_contents(reason_argument) {
            return vec![literal];
        }
        if let Some(value) = string_constant_in(walk, reason_argument) {
            return vec![value];
        }
        let enclosing_function =
            &walk[start_of_the_function_enclosing(walk, call_start)..call_start];
        if reason_argument == "reason" {
            assert!(
                enclosing_function.contains(
                    "Some((_, reason)) = INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED"
                ),
                "实参 reason 绑在遍历方向已判损坏那张表上"
            );
            return singlefs_checker::walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED
                .iter()
                .map(|(_, reason)| (*reason).to_string())
                .collect();
        }
        for (binding, enum_name, source_file) in REASON_ENUMS_OF_THE_POOL_CHECKER {
            if reason_argument != format!("{binding}.reason()") {
                continue;
            }
            assert!(
                enclosing_function.contains(&format!("Err({binding}) =>")),
                "{binding} 绑在同一个函数的 Err({binding}) 里"
            );
            let enum_source = match source_file {
                PoolCheckerSourceFile::Walk => walk,
                PoolCheckerSourceFile::Rebuild => rebuild,
            };
            let implementation_start = enum_source
                .find(&format!("impl {enum_name} {{"))
                .unwrap_or_else(|| panic!("源码里有 impl {enum_name}"));
            let reason_function = implementation_start
                + enum_source[implementation_start..]
                    .find("fn reason(self) -> &'static str {")
                    .unwrap_or_else(|| panic!("impl {enum_name} 里有 fn reason"));
            let body_open =
                reason_function + enum_source[reason_function..].find('{').expect("函数体");
            let body =
                &enum_source[body_open..=position_of_the_matching_bracket(enum_source, body_open)];
            let literals = string_literals_in(body);
            assert_eq!(
                literals.len(),
                body.matches("=>").count(),
                "{enum_name}::reason 的每一臂都是一句串字面量"
            );
            return literals;
        }
        panic!("认不得 I-1.13 的这一种理由实参：{reason_argument}（池级 checker 多了一种理由的来源，在这里补认法）");
    }

    /// 结局表（[`SCAN_REBUILD_OUTCOMES`]）里不适用的每一句，与扫描池级 checker 源码认出的 I-1.13 能报的每一句不适用理由逐句相等：
    /// `walk.rs` 里判定累加器四种调用的第一个实参点名 I-1.13 的那几处（直接写那个常量，或遍历 `IMPLEMENTED_INVARIANTS` 的那一圈——
    /// 走到扫描重建之前整池交回的那几处），理由实参照 [`reasons_of_the_argument`] 认；`partially_judged` 不报 I-1.13，成立与违例那一处在。
    /// `walk.rs` 里别处写到 I-1.13 或 `IMPLEMENTED_INVARIANTS`（不是那几种调用的第一个实参、不是 import、不是那一圈的头）、别的源码文件写到
    /// I-1.13、`image.rs` 里有调用报 I-1.13，都判红：这份扫描认不得它报什么。`Judgements::into_report` 的缺省理由不在里面
    /// （每条路径都给 I-1.13 记了判定或理由：四处整池交回各记一句，其余都走到 `judge_scan_rebuild_agrees_with_the_walked_index`，那里每条出口都记）。
    /// 表少一句、多一句、或一句的字与池级 checker 的不同，这条红。
    #[test]
    fn every_reason_the_pool_checker_can_give_for_the_scan_rebuild_is_a_row_of_the_outcome_table() {
        const SCAN_REBUILD_INVARIANT_TOKEN: &str = "SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX";
        const SCAN_REBUILD_INVARIANT_LITERAL: &str = "\"I-1.13\"";
        const LOOP_OVER_EVERY_INVARIANT: &str =
            "for invariant in crate::image::IMPLEMENTED_INVARIANTS {";
        let walk =
            source_with_line_comments_blanked(&pool_checker_source_before_its_tests("walk.rs"));
        let rebuild =
            source_with_line_comments_blanked(&pool_checker_source_before_its_tests("rebuild.rs"));
        let line_of = |source: &str, position: usize| source[..position].matches('\n').count() + 1;

        let mut reasons_in_the_source = std::collections::BTreeSet::new();
        let mut first_argument_starts = std::collections::BTreeSet::new();
        let mut is_judged_in_the_source = false;
        for method in JudgementsMethodNamingAnInvariant::EVERY_METHOD {
            for (call_start, arguments) in method_calls_in(&walk, method.name()) {
                let first = arguments.first().unwrap_or_else(|| {
                    panic!(
                        "walk.rs 第 {} 行的 {} 调用有实参",
                        line_of(&walk, call_start),
                        method.name()
                    )
                });
                first_argument_starts.insert(first.start);
                if !first_argument_names_the_scan_rebuild_invariant(&walk, call_start, &first.text)
                {
                    continue;
                }
                match method {
                    JudgementsMethodNamingAnInvariant::Judge => is_judged_in_the_source = true,
                    JudgementsMethodNamingAnInvariant::PartiallyJudged => panic!(
                        "walk.rs 第 {} 行把 I-1.13 报成部分没判：结局表里没有这一种",
                        line_of(&walk, call_start)
                    ),
                    JudgementsMethodNamingAnInvariant::NotApplicable
                    | JudgementsMethodNamingAnInvariant::PartOfTheRangeNotJudged => {
                        let reason = arguments.get(1).unwrap_or_else(|| {
                            panic!(
                                "walk.rs 第 {} 行的 {} 调用有理由实参",
                                line_of(&walk, call_start),
                                method.name()
                            )
                        });
                        reasons_in_the_source.extend(reasons_of_the_argument(
                            &walk,
                            &rebuild,
                            call_start,
                            &reason.text,
                        ));
                    }
                }
            }
        }

        let import_start = walk
            .find("use crate::image::{")
            .expect("walk.rs 从 image 引名字");
        let import_end = import_start + walk[import_start..].find("};").expect("import 收尾");
        for (token_start, _) in walk.match_indices(SCAN_REBUILD_INVARIANT_TOKEN) {
            assert!(
                (import_start..import_end).contains(&token_start)
                    || first_argument_starts.contains(&token_start),
                "walk.rs 第 {} 行另写到 I-1.13（不是判定累加器调用的第一个实参，也不在 import 里）：这份扫描认不得它报什么，补认法",
                line_of(&walk, token_start)
            );
        }
        for (token_start, _) in walk.match_indices(SCAN_REBUILD_INVARIANT_LITERAL) {
            assert!(
                first_argument_starts.contains(&token_start),
                "walk.rs 第 {} 行另写到 \"I-1.13\"：这份扫描认不得它报什么，补认法",
                line_of(&walk, token_start)
            );
        }
        for (token_start, _) in walk.match_indices("IMPLEMENTED_INVARIANTS") {
            let loop_header_start = token_start + "IMPLEMENTED_INVARIANTS".len();
            assert!(
                walk[..loop_header_start].ends_with(&LOOP_OVER_EVERY_INVARIANT[..LOOP_OVER_EVERY_INVARIANT.len() - 2]),
                "walk.rs 第 {} 行另写到 IMPLEMENTED_INVARIANTS（不是遍历每条不变量那一圈的头）：这份扫描认不得它报什么，补认法",
                line_of(&walk, token_start)
            );
        }
        let checker_source_directory =
            format!("{}/../singlefs-checker/src", env!("CARGO_MANIFEST_DIR"));
        let mut file_names: Vec<String> = std::fs::read_dir(&checker_source_directory)
            .unwrap_or_else(|error| panic!("读得到 {checker_source_directory}：{error}"))
            .map(|entry| {
                entry
                    .expect("读得到目录项")
                    .file_name()
                    .into_string()
                    .expect("文件名是 UTF-8")
            })
            .filter(|file_name| file_name.ends_with(".rs"))
            .collect();
        file_names.sort();
        assert!(
            file_names.iter().any(|file_name| file_name == "walk.rs")
                && file_names.iter().any(|file_name| file_name == "image.rs"),
            "池级 checker 的源码里有 walk.rs 与 image.rs：{file_names:?}"
        );
        for file_name in &file_names {
            let source =
                source_with_line_comments_blanked(&pool_checker_source_before_its_tests(file_name));
            match file_name.as_str() {
                "walk.rs" => {}
                "image.rs" => {
                    for method in JudgementsMethodNamingAnInvariant::EVERY_METHOD {
                        for (call_start, arguments) in method_calls_in(&source, method.name()) {
                            let first = arguments
                                .first()
                                .map_or("", |argument| argument.text.as_str());
                            assert!(
                                !first_argument_names_the_scan_rebuild_invariant(&source, call_start, first),
                                "image.rs 第 {} 行的 {} 调用报 I-1.13：这份扫描只认 walk.rs 里那几处，补认法",
                                line_of(&source, call_start),
                                method.name()
                            );
                        }
                    }
                }
                _ => assert!(
                    !source.contains(SCAN_REBUILD_INVARIANT_TOKEN)
                        && !source.contains(SCAN_REBUILD_INVARIANT_LITERAL),
                    "{file_name} 写到了 I-1.13：这份扫描只认 walk.rs 里那几处，补认法"
                ),
            }
        }

        let reasons_in_the_table: std::collections::BTreeSet<String> = SCAN_REBUILD_OUTCOMES
            .iter()
            .filter_map(|outcome| match outcome.verdict_of_the_pool_checker {
                ScanRebuildVerdictOfThePoolChecker::NotApplicable(reason) => {
                    Some(reason.to_string())
                }
                ScanRebuildVerdictOfThePoolChecker::Holds
                | ScanRebuildVerdictOfThePoolChecker::Violated => None,
            })
            .collect();
        let rows_that_are_not_applicable = SCAN_REBUILD_OUTCOMES
            .iter()
            .filter(|outcome| {
                matches!(
                    outcome.verdict_of_the_pool_checker,
                    ScanRebuildVerdictOfThePoolChecker::NotApplicable(_)
                )
            })
            .count();
        assert_eq!(
            reasons_in_the_table.len(),
            rows_that_are_not_applicable,
            "结局表里不适用的理由一句一行，不重"
        );
        let missing_from_the_table: Vec<&String> = reasons_in_the_source
            .difference(&reasons_in_the_table)
            .collect();
        let not_given_by_the_pool_checker: Vec<&String> = reasons_in_the_table
            .difference(&reasons_in_the_source)
            .collect();
        assert!(
            missing_from_the_table.is_empty() && not_given_by_the_pool_checker.is_empty(),
            "结局表与池级 checker 源码里 I-1.13 能报的不适用理由逐句相等（源码里认出 {} 句、表里 {} 句）。\n表里缺的：{missing_from_the_table:#?}\n池级 checker 报不出的：{not_given_by_the_pool_checker:#?}",
            reasons_in_the_source.len(),
            reasons_in_the_table.len()
        );
        assert!(
            is_judged_in_the_source,
            "walk.rs 里有判 I-1.13 成立或违例的那一处（结局表的成立、违例两行对着它）"
        );
        for verdict in [
            ScanRebuildVerdictOfThePoolChecker::Holds,
            ScanRebuildVerdictOfThePoolChecker::Violated,
        ] {
            assert_eq!(
                SCAN_REBUILD_OUTCOMES
                    .iter()
                    .filter(|outcome| outcome.verdict_of_the_pool_checker == verdict)
                    .count(),
                1,
                "结局表里 {verdict:?} 恰好一行"
            );
        }
    }
}

//! 判器一侧的结论写法（崩溃点重放合并第三轮判决 J8、第三轮正文 H8「改结论的几处（红项编码、恢复字、GPU 结论换红项）搬进判器一侧的文件」）：
//! 一个状态判完之后，存进判定库的那几样怎么从判定换出来——红项的编码（[`encoded_verdict_of_the_red_items`]）、恢复字（两遍各自落到的根、
//! 走读结局与失败点，[`recovery_words_of_the_cross_check_words`]；对拍比的那几个字 [`CrossCheckWords`] 的 CPU 一侧
//! [`cross_check_words_of_cpu_judgement`]）、失败点码表（[`gpu_failure_point_code`]，GPU 内核里的常量照它生成）、计数向量
//! （[`StateCountsOfABlock`]：逐条不变量判到 / 违例 / 不适用、按发布分的态数，存成 [`BlockCountVector`]）。
//!
//! 不挂特性：没编进 `gpu` 的二进制判的 CPU 结论照样带恢复字（失败点码表不在 `gpu` 特性里）。这一份在两个判器的闭包里：
//! CPU 判器（`crate::crash`）里的 [`Layer0StateJudgement`] 的方法写在这里（`impl` 头点名它，闭包按 `impl` 头收进来），
//! GPU 判器（`crate::crash_judge_gpu`）引它；改这里一个字，GPU 判器与每份设点文件的 CPU 判器版本都换。
//! 流水线（`crate::crash_amplification`）与判定库（`crate::verdict_store`）只调这里：判定库只管这几样的字节布局，不解释它们。

use std::collections::BTreeMap;

use singlefs_checker::image::{InvariantVerdict, IMPLEMENTED_INVARIANTS};
use singlefs_core::recovery::{RecoveryFailure, RecoveryOutcome};

use crate::crash::{
    Layer0OracleViolationKind, Layer0PublishOfState, Layer0StateJudgement,
    LAYER0_RED_PASS_POOL_CHECKER, LAYER0_RED_PASS_RECORD_CHECKER,
    RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT, RECORD_CHECKER_ROOT_WITHOUT_RECORD,
};

// ============================== 红项的编码 ==============================

/// 一个状态的判定向量，按 [`encoded_verdict_of_the_red_items`] 的编码存成字节（判定库不解释它）；两份字节相同就是同一个判定向量。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EncodedVerdictVector(pub Vec<u8>);

/// 判定向量里红项之间的分隔：一项一行。
const RED_ITEM_SEPARATOR: &str = "\n";

/// 一个状态存进库的判定向量：红项（`Layer0StateJudgement::red_items` 那种写法）逐项、换行接起来的字节；绿的状态是空向量。
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

impl Layer0StateJudgement {
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
/// （`crate::crash::Layer0PublishOfState::name`）。不适用的到存块时按状态数补：每个状态每条不变量要么判到（成立或违例）、要么不适用
/// （连同部分没判，与层 0 计数同一口径：`crash::Layer0Tally::checker_not_applicable_states`；GPU 那一份「判到位没置」也是这一格，
/// 判到位的口径照 `crate::crash_judge_gpu` 文件头）。观察者计数不在这里数，观察者那一份结论另存（[`BlockCountVector::per_observer`]）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateCountsOfABlock {
    judged_states_of_each_invariant: BTreeMap<&'static str, InvariantStateCounts>,
    states_by_publish: BTreeMap<String, u64>,
}

impl StateCountsOfABlock {
    /// CPU 判器一个状态的池级 checker 判定记进来。
    pub fn count_the_cpu_judgement(&mut self, judgement: &Layer0StateJudgement) {
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
    pub fn count_the_publish_of_a_state(&mut self, publish: Layer0PublishOfState) {
        self.count_states_of_a_publish(publish, 1);
    }

    /// 同一次发布的 `states` 个状态一起记进来（GPU 判器那一路按块的序号区间与各段归哪次发布现算，`crate::crash_amplification`）。
    pub fn count_states_of_a_publish(&mut self, publish: Layer0PublishOfState, states: u64) {
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
    /// 看 journal 那一遍 oracle 的七类从这一位起（与 `Layer0OracleViolationKind::EVERY_KIND` 同序）。
    pub const CONSULTED_ORACLE_FIRST: u32 = 0;
    /// 不看 journal 那一遍从这一位起。
    pub const IGNORED_ORACLE_FIRST: u32 = 8;
    pub const ROOT_WITHOUT_RECORD: u32 = 16;
    pub const CLAIMED_STATE_MISSING_UNIT: u32 = 17;
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
/// CPU 那一侧由 [`cross_check_words_of_cpu_judgement`] 换出来，两份逐字段相等才算对上（CPU 那一侧判不了恒为假：GPU 判不了的状态对不上）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrossCheckWords {
    /// 结论第 0 字去掉判不了那一位：两遍 oracle 各七类（[`verdict_bits`]）、记录核对器两条。
    pub red_bits: u32,
    /// 池级 checker 的违例位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`）。
    pub pool_checker_violated_bits: u128,
    pub consulted_pass_landed_root: LandedRootWords,
    pub ignored_pass_landed_root: LandedRootWords,
    pub consulted_pass_outcome: u32,
    pub ignored_pass_outcome: u32,
    pub consulted_pass_failure_point: u32,
    pub ignored_pass_failure_point: u32,
    /// 池级 checker 逐条判到位（成立或违例）。
    pub pool_checker_judged_bits: u128,
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

/// oracle 一类违例在红位里的位置（与 `Layer0OracleViolationKind::EVERY_KIND` 同序）。
fn oracle_kind_position(kind: Layer0OracleViolationKind) -> u32 {
    u32::try_from(
        Layer0OracleViolationKind::EVERY_KIND
            .iter()
            .position(|candidate| *candidate == kind)
            .expect("七类之一"),
    )
    .expect("七类，装得进 u32")
}

/// 池级 checker 一份判定的违例位与判到位（位 i ↔ `IMPLEMENTED_INVARIANTS[i]`；判到 = 成立或违例，不适用与部分没判都不算）。
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
pub fn cross_check_words_of_cpu_judgement(judgement: &Layer0StateJudgement) -> CrossCheckWords {
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
        gpu_could_not_judge: false,
    }
}

// ============================== 给定状态（随机历史两项）的对拍字与红项 ==============================

/// 红位里两遍 oracle 的那十四位（看 journal 那一遍七类、不看那一遍七类，[`verdict_bits`]）：给定状态表那一路两边都不判不比
/// （用户 2026-10-10 定「第一截与第三截都上 GPU」：随机历史两项的 CPU 判法今天不跑层 0 的 oracle，也不给期望版本表）。
#[must_use]
pub fn oracle_red_bits() -> u32 {
    Layer0OracleViolationKind::EVERY_KIND
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
/// 不从 [`Layer0StateJudgement`] 来，那一份带 oracle，这一路不判 oracle：红位里 oracle 那十四位恒为 0）。
#[must_use]
pub fn cross_check_words_of_a_given_state(
    consulted: &singlefs_core::recovery::RecoveryReport,
    ignored: &singlefs_core::recovery::RecoveryReport,
    pool_checker: &[(&'static str, InvariantVerdict)],
    records: singlefs_harness::memory_pool::RecordCheck,
) -> CrossCheckWords {
    let mut red_bits = 0u32;
    if records.root_without_record {
        red_bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
    }
    if records.claimed_state_missing_unit {
        red_bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
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
/// `Layer0StateJudgement::red_items` 同名；oracle 不判，不出项）。GPU 与 CPU 两侧都从对拍字换出来，同一个状态两侧的项逐项同序。
#[must_use]
pub fn red_items_of_a_given_state(words: &CrossCheckWords) -> Vec<String> {
    let mut items: Vec<String> = IMPLEMENTED_INVARIANTS
        .iter()
        .enumerate()
        .filter(|(position, _)| words.pool_checker_violated_bits & (1u128 << position) != 0)
        .map(|(_, invariant)| format!("{LAYER0_RED_PASS_POOL_CHECKER}:{invariant}"))
        .collect();
    let records = record_check_of_the_cross_check_words(words);
    if records.root_without_record {
        items.push(format!(
            "{LAYER0_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_ROOT_WITHOUT_RECORD}"
        ));
    }
    if records.claimed_state_missing_unit {
        items.push(format!(
            "{LAYER0_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
        ));
    }
    items
}

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
            .strip_prefix(LAYER0_RED_PASS_RECORD_CHECKER)
            .and_then(|rest| rest.strip_prefix(':'));
        let pool_checker_item = item
            .strip_prefix(LAYER0_RED_PASS_POOL_CHECKER)
            .and_then(|rest| rest.strip_prefix(':'));
        match (record_item, pool_checker_item) {
            (Some(RECORD_CHECKER_ROOT_WITHOUT_RECORD), None) => {
                red_bits |= 1 << verdict_bits::ROOT_WITHOUT_RECORD;
            }
            (Some(RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT), None) => {
                red_bits |= 1 << verdict_bits::CLAIMED_STATE_MISSING_UNIT;
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
        gpu_could_not_judge: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
                    "{LAYER0_RED_PASS_POOL_CHECKER}:{}",
                    IMPLEMENTED_INVARIANTS[0]
                ),
                format!(
                    "{LAYER0_RED_PASS_POOL_CHECKER}:{}",
                    IMPLEMENTED_INVARIANTS[2]
                ),
                format!(
                    "{LAYER0_RED_PASS_RECORD_CHECKER}:{RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT}"
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
        assert_eq!(
            read_back, without_the_judged_bits,
            "判到位不存，别的逐字段相同"
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
        first_piece.count_the_publish_of_a_state(Layer0PublishOfState::AfterTheLastRoot);
        first_piece.count_the_publish_of_a_state(Layer0PublishOfState::AfterTheLastRoot);
        let mut second_piece = StateCountsOfABlock::default();
        second_piece.count_the_publish_of_a_state(Layer0PublishOfState::EveryWritePersisted);
        second_piece.count_states_of_a_publish(Layer0PublishOfState::AfterTheLastRoot, 3);
        second_piece.count_states_of_a_publish(Layer0PublishOfState::EveryWritePersisted, 0);
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
}

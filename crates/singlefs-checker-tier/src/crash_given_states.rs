//! 给定状态表（崩溃点重放合并第三轮判决 J3、J5、J6；用户 2026-10-10 定的四件）：表的形态、一个状态的结论、判器 trait
//! [`GivenStatesJudge`] 与它不经判定库的 CPU 就地判（[`GivenStatesJudgedOnTheCpuInPlace`]、[`judge_a_given_state_on_the_cpu`]）。
//! 随机历史两项（崩溃注入、读故障）建表交它判，崩溃点重放的引擎（`crate::crash_amplification::GivenStatesEngine`）实现这个 trait。
//! 这一份从 `crash_injection` 拆出来：它不引随机历史脚手架、不引崩溃注入，流水线引它时，层 0 用例经流水线收的判法闭包里就没有
//! 崩溃注入与随机历史（与 `crate::writable_mount_phases` 同一种拆法）；崩溃注入经 `pub use` 照旧转出，用例从那里引。

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::recovery::{recover, JournalPolicy};
use singlefs_harness::memory_pool::{
    CrashImage, MemoryPool, RecordCheck, RecordStreamContinuity, RecordStreamSeam, RetainedWrite,
};

use crate::crash::check_records_against;
use crate::crash_judge_conclusions::{
    cross_check_words_of_a_given_state, record_check_of_the_cross_check_words,
    red_items_of_a_given_state, BlockCountVector, CrossCheckWords, StateCountsOfABlock,
};
use crate::writable_mount_phases::materialized_crash_image;

/// 一张给定状态表：同一个基线、同一张写表、同一组记录流接缝上的一批崩溃状态，每个状态一个持久集合（与写表逐条对应）。
/// 基线是 mkfs 之后的池、写表从 mkfs 第一个不是清零的写起（用户 2026-10-10 定「基线改成 mkfs 之后」；mkfs 的数据写留在写表头上、每个状态里都落了，
/// 创世那几次发布照样进两侧记录核对器的发布表，`crate::crash_injection::HistoryAfterTheMakeFilesystem`）。
/// 第一截一张表是一段历史上掉电接缝相同的那几个第一次崩溃状态；第三截一张表是一个第一次崩溃状态上的二次崩溃（写表是 mkfs 之后的历史
/// 接挂载那一段、再接之后那一次发布，在历史与挂载之间断开，[`RecordStreamSeam::AfterACrash`]）。这几个状态上只读一张镜像的那几样
/// （看 journal 与不看 journal 两遍恢复、池级 checker、记录核对器）交 [`GivenStatesJudge`] 判：配了 GPU 走 GPU 判器的给定状态表。
pub struct GivenStatesOfATable<'table> {
    /// 这张表在这一项里的名字（进判定库的块键：哪段历史、哪一截、哪一个第一次崩溃状态）。
    pub table_name: String,
    pub base: &'table MemoryPool,
    pub writes: &'table [RetainedWrite],
    /// 写表的切段（每个写都在一段里）：给定状态表不按段展开，切段只供建判表时摆撕裂镜像。
    pub segments: &'table [Vec<usize>],
    /// 记录核对器分发布的接缝，按写表下标从小到大；空是一条录制流。
    pub seams: &'table [RecordStreamSeam],
    pub persisted_sets: Vec<Vec<bool>>,
}

impl GivenStatesOfATable<'_> {
    /// 记录核对器拿到的这张写表是怎么来的。
    #[must_use]
    pub fn continuity(&self) -> RecordStreamContinuity<'_> {
        if self.seams.is_empty() {
            RecordStreamContinuity::OneRecording
        } else {
            RecordStreamContinuity::BrokenAtSeams(self.seams)
        }
    }
}

/// 一个给定状态的结论：对拍比的那几个字（两遍 oracle 的位去掉，`crash_judge_conclusions::cross_check_words_without_the_oracle`），
/// 连同这一趟在 CPU 上判出的池级 checker 违例细节（GPU 判的、从判定库读回的没有细节，是空的）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GivenStateConclusion {
    pub words: CrossCheckWords,
    pub pool_checker_violation_details: Vec<(&'static str, String)>,
}

/// 结论里没有细节的池级 checker 违例（GPU 判的、从判定库读回的）照这一句记：要细节就在 CPU 上重判那一个状态。
pub const POOL_CHECKER_VIOLATION_WITHOUT_ITS_DETAIL: &str =
    "违例位置了（这一份结论是 GPU 判器判的或从判定库读回的，不带细节：要细节在 CPU 上重判这一个状态）";

impl GivenStateConclusion {
    /// 池级 checker 判红的那几条（不变量、细节），按 `IMPLEMENTED_INVARIANTS` 的次序：这一趟 CPU 判出的带它的细节，没有的写
    /// [`POOL_CHECKER_VIOLATION_WITHOUT_ITS_DETAIL`]。
    #[must_use]
    pub fn pool_checker_violations(&self) -> Vec<(&'static str, String)> {
        singlefs_checker::image::IMPLEMENTED_INVARIANTS
            .iter()
            .enumerate()
            .filter(|(position, _)| {
                self.words.pool_checker_violated_bits & (1u128 << position) != 0
            })
            .map(|(_, invariant)| {
                let detail = self
                    .pool_checker_violation_details
                    .iter()
                    .find(|(violated, _)| violated == invariant)
                    .map_or_else(
                        || POOL_CHECKER_VIOLATION_WITHOUT_ITS_DETAIL.to_string(),
                        |(_, detail)| detail.clone(),
                    );
                (*invariant, detail)
            })
            .collect()
    }

    /// 记录核对器两条判据在这个状态上的结论。
    #[must_use]
    pub fn record_check(&self) -> RecordCheck {
        record_check_of_the_cross_check_words(&self.words)
    }

    /// 这个状态存进判定库的红项（池级 checker 违例逐条、记录核对器两条；`crash_judge_conclusions::red_items_of_a_given_state`）：空就是不红。
    #[must_use]
    pub fn red_items(&self) -> Vec<String> {
        red_items_of_a_given_state(&self.words)
    }
}

/// 一张给定状态表的结论：逐状态（与 [`GivenStatesOfATable::persisted_sets`] 同序），加这张表的计数向量（逐条不变量判到 / 违例 / 不适用，
/// 崩溃点重放合并第三轮判决 J7；从判定库读回的照存的那一份）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GivenStatesConclusionsOfATable {
    pub states: Vec<GivenStateConclusion>,
    pub count_vector: BlockCountVector,
    pub source: GivenStatesConclusionsSource,
}

/// 一张给定状态表的结论是怎么来的：这台机器判的（或从判定库读回的），还是两台分片里这张表归别的机器判、库里又没有时交的占位结论
/// （不红、两遍都没落到根：调用方照它往下走，这一趟的计数偏少、新发现可能漏，崩溃注入把这几张表记进报告，入口照「本次未跑」交回）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GivenStatesConclusionsSource {
    JudgedOrReadBackOnThisHost,
    PlaceholderOfATableLeftToAnotherHost,
}

/// 给定状态表交谁判。两个真实现：[`GivenStatesJudgedOnTheCpuInPlace`]（不经判定库，在调用的这条线程上用 CPU 判器判；没编进
/// `verdict-store` 特性的二进制只有它）与崩溃点重放的引擎 `crate::crash_amplification::GivenStatesEngine`（照本机多机配置走 GPU 或 CPU，
/// 配了 GPU 时 CPU 同一批也判、逐状态比，结论按表存进判定库，第三轮正文 H3「也是这个入口与这个判定库里的项」）。
/// 崩溃注入的工作线程各自调它，所以要 `Sync`。
pub trait GivenStatesJudge: Sync {
    /// 调用方叫没叫停（崩溃点重放的引擎照流水线那一侧的叫停层：叫停文件在不在，见过一次就记住）：崩溃注入每领一片种子之前问一次，
    /// 叫停了就不再领（手里那一片判完，判过的表在库里），交回的报告记着叫停留下了几段历史（`crate::crash_injection::CrashInjectionReport::histories_left_by_the_stop_request`）。
    fn stop_is_requested(&self) -> bool;

    /// 判一张表的每个状态。
    ///
    /// # Errors
    /// 判不了这张表（判定库读写失败、配了 GPU 而卡用不上……）：原因写在串里，调用方照它判红，不退回别处判。
    fn judge_the_given_states(
        &self,
        table: &GivenStatesOfATable<'_>,
    ) -> Result<GivenStatesConclusionsOfATable, String>;
}

/// 不经判定库、在调用的这条线程上逐状态用 CPU 判器判（[`judge_the_given_states_on_the_cpu`]）。
pub struct GivenStatesJudgedOnTheCpuInPlace;

impl GivenStatesJudge for GivenStatesJudgedOnTheCpuInPlace {
    /// 就地判没有叫停这一层：不经引擎的跑法从来不叫停。
    fn stop_is_requested(&self) -> bool {
        false
    }

    fn judge_the_given_states(
        &self,
        table: &GivenStatesOfATable<'_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        Ok(judge_the_given_states_on_the_cpu(table))
    }
}

/// CPU 判器判一张给定状态表的每个状态（[`judge_a_given_state_on_the_cpu`]），计数向量照对拍字数（[`count_vector_of_the_given_states`]）。
/// 引擎的 CPU 那一路与配了 GPU 时的对照那一遍也调它。
#[must_use]
pub fn judge_the_given_states_on_the_cpu(
    table: &GivenStatesOfATable<'_>,
) -> GivenStatesConclusionsOfATable {
    let states: Vec<GivenStateConclusion> = table
        .persisted_sets
        .iter()
        .map(|persisted| judge_a_given_state_on_the_cpu(table, persisted))
        .collect();
    let words: Vec<CrossCheckWords> = states.iter().map(|state| state.words).collect();
    GivenStatesConclusionsOfATable {
        count_vector: count_vector_of_the_given_states(&words),
        states,
        source: GivenStatesConclusionsSource::JudgedOrReadBackOnThisHost,
    }
}

/// 一张表各状态的对拍字数出的计数向量：判到位与违例位逐条记（`StateCountsOfABlock::count_the_gpu_bits`，GPU 与 CPU 两侧同一个数法：
/// 判到 = 成立或违例，不适用与部分没判都进不适用）；按发布分的态数不数（给定状态不按段归发布）。
#[must_use]
pub fn count_vector_of_the_given_states(words: &[CrossCheckWords]) -> BlockCountVector {
    let mut counts = StateCountsOfABlock::default();
    for state in words {
        counts.count_the_gpu_bits(
            state.pool_checker_judged_bits,
            state.pool_checker_violated_bits,
        );
    }
    counts.into_count_vector(u64::try_from(words.len()).expect("状态数装得进 u64"))
}

/// CPU 判器判一个给定状态：基线叠上持久了的写落成一个池（[`materialized_crash_image`]），看 journal 与不看 journal 各恢复一遍、
/// 跑池级 checker、跑记录核对器（崩溃态镜像与恢复后的镜像都是这一份：只读恢复不改盘；恢复自称的状态取看 journal 那一遍落到的根；
/// 写表照这张表的接缝分发布）。oracle 不判（这一路不给期望版本表）。
///
/// # Panics
/// 持久集合与写表不一样长。
#[must_use]
pub fn judge_a_given_state_on_the_cpu(
    table: &GivenStatesOfATable<'_>,
    persisted: &[bool],
) -> GivenStateConclusion {
    assert_eq!(
        persisted.len(),
        table.writes.len(),
        "给定状态的持久集合与这张表的写表逐条对应"
    );
    let pool = materialized_crash_image(&CrashImage {
        base: table.base,
        writes: table.writes,
        persisted: persisted.to_vec(),
    });
    let consulted = recover(&pool, JournalPolicy::Consult);
    let ignored = recover(&pool, JournalPolicy::Ignore);
    let pool_checker = check_pool_image(&pool);
    let records = check_records_against(
        &pool,
        &pool,
        table.writes,
        persisted,
        table.continuity(),
        consulted.effective_root,
    );
    let pool_checker_violation_details: Vec<(&'static str, String)> = pool_checker
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect();
    GivenStateConclusion {
        words: cross_check_words_of_a_given_state(&consulted, &ignored, &pool_checker, records),
        pool_checker_violation_details,
    }
}

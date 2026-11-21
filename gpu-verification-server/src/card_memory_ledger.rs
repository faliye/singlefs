//! 一张卡上驻留的判器占多少显存，与按额度收判器的账。
//!
//! 一个判器占的：表，加一次派活带的态数（至少 1、不超过这条流的态数）的草稿区与结论、回读缓冲，按 MiB 向上取整；
//! 与验证包 `crash_judge_dispatch::mebibytes_a_judge_needs` 同一个算法（`tests/card_memory_matches_the_verification_package.rs` 逐个对拍）。
//! 驱动那一份（`MEBIBYTES_LEFT_FOR_THE_DRIVER`）每张卡只付一次：一张卡上判器合计不超过「这张卡给服务的额度 − 驱动那一份」。
//! 新装那一刻表在卡上占两份（[`mebibytes_while_loading_a_judge`]）：驻留的账只记一份，另一份由批次调度挑新流、分草稿区时留出来。
//! 账只能经 [`CardMemoryLedger::admit`] 收判器，越过额度的收不进；一张卡这一波之后驻留哪几个判器只能由账交出
//! （[`ResidentJudgesWithinTheQuota`]，字段私有），所以越界的计划构造不出来。
//!
//! 核对器（验证包 `crash_verify_gpu::GpuFactsVerifier`，核对待判行那一族）不驻留：服务主循环派它上卡之前照这一张卡驻留的判器记一本账、
//! 经 [`CardMemoryLedger::lend_to_a_facts_verifier`] 借给它（借不到就先换出驻留的判器再借），核完工作线程丢掉核对器，借的随之还回去。
//! 它占的（[`mebibytes_of_a_facts_verifier`]）照它建的缓冲算：上卡一次的六张事实表（录制流每次写、每对重放、每段、段写表、每单元、每根；
//! 空表也建一个字），表照两份算（验证包 `crash_verify_gpu` 用 `create_buffer_init` 建这六张表，同判器新装那一刻，wgpu 另开一份同样大的暂存，
//! 下一次提交做完才放，[`mebibytes_while_loading_a_judge`]），加一块一趟的头、结论与回读（每态各一个字）；一块接一块核，同时还活着的趟数照验证包
//! `gpu_memory_ledger::PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD`（丢掉的缓冲等设备下一次收拾才还给驱动），合起来折成 MiB 照
//! `gpu_memory_ledger::mebibytes_of_buffers_held_at_once`。每写、每段、每单元、每根与头的字数取验证包 `crash_verify_gpu` 公开的表宽常量；
//! 每对重放两个字那一边没公开，这里记一处（[`FACTS_VERIFIER_WORDS_PER_REPLAY`]）。编好的管线算在驱动那一份里。

use singlefs_verification::crash_facts::FlowFacts;
use singlefs_verification::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
use singlefs_verification::crash_judge_gpu::{SCRATCH_WORDS, VERDICT_WORDS};
use singlefs_verification::crash_verify_gpu::{
    FACTS_VERIFIER_HEADER_WORDS, FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE,
    FACTS_VERIFIER_WORDS_PER_ROOT, FACTS_VERIFIER_WORDS_PER_SEGMENT, FACTS_VERIFIER_WORDS_PER_UNIT,
};
use singlefs_verification::gpu_memory_ledger::{
    mebibytes_of_buffers_held_at_once, PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
};

use crate::break_switches::{BreakSwitch, BreakSwitches};
use crate::stream_identity::{StreamFacts, StreamKey};

pub const BYTES_PER_MEBIBYTE: u64 = 1024 * 1024;

/// 卡在服务里的号（服务起的时候给每张卡排的，计划与工作线程按它对上卡）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardIndex(pub u32);

/// 一张给服务用的卡：它的号与它给服务的显存额度（MiB，含驱动那一份；真卡上是 `GpuCard::memory_budget_mebibytes`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardOfTheService {
    pub card: CardIndex,
    pub service_quota_mebibytes: u64,
}

impl CardOfTheService {
    /// 这张卡上判器合计能占的（MiB）：额度减驱动那一份，额度比驱动那一份还小的是 0。
    #[must_use]
    pub fn mebibytes_for_the_judges(&self) -> u64 {
        self.service_quota_mebibytes
            .saturating_sub(MEBIBYTES_LEFT_FOR_THE_DRIVER)
    }
}

/// 一个态在卡上占的字节：草稿区 `SCRATCH_WORDS` 个字，结论与回读缓冲各 `VERDICT_WORDS` 个字。
#[must_use]
pub fn bytes_of_one_state_on_the_card() -> u64 {
    u64::try_from(SCRATCH_WORDS * 4 + 2 * VERDICT_WORDS * 4).expect("装得进 u64")
}

/// 一个判器一次派活实际开多少个态的草稿区：要的、这条流的态数取小，至少 1（与验证包
/// `gpu_memory_ledger::states_in_one_dispatch_capped_by_the_state_count` 同一个压法）。
#[must_use]
pub fn states_a_judge_opens(facts: &StreamFacts, requested_states_per_dispatch: u64) -> u64 {
    requested_states_per_dispatch.min(facts.state_count).max(1)
}

/// 一个判器在卡上占多少显存（MiB，向上取整），不算驱动那一份。
#[must_use]
pub fn mebibytes_of_a_judge(facts: &StreamFacts, requested_states_per_dispatch: u64) -> u64 {
    let states = states_a_judge_opens(facts, requested_states_per_dispatch);
    facts.table_mebibytes
        + states
            .checked_mul(bytes_of_one_state_on_the_card())
            .expect("一次派活的草稿区字节数装得进 u64")
            .div_ceil(BYTES_PER_MEBIBYTE)
}

/// 新装一个判器的那一刻卡上要的（MiB）：判器占的（[`mebibytes_of_a_judge`]）再加一份表。判表上卡时验证包用 `create_buffer_init` 建，
/// wgpu 给不带 `MAP_WRITE` 的缓冲另开一份同样大的暂存缓冲，下一次提交做完才放；一波里的判器全装完才判头一批，所以这一波新装的表的暂存
/// 与它们的草稿区同时在显存里。驻留的账（[`CardMemoryLedger::admit`]）只记一份表。
#[must_use]
pub fn mebibytes_while_loading_a_judge(
    facts: &StreamFacts,
    requested_states_per_dispatch: u64,
) -> u64 {
    mebibytes_of_a_judge(facts, requested_states_per_dispatch) + facts.table_mebibytes
}

/// 草稿区一个态至少要几 MiB（一个态的字节向上取整到 MiB）：分草稿区时每个判器至少分到这么多才装得上。
#[must_use]
pub fn mebibytes_of_one_state_rounded_up() -> u64 {
    bytes_of_one_state_on_the_card().div_ceil(BYTES_PER_MEBIBYTE)
}

/// 给 `scratch_mebibytes` 的草稿区，装得下几个态（向下取整）。
#[must_use]
pub fn states_fitting_in(scratch_mebibytes: u64) -> u64 {
    scratch_mebibytes
        .checked_mul(BYTES_PER_MEBIBYTE)
        .expect("草稿区字节数装得进 u64")
        / bytes_of_one_state_on_the_card()
}

/// 核对器 `replays` 那张表每对重放占几个 u32（重放在枚举写表里的下标、被重放的那次写；验证包 `GpuFactsVerifier` 建表时每对推两个字）。
pub const FACTS_VERIFIER_WORDS_PER_REPLAY: u64 = 2;
/// 卡上一个字的字节。
const BYTES_PER_GPU_WORD: u64 = 4;

fn word_count(count: usize) -> u64 {
    u64::try_from(count).expect("表长装得进 u64")
}

/// 一个核对器上卡一次的六张事实表合起来的字节（文件头「核对器」；空表也建一个字）。
#[must_use]
pub fn bytes_of_the_facts_tables_on_the_card(facts: &FlowFacts) -> u64 {
    let replays: u64 = facts
        .torn_images
        .iter()
        .map(|torn| word_count(torn.replays.len()))
        .sum();
    // 段写表里接着段的写、分片的段再接片起点的各位，一位一个字
    let writes_of_the_segments: u64 = facts
        .segments
        .iter()
        .map(|segment| {
            word_count(segment.writes.len())
                + segment
                    .slice_start
                    .as_ref()
                    .map_or(0, |slice_start| word_count(slice_start.digits.len()))
        })
        .sum();
    let words_of_each_table = [
        word_count(facts.is_tearable.len()) * word_count(FACTS_VERIFIER_WORDS_PER_RECORDED_WRITE),
        replays * FACTS_VERIFIER_WORDS_PER_REPLAY,
        word_count(facts.segments.len()) * word_count(FACTS_VERIFIER_WORDS_PER_SEGMENT),
        writes_of_the_segments,
        word_count(facts.units.len()) * word_count(FACTS_VERIFIER_WORDS_PER_UNIT),
        word_count(facts.roots.len()) * word_count(FACTS_VERIFIER_WORDS_PER_ROOT),
    ];
    words_of_each_table
        .iter()
        .map(|words| (*words).max(1) * BYTES_PER_GPU_WORD)
        .sum()
}

/// 核一块一趟建的缓冲的字节：头（`FACTS_VERIFIER_HEADER_WORDS` 个字）与结论（每态一个字）由 `create_buffer_init` 建（提交做完前带一份同样大的暂存，各算两份），
/// 回读每态一个字（`create_buffer` 建，算一份）。
#[must_use]
pub fn bytes_of_one_facts_verifier_pass(states: u64) -> u64 {
    (2 * word_count(FACTS_VERIFIER_HEADER_WORDS) + 3 * states) * BYTES_PER_GPU_WORD
}

/// 一个核对器在卡上借多少显存（MiB，向上取整），不算驱动那一份：六张表两份加同时还活着的几趟（文件头「核对器」），一趟按这一组里最大的那一块的态数算。
#[must_use]
pub fn mebibytes_of_a_facts_verifier(facts: &FlowFacts, states_of_the_largest_block: u64) -> u64 {
    mebibytes_of_a_facts_verifier_under(facts, states_of_the_largest_block, &BreakSwitches::none())
}

/// 同 [`mebibytes_of_a_facts_verifier`]，照弄坏开关算：facts-tables-staging-not-reserved 下六张表只算一份（不留上卡时的暂存）。
#[must_use]
pub fn mebibytes_of_a_facts_verifier_under(
    facts: &FlowFacts,
    states_of_the_largest_block: u64,
    break_switches: &BreakSwitches,
) -> u64 {
    let copies_of_the_tables = if break_switches.is_on(BreakSwitch::FactsTablesStagingNotReserved) {
        1
    } else {
        2
    };
    mebibytes_of_buffers_held_at_once(
        bytes_of_the_facts_tables_on_the_card(facts)
            .checked_mul(copies_of_the_tables)
            .expect("事实表两份的字节数装得进 u64"),
        bytes_of_one_facts_verifier_pass(states_of_the_largest_block),
        PASSES_ALIVE_AT_ONCE_ON_EACH_THREAD,
    )
}

/// 借给核对器时越过了额度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactsVerifierDoesNotFit {
    pub card: CardIndex,
    pub verifier_mebibytes: u64,
    pub free_mebibytes: u64,
}

/// 账上的一个判器：哪条流、一次派活带几个态（装的时候定，之后不变）、占多少显存。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JudgeOnTheCard {
    pub stream: StreamKey,
    pub states_per_dispatch: u64,
    pub mebibytes: u64,
}

/// 收判器时越过了额度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JudgeDoesNotFit {
    pub card: CardIndex,
    pub stream: StreamKey,
    pub judge_mebibytes: u64,
    pub free_mebibytes: u64,
}

/// 一张卡这一波的显存账。
#[derive(Debug)]
pub struct CardMemoryLedger {
    card: CardIndex,
    capacity_mebibytes: u64,
    occupied_mebibytes: u64,
    judges: Vec<JudgeOnTheCard>,
    is_the_quota_checked: bool,
}

impl CardMemoryLedger {
    /// 一本空账：判器合计能占 [`CardOfTheService::mebibytes_for_the_judges`]。
    #[must_use]
    pub fn empty(card: &CardOfTheService, break_switches: &BreakSwitches) -> Self {
        Self {
            card: card.card,
            capacity_mebibytes: card.mebibytes_for_the_judges(),
            occupied_mebibytes: 0,
            judges: Vec::new(),
            is_the_quota_checked: !break_switches.is_on(BreakSwitch::MemoryLedgerUnchecked),
        }
    }

    /// 收一个判器。
    ///
    /// # Errors
    /// 收下它就越过额度（[`JudgeDoesNotFit`]）：账不动。
    ///
    /// # Panics
    /// 同一条流在这张卡上收两次（一张卡上一条流只有一个判器）。
    pub fn admit(
        &mut self,
        stream: StreamKey,
        facts: &StreamFacts,
        states_per_dispatch: u64,
    ) -> Result<(), JudgeDoesNotFit> {
        assert!(
            self.judges.iter().all(|judge| judge.stream != stream),
            "卡 {:?} 上 {stream:?} 收了两次",
            self.card
        );
        let judge_mebibytes = mebibytes_of_a_judge(facts, states_per_dispatch);
        let free_mebibytes = self.free_mebibytes();
        if self.is_the_quota_checked && judge_mebibytes > free_mebibytes {
            return Err(JudgeDoesNotFit {
                card: self.card,
                stream,
                judge_mebibytes,
                free_mebibytes,
            });
        }
        self.occupied_mebibytes += judge_mebibytes;
        self.judges.push(JudgeOnTheCard {
            stream,
            states_per_dispatch,
            mebibytes: judge_mebibytes,
        });
        Ok(())
    }

    /// 借给一个核对器 `verifier_mebibytes`（[`mebibytes_of_a_facts_verifier`]，文件头「核对器」）：借了就越过额度的借不到、账不动。
    /// 借出的那一份不记成驻留的判器：服务主循环只拿这本账判这张卡此刻放不放得下它，判完就丢。
    ///
    /// # Errors
    /// 借了就越过额度（[`FactsVerifierDoesNotFit`]）。
    pub fn lend_to_a_facts_verifier(
        &mut self,
        verifier_mebibytes: u64,
    ) -> Result<(), FactsVerifierDoesNotFit> {
        let free_mebibytes = self.free_mebibytes();
        if self.is_the_quota_checked && verifier_mebibytes > free_mebibytes {
            return Err(FactsVerifierDoesNotFit {
                card: self.card,
                verifier_mebibytes,
                free_mebibytes,
            });
        }
        self.occupied_mebibytes += verifier_mebibytes;
        Ok(())
    }

    /// 此刻还空着多少（MiB）。
    #[must_use]
    pub fn free_mebibytes(&self) -> u64 {
        self.capacity_mebibytes
            .saturating_sub(self.occupied_mebibytes)
    }

    /// 收完了：交出这张卡驻留的判器。
    #[must_use]
    pub fn into_resident_judges(self) -> ResidentJudgesWithinTheQuota {
        ResidentJudgesWithinTheQuota {
            card: self.card,
            capacity_mebibytes: self.capacity_mebibytes,
            occupied_mebibytes: self.occupied_mebibytes,
            judges: self.judges,
        }
    }
}

/// 一张卡驻留的判器，合计不超过额度减驱动那一份：只有 [`CardMemoryLedger::into_resident_judges`] 造得出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResidentJudgesWithinTheQuota {
    card: CardIndex,
    capacity_mebibytes: u64,
    occupied_mebibytes: u64,
    judges: Vec<JudgeOnTheCard>,
}

impl ResidentJudgesWithinTheQuota {
    #[must_use]
    pub fn card(&self) -> CardIndex {
        self.card
    }

    /// 判器合计能占的（MiB）。
    #[must_use]
    pub fn capacity_mebibytes(&self) -> u64 {
        self.capacity_mebibytes
    }

    /// 判器合计占的（MiB）。
    #[must_use]
    pub fn occupied_mebibytes(&self) -> u64 {
        self.occupied_mebibytes
    }

    /// 驻留的判器，按收进账的先后。
    #[must_use]
    pub fn judges(&self) -> &[JudgeOnTheCard] {
        &self.judges
    }
}

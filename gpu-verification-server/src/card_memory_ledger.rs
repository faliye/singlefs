//! 一张卡上驻留的判器占多少显存，与按额度收判器的账。
//!
//! 一个判器占的：表，加一次派活带的态数（至少 1、不超过这条流的态数）的草稿区与结论、回读缓冲，按 MiB 向上取整；
//! 与验证包 `crash_judge_dispatch::mebibytes_a_judge_needs` 同一个算法（`tests/card_memory_matches_the_verification_package.rs` 逐个对拍）。
//! 驱动那一份（`MEBIBYTES_LEFT_FOR_THE_DRIVER`）每张卡只付一次：一张卡上判器合计不超过「这张卡给服务的额度 − 驱动那一份」。
//! 账只能经 [`CardMemoryLedger::admit`] 收判器，越过额度的收不进；一张卡这一波之后驻留哪几个判器只能由账交出
//! （[`ResidentJudgesWithinTheQuota`]，字段私有），所以越界的计划构造不出来。

use singlefs_verification::crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER;
use singlefs_verification::crash_judge_gpu::{SCRATCH_WORDS, VERDICT_WORDS};

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

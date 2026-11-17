//! 弄坏开关 `GPU_VERIFICATION_SERVER_BREAK=<项>[,<项>…]`：只给测试证红用（`.claude/rules/test-script-discipline.md`「测试脚本自己也要被测试证明有效」）。
//! 批次调度、待判池（内存那一份与判定库那一份）与取活的主循环都不读环境：开关经 [`BreakSwitches`] 交进它们，服务进程交的是
//! [`BreakSwitches::none`]；只有测试经 [`BreakSwitches::of_the_environment`] 读这个变量。每一项弄坏哪一处、打开后哪个测试目标里哪一条测试红，
//! 表在 [`BreakSwitch::red_test_target`] 与 [`BreakSwitch::red_test`]，README「弄坏开关」一节逐项相同。

use std::collections::BTreeSet;

/// 环境变量名。
pub const BREAK_VARIABLE: &str = "GPU_VERIFICATION_SERVER_BREAK";

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BreakSwitch {
    /// 一张卡的显存账收判器时不核额度，越过额度的也收下。
    MemoryLedgerUnchecked,
    /// 给新装的判器分草稿区时不扣这张卡上留着的判器占的显存。
    ScratchSplitIgnoresKeptJudges,
    /// 切批时取过的态不从池里那一截扣掉，下一批又从同一个态起切。
    CarvedStatesNotConsumed,
    /// 内存池把一块派出去一部分时，没派到的那一截也当派出去了。
    PoolDropsTheRestOfAPartlyDispatchedBlock,
    /// 不看水位线：池里有态就发。
    WatermarkIgnored,
    /// 不看限时：池里的态不够填满每张卡时，最老的待判等多久都不发。
    WaitLimitIgnored,
    /// 新装的判器一次派活的态数不按这条流的态数压。
    BatchNotCappedByTheStateCount,
    /// 一张卡上新装的几条流不均分剩下的显存，按入池先后先到先拿。
    ScratchSplitByEntryOrder,
    /// 一张卡上一条流这一波能派的批连着派完，才轮到下一条流。
    BatchesOfAStreamBackToBack,
    /// 流不按入池先后排，按键排。
    PoolOrderIgnored,
    /// 等过限时、哪张空着的卡都装不下的流不腾卡给它。
    StarvingStreamsNotRescued,
    /// 不认驻留：已经在卡上的判器不先派回那张卡，当新流重装。
    ResidenceIgnored,
    /// 要给新流腾显存时把这张卡上的判器全换出，有待判态的也换。
    EveryResidentJudgeEvicted,
    /// 调度不把输入排成规范次序，照调用方给的次序走。
    InputOrderFollowed,
    /// 判定库那一份待判池认领时别的持有者持着也当认领到了。
    ClaimHeldByOthersIgnored,
    /// 判定库那一份待判池不对版本：期待的 GPU 判器组键与编进来的不同也取。
    VersionMismatchIgnored,
    /// 判定库那一份待判池认领时另要期待的结论版本键是这个二进制算得出的那几把之一（算不出的不认领、点名版本对不上）。
    ConclusionVersionChecked,
    /// 写回时结论帧不照待判行里的结论版本键编，换成这个二进制算的头一把。
    ConclusionWrittenUnderThisBuildKey,
    /// 强制现判的块也先问库里判完没有，判完的不取。
    ForcedBlockReadsTheLibrary,
    /// 强制现判的块写结论那一帧不写凭据行。
    RejudgedRowNotWritten,
    /// 结论那一帧没写成时结论不留，只打红行。
    RefusedConclusionsDropped,
    /// 起来不开卡，库里有了能取的块才开。
    CardsOpenedOnlyWithWork,
    /// 池空一圈（没有在判的块、这一圈也没认领到）就放掉卡，再来活再开。
    CardsLetGoWhenThePoolIdles,
    /// 判完而可复用的块不删它的待判行（只不取）。
    ReusableBlockPendingRowKept,
    /// 写结论那一帧因待判行读完之后变了被拒时不重读、当场报红。
    PendingRowChangeNotReread,
    /// 写结论那一帧因结论块那一行编帧之后变了被拒时不重编、当场报红。
    GuardRowChangeNotReread,
}

/// 批次调度那几项红在这个测试目标里。
const BATCH_SCHEDULING_TARGET: &str = "batch_scheduling_requirements";
/// 判定库那一份待判池与取活循环持卡那几项红在这个测试目标里。
const LIBRARY_POOL_TARGET: &str = "library_pending_pool_on_a_temporary_library";

impl BreakSwitch {
    pub const EVERY: [BreakSwitch; 26] = [
        BreakSwitch::MemoryLedgerUnchecked,
        BreakSwitch::ScratchSplitIgnoresKeptJudges,
        BreakSwitch::CarvedStatesNotConsumed,
        BreakSwitch::PoolDropsTheRestOfAPartlyDispatchedBlock,
        BreakSwitch::WatermarkIgnored,
        BreakSwitch::WaitLimitIgnored,
        BreakSwitch::BatchNotCappedByTheStateCount,
        BreakSwitch::ScratchSplitByEntryOrder,
        BreakSwitch::BatchesOfAStreamBackToBack,
        BreakSwitch::PoolOrderIgnored,
        BreakSwitch::StarvingStreamsNotRescued,
        BreakSwitch::ResidenceIgnored,
        BreakSwitch::EveryResidentJudgeEvicted,
        BreakSwitch::InputOrderFollowed,
        BreakSwitch::ClaimHeldByOthersIgnored,
        BreakSwitch::VersionMismatchIgnored,
        BreakSwitch::ConclusionVersionChecked,
        BreakSwitch::ConclusionWrittenUnderThisBuildKey,
        BreakSwitch::ForcedBlockReadsTheLibrary,
        BreakSwitch::RejudgedRowNotWritten,
        BreakSwitch::RefusedConclusionsDropped,
        BreakSwitch::CardsOpenedOnlyWithWork,
        BreakSwitch::CardsLetGoWhenThePoolIdles,
        BreakSwitch::ReusableBlockPendingRowKept,
        BreakSwitch::PendingRowChangeNotReread,
        BreakSwitch::GuardRowChangeNotReread,
    ];

    /// 变量里写的项名。
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            BreakSwitch::MemoryLedgerUnchecked => "memory-ledger-unchecked",
            BreakSwitch::ScratchSplitIgnoresKeptJudges => "scratch-split-ignores-kept-judges",
            BreakSwitch::CarvedStatesNotConsumed => "carved-states-not-consumed",
            BreakSwitch::PoolDropsTheRestOfAPartlyDispatchedBlock => {
                "pool-drops-the-rest-of-a-partly-dispatched-block"
            }
            BreakSwitch::WatermarkIgnored => "watermark-ignored",
            BreakSwitch::WaitLimitIgnored => "wait-limit-ignored",
            BreakSwitch::BatchNotCappedByTheStateCount => "batch-not-capped-by-the-state-count",
            BreakSwitch::ScratchSplitByEntryOrder => "scratch-split-by-entry-order",
            BreakSwitch::BatchesOfAStreamBackToBack => "batches-of-a-stream-back-to-back",
            BreakSwitch::PoolOrderIgnored => "pool-order-ignored",
            BreakSwitch::StarvingStreamsNotRescued => "starving-streams-not-rescued",
            BreakSwitch::ResidenceIgnored => "residence-ignored",
            BreakSwitch::EveryResidentJudgeEvicted => "every-resident-judge-evicted",
            BreakSwitch::InputOrderFollowed => "input-order-followed",
            BreakSwitch::ClaimHeldByOthersIgnored => "claim-held-by-others-ignored",
            BreakSwitch::VersionMismatchIgnored => "version-mismatch-ignored",
            BreakSwitch::ConclusionVersionChecked => "conclusion-version-checked",
            BreakSwitch::ConclusionWrittenUnderThisBuildKey => {
                "conclusion-written-under-this-build-key"
            }
            BreakSwitch::ForcedBlockReadsTheLibrary => "forced-block-reads-the-library",
            BreakSwitch::RejudgedRowNotWritten => "rejudged-row-not-written",
            BreakSwitch::RefusedConclusionsDropped => "refused-conclusions-dropped",
            BreakSwitch::CardsOpenedOnlyWithWork => "cards-opened-only-with-work",
            BreakSwitch::CardsLetGoWhenThePoolIdles => "cards-let-go-when-the-pool-idles",
            BreakSwitch::ReusableBlockPendingRowKept => "reusable-block-pending-row-kept",
            BreakSwitch::PendingRowChangeNotReread => "pending-row-change-not-reread",
            BreakSwitch::GuardRowChangeNotReread => "guard-row-change-not-reread",
        }
    }

    /// 打开这一项之后必须红的那一条测试在哪个测试目标里（`tests/<目标>.rs`）。
    #[must_use]
    pub fn red_test_target(self) -> &'static str {
        match self {
            BreakSwitch::ClaimHeldByOthersIgnored
            | BreakSwitch::VersionMismatchIgnored
            | BreakSwitch::ConclusionVersionChecked
            | BreakSwitch::ConclusionWrittenUnderThisBuildKey
            | BreakSwitch::ForcedBlockReadsTheLibrary
            | BreakSwitch::RejudgedRowNotWritten
            | BreakSwitch::RefusedConclusionsDropped
            | BreakSwitch::CardsOpenedOnlyWithWork
            | BreakSwitch::CardsLetGoWhenThePoolIdles
            | BreakSwitch::ReusableBlockPendingRowKept
            | BreakSwitch::PendingRowChangeNotReread
            | BreakSwitch::GuardRowChangeNotReread => LIBRARY_POOL_TARGET,
            BreakSwitch::MemoryLedgerUnchecked
            | BreakSwitch::ScratchSplitIgnoresKeptJudges
            | BreakSwitch::CarvedStatesNotConsumed
            | BreakSwitch::PoolDropsTheRestOfAPartlyDispatchedBlock
            | BreakSwitch::WatermarkIgnored
            | BreakSwitch::WaitLimitIgnored
            | BreakSwitch::BatchNotCappedByTheStateCount
            | BreakSwitch::ScratchSplitByEntryOrder
            | BreakSwitch::BatchesOfAStreamBackToBack
            | BreakSwitch::PoolOrderIgnored
            | BreakSwitch::StarvingStreamsNotRescued
            | BreakSwitch::ResidenceIgnored
            | BreakSwitch::EveryResidentJudgeEvicted
            | BreakSwitch::InputOrderFollowed => BATCH_SCHEDULING_TARGET,
        }
    }

    /// 打开这一项之后必须红的那一条测试（[`Self::red_test_target`] 那个目标里的函数名）。
    #[must_use]
    pub fn red_test(self) -> &'static str {
        match self {
            BreakSwitch::MemoryLedgerUnchecked | BreakSwitch::ScratchSplitIgnoresKeptJudges => {
                "judges_resident_on_a_card_never_exceed_the_quota_less_the_driver_share"
            }
            BreakSwitch::CarvedStatesNotConsumed
            | BreakSwitch::PoolDropsTheRestOfAPartlyDispatchedBlock => {
                "every_pending_state_is_dispatched_exactly_once_and_the_rest_stays_in_the_pool"
            }
            BreakSwitch::WatermarkIgnored | BreakSwitch::WaitLimitIgnored => {
                "a_wave_goes_out_when_every_card_fills_once_or_the_oldest_waited_the_limit_and_never_from_an_empty_pool"
            }
            BreakSwitch::BatchNotCappedByTheStateCount | BreakSwitch::ScratchSplitByEntryOrder => {
                "streams_sharing_a_card_split_the_remaining_memory_and_a_short_stream_takes_its_own_state_count"
            }
            BreakSwitch::BatchesOfAStreamBackToBack => {
                "batches_of_streams_sharing_a_card_alternate_from_wave_to_wave"
            }
            BreakSwitch::PoolOrderIgnored | BreakSwitch::StarvingStreamsNotRescued => {
                "streams_are_taken_in_entry_order_and_a_short_stream_is_not_left_behind_a_long_one"
            }
            BreakSwitch::ResidenceIgnored | BreakSwitch::EveryResidentJudgeEvicted => {
                "a_resident_stream_goes_back_to_its_card_and_only_judges_without_pending_states_are_evicted"
            }
            BreakSwitch::InputOrderFollowed => {
                "the_same_inputs_give_the_same_plan_whatever_order_they_come_in"
            }
            BreakSwitch::ClaimHeldByOthersIgnored => {
                "a_block_claimed_by_another_holder_is_left_to_it"
            }
            BreakSwitch::VersionMismatchIgnored => {
                "a_row_expecting_another_gpu_judge_is_not_claimed_and_is_named"
            }
            BreakSwitch::ConclusionVersionChecked
            | BreakSwitch::ConclusionWrittenUnderThisBuildKey => {
                "a_row_whose_conclusion_version_this_build_does_not_compute_is_claimed_and_written_under_the_row_key"
            }
            BreakSwitch::ForcedBlockReadsTheLibrary | BreakSwitch::RejudgedRowNotWritten => {
                "a_forced_block_is_judged_without_reading_the_library_and_leaves_a_rejudged_row"
            }
            BreakSwitch::RefusedConclusionsDropped => {
                "conclusions_refused_at_the_write_are_kept_and_reported_red"
            }
            BreakSwitch::CardsOpenedOnlyWithWork | BreakSwitch::CardsLetGoWhenThePoolIdles => {
                "the_cards_are_opened_at_start_and_held_while_the_pool_idles"
            }
            BreakSwitch::ReusableBlockPendingRowKept => {
                "a_reusable_block_is_not_taken_and_its_pending_row_is_deleted_under_the_guard"
            }
            BreakSwitch::PendingRowChangeNotReread => {
                "a_pending_row_raised_to_forced_while_judged_is_reread_and_written_with_its_credential"
            }
            BreakSwitch::GuardRowChangeNotReread => {
                "a_conclusion_changed_under_the_guard_is_compared_again_before_writing"
            }
        }
    }
}

/// 打开了的几项。没有不变量，空的就是平常跑。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BreakSwitches(BTreeSet<BreakSwitch>);

impl BreakSwitches {
    /// 一项都不开：服务主循环交的就是它。
    #[must_use]
    pub fn none() -> Self {
        Self(BTreeSet::new())
    }

    /// 从变量的值读（`None` 是没设）：逗号隔开的项名，前后空白不算，空项不算。
    ///
    /// # Errors
    /// 有一项不是 [`BreakSwitch::name`] 里的名字：交回那一项与认得的全部项名。
    pub fn from_the_variable_value(value: Option<&str>) -> Result<Self, String> {
        let mut switches = BTreeSet::new();
        for written in value.unwrap_or("").split(',').map(str::trim) {
            if written.is_empty() {
                continue;
            }
            let switch = BreakSwitch::EVERY
                .into_iter()
                .find(|switch| switch.name() == written)
                .ok_or_else(|| {
                    let known: Vec<&str> = BreakSwitch::EVERY
                        .iter()
                        .map(|switch| switch.name())
                        .collect();
                    format!(
                        "{BREAK_VARIABLE} 里的「{written}」不是认得的项；认得的：{}",
                        known.join("、")
                    )
                })?;
            switches.insert(switch);
        }
        Ok(Self(switches))
    }

    /// 测试照这个进程的环境读。
    ///
    /// # Panics
    /// 变量里有认不得的项（写错的项不许悄悄当没开）。
    #[must_use]
    pub fn of_the_environment() -> Self {
        let value = std::env::var(BREAK_VARIABLE).ok();
        Self::from_the_variable_value(value.as_deref())
            .unwrap_or_else(|refusal| panic!("{refusal}"))
    }

    #[must_use]
    pub fn is_on(&self, switch: BreakSwitch) -> bool {
        self.0.contains(&switch)
    }
}

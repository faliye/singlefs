//! GPU 判器一次派活里的几趟（[`GpuJudgePass`]）、每趟提交等卡等了多久（一次派活、一趟、整条流三层），与照它们打的汇总行。

/// 判器的挂钟花在哪：卡在算的那一段之外都是 CPU 这一侧的。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GpuJudgeTimeSpent {
    pub dispatches: u64,
    /// 带得最多的一次派活几个状态。
    pub states_in_the_widest_dispatch: u64,
    /// 卡算得最久的一次派活多久。
    pub seconds_of_the_longest_dispatch: f64,
    pub seconds_preparing: f64,
    pub seconds_waiting_for_the_card: f64,
    pub seconds_reading_back: f64,
}

/// GPU 判器一次派活里依次交的各趟（[`GpuCrashJudge`] 一次派活一趟一次提交、等它算完再交下一次），
/// 按派活的次序；记账分片那一趟一次派活交几片就是几次提交，`Readback` 是把结论拷回的那一次提交。
/// 名字（[`Self::name`]）与 `GPU_LIFECYCLE` 行里的 `pass=` 相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuJudgePass {
    Recovery,
    CheckerWalk,
    CheckerScan,
    CheckerScanAllocationRecordsDisjoint,
    CheckerScanAllocationGenerationsThroughTheRollbackFloorRaises,
    CheckerAccountingHead,
    CheckerAccountingSlice,
    CheckerAccountingDecide,
    CheckerAccountingFinal,
    CheckerReferencesAndSnapshots,
    CheckerScanRebuild,
    Readback,
}

/// GPU 判器一次派活有几趟（[`GpuJudgePass`] 的成员数）。
pub const GPU_JUDGE_PASSES: usize = 12;

impl GpuJudgePass {
    pub const IN_THE_ORDER_OF_THE_DISPATCH: [Self; GPU_JUDGE_PASSES] = [
        Self::Recovery,
        Self::CheckerWalk,
        Self::CheckerScan,
        Self::CheckerScanAllocationRecordsDisjoint,
        Self::CheckerScanAllocationGenerationsThroughTheRollbackFloorRaises,
        Self::CheckerAccountingHead,
        Self::CheckerAccountingSlice,
        Self::CheckerAccountingDecide,
        Self::CheckerAccountingFinal,
        Self::CheckerReferencesAndSnapshots,
        Self::CheckerScanRebuild,
        Self::Readback,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Recovery => "recovery",
            Self::CheckerWalk => "checker_walk",
            Self::CheckerScan => "checker_scan",
            Self::CheckerScanAllocationRecordsDisjoint => {
                "checker_scan_allocation_records_disjoint"
            }
            Self::CheckerScanAllocationGenerationsThroughTheRollbackFloorRaises => {
                "checker_scan_allocation_generations_through_the_rollback_floor_raises"
            }
            Self::CheckerAccountingHead => "checker_accounting_head",
            Self::CheckerAccountingSlice => "checker_accounting_slice",
            Self::CheckerAccountingDecide => "checker_accounting_decide",
            Self::CheckerAccountingFinal => "checker_accounting_final",
            Self::CheckerReferencesAndSnapshots => "checker_references_and_snapshots",
            Self::CheckerScanRebuild => "checker_scan_rebuild",
            Self::Readback => "readback",
        }
    }

    /// 在 [`Self::IN_THE_ORDER_OF_THE_DISPATCH`] 里的下标。
    #[must_use]
    pub const fn index_in_the_dispatch(self) -> usize {
        match self {
            Self::Recovery => 0,
            Self::CheckerWalk => 1,
            Self::CheckerScan => 2,
            Self::CheckerScanAllocationRecordsDisjoint => 3,
            Self::CheckerScanAllocationGenerationsThroughTheRollbackFloorRaises => 4,
            Self::CheckerAccountingHead => 5,
            Self::CheckerAccountingSlice => 6,
            Self::CheckerAccountingDecide => 7,
            Self::CheckerAccountingFinal => 8,
            Self::CheckerReferencesAndSnapshots => 9,
            Self::CheckerScanRebuild => 10,
            Self::Readback => 11,
        }
    }
}

/// 一次派活里各趟的提交：每趟几次、等卡纳秒之和、最久那一次几纳秒（[`GpuJudgePass::index_in_the_dispatch`] 为下标）。
/// 定长、不分配：派活里每次提交等完记一次（[`Self::note_a_submission`]）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubmissionWaitsOfOneDispatch {
    submissions_of_each_pass: [u64; GPU_JUDGE_PASSES],
    nanoseconds_waiting_of_each_pass: [u64; GPU_JUDGE_PASSES],
    nanoseconds_of_the_longest_submission_of_each_pass: [u64; GPU_JUDGE_PASSES],
}

impl SubmissionWaitsOfOneDispatch {
    /// `pass` 那一趟又交了一次提交，等卡等了 `nanoseconds`。
    pub fn note_a_submission(&mut self, pass: GpuJudgePass, nanoseconds: u64) {
        let index = pass.index_in_the_dispatch();
        self.submissions_of_each_pass[index] += 1;
        self.nanoseconds_waiting_of_each_pass[index] += nanoseconds;
        self.nanoseconds_of_the_longest_submission_of_each_pass[index] =
            self.nanoseconds_of_the_longest_submission_of_each_pass[index].max(nanoseconds);
    }

    /// 这一次派活全部提交的等卡纳秒之和：判器把它加进自己等卡的总账（[`GpuJudgeTimeSpent::seconds_waiting_for_the_card`] 那一笔）。
    #[must_use]
    pub fn nanoseconds_waiting_for_the_card(&self) -> u64 {
        self.nanoseconds_waiting_of_each_pass.iter().sum()
    }

    /// 这一次派活里最久的那一次提交（不分哪一趟）。
    #[must_use]
    pub fn nanoseconds_of_the_longest_submission(&self) -> u64 {
        self.nanoseconds_of_the_longest_submission_of_each_pass
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
    }
}

/// 一趟在一个判器上到此刻为止的提交账：几次、等卡纳秒之和、最久那一次。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SubmissionWaitsOfAPass {
    pub submissions: u64,
    pub nanoseconds_waiting_for_the_card: u64,
    pub nanoseconds_of_the_longest_submission: u64,
}

/// 一个判器（一张卡）上各趟到此刻为止的提交账，与每次提交带几个状态的分布（求中位数用）。
/// 一次派活的每次提交带的状态数相同（这次派活的状态数），所以分布按派活的状态数记：状态数 → 各趟在这样宽的派活里交了几次提交。
/// 一次派活并一次账（[`Self::add_a_dispatch`]）；分布只在出现没见过的派活宽度时多一个节点。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SubmissionWaitsOfThePasses {
    passes: [SubmissionWaitsOfAPass; GPU_JUDGE_PASSES],
    submissions_of_each_pass_by_states_per_submission:
        std::collections::BTreeMap<u64, [u64; GPU_JUDGE_PASSES]>,
}

impl SubmissionWaitsOfThePasses {
    /// 并进一次带 `states_per_submission` 个状态的派活。
    pub fn add_a_dispatch(
        &mut self,
        states_per_submission: u64,
        waits_of_the_dispatch: &SubmissionWaitsOfOneDispatch,
    ) {
        let submissions_by_pass = self
            .submissions_of_each_pass_by_states_per_submission
            .entry(states_per_submission)
            .or_insert([0; GPU_JUDGE_PASSES]);
        for (index, pass) in self.passes.iter_mut().enumerate() {
            let submissions = waits_of_the_dispatch.submissions_of_each_pass[index];
            pass.submissions += submissions;
            pass.nanoseconds_waiting_for_the_card +=
                waits_of_the_dispatch.nanoseconds_waiting_of_each_pass[index];
            pass.nanoseconds_of_the_longest_submission =
                pass.nanoseconds_of_the_longest_submission.max(
                    waits_of_the_dispatch.nanoseconds_of_the_longest_submission_of_each_pass[index],
                );
            submissions_by_pass[index] += submissions;
        }
    }

    /// 并进另一个判器的账（同一张卡在收尾再起的那一轮里又建了一个判器）。
    pub fn merge(&mut self, other: &Self) {
        for (pass, other_pass) in self.passes.iter_mut().zip(&other.passes) {
            pass.submissions += other_pass.submissions;
            pass.nanoseconds_waiting_for_the_card += other_pass.nanoseconds_waiting_for_the_card;
            pass.nanoseconds_of_the_longest_submission = pass
                .nanoseconds_of_the_longest_submission
                .max(other_pass.nanoseconds_of_the_longest_submission);
        }
        for (states_per_submission, other_submissions) in
            &other.submissions_of_each_pass_by_states_per_submission
        {
            let submissions = self
                .submissions_of_each_pass_by_states_per_submission
                .entry(*states_per_submission)
                .or_insert([0; GPU_JUDGE_PASSES]);
            for (submissions_of_the_pass, other_of_the_pass) in
                submissions.iter_mut().zip(other_submissions)
            {
                *submissions_of_the_pass += other_of_the_pass;
            }
        }
    }

    #[must_use]
    pub fn of_the_pass(&self, pass: GpuJudgePass) -> SubmissionWaitsOfAPass {
        self.passes[pass.index_in_the_dispatch()]
    }

    /// 各趟等卡纳秒之和（与判器等卡的总账是同一笔：两边都由每次派活的 [`SubmissionWaitsOfOneDispatch::nanoseconds_waiting_for_the_card`] 加成）。
    #[must_use]
    pub fn nanoseconds_waiting_for_the_card(&self) -> u64 {
        self.passes
            .iter()
            .map(|pass| pass.nanoseconds_waiting_for_the_card)
            .sum()
    }

    /// `pass` 那一趟每次提交带几个状态的中位数：全部提交按状态数从小到大排，取第 ⌊(次数 − 1) / 2⌋ 个（偶数次取中间两个里小的那个）；一次都没交是 0。
    #[must_use]
    pub fn median_states_per_submission(&self, pass: GpuJudgePass) -> u64 {
        let index = pass.index_in_the_dispatch();
        let submissions = self.passes[index].submissions;
        if submissions == 0 {
            return 0;
        }
        let position_of_the_median = (submissions - 1) / 2;
        let mut submissions_so_far = 0u64;
        self.submissions_of_each_pass_by_states_per_submission
            .iter()
            .find(|(_, submissions_by_pass)| {
                submissions_so_far += submissions_by_pass[index];
                submissions_so_far > position_of_the_median
            })
            .map(|(states_per_submission, _)| *states_per_submission)
            .expect("分布里这一趟的提交次数之和等于这一趟的提交次数（两边在 add_a_dispatch 与 merge 里一起加）")
    }
}

/// 每趟用时汇总行的行首（[`gpu_pass_summaries`]、[`GpuPassSummary::line`]）。
pub const GPU_PASS_LINE_PREFIX: &str = "CRASH_AMPLIFICATION_GPU_PASS";

/// 一条流（或一趟里的一段块区间）判完之后一张卡一趟的汇总；没有卡判过（整段复用，一次提交都没有）时只有一份、`card` 是 `None`。
#[derive(Clone, Debug, PartialEq)]
pub struct GpuPassSummary {
    /// 卡名与序号；`None` 是这条流一张卡都没派活。
    pub card: Option<(String, u32)>,
    /// `None` 与 `card` 同。
    pub pass: Option<GpuJudgePass>,
    pub submissions: u64,
    pub seconds_waiting_for_the_card: f64,
    pub seconds_of_the_longest_submission: f64,
    pub median_states_per_submission: u64,
    /// 这张卡这一趟的等卡秒数占这条流全部卡、全部趟等卡秒数之和的比例；全部之和是 0 时是 0。
    pub share_of_the_waits_of_the_flow: f64,
}

impl GpuPassSummary {
    /// 汇总那一行：`CRASH_AMPLIFICATION_GPU_PASS flow=<流> card=<卡名> index=<序号> pass=<趟名> submissions=… seconds_waiting_for_the_card=…
    /// seconds_of_the_longest_submission=… median_states_per_submission=… share_of_the_waits_of_the_flow=…`；没有卡的那一份卡、序号、趟名写 `none`。
    #[must_use]
    pub fn line(&self, flow_name: &str) -> String {
        let (card, index) = match &self.card {
            Some((name, index)) => (format!("{name:?}"), index.to_string()),
            None => ("none".to_string(), "none".to_string()),
        };
        format!(
            "{GPU_PASS_LINE_PREFIX} flow={flow_name} card={card} index={index} pass={} submissions={} seconds_waiting_for_the_card={:.3} \
             seconds_of_the_longest_submission={:.3} median_states_per_submission={} share_of_the_waits_of_the_flow={:.4}",
            self.pass.map_or("none", GpuJudgePass::name),
            self.submissions,
            self.seconds_waiting_for_the_card,
            self.seconds_of_the_longest_submission,
            self.median_states_per_submission,
            self.share_of_the_waits_of_the_flow
        )
    }
}

/// 一条流判完时每张卡每一趟的汇总，卡按 `cards`（卡名、序号、那张卡的判器的提交账）里头一次出现的先后、趟按派活的次序：
/// 同一张卡（序号相同）收工几次的账并成一份；`cards` 空着（整段复用、一次提交都没有）交一份提交次数 0 的。
/// 比例的分母是这条流全部卡、全部趟的等卡纳秒之和。
#[must_use]
pub fn gpu_pass_summaries<'cards>(
    cards: impl IntoIterator<Item = (&'cards str, u32, &'cards SubmissionWaitsOfThePasses)>,
) -> Vec<GpuPassSummary> {
    let mut merged_by_card: Vec<(&'cards str, u32, SubmissionWaitsOfThePasses)> = Vec::new();
    for (name, card_index, waits_of_the_card) in cards {
        match merged_by_card
            .iter_mut()
            .find(|(_, merged_index, _)| *merged_index == card_index)
        {
            Some((_, _, merged_waits)) => merged_waits.merge(waits_of_the_card),
            None => merged_by_card.push((name, card_index, waits_of_the_card.clone())),
        }
    }
    if merged_by_card.is_empty() {
        return vec![GpuPassSummary {
            card: None,
            pass: None,
            submissions: 0,
            seconds_waiting_for_the_card: 0.0,
            seconds_of_the_longest_submission: 0.0,
            median_states_per_submission: 0,
            share_of_the_waits_of_the_flow: 0.0,
        }];
    }
    let nanoseconds_waiting_of_the_flow: u64 = merged_by_card
        .iter()
        .map(|(_, _, merged_waits)| merged_waits.nanoseconds_waiting_for_the_card())
        .sum();
    let mut summaries = Vec::with_capacity(merged_by_card.len() * GPU_JUDGE_PASSES);
    for (name, card_index, merged_waits) in &merged_by_card {
        for pass in GpuJudgePass::IN_THE_ORDER_OF_THE_DISPATCH {
            let of_the_pass = merged_waits.of_the_pass(pass);
            summaries.push(GpuPassSummary {
                card: Some(((*name).to_string(), *card_index)),
                pass: Some(pass),
                submissions: of_the_pass.submissions,
                seconds_waiting_for_the_card: of_the_pass.nanoseconds_waiting_for_the_card as f64
                    / 1e9,
                seconds_of_the_longest_submission: of_the_pass.nanoseconds_of_the_longest_submission
                    as f64
                    / 1e9,
                median_states_per_submission: merged_waits.median_states_per_submission(pass),
                share_of_the_waits_of_the_flow: if nanoseconds_waiting_of_the_flow == 0 {
                    0.0
                } else {
                    of_the_pass.nanoseconds_waiting_for_the_card as f64
                        / nanoseconds_waiting_of_the_flow as f64
                },
            });
        }
    }
    summaries
}

#[cfg(test)]
mod tests;

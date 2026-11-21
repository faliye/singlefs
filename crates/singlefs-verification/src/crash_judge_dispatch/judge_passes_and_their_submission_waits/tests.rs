/// 秒数写成纳秒（测试里给的都是整的毫秒数）。
fn nanoseconds_of_milliseconds(milliseconds: u64) -> u64 {
    milliseconds * 1_000_000
}

/// 两次派活的提交账：一次带 4 个状态（恢复 3 秒；记账分片三片 0.5、1.5、0.25 秒，最久的那一片在中间；拷回 0.75 秒），
/// 一次带 8 个状态（恢复 1 秒；分片两片 0.25、0.5 秒；拷回 0.25 秒）。别的趟一次都没交。
fn two_dispatches_of_a_judge() -> (
    super::SubmissionWaitsOfOneDispatch,
    super::SubmissionWaitsOfOneDispatch,
    super::SubmissionWaitsOfThePasses,
) {
    use super::GpuJudgePass;
    let mut four_states = super::SubmissionWaitsOfOneDispatch::default();
    four_states.note_a_submission(GpuJudgePass::Recovery, nanoseconds_of_milliseconds(3000));
    for milliseconds in [500, 1500, 250] {
        four_states.note_a_submission(
            GpuJudgePass::CheckerAccountingSlice,
            nanoseconds_of_milliseconds(milliseconds),
        );
    }
    four_states.note_a_submission(GpuJudgePass::Readback, nanoseconds_of_milliseconds(750));
    let mut eight_states = super::SubmissionWaitsOfOneDispatch::default();
    eight_states.note_a_submission(GpuJudgePass::Recovery, nanoseconds_of_milliseconds(1000));
    for milliseconds in [250, 500] {
        eight_states.note_a_submission(
            GpuJudgePass::CheckerAccountingSlice,
            nanoseconds_of_milliseconds(milliseconds),
        );
    }
    eight_states.note_a_submission(GpuJudgePass::Readback, nanoseconds_of_milliseconds(250));
    let mut passes = super::SubmissionWaitsOfThePasses::default();
    passes.add_a_dispatch(4, &four_states);
    passes.add_a_dispatch(8, &eight_states);
    (four_states, eight_states, passes)
}

/// 各趟的提交账：次数与等卡之和是加起来的，最久的是最久的那一次（不是最后一次），中位数是全部提交按态数排开取下中位；
/// 各趟等卡之和等于每次派活交给判器等卡总账的那一笔之和（`GpuJudgeTimeSpent::seconds_waiting_for_the_card` 加的就是它），值钉成绝对数。
#[test]
fn submission_waits_of_the_passes_add_up_keep_the_longest_submission_and_take_the_lower_median() {
    use super::GpuJudgePass;
    let (four_states, eight_states, passes) = two_dispatches_of_a_judge();
    assert_eq!(
        four_states.nanoseconds_waiting_for_the_card(),
        nanoseconds_of_milliseconds(3000 + 500 + 1500 + 250 + 750),
        "带 4 个状态那一次派活等卡之和"
    );
    assert_eq!(
        four_states.nanoseconds_of_the_longest_submission(),
        nanoseconds_of_milliseconds(3000),
        "带 4 个状态那一次派活最久的那一次提交是恢复那一趟"
    );
    assert_eq!(
        eight_states.nanoseconds_of_the_longest_submission(),
        nanoseconds_of_milliseconds(1000),
        "带 8 个状态那一次派活最久的那一次提交是恢复那一趟"
    );
    assert_eq!(
        passes.of_the_pass(GpuJudgePass::Recovery),
        super::SubmissionWaitsOfAPass {
            submissions: 2,
            nanoseconds_waiting_for_the_card: nanoseconds_of_milliseconds(4000),
            nanoseconds_of_the_longest_submission: nanoseconds_of_milliseconds(3000),
        },
        "恢复那一趟：两次、共 4 秒、最久 3 秒（先派的那一次）"
    );
    assert_eq!(
        passes.of_the_pass(GpuJudgePass::CheckerAccountingSlice),
        super::SubmissionWaitsOfAPass {
            submissions: 5,
            nanoseconds_waiting_for_the_card: nanoseconds_of_milliseconds(3000),
            nanoseconds_of_the_longest_submission: nanoseconds_of_milliseconds(1500),
        },
        "记账分片那一趟：五片、共 3 秒、最久 1.5 秒（头一次派活的中间那一片）"
    );
    assert_eq!(
        passes.of_the_pass(GpuJudgePass::Readback),
        super::SubmissionWaitsOfAPass {
            submissions: 2,
            nanoseconds_waiting_for_the_card: nanoseconds_of_milliseconds(1000),
            nanoseconds_of_the_longest_submission: nanoseconds_of_milliseconds(750),
        },
        "拷回那一趟：两次、共 1 秒、最久 0.75 秒"
    );
    assert_eq!(
        passes.of_the_pass(GpuJudgePass::CheckerWalk),
        super::SubmissionWaitsOfAPass::default(),
        "一次都没交的趟是零"
    );
    assert_eq!(
        passes.median_states_per_submission(GpuJudgePass::CheckerAccountingSlice),
        4,
        "分片那一趟的五次提交按态数排开是 4、4、4、8、8，中位是 4"
    );
    assert_eq!(
        passes.median_states_per_submission(GpuJudgePass::Recovery),
        4,
        "恢复那一趟两次是 4、8，偶数次取中间两个里小的那个"
    );
    assert_eq!(
        passes.median_states_per_submission(GpuJudgePass::CheckerWalk),
        0,
        "一次都没交的趟中位数是 0"
    );
    assert_eq!(
        passes.nanoseconds_waiting_for_the_card(),
        four_states.nanoseconds_waiting_for_the_card()
            + eight_states.nanoseconds_waiting_for_the_card(),
        "各趟等卡之和等于两次派活交给判器等卡总账的两笔之和"
    );
    assert_eq!(
        passes.nanoseconds_waiting_for_the_card(),
        nanoseconds_of_milliseconds(8000),
        "各趟等卡之和是 8 秒"
    );
}

/// 汇总行：同一张卡收工两次的账并成一份，卡按头一次出现的先后、趟按派活的次序各一行；比例的分母是全部卡全部趟，
/// 各行比例之和等于 1；那一行原样钉住。
#[test]
fn pass_summaries_merge_a_card_that_finished_twice_and_share_the_waits_of_the_whole_flow() {
    use super::{gpu_pass_summaries, GpuJudgePass, GPU_JUDGE_PASSES};
    let (_, _, passes_of_the_card_at_index_one) = two_dispatches_of_a_judge();
    let mut recovery_only = super::SubmissionWaitsOfOneDispatch::default();
    recovery_only.note_a_submission(GpuJudgePass::Recovery, nanoseconds_of_milliseconds(4000));
    let mut first_round_of_the_card_at_index_zero = super::SubmissionWaitsOfThePasses::default();
    first_round_of_the_card_at_index_zero.add_a_dispatch(16, &recovery_only);
    let mut second_round_of_the_card_at_index_zero = super::SubmissionWaitsOfThePasses::default();
    second_round_of_the_card_at_index_zero.add_a_dispatch(2, &recovery_only);
    let summaries = gpu_pass_summaries([
        (
            "card at index zero",
            0,
            &first_round_of_the_card_at_index_zero,
        ),
        ("card at index one", 1, &passes_of_the_card_at_index_one),
        (
            "card at index zero",
            0,
            &second_round_of_the_card_at_index_zero,
        ),
    ]);
    assert_eq!(
        summaries.len(),
        2 * GPU_JUDGE_PASSES,
        "两张卡各一份、每份每趟一行"
    );
    let recovery_of_the_card_at_index_zero = &summaries[0];
    assert_eq!(
        recovery_of_the_card_at_index_zero.card,
        Some(("card at index zero".to_string(), 0))
    );
    assert_eq!(
        recovery_of_the_card_at_index_zero.pass,
        Some(GpuJudgePass::Recovery)
    );
    assert_eq!(
        recovery_of_the_card_at_index_zero.submissions, 2,
        "收工两次的账并成一份"
    );
    assert_eq!(
        recovery_of_the_card_at_index_zero.seconds_waiting_for_the_card,
        8.0
    );
    assert_eq!(
        recovery_of_the_card_at_index_zero.seconds_of_the_longest_submission,
        4.0
    );
    assert_eq!(
        recovery_of_the_card_at_index_zero.median_states_per_submission, 2,
        "两轮的分布并在一起：2、16 取下中位"
    );
    assert_eq!(
        recovery_of_the_card_at_index_zero.share_of_the_waits_of_the_flow, 0.5,
        "这条流等卡共 16 秒（这张卡 8 秒、另一张 8 秒），这一行 8 秒"
    );
    assert_eq!(
        summaries[GPU_JUDGE_PASSES].card,
        Some(("card at index one".to_string(), 1)),
        "另一张卡在头一张的十二行之后"
    );
    let slice_of_the_card_at_index_one = summaries
        .iter()
        .find(|summary| {
            summary.card == Some(("card at index one".to_string(), 1))
                && summary.pass == Some(GpuJudgePass::CheckerAccountingSlice)
        })
        .expect("另一张卡有记账分片那一行");
    assert_eq!(
        slice_of_the_card_at_index_one.share_of_the_waits_of_the_flow, 0.1875,
        "3 秒占 16 秒"
    );
    assert_eq!(
        slice_of_the_card_at_index_one.line("new-pool-file-creation"),
        "CRASH_AMPLIFICATION_GPU_PASS flow=new-pool-file-creation card=\"card at index one\" index=1 pass=checker_accounting_slice submissions=5 \
             seconds_waiting_for_the_card=3.000 seconds_of_the_longest_submission=1.500 median_states_per_submission=4 share_of_the_waits_of_the_flow=0.1875"
    );
    assert_eq!(
        summaries
            .iter()
            .map(|summary| summary.share_of_the_waits_of_the_flow)
            .sum::<f64>(),
        1.0,
        "各行比例之和是 1"
    );
}

/// 一张卡都没派活（整段复用）：一行提交次数 0 的，卡、序号、趟名写 none，不省略。
#[test]
fn pass_summaries_of_a_flow_without_a_dispatch_are_one_line_with_no_submission() {
    let summaries = super::gpu_pass_summaries(std::iter::empty());
    assert_eq!(summaries.len(), 1, "整段复用的流也有一行");
    assert_eq!(
        summaries[0].line("flow-reused-in-full"),
        "CRASH_AMPLIFICATION_GPU_PASS flow=flow-reused-in-full card=none index=none pass=none submissions=0 seconds_waiting_for_the_card=0.000 \
             seconds_of_the_longest_submission=0.000 median_states_per_submission=0 share_of_the_waits_of_the_flow=0.0000"
    );
}

/// 趟的次序表与下标对得上（账按下标记，次序表错位就把一趟的数记到另一趟名下），名字两两不同。
#[test]
fn every_pass_in_the_order_of_the_dispatch_sits_at_its_own_index_under_its_own_name() {
    let passes = super::GpuJudgePass::IN_THE_ORDER_OF_THE_DISPATCH;
    for (position, pass) in passes.iter().enumerate() {
        assert_eq!(pass.index_in_the_dispatch(), position, "{pass:?}");
    }
    let names: std::collections::BTreeSet<&str> = passes.iter().map(|pass| pass.name()).collect();
    assert_eq!(names.len(), passes.len(), "趟名两两不同：{names:?}");
}

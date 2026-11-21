use super::*;
use crate::crash_judge_gpu::GPU_JUDGE_CHECKER_COMMON_SOURCE;

/// NVIDIA 卡上 wgpu 给的存储绑定上限。
const BINDING_LIMIT: u64 = 2_147_483_644;

/// 一个字几个字节（草稿区、结论都按 u32 字排）。
const BYTES_PER_WORD: u64 = 4;

/// 一 MiB 几个字节。
const BYTES_PER_MEBIBYTE: u64 = 1024 * 1024;

/// 内核里每个状态草稿区的末端（`crash_judge_checker_common.wgsl` 里 `const SCRATCH_END: u32 = …u;` 那一行，按字计）：
/// 派活这一侧的钉值都从它算，不从派活自己用的 [`super::SCRATCH_WORDS`] 算——派活给每个状态的字数与内核的草稿区对不上时，
/// 下面几条钉值跟着红；内核再加表、草稿区再抬，这几个数不用手改。
fn scratch_words_of_the_kernels() -> u64 {
    let declaration_prefix = "const SCRATCH_END: u32 = ";
    GPU_JUDGE_CHECKER_COMMON_SOURCE
        .lines()
        .find_map(|line| line.strip_prefix(declaration_prefix))
        .expect("池级 checker 共用那一半里有 SCRATCH_END 那一行")
        .strip_suffix("u;")
        .expect("那一行以 `u;` 收尾")
        .parse()
        .expect("草稿区末端是一个十进制字数")
}

/// 一条草稿缓冲装几个状态由绑定上限定；一次派活带几个由给草稿区的显存定，到 [`MAXIMUM_SCRATCH_LANES`] 条为止，至少 1 个。
/// 此刻每个状态 2 009 599 × 4 = 8 038 396 字节：一条缓冲 2 147 483 644 ÷ 8 038 396 = 267 个、12 GiB 装 1602 个（都向下取整）。
#[test]
fn states_in_one_dispatch_follow_the_scratch_memory_up_to_the_lane_limit() {
    let bytes_per_state = scratch_words_of_the_kernels() * BYTES_PER_WORD;
    let states_per_lane = usize::try_from(BINDING_LIMIT / bytes_per_state).expect("装得进 usize");
    assert_eq!(
        states_per_scratch_lane(BINDING_LIMIT),
        states_per_lane,
        "一条缓冲装绑定上限 ÷ 每个状态 {bytes_per_state} 字节个状态（向下取整）"
    );
    assert_eq!(states_per_scratch_lane(1), 1, "再小的上限也带一个");
    let twelve_gibibytes_in_mebibytes = 12 * 1024;
    let states_in_twelve_gibibytes =
        usize::try_from(twelve_gibibytes_in_mebibytes * BYTES_PER_MEBIBYTE / bytes_per_state)
            .expect("装得进 usize");
    assert!(
        states_in_twelve_gibibytes < states_per_lane * MAXIMUM_SCRATCH_LANES,
        "12 GiB 装的状态少于条数上限：这一格判的是显存那一边"
    );
    assert_eq!(
        states_per_dispatch_within_the_scratch_memory(BINDING_LIMIT, twelve_gibibytes_in_mebibytes),
        states_in_twelve_gibibytes,
        "12 GiB 的草稿区装 12 GiB ÷ 每个状态 {bytes_per_state} 字节个状态（向下取整）"
    );
    assert_eq!(
        states_per_dispatch_within_the_scratch_memory(BINDING_LIMIT, 0),
        1,
        "一点显存都不给也带一个"
    );
    assert_eq!(
        states_per_dispatch_within_the_scratch_memory(BINDING_LIMIT, 1024 * 1024),
        states_per_lane * MAXIMUM_SCRATCH_LANES,
        "显存再多也只到条数上限"
    );
}

/// 卡池按 [`super::mebibytes_a_judge_needs`] 借给用例的卡（额度是借的量加驱动那一份）上，判器开的草稿区不少于借的时候算进去的状态数：
/// 小流（45、489 个状态）照状态数、大流（131 195 个）照要的宽度（一条缓冲那么窄、三条缓冲加窄的那一档那么宽）。
/// 表取 1 MiB：45 个状态要 1 + ⌈45 × (每个状态的草稿区 + 两份结论字 2 × 25 × 4) ÷ 1 048 576⌉ MiB；
/// 此刻 1 + ⌈45 × 8 038 596 ÷ 1 048 576⌉ = 1 + ⌈361 736 820 ÷ 1 048 576⌉ = 1 + 345 = 346 MiB。
#[test]
fn card_lent_what_a_judge_needs_opens_scratch_for_every_state_it_was_lent_for() {
    use super::{mebibytes_a_judge_needs, states_in_flight_within, PackedJudgeTables};
    let tables_mebibytes = 1;
    let tables_of = |state_count: u64| PackedJudgeTables {
        words: vec![
            0;
            usize::try_from(tables_mebibytes * BYTES_PER_MEBIBYTE / BYTES_PER_WORD)
                .expect("装得进 usize")
        ],
        state_count,
    };
    let verdict_words = u64::try_from(VERDICT_WORDS).expect("装得进 u64");
    let bytes_per_state_with_its_verdicts =
        (scratch_words_of_the_kernels() + 2 * verdict_words) * BYTES_PER_WORD;
    let small_flow_states = 45;
    assert_eq!(
        mebibytes_a_judge_needs(&tables_of(small_flow_states), usize::MAX),
        tables_mebibytes
            + (small_flow_states * bytes_per_state_with_its_verdicts)
                .div_ceil(BYTES_PER_MEBIBYTE),
        "表 {tables_mebibytes} MiB 加 {small_flow_states} 个状态各 {bytes_per_state_with_its_verdicts} 字节（向上取整到 MiB）"
    );
    for (state_count, requested) in [
        (45, usize::MAX),
        (489, usize::MAX),
        (131_195, 1278),
        (131_195, 3 * 1278),
        (131_195, 256),
    ] {
        let tables = tables_of(state_count);
        let lent = mebibytes_a_judge_needs(&tables, requested);
        let states_lent_for = requested.min(usize::try_from(state_count).expect("装得进 usize"));
        assert!(
            states_in_flight_within(
                BINDING_LIMIT,
                lent + super::MEBIBYTES_LEFT_FOR_THE_DRIVER,
                &tables
            ) >= states_lent_for,
            "{state_count} 个状态、要 {requested} 个一次：借了 {lent} MiB，判器开出来的草稿区装不下借的时候算进去的 {states_lent_for} 个"
        );
    }
}

/// 量宽度照两张卡上量到的数（固定脚本到回退、展开 16、流的中间，一档加一条 2170 个状态的缓冲）：
/// 5090 到 23 870 个时每个状态的用时翻倍，停，取 21 700；5060 Ti 到 8 680 个时一次 0.324 秒，停，取 6 510。
#[test]
fn the_dispatch_width_is_the_quickest_per_state_before_the_card_slows_down() {
    use super::{calibrated_width, calibration_goes_on};
    let rtx_5090 = [
        (2170, 0.028),
        (4340, 0.028),
        (6510, 0.028),
        (8680, 0.029),
        (10_850, 0.029),
        (13_020, 0.031),
        (15_190, 0.031),
        (17_360, 0.032),
        (19_530, 0.033),
        (21_700, 0.034),
        (23_870, 0.071),
    ];
    for measured in 1..rtx_5090.len() {
        assert!(
            calibration_goes_on(
                &rtx_5090[..measured],
                rtx_5090[measured - 1].1,
                31_291,
                super::LONGEST_DISPATCH_SECONDS
            ),
            "5090 前 {measured} 档每个状态越来越快：接着加宽"
        );
    }
    assert!(
        !calibration_goes_on(&rtx_5090, 0.071, 31_291, super::LONGEST_DISPATCH_SECONDS),
        "5090 到 23870 个每个状态慢了一倍：停"
    );
    assert_eq!(calibrated_width(&rtx_5090), 21_700);
    let rtx_5060_ti = [(2170, 0.029), (4340, 0.032), (6510, 0.039), (8680, 0.324)];
    assert!(calibration_goes_on(
        &rtx_5060_ti[..3],
        0.039,
        14_072,
        super::LONGEST_DISPATCH_SECONDS
    ));
    assert!(!calibration_goes_on(
        &rtx_5060_ti,
        0.324,
        14_072,
        super::LONGEST_DISPATCH_SECONDS
    ));
    assert_eq!(calibrated_width(&rtx_5060_ti), 6510);
    assert!(
        !calibration_goes_on(
            &[(2170, 0.2), (4340, 0.26)],
            0.26,
            31_291,
            super::LONGEST_DISPATCH_SECONDS
        ),
        "每个状态还在变快，可最久那一次提交超过 0.25 秒：停"
    );
    assert!(
        calibration_goes_on(&[(267, 0.300)], 0.091, 1727, super::LONGEST_DISPATCH_SECONDS),
        "整次派活 0.300 秒是几趟、几片提交加起来的，最久那一次提交只有 0.091 秒：接着加宽（5080 上截断流量到的那一档）"
    );
    assert!(
        calibration_goes_on(
            &[(267, 0.300), (534, 0.310)],
            0.095,
            1727,
            super::LONGEST_DISPATCH_SECONDS
        ),
        "整次派活过了 0.25 秒、每个状态在变快、最久那一次提交没过 0.25 秒：接着加宽"
    );
    assert!(
        !calibration_goes_on(&[(2170, 0.02)], 0.02, 2170, super::LONGEST_DISPATCH_SECONDS),
        "到了这张卡带得了的上限：停"
    );
}

/// 量完一次，最宽那一档不是选中的（过了拐点）就记下它，以后不再试到它；最宽那一档就是选中的（没撞到拐点）不记。
#[test]
fn width_past_the_knee_is_not_tried_again() {
    use super::knee_after_calibration;
    let past_the_knee = [(2170, 0.076), (4340, 0.110), (6510, 0.180), (8680, 3.6)];
    assert_eq!(knee_after_calibration(&past_the_knee, usize::MAX), 8680);
    assert_eq!(
        knee_after_calibration(&past_the_knee, 6000),
        6000,
        "已经记下更窄的拐点：不放宽"
    );
    let up_to_the_capacity = [(2170, 0.03), (4340, 0.035)];
    assert_eq!(
        knee_after_calibration(&up_to_the_capacity, usize::MAX),
        usize::MAX
    );
}

/// 一次派活算得太久：下一次减半，减到 1 个状态为止（宽度到 1 再慢也不到 0）。
#[test]
fn dispatch_that_took_too_long_halves_the_next_one() {
    use super::width_after_a_slow_dispatch;
    assert_eq!(width_after_a_slow_dispatch(21_700, 31_291), 10_850);
    assert_eq!(
        width_after_a_slow_dispatch(300, 31_291),
        150,
        "几百个状态还接着减半"
    );
    assert_eq!(width_after_a_slow_dispatch(100, 100), 50);
    assert_eq!(width_after_a_slow_dispatch(3, 31_291), 1);
    assert_eq!(
        width_after_a_slow_dispatch(1, 31_291),
        1,
        "1 个状态：不再减半"
    );
}

/// 没量过拆不开的下限，用时不到 [`super::LONGEST_DISPATCH_SECONDS`] 的四分之一（0.125 秒）又带满了宽度：下一次加倍；
/// 0.125 秒到 0.5 秒之间不动，0.5 秒本身也不动。
#[test]
fn quick_dispatch_doubles_the_next_and_one_near_the_limit_keeps_it() {
    use super::dispatch_width_after_a_dispatch;
    for (seconds, width_after, what) in [
        (0.001, 128, "0.001 秒：加倍"),
        (0.124, 128, "0.124 秒：加倍"),
        (0.126, 64, "0.126 秒：不动"),
        (0.3, 64, "0.3 秒：不动"),
        (0.5, 64, "0.5 秒：还没超过上限，不动"),
    ] {
        assert_eq!(
            dispatch_width_after_a_dispatch(64, 64, seconds, 2170, 31_291, false),
            width_after,
            "宽度 64、{what}"
        );
    }
}

/// 没量过拆不开的下限，用时超过 [`super::LONGEST_DISPATCH_SECONDS`]：照实际带的状态数减半，不许到驱动会掐的那条线。
/// 这一段只剩 3 个状态时宽度 1000 还算了 0.6 秒，下一次带 1 个，不是 500 个。
#[test]
fn slow_dispatch_halves_what_it_carried_down_to_one_state() {
    use super::dispatch_width_after_a_dispatch;
    assert_eq!(
        dispatch_width_after_a_dispatch(64, 64, 0.51, 2170, 31_291, false),
        32,
        "0.51 秒：过了 0.5 秒，减半"
    );
    assert_eq!(
        dispatch_width_after_a_dispatch(64, 64, 0.6, 2170, 31_291, false),
        32
    );
    assert_eq!(
        dispatch_width_after_a_dispatch(1000, 3, 0.6, 2170, 31_291, false),
        1,
        "只带了 3 个状态就超了"
    );
    assert_eq!(
        dispatch_width_after_a_dispatch(1, 1, 5.9, 2170, 31_291, false),
        1,
        "1 个状态还超：下限 1"
    );
}

/// 加宽不超过一条草稿缓冲、不超过容量；到了一条草稿缓冲往上归量宽度管，用时再短也不动。
#[test]
fn widening_stops_at_one_scratch_lane_and_at_the_capacity() {
    use super::dispatch_width_after_a_dispatch;
    assert_eq!(
        dispatch_width_after_a_dispatch(2048, 2048, 0.01, 2170, 31_291, false),
        2170,
        "加倍会过一条草稿缓冲：停在 2170"
    );
    assert_eq!(
        dispatch_width_after_a_dispatch(8, 8, 0.01, 2170, 10, false),
        10,
        "加倍会过容量：停在 10"
    );
    assert_eq!(
        dispatch_width_after_a_dispatch(2170, 2170, 0.01, 2170, 31_291, false),
        2170,
        "已经一条草稿缓冲"
    );
    assert_eq!(
        dispatch_width_after_a_dispatch(21_700, 21_700, 0.01, 2170, 31_291, false),
        21_700,
        "量出来的宽度不在这里涨"
    );
}

/// 这一段剩的状态比宽度少：算得再快也说不出带满了多久，不加宽。
#[test]
fn dispatch_that_did_not_fill_its_width_does_not_widen_it() {
    use super::dispatch_width_after_a_dispatch;
    assert_eq!(
        dispatch_width_after_a_dispatch(64, 3, 0.001, 2170, 31_291, false),
        64
    );
}

/// 宽度 1 那一次 0.565 秒（崩溃放量那条重的流在 5090 上量到的）是拆不开的下限：下一次加宽，不因为过了 0.5 秒停在 1；
/// 量过下限之后最久那一次提交不过驱动那条线的一半（1 秒）接着加宽、到一条草稿缓冲为止，在 1 秒到 2 秒之间不动，过了 2 秒才减半。
#[test]
fn a_dispatch_of_one_state_over_the_target_is_the_indivisible_floor_and_the_width_grows_from_it() {
    use super::dispatch_width_after_a_dispatch;
    for (width, seconds, width_after, what) in [
        (1, 0.565, 2, "宽度 1、恰好是下限：加宽"),
        (2, 0.6, 4, "宽度 2、0.6 秒：加宽"),
        (
            1024,
            0.84,
            1923,
            "宽度 1024、0.84 秒：加宽，不过一条草稿缓冲",
        ),
        (64, 0.9, 128, "宽度 64、0.9 秒，没过那条线的一半：加宽"),
        (64, 1.69, 64, "宽度 64、1.69 秒，过了一半、没过那条线：不动"),
        (64, 1.7, 64, "宽度 64、1.7 秒：不动，不拿下限的倍数比"),
    ] {
        assert_eq!(
            dispatch_width_after_a_dispatch(width, width, seconds, 1923, 3846, true),
            width_after,
            "下限 0.565 秒、{what}"
        );
    }
}

/// 量过拆不开的下限：下限多短都一样，最久那一次提交不过驱动会掐的那条线的一半（1 秒）就加宽，过了那条线（2 秒）才减半。
/// 下限 0.27 秒、带满 128 个状态最久一次 0.42 秒（本机 32G 卡上 unlink-merging-inode-leaf-containers 头 300 块量到的那一档）照样加宽。
#[test]
fn a_measured_floor_widens_up_to_half_the_driver_line_and_halves_only_past_the_line() {
    use super::dispatch_width_after_a_dispatch;
    for (seconds, width_after, what) in [
        (0.42, 256, "0.42 秒：加宽"),
        (0.99, 256, "0.99 秒：没过那条线的一半，加宽"),
        (1.01, 128, "1.01 秒：过了那条线的一半，不动"),
        (1.99, 128, "1.99 秒：没过那条线，不减半"),
        (2.01, 64, "2.01 秒：过了那条线，减半"),
    ] {
        assert_eq!(
            dispatch_width_after_a_dispatch(128, 128, seconds, 2170, 31_291, true),
            width_after,
            "量过下限、宽度 128 带满、{what}"
        );
    }
    assert_eq!(
        dispatch_width_after_a_dispatch(128, 100, 0.42, 2170, 31_291, true),
        128,
        "没带满宽度：不加宽"
    );
}

/// 下限过了驱动会掐的那条线的一半（1.5 秒）或那条线本身（2.5 秒）：宽度 1 也不加宽（再加一倍，状态互相拖慢时就过线），停在下限上。
#[test]
fn a_floor_over_half_the_driver_line_keeps_the_width_at_one_state() {
    use super::dispatch_width_after_a_dispatch;
    assert_eq!(
        dispatch_width_after_a_dispatch(1, 1, 1.5, 1923, 3846, true),
        1
    );
    assert_eq!(
        dispatch_width_after_a_dispatch(1, 1, 2.5, 1923, 3846, true),
        1
    );
    assert_eq!(
        dispatch_width_after_a_dispatch(1, 1, 0.9, 1923, 3846, true),
        2,
        "0.9 秒不过那条线的一半：加宽"
    );
}

/// 许的上限：量过拆不开的下限就是驱动会掐的那条线 2 秒，不看下限多长；没量过是 0.5 秒。
#[test]
fn the_tolerated_submission_is_the_driver_line_once_the_floor_is_measured_and_half_a_second_before()
{
    use super::longest_tolerated_submission_seconds;
    assert_eq!(longest_tolerated_submission_seconds(true), 2.0);
    assert_eq!(longest_tolerated_submission_seconds(false), 0.5);
}

/// 拆不开的下限怎么记：带 1 个状态的那一次就是下限（变重变轻都跟着）；带得更多的那一次比下限快就把下限压到它，慢不动；没量过的还是没量过。
#[test]
fn the_indivisible_floor_is_the_dispatch_of_one_state_and_falls_with_a_quicker_wider_one() {
    use super::indivisible_floor_after_a_dispatch;
    assert_eq!(
        indivisible_floor_after_a_dispatch(None, 1, 0.565),
        Some(0.565)
    );
    assert_eq!(
        indivisible_floor_after_a_dispatch(Some(0.2), 1, 0.9),
        Some(0.9)
    );
    assert_eq!(
        indivisible_floor_after_a_dispatch(Some(0.565), 4, 0.9),
        Some(0.565)
    );
    assert_eq!(
        indivisible_floor_after_a_dispatch(Some(0.565), 4, 0.3),
        Some(0.3)
    );
    assert_eq!(indivisible_floor_after_a_dispatch(None, 4, 0.3), None);
}

/// 停在下限上的数法：只带 1 个状态、之后还是 1 个、又过了 0.5 秒，才在前面的次数上加一；三样缺一样从 0 重数。
#[test]
fn a_dispatch_counts_as_stuck_only_at_one_state_staying_at_one_state_over_the_target() {
    use super::dispatches_stuck_at_the_floor_after_a_dispatch;
    assert_eq!(
        dispatches_stuck_at_the_floor_after_a_dispatch(3, 1, 1.5, 1),
        4
    );
    assert_eq!(
        dispatches_stuck_at_the_floor_after_a_dispatch(3, 1, 1.5, 2),
        0,
        "加了宽"
    );
    assert_eq!(
        dispatches_stuck_at_the_floor_after_a_dispatch(3, 1, 0.4, 1),
        0,
        "没过 0.5 秒"
    );
    assert_eq!(
        dispatches_stuck_at_the_floor_after_a_dispatch(3, 2, 1.5, 1),
        0,
        "带了 2 个状态"
    );
}

/// 量宽度照许的上限的一半停：量过下限时许 2 秒，一条缓冲那一档最久一次提交 0.99 秒接着加宽、1.01 秒停；照 0.5 秒的一半，0.6 秒就停了。
#[test]
fn calibration_over_an_indivisible_floor_goes_on_under_half_the_tolerated_submission() {
    use super::{calibration_goes_on, longest_tolerated_submission_seconds};
    assert!(calibration_goes_on(
        &[(1923, 0.6)],
        0.6,
        3846,
        longest_tolerated_submission_seconds(true)
    ));
    assert!(calibration_goes_on(
        &[(1923, 0.99)],
        0.99,
        3846,
        longest_tolerated_submission_seconds(true)
    ));
    assert!(!calibration_goes_on(
        &[(1923, 1.01)],
        1.01,
        3846,
        longest_tolerated_submission_seconds(true)
    ));
}

/// 记账分片一片算得多于 0.125 秒（[`super::LONGEST_DISPATCH_SECONDS`] 的四分之一）下一片减半、不窄过 16 个位置；少于 0.03125 秒（十六分之一）
/// 加倍、不宽过一轮的位置数 65 536；中间不动。0.2 秒与 0.05 秒两档照每片 0.25 秒那一档调时一个不动、一个加倍，钉的是门槛低了一倍。
#[test]
fn an_accounting_slice_slower_than_an_eighth_of_a_second_halves_the_next_and_one_quicker_than_a_thirty_second_doubles_it(
) {
    use super::accounting_slice_width_after_a_slice;
    for (seconds, width_after, what) in [
        (0.3, 512, "0.3 秒：减半"),
        (0.2, 512, "0.2 秒：减半（还没到每片的上限 0.25 秒也减）"),
        (0.13, 512, "0.13 秒：减半"),
        (0.12, 1024, "0.12 秒：不动"),
        (0.05, 1024, "0.05 秒：不动（不加倍）"),
        (0.035, 1024, "0.035 秒：不动"),
        (0.03, 2048, "0.03 秒：加倍"),
    ] {
        assert_eq!(
            accounting_slice_width_after_a_slice(1024, seconds),
            width_after,
            "片宽 1024、{what}"
        );
    }
    assert_eq!(
        accounting_slice_width_after_a_slice(16, 0.3),
        16,
        "最窄 16 个位置：不再减半"
    );
    assert_eq!(
        accounting_slice_width_after_a_slice(65_536, 0.01),
        65_536,
        "整轮一片：不再加倍"
    );
}

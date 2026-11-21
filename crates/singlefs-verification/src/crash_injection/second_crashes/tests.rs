use super::*;
use crate::mount_transparency::WitnessedCounterBranchThatHolds;

/// 二次崩溃的种子混进第一次崩溃那个段内子集（攻方改法 F3，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第五节第 1 行）：
/// 同一历史种子（7）、同一段号（5）、同一份挂载写表，段内子集不同（第 0 个写落了 / 第 1 个写落了，各三个写）的两个第一次崩溃态，
/// 摆出的二次崩溃不同。挂载写表造成两段：写行 4 个写、暖机 8 个写。怎么挑的：在 harness 草稿探针里照
/// [`draw_second_crash_points`] 原样抄一份算这组输入，混子集时两份摆法在写行与暖机两段的持久集合都不同，不混时逐个相同。
/// 判别力：种子不混子集（`crates/mutations.tsv` 一行，指纹换成空子集的）时两份摆法逐个相同，这一条红。
#[test]
fn two_first_crash_states_differing_only_in_the_persisted_subset_draw_different_second_crashes() {
    let phase_of_each_write: Vec<WritableMountPhase> = (0..12)
        .map(|write_index| {
            if write_index < 4 {
                WritableMountPhase::RowPublish
            } else {
                WritableMountPhase::WarmUp
            }
        })
        .collect();
    let segments: Vec<Vec<usize>> = vec![(0..4).collect(), (4..12).collect()];
    let first_crash_with = |persisted_within_the_segment: Vec<bool>| CrashPoint {
        segment_index: 5,
        persisted_within_the_segment,
        step: StepPosition::Operation(0),
        operation_kind: Some(HistoryOperationKind::PublishOverwrite),
    };
    let first_write_landed = first_crash_with(vec![true, false, false]);
    let second_write_landed = first_crash_with(vec![false, true, false]);
    let drawn_after_the_first_write_landed = draw_second_crash_points(
        HistorySeed(7),
        &first_write_landed,
        &segments,
        &phase_of_each_write,
    );
    let drawn_after_the_second_write_landed = draw_second_crash_points(
        HistorySeed(7),
        &second_write_landed,
        &segments,
        &phase_of_each_write,
    );
    assert_eq!(
        drawn_after_the_first_write_landed,
        draw_second_crash_points(
            HistorySeed(7),
            &first_write_landed,
            &segments,
            &phase_of_each_write
        ),
        "前提：同一个第一次崩溃态摆两次，摆法相同"
    );
    assert_ne!(
        drawn_after_the_first_write_landed, drawn_after_the_second_write_landed,
        "同一历史种子、同一段号、段内子集不同的两个第一次崩溃态要摆出不同的二次崩溃"
    );
}

/// 二次崩溃的种子把段内子集整个折进去（攻方改法 F3-c，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节
/// 「K2 F3 内联用例判别力」与第三节第 2 条）：同一历史种子（0）、同一段号（16）、长 12 的段，段内扣下集合非空的全部 4095 个第一次崩溃态
/// （12 个写的 4096 个持久子集去掉全持久的那一个：全持久不是崩在这一段里的态）各算一次种子，两两不同。上一条只比两个子集，
/// 两个子集的第一个写一个落了一个没落，只折第一个写的弱指纹也分得开。攻方在副本上量过：入库指纹 4095 个种子各不相同，
/// 只折第一个写落没落时剩 2 个，只取第一个被扣下的写的下标时剩 12 个（`research/prompts/m3-formal-code-b21-r1-opus-model/out/f3.txt`）。
/// 判别力：指纹只折段内第一个写落没落（`crates/mutations.tsv` 一行）时种子只剩 2 个，这一条红。
#[test]
fn all_first_crash_states_of_a_segment_of_twelve_writes_seed_their_second_crashes_differently() {
    const WRITES_OF_THE_SEGMENT: usize = 12;
    let all_persisted_mask = (1_u32 << WRITES_OF_THE_SEGMENT) - 1;
    let seeds: BTreeSet<SecondCrashPointSeed> = (0..all_persisted_mask)
        .map(|persisted_mask| {
            seed_of_the_second_crash_points(
                HistorySeed(0),
                &CrashPoint {
                    segment_index: 16,
                    persisted_within_the_segment: (0..WRITES_OF_THE_SEGMENT)
                        .map(|write_within_the_segment| {
                            (persisted_mask >> write_within_the_segment) & 1 == 1
                        })
                        .collect(),
                    step: StepPosition::Operation(0),
                    operation_kind: Some(HistoryOperationKind::PublishOverwrite),
                },
            )
        })
        .collect();
    assert_eq!(
        seeds.len(),
        4095,
        "长 12 的段上 4095 个第一次崩溃态（扣下集合非空）算出的二次崩溃种子两两不同"
    );
}

/// F3-d 三条用例变的是段末这么多个写的持久子集。
const WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT: usize = 12;

/// 段末 [`WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT`] 个写的持久子集里扣下集合非空的那几种（全持久不是崩在这一段里的态）：4095 种。
const FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT: usize =
    (1 << WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT) - 1;

/// 一个段上的第一次崩溃态逐个算二次崩溃的种子：段号 `segment_index`，段内前 `writes_landed_before_the_varied_end` 个写都落了，
/// 段末 [`WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT`] 个写取扣下集合非空的全部 4095 种（段末第 k 个写落没落取掩码第 k 位），
/// 次序照掩码从 0 起。F3-d 三条用例共用这一份搭建，各自数种子、各自断言。
fn seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
    seed: HistorySeed,
    segment_index: usize,
    writes_landed_before_the_varied_end: usize,
) -> Vec<SecondCrashPointSeed> {
    (0..FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT)
        .map(|persisted_mask_of_the_varied_end| {
            seed_of_the_second_crash_points(
                seed,
                &CrashPoint {
                    segment_index,
                    persisted_within_the_segment: std::iter::repeat_n(
                        true,
                        writes_landed_before_the_varied_end,
                    )
                    .chain((0..WRITES_VARIED_AT_THE_END_OF_THE_SEGMENT).map(
                        |write_within_the_varied_end| {
                            (persisted_mask_of_the_varied_end >> write_within_the_varied_end) & 1
                                == 1
                        },
                    ))
                    .collect(),
                    step: StepPosition::Operation(0),
                    operation_kind: Some(HistoryOperationKind::PublishOverwrite),
                },
            )
        })
        .collect()
}

/// 二次崩溃的种子跨段号也两两不同（攻方改法 F3-d，判决 `research/prompts/m3-formal-code-b23-r1-main-verification.md` 第二节
/// 「K1 F3-c 4095 断言的判别力」与第三节第 2 条）：历史种子 0，段号 0..=63 各取长 12 的段上扣下集合非空的 4095 个第一次崩溃态，
/// 64 × 4095 = 262080 个种子合成一个集合数，两两不同——不只是同一段号里各自不同，段号 a 上的一个子集与段号 b 上的另一个子集也不撞
/// （攻方 `f3_fix` 的 A 那一格就是这样数的：`research/prompts/m3-formal-code-b23-r1-opus-model/probe/opus_probe.rs`）。
/// 上一条固定段号 16，种子里去掉段号、或四样改成或，它照绿。
/// 判别力：种子不异或段号、指纹改成按位拼接（段号与子集掩码在低位撞）、四样改成或（`crates/mutations.tsv` 各一行）时这一条红；
/// 攻方在模型上量过三种各剩 4095、4096、13254 个（`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 的 A 那一列）。
#[test]
fn first_crash_states_of_segments_zero_to_sixty_three_seed_their_second_crashes_differently_across_segments(
) {
    const SEGMENT_INDEXES: std::ops::RangeInclusive<usize> = 0..=63;
    let seeds: Vec<SecondCrashPointSeed> = SEGMENT_INDEXES
        .flat_map(|segment_index| {
            seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
                HistorySeed(0),
                segment_index,
                0,
            )
        })
        .collect();
    assert_eq!(
        seeds.len(),
        SEGMENT_INDEXES.count() * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
        "前提：段号 0..=63 各 4095 个第一次崩溃态，共 262080 个"
    );
    let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
    assert_eq!(
            distinct_seeds.len(),
            seeds.len(),
            "历史种子 0、段号 0..=63、长 12 的段上 262080 个第一次崩溃态算出的二次崩溃种子跨段号也两两不同"
        );
}

/// 长于 12 的段末 12 个写也折进种子（F3-d，同上判决）：历史种子 0、段号 16、长 28 的段，前 16 个写都落了、末 12 个写取扣下集合非空的
/// 4095 种，4095 个种子两两不同。真历史里的候选段长到 26（384 槽）/ 30（4 GiB）个写（攻方 `research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-segments.txt`），
/// 长 12 的段只看得见前 12 个写：只折前 12 / 16 个写的指纹在上一条照绿，在这里末 12 个写全被丢掉。
/// 判别力：指纹只折段内前 12 个写、只折前 16 个写（`crates/mutations.tsv` 各一行）时 4095 个第一次崩溃态只剩 1 个种子，这一条红
/// （攻方量过，`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 的 B 那一列）。
#[test]
fn first_crash_states_of_a_segment_of_twenty_eight_writes_differing_only_in_the_last_twelve_seed_their_second_crashes_differently(
) {
    const WRITES_LANDED_BEFORE_THE_VARIED_END: usize = 16;
    let seeds = seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
        HistorySeed(0),
        16,
        WRITES_LANDED_BEFORE_THE_VARIED_END,
    );
    let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
    assert_eq!(
        distinct_seeds.len(),
        FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
        "长 28 的段、前 16 个写都落了、末 12 个写取 4095 种：二次崩溃种子两两不同"
    );
}

/// 二次崩溃的种子随历史种子变（F3-d，同上判决）：段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 1 上各算一遍，
/// 8190 个种子合成一个集合数，两两不同。
/// 判别力：种子不异或历史种子（`crates/mutations.tsv` 一行）时两个历史种子上同一子集撞成一个，这一条红；
/// 攻方在模型上量过它剩 4095 个，按位拼接剩 4096 个、四样改成或剩 5998 个（`research/prompts/m3-formal-code-b23-r1-opus-model/out/f3-fix.txt` 的 C 那一列）。
#[test]
fn first_crash_states_under_history_seeds_zero_and_one_seed_their_second_crashes_differently() {
    const HISTORY_SEEDS: [HistorySeed; 2] = [HistorySeed(0), HistorySeed(1)];
    let seeds: Vec<SecondCrashPointSeed> = HISTORY_SEEDS
        .into_iter()
        .flat_map(|seed| {
            seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
                seed, 16, 0,
            )
        })
        .collect();
    let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
    assert_eq!(
            distinct_seeds.len(),
            HISTORY_SEEDS.len() * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
            "段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 1 上算出的 8190 个二次崩溃种子两两不同"
        );
}

/// 二次崩溃的种子在段号过 63 之后也两两不同（攻方改法 F3-e 的 A2，判决 `research/prompts/m3-formal-code-b24-r1-main-verification.md`
/// 第二节「K1 改法 F3-e」与第三节第 2 条；原型 `research/prompts/m3-formal-code-b24-r1-opus-model/probe/opus-b24-probe/src/main.rs` 的
/// `f3_proposed`）：历史种子 0，段号 0..=63 再加 2^k、2^k + 1（k = 6..=20）共 94 个段号，各取长 12 的段上扣下集合非空的 4095 个第一次崩溃态，
/// 384930 个种子合成一个集合数，两两不同。真历史里的段号过得了 63（设变量放大跑的那一批战役量到 210；战役的段从整条流起点数、含 mkfs，
/// mkfs 作废屏障之后同一段历史各段号加 1，今天的数没量），段号 0..=63 那一条看不见只取段号低 6 位的种子。
/// 被攻过零轮：这一组取样（段号 2^k、2^k + 1）对更长的历史、别的种子基罩不罩得住没人攻过。
/// 判别力：段号只取低 6 位（`crates/mutations.tsv` 一行，攻方 M7d）时段号 64 与 0 撞，这一条红。
#[test]
fn first_crash_states_of_segments_past_sixty_three_seed_their_second_crashes_differently_across_segments(
) {
    let segment_indexes: Vec<usize> = (0..=63)
        .chain((6..=20).flat_map(|exponent| [1_usize << exponent, (1_usize << exponent) + 1]))
        .collect();
    let seeds: Vec<SecondCrashPointSeed> = segment_indexes
        .iter()
        .flat_map(|segment_index| {
            seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
                HistorySeed(0),
                *segment_index,
                0,
            )
        })
        .collect();
    assert_eq!(
        seeds.len(),
        94 * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
        "前提：94 个段号各 4095 个第一次崩溃态，共 384930 个"
    );
    let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
    assert_eq!(
            distinct_seeds.len(),
            seeds.len(),
            "历史种子 0、段号 0..=63 与 2^k、2^k + 1（k = 6..=20）、长 12 的段上 384930 个第一次崩溃态算出的二次崩溃种子两两不同"
        );
}

/// 长 32 的段里扣下的写落在任何位置都折进种子（攻方改法 F3-e 的 B3，同上判决与原型）：历史种子 0、段号 16、长 32 的段，扣下其中一个写
/// （32 种）或两个写（496 种），528 个第一次崩溃态的种子两两不同。上面长 28 的那一条只变段末 12 个写，只折段末 12 个写的指纹在它上面照绿。
/// 被攻过零轮：段长 32、只扣一两个写的取样对更长的段、扣得更多的子集罩不罩得住没人攻过。
/// 判别力：指纹只折段内末 12 个写（`crates/mutations.tsv` 一行，攻方 M7a）时扣下前 20 个写里哪一个都撞成一个种子，这一条红。
#[test]
fn first_crash_states_of_a_segment_of_thirty_two_writes_withholding_one_or_two_writes_anywhere_seed_their_second_crashes_differently(
) {
    const WRITES_OF_THE_SEGMENT: usize = 32;
    let first_crash_with_withheld = |withheld: &[usize]| CrashPoint {
        segment_index: 16,
        persisted_within_the_segment: (0..WRITES_OF_THE_SEGMENT)
            .map(|write_within_the_segment| !withheld.contains(&write_within_the_segment))
            .collect(),
        step: StepPosition::Operation(0),
        operation_kind: Some(HistoryOperationKind::PublishOverwrite),
    };
    let seeds: Vec<SecondCrashPointSeed> = (0..WRITES_OF_THE_SEGMENT)
        .flat_map(|first_withheld| {
            std::iter::once(vec![first_withheld]).chain(
                (first_withheld + 1..WRITES_OF_THE_SEGMENT)
                    .map(move |second_withheld| vec![first_withheld, second_withheld]),
            )
        })
        .map(|withheld| {
            seed_of_the_second_crash_points(HistorySeed(0), &first_crash_with_withheld(&withheld))
        })
        .collect();
    assert_eq!(
        seeds.len(),
        528,
        "前提：长 32 的段扣下一个写 32 种、两个写 496 种"
    );
    let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
    assert_eq!(
            distinct_seeds.len(),
            seeds.len(),
            "历史种子 0、段号 16、长 32 的段上扣下一两个写的 528 个第一次崩溃态算出的二次崩溃种子两两不同"
        );
}

/// 二次崩溃的种子随历史种子的高位也变（攻方改法 F3-e 的 C2，同上判决与原型）：段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0、1、2^63
/// 上各算一遍，12285 个种子两两不同。上面历史种子 0 与 1 那一条只差最低位，只取历史种子最低位的种子在它上面照绿。
/// 被攻过零轮：历史种子 0、1、2^63 这三个取样对别的种子基罩不罩得住没人攻过。
/// 判别力：历史种子只取最低位（`crates/mutations.tsv` 一行，攻方 M7f）时 0 与 2^63 撞，这一条红。
#[test]
fn first_crash_states_under_history_seeds_zero_one_and_two_to_the_sixty_third_seed_their_second_crashes_differently(
) {
    const HISTORY_SEEDS: [HistorySeed; 3] = [HistorySeed(0), HistorySeed(1), HistorySeed(1 << 63)];
    let seeds: Vec<SecondCrashPointSeed> = HISTORY_SEEDS
        .into_iter()
        .flat_map(|seed| {
            seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
                seed, 16, 0,
            )
        })
        .collect();
    let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
    assert_eq!(
            distinct_seeds.len(),
            HISTORY_SEEDS.len() * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
            "段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0、1、2^63 上算出的 12285 个二次崩溃种子两两不同"
        );
}

/// 二次崩溃的种子随历史种子的每一位都变（攻方取样 C3，判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2
/// 「F3-e 取样（C4 / J2-f）」那一行与第三节第 1 条、改法 G6；原型 `research/prompts/m3-formal-code-b26-r1-opus-model/probe/src/f3.rs` 的
/// `F3PROPOSED`）：段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 2^k（k = 0..=63）共 65 个种子上各算一遍，266175 个种子两两不同。
/// 上面 0、1、2^63 那一条只看得见最低位与最高位，历史种子对 3 取余、只留低 8 位与最高位的种子在它上面照绿（攻方量过，`research/prompts/m3-formal-code-b26-r1-opus-model/out/f3.txt`）。
/// b24 已是最后一轮，只加宽取样：这一组取样对别的种子形状罩不罩得住没人攻过，剩下的漏形记在 b24 那笔欠账里。
/// 判别力：历史种子对 3 取余（`crates/mutations.tsv` 一行）时 2^k 与 0、1、2 撞，这一条红。
#[test]
fn first_crash_states_under_history_seeds_zero_and_every_power_of_two_seed_their_second_crashes_differently(
) {
    let history_seeds: Vec<HistorySeed> = std::iter::once(HistorySeed(0))
        .chain((0..64).map(|bit| HistorySeed(1_u64 << bit)))
        .collect();
    let seeds: Vec<SecondCrashPointSeed> = history_seeds
        .iter()
        .flat_map(|seed| {
            seeds_of_the_second_crashes_over_every_withheld_subset_of_the_end_of_a_segment(
                *seed, 16, 0,
            )
        })
        .collect();
    assert_eq!(
        seeds.len(),
        65 * FIRST_CRASH_STATES_OVER_THE_VARIED_END_OF_A_SEGMENT,
        "前提：历史种子 0 与 2^0..=2^63 共 65 个，各 4095 个第一次崩溃态"
    );
    let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
    assert_eq!(
            distinct_seeds.len(),
            seeds.len(),
            "段号 16、长 12 的段上 4095 个第一次崩溃态在历史种子 0 与 2^k（k = 0..=63）上算出的 266175 个二次崩溃种子两两不同"
        );
}

/// 长 1024 的段里扣下的那一个写落在任何位置都折进种子（攻方取样 B4，同上判决、改法 G6 与原型）：历史种子 0、段号 16、长 1024 的段，
/// 扣下其中一个写（1024 种），1024 个第一次崩溃态的种子两两不同。上面长 32 的那一条看不见第 32 个写之后：指纹只折段内前 32 个或末 32 个写的
/// 在它上面照绿（攻方量过，`research/prompts/m3-formal-code-b26-r1-opus-model/out/f3.txt`）。
/// b24 已是最后一轮，只加宽取样：段长 1024、只扣一个写的取样对更长的段、扣得更多的子集罩不罩得住没人攻过，剩下的漏形记在 b24 那笔欠账里。
/// 判别力：指纹只折段内前 32 个写（`crates/mutations.tsv` 一行）时扣下第 32 个写之后哪一个都撞成一个种子，这一条红。
#[test]
fn first_crash_states_of_a_segment_of_one_thousand_twenty_four_writes_withholding_one_write_anywhere_seed_their_second_crashes_differently(
) {
    const WRITES_OF_THE_SEGMENT: usize = 1024;
    let seeds: Vec<SecondCrashPointSeed> = (0..WRITES_OF_THE_SEGMENT)
        .map(|withheld| {
            seed_of_the_second_crash_points(
                HistorySeed(0),
                &CrashPoint {
                    segment_index: 16,
                    persisted_within_the_segment: (0..WRITES_OF_THE_SEGMENT)
                        .map(|write_within_the_segment| write_within_the_segment != withheld)
                        .collect(),
                    step: StepPosition::Operation(0),
                    operation_kind: Some(HistoryOperationKind::PublishOverwrite),
                },
            )
        })
        .collect();
    let distinct_seeds: BTreeSet<SecondCrashPointSeed> = seeds.iter().copied().collect();
    assert_eq!(
            distinct_seeds.len(),
            WRITES_OF_THE_SEGMENT,
            "历史种子 0、段号 16、长 1024 的段上扣下一个写的 1024 个第一次崩溃态算出的二次崩溃种子两两不同"
        );
}

/// I-8.4 的 R-换 在真的二次崩溃上不误红：两段历史（[`crash_injections_into_histories_reaching_every_mount_phase`]），
/// 起点那一段也摆崩溃状态；每个二次崩溃镜像上都再挂了一次，取号、写行、暖机、续做四段各有判过的，
/// 读出的内容都与没崩过的那一次挂载之后读出的相同（攻方量到实例代号、实例表行、扇区都不同的那种也算相同）。
#[test]
fn the_remount_after_each_second_crash_reads_the_same_content_as_the_uninterrupted_mount_in_every_mount_phase(
) {
    let (tally, injections) = crash_injections_into_histories_reaching_every_mount_phase();
    let tally = &tally;
    assert!(
        injections
            .iter()
            .flat_map(|injection| injection.crash_points.iter())
            .any(|crash_point| crash_point.step == StepPosition::StartingPoint),
        "起点那一段（新池新建文件做完）里要有崩溃状态"
    );
    assert_eq!(
        tally.remounts_after_a_second_crash, tally.second_crash_points,
        "每个二次崩溃镜像上都再挂过一次：{tally:?}"
    );
    for phase in WritableMountPhase::ALL {
        assert!(
            tally
                .remount_idempotence_judged_by_phase
                .get(&phase)
                .copied()
                .unwrap_or(0)
                >= 1,
            "崩在挂载的 {} 里再挂，一次都没判过 I-8.4：判了的 {:?}，没判的 {:?}",
            phase.name(),
            tally.remount_idempotence_judged_by_phase,
            tally.remount_idempotence_not_judged
        );
    }
    assert_eq!(
        tally.remounts_after_a_second_crash_reading_different_content,
        0,
        "二次崩溃之后再挂读出的内容与没崩过的那一次不同（I-8.4 的 R-换 判红）：{:?}",
        injections
            .iter()
            .flat_map(|injection| injection.new_findings.iter())
            .filter(|finding| matches!(
                finding.stage,
                CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        tally.remounts_after_a_second_crash_reading_the_same_content,
        tally
            .remount_idempotence_judged_by_phase
            .values()
            .sum::<u64>(),
        "判过的都读出相同的内容"
    );
}

/// 只有第一块盘最新那一槽系统配置读回全 0 时的可写挂载（C696（崩溃注入的读故障一支在随机历史里从没触发））：读故障下发写的那一种
/// （每块盘都藏的那一种被乙入判不出拒可写，一个写都不发）。
const NEWEST_SYSTEM_CONFIGURATION_SLOT_ZEROS_MOUNT: HistoryOperationKind =
    HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros;

/// 单元区 384 槽的小盘、不逐步跑 checker（设计调查量读故障那一支用的那一档）。
fn execution_on_384_slot_devices_without_the_per_step_checker() -> HistoryExecution {
    use crate::history::{HistoryDeviceWidth, PerStepChecker};
    HistoryExecution {
        per_step_checker: PerStepChecker::Skipped,
        device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
        space_admission: SpaceAdmission::JudgedByTheFormula,
    }
}

/// C696 判别力（步表那一半；设计调查报告第三节：候选「丁」之下 2676 个注入段里 614 段按写表下标会被认成没注入）：
/// 比重取 `GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS`，从这个测试周期的种子基起取前 4 段「只有第一块盘最新那一槽
/// 系统配置读回全 0 的可写挂载」往录制流里发过东西的历史（每段 24 步），每段按种子抽 16 个崩溃状态。用例自己再跑一遍同一段历史，照录制流下标
/// 独立数「崩溃窗口末尾那个写（录制流下标）之前那一步发过东西」的崩溃状态，崩溃注入的计数（`crash_points_after_a_step_reading_under_injected_faults`）
/// 要与它相等；另核这批崩溃状态里按录制流下标算注入过、按写表下标算没注入的至少一个（F5 那一种接错在这批上改得了计数），注入过的至少一个。
/// 判别力：读故障认定的崩溃窗口末尾交成写表下标（F5，`crates/mutations.tsv` 一行）时计数少了按写表下标认成没注入的那几个；认定函数恒交
/// NoneInjected（一行）时计数是 0——两种都与独立数的不等，这一条红。
#[test]
fn crash_states_after_a_newest_system_configuration_slot_zeros_mount_count_as_read_faulted_by_the_stream_index_of_the_crash_window(
) {
    use crate::history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
    const HISTORIES: usize = 4;
    const OPERATIONS_PER_HISTORY: usize = 24;
    const SEEDS_SEARCHED: u64 = 64;
    let execution = execution_on_384_slot_devices_without_the_per_step_checker();
    let mut histories_taken = 0usize;
    let mut counted_by_the_injection = 0u64;
    let mut read_faulted_by_the_stream_index = 0u64;
    let mut read_faulted_by_the_stream_index_but_not_by_the_write_index = 0u64;
    // 迭代上界是 SEEDS_SEARCHED 个种子；跨轮携带的是四样计数；取够 HISTORIES 段就停。
    for seed_offset in 0..SEEDS_SEARCHED {
        if histories_taken == HISTORIES {
            break;
        }
        let history = generate_history_with_weights(
            HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + seed_offset),
            OPERATIONS_PER_HISTORY,
            &GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS,
        );
        let stream = SharedStream::retaining_contents();
        let mut operations_after_each_step: Vec<(StepPosition, usize)> = Vec::new();
        execute_history_with(&history, execution, &stream, &mut |observation| {
            operations_after_each_step.push((observation.position, stream.operation_count()));
        });
        // 读故障下发过东西的那几步各自的第一个录制流下标。
        let mut operations_before_the_step = 0usize;
        let mut earliest_stream_index_of_each_read_faulted_step: Vec<usize> = Vec::new();
        for (position, operations_after_the_step) in &operations_after_each_step {
            if let StepPosition::Operation(step_index) = position {
                if history.operations[*step_index].kind()
                    == NEWEST_SYSTEM_CONFIGURATION_SLOT_ZEROS_MOUNT
                    && operations_before_the_step < *operations_after_the_step
                {
                    earliest_stream_index_of_each_read_faulted_step
                        .push(operations_before_the_step);
                }
            }
            operations_before_the_step = *operations_after_the_step;
        }
        if earliest_stream_index_of_each_read_faulted_step.is_empty() {
            continue;
        }
        let (_writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
            &stream.retained_operations(),
            &execution.device_width.fixed_geometry(),
        );
        let read_faulted_by_the_end_of_the_window = |last_index_of_the_window: usize| {
            earliest_stream_index_of_each_read_faulted_step
                .iter()
                .any(|earliest_stream_index| *earliest_stream_index <= last_index_of_the_window)
        };
        let injection = inject_crashes_into_history(
            &history,
            execution,
            CrashPointDraw::Sampled {
                crash_points_per_history: 16,
            },
            None,
            given_states_engine_for("crash_injection_unit_tests").engine(),
        );
        for crash_point in &injection.crash_points {
            let last_write_of_the_window = segments[crash_point.segment_index][0]
                + crash_point.persisted_within_the_segment.len()
                - 1;
            if read_faulted_by_the_end_of_the_window(stream_indexes[last_write_of_the_window]) {
                read_faulted_by_the_stream_index += 1;
                if !read_faulted_by_the_end_of_the_window(last_write_of_the_window) {
                    read_faulted_by_the_stream_index_but_not_by_the_write_index += 1;
                }
            }
        }
        counted_by_the_injection += injection
            .tally
            .crash_points_after_a_step_reading_under_injected_faults;
        histories_taken += 1;
    }
    assert_eq!(
        histories_taken, HISTORIES,
        "前提：种子基起 {SEEDS_SEARCHED} 段里有 {HISTORIES} 段的那一步往录制流里发过东西"
    );
    assert!(
        read_faulted_by_the_stream_index >= 1,
        "前提：抽到的崩溃状态里至少一个在那一步发过东西之后"
    );
    assert!(
            read_faulted_by_the_stream_index_but_not_by_the_write_index >= 1,
            "前提：至少一个崩溃状态按录制流下标算注入过、按写表下标算没注入（F5 那一种接错在这批上改得了计数）"
        );
    assert_eq!(
            counted_by_the_injection, read_faulted_by_the_stream_index,
            "崩溃注入认定的注入过读故障的崩溃状态数，与按录制流下标独立数的相等（其中 {read_faulted_by_the_stream_index_but_not_by_the_write_index} 个按写表下标会认成没注入）"
        );
}

/// 起点（取号、暖机、第一个文件）之后：一次「只有第一块盘最新那一槽系统配置读回全 0 的可写挂载」（读故障下发写，C696），再一次覆盖写。
fn history_of_a_newest_system_configuration_slot_zeros_mount_and_one_overwrite() -> GeneratedHistory
{
    use crate::history::{ContentChoice, ContentLength};
    GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![
                HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros,
                HistoryOperation::PublishOverwrite(ContentChoice {
                    length: ContentLength::InsideOneDataUnit { selector: 0 },
                    fill_seed: 1,
                }),
            ],
        }
}

/// 那段历史的写表与切段（与 [`inject_crashes_into_history`] 自己切的同一份），覆盖写那次发布的第一个写在写表里的下标；
/// 另核读回全 0 的那次可写挂载往录制流里发过东西、两步都做成。
fn writes_segments_and_first_write_of_the_overwrite_after_the_zeros_mount(
    history: &GeneratedHistory,
    execution: HistoryExecution,
) -> (Vec<RetainedWrite>, Vec<Vec<usize>>, usize) {
    let stream = SharedStream::retaining_contents();
    let mut operations_after_each_step: Vec<(StepPosition, usize)> = Vec::new();
    let run = execute_history_with(history, execution, &stream, &mut |observation| {
        operations_after_each_step.push((observation.position, stream.operation_count()));
    });
    assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
    let [(StepPosition::StartingPoint, after_the_starting_point), (StepPosition::Operation(0), after_the_zeros_mount), (StepPosition::Operation(1), _)] =
        operations_after_each_step[..]
    else {
        panic!("前提：起点与两步各登记一次：{operations_after_each_step:?}");
    };
    assert!(
        after_the_starting_point < after_the_zeros_mount,
        "前提：读回全 0 的那次可写挂载往录制流里发过东西（取号、写行、暖机）"
    );
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let first_write_of_the_overwrite = stream_indexes
        .iter()
        .position(|stream_index| *stream_index >= after_the_zeros_mount)
        .expect("覆盖写那次发布写过东西");
    (writes, segments, first_write_of_the_overwrite)
}

/// C696 与 D13（验证路线） 已定项 1 挂载透明的射程（只管不带故障的状态）：第一次崩溃摆在覆盖写那次发布的第一个写上（它一个写都没落，
/// 盘面停在读回全 0 的那次可写挂载做成之后），这个态注入过读故障，崩溃镜像上那一次可写挂载做成，挂载透明整个不判，它途中按段摆的
/// 三个二次崩溃也不判，同一理由共记四次没判。
/// 判别力（`crates/mutations.tsv` 各一行）：第一次崩溃镜像上照判挂载透明（C696 射程那一行）时没判的那一条理由只剩二次崩溃那 3 次；
/// 二次崩溃那一截不看标记、照判时它是 1；二次崩溃那一截跳过却不记没判时它也是 1。三种这一条都红。
// crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态（ExactlyOneCrashState），不枚举
#[test]
fn mount_transparency_is_not_judged_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount(
) {
    let history = history_of_a_newest_system_configuration_slot_zeros_mount_and_one_overwrite();
    let execution = execution_on_384_slot_devices_without_the_per_step_checker();
    let (_writes, segments, first_write_of_the_overwrite) =
        writes_segments_and_first_write_of_the_overwrite_after_the_zeros_mount(&history, execution);
    let overwrite_segment_index = segments
        .iter()
        .position(|segment| segment[0] == first_write_of_the_overwrite)
        .expect("覆盖写那次发布从一段的第一个写起（它前面是挂载最后那道屏障）");
    let injection = inject_crashes_into_history(
        &history,
        execution,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: overwrite_segment_index,
            writes_from_the_segment_on: 1,
            withheld_among_them: &[0],
        },
        None,
        given_states_engine_for("crash_injection_unit_tests").engine(),
    );
    let tally = &injection.tally;
    assert_eq!(
            (
                tally.crash_points,
                tally.crash_points_after_a_step_reading_under_injected_faults,
                tally.writable_mounts_after_the_crash_succeeded,
                tally.second_crash_points
            ),
            (1, 1, 1, 3),
            "前提：只摆写死的那一个第一次崩溃态，它注入过读故障，它上面的可写挂载做成，挂载途中按段摆了三个二次崩溃：{}",
            tally.render()
        );
    assert_eq!(
            (
                tally.mount_transparency_judged.clone(),
                tally
                    .mount_transparency_judged_on_second_crashes_by_phase
                    .clone(),
                tally
                    .mount_transparency_not_judged
                    .get(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
                    .copied()
            ),
            (BTreeMap::new(), BTreeMap::new(), Some(1 + 3)),
            "注入过读故障的崩溃态上挂载透明一个动作都不判：崩溃镜像上那次挂载记一次，它途中的三个二次崩溃各记一次：{}",
            tally.render()
        );
}

/// 「只有第一块盘最新那一槽系统配置读回全 0 的可写挂载」之后那次覆盖写的第一个写上摆第一次崩溃（注入过读故障的态），第二截那次可写挂载的
/// 写行那次发布上摆一个二次崩溃——根槽 FUA 没落、紧跟着的两槽系统配置轮换落了；交回这一趟崩溃注入。两条靶向用例共用这份搭建，各自断言。
fn injection_with_a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount(
) -> HistoryCrashInjection {
    let history = history_of_a_newest_system_configuration_slot_zeros_mount_and_one_overwrite();
    let execution = execution_on_384_slot_devices_without_the_per_step_checker();
    let (writes, segments, first_write_of_the_overwrite) =
        writes_segments_and_first_write_of_the_overwrite_after_the_zeros_mount(&history, execution);
    let overwrite_segment_index = segments
        .iter()
        .position(|segment| segment[0] == first_write_of_the_overwrite)
        .expect("覆盖写那次发布从一段的第一个写起（它前面是挂载最后那道屏障）");
    let mut first_crash_image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    first_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &first_crash_image,
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &CrashPoint {
            segment_index: overwrite_segment_index,
            persisted_within_the_segment: vec![false],
            step: StepPosition::Operation(1),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
        },
    );
    assert!(
        matches!(mount_run.outcome, WritableMountOutcome::Published),
        "前提：读回全 0 的那次挂载之后那一版上的可写挂载与之后那一次覆盖写都做成：{:?}",
        mount_run.outcome
    );
    let (mount_writes, mount_segments, _mount_stream_indexes) =
        writes_and_segments_with_stream_indexes(
            &mount_run.mount_operations,
            &execution.device_width.fixed_geometry(),
        );
    let phase_of_each_write = writable_mount_phase_of_each_write(
        &mount_writes,
        mount_run.publishes_before_the_deletion_queue_continuation,
        mount_run.publishes_before_the_instance_table_sweep,
    );
    let root_of_the_row_publish = (0..mount_writes.len())
        .find(|write| {
            phase_of_each_write[*write] == WritableMountPhase::RowPublish
                && mount_writes[*write].kind == StepKind::RootRecordFua
        })
        .expect("前提：写行那次发布写了根槽");
    let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
        .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
        .collect();
    assert_eq!(
        rotation_after_the_root.len(),
        2,
        "前提：写行那次发布的根槽 FUA 之后紧跟着每块盘轮换一槽系统配置"
    );
    let segment_of_the_root =
        segment_containing_the_write(&mount_segments, root_of_the_row_publish);
    let first_write_of_the_segment_of_the_root = mount_segments[segment_of_the_root][0];
    let last_rotation = rotation_after_the_root[rotation_after_the_root.len() - 1];
    inject_crashes_into_history_drawing_second_crashes(
        &history,
        execution,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: overwrite_segment_index,
            writes_from_the_segment_on: 1,
            withheld_among_them: &[0],
        },
        SecondCrashPointDraw::ExactlyOneSecondCrashState {
            segment_index: segment_of_the_root,
            writes_from_the_segment_on: last_rotation + 1 - first_write_of_the_segment_of_the_root,
            withheld_among_them: vec![
                root_of_the_row_publish - first_write_of_the_segment_of_the_root,
            ]
            .leak(),
        },
        None,
        given_states_engine_for("crash_injection_unit_tests").engine(),
    )
}

/// C696「第三截调用点写死成 NoneInjected」那一种接错的靶向用例（W 为真的阳性对照，造法同
/// [`second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site`]）：第一次崩溃摆在
/// 「只有第一块盘最新那一槽系统配置读回全 0 的可写挂载」之后那次覆盖写的第一个写上（注入过读故障的态），第二截那次可写挂载的写行那次发布上摆一个
/// 二次崩溃——根槽 FUA 没落、紧跟着的两槽系统配置轮换落了。二次崩溃镜像上再挂以判据 N-配置 拒、判据复算为真；崩溃态注入过读故障，
/// 第三截判过一次、不判 C670 那一条（不合法 0 次、没有新发现）。
/// 判别力：第三截调用点把读故障的认定写死成 NoneInjected（`crates/mutations.tsv` 一行）时不合法变 1、报一条新发现，这一条红。
/// 随机历史上那一种接错是等价变异（设计调查报告第三节：注入态与第三截二次崩溃态上判据从没为真），等价性留档在
/// `crates/singlefs-verification/tests/crash_injection_read_faulted_states_never_hold_the_witness_criterion.rs`。
// crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
#[test]
fn second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount_is_not_judged_an_illegal_witness_refusal(
) {
    let injection =
            injection_with_a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount();
    let tally = &injection.tally;
    assert_eq!(
        (
            tally.crash_points,
            tally.crash_points_after_a_step_reading_under_injected_faults,
            tally.writable_mounts_after_the_crash_succeeded
        ),
        (1, 1, 1),
        "前提：只摆写死的那一个第一次崩溃态，它注入过读故障，它上面的可写挂载做成：{}",
        tally.render()
    );
    assert_eq!(
        (
            tally.second_crash_points,
            tally
                .second_crash_points_by_phase
                .get(&WritableMountPhase::RowPublish)
                .copied()
        ),
        (1, Some(1)),
        "前提：只摆写死的那一个二次崩溃，崩在写行里：{}",
        tally.render()
    );
    assert_eq!(
            (
                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
            ),
            (1, 0),
            "二次崩溃镜像上再挂以判据 N-配置 拒、判过一次；第一次崩溃那个态注入过读故障，不判 C670 那一条：{:?}",
            injection
                .new_findings
                .iter()
                .map(CrashPointFinding::render)
                .collect::<Vec<_>>()
        );
}

/// 改法 G5（判决 `research/prompts/m3-formal-code-b26-r1-main-verification.md` 第二节 J2「二次崩溃镜像在注入态下照判（J2-d）」那一行，
/// D13（验证路线） 已定项 1「带故障的状态不在射程」）：同上那一份搭建——第一次崩溃那个态注入过读故障，它上面那次可写挂载做成，写行里摆一个
/// 二次崩溃。第二截那一次挂载透明不判；那个二次崩溃镜像上的挂载透明也不判，与第二截同一个理由再记一次（共 2 次），按段判过的一次都没有。
/// 判别力：二次崩溃那一截不看读故障标记、照判挂载透明（`crates/mutations.tsv` 一行）时按段判过的有写行 1 次、没判的只记 1 次，这一条红。
// crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
#[test]
fn mount_transparency_is_not_judged_on_a_second_crash_inside_the_mount_on_a_crash_state_after_a_newest_system_configuration_slot_zeros_mount(
) {
    let injection =
            injection_with_a_second_crash_where_only_the_rotation_landed_after_a_newest_system_configuration_slot_zeros_mount();
    let tally = &injection.tally;
    assert_eq!(
            (
                tally.crash_points_after_a_step_reading_under_injected_faults,
                tally.writable_mounts_after_the_crash_succeeded,
                tally.second_crash_points
            ),
            (1, 1, 1),
            "前提：只摆写死的那一个第一次崩溃态（注入过读故障，它上面的可写挂载做成）与那一个二次崩溃：{}",
            tally.render()
        );
    assert_eq!(
        (
            tally
                .mount_transparency_judged_on_second_crashes_by_phase
                .clone(),
            tally
                .mount_transparency_not_judged
                .get(MOUNT_TRANSPARENCY_NOT_JUDGED_ON_A_READ_FAULTED_CRASH_STATE)
                .copied()
        ),
        (BTreeMap::new(), Some(2)),
        "注入过读故障的崩溃态上，那次挂载与它途中的二次崩溃都不判挂载透明，按理由各记一次：{}",
        tally.render()
    );
}

/// C670 在第三截的判法（攻方改法 P1）：把「只落了覆盖写那次轮换」的盘面当成一个二次崩溃镜像交给第三截，没崩过的那一次挂载做成过（`once` 有内容）。
/// 再挂以判据 N-配置 拒；崩溃态没注入读故障、判据在这份盘面上复算为真：I-8.4 照旧记「再挂被拒」不判，C670 那一条判过一次、不合法一次，
/// 记一条第三截的发现。判别力：再挂被拒那一支不判 C670、或换回不判 C670 的旧判官（`crates/mutations.tsv` 两行），这一条红。
#[test]
fn remount_refused_on_a_second_crash_image_where_only_a_rotation_landed_is_an_illegal_witness_refusal(
) {
    let (history, execution) = history_of_the_first_file_and_one_overwrite();
    let (writes, _segments, first_write_of_the_overwrite) =
        writes_segments_and_first_write_of_the_overwrite(&history, execution);
    let writes_of_the_overwrite = &writes[first_write_of_the_overwrite..];
    assert_the_overwrite_rotates_one_system_configuration_slot_per_device(writes_of_the_overwrite);
    let mut second_crash_image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    second_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
    let rotation: Vec<RetainedWrite> = writes_of_the_overwrite
        .iter()
        .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
        .cloned()
        .collect();
    second_crash_image.apply_writes(&rotation);
    let recovered = read_recovery_fields(&second_crash_image, ComparedVersion::TheRecoveredVersion);
    let crash_point = CrashPoint {
        segment_index: 0,
        persisted_within_the_segment: vec![false],
        step: StepPosition::StartingPoint,
        operation_kind: None,
    };
    let mut tally = CrashInjectionTally::default();
    let mut findings = FindingsOfOneHistory {
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
    };
    judge_the_remount_after_a_second_crash(
        &mut tally,
        &mut findings,
        &Ok(content_read_by_the_recovery_report(&recovered.report)),
        &RefusalOnTheCrashImage {
            crash_image_materialized: &second_crash_image,
            filesystem_identifier_of_the_pool: &execution
                .device_width
                .parameters()
                .filesystem_identifier,
            version_landed_on_after_the_crash: recovered.report.effective_root,
            read_only_read_back: ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            read_faults: ReadFaultsInTheCrashState::NoneInjected,
        },
        &execution.device_width.parameters(),
        execution.space_admission,
        InstanceTableSweepConfiguration::PRODUCT,
        WritableMountPhase::RowPublish,
        history.seed,
        &crash_point,
        None,
    );
    assert_eq!(
        tally
            .remount_idempotence_not_judged
            .get(&RemountIdempotenceNotJudged::TheRemountWasRefused)
            .copied(),
        Some(1),
        "前提：再挂被拒，I-8.4 不判：{}",
        tally.render()
    );
    assert_eq!(
        (
            tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
            tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
        ),
        (1, 1),
        "拒因是判据 N-配置、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
        tally.render()
    );
    assert_eq!(
        findings.known_red_hits.len() + findings.new_findings.len(),
        1,
        "第三截记一条：已知红 {:?}，新发现 {:?}",
        findings.known_red_hits,
        findings
            .new_findings
            .iter()
            .map(CrashPointFinding::render)
            .collect::<Vec<_>>()
    );
}

/// Q3 在第三截的计数（m3-close-g4-rewrite- V4 那一行：Q3 第三截的计数没有测试断言、变异表也没有行点名）：一份不合法的 Q3 拒
/// （崩溃态没注入读故障、判据复算为真）与一份合法的 Q3 拒各交一次给第三截那一判，再把这一份计数并进另一份已记过一次判过、一次不合法的计数。
/// 预期：这一份判过 1 + 1 次、不合法 1 次，不合法那一份交回拒因与那一条条件、合法那一份交回 None；并起来判过 1 + 2、不合法 1 + 1，
/// 渲染里 Q3 那一行带这两个数。判别力：`crates/mutations.tsv`「根环一簇 R12」那一行（第三截不记 Q3 那一条不合法），这一条红。
#[test]
fn the_third_cut_counts_each_no_valid_root_refusal_and_each_one_whose_criterion_is_true_without_read_faults(
) {
    let mut tally = CrashInjectionTally::default();
    let criterion_true_without_read_faults = vec![
        RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
    ];
    assert_eq!(
            reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
                &mut tally,
                WritableMountRefusalVerdict::Illegal {
                    reason: RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator,
                    conditions_not_met: criterion_true_without_read_faults.clone(),
                },
            ),
            Some((
                RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator,
                criterion_true_without_read_faults
            )),
            "不合法的 Q3 拒：交回拒因与那一条条件（报红用）"
        );
    assert_eq!(
            reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
                &mut tally,
                WritableMountRefusalVerdict::Legal(
                    RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator
                ),
            ),
            None,
            "合法的 Q3 拒：只记判过、不报红"
        );
    assert_eq!(
            (
                tally.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
                tally.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults
            ),
            (1 + 1, 1),
            "这一份：判过两次（不合法、合法各一），不合法一次：{}",
            tally.render()
        );
    let mut earlier = CrashInjectionTally::default();
    assert!(
            reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults(
                &mut earlier,
                WritableMountRefusalVerdict::Illegal {
                    reason: RefusalReasonOnTheLegalRefusalList::NoValidRootInTheRootRingOfTheRebuiltAllocator,
                    conditions_not_met: vec![
                        RefusalConditionNotMet::NoValidRootInTheRootRingCriterionTrueOnACrashImageWithoutReadFaults,
                    ],
                },
            )
            .is_some(),
            "另一份先记过一次不合法的 Q3 拒"
        );
    earlier.absorb(&tally);
    assert_eq!(
            (
                earlier.remount_refusals_for_no_valid_root_in_the_root_ring_of_the_rebuilt_allocator,
                earlier.remount_refusals_with_the_no_valid_root_in_the_root_ring_criterion_true_without_read_faults
            ),
            (1 + 2, 1 + 1),
            "并起来：判过、不合法各按两份相加：{}",
            earlier.render()
        );
    assert!(
        earlier.render().contains(
            "（第三截判 Q3 那一条）：3 次，崩溃态没注入读故障而判据复算为真、拒得不合法 2"
        ),
        "渲染里 Q3 那一行带判过与不合法两个数：{}",
        earlier.render()
    );
}

/// 改法 G4 在第三截的判法（与 C670 那一条同一个接法）：第一个文件之后的盘面上，盘 0 两个系统配置槽各改一个字节（读得出、自证不过，设备不报错），
/// 另把盘 0 上所选那一版的一个单元（两盘同槽的那一个，盘 1 那一份不动）改一个字节，当成一个二次崩溃镜像交给第三截，没崩过的那一次挂载做成过
/// （`once` 有内容）。盘 0 两槽全废，可写挂载照丙' 过补写闸；那个单元在盘 0 上读出的字节与验得过的那一份不同，补写闸闸一
/// （`RewriteGateNotMet::UnitsOfTheSelectedVersionNotAllOnTheDevice`，`singlefs_core::mount` 的补写闸）不过，不补写、照旧拒。
/// 只改两槽、单元都在的盘面过得了补写闸、挂上了，拒不出来（三方判决 `research/prompts/m3-close-g4-rewrite-r1-main-verification.md` 第三节，测试走甲）。
/// 再挂以「读阶段判完之后系统配置又读不出」拒；崩溃态没注入读故障、那一拒的判据在这份盘面上复算为真（盘 0 一份自证过的系统配置都没有）：
/// I-8.4 照旧记「再挂被拒」不判，G4 那一条判过一次、不合法一次，记一条第三截的发现；判据 N-配置 那两样计数不动。
/// 判别力：第三截不认 G4 那一条（`crates/mutations.tsv` 一行），这一条红。
#[test]
fn remount_refused_on_a_second_crash_image_where_device_zero_has_both_system_configuration_slots_torn_is_an_illegal_refusal_after_the_read_stage(
) {
    /// 撕裂一个系统配置槽时改的那个字节在槽里的位置：槽中间，落在整槽校验和罩着的范围里。
    const TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT: usize = 2048;
    let (history, execution) = history_of_the_first_file_and_one_overwrite();
    let (writes, _segments, first_write_of_the_overwrite) =
        writes_segments_and_first_write_of_the_overwrite(&history, execution);
    let mut second_crash_image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    second_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
    let slot_spacing = u64::from(
        execution
            .device_width
            .parameters()
            .geometry
            .fixed_structure_slot_spacing,
    );
    let slot_bytes = usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES)
        .expect("系统配置槽 4096 字节装得进 usize");
    // 迭代上界是一块盘的两个系统配置槽；不跨轮携带状态；没有提前出口。
    for slot_index in 0..singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let offset = DeviceOffsetInBytes(slot_index * slot_spacing);
        let mut slot = PoolReader::read(&second_crash_image, DeviceIdentity(0), offset, slot_bytes)
            .expect("内存盘上系统配置槽读得出");
        slot[TORN_BYTE_INSIDE_THE_SYSTEM_CONFIGURATION_SLOT] ^= 0xFF;
        second_crash_image
            .devices
            .get_mut(&DeviceIdentity(0))
            .expect("池里有盘 0")
            .write(offset, &slot);
    }
    // 补写闸闸一不过：盘 0 上所选那一版的一个单元改一个字节。挑第一个文件那次发布在盘 0 上写的最后一个单元、而盘 1 同一落点也写过
    // （两盘同槽，盘 1 那一份验得过，择到的那一版照旧是它）
    let unit_write_on_both_devices = writes[..first_write_of_the_overwrite]
        .iter()
        .rev()
        .find(|write| {
            write.kind == StepKind::UnitWrite
                && write.device == DeviceIdentity(0)
                && writes[..first_write_of_the_overwrite].iter().any(|mirror| {
                    mirror.kind == StepKind::UnitWrite
                        && mirror.device == DeviceIdentity(1)
                        && mirror.offset == write.offset
                })
        })
        .expect("前提：第一个文件那次发布在两块盘的同一落点上各写了一份单元");
    let unit_length = usize::try_from(unit_write_on_both_devices.length_in_bytes())
        .expect("一个单元装得进 usize");
    let mut unit_on_device_zero = PoolReader::read(
        &second_crash_image,
        DeviceIdentity(0),
        unit_write_on_both_devices.offset,
        unit_length,
    )
    .expect("内存盘上单元读得出");
    unit_on_device_zero[unit_length / 2] ^= 0xFF;
    second_crash_image
        .devices
        .get_mut(&DeviceIdentity(0))
        .expect("池里有盘 0")
        .write(unit_write_on_both_devices.offset, &unit_on_device_zero);
    let recovered = read_recovery_fields(&second_crash_image, ComparedVersion::TheRecoveredVersion);
    let crash_point = CrashPoint {
        segment_index: 0,
        persisted_within_the_segment: vec![false],
        step: StepPosition::StartingPoint,
        operation_kind: None,
    };
    let mut tally = CrashInjectionTally::default();
    let mut findings = FindingsOfOneHistory {
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
    };
    judge_the_remount_after_a_second_crash(
        &mut tally,
        &mut findings,
        &Ok(content_read_by_the_recovery_report(&recovered.report)),
        &RefusalOnTheCrashImage {
            crash_image_materialized: &second_crash_image,
            filesystem_identifier_of_the_pool: &execution
                .device_width
                .parameters()
                .filesystem_identifier,
            version_landed_on_after_the_crash: recovered.report.effective_root,
            read_only_read_back: ReadOnlyReadBackOfTheCrashImage::InsideTheAllowedSet,
            read_faults: ReadFaultsInTheCrashState::NoneInjected,
        },
        &execution.device_width.parameters(),
        execution.space_admission,
        InstanceTableSweepConfiguration::PRODUCT,
        WritableMountPhase::RowPublish,
        history.seed,
        &crash_point,
        None,
    );
    assert_eq!(
        tally
            .remount_idempotence_not_judged
            .get(&RemountIdempotenceNotJudged::TheRemountWasRefused)
            .copied(),
        Some(1),
        "前提：再挂被拒，I-8.4 不判：{}",
        tally.render()
    );
    assert_eq!(
        (
            tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
            tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
        ),
        (0, 0),
        "前提：拒因不是判据 N-配置：{}",
        tally.render()
    );
    assert_eq!(
            (
                tally.remount_refusals_for_a_system_configuration_unreadable_after_the_read_stage,
                tally.remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults
            ),
            (1, 1),
            "拒因是读阶段判完之后系统配置又读不出、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
            tally.render()
        );
    assert_eq!(
        findings.known_red_hits.len() + findings.new_findings.len(),
        1,
        "第三截记一条：已知红 {:?}，新发现 {:?}",
        findings.known_red_hits,
        findings
            .new_findings
            .iter()
            .map(CrashPointFinding::render)
            .collect::<Vec<_>>()
    );
}

/// C670 在第三截的判别力（攻方改法 P1，`research/prompts/m3-formal-code-b16-r1-main-verification.md` 第二节 K2-b）：起点加一次覆盖写、不重挂，
/// 短段整段枚举；历史里没有带读故障的步。二次崩溃之后再挂以判据 N-配置 被拒、判据在二次崩溃镜像上复算为真的，一个都没有。
/// 攻方自造的坏实现 V242u（只在带单元的发布上先轮换后根，`crates/mutations.tsv` 里那一行）在这段历史上第二截看不见（第一次崩溃镜像上 0 个：
/// 同一实例的记录照重放），第二截那次挂载的写行是新实例第一次带单元的发布，「轮换落了、根没落」只经二次崩溃摆得出来——攻方原型逐个枚举短段量到 108 态；
/// 这里每个崩溃态每段只抽一个二次崩溃，能不能抽中是推的（报告里估了抽中的概率）。
// crash-case-check:not-a-crash-case 起点加一步的合成历史、短段整段枚举，不调全量枚举，崩溃态数以百计（推的，没量过）
#[test]
fn no_remount_after_a_second_crash_without_read_faults_is_refused_writable_for_a_witness_newer_than_the_selected_version(
) {
    let (history, execution) = history_of_the_first_file_and_one_overwrite();
    let injection = inject_crashes_into_history(
        &history,
        execution,
        CrashPointDraw::EveryProperSubsetOfShortSegments {
            segment_writes: 4,
            sampled_in_longer_segments: 2,
        },
        None,
        given_states_engine_for("crash_injection_unit_tests").engine(),
    );
    let tally = &injection.tally;
    assert_eq!(
        tally.crash_points_after_a_step_reading_under_injected_faults,
        0,
        "这段历史没有带读故障的步，每个崩溃态都判 C670 那一条：{}",
        tally.render()
    );
    assert!(
        injection.crash_points.iter().any(|crash_point| {
            matches!(
                crash_point.operation_kind,
                Some(HistoryOperationKind::PublishOverwrite)
            )
        }),
        "覆盖写那一步要摆到崩溃态：{}",
        tally.render()
    );
    assert!(
        tally
            .second_crash_points_by_phase
            .get(&WritableMountPhase::RowPublish)
            .copied()
            .unwrap_or(0)
            >= 1,
        "前提：写行那一段摆到了二次崩溃：{}",
        tally.render()
    );
    assert_eq!(
        tally.remount_refusals_with_the_witness_criterion_true_without_read_faults,
        0,
        "二次崩溃之后再挂以见证判据为由拒、崩溃态没注入读故障而判据复算为真（C670）：{:?}",
        injection
            .new_findings
            .iter()
            .filter(|finding| matches!(
                finding.stage,
                CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
            ))
            .map(CrashPointFinding::render)
            .collect::<Vec<_>>()
    );
}

/// C670 在第三截经调用点的判法（攻方改法 F2，`research/prompts/m3-formal-code-b18-r1-main-verification.md` 第二节 K2）：第一个文件之后一次覆盖写的历史，
/// 第一次崩溃摆在覆盖写那次发布的第一个写上（它一个写都没落，盘面就是起点那一版）；第二截那次可写挂载的写行那次发布上摆一个二次崩溃——
/// 根槽 FUA 没落、紧跟着的两槽系统配置轮换落了（越过根槽那一段的段尾：今天的持久顺序下根槽 FUA 先于轮换、各成一段，段内真子集摆不出来，
/// 与 [`crash_state_where_only_the_rotation_of_an_overwrite_landed_is_an_illegal_witness_refusal_of_the_writable_mount_after_the_crash`] 越过段尾同一个理由）。
/// 新实例的记录不跨实例边界施加，二次崩溃镜像上只读恢复落在起点那一版，见证值比它新：再挂以判据 N-配置 拒；崩溃态没注入读故障、判据复算为真，
/// 第三截判过一次、不合法一次。判别力：调用点（[`judge_second_crashes_inside_the_writable_mount`] 拼 [`RefusalOnTheCrashImage`] 那几行）
/// 把读故障恒交 `Injected`、再挂的盘换成第一次崩溃的镜像、换成没叠这一段的基线（`crates/mutations.tsv` 三行），不合法都变 0。
// crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
#[test]
fn second_crash_where_only_the_rotation_of_the_row_publish_landed_is_an_illegal_witness_refusal_through_the_call_site(
) {
    let (history, execution) = history_of_the_first_file_and_one_overwrite();
    let (writes, segments, first_write_of_the_overwrite) =
        writes_segments_and_first_write_of_the_overwrite(&history, execution);
    let overwrite_segment_index = segments
        .iter()
        .position(|segment| segment[0] == first_write_of_the_overwrite)
        .expect("覆盖写那次发布从一段的第一个写起（它前面是起点末尾那道屏障）");
    // 第二截那次挂载写出的写表：与注入里那一次同一份盘面（覆盖写一个写都没落）、同一份参数；挂载之后那一次发布写的内容随崩溃点变，不在挂载写表里。
    let mut first_crash_image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    first_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &first_crash_image,
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &CrashPoint {
            segment_index: overwrite_segment_index,
            persisted_within_the_segment: vec![false],
            step: StepPosition::Operation(0),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
        },
    );
    assert!(
        matches!(mount_run.outcome, WritableMountOutcome::Published),
        "前提：起点那一版上的可写挂载与之后那一次覆盖写都做成：{:?}",
        mount_run.outcome
    );
    let (mount_writes, mount_segments, _mount_stream_indexes) =
        writes_and_segments_with_stream_indexes(
            &mount_run.mount_operations,
            &execution.device_width.fixed_geometry(),
        );
    let phase_of_each_write = writable_mount_phase_of_each_write(
        &mount_writes,
        mount_run.publishes_before_the_deletion_queue_continuation,
        mount_run.publishes_before_the_instance_table_sweep,
    );
    let root_of_the_row_publish = (0..mount_writes.len())
        .find(|write| {
            phase_of_each_write[*write] == WritableMountPhase::RowPublish
                && mount_writes[*write].kind == StepKind::RootRecordFua
        })
        .expect("前提：写行那次发布写了根槽");
    let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
        .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
        .collect();
    assert_eq!(
        rotation_after_the_root.len(),
        2,
        "前提：写行那次发布的根槽 FUA 之后紧跟着每块盘轮换一槽系统配置"
    );
    let segment_of_the_root =
        segment_containing_the_write(&mount_segments, root_of_the_row_publish);
    let first_write_of_the_segment_of_the_root = mount_segments[segment_of_the_root][0];
    let last_rotation = rotation_after_the_root[rotation_after_the_root.len() - 1];
    let injection = inject_crashes_into_history_drawing_second_crashes(
        &history,
        execution,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: overwrite_segment_index,
            writes_from_the_segment_on: 1,
            withheld_among_them: &[0],
        },
        SecondCrashPointDraw::ExactlyOneSecondCrashState {
            segment_index: segment_of_the_root,
            writes_from_the_segment_on: last_rotation + 1 - first_write_of_the_segment_of_the_root,
            withheld_among_them: vec![
                root_of_the_row_publish - first_write_of_the_segment_of_the_root,
            ]
            .leak(),
        },
        None,
        given_states_engine_for("crash_injection_unit_tests").engine(),
    );
    let tally = &injection.tally;
    assert_eq!(
        (
            tally.crash_points,
            tally.writable_mounts_after_the_crash_succeeded
        ),
        (1, 1),
        "前提：只摆写死的那一个第一次崩溃态，它上面的可写挂载做成（没崩过的那一次读得出内容）：{}",
        tally.render()
    );
    assert_eq!(
        tally.crash_points_after_a_step_reading_under_injected_faults,
        0,
        "这段历史没有带读故障的步：{}",
        tally.render()
    );
    assert_eq!(
        (
            tally.second_crash_points,
            tally
                .second_crash_points_by_phase
                .get(&WritableMountPhase::RowPublish)
                .copied()
        ),
        (1, Some(1)),
        "前提：只摆写死的那一个二次崩溃，崩在写行里：{}",
        tally.render()
    );
    assert_eq!(
            (
                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
            ),
            (1, 1),
            "二次崩溃镜像上再挂以判据 N-配置 拒、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
            tally.render()
        );
}

/// 第三截调用点交给判官的「所选那一版」是二次崩溃镜像上只读恢复落到的那一版（判决 `research/prompts/m3-formal-code-b20-r1-main-verification.md`
/// 第二节 K1「第四种接错」）：第一次崩溃与上一条同一个（覆盖写一个写都没落，只读恢复落在起点那一版）；第二截那次可写挂载的写行那次发布上
/// 摆一个二次崩溃——写行那次的记录（每块盘一份）没落，根槽 FUA 与紧跟着的两槽系统配置轮换落了（越过记录与根槽之间那道屏障，与上一条同一个理由）。
/// 二次崩溃镜像上只读恢复落在写行那一版（新实例的根），与第一次崩溃不同；它那次发布的末条记录读不出，计数器等于 c_见证 的记录也读不出：
/// 再挂以判据 N-配置 拒，C670 那一条成立在「两条都读不出」那一支。判别力：调用点把所选那一版交成第一次崩溃那一版（`crates/mutations.tsv` 一行）时
/// 计数照样 (1, 1)，判官列出的那一支换成「c_见证 大于所选那一版末条记录的计数器」，红在那一支的两条断言上。计数分不开：在 harness 草稿探针里
/// 量过，三段短历史 131 个第一次崩溃态上挂载写表每个前缀与每个前缀扣一个写（177198 个二次崩溃态），以及这一个第一次崩溃态上每个前缀扣至多两个写，
/// 两种所选那一版的 (判过, 不合法) 没有一处不同；判官结论不同的只有扣下写行（或第一次暖机）那次记录、根槽落了的这一类，都只差这一支。
/// 那几处没量到的越屏障靶向态里计数分得开（攻方量过，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md` 第二节 K3），
/// 钉在 [`second_crash_losing_the_records_of_the_row_publish_after_a_first_crash_losing_the_records_of_the_first_file_counts_one_illegal_witness_refusal`]。
// crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
#[test]
fn second_crash_landing_on_the_row_publish_whose_records_were_lost_names_the_selected_version_of_the_second_crash_image(
) {
    let (history, execution) = history_of_the_first_file_and_one_overwrite();
    let (writes, segments, first_write_of_the_overwrite) =
        writes_segments_and_first_write_of_the_overwrite(&history, execution);
    let overwrite_segment_index = segments
        .iter()
        .position(|segment| segment[0] == first_write_of_the_overwrite)
        .expect("覆盖写那次发布从一段的第一个写起（它前面是起点末尾那道屏障）");
    let mut first_crash_image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    first_crash_image.apply_writes(&writes[..first_write_of_the_overwrite]);
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &first_crash_image,
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &CrashPoint {
            segment_index: overwrite_segment_index,
            persisted_within_the_segment: vec![false],
            step: StepPosition::Operation(0),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
        },
    );
    assert!(
        matches!(mount_run.outcome, WritableMountOutcome::Published),
        "前提：起点那一版上的可写挂载与之后那一次覆盖写都做成：{:?}",
        mount_run.outcome
    );
    let (mount_writes, mount_segments, _mount_stream_indexes) =
        writes_and_segments_with_stream_indexes(
            &mount_run.mount_operations,
            &execution.device_width.fixed_geometry(),
        );
    let phase_of_each_write = writable_mount_phase_of_each_write(
        &mount_writes,
        mount_run.publishes_before_the_deletion_queue_continuation,
        mount_run.publishes_before_the_instance_table_sweep,
    );
    let root_of_the_row_publish = (0..mount_writes.len())
        .find(|write| {
            phase_of_each_write[*write] == WritableMountPhase::RowPublish
                && mount_writes[*write].kind == StepKind::RootRecordFua
        })
        .expect("前提：写行那次发布写了根槽");
    let records_of_the_row_publish: Vec<usize> = (0..root_of_the_row_publish)
        .filter(|write| {
            phase_of_each_write[*write] == WritableMountPhase::RowPublish
                && mount_writes[*write].kind == StepKind::JournalRecord
        })
        .collect();
    let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
        .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
        .collect();
    assert_eq!(
        (
            records_of_the_row_publish.len(),
            rotation_after_the_root.len()
        ),
        (2, 2),
        "前提：写行那次发布在根槽之前每块盘写一份记录，根槽 FUA 之后紧跟着每块盘轮换一槽系统配置"
    );
    let segment_of_the_first_record =
        segment_containing_the_write(&mount_segments, records_of_the_row_publish[0]);
    let first_write_of_the_segment = mount_segments[segment_of_the_first_record][0];
    let last_rotation = rotation_after_the_root[rotation_after_the_root.len() - 1];
    let withheld_among_them: Vec<usize> = records_of_the_row_publish
        .iter()
        .map(|record| record - first_write_of_the_segment)
        .collect();
    // 前提：两次崩溃之后只读恢复落的版本不同（同一份盘面在这里照摆法叠一遍）。
    let mut second_crash_image = first_crash_image.clone();
    second_crash_image.apply_writes(&mount_writes[..first_write_of_the_segment]);
    let persisted_writes_of_the_second_crash: Vec<RetainedWrite> = (first_write_of_the_segment
        ..=last_rotation)
        .filter(|write| !records_of_the_row_publish.contains(write))
        .map(|write| mount_writes[write].clone())
        .collect();
    second_crash_image.apply_writes(&persisted_writes_of_the_second_crash);
    let version_after_the_first_crash =
        recover(&first_crash_image, JournalPolicy::Consult).effective_root;
    let version_after_the_second_crash =
        recover(&second_crash_image, JournalPolicy::Consult).effective_root;
    assert_ne!(
        version_after_the_first_crash, version_after_the_second_crash,
        "前提：第一次崩溃落在起点那一版、二次崩溃落在写行那一版，两次只读恢复落的版本不同"
    );
    let injection = inject_crashes_into_history_drawing_second_crashes(
        &history,
        execution,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: overwrite_segment_index,
            writes_from_the_segment_on: 1,
            withheld_among_them: &[0],
        },
        SecondCrashPointDraw::ExactlyOneSecondCrashState {
            segment_index: segment_of_the_first_record,
            writes_from_the_segment_on: last_rotation + 1 - first_write_of_the_segment,
            withheld_among_them: withheld_among_them.leak(),
        },
        None,
        given_states_engine_for("crash_injection_unit_tests").engine(),
    );
    let tally = &injection.tally;
    assert_eq!(
        (
            tally.crash_points,
            tally.writable_mounts_after_the_crash_succeeded,
            tally.crash_points_after_a_step_reading_under_injected_faults
        ),
        (1, 1, 0),
        "前提：只摆写死的那一个第一次崩溃态、它上面的可写挂载做成、这段历史没有带读故障的步：{}",
        tally.render()
    );
    assert_eq!(
        (
            tally.second_crash_points,
            tally
                .second_crash_points_by_phase
                .get(&WritableMountPhase::RowPublish)
                .copied()
        ),
        (1, Some(1)),
        "前提：只摆写死的那一个二次崩溃，崩在写行里：{}",
        tally.render()
    );
    assert_eq!(
            (
                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
            ),
            (1, 1),
            "二次崩溃镜像上再挂以判据 N-配置 拒、崩溃态没注入读故障而判据复算为真：判过一次、不合法一次：{}",
            tally.render()
        );
    let remount_findings: Vec<String> = injection
        .new_findings
        .iter()
        .filter(|finding| {
            matches!(
                finding.stage,
                CrashStateStage::RemountAfterASecondCrashInsideTheWritableMount(_)
            )
        })
        .map(|finding| {
            finding
                .observation
                .model_disagreement
                .as_ref()
                .unwrap_or_else(|| panic!("拒得不合法记成模型对不上：{}", finding.render()))
                .implementation_answer
                .clone()
        })
        .collect();
    let [remount_finding] = remount_findings.as_slice() else {
        panic!("第三截再挂那一判要恰好一条新发现：{remount_findings:?}");
    };
    let branch_of_the_second_crash_version = format!(
            "{:?}",
            WitnessedCounterBranchThatHolds::NeitherTheLastRecordOfTheSelectedVersionNorTheRecordAtTheWitnessedCounterReadable
        );
    let branch_of_the_first_crash_version = format!(
        "{:?}",
        WitnessedCounterBranchThatHolds::WitnessedCounterAboveTheLastRecordOfTheSelectedVersion
    );
    assert!(
            remount_finding.contains(&branch_of_the_second_crash_version),
            "所选那一版是写行那一版：它的末条记录与计数器等于 c_见证 的记录都读不出，判官要列 {branch_of_the_second_crash_version}：{remount_finding}"
        );
    assert!(
            !remount_finding.contains(&branch_of_the_first_crash_version),
            "所选那一版不是第一次崩溃落的起点那一版（那一版的末条记录读得出），不许列 {branch_of_the_first_crash_version}：{remount_finding}"
        );
}

/// 第三截「所选那一版」的接线在计数层分得开的靶向态（攻方改法 K3-a，判决 `research/prompts/m3-formal-code-b21-r1-main-verification.md`
/// 第二节 K3 与第三节第 2 条；攻方探针 `research/prompts/m3-formal-code-b21-r1-opus-model/probe/opus_probe.rs` 的 `k3_sweep`，
/// 产物 `…/out/k3-sweep.txt` 那一行 `first(last=80,withheld=[0, 1, 4]) second(last=19,withheld=[0, 1, 3])`）。与上两条同一段历史。
/// 第一次崩溃从第一个文件那次发布的第一份记录起，摆到覆盖写那次的根槽为止：第一个文件那次的两份记录与它根槽之后第二槽系统配置轮换扣下，
/// 别的（第一个文件那次的根槽、第一槽轮换，覆盖写那次的单元、记录与根槽）都落了——越过记录与根槽之间、根槽与轮换之间的屏障。
/// 第二截那次可写挂载的写行那次发布上摆一个二次崩溃：写行那次的两份记录与根槽之后第一槽轮换扣下，根槽落了，第二槽轮换没摆到。
/// 第三截再挂以判据 N-配置 拒；所选那一版取二次崩溃镜像上只读恢复落到的那一版（写行那一版），它的末条记录与计数器等于 c_见证 的记录都读不出，
/// 判据为真：判过一次、不合法一次。判别力：调用点把所选那一版交成第一次崩溃那一版（`crates/mutations.tsv` 一行，原文与上一条那一行相同）时
/// 判据复算为假、计数变 (1, 0)，红在最后一条断言（攻方在副本上量过，`research/prompts/m3-formal-code-b21-r1-opus-model/out/k3-sweep.txt`）。
/// 上一条那一态在计数层分不开这两种接线，只在判官列出的那一支上分得开；这一态在计数层就分得开。
// crash-case-check:not-a-crash-case 靶向用例只摆写死的一个第一次崩溃态与一个二次崩溃（ExactlyOneCrashState、ExactlyOneSecondCrashState），不枚举
#[test]
fn second_crash_losing_the_records_of_the_row_publish_after_a_first_crash_losing_the_records_of_the_first_file_counts_one_illegal_witness_refusal(
) {
    let (history, execution) = history_of_the_first_file_and_one_overwrite();
    let (writes, segments, first_write_of_the_overwrite) =
        writes_segments_and_first_write_of_the_overwrite(&history, execution);
    let root_of_the_overwrite = (first_write_of_the_overwrite..writes.len())
        .find(|write| writes[*write].kind == StepKind::RootRecordFua)
        .expect("前提：覆盖写那次发布写了根槽");
    let root_of_the_first_file = (0..first_write_of_the_overwrite)
        .rev()
        .find(|write| writes[*write].kind == StepKind::RootRecordFua)
        .expect("前提：第一个文件那次发布写了根槽");
    let root_before_the_first_file = (0..root_of_the_first_file)
        .rev()
        .find(|write| writes[*write].kind == StepKind::RootRecordFua)
        .expect("前提：第一个文件之前那次发布写了根槽");
    let records_of_the_first_file: Vec<usize> = (root_before_the_first_file + 1
        ..root_of_the_first_file)
        .filter(|write| writes[*write].kind == StepKind::JournalRecord)
        .collect();
    let rotation_after_the_first_file: Vec<usize> = (root_of_the_first_file + 1
        ..first_write_of_the_overwrite)
        .filter(|write| writes[*write].kind == StepKind::SystemConfigurationSlot)
        .collect();
    assert_eq!(
        (
            records_of_the_first_file.len(),
            rotation_after_the_first_file.len()
        ),
        (2, 2),
        "前提：第一个文件那次发布在根槽之前每块盘写一份记录，根槽之后每块盘轮换一槽系统配置"
    );
    let first_crash_segment_index =
        segment_containing_the_write(&segments, records_of_the_first_file[0]);
    let first_write_of_the_first_crash_segment = segments[first_crash_segment_index][0];
    let withheld_in_the_first_crash: Vec<usize> = records_of_the_first_file
        .iter()
        .chain(std::iter::once(&rotation_after_the_first_file[1]))
        .map(|write| write - first_write_of_the_first_crash_segment)
        .collect();
    let writes_of_the_first_crash =
        root_of_the_overwrite + 1 - first_write_of_the_first_crash_segment;
    let mut first_crash_image = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    first_crash_image.apply_writes(&writes[..first_write_of_the_first_crash_segment]);
    let persisted_writes_of_the_first_crash: Vec<RetainedWrite> = (0..writes_of_the_first_crash)
        .filter(|within_the_state| !withheld_in_the_first_crash.contains(within_the_state))
        .map(|within_the_state| {
            writes[first_write_of_the_first_crash_segment + within_the_state].clone()
        })
        .collect();
    first_crash_image.apply_writes(&persisted_writes_of_the_first_crash);
    // 第二截那次挂载写出的写表：与注入里那一次同一份盘面、同一份参数；挂载之后那一次发布写的内容随崩溃点变，不在挂载写表里。
    let mount_run = mount_and_publish_once_on_the_crash_image(
        &first_crash_image,
        &execution.device_width.parameters(),
        execution,
        history.seed,
        &CrashPoint {
            segment_index: first_crash_segment_index,
            persisted_within_the_segment: (0..writes_of_the_first_crash)
                .map(|within_the_state| !withheld_in_the_first_crash.contains(&within_the_state))
                .collect(),
            step: StepPosition::Operation(0),
            operation_kind: Some(HistoryOperationKind::PublishOverwrite),
        },
    );
    assert!(
        matches!(mount_run.outcome, WritableMountOutcome::Published),
        "前提：第一次崩溃镜像上的可写挂载与之后那一次发布都做成：{:?}",
        mount_run.outcome
    );
    let (mount_writes, mount_segments, _mount_stream_indexes) =
        writes_and_segments_with_stream_indexes(
            &mount_run.mount_operations,
            &execution.device_width.fixed_geometry(),
        );
    let phase_of_each_write = writable_mount_phase_of_each_write(
        &mount_writes,
        mount_run.publishes_before_the_deletion_queue_continuation,
        mount_run.publishes_before_the_instance_table_sweep,
    );
    let root_of_the_row_publish = (0..mount_writes.len())
        .find(|write| {
            phase_of_each_write[*write] == WritableMountPhase::RowPublish
                && mount_writes[*write].kind == StepKind::RootRecordFua
        })
        .expect("前提：写行那次发布写了根槽");
    let records_of_the_row_publish: Vec<usize> = (0..root_of_the_row_publish)
        .filter(|write| {
            phase_of_each_write[*write] == WritableMountPhase::RowPublish
                && mount_writes[*write].kind == StepKind::JournalRecord
        })
        .collect();
    let rotation_after_the_root: Vec<usize> = (root_of_the_row_publish + 1..mount_writes.len())
        .take_while(|write| mount_writes[*write].kind == StepKind::SystemConfigurationSlot)
        .collect();
    assert_eq!(
        (
            records_of_the_row_publish.len(),
            rotation_after_the_root.len()
        ),
        (2, 2),
        "前提：写行那次发布在根槽之前每块盘写一份记录，根槽 FUA 之后紧跟着每块盘轮换一槽系统配置"
    );
    let second_crash_segment_index =
        segment_containing_the_write(&mount_segments, records_of_the_row_publish[0]);
    let first_write_of_the_second_crash_segment = mount_segments[second_crash_segment_index][0];
    let withheld_in_the_second_crash: Vec<usize> = records_of_the_row_publish
        .iter()
        .chain(std::iter::once(&rotation_after_the_root[0]))
        .map(|write| write - first_write_of_the_second_crash_segment)
        .collect();
    // 前提：两次崩溃之后只读恢复落的版本不同（同一份盘面在这里照摆法叠一遍）。
    let mut second_crash_image = first_crash_image.clone();
    second_crash_image.apply_writes(&mount_writes[..first_write_of_the_second_crash_segment]);
    let persisted_writes_of_the_second_crash: Vec<RetainedWrite> =
        (first_write_of_the_second_crash_segment..=rotation_after_the_root[0])
            .filter(|write| {
                !withheld_in_the_second_crash
                    .contains(&(write - first_write_of_the_second_crash_segment))
            })
            .map(|write| mount_writes[write].clone())
            .collect();
    second_crash_image.apply_writes(&persisted_writes_of_the_second_crash);
    assert_ne!(
        recover(&first_crash_image, JournalPolicy::Consult).effective_root,
        recover(&second_crash_image, JournalPolicy::Consult).effective_root,
        "前提：第一次崩溃落在覆盖写那一版、二次崩溃落在写行那一版，两次只读恢复落的版本不同"
    );
    let injection = inject_crashes_into_history_drawing_second_crashes(
        &history,
        execution,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: first_crash_segment_index,
            writes_from_the_segment_on: writes_of_the_first_crash,
            withheld_among_them: withheld_in_the_first_crash.leak(),
        },
        SecondCrashPointDraw::ExactlyOneSecondCrashState {
            segment_index: second_crash_segment_index,
            writes_from_the_segment_on: rotation_after_the_root[0] + 1
                - first_write_of_the_second_crash_segment,
            withheld_among_them: withheld_in_the_second_crash.leak(),
        },
        None,
        given_states_engine_for("crash_injection_unit_tests").engine(),
    );
    let tally = &injection.tally;
    assert_eq!(
        (
            tally.crash_points,
            tally.writable_mounts_after_the_crash_succeeded,
            tally.crash_points_after_a_step_reading_under_injected_faults
        ),
        (1, 1, 0),
        "前提：只摆写死的那一个第一次崩溃态、它上面的可写挂载做成、这段历史没有带读故障的步：{}",
        tally.render()
    );
    assert_eq!(
        (
            tally.second_crash_points,
            tally
                .second_crash_points_by_phase
                .get(&WritableMountPhase::RowPublish)
                .copied()
        ),
        (1, Some(1)),
        "前提：只摆写死的那一个二次崩溃，崩在写行里：{}",
        tally.render()
    );
    assert_eq!(
            (
                tally.remount_refusals_for_a_newer_publish_witnessed_by_system_configuration,
                tally.remount_refusals_with_the_witness_criterion_true_without_read_faults
            ),
            (1, 1),
            "二次崩溃镜像上再挂以判据 N-配置 拒，对着二次崩溃镜像上只读恢复落到的那一版复算判据为真：判过一次、不合法一次：{}",
            tally.render()
        );
}

//! J6Qn 按 I-8.12 拆行（登记 r3 5.6、5.7、第七节乙类）：拆行的钉值、拆出的行与判决行，连同建表比较、镜像、J6Qn、J7a、C8 一族与卸载记号的对照读数。

use super::*;

// ── J6Qn 按 I-8.12 拆行（登记 r3 5.6、5.7、第七节乙类）

/// 对抗一族每个 (域, n) 上，表满足 / 不满足 I-8.12 的态数（登记 r3 第七节乙类「Q3 的分类计数」，`anchors_r3_i812.py` 动态规划与暴力枚举互核）：
/// (域, n, 成立, 不成立)。三个域各 n 的态数之和等于原登记 5.7 的 57 585 320、788 838、6 139 590。
const PINNED_TXG_ORDER_STATE_COUNTS: [(Domain, usize, u64, u64); 10] = [
    (Domain::AdversarialBigRing, 1, 146, 0),
    (Domain::AdversarialBigRing, 2, 9_122, 1_536),
    (Domain::AdversarialBigRing, 3, 523_570, 254_464),
    (Domain::AdversarialBigRing, 4, 28_459_330, 28_337_152),
    (Domain::AdversarialSmallRingSix, 1, 146, 0),
    (Domain::AdversarialSmallRingSix, 2, 9_122, 1_536),
    (Domain::AdversarialSmallRingSix, 3, 523_570, 254_464),
    (Domain::AdversarialBackChain, 1, 290, 0),
    (Domain::AdversarialBackChain, 2, 35_906, 6_144),
    (Domain::AdversarialBackChain, 3, 4_079_970, 2_017_280),
];

/// KF3 假结局（第一次返回所选根、第二次报「不止一条末条标志」）在对照界里必须读到的四格（登记 r3 第七节乙类，`anchors_r3_i812_premise.py`）：
/// (域, n, 前提真·I-8.12 成立, 前提真·不成立, 前提假·成立, 前提假·不成立)。每格的违例数与 judged 都等于它（对照界内第一次调用都「返回」）。
pub(super) const PINNED_KF3_CELLS: [(Domain, usize, [u64; 4]); 7] = [
    (Domain::AdversarialBigRing, 1, [146, 0, 0, 0]),
    (Domain::AdversarialBigRing, 2, [9_122, 1_536, 0, 0]),
    (Domain::AdversarialSmallRingSix, 1, [146, 0, 0, 0]),
    (Domain::AdversarialSmallRingSix, 2, [9_122, 1_536, 0, 0]),
    (
        Domain::AdversarialSmallRingSix,
        3,
        [490_802, 221_696, 32_768, 32_768],
    ),
    (Domain::AdversarialBackChain, 1, [290, 0, 0, 0]),
    (Domain::AdversarialBackChain, 2, [35_906, 6_144, 0, 0]),
];

/// 四格的次序（前提真·成立、前提真·不成立、前提假·成立、前提假·不成立），与 [`PINNED_KF3_CELLS`] 里四个数的次序一致。
const TXG_ORDER_SPLIT_CELLS: [(bool, bool); 4] =
    [(true, true), (true, false), (false, true), (false, false)];

fn pinned_txg_order_state_counts(domain: Domain, size: usize) -> Option<(u64, u64)> {
    PINNED_TXG_ORDER_STATE_COUNTS
        .iter()
        .find(|(pinned_domain, pinned_size, _, _)| *pinned_domain == domain && *pinned_size == size)
        .map(|(_, _, holds, fails)| (*holds, *fails))
}

fn pinned_kf3_cells(domain: Domain, size: usize) -> Option<[u64; 4]> {
    PINNED_KF3_CELLS
        .iter()
        .find(|(pinned_domain, pinned_size, _)| *pinned_domain == domain && *pinned_size == size)
        .map(|(_, _, cells)| *cells)
}

/// 对抗一族这一个 (域, n) 上，表满足 / 不满足 I-8.12 的态数等于登记的分类计数；不符 ⇒ 作废条款 TV4（装置数错了）。
pub(super) fn assert_the_pinned_txg_order_state_counts(domain: Domain, size: usize, tally: &Tally) {
    let (pinned_holding, pinned_failing) = pinned_txg_order_state_counts(domain, size).unwrap_or_else(|| {
        panic!(
            "作废条款 TV4：{} 域 n = {size} 没有登记 r3 第七节乙类的 I-8.12 分类计数，这一格没被钉住",
            domain.label()
        )
    });
    assert_eq!(
        (
            tally.count_of(Counter::TablesKeepTheTxgOrder),
            tally.count_of(Counter::TablesBreakTheTxgOrder)
        ),
        (pinned_holding, pinned_failing),
        "作废条款 TV4：{} 域 n = {size} 的 I-8.12 成立 / 不成立态数与登记 r3 第七节乙类不等",
        domain.label()
    );
    assert_eq!(
        tally.count_of(Counter::TablesKeepTheTxgOrder)
            + tally.count_of(Counter::TablesBreakTheTxgOrder),
        tally.count_of(Counter::States),
        "作废条款 TV4：{} 域 n = {size} 的 I-8.12 成立与不成立两类态数之和不等于状态数",
        domain.label()
    );
}

/// 拆开的四行对回未拆的两行（登记 r3 作废条款 TV6）：前提真那两格的违例数之和与 judged 之和各等于 `J6Qn_premise_holds` 一行的，前提假同理。
/// 对得上交回 `None`，对不上交回一句说明。
pub(super) fn the_split_rows_differ_from_the_unsplit_rows(tally: &Tally) -> Option<String> {
    let sum_of_cells = |premise_holds: bool, pick: fn((Counter, Counter)) -> Counter| {
        TXG_ORDER_SPLIT_CELLS
            .iter()
            .filter(|(cell_premise_holds, _)| *cell_premise_holds == premise_holds)
            .map(|(cell_premise_holds, txg_order_holds)| {
                tally.count_of(pick(j6qn_cell_counters(
                    *cell_premise_holds,
                    *txg_order_holds,
                )))
            })
            .sum::<u64>()
    };
    let judged_of = |cells: (Counter, Counter)| cells.0;
    let violations_of = |cells: (Counter, Counter)| cells.1;
    let split_and_unsplit = [
        (
            "前提真的违例数",
            sum_of_cells(true, violations_of),
            tally.count_of(Counter::FixedPointQnPremiseHoldsViolations),
        ),
        (
            "前提真的 judged",
            sum_of_cells(true, judged_of),
            tally.count_of(Counter::FixedPointQnPremiseHoldsJudged),
        ),
        (
            "前提假的违例数",
            sum_of_cells(false, violations_of),
            tally.count_of(Counter::FixedPointQnPremiseFailsViolations),
        ),
        (
            "前提假的 judged",
            sum_of_cells(false, judged_of),
            tally.count_of(Counter::FixedPointQnPremiseFailsJudged),
        ),
    ];
    let differing: Vec<String> = split_and_unsplit
        .iter()
        .filter(|(_, split_sum, unsplit)| split_sum != unsplit)
        .map(|(what, split_sum, unsplit)| {
            format!("{what}：拆开之后之和 {split_sum}、未拆那一行 {unsplit}")
        })
        .collect();
    (!differing.is_empty()).then(|| differing.join("；"))
}

/// 对抗一族一个 (域, n) 的 I-8.12 拆行（登记 r3 5.6）：四格各一行（judged、违例数、违例里最先的见证），另一行报全部状态里表成立 / 不成立的态数。
/// 不是 r3 档、或不是对抗一族，一行都不出。
pub(super) fn rows_of_the_txg_order_split(
    run_profile: RunProfile,
    domain: Domain,
    place: &str,
    tally: &Tally,
) -> Vec<String> {
    if !run_profile.splits_j6qn_by_the_txg_order() || domain.family() != Family::Adversarial {
        return Vec::new();
    }
    let mut rows: Vec<String> = TXG_ORDER_SPLIT_CELLS
        .iter()
        .map(|(premise_holds, txg_order_holds)| {
            let (judged_counter, violations_counter) =
                j6qn_cell_counters(*premise_holds, *txg_order_holds);
            format!(
                "name=violations {place} criterion={} violations={} judged={} witness={}",
                violations_counter.label(),
                tally.count_of(violations_counter),
                tally.count_of(judged_counter),
                witness_field(tally, violations_counter)
            )
        })
        .collect();
    rows.push(format!(
        "name=i812_states {place} i812_holds={} i812_fails={} states={}",
        tally.count_of(Counter::TablesKeepTheTxgOrder),
        tally.count_of(Counter::TablesBreakTheTxgOrder),
        tally.count_of(Counter::States)
    ));
    rows
}

/// 对抗一族一个域在全部 n 上的 I-8.12 拆行合计（登记 r3 5.6「每个域收尾的 criteria=r2 判决行后面另起一行 criteria=r3_i812」）。
pub(super) fn verdict_row_of_the_txg_order_split(place: &str, total: &Tally) -> String {
    let cell = |premise_holds: bool, txg_order_holds: bool| {
        let (judged_counter, violations_counter) =
            j6qn_cell_counters(premise_holds, txg_order_holds);
        (
            total.count_of(violations_counter),
            total.count_of(judged_counter),
        )
    };
    let (holds_holds_violations, holds_holds_judged) = cell(true, true);
    let (holds_fails_violations, holds_fails_judged) = cell(true, false);
    let (fails_holds_violations, fails_holds_judged) = cell(false, true);
    let (fails_fails_violations, fails_fails_judged) = cell(false, false);
    format!(
        "name=verdict {place} criteria=r3_i812 j6qn_premise_holds_i812_holds_violations={holds_holds_violations} j6qn_premise_holds_i812_holds_judged={holds_holds_judged} j6qn_premise_holds_i812_fails_violations={holds_fails_violations} j6qn_premise_holds_i812_fails_judged={holds_fails_judged} j6qn_premise_fails_i812_holds_violations={fails_holds_violations} j6qn_premise_fails_i812_holds_judged={fails_holds_judged} j6qn_premise_fails_i812_fails_violations={fails_fails_violations} j6qn_premise_fails_i812_fails_judged={fails_fails_judged} i812_holds={} i812_fails={} split_rows_equal_the_unsplit_rows={}",
        total.count_of(Counter::TablesKeepTheTxgOrder),
        total.count_of(Counter::TablesBreakTheTxgOrder),
        the_split_rows_differ_from_the_unsplit_rows(total).is_none()
    )
}

/// KF3 的对照（登记 r3 5.7、第七节乙类）：对抗一族对照界（A n ≤ 2、A6 n ≤ 3、AC n ≤ 2）每个 n 上，第一次返回所选根、第二次报「不止一条末条标志」，
/// 四格各自的违例数与 judged 都等于钉的数；全部状态里表成立 / 不成立的态数也等于钉的分类计数。
pub(super) fn txg_order_split_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let criterion = Criterion::QnFixedPointSplitByTxgOrder;
    let control =
        ControlKind::FirstCallReturnsTheChosenRootAndSecondCallReportsMoreThanOneLastRecordFlag;
    let mut readings = Vec::new();
    for (domain, largest_size) in [
        (
            Domain::AdversarialBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
        ),
        (
            Domain::AdversarialSmallRingSix,
            CONTROL_LARGEST_SIZE_OF_THE_TXG_ORDER_SMALL_RING_DOMAIN,
        ),
        (
            Domain::AdversarialBackChain,
            CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
        ),
    ] {
        for size in domain.smallest_size()..=largest_size {
            let tally = tally_of_a_domain_size(
                domain,
                size,
                Implementation::Control(control),
                run_profile_of_a_control(control, run_profile),
                thread_count,
            );
            let pinned_cells = pinned_kf3_cells(domain, size)
                .expect("对照界里每个 (域, n) 都有登记 r3 第七节乙类 KF3 的四格");
            let (pinned_holding, pinned_failing) = pinned_txg_order_state_counts(domain, size)
                .expect("对照界里每个 (域, n) 都有登记 r3 第七节乙类的分类计数");
            let place = format!(
                "domain={},n={size},implementation=first_call_returns_the_chosen_root_second_call_reports_more_than_one_last_record_flag",
                domain.label()
            );
            for ((premise_holds, txg_order_holds), pinned) in
                TXG_ORDER_SPLIT_CELLS.iter().zip(pinned_cells)
            {
                let (judged_counter, violations_counter) =
                    j6qn_cell_counters(*premise_holds, *txg_order_holds);
                readings.push(reading_that_must_equal(
                    criterion,
                    format!("{place},counter={}", violations_counter.label()),
                    tally.count_of(violations_counter),
                    pinned,
                ));
                readings.push(reading_that_must_equal(
                    criterion,
                    format!("{place},counter={}", judged_counter.label()),
                    tally.count_of(judged_counter),
                    pinned,
                ));
            }
            readings.push(reading_that_must_equal(
                criterion,
                format!("{place},counter={}", Counter::TablesKeepTheTxgOrder.label()),
                tally.count_of(Counter::TablesKeepTheTxgOrder),
                pinned_holding,
            ));
            readings.push(reading_that_must_equal(
                criterion,
                format!(
                    "{place},counter={}",
                    Counter::TablesBreakTheTxgOrder.label()
                ),
                tally.count_of(Counter::TablesBreakTheTxgOrder),
                pinned_failing,
            ));
        }
    }
    readings
}

/// L 域 N ≤ 5 上直接建的表非空的态数（至少一条记录落了盘的态数）：登记 r2 第七节乙类「L 大环对照界 N ≤ 5：……至少一条记录落盘的态数 2 989」
/// （每段历史的 2^N 个落盘子集里去掉空集：1 + 9 + 63 + 405 + 2 511）。
const STATES_WITH_A_RECORD_ON_DISK_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN: u64 =
    2_989;

/// T-表的对照（登记 r2 5.5、第七节）：扫盘建表之后删掉键最大的那条记录、直接建的表不动。L 域 N ≤ 5 上 `tables_differ` 等于直接建的表非空的态数
/// （装置另数一遍，钉 2 989），`rows_differ` 大于 0。
pub(super) fn table_comparison_control_readings_on_tables(
    criterion: Criterion,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ControlKind::ScanTableLosesItsLargestKeyRecord,
        run_profile,
        thread_count,
    );
    let place = |counter: Counter| {
        control_place(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            "scan_table_loses_its_largest_key_record",
            counter,
        )
    };
    vec![
        reading_that_must_equal(
            criterion,
            place(Counter::TablesDiffer),
            tally.count_of(Counter::TablesDiffer),
            tally.count_of(Counter::DirectTableHoldsARecord),
        ),
        reading_that_must_equal(
            criterion,
            place(Counter::DirectTableHoldsARecord),
            tally.count_of(Counter::DirectTableHoldsARecord),
            STATES_WITH_A_RECORD_ON_DISK_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ),
        reading_that_must_be_positive(
            criterion,
            place(Counter::RowsDiffer),
            tally.count_of(Counter::RowsDiffer),
        ),
    ]
}

/// T-行的对照（登记 r2 5.5、第七节）：扫盘建表那一遍的实现结局换成 `Panicked`、表不动。L 域 N ≤ 5 上 `rows_differ` 等于状态数（3 110）、
/// `tables_differ` 是 0（两格分得开）。
pub(super) fn table_comparison_control_readings_on_rows(
    criterion: Criterion,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ControlKind::ScanPassOutcomeBecomesAPanic,
        run_profile,
        thread_count,
    );
    let place = |counter: Counter| {
        control_place(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            "scan_pass_outcome_becomes_a_panic",
            counter,
        )
    };
    vec![
        reading_that_must_equal(
            criterion,
            place(Counter::RowsDiffer),
            tally.count_of(Counter::RowsDiffer),
            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ),
        reading_that_must_equal(
            criterion,
            place(Counter::States),
            tally.count_of(Counter::States),
            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ),
        reading_that_must_equal(
            criterion,
            place(Counter::TablesDiffer),
            tally.count_of(Counter::TablesDiffer),
            0,
        ),
    ]
}

/// 多盘三个域与各自对照的界、j\* − c ≥ 1 的态数（登记 r2 第七节）。
const MIRRORED_DOMAINS_WITH_THE_CONTROL_BOUND: [(Domain, usize, u64); 3] = [
    (
        Domain::MirroredBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_BIG_RING_DOMAIN,
        MIRRORED_BIG_RING_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND,
    ),
    (
        Domain::MirroredSmallRingSix,
        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_SMALL_RING_SIX_DOMAIN,
        MIRRORED_SMALL_RING_SIX_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND,
    ),
    (
        Domain::MirroredMonotoneBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN,
        MIRRORED_MONOTONE_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND,
    ),
];

fn control_place(
    domain: Domain,
    largest_size: usize,
    implementation_label: &str,
    counter: Counter,
) -> String {
    format!(
        "domain={},max_n={largest_size},implementation={implementation_label},counter={}",
        domain.label(),
        counter.label()
    )
}

/// 多盘「任一份在即在」（登记 r2 5.5、第七节）：理想恢复但 j\* 只数 0 号盘那一份。M 域 N ≤ 3 的 J4 = 100、M6 域 N ≤ 4 = 996，
/// L 域 N ≤ 5（单盘，0 号盘那一份就是全部）= 0。
pub(super) fn any_copy_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let control = ControlKind::IdealCountingOnlyTheFirstDeviceCopies;
    [
        (
            Domain::MirroredBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_BIG_RING_DOMAIN,
            MIRRORED_BIG_RING_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION,
        ),
        (
            Domain::MirroredSmallRingSix,
            CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_SMALL_RING_SIX_DOMAIN,
            MIRRORED_SMALL_RING_SIX_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION,
        ),
        (
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            0,
        ),
    ]
    .into_iter()
    .map(|(domain, largest_size, expected)| {
        let tally = control_tally(domain, largest_size, control, run_profile, thread_count);
        reading_that_must_equal(
            Criterion::MultiDiskAnyCopyIsOnDisk,
            control_place(domain, largest_size, "ideal_counting_only_the_first_device_copies", Counter::FarthestVersionViolations),
            tally.count_of(Counter::FarthestVersionViolations),
            expected,
        )
    })
    .collect()
}

/// 原登记 J1 那条对照（施加过就返回时间线之外的根）在多盘三个域上：J1 违例数与 j\* − c ≥ 1 的态数都等于第七节的值。
pub(super) fn mirrored_returned_root_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let mut readings = Vec::new();
    for (domain, largest_size, farthest_at_least_one) in MIRRORED_DOMAINS_WITH_THE_CONTROL_BOUND {
        let tally = control_tally(
            domain,
            largest_size,
            ControlKind::OffTimelineRootWheneverApplied,
            run_profile,
            thread_count,
        );
        for counter in [Counter::ReturnedRootViolations, Counter::FarthestAtLeastOne] {
            readings.push(reading_that_must_equal(
                Criterion::MultiDiskReturnedRoot,
                control_place(
                    domain,
                    largest_size,
                    "off_timeline_root_whenever_applied",
                    counter,
                ),
                tally.count_of(counter),
                farthest_at_least_one,
            ));
        }
    }
    readings
}

/// 原登记 J4 那条对照（永远返回所选根）在多盘三个域上：J4 违例数等于第七节的值，J1、J2、J3 是 0。
pub(super) fn mirrored_farthest_version_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let mut readings = Vec::new();
    for (domain, largest_size, farthest_at_least_one) in MIRRORED_DOMAINS_WITH_THE_CONTROL_BOUND {
        let tally = control_tally(
            domain,
            largest_size,
            ControlKind::AlwaysReturnsTheChosenRoot,
            run_profile,
            thread_count,
        );
        for (counter, expected) in [
            (Counter::FarthestVersionViolations, farthest_at_least_one),
            (Counter::ReturnedRootViolations, 0),
            (Counter::RootPlusDiskViolations, 0),
            (Counter::InFlightLimitViolations, 0),
        ] {
            readings.push(reading_that_must_equal(
                Criterion::MultiDiskFarthestVersion,
                control_place(
                    domain,
                    largest_size,
                    "always_returns_the_chosen_root",
                    counter,
                ),
                tally.count_of(counter),
                expected,
            ));
        }
    }
    readings
}

/// MC 宽对上 J5a、J5b 的对照（登记 r2 第十二节修订 3 补的；原登记 5.4 J5 那条假结局搬到多盘）：记录每条都有一份在盘上时返回所选根，
/// 其余照理想恢复；MC 域 Nc ≤ 2 宽对上今天那两列都大于 0。
pub(super) fn mirrored_monotone_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let tally = control_tally(
        Domain::MirroredMonotoneBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN,
        ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk,
        run_profile,
        thread_count,
    );
    [PairColumn::TodayAppliedSetViolations, PairColumn::TodayLandedVersionViolations]
        .into_iter()
        .map(|column| {
            reading_that_must_be_positive(
                Criterion::MultiDiskAppliedSetAndLandedVersion,
                format!(
                    "domain=MC,max_n={CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN},implementation=returns_the_chosen_root_when_every_record_is_on_disk,set=wide,column={}",
                    column.label()
                ),
                tally.pair(PairSet::Wide, column),
            )
        })
        .collect()
}

/// J6Qn 的三条对照（登记 r2 5.5；第七节把它们钉成等式：L 域 N ≤ 5 两条各 3 110，E1 全界原 J6 70、J6Qn 0）。
pub(super) fn j6qn_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let criterion = Criterion::QnFixedPoint;
    let sum_of_both_premises = |tally: &Tally, premise_holds: Counter, premise_fails: Counter| {
        tally.count_of(premise_holds) + tally.count_of(premise_fails)
    };
    let mut readings = Vec::new();
    for (control, implementation_label) in [
        (
            ControlKind::SecondCallReportsMoreThanOneLastRecordFlag,
            "second_call_reports_more_than_one_last_record_flag",
        ),
        (
            ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor,
            "second_call_reports_chosen_root_txg_has_no_successor",
        ),
    ] {
        let tally = control_tally(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            control,
            run_profile,
            thread_count,
        );
        let place = format!(
            "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation={implementation_label}"
        );
        readings.push(reading_that_must_equal(
            criterion,
            format!("{place},counter=J6Qn_premise_holds_plus_J6Qn_premise_fails"),
            sum_of_both_premises(
                &tally,
                Counter::FixedPointQnPremiseHoldsViolations,
                Counter::FixedPointQnPremiseFailsViolations,
            ),
            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ));
        readings.push(reading_that_must_equal(
            criterion,
            format!("{place},counter=j6qn_premise_holds_judged_plus_j6qn_premise_fails_judged"),
            sum_of_both_premises(
                &tally,
                Counter::FixedPointQnPremiseHoldsJudged,
                Counter::FixedPointQnPremiseFailsJudged,
            ),
            STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
        ));
        if control == ControlKind::SecondCallReportsTheChosenRootTxgHasNoSuccessor {
            readings.push(reading_that_must_equal(
                criterion,
                format!("{place},counter=J6_premise_holds_plus_J6_premise_fails"),
                sum_of_both_premises(
                    &tally,
                    Counter::FixedPointPremiseHoldsViolations,
                    Counter::FixedPointPremiseFailsViolations,
                ),
                STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN,
            ));
        }
    }
    let e1_largest_size = BoundsTier::SegmentOne
        .largest_size(Domain::TxgOneBelowTheLargest)
        .expect("第一段跑 E1");
    let tally = control_tally(
        Domain::TxgOneBelowTheLargest,
        e1_largest_size,
        ControlKind::IdealFirstCallThenTodaysSecondCall,
        run_profile,
        thread_count,
    );
    let place = format!(
        "domain=E1,max_n={e1_largest_size},implementation=ideal_first_call_then_todays_second_call"
    );
    readings.push(reading_that_must_equal(
        criterion,
        format!("{place},counter=J6_premise_holds_plus_J6_premise_fails"),
        sum_of_both_premises(
            &tally,
            Counter::FixedPointPremiseHoldsViolations,
            Counter::FixedPointPremiseFailsViolations,
        ),
        E1_STATES_WHOSE_FARTHEST_VERSION_TXG_IS_THE_LARGEST,
    ));
    readings.push(reading_that_must_equal(
        criterion,
        format!("{place},counter=J6Qn_premise_holds_plus_J6Qn_premise_fails"),
        sum_of_both_premises(
            &tally,
            Counter::FixedPointQnPremiseHoldsViolations,
            Counter::FixedPointQnPremiseFailsViolations,
        ),
        0,
    ));
    readings
}

/// J7a 一格的对照（登记 r2 5.5、第七节）：理想恢复、施加过时把这个字段换成落到的那一版的。L 域 N ≤ 5 上这一格 = 762、另两格 = 0；
/// 同一个对照在 r1 档生成器上这一格 = 0（判别力自证：看得见是生成器换了的结果）。
pub(super) fn j7a_control_readings(
    criterion: Criterion,
    field: FieldOfTheRebuiltRoot,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let control = ControlKind::IdealWithAFieldOfTheLandedVersion(field);
    let place = format!(
        "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation=ideal_with_the_landed_versions_{}",
        field.label()
    );
    let tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        control,
        run_profile,
        thread_count,
    );
    let mut readings = Vec::new();
    for judged_field in FieldOfTheRebuiltRoot::ALL {
        let counter = judged_field.violations_counter();
        readings.push(reading_that_must_equal(
            criterion,
            format!("{place},counter={}", counter.label()),
            tally.count_of(counter),
            if judged_field == field {
                FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND
            } else {
                0
            },
        ));
    }
    let constant_fields_generator_tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        control,
        RunProfile::CONSTANT_FIELDS_GENERATOR_WITH_VARYING_FIELDS_CRITERIA,
        thread_count,
    );
    readings.push(reading_that_must_equal(
        criterion,
        format!(
            "{place},generator=r1,counter={}",
            field.violations_counter().label()
        ),
        constant_fields_generator_tally.count_of(field.violations_counter()),
        0,
    ));
    readings
}

/// r5 档跑的是哪一种卸载记号奇偶（J8 的对照只在 r5 档跑，r5 档的生成器随版本变）。
fn marker_parity_of_the_root_fields_in_records_profile(run_profile: RunProfile) -> MarkerParity {
    match run_profile.generator {
        FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(marker_parity) => marker_parity,
        FieldsBeyondTheProjectionGenerator::SameForEveryVersion => {
            unreachable!(
                "J8 的对照只在 r5 档跑（Criterion::is_judged_under），r5 档的生成器随版本变"
            )
        }
    }
}

/// 一条 J8 对照（登记 r5 5.4）在 L 域 N ≤ 5 上逐格钉值、在 r1 档那种生成器上逐格必须是 0、反奇偶另在 C 域 Nc ≤ 3 上逐格必须大于 0。
/// `pinned_cells` 是 (量, r5 生成器上的钉值)；`cells_that_must_be_zero_on_the_constant_fields_generator` 与 `cells_above_zero_on_the_monotone_domain_under_odd`
/// 各列一组量。
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数都是这一条对照要用的一样东西，凑不成一个带不变量的概念"
)]
fn c8_control_readings(
    criterion: Criterion,
    control_label: &str,
    control: ControlKind,
    pinned_cells: &[(Counter, u64)],
    cells_that_must_be_zero_on_the_constant_fields_generator: &[Counter],
    cells_above_zero_on_the_monotone_domain_under_odd: &[Counter],
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let marker_parity = marker_parity_of_the_root_fields_in_records_profile(run_profile);
    let place = format!(
        "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation={control_label}"
    );
    let big_ring_tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        control,
        run_profile,
        thread_count,
    );
    let mut readings: Vec<ControlReading> = pinned_cells
        .iter()
        .map(|(counter, pinned_value)| {
            reading_that_must_equal(
                criterion,
                format!(
                    "{place},counter={},marker_parity={}",
                    counter.label(),
                    marker_parity.label()
                ),
                big_ring_tally.count_of(*counter),
                *pinned_value,
            )
        })
        .collect();
    let constant_fields_generator_tally = control_tally(
        Domain::LegalBigRing,
        CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
        control,
        RunProfile::CONSTANT_FIELDS_GENERATOR_WITH_ROOT_FIELDS_IN_RECORDS_CRITERIA,
        thread_count,
    );
    for counter in cells_that_must_be_zero_on_the_constant_fields_generator {
        readings.push(reading_that_must_equal(
            criterion,
            format!("{place},generator=r1,counter={}", counter.label()),
            constant_fields_generator_tally.count_of(*counter),
            0,
        ));
    }
    match marker_parity {
        MarkerParity::EvenVersionsWrittenByTheUnmountSequence => {}
        MarkerParity::OddVersionsWrittenByTheUnmountSequence => {
            let monotone_tally = control_tally(
                Domain::MonotoneBigRing,
                CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
                control,
                run_profile,
                thread_count,
            );
            for counter in cells_above_zero_on_the_monotone_domain_under_odd {
                readings.push(reading_that_must_be_positive(
                    criterion,
                    format!(
                        "domain=C,max_n={CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN},implementation={control_label},counter={},marker_parity=odd",
                        counter.label()
                    ),
                    monotone_tally.count_of(*counter),
                ));
            }
        }
    }
    readings
}

/// C8a（登记 r5 5.4、第七节乙类）：理想恢复、施加过时三项都抄所选根的。L 域 N ≤ 5 上 J8a 实例表指针 762、分配记录树根 762
/// （第 j 版的两个槽与第 0 版的不同 ⟺ j ≥ 1）、卸载记号 620（第 j 版与第 0 版的记号不同 ⟺ j 为奇数，两种奇偶都如此），J8b、J8c 0。
pub(super) fn c8a_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    c8_control_readings(
        Criterion::J8aFieldsFromTheNewRootSegment,
        "C8a",
        ControlKind::IdealCopyingTheChosenRecordFieldsWhenApplied,
        &[
            (
                Counter::J8aInstanceTableViolations,
                FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
            ),
            (
                Counter::J8aAllocationRecordTreeRootViolations,
                FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
            ),
            (
                Counter::J8aUnmountMarkerViolations,
                FARTHEST_IS_ODD_STATES_WITHIN_THE_CONTROL_BOUND,
            ),
            (Counter::J8bViolations, 0),
            (Counter::J8cViolations, 0),
        ],
        &[
            Counter::J8aInstanceTableViolations,
            Counter::J8aAllocationRecordTreeRootViolations,
            Counter::J8aUnmountMarkerViolations,
        ],
        &[
            Counter::J8aInstanceTableViolations,
            Counter::J8aAllocationRecordTreeRootViolations,
        ],
        run_profile,
        thread_count,
    )
}

/// C8b（登记 r5 5.4、第七节乙类）：理想恢复、施加过时卸载记号一律写 `NotWritten…`。落到的那一版是 `Written…` ⟺ `even` 下 j 为偶数、
/// `odd` 下 j 为奇数，施加过的态里 j ≥ 1：L 域 N ≤ 5 上 `even` 142、`odd` 620。
pub(super) fn c8b_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let pinned_unmount_marker_violations =
        match marker_parity_of_the_root_fields_in_records_profile(run_profile) {
            MarkerParity::EvenVersionsWrittenByTheUnmountSequence => {
                FARTHEST_IS_EVEN_AND_AT_LEAST_TWO_STATES_WITHIN_THE_CONTROL_BOUND
            }
            MarkerParity::OddVersionsWrittenByTheUnmountSequence => {
                FARTHEST_IS_ODD_STATES_WITHIN_THE_CONTROL_BOUND
            }
        };
    c8_control_readings(
        Criterion::J8aUnmountMarkerFromTheNewRootSegment,
        "C8b",
        ControlKind::IdealWritingOneUnmountMarkerWhenApplied,
        &[(
            Counter::J8aUnmountMarkerViolations,
            pinned_unmount_marker_violations,
        )],
        &[Counter::J8aUnmountMarkerViolations],
        &[],
        run_profile,
        thread_count,
    )
}

/// C8c（登记 r5 5.4、第七节乙类）：理想恢复、施加过时 fsid 取落到的那一版 R_j 的。fsid 末字节取 j，第 j 版与第 0 版不同 ⟺ j ≥ 1：
/// L 域 N ≤ 5 上 J8b 762，J8a 三格 0。
pub(super) fn c8c_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    c8_control_readings(
        Criterion::J8bFilesystemIdentifierOfTheChosenRoot,
        "C8c",
        ControlKind::IdealWithAFieldOfTheLandedVersion(FieldOfTheRebuiltRoot::FilesystemIdentifier),
        &[
            (
                Counter::J8bViolations,
                FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
            ),
            (Counter::J8aInstanceTableViolations, 0),
            (Counter::J8aAllocationRecordTreeRootViolations, 0),
            (Counter::J8aUnmountMarkerViolations, 0),
        ],
        &[Counter::J8bViolations],
        &[Counter::J8bViolations],
        run_profile,
        thread_count,
    )
}

/// C8d（登记 r5 5.4、第七节乙类）：理想恢复、一条没施加时卸载记号写成 `NotWritten…`。一条没施加 ⟺ j* = 0；R_0 的记号 `even` 下
/// `Written…`、`odd` 下 `NotWritten…`：L 域 N ≤ 5 上 `even` 2 348、`odd` 0（`odd` 另在 C 域 Nc ≤ 3 上大于 0）。
pub(super) fn c8d_control_readings(
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let pinned_nothing_applied_violations =
        match marker_parity_of_the_root_fields_in_records_profile(run_profile) {
            MarkerParity::EvenVersionsWrittenByTheUnmountSequence => {
                FARTHEST_IS_THE_CHOSEN_VERSION_STATES_WITHIN_THE_CONTROL_BOUND
            }
            MarkerParity::OddVersionsWrittenByTheUnmountSequence => 0,
        };
    c8_control_readings(
        Criterion::J8cNothingAppliedReturnsTheChosenRoot,
        "C8d",
        ControlKind::IdealWritingNotWrittenWhenNothingIsApplied,
        &[(Counter::J8cViolations, pinned_nothing_applied_violations)],
        &[Counter::J8cViolations],
        &[Counter::J8cViolations],
        run_profile,
        thread_count,
    )
}

/// J7b、J7c 的对照（登记 r2 5.5、第七节、第八节奇偶的判别力自证）。偶数版带记号（R_0 是 `Written…`）：L 域 N ≤ 5 上
/// 等于 `count_when_the_first_version_is_written`，r1 档生成器上 0。奇数版带记号（R_0 是 `NotWritten…`）：L 域 0（假结局与期望同值），
/// C 域 Nc ≤ 3 大于 0（所选根是第几版取决于哪些根槽落了，两种记号的所选根都有）。
pub(super) fn unmount_marker_control_readings(
    criterion: Criterion,
    control: ControlKind,
    counter: Counter,
    count_when_the_first_version_is_written: u64,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let big_ring_place = format!(
        "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},implementation={},counter={}",
        criterion.label(),
        counter.label()
    );
    let big_ring_tally = |profile: RunProfile| {
        control_tally(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            control,
            profile,
            thread_count,
        )
        .count_of(counter)
    };
    let constant_fields_generator_reading = reading_that_must_equal(
        criterion,
        format!("{big_ring_place},generator=r1"),
        big_ring_tally(RunProfile::CONSTANT_FIELDS_GENERATOR_WITH_VARYING_FIELDS_CRITERIA),
        0,
    );
    match run_profile.generator {
        FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(
            MarkerParity::EvenVersionsWrittenByTheUnmountSequence,
        ) => vec![
            reading_that_must_equal(
                criterion,
                format!("{big_ring_place},marker_parity=even"),
                big_ring_tally(run_profile),
                count_when_the_first_version_is_written,
            ),
            constant_fields_generator_reading,
        ],
        FieldsBeyondTheProjectionGenerator::VaryWithTheVersion(
            MarkerParity::OddVersionsWrittenByTheUnmountSequence,
        ) => {
            let monotone_tally = control_tally(
                Domain::MonotoneBigRing,
                CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
                control,
                run_profile,
                thread_count,
            );
            vec![
                reading_that_must_equal(
                    criterion,
                    format!("{big_ring_place},marker_parity=odd"),
                    big_ring_tally(run_profile),
                    0,
                ),
                reading_that_must_be_positive(
                    criterion,
                    format!(
                        "domain=C,max_n={CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN},implementation={},counter={},marker_parity=odd",
                        criterion.label(),
                        counter.label()
                    ),
                    monotone_tally.count_of(counter),
                ),
                constant_fields_generator_reading,
            ]
        }
        FieldsBeyondTheProjectionGenerator::SameForEveryVersion => {
            unreachable!(
                "J7 的对照只在 r2 档跑（Criterion::is_judged_under），r2 档的生成器随版本变"
            )
        }
    }
}

/// 阳性对照那段循环：遍历判据表的每一行（只跑选中的那一族、这一档判的判据）。
pub(super) fn positive_control_readings(
    families: &[Family],
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let mut readings = Vec::new();
    for criterion in Criterion::ALL {
        if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {
            continue;
        }
        if !criterion.is_judged_under(run_profile) {
            continue;
        }
        readings.extend(control_readings_of(*criterion, run_profile, thread_count));
    }
    readings
}

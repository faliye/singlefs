//! 阳性对照（登记 5.4）：判据表每一行的假结局、对照的读数与读数要满足的钉值。

use super::*;

impl Criterion {
    pub(super) const fn family(self) -> Family {
        match self {
            Self::LegalDefined
            | Self::ReturnedRoot
            | Self::RootPlusDisk
            | Self::WithinTheInFlightLimit
            | Self::FarthestVersion
            | Self::J5aAppliedSet
            | Self::J5bLandedVersion
            | Self::J5aAndJ5bAreSeparable
            | Self::FixedPoint
            | Self::Ia1LargestTxgFails
            | Self::Ia2NoVerificationCounted
            | Self::QnFixedPoint
            | Self::J7aInstanceTable
            | Self::J7aAllocationRecordTreeRoot
            | Self::J7aFilesystemIdentifier
            | Self::J7bUnmountMarkerOfARebuiltRoot
            | Self::J7cUnmountMarkerWhenNothingIsApplied
            | Self::J8aFieldsFromTheNewRootSegment
            | Self::J8aUnmountMarkerFromTheNewRootSegment
            | Self::J8bFilesystemIdentifierOfTheChosenRoot
            | Self::J8cNothingAppliedReturnsTheChosenRoot
            | Self::TableComparisonSeesADifferentTable
            | Self::TableComparisonSeesADifferentRow => Family::LegalHistory,
            Self::AdversarialDefined
            | Self::AsWholeTable
            | Self::AsGapContinues
            | Self::AsGapStops
            | Self::AMutual
            | Self::AStructure
            | Self::ALimit
            | Self::QnFixedPointSplitByTxgOrder => Family::Adversarial,
            Self::MultiDiskAnyCopyIsOnDisk
            | Self::MultiDiskReturnedRoot
            | Self::MultiDiskFarthestVersion
            | Self::MultiDiskAppliedSetAndLandedVersion => Family::MultiDisk,
        }
    }

    /// T-表与 T-行判的是每个域上两种建表法的逐态比，对照只拿 L 域跑一遍；每个带 `--table both` 的跑法都带这两条对照，不管选的是哪一族。
    pub(super) const fn is_judged_in_every_family(self) -> bool {
        matches!(
            self,
            Self::TableComparisonSeesADifferentTable | Self::TableComparisonSeesADifferentRow
        )
    }

    /// 这一档判不判这条判据：J6Qn 与多盘四条在 r2、r3、r5 档判，J7 只在 r2、r3 档判（登记 r2 5.4），J8 只在 r5 档判（登记 r5 5.3），
    /// J6Qn 的 I-8.12 拆行在 r3、r5 档判（登记 r3 5.6），T-表与 T-行只在 `--table both` 时判，别的各档都判。
    pub(super) const fn is_judged_under(self, run_profile: RunProfile) -> bool {
        match self {
            Self::QnFixedPointSplitByTxgOrder => run_profile.splits_j6qn_by_the_txg_order(),
            Self::TableComparisonSeesADifferentTable | Self::TableComparisonSeesADifferentRow => {
                run_profile.compares_the_two_record_tables()
            }
            Self::J7aInstanceTable
            | Self::J7aAllocationRecordTreeRoot
            | Self::J7aFilesystemIdentifier
            | Self::J7bUnmountMarkerOfARebuiltRoot
            | Self::J7cUnmountMarkerWhenNothingIsApplied => run_profile.judges_copied_fields(),
            Self::J8aFieldsFromTheNewRootSegment
            | Self::J8aUnmountMarkerFromTheNewRootSegment
            | Self::J8bFilesystemIdentifierOfTheChosenRoot
            | Self::J8cNothingAppliedReturnsTheChosenRoot => {
                run_profile.judges_new_root_segment_fields()
            }
            Self::QnFixedPoint
            | Self::MultiDiskAnyCopyIsOnDisk
            | Self::MultiDiskReturnedRoot
            | Self::MultiDiskFarthestVersion
            | Self::MultiDiskAppliedSetAndLandedVersion => run_profile.judges_j6qn(),
            Self::LegalDefined
            | Self::ReturnedRoot
            | Self::RootPlusDisk
            | Self::WithinTheInFlightLimit
            | Self::FarthestVersion
            | Self::J5aAppliedSet
            | Self::J5bLandedVersion
            | Self::J5aAndJ5bAreSeparable
            | Self::FixedPoint
            | Self::Ia1LargestTxgFails
            | Self::Ia2NoVerificationCounted
            | Self::AdversarialDefined
            | Self::AsWholeTable
            | Self::AsGapContinues
            | Self::AsGapStops
            | Self::AMutual
            | Self::AStructure
            | Self::ALimit => true,
        }
    }
}

/// 阳性对照的一个读数：哪条判据、在哪里量的什么、量到多少、必须看到什么、看到了没有。
#[derive(Clone, Debug)]
pub(super) struct ControlReading {
    pub(super) criterion: Criterion,
    pub(super) measured_where: String,
    pub(super) value: u64,
    pub(super) must_see: String,
    pub(super) is_red_as_required: bool,
}

/// 对照的界取第一段各域的界减半，小环照原界（登记 5.4）。
pub(super) const CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN: usize = 5;
const CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN: usize = 3;
pub(super) const CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN: usize = 3;
pub(super) const CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN: usize = 2;
/// 对抗一族的 6 槽小环在 I-8.12 拆行的对照里扫到 n = 3（登记 r3 5.7：A6 n ≤ 3，那里前提为假的态才出现）。
pub(super) const CONTROL_LARGEST_SIZE_OF_THE_TXG_ORDER_SMALL_RING_DOMAIN: usize = 3;
/// 对照界里 L 域 j\* ≥ 1 的态数：登记第七节见证数 N = 1..5 的和，0 + 2 + 16 + 104 + 640。
pub(super) const FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 762;
/// 对照界里 L 域的态数：登记 r2 第七节乙类「L 大环对照界 N ≤ 5：态数 3 110」（2 + 12 + 72 + 432 + 2 592）。
pub(super) const STATES_WITHIN_THE_CONTROL_BOUND_OF_THE_BIG_RING_LEGAL_DOMAIN: u64 = 3_110;
/// 对照界里 L 域 j\* = 0 的态数：登记 r2 第七节乙类「j\* = 0 2 348」。
pub(super) const FARTHEST_IS_THE_CHOSEN_VERSION_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 2_348;
/// 对照界里 L 域 j\* 为奇数的态数：登记 r5 第七节乙类「j\* 为奇数 620」（逐 N 0 + 2 + 14 + 86 + 518，`anchors_r5.py`）。
pub(super) const FARTHEST_IS_ODD_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 620;
/// 对照界里 L 域 j\* 为偶数且 ≥ 2 的态数：登记 r5 第七节乙类「j\* 为偶数且 ≥ 2 的 142」（620 + 142 = 762）。
pub(super) const FARTHEST_IS_EVEN_AND_AT_LEAST_TWO_STATES_WITHIN_THE_CONTROL_BOUND: u64 = 142;
/// E1 全界（N ≤ 4）上 j\* = 1 的态数：登记 r2 第七节乙类「E1 上 j\* = 1 的态数 0、2、12、56，合 70」。
pub(super) const E1_STATES_WHOSE_FARTHEST_VERSION_TXG_IS_THE_LARGEST: u64 = 70;
/// 多盘对照的界（登记 r2 5.5 末段）：M 到 N = 3，M6 到 N = 4，MC 到 Nc = 2。
pub(super) const CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_BIG_RING_DOMAIN: usize = 3;
pub(super) const CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_SMALL_RING_SIX_DOMAIN: usize = 4;
pub(super) const CONTROL_LARGEST_SIZE_OF_THE_MIRRORED_MONOTONE_DOMAIN: usize = 2;
/// 登记 r2 第七节「多盘「任一份在即在」」：只数 0 号盘的 j\* 与任一份的 j\* 不等的态数，M 域 N ≤ 3 是 0 + 4 + 96，
/// M6 域 N ≤ 4 是 0 + 4 + 96 + 896（`anchors_r2c.py` 的 `jstar_disk0_only_differs`）。
pub(super) const MIRRORED_BIG_RING_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION: u64 =
    100;
pub(super) const MIRRORED_SMALL_RING_SIX_STATES_WHERE_THE_FIRST_DEVICE_ALONE_GIVES_ANOTHER_FARTHEST_VERSION:
    u64 = 996;
/// 原登记 J1、J4 两条对照在多盘域上的绝对值（登记 r2 第七节）：j\* − c ≥ 1 的态数，M 域 N ≤ 3 是 0 + 12 + 216，
/// M6 域 N ≤ 4 是 0 + 12 + 216 + 1 824，MC 域 Nc ≤ 2 是 3 + 42。
pub(super) const MIRRORED_BIG_RING_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND: u64 = 228;
pub(super) const MIRRORED_SMALL_RING_SIX_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND: u64 =
    2_052;
pub(super) const MIRRORED_MONOTONE_FARTHEST_AT_LEAST_ONE_WITHIN_THE_CONTROL_BOUND: u64 = 45;

/// 一条假结局跑在哪个档上：T-表、T-行两条对照拿 `--table both`，别的对照一律拿直接建表、盘次序照常（对照证的是判据分得出差别，
/// 与这一次跑选了哪张表、盘报什么次序无关；这样 `--table both` 的跑法里别的对照读数与默认跑法逐字相同）。
pub(super) fn run_profile_of_a_control(
    control: ControlKind,
    run_profile: RunProfile,
) -> RunProfile {
    let without_the_record_table_knobs = run_profile
        .with_table_source(RecordTableSource::Direct)
        .with_reported_disk_order(ReportedDiskOrder::FirstDeviceBeforeSecond);
    if matches!(
        control,
        ControlKind::ScanTableLosesItsLargestKeyRecord | ControlKind::ScanPassOutcomeBecomesAPanic
    ) {
        without_the_record_table_knobs.with_table_source(RecordTableSource::Both)
    } else {
        without_the_record_table_knobs
    }
}

/// 一条假结局在一个域上、从最小的 N 扫到 `largest_size` 的合计。
pub(super) fn control_tally(
    domain: Domain,
    largest_size: usize,
    control: ControlKind,
    run_profile: RunProfile,
    thread_count: usize,
) -> Tally {
    let run_profile = run_profile_of_a_control(control, run_profile);
    let mut total = Tally::new();
    for size in domain.smallest_size()..=largest_size {
        total.merge(&tally_of_a_domain_size(
            domain,
            size,
            Implementation::Control(control),
            run_profile,
            thread_count,
        ));
    }
    total
}

/// AS 三条的对照要 A-互 三个数都不为 0；对照界 n ≤ 2 里三个数全为 0 时照登记 5.4 放宽 n，一档一档放到第一段的界为止。
fn smallest_adversarial_size_where_the_three_readings_part(
    run_profile: RunProfile,
    thread_count: usize,
) -> usize {
    let largest_registered_size = BoundsTier::SegmentOne
        .largest_size(Domain::AdversarialBigRing)
        .expect("第一段跑 A 域");
    for largest_size in CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN..=largest_registered_size {
        let tally = control_tally(
            Domain::AdversarialBigRing,
            largest_size,
            ControlKind::SpecificationWithReading(FollowReading::WholeTable),
            run_profile,
            thread_count,
        );
        if tally.count_of(Counter::MutualWholeTableAndGapContinues) > 0
            && tally.count_of(Counter::MutualWholeTableAndGapStops) > 0
            && tally.count_of(Counter::MutualGapContinuesAndGapStops) > 0
        {
            return largest_size;
        }
    }
    largest_registered_size
}

pub(super) fn reading_that_must_be_positive(
    criterion: Criterion,
    measured_where: String,
    value: u64,
) -> ControlReading {
    ControlReading {
        criterion,
        measured_where,
        value,
        must_see: "greater_than_0".to_string(),
        is_red_as_required: value > 0,
    }
}

pub(super) fn reading_that_must_equal(
    criterion: Criterion,
    measured_where: String,
    value: u64,
    expected: u64,
) -> ControlReading {
    ControlReading {
        criterion,
        measured_where,
        value,
        must_see: format!("equals_{expected}"),
        is_red_as_required: value == expected,
    }
}

/// AS 三条各自的对照：拿另一种读法的 S 当实现，违例数必须等于 A-互 里那两种读法之间的那个数，且不为 0。
fn specification_control_readings(
    criterion: Criterion,
    other_reading: FollowReading,
    violation_counter: Counter,
    mutual_counter: Counter,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let largest_size =
        smallest_adversarial_size_where_the_three_readings_part(run_profile, thread_count);
    let tally = control_tally(
        Domain::AdversarialBigRing,
        largest_size,
        ControlKind::SpecificationWithReading(other_reading),
        run_profile,
        thread_count,
    );
    vec![
        reading_that_must_equal(
            criterion,
            format!(
                "domain=A,max_n={largest_size},implementation=S_{},counter={}",
                other_reading.label(),
                violation_counter.label()
            ),
            tally.count_of(violation_counter),
            tally.count_of(mutual_counter),
        ),
        reading_that_must_be_positive(
            criterion,
            format!(
                "domain=A,max_n={largest_size},counter={}",
                mutual_counter.label()
            ),
            tally.count_of(mutual_counter),
        ),
    ]
}

/// 判据表一行的阳性对照（登记 5.4 那张表的一行）。
pub(super) fn control_readings_of(
    criterion: Criterion,
    run_profile: RunProfile,
    thread_count: usize,
) -> Vec<ControlReading> {
    let big_ring_legal = |control: ControlKind| {
        control_tally(
            Domain::LegalBigRing,
            CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN,
            control,
            run_profile,
            thread_count,
        )
    };
    let adversarial = |largest_size: usize, control: ControlKind| {
        control_tally(
            Domain::AdversarialBigRing,
            largest_size,
            control,
            run_profile,
            thread_count,
        )
    };
    let legal_where = |counter: Counter| {
        format!(
            "domain=L,max_n={CONTROL_LARGEST_SIZE_OF_THE_BIG_RING_LEGAL_DOMAIN},counter={}",
            counter.label()
        )
    };
    match criterion {
        Criterion::LegalDefined => {
            let tally = big_ring_legal(ControlKind::PanicsOnTheDesignatedState);
            vec![reading_that_must_equal(
                criterion,
                legal_where(Counter::Panicked),
                tally.count_of(Counter::Panicked),
                1,
            )]
        }
        Criterion::ReturnedRoot => {
            let tally = big_ring_legal(ControlKind::OffTimelineRootWheneverApplied);
            vec![
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::ReturnedRootViolations),
                    tally.count_of(Counter::ReturnedRootViolations),
                    FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::FarthestAtLeastOne),
                    tally.count_of(Counter::FarthestAtLeastOne),
                    FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
                ),
            ]
        }
        Criterion::RootPlusDisk => {
            let tally = big_ring_legal(ControlKind::LandsOnePublishTooFarOverAnIncompletePublish);
            let named_units_tally = control_tally(
                Domain::NamedUnitsVerified,
                CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN,
                ControlKind::LandsOnePublishTooFarOverAnIncompletePublish,
                run_profile,
                thread_count,
            );
            vec![
                reading_that_must_be_positive(
                    criterion,
                    legal_where(Counter::RootPlusDiskViolations),
                    tally.count_of(Counter::RootPlusDiskViolations),
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::ReturnedRootViolations),
                    tally.count_of(Counter::ReturnedRootViolations),
                    0,
                ),
                reading_that_must_be_positive(
                    criterion,
                    format!(
                        "domain=U_verify_on,max_n={CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN},counter=J2"
                    ),
                    named_units_tally.count_of(Counter::RootPlusDiskViolations),
                ),
            ]
        }
        Criterion::WithinTheInFlightLimit => {
            let small_ring_largest_size = BoundsTier::SegmentOne
                .largest_size(Domain::LegalSmallRingSix)
                .expect("第一段跑 L 小环 6");
            let small_ring_tally = control_tally(
                Domain::LegalSmallRingSix,
                small_ring_largest_size,
                ControlKind::IgnoresTheInFlightLimit,
                run_profile,
                thread_count,
            );
            let big_ring_tally = big_ring_legal(ControlKind::IgnoresTheInFlightLimit);
            vec![
                reading_that_must_be_positive(
                    criterion,
                    format!("domain=L6,max_n={small_ring_largest_size},counter=J3"),
                    small_ring_tally.count_of(Counter::InFlightLimitViolations),
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::InFlightLimitViolations),
                    big_ring_tally.count_of(Counter::InFlightLimitViolations),
                    0,
                ),
            ]
        }
        Criterion::FarthestVersion => {
            let tally = big_ring_legal(ControlKind::AlwaysReturnsTheChosenRoot);
            vec![
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::FarthestVersionViolations),
                    tally.count_of(Counter::FarthestVersionViolations),
                    FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::ReturnedRootViolations),
                    tally.count_of(Counter::ReturnedRootViolations),
                    0,
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::RootPlusDiskViolations),
                    tally.count_of(Counter::RootPlusDiskViolations),
                    0,
                ),
                reading_that_must_equal(
                    criterion,
                    legal_where(Counter::InFlightLimitViolations),
                    tally.count_of(Counter::InFlightLimitViolations),
                    0,
                ),
            ]
        }
        Criterion::J5aAppliedSet | Criterion::J5bLandedVersion => {
            let tally = control_tally(
                Domain::MonotoneBigRing,
                CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
                ControlKind::ReturnsTheChosenRootWhenEveryRecordIsOnDisk,
                run_profile,
                thread_count,
            );
            let column = if criterion == Criterion::J5aAppliedSet {
                PairColumn::TodayAppliedSetViolations
            } else {
                PairColumn::TodayLandedVersionViolations
            };
            vec![reading_that_must_be_positive(
                criterion,
                format!(
                    "domain=C,max_n={CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN},set=wide,column={}",
                    column.label()
                ),
                tally.pair(PairSet::Wide, column),
            )]
        }
        Criterion::J5aAndJ5bAreSeparable => {
            let small_ring_largest_size = BoundsTier::SegmentOne
                .largest_size(Domain::MonotoneSmallRingSix)
                .expect("第一段跑 C 小环 6");
            let mut readings = Vec::new();
            for (domain, largest_size) in [
                (
                    Domain::MonotoneBigRing,
                    CONTROL_LARGEST_SIZE_OF_THE_MONOTONE_BIG_RING_DOMAIN,
                ),
                (Domain::MonotoneSmallRingSix, small_ring_largest_size),
            ] {
                let tally = control_tally(
                    domain,
                    largest_size,
                    ControlKind::IdealWithTheCumulativeAppliedSet,
                    run_profile,
                    thread_count,
                );
                for set in PairSet::ALL {
                    readings.push(reading_that_must_equal(
                        criterion,
                        format!(
                            "domain={},max_n={largest_size},set={},column=today_j5a_must_equal_today_j5b",
                            domain.label(),
                            set.label()
                        ),
                        tally.pair(*set, PairColumn::TodayAppliedSetViolations),
                        tally.pair(*set, PairColumn::TodayLandedVersionViolations),
                    ));
                }
            }
            readings
        }
        Criterion::FixedPoint => {
            let tally = big_ring_legal(ControlKind::SecondCallMovesOn);
            vec![reading_that_must_be_positive(
                criterion,
                legal_where(Counter::FixedPointPremiseHoldsViolations),
                tally.count_of(Counter::FixedPointPremiseHoldsViolations)
                    + tally.count_of(Counter::FixedPointPremiseFailsViolations),
            )]
        }
        Criterion::Ia1LargestTxgFails => {
            let largest_size = BoundsTier::SegmentOne
                .largest_size(Domain::TxgAtTheLargest)
                .expect("第一段跑 E2");
            let tally = control_tally(
                Domain::TxgAtTheLargest,
                largest_size,
                ControlKind::ReportsNoFailureOnTheLargestTxg,
                run_profile,
                thread_count,
            );
            vec![reading_that_must_be_positive(
                criterion,
                format!("domain=E2,max_n={largest_size},counter=IA1"),
                tally.count_of(Counter::Ia1Violations),
            )]
        }
        Criterion::Ia2NoVerificationCounted => {
            let tally = control_tally(
                Domain::NamedUnitsNotVerified,
                CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN,
                ControlKind::ReportsOneVerificationWithoutVerifying,
                run_profile,
                thread_count,
            );
            vec![reading_that_must_be_positive(
                criterion,
                format!(
                    "domain=U_verify_off,max_n={CONTROL_LARGEST_SIZE_OF_THE_NAMED_UNITS_DOMAIN},counter=IA2"
                ),
                tally.count_of(Counter::Ia2Violations),
            )]
        }
        Criterion::AdversarialDefined => {
            let tally = adversarial(
                CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
                ControlKind::PanicsOnTheDesignatedState,
            );
            vec![reading_that_must_equal(
                criterion,
                format!(
                    "domain=A,max_n={CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN},counter=panicked"
                ),
                tally.count_of(Counter::Panicked),
                1,
            )]
        }
        Criterion::AsWholeTable => specification_control_readings(
            criterion,
            FollowReading::ConsecutiveGapContinues,
            Counter::AsWholeTableViolations,
            Counter::MutualWholeTableAndGapContinues,
            run_profile,
            thread_count,
        ),
        Criterion::AsGapContinues => specification_control_readings(
            criterion,
            FollowReading::ConsecutiveGapStops,
            Counter::AsGapContinuesViolations,
            Counter::MutualGapContinuesAndGapStops,
            run_profile,
            thread_count,
        ),
        Criterion::AsGapStops => specification_control_readings(
            criterion,
            FollowReading::WholeTable,
            Counter::AsGapStopsViolations,
            Counter::MutualWholeTableAndGapStops,
            run_profile,
            thread_count,
        ),
        Criterion::AMutual => {
            let largest_size =
                smallest_adversarial_size_where_the_three_readings_part(run_profile, thread_count);
            let tally = adversarial(
                largest_size,
                ControlKind::SpecificationWithReading(FollowReading::NeverJudges),
            );
            vec![reading_that_must_be_positive(
                criterion,
                format!(
                    "domain=A,max_n={largest_size},implementation=S_never_judges,counter=AS_whole_table"
                ),
                tally.count_of(Counter::AsWholeTableViolations),
            )]
        }
        Criterion::AStructure => {
            let tally = adversarial(
                CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
                ControlKind::RootFromARecordWithoutTheLastRecordFlag,
            );
            vec![reading_that_must_be_positive(
                criterion,
                format!(
                    "domain=A,max_n={CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN},counter=A_structure"
                ),
                tally.count_of(Counter::StructureViolations),
            )]
        }
        Criterion::ALimit => {
            let tally = adversarial(
                CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN,
                ControlKind::ReportsOneRecordOverTheLimit,
            );
            vec![reading_that_must_equal(
                criterion,
                format!(
                    "domain=A,max_n={CONTROL_LARGEST_SIZE_OF_THE_ADVERSARIAL_DOMAIN},counter=A_limit_must_equal_states"
                ),
                tally.count_of(Counter::LimitViolations),
                tally.count_of(Counter::States),
            )]
        }
        Criterion::QnFixedPoint => j6qn_control_readings(run_profile, thread_count),
        Criterion::J7aInstanceTable => j7a_control_readings(
            criterion,
            FieldOfTheRebuiltRoot::InstanceTable,
            run_profile,
            thread_count,
        ),
        Criterion::J7aAllocationRecordTreeRoot => j7a_control_readings(
            criterion,
            FieldOfTheRebuiltRoot::AllocationRecordTreeRoot,
            run_profile,
            thread_count,
        ),
        Criterion::J7aFilesystemIdentifier => j7a_control_readings(
            criterion,
            FieldOfTheRebuiltRoot::FilesystemIdentifier,
            run_profile,
            thread_count,
        ),
        Criterion::J7bUnmountMarkerOfARebuiltRoot => unmount_marker_control_readings(
            criterion,
            ControlKind::IdealCopyingTheChosenUnmountMarkerWhenApplied,
            Counter::J7bViolations,
            FARTHEST_AT_LEAST_ONE_STATES_WITHIN_THE_CONTROL_BOUND,
            run_profile,
            thread_count,
        ),
        Criterion::J7cUnmountMarkerWhenNothingIsApplied => unmount_marker_control_readings(
            criterion,
            ControlKind::IdealWritingNotWrittenWhenNothingIsApplied,
            Counter::J7cViolations,
            FARTHEST_IS_THE_CHOSEN_VERSION_STATES_WITHIN_THE_CONTROL_BOUND,
            run_profile,
            thread_count,
        ),
        Criterion::J8aFieldsFromTheNewRootSegment => {
            c8a_control_readings(run_profile, thread_count)
        }
        Criterion::J8aUnmountMarkerFromTheNewRootSegment => {
            c8b_control_readings(run_profile, thread_count)
        }
        Criterion::J8bFilesystemIdentifierOfTheChosenRoot => {
            c8c_control_readings(run_profile, thread_count)
        }
        Criterion::J8cNothingAppliedReturnsTheChosenRoot => {
            c8d_control_readings(run_profile, thread_count)
        }
        Criterion::MultiDiskAnyCopyIsOnDisk => any_copy_control_readings(run_profile, thread_count),
        Criterion::MultiDiskReturnedRoot => {
            mirrored_returned_root_control_readings(run_profile, thread_count)
        }
        Criterion::MultiDiskFarthestVersion => {
            mirrored_farthest_version_control_readings(run_profile, thread_count)
        }
        Criterion::MultiDiskAppliedSetAndLandedVersion => {
            mirrored_monotone_control_readings(run_profile, thread_count)
        }
        Criterion::TableComparisonSeesADifferentTable => {
            table_comparison_control_readings_on_tables(criterion, run_profile, thread_count)
        }
        Criterion::TableComparisonSeesADifferentRow => {
            table_comparison_control_readings_on_rows(criterion, run_profile, thread_count)
        }
        Criterion::QnFixedPointSplitByTxgOrder => {
            txg_order_split_control_readings(run_profile, thread_count)
        }
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod root_choice_fault_tests;

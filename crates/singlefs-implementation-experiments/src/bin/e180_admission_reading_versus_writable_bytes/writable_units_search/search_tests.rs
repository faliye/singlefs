use super::*;

/// 假探测器：落点那一道、journal 记录上限、式子那一判各有一个界，次序同 `prepare_the_version_publish`（先落点、后记录数、最后式子）。
struct FakeProber {
    landing_units: u64,
    record_limit: u64,
    /// 记录数 = 单元数 + 固定几条 + 每多少个单元再多一条（让「固定几条」随单元数缓慢地变）。
    fixed_extra_records: u64,
    units_per_additional_record: u64,
    writable_units: u64,
    parallel_probes: usize,
    /// 探了几个点（按需收紧与一律收紧的探测次数对着报）。
    probes_run: std::cell::Cell<u64>,
}

impl FakeProber {
    fn records_of(&self, units: u64) -> u64 {
        units + self.fixed_extra_records + units / self.units_per_additional_record
    }
}

impl UnitsProbe for FakeProber {
    fn probe_in_parallel(&self, unit_counts: &[u64]) -> Vec<ProbeVerdict> {
        self.probes_run
            .set(self.probes_run.get() + u64::try_from(unit_counts.len()).expect("探测数"));
        unit_counts
            .iter()
            .map(|units| {
                if *units > self.landing_units {
                    ProbeVerdict::RefusedForSpace
                } else if self.records_of(*units) > self.record_limit {
                    ProbeVerdict::JournalLimit {
                        records_of_the_publish: self.records_of(*units),
                        record_limit: self.record_limit,
                    }
                } else if *units > self.writable_units {
                    ProbeVerdict::RefusedForSpace
                } else {
                    ProbeVerdict::Admitted
                }
            })
            .collect()
    }

    fn parallel_probe_count(&self) -> usize {
        self.parallel_probes
    }
}

fn prober(writable_units: u64, landing_units: u64, parallel_probes: usize) -> FakeProber {
    FakeProber {
        landing_units,
        record_limit: 65_536,
        fixed_extra_records: 3,
        units_per_additional_record: 1000,
        writable_units,
        parallel_probes,
        probes_run: std::cell::Cell::new(0),
    }
}

#[test]
fn the_search_finds_the_exact_boundary_below_the_journal_cap_for_every_parallelism() {
    for parallel_probes in [1, 2, 6, 31] {
        for writable_units in [0, 1, 2, 777, 1234, 40_000] {
            let mut model = JournalModel::NOT_LEARNED_YET;
            let found = find_writable_units(
                &prober(writable_units, 100_000, parallel_probes),
                writable_units + 500,
                &mut model,
                &LowerBoundTightening::AtEveryCensoredState,
            );
            assert_eq!(
                found,
                WritableUnits::Exact {
                    units: writable_units,
                    non_monotone_probes_seen: false
                },
                "writable={writable_units} parallel={parallel_probes}"
            );
        }
    }
}

#[test]
fn pool_roomier_than_the_journal_cap_is_censored_at_the_largest_units_that_still_pass_the_record_limit(
) {
    // 固定那几条记录随单元数变（每 1000 个单元多一条）：在 12 万个单元处学来的上限偏低，下界得往上收到记录上限边上。
    // 每多少个单元多一条取几种，其中一种大到不变；并行度取 1、2、6、31，覆盖步长翻倍那一支的各种点数。
    for units_per_additional_record in [1000, 100, 7000, u64::MAX] {
        for parallel_probes in [1, 2, 6, 31] {
            let mut fake = prober(200_000, 200_000, parallel_probes);
            fake.units_per_additional_record = units_per_additional_record;
            let mut model = JournalModel::NOT_LEARNED_YET;
            let found = find_writable_units(
                &fake,
                130_000,
                &mut model,
                &LowerBoundTightening::AtEveryCensoredState,
            );
            match found {
                WritableUnits::AtLeastBecauseOfTheJournalCap {
                    lower_bound_units, ..
                } => {
                    assert!(fake.records_of(lower_bound_units) <= fake.record_limit);
                    assert!(
                        fake.records_of(lower_bound_units + 1) > fake.record_limit,
                        "下界已经收到记录上限边上：再多一个单元就被记录数拒 \
                             (每多少单元多一条={units_per_additional_record} 并行={parallel_probes} 下界={lower_bound_units})"
                    );
                }
                other @ (WritableUnits::Exact { .. } | WritableUnits::Unreadable(_)) => {
                    panic!("该是截尾，得到 {other:?}")
                }
            }
            assert!(model.largest_units_below_the_record_limit.is_some());
        }
    }
}

#[test]
fn space_bound_that_is_not_a_bound_is_widened_to_the_journal_cap_when_the_cap_is_known() {
    let fake = prober(30_000, 100_000, 4);
    let mut model = JournalModel {
        largest_units_below_the_record_limit: Some(64_000),
        records_beyond_one_per_unit: Some(3),
        record_limit: Some(65_536),
    };
    // 给的上界 10_000 偏低（第一个探测点 10_001 其实放行）：装置往记录上限那一边找，仍找得到 30_000。
    let found = find_writable_units(
        &fake,
        10_000,
        &mut model,
        &LowerBoundTightening::AtEveryCensoredState,
    );
    assert_eq!(
        found,
        WritableUnits::Exact {
            units: 30_000,
            non_monotone_probes_seen: false
        }
    );
}

#[test]
fn state_where_the_landing_gate_refuses_before_the_record_limit_is_found_exactly_at_zero() {
    let fake = prober(0, 0, 3);
    let mut model = JournalModel::NOT_LEARNED_YET;
    assert_eq!(
        find_writable_units(
            &fake,
            0,
            &mut model,
            &LowerBoundTightening::AtEveryCensoredState
        ),
        WritableUnits::Exact {
            units: 0,
            non_monotone_probes_seen: false
        }
    );
}

#[test]
fn the_search_does_not_report_the_boundary_one_unit_too_high() {
    // 二分多一单元的变异要在这里红：T 必须恰是最大放行的单元数。
    let fake = prober(5_000, 100_000, 6);
    let mut model = JournalModel::NOT_LEARNED_YET;
    let found = find_writable_units(
        &fake,
        5_100,
        &mut model,
        &LowerBoundTightening::AtEveryCensoredState,
    );
    let WritableUnits::Exact { units, .. } = found else {
        panic!("该是精确值")
    };
    assert_eq!(fake.probe_one(units), ProbeVerdict::Admitted);
    assert_eq!(fake.probe_one(units + 1), ProbeVerdict::RefusedForSpace);
}

#[test]
fn the_journal_model_learns_the_cap_from_the_records_beyond_one_per_unit() {
    let mut model = JournalModel::NOT_LEARNED_YET;
    model.learn_from(120_000, 120_003, 65_536);
    assert_eq!(model.records_beyond_one_per_unit, Some(3));
    assert_eq!(model.largest_units_below_the_record_limit, Some(65_533));
    assert_eq!(model.record_limit, Some(65_536));
}

/// 一串假状态里的一个（同一个模型从头学起，像一族历史那样一个接一个探）：可写的单元数、空间上界、各臂要的单元数。
struct FakeState {
    name: &'static str,
    writable_units: u64,
    space_bound: u64,
    units_asked_for_by_the_arms: Vec<u64>,
}

/// 记录上限边上的单元数：再多一个就被记录数拒。照假探测器的记录数逐个往上数，不经搜索。
fn units_at_the_record_limit_edge(fake: &FakeProber) -> u64 {
    let mut units = 0;
    while fake.records_of(units + 1) <= fake.record_limit {
        units += 1;
    }
    units
}

/// 在远处（[`PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS`]）学一次记录上限之后，记录上限处放行的那个点：上限减去远处那一次多出来的记录数。
fn admitted_point_learned_far_above_the_cap(fake: &FakeProber) -> u64 {
    let far = PROBE_FAR_ABOVE_ANY_JOURNAL_CAP_UNITS;
    fake.record_limit - (fake.records_of(far) - far)
}

/// 各臂要的单元数取在：放行点及以下、放行点之上一个、记录上限边上两侧、记录上限之上，外加一个可写量落在放行点与边之间的截尾状态
/// 和一个不截尾的状态。
fn fake_states(admitted_point: u64, edge: u64, record_limit: u64) -> Vec<FakeState> {
    let between = admitted_point + (edge - admitted_point) / 2;
    vec![
        FakeState {
            name: "censored_every_ask_at_or_below_the_admitted_point",
            writable_units: 200_000,
            space_bound: 130_000,
            units_asked_for_by_the_arms: vec![0, admitted_point / 2, admitted_point],
        },
        FakeState {
            name: "censored_an_ask_one_above_the_admitted_point",
            writable_units: 200_000,
            space_bound: 130_000,
            units_asked_for_by_the_arms: vec![0, admitted_point + 1],
        },
        FakeState {
            name: "censored_asks_on_both_sides_of_the_record_limit_edge",
            writable_units: 200_000,
            space_bound: 130_000,
            units_asked_for_by_the_arms: vec![edge, edge + 1, record_limit],
        },
        FakeState {
            name: "censored_asks_above_the_record_limit_only",
            writable_units: 200_000,
            space_bound: 130_000,
            units_asked_for_by_the_arms: vec![record_limit + 1, 10 * record_limit],
        },
        FakeState {
            name: "exact_below_the_cap",
            writable_units: 40_000,
            space_bound: 40_500,
            units_asked_for_by_the_arms: vec![39_999, 40_000, 40_001, admitted_point + 1],
        },
        FakeState {
            name: "censored_writable_between_the_admitted_point_and_the_edge",
            writable_units: between,
            space_bound: 130_000,
            units_asked_for_by_the_arms: vec![between, between + 1],
        },
        FakeState {
            name: "censored_after_an_exact_state_every_ask_at_or_below_the_admitted_point",
            writable_units: 200_000,
            space_bound: 130_000,
            units_asked_for_by_the_arms: vec![admitted_point],
        },
    ]
}

/// 一个状态上量到的：T 的结果、每条臂的判定、探了几个点。
struct MeasuredFakeState {
    writable: WritableUnits,
    judgements: Vec<Judgement>,
    probes_run: u64,
}

fn tightening_at_every_censored_state(_state: &FakeState) -> LowerBoundTightening {
    LowerBoundTightening::AtEveryCensoredState
}

fn tightening_only_where_an_arm_judgement_can_change(state: &FakeState) -> LowerBoundTightening {
    LowerBoundTightening::OnlyWhereAnArmJudgementCanChange {
        units_asked_for_by_the_arms: state.units_asked_for_by_the_arms.clone(),
    }
}

/// 照一族历史的样子一个状态接一个探（模型跨状态带着），截尾的状态收不收紧照 `tightening_of`。
fn measure_fake_states(
    states: &[FakeState],
    units_per_additional_record: u64,
    parallel_probes: usize,
    tightening_of: fn(&FakeState) -> LowerBoundTightening,
) -> Vec<MeasuredFakeState> {
    let mut model = JournalModel::NOT_LEARNED_YET;
    states
        .iter()
        .map(|state| {
            let mut fake = prober(state.writable_units, 200_000, parallel_probes);
            fake.units_per_additional_record = units_per_additional_record;
            let tightening = tightening_of(state);
            let writable = find_writable_units(&fake, state.space_bound, &mut model, &tightening);
            let judgements = state
                .units_asked_for_by_the_arms
                .iter()
                .map(|units| judge(units * DATA_UNIT_PAYLOAD_BYTES, &writable))
                .collect();
            MeasuredFakeState {
                writable,
                judgements,
                probes_run: fake.probes_run.get(),
            }
        })
        .collect()
}

/// 登记第十二节修订 4：截尾状态上只在某条臂要的单元数落在（放行点, 记录上限] 里时收紧，每条臂在每个状态上的判定与一律收紧逐项相同；
/// 探测次数逐状态打出来（`--nocapture` 看），按需收紧的不多于一律收紧，合计严格少。
#[test]
fn tightening_only_where_an_arm_judgement_can_change_judges_every_arm_as_tightening_at_every_censored_state(
) {
    for units_per_additional_record in [1000, 100, 7000, u64::MAX] {
        for parallel_probes in [1, 2, 6, 31] {
            let mut reference = prober(200_000, 200_000, parallel_probes);
            reference.units_per_additional_record = units_per_additional_record;
            let edge = units_at_the_record_limit_edge(&reference);
            let admitted_point = admitted_point_learned_far_above_the_cap(&reference);
            let states = fake_states(admitted_point, edge, reference.record_limit);
            let at_every = measure_fake_states(
                &states,
                units_per_additional_record,
                parallel_probes,
                tightening_at_every_censored_state,
            );
            let on_demand = measure_fake_states(
                &states,
                units_per_additional_record,
                parallel_probes,
                tightening_only_where_an_arm_judgement_can_change,
            );
            let case = format!(
                "每多少单元多一条={units_per_additional_record} 并行={parallel_probes} 放行点={admitted_point} 边={edge}"
            );
            let mut probes_at_every_in_total = 0;
            let mut probes_on_demand_in_total = 0;
            for ((state, every), demand) in states.iter().zip(&at_every).zip(&on_demand) {
                assert_eq!(
                    demand.judgements, every.judgements,
                    "{case} 状态={}：按需收紧与一律收紧给的判定不同",
                    state.name
                );
                assert!(
                    demand.probes_run <= every.probes_run,
                    "{case} 状态={}：按需收紧探得比一律收紧多（{} > {}）",
                    state.name,
                    demand.probes_run,
                    every.probes_run
                );
                println!(
                    "E180-tightening-probes units_per_additional_record={units_per_additional_record} \
                         parallel={parallel_probes} state={} at_every_censored_state={} \
                         only_where_an_arm_judgement_can_change={} result_at_every={:?} result_on_demand={:?}",
                    state.name, every.probes_run, demand.probes_run, every.writable, demand.writable
                );
                probes_at_every_in_total += every.probes_run;
                probes_on_demand_in_total += demand.probes_run;
            }
            assert!(
                probes_on_demand_in_total < probes_at_every_in_total,
                "{case}：合计探测次数按需收紧 {probes_on_demand_in_total} 应严格少于一律收紧 {probes_at_every_in_total}"
            );
        }
    }
}

/// 钉绝对值：哪一个状态收、哪一个不收，不收的下界停在放行点、探测次数是学上限 1 次加放行点 1 次（第一个状态）或只有放行点 1 次。
#[test]
fn tightening_on_demand_leaves_the_lower_bound_at_the_admitted_point_unless_an_ask_lies_above_it_within_the_record_limit(
) {
    for units_per_additional_record in [1000, 100, 7000, u64::MAX] {
        for parallel_probes in [1, 2, 6, 31] {
            let mut reference = prober(200_000, 200_000, parallel_probes);
            reference.units_per_additional_record = units_per_additional_record;
            let edge = units_at_the_record_limit_edge(&reference);
            let admitted_point = admitted_point_learned_far_above_the_cap(&reference);
            let states = fake_states(admitted_point, edge, reference.record_limit);
            let measured = measure_fake_states(
                &states,
                units_per_additional_record,
                parallel_probes,
                tightening_only_where_an_arm_judgement_can_change,
            );
            let case =
                format!("每多少单元多一条={units_per_additional_record} 并行={parallel_probes}");
            let untightened = WritableUnits::AtLeastBecauseOfTheJournalCap {
                lower_bound_units: admitted_point,
                lower_bound_was_tightened: false,
            };
            assert_eq!(measured[0].writable, untightened, "{case} 第一个状态不收");
            assert_eq!(
                measured[0].probes_run, 2,
                "{case} 第一个状态：学上限 1 次 + 放行点 1 次"
            );
            assert_eq!(
                measured[3].writable, untightened,
                "{case} 只有记录上限之上的臂：不收"
            );
            assert_eq!(
                measured[3].probes_run, 1,
                "{case} 学过上限之后不收的状态只探放行点 1 次"
            );
            assert_eq!(
                measured[4].writable,
                WritableUnits::Exact {
                    units: 40_000,
                    non_monotone_probes_seen: false
                },
                "{case} 不截尾的状态照旧取精确值"
            );
            assert_eq!(
                measured[6].writable, untightened,
                "{case} 精确状态之后再截尾：不收"
            );
            assert_eq!(measured[6].probes_run, 1, "{case}");
            assert_eq!(
                measured[2].writable,
                WritableUnits::AtLeastBecauseOfTheJournalCap {
                    lower_bound_units: edge,
                    lower_bound_was_tightened: true,
                },
                "{case} 有臂要边上的单元数：收到边上"
            );
            assert_eq!(
                measured[2].judgements,
                vec![
                    Judgement::CertainlyDoesNotOverReportBelowTheLowerBound,
                    Judgement::Undetermined,
                    Judgement::Undetermined,
                ],
                "{case} 边上的判一定不多报，边之上一个与记录上限判未定"
            );
            assert_eq!(
                measured[3].judgements,
                vec![Judgement::Undetermined, Judgement::Undetermined],
                "{case}"
            );
            // 可写量在放行点与边之间：收紧碰到被空间拒的点就停（下界照样成立），停在放行点与可写量之间，不一定贴着可写量。
            let between = admitted_point + (edge - admitted_point) / 2;
            match &measured[5].writable {
                WritableUnits::AtLeastBecauseOfTheJournalCap {
                    lower_bound_units,
                    lower_bound_was_tightened: true,
                } => assert!(
                    (admitted_point..=between).contains(lower_bound_units),
                    "{case} 下界 {lower_bound_units} 该在放行点 {admitted_point} 与可写量 {between} 之间"
                ),
                other @ (WritableUnits::Exact { .. }
                | WritableUnits::AtLeastBecauseOfTheJournalCap { .. }
                | WritableUnits::Unreadable(_)) => {
                    panic!("{case} 可写量在放行点与边之间：该是收过的截尾，得到 {other:?}")
                }
            }
        }
    }
}

/// 记录上限还没学到时（模型没有上限），按需收紧照收：判不了收紧改不改判定，就按改算。
#[test]
fn tightening_on_demand_counts_an_unknown_record_limit_as_one_that_could_change_a_judgement() {
    assert!(tightening_can_change_an_arm_judgement(10, None, &[11]));
    assert!(!tightening_can_change_an_arm_judgement(10, None, &[10]));
    assert!(tightening_can_change_an_arm_judgement(10, Some(11), &[11]));
    assert!(!tightening_can_change_an_arm_judgement(10, Some(11), &[12]));
    assert!(!tightening_can_change_an_arm_judgement(10, Some(11), &[]));
}

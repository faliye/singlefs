//! 择根合判一族的阳性对照（登记 5.7）、产物行、命令行与主流程（`--family root_choice`，只在 `--profile r3`）。

use super::*;

// ── 阳性对照（登记 5.7；新的几条读实现的结局或调三步，不进启动自检，放进单测，产物里打 `name=control` 行）

#[derive(Clone, Debug)]
struct RootChoiceControlReading {
    criterion: RootChoiceCriterion,
    measured_where: String,
    value: u64,
    must_see: String,
    is_red_as_required: bool,
}

fn root_choice_reading_that_must_equal(
    criterion: RootChoiceCriterion,
    measured_where: String,
    value: u64,
    expected: u64,
) -> RootChoiceControlReading {
    RootChoiceControlReading {
        criterion,
        measured_where,
        value,
        must_see: format!("equals_{expected}"),
        is_red_as_required: value == expected,
    }
}

fn root_choice_place_of(domain: RootChoiceDomain, cell: RootChoiceCell) -> String {
    format!(
        "domain={} ring_slots={} slots_per_region={} mounts={} user_publishes={}",
        domain.label(),
        domain.ring().ring_slots,
        domain.slots_per_region(),
        cell.mounts,
        cell.user_publishes
    )
}

/// 每条判据在每个 (域, M, U) 格上的对照读数：KJ0 在 K 域 M = 1、U = 0 的指定一态上 panic（违例数 1）；KJ1、KJ2 在 KF1 下等于第七节乙类 KF1 那两列；
/// KJ1、KJ2、KJ3 在 KF2 下各等于「有已确认发布的态数」；KJ2 在 KL 上本地写的几种择根序下等于 0、6、0、4（今天的序 0；第 7 次重跑登记第七节乙类 B2）。
fn root_choice_control_readings(
    selected_cells: &[(RootChoiceDomain, Vec<RootChoiceCell>)],
    thread_count: usize,
) -> Vec<RootChoiceControlReading> {
    let mut readings = Vec::new();
    for (domain, cells) in selected_cells {
        for cell in cells {
            let place = root_choice_place_of(*domain, *cell);
            if !domain.is_a_legal_history() {
                for rule in LocalRootChoiceRule::ALL {
                    let tally = root_choice_tally_of_a_cell(
                        *domain,
                        *cell,
                        RootChoiceImplementation::LocalChoice(rule),
                        thread_count,
                    );
                    readings.push(root_choice_reading_that_must_equal(
                        RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
                        format!("{place} local_choice={}", rule.label()),
                        tally.count_of(RootChoiceCounter::Kj2Violations),
                        pinned_root_choice_adversarial_kj2_violations(rule),
                    ));
                }
                continue;
            }
            let pinned = pinned_root_choice_cell(*domain, *cell)
                .expect("每个跑到的合法历史格都在登记第七节乙类的表里");
            if *domain == RootChoiceDomain::BigRing && cell.mounts == 1 && cell.user_publishes == 0
            {
                let tally = root_choice_tally_of_a_cell(
                    *domain,
                    *cell,
                    RootChoiceImplementation::PanicsOnTheDesignatedState,
                    thread_count,
                );
                readings.push(root_choice_reading_that_must_equal(
                    RootChoiceCriterion::Kj0Defined,
                    format!("{place} fake_outcome=panics_on_the_designated_state"),
                    tally.count_of(RootChoiceCounter::Kj0Violations),
                    1,
                ));
            }
            let kf1 = root_choice_tally_of_a_cell(
                *domain,
                *cell,
                RootChoiceImplementation::ChoosesTheOldestRoot,
                thread_count,
            );
            for (criterion, counter, expected) in [
                (
                    RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
                    RootChoiceCounter::Kj1Violations,
                    pinned.oldest_root_kj1,
                ),
                (
                    RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
                    RootChoiceCounter::Kj2Violations,
                    pinned.oldest_root_kj2,
                ),
            ] {
                readings.push(root_choice_reading_that_must_equal(
                    criterion,
                    format!("{place} fake_outcome=KF1_chooses_the_oldest_root"),
                    kf1.count_of(counter),
                    expected,
                ));
            }
            let kf2 = root_choice_tally_of_a_cell(
                *domain,
                *cell,
                RootChoiceImplementation::LandsOneVersionBeforeTheLastConfirmed,
                thread_count,
            );
            for (criterion, counter) in [
                (
                    RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
                    RootChoiceCounter::Kj1Violations,
                ),
                (
                    RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
                    RootChoiceCounter::Kj2Violations,
                ),
                (
                    RootChoiceCriterion::Kj3LandsOnTheIdealRecovery,
                    RootChoiceCounter::Kj3Violations,
                ),
            ] {
                readings.push(root_choice_reading_that_must_equal(
                    criterion,
                    format!("{place} fake_outcome=KF2_lands_one_version_before_the_last_confirmed"),
                    kf2.count_of(counter),
                    pinned.confirmed_states,
                ));
            }
        }
    }
    readings
}

// ── 产物行

/// `name=root_choice_states` 行里数的量，按这个次序逐个写 `名字=数`。
const ROOT_CHOICE_COUNTERS_OF_THE_STATES_ROW: [RootChoiceCounter; 18] = [
    RootChoiceCounter::States,
    RootChoiceCounter::ConfirmedStates,
    RootChoiceCounter::FirstSelfVerifiedRootIsNotTheNewest,
    RootChoiceCounter::SameTxgDifferentInstanceRoots,
    RootChoiceCounter::HigherInstanceLowerTxgRoots,
    RootChoiceCounter::TwoOrMoreInstancesInTheRing,
    RootChoiceCounter::IdealRecoveryApplied,
    RootChoiceCounter::AboveNewestRootOverLimit,
    RootChoiceCounter::AboveNewestRootAtLimit,
    RootChoiceCounter::AboveChosenRootOverLimit,
    RootChoiceCounter::AboveChosenRootAtLimit,
    RootChoiceCounter::StatesWithoutASelfVerifiedRoot,
    RootChoiceCounter::PanickedStates,
    RootChoiceCounter::ChoseNothingStates,
    RootChoiceCounter::ReplayFailedStates,
    RootChoiceCounter::ReplayAppliedStates,
    RootChoiceCounter::SlotReadingsChecked,
    RootChoiceCounter::SlotReadingsDifferingFromTheTruth,
];

/// 一个 (域, M, U) 格的全部行：状态数与几样计数、每条判据一行违例数（KL 只判 KJ0、KJ2，另两条不出行）。
pub(super) fn emit_the_rows_of_a_root_choice_cell(
    emitter: &mut Emitter,
    domain: RootChoiceDomain,
    cell: RootChoiceCell,
    tally: &RootChoiceTally,
) {
    let place = root_choice_place_of(domain, cell);
    let counter_fields: Vec<String> = ROOT_CHOICE_COUNTERS_OF_THE_STATES_ROW
        .iter()
        .map(|counter| format!("{}={}", counter.label(), tally.count_of(*counter)))
        .collect();
    emitter.emit_result_line(&format!(
        "name=root_choice_states {place} {} in_flight_limit={} above_newest_root_max={} above_chosen_root_max={} confirmed_publishes_max={} largest_counter={} slot_reading_witness={}",
        counter_fields.join(" "),
        domain.ring().local_in_flight_limit(),
        tally.above_newest_root_maximum,
        tally.above_chosen_root_maximum,
        tally.confirmed_publishes_maximum,
        tally.largest_counter,
        tally.witness(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
    ));
    let mut violations_row =
        |criterion: RootChoiceCriterion, violations: RootChoiceCounter, judged: u64| {
            emitter.emit_result_line(&format!(
                "name=violations {place} criterion={} violations={} judged={judged} witness={}",
                criterion.label(),
                tally.count_of(violations),
                tally.witness(violations)
            ));
        };
    violations_row(
        RootChoiceCriterion::Kj0Defined,
        RootChoiceCounter::Kj0Violations,
        tally.count_of(RootChoiceCounter::States),
    );
    if domain.is_a_legal_history() {
        violations_row(
            RootChoiceCriterion::Kj1NotEarlierThanTheLastConfirmedPublish,
            RootChoiceCounter::Kj1Violations,
            tally.count_of(RootChoiceCounter::Kj1Judged),
        );
    }
    violations_row(
        RootChoiceCriterion::Kj2NotEarlierThanAnySelfVerifiedRoot,
        RootChoiceCounter::Kj2Violations,
        tally.count_of(RootChoiceCounter::Kj2Judged),
    );
    if domain.is_a_legal_history() {
        violations_row(
            RootChoiceCriterion::Kj3LandsOnTheIdealRecovery,
            RootChoiceCounter::Kj3Violations,
            tally.count_of(RootChoiceCounter::Kj3Judged),
        );
    }
}

/// 一个域扫完之后的判决行（各判据在这个域全部格上的合计），并记作废与停机。
pub(super) fn emit_the_verdict_of_a_root_choice_domain(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    domain: RootChoiceDomain,
    cells: &[RootChoiceCell],
    total: &RootChoiceTally,
) {
    let largest_mounts = cells.iter().map(|cell| cell.mounts).max().unwrap_or(0);
    let largest_user_publishes = cells
        .iter()
        .map(|cell| cell.user_publishes)
        .max()
        .unwrap_or(0);
    // 登记作废条款 TV10：域是空的（合法历史三个域合起来没有一个已确认的态，KJ1 一态都没判）；KL 上（第 7 次重跑登记 5.6 H3）
    // 实例代号更高而 txg 更低同在为 0，或同 txg 不同实例同在不为 0（摆根没照落点）。
    let domain_is_nonempty = if domain.is_a_legal_history() {
        total.count_of(RootChoiceCounter::ConfirmedStates) > 0
    } else {
        total.count_of(RootChoiceCounter::HigherInstanceLowerTxgRoots) > 0
            && total.count_of(RootChoiceCounter::SameTxgDifferentInstanceRoots) == 0
    };
    if !domain_is_nonempty {
        findings.void_conditions.push(format!(
            "TV10 {} 域是空的（合法历史域没有一个已确认发布的态；KL 上实例代号更高而 txg 更低同在为 0，或同 txg 不同实例同在不为 0），这个域的 0 违例不算数",
            domain.label()
        ));
    }
    // 登记停机条款 TH4：读槽对拍不一致，不当 KJ 的违例记。
    if total.count_of(RootChoiceCounter::SlotReadingsDifferingFromTheTruth) > 0 {
        findings.halt_conditions.push(format!(
            "TH4 {} 域里 read_root_ring_slot 交回的与生成时的真相不同的槽数 {}（见证 {}）",
            domain.label(),
            total.count_of(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
            total.witness(RootChoiceCounter::SlotReadingsDifferingFromTheTruth)
        ));
    }
    // 登记第七节甲类、失败条款 TF9：合法历史三个域里今天的 choose_root 交回空、replay_journal 报错的态数是 0。
    if domain.is_a_legal_history()
        && total.count_of(RootChoiceCounter::ChoseNothingStates)
            + total.count_of(RootChoiceCounter::ReplayFailedStates)
            > 0
    {
        findings.halt_conditions.push(format!(
            "TF9 {} 域里 choose_root 交回空的态数 {}、replay_journal 报错的态数 {}（登记第七节甲类锚点都是 0）",
            domain.label(),
            total.count_of(RootChoiceCounter::ChoseNothingStates),
            total.count_of(RootChoiceCounter::ReplayFailedStates)
        ));
    }
    emitter.emit_result_line(&format!(
        "name=verdict domain={} ring_slots={} slots_per_region={} largest_mounts={largest_mounts} largest_user_publishes={largest_user_publishes} criteria=root_choice states={} confirmed_states={} kj0_violations={} panicked={} choose_root_returned_none={} replay_failed={} kj1_violations={} kj1_judged={} kj2_violations={} kj2_judged={} kj3_violations={} kj3_judged={} slot_readings_differing_from_the_truth={} domain_is_nonempty={domain_is_nonempty}",
        domain.label(),
        domain.ring().ring_slots,
        domain.slots_per_region(),
        total.count_of(RootChoiceCounter::States),
        total.count_of(RootChoiceCounter::ConfirmedStates),
        total.count_of(RootChoiceCounter::Kj0Violations),
        total.count_of(RootChoiceCounter::PanickedStates),
        total.count_of(RootChoiceCounter::ChoseNothingStates),
        total.count_of(RootChoiceCounter::ReplayFailedStates),
        total.count_of(RootChoiceCounter::Kj1Violations),
        total.count_of(RootChoiceCounter::Kj1Judged),
        total.count_of(RootChoiceCounter::Kj2Violations),
        total.count_of(RootChoiceCounter::Kj2Judged),
        total.count_of(RootChoiceCounter::Kj3Violations),
        total.count_of(RootChoiceCounter::Kj3Judged),
        total.count_of(RootChoiceCounter::SlotReadingsDifferingFromTheTruth),
    ));
}

// ── 命令行与主流程（`--family root_choice`，只在 `--profile r3`）

pub(super) struct RootChoiceSelection {
    /// 这一趟判的登记界，按次序逐段判（[`bounds_to_judge_of_the_option`]）。
    pub(super) bounds_to_judge: Vec<BoundsTier>,
    pub(super) only: Option<Vec<String>>,
    pub(super) largest_user_publishes_cap: Option<usize>,
}

const ROOT_CHOICE_FAMILY_LABEL: &str = "root_choice";

pub(super) fn arguments_name_the_root_choice_family(arguments: &[String]) -> bool {
    arguments
        .windows(2)
        .any(|pair| pair[0] == "--family" && pair[1] == ROOT_CHOICE_FAMILY_LABEL)
}

/// `--bounds segment1|segment2`（这一族 segment1 与 segment2 同界；不给是不指定范围，两段都判，[`bounds_to_judge_of_the_option`]）、`--family root_choice`、
/// `--only K,K9,KW,KL`（指定范围）、`--max-u <U>`（每个格的 U 封顶，缩域）。
pub(super) fn parse_the_root_choice_command_line(
    arguments: &[String],
) -> Result<RootChoiceSelection, String> {
    let mut bounds: Option<BoundsTier> = None;
    let mut only: Option<Vec<String>> = None;
    let mut largest_user_publishes_cap: Option<usize> = None;
    let mut position = 0;
    while position < arguments.len() {
        let option = arguments[position].as_str();
        let value = arguments
            .get(position + 1)
            .ok_or_else(|| format!("{option} 后面缺一个值"))?
            .as_str();
        match option {
            "--bounds" => {
                bounds = Some(match value {
                    "segment1" => BoundsTier::SegmentOne,
                    "segment2" => BoundsTier::SegmentTwo,
                    unknown_tier => return Err(format!("不认识的界：{unknown_tier}")),
                });
            }
            "--family" => {
                if value != ROOT_CHOICE_FAMILY_LABEL {
                    return Err(format!("不认识的族：{value}"));
                }
            }
            "--only" => {
                let names: Vec<String> = value.split(',').map(str::to_string).collect();
                for name in &names {
                    if !RootChoiceDomain::ALL
                        .iter()
                        .any(|domain| domain.label() == name)
                    {
                        return Err(format!("不认识的域：{name}（这一族的域是 K、K9、KW、KL）"));
                    }
                }
                only = Some(names);
            }
            "--max-u" => {
                largest_user_publishes_cap = Some(
                    value
                        .parse::<usize>()
                        .map_err(|error| format!("--max-u 要一个非负整数：{error}"))?,
                );
            }
            unknown_option => return Err(format!("不认识的参数：{unknown_option}")),
        }
        position += 2;
    }
    Ok(RootChoiceSelection {
        bounds_to_judge: bounds_to_judge_of_the_option(bounds),
        only,
        largest_user_publishes_cap,
    })
}

pub(super) fn run_the_root_choice_family(arguments: &[String], run_profile: RunProfile) {
    let selection = match parse_the_root_choice_command_line(arguments) {
        Ok(selection) => selection,
        Err(problem) => {
            eprintln!("  ✗ {problem}");
            eprintln!("  → 怎么办：{USAGE}");
            std::process::exit(2);
        }
    };
    install_the_panic_hook_that_stays_quiet_inside_the_replay_call();
    let outcomes: Vec<OutcomeOfTheBounds> = selection
        .bounds_to_judge
        .iter()
        .map(|bounds| run_the_root_choice_family_under_the_bounds(&selection, *bounds, run_profile))
        .collect();
    exit_when_some_bounds_ended_void_or_halted(&outcomes);
}

/// 择根一族在一段登记界下跑一趟：配置行、自检、阳性对照、各域、判定行都照这一段的界（[`run_the_root_choice_family`] 逐段调它）。
fn run_the_root_choice_family_under_the_bounds(
    selection: &RootChoiceSelection,
    bounds: BoundsTier,
    run_profile: RunProfile,
) -> OutcomeOfTheBounds {
    let thread_count = thread_count_from_the_environment();
    let run_started = Instant::now();
    let mut emitter = Emitter { emitted: 0 };
    let mut findings = RunFindings::default();
    let selected_cells: Vec<(RootChoiceDomain, Vec<RootChoiceCell>)> = RootChoiceDomain::ALL
        .iter()
        .copied()
        .filter(|domain| {
            selection
                .only
                .as_ref()
                .is_none_or(|names| names.iter().any(|name| name == domain.label()))
        })
        .map(|domain| {
            (
                domain,
                root_choice_cells_of(domain, bounds, selection.largest_user_publishes_cap),
            )
        })
        .collect();
    emitter.emit_result_line(&format!(
        "name=config experiment=E165 bounds={} family={ROOT_CHOICE_FAMILY_LABEL} only={} max_u={} root_slot_bytes={ROOT_CHOICE_ROOT_SLOT_BYTES} region_devices=0,1,0 slots_per_region_wide={ROOT_CHOICE_SLOTS_PER_REGION_WIDE} slots_per_region_narrow={ROOT_CHOICE_SLOTS_PER_REGION_NARROW}{}{}",
        bounds.label(),
        selection
            .only
            .as_ref()
            .map_or_else(|| "every_domain".to_string(), |names| names.join(",")),
        selection
            .largest_user_publishes_cap
            .map_or_else(|| "none".to_string(), |cap| cap.to_string()),
        profile_fields_of_the_run_rows(run_profile),
        narrowing_field_of_the_configuration_line(selection.largest_user_publishes_cap, "max_u"),
    ));

    // ① 常量回比断言（不过当场 panic，进程以非 0 退出，一行结果都不算）。
    cross_check_root_choice_local_constants_against_the_implementation();
    emitter.emit_result_line(
        "name=self_check check=root_choice_local_constants_equal_the_implementation passed=true",
    );
    let self_checks_passed = 1;
    for ring in [RingGeometry::BIG, RingGeometry::SMALL_NINE] {
        emitter.emit_result_line(&format!(
            "name=in_flight_limit ring_slots={} local={} implementation={}",
            ring.ring_slots,
            ring.local_in_flight_limit(),
            journal_in_flight_record_limit(ring.ring_bytes())
        ));
    }

    // ② 阳性对照：每条判据都跑；有一行没红，这一段作废（V2）。
    let controls_started = Instant::now();
    let control_readings = root_choice_control_readings(&selected_cells, thread_count);
    for reading in &control_readings {
        emitter.emit_result_line(&format!(
            "name=control criterion={} measured={} value={} must_see={} red_as_required={}",
            reading.criterion.label(),
            reading.measured_where.replace(' ', ","),
            reading.value,
            reading.must_see,
            reading.is_red_as_required
        ));
        if !reading.is_red_as_required {
            findings.void_conditions.push(format!(
                "V2 判据 {} 的阳性对照没红：{} 量到 {}，必须 {}",
                reading.criterion.label(),
                reading.measured_where,
                reading.value,
                reading.must_see
            ));
        }
    }
    println!(
        "E7TIMING name=elapsed scope=positive_controls elapsed_seconds={:.3} threads={thread_count}",
        controls_started.elapsed().as_secs_f64()
    );

    // ③ 各域各格：今天的三步。
    let mut total_states: u64 = 0;
    for (domain, cells) in &selected_cells {
        let mut domain_total = RootChoiceTally::new();
        for cell in cells {
            let cell_started = Instant::now();
            let tally = root_choice_tally_of_a_cell(
                *domain,
                *cell,
                RootChoiceImplementation::TodayThreeSteps,
                thread_count,
            );
            if domain.is_a_legal_history() {
                assert_the_pinned_root_choice_cell(*domain, *cell, &tally);
            } else {
                assert_the_pinned_landing_slot_adversarial_shape(&tally);
            }
            emit_the_rows_of_a_root_choice_cell(&mut emitter, *domain, *cell, &tally);
            println!(
                "E7TIMING name=elapsed scope=cell domain={} mounts={} user_publishes={} states={} elapsed_seconds={:.3} threads={thread_count}",
                domain.label(),
                cell.mounts,
                cell.user_publishes,
                tally.count_of(RootChoiceCounter::States),
                cell_started.elapsed().as_secs_f64()
            );
            domain_total.merge(&tally);
        }
        total_states += domain_total.count_of(RootChoiceCounter::States);
        emit_the_verdict_of_a_root_choice_domain(
            &mut emitter,
            &mut findings,
            *domain,
            cells,
            &domain_total,
        );
    }

    for condition in &findings.void_conditions {
        emitter.emit_result_line(&format!(
            "name=void condition={}",
            condition.replace(' ', "_")
        ));
    }
    for condition in &findings.halt_conditions {
        emitter.emit_result_line(&format!(
            "name=halt condition={}",
            condition.replace(' ', "_")
        ));
    }
    emitter.emit_result_line(&format!(
        "name=verdict scope=run bounds={} family={ROOT_CHOICE_FAMILY_LABEL} states={total_states} self_checks_passed={self_checks_passed} control_readings={} controls_not_red={} void_conditions={} halt_conditions={}{}",
        bounds.label(),
        control_readings.len(),
        control_readings
            .iter()
            .filter(|reading| !reading.is_red_as_required)
            .count(),
        findings.void_conditions.len(),
        findings.halt_conditions.len(),
        profile_fields_of_the_run_rows(run_profile),
    ));
    println!(
        "E7TIMING name=elapsed scope=run states={total_states} elapsed_seconds={:.3} threads={thread_count}",
        run_started.elapsed().as_secs_f64()
    );
    emitter.done();
    OutcomeOfTheBounds::of_the_findings(&findings)
}

#[cfg(test)]
mod tests;

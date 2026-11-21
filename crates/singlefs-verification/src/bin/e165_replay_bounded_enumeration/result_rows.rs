//! 产物行：`Emitter`、落后类的行、每个 (域, N) 的计数行与每个域的判决行。

use super::*;

// ── 产物行

/// 逐行打 `E7RESULT`，数着打了几行（收尾行 `name=done emitted=N` 把自己也算进去）。
pub(super) struct Emitter {
    pub(super) emitted: usize,
}

impl Emitter {
    pub(super) fn emit_result_line(&mut self, text: &str) {
        println!("E7RESULT {text}");
        self.emitted += 1;
    }

    pub(super) fn done(&mut self) {
        println!("E7RESULT name=done emitted={}", self.emitted + 1);
        self.emitted += 1;
    }
}

/// 跑完要交代的两类事：作废（登记第十一节 V 条）与停机（H 条、F11）。有一条，进程以非 0 退出。
#[derive(Default)]
pub(super) struct RunFindings {
    pub(super) void_conditions: Vec<String>,
    pub(super) halt_conditions: Vec<String>,
}

pub(super) fn witness_field(tally: &Tally, counter: Counter) -> String {
    tally.witness(counter).unwrap_or("none").to_string()
}

/// 落后类别的五行（登记 r2 SQ9、SQ17）：相等、落后（两个方向之和）、互不包含、0 号盘落后、1 号盘落后；每行列出
/// [`LAGGING_CLASS_COLUMNS`] 各量。`row_name` 是 `lagging_class`（每个 (域, N)）或 `verdict`（域的合计）。
fn lagging_class_rows(
    row_name: &str,
    place: &str,
    tally: &Tally,
    run_profile: RunProfile,
) -> Vec<String> {
    let shown_columns = lagging_class_columns_shown_under(run_profile);
    let groups: [(&str, &[LaggingClass]); 5] = [
        (LaggingClass::Equal.label(), &[LaggingClass::Equal]),
        (
            "lagging",
            &[
                LaggingClass::FirstDeviceBehind,
                LaggingClass::SecondDeviceBehind,
            ],
        ),
        (LaggingClass::Crossed.label(), &[LaggingClass::Crossed]),
        (
            LaggingClass::FirstDeviceBehind.label(),
            &[LaggingClass::FirstDeviceBehind],
        ),
        (
            LaggingClass::SecondDeviceBehind.label(),
            &[LaggingClass::SecondDeviceBehind],
        ),
    ];
    groups
        .iter()
        .map(|(group_label, classes)| {
            let columns: Vec<String> = shown_columns
                .iter()
                .map(|counter| {
                    let count: u64 = classes
                        .iter()
                        .map(|class| tally.lagging_class_count(*class, *counter))
                        .sum();
                    format!("{}={count}", counter.label())
                })
                .collect();
            format!(
                "name={row_name} {place} lagging_class={group_label} {}",
                columns.join(" ")
            )
        })
        .collect()
}

/// 一个 (域, N) 的全部行：状态数与见证、每条判据一行违例数、S 自己的对照、C 域的状态对、对抗域的 A-互。
pub(super) fn emit_the_rows_of_a_domain_size(
    emitter: &mut Emitter,
    domain: Domain,
    size: usize,
    tally: &Tally,
    run_profile: RunProfile,
) {
    let place = format!(
        "domain={} ring_slots={} n={size}",
        domain.label(),
        domain.ring().ring_slots
    );
    emitter.emit_result_line(&format!(
        "name=states {place} states={} panicked={} failed={} applied_ge1={} applied_ge2={} farthest_ge1={} farthest_ge2={} in_flight_limit={} above_water_max={} above_water_over_limit={} above_water_at_limit={} chosen_root_is_a_later_version={} off_timeline_landings={} states_holding_a_record_rejected_by_parse={}",
        tally.count_of(Counter::States),
        tally.count_of(Counter::Panicked),
        tally.count_of(Counter::Failed),
        tally.count_of(Counter::AppliedAtLeastOne),
        tally.count_of(Counter::AppliedAtLeastTwo),
        tally.count_of(Counter::FarthestAtLeastOne),
        tally.count_of(Counter::FarthestAtLeastTwo),
        domain.ring().local_in_flight_limit(),
        tally.above_water_maximum,
        tally.count_of(Counter::AboveWaterOverLimit),
        tally.count_of(Counter::AboveWaterAtLimit),
        tally.count_of(Counter::ChosenRootIsALaterVersion),
        tally.count_of(Counter::OffTimelineLandings),
        tally.count_of(Counter::StatesHoldingARecordRejectedByParse),
    ));
    let mut violations_row = |criterion: &str, violations: Counter, judged: u64| {
        emitter.emit_result_line(&format!(
            "name=violations {place} criterion={criterion} violations={} judged={judged} witness={}",
            tally.count_of(violations),
            witness_field(tally, violations)
        ));
    };
    let states = tally.count_of(Counter::States);
    match domain.family() {
        Family::LegalHistory | Family::MultiDisk => {
            violations_row("J0", Counter::Panicked, states);
            violations_row("J1", Counter::ReturnedRootViolations, states);
            violations_row("J2", Counter::RootPlusDiskViolations, states);
            violations_row("J3", Counter::InFlightLimitViolations, states);
            violations_row("J4", Counter::FarthestVersionViolations, states);
        }
        Family::Adversarial => {
            violations_row("A0", Counter::Panicked, states);
            violations_row("AS_whole_table", Counter::AsWholeTableViolations, states);
            violations_row(
                "AS_consecutive_gap_continues",
                Counter::AsGapContinuesViolations,
                states,
            );
            violations_row(
                "AS_consecutive_gap_stops",
                Counter::AsGapStopsViolations,
                states,
            );
            violations_row(
                "A_structure",
                Counter::StructureViolations,
                states - tally.count_of(Counter::Failed) - tally.count_of(Counter::Panicked),
            );
            violations_row(
                "A_limit",
                Counter::LimitViolations,
                states - tally.count_of(Counter::Failed) - tally.count_of(Counter::Panicked),
            );
        }
    }
    violations_row(
        "J6_premise_holds",
        Counter::FixedPointPremiseHoldsViolations,
        tally.count_of(Counter::FixedPointPremiseHoldsJudged),
    );
    violations_row(
        "J6_premise_fails",
        Counter::FixedPointPremiseFailsViolations,
        tally.count_of(Counter::FixedPointPremiseFailsJudged),
    );
    if run_profile.judges_j6qn() {
        violations_row(
            "J6Qn_premise_holds",
            Counter::FixedPointQnPremiseHoldsViolations,
            tally.count_of(Counter::FixedPointQnPremiseHoldsJudged),
        );
        violations_row(
            "J6Qn_premise_fails",
            Counter::FixedPointQnPremiseFailsViolations,
            tally.count_of(Counter::FixedPointQnPremiseFailsJudged),
        );
    }
    if run_profile.judges_copied_fields() {
        for field in FieldOfTheRebuiltRoot::ALL {
            violations_row(
                field.violations_counter().label(),
                field.violations_counter(),
                tally.count_of(Counter::CopiedFieldsJudged),
            );
        }
        violations_row(
            "J7b",
            Counter::J7bViolations,
            tally.count_of(Counter::J7bJudged),
        );
        violations_row(
            "J7c",
            Counter::J7cViolations,
            tally.count_of(Counter::J7cJudged),
        );
    }
    // r5 档（登记 r5 5.3）：J7 五行换成 J8 五行与「无出处」一行；「无出处」只报，它的 judged 是投影不等于所选根的态数。
    if run_profile.judges_new_root_segment_fields() {
        for counter in [
            Counter::J8aInstanceTableViolations,
            Counter::J8aAllocationRecordTreeRootViolations,
            Counter::J8aUnmountMarkerViolations,
        ] {
            violations_row(counter.label(), counter, tally.count_of(Counter::J8aJudged));
        }
        violations_row(
            "J8b",
            Counter::J8bViolations,
            tally.count_of(Counter::NewRootSegmentFieldsJudged),
        );
        violations_row(
            "J8c",
            Counter::J8cViolations,
            tally.count_of(Counter::J8cJudged),
        );
        violations_row(
            "J8_without_a_source",
            Counter::NewRootSegmentFieldsWithoutASource,
            tally.count_of(Counter::J8aJudged)
                + tally.count_of(Counter::NewRootSegmentFieldsWithoutASource),
        );
    }
    if tally.count_of(Counter::Ia1Judged) > 0 {
        violations_row(
            "IA1",
            Counter::Ia1Violations,
            tally.count_of(Counter::Ia1Judged),
        );
    }
    if tally.count_of(Counter::Ia2Judged) > 0 {
        violations_row(
            "IA2",
            Counter::Ia2Violations,
            tally.count_of(Counter::Ia2Judged),
        );
    }
    emitter.emit_result_line(&format!(
        "name=fixed_point {place} first_call_failed={} third_call_equals_second={} third_call_differs_or_is_missing={}",
        tally.count_of(Counter::FixedPointFirstCallFailed),
        tally.count_of(Counter::FixedPointThirdCallEqualsSecond),
        tally.count_of(Counter::FixedPointThirdCallDiffersOrIsMissing),
    ));
    if run_profile.judges_j6qn() {
        emitter.emit_result_line(&format!(
            "name=first_returned_root_txg {place} first_returned_root_txg_is_the_largest={}",
            tally.count_of(Counter::FirstReturnedRootTxgIsTheLargest),
        ));
    }
    if domain.specification_self_check() == SpecificationSelfCheck::Judged {
        for counter in [
            Counter::SpecificationWholeTableMismatch,
            Counter::SpecificationGapContinuesMismatch,
            Counter::SpecificationGapStopsMismatch,
        ] {
            emitter.emit_result_line(&format!(
                "name=specification_self_check {place} reading={} mismatches={} witness={}",
                counter.label(),
                tally.count_of(counter),
                witness_field(tally, counter)
            ));
        }
    }
    if !pinned_monotone_counts(domain).is_empty() {
        emitter.emit_result_line(&format!(
            "name=legal_crash_states {place} cut_own={} cut_with={} pair_states_without_an_interval={}",
            tally.count_of(Counter::LegalCrashStatesWhenTheRootWriteIsItsOwnSegment),
            tally.count_of(Counter::LegalCrashStatesWhenTheRootWriteSharesTheRecordSegment),
            tally.count_of(Counter::PairStatesWithoutAnInterval),
        ));
        for set in PairSet::ALL {
            emitter.emit_result_line(&format!(
                "name=pairs {place} set={} pairs={} today_j5a={} today_j5b={} ideal_j5a={} ideal_j5b={}",
                set.label(),
                tally.pair(*set, PairColumn::Pairs),
                tally.pair(*set, PairColumn::TodayAppliedSetViolations),
                tally.pair(*set, PairColumn::TodayLandedVersionViolations),
                tally.pair(*set, PairColumn::IdealAppliedSetViolations),
                tally.pair(*set, PairColumn::IdealLandedVersionViolations),
            ));
        }
    }
    if !pinned_mirrored_counts(domain).is_empty() {
        for row in lagging_class_rows("lagging_class", &place, tally, run_profile) {
            emitter.emit_result_line(&row);
        }
    }
    if !pinned_mirrored_monotone_counts(domain).is_empty() {
        emitter.emit_result_line(&format!(
            "name=pair_states {place} pair_states_without_an_interval={}",
            tally.count_of(Counter::PairStatesWithoutAnInterval),
        ));
        emitter.emit_result_line(&format!(
            "name=pairs {place} set={} pairs={} today_j5a={} today_j5b={} ideal_j5a={} ideal_j5b={}",
            PairSet::Wide.label(),
            tally.pair(PairSet::Wide, PairColumn::Pairs),
            tally.pair(PairSet::Wide, PairColumn::TodayAppliedSetViolations),
            tally.pair(PairSet::Wide, PairColumn::TodayLandedVersionViolations),
            tally.pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations),
            tally.pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations),
        ));
    }
    if run_profile.compares_the_two_record_tables() {
        let reported_order_fields = match run_profile.reported_disk_order {
            ReportedDiskOrder::FirstDeviceBeforeSecond => String::new(),
            ReportedDiskOrder::SecondDeviceBeforeFirst => format!(
                " tables_differ_in_reported_order={} rows_differ_in_reported_order={} witness_in_reported_order={} rows_witness_in_reported_order={}",
                tally.count_of(Counter::TablesDifferInReportedOrder),
                tally.count_of(Counter::RowsDifferInReportedOrder),
                witness_field(tally, Counter::TablesDifferInReportedOrder),
                witness_field(tally, Counter::RowsDifferInReportedOrder),
            ),
        };
        emitter.emit_result_line(&format!(
            "name=table_comparison {place} states={} tables_differ={} rows_differ={} direct_table_holds_a_record={} fully_landed_scan_table_size_checked={} witness={} rows_witness={}{reported_order_fields}",
            tally.count_of(Counter::States),
            tally.count_of(Counter::TablesDiffer),
            tally.count_of(Counter::RowsDiffer),
            tally.count_of(Counter::DirectTableHoldsARecord),
            tally.count_of(Counter::FullyLandedScanTableSizeChecked),
            witness_field(tally, Counter::TablesDiffer),
            witness_field(tally, Counter::RowsDiffer),
        ));
    }
    for row in rows_of_the_txg_order_split(run_profile, domain, &place, tally) {
        emitter.emit_result_line(&row);
    }
    if domain.family() == Family::Adversarial {
        emitter.emit_result_line(&format!(
            "name=specification {place} expects_failure={}",
            tally.count_of(Counter::SpecificationExpectsFailure)
        ));
        for counter in [
            Counter::MutualWholeTableAndGapContinues,
            Counter::MutualWholeTableAndGapStops,
            Counter::MutualGapContinuesAndGapStops,
        ] {
            emitter.emit_result_line(&format!(
                "name=mutual {place} readings={} differing={} witness={}",
                counter.label(),
                tally.count_of(counter),
                witness_field(tally, counter)
            ));
        }
    }
}

/// 一个域扫完之后的判决行：各判据在这个域全部 N 上的合计。也在这里记作废与停机。
pub(super) fn emit_the_verdict_of_a_domain(
    emitter: &mut Emitter,
    findings: &mut RunFindings,
    domain: Domain,
    largest_size: usize,
    total: &Tally,
    run_profile: RunProfile,
) {
    let place = format!(
        "domain={} ring_slots={} smallest_n={} largest_n={largest_size}",
        domain.label(),
        domain.ring().ring_slots,
        domain.smallest_size()
    );
    // 域非空见证（登记 Q12、作废条款 V4）：实现施加过至少一次发布的态数必须大于 0，E2 除外。
    let domain_is_nonempty =
        total.count_of(Counter::AppliedAtLeastOne) > 0 || domain == Domain::TxgAtTheLargest;
    if !domain_is_nonempty {
        findings.void_conditions.push(format!(
            "V4 {} 域没有一个态施加过发布，这个域的 0 违例不算数",
            domain.label()
        ));
    }
    let specification_mismatches = total.count_of(Counter::SpecificationWholeTableMismatch)
        + total.count_of(Counter::SpecificationGapContinuesMismatch)
        + total.count_of(Counter::SpecificationGapStopsMismatch);
    if specification_mismatches > 0 {
        findings.void_conditions.push(format!(
            "V3 {} 域里 S 的期望落点不等于 j* 的态数（三种读法合计）{specification_mismatches}",
            domain.label()
        ));
    }
    match domain.family() {
        Family::LegalHistory | Family::MultiDisk => {
            // 登记 r2 第七节甲类、SF9：M、M6、MC 里今天的实现第一次调用报错的态数是 0；不符停下，两边都查。
            if domain.family() == Family::MultiDisk && total.count_of(Counter::Failed) > 0 {
                findings.halt_conditions.push(format!(
                    "SF9 {} 域里今天的实现第一次调用报错的态数 {}（登记 r2 第七节甲类锚点是 0）",
                    domain.label(),
                    total.count_of(Counter::Failed)
                ));
            }
            // 甲类锚点（登记第七节）：L、L 小环、U 开着验证、X、E1 里原实现报错的态数是 0；不符走 F11，停下两边都查。
            let failed_states_are_pinned_to_zero =
                domain.specification_self_check() == SpecificationSelfCheck::Judged;
            if failed_states_are_pinned_to_zero && total.count_of(Counter::Failed) > 0 {
                findings.halt_conditions.push(format!(
                    "F11 {} 域里原实现报错的态数 {}（登记第七节甲类锚点是 0）",
                    domain.label(),
                    total.count_of(Counter::Failed)
                ));
            }
            emitter.emit_result_line(&format!(
                "name=verdict {place} states={} j0_violations={} failed_states={} j1_violations={} j2_violations={} j3_violations={} j4_violations={} j6_premise_holds_violations={} j6_premise_fails_violations={} j6_premise_fails_judged={} ia1_violations={} ia2_violations={} specification_self_check_mismatches={specification_mismatches} applied_ge1={} applied_ge2={} domain_is_nonempty={domain_is_nonempty}",
                total.count_of(Counter::States),
                total.count_of(Counter::Panicked),
                total.count_of(Counter::Failed),
                total.count_of(Counter::ReturnedRootViolations),
                total.count_of(Counter::RootPlusDiskViolations),
                total.count_of(Counter::InFlightLimitViolations),
                total.count_of(Counter::FarthestVersionViolations),
                total.count_of(Counter::FixedPointPremiseHoldsViolations),
                total.count_of(Counter::FixedPointPremiseFailsViolations),
                total.count_of(Counter::FixedPointPremiseFailsJudged),
                total.count_of(Counter::Ia1Violations),
                total.count_of(Counter::Ia2Violations),
                total.count_of(Counter::AppliedAtLeastOne),
                total.count_of(Counter::AppliedAtLeastTwo),
            ));
            if !pinned_monotone_counts(domain).is_empty() {
                for set in PairSet::ALL {
                    emitter.emit_result_line(&format!(
                        "name=verdict {place} set={} pairs={} today_j5a_violations={} today_j5b_violations={} ideal_j5a_violations={} ideal_j5b_violations={}",
                        set.label(),
                        total.pair(*set, PairColumn::Pairs),
                        total.pair(*set, PairColumn::TodayAppliedSetViolations),
                        total.pair(*set, PairColumn::TodayLandedVersionViolations),
                        total.pair(*set, PairColumn::IdealAppliedSetViolations),
                        total.pair(*set, PairColumn::IdealLandedVersionViolations),
                    ));
                }
            }
            if !pinned_mirrored_counts(domain).is_empty() {
                for row in lagging_class_rows("verdict", &place, total, run_profile) {
                    emitter.emit_result_line(&row);
                }
            }
            if !pinned_mirrored_monotone_counts(domain).is_empty() {
                emitter.emit_result_line(&format!(
                    "name=verdict {place} set={} pairs={} today_j5a_violations={} today_j5b_violations={} ideal_j5a_violations={} ideal_j5b_violations={}",
                    PairSet::Wide.label(),
                    total.pair(PairSet::Wide, PairColumn::Pairs),
                    total.pair(PairSet::Wide, PairColumn::TodayAppliedSetViolations),
                    total.pair(PairSet::Wide, PairColumn::TodayLandedVersionViolations),
                    total.pair(PairSet::Wide, PairColumn::IdealAppliedSetViolations),
                    total.pair(PairSet::Wide, PairColumn::IdealLandedVersionViolations),
                ));
            }
        }
        Family::Adversarial => {
            emitter.emit_result_line(&format!(
                "name=verdict {place} states={} a0_violations={} failed_states={} as_whole_table_violations={} as_consecutive_gap_continues_violations={} as_consecutive_gap_stops_violations={} readings_differing_whole_table_vs_consecutive_gap_continues={} readings_differing_whole_table_vs_consecutive_gap_stops={} readings_differing_consecutive_gap_continues_vs_consecutive_gap_stops={} a_structure_violations={} a_limit_violations={} j6_premise_holds_violations={} j6_premise_fails_violations={} j6_premise_fails_judged={} applied_ge1={} states_holding_a_record_rejected_by_parse={} domain_is_nonempty={domain_is_nonempty}",
                total.count_of(Counter::States),
                total.count_of(Counter::Panicked),
                total.count_of(Counter::Failed),
                total.count_of(Counter::AsWholeTableViolations),
                total.count_of(Counter::AsGapContinuesViolations),
                total.count_of(Counter::AsGapStopsViolations),
                total.count_of(Counter::MutualWholeTableAndGapContinues),
                total.count_of(Counter::MutualWholeTableAndGapStops),
                total.count_of(Counter::MutualGapContinuesAndGapStops),
                total.count_of(Counter::StructureViolations),
                total.count_of(Counter::LimitViolations),
                total.count_of(Counter::FixedPointPremiseHoldsViolations),
                total.count_of(Counter::FixedPointPremiseFailsViolations),
                total.count_of(Counter::FixedPointPremiseFailsJudged),
                total.count_of(Counter::AppliedAtLeastOne),
                total.count_of(Counter::StatesHoldingARecordRejectedByParse),
            ));
        }
    }
    if run_profile.judges_j6qn() {
        let first_returned_root_txg_is_the_largest =
            total.count_of(Counter::FirstReturnedRootTxgIsTheLargest);
        // 登记 r2 作废条款 SV4：E1 上 ρ_1 的 txg 是 u64 最大值的态数为 0，J6Qn 在 E1 上的 0 不算「Qn 管住了顶格」。
        if domain == Domain::TxgOneBelowTheLargest && first_returned_root_txg_is_the_largest == 0 {
            findings.void_conditions.push(
                "SV4 E1 域上第一次返回的根 txg 是 u64 最大值的态数为 0，J6Qn 在 E1 上的 0 不算数"
                    .to_string(),
            );
        }
    }
    if run_profile.judges_copied_fields() {
        let first_returned_root_txg_is_the_largest =
            total.count_of(Counter::FirstReturnedRootTxgIsTheLargest);
        emitter.emit_result_line(&format!(
            "name=verdict {place} criteria=r2 j6qn_premise_holds_violations={} j6qn_premise_holds_judged={} j6qn_premise_fails_violations={} j6qn_premise_fails_judged={} j7_judged={} j7a_instance_table_violations={} j7a_allocation_record_tree_root_violations={} j7a_filesystem_identifier_violations={} j7b_violations={} j7b_judged={} j7c_violations={} j7c_judged={} first_returned_root_txg_is_the_largest={first_returned_root_txg_is_the_largest}",
            total.count_of(Counter::FixedPointQnPremiseHoldsViolations),
            total.count_of(Counter::FixedPointQnPremiseHoldsJudged),
            total.count_of(Counter::FixedPointQnPremiseFailsViolations),
            total.count_of(Counter::FixedPointQnPremiseFailsJudged),
            total.count_of(Counter::CopiedFieldsJudged),
            total.count_of(Counter::J7aInstanceTableViolations),
            total.count_of(Counter::J7aAllocationRecordTreeRootViolations),
            total.count_of(Counter::J7aFilesystemIdentifierViolations),
            total.count_of(Counter::J7bViolations),
            total.count_of(Counter::J7bJudged),
            total.count_of(Counter::J7cViolations),
            total.count_of(Counter::J7cJudged),
        ));
    }
    if run_profile.judges_new_root_segment_fields() {
        emitter.emit_result_line(&format!(
            "name=verdict {place} criteria=r5 j6qn_premise_holds_violations={} j6qn_premise_holds_judged={} j6qn_premise_fails_violations={} j6qn_premise_fails_judged={} j8_judged={} j8a_judged={} j8a_instance_table_violations={} j8a_allocation_record_tree_root_violations={} j8a_unmount_marker_violations={} j8b_violations={} j8b_judged={} j8c_violations={} j8c_judged={} j8_without_a_source={} first_returned_root_txg_is_the_largest={}",
            total.count_of(Counter::FixedPointQnPremiseHoldsViolations),
            total.count_of(Counter::FixedPointQnPremiseHoldsJudged),
            total.count_of(Counter::FixedPointQnPremiseFailsViolations),
            total.count_of(Counter::FixedPointQnPremiseFailsJudged),
            total.count_of(Counter::NewRootSegmentFieldsJudged),
            total.count_of(Counter::J8aJudged),
            total.count_of(Counter::J8aInstanceTableViolations),
            total.count_of(Counter::J8aAllocationRecordTreeRootViolations),
            total.count_of(Counter::J8aUnmountMarkerViolations),
            total.count_of(Counter::J8bViolations),
            total.count_of(Counter::NewRootSegmentFieldsJudged),
            total.count_of(Counter::J8cViolations),
            total.count_of(Counter::J8cJudged),
            total.count_of(Counter::NewRootSegmentFieldsWithoutASource),
            total.count_of(Counter::FirstReturnedRootTxgIsTheLargest),
        ));
    }
    if run_profile.splits_j6qn_by_the_txg_order() && domain.family() == Family::Adversarial {
        emitter.emit_result_line(&verdict_row_of_the_txg_order_split(&place, total));
    }
    if run_profile.compares_the_two_record_tables() {
        let reported_order_fields = match run_profile.reported_disk_order {
            ReportedDiskOrder::FirstDeviceBeforeSecond => String::new(),
            ReportedDiskOrder::SecondDeviceBeforeFirst => format!(
                " tables_differ_in_reported_order={} rows_differ_in_reported_order={}",
                total.count_of(Counter::TablesDifferInReportedOrder),
                total.count_of(Counter::RowsDifferInReportedOrder),
            ),
        };
        emitter.emit_result_line(&format!(
            "name=verdict {place} criteria=r2_record_tables tables_differ={} rows_differ={}{reported_order_fields}",
            total.count_of(Counter::TablesDiffer),
            total.count_of(Counter::RowsDiffer),
        ));
    }
}

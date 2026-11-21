use super::*;

// ============================================================================
// 十二、合计与产物行（登记第六节「判定写法」：每组每个量各报一行；第八节轨迹三样；第十节、第十一节逐条结论）。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ArmError {
    TodayE1,
    TodayE2,
    WithPointerBirthE1,
    WithPointerBirthE2,
}

pub(super) const ARM_ERRORS: [ArmError; 4] = [
    ArmError::TodayE1,
    ArmError::TodayE2,
    ArmError::WithPointerBirthE1,
    ArmError::WithPointerBirthE2,
];

impl ArmError {
    pub(super) fn arm(self) -> &'static str {
        match self {
            ArmError::TodayE1 | ArmError::TodayE2 => "today",
            ArmError::WithPointerBirthE1 | ArmError::WithPointerBirthE2 => "with_pointer_birth",
        }
    }

    pub(super) fn error(self) -> &'static str {
        match self {
            ArmError::TodayE1 | ArmError::WithPointerBirthE1 => "E1",
            ArmError::TodayE2 | ArmError::WithPointerBirthE2 => "E2",
        }
    }

    pub(super) fn of(self, counts: &ArmCounts) -> u64 {
        match self {
            ArmError::TodayE1 => counts.today_e1,
            ArmError::TodayE2 => counts.today_e2,
            ArmError::WithPointerBirthE1 => counts.with_pointer_birth_e1,
            ArmError::WithPointerBirthE2 => counts.with_pointer_birth_e2,
        }
    }
}

/// 一组（组 × 取样点）的合计。`only_swept` 时只合计清扫发成了的那几段（Q6、Q9、Q10、Q11 的读法，登记 F6）。
#[derive(Clone, Debug, Default)]
pub(super) struct GroupTotals {
    pub(super) histories: u64,
    pub(super) produced_images: u64,
    stopped_histories: u64,
    pub(super) judged_images: u64,
    pub(super) unreadable_images: u64,
    pub(super) hole_pairs: u64,
    pub(super) truth_counts: BTreeMap<Truth, u64>,
    pub(super) pointer_unreadable_pairs: u64,
    pub(super) pairs: ArmCounts,
    pub(super) images_with_a_positive_count: ArmCounts,
    first_image_with_a_positive_count: BTreeMap<&'static str, String>,
    pub(super) release_generation_verdicts: BTreeMap<String, u64>,
    pub(super) reference_interval_verdicts: BTreeMap<String, u64>,
    pub(super) swept_histories: u64,
    pub(super) sweep_txg_matches_the_prediction: u64,
    pub(super) first_version_violations: u64,
    pub(super) adjacent_version_violations: u64,
    pub(super) birth_after_the_txg_violations: u64,
    pub(super) instance_one_first_version_mismatches: u64,
    pub(super) sweep_pointer_changes: u64,
}

pub(super) fn history_swept(result: &HistoryResult) -> bool {
    result
        .sweep
        .as_ref()
        .is_some_and(|sweep| sweep.sweep_version.is_some())
}

pub(super) fn totals_of(results: &[&HistoryResult], only_swept: bool) -> GroupTotals {
    let mut totals = GroupTotals::default();
    for result in results {
        if only_swept && !history_swept(result) {
            continue;
        }
        totals.histories += 1;
        totals.stopped_histories += u64::from(result.stopped.is_some());
        totals.first_version_violations +=
            u64::try_from(result.premise.first_version_violations.len()).expect("条数装得进 u64");
        totals.adjacent_version_violations +=
            u64::try_from(result.premise.adjacent_version_violations.len())
                .expect("条数装得进 u64");
        totals.birth_after_the_txg_violations +=
            u64::try_from(result.premise.birth_after_the_txg_violations.len())
                .expect("条数装得进 u64");
        totals.instance_one_first_version_mismatches +=
            result.premise.instance_one_first_version_mismatches;
        totals.sweep_pointer_changes += result.premise.sweep_pointer_changes;
        if let Some(sweep) = &result.sweep {
            if let Some(version) = sweep.sweep_version {
                totals.swept_histories += 1;
                totals.sweep_txg_matches_the_prediction +=
                    u64::from(version.txg == sweep.predicted_sweep_txg);
            }
        }
        for image in &result.images {
            totals.produced_images += 1;
            if image.reading.is_err() {
                totals.unreadable_images += 1;
                continue;
            }
            totals.judged_images += 1;
            totals.hole_pairs += u64::try_from(image.truths.len()).expect("对数装得进 u64");
            for truth in &image.truths {
                *totals.truth_counts.entry(*truth).or_insert(0) += 1;
            }
            totals.pointer_unreadable_pairs += image.pointer_unreadable_pairs;
            totals.pairs.add(image.counts);
            for arm_error in ARM_ERRORS {
                if arm_error.of(&image.counts) > 0 {
                    let mut one = ArmCounts::default();
                    match arm_error {
                        ArmError::TodayE1 => one.today_e1 = 1,
                        ArmError::TodayE2 => one.today_e2 = 1,
                        ArmError::WithPointerBirthE1 => one.with_pointer_birth_e1 = 1,
                        ArmError::WithPointerBirthE2 => one.with_pointer_birth_e2 = 1,
                    }
                    totals.images_with_a_positive_count.add(one);
                    let key = match arm_error {
                        ArmError::TodayE1 => "today_E1",
                        ArmError::TodayE2 => "today_E2",
                        ArmError::WithPointerBirthE1 => "with_pointer_birth_E1",
                        ArmError::WithPointerBirthE2 => "with_pointer_birth_E2",
                    };
                    totals
                        .first_image_with_a_positive_count
                        .entry(key)
                        .or_insert_with(|| format!("{}/{}", result.name, image.label));
                }
            }
            if let Some((release_generation_verdict, reference_interval_verdict)) =
                &image.checker_verdicts
            {
                *totals
                    .release_generation_verdicts
                    .entry(release_generation_verdict.clone())
                    .or_insert(0) += 1;
                *totals
                    .reference_interval_verdicts
                    .entry(reference_interval_verdict.clone())
                    .or_insert(0) += 1;
            }
        }
    }
    totals
}

pub(super) fn count_text(counts: &BTreeMap<String, u64>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }
    counts
        .iter()
        .map(|(name, count)| format!("{name}:{count}"))
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn truth_count(counts: &BTreeMap<Truth, u64>, truth: Truth) -> u64 {
    counts.get(&truth).copied().unwrap_or(0)
}

fn keys_text(keys: impl Iterator<Item = VersionKey>) -> String {
    let texts: Vec<String> = keys.map(VersionKey::text).collect();
    if texts.is_empty() {
        "none".to_string()
    } else {
        texts.join(",")
    }
}

pub(super) fn pointer_birth_text(pointer_birth: PointerBirth) -> String {
    match pointer_birth {
        PointerBirth::Readable(txg) => txg.to_string(),
        PointerBirth::AllZeroOrUnreadable => "unreadable".to_string(),
    }
}

/// 一张镜像一行：最新根、F 生效值、候选集、由记录施加出来的版本、每个洞对（R⁻ 那一组 > R：真值：两条臂：R 的出生 txg）与四个计数。
pub(super) fn image_line(result: &HistoryResult, image: &ImageResult) -> String {
    let prefix = format!(
        "name=image group={} point={} history={} image={} step={}",
        result.specification.group.name(),
        result.specification.point.name(),
        result.name,
        image.label,
        image.step
    );
    let checker = image
        .checker_verdicts
        .as_ref()
        .map_or("release_generation_verdict=not_run reference_interval_verdict=not_run".to_string(), |(release_generation_verdict, reference_interval_verdict)| {
            format!("release_generation_verdict={release_generation_verdict} reference_interval_verdict={reference_interval_verdict}")
        });
    match &image.reading {
        Err(reason) => format!("{prefix} unreadable={} {checker}", reason.replace(' ', "_")),
        Ok(reading) => {
            let pairs: Vec<String> = reading
                .hole_pairs
                .iter()
                .zip(&image.truths)
                .map(|(pair, truth)| {
                    format!(
                        "{}>{}:{}:today{}:with_pointer_birth{}:birth{}",
                        keys_text(pair.previous_group.iter().copied()),
                        pair.version.key.text(),
                        truth.name(),
                        u8::from(pair.bridged_today),
                        u8::from(pair.bridged_with_the_pointer_birth),
                        pointer_birth_text(pair.version.pointer_birth)
                    )
                })
                .collect();
            format!(
                "{prefix} newest={} floor={} candidates={} applied_only_by_records={} hole_pairs={} pairs={} today_e1={} today_e2={} with_pointer_birth_e1={} with_pointer_birth_e2={} pointer_unreadable_pairs={} corrupted_skipped={} {checker}",
                reading.newest.text(),
                reading.effective_floor,
                keys_text(reading.candidates.iter().map(|version| version.key)),
                keys_text(reading.applied_only_by_records.iter().map(|version| version.key)),
                reading.hole_pairs.len(),
                if pairs.is_empty() { "none".to_string() } else { pairs.join(";") },
                image.counts.today_e1,
                image.counts.today_e2,
                image.counts.with_pointer_birth_e1,
                image.counts.with_pointer_birth_e2,
                image.pointer_unreadable_pairs,
                image.corrupted_skipped.len()
            )
        }
    }
}

/// 一段历史一行：尝试与造出的张数、写者停在哪、清扫的读数、前提 P1–P3 的计数与逐条。
pub(super) fn history_lines(result: &HistoryResult) -> Vec<String> {
    let attempted = attempted_images(&result.specification)
        .map_or("unknown_before_the_run".to_string(), |count| {
            count.to_string()
        });
    let sweep = result.sweep.as_ref().map_or("not_a_sweep_history".to_string(), |sweep| {
        format!(
            "sweep_instance={} sweep_outcome={} sweep_txg={} predicted_sweep_txg={} last_txg_before_the_sweep_instance={} instances_mounted_before_the_sweep={}",
            sweep.sweep_instance,
            sweep.outcome,
            sweep.sweep_version.map_or("none".to_string(), |version| version.txg.to_string()),
            sweep.predicted_sweep_txg,
            sweep.last_txg_before_the_sweep_instance,
            sweep.instances_mounted_before_the_sweep
        )
    });
    let premise = &result.premise;
    let mut lines = vec![
        format!(
            "name=history group={} point={} history={} attempted_images={attempted} produced_images={} stopped={} versions={} {sweep}",
            result.specification.group.name(),
            result.specification.point.name(),
            result.name,
            result.images.len(),
            result.stopped.as_deref().map_or("none".to_string(), |reason| reason.replace(' ', "_")),
            result.ledger.versions.len()
        ),
        format!(
            "name=premise history={} first_version_violations={} adjacent_version_violations={} birth_after_the_txg_violations={} instance_one_first_version_mismatches={} sweep_pointer_changes={}",
            result.name,
            premise.first_version_violations.len(),
            premise.adjacent_version_violations.len(),
            premise.birth_after_the_txg_violations.len(),
            premise.instance_one_first_version_mismatches,
            premise.sweep_pointer_changes
        ),
    ];
    for (clause, details) in [
        ("P1", &premise.first_version_violations),
        ("P2", &premise.adjacent_version_violations),
        ("P3", &premise.birth_after_the_txg_violations),
    ] {
        for detail in details {
            lines.push(format!(
                "name=premise_violation history={} clause={clause} detail={}",
                result.name,
                detail.replace(' ', "_")
            ));
        }
    }
    lines
}

/// 每段历史的轨迹三样（登记第八节）：按步，每条臂每种错各报峰值、为正的步数、期末值。
pub(super) fn trajectory_of(result: &HistoryResult, arm_error: ArmError) -> (u64, u64, u64) {
    let values: Vec<u64> = result
        .images
        .iter()
        .filter(|image| image.reading.is_ok())
        .map(|image| arm_error.of(&image.counts))
        .collect();
    let peak = values.iter().copied().max().unwrap_or(0);
    let positive_steps =
        u64::try_from(values.iter().filter(|value| **value > 0).count()).expect("步数装得进 u64");
    let final_value = values.last().copied().unwrap_or(0);
    (peak, positive_steps, final_value)
}

pub(super) fn results_of(
    results: &[HistoryResult],
    group: Group,
    point: SamplingPoint,
) -> Vec<&HistoryResult> {
    results
        .iter()
        .filter(|result| result.specification.group == group && result.specification.point == point)
        .collect()
}

/// 一组一行合计、四条轨迹行；sw 三组另报只合计清扫发成了的那几段的一行（`scope=swept_histories`）。
pub(super) fn group_lines(
    results: &[HistoryResult],
    group: Group,
    point: SamplingPoint,
) -> Vec<String> {
    let members = results_of(results, group, point);
    if members.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let scopes: Vec<(&str, bool)> = if matches!(
        group,
        Group::SweepCrash | Group::SweepRead | Group::SweepNone
    ) {
        vec![("all_histories", false), ("swept_histories", true)]
    } else {
        vec![("all_histories", false)]
    };
    for (scope, only_swept) in scopes {
        let totals = totals_of(&members, only_swept);
        lines.push(format!(
            "name=group group={} point={} scope={scope} histories={} stopped_histories={} produced_images={} judged_images={} unreadable_images={} hole_pairs={} truth_directly_bridged={} truth_existed_versions_in_between={} truth_previous_not_on_the_chain={} truth_version_missing_from_the_ledger={} pointer_unreadable_pairs={} today_e1_pairs={} today_e1_images={} today_e2_pairs={} today_e2_images={} with_pointer_birth_e1_pairs={} with_pointer_birth_e1_images={} with_pointer_birth_e2_pairs={} with_pointer_birth_e2_images={} swept_histories={} sweep_txg_matches_the_prediction={} first_version_violations={} adjacent_version_violations={} birth_after_the_txg_violations={} instance_one_first_version_mismatches={} sweep_pointer_changes={} release_generation_verdict={} reference_interval_verdict={}",
            group.name(),
            point.name(),
            totals.histories,
            totals.stopped_histories,
            totals.produced_images,
            totals.judged_images,
            totals.unreadable_images,
            totals.hole_pairs,
            truth_count(&totals.truth_counts, Truth::DirectlyBridged),
            truth_count(&totals.truth_counts, Truth::ExistedVersionsInBetween),
            truth_count(&totals.truth_counts, Truth::PreviousNotOnTheChain),
            truth_count(&totals.truth_counts, Truth::VersionMissingFromTheLedger),
            totals.pointer_unreadable_pairs,
            totals.pairs.today_e1,
            totals.images_with_a_positive_count.today_e1,
            totals.pairs.today_e2,
            totals.images_with_a_positive_count.today_e2,
            totals.pairs.with_pointer_birth_e1,
            totals.images_with_a_positive_count.with_pointer_birth_e1,
            totals.pairs.with_pointer_birth_e2,
            totals.images_with_a_positive_count.with_pointer_birth_e2,
            totals.swept_histories,
            totals.sweep_txg_matches_the_prediction,
            totals.first_version_violations,
            totals.adjacent_version_violations,
            totals.birth_after_the_txg_violations,
            totals.instance_one_first_version_mismatches,
            totals.sweep_pointer_changes,
            count_text(&totals.release_generation_verdicts),
            count_text(&totals.reference_interval_verdicts)
        ));
        for (key, first) in &totals.first_image_with_a_positive_count {
            lines.push(format!(
                "name=first_positive_image group={} point={} scope={scope} count={key} image={first}",
                group.name(),
                point.name()
            ));
        }
        for arm_error in ARM_ERRORS {
            let mut peak_history = ("none".to_string(), 0);
            let mut positive_steps_total = 0;
            let mut histories_with_a_positive_final_value = 0;
            for result in members
                .iter()
                .filter(|result| !only_swept || history_swept(result))
            {
                let (peak, positive_steps, final_value) = trajectory_of(result, arm_error);
                if peak > peak_history.1 {
                    peak_history = (result.name.clone(), peak);
                }
                positive_steps_total += positive_steps;
                histories_with_a_positive_final_value += u64::from(final_value > 0);
            }
            lines.push(format!(
                "name=trajectory group={} point={} scope={scope} arm={} error={} peak={} peak_history={} positive_steps_total={positive_steps_total} histories_with_a_positive_final_value={histories_with_a_positive_final_value}",
                group.name(),
                point.name(),
                arm_error.arm(),
                arm_error.error(),
                peak_history.1,
                peak_history.0
            ));
        }
    }
    lines
}

/// 每段历史每条臂每种错的轨迹三样各一行。
pub(super) fn history_trajectory_lines(result: &HistoryResult) -> Vec<String> {
    ARM_ERRORS
        .iter()
        .map(|arm_error| {
            let (peak, positive_steps, final_value) = trajectory_of(result, *arm_error);
            format!(
                "name=history_trajectory history={} arm={} error={} peak={peak} positive_steps={positive_steps} final={final_value}",
                result.name,
                arm_error.arm(),
                arm_error.error()
            )
        })
        .collect()
}

pub(super) const GROUP_ORDER: [Group; 7] = [
    Group::NewestRowBad,
    Group::RollbackSlotBad,
    Group::ShortRing,
    Group::SkippedRow,
    Group::SweepCrash,
    Group::SweepRead,
    Group::SweepNone,
];

pub(super) const POINT_ORDER: [SamplingPoint; 5] = [
    SamplingPoint::Main,
    SamplingPoint::SweepThresholdProductDefault,
    SamplingPoint::LongJournalRing,
    SamplingPoint::SkippedRowWiderRing,
    SamplingPoint::NewestRowAndRollbackWiderRing,
];

/// 停机条款（登记第十一节「停机」）：装置与 `crates/` 对不上。触发的那一段不出结论，交主 agent。
pub(super) struct StopClauses {
    pub(super) lines: Vec<String>,
    pub(super) triggered: Vec<&'static str>,
}

fn first_detail(details: impl Iterator<Item = String>) -> String {
    details
        .into_iter()
        .next()
        .map_or("none".to_string(), |detail| detail.replace(' ', "_"))
}

/// S5 的一格在全部镜像上的合计。
fn sum_over_the_images(
    results: &[HistoryResult],
    pick: fn(&RecordsAppliedComparison) -> usize,
) -> usize {
    results
        .iter()
        .flat_map(|result| result.images.iter())
        .map(|image| pick(&image.records_applied))
        .sum()
}

pub(super) fn stop_clauses(results: &[HistoryResult], constants: &[ConstantCheck]) -> StopClauses {
    let mut lines = Vec::new();
    let mut triggered = Vec::new();
    let images = || {
        results
            .iter()
            .flat_map(|result| result.images.iter().map(move |image| (result, image)))
    };
    let ring_slot_mismatch_images = images()
        .filter(|(_, image)| !image.ring_slot_mismatches.is_empty())
        .count();
    lines.push(format!(
        "name=stop clause=S1 checked_images={} triggered_images={ring_slot_mismatch_images} first={}",
        images().count(),
        first_detail(images().flat_map(|(result, image)| {
            image
                .ring_slot_mismatches
                .iter()
                .map(move |detail| format!("{}/{}:{detail}", result.name, image.label))
        }))
    ));
    if ring_slot_mismatch_images > 0 {
        triggered.push("S1");
    }
    let compared: u64 = results
        .iter()
        .map(|result| result.ledger.root_slot_writes_compared)
        .sum();
    let without: u64 = results
        .iter()
        .map(|result| result.ledger.root_slot_writes_without_a_returned_root)
        .sum();
    let root_slot_write_mismatch_count: usize = results
        .iter()
        .map(|result| result.ledger.root_slot_write_mismatches.len())
        .sum();
    lines.push(format!(
        "name=stop clause=S2 compared_root_slot_writes={compared} root_slot_writes_without_a_returned_root={without} mismatches={root_slot_write_mismatch_count} first={}",
        first_detail(results.iter().flat_map(|result| result.ledger.root_slot_write_mismatches.iter().cloned()))
    ));
    if root_slot_write_mismatch_count > 0 || compared == 0 {
        triggered.push("S2");
    }
    let row_mismatch_images = images()
        .filter(|(_, image)| image.row_mismatch.is_some())
        .count();
    lines.push(format!(
        "name=stop clause=S3 checked_images={} triggered_images={row_mismatch_images} first={}",
        images().filter(|(_, image)| image.reading.is_ok()).count(),
        first_detail(images().filter_map(|(result, image)| {
            image
                .row_mismatch
                .as_ref()
                .map(|detail| format!("{}/{}:{detail}", result.name, image.label))
        }))
    ));
    if row_mismatch_images > 0 {
        triggered.push("S3");
    }
    let constant_failures = constants.iter().filter(|check| !check.holds()).count();
    let formula_mismatches: usize = results
        .iter()
        .map(|result| result.ledger.ring_formula_mismatches.len())
        .sum();
    lines.push(format!(
        "name=stop clause=S4 constants_checked={} constants_unequal={constant_failures} ring_formula_mismatches={formula_mismatches} first={}",
        constants.len(),
        first_detail(results.iter().flat_map(|result| result.ledger.ring_formula_mismatches.iter().cloned()))
    ));
    if constant_failures > 0 || formula_mismatches > 0 {
        triggered.push("S4");
    }
    let records_applied_mismatch_images = images()
        .filter(|(_, image)| image.records_applied.mismatch_count() > 0)
        .count();
    lines.push(format!(
        "name=stop clause=S5 checked_images={} triggered_images={records_applied_mismatch_images} apparatus_extra_with_a_root_that_landed_once={} apparatus_extra_otherwise={} apparatus_missing={} first={}",
        images().filter(|(_, image)| image.reading.is_ok()).count(),
        sum_over_the_images(results, |comparison| comparison.apparatus_extra_with_a_root_that_landed_once.len()),
        sum_over_the_images(results, |comparison| comparison.apparatus_extra_otherwise.len()),
        sum_over_the_images(results, |comparison| comparison.apparatus_missing.len()),
        first_detail(images().filter(|(_, image)| image.records_applied.mismatch_count() > 0).map(|(result, image)| {
            format!(
                "{}/{}:extra_landed_once={}:extra_otherwise={}:missing={}",
                result.name,
                image.label,
                keys_text(image.records_applied.apparatus_extra_with_a_root_that_landed_once.iter().copied()),
                keys_text(image.records_applied.apparatus_extra_otherwise.iter().copied()),
                keys_text(image.records_applied.apparatus_missing.iter().copied())
            )
        }))
    ));
    if records_applied_mismatch_images > 0 {
        triggered.push("S5");
    }
    let consistency: usize = results
        .iter()
        .map(|result| result.ledger.consistency_problems.len())
        .sum();
    lines.push(format!(
        "name=stop clause=ledger_consistency problems={consistency} first={}",
        first_detail(
            results
                .iter()
                .flat_map(|result| result.ledger.consistency_problems.iter().cloned())
        )
    ));
    if consistency > 0 {
        triggered.push("ledger_consistency");
    }
    StopClauses { lines, triggered }
}

#[cfg(test)]
mod tests;

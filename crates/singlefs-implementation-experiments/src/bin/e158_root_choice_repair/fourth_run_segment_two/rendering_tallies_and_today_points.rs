//! 第二段的产物行、家族计数、随机历史的阳性对照与今天那一臂的读数。

use super::*;

pub(super) fn render_random_mount(
    family: SegmentTwoFamily,
    label: &str,
    attempt: &str,
    arm: FourthRunArm,
    mount: &RandomMount,
) -> String {
    let judgement = mount.judgement.as_ref().map_or(
        "device_judgement=none truth_newer=false".to_string(),
        render_device_judgement,
    );
    let extras = mount.reading.extras.map_or("none".to_string(), |extras| {
        format!(
            "{}:{}",
            extras.reread_rounds_limit, extras.zeroed_journal_ring_slots_are_cached
        )
    });
    format!(
        "name=r4_random_mount family={} geometry={HARNESS_GEOMETRY_NAME} seed={label} step={} attempt={attempt} arm={} operation={} phase={} duration=W hidden_ranges={} reads_over_hidden_ranges={} base_fingerprint={:016x} {} a_reads_before_first_wait={} a_arm_extras={extras} {judgement} last_confirmed={} last_confirmed_rollback_aware={} e_content={} device_e_content={} q2_rollback={} q2_rollback_rollback_aware={} f6={} s4_findings={:?}",
        family.name(),
        mount.step_label,
        arm.name,
        mount.operation,
        mount.phase,
        mount.hidden_ranges,
        mount.reading.window.reads_over_hidden_ranges,
        mount.base_fingerprint,
        mount.summary.render("a"),
        optional_number_text(mount.reading.window.reads_before_the_first_wait),
        mount.literal_last,
        mount.rollback_aware_last,
        optional_number_text(mount.effective_content),
        optional_number_text(mount.device_effective_content),
        mount.older_version_returned,
        mount.older_version_returned_rollback_aware,
        mount
            .failure_clause_six
            .map_or("na".to_string(), |holds| holds.to_string()),
        mount.stop_clause_four_findings
    )
}

fn render_random_cell(
    family: SegmentTwoFamily,
    label: &str,
    arm: FourthRunArm,
    cell: &RandomCell,
    loss: &RandomCellLoss,
) -> String {
    let read_backs: Vec<String> = cell
        .read_backs
        .iter()
        .map(|(name, read_back)| read_back.render(name))
        .collect();
    format!(
        "name=r4_random_cell family={} geometry={HARNESS_GEOMETRY_NAME} seed={label} arm={} cell={} kind={} order={} protected={} protected_root={} protected_rollback_aware={} q1_in_ring_overwritten={} q1_out_of_ring_overwritten={} q1_landing_overwritten={} overwrite_cell={} q1_first_change={} q2_rollback={} q2_rollback_rollback_aware={} {} q3_lost={} q3_lost_rollback_aware={} q3_last_confirmed={} q3c_tail={} q3c_lost={} lost_write_cell={} lost_write_cell_rollback_aware={} q0_destination={}",
        family.name(),
        arm.name,
        cell.name,
        cell.kind,
        cell.order,
        cell.protected
            .as_ref()
            .map_or("none".to_string(), |protected| protected.content_number.to_string()),
        key_text(cell.protected.as_ref().map(|protected| protected.root)),
        cell.protected_rollback_aware,
        loss.in_ring_overwritten,
        loss.out_of_ring_overwritten,
        loss.landing_overwritten,
        loss.overwrite_cell(),
        render_first_change(loss.first_change),
        loss.older_version_returned,
        loss.older_version_returned_rollback_aware,
        read_backs.join(" "),
        loss.read_back_lost,
        loss.read_back_lost_rollback_aware,
        loss.read_back_last_confirmed,
        cell.tail_class,
        loss.broken_root_slots_read_back_lost,
        loss.lost_write_cell(),
        loss.lost_write_cell_rollback_aware(),
        cell.destination.clone().unwrap_or_else(|| "na".to_string())
    )
}

/// 峰值、为正的次数、期末值（登记 8.1：被谓词消费的量报轨迹）。
fn trajectory(values: &[u64]) -> String {
    format!(
        "peak{}:positive{}:final{}",
        values.iter().max().copied().unwrap_or(0),
        values.iter().filter(|value| **value > 0).count(),
        values
            .last()
            .map_or("none".to_string(), ToString::to_string)
    )
}

pub(super) fn render_random_segment(
    family: SegmentTwoFamily,
    label: &str,
    arm: FourthRunArm,
    executor: &RandomExecutor,
    branch: &BranchResult,
    operations: usize,
) -> String {
    let operation_steps: Vec<&RandomStepRecord> = executor
        .steps
        .iter()
        .filter(|step| step.order >= 1 && step.order <= operations)
        .collect();
    let checked: Vec<&RandomStepRecord> = executor
        .steps
        .iter()
        .filter(|step| step.checker.is_some())
        .collect();
    let first_red = checked
        .iter()
        .find(|step| {
            step.checker
                .as_ref()
                .is_some_and(|reading| !reading.red_invariants.is_empty())
        })
        .map(|step| step.order);
    let abandoned_red: Vec<&&RandomStepRecord> = checked
        .iter()
        .filter(|step| {
            step.checker
                .as_ref()
                .is_some_and(CheckerReading::abandoned_half_is_red)
        })
        .collect();
    let mut named: Vec<TimelineRoot> = Vec::new();
    for step in &abandoned_red {
        for root in step
            .checker
            .as_ref()
            .map(|reading| reading.abandoned_roots_named.clone())
            .unwrap_or_default()
        {
            if !named.contains(&root) {
                named.push(root);
            }
        }
    }
    let count_outcomes = |prefix: &str| {
        operation_steps
            .iter()
            .filter(|step| step.outcome.starts_with(prefix))
            .count()
    };
    let abandoned_unreadable: Vec<u64> = branch
        .mounts
        .iter()
        .filter_map(|mount| mount.summary.abandoned_roots_unreadable)
        .collect();
    let isolated: Vec<u64> = branch
        .mounts
        .iter()
        .filter_map(|mount| mount.summary.isolated)
        .collect();
    let rounds: Vec<u64> = branch
        .mounts
        .iter()
        .filter_map(|mount| {
            mount
                .summary
                .observation
                .as_ref()
                .map(|observation| u64::from(observation.reread_rounds))
        })
        .collect();
    format!(
        "name=r4_random_segment family={} geometry={HARNESS_GEOMETRY_NAME} seed={label} arm={} attempt={} operations={operations} steps_walked={} tail_run={} stop_at_the_first_checker_red={RANDOM_HISTORIES_STOP_AT_THE_FIRST_CHECKER_RED} stopped_at_the_first_red={} panicked={:?} starting_point_error={:?} crash_recovery_orders={:?} applied={} refused={} not_applicable={} checker_runs={} checker_first_red_order={} checker_first_abandoned_red_order={} checker_abandoned_red_steps={} checker_abandoned_roots_named={:?} mounts={} cells={} abandoned_roots_unreadable={} isolated={} reread_rounds={} {}",
        family.name(),
        arm.name,
        branch.attempt,
        operation_steps.len(),
        executor.steps.iter().any(|step| step.operation == "tail"),
        optional_number_text(executor.stopped_at_the_first_red.map(|order| u64::try_from(order).expect("步"))),
        executor.panicked,
        executor.starting_point_error,
        executor
            .crash_recovery_events
            .iter()
            .map(|event| event.order)
            .collect::<Vec<_>>(),
        count_outcomes("applied"),
        count_outcomes("refused"),
        count_outcomes("not_applicable"),
        checked.len(),
        optional_number_text(first_red.map(|order| u64::try_from(order).expect("步"))),
        optional_number_text(
            abandoned_red
                .first()
                .map(|step| u64::try_from(step.order).expect("步"))
        ),
        abandoned_red.len(),
        named,
        branch.mounts.len(),
        branch.cells.len(),
        trajectory(&abandoned_unreadable),
        trajectory(&isolated),
        trajectory(&rounds),
        branch.detail
    )
}

pub(super) fn render_random_step(
    family: SegmentTwoFamily,
    label: &str,
    arm: FourthRunArm,
    step: &RandomStepRecord,
) -> String {
    format!(
        "name=r4_random_step family={} seed={label} arm={} order={} operation={} outcome={:?} fingerprint_after={:016x} checker_red={} abandoned_named={} {}",
        family.name(),
        arm.name,
        step.order,
        step.operation,
        step.outcome,
        step.fingerprint_after,
        step.checker.as_ref().map_or("not_run".to_string(), |reading| format!(
            "{:?}",
            reading.red_invariants
        )),
        step.checker.as_ref().map_or("not_run".to_string(), |reading| format!(
            "{:?}",
            reading.abandoned_roots_named
        )),
        step.read_back
            .as_ref()
            .map_or("read_back=none".to_string(), |read_back| read_back.render("read_back"))
    )
}

/// 一族在一臂上的汇总（Q0–Q3、丢写格；`r4-compare` 把它换成 `r4_loss`）。
#[derive(Default)]
pub(super) struct FamilyTally {
    pub(super) segments: u64,
    pub(super) segments_panicked: u64,
    pub(super) cells: u64,
    pub(super) lost_write_cells: u64,
    pub(super) lost_write_cells_rollback_aware: u64,
    /// 丢写格的分解（只报数，不改丢写格的算法）：不算 ③c 的；Q1 只算「还在根环里」那一栏、也不算 ③c 的。
    pub(super) lost_write_cells_without_the_broken_root_slots_read_back: u64,
    pub(super) lost_write_cells_with_in_ring_overwrite_only_and_without_the_broken_root_slots_read_back:
        u64,
    pub(super) overwrite_cells: u64,
    pub(super) in_ring_overwrite_cells: u64,
    pub(super) out_of_ring_overwrite_cells: u64,
    pub(super) overwrite_cells_first_changed_while_the_newest: u64,
    pub(super) older_version_returned_cells: u64,
    pub(super) older_version_returned_cells_rollback_aware: u64,
    pub(super) read_back_lost_cells: u64,
    pub(super) read_back_lost_cells_rollback_aware: u64,
    pub(super) broken_root_slots_read_back_lost_cells: u64,
    pub(super) read_back_last_confirmed_cells: u64,
    pub(super) destinations: BTreeMap<String, u64>,
    pub(super) lost_write_cells_by_destination: BTreeMap<String, u64>,
    pub(super) mounts: u64,
    pub(super) classes: BTreeMap<String, u64>,
    pub(super) stop_clause_four_mismatches: u64,
    pub(super) failure_clause_six_checked: u64,
    pub(super) failure_clause_six_failures: u64,
    pub(super) branches_not_constructible: u64,
}

impl FamilyTally {
    pub(super) fn absorb_branch(&mut self, branch: &BranchResult) {
        if branch.detail.contains("not_constructible=")
            || branch.detail.contains("fault_not_reached=true")
        {
            self.branches_not_constructible += 1;
        }
        for mount in &branch.mounts {
            self.mounts += 1;
            *self
                .classes
                .entry(mount.summary.class.to_string())
                .or_insert(0) += 1;
            self.stop_clause_four_mismatches +=
                u64::try_from(mount.stop_clause_four_findings.len()).expect("条数");
            if let Some(holds) = mount.failure_clause_six {
                self.failure_clause_six_checked += 1;
                if !holds {
                    self.failure_clause_six_failures += 1;
                }
            }
        }
        for (cell, loss) in &branch.cells {
            self.cells += 1;
            let destination = cell
                .destination
                .clone()
                .unwrap_or_else(|| cell.kind.to_string());
            *self.destinations.entry(destination.clone()).or_insert(0) += 1;
            if loss.lost_write_cell() {
                self.lost_write_cells += 1;
                *self
                    .lost_write_cells_by_destination
                    .entry(destination)
                    .or_insert(0) += 1;
            }
            if loss.lost_write_cell_rollback_aware() {
                self.lost_write_cells_rollback_aware += 1;
            }
            if loss.overwrite_cell() || loss.older_version_returned || loss.read_back_lost {
                self.lost_write_cells_without_the_broken_root_slots_read_back += 1;
            }
            if loss.in_ring_overwritten > 0 || loss.older_version_returned || loss.read_back_lost {
                self.lost_write_cells_with_in_ring_overwrite_only_and_without_the_broken_root_slots_read_back += 1;
            }
            if loss.overwrite_cell() {
                self.overwrite_cells += 1;
                if loss
                    .first_change
                    .is_some_and(|change| change.later_confirmations == 0)
                {
                    self.overwrite_cells_first_changed_while_the_newest += 1;
                }
            }
            if loss.in_ring_overwritten > 0 {
                self.in_ring_overwrite_cells += 1;
            }
            if loss.out_of_ring_overwritten > 0 {
                self.out_of_ring_overwrite_cells += 1;
            }
            if loss.older_version_returned {
                self.older_version_returned_cells += 1;
            }
            if loss.older_version_returned_rollback_aware {
                self.older_version_returned_cells_rollback_aware += 1;
            }
            if loss.read_back_lost {
                self.read_back_lost_cells += 1;
            }
            if loss.read_back_lost_rollback_aware {
                self.read_back_lost_cells_rollback_aware += 1;
            }
            if loss.broken_root_slots_read_back_lost {
                self.broken_root_slots_read_back_lost_cells += 1;
            }
            if loss.read_back_last_confirmed {
                self.read_back_last_confirmed_cells += 1;
            }
        }
    }

    pub(super) fn render(
        &self,
        family: SegmentTwoFamily,
        arm: FourthRunArm,
        extra: &str,
    ) -> String {
        format!(
            "name=r4_random_summary family={} geometry={HARNESS_GEOMETRY_NAME} arm={} segments={} segments_panicked={} cells={} lost_write_cells_q1_q2_q3={} lost_write_cells_rollback_aware={} lost_write_cells_without_q3c={} lost_write_cells_with_q1_in_ring_only_and_without_q3c={} q1_overwrite_cells={} q1_in_ring_overwrite_cells={} q1_out_of_ring_overwrite_cells={} q1_overwrite_cells_first_changed_while_the_newest={} q2_rollback_cells={} q2_rollback_cells_rollback_aware={} q3_lost_cells={} q3_lost_cells_rollback_aware={} q3c_lost_cells={} q3_last_confirmed_cells={} q0_destinations={:?} lost_write_cells_by_destination={:?} tested_mounts={} tested_classes={:?} s4_mismatches={} f6_checked={} f6_failures={} branches_not_constructible={} {extra}",
            family.name(),
            arm.name,
            self.segments,
            self.segments_panicked,
            self.cells,
            self.lost_write_cells,
            self.lost_write_cells_rollback_aware,
            self.lost_write_cells_without_the_broken_root_slots_read_back,
            self.lost_write_cells_with_in_ring_overwrite_only_and_without_the_broken_root_slots_read_back,
            self.overwrite_cells,
            self.in_ring_overwrite_cells,
            self.out_of_ring_overwrite_cells,
            self.overwrite_cells_first_changed_while_the_newest,
            self.older_version_returned_cells,
            self.older_version_returned_cells_rollback_aware,
            self.read_back_lost_cells,
            self.read_back_lost_cells_rollback_aware,
            self.broken_root_slots_read_back_lost_cells,
            self.read_back_last_confirmed_cells,
            self.destinations,
            self.lost_write_cells_by_destination,
            self.mounts,
            self.classes,
            self.stop_clause_four_mismatches,
            self.failure_clause_six_checked,
            self.failure_clause_six_failures,
            self.branches_not_constructible
        )
    }
}

pub(super) fn lines_of_a_branch(
    family: SegmentTwoFamily,
    label: &str,
    arm: FourthRunArm,
    branch: &BranchResult,
) -> Vec<String> {
    let mut lines: Vec<String> = branch
        .mounts
        .iter()
        .map(|mount| render_random_mount(family, label, &branch.attempt, arm, mount))
        .collect();
    lines.extend(
        branch
            .cells
            .iter()
            .map(|(cell, loss)| render_random_cell(family, label, arm, cell, loss)),
    );
    lines
}

// ------------------------------------------------------------------------
// 13.10 PC-随（登记 5.4）：H-随 的 (a) 种子基 + 2、(b) 最短复现、(c) 第 43 行那条，每一臂都跑。
// ------------------------------------------------------------------------

/// PC-随 的三段：`of` 坐标、在 H-随 里的名字。
pub(super) const POSITIVE_CONTROL_RANDOM_SEGMENTS: [(&str, &str); 3] = [
    ("a", "base+2"),
    ("b", SHORTEST_REPRODUCTION_LABEL),
    ("c", ROW_FORTY_THREE_LABEL),
];

pub(super) fn first_abandoned_red(executor: &RandomExecutor) -> Option<(usize, Vec<TimelineRoot>)> {
    executor.steps.iter().find_map(|step| {
        step.checker
            .as_ref()
            .filter(|reading| reading.abandoned_half_is_red())
            .map(|reading| (step.order, reading.abandoned_roots_named.clone()))
    })
}

pub(super) fn checker_ever_named(executor: &RandomExecutor, root: TimelineRoot) -> bool {
    executor.steps.iter().any(|step| {
        step.checker
            .as_ref()
            .is_some_and(|reading| reading.abandoned_roots_named.contains(&root))
    })
}

/// 一段 PC-随 在一臂上的句子（每一句的标签与打不中走哪一条见登记 5.4 PC-随 那一行与执行员修订第 12 条的写法）。
pub(super) fn positive_control_random_sentences(
    arm: FourthRunArm,
    of: &str,
    executor: &RandomExecutor,
    branch: &BranchResult,
) -> (Vec<ControlSentence>, String) {
    let mut sentences = Vec::new();
    let event = executor.crash_recovery_events.first();
    let mount = event.and_then(|event| branch.mounts.get(event.mount_index));
    let cell = event.and_then(|event| {
        branch
            .cells
            .iter()
            .find(|(cell, _)| cell.kind == "crash_recovery" && cell.order == event.order)
    });
    let first_red = first_abandoned_red(executor);
    if arm.is_today() {
        let expected = match of {
            "a" => Some((
                POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_THE_SEED_PLUS_TWO.0,
                Some(POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_THE_SEED_PLUS_TWO.1),
            )),
            "b" => Some((POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_THE_SHORTEST, None)),
            "c" => Some((
                POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_ROW_FORTY_THREE.0,
                Some(POSITIVE_CONTROL_RANDOM_FIRST_RED_OF_ROW_FORTY_THREE.1),
            )),
            _ => None,
        };
        if let Some((operation_index, root)) = expected {
            let holds = first_red.as_ref().is_some_and(|(order, named)| {
                *order == operation_index + 1 && root.is_none_or(|root| named.contains(&root))
            });
            sentences.push(sentence(
                SentenceTag::TodayDescription,
                "checker_first_abandoned_red_step_and_root",
                SentenceVerdict::of(holds),
                "F19",
            ));
        }
        if of == "b" || of == "c" {
            let protected_root =
                cell.and_then(|(cell, _)| cell.protected.as_ref().map(|protected| protected.root));
            let verdict = match (protected_root, cell) {
                (Some(root), Some((_, loss))) => SentenceVerdict::conditional(
                    checker_ever_named(executor, root),
                    loss.in_ring_overwritten > 0,
                ),
                (None, _) | (_, None) => SentenceVerdict::NotConstructible,
            };
            sentences.push(sentence(
                SentenceTag::Measured,
                "checker_names_the_protected_root_implies_q1_in_ring",
                verdict,
                "V1",
            ));
        }
        let verdict = match (mount, cell, event) {
            (Some(mount), Some((_, loss)), Some(event)) => SentenceVerdict::conditional(
                class_is_a_successful_mount(mount.summary.class)
                    && mount
                        .device_effective_content
                        .zip(event.hidden_content_number)
                        .is_some_and(|(effective, hidden)| effective < hidden),
                loss.older_version_returned,
            ),
            (None, _, _) | (_, None, _) | (_, _, None) => SentenceVerdict::NotConstructible,
        };
        sentences.push(sentence(
            SentenceTag::Measured,
            "ok_on_an_older_version_implies_q2_rollback",
            verdict,
            "V1",
        ));
    } else {
        match arm.witness {
            FourthRunWitness::SystemConfiguration
            | FourthRunWitness::SystemConfigurationCarried => {
                let verdict = match mount {
                    Some(mount) => SentenceVerdict::of(
                        mount.summary.member == arm.refusal_member()
                            && mount.summary.writes == 0
                            && (!arm.rereads()
                                || mount
                                    .summary
                                    .observation
                                    .as_ref()
                                    .is_some_and(|observation| {
                                        u64::from(observation.reread_rounds) == arm.reread_rounds
                                    })),
                    ),
                    None => SentenceVerdict::NotConstructible,
                };
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "refuses_at_the_crash_recovery_step_before_any_write",
                    verdict,
                    "F9",
                ));
            }
            FourthRunWitness::UnreadableRootRingSlot => {
                let verdict = match mount {
                    Some(mount) => {
                        SentenceVerdict::of(mount.summary.observation.as_ref().is_some_and(
                            |observation| {
                                observation.witness_judged
                                    && !observation.newer_root_on_the_first_pass
                            },
                        ))
                    }
                    None => SentenceVerdict::NotConstructible,
                };
                sentences.push(sentence(
                    SentenceTag::ArmDefinition,
                    "slot_witness_false_at_the_crash_recovery_step",
                    verdict,
                    "F12",
                ));
            }
            FourthRunWitness::NotJudged => {}
        }
    }
    let detail = format!(
        "crash_recovery_order={} first_abandoned_red={} mount_class={} mount_member={} mount_writes={} q1_in_ring={} q2_rollback={}",
        event.map_or("none".to_string(), |event| event.order.to_string()),
        first_red.map_or("none".to_string(), |(order, named)| format!("order{order}:{named:?}")),
        mount.map_or("none", |mount| mount.summary.class),
        mount.map_or("none".to_string(), |mount| mount.summary.member.clone()),
        mount.map_or("none".to_string(), |mount| mount.summary.writes.to_string()),
        cell.map_or("none".to_string(), |(_, loss)| loss.in_ring_overwritten.to_string()),
        cell.map_or("none".to_string(), |(_, loss)| loss.older_version_returned.to_string())
    );
    (sentences, detail)
}

// ------------------------------------------------------------------------
// 13.11 各族怎么跑。今天那一臂先跑：与 harness 执行器逐步比、摆注入点与崩溃状态、判 S5a / S5b / S7 / S9，打进它的产物；
//       别的臂从今天那一臂的产物读回注入点、崩溃状态与停下的族（环境变量 `SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT`）。
// ------------------------------------------------------------------------

const TODAY_PRODUCT_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_E158_SEGMENT_TWO_TODAY_PRODUCT";

/// 今天那一臂的产物里别的臂要用的几样。
#[derive(Default)]
pub(super) struct TodayPoints {
    pub(super) fault_points: Vec<FaultPointKey>,
    pub(super) crash_points: BTreeMap<&'static str, Vec<CrashPointKey>>,
    pub(super) stopped_families: BTreeSet<String>,
}

fn family_named(name: &str) -> Option<SegmentTwoFamily> {
    [
        SegmentTwoFamily::RandomRed,
        SegmentTwoFamily::RandomAll,
        SegmentTwoFamily::SeventhFault,
        SegmentTwoFamily::SeventhCrash,
        SegmentTwoFamily::EighthCrash,
    ]
    .into_iter()
    .find(|family| family.name() == name)
}

pub(super) fn read_today_points() -> Result<TodayPoints, String> {
    let path = env::var(TODAY_PRODUCT_ENVIRONMENT_VARIABLE)
        .map_err(|_| format!("环境变量 {TODAY_PRODUCT_ENVIRONMENT_VARIABLE} 没给"))?;
    let text = std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
    if !text
        .lines()
        .last()
        .is_some_and(|line| line.starts_with("E7RESULT name=done emitted="))
    {
        return Err(format!("{path}: 末行不是完成标记"));
    }
    let mut points = TodayPoints::default();
    for line in text.lines().filter(|line| line.starts_with("E7RESULT ")) {
        let fields = fields_of_a_result_line(line);
        let text_of = |name: &str| fields.get(name).map_or("", String::as_str);
        let number_of = |name: &str| text_of(name).parse::<usize>().ok();
        match text_of("name") {
            "r4_segment_two_fault_point" => {
                let (Some(index), Some(order), Some(fault), Ok(ordinal_within_the_step)) = (
                    number_of("index"),
                    number_of("order"),
                    parse_injected_fault(text_of("fault")),
                    text_of("ordinal_within_the_step").parse::<u64>(),
                ) else {
                    return Err(format!("注入点那一行读不出：{line}"));
                };
                points.fault_points.push(FaultPointKey {
                    index,
                    fault,
                    order,
                    ordinal_within_the_step,
                });
            }
            "r4_segment_two_crash_point" => {
                let (
                    Some(family),
                    Some(index),
                    Some(order_of_the_first_write),
                    Some(segment_within_the_step),
                ) = (
                    family_named(text_of("family")),
                    number_of("index"),
                    number_of("order_of_the_first_write"),
                    number_of("segment_within_the_step"),
                )
                else {
                    return Err(format!("崩溃状态那一行读不出：{line}"));
                };
                points
                    .crash_points
                    .entry(family.name())
                    .or_default()
                    .push(CrashPointKey {
                        index,
                        order_of_the_first_write,
                        segment_within_the_step,
                        persisted_within_the_segment: parse_persisted(text_of("persisted")),
                    });
            }
            "stop" if !text_of("family").is_empty() => {
                points
                    .stopped_families
                    .insert(text_of("family").to_string());
            }
            _ => {}
        }
    }
    Ok(points)
}

/// 一族跑完交回的结果行与停机（停了的族只打停机那几行，别的行不进产物）。
pub(super) struct FamilyRun {
    pub(super) lines: Vec<String>,
    pub(super) stop: Option<String>,
}

pub(super) fn emit_family(family: SegmentTwoFamily, run: FamilyRun) {
    match run.stop {
        Some(stop) => emit_result(&format!(
            "name=stop family={} {stop} dropped_result_lines={}",
            family.name(),
            run.lines.len()
        )),
        None => {
            for line in &run.lines {
                emit_result(line);
            }
        }
    }
}

pub(super) fn agreement_line(
    family: SegmentTwoFamily,
    compared: usize,
    mismatches: &[String],
) -> String {
    format!(
        "name=r4_segment_two_executor_agreement family={} compared_steps={compared} mismatches={} verdict={} detail={:?}",
        family.name(),
        mismatches.len(),
        if compared > 0 && mismatches.is_empty() { "pass" } else { "fail" },
        mismatches.iter().take(5).collect::<Vec<_>>()
    )
}

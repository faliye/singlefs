//! 每个世界、活化、近满、排空失败、坏槽、代价增长各自打出的窗口行与整体判定行，以及窗口臂的整趟。

use super::*;

/// 六个基本世界 × S × 丢盘 × 四条臂；阳性对照臂只进 steady、S ∈ {4, 16}（跑前登记第五节）。
pub(super) fn emit_window_world_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily) {
    for world in WORLDS {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            for arm in family.arms.iter().copied() {
                let lost_disks: &[usize] = if world == World::DiskLoss { &[0, 1] } else { &[0] };
                for &lost_disk in lost_disks {
                    let total = fresh_total_over_seeds(SEED_COUNT, |seed| run_world(arm, world, slots_per_region, seed, lost_disk));
                    emit_window_row(emitter, rows, family, WindowRowDraft {
                        line_name: family.lines.world,
                        world_class: world.name(),
                        row_key: format!("world={} s={slots_per_region} lost_disk={lost_disk}", world.name()),
                        arm,
                        outcome_fields: world_outcome_fields(&total),
                        judgement: RowJudgement::of(&total, false),
                        is_report_only_world: world == World::RollbackFirstPublish,
                        has_false_enospc_field: true,
                        metrics: &total,
                    });
                }
            }
        }
    }
    for world in family.short_arm_worlds.iter().copied() {
        for arm in family.short_arms.iter().copied() {
            for slots_per_region in WINDOW_SHORT_SLOTS_SWEEP {
                let total = fresh_total_over_seeds(SEED_COUNT, |seed| run_world(arm, world, slots_per_region, seed, 0));
                emit_window_row(emitter, rows, family, WindowRowDraft {
                    line_name: family.lines.world,
                    world_class: world.name(),
                    row_key: format!("world={} s={slots_per_region} lost_disk=0", world.name()),
                    arm,
                    outcome_fields: world_outcome_fields(&total),
                    judgement: RowJudgement::of(&total, false),
                    is_report_only_world: false,
                    has_false_enospc_field: true,
                    metrics: &total,
                });
            }
        }
    }
}

/// 活化一族：原取样点五种变体，或 H2（跳号换成区域 0，只跑 skip_then_raise 三种）。
pub(super) fn emit_window_activation_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily, point: GeometryPoint) {
    let patterns = failure_patterns();
    let (line_name, variants): (&'static str, Vec<ActivationVariant>) = match point {
        GeometryPoint::Original => {
            let mut variants = vec![ActivationVariant::Plain, ActivationVariant::CrashWithoutLossFirst];
            variants.extend(SKIP_THEN_RAISE_REPEATS.iter().map(|repeats| ActivationVariant::SkipThenRaise { repeats: *repeats }));
            (family.lines.activation, variants)
        }
        GeometryPoint::Reversed => (family.lines.activation_reversed, SKIP_THEN_RAISE_REPEATS.iter().map(|repeats| ActivationVariant::SkipThenRaise { repeats: *repeats }).collect()),
    };
    let mut detail_lines = 0u64;
    for variant in variants {
        let skip_repeats = match variant {
            ActivationVariant::SkipThenRaise { repeats } => repeats,
            ActivationVariant::Plain | ActivationVariant::CrashWithoutLossFirst => 0,
        };
        let short_arms_here: &[Arm] =
            if family.short_arms_enter_activation_faults && point == GeometryPoint::Original && variant == ActivationVariant::Plain { &family.short_arms } else { &[] };
        for slots_per_region in FAULT_SLOTS_SWEEP {
            for arm in family.arms.iter().chain(short_arms_here.iter()).copied() {
                let mut by_class = [[Metrics::fresh(); 3]; DISK_COUNT];
                for seed in 0..FAULT_SEED_COUNT {
                    let prefix = match point {
                        GeometryPoint::Original => activation_prefix(arm, slots_per_region, seed, skip_repeats),
                        GeometryPoint::Reversed => activation_prefix_with_jump_region(arm, slots_per_region, seed, skip_repeats, 0),
                    };
                    let Some(prefix) = prefix else {
                        continue;
                    };
                    for pattern in &patterns {
                        for crash_window in 1..=ACTIVATION_FAULT_CRASH_WINDOWS {
                            for (lost_disk, class_row) in by_class.iter_mut().enumerate().take(DISK_COUNT) {
                                let case = run_activation_case(&prefix, pattern, crash_window, lost_disk, variant);
                                class_row[pattern.len()].add(&case);
                                if (case.fake_candidates > 0 || case.unrecoverable > 0) && detail_lines < DETAIL_LINES_MAXIMUM {
                                    detail_lines += 1;
                                    println!("{}", emitter.emit_raw(&format!(
                                        "name={} line={line_name} variant={} arm={} s={slots_per_region} seed={seed} failures={} crash_window={crash_window} \
                                         lost_disk={lost_disk} fakes={} unrecoverable={}",
                                        family.lines.detail_activation, variant.name(), arm.name(), format_set(pattern), case.fake_candidates, case.unrecoverable
                                    )));
                                }
                            }
                        }
                    }
                }
                for (lost_disk, classes) in by_class.iter().enumerate() {
                    for (failure_count, total) in classes.iter().enumerate() {
                        emit_window_row(emitter, rows, family, WindowRowDraft {
                            line_name,
                            world_class: activation_class(variant),
                            row_key: format!("variant={} s={slots_per_region} lost_disk={lost_disk} failures={failure_count}", variant.name()),
                            arm,
                            outcome_fields: activation_outcome_fields(total),
                            judgement: RowJudgement::of(total, false),
                            is_report_only_world: false,
                            has_false_enospc_field: false,
                            metrics: total,
                        });
                    }
                }
            }
        }
    }
}

/// near_full：两档开销对 × S × 四条臂（判假性 ENOSPC）。
pub(super) fn emit_window_near_full_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily) {
    for slots_per_region in SLOTS_PER_REGION_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in family.arms.iter().copied() {
                let ring_depth = REGION_COUNT * slots_per_region;
                let total = fresh_total_over_seeds(SEED_COUNT, |seed| run_near_full(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, None));
                let formula_cost = arm.formula_cost(mount_cost, declared_maximum_cost);
                let reserve = arm.reserve_blocks(ring_depth, formula_cost);
                let residue = arm.push_residue_blocks(ring_depth, formula_cost);
                let bound = arm.publication_bound(ring_depth);
                let outcome_fields = format!(
                    "reserve={reserve} residue={residue} hidden_blocks={} hidden_bytes={} avg_lag_x100={} max_lag={} false_enospc_raw={} false_enospc_arm={} \
                     true_enospc={} write_after_delete_failed={} first_failure_window={} last_failure_window={} max_publications={} bound={bound} \
                     admissions_under_pressure={} forced_per_window_x100={} stalls={} halted_before_counting={} fakes={} unrecoverable={} \
                     retention_violations={} min_candidates={} avg_candidates_x100={} avg_nonempty_x100={} min_states={} avg_states_x100={}",
                    reserve + residue, (reserve + residue) * BLOCK_BYTES, total.average_lag_in_hundredths(), total.lag_maximum,
                    total.false_enospc_raw, total.false_enospc_arm, total.true_enospc, total.write_after_delete_failed,
                    total.first_failure_window_or_zero(), total.last_write_after_delete_failure_window, total.maximum_publications_per_admission,
                    total.admissions_under_pressure, total.forced_publications_per_window_in_hundredths(), total.stalls, total.halted_before_counting,
                    total.fake_candidates, total.unrecoverable, total.retention_violations, total.minimum_candidates_or_zero(),
                    total.average_candidates_in_hundredths(), total.average_nonempty_candidates_in_hundredths(),
                    total.minimum_distinct_states_or_zero(), total.average_distinct_states_in_hundredths()
                );
                emit_window_row(emitter, rows, family, WindowRowDraft {
                    line_name: family.lines.near_full,
                    world_class: NEAR_FULL_CLASS,
                    row_key: format!("s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost}"),
                    arm,
                    outcome_fields,
                    judgement: RowJudgement::of(&total, true),
                    is_report_only_world: false,
                    has_false_enospc_field: true,
                    metrics: &total,
                });
            }
        }
    }
}

/// drain_failure：失败 1、2、3 次（判假性 ENOSPC 只对 ≤ k_tol 的格）。
pub(super) fn emit_window_drain_failure_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily) {
    let mut detail_lines = 0u64;
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in family.arms.iter().copied() {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                let drain_patterns = drain_failure_patterns(arm.drain_failure_offset_limit(ring_depth));
                let mut by_count = [Metrics::fresh(); 4];
                let mut cases_by_count = [0u64; 4];
                let mut runs_with_false = [0u64; 4];
                let mut runs_with_write_after_delete = [0u64; 4];
                for seed in 0..FAULT_SEED_COUNT {
                    let Some(prefix) = drain_failure_prefix(arm, slots_per_region, mount_cost, declared_maximum_cost, seed) else {
                        continue;
                    };
                    for pattern in &drain_patterns {
                        let case = run_drain_failure_case(&prefix, pattern);
                        by_count[pattern.len()].add(&case);
                        cases_by_count[pattern.len()] += 1;
                        runs_with_false[pattern.len()] += u64::from(case.false_enospc_arm > 0);
                        runs_with_write_after_delete[pattern.len()] += u64::from(case.write_after_delete_failed > 0);
                        if (case.false_enospc_arm > 0 || case.fake_candidates > 0 || case.unrecoverable > 0) && detail_lines < 2 * DETAIL_LINES_MAXIMUM {
                            detail_lines += 1;
                            println!("{}", emitter.emit_raw(&format!(
                                "name={} arm={} s={slots_per_region} c0={mount_cost} seed={seed} failures={} false_enospc_arm={} fakes={} unrecoverable={} \
                                 write_after_delete_failed={} max_publications={} stalls={}",
                                family.lines.detail_drain, arm.name(), format_set(pattern), case.false_enospc_arm, case.fake_candidates, case.unrecoverable,
                                case.write_after_delete_failed, case.maximum_publications_per_admission, case.stalls
                            )));
                        }
                    }
                }
                for failure_count in 1..=3usize {
                    let total = by_count[failure_count];
                    let outcome_fields = format!(
                        "cases={} runs_with_false_enospc={} runs_with_write_after_delete_failed={} false_enospc_arm={} false_enospc_raw={} \
                         write_after_delete_failed={} max_publications={} bound={bound} stalls={} fakes={} unrecoverable={} retention_violations={} injected={}",
                        cases_by_count[failure_count], runs_with_false[failure_count], runs_with_write_after_delete[failure_count],
                        total.false_enospc_arm, total.false_enospc_raw, total.write_after_delete_failed, total.maximum_publications_per_admission,
                        total.stalls, total.fake_candidates, total.unrecoverable, total.retention_violations, total.root_write_failures_injected
                    );
                    emit_window_row(emitter, rows, family, WindowRowDraft {
                        line_name: family.lines.drain_failure,
                        world_class: DRAIN_FAILURE_CLASS,
                        row_key: format!("s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} failures={failure_count}"),
                        arm,
                        outcome_fields,
                        judgement: RowJudgement::of(&total, failure_count as u64 <= TOLERATED_ROOT_WRITE_FAILURES),
                        is_report_only_world: false,
                        has_false_enospc_field: true,
                        metrics: &total,
                    });
                }
            }
        }
    }
}

/// broken_slot：原取样点（槽 1，区域 0、盘 0）或 H1（槽 S，区域 1、盘 1）。
pub(super) fn emit_window_broken_slot_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, growth: &mut Vec<StuckGrowthSummary>, family: &RowFamily, point: GeometryPoint) {
    let (line_name, growth_line_name) = match point {
        GeometryPoint::Original => (family.lines.broken_slot, family.lines.broken_slot_growth),
        GeometryPoint::Reversed => (family.lines.broken_slot_reversed, family.lines.broken_slot_reversed_growth),
    };
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in family.arms.iter().copied() {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                let mut total = Metrics::fresh();
                let mut seeds_with_no_space = 0u64;
                let mut seeds_halted = 0u64;
                let mut seed_marks: Vec<[Option<StuckMark>; 3]> = Vec::new();
                for seed in 0..FAULT_SEED_COUNT {
                    let one = match point {
                        GeometryPoint::Original => run_broken_slot(arm, slots_per_region, mount_cost, declared_maximum_cost, seed),
                        GeometryPoint::Reversed => {
                            let first_slot_of_region_one = usize::try_from(slots_per_region).expect("槽数装得进 usize");
                            run_broken_slot_at(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, first_slot_of_region_one)
                        }
                    };
                    seeds_with_no_space += u64::from(one.write_after_delete_failed > 0);
                    seeds_halted += u64::from(one.stalls > 0);
                    seed_marks.push(one.second_rerun.marks);
                    total.add(&one);
                }
                let outcome_fields = format!(
                    "seeds={FAULT_SEED_COUNT} seeds_with_no_space={seeds_with_no_space} seeds_halted={seeds_halted} first_failure_window={} false_enospc_arm={} \
                     false_enospc_raw={} write_after_delete_failed={} max_publications={} bound={bound} stalls={} fakes={} unrecoverable={} \
                     retention_violations={} injected={} forced_per_window_x100={} max_lag={}",
                    total.first_failure_window_or_zero(), total.false_enospc_arm, total.false_enospc_raw, total.write_after_delete_failed,
                    total.maximum_publications_per_admission, total.stalls, total.fake_candidates, total.unrecoverable, total.retention_violations,
                    total.root_write_failures_injected, total.forced_publications_per_window_in_hundredths(), total.lag_maximum
                );
                let row_key = format!("s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost}");
                if family.is_second_rerun {
                    growth.push(emit_stuck_growth_row(emitter, growth_line_name, arm, &row_key, &seed_marks));
                }
                emit_window_row(emitter, rows, family, WindowRowDraft {
                    line_name,
                    world_class: BROKEN_SLOT_CLASS,
                    row_key,
                    arm,
                    outcome_fields,
                    judgement: RowJudgement::of(&total, true),
                    is_report_only_world: false,
                    has_false_enospc_field: true,
                    metrics: &total,
                });
            }
        }
    }
}

/// cost_growth：开销升到声明的上界（判）与超过上界 3 块（只报）。
pub(super) fn emit_window_cost_growth_rows(emitter: &mut Emitter, rows: &mut Vec<WindowRow>, family: &RowFamily) {
    for slots_per_region in FAULT_SLOTS_SWEEP {
        for (mount_cost, declared_maximum_cost) in COST_PAIRS {
            for arm in family.arms.iter().copied() {
                let ring_depth = REGION_COUNT * slots_per_region;
                let bound = arm.publication_bound(ring_depth);
                for grown_cost in [declared_maximum_cost, declared_maximum_cost + COST_OVER_DECLARED_EXTRA] {
                    let is_within_declared = grown_cost <= declared_maximum_cost;
                    let total = fresh_total_over_seeds(FAULT_SEED_COUNT, |seed| run_cost_growth(arm, slots_per_region, mount_cost, declared_maximum_cost, seed, grown_cost));
                    let outcome_fields = format!(
                        "false_enospc_arm={} false_enospc_raw={} true_enospc={} write_after_delete_failed={} first_failure_window={} max_publications={} bound={bound} \
                         stalls={} fakes={} unrecoverable={} retention_violations={} forced_per_window_x100={}",
                        total.false_enospc_arm, total.false_enospc_raw, total.true_enospc, total.write_after_delete_failed, total.first_failure_window_or_zero(),
                        total.maximum_publications_per_admission, total.stalls, total.fake_candidates, total.unrecoverable, total.retention_violations,
                        total.forced_publications_per_window_in_hundredths()
                    );
                    emit_window_row(emitter, rows, family, WindowRowDraft {
                        line_name: family.lines.cost_growth,
                        world_class: COST_GROWTH_CLASS,
                        row_key: format!("s={slots_per_region} c0={mount_cost} c_max={declared_maximum_cost} grown_to={grown_cost} within_declared={}", u8::from(is_within_declared)),
                        arm,
                        outcome_fields,
                        judgement: RowJudgement::of(&total, is_within_declared),
                        is_report_only_world: false,
                        has_false_enospc_field: true,
                        metrics: &total,
                    });
                }
            }
        }
    }
}

/// 旧登记判据 1 的跑前式子，套在新臂上：steady 里环没有洞，窗口等于整环，发布数、扣住块数与滞后量应与 G2S 一样。返回不成立的行数。
pub(super) fn emit_window_crit1_rows(emitter: &mut Emitter, family: &RowFamily) -> (u64, Vec<String>) {
    let mut rows_not_holding = 0u64;
    let mut line_bodies: Vec<String> = Vec::new();
    for arm in family.arms.iter().copied() {
        for slots_per_region in SLOTS_PER_REGION_SWEEP {
            for (cost, _) in COST_PAIRS {
                let (publication_set, maximum_publications, maximum_pinned, maximum_lag, is_all_released) = push_probe_over_landings(arm, slots_per_region, cost);
                let (expect_set, expect_maximum, expect_pinned, expect_lag) = expected_push(arm, slots_per_region, cost);
                let holds = is_all_released && maximum_publications == expect_maximum && maximum_pinned == expect_pinned && maximum_lag == expect_lag
                    && publication_set.iter().all(|publications| expect_set.contains(publications));
                rows_not_holding += u64::from(!holds);
                let line_body = format!(
                    "arm={} s={slots_per_region} empty_cost={cost} publications={} max_publications={maximum_publications} \
                     expect_publications={} expect_max={expect_maximum} max_pinned={maximum_pinned} expect_pinned={expect_pinned} \
                     lag={maximum_lag} expect_lag={expect_lag} released={} holds={}",
                    arm.name(), format_set(&publication_set), format_set(&expect_set), u8::from(is_all_released), u8::from(holds)
                );
                print_and_record(emitter.emit_raw(&format!("name={} {line_body}", family.lines.crit1_push)));
                line_bodies.push(line_body);
            }
        }
    }
    (rows_not_holding, line_bodies)
}

fn is_original_sampling_row(row: &WindowRow, family: &RowFamily) -> bool {
    row.line_name != family.lines.activation_reversed && row.line_name != family.lines.broken_slot_reversed && row.line_name != family.lines.disk_stuck_reversed
}

fn count_rows(rows: &[&WindowRow], predicate: impl Fn(&WindowRow) -> bool) -> usize {
    rows.iter().filter(|row| predicate(row)).count()
}

/// 第六节第 8–10 行：被测三条臂与旧门槛对照各自的逐行判定计数。
pub(super) fn emit_window_row_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily, crit1_rows_not_holding: u64) {
    for arm in family.arms.iter().copied() {
        let arm_rows: Vec<&WindowRow> = rows.iter().filter(|row| row.arm == arm && is_original_sampling_row(row, family)).collect();
        println!("{}", emitter.emit_raw(&format!(
            "name={} arm={} rows={} rows_with_fake_candidates={} rows_with_unrecoverable={} judged_rows_with_false_enospc={} \
             rollback_first_publish_rows={} rollback_first_publish_rows_with_fake_candidates={} rollback_first_publish_rows_with_unrecoverable={} crit1_push_rows_not_holding={}",
            family.lines.verdict_window, arm.name(), arm_rows.len(),
            count_rows(&arm_rows, |row| !row.is_report_only_world && row.judgement.has_fake_candidates),
            count_rows(&arm_rows, |row| !row.is_report_only_world && row.judgement.has_unrecoverable),
            count_rows(&arm_rows, |row| row.judgement.has_false_enospc_in_a_judged_row),
            count_rows(&arm_rows, |row| row.is_report_only_world),
            count_rows(&arm_rows, |row| row.is_report_only_world && row.judgement.has_fake_candidates),
            count_rows(&arm_rows, |row| row.is_report_only_world && row.judgement.has_unrecoverable),
            crit1_rows_not_holding
        )));
    }
    for arm in family.short_arms.iter().copied() {
        let arm_rows: Vec<&WindowRow> = rows.iter().filter(|row| row.arm == arm).collect();
        println!("{}", emitter.emit_raw(&format!(
            "name={}_short arm={} rows={} rows_where_the_detector_saw_zero_fake_candidates={}",
            family.lines.verdict_window, arm.name(), arm_rows.len(), count_rows(&arm_rows, |row| !row.judgement.has_fake_candidates)
        )));
    }
}

/// 第六节第 6 行：G2SW 每类世界里门槛差为正的次数；注入根槽写失败的六类里合计为 0 的类数。
pub(super) fn emit_window_gate_reach_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    let window_rows: Vec<&WindowRow> = rows.iter().filter(|row| row.arm == family.window_arm && is_original_sampling_row(row, family)).collect();
    let mut classes: Vec<&str> = Vec::new();
    for row in &window_rows {
        if !classes.contains(&row.world_class.as_str()) {
            classes.push(row.world_class.as_str());
        }
    }
    let mut injection_classes_without_window_effect = 0u64;
    for class in classes {
        let class_rows: Vec<&&WindowRow> = window_rows.iter().filter(|row| row.world_class == class).collect();
        let positive_samples_total: u64 = class_rows.iter().map(|row| row.gate_difference_positive_samples).sum();
        let is_injecting = WORLD_CLASSES_INJECTING_ROOT_WRITE_FAILURES.contains(&class);
        injection_classes_without_window_effect += u64::from(is_injecting && positive_samples_total == 0);
        println!("{}", emitter.emit_raw(&format!(
            "name={}_gate_reach class={class} injects_root_write_failures={} rows={} rows_with_positive_gate_difference={} positive_samples_total={positive_samples_total}",
            family.lines.verdict_window, u8::from(is_injecting), class_rows.len(), class_rows.iter().filter(|row| row.gate_difference_positive_samples > 0).count()
        )));
    }
    println!("{}", emitter.emit_raw(&format!(
        "name={}_gate_reach_summary injection_classes={} injection_classes_without_window_effect={injection_classes_without_window_effect}",
        family.lines.verdict_window, WORLD_CLASSES_INJECTING_ROOT_WRITE_FAILURES.len()
    )));
}

/// 第六节第 7 行：同一组参数上 G2SW_ringgate 那一行与 G2SW 那一行，除臂名外整行比。
pub(super) fn emit_window_self_proof_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    let ringgate_text_by_key: HashMap<(&str, &str), &str> = rows.iter()
        .filter(|row| row.arm == family.ring_gate_arm)
        .map(|row| ((row.line_name, row.row_key.as_str()), row.text_without_arm.as_str())).collect();
    let mut compared = 0u64;
    let mut differing = 0u64;
    let mut differing_by_class: Vec<(String, u64)> = Vec::new();
    for row in rows.iter().filter(|row| row.arm == family.window_arm && is_original_sampling_row(row, family)) {
        let Some(ringgate_text) = ringgate_text_by_key.get(&(row.line_name, row.row_key.as_str())) else {
            continue;
        };
        compared += 1;
        if *ringgate_text != row.text_without_arm.as_str() {
            differing += 1;
            match differing_by_class.iter_mut().find(|(class, _)| *class == row.world_class) {
                Some((_, count)) => *count += 1,
                None => differing_by_class.push((row.world_class.clone(), 1)),
            }
        }
    }
    for (class, count) in &differing_by_class {
        println!("{}", emitter.emit_raw(&format!("name={}_self_proof_class class={class} rows_differing_from_ringgate={count}", family.lines.verdict_window)));
    }
    println!("{}", emitter.emit_raw(&format!("name={}_self_proof rows_compared={compared} rows_differing_from_ringgate={differing}", family.lines.verdict_window)));
}

/// 第六节第 12 行：G2SW_Q、G2SW_L 各与 G2SW 的逐行判定比。
pub(super) fn emit_window_reading_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    let window_judgement_by_key: HashMap<(&str, &str), RowJudgement> = rows.iter()
        .filter(|row| row.arm == family.window_arm && is_original_sampling_row(row, family))
        .map(|row| ((row.line_name, row.row_key.as_str()), row.judgement)).collect();
    let mut detail_lines = 0u64;
    for reading_arm in [family.disk_hold_arm, family.ring_lag_arm] {
        let mut compared = 0u64;
        let mut differing = 0u64;
        for row in rows.iter().filter(|row| row.arm == reading_arm && is_original_sampling_row(row, family)) {
            let Some(window_judgement) = window_judgement_by_key.get(&(row.line_name, row.row_key.as_str())) else {
                continue;
            };
            compared += 1;
            if *window_judgement != row.judgement {
                differing += 1;
                if detail_lines < 2 * DETAIL_LINES_MAXIMUM {
                    detail_lines += 1;
                    println!("{}", emitter.emit_raw(&format!(
                        "name={} reading_arm={} line={} {} window_judgement={window_judgement:?} reading_judgement={:?}",
                        family.lines.readings_diff_row, reading_arm.name(), row.line_name, row.row_key, row.judgement
                    )));
                }
            }
        }
        println!("{}", emitter.emit_raw(&format!(
            "name={}_readings reading_arm={} rows_compared={compared} rows_with_a_judgement_different_from_window={differing}", family.lines.verdict_window, reading_arm.name()
        )));
    }
}

/// 第八节几何敏感性：H1（坏槽换到盘 1）、H2（跳号换到盘 0）的逐行判定与原取样点比。
pub(super) fn emit_window_geometry_verdicts(emitter: &mut Emitter, rows: &[WindowRow], family: &RowFamily) {
    let mut points = vec![("h1", family.lines.broken_slot_reversed, family.lines.broken_slot), ("h2", family.lines.activation_reversed, family.lines.activation)];
    if family.is_second_rerun {
        points.push(("h3", family.lines.disk_stuck_reversed, family.lines.disk_stuck));
    }
    for arm in family.arms.iter().copied() {
        for (point_name, reversed_line, original_line) in points.iter().copied() {
            let original_judgement_by_key: HashMap<&str, RowJudgement> = rows.iter()
                .filter(|row| row.arm == arm && row.line_name == original_line)
                .map(|row| (row.row_key.as_str(), row.judgement)).collect();
            let mut compared = 0u64;
            let mut differing = 0u64;
            for row in rows.iter().filter(|row| row.arm == arm && row.line_name == reversed_line) {
                let Some(original_judgement) = original_judgement_by_key.get(row.row_key.as_str()) else {
                    continue;
                };
                compared += 1;
                differing += u64::from(*original_judgement != row.judgement);
            }
            if compared == 0 && family.is_second_rerun {
                continue;
            }
            println!("{}", emitter.emit_raw(&format!(
                "name={}_geometry point={point_name} arm={} rows_compared={compared} rows_with_a_judgement_different_from_the_original_point={differing}",
                family.lines.verdict_window, arm.name()
            )));
        }
    }
}

/// 第 1 次重跑的全部新臂行与它们的判决行；接在原产物全部行之后、`name=done` 之前。返回行簿给第 2 次重跑逐行比。
pub(super) fn run_window_arms(emitter: &mut Emitter) -> (Vec<WindowRow>, Vec<String>) {
    let family = first_rerun_family();
    let family = &family;
    println!("{}", emitter.emit_raw(&format!(
        "name=w_config window_arms={} short_arms={} short_slots={} window_length_rule=txg_plus_ring_depth_greater_than_anchor short_window_length_rule=txg_plus_ring_depth_minus_regions_greater_than_anchor \
         geometry_points=h1/h2 seeds={SEED_COUNT} fault_seeds={FAULT_SEED_COUNT}",
        WINDOW_ARMS.iter().map(|arm| arm.name()).collect::<Vec<&str>>().join("/"),
        WINDOW_SHORT_ARMS.iter().map(|arm| arm.name()).collect::<Vec<&str>>().join("/"),
        format_set(&WINDOW_SHORT_SLOTS_SWEEP)
    )));
    let mut rows: Vec<WindowRow> = Vec::new();
    let mut unused_growth: Vec<StuckGrowthSummary> = Vec::new();
    let (crit1_rows_not_holding, crit1_line_bodies) = emit_window_crit1_rows(emitter, family);
    emit_window_world_rows(emitter, &mut rows, family);
    emit_window_activation_rows(emitter, &mut rows, family, GeometryPoint::Original);
    emit_window_near_full_rows(emitter, &mut rows, family);
    emit_window_drain_failure_rows(emitter, &mut rows, family);
    emit_window_broken_slot_rows(emitter, &mut rows, &mut unused_growth, family, GeometryPoint::Original);
    emit_window_cost_growth_rows(emitter, &mut rows, family);
    emit_window_activation_rows(emitter, &mut rows, family, GeometryPoint::Reversed);
    emit_window_broken_slot_rows(emitter, &mut rows, &mut unused_growth, family, GeometryPoint::Reversed);
    emit_window_row_verdicts(emitter, &rows, family, crit1_rows_not_holding);
    emit_window_gate_reach_verdicts(emitter, &rows, family);
    emit_window_self_proof_verdicts(emitter, &rows, family);
    emit_window_reading_verdicts(emitter, &rows, family);
    emit_window_geometry_verdicts(emitter, &rows, family);
    (rows, crit1_line_bodies)
}

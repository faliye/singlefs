//! 各条判器的产物行：核对行、新鲜度、操纵、序号、存量、预扫、挂载、附加行、操作行、旗线与几何敏感性行与总报告。

use e7_index_bench::{i64_from_u64, i64_from_usize};
use super::*;

fn checks_lines(world: &World<'_>, all_items: &[MeasuredItem]) -> (Vec<String>, BTreeMap<&'static str, u64>) {
    let mut lines = Vec::new();
    let mut counts: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut bump = |name: &'static str, failed: bool| {
        *counts.entry(name).or_insert(0) += u64::from(failed);
    };
    for item in &world.items {
        let constants = constants_mismatches(item);
        let history = history_mismatch(item);
        let sequence = declared_sequence_mismatch(item);
        let closure = closure_check(item);
        let shortfall = if item.arm == "head" { live_units_shortfall(item) } else { None };
        bump("items_without_item_done", !item.done);
        bump("items_refused", item.refused);
        bump("constants_mismatches", !constants.is_empty());
        bump("history_mismatches", history.is_some());
        bump("script_sequence_mismatches", sequence.is_some());
        bump("closure_model_mismatches", !closure.closure_mismatches.is_empty());
        bump("final_set_not_covering", closure.final_not_covering > 0);
        bump("final_nodes_mismatches", closure.final_nodes_mismatches > 0);
        bump("edge_walk_mismatches", closure.walk_mismatches > 0);
        bump("live_units_shortfalls", shortfall.is_some());
        lines.push(format!(
            "name=item_checks item={} done={} refused={} constants={} history={} script_sequence={} closure_publishes={} closure_mismatches={} final_not_covering={} final_nodes_mismatches={} edge_walk_mismatches={} live_units={}",
            item.name,
            yes_no(item.done),
            yes_no(item.refused),
            list_text(&constants),
            history.unwrap_or_else(|| "ok".to_string()),
            sequence.unwrap_or_else(|| "ok".to_string()),
            closure.publishes,
            first_labels(&closure.closure_mismatches),
            closure.final_not_covering,
            closure.final_nodes_mismatches,
            closure.walk_mismatches,
            shortfall.unwrap_or_else(|| "ok".to_string())
        ));
        if item.script == "first_repeated" {
            let rounds = negative_control_rounds(item);
            bump("negative_control_over_four_rounds", rounds.is_none_or(|rounds| rounds > 4));
            lines.push(format!("name=negative_control item={} rounds={}", item.name, rounds.map_or("unread".to_string(), |rounds| rounds.to_string())));
        }
    }
    let twins: Vec<&MeasuredItem> = all_items.iter().filter(|item| item.is_twin).collect();
    let mut twin_failures = 0;
    for twin in &twins {
        let original = world.items.iter().find(|item| item.name == twin.name);
        let difference = original.and_then(|original| twin_difference(original, twin));
        twin_failures += u64::from(original.is_none() || difference.is_some());
        lines.push(format!("name=determinism_twin item={} identical={} first_difference_line={}", twin.name, yes_no(original.is_some() && difference.is_none()), difference.map_or("-".to_string(), |line| line.to_string())));
    }
    counts.insert("twin_differences", twin_failures);
    let mut plain_failures = 0;
    for plain in world.items.iter().filter(|item| item.arm == "plain") {
        let counts_item = world.head_item(&plain.point, &plain.script);
        let equal = counts_item.is_some_and(|counts_item| plain_matches_counts(counts_item, plain));
        plain_failures += u64::from(!equal);
        lines.push(format!("name=plain_versus_counts item={} image_sha256_equal={}", plain.name, yes_no(equal)));
    }
    counts.insert("plain_differences", plain_failures);
    (lines, counts)
}

fn freshness_lines(sets: &Sets<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    for (variant, tag) in FRESHNESS_VARIANTS.iter().enumerate() {
        let reading = judge_freshness_rule(variant, &sets.training, &sets.script, &sets.geometry);
        lines.push(format!(
            "name=freshness_rule variant={tag} threshold={} flagged_in_training={} training_failures={} training_failure_labels={} script_flagged={} script_unflagged={} script_mismatches={} script_mismatch_labels={} geometry_flagged={} geometry_unflagged={} geometry_mismatches={} geometry_mismatch_labels={} verdict={}",
            reading.threshold.map_or("unread".to_string(), |value| value.to_string()),
            reading.flagged_in_training,
            reading.defining_failures.len(),
            first_labels(&reading.defining_failures),
            reading.script.flagged,
            reading.script.unflagged,
            reading.script.mismatches.len(),
            first_labels(&reading.script.mismatches),
            reading.geometry.flagged,
            reading.geometry.unflagged,
            reading.geometry.mismatches.len(),
            first_labels(&reading.geometry.mismatches),
            reading.verdict().tag()
        ));
    }
    lines
}

fn manipulation_lines(world: &World<'_>) -> Vec<String> {
    let first_mount_by_script = world.first_mount_by_script(scripts::TRAINING_POINT);
    let every: Vec<&OperationView> = world.all_views_at(scripts::TRAINING_POINT);
    let reading = judge_manipulation(&first_mount_by_script, &every);
    let mut lines: Vec<String> = reading
        .flagged_by_script
        .iter()
        .map(|(script, count, selectors)| format!("name=freshness_manipulation_script script={script} flagged_operations={count} selectors={}", list_text(&selectors.iter().cloned().collect::<Vec<_>>())))
        .collect();
    lines.push(format!(
        "name=freshness_manipulation scripts={} position_follows={} count_follows={} retouched_operations={} retouched_and_flagged={} retouched_flagged_labels={}",
        reading.flagged_by_script.len(),
        yes_no(reading.position_follows),
        yes_no(reading.count_follows),
        reading.retouch_operations,
        reading.retouch_flagged.len(),
        list_text(&reading.retouch_flagged)
    ));
    lines
}

fn ordinal_lines(world: &World<'_>, point: &str) -> Vec<String> {
    let by_script = world.first_mount_by_script(point);
    let equality = judge_ordinal_equality(&by_script);
    let mut lines = vec![format!(
        "name=ordinal_equality point={point} script_pairs={} flag_depends_only_on_the_ordinal={} first_difference={}",
        equality.pairs,
        yes_no(equality.pairs > 0 && equality.first_difference.is_none()),
        equality.first_difference.clone().unwrap_or_else(|| "-".to_string())
    )];
    let Some(walk_forward) = by_script.get("walk_forward") else {
        return lines;
    };
    let fillers: Vec<(String, u64, Vec<&OperationView>)> = by_script
        .iter()
        .filter_map(|(script, views)| {
            let count = script.strip_prefix("filler_create_").or_else(|| script.strip_prefix("filler_young_"))?.parse().ok()?;
            Some((script.clone(), count, views.clone()))
        })
        .collect();
    let shift = judge_ordinal_shift(walk_forward, &fillers);
    for row in &shift.rows {
        lines.push(format!(
            "name=ordinal_shift_row point={point} script={} filler_count={} first_ordinal={} first_flagged_ordinal={} first_flagged_selector={}",
            row.script,
            row.filler_count,
            row.first_ordinal.map_or("-".to_string(), |value| value.to_string()),
            row.first_flagged_ordinal.map_or("-".to_string(), |value| value.to_string()),
            if row.first_flagged_selector.is_empty() { "-" } else { &row.first_flagged_selector }
        ));
    }
    lines.push(format!(
        "name=ordinal_shift point={point} walk_forward_first_flagged_ordinal={} fillers={} verdict={}",
        shift.walk_forward_first_flagged_ordinal.map_or("-".to_string(), |value| value.to_string()),
        shift.rows.len(),
        shift.verdict
    ));
    lines
}

fn quantity_lines(world: &World<'_>, sets: &Sets<'_>) -> Vec<String> {
    let pooled_views: Vec<&OperationView> = world.all_views_at(scripts::TRAINING_POINT).into_iter().filter(|view| view.epoch == 0).collect();
    let mut lines = Vec::new();
    for (quantity_position, name) in ["stale_nodes", "chain_depth", "released_backlog", "publishes_since_flag"].iter().enumerate() {
        for relative in [false, true] {
            let pick = |views: &[&OperationView]| -> Vec<(f64, bool)> {
                views.iter().map(|view| {
                    let (_, absolute, ratio) = view.quantities()[quantity_position];
                    (if relative { ratio } else { absolute }, view.flagged)
                }).collect()
            };
            let reading = judge_stored_quantity(&pick(&pooled_views), &pick(&sets.geometry));
            lines.push(format!(
                "name=stored_quantity quantity={name} relative={} threshold={:.4} pooled_errors={} flagged_values={} unflagged_values={} geometry_errors={} verdict={}",
                yes_no(relative),
                reading.pooled.threshold,
                reading.pooled.errors,
                reading.pooled.flagged_values,
                reading.pooled.unflagged_values,
                reading.geometry_errors,
                reading.verdict.tag()
            ));
        }
    }
    lines
}

fn presweep_lines(world: &World<'_>, point: &str) -> Vec<String> {
    let mut points = Vec::new();
    let mut scripts_of_the_sweep: Vec<(u64, String)> = vec![(0, "walk_forward".to_string())];
    scripts_of_the_sweep.extend([1, 2, 4, 8, 16, 32, 64].map(|count| (count, format!("presweep_{count}"))));
    for (count, script) in scripts_of_the_sweep {
        let Some(views) = world.views_of(point, &script) else {
            continue;
        };
        let first_mount_views = first_mount(views);
        let Some(first) = first_mount_views.iter().find(|view| view.selector == "first") else {
            continue;
        };
        let (flagged_ordinal, selector) = first_flagged(&first_mount_views);
        points.push(SweepPoint {
            presweep_count: count,
            flagged: first.flagged,
            stale_nodes: first.stale_nodes,
            publishes_since_flag: first.publishes_since_flag,
            final_nodes: first.final_nodes,
            first_flagged_ordinal: flagged_ordinal,
            first_flagged_selector: selector,
        });
    }
    let mut lines: Vec<String> = points
        .iter()
        .map(|sweep| {
            format!(
                "name=presweep_point point={point} presweep_count={} first_flagged={} stale_nodes={} publishes_since_flag={} final_nodes={} first_flagged_ordinal={} first_flagged_selector={}",
                sweep.presweep_count,
                yes_no(sweep.flagged),
                sweep.stale_nodes,
                sweep.publishes_since_flag,
                sweep.final_nodes,
                sweep.first_flagged_ordinal.map_or("-".to_string(), |value| value.to_string()),
                if sweep.first_flagged_selector.is_empty() { "-" } else { &sweep.first_flagged_selector }
            )
        })
        .collect();
    let reading = judge_presweep(&points);
    lines.push(format!(
        "name=presweep point={point} flagged_counts={} unflagged_counts={} between={} verdict={}",
        list_text(&reading.flagged_counts.iter().map(u64::to_string).collect::<Vec<_>>()),
        list_text(&reading.unflagged_counts.iter().map(u64::to_string).collect::<Vec<_>>()),
        reading.between.map_or("-".to_string(), |(flagged, unflagged)| format!("{flagged}..{unflagged}")),
        reading.verdict
    ));
    lines
}

/// 一个点上 `mount_then_first` 的行四读数（前提、集合、带出的那一步）；缺项交 `None`。
pub(super) fn mount_reading_at(world: &World<'_>, point: &str) -> Option<(MountReading, Vec<PublishRecord>)> {
    let walk_forward = world.head_item(point, "walk_forward")?;
    let walk_views = world.views_of(point, "walk_forward")?;
    let first_in_walk = walk_views.iter().find(|view| view.selector == "first")?;
    let spike: NodeSet = first_in_walk.seed.union(&first_in_walk.pulled).copied().collect();
    let _ = walk_forward;
    let mounted = world.head_item(point, "mount_then_first")?;
    let mounted_views = world.views_of(point, "mount_then_first")?;
    let first_after = mounted_views.iter().find(|view| view.epoch == 1 && view.selector == "first")?;
    let remount = mounted.remounts.iter().find(|remount| remount.epoch == 1)?;
    let steps: Vec<(String, NodeSet)> = remount.publishes.iter().map(|publish| (mount_step_name(publish), publish.final_set.clone())).collect();
    Some((judge_mount_carries(first_after.flagged, &spike, &steps), remount.publishes.clone()))
}

fn mount_lines(world: &World<'_>, point: &str, crates_root: Option<&str>) -> Vec<String> {
    let Some((reading, publishes)) = mount_reading_at(world, point) else {
        return vec![format!("name=mount_carries point={point} verdict=unread reason=the_walk_forward_or_mount_then_first_item_is_missing")];
    };
    let mut lines = vec![format!(
        "name=mount_carries point={point} first_after_the_mount_flagged={} nodes_of_the_spike={} carried_by_the_mount={} share={:.3} verdict={} remount_publishes={}",
        yes_no(reading.premise_flagged_after_the_mount),
        reading.nodes_of_the_spike,
        reading.carried,
        reading.share,
        reading.verdict,
        publishes.len()
    )];
    let steps_text: Vec<String> = reading.shares_by_step.iter().map(|(step, count)| format!("{step}:{count}")).collect();
    let (site, plan_function) = reading.located_step.as_ref().map_or(("-".to_string(), "-"), |step| {
        let record = publishes.iter().find(|publish| &mount_step_name(publish) == step);
        (record.map_or("-".to_string(), |publish| publish.site.clone()), plan_function_of_the_mount_step(step))
    });
    let plan_position = crates_root.map_or("position_pending".to_string(), |root| {
        let source = std::fs::read_to_string(format!("{root}/crates/singlefs-core/src/mount.rs")).unwrap_or_default();
        line_of_the_function(&source, plan_function).map_or("position_pending".to_string(), |line| format!("crates/singlefs-core/src/mount.rs:{line}"))
    });
    lines.push(format!(
        "name=mount_carried_step point={point} carried_nodes_by_first_step={} located_step={} verdict={} first_merge_site={site} plan_function={plan_function} plan_position={plan_position}",
        list_text(&steps_text),
        reading.located_step.clone().unwrap_or_else(|| "-".to_string()),
        reading.one_step_verdict
    ));
    lines
}

/// Q10：先碰再挂载与两次挂载、挂载前碰的不是同一批：逐项报，不判。
fn mount_variant_lines(world: &World<'_>, point: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for script in ["first_mount_first", "mount_twice_first", "young_mount_first", "old_mount_old"] {
        let (Some(item), Some(views)) = (world.head_item(point, script), world.views_of(point, script)) else {
            continue;
        };
        let carried_by_the_mount: NodeSet = item.remounts.iter().flat_map(|remount| remount.publishes.iter()).flat_map(|publish| publish.final_set.iter().copied()).collect();
        let before_the_mount: NodeSet = views.iter().filter(|view| view.epoch == 0).flat_map(|view| view.seed.union(&view.pulled).copied()).collect();
        for view in views.iter().filter(|view| view.epoch > 0) {
            let nodes: NodeSet = view.seed.union(&view.pulled).copied().collect();
            lines.push(format!(
                "name=mount_variant item={} operation={} step_index={} flagged={} fresh_mount_wide={} fresh_ever_wide={} nodes={} in_the_mount_publishes={} touched_before_the_mount={} not_in_the_mount_publishes={}",
                item.name,
                view.name,
                view.step_index,
                yes_no(view.flagged),
                view.fresh[0],
                view.fresh[2],
                nodes.len(),
                nodes.intersection(&carried_by_the_mount).count(),
                nodes.intersection(&before_the_mount).count(),
                nodes.difference(&carried_by_the_mount).count()
            ));
        }
    }
    lines
}

/// 行五（Q11、Q12、Q13）：一个点上每个旗上操作与它的可比操作的行表差。
fn extra_lines(world: &World<'_>, point: &str, crates_root: Option<&str>) -> Vec<String> {
    let candidates: Vec<OperationView> = world.all_views_at(point).into_iter().cloned().collect();
    let mut lines = Vec::new();
    let steps_of = |view: &OperationView| -> Option<BTreeMap<&'static str, u64>> {
        let item = world.items.iter().find(|item| item.name == view.item)?;
        let operation = item.operations.iter().find(|operation| operation.step_index == view.step_index)?;
        rows_of_the_operation(item, operation)
    };
    let rounds_of = |view: &OperationView| -> Vec<(String, u64)> {
        world
            .items
            .iter()
            .find(|item| item.name == view.item)
            .and_then(|item| item.operations.iter().find(|operation| operation.step_index == view.step_index))
            .map_or_else(Vec::new, |operation| operation.settles.iter().flat_map(|settle| settle.rounds_detail.iter().cloned()).collect())
    };
    for target in candidates.iter().filter(|view| view.flagged && view.epoch == 0) {
        let comparables = pick_comparables(target, &candidates);
        let label = format!("{}#{}", target.item, target.step_index);
        let (Some(first), Some(second)) = (comparables.first(), comparables.get(1)) else {
            lines.push(format!("name=extra_steps operation={label} comparables={} verdict=no_comparable_operation", comparables.len()));
            continue;
        };
        let (Some(rows), Some(first_rows), Some(second_rows)) = (steps_of(target), steps_of(first), steps_of(second)) else {
            lines.push(format!("name=extra_steps operation={label} verdict=unread reason=a_judgement_reading_is_missing"));
            continue;
        };
        let total = |step_rows: &BTreeMap<&'static str, u64>| i64_from_u64(step_rows.values().sum::<u64>());
        let extra = total(&rows) - total(&first_rows);
        let noise = (total(&second_rows) - total(&first_rows)).unsigned_abs();
        let differences: Vec<(String, i64)> = rows.iter().map(|(row, steps)| ((*row).to_string(), i64_from_u64(*steps) - i64_from_u64(first_rows.get(row).copied().unwrap_or(0)))).collect();
        let (share, top) = top_rows_share(&differences, extra);
        let verdict = classify_extra(extra, noise, share);
        let top_text: Vec<String> = top.iter().map(|(row, difference)| format!("{row}:{difference}")).collect();
        let round_text: Vec<String> = round_shares(&rounds_of(target), &rounds_of(first)).iter().map(|(round, exit, round_share)| format!("{round}:{exit}:{round_share:.3}")).collect();
        lines.push(format!(
            "name=extra_steps operation={label} selector={} comparable_first={}#{} comparable_second={}#{} steps={} steps_first={} steps_second={} extra={extra} noise={noise} top_rows={} top_share={share:.3} round_shares_of_the_raw_difference={} verdict={verdict}",
            target.selector,
            first.item,
            first.step_index,
            second.item,
            second.step_index,
            total(&rows),
            total(&first_rows),
            total(&second_rows),
            list_text(&top_text),
            list_text(&round_text)
        ));
        if verdict == "located_in_the_top_rows" {
            for (row, difference) in &top {
                lines.push(format!("name=extra_steps_position operation={label} row={row} difference={difference} position={}", position_of_the_row(row, crates_root)));
            }
        }
        let final_of = |view: &OperationView| i64_from_usize(view.seed.union(&view.pulled).count());
        lines.push(format!(
            "name=extra_steps_per_node operation={label} extra_nodes={} steps_per_extra_node={}",
            final_of(target) - final_of(first),
            steps_per_extra_node(extra, final_of(target) - final_of(first)).map_or("not_defined_without_positive_extra".to_string(), |value| format!("{value:.3}"))
        ));
    }
    lines
}

/// 旗清单（Q1）：每个项每个用户操作一行。
fn operation_lines(world: &World<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    for item in &world.items {
        for view in &world.views[&item.name] {
            lines.push(format!(
                "name=operation_row item={} mount_epoch={} ordinal_user={} step_index={} operation={} final_nodes={} flagged={} flagged_at_the_low_line={} flagged_at_the_high_line={} median_inflated={} fresh_mount_wide={} fresh_mount_narrow={} fresh_ever_wide={} fresh_ever_narrow={} seed_nodes={} pulled_nodes={} stale_nodes={} chain_depth={} released_backlog={} publishes_since_flag={}",
                view.item,
                view.epoch,
                view.ordinal_user,
                view.step_index,
                view.name,
                view.final_nodes,
                yes_no(view.flagged),
                yes_no(view.flag_by_line[LOW_FLOOR_LINE_POSITION]),
                yes_no(view.flag_by_line[2]),
                yes_no(view.median_inflated),
                view.fresh[0],
                view.fresh[1],
                view.fresh[2],
                view.fresh[3],
                view.seed.len(),
                view.pulled.len(),
                view.stale_nodes,
                view.chain_depth,
                view.released_backlog,
                view.publishes_since_flag
            ));
        }
    }
    lines
}

/// 复现闸（Q0）：每个点的 `walk_forward` 里旗上操作的个数；返回行与每点的判定。
fn gate_lines(world: &World<'_>) -> (Vec<String>, Vec<(String, &'static str)>) {
    let mut lines = Vec::new();
    let mut verdicts = Vec::new();
    for point in world.points() {
        let Some(views) = world.views_of(&point, "walk_forward") else {
            continue;
        };
        let nodes: Vec<u64> = first_mount(views).iter().map(|view| view.final_nodes).collect();
        let gate = reproduction_gate(&nodes);
        lines.push(format!(
            "name=reproduction_gate point={point} operations={} median_final_nodes={} line_at_the_main_floor={} flagged_at_14={} flagged_at_28={} flagged_at_56={} verdict={}",
            nodes.len(),
            lower_median(&nodes),
            spike_line(SPIKE_FLOOR, &nodes),
            gate.flagged_by_line[0],
            gate.flagged_by_line[1],
            gate.flagged_by_line[2],
            gate.verdict
        ));
        verdicts.push((point, gate.verdict));
    }
    (lines, verdicts)
}

/// 几何敏感性（第八节）：同一个量在每个取样点上的判定串；一个点与训练点判不同记不稳定。
fn geometry_sensitivity_line(quantity: &str, per_point: &[(String, String)]) -> String {
    let stable = per_point.windows(2).all(|pair| pair[0].1 == pair[1].1);
    let text: Vec<String> = per_point.iter().map(|(point, verdict)| format!("{point}:{verdict}")).collect();
    format!("name=geometry_sensitivity quantity={quantity} points={} per_point={} stable={}", per_point.len(), list_text(&text), yes_no(stable))
}

fn geometry_sensitivity_lines(world: &World<'_>, sets: &Sets<'_>, gate: &[(String, &'static str)], crates_root: Option<&str>) -> Vec<String> {
    let mut lines = Vec::new();
    let _ = crates_root;
    let points: Vec<String> = world.points().into_iter().collect();
    lines.push(geometry_sensitivity_line("reproduction_gate", &gate.iter().map(|(point, verdict)| (point.clone(), (*verdict).to_string())).collect::<Vec<_>>()));
    let threshold = threshold_from_training(&sets.training, 0);
    let rule: Vec<(String, String)> = points
        .iter()
        .map(|point| {
            let views = world.all_views_at(point);
            let verdict = threshold.map_or("not_run".to_string(), |threshold| format!("mismatches_{}", mismatches_at(&views, 0, threshold).mismatches.len()));
            (point.clone(), verdict)
        })
        .collect();
    lines.push(geometry_sensitivity_line("freshness_rule_mount_wide", &rule));
    let equality: Vec<(String, String)> = points.iter().map(|point| (point.clone(), yes_no(judge_ordinal_equality(&world.first_mount_by_script(point)).first_difference.is_none()).to_string())).collect();
    lines.push(geometry_sensitivity_line("ordinal_equality", &equality));
    let mount: Vec<(String, String)> = points.iter().map(|point| (point.clone(), mount_reading_at(world, point).map_or("unread", |(reading, _)| reading.verdict).to_string())).collect();
    lines.push(geometry_sensitivity_line("mount_carries", &mount));
    let located: Vec<(String, String)> = points
        .iter()
        .map(|point| {
            let verdicts: BTreeSet<String> = extra_lines(world, point, None).iter().filter_map(|line| line.rsplit_once("verdict=").map(|(_, verdict)| verdict.to_string())).filter(|verdict| !verdict.contains(' ')).collect();
            (point.clone(), list_text(&verdicts.into_iter().collect::<Vec<_>>()))
        })
        .collect();
    lines.push(geometry_sensitivity_line("extra_steps_location", &located));
    lines
}

/// 读完全部产物之后的报告：每个量各占一行，末尾 `name=verdict`。
pub(super) fn report(items: &[MeasuredItem], crates_root: Option<&str>) -> Vec<String> {
    let world = World::new(items);
    let sets = sets_of(&world);
    let (check_lines, failures) = checks_lines(&world, items);
    let (gate_text, gate_verdicts) = gate_lines(&world);
    let mut lines = Vec::new();
    lines.extend(check_lines);
    lines.extend(gate_text);
    lines.extend(operation_lines(&world));
    lines.extend(freshness_lines(&sets));
    lines.extend(manipulation_lines(&world));
    for point in world.points() {
        lines.extend(ordinal_lines(&world, &point));
        lines.extend(mount_lines(&world, &point, crates_root));
        lines.extend(extra_lines(&world, &point, crates_root));
    }
    lines.extend(quantity_lines(&world, &sets));
    lines.extend(presweep_lines(&world, scripts::TRAINING_POINT));
    for point in world.points() {
        lines.extend(mount_variant_lines(&world, &point));
        if point != scripts::TRAINING_POINT {
            lines.extend(presweep_lines(&world, &point));
        }
    }
    lines.extend(geometry_sensitivity_lines(&world, &sets, &gate_verdicts, crates_root));
    let count = |name: &str| failures.get(name).copied().unwrap_or(0);
    let zero_or = |name: &str| if count(name) == 0 { "zero".to_string() } else { count(name).to_string() };
    let training_gate = gate_verdicts.iter().find(|(point, _)| point == scripts::TRAINING_POINT).map_or("not_run", |(_, verdict)| *verdict);
    lines.push(format!(
        "name=verdict items_read={} items_without_item_done={} items_refused={} constants_mismatches={} history_mismatches={} script_sequence_mismatches={} closure_model_mismatches={} final_set_not_covering={} final_nodes_mismatches={} edge_walk_mismatches={} live_units_shortfalls={} negative_control_over_four_rounds={} twin_differences={} plain_differences={} reproduction_at_the_training_point={training_gate}",
        world.items.len(),
        zero_or("items_without_item_done"),
        zero_or("items_refused"),
        zero_or("constants_mismatches"),
        zero_or("history_mismatches"),
        zero_or("script_sequence_mismatches"),
        zero_or("closure_model_mismatches"),
        zero_or("final_set_not_covering"),
        zero_or("final_nodes_mismatches"),
        zero_or("edge_walk_mismatches"),
        zero_or("live_units_shortfalls"),
        zero_or("negative_control_over_four_rounds"),
        zero_or("twin_differences"),
        zero_or("plain_differences")
    ));
    lines
}

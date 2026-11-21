//! 不动点上界：上界的判、收集各写的上界、各段历史与上界的判定行。

use super::*;

/// 判据（登记第六节 Q1）的门槛：最小值 ≥ 0 判「P1 是这批写的上界」。
const MINIMUM_SLOTS_FOR_AN_UPPER_BOUND: i64 = 0;
/// 判据（登记第六节 Q5）的门槛：G10 为真的状态数 ≥ 1 判「可达」。
pub(super) const MINIMUM_SLOTS_ENOUGH_AND_ALIGNED_PAIRS_SHORT_STATES_FOR_REACHABLE: u64 = 1;

pub(super) fn checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_given_threshold(
    tally: &WriteTally,
    threshold: i64,
) -> Option<bool> {
    tally
        .checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum
        .map(|minimum| minimum >= threshold)
}

pub(super) fn checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(
    tally: &WriteTally,
) -> Option<bool> {
    checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_given_threshold(
        tally,
        MINIMUM_SLOTS_FOR_AN_UPPER_BOUND,
    )
}

pub(super) fn slots_enough_and_aligned_pairs_short_is_reachable_given_threshold(
    slots_enough_and_aligned_pairs_short_states: u64,
    threshold: u64,
) -> bool {
    slots_enough_and_aligned_pairs_short_states >= threshold
}

fn text_of(value: Option<i64>) -> String {
    value.map_or_else(|| "none".to_string(), |number| number.to_string())
}

/// F9 的全部观测：逐类汇总、被拒的写、停机原因。
#[derive(Default)]
pub(super) struct FixedPointUpperBoundCollector {
    tallies: std::collections::BTreeMap<String, WriteTally>,
    refusal_counts: std::collections::BTreeMap<&'static str, u64>,
    stop_reasons: Vec<String>,
}

impl FixedPointUpperBoundCollector {
    pub(super) fn record_write(
        &mut self,
        history: &str,
        geometry: &str,
        step: usize,
        write: &PublishedWrite,
    ) {
        let class = format!(
            "{history}/{geometry}/height_before_{}_after_{}",
            write.height_before, write.height_after
        );
        self.tallies.entry(class).or_default().add(write);
        self.tallies
            .entry(format!("{history}/{geometry}/all"))
            .or_default()
            .add(write);
        self.tallies
            .entry("all".to_string())
            .or_default()
            .add(write);
        let records_text: Vec<String> = write
            .true_fixed_point_by_records
            .iter()
            .map(|(device, slots)| format!("{}:{slots}", device.0))
            .collect();
        emit(&format!(
            "name=fixed_point_upper_bound_write history={history} geometry={geometry} step={step} units={} height_before={} height_after={} true_fixed_point={} true_fixed_point_by_records={} checkpoint_cost={} local_checkpoint_cost={} checkpoint_cost_with_ordinary_mapping_changes={} ordinary_deleted={} ordinary_inserted={} checkpoint_cost_with_ordinary_mapping_changes_minus_true={} checkpoint_cost_minus_true={} checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost={}",
            write.units,
            write.height_before,
            write.height_after,
            write.true_fixed_point,
            records_text.join(","),
            write.checkpoint_cost,
            write.local_checkpoint_cost,
            write.checkpoint_cost_with_ordinary_mapping_changes,
            write.ordinary.deleted,
            write.ordinary.inserted,
            write.checkpoint_cost_with_ordinary_mapping_changes_minus_true(),
            write.checkpoint_cost_minus_true(),
            write.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost(),
        ));
    }

    pub(super) fn record_refusal(
        &mut self,
        history: &str,
        geometry: &str,
        step: usize,
        units: usize,
        refusal: &RefusalClass,
    ) {
        *self.refusal_counts.entry(refusal.name()).or_default() += 1;
        let detail = refusal.detail();
        emit(&format!(
            "name=fixed_point_upper_bound_refused history={history} geometry={geometry} step={step} units={units} refusal={} detail={:?}",
            refusal.name(),
            detail
        ));
    }

    fn emit_tallies(&self) {
        for (class, tally) in &self.tallies {
            emit(&format!(
                "name=fixed_point_upper_bound_tally class={class} writes={} checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum={} checkpoint_cost_with_ordinary_mapping_changes_minus_true_negative_writes={} checkpoint_cost_minus_true_minimum={} checkpoint_cost_minus_true_negative_writes={} checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_minimum={} checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum={} true_fixed_point_maximum={}",
                tally.writes,
                text_of(tally.checkpoint_cost_with_ordinary_mapping_changes_minus_true_minimum),
                tally.checkpoint_cost_with_ordinary_mapping_changes_below_true_writes,
                text_of(tally.checkpoint_cost_minus_true_minimum),
                tally.checkpoint_cost_below_true_writes,
                text_of(tally.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_minimum),
                text_of(tally.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum),
                tally.true_fixed_point_maximum,
            ));
        }
    }
}

/// 一次写：发成的进汇总、被拒的记一笔；B3 对拍不等返回 `Err`（停机）。
#[allow(
    clippy::too_many_arguments,
    reason = "八个参数各是一样独立的东西（汇总、历史名、几何名、步号、池、单元数、节点容量、内容缓冲），收成结构体只是换个地方再摊开；编码纪律：参数不按个数限"
)]
pub(super) fn run_one_write(
    collector: &mut FixedPointUpperBoundCollector,
    history: &str,
    geometry: &str,
    step: usize,
    pool: &mut Pool,
    units: usize,
    capacities: CodeTwoTreeNodeCapacities,
    content: &mut ContentBuffer,
) -> Result<WriteOutcome, String> {
    let outcome = write_units_and_account(pool, units, capacities, content)?;
    match &outcome {
        WriteOutcome::Published(write) => collector.record_write(history, geometry, step, write),
        WriteOutcome::Refused(refusal) => {
            collector.record_refusal(history, geometry, step, units, refusal);
        }
    }
    Ok(outcome)
}

/// 登记 5.1 节 HF9-产品容量：第一个文件 1 单元，接五档 n 各一次换写（每档都从同一个 1 单元的版本起，在拷贝上做）。
const PRODUCT_CAPACITY_UNIT_COUNTS: [usize; 5] = [1000, 2500, 4000, 5500, 7000];

/// 登记 5.1 节 HF9-三层压小的容量：叶 8 条、内部 4 个孩子（记账树与中央映射树同一档）。
pub(super) const SMALL_TREE_NODE_CAPACITY: CodeTwoTreeNodeCapacity = CodeTwoTreeNodeCapacity {
    leaf_entries: 8,
    internal_entries: 4,
};

pub(super) const SMALL_TREE_NODE_CAPACITIES: CodeTwoTreeNodeCapacities =
    CodeTwoTreeNodeCapacities::CappedForTests {
        accounting: SMALL_TREE_NODE_CAPACITY,
        central_mapping: SMALL_TREE_NODE_CAPACITY,
    };

fn run_product_capacity_history(
    collector: &mut FixedPointUpperBoundCollector,
    geometry: Geometry,
    content: &mut ContentBuffer,
) -> Result<(), String> {
    let mut base =
        build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
    for (step, units) in PRODUCT_CAPACITY_UNIT_COUNTS.iter().enumerate() {
        let mut fork = base.fork();
        run_one_write(
            collector,
            "product_capacity_history",
            geometry.name,
            step,
            &mut fork,
            *units,
            CodeTwoTreeNodeCapacities::FromTheNodeFormat,
            content,
        )?;
    }
    Ok(())
}

/// 登记 5.1 节 HF9-三层：压小容量，从 1 单元起每次把文件换写得长 `step_units` 个单元，
/// 连写到中央映射树高 ≥ 4（层级 + 1 = 4）为止，取过 2 层与 3 层的每一次写。
/// `step_units` = 64 是登记的那一臂；另补一条臂 `step_units` = 4（登记修订里写明：64 一步只经过 2、3 层各一两次）。
fn run_three_level_history(
    collector: &mut FixedPointUpperBoundCollector,
    history: &str,
    geometry: Geometry,
    step_units: usize,
    content: &mut ContentBuffer,
) -> Result<usize, String> {
    const MOST_STEPS: usize = 400;
    const STOP_AT_HEIGHT: u64 = 4;
    let mut pool = build_pool_after_the_first_file(geometry, SMALL_TREE_NODE_CAPACITIES);
    let mut units = 1;
    let mut writes = 0;
    for step in 0..MOST_STEPS {
        units += step_units;
        match run_one_write(
            collector,
            history,
            geometry.name,
            step,
            &mut pool,
            units,
            SMALL_TREE_NODE_CAPACITIES,
            content,
        )? {
            WriteOutcome::Published(write) => {
                writes += 1;
                if write.height_after >= STOP_AT_HEIGHT {
                    return Ok(writes);
                }
            }
            WriteOutcome::Refused(_) => return Ok(writes),
        }
    }
    Ok(writes)
}

/// 落得下那一判放行了吗：发成，或被拒在固定点（落得下那一判与式子那一判都过了）。
pub(super) fn passed_the_landing_check_and_the_formula(outcome: &WriteOutcome) -> bool {
    match outcome {
        WriteOutcome::Published(_) => true,
        WriteOutcome::Refused(RefusalClass::FixedPointPlacement { .. }) => true,
        WriteOutcome::Refused(RefusalClass::SpaceAdmission { .. } | RefusalClass::Other { .. }) => {
            false
        }
    }
}

/// 一块盘上数据单元落得下的槽对数（段外，两槽都没挡）。
pub(super) fn data_unit_pairs_outside_the_clusters_of(
    allocator: &PoolAllocator,
    device_index: usize,
) -> u64 {
    allocator.devices[device_index]
        .unblocked_slot_pairs_outside_the_cluster_segments(allocator.cluster_segments())
        .0
}

pub(super) fn data_unit_pairs_outside_the_clusters(pool: &Pool, device_index: usize) -> u64 {
    data_unit_pairs_outside_the_clusters_of(&pool.allocator, device_index)
}

fn emit_fixed_point_upper_bound_verdicts(
    collector: &FixedPointUpperBoundCollector,
    edges: &[(&str, EdgeReport)],
) {
    let all = collector.tallies.get("all").cloned().unwrap_or_default();
    let three_levels_mechanism_present = collector
        .tallies
        .iter()
        .filter(|(class, _)| class.starts_with("three_level_history"))
        .any(|(class, tally)| {
            class.ends_with("/all") && tally.checkpoint_cost_below_true_writes > 0
        });
    let edge_cell_built = edges
        .iter()
        .any(|(_, report)| report.refused_in_the_fixed_point > 0);
    let edge_found_in_every_edge_geometry = edges
        .iter()
        .all(|(_, report)| report.edge_not_found_reason.is_none());
    emit(&format!(
        "name=verdict_fixed_point_upper_bound writes={} checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write={:?} checkpoint_cost_with_ordinary_mapping_changes_below_true_writes={} checkpoint_cost_below_true_writes={} checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum={} checkpoint_cost_with_ordinary_mapping_changes_equals_checkpoint_cost_on_every_write={} three_level_history_shows_true_above_checkpoint_cost={three_levels_mechanism_present} edge_cell_fixed_point_refusals_built={edge_cell_built} edge_found_in_every_edge_geometry={edge_found_in_every_edge_geometry} stops={}",
        all.writes,
        checkpoint_cost_with_ordinary_mapping_changes_is_an_upper_bound_on_every_write(&all),
        all.checkpoint_cost_with_ordinary_mapping_changes_below_true_writes,
        all.checkpoint_cost_below_true_writes,
        text_of(all.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum),
        all.checkpoint_cost_with_ordinary_mapping_changes_minus_checkpoint_cost_maximum == Some(0),
        collector.stop_reasons.len(),
    ));
    for (kind, count) in &collector.refusal_counts {
        emit(&format!(
            "name=fixed_point_upper_bound_refusal_count kind={kind} count={count}"
        ));
    }
    for (geometry, report) in edges {
        emit(&format!(
            "name=fixed_point_upper_bound_edge_summary geometry={geometry} pairs_outside_the_clusters={} filled_units={} boundary_units={} window_first={} window_last={} published={} refused_by_space_admission={} refused_in_the_fixed_point={} refused_other={} fixed_point_refusals_in_the_first_sixty_four={} fixed_point_refused_units={:?} first_and_last_space_refused={:?}",
            report.pairs_outside_the_clusters,
            report.filled_units,
            report.boundary_units,
            report.window_first,
            report.window_last,
            report.published,
            report.refused_by_space_admission,
            report.refused_in_the_fixed_point,
            report.refused_other,
            report.fixed_point_refusals_in_the_first_sixty_four,
            report.fixed_point_refused_units,
            report.space_refused_units_first_and_last,
        ));
        emit(&format!(
            "name=fixed_point_upper_bound_edge_search geometry={geometry} edge_found={} filled_units={} lower_bound_units={} lower_bound_refused_tries={:?} bisection_rounds={} bisection_points_tried={} bisection_points_passed={} edge_not_found_reason={:?}",
            report.edge_not_found_reason.is_none(),
            report.filled_units,
            text_of_unsigned(report.lower_bound_units),
            report.lower_bound_refused_tries,
            report.bisection_rounds,
            report.bisection_points_tried,
            report.bisection_points_passed,
            report.edge_not_found_reason.as_deref().unwrap_or("none"),
        ));
    }
}

/// 有一个边历史没找到边（下界没找到、一直放行到上限、或放行与被拒次序不一致）就停机：这样的边界不是观测夹住的，窗口没试，
/// 不能当 Q4 的读数。产物照打（含判决行），停机行由 `main` 打，退出码 3。
pub(super) fn stop_when_an_edge_was_not_found(edges: &[(&str, EdgeReport)]) -> Result<(), String> {
    let not_found: Vec<String> = edges
        .iter()
        .filter_map(|(geometry, report)| {
            report
                .edge_not_found_reason
                .as_ref()
                .map(|reason| format!("{geometry}：{reason}"))
        })
        .collect();
    if not_found.is_empty() {
        Ok(())
    } else {
        Err(format!("没找到边：{}", not_found.join("；")))
    }
}

pub(super) fn run_fixed_point_upper_bound(content: &mut ContentBuffer) -> Result<(), String> {
    let mut collector = FixedPointUpperBoundCollector::default();
    for geometry in [GEOMETRY_DEFAULT, GEOMETRY_NARROW, GEOMETRY_SMALL] {
        run_product_capacity_history(&mut collector, geometry, content)?;
    }
    for geometry in [GEOMETRY_DEFAULT, GEOMETRY_SMALL] {
        let sixty_four = run_three_level_history(
            &mut collector,
            "three_level_history_step_64",
            geometry,
            64,
            content,
        )?;
        let four = run_three_level_history(
            &mut collector,
            "three_level_history_step_4",
            geometry,
            4,
            content,
        )?;
        emit(&format!(
            "name=fixed_point_upper_bound_three_level_histories geometry={} writes_step_64={sixty_four} writes_step_4={four}",
            geometry.name
        ));
    }
    let requested_edges = edge_geometries_narrowed_or_registered(
        std::env::var(EDGE_GEOMETRIES_VARIABLE).ok().as_deref(),
    )?;
    emit(&format!(
        "name=fixed_point_upper_bound_setup edge_geometries={requested_edges} edge_geometries_are_the_registered_and_added_ones={}",
        requested_edges == format!("{},{}", GEOMETRY_NARROW.name, GEOMETRY_SMALL.name)
    ));
    let mut edges = Vec::new();
    for geometry in [GEOMETRY_NARROW, GEOMETRY_SMALL] {
        if requested_edges.split(',').any(|name| name == geometry.name) {
            edges.push((
                geometry.name,
                run_edge_history(&mut collector, geometry, content)?,
            ));
        }
    }
    collector.emit_tallies();
    emit_fixed_point_upper_bound_verdicts(&collector, &edges);
    stop_when_an_edge_was_not_found(&edges)
}

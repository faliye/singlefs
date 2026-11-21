//! 按观测算预留够不够、挪动的门槛判不判得开、几次的读数稳不稳，打每个几何的汇总行与判定行。
use super::*;

/// 条款读法的保留池 P条 = 2 × 5 + (B − 1) × c_max，c_max 取现算的 κ（登记第七节 A2；D16（发布语义） 已定项 1「`df`」那一行、
/// D28（挂载期承诺量） 已定项 4 末条）。代码读法的保留池 P代 = κ。
pub(super) fn clause_reading_reserve(checkpoint_cost: u64) -> u64 {
    CLAUSE_RESERVE_CONSTANT_BLOCKS + EMPTY_PUBLISHES_OF_ONE_ADMISSION_AT_MOST * checkpoint_cost
}

impl ReserveFit {
    fn code_reserve_holds_the_longest_floor_raise_chain(self) -> bool {
        self.code_empty_publishes_held >= PUBLISHES_OF_THE_LONGEST_FLOOR_RAISE_CHAIN
    }
    fn code_reserve_holds_the_empty_publishes_of_one_admission(self) -> bool {
        self.code_empty_publishes_held >= EMPTY_PUBLISHES_OF_ONE_ADMISSION_AT_MOST
    }
    fn clause_reserve_holds_the_longest_floor_raise_chain(self) -> bool {
        self.clause_empty_publishes_held >= PUBLISHES_OF_THE_LONGEST_FLOOR_RAISE_CHAIN
    }
    fn clause_reserve_holds_the_empty_publishes_of_one_admission(self) -> bool {
        self.clause_empty_publishes_held >= EMPTY_PUBLISHES_OF_ONE_ADMISSION_AT_MOST
    }
}

/// 一个几何上的 X3：c实 取取得出的空发布里最大的那次（登记第六节 X3「max c实_G」）；一次都取不出、或最大的是 0，交不出（不当 0）。
pub(super) fn reserve_fit(
    checkpoint_cost: u64,
    actual_empty_publish_slots: &[u64],
) -> Option<ReserveFit> {
    let largest_actual_empty_publish_slots = actual_empty_publish_slots.iter().copied().max()?;
    if largest_actual_empty_publish_slots == 0 {
        return None;
    }
    let code_reserve = checkpoint_cost;
    let clause_reserve = clause_reading_reserve(checkpoint_cost);
    Some(ReserveFit {
        checkpoint_cost,
        largest_actual_empty_publish_slots,
        code_reserve,
        clause_reserve,
        code_empty_publishes_held: code_reserve / largest_actual_empty_publish_slots,
        clause_empty_publishes_held: clause_reserve / largest_actual_empty_publish_slots,
    })
}

/// 判别力自证（登记第八节）：把门槛挪到观测到的最小、最大两个取样点之间（取 ⌈(最小 + 最大) ÷ 2⌉，落在 (最小, 最大] 里），
/// 用同一个比较（n ≥ 门槛）在每个几何上重判：挪过的门槛下判定有真有假才算「翻得动」（`turns`）；与原门槛下的判定一真一假的几何数另报。
/// 观测到的最小与最大相等时没有「之间」，记翻不动（取样点同值，那一格判据不算数）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThresholdDiscrimination {
    observed_minimum: u64,
    observed_maximum: u64,
    moved_threshold: Option<u64>,
    holding_at_the_moved_threshold: usize,
    failing_at_the_moved_threshold: usize,
    changed_from_the_original_threshold: usize,
}

impl ThresholdDiscrimination {
    fn turns(self) -> bool {
        self.holding_at_the_moved_threshold > 0 && self.failing_at_the_moved_threshold > 0
    }
}

fn discrimination_of_a_moved_threshold(
    observed: &[u64],
    original_threshold: u64,
) -> Option<ThresholdDiscrimination> {
    let observed_minimum = observed.iter().copied().min()?;
    let observed_maximum = observed.iter().copied().max()?;
    if observed_minimum == observed_maximum {
        return Some(ThresholdDiscrimination {
            observed_minimum,
            observed_maximum,
            moved_threshold: None,
            holding_at_the_moved_threshold: observed.len(),
            failing_at_the_moved_threshold: 0,
            changed_from_the_original_threshold: 0,
        });
    }
    let moved_threshold = observed_minimum + (observed_maximum - observed_minimum).div_ceil(2);
    let holding_at_the_moved_threshold = observed
        .iter()
        .filter(|held| **held >= moved_threshold)
        .count();
    let changed_from_the_original_threshold = observed
        .iter()
        .filter(|held| (**held >= original_threshold) != (**held >= moved_threshold))
        .count();
    Some(ThresholdDiscrimination {
        observed_minimum,
        observed_maximum,
        moved_threshold: Some(moved_threshold),
        holding_at_the_moved_threshold,
        failing_at_the_moved_threshold: observed.len() - holding_at_the_moved_threshold,
        changed_from_the_original_threshold,
    })
}

/// 一列数的最大、最小、最后一个与个数（产物里报的轨迹三样，登记第八节）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SeriesStatistics {
    count: usize,
    maximum: u64,
    minimum: u64,
    last: u64,
}

fn series_statistics(series: &[u64]) -> Option<SeriesStatistics> {
    Some(SeriesStatistics {
        count: series.len(),
        maximum: series.iter().copied().max()?,
        minimum: series.iter().copied().min()?,
        last: *series.last()?,
    })
}

fn statistics_text(statistics: Option<SeriesStatistics>) -> String {
    match statistics {
        Some(statistics) => format!(
            "count={} max={} min={} last={}",
            statistics.count, statistics.maximum, statistics.minimum, statistics.last
        ),
        None => "count=0 max=unobtainable min=unobtainable last=unobtainable".to_string(),
    }
}

/// 产物里定 κ_G 的三种取法（登记修订）：带树那几版上的最小（保留池最小那一刻，主判定）、最大，与最后那一版。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CheckpointCostOfTheGeometry {
    MinimumOnVersionsWithTrees,
    MaximumOnVersionsWithTrees,
    FinalVersion,
}

impl CheckpointCostOfTheGeometry {
    const ALL: [CheckpointCostOfTheGeometry; 3] = [
        CheckpointCostOfTheGeometry::MinimumOnVersionsWithTrees,
        CheckpointCostOfTheGeometry::MaximumOnVersionsWithTrees,
        CheckpointCostOfTheGeometry::FinalVersion,
    ];

    pub(super) fn label(self) -> &'static str {
        match self {
            CheckpointCostOfTheGeometry::MinimumOnVersionsWithTrees => {
                "minimum_on_versions_with_trees"
            }
            CheckpointCostOfTheGeometry::MaximumOnVersionsWithTrees => {
                "maximum_on_versions_with_trees"
            }
            CheckpointCostOfTheGeometry::FinalVersion => "final_version",
        }
    }
}

fn unit_area_text(unit_area: (u64, u64)) -> String {
    format!("[{},{})", unit_area.0, unit_area.1)
}

/// 一个几何的表头行：几何、环长、单元区（本地算的与分配器的）、根层级、K1、历史的结局与被拒次数。
pub(super) fn geometry_row(report: &HistoryReport) -> String {
    let geometry = report.geometry;
    let local_unit_area = geometry.local_unit_area();
    let allocator_unit_areas = report
        .unit_areas_of_the_allocator
        .iter()
        .map(|unit_area| unit_area_text(*unit_area))
        .collect::<Vec<_>>()
        .join(";");
    let (root_level_text, allocation_record_tree_positions_text) = match report
        .allocation_record_tree_root_level
    {
        Some(root_level) => (
            root_level.to_string(),
            local_allocation_record_tree_positions(root_level, &[local_unit_area, local_unit_area])
                .to_string(),
        ),
        None => ("unobtainable".to_string(), "unobtainable".to_string()),
    };
    format!(
        "name=geometry geometry={} slots_per_region={} pool_size={} device_bytes={} journal_ring_bytes={} local_unit_area={} allocator_unit_areas={} allocation_record_tree_root_level={root_level_text} k1_local={allocation_record_tree_positions_text} end={} refusals={} first_refusal={} publishes={} obtained_publishes={} completes_the_history={}",
        geometry.label(),
        geometry.slots_per_region,
        geometry.pool_size.label(),
        geometry.device_bytes(),
        geometry.journal_ring_bytes(),
        unit_area_text(local_unit_area),
        if allocator_unit_areas.is_empty() { "unobtainable".to_string() } else { allocator_unit_areas },
        report.end.text(),
        report.refusals.len(),
        report.refusals.first().map_or("none", String::as_str),
        report.publishes.len(),
        report.publishes.iter().filter(|observation| observation.fixed_point.is_some()).count(),
        report.completes_the_history(),
    )
}

/// 一次发布一行（登记第五节）：种类、txg、κ（核心层、本地与本地各项）、两条路的固定点、普通分配与实例切换预留的槽数。
pub(super) fn publish_row(label: &str, observation: &PublishObservation) -> String {
    let checkpoint_cost_fields = match observation.checkpoint_cost {
        Some(cost) => format!(
            "built_on={} kappa_core={} kappa_local={} kappa_local_terms={}",
            cost.built_on.label(),
            cost.core,
            cost.local.total(),
            cost.local.terms_text()
        ),
        None => "built_on=not_in_hand kappa_core=unobtainable kappa_local=unobtainable kappa_local_terms=unobtainable".to_string(),
    };
    let fixed_point_fields = match &observation.fixed_point {
        Some(account) => format!(
            "fixed_point_by_roles={} fixed_point_by_records={} ordinary_slots={} instance_switch_slots={}",
            account.by_roles,
            account.by_records_text(),
            account.ordinary_slots,
            account.instance_switch_slots
        ),
        None => "fixed_point_by_roles=unobtainable fixed_point_by_records=unobtainable ordinary_slots=unobtainable instance_switch_slots=unobtainable".to_string(),
    };
    format!(
        "name=publish geometry={label} cycle={} kind={} txg={} obtained={} {checkpoint_cost_fields} {fixed_point_fields}",
        observation.cycle,
        observation.kind.label(),
        observation.txg,
        observation.fixed_point.is_some(),
    )
}

/// 停机条款 S2（本地 κ 与核心层不同）、S3（两条路不同）、S4 的单元区那一项（本地算的单元区与分配器的不同），与 F3 的 A6
/// （取得出的空发布 c实 > 那一次之前的 κ）。每一处一行，空的就是一处都没有。
pub(super) fn stop_findings_of_a_geometry(report: &HistoryReport) -> Vec<String> {
    let label = report.geometry.label();
    let mut findings = Vec::new();
    let local_unit_area = report.geometry.local_unit_area();
    if report
        .unit_areas_of_the_allocator
        .iter()
        .any(|unit_area| *unit_area != local_unit_area)
    {
        findings.push(format!(
            "name=stop clause=s4_unit_area_differs geometry={label} local={} allocator={}",
            unit_area_text(local_unit_area),
            report
                .unit_areas_of_the_allocator
                .iter()
                .map(|unit_area| unit_area_text(*unit_area))
                .collect::<Vec<_>>()
                .join(";")
        ));
    }
    let final_reading = report.final_checkpoint_cost.map(|cost| (u64::MAX, cost));
    let readings = report
        .publishes
        .iter()
        .filter_map(|observation| {
            observation
                .checkpoint_cost
                .map(|cost| (observation.txg, cost))
        })
        .chain(final_reading);
    for (txg, cost) in readings {
        if cost.core != cost.local.total() {
            findings.push(format!(
                "name=stop clause=s2_local_kappa_differs_from_core geometry={label} before_txg={} built_on={} core={} local={} local_terms={}",
                if txg == u64::MAX { "after_the_last".to_string() } else { txg.to_string() },
                cost.built_on.label(),
                cost.core,
                cost.local.total(),
                cost.local.terms_text()
            ));
        }
    }
    for observation in &report.publishes {
        let Some(account) = &observation.fixed_point else {
            continue;
        };
        if account.routes_agree_on_every_device() == Some(false) {
            findings.push(format!(
                "name=stop clause=s3_routes_differ geometry={label} kind={} txg={} fixed_point_by_roles={} fixed_point_by_records={} ordinary_slots={} instance_switch_slots={}",
                observation.kind.label(),
                observation.txg,
                account.by_roles,
                account.by_records_text(),
                account.ordinary_slots,
                account.instance_switch_slots
            ));
        }
        if let (true, Some(actual), Some(cost)) = (
            observation.kind.is_an_empty_publish(),
            observation.actual_empty_publish_cost(),
            observation.checkpoint_cost,
        ) {
            if actual > cost.core {
                findings.push(format!(
                    "name=stop clause=a6_actual_empty_publish_exceeds_kappa geometry={label} kind={} txg={} actual={actual} kappa_core={}",
                    observation.kind.label(),
                    observation.txg,
                    cost.core
                ));
            }
        }
    }
    findings
}

/// PC7 那一行（登记第五节「阳性对照」）与它判不判作废（第十节 F1 = 第十一节 V1）：钩子开着两条路必须不同、钩子关着必须相同、
/// 那一次普通分配槽数 > 0；每一次取得出的空发布普通分配槽数 = 0。
pub(super) fn positive_control_row_and_holds(report: &HistoryReport) -> (String, bool) {
    let label = report.geometry.label();
    let empty_publishes_with_ordinary_slots = report
        .publishes
        .iter()
        .filter(|observation| observation.kind.is_an_empty_publish())
        .filter(|observation| {
            observation
                .fixed_point
                .as_ref()
                .is_some_and(|account| account.ordinary_slots > 0)
        })
        .count();
    match report.positive_control_of_the_routes {
        Some(control) => {
            let holds = control.hook_on_routes_agree == Some(false)
                && control.hook_off_routes_agree == Some(true)
                && control.ordinary_slots > 0
                && empty_publishes_with_ordinary_slots == 0;
            (
                format!(
                    "name=positive_control_routes geometry={label} first_overwrite_txg={} hook_on_routes_agree={} hook_off_routes_agree={} ordinary_slots={} empty_publishes_with_ordinary_slots={empty_publishes_with_ordinary_slots} holds={holds}",
                    control.txg,
                    optional_bool_text(control.hook_on_routes_agree),
                    optional_bool_text(control.hook_off_routes_agree),
                    control.ordinary_slots,
                ),
                holds,
            )
        }
        None => (
            format!(
                "name=positive_control_routes geometry={label} first_overwrite_txg=unobtainable empty_publishes_with_ordinary_slots={empty_publishes_with_ordinary_slots} holds=false"
            ),
            false,
        ),
    }
}

fn optional_bool_text(value: Option<bool>) -> String {
    value.map_or_else(|| "unobtainable".to_string(), |value| value.to_string())
}

/// 一个几何的汇总行（登记第六节 X1–X5、第八节 κ 的轨迹）与判定行。取得出的空发布少于 6 次的，打一行作废（V6）、不出判定。
pub(super) fn emit_the_summary_of_a_geometry(report: &HistoryReport) -> Option<GeometryVerdicts> {
    let label = report.geometry.label();
    let kinds_seen: Vec<PublishKind> = {
        let mut kinds = Vec::new();
        for observation in &report.publishes {
            if !kinds.contains(&observation.kind) {
                kinds.push(observation.kind);
            }
        }
        kinds
    };
    for kind in &kinds_seen {
        let kappa_series: Vec<u64> = report
            .publishes
            .iter()
            .filter(|observation| observation.kind == *kind)
            .filter_map(|observation| observation.checkpoint_cost.map(|cost| cost.core))
            .collect();
        emit(&format!(
            "name=x1_kappa geometry={label} kind={} {}",
            kind.label(),
            statistics_text(series_statistics(&kappa_series))
        ));
    }
    let kappa_on_versions_with_trees: Vec<u64> = report
        .publishes
        .iter()
        .filter_map(|observation| observation.checkpoint_cost)
        .filter(|cost| cost.built_on == BuiltOn::VersionWithTrees)
        .map(|cost| cost.core)
        .collect();
    emit(&format!(
        "name=x1_kappa geometry={label} kind=all_built_on_versions_with_trees {}",
        statistics_text(series_statistics(&kappa_on_versions_with_trees))
    ));
    let final_kappa = report.final_checkpoint_cost.map(|cost| cost.core);
    let first_file_version_kappa = report
        .publishes
        .iter()
        .skip_while(|observation| observation.kind != PublishKind::FirstFileVersion)
        .nth(1)
        .and_then(|observation| observation.checkpoint_cost)
        .map(|cost| cost.core);
    let peak = kappa_on_versions_with_trees
        .iter()
        .copied()
        .chain(final_kappa)
        .max();
    let publishes_above_the_first_file_version = first_file_version_kappa.map(|first| {
        kappa_on_versions_with_trees
            .iter()
            .filter(|kappa| **kappa > first)
            .count()
    });
    emit(&format!(
        "name=kappa_trajectory geometry={label} peak={} first_file_version_kappa={} publishes_built_on_a_version_above_it={} final={}",
        peak.map_or_else(|| "unobtainable".to_string(), |value| value.to_string()),
        first_file_version_kappa.map_or_else(|| "unobtainable".to_string(), |value| value.to_string()),
        publishes_above_the_first_file_version.map_or_else(|| "unobtainable".to_string(), |value| value.to_string()),
        final_kappa.map_or_else(|| "unobtainable".to_string(), |value| value.to_string()),
    ));
    let mut actual_empty_publish_slots = Vec::new();
    for kind in kinds_seen
        .iter()
        .copied()
        .filter(|kind| kind.counts_for_the_actual_empty_publish_cost())
    {
        let of_this_kind: Vec<&PublishObservation> = report
            .publishes
            .iter()
            .filter(|observation| observation.kind == kind)
            .collect();
        let obtained: Vec<u64> = of_this_kind
            .iter()
            .filter_map(|observation| observation.actual_empty_publish_cost())
            .collect();
        emit(&format!(
            "name=x2_actual_empty_publish_cost geometry={label} kind={} not_obtained={} {}",
            kind.label(),
            of_this_kind.len() - obtained.len(),
            statistics_text(series_statistics(&obtained))
        ));
        actual_empty_publish_slots.extend(obtained);
    }
    let overwrite_fixed_points: Vec<u64> = report
        .publishes
        .iter()
        .filter(|observation| observation.kind == PublishKind::Overwrite)
        .filter_map(|observation| {
            let account = observation.fixed_point.as_ref()?;
            (account.routes_agree_on_every_device() == Some(true)).then_some(account.by_roles)
        })
        .collect();
    let largest_actual = actual_empty_publish_slots.iter().copied().max();
    let largest_overwrite = overwrite_fixed_points.iter().copied().max();
    let overwrite_excess = match (largest_overwrite, largest_actual) {
        (Some(overwrite), Some(actual)) => (
            overwrite > actual,
            overwrite.saturating_sub(actual).to_string(),
        ),
        (None, _) | (_, None) => (false, "unobtainable".to_string()),
    };
    emit(&format!(
        "name=x5_overwrite_fixed_point geometry={label} {} exceeds_the_largest_actual_empty_publish={} excess_slots={}",
        statistics_text(series_statistics(&overwrite_fixed_points)),
        overwrite_excess.0,
        overwrite_excess.1,
    ));
    let obtained_empty_publishes = actual_empty_publish_slots.len();
    if obtained_empty_publishes < OBTAINED_EMPTY_PUBLISHES_AT_LEAST {
        emit(&format!(
            "name=geometry_void clause=v6_fewer_than_six_obtained_empty_publishes geometry={label} obtained_empty_publishes={obtained_empty_publishes}"
        ));
        return None;
    }
    let kappa_minimum = kappa_on_versions_with_trees.iter().copied().min();
    let kappa_maximum = kappa_on_versions_with_trees.iter().copied().max();
    let mut fits = Vec::new();
    for reading in CheckpointCostOfTheGeometry::ALL {
        let kappa_of_this_reading = match reading {
            CheckpointCostOfTheGeometry::MinimumOnVersionsWithTrees => kappa_minimum,
            CheckpointCostOfTheGeometry::MaximumOnVersionsWithTrees => kappa_maximum,
            CheckpointCostOfTheGeometry::FinalVersion => final_kappa,
        };
        let Some(fit) =
            kappa_of_this_reading.and_then(|kappa| reserve_fit(kappa, &actual_empty_publish_slots))
        else {
            emit(&format!(
                "name=x3_reserve_fit geometry={label} kappa_of_the_geometry={} kappa=unobtainable",
                reading.label()
            ));
            continue;
        };
        emit(&format!(
            "name=x3_reserve_fit geometry={label} kappa_of_the_geometry={} kappa={} largest_actual_empty_publish_slots={} code_reserve={} clause_reserve={} code_empty_publishes_held={} clause_empty_publishes_held={}",
            reading.label(),
            fit.checkpoint_cost,
            fit.largest_actual_empty_publish_slots,
            fit.code_reserve,
            fit.clause_reserve,
            fit.code_empty_publishes_held,
            fit.clause_empty_publishes_held,
        ));
        for (device_number, (first_slot, end_slot)) in report
            .unit_areas_of_the_allocator
            .iter()
            .copied()
            .enumerate()
        {
            let unit_area_slots = end_slot - first_slot;
            let code_millionths =
                u128::from(fit.code_reserve) * 1_000_000 / u128::from(unit_area_slots);
            let clause_millionths =
                u128::from(fit.clause_reserve) * 1_000_000 / u128::from(unit_area_slots);
            emit(&format!(
                "name=x4_reserve_share geometry={label} kappa_of_the_geometry={} device={device_number} unit_area_slots={unit_area_slots} code_reserve_millionths={code_millionths} clause_reserve_millionths={clause_millionths} difference_millionths={}",
                reading.label(),
                clause_millionths - code_millionths,
            ));
        }
        emit(&format!(
            "name=verdict geometry={label} kappa_of_the_geometry={} code_reserve_holds_the_longest_floor_raise_chain={} code_reserve_holds_the_empty_publishes_of_one_admission={} clause_reserve_holds_the_longest_floor_raise_chain={} clause_reserve_holds_the_empty_publishes_of_one_admission={} largest_actual_empty_publish_within_kappa={}",
            reading.label(),
            fit.code_reserve_holds_the_longest_floor_raise_chain(),
            fit.code_reserve_holds_the_empty_publishes_of_one_admission(),
            fit.clause_reserve_holds_the_longest_floor_raise_chain(),
            fit.clause_reserve_holds_the_empty_publishes_of_one_admission(),
            fit.largest_actual_empty_publish_slots <= fit.checkpoint_cost,
        ));
        fits.push((reading, fit));
    }
    Some(GeometryVerdicts {
        geometry: report.geometry,
        fits,
    })
}

/// 判别力自证与几何敏感性（登记第八节）：X3 两个门槛（3 与 7）在两种读法上各挪一次；每个判定在 G-min 对 G-4（同一个 S）、
/// S = 4 对 S = 16（同一档池大小）上各比一次，一真一假记 `stable=false`。都只用 κ_G 取带树那几版上的最小那一种（主判定）。
pub(super) fn emit_discrimination_and_stability(verdicts: &[GeometryVerdicts]) {
    let primary: Vec<(Geometry, ReserveFit)> = verdicts
        .iter()
        .filter_map(|geometry_verdicts| {
            geometry_verdicts
                .fits
                .iter()
                .find(|(reading, _fit)| {
                    *reading == CheckpointCostOfTheGeometry::MinimumOnVersionsWithTrees
                })
                .map(|(_reading, fit)| (geometry_verdicts.geometry, *fit))
        })
        .collect();
    let code_held: Vec<u64> = primary
        .iter()
        .map(|(_geometry, fit)| fit.code_empty_publishes_held)
        .collect();
    let clause_held: Vec<u64> = primary
        .iter()
        .map(|(_geometry, fit)| fit.clause_empty_publishes_held)
        .collect();
    for (reading_name, held) in [("code", &code_held), ("clause", &clause_held)] {
        for original_threshold in [
            PUBLISHES_OF_THE_LONGEST_FLOOR_RAISE_CHAIN,
            EMPTY_PUBLISHES_OF_ONE_ADMISSION_AT_MOST,
        ] {
            match discrimination_of_a_moved_threshold(held, original_threshold) {
                Some(discrimination) => emit(&format!(
                    "name=discrimination item=x3_{reading_name}_reserve original_threshold={original_threshold} geometries={} observed_minimum={} observed_maximum={} moved_threshold={} holding_at_the_moved_threshold={} failing_at_the_moved_threshold={} changed_from_the_original_threshold={} turns={}",
                    held.len(),
                    discrimination.observed_minimum,
                    discrimination.observed_maximum,
                    discrimination.moved_threshold.map_or_else(|| "none_sample_points_equal".to_string(), |threshold| threshold.to_string()),
                    discrimination.holding_at_the_moved_threshold,
                    discrimination.failing_at_the_moved_threshold,
                    discrimination.changed_from_the_original_threshold,
                    discrimination.turns(),
                )),
                None => emit(&format!(
                    "name=discrimination item=x3_{reading_name}_reserve original_threshold={original_threshold} geometries=0"
                )),
            }
        }
    }
    type VerdictOfAFit = fn(ReserveFit) -> bool;
    let verdict_functions: [(&str, VerdictOfAFit); 4] = [
        (
            "code_reserve_holds_the_longest_floor_raise_chain",
            ReserveFit::code_reserve_holds_the_longest_floor_raise_chain,
        ),
        (
            "code_reserve_holds_the_empty_publishes_of_one_admission",
            ReserveFit::code_reserve_holds_the_empty_publishes_of_one_admission,
        ),
        (
            "clause_reserve_holds_the_longest_floor_raise_chain",
            ReserveFit::clause_reserve_holds_the_longest_floor_raise_chain,
        ),
        (
            "clause_reserve_holds_the_empty_publishes_of_one_admission",
            ReserveFit::clause_reserve_holds_the_empty_publishes_of_one_admission,
        ),
    ];
    let fit_of = |geometry: Geometry| {
        primary
            .iter()
            .find(|(candidate, _fit)| *candidate == geometry)
            .map(|(_geometry, fit)| *fit)
    };
    let mut pairs: Vec<(Geometry, Geometry)> = Vec::new();
    for (left, _left_fit) in &primary {
        for (right, _right_fit) in &primary {
            let smallest_against_four_gibibytes = left.slots_per_region == right.slots_per_region
                && matches!(left.pool_size, PoolSize::SmallestTrial { .. })
                && right.pool_size == PoolSize::FourGibibytes;
            let four_against_sixteen = left.slots_per_region
                == ROOT_RING_SLOTS_PER_REGION_MINIMUM_LOCAL
                && right.slots_per_region == ROOT_RING_SLOTS_PER_REGION_MAXIMUM_LOCAL
                && same_pool_size_band(left.pool_size, right.pool_size);
            if smallest_against_four_gibibytes || four_against_sixteen {
                pairs.push((*left, *right));
            }
        }
    }
    for (verdict_name, verdict_of) in verdict_functions {
        for (left, right) in &pairs {
            let (Some(left_fit), Some(right_fit)) = (fit_of(*left), fit_of(*right)) else {
                continue;
            };
            let (left_verdict, right_verdict) = (verdict_of(left_fit), verdict_of(right_fit));
            emit(&format!(
                "name=stability item={verdict_name} pair={}~{} left={left_verdict} right={right_verdict} stable={}",
                left.label(),
                right.label(),
                left_verdict == right_verdict,
            ));
        }
    }
}

/// 两个几何的池大小是不是同一档（G-min 的各次试跑算一档）。
fn same_pool_size_band(left: PoolSize, right: PoolSize) -> bool {
    match (left, right) {
        (PoolSize::SmallestTrial { .. }, PoolSize::SmallestTrial { .. })
        | (PoolSize::OneGibibyte, PoolSize::OneGibibyte)
        | (PoolSize::FourGibibytes, PoolSize::FourGibibytes) => true,
        (PoolSize::SmallestTrial { .. }, PoolSize::OneGibibyte | PoolSize::FourGibibytes)
        | (PoolSize::OneGibibyte, PoolSize::SmallestTrial { .. } | PoolSize::FourGibibytes)
        | (PoolSize::FourGibibytes, PoolSize::SmallestTrial { .. } | PoolSize::OneGibibyte) => {
            false
        }
    }
}

#[cfg(test)]
mod tests;

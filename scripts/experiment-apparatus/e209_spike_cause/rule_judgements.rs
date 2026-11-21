//! 新鲜度、操纵、序号相等、序号位移、存量、预扫、挂载携带各条规则的判器，与附加行的挑选和分类。

use e7_index_bench::i64_from_u64;
use super::*;

// ---- 行一：旗跟「碰到本挂载没带出过的节点」走（Q2、Q3）----

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SetMismatches {
    pub(super) flagged: usize,
    pub(super) unflagged: usize,
    pub(super) mismatches: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuleVerdict {
    Holds,
    DoesNotHold,
    TooFewFlaggedOperations,
    NotRun,
}

impl RuleVerdict {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Self::Holds => "holds",
            Self::DoesNotHold => "does_not_hold",
            Self::TooFewFlaggedOperations => "too_few_flagged_operations",
            Self::NotRun => "not_run",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct RuleReading {
    variant: usize,
    /// 训练集里旗上操作新鲜度的最小值；训练集没有旗上操作交 `None`。
    pub(super) threshold: Option<u64>,
    pub(super) flagged_in_training: usize,
    /// 训练集里新鲜度不小于门槛却不旗的操作。
    pub(super) defining_failures: Vec<String>,
    pub(super) script: SetMismatches,
    pub(super) geometry: SetMismatches,
}

impl RuleReading {
    /// 合成对照的绿：定规则没失败、两类检验集上没有不符（不看检验力）。
    pub(super) fn agrees(&self) -> bool {
        self.threshold.is_some() && self.defining_failures.is_empty() && self.script.mismatches.is_empty() && self.geometry.mismatches.is_empty()
    }

    pub(super) fn verdict(&self) -> RuleVerdict {
        if self.threshold.is_none() {
            RuleVerdict::NotRun
        } else if !self.agrees() {
            RuleVerdict::DoesNotHold
        } else if [&self.script, &self.geometry].iter().any(|set| set.flagged < FLAGGED_OPERATIONS_AT_LEAST || set.unflagged < FLAGGED_OPERATIONS_AT_LEAST) {
            RuleVerdict::TooFewFlaggedOperations
        } else {
            RuleVerdict::Holds
        }
    }
}

fn label_of(view: &OperationView, variant: usize) -> String {
    format!("{}#{}:fresh={}:observed_flag={}", view.item, view.ordinal_user, view.fresh[variant], u8::from(view.flagged))
}

/// 门槛下预测（`fresh ≥ 门槛` 旗上）对观测旗的不符。
pub(super) fn mismatches_at(views: &[&OperationView], variant: usize, threshold: u64) -> SetMismatches {
    SetMismatches {
        flagged: views.iter().filter(|view| view.flagged).count(),
        unflagged: views.iter().filter(|view| !view.flagged).count(),
        mismatches: views.iter().filter(|view| (view.fresh[variant] >= threshold) != view.flagged).map(|view| label_of(view, variant)).collect(),
    }
}

pub(super) fn threshold_from_training(training: &[&OperationView], variant: usize) -> Option<u64> {
    training.iter().filter(|view| view.flagged).map(|view| view.fresh[variant]).min()
}

/// 用训练集定规则（门槛取旗上操作新鲜度的最小值）、在两类检验集上检。
pub(super) fn judge_freshness_rule(variant: usize, training: &[&OperationView], script: &[&OperationView], geometry: &[&OperationView]) -> RuleReading {
    let threshold = threshold_from_training(training, variant);
    let Some(threshold) = threshold else {
        return RuleReading { variant, ..RuleReading::default() };
    };
    RuleReading {
        variant,
        threshold: Some(threshold),
        flagged_in_training: training.iter().filter(|view| view.flagged).count(),
        defining_failures: training.iter().filter(|view| !view.flagged && view.fresh[variant] >= threshold).map(|view| label_of(view, variant)).collect(),
        script: mismatches_at(script, variant, threshold),
        geometry: mismatches_at(geometry, variant, threshold),
    }
}

/// 行一的两个翻面观测（Q3）：三个脚本旗上操作的选择子集合与个数，与「再碰还旗」。
#[derive(Clone, Debug, Default)]
pub(super) struct Manipulation {
    pub(super) flagged_by_script: Vec<(String, usize, BTreeSet<String>)>,
    pub(super) position_follows: bool,
    pub(super) count_follows: bool,
    pub(super) retouch_operations: usize,
    pub(super) retouch_flagged: Vec<String>,
}

pub(super) fn judge_manipulation(first_mount_by_script: &BTreeMap<String, Vec<&OperationView>>, every_operation: &[&OperationView]) -> Manipulation {
    let flagged_by_script: Vec<(String, usize, BTreeSet<String>)> = MANIPULATION_SCRIPTS
        .iter()
        .filter_map(|script| first_mount_by_script.get(*script).map(|views| (script, views)))
        .map(|(script, views)| {
            let selectors: BTreeSet<String> = views.iter().filter(|view| view.flagged).map(|view| view.selector.clone()).collect();
            ((*script).to_string(), views.iter().filter(|view| view.flagged).count(), selectors)
        })
        .collect();
    let distinct_selector_sets: BTreeSet<&BTreeSet<String>> = flagged_by_script.iter().map(|(_, _, selectors)| selectors).collect();
    let distinct_counts: BTreeSet<usize> = flagged_by_script.iter().map(|(_, count, _)| *count).collect();
    let retouched: Vec<&&OperationView> = every_operation.iter().filter(|view| view.fresh[0] == 0).collect();
    Manipulation {
        position_follows: distinct_selector_sets.len() >= 2,
        count_follows: distinct_counts.len() >= 2,
        retouch_operations: retouched.len(),
        retouch_flagged: retouched.iter().filter(|view| view.flagged).map(|view| format!("{}#{}", view.item, view.ordinal_user)).collect(),
        flagged_by_script,
    }
}

// ---- 行二：旗只由挂载之后的第几次操作定（Q4、Q5）----

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct OrdinalEquality {
    pub(super) pairs: usize,
    pub(super) first_difference: Option<String>,
}

/// 同一个点内全部脚本两两比：第 0 次挂载里每个都有值的序号上旗是否相同；一边没有值的序号不比。
pub(super) fn judge_ordinal_equality(first_mount_by_script: &BTreeMap<String, Vec<&OperationView>>) -> OrdinalEquality {
    let scripts: Vec<(&String, &Vec<&OperationView>)> = first_mount_by_script.iter().collect();
    let mut reading = OrdinalEquality::default();
    for (position, (first_script_name, first_script_views)) in scripts.iter().enumerate() {
        for (second_script_name, second_script_views) in &scripts[position + 1..] {
            reading.pairs += 1;
            let shorter = u64::try_from(first_script_views.len().min(second_script_views.len())).expect("个数");
            for ordinal in 1..=shorter {
                if flag_at(first_script_views, ordinal) != flag_at(second_script_views, ordinal) && reading.first_difference.is_none() {
                    reading.first_difference = Some(format!("{first_script_name}/{second_script_name}#{ordinal}"));
                }
            }
        }
    }
    reading
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ShiftRow {
    pub(super) script: String,
    pub(super) filler_count: u64,
    pub(super) first_ordinal: Option<u64>,
    pub(super) first_flagged_ordinal: Option<u64>,
    pub(super) first_flagged_selector: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OrdinalShift {
    pub(super) walk_forward_first_flagged_ordinal: Option<u64>,
    pub(super) rows: Vec<ShiftRow>,
    pub(super) verdict: &'static str,
}

/// 第一个旗上操作的序号与选择子。
pub(super) fn first_flagged(views: &[&OperationView]) -> (Option<u64>, String) {
    views.iter().find(|view| view.flagged).map_or((None, String::new()), |view| (Some(view.ordinal_user), view.selector.clone()))
}

/// 把 `first` 挪到序号 `n + 1`：每个填充脚本里 `first` 的序号、旗与第一个旗上操作；判「旗跟操作走」「旗跟序号走」「都不是」。
pub(super) fn judge_ordinal_shift(walk_forward: &[&OperationView], fillers: &[(String, u64, Vec<&OperationView>)]) -> OrdinalShift {
    let (walk_forward_ordinal, _) = first_flagged(walk_forward);
    let rows: Vec<ShiftRow> = fillers
        .iter()
        .map(|(script, count, views)| {
            let (flagged_ordinal, selector) = first_flagged(views);
            ShiftRow {
                script: script.clone(),
                filler_count: *count,
                first_ordinal: views.iter().find(|view| view.selector == "first").map(|view| view.ordinal_user),
                first_flagged_ordinal: flagged_ordinal,
                first_flagged_selector: selector,
            }
        })
        .collect();
    let follows_the_operation = !rows.is_empty() && rows.iter().all(|row| row.first_ordinal == Some(row.filler_count + 1) && row.first_flagged_ordinal == row.first_ordinal);
    let follows_the_ordinal = !rows.is_empty() && walk_forward_ordinal.is_some() && rows.iter().all(|row| row.first_flagged_ordinal == walk_forward_ordinal);
    let verdict = if follows_the_operation {
        "follows_the_operation"
    } else if follows_the_ordinal {
        "follows_the_ordinal"
    } else {
        "neither"
    };
    OrdinalShift { walk_forward_first_flagged_ordinal: walk_forward_ordinal, rows, verdict }
}

// ---- 行三：旗由攒下的量跨过门槛决定（Q6、Q7）----

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ThresholdFit {
    pub(super) threshold: f64,
    pub(super) errors: usize,
    pub(super) flagged_values: usize,
    pub(super) unflagged_values: usize,
}

fn distinct_values(points: &[(f64, bool)], flagged: bool) -> usize {
    let mut values: Vec<f64> = points.iter().filter(|point| point.1 == flagged).map(|point| point.0).collect();
    values.sort_by(f64::total_cmp);
    values.dedup();
    values.len()
}

/// 使分错数最少的门槛（`值 ≥ 门槛` 旗上）；分错数相同取小的门槛。候选是出现过的每个值。
fn best_threshold(points: &[(f64, bool)]) -> ThresholdFit {
    let mut candidates: Vec<f64> = points.iter().map(|point| point.0).collect();
    candidates.sort_by(f64::total_cmp);
    candidates.dedup();
    let errors_at = |threshold: f64| points.iter().filter(|(value, flagged)| (*value >= threshold) != *flagged).count();
    let mut best = ThresholdFit { threshold: f64::INFINITY, errors: points.len() + 1, flagged_values: 0, unflagged_values: 0 };
    for candidate in candidates {
        let errors = errors_at(candidate);
        if errors < best.errors {
            best.threshold = candidate;
            best.errors = errors;
        }
    }
    best.flagged_values = distinct_values(points, true);
    best.unflagged_values = distinct_values(points, false);
    best
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum QuantityVerdict {
    Holds,
    DoesNotHold,
    OnlyInTheTrainingGeometry,
    TooFewDistinctValues,
    TestGeometryNotRun,
    NotRun,
}

impl QuantityVerdict {
    pub(super) fn tag(self) -> &'static str {
        match self {
            Self::Holds => "holds",
            Self::DoesNotHold => "does_not_hold",
            Self::OnlyInTheTrainingGeometry => "only_in_the_training_geometry",
            Self::TooFewDistinctValues => "too_few_distinct_values",
            Self::TestGeometryNotRun => "test_geometry_not_run",
            Self::NotRun => "not_run",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct QuantityReading {
    pub(super) pooled: ThresholdFit,
    pub(super) geometry_errors: usize,
    pub(super) verdict: QuantityVerdict,
}

/// 一格（一个量、绝对或相对）：池化（训练集加检验集一）里找门槛，再用这个门槛（不重拟）在检验集二上数分错。
pub(super) fn judge_stored_quantity(pooled: &[(f64, bool)], geometry: &[(f64, bool)]) -> QuantityReading {
    let fit = best_threshold(pooled);
    let geometry_errors = geometry.iter().filter(|(value, flagged)| (*value >= fit.threshold) != *flagged).count();
    let verdict = if pooled.is_empty() || fit.flagged_values == 0 {
        QuantityVerdict::NotRun
    } else if fit.errors > 0 {
        QuantityVerdict::DoesNotHold
    } else if fit.flagged_values < 2 || fit.unflagged_values < 2 {
        QuantityVerdict::TooFewDistinctValues
    } else if geometry.is_empty() {
        QuantityVerdict::TestGeometryNotRun
    } else if geometry_errors > 0 {
        QuantityVerdict::OnlyInTheTrainingGeometry
    } else {
        QuantityVerdict::Holds
    };
    QuantityReading { pooled: fit, geometry_errors, verdict }
}

/// Q7 的一个取样点：`first` 前面先做 m 次冷文件覆盖写之后，`first` 那一步的读数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SweepPoint {
    pub(super) presweep_count: u64,
    pub(super) flagged: bool,
    pub(super) stale_nodes: u64,
    pub(super) publishes_since_flag: u64,
    pub(super) final_nodes: u64,
    pub(super) first_flagged_ordinal: Option<u64>,
    pub(super) first_flagged_selector: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SweepReading {
    pub(super) flagged_counts: Vec<u64>,
    pub(super) unflagged_counts: Vec<u64>,
    pub(super) between: Option<(u64, u64)>,
    pub(super) verdict: &'static str,
}

pub(super) fn judge_presweep(points: &[SweepPoint]) -> SweepReading {
    let flagged_counts: Vec<u64> = points.iter().filter(|point| point.flagged).map(|point| point.presweep_count).collect();
    let unflagged_counts: Vec<u64> = points.iter().filter(|point| !point.flagged).map(|point| point.presweep_count).collect();
    let between = match (flagged_counts.iter().max(), unflagged_counts.iter().min()) {
        (Some(largest_flagged), Some(smallest_unflagged)) if largest_flagged < smallest_unflagged => Some((*largest_flagged, *smallest_unflagged)),
        _ => None,
    };
    let verdict = if flagged_counts.is_empty() || unflagged_counts.is_empty() {
        "threshold_sides_not_both_sampled"
    } else if between.is_some() {
        "threshold_between_two_sample_points"
    } else {
        "not_monotone_in_the_presweep_count"
    };
    SweepReading { flagged_counts, unflagged_counts, between, verdict }
}

// ---- 行四：重挂载之后不旗，是因为挂载那一步已经把那批节点带出（Q8、Q9）----

#[derive(Clone, Debug, PartialEq)]
pub(super) struct MountReading {
    pub(super) premise_flagged_after_the_mount: bool,
    pub(super) nodes_of_the_spike: usize,
    pub(super) carried: usize,
    pub(super) share: f64,
    pub(super) verdict: &'static str,
    /// 带出的节点各占挂载自己的哪一步（步名 → 首次出现在这一步的节点数）。
    pub(super) shares_by_step: Vec<(String, usize)>,
    /// 最大一步占带出节点的比例 ≥ 线时它的步名；不到线交 `None`。
    pub(super) located_step: Option<String>,
    pub(super) one_step_verdict: &'static str,
}

/// `spike_nodes` 是不挂载的项里尖峰那次操作的 `seed ∪ pulled`；`mount_steps` 是挂载自己的发布（写行、暖机）按次序各步的定下集合；
/// `flagged_after_the_mount` 是挂载之后第一次覆盖写是否旗上（前提）。
pub(super) fn judge_mount_carries(flagged_after_the_mount: bool, spike_nodes: &NodeSet, mount_steps: &[(String, NodeSet)]) -> MountReading {
    let carried: NodeSet = spike_nodes.iter().copied().filter(|node| mount_steps.iter().any(|(_, set)| set.contains(node))).collect();
    let share = if spike_nodes.is_empty() { 0.0 } else { carried.len() as f64 / spike_nodes.len() as f64 };
    let verdict = if flagged_after_the_mount {
        "mount_did_not_stop_the_spike"
    } else if share >= MOUNT_SHARE_CARRIED_AT_LEAST {
        "mount_carried_the_nodes"
    } else if share < MOUNT_SHARE_NOT_CARRIED_BELOW {
        "not_flagged_but_not_carried_by_the_mount"
    } else {
        "partly_carried"
    };
    let mut shares_by_step: Vec<(String, usize)> = mount_steps.iter().map(|(name, _)| (name.clone(), 0)).collect();
    for node in &carried {
        if let Some(position) = mount_steps.iter().position(|(_, set)| set.contains(node)) {
            shares_by_step[position].1 += 1;
        }
    }
    let largest = shares_by_step.iter().max_by_key(|(_, count)| *count).cloned();
    let located_step = largest.filter(|(_, count)| !carried.is_empty() && *count as f64 / carried.len() as f64 >= ONE_STEP_SHARE_AT_LEAST).map(|(name, _)| name);
    let one_step_verdict = if carried.is_empty() {
        "no_carried_nodes"
    } else if located_step.is_some() {
        "located_one_step"
    } else {
        "spread_over_several_steps"
    };
    MountReading {
        premise_flagged_after_the_mount: flagged_after_the_mount,
        nodes_of_the_spike: spike_nodes.len(),
        carried: carried.len(),
        share,
        verdict,
        shares_by_step,
        located_step,
        one_step_verdict,
    }
}

/// 重挂载自己的发布按次序命名：写行一次，暖机 1、2…
pub(super) fn mount_step_name(publish: &PublishRecord) -> String {
    if publish.kind == "write_row" {
        "write_row".to_string()
    } else {
        format!("warm_up_{}", publish.order)
    }
}

/// 步名对应的计划函数（`crates/singlefs-core/src/mount.rs`），行号照 `--crates-root` 现取。
pub(super) fn plan_function_of_the_mount_step(step_name: &str) -> &'static str {
    if step_name == "write_row" {
        "row_publish_plan_on_file_version"
    } else {
        "empty_publish_plan_after_file_version"
    }
}

// ---- 行五：多出来的步数落在哪几行（Q11、Q12、Q13）----

/// 旗上操作的可比操作：同点别的脚本项里同一选择子、同一声明单元数、不在旗上的操作，每个项取头一个，按项名字典序取前两个。
pub(super) fn pick_comparables<'views>(target: &OperationView, candidates: &'views [OperationView]) -> Vec<&'views OperationView> {
    let mut sorted: Vec<&OperationView> = candidates
        .iter()
        .filter(|view| view.point == target.point && view.item != target.item && view.selector == target.selector && view.old_units == target.old_units && view.new_units == target.new_units && !view.flagged)
        .collect();
    sorted.sort_by(|left, right| (&left.item, left.step_index).cmp(&(&right.item, right.step_index)));
    sorted.dedup_by(|later, earlier| later.item == earlier.item);
    sorted.into_iter().take(2).collect()
}

/// 分辨：多出的步数不小于噪声的 10 倍；噪声 0 记「噪声 0」，算分辨够。
pub(super) fn extra_is_resolved(extra: u64, noise: u64) -> bool {
    noise == 0 || extra >= NOISE_MULTIPLE * noise
}

/// 逐行差 `d_r`（旗上操作的行值减可比操作的行值）：前三大行的和占 `extra` 的比例，与那三行。
pub(super) fn top_rows_share(differences: &[(String, i64)], extra: i64) -> (f64, Vec<(String, i64)>) {
    let mut sorted = differences.to_vec();
    sorted.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let top: Vec<(String, i64)> = sorted.into_iter().take(TOP_ROW_COUNT).filter(|row| row.1 > 0).collect();
    let share = if extra > 0 { top.iter().map(|row| row.1).sum::<i64>() as f64 / extra as f64 } else { 0.0 };
    (share, top)
}

/// 一个旗上操作的行五判定：分辨不够不进判；前三大行占比不到线记分散。
pub(super) fn classify_extra(extra: i64, noise: u64, top_share: f64) -> &'static str {
    if extra <= 0 {
        "no_extra_steps"
    } else if !extra_is_resolved(u64::try_from(extra).expect("正数"), noise) {
        "not_resolved"
    } else if top_share >= TOP_ROWS_SHARE_AT_LEAST {
        "located_in_the_top_rows"
    } else {
        "spread_over_many_rows"
    }
}

/// 每轮的差占全部轮的差之和的比例：旗上操作与可比操作的固定点按轮对齐（缺的轮记 0），每轮带它的并入出口。
/// 轮内步数是 `round_row` 的原始计数（含整份分配器复制），与 `extra` 的 B8 模型步数不是同一口径，所以分母是各轮差的和，不是 `extra`；和不为正记 0。
pub(super) fn round_shares(flagged: &[(String, u64)], comparable: &[(String, u64)]) -> Vec<(usize, String, f64)> {
    let rounds = flagged.len().max(comparable.len());
    let differences: Vec<i64> = (0..rounds)
        .map(|position| i64_from_u64(flagged.get(position).map_or(0, |round| round.1)) - i64_from_u64(comparable.get(position).map_or(0, |round| round.1)))
        .collect();
    let total: i64 = differences.iter().sum();
    (0..rounds)
        .map(|position| {
            let exit = flagged.get(position).map_or_else(|| "none".to_string(), |round| round.0.clone());
            (position + 1, exit, if total > 0 { differences[position] as f64 / total as f64 } else { 0.0 })
        })
        .collect()
}

/// B8 每一行在 `crates/` 里的步：(行名, 文件（相对 `crates/singlefs-core/src/`）, 函数名)。函数名空是「位置待定」。
pub(super) const ROW_FUNCTIONS: [(&str, &str, &str); 28] = [
    ("whole_allocator_copy_before_the_judgement", "transaction.rs", "publish_version_of_trees_relocating_units_after_a_check"),
    ("inode_record_linear_find", "transaction.rs", "inode_record_of"),
    ("files_other_than_the_first_plan", "transaction.rs", "plan_the_files_other_than_the_first_after_this_publish"),
    ("file_other_than_the_first_linear_find", "transaction.rs", "file_other_than_the_first"),
    ("inodes_with_an_extent_entry_set", "transaction.rs", "inodes_with_an_extent_upper_leaf_entry"),
    ("extent_upper_segment_plan", "extent_tree.rs", "plan_the_extent_upper_segment"),
    ("extent_roles_rewritten_scan", "transaction.rs", "extent_tree_roles_rewritten"),
    ("root_header_linear_find", "transaction.rs", "unit"),
    ("checkpoint_cost_accounting_tree_plan", "admission.rs", "accounting_tree_nodes_after_the_publish_built_on"),
    ("cluster_segment_sum", "allocator.rs", "unblocked_slot_pairs_outside_the_cluster_segments"),
    ("allocation_record_tree_node_set", "transaction.rs", "settle_the_allocation_record_tree"),
    ("allocation_record_tree_plan", "allocation_record_tree.rs", "plan_the_tree_after_this_publish"),
    ("inode_number_watermark_steps", "transaction.rs", "inode_number_watermark_steps"),
    ("deadlist_changes_indexes", "transaction.rs", "deadlist_changes_of_this_publish"),
    ("multi_level_tree_planning", "transaction.rs", "plan_the_multi_level_trees"),
    ("mapped_units_of_container_trees", "transaction.rs", "mapped_units_of_a_container_tree"),
    ("moved_entries_path_tail_filter", "transaction.rs", "plan_the_multi_level_trees"),
    ("release_judgement_version_indexes", "transaction.rs", "placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots"),
    ("whole_allocator_copy_in_a_settling_round", "transaction.rs", "settle_the_allocation_record_tree"),
    ("allocator_release_linear_find", "allocator.rs", "release_leaving_the_record_allocated_on"),
    ("user_data_slot_linear_scan", "allocator.rs", "lowest_unblocked_slot_pair_in_the_segment"),
    ("record_intersection_filter", "allocator.rs", "make_room_for_record_on_device"),
    ("empty_segment_linear_scan", "allocator.rs", "lowest_empty_segment_outside_the_region"),
    ("fallback_slot_linear_scan", "allocator.rs", "lowest_commit_generated_fallback_slot_outside_the_region"),
    ("records_of_each_leaf", "allocation_record_tree.rs", "records_of_each_leaf"),
    ("accounting_entries_of_this_publish", "transaction.rs", "accounting_entries_of_this_publish"),
    ("instance_table_chain", "transaction.rs", "instance_table_chain_to_release"),
    ("mapped_units_whole_table", "transaction.rs", ""),
];

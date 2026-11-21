//! Q1 的家族：H1 家族 + Φ1 故障注入 + 候选 (a)(b)(c) 判定的整趟、汇总与阳性对照。

use super::*;

/// 一个差集落点 (设备, 槽) 的聚合上下文——(最小分配记录代, 最大分配记录代,
/// 最小被抛弃根 txg, 最大被抛弃根 txg, 是否在任何一条可读根自己的中央映射树里也出现过)。
pub(super) type SubsetDiffPairContext = (u64, u64, u64, u64, bool);

#[derive(Default)]
pub(super) struct LedgerFaultFamilySummary {
    node_count: u64,
    pub(super) rejected_rollback_attempts: u64,
    unresolved_units: u64,
    empty_baseline_pairs: u64,
    empty_baseline_trigger_count: u64,
    pub(super) pairs: u64,
    trigger_count: u64,
    error_members: BTreeMap<String, u64>,
    by_aspect_severity: BTreeMap<(&'static str, &'static str), (u64, u64)>,
    allocation_record_tree_recompute_outcomes: BTreeMap<&'static str, u64>,
    allocation_record_tree_released_zero: u64,
    allocation_record_tree_released_nonzero: u64,
    tree_table_has_copy_elsewhere_count: u64,
    tree_table_no_copy_elsewhere_count: u64,
    /// 候选 (b) 树表指称「crates 有路」子分支：键 = (verify_named_units, 结局标签)。
    tree_table_crates_has_a_path_outcomes: BTreeMap<(bool, &'static str), u64>,
    /// 候选 (b) 分配记录树指称「从内容树反推占用集合」（中央映射树）：outcome 标签 → 计数。
    allocation_record_tree_from_content_trees_outcomes: BTreeMap<&'static str, u64>,
    /// 同上，「走全」版本（中央映射树 + 树表 / INODE / EXTENT / ACCOUNTING 四类结构节点）。
    allocation_record_tree_from_all_structural_trees_outcomes: BTreeMap<&'static str, u64>,
    /// `subset_of_pristine` 那些格里 `pristine − 走全` 差集的每个 (设备, 槽) 按结构归类，
    /// 键 = (设备, 槽, 归类标签)，值 = 这个三元组在整个 H1 族里出现的次数（同一物理落点可能在多个
    /// (历史节点, 被抛弃根) 组合上重复出现，去重看键、频次看值）。
    subset_diff_pairs: BTreeMap<(u32, u64, &'static str), u64>,
    /// （根因排查②）：同一个 (设备, 槽) 差集对，跨全部触发它的 (历史节点, 被抛弃根) 聚合出的
    /// 上下文——(最小分配记录代, 最大分配记录代, 最小被抛弃根 txg, 最大被抛弃根 txg,
    /// 是否在任何一条可读根自己的中央映射树里也出现过)。最后一项为真 ⇒ 这个落点做过某条根的当前映射，
    /// 佐证「超期未释放的历史内容单元」这个机制,而不是「结构树种类漏走」。
    subset_diff_pair_context: BTreeMap<(u32, u64), SubsetDiffPairContext>,
    persistence_cost_values: BTreeSet<u64>,
    rejected_zero_device_bytes: u64,
    rejected_nonzero_device_bytes: u64,
    faulted_operation_kinds_tried: BTreeSet<String>,
    /// （岔路单第 1 行 op1 第三种变体，跑前登记 Q1-1c 原文「抬 F 那一处单独一行」）：
    /// 不并进 `pairs`/`trigger_count`/`by_aspect_severity`/`error_members`——候选 (a) 的 A1 副本
    /// 按 5.2 原文不改 `raise_rollback_floor`，混进同一组计数会让 Q1-1a/b 的既有读数（session
    /// 已经坐实）掺进一个定义上不同的入口点。
    raise_floor_pairs: u64,
    raise_floor_trigger_count: u64,
    raise_floor_error_members: BTreeMap<String, u64>,
    raise_floor_by_aspect_severity: BTreeMap<(&'static str, &'static str), (u64, u64)>,
    /// 这个历史节点在这一步没有 [F+1, 上限] 的余量可抬（F 已到上限），或探测步骤本身失败——不是
    /// 「该测到却漏了」，是这条历史在这一格没有 op1 第三种变体可跑（`raise_floor_targets_for` 的
    /// 文档注）。
    raise_floor_history_nodes_without_room: u64,
}

/// 跑前登记 5.1 Φ1 + 六、报哪些量：H1 全族 × 每条被抛弃可读根 × 2 个单元 × 3 个严重度 × op1 三种
/// （`mount_writable`、`mount_rollback`、含「先不注入地 `mount_writable` 再
/// `raise_rollback_floor`」，只做「瞬时」；「持续」只在 `mount_writable_trajectory`
/// 里单独测过 PC1-a 选中的那一个落点，这个主循环本身不做持续故障）。`is_a1_arm` 只影响 Q1-4/Q1-1c 的
/// 报告措辞（候选 (a) 的错误成员字符串只在编译进 A1 副本时才会真的出现，装置源码两边共用一份，
/// 靠字符串匹配、不靠枚举名，跨臂都能编译）。
pub(super) fn run_ledger_fault_family(geometry: &Geometry) -> LedgerFaultFamilySummary {
    let parameters = parameters_for(geometry);
    let fixed_geometry = fixed_geometry_for(geometry);
    let family = ledger_fault_history_family(geometry);
    let mut summary = LedgerFaultFamilySummary {
        node_count: u64::try_from(family.nodes.len()).expect("节点数装得进 u64"),
        rejected_rollback_attempts: family.rejected_rollback_attempts,
        ..Default::default()
    };

    for history_node in &family.nodes {
        let node = &history_node.node;
        let Ok(pool_geometry) = independent_geometry(&node.pool) else {
            continue;
        };
        let readable = readable_roots_independent(&node.pool, &pool_geometry);
        let Some(event) = node.rollback_events.first() else {
            continue;
        };
        let abandoned_readable: Vec<TimelineRoot> = event
            .abandoned
            .iter()
            .copied()
            .filter(|root| readable.contains(root))
            .collect();
        let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
        // （岔路单第 1 行 op1 第三种变体）：这个历史节点上「先不注入地 mount_writable 再
        // raise_rollback_floor」能测的每一个目标 floor 值，算一次即可，全部 (被抛弃根, 指称, 严重度)
        // 组合共用（探测步骤本身不看被抛弃根、不看故障目标，与它们互不相关）。
        let raise_floor_targets = raise_floor_targets_for(node, &parameters);
        if raise_floor_targets.is_empty() {
            summary.raise_floor_history_nodes_without_room += 1;
        }

        // Φ1「外加空集」：每个节点一次，不注入任何故障时 op1 = mount_writable 是不是自然触发
        // （物理绕环等非注入原因）。故障数固定为 `FaultSeverity::Empty.fault_count()` = 0。
        debug_assert_eq!(FaultSeverity::Empty.fault_count(), 0);
        let empty_attempt = attempt_faulted_operation(
            node,
            &parameters,
            fixed_geometry,
            &FaultedOperationKind::MountWritable,
            &[],
        );
        summary.empty_baseline_pairs += 1;
        if let Some(count) = empty_attempt.abandoned_roots_unreadable {
            if count > 0 {
                summary.empty_baseline_trigger_count += 1;
            }
        }

        for &abandoned_root in &abandoned_readable {
            let Some(record) = root_record_of(&node.pool, &pool_geometry, abandoned_root) else {
                continue;
            };
            let tree_table_locations = record.tree_table.locations;
            let allocation_locations = locate_allocation_record_tree(&plain_devices, &record);

            for aspect in [LedgerAspect::TreeTable, LedgerAspect::AllocationRecordTree] {
                let locations = match aspect {
                    LedgerAspect::TreeTable => Some(tree_table_locations),
                    LedgerAspect::AllocationRecordTree => {
                        allocation_locations.as_ref().ok().copied()
                    }
                };
                let Some(locations) = locations else {
                    summary.unresolved_units += 1;
                    continue;
                };
                summary.persistence_cost_values.insert(16384u64 * 2);

                for severity in [
                    FaultSeverity::Both,
                    FaultSeverity::Disk0,
                    FaultSeverity::Disk1,
                ] {
                    let fault_targets = fault_targets_for(&locations, severity);
                    let mut faulted_operation_kinds = vec![FaultedOperationKind::MountWritable];
                    for &target in &readable {
                        faulted_operation_kinds.push(FaultedOperationKind::MountRollbackTo(target));
                    }
                    for kind in &faulted_operation_kinds {
                        summary.faulted_operation_kinds_tried.insert(kind.label());
                        let attempt = attempt_faulted_operation(
                            node,
                            &parameters,
                            fixed_geometry,
                            kind,
                            &fault_targets,
                        );
                        summary.pairs += 1;
                        let key = (aspect.label(), severity.label());
                        let entry = summary.by_aspect_severity.entry(key).or_insert((0, 0));
                        entry.1 += 1;
                        if let Some(count) = attempt.abandoned_roots_unreadable {
                            if count > 0 {
                                summary.trigger_count += 1;
                                entry.0 += 1;
                            }
                        }
                        if let Some(member) = &attempt.error_member {
                            *summary.error_members.entry(member.clone()).or_insert(0) += 1;
                            if member == "AbandonedRootLedgerUnreadable" {
                                let total_bytes: u64 = attempt.device_write_bytes.values().sum();
                                if total_bytes == 0 {
                                    summary.rejected_zero_device_bytes += 1;
                                } else {
                                    summary.rejected_nonzero_device_bytes += 1;
                                }
                            }
                        }

                        // Q1-2：只在分配记录树指称、op1 = mount_writable 时做一次独立走树，避免同一个
                        // (单元, 严重度) 因为多个 mount_rollback 目标被反复走很多遍。
                        if aspect == LedgerAspect::AllocationRecordTree
                            && matches!(kind, FaultedOperationKind::MountWritable)
                        {
                            let pristine = allocation_records_under_root(&plain_devices, &record)
                                .map(|records| {
                                    let released = records
                                        .iter()
                                        .filter(|allocation_record| allocation_record.is_released)
                                        .count();
                                    let unreleased: BTreeSet<(u32, u64)> = records
                                        .iter()
                                        .filter(|allocation_record| !allocation_record.is_released)
                                        .map(|allocation_record| {
                                            (allocation_record.device.0, allocation_record.slot.0)
                                        })
                                        .collect();
                                    (unreleased, released)
                                });
                            match (
                                pristine,
                                walk_allocation_records_with_fault(
                                    node,
                                    fixed_geometry,
                                    &fault_targets,
                                    &record,
                                ),
                            ) {
                                (Ok((pristine_set, released)), Ok(faulted_records)) => {
                                    let faulted_set: BTreeSet<(u32, u64)> = faulted_records
                                        .iter()
                                        .filter(|(_, _, is_released)| !is_released)
                                        .map(|(device, slot, _)| (*device, *slot))
                                        .collect();
                                    let outcome = if faulted_set == pristine_set {
                                        "can_recompute"
                                    } else {
                                        "unequal"
                                    };
                                    *summary
                                        .allocation_record_tree_recompute_outcomes
                                        .entry(outcome)
                                        .or_insert(0) += 1;
                                    if released == 0 {
                                        summary.allocation_record_tree_released_zero += 1;
                                    } else {
                                        summary.allocation_record_tree_released_nonzero += 1;
                                    }
                                }
                                (Ok(_), Err(_)) => {
                                    *summary
                                        .allocation_record_tree_recompute_outcomes
                                        .entry("cannot_recompute")
                                        .or_insert(0) += 1;
                                }
                                (Err(_), _) => {
                                    // 孪生镜像自己都读不出（不该发生在未注入的镜像上），单独计一类。
                                    *summary
                                        .allocation_record_tree_recompute_outcomes
                                        .entry("pristine_unreadable")
                                        .or_insert(0) += 1;
                                }
                            }

                            // 候选 (b) 分配记录树指称「从内容树反推占用集合」（修订项 5 提的操作化）：
                            // 不重读这一格被注入的分配记录树节点本身，改走中央映射树（不受这一格故障影响，
                            // Φ1 只挡树表或分配记录树那一个单元）。孪生镜像上各自独立算出两个集合再比对，
                            // fault_targets 完全不参与这一支（这正是它不重读目标节点的证据）。
                            let pristine_unreleased = allocation_records_under_root(
                                &plain_devices,
                                &record,
                            )
                            .map(|records| {
                                records
                                    .iter()
                                    .filter(|allocation_record| !allocation_record.is_released)
                                    .map(|allocation_record| {
                                        (allocation_record.device.0, allocation_record.slot.0)
                                    })
                                    .collect::<BTreeSet<(u32, u64)>>()
                            });
                            let from_content_trees =
                                allocation_record_tree_reachable_placements_via_central_mapping(
                                    &plain_devices,
                                    &record,
                                );
                            let outcome = match (pristine_unreleased, from_content_trees) {
                                (Ok(pristine_set), Ok(mapped_set)) => {
                                    if mapped_set == pristine_set {
                                        "matches_pristine"
                                    } else {
                                        "unequal"
                                    }
                                }
                                (Ok(_), Err(_)) => "central_mapping_unreadable",
                                (Err(_), _) => "pristine_unreadable",
                            };
                            *summary
                                .allocation_record_tree_from_content_trees_outcomes
                                .entry(outcome)
                                .or_insert(0) += 1;

                            // 同上，「走全」版本：与上面完全并行的第二次独立比对，同一个
                            // (pristine_unreleased 再算一次, all_structural_trees) 对，gating 条件
                            // 逐字相同，好让两个 outcome 表的分母可以直接对齐比较。
                            let pristine_records_again =
                                allocation_records_under_root(&plain_devices, &record);
                            let pristine_unreleased_again =
                                pristine_records_again.as_ref().map(|records| {
                                    records
                                        .iter()
                                        .filter(|allocation_record| !allocation_record.is_released)
                                        .map(|allocation_record| {
                                            (allocation_record.device.0, allocation_record.slot.0)
                                        })
                                        .collect::<BTreeSet<(u32, u64)>>()
                                });
                            let from_all_structural_trees =
                                allocation_record_tree_reachable_placements_via_all_structural_trees(
                                    &plain_devices,
                                    &record,
                                );
                            let all_structural_outcome =
                                match (&pristine_unreleased_again, &from_all_structural_trees) {
                                    (Ok(pristine_set), Ok(mapped_set)) => {
                                        if mapped_set == pristine_set {
                                            "matches_pristine"
                                        } else if mapped_set.is_superset(pristine_set) {
                                            "superset_of_pristine"
                                        } else if mapped_set.is_subset(pristine_set) {
                                            "subset_of_pristine"
                                        } else {
                                            "unequal_neither_subset_nor_superset"
                                        }
                                    }
                                    (Ok(_), Err(_)) => "structural_trees_unreadable",
                                    (Err(_), _) => "pristine_unreadable",
                                };
                            *summary
                                .allocation_record_tree_from_all_structural_trees_outcomes
                                .entry(all_structural_outcome)
                                .or_insert(0) += 1;

                            // 171 格里的差集逐项归类（见 `classify_diff_pair` 文档注）。
                            if all_structural_outcome == "subset_of_pristine" {
                                if let (Ok(pristine_set), Ok(mapped_set), Ok(raw_records)) = (
                                    &pristine_unreleased_again,
                                    &from_all_structural_trees,
                                    &pristine_records_again,
                                ) {
                                    for pair in pristine_set.difference(mapped_set) {
                                        let label = classify_diff_pair(
                                            &plain_devices,
                                            &record,
                                            allocation_locations.as_ref().ok().copied(),
                                            raw_records,
                                            *pair,
                                        );
                                        *summary
                                            .subset_diff_pairs
                                            .entry((pair.0, pair.1, label))
                                            .or_insert(0) += 1;

                                        // 根因排查②：这个落点的分配记录代、这个被抛弃根自己的 txg，
                                        // 以及它是否在任何一条可读根**自己的**中央映射树里也出现过
                                        // （出现过 ⇒ 它曾是某条根的当前内容，现在只是被更晚的版本
                                        // 换下、还没被释放，不是遗漏了某种结构树）。
                                        let generation = raw_records
                                            .iter()
                                            .find(|candidate| {
                                                (candidate.device.0, candidate.slot.0) == *pair
                                            })
                                            .map(|candidate| candidate.generation.0)
                                            .unwrap_or(u64::MAX);
                                        let referenced_elsewhere = readable.iter().any(
                                            |&candidate_root| {
                                                root_record_of(
                                                    &node.pool,
                                                    &pool_geometry,
                                                    candidate_root,
                                                )
                                                .and_then(|candidate_record| {
                                                    allocation_record_tree_reachable_placements_via_central_mapping(
                                                        &plain_devices,
                                                        &candidate_record,
                                                    )
                                                    .ok()
                                                })
                                                .is_some_and(|mapped| mapped.contains(pair))
                                            },
                                        );
                                        let context = summary
                                            .subset_diff_pair_context
                                            .entry((pair.0, pair.1))
                                            .or_insert((
                                                generation,
                                                generation,
                                                record.checkpoint_txg.0,
                                                record.checkpoint_txg.0,
                                                referenced_elsewhere,
                                            ));
                                        context.0 = context.0.min(generation);
                                        context.1 = context.1.max(generation);
                                        context.2 = context.2.min(record.checkpoint_txg.0);
                                        context.3 = context.3.max(record.checkpoint_txg.0);
                                        context.4 |= referenced_elsewhere;
                                    }
                                }
                            }
                        }

                        // 候选 (b) 树表指称「crates 有路」子分支（跑前登记 5.2 ②）：只在树表指称、
                        // op1 = mount_writable 时做一次，两种 verify_named_units 都跑。
                        if aspect == LedgerAspect::TreeTable
                            && matches!(kind, FaultedOperationKind::MountWritable)
                        {
                            for verify_named_units in [true, false] {
                                let outcome = tree_table_crates_has_a_path(
                                    node,
                                    fixed_geometry,
                                    &fault_targets,
                                    &pool_geometry,
                                    abandoned_root,
                                    &record,
                                    verify_named_units,
                                );
                                *summary
                                    .tree_table_crates_has_a_path_outcomes
                                    .entry((verify_named_units, outcome.label()))
                                    .or_insert(0) += 1;
                            }
                        }
                    }

                    // （岔路单第 1 行 op1 第三种变体，「抬 F 那一处单独一行」——跑前登记
                    // Q1-1c 原文）：这一支不并进上面的 `faulted_operation_kinds`/`summary.pairs`——
                    // 候选 (a) 的 A1 副本按 5.2 原文不改 `raise_rollback_floor`，混进同一组计数会让
                    // Q1-1a/b 的既有读数（已经坐实）掺进一个定义上不同的入口点。
                    for &target_floor in &raise_floor_targets {
                        let kind =
                            FaultedOperationKind::MountWritableThenRaiseFloorTo(target_floor);
                        summary.faulted_operation_kinds_tried.insert(kind.label());
                        let attempt = attempt_faulted_operation(
                            node,
                            &parameters,
                            fixed_geometry,
                            &kind,
                            &fault_targets,
                        );
                        summary.raise_floor_pairs += 1;
                        let key = (aspect.label(), severity.label());
                        let entry = summary
                            .raise_floor_by_aspect_severity
                            .entry(key)
                            .or_insert((0, 0));
                        entry.1 += 1;
                        if let Some(count) = attempt.abandoned_roots_unreadable {
                            if count > 0 {
                                summary.raise_floor_trigger_count += 1;
                                entry.0 += 1;
                            }
                        }
                        if let Some(member) = &attempt.error_member {
                            *summary
                                .raise_floor_error_members
                                .entry(member.clone())
                                .or_insert(0) += 1;
                        }
                    }
                }
            }

            // Q1-2 树表指称候选 (b)「有副本」：只需要判一次（与严重度、op1 无关，只看孪生镜像本身）。
            if let Some(tree_table_bytes) = read_node_bytes(&plain_devices, &tree_table_locations) {
                if tree_table_has_copy_elsewhere(
                    &plain_devices,
                    &node.pool,
                    &pool_geometry,
                    abandoned_root,
                    &tree_table_bytes,
                ) {
                    summary.tree_table_has_copy_elsewhere_count += 1;
                } else {
                    summary.tree_table_no_copy_elsewhere_count += 1;
                }
            }
        }
    }
    summary
}

pub(super) fn emit_ledger_fault_family_summary(
    geometry: &Geometry,
    summary: &LedgerFaultFamilySummary,
) {
    emit_result(&format!(
        "name=q1_h1_family_size geometry={} nodes={} rejected_rollback_attempts={} unresolved_units={}",
        geometry.label, summary.node_count, summary.rejected_rollback_attempts, summary.unresolved_units
    ));
    emit_result(&format!(
        "name=q1_1a_empty_baseline geometry={} pairs={} trigger_count={} fault_count={}",
        geometry.label,
        summary.empty_baseline_pairs,
        summary.empty_baseline_trigger_count,
        FaultSeverity::Empty.fault_count()
    ));
    emit_result(&format!(
        "name=q1_1a_trigger_summary geometry={} pairs={} trigger_count={} op1_kinds_tried={}",
        geometry.label,
        summary.pairs,
        summary.trigger_count,
        summary.faulted_operation_kinds_tried.len()
    ));
    for ((aspect, severity), (trigger, pairs)) in &summary.by_aspect_severity {
        emit_result(&format!(
            "name=q1_1a_by_aspect_severity geometry={} aspect={aspect} severity={severity} pairs={pairs} trigger={trigger}"
        , geometry.label));
    }
    for (member, count) in &summary.error_members {
        emit_result(&format!(
            "name=q1_1c_error_member geometry={} member={member} count={count}",
            geometry.label
        ));
    }
    // （岔路单第 1 行 op1 第三种变体，跑前登记 Q1-1c 原文「抬 F 那一处单独一行」）：
    // 单独一组名字（`raise_floor`），不与上面 mount_writable/mount_rollback 那两种 op1 的
    // q1_1a_trigger_summary/q1_1a_by_aspect_severity/q1_1c_error_member 合并。
    emit_result(&format!(
        "name=q1_1a_raise_floor_trigger_summary geometry={} pairs={} trigger_count={} history_nodes_without_room={}",
        geometry.label,
        summary.raise_floor_pairs,
        summary.raise_floor_trigger_count,
        summary.raise_floor_history_nodes_without_room
    ));
    for ((aspect, severity), (trigger, pairs)) in &summary.raise_floor_by_aspect_severity {
        emit_result(&format!(
            "name=q1_1a_raise_floor_by_aspect_severity geometry={} aspect={aspect} severity={severity} pairs={pairs} trigger={trigger}"
        , geometry.label));
    }
    for (member, count) in &summary.raise_floor_error_members {
        emit_result(&format!(
            "name=q1_1c_raise_floor_error_member geometry={} member={member} count={count}",
            geometry.label
        ));
    }
    for (outcome, count) in &summary.allocation_record_tree_recompute_outcomes {
        emit_result(&format!(
            "name=q1_2_allocation_record_tree geometry={} outcome={outcome} count={count}",
            geometry.label
        ));
    }
    emit_result(&format!(
        "name=q1_2_allocation_released_records geometry={} zero={} nonzero={}",
        geometry.label,
        summary.allocation_record_tree_released_zero,
        summary.allocation_record_tree_released_nonzero
    ));
    emit_result(&format!(
        "name=q1_2_tree_table_copy_elsewhere geometry={} has_copy={} no_copy={}",
        geometry.label,
        summary.tree_table_has_copy_elsewhere_count,
        summary.tree_table_no_copy_elsewhere_count
    ));
    for ((verify_named_units, outcome), count) in &summary.tree_table_crates_has_a_path_outcomes {
        emit_result(&format!(
            "name=q1_2_tree_table_crates_has_a_path geometry={} verify_named_units={verify_named_units} outcome={outcome} count={count}",
            geometry.label
        ));
    }
    for (outcome, count) in &summary.allocation_record_tree_from_content_trees_outcomes {
        emit_result(&format!(
            "name=q1_2_allocation_record_tree_from_content_trees geometry={} outcome={outcome} count={count}",
            geometry.label
        ));
    }
    for (outcome, count) in &summary.allocation_record_tree_from_all_structural_trees_outcomes {
        emit_result(&format!(
            "name=q1_2_allocation_record_tree_from_all_structural_trees geometry={} outcome={outcome} count={count}",
            geometry.label
        ));
    }
    emit_result(&format!(
        "name=q1_3_persistence_cost_values geometry={} values={:?}",
        geometry.label, summary.persistence_cost_values
    ));
    emit_result(&format!(
        "name=q1_4_a_rejected_device_bytes geometry={} zero_byte_rejections={} nonzero_byte_rejections={}",
        geometry.label,
        summary.rejected_zero_device_bytes,
        summary.rejected_nonzero_device_bytes
    ));
    // 171 格 subset_of_pristine 的差集逐项归类——先按 (设备, 槽, 标签) 去重打一行，
    // 再按标签汇总出现次数，两张表合起来就是「逐个打出来 + 各属于哪种结构」。
    let mut label_totals: BTreeMap<&'static str, u64> = BTreeMap::new();
    for ((device, slot, label), occurrences) in &summary.subset_diff_pairs {
        emit_result(&format!(
            "name=q1_2_subset_diff_pair geometry={} device={device} slot={slot} label={label} occurrences={occurrences}",
            geometry.label
        ));
        *label_totals.entry(label).or_insert(0) += occurrences;
    }
    for (label, total) in &label_totals {
        emit_result(&format!(
            "name=q1_2_subset_diff_label_total geometry={} label={label} occurrences={total}",
            geometry.label
        ));
    }
    emit_result(&format!(
        "name=q1_2_subset_diff_distinct_pairs geometry={} distinct_pairs={}",
        geometry.label,
        summary.subset_diff_pairs.len()
    ));
    for (
        (device, slot),
        (
            minimum_generation,
            maximum_generation,
            minimum_abandoned_root_txg,
            maximum_abandoned_root_txg,
            referenced_elsewhere,
        ),
    ) in &summary.subset_diff_pair_context
    {
        emit_result(&format!(
            "name=q1_2_subset_diff_pair_context geometry={} device={device} slot={slot} minimum_generation={minimum_generation} maximum_generation={maximum_generation} minimum_abandoned_root_txg={minimum_abandoned_root_txg} maximum_abandoned_root_txg={maximum_abandoned_root_txg} referenced_by_some_readable_roots_own_mapping={referenced_elsewhere}",
            geometry.label
        ));
    }
}

/// PC1-a / PC1-b（跑前登记 5.2 阳性对照）：H1 族按固定次序（`overwrites_before_rollback` 升序、
/// `rollback_target` 按 `BTreeSet` 升序、`overwrites_after_rollback` 升序，
/// 再按每个节点的被抛弃可读根升序）第一个满足「分配记录树节点 u 不共享」的历史。两条共用同一个
/// (节点, 被抛弃根)；PC1-a 打分配记录树指称，PC1-b 打树表指称，都是「两份都读失败、瞬时、
/// op1 = mount_writable」。
pub(super) fn run_ledger_fault_positive_controls(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let fixed_geometry = fixed_geometry_for(geometry);
    let family = ledger_fault_history_family(geometry);

    for history_node in &family.nodes {
        let node = &history_node.node;
        let Ok(pool_geometry) = independent_geometry(&node.pool) else {
            continue;
        };
        let readable = readable_roots_independent(&node.pool, &pool_geometry);
        let Some(event) = node.rollback_events.first() else {
            continue;
        };
        let abandoned_readable: Vec<TimelineRoot> = event
            .abandoned
            .iter()
            .copied()
            .filter(|root| readable.contains(root))
            .collect();
        let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);

        for &abandoned_root in &abandoned_readable {
            let Some(record) = root_record_of(&node.pool, &pool_geometry, abandoned_root) else {
                continue;
            };
            let Ok(allocation_locations) = locate_allocation_record_tree(&plain_devices, &record)
            else {
                continue;
            };
            let allocation_keys: BTreeSet<(u32, u64)> =
                allocation_locations.iter().map(location_key).collect();
            if is_unit_shared(
                &plain_devices,
                &node.pool,
                &pool_geometry,
                abandoned_root,
                &allocation_keys,
            ) {
                continue;
            }

            // 找到了：装置自己数「账要经过这个单元的被抛弃根条数」——有几条被抛弃可读根的分配记录树
            // 指针落在同一个 (设备, 槽) 上。
            let mut roots_through_unit = 0u64;
            for &candidate in &abandoned_readable {
                if let Some(candidate_record) =
                    root_record_of(&node.pool, &pool_geometry, candidate)
                {
                    if let Ok(candidate_locations) =
                        locate_allocation_record_tree(&plain_devices, &candidate_record)
                    {
                        let candidate_keys: BTreeSet<(u32, u64)> =
                            candidate_locations.iter().map(location_key).collect();
                        if !candidate_keys.is_disjoint(&allocation_keys) {
                            roots_through_unit += 1;
                        }
                    }
                }
            }

            let allocation_fault_targets =
                fault_targets_for(&allocation_locations, FaultSeverity::Both);
            let allocation_attempt = attempt_faulted_operation(
                node,
                &parameters,
                fixed_geometry,
                &FaultedOperationKind::MountWritable,
                &allocation_fault_targets,
            );
            let allocation_total_bytes: u64 = allocation_attempt.device_write_bytes.values().sum();
            emit_result(&format!(
                "name=pc1_a geometry={} op1={} overwrites_before_rollback={} rollback_target={:?} overwrites_after_rollback={} abandoned_root={:?} roots_through_unit={} outcome_abandoned_roots_unreadable={:?} error_member={:?} error_debug={:?} device_write_bytes={}",
                geometry.label,
                FaultedOperationKind::MountWritable.label(),
                history_node.overwrites_before_rollback,
                history_node.rollback_target,
                history_node.overwrites_after_rollback,
                abandoned_root,
                roots_through_unit,
                allocation_attempt.abandoned_roots_unreadable,
                allocation_attempt.error_member,
                allocation_attempt.error_debug,
                allocation_total_bytes
            ));

            // （岔路单第 1 行还差项④，8.1「持续故障下 op1 及其后两次挂载各一个值；瞬时
            // 故障下 op1 一个值、后两次（不注入）各一个值」）：用 PC1-a 已经选中的这同一个
            // (历史, 被抛弃根, 分配记录树单元, 两份都读失败) 当轨迹的起点——`allocation_attempt`
            // 已经证明这一格会触发，是这条轨迹天然、可复现的落点，不另挑一个未经验证的构造。
            let persistent_trajectory = mount_writable_trajectory(
                &node.pool,
                &parameters,
                fixed_geometry,
                &allocation_fault_targets,
                true,
                3,
            );
            emit_result(&format!(
                "name=q1_1a_op1_trajectory geometry={} mode=persistent trajectory={:?}",
                geometry.label, persistent_trajectory
            ));
            let transient_trajectory = mount_writable_trajectory(
                &node.pool,
                &parameters,
                fixed_geometry,
                &allocation_fault_targets,
                false,
                3,
            );
            emit_result(&format!(
                "name=q1_1a_op1_trajectory geometry={} mode=transient trajectory={:?}",
                geometry.label, transient_trajectory
            ));

            let pristine_allocation =
                allocation_records_under_root(&plain_devices, &record).map(|records| {
                    records
                        .into_iter()
                        .filter(|allocation_record| !allocation_record.is_released)
                        .map(|allocation_record| {
                            (allocation_record.device.0, allocation_record.slot.0)
                        })
                        .collect::<BTreeSet<_>>()
                });
            let faulted_allocation_walk = walk_allocation_records_with_fault(
                node,
                fixed_geometry,
                &allocation_fault_targets,
                &record,
            );
            let allocation_walk_outcome = match (&pristine_allocation, &faulted_allocation_walk) {
                (Ok(pristine_set), Ok(faulted_records)) => {
                    let faulted_set: BTreeSet<(u32, u64)> = faulted_records
                        .iter()
                        .filter(|(_, _, is_released)| !is_released)
                        .map(|(device, slot, _)| (*device, *slot))
                        .collect();
                    if faulted_set == *pristine_set {
                        "can_recompute"
                    } else {
                        "unequal"
                    }
                }
                (Ok(_), Err(_)) => "cannot_recompute",
                (Err(_), _) => "pristine_unreadable",
            };
            emit_result(&format!(
                "name=pc1_a_candidate_b_allocation_record_tree geometry={} outcome={allocation_walk_outcome}",
                geometry.label
            ));

            let tree_table_locations = record.tree_table.locations;
            let tree_table_fault_targets =
                fault_targets_for(&tree_table_locations, FaultSeverity::Both);
            let tree_table_attempt = attempt_faulted_operation(
                node,
                &parameters,
                fixed_geometry,
                &FaultedOperationKind::MountWritable,
                &tree_table_fault_targets,
            );
            let mut roots_through_tree_table = 0u64;
            for &candidate in &abandoned_readable {
                if let Some(candidate_record) =
                    root_record_of(&node.pool, &pool_geometry, candidate)
                {
                    let candidate_keys: BTreeSet<(u32, u64)> = candidate_record
                        .tree_table
                        .locations
                        .iter()
                        .map(location_key)
                        .collect();
                    let target_keys: BTreeSet<(u32, u64)> =
                        tree_table_locations.iter().map(location_key).collect();
                    if !candidate_keys.is_disjoint(&target_keys) {
                        roots_through_tree_table += 1;
                    }
                }
            }
            emit_result(&format!(
                "name=pc1_b geometry={} overwrites_before_rollback={} rollback_target={:?} overwrites_after_rollback={} abandoned_root={:?} roots_through_unit={} outcome_abandoned_roots_unreadable={:?} error_member={:?}",
                geometry.label,
                history_node.overwrites_before_rollback,
                history_node.rollback_target,
                history_node.overwrites_after_rollback,
                abandoned_root,
                roots_through_tree_table,
                tree_table_attempt.abandoned_roots_unreadable,
                tree_table_attempt.error_member
            ));
            let allocation_walk_still_fails_when_tree_table_faulted =
                walk_allocation_records_with_fault(
                    node,
                    fixed_geometry,
                    &tree_table_fault_targets,
                    &record,
                )
                .is_err();
            emit_result(&format!(
                "name=pc1_b_candidate_b_allocation_record_tree geometry={} walk_fails={}",
                geometry.label, allocation_walk_still_fails_when_tree_table_faulted
            ));
            if let Some(tree_table_bytes) = read_node_bytes(&plain_devices, &tree_table_locations) {
                let has_copy = tree_table_has_copy_elsewhere(
                    &plain_devices,
                    &node.pool,
                    &pool_geometry,
                    abandoned_root,
                    &tree_table_bytes,
                );
                emit_result(&format!(
                    "name=pc1_b_candidate_b_tree_table geometry={} has_copy_elsewhere={has_copy}",
                    geometry.label
                ));
            }
            return;
        }
    }
    emit_result(&format!(
        "name=pc1_a_and_b geometry={} verdict=not_found reason=\"H1 全族没有一条『分配记录树节点不共享』的历史\"",
        geometry.label
    ));
}

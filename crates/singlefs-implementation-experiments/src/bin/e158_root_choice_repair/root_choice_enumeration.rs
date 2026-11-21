//! H3 的穷举（σ ∈ Σ3^L）与择根家族、小环取法、Q3-2 与 Q2-3 的算术、PC3，与按几何点各跑一趟。

use super::*;

// ============================================================================
// 六、H3 的穷举：σ ∈ Σ3^L，Σ3 = {W, M, R(j)}，j 取那一刻每一条可读根；不抽样、全枚举。
//    「省法」（5.1「H3 的省法」）不在这一段实现：这一段只跑「不注入」，本来就不做 (b)(c) 那两次带故障的挂载，
//    省法省的正是那两次——不注入时没有省的必要，这里如实全枚举、不引省法。
// ============================================================================

#[derive(Default)]
pub(super) struct EnumerationSummary {
    pub(super) histories: u64,
    pub(super) peak_events_live: u64,
    peak_roots_live: u64,
    steps_with_positive_events_live: u64,
    /// H3 按固定次序第一个满足「events_live ≥ 1」的节点（PC3 用）。
    pub(super) first_events_live_node: Option<SimNode>,
    pub(super) errors: Vec<String>,
    /// 8.1「期末值」：只在叶节点（σ 跑满 L 步，或没有更长的合法后继）记一次，
    /// 与 `peak_events_live`（跨全部步、含中间步）分开报。
    leaf_count: u64,
    leaves_with_positive_events_live: u64,
    peak_events_live_at_a_leaf: u64,
}

#[allow(
    clippy::too_many_arguments,
    reason = "H3 穷举需要节点、剩余深度、mkfs 参数、几何、汇总器与 trace 回调六项，各自有名字有类型，拆结构体只是把参数挪个地方"
)]
pub(super) fn enumerate(
    node: &SimNode,
    depth_remaining: u64,
    parameters: &MakeFilesystemParameters,
    geometry: &PoolGeometry,
    summary: &mut EnumerationSummary,
    trace: &mut dyn FnMut(&SimNode, u64, u64),
) {
    summary.histories += 1;
    let readable = readable_roots_independent(&node.pool, geometry);
    let (events_live, roots_live) = live_rollback_events(node, &readable);
    trace(node, events_live, roots_live);
    summary.peak_events_live = summary.peak_events_live.max(events_live);
    summary.peak_roots_live = summary.peak_roots_live.max(roots_live);
    if events_live > 0 {
        summary.steps_with_positive_events_live += 1;
        if summary.first_events_live_node.is_none() {
            summary.first_events_live_node = Some(node.clone());
        }
    }
    if depth_remaining == 0 {
        summary.leaf_count += 1;
        if events_live > 0 {
            summary.leaves_with_positive_events_live += 1;
        }
        summary.peak_events_live_at_a_leaf = summary.peak_events_live_at_a_leaf.max(events_live);
        return;
    }
    // W：只在挂着时合法。
    if node.session.is_some() {
        match apply_overwrite(node, parameters) {
            Ok(next) => enumerate(
                &next,
                depth_remaining - 1,
                parameters,
                geometry,
                summary,
                trace,
            ),
            Err(error) => summary
                .errors
                .push(format!("W 前提不满足（记为前提不满足，不算错）：{error}")),
        }
    }
    // M：关闭并 mount_writable，任何时候都合法。
    match apply_mount_writable(node, parameters) {
        Ok(next) => enumerate(
            &next,
            depth_remaining - 1,
            parameters,
            geometry,
            summary,
            trace,
        ),
        Err(error) => summary
            .errors
            .push(format!("M 前提不满足（记为前提不满足，不算错）：{error}")),
    }
    // R(j)：关闭并 mount_rollback 到那一刻每一条可读根。
    for root in &readable {
        let target = RollbackTarget {
            instance: InstanceGeneration(root.0),
            checkpoint_txg: CheckpointTxg(root.1),
        };
        match apply_mount_rollback(node, target, parameters) {
            Ok(next) => enumerate(
                &next,
                depth_remaining - 1,
                parameters,
                geometry,
                summary,
                trace,
            ),
            Err(error) => {
                // 候选排除（不在候选集、低于 F、被抛弃时间线）按结局记，不算装置的错，只是这个分支到此为止。
                summary.errors.push(format!(
                    "R({root:?}) 前提不满足（记为前提不满足，不算错）：{error}"
                ));
            }
        }
    }
}

/// H3 一族：给定几何、overwrites_before_sigma 的取值集合、σ 的最大长度 L，逐一枚举、汇总。
fn run_root_choice_family(
    geometry: &Geometry,
    initial_overwrite_counts: &[u64],
    sigma_length_limit: u64,
    trace_label: &str,
    verbose_trace: bool,
) -> EnumerationSummary {
    let parameters = parameters_for(geometry);
    let mut summary = EnumerationSummary::default();
    for &overwrites_before_sigma in initial_overwrite_counts {
        let root_node = match bootstrap(geometry, overwrites_before_sigma) {
            Ok(node) => node,
            Err(error) => {
                summary.errors.push(format!(
                    "bootstrap(overwrites_before_sigma={overwrites_before_sigma}) 失败：{error}"
                ));
                continue;
            }
        };
        let independent_geometry_view = match independent_geometry(&root_node.pool) {
            Ok(geometry_view) => geometry_view,
            Err(error) => {
                summary.errors.push(error);
                continue;
            }
        };
        let mut steps_visited_for_this_overwrite_count: u64 = 0;
        let mut trace = |node: &SimNode, events_live: u64, roots_live: u64| {
            steps_visited_for_this_overwrite_count += 1;
            if verbose_trace {
                println!(
                    "TRACE geometry={} overwrites_before_sigma={} step={} path={:?} events_live={} roots_live={}",
                    trace_label,
                    overwrites_before_sigma,
                    steps_visited_for_this_overwrite_count,
                    node.path,
                    events_live,
                    roots_live
                );
            }
        };
        enumerate(
            &root_node,
            sigma_length_limit,
            &parameters,
            &independent_geometry_view,
            &mut summary,
            &mut trace,
        );
        println!(
            "N1_DONE geometry={trace_label} overwrites_before_sigma={overwrites_before_sigma} histories_so_far={} peak_events_live={} peak_roots_live={}",
            summary.histories, summary.peak_events_live, summary.peak_roots_live
        );
    }
    summary
}

// ============================================================================
// 七、小环取法（8.2）：从 48 KiB 起、每次加 16 KiB，取第一个 mkfs 收、且「H2 最长模板」整段跑得通的环长。
//    H2 不在这一段跑；这里按 8.2 原文的校准点（overwrites_before_sigma=6、mount_writable、n2=2）复刻一个不依赖 H2 自身实现的代理探针：
//    只判「这几步任何一步是否报错」，不判是哪一类错——H2 完整实现（P331 判定、崩溃档）留给第三段。
//    这个简化在报告里点名，交主 agent 核这个代理够不够格。
// ============================================================================

fn small_ring_probe(ring_bytes: u64) -> bool {
    let geometry = Geometry {
        label: "small-ring-probe",
        slots_per_region: 8,
        journal_ring_bytes: ring_bytes,
    };
    let parameters = parameters_for(&geometry);
    let mut node = match bootstrap(&geometry, 6) {
        Ok(node) => node,
        Err(_) => return false,
    };
    node = match apply_mount_writable(&node, &parameters) {
        Ok(node) => node,
        Err(_) => return false,
    };
    for _ in 0..2 {
        node = match apply_overwrite(&node, &parameters) {
            Ok(node) => node,
            Err(_) => return false,
        };
    }
    true
}

// ============================================================================
// 十、Q3-2、Q2-3 的算术：只用 N_w，不建副本、不跑臂。
// ============================================================================

pub(super) fn report_witness_width_and_slot_overlap_arithmetic(peak_live_rollback_events: u64) {
    let single_16 = 16u64;
    let single_12 = 12u64;
    let table_16 = 1 + 16 * peak_live_rollback_events;
    let table_12 = 1 + 12 * peak_live_rollback_events;
    for (label, width) in [
        ("单条16", single_16),
        ("单条12", single_12),
        ("表1+16*peak", table_16),
        ("表1+12*peak", table_12),
    ] {
        let within_512 = 512i64 - 481 - i64::try_from(width).expect("宽度装得进 i64");
        let within_4096 = 4096i64 - 481 - i64::try_from(width).expect("宽度装得进 i64");
        emit_result(&format!(
            "name=q3_2_witness_width_margin peak_live_rollback_events={peak_live_rollback_events} width_label={label} width_bytes={width} margin_within_512={within_512} margin_within_512_verdict={} margin_within_4096={within_4096} margin_within_4096_verdict={}",
            if within_512 < 0 { "past_512" } else { "fits" },
            if within_4096 < 0 { "past_4096" } else { "fits" }
        ));
    }
    for (label, width) in [
        ("单条16", single_16),
        ("单条12", single_12),
        ("表1+16*peak", table_16),
        ("表1+12*peak", table_12),
    ] {
        let plus8_within_512 = 512i64 - 481 - 8 - i64::try_from(width).expect("宽度装得进 i64");
        emit_result(&format!(
            "name=q2_3_witness_width_with_candidate_two_txg_field peak_live_rollback_events={peak_live_rollback_events} width_label={label} width_bytes={width} margin_within_512_with_plus8={plus8_within_512} verdict={}",
            if plus8_within_512 < 0 { "past_512_together" } else { "fits" }
        ));
    }
}

// ============================================================================
// 十一、PC3：H3 按固定次序第一个满足「events_live ≥ 1」的历史；F = 那一刻「回退实例」全部根所在的根槽
//    （回退实例 = 刚做完的那次 R(j) 建立起来的新实例——按 4 节已有用例「回退实例 3 的四条根……四条都读不出时
//    恢复退到被抛弃的根 (2, 8)」那个形态操作化：让「刚接手的这个新实例」自己的全部根变得读不出，
//    看择根会不会退回它自己抛弃掉的那条旧时间线）。这个操作化不在 5.6 的清单里，交回报告里点名，请主 agent 核对
//    C332 原文是不是同一个意思。
// ============================================================================

pub(super) fn run_pc3(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    geometry_view: &PoolGeometry,
) -> bool {
    let session_instance = match &node.session {
        Some(session) => session.instance.0,
        None => {
            emit_result(&format!(
                "name=pc3 path={:?} verdict=fail reason=\"没有开着的会话，算不出回退实例\"",
                node.path
            ));
            return false;
        }
    };
    let readable = readable_roots_independent(&node.pool, geometry_view);
    let (events_live, _) = live_rollback_events(node, &readable);
    if events_live == 0 {
        emit_result(&format!(
            "name=pc3 path={:?} verdict=fail reason=\"events_live=0，不满足 PC3 的触发条件\"",
            node.path
        ));
        return false;
    }
    let expected_root = node
        .rollback_events
        .iter()
        .flat_map(|event| event.abandoned.iter().copied())
        .filter(|root| readable.contains(root))
        .max_by_key(|root| (root.1, root.0))
        .expect("events_live>0 时读得出的被抛弃根非空");

    let all_roots = readable_roots_with_slots(&node.pool, geometry_view);
    let named = all_roots
        .iter()
        .filter(|(_, _, root)| root.0 == session_instance)
        .fold(NamedRootRingSlots::NONE, |named, (region, slot, _)| {
            named.with(RootRingSlot {
                region: *region,
                slot: *slot,
            })
        });
    let named_count = all_roots
        .iter()
        .filter(|(_, _, root)| root.0 == session_instance)
        .count();
    if named_count == 0 {
        emit_result(&format!(
            "name=pc3 path={:?} verdict=fail reason=\"回退实例 {session_instance} 在根环里一条自证根都没有，造不出故障集合\"",
            node.path
        ));
        return false;
    }

    let target = RootRingSlotTarget {
        named_slots: named,
        region_devices: region_devices(),
        fixed_structure_slot_spacing: 4096,
    };

    // (a) 带着故障的冷启动 recover：走 `PoolReaderWithUnreadableRootRingSlots` 包读者的那一路。
    let devices_for_recover = devices_from_pool(&node.pool, IMAGE_BYTES);
    let wrapped_reader = PoolReaderWithUnreadableRootRingSlots::new(&devices_for_recover, target);
    let recovery_report = recover(&wrapped_reader, JournalPolicy::Consult);
    let mut recover_failure_detail = String::new();
    let recover_root = match &recovery_report.outcome {
        RecoveryOutcome::NoFile { root } => Some((root.0 .0, root.1 .0)),
        RecoveryOutcome::FileRead { root, .. } => Some((root.0 .0, root.1 .0)),
        RecoveryOutcome::Failed { root, failure } => {
            recover_failure_detail = format!("{failure:?}（所选根 {root:?}）");
            None
        }
    };
    let recover_check_passed = recover_root == Some(expected_root);

    // (b) 带着故障的 mount_writable：走 `FaultInjectingBlockDevice` 包块设备那一路。
    let plan = SharedFaultPlan::unarmed(fixed_geometry_for(&GEOMETRY_PRIMARY));
    plan.arm(FaultSchedule::every_read_of_named_root_ring_slots_fails(
        target,
    ));
    let mut faulted_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        devices_from_pool(&node.pool, IMAGE_BYTES)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(identity, device, plan.clone()),
                )
            })
            .collect();
    let mount_result = mount_writable(parameters, &mut faulted_devices);
    let mut mount_failure_detail = String::new();
    let (mount_writable_check_passed, mount_root) = match mount_result {
        Ok(mounted) => {
            let chosen = root_pair(&mounted.output.chosen_root);
            (chosen == expected_root, Some(chosen))
        }
        Err(error) => {
            mount_failure_detail = format!("{error:?}");
            (false, None)
        }
    };

    emit_result(&format!(
        "name=pc3 path={:?} rollback_instance={session_instance} named_root_slot_count={named_count} expected_root={expected_root:?} recover_verdict={} recover_root={recover_root:?} recover_reads_refused={} recover_failure={recover_failure_detail:?} mount_writable_verdict={} mount_writable_root={mount_root:?} mount_writable_failure={mount_failure_detail:?}",
        node.path,
        if recover_check_passed { "pass" } else { "fail" },
        wrapped_reader.reads_refused(),
        if mount_writable_check_passed { "pass" } else { "fail" }
    ));

    recover_check_passed && mount_writable_check_passed
}

// ============================================================================
// 十二、PC-Nw：手写假账单测计数函数（见 `geometry_and_simulated_history/tests.rs`）。
// ============================================================================

pub(super) fn find_small_ring() -> u64 {
    let mut candidate = 48 * 1024u64;
    loop {
        if small_ring_probe(candidate) {
            return candidate;
        }
        candidate += 16 * 1024;
        assert!(
            candidate <= 3 * 1024 * 1024,
            "小环搜索超过 3 MiB 仍未找到，8.2 的规则或代理探针有问题"
        );
    }
}

// ============================================================================
// 十三、main：按命令行第一个参数选模式，`all` 顺序跑完这一段能跑的全部内容。
// ============================================================================

pub(super) fn run_root_choice_geometry(
    geometry: &Geometry,
    initial_overwrite_counts: &[u64],
    sigma_length_limit: u64,
) -> EnumerationSummary {
    let summary = run_root_choice_family(
        geometry,
        initial_overwrite_counts,
        sigma_length_limit,
        geometry.label,
        false,
    );
    emit_result(&format!(
        "name=h3_family_summary geometry={} overwrites_before_sigma={:?} sigma_length_limit={} histories={} peak_events_live={} peak_roots_live={} steps_with_positive_events_live={} leaf_count={} leaves_with_positive_events_live={} peak_events_live_at_a_leaf={} error_count={}",
        geometry.label,
        initial_overwrite_counts,
        sigma_length_limit,
        summary.histories,
        summary.peak_events_live,
        summary.peak_roots_live,
        summary.steps_with_positive_events_live,
        summary.leaf_count,
        summary.leaves_with_positive_events_live,
        summary.peak_events_live_at_a_leaf,
        summary.errors.len()
    ));
    let mut error_categories: std::collections::BTreeMap<&str, u64> =
        std::collections::BTreeMap::new();
    for error in &summary.errors {
        let category = error
            .split(':')
            .nth(1)
            .map(str::trim)
            .and_then(|rest| rest.split_whitespace().next())
            .unwrap_or("?");
        *error_categories.entry(category).or_insert(0) += 1;
    }
    for (category, count) in &error_categories {
        emit_result(&format!(
            "name=h3_family_pruned_branch_category geometry={} category={category} count={count}",
            geometry.label
        ));
    }
    summary
}

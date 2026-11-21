//! PC2 阳性对照（直接复现判决 K2）、H-C1 记录扫描水位候选的直接构造与对照、H-C2 系统配置槽这一类证据的直接构造与对照。

use super::*;

// ============================================================================
// 十三点五、阳性对照 PC2（岔路单第 2 行 ①）：直接复现判决 K2
//    （`research/prompts/m2-rootchoice-repair-r1-main-verification.md`「### K2」）已经判过的两个
//    具体构造——甲「计数器从全环最大 + 1 起」4 个瞬时根槽读失败打中；乙「系统配置带 txg」0 个注入
//    故障、1 个崩溃点打中——不靠穷举搜索。复现得出，证明装置（故障装配、P331 评判）看得见判决说
//    存在的打中；复现不出，先查装置哪里没对上，不把「穷举没打中」当结论（见交回报告）。
//
//    臂对齐（登记 5.6 项 2、5.3）：K2 表「甲」= 这份登记的「甲-txg」（`first_txg_of_new_instance`
//    只取 `highest_root_txg` + 1，D16（发布语义） 已定项 6 的 checkpoint_txg 读法）。**不是**「甲-jsn」
//    （今天的 `next_counter`）：`mount.rs` 现查过——`next_counter`（第 1422 行 `records.values().
//    map(|record| record.counter).max()...+1`）只决定下一条 journal 记录的 jsn，`first_txg_of_
//    new_instance`（第 322–339 行）从未读它；这两个量在代码里完全不相交，「甲-jsn」在「选根 txg」
//    这一格没有可以打中的机制，K2 给的具体历史（H-A，报告二·一节）用的 W 定义
//    （「环里全部自证过且读得出的根的最大 txg」）逐字对应 `highest_root_txg`，即甲-txg。
//
//    K2 表「乙」= 这份登记的「乙-只配置」的一个更窄子集：只需要系统配置带一个 `published_txg`
//    字段、取号写占位 0、轮换写真实值——不需要乙-留环额外的「续传」那一半（H-B1 只用零故障 + 一个
//    崩溃点，不需要读环）。这个函数只在改了该候选的副本上才会打中；跑在没有 `published_txg` 字段的
//    臂（今天、甲-txg）上必须不打中——见函数内注释。
// ============================================================================

/// PC2 甲专用取故障目标：给定一组 (实例, txg)，在孪生（不注入）镜像上用 `checker_image` 的独立解码
/// 把它们各自解到 (设备, 偏移)——与 `evidence_items_at` 同一套坐标系，只是按 txg 精确点名，不按权重
/// 枚举。
fn root_ring_slot_targets_for(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    wanted: &BTreeSet<TimelineRoot>,
) -> Vec<(DeviceIdentity, DeviceOffsetInBytes)> {
    let positions = checker_image::root_slot_positions(geometry);
    checker_image::valid_roots(pool, geometry)
        .into_iter()
        .filter(|(_, _, view)| wanted.contains(&(view.instance, view.checkpoint_txg)))
        .filter_map(|(region, slot, _view)| {
            positions
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    *candidate_region == region && *candidate_slot == slot
                })
                .map(|(_, _, device_number, offset)| {
                    (DeviceIdentity(*device_number), DeviceOffsetInBytes(*offset))
                })
        })
        .collect()
}

/// PC2 甲专用挂载：一次带故障的 `mount_writable`，**不**按 `MountOutput::effective_root` 截断时间线
/// ——这正是 `attempt_rootback_probe_and_advance` 测不出 H-A 的原因之一（另一个原因是本节顶部记的
/// 多落点装故障的根因，两个原因分开修）：H-A 的机制不是「这次挂载自己的 recover/replay 被故障弄
/// 糊涂」（`effective_root` 靠 journal 记录重建，H-A 只点名根槽、记录仍读得出，`replay_journal`
/// 照样把 `effective_root` 重建回旧时间线的真实末端，`attempt_rootback_probe_and_advance` 因此永远
/// 截不断）——而是「这次挂载自己的 `first_txg_of_new_instance` 被故障压低，新根写进旧根占的物理槽位
/// （按 txg 算槽位，读故障不挡写），旧根本身的字节没被任何写碰到、故障一撤销就又冒出来、在下一次
/// 冷启动的择根里凭更高 txg 赢回去」。这里原样接受 `mount_writable` 给出的 `chosen_root`（不 assert
/// 它等于 tip）、把写行/暖机的根追加在原时间线**末尾**（不做任何截断——旧的高 txg 根条目原样留着）。
fn attempt_a_faulted_mount_writable_and_append(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
) -> Result<SimNode, String> {
    let (mounted, raw_devices) = attempt_a_faulted_mount_writable(node, parameters, fault_targets)?;
    let mut timeline = node.timeline.clone();
    let mut content_by_root = node.content_by_root.clone();
    let carried_forward_content = content_by_root
        .get(&root_pair(&mounted.output.chosen_root))
        .cloned()
        .unwrap_or(None);
    timeline.push(root_pair(mounted.output.row_publish.root()));
    content_by_root.insert(
        root_pair(mounted.output.row_publish.root()),
        carried_forward_content.clone(),
    );
    for warm_up_publish in &mounted.output.warm_up_publishes {
        timeline.push(root_pair(warm_up_publish.root()));
        content_by_root.insert(
            root_pair(warm_up_publish.root()),
            carried_forward_content.clone(),
        );
    }
    let mut path = node.path.clone();
    path.push(HistoryStep::MountWritable);
    Ok(SimNode {
        pool: memory_pool_of(&raw_devices, IMAGE_BYTES),
        session: Some(Session {
            allocator: mounted.allocator,
            current: mounted.current,
            instance: mounted.output.instance,
        }),
        timeline,
        rollback_events: node.rollback_events.clone(),
        path,
        content_by_root,
    })
}

/// PC2 甲：复现 H-A（`research/prompts/m2-rootchoice-repair-r1-opus-output.md`「二·一　修法甲「计数器从全环最大 + 1 起」：历史 H-A（挡不住）」那一节）。
/// 第一次尝试重用 `search_minimum_weight_that_triggers_rootback`（穷举下界那一套）在
/// `overwrites_before_the_rootback_probe = 4` 上跑，**20000 个子集全试完、一次都没打中**——查出
/// 原因：那一套靠 `attempt_rootback_probe_and_advance` 按 `effective_root` 截断时间线，H-A 的机制
/// 与「`effective_root` 被故障影响」无关（见 `attempt_a_faulted_mount_writable_and_append` 的
/// 文档），穷举下界这一套测不出这一类打中——不是「4 个故障不够」，是这件工具量错了维度。改用专门的
/// 构造：n1 = 4 次覆盖写把 tip 推到 (1, 7)（mkfs 起 2 次暖机 + 首个文件 txg 3 + 4 次覆盖写）；对
/// tip 最新的 4 条根槽（对应 H-A「顶上连续 4 个」）精确装读故障；op2 = 带故障 `mount_writable`；
/// 成功则 n2 = 1 次覆盖写；撤故障、判 P331。
pub(super) fn run_positive_control_for_the_root_ring_txg_only_candidate(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_root_ring_txg_only geometry={} candidate=root_ring_txg_only overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_root_ring_txg_only geometry={} candidate=root_ring_txg_only overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let topmost_four: BTreeSet<TimelineRoot> =
        node.timeline.iter().rev().take(4).copied().collect();
    let fault_targets = root_ring_slot_targets_for(&node.pool, &geometry_view, &topmost_four);
    let fault_target_count = fault_targets.len();
    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_root_ring_txg_only geometry={} candidate=root_ring_txg_only overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} fault_target_count={fault_target_count} error={error:?} stage=the_faulted_mount",
                geometry.label
            ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_pc2_root_ring_txg_only geometry={} candidate=root_ring_txg_only overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} fault_target_count={fault_target_count} error={error:?} stage=the_confirmed_overwrite",
                    geometry.label
                ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_pc2_root_ring_txg_only geometry={} candidate=root_ring_txg_only overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 topmost_four={topmost_four:?} fault_target_count={fault_target_count} expected_by_k2_table_fault_count=4 hit={hit} expected_by_k2_table_hit=true first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

/// PC2 乙：复现 H-B1（`research/prompts/m2-rootchoice-repair-r1-opus-output.md`「二·二　修法乙「系统配置带 txg」：历史 H-B1（挡不住，0 个注入故障）」那一节）。
/// 造一次「崩在取号之后、写行发布最早的字节之前」——`acquire_instance`（`crates/singlefs-core`
/// 公开函数）自己只做「逐盘写系统配置槽 + 一道屏障」两步（`transaction.rs` 的 `write_acquired_
/// instance`），中途没有别的写；单独调它、原样收工，产生的设备字节与那个崩溃点完全相同，不需要层
/// 0 的崩溃点枚举器。之后在这份「崩溃」镜像上正常做一次 `mount_writable`（相当于 H-B1 的挂载 3）
/// 加一次覆盖写（n2 = 1），再判 P331。
pub(super) fn run_positive_control_for_the_configuration_published_txg_candidate(
    geometry: &Geometry,
) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_crash = 2u64;
    let node = match bootstrap(geometry, overwrites_before_the_crash) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_configuration_published_txg geometry={} candidate=configuration_published_txg overwrites_before_the_crash={overwrites_before_the_crash} error={error:?} stage=bootstrap"
            , geometry.label));
            return;
        }
    };
    let mut devices_for_the_crashed_acquisition = devices_from_pool(&node.pool, IMAGE_BYTES);
    let acquisition_before_the_crash = {
        let mut writer = PoolWriter::new(
            &parameters,
            devices_for_the_crashed_acquisition.as_mut_slice(),
        );
        acquire_instance(&mut writer)
    };
    let acquired_instance_before_the_crash = match acquisition_before_the_crash {
        Ok(instance) => instance,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_configuration_published_txg geometry={} candidate=configuration_published_txg overwrites_before_the_crash={overwrites_before_the_crash} error={error:?} stage=acquire_instance_before_the_crash"
            , geometry.label));
            return;
        }
    };
    let node_after_the_crash = SimNode {
        pool: memory_pool_of(&devices_for_the_crashed_acquisition, IMAGE_BYTES),
        ..node.clone()
    };
    let node_after_the_next_mount = match apply_mount_writable(&node_after_the_crash, &parameters) {
        Ok(mounted_node) => mounted_node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_pc2_configuration_published_txg geometry={} candidate=configuration_published_txg overwrites_before_the_crash={overwrites_before_the_crash} error={error:?} stage=mount_after_the_crash"
            , geometry.label));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_next_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_pc2_configuration_published_txg geometry={} candidate=configuration_published_txg overwrites_before_the_crash={overwrites_before_the_crash} error={error:?} stage=overwrite_after_the_crash"
                , geometry.label));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_pc2_configuration_published_txg geometry={} candidate=configuration_published_txg overwrites_before_the_crash={overwrites_before_the_crash} acquired_instance_before_the_crash={} injected_faults=0 crash_points=1 hit={hit} expected_by_k2_table=true first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label, acquired_instance_before_the_crash.0
    ));
}

/// H-C1 直接构造（判决 K2 给丙的具体历史，`m2-rootchoice-repair-r1-main-verification.md`「K2」一节「丙　记录扫描水位」那一行：
/// 「12 个」= 4 根槽 + 4 条记录各在两块盘上都读不出，总权重 4×1 + 4×2 = 12）。丙 = 今天的
/// `first_txg_of_new_instance`，同时取根环与记录两路的 max（`mount.rs` 的
/// `highest_ring_txg.max(highest_record_txg)`）——只挡根环（甲-txg 那种攻法）挡不住它：记录那一路
/// 会把真实的 tip txg 重新暴露出来。这里在与甲-txg 同一段历史（n1 = 4，tip 落在顶上 4 条根槽）上，
/// 除了精确点名顶上 4 条根槽（复用 `root_ring_slot_targets_for`，与 PC2 甲同一份坐标系），另外从
/// `evidence_items_at` 取记录类证据项里计数器最大的 4 条（`evidence_items_at` 按 ring 槽位
/// 0..ring_slots 递增枚举、无绕环时恰是计数器递增序，取尾部 4 条即计数器最大的 4 条），两份镜像上
/// 各自的 (设备, 偏移) 全部拦下（每条记录 `targets.len()==2`，在不注入的孪生镜像上取到，权重恰为
/// 2）。验证丙在这个具体构造下是否被打中，是不是恰好需要这 12 个。
pub(super) fn run_direct_construction_for_the_record_scan_watermark_candidate(geometry: &Geometry) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_record_scan_watermark geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_record_scan_watermark geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let topmost_four: BTreeSet<TimelineRoot> =
        node.timeline.iter().rev().take(4).copied().collect();
    let root_fault_targets = root_ring_slot_targets_for(&node.pool, &geometry_view, &topmost_four);
    let root_target_count = root_fault_targets.len();

    let evidence_items = evidence_items_at(&node.pool, &geometry_view, false);
    let record_items: Vec<&EvidenceItem> = evidence_items
        .iter()
        .filter(|item| item.kind == EvidenceKind::JournalRecord)
        .collect();
    let top_four_records: Vec<&EvidenceItem> = record_items.iter().rev().take(4).copied().collect();
    let top_four_record_labels: Vec<String> = top_four_records
        .iter()
        .map(|item| item.label.clone())
        .collect();
    let record_fault_targets: Vec<(DeviceIdentity, DeviceOffsetInBytes)> = top_four_records
        .iter()
        .flat_map(|item| item.targets.clone())
        .collect();
    let record_target_count = record_fault_targets.len();

    let mut fault_targets = root_fault_targets;
    fault_targets.extend(record_fault_targets);
    let fault_target_count = fault_targets.len();
    let fault_weight = root_target_count + 2 * top_four_records.len();

    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_record_scan_watermark geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} record_labels={top_four_record_labels:?} root_target_count={root_target_count} record_target_count={record_target_count} fault_target_count={fault_target_count} fault_weight={fault_weight} error={error:?} stage=the_faulted_mount",
                geometry.label
            ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_record_scan_watermark geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} record_labels={top_four_record_labels:?} root_target_count={root_target_count} record_target_count={record_target_count} fault_target_count={fault_target_count} fault_weight={fault_weight} error={error:?} stage=the_confirmed_overwrite",
                geometry.label
            ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_h_c1_record_scan_watermark geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 topmost_four={topmost_four:?} record_labels={top_four_record_labels:?} root_target_count={root_target_count} record_target_count={record_target_count} fault_target_count={fault_target_count} fault_weight={fault_weight} expected_by_k2_table_fault_count=12 hit={hit} expected_by_k2_table_hit=true first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

/// H-C1 负对照：只拦 4 条根槽、不拦记录（= 甲-txg 那种攻法的目标集合），丙必须不打中——证明
/// 「只挡根环挡不住丙」这句话，不是「随便挡够 4 个目标就够了」。
pub(super) fn run_negative_control_root_slots_only_for_the_record_scan_watermark_candidate(
    geometry: &Geometry,
) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_negative_control_root_slots_only geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_negative_control_root_slots_only geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let topmost_four: BTreeSet<TimelineRoot> =
        node.timeline.iter().rev().take(4).copied().collect();
    let fault_targets = root_ring_slot_targets_for(&node.pool, &geometry_view, &topmost_four);
    let fault_target_count = fault_targets.len();
    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_negative_control_root_slots_only geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} fault_target_count={fault_target_count} error={error:?} stage=the_faulted_mount",
                geometry.label
            ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_negative_control_root_slots_only geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} topmost_four={topmost_four:?} fault_target_count={fault_target_count} error={error:?} stage=the_confirmed_overwrite",
                geometry.label
            ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_h_c1_negative_control_root_slots_only geometry={} candidate=record_scan_watermark overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 topmost_four={topmost_four:?} fault_target_count={fault_target_count} expected_by_k2_table_fault_count=4 hit={hit} expected_hit=false first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

/// H-C1 下界探针（参数 `weight_ceiling`）：直接构造（上面两个
/// 函数）证实丙在 n1=4 这段历史上权重 12 能打中；`run_rootback_tolerance_family` 的主搜索在 n1=4 上
/// 受计数上限限制，没有覆盖到权重 12 以下的全部组合。这里在同一个 n1=4 节点上，把
/// `search_minimum_weight_that_triggers_rootback` 的 `weight_ceiling` 显式设成调用方给定的权重，
/// 只为回答一个问题：穷举能不能在这一档权重内走到头（找到一个更小的命中会推翻「恰好 12」，走到权重
/// 12 都没有命中而 12 本身打中会坐实「恰好 12」，`weight_ceiling` 传 `None` 时穷举到完整空间的顶）。
/// 不改判据、不改臂定义，只是把已有搜索函数的一个参数换成调用方要的权重。
pub(super) fn run_record_scan_watermark_lower_bound_probe(
    geometry: &Geometry,
    weight_ceiling: Option<u64>,
) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_lower_bound_probe geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} weight_ceiling={weight_ceiling:?} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c1_lower_bound_probe geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} weight_ceiling={weight_ceiling:?} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let outcome = search_minimum_weight_that_triggers_rootback(
        &node,
        &parameters,
        &geometry_view,
        1,
        weight_ceiling,
        false,
    );
    match outcome.weight_at_first_hit {
        Some(weight) => emit_result(&format!(
            "name=q2_1_h_c1_lower_bound_probe geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} weight_ceiling={weight_ceiling:?} k_min={weight} hit_count_at_that_weight={} first_hit_combination={:?} subsets_tried={} full_space_subset_count={} rootback_probe_failures={} overwrite_failures={}",
            geometry.label,
            outcome.hit_count_at_that_weight,
            outcome.first_hit_combination_labels,
            outcome.subsets_tried,
            outcome.full_space_subset_count,
            outcome.rootback_probe_failures,
            outcome.overwrite_failures
        )),
        None if outcome.stopped_by_weight_ceiling => emit_result(&format!(
            "name=q2_1_h_c1_lower_bound_probe geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} weight_ceiling={weight_ceiling:?} k_min=not_found_up_to_weight_ceiling highest_weight_examined={} maximum_weight_available={} full_space_subset_count={} subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
            geometry.label, outcome.highest_weight_examined, outcome.maximum_weight, outcome.full_space_subset_count, outcome.subsets_tried, outcome.rootback_probe_failures, outcome.overwrite_failures
        )),
        None => emit_result(&format!(
            "name=q2_1_h_c1_lower_bound_probe geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} weight_ceiling={weight_ceiling:?} k_min=none_found_within_full_evidence_space maximum_weight_available={} full_space_subset_count={} subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
            geometry.label, outcome.maximum_weight, outcome.full_space_subset_count, outcome.subsets_tried, outcome.rootback_probe_failures, outcome.overwrite_failures
        )),
    }
}

// ============================================================================
// 十三点六、H-C2 直接构造（岔路单第 2 行 Φ2 系统配置槽这一类证据）：
//
//    机制假说：`choose_system_configuration`（`crates/singlefs-core/src/recovery.rs` 里
//    现查）按 `reader.device_identities()` 的顺序取**第一块「至少有一份自证槽」的盘**的
//    `best_on_device` 直接当 `chosen`——`chosen` 只在 `None` 时被赋值一次（第 559–560 行），后面的
//    盘只核对 `filesystem_identifier` / `device_count` 一致，**不参与「取哪块盘的值」这一步**。
//    只要把设备 0 上世代号更高的那一槽读失败，`best_on_device` 就回退到设备 0 自己世代号更旧的那
//    一槽（两槽都自证过，见 `system_configuration_slot_positions`），`chosen` 里的 `published_txg`
//    随之读成旧值——设备 1 上真实的新值完全不参与这一步。权重恰为 1。三个函数都用与 H-C1 相同的
//    「append 不截断」挂载路径（`attempt_a_faulted_mount_writable_and_append`），与
//    `run_rootback_tolerance_family_with_system_configuration_evidence` 的穷举路径
//    （`attempt_rootback_probe_and_advance`）是两条独立的代码路径，互为校验。
// ============================================================================

/// 在孪生（不注入）镜像上找设备 0 两个槽里世代号更高的那一个：读原始字节、用
/// `singlefs_checker::check_system_configuration_slot` 校验+解码，取 `slot_generation` 更大的那份。
/// 两槽都读不出或都解不开 ⇒ `None`（这段历史还没轮换过第二次，构造不出，由调用方报错退出）。
fn newest_self_certified_system_configuration_slot_on_device_zero(
    pool: &MemoryPool,
    slot_spacing: u64,
) -> Option<(u64, u64)> {
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut newest: Option<(u64, u64)> = None;
    for (device_number, slot_index, offset) in system_configuration_slot_positions(slot_spacing) {
        if device_number != 0 {
            continue;
        }
        let Some(bytes) = pool.read(device_number, offset, slot_bytes) else {
            continue;
        };
        let Ok(view) = singlefs_checker::check_system_configuration_slot(&bytes) else {
            continue;
        };
        if newest.is_none_or(|(_, generation)| view.slot_generation > generation) {
            newest = Some((slot_index, view.slot_generation));
        }
    }
    newest
}

/// 同上，取世代号更小（较旧）的那一份；两槽世代号相等（只轮换过一次）时交回 `None`（负对照这段
/// 历史用不上，跳过不报错）。
fn oldest_self_certified_system_configuration_slot_on_device_zero(
    pool: &MemoryPool,
    slot_spacing: u64,
) -> Option<(u64, u64)> {
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut generations: Vec<(u64, u64)> = Vec::new();
    for (device_number, slot_index, offset) in system_configuration_slot_positions(slot_spacing) {
        if device_number != 0 {
            continue;
        }
        let Some(bytes) = pool.read(device_number, offset, slot_bytes) else {
            continue;
        };
        let Ok(view) = singlefs_checker::check_system_configuration_slot(&bytes) else {
            continue;
        };
        generations.push((slot_index, view.slot_generation));
    }
    if generations.len() < 2 {
        return None;
    }
    generations
        .into_iter()
        .min_by_key(|(_, generation)| *generation)
}

/// H-C2 直接构造：只把设备 0 上世代号更高的那一槽读失败（权重 1），验证机制假说描述的打中。
pub(super) fn run_direct_construction_for_the_system_configuration_evidence_class(
    geometry: &Geometry,
) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let Some((slot_index, generation)) =
        newest_self_certified_system_configuration_slot_on_device_zero(
            &node.pool,
            geometry_view.slot_spacing,
        )
    else {
        emit_result(&format!(
            "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error=\"设备 0 上没有自证过的系统配置槽\" stage=find_newest_slot",
            geometry.label
        ));
        return;
    };
    let offset = slot_index * geometry_view.slot_spacing;
    let fault_targets = vec![(DeviceIdentity(0), DeviceOffsetInBytes(offset))];
    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_faulted_mount",
                    geometry.label
                ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_confirmed_overwrite",
                    geometry.label
                ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_h_c2_system_configuration_evidence geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 mechanism_hypothesis=fault_the_newer_generation_slot_on_device_0 hit={hit} first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

/// 负对照一：只把设备 0 上世代号更旧的那一槽读失败——`best_on_device` 应当仍旧解出自证过的、
/// 世代号更高的那一槽，`chosen` 不受影响，必须不打中。历史上两槽世代号还没分开过（只轮换过一次）
/// 时这段历史跳过，报 `skipped=true`，不算「不打中」。
pub(super) fn run_negative_control_older_slot_for_the_system_configuration_evidence_class(
    geometry: &Geometry,
) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let Some((slot_index, generation)) =
        oldest_self_certified_system_configuration_slot_on_device_zero(
            &node.pool,
            geometry_view.slot_spacing,
        )
    else {
        emit_result(&format!(
            "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} skipped=true reason=\"设备 0 上两槽世代号还没分开过\"",
            geometry.label
        ));
        return;
    };
    let offset = slot_index * geometry_view.slot_spacing;
    let fault_targets = vec![(DeviceIdentity(0), DeviceOffsetInBytes(offset))];
    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_faulted_mount",
                    geometry.label
                ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_confirmed_overwrite",
                    geometry.label
                ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_h_c2_negative_control_older_slot geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 device0_faulted_slot_index={slot_index} device0_faulted_slot_generation={generation} fault_target_count=1 skipped=false hit={hit} expected_hit=false first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

/// 负对照二：只把设备 1 上世代号更高的那一槽读失败（不碰设备 0）——机制假说说的是「先到手的那块盘
/// 说了算」，设备 0 排在前面、没被碰过，`chosen` 应当仍取设备 0 的当前值，必须不打中。
pub(super) fn run_negative_control_other_device_for_the_system_configuration_evidence_class(
    geometry: &Geometry,
) {
    let parameters = parameters_for(geometry);
    let overwrites_before_the_rootback_probe = 4u64;
    let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?} stage=bootstrap",
                geometry.label
            ));
            return;
        }
    };
    let geometry_view = match independent_geometry(&node.pool) {
        Ok(view) => view,
        Err(error) => {
            emit_result(&format!(
                "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error} stage=geometry",
                geometry.label
            ));
            return;
        }
    };
    let slot_bytes =
        usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let mut newest_on_device_one: Option<(u64, u64)> = None;
    for (device_number, slot_index, offset) in
        system_configuration_slot_positions(geometry_view.slot_spacing)
    {
        if device_number != 1 {
            continue;
        }
        let Some(bytes) = node.pool.read(device_number, offset, slot_bytes) else {
            continue;
        };
        let Ok(view) = singlefs_checker::check_system_configuration_slot(&bytes) else {
            continue;
        };
        if newest_on_device_one.is_none_or(|(_, generation)| view.slot_generation > generation) {
            newest_on_device_one = Some((slot_index, view.slot_generation));
        }
    }
    let Some((slot_index, generation)) = newest_on_device_one else {
        emit_result(&format!(
            "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error=\"设备 1 上没有自证过的系统配置槽\" stage=find_newest_slot",
            geometry.label
        ));
        return;
    };
    let offset = slot_index * geometry_view.slot_spacing;
    let fault_targets = vec![(DeviceIdentity(1), DeviceOffsetInBytes(offset))];
    let node_after_the_faulted_mount = match attempt_a_faulted_mount_writable_and_append(
        &node,
        &parameters,
        &fault_targets,
    ) {
        Ok(node) => node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device1_faulted_slot_index={slot_index} device1_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_faulted_mount",
                    geometry.label
                ));
            return;
        }
    };
    let node_after_the_confirmed_overwrite = match apply_overwrite(
        &node_after_the_faulted_mount,
        &parameters,
    ) {
        Ok(overwritten_node) => overwritten_node,
        Err(error) => {
            emit_result(&format!(
                    "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} device1_faulted_slot_index={slot_index} device1_faulted_slot_generation={generation} fault_target_count=1 error={error:?} stage=the_confirmed_overwrite",
                    geometry.label
                ));
            return;
        }
    };
    let (hit, first_check_detail, second_check_detail) =
        evaluate_rootback_predicate(&node_after_the_confirmed_overwrite, &parameters);
    emit_result(&format!(
        "name=q2_1_h_c2_negative_control_other_device geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe=1 device1_faulted_slot_index={slot_index} device1_faulted_slot_generation={generation} fault_target_count=1 hit={hit} expected_hit=false first_check={first_check_detail:?} second_check={second_check_detail:?}",
        geometry.label
    ));
}

#[cfg(test)]
mod tests;

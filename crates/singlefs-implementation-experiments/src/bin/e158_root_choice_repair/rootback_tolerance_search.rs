//! Q2-1：H2 家族 + Φ2 证据项枚举 + 权重从小到大的穷举下界搜索。

use super::*;

// ============================================================================
// 十三、Q2-1（跑前登记岔路单第 2 行 ①，C331 修法）：H2 家族 + Φ2 证据项枚举 + 权重从小到大的穷举下界搜索。
//
//    跑前登记 5.1/5.3 里没做的部分（交回报告「它答不了的」逐条点名）：
//    「构造上界」那一半、H2-R（op2 = `mount_rollback`）、H2c（崩溃档）、系统配置证据项这一类
//    （丙 = 甲-jsn 与甲-txg 两条臂的 `first_txg_of_new_instance` 都不读系统配置，藏它对这两条臂的
//    择根无影响，跳过不改变这两条臂的搜索结果）、PC2 阳性对照（需要给每条臂的副本加一个只供本实验的
//    环境变量开关，属于另一块工程量）、n2=2、n1>3。这一段只做：H2 主族（op2=`mount_writable`）在
//    n1 ∈ {0,1,2,3}、n2=1 上、Φ2 = {根环槽, journal 记录} 两类证据的穷举下界。
//
//    系统配置证据项这一类（`evidence_items_at` 新增
//    `include_system_configuration_slots` 开关），但仍然**只在开关打开时**才进 Φ2——甲-txg、丙、
//    今天（=甲-jsn）三条臂不读系统配置，开着这个开关也不会改变它们的搜索结果，只会白白扩大穷举
//    空间；`run_rootback_tolerance_family`（既有六份副本用的那个）与 `search_minimum_weight_that_
//    triggers_rootback` 的既有调用点都仍传 `false`，行为逐字节不变。新增
//    `run_rootback_tolerance_family_with_system_configuration_evidence` 只在读系统配置的三条臂
//    （乙-留环、丁-留环、丁-只配置）上跑。
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EvidenceKind {
    RootRingSlot,
    JournalRecord,
    /// 第三类（跑前登记 5.1「每块盘上每个自证的系统配置槽（权重 1）」）：只在
    /// `evidence_items_at` 的 `include_system_configuration_slots` 打开时才会出现。
    SystemConfigurationSlot,
}

/// Φ2 的一个证据项：根环槽权重恒 1（根环每个区域第一版只住一块盘，D22（单元原子性怎么合成） 已定项 16
/// 第 4 条）；journal 记录的权重 = 它当下自证过的份数（`targets.len()`，不注入时通常 2）；系统配置槽
/// 权重恒 1（每个 (设备, 槽) 各自一份，不像根环槽那样跨盘）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EvidenceItem {
    pub(super) kind: EvidenceKind,
    pub(super) label: String,
    pub(super) targets: Vec<(DeviceIdentity, DeviceOffsetInBytes)>,
}

/// 系统配置槽第 0、1 槽各自在两块盘上的 (设备, 槽号, 偏移)：槽 0 恒在偏移 0，槽 1 在偏移
/// `slot_spacing`（`PoolGeometry` 已经把它解出来了，D2（RAID 条带策略） 已定项 19：
/// `max(4096, mkfs 时探测的 io_min)`，池级一个值，两块盘同一个槽距）；固定两块盘（这个实验全程
/// devs=2），不看几何里的 `regions`。
pub(super) fn system_configuration_slot_positions(slot_spacing: u64) -> [(u32, u64, u64); 4] {
    [
        (0, 0, 0),
        (0, 1, slot_spacing),
        (1, 0, 0),
        (1, 1, slot_spacing),
    ]
}

/// op2 那一刻不注入的孪生镜像上的证据项：根环里每个存着自证根的槽，加上 journal 环里每一条至少一份
/// 自证的记录；`include_system_configuration_slots` 打开时再加每块盘上每个自证过的系统配置槽
/// （跑前登记 5.1 Φ2 定义的第三类——关着时行为与不含这一类时逐字节不变，见本节顶部说明）。
pub(super) fn evidence_items_at(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    include_system_configuration_slots: bool,
) -> Vec<EvidenceItem> {
    let mut items = Vec::new();
    let valid_roots = checker_image::valid_roots(pool, geometry);
    let positions = checker_image::root_slot_positions(geometry);
    for (region, slot, _view) in &valid_roots {
        if let Some((_, _, device_number, offset)) =
            positions
                .iter()
                .find(|(candidate_region, candidate_slot, _, _)| {
                    candidate_region == region && candidate_slot == slot
                })
        {
            items.push(EvidenceItem {
                kind: EvidenceKind::RootRingSlot,
                label: format!("root(region={region},slot={slot})"),
                targets: vec![(DeviceIdentity(*device_number), DeviceOffsetInBytes(*offset))],
            });
        }
    }
    let expected_filesystem_identifier = unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    let ring_slots = geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES;
    for slot_index in 0..ring_slots {
        let offset = record_offset(slot_index + 1, geometry.journal_ring_bytes);
        let mut targets = Vec::new();
        let mut sample = None;
        for device_number in [0u32, 1u32] {
            if let Some(bytes) = pool.read(device_number, offset.0, record_bytes) {
                if let Some(record) = JournalRecord::parse(&bytes, expected_filesystem_identifier) {
                    targets.push((DeviceIdentity(device_number), offset));
                    sample.get_or_insert((record.instance.0, record.counter));
                }
            }
        }
        if let Some((instance, counter)) = sample {
            items.push(EvidenceItem {
                kind: EvidenceKind::JournalRecord,
                label: format!("record(instance={instance},counter={counter})"),
                targets,
            });
        }
    }
    if include_system_configuration_slots {
        let slot_bytes =
            usize::try_from(singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
        for (device_number, slot_index, offset) in
            system_configuration_slot_positions(geometry.slot_spacing)
        {
            if let Some(bytes) = pool.read(device_number, offset, slot_bytes) {
                if singlefs_checker::check_system_configuration_slot(&bytes).is_ok() {
                    items.push(EvidenceItem {
                        kind: EvidenceKind::SystemConfigurationSlot,
                        label: format!("configuration(device={device_number},slot={slot_index})"),
                        targets: vec![(DeviceIdentity(device_number), DeviceOffsetInBytes(offset))],
                    });
                }
            }
        }
    }
    items
}

/// 权重恰为 `weight` 的证据项子集：权重恒 1 的那一批（根槽；`include_system_configuration_slots`
/// 打开时系统配置槽也并进这一批，两类都是权重 1，对这个函数的组合算术没有分别）取
/// `weight_one_count` 个、记录（权重 2，见 `evidence_items_at` 里 `targets` 恒两份）取
/// `record_count` 个，`weight_one_count + 2 * record_count = weight`，逐个 `record_count` 值把
/// 两边的组合叉乘。
fn candidate_fault_sets_of_weight(
    weight_one_items: &[EvidenceItem],
    journal_record_items: &[EvidenceItem],
    weight: u64,
) -> Vec<Vec<EvidenceItem>> {
    let mut candidates = Vec::new();
    let mut record_count = 0u64;
    while record_count * 2 <= weight {
        let weight_one_count = weight - record_count * 2;
        if let (Ok(weight_one_count), Ok(record_count_usize)) = (
            usize::try_from(weight_one_count),
            usize::try_from(record_count),
        ) {
            if weight_one_count <= weight_one_items.len()
                && record_count_usize <= journal_record_items.len()
            {
                for weight_one_combination in
                    combinations_of_size(weight_one_items, weight_one_count)
                {
                    for record_combination in
                        combinations_of_size(journal_record_items, record_count_usize)
                    {
                        let mut combination = weight_one_combination.clone();
                        combination.extend(record_combination.iter().cloned());
                        candidates.push(combination);
                    }
                }
            }
        }
        record_count += 1;
    }
    candidates
}

/// PC2 排查出的根因：`FaultInjectingBlockDevice` 一次只装一条
/// `SharedFaultPlan`、一条计划只认一个精确偏移；`fault_injection.rs` 自己的读法写死表说得很清楚
/// 「一个 `SharedFaultPlan` 一次只装一条计划，多落点只能这样叠」（一块盘一层）——而
/// `attempt_rootback_probe_and_advance`（下面这个函数）原来对每块盘只用 `.find()` 挑
/// `fault_targets` 里第一个落在这块盘的目标去装那一层，**同一块盘上第二个及以后的目标偏移从未真正
/// 装上故障**。只有 2 块盘时，一个多落点组合里「至少两个落在同一块盘」极常见（按鸽笼原理，权重 ≥ 2
/// 就可能撞上、权重越高越必然撞上），H2 全家族至今用这套机制跑出的「0 命中」都因此打上问号——
/// 这不是「故障不够多」，是**故障根本没有全装上**。修法：不再嵌套 `FaultInjectingBlockDevice`，
/// 换一个能一次记住多个精确偏移、每次读都在这些偏移上报错的本地包装，一块盘一份、一次装完它全部
/// 该装的目标。
pub(super) struct MultiOffsetReadFailingBlockDevice {
    pub(super) inner: SparseBlockDevice,
    pub(super) failing_offsets: BTreeSet<u64>,
}

impl BlockDevice for MultiOffsetReadFailingBlockDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        if self.failing_offsets.contains(&offset.0) {
            return Err(injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 把 `node.pool` 按 `fault_targets` 包上多落点读故障（一块盘一份 `MultiOffsetReadFailingBlockDevice`，
/// 每一份记它自己那几个精确偏移），跑一次 `mount_writable`，交回挂载结果与撤故障之后的原始设备。
pub(super) fn attempt_a_faulted_mount_writable(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
) -> Result<
    (
        singlefs_core::mount::Mounted,
        Vec<(DeviceIdentity, SparseBlockDevice)>,
    ),
    String,
> {
    let plain = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mut wrapped: Vec<(DeviceIdentity, MultiOffsetReadFailingBlockDevice)> = Vec::new();
    for (identity, device) in plain {
        let failing_offsets: BTreeSet<u64> = fault_targets
            .iter()
            .filter(|(target_device, _)| *target_device == identity)
            .map(|(_, offset)| offset.0)
            .collect();
        wrapped.push((
            identity,
            MultiOffsetReadFailingBlockDevice {
                inner: device,
                failing_offsets,
            },
        ));
    }
    let mounted = mount_writable(parameters, &mut wrapped)
        .map_err(|error| format!("faulted mount_writable: {error:?}"))?;
    let raw_devices: Vec<(DeviceIdentity, SparseBlockDevice)> = wrapped
        .into_iter()
        .map(|(identity, wrapped_device)| (identity, wrapped_device.inner))
        .collect();
    Ok((mounted, raw_devices))
}

/// 一次 op2 尝试：按 `fault_targets` 装故障，只在这次调用期间生效（瞬时），成功就把落盘状态解出来、
/// 按 `MountOutput::effective_root` 在装置自记时间线上的位置截断——这正是 H2 要测的：故障让
/// `effective_root` 从比时间线尾更旧的根接着走。与 `apply_mount_rollback` 同一套「父链」逻辑，
/// 但目标是 `effective_root`，不是调用方给的回退目标，也不记 `RollbackEvent`（这不是一次语义回退，
/// 是故障造成的断支，5.0「末端之后那一截记作断支」）。
fn attempt_rootback_probe_and_advance(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
) -> Result<SimNode, String> {
    let (mounted, raw_devices) = attempt_a_faulted_mount_writable(node, parameters, fault_targets)
        .map_err(|error| format!("op2: {error}"))?;
    let effective = root_pair(&mounted.output.effective_root);
    let Some(position) = node.timeline.iter().position(|root| *root == effective) else {
        return Err(format!(
            "op2 的 effective_root {effective:?} 不在装置自记的时间线 {:?} 上（不应发生）",
            node.timeline
        ));
    };
    let mut timeline = node.timeline[..=position].to_vec();
    let mut content_by_root = node.content_by_root.clone();
    let carried_forward_content = content_by_root.get(&effective).cloned().unwrap_or(None);
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

/// P331 的一处判定（5.0）：所选根不在时间线上记「倒挂」；在，而结局不是 `FileRead` 带时间线末端那一版
/// 的内容记「读不回」。
fn judge_recovery_outcome(
    label: &str,
    outcome: &RecoveryOutcome,
    timeline: &[TimelineRoot],
    expected_content: &Option<Vec<u8>>,
) -> (bool, String) {
    let chosen = match outcome {
        RecoveryOutcome::NoFile { root } => Some((root.0 .0, root.1 .0)),
        RecoveryOutcome::FileRead { root, .. } => Some((root.0 .0, root.1 .0)),
        RecoveryOutcome::Failed { root, .. } => root.map(|value| (value.0 .0, value.1 .0)),
    };
    let Some(chosen) = chosen else {
        return (
            false,
            format!("{label}: recover 完全失败 outcome={outcome:?}"),
        );
    };
    if !timeline.contains(&chosen) {
        return (false, format!("{label}: 倒挂 chosen_root={chosen:?}"));
    }
    match outcome {
        RecoveryOutcome::FileRead { content, .. } if Some(content.clone()) == *expected_content => {
            (true, "ok".to_string())
        }
        RecoveryOutcome::NoFile { .. } if expected_content.is_none() => (true, "ok".to_string()),
        RecoveryOutcome::NoFile { .. }
        | RecoveryOutcome::FileRead { .. }
        | RecoveryOutcome::Failed { .. } => (
            false,
            format!("{label}: 读不回 outcome={outcome:?} expected={expected_content:?}"),
        ),
    }
}

/// P331 两处判定都跑（5.0）：① 冷启动 recover；② 再做一次不注入的 `mount_writable`，之后再冷启动
/// recover。任一处不成立即「打中」。交回 (打中?, 检查①的细节, 检查②的细节)。
pub(super) fn evaluate_rootback_predicate(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
) -> (bool, String, String) {
    let tip = *node.timeline.last().expect("timeline 非空");
    let expected_content = node.content_by_root.get(&tip).cloned().unwrap_or(None);

    let devices_for_first_check = devices_from_pool(&node.pool, IMAGE_BYTES);
    let first_report = recover(&devices_for_first_check, JournalPolicy::Consult);
    let (first_check_passed, first_detail) = judge_recovery_outcome(
        "cold_recover",
        &first_report.outcome,
        &node.timeline,
        &expected_content,
    );

    let mut devices_for_second_check = devices_from_pool(&node.pool, IMAGE_BYTES);
    let (second_check_passed, second_detail) = match mount_writable(
        parameters,
        &mut devices_for_second_check,
    ) {
        Err(error) => (false, format!("unfaulted_mount_writable: {error:?}")),
        Ok(mounted) => {
            let chosen = root_pair(&mounted.output.chosen_root);
            if !node.timeline.contains(&chosen) {
                (
                    false,
                    format!("unfaulted_mount_writable: 倒挂 chosen_root={chosen:?}"),
                )
            } else {
                // 5.0 原文这一半只比内容号，不再比「所选根在不在时间线上」：这次探针性的
                // `mount_writable` 本身推进了状态（新实例的写行/暖机），它自己的
                // `chosen_root` 已经在上面判过「倒挂」；读回时的所选根天然是这个新实例的
                // 根，不在 `node.timeline`（那份只记到 op2 探针为止），重新套一次
                // `judge_recovery_outcome` 的「倒挂」半句会把这个正常前进误判成倒挂。
                let pool_after = memory_pool_of(&devices_for_second_check, IMAGE_BYTES);
                let devices_for_readback = devices_from_pool(&pool_after, IMAGE_BYTES);
                let readback_report = recover(&devices_for_readback, JournalPolicy::Consult);
                match &readback_report.outcome {
                        RecoveryOutcome::FileRead { content, .. }
                            if Some(content.clone()) == expected_content =>
                        {
                            (true, "ok".to_string())
                        }
                        RecoveryOutcome::NoFile { .. } if expected_content.is_none() => {
                            (true, "ok".to_string())
                        }
                        other @ (RecoveryOutcome::NoFile { .. }
                        | RecoveryOutcome::FileRead { .. }
                        | RecoveryOutcome::Failed { .. }) => (
                            false,
                            format!(
                                "unfaulted_mount_writable_then_recover: 读不回 outcome={other:?} expected={expected_content:?}"
                            ),
                        ),
                    }
            }
        }
    };
    (
        !(first_check_passed && second_check_passed),
        first_detail,
        second_detail,
    )
}

pub(super) struct RootbackToleranceSearchOutcome {
    pub(super) weight_at_first_hit: Option<u64>,
    pub(super) hit_count_at_that_weight: u64,
    /// 打中这一权重的第一个子集，按各证据项的 `label` 记（跑前登记 5.1「记下这一权重上打中的子集
    /// 个数与第一个子集」）。
    pub(super) first_hit_combination_labels: Vec<String>,
    /// 这一次搜索实际枚举过的子集总数（各完整权重层累计；从不在某一档中途停，见函数文档）。
    pub(super) subsets_tried: u64,
    /// （`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行还差项②）：搜到的最高一档权重，
    /// 与 `weight_at_first_hit`/`stopped_by_weight_ceiling` 一起读——三者合起来就是
    /// 「穷举到权重几」的完整交代，不许只报一个「capped」布尔值把这句话吞掉。
    pub(super) highest_weight_examined: u64,
    /// 因为传了 `weight_ceiling` 而在权重层边界（不是档中途）停下、且这一档还没有打中、且没有到
    /// `maximum_weight`（还有没搜到的档）。真时这一行必须同时报 `full_space_subset_count` 与
    /// `highest_weight_examined`，不许只写「capped」三个字。
    pub(super) stopped_by_weight_ceiling: bool,
    pub(super) rootback_probe_failures: u64,
    pub(super) overwrite_failures: u64,
    /// 这个模板 Φ2（根槽 + journal 记录两类，`include_system_configuration_slots` 打开时再加系统
    /// 配置槽）的证据项权重之和：`weight_at_first_hit` 为 `None` 且 `stopped_by_weight_ceiling` 为假
    /// 时，说明穷举已经走完全部权重都没有打中——这个数就是「这个模板最多能凑出多大的故障集合」，
    /// 交回报告解释「没打中」是不是因为证据本来就不够多。
    pub(super) maximum_weight: u64,
    /// 完整证据空间的子集总数 = 2^(权重一类证据项数 + 权重二类证据项数)——不管这次搜索实际走到
    /// 哪一档，这个数恒报，回答「完整空间多大」（`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行
    /// 还差项②逐字要求：「先对每个 n1 算出完整证据空间的子集数」）。
    pub(super) full_space_subset_count: u64,
}

/// Q2-1 的穷举下界：按总权重从小到大枚举 Φ2 子集，**同权重内全枚举、只在权重层的边界上决定停不停**
/// （不在一档中途停——`m2-rootchoice-repair-r1-forks.md` 岔路单第 2 行还差项②：「这类故障、崩溃点的
/// 枚举不为省时间缩范围……不许靠计数上限静默截断」）。`weight_ceiling` 为 `None` 时穷举到
/// `maximum_weight`（完整证据空间的最高一档，等价于穷举完整个 2^N 子集空间）；为 `Some(w)` 时最多穷举
/// 到权重 `w`（这一档仍然全枚举完才停，不掐在档中途），调用方要在结果行里同时报
/// `full_space_subset_count`（完整空间多大）与 `highest_weight_examined`（穷举到权重几），不许只写
/// 「capped」。`include_system_configuration_slots` 打开时 Φ2 加上跑前登记 5.1 的第三类证据
/// 既有调用点都传 `false`，行为不变（见十三节顶部说明）。
pub(super) fn search_minimum_weight_that_triggers_rootback(
    node_before_rootback_probe: &SimNode,
    parameters: &MakeFilesystemParameters,
    geometry_view: &PoolGeometry,
    overwrites_after_the_rootback_probe: u64,
    weight_ceiling: Option<u64>,
    include_system_configuration_slots: bool,
) -> RootbackToleranceSearchOutcome {
    let items = evidence_items_at(
        &node_before_rootback_probe.pool,
        geometry_view,
        include_system_configuration_slots,
    );
    let weight_one_items: Vec<EvidenceItem> = items
        .iter()
        .filter(|item| {
            item.kind == EvidenceKind::RootRingSlot
                || item.kind == EvidenceKind::SystemConfigurationSlot
        })
        .cloned()
        .collect();
    let journal_record_items: Vec<EvidenceItem> = items
        .iter()
        .filter(|item| item.kind == EvidenceKind::JournalRecord)
        .cloned()
        .collect();
    let weight_one_count = u64::try_from(weight_one_items.len()).expect("证据项数装得进 u64");
    let journal_record_count =
        u64::try_from(journal_record_items.len()).expect("证据项数装得进 u64");
    let maximum_weight = weight_one_count + 2 * journal_record_count;
    // 完整证据空间的子集总数 = 2^(权重一类证据项数 + 权重二类证据项数)：每一类证据项各自「选或不选」
    // 独立自由，乘起来就是整棵幂集，与按权重分档遍历是同一个空间的两种数法（分档遍历把它切成
    // maximum_weight+1 个互斥的档，各档之和恰等于这个数——这条恒等式本身也是「没有静默漏掉子集」的
    // 一个自证）。
    let total_items = weight_one_count + journal_record_count;
    let full_space_subset_count = 1u64
        .checked_shl(u32::try_from(total_items).unwrap_or(u32::MAX))
        .unwrap_or(u64::MAX);
    // 调用方显式给了权重上限就用它（截到 `maximum_weight` 之内，超过等于不设上限）；没给时，完整
    // 空间装得进预算就穷举到底，装不进就退到 `DEFAULT_WEIGHT_CEILING_WHEN_INFEASIBLE`（权重 12）——
    // 这一步判断本身也在结果里报出来（`RootbackToleranceSearchOutcome::full_space_subset_count` 与
    // `highest_weight_examined`），不是又一层静默截断。
    let effective_ceiling = match weight_ceiling {
        Some(explicit) => explicit.min(maximum_weight),
        None if full_space_subset_count <= FEASIBLE_FULL_SEARCH_SUBSET_BUDGET => maximum_weight,
        None => DEFAULT_WEIGHT_CEILING_WHEN_INFEASIBLE.min(maximum_weight),
    };

    let mut subsets_tried = 0u64;
    let mut rootback_probe_failures = 0u64;
    let mut overwrite_failures = 0u64;
    let mut weight = 0u64;
    loop {
        // 每一档都全枚举完才检查停不停——不在 `for combination in &candidates` 中途插入计数检查，
        // 这正是「不许靠计数上限静默截断」要求的：停只停在档的边界上，且停的理由（到顶/到穷举上限）
        // 与这一档、这一次穷举到的最高权重一起报出去，不许只留一个 `capped` 布尔值。
        let candidates =
            candidate_fault_sets_of_weight(&weight_one_items, &journal_record_items, weight);
        let mut hit_count = 0u64;
        let mut first_hit_combination_labels: Vec<String> = Vec::new();
        for combination in &candidates {
            subsets_tried += 1;
            let fault_targets: Vec<(DeviceIdentity, DeviceOffsetInBytes)> = combination
                .iter()
                .flat_map(|item| item.targets.clone())
                .collect();
            let mut node = match attempt_rootback_probe_and_advance(
                node_before_rootback_probe,
                parameters,
                &fault_targets,
            ) {
                Ok(node) => node,
                Err(_) => {
                    rootback_probe_failures += 1;
                    continue;
                }
            };
            let mut overwrite_ok = true;
            for _ in 0..overwrites_after_the_rootback_probe {
                match apply_overwrite(&node, parameters) {
                    Ok(next) => node = next,
                    Err(_) => {
                        overwrite_ok = false;
                        break;
                    }
                }
            }
            if !overwrite_ok {
                overwrite_failures += 1;
                continue;
            }
            let (hit, _, _) = evaluate_rootback_predicate(&node, parameters);
            if hit {
                if hit_count == 0 {
                    first_hit_combination_labels =
                        combination.iter().map(|item| item.label.clone()).collect();
                }
                hit_count += 1;
            }
        }
        if hit_count > 0 {
            return RootbackToleranceSearchOutcome {
                weight_at_first_hit: Some(weight),
                hit_count_at_that_weight: hit_count,
                first_hit_combination_labels,
                subsets_tried,
                highest_weight_examined: weight,
                stopped_by_weight_ceiling: false,
                rootback_probe_failures,
                overwrite_failures,
                maximum_weight,
                full_space_subset_count,
            };
        }
        if weight >= effective_ceiling {
            return RootbackToleranceSearchOutcome {
                weight_at_first_hit: None,
                hit_count_at_that_weight: 0,
                first_hit_combination_labels: Vec::new(),
                subsets_tried,
                highest_weight_examined: weight,
                stopped_by_weight_ceiling: weight < maximum_weight,
                rootback_probe_failures,
                overwrite_failures,
                maximum_weight,
                full_space_subset_count,
            };
        }
        weight += 1;
    }
}

/// H2 主族（跑前登记 5.1）：mkfs → `mount_writable`（实例 1）→ 首个文件 → n1 次覆盖写 → 关闭 →
/// op2 = `mount_writable`（受注入）→ 成功则 n2 次覆盖写 → 关闭 → 判 P331。n1 ∈ {0,...,6}（登记要求
/// 的全范围（n1 取 0 到 6，不止 0 到 3）；
/// 另加 H2-R 的 `mount_rollback` 变体与 H2c 崩溃档，这两个仍没做，见交回报告「它答不了的」）。
/// `include_system_configuration_slots`：开关，打开时 Φ2 加系统配置槽这一类证据、
/// 结果行改名 `q2_1_minimum_weight_with_configuration_evidence`（不与既有六份副本已经跑过的
/// `q2_1_minimum_weight` 混在一起）；关着时逐字节复现既有调用点的行为。
/// `overwrites_before_the_rootback_probe_range`：n1 的取值区间（跑前登记要求 {0,...,6} 全范围）；
/// 它是参数、不写死 `0..=6`，为的是能把一次挂钟预算装不下的全范围拆成几次跑
/// （见 `main` 里 `q2-1-g0-configuration-evidence` 的可选命令行参数）——不改判据、不改任何一个 n1
/// 值本身怎么算，只改跑哪几个 n1。既有调用点仍传 `0..=6`，行为不变。
pub(super) fn run_rootback_tolerance_family(
    geometry: &Geometry,
    overwrites_after_the_rootback_probe: u64,
    include_system_configuration_slots: bool,
    overwrites_before_the_rootback_probe_range: std::ops::RangeInclusive<u64>,
    weight_ceiling: Option<u64>,
) {
    let parameters = parameters_for(geometry);
    let result_name = if include_system_configuration_slots {
        "q2_1_minimum_weight_with_configuration_evidence"
    } else {
        "q2_1_minimum_weight"
    };
    for overwrites_before_the_rootback_probe in overwrites_before_the_rootback_probe_range {
        let node = match bootstrap(geometry, overwrites_before_the_rootback_probe) {
            Ok(node) => node,
            Err(error) => {
                emit_result(&format!(
                    "name=q2_1_bootstrap_failed geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error:?}",
                    geometry.label
                ));
                continue;
            }
        };
        let geometry_view = match independent_geometry(&node.pool) {
            Ok(view) => view,
            Err(error) => {
                emit_result(&format!(
                    "name=q2_1_geometry_failed geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} error={error}",
                    geometry.label
                ));
                continue;
            }
        };
        let outcome = search_minimum_weight_that_triggers_rootback(
            &node,
            &parameters,
            &geometry_view,
            overwrites_after_the_rootback_probe,
            weight_ceiling,
            include_system_configuration_slots,
        );
        match outcome.weight_at_first_hit {
            Some(weight) => emit_result(&format!(
                "name={result_name} geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe={overwrites_after_the_rootback_probe} k_min={weight} hit_count_at_that_weight={} first_hit_combination={:?} subsets_tried={} full_space_subset_count={} rootback_probe_failures={} overwrite_failures={}",
                geometry.label,
                outcome.hit_count_at_that_weight,
                outcome.first_hit_combination_labels,
                outcome.subsets_tried,
                outcome.full_space_subset_count,
                outcome.rootback_probe_failures,
                outcome.overwrite_failures
            )),
            None if outcome.stopped_by_weight_ceiling => emit_result(&format!(
                "name={result_name} geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe={overwrites_after_the_rootback_probe} k_min=not_found_up_to_weight_ceiling highest_weight_examined={} maximum_weight_available={} full_space_subset_count={} subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
                geometry.label, outcome.highest_weight_examined, outcome.maximum_weight, outcome.full_space_subset_count, outcome.subsets_tried, outcome.rootback_probe_failures, outcome.overwrite_failures
            )),
            None => emit_result(&format!(
                "name={result_name} geometry={} overwrites_before_the_rootback_probe={overwrites_before_the_rootback_probe} overwrites_after_the_rootback_probe={overwrites_after_the_rootback_probe} k_min=none_found_within_full_evidence_space maximum_weight_available={} full_space_subset_count={} subsets_tried={} rootback_probe_failures={} overwrite_failures={}",
                geometry.label, outcome.maximum_weight, outcome.full_space_subset_count, outcome.subsets_tried, outcome.rootback_probe_failures, outcome.overwrite_failures
            )),
        }
    }
}

#[cfg(test)]
mod tests;

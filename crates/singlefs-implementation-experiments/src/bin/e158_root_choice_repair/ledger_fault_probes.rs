//! Q1 的探针：H1 家族的故障落点、Φ1 故障注入的单次尝试、抬底线与轨迹、树表与分配记录树的指称（候选 (b)）。

use super::*;

// ============================================================================
// 十二、Q1（岔路单第 1 行，C393）：H1 家族 + Φ1 故障注入 + 候选 (a)(b)(c) 判定。
//
// Φ1 的定位不靠幂集：对每条被抛弃的、根槽读得出的根 a，各取它的两个单元——树表单元（根记录
// 直接指着的）与分配记录树节点（树表非 0 条时是树表里 TREE_KIND_ALLOCATION 那一条目指的根；
// 树表 0 条时是根记录自己的 `allocation_record_tree_root`）——这两个单元的两条物理位置从
// `crates/singlefs-core` 的公开 API 现查得到（`RootRecord::parse_slot`、`tree_table_has_no_entries`、
// `parse_index_node`、`TreeTableEntry::parse`，见交回报告第 2.2 节），不猜偏移。
// ============================================================================

/// 一条位置条目的去重键：(设备号, 槽号)。
pub(super) fn location_key(location: &LocationEntry) -> (u32, u64) {
    (location.device.0, location.slot.0)
}

/// 从孪生（未注入）镜像上，按 (实例, txg) 取回完整的 `RootRecord`（要它的 `tree_table` /
/// `allocation_record_tree_root` 两条指针，`RootView` 没有暴露它们）。**不能**直接拿
/// `RootView::record_bytes`（`checker_image::valid_roots` 截到 466 字节）喂给 `RootRecord::parse_slot`：
/// 自证校验和是按整槽宽（`geometry.physical_block_size`，本装置是 512）算的，截短之后再解，
/// `wide_checksum_field_holds` 拿 466 当整槽宽重算，跟原校验和对不上，`parse_slot` 会稳定交回 `None`
/// （这是这一段调试打中的一个坑，交回报告点名）。改成按 `checker_image::root_slot_positions` 给的
/// (区域, 槽, 设备, 偏移) 自己重新整槽读一遍——与 `valid_roots` 内部读的是同一段字节、同一个宽度。
pub(super) fn root_record_of(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    target: TimelineRoot,
) -> Option<RootRecord> {
    let slot_bytes = usize::try_from(geometry.physical_block_size).ok()?;
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    for (_region, _slot, device_number, offset) in checker_image::root_slot_positions(geometry) {
        let identity = DeviceIdentity(device_number);
        let Some((_, device)) = devices.iter().find(|(candidate, _)| *candidate == identity) else {
            continue;
        };
        let mut buffer = vec![0u8; slot_bytes];
        if device
            .read_at(DeviceOffsetInBytes(offset), &mut buffer)
            .is_err()
        {
            continue;
        }
        if let Some(record) = RootRecord::parse_slot(&buffer, &FILESYSTEM_IDENTIFIER) {
            if (record.instance.0, record.checkpoint_txg.0) == target {
                return Some(record);
            }
        }
    }
    None
}

/// 按一条指针的两条位置条目，从**未注入**的装置去读整份节点字节（16384 字节，`NODE_BYTES`）；
/// 两份里随便一份读得出就交回，两份都读不出交 `None`。只用于发现物理位置，不参与判据本身。
pub(super) fn read_node_bytes(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    locations: &[LocationEntry; 2],
) -> Option<Vec<u8>> {
    for location in locations {
        if let Some((_, device)) = devices
            .iter()
            .find(|(identity, _)| *identity == location.device)
        {
            let mut buffer = vec![0u8; 16384];
            if device
                .read_at(location.slot.to_device_offset(), &mut buffer)
                .is_ok()
            {
                return Some(buffer);
            }
        }
    }
    None
}

/// 分配记录树节点（指称二）的两条物理位置：树表 0 条时是根记录自己的 `allocation_record_tree_root`；
/// 否则要先读树表节点、找 `TREE_KIND_ALLOCATION` 那一条目。走的是 `crates/singlefs-core::recovery`
/// 与 `records` 两个模块的公开函数，不碰任何 `pub(crate)` 内部函数（交回报告第 2.2 节的侦察结论）。
#[allow(
    clippy::ptr_arg,
    reason = "tree_table_has_no_entries 等走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn（与 crates/singlefs-core/src/mount.rs 的 placements_referenced_by_root 同一条理由）"
)]
pub(super) fn locate_allocation_record_tree(
    devices: &Vec<(DeviceIdentity, SparseBlockDevice)>,
    root: &RootRecord,
) -> Result<[LocationEntry; 2], String> {
    match tree_table_has_no_entries(devices, root) {
        Ok(true) => Ok(root.allocation_record_tree_root.locations),
        Ok(false) => {
            let node_bytes = read_node_bytes(devices, &root.tree_table.locations)
                .ok_or_else(|| "树表两份都读不出".to_string())?;
            let tree_table =
                parse_index_node(&node_bytes).map_err(|error| format!("树表解不开: {error:?}"))?;
            for entry_bytes in &tree_table.entries {
                if let Some(entry) = TreeTableEntry::parse(entry_bytes) {
                    if entry.kind == TREE_KIND_ALLOCATION {
                        return Ok(entry.root.locations);
                    }
                }
            }
            Err("树表里没有 TREE_KIND_ALLOCATION 条目".to_string())
        }
        Err(error) => Err(format!("tree_table_has_no_entries: {error:?}")),
    }
}

/// 「共享」标记（5.1 原文）：这个单元的两条位置键，是否也被某条**没被抛弃**的可读根引用
/// （树表指针或分配记录树指针任一个落在同一 (设备, 槽) 上）。H1 的基础两种 op1（`mount_writable`、
/// `mount_rollback`）都不抬 F，候选集恒等于「全部可读根」，因此这里不用另读回退下界。
#[allow(
    clippy::ptr_arg,
    reason = "转给 locate_allocation_record_tree，它要 &dyn PoolReader、要一个有大小的读者"
)]
pub(super) fn is_unit_shared(
    devices: &Vec<(DeviceIdentity, SparseBlockDevice)>,
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    exclude: TimelineRoot,
    unit_keys: &BTreeSet<(u32, u64)>,
) -> bool {
    for candidate in readable_roots_independent(pool, geometry) {
        if candidate == exclude {
            continue;
        }
        let Some(record) = root_record_of(pool, geometry, candidate) else {
            continue;
        };
        let tree_keys: BTreeSet<(u32, u64)> = record
            .tree_table
            .locations
            .iter()
            .map(location_key)
            .collect();
        if !tree_keys.is_disjoint(unit_keys) {
            return true;
        }
        if let Ok(allocation_locations) = locate_allocation_record_tree(devices, &record) {
            let allocation_keys: BTreeSet<(u32, u64)> =
                allocation_locations.iter().map(location_key).collect();
            if !allocation_keys.is_disjoint(unit_keys) {
                return true;
            }
        }
    }
    false
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LedgerAspect {
    TreeTable,
    AllocationRecordTree,
}

impl LedgerAspect {
    pub(super) fn label(self) -> &'static str {
        match self {
            LedgerAspect::TreeTable => "tree_table",
            LedgerAspect::AllocationRecordTree => "allocation_record_tree",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FaultSeverity {
    Empty,
    Both,
    Disk0,
    Disk1,
}

impl FaultSeverity {
    pub(super) fn label(self) -> &'static str {
        match self {
            FaultSeverity::Empty => "empty",
            FaultSeverity::Both => "both_copies",
            FaultSeverity::Disk0 => "disk0_only",
            FaultSeverity::Disk1 => "disk1_only",
        }
    }

    /// 故障数（5.1 表：两份都读失败 = 2 个故障，只一份 = 1 个，空集 = 0）。
    pub(super) fn fault_count(self) -> u64 {
        match self {
            FaultSeverity::Empty => 0,
            FaultSeverity::Both => 2,
            FaultSeverity::Disk0 | FaultSeverity::Disk1 => 1,
        }
    }
}

/// 按严重度把一个单元的两条位置条目变成注入目标（(设备, 偏移) 表）。
pub(super) fn fault_targets_for(
    locations: &[LocationEntry; 2],
    severity: FaultSeverity,
) -> Vec<(DeviceIdentity, DeviceOffsetInBytes)> {
    match severity {
        FaultSeverity::Empty => Vec::new(),
        FaultSeverity::Both => locations
            .iter()
            .map(|location| (location.device, location.slot.to_device_offset()))
            .collect(),
        FaultSeverity::Disk0 => locations
            .iter()
            .filter(|location| location.device == DeviceIdentity(0))
            .map(|location| (location.device, location.slot.to_device_offset()))
            .collect(),
        FaultSeverity::Disk1 => locations
            .iter()
            .filter(|location| location.device == DeviceIdentity(1))
            .map(|location| (location.device, location.slot.to_device_offset()))
            .collect(),
    }
}

#[derive(Clone, Debug)]
#[allow(
    clippy::enum_variant_names,
    reason = "三个变体都是「哪一种挂载/抬 F 操作」，共享 Mount 前缀是命名准确，不是同一个词重复"
)]
pub(super) enum FaultedOperationKind {
    MountWritable,
    MountRollbackTo(TimelineRoot),
    /// op1 第三种变体（跑前登记 5.1 H1 行）：先不注入地做一次 `mount_writable`（只为打开会话，
    /// 不是受注入的一步），再把这个目标 floor 值喂给受注入的 `raise_rollback_floor`。
    MountWritableThenRaiseFloorTo(CheckpointTxg),
}

impl FaultedOperationKind {
    pub(super) fn label(&self) -> String {
        match self {
            FaultedOperationKind::MountWritable => "mount_writable".to_string(),
            FaultedOperationKind::MountRollbackTo(target) => format!("mount_rollback({target:?})"),
            FaultedOperationKind::MountWritableThenRaiseFloorTo(floor) => {
                format!("mount_writable_then_raise_rollback_floor({floor:?})")
            }
        }
    }
}

/// 一次 op1 尝试的结局：`Ok` 时交回 `MountOutput::abandoned_roots_unreadable`；`Err` 时交回错误的
/// `Debug` 首词（跨臂只用字符串比较——A1 副本的 `MountError::AbandonedRootLedgerUnreadable` 在
/// 「今天」那份 `crates/` 里根本不存在这个变体，装置源码两边共用一份，不能直接 `match` 枚举名）。
/// 另外把这次调用设备一层写的字节数按盘记下（Q1-4：候选 (a) 被拒那次必须是 0）。
pub(super) struct FaultedOperationAttempt {
    pub(super) abandoned_roots_unreadable: Option<u64>,
    pub(super) error_debug: Option<String>,
    pub(super) error_member: Option<String>,
    pub(super) device_write_bytes: BTreeMap<u32, u64>,
}

/// 从 `MountError` 的 `Debug` 输出里取枚举变体名（首个 `(`、`{` 或空格之前的那一段）。跨臂只靠
/// 字符串比较：A1 副本新增的 `AbandonedRootLedgerUnreadable` 变体在「今天」那份 `crates/` 里不存在，
/// 装置源码两边共用一份，不能写 `match MountError::AbandonedRootLedgerUnreadable { .. }`（编不过今天那份）。
pub(super) fn error_member_of_debug(debug: &str) -> String {
    debug
        .split(['(', '{', ' '])
        .next()
        .unwrap_or(debug)
        .to_string()
}

pub(super) fn attempt_faulted_operation(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    fixed_geometry: FixedGeometry,
    kind: &FaultedOperationKind,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
) -> FaultedOperationAttempt {
    // op1 第三种变体：唯一一种需要在受注入的那一步**之前**先做一次完全不注入的
    // `mount_writable`（打开会话）的 op1，装置结构与另外两种不同，另开一个函数、在这里提前分派。
    if let FaultedOperationKind::MountWritableThenRaiseFloorTo(new_floor) = kind {
        return attempt_mount_writable_then_raise_floor(
            node,
            parameters,
            fixed_geometry,
            *new_floor,
            fault_targets,
        );
    }
    let plain = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mut plans: Vec<(DeviceIdentity, SharedFaultPlan)> = Vec::new();
    let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        Vec::new();
    for (identity, device) in plain {
        let plan = SharedFaultPlan::unarmed(fixed_geometry);
        if let Some((_, offset)) = fault_targets
            .iter()
            .find(|(target_device, _)| *target_device == identity)
        {
            plan.arm(FaultSchedule {
                fault: InjectedFault::ReadFails,
                device: FaultDeviceSelector::OnlyDevice(identity),
                placement: FaultPlacement::OffsetExactly(*offset),
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
            });
        }
        plans.push((identity, plan.clone()));
        wrapped.push((
            identity,
            FaultInjectingBlockDevice::new(identity, device, plan),
        ));
    }
    let outcome: Result<u64, String> = match kind {
        FaultedOperationKind::MountWritable => mount_writable(parameters, &mut wrapped)
            .map(|mounted| mounted.output.abandoned_roots_unreadable)
            .map_err(|error| format!("{error:?}")),
        FaultedOperationKind::MountRollbackTo(target) => mount_rollback(
            parameters,
            &mut wrapped,
            RollbackTarget {
                instance: InstanceGeneration(target.0),
                checkpoint_txg: CheckpointTxg(target.1),
            },
            ShadowLedger::On,
        )
        .map(|mounted| mounted.output.abandoned_roots_unreadable),
        FaultedOperationKind::MountWritableThenRaiseFloorTo(_) => {
            unreachable!("MountWritableThenRaiseFloorTo 在函数开头已经提前分派、提前 return 过了")
        }
    };
    let mut device_write_bytes = BTreeMap::new();
    for (identity, plan) in &plans {
        device_write_bytes.insert(identity.0, plan.counts_of_device(*identity).written_bytes);
    }
    match outcome {
        Ok(count) => FaultedOperationAttempt {
            abandoned_roots_unreadable: Some(count),
            error_debug: None,
            error_member: None,
            device_write_bytes,
        },
        Err(debug) => {
            let member = error_member_of_debug(&debug);
            FaultedOperationAttempt {
                abandoned_roots_unreadable: None,
                error_debug: Some(debug),
                error_member: Some(member),
                device_write_bytes,
            }
        }
    }
}

/// op1 第三种变体（跑前登记 H1 行「先不注入地 `mount_writable` 再 `raise_rollback_floor` 到
/// [F+1, 上限] 里每一个值」）：floor 目标枚举成 (F, 上限] 半开区间——`current_floor` 本身不许再抬
/// （抬到自己等于没抬），`ceiling` 一定要能抬到。`ceiling <= current_floor` 时天然给出空区间
/// （Rust 的 `..=` 起点大于终点时不产出任何元素），不用另写一条判空分支。
fn floor_targets_between(
    current_floor: CheckpointTxg,
    ceiling: CheckpointTxg,
) -> Vec<CheckpointTxg> {
    ((current_floor.0 + 1)..=ceiling.0)
        .map(CheckpointTxg)
        .collect()
}

/// op1 第三种变体的目标 floor 枚举：先不注入地做一次 `mount_writable`（试探性，用完即弃，只为量出
/// 这一步的 F 与上限——它本身不是受注入的一步），从它交回的 `current.root.rollback_floor` 起算 F，
/// 再用 `crates/` 已有的 `rollback_floor_ceiling`（与 `raise_rollback_floor` 内部用的是同一个函数）
/// 算出上限（读坏、重读仍坏就拒的槽与抬 F 那一串同一个口径，`ring_slots_whose_reread_still_bad_refuses_the_ceiling_by`：
/// 被抛弃根的槽读坏不拒、不进上限）。交回空 `Vec`：探测步骤失败（挂载不成功、没有文件、系统配置或实例表读不出、
/// `rollback_floor_ceiling` 报错），或 F 已经等于上限（没有余量可抬）——两种都不是「该测到却漏了」，
/// 是这条历史在这一格没有 op1 第三种变体可跑，调用方按 `raise_floor_history_nodes_without_room`
/// 单独计一次，不归进任何触发计数。
pub(super) fn raise_floor_targets_for(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
) -> Vec<CheckpointTxg> {
    let mut probe_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let Ok(mounted) = mount_writable(parameters, &mut probe_devices) else {
        return Vec::new();
    };
    let PoolVersion::WithFile(current) = mounted.current else {
        return Vec::new();
    };
    let Ok(system_configuration) = choose_system_configuration(&probe_devices) else {
        return Vec::new();
    };
    let Ok(table) = instance_table_chain_of_root(&probe_devices, &current.root) else {
        return Vec::new();
    };
    let current_floor = current.root.rollback_floor;
    let Ok(ceiling) = rollback_floor_ceiling(
        &probe_devices,
        &system_configuration,
        current_floor,
        &table.records,
        &ring_slots_whose_reread_still_bad_refuses_the_ceiling_by(&mounted.allocator),
    ) else {
        return Vec::new();
    };
    floor_targets_between(current_floor, ceiling)
}

/// op1 第三种变体的「试一次看结局，用完即弃」实现：第一步（打开会话）**不注入**——`raise_rollback_
/// floor` 要 `&mut PoolAllocator`/`&mut TransactionOutput`（一个开着的会话），而这里进来的 `node`
/// 已经「关闭」（`session: None`），受注入的一步只是 `raise_rollback_floor` 本身（跑前登记 5.1 H1
/// 行「先不注入地 `mount_writable` 再 `raise_rollback_floor`」）。**不追加进 `node.timeline`**：
/// `raise_rollback_floor` 交回的 `RaisedFloor::abandoned_roots_unreadable` 是从盘上现读实例表 +
/// 现行根的候选集算出来的（`isolate_slots_referenced_only_by_abandoned_roots`，与
/// `mount_writable`/`mount_rollback` 那两种 op1 共用同一条计数管道，`crates/singlefs-core/src/
/// mount.rs` 里的同名函数），不依赖装置自己的内存时间线；与 `MountRollbackTo` 分支同理——那个分支
/// 交回的 `mounted.output.warm_up_publishes` 等中间根同样只用来读它的最终 `abandoned_roots_
/// unreadable` 字段，从不追加进任何时间线（`attempt_faulted_operation` 整体是「试一次看结局，用完
/// 即弃」，不产出可继续使用的 `SimNode`）。两步用的字节不共享（`devices_from_pool` 每次都深拷贝，
/// 第一步之后的 `probe_devices` 直接喂给第二步的包装层，不再回读 `node.pool`），故障只出现在第二
/// 步的 `wrapped`。
fn attempt_mount_writable_then_raise_floor(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
    fixed_geometry: FixedGeometry,
    new_floor: CheckpointTxg,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
) -> FaultedOperationAttempt {
    let mut probe_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mounted = match mount_writable(parameters, &mut probe_devices) {
        Ok(mounted) => mounted,
        Err(error) => {
            return FaultedOperationAttempt {
                abandoned_roots_unreadable: None,
                error_debug: Some(format!("先不注入地 mount_writable 失败：{error:?}")),
                error_member: Some("PrecedingUnfaultedMountWritableFailed".to_string()),
                device_write_bytes: BTreeMap::new(),
            };
        }
    };
    let PoolVersion::WithFile(current) = mounted.current else {
        return FaultedOperationAttempt {
            abandoned_roots_unreadable: None,
            error_debug: Some("先不注入地 mount_writable 交回的现行版本没有文件".to_string()),
            error_member: Some("PrecedingUnfaultedMountWritableWithoutFile".to_string()),
            device_write_bytes: BTreeMap::new(),
        };
    };
    let mut current = std::sync::Arc::unwrap_or_clone(current);
    let mut allocator = mounted.allocator;
    let mut plans: Vec<(DeviceIdentity, SharedFaultPlan)> = Vec::new();
    let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        Vec::new();
    for (identity, device) in probe_devices {
        let plan = SharedFaultPlan::unarmed(fixed_geometry);
        if let Some((_, offset)) = fault_targets
            .iter()
            .find(|(target_device, _)| *target_device == identity)
        {
            plan.arm(FaultSchedule {
                fault: InjectedFault::ReadFails,
                device: FaultDeviceSelector::OnlyDevice(identity),
                placement: FaultPlacement::OffsetExactly(*offset),
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
            });
        }
        plans.push((identity, plan.clone()));
        wrapped.push((
            identity,
            FaultInjectingBlockDevice::new(identity, device, plan),
        ));
    }
    let raised = raise_rollback_floor(
        parameters,
        &mut wrapped,
        &mut allocator,
        &mut current,
        new_floor,
        ShadowLedger::On,
    );
    let mut device_write_bytes = BTreeMap::new();
    for (identity, plan) in &plans {
        device_write_bytes.insert(identity.0, plan.counts_of_device(*identity).written_bytes);
    }
    match raised {
        Ok(raised) => FaultedOperationAttempt {
            abandoned_roots_unreadable: Some(raised.abandoned_roots_unreadable),
            error_debug: None,
            error_member: None,
            device_write_bytes,
        },
        Err(error) => {
            let debug = format!("{error:?}");
            let member = error_member_of_debug(&debug);
            FaultedOperationAttempt {
                abandoned_roots_unreadable: None,
                error_debug: Some(debug),
                error_member: Some(member),
                device_write_bytes,
            }
        }
    }
}

/// （`m2-rootchoice-repair-r1-forks.md` 岔路单第 1 行还差项④）：op1 起的多次挂载轨迹——
/// 第一节读法写死表「持续」＝从 op1 这次调用起到这段历史结束都有效，「瞬时」＝只在 op1 那次调用期间
/// 有效；第八节 8.1「abandoned_roots_unreadable，按 op1 起的每一次挂载」要的轨迹正是这个函数产出的。
/// **与 `attempt_faulted_operation` 的关键差别**：那个函数每次都从 `node.pool` 重新
/// `devices_from_pool`，故障目标之外的写不回原池（`SparseDevice` 是普通值类型、`.clone()` 深拷贝，
/// 不共享底层字节，这是刻意的「试一次看结局」语义，见它的用法）；这里要的是「op1 落盘之后接着挂」，
/// 所以每一步显式把 `FaultInjectingBlockDevice::inner().image` 取出来拼回一个新的 `MemoryPool`，
/// 喂给下一步——**这是这个函数与 `attempt_faulted_operation` 的唯一结构性差别，其余装故障、判结局的
/// 写法逐字照抄，不引入第二套注入逻辑**。`persistent = true` 时每一步都用同一组 `fault_targets`
/// （对应「持续」）；`persistent = false` 时只有第一步（op1 自己）带故障，之后的步不装任何故障
/// （对应「瞬时」，且与「瞽时」的读法一致：故障在 op1 那次调用返回后即撤）。某一步交回 `Err` 就停在
/// 那一步，`results` 里这一步与之后的步都记 `None`（挂载失败之后没有新的池状态可以接着挂，不能凭空
/// 编一个「本该」发生的计数）。
pub(super) fn mount_writable_trajectory(
    starting_pool: &MemoryPool,
    parameters: &MakeFilesystemParameters,
    fixed_geometry: FixedGeometry,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
    persistent: bool,
    steps: usize,
) -> Vec<Option<u64>> {
    let mut pool = starting_pool.clone();
    let mut results = Vec::new();
    for step in 0..steps {
        let active_targets: &[(DeviceIdentity, DeviceOffsetInBytes)] = if step == 0 || persistent {
            fault_targets
        } else {
            &[]
        };
        let plain = devices_from_pool(&pool, IMAGE_BYTES);
        let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
            Vec::new();
        for (identity, device) in plain {
            let plan = SharedFaultPlan::unarmed(fixed_geometry);
            if let Some((_, offset)) = active_targets
                .iter()
                .find(|(target_device, _)| *target_device == identity)
            {
                plan.arm(FaultSchedule {
                    fault: InjectedFault::ReadFails,
                    device: FaultDeviceSelector::OnlyDevice(identity),
                    placement: FaultPlacement::OffsetExactly(*offset),
                    counting: FaultCounting::AcrossThePool,
                    occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
                });
            }
            wrapped.push((
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan),
            ));
        }
        match mount_writable(parameters, &mut wrapped) {
            Ok(mounted) => {
                results.push(Some(mounted.output.abandoned_roots_unreadable));
                let next_devices: BTreeMap<DeviceIdentity, SparseDevice> = wrapped
                    .iter()
                    .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
                    .collect();
                pool = MemoryPool {
                    devices: next_devices,
                    device_size_in_bytes: IMAGE_BYTES,
                };
            }
            Err(_) => {
                results.push(None);
                break;
            }
        }
    }
    while results.len() < steps {
        results.push(None);
    }
    results
}

/// H1 家族里的一个「op1 之前」节点：mkfs → `mount_writable` → 首个文件 →
/// `overwrites_before_rollback` 次覆盖写 → 关闭 → `mount_rollback` 到 `rollback_target` →
/// `overwrites_after_rollback` 次覆盖写 → 关闭。op1 本身（受注入的一步）不在这个结构体里，
/// 由调用方按 `FaultedOperationKind` 逐个尝试。
pub(super) struct LedgerFaultHistoryNode {
    pub(super) node: SimNode,
    pub(super) overwrites_before_rollback: u64,
    pub(super) rollback_target: TimelineRoot,
    pub(super) overwrites_after_rollback: u64,
}

/// H1 全族逐一枚举（不抽样）：`overwrites_before_rollback` ∈ {1,2,3}；`rollback_target` 取那一刻每一条
/// 可读根（按 `BTreeSet` 升序，固定次序）；`overwrites_after_rollback` ∈ {0,1,2}。回退到某个候选被拒的
/// 分支到此为止，计入 `rejected_rollback_attempts`（不是 H1 节点）。
pub(super) struct LedgerFaultHistoryFamily {
    pub(super) nodes: Vec<LedgerFaultHistoryNode>,
    pub(super) rejected_rollback_attempts: u64,
}

pub(super) fn ledger_fault_history_family(geometry: &Geometry) -> LedgerFaultHistoryFamily {
    let parameters = parameters_for(geometry);
    let mut nodes = Vec::new();
    let mut rejected_rollback_attempts = 0u64;
    for overwrites_before_rollback in [1u64, 2, 3] {
        let Ok(base) = bootstrap(geometry, overwrites_before_rollback) else {
            continue;
        };
        let Ok(base_geometry) = independent_geometry(&base.pool) else {
            continue;
        };
        let candidates: Vec<TimelineRoot> = readable_roots_independent(&base.pool, &base_geometry)
            .into_iter()
            .collect();
        for rollback_target in candidates {
            let target = RollbackTarget {
                instance: InstanceGeneration(rollback_target.0),
                checkpoint_txg: CheckpointTxg(rollback_target.1),
            };
            match apply_mount_rollback(&base, target, &parameters) {
                Ok(after_rollback) => {
                    let mut current = after_rollback;
                    for overwrites_after_rollback in [0u64, 1, 2] {
                        nodes.push(LedgerFaultHistoryNode {
                            node: current.clone(),
                            overwrites_before_rollback,
                            rollback_target,
                            overwrites_after_rollback,
                        });
                        if overwrites_after_rollback < 2 {
                            match apply_overwrite(&current, &parameters) {
                                Ok(next) => current = next,
                                Err(_) => break,
                            }
                        }
                    }
                }
                Err(_) => rejected_rollback_attempts += 1,
            }
        }
    }
    LedgerFaultHistoryFamily {
        nodes,
        rejected_rollback_attempts,
    }
}

/// 分配记录树节点带同一组故障走一遍（候选 (b) 分配记录树指称，Q1-2）：交回 `(设备, 槽, 已释放)` 的原始
/// 列表；调用方按 `!released` 过滤成「未释放落点集合」再比对。
pub(super) fn walk_allocation_records_with_fault(
    node: &SimNode,
    fixed_geometry: FixedGeometry,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
    root: &RootRecord,
) -> Result<Vec<(u32, u64, bool)>, String> {
    let plain = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        Vec::new();
    for (identity, device) in plain {
        let plan = SharedFaultPlan::unarmed(fixed_geometry);
        if let Some((_, offset)) = fault_targets
            .iter()
            .find(|(target_device, _)| *target_device == identity)
        {
            plan.arm(FaultSchedule {
                fault: InjectedFault::ReadFails,
                device: FaultDeviceSelector::OnlyDevice(identity),
                placement: FaultPlacement::OffsetExactly(*offset),
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
            });
        }
        wrapped.push((
            identity,
            FaultInjectingBlockDevice::new(identity, device, plan),
        ));
    }
    allocation_records_under_root(&wrapped, root)
        .map(|records| {
            records
                .into_iter()
                .map(|record| (record.device.0, record.slot.0, record.is_released))
                .collect()
        })
        .map_err(|error| format!("{error:?}"))
}

/// 树表指称候选 (b)「有副本」：在孪生（未注入）镜像上，别的可读根的树表节点字节里有没有一份与
/// `target_bytes` 逐字节相同的。「crates 有路」（调 `rebuild_version`/`replay_journal`）这一子分支
/// 这一段没有实现，见交回报告——「有副本」为真已经够支持 (b) 在这一格可重建，「有副本」为假时
/// 报「无来源（只测过有副本这一支）」，不等价于「crates 也无路」。
pub(super) fn tree_table_has_copy_elsewhere(
    plain_devices: &[(DeviceIdentity, SparseBlockDevice)],
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    exclude: TimelineRoot,
    target_bytes: &[u8],
) -> bool {
    for candidate in readable_roots_independent(pool, geometry) {
        if candidate == exclude {
            continue;
        }
        if let Some(record) = root_record_of(pool, geometry, candidate) {
            if let Some(bytes) = read_node_bytes(plain_devices, &record.tree_table.locations) {
                if bytes == target_bytes {
                    return true;
                }
            }
        }
    }
    false
}

/// 找被抛弃根在同一实例上、checkpoint_txg 更小、且今天仍读得出的那一条根——候选 (b) 树表指称
/// 「crates 有路」子分支要从它出发（跑前登记 5.2 ②）。取 checkpoint_txg 最大的那一条（最近的祖先）。
fn nearest_older_readable_root_on_same_instance(
    pool: &MemoryPool,
    geometry: &PoolGeometry,
    exclude: TimelineRoot,
    target: &RootRecord,
) -> Option<(TimelineRoot, RootRecord)> {
    let mut best: Option<(TimelineRoot, RootRecord)> = None;
    for candidate in readable_roots_independent(pool, geometry) {
        if candidate == exclude {
            continue;
        }
        let Some(record) = root_record_of(pool, geometry, candidate) else {
            continue;
        };
        if record.instance != target.instance || record.checkpoint_txg >= target.checkpoint_txg {
            continue;
        }
        let take_it = match &best {
            None => true,
            Some((_, current_best)) => record.checkpoint_txg > current_best.checkpoint_txg,
        };
        if take_it {
            best = Some((candidate, record));
        }
    }
    best
}

/// 候选 (b) 树表指称「crates 有路」子分支的结局（跑前登记 5.2 ②）。**这一支给出的是指针级别（位置 +
/// 校验和）的复算，不是节点内容的 16 KiB 字节本身**——要拿到字节仍要照这个指针再读一次两份物理拷贝，
/// 而那正是这一格要绕开的读；这个限定在交回报告里写清楚，不写成「crates 有路 = 给得出字节」。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TreeTableCratesHasAPathOutcome {
    /// 没有更旧的可读同实例根：这条路径的前提不成立。
    NoOlderReadableRootOnTimeline,
    /// 装置自己解不出系统配置（独立扫描 journal 记录要用它），走不下去。
    CannotDecodeSystemConfiguration,
    /// replay 交回的根没有推进到目标的 checkpoint_txg：链断在中途（可能正是因为验证撞上了被注入的物理位置）。
    DidNotReachTarget,
    /// replay 推进到了目标的 checkpoint_txg，但交回的 `tree_table` 指针与目标不同——crates 这条路给出的答案本身不对。
    ReachedTargetButPointerDiffers,
    /// replay 推进到了目标的 checkpoint_txg，且交回的 `tree_table` 指针（位置 + 校验和）与目标逐字段相同。
    ReachedTargetWithMatchingPointer,
}

impl TreeTableCratesHasAPathOutcome {
    pub(super) fn label(self) -> &'static str {
        match self {
            TreeTableCratesHasAPathOutcome::NoOlderReadableRootOnTimeline => {
                "no_older_readable_root_on_timeline"
            }
            TreeTableCratesHasAPathOutcome::CannotDecodeSystemConfiguration => {
                "cannot_decode_system_configuration"
            }
            TreeTableCratesHasAPathOutcome::DidNotReachTarget => "did_not_reach_target",
            TreeTableCratesHasAPathOutcome::ReachedTargetButPointerDiffers => {
                "reached_target_but_pointer_differs"
            }
            TreeTableCratesHasAPathOutcome::ReachedTargetWithMatchingPointer => {
                "reached_target_with_matching_pointer"
            }
        }
    }
}

/// 候选 (b) 树表指称「crates 有路」子分支（跑前登记 5.2 ②）：从同一实例上更旧的可读根出发，调
/// `crates/` 已有的 `replay_journal`，看它能不能不读目标那两份物理拷贝就交回同一个 `tree_table` 指针。
/// `verify_named_units` 两种都跑：`true` 走今天挂载真用的口径（逐项验证点名单元，可能因此撞上同一个
/// 被注入的物理位置）；`false` 只信记录自带的新根段字段、不去验证任何单元字节。两者的差异本身就是
/// 「这条路独不独立于那次被挡住的读」这句话的答案。
pub(super) fn tree_table_crates_has_a_path(
    node: &SimNode,
    fixed_geometry: FixedGeometry,
    fault_targets: &[(DeviceIdentity, DeviceOffsetInBytes)],
    geometry: &PoolGeometry,
    exclude: TimelineRoot,
    target: &RootRecord,
    verify_named_units: bool,
) -> TreeTableCratesHasAPathOutcome {
    let Some((_, older_root)) =
        nearest_older_readable_root_on_same_instance(&node.pool, geometry, exclude, target)
    else {
        return TreeTableCratesHasAPathOutcome::NoOlderReadableRootOnTimeline;
    };
    let plain_devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let Ok(system_configuration) = choose_system_configuration(&plain_devices) else {
        return TreeTableCratesHasAPathOutcome::CannotDecodeSystemConfiguration;
    };
    // 记录本身不在这一格的故障目标里（Φ1 只挡树表或分配记录树那一个单元），按不注入的孪生镜像扫出全量——
    // 独立扫描 journal 不是这一支要考的东西，考的是 replay 拿到记录之后还要不要碰目标那份物理拷贝。
    let records = scan_journal(&plain_devices, &system_configuration);
    let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        Vec::new();
    for (identity, device) in devices_from_pool(&node.pool, IMAGE_BYTES) {
        let plan = SharedFaultPlan::unarmed(fixed_geometry);
        if let Some((_, offset)) = fault_targets
            .iter()
            .find(|(target_device, _)| *target_device == identity)
        {
            plan.arm(FaultSchedule {
                fault: InjectedFault::ReadFails,
                device: FaultDeviceSelector::OnlyDevice(identity),
                placement: FaultPlacement::OffsetExactly(*offset),
                counting: FaultCounting::AcrossThePool,
                occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
            });
        }
        wrapped.push((
            identity,
            FaultInjectingBlockDevice::new(identity, device, plan),
        ));
    }
    let (_report, replayed_root) = replay_journal(
        &wrapped,
        &older_root,
        geometry.journal_ring_bytes,
        &records,
        if verify_named_units {
            NamedUnitVerification::Verify
        } else {
            NamedUnitVerification::Skip
        },
    )
    .expect("装置里的记录都由写者写出，所选根那次发布只有一条带末条标志：锚点认得出");
    if replayed_root.instance != target.instance
        || replayed_root.checkpoint_txg != target.checkpoint_txg
    {
        return TreeTableCratesHasAPathOutcome::DidNotReachTarget;
    }
    if replayed_root.tree_table == target.tree_table {
        TreeTableCratesHasAPathOutcome::ReachedTargetWithMatchingPointer
    } else {
        TreeTableCratesHasAPathOutcome::ReachedTargetButPointerDiffers
    }
}

/// 候选 (b) 分配记录树指称「从内容树反推占用集合」这一种操作化（跑前登记 5.2 ②，修订项 5
/// 提出的思路）：不重读被注入的那一个分配记录树节点本身，改走中央映射树——它是这一版的逻辑到物理的
/// 权威索引（D1（数据可移动性 / 反向索引）），每条条目携带两条独立于分配记录树的位置条目。对第一版这种
/// 没有快照、只有一个活头的简单历史，中央映射树里出现过的落点集合就应当等于「未释放」的落点集合。
/// **射程**：只走了中央映射树一条内容树（`crates::mounted_read::open_pool_for_read` 走读 extent / inode
/// 树是为了服务文件读，与「占用集合」这个问题不是同一件事——占用靠中央映射，不靠 extent/inode）；
/// 有快照、多版本共享落点的情形没有覆盖（第一版这批历史都不产生快照，见交回报告）。
///
/// **只读一层、把根当叶解**——中央映射树条目数一旦
/// 超过单节点叶容量（`transaction.rs` 里「中央映射树叶 294」那一句）就会长成多层，那时根节点是
/// 内部节点（`level > 0`），装着的是子节点指针、不是 `parse_mapping_entry` 认得的叶条目格式。
/// 这里显式核 `level == 0` 再往下解，不靠「宽度不够、`parse_mapping_entry` 自然读不出」这种隐式失败
/// 兜底——内部条目宽约 114 字节（`transaction.rs` 同一行「内部 143」按 16384 / 143 反推），比
/// `MAPPING_ENTRY_BYTES`=55 宽，`parse_mapping_entry` 的长度检查拦不住它，会把内部指针字节当叶
/// 条目误解出一对看似合法的 `LocationEntry`——这正是「不许静默算错」要挡的那种失败，`level` 检查把
/// 它变成一条会报错、不会算错的路。E158 这批历史全部只写一个文件、只有一个逻辑映射 key（跨 n1/m/σ
/// 的全部覆盖写都在原地更新同一条条目，不新增条目），条目数最多到 1（见交回报告的现场核对），远低于
/// 294，这批历史至今不会撞上多层——但装置不该靠这条事实免检，`level` 一变就该显式报错，不是碰巧躲过。
///
/// # Errors
/// 中央映射树根两份都读不出、解不开，或根节点 `level != 0`（长成了多层，这个函数还不会整棵读）。
pub(super) fn allocation_record_tree_reachable_placements_via_central_mapping(
    plain_devices: &[(DeviceIdentity, SparseBlockDevice)],
    root: &RootRecord,
) -> Result<BTreeSet<(u32, u64)>, String> {
    let mapping_root_bytes = read_node_bytes(plain_devices, &root.mapping_root.locations)
        .ok_or_else(|| "中央映射树根两份都读不出".to_string())?;
    let mapping_root = parse_index_node(&mapping_root_bytes)
        .map_err(|error| format!("中央映射树根解不开: {error:?}"))?;
    if mapping_root.level != 0 {
        return Err(format!(
            "中央映射树根不是叶（level={}，entries={}）：树已经长成多层，这个函数只会整棵读单叶版本，不认内部节点条目",
            mapping_root.level,
            mapping_root.entries.len()
        ));
    }
    let mut placements = BTreeSet::new();
    for entry_bytes in &mapping_root.entries {
        let (_key, locations) =
            parse_mapping_entry(entry_bytes).ok_or_else(|| "中央映射条目解不开".to_string())?;
        for location in locations {
            placements.insert((location.device.0, location.slot.0));
        }
    }
    Ok(placements)
}

/// 候选 (b) 分配记录树指称「从内容树反推占用集合」走全（跑前登记 5.2 ②「它答不了的」
/// 缺口①）：在上一个函数（只走中央映射树这一条内容树）的基础上，把「码 2」结构节点自己的落点也并进
/// 占用集合——`AllocationRecord`（`crates/singlefs-core/src/allocator.rs` 的 `pub struct AllocationRecord`）记的是**任意
/// 已分配的跨度**，不只是中央映射树指向的数据单元；树表自己、树表里 `TREE_KIND_INODE` /
/// `TREE_KIND_EXTENT` / `TREE_KIND_ACCOUNTING` 三种子树各自的根节点、以及中央映射树**自己**的根节点
/// （区别于它指向的数据单元），都是「码 2 节点取最低空槽」（`allocator.rs` 文件头）分配出来的、理应
/// 出现在「未释放」集合里的落点。`TREE_KIND_ALLOCATION` 不并进来——那是分配记录树自己，并进来是把
/// 要验证的账当成账的输入，循环论证。**这仍不是穷举**：`TREE_KIND_LIVELIST` / `_SPARSE_SIDE_TABLE` /
/// `_DEADLIST` 三种树表里可能存在的种类没有并入（第一版这批历史是否会产生它们、要不要算，留给下一段）。
pub(super) fn allocation_record_tree_reachable_placements_via_all_structural_trees(
    plain_devices: &[(DeviceIdentity, SparseBlockDevice)],
    root: &RootRecord,
) -> Result<BTreeSet<(u32, u64)>, String> {
    let mut placements =
        allocation_record_tree_reachable_placements_via_central_mapping(plain_devices, root)?;
    for location in &root.mapping_root.locations {
        placements.insert((location.device.0, location.slot.0));
    }
    for location in &root.tree_table.locations {
        placements.insert((location.device.0, location.slot.0));
    }
    let tree_table_bytes = read_node_bytes(plain_devices, &root.tree_table.locations)
        .ok_or_else(|| "树表两份都读不出".to_string())?;
    let tree_table =
        parse_index_node(&tree_table_bytes).map_err(|error| format!("树表解不开: {error:?}"))?;
    for entry_bytes in &tree_table.entries {
        let Some(entry) = TreeTableEntry::parse(entry_bytes) else {
            continue;
        };
        if matches!(
            entry.kind,
            TREE_KIND_INODE | TREE_KIND_EXTENT | TREE_KIND_ACCOUNTING
        ) {
            for location in &entry.root.locations {
                placements.insert((location.device.0, location.slot.0));
            }
        }
    }
    Ok(placements)
}

/// （跑前登记 5.2 ②「它答不了的」缺口①，`m2-rootchoice-repair-r1-forks.md` #1「被抛弃根的账读不出时，「修复」指什么动作」还差项）：
/// `subset_of_pristine` 那些格里，`pristine − 走全` 差集的每一个 (设备, 槽) 按结构归类——先按跑前登记
/// 5.2 表原文点名的五类结构逐项核对（分配记录树自己的节点；树表登记的 `LIVELIST` / `SPARSE_SIDE_TABLE`
/// / `DEADLIST` 根，`transaction.rs` 的 `table_entry` 调用点显示这三类树表条目的根**恒为
/// `NodePointer::empty_root()`**——这批历史、乃至今天全仓任何一条写路径都不产生这三类树的物理节点，
/// 所以这一分支理论上永远查不到命中，仍按登记要求逐项核，核不到就如实报「查无归属」）；查无归属时按
/// `AllocationRecord.span_slots` 分「码 2 结构节点（1 槽，`allocator.rs` 顶注『码 2 节点取最低空槽』）」
/// 与「码 3 数据容器（2 槽，`DATA_UNIT_BYTES`=32768=2×`SLOT_BYTES`）」两档报告——span 是分配器自己写死
/// 的界，不是这一段猜的。
pub(super) fn classify_diff_pair(
    plain_devices: &[(DeviceIdentity, SparseBlockDevice)],
    root: &RootRecord,
    allocation_record_tree_locations: Option<[LocationEntry; 2]>,
    records: &[AllocationRecord],
    pair: (u32, u64),
) -> &'static str {
    // 根因排查②的实测结果：171 格的差集**逐字节等于** `root.instance_table.locations`
    // （实例表自己的物理节点，`RootRecord` 第 23 行，与 `tree_table` / `mapping_root` 是三条并列的独立
    // 指针，不挂在树表的七条条目里，`allocation_record_tree_reachable_placements_via_all_structural_trees`
    // 从未读过它）——`crates/singlefs-core/src/make_filesystem.rs` 里
    // `INSTANCE_TABLE_SLOT = SlotNumber(UNIT_AREA_START_SLOT)`，`crates/singlefs-format/src/lib.rs`
    // 第 231 行 `UNIT_AREA_START_SLOT: u64 = 50176`，与实测 `device=0/1 slot=50176` 逐位吻合；
    // `mark_format_time_units` 给它的 `Placement { slot: INSTANCE_TABLE_SLOT, span: 2 }` 与实测
    // `span=2`、`generation=0`（格式时刻分配，不是任何一次真实发布的 txg）吻合。**这不是 LIVELIST /
    // SPARSE_SIDE_TABLE / DEADLIST 三种树的节点**（那三种树表条目的根恒为 `NodePointer::empty_root()`，
    // `transaction.rs` 的 `publish_admitted` 每次发布都这样写，这批历史、乃至今天全仓任何
    // 一条写路径都不产生它们的物理节点——判定见交回报告）。
    if root
        .instance_table
        .locations
        .iter()
        .any(|location| location_key(location) == pair)
    {
        return "instance_table_self_node_not_walked_by_the_all_structural_trees_arm";
    }
    if let Some(locations) = allocation_record_tree_locations {
        if locations
            .iter()
            .any(|location| location_key(location) == pair)
        {
            return "allocation_record_tree_self_node";
        }
    }
    if let Some(tree_table_bytes) = read_node_bytes(plain_devices, &root.tree_table.locations) {
        if let Ok(tree_table) = parse_index_node(&tree_table_bytes) {
            for entry_bytes in &tree_table.entries {
                if let Some(entry) = TreeTableEntry::parse(entry_bytes) {
                    if matches!(
                        entry.kind,
                        TREE_KIND_LIVELIST | TREE_KIND_SPARSE_SIDE_TABLE | TREE_KIND_DEADLIST
                    ) && entry
                        .root
                        .locations
                        .iter()
                        .any(|location| location_key(location) == pair)
                    {
                        return "livelist_sparse_deadlist_root_but_always_empty";
                    }
                }
            }
        }
    }
    match records
        .iter()
        .find(|record| (record.device.0, record.slot.0) == pair)
        .map(|record| record.span_slots)
    {
        Some(2) => "span2_unclassified_not_instance_table_not_content_mapping",
        Some(1) => "span1_structural_node_not_a_livelist_sparse_deadlist_root",
        Some(_) => "span_other_unexpected",
        None => "not_found_in_raw_allocation_records",
    }
}

#[cfg(test)]
mod tests;

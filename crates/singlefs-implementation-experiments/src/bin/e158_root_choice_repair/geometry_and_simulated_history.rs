//! 几何（本轮跑的四个几何点）、装置状态（内存盘、会话、装置自己的父链账本）与步进（W 覆盖写、M 可写挂载、R(j) 挂着回退）。

use super::*;

/// 这个装置里的「管理员回退」：先可写挂载、再挂着回退到同一个目标（`mount::roll_back_by_a_forward_publish`，
/// D23（journal 的角色与格式） 已定项 14）。交回的 `Mounted` 是那次可写挂载的（`output` 里的写行与暖机是它的），
/// `current` 换成回退那次发布之后的一版。影子账那一臂恒开（可写挂载的产品路径）。
/// 回退是一次向前发布，不抛弃根：`apply_mount_rollback` 的账本照发布如实记（`ledger_after_a_rollback_published_forward`）。
/// 用到账本的 R(j) 一步：H3 的 `enumerate`、Q3-1 的 `enumerate_fault_set_violations`、H1 的 `ledger_fault_history_family`。
/// op1 `MountRollbackTo` 与 Q2-2a 固定脚本直接调这个函数，不进账本。
pub(super) fn mount_rollback<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    target: RollbackTarget,
    _shadow_ledger_is_always_on: ShadowLedger,
) -> Result<Mounted, String> {
    let mut mounted =
        mount_writable(parameters, devices).map_err(|error| format!("可写挂载：{error:?}"))?;
    let PoolVersion::WithFile(current) = &mut mounted.current else {
        return Err("可写挂载之后现行那一版树表 0 条：挂着回退要带文件的现行版本".to_string());
    };
    let current = std::sync::Arc::make_mut(current);
    roll_back_by_a_forward_publish(parameters, devices, &mut mounted.allocator, current, target)
        .map_err(|error| format!("挂着回退：{error:?}"))?;
    Ok(mounted)
}

// ============================================================================
// 二、几何：本轮跑的四个几何点（5.1 H3 行：GEOMETRY_PRIMARY、S=16、小环、S=4 另跑 L=4）。
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub(super) struct Geometry {
    pub(super) label: &'static str,
    pub(super) slots_per_region: u64,
    pub(super) journal_ring_bytes: u64,
}

pub(super) const GEOMETRY_PRIMARY: Geometry = Geometry {
    label: "GEOMETRY_PRIMARY(S=8,ring=3MiB)",
    slots_per_region: 8,
    journal_ring_bytes: 3 * 1024 * 1024,
};
pub(super) const GEOMETRY_LARGER_ROOT_RING: Geometry = Geometry {
    label: "S16(S=16,ring=3MiB)",
    slots_per_region: 16,
    journal_ring_bytes: 3 * 1024 * 1024,
};
pub(super) const GEOMETRY_SMALLER_ROOT_RING: Geometry = Geometry {
    label: "S4(S=4,ring=3MiB)",
    slots_per_region: 4,
    journal_ring_bytes: 3 * 1024 * 1024,
};

pub(super) fn region_devices() -> [DeviceIdentity; 3] {
    [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)]
}

pub(super) fn parameters_for(geometry: &Geometry) -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: region_devices(),
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: 4096,
            journal_ring_bytes: geometry.journal_ring_bytes,
            root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
                geometry.slots_per_region,
            )
            .expect("S 在 [4,16] 区间内"),
        },
    }
}

pub(super) fn fixed_geometry_for(geometry: &Geometry) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: 4096,
        journal_ring_bytes: geometry.journal_ring_bytes,
        root_ring_slots_per_region: RootRingSlotsPerRegion::from_system_configuration_field(
            geometry.slots_per_region,
        )
        .expect("S 在 [4,16] 区间内"),
    }
}

// ============================================================================
// 三、装置状态：内存盘 + 会话 + 装置自己的父链账本（不借 `crates/` 的实例表判时间线）。
// ============================================================================

pub(super) type TimelineRoot = (u32, u64);

#[derive(Clone)]
pub(super) struct Session {
    pub(super) allocator: PoolAllocator,
    pub(super) current: PoolVersion,
    pub(super) instance: InstanceGeneration,
}

#[derive(Clone)]
pub(super) struct RollbackEvent {
    /// 这次回退抛弃的根。回退是一次向前发布（D23（journal 的角色与格式） 已定项 14），`apply_mount_rollback`
    /// 记的恒为空；非空的账只出现在手写账本的单测里（PC-Nw、V1 的计数函数）。
    pub(super) abandoned: BTreeSet<TimelineRoot>,
}

/// 只为 `TRACE` 行里的 `path` 打印用：`RollbackTo` 的两个字段只被 `Debug` 读，`dead_code` 分析看不见
/// 派生的 `Debug` 实现，允许它照报。
#[allow(
    dead_code,
    reason = "RollbackTo 的两个字段只被派生的 Debug 实现读，dead_code 分析看不见这条路"
)]
#[derive(Clone, Copy, Debug)]
pub(super) enum HistoryStep {
    Overwrite,
    MountWritable,
    RollbackTo(u32, u64),
}

#[derive(Clone)]
pub(super) struct SimNode {
    pub(super) pool: MemoryPool,
    pub(super) session: Option<Session>,
    /// 装置自记的「当前时间线」：从 mkfs 的第 0 代根到最新一次成功发布的根，按发布次序排列；回退那次发布也是一条。
    pub(super) timeline: Vec<TimelineRoot>,
    pub(super) rollback_events: Vec<RollbackEvent>,
    pub(super) path: Vec<HistoryStep>,
    /// Q3-1（P332 的 V2 与严格判据）用：这个模拟池里出现过的每一条根对应的「内容号」——
    /// `None` = 那一版没有文件，`Some(字节)` = 那一版文件的确定性内容。只增不删。
    pub(super) content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>>,
}

pub(super) fn new_devices(image_bytes: u64) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(image_bytes, PhysicalBlockSizeInBytes(512)),
            )
        })
        .collect()
}

pub(super) fn memory_pool_of(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    image_bytes: u64,
) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: image_bytes,
    }
}

pub(super) fn devices_from_pool(
    pool: &MemoryPool,
    image_bytes: u64,
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            let mut device = SparseBlockDevice::new(image_bytes, PhysicalBlockSizeInBytes(512));
            if let Some(image) = pool.devices.get(&identity) {
                device.image = image.clone();
            }
            (identity, device)
        })
        .collect()
}

pub(super) fn deterministic_content(tag: u64, length: usize) -> Vec<u8> {
    (0..length)
        .map(|index| {
            let value = (index as u64)
                .wrapping_mul(131)
                .wrapping_add(tag.wrapping_mul(977))
                .wrapping_add(7);
            (value % 251) as u8
        })
        .collect()
}

pub(super) fn root_pair(root: &singlefs_core::root_record::RootRecord) -> TimelineRoot {
    (root.instance.0, root.checkpoint_txg.0)
}

// ============================================================================
// 四、步进：W（覆盖写）、M（关闭并 `mount_writable`）、R(j)（关闭并 `mount_rollback` 到 j）。
//    三个都从 `&SimNode` 读、交回一个新 `SimNode`；不改传入的那个（DFS 靠这个做回溯）。
// ============================================================================

pub(super) fn bootstrap(
    geometry: &Geometry,
    overwrites_before_sigma: u64,
) -> Result<SimNode, String> {
    let parameters = parameters_for(geometry);
    let mut devices = new_devices(IMAGE_BYTES);
    let genesis =
        make_filesystem(&parameters, &mut devices).map_err(|error| format!("mkfs: {error:?}"))?;
    // mkfs 同一个进程里接着写的分配器照 mkfs 按环长现算的单元区起点建（实审 A3b 报告 Q8：这里的环 3 MiB，起点 1216，
    // 不是编译期常量 50176），并装上 mkfs 刚写下的根环。
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content = deterministic_content(0, FIRST_FILE_BYTES);
    let (warm_up_output, first_output) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer)
            .map_err(|error| format!("acquire_instance: {error:?}"))?;
        let warm = warm_up(&mut writer, &genesis.root, instance)
            .map_err(|error| format!("warm_up: {error:?}"))?;
        let output = publish_first_file(
            &mut writer,
            &mut allocator,
            warm.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm.last_record_bytes,
        )
        .map_err(|error| format!("publish_first_file: {error:?}"))?;
        (warm, output)
    };
    let mut timeline = vec![root_pair(&genesis.root)];
    let mut content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>> = BTreeMap::new();
    content_by_root.insert(root_pair(&genesis.root), None);
    for root in &warm_up_output.roots {
        timeline.push(root_pair(root));
        content_by_root.insert(root_pair(root), None);
    }
    timeline.push(root_pair(&first_output.root));
    content_by_root.insert(root_pair(&first_output.root), Some(content.clone()));
    let instance = InstanceGeneration(1);
    let mut node = SimNode {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: Some(Session {
            allocator,
            current: PoolVersion::WithFile(std::sync::Arc::new(first_output)),
            instance,
        }),
        timeline,
        rollback_events: Vec::new(),
        path: Vec::new(),
        content_by_root,
    };
    for _ in 0..overwrites_before_sigma {
        node = apply_overwrite(&node, &parameters)?;
    }
    Ok(node)
}

pub(super) fn apply_overwrite(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
) -> Result<SimNode, String> {
    let session = node
        .session
        .as_ref()
        .ok_or_else(|| "W: 没有开着的会话".to_string())?;
    let PoolVersion::WithFile(previous) = &session.current else {
        return Err("W: 现行版本没有文件".to_string());
    };
    let step_tag = node.path.len() as u64 + 1;
    let content = deterministic_content(step_tag, OVERWRITE_FILE_BYTES);
    let mut devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mut allocator = session.allocator.clone();
    let output = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut allocator,
            previous,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 * step_tag,
            },
            session.instance,
        )
        .map_err(|error| format!("publish_overwrite: {error:?}"))?
    };
    let mut timeline = node.timeline.clone();
    timeline.push(root_pair(&output.root));
    let mut content_by_root = node.content_by_root.clone();
    content_by_root.insert(root_pair(&output.root), Some(content));
    let mut path = node.path.clone();
    path.push(HistoryStep::Overwrite);
    Ok(SimNode {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: Some(Session {
            allocator,
            current: PoolVersion::WithFile(std::sync::Arc::new(output)),
            instance: session.instance,
        }),
        timeline,
        rollback_events: node.rollback_events.clone(),
        path,
        content_by_root,
    })
}

pub(super) fn apply_mount_writable(
    node: &SimNode,
    parameters: &MakeFilesystemParameters,
) -> Result<SimNode, String> {
    let mut devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mounted = mount_writable(parameters, &mut devices)
        .map_err(|error| format!("mount_writable: {error:?}"))?;
    let tip = *node.timeline.last().expect("timeline 非空");
    let chosen = root_pair(&mounted.output.chosen_root);
    if chosen != tip {
        return Err(format!(
            "S5: mount_writable 的 chosen_root {chosen:?} 与装置自记的时间线尾 {tip:?} 不一致（不注入故障时应当相等）"
        ));
    }
    let carried_forward_content = node.content_by_root.get(&tip).cloned().unwrap_or(None);
    let mut timeline = node.timeline.clone();
    timeline.push(root_pair(mounted.output.row_publish.root()));
    let mut content_by_root = node.content_by_root.clone();
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
        pool: memory_pool_of(&devices, IMAGE_BYTES),
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

/// 一次回退之后的账：时间线与每条根的内容号。
struct LedgerAfterRollback {
    timeline: Vec<TimelineRoot>,
    content_by_root: BTreeMap<TimelineRoot, Option<Vec<u8>>>,
}

/// 如实的账本：回退是一次向前发布，什么都不截（D23（journal 的角色与格式） 已定项 14）。
/// `mount_rollback` 先做了一次可写挂载，写行与暖机那几条根的内容是挂载之前最新根的；
/// 回退那次发布的根内容是目标的，它是时间线的尾。
fn ledger_after_a_rollback_published_forward(
    node: &SimNode,
    target_root: TimelineRoot,
    chosen_root_of_the_mount: TimelineRoot,
    row_and_warm_up_roots: &[TimelineRoot],
    rollback_root: TimelineRoot,
) -> Result<LedgerAfterRollback, String> {
    if !node.timeline.contains(&target_root) {
        return Err(format!(
            "S5: 回退目标 {target_root:?} 不在装置自记的时间线上"
        ));
    }
    let tip = *node
        .timeline
        .last()
        .expect("bootstrap 至少写下 mkfs 根，时间线非空");
    if chosen_root_of_the_mount != tip {
        return Err(format!(
            "S5: mount_writable 的 chosen_root {chosen_root_of_the_mount:?} 与装置自记的时间线尾 {tip:?} 不一致（不注入故障时应当相等）"
        ));
    }
    let content_before_the_rollback = node.content_by_root.get(&tip).cloned().unwrap_or(None);
    let target_content = node
        .content_by_root
        .get(&target_root)
        .cloned()
        .unwrap_or(None);
    let mut timeline_as_published = node.timeline.clone();
    let mut content_by_root = node.content_by_root.clone();
    for publish_root in row_and_warm_up_roots {
        timeline_as_published.push(*publish_root);
        content_by_root.insert(*publish_root, content_before_the_rollback.clone());
    }
    timeline_as_published.push(rollback_root);
    content_by_root.insert(rollback_root, target_content);
    Ok(LedgerAfterRollback {
        timeline: timeline_as_published,
        content_by_root,
    })
}

pub(super) fn apply_mount_rollback(
    node: &SimNode,
    target: RollbackTarget,
    parameters: &MakeFilesystemParameters,
) -> Result<SimNode, String> {
    let mut devices = devices_from_pool(&node.pool, IMAGE_BYTES);
    let mounted = mount_rollback(parameters, &mut devices, target, ShadowLedger::On)
        .map_err(|error| format!("mount_rollback: {error:?}"))?;
    let PoolVersion::WithFile(published) = &mounted.current else {
        return Err("S5: 回退之后现行那一版不带文件".to_string());
    };
    let mut row_and_warm_up_roots = vec![root_pair(mounted.output.row_publish.root())];
    for warm_up_publish in &mounted.output.warm_up_publishes {
        row_and_warm_up_roots.push(root_pair(warm_up_publish.root()));
    }
    let ledger = ledger_after_a_rollback_published_forward(
        node,
        (target.instance.0, target.checkpoint_txg.0),
        root_pair(&mounted.output.chosen_root),
        &row_and_warm_up_roots,
        root_pair(&published.root),
    )?;
    let mut rollback_events = node.rollback_events.clone();
    rollback_events.push(RollbackEvent {
        abandoned: BTreeSet::new(),
    });
    let mut path = node.path.clone();
    path.push(HistoryStep::RollbackTo(
        target.instance.0,
        target.checkpoint_txg.0,
    ));
    Ok(SimNode {
        pool: memory_pool_of(&devices, IMAGE_BYTES),
        session: Some(Session {
            allocator: mounted.allocator,
            current: mounted.current,
            instance: mounted.output.instance,
        }),
        timeline: ledger.timeline,
        rollback_events,
        path,
        content_by_root: ledger.content_by_root,
    })
}

#[cfg(test)]
mod tests;

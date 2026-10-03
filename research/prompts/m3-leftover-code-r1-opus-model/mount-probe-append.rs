
/// 攻方腿副本探针（m3-leftover-code-r1-opus）：根环里每条可读根，账那一臂与从映射现算那一臂各交什么。
#[doc(hidden)]
pub type OpusProbePlacements = Option<Vec<(DeviceIdentity, crate::address::SlotNumber, u64)>>;

#[doc(hidden)]
#[allow(clippy::ptr_arg, clippy::type_complexity)]
pub fn opus_probe_placements_of_every_ring_root<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
) -> Vec<(RootRecord, Result<bool, RecoveryFailure>, OpusProbePlacements, OpusProbePlacements)> {
    let system_configuration = choose_system_configuration(&*devices).expect("探针：择系统配置");
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(&system_configuration);
    readable_roots_with_ring_slots(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .iter()
    .map(|(_, root)| {
        (
            *root,
            tree_table_has_no_entries(devices, root),
            placements_referenced_by_root(devices, unit_area_start, root),
            placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root),
        )
    })
    .collect()
}

/// 探针：每条根是不是被最新那条根的实例表判成被抛弃。
#[doc(hidden)]
#[allow(clippy::ptr_arg)]
pub fn opus_probe_abandoned_flags<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
) -> Vec<((InstanceGeneration, CheckpointTxg), Option<bool>)> {
    let system_configuration = choose_system_configuration(&*devices).expect("探针：择系统配置");
    let table = choose_root(devices, &system_configuration)
        .and_then(|newest| instance_table_of_root(devices, &newest));
    readable_roots_with_ring_slots(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
    )
    .iter()
    .map(|(_, root)| {
        (
            (root.instance, root.checkpoint_txg),
            table.as_ref().map(|table| abandoned_by_table(root, table)),
        )
    })
    .collect()
}

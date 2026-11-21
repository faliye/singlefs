use super::*;

/// H-C2 机制假说的最小复现，不依赖任何臂改法：只把设备 0 上世代号更高的系统配置槽读失败，
/// `singlefs_core::recovery::choose_system_configuration` 交回的 `slot_generation` 应当退回
/// 设备 0 自己更旧的那一份，不看设备 1（`recovery.rs` 的 `choose_system_configuration`，`chosen` 只在 `None` 时
/// 赋值一次）。这条测试不碰任何臂改法，验证的是「今天」这份 `choose_system_configuration`
/// 本身的行为——H-C2 三个驱动函数（十三点六节）依赖的正是这个行为。
#[test]
fn faulting_the_newer_device_zero_system_configuration_slot_regresses_the_chosen_generation() {
    let node = bootstrap(&GEOMETRY_PRIMARY, 1).expect("bootstrap 应当成功");
    let geometry_view = independent_geometry(&node.pool).expect("几何应当能独立解出来");
    let (newest_slot_index, newest_generation) =
        newest_self_certified_system_configuration_slot_on_device_zero(
            &node.pool,
            geometry_view.slot_spacing,
        )
        .expect("这段历史上设备 0 应当至少有一个自证过的系统配置槽");
    let (_, oldest_generation) = oldest_self_certified_system_configuration_slot_on_device_zero(
        &node.pool,
        geometry_view.slot_spacing,
    )
    .expect("这段历史上设备 0 的两个槽应当都自证过（多次轮换之后）");
    assert!(
        oldest_generation < newest_generation,
        "两槽世代号应当不同：oldest={oldest_generation} newest={newest_generation}"
    );

    let plain = devices_from_pool(&node.pool, IMAGE_BYTES);
    let baseline = choose_system_configuration(&plain).expect("不注入时应当读得出系统配置");
    assert_eq!(
        baseline.quantities.slot_generation, newest_generation,
        "不注入时 chosen 应当是当下最新那一份"
    );

    let offset = newest_slot_index * geometry_view.slot_spacing;
    let mut wrapped: Vec<(DeviceIdentity, MultiOffsetReadFailingBlockDevice)> = Vec::new();
    for (identity, device) in plain {
        let failing_offsets: BTreeSet<u64> = if identity == DeviceIdentity(0) {
            BTreeSet::from([offset])
        } else {
            BTreeSet::new()
        };
        wrapped.push((
            identity,
            MultiOffsetReadFailingBlockDevice {
                inner: device,
                failing_offsets,
            },
        ));
    }
    let faulted = choose_system_configuration(&wrapped).expect("设备 0 的另一槽仍自证得过");
    assert_eq!(
        faulted.quantities.slot_generation, oldest_generation,
        "设备 0 的新槽读失败之后，chosen 应当退回设备 0 自己更旧的那一份；\
             读成 {newest_generation}（未受影响的设备 1 的值）就说明选择逻辑其实看了别的盘"
    );
}

/// PC2 甲：`root_ring_slot_targets_for` 按 (实例, txg) 精确点名，不多不少。
#[test]
fn root_ring_slot_targets_for_finds_exactly_the_named_roots() {
    let node = bootstrap(&GEOMETRY_PRIMARY, 4).expect("bootstrap 应当成功");
    let geometry_view = independent_geometry(&node.pool).expect("已经过 mkfs 的池应当解得出几何");
    let topmost_two: BTreeSet<TimelineRoot> = node.timeline.iter().rev().take(2).copied().collect();
    let targets = root_ring_slot_targets_for(&node.pool, &geometry_view, &topmost_two);
    assert_eq!(
        targets.len(),
        2,
        "点名两条根，每条第一版几何下只住一块盘，应当解出两个 (设备, 偏移)"
    );
    let mut deduplicated: BTreeSet<(DeviceIdentity, DeviceOffsetInBytes)> = BTreeSet::new();
    for target in &targets {
        deduplicated.insert(*target);
    }
    assert_eq!(deduplicated.len(), 2, "两条根不应该解到同一个 (设备, 偏移)");
}

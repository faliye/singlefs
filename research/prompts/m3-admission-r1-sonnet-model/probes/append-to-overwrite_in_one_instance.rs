
/// 草稿探针（腿自己的副本，不入库）：同上一条的历史，但把只供测试的开关关掉（准入先拒不判），让落点那一道去拒。
/// 预期：`PlacementRefused`；分配器退回进去之前那一份。这是 `crates/mutations.tsv` 第 12 行（失败的发布不退回分配器）要的那种历史。
#[test]
fn probe_c576_with_the_switch_off_publish_running_out_of_space_midway_leaves_the_allocator_as_it_was() {
    let mut pool = build_pool("no-space-midway-switch-off");
    pool.allocator.allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3)).expect("A 的段里还有槽");
    for _drain_round in 0..50 {
        pool.allocator
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
            .expect("段里还有槽");
    }
    let unit_area_end = IMAGE_BYTES / SLOT_BYTES;
    for device_map in &mut pool.allocator.devices {
        for slot in UNIT_AREA_START_SLOT..unit_area_end {
            if slot == 50182 || slot == 50183 {
                continue;
            }
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    pool.allocator
        .set_space_admission(singlefs_core::admission::SpaceAdmission::SkippedByTheTestOnlySwitch);
    let records_before = pool.allocator.records().to_vec();
    let previous = pool.output.clone();
    let result = try_overwrite(&mut pool, &previous);
    println!("name=c576-probe-switch-off result={:?}", result.as_ref().err().map(|e| format!("{e:?}").chars().take(160).collect::<String>()));
    assert!(matches!(&result, Err(PublishError::PlacementRefused { .. })), "落点那一道拒：{:?}", result.as_ref().err());
    assert_eq!(pool.allocator.records(), &records_before[..], "失败的发布退回");
    assert!(pool.allocator.devices[0].is_free(SlotNumber(50182)), "B 拿到过的数据槽退回去了");
    assert_eq!(pool.allocator.open_segment(), Some(SlotNumber(50240)), "开放段也退回去了");
}

/// 草稿探针（腿自己的副本，不入库）：准入先拒放行、落点那一道拒、而且拒在已经取走前面几个落点之后的历史——
/// 数据单元拿走唯一的一对 50182–50183，剩下没挡的槽是 700 个互不相邻的奇数槽（够 `commit_generated_slots` 的槽数，不成对）；
/// inode 叶容器要对齐的一对，取不到。准入先拒（只数槽）放行，`PlacementRefused` 在分配器已经被前面的落点改动之后返回。
#[test]
fn probe_c576_pair_fragmented_publish_passes_the_landing_check_and_is_refused_by_the_placement_after_the_allocator_was_touched() {
    let mut pool = build_pool("no-pair-midway");
    let first_drained = pool
        .allocator
        .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
        .expect("A 的段里还有槽");
    let _ = first_drained;
    for _drain_round in 0..50 {
        pool.allocator
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
            .expect("段里还有槽");
    }
    let unit_area_end = IMAGE_BYTES / SLOT_BYTES;
    let keep_free = |slot: u64| slot == 50182 || slot == 50183 || (slot >= 50185 && slot % 2 == 1 && slot < 50185 + 2 * 700);
    for device_map in &mut pool.allocator.devices {
        for slot in UNIT_AREA_START_SLOT..unit_area_end {
            if keep_free(slot) {
                continue;
            }
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    let records_before = pool.allocator.records().to_vec();
    let previous = pool.output.clone();
    let result = try_overwrite(&mut pool, &previous);
    println!("name=c576-probe-pair-fragmented result={:?}", result.as_ref().err().map(|e| format!("{e:?}").chars().take(200).collect::<String>()));
    assert!(matches!(&result, Err(PublishError::PlacementRefused { .. })), "落点那一道拒：{:?}", result.as_ref().err());
    assert_eq!(pool.allocator.records(), &records_before[..], "失败的发布退回");
    assert!(pool.allocator.devices[0].is_free(SlotNumber(50182)), "数据槽退回去了");
}

//! 读数与压满：每块盘的可用字节、保留量读数、分配记录树位置与从单元区末尾压满的单测。

use super::*;

/// 空闲图的几个计数：(已分配, 空闲, 待释放, 空闲段数, 全空的聚簇段数, 没挡的槽数)。
fn counts_of_the_device_free_map(map: &DeviceFreeMap) -> [u64; 6] {
    [
        map.allocated_slots(),
        map.free_slots(),
        map.deferred_slots(),
        map.free_runs(),
        map.empty_segments(),
        map.unblocked_slots(),
    ]
}

#[test]
fn unmarking_the_filled_slots_gives_back_every_count_the_fill_took() {
    let mut device_maps = vec![DeviceFreeMap::new(DeviceIdentity(0), PRODUCT_DEVICE_BYTES)];
    let start_slot = device_maps[0].unit_area_start().slot().0;
    let end_slot = start_slot + device_maps[0].unit_area_slots();
    device_maps[0].mark_allocated(SlotNumber(start_slot), 3);
    device_maps[0].mark_allocated(SlotNumber(end_slot - 70), 1);
    let counts_before_the_fill = counts_of_the_device_free_map(&device_maps[0]);
    let filled_slots = mark_free_slots_from_the_unit_area_end(&mut device_maps[0], 130);
    assert_eq!(filled_slots.len(), 130, "单元区末尾往前有 130 个空闲槽可标");
    assert!(!filled_slots.contains(&(end_slot - 70)), "已经占着的槽跳过");
    assert_eq!(
        device_maps[0].allocated_slots(),
        3 + 1 + 130,
        "压满标掉的槽进了已分配（2026-10-01 第一段 S7 停机：记账比走读多的就是它们）"
    );
    unmark_the_filled_slots(&mut device_maps, std::slice::from_ref(&filled_slots));
    assert_eq!(
        counts_of_the_device_free_map(&device_maps[0]),
        counts_before_the_fill,
        "撤掉压满之后六个计数都回到压满之前"
    );
    assert!(
        filled_slots
            .iter()
            .all(|slot| device_maps[0].is_free(SlotNumber(*slot))),
        "标掉的槽逐个回到空闲"
    );
}

#[test]
fn publish_after_the_filled_slots_are_unmarked_leaves_a_pool_the_checker_holds() {
    // 池级 checker 那一判经给定状态表两段判：配置开了 GPU 而这份二进制没编进 gpu 特性时判段不跑，照本次未跑交回
    if singlefs_verification::crash_amplification::judging_test_not_run_without_the_gpu_feature(
        "e168_delete_ring_window_borrow::publish_after_the_filled_slots_are_unmarked_leaves_a_pool_the_checker_holds",
    ) {
        return;
    }
    let PoolWithAFile::Written(mut pool) = pool_with_a_file(
        narrow_geometry_with_the_ring_window_reserved(ROOT_RING_SLOTS_PER_REGION_MINIMUM),
        1,
    ) else {
        panic!("单元区 790 槽、S = 4 上写得进一个单元的文件");
    };
    let FillOutcome::WithinOneSlotAtOrBelowZero {
        filled_slots_of_each_device,
        ..
    } = fill_from_the_unit_area_end(&mut pool)
    else {
        panic!("单元区 790 槽、S = 4、一个单元的池压得满");
    };
    assert!(
        filled_slots_of_each_device
            .iter()
            .all(|filled_slots| !filled_slots.is_empty()),
        "两块盘都标了槽"
    );
    unmark_the_filled_slots(
        &mut pool.session.allocator.devices,
        &filled_slots_of_each_device,
    );
    let content = content_needing(1, 3);
    let overwrite = UserChange::Overwrite(FirstFile {
        content: &content,
        write_time_seconds: WRITE_TIME_SECONDS + 120,
    });
    if let Err(refusal) = pool
        .session
        .publish_user_change(&mut pool.devices, overwrite)
    {
        panic!("撤掉压满之后式子可用回到压满之前，覆盖写一个单元放得进：{refusal:?}");
    }
    assert_eq!(
        checker_violations(
            "unit-test/a-publish-after-the-filled-slots-are-unmarked",
            &pool.image()
        ),
        Vec::<String>::new(),
        "撤掉压满之后再发一次，记账按分配器重写，checker 不判红（不撤就是 S7 那两条 I-3.1、I-3.11）"
    );
}

//! 被抛弃根引用的 defer 槽：挂载那一刻与根槽读坏时的抬 F 都不双扣。

use super::*;

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed`；池级 checker 那一遍在 同名的给定状态用例（`crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_given_states_engine(
        scenarios_of_raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table::run_the_scenario_where_defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed,
    );
}

/// 根槽这一遍读坏的被抛弃根那一圈同样不双扣（派发第十六块第 2 条）：同一份盘面挂上（实例 5，F = C 的 txg）之后 C 的根槽读报错，
/// 把 F 抬到它现在的值（只供测试强制进入的那一档，F 不变）：抬 F 那一串读根环时 C 那一槽读坏、照根环表里记着的落点进影子账
/// （D16（发布语义） 已定项 1「根槽这一次读坏」，H1），那几个 defer 槽释放代高于 F、这一串不回收：抬完之后没有一个槽同时算进「已分配」与第九项，
/// C 引用的槽一个都不空闲。这一圈照旧整段隔离时抬完那一句红。
#[test]
fn defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are_not_counted_twice_by_a_floor_raise(
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let image = image_after_raising_the_floor_to_the_txg_of_the_abandoned_root(&built);
    let fault =
        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let (mut pool, recorded_for_the_abandoned_publish) = mounted_while_the_abandoned_root_reads(
        &PoolWithAnAbandonedRoot {
            parameters: built.parameters.clone(),
            device_bytes: built.device_bytes,
            image,
            write_time_seconds: built.write_time_seconds + 3000,
            abandoned_root: built.abandoned_root,
            newest_valid_root: built.newest_valid_root,
        },
        &fault,
    );
    let floor = pool.rollback_floor();
    assert_eq!(
        floor, built.abandoned_root.checkpoint_txg,
        "挂载那一刻 F 是 C 的 txg（阳性对照）"
    );
    let defer_slots_of_the_abandoned_publish = released_above_the_floor_and_covered_by(
        &pool.session().allocator,
        &recorded_for_the_abandoned_publish,
        floor,
    );
    assert!(
        !defer_slots_of_the_abandoned_publish.is_empty(),
        "C 引用的槽里有现行账已释放、释放代高于 F 的（阳性对照）"
    );
    fault.is_active.set(true);
    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
    let mut session = pool.session.take().expect("挂着的会话");
    let raised = {
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("挂载落到的那一版带文件");
        };
        let current = Arc::make_mut(current);
        raise_rollback_floor(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            floor,
            ShadowLedger::On,
        )
    };
    pool.session = Some(session);
    let raised =
        raised.unwrap_or_else(|refusal| panic!("C 的根槽读坏、F 不变的抬 F 做得成：{refusal:?}"));
    assert_eq!(
        raised.abandoned_roots_unreadable, 0,
        "C 的根槽读坏：照根环表里记着的落点算，不读它的账"
    );
    let allocator = &pool.session().allocator;
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(allocator),
        0,
        "抬完之后没有一个槽同时算进「已分配」与第九项"
    );
    let still_allocated_defer_slots: Vec<&(DeviceIdentity, SlotNumber)> =
        defer_slots_of_the_abandoned_publish
            .iter()
            .filter(|(device, slot)| {
                allocator.devices.iter().any(|device_map| {
                    device_map.device == *device
                        && !device_map.is_free(*slot)
                        && !device_map.is_isolated(*slot)
                })
            })
            .collect();
    assert!(
        !still_allocated_defer_slots.is_empty(),
        "抬完之后这几个 defer 槽还分配着、这一串没回收（阳性对照：这一圈遇到了分配位置着的槽）"
    );
    assert_none_of_the_placements_is_free(allocator, &recorded_for_the_abandoned_publish, "C 记着");
}

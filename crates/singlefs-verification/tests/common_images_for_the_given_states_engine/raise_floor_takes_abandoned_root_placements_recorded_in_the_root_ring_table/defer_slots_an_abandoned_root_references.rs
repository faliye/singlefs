//! 被抛弃根引用的 defer 槽在挂载那一刻不双扣、回收之后照旧隔离。

use super::*;

/// 实例 4 里覆盖写几次再把 F 抬到 C 的 txg：之后有 4 个以上不同的非空状态，准入上限高过 C 的 txg（抬 F 只供测试强制进入的那一档）。
pub const OVERWRITES_IN_THE_INSTANCE_RAISING_THE_FLOOR_TO_THE_ABANDONED_ROOT: usize = 4;

/// 挂上之后至多覆盖写几次去等 C 离开根环。
pub const OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST: usize = 40;

/// 被抛弃根那一圈的 defer 槽的池：崩溃恢复抛弃了 C 的那份盘面
/// （[`pool_whose_newest_root_was_abandoned_by_a_crash_recovery`]：实例 3 择根退到 C 之前那一条 B、写行换下 C 与 B 共用的实例 2 那张实例表），
/// 实例 4 挂上、覆盖写 4 次、把 F 抬到 C 的 txg（B 不再是候选根），停在那里（进程退出）。交回实例 4 退出时的盘面。
/// 这之后的可写挂载里，C 引用、现行账里已释放（实例 3 的写行换下的）、释放代高于 F 的槽不再被候选根豁免，还在 defer 窗口里（分配位置着）。
pub fn image_after_raising_the_floor_to_the_txg_of_the_abandoned_root(
    built: &PoolWithAnAbandonedRoot,
) -> MemoryPool {
    let mut pool = PoolUnderTest {
        parameters: built.parameters.clone(),
        device_bytes: built.device_bytes,
        devices: plain_devices_on(&built.image),
        session: None,
        write_time_seconds: built.write_time_seconds + 1000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    pool.crash_and_mount_writable()
        .expect("实例 4 可写挂载做得成（C 的根槽与账读得出）");
    for overwrite_number in 1..=OVERWRITES_IN_THE_INSTANCE_RAISING_THE_FLOOR_TO_THE_ABANDONED_ROOT {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("实例 4 第 {overwrite_number} 次覆盖写：{refusal:?}"));
    }
    let mut session = pool.session.take().expect("挂着的会话");
    let raised = {
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("覆盖写之后现行那一版带文件");
        };
        let current = Arc::make_mut(current);
        raise_rollback_floor(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            built.abandoned_root.checkpoint_txg,
            ShadowLedger::On,
        )
    };
    raised.unwrap_or_else(|refusal| panic!("实例 4 把 F 抬到 C 的 txg 做得成：{refusal:?}"));
    pool.image()
}

/// `placements` 罩着的槽里，现行账（`allocator` 的记录）里记成已释放、释放代高于 `floor` 的（盘、槽）：挂载那一刻还在 defer 窗口里、
/// 不被候选根（txg ≥ F）豁免的那一截的候选。从记录算，不经被测的方法。
pub fn released_above_the_floor_and_covered_by(
    allocator: &PoolAllocator,
    placements: &[PlacementOnDevice],
    floor: CheckpointTxg,
) -> Vec<(DeviceIdentity, SlotNumber)> {
    allocator
        .records()
        .iter()
        .filter(|record| record.is_released && record.generation > floor)
        .flat_map(|record| {
            (record.slot.0..record.slot.0 + u64::from(record.span_slots))
                .map(move |slot| (record.device, SlotNumber(slot)))
        })
        .filter(|(device, slot)| {
            placements.iter().any(|placement| {
                placement.device == *device
                    && slot.0 >= placement.slot.0
                    && slot.0 < placement.slot.0 + placement.span
            })
        })
        .collect()
}

/// 被抛弃根那一圈挂载那一刻不双扣、回收之后照旧隔离（派发第十六块第 2 条）：实例 4 把 F 抬到 C 的 txg 之后崩了再挂（实例 5，F = C 的 txg）。
/// 挂载那一刻 C 记成被抛弃根，它引用的槽里有现行账已释放、释放代高于 F（还在 defer 窗口里、分配位置着）、不被候选根豁免的：
/// 两块盘没有一个槽同时算进「已分配」与第九项。之后覆盖写到 C 离开根环：C 还在环里时它引用的槽一个都不空闲，其中那几个 defer 槽
/// 至少有一个在 C 离环之前被回收、补上了隔离（回收时照根环表补，阳性对照：走到了回收那一处）；每一步池级 checker 不红 I-7.4、I-4.8。
/// 被抛弃根那一圈照旧整段隔离时挂载那一刻那一句红。
pub fn run_the_scenario_where_defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed(
    images: &mut PoolCheckerImages,
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let image = image_after_raising_the_floor_to_the_txg_of_the_abandoned_root(&built);
    let fault =
        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let mut devices = devices_with_the_read_fault_on(&image, &fault);
    // 空间准入关着（只供测试的开关）：会话不为空间推抬 F，一次覆盖写只发一次、txg 只走一格，看得到「C 那一槽写掉之前那一次发布」之后的那一刻。
    let mounted = mount_writable_with_space_admission(
        &built.parameters,
        &mut devices,
        SpaceAdmission::SkippedByTheTestOnlySwitch,
    )
    .expect("实例 5 可写挂载做得成");
    assert_eq!(
        mounted.output.effective_root.rollback_floor, built.abandoned_root.checkpoint_txg,
        "挂载那一刻 F 是 C 的 txg（阳性对照：B 不再是候选根）"
    );
    let recorded_for_the_abandoned_publish = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_recorded_for_the_abandoned_root(
            built.abandoned_root.instance,
            built.abandoned_root.checkpoint_txg,
        )
        .filter(|placements| !placements.is_empty())
        .expect("挂载那一刻根环表把 C 记成被抛弃根、带着它引用的落点");
    let defer_slots_of_the_abandoned_publish = released_above_the_floor_and_covered_by(
        &mounted.allocator,
        &recorded_for_the_abandoned_publish,
        built.abandoned_root.checkpoint_txg,
    );
    assert!(
        !defer_slots_of_the_abandoned_publish.is_empty(),
        "C 引用的槽里有现行账已释放、释放代高于 F 的（阳性对照：这一形走到了被抛弃根引用的 defer 槽）"
    );
    let free_defer_slots_of_the_abandoned_publish_at_the_mount: Vec<&(DeviceIdentity, SlotNumber)> =
        defer_slots_of_the_abandoned_publish
            .iter()
            .filter(|(device, slot)| {
                mounted
                    .allocator
                    .devices
                    .iter()
                    .any(|device_map| device_map.device == *device && device_map.is_free(*slot))
            })
            .collect();
    assert!(
        free_defer_slots_of_the_abandoned_publish_at_the_mount.is_empty(),
        "挂载那一刻这几个 defer 槽分配位置着、不空闲：空闲的 {free_defer_slots_of_the_abandoned_publish_at_the_mount:?}"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
        0,
        "挂载那一刻没有一个槽同时算进「已分配」与第九项（D28（挂载期承诺量） 已定项 1 第九项：被抛弃根引用的 defer 槽算在「已分配」里）"
    );
    let mut pool = pool_under_test_after_the_mount(
        &built.parameters,
        built.device_bytes,
        devices,
        mounted,
        built.write_time_seconds + 3000,
    );
    let key_of_the_abandoned_publish = (
        built.abandoned_root.instance,
        built.abandoned_root.checkpoint_txg,
    );
    let mut a_defer_slot_was_reclaimed_and_isolated_while_the_abandoned_publish_was_in_the_ring =
        false;
    let mut overwrites = 0;
    // 迭代上界 OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST；跨轮带计数与「回收补隔离走到了没有」；出口：C 离开根环，
    // 被拒、双扣、C 引用的槽空闲、checker 红都当场判红（panic）。
    while root_is_in_the_ring(&pool.image(), key_of_the_abandoned_publish) {
        assert!(
            overwrites < OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST,
            "{OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST} 次覆盖写之内 C 那一槽写掉、C 离开根环"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {} 次覆盖写被拒：{refusal:?}", overwrites + 1));
        overwrites += 1;
        let allocator = &pool.session().allocator;
        assert_eq!(
            slots_counted_both_as_allocated_and_as_isolated(allocator),
            0,
            "第 {overwrites} 次覆盖写之后没有一个槽同时算进「已分配」与第九项"
        );
        if root_is_in_the_ring(&pool.image(), key_of_the_abandoned_publish) {
            let free_among_the_placements_of_the_abandoned_publish =
                free_slots_covered_by(allocator, &recorded_for_the_abandoned_publish);
            assert!(
                free_among_the_placements_of_the_abandoned_publish.is_empty(),
                "第 {overwrites} 次覆盖写之后 C 还在根环里：它引用的槽一个都不空闲，空闲的 {free_among_the_placements_of_the_abandoned_publish:?}"
            );
            a_defer_slot_was_reclaimed_and_isolated_while_the_abandoned_publish_was_in_the_ring |=
                defer_slots_of_the_abandoned_publish
                    .iter()
                    .any(|(device, slot)| {
                        allocator.devices.iter().any(|device_map| {
                            device_map.device == *device && device_map.is_isolated(*slot)
                        })
                    });
        }
        images.hand_out(
            &format!("第 {overwrites} 次覆盖写之后 C 引用的单元一个都没复用"),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
    assert!(
        a_defer_slot_was_reclaimed_and_isolated_while_the_abandoned_publish_was_in_the_ring,
        "C 离环之前有一个 defer 槽被回收、照根环表补上了隔离（阳性对照：走到了回收时补隔离那一处）"
    );
}

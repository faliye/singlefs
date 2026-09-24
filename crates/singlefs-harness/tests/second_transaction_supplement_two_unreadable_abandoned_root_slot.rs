//! 里程碑「第二个事务」增补 2 收口表第 27 行点名的那一笔：**被抛弃根的根槽自己读不出时，影子账既不隔离也不计数**。
//!
//! 影子账（`mount::isolate_slots_referenced_only_by_abandoned_roots`）的输入是 `recovery::readable_roots`——
//! 读得出的那些根。一条被抛弃根的**根槽自己**读不出时它压根不在这份输入里：它独占的那几个槽进不了隔离集，
//! `MountOutput::abandoned_roots_unreadable`（那一笔数的是「根读得出、账读不出」）也不加一。
//! 于是这几个槽在下一次分配里发得出去，而那条被抛弃根还引用着它们。
//!
//! 只供测试的开关 `RootRingSlotTarget` 点名 (区域, 槽)、让那个槽上**每一次**读都失败，走得到这一格。
//! 两条臂只差一个点名集合：不点名（阳性对照，证明这几个槽本来罩得住、而且这层包装自己什么都不改）、
//! 点名实例 1 第二次发布那条根的槽（暴露面）。
//!
//! ⚠️ 这一格该是什么样还没有条款（隔离不了要不要拒绝挂载、要不要另立一个计数、要不要按最坏整段隔离），
//! 用例只钉住今天的行为与暴露面，不钉它该是什么样。

mod common;

use common::{build_pool, geometry, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mount::{mount_rollback, mount_writable, RollbackTarget, ShadowLedger};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_format::UNIT_AREA_START_SLOT;
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, NamedRootRingSlots, RootRingSlotTarget,
    SharedFaultPlan,
};

/// 这一轮的回退里，实例 1 第二次发布那条根所在的根环槽读不读得出。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AbandonedRootSlotReadability {
    /// 阳性对照：同一层故障注入包装，点名集合为空，一次都不注入。
    Readable,
    /// 暴露面：那个槽上每一次读都返回块设备错，持续，不是一次瞬时错。
    UnreadableOnEveryRead,
}

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let previous = pool.output.clone();
    let output = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        instance,
    )
    .expect("覆盖写");
    pool.output = output.clone();
    output
}

/// 回退那一次挂载报出来的东西，加上它算出来的分配器。镜像跟着 `_pool` 活到这一条用例结束。
struct RollbackFacts {
    isolated_slots_per_device: Vec<(DeviceIdentity, u64)>,
    abandoned_roots_unreadable: u64,
    /// 实例 1 第二次发布（被抛弃的那条根）新分配的落点：(起点槽, 跨度)，只取盘 0 的。
    slots_allocated_by_the_abandoned_publish: Vec<(u64, u16)>,
    allocator: PoolAllocator,
    /// 点名的槽上注入了几次：阳性对照恒 0，暴露面 ≥ 1。
    faults_fired: usize,
    _pool: BuiltPool,
}

/// 固定脚本到回退：A、B（实例 1 的第二次发布，txg 4）、重开取号 2、写行、暖机两次、C，再重开回退到 (1, 3)。
/// 回退把 B 与实例 2 的全部根抛弃；回退这一次的设备包一层故障注入，按 `readability` 决定点不点名 B 的根槽。
fn rollback_with(tag: &str, readability: AbandonedRootSlotReadability) -> RollbackFacts {
    let mut pool = build_pool(tag);
    let abandoned_publish =
        overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    let reopened = pool.reopen_recorded();

    let named_slots = match readability {
        AbandonedRootSlotReadability::Readable => NamedRootRingSlots::NONE,
        AbandonedRootSlotReadability::UnreadableOnEveryRead => {
            NamedRootRingSlots::NONE.with(target_for_publish(
                abandoned_publish.root.checkpoint_txg,
                parameters().geometry.root_ring_slots_per_region,
            ))
        }
    };
    let plan = SharedFaultPlan::unarmed(geometry());
    plan.arm(FaultSchedule::every_read_of_named_root_ring_slots_fails(
        RootRingSlotTarget {
            named_slots,
            region_devices: parameters().region_devices,
            fixed_structure_slot_spacing: parameters().geometry.fixed_structure_slot_spacing,
        },
    ));
    let mut wrapped: Vec<(DeviceIdentity, FaultInjectingBlockDevice<Recorded>)> = reopened
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, device, plan.clone()),
            )
        })
        .collect();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut wrapped,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        },
        ShadowLedger::On,
    )
    .expect("回退");
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner()))
            .collect(),
    );
    // 回退之后现行的那一版：接着在这次挂载里发布的用例要从它往下接。
    pool.output = rolled_back
        .current
        .clone()
        .into_file_version()
        .expect("回退到 A，现行那一版带文件");
    RollbackFacts {
        isolated_slots_per_device: rolled_back.output.isolated_slots_per_device.clone(),
        abandoned_roots_unreadable: rolled_back.output.abandoned_roots_unreadable,
        slots_allocated_by_the_abandoned_publish: abandoned_publish
            .allocation_records
            .iter()
            .filter(|record| {
                record.device == DeviceIdentity(0)
                    && record.generation == abandoned_publish.root.checkpoint_txg
                    && !record.is_released
            })
            .map(|record| (record.slot.0, record.span_slots))
            .collect(),
        allocator: rolled_back.allocator,
        faults_fired: plan.fired_count(),
        _pool: pool,
    }
}

/// 两条臂里 `is_free` 判得不一样的槽：一条臂罩着、另一条臂发得出去的就是这些。
fn slots_whose_freedom_differs(left: &PoolAllocator, right: &PoolAllocator) -> Vec<u64> {
    let unit_area_slots = left.devices[0].unit_area_slots();
    assert_eq!(
        unit_area_slots,
        right.devices[0].unit_area_slots(),
        "两条臂的单元区一样大"
    );
    (0..unit_area_slots)
        .map(|offset| SlotNumber(UNIT_AREA_START_SLOT + offset))
        .filter(|slot| left.devices[0].is_free(*slot) != right.devices[0].is_free(*slot))
        .map(|slot| slot.0)
        .collect()
}

/// 收口表第 27 行「被抛弃根的根槽读不出时既不隔离也不计数」：一条被抛弃根的根槽持续读不出时，
/// 只有它引用的那四个槽从隔离集里掉出来（发得出去了），而报出来的读不出计数仍然是 0——
/// 两半都没人罩，外面看不出少罩了几个槽。
#[test]
fn an_abandoned_root_whose_own_root_slot_is_unreadable_is_neither_isolated_nor_counted() {
    let readable = rollback_with(
        "unreadable-abandoned-root-slot-readable",
        AbandonedRootSlotReadability::Readable,
    );
    let unreadable = rollback_with(
        "unreadable-abandoned-root-slot-unreadable",
        AbandonedRootSlotReadability::UnreadableOnEveryRead,
    );

    // 阳性对照：点名集合为空时一次都不注入，这条臂与不包装走的是同一条路。
    assert_eq!(
        readable.faults_fired, 0,
        "点名集合为空：这层包装自己什么都不改"
    );
    assert!(
        unreadable.faults_fired >= 1,
        "点名的槽上真的注入过：{} 次（这一段回退读那个槽几次不是这条用例要钉的量，只要求它大于 0）",
        unreadable.faults_fired
    );

    assert_eq!(
        readable.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 34), (DeviceIdentity(1), 34)],
        "根槽都读得出时影子账罩住 34 个槽：B 与实例 2 那几条被抛弃根引用、而候选集与 A 这一版账里都不引用的那些"
    );
    assert_eq!(
        unreadable.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 30), (DeviceIdentity(1), 30)],
        "B 的根槽读不出：它独占的四个槽掉出隔离集，两块盘各少罩 4 个"
    );
    assert_eq!(
        unreadable.abandoned_roots_unreadable, 0,
        "「不计数」那一半：这个计数只数「根读得出、账读不出」，根槽自己读不出的一条都不算"
    );
    assert_eq!(
        readable.abandoned_roots_unreadable, 0,
        "阳性对照这条臂本来就没有读不出的账"
    );

    // 掉出来的是哪几个槽：按 is_free 逐槽比，与上面两个计数是两条独立的读法。
    let differing_slots = slots_whose_freedom_differs(&readable.allocator, &unreadable.allocator);
    assert_eq!(
        differing_slots,
        vec![50253, 50254, 50255, 50256],
        "B 独占的四个一槽单元：根槽读得出时罩着，读不出时发得出去"
    );
    assert_eq!(
        u64::try_from(differing_slots.len()).expect("四个"),
        readable.isolated_slots_per_device[0].1 - unreadable.isolated_slots_per_device[0].1,
        "逐槽比出来的差与报出来的隔离计数之差对得上"
    );
    for slot in &differing_slots {
        assert!(
            !readable.allocator.devices[0].is_free(SlotNumber(*slot)),
            "根槽读得出时 {slot} 罩着"
        );
        assert!(
            unreadable.allocator.devices[0].is_free(SlotNumber(*slot)),
            "根槽读不出时 {slot} 发得出去，而 B 的根还引用着它"
        );
        assert!(
            readable
                .slots_allocated_by_the_abandoned_publish
                .contains(&(*slot, 1)),
            "{slot} 正是 B 那次发布新分配的一槽单元，不是别处来的：{:?}",
            readable.slots_allocated_by_the_abandoned_publish
        );
    }
    assert_eq!(
        unreadable.slots_allocated_by_the_abandoned_publish,
        readable.slots_allocated_by_the_abandoned_publish,
        "两条臂到回退之前走的是同一条脚本：B 那次发布分配的落点一样"
    );
}

/// 接着回退那次挂载做下去：分配器换成回退交回的那一个（现行版本 `rollback_with` 已经接好），停在 txg 10。
fn continue_in_the_rollback_mount(facts: &mut RollbackFacts) {
    facts._pool.allocator = facts.allocator.clone();
    assert_eq!(
        facts._pool.output.root.checkpoint_txg,
        CheckpointTxg(10),
        "回退那次挂载做完停在 txg 10"
    );
}

/// 在回退那次挂载里接着连续覆盖写（实例 3），直到现行版本的 txg 到 `last_txg`，交回这一回发的每一版。
/// 覆盖写的内容按 txg 现算：同一个 txg 上写的内容与分几回调无关。
fn overwrite_after_the_rollback_up_to(
    facts: &mut RollbackFacts,
    last_txg: CheckpointTxg,
) -> Vec<(CheckpointTxg, TransactionOutput)> {
    let mut published = Vec::new();
    while facts._pool.output.root.checkpoint_txg < last_txg {
        let steps = usize::try_from(facts._pool.output.root.checkpoint_txg.0 - 10)
            .expect("回退那次挂载停在 txg 10，之后每一步加一");
        let output = overwrite_in_process(
            &mut facts._pool,
            &content_of(2000 + steps, steps + 41),
            InstanceGeneration(3),
        );
        published.push((output.root.checkpoint_txg, output));
    }
    published
}

/// B 那次发布独占的四个一槽单元（它的四个固定点）：候选集与 A 这一版账里都不引用，回退时影子账隔离着。
const SLOTS_ONLY_THE_ABANDONED_ROOT_B_REFERENCES: [u64; 4] = [50253, 50254, 50255, 50256];

/// 收口表第 27 行「alloc-basis 第三轮转来的四条」里的第 ④ 条，C503（隔离位清零的时机条文与实现说反话） 用户 2026-09-23 定改代码：
/// D28（挂载期承诺量） 已定项 1 第九项「被抛弃的根被轮转覆写时清零」。被抛弃根 B（txg 4）的根槽在同一次挂载里被 txg 28 的根盖掉，
/// 那一次发布里清掉**只有 B 撑着**的隔离位：B 独占的四个固定点，加 txg 27 那次挂载内回收时因 B 还引用着而补隔离的 mkfs 那片实例表
/// （两槽，见 `a_slot_reclaimed_while_an_abandoned_root_still_references_it_stays_isolated_until_that_root_leaves_the_ring`）——
/// 两块盘各从 36 掉回 30；B 与实例 2 那几条根共用的槽（实例 2 那几条还在环里）照旧隔离着。
/// 先用纯算术钉住轮转真的发生了（txg 28 的落点就是 txg 4 那个槽），再看那几个槽：盖掉之前一个都发不出去，盖掉之后都回到空闲、没有新记录。
#[test]
fn the_isolation_bits_only_the_abandoned_root_holds_are_cleared_by_the_publish_that_overwrites_its_root_slot(
) {
    let mut facts = rollback_with(
        "isolation-cleared-by-rotation",
        AbandonedRootSlotReadability::Readable,
    );
    assert_eq!(
        target_for_publish(
            CheckpointTxg(28),
            parameters().geometry.root_ring_slots_per_region,
        ),
        target_for_publish(
            CheckpointTxg(4),
            parameters().geometry.root_ring_slots_per_region,
        ),
        "txg 28 的根落在 txg 4（被抛弃的 B）那个槽上：轮转真的覆写得到它"
    );
    assert_eq!(
        facts
            .allocator
            .devices
            .iter()
            .map(|device_map| device_map.isolated_slots())
            .collect::<Vec<_>>(),
        vec![34, 34],
        "回退那一刻影子账两块盘各罩住 34 个槽"
    );
    continue_in_the_rollback_mount(&mut facts);
    let up_to_txg_27 = overwrite_after_the_rollback_up_to(&mut facts, CheckpointTxg(27));
    assert_eq!(
        up_to_txg_27.len(),
        17,
        "同一次挂载里连续覆盖写 17 次，从 txg 10 走到 27"
    );
    let isolated_before_the_overwrite: Vec<u64> = facts
        ._pool
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.isolated_slots())
        .collect();
    assert_eq!(
        isolated_before_the_overwrite,
        vec![36, 36],
        "盖掉 B 的根槽之前：34 个，加 txg 27 那次回收时补隔离的 mkfs 实例表两槽"
    );
    for slot in SLOTS_ONLY_THE_ABANDONED_ROOT_B_REFERENCES {
        for device_map in &facts._pool.allocator.devices {
            assert!(
                !device_map.is_free(SlotNumber(slot)),
                "盘 {:?}：B 的根槽还在环里时 {slot} 隔离着",
                device_map.device
            );
        }
    }

    let the_overwrite_of_b = overwrite_after_the_rollback_up_to(&mut facts, CheckpointTxg(28));
    assert_eq!(
        the_overwrite_of_b
            .iter()
            .map(|(txg, _)| *txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(28)],
        "只多发 txg 28 这一次：盖掉 B 根槽的那一次"
    );
    assert_eq!(
        facts
            ._pool
            .allocator
            .devices
            .iter()
            .map(|device_map| device_map.isolated_slots())
            .collect::<Vec<_>>(),
        vec![30, 30],
        "B 的根槽被盖掉那一次发布里，只有 B 撑着的六个隔离位（四个固定点、mkfs 实例表两槽）两块盘各清掉"
    );
    for slot in SLOTS_ONLY_THE_ABANDONED_ROOT_B_REFERENCES {
        for device_map in &facts._pool.allocator.devices {
            assert!(
                device_map.is_free(SlotNumber(slot)),
                "盘 {:?}：B 离开根环之后 {slot} 回到空闲",
                device_map.device
            );
            assert_eq!(
                facts
                    ._pool
                    .allocator
                    .record_for(device_map.device, SlotNumber(slot)),
                None,
                "{slot} 在现行账里从没分配过：清掉隔离位之后它就是空槽"
            );
        }
    }
}

/// C518（一次挂载之内环转过一圈之后不回收） 修好之后那一格：挂载内回收出来的槽，被抛弃的根还引用着的不许发出去
/// （D23（journal 的角色与格式） 已定项 14 主句：被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配）。
/// mkfs 那片实例表（50176 两槽）在回退那次写行（txg 9）被换下、进 defer；A（txg 3）是最后一条引用它的候选根，txg 27 盖掉 A 之后
/// 环里最旧有效根变成 9，它按谓词回收——可 B（txg 4，被抛弃，照抄着同一片实例表）还在环里，于是当场补隔离；
/// txg 28 盖掉 B 之后它才回到空闲，txg 29 的数据单元就落在那里。
#[test]
fn a_slot_reclaimed_while_an_abandoned_root_still_references_it_stays_isolated_until_that_root_leaves_the_ring(
) {
    let mut facts = rollback_with(
        "reclaimed-while-abandoned-root-references",
        AbandonedRootSlotReadability::Readable,
    );
    let mkfs_instance_table = singlefs_core::make_filesystem::INSTANCE_TABLE_SLOT;
    continue_in_the_rollback_mount(&mut facts);
    overwrite_after_the_rollback_up_to(&mut facts, CheckpointTxg(26));
    let deferred_before_txg_27 = facts._pool.allocator.devices[0].deferred_slots();
    assert_eq!(
        facts
            ._pool
            .allocator
            .record_for(DeviceIdentity(0), mkfs_instance_table)
            .map(|record| (record.generation, record.is_released)),
        Some((CheckpointTxg(9), true)),
        "mkfs 那片实例表在回退那次写行（txg 9）被换下"
    );
    let released_by_the_rollback_row_publish: u64 = facts
        ._pool
        .allocator
        .records()
        .iter()
        .filter(|record| {
            record.device == DeviceIdentity(0)
                && record.is_released
                && record.generation == CheckpointTxg(9)
        })
        .map(|record| u64::from(record.span_slots))
        .sum();
    assert_eq!(
        released_by_the_rollback_row_publish, 6,
        "回退那次写行（txg 9）换下 A 那一版的实例表（两槽）与四个固定点"
    );
    let up_to_txg_27 = overwrite_after_the_rollback_up_to(&mut facts, CheckpointTxg(27));
    let (_, txg_27) = up_to_txg_27.last().expect("发了 txg 27");
    let released_by_txg_27: u64 = txg_27.released.iter().map(|placement| placement.span).sum();
    assert_eq!(
        facts._pool.allocator.devices[0].deferred_slots() + released_by_the_rollback_row_publish,
        deferred_before_txg_27 + released_by_txg_27,
        "txg 27 盖掉 A 之后环里最旧有效根是 9：释放代 9 的六个槽（mkfs 实例表那两槽在内）按谓词回收、离开 defer 队列"
    );
    for device_map in &facts._pool.allocator.devices {
        for slot in [mkfs_instance_table.0, mkfs_instance_table.0 + 1] {
            assert!(
                !device_map.is_free(SlotNumber(slot)),
                "盘 {:?}：{slot} 回收了，但 B 还在环里引用它，补了隔离",
                device_map.device
            );
        }
    }
    let up_to_txg_29 = overwrite_after_the_rollback_up_to(&mut facts, CheckpointTxg(29));
    let (_, txg_29) = up_to_txg_29.last().expect("发了 txg 29");
    assert_eq!(
        txg_29
            .unit(singlefs_core::transaction::TransactionUnit::Data(
                DataUnitIndexInFile::FIRST,
            ))
            .slot,
        mkfs_instance_table,
        "B 离开根环之后那两槽回到空闲，txg 29 的数据单元落回 50176"
    );
}

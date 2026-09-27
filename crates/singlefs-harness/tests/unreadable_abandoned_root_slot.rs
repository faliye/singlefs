//! 里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 27 行点名的那一笔：**被抛弃根的根槽自己读不出时，影子账既不隔离也不计数**。
//!
//! 影子账（`mount::isolate_slots_referenced_only_by_abandoned_roots`）的输入是 `recovery::readable_roots`——
//! 读得出的那些根。一条被抛弃根的**根槽自己**读不出时它压根不在这份输入里：它独占的那几个槽进不了隔离集，
//! `MountOutput::abandoned_roots_unreadable`（那一笔数的是「根读得出、账读不出」）也不加一。
//! 于是这几个槽在下一次分配里发得出去，而那条被抛弃根还引用着它们。
//!
//! 被抛弃的根由崩溃恢复造出（管理员回退改成挂着时的向前发布之后不抛弃任何根，D23（journal 的角色与格式） 已定项 14）：
//! `common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 抛弃 C（txg 8），之后再重开可写挂载一次。
//! 只供测试的开关 `RootRingSlotTarget` 点名 (区域, 槽)、让那个槽上**每一次**读都失败，走得到这一格。
//! 两条臂只差一个点名集合：不点名（阳性对照，证明这几个槽本来罩得住、而且这层包装自己什么都不改）、
//! 点名 C 那条根的槽（暴露面）。
//!
//! C554 乙（用户 2026-09-27 定：可写挂载读到的样子里有更新的东西读不出就重读一次，仍读不出拒可写）罩不到这一格：判据 N-配置
//! 只问「系统配置有没有见证过比所选那一版新的发布」，C 比重开时所选的那一版旧，判据为假、不重读（E158 第 3 次跑登记 PC-554 那一句写的就是
//! 这个结局）。C 也只能是系统配置没见证到的根——见证过的最新根暂时读不出，C554 乙在崩溃恢复那次挂载就拒可写，抛弃不了它
//! （`common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before` 因此让见证它的系统配置槽坏掉）。
//! ⚠️ 被抛弃根的根槽读不出这一格该是什么样还没有条款（隔离不了要不要拒绝挂载、要不要另立一个计数、要不要按最坏整段隔离），
//! 用例钉住乙之后照旧的行为与暴露面，不钉它该是什么样。
mod common;

use common::{
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before, build_pool, geometry,
    parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mount::{
    mount_writable, NewerPublishWitness, ReadStageSettled, RollbackTarget,
    SelectedVersionAgainstTheWitness, WitnessedCounterComparison,
};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::{publish_overwrite, FirstFile, PoolWriter, TransactionOutput};
use singlefs_format::UNIT_AREA_START_SLOT;
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, NamedRootRingSlots, RootRingSlotTarget,
    SharedFaultPlan,
};

/// 这一轮重开挂载里，被抛弃的根 C 所在的根环槽读不读得出。
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

/// 崩溃恢复抛弃 C 之后那一次重开挂载报出来的东西，加上它算出来的分配器。镜像跟着 `_pool` 活到这一条用例结束。
struct RemountFacts {
    isolated_slots_per_device: Vec<(DeviceIdentity, u64)>,
    abandoned_roots_unreadable: u64,
    /// C（被抛弃的那条根）那次发布新分配的落点：(起点槽, 跨度)，只取盘 0 的。
    slots_allocated_by_the_abandoned_publish: Vec<(u64, u16)>,
    /// 实例 2 写行那次（txg 5）写出的实例表那片的起点槽：实例 2 的四条根与 C 都照抄它，实例 3 写行（txg 9）把它换下。
    instance_table_slot_of_the_second_instance: SlotNumber,
    allocator: PoolAllocator,
    /// 点名的槽上注入了几次：阳性对照恒 0，暴露面 ≥ 1。
    faults_fired: usize,
    /// 重开那一次挂载的读阶段在哪一遍判完（C554 乙）。
    read_stage: ReadStageSettled,
    _pool: BuiltPool,
}

/// 固定脚本到崩溃恢复抛弃 C：A、B（实例 1）、重开取号 2、写行（txg 5）、暖机两次（6、7）、C（txg 8）；
/// `common::abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before`：C 的根槽与数据单元暂时读不出，恢复落到 (2, 7)，
/// 实例 3 写行与暖机（txg 9、10），再把 C 写回——C 读得出、按实例 3 的实例表判被抛弃。之后再重开可写挂载一次（实例 4），
/// 这一次的设备包一层故障注入，按 `readability` 决定点不点名 C 的根槽。
fn remount_after_a_recovery_abandoned_the_third_version(
    tag: &str,
    readability: AbandonedRootSlotReadability,
) -> RemountFacts {
    let mut pool = build_pool(tag);
    overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1));
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    let instance_table_slot_of_the_second_instance =
        pool.output.root.instance_table.locations[0].slot;
    let abandoned_publish =
        overwrite_in_process(&mut pool, &content_of(2500, 11), InstanceGeneration(2));
    abandon_the_newest_root_by_a_recovery_that_lands_on_the_root_before(
        &mut pool,
        &abandoned_publish,
    );
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
    let remounted = mount_writable(&parameters(), &mut wrapped).expect("重开可写挂载");
    assert_eq!(remounted.output.instance, InstanceGeneration(4));
    pool.devices = Some(
        wrapped
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner()))
            .collect(),
    );
    // 重开之后现行的那一版：接着在这次挂载里发布的用例要从它往下接。
    pool.output = remounted
        .current
        .clone()
        .into_file_version()
        .expect("落到带文件的一版上，现行那一版带文件");
    RemountFacts {
        isolated_slots_per_device: remounted.output.isolated_slots_per_device.clone(),
        abandoned_roots_unreadable: remounted.output.abandoned_roots_unreadable,
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
        instance_table_slot_of_the_second_instance,
        allocator: remounted.allocator,
        faults_fired: plan.fired_count(),
        read_stage: remounted.output.rereads.read_stage,
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

/// 接着重开那次挂载做下去：分配器换成挂载交回的那一个（现行版本 `remount_after_a_recovery_abandoned_the_third_version` 已经接好）。
fn continue_in_the_remount(facts: &mut RemountFacts) {
    facts._pool.allocator = facts.allocator.clone();
}

/// 在重开那次挂载里接着连续覆盖写（实例 4），直到现行版本的 txg 到 `last_txg`，交回这一回发的每一版。
/// 覆盖写的内容按 txg 现算：同一个 txg 上写的内容与分几回调无关。
fn overwrite_after_the_remount_up_to(
    facts: &mut RemountFacts,
    last_txg: CheckpointTxg,
) -> Vec<(CheckpointTxg, TransactionOutput)> {
    let mut published = Vec::new();
    while facts._pool.output.root.checkpoint_txg < last_txg {
        let steps = usize::try_from(facts._pool.output.root.checkpoint_txg.0).expect("txg");
        let output = overwrite_in_process(
            &mut facts._pool,
            &content_of(2000 + steps, steps + 41),
            InstanceGeneration(4),
        );
        published.push((output.root.checkpoint_txg, output));
    }
    published
}

/// 被抛弃根 C 那次发布独占的槽（它的数据单元 50184–50185，加分配记录树的节点、记账树、映射树、树表那几片；跨 2 槽的单元两槽都在内）：
/// 实例 4 那一版账里不引用，重开时影子账隔离着。
const SLOTS_ONLY_THE_ABANDONED_THIRD_VERSION_REFERENCES: [u64; 14] = [
    50184, 50185, 50330, 50332, 50333, 50334, 50335, 50336, 50337, 50338, 50339, 50340, 50341,
    50342,
];

/// 收口表第 27 行「被抛弃根的根槽读不出时既不隔离也不计数」，C554 乙之后照旧：一条被抛弃根的根槽持续读不出时，
/// 只有它引用的那几个槽从隔离集里掉出来（发得出去了），而报出来的读不出计数仍然是 0——
/// 两半都没人罩，外面看不出少罩了几个槽。被抛弃的根由崩溃恢复造出（C，txg 8，系统配置没见证到它），两条臂只差重开那一次点不点名 C 的根槽。
/// 重开那一次两条臂的读阶段都在第一遍判完：所选那一版 (3, 10) 就是系统配置见证到的那次发布（jsn 10），C 比它旧，C554 乙不重读。
#[test]
fn an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_is_neither_isolated_nor_counted_when_its_root_slot_is_unreadable(
) {
    let readable = remount_after_a_recovery_abandoned_the_third_version(
        "unreadable-abandoned-root-slot-readable",
        AbandonedRootSlotReadability::Readable,
    );
    let unreadable = remount_after_a_recovery_abandoned_the_third_version(
        "unreadable-abandoned-root-slot-unreadable",
        AbandonedRootSlotReadability::UnreadableOnEveryRead,
    );

    let settled_on_the_first_read = ReadStageSettled::OnTheFirstRead {
        first_read: SelectedVersionAgainstTheWitness {
            selected_version: RollbackTarget {
                instance: InstanceGeneration(3),
                checkpoint_txg: CheckpointTxg(10),
            },
            witness: NewerPublishWitness {
                witnessed_journal_counter: 10,
                comparison: WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                    selected_version_last_record_counter: 10,
                },
            },
        },
    };
    assert_eq!(
        (readable.read_stage, unreadable.read_stage),
        (settled_on_the_first_read, settled_on_the_first_read),
        "两条臂重开时都落到 (3, 10)、系统配置见证到的就是它（jsn 10），C554 乙不重读：C 的根槽读不读得出它都不管"
    );

    // 阳性对照：点名集合为空时一次都不注入，这条臂与不包装走的是同一条路。
    assert_eq!(
        readable.faults_fired, 0,
        "点名集合为空：这层包装自己什么都不改"
    );
    assert!(
        unreadable.faults_fired >= 1,
        "点名的槽上真的注入过：{} 次（这一次挂载读那个槽几次不是这条用例要钉的量，只要求它大于 0）",
        unreadable.faults_fired
    );

    assert_eq!(
        readable.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 14), (DeviceIdentity(1), 14)],
        "根槽读得出时影子账罩住 14 个槽：C 那次发布独占的那些"
    );
    assert_eq!(
        unreadable.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "C 的根槽读不出：它独占的 14 个槽全掉出隔离集"
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
        SLOTS_ONLY_THE_ABANDONED_THIRD_VERSION_REFERENCES.to_vec(),
        "C 独占的那几个槽：根槽读得出时罩着，读不出时发得出去"
    );
    assert_eq!(
        u64::try_from(differing_slots.len()).expect("十四个"),
        readable.isolated_slots_per_device[0].1 - unreadable.isolated_slots_per_device[0].1,
        "逐槽比出来的差与报出来的隔离计数之差对得上"
    );
    let slots_of_the_abandoned_publish: Vec<u64> = readable
        .slots_allocated_by_the_abandoned_publish
        .iter()
        .flat_map(|(slot, span)| *slot..*slot + u64::from(*span))
        .collect();
    for slot in &differing_slots {
        assert!(
            !readable.allocator.devices[0].is_free(SlotNumber(*slot)),
            "根槽读得出时 {slot} 罩着"
        );
        assert!(
            unreadable.allocator.devices[0].is_free(SlotNumber(*slot)),
            "根槽读不出时 {slot} 发得出去，而 C 的根还引用着它"
        );
        assert!(
            slots_of_the_abandoned_publish.contains(slot),
            "{slot} 正是 C 那次发布新分配的，不是别处来的：{:?}",
            readable.slots_allocated_by_the_abandoned_publish
        );
    }
    assert_eq!(
        unreadable.slots_allocated_by_the_abandoned_publish,
        readable.slots_allocated_by_the_abandoned_publish,
        "两条臂到重开之前走的是同一条脚本：C 那次发布分配的落点一样"
    );
}

/// 收口表第 27 行「alloc-basis 第三轮转来的四条」里的第 ④ 条，C503（隔离位清零的时机条文与实现说反话） 用户 2026-09-23 定改代码：
/// D28（挂载期承诺量） 已定项 1 第九项「被抛弃的根被轮转覆写时清零」。被抛弃根 C（txg 8）的根槽在同一次挂载里被 txg 32 的根盖掉，
/// 那一次发布里清掉**只有 C 撑着**的隔离位：C 独占的 14 槽，加 txg 31 那次挂载内回收时因 C 还引用着而补隔离的实例 2 那片实例表
/// （两槽，见 `slot_reclaimed_while_an_abandoned_root_still_references_it_stays_isolated_until_that_root_leaves_the_ring`）——
/// 两块盘各从 16 掉回 0。先用纯算术钉住轮转真的发生了（txg 32 的落点就是 txg 8 那个槽），再看那几个槽：盖掉之前一个都发不出去，
/// 盖掉之后都回到空闲、没有新记录。
#[test]
fn the_isolation_bits_only_the_abandoned_root_holds_are_cleared_by_the_publish_that_overwrites_its_root_slot(
) {
    let mut facts = remount_after_a_recovery_abandoned_the_third_version(
        "isolation-cleared-by-rotation",
        AbandonedRootSlotReadability::Readable,
    );
    assert_eq!(
        target_for_publish(
            CheckpointTxg(32),
            parameters().geometry.root_ring_slots_per_region,
        ),
        target_for_publish(
            CheckpointTxg(8),
            parameters().geometry.root_ring_slots_per_region,
        ),
        "txg 32 的根落在 txg 8（被抛弃的 C）那个槽上：轮转真的覆写得到它"
    );
    assert_eq!(
        facts
            .allocator
            .devices
            .iter()
            .map(|device_map| device_map.isolated_slots())
            .collect::<Vec<_>>(),
        vec![14, 14],
        "重开那一刻影子账两块盘各罩住 14 个槽"
    );
    assert_eq!(
        facts._pool.output.root.checkpoint_txg,
        CheckpointTxg(13),
        "重开那次挂载写行 txg 11、暖机 12、13"
    );
    continue_in_the_remount(&mut facts);
    let up_to_txg_31 = overwrite_after_the_remount_up_to(&mut facts, CheckpointTxg(31));
    assert_eq!(
        up_to_txg_31.len(),
        18,
        "同一次挂载里连续覆盖写 18 次，从 txg 13 走到 31"
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
        vec![16, 16],
        "盖掉 C 的根槽之前：14 个，加 txg 31 那次回收时补隔离的实例 2 那片实例表两槽"
    );
    for slot in SLOTS_ONLY_THE_ABANDONED_THIRD_VERSION_REFERENCES {
        for device_map in &facts._pool.allocator.devices {
            assert!(
                !device_map.is_free(SlotNumber(slot)),
                "盘 {:?}：C 的根槽还在环里时 {slot} 隔离着",
                device_map.device
            );
        }
    }

    let the_overwrite_of_the_third_versions_root_slot =
        overwrite_after_the_remount_up_to(&mut facts, CheckpointTxg(32));
    assert_eq!(
        the_overwrite_of_the_third_versions_root_slot
            .iter()
            .map(|(txg, _)| *txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(32)],
        "只多发 txg 32 这一次：盖掉 C 根槽的那一次"
    );
    assert_eq!(
        facts
            ._pool
            .allocator
            .devices
            .iter()
            .map(|device_map| device_map.isolated_slots())
            .collect::<Vec<_>>(),
        vec![0, 0],
        "C 的根槽被盖掉那一次发布里，只有 C 撑着的十六个隔离位（独占的 14 槽、实例 2 那片实例表两槽）两块盘各清掉"
    );
    for slot in SLOTS_ONLY_THE_ABANDONED_THIRD_VERSION_REFERENCES {
        for device_map in &facts._pool.allocator.devices {
            assert!(
                device_map.is_free(SlotNumber(slot)),
                "盘 {:?}：C 离开根环之后 {slot} 回到空闲",
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
/// 实例 2 写行那次（txg 5）写出的实例表那片（50304 两槽）在实例 3 写行（txg 9）被换下、进 defer；(2, 7) 是最后一条引用它的有效根，
/// txg 31 盖掉 (2, 7) 之后环里最旧有效根变成 9，它按谓词回收——可 C（txg 8，被抛弃，照抄着同一片实例表）还在环里，于是当场补隔离；
/// txg 32 盖掉 C 之后它才回到空闲，txg 34 那次发布把一个单元落回那里。
#[test]
fn slot_reclaimed_while_an_abandoned_root_still_references_it_stays_isolated_until_that_root_leaves_the_ring(
) {
    let mut facts = remount_after_a_recovery_abandoned_the_third_version(
        "reclaimed-while-abandoned-root-references",
        AbandonedRootSlotReadability::Readable,
    );
    let instance_table_of_the_second_instance = facts.instance_table_slot_of_the_second_instance;
    continue_in_the_remount(&mut facts);
    overwrite_after_the_remount_up_to(&mut facts, CheckpointTxg(30));
    let deferred_before_txg_31 = facts._pool.allocator.devices[0].deferred_slots();
    assert_eq!(
        facts
            ._pool
            .allocator
            .record_for(DeviceIdentity(0), instance_table_of_the_second_instance)
            .map(|record| (record.generation, record.is_released)),
        Some((CheckpointTxg(9), true)),
        "实例 2 那片实例表在实例 3 写行（txg 9）被换下"
    );
    let released_by_the_third_instance_row_publish: u64 = facts
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
        released_by_the_third_instance_row_publish, 10,
        "实例 3 写行（txg 9）换下 (2, 7) 那一版的实例表（两槽）与四个固定点（分配记录树五个节点加三个角色，八槽）"
    );
    let up_to_txg_31 = overwrite_after_the_remount_up_to(&mut facts, CheckpointTxg(31));
    let (_, txg_31) = up_to_txg_31.last().expect("发了 txg 31");
    let released_by_txg_31: u64 = txg_31.released.iter().map(|placement| placement.span).sum();
    assert_eq!(
        facts._pool.allocator.devices[0].deferred_slots()
            + released_by_the_third_instance_row_publish,
        deferred_before_txg_31 + released_by_txg_31,
        "txg 31 盖掉 (2, 7) 之后环里最旧有效根是 9：释放代 9 的十个槽（实例 2 那片实例表两槽在内）按谓词回收、离开 defer 队列"
    );
    for device_map in &facts._pool.allocator.devices {
        for slot in [
            instance_table_of_the_second_instance.0,
            instance_table_of_the_second_instance.0 + 1,
        ] {
            assert!(
                !device_map.is_free(SlotNumber(slot)),
                "盘 {:?}：{slot} 回收了，但 C 还在环里引用它，补了隔离",
                device_map.device
            );
        }
    }
    overwrite_after_the_remount_up_to(&mut facts, CheckpointTxg(32));
    for device_map in &facts._pool.allocator.devices {
        assert!(
            device_map.is_free(instance_table_of_the_second_instance),
            "盘 {:?}：C 离开根环之后那两槽回到空闲",
            device_map.device
        );
    }
    overwrite_after_the_remount_up_to(&mut facts, CheckpointTxg(34));
    assert_eq!(
        facts
            ._pool
            .allocator
            .record_for(DeviceIdentity(0), instance_table_of_the_second_instance)
            .map(|record| (record.generation, record.is_released)),
        Some((CheckpointTxg(34), false)),
        "txg 34 那次发布把一个单元落回 {instance_table_of_the_second_instance:?}"
    );
}

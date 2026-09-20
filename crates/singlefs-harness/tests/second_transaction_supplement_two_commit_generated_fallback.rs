//! 里程碑「第二个事务」增补 2 收进来的 C369（提交内生块段耗尽时没有回落）：D3（空间分配） 已定项 8 ②
//! 「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」。
//! 池里没有全空的 64 槽聚簇段、开放段也用满，每块盘上却还有大片空槽：一次发布要成功，提交内生块落在回落政策函数给的槽上。
//! 回落与 bump 游标绕开同一套位（已分配、影子账隔离、抬 F 扣住），不是绕开影子账或回收扣住的第二条路。
//! 另钉增补 2 第 ② 行里「抬 F 的空发布分配不到固定点（开放段满、唯一全空段正是扣住的那一段）」补了回落之后的行为：
//! 扣住的段外面还有不被挡的空槽时抬 F 成功、落点一个都不在扣住的槽上；连一个都没有时照旧落点被拒（`PlacementRefused`，每块盘上都没有）、扣住位留在这个进程里。
//!
//! 「没有全空段」的形态直接改空闲图造（开放段剩下的槽占满、每个全空段占掉段首一槽），不写分配记录：
//! 第一版分配记录树只有一个节点（812 条），用真发布占满 3312 个段装不下。所以这些池上记账与记录对不上，这里不跑池级 checker。

mod common;

use std::collections::BTreeSet;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{PlacementRefusal, PoolAllocator, UnitFootprint};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, MountError, RaisedFloor, ShadowLedger,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{
    publish_overwrite, FirstFile, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_format::{CLUSTER_SEGMENT_SLOTS, UNIT_AREA_START_SLOT};

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

/// 同一个进程里接着现行版本覆盖写一次，错误原样交回。
fn try_overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
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
    )?;
    pool.output = output.clone();
    Ok(output)
}

/// 每块盘上把开放段剩下的槽占满，再给每个全空段占掉段首一槽：之后一个全空段都没有，每块盘上仍有大片空槽。
fn leave_no_empty_cluster_segment(allocator: &mut PoolAllocator) {
    let open = allocator.open_segment().expect("发布之后开放段还开着");
    for device_map in &mut allocator.devices {
        for slot in open.0..open.0 + CLUSTER_SEGMENT_SLOTS {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
        let full_segments = device_map.unit_area_slots() / CLUSTER_SEGMENT_SLOTS;
        for segment_index in 0..full_segments {
            if device_map.segment_is_empty(usize::try_from(segment_index).expect("段号")) {
                device_map.mark_allocated(
                    SlotNumber(UNIT_AREA_START_SLOT + segment_index * CLUSTER_SEGMENT_SLOTS),
                    1,
                );
            }
        }
    }
}

/// C369 的验收：第一个事务 A 之后开放段 [50240, 50304) 用满、别的段都不全空，覆盖写 B 要成功。
/// 回落政策函数在这张空闲图上给的落点（手算，不调分配器）：B 先释放 A 的八个落点（进 defer、槽仍占着），数据单元不受段约束、
/// 取 A 的 50180 之后最低的偶数空槽对 50182；extent 树根是第一个提交内生块，开放段装不下、没有全空段 ⇒ 回落到槽号最小的空槽 50179
/// （mkfs 的 50176–50178 与 A 的数据单元之间那个从没分配过的洞）；inode 树叶容器两槽、起点 32768 对齐，50182–50183 刚给了数据单元 ⇒ 50184；
/// 之后的一槽节点按 bump 次序接着取最低空槽 50186–50190。两块盘上的分配记录同槽、分配代 4；冷启动读回 B。
#[test]
fn publish_on_pool_without_empty_cluster_segment_falls_back_to_lowest_free_slot_on_every_device() {
    let mut pool = build_pool("supplement-two-fallback-publish");
    leave_no_empty_cluster_segment(&mut pool.allocator);
    for device_map in &pool.allocator.devices {
        assert_eq!(
            device_map.lowest_empty_segment(),
            None,
            "盘 {:?} 上还有全空段：造的形态不对",
            device_map.device
        );
        assert!(
            device_map.free_slots() > 200_000,
            "盘 {:?} 上仍有大片空槽：{}",
            device_map.device,
            device_map.free_slots()
        );
    }
    let second_content = content_of(4100, 3);
    let second = try_overwrite_in_process(&mut pool, &second_content, InstanceGeneration(1))
        .expect("没有全空段、每块盘仍有空槽：发布必须成功（C369）");
    let expected_slots = [
        (TransactionUnit::Data, 50182),
        (TransactionUnit::ExtentRoot, 50179),
        (TransactionUnit::InodeLeaf, 50184),
        (TransactionUnit::InodeRoot, 50186),
        (TransactionUnit::AllocationTree, 50187),
        (TransactionUnit::AccountingTree, 50188),
        (TransactionUnit::MappingTree, 50189),
        (TransactionUnit::TreeTable, 50190),
    ];
    for (identity, slot) in expected_slots {
        assert_eq!(
            second.unit(identity).slot,
            SlotNumber(slot),
            "{}：落点要等于回落政策函数",
            identity.tag()
        );
        for device in [DeviceIdentity(0), DeviceIdentity(1)] {
            let record = pool
                .allocator
                .record_for(device, SlotNumber(slot))
                .unwrap_or_else(|| panic!("{} 在盘 {device:?} 上没有分配记录", identity.tag()));
            assert!(!record.is_released, "{} 的记录仍分配", identity.tag());
            assert_eq!(
                record.generation,
                CheckpointTxg(4),
                "{} 的分配代",
                identity.tag()
            );
        }
    }
    assert_eq!(
        pool.allocator.open_segment(),
        None,
        "回落不开段：没有全空段可开"
    );
    assert_eq!(pool.allocator.policy_mismatches, 0);

    let reopened = pool.reopen_cold();
    assert_eq!(
        recover(&reopened, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: second_content,
        },
        "冷启动择 B 的根、从回落的落点读回第二次的内容"
    );
}

/// 回落绕开影子账隔离的槽（与 bump 游标同一套位）：没有全空段时把 50179 在两块盘上都隔离，下一个一槽的提交内生块越过它、落到
/// 50182（50180–50181 是 A 的数据单元）。
#[test]
fn fallback_skips_a_slot_isolated_by_the_shadow_ledger() {
    let mut pool = build_pool("supplement-two-fallback-isolated");
    leave_no_empty_cluster_segment(&mut pool.allocator);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        pool.allocator
            .isolate_abandoned(device, SlotNumber(50179), 1);
    }
    let placement = pool
        .allocator
        .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(4))
        .expect("隔离之外还有空槽");
    assert_eq!(
        placement.slot,
        SlotNumber(50182),
        "被隔离的 50179 不许发出去，回落取下一个不被挡的空槽"
    );
}

/// 抬 F 的历史照步 5 那条「扣住到生效」用例：A、B、重开可写挂载（实例 2）、六次覆盖写（txg 8–13）。
fn build_six_overwrites_after_a_writable_remount(tag: &str) -> BuiltPool {
    let mut pool = build_pool(tag);
    try_overwrite_in_process(&mut pool, &content_of(4100, 3), InstanceGeneration(1))
        .expect("覆盖写 B");
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    pool.allocator = mounted.allocator;
    pool.output = mounted
        .current
        .into_file_version()
        .expect("B 之后重开，现行那一版带文件");
    for seed in [31usize, 37, 41, 43, 47, 53] {
        try_overwrite_in_process(
            &mut pool,
            &content_of(2000 + seed, seed),
            InstanceGeneration(2),
        )
        .expect("覆盖写");
    }
    assert_eq!(pool.output.root.checkpoint_txg, CheckpointTxg(13));
    pool
}

fn raise_floor(pool: &mut BuiltPool, new_floor: CheckpointTxg) -> Result<RaisedFloor, MountError> {
    let mut current = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let raised = raise_rollback_floor(
        &parameters(),
        devices,
        &mut pool.allocator,
        &mut current,
        new_floor,
        ShadowLedger::On,
    );
    pool.output = current;
    raised
}

/// 增补 2 第 ② 行那一格，扣住的段外面还有不被挡的空槽：抬 F 之前开放段占满、每个全空段占掉段首一槽；抬 F 到 8 回收 A、B 与
/// 重开之后前几版释放的落点，[50240, 50304) 里只剩回收的槽 ⇒ 它是唯一的全空段、又整段扣住。补回落之前这里报 `NoSpaceFor`；
/// 今天抬 F 成功，空发布的固定点全部回落，一个都不在这次回收（扣住）的槽上；生效放开之后那一段是全池唯一的全空段。
#[test]
fn raising_the_floor_when_the_only_empty_segment_is_held_falls_back_to_slots_outside_the_hold() {
    let mut pool = build_six_overwrites_after_a_writable_remount("supplement-two-raise-falls-back");
    leave_no_empty_cluster_segment(&mut pool.allocator);
    let raised = raise_floor(&mut pool, CheckpointTxg(8))
        .expect("扣住的段外面还有不被挡的空槽：回落之后抬 F 的空发布拿得到固定点");
    let reclaimed_slots: BTreeSet<u64> = raised
        .reclaimed
        .iter()
        .flat_map(|placement| placement.slot.0..placement.slot.0 + placement.span)
        .collect();
    assert!(
        reclaimed_slots.contains(&50240),
        "A 的 extent 树根 50240 在这次回收里：{reclaimed_slots:?}"
    );
    let handed_out: Vec<(u64, u64)> = raised
        .publishes
        .iter()
        .flat_map(|publish| {
            publish
                .placements()
                .into_iter()
                .map(|placement| (publish.root.checkpoint_txg.0, placement.slot.0))
        })
        .collect();
    for (txg, slot) in &handed_out {
        assert!(
            !reclaimed_slots.contains(slot),
            "txg {txg} 的固定点落在这次回收的槽 {slot} 上：F 还没在两块盘上生效"
        );
    }
    // 回落落点手算：扣住的是 50176–50178（mkfs 两个单元）、50180–50183（A、B 的数据单元）与 [50240, 50304) 里 A、B 的提交内生块；
    // 50184–50193 是 txg 8–12 的数据单元（释放代 9–13 > 8，仍在 defer）、50194 是现行数据单元 ⇒ 不被挡的最低空槽依次是 50179、50196 起。
    let rewritten_fixed_points = [
        TransactionUnit::AllocationTree,
        TransactionUnit::AccountingTree,
        TransactionUnit::MappingTree,
        TransactionUnit::TreeTable,
    ];
    let fixed_point_slots: Vec<(u64, Vec<u64>)> = raised
        .publishes
        .iter()
        .map(|publish| {
            (
                publish.root.checkpoint_txg.0,
                rewritten_fixed_points
                    .iter()
                    .map(|identity| publish.unit(*identity).slot.0)
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        fixed_point_slots,
        vec![
            (14, vec![50179, 50196, 50197, 50198]),
            (15, vec![50199, 50200, 50201, 50202]),
            (16, vec![50203, 50204, 50205, 50206]),
        ],
        "三次空发布的固定点都落在回落政策函数给的槽上"
    );
    assert_eq!(
        pool.allocator.open_segment(),
        None,
        "空发布全走回落，没开段"
    );
    for device_map in &pool.allocator.devices {
        assert_eq!(
            device_map.lowest_empty_segment(),
            Some(SlotNumber(50240)),
            "盘 {:?}：生效放开之后，回收空的那一段是唯一的全空段",
            device_map.device
        );
        assert_eq!(device_map.empty_segments(), 1);
    }
}

/// 同一格，扣住的槽之外一个空槽都没有：抬 F 之前把每块盘上的空槽全占掉，回收出来的全是扣住的槽 ⇒ 第一次空发布照旧落点被拒（`PlacementRefused`、
/// 每块盘上都没有）、
/// 一个写都没发；扣住位留在这个进程里：记账算它们空闲，分配器却一个都发不出去。
#[test]
fn raising_the_floor_with_no_free_slot_outside_the_hold_still_fails_and_the_hold_stays_in_the_process(
) {
    let mut pool = build_six_overwrites_after_a_writable_remount("supplement-two-raise-no-slot");
    for device_map in &mut pool.allocator.devices {
        for slot in UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + device_map.unit_area_slots() {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    let operations_before = pool.stream.operations().len();
    let refused = raise_floor(&mut pool, CheckpointTxg(8));
    assert!(
        matches!(
            refused,
            Err(MountError::Publish(PublishError::PlacementRefused {
                unit: TransactionUnit::AllocationTree,
                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
            }))
        ),
        "回收出来的全是扣住的槽，第一个固定点就拿不到（每块盘上都没有：容量不够那一种）：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "报错在任何写之前"
    );
    for device_map in &pool.allocator.devices {
        assert!(
            device_map.free_slots() > 0,
            "盘 {:?}：回收的槽记账上已算空闲",
            device_map.device
        );
    }
    assert_eq!(
        pool.allocator
            .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(14)),
        Err(PlacementRefusal::NoFreeSlotOnAnyDevice),
        "扣住位没放开：之后的发布照样分配不到固定点"
    );
}

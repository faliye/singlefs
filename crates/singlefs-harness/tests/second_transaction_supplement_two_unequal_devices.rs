//! 里程碑「第二个事务」增补 2 收进来的 C368（分配器落点只看盘 0，盘不等大时断言失败）：D2（RAID 条带策略） 已定项 2「各盘不必等大」，
//! D3（空间分配） 已定项 8 第 1 条「在每一块被选中的设备上各自取该设备内」。
//! 两块不等大的盘（盘 0 4 GiB、盘 1 3 GiB 加 33 槽）：mkfs、第一个事务、可写挂载今天都接受；填到小盘单元区末尾之后，
//! 分配按设备取落点，小盘答不出就在动任何状态之前拒绝、发布报 `NoSpaceFor`，不 panic、一个写都不发。
//! 另钉两个第一版不支持、没有条款的分支（拒绝成员的名字说哪条没定）：各盘给提交内生块的去处不同（一块开段一块回落，D3 已定项 8 待办 ①）；
//! 各盘的用户数据落点不同（`Placement` 两盘同槽）。
//!
//! 「填满」直接改空闲图造，不写分配记录（分配记录树第一版一个节点装 812 条），这些池上记账与记录对不上，这里不跑池级 checker。

mod common;

use std::path::PathBuf;

use common::{
    disk_snapshot, file_content, image_path, parameters, Recorded, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::{
    AllocationRecord, CommitGeneratedDeviceAnswer, DeviceFreeMap, Placement, PlacementRefusal,
    PoolAllocator, UnitFootprint,
};
use singlefs_core::block_device::{BlockDevice, FileBackedBlockDevice, PhysicalBlockSizeInBytes};
use singlefs_core::make_filesystem::{
    make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 盘 0：4 GiB，单元区 211968 槽 = 3312 个整段。
const LARGER_DEVICE_BYTES: u64 = IMAGE_BYTES;
/// 盘 1：3 GiB 加 33 个槽，单元区 146465 槽 = 2288 个整段加 33 槽的尾巴（尾巴不算段）；journal 环 768 MiB 不超过容量的四分之一。
const SMALLER_DEVICE_BYTES: u64 = (3 << 30) + 33 * SLOT_BYTES;
const DEVICE_BYTES: [u64; 2] = [LARGER_DEVICE_BYTES, SMALLER_DEVICE_BYTES];

/// 小盘单元区末尾（不含）：50176 + 146465。
const SMALLER_UNIT_AREA_END_SLOT: u64 = 196_641;

struct UnequalPool {
    paths: Vec<PathBuf>,
    devices: Vec<(DeviceIdentity, Recorded)>,
    stream: SharedStream,
    allocator: PoolAllocator,
    output: TransactionOutput,
}

impl Drop for UnequalPool {
    fn drop(&mut self) {
        for path in &self.paths {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn open_recorded_devices(
    paths: &[PathBuf],
    stream: &SharedStream,
) -> Vec<(DeviceIdentity, Recorded)> {
    paths
        .iter()
        .zip(DEVICE_BYTES)
        .enumerate()
        .map(|(index, (path, device_bytes))| {
            let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
            let file = FileBackedBlockDevice::open_or_create(
                path,
                device_bytes,
                PhysicalBlockSizeInBytes(512),
            )
            .expect("开镜像");
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, file, stream.clone()),
            )
        })
        .collect()
}

/// 两块不等大的盘：mkfs、取号、暖机、第一个事务 A，冷启动读回 A，再可写挂载（实例 2）。每一步今天都接受，接不接受就在这里判红。
fn build_unequal_pool(tag: &str) -> UnequalPool {
    let paths: Vec<PathBuf> = (0..2u32).map(|device| image_path(tag, device)).collect();
    let stream = SharedStream::new();
    let mut formatting_process_devices = open_recorded_devices(&paths, &stream);
    let genesis = make_filesystem(&parameters(), &mut formatting_process_devices)
        .expect("盘不等大的 mkfs 今天接受：几何检查按最小的那块盘算");
    let mut allocator = PoolAllocator::new(
        formatting_process_devices
            .iter()
            .map(|(identity, device)| DeviceFreeMap::new(*identity, device.size_in_bytes()))
            .collect(),
    );
    assert_eq!(allocator.devices[0].unit_area_slots(), 211_968);
    assert_eq!(
        allocator.devices[1].unit_area_slots(),
        SMALLER_UNIT_AREA_END_SLOT - UNIT_AREA_START_SLOT
    );
    allocator.mark_format_time_units(
        Placement {
            slot: INSTANCE_TABLE_SLOT,
            span: 2,
        },
        Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1,
        },
    );
    let content = file_content();
    let first = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, &mut formatting_process_devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut writer,
            &mut allocator,
            &genesis.root,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("盘不等大的第一个事务今天接受")
    };
    assert_eq!(
        first.unit(TransactionUnit::Data).slot,
        SlotNumber(50180),
        "两块盘在低处的空闲图一样，落点与等大的池相同"
    );
    drop(formatting_process_devices);
    let mut remounted_devices = open_recorded_devices(&paths, &stream);
    assert_eq!(
        recover(&remounted_devices, JournalPolicy::Consult).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content,
        },
        "盘不等大的池冷启动读回 A"
    );
    let mounted =
        mount_writable(&parameters(), &mut remounted_devices).expect("盘不等大的可写挂载今天接受");
    UnequalPool {
        paths,
        devices: remounted_devices,
        stream,
        allocator: mounted.allocator,
        output: mounted
            .current
            .into_file_version()
            .expect("A 之后重开，现行那一版带文件"),
    }
}

/// 两块盘上把 [单元区起点, 小盘单元区末尾) 里的空槽都标成已分配，`left_free` 里的槽留着；大盘在小盘末尾之后的槽不动。
fn occupy_free_slots_below_the_smaller_unit_area_end(
    allocator: &mut PoolAllocator,
    left_free: &[u64],
) {
    for device_map in &mut allocator.devices {
        for slot in UNIT_AREA_START_SLOT..SMALLER_UNIT_AREA_END_SLOT {
            if !left_free.contains(&slot) && device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// 分配器里拒绝之后必须不变的那些量。
#[derive(Debug, PartialEq, Eq)]
struct AllocatorFingerprint {
    records: Vec<AllocationRecord>,
    open_segment: Option<SlotNumber>,
    counts_per_device: Vec<DeviceCounts>,
}

#[derive(Debug, PartialEq, Eq)]
struct DeviceCounts {
    allocated_slots: u64,
    free_slots: u64,
    deferred_slots: u64,
    isolated_slots: u64,
    free_runs: u64,
    empty_segments: u64,
}

fn fingerprint(allocator: &PoolAllocator) -> AllocatorFingerprint {
    AllocatorFingerprint {
        records: allocator.records().to_vec(),
        open_segment: allocator.open_segment(),
        counts_per_device: allocator
            .devices
            .iter()
            .map(|device_map| DeviceCounts {
                allocated_slots: device_map.allocated_slots(),
                free_slots: device_map.free_slots(),
                deferred_slots: device_map.deferred_slots(),
                isolated_slots: device_map.isolated_slots(),
                free_runs: device_map.free_runs(),
                empty_segments: device_map.empty_segments(),
            })
            .collect(),
    }
}

/// 接着现行版本覆盖写一次，错误原样交回。
fn try_overwrite(pool: &mut UnequalPool) -> Result<TransactionOutput, PublishError> {
    let publish_parameters = parameters();
    let content = vec![5u8; 4100];
    let previous = pool.output.clone();
    let mut writer = PoolWriter::new(&publish_parameters, &mut pool.devices);
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        previous.root.instance,
    )
}

/// C368 的验收：小盘单元区末尾那一对偶数槽在两块盘上都空时，用户数据按设备取、两块盘都落在那里；之后小盘一个空槽都没有、
/// 大盘在小盘末尾之后还有 1 万多个槽——用户数据、一槽节点、两槽容器都拒成「小盘满了、设备集合怎么选没有条款」，分配器一样没动；
/// 走发布路径报 `NoSpaceFor`（数据单元先分配），录制流一步都不多、分配器退回。只看盘 0 的写法在这里 panic（小盘越界）。
#[test]
fn filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking(
) {
    let mut pool = build_unequal_pool("supplement-two-unequal-fill");
    let generation = CheckpointTxg(pool.output.root.checkpoint_txg.0 + 1);
    occupy_free_slots_below_the_smaller_unit_area_end(&mut pool.allocator, &[196_638, 196_639]);
    let last_pair = pool
        .allocator
        .try_allocate_user_data(generation)
        .expect("小盘单元区末尾那一对在两块盘上都空");
    assert_eq!(
        last_pair,
        Placement {
            slot: SlotNumber(196_638),
            span: 2
        }
    );
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert!(
            pool.allocator
                .record_for(device, SlotNumber(196_638))
                .is_some(),
            "盘 {device:?} 上有这一对的分配记录"
        );
    }
    for device_map in &mut pool.allocator.devices {
        if device_map.is_free(SlotNumber(196_640)) {
            device_map.mark_allocated(SlotNumber(196_640), 1);
        }
    }
    assert_eq!(pool.allocator.devices[1].free_slots(), 0, "小盘满了");
    assert!(
        pool.allocator.devices[0].free_slots() >= 211_968 - 146_465,
        "大盘在小盘末尾之后还有空槽：{}",
        pool.allocator.devices[0].free_slots()
    );

    let before = fingerprint(&pool.allocator);
    let smaller_device_full = PlacementRefusal::SomeDevicesFullDeviceSetSelectionUndefined {
        full_devices: vec![DeviceIdentity(1)],
    };
    assert_eq!(
        pool.allocator.try_allocate_user_data(generation),
        Err(smaller_device_full.clone()),
        "用户数据"
    );
    assert_eq!(
        pool.allocator
            .try_allocate_commit_generated(UnitFootprint::OneSlot, generation),
        Err(smaller_device_full.clone()),
        "一槽节点"
    );
    assert_eq!(
        pool.allocator
            .try_allocate_commit_generated(UnitFootprint::TwoSlotsAligned, generation),
        Err(smaller_device_full),
        "两槽容器"
    );
    assert_eq!(
        fingerprint(&pool.allocator),
        before,
        "拒绝时分配器一样都没动"
    );

    let operations_before = pool.stream.operations().len();
    let refused = try_overwrite(&mut pool);
    assert!(
        matches!(
            refused,
            Err(PublishError::NoSpaceFor {
                unit: TransactionUnit::Data
            })
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "拒在任何写之前"
    );
    assert_eq!(fingerprint(&pool.allocator), before, "失败的发布退回分配器");
}

/// 各盘给提交内生块的去处不同：小盘单元区里只剩末尾三个槽、一个全空段都没有 ⇒ 小盘回落到 196638；大盘在小盘末尾之后还有全空段 ⇒
/// 开段 196672。各盘上的聚簇段要不要对齐没有条款（D3（空间分配） 已定项 8 待办 ①），拒成那个成员、分配器不动；发布里数据单元两块盘
/// 同落 196638，extent 树根在这里被拒 ⇒ `NoSpaceFor`，录制流一步都不多。
#[test]
fn devices_answering_different_places_for_a_commit_generated_block_are_refused_before_anything_is_written(
) {
    let mut pool = build_unequal_pool("supplement-two-unequal-disagree");
    let generation = CheckpointTxg(pool.output.root.checkpoint_txg.0 + 1);
    occupy_free_slots_below_the_smaller_unit_area_end(
        &mut pool.allocator,
        &[196_638, 196_639, 196_640],
    );
    let before = fingerprint(&pool.allocator);
    assert_eq!(
        before.open_segment,
        Some(SlotNumber(50_304)),
        "挂载时写行与暖机开的段还开着（已被占满）：拒绝不许把它关掉"
    );
    assert_eq!(
        pool.allocator
            .try_allocate_commit_generated(UnitFootprint::OneSlot, generation),
        Err(
            PlacementRefusal::CommitGeneratedPlacementsDifferAcrossDevicesSegmentAlignmentUndefined {
                answer_per_device: vec![
                    (
                        DeviceIdentity(0),
                        CommitGeneratedDeviceAnswer::OpenEmptySegment(SlotNumber(196_672))
                    ),
                    (
                        DeviceIdentity(1),
                        CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(SlotNumber(196_638))
                    ),
                ],
            }
        )
    );
    assert_eq!(
        fingerprint(&pool.allocator),
        before,
        "拒绝时分配器一样都没动"
    );

    let operations_before = pool.stream.operations().len();
    let refused = try_overwrite(&mut pool);
    assert!(
        matches!(
            refused,
            Err(PublishError::NoSpaceFor {
                unit: TransactionUnit::ExtentRoot
            })
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "拒在任何写之前"
    );
    assert_eq!(fingerprint(&pool.allocator), before, "失败的发布退回分配器");
}

/// 各盘的用户数据落点不同（等大的池，只在盘 0 上隔离 50182–50183：只有拼出来的、两盘分配记录不对称的镜像走得到）：
/// 盘 0 答 50184、盘 1 答 50182，`Placement` 两盘同槽装不下，拒成那个成员、分配器不动；覆盖写报 `NoSpaceFor`，盘上逐项不变。
#[test]
fn user_data_slots_that_differ_across_devices_are_refused_before_anything_is_written() {
    let mut pool = common::build_pool("supplement-two-user-data-disagree");
    pool.allocator
        .isolate_abandoned(DeviceIdentity(0), SlotNumber(50_182), 2);
    let before = fingerprint(&pool.allocator);
    let snapshot_before = disk_snapshot(&pool.memory_pool(), &pool.stream);
    assert_eq!(
        pool.allocator.try_allocate_user_data(CheckpointTxg(4)),
        Err(
            PlacementRefusal::UserDataSlotsDifferAcrossDevicesPerDeviceSlotsUnsupported {
                slot_per_device: vec![
                    (DeviceIdentity(0), SlotNumber(50_184)),
                    (DeviceIdentity(1), SlotNumber(50_182)),
                ],
            }
        )
    );
    assert_eq!(
        fingerprint(&pool.allocator),
        before,
        "拒绝时分配器一样都没动"
    );

    let publish_parameters = parameters();
    let content = vec![9u8; 4100];
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let refused = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    );
    assert!(
        matches!(
            refused,
            Err(PublishError::NoSpaceFor {
                unit: TransactionUnit::Data
            })
        ),
        "{:?}",
        refused.as_ref().err()
    );
    assert_eq!(fingerprint(&pool.allocator), before, "失败的发布退回分配器");
    assert_eq!(
        disk_snapshot(&pool.memory_pool(), &pool.stream),
        snapshot_before,
        "盘上逐项不变：超级块槽、根环里的根、录制流步数"
    );
}

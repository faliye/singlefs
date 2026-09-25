//! 里程碑「第二个事务」增补 2 收进来的 C369（提交内生块段耗尽时没有回落）：D3（空间分配） 已定项 8 ②
//! 「提交内生块的段耗尽时回落到该设备内槽号最小的空槽，同样排除 `R`」。
//! 池里没有全空的 64 槽聚簇段、开放段也用满，每块盘上却还有大片空槽：一次发布要成功，提交内生块落在回落政策函数给的槽上。
//! 回落与 bump 游标绕开同一套位（已分配、影子账隔离、抬 F 扣住），不是绕开影子账或回收扣住的第二条路。
//! 另钉增补 2 第 ② 行里「抬 F 的空发布分配不到固定点（开放段满、唯一全空段正是扣住的那一段）」补了回落之后的行为：
//! 扣住的段外面还有不被挡的空槽时抬 F 成功、落点一个都不在扣住的槽上；连一个都没有时照旧落点被拒（`PlacementRefused`，每块盘上都没有）。
//! 抬 F 那一串在任何写之前整串预演（代码三方 `research/prompts/m2-final-code-r3-main-verification.md` 第四节第 1 条）：
//! 第一、二、三次哪一次取不到落点都在预演里拒、一次都不发，分配器换回抬 F 之前那一份（C546（抬 F 被拒时扣住的槽不退回））。
//!
//! 「没有全空段」的形态直接改空闲图造（开放段剩下的槽占满、每个全空段占掉段首一槽），不写分配记录：
//! 用真发布占满 3312 个段要几千次发布（原先分配记录树只有一个节点、812 条，根本装不下；按位置寻址之后没有那道墙，
//! D8（核心索引结构） 已定项 14，但仍太慢）。所以这些池上记账与记录对不上，这里不跑池级 checker。

mod common;

use std::collections::BTreeSet;

use common::{build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNodePosition;
use singlefs_core::allocator::{PlacementRefusal, PoolAllocator, UnitFootprint};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, MountError, PublishSequenceFailed, RaisedFloor,
    ShadowLedger,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolWriter,
    PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_core::write_accounting::WriteCallsAndBytes;
use singlefs_format::{CLUSTER_SEGMENT_SLOTS, UNIT_AREA_START_SLOT};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

/// 分配记录树根之下 (层级, 盘, 同盘同层序号) 那个节点的角色（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）。
fn allocation_record_tree_node(level: u8, device: u32, index_in_device: u64) -> TransactionUnit {
    TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
        level,
        device: DeviceIdentity(device),
        index_in_device,
    })
}

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
/// 回落政策函数在这张空闲图上给的落点（手算，不调分配器）：B 先释放 A 的十二个落点（进 defer、槽仍占着），数据单元不受段约束、
/// 取 A 的 50180 之后最低的偶数空槽对 50182；extent 树根是第一个提交内生块，开放段装不下、没有全空段 ⇒ 回落到槽号最小的空槽 50179
/// （mkfs 的 50176–50178 与 A 的数据单元之间那个从没分配过的洞）；inode 树叶容器两槽、起点 32768 对齐，50182–50183 刚给了数据单元 ⇒ 50184；
/// 之后的一槽节点按 bump 次序接着取最低空槽 50186–50194（分配记录树按位置寻址，D8（核心索引结构） 已定项 14：B 改的记录都在两块盘各自的
/// 叶 61 里，两片叶、两个第 1 层节点、根，先叶后根，再是记账树、映射树、树表）。两块盘上的分配记录同槽、分配代 4；冷启动读回 B。
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
        (TransactionUnit::Data(DataUnitIndexInFile::FIRST), 50182),
        (TransactionUnit::ExtentRoot, 50179),
        (
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
            50184,
        ),
        (TransactionUnit::InodeRoot, 50186),
        (allocation_record_tree_node(0, 0, 61), 50187),
        (allocation_record_tree_node(0, 1, 61), 50188),
        (allocation_record_tree_node(1, 0, 0), 50189),
        (allocation_record_tree_node(1, 1, 0), 50190),
        (TransactionUnit::AllocationTree, 50191),
        (TransactionUnit::AccountingTree, 50192),
        (TransactionUnit::MappingTree, 50193),
        (TransactionUnit::TreeTable, 50194),
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

/// 一串发布失败时交出的账（已落盘的那几次加失败账）合起来的写调用数与字节数。
fn sum_of_accounts(failed: &PublishSequenceFailed) -> WriteCallsAndBytes {
    failed
        .writes_of_persisted_publishes
        .iter()
        .chain(&failed.writes_of_failed_publishes)
        .fold(WriteCallsAndBytes::NONE, |sum, writes| {
            sum.plus(writes.total())
        })
}

/// 录制流第 `from` 步之后录下的写（普通写、FUA 写、写零各算一次调用、按长度算字节；屏障不算）：写入口下面那一层数到的。
fn recorded_writes_since(stream: &SharedStream, from: usize) -> WriteCallsAndBytes {
    stream.operations()[from..]
        .iter()
        .filter(|operation| match operation.kind {
            RecordedOperationKind::Write
            | RecordedOperationKind::WriteForceUnitAccess
            | RecordedOperationKind::WriteZeroes => true,
            RecordedOperationKind::Barrier => false,
        })
        .fold(WriteCallsAndBytes::NONE, |sum, operation| {
            sum.plus(WriteCallsAndBytes {
                write_calls: 1,
                written_bytes: operation.length,
            })
        })
}

/// 内存盘外面包录制器，录制器外面包故障注入（注入层在录制器外面：报错的那次写不进录制流）。
type FaultInjectedDevice = FaultInjectingBlockDevice<RecordingBlockDevice<SparseBlockDevice>>;

/// 写故障那一格要的池：两块 4 GiB 内存盘、注入计划先不开，mkfs 同一个进程里第一个文件（txg 3）之后覆盖写五次（txg 4–8）；
/// 分配器是 mkfs 之后那一份（`allocator_after_make_filesystem`，装着根环表）。同一个进程里搭两次逐字节相同。
struct PoolOnFaultInjectedDevices {
    devices: Vec<(DeviceIdentity, FaultInjectedDevice)>,
    stream: SharedStream,
    fault_plan: SharedFaultPlan,
    allocator: PoolAllocator,
    output: TransactionOutput,
}

fn five_overwrites_on_fault_injected_devices() -> PoolOnFaultInjectedDevices {
    let parameters = parameters();
    let stream = SharedStream::new();
    let fault_plan = SharedFaultPlan::unarmed(common::geometry());
    let mut devices: Vec<(DeviceIdentity, FaultInjectedDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(
                        identity,
                        RecordingBlockDevice::with_shared_stream(
                            identity,
                            SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                            stream.clone(),
                        ),
                        fault_plan.clone(),
                    ),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let output = {
        let mut writer = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut writer).expect("取号");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let mut output = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content_of(3000, 1),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件");
        for seed in [31usize, 37, 41, 43, 47] {
            output = publish_overwrite(
                &mut writer,
                &mut allocator,
                &output,
                FirstFile {
                    content: &content_of(2000 + seed, seed),
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
                },
                instance,
            )
            .expect("覆盖写");
        }
        output
    };
    assert_eq!(output.root.checkpoint_txg, CheckpointTxg(8));
    PoolOnFaultInjectedDevices {
        devices,
        stream,
        fault_plan,
        allocator,
        output,
    }
}

/// 在注入计划的那块池上抬 F 到 4（上限 5：第 4 新的非空根）：txg 9 落盘 0、txg 10 落盘 1，这一串两次空发布。
fn raise_the_floor_to_four(
    pool: &mut PoolOnFaultInjectedDevices,
) -> Result<RaisedFloor, MountError> {
    raise_rollback_floor(
        &parameters(),
        &mut pool.devices,
        &mut pool.allocator,
        &mut pool.output,
        CheckpointTxg(4),
        ShadowLedger::On,
    )
}

/// C516（抬 F 那一串发布被拒时前面几次已落盘）在预演之后还走得到的那一格：落点、准入这些落盘之前的错在预演里就报了（一次都不发），
/// 真发时还报得出错的是落盘途中失败。第一次空发布（txg 9）落盘之后，第二次（txg 10）的第一个写报块设备错：
/// 报出这一串已经落盘了 1 次、连同那一次的写账（增补 2 收口表第 58 行：抬 F 的写入口随错丢掉，账要随错交出），
/// 两样相加与录制器数到的写逐项相等；调用方的现行版本已经是落盘的那一次。第一次有几个写由同一块池不注入先抬一遍数出来。
#[test]
fn a_raise_whose_second_empty_publish_fails_on_a_write_reports_that_one_publish_of_the_sequence_persisted(
) {
    let mut counting_pool = five_overwrites_on_fault_injected_devices();
    let raised = raise_the_floor_to_four(&mut counting_pool).expect("不注入时这一串做成");
    assert_eq!(
        raised
            .publishes
            .iter()
            .map(|publish| publish.root.checkpoint_txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(9), CheckpointTxg(10)],
        "这一串两次空发布"
    );
    let write_calls_of_the_first_empty_publish = raised.publishes[0].writes.total().write_calls;

    let mut pool = five_overwrites_on_fault_injected_devices();
    let recorded_before_the_raise = pool.stream.operation_count();
    pool.fault_plan
        .arm(FaultSchedule::the_nth_call_across_the_pool(
            InjectedFault::WriteFails,
            write_calls_of_the_first_empty_publish + 1,
        ));
    let refused = raise_the_floor_to_four(&mut pool);
    let Err(MountError::RaiseFloorSequencePublishFailed(failed)) = refused else {
        panic!(
            "第二次空发布的第一个写报错，报的应是这一串走到哪一步：{:?}",
            refused.as_ref().err()
        );
    };
    assert!(
        matches!(failed.cause, PublishError::BlockDevice(_)),
        "{:?}",
        failed.cause
    );
    assert_eq!(
        failed.writes_of_persisted_publishes.len(),
        1,
        "这一串在报错之前已经落盘了一次（txg 9）"
    );
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(9),
        "调用方的现行版本已经是落盘的那一次"
    );
    assert_eq!(
        failed.writes_of_persisted_publishes,
        vec![pool.output.writes.clone()],
        "已落盘那一次的账就是 txg 9 那次发布自己记的账"
    );
    assert_eq!(
        sum_of_accounts(&failed),
        recorded_writes_since(&pool.stream, recorded_before_the_raise),
        "写入口交出的账相加与录制器数到的写逐项相等"
    );
}

/// 抬 F 之前每块盘上只留 `free_slots_left` 个两盘共同的空槽（槽号最低的那几个），其余空槽全占掉。
fn leave_only_the_lowest_common_free_slots(pool: &mut BuiltPool, free_slots_left: usize) {
    let lowest_free_slots: Vec<u64> = (UNIT_AREA_START_SLOT
        ..UNIT_AREA_START_SLOT + pool.allocator.devices[0].unit_area_slots())
        .filter(|slot| {
            pool.allocator
                .devices
                .iter()
                .all(|device_map| device_map.is_free(SlotNumber(*slot)))
        })
        .take(free_slots_left)
        .collect();
    assert_eq!(
        lowest_free_slots.len(),
        free_slots_left,
        "两块盘上共同的空槽至少 {free_slots_left} 个"
    );
    for device_map in &mut pool.allocator.devices {
        for slot in UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + device_map.unit_area_slots() {
            if device_map.is_free(SlotNumber(slot)) && !lowest_free_slots.contains(&slot) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
}

/// 抬 F 到 8 那一串（txg 14、15、16 三次空发布：14 落盘 0 的区域 2、15 落盘 0、16 落盘 1）在任何写之前的整串预演里
/// 第 `refused_publish_in_the_sequence` 次（从 1 数）取不到落点：报 `RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite`，
/// 拿不到的是 `unit_that_found_no_slot`（每块盘上都没有）；一个写、一道屏障都没发，盘上逐字节不变（两盘系统配置槽、根环里的根、录制流步数），
/// 调用方的现行版本没动，分配器与抬 F 之前逐项相同（按整个的 `Debug` 比：位图的已分配、隔离、扣住三种位与计数、记录、回收过的落点、开放段）。
fn assert_the_raise_is_refused_by_the_rehearsal_before_any_write(
    pool: &mut BuiltPool,
    refused_publish_in_the_sequence: usize,
    unit_that_found_no_slot: TransactionUnit,
) {
    let allocator_before_the_raise = format!("{:?}", pool.allocator);
    let before = common::disk_snapshot(&pool.memory_pool(), &pool.stream);
    let refused = raise_floor(pool, CheckpointTxg(8));
    assert!(
        matches!(
            &refused,
            Err(MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                refused_publish_in_the_sequence: refused_at,
                publishes_in_the_sequence: 3,
                cause: PublishError::PlacementRefused {
                    unit,
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                },
            }) if *refused_at == refused_publish_in_the_sequence && *unit == unit_that_found_no_slot
        ),
        "三次空发布里第 {refused_publish_in_the_sequence} 次的 {unit_that_found_no_slot:?} 在预演里拿不到落点：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        common::disk_snapshot(&pool.memory_pool(), &pool.stream),
        before,
        "预演里就拒：一个写、一道屏障都没发，盘上逐字节不变"
    );
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(13),
        "调用方的现行版本没动：这一串一次都没落盘"
    );
    assert!(
        format!("{:?}", pool.allocator) == allocator_before_the_raise,
        "被拒之后分配器与抬 F 之前逐项相同（扣住的槽放开、回收的回到 defer、补的隔离撤掉）"
    );
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
    // 每次空发布重写的固定点按 bump 次序：分配记录树根之下那几个节点（按位置寻址，D8（核心索引结构） 已定项 14，先叶后根）、根、
    // 记账树、映射树、树表。txg 14 换下的上一版固定点在 50344 之后（叶 62）、新取的落点在叶 61 ⇒ 两块盘各两片叶、两个第 1 层节点，
    // 共十个单元；txg 15、16 换下的与新取的都在叶 61 ⇒ 各八个。
    let fixed_point_slots: Vec<(u64, Vec<u64>)> = raised
        .publishes
        .iter()
        .map(|publish| {
            (
                publish.root.checkpoint_txg.0,
                publish
                    .rewritten
                    .iter()
                    .map(|identity| publish.unit(*identity).slot.0)
                    .collect(),
            )
        })
        .collect();
    assert_eq!(
        fixed_point_slots,
        vec![
            (
                14,
                std::iter::once(50179)
                    .chain(50196..=50204)
                    .collect::<Vec<u64>>()
            ),
            (15, (50205..=50212).collect()),
            (16, (50213..=50220).collect()),
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

/// 同一格，扣住的槽之外一个空槽都没有：抬 F 之前把每块盘上的空槽全占掉，回收出来的全是扣住的槽 ⇒ 这一串（txg 14、15、16 三次空发布）
/// 在任何写之前的整串预演里第一次就落点被拒（`PlacementRefused`、每块盘上都没有）、一个写都没发。C546（抬 F 被拒时扣住的槽不退回）：
/// 这一串一次都没落盘，F 没有一条根带出去，分配器与抬 F 之前逐项相同——回收的回到 defer、扣住位放开、补的隔离撤掉。改之前扣住位留在这个进程里：
/// 记账算它们空闲（每块盘的空闲槽数从 0 涨上去），分配器却一个都发不出去，D28（挂载期承诺量） 的式子把它们算成可用。
/// 第二次、第三次在预演里被拒的两格见 [`a_raise_whose_second_empty_publish_finds_no_slot_is_refused_by_the_rehearsal_before_any_write`]
/// 与 [`a_raise_whose_third_empty_publish_finds_no_slot_is_refused_by_the_rehearsal_before_any_write`]。
#[test]
fn raising_the_floor_with_no_free_slot_outside_the_hold_fails_before_any_write_and_hands_the_allocator_back_exactly_as_before_the_raise(
) {
    let mut pool = build_six_overwrites_after_a_writable_remount("supplement-two-raise-no-slot");
    for device_map in &mut pool.allocator.devices {
        for slot in UNIT_AREA_START_SLOT..UNIT_AREA_START_SLOT + device_map.unit_area_slots() {
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    // 「逐项相同」按分配器整个的 `Debug` 比：每块盘的位图（已分配、隔离、扣住三种位）与计数、记录、回收过的落点、开放段与游标都在里面。
    let allocator_before_the_raise = format!("{:?}", pool.allocator);
    let free_slots_before_the_raise: Vec<u64> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.free_slots())
        .collect();
    assert_eq!(
        free_slots_before_the_raise,
        vec![0, 0],
        "抬 F 之前每块盘上一个空槽都不剩"
    );
    let operations_before = pool.stream.operations().len();
    let refused = raise_floor(&mut pool, CheckpointTxg(8));
    // 第一次空发布按 bump 次序的第一个单元：分配记录树按位置寻址（D8（核心索引结构） 已定项 14），它换下的上一版固定点在 50344 之后，
    // 盘 0 那片叶 62 排在最前（先叶后根）。
    let first_unit_of_the_first_empty_publish = allocation_record_tree_node(0, 0, 62);
    assert!(
        matches!(
            &refused,
            Err(MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite {
                refused_publish_in_the_sequence: 1,
                publishes_in_the_sequence: 3,
                cause: PublishError::PlacementRefused {
                    unit,
                    refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
                },
            }) if *unit == first_unit_of_the_first_empty_publish
        ),
        "回收出来的全是扣住的槽，第一次空发布的第一个固定点就拿不到（每块盘上都没有：容量不够那一种），预演里就拒、这一串一次都没落盘：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        pool.stream.operations().len(),
        operations_before,
        "报错在任何写之前"
    );
    let free_slots_after_the_refusal: Vec<u64> = pool
        .allocator
        .devices
        .iter()
        .map(|device_map| device_map.free_slots())
        .collect();
    assert_eq!(
        free_slots_after_the_refusal, free_slots_before_the_raise,
        "回收的槽回到 defer：记账不再把一个发不出去的槽算成空闲"
    );
    assert!(
        format!("{:?}", pool.allocator) == allocator_before_the_raise,
        "被拒之后分配器与抬 F 之前逐项相同（位图、计数、记录、回收过的落点、开放段）"
    );
    assert_eq!(
        pool.output.root.checkpoint_txg,
        CheckpointTxg(13),
        "调用方的现行版本没动：这一串一次都没落盘"
    );
    assert_eq!(
        pool.allocator
            .try_allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(14)),
        Err(PlacementRefusal::NoFreeSlotOnAnyDevice),
        "每块盘上本来就一个空槽都没有"
    );
}

/// C546（抬 F 被拒时扣住的槽不退回） 第二次起那一半（代码三方 `research/prompts/m2-final-code-r3-main-verification.md` 第三节「越格线索」、
/// 第四节第 1 条）。抬 F 之前每块盘上只留十个两盘共同的空槽（一次空发布重写四个固定点，D16（发布语义） 已定项 9；分配记录树按位置寻址，
/// D8（核心索引结构） 已定项 14，第一次空发布换下的上一版固定点在叶 62、新取的落点在叶 61，分配记录树那一个固定点是两块盘各两片叶、
/// 两个第 1 层节点与根七个节点，加记账树、映射树、树表共十个单元），其余空槽全占掉；抬 F 回收出来的槽扣到生效为止。
/// 预演里第一次空发布（txg 14）拿走那十个槽，第二次（txg 15）一个都拿不到，报的是它按 bump 次序的第一个单元（盘 0 那片叶 61）。
/// 改之前这一串逐次发：txg 14、带新 F = 8 的那条根已经落盘（F 没在两块盘上生效）、第二次才被拒，分配器停在第一次之后、
/// 扣住的槽留在这个进程里（C516（抬 F 那一串发布被拒时前面几次已落盘） 那一格：`RaiseFloorSequencePublishFailed`、已落盘 1 份）。
#[test]
fn a_raise_whose_second_empty_publish_finds_no_slot_is_refused_by_the_rehearsal_before_any_write() {
    let mut pool =
        build_six_overwrites_after_a_writable_remount("supplement-two-raise-refused-at-the-second");
    leave_only_the_lowest_common_free_slots(&mut pool, 10);
    assert_the_raise_is_refused_by_the_rehearsal_before_any_write(
        &mut pool,
        2,
        allocation_record_tree_node(0, 0, 61),
    );
}

/// 同一格推到第三次：只留十八个两盘共同的空槽——预演里第一次（txg 14）拿走十个、第二次（txg 15）八个
/// （[`raising_the_floor_when_the_only_empty_segment_is_held_falls_back_to_slots_outside_the_hold`] 钉着三次各拿几个），
/// 第三次（txg 16，落盘 1）一个都拿不到，报它按 bump 次序的第一个单元（它换下的第二次那几个落点也在叶 61：盘 0 那片叶 61）。
/// 改之前前两次已经落盘、第三次才被拒（已落盘 2 份）。
#[test]
fn a_raise_whose_third_empty_publish_finds_no_slot_is_refused_by_the_rehearsal_before_any_write() {
    let mut pool =
        build_six_overwrites_after_a_writable_remount("supplement-two-raise-refused-at-the-third");
    leave_only_the_lowest_common_free_slots(&mut pool, 18);
    assert_the_raise_is_refused_by_the_rehearsal_before_any_write(
        &mut pool,
        3,
        allocation_record_tree_node(0, 0, 61),
    );
}

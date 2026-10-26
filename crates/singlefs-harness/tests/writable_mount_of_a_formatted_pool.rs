//! 里程碑「覆盖写、释放、回退与复用」步 3 的另一格（只做过 mkfs 的池允许可写挂载）：mkfs 之后进程退出、重开两个镜像走可写挂载——
//! 恢复择到 mkfs 的第 0 代根、环里一条记录都没有；取号 1；上一个实例是 0，不写行（D18（块里携带什么信息） 已定项 11）；
//! 树表 0 条 ⇒ 写行那次发布与暖机都写零个单元（D16（发布语义） 已定项 9），txg 1 落盘 1、txg 2 落盘 0；之后在同一个进程里发布第一个文件版本（txg 3）。
//! mkfs 之后的整条录制流与 mkfs 同一个进程里跑新池新建文件逐字节相同（第一次可写挂载要写的区间是空的，新池新建文件的字节不变）；
//! 冷启动读回那个文件；每一步的镜像池级 checker 一条违例都没有（没有文件的两步里没有记账树与 inode 树的 8 条报不适用）。
//!
//! 这一份还罩着挂载不止一次的那几格：
//! 树表 0 条的一版上**写实例表行**（只重写实例表一个单元）、第一个文件版本从现行那一版接着算 txg 与 jsn、
//! 写过行的那一版上再挂载一次被拒（被换下的那片实例表记在哪没有条款）。回退到树表 0 条的根那两格随回退改成挂着时的向前发布删掉
//! （向前回退的候选要带文件，D23（journal 的角色与格式） 已定项 14；树表 0 条的根拒成 `VersionWithoutFile`，由步 4 的候选集用例钉）。
//! 端到端那一条是 `a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold`。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/writable_mount_of_a_formatted_pool.rs"]
mod scenarios_of_writable_mount_of_a_formatted_pool;

use common::{
    crash_state_devices, disk_snapshot, file_content, format_pool, geometry,
    memory_pool_of_sparse_devices, parameters, unreadable_root_slot_of, with_unreadable_ranges,
    without_unreadable_ranges, FailingReadsOfARange, FormattedPool, SharedUnreadableRanges,
    UnreadableRange, UnreadableRangeReadBack, FIXED_WRITE_TIME_SECONDS,
};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_writable_mount_of_a_formatted_pool::{
    devices_crashed_right_after_the_root_slot_of, region_device,
};
use singlefs_checker::image::parse_node_pointer;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::{
    back_chain_of, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
};
use singlefs_core::make_filesystem::{INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::{
    mount_writable, InstanceRow, MountError, NewerPublishWitness, ReadStageSettled, RollbackTarget,
    SelectedVersionAgainstTheWitness, WitnessedCounterComparison,
};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryFailure, RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_without_units, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError,
    TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees, ZeroUnitPublishPlan,
};
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::memory_pool::writes_and_segments;
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

/// 每块盘第 `ordinal` 次读系统配置槽 0（偏移 0）时注入一次读错，别的读写原样交给内层
/// （m2-emptypool-nonempty-r1 云端攻方腿模型 `opus_attack_emptypool.rs` 里 `TransientSystemConfigurationReadErrorDevice` 的做法）。
/// 用的是通用的故障注入包装（增补 3 第 4 件，`singlefs_harness::fault_injection`；这里原先手写的
/// `TransientSystemConfigurationReadErrorDevice` 2026-09-21 并进了它）：逐盘数，所以两块盘各在自己第 `ordinal` 次上报错一次。
fn fail_the_nth_read_of_system_configuration_slot_zero_on_each_device(
    ordinal: u64,
) -> FaultSchedule {
    FaultSchedule {
        fault: InjectedFault::ReadFails,
        device: FaultDeviceSelector::EveryDevice,
        placement: FaultPlacement::OffsetExactly(DeviceOffsetInBytes(0)),
        counting: FaultCounting::PerDevice,
        occurrence: FaultOccurrence::TheNthMatchingCall(ordinal),
    }
}

/// 一次可写挂载里每块盘上第几次读系统配置槽 0 是取号之前的判定那一读（`transaction::instance_generation_to_acquire`）。
/// 判定之前每块盘各读槽 0 五次：`mount_writable` 择系统配置一次（`recovery::choose_system_configuration`）、核盘表里这块盘的本盘设备号
/// 一次（`mount::device_table_disagreeing_with` 读两槽，实审 A1b Q4）、重放里择系统配置一次（`recovery::replay_journal`）、
/// 读阶段判「系统配置见证过比所选那一版新的发布」一次（C554 乙，`mount::newer_publish_witness` 读两槽；判假不重读）、
/// 读阶段之后判退那一步先问一次 F2 判法的见证（`recovery::fall_back_judged_on_the_three_readers`，读两槽：判退看三样读者的并集之后
/// 一条记录都没施加的挂载每次都读，C755）。判定之后取号写之前重算一次。回退见证删掉之前是 7（择根、重放、影子账各读一次见证，判定之后写见证表之前
/// 再读一次）；核本盘设备号之前是 3，C554 乙之前是 4，判退看并集之前是 5。数法：注入点从 1 挨个试到 8，只有判定那一读让判定与重算分叉成「判定号 1、重算号 2」（实三草稿里量的）；
/// 读法变了（多读或少读一次），这个数跟着改，注入点就还是判定那一读。
const DECISION_READ_OF_SYSTEM_CONFIGURATION_SLOT_ZERO_IN_A_WRITABLE_MOUNT: u64 = 6;

/// 取号之前判定算出的号与取号写之前重算的号不同，取号不写、报错（m2-emptypool-nonempty-r1 云端攻方腿 Z2）：mkfs 之后第一次可写挂载崩在
/// 取号第一轮两写之后、第二轮落盘之前（同一个进程里取号就停、镜像关掉；两盘系统配置槽 0 已是号 1，槽 1 还是 mkfs 写的号 0——取号写两轮
/// （甲'）之后这一形只在两轮之间崩才有，用例取完号把两盘槽 1 写回取号之前的字节来造），重开时两块盘在判定那一读上（第 6 次读系统配置槽 0，
/// `DECISION_READ_OF_SYSTEM_CONFIGURATION_SLOT_ZERO_IN_A_WRITABLE_MOUNT`）各报一次瞬时读错——
/// 判定那一遍只看到 mkfs 的槽 1（号 0）、算出号 1、要写的行区间为空放行；取号重算读到号 1、算出号 2 ⇒ 返回
/// `InstanceGenerationChangedBeforeAcquisition { expected: 1, recomputed: 2 }`，`DiskSnapshot` 不变。
#[test]
fn transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write(
) {
    let mut formatted = format_pool("step-three-formatted-transient-read-error");
    {
        let publish_parameters = parameters();
        let slot_one_offset = DeviceOffsetInBytes(u64::from(
            publish_parameters.geometry.fixed_structure_slot_spacing,
        ));
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let slot_one_before_the_acquisition: Vec<Vec<u8>> = open_devices
            .iter()
            .map(|(_, device)| {
                let mut bytes = vec![0u8; 4096];
                device
                    .read_at(slot_one_offset, &mut bytes)
                    .expect("读得到槽 1");
                bytes
            })
            .collect();
        {
            let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
            assert_eq!(
                acquire_instance(&mut writer).expect("取号"),
                InstanceGeneration(1)
            );
        }
        // 崩在取号两轮之间：第二轮那一写（落槽 1）没落盘，槽 1 还是取号之前那一份。
        for ((_, device), bytes) in open_devices
            .iter_mut()
            .zip(&slot_one_before_the_acquisition)
        {
            device
                .write_at(slot_one_offset, bytes, WriteDurability::Plain)
                .expect("槽 1 写回取号之前的字节");
        }
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    let plan = SharedFaultPlan::armed(
        geometry(),
        fail_the_nth_read_of_system_configuration_slot_zero_on_each_device(
            DECISION_READ_OF_SYSTEM_CONFIGURATION_SLOT_ZERO_IN_A_WRITABLE_MOUNT,
        ),
    );
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<_>)> = formatted
        .reopen_recorded()
        .into_iter()
        .map(|(identity, recorded)| {
            (
                identity,
                FaultInjectingBlockDevice::new(identity, recorded, plan.clone()),
            )
        })
        .collect();
    let refused = mount_writable(&parameters(), &mut devices);
    formatted.devices = Some(
        devices
            .into_iter()
            .map(|(identity, device)| (identity, device.into_inner()))
            .collect(),
    );
    assert!(
        matches!(
            refused,
            Err(MountError::InstanceGenerationChangedBeforeAcquisition {
                expected: InstanceGeneration(1),
                recomputed: InstanceGeneration(2),
            })
        ),
        "判定看到号 1、取号重算得号 2：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        disk_snapshot(&formatted.memory_pool(), &formatted.stream),
        before,
        "取号一个字节都没写"
    );
}

/// 树表 0 条的一版上写实例表行：mkfs 之后第一次可写挂载崩在取号两写之后（盘上只多了号 1，没有记录、没有新根），
/// 重开可写挂载——所选根是 mkfs 的第 0 代根、要取的号 2、要写的行区间 [1, 2) 不为空 ⇒ 写行那次发布**只重写实例表一个单元**
/// （树表 0 条 ⇒ 没有记账树，D16（发布语义） 已定项 9 的那五样一样都不写；D18（块里携带什么信息） 已定项 11 要求每次可写挂载都写行），
/// 被烧掉的号 1 记成中间实例行 (1, 0, 0)；txg 1 jsn 1 落盘 1、暖机 txg 2 落盘 0。
#[test]
fn writable_mount_after_a_crash_right_after_acquiring_an_instance_writes_a_row_for_the_burnt_instance(
) {
    let mut formatted = format_pool("step-three-formatted-crash-after-acquisition");
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        acquire_instance(&mut writer).expect("取号");
    }
    let genesis_instance_table_slot = formatted.genesis.root.instance_table.locations[0].slot;
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("写行那次发布做成");
    formatted.devices = Some(devices);
    assert_eq!(mounted.output.instance, InstanceGeneration(2));
    assert_eq!(
        mounted.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(0),
            applied_transaction_high_water: 0,
        }],
        "号 1 被烧掉、什么都没发布过：中间实例行 (1, 0, 0)"
    );
    let PoolVersion::WithoutFile(row) = &mounted.output.row_publish else {
        panic!("树表 0 条：写行那次发布仍是「没有文件版本」的一版")
    };
    assert_eq!(
        (
            row.root.instance,
            row.root.checkpoint_txg,
            row.record.counter,
            row.record.transaction,
            row.record.back_chain,
            row.record.named.len()
        ),
        (InstanceGeneration(2), CheckpointTxg(1), 1, 0, 0, 6),
        "txg 1、jsn 1、事务号 0、本实例第一条反向链 0；点名实例表与分配记录树五个节点（按位置寻址，D8（核心索引结构） 已定项 14）"
    );
    assert_eq!(
        row.record.ordinal_within_publish,
        JournalRecordOrdinalWithinPublish::FIRST,
        "写行那次发布只有这一条记录：本次发布内序号 1（D23（journal 的角色与格式） 已定项 4）"
    );
    assert_eq!(
        row.record.place_in_publish,
        JournalRecordPlaceInPublish::LastRecordOfThePublish,
        "写行那次发布只有这一条记录：它就是本次发布末条，记录标志位 0 写 1（D23（journal 的角色与格式） 已定项 17）"
    );
    assert_eq!(
        row.root.tree_table, formatted.genesis.root.tree_table,
        "树表照抄 mkfs 那片 0 条的"
    );
    assert_ne!(
        row.root.instance_table.locations[0].slot, genesis_instance_table_slot,
        "实例表 COW 到新落点"
    );
    assert_eq!(
        mounted.output.warm_up_publishes.len(),
        1,
        "txg 1 落盘 1、txg 2 落盘 0"
    );
    let table = singlefs_core::recovery::instance_table_of_root(
        &formatted.memory_pool(),
        mounted.output.warm_up_publishes[0].root(),
    )
    .expect("盘上那片实例表解得开");
    assert_eq!(table.rows, mounted.output.rows_written, "盘上写着这一行");
}

/// 环里留着一条孤记录时，只做过 mkfs 的池照常可写挂载（2026-09-23 用户定案收窄 R4：`NEW_POOL_FILE_CREATION_TXG` 只管 mkfs
/// 同一个进程里那条流，不管任何池的第一个文件版本，所以「新实例的第一次发布不是 txg 1、jsn 1」不再是拒绝的理由）：
/// mkfs 之后第一次可写挂载取号写两轮那四次写（两块盘各两槽，写进号 1）被说谎的设备吞掉（`InjectedFault::WriteIsSwallowed`：报成功、
/// 一个字节都没落盘；这次挂载之后的系统配置槽写一并吞掉，它们都在 txg 1 的根槽写之后、不进崩溃状态），之后崩在 txg 1 的记录两写持久、
/// txg 1 的根槽没持久（真崩溃留不下这一形：取号写与记录写之间有屏障，记录持久而取号写没持久
/// 只有说谎的设备做得出；代码轮 LR6 把用例自己改写系统配置槽的造法换成这一种）——两块盘两槽仍是 mkfs 写的那一份，择系统配置只剩 mkfs 的那一份、
/// 根环只有第 0 代根，要取的号 1、要写的行为空；环里那条 txg 1 的记录让新实例从
/// txg 2、jsn 2 起 ⇒ 零单元发布 txg 2（落盘 0），暖机推到 txg 4（落盘 1）才覆盖两块盘，之后第一个文件版本接着写 txg 5。
#[test]
fn formatted_pool_mount_starting_after_a_leftover_record_publishes_from_the_next_txg() {
    let mut formatted = format_pool("step-three-formatted-leftover-record");
    // 取号写两轮那四次写（每块盘头两次写系统配置槽，两槽各一次）被设备吞掉：注入层包在录制器外面，吞掉的写不进录制流，录制流里的就是落到设备上的。
    // 只吞取号第一轮那一写不够（改前的造法）：取号写两轮之后第二轮把号 1 写进槽 1，号 1 留在盘上、下一次取 2，测不到「号 1 不在盘上、重取 1 无害」。
    // 不吞成撕裂：C331（择根倒挂压过已确认的写） 取甲之后见证读缺一槽就拒可写（用户 2026-09-28 定），撕裂造的走不到孤记录这一支；
    // 吞掉之后两槽仍是 mkfs 写的那一份（mkfs 两槽写同一份字节），两槽都自证得过。
    let system_configuration_slots_end = 2 * u64::from(geometry().fixed_structure_slot_spacing);
    let swallowing_the_acquisition = SharedFaultPlan::armed(
        geometry(),
        FaultSchedule {
            fault: InjectedFault::WriteIsSwallowed,
            device: FaultDeviceSelector::EveryDevice,
            placement: FaultPlacement::OffsetBelow(system_configuration_slots_end),
            counting: FaultCounting::PerDevice,
            occurrence: FaultOccurrence::EVERY_MATCHING_CALL,
        },
    );
    let mut first_mount_devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<_>)> = formatted
        .reopen_recorded()
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                FaultInjectingBlockDevice::new(
                    identity,
                    device,
                    swallowing_the_acquisition.clone(),
                ),
            )
        })
        .collect();
    // 说谎的设备之后这次挂载自己怎么收尾不判：只要它在 txg 1 的根槽之前那几次写（录制流里）。
    let _first_mount = mount_writable(&parameters(), &mut first_mount_devices);
    assert!(
        swallowing_the_acquisition.fired().len() >= 4,
        "两块盘各吞了取号两轮那两次系统配置槽写（之后的轮换写一并吞掉）：{}",
        swallowing_the_acquisition.fired().len()
    );
    drop(first_mount_devices);
    let operations = formatted.retained_operations();
    let (writes, _segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    let first_root_slot_write = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("第一次可写挂载写了 txg 1 的根槽");
    let persisted: Vec<bool> = (0..writes.len())
        .map(|write_index| write_index < first_root_slot_write)
        .collect();
    assert!(
        writes[..first_root_slot_write]
            .iter()
            .any(|write| write.kind == StepKind::JournalRecord),
        "txg 1 的根槽之前落了它的记录"
    );
    let crash_stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(
        &formatted.memory_pool_after_mkfs(),
        &writes,
        &persisted,
        &crash_stream,
    );
    let image_before = memory_pool_of_sparse_devices(&devices);
    let before = disk_snapshot(&image_before, &crash_stream);
    assert_eq!(
        before
            .readable_roots
            .iter()
            .map(|root| (root.instance, root.checkpoint_txg))
            .collect::<Vec<_>>(),
        vec![(InstanceGeneration(0), CheckpointTxg(0)); 3],
        "根环只有三份第 0 代根"
    );
    let mounted = mount_writable(&parameters(), &mut devices).expect("孤记录之后照常可写挂载");
    assert_eq!(mounted.output.instance, InstanceGeneration(1), "要取的号 1");
    assert!(
        mounted.output.rows_written.is_empty(),
        "上一个实例是 0：不写行"
    );
    assert_eq!(
        (
            mounted.output.row_publish.root().checkpoint_txg,
            mounted.output.row_publish.record().counter
        ),
        (CheckpointTxg(2), 2),
        "环里那条 txg 1 的记录让新实例从 txg 2、jsn 2 起"
    );
    assert_eq!(
        mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| publish.root().checkpoint_txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(3), CheckpointTxg(4)],
        "txg 2 与 3 都落盘 0，推到 txg 4（落盘 1）才覆盖两块盘"
    );
    assert_eq!(region_device(2), DeviceIdentity(0));
    assert_eq!(region_device(3), DeviceIdentity(0));
    assert_eq!(region_device(4), DeviceIdentity(1));
    // 第一个文件版本接着写 txg 5：`publish_first_file` 从现行那一版接着算，不写死 3。
    let mut allocator = mounted.allocator;
    let current = mounted.current;
    let content = file_content();
    let first = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(1),
            current.record_bytes(),
        )
        .expect("第一个文件版本")
    };
    assert_eq!(
        (first.root.checkpoint_txg, first.record.counter),
        (CheckpointTxg(5), 5)
    );
    assert_eq!(
        recover(
            &memory_pool_of_sparse_devices(&devices),
            JournalPolicy::Consult
        )
        .outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(5)),
            content
        },
        "冷启动读回那个文件"
    );
}

/// `publish_first_file` 的两个入参说的不是同一版就不写、报错（m2-emptypool-nonempty-r1 云端攻方腿 Z3-A 那一层）：
/// 新根的 txg 从「要建在上面的那一版」那条根接着算、jsn 从上一条记录接着算，两者对不上时接上去会盖在别的发布上。
/// mkfs 之后取号、暖机两次（txg 1、2），再发一次零单元发布（txg 3、jsn 3），接着拿 **txg 2 的那条根** 配 **txg 3 的那条记录**
/// 调 `publish_first_file` ⇒ 返回 `FirstFileVersionDoesNotFollowTheVersionItBuildsOn`，录制流一步不多、分配器不动。
#[test]
fn first_file_version_whose_root_and_previous_record_disagree_is_refused_before_any_write() {
    let mut formatted = format_pool("step-three-formatted-first-file-after-txg-three");
    let publish_parameters = parameters();
    let genesis_root = formatted.genesis.root;
    let open_devices = formatted.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis_root, instance).expect("暖机");
    let third = publish_without_units(
        &mut writer,
        warmed.roots.last().expect("暖机两代根"),
        ZeroUnitPublishPlan {
            txg: CheckpointTxg(3),
            counter: 3,
            instance,
            back_chain: back_chain_of(&warmed.last_record_bytes),
            rollback_floor: CheckpointTxg(0),
            tree_identifier_watermark: warmed
                .roots
                .last()
                .expect("暖机两代根")
                .tree_identifier_watermark,
        },
    )
    .expect("第三次零单元发布");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
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
    let records_before = allocator.records().to_vec();
    let recorded_before = formatted.stream.operations().len();
    let content = file_content();
    let refused = publish_first_file(
        &mut writer,
        &mut allocator,
        warmed.roots.last().expect("暖机两代根"),
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &third.record_bytes,
    );
    assert!(
        matches!(
            refused,
            Err(
                PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn {
                    version_to_build_on: CheckpointTxg(2),
                    previous_record: Some((CheckpointTxg(3), 3))
                }
            )
        ),
        "要建在 txg 2 那一版上、而上一条记录是 txg 3：{:?}",
        refused.as_ref().err()
    );
    assert_eq!(
        formatted.stream.operations().len(),
        recorded_before,
        "一个写、一道屏障都没发"
    );
    assert_eq!(allocator.records(), records_before.as_slice(), "分配器没动");
}

/// C511（回退到无文件那一版之后诞生代怎么接） 第 3 步给 `publish_first_file` 新加的两道，都在任何写之前返回：
/// ① 判「那一版有没有过文件版本」改看它的树表条数，而那一版的树表单元两份都读不出（两盘上 mkfs 那片树表整个写零）
///    ⇒ `TreeTableOfTheVersionToBuildOnUnreadable`，带着恢复路径那一格的原因；
/// ② 十二棵树从那一版的水位（盘上读来的 8 字节）起连号发，而水位离 `u64::MAX` 不到十二个号
///    ⇒ `TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees`，带着那个水位。
/// 两格都是一个写、一道屏障都没发（录制流一步不多）、分配器不动。
#[test]
fn first_file_version_that_cannot_judge_or_issue_its_trees_is_refused_before_any_write() {
    let mut formatted = format_pool("step-three-formatted-first-file-cannot-issue-trees");
    let publish_parameters = parameters();
    let genesis_root = formatted.genesis.root;
    let open_devices = formatted.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
    let instance = acquire_instance(&mut writer).expect("取号");
    let warmed = warm_up(&mut writer, &genesis_root, instance).expect("暖机");
    let warm_up_root = *warmed.roots.last().expect("暖机两代根");
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
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
    let records_before = allocator.records().to_vec();
    let content = file_content();

    // ②：水位离 u64::MAX 只剩三个号。
    let mut root_with_no_room_for_the_file_version_trees = warm_up_root;
    root_with_no_room_for_the_file_version_trees.tree_identifier_watermark = u64::MAX - 3;
    let recorded_before_the_watermark_case = formatted.stream.operations().len();
    let refused_for_the_watermark = publish_first_file(
        &mut writer,
        &mut allocator,
        &root_with_no_room_for_the_file_version_trees,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    );
    assert!(
        matches!(
            refused_for_the_watermark,
            Err(PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(
                TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees {
                    tree_identifier_watermark
                }
            )) if tree_identifier_watermark == u64::MAX - 3
        ),
        "水位离 u64::MAX 不到十二个号：{:?}",
        refused_for_the_watermark.as_ref().err()
    );
    assert_eq!(
        formatted.stream.operations().len(),
        recorded_before_the_watermark_case,
        "水位那一格：一个写、一道屏障都没发"
    );
    assert_eq!(allocator.records(), records_before.as_slice(), "分配器没动");

    // ①：两盘上 mkfs 那片树表整个写零（整单元校验和对不上），那一版的树表两份都读不出。
    for (_, device) in writer.devices.iter_mut() {
        device
            .write_at(
                DeviceOffsetInBytes(TREE_TABLE_GENESIS_SLOT.0 * 16384),
                &[0u8; 16384],
                WriteDurability::ForceUnitAccess,
            )
            .expect("抹树表写得进去");
    }
    let recorded_before_the_tree_table_case = formatted.stream.operations().len();
    let refused_for_the_tree_table = publish_first_file(
        &mut writer,
        &mut allocator,
        &warm_up_root,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS,
        },
        instance,
        &warmed.last_record_bytes,
    );
    assert!(
        matches!(
            refused_for_the_tree_table,
            Err(PublishError::TreeTableOfTheVersionToBuildOnUnreadable {
                failure: RecoveryFailure::UnitUnreadable { slot }
            }) if slot == TREE_TABLE_GENESIS_SLOT
        ),
        "那一版的树表两份都读不出：{:?}",
        refused_for_the_tree_table.as_ref().err()
    );
    assert_eq!(
        formatted.stream.operations().len(),
        recorded_before_the_tree_table_case,
        "树表那一格：一个写、一道屏障都没发"
    );
    assert_eq!(allocator.records(), records_before.as_slice(), "分配器没动");
}

/// 崩在暖机与第一个文件版本之间，重开照样写得出行：新池新建文件取号、暖机两次之后、第一个文件版本之前崩溃
/// （同一个进程里做到暖机就停、镜像关掉），重开可写挂载——所选根 (1, 2)、树表 0 条，要取的号 2，要写的行区间 [1, 2) 不为空 ⇒
/// 写行那次发布只重写实例表一个单元（txg 3、jsn 3、落盘 0），暖机一次（txg 4、落盘 1）；写出的行是 (1, 2, 0)
/// ——上一个实例 1 那一行，选的根 txg 2、W = 0（环里两条记录都是事务号 0 的空发布）。
/// 再往下第三次可写挂载会撞 `VersionWithoutFileNotWrittenByMakeFilesystem`（被换下的那一片实例表记在哪没有条款），
/// 由 `a_third_writable_mount_on_a_version_whose_rows_were_written_is_refused_before_touching_the_allocator` 钉住。
#[test]
fn writable_mount_after_a_crash_between_warm_up_and_the_first_file_writes_the_row_for_the_previous_instance(
) {
    let mut formatted = format_pool("step-three-formatted-crash-after-warm-up");
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号");
        assert_eq!(instance, InstanceGeneration(1));
        warm_up(&mut writer, &formatted.genesis.root, instance).expect("暖机");
    }
    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    assert_eq!(
        before
            .readable_roots
            .iter()
            .map(|root| (root.instance, root.checkpoint_txg))
            .max(),
        Some((InstanceGeneration(1), CheckpointTxg(2))),
        "崩溃点：暖机两次的根都在盘上"
    );
    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("写行那次发布做成");
    formatted.devices = Some(devices);
    assert_eq!(mounted.output.instance, InstanceGeneration(2));
    assert_eq!(
        mounted.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(2),
            applied_transaction_high_water: 0,
        }],
        "上一个实例 1 那一行：选的根 txg 2、W = 0"
    );
    let PoolVersion::WithoutFile(row) = &mounted.output.row_publish else {
        panic!("树表 0 条：写行那次发布仍是「没有文件版本」的一版")
    };
    assert_eq!(
        (
            row.root.checkpoint_txg,
            row.record.counter,
            row.record.transaction,
            row.record.back_chain,
            row.record.named.len()
        ),
        (CheckpointTxg(3), 3, 0, 0, 6),
        "txg 3、jsn 3、事务号 0、本实例第一条反向链 0；点名实例表与分配记录树五个节点（按位置寻址，D8（核心索引结构） 已定项 14）"
    );
    assert_eq!(region_device(3), DeviceIdentity(0), "txg 3 落盘 0");
    assert_eq!(
        mounted
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| publish.root().checkpoint_txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(4)],
        "txg 4 落盘 1，一次就覆盖两块盘"
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_writable_mount_of_a_formatted_pool::the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_writable_mount_of_a_formatted_pool::the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool,
    );
}

/// 树表 0 条、实例表已经不是 mkfs 那一片、而根记录里的分配记录树根指针是全零（旧格式的一版，或坏镜像）：
/// 第三次可写挂载在动分配器之前拒绝，返回 `VersionWithoutFileNotWrittenByMakeFilesystem`。
///
/// **R8 的依据 2026-09-23 换了**：此前这一格拦的是一条正当历史（写行之后重开，被换下的那片实例表没有地方记）；
/// C512（树表 0 条的一版上被换下的单元记在哪） 定案之后那条历史自带分配记录树、照常挂载
/// （`the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool`）。
/// 今天这一道拦的只剩「树表或实例表不是 mkfs 写的那一版、而这一版又没有自己的分配记录树」这种镜像——
/// 实现一处都不写得出它，所以要把根记录那一项抹成全零才走得到。抹掉之后盘上逐字节的其余部分不变：
/// 两盘系统配置槽不变、根环没有新根、录制流一步都没多。
#[test]
fn a_third_writable_mount_on_a_version_whose_rows_were_written_is_refused_before_touching_the_allocator(
) {
    let mut formatted = format_pool("step-three-formatted-third-mount");
    let mut first = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first).expect("第一次可写挂载：不写行");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let row_written = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    let root_after_the_row = *row_written.current.root();
    formatted.devices = Some(second);
    assert_ne!(
        root_after_the_row.instance_table.head.birth_txg,
        CheckpointTxg(0),
        "写过行之后实例表的诞生 txg 不再是 0"
    );
    plant_a_root_without_its_allocation_record_tree(&mut formatted, &root_after_the_row);

    let before = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    let mut third = formatted.reopen_recorded();
    let refused = mount_writable(&parameters(), &mut third);
    formatted.devices = Some(third);
    assert!(
        matches!(
            refused,
            Err(MountError::VersionWithoutFileNotWrittenByMakeFilesystem {
                root: RollbackTarget {
                    instance: InstanceGeneration(2),
                    checkpoint_txg: CheckpointTxg(4)
                },
                tree_table_birth_txg: CheckpointTxg(0),
                ..
            })
        ),
        "树表 0 条、实例表已经不是 mkfs 那一片、又没有自己的分配记录树：{:?}",
        refused.as_ref().err()
    );
    let after = disk_snapshot(&formatted.memory_pool(), &formatted.stream);
    assert_eq!(
        after.system_configuration_slots, before.system_configuration_slots,
        "两盘系统配置槽逐字节不变：没有取号"
    );
    assert_eq!(after.readable_roots, before.readable_roots, "根环没有新根");
    assert_eq!(
        after.recorded_operations, before.recorded_operations,
        "一个写、一道屏障都没发"
    );
}

/// 根记录里分配记录树根指针那 86 字节在根槽里的位置（D22（单元原子性怎么合成） 已定项 7 的偏移表：
/// 中央映射树根指针 256、分配记录树根指针 342、回收门槛 t 428）。
const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_POINTER: std::ops::Range<usize> = 342..428;

/// 一条根在根环里那个槽的原样字节，从盘上那一份读（不经核心层的解析）。
fn root_slot_bytes_on_disk(formatted: &FormattedPool, checkpoint_txg: CheckpointTxg) -> Vec<u8> {
    let target = target_for_publish(checkpoint_txg, geometry().root_ring_slots_per_region);
    let offset = slot_offset(target, geometry().fixed_structure_slot_spacing);
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("根槽宽");
    PoolReader::read(&formatted.memory_pool(), device, offset, slot_bytes).expect("根槽读得到")
}

/// 根记录里「分配记录树根指针」那一项在带文件的一版上恒全零（D16（发布语义） 已定项 9、D22（单元原子性怎么合成） 已定项 7：
/// 只有树表 0 条、写过行的那一版写非零，mkfs 的第 0 代与带文件的一版写全 0；带文件的一版的分配记录树住树表条目，
/// 两处都写就成了同一个量的两份手抄。代码三方 `research/prompts/m2-wave3-code-r1-main-verification.md` 第三节 Y5 查出
/// `transaction.rs` 注释点名的这条用例不存在，第四节第 3 条补上）。
/// 建池 → 可写挂载（不写行）→ 退出 → 再可写挂载（写行那次发布写出非零的一项，暖机照抄）→ 第一个文件版本 → 覆盖写一版：
/// 写行那一版盘上根槽 342..428 按 checker 的字段表解得出那片分配记录树的指针（偏移钉在盘上，不只钉读写往返）；
/// 接在它后面的两版带文件的根，交回的根记录里这一项是 `NodePointer::empty_root()`，盘上那 86 字节全零。
#[test]
fn root_record_of_a_file_version_leaves_the_allocation_record_tree_pointer_zero() {
    let mut formatted =
        format_pool("step-three-formatted-file-version-allocation-record-tree-root");
    let mut first_mount_devices = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first_mount_devices).expect("第一次可写挂载：不写行");
    formatted.devices = Some(first_mount_devices);
    let mut second_mount_devices = formatted.reopen_recorded();
    let second_mount =
        mount_writable(&parameters(), &mut second_mount_devices).expect("第二次可写挂载：写行");
    formatted.devices = Some(second_mount_devices);
    let row_root = *second_mount.output.row_publish.root();
    assert_ne!(
        row_root.allocation_record_tree_root,
        NodePointer::empty_root(),
        "写行那次发布把它那片分配记录树的根写进根记录"
    );
    let on_disk = parse_node_pointer(
        &root_slot_bytes_on_disk(&formatted, row_root.checkpoint_txg)
            [ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_POINTER],
    );
    assert_eq!(
        (
            on_disk.all_zero,
            on_disk.birth_txg,
            on_disk
                .locations
                .map(|location| (location.device, location.slot))
        ),
        (
            false,
            row_root.checkpoint_txg.0,
            row_root
                .allocation_record_tree_root
                .locations
                .map(|location| (location.device.0, location.slot.0))
        ),
        "写行那一版：盘上根槽 342..428 按字段表解出来就是那片分配记录树的指针"
    );
    let current = second_mount.current;
    assert_eq!(
        current.root().allocation_record_tree_root,
        row_root.allocation_record_tree_root,
        "暖机照抄写行那一版的这一项：第一个文件版本接的那一版这一项非零"
    );

    let mut allocator = second_mount.allocator;
    let publish_parameters = parameters();
    let (first_file, overwritten) = {
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        let first_file = publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(2),
            current.record_bytes(),
        )
        .expect("第一个文件版本");
        let overwrite_content: Vec<u8> = file_content().iter().rev().copied().collect();
        let overwritten = publish_overwrite(
            &mut writer,
            &mut allocator,
            &first_file,
            FirstFile {
                content: &overwrite_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(2),
        )
        .expect("覆盖写一版");
        (first_file, overwritten)
    };
    for (version_name, version) in [
        ("第一个文件版本", &first_file),
        ("覆盖写那一版", &overwritten),
    ] {
        assert_eq!(
            version.root.allocation_record_tree_root,
            NodePointer::empty_root(),
            "{version_name}（txg {}）：交回的根记录里分配记录树根指针全零",
            version.root.checkpoint_txg.0
        );
        assert!(
            root_slot_bytes_on_disk(&formatted, version.root.checkpoint_txg)
                [ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT_POINTER]
                .iter()
                .all(|byte| *byte == 0),
            "{version_name}（txg {}）：盘上根槽 342..428 全零",
            version.root.checkpoint_txg.0
        );
    }
}

/// 把最新那条根的分配记录树根指针抹成全零、重新自证、原样写回它在根环里的那个槽：
/// 造一份「实现写不出来」的镜像（R8 今天只剩这一类对象）。
fn plant_a_root_without_its_allocation_record_tree(
    formatted: &mut FormattedPool,
    root: &singlefs_core::root_record::RootRecord,
) {
    let planted = singlefs_core::root_record::RootRecord {
        allocation_record_tree_root: NodePointer::empty_root(),
        ..*root
    };
    let target = target_for_publish(root.checkpoint_txg, geometry().root_ring_slots_per_region);
    let offset = slot_offset(target, geometry().fixed_structure_slot_spacing);
    let device = parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let slot_bytes = usize::try_from(parameters().geometry.physical_block_size).expect("根槽宽");
    let devices = formatted.devices.as_mut().expect("镜像还开着");
    let (_, image) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("区域那块盘在池里");
    image
        .write_at(
            offset,
            &planted.to_slot(slot_bytes),
            WriteDurability::ForceUnitAccess,
        )
        .expect("种根写得进去");
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_writable_mount_of_a_formatted_pool::writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_writable_mount_of_a_formatted_pool::writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_writable_mount_of_a_formatted_pool::a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold() {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_writable_mount_of_a_formatted_pool::a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold,
    );
}

/// D8（核心索引结构） 已定项 8 ②「号永不重发」在崩溃恢复那一形上（原先由回退到暖机根之后再发第一个文件版本那几条用例护，
/// 回退改成挂着时的向前发布之后那一形没有了，D23（journal 的角色与格式） 已定项 14）：只做过 mkfs 的池可写挂载（实例 1，暖机），
/// 再挂一次（实例 2，写行 txg 3、暖机 txg 4）发第一个文件版本（txg 5，十三棵树 65536..65548），崩在 txg 5 的根槽 FUA 之后、系统配置轮换之前
/// （录制流切到那一写为止：系统配置只见证到 txg 4）。可写挂载一次，txg 5 的根槽与它那条记录点名的一个单元（数据单元盘 0 那一份）读不出——
/// 择根落到实例 2 的暖机根 (2, 4)（树表 0 条、根上水位 65536），txg 5 那条记录施加前验点名单元验不过、不施加；系统配置见证到的就是 (2, 4)
/// 那次，C554 乙不重读、照常挂载（见证过的根暂时读不出会被拒，`a_mount_that_cannot_read_a_newer_root_rereads_once_then_refuses_writable.rs`）。
/// 新实例 3 的水位按根环与环里记录取 max = 65549。实例 3 接着发第一个文件版本：
/// - 十三棵树从 65549 发 65549..65561、新水位 65562，不从那一版根上的 65536 重发（照 mkfs 水位发就与盘上实例 2 的节点撞号）；
/// - 「树建没建」按树表条数判（0 条），不按水位是不是 65536（水位 65549、树表 0 条照样发得出第一个文件版本）；
/// - 只读挂载沿新根读回这一版的内容：中央映射树是 65553，按根记录里映射根指针的出生树认，不写死 65540。
#[test]
fn the_first_file_version_after_a_recovery_dropped_an_earlier_one_issues_its_trees_above_the_ring_watermark_and_reads_back(
) {
    let mut formatted = format_pool("first-file-after-a-recovery-dropped-an-earlier-one");
    let mut first = formatted.reopen_recorded();
    let first_instance = mount_writable(&parameters(), &mut first).expect("第一次可写挂载");
    assert_eq!(first_instance.output.instance, InstanceGeneration(1));
    drop(first_instance);
    let second_instance = mount_writable(&parameters(), &mut first).expect("第二次可写挂载");
    assert_eq!(second_instance.output.instance, InstanceGeneration(2));
    let PoolVersion::WithoutFile(version_of_the_second_instance) = &second_instance.current else {
        panic!("只做过 mkfs 的池上现行那一版树表 0 条")
    };
    let mut allocator_of_the_second_instance = second_instance.allocator.clone();
    let dropped_first_file = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, first.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator_of_the_second_instance,
            &version_of_the_second_instance.root,
            FirstFile {
                content: &file_content(),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            second_instance.output.instance,
            &version_of_the_second_instance.record_bytes,
        )
        .expect("实例 2 发第一个文件版本")
    };
    assert_eq!(
        dropped_first_file.tree_identifiers.highest().0,
        65548,
        "实例 2 从 mkfs 的水位 65536 发 65536..65548"
    );
    assert_eq!(
        dropped_first_file.root.checkpoint_txg,
        CheckpointTxg(5),
        "实例 2 写行 txg 3、暖机 txg 4，第一个文件版本 txg 5"
    );
    formatted.devices = Some(first);
    let crashed = devices_crashed_right_after_the_root_slot_of(
        &formatted,
        dropped_first_file.root.checkpoint_txg,
    );
    let named_data_unit = dropped_first_file.data_pointers[0].locations[0];
    let unreadable = SharedUnreadableRanges::new(
        vec![
            unreadable_root_slot_of(
                dropped_first_file.root.checkpoint_txg,
                FailingReadsOfARange::Every,
            ),
            UnreadableRange {
                device: named_data_unit.device,
                offset_in_bytes: named_data_unit.slot.to_device_offset().0,
                length_in_bytes: singlefs_format::DATA_UNIT_BYTES,
                failing_reads: FailingReadsOfARange::Every,
            },
        ],
        UnreadableRangeReadBack::DeviceError,
    );
    let mut reopened_with_the_fault = with_unreadable_ranges(crashed, &unreadable);
    let third_instance = mount_writable(&parameters(), &mut reopened_with_the_fault).expect(
        "txg 5 的根读不出、那条记录验不过：择根落到 (2, 4)，系统配置见证到的就是它，照常可写挂载",
    );
    let mut reopened = without_unreadable_ranges(reopened_with_the_fault);
    assert_eq!(
        (
            third_instance.output.instance,
            third_instance.output.effective_root.instance,
            third_instance.output.effective_root.checkpoint_txg,
            third_instance.output.rereads.read_stage
        ),
        (
            InstanceGeneration(3),
            InstanceGeneration(2),
            CheckpointTxg(4),
            ReadStageSettled::OnTheFirstRead {
                first_read: SelectedVersionAgainstTheWitness {
                    selected_version: RollbackTarget {
                        instance: InstanceGeneration(2),
                        checkpoint_txg: CheckpointTxg(4),
                    },
                    witness: NewerPublishWitness {
                        witnessed_journal_counter: 4,
                        comparison:
                            WitnessedCounterComparison::AgainstTheSelectedVersionsLastRecord {
                                selected_version_last_record_counter: 4,
                            },
                    },
                },
            }
        ),
        "实例 3 接在实例 2 的暖机根 (2, 4) 上：系统配置见证到 jsn 4，就是它，不重读"
    );
    let PoolVersion::WithoutFile(version_of_the_third_instance) = &third_instance.current else {
        panic!("落到实例 2 的暖机根：现行那一版树表 0 条")
    };
    assert_eq!(
        version_of_the_third_instance.root.tree_identifier_watermark, 65549,
        "实例 3 那一版带根环与环里记录的水位 max：实例 2 的记录带的 65549"
    );
    let mut allocator_of_the_third_instance = third_instance.allocator.clone();
    let content = (0..3300usize)
        .map(|index| u8::try_from((index * 7 + 17) % 253).expect("小于 256"))
        .collect::<Vec<u8>>();
    let first_file_of_the_third_instance = {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, reopened.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator_of_the_third_instance,
            &version_of_the_third_instance.root,
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            third_instance.output.instance,
            &version_of_the_third_instance.record_bytes,
        )
        .expect("水位 65549、树表 0 条：照样发得出第一个文件版本")
    };
    assert_eq!(
        (
            first_file_of_the_third_instance
                .tree_identifiers
                .highest()
                .0,
            first_file_of_the_third_instance
                .root
                .tree_identifier_watermark,
            first_file_of_the_third_instance
                .root
                .mapping_root
                .head
                .birth_tree
                .0
        ),
        (65561, 65562, 65553),
        "实例 3 从 65549 发 65549..65561、新水位 65562，中央映射树是 65553"
    );
    let image = memory_pool_of_sparse_devices(&reopened);
    let mounted = singlefs_core::mounted_read::mount_read_only(&image).expect("只读挂载");
    let file = mounted
        .mounted
        .open_file(
            &image,
            singlefs_core::address::InodeNumber(singlefs_core::transaction::FIRST_INODE_NUMBER),
        )
        .expect("打开文件");
    let read_back = file
        .read_at(
            &image,
            singlefs_core::address::FileOffsetInBytes(0),
            u64::try_from(content.len()).expect("3300"),
        )
        .expect("只读挂载读得回这一版");
    assert_eq!(read_back.bytes, content, "读回实例 3 那一版的内容");
}

/// 场景见 `scenarios_of_writable_mount_of_a_formatted_pool::crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_refuses_writable_when_the_read_fault_persists_through_the_one_reread`：
/// 读故障持续到重读之后 M3 拒可写，这条路上一张镜像都不交给池级 checker（与读故障撤在重读之前那一条共用一个场景函数），checker 档没有同名用例。
#[test]
fn crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_refuses_writable_when_the_read_fault_persists_through_the_one_reread(
) {
    scenarios_of_writable_mount_of_a_formatted_pool::crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_refuses_writable_when_the_read_fault_persists_through_the_one_reread(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_writable_mount_of_a_formatted_pool::crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_chooses_it_when_the_read_fault_is_lifted_before_the_one_reread`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_chooses_it_when_the_read_fault_is_lifted_before_the_one_reread(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_writable_mount_of_a_formatted_pool::crash_recovery_that_cannot_read_the_witnessed_row_publish_of_the_version_without_file_chooses_it_when_the_read_fault_is_lifted_before_the_one_reread,
    );
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_writable_mount_of_a_formatted_pool::crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them(
) {
    PoolCheckerImages::run_a_harness_scenario_leaving_its_images_to_the_checker_tier(
        scenarios_of_writable_mount_of_a_formatted_pool::crash_recovery_abandoning_the_unwitnessed_row_publish_of_the_version_without_file_keeps_its_tree_table_and_allocation_record_tree_root_isolated_once_the_floor_passes_them,
    );
}

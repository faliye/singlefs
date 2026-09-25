//! 里程碑「第二个事务」步 3 的另一格（2026-09-17 用户定：只做过 mkfs 的池允许可写挂载）：mkfs 之后进程退出、重开两个镜像走可写挂载——
//! 恢复择到 mkfs 的第 0 代根、环里一条记录都没有；取号 1；上一个实例是 0，不写行（D18（块里携带什么信息） 已定项 11）；
//! 树表 0 条 ⇒ 写行那次发布与暖机都写零个单元（D16（发布语义） 已定项 9），txg 1 落盘 1、txg 2 落盘 0；之后在同一个进程里发布第一个文件版本（txg 3）。
//! mkfs 之后的整条录制流与 mkfs 同一个进程里跑第一个事务逐字节相同（第一次可写挂载要写的区间是空的，第一个事务的字节不变）；
//! 冷启动读回那个文件；每一步的镜像池级 checker 一条违例都没有（没有文件的两步里没有记账树与 inode 树的 8 条报不适用）。
//!
//! 2026-09-23 用户定案「走到底，让那条零故障历史真跑通」之后这一份还罩着挂载不止一次的那几格：
//! 树表 0 条的一版上**写实例表行**（只重写实例表一个单元）、第一个文件版本从现行那一版接着算 txg 与 jsn、
//! 写过行的那一版上再挂载一次被拒（被换下的那片实例表记在哪没有条款）、环里没有带文件的根时回退到树表 0 条的根照常做。
//! 端到端那一条是 `a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold`。

mod common;

use common::{
    build_pool, crash_state_devices, disk_snapshot, file_content, format_pool, geometry,
    memory_pool_of_sparse_devices, parameters, FormattedPool, FIXED_WRITE_TIME_SECONDS,
};
use singlefs_checker::image::{parse_node_pointer, InvariantVerdict};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::pointer::NodePointer;

use singlefs_core::journal::{
    back_chain_of, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
};
use singlefs_core::make_filesystem::{INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::{
    mount_rollback, mount_writable, InstanceRow, MountError, RollbackTarget, ShadowLedger,
};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryFailure,
    RecoveryOutcome,
};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_without_units, warm_up,
    FirstFile, PoolVersion, PoolWriter, PublishError,
    TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees, ZeroUnitPublishPlan,
};
use singlefs_harness::crash::writes_and_segments;
use singlefs_harness::fault_injection::{
    FaultCounting, FaultDeviceSelector, FaultInjectingBlockDevice, FaultOccurrence, FaultPlacement,
    FaultSchedule, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::SharedStream;

/// 每块盘第 `ordinal` 次读系统配置槽 0（偏移 0）时注入一次读错，别的读写原样交给内层
/// （m2-emptypool-nonempty-r1 云端攻方腿模型 `opus_attack_emptypool.rs` 第 294 行起的做法）。
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
/// 判定之前每块盘各读槽 0 六次：`mount_writable` 择系统配置一次（`recovery::choose_system_configuration`）、择根读回退见证一次
/// （`recovery::choose_root` → `rollback_witness_of_the_pool`）；重放里择系统配置、读回退见证各一次（`recovery::replay_journal`）；
/// 重建分配器时择根读一次、影子账再读一次回退见证（`mount::rebuilt_allocator`）。判定之后还有两读：写回退见证表之前读一次
/// （`mount::rollback_witness_tables_of_this_mount`）、取号写之前重算一次。数法：给读槽 0 的地方各打一行，照这一次挂载的次序数出来的；
/// 读法变了（多读或少读一次），这个数跟着改，注入点就还是判定那一读。
const DECISION_READ_OF_SYSTEM_CONFIGURATION_SLOT_ZERO_IN_A_WRITABLE_MOUNT: u64 = 7;

/// 取号之前判定算出的号与取号写之前重算的号不同，取号不写、报错（m2-emptypool-nonempty-r1 云端攻方腿 Z2）：mkfs 之后第一次可写挂载崩在
/// 取号两写之后（同一个进程里取号就停、镜像关掉；两盘系统配置槽 0 已是号 1），重开时两块盘在判定那一读上（第 7 次读系统配置槽 0，
/// `DECISION_READ_OF_SYSTEM_CONFIGURATION_SLOT_ZERO_IN_A_WRITABLE_MOUNT`）各报一次瞬时读错——
/// 判定那一遍只看到 mkfs 的槽 1（号 0）、算出号 1、要写的行区间为空放行；取号重算读到号 1、算出号 2 ⇒ 返回
/// `InstanceGenerationChangedBeforeAcquisition { expected: 1, recomputed: 2 }`，`DiskSnapshot` 不变。
#[test]
fn transient_system_configuration_read_errors_between_the_refusal_and_the_acquisition_refuse_before_any_write(
) {
    let mut formatted = format_pool("step-three-formatted-transient-read-error");
    {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        assert_eq!(
            acquire_instance(&mut writer).expect("取号"),
            InstanceGeneration(1)
        );
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
            is_rollback: false,
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

/// 环里留着一条孤记录时，只做过 mkfs 的池照常可写挂载（2026-09-23 用户定案收窄 R4：`FIRST_TRANSACTION_TXG` 只管 mkfs
/// 同一个进程里那条流，不管任何池的第一个文件版本，所以「新实例的第一次发布不是 txg 1、jsn 1」不再是拒绝的理由）：
/// mkfs 之后第一次可写挂载崩在取号两写与 txg 1 的记录两写都持久、txg 1 的根槽没持久；两块盘系统配置槽 0（取号写进号 1 的那一槽）
/// 各坏一个字节——择系统配置只剩 mkfs 的槽 1、根环只有第 0 代根，要取的号 1、要写的行为空；环里那条 txg 1 的记录让新实例从
/// txg 2、jsn 2 起 ⇒ 零单元发布 txg 2（落盘 0），暖机推到 txg 4（落盘 1）才覆盖两块盘，之后第一个文件版本接着写 txg 5。
#[test]
fn formatted_pool_mount_starting_after_a_leftover_record_publishes_from_the_next_txg() {
    let mut formatted = format_pool("step-three-formatted-leftover-record");
    let mut first_mount_devices = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first_mount_devices).expect("第一次可写挂载");
    formatted.devices = Some(first_mount_devices);
    let operations = formatted.retained_operations();
    let (writes, segments) =
        writes_and_segments(&operations[formatted.mkfs_operation_count..], &geometry());
    let mut persisted = vec![false; writes.len()];
    for write_index in segments[0].iter().chain(segments[1].iter()) {
        persisted[*write_index] = true;
    }
    let crash_stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(
        &formatted.memory_pool_after_mkfs(),
        &writes,
        &persisted,
        &crash_stream,
    );
    for (_, device) in &mut devices {
        let mut sector = vec![0u8; 512];
        device
            .inner()
            .read_at(DeviceOffsetInBytes(0), &mut sector)
            .expect("读系统配置槽 0");
        sector[100] ^= 0xff;
        device
            .inner_mut()
            .image
            .write(DeviceOffsetInBytes(0), &sector);
    }
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
/// ② 八棵树从那一版的水位（盘上读来的 8 字节）起连号发，而水位离 `u64::MAX` 不到八个号
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
    let mut root_with_no_room_for_eight_trees = warm_up_root;
    root_with_no_room_for_eight_trees.tree_identifier_watermark = u64::MAX - 3;
    let recorded_before_the_watermark_case = formatted.stream.operations().len();
    let refused_for_the_watermark = publish_first_file(
        &mut writer,
        &mut allocator,
        &root_with_no_room_for_eight_trees,
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
        "水位离 u64::MAX 不到八个号：{:?}",
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

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters().geometry.root_ring_slots_per_region,
    );
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

/// 最新的根下面还没有记账树、inode 树与分配记录树时（mkfs 之后、零单元发布之后）checker 报不适用的那 20 条；
/// 其余 24 条要真被评估过且成立。I-3.11（已分配减 defer 等于最新根走读） 与 I-3.1（已分配统计对得上） 同一个理由：最新根下面没有记账树。I-9.14（树表条目的诞生 txg 跨根不变） 在这里报不适用是因为 mkfs 种的第 0 版树表是空的：
/// 一棵树的条目都没有，跨根比不出来（有了文件版本之后也只有一条根有条目，见 `NOT_APPLICABLE_WITH_ONE_FILE_VERSION`）；
/// I-5.4（分配记录罩住的槽互不相交） 同一个理由：第 0 版树表里没有分配记录树，候选集里一条根的记录都走不到；
/// I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号） 也是这个理由。
/// I-1.10（码 2 条目宽等于字段表宽） 也是这个理由：一条树表条目都没有，就没有哪个码 2 节点的条目宽可比。
/// I-8.7（实例内事务号不重号） 在这里报不适用是因为环里一条非 0 事务号的记录都没有：写行与暖机的空发布都写 0
/// （D23（journal 的角色与格式） 已定项 19 ①）。I-8.8（前缀里的事务不被切开） 是因为没有哪个非 0 事务号落了两条记录、
/// 也没有哪一组一条提交标记都没有。I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉） 与 I-9.7 同一个理由：一条 inode 记录都没有。
/// I-7.9（回退下界 F 不高于抬 F 的上限） 是因为没抬过 F：每条根带的 F 都是 0（这个文件里哪一步都不抬，下面四张表都有它）。
const NOT_APPLICABLE_WITHOUT_FILE: [&str; 20] = [
    "I-1.10", "I-3.1", "I-3.9", "I-3.10", "I-3.11", "I-5.2", "I-5.4", "I-7.9", "I-8.7", "I-8.8",
    "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14", "I-9.15",
];

/// mkfs 刚写完那一份要多报两条不适用：journal 环整段是 0，一条自证过的记录都没有 ⇒ I-8.6（反向链算法） 与
/// I-8.9（一次发布的记录序号连续且只有末条带标志） 都没有判的对象。
/// 可写挂载之后环里有记录（写行 / 暖机的空发布），它们就真被评估过了——所以这两条只在 mkfs 之后那一格。
const NOT_APPLICABLE_RIGHT_AFTER_MKFS: [&str; 22] = [
    "I-1.10", "I-3.1", "I-3.9", "I-3.10", "I-3.11", "I-5.2", "I-5.4", "I-7.9", "I-8.6", "I-8.7",
    "I-8.8", "I-8.9", "I-9.1", "I-9.2", "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13",
    "I-9.14", "I-9.15",
];

/// 写完第一个文件版本之后仍报不适用的那四条：
/// I-7.9（回退下界 F 不高于抬 F 的上限）——没抬过 F，没有抬 F 的根可判（坏镜像与阳性对照在 `checker_known_bad_images.rs` 步 5 那段历史上）；
/// I-9.14——树表条目只出现在这一条根的树表里（mkfs 种的第 0 版树表是空的、两次暖机空发布不写树表），
/// 它要同一棵树的条目出现在两个树表单元里才比得出来；
/// I-8.7——这个实例只发过一个承载事务的版本（事务号 1），环里其余记录都是事务号 0 的空发布，
/// 同一实例凑不出两条非 0 的号来比；
/// I-8.8（前缀里的事务不被切开）——第一版一事务一条、每条都带提交标记，③ ④ 没有对象。
/// 三条比得出来的镜像都在 `checker_known_bad_images.rs`（再覆盖写一次：树表两版、事务号 1 与 2；
/// B 之后再接一事务两条记录的阳性对照与坏镜像）。
const NOT_APPLICABLE_WITH_ONE_FILE_VERSION: [&str; 4] = ["I-7.9", "I-8.7", "I-8.8", "I-9.14"];

/// 树表 0 条的一版上写行那次发布之后：比 `NOT_APPLICABLE_WITHOUT_FILE` 少 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）
/// 与 I-1.10（码 2 条目宽等于字段表宽）两条。写行那次发布写了一棵分配记录树、根指针住根记录（C512（树表 0 条的一版上被换下的单元记在哪），
/// D16（发布语义） 已定项 9），里面未释放的那几条记录（被换下的 mkfs 那片实例表已带释放标志，归 I-3.9）起点槽上都读得出单元头，
/// I-3.10 在这一格真被评估过；这棵树按位置寻址（D8（核心索引结构） 已定项 14），checker 从根记录那一项按位置走下去、每个节点判条目宽，
/// I-1.10 也真被评估过。
/// I-5.4（分配记录罩住的槽互不相交） 仍报不适用：它只从树表条目找分配记录树，不读根记录直接持有的那一棵。
const NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE: [&str; 18] = [
    "I-3.1", "I-3.9", "I-3.11", "I-5.2", "I-5.4", "I-7.9", "I-8.7", "I-8.8", "I-9.1", "I-9.2",
    "I-9.4", "I-9.6", "I-9.7", "I-9.10", "I-9.12", "I-9.13", "I-9.14", "I-9.15",
];

/// 这一步的镜像上 checker 的每一条判决逐条核：`not_applicable` 里的报不适用，其余每一条都真被评估过且成立（一条违例都没有）。
fn assert_checker_verdicts(pool: &FormattedPool, step: &str, not_applicable: &[&str]) {
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(
        verdicts.len(),
        singlefs_checker::image::IMPLEMENTED_INVARIANTS.len()
    );
    for (invariant, verdict) in &verdicts {
        if not_applicable.contains(invariant) {
            assert!(
                matches!(verdict, InvariantVerdict::NotApplicable(_)),
                "{step}：{invariant} 该报不适用：{verdict:?}"
            );
        } else {
            assert_eq!(
                *verdict,
                InvariantVerdict::Holds,
                "{step}：{invariant} 要真被评估过且成立"
            );
        }
    }
}

/// 崩在暖机与第一个文件版本之间，重开照样写得出行：第一个事务取号、暖机两次之后、第一个文件版本之前崩溃
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
            is_rollback: false,
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

/// 建池 → 可写挂载（不写文件）→ 退出 → 再可写挂载（写行）→ 退出 → **第三次可写挂载做得成**
/// （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案：根记录加一项放分配记录树的根指针）：
/// 写行那次发布把这一版的全部分配记录写成一个节点，被换下的那片实例表带着已释放标志进了它，
/// 重开时从它重建账 ⇒ **那一片没有被当空闲槽发出去**。
/// 三条断言各盯一处：那一片在重建出来的账里带着已释放标志、第三次挂载新写的那几个单元一个都没落在它的槽上、池级 checker 一条违例都没有。
/// 判别力自证：把写行那次发布里建最小分配记录树那一步去掉（`crates/mutations.tsv` 那两行），这三条各自变红。
#[test]
fn the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool(
) {
    let mut formatted = format_pool("step-three-formatted-third-mount-succeeds");
    let mut first = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first).expect("第一次可写挂载：不写行");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let row_written = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    let root_after_the_row = *row_written.current.root();
    let swapped_out_instance_table = formatted
        .genesis
        .root
        .instance_table
        .locations
        .map(|location| location.slot);
    formatted.devices = Some(second);
    assert_ne!(
        root_after_the_row.instance_table.head.birth_txg,
        CheckpointTxg(0),
        "写过行之后实例表的诞生 txg 不再是 0"
    );
    assert_ne!(
        root_after_the_row.allocation_record_tree_root,
        NodePointer::empty_root(),
        "写行那次发布把分配记录树的根写进了根记录"
    );

    let mut third = formatted.reopen_recorded();
    let third_mount = mount_writable(&parameters(), &mut third).expect("第三次可写挂载做得成");
    let reused = third_mount
        .output
        .row_publish
        .root()
        .instance_table
        .locations
        .iter()
        .any(|location| swapped_out_instance_table.contains(&location.slot));
    formatted.devices = Some(third);
    assert!(
        !reused,
        "被换下的那片实例表（槽 {swapped_out_instance_table:?}）没有被当空闲槽发给第三次挂载写的那片"
    );
    let rebuilt_record_says_released = third_mount
        .allocator
        .records()
        .iter()
        .filter(|record| swapped_out_instance_table.contains(&record.slot))
        .all(|record| record.is_released);
    let rebuilt_record_count = third_mount
        .allocator
        .records()
        .iter()
        .filter(|record| swapped_out_instance_table.contains(&record.slot))
        .count();
    assert_eq!(rebuilt_record_count, 2, "两盘各一条记录罩着被换下的那一片");
    assert!(
        rebuilt_record_says_released,
        "重建出来的账里那两条记录带着已释放标志：写行那次发布的释放从盘上取回来了"
    );
    let verdicts = check_pool_image(&formatted.memory_pool());
    for (invariant, verdict) in &verdicts {
        assert!(
            !matches!(verdict, InvariantVerdict::Violated(_)),
            "第三次可写挂载之后 {invariant} 判红：{verdict:?}"
        );
    }
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
/// 中央映射树根指针 256、分配记录树根指针 342、算法类型 428）。
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

/// 验收：取号 1、不写行；写行那次发布 (1, 1)、jsn 1、事务号 0、反向链 0、零单元、落盘 1；暖机一次 (1, 2)、jsn 2、反向链接 jsn 1、落盘 0；
/// 分配器只有 mkfs 的两个单元（每盘 3 槽）；第一个文件版本 (1, 3)、jsn 3、事务号 1，数据单元落 50180、换下 mkfs 那片树表；
/// mkfs 之后的录制流与第一个事务那条逐字节相同、写出的那一版也相同；冷启动读回文件；checker 一条违例都没有——mkfs 之后
/// 22 条成立（环是空的，I-8.6 也报不适用）、挂载之后 23 条成立，没有记账树 / inode 树 / 分配记录树的那几条报不适用，
/// 第一个文件版本之后 36 条成立、I-9.14、I-8.7 与 I-8.8 仍报不适用（树表条目只出现在这一条根的树表里；同一实例只有一条非 0 事务号的记录；
/// 没有哪个事务号落了两条记录、每条都带提交标记）。
#[test]
fn writable_mount_of_a_formatted_pool_takes_instance_one_publishes_two_zero_unit_roots_and_the_first_file_version_reads_back_cold(
) {
    let mut formatted = format_pool("step-three-formatted-pool");
    assert_checker_verdicts(&formatted, "mkfs 之后", &NOT_APPLICABLE_RIGHT_AFTER_MKFS);

    let mut devices = formatted.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("只做过 mkfs 的池可写挂载");
    formatted.devices = Some(devices);
    let output = &mounted.output;
    assert_eq!(
        output.instance,
        InstanceGeneration(1),
        "取号 = max(系统配置 0, 根环 0) + 1"
    );
    assert_eq!(
        (
            output.chosen_root.instance,
            output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(0), CheckpointTxg(0)),
        "所选根是 mkfs 的第 0 代根"
    );
    assert_eq!(output.effective_root, output.chosen_root);
    assert_eq!(output.journal.valid_records, 0, "环里一条记录都没有");
    assert!(output.rows_written.is_empty(), "上一个实例是 0：不写行");
    let PoolVersion::WithoutFile(row) = &output.row_publish else {
        panic!("树表 0 条：写行那次发布写零个单元")
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
        (InstanceGeneration(1), CheckpointTxg(1), 1, 0, 0, 0),
        "txg = max(根环 0, 记录无) + 1；jsn 从 1 起；事务号 0；本实例第一条反向链 0；不点名"
    );
    assert_eq!(
        (
            row.root.tree_table,
            row.root.instance_table,
            row.root.mapping_root
        ),
        (
            formatted.genesis.root.tree_table,
            formatted.genesis.root.instance_table,
            formatted.genesis.root.mapping_root
        ),
        "根记录照 mkfs 的根：树表、实例表、映射树根都没换"
    );
    assert_eq!(region_device(1), DeviceIdentity(1), "txg 1 落盘 1");
    assert_eq!(
        output.warm_up_publishes.len(),
        1,
        "txg 2 落盘 0，一次就覆盖两块盘"
    );
    let PoolVersion::WithoutFile(warm_up) = &output.warm_up_publishes[0] else {
        panic!("树表 0 条：暖机写零个单元")
    };
    assert_eq!(
        (
            warm_up.root.checkpoint_txg,
            warm_up.record.counter,
            warm_up.record.transaction,
            warm_up.record.back_chain
        ),
        (CheckpointTxg(2), 2, 0, back_chain_of(&row.record_bytes))
    );
    assert_eq!(region_device(2), DeviceIdentity(0), "txg 2 落盘 0");
    assert_eq!(
        mounted.current, output.warm_up_publishes[0],
        "接下来的发布接在 txg 2 后面"
    );
    assert_eq!(
        output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],
        "没有被抛弃的根"
    );
    for device_map in &mounted.allocator.devices {
        assert_eq!(
            (device_map.allocated_slots(), device_map.deferred_slots()),
            (3, 0),
            "只有 mkfs 的实例表 2 槽与树表 1 槽"
        );
    }
    assert_checker_verdicts(&formatted, "可写挂载之后", &NOT_APPLICABLE_WITHOUT_FILE);
    assert_eq!(
        recover(&formatted.memory_pool(), JournalPolicy::Consult).outcome,
        RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(2))
        }
    );

    // 第一个文件版本：接在 txg 2 那条零单元发布后面。
    let mut allocator = mounted.allocator;
    let current = mounted.current;
    let content = file_content();
    let first = {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
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
        (
            first.root.instance,
            first.root.checkpoint_txg,
            first.record.counter,
            first.record.transaction,
            first.record.back_chain
        ),
        (
            InstanceGeneration(1),
            CheckpointTxg(3),
            3,
            1,
            back_chain_of(current.record_bytes())
        )
    );
    assert_eq!(
        first.data_pointers[0].locations[0].slot,
        SlotNumber(50180),
        "mkfs 的两个单元占着 50176–50178"
    );
    assert_eq!(
        first.released,
        vec![Placement {
            slot: SlotNumber(50178),
            span: 1
        }],
        "换下 mkfs 那片第 0 版树表"
    );

    let reference = build_pool("step-three-formatted-pool-reference");
    assert_eq!(
        formatted.retained_operations()[formatted.mkfs_operation_count..],
        reference.retained_operations()[reference.mkfs_operation_count..],
        "mkfs 之后的录制流与 mkfs 同一个进程里跑第一个事务逐字节相同"
    );
    assert_eq!(first, reference.output, "写出的那一版也相同");

    assert_checker_verdicts(
        &formatted,
        "第一个文件版本之后",
        &NOT_APPLICABLE_WITH_ONE_FILE_VERSION,
    );
    let reopened = formatted.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content
        }
    );
    assert_eq!(report.journal.valid_records, 3);
}

/// **2026-09-23 用户定案的那条零故障历史，端到端**：建池 → 可写挂载 → 不写文件 → 正常退出 → 再可写挂载 → 写文件 →
/// 冷启动读回来逐字节相同。今天被一条拒绝链挡着的每一道在这里都要放行：
/// R2（回退候选集里排除树表 0 条）、R1（树表 0 条的一版上写实例表行）、R4（空池挂载的形状判定）、
/// `publish_first_file` 写死的 txg / jsn / 实例表指针。
///
/// 每一格的数：
/// - 第一次可写挂载：取号 1、不写行、零单元发布 txg 1（落盘 1）、暖机 txg 2（落盘 0）；
/// - 第二次可写挂载：取号 2、写行 (1, 2, 0)，写行那次发布**只重写实例表一个单元**、txg 3 jsn 3（落盘 0）、暖机 txg 4（落盘 1）；
/// - 第一个文件版本：txg 5、jsn 5、事务号 1，照抄的是**写行之后那一版**的实例表指针（不是 mkfs 那一版——照抄错了这一行就丢）；
/// - 冷启动：择 (2, 5) 读回那个文件，内容逐字节相同；盘上那片实例表里仍是写行时那一条。
#[test]
fn a_formatted_pool_mounted_twice_writes_the_row_then_the_first_file_and_reads_it_back_cold() {
    let mut formatted = format_pool("step-three-formatted-two-mounts-then-a-file");

    // 一、第一次可写挂载：不写文件，正常退出。
    let mut first_mount_devices = formatted.reopen_recorded();
    let first_mount =
        mount_writable(&parameters(), &mut first_mount_devices).expect("第一次可写挂载");
    formatted.devices = Some(first_mount_devices);
    assert_eq!(first_mount.output.instance, InstanceGeneration(1));
    assert!(
        first_mount.output.rows_written.is_empty(),
        "上一个实例是 0：不写行"
    );
    assert_eq!(
        (
            first_mount.output.row_publish.root().checkpoint_txg,
            first_mount.current.root().checkpoint_txg
        ),
        (CheckpointTxg(1), CheckpointTxg(2))
    );
    assert_eq!(
        first_mount.output.row_publish.record().named.len(),
        0,
        "不写行那一次是零单元发布"
    );
    drop(first_mount);

    // 二、再可写挂载：写行。
    let mut second_mount_devices = formatted.reopen_recorded();
    let second_mount =
        mount_writable(&parameters(), &mut second_mount_devices).expect("第二次可写挂载");
    formatted.devices = Some(second_mount_devices);
    assert_eq!(second_mount.output.instance, InstanceGeneration(2));
    assert_eq!(
        second_mount.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(2),
            applied_transaction_high_water: 0,
            is_rollback: false,
        }],
        "上一个实例 1 那一行：选的根 txg 2、W = 0"
    );
    let PoolVersion::WithoutFile(row) = &second_mount.output.row_publish else {
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
        (InstanceGeneration(2), CheckpointTxg(3), 3, 0, 0, 6),
        "写行那次发布：txg 3、jsn 3、事务号 0、反向链 0、点名实例表与分配记录树五个节点（按位置寻址，D8（核心索引结构） 已定项 14）"
    );
    assert_eq!(
        row.root.tree_table, formatted.genesis.root.tree_table,
        "树表照抄 mkfs 那片 0 条的"
    );
    assert_ne!(
        row.root.instance_table.locations[0].slot,
        formatted.genesis.root.instance_table.locations[0].slot,
        "实例表 COW 到新落点：mkfs 那一片被换下"
    );
    assert_eq!(region_device(3), DeviceIdentity(0), "txg 3 落盘 0");
    assert_eq!(
        second_mount
            .output
            .warm_up_publishes
            .iter()
            .map(|publish| publish.root().checkpoint_txg)
            .collect::<Vec<_>>(),
        vec![CheckpointTxg(4)],
        "txg 4 落盘 1，一次就覆盖两块盘"
    );
    assert_eq!(region_device(4), DeviceIdentity(1));
    // mkfs 那片实例表进了 defer 队列（释放代 3）：还被根环里 txg ≤ 2 的候选根引用着，抬 F 之前不回收。
    for device_map in &second_mount.allocator.devices {
        assert_eq!(
            (device_map.allocated_slots(), device_map.deferred_slots()),
            (10, 2),
            "mkfs 的实例表 2 槽 + 树表 1 槽 + 新写的实例表 2 槽 + 这一版自己那棵分配记录树五个节点 5 槽（C512；按位置寻址，D8（核心索引结构） 已定项 14）；换下的 2 槽在 defer 队列里"
        );
    }
    assert_checker_verdicts(
        &formatted,
        "写行那次发布之后",
        &NOT_APPLICABLE_AFTER_THE_ROW_PUBLISH_WITHOUT_FILE,
    );

    // 三、写文件：接在写行之后那一版后面。
    let mut allocator = second_mount.allocator;
    let current = second_mount.current;
    assert_eq!(
        current.root().instance_table,
        row.root.instance_table,
        "暖机照抄写行那一版的实例表指针"
    );
    let content = file_content();
    let first_file = {
        let publish_parameters = parameters();
        let open_devices = formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, open_devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut allocator,
            current.root(),
            FirstFile {
                content: &content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            InstanceGeneration(2),
            current.record_bytes(),
        )
        .expect("第一个文件版本")
    };
    assert_eq!(
        (
            first_file.root.instance,
            first_file.root.checkpoint_txg,
            first_file.record.counter,
            first_file.record.transaction,
            first_file.record.back_chain
        ),
        (
            InstanceGeneration(2),
            CheckpointTxg(5),
            5,
            1,
            back_chain_of(current.record_bytes())
        ),
        "第一个文件版本 txg 5、jsn 5：从现行那一版接着算，不写死 3"
    );
    assert_eq!(
        first_file.root.instance_table, row.root.instance_table,
        "照抄的是写行之后那一版的实例表指针，不是 mkfs 那一版"
    );
    // 写行那一版自己那棵分配记录树的五个节点（按位置寻址，D8（核心索引结构） 已定项 14）：它那次发布点名的单元里实例表之外的那几个，
    // 先叶后根；第一个文件版本改的记录都在那两片叶里，五个都重写、都换下。
    let allocation_record_tree_nodes_of_the_row_version: Vec<Placement> = row
        .record
        .named
        .iter()
        .map(|named| named.locations[0].slot)
        .filter(|slot| *slot != row.root.instance_table.locations[0].slot)
        .map(|slot| Placement { slot, span: 1 })
        .collect();
    assert_eq!(allocation_record_tree_nodes_of_the_row_version.len(), 5);
    assert_eq!(
        first_file.released,
        std::iter::once(Placement {
            slot: TREE_TABLE_GENESIS_SLOT,
            span: 1
        })
        .chain(allocation_record_tree_nodes_of_the_row_version)
        .collect::<Vec<_>>(),
        "换下 mkfs 那片第 0 版树表，加写行那一版自己那棵分配记录树的五个节点（C512）"
    );

    // 四、冷启动读回来逐字节相同，行还在。
    let reopened = formatted.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(5)),
            content
        },
        "冷启动择 (2, 5) 读回那个文件，逐字节相同"
    );
    assert_eq!(report.journal.valid_records, 5);
    let system_configuration = choose_system_configuration(&reopened).expect("择系统配置");
    let newest = choose_root(&reopened, &system_configuration).expect("择根");
    let table = singlefs_core::recovery::instance_table_of_root(&reopened, &newest)
        .expect("盘上那片实例表解得开");
    assert_eq!(
        table.rows,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(2),
            applied_transaction_high_water: 0,
            is_rollback: false,
        }],
        "写行那一条还在：第一个文件版本照抄的是写行之后那一版的指针"
    );
    assert_checker_verdicts(
        &formatted,
        "第一个文件版本之后",
        &NOT_APPLICABLE_WITH_ONE_FILE_VERSION,
    );
}

/// 环里一条带文件版本的根都没有时，回退到树表 0 条的根照常做（R2 那一道：候选集只有三条，「树表 0 条」不在其内，
/// C493（回退候选集条文与实现说反话） 还清）：建池 → 可写挂载（不写文件，txg 1、2）→ 退出 → 再可写挂载（写行，txg 3、4）→ 退出 →
/// 回退到 (1, 2)。取号 3、写两行——被退回的实例 1 那一条回退行 (1, 2, 0)（flags bit0 = 1）与中间实例 (2, 0, 0)；
/// 回退行那次发布只重写实例表一个单元、txg 5 jsn 5，暖机到本实例的根覆盖每块盘；冷启动择新根读回「没有文件」；
/// 盘上那片新实例表里就是这两行；池级 checker 一条违例都没有。
///
/// 这一格同时钉住影子账认得树表 0 条的被抛弃根：第二次挂载写出去的那片实例表只被 (2, 3)、(2, 4) 引用，
/// 这次回退把它们抛弃，而它们没有分配记录树——影子账要按根记录那两条指针把它隔离，不然回退这次的新实例表就发在它上面
/// （2026-09-23 崩溃注入打中的那一格）。
/// 影子账罩着被抛弃根那片分配记录树节点（D23（journal 的角色与格式） 已定项 14：被抛弃时间线的根离开根环之前，它们引用的单元
/// 不许重新分配）。同一段历史：第二次可写挂载在树表 0 条的一版上写行（实例 2，txg 3 重写实例表与分配记录树、txg 4 暖机照抄），
/// 再回退到 (1, 2)——实例 2 那两条根被抛弃，它们引用的那片实例表（2 槽）与那棵分配记录树的五个节点（按位置寻址，D8（核心索引结构）
/// 已定项 14：4 GiB 两块盘上根在第 2 层，两块盘各一片叶、各一个第 1 层节点、根；根指针住根记录那一项）
/// 都只被被抛弃根引用，每块盘隔离 7 槽，那几个节点的槽回退之后不空闲；回退那次发布写的实例表与分配记录树一片都不落在它们上面。
/// 只认根记录里实例表与树表两条指针时那五个节点每盘少隔离 5 槽，回退之后它们是空闲槽。
#[test]
fn rolling_back_before_any_file_version_isolates_the_allocation_record_node_only_the_abandoned_roots_reference(
) {
    let mut formatted = format_pool("step-three-formatted-rollback-isolates-the-node");
    let mut first = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first).expect("第一次可写挂载：不写行");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let row_written = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    formatted.devices = Some(second);
    assert_ne!(
        row_written.current.root().allocation_record_tree_root,
        NodePointer::empty_root(),
        "写行那次发布把分配记录树的根写进了根记录"
    );
    // 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，写行那次写出五个节点（两块盘各自的叶 61、
    // 各自的第 1 层节点 0 与根），都点名在那次发布的记录里；实例表那一项之外就是它们。
    let abandoned_instance_table_slot = row_written.current.root().instance_table.locations[0].slot;
    let abandoned_node_slots: Vec<SlotNumber> = row_written
        .output
        .row_publish
        .record()
        .named
        .iter()
        .map(|named| {
            assert_eq!(named.locations[1].slot, named.locations[0].slot, "两盘同槽");
            named.locations[0].slot
        })
        .filter(|slot| *slot != abandoned_instance_table_slot)
        .collect();
    assert_eq!(
        abandoned_node_slots.len(),
        5,
        "写行那次发布写的分配记录树节点：两块盘各一片叶、各一个第 1 层节点、根"
    );

    let mut devices = formatted.reopen_recorded();
    let rolled_back = mount_rollback(
        &parameters(),
        &mut devices,
        RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(2),
        },
        ShadowLedger::On,
    )
    .expect("环里没有带文件的根：回退照常做");
    formatted.devices = Some(devices);
    assert_eq!(
        rolled_back.output.isolated_slots_per_device,
        vec![(DeviceIdentity(0), 7), (DeviceIdentity(1), 7)],
        "实例 2 那片实例表 2 槽加那五个分配记录树节点 5 槽，逐盘"
    );
    assert_eq!(rolled_back.output.abandoned_roots_unreadable, 0);
    for device_map in &rolled_back.allocator.devices {
        for abandoned_node_slot in &abandoned_node_slots {
            assert!(
                !device_map.is_free(*abandoned_node_slot),
                "盘 {:?}：被抛弃根那棵分配记录树的节点（槽 {abandoned_node_slot:?}）回退之后不空闲",
                device_map.device
            );
        }
    }
    let PoolVersion::WithoutFile(row) = &rolled_back.output.row_publish else {
        panic!("树表 0 条：回退行那次发布仍是「没有文件版本」的一版")
    };
    let instance_table_slot = row.root.instance_table.locations[0].slot;
    let written_by_the_rollback: Vec<SlotNumber> = row
        .record
        .named
        .iter()
        .map(|named| named.locations[0].slot)
        .chain([SlotNumber(instance_table_slot.0 + 1)])
        .collect();
    assert!(
        written_by_the_rollback.contains(&instance_table_slot)
            && written_by_the_rollback
                .contains(&row.root.allocation_record_tree_root.locations[0].slot),
        "回退那次发布点名了它写的实例表与分配记录树的根：{written_by_the_rollback:?}"
    );
    for abandoned_node_slot in &abandoned_node_slots {
        assert!(
            !written_by_the_rollback.contains(abandoned_node_slot),
            "回退那次发布写的实例表与分配记录树（槽 {written_by_the_rollback:?}）不落在被抛弃根那棵树的节点（槽 {abandoned_node_slot:?}）上"
        );
    }
}

#[test]
fn rolling_back_to_a_warm_up_root_before_any_file_version_rewrites_only_the_instance_table() {
    let mut formatted = format_pool("step-three-formatted-rollback-before-any-file");
    let mut first = formatted.reopen_recorded();
    mount_writable(&parameters(), &mut first).expect("第一次可写挂载：不写行");
    formatted.devices = Some(first);
    let mut second = formatted.reopen_recorded();
    let row_written = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    let instance_table_written_by_the_row_publish =
        row_written.current.root().instance_table.locations[0].slot;
    formatted.devices = Some(second);

    let warm_up_root = RollbackTarget {
        instance: InstanceGeneration(1),
        checkpoint_txg: CheckpointTxg(2),
    };
    let mut devices = formatted.reopen_recorded();
    let rolled_back = mount_rollback(&parameters(), &mut devices, warm_up_root, ShadowLedger::On)
        .expect("环里没有带文件的根：回退照常做");
    formatted.devices = Some(devices);
    assert_eq!(rolled_back.output.instance, InstanceGeneration(3));
    assert_eq!(
        rolled_back.output.rows_written,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(2),
                applied_transaction_high_water: 0,
                is_rollback: true,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            },
        ],
        "被退回的实例 1 那一条回退行，加中间实例 (2, 0, 0)"
    );
    let PoolVersion::WithoutFile(row) = &rolled_back.output.row_publish else {
        panic!("树表 0 条：回退行那次发布仍是「没有文件版本」的一版")
    };
    assert_eq!(
        (
            row.root.instance,
            row.root.checkpoint_txg,
            row.record.counter,
            row.record.named.len()
        ),
        (InstanceGeneration(3), CheckpointTxg(5), 5, 6),
        "txg = 环里最大 4 + 1；jsn 接在环里最大的 4 之后；点名实例表与分配记录树五个节点（按位置寻址，D8（核心索引结构） 已定项 14）"
    );
    assert_ne!(
        row.root.instance_table.locations[0].slot, instance_table_written_by_the_row_publish,
        "影子账把第二次挂载写出去的那片实例表隔离了：回退这次没发在它上面"
    );
    let reopened = formatted.memory_pool();
    let system_configuration = choose_system_configuration(&reopened).expect("择系统配置");
    let newest = choose_root(&reopened, &system_configuration).expect("择根");
    assert_eq!(newest.instance, InstanceGeneration(3));
    let table = singlefs_core::recovery::instance_table_of_root(&reopened, &newest)
        .expect("盘上那片实例表解得开");
    assert_eq!(
        table.rows, rolled_back.output.rows_written,
        "盘上写着这两行"
    );
    assert_eq!(
        recover(&reopened, JournalPolicy::Consult).outcome,
        RecoveryOutcome::NoFile {
            root: (newest.instance, newest.checkpoint_txg)
        },
        "回退到的那一版树表 0 条：冷启动读不到文件"
    );
    let violated: Vec<&str> = check_pool_image(&reopened)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| invariant)
        .collect();
    assert!(
        violated.is_empty(),
        "池级 checker 一条违例都没有：{violated:?}"
    );
}

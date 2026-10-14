//! harness 测试文件 `crates/singlefs-harness/tests/writable_mount_takes_a_second_instance.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNodePosition;
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::journal::back_chain_of;
use singlefs_core::mount::{mount_writable, InstanceRow, InstanceTableRecords, Mounted};
use singlefs_core::records::{
    STATISTIC_ALLOCATED_BYTES, STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_FREE_BYTES,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::target_for_publish;
use singlefs_core::transaction::{
    publish_overwrite, publish_version, FirstFile, InstanceTablePlan, InstanceTableRewrite,
    PoolWriter, PublishPlan, TransactionOutput, TransactionUnit,
};
use singlefs_format::SLOT_BYTES;

/// 分配记录树根之下 (层级, 盘, 同盘同层序号) 那个节点的角色（D8（核心索引结构） 已定项 14：按绝对槽号按位置寻址）。
pub fn allocation_record_tree_node(
    level: u8,
    device: u32,
    index_in_device: u64,
) -> TransactionUnit {
    TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
        level,
        device: DeviceIdentity(device),
        index_in_device,
    })
}

pub const SECOND_FILE_BYTES: usize = 4100;

pub const THIRD_FILE_BYTES: usize = 2500;

pub fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

pub fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

pub fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

/// 交给写者的上一版的回退下界 F：照旧，或抬到给定值（只改上一版的 F，别的照旧）。抬过的那一版记录新根段里的 F 与所选根的不同，
/// 施加出来的根取的是哪一份才分得出来（D23（journal 的角色与格式） 已定项 15「施加一条记录 = 把所选根的这四个字段换成记录里的」）。
pub enum RollbackFloorHandedToTheWriter {
    AsInThePreviousVersion,
    RaisedTo(CheckpointTxg),
}

/// 同一个进程、同一个实例里再发布一版（发布 B 用；步 3 的「预置一条实例 1 的记录」也用它造）。
pub fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
    rollback_floor: RollbackFloorHandedToTheWriter,
) -> TransactionOutput {
    let mut previous = pool.output.clone();
    match rollback_floor {
        RollbackFloorHandedToTheWriter::AsInThePreviousVersion => {}
        RollbackFloorHandedToTheWriter::RaisedTo(raised_floor) => {
            previous.root.rollback_floor = raised_floor;
        }
    }
    crate::common::publish_overwrite_in_process(
        pool,
        &previous,
        content,
        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
        instance,
    )
    .expect("覆盖写")
}

/// A → B 之后进程退出、重开并可写挂载。
pub fn build_publish_second_version_and_remount(
    tag: &str,
) -> (BuiltPool, TransactionOutput, Mounted) {
    let mut pool = build_pool(tag);
    let second = overwrite_in_process(
        &mut pool,
        &second_content(),
        InstanceGeneration(1),
        RollbackFloorHandedToTheWriter::AsInThePreviousVersion,
    );
    pool.output = second.clone();
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    (pool, second, mounted)
}

pub fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(
        CheckpointTxg(txg),
        parameters().geometry.root_ring_slots_per_region,
    );
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

pub fn accounting_value(output: &TransactionOutput, statistic: u16, device: DeviceIdentity) -> u64 {
    output
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == statistic && entry.device == device)
        .map(|entry| entry.value)
        .expect("带设备维的行每盘一行")
}

/// 验收第一、二条：取号 2；行 (1, 4, 0)；写行发布 txg 5 重写实例表 + 固定点 8 个单元（分配记录树 5 个节点 + 记账 + 映射 + 树表）、事务号 0、反向链 0；暖机两次落到两块盘；
/// 发布 C 接在后面（txg 8、事务号 1、反向链接 txg 7 那条）；冷启动择 (2, 8) 读回第三次的内容；记账与分配器对得上；checker 全绿。
pub fn remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, second, mut mounted) =
        build_publish_second_version_and_remount("step-three-remount");
    let output = &mounted.output;
    assert_eq!(
        output.instance,
        InstanceGeneration(2),
        "取号 = max(系统配置 1, 根环 1) + 1"
    );
    assert_eq!(
        (
            output.chosen_root.instance,
            output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(4)),
        "B 的根持久了：所选根就是它"
    );
    assert_eq!(output.effective_root, output.chosen_root, "没有记录要施加");
    assert_eq!(output.journal.valid_records, 4);
    assert_eq!(output.journal.prefix_applied, 0);
    assert_eq!(
        output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(4),
            applied_transaction_high_water: 0,
        }],
        "实例 1 那一行 (1, 4, 0)；实例 0 不写行"
    );

    let row = output
        .row_publish
        .file_version()
        .expect("B 带文件：写行发布是带文件的一版");
    assert_eq!(
        (row.root.instance, row.root.checkpoint_txg),
        (InstanceGeneration(2), CheckpointTxg(5))
    );
    assert_eq!(row.record.counter, 5, "jsn 全池接着走");
    assert_eq!(row.record.transaction, 0, "空发布的事务号 0");
    assert_eq!(row.record.back_chain, 0, "本实例的第一条反向链恒 0");
    // 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，这几次发布改的记录都在两块盘各自的叶 61 里，
    // 每次重写两片叶、两个第 1 层节点与根，先叶后根。
    let allocation_record_tree_nodes_below_the_root = [
        allocation_record_tree_node(0, 0, 61),
        allocation_record_tree_node(0, 1, 61),
        allocation_record_tree_node(1, 0, 0),
        allocation_record_tree_node(1, 1, 0),
    ];
    assert_eq!(
        row.rewritten,
        std::iter::once(TransactionUnit::InstanceTable)
            .chain(allocation_record_tree_nodes_below_the_root)
            .chain([
                TransactionUnit::AllocationTree,
                TransactionUnit::AccountingTree,
                TransactionUnit::MappingTree,
                TransactionUnit::TreeTable,
            ])
            .collect::<Vec<_>>(),
        "写行发布重写实例表 + 固定点 8 个单元（记账树已存在 ⇒ 空发布也重写，D16 已定项 9）：分配记录树根之下那四个节点、四个固定点角色的根（分配记录树、记账、映射、树表）"
    );
    assert_eq!(row.record.named.len(), 9, "点名的只有重写的九个");
    assert_eq!(
        row.data_pointers[0], second.data_pointers[0],
        "文件角色照抄 B 的"
    );
    assert_eq!(row.inode_record, second.inode_record);
    for identity in [
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
        TransactionUnit::InodeRoot,
    ] {
        assert_eq!(
            row.unit(identity).slot,
            second.unit(identity).slot,
            "{identity:?} 照抄、不换落点"
        );
    }
    assert_eq!(
        row.tree_root_pointer(singlefs_format::TREE_IDENTIFIER_EXTENT),
        second.tree_root_pointer(singlefs_format::TREE_IDENTIFIER_EXTENT),
        "树表里 extent 根指针照抄 B 的"
    );
    let table = InstanceTableRecords::parse(&row.unit(TransactionUnit::InstanceTable).bytes)
        .expect("写出的实例表单元解得开");
    assert_eq!(table.rows, output.rows_written);
    assert_eq!(
        row.released.len(),
        9,
        "写行发布释放 mkfs 的实例表单元 + B 的固定点 8 个单元（四个固定点角色的根与分配记录树根之下那四个节点）"
    );

    assert_eq!(
        output
            .warm_up_publishes
            .iter()
            .map(|publish| publish.root().checkpoint_txg.0)
            .collect::<Vec<_>>(),
        [6, 7],
        "实例 2 从 txg 5 起要两次：txg 6 落盘 0 白费、txg 7 落盘 1"
    );
    assert_eq!(region_device(5), DeviceIdentity(0));
    assert_eq!(region_device(6), DeviceIdentity(0));
    assert_eq!(region_device(7), DeviceIdentity(1));
    for publish in output
        .warm_up_publishes
        .iter()
        .map(|publish| publish.file_version().expect("带文件的一版上的暖机"))
    {
        assert_eq!(
            publish.rewritten,
            allocation_record_tree_nodes_below_the_root
                .into_iter()
                .chain([
                    TransactionUnit::AllocationTree,
                    TransactionUnit::AccountingTree,
                    TransactionUnit::MappingTree,
                    TransactionUnit::TreeTable,
                ])
                .collect::<Vec<_>>()
        );
        assert_eq!(publish.record.transaction, 0);
        assert_eq!(
            publish.released.len(),
            8,
            "每次暖机释放上一次的固定点 8 个单元（四个固定点角色的根与分配记录树根之下那四个节点）"
        );
    }
    let current = mounted
        .current
        .file_version()
        .expect("B 之后重开，现行那一版带文件")
        .clone();
    assert_eq!(
        current.root.checkpoint_txg,
        CheckpointTxg(7),
        "接下来的发布接在 txg 7 后面"
    );

    // 发布 C：实例 2 的第一次文件发布。
    let third = third_content();
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let third_publish = publish_overwrite(
        &mut writer,
        &mut mounted.allocator,
        &current,
        FirstFile {
            content: &third,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
        },
        InstanceGeneration(2),
    )
    .expect("发布 C");
    assert_eq!(
        (
            third_publish.root.instance,
            third_publish.root.checkpoint_txg
        ),
        (InstanceGeneration(2), CheckpointTxg(8))
    );
    assert_eq!(third_publish.record.counter, 8);
    assert_eq!(
        third_publish.record.transaction, 1,
        "事务号按实例计数从 1 起"
    );
    assert_eq!(
        third_publish.record.back_chain,
        back_chain_of(&current.record_bytes),
        "反向链接 txg 7 那条"
    );
    assert_eq!(
        third_publish.rewritten,
        TransactionUnit::IN_BUMP_ORDER[..4]
            .iter()
            .copied()
            .chain(allocation_record_tree_nodes_below_the_root)
            .chain(TransactionUnit::IN_BUMP_ORDER[4..].iter().copied())
            .collect::<Vec<_>>(),
        "七个角色按 bump 次序，分配记录树根之下那四个节点排在它的根之前（先叶后根）"
    );
    assert_eq!(
        third_publish.released.len(),
        12,
        "释放 B 的四个文件单元 + txg 7 的固定点 8 个单元（四个固定点角色的根与分配记录树根之下那四个节点）"
    );
    assert_eq!(
        third_publish
            .inode_record
            .expect("第三次发布那一版有第一个文件那条记录")
            .object_birth,
        CheckpointTxg(3),
        "对象出生代照旧"
    );
    let device_map = &mounted.allocator.devices[0];
    // 每版的固定点单元是四个角色加分配记录树根之下那四个节点，共 8 槽；带文件的一版再加文件四个单元 6 槽，共 14 槽。
    // 占着：mkfs 3 + A 14 + B 14 + txg5（实例表 2 + 8）+ txg6 8 + txg7 8 + C 14 = 71；
    // defer：mkfs 树表 1 + A 14 + B 14 + mkfs 实例表 2 + txg5 的 8 + txg6 的 8 + txg7 的 8 = 55。
    assert_eq!(device_map.allocated_slots(), 71);
    assert_eq!(device_map.deferred_slots(), 55);
    assert_eq!(device_map.free_slots(), 211_968 - 71);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_ALLOCATED_BYTES, device),
            71 * SLOT_BYTES
        );
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_DEFER_QUEUE_BYTES, device),
            55 * SLOT_BYTES
        );
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_FREE_BYTES, device),
            (211_968 - 71) * SLOT_BYTES
        );
    }
    assert_eq!(
        third_publish
            .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .slot,
        SlotNumber(50184),
        "C 的数据单元落在 B 之后的下一对偶数空槽（A、B 的都占着）"
    );

    // 冷启动：择 (2, 8)，读回第三次的内容；八条记录都在环里、没有一条高于水位。
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(8)),
            content: third.clone()
        }
    );
    assert_eq!(report.journal.valid_records, 8);
    assert_eq!(report.journal.above_water, 0);
    assert_eq!(report.journal.prefix_applied, 0);
    assert_eq!(
        recover(&reopened, JournalPolicy::Ignore).outcome,
        report.outcome,
        "根槽已持久：看不看 journal 一样"
    );

    // 池级 checker：一条不红，I-3.1、I-3.8、I-5.2、I-7.2、I-7.7 真被评估过且成立。
    images.hand_out(
        "重开取号 2、写行、暖机两块盘、发第三版之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::no_invariant_violated()
            .and_holds("I-3.1")
            .and_holds("I-3.8")
            .and_holds("I-5.2")
            .and_holds("I-7.2")
            .and_holds("I-7.7"),
    );
}

/// I-3.8（实例表行唯一且低于挂载根）的坏镜像：拿真写者再发一版，实例表里多写一行 (2, 7, 0)——行的实例代号等于挂载根的实例，
/// 别的都合法；池级 checker 只有 I-3.8 判红。
pub fn checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root(
    images: &mut PoolCheckerImages,
) {
    let (mut pool, _second, mut mounted) =
        build_publish_second_version_and_remount("step-three-bad-row");
    let current = mounted
        .current
        .file_version()
        .expect("B 之后重开，现行那一版带文件")
        .clone();
    let table = InstanceTableRecords::parse(&current.unit(TransactionUnit::InstanceTable).bytes)
        .expect("实例表解得开");
    let mut bad_table = table.clone();
    bad_table.rows.push(InstanceRow {
        instance: InstanceGeneration(2),
        selected_root_txg: CheckpointTxg(7),
        applied_transaction_high_water: 0,
    });
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_version(
        &mut writer,
        &mut mounted.allocator,
        PublishPlan {
            txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
            counter: current.record.counter + 1,
            transaction: 0,
            // 新实例（2）自己的第一条记录，事务号按实例各算各的。
            highest_transaction_number_before_this_publish: 0,
            instance: InstanceGeneration(2),
            back_chain: back_chain_of(&current.record_bytes),
            file: None,
            // 写行那次发布不碰 inode 树。
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Rewrite(InstanceTableRewrite {
                rows: bad_table.rows,
                // 这一版的表只有一片：被换下的就是根记录指着的那一片。
                replaced_chain: vec![current.root.instance_table],
            }),
            tree_birth_txg: current.tree_birth_txg(),
            tree_identifier_watermark: current.root.tree_identifier_watermark,
            rollback_floor: current.root.rollback_floor,
        },
        Some(&current),
    )
    .expect("写者不拦这一行：拦它的是 checker");
    images.hand_out(
        "写行写进一条实例不低于挂载根的行之后",
        || pool.memory_pool(),
        PoolCheckerExpectation::exactly_these_violated(&["I-3.8"]),
    );
}

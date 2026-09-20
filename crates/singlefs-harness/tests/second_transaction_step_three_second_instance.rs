//! 里程碑「第二个事务」步 3 的验收：发布 B 之后进程退出、重开两个镜像走可写挂载——恢复、从盘上重建上一版与分配器、
//! 取实例代号 2、给实例 1 写行 (1, 4, 0)、写行发布 txg 5、暖机两次（txg 6 落盘 0 白费、txg 7 落盘 1）——再发布 C；
//! 冷启动择实例 2 的根读回第三次的内容；实例表读回那一行；池级 checker 全绿（含 I-3.8）。

mod common;

use common::{build_pool, parameters, BuiltPool, Recorded, FIXED_WRITE_TIME_SECONDS};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::journal::back_chain_of;
use singlefs_core::journal::record_offset;
use singlefs_core::mount::{mount_writable, InstanceRow, InstanceTableRecords, Mounted};
use singlefs_core::records::{
    STATISTIC_ALLOCATED_BYTES, STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_FREE_BYTES,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    publish_overwrite, publish_version, FirstFile, InstanceTablePlan, PoolWriter, PublishPlan,
    TransactionOutput, TransactionUnit,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_format::SLOT_BYTES;

const SECOND_FILE_BYTES: usize = 4100;
const THIRD_FILE_BYTES: usize = 2500;

fn content_of(length: usize, seed: usize) -> Vec<u8> {
    (0..length)
        .map(|index| u8::try_from((index * 7 + seed) % 253).expect("小于 256"))
        .collect()
}

fn second_content() -> Vec<u8> {
    content_of(SECOND_FILE_BYTES, 3)
}

fn third_content() -> Vec<u8> {
    content_of(THIRD_FILE_BYTES, 11)
}

/// 同一个进程、同一个实例里再发布一版（发布 B 用；步 3 的「预置一条实例 1 的记录」也用它造）。
fn overwrite_in_process(
    pool: &mut BuiltPool,
    content: &[u8],
    instance: InstanceGeneration,
) -> TransactionOutput {
    let previous = pool.output.clone();
    common::publish_overwrite_in_process(
        pool,
        &previous,
        content,
        FIXED_WRITE_TIME_SECONDS + 60,
        instance,
    )
    .expect("覆盖写")
}

/// A → B 之后进程退出、重开并可写挂载。
fn build_publish_second_version_and_remount(tag: &str) -> (BuiltPool, TransactionOutput, Mounted) {
    let mut pool = build_pool(tag);
    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    pool.output = second.clone();
    let mut devices = pool.reopen_recorded();
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    pool.devices = Some(devices);
    (pool, second, mounted)
}

fn region_device(txg: u64) -> DeviceIdentity {
    let target = target_for_publish(CheckpointTxg(txg));
    parameters().region_devices[usize::try_from(target.region).expect("区域号")]
}

fn accounting_value(output: &TransactionOutput, statistic: u16, device: DeviceIdentity) -> u64 {
    output
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == statistic && entry.device == device)
        .map(|entry| entry.value)
        .expect("带设备维的行每盘一行")
}

/// 验收第一、二条：取号 2；行 (1, 4, 0)；写行发布 txg 5 重写实例表 + 四个固定点单元、事务号 0、反向链 0；暖机两次落到两块盘；
/// 发布 C 接在后面（txg 8、事务号 1、反向链接 txg 7 那条）；冷启动择 (2, 8) 读回第三次的内容；记账与分配器对得上；checker 全绿。
#[test]
fn remount_takes_instance_two_writes_the_row_warms_up_both_devices_and_publishes_the_third_version()
{
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
            is_rollback: false,
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
    assert_eq!(
        row.rewritten,
        [
            TransactionUnit::InstanceTable,
            TransactionUnit::AllocationTree,
            TransactionUnit::AccountingTree,
            TransactionUnit::MappingTree,
            TransactionUnit::TreeTable,
        ],
        "写行发布重写实例表 + 四个固定点单元（记账树已存在 ⇒ 空发布也重写，D16 已定项 9）"
    );
    assert_eq!(row.record.named.len(), 5, "点名的只有重写的五个");
    assert_eq!(row.data_pointer, second.data_pointer, "文件角色照抄 B 的");
    assert_eq!(row.inode_record, second.inode_record);
    for identity in [
        TransactionUnit::Data,
        TransactionUnit::ExtentRoot,
        TransactionUnit::InodeLeaf,
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
        5,
        "写行发布释放 mkfs 的实例表单元 + B 的四个固定点单元"
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
            [
                TransactionUnit::AllocationTree,
                TransactionUnit::AccountingTree,
                TransactionUnit::MappingTree,
                TransactionUnit::TreeTable,
            ]
        );
        assert_eq!(publish.record.transaction, 0);
        assert_eq!(
            publish.released.len(),
            4,
            "每次暖机释放上一次的四个固定点单元"
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
    assert_eq!(third_publish.rewritten, TransactionUnit::IN_BUMP_ORDER);
    assert_eq!(
        third_publish.released.len(),
        8,
        "释放 B 的四个文件单元 + txg 7 的四个固定点单元"
    );
    assert_eq!(
        third_publish.inode_record.object_birth,
        CheckpointTxg(3),
        "对象出生代照旧"
    );
    let device_map = &mounted.allocator.devices[0];
    // 占着：mkfs 3 + A 10 + B 10 + txg5（实例表 2 + 4）+ txg6 4 + txg7 4 + C 10 = 47；
    // defer：mkfs 树表 1 + A 10 + B 10 + mkfs 实例表 2 + txg5 的 4 + txg6 的 4 + txg7 的 4 = 35。
    assert_eq!(device_map.allocated_slots(), 47);
    assert_eq!(device_map.deferred_slots(), 35);
    assert_eq!(device_map.free_slots(), 211_968 - 47);
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_ALLOCATED_BYTES, device),
            47 * SLOT_BYTES
        );
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_DEFER_QUEUE_BYTES, device),
            35 * SLOT_BYTES
        );
        assert_eq!(
            accounting_value(&third_publish, STATISTIC_FREE_BYTES, device),
            (211_968 - 47) * SLOT_BYTES
        );
    }
    assert_eq!(
        third_publish.unit(TransactionUnit::Data).slot,
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

    // 池级 checker：全部判 Holds，I-3.8 真被评估过。
    let image = pool.memory_pool();
    let verdicts = check_pool_image(&image);
    let violated: Vec<_> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .collect();
    assert!(violated.is_empty(), "checker 判红：{violated:?}");
    for invariant in ["I-3.1", "I-3.8", "I-5.2", "I-7.2", "I-7.7"] {
        assert!(
            verdicts
                .iter()
                .any(|(name, verdict)| *name == invariant && *verdict == InvariantVerdict::Holds),
            "{invariant} 要真被评估过且成立"
        );
    }
}

/// 验收第二条的反面（D16 已定项 8 买的东西）：把实例 2 在盘 0 上的两个根槽（txg 5、6）都改坏，冷启动仍择到盘 1 上 txg 7 那个根、
/// 内容还是第二次的；暖机只推一次的话盘 1 上没有实例 2 的根，恢复退回实例 1。
#[test]
fn damaging_every_instance_two_root_on_one_device_still_leaves_a_root_on_the_other_device() {
    let (pool, _second, mounted) =
        build_publish_second_version_and_remount("step-three-one-device-damaged");
    let mut image = pool.memory_pool();
    for txg in [5u64, 6] {
        let target = target_for_publish(CheckpointTxg(txg));
        image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
    }
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(2), CheckpointTxg(7))),
        "盘 0 上实例 2 的根都坏了，盘 1 上 txg 7 那个还在"
    );
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), CheckpointTxg(7)),
            content: second_content()
        }
    );
    assert_eq!(mounted.output.warm_up_publishes.len(), 2);
}

/// 验收第三条：预置一条实例 1 的记录（jsn 5、txg 5、事务号 3，点名的单元也在盘上），把它的根槽改坏 ⇒ 重开那一刻所选根是 B 的根、
/// 同一个实例、没有回退行 ⇒ 恢复必须施加它，行写 (1, 5, 3)，写行发布的 txg 从 6 起。
#[test]
fn stray_record_of_the_previous_instance_is_applied_on_remount_and_its_transaction_lands_in_the_row(
) {
    let mut pool = build_pool("step-three-stray-record");
    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    pool.output = second;
    let stray = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
    assert_eq!(
        (
            stray.root.checkpoint_txg,
            stray.record.counter,
            stray.record.transaction
        ),
        (CheckpointTxg(5), 5, 3)
    );
    let mut devices: Vec<(DeviceIdentity, Recorded)> = pool.reopen_recorded();
    // 把 txg 5 的根槽改坏：只留记录与单元，模拟「根槽没持久」。
    let target = target_for_publish(CheckpointTxg(5));
    let device = region_device(5);
    let offset = slot_offset(target, 4096);
    let (_, damaged_device) = devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("区域归属的盘在池里");
    let mut garbage = vec![0u8; 4096];
    garbage[0] = 0xff;
    damaged_device
        .write_at(offset, &garbage, WriteDurability::Plain)
        .expect("改坏根槽");
    let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
    assert_eq!(
        (
            mounted.output.chosen_root.instance,
            mounted.output.chosen_root.checkpoint_txg
        ),
        (InstanceGeneration(1), CheckpointTxg(4))
    );
    assert_eq!(mounted.output.journal.prefix_applied, 1, "jsn 5 被施加");
    assert_eq!(mounted.output.journal.maximum_applied_transaction, 3);
    assert_eq!(
        mounted.output.effective_root.checkpoint_txg,
        CheckpointTxg(5),
        "施加之后的根是第 5 代"
    );
    assert_eq!(
        mounted.output.rows_written,
        vec![InstanceRow {
            instance: InstanceGeneration(1),
            selected_root_txg: CheckpointTxg(5),
            applied_transaction_high_water: 3,
            is_rollback: false,
        }]
    );
    assert_eq!(
        mounted.output.row_publish.root().checkpoint_txg,
        CheckpointTxg(6),
        "新实例的第一次发布 = max(根环 4, 记录 5) + 1"
    );
    assert_eq!(mounted.output.row_publish.record().counter, 6);
    pool.devices = Some(devices);
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(2), mounted.current.root().checkpoint_txg),
            content: third_content()
        },
        "施加了那条记录之后照抄的文件角色就是第三次的内容"
    );
}

/// I-3.8（实例表行唯一且低于挂载根）的坏镜像：拿真写者再发一版，实例表里多写一行 (2, 7, 0)——行的实例代号等于挂载根的实例，
/// 别的都合法；池级 checker 只有 I-3.8 判红。
#[test]
fn checker_rejects_an_instance_table_row_whose_instance_is_not_below_the_mount_root() {
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
        is_rollback: false,
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
            instance: InstanceGeneration(2),
            back_chain: back_chain_of(&current.record_bytes),
            file: None,
            instance_table: InstanceTablePlan::Rewrite(bad_table.to_records()),
            tree_birth_txg: current.tree_birth_txg(),
            tree_identifier_watermark: current.root.tree_identifier_watermark,
            rollback_floor: current.root.rollback_floor,
        },
        Some(&current),
    )
    .expect("写者不拦这一行：拦它的是 checker");
    let verdicts = check_pool_image(&pool.memory_pool());
    let violated: Vec<&str> = verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(violated, ["I-3.8"], "只有 I-3.8 红：{verdicts:?}");
}

/// 前缀链首接在所选根自己那条记录之后（D23 已定项 14 第 1 条「链从所选根覆盖的最后一条记录之后接」）：同一实例里再发两版（txg 5、6），
/// 把两个根槽改坏、再把 jsn 5 的记录两份都改坏——所选根 (1, 4) 自己那条 jsn 4 读得出，链首该是 jsn 5，jsn 5 读不出 ⇒ 断号即止，
/// jsn 6 一条都不施加，读回第二次的内容；从「水位之上最小的那条」接的写法会施加 jsn 6、读出第四版。
#[test]
fn one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact(
) {
    let mut pool = build_pool("step-three-chain-start");
    let second = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
    pool.output = second;
    let third = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
    pool.output = third;
    let fourth = overwrite_in_process(&mut pool, &content_of(3300, 17), InstanceGeneration(1));
    assert_eq!(
        (fourth.root.checkpoint_txg, fourth.record.counter),
        (CheckpointTxg(6), 6)
    );
    let mut image = pool.memory_pool();
    for txg in [5u64, 6] {
        let target = target_for_publish(CheckpointTxg(txg));
        image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
    }
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        image.flip_byte(device, record_offset(5, JOURNAL_RING_DEFAULT_BYTES), 300);
    }
    let report = recover(&image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: second_content()
        },
        "jsn 5 读不出、jsn 6 不许接上：{:?}",
        report.journal
    );
    assert_eq!(report.journal.valid_records, 5, "jsn 1–4、6");
    assert_eq!(report.journal.prefix_applied, 0);
}

/// 三方第一轮攻方腿打中的一格：所选根自己那条记录读不出时链首没有锚点，不许把水位之上最小的那条无条件接上——同一实例再发三版
/// （txg 4、5、6），改坏 txg 5、6 的根槽让所选根退到 (1, 4)，再改坏 jsn 4 与 jsn 5 两份镜像：jsn 6 的 txg 是 6、不是 4 + 1 ⇒ 一条都不施加、
/// 读回第二版；只改坏 jsn 4 时 jsn 5 的 txg 正是 5 ⇒ 接上、再顺着 jsn 6 施加到第四版。写行随之：前一种写 (1, 4, 0)，后一种 (1, 6, 4)。
#[test]
fn torn_anchor_record_lets_the_chain_start_only_at_the_next_checkpoint_txg() {
    for (
        tear_jsn_five,
        expected_effective_root,
        expected_content,
        expected_applied,
        expected_row,
    ) in [
        (
            true,
            CheckpointTxg(4),
            second_content(),
            0,
            (CheckpointTxg(4), 0),
        ),
        (
            false,
            CheckpointTxg(6),
            content_of(3300, 17),
            2,
            (CheckpointTxg(6), 4),
        ),
    ] {
        let mut pool = build_pool("step-three-torn-anchor");
        pool.output = overwrite_in_process(&mut pool, &second_content(), InstanceGeneration(1));
        pool.output = overwrite_in_process(&mut pool, &third_content(), InstanceGeneration(1));
        let fourth = overwrite_in_process(&mut pool, &content_of(3300, 17), InstanceGeneration(1));
        assert_eq!(fourth.record.counter, 6);
        let mut image = pool.memory_pool();
        for txg in [5u64, 6] {
            let target = target_for_publish(CheckpointTxg(txg));
            image.flip_byte(region_device(txg), slot_offset(target, 4096), 100);
        }
        let torn: Vec<u64> = if tear_jsn_five { vec![4, 5] } else { vec![4] };
        for jsn in &torn {
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                image.flip_byte(device, record_offset(*jsn, JOURNAL_RING_DEFAULT_BYTES), 300);
            }
        }
        let report = recover(&image, JournalPolicy::Consult);
        assert_eq!(
            (report.outcome, report.effective_root),
            (
                RecoveryOutcome::FileRead {
                    root: (InstanceGeneration(1), CheckpointTxg(4)),
                    content: expected_content
                },
                Some((InstanceGeneration(1), expected_effective_root))
            ),
            "撕掉 {torn:?}（所选根 (1, 4)）：{:?}",
            report.journal
        );
        assert_eq!(report.journal.prefix_applied, expected_applied);
        // 同一段历史做可写挂载：写出的行 W 只罩住真被施加的前缀。
        let mut devices = pool.reopen_recorded();
        for txg in [5u64, 6] {
            let target = target_for_publish(CheckpointTxg(txg));
            let device = region_device(txg);
            let offset = slot_offset(target, 4096);
            let (_, recorded) = devices
                .iter_mut()
                .find(|(identity, _)| *identity == device)
                .expect("那块盘");
            let mut bytes = vec![0u8; 4096];
            recorded.read_at(offset, &mut bytes).expect("读根槽");
            bytes[100] ^= 0xff;
            recorded
                .write_at(offset, &bytes, WriteDurability::Plain)
                .expect("改坏根槽");
        }
        for jsn in &torn {
            for (_, recorded) in devices.iter_mut() {
                let offset = record_offset(*jsn, JOURNAL_RING_DEFAULT_BYTES);
                let mut bytes = vec![0u8; 4096];
                recorded.read_at(offset, &mut bytes).expect("读记录");
                bytes[300] ^= 0xff;
                recorded
                    .write_at(offset, &bytes, WriteDurability::Plain)
                    .expect("改坏记录");
            }
        }
        let mounted = mount_writable(&parameters(), &mut devices).expect("可写挂载");
        pool.devices = Some(devices);
        assert_eq!(
            mounted
                .output
                .rows_written
                .iter()
                .map(|row| (row.selected_root_txg, row.applied_transaction_high_water))
                .collect::<Vec<_>>(),
            vec![expected_row],
            "撕掉 {torn:?} 之后写的行"
        );
    }
}

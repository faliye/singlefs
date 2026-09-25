//! 里程碑「第二个事务」增补 2 收口表第 62 行（C533（记录新根段缺实例表与分配记录树两个根指针））：复现，只坐实、不修。
//!
//! 历史：只做过 mkfs 的池 → 可写挂载（取号 1、不写行、零单元发布 txg 1、暖机 txg 2）→ 进程退出 → 再可写挂载：取号 2，
//! 写行那次发布在树表 0 条的一版上另写实例表与一棵分配记录树（D16（发布语义） 已定项 9；按位置寻址，D8（核心索引结构） 已定项 14：
//! 4 GiB 两块盘上五个节点），txg 3、jsn 3——崩在这次发布的记录落盘之后、根槽落盘之前 → 重开。
//! 下文说的「那两个单元」指实例表与分配记录树（后者是五个节点），一共六个单元。
//!
//! 这条用例钉住今天的样子，逐条对着收口表第 62 行要问的几样：
//! - 那两个单元落在哪：写行那次发布取的两个落点，崩溃镜像上两块盘都有、校验和对得上；记录 jsn 3 两份也在。
//! - 恢复重建的根拿不拿得到它们：拿不到，而且那条记录**根本不施加**——它是实例 2 的第一条，所选根是实例 1 的 (1, 2)，
//!   前缀规则不跨实例边界（D23（journal 的角色与格式） 已定项 14 注 1），水位之上一条都没有。重建出来的根就是 (1, 2) 本身：
//!   实例表指针是 mkfs 那一片、分配记录树根指针全零。写行那次发布恒是新实例的第一次发布，所以「施加一条写行记录」今天走不到。
//! - checker 怎么算它们：遍历从根走，谁也不引用这两个单元，一条违例都没有；把它们改坏，判决逐条不变（checker 不读它们）。
//! - 分配器怎么算它们：重开之后的可写挂载从 mkfs 那一版的账重建，这两个落点不在任何一条分配记录里、是空闲槽；
//!   这次挂载写行那次发布取的落点与崩掉的那次逐槽相同，把它们盖掉。之后 checker 仍一条违例都没有。
//!
//! 条款怎么定（新根段加这两样 / 这种发布整次不施加 / 别的）交用户；这里只记今天的结局。

mod common;

use common::{
    crash_state_devices, format_pool, geometry, memory_pool_of_sparse_devices, parameters,
    IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::journal::{record_offset, JournalRecord};
use singlefs_core::mount::{mount_writable, InstanceRow};
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, recover, replay_journal, rollback_high_water_of_root,
    scan_journal, JournalPolicy, PoolReader, RecoveryOutcome,
};
use singlefs_core::transaction::PoolVersion;
use singlefs_core::unit::{unit_filesystem_identifier, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED};
use singlefs_format::{
    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES,
};
use singlefs_harness::crash::{writes_and_segments, MemoryPool};
use singlefs_harness::segments::StepKind;
use singlefs_harness::SharedStream;

const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

fn unit_bytes_of_class(unit_class: u8) -> usize {
    let bytes = match unit_class {
        UNIT_CLASS_PACKED => DATA_UNIT_BYTES,
        UNIT_CLASS_INDEX_NODE => NODE_BYTES,
        other => panic!("写行那次发布只点名实例表（码 3）与分配记录节点（码 2）：{other}"),
    };
    usize::try_from(bytes).expect("单元宽装得进 usize")
}

fn assert_no_invariant_is_violated(image: &MemoryPool, step: &str) {
    let violated: Vec<(&str, InvariantVerdict)> = check_pool_image(image)
        .into_iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .collect();
    assert!(violated.is_empty(), "{step}：checker 判红 {violated:?}");
}

#[test]
fn c533_row_publish_record_persisted_without_its_root_on_a_formatted_pool_is_never_applied_and_its_two_units_are_free_slots_that_the_next_mount_overwrites(
) {
    let mut formatted = format_pool("c533-row-publish-record-without-root");
    let mut first = formatted.reopen_recorded();
    let first_mount = mount_writable(&parameters(), &mut first).expect("第一次可写挂载：不写行");
    formatted.devices = Some(first);
    assert_eq!(first_mount.current.root().checkpoint_txg, CheckpointTxg(2));
    let operations_before_the_second_mount = formatted.stream.operation_count();
    let mut second = formatted.reopen_recorded();
    let row_written = mount_writable(&parameters(), &mut second).expect("第二次可写挂载：写行");
    formatted.devices = Some(second);
    let PoolVersion::WithoutFile(row_publish) = &row_written.output.row_publish else {
        panic!("树表 0 条的一版上写行，写出来的仍是没有文件版本的一版")
    };
    assert_eq!(
        (
            row_publish.root.instance,
            row_publish.root.checkpoint_txg,
            row_publish.record.counter
        ),
        (InstanceGeneration(2), CheckpointTxg(3), 3)
    );
    // 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，写行那次写出五个节点
    // （两块盘各自的叶 61、各自的第 1 层节点 0 与根），加实例表共六个单元。
    assert_eq!(
        row_publish.record.named.len(),
        6,
        "写行那次发布点名实例表与分配记录树五个节点，六个单元"
    );
    let unit_slots_of_the_row_publish = |named: &[singlefs_core::journal::NamedUnit]| {
        named
            .iter()
            .map(|named| {
                assert_eq!(named.locations[1].slot, named.locations[0].slot, "两盘同槽");
                named.locations[0].slot
            })
            .collect::<Vec<SlotNumber>>()
    };
    let slots_of_the_crashed_row_publish = unit_slots_of_the_row_publish(&row_publish.record.named);
    assert!(slots_of_the_crashed_row_publish
        .contains(&row_publish.root.instance_table.locations[0].slot));
    assert!(slots_of_the_crashed_row_publish
        .contains(&row_publish.root.allocation_record_tree_root.locations[0].slot));

    // 崩溃状态：第二次挂载写出去的写里，根槽写之前的全部持久（取号两写、六个单元各两写、记录两写），根槽与之后的一个都没有。
    let operations = formatted.retained_operations();
    let mut base = MemoryPool::with_devices(&BOTH_DEVICES, IMAGE_BYTES);
    base.apply(&operations[..operations_before_the_second_mount]);
    let (writes, _segments) = writes_and_segments(
        &operations[operations_before_the_second_mount..],
        &geometry(),
    );
    let first_root_write = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("写行那次发布有根槽写");
    let persisted_kinds: Vec<StepKind> = writes[..first_root_write]
        .iter()
        .map(|write| write.kind)
        .collect();
    assert_eq!(
        persisted_kinds,
        [StepKind::SystemConfigurationSlot; 2]
            .into_iter()
            .chain([StepKind::UnitWrite; 12])
            .chain([StepKind::JournalRecord; 2])
            .collect::<Vec<_>>(),
        "根槽写之前是取号两写、六个单元各两写、记录两写"
    );
    let persisted: Vec<bool> = (0..writes.len())
        .map(|write_index| write_index < first_root_write)
        .collect();
    let crash_stream = SharedStream::retaining_contents();
    let mut devices = crash_state_devices(&base, &writes, &persisted, &crash_stream);
    let crash_image = memory_pool_of_sparse_devices(&devices);

    // 那两个单元与那条记录都在盘上。
    for named in &row_publish.record.named {
        for location in &named.locations {
            let bytes = crash_image
                .read(
                    location.device,
                    location.slot.to_device_offset(),
                    unit_bytes_of_class(named.unit_class),
                )
                .expect("单元区读得到");
            assert_eq!(
                crc32_castagnoli(&bytes),
                location.unit_checksum,
                "写行那次发布的单元（码 {}）在盘 {} 槽 {} 上完整",
                named.unit_class,
                location.device.0,
                location.slot.0
            );
        }
    }
    let filesystem_identifier = unit_filesystem_identifier(&parameters().filesystem_identifier);
    for device in BOTH_DEVICES {
        let bytes = crash_image
            .read(
                device,
                record_offset(3, JOURNAL_RING_DEFAULT_BYTES),
                usize::try_from(JOURNAL_RECORD_BYTES).expect("4096"),
            )
            .expect("环里读得到");
        let record = JournalRecord::parse(&bytes, filesystem_identifier)
            .expect("写行那条记录两份都自证得过");
        assert_eq!(
            (record.instance, record.checkpoint_txg, record.counter),
            (InstanceGeneration(2), CheckpointTxg(3), 3)
        );
    }

    // 恢复：所选根 (1, 2)，那条记录是实例 2 的，不在前缀候选里；重建出来的根就是 (1, 2)，拿不到那两个单元。
    let report = recover(&crash_image, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::NoFile {
            root: (InstanceGeneration(1), CheckpointTxg(2))
        }
    );
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(2)))
    );
    assert_eq!(
        (
            report.journal.valid_records,
            report.journal.above_water,
            report.journal.prefix_applied
        ),
        (3, 0, 0),
        "jsn 1–3 都读得出；jsn 3 是实例 2 的，前缀不跨实例边界，水位之上一条都没有"
    );
    let system_configuration =
        choose_system_configuration(&crash_image).expect("崩溃镜像上系统配置择得出");
    let chosen_root = choose_root(&crash_image, &system_configuration).expect("择得出根");
    let records = scan_journal(&crash_image, &system_configuration);
    let (_, rebuilt_root) = replay_journal(
        &crash_image,
        &chosen_root,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records,
        true,
        rollback_high_water_of_root(&crash_image, &chosen_root),
    )
    .expect("所选根那次发布只有一条记录带末条标志：锚点认得出");
    assert_eq!(rebuilt_root, chosen_root, "重建出来的根就是所选根本身");
    assert_eq!(
        rebuilt_root.instance_table, formatted.genesis.root.instance_table,
        "重建出来的根指着 mkfs 那一片实例表，不是写行那次写的那一片"
    );
    assert_eq!(
        rebuilt_root.allocation_record_tree_root,
        NodePointer::empty_root(),
        "重建出来的根没有分配记录树，写行那次写的那片节点谁也不引用"
    );

    // checker：遍历从根走，那两个单元谁也不引用，一条违例都没有；把它们在两块盘上都改坏，每一条判决逐字不变——checker 不读它们。
    assert_no_invariant_is_violated(&crash_image, "崩溃镜像");
    let mut orphan_units_damaged = crash_image.clone();
    for named in &row_publish.record.named {
        for location in &named.locations {
            orphan_units_damaged.flip_byte(location.device, location.slot.to_device_offset(), 100);
        }
    }
    assert_eq!(
        check_pool_image(&orphan_units_damaged),
        check_pool_image(&crash_image),
        "两个孤单元两块盘上都改坏之后 checker 的判决逐条不变"
    );

    // 重开可写挂载：账从 mkfs 那一版重建，那两个落点是空闲槽；写行那次发布取到同样的两个落点、把它们盖掉。
    let third_mount = mount_writable(&parameters(), &mut devices).expect("崩溃之后重开可写挂载");
    assert_eq!(
        third_mount.output.instance,
        InstanceGeneration(3),
        "号 2 取号时已写进系统配置，烧掉了"
    );
    assert_eq!(
        third_mount.output.rows_written,
        vec![
            InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(2),
                applied_transaction_high_water: 0,
                is_rollback: false,
            },
            InstanceRow {
                instance: InstanceGeneration(2),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            },
        ],
        "实例 1 那一行 (1, 2, 0)；烧掉的实例 2 按中间实例写 (2, 0, 0)"
    );
    let PoolVersion::WithoutFile(rewritten_row_publish) = &third_mount.output.row_publish else {
        panic!("仍是树表 0 条的一版")
    };
    assert_eq!(
        (
            rewritten_row_publish.root.checkpoint_txg,
            rewritten_row_publish.record.counter
        ),
        (CheckpointTxg(4), 4),
        "txg 与 jsn 都从环里那条孤记录（txg 3、jsn 3）之后接"
    );
    assert_eq!(
        unit_slots_of_the_row_publish(&rewritten_row_publish.record.named),
        slots_of_the_crashed_row_publish,
        "分配器不认得崩掉那次写的六个落点：这次写行取到的正是它们，次序也一样"
    );
    let mut orphan_placements_in_the_rebuilt_records = third_mount
        .allocator
        .records()
        .iter()
        .filter(|record| slots_of_the_crashed_row_publish.contains(&record.slot))
        .map(|record| (record.device, record.slot, record.is_released))
        .collect::<Vec<_>>();
    orphan_placements_in_the_rebuilt_records.sort();
    let mut expected_records: Vec<(DeviceIdentity, SlotNumber, bool)> = BOTH_DEVICES
        .iter()
        .flat_map(|device| {
            slots_of_the_crashed_row_publish
                .iter()
                .map(move |slot| (*device, *slot, false))
        })
        .collect();
    expected_records.sort();
    assert_eq!(
        orphan_placements_in_the_rebuilt_records, expected_records,
        "挂载之后罩着这六个槽的只有这次写行新记的那几条（两盘各六条、都未释放）"
    );
    assert_no_invariant_is_violated(
        &memory_pool_of_sparse_devices(&devices),
        "崩溃之后重开可写挂载",
    );
}

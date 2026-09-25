//! 事务号 0 的发布切成多条记录（D23（journal 的角色与格式） 已定项 17「末条再跨记录」、已定项 7 提交标记、已定项 4 读者规则）。
//!
//! - 树表 0 条的一版上写行（`publish_instance_table_on_version_without_file`）点名项多于一条记录装得下时，接到带文件那一路同一个切法
//!   （`roles_named_by_each_record_of_the_publish`）：装满一条再开下一条，只有真正的最后一条带「本次发布末条」标志与提交标记，
//!   本次发布内序号依次 1..N（实十六接续报告 `research/prompts/m2-writepath-implementer-report.md` 第六节 Q7：片数 ≥ 67 时这一路原先只写一条、
//!   往 4096 字节的记录里写第 68 项时越界 panic）。产品路径上要实例表长到 67 片才走得到，这里用只供测试的开关
//!   （`PoolWriter::set_journal_record_named_entry_capacity`）把一条记录装的项数压到 1，两个单元就造得出跨记录的写行。
//! - 事务号 0 的发布跨多条记录时，前几条的提交标记写 0；读者读到提交标记 2..=255 当那条记录损坏、断链即止（主 agent 2026-09-24 定，
//!   与已定项 4 读者规则同一处置），池级 checker 的 I-8.8 判「提交标记字节」违例——两边说的是同一件事。

mod common;

use common::{build_pool, file_content, memory_pool_of_sparse_devices, parameters, IMAGE_BYTES};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::checksum::wide_checksum_with_field_zeroed;
use singlefs_core::journal::{
    back_chain_of, record_offset, JournalRecordOrdinalWithinPublish, JournalRecordPlaceInPublish,
    JOURNAL_HEADER_CHECKSUM_OFFSET,
};
use singlefs_core::make_filesystem::{
    make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT,
};
use singlefs_core::mount::{mount_writable, InstanceRow};
use singlefs_core::recovery::{
    recover, JournalPolicy, PoolReader, RecoveryOutcome, RecoveryReport,
};
use singlefs_core::transaction::{
    acquire_instance, publish_instance_table_on_version_without_file, publish_version,
    publish_without_units, InstanceTableOnlyPublishPlan, InstanceTablePlan, InstanceTableRewrite,
    JournalRecordNamedEntryCapacity, PoolWriter, PublishPlan, ZeroUnitPublishPlan,
};
use singlefs_format::{JOURNAL_RECORD_BYTES, JOURNAL_RING_DEFAULT_BYTES};
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

const DISKS: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

type Devices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 提交标记在记录头里的偏移（D23（journal 的角色与格式） 已定项 4 的字段表：事务号 78、提交标记 86）。
const COMMIT_MARKER_OFFSET: usize = 86;

fn sparse_devices(stream: &SharedStream) -> Devices {
    DISKS
        .iter()
        .map(|identity| {
            (
                *identity,
                RecordingBlockDevice::with_shared_stream(
                    *identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

/// 录制流里最后一次根槽 FUA 写之前的那些写施加到两块空内存盘上：崩在那次发布的根落盘之前。
fn image_before_the_last_root(stream: &SharedStream) -> MemoryPool {
    let operations = stream.retained_operations();
    let last_root_slot_write = operations
        .iter()
        .rposition(|retained| {
            retained.operation.kind == RecordedOperationKind::WriteForceUnitAccess
        })
        .expect("写过根槽");
    let mut image = MemoryPool::with_devices(&DISKS, IMAGE_BYTES);
    image.apply(&operations[..last_root_slot_write]);
    image
}

fn violated(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<&'static str> {
    verdicts
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(invariant, _)| *invariant)
        .collect()
}

fn effective_root_and_applied(
    report: &RecoveryReport,
) -> (Option<(InstanceGeneration, CheckpointTxg)>, usize) {
    (report.effective_root, report.journal.prefix_applied)
}

/// 写行那次发布点名几个单元：实例表那一片加分配记录树五个节点（按位置寻址，D8（核心索引结构） 已定项 14：4 GiB 两块盘上根在第 2 层，
/// 两块盘各自的叶 61、各自的第 1 层节点 0 与根）。开关压到一条一项，就是几条记录。
const ROW_PUBLISH_UNITS: u64 = 6;

/// 验收（D23（journal 的角色与格式） 已定项 17，实十六接续报告第六节 Q7）：树表 0 条的一版上写行，点名项（实例表那一片 + 分配记录树五个节点）
/// 多于一条记录装得下（开关压到一条一项）⇒ 末条再跨记录：六条记录，jsn 1–6，事务号都是 0，本次发布内序号 1–6，只有第六条带末条标志与提交标记，
/// 每条的反向链罩前一条的头；六条合起来点名的就是这次重写的六个单元。之后同一实例再发一次零单元发布（txg 2），崩在它的根落盘之前：
/// 恢复择写行那次的根（实例 2、txg 1），锚点按末条标志认在 jsn 6，施加 txg 2 那一条；池级 checker 在崩溃镜像与整条流上都一条不红；
/// 再可写挂载照常（从写行那次写下的分配记录树重建账）。
#[test]
fn a_row_publish_whose_named_units_do_not_fit_one_record_spills_over_and_recovery_and_the_checker_accept_it(
) {
    let stream = SharedStream::retaining_contents();
    let mut devices = sparse_devices(&stream);
    let publish_parameters = parameters();
    let genesis = make_filesystem(&publish_parameters, &mut devices).expect("mkfs");
    let mut allocator = PoolAllocator::new(
        DISKS
            .iter()
            .map(|device| DeviceFreeMap::new(*device, IMAGE_BYTES))
            .collect(),
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
    let (row_publish, following) = {
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        writer.set_journal_record_named_entry_capacity(
            JournalRecordNamedEntryCapacity::CappedForTests {
                named_entries_per_record: 1,
            },
        );
        // 取号 1 之后崩溃（写行那次没发出去），再取号 2：实例 2 的写行要给实例 1 补一行 (1, 0, 0)。
        acquire_instance(&mut writer).expect("取号 1");
        let instance = acquire_instance(&mut writer).expect("取号 2");
        assert_eq!(instance, InstanceGeneration(2));
        let rewrite = InstanceTableRewrite {
            rows: vec![InstanceRow {
                instance: InstanceGeneration(1),
                selected_root_txg: CheckpointTxg(0),
                applied_transaction_high_water: 0,
                is_rollback: false,
            }],
            replaced_chain: vec![genesis.root.instance_table],
        };
        let row_publish = publish_instance_table_on_version_without_file(
            &mut writer,
            &mut allocator,
            &genesis.root,
            InstanceTableOnlyPublishPlan {
                txg: CheckpointTxg(1),
                counter: 1,
                instance,
                back_chain: 0,
                rollback_floor: CheckpointTxg(0),
                instance_table: &rewrite,
                tree_identifier_watermark: genesis.root.tree_identifier_watermark,
            },
        )
        .expect("树表 0 条的一版上写行，点名项跨六条记录");
        let following = publish_without_units(
            &mut writer,
            &row_publish.root,
            ZeroUnitPublishPlan {
                txg: CheckpointTxg(2),
                counter: row_publish.record.counter + 1,
                instance,
                back_chain: back_chain_of(&row_publish.record_bytes),
                rollback_floor: CheckpointTxg(0),
                tree_identifier_watermark: row_publish.root.tree_identifier_watermark,
            },
        )
        .expect("接着一次零单元发布");
        (row_publish, following)
    };

    let records: Vec<_> = row_publish
        .earlier_records_of_this_publish
        .iter()
        .map(|written| (written.record.clone(), written.bytes.clone()))
        .chain([(row_publish.record.clone(), row_publish.record_bytes.clone())])
        .collect();
    assert_eq!(
        records
            .iter()
            .map(|(record, _)| (
                record.counter,
                record.transaction,
                record.ordinal_within_publish,
                record.is_commit,
                record.place_in_publish,
                record.named.len()
            ))
            .collect::<Vec<_>>(),
        (1..=ROW_PUBLISH_UNITS)
            .map(|ordinal| (
                ordinal,
                0,
                JournalRecordOrdinalWithinPublish(u32::try_from(ordinal).expect("六条")),
                ordinal == ROW_PUBLISH_UNITS,
                if ordinal == ROW_PUBLISH_UNITS {
                    JournalRecordPlaceInPublish::LastRecordOfThePublish
                } else {
                    JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
                },
                1
            ))
            .collect::<Vec<_>>(),
        "写行那次发布跨六条记录：jsn 1–6，事务号 0，序号 1–6，提交标记与末条标志只在第六条"
    );
    assert_eq!(
        records[0].0.back_chain, 0,
        "第一条是本实例第一条（反向链 0）"
    );
    for pair in records.windows(2) {
        assert_eq!(
            pair[1].0.back_chain,
            back_chain_of(&pair[0].1),
            "后一条的反向链罩前一条的头"
        );
    }
    let named_slots: Vec<_> = records
        .iter()
        .flat_map(|(record, _)| record.named.iter().map(|named| named.locations[0].slot))
        .collect();
    assert_eq!(
        (named_slots.len(), named_slots.first(), named_slots.last()),
        (
            6,
            Some(&row_publish.root.instance_table.locations[0].slot),
            Some(&row_publish.root.allocation_record_tree_root.locations[0].slot)
        ),
        "六条合起来点名的是这次重写的实例表那一片与分配记录树五个节点，按 bump 次序（实例表先，分配记录树先叶后根）"
    );

    let crash_image = image_before_the_last_root(&stream);
    let report = recover(&crash_image, JournalPolicy::Consult);
    assert_eq!(
        effective_root_and_applied(&report),
        (Some((InstanceGeneration(2), CheckpointTxg(2))), 1),
        "崩在 txg 2 的根之前：择写行那次的根，锚点认在带末条标志的 jsn 6，txg 2 那一条施加上去（{:?}）",
        report.outcome
    );
    assert!(
        matches!(report.outcome, RecoveryOutcome::NoFile { .. }),
        "这一版没有文件：{:?}",
        report.outcome
    );
    let crash_verdicts = check_pool_image(&crash_image);
    assert_eq!(
        violated(&crash_verdicts),
        Vec::<&str>::new(),
        "崩溃镜像上池级 checker 一条不红（I-8.9 的序号与末条标志、I-8.8 的提交标记、I-8.6 的反向链都认跨记录的写行）"
    );
    let whole_image = memory_pool_of_sparse_devices(&devices);
    assert_eq!(
        following.root.checkpoint_txg,
        CheckpointTxg(2),
        "零单元发布接在写行之后"
    );
    assert_eq!(
        violated(&check_pool_image(&whole_image)),
        Vec::<&str>::new(),
        "整条流写完的镜像上池级 checker 一条不红"
    );
    let remounted = mount_writable(&parameters(), &mut devices).expect("再可写挂载照常");
    assert_eq!(remounted.output.instance, InstanceGeneration(3));
}

/// 带文件的一版之后的一次空发布（事务号 0，重写分配记录树、记账树、中央映射树与树表），开关压到一条记录装两项：它跨几条记录，
/// 前几条的提交标记写 0、只有末条带；崩在它的根落盘之前，恢复把这几条整次施加（阳性对照）。
/// 把第一条的提交标记字节改成 2（重封头部校验和，两盘各一份）：读者当它损坏、断链即止，那次发布一条都不施加，读回 A；
/// 把 2 读成「不带」的读者（改之前）照样接上、整次施加。池级 checker 的 I-8.8 在同一份镜像上判「提交标记字节」违例。
#[test]
fn a_commit_marker_other_than_zero_or_one_counts_as_torn_and_breaks_the_chain_and_the_checker_reddens(
) {
    let mut pool = build_pool("multi-record-transaction-zero-commit-marker");
    let first = pool.output.clone();
    let empty_publish = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        writer.set_journal_record_named_entry_capacity(
            JournalRecordNamedEntryCapacity::CappedForTests {
                named_entries_per_record: 2,
            },
        );
        publish_version(
            &mut writer,
            &mut pool.allocator,
            PublishPlan {
                txg: CheckpointTxg(first.root.checkpoint_txg.0 + 1),
                counter: first.record.counter + 1,
                transaction: 0,
                highest_transaction_number_before_this_publish: first
                    .highest_transaction_number_in_this_instance,
                instance: InstanceGeneration(1),
                back_chain: back_chain_of(&first.record_bytes),
                file: None,
                new_inode_records: &[],
                instance_table: InstanceTablePlan::Carry(first.root.instance_table),
                tree_birth_txg: first.tree_birth_txg(),
                tree_identifier_watermark: first.root.tree_identifier_watermark,
                rollback_floor: first.root.rollback_floor,
            },
            Some(&first),
        )
        .expect("空发布")
    };
    let counters: Vec<u64> = empty_publish
        .earlier_records_of_this_publish
        .iter()
        .map(|written| written.record.counter)
        .chain([empty_publish.record.counter])
        .collect();
    assert!(
        counters.len() >= 2,
        "一条记录装两项，空发布重写的四五个角色跨两条以上：{counters:?}"
    );
    assert!(
        empty_publish
            .earlier_records_of_this_publish
            .iter()
            .all(|written| written.record.transaction == 0 && !written.record.is_commit)
            && empty_publish.record.transaction == 0
            && empty_publish.record.is_commit,
        "事务号 0 的发布跨多条记录：前几条的提交标记写 0，只有末条带"
    );

    let crash_image = image_before_the_last_root(&pool.stream);
    let applied_version = (InstanceGeneration(1), empty_publish.root.checkpoint_txg);
    let control = recover(&crash_image, JournalPolicy::Consult);
    assert_eq!(
        effective_root_and_applied(&control),
        (Some(applied_version), counters.len()),
        "阳性对照：什么都不改时空发布那几条整次施加"
    );

    let mut damaged = crash_image.clone();
    set_commit_marker_byte(&mut damaged, counters[0], 2);
    let report = recover(&damaged, JournalPolicy::Consult);
    assert_eq!(
        effective_root_and_applied(&report),
        (Some((InstanceGeneration(1), first.root.checkpoint_txg)), 0),
        "第一条的提交标记是 2：当它损坏、断链即止，空发布一条都不施加"
    );
    assert!(
        matches!(&report.outcome, RecoveryOutcome::FileRead { content, .. } if *content == file_content()),
        "读回 A：{:?}",
        report.outcome
    );
    let verdict = check_pool_image(&damaged)
        .into_iter()
        .find(|(invariant, _)| *invariant == "I-8.8")
        .map(|(_, verdict)| verdict)
        .expect("checker 每次都报 I-8.8");
    assert!(
        matches!(&verdict, InvariantVerdict::Violated(detail) if detail.contains("提交标记字节")),
        "checker 的 I-8.8 判「提交标记字节」：{verdict:?}"
    );
}

/// 把 `counter` 那条记录的提交标记字节改成 `value`，两盘各一份，再重封头部校验和（罩整条 4096、自身按 0 参与）：
/// 这条记录照样自证得过，拦住它的只剩那一字节。
fn set_commit_marker_byte(image: &mut MemoryPool, counter: u64, value: u8) {
    let offset = record_offset(counter, JOURNAL_RING_DEFAULT_BYTES);
    let record_bytes = usize::try_from(JOURNAL_RECORD_BYTES).expect("4096");
    for device in DISKS {
        let mut bytes =
            PoolReader::read(image, device, offset, record_bytes).expect("环里那一格读得到");
        bytes[COMMIT_MARKER_OFFSET] = value;
        let digest =
            wide_checksum_with_field_zeroed(&bytes, record_bytes, JOURNAL_HEADER_CHECKSUM_OFFSET);
        bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
            .copy_from_slice(&digest);
        image
            .devices
            .get_mut(&device)
            .expect("有这块盘")
            .write(offset, &bytes);
    }
}

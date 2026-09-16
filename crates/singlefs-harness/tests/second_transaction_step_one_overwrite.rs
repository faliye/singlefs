//! 里程碑「第二个事务」步 1 / 步 2 的验收：第一个事务之后、同一个实例里对同一个文件覆盖写一次（发布 B），
//! 冷启动读回第二次的内容；录制流的段序列与第一个事务同型 `16+2+1+2`；被换下的八个单元在分配记录树里改写成
//! 已释放 + 释放代 4、记账的 defer 待释放行等于它们的字节数、已分配行仍把它们算在内（I-3.1 读法甲：根环里 A 的根还引用着它们）；
//! 池级 checker 全绿；坏字节探针：新单元两份都坏 ⇒ 恢复失败，旧单元两份都坏 ⇒ 最新根不受影响，B 的根槽坏一字节 ⇒ 由记录重建。

mod common;

use std::collections::BTreeSet;

use common::{
    build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber};
use singlefs_core::allocator::UnitFootprint;
use singlefs_core::journal::{back_chain_of, record_offset};
use singlefs_core::records::{
    build_mapping_entry, parse_mapping_entry, STATISTIC_ALLOCATED_BYTES,
    STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS,
    STATISTIC_FREE_BYTES, STATISTIC_INODE_WATERMARK,
};
use singlefs_core::recovery::{
    choose_superblock, recover, JournalPolicy, JournalScanReport, PoolReader, RecoveryFailure,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    mapping_locations_for_key, placements_to_release_via_mapping, publish_overwrite, FirstFile,
    PoolWriter, PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{
    build_index_node, data_unit_payload_capacity, index_node_entry_capacity, parse_index_node,
};
use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::crash::MemoryPool;
use singlefs_harness::segments::{segment_kinds_text, segment_sizes_text, split_into_segments};

/// 第二次写的内容：与第一次不同长、不同字节。
const SECOND_FILE_BYTES: usize = 4100;

fn second_content() -> Vec<u8> {
    (0..SECOND_FILE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

/// 在第一个事务之后、同一个进程同一个实例里覆盖写一次；返回 B 的输出与它在录制流里的起点。
fn overwrite(pool: &mut BuiltPool) -> (TransactionOutput, usize) {
    let operations_before = pool.stream.operations().len();
    let previous = pool.output.clone();
    let output = try_overwrite(pool, &previous).expect("覆盖写");
    (output, operations_before)
}

/// 接在 `previous` 之后覆盖写一次，错误原样交回（装不下、不在映射那几条用例要看它）。
fn try_overwrite(
    pool: &mut BuiltPool,
    previous: &TransactionOutput,
) -> Result<TransactionOutput, PublishError> {
    let parameters = parameters();
    let content = second_content();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        previous,
        FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    )
}

fn slot_of(output: &TransactionOutput, identity: TransactionUnit) -> SlotNumber {
    output.unit(identity).slot
}

#[test]
fn overwrite_publishes_the_second_version_through_the_same_commit_shape() {
    let mut pool = build_pool("overwrite-shape");
    let (second, operations_before) = overwrite(&mut pool);
    let first = &pool.output;

    let operations = pool.stream.operations();
    let segments = split_into_segments(&operations[operations_before..], &geometry());
    assert_eq!(
        segment_sizes_text(&segments),
        "16+2+1+2",
        "覆盖写的段序列与第一个事务同型（登记表八 B 那一行的预想）"
    );
    assert_eq!(
        segment_kinds_text(&segments),
        "[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]"
    );

    assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
    assert_eq!(second.root.instance, InstanceGeneration(1));
    assert_eq!(
        second.root.tree_identifier_watermark, 19,
        "没有建新树，水位不动"
    );
    assert_eq!(second.root.rollback_floor, CheckpointTxg(0));
    assert_eq!(
        second.root.instance_table, first.root.instance_table,
        "实例表单元照旧"
    );
    assert_ne!(
        second.root.mapping_root, first.root.mapping_root,
        "映射树 COW 出新版本"
    );
    assert_eq!(second.record.counter, 4, "jsn 接着 A 的 3");
    assert_eq!(
        second.record.transaction, 2,
        "事务号按实例计数从 1 起，B 是 2"
    );
    assert_eq!(second.record.checkpoint_txg, CheckpointTxg(4));
    assert_eq!(
        second.record.back_chain,
        back_chain_of(&first.record_bytes),
        "反向链 = A 那条记录头的 CRC32C"
    );
    assert_eq!(second.record.named.len(), 8, "点名项 = 这次新写的单元数");

    let expected_slots = [
        (TransactionUnit::Data, 50182),
        (TransactionUnit::ExtentRoot, 50249),
        (TransactionUnit::InodeLeaf, 50250),
        (TransactionUnit::InodeRoot, 50252),
        (TransactionUnit::AllocationTree, 50253),
        (TransactionUnit::AccountingTree, 50254),
        (TransactionUnit::MappingTree, 50255),
        (TransactionUnit::TreeTable, 50256),
    ];
    for (identity, slot) in expected_slots {
        assert_eq!(
            slot_of(&second, identity),
            SlotNumber(slot),
            "{}：数据单元落 50180 之后最低的偶数空槽对，提交内生块接着 A 的 bump 往后",
            identity.tag()
        );
    }
    assert_eq!(
        second.released,
        first.placements(),
        "释放的正是 A 写出的八个落点"
    );
    assert_eq!(
        second.inode_record.object_birth,
        CheckpointTxg(3),
        "对象出生代不改"
    );
    assert_eq!(
        second.inode_record.change_count, 4,
        "改动计数 = 这次发布的 checkpoint_txg"
    );
    assert_eq!(
        second.inode_record.size,
        u64::try_from(SECOND_FILE_BYTES).expect("长度")
    );
    assert_eq!(
        second.inode_record.write_time_seconds,
        FIXED_WRITE_TIME_SECONDS + 60
    );
    assert_eq!(second.key_order_mismatches, 0);
    assert_eq!(pool.allocator.policy_mismatches, 0);

    // 根槽落区域 4 mod 3 = 1 的槽 (4 div 3) mod 8 = 1，区域 1 归盘 1；超级块世代号 6、tail = 4。
    let image = pool.memory_pool();
    let target = target_for_publish(CheckpointTxg(4));
    let region_device =
        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    assert_eq!(region_device, DeviceIdentity(1));
    let root_bytes =
        PoolReader::read(&image, region_device, slot_offset(target, 4096), 512).expect("读根槽");
    let root = RootRecord::parse_slot(&root_bytes, &parameters().filesystem_identifier)
        .expect("根槽自证过");
    assert_eq!(root, second.root);
    let superblock = choose_superblock(&image).expect("超级块");
    assert_eq!(superblock.journal_tail, 4);
    assert_eq!(
        superblock.slot_generation, 6,
        "mkfs 1、取号 2、暖机 3 / 4、A 5、B 6"
    );
}

#[test]
fn release_rewrites_the_first_versions_records_and_accounting_moves_them_into_the_defer_queue() {
    let mut pool = build_pool("overwrite-release");
    let first_slots: BTreeSet<SlotNumber> =
        pool.output.units.iter().map(|unit| unit.slot).collect();
    let first_mapping_keys: BTreeSet<Vec<u8>> = pool.output.mapping_keys.iter().cloned().collect();
    let (second, _) = overwrite(&mut pool);
    let second_slots: BTreeSet<SlotNumber> = second.units.iter().map(|unit| unit.slot).collect();

    assert_eq!(
        second.allocation_records.len(),
        36,
        "mkfs 2 + A 8 + B 8 个落点 × 2 盘：A 的改写不删"
    );
    let mut released_count = 0;
    let mut fresh_count = 0;
    let mut format_time_count = 0;
    for record in &second.allocation_records {
        if first_slots.contains(&record.slot) {
            assert!(
                record.is_released,
                "A 的落点 {} 改写成已释放",
                record.slot.0
            );
            assert_eq!(record.generation, CheckpointTxg(4), "释放代 = B 的 txg");
            released_count += 1;
        } else if second_slots.contains(&record.slot) {
            assert!(!record.is_released);
            assert_eq!(record.generation, CheckpointTxg(4), "分配代 = B 的 txg");
            fresh_count += 1;
        } else {
            assert_eq!(record.generation, CheckpointTxg(0), "mkfs 的单元分配代 0");
            format_time_count += 1;
        }
    }
    assert_eq!(
        (released_count, fresh_count, format_time_count),
        (16, 16, 4)
    );

    let unit_area_slots = IMAGE_BYTES / SLOT_BYTES - UNIT_AREA_START_SLOT;
    let occupied_slots = 3 + 10 + 10;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        let value = |statistic: u16| {
            second
                .accounting_entries
                .iter()
                .find(|entry| entry.statistic == statistic && entry.device == device)
                .map(|entry| entry.value)
                .expect("带设备维的行每盘一行")
        };
        assert_eq!(
            value(STATISTIC_ALLOCATED_BYTES),
            occupied_slots * SLOT_BYTES,
            "已分配 = mkfs 3 槽 + A 10 槽 + B 10 槽：A 的单元仍被根环里 A 的根引用、仍占着空间（I-3.1 读法甲）"
        );
        assert_eq!(
            value(STATISTIC_DEFER_QUEUE_BYTES),
            10 * SLOT_BYTES,
            "defer 待释放 = A 的八个单元 10 槽"
        );
        assert_eq!(
            value(STATISTIC_FREE_BYTES),
            3_472_506_880,
            "空闲钉绝对值：4 GiB 镜像单元区 211968 槽里占着 23 槽，剩 211945 槽 × 16384；分配器独立维护它，不由「单元区 − 已分配」现算，已释放的不算空闲"
        );
        assert_eq!(
            unit_area_slots, 211_968,
            "钉住单元区容量，checker 的 I-5.2 拿空闲 + 已分配对它"
        );
        assert_eq!(
            value(STATISTIC_FRAGMENTATION_RUNS),
            4,
            "[50179]、[50184, 50239]、[50241]、[50257, 末]"
        );
        assert_eq!(
            value(STATISTIC_EMPTY_CLUSTER_SEGMENTS),
            3310,
            "B 的提交内生块仍在 A 开的那个段里"
        );
    }
    let watermark = second
        .accounting_entries
        .iter()
        .find(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
        .expect("inode 号水位");
    assert_eq!(watermark.value, 2, "没有建新 inode");
    assert_eq!(second.accounting_entries.len(), 15);

    assert_eq!(second.mapping_keys.len(), 6, "码 1 一条 + 码 2 / 码 3 五条");
    for key in &second.mapping_keys {
        assert!(
            !first_mapping_keys.contains(key),
            "A 的映射条目一条都不留：释放走映射、条目删掉"
        );
    }
    assert_eq!(second.tree_table_entries.len(), 7);
    for (entry, previous) in second
        .tree_table_entries
        .iter()
        .zip(&pool.output.tree_table_entries)
    {
        assert_eq!(entry.tree, previous.tree);
        assert_eq!(entry.birth_txg, CheckpointTxg(3), "树的诞生 txg 不随重写变");
    }
    assert_eq!(
        second.tree_table_entries[0].root.head.birth_txg,
        CheckpointTxg(4),
        "extent 树根 COW 到 txg 4"
    );
}

#[test]
fn cold_start_reads_the_second_content_and_the_pool_checker_stays_green() {
    let mut pool = build_pool("overwrite-cold");
    let (_, _) = overwrite(&mut pool);
    let image = pool.memory_pool();
    let reopened = pool.reopen_cold();
    let report = recover(&reopened, JournalPolicy::Consult);
    assert_eq!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: second_content()
        },
        "冷启动择 B 的根、读回第二次的内容"
    );
    assert_eq!(
        report.journal,
        JournalScanReport {
            valid_records: 4,
            above_water: 0,
            prefix_applied: 0,
            verification_passed: 0,
            verification_failed: 0
        },
        "暖机两条 + A + B 四条记录，没有一条高于所选根"
    );
    assert_eq!(report.mapping_fallbacks, 0);
    assert_eq!(
        recover(&reopened, JournalPolicy::Ignore).outcome,
        report.outcome,
        "根槽已持久：看不看 journal 结果一样"
    );
    assert_ne!(
        report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(4)),
            content: file_content()
        },
        "第一次的内容从最新根出发读不到"
    );

    let verdicts = check_pool_image(&image);
    for (invariant, verdict) in &verdicts {
        assert!(
            !matches!(verdict, InvariantVerdict::Violated(_)),
            "{invariant} 在覆盖写之后的镜像上判红：{verdict:?}"
        );
    }
    for must_hold in ["I-3.1", "I-5.1", "I-5.2", "I-7.2"] {
        let (_, verdict) = verdicts
            .iter()
            .find(|(invariant, _)| *invariant == must_hold)
            .expect("checker 报了这一条");
        assert_eq!(
            verdict,
            &InvariantVerdict::Holds,
            "{must_hold} 要真被评估过、且成立"
        );
    }
}

#[test]
fn damage_probes_after_the_overwrite_tell_the_new_unit_from_the_released_one() {
    let mut pool = build_pool("overwrite-probes");
    let (second, _) = overwrite(&mut pool);
    let full = pool.memory_pool();
    let second_data = slot_of(&second, TransactionUnit::Data).to_device_offset();
    let first_data = pool
        .output
        .unit(TransactionUnit::Data)
        .slot
        .to_device_offset();
    let both = |image: &MemoryPool, offset, byte| {
        let mut damaged = image.clone();
        damaged.flip_byte(DeviceIdentity(0), offset, byte);
        damaged.flip_byte(DeviceIdentity(1), offset, byte);
        damaged
    };
    let file_read = RecoveryOutcome::FileRead {
        root: (InstanceGeneration(1), CheckpointTxg(4)),
        content: second_content(),
    };

    let new_unit_damaged = both(&full, second_data, 200);
    let new_unit_report = recover(&new_unit_damaged, JournalPolicy::Consult);
    assert_eq!(
        new_unit_report.outcome,
        RecoveryOutcome::Failed {
            root: Some((InstanceGeneration(1), CheckpointTxg(4))),
            failure: RecoveryFailure::MappingStillUnreadable {
                slot: SlotNumber(50182)
            }
        },
        "新单元两份都坏：提示读不到、经映射仍读不到，不返回坏数据"
    );
    assert_eq!(new_unit_report.mapping_fallbacks, 1);

    let released_unit_damaged = both(&full, first_data, 200);
    assert_eq!(
        recover(&released_unit_damaged, JournalPolicy::Consult).outcome,
        file_read,
        "被换下的旧单元两份都坏：最新根不引用它，读回不受影响"
    );

    let record_damaged = both(&full, record_offset(4, JOURNAL_RING_DEFAULT_BYTES), 200);
    let record_damaged_report = recover(&record_damaged, JournalPolicy::Consult);
    assert_eq!(
        record_damaged_report.outcome, file_read,
        "B 的根槽已持久，记录坏了不碍事"
    );
    assert_eq!(record_damaged_report.journal.valid_records, 3);

    let target = target_for_publish(CheckpointTxg(4));
    let region_device =
        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    let mut root_damaged = full.clone();
    root_damaged.flip_byte(region_device, slot_offset(target, 4096), 100);
    let root_damaged_report = recover(&root_damaged, JournalPolicy::Consult);
    assert_eq!(
        root_damaged_report.outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: second_content()
        },
        "B 的根槽坏一字节：择回 A 的根，再由 jsn 4 那条记录重建第 4 代根，内容仍是第二次的"
    );
    assert_eq!(
        root_damaged_report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4)))
    );
    assert_eq!(root_damaged_report.journal.prefix_applied, 1);
    assert_eq!(root_damaged_report.journal.verification_passed, 1);
    assert_eq!(
        recover(&root_damaged, JournalPolicy::Ignore).outcome,
        RecoveryOutcome::FileRead {
            root: (InstanceGeneration(1), CheckpointTxg(3)),
            content: file_content()
        },
        "不看 journal 就退回 A：第一次的内容——journal 在这一格承重"
    );
}

/// 释放判定路径（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：释放一律经映射，不经提示）：A 的八个落点经 A 的映射取出来与 A 自己记的槽号逐个相同；
/// B 之后 A 的数据单元那把 key 在 B 的映射里查不到（「不在映射」）；把 A 的映射节点重装成少了码 1 那一条的形态再覆盖写 ⇒ 报「不在映射」、
/// 一个落点都不释放、分配器一条记录都没改写——按提示释放的写法会把它当成一次正常的覆盖写。
#[test]
fn release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released() {
    let mut pool = build_pool("release-via-mapping");
    let first = pool.output.clone();
    let via_mapping =
        placements_to_release_via_mapping(&first, &pool.allocator).expect("A 的六条映射都在");
    assert_eq!(
        via_mapping,
        first.placements(),
        "经映射取的八个落点与写者自己记的槽号一致（两条路各自算）"
    );
    let first_data_key = &first
        .mapped_units
        .iter()
        .find(|(unit, _)| *unit == TransactionUnit::Data)
        .expect("码 1 一把 key")
        .1;
    let (second, _) = overwrite(&mut pool);
    let second_mapping = &second.unit(TransactionUnit::MappingTree).bytes;
    assert_eq!(
        mapping_locations_for_key(second_mapping, first_data_key),
        None,
        "释放判定路径对已换下的单元报「不在映射」"
    );
    let second_data_key = &second
        .mapped_units
        .iter()
        .find(|(unit, _)| *unit == TransactionUnit::Data)
        .expect("码 1 一把 key")
        .1;
    assert_eq!(
        mapping_locations_for_key(second_mapping, second_data_key)
            .map(|locations| locations[0].slot),
        Some(SlotNumber(50182)),
        "B 自己的数据单元经映射查得到、落点 50182"
    );

    let mut fresh_pool = build_pool("release-missing-entry");
    let mut damaged = fresh_pool.output.clone();
    let mapping_node = parse_index_node(&damaged.unit(TransactionUnit::MappingTree).bytes)
        .expect("A 的映射节点解得开");
    let kept: Vec<Vec<u8>> = mapping_node
        .entries
        .iter()
        .filter(|entry| !entry.starts_with(first_data_key))
        .cloned()
        .collect();
    assert_eq!(kept.len(), 5, "删掉码 1 那一条，剩五条");
    let rebuilt = build_index_node(
        mapping_node.tree,
        mapping_node.level,
        mapping_node.key_width,
        &kept[0][..mapping_node.key_width],
        &kept[4][..mapping_node.key_width],
        mapping_node.birth_txg,
        &parameters().filesystem_identifier,
        mapping_node.instance,
        mapping_node.birth_sequence,
        u16::try_from(mapping_node.entry_width).expect("条目宽"),
        &kept,
    );
    let mapping_index = damaged
        .units
        .iter()
        .position(|unit| unit.identity == TransactionUnit::MappingTree)
        .expect("八个单元每种一个");
    damaged.units[mapping_index].bytes = rebuilt;
    let records_before = fresh_pool.allocator.records().to_vec();
    let result = try_overwrite(&mut fresh_pool, &damaged);
    assert!(
        matches!(
            result,
            Err(PublishError::ReleaseNotInMapping {
                unit: TransactionUnit::Data
            })
        ),
        "删掉映射条目 ⇒ 报「不在映射」而不是按提示释放：{result:?}"
    );
    assert_eq!(
        fresh_pool.allocator.records(),
        &records_before[..],
        "报「不在映射」就不动分配器：一条记录都没改写"
    );
}

/// 分配记录树第一版只有一个节点（16384 − 头 135 = 16249 字节，每条 20 字节，装 812 条）：每次覆盖写每盘加 8 条、释放只改写不删，
/// 第 49 次之后 804 条，第 50 次要 820 条 ⇒ 报 `AllocationRecordsExceedOneNode`，不 panic，而且报在动分配器之前。
#[test]
fn repeated_overwrites_report_a_full_allocation_node_instead_of_panicking() {
    assert_eq!(index_node_entry_capacity(10, 20), 812);
    let mut pool = build_pool("fifty-overwrites");
    let mut successful_rounds = 0;
    let failure = loop {
        let previous = pool.output.clone();
        match try_overwrite(&mut pool, &previous) {
            Ok(output) => {
                pool.output = output;
                successful_rounds += 1;
            }
            Err(error) => break error,
        }
    };
    assert_eq!(
        successful_rounds, 49,
        "20 + 16 × 49 = 804 ≤ 812，第 50 次要 820 条"
    );
    assert_eq!(pool.allocator.records().len(), 804);
    assert!(
        matches!(
            failure,
            PublishError::AllocationRecordsExceedOneNode {
                records: 820,
                capacity: 812
            }
        ),
        "{failure:?}"
    );
    assert_eq!(
        pool.allocator
            .records()
            .iter()
            .filter(|record| record.is_released)
            .count(),
        16 * 49,
        "报错在动分配器之前：最后一版的八个落点没被释放"
    );
}

/// 可再分配谓词（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）在步 5 之前没有实现：这条用例把今天的形态钉住——
/// 释放过的落点一个都不再发出去、defer 只增不减、空闲随分配单调减。它是 C22（刚释放的块立即重分配）的弱形态：把 `mark_released` 改成清位图（立即复用）它红；
/// 步 5 把回收接上那天它也必须红（defer 会减、50180 会回来），到时改成按谓词判。
#[test]
fn released_placements_are_not_handed_out_again_before_reclaim_exists() {
    let mut pool = build_pool("no-reclaim-yet");
    let first_placements = pool.output.placements();
    let (second, _) = overwrite(&mut pool);
    let second_placements = second.placements();
    pool.output = second;
    let (third, _) = overwrite(&mut pool);
    let third_placements = third.placements();
    let device_map = &pool.allocator.devices[0];
    assert_eq!(
        device_map.deferred_slots(),
        20,
        "A 与 B 的 20 槽都在 defer 队列里，一个都没放回"
    );
    assert_eq!(
        device_map.allocated_slots(),
        3 + 10 + 10 + 10,
        "占着的：mkfs 3 + A 10 + B 10 + C 10"
    );
    assert_eq!(device_map.free_slots(), 211_968 - 33, "空闲只随分配减");
    for released in first_placements.iter().chain(second_placements.iter()) {
        assert!(
            !third_placements
                .iter()
                .any(|placement| placement.slot == released.slot),
            "释放过的落点 {released:?} 在回收实现之前不许再发出去"
        );
        assert!(!device_map.is_free(released.slot), "已释放的槽仍占着");
    }
    assert_eq!(
        slot_of(&third, TransactionUnit::Data),
        SlotNumber(50184),
        "C 的数据单元落在 B 之后的下一对偶数空槽"
    );
}

/// 把上一版的映射节点重装成「码 1 那一条的两处位置都指向 `slot`」的形态，别的条目不动。
fn previous_with_data_mapping_entry_pointing_at(
    output: &TransactionOutput,
    slot: SlotNumber,
) -> TransactionOutput {
    let mut damaged = output.clone();
    let data_key = &output
        .mapped_units
        .iter()
        .find(|(unit, _)| *unit == TransactionUnit::Data)
        .expect("码 1 一把 key")
        .1;
    let mapping_node = parse_index_node(&damaged.unit(TransactionUnit::MappingTree).bytes)
        .expect("上一版的映射节点解得开");
    let entries: Vec<Vec<u8>> = mapping_node
        .entries
        .iter()
        .map(|entry| {
            let (key, mut locations) = parse_mapping_entry(entry);
            if key == *data_key {
                for location in &mut locations {
                    location.slot = slot;
                }
                build_mapping_entry(&key, locations)
            } else {
                entry.clone()
            }
        })
        .collect();
    let rebuilt = build_index_node(
        mapping_node.tree,
        mapping_node.level,
        mapping_node.key_width,
        &entries[0][..mapping_node.key_width],
        &entries[entries.len() - 1][..mapping_node.key_width],
        mapping_node.birth_txg,
        &parameters().filesystem_identifier,
        mapping_node.instance,
        mapping_node.birth_sequence,
        u16::try_from(mapping_node.entry_width).expect("条目宽"),
        &entries,
    );
    let mapping_index = damaged
        .units
        .iter()
        .position(|unit| unit.identity == TransactionUnit::MappingTree)
        .expect("八个单元每种一个");
    damaged.units[mapping_index].bytes = rebuilt;
    damaged
}

/// 映射条目在、落点指错：指向一个没有分配记录的槽 ⇒ `ReleaseTargetNotAllocated`；指向 A 的 extent 根（1 槽的单元）⇒ 记录的跨度 1
/// 与码 1 该有的 2 不符 ⇒ `ReleaseSpanMismatch`。两次都在动分配器之前报错——查得到 key 就直接交给 `release` 的写法会在断言上 panic。
#[test]
fn release_reports_a_mapping_entry_whose_slot_has_no_record_or_the_wrong_span_instead_of_panicking()
{
    let mut pool = build_pool("mapping-entry-points-elsewhere");
    let records_before = pool.allocator.records().to_vec();
    let unallocated = previous_with_data_mapping_entry_pointing_at(&pool.output, SlotNumber(60000));
    let unallocated_result = try_overwrite(&mut pool, &unallocated);
    assert!(
        matches!(
            unallocated_result,
            Err(PublishError::ReleaseTargetNotAllocated {
                unit: TransactionUnit::Data,
                slot: SlotNumber(60000)
            })
        ),
        "{unallocated_result:?}"
    );
    let extent_root_slot = slot_of(&pool.output, TransactionUnit::ExtentRoot);
    let one_slot_unit =
        previous_with_data_mapping_entry_pointing_at(&pool.output, extent_root_slot);
    let one_slot_result = try_overwrite(&mut pool, &one_slot_unit);
    assert!(
        matches!(
            one_slot_result,
            Err(PublishError::ReleaseSpanMismatch {
                unit: TransactionUnit::Data,
                slot,
                recorded_span: 1,
                expected_span: 2
            }) if slot == extent_root_slot
        ),
        "{one_slot_result:?}"
    );
    assert_eq!(
        pool.allocator.records(),
        &records_before[..],
        "两次都在动分配器之前报错：一条记录都没改写"
    );
}

/// 准入之后失败的发布不留半新的池：把 A 开的那个提交内生段用到头、单元区里别的空槽全标成已分配、只留 50182–50183 给 B 的数据单元，
/// B 释放了 A、分配到了数据单元、第一个提交内生块拿不到 ⇒ `NoSpaceFor`；返回之后分配器要和进去之前一模一样。
#[test]
fn publish_running_out_of_space_midway_leaves_the_allocator_as_it_was() {
    let mut pool = build_pool("no-space-midway");
    let first_drained = pool
        .allocator
        .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
        .expect("A 的段里还有槽");
    assert_eq!(
        first_drained.slot,
        SlotNumber(50249),
        "A 之后 bump 游标停在 50249"
    );
    for _drain_round in 0..54 {
        pool.allocator
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
            .expect("段里还有槽");
    }
    assert_eq!(
        pool.allocator.open_segment(),
        Some(SlotNumber(50240)),
        "段用到头但还开着"
    );
    let unit_area_end = IMAGE_BYTES / SLOT_BYTES;
    for device_map in &mut pool.allocator.devices {
        for slot in UNIT_AREA_START_SLOT..unit_area_end {
            if slot == 50182 || slot == 50183 {
                continue;
            }
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    let records_before = pool.allocator.records().to_vec();
    let previous = pool.output.clone();
    let result = try_overwrite(&mut pool, &previous);
    assert!(
        matches!(
            result,
            Err(PublishError::NoSpaceFor {
                unit: TransactionUnit::ExtentRoot
            })
        ),
        "数据单元拿到了 50182，第一个提交内生块拿不到：{result:?}"
    );
    assert_eq!(
        pool.allocator.records(),
        &records_before[..],
        "失败的发布退回：A 的记录没改写成已释放、B 的数据单元没留下记录"
    );
    let device_map = &pool.allocator.devices[0];
    assert_eq!(device_map.deferred_slots(), 0, "A 的落点没进 defer 队列");
    assert!(
        device_map.is_free(SlotNumber(50182)),
        "B 拿到过的数据槽退回去了"
    );
    assert_eq!(
        pool.allocator.open_segment(),
        Some(SlotNumber(50240)),
        "开放段也退回去了"
    );
}

/// 内容装不进一个数据单元（32768 − 头 105 − 预留 29 = 32634 字节）是调用方能恢复的失败：报 `ContentExceedsDataUnit`、不动分配器、不 panic。
#[test]
fn content_larger_than_a_data_unit_payload_is_refused_before_anything_is_touched() {
    assert_eq!(data_unit_payload_capacity(), 32_634);
    let mut pool = build_pool("content-too-long");
    let records_before = pool.allocator.records().to_vec();
    let too_long = vec![7u8; 32_635];
    let previous = pool.output.clone();
    let parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
    let result = publish_overwrite(
        &mut writer,
        &mut pool.allocator,
        &previous,
        FirstFile {
            content: &too_long,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
        },
        InstanceGeneration(1),
    );
    assert!(
        matches!(
            result,
            Err(PublishError::ContentExceedsDataUnit {
                bytes: 32_635,
                capacity: 32_634
            })
        ),
        "{result:?}"
    );
    assert_eq!(pool.allocator.records(), &records_before[..]);
}

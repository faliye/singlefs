//! 里程碑「覆盖写、释放、回退与复用」步 1 / 步 2 的验收：新池新建文件之后、同一个实例里对同一个文件覆盖写一次（发布 B），
//! 冷启动读回第二次的内容；录制流的段序列与新池新建文件同型 `24+2+1+2`（七个角色加分配记录树五个节点，D8（核心索引结构） 已定项 14）；
//! 被换下的十二个单元在分配记录树里改写成已释放 + 释放代 4、记账的 defer 待释放行等于它们的字节数、已分配行仍把它们算在内（I-3.1 读法甲：根环里 A 的根还引用着它们）；
//! 池级 checker 全绿；坏字节探针：新单元两份都坏 ⇒ 恢复失败，旧单元两份都坏 ⇒ 最新根不受影响，B 的根槽坏一字节 ⇒ 由记录重建。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/overwrite_in_one_instance.rs"]
mod scenarios_of_overwrite_in_one_instance;

use common::{
    build_pool, file_content, geometry, parameters, BuiltPool, FIXED_WRITE_TIME_SECONDS,
    IMAGE_BYTES,
};
use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_overwrite_in_one_instance::{
    overwrite, second_content, try_overwrite, SECOND_FILE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
    SlotNumber,
};
use singlefs_core::admission::{
    AvailableBytesOnOneDevice, BytesOnOneDevice, DeviceShortOfDemand, SpaceAdmission,
};
use singlefs_core::allocation_record_tree::AllocationRecordTreeNodePosition;
use singlefs_core::allocator::{
    AllocationRecord, PlacementRefusal, PoolAllocator, ReuseWindow, UnitFootprint,
};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::journal::{back_chain_of, record_offset};
use singlefs_core::records::{
    build_mapping_entry, parse_mapping_entry, STATISTIC_ALLOCATED_BYTES,
    STATISTIC_DEFER_QUEUE_BYTES, STATISTIC_EMPTY_CLUSTER_SEGMENTS, STATISTIC_FRAGMENTATION_RUNS,
    STATISTIC_FREE_BYTES, STATISTIC_INODE_WATERMARK,
};
use singlefs_core::recovery::{
    choose_system_configuration, recover, JournalPolicy, PoolReader, RecoveryFailure,
    RecoveryOutcome,
};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::transaction::{
    mapping_locations_for_key, placements_to_release_via_mapping, publish_overwrite, FirstFile,
    MappingLookup, PoolWriter, PublishError, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{build_index_node, data_unit_payload_capacity, parse_index_node};
use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, JOURNAL_RING_DEFAULT_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::memory_pool::MemoryPool;
use singlefs_harness::segments::{segment_kinds_text, segment_sizes_text, split_into_segments};
use std::collections::BTreeSet;

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
        "24+2+1+2",
        "覆盖写的段序列与新池新建文件同型（登记表八 B 那一行的预想；分配记录树按位置寻址之后十二个单元，D8（核心索引结构） 已定项 14）"
    );
    assert_eq!(
        segment_kinds_text(&segments),
        "[unit_write×24,barrier×2]|[journal_record×2,barrier×2]|[root_record_fua]|[system_configuration_slot×2,barrier×2]",
        "种类串里屏障按设备数：每道池屏障两块盘各记一步（实审 B3a-2 第 1 条，录制器按设备记屏障）；C577 之后系统配置轮换之后、\
         发布返回之前多一道池屏障，末段多 barrier×2"
    );

    assert_eq!(second.root.checkpoint_txg, CheckpointTxg(4));
    assert_eq!(second.root.instance, InstanceGeneration(1));
    assert_eq!(
        second.root.tree_identifier_watermark, 65549,
        "没有建新树，水位不动（第一个文件版本发 65536..65548 之后是 65549，D8 已定项 11）"
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
    assert_eq!(second.record.named.len(), 12, "点名项 = 这次新写的单元数");

    // 分配记录树按位置寻址（D8（核心索引结构） 已定项 14）：4 GiB 两块盘上根在第 2 层，B 改的记录都在两块盘各自的叶 61 里，
    // 重写那两片叶、两个第 1 层节点与根，先叶后根。
    let allocation_record_tree_node = |level: u8, device: u32, index_in_device: u64| {
        TransactionUnit::AllocationTreeNodeBelowTheRoot(AllocationRecordTreeNodePosition {
            level,
            device: DeviceIdentity(device),
            index_in_device,
        })
    };
    let expected_slots = [
        (TransactionUnit::Data(DataUnitIndexInFile::FIRST), 50182),
        (TransactionUnit::ExtentRoot, 50253),
        (
            TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST),
            50254,
        ),
        (TransactionUnit::InodeRoot, 50256),
        (allocation_record_tree_node(0, 0, 61), 50257),
        (allocation_record_tree_node(0, 1, 61), 50258),
        (allocation_record_tree_node(1, 0, 0), 50259),
        (allocation_record_tree_node(1, 1, 0), 50260),
        (TransactionUnit::AllocationTree, 50261),
        (TransactionUnit::AccountingTree, 50262),
        (TransactionUnit::MappingTree, 50263),
        (TransactionUnit::TreeTable, 50264),
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
        "释放的正是 A 写出的十二个落点"
    );
    assert_eq!(
        second
            .inode_record
            .expect("覆盖写那一版有第一个文件那条记录")
            .object_birth,
        CheckpointTxg(3),
        "对象出生代不改"
    );
    assert_eq!(
        second
            .inode_record
            .expect("覆盖写那一版有第一个文件那条记录")
            .change_count,
        4,
        "改动计数 = 这次发布的 checkpoint_txg"
    );
    assert_eq!(
        second
            .inode_record
            .expect("覆盖写那一版有第一个文件那条记录")
            .size,
        u64::try_from(SECOND_FILE_BYTES).expect("长度")
    );
    assert_eq!(
        second
            .inode_record
            .expect("覆盖写那一版有第一个文件那条记录")
            .write_time_seconds,
        FIXED_WRITE_TIME_SECONDS + 60
    );
    assert_eq!(second.key_order_mismatches, 0);

    // 根槽落区域 4 mod 3 = 1 的槽 (4 div 3) mod 8 = 1，区域 1 归盘 1；系统配置世代号 6、tail = 4。
    let image = pool.memory_pool();
    let target = target_for_publish(
        CheckpointTxg(4),
        parameters().geometry.root_ring_slots_per_region,
    );
    let region_device =
        parameters().region_devices[usize::try_from(target.region).expect("区域号")];
    assert_eq!(region_device, DeviceIdentity(1));
    let root_bytes =
        PoolReader::read(&image, region_device, slot_offset(target, 4096), 512).expect("读根槽");
    let root = RootRecord::parse_slot(&root_bytes, &parameters().filesystem_identifier)
        .expect("根槽自证过");
    assert_eq!(root, second.root);
    let system_configuration = choose_system_configuration(&image).expect("系统配置");
    assert_eq!(system_configuration.quantities.journal_tail, 4);
    assert_eq!(
        system_configuration.quantities.slot_generation, 6,
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
        52,
        "mkfs 2 + A 12 + B 12 个落点 × 2 盘（七个角色加分配记录树五个节点）：A 的改写不删"
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
        } else if record.slot == SlotNumber(50178) {
            assert!(record.is_released, "mkfs 的第 0 版树表被 A 换下、已释放");
            assert_eq!(record.generation, CheckpointTxg(3), "释放代 = A 的 txg");
            format_time_count += 1;
        } else {
            assert_eq!(
                record.generation,
                CheckpointTxg(0),
                "mkfs 的实例表单元分配代 0"
            );
            format_time_count += 1;
        }
    }
    assert_eq!(
        (released_count, fresh_count, format_time_count),
        (24, 24, 4)
    );

    let unit_area_slots = IMAGE_BYTES / SLOT_BYTES - UNIT_AREA_START_SLOT;
    let occupied_slots = 3 + 14 + 14;
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
            "已分配 = mkfs 3 槽 + A 14 槽 + B 14 槽：A 的单元仍被根环里 A 的根引用、仍占着空间（I-3.1 读法甲）"
        );
        assert_eq!(
            value(STATISTIC_DEFER_QUEUE_BYTES),
            15 * SLOT_BYTES,
            "defer 待释放 = A 的十二个单元 14 槽 + A 换下的 mkfs 树表 1 槽"
        );
        assert_eq!(
            value(STATISTIC_FREE_BYTES),
            3_472_375_808,
            "空闲钉绝对值：4 GiB 镜像单元区 211968 槽里占着 31 槽，剩 211937 槽 × 16384；分配器独立维护它，不由「单元区 − 已分配」现算，已释放的不算空闲"
        );
        assert_eq!(
            unit_area_slots, 211_968,
            "钉住单元区容量，checker 的 I-5.2 拿空闲 + 已分配对它"
        );
        assert_eq!(
            value(STATISTIC_FRAGMENTATION_RUNS),
            4,
            "[50179]、[50184, 50239]、[50241]、[50265, 末]"
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
        .filter(|entry| entry.statistic == STATISTIC_INODE_WATERMARK)
        .max_by_key(|entry| entry.generation)
        .expect("inode 号水位");
    assert_eq!(watermark.value, 2, "没有建新 inode");
    assert_eq!(
        second.accounting_entries.len(),
        16,
        "这一版自己的 15 行，加 A 那次留的一级 inode 号水位台阶 (A 的 txg, 2)（D5 已定项 2 射程）"
    );

    assert_eq!(
        second.mapping_keys.len(),
        10,
        "码 1 一条 + 码 2 / 码 3 九条（extent 根、inode 叶、inode 根、记账树根，分配记录树五个节点）"
    );
    for key in &second.mapping_keys {
        assert!(
            !first_mapping_keys.contains(key),
            "A 的映射条目一条都不留：释放走映射、条目删掉"
        );
    }
    assert_eq!(second.tree_table_entries.len(), 12);
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_overwrite_in_one_instance::cold_start_reads_the_second_content_and_the_pool_checker_stays_green`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn cold_start_reads_the_second_content_and_the_pool_checker_stays_green() {
    scenarios_of_overwrite_in_one_instance::cold_start_reads_the_second_content_and_the_pool_checker_stays_green(&mut PoolCheckerImages::left_to_the_checker_tier());
}

#[test]
fn damage_probes_after_the_overwrite_tell_the_new_unit_from_the_released_one() {
    let mut pool = build_pool("overwrite-probes");
    let (second, _) = overwrite(&mut pool);
    let full = pool.memory_pool();
    let second_data =
        slot_of(&second, TransactionUnit::Data(DataUnitIndexInFile::FIRST)).to_device_offset();
    let first_data = pool
        .output
        .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
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

    let target = target_for_publish(
        CheckpointTxg(4),
        parameters().geometry.root_ring_slots_per_region,
    );
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

/// 释放判定路径（D19（块指针的结构与宽度预算） 已定项 5 第 1 条：释放一律经映射，不经提示）：A 的十二个落点经 A 的映射取出来与 A 自己记的槽号逐个相同；
/// B 之后 A 的数据单元那把 key 在 B 的映射里查不到（「不在映射」）；把 A 的映射节点重装成少了码 1 那一条的形态再覆盖写 ⇒ 报「不在映射」、
/// 一个落点都不释放、分配器一条记录都没改写——按提示释放的写法会把它当成一次正常的覆盖写。
#[test]
fn release_goes_through_the_previous_mapping_and_a_missing_entry_is_reported_not_released() {
    let mut pool = build_pool("release-via-mapping");
    let first = pool.output.clone();
    // A 重写的十二个单元（七个角色加分配记录树五个节点，D8（核心索引结构） 已定项 14），按 bump 次序。
    let via_mapping = placements_to_release_via_mapping(&first, &pool.allocator, &first.rewritten)
        .expect("A 的十条映射都在");
    assert_eq!(
        via_mapping,
        first.placements(),
        "经映射取的十二个落点与写者自己记的槽号一致（两条路各自算）"
    );
    let first_data_key = &first
        .mapped_units
        .iter()
        .find(|(unit, _)| *unit == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .expect("码 1 一把 key")
        .1;
    let (second, _) = overwrite(&mut pool);
    let second_mapping = &second.unit(TransactionUnit::MappingTree).bytes;
    assert_eq!(
        mapping_locations_for_key(second_mapping, first_data_key),
        MappingLookup::NodeMalformedOrNoEntryWithThisKey,
        "释放判定路径对已换下的单元报「不在映射」"
    );
    let second_data_key = &second
        .mapped_units
        .iter()
        .find(|(unit, _)| *unit == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .expect("码 1 一把 key")
        .1;
    let MappingLookup::Found(second_data_locations) =
        mapping_locations_for_key(second_mapping, second_data_key)
    else {
        panic!("B 自己的数据单元在 B 的映射里查得到");
    };
    assert_eq!(
        second_data_locations.map(|location| location.slot),
        [SlotNumber(50182); 2],
        "B 自己的数据单元经映射查得到、两盘都落在 50182"
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
    assert_eq!(kept.len(), 9, "删掉码 1 那一条，剩九条");
    let rebuilt = build_index_node(
        mapping_node.tree,
        mapping_node.level,
        mapping_node.key_width,
        &kept[0][..mapping_node.key_width],
        &kept[kept.len() - 1][..mapping_node.key_width],
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
                unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST)
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_overwrite_in_one_instance::repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree(
) {
    scenarios_of_overwrite_in_one_instance::repeated_overwrites_go_past_the_812_records_of_one_leaf_across_several_leaves_of_the_allocation_record_tree(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_overwrite_in_one_instance::allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records(
) {
    scenarios_of_overwrite_in_one_instance::allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 可再分配谓词（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」）：F 不抬（恒 0）时一个已释放的落点都回不来——
/// 这条用例把 F = 0 的形态钉住：释放过的落点一个都不再发出去、defer 只增不减、空闲随分配单调减。它是 C22（刚释放的块立即重分配）的弱形态：
/// 把 `mark_released` 改成清位图（立即复用）它红；抬 F 之后的回收与复用在步 5 的用例里（`reuse_after_raising_the_floor.rs`）。
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
        29,
        "A 与 B 的 28 槽加 mkfs 树表那 1 槽都在 defer 队列里，一个都没放回（每版 14 槽：七个角色 9 槽加分配记录树五个节点）"
    );
    assert_eq!(
        device_map.allocated_slots(),
        3 + 14 + 14 + 14,
        "占着的：mkfs 3 + A 14 + B 14 + C 14"
    );
    assert_eq!(device_map.free_slots(), 211_968 - 45, "空闲只随分配减");
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
        slot_of(&third, TransactionUnit::Data(DataUnitIndexInFile::FIRST)),
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
        .find(|(unit, _)| *unit == TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .expect("码 1 一把 key")
        .1;
    let mapping_node = parse_index_node(&damaged.unit(TransactionUnit::MappingTree).bytes)
        .expect("上一版的映射节点解得开");
    let entries: Vec<Vec<u8>> = mapping_node
        .entries
        .iter()
        .map(|entry| {
            let (key, mut locations) = parse_mapping_entry(entry).expect("上一版的映射条目宽 55");
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

/// 把上一版的映射节点重装成「自述的条目宽 = key 宽 27，条目跟着截短」的形态：节点自己仍然自洽
/// （条目数 × 27 = 声明长度、两道校验和都对得上），`parse_index_node` 也只判了条目宽 ≥ key 宽。
fn previous_with_mapping_entries_narrowed_to_the_key_width(
    output: &TransactionOutput,
) -> TransactionOutput {
    let mut damaged = output.clone();
    let mapping_node = parse_index_node(&damaged.unit(TransactionUnit::MappingTree).bytes)
        .expect("上一版的映射节点解得开");
    let entries: Vec<Vec<u8>> = mapping_node
        .entries
        .iter()
        .map(|entry| entry[..mapping_node.key_width].to_vec())
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
        u16::try_from(mapping_node.key_width).expect("key 宽 27"),
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

/// 上一版的映射节点自述的条目宽缩到 key 宽 27（panic 面普查 R2 在释放判定路径上的那一处）⇒ 报
/// `MappingEntryNarrowerThanItsFieldTable`，不按字段表切到偏移 55、也不报成「不在映射」：
/// 「节点里的条目切不动」与「节点好、里面没有这把 key」是调用方要分开看的两件事。
/// 在动分配器之前报错——一条记录都没改写。
#[test]
fn release_reports_a_mapping_entry_narrower_than_its_field_table_instead_of_slicing_past_it() {
    let mut pool = build_pool("mapping-entry-narrower-than-the-field-table");
    let records_before = pool.allocator.records().to_vec();
    let narrowed = previous_with_mapping_entries_narrowed_to_the_key_width(&pool.output);
    let result = try_overwrite(&mut pool, &narrowed);
    assert!(
        matches!(
            result,
            Err(PublishError::MappingEntryNarrowerThanItsFieldTable {
                unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
                entry_bytes: 27,
                field_table_bytes: 55
            })
        ),
        "{result:?}"
    );
    assert_eq!(
        pool.allocator.records(),
        &records_before[..],
        "在动分配器之前报错：一条记录都没改写"
    );
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
                unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
                device: DeviceIdentity(0),
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
                unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
                device: DeviceIdentity(0),
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

/// 准入之后失败的发布不留半新的池：把 A 开的那个提交内生段用到头、单元区里别的空槽全标成已分配、只留 50182–50183 给 B 的数据单元。
/// B 的数据单元落得下这一对，但两个数据槽被它取走之后没挡的槽剩 0，第一个提交内生块（extent 根 + inode 叶容器 + inode 根 + ckpt_cost）
/// 落不下 ⇒ 准入先拒（C545（空间准入罩不住分裂与聚簇段层），用户 2026-09-27 定「准入先拒」，`PublishError::SpaceAdmissionRefused`），
/// 在走分配记录树的固定点、动分配器与发任何一个写之前返回；返回之后分配器要和进去之前一模一样。
#[test]
fn publish_running_out_of_space_midway_leaves_the_allocator_as_it_was() {
    let mut pool = build_pool("no-space-midway");
    let first_drained = pool
        .allocator
        .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
        .expect("A 的段里还有槽");
    assert_eq!(
        first_drained.slot,
        SlotNumber(50253),
        "A 之后 bump 游标停在 50253（A 的提交内生块 50240..=50252：分配记录树五个节点，D8（核心索引结构） 已定项 14）"
    );
    // 段 [50240, 50304) 里 50253 之后还剩 50 槽。
    for _drain_round in 0..50 {
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
    let Err(PublishError::SpaceAdmissionRefused(refusal)) = &result else {
        panic!("数据单元拿到了 50182，第一个提交内生块（extent 根 + inode 叶容器 + inode 根 + ckpt_cost）落不下：{result:?}");
    };
    assert_eq!(
        refusal.short_devices,
        [DeviceIdentity(0), DeviceIdentity(1)]
            .map(|device| DeviceShortOfDemand {
                device,
                available: AvailableBytesOnOneDevice(0),
                demand: BytesOnOneDevice(8_962_048),
            })
            .to_vec(),
        "每块盘两个数据槽被数据单元取走之后没挡的槽剩 0（0 字节）< 第一个提交内生块要的 547 槽（8 962 048 字节；ckpt_cost 含谱系、退役那两项各 3 块）"
    );
    assert_eq!(
        pool.allocator.records(),
        &records_before[..],
        "失败的发布退回：A 的记录没改写成已释放、B 的数据单元没留下记录"
    );
    let device_map = &pool.allocator.devices[0];
    assert_eq!(
        device_map.deferred_slots(),
        1,
        "A 的落点没进 defer 队列；队列里只有 A 换下的 mkfs 树表那 1 槽"
    );
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

/// 分配器里失败的发布退回之后必须不变的那些量（同 `unequal_devices.rs` 的 `AllocatorFingerprint`）。
#[derive(Debug, PartialEq, Eq)]
struct AllocatorFingerprint {
    records: Vec<AllocationRecord>,
    open_segment: Option<SlotNumber>,
    counts_per_device: Vec<DeviceCountsInTheFingerprint>,
}

#[derive(Debug, PartialEq, Eq)]
struct DeviceCountsInTheFingerprint {
    allocated_slots: u64,
    free_slots: u64,
    deferred_slots: u64,
    isolated_slots: u64,
    free_runs: u64,
    empty_segments: u64,
}

fn allocator_fingerprint(allocator: &PoolAllocator) -> AllocatorFingerprint {
    AllocatorFingerprint {
        records: allocator.records().to_vec(),
        open_segment: allocator.open_segment(),
        counts_per_device: allocator
            .devices
            .iter()
            .map(|device_map| DeviceCountsInTheFingerprint {
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

/// 改坏一块盘上某个槽起的那一份单元（同 `release_checksum_quarantine.rs` 的 `corrupt_the_copy_on`）：翻它第 4096 字节起那个扇区的
/// 第一个字节，位置项里的校验和从此对不上。
fn corrupt_the_copy_on(pool: &mut BuiltPool, device: DeviceIdentity, slot: SlotNumber) {
    let (_, block_device) = pool
        .devices
        .as_mut()
        .expect("镜像还开着")
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    let sector_offset = DeviceOffsetInBytes(slot.to_device_offset().0 + 4096);
    let mut sector = vec![0u8; 512];
    block_device
        .read_at(sector_offset, &mut sector)
        .expect("读那一个扇区");
    sector[0] ^= 0xff;
    block_device
        .write_at(sector_offset, &sector, WriteDurability::Plain)
        .expect("写回改坏的扇区");
}

/// C576（三条用例被准入先拒连带红，改法待选） 第 12 行变异「失败的发布不退回分配器」的取样点：落点在分配器的拷贝上走得通、
/// 真发时才被拒的一段历史。取落点先在拷贝上照发布路径的次序走一遍（`settle_the_allocation_record_tree`），拷贝上的释放不知道
/// 读盘核出对不上的那一份要隔离、照常释放；真发时那一份两盘的分配记录都留在已分配（D19（块指针的结构与宽度预算） 已定项 5）。
/// 复用窗口置 0（只供测试的开关）让被换下的落点在释放时就回到空闲池，两边就分了叉：拷贝上 B 的数据单元落回 A 的 50180，
/// 真发时 50180 还占着、单元区里段外再没有空的槽对 ⇒ `publish_admitted` 里取数据单元的落点被拒（`PlacementRefused`），
/// 这时真分配器上已经做完了这次的释放与回收。准入两判都关掉（只供测试的开关），不然落得下那一判在走拷贝之前就按段外槽对数拒。
/// 返回之后分配器要和进去之前一模一样。
#[test]
fn publish_refused_only_after_the_real_allocator_released_and_reclaimed_leaves_the_allocator_as_it_was(
) {
    let mut pool = build_pool("refused-after-the-real-release");
    let previous = pool.output.clone();
    let data_slot_of_the_previous_version = previous.data_pointers[0].locations[0].slot;
    assert_eq!(
        data_slot_of_the_previous_version,
        SlotNumber(50180),
        "A 的数据单元落在 50180–50181（字节表五）"
    );
    corrupt_the_copy_on(
        &mut pool,
        DeviceIdentity(0),
        data_slot_of_the_previous_version,
    );
    pool.allocator
        .set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
    pool.allocator.set_reuse_window(ReuseWindow::ForcedToZero);
    let open_segment = pool
        .allocator
        .open_segment()
        .expect("A 开的提交内生段还开着");
    let open_segment_slots = open_segment.0..open_segment.0 + CLUSTER_SEGMENT_SLOTS;
    let unit_area_end = IMAGE_BYTES / SLOT_BYTES;
    // 开放段之外的空槽全标成已分配：用户数据在段外只剩 A 那一对会被释放的槽，提交内生块照旧从开放段 bump。
    for device_map in &mut pool.allocator.devices {
        for slot in UNIT_AREA_START_SLOT..unit_area_end {
            if open_segment_slots.contains(&slot) {
                continue;
            }
            if device_map.is_free(SlotNumber(slot)) {
                device_map.mark_allocated(SlotNumber(slot), 1);
            }
        }
    }
    let before = allocator_fingerprint(&pool.allocator);

    let result = try_overwrite(&mut pool, &previous);

    assert!(
        matches!(
            result,
            Err(PublishError::PlacementRefused {
                unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
                refusal: PlacementRefusal::NoFreeSlotOnAnyDevice,
            })
        ),
        "真发时 50180 隔离着、段外没有空的槽对，B 的数据单元取不到落点：{result:?}"
    );
    assert_eq!(
        allocator_fingerprint(&pool.allocator),
        before,
        "失败的发布退回分配器：被换下的落点没改写成已释放、没回收，开放段与各盘的计数照旧"
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

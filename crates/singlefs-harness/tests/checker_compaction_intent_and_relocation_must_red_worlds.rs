//! 池级 checker 为自建 GC 新判的几条（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1、G3、G9；正推
//! `research/prompts/m4-gc-r1-sonnet-output.md`「checker 多判哪几条」那张表的 N2–N6）各配合法世界与必红世界：
//! I-3.15（N2 反查：未释放记录罩住的单元由映射指在这一槽）、I-5.6（N3：活整理意图的区域里没有签发之后分配的记录）、
//! I-4.4（N4：意图只描述目标态，判 logged_ops 条目）、I-3.16（N5 = C413：碎片度与全空段统计对得上）、
//! I-1.14（N6：同一映射 key 在各版里的单元逐字节相同）。
//!
//! 搬迁用发布入口 `transaction::publish_relocation_batch` 写（合法那一份与「只写副本不改映射」那一份走只供测试的开关）；
//! 整理意图今天还不上盘（写路径归核心层），logged_ops 树按字节造进最新那一版：新池新建文件之后，在最后一个在册单元的下一槽写一个
//! logged_ops 根兼叶（树 ID 取树表里种类 12 那一条的、出生在最新那一版、出生序号 0），分配记录两盘各记一条（分配代 = 诞生代号）、
//! 中央映射加一条它的 key、树表种类 12 那一条的根指针指它、记账的已分配与空闲跟着改；改过的单元重封，新的整单元校验和沿引用链补到根槽
//! （`common_pointer_head_corruption::replace_the_unit_and_propagate_its_checksum`）。
mod common;
mod common_images_for_the_checker_tier;
mod common_pointer_head_corruption;
#[path = "common_images_for_the_checker_tier/release_checksum_quarantine.rs"]
mod scenarios_of_release_checksum_quarantine;

use common::{
    build_pool, file_content, overwrite_in_process_as_instance_one_and_advance_the_pool,
    parameters, BuiltPool,
};
use common_images_for_the_checker_tier::PoolCheckerImages;
use common_pointer_head_corruption::{
    allocation_record_tree_nodes_as_far_as_readable, newest_root, node_bytes, node_pointer_bytes,
    replace_the_unit_and_propagate_its_checksum, the_slot_of, tree_table_entry_of_kind,
    unit_bytes_at,
};
use singlefs_checker::image::{
    InvariantVerdict, COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
    FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
    UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{
    CheckpointTxg, DataUnitIndexInFile, DeviceIdentity, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{Placement, ALLOCATION_RECORD_RELEASED_FLAG};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::compaction_plan::{
    CompactionSwitches, PlannedRelocation, RelocationPublishOrder,
};
use singlefs_core::logged_operations::{
    CompactionIntent, CompactionRegion, LoggedOperation, LoggedOperationEntry,
};
use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
use singlefs_core::records::{
    build_mapping_entry, mapping_key_for_node, TreeTableEntry, TREE_KIND_ACCOUNTING,
    TREE_KIND_ALLOCATION, TREE_KIND_LOGGED_OPERATIONS,
};
use singlefs_core::transaction::{
    central_mapping_locations_in_the_version, publish_relocation_batch, MappingLookup, PoolWriter,
    RelocationBatchRefused, TransactionOutput, TransactionUnit,
};
use singlefs_core::unit::{
    build_index_node, parse_index_node, seal_header_checksum, IndexNodeHeader,
    UNIT_CLASS_INDEX_NODE,
};
use singlefs_format::{
    CLUSTER_SEGMENT_SLOTS, LOGGED_OPERATIONS_ENTRY_BYTES, LOGGED_OPERATIONS_KEY_BYTES,
    MAPPING_KEY_BYTES, SLOT_BYTES, UNIT_AREA_START_SLOT,
};
use singlefs_harness::memory_pool::MemoryPool;

const INSTANCE: InstanceGeneration = InstanceGeneration(1);
const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];

/// 分配记录 20 字节：设备身份 4、起点槽 6、跨度 2（最高位是已释放标志）、分配代或释放代 8（D3（空间分配） 已定项 7）。
const ALLOCATION_RECORD_BYTES: usize = 20;
const ALLOCATION_RECORD_SPAN_OFFSET: usize = 10;
const ALLOCATION_RECORD_GENERATION_OFFSET: usize = 12;
/// 记账条目 34 字节：key 22（统计量 2 + 树 8 + 设备 4 + 代 8）之后是值 8（D8（核心索引结构） 已定项 11）。
const ACCOUNTING_ROW_DEVICE_OFFSET: usize = 2 + 8;
const ACCOUNTING_ROW_VALUE_OFFSET: usize = 22;
/// 记账统计量的行号（D5（快照 / 空间记账机制） 已定项 4）：已分配 1、空闲 2（字节），runs 10、全空聚簇段数 11（个），都带设备维。
const STATISTIC_ALLOCATED_BYTES: u16 = 1;
const STATISTIC_FREE_BYTES: u16 = 2;
const STATISTIC_FRAGMENTATION_RUNS: u16 = 10;
const STATISTIC_EMPTY_CLUSTER_SEGMENTS: u16 = 11;
/// 中央映射叶条目 55 字节：key 27 之后两条位置条目各 14（设备身份 4、槽号 6、整单元校验和 4，D19（块指针的结构与宽度预算） 已定项 10）。
const MAPPING_LOCATION_ENTRY_BYTES: usize = 14;
/// 整理意图条目 44 字节里的几段（判决 G1：key 种类 2 + 签发 txg 8；value R 的设备 4、起始槽 6、槽数 2、`c0` 8、`segs0` 8、补零 6）。
const COMPACTION_INTENT_REGION_START_SLOT_OFFSET: usize = 14;
const COMPACTION_INTENT_PADDING_OFFSET: usize = 38;
/// 码 1 数据单元：明文头 [0, 105)，载荷校验和住 101（D18（块里携带什么信息） 已定项 7 的字段表）。
const DATA_UNIT_HEADER_END: usize = 105;
const DATA_UNIT_PAYLOAD_CHECKSUM_OFFSET: usize = 101;
/// 第一个文件版本的单元区只用到 50 26x 附近：这一槽在两块盘上都没被分配、没被引用。
const SLOT_NOBODY_ALLOCATED: u64 = 50_400;

fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
}

fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
}

fn read_u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn read_six_bytes_at(bytes: &[u8], offset: usize) -> u64 {
    let mut wide = [0u8; 8];
    wide[..6].copy_from_slice(&bytes[offset..offset + 6]);
    u64::from_le_bytes(wide)
}

fn mapping_key_bytes() -> usize {
    usize::try_from(MAPPING_KEY_BYTES).expect("27")
}

/// 码 2 节点改过之后重封：载荷 CRC（罩 [86 + 2k, 16384)，住 76 + 2k）与头校验和（罩 [0, 86 + 2k)），k 是住偏移 51 的 key 宽。
fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 码 1 数据单元改过载荷之后重封：载荷 CRC 与头校验和。
fn reseal_data_unit(unit: &mut [u8]) {
    let payload_checksum = crc32_castagnoli(&unit[DATA_UNIT_HEADER_END..]);
    unit[DATA_UNIT_PAYLOAD_CHECKSUM_OFFSET..DATA_UNIT_PAYLOAD_CHECKSUM_OFFSET + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(unit, DATA_UNIT_HEADER_END);
}

/// 一个码 2 节点换成同一个头、`entries` 这些条目，key 区间取 `smallest_key` / `largest_key`，重封。
fn rebuilt_with_entries(
    header: &IndexNodeHeader,
    smallest_key: &[u8],
    largest_key: &[u8],
    entries: &[Vec<u8>],
) -> Vec<u8> {
    let mut rebuilt = build_index_node(
        header.tree,
        header.level,
        header.key_width,
        smallest_key,
        largest_key,
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽装得进 u16"),
        entries,
    );
    reseal_index_node(&mut rebuilt);
    rebuilt
}

fn violated(verdicts: &[(&'static str, InvariantVerdict)]) -> Vec<(&'static str, String)> {
    verdicts
        .iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => Some((*invariant, detail.clone())),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

fn violated_names(violations: &[(&'static str, String)]) -> Vec<&'static str> {
    violations.iter().map(|(invariant, _)| *invariant).collect()
}

fn verdict_among(
    verdicts: &[(&'static str, InvariantVerdict)],
    invariant: &str,
) -> InvariantVerdict {
    verdicts
        .iter()
        .find(|(candidate, _)| *candidate == invariant)
        .map(|(_, verdict)| verdict.clone())
        .expect("清单里有这一条")
}

fn detail_of<'violations>(
    violations: &'violations [(&'static str, String)],
    invariant: &str,
) -> &'violations str {
    violations
        .iter()
        .find(|(candidate, _)| *candidate == invariant)
        .map(|(_, detail)| detail.as_str())
        .unwrap_or_else(|| panic!("{invariant} 要红：{violations:?}"))
}

// ─── 搬迁：一批照判决 G3 发出去 ───

/// `version` 里 `identity` 这个角色的映射 key 与它的源落点（按映射位置）：一批搬它的计划项。
fn planned_relocation_of(
    version: &TransactionOutput,
    identity: TransactionUnit,
) -> PlannedRelocation {
    let (_, mapping_key) = version
        .mapped_units
        .iter()
        .find(|(role, _)| *role == identity)
        .expect("这个角色进映射");
    PlannedRelocation {
        mapping_key: mapping_key.clone(),
        source: Placement {
            slot: mapping_locations_of(version, mapping_key)[0].slot,
            span: identity.span_slots(),
        },
    }
}

fn mapping_locations_of(version: &TransactionOutput, mapping_key: &[u8]) -> [LocationEntry; 2] {
    match central_mapping_locations_in_the_version(version, mapping_key) {
        MappingLookup::Found(locations) => locations,
        MappingLookup::NodeMalformedOrNoEntryWithThisKey
        | MappingLookup::EntryNarrowerThanItsFieldTable { .. } => {
            panic!("这把 key 在这一版的映射里")
        }
    }
}

/// 同一个进程里接着 `pool.output` 发一批，池接到它那一版上。
fn relocate_and_advance(
    pool: &mut BuiltPool,
    planned: &[PlannedRelocation],
) -> Result<TransactionOutput, RelocationBatchRefused> {
    let publish_parameters = parameters();
    let previous = pool.output.clone();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    let relocated = publish_relocation_batch(
        &mut writer,
        &mut pool.allocator,
        &previous,
        planned,
        INSTANCE,
    )?;
    pool.output = relocated.clone();
    Ok(relocated)
}

/// 新池新建文件之后照判决 G3 搬第一个文件的数据单元一批：交回池、计划项与搬到哪一槽。
fn pool_after_relocating_the_first_data_unit(
    tag: &str,
    order: RelocationPublishOrder,
) -> (BuiltPool, PlannedRelocation, SlotNumber) {
    let mut pool = build_pool(tag);
    let product = CompactionSwitches::PRODUCT;
    pool.allocator.set_compaction_switches(CompactionSwitches {
        batch_size: product.batch_size,
        forced_region: product.forced_region,
        run_forcing: product.run_forcing,
        region_exclusion: product.region_exclusion,
        destination_policy: product.destination_policy,
        release_checksum_mismatch_after_issue: product.release_checksum_mismatch_after_issue,
        empty_cluster_segments_floor: product.empty_cluster_segments_floor,
        relocation_publish_order: order,
    });
    let planned = planned_relocation_of(
        &pool.output.clone(),
        TransactionUnit::Data(DataUnitIndexInFile::FIRST),
    );
    let batch =
        relocate_and_advance(&mut pool, std::slice::from_ref(&planned)).expect("一批发得出去");
    let [relocated] = batch.units_relocated_by_this_publish.as_slice() else {
        panic!("这一批只搬了一个单元")
    };
    let destination = relocated.destination[0].slot;
    assert_ne!(destination, planned.source.slot, "搬到了别的槽");
    (pool, planned, destination)
}

/// 合法世界：照判决 G3 搬一批之后，反查、逐字节相同、runs 与全空段三条都真被评估过且成立，一条违例都没有。
/// I-1.14 比上的是数据单元那把 key：上一版落在源槽、这一批落在目的槽，两份逐字节相同（搬迁原样拷、头不盖戳）。
#[test]
fn a_relocation_batch_through_the_mapping_holds_the_reverse_lookup_the_same_bytes_and_the_full_scan_statistics(
) {
    let (pool, _, _) = pool_after_relocating_the_first_data_unit(
        "compaction-checker-legal-batch",
        RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping,
    );
    let verdicts = check_pool_image(&pool.memory_pool());
    assert_eq!(violated(&verdicts), [], "合法的一批之后一条违例都没有");
    for invariant in [
        UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
        UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
        FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
    ] {
        assert_eq!(
            verdict_among(&verdicts, invariant),
            InvariantVerdict::Holds,
            "{invariant} 在合法的一批之后真被评估过且成立"
        );
    }
}

// ─── I-3.15（N2 反查）：三形各一份必红世界 ───

/// 必红世界「只改映射不写副本」：新池新建文件之后，数据单元那条映射条目的两条位置条目改指一个没人分配、也没写副本的槽，分配记录一条不动。
/// 源槽那条未释放的记录罩住的数据单元，在映射里那把相符的 key 落在别处：I-3.15 红、点名源槽（映射那一遍读不出目的槽，I-2.1、I-7.2 也红）。
#[test]
fn a_mapping_entry_moved_without_a_copy_reddens_the_reverse_lookup_at_the_source() {
    let pool = build_pool("compaction-checker-mapping-moved-without-a-copy");
    let mut image = pool.memory_pool();
    let source = pool
        .output
        .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
        .slot;
    let mapping_slot = the_slot_of(&newest_root(&image).mapping_root);
    let mapping = parse_index_node(&unit_bytes_at(&image, mapping_slot, node_bytes()))
        .expect("中央映射根解得开");
    assert_eq!(mapping.level, 0, "新池新建文件那一版的映射树是根兼叶");
    let mut entries = mapping.entries.clone();
    let entry = entries
        .iter_mut()
        .find(|entry| read_six_bytes_at(entry, mapping_key_bytes() + 4) == source.0)
        .expect("映射里有数据单元那一条");
    for location in 0..2 {
        let slot_at = mapping_key_bytes() + location * MAPPING_LOCATION_ENTRY_BYTES + 4;
        entry[slot_at..slot_at + 6].copy_from_slice(&SLOT_NOBODY_ALLOCATED.to_le_bytes()[..6]);
    }
    let rebuilt = rebuilt_with_entries(
        &mapping,
        &mapping.smallest_key,
        &mapping.largest_key,
        &entries,
    );
    replace_the_unit_and_propagate_its_checksum(&mut image, mapping_slot, &rebuilt);
    let red = violated(&check_pool_image(&image));
    let detail = detail_of(
        &red,
        UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
    );
    assert!(
        detail.contains(&format!("槽 {} 的分配记录", source.0))
            && detail.contains(&format!("{SLOT_NOBODY_ALLOCATED})")),
        "反查点名源槽 {} 的记录、相符的 key 落在 {SLOT_NOBODY_ALLOCATED}：{detail}",
        source.0
    );
}

/// 必红世界「只写副本不改映射」：只供测试的开关让一批照样写目的地与它的分配记录、释放源落点，映射条目与各处指针却留在源槽
/// （`RelocationPublishOrder::SourceReleasedWhileTheMappingStillNamesItForTests`）。目的槽那条未释放的记录罩住的副本，映射里那把 key 指着源槽：
/// I-3.15 红、点名目的槽。
#[test]
fn a_copy_written_while_the_mapping_still_names_the_source_reddens_the_reverse_lookup_at_the_destination(
) {
    let (pool, planned, destination) = pool_after_relocating_the_first_data_unit(
        "compaction-checker-copy-without-the-mapping",
        RelocationPublishOrder::SourceReleasedWhileTheMappingStillNamesItForTests,
    );
    assert_eq!(
        mapping_locations_of(&pool.output, &planned.mapping_key)[0].slot,
        planned.source.slot,
        "映射条目留在源槽"
    );
    let red = violated(&check_pool_image(&pool.memory_pool()));
    let detail = detail_of(
        &red,
        UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
    );
    assert!(
        detail.contains(&format!("槽 {} 的分配记录", destination.0)),
        "反查点名目的槽 {}：{detail}",
        destination.0
    );
}

/// 必红世界「有记录没人引用」（判决 G9 的「已死」类）：覆盖写之后旧的那个数据单元已释放，把它在最新那一版的两条记录改回未释放
/// （分配代取它的诞生代号，I-3.10 照旧成立）。最新那一版的映射里没有一把 key 与它相符：I-3.15 红、说明里写「与它相符的 key 落在：没有」
/// （已分配统计跟着对不上，I-3.11 也红）。
#[test]
fn an_unreleased_record_nobody_references_reddens_the_reverse_lookup() {
    let mut pool = build_pool("compaction-checker-dead-record");
    let first_version = pool.output.clone();
    let old_data_unit = first_version.unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST));
    let old_slot = old_data_unit.slot;
    let old_birth_txg = first_version.root.checkpoint_txg;
    let mut overwritten_content = file_content();
    overwritten_content[0] ^= 0xff;
    let overwrite =
        overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &overwritten_content);
    assert_ne!(
        overwrite
            .unit(TransactionUnit::Data(DataUnitIndexInFile::FIRST))
            .slot,
        old_slot,
        "覆盖写把数据单元写到了别的槽"
    );
    let mut image = pool.memory_pool();
    for device in BOTH_DEVICES {
        let (leaf_slot, leaf) = allocation_leaf_holding(&image, device, old_slot);
        let header = parse_index_node(&leaf).expect("分配记录叶解得开");
        let mut records = header.entries.clone();
        let record = records
            .iter_mut()
            .find(|record| {
                read_u32_at(record, 0) == device.0 && read_six_bytes_at(record, 4) == old_slot.0
            })
            .expect("旧数据单元的记录在这片叶里");
        let span = read_u16_at(record, ALLOCATION_RECORD_SPAN_OFFSET);
        assert_ne!(
            span & ALLOCATION_RECORD_RELEASED_FLAG,
            0,
            "覆盖写之后旧数据单元的记录已释放"
        );
        record[ALLOCATION_RECORD_SPAN_OFFSET..ALLOCATION_RECORD_SPAN_OFFSET + 2]
            .copy_from_slice(&(span & !ALLOCATION_RECORD_RELEASED_FLAG).to_le_bytes());
        record[ALLOCATION_RECORD_GENERATION_OFFSET..ALLOCATION_RECORD_GENERATION_OFFSET + 8]
            .copy_from_slice(&old_birth_txg.0.to_le_bytes());
        let rebuilt =
            rebuilt_with_entries(&header, &header.smallest_key, &header.largest_key, &records);
        replace_the_unit_and_propagate_its_checksum(&mut image, leaf_slot, &rebuilt);
    }
    let red = violated(&check_pool_image(&image));
    let detail = detail_of(
        &red,
        UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
    );
    assert!(
        detail.contains(&format!("槽 {} 的分配记录", old_slot.0))
            && detail.contains("与它相符的 key 落在：没有"),
        "反查点名旧槽 {}、映射里没有一把相符的 key：{detail}",
        old_slot.0
    );
}

/// 合法世界「写者隔离了一份头用不了的单元」（判决 `research/prompts/m4-i315-r1-main-verification.md` 第三节 C1「乙二」，调查员那一形）：
/// 镜像怎么搭、实现的行为核什么见 `scenarios_of_release_checksum_quarantine::a_quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup`
/// （两档共用那一份：checker 档的同名用例与 GPU 与 CPU 对拍收同一张镜像）；这里收下它交的那一张就地判。
/// 组只取头用得了的那一份（改之前的读法）时盘 0 那一份对得上位置项，I-3.15 红。
#[test]
fn a_quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup() {
    let mut handed_out = PoolCheckerImages::collected_for_the_checker_tier();
    scenarios_of_release_checksum_quarantine::a_quarantined_unit_whose_copy_on_one_device_has_an_unusable_header_holds_the_reverse_lookup(&mut handed_out);
    let collected = handed_out.into_collected();
    let [image] = collected.as_slice() else {
        panic!("场景交一张镜像：交了 {} 张", collected.len());
    };
    let verdicts = check_pool_image(&image.image);
    for invariant in [
        UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
        "I-3.1",
        "I-3.11",
    ] {
        assert_eq!(
            verdict_among(&verdicts, invariant),
            InvariantVerdict::Holds,
            "{invariant}：隔离的单元（盘 1 那一份头用不了）不判红"
        );
    }
}

// ─── I-1.14（N6）：搬迁时改一个字节 ───

/// 必红世界（判决 G7「执行层靠 N6」那一个必出反例）：合法的一批之后，把目的槽那份副本的载荷改一个字节、重封，映射条目的校验和跟着补
/// （副本对得上它自己的映射条目，I-2.1 不在那里红）。数据单元那把 key 在上一版指源槽那一份、在这一批指目的槽那一份，两份字节不同：I-1.14 红。
#[test]
fn a_byte_changed_in_the_relocated_copy_reddens_the_same_bytes_invariant() {
    let (pool, _, destination) = pool_after_relocating_the_first_data_unit(
        "compaction-checker-relocated-copy-changed",
        RelocationPublishOrder::SourceReleasedInThePublishThatMovesTheMapping,
    );
    let mut image = pool.memory_pool();
    let data_unit_length = usize::try_from(2 * SLOT_BYTES).expect("32768");
    let mut changed = unit_bytes_at(&image, destination, data_unit_length);
    changed[DATA_UNIT_HEADER_END + 100] ^= 0xff;
    reseal_data_unit(&mut changed);
    replace_the_unit_and_propagate_its_checksum(&mut image, destination, &changed);
    let red = violated(&check_pool_image(&image));
    let detail = detail_of(
        &red,
        UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS,
    );
    assert!(
        detail.contains("字节不同") && detail.contains(&format!(", {}, ", destination.0)),
        "I-1.14 点名目的槽 {} 那一份：{detail}",
        destination.0
    );
}

// ─── I-3.16（N5 = C413）：增量少加一次 ───

/// 最新那一版记账树（根兼叶）里 `statistic` 在盘 0 那一行的值减一（增量维护少加了一次），重封、补校验和。
fn image_with_a_device_zero_row_one_short(tag: &str, statistic: u16) -> MemoryPool {
    let pool = build_pool(tag);
    let mut image = pool.memory_pool();
    let accounting_slot = the_slot_of(&tree_table_entry_of_kind(&image, TREE_KIND_ACCOUNTING).root);
    let accounting = parse_index_node(&unit_bytes_at(&image, accounting_slot, node_bytes()))
        .expect("记账根解得开");
    assert_eq!(accounting.level, 0, "新池新建文件那一版的记账树是根兼叶");
    let mut rows = accounting.entries.clone();
    let row = rows
        .iter_mut()
        .find(|row| {
            read_u16_at(row, 0) == statistic
                && read_u32_at(row, ACCOUNTING_ROW_DEVICE_OFFSET) == BOTH_DEVICES[0].0
        })
        .expect("记账里有盘 0 这一行");
    let one_short = read_u64_at(row, ACCOUNTING_ROW_VALUE_OFFSET) - 1;
    row[ACCOUNTING_ROW_VALUE_OFFSET..ACCOUNTING_ROW_VALUE_OFFSET + 8]
        .copy_from_slice(&one_short.to_le_bytes());
    let rebuilt = rebuilt_with_entries(
        &accounting,
        &accounting.smallest_key,
        &accounting.largest_key,
        &rows,
    );
    replace_the_unit_and_propagate_its_checksum(&mut image, accounting_slot, &rebuilt);
    image
}

/// 必红世界（C413 的判别力自证「增量少加一次」）：盘 0 的 runs 那一行少一。全扫重算的 runs 与它对不上：只红 I-3.16。
#[test]
fn a_runs_row_one_short_reddens_only_the_full_scan_statistics() {
    let image = image_with_a_device_zero_row_one_short(
        "compaction-checker-runs-one-short",
        STATISTIC_FRAGMENTATION_RUNS,
    );
    let red = violated(&check_pool_image(&image));
    assert_eq!(
        violated_names(&red),
        [FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN],
        "只红 I-3.16：{red:?}"
    );
    assert!(
        red[0].1.contains("盘 0：记账的 runs"),
        "说明点名盘 0 的 runs：{}",
        red[0].1
    );
}

/// 必红世界：盘 0 的全空聚簇段数那一行少一。新池新建文件那一版上两种口径算出的全空段数相同（没有只剩 defer 槽的段），这一半判得了：只红 I-3.16。
#[test]
fn an_empty_segment_row_one_short_reddens_only_the_full_scan_statistics() {
    let image = image_with_a_device_zero_row_one_short(
        "compaction-checker-empty-segments-one-short",
        STATISTIC_EMPTY_CLUSTER_SEGMENTS,
    );
    let red = violated(&check_pool_image(&image));
    assert_eq!(
        violated_names(&red),
        [FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN],
        "只红 I-3.16：{red:?}"
    );
    assert!(
        red[0].1.contains("盘 0：记账的全空聚簇段数"),
        "说明点名盘 0 的全空聚簇段数：{}",
        red[0].1
    );
}

// ─── I-4.4（N4）与 I-5.6（N3）：logged_ops 树按字节造进最新那一版 ───

/// 最新那一版分配记录树里装着盘 `device` 的记录、按位置罩住 `slot` 的那片叶（槽号、叶的字节）：叶里有一条盘 `device` 的记录起点 ≤ `slot`，
/// 叶头的最大 key 罩到 `slot`。
fn allocation_leaf_holding(
    image: &MemoryPool,
    device: DeviceIdentity,
    slot: SlotNumber,
) -> (SlotNumber, Vec<u8>) {
    let root = tree_table_entry_of_kind(image, TREE_KIND_ALLOCATION).root;
    allocation_record_tree_nodes_as_far_as_readable(image, &root)
        .into_iter()
        .map(|pointer| {
            let node_slot = the_slot_of(&pointer);
            (node_slot, unit_bytes_at(image, node_slot, node_bytes()))
        })
        .find(|(_, node)| {
            let header = parse_index_node(node).expect("分配记录树节点解得开");
            let largest = (
                read_u32_at(&header.largest_key, 0),
                read_six_bytes_at(&header.largest_key, 4),
            );
            header.level == 0
                && header.entries.iter().any(|record| {
                    read_u32_at(record, 0) == device.0 && read_six_bytes_at(record, 4) <= slot.0
                })
                && (device.0, slot.0) <= largest
        })
        .expect("最新那一版的分配记录树里有一片叶罩住这一槽")
}

/// 最新那一版分配记录里盘 0 上最后一个在册单元（起点槽最大的那条记录）之后的第一槽，与那条记录未释放与否。
fn slot_after_the_last_recorded_unit(image: &MemoryPool) -> (SlotNumber, bool) {
    let root = tree_table_entry_of_kind(image, TREE_KIND_ALLOCATION).root;
    allocation_record_tree_nodes_as_far_as_readable(image, &root)
        .into_iter()
        .filter_map(|pointer| {
            parse_index_node(&unit_bytes_at(image, the_slot_of(&pointer), node_bytes())).ok()
        })
        .filter(|header| header.level == 0)
        .flat_map(|header| header.entries)
        .filter(|record| read_u32_at(record, 0) == BOTH_DEVICES[0].0)
        .map(|record| {
            let span = read_u16_at(&record, ALLOCATION_RECORD_SPAN_OFFSET);
            (
                SlotNumber(
                    read_six_bytes_at(&record, 4)
                        + u64::from(span & !ALLOCATION_RECORD_RELEASED_FLAG),
                ),
                span & ALLOCATION_RECORD_RELEASED_FLAG == 0,
            )
        })
        .max_by_key(|(end, _)| *end)
        .expect("最新那一版有在册的单元")
}

/// logged_ops 条目 44：照核心层的字段表写一条整理意图（`LoggedOperationEntry::to_bytes`），R 是盘 0 上 `region_start` 起的那一个聚簇段。
fn compaction_intent_entry(
    issued_txg: CheckpointTxg,
    region_start: SlotNumber,
    allocation_generation_at_issue: CheckpointTxg,
) -> Vec<u8> {
    let entry = LoggedOperationEntry {
        issued_txg,
        operation: LoggedOperation::Compaction(CompactionIntent {
            region: CompactionRegion::of_one_cluster_segment(BOTH_DEVICES[0], region_start)
                .expect("R 是一个对齐的聚簇段"),
            allocation_generation_at_issue,
            empty_cluster_segments_at_issue: 0,
        }),
    }
    .to_bytes();
    assert_eq!(
        entry.len(),
        usize::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44")
    );
    entry
}

/// 新池新建文件之后，把 logged_ops 树按字节造进最新那一版（文件头那一段），根兼叶装 `entries`（按 key 升序给）。交回镜像与最新那一版的 txg。
fn image_with_logged_operations_entries_in_the_newest_version(
    tag: &str,
    entries_for_the_newest_version: impl FnOnce(CheckpointTxg) -> Vec<Vec<u8>>,
) -> MemoryPool {
    let pool = build_pool(tag);
    let mut image = pool.memory_pool();
    let root = newest_root(&image);
    let newest_txg = root.checkpoint_txg;
    let entries = entries_for_the_newest_version(newest_txg);
    let logged_operations_entry = tree_table_entry_of_kind(&image, TREE_KIND_LOGGED_OPERATIONS);
    assert_eq!(
        logged_operations_entry.root,
        NodePointer::empty_root(),
        "新池新建文件那一版 logged_ops 树是零根"
    );
    // logged_ops 根兼叶落在最后一个在册单元的下一槽：左边那一槽是未释放记录罩着的、右边一段空闲，runs 不变。
    let (node_slot, the_last_record_is_unreleased) = slot_after_the_last_recorded_unit(&image);
    assert!(
        the_last_record_is_unreleased,
        "最后一个在册单元未释放：新节点左边那一槽占着"
    );
    let key_bytes = usize::try_from(LOGGED_OPERATIONS_KEY_BYTES).expect("10");
    let mut node = build_index_node(
        logged_operations_entry.tree,
        0,
        key_bytes,
        &entries.first().expect("至少一条条目")[..key_bytes],
        &entries.last().expect("至少一条条目")[..key_bytes],
        newest_txg,
        &parameters().filesystem_identifier,
        root.instance,
        BirthSequence(0),
        u16::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44"),
        &entries,
    );
    reseal_index_node(&mut node);
    for device in BOTH_DEVICES {
        image
            .devices
            .get_mut(&device)
            .expect("池里有这块盘")
            .write(node_slot.to_device_offset(), &node);
    }
    let node_checksum = crc32_castagnoli(&node);
    let node_pointer = NodePointer {
        head: PointerHead {
            birth_tree: logged_operations_entry.tree,
            birth_txg: newest_txg,
        },
        locations: BOTH_DEVICES.map(|device| LocationEntry {
            device,
            slot: node_slot,
            unit_checksum: node_checksum,
        }),
        instance: root.instance,
        birth_sequence: BirthSequence(0),
    };
    // 分配记录：两盘各一条未释放、跨 1 槽、分配代 = 诞生代号（I-3.10）。
    for device in BOTH_DEVICES {
        let (leaf_slot, leaf) = allocation_leaf_holding(&image, device, node_slot);
        let header = parse_index_node(&leaf).expect("分配记录叶解得开");
        let mut records = header.entries.clone();
        let mut record_of_the_node = vec![0u8; ALLOCATION_RECORD_BYTES];
        record_of_the_node[..4].copy_from_slice(&device.0.to_le_bytes());
        record_of_the_node[4..10].copy_from_slice(&node_slot.0.to_le_bytes()[..6]);
        record_of_the_node[ALLOCATION_RECORD_SPAN_OFFSET..ALLOCATION_RECORD_SPAN_OFFSET + 2]
            .copy_from_slice(&1u16.to_le_bytes());
        record_of_the_node
            [ALLOCATION_RECORD_GENERATION_OFFSET..ALLOCATION_RECORD_GENERATION_OFFSET + 8]
            .copy_from_slice(&newest_txg.0.to_le_bytes());
        records.push(record_of_the_node);
        records.sort_by_key(|record| (read_u32_at(record, 0), read_six_bytes_at(record, 4)));
        let rebuilt =
            rebuilt_with_entries(&header, &header.smallest_key, &header.largest_key, &records);
        replace_the_unit_and_propagate_its_checksum(&mut image, leaf_slot, &rebuilt);
    }
    // 中央映射：加一条 logged_ops 根的 key，按 key 升序放好，叶头的区间贴紧首末条目。
    let mapping_slot = the_slot_of(&newest_root(&image).mapping_root);
    let mapping = parse_index_node(&unit_bytes_at(&image, mapping_slot, node_bytes()))
        .expect("中央映射根解得开");
    assert_eq!(mapping.level, 0, "新池新建文件那一版的映射树是根兼叶");
    let mut mapping_entries = mapping.entries.clone();
    mapping_entries.push(build_mapping_entry(
        &mapping_key_for_node(UNIT_CLASS_INDEX_NODE, node_pointer),
        node_pointer.locations,
    ));
    let mapping_key_order = |entry: &Vec<u8>| {
        (
            entry[0],
            read_u64_at(entry, 1),
            read_u64_at(entry, 9),
            read_u32_at(entry, 17),
            read_six_bytes_at(entry, 21),
        )
    };
    mapping_entries.sort_by_key(mapping_key_order);
    let rebuilt_mapping = rebuilt_with_entries(
        &mapping,
        &mapping_entries.first().expect("有条目")[..mapping_key_bytes()],
        &mapping_entries.last().expect("有条目")[..mapping_key_bytes()],
        &mapping_entries,
    );
    replace_the_unit_and_propagate_its_checksum(&mut image, mapping_slot, &rebuilt_mapping);
    // 树表：种类 12 那一条的根指针指 logged_ops 根，别的字段照旧。
    let tree_table_slot = the_slot_of(&newest_root(&image).tree_table);
    let tree_table = parse_index_node(&unit_bytes_at(&image, tree_table_slot, node_bytes()))
        .expect("树表单元解得开");
    let tree_table_entries: Vec<Vec<u8>> = tree_table
        .entries
        .iter()
        .map(|entry| {
            let parsed = TreeTableEntry::parse(entry).expect("树表条目解得开");
            if parsed.kind != TREE_KIND_LOGGED_OPERATIONS {
                return entry.clone();
            }
            let mut rewritten = entry.clone();
            rewritten[14..100].copy_from_slice(&node_pointer_bytes(&node_pointer));
            rewritten
        })
        .collect();
    let rebuilt_tree_table = rebuilt_with_entries(
        &tree_table,
        &tree_table.smallest_key,
        &tree_table.largest_key,
        &tree_table_entries,
    );
    replace_the_unit_and_propagate_its_checksum(&mut image, tree_table_slot, &rebuilt_tree_table);
    // 记账：两盘各多占一槽（已分配加、空闲减）；节点挨着最后一个占着的槽，runs 不变；它开出一个新聚簇段时全空段数减一。
    let opens_a_new_segment =
        (node_slot.0 - UNIT_AREA_START_SLOT).is_multiple_of(CLUSTER_SEGMENT_SLOTS);
    let accounting_slot = the_slot_of(&tree_table_entry_of_kind(&image, TREE_KIND_ACCOUNTING).root);
    let accounting = parse_index_node(&unit_bytes_at(&image, accounting_slot, node_bytes()))
        .expect("记账根解得开");
    let mut rows = accounting.entries.clone();
    for row in &mut rows {
        if !BOTH_DEVICES
            .iter()
            .any(|device| device.0 == read_u32_at(row, ACCOUNTING_ROW_DEVICE_OFFSET))
        {
            continue;
        }
        let value = read_u64_at(row, ACCOUNTING_ROW_VALUE_OFFSET);
        let changed = match read_u16_at(row, 0) {
            STATISTIC_ALLOCATED_BYTES => value + SLOT_BYTES,
            STATISTIC_FREE_BYTES => value - SLOT_BYTES,
            STATISTIC_EMPTY_CLUSTER_SEGMENTS if opens_a_new_segment => value - 1,
            _other_statistic => value,
        };
        row[ACCOUNTING_ROW_VALUE_OFFSET..ACCOUNTING_ROW_VALUE_OFFSET + 8]
            .copy_from_slice(&changed.to_le_bytes());
    }
    let rebuilt_accounting = rebuilt_with_entries(
        &accounting,
        &accounting.smallest_key,
        &accounting.largest_key,
        &rows,
    );
    replace_the_unit_and_propagate_its_checksum(&mut image, accounting_slot, &rebuilt_accounting);
    image
}

/// 单元区第一个聚簇段：新池新建文件的单元都在里面（实例表、树表、第一个数据单元、各棵树的节点）。
fn first_cluster_segment() -> SlotNumber {
    SlotNumber(UNIT_AREA_START_SLOT)
}

/// 合法世界：最新那一版 logged_ops 里一条整理意图，R 是单元区第一个聚簇段、`c0` 与签发 txg 都是这一版的 txg。字段都在白名单里、补零全 0、
/// R 对齐且在单元区内（I-4.4 成立）；R 里的记录分配代都不大于 `c0`（I-5.6 成立）；logged_ops 根兼叶照多层码 2 树走、记进引用，一条违例都没有。
#[test]
fn a_well_formed_compaction_intent_holds_the_target_state_and_the_region_exclusion() {
    let image = image_with_logged_operations_entries_in_the_newest_version(
        "compaction-checker-legal-intent",
        |newest_txg| {
            vec![compaction_intent_entry(
                newest_txg,
                first_cluster_segment(),
                newest_txg,
            )]
        },
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(violated(&verdicts), [], "合法的整理意图一条违例都不该有");
    for invariant in [
        INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
        COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED,
        UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
        FRAGMENTATION_AND_EMPTY_SEGMENT_STATISTICS_EQUAL_A_FULL_SCAN,
    ] {
        assert_eq!(
            verdict_among(&verdicts, invariant),
            InvariantVerdict::Holds,
            "{invariant} 在带整理意图的镜像上真被评估过且成立"
        );
    }
}

/// 必红世界（N4「往条目里塞一个『再搬 N 个』」）：整理意图 value 末尾补零的第一个字节写 5——字段白名单之外的增量。只红 I-4.4。
#[test]
fn a_compaction_intent_carrying_an_increment_in_its_padding_reddens_only_the_target_state_invariant(
) {
    let image = image_with_logged_operations_entries_in_the_newest_version(
        "compaction-checker-intent-with-an-increment",
        |newest_txg| {
            let mut entry =
                compaction_intent_entry(newest_txg, first_cluster_segment(), newest_txg);
            entry[COMPACTION_INTENT_PADDING_OFFSET] = 5;
            vec![entry]
        },
    );
    let red = violated(&check_pool_image(&image));
    assert_eq!(
        violated_names(&red),
        [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE],
        "只红 I-4.4：{red:?}"
    );
    assert!(
        red[0].1.contains("补零"),
        "说明点名补零里的增量：{}",
        red[0].1
    );
}

/// 必红世界（判决 G1「R 的解码拒不对齐的值」）：整理意图 R 的起始槽改成第一个聚簇段起点之后一槽，不再 64 对齐。只红 I-4.4
/// （R 不合法的意图不判 I-5.6，这一份上 I-5.6 报不适用）。
#[test]
fn a_compaction_intent_whose_region_is_off_the_segment_boundary_reddens_only_the_target_state_invariant(
) {
    let image = image_with_logged_operations_entries_in_the_newest_version(
        "compaction-checker-intent-off-the-boundary",
        |newest_txg| {
            let mut entry =
                compaction_intent_entry(newest_txg, first_cluster_segment(), newest_txg);
            entry[COMPACTION_INTENT_REGION_START_SLOT_OFFSET
                ..COMPACTION_INTENT_REGION_START_SLOT_OFFSET + 6]
                .copy_from_slice(&(UNIT_AREA_START_SLOT + 1).to_le_bytes()[..6]);
            vec![entry]
        },
    );
    let verdicts = check_pool_image(&image);
    let red = violated(&verdicts);
    assert_eq!(
        violated_names(&red),
        [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE],
        "只红 I-4.4：{red:?}"
    );
    assert!(
        red[0].1.contains("R 不对齐"),
        "说明点名 R 不对齐：{}",
        red[0].1
    );
    assert!(
        matches!(
            verdict_among(
                &verdicts,
                COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED
            ),
            InvariantVerdict::NotApplicable(_)
        ),
        "R 不合法的意图不判 I-5.6"
    );
}

/// 必红世界（N3「关掉 R 保护，往 R 里分配」）：整理意图的 `c0` 取最新那一版的前一个 txg，R 是第一个聚簇段——新池新建文件那一版分配进 R 的单元
/// 分配代都是这一版的 txg，大于 `c0`：意图签发之后还往 R 里分配了。只红 I-5.6。
#[test]
fn records_allocated_into_the_region_after_the_intent_was_issued_redden_the_region_exclusion() {
    let image = image_with_logged_operations_entries_in_the_newest_version(
        "compaction-checker-allocation-into-the-region",
        |newest_txg| {
            vec![compaction_intent_entry(
                newest_txg,
                first_cluster_segment(),
                CheckpointTxg(newest_txg.0 - 1),
            )]
        },
    );
    let red = violated(&check_pool_image(&image));
    assert_eq!(
        violated_names(&red),
        [COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED],
        "只红 I-5.6：{red:?}"
    );
    assert!(
        red[0].1.contains("R 排他被破"),
        "说明点名 R 排他：{}",
        red[0].1
    );
}

/// 必红世界（主走读按 logged_ops 的 key 形态走那棵树，判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F10，
/// 调查员报告「第 4436 行」一节）：logged_ops 根兼叶里放两条一模一样的整理意图（同 key，不严格递增）。只红 I-1.1，说明点名 logged_ops 那棵树。
/// logged_ops 节点经中央映射那一圈照样记进引用，主走读不走这棵树时引用与 I-3.1 不变；少的是这个节点自己的 I-1.1 / I-1.10，这条就在那里红。
#[test]
fn two_compaction_intents_under_one_key_in_the_logged_operations_leaf_redden_only_the_key_order() {
    let image = image_with_logged_operations_entries_in_the_newest_version(
        "compaction-checker-intents-under-one-key",
        |newest_txg| {
            let entry = compaction_intent_entry(newest_txg, first_cluster_segment(), newest_txg);
            vec![entry.clone(), entry]
        },
    );
    let red = violated(&check_pool_image(&image));
    assert_eq!(
        violated_names(&red),
        ["I-1.1"],
        "logged_ops 叶里两条同 key 的条目只红 I-1.1：{red:?}"
    );
    assert!(
        red[0].1.contains("logged_ops"),
        "说明点名 logged_ops 那棵树：{}",
        red[0].1
    );
}

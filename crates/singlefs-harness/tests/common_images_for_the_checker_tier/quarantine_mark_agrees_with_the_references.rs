//! harness 测试文件 `crates/singlefs-harness/tests/checker_quarantine_mark_agrees_with_the_references.rs` 的场景函数：池级 checker 只认隔离记号
//! （D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」，用户 2026-10-22 定落点甲、checker 只认记号，变体 `mark_only`）的四个世界，
//! 池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去（做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件收下镜像就地判；
//! checker 档 `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 经 `#[path]` 引这一份做 GPU 与 CPU 对拍。
//!
//! 每个世界的底子相同：新池新建文件（A，txg 3）之后 A 的数据单元两块盘那一份都改坏，覆盖写 B（txg 4）释放之前读盘核出对不上、隔离它
//! （两块盘那条记录都留在已分配、分配代 3、带隔离记号），再覆盖写 C、D、E，抬 F 到 4——A 的根掉出候选集，那两条记录从此没有任何有效根引用。
//! 要「记号与引用不符」的那几形，在 B 之后或抬 F 之前改一次内存里的分配器的记录（下一次发布照它重写那片叶，盘上的镜像自洽、校验和都对得上），
//! 代替一个写错记号的写者。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, copy_each_root_into_the_new_root_segment_of_its_journal_records, parameters,
    publish_overwrite_in_process, BuiltPool, PoolImageGeometry, FIXED_WRITE_TIME_SECONDS,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use crate::scenarios_of_release_checksum_quarantine::{
    content_of, corrupt_the_copy_on, data_slot_of_the_first_version,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::allocator::{Placement, PoolAllocator};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::mount::{raise_rollback_floor, ShadowLedger};
use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer};
use singlefs_core::records::{TreeTableEntry, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, RootRingSlot};
use singlefs_core::unit::{build_index_node, parse_index_node, IndexNodeHeader};
use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES, ROOT_RING_REGIONS, SLOT_BYTES};
use singlefs_harness::memory_pool::{MemoryPool, SECTOR_BYTES};

/// 池里两块盘。
const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
/// 数据单元跨两槽。
const DATA_UNIT_SPAN_SLOTS: u64 = 2;
/// 抬 F 抬到 B 的 txg：A 的根（txg 3）掉出候选集。
const FLOOR_ABOVE_THE_FIRST_VERSION: CheckpointTxg = CheckpointTxg(4);
/// 隔离记号与引用相符那一条（`singlefs_checker::image::QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES`）。
pub const QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES: &str = "I-3.17";

/// 底子搭完的样子：池、A 的数据单元的起点槽、最新那一版（抬 F 那一串的最后一次空发布，数据单元照抄 E 的）数据单元的起点槽。
pub struct QuarantinedUnitNoRootReferences {
    pub pool: BuiltPool,
    pub data_slot: SlotNumber,
    pub newest_data_slot: SlotNumber,
}

/// 底子（见文件头）：B 隔离之后把内存里的分配器交给 `after_the_quarantine`（带着 A 的数据单元的落点），抬 F 之前交给
/// `before_raising_the_floor`（带着 E 的数据单元的落点）；两处都不改就是写者照条款写的那一形。
///
/// # Panics
/// B 没有隔离两块盘那两份。
pub fn quarantined_unit_no_root_references(
    tag: &str,
    after_the_quarantine: impl FnOnce(&mut PoolAllocator, Placement),
    before_raising_the_floor: impl FnOnce(&mut PoolAllocator, Placement),
) -> QuarantinedUnitNoRootReferences {
    let mut pool = build_pool(tag);
    let first = pool.output.clone();
    let data_slot = data_slot_of_the_first_version(&pool);
    {
        let devices = pool.devices.as_mut().expect("镜像还开着");
        for device in BOTH_DEVICES {
            corrupt_the_copy_on(devices, device, data_slot);
        }
    }
    let mut current = publish_overwrite_in_process(
        &mut pool,
        &first,
        &content_of(2800, 5),
        FIXED_WRITE_TIME_SECONDS + 60,
        InstanceGeneration(1),
    )
    .expect("核出对不上时发布照成");
    assert_eq!(
        current.quarantined_after_release_checksum_mismatch.len(),
        2,
        "两盘那一份都隔离"
    );
    after_the_quarantine(
        &mut pool.allocator,
        Placement {
            slot: data_slot,
            span: DATA_UNIT_SPAN_SLOTS,
        },
    );
    for (seed, seconds) in [(6, 120), (7, 180), (8, 240)] {
        current = publish_overwrite_in_process(
            &mut pool,
            &current,
            &content_of(2700, seed),
            FIXED_WRITE_TIME_SECONDS + seconds,
            InstanceGeneration(1),
        )
        .expect("接着覆盖写");
    }
    let newest_data_slot = current.data_pointers[0].locations[0].slot;
    before_raising_the_floor(
        &mut pool.allocator,
        Placement {
            slot: newest_data_slot,
            span: DATA_UNIT_SPAN_SLOTS,
        },
    );
    let raised = {
        let publish_parameters = parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        raise_rollback_floor(
            &publish_parameters,
            devices,
            &mut pool.allocator,
            &mut current,
            FLOOR_ABOVE_THE_FIRST_VERSION,
            ShadowLedger::On,
        )
        .expect("抬 F 到 4：A 的根（txg 3）掉出候选集")
    };
    assert!(!raised.publishes.is_empty(), "抬 F 发了空发布");
    assert_eq!(
        current.data_pointers[0].locations[0].slot, newest_data_slot,
        "抬 F 那一串空发布照抄 E 的数据单元"
    );
    QuarantinedUnitNoRootReferences {
        pool,
        data_slot,
        newest_data_slot,
    }
}

/// 内存里的分配器不动。
pub fn leave_the_account_as_the_writer_left_it(
    _allocator: &mut PoolAllocator,
    _placement: Placement,
) {
}

/// 去掉这个落点在 `devices` 那几块盘上那条记录的隔离记号（代替写者漏写记号）。
fn clear_the_quarantine_mark_on(
    allocator: &mut PoolAllocator,
    placement: Placement,
    devices: &[DeviceIdentity],
) {
    for device in devices {
        allocator.clear_the_quarantine_mark_of_a_record_the_rollback_target_references(
            *device, placement,
        );
    }
}

/// 一块盘上 `slot` 起那一份换成 `unit`（整份写进镜像）。
pub fn replace_the_copy_in_the_image(
    image: &mut MemoryPool,
    device: DeviceIdentity,
    slot: SlotNumber,
    unit: &[u8],
) {
    image
        .devices
        .get_mut(&device)
        .expect("池里有这块盘")
        .write(slot.to_device_offset(), unit);
}

/// 一块盘上 `slot` 起的整份数据单元。
pub fn data_unit_in_the_image(
    image: &MemoryPool,
    device: DeviceIdentity,
    slot: SlotNumber,
) -> Vec<u8> {
    image.devices.get(&device).expect("池里有这块盘").read(
        slot.to_device_offset(),
        usize::try_from(DATA_UNIT_BYTES).expect("32768"),
    )
}

/// 世界一：两块盘都带记号的隔离不红——I-3.1、I-3.11、I-3.15 豁免、I-3.17 成立；两份副本坏着与改回对得上（再翻一次同一个字节）都一样，
/// 只认记号、不读副本。按副本判（改之前的读法）时改回对得上那一张 I-3.1、I-3.11、I-3.15 红。
pub fn a_quarantine_carrying_the_mark_on_both_devices_is_exempted_without_reading_its_copies(
    images: &mut PoolCheckerImages,
) {
    let mut built = quarantined_unit_no_root_references(
        "quarantine-mark-on-both-devices",
        leave_the_account_as_the_writer_left_it,
        leave_the_account_as_the_writer_left_it,
    );
    let the_marked_quarantine_holds = || {
        PoolCheckerExpectation::only_the_named_invariants()
            .and_holds("I-3.1")
            .and_holds("I-3.11")
            .and_holds("I-3.15")
            .and_holds(QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES)
    };
    let image_with_the_damaged_copies = built.pool.memory_pool();
    images.hand_out(
        "两块盘都带记号、两份副本坏着",
        || image_with_the_damaged_copies.clone(),
        the_marked_quarantine_holds(),
    );
    {
        let devices = built.pool.devices.as_mut().expect("镜像还开着");
        for device in BOTH_DEVICES {
            corrupt_the_copy_on(devices, device, built.data_slot);
        }
    }
    let image_with_the_copies_intact = built.pool.memory_pool();
    images.hand_out(
        "两块盘都带记号、两份副本改回对得上",
        || image_with_the_copies_intact.clone(),
        the_marked_quarantine_holds(),
    );
}

/// 世界二：一块盘带记号、一块盘不带（B 之后去掉盘 0 那条记录的记号，C 照它重写盘 0 那片叶）：I-3.17 红在第三句；那一组不豁免，I-3.1、I-3.11 照判违例。
/// I-3.17 不判第三句时 I-3.17 不红。
pub fn a_quarantine_carrying_the_mark_on_one_device_only_is_violated_by_the_third_sentence(
    images: &mut PoolCheckerImages,
) {
    let built = quarantined_unit_no_root_references(
        "quarantine-mark-on-one-device-only",
        |allocator, placement| {
            clear_the_quarantine_mark_on(allocator, placement, &[DeviceIdentity(0)]);
        },
        leave_the_account_as_the_writer_left_it,
    );
    let image = built.pool.memory_pool();
    images.hand_out(
        "盘 1 带记号、盘 0 不带",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated(QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES)
            .and_violated("I-3.1")
            .and_violated("I-3.11"),
    );
}

/// 世界三：带记号的记录罩住的单元被最新根引用（抬 F 之前给 E 的数据单元两块盘那两条记录置上记号，抬 F 那一串空发布照抄 E 的数据单元、
/// 照账重写那片叶）：I-3.17 红在第二句；A 那一组照旧两块盘都带记号、豁免，I-3.15 不红。I-3.17 不判第二句时 I-3.17 不红。
pub fn a_record_carrying_the_mark_under_a_unit_the_newest_root_references_is_violated_by_the_second_sentence(
    images: &mut PoolCheckerImages,
) {
    let built = quarantined_unit_no_root_references(
        "quarantine-mark-under-a-referenced-unit",
        leave_the_account_as_the_writer_left_it,
        |allocator, newest_data_placement| {
            // 两块盘都点名：两条记录都留在已分配、置上记号，释放代用不上
            allocator.release_leaving_the_record_allocated_on(
                newest_data_placement,
                CheckpointTxg(u64::MAX),
                &BOTH_DEVICES,
            );
        },
    );
    let image = built.pool.memory_pool();
    images.hand_out(
        "最新根引用的数据单元两块盘那两条记录带记号",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated(QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES)
            .and_not_violated("I-3.15"),
    );
}

/// 世界四：没记号的隔离照判违例（B 之后去掉两块盘那两条记录的记号，C 照它重写两片叶）：两份副本坏着、改回对得上两张都是 I-3.1、I-3.11、I-3.15 红，
/// 不退回按副本判；一条带记号的记录都没有，I-3.17 不红。按副本判（`mark_or_today` 那一变体、改之前的读法）时副本坏着那一张三条都豁免。
pub fn a_quarantine_without_the_mark_is_violated_whether_or_not_its_copies_read_back(
    images: &mut PoolCheckerImages,
) {
    let mut built = quarantined_unit_no_root_references(
        "quarantine-without-the-mark",
        |allocator, placement| {
            clear_the_quarantine_mark_on(allocator, placement, &BOTH_DEVICES);
        },
        leave_the_account_as_the_writer_left_it,
    );
    let the_unmarked_quarantine_is_violated = || {
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated("I-3.1")
            .and_violated("I-3.11")
            .and_violated("I-3.15")
            .and_not_violated(QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES)
    };
    let image_with_the_damaged_copies = built.pool.memory_pool();
    images.hand_out(
        "两块盘都不带记号、两份副本坏着",
        || image_with_the_damaged_copies.clone(),
        the_unmarked_quarantine_is_violated(),
    );
    {
        let devices = built.pool.devices.as_mut().expect("镜像还开着");
        for device in BOTH_DEVICES {
            corrupt_the_copy_on(devices, device, built.data_slot);
        }
    }
    let image_with_the_copies_intact = built.pool.memory_pool();
    images.hand_out(
        "两块盘都不带记号、两份副本改回对得上",
        || image_with_the_copies_intact.clone(),
        the_unmarked_quarantine_is_violated(),
    );
}

/// 世界五：同一单元的两条记录分落树表里两棵分配记录树、只有一条带记号——I-3.17 第三句按最新根的全部分配记录树合起来归组
/// （`walk.rs` 的 `allocation_records_of_the_newest_root_without_judging` 把种类 3 的每一棵读进一张表，`judge_quarantine_marks_against_the_references`
/// 按单元归组），红在第三句；只在一棵树里归组的读法（GPU 扫描内核改之前的 `c_judge_the_quarantine_marks_of_the_records_read`）两棵树里各看不到
/// 另一条，不红。那一组有一条不带记号、不豁免，I-3.1、I-3.11 照判违例。
///
/// 底子是世界二（盘 0 那条不带记号、盘 1 那条带）。写者只写一棵分配记录树，另一棵按字节造
/// （[`split_the_allocation_record_tree_of_the_newest_root_by_device`]）：树表里种类 3 两条、deadlist 一条都没有，I-9.16 红在 ③ ④；按字节造的那棵的根头里的树号
/// 照原来的根是分配记录树的号，树表里那一条的树号是 deadlist 的，I-1.3 红。
pub fn a_unit_whose_records_fall_into_two_allocation_record_trees_carrying_the_mark_in_one_only_is_violated_by_the_third_sentence(
    images: &mut PoolCheckerImages,
) {
    let built = quarantined_unit_no_root_references(
        "quarantine-mark-on-one-device-in-two-allocation-record-trees",
        |allocator, placement| {
            clear_the_quarantine_mark_on(allocator, placement, &[DeviceIdentity(0)]);
        },
        leave_the_account_as_the_writer_left_it,
    );
    let mut image = built.pool.memory_pool();
    split_the_allocation_record_tree_of_the_newest_root_by_device(&mut image);
    images.hand_out(
        "盘 0 那条不带记号、在写者写的那棵分配记录树里，盘 1 那条带记号、在按字节造的那棵里",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated(QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES)
            .and_violated("I-3.1")
            .and_violated("I-3.11")
            .and_violated("I-9.16")
            .and_violated("I-1.3"),
    );
}

/// 一份位置条目的 14 字节（盘 4、槽 6、整单元校验和 4，`LocationEntry::write_to` 的写法）：码 2 节点的条目与根槽里引用一个单元的那一份，
/// 照这串找、照这串换。
fn location_entry_bytes(device: DeviceIdentity, slot: SlotNumber, unit_checksum: u32) -> Vec<u8> {
    [
        device.0.to_le_bytes().to_vec(),
        slot.0.to_le_bytes()[..6].to_vec(),
        unit_checksum.to_le_bytes().to_vec(),
    ]
    .concat()
}

/// `haystack` 里每一处 `needle` 换成 `replacement`（两串等长），交回换了几处。迭代上界是 `haystack` 的长度。
fn replace_every_occurrence(haystack: &mut [u8], needle: &[u8], replacement: &[u8]) -> usize {
    assert_eq!(needle.len(), replacement.len(), "换上去的与原来的等长");
    let mut replaced = 0;
    let mut index = 0;
    while index + needle.len() <= haystack.len() {
        if haystack[index..index + needle.len()] == *needle {
            haystack[index..index + needle.len()].copy_from_slice(replacement);
            replaced += 1;
            index += needle.len();
        } else {
            index += 1;
        }
    }
    replaced
}

/// 一块盘上一份单元换了字节：它在哪、换之前与换之后的整单元校验和。
#[derive(Clone, Copy, Debug)]
struct UnitCopyRewritten {
    device: DeviceIdentity,
    slot: SlotNumber,
    checksum_before: u32,
    checksum_after: u32,
}

fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

/// 一块盘上 `slot` 起一个码 2 节点那么长的字节。
fn node_copy_at(image: &MemoryPool, device: DeviceIdentity, slot: SlotNumber) -> Vec<u8> {
    image
        .devices
        .get(&device)
        .expect("池里有这块盘")
        .read(slot.to_device_offset(), node_bytes())
}

/// 照 `header` 的头（树、层级、key 宽与 key 区间、诞生代号、实例代号、条目宽）重建一个码 2 节点，条目换成 `entries`、出生序号换成 `birth_sequence`。
fn index_node_rebuilt_from_the_header(
    header: &IndexNodeHeader,
    entries: &[Vec<u8>],
    birth_sequence: BirthSequence,
) -> Vec<u8> {
    build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        header.birth_txg,
        &parameters().filesystem_identifier,
        header.instance,
        birth_sequence,
        u16::try_from(header.entry_width).expect("码 2 条目宽装得进 u16"),
        entries,
    )
}

/// 单元区里写过的每个槽（两块盘各数各的）上解得开的码 2 节点：（盘, 槽）。
fn index_node_copies_in_the_unit_area(image: &MemoryPool) -> Vec<(DeviceIdentity, SlotNumber)> {
    let unit_area_start = PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters()
        .unit_area_start_slot();
    let sectors_per_slot = SLOT_BYTES / SECTOR_BYTES;
    let mut copies = Vec::new();
    for (device, sparse_device) in &image.devices {
        let mut written_slots: Vec<u64> = sparse_device
            .written_sectors()
            .keys()
            .map(|sector| sector / sectors_per_slot)
            .filter(|slot| *slot >= unit_area_start.0)
            .collect();
        written_slots.dedup();
        for slot in written_slots {
            if parse_index_node(&node_copy_at(image, *device, SlotNumber(slot))).is_ok() {
                copies.push((*device, SlotNumber(slot)));
            }
        }
    }
    copies
}

/// 根环每一槽（三个区、每区 S 槽）上自证过的根：（盘, 槽的偏移, 根）。
fn roots_in_the_root_ring(
    image: &MemoryPool,
) -> Vec<(DeviceIdentity, DeviceOffsetInBytes, RootRecord)> {
    let make_filesystem_parameters = parameters();
    let geometry = PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters();
    let mut roots = Vec::new();
    for region in 0..ROOT_RING_REGIONS {
        let device = make_filesystem_parameters.region_devices
            [usize::try_from(region).expect("区域号装得进 usize")];
        for slot in 0..make_filesystem_parameters
            .geometry
            .root_ring_slots_per_region
            .count()
        {
            let offset = slot_offset(
                RootRingSlot { region, slot },
                make_filesystem_parameters
                    .geometry
                    .fixed_structure_slot_spacing,
            );
            let bytes = image
                .devices
                .get(&device)
                .expect("池里有这块盘")
                .read(offset, geometry.root_slot_bytes());
            if let Some(root) =
                RootRecord::parse_slot(&bytes, &make_filesystem_parameters.filesystem_identifier)
            {
                roots.push((device, offset, root));
            }
        }
    }
    roots
}

/// 根环里实例代号与 txg 最大的那条根。
fn newest_root_in_the_root_ring(image: &MemoryPool) -> RootRecord {
    roots_in_the_root_ring(image)
        .into_iter()
        .map(|(_, _, root)| root)
        .max_by_key(|root| (root.instance, root.checkpoint_txg))
        .expect("根环里有自证过的根")
}

/// 一条指针的位置条目里指着 `rewritten` 那一份、带着它换之前的校验和的，换成换之后的；交回换了几条。
fn rewrite_the_location_entries_of_a_pointer(
    pointer: &mut NodePointer,
    rewritten: UnitCopyRewritten,
) -> usize {
    let mut replaced = 0;
    for location in &mut pointer.locations {
        if location.device == rewritten.device
            && location.slot == rewritten.slot
            && location.unit_checksum == rewritten.checksum_before
        {
            location.unit_checksum = rewritten.checksum_after;
            replaced += 1;
        }
    }
    replaced
}

/// 几份单元换了字节之后，引用它们的位置条目沿引用链补上新的整单元校验和：单元区里每个码 2 节点（分配记录树的父节点、树表、中央映射树）
/// 条目里带着旧的那一串的换成新的、照原头重建、写回原处（它自己换了字节，接着往上补），根环里每条根的四条指针同样换、重封；
/// 最后把每条根抄进同代 journal 记录的新根段（`copy_each_root_into_the_new_root_segment_of_its_journal_records`）。
/// 待补的按换的先后排队、先换的先补：同一份单元换过几次（两块盘上的孩子各换一次，父节点就换两次）时，引用它的位置条目照次序一步步换到最后那个校验和。
/// 迭代上界：每一轮从队头取一份，码 2 节点逐个查一遍；补出来的新一份只有引用链往上那几层，引用链无环。
fn propagate_the_rewritten_unit_copies(
    image: &mut MemoryPool,
    rewritten_copies: Vec<UnitCopyRewritten>,
) {
    let index_node_copies = index_node_copies_in_the_unit_area(image);
    let mut pending: std::collections::VecDeque<UnitCopyRewritten> = rewritten_copies.into();
    while let Some(rewritten) = pending.pop_front() {
        let before =
            location_entry_bytes(rewritten.device, rewritten.slot, rewritten.checksum_before);
        let after =
            location_entry_bytes(rewritten.device, rewritten.slot, rewritten.checksum_after);
        for (device, slot) in &index_node_copies {
            let bytes = node_copy_at(image, *device, *slot);
            let header = parse_index_node(&bytes).expect("单元区里数到的码 2 节点照旧解得开");
            let mut entries = header.entries.clone();
            let replaced: usize = entries
                .iter_mut()
                .map(|entry| replace_every_occurrence(entry, &before, &after))
                .sum();
            if replaced == 0 {
                continue;
            }
            assert_eq!(
                index_node_rebuilt_from_the_header(&header, &header.entries, header.birth_sequence),
                bytes,
                "照原头原样重建的码 2 节点与盘上那一份逐字节相同：重建这一步只换条目"
            );
            let rebuilt =
                index_node_rebuilt_from_the_header(&header, &entries, header.birth_sequence);
            image
                .devices
                .get_mut(device)
                .expect("池里有这块盘")
                .write(slot.to_device_offset(), &rebuilt);
            pending.push_back(UnitCopyRewritten {
                device: *device,
                slot: *slot,
                checksum_before: crc32_castagnoli(&bytes),
                checksum_after: crc32_castagnoli(&rebuilt),
            });
        }
        let root_slot_bytes =
            PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters()
                .root_slot_bytes();
        for (device, offset, root) in roots_in_the_root_ring(image) {
            let mut rewritten_root = root;
            let replaced = rewrite_the_location_entries_of_a_pointer(
                &mut rewritten_root.tree_table,
                rewritten,
            ) + rewrite_the_location_entries_of_a_pointer(
                &mut rewritten_root.instance_table,
                rewritten,
            ) + rewrite_the_location_entries_of_a_pointer(
                &mut rewritten_root.mapping_root,
                rewritten,
            ) + rewrite_the_location_entries_of_a_pointer(
                &mut rewritten_root.allocation_record_tree_root,
                rewritten,
            );
            if replaced == 0 {
                continue;
            }
            let sparse_device = image.devices.get_mut(&device).expect("池里有这块盘");
            assert_eq!(
                root.to_slot(root_slot_bytes),
                sparse_device.read(offset, root_slot_bytes),
                "照原样重封的根槽与盘上那一槽逐字节相同：重封这一步只换位置条目"
            );
            sparse_device.write(offset, &rewritten_root.to_slot(root_slot_bytes));
        }
    }
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
}

/// 把 `bytes` 写进 `pointer` 两条位置条目指的那两份（每块盘一份），交回两份各自换了什么。
fn write_both_copies_of_a_unit(
    image: &mut MemoryPool,
    pointer: &NodePointer,
    bytes: &[u8],
) -> Vec<UnitCopyRewritten> {
    pointer
        .locations
        .iter()
        .map(|location| {
            let before = node_copy_at(image, location.device, location.slot);
            image
                .devices
                .get_mut(&location.device)
                .expect("池里有这块盘")
                .write(location.slot.to_device_offset(), bytes);
            UnitCopyRewritten {
                device: location.device,
                slot: location.slot,
                checksum_before: crc32_castagnoli(&before),
                checksum_after: crc32_castagnoli(bytes),
            }
        })
        .collect()
}

/// 这块盘上写过的槽里最大的那一个的下一槽（没写过的槽）。
fn slot_after_every_written_slot(image: &MemoryPool, device: DeviceIdentity) -> SlotNumber {
    let sectors_per_slot = SLOT_BYTES / SECTOR_BYTES;
    let highest_written_slot = image
        .devices
        .get(&device)
        .expect("池里有这块盘")
        .written_sectors()
        .keys()
        .next_back()
        .map(|sector| sector / sectors_per_slot)
        .expect("这块盘写过");
    SlotNumber(highest_written_slot + 1)
}

/// 单元区里与 `header` 同诞生代号、同实例代号的码 2 节点里最大的出生序号。
fn highest_birth_sequence_of_the_index_nodes_born_with(
    image: &MemoryPool,
    header: &IndexNodeHeader,
) -> u32 {
    index_node_copies_in_the_unit_area(image)
        .into_iter()
        .map(|(device, slot)| {
            parse_index_node(&node_copy_at(image, device, slot)).expect("数到的码 2 节点解得开")
        })
        .filter(|node| node.birth_txg == header.birth_txg && node.instance == header.instance)
        .map(|node| node.birth_sequence.0)
        .max()
        .expect("至少有 header 自己那一个节点")
}

/// 最新根的分配记录树按盘拆成两棵：根（层级按池几何定，`position_addressed::allocation_record_tree_root_level`；两块盘各有条目指着那块盘的下一层）
/// 原处只留盘 0 的条目，盘 1 的条目搬进一个新的根——头照原来的根（同一棵树的号、同一个诞生代号与实例代号），出生序号取同代同实例的码 2 节点里没用过的
/// 下一个（中央映射里没有它的 key，照它自己的位置条目读），写在两块盘单元区里写过的槽之后；树表里 deadlist 那一条（第一版不建根、根指针全零）
/// 改成种类 3、根指针指着新根。两处改动各沿引用链补上校验和（[`propagate_the_rewritten_unit_copies`]）。新根没有分配记录。
///
/// # Panics
/// 最新根的分配记录树的根上两块盘不是各有条目，或树表里没有根指针全零的 deadlist 那一条。
fn split_the_allocation_record_tree_of_the_newest_root_by_device(image: &mut MemoryPool) {
    let tree_table_of = |pool_image: &MemoryPool, root: &RootRecord| {
        let location = root.tree_table.locations[0];
        parse_index_node(&node_copy_at(pool_image, location.device, location.slot))
            .expect("最新根的树表解得开")
    };
    let newest_root = newest_root_in_the_root_ring(image);
    let allocation_root_pointer = tree_table_of(image, &newest_root)
        .entries
        .iter()
        .map(|entry| TreeTableEntry::parse(entry).expect("写者写的树表条目解得开"))
        .find(|entry| entry.kind == TREE_KIND_ALLOCATION)
        .expect("最新根的树表里有分配记录树那一条")
        .root;
    let root_location = allocation_root_pointer.locations[0];
    let root_bytes = node_copy_at(image, root_location.device, root_location.slot);
    let root_node = parse_index_node(&root_bytes).expect("分配记录树的根解得开");
    assert_eq!(
        index_node_rebuilt_from_the_header(
            &root_node,
            &root_node.entries,
            root_node.birth_sequence
        ),
        root_bytes,
        "照原头原样重建的分配记录树根与写者写下的逐字节相同"
    );
    let device_of_the_entry = |entry: &Vec<u8>| {
        u32::from_le_bytes(entry[..4].try_into().expect("条目 key 头 4 字节是盘"))
    };
    let (entries_on_device_zero, entries_on_device_one): (Vec<Vec<u8>>, Vec<Vec<u8>>) = root_node
        .entries
        .iter()
        .cloned()
        .partition(|entry| device_of_the_entry(entry) == 0);
    assert!(
        !entries_on_device_zero.is_empty()
            && !entries_on_device_one.is_empty()
            && entries_on_device_one
                .iter()
                .all(|entry| device_of_the_entry(entry) == 1),
        "分配记录树的根上盘 0、盘 1 各有条目（各指着那块盘的下一层）"
    );
    let root_keeping_device_zero = index_node_rebuilt_from_the_header(
        &root_node,
        &entries_on_device_zero,
        root_node.birth_sequence,
    );
    let rewritten_root_copies =
        write_both_copies_of_a_unit(image, &allocation_root_pointer, &root_keeping_device_zero);
    propagate_the_rewritten_unit_copies(image, rewritten_root_copies);

    let birth_sequence_of_the_new_root = BirthSequence(
        highest_birth_sequence_of_the_index_nodes_born_with(image, &root_node)
            .checked_add(1)
            .expect("出生序号没用到 u32 的顶"),
    );
    let new_root = index_node_rebuilt_from_the_header(
        &root_node,
        &entries_on_device_one,
        birth_sequence_of_the_new_root,
    );
    let new_root_checksum = crc32_castagnoli(&new_root);
    let new_root_pointer = NodePointer {
        head: allocation_root_pointer.head,
        locations: allocation_root_pointer
            .locations
            .map(|location| LocationEntry {
                device: location.device,
                slot: slot_after_every_written_slot(image, location.device),
                unit_checksum: new_root_checksum,
            }),
        instance: allocation_root_pointer.instance,
        birth_sequence: birth_sequence_of_the_new_root,
    };
    for location in &new_root_pointer.locations {
        image
            .devices
            .get_mut(&location.device)
            .expect("池里有这块盘")
            .write(location.slot.to_device_offset(), &new_root);
    }

    let newest_root_after_the_root_was_split = newest_root_in_the_root_ring(image);
    let tree_table = tree_table_of(image, &newest_root_after_the_root_was_split);
    let mut deadlist_entries_turned_into_allocation_entries = 0;
    let entries: Vec<Vec<u8>> = tree_table
        .entries
        .iter()
        .map(|entry_bytes| {
            let mut entry = TreeTableEntry::parse(entry_bytes).expect("写者写的树表条目解得开");
            if entry.kind != TREE_KIND_DEADLIST {
                return entry_bytes.clone();
            }
            assert_eq!(
                entry.to_bytes(),
                *entry_bytes,
                "deadlist 那一条照原样重编与写者写下的逐字节相同"
            );
            assert_eq!(
                entry.root,
                NodePointer::empty_root(),
                "deadlist 那一条第一版不建根、根指针全零"
            );
            entry.kind = TREE_KIND_ALLOCATION;
            entry.root = new_root_pointer;
            deadlist_entries_turned_into_allocation_entries += 1;
            entry.to_bytes()
        })
        .collect();
    assert_eq!(
        deadlist_entries_turned_into_allocation_entries, 1,
        "树表里 deadlist 恰好一条"
    );
    let tree_table_with_two_allocation_entries =
        index_node_rebuilt_from_the_header(&tree_table, &entries, tree_table.birth_sequence);
    let rewritten_tree_table_copies = write_both_copies_of_a_unit(
        image,
        &newest_root_after_the_root_was_split.tree_table,
        &tree_table_with_two_allocation_entries,
    );
    propagate_the_rewritten_unit_copies(image, rewritten_tree_table_copies);
}

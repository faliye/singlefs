//! 造一种不合法的镜像：几条老根各把自己那棵分配记录树整棵照抄到空槽上、每条记录的代号加一个这条根独有的偏移，
//! 再把这条根的树表抄一份改指新树、根槽改指新树表。抄出来的每个节点、树表与根槽都照 `singlefs-core` 的写法重建
//! （`build_index_node`、`TreeTableEntry::to_bytes`、`RootRecord::to_slot`），校验和由它们算、指针里的整单元校验和照新字节算，
//! 所以一路校验和都对得上，坏的只是内容：这几条根自己那一版的未释放记录（盘、槽、代）与别的版本不重合。
//! 原来的树、树表与 journal 记录一个字不动（记录里那条根的树表指针因此与根槽不同）。
//!
//! 给 GPU 扫描内核 I-3.10 去重表（`C_MAX_EXAMINED`）满了之后照判那一判造输入：合法的流上各版未释放记录去重累计不多于最新那一版的
//! 分配记录条数（回收要求释放代不高于回退下界，候选集里的版本都不低于它），一版多于 4096 条时分配记录那张表先满；
//! 老根改了代之后，同一批还在用的单元在每条改过的根里各算一份，累计越过那张表、单版的记录条数不变。
//! 只依赖 `singlefs-core`、`singlefs-format` 与 `singlefs-harness` 的内存池，不依赖 checker 档。

use std::collections::BTreeSet;

use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::bytes::{ByteReader, ByteWriter};
use singlefs_core::checksum::crc32_castagnoli;
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::pointer::NodePointer;
use singlefs_core::records::{TreeTableEntry, TREE_KIND_ALLOCATION};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish};
use singlefs_core::unit::{build_index_node, parse_index_node, IndexNodeHeader};
use singlefs_format::{NODE_BYTES, SLOT_BYTES};
use singlefs_harness::memory_pool::MemoryPool;

/// 分配记录的键：盘、槽、代（扫描内核 I-3.10 去重的键，同 CPU 判器 `judge_allocation_generations_against_unit_births`）。
pub type AllocationRecordKey = (DeviceIdentity, u64, CheckpointTxg);

/// 抄出来的节点往哪些空槽放：从 `next_slot` 起逐个往后取，每取一个先核两块盘上这个槽整片都是 0（没写过）。
pub struct FreeSlotsForTheCopies {
    next_slot: u64,
}

impl FreeSlotsForTheCopies {
    #[must_use]
    pub fn starting_at(first_slot: u64) -> Self {
        Self {
            next_slot: first_slot,
        }
    }

    /// 取下一个槽：`devices` 上这个槽都没写过。
    ///
    /// # Panics
    /// 有一块盘上这个槽写过东西（起点挑错了，抄出来的节点会盖掉别的单元）。
    fn take_one_unwritten_on(&mut self, image: &MemoryPool, devices: [DeviceIdentity; 2]) -> u64 {
        let slot = self.next_slot;
        self.next_slot += 1;
        for device in devices {
            let bytes = image
                .devices
                .get(&device)
                .unwrap_or_else(|| panic!("指针点名的盘 {} 在池里", device.0))
                .read(
                    DeviceOffsetInBytes(slot * SLOT_BYTES),
                    usize::try_from(NODE_BYTES).expect("16384"),
                );
            assert!(
                bytes.iter().all(|byte| *byte == 0),
                "盘 {} 槽 {slot} 写过东西：抄出来的节点只放没写过的槽",
                device.0
            );
        }
        slot
    }
}

/// 读一个码 2 节点：按指针第一条位置条目的盘与槽读整个单元。
fn read_node(image: &MemoryPool, pointer: &NodePointer) -> Vec<u8> {
    let location = pointer.locations[0];
    image
        .devices
        .get(&location.device)
        .unwrap_or_else(|| panic!("指针点名的盘 {} 在池里", location.device.0))
        .read(
            DeviceOffsetInBytes(location.slot.0 * SLOT_BYTES),
            usize::try_from(NODE_BYTES).expect("16384"),
        )
}

/// 照 `header` 的身份字段与 `entries` 重建一个码 2 节点（载荷与头两道校验和由 `build_index_node` 算）。
fn rebuilt_node(
    header: &IndexNodeHeader,
    entries: &[Vec<u8>],
    parameters: &MakeFilesystemParameters,
) -> Vec<u8> {
    build_index_node(
        header.tree,
        header.level,
        header.key_width,
        &header.smallest_key,
        &header.largest_key,
        header.birth_txg,
        &parameters.filesystem_identifier,
        header.instance,
        header.birth_sequence,
        u16::try_from(header.entry_width).expect("条目宽 2 字节"),
        entries,
    )
}

/// 把 `bytes` 写到一个空槽上（两块盘各一份，同槽），交回指着它的指针：照 `pointer` 抄，只换两条位置条目的槽与整单元校验和。
fn write_to_a_free_slot(
    image: &mut MemoryPool,
    pointer: &NodePointer,
    bytes: &[u8],
    free_slots: &mut FreeSlotsForTheCopies,
) -> NodePointer {
    let devices = [pointer.locations[0].device, pointer.locations[1].device];
    let slot = free_slots.take_one_unwritten_on(image, devices);
    for device in devices {
        image
            .devices
            .get_mut(&device)
            .unwrap_or_else(|| panic!("指针点名的盘 {} 在池里", device.0))
            .write(DeviceOffsetInBytes(slot * SLOT_BYTES), bytes);
    }
    let unit_checksum = crc32_castagnoli(bytes);
    let mut copied = *pointer;
    for location in &mut copied.locations {
        location.slot = SlotNumber(slot);
        location.unit_checksum = unit_checksum;
    }
    copied
}

/// 照抄 `pointer` 指的那个分配记录树节点与它底下整棵子树：叶里每条记录的代号加 `generation_offset`（未释放的那几条连同新代记进
/// `unreleased_record_keys`），内部节点的子指针换成指向抄出来的孩子。交回指向抄出来的这个节点的指针。
///
/// # Panics
/// 节点解不开；内部条目里的子指针读不出；照抄不改时重建出来的字节与盘上原样不同（重建的写法与盘上的对不上，这份镜像就不只坏在代号上）。
fn copy_the_subtree_under_another_generation(
    image: &mut MemoryPool,
    pointer: &NodePointer,
    generation_offset: u64,
    parameters: &MakeFilesystemParameters,
    free_slots: &mut FreeSlotsForTheCopies,
    unreleased_record_keys: &mut BTreeSet<AllocationRecordKey>,
) -> NodePointer {
    let original = read_node(image, pointer);
    let header = parse_index_node(&original).expect("分配记录树的节点解得开");
    assert_eq!(
        rebuilt_node(&header, &header.entries, parameters),
        original,
        "照抄不改时重建出来的节点与盘上原样逐字节相同"
    );
    let entries: Vec<Vec<u8>> = if header.level == 0 {
        header
            .entries
            .iter()
            .map(|entry| {
                let record = AllocationRecord::parse(entry).expect("分配记录 20 字节");
                let regenerated = AllocationRecord {
                    device: record.device,
                    slot: record.slot,
                    span_slots: record.span_slots,
                    generation: CheckpointTxg(record.generation.0 + generation_offset),
                    is_released: record.is_released,
                };
                if !regenerated.is_released {
                    unreleased_record_keys.insert((
                        regenerated.device,
                        regenerated.slot.0,
                        regenerated.generation,
                    ));
                }
                regenerated.to_bytes()
            })
            .collect()
    } else {
        header
            .entries
            .iter()
            .map(|entry| {
                let child = NodePointer::read_judging_the_encryption_compression_and_extent_offset_fields_from(
                    &mut ByteReader::at(entry, header.key_width),
                )
                .expect("内部条目里的子指针读得出");
                let copied_child = copy_the_subtree_under_another_generation(
                    image,
                    &child,
                    generation_offset,
                    parameters,
                    free_slots,
                    unreleased_record_keys,
                );
                let mut writer = ByteWriter::new(header.entry_width);
                writer.put(&entry[..header.key_width]);
                copied_child.write_to(&mut writer);
                writer.into_bytes()
            })
            .collect()
    };
    let copied = rebuilt_node(&header, &entries, parameters);
    write_to_a_free_slot(image, pointer, &copied, free_slots)
}

/// 把 `checkpoint_txg` 那条根（根环里发布这一版时写的那一格）的分配记录树整棵照抄、每条记录的代号加 `generation_offset`，
/// 树表抄一份改指新树，根槽改指新树表、重封自证校验和（`RootRecord::to_slot`）。原来的树、树表不动。
/// 交回抄出来的树里每条未释放记录的键（代号已加偏移）。
///
/// # Panics
/// 根环里那一格解不出这一版的根；树表解不开、里面没有分配记录树；照抄不改时重建出来的树表、条目或根槽与盘上原样不同；
/// 空槽不空。
#[must_use]
pub fn copy_the_allocation_record_tree_of_an_old_root_under_another_generation(
    image: &mut MemoryPool,
    parameters: &MakeFilesystemParameters,
    checkpoint_txg: CheckpointTxg,
    generation_offset: u64,
    free_slots: &mut FreeSlotsForTheCopies,
) -> BTreeSet<AllocationRecordKey> {
    let target = target_for_publish(
        checkpoint_txg,
        parameters.geometry.root_ring_slots_per_region,
    );
    let root_device = parameters.region_devices[usize::try_from(target.region).expect("区域号")];
    let root_offset = slot_offset(target, parameters.geometry.fixed_structure_slot_spacing);
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    let original_root_slot = image
        .devices
        .get(&root_device)
        .expect("根环那一格所在的盘在池里")
        .read(root_offset, root_slot_bytes);
    let mut root = RootRecord::parse_slot(&original_root_slot, &parameters.filesystem_identifier)
        .unwrap_or_else(|| panic!("根环里 txg {} 那一格解得出根", checkpoint_txg.0));
    assert_eq!(
        root.checkpoint_txg, checkpoint_txg,
        "根环里那一格是这一版的根（环还没转回来盖掉它）"
    );
    assert_eq!(
        root.to_slot(root_slot_bytes),
        original_root_slot,
        "照抄不改时重建出来的根槽与盘上原样逐字节相同"
    );
    let original_tree_table = read_node(image, &root.tree_table);
    let tree_table = parse_index_node(&original_tree_table).expect("树表解得开");
    let mut unreleased_record_keys = BTreeSet::new();
    let mut allocation_record_tree_was_copied = false;
    let mut entries = Vec::with_capacity(tree_table.entries.len());
    for entry in &tree_table.entries {
        let parsed = TreeTableEntry::parse(entry).expect("树表条目解得开");
        assert_eq!(
            &parsed.to_bytes(),
            entry,
            "照抄不改时重建出来的树表条目与盘上原样逐字节相同"
        );
        if parsed.kind != TREE_KIND_ALLOCATION {
            entries.push(entry.clone());
            continue;
        }
        let copied_root = copy_the_subtree_under_another_generation(
            image,
            &parsed.root,
            generation_offset,
            parameters,
            free_slots,
            &mut unreleased_record_keys,
        );
        allocation_record_tree_was_copied = true;
        entries.push(
            TreeTableEntry {
                kind: parsed.kind,
                tree: parsed.tree,
                root: copied_root,
                birth_txg: parsed.birth_txg,
                head_identifier: parsed.head_identifier,
            }
            .to_bytes(),
        );
    }
    assert!(
        allocation_record_tree_was_copied,
        "txg {} 那一版的树表里有分配记录树",
        checkpoint_txg.0
    );
    assert_eq!(
        rebuilt_node(&tree_table, &tree_table.entries, parameters),
        original_tree_table,
        "照抄不改时重建出来的树表与盘上原样逐字节相同"
    );
    let copied_tree_table = rebuilt_node(&tree_table, &entries, parameters);
    root.tree_table = write_to_a_free_slot(image, &root.tree_table, &copied_tree_table, free_slots);
    image
        .devices
        .get_mut(&root_device)
        .expect("根环那一格所在的盘在池里")
        .write(root_offset, &root.to_slot(root_slot_bytes));
    unreleased_record_keys
}

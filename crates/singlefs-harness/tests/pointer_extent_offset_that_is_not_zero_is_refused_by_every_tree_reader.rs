//! 指针头部 extent 偏移那 2 字节第一版恒 0，**读指针头部的每一个 core 读者**遇到非 0 都判该指针所在的结构损坏，同 MAC / nonce
//! （D19（块指针的结构与宽度预算） 已定项 3；判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行、
//! C581（extent 偏移非 0 没有判据））。根记录、多层码 2 树内部条目与 journal 新根段那三类读者另有用例
//! （`pointer_extent_offset_that_is_not_zero_is_refused_by_both_mounts.rs`、`corrupt_on_disk_content_is_refused_instead_of_panicking.rs`）；
//! 这里一类读者一条用例：树表条目的根指针、inode 树内部条目的子指针、extent 上段叶条目里内联的数据指针与下段根指针、
//! extent 叶记录的数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针。
//! 影子账认单元的那两个读者（分配记录树的 `node_pointers_as_far_as_readable`、实例表链的 `instance_table_page_pointers_as_far_as_readable`）
//! 遇到 extent 偏移非 0 照旧认它指的孩子：影子账要的是多隔离、不漏隔离（主 agent 2026-10-03 判，`records/2026-09-30-里程碑三开工第一轮.md`
//! 第 8 批 core 簇第 7 条），在分配记录树与实例表链那两条里一并钉住。
//!
//! 坏镜像只改一处：在发布路径写出的一版上找到那条指针所在的单元，把那条指针头部偏移 32 的 extent 偏移由 0 改成 1、重封这个单元，
//! 新的整单元校验和沿引用链补上去（父单元里的位置条目、中央映射条目、根槽，`propagate_the_new_unit_checksum`），
//! 再把每条根抄进它那几条 journal 记录的新根段；读者走到被改的那条指针才判得到。每条用例先在改之前的同一版上核：读者照常读出。
//! 改之前这些读者跳过这 2 字节、照常读出（正推腿 `research/prompts/m3-batch4-code-r1-sonnet-output.md` 4.3 节那份镜像：
//! extent 树根里那条数据指针 extent 偏移改 1，只读挂载与读第一个字节都成功）。

mod common;

use common::{
    build_pool, copy_each_root_into_the_new_root_segment_of_its_journal_records,
    memory_pool_of_sparse_devices, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, IMAGE_BYTES,
};
use singlefs_core::address::{
    DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
};
use singlefs_core::allocation_record_tree::{
    node_pointers_as_far_as_readable, AllocationRecordTreeGeometry,
    ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::{ByteReader, ByteWriter};
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::extent_tree::{
    ExtentUpperLeafEntry, ExtentUpperLeafTarget,
    EXTENT_LEAF_RECORD_DATA_POINTER_EXTENT_OFFSET_NOT_ZERO,
    EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
    EXTENT_UPPER_LEAF_ENTRY_POINTER_EXTENT_OFFSET_NOT_ZERO,
};
use singlefs_core::instance_table::{
    InstanceTableChainRecord, InstanceTablePage, InstanceTablePageIndex,
};
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::mounted_read::{
    mount_read_only, MountReadOnlyFailure, OpenFileFailure, OpenPoolForReadFailure,
};
use singlefs_core::pointer::{
    DataPointer, NodePointer, PointerHeadExtentOffsetNotZero,
    PointerHeadFieldOutsideTheFirstVersion,
};
use singlefs_core::records::{
    parse_extent_record, parse_inode_internal_entry, TreeTableEntry, TREE_KIND_ALLOCATION,
    TREE_KIND_EXTENT, TREE_KIND_INODE,
};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, instance_table_chain_of_root,
    instance_table_page_pointers_as_far_as_readable, recover, JournalPolicy, RecoveryFailure,
    RecoveryOutcome, INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
};
use singlefs_core::root_record::{RootRecord, ROOT_CHECKSUM_OFFSET};
use singlefs_core::root_ring::slot_offset;
use singlefs_core::transaction::{acquire_instance, PoolWriter, FIRST_INODE_NUMBER};
use singlefs_core::unit::{
    data_unit_payload_capacity, parse_data_unit, parse_index_node, parse_packed_unit,
    seal_header_checksum, UNIT_CLASS_DATA, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
};
use singlefs_format::{
    DATA_POINTER_BYTES, DATA_UNIT_BYTES, NODE_BYTES, NODE_POINTER_BYTES, SLOT_BYTES,
    UNIT_AREA_START_SLOT,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::scenario::run_new_pool_file_creation;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 指针头部里 extent 偏移那 2 字节的偏移：MAC 16、nonce 12、算法类型 1、压缩算法码 1、压后长度 2 之后（D19（块指针的结构与宽度预算） 已定项 7 的字段表）。
const EXTENT_OFFSET_IN_THE_POINTER_HEAD: usize = 32;
/// 单元头第 6 个字节是单元类（共同前缀：magic 4、格式版本 2 之后）。
const UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER: usize = 6;
/// 根槽宽 = physical_block_size（`common::parameters` 取 512）。
const ROOT_SLOT_BYTES: usize = 512;
/// 一片实例表里数据行的上限：370 条记录减去链指针那一条（D18（块里携带什么信息） 已定项 11）。
const INSTANCE_ROWS_PER_PAGE: u32 = 369;

fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

fn data_unit_bytes() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
}

/// 盘 0 上从 `slot` 起 `length_in_bytes` 字节（第一版两盘同槽同内容，读一份就够）。
fn unit_bytes_at(image: &MemoryPool, slot: SlotNumber, length_in_bytes: usize) -> Vec<u8> {
    image
        .devices
        .get(&DeviceIdentity(0))
        .expect("池里有盘 0")
        .read(slot.to_device_offset(), length_in_bytes)
}

/// 一个单元按单元类的整单元宽；单元类不认识（槽里不是单元的开头）时是 None。
fn unit_bytes_of_class(unit_class: u8) -> Option<usize> {
    match unit_class {
        UNIT_CLASS_DATA | UNIT_CLASS_PACKED => Some(data_unit_bytes()),
        UNIT_CLASS_INDEX_NODE => Some(node_bytes()),
        _not_a_unit_class => None,
    }
}

/// 改过的单元重封：载荷 CRC 与头校验和（码 1 罩 [105, 32768)、住 101，头 [0, 105)；码 3 罩 [107, 32768)、住 89，头 [0, 107)；
/// 码 2 罩 [86 + 2k, 16384)、住 76 + 2k，头 [0, 86 + 2k)，k 是 key 宽、住偏移 51）。
fn reseal_unit(unit: &mut [u8]) {
    let (header_end, payload_checksum_offset) = match unit[UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER] {
        UNIT_CLASS_DATA => (105, 101),
        UNIT_CLASS_PACKED => (107, 89),
        UNIT_CLASS_INDEX_NODE => {
            let key_width = usize::from(unit[51]);
            (86 + 2 * key_width, 76 + 2 * key_width)
        }
        unknown_class => panic!("只重封三类单元，读到类 {unknown_class}"),
    };
    let payload_checksum = crc32_castagnoli(&unit[header_end..]);
    unit[payload_checksum_offset..payload_checksum_offset + 4]
        .copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(unit, header_end);
}

/// 单元区里盘 0 上写过的每个单元（起始槽、整单元宽）：按槽号升序逐槽看单元头，按单元类取宽、解得开才算一个单元，32 KiB 的单元跳过它的第二个槽。
fn units_in_the_unit_area(image: &MemoryPool) -> Vec<(SlotNumber, usize)> {
    let slot_bytes = SLOT_BYTES;
    let unit_area_start = DeviceOffsetInBytes(UNIT_AREA_START_SLOT * slot_bytes);
    let device = image.devices.get(&DeviceIdentity(0)).expect("池里有盘 0");
    let written_slots: std::collections::BTreeSet<u64> = device
        .written_sectors_in(unit_area_start, IMAGE_BYTES - unit_area_start.0)
        .into_iter()
        .map(|sector| sector * 512 / slot_bytes)
        .collect();
    let mut units = Vec::new();
    let mut first_slot_not_yet_covered = UNIT_AREA_START_SLOT;
    // 迭代上界是写过的槽数；跨轮携带的是上一个单元之后的第一个槽（32 KiB 单元的第二个槽不当单元的开头看）。
    for slot in written_slots {
        if slot < first_slot_not_yet_covered {
            continue;
        }
        let header = unit_bytes_at(image, SlotNumber(slot), 512);
        let Some(length_in_bytes) =
            unit_bytes_of_class(header[UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER])
        else {
            continue;
        };
        let bytes = unit_bytes_at(image, SlotNumber(slot), length_in_bytes);
        let is_a_unit = match header[UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER] {
            UNIT_CLASS_DATA => parse_data_unit(&bytes).is_ok(),
            UNIT_CLASS_PACKED => parse_packed_unit(&bytes).is_ok(),
            _index_node => parse_index_node(&bytes).is_ok(),
        };
        if is_a_unit {
            units.push((SlotNumber(slot), length_in_bytes));
            first_slot_not_yet_covered =
                slot + u64::try_from(length_in_bytes).expect("单元宽") / slot_bytes;
        }
    }
    units
}

/// 位置条目 14 字节的盘上样子：设备身份 4、槽号 6、整单元校验和 4。
fn location_entry_bytes(device: DeviceIdentity, slot: SlotNumber, unit_checksum: u32) -> Vec<u8> {
    [
        device.0.to_le_bytes().to_vec(),
        slot.0.to_le_bytes()[..6].to_vec(),
        unit_checksum.to_le_bytes().to_vec(),
    ]
    .concat()
}

/// 把 `container` 里每一处 `from` 换成 `to`（等长），交回换了几处。
fn replace_every_occurrence(container: &mut [u8], from: &[u8], to: &[u8]) -> usize {
    assert_eq!(from.len(), to.len(), "等长替换");
    let mut replaced = 0;
    let mut position = 0;
    // 迭代上界是容器的字节数；跨轮携带的是下一个要比的位置（换过的那一段整段跳过）。
    while position + from.len() <= container.len() {
        if container[position..position + from.len()] == *from {
            container[position..position + from.len()].copy_from_slice(to);
            replaced += 1;
            position += from.len();
        } else {
            position += 1;
        }
    }
    replaced
}

/// 根环里每个根槽（盘、偏移）。
fn root_slots(image: &MemoryPool) -> Vec<(DeviceIdentity, DeviceOffsetInBytes)> {
    let system_configuration = choose_system_configuration(image).expect("系统配置择得到");
    every_root_ring_slot(&system_configuration.immutable.sizes)
        .into_iter()
        .map(|ring_slot| {
            (
                system_configuration.immutable.region_devices
                    [usize::try_from(ring_slot.region).expect("区域号")],
                slot_offset(
                    ring_slot,
                    system_configuration
                        .immutable
                        .sizes
                        .fixed_structure_slot_spacing,
                ),
            )
        })
        .collect()
}

/// `slot` 那个单元（两盘同槽）的整单元校验和由 `old` 换成了 `new`：引用它的每一条位置条目（单元区里的父单元、中央映射条目、根槽）
/// 换成新的，被换的单元重封、它自己的新校验和再往上补，直到根槽；最后把每条根抄进它那几条 journal 记录的新根段。
fn propagate_the_new_unit_checksum(image: &mut MemoryPool, slot: SlotNumber, old: u32, new: u32) {
    let units = units_in_the_unit_area(image);
    let mut pending = vec![(slot, old, new)];
    // 迭代上界：每换一处，被换的单元的新校验和入队一次；引用链从叶到根是有限的、不成环（单元只被更晚写出的单元引用）。
    while let Some((changed_slot, old_checksum, new_checksum)) = pending.pop() {
        if old_checksum == new_checksum {
            continue;
        }
        for (container_slot, length_in_bytes) in &units {
            if *container_slot == changed_slot {
                continue;
            }
            let before = unit_bytes_at(image, *container_slot, *length_in_bytes);
            let mut after = before.clone();
            let mut replaced = 0;
            for device in [DeviceIdentity(0), DeviceIdentity(1)] {
                replaced += replace_every_occurrence(
                    &mut after,
                    &location_entry_bytes(device, changed_slot, old_checksum),
                    &location_entry_bytes(device, changed_slot, new_checksum),
                );
            }
            if replaced > 0 {
                reseal_unit(&mut after);
                for device in image.devices.values_mut() {
                    device.write(container_slot.to_device_offset(), &after);
                }
                pending.push((
                    *container_slot,
                    crc32_castagnoli(&before),
                    crc32_castagnoli(&after),
                ));
            }
        }
        for (device, offset) in root_slots(image) {
            let sparse = image.devices.get_mut(&device).expect("根槽所在的盘在池里");
            let mut root_slot = sparse.read(offset, ROOT_SLOT_BYTES);
            let mut replaced = 0;
            for location_device in [DeviceIdentity(0), DeviceIdentity(1)] {
                replaced += replace_every_occurrence(
                    &mut root_slot,
                    &location_entry_bytes(location_device, changed_slot, old_checksum),
                    &location_entry_bytes(location_device, changed_slot, new_checksum),
                );
            }
            if replaced > 0 {
                let digest = wide_checksum_with_field_zeroed(
                    &root_slot,
                    ROOT_SLOT_BYTES,
                    ROOT_CHECKSUM_OFFSET,
                );
                root_slot[ROOT_CHECKSUM_OFFSET..ROOT_CHECKSUM_OFFSET + 32].copy_from_slice(&digest);
                sparse.write(offset, &root_slot);
            }
        }
    }
    copy_each_root_into_the_new_root_segment_of_its_journal_records(image);
}

/// `container` 那个单元里恰好一处是 `pointer_bytes`（一条指针的盘上原样）：把它头部的 extent 偏移由 0 改成 1、重封、写回两盘，
/// 新的整单元校验和沿引用链补上去。
fn set_the_extent_offset_of_the_pointer_inside_the_unit_to_one(
    image: &mut MemoryPool,
    container: SlotNumber,
    container_bytes: usize,
    pointer_bytes: &[u8],
) {
    let before = unit_bytes_at(image, container, container_bytes);
    let occurrences: Vec<usize> = (0..=before.len() - pointer_bytes.len())
        .filter(|start| before[*start..*start + pointer_bytes.len()] == *pointer_bytes)
        .collect();
    assert_eq!(
        occurrences.len(),
        1,
        "槽 {container:?} 那个单元里这条指针恰好出现一次"
    );
    let extent_offset_at = occurrences[0] + EXTENT_OFFSET_IN_THE_POINTER_HEAD;
    assert_eq!(
        before[extent_offset_at..extent_offset_at + 2],
        [0, 0],
        "第一版写者写的 extent 偏移是 0"
    );
    let mut after = before.clone();
    after[extent_offset_at..extent_offset_at + 2].copy_from_slice(&1u16.to_le_bytes());
    reseal_unit(&mut after);
    for device in image.devices.values_mut() {
        device.write(container.to_device_offset(), &after);
    }
    propagate_the_new_unit_checksum(
        image,
        container,
        crc32_castagnoli(&before),
        crc32_castagnoli(&after),
    );
}

fn node_pointer_bytes(pointer: &NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(NODE_POINTER_BYTES).expect("86"));
    pointer.write_to(&mut writer);
    writer.into_bytes()
}

fn data_pointer_bytes(pointer: &DataPointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(DATA_POINTER_BYTES).expect("88"));
    pointer.write_to(&mut writer);
    writer.into_bytes()
}

fn the_slot_of(pointer: &NodePointer) -> SlotNumber {
    assert_eq!(
        pointer.locations[0].slot, pointer.locations[1].slot,
        "第一版两盘同槽"
    );
    pointer.locations[0].slot
}

/// 择根择到的那条根（只读恢复与挂载择的是它）。
fn newest_root(image: &MemoryPool) -> RootRecord {
    let system_configuration = choose_system_configuration(image).expect("系统配置择得到");
    choose_root(image, &system_configuration).expect("择得到根")
}

/// 最新那条根的树表里种类为 `kind` 的那条条目（树表单元按根记录里的树表指针读）。
fn tree_table_entry_of_kind(image: &MemoryPool, kind: u16) -> TreeTableEntry {
    let root = newest_root(image);
    let tree_table = parse_index_node(&unit_bytes_at(
        image,
        the_slot_of(&root.tree_table),
        node_bytes(),
    ))
    .expect("树表单元解得开");
    tree_table
        .entries
        .iter()
        .filter_map(|entry| TreeTableEntry::parse(entry))
        .find(|entry| entry.kind == kind)
        .expect("树表里有这种树")
}

/// 一个码 2 节点第 0 条条目。
fn first_entry_of_the_node_at(image: &MemoryPool, pointer: &NodePointer) -> Vec<u8> {
    parse_index_node(&unit_bytes_at(image, the_slot_of(pointer), node_bytes()))
        .expect("节点解得开")
        .entries
        .first()
        .expect("节点里至少一条条目")
        .clone()
}

/// extent 树根（第一版只有一片上段叶：根兼叶）第 0 条上段叶条目。
fn first_upper_leaf_entry(image: &MemoryPool) -> ExtentUpperLeafEntry {
    let extent_root = tree_table_entry_of_kind(image, TREE_KIND_EXTENT).root;
    ExtentUpperLeafEntry::parse(&first_entry_of_the_node_at(image, &extent_root))
        .expect("上段叶条目解得开")
}

/// 只读挂载照常挂上，打开第一个文件被拒：交回打开那一步的错。
fn open_the_first_file_refused(image: &MemoryPool) -> OpenFileFailure {
    let mounted = match mount_read_only(image) {
        Ok(mounted) => mounted,
        Err(failure) => panic!("只读挂载那一步不读 extent 树，要挂得上：{failure:?}"),
    };
    match mounted
        .mounted
        .open_file(image, InodeNumber(FIRST_INODE_NUMBER))
    {
        Ok(_file) => panic!("extent 树里那条指针头部 extent 偏移非 0，打开文件却成功了"),
        Err(failure) => failure,
    }
}

/// 只读挂载、打开第一个文件、读第一个字节都成功：交回读到的字节数。
fn first_byte_of_the_first_file_read(image: &MemoryPool) -> usize {
    let mounted = match mount_read_only(image) {
        Ok(mounted) => mounted,
        Err(failure) => panic!("改之前只读挂载要挂得上：{failure:?}"),
    };
    let file = mounted
        .mounted
        .open_file(image, InodeNumber(FIRST_INODE_NUMBER))
        .expect("改之前打开第一个文件");
    file.read_at(image, FileOffsetInBytes(0), 1)
        .expect("改之前读第一个字节")
        .bytes
        .len()
}

/// 冷启动恢复报的那个失败成员（恢复沿择到的根走读，走到被改的那条指针）。
fn recovery_failure(image: &MemoryPool) -> RecoveryFailure {
    match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::Failed { failure, .. } => failure,
        RecoveryOutcome::FileRead { .. } | RecoveryOutcome::NoFile { .. } => {
            panic!("被改的那条指针在冷启动走读的路上，恢复却成功了")
        }
    }
}

type WritableDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

fn writable_devices_of(image: &MemoryPool) -> WritableDevices {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device = SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

/// 可写挂载被拒、盘上逐字节不变：交回拒的那个错。
fn writable_mount_refused_before_any_write(image: &MemoryPool) -> MountError {
    let mut devices = writable_devices_of(image);
    let refusal = match mount_writable(&parameters(), &mut devices) {
        Ok(mounted) => panic!("坏镜像被可写挂载收下了：实例 {:?}", mounted.output.instance),
        Err(refusal) => refusal,
    };
    for (identity, device) in &devices {
        assert!(
            device.image == image.devices[identity],
            "可写挂载在任何写之前拒：盘 {identity:?} 逐字节不变（{refusal:?}）"
        );
    }
    refusal
}

/// 新池新建文件之后再顺序写一版 `data_units` 个数据单元的文件（最后一个装一半）。
fn image_after_a_sequential_write_of(tag: &str, data_units: usize) -> MemoryPool {
    let mut pool = build_pool(tag);
    let payload_capacity = data_unit_payload_capacity();
    let content: Vec<u8> = (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 31 + 7) % 251).expect("小于 256"))
        .collect();
    sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    pool.memory_pool()
}

/// 场景：新池新建文件那一版，树表里 inode 树那条条目的根指针头部 extent 偏移改成 1（树表单元重封、根记录补校验和）。
/// 预期：冷启动恢复与可写挂载都报「树表条目」解不开（`UnitMalformed`），可写挂载在任何写之前拒；只读挂载报树表条目解不开。
/// 改之前 `TreeTableEntry::parse` 跳过这 2 字节，三个入口都照常走下去。
#[test]
fn a_tree_table_entry_whose_root_pointer_carries_a_non_zero_extent_offset_is_refused() {
    let mut image = build_pool("extent-offset-tree-table-entry").memory_pool();
    assert!(
        matches!(
            recover(&image, JournalPolicy::Consult).outcome,
            RecoveryOutcome::FileRead { .. }
        ),
        "改之前冷启动读得出文件"
    );
    let tree_table = newest_root(&image).tree_table;
    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    set_the_extent_offset_of_the_pointer_inside_the_unit_to_one(
        &mut image,
        the_slot_of(&tree_table),
        node_bytes(),
        &node_pointer_bytes(&inode_root),
    );
    let tree_table_entry_malformed = RecoveryFailure::UnitMalformed {
        what: "树表条目"
    };
    assert_eq!(
        recovery_failure(&image),
        tree_table_entry_malformed,
        "冷启动恢复：树表条目解不开"
    );
    assert!(
        matches!(
            mount_read_only(&image),
            Err(MountReadOnlyFailure::Open(
                OpenPoolForReadFailure::TreeTableEntryMalformed
            ))
        ),
        "只读挂载：打开挂载态时树表条目解不开"
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(&refusal, MountError::Recovery(failure) if *failure == tree_table_entry_malformed),
        "可写挂载：树表条目解不开，实际 {refusal:?}"
    );
}

/// 场景：新池新建文件那一版，inode 树根第 0 条内部条目的子指针头部 extent 偏移改成 1（inode 树根重封，树表、中央映射、根记录补校验和）。
/// 预期：冷启动恢复报 `UnitMalformed`（[`INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO`]），只读挂载报 inode 树内部条目的子指针
/// 不能解引用（`PointerInTheRecordCannotBeDereferenced`，与条目宽窄于字段表的 `RecordMalformed` 分开报）。
/// 改之前两个读者都跳过这 2 字节、照常读出文件。
#[test]
fn an_inode_internal_entry_whose_child_pointer_carries_a_non_zero_extent_offset_is_refused() {
    let mut image = build_pool("extent-offset-inode-internal-entry").memory_pool();
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "改之前读得出第一个字节"
    );
    let inode_root = tree_table_entry_of_kind(&image, TREE_KIND_INODE).root;
    let (_separator_key, _identity, child) =
        parse_inode_internal_entry(&first_entry_of_the_node_at(&image, &inode_root))
            .expect("改之前 inode 内部条目解得开");
    set_the_extent_offset_of_the_pointer_inside_the_unit_to_one(
        &mut image,
        the_slot_of(&inode_root),
        node_bytes(),
        &node_pointer_bytes(&child),
    );
    assert_eq!(
        recovery_failure(&image),
        RecoveryFailure::UnitMalformed {
            what: INODE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
        },
        "冷启动恢复：inode 内部条目的子指针 extent 偏移非 0"
    );
    assert!(
        matches!(
            mount_read_only(&image),
            Err(MountReadOnlyFailure::Open(
                OpenPoolForReadFailure::PointerInTheRecordCannotBeDereferenced {
                    what: "inode 树内部条目的子指针",
                    field: PointerHeadFieldOutsideTheFirstVersion::ExtentOffsetNotZero(
                        PointerHeadExtentOffsetNotZero { extent_offset: 1 }
                    ),
                }
            ))
        ),
        "只读挂载：inode 树内部条目的子指针不能解引用"
    );
}

/// 场景：正推腿那一份镜像——新池新建文件那一版（3000 字节，一个内联的数据单元），extent 树根里那条上段叶条目内联的数据指针
/// 头部 extent 偏移改成 1（extent 树根重封，树表、中央映射、根记录补校验和）。
/// 预期：只读挂载照常挂上，打开文件报 extent 树走读失败（`UnitMalformed`，[`EXTENT_UPPER_LEAF_ENTRY_POINTER_EXTENT_OFFSET_NOT_ZERO`]）；
/// 冷启动恢复报同一个成员。改之前只读挂载与读第一个字节都成功（正推腿 4.3 节）。
#[test]
fn an_extent_upper_leaf_entry_whose_inline_data_pointer_carries_a_non_zero_extent_offset_is_refused(
) {
    let mut image = build_pool("extent-offset-inline-data-pointer").memory_pool();
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "改之前读得出第一个字节"
    );
    let ExtentUpperLeafTarget::InlineDataUnit(data_pointer) = first_upper_leaf_entry(&image).target
    else {
        panic!("3000 字节的文件是一个内联的数据单元（标签 2）");
    };
    let extent_root = tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
    set_the_extent_offset_of_the_pointer_inside_the_unit_to_one(
        &mut image,
        the_slot_of(&extent_root),
        node_bytes(),
        &data_pointer_bytes(&data_pointer),
    );
    let upper_leaf_entry_refused = RecoveryFailure::UnitMalformed {
        what: EXTENT_UPPER_LEAF_ENTRY_POINTER_EXTENT_OFFSET_NOT_ZERO,
    };
    assert_eq!(
        open_the_first_file_refused(&image),
        OpenFileFailure::ExtentTreeWalk(upper_leaf_entry_refused.clone()),
        "打开文件：上段叶条目内联的数据指针 extent 偏移非 0"
    );
    assert_eq!(
        recovery_failure(&image),
        upper_leaf_entry_refused,
        "冷启动恢复：同一个成员"
    );
}

/// 场景：新池新建文件之后顺序写成两个数据单元的文件，上段叶条目（标签 1）里下段根指针头部 extent 偏移改成 1。
/// 预期：打开文件与冷启动恢复都报 `UnitMalformed`（[`EXTENT_UPPER_LEAF_ENTRY_POINTER_EXTENT_OFFSET_NOT_ZERO`]）。改之前照常读出。
#[test]
fn an_extent_upper_leaf_entry_whose_lower_segment_root_pointer_carries_a_non_zero_extent_offset_is_refused(
) {
    let mut image = image_after_a_sequential_write_of("extent-offset-lower-segment-root", 2);
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "改之前读得出第一个字节"
    );
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&image).target
    else {
        panic!("两个数据单元的文件在上段叶里是一条标签 1 的条目");
    };
    let extent_root = tree_table_entry_of_kind(&image, TREE_KIND_EXTENT).root;
    set_the_extent_offset_of_the_pointer_inside_the_unit_to_one(
        &mut image,
        the_slot_of(&extent_root),
        node_bytes(),
        &node_pointer_bytes(&lower_root),
    );
    let upper_leaf_entry_refused = RecoveryFailure::UnitMalformed {
        what: EXTENT_UPPER_LEAF_ENTRY_POINTER_EXTENT_OFFSET_NOT_ZERO,
    };
    assert_eq!(
        open_the_first_file_refused(&image),
        OpenFileFailure::ExtentTreeWalk(upper_leaf_entry_refused.clone()),
        "打开文件：下段根指针 extent 偏移非 0"
    );
    assert_eq!(
        recovery_failure(&image),
        upper_leaf_entry_refused,
        "冷启动恢复：同一个成员"
    );
}

/// 场景：两个数据单元的文件，下段只有一片叶（根兼叶），它第 0 条叶记录的数据指针头部 extent 偏移改成 1。
/// 预期：打开文件与冷启动恢复都报 `UnitMalformed`（[`EXTENT_LEAF_RECORD_DATA_POINTER_EXTENT_OFFSET_NOT_ZERO`]），
/// 不再报成「条目窄于字段表」。改之前照常读出。
#[test]
fn an_extent_leaf_record_whose_data_pointer_carries_a_non_zero_extent_offset_is_refused() {
    let mut image = image_after_a_sequential_write_of("extent-offset-leaf-record", 2);
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "改之前读得出第一个字节"
    );
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&image).target
    else {
        panic!("两个数据单元的文件在上段叶里是一条标签 1 的条目");
    };
    let (_key, data_pointer) =
        parse_extent_record(&first_entry_of_the_node_at(&image, &lower_root))
            .expect("改之前叶记录解得开");
    set_the_extent_offset_of_the_pointer_inside_the_unit_to_one(
        &mut image,
        the_slot_of(&lower_root),
        node_bytes(),
        &data_pointer_bytes(&data_pointer),
    );
    let leaf_record_refused = RecoveryFailure::UnitMalformed {
        what: EXTENT_LEAF_RECORD_DATA_POINTER_EXTENT_OFFSET_NOT_ZERO,
    };
    assert_eq!(
        open_the_first_file_refused(&image),
        OpenFileFailure::ExtentTreeWalk(leaf_record_refused.clone()),
        "打开文件：叶记录的数据指针 extent 偏移非 0"
    );
    assert_eq!(
        recovery_failure(&image),
        leaf_record_refused,
        "冷启动恢复：同一个成员"
    );
}

/// 下段两层的文件有几个数据单元：比一片下段叶装得下的 144 个多一个（D8（核心索引结构） 已定项 14「实现取值」）。
const DATA_UNITS_OF_A_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT: usize = 145;

/// 场景：145 个数据单元的文件，下段两片叶上面长出第 1 层的根；下段根第 0 条内部条目的子指针头部 extent 偏移改成 1。
/// 预期：打开文件与冷启动恢复都报 `UnitMalformed`（[`EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO`]）。改之前照常读出。
#[test]
fn an_extent_internal_entry_whose_child_pointer_carries_a_non_zero_extent_offset_is_refused() {
    let mut image = image_after_a_sequential_write_of(
        "extent-offset-extent-internal-entry",
        DATA_UNITS_OF_A_FILE_WITH_A_TWO_LEVEL_LOWER_SEGMENT,
    );
    assert_eq!(
        first_byte_of_the_first_file_read(&image),
        1,
        "改之前读得出第一个字节"
    );
    let ExtentUpperLeafTarget::LowerSegmentRoot(lower_root) = first_upper_leaf_entry(&image).target
    else {
        panic!("145 个数据单元的文件在上段叶里是一条标签 1 的条目");
    };
    let lower_root_bytes = unit_bytes_at(&image, the_slot_of(&lower_root), node_bytes());
    assert_eq!(
        parse_index_node(&lower_root_bytes)
            .expect("下段根解得开")
            .level,
        1,
        "145 个单元：下段根在第 1 层"
    );
    let first_child = NodePointer::read_from(&mut ByteReader::at(
        &first_entry_of_the_node_at(&image, &lower_root),
        24,
    ));
    set_the_extent_offset_of_the_pointer_inside_the_unit_to_one(
        &mut image,
        the_slot_of(&lower_root),
        node_bytes(),
        &node_pointer_bytes(&first_child),
    );
    let internal_entry_refused = RecoveryFailure::UnitMalformed {
        what: EXTENT_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO,
    };
    assert_eq!(
        open_the_first_file_refused(&image),
        OpenFileFailure::ExtentTreeWalk(internal_entry_refused.clone()),
        "打开文件：下段内部条目的子指针 extent 偏移非 0"
    );
    assert_eq!(
        recovery_failure(&image),
        internal_entry_refused,
        "冷启动恢复：同一个成员"
    );
}

/// 一棵分配记录树从根往下认得出的节点指针（影子账认被抛弃根引用的单元时用的那个读者），节点按指针里第一条位置条目从盘 0 读。
fn allocation_record_tree_nodes_as_far_as_readable(
    image: &MemoryPool,
    root_pointer: &NodePointer,
) -> Vec<NodePointer> {
    node_pointers_as_far_as_readable(
        root_pointer,
        &AllocationRecordTreeGeometry::of_reader(image),
        &mut |pointer: &NodePointer| {
            Some(unit_bytes_at(
                image,
                pointer.locations[0].slot,
                node_bytes(),
            ))
        },
    )
}

/// 场景：新池新建文件那一版，分配记录树根（层级由池几何定，第一版两块 4 GiB 盘上在第 1 层以上）第 0 条内部条目的子指针头部
/// extent 偏移改成 1。
/// 预期：可写挂载在任何写之前拒，报 `UnitMalformed`（[`ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO`]）；
/// 影子账认单元的那个读者照旧认它指的孩子（认得出的节点与改之前一样多，那个孩子在其中）。改之前可写挂载照常挂上。
#[test]
fn an_allocation_record_tree_internal_entry_whose_child_pointer_carries_a_non_zero_extent_offset_is_refused(
) {
    let mut image = build_pool("extent-offset-allocation-internal-entry").memory_pool();
    let allocation_root = tree_table_entry_of_kind(&image, TREE_KIND_ALLOCATION).root;
    let nodes_before = allocation_record_tree_nodes_as_far_as_readable(&image, &allocation_root);
    let root_entry = first_entry_of_the_node_at(&image, &allocation_root);
    assert!(
        parse_index_node(&unit_bytes_at(
            &image,
            the_slot_of(&allocation_root),
            node_bytes()
        ))
        .expect("分配记录树根解得开")
        .level
            >= 1,
        "分配记录树根是内部节点"
    );
    let first_child = NodePointer::read_from(&mut ByteReader::at(&root_entry, 10));
    assert!(
        nodes_before.contains(&first_child),
        "改之前影子账那个读者认得出第 0 条条目指的孩子"
    );
    set_the_extent_offset_of_the_pointer_inside_the_unit_to_one(
        &mut image,
        the_slot_of(&allocation_root),
        node_bytes(),
        &node_pointer_bytes(&first_child),
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(
            &refusal,
            MountError::Recovery(RecoveryFailure::UnitMalformed { what })
                if *what == ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_CHILD_POINTER_EXTENT_OFFSET_NOT_ZERO
        ),
        "可写挂载：分配记录树内部条目的子指针 extent 偏移非 0，实际 {refusal:?}"
    );
    let allocation_root_after = tree_table_entry_of_kind(&image, TREE_KIND_ALLOCATION).root;
    let nodes_after =
        allocation_record_tree_nodes_as_far_as_readable(&image, &allocation_root_after);
    assert!(
        nodes_after.len() == nodes_before.len()
            && nodes_after
                .iter()
                .any(|pointer| pointer.locations[0].slot == first_child.locations[0].slot),
        "影子账那个读者照认那条条目指的孩子（多隔离、不漏隔离）：改之前 {} 个节点，改之后 {} 个",
        nodes_before.len(),
        nodes_after.len()
    );
}

type RecordedSparseDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 两块 4 GiB 内存盘上新池新建文件，再连着 369 次「取号之后崩溃」、可写挂载一次：给 [1, 371) 写 370 行——一片装不下，
/// 实例表链两片（`instance_table_second_page_write.rs` 同一个造法）。
fn image_whose_instance_table_has_two_pages() -> MemoryPool {
    let stream = SharedStream::new();
    let mut devices: RecordedSparseDevices = [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    SparseBlockDevice::new(IMAGE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect();
    run_new_pool_file_creation(&parameters(), &mut devices, &stream, |_, _| {})
        .expect("新池新建文件");
    {
        let publish_parameters = parameters();
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        for _ in 0..INSTANCE_ROWS_PER_PAGE {
            acquire_instance(&mut writer).expect("取号");
        }
    }
    mount_writable(&parameters(), &mut devices).expect("370 行写成两片，可写挂载要成立");
    memory_pool_of_sparse_devices(&devices)
}

/// 场景：实例表链两片的那一版，第 0 片末尾那条链指针记录里下一片指针的头部 extent 偏移改成 1（第 0 片重封，引用它的根记录补校验和）。
/// 预期：沿链读整张实例表报这一片解不开（`UnitMalformed`）；影子账认链上各片的那个读者照认两片（多隔离、不漏隔离）；
/// 可写挂载在任何写之前拒（`InstanceTableMalformed`）。改之前链上两片都认得出、整张表读得出。
#[test]
fn an_instance_table_chain_pointer_carrying_a_non_zero_extent_offset_is_refused() {
    let mut image = image_whose_instance_table_has_two_pages();
    let root = newest_root(&image);
    assert_eq!(
        instance_table_page_pointers_as_far_as_readable(&image, &root).len(),
        2,
        "改之前链上两片都认得出"
    );
    let first_page = unit_bytes_at(&image, the_slot_of(&root.instance_table), data_unit_bytes());
    let InstanceTableChainRecord::NextPage(second_page) =
        InstanceTablePage::parse(&first_page, InstanceTablePageIndex::FIRST)
            .expect("改之前第 0 片解得开")
            .chain
    else {
        panic!("第 0 片的链指针记录是「有下一片」");
    };
    set_the_extent_offset_of_the_pointer_inside_the_unit_to_one(
        &mut image,
        the_slot_of(&root.instance_table),
        data_unit_bytes(),
        &node_pointer_bytes(&second_page),
    );
    let root_after = newest_root(&image);
    assert!(
        matches!(
            instance_table_chain_of_root(&image, &root_after),
            Err(RecoveryFailure::UnitMalformed { .. })
        ),
        "沿链读：第 0 片的链指针记录解不开"
    );
    assert_eq!(
        instance_table_page_pointers_as_far_as_readable(&image, &root_after).len(),
        2,
        "影子账那个读者照认两片：第 1 片那条指针的 extent 偏移非 0 也照它的位置条目认"
    );
    let refusal = writable_mount_refused_before_any_write(&image);
    assert!(
        matches!(&refusal, MountError::InstanceTableMalformed),
        "可写挂载：所选根的实例表沿链有一片解不开，实际 {refusal:?}"
    );
}

//! 改坏一条盘上指针的头部（或它所在节点的条目宽）、沿引用链补校验和的搭建：在发布路径写出的一版上找到那条指针所在的单元，
//! 把它头部的一段由第一版的取值改成别的、重封这个单元，新的整单元校验和沿引用链补上去（父单元里的位置条目、中央映射条目、根槽），
//! 再把每条根抄进它那几条 journal 记录的新根段。做法与 `pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`
//! 里那一份相同，这里按改哪一段字段参数化（那一份只改 extent 偏移）。
//! 用这一份的测试文件同时声明 `mod common;`（这里引它的建池与改记录那几样）。
#![allow(dead_code, reason = "每个测试文件各自只用到其中一部分")]

use singlefs_core::address::{
    DeviceIdentity, DeviceOffsetInBytes, FileOffsetInBytes, InodeNumber, SlotNumber,
};
use singlefs_core::allocation_record_tree::{
    node_pointers_as_far_as_readable, AllocationRecordTreeGeometry,
};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::bytes::ByteWriter;
use singlefs_core::checksum::{crc32_castagnoli, wide_checksum_with_field_zeroed};
use singlefs_core::extent_tree::ExtentUpperLeafEntry;
use singlefs_core::mount::{mount_writable, MountError};
use singlefs_core::mounted_read::{mount_read_only, OpenFileFailure};
use singlefs_core::pointer::{DataPointer, NodePointer};
use singlefs_core::records::{TreeTableEntry, TREE_KIND_EXTENT};
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, every_root_ring_slot, recover, JournalPolicy,
    RecoveryFailure, RecoveryOutcome,
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

use crate::common::{
    build_pool, copy_each_root_into_the_new_root_segment_of_its_journal_records,
    memory_pool_of_sparse_devices, parameters,
    sequential_write_in_process_as_instance_one_and_advance_the_pool, IMAGE_BYTES,
};

/// 单元头第 6 个字节是单元类（共同前缀：magic 4、格式版本 2 之后）。
const UNIT_CLASS_OFFSET_IN_THE_UNIT_HEADER: usize = 6;
/// 根槽宽 = physical_block_size（`common::parameters` 取 512）。
const ROOT_SLOT_BYTES: usize = 512;
/// 一片实例表里数据行的上限：370 条记录减去链指针那一条（D18（块里携带什么信息） 已定项 11）。
const INSTANCE_ROWS_PER_PAGE: u32 = 369;

/// 指针头部里第一版写定值的一段（D19（块指针的结构与宽度预算） 已定项 7 的字段表：MAC 16 / nonce 12 / 算法类型 1 / 压缩算法码 1 / 压后长度 2 /
/// extent 偏移 2），与把它改成的那个非第一版的值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerHeadFieldChange {
    /// MAC 第 0 字节改 1（加密关着时 MAC 恒 0，D19 已定项 3）。
    FirstMacByteToOne,
    /// nonce 第 0 字节改 1（加密关着时 nonce 恒 0，D19 已定项 3）。
    FirstNonceByteToOne,
    /// 算法类型改 1（登记过的加密算法，第一版不支持加密，D9（加密） 已定项 10、D22（单元原子性怎么合成） 已定项 17）。
    AlgorithmTypeToOne,
    /// 压缩算法码改 1（登记表第一版只有码 0，D9（加密） 已定项 14）。
    CompressionAlgorithmCodeToOne,
    /// 压缩算法码仍 0、压后长度改 1（D9（加密） 已定项 14「码 0 时它恒 0，非 0 判该指针损坏」）。
    CompressedLengthToOneWithoutCompression,
}

impl PointerHeadFieldChange {
    /// 这一段在指针头部里的偏移与改成的字节。
    fn offset_and_bytes(self) -> (usize, Vec<u8>) {
        match self {
            PointerHeadFieldChange::FirstMacByteToOne => (0, vec![1]),
            PointerHeadFieldChange::FirstNonceByteToOne => (16, vec![1]),
            PointerHeadFieldChange::AlgorithmTypeToOne => (28, vec![1]),
            PointerHeadFieldChange::CompressionAlgorithmCodeToOne => (29, vec![1]),
            PointerHeadFieldChange::CompressedLengthToOneWithoutCompression => {
                (30, 1u16.to_le_bytes().to_vec())
            }
        }
    }
}

pub fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

pub fn data_unit_bytes() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
}

/// 盘 0 上从 `slot` 起 `length_in_bytes` 字节（第一版两盘同槽同内容，读一份就够）。
pub fn unit_bytes_at(image: &MemoryPool, slot: SlotNumber, length_in_bytes: usize) -> Vec<u8> {
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
    let unit_area_start = DeviceOffsetInBytes(UNIT_AREA_START_SLOT * SLOT_BYTES);
    let device = image.devices.get(&DeviceIdentity(0)).expect("池里有盘 0");
    let written_slots: std::collections::BTreeSet<u64> = device
        .written_sectors_in(unit_area_start, IMAGE_BYTES - unit_area_start.0)
        .into_iter()
        .map(|sector| sector * 512 / SLOT_BYTES)
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
                slot + u64::try_from(length_in_bytes).expect("单元宽") / SLOT_BYTES;
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

/// `container` 那个单元换成 `after`（同宽）：写回两盘，新的整单元校验和沿引用链补上去。`after` 调用方已重封。
pub fn replace_the_unit_and_propagate_its_checksum(
    image: &mut MemoryPool,
    container: SlotNumber,
    after: &[u8],
) {
    let before = unit_bytes_at(image, container, after.len());
    for device in image.devices.values_mut() {
        device.write(container.to_device_offset(), after);
    }
    propagate_the_new_unit_checksum(
        image,
        container,
        crc32_castagnoli(&before),
        crc32_castagnoli(after),
    );
}

/// `container` 那个单元里恰好一处是 `pointer_bytes`（一条指针的盘上原样）：把它头部的 `change` 那一段改掉、重封、写回两盘，
/// 新的整单元校验和沿引用链补上去。
pub fn change_a_pointer_head_field_inside_the_unit(
    image: &mut MemoryPool,
    container: SlotNumber,
    container_bytes: usize,
    pointer_bytes: &[u8],
    change: PointerHeadFieldChange,
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
    let (offset_in_the_head, new_bytes) = change.offset_and_bytes();
    let field_at = occurrences[0] + offset_in_the_head;
    assert!(
        before[field_at..field_at + new_bytes.len()]
            .iter()
            .all(|byte| *byte == 0),
        "第一版写者在这一段写的是 0"
    );
    let mut after = before.clone();
    after[field_at..field_at + new_bytes.len()].copy_from_slice(&new_bytes);
    reseal_unit(&mut after);
    replace_the_unit_and_propagate_its_checksum(image, container, &after);
}

pub fn node_pointer_bytes(pointer: &NodePointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(NODE_POINTER_BYTES).expect("86"));
    pointer.write_to(&mut writer);
    writer.into_bytes()
}

pub fn data_pointer_bytes(pointer: &DataPointer) -> Vec<u8> {
    let mut writer = ByteWriter::new(usize::try_from(DATA_POINTER_BYTES).expect("88"));
    pointer.write_to(&mut writer);
    writer.into_bytes()
}

pub fn the_slot_of(pointer: &NodePointer) -> SlotNumber {
    assert_eq!(
        pointer.locations[0].slot, pointer.locations[1].slot,
        "第一版两盘同槽"
    );
    pointer.locations[0].slot
}

/// 择根择到的那条根（只读恢复与挂载择的是它）。
pub fn newest_root(image: &MemoryPool) -> RootRecord {
    let system_configuration = choose_system_configuration(image).expect("系统配置择得到");
    choose_root(image, &system_configuration).expect("择得到根")
}

/// 最新那条根的树表里种类为 `kind` 的那条条目（树表单元按根记录里的树表指针读）。
pub fn tree_table_entry_of_kind(image: &MemoryPool, kind: u16) -> TreeTableEntry {
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
pub fn first_entry_of_the_node_at(image: &MemoryPool, pointer: &NodePointer) -> Vec<u8> {
    parse_index_node(&unit_bytes_at(image, the_slot_of(pointer), node_bytes()))
        .expect("节点解得开")
        .entries
        .first()
        .expect("节点里至少一条条目")
        .clone()
}

/// extent 树根（第一版只有一片上段叶：根兼叶）第 0 条上段叶条目。
pub fn first_upper_leaf_entry(image: &MemoryPool) -> ExtentUpperLeafEntry {
    let extent_root = tree_table_entry_of_kind(image, TREE_KIND_EXTENT).root;
    ExtentUpperLeafEntry::parse(&first_entry_of_the_node_at(image, &extent_root))
        .expect("上段叶条目解得开")
}

/// 只读挂载照常挂上，打开第一个文件被拒：交回打开那一步的错。
pub fn open_the_first_file_refused(image: &MemoryPool) -> OpenFileFailure {
    let mounted = match mount_read_only(image) {
        Ok(mounted) => mounted,
        Err(failure) => panic!("只读挂载那一步不读 extent 树，要挂得上：{failure:?}"),
    };
    match mounted
        .mounted
        .open_file(image, InodeNumber(FIRST_INODE_NUMBER))
    {
        Ok(_file) => panic!("extent 树里那条指针头部不是第一版的取值，打开文件却成功了"),
        Err(failure) => failure,
    }
}

/// 只读挂载、打开第一个文件、读第一个字节都成功：交回读到的字节数。
pub fn first_byte_of_the_first_file_read(image: &MemoryPool) -> usize {
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
pub fn recovery_failure(image: &MemoryPool) -> RecoveryFailure {
    match recover(image, JournalPolicy::Consult).outcome {
        RecoveryOutcome::Failed { failure, .. } => failure,
        RecoveryOutcome::FileRead { .. } | RecoveryOutcome::NoFile { .. } => {
            panic!("被改的那条指针在冷启动走读的路上，恢复却成功了")
        }
    }
}

pub type WritableDevices = Vec<(DeviceIdentity, SparseBlockDevice)>;

pub fn writable_devices_of(image: &MemoryPool) -> WritableDevices {
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
pub fn writable_mount_refused_before_any_write(image: &MemoryPool) -> MountError {
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
pub fn image_after_a_sequential_write_of(tag: &str, data_units: usize) -> MemoryPool {
    let mut pool = build_pool(tag);
    let payload_capacity = data_unit_payload_capacity();
    let content: Vec<u8> = (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 31 + 7) % 251).expect("小于 256"))
        .collect();
    sequential_write_in_process_as_instance_one_and_advance_the_pool(&mut pool, &content);
    pool.memory_pool()
}

/// 一棵分配记录树从根往下认得出的节点指针（影子账认被抛弃根引用的单元时用的那个读者），节点按指针里第一条位置条目从盘 0 读。
pub fn allocation_record_tree_nodes_as_far_as_readable(
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

type RecordedSparseDevices = Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)>;

/// 两块 4 GiB 内存盘上新池新建文件，再连着 369 次「取号之后崩溃」、可写挂载一次：给 [1, 371) 写 370 行——一片装不下，
/// 实例表链两片（`instance_table_second_page_write.rs` 同一个造法）。
pub fn image_whose_instance_table_has_two_pages() -> MemoryPool {
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

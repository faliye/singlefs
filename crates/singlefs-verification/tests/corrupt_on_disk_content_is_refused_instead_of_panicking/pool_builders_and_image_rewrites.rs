//! 坏盘输入用例共用的造盘与改盘：两块内存稀疏盘上 mkfs、写第一个文件造池，改了盘上字节之后把指着它的校验和一路重封到根记录。

use super::*;

const DEVICE_BYTES: u64 = 4 << 30;

pub(super) const MEBIBYTE: u64 = 1 << 20;

pub(super) const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-rev-a3a";

pub(super) const WRITE_TIME_SECONDS: u64 = 1_788_200_000;

/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
pub(super) const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes =
    PhysicalBlockSizeInBytes(512);

/// 单元区起点之下、根环区域 0（1 MiB 起、8 槽 × 4096）与区域 1（4 MiB 起）之间没人用的一个 16 KiB 槽：
/// 把一个单元的一份拷贝放在这里，读者按位置条目读得到、校验和对得上，槽号却不在单元区里。
pub(super) const SLOT_BELOW_THE_UNIT_AREA_NOBODY_USES: SlotNumber = SlotNumber(100);

pub(super) type Devices = Vec<(DeviceIdentity, SparseBlockDevice)>;

pub(super) fn parameters() -> MakeFilesystemParameters {
    MakeFilesystemParameters {
        filesystem_identifier: FILESYSTEM_IDENTIFIER,
        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
        geometry: SystemImmutableSizes {
            physical_block_size: 512,
            minimum_input_output_bytes: 512,
            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
        },
    }
}

pub(super) fn node_bytes() -> usize {
    usize::try_from(NODE_BYTES).expect("16384")
}

pub(super) fn data_unit_bytes() -> usize {
    usize::try_from(DATA_UNIT_BYTES).expect("32768")
}

fn two_sparse_devices() -> Devices {
    [DeviceIdentity(0), DeviceIdentity(1)]
        .into_iter()
        .map(|identity| {
            (
                identity,
                SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
            )
        })
        .collect()
}

/// 两块内存盘上只做过 mkfs 的池（树表 0 条、没取过号）。
pub(super) fn formatted_pool() -> Devices {
    let mut devices = two_sparse_devices();
    make_filesystem(&parameters(), &mut devices).expect("默认几何上 mkfs");
    devices
}

/// 两块内存盘上 mkfs → 取号 1 → 暖机 → 第一个文件（txg 3）：记账树、中央映射树都是单个叶根，inode 树根下一片叶容器。
pub(super) fn pool_after_the_first_file() -> (Devices, TransactionOutput) {
    let (devices, _allocator, first) = pool_and_its_allocator_after_the_first_file();
    (devices, first)
}

/// [`pool_after_the_first_file`] 连同接着发布要用的分配器。
pub(super) fn pool_and_its_allocator_after_the_first_file(
) -> (Devices, PoolAllocator, TransactionOutput) {
    let parameters = parameters();
    let mut devices = two_sparse_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
    let mut allocator: PoolAllocator =
        allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let content: Vec<u8> = (0..3000u32)
        .map(|index| u8::try_from(index % 251).expect("小于 256"))
        .collect();
    let first = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写了两条根"),
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件")
    };
    (devices, allocator, first)
}

/// 两块盘此刻的整份镜像：拒之前、拒之后逐字节比。
fn images_of(devices: &Devices) -> Vec<(DeviceIdentity, SparseDevice)> {
    devices
        .iter()
        .map(|(identity, device)| (*identity, device.image.clone()))
        .collect()
}

pub(super) fn device_mut(
    devices: &mut Devices,
    identity: DeviceIdentity,
) -> &mut SparseBlockDevice {
    &mut devices
        .iter_mut()
        .find(|(candidate, _)| *candidate == identity)
        .expect("池里有这块盘")
        .1
}

pub(super) fn read_bytes(
    devices: &Devices,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length: usize,
) -> Vec<u8> {
    PoolReader::read(devices, device, offset, length).expect("读得到")
}

/// 按位置条目把一个单元的字节写到它的两份落点上（两块盘各一份），交回整单元 CRC-32C（位置条目里记的那个校验和）。
pub(super) fn write_unit_at_both_locations(
    devices: &mut Devices,
    locations: &[LocationEntry; 2],
    unit: &[u8],
) -> u32 {
    for location in locations {
        device_mut(devices, location.device)
            .image
            .write(location.slot.to_device_offset(), unit);
    }
    crc32_castagnoli(unit)
}

/// 码 2 节点改过条目区或头之后重封：载荷 CRC（罩 [86 + 2k, 16384)，住 76 + 2k）与头校验和（罩 [0, 86 + 2k)）。
pub(super) fn reseal_index_node(node: &mut [u8]) {
    let key_width = usize::from(node[51]);
    let header_end = 86 + 2 * key_width;
    let payload_checksum = crc32_castagnoli(&node[header_end..]);
    node[header_end - 10..header_end - 6].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(node, header_end);
}

/// 码 3 单元改过之后重封：载荷 CRC（罩 [107, 32768)，住 89）与头校验和（罩 [0, 107)）。
pub(super) fn reseal_packed_unit(unit: &mut [u8]) {
    let payload_checksum = crc32_castagnoli(&unit[107..]);
    unit[89..93].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(unit, 107);
}

/// 码 1 单元改过之后重封：载荷 CRC（罩 [105, 32768)，住 101）与头校验和（罩 [0, 105)）。
pub(super) fn reseal_data_unit(unit: &mut [u8]) {
    let payload_checksum = crc32_castagnoli(&unit[105..]);
    unit[101..105].copy_from_slice(&payload_checksum.to_le_bytes());
    seal_header_checksum(unit, 105);
}

/// 码 2 节点第 `index` 条条目在节点里的字节范围（条目区从 86 + 2k + 29 起，条目宽住 84 + 2k）。
pub(super) fn entry_range_in_node(node: &[u8], index: usize) -> std::ops::Range<usize> {
    let key_width = usize::from(node[51]);
    let entry_width = usize::from(u16::from_le_bytes([
        node[84 + 2 * key_width],
        node[85 + 2 * key_width],
    ]));
    let entries_start = 86 + 2 * key_width + 29;
    entries_start + index * entry_width..entries_start + (index + 1) * entry_width
}

/// 一条节点指针里两条位置条目的整单元校验和换成 `unit_checksum`（指针在字节里从 `pointer_offset` 起：头部 50，位置条目 14 × 2，
/// 每条的校验和住条目偏移 4 + 6）。
pub(super) fn repoint_the_checksums_of_a_pointer(
    bytes: &mut [u8],
    pointer_offset: usize,
    unit_checksum: u32,
) {
    for location_index in 0..2 {
        let checksum_offset = pointer_offset + 50 + location_index * 14 + 10;
        bytes[checksum_offset..checksum_offset + 4].copy_from_slice(&unit_checksum.to_le_bytes());
    }
}

/// 系统配置里的根环几何（读根槽要它）。
pub(super) fn system_configuration_of(devices: &Devices) -> SystemConfiguration {
    choose_system_configuration(devices).expect("系统配置择得到")
}

/// 根环里自证过的每一条根，连同它住的槽。
pub(super) fn roots_in_the_ring(devices: &Devices) -> Vec<(RootRingSlot, RootRecord)> {
    let system_configuration = system_configuration_of(devices);
    every_root_ring_slot(&system_configuration.immutable.sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                devices,
                &system_configuration.immutable.region_devices,
                &system_configuration.immutable.sizes,
                &system_configuration.immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => Some((ring_slot, root)),
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect()
}

/// 根环里 (txg, 实例代号) 最大的那条根与它住的槽（择根择到的就是它）。
pub(super) fn newest_root_in_the_ring(devices: &Devices) -> (RootRingSlot, RootRecord) {
    roots_in_the_ring(devices)
        .into_iter()
        .max_by_key(|(_, root)| (root.checkpoint_txg, root.instance))
        .expect("根环里有自证过的根")
}

/// 把一条根按它的几何写进根环的一个槽（整槽自证校验和由 `RootRecord::to_slot` 算）。
pub(super) fn write_root_into_the_ring_slot(
    devices: &mut Devices,
    ring_slot: RootRingSlot,
    root: &RootRecord,
) {
    let parameters = parameters();
    let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
    device_mut(devices, device).image.write(
        offset,
        &root.to_slot(usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽")),
    );
}

/// 中央映射树根（单个叶根，这里改的几版都是）里码 2 节点 `node_pointer` 那一条映射条目的两条位置条目，整单元校验和换成 `node_checksum`。
/// 改完重封映射树根自己的两道校验和（下一次在同一个根上改另一条时，解它要先过这两道）。
pub(super) fn repoint_the_mapping_entry_of_an_index_node(
    mapping_root: &mut [u8],
    node_pointer: NodePointer,
    node_checksum: u32,
) {
    let mapping_key = mapping_key_for_node(UNIT_CLASS_INDEX_NODE, node_pointer);
    let mapping_header = parse_index_node(mapping_root).expect("映射树根解得开");
    assert_eq!(
        mapping_header.level, 0,
        "第一个文件那一版与之后建空 inode 那一版的映射树都是单个叶根"
    );
    let mapping_entry_index = mapping_header
        .entries
        .iter()
        .position(|entry| parse_mapping_entry(entry).is_some_and(|(key, _)| key == mapping_key))
        .expect("映射里有这个节点");
    let mapping_entry_range = entry_range_in_node(mapping_root, mapping_entry_index);
    for location_index in 0..2 {
        // 映射条目：key 27 + 位置条目 14 × 2，每条的校验和住条目偏移 4 + 6。
        let checksum_offset = mapping_entry_range.start + 27 + location_index * 14 + 10;
        mapping_root[checksum_offset..checksum_offset + 4]
            .copy_from_slice(&node_checksum.to_le_bytes());
    }
    reseal_index_node(mapping_root);
}

/// 把最新那条根的树表里种类为 `kind` 的那棵树的根节点改一下（`change` 改字节，之后这里重封它），再把指着它的每一道校验和都重算：
/// 树表条目里那条根指针、中央映射里它那一条（单个叶根的映射树）、树表单元、映射树根、根记录。读者沿根记录读下来，走到被改的节点才判得到。
pub(super) fn rewrite_the_root_node_of_a_tree_of_the_newest_root(
    devices: &mut Devices,
    kind: u16,
    change: impl FnOnce(&mut Vec<u8>),
) {
    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
    let mut tree_table = read_bytes(
        devices,
        root.tree_table.locations[0].device,
        root.tree_table.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let tree_table_header = parse_index_node(&tree_table).expect("树表单元解得开");
    let entry_index = tree_table_header
        .entries
        .iter()
        .position(|entry| TreeTableEntry::parse(entry).is_some_and(|parsed| parsed.kind == kind))
        .expect("树表里有这种树");
    let child = TreeTableEntry::parse(&tree_table_header.entries[entry_index])
        .expect("树表条目解得开")
        .root;
    let mut node = read_bytes(
        devices,
        child.locations[0].device,
        child.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    change(&mut node);
    reseal_index_node(&mut node);
    let node_checksum = write_unit_at_both_locations(devices, &child.locations, &node);

    let entry_range = entry_range_in_node(&tree_table, entry_index);
    // 树表条目里根指针从条目偏移 14 起（树 ID 8 + 条目长度 2 + 种类 2 + flags 2）。
    repoint_the_checksums_of_a_pointer(&mut tree_table[entry_range], 14, node_checksum);
    reseal_index_node(&mut tree_table);
    let tree_table_checksum =
        write_unit_at_both_locations(devices, &root.tree_table.locations, &tree_table);

    let mut mapping_root = read_bytes(
        devices,
        root.mapping_root.locations[0].device,
        root.mapping_root.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    repoint_the_mapping_entry_of_an_index_node(&mut mapping_root, child, node_checksum);
    let mapping_root_checksum =
        write_unit_at_both_locations(devices, &root.mapping_root.locations, &mapping_root);

    for location in &mut root.tree_table.locations {
        location.unit_checksum = tree_table_checksum;
    }
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = mapping_root_checksum;
    }
    write_root_into_the_ring_slot(devices, ring_slot, &root);
}

/// 把最新那条根的树表里种类为 `kind` 的那条条目改一下（`change` 改条目的 200 字节），重封树表单元、重算根记录里树表指针的校验和。
pub(super) fn rewrite_a_tree_table_entry_of_the_newest_root(
    devices: &mut Devices,
    kind: u16,
    change: impl FnOnce(&mut [u8]),
) {
    let (ring_slot, mut root) = newest_root_in_the_ring(devices);
    let mut tree_table = read_bytes(
        devices,
        root.tree_table.locations[0].device,
        root.tree_table.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let tree_table_header = parse_index_node(&tree_table).expect("树表单元解得开");
    let entry_index = tree_table_header
        .entries
        .iter()
        .position(|entry| TreeTableEntry::parse(entry).is_some_and(|parsed| parsed.kind == kind))
        .expect("树表里有这种树");
    let entry_range = entry_range_in_node(&tree_table, entry_index);
    change(&mut tree_table[entry_range]);
    reseal_index_node(&mut tree_table);
    let tree_table_checksum =
        write_unit_at_both_locations(devices, &root.tree_table.locations, &tree_table);
    for location in &mut root.tree_table.locations {
        location.unit_checksum = tree_table_checksum;
    }
    write_root_into_the_ring_slot(devices, ring_slot, &root);
}

/// 可写挂载被拒、盘上逐字节不变：交回拒的那个错。
pub(super) fn writable_mount_refused_before_any_write(devices: &mut Devices) -> MountError {
    let before = images_of(devices);
    let refusal = match mount_writable(&parameters(), devices) {
        Ok(mounted) => panic!(
            "坏镜像被可写挂载收下了：这一版是实例 {:?}、txg {:?}",
            mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg
        ),
        Err(refusal) => refusal,
    };
    assert_eq!(
        images_of(devices),
        before,
        "拒在任何写之前：两块盘逐字节不变（{refusal:?}）"
    );
    refusal
}

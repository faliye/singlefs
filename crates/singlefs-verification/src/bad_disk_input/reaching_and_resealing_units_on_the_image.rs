//! 在镜像上找到要改的那个单元并在改完之后照新内容重算校验和写回：读写字段、走到树表与各树的根、沿引用链往下、逐层重封。

use super::*;

/// 一份镜像上「最新那条根下面该有的那几棵树」在不在：指着普查那几族的坏法要坏的就是它们。
/// 树表 0 条的盘面（只做过 mkfs、回退到第 0 代根、零单元发布那几版）上一棵都没有，每一种指着普查的坏法都只会记「不适用」。
#[must_use]
pub fn newest_root_has_the_trees_the_targeted_damages_need(image: &MemoryPool) -> bool {
    let Some(reached) = reach_the_tree_table(image) else {
        return false;
    };
    // extent / 记账两棵要的是「非空的叶」：按记录改的那几条坏法坏的是根那一片（第一版只有 inode 1 有 extent，extent 树上段只有根兼叶；
    // 记账树在两块盘的池上是根兼叶）。分配记录树按位置寻址、根恒在第 1 层以上（D8（核心索引结构） 已定项 14），坏记录的坏法沿路走到叶，
    // 这里只要根非空。inode 树只要非空：条目宽那一条对内部节点与叶都做得出来。
    let non_empty_leaf = [TREE_KIND_EXTENT, TREE_KIND_ACCOUNTING]
        .into_iter()
        .all(|tree_kind| {
            chain_from_the_tree_table(image, &reached, tree_kind).is_some_and(|chain| {
                index_node_is_a_leaf(&chain.tree_root_node)
                    && IndexNodeEntryLayout::of(&chain.tree_root_node).entry_count > 0
            })
        });
    let allocation_record_tree_is_non_empty =
        chain_from_the_tree_table(image, &reached, TREE_KIND_ALLOCATION_RECORDS)
            .is_some_and(|chain| IndexNodeEntryLayout::of(&chain.tree_root_node).entry_count > 0);
    // 中央映射树的根住根记录、不经树表（D19 已定项 11）：条目宽那一条要它非空。
    let central_mapping_is_non_empty = open_chain_to_the_central_mapping_tree_root(image)
        .is_some_and(|chain| IndexNodeEntryLayout::of(&chain.mapping_root_node).entry_count > 0);
    non_empty_leaf
        && allocation_record_tree_is_non_empty
        && central_mapping_is_non_empty
        && chain_from_the_tree_table(image, &reached, TREE_KIND_INODE)
            .is_some_and(|chain| IndexNodeEntryLayout::of(&chain.tree_root_node).entry_count > 0)
}

/// 一份镜像上最新那条根指着的树表是 0 条：只做过 mkfs、第一个文件版本之前的写行与暖机那几版、回退到这样一版的根。
/// C481（坏盘输入的基线镜像取不到树表 0 条的盘面） 要的那一档基线就是它。树表走不到（根择不出、指针全零、条目宽不是 200）不算。
#[must_use]
pub fn newest_root_tree_table_has_no_entries(image: &MemoryPool) -> bool {
    reach_the_tree_table(image)
        .is_some_and(|reached| IndexNodeEntryLayout::of(&reached.tree_table_node).entry_count == 0)
}

/// 把一份镜像接成可写挂载要的那一串块设备（每块盘一份内存里的稀疏盘）。
pub(super) fn block_devices_of(
    image: &MemoryPool,
    device_width: HistoryDeviceWidth,
) -> Vec<(DeviceIdentity, SparseBlockDevice)> {
    image
        .devices
        .iter()
        .map(|(identity, sparse)| {
            let mut device =
                SparseBlockDevice::new(device_width.device_bytes(), PhysicalBlockSizeInBytes(512));
            device.image = sparse.clone();
            (*identity, device)
        })
        .collect()
}

// ---- 盘上的小端读写：与 checker、实现各写一份（D13 已定项 5，只共享 `singlefs-format` 的常量） ----

pub(super) fn read_u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("2 字节"))
}
pub(super) fn read_u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4 字节"))
}
pub(super) fn read_six_byte_unsigned_at(bytes: &[u8], offset: usize) -> u64 {
    let mut widened = [0u8; 8];
    widened[..6].copy_from_slice(&bytes[offset..offset + 6]);
    u64::from_le_bytes(widened)
}
pub(super) fn write_six_byte_unsigned_at(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 6].copy_from_slice(&value.to_le_bytes()[..6]);
}

// ---- 校验和重算 ----

/// 把一个 32 字节的宽校验和字段按新内容重算：字段自身按 0 参与，低 4 字节写 CRC-32C、其余 28 字节恒 0
/// （`singlefs-format` 的 `WIDE_CHECKSUM_BYTES` 注、checker 的 `checksum_field_holds` 同一条口径）。
pub(super) fn reseal_wide_checksum_field(bytes: &mut [u8], cover_end: usize, field_offset: usize) {
    let field_width = usize::try_from(WIDE_CHECKSUM_BYTES).expect("32");
    bytes[field_offset..field_offset + field_width].fill(0);
    let digest = crc32_castagnoli_table(&bytes[..cover_end]).to_le_bytes();
    bytes[field_offset..field_offset + digest.len()].copy_from_slice(&digest);
}

pub(super) fn index_node_key_width(node: &[u8]) -> usize {
    usize::from(node[INDEX_NODE_KEY_WIDTH_OFFSET])
}

/// 码 2 节点明文头的末尾 = 86 + 2 × key 宽（也是载荷 CRC 罩的那一段的起点）。
fn index_node_plaintext_header_end(node: &[u8]) -> usize {
    usize::try_from(INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE).expect("86")
        + 2 * index_node_key_width(node)
}

pub(super) fn index_node_entry_count_offset(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node) - 4
}
pub(super) fn index_node_entry_width_offset(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node) - 2
}
fn index_node_payload_checksum_offset(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node) - 10
}
pub(super) fn index_node_entries_start(node: &[u8]) -> usize {
    index_node_plaintext_header_end(node)
        + usize::try_from(NONCE_MAC_ALGORITHM_RESERVED_BYTES).expect("29")
}

/// 一个码 2 节点改完之后把它自己的两道校验和重算：先载荷 CRC（它住在头里），再头校验和（它罩着载荷 CRC 那 4 字节）。
pub(super) fn reseal_index_node(node: &mut [u8]) {
    let header_end = index_node_plaintext_header_end(node);
    let payload_checksum_offset = index_node_payload_checksum_offset(node);
    let payload_digest = crc32_castagnoli_table(&node[header_end..]).to_le_bytes();
    node[payload_checksum_offset..payload_checksum_offset + payload_digest.len()]
        .copy_from_slice(&payload_digest);
    reseal_wide_checksum_field(node, header_end, UNIT_HEADER_CHECKSUM_OFFSET);
}

/// 一个码 1 数据单元改完之后把它自己的两道校验和重算。
pub(super) fn reseal_data_unit(unit: &mut [u8]) {
    let header_end = usize::try_from(DATA_UNIT_HEADER_BYTES).expect("105");
    let payload_digest = crc32_castagnoli_table(&unit[header_end..]).to_le_bytes();
    unit[DATA_UNIT_PAYLOAD_CHECKSUM_OFFSET
        ..DATA_UNIT_PAYLOAD_CHECKSUM_OFFSET + payload_digest.len()]
        .copy_from_slice(&payload_digest);
    reseal_wide_checksum_field(unit, header_end, UNIT_HEADER_CHECKSUM_OFFSET);
}

/// 把一条指针（86 或 88 字节）两条位置条目里的整单元校验和都改成 `checksum`。
pub(super) fn write_unit_checksum_into_both_locations(pointer: &mut [u8], checksum: u32) {
    for location_offset in [
        POINTER_FIRST_LOCATION_OFFSET,
        POINTER_SECOND_LOCATION_OFFSET,
    ] {
        let field = location_offset + LOCATION_UNIT_CHECKSUM_OFFSET_IN_ENTRY;
        pointer[field..field + 4].copy_from_slice(&checksum.to_le_bytes());
    }
}

pub(super) fn newest_geometry(image: &MemoryPool) -> Option<PoolGeometry> {
    chosen_system_configurations(image)
        .into_iter()
        .find_map(|(_, chosen)| chosen)
        .map(|(_, geometry)| geometry)
}

pub(super) fn newest_root_slot(
    image: &MemoryPool,
    geometry: &PoolGeometry,
) -> Option<NewestRootSlot> {
    let slot_width = usize::try_from(geometry.physical_block_size).ok()?;
    let (region, slot, newest_view) = valid_roots(image, geometry)
        .into_iter()
        .max_by_key(|(_, _, view)| (view.checkpoint_txg, view.instance))?;
    let (_, _, device, offset_in_bytes) = root_slot_positions(geometry).into_iter().find(
        |(candidate_region, candidate_slot, _, _)| {
            *candidate_region == region && *candidate_slot == slot
        },
    )?;
    let slot_bytes = ImageReader::read(image, device, offset_in_bytes, slot_width)?;
    Some(NewestRootSlot {
        device,
        offset_in_bytes,
        slot_bytes,
        checkpoint_txg: newest_view.checkpoint_txg,
    })
}

/// 按一条码 2 / 码 3 指针读它指的单元：两条位置条目必须同槽（D2 已定项 10），不同槽就不走这条链。
pub(super) fn read_unit_via_pointer(
    image: &MemoryPool,
    pointer: &[u8],
    unit_bytes: usize,
) -> Option<([u32; 2], u64, Vec<u8>)> {
    let view = parse_node_pointer(pointer);
    if view.all_zero || view.locations[0].slot != view.locations[1].slot {
        return None;
    }
    let devices = [view.locations[0].device, view.locations[1].device];
    let slot = view.locations[0].slot;
    let bytes = ImageReader::read(image, devices[0], slot * SLOT_BYTES, unit_bytes)?;
    Some((devices, slot, bytes))
}

pub(super) fn write_unit_to_both_devices(
    image: &mut MemoryPool,
    devices: [u32; 2],
    slot: u64,
    bytes: &[u8],
) {
    for device in devices {
        if let Some(sparse) = image.devices.get_mut(&DeviceIdentity(device)) {
            sparse.write(DeviceOffsetInBytes(slot * SLOT_BYTES), bytes);
        }
    }
}

/// 从最新那条自证过的根走到树表节点，**条目宽是多少都走**；根择不出、树表指针全零都交回 None。
/// 只有坏树表单元自己那一条坏法走这一支（它要坏的正是条目宽），别的坏法走
/// [`reach_the_tree_table`]——它们要从树表条目里按 200 的固定偏移读种类与根指针。
pub(super) fn reach_the_tree_table_whatever_its_entry_width_says(
    image: &MemoryPool,
) -> Option<TreeTableReached> {
    let geometry = newest_geometry(image)?;
    let newest_root = newest_root_slot(image, &geometry)?;
    let root_slot = newest_root.slot_bytes;
    let (tree_table_devices, tree_table_slot, tree_table_node) = read_unit_via_pointer(
        image,
        &root_slot[ROOT_TREE_TABLE_POINTER_OFFSET..ROOT_TREE_TABLE_POINTER_OFFSET + 86],
        usize::try_from(NODE_BYTES).expect("16384"),
    )?;
    Some(TreeTableReached {
        root_device: newest_root.device,
        root_offset: newest_root.offset_in_bytes,
        root_slot,
        tree_table_devices,
        tree_table_slot,
        tree_table_node,
    })
}

/// 从最新那条自证过的根走到树表节点；根择不出、树表指针全零、条目宽不是 200，都交回 None。
fn reach_the_tree_table(image: &MemoryPool) -> Option<TreeTableReached> {
    let reached = reach_the_tree_table_whatever_its_entry_width_says(image)?;
    if IndexNodeEntryLayout::of(&reached.tree_table_node).entry_width
        != usize::try_from(TREE_TABLE_ENTRY_BYTES).expect("200")
    {
        return None;
    }
    Some(reached)
}

/// 从树表接着走到 `tree_kind` 那棵树的根节点；树表里没有这棵树、根指针全零、或者节点读不出来，都交回 None。
fn chain_from_the_tree_table(
    image: &MemoryPool,
    reached: &TreeTableReached,
    tree_kind: u16,
) -> Option<ChainToATreeRoot> {
    let layout = IndexNodeEntryLayout::of(&reached.tree_table_node);
    let tree_table_entry_index = (0..layout.entry_count).find(|index| {
        read_u16_at(
            &reached.tree_table_node,
            layout.offset_of_entry(*index) + TREE_TABLE_ENTRY_KIND_OFFSET,
        ) == tree_kind
    })?;
    let root_pointer_start =
        layout.offset_of_entry(tree_table_entry_index) + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
    let (tree_root_devices, tree_root_slot, tree_root_node) = read_unit_via_pointer(
        image,
        &reached.tree_table_node[root_pointer_start..root_pointer_start + 86],
        usize::try_from(NODE_BYTES).expect("16384"),
    )?;
    Some(ChainToATreeRoot {
        tree_kind,
        root_device: reached.root_device,
        root_offset: reached.root_offset,
        root_slot: reached.root_slot.clone(),
        tree_table_devices: reached.tree_table_devices,
        tree_table_slot: reached.tree_table_slot,
        tree_table_node: reached.tree_table_node.clone(),
        tree_table_entry_index,
        tree_root_devices,
        tree_root_slot,
        tree_root_node,
    })
}

/// 开一条走到 `tree_kind` 那棵树根节点的链；链上哪一环走不通都交回 None。
pub(super) fn open_chain_to_the_root_of(
    image: &MemoryPool,
    tree_kind: u16,
) -> Option<ChainToATreeRoot> {
    chain_from_the_tree_table(image, &reach_the_tree_table(image)?, tree_kind)
}

/// 从最新那条自证过的根走到中央映射树根节点；根择不出、映射根指针全零、两条位置条目不同槽，都交回 None。
pub(super) fn open_chain_to_the_central_mapping_tree_root(
    image: &MemoryPool,
) -> Option<ChainToTheCentralMappingTreeRoot> {
    let geometry = newest_geometry(image)?;
    let newest_root = newest_root_slot(image, &geometry)?;
    let root_slot = newest_root.slot_bytes;
    let (mapping_root_devices, mapping_root_slot, mapping_root_node) = read_unit_via_pointer(
        image,
        &root_slot[ROOT_MAPPING_POINTER_OFFSET..ROOT_MAPPING_POINTER_OFFSET + 86],
        usize::try_from(NODE_BYTES).expect("16384"),
    )?;
    Some(ChainToTheCentralMappingTreeRoot {
        root_device: newest_root.device,
        root_offset: newest_root.offset_in_bytes,
        root_slot,
        mapping_root_devices,
        mapping_root_slot,
        mapping_root_node,
    })
}

/// 把改过的映射链逐环写回镜像：映射树根 → 根记录里那条指针的整单元校验和 → 根槽的自证校验和。
pub(super) fn seal_and_write_back_the_central_mapping_chain(
    mut chain: ChainToTheCentralMappingTreeRoot,
    image: &mut MemoryPool,
) {
    reseal_index_node(&mut chain.mapping_root_node);
    write_unit_to_both_devices(
        image,
        chain.mapping_root_devices,
        chain.mapping_root_slot,
        &chain.mapping_root_node,
    );
    let mapping_root_checksum = crc32_castagnoli_table(&chain.mapping_root_node);
    write_unit_checksum_into_both_locations(
        &mut chain.root_slot[ROOT_MAPPING_POINTER_OFFSET..ROOT_MAPPING_POINTER_OFFSET + 86],
        mapping_root_checksum,
    );
    let cover_end = chain.root_slot.len();
    reseal_wide_checksum_field(&mut chain.root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
    if let Some(sparse) = image.devices.get_mut(&DeviceIdentity(chain.root_device)) {
        sparse.write(DeviceOffsetInBytes(chain.root_offset), &chain.root_slot);
    }
}

/// 把改过的树表单元逐环写回镜像：树表节点两道（载荷 CRC 与头校验和）→ 根记录里那条指针的整单元校验和 →
/// 根槽的自证校验和。坏的是树表**自己**，所以链只到树表为止，不往下走到某一棵树的根
/// （[`seal_and_write_back_the_chain`] 那五道里最里面两道没有对象）。
pub(super) fn seal_and_write_back_the_tree_table(
    mut reached: TreeTableReached,
    image: &mut MemoryPool,
) {
    reseal_index_node(&mut reached.tree_table_node);
    write_unit_to_both_devices(
        image,
        reached.tree_table_devices,
        reached.tree_table_slot,
        &reached.tree_table_node,
    );
    let tree_table_checksum = crc32_castagnoli_table(&reached.tree_table_node);
    write_unit_checksum_into_both_locations(
        &mut reached.root_slot[ROOT_TREE_TABLE_POINTER_OFFSET..ROOT_TREE_TABLE_POINTER_OFFSET + 86],
        tree_table_checksum,
    );
    let cover_end = reached.root_slot.len();
    reseal_wide_checksum_field(&mut reached.root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
    if let Some(sparse) = image.devices.get_mut(&DeviceIdentity(reached.root_device)) {
        sparse.write(DeviceOffsetInBytes(reached.root_offset), &reached.root_slot);
    }
}

/// 从一棵树的根（`root_node`）往下走到一片叶：每一层在内部条目里取 key 满足 `goes_at_or_before_the_target` 的最后一条
/// （一条都不满足取第 0 条），读它的子指针（紧跟 key 的 86 字节）指的节点。交回沿路读到的节点，根之下第一层在前、叶在末；
/// 根自己就是叶时交回空的。哪一层一条条目都没有、条目宽窄于 key + 子指针、子指针读不出来，交回 `None`。
/// 分配记录树与 extent 树按位置寻址（D8（核心索引结构） 已定项 14），根之下还有节点，坏记录的坏法要走到叶。
pub(super) fn path_down_to_the_leaf_holding(
    image: &MemoryPool,
    root_node: &[u8],
    goes_at_or_before_the_target: &dyn Fn(&[u8]) -> bool,
) -> Option<Vec<NodeBelowTheRoot>> {
    let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
    let mut path: Vec<NodeBelowTheRoot> = Vec::new();
    // 迭代上界是树高（码 2 头的层级是 1 字节，至多 256 层）；跨轮携带的是走到的那个节点。
    for _level in 0..=u8::MAX {
        let current: &[u8] = path.last().map_or(root_node, |node| node.bytes.as_slice());
        if index_node_is_a_leaf(current) {
            return Some(path);
        }
        let layout = IndexNodeEntryLayout::of(current);
        let key_width = index_node_key_width(current);
        if layout.entry_count == 0 || layout.entry_width < key_width + 86 {
            return None;
        }
        let chosen = (0..layout.entry_count)
            .rev()
            .find(|index| {
                let start = layout.offset_of_entry(*index);
                goes_at_or_before_the_target(&current[start..start + key_width])
            })
            .unwrap_or(0);
        let pointer_start = layout.offset_of_entry(chosen) + key_width;
        let (devices, slot, bytes) = read_unit_via_pointer(
            image,
            &current[pointer_start..pointer_start + 86],
            node_bytes,
        )?;
        path.push(NodeBelowTheRoot {
            devices,
            slot,
            bytes,
            entry_index_in_the_parent: chosen,
        });
    }
    None
}

/// 沿 `path`（[`path_down_to_the_leaf_holding`] 交回的那一路）从叶往上：每个节点重封、写回它的两块盘，它的整单元校验和写进
/// 上一层指着它的那条条目的子指针两条位置条目（最上面那个的上一层是 `root_node`）。根自己由调用方接着封
/// （[`seal_and_write_back_the_chain`]）。
pub(super) fn reseal_the_path_below_the_root(
    image: &mut MemoryPool,
    root_node: &mut [u8],
    path: &mut [NodeBelowTheRoot],
) {
    // 迭代上界是这一路的节点数；每一轮只动这一层与它上一层。
    for position in (0..path.len()).rev() {
        let (above, from_here) = path.split_at_mut(position);
        let node = &mut from_here[0];
        reseal_index_node(&mut node.bytes);
        write_unit_to_both_devices(image, node.devices, node.slot, &node.bytes);
        let checksum = crc32_castagnoli_table(&node.bytes);
        let parent: &mut [u8] = match above.last_mut() {
            Some(parent) => &mut parent.bytes,
            None => root_node,
        };
        let pointer_start = IndexNodeEntryLayout::of(parent)
            .offset_of_entry(node.entry_index_in_the_parent)
            + index_node_key_width(parent);
        write_unit_checksum_into_both_locations(
            &mut parent[pointer_start..pointer_start + 86],
            checksum,
        );
    }
}

/// 把改过的链逐环写回镜像：树根 → 树表条目里的整单元校验和 → 树表节点 → 根记录里的整单元校验和 → 根槽的自证校验和。
/// 少重算一道，读者在解析之前就先拒了，指着普查那几族的坏法一次也打不到。
pub(super) fn seal_and_write_back_the_chain(mut chain: ChainToATreeRoot, image: &mut MemoryPool) {
    reseal_index_node(&mut chain.tree_root_node);
    write_unit_to_both_devices(
        image,
        chain.tree_root_devices,
        chain.tree_root_slot,
        &chain.tree_root_node,
    );

    let layout = IndexNodeEntryLayout::of(&chain.tree_table_node);
    let root_pointer_start =
        layout.offset_of_entry(chain.tree_table_entry_index) + TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET;
    let tree_root_checksum = crc32_castagnoli_table(&chain.tree_root_node);
    write_unit_checksum_into_both_locations(
        &mut chain.tree_table_node[root_pointer_start..root_pointer_start + 86],
        tree_root_checksum,
    );
    reseal_index_node(&mut chain.tree_table_node);
    write_unit_to_both_devices(
        image,
        chain.tree_table_devices,
        chain.tree_table_slot,
        &chain.tree_table_node,
    );

    let tree_table_checksum = crc32_castagnoli_table(&chain.tree_table_node);
    write_unit_checksum_into_both_locations(
        &mut chain.root_slot[ROOT_TREE_TABLE_POINTER_OFFSET..ROOT_TREE_TABLE_POINTER_OFFSET + 86],
        tree_table_checksum,
    );
    let cover_end = chain.root_slot.len();
    reseal_wide_checksum_field(&mut chain.root_slot, cover_end, ROOT_SELF_CHECKSUM_OFFSET);
    if let Some(sparse) = image.devices.get_mut(&DeviceIdentity(chain.root_device)) {
        sparse.write(DeviceOffsetInBytes(chain.root_offset), &chain.root_slot);
    }
}

pub(super) fn index_node_is_a_leaf(node: &[u8]) -> bool {
    node[INDEX_NODE_LEVEL_OFFSET] == 0
}

// ---- 逐个坏法 ----

/// 镜像上写过的每一个扇区（哪块盘、第几个扇区）。
pub(super) fn written_sectors_of(image: &MemoryPool) -> Vec<(DeviceIdentity, u64)> {
    let mut sectors = Vec::new();
    for (identity, sparse) in &image.devices {
        for sector in sparse.written_sectors_in(DeviceOffsetInBytes(0), image.device_size_in_bytes)
        {
            sectors.push((*identity, sector));
        }
    }
    sectors
}

pub(super) fn sector_of(sparse: &SparseDevice, sector: u64) -> Vec<u8> {
    sparse.read(
        DeviceOffsetInBytes(sector * SECTOR_BYTES),
        usize::try_from(SECTOR_BYTES).expect("512"),
    )
}

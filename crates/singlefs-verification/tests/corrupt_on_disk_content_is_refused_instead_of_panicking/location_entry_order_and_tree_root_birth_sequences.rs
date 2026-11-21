//! 位置条目降序排的指针与出生序号同指针不符的树根：树表根指针、创世根的实例表指针、extent 树与分配记录树的根。

use super::*;

// ─── C476 普查「缺口」那张表 `pointer.rs` 那一行：I-2.5 升序断言，照抄写回盘上读来的指针时走得到 ───

/// 根记录里实例表指针在槽里的偏移：magic 4 + fsid 16 + flags 4 + 实例代号 4 + txg 8 + 树表指针 86 + 树 ID 水位 8 + F 8 + 自证校验和 32。
const INSTANCE_TABLE_POINTER_OFFSET_IN_THE_ROOT_SLOT: usize = 170;

/// 根记录自证校验和的偏移（`root_record::ROOT_CHECKSUM_OFFSET`，D22（单元原子性怎么合成） 已定项 7 的字段表）。
const ROOT_CHECKSUM_OFFSET_IN_THE_ROOT_SLOT: usize = 138;

/// 一条节点指针里两条位置条目（头部 50 之后，各 14 字节）对调。
fn swap_the_two_location_entries_of_a_pointer(bytes: &mut [u8], pointer_offset: usize) {
    let first_start = pointer_offset + 50;
    let first = bytes[first_start..first_start + 14].to_vec();
    let second = bytes[first_start + 14..first_start + 28].to_vec();
    bytes[first_start..first_start + 14].copy_from_slice(&second);
    bytes[first_start + 14..first_start + 28].copy_from_slice(&first);
}

/// 场景：第一个文件那一版的树表里 extent 树那条条目的根指针，两条位置条目对调成（盘 1, 盘 0）——读者按位置条目逐条试，照样读得到；
/// 树表与根记录重封。
/// 预期：可写挂载在任何写之前报 I-2.5（位置条目按设备身份升序）。重建若照收，写行那次发布照抄这条指针写新树表，
/// 在 `NodePointer::write_to` 的升序断言上 panic（实审 A3a 实测）。
#[test]
fn a_tree_table_root_pointer_with_descending_location_entries_is_refused_by_the_writable_mount() {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_a_tree_table_entry_of_the_newest_root(
        &mut devices,
        singlefs_core::records::TREE_KIND_EXTENT,
        |entry| swap_the_two_location_entries_of_a_pointer(entry, 14),
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-2.5",
                detail: "位置条目不按设备身份严格升序",
            })
        ),
        "逆序的树表根指针：{refusal:?}"
    );
}

/// 场景：只做过 mkfs 的池，三个区域里第 0 代根的实例表指针两条位置条目对调、整槽自证校验和重算。
/// 预期：可写挂载在任何写之前报 I-2.5。第一次可写挂载的零单元发布若照抄这条指针写新根，在 `NodePointer::write_to` 的升序断言上 panic。
#[test]
fn a_genesis_root_whose_instance_table_pointer_has_descending_location_entries_is_refused_by_the_writable_mount(
) {
    let mut devices = formatted_pool();
    let parameters = parameters();
    let root_slot_bytes = usize::try_from(parameters.geometry.physical_block_size).expect("根槽宽");
    for (ring_slot, root) in roots_in_the_ring(&devices) {
        let mut slot = root.to_slot(root_slot_bytes);
        swap_the_two_location_entries_of_a_pointer(
            &mut slot,
            INSTANCE_TABLE_POINTER_OFFSET_IN_THE_ROOT_SLOT,
        );
        let digest = wide_checksum_with_field_zeroed(
            &slot,
            root_slot_bytes,
            ROOT_CHECKSUM_OFFSET_IN_THE_ROOT_SLOT,
        );
        slot[ROOT_CHECKSUM_OFFSET_IN_THE_ROOT_SLOT..ROOT_CHECKSUM_OFFSET_IN_THE_ROOT_SLOT + 32]
            .copy_from_slice(&digest);
        let device = parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
        let offset = slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing);
        device_mut(&mut devices, device).image.write(offset, &slot);
    }
    let swapped_roots = roots_in_the_ring(&devices);
    assert_eq!(swapped_roots.len(), 3, "对调之后三条根照样自证得过");
    assert_eq!(
        swapped_roots[0].1.instance_table.locations[0].device,
        DeviceIdentity(1),
        "实例表指针第一条位置条目现在是盘 1"
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-2.5",
                detail: "位置条目不按设备身份严格升序",
            })
        ),
        "逆序的第 0 代实例表指针：{refusal:?}"
    );
}

// ─── 第 33 条续：重建读另外三棵树也照冷走读判全，一棵一条 ───

/// 码 2 节点头里出生序号那 4 字节住 72 + 2k（D18（块里携带什么信息） 已定项 18 的偏移表），加一。
fn bump_the_birth_sequence_in_the_header(node: &mut [u8]) {
    let birth_sequence_offset = 72 + 2 * usize::from(node[51]);
    let birth_sequence = u32::from_le_bytes(
        node[birth_sequence_offset..birth_sequence_offset + 4]
            .try_into()
            .expect("4 字节"),
    );
    node[birth_sequence_offset..birth_sequence_offset + 4]
        .copy_from_slice(&(birth_sequence + 1).to_le_bytes());
}

fn refused_with_a_tree_root_birth_sequence_differing_from_its_pointer(
    refusal: &MountError,
) -> bool {
    matches!(
        refusal,
        MountError::Recovery(RecoveryFailure::InvariantViolated {
            invariant: "I-1.2",
            detail: "树根出生序号与指针不符",
        })
    )
}

/// 场景：第一个文件那一版的 extent 树根节点头里的出生序号加一，节点与指着它的每一道校验和重算。
/// 预期：可写挂载在任何写之前报 I-1.2「树根出生序号与指针不符」。重建若按「位置那几样」读 extent 树、不核头里的出生身份，就收下它。
#[test]
fn an_extent_tree_root_whose_birth_sequence_differs_from_its_pointer_is_refused_by_the_writable_mount(
) {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        singlefs_core::records::TREE_KIND_EXTENT,
        |node| bump_the_birth_sequence_in_the_header(node),
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        refused_with_a_tree_root_birth_sequence_differing_from_its_pointer(&refusal),
        "extent 树根的出生序号判 I-1.2：{refusal:?}"
    );
}

/// 场景：第一个文件那一版的分配记录树根节点头里的出生序号加一，节点与指着它的每一道校验和重算。
/// 预期：可写挂载在任何写之前报 I-1.2「树根出生序号与指针不符」。重建若按「位置那几样」读分配记录树、不核头里的出生身份，就收下它。
#[test]
fn an_allocation_record_tree_root_whose_birth_sequence_differs_from_its_pointer_is_refused_by_the_writable_mount(
) {
    let (mut devices, _first) = pool_after_the_first_file();
    rewrite_the_root_node_of_a_tree_of_the_newest_root(
        &mut devices,
        singlefs_core::records::TREE_KIND_ALLOCATION,
        |node| bump_the_birth_sequence_in_the_header(node),
    );
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        refused_with_a_tree_root_birth_sequence_differing_from_its_pointer(&refusal),
        "分配记录树根的出生序号判 I-1.2：{refusal:?}"
    );
}

/// 场景：第一个文件那一版的中央映射树（单个叶根）里第二条条目整条抄成第一条，叶重封、根记录里映射根指针的校验和重算。
/// 预期：可写挂载在任何写之前报「叶里的条目不按 key 严格递增」（I-1.1）。重建若只按「拼得成一棵树」读映射树、不判 key 次序，就收下它。
#[test]
fn a_central_mapping_leaf_with_a_repeated_key_is_refused_by_the_writable_mount_before_any_write() {
    let (mut devices, _first) = pool_after_the_first_file();
    let (ring_slot, mut root) = newest_root_in_the_ring(&devices);
    let mut mapping_root = read_bytes(
        &devices,
        root.mapping_root.locations[0].device,
        root.mapping_root.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    let first_entry = mapping_root[entry_range_in_node(&mapping_root, 0)].to_vec();
    let second_entry_range = entry_range_in_node(&mapping_root, 1);
    mapping_root[second_entry_range].copy_from_slice(&first_entry);
    reseal_index_node(&mut mapping_root);
    let mapping_root_checksum =
        write_unit_at_both_locations(&mut devices, &root.mapping_root.locations, &mapping_root);
    for location in &mut root.mapping_root.locations {
        location.unit_checksum = mapping_root_checksum;
    }
    write_root_into_the_ring_slot(&mut devices, ring_slot, &root);
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        matches!(
            refusal,
            MountError::Recovery(RecoveryFailure::InvariantViolated {
                invariant: "I-1.1",
                detail: "叶里的条目不按 key 严格递增",
            })
        ),
        "重复 key 的映射叶判 key 次序：{refusal:?}"
    );
}

/// 场景：只做过 mkfs 的池可写挂载两次（第二次写行 (1, …)，树表 0 条的这一版建起自己的分配记录树，C512），再把最新那条根的
/// 分配记录树根节点头里的出生序号加一，节点重封、根记录里那条指针的校验和重算。
/// 预期：第三次可写挂载在任何写之前报 I-1.2「树根出生序号与指针不符」。重开树表 0 条的一版若只按「位置那几样」读这棵树，就收下它。
#[test]
fn the_allocation_record_tree_of_a_version_without_file_is_judged_like_the_cold_walk_on_the_writable_mount(
) {
    let mut devices = formatted_pool();
    mount_writable(&parameters(), &mut devices).expect("第一次可写挂载：实例 1，零单元发布");
    let second = mount_writable(&parameters(), &mut devices).expect("第二次可写挂载：实例 2 写行");
    assert!(
        second.current.file_version().is_none(),
        "第二次挂载之后仍是树表 0 条的一版"
    );
    let (ring_slot, mut root) = newest_root_in_the_ring(&devices);
    assert_ne!(
        root.allocation_record_tree_root,
        NodePointer::empty_root(),
        "写过行的树表 0 条那一版有自己的分配记录树"
    );
    let pointer = root.allocation_record_tree_root;
    let mut node = read_bytes(
        &devices,
        pointer.locations[0].device,
        pointer.locations[0].slot.to_device_offset(),
        node_bytes(),
    );
    bump_the_birth_sequence_in_the_header(&mut node);
    reseal_index_node(&mut node);
    let node_checksum = write_unit_at_both_locations(&mut devices, &pointer.locations, &node);
    for location in &mut root.allocation_record_tree_root.locations {
        location.unit_checksum = node_checksum;
    }
    write_root_into_the_ring_slot(&mut devices, ring_slot, &root);
    let refusal = writable_mount_refused_before_any_write(&mut devices);
    assert!(
        refused_with_a_tree_root_birth_sequence_differing_from_its_pointer(&refusal),
        "树表 0 条那一版的分配记录树根判 I-1.2：{refusal:?}"
    );
}

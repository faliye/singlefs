//! 映射里换掉一个键：可写挂载拒绝换了键的盘（搬迁之后换的也拒），重建换了键的版本被拒，映射里没有那个键的条目的版本读回时照样计数读坏的那一份。

use super::*;

/// 这把数据单元 key 的事务号加一（尾段是事务号低 48 位，小端）。交的是这个文件 key 序里最末的单元时，新 key 不撞别的单元（事务号按单元序连着发）、仍小于后面别的类的 key。
fn mapping_key_with_the_transaction_number_raised_by_one(key: &[u8]) -> Vec<u8> {
    let mut transaction_number_bytes = [0u8; 8];
    transaction_number_bytes[..BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY]
        .copy_from_slice(&key[key.len() - BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY..]);
    let raised = u64::from_le_bytes(transaction_number_bytes) + 1;
    let mut raised_key = key.to_vec();
    let tail_starts_at = raised_key.len() - BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY;
    raised_key[tail_starts_at..]
        .copy_from_slice(&raised.to_le_bytes()[..BYTES_OF_THE_TAIL_OF_A_MAPPING_KEY]);
    raised_key
}

/// 造一张映射里查不到 `key` 的盘：`version` 那一版的映射树根兼叶上 `key` 那一条换成 `replacement`（位置条目不动），条目仍严格升序，叶按写者的写法重装（两道校验和都对），
/// 两份写回原槽，根记录里映射根的整单元校验和换成新字节的，根槽重封写回。日志记录里那份新根段不动：它只用来施加水位之上的发布，这一条根不在水位之上。交回盘上那条根。
fn replace_the_key_in_the_mapping_root_on_disk(
    devices: &mut Devices,
    version: &TransactionOutput,
    key: &[u8],
    replacement: &[u8],
) -> RootRecord {
    let mut root = version.root;
    let forged = mapping_root_with_the_key_replaced(version, key, replacement);
    let forged_checksum = crc32_castagnoli(&forged);
    for location in &mut root.mapping_root.locations {
        device_named(devices, location.device)
            .write_at(
                location.slot.to_device_offset(),
                &forged,
                WriteDurability::Plain,
            )
            .expect("写回映射树根");
        location.unit_checksum = forged_checksum;
    }
    let pool_parameters = parameters();
    let ring_slot = target_for_publish(
        root.checkpoint_txg,
        pool_parameters.geometry.root_ring_slots_per_region,
    );
    let device_of_the_ring_slot =
        pool_parameters.region_devices[usize::try_from(ring_slot.region).expect("区域号")];
    device_named(devices, device_of_the_ring_slot)
        .write_at(
            slot_offset(
                ring_slot,
                pool_parameters.geometry.fixed_structure_slot_spacing,
            ),
            &root.to_slot(
                usize::try_from(pool_parameters.geometry.physical_block_size).expect("根槽宽"),
            ),
            WriteDurability::Plain,
        )
        .expect("写回根槽");
    root
}

/// `version` 那一版的映射树根兼叶字节，`key` 那一条换成 `replacement`（位置条目不动），条目仍严格升序，按写者的写法重装（两道校验和都对）。
fn mapping_root_with_the_key_replaced(
    version: &TransactionOutput,
    key: &[u8],
    replacement: &[u8],
) -> Vec<u8> {
    let original = parse_index_node(
        &version
            .units
            .iter()
            .find(|unit| unit.identity == TransactionUnit::MappingTree)
            .expect("三个单元的池映射树根兼叶")
            .bytes,
    )
    .expect("映射树根解得开");
    let entries: Vec<(Vec<u8>, [LocationEntry; 2])> = original
        .entries
        .iter()
        .map(|entry| {
            let (entry_key, locations) = parse_mapping_entry(entry).expect("映射条目切得开");
            if entry_key == key {
                (replacement.to_vec(), locations)
            } else {
                (entry_key, locations)
            }
        })
        .collect();
    assert!(
        entries.windows(2).all(|pair| {
            CodeTwoTreeKey::new(&pair[0].0, CodeTwoKeyFieldWidths::CENTRAL_MAPPING)
                < CodeTwoTreeKey::new(&pair[1].0, CodeTwoKeyFieldWidths::CENTRAL_MAPPING)
        }),
        "前提：换了 key 之后条目仍严格升序"
    );
    build_index_node(
        original.tree,
        original.level,
        original.key_width,
        &original.smallest_key,
        &original.largest_key,
        original.birth_txg,
        &version.root.filesystem_identifier,
        original.instance,
        original.birth_sequence,
        u16::try_from(original.entry_width).expect("条目宽"),
        &entries
            .iter()
            .map(|(entry_key, locations)| build_mapping_entry(entry_key, *locations))
            .collect::<Vec<Vec<u8>>>(),
    )
}

/// 这张盘可写挂载被拒在映射 key 集合那一判（`recovery::central_mapping_keys_are_those_of_the_mapped_units_and_the_entries_kept_for_the_snapshots`），盘上逐字节不变。
fn assert_the_writable_mount_refuses_the_key_set_of_the_mapping(devices: &mut Devices) {
    let image_before_the_mount = memory_pool_of(devices);
    match mount_writable(&parameters(), devices) {
        Err(MountError::Recovery(RecoveryFailure::InvariantViolated { invariant, .. })) => {
            assert_eq!(invariant, CENTRAL_MAPPING_KEY_SET_INVARIANT);
        }
        Err(other) => panic!("拒在映射 key 集合那一判，交回 {other:?}"),
        Ok(_) => panic!("映射 key 集合对不上的盘不许挂上"),
    }
    assert_eq!(
        memory_pool_of(devices),
        image_before_the_mount,
        "拒在任何写之前"
    );
}

/// 映射 key 被换（封得对、仍升序）、提示没过期：可写挂载拒。判别力：`crates/mutations.tsv`「映射 key 集合：建上一版不判」「映射 key 集合：只比条数」两行。
#[test]
fn a_disk_with_a_key_replaced_in_the_mapping_is_refused_by_the_writable_mount() {
    let mut pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(
        &pool.version,
        TransactionUnit::Data(LAST_UNIT_IN_THE_KEY_ORDER),
    );
    let locations = mapped_locations(&pool.version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let replacement = mapping_key_with_the_transaction_number_raised_by_one(&key);
    let version = pool.version.clone();
    replace_the_key_in_the_mapping_root_on_disk(&mut pool.devices, &version, &key, &replacement);
    assert_the_writable_mount_refuses_the_key_set_of_the_mapping(&mut pool.devices);
}

/// 映射 key 被换、提示过期（那一批搬过这个单元，父节点里的提示还指隔离着的旧落点）：不拒的话第一次发布把这把 key 盖回旧落点、搬到的新槽没人引用；可写挂载拒。
/// 判别力：`crates/mutations.tsv`「映射 key 集合：建上一版不判（提示过期那一形）」那一行。
#[test]
fn a_key_replaced_after_a_relocation_is_refused_by_the_writable_mount() {
    let mut pool = pool_holding(&content_of_units(3));
    let mut session = session_of_a_writable_mount(&mut pool.devices);
    let version = current_version_of(&session);
    let key = mapping_key_of_role(&version, TransactionUnit::Data(LAST_UNIT_IN_THE_KEY_ORDER));
    let old_locations = mapped_locations(&version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &old_locations[0], BYTES_OF_A_DATA_UNIT);
    whole_first_file_read_through_the_session(&session, &pool.devices);
    let relocated = session
        .publish_user_change(&mut pool.devices, new_inode_change(3))
        .expect("建一个空 inode");
    let (mapping_keys, _, _) = relocated_of(&relocated);
    assert_eq!(mapping_keys, std::slice::from_ref(&key));
    let after = current_version_of(&session);
    drop(session);
    let replacement = mapping_key_with_the_transaction_number_raised_by_one(&key);
    replace_the_key_in_the_mapping_root_on_disk(&mut pool.devices, &after, &key, &replacement);
    assert_the_writable_mount_refuses_the_key_set_of_the_mapping(&mut pool.devices);
}

/// 直接重建同一张盘：拒在同一判。判别力：`crates/mutations.tsv`「映射 key 集合：不等也放行（拒因写反）」那一行。
#[test]
fn rebuilding_a_version_with_a_key_replaced_in_the_mapping_is_refused() {
    let mut pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(
        &pool.version,
        TransactionUnit::Data(LAST_UNIT_IN_THE_KEY_ORDER),
    );
    let replacement = mapping_key_with_the_transaction_number_raised_by_one(&key);
    let version = pool.version.clone();
    let root_on_disk = replace_the_key_in_the_mapping_root_on_disk(
        &mut pool.devices,
        &version,
        &key,
        &replacement,
    );
    let image = memory_pool_of(&pool.devices);
    match rebuild_version(&image, &root_on_disk, Some(version.record.clone())) {
        Err(RebuildVersionFailure::Walk(RecoveryFailure::InvariantViolated {
            invariant, ..
        })) => assert_eq!(invariant, CENTRAL_MAPPING_KEY_SET_INVARIANT),
        other @ (Ok(
            RebuiltVersion::WithFile(_)
            | RebuiltVersion::WithoutFile
            | RebuiltVersion::WithTreesAndNoFile(_),
        )
        | Err(_)) => panic!("拒在映射 key 集合那一判，交回 {other:?}"),
    }
}

/// 读回用的那一版（内存里）映射查不到这把 key：判不出提示过期，按提示读到的只坏一份照计。盘上的这一形可写挂载已经拒了，这一支只有内存里那一版对不上才走得到。
/// 判别力：`crates/mutations.tsv`「查不到 key 改记没坏：部分写读回映射查不到时把真的一份改记没坏」那一行。
#[test]
fn a_read_back_on_a_version_whose_mapping_has_no_entry_for_the_key_still_counts_the_damaged_copy() {
    let mut pool = pool_holding(&content_of_units(3));
    let key = mapping_key_of_role(
        &pool.version,
        TransactionUnit::Data(LAST_UNIT_IN_THE_KEY_ORDER),
    );
    let locations = mapped_locations(&pool.version, &key).expect("在映射里");
    damage_the_copy_at(&mut pool.devices, &locations[0], BYTES_OF_A_DATA_UNIT);
    let mut version_with_the_key_replaced = pool.version.clone();
    let replacement = mapping_key_with_the_transaction_number_raised_by_one(&key);
    let forged_root =
        mapping_root_with_the_key_replaced(&version_with_the_key_replaced, &key, &replacement);
    version_with_the_key_replaced
        .units
        .iter_mut()
        .find(|unit| unit.identity == TransactionUnit::MappingTree)
        .expect("三个单元的池映射树根兼叶")
        .bytes = Arc::from(forged_root);
    assert!(
        mapped_locations(&version_with_the_key_replaced, &key).is_none(),
        "前提：查不到这把 key"
    );
    let read_back = read_back_the_old_data_unit_telling_whether_a_copy_failed(
        &memory_pool_of(&pool.devices),
        &version_with_the_key_replaced,
        FIRST_FILE,
        pool.version.inode_record.expect("第一个文件那条记录"),
        LAST_UNIT_IN_THE_KEY_ORDER,
        &parameters().filesystem_identifier,
    )
    .expect("按提示读得回");
    assert_eq!(
        read_back.copies,
        CopiesOfAUnitRead::FromTheOtherCopyAfterOneFailed,
        "查不到 key 判不出提示过期：照计只坏一份"
    );
}

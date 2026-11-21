use super::*;

/// 一条码 2 指针，两条位置条目落在盘 0、盘 1 的 `slot` 上，各带整单元校验和 `checksum`（头部与尾段全零）。
fn node_pointer_at_the_slot(slot: u64, checksum: u32) -> PointerView {
    let mut pointer_bytes = vec![0u8; node_pointer_bytes()];
    for (device, location_offset) in [(0u32, 50usize), (1u32, 64usize)] {
        pointer_bytes[location_offset..location_offset + 4].copy_from_slice(&device.to_le_bytes());
        pointer_bytes[location_offset + 4..location_offset + 10]
            .copy_from_slice(&slot.to_le_bytes()[..6]);
        pointer_bytes[location_offset + 10..location_offset + 14]
            .copy_from_slice(&checksum.to_le_bytes());
    }
    parse_node_pointer(&pointer_bytes)
}

/// 源槽与搬迁目的地：同一个单元（同一个整单元校验和），提示在源槽、映射在目的地。
const SOURCE_SLOT_OF_THE_RELOCATED_UNIT: u64 = 50_322;
const DESTINATION_SLOT_OF_THE_RELOCATED_UNIT: u64 = 50_397;
const CHECKSUM_OF_THE_RELOCATED_UNIT: u32 = 0x5EC0_17DA;

/// 次级读法的缓存（[`read_through_a_cache_keyed_by_the_located_placement`]）：一条提示已刷新的指针（提示就是映射位置，不试提示）读不出、
/// 记下「读不出」；另一版里同一个单元的指针提示还在源槽、认到同一个映射落点，要试提示——缓存里那一格「读不出」不拿来用，照读一遍。
#[test]
fn cached_unreadable_placement_is_read_again_by_a_pointer_that_tries_its_hint() {
    let mut cache: BTreeMap<(u32, u64, u32), Option<u64>> = BTreeMap::new();
    let pointer_whose_hint_is_the_mapping_position = node_pointer_at_the_slot(
        DESTINATION_SLOT_OF_THE_RELOCATED_UNIT,
        CHECKSUM_OF_THE_RELOCATED_UNIT,
    );
    let pointer_whose_hint_is_the_source = node_pointer_at_the_slot(
        SOURCE_SLOT_OF_THE_RELOCATED_UNIT,
        CHECKSUM_OF_THE_RELOCATED_UNIT,
    )
    .located_by_the_central_mapping(pointer_whose_hint_is_the_mapping_position.locations);
    assert_eq!(
        read_through_a_cache_keyed_by_the_located_placement(
            &mut cache,
            &pointer_whose_hint_is_the_mapping_position,
            || None,
        ),
        None
    );
    let mut reads_of_the_hint = 0u32;
    let read_back = read_through_a_cache_keyed_by_the_located_placement(
        &mut cache,
        &pointer_whose_hint_is_the_source,
        || {
            reads_of_the_hint += 1;
            Some(7)
        },
    );
    assert_eq!(
        (read_back, reads_of_the_hint),
        (Some(7), 1),
        "缓存里那一格「读不出」来自不试提示的那一读：要试提示的这一读照读一遍、读出提示那一份"
    );
}

/// 同上，反过来：试了提示的那一读读不出，不记进缓存（同一个映射落点，另一版的提示可能读得出、不试提示的那一版可能读得出映射位置）；
/// 之后不试提示的那一读照读。
#[test]
fn an_unreadable_result_of_a_read_that_tried_its_hint_is_not_cached() {
    let mut cache: BTreeMap<(u32, u64, u32), Option<u64>> = BTreeMap::new();
    let pointer_whose_hint_is_the_mapping_position = node_pointer_at_the_slot(
        DESTINATION_SLOT_OF_THE_RELOCATED_UNIT,
        CHECKSUM_OF_THE_RELOCATED_UNIT,
    );
    let pointer_whose_hint_is_the_source = node_pointer_at_the_slot(
        SOURCE_SLOT_OF_THE_RELOCATED_UNIT,
        CHECKSUM_OF_THE_RELOCATED_UNIT,
    )
    .located_by_the_central_mapping(pointer_whose_hint_is_the_mapping_position.locations);
    assert!(pointer_whose_hint_is_the_source.hints_differ_from_the_locations_the_walk_takes());
    assert_eq!(
        read_through_a_cache_keyed_by_the_located_placement(
            &mut cache,
            &pointer_whose_hint_is_the_source,
            || None,
        ),
        None
    );
    assert!(
        cache.is_empty(),
        "试了提示的那一读读不出：不记进缓存，缓存里一格都没有"
    );
    assert_eq!(
        read_through_a_cache_keyed_by_the_located_placement(
            &mut cache,
            &pointer_whose_hint_is_the_mapping_position,
            || Some(11),
        ),
        Some(11)
    );
}

/// 销毁中的快照（码 14 flags 位 0）连同头 ID 指它的码 15 不再钉住块：I-3.9 那一遍不经它们的树记引用；活快照那一对照数。
#[test]
fn the_trees_of_a_snapshot_being_destroyed_are_left_out_of_the_references_of_its_root() {
    let entries = synthetic_tree_table_with_snapshots(&[
        (FIRST_SNAPSHOT_TREE, FIRST_SNAPSHOT_TXG, 0, 0),
        (
            SECOND_SNAPSHOT_TREE,
            SECOND_SNAPSHOT_TXG,
            FIRST_SNAPSHOT_TXG,
            TREE_TABLE_ENTRY_FLAG_SNAPSHOT_BEING_DESTROYED,
        ),
    ]);
    assert_eq!(
        snapshot_trees_of_snapshots_being_destroyed(&entries),
        BTreeSet::from([SECOND_SNAPSHOT_TREE, SECOND_SNAPSHOT_TREE + 1]),
        "S2 的码 14 与配它的码 15 不记引用，S1 那一对照记"
    );
}

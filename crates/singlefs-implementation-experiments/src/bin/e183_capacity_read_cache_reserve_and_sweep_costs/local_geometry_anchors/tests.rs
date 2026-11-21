use super::*;

#[test]
fn default_disk_ring_window_equals_the_registered_absolute_bytes() {
    let expected = [(4u64, 79_659_008u64), (8, 153_190_400), (16, 300_253_184)];
    for (slots_per_region, bytes) in expected {
        let window = ring_window_by_the_clauses(
            4 * GIBIBYTE,
            JOURNAL_RING_DEFAULT_BYTES,
            slots_per_region,
            0,
        );
        assert_eq!(window.ring_window_bytes(), bytes, "S = {slots_per_region}");
        assert_eq!(window.unit_area_slots, 211_968);
        assert_eq!(window.deletion_publish_slots, 374);
    }
    let largest =
        ring_window_by_the_clauses((16 << 40) - MEBIBYTE, JOURNAL_RING_DEFAULT_BYTES, 8, 0);
    assert_eq!(largest.deletion_publish_slots, 804);
    assert_eq!(largest.root_level, 3);
}

#[test]
fn retirement_tree_levels_add_one_slot_each_to_the_deletion_publish() {
    let without = ring_window_by_the_clauses(4 * GIBIBYTE, JOURNAL_RING_DEFAULT_BYTES, 8, 0);
    let with_two_levels =
        ring_window_by_the_clauses(4 * GIBIBYTE, JOURNAL_RING_DEFAULT_BYTES, 8, 2);
    assert_eq!(
        with_two_levels.deletion_publish_slots,
        without.deletion_publish_slots + 2
    );
}

#[test]
fn chain_rewrite_bytes_cross_to_a_second_page_exactly_at_the_registered_row() {
    for (rows, bytes) in REGISTERED_CHAIN_REWRITE_BYTES {
        assert_eq!(
            chain_rewrite_bytes_of_the_switch_reserve(rows),
            bytes,
            "rows0 = {rows}"
        );
    }
    assert_eq!(instance_table_chain_pages(366), 1);
    assert_eq!(instance_table_chain_pages(367), 2);
}

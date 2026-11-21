use super::*;

fn terms_with_distinct_powers_of_two() -> DeviceTerms {
    DeviceTerms {
        device: 0,
        capacity_bytes: 1 << 30,
        allocated_bytes: 1 << 20,
        unreclaimable_bytes: 0,
        mount_time_commitment_bytes: 1,
        abandoned_root_exclusive_bytes: 2,
        pending_delete_share_bytes: 4,
        committed_reservation_share_bytes: 8,
        checkpoint_reserve_pool_share_bytes: 16,
        ring_window_bytes: 32,
        available_bytes: 0,
    }
}

#[test]
fn admission_reserve_adds_all_six_items_including_the_ring_window() {
    let terms = terms_with_distinct_powers_of_two();
    assert_eq!(terms.admission_reserve_bytes(), 1 + 2 + 4 + 8 + 16 + 32);
    let item_sum: u64 = terms
        .admission_reserve_items()
        .iter()
        .map(|(_, bytes)| *bytes)
        .sum();
    assert_eq!(item_sum, 1 + 2 + 4 + 8 + 16 + 32);
}

#[test]
fn dominance_needs_at_least_half_of_the_total() {
    let exactly_half = [
        ("heaviest_item", 50),
        ("middle_item", 30),
        ("lightest_item", 20),
    ];
    assert_eq!(
        dominance_of(&exactly_half),
        Dominance::Dominant {
            item: "heaviest_item",
            share_permille: 500
        }
    );
    let just_below_half = [
        ("heaviest_item", 49),
        ("middle_item", 30),
        ("lightest_item", 21),
    ];
    assert_eq!(
        dominance_of(&just_below_half),
        Dominance::NoItemAboveHalf {
            largest_item: "heaviest_item",
            share_permille: 490
        }
    );
    let nothing_reserved = [("heaviest_item", 0), ("middle_item", 0)];
    assert!(matches!(
        dominance_of(&nothing_reserved),
        Dominance::NoItemAboveHalf {
            share_permille: 0,
            ..
        }
    ));
}

#[test]
fn moving_the_dominance_threshold_across_the_largest_share_flips_the_class() {
    let dominant = [
        ("heaviest_item", 60),
        ("middle_item", 25),
        ("lightest_item", 15),
    ];
    assert!(dominance_of(&dominant).is_dominant());
    assert_eq!(moved_threshold_permille(&dominant), 601);
    assert!(dominance_flips_when_the_threshold_is_moved(&dominant));
    let spread = [
        ("heaviest_item", 40),
        ("middle_item", 35),
        ("lightest_item", 25),
    ];
    assert!(!dominance_of(&spread).is_dominant());
    assert_eq!(moved_threshold_permille(&spread), 400);
    assert!(dominance_flips_when_the_threshold_is_moved(&spread));
    assert!(dominance_with_threshold(&spread, 400).is_dominant());
    assert!(!dominance_with_threshold(&spread, 401).is_dominant());
}

#[test]
fn mount_time_commitment_by_the_clauses_matches_the_hand_computed_value() {
    assert_eq!(
        mount_time_commitment_by_the_clauses(0, 10),
        4 * (32_768 + 3 * 10 * 16_384)
    );
    assert_eq!(
        mount_time_commitment_by_the_clauses(367, 10),
        4 * (2 * 32_768 + 3 * 10 * 16_384)
    );
    assert_eq!(
        checkpoint_cost_blocks_solved_from_the_commitment(
            mount_time_commitment_by_the_clauses(5, 13),
            5
        ),
        Some(13)
    );
    assert_eq!(
        checkpoint_cost_blocks_solved_from_the_commitment(
            mount_time_commitment_by_the_clauses(5, 13) + 1,
            5
        ),
        None
    );
}

fn snapshot_by_the_clauses(entry: ReadingEntry, rows: u64, blocks: u64) -> ReadingSnapshot {
    let device = |number: u32| DeviceTerms {
        device: number,
        capacity_bytes: unit_area_slots(64 * MEBIBYTE, MEBIBYTE) * SLOT_BYTES,
        allocated_bytes: 0,
        unreclaimable_bytes: 0,
        mount_time_commitment_bytes: mount_time_commitment_by_the_clauses(rows, blocks),
        abandoned_root_exclusive_bytes: 0,
        pending_delete_share_bytes: 0,
        committed_reservation_share_bytes: 0,
        checkpoint_reserve_pool_share_bytes: blocks * SLOT_BYTES,
        ring_window_bytes: 0,
        available_bytes: 0,
    };
    ReadingSnapshot {
        entry,
        instance_rows_of_the_switch_reserve: rows,
        replicas: 2,
        checkpoint_reserve_pool_bytes: blocks * SLOT_BYTES * 2,
        pending_delete_bytes: 0,
        committed_reservation_bytes: 0,
        retirement_tree_internal_levels: None,
        devices: vec![device(0), device(1)],
    }
}

#[test]
fn reading_checks_hold_on_a_reading_built_by_the_clauses_and_fail_when_one_item_is_off() {
    let consistent = snapshot_by_the_clauses(ReadingEntry::OfAWritableMount, 0, 10);
    let checks = checks_of_a_reading(&consistent, 64 * MEBIBYTE, MEBIBYTE, 8);
    assert!(checks.all_hold(), "{checks:?}");
    assert_eq!(checks.checkpoint_cost_blocks_by_the_pool, Some(10));
    let mut commitment_off = consistent.clone();
    commitment_off.devices[1].mount_time_commitment_bytes += SLOT_BYTES;
    assert!(!checks_of_a_reading(&commitment_off, 64 * MEBIBYTE, MEBIBYTE, 8).all_hold());
    let mut deleting_pending = consistent.clone();
    deleting_pending.pending_delete_bytes = 2;
    assert!(!checks_of_a_reading(&deleting_pending, 64 * MEBIBYTE, MEBIBYTE, 8).all_hold());
    let mut window_on_an_empty_pool = consistent;
    window_on_an_empty_pool.devices[0].ring_window_bytes = SLOT_BYTES;
    assert!(!checks_of_a_reading(&window_on_an_empty_pool, 64 * MEBIBYTE, MEBIBYTE, 8).all_hold());
}

#[test]
fn the_publish_entry_reads_one_more_instance_row_than_the_mount_entry() {
    assert_eq!(
        ReadingEntry::OfAWritableMount.instance_rows_of_the_switch_reserve(366),
        366
    );
    assert_eq!(
        ReadingEntry::BeforeAPublish.instance_rows_of_the_switch_reserve(366),
        367
    );
}

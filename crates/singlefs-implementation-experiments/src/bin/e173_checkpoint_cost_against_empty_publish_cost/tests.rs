use super::*;

#[test]
fn the_journal_ring_is_the_default_on_four_gibibytes_and_a_quarter_of_the_device_below_three_gibibytes(
) {
    assert_eq!(journal_ring_bytes_of_a_device(4 * GIBIBYTE), 805_306_368);
    assert_eq!(journal_ring_bytes_of_a_device(GIBIBYTE), 256 * MEBIBYTE);
    assert_eq!(journal_ring_bytes_of_a_device(32 * MEBIBYTE), 8 * MEBIBYTE);
    let four_gibibytes = Geometry {
        slots_per_region: 8,
        pool_size: PoolSize::FourGibibytes,
    };
    assert_eq!(
        four_gibibytes.local_unit_area(),
        (50176, 262_144),
        "B10：G-4 的单元区 [50176, 262144)"
    );
}

#[test]
fn the_smallest_pool_search_takes_32_mebibytes_when_every_trial_completes() {
    assert_eq!(
        smallest_completed_trial(&SMALLEST_POOL_TRIAL_MEBIBYTES, |_mebibytes| true),
        Some(32)
    );
}

#[test]
fn the_smallest_pool_search_takes_the_first_completed_trial_in_size_order_and_none_when_none_completes(
) {
    assert_eq!(
        smallest_completed_trial(&SMALLEST_POOL_TRIAL_MEBIBYTES, |mebibytes| mebibytes >= 128),
        Some(128)
    );
    assert_eq!(
        smallest_completed_trial(&SMALLEST_POOL_TRIAL_MEBIBYTES, |_mebibytes| false),
        None
    );
}

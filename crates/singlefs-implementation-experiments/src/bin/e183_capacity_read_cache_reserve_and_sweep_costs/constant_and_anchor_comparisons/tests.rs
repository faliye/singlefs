use super::*;

#[test]
fn local_geometry_reproduces_every_registered_script_row() {
    for anchor in &REGISTERED_RING_WINDOW_ANCHORS {
        let local = ring_window_by_the_clauses(
            anchor.device_bytes,
            anchor.journal_ring_bytes,
            anchor.slots_per_region,
            0,
        );
        assert!(
            anchor.is_reproduced_by(&local),
            "{} 字节、每区槽数 {}：{local:?}",
            anchor.device_bytes,
            anchor.slots_per_region
        );
    }
}

#[test]
fn local_constants_equal_the_implementation_constants() {
    for comparison in constants_against_the_implementation() {
        assert!(
            comparison.is_equal(),
            "{}：本地 {} 实现 {}",
            comparison.name,
            comparison.local,
            comparison.implementation
        );
    }
}

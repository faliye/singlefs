use super::*;

/// 几何核两半（登记第五节「负载」第 4 条，后一半照第十二节修订第 8 条判强制选段后备点名的那一段）：碎片段九成以上恰好 32 槽活，
/// 点名的那一段只装活数据单元、活槽不多于 32；greedy 那一段只报，它选中整段满的也不让几何判不成。
#[test]
fn the_geometry_is_formed_only_when_the_written_segments_are_half_live_and_the_forced_segment_is_a_half_live_data_segment(
) {
    let records: Vec<AllocationRecord> = [(0, 16), (1, 32), (3, 8)]
        .into_iter()
        .flat_map(|(segment_index, units)| data_unit_records_in_the_segment(segment_index, units))
        .collect();
    let unit_area = UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: UnitAreaStart::of_the_default_journal_ring(),
        slots: 4 * LOCAL_CLUSTER_SEGMENT_SLOTS,
    };
    let data_slots: BTreeSet<u64> = records.iter().map(|record| record.slot.0).collect();
    let segment_start = |segment_index: u64| {
        LOCAL_UNIT_AREA_START_SLOT + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS
    };
    let half_formed = geometry_of_the_records(
        &records,
        unit_area,
        &[segment_start(0), segment_start(1)].into(),
        &data_slots,
        Some(segment_start(1)),
        Some(segment_start(3)),
    );
    assert_eq!(half_formed.written_segments, 2);
    assert_eq!(half_formed.written_segments_with_half_live, 1);
    assert_eq!(
        half_formed.forced,
        SegmentOfTheChoice {
            start_slot: Some(segment_start(3)),
            live_slots: 16,
            holds_only_data_units: true,
        }
    );
    assert_eq!(
        half_formed.greedy,
        SegmentOfTheChoice {
            start_slot: Some(segment_start(1)),
            live_slots: 64,
            holds_only_data_units: true,
        }
    );
    assert_eq!(
        half_formed.live_slot_histogram,
        BTreeMap::from([(0, 1), (16, 1), (32, 1), (64, 1)])
    );
    assert!(!half_formed.is_formed());
    // greedy 选中整段满的那一段，强制点名的是只装数据、16 槽活的那一段：照点名的那一段判，几何成。
    let formed = geometry_of_the_records(
        &records,
        unit_area,
        &[segment_start(0)].into(),
        &data_slots,
        Some(segment_start(1)),
        Some(segment_start(3)),
    );
    assert!(!formed.greedy.is_fragment_like());
    assert!(formed.is_formed());
    // 点名的那一段装着别的单元（不在活数据单元的槽里）：判不成。
    let forced_on_other_units = geometry_of_the_records(
        &records,
        unit_area,
        &[segment_start(0)].into(),
        &BTreeSet::new(),
        Some(segment_start(3)),
        Some(segment_start(3)),
    );
    assert!(!forced_on_other_units.forced.holds_only_data_units);
    assert!(!forced_on_other_units.is_formed());
    // 点名的那一段活槽超过 32：判不成。
    let forced_on_a_full_segment = geometry_of_the_records(
        &records,
        unit_area,
        &[segment_start(0)].into(),
        &data_slots,
        Some(segment_start(3)),
        Some(segment_start(1)),
    );
    assert_eq!(forced_on_a_full_segment.forced.live_slots, 64);
    assert!(!forced_on_a_full_segment.is_formed());
    let nothing_forced = geometry_of_the_records(
        &records,
        unit_area,
        &[segment_start(0)].into(),
        &data_slots,
        Some(segment_start(3)),
        None,
    );
    assert!(!nothing_forced.is_formed());
}

/// 强制选段后备的点名法（登记第十二节修订第 8 条）：碎片段里还没点过名的、只装活数据单元、有活槽而不多于 32 的段里起点最小的那一段；
/// 用完交回 `None`。
#[test]
fn the_forced_fallback_names_the_lowest_fragment_segment_not_yet_forced_holding_only_data_units_and_at_most_half_live(
) {
    let records: Vec<AllocationRecord> = [(0, 16), (1, 32), (2, 16), (3, 8)]
        .into_iter()
        .flat_map(|(segment_index, units)| data_unit_records_in_the_segment(segment_index, units))
        .collect();
    let unit_area = UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: UnitAreaStart::of_the_default_journal_ring(),
        slots: 5 * LOCAL_CLUSTER_SEGMENT_SLOTS,
    };
    let segment_start = |segment_index: u64| {
        LOCAL_UNIT_AREA_START_SLOT + segment_index * LOCAL_CLUSTER_SEGMENT_SLOTS
    };
    // 头一段里有一个单元不是活数据单元；第二段整段满；最后一段空着。
    let data_slots: BTreeSet<u64> = records
        .iter()
        .map(|record| record.slot.0)
        .filter(|slot| *slot != segment_start(0))
        .collect();
    let live_slots = live_slots_per_cluster_segment_by_the_records(&records, unit_area);
    let segments_holding_other_records =
        segments_holding_a_record_other_than_a_data_unit(&records, unit_area, &data_slots);
    assert_eq!(
        segments_holding_other_records,
        BTreeSet::from([segment_start(0)])
    );
    let fragment_segment_starts: BTreeSet<u64> = (0..5).map(segment_start).collect();
    let forced_after = |already_forced: &[u64]| {
        fragment_segment_to_force(
            &fragment_segment_starts,
            &already_forced.iter().copied().collect::<BTreeSet<u64>>(),
            LOCAL_UNIT_AREA_START_SLOT,
            &live_slots,
            &segments_holding_other_records,
        )
    };
    assert_eq!(
        forced_after(&[]),
        Some(SegmentOfTheChoice {
            start_slot: Some(segment_start(2)),
            live_slots: 32,
            holds_only_data_units: true,
        })
    );
    assert_eq!(
        forced_after(&[segment_start(2)]).and_then(|segment| segment.start_slot),
        Some(segment_start(3))
    );
    assert_eq!(forced_after(&[segment_start(2), segment_start(3)]), None);
}

use super::*;

#[test]
fn unit_scan_on_wide_reads_every_death_also_when_keys_are_reborn() {
    for specification in [
        HistorySpecification::MixedRandom { seed: 1 },
        HistorySpecification::BuildAndDeleteEmptyInodes {
            inode_count: 300,
            order: DeletionOrder::Random(2),
        },
    ] {
        let simulation = simulate(specification, Geometry::Wide, WritePath::WithoutTombstones, None, false);
        assert!(judged(&simulation, "A3_unit_scan_on_wide_reads_every_death") > 0);
        assert_eq!(violations(&simulation, "A3_unit_scan_on_wide_reads_every_death"), 0, "{}", specification.label());
    }
}

#[test]
fn death_records_read_zero_on_every_geometry() {
    for geometry in [Geometry::Ring, Geometry::Wide] {
        for specification in [
            SMALL_REWRITE,
            HistorySpecification::BuildAndDeleteEmptyInodes {
                inode_count: 300,
                order: DeletionOrder::EvenThenOdd,
            },
        ] {
            let simulation = simulate(specification, geometry, WritePath::WithoutTombstones, None, false);
            assert_eq!(violations(&simulation, "A4_death_records_zero"), 0, "{} {}", geometry.name(), specification.label());
            assert_eq!(violations(&simulation, "A13_death_records_zero_on_ring"), 0, "{} {}", geometry.name(), specification.label());
            assert!(judged(&simulation, "A4_death_records_zero") + judged(&simulation, "A13_death_records_zero_on_ring") > 0);
        }
    }
}

#[test]
fn the_incremental_unit_scan_equals_the_brute_force_scan_at_every_scan() {
    for (specification, write_path) in [
        (
            HistorySpecification::LargeFileRewriteAndDelete {
                data_units: 145,
                rewrites: 5,
                overwrite_before_unlink: true,
                batch: 1,
            },
            WritePath::WithoutTombstones,
        ),
        (
            HistorySpecification::BuildAndDeleteEmptyInodes {
                inode_count: 300,
                order: DeletionOrder::Random(1),
            },
            WritePath::WithTombstones,
        ),
    ] {
        let simulation = simulate(specification, Geometry::Ring, write_path, None, true);
        assert!(judged(&simulation, "unit_scan_incremental_equals_brute_force") > 10);
        assert_eq!(violations(&simulation, "unit_scan_incremental_equals_brute_force"), 0, "{}", specification.label());
    }
}

#[test]
fn the_ring_window_reaches_twenty_three_publishes_back_and_not_twenty_four() {
    let mut read = KeysWithProviders::new();
    let provider_released_at = |released_at: u64| Provider {
        newest_birth: 1,
        freshest_release: Some(released_at),
        has_live_provider: false,
        provider_count: 1,
    };
    read.insert(single_field_key(7), provider_released_at(100 - 23));
    read.insert(single_field_key(8), provider_released_at(100 - 24));
    let measurement = measure_against_truth(&read, &BTreeSet::new(), 100);
    assert_eq!(measurement.resurrected, 2);
    assert_eq!(measurement.resurrected_in_window, 1);
}

#[test]
fn the_trajectory_keeps_the_peak_after_the_value_falls_back() {
    let mut trajectory = Trajectory::default();
    for (txg, resurrected) in [(3, 0), (4, 3), (5, 0)] {
        trajectory.observe(
            txg,
            CellMeasurement {
                resurrected,
                missed: resurrected,
                resurrected_in_window: 0,
            },
            1,
            false,
        );
    }
    assert_eq!(trajectory.resurrected_peak, 3);
    assert_eq!(trajectory.resurrected_peak_at, 4);
    assert_eq!(trajectory.resurrected_positive_scans, 1);
    assert_eq!(trajectory.resurrected_final, 0);
    assert_eq!(trajectory.missed_peak, 3);
    assert_eq!(trajectory.outside_window_peak, 3);
}

#[test]
fn tombstone_points_only_at_its_own_object_birth_and_its_pointer_range() {
    let tombstones = [TombstoneRecordSummary {
        object: 5,
        object_birth: 10,
        death_txg: 20,
    }];
    let record = |object_birth: u64| OccurrenceTag::InodeRecord { inode: 5, object_birth };
    assert!(tombstone_suppresses(&tombstones, record(10)));
    assert!(!tombstone_suppresses(&tombstones, record(11)));
    let extent = |pointer_birth: u64| OccurrenceTag::ExtentRecord { inode: 5, pointer_birth };
    assert!(!tombstone_suppresses(&tombstones, extent(9)));
    assert!(tombstone_suppresses(&tombstones, extent(10)));
    assert!(tombstone_suppresses(&tombstones, extent(20)));
    assert!(!tombstone_suppresses(&tombstones, extent(21)));
    assert!(tombstone_suppresses(&tombstones, OccurrenceTag::DeletedInode { inode: 5 }));
    assert!(!tombstone_suppresses(&tombstones, OccurrenceTag::Untagged));
}

fn handcrafted_unit(owner: UnitOwner, group: Option<SelectionGroup>, birth: u64, slot: u64, contents: UnitContents) -> Unit {
    Unit {
        owner,
        class: UNIT_CLASS_DATA,
        group,
        write_order: (birth, MOUNT_INSTANCE, 0),
        slot,
        span_slots: 2,
        release_txg: None,
        is_readable: true,
        mapping_key: Some([UNIT_CLASS_DATA, TREE_IDENTIFIER_EXTENT, birth, MOUNT_INSTANCE, slot]),
        contents,
    }
}

fn data_unit_of(object: u64, object_birth: u64, anchor_unit: u64, birth: u64, slot: u64) -> Unit {
    handcrafted_unit(
        UnitOwner::FileData,
        Some(SelectionGroup::DataUnit {
            birth_tree: TREE_IDENTIFIER_EXTENT,
            object,
            object_birth,
            anchor_unit,
        }),
        birth,
        slot,
        UnitContents::DataUnit {
            object,
            object_birth,
            anchor_unit,
        },
    )
}

fn inode_container_of(records: Vec<InodeRecordSummary>, birth: u64, slot: u64) -> Unit {
    handcrafted_unit(
        UnitOwner::InodeContainer,
        Some(SelectionGroup::PackedContainer {
            birth_tree: TREE_IDENTIFIER_INODE,
            record_type: PACKED_TYPE_INODE,
            container_number: 1,
            container_birth: 3,
        }),
        birth,
        slot,
        UnitContents::InodeRecords(records),
    )
}

fn read_handcrafted(units: Vec<Unit>, reader: Reader) -> ReadingResult {
    let mut disk = Disk::default();
    let mut lists = UnitLists::default();
    for unit in units {
        let unit_identifier = disk.units.len();
        match unit.owner {
            UnitOwner::FileData => lists.data_units.push(unit_identifier),
            UnitOwner::InodeContainer => lists.inode_containers.push(unit_identifier),
            UnitOwner::ExtentLowerNode
            | UnitOwner::ExtentRoot
            | UnitOwner::InodeRoot
            | UnitOwner::AllocationNode
            | UnitOwner::AccountingNode
            | UnitOwner::MappingNode
            | UnitOwner::DeletedInodesRoot
            | UnitOwner::TombstoneContainer
            | UnitOwner::TombstoneRoot
            | UnitOwner::TreeTable
            | UnitOwner::InstanceTable => {}
        }
        disk.write(unit);
    }
    let death_time = HashMap::new();
    let inputs = ScanInputs {
        disk: &disk,
        unit_lists: &lists,
        death_time: &death_time,
        planted: None,
        erased: None,
    };
    select_current_reading(reader, Arm::WithoutTombstones, &inputs)
}

#[test]
fn the_gated_reader_drops_data_units_whose_object_has_no_inode_record() {
    let units = vec![data_unit_of(7, 4, 0, 4, 60_000)];
    let select_current = read_handcrafted(units.clone(), Reader::SelectCurrent);
    let gated = read_handcrafted(units, Reader::SelectCurrentGatedByObject);
    assert!(select_current.keys[Namespace::Extent.position()].contains_key(&extent_key(7, 0)));
    assert!(gated.keys[Namespace::Extent.position()].is_empty());
    let with_record = vec![
        data_unit_of(7, 4, 0, 4, 60_000),
        inode_container_of(vec![InodeRecordSummary { inode: 7, object_birth: 4, nlink: 1 }], 5, 60_010),
    ];
    let gated_with_record = read_handcrafted(with_record, Reader::SelectCurrentGatedByObject);
    assert!(gated_with_record.keys[Namespace::Extent.position()].contains_key(&extent_key(7, 0)));
}

#[test]
fn select_current_takes_the_newest_version_of_a_group() {
    let units = vec![data_unit_of(1, 3, 0, 5, 60_000), data_unit_of(1, 3, 0, 7, 60_002)];
    let reading = read_handcrafted(units, Reader::SelectCurrent);
    let slots: Vec<KeyFields> = reading.keys[Namespace::AllocationUnitsOfTheVersion.position()]
        .keys()
        .copied()
        .collect();
    assert_eq!(slots, vec![single_field_key(60_002)]);
}

#[test]
fn deleted_inodes_select_current_reads_the_records_with_nlink_zero() {
    let units = vec![inode_container_of(
        vec![
            InodeRecordSummary { inode: 4, object_birth: 3, nlink: 0 },
            InodeRecordSummary { inode: 5, object_birth: 3, nlink: 1 },
        ],
        6,
        60_000,
    )];
    let reading = read_handcrafted(units, Reader::SelectCurrent);
    let deleted: Vec<KeyFields> = reading.keys[Namespace::DeletedInodes.position()].keys().copied().collect();
    assert_eq!(deleted, vec![single_field_key(4)]);
    assert_eq!(reading.keys[Namespace::Inode.position()].len(), 2);
}

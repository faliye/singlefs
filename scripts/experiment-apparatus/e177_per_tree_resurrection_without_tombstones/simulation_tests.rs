use super::*;

fn trajectory_of(simulation: &Simulation, arm: Arm, reader: Reader, tree: ReportedTree) -> Trajectory {
    simulation
        .trajectories
        .get(&(arm, reader, tree))
        .cloned()
        .expect("每一格都有轨迹")
}

#[test]
fn accounting_rows_die_fifteen_per_version_and_unlink_takes_one_publish_per_batch() {
    let specification = HistorySpecification::LargeFileRewriteAndDelete {
        data_units: 3,
        rewrites: 0,
        overwrite_before_unlink: false,
        batch: 1,
    };
    let simulation = simulate(specification, Geometry::Wide, WritePath::WithoutTombstones, None, false);
    // 起点 3 次（txg 3、4、5）+ 顺序写 1 次 + unlink 4 次（意图 + 两批 + 最后一批）。
    assert_eq!(simulation.publishes, 8);
    assert_eq!(simulation.scans, 8);
    let accounting = trajectory_of(&simulation, Arm::WithoutTombstones, Reader::UnitScan, ReportedTree::Accounting);
    assert_eq!(accounting.deaths_final, 105);
    assert_eq!(simulation.unlink_publish_counts[0].publishes, 4);
    assert_eq!(violations(&simulation, "A2_accounting_rows_per_version"), 0);
    assert_eq!(violations(&simulation, "A5_A10_unlink_publishes"), 0);
}

#[test]
fn empty_inode_build_and_delete_family_with_todays_tombstones_builds_and_drops_containers_as_the_clauses_say() {
    let specification = HistorySpecification::BuildAndDeleteEmptyInodes {
        inode_count: 300,
        order: DeletionOrder::Ascending,
    };
    let simulation = simulate(specification, Geometry::Ring, WritePath::WithTombstones, None, false);
    assert_eq!(simulation.version.tombstone_containers.len(), 1);
    assert_eq!(simulation.version.tombstone_containers[0].records.len(), 299);
    assert!(judged(&simulation, "K4_emptied_container_not_written") >= 1);
    for name in [
        "K3_tombstone_death_txg_is_the_intent_txg",
        "K4_containers_after_building",
        "K4_container_number_is_its_first_record",
        "K4_emptied_container_not_written",
        "A5_A10_unlink_publishes",
        "A15_reuse_waits_a_ring_length",
    ] {
        assert_eq!(violations(&simulation, name), 0, "{name}");
    }
    let inode = trajectory_of(&simulation, Arm::TodaysTombstones, Reader::UnitScan, ReportedTree::Inode);
    assert_eq!(inode.deaths_final, 299);
    let deleted = trajectory_of(&simulation, Arm::TodaysTombstones, Reader::UnitScan, ReportedTree::DeletedInodes);
    assert_eq!(deleted.deaths_final, 299);
    let tombstone = trajectory_of(&simulation, Arm::TodaysTombstones, Reader::UnitScan, ReportedTree::Tombstone);
    assert_eq!(tombstone.deaths_peak, 0);
    // 建满 300 个时两片容器，号 1 与 234；删 234..300 之后右边那片删空不落盘。
    assert_eq!(simulation.version.inode_containers.len(), 1);
    assert_eq!(simulation.version.inode_containers[0].container_number, 1);
}

#[test]
fn an_overwrite_before_unlink_kills_all_but_one_extent_key_and_releases_every_old_data_unit() {
    let simulation = simulate(SMALL_REWRITE, Geometry::Wide, WritePath::WithoutTombstones, None, false);
    assert_eq!(judged(&simulation, "A12_overwrite_kills_all_but_one_extent_key"), 1);
    assert_eq!(violations(&simulation, "A12_overwrite_kills_all_but_one_extent_key"), 0);
    assert_eq!(violations(&simulation, "A12_overwrite_releases_every_old_data_unit"), 0);
    let extent = trajectory_of(&simulation, Arm::WithoutTombstones, Reader::UnitScan, ReportedTree::Extent);
    assert_eq!(extent.deaths_final, 3);
    assert_eq!(simulation.unlink_publish_counts[0].publishes, 2);
    assert_eq!(simulation.unlink_publish_counts[0].data_units_at_the_intent, 1);
}

#[test]
fn user_data_never_lands_in_a_cluster_segment_of_this_mount() {
    let specification = HistorySpecification::LargeFileRewriteAndDelete {
        data_units: 300,
        rewrites: 0,
        overwrite_before_unlink: false,
        batch: PRODUCT_UNLINK_BATCH_DATA_UNITS,
    };
    let simulation = simulate(specification, Geometry::Ring, WritePath::WithoutTombstones, None, false);
    assert!(judged(&simulation, "A18_user_data_outside_cluster_segments") >= 300);
    assert_eq!(violations(&simulation, "A18_user_data_outside_cluster_segments"), 0);
    assert_eq!(violations(&simulation, "A18_opened_segment_was_empty"), 0);
    // txg 6 的 300 个数据单元要越过写行那次开的段 50304：落点里有越过它的，也没有落进它的。
    let sequential_write = &simulation.placement_log[3];
    assert!(sequential_write.iter().take(300).any(|(slot, _)| *slot >= UNIT_AREA_START_SLOT + 3 * CLUSTER_SEGMENT_SLOTS));
    assert!(!sequential_write
        .iter()
        .take(300)
        .any(|(slot, _)| (UNIT_AREA_START_SLOT + 2 * CLUSTER_SEGMENT_SLOTS..UNIT_AREA_START_SLOT + 3 * CLUSTER_SEGMENT_SLOTS).contains(slot)));
}

#[test]
fn readable_units_equal_written_minus_overwritten_on_the_ring_and_when_tight() {
    let mut ring = Simulation::start(Geometry::Ring, WritePath::WithoutTombstones, PRODUCT_UNLINK_BATCH_DATA_UNITS, None, false);
    for _ in 0..40 {
        ring.apply(UserChange::Overwrite);
    }
    assert!(judged(&ring, "A19_readable_units_equal_written_minus_overwritten") >= 40);
    assert_eq!(violations(&ring, "A19_readable_units_equal_written_minus_overwritten"), 0);
    assert!(ring.disk.units_made_unreadable > 0);
    let wide = simulate(SMALL_REWRITE, Geometry::Wide, WritePath::WithoutTombstones, None, false);
    let tight = simulate(
        SMALL_REWRITE,
        Geometry::Tight,
        WritePath::WithoutTombstones,
        Some(tight_unit_area_slots_from(wide.slots_held_during_a_publish_peak)),
        false,
    );
    assert_eq!(tight.does_not_fit_at, None);
    assert!(tight.disk.units_made_unreadable > 0);
    assert_eq!(wide.disk.units_made_unreadable, 0);
    assert_eq!(violations(&tight, "A19_readable_units_equal_written_minus_overwritten"), 0);
    assert_eq!(violations(&tight, "tight_reuse_only_of_released_before_this_publish"), 0);
}

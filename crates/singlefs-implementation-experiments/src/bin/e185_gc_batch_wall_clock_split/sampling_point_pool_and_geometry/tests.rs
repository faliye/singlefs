use super::*;

/// N11：一段 16 个这次写的槽、2 个别的槽（节点），已释放的不算；单元区起点之下的记录不进逐段成分；活槽 = 16 且全是这次写的那一段在占比 0.25 下算造成。
#[test]
fn segment_composition_counts_this_workload_and_other_slots_apart() {
    let mut live_data_inode_by_slot = BTreeMap::new();
    let mut records = vec![record(50_000, 1, false)];
    for index in 0..8_u64 {
        let slot = 50_304 + 2 * index;
        records.push(record(slot, 2, false));
        live_data_inode_by_slot.insert(slot, 100 + index);
        let slot_in_the_next_segment = 50_368 + 2 * index;
        records.push(record(slot_in_the_next_segment, 2, false));
        live_data_inode_by_slot.insert(slot_in_the_next_segment, 200 + index);
    }
    records.push(record(50_320, 1, false));
    records.push(record(50_321, 1, false));
    records.push(record(50_330, 2, true));
    let composition =
        composition_of_the_segment(&records, region_at(50_304), &live_data_inode_by_slot);
    assert_eq!(
        composition,
        SegmentComposition {
            start_slot: 50_304,
            live_slots: 18,
            slots_of_this_workload: 16,
            other_slots: 2,
        }
    );
    let every_segment = compositions_of_every_segment_holding_an_unreleased_record(
        &records,
        50_304,
        &live_data_inode_by_slot,
    );
    assert_eq!(
        every_segment
            .iter()
            .map(|segment| segment.start_slot)
            .collect::<Vec<_>>(),
        vec![50_304, 50_368]
    );
    let point = SamplingPoint {
        fraction: LiveSlotFraction::OneQuarter,
        pool_data_units: 1024,
    };
    let lines: Vec<String> = every_segment
        .iter()
        .map(|segment| {
            segment_composition_line(point, OverwriteOrder::HighestFullSegmentLast, 0, *segment)
        })
        .collect();
    assert!(
        lines[0].ends_with(
            "region_start_slot=50304 live_slots=18 slots_of_this_workload=16 other_slots=2 formed=false"
        ),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].ends_with(
            "region_start_slot=50368 live_slots=16 slots_of_this_workload=16 other_slots=0 formed=true"
        ),
        "{}",
        lines[1]
    );
}

/// N11、N12：成分汇总把逐段数出的造成段数与几何核那条路数的并列、对不上写 `counts_agree=false`；数了落点的写三个数与预测，没数的写 `status=not_run`。
#[test]
fn construction_reading_lines_compare_the_two_counts_of_formed_segments_and_report_the_landing() {
    let recipe = PoolRecipe {
        point: SamplingPoint {
            fraction: LiveSlotFraction::OneQuarter,
            pool_data_units: 1024,
        },
        overwrite_order: OverwriteOrder::HighestFullSegmentLast,
        extra_overwritten_inodes: BTreeSet::new(),
    };
    let mut pool = constructed_pool_without_devices();
    pool.overwritten_units = 776;
    pool.overwrite_landing = Some(OverwriteLanding {
        copies_landing_in_holes_of_this_construction: 752,
        copies_landing_elsewhere: 24,
    });
    let prediction_with = |geometry_formed_segments: Vec<CompactionRegion>| GreedyPrediction {
        greedy_region: None,
        greedy_composition: None,
        geometry_formed_segments,
        mapped_units: 0,
        segment_compositions: vec![
            SegmentComposition {
                start_slot: 50_304,
                live_slots: 16,
                slots_of_this_workload: 16,
                other_slots: 0,
            },
            SegmentComposition {
                start_slot: 50_368,
                live_slots: 18,
                slots_of_this_workload: 16,
                other_slots: 2,
            },
        ],
    };
    let agreeing =
        construction_reading_lines(&recipe, &pool, &prediction_with(vec![region_at(50_304)]), 0);
    assert_eq!(agreeing.len(), 4);
    assert!(agreeing[0].ends_with(
        "region_start_slot=50304 live_slots=16 slots_of_this_workload=16 other_slots=0 formed=true"
    ));
    assert!(
        agreeing[2].ends_with("segments=2 formed_segments=1 geometry_formed_segments_of_the_attempt=1 counts_agree=true"),
        "{}",
        agreeing[2]
    );
    assert!(
        agreeing[3].ends_with("status=counted overwritten_units=776 copies_landing_in_holes_of_this_construction=752 copies_landing_elsewhere=24 predicted_copies_landing_in_holes=752 root_ring_entries=24"),
        "{}",
        agreeing[3]
    );
    let disagreeing = construction_reading_lines(&recipe, &pool, &prediction_with(Vec::new()), 1);
    assert!(
        disagreeing[2].ends_with("geometry_formed_segments_of_the_attempt=0 counts_agree=false")
    );
    pool.overwrite_landing = None;
    let not_counted = construction_reading_lines(&recipe, &pool, &prediction_with(Vec::new()), 1);
    assert!(not_counted[3].ends_with("workload_changes=1 status=not_run overwritten_units=776"));
}

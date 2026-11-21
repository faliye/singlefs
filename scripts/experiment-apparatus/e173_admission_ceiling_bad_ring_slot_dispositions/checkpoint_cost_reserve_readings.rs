use super::*;

fn geometry_row(slots_per_region: u64, pool_size: SampledPoolSize, largest_actual_empty_publish_slots: i64, checkpoint_cost_slots: i64, product_line: usize) -> CheckpointCostGeometry {
    CheckpointCostGeometry {
        slots_per_region,
        pool_size,
        largest_actual_empty_publish_slots,
        checkpoint_cost_slots,
        product_line,
        first_request_under_the_code_reading: None,
        first_request_under_the_clause_reading: None,
    }
}

/// 场景：κ = 529、c实 = 5（两块 4 GiB 盘上 K1 那一项的量级，`reserve_fit.py 5 529` 那一组）。
/// 预期：代码读法 P = 529，条款读法 P = 10 + 7 × 529 = 3713，改回 10 + 7c 的 P = 45；原两种读法不带池大小、键后面不多打字段。
#[test]
fn checkpoint_cost_readings_take_kappa_ten_plus_seven_kappa_and_ten_plus_seven_actual_empty_publishes() {
    let [code_reading, clause_reading, reverted_reading] = checkpoint_cost_readings_of(geometry_row(4, SampledPoolSize::FourGibibytes, 5, 529, 1));
    assert_eq!(code_reading.reserve_blocks(5), 529, "代码读法 P代 = κ_G");
    assert_eq!(clause_reading.reserve_blocks(5), 10 + 3703, "条款读法 P条 = 10 + 7 × 529（reserve_fit.py 5 529 的 reading=clause P=3713）");
    assert_eq!(reverted_reading.reserve_blocks(5), 10 + 35, "改回 10 + 7c：c实 = 5 时 45");
    assert_eq!((code_reading.label(), clause_reading.label(), reverted_reading.label()), ("ckpt-code", "ckpt-clause", "reverted-10+7c"));
    assert_eq!(code_reading.extra_key_fields(), " pool_size=g_4 kappa=529");
    for old_reading in RESERVE_READINGS {
        assert_eq!((old_reading.pool_size_label(), old_reading.extra_key_fields()), ("", String::new()), "原两种读法的产物行不变（停机条款 S1）");
    }
}

/// 场景：一个几何 κ = 14、c实 = 3。预期：池的 c 取 c实 = 3（不是 κ），每种读法 × 三种池松紧各一份，共 9 份。
#[test]
fn checkpoint_cost_pools_take_the_actual_empty_publish_as_the_metadata_blocks() {
    let pools = checkpoint_cost_pool_settings(&[geometry_row(8, SampledPoolSize::SmallestCompleting { mebibytes: 64 }, 3, 14, 1)]);
    assert_eq!(pools.len(), 9);
    assert!(pools.iter().all(|pool| pool.metadata_blocks == 3 && pool.geometry.slots_per_region == 8), "c 取 c实_G、S 取几何的 S");
    assert_eq!(pools.iter().filter(|pool| pool.reserve_blocks() == 14).count(), 3, "代码读法 P = κ = 14 的三份");
}

/// 场景：PC5（第七节 B8）——κ = 5、c = 5、S = 4、H1、近满 k = 0、不注读故障，八对臂 × 三种读法。
/// 预期：P = 5 那一串落不了点被拒，P = 45 一串抬 F 之后发成（上限 47、F 47、空闲 − P 50）；24 格全对、没有作废。
#[test]
fn positive_control_reproduces_the_first_request_at_reserve_five_and_forty_five_on_every_arm() {
    let mut emitter = Emitter::new();
    let mut tally = RunTally::default();
    emit_the_checkpoint_cost_positive_control(&mut emitter, &mut tally);
    assert_eq!((tally.self_checks_passed, tally.void_conditions.clone()), (24, Vec::<String>::new()));
}

fn segment_one_product_text() -> String {
    [
        "E7RESULT name=config section=segment-one",
        "E7RESULT name=x3_reserve_fit geometry=s4_g_4 kappa_of_the_geometry=minimum_on_versions_with_trees kappa=529 largest_actual_empty_publish_slots=5 code_reserve=529 clause_reserve=3713 code_empty_publishes_held=105 clause_empty_publishes_held=742",
        "E7RESULT name=x3_reserve_fit geometry=s4_g_4 kappa_of_the_geometry=maximum_on_versions_with_trees kappa=531 largest_actual_empty_publish_slots=5 code_reserve=531 clause_reserve=3727 code_empty_publishes_held=106 clause_empty_publishes_held=745",
        "E7RESULT name=x3_reserve_fit geometry=s16_g_min_32mib kappa_of_the_geometry=minimum_on_versions_with_trees kappa=14 largest_actual_empty_publish_slots=3 code_reserve=14 clause_reserve=108 code_empty_publishes_held=4 clause_empty_publishes_held=36",
    ]
    .join("\n")
}

/// 场景：第一段产物两行（取最小 κ 的那一种），常量表照抄、改一个数、少一行。
/// 预期：照抄对得上；κ 改一个数、行号写错、少一行各报一处对不上（停机条款 S5）。
#[test]
fn the_geometry_table_is_checked_against_the_segment_one_product() {
    let product = segment_one_product_text();
    let copied = [geometry_row(4, SampledPoolSize::FourGibibytes, 5, 529, 2), geometry_row(16, SampledPoolSize::SmallestCompleting { mebibytes: 32 }, 3, 14, 4)];
    assert_eq!(segment_one_mismatches(&copied, &product), Vec::<String>::new());
    let one_number_off = [geometry_row(4, SampledPoolSize::FourGibibytes, 5, 530, 2), copied[1]];
    assert_eq!(segment_one_mismatches(&one_number_off, &product), vec!["s4_g_4:kappa_table=530_product=529".to_string()]);
    let wrong_line = [geometry_row(4, SampledPoolSize::FourGibibytes, 5, 529, 3), copied[1]];
    assert_eq!(segment_one_mismatches(&wrong_line, &product), vec!["s4_g_4:line_table=3_product=2".to_string()]);
    assert_eq!(segment_one_mismatches(&copied[..1], &product), vec!["s16_g_min_32mib:not_in_the_table".to_string()]);
}

/// 场景：第一段产物的几何写法。预期：三档池大小与 S 都读得回、与 [`SampledPoolSize::label`] 往返一致；认不出的写法交回 None。
#[test]
fn geometry_labels_of_the_segment_one_product_read_back_to_the_pool_sizes() {
    for (expected, label) in [
        ((8, SampledPoolSize::OneGibibyte), "s8_g_1"),
        ((4, SampledPoolSize::FourGibibytes), "s4_g_4"),
        ((16, SampledPoolSize::SmallestCompleting { mebibytes: 128 }), "s16_g_min_128mib"),
    ] {
        assert_eq!(SampledPoolSize::of_the_geometry_label(label), Some(expected));
        assert_eq!(format!("s{}_{}", expected.0, expected.1.label()), label);
    }
    for unrecognized in ["s8_g_7", "x8_g_1", "s8_g_min_mib"] {
        assert_eq!(SampledPoolSize::of_the_geometry_label(unrecognized), None, "{unrecognized}");
    }
}

fn readings_with(slots_per_region: u64, baseline_rejected: i64, verdict_hits_by_row: &[(&str, bool)]) -> ReadingsOfACell {
    ReadingsOfACell {
        slots_per_region,
        pool_size: "g_4",
        fullness: PoolFullness::NearlyFullWithNoSpareOverwrite,
        readings: (BTreeMap::new(), [TrajectoryAggregate::empty(); TRAJECTORY_SERIES], 0, Aggregate::empty(), baseline_rejected),
        verdict_hits_by_row: verdict_hits_by_row.iter().map(|(label, hit)| (label.to_string(), *hit)).collect(),
        baseline_rejected,
        baseline_first_rejected_request: (baseline_rejected > 0).then_some(0),
    }
}

/// 场景：一格上 Q1-1 代码读法打中、条款读法不打中；另一格 Q1-2 两边都打中。
/// 预期：Q1-1 那一个几何「只代码读法打中」1、「只条款读法打中」0；Q1-2「两边都打中」1；全部配得上对。
#[test]
fn reading_splits_count_the_code_only_hit_in_its_own_column() {
    let code = BTreeMap::from([("cell".to_string(), readings_with(4, 0, &[("Q1-1", true), ("Q1-2", true)]))]);
    let clause = BTreeMap::from([("cell".to_string(), readings_with(4, 0, &[("Q1-1", false), ("Q1-2", true)]))]);
    let (splits, unpaired) = reading_splits(&code, &clause);
    assert_eq!(unpaired, 0);
    assert_eq!(splits[&("Q1-1".to_string(), 4, "g_4")], ReadingSplit { cells: 1, only_code_hit: 1, only_clause_hit: 0, both_hit: 0 });
    assert_eq!(splits[&("Q1-2".to_string(), 4, "g_4")], ReadingSplit { cells: 1, only_code_hit: 0, only_clause_hit: 0, both_hit: 1 });
}

/// 场景：零点两种读法都不拒，读坏之下代码读法拒 1 次（`rejected` 那个量最大 1）；另一格零点只有代码读法拒、第一次落在第 0 次请求。
/// 预期：Z3 只数零点——第一格不算，第二格算「只代码读法零点被拒」1、代码读法第一次被拒落在 0、条款读法 none。
#[test]
fn the_zero_point_comparison_counts_the_baseline_and_not_the_run_with_the_read_fault() {
    let mut rejected_under_the_fault = readings_with(4, 0, &[]);
    let mut rejected_once = Aggregate::empty();
    rejected_once.add(1, 1);
    rejected_under_the_fault.readings.0.insert("rejected", rejected_once);
    let code = BTreeMap::from([("fault_only".to_string(), rejected_under_the_fault), ("zero_point".to_string(), readings_with(8, 1, &[]))]);
    let clause = BTreeMap::from([("fault_only".to_string(), readings_with(4, 0, &[])), ("zero_point".to_string(), readings_with(8, 0, &[]))]);
    let counts = zero_point_counts(&code, &clause);
    assert_eq!(counts[&(4, "g_4", "near0")], (1, 0, 0, 0, None, None), "读坏之下的被拒不进 Z3");
    assert_eq!(counts[&(8, "g_4", "near0")], (1, 1, 0, 0, Some(0), None));
}

/// 场景：改回 10 + 7c 那一臂与代码读法在一格上读数相同、在另一格上 Q1-1 的判定不同。
/// 预期：判别力对照数出 2 格里 1 格读数不同、1 格 Y1 判定不同，摘要写「读数不同」。
#[test]
fn the_reverted_reserve_discrimination_counts_cells_whose_readings_differ() {
    let reverted = BTreeMap::from([("same".to_string(), readings_with(4, 0, &[("Q1-1", false)])), ("differs".to_string(), readings_with(4, 0, &[("Q1-1", true)]))]);
    let mut differing = readings_with(4, 1, &[("Q1-1", false)]);
    differing.readings.4 = 1;
    let code = BTreeMap::from([("same".to_string(), readings_with(4, 0, &[("Q1-1", false)])), ("differs".to_string(), differing)]);
    assert_eq!(reverted_reserve_differences(&reverted, &code), BTreeMap::from([((4, "g_4"), (2, 1, 1))]));
    let by_reserve = BTreeMap::from([("reverted-10+7c", reverted), ("ckpt-code", code.clone()), ("ckpt-clause", code)]);
    let mut emitter = Emitter::new();
    let mut tally = RunTally::default();
    emit_the_reverted_reserve_discrimination(&mut emitter, &mut tally, &by_reserve);
    assert_eq!((tally.self_checks_passed, tally.void_conditions.len()), (2, 0), "两条被比的臂都配得上对");
}

/// 场景：抄进常量表的 9 个几何（第一段产物那份）。
/// 预期：S 取 4、8、16 各三个池大小；κ_G 与 c实_G 是产物抄下的数（`reserve_fit.py` 的 B11 读数：18/6 装 3 次、130/6 装 21 次、543/10 装 54 次、26/6 装 4 次）；
/// 只有 S = 4 的三行带第一次请求的期望（B9）；行号两两不同且递增。
#[test]
fn the_registered_geometry_table_holds_the_nine_geometries_of_the_segment_one_product() {
    let table: Vec<(u64, &str, i64, i64, usize)> = CHECKPOINT_COST_GEOMETRIES
        .iter()
        .map(|row| (row.slots_per_region, row.pool_size.label(), row.largest_actual_empty_publish_slots, row.checkpoint_cost_slots, row.product_line))
        .collect();
    assert_eq!(
        table,
        vec![
            (4, "g_min_64mib", 6, 18, 3299),
            (4, "g_1", 6, 130, 3322),
            (4, "g_4", 10, 543, 3345),
            (8, "g_min_64mib", 6, 18, 3368),
            (8, "g_1", 6, 130, 3391),
            (8, "g_4", 10, 543, 3414),
            (16, "g_min_128mib", 6, 26, 3437),
            (16, "g_1", 6, 130, 3460),
            (16, "g_4", 10, 543, 3483),
        ]
    );
    let held: Vec<i64> = CHECKPOINT_COST_GEOMETRIES.iter().map(|row| row.checkpoint_cost_slots / row.largest_actual_empty_publish_slots).collect();
    assert_eq!(held, vec![3, 21, 54, 3, 21, 54, 4, 21, 54], "⌊κ_G ÷ c实_G⌋ 等于 reserve_fit.py 的 empties_held（第七节 B11）");
    for row in CHECKPOINT_COST_GEOMETRIES {
        assert_eq!(row.first_request_under_the_code_reading.is_some(), row.slots_per_region == 4, "只有 S = 4 的几何带 B9 的期望：{}", row.label());
        assert_eq!(row.first_request_under_the_clause_reading, row.first_request_under_the_code_reading, "两种读法的 空闲 − P 相同");
    }
    assert!(CHECKPOINT_COST_GEOMETRIES.windows(2).all(|pair| pair[0].product_line < pair[1].product_line));
}

/// 场景：G-4、S = 4 那一行（κ_G = 543、c实_G = 10），第七节 B9 的 `first_request.py 4 10 8 543` 与 `… 3811`。
/// 预期：代码读法 P = 543、条款读法 P = 10 + 7 × 543 = 3811、判别力对照 P = 10 + 7 × 10 = 80；
/// 第一次请求在一串抬 F（上限 47、三次发布）之后发成，发成之后 空闲 − P = 78 − 18 = 60（`first_request.py` 的 `user_room=78 need=18`）；
/// 代码读法的 P 若取 c（MR9）或 c 若取 κ（MR11），P = 10 或 c = 543 都装不下三次空发布，被拒。
#[test]
fn the_four_gibibyte_geometry_at_four_slots_pins_the_reserve_and_the_first_request_to_the_registered_values() {
    let row = CHECKPOINT_COST_GEOMETRIES[2];
    assert_eq!((row.slots_per_region, row.pool_size, row.largest_actual_empty_publish_slots, row.checkpoint_cost_slots), (4, SampledPoolSize::FourGibibytes, 10, 543));
    let [code_reading, clause_reading, reverted_reading] = checkpoint_cost_readings_of(row);
    assert_eq!(
        (code_reading.reserve_blocks(10), clause_reading.reserve_blocks(10), reverted_reading.reserve_blocks(10)),
        (543, 3811, 80),
        "reserve_fit.py 10 543 的 reading=code P=543、reading=clause P=3811；改回 10 + 7c 是 80"
    );
    let geometry = RingGeometry { slots_per_region: 4 };
    let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, 10);
    for (reserve_reading, expected_reserve) in [(code_reading, 543), (clause_reading, 3811)] {
        let settings = PoolSettings { geometry, metadata_blocks: 10, reserve_reading, fullness: PoolFullness::NearlyFullWithNoSpareOverwrite };
        assert_eq!(settings.reserve_blocks(), expected_reserve, "池里的 P：{}", reserve_reading.label());
        for arm in every_arm_pair() {
            assert_eq!(
                first_request_reading(&history, settings, arm),
                "accepted=true ceiling=47 sequence_publishes=3 floor=47 free_above_reserve_after=60",
                "{} {}",
                reserve_reading.label(),
                label_of_the_arm(arm)
            );
        }
    }
}

/// 场景：三个 S = 4 的几何 × 两种读法 × 八对臂，每一格的第一次请求对常量表里抄下的 B9 期望。
/// 预期：48 格全对、没有作废（运行时 PC6 同一件事；这里在产物之前就核装置的读数写法与 `first_request.py` 一致）。
#[test]
fn the_geometry_first_requests_of_the_three_four_slot_geometries_equal_the_registered_readings_on_every_arm() {
    let mut emitter = Emitter::new();
    let mut tally = RunTally::default();
    emit_the_geometry_first_requests(&mut emitter, &mut tally, CHECKPOINT_COST_GEOMETRIES);
    assert_eq!((tally.self_checks_passed, tally.void_conditions.clone()), (3 * 2 * 8, Vec::<String>::new()));
}

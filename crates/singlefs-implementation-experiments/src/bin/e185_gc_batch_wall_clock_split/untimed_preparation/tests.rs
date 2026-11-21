use super::*;

// ───────────── 不计时的准备并行（登记第十二节 12.10） ─────────────

#[test]
fn memory_cap_takes_the_environment_then_the_control_group_then_twenty_eight_gibibytes() {
    assert_eq!(
        memory_cap_given(Some("16G"), Some("8589934592\n")),
        MemoryCap {
            bytes: 17_179_869_184,
            source: MemoryCapSource::Environment
        }
    );
    assert_eq!(
        memory_cap_given(None, Some("8589934592\n")),
        MemoryCap {
            bytes: 8_589_934_592,
            source: MemoryCapSource::ControlGroup
        }
    );
    let default_cap = MemoryCap {
        bytes: 30_064_771_072,
        source: MemoryCapSource::Default,
    };
    assert_eq!(memory_cap_given(None, Some("max\n")), default_cap);
    assert_eq!(memory_cap_given(None, None), default_cap);
    assert_eq!(bytes_of_a_memory_size_text("512M"), Some(536_870_912));
    assert_eq!(bytes_of_a_memory_size_text("28"), Some(28));
    assert_eq!(bytes_of_a_memory_size_text("0G"), None);
    assert_eq!(bytes_of_a_memory_size_text("lots"), None);
}

#[test]
#[should_panic(expected = "E185_MEMORY_CAP")]
fn memory_cap_refuses_an_unreadable_environment_value() {
    let _cap = memory_cap_given(Some("lots"), None);
}

#[test]
fn pool_footprint_estimates_before_construction_at_the_registered_pool_sizes() {
    let estimate = |pool_data_units: u64| {
        estimated_pool_footprint_bytes_before_construction(SamplingPoint {
            fraction: LiveSlotFraction::OneQuarter,
            pool_data_units,
        })
    };
    // 两块盘 × (2P + 64) 个单元 × 32768 字节 × 2（元数据同等量）÷ 512 × 640
    assert_eq!(estimate(1024), 346_030_080);
    assert_eq!(estimate(8192), 2_694_840_320);
}

#[test]
fn pool_footprint_counts_six_hundred_forty_bytes_per_written_sector() {
    let mut devices = Vec::new();
    for (device_number, sectors) in [(0_u32, 3_u64), (1, 2)] {
        let mut device = SparseBlockDevice::new(
            LOCAL_DEVICE_BYTES,
            PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
        );
        for sector in 0..sectors {
            device
                .image
                .write(DeviceOffsetInBytes(sector * 4096), &[7_u8; 512]);
        }
        devices.push((DeviceIdentity(device_number), device));
    }
    let pool = ConstructedPool {
        devices,
        live_data_inode_by_slot: BTreeMap::new(),
        unit_area_start_slot: 0,
        overwritten_units: 0,
        filler_units: 0,
        floor_raises_pushed_by_the_session: 0,
        release_rounds: 0,
        is_every_overwritten_slot_released: true,
        construction_seconds: 0.0,
        overwrite_landing: None,
    };
    assert_eq!(footprint_bytes_of_the_pool(&pool), 3200, "5 个扇区 × 640");
}

#[test]
fn preparation_budget_subtracts_the_held_pools_and_the_measurement_headroom() {
    assert_eq!(
        preparation_budget_bytes(28 << 30, 4 << 30, 2 << 30),
        22 << 30
    );
    assert_eq!(preparation_budget_bytes(28 << 30, 0, 0), 28 << 30);
    assert_eq!(preparation_budget_bytes(4 << 30, 3 << 30, 2 << 30), 0);
}

#[test]
fn next_group_takes_the_smaller_of_the_threads_and_the_memory_budget_over_one_peak() {
    let peaks = vec![2_u64 << 30; 20];
    assert_eq!(
        tasks_of_the_next_group(&peaks, 31, 28 << 30),
        14,
        "28 GiB ÷ 2 GiB = 14 份，少于 31 条线程"
    );
    assert_eq!(
        tasks_of_the_next_group(&peaks, 4, 28 << 30),
        4,
        "线程少时按线程"
    );
    assert_eq!(
        tasks_of_the_next_group(&peaks[..3], 31, 28 << 30),
        3,
        "份数少时按份数"
    );
    assert_eq!(
        tasks_of_the_next_group(&[5 << 30, 5 << 30, 1 << 30, 1 << 30], 31, 11 << 30),
        3,
        "峰值不同时按次序累加到预算"
    );
}

#[test]
fn next_group_holds_none_when_one_peak_exceeds_the_budget_or_nothing_is_left() {
    assert_eq!(
        tasks_of_the_next_group(&[3 << 30, 3 << 30], 31, 2 << 30),
        0,
        "保底一份不在这里，在队列里"
    );
    assert_eq!(
        tasks_of_the_next_group(&[1 << 30, 3 << 30, 1 << 30], 31, 2 << 30),
        1,
        "装不下的那一份之后的都不派"
    );
    assert_eq!(tasks_of_the_next_group(&[], 31, 28 << 30), 0);
}

/// 假的准备：不造池、不建镜像。`point_whose_workload_changes` 那一点的几何核交回改过的配方（多覆盖写 inode 7），池的内存都报 `footprint_bytes`。
fn fake_preparation_outcome(
    work: PreparationWork,
    point_whose_workload_changes: Option<SamplingPoint>,
    footprint_bytes: u64,
) -> PreparationOutcome {
    match work {
        PreparationWork::PrepareThePoint(point) => {
            let mut recipe = first_recipe_of(point);
            if point_whose_workload_changes == Some(point) {
                recipe.extra_overwritten_inodes.insert(7);
            }
            PreparationOutcome::Prepared {
                prepared: PreparedPoint {
                    recipe,
                    choice: ChoiceOfTheRegion::Greedy,
                    is_the_geometry_formed: true,
                    pool: constructed_pool_without_devices(),
                    reference_digests: Vec::new(),
                    lines: vec![format!("name=geometry_attempt {}", point.label())],
                },
                footprint_bytes,
            }
        }
        PreparationWork::BuildTheRealDiskImages { .. } => {
            PreparationOutcome::RealDiskImages(Err("fake_images_not_built".to_string()))
        }
        PreparationWork::Rebuild(recipe) => PreparationOutcome::Rebuilt {
            rebuilt: RebuiltPool {
                recipe,
                pool: constructed_pool_without_devices(),
                digests: Vec::new(),
            },
            footprint_bytes,
        },
    }
}

#[test]
fn preparation_units_follow_the_measurement_order_with_the_images_before_rounds_two_to_five() {
    let rounds_of = |point_index: usize| {
        (2..=5).map(move |round| PreparationUnit::RebuildForTheRound { point_index, round })
    };
    let mut expected = Vec::new();
    for point_index in 0..2 {
        expected.push(PreparationUnit::PrepareThePoint { point_index });
        expected.push(PreparationUnit::BuildTheRealDiskImages { point_index });
        expected.extend(rounds_of(point_index));
    }
    assert_eq!(preparation_units_in_measurement_order(2, true), expected);
    assert_eq!(
        preparation_units_in_measurement_order(1, false),
        std::iter::once(PreparationUnit::PrepareThePoint { point_index: 0 })
            .chain(rounds_of(0))
            .collect::<Vec<_>>()
    );
}

#[test]
fn preparation_unit_peaks_are_two_pools_to_prepare_five_quarters_to_rebuild_and_one_pool_for_the_images(
) {
    let points = every_sampling_point();
    let settings = preparation_settings_with(31, 28 << 30);
    let worker = |work: PreparationWork| fake_preparation_outcome(work, None, 400_000_000);
    let directory = Path::new("e185-peak-directory-never-created");
    let mut queue =
        PreparationQueue::new("wall_clock", &points, Some(directory), &settings, &worker);
    let prepare_the_smallest = PreparationUnit::PrepareThePoint { point_index: 0 };
    assert_eq!(
        queue.peak_bytes_of(prepare_the_smallest),
        692_060_160,
        "两份池（造好的与几何核的拷贝），按估的"
    );
    assert_eq!(
        queue.peak_bytes_of(PreparationUnit::RebuildForTheRound {
            point_index: 1,
            round: 2
        }),
        3_368_550_400,
        "还没造好的点按估的 5/4"
    );
    assert_eq!(
        queue.headroom_bytes(),
        5_389_680_640,
        "余量是最大一份池（估的）的两倍"
    );
    let work = queue.start(prepare_the_smallest);
    queue.finish(prepare_the_smallest, worker(work));
    let rebuild = PreparationUnit::RebuildForTheRound {
        point_index: 0,
        round: 2,
    };
    assert_eq!(
        queue.peak_bytes_of(rebuild),
        500_000_000,
        "造好了按真写过的 5/4"
    );
    assert_eq!(
        queue.peak_bytes_of(PreparationUnit::BuildTheRealDiskImages { point_index: 0 }),
        400_000_000
    );
    assert_eq!(queue.accounted_bytes(), 400_000_000, "造好的池留在账上");
}

#[test]
fn first_start_under_twenty_eight_gibibytes_takes_fifteen_units_in_the_measurement_order_and_under_eight_takes_five(
) {
    let points = every_sampling_point();
    let directory = Path::new("e185-first-start-directory-never-created");
    let settings = preparation_settings_with(31, 28 << 30);
    let queue = PreparationQueue::new(
        "wall_clock",
        &points,
        Some(directory),
        &settings,
        &run_the_preparation_work,
    );
    let (units, is_guaranteed) = queue.units_to_start(31, true);
    let expected: Vec<PreparationUnit> = queue
        .order
        .iter()
        .copied()
        .filter(|unit| !matches!(unit, PreparationUnit::BuildTheRealDiskImages { .. }))
        .take(15)
        .collect();
    assert_eq!(
        units, expected,
        "按量的次序派；镜像要等那一点造好；第四点那五份装不下"
    );
    assert!(!is_guaranteed);
    assert_eq!(
        units
            .iter()
            .map(|unit| queue.peak_bytes_of(*unit))
            .sum::<u64>(),
        23_708_303_360,
        "前三点各两份池加四份 5/4"
    );
    assert_eq!(
        queue.headroom_bytes() + 24_675_090_432,
        30_064_771_072,
        "预算 = 28 GiB − 两份最大的池"
    );
    assert_eq!(queue.units_to_start(4, true).0.len(), 4, "线程少时按线程");
    let settings_under_eight = preparation_settings_with(31, 8 << 30);
    let queue_under_eight = PreparationQueue::new(
        "wall_clock",
        &points,
        Some(directory),
        &settings_under_eight,
        &run_the_preparation_work,
    );
    assert_eq!(queue_under_eight.units_to_start(31, true).0.len(), 5);
    assert_eq!(
        preparation_configuration_fields(&points, Some(directory), &settings),
        "preparation_threads=31 preparation_memory_cap_bytes=30064771072 preparation_memory_cap_source=environment_variable preparation_units=24 preparation_units_started_at_once_by_the_estimates=15 preparation_plan_by_the_estimates=in_batches"
    );
    assert_eq!(
        preparation_configuration_fields(&points, None, &preparation_settings_with(31, 64 << 30)),
        "preparation_threads=31 preparation_memory_cap_bytes=68719476736 preparation_memory_cap_source=environment_variable preparation_units=20 preparation_units_started_at_once_by_the_estimates=20 preparation_plan_by_the_estimates=all_at_once"
    );
}

#[test]
fn real_disk_images_of_a_point_wait_for_its_pool_and_for_every_point_before_it_to_be_measured() {
    let points = [small_point(96), small_point(128)];
    let settings = preparation_settings_with(31, 28 << 30);
    let worker = |work: PreparationWork| fake_preparation_outcome(work, None, 1000);
    let directory = Path::new("e185-image-order-directory-never-created");
    let mut queue =
        PreparationQueue::new("wall_clock", &points, Some(directory), &settings, &worker);
    let images_of_the_larger = PreparationUnit::BuildTheRealDiskImages { point_index: 1 };
    assert!(!queue.is_ready(images_of_the_larger), "那一点还没造好");
    for point_index in 0..2 {
        let unit = PreparationUnit::PrepareThePoint { point_index };
        let work = queue.start(unit);
        queue.finish(unit, worker(work));
    }
    assert!(queue.is_ready(PreparationUnit::BuildTheRealDiskImages { point_index: 0 }));
    assert!(
        !queue.is_ready(images_of_the_larger),
        "前一点还没量完，盘上不建第二份镜像"
    );
    queue.finish_the_point(0);
    assert!(queue.is_ready(images_of_the_larger));
    assert_eq!(queue.accounted_bytes(), 1000, "量完的那一点不再记账");
}

#[test]
fn later_points_start_only_when_all_of_their_units_fit() {
    let point = small_point(96);
    let pool_bytes = estimated_pool_footprint_bytes_before_construction(point);
    let worker = |work: PreparationWork| fake_preparation_outcome(work, None, pool_bytes);
    // 余量 2 份池；一点整点是 2 份（造池与几何核）+ 4 × 5/4 份（重造）= 7 份。上限 12 份时第二点只装得下造池那一份，整点不派
    for (cap_in_pools, expected_units) in [(12, 5), (16, 10)] {
        let settings = preparation_settings_with(31, cap_in_pools * pool_bytes);
        let queue = PreparationQueue::new("stage_split", &[point, point], None, &settings, &worker);
        let (units, is_guaranteed) = queue.units_to_start(31, true);
        assert_eq!(units.len(), expected_units, "上限 {cap_in_pools} 份池");
        assert!(!is_guaranteed);
    }
}

#[test]
fn process_status_gives_the_thread_count_and_only_one_thread_counts_as_alone() {
    let status = "Name:\te185\nThreads:\t1\nSigQ:\t0/1\n";
    assert_eq!(threads_in_the_process_status(status), Some(1));
    assert!(is_only_the_measuring_thread_left(status));
    assert!(!is_only_the_measuring_thread_left(
        "Name:\te185\nThreads:\t3\n"
    ));
    assert!(!is_only_the_measuring_thread_left("Name:\te185\n"));
    let own_status = std::fs::read_to_string("/proc/self/status").expect("读得到本进程的 status");
    assert!(
        threads_in_the_process_status(&own_status).is_some_and(|threads| threads >= 1),
        "本进程的 status 里读得出线程数"
    );
}

#[test]
fn global_queue_runs_every_unit_of_both_points_at_once_on_its_own_thread() {
    let points = [small_point(96), small_point(128)];
    let units_in_the_queue = 10_u64;
    let arrived = AtomicU64::new(0);
    let running = AtomicU64::new(0);
    let most_running = AtomicU64::new(0);
    let worker = |work: PreparationWork| {
        let now_running = running.fetch_add(1, Ordering::SeqCst) + 1;
        most_running.fetch_max(now_running, Ordering::SeqCst);
        arrived.fetch_add(1, Ordering::SeqCst);
        let waited = Instant::now();
        // 等十份都起来（至多 10 秒）：同时在跑的份数才看得出来
        while arrived.load(Ordering::SeqCst) < units_in_the_queue
            && waited.elapsed() < Duration::from_secs(10)
        {
            std::thread::sleep(Duration::from_millis(1));
        }
        running.fetch_sub(1, Ordering::SeqCst);
        fake_preparation_outcome(work, None, 1000)
    };
    let mut emitter = Emitter::new();
    let settings = preparation_settings_with(31, 28 << 30);
    let mut queue = PreparationQueue::new("stage_split", &points, None, &settings, &worker);
    let _prepared = queue.prepared_point(0, &mut emitter);
    for point_index in 0..2 {
        for round in 2..=ROUNDS {
            let _rebuilt = queue.take_the_rebuilt_pool(point_index, round, &mut emitter);
        }
    }
    assert_eq!(queue.phases.len(), 1, "一次准备就把两点十份全做完");
    let phase = &queue.phases[0];
    assert_eq!(
        (
            phase.started_prepare_units,
            phase.started_rebuild_units,
            phase.started_real_disk_image_units
        ),
        (2, 8, 0)
    );
    assert_eq!(
        phase.most_threads_at_once, 10,
        "十份同时派出，不是每点一组的至多五份"
    );
    assert_eq!(phase.distinct_threads, 10, "每份各一条线程");
    assert_eq!(most_running.load(Ordering::SeqCst), 10, "十份真同时在跑");
    assert_eq!(running.load(Ordering::SeqCst), 0, "交回时准备线程都已停");
}

#[test]
fn speculative_rebuilds_of_a_point_whose_geometry_changed_the_workload_are_voided_and_rebuilt_with_the_new_recipe(
) {
    let changed = small_point(96);
    let unchanged = small_point(128);
    let points = [changed, unchanged];
    let worker = |work: PreparationWork| fake_preparation_outcome(work, Some(changed), 1000);
    let settings = preparation_settings_with(31, 28 << 30);
    let mut emitter = Emitter::new();
    let mut queue = PreparationQueue::new("stage_split", &points, None, &settings, &worker);
    let prepared_changed = queue.prepared_point(0, &mut emitter);
    assert_eq!(
        prepared_changed.recipe.extra_overwritten_inodes,
        BTreeSet::from([7])
    );
    let phase_that_prepared_the_changed_point = queue.phases[0].clone();
    assert_eq!(
        phase_that_prepared_the_changed_point.started_speculative_rebuild_units, 8,
        "两点的重造都在几何核定之前推测着派出"
    );
    assert_eq!(
        phase_that_prepared_the_changed_point.speculative_rebuilds_voided, 4,
        "改了负载的那一点四份作废"
    );
    assert_eq!(queue.points[0].speculative_rebuilds_voided, 4);
    assert_eq!(queue.points[1].speculative_rebuilds_voided, 0);
    for round in 2..=ROUNDS {
        let rebuilt = queue.take_the_rebuilt_pool(0, round, &mut emitter);
        assert_eq!(
            rebuilt.recipe, prepared_changed.recipe,
            "第 {round} 轮按新配方重造"
        );
    }
    assert_eq!(queue.phases.len(), 2, "作废的四份在下一次准备里重造");
    assert_eq!(
        (
            queue.phases[1].started_rebuild_units,
            queue.phases[1].started_speculative_rebuild_units
        ),
        (4, 0)
    );
    queue.finish_the_point(0);
    let prepared_unchanged = queue.prepared_point(1, &mut emitter);
    assert_eq!(prepared_unchanged.recipe, first_recipe_of(unchanged));
    for round in 2..=ROUNDS {
        let rebuilt = queue.take_the_rebuilt_pool(1, round, &mut emitter);
        assert_eq!(rebuilt.recipe, prepared_unchanged.recipe);
    }
    assert_eq!(queue.phases.len(), 2, "没改负载的那一点推测造的照用");
}

#[test]
fn global_queue_keeps_the_pools_in_hand_under_the_memory_cap_and_measures_in_batches() {
    let point = small_point(96);
    let points = [point, point];
    let pool_bytes = estimated_pool_footprint_bytes_before_construction(point);
    assert_eq!(pool_bytes, 41_943_040);
    // 在手的池按四分之一份计：造池与几何核在跑 8，重造在跑 5，造好没取走的 4
    let quarters_in_hand = AtomicU64::new(0);
    let most_quarters_in_hand = AtomicU64::new(0);
    let worker = |work: PreparationWork| {
        let peak_quarters = match &work {
            PreparationWork::PrepareThePoint(_) => 8,
            PreparationWork::Rebuild(_) => 5,
            PreparationWork::BuildTheRealDiskImages { .. } => 4,
        };
        let now_in_hand =
            quarters_in_hand.fetch_add(peak_quarters, Ordering::SeqCst) + peak_quarters;
        most_quarters_in_hand.fetch_max(now_in_hand, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(5));
        let outcome = fake_preparation_outcome(work, None, pool_bytes);
        quarters_in_hand.fetch_sub(peak_quarters, Ordering::SeqCst);
        quarters_in_hand.fetch_add(4, Ordering::SeqCst);
        outcome
    };
    // 预算 = 6 份 − 余量 2 份 = 4 份 = 16 个四分之一
    let settings = preparation_settings_with(31, 6 * pool_bytes);
    let mut emitter = Emitter::new();
    let mut queue = PreparationQueue::new("stage_split", &points, None, &settings, &worker);
    for point_index in 0..2 {
        let _prepared = queue.prepared_point(point_index, &mut emitter);
        for round in 2..=ROUNDS {
            let _rebuilt = queue.take_the_rebuilt_pool(point_index, round, &mut emitter);
            quarters_in_hand.fetch_sub(4, Ordering::SeqCst);
        }
        queue.finish_the_point(point_index);
        quarters_in_hand.fetch_sub(4, Ordering::SeqCst);
    }
    assert!(
        most_quarters_in_hand.load(Ordering::SeqCst) <= 16,
        "同时在手的不超过预算：{} 个四分之一",
        most_quarters_in_hand.load(Ordering::SeqCst)
    );
    assert_eq!(quarters_in_hand.load(Ordering::SeqCst), 0);
    let started_per_phase: Vec<(usize, usize)> = queue
        .phases
        .iter()
        .map(|phase| (phase.started_prepare_units, phase.started_rebuild_units))
        .collect();
    assert_eq!(
        started_per_phase,
        vec![(1, 1), (0, 2), (0, 1), (1, 1), (0, 2), (0, 1)],
        "装不下就分批：批内并行、批间串行量；要的那一份好了就不再派新的"
    );
    assert_eq!(
        queue.phases[0].largest_accounted_bytes,
        2 * pool_bytes + pool_bytes * 5 / 4
    );
    assert!(queue.phases.iter().all(
        |phase| phase.largest_accounted_bytes <= 4 * pool_bytes && phase.guaranteed_units == 0
    ));
}

#[test]
fn queue_with_no_memory_to_spare_still_prepares_one_unit_at_a_time() {
    let points = [small_point(96)];
    let worker = |work: PreparationWork| fake_preparation_outcome(work, None, 1000);
    let settings = preparation_settings_with(31, 1);
    let mut emitter = Emitter::new();
    let mut queue = PreparationQueue::new("stage_split", &points, None, &settings, &worker);
    let _prepared = queue.prepared_point(0, &mut emitter);
    for round in 2..=ROUNDS {
        let _rebuilt = queue.take_the_rebuilt_pool(0, round, &mut emitter);
    }
    assert_eq!(queue.phases.len(), 5);
    assert!(queue
        .phases
        .iter()
        .all(|phase| phase.guaranteed_units == 1 && phase.most_threads_at_once == 1));
}

#[test]
fn phase_starts_no_new_unit_once_the_wanted_one_is_ready() {
    let point = small_point(96);
    let pool_bytes = estimated_pool_footprint_bytes_before_construction(point);
    let started = AtomicU64::new(0);
    // 重造等到第三份开派（要的那一份好了之后还在派新的）或 300 毫秒：造池与几何核（要的那一份）早已做完、交回
    let worker = |work: PreparationWork| {
        started.fetch_add(1, Ordering::SeqCst);
        if matches!(work, PreparationWork::Rebuild(_)) {
            let waited = Instant::now();
            while started.load(Ordering::SeqCst) < 3
                && waited.elapsed() < Duration::from_millis(300)
            {
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        fake_preparation_outcome(work, None, pool_bytes)
    };
    // 预算 4 份池：开头派造池与几何核（2 份）与第 2 轮重造（5/4 份）；造好之后账上只剩 1 份池加在跑的重造，第 3 轮重造装得下
    let settings = preparation_settings_with(31, 6 * pool_bytes);
    let mut emitter = Emitter::new();
    let mut queue = PreparationQueue::new("stage_split", &[point], None, &settings, &worker);
    let _prepared = queue.prepared_point(0, &mut emitter);
    assert_eq!(
        (
            queue.phases[0].started_prepare_units,
            queue.phases[0].started_rebuild_units
        ),
        (1, 1),
        "要的那一份好了就不再派新的，等在跑的交回"
    );
    assert_eq!(started.load(Ordering::SeqCst), 2);
}

/// 准备行里去掉造池计时那一段（计时每次不同，其余逐字比）。
fn lines_without_the_construction_seconds<'prepared>(
    prepared_points: impl IntoIterator<Item = &'prepared PreparedPoint>,
) -> Vec<String> {
    prepared_points
        .into_iter()
        .flat_map(|prepared| prepared.lines.iter())
        .map(|line| {
            line.split(' ')
                .filter(|field| !field.starts_with("construction_seconds="))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

#[test]
fn global_queue_of_small_pools_gives_the_serial_digests_and_lines_in_the_point_order() {
    let points = [small_point(128), small_point(96)];
    let serial: Vec<PreparedPoint> = points
        .iter()
        .map(|point| prepare_the_sampling_point(*point))
        .collect();
    let settings = preparation_settings_with(31, 28 << 30);
    let mut emitter = Emitter::new();
    let mut queue = PreparationQueue::new(
        "stage_split",
        &points,
        None,
        &settings,
        &run_the_preparation_work,
    );
    let mut queued: Vec<Arc<PreparedPoint>> = Vec::new();
    for (point_index, serial_point) in serial.iter().enumerate() {
        let prepared = queue.prepared_point(point_index, &mut emitter);
        assert_eq!(prepared.recipe, serial_point.recipe);
        assert_eq!(
            digests_of_the_pool(&prepared.pool),
            digests_of_the_pool(&serial_point.pool),
            "队列里造的池与串行造的逐块相同（{}）",
            serial_point.recipe.point.label()
        );
        assert_eq!(prepared.reference_digests, serial_point.reference_digests);
        for round in 2..=ROUNDS {
            let rebuilt = queue.take_the_rebuilt_pool(point_index, round, &mut emitter);
            assert_eq!(rebuilt.recipe, serial_point.recipe);
            assert_eq!(
                rebuilt.digests, serial_point.reference_digests,
                "第 {round} 轮的重造与串行造的相同"
            );
        }
        queue.finish_the_point(point_index);
        queued.push(prepared);
    }
    let queued_lines = lines_without_the_construction_seconds(queued.iter().map(Arc::as_ref));
    assert_eq!(
        queued_lines,
        lines_without_the_construction_seconds(&serial)
    );
    assert!(
        queued_lines.first().is_some_and(
            |line| line.starts_with("name=geometry_attempt fraction=0.75 pool_data_units=128 ")
        ),
        "头一行是先量的那一点：{queued_lines:?}"
    );
    let changed_points = serial
        .iter()
        .filter(|serial_point| serial_point.recipe != first_recipe_of(serial_point.recipe.point))
        .count();
    let phase = &queue.phases[0];
    assert_eq!(phase.most_threads_at_once, 10, "两点十份一次派出");
    assert_eq!(phase.started_speculative_rebuild_units, 8);
    assert_eq!(phase.speculative_rebuilds_voided, changed_points * 4);
}

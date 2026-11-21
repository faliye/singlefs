use super::*;

// ---- 丙段代价（登记第十二节修订第 9 条）----

/// 盘 `device` 上从单元区起点偏 `offset_from_the_unit_area_start` 槽（可以是负的：落在单元区之前）起、跨 `span_slots` 槽的一条记录。
fn record_at_the_offset(
    device: u32,
    offset_from_the_unit_area_start: i64,
    span_slots: u16,
    is_released: bool,
) -> AllocationRecord {
    AllocationRecord {
        device: DeviceIdentity(device),
        slot: SlotNumber(
            LOCAL_UNIT_AREA_START_SLOT
                .checked_add_signed(offset_from_the_unit_area_start)
                .expect("槽号不为负"),
        ),
        span_slots,
        generation: singlefs_core::address::CheckpointTxg(7),
        is_released,
        quarantine_mark: singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
    }
}

/// 装置自己数的每段活槽数钉死在手算的数上，并与核心层的 CPU 写法逐段相同：单元区三个整段加 10 槽的尾巴；
/// 起在单元区之前的那条只数落进来的 2 槽，跨两段的那条两段各 4 槽，已释放的与别的盘上的不数，伸进尾巴的那条只数整段里的 2 槽。
#[test]
fn the_apparatus_count_of_live_slots_per_segment_pins_the_hand_counted_values_and_agrees_with_the_core(
) {
    let unit_area = UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: UnitAreaStart::of_the_default_journal_ring(),
        slots: 3 * LOCAL_CLUSTER_SEGMENT_SLOTS + 10,
    };
    let records = [
        record_at_the_offset(0, -4, 6, false),
        record_at_the_offset(0, 60, 8, false),
        record_at_the_offset(0, 70, 2, true),
        record_at_the_offset(1, 64, 2, false),
        record_at_the_offset(0, 130, 2, false),
        record_at_the_offset(0, 190, 6, false),
    ];
    let by_the_apparatus = live_slots_of_each_segment_counted_by_the_apparatus(&records, unit_area);
    assert_eq!(by_the_apparatus, [2_u64 + 4, 4, 2 + 2]);
    assert_eq!(
        by_the_apparatus,
        live_slots_per_cluster_segment_by_the_records(&records, unit_area)
    );
}

/// 五遍取正中那一遍，四遍取正中两遍的平均，一遍都没有是零（修订第 9 条「取中位」）。
#[test]
fn the_median_wall_of_five_repeats_is_the_middle_one_and_of_four_the_mean_of_the_middle_two() {
    let nanoseconds = |values: &[u64]| -> Vec<Duration> {
        values
            .iter()
            .map(|value| Duration::from_nanos(*value))
            .collect()
    };
    assert_eq!(
        median_wall_of(&nanoseconds(&[500, 100, 400, 200, 300])),
        Duration::from_nanos(300)
    );
    assert_eq!(
        median_wall_of(&nanoseconds(&[40, 10, 30, 20])),
        Duration::from_nanos(25)
    );
    assert_eq!(median_wall_of(&[]), Duration::ZERO);
}

fn counts_of_the_passes(
    live_slots_of_each_pass: Vec<Vec<Option<u64>>>,
) -> StageThreeCountsOfTheArm {
    StageThreeCountsOfTheArm {
        walls: [500, 100, 400, 200, 300]
            .into_iter()
            .map(Duration::from_nanos)
            .take(live_slots_of_each_pass.len())
            .collect(),
        live_slots_of_each_pass,
        the_opening_thread_was_running_at_the_first_pass: false,
    }
}

/// 小池上（同几何核那两条小池用例的起步镜像）两条臂各数五遍：每一遍每段的活槽数逐段相同、都等于装置自己数的，活槽合计不少于
/// P 个活数据单元的槽数；CPU 臂照产品 greedy 跑的那一次 GC 调用量得到挂钟、丙段那一截量不到。GPU 臂在这里是在 CPU 上算的替身
/// （真卡那一条带 tests-on-a-real-gpu 另跑）。一个场景：造池太久，同一份起步镜像上按次序做两步。
#[test]
fn the_two_arms_count_the_same_live_slots_per_segment_on_the_small_pool_as_the_apparatus_and_a_greedy_call_is_timed(
) {
    let image = small_pool_image();
    let (allocation_records, unit_area) =
        allocation_records_and_unit_area_of_the_image(&image).expect("量挂载成");
    let by_the_apparatus =
        live_slots_of_each_segment_counted_by_the_apparatus(&allocation_records, unit_area);
    assert_eq!(
        u64::try_from(by_the_apparatus.len()).expect("段数"),
        unit_area.slots / LOCAL_CLUSTER_SEGMENT_SLOTS
    );
    assert!(
        by_the_apparatus.iter().sum::<u64>()
            >= SMALL_POOL_LIVE_DATA_UNITS * (LOCAL_DATA_UNIT_BYTES / LOCAL_SLOT_BYTES),
        "{}",
        by_the_apparatus.iter().sum::<u64>()
    );
    let on_the_cpu = stage_three_counts_of_the_counter(
        &mut StageThreeCounter::<CpuBackedStandIn>::OnTheCpu,
        &allocation_records,
        unit_area,
    )
    .expect("CPU 臂不出错");
    let on_the_stand_in = stage_three_counts_of_the_counter(
        &mut StageThreeCounter::OnTheGpu(CpuBackedStandIn {
            corrupted_crc32c: false,
        }),
        &allocation_records,
        unit_area,
    )
    .expect("替身不出错");
    let expected_pass: Vec<Option<u64>> = by_the_apparatus.iter().copied().map(Some).collect();
    for counts in [&on_the_cpu, &on_the_stand_in] {
        assert_eq!(counts.walls.len(), 5, "修订第 9 条：各量 5 次");
        assert_eq!(counts.live_slots_of_each_pass.len(), 5);
        assert!(
            counts
                .live_slots_of_each_pass
                .iter()
                .all(|pass| *pass == expected_pass),
            "{counts:?}"
        );
        assert!(every_pass_agrees_with_the_apparatus_count(
            counts,
            &by_the_apparatus
        ));
    }
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let share = stage_three_share_of_one_greedy_call(
        &image,
        ComputeOfTheArm::Cpu,
        &[],
        &memory_leaked,
        None,
    )
    .expect("CPU 臂挂载得成");
    assert!(share.gc_call_wall > Duration::ZERO, "{share:?}");
    assert_eq!(share.stage_three, None);
}

/// 小池上 GPU 点名一张卡臂照产品 greedy 跑的那一次 GC 调用：丙段至少调了一次、那一截不长于整次调用。只在真卡上跑通、没有 CPU 一侧可比
/// （CPU 臂那一截从包外量不到），不入对拍结论；只录那一段（`SINGLEFS_CRASH_AMPLIFICATION_STAGE=record`）没有要录的，不造池、不取卡就返回。
#[cfg(feature = "tests-on-a-real-gpu")]
#[test]
fn a_greedy_call_on_a_real_card_runs_the_stage_three_on_the_gpu_within_the_call_on_the_small_pool()
{
    let test_name =
        "a_greedy_call_on_a_real_card_runs_the_stage_three_on_the_gpu_within_the_call_on_the_small_pool";
    if !singlefs_verification::crash_library::RunStage::of_this_process().judges() {
        println!("NO_CARD_TAKEN_IN_THE_RECORD_STAGE test={test_name}");
        return;
    }
    if !the_task_server_hands_over_cards_to_this_process() {
        println!(
            "NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={REASON_WITHOUT_CARDS_FROM_THE_TASK_SERVER}"
        );
        return;
    }
    let image = small_pool_image();
    let cards =
        cards_of_this_process().unwrap_or_else(|reason| panic!("判段要开卡，取卡没成：{reason}"));
    if cards.vulkan_instance.is_none() {
        return;
    }
    let memory_leaked = GpuMemoryLeakedInThisProcess::with_nothing_leaked();
    let share = stage_three_share_of_one_greedy_call(
        &image,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        &cards.discrete_gpus,
        &memory_leaked,
        cards.vulkan_instance.as_ref(),
    )
    .expect("GPU 点名一张卡臂挂载得成");
    let stage_three = share.stage_three.expect("GPU 臂外面套了丙段计时");
    assert!(stage_three.calls >= 1, "{share:?}");
    assert!(stage_three.wall <= share.gc_call_wall, "{share:?}");
}

/// 两种产物行的形态：修订第 9 条写的四个键在前、次序照写，装置另报的在后；CPU 臂丙段那一截写量不到；结果行读回来认得出种类与键。
#[test]
fn the_stage_three_lines_carry_the_registered_keys_first_and_in_order() {
    let agreeing = vec![Some(6), Some(4), Some(4)];
    let cost_fields = stage_three_cost_fields(
        UnitAreaTier::FourGibibytes,
        ComputeOfTheArm::Cpu,
        &counts_of_the_passes(vec![agreeing; 5]),
        &[6, 4, 4],
    );
    assert_eq!(
        cost_fields,
        "tier=4GiB arm=cpu median_nanoseconds=300 repeats=5 nanoseconds_of_each_repeat=500,100,400,200,300 \
             segments=3 live_slots=14 every_repeat_agrees_with_the_apparatus_count=true \
             gpu_opening_thread_was_running_at_the_first_repeat=false"
    );
    let share_without_the_stage_three = StageThreeShareOfOneGreedyCall {
        gc_call_wall: Duration::from_nanos(7000),
        stage_three: None,
        batches: 3,
        the_opening_thread_was_running_at_the_call: false,
    };
    assert_eq!(
        stage_three_share_fields(
            UnitAreaTier::FourGibibytes,
            ComputeOfTheArm::Cpu,
            &share_without_the_stage_three
        ),
        "tier=4GiB arm=cpu gc_call_nanoseconds=7000 stage_three_nanoseconds=not_reachable_from_outside_the_core \
             stage_three_calls=not_reachable_from_outside_the_core batches=3 gpu_opening_thread_was_running_at_the_call=false"
    );
    let share_fields = stage_three_share_fields(
        UnitAreaTier::FourGibibytes,
        ComputeOfTheArm::GpuOnTheCardWithTheLowestPciAddress,
        &StageThreeShareOfOneGreedyCall {
            gc_call_wall: Duration::from_nanos(7000),
            stage_three: Some(StageThreeTiming {
                calls: 2,
                wall: Duration::from_nanos(900),
            }),
            batches: 3,
            the_opening_thread_was_running_at_the_call: false,
        },
    );
    assert_eq!(
        share_fields,
        "tier=4GiB arm=gpu_one_card gc_call_nanoseconds=7000 stage_three_nanoseconds=900 stage_three_calls=2 \
             batches=3 gpu_opening_thread_was_running_at_the_call=false"
    );
    let (kind, fields) = fields_of_a_result_line(&format!(
        "{RESULT_LINE_PREFIX} name=stage_three_share {share_fields}"
    ))
    .expect("认得出是结果行");
    assert_eq!(kind, "stage_three_share");
    assert_eq!(
        fields.get("stage_three_nanoseconds").map(String::as_str),
        Some("900")
    );
}

/// 丙段计时的包装：每次丙段记一次、判定原样交回（里面是在 CPU 上算的替身）；别的段不记。
#[test]
fn the_stage_three_timing_wrapper_counts_every_live_slot_call_and_hands_the_decisions_through() {
    let records: Vec<AllocationRecord> = [(0, 16), (1, 32)]
        .into_iter()
        .flat_map(|(segment_index, units)| data_unit_records_in_the_segment(segment_index, units))
        .collect();
    let unit_area = UnitAreaOfOneDevice {
        device: DeviceIdentity(0),
        start: UnitAreaStart::of_the_default_journal_ring(),
        slots: 4 * LOCAL_CLUSTER_SEGMENT_SLOTS,
    };
    let segments = cluster_segment_entry_keys_of_the_unit_area(unit_area);
    let timing: SharedStageThreeTiming = Arc::new(Mutex::new(StageThreeTiming::default()));
    let mut wrapper = GpuComputeTimingTheStageThree {
        inner: CpuBackedStandIn {
            corrupted_crc32c: false,
        },
        timing: Arc::clone(&timing),
    };
    let contents: [Vec<u8>; 1] = [vec![1_u8; 32768]];
    let _checksums = wrapper
        .verify_unit_checksums(&units_to_verify(&contents))
        .expect("替身不出错");
    for _call in 0..2 {
        let decisions = wrapper
            .count_live_slots_per_cluster_segment(&records, unit_area, &segments)
            .expect("替身不出错");
        assert_eq!(
            decisions,
            [32, 64, 0, 0].map(GpuEntryDecision::ConfirmedByBothKernels)
        );
    }
    assert_eq!(timing.lock().expect("没在别处 panic 过").calls, 2);
}

/// 每一遍逐段相同才算对得上：有一遍差一段、有一段这一轮不确定、段数少一段，都对不上。
#[test]
fn one_pass_differing_from_the_apparatus_in_one_segment_or_left_uncertain_does_not_agree() {
    let by_the_apparatus = [6_u64, 4, 4];
    let agreeing = vec![Some(6), Some(4), Some(4)];
    assert!(every_pass_agrees_with_the_apparatus_count(
        &counts_of_the_passes(vec![agreeing.clone(); 5]),
        &by_the_apparatus
    ));
    for odd_pass in [
        vec![Some(6), Some(5), Some(4)],
        vec![Some(6), None, Some(4)],
        vec![Some(6), Some(4)],
    ] {
        let mut passes = vec![agreeing.clone(); 4];
        passes.push(odd_pass.clone());
        assert!(
            !every_pass_agrees_with_the_apparatus_count(
                &counts_of_the_passes(passes),
                &by_the_apparatus
            ),
            "{odd_pass:?}"
        );
    }
}

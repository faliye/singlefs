// ============================== 单测（不起卡；变异表 scripts/mutation/tables/e174_lease_drop_wait_memory_return.tsv 的变异由它们抓） ==============================

use super::*;

fn first_lease_of(job: &ChildJob) -> LeaseRecord {
    let outcome = simulate_job(job).0;
    assert!(outcome.child_void.is_empty(), "模型卡上子进程不该作废：{:?}", outcome.child_void);
    outcome.leases.into_iter().next().expect("至少一笔借据")
}

fn lease_with_samples(pre_mebibytes: u64, pre_spread_mebibytes: u64, used: &[Option<u64>]) -> LeaseRecord {
    let job = simulated_job(Group::MainGrid, 64, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest, 1);
    let mut record = LeaseRecord::new(job, 1);
    record.pre_mebibytes = pre_mebibytes;
    record.pre_spread_mebibytes = pre_spread_mebibytes;
    record.samples = used
        .iter()
        .enumerate()
        .map(|(index, used_mebibytes)| {
            let started = u64::try_from(index).expect("下标装得进 u64") * 1_000;
            WindowSample { started_microseconds: started, finished_microseconds: started + 500, used_mebibytes: *used_mebibytes }
        })
        .collect();
    record
}

#[test]
fn every_self_check_passes() {
    let failed: Vec<String> = self_check_results().into_iter().filter(|(_, passed)| !*passed).map(|(name, _)| name).collect();
    assert!(failed.is_empty(), "红的自检：{failed:?}");
    assert!(self_check_results().len() >= 30, "自检条数不该变少");
}

#[test]
fn report_parser_reads_the_process_line_and_not_the_bar_one_line() {
    let report = SimulatedCardHost::new(Card::Rtx5090, 50_000).report_text();
    let reading = parse_card_reading(&report, "01:00.0", SIMULATED_PROCESS_IDENTIFIER).expect("读得出");
    assert_eq!(reading.total_mebibytes, Some(32607), "Total 是 FB 那一小节的，不是 BAR1 的 32768");
    assert_eq!(reading.process_used_mebibytes, Some(SIMULATED_RUNTIME_MEBIBYTES), "进程占着的是 Processes 下的，不是 BAR1 的 Used 1 MiB");
    assert_eq!(reading.listed_process_identifiers, vec![SIMULATED_PROCESS_IDENTIFIER]);
    assert!(parse_card_reading(&report, "10:00.0", SIMULATED_PROCESS_IDENTIFIER).is_err(), "报告里没有的卡是读不出，不是 0");
}

#[test]
fn report_parser_adds_up_every_line_of_this_process_and_skips_others() {
    let report = "GPU 00000000:01:00.0\n    FB Memory Usage\n        Total : 32607 MiB\n    Processes\n        Process ID : 10\n            Used GPU Memory : 100 MiB\n        Process ID : 11\n            Used GPU Memory : 7 MiB\n        Process ID : 10\n            Used GPU Memory : 23 MiB\n";
    let reading = parse_card_reading(report, "01:00.0", 10).expect("读得出");
    assert_eq!(reading.process_used_mebibytes, Some(123));
    assert_eq!(reading.listed_process_identifiers, vec![10, 11, 10]);
    let unreadable = report.replace("23 MiB", "N/A");
    assert!(parse_card_reading(&unreadable, "01:00.0", 10).expect("读得出").process_used_is_unreadable);
    let not_listed = parse_card_reading(report, "01:00.0", 99).expect("读得出");
    assert_eq!(not_listed.process_used_mebibytes, None, "没列这个进程是读不到，不当 0");
}

#[test]
fn compute_applications_parser_sums_rows_of_this_card_and_process() {
    let text = "00000000:01:00.0, 10, 100\n00000000:01:00.0, 11, 7\n00000000:10:00.0, 10, 5\n00000000:01:00.0, 10, 20\n";
    assert_eq!(parse_compute_applications(text, "01:00.0", 10), Ok(Some(120)));
    assert_eq!(parse_compute_applications(text, "01:00.0", 99), Ok(None));
    assert!(parse_compute_applications("00000000:01:00.0, 10\n", "01:00.0", 10).is_err());
}

#[test]
fn piece_plan_covers_every_registered_size() {
    let sizes_and_counts = [(1_u64, 1_u64), (64, 1), (512, 1), (2048, 2), (8192, 8)];
    for (size_mebibytes, expected_count) in sizes_and_counts {
        let job = simulated_job(Group::MainGrid, size_mebibytes, Liveness::Done, Method::OneBuild, Arm::WaitLatest, 1);
        let lease = first_lease_of(&job);
        assert_eq!(lease.piece_count, expected_count, "S={size_mebibytes} MiB 的张数");
        assert_eq!(lease.piece_count * lease.piece_bytes, size_mebibytes * MEBIBYTE, "张数 × 每张 == S（加法形）");
        assert!(!lease.is_void(), "S={size_mebibytes} MiB：{:?}", lease.void_reasons);
    }
}

#[test]
fn allocator_peak_grows_by_at_least_the_whole_planned_size() {
    let job = simulated_job(Group::MainGrid, 512, Liveness::Done, Method::OneBuild, Arm::WaitLatest, 1);
    let lease = first_lease_of(&job);
    let before = lease.allocator_pre.expect("模型卡给分配器两项");
    let at_peak = lease.allocator_peak.expect("模型卡给分配器两项");
    assert!(at_peak.allocated_bytes >= before.allocated_bytes + 512 * MEBIBYTE, "登记 B3");
    assert!(!lease.void_reasons.contains(&VoidReason::AllocatorBelowPlan));
    assert!(lease.peak_mebibytes.expect("读到了") + lease.tau_mebibytes() >= lease.pre_mebibytes + 512, "登记 B4 后一半");
}

#[test]
fn heavy_wait_arm_waits_for_the_rest_of_the_work_and_the_outer_timer_covers_it() {
    let job = simulated_job(Group::MainGrid, 64, Liveness::InFlightHeavy, Method::OneBuild, Arm::WaitLatest, 1);
    let lease = first_lease_of(&job);
    assert!(lease.void_reasons.is_empty(), "K7、K8、K9 都该过：{:?}", lease.void_reasons);
    let work = lease.work_microseconds;
    assert_eq!(work, 400_000);
    assert!(lease.wait_microseconds + lease.submit_to_poll_microseconds >= work * 4 / 5, "K7：T_wait + e ≥ 0.8 W（0.8 写成 4/5，不引常量）");
    assert!(lease.wait_microseconds < work * 4 / 5, "e 含两次读数，所以 T_wait 本身小于 0.8 W：K7 的左边必须带上 e");
    assert!(lease.wait_microseconds >= 200_000, "等待臂在在飞·重上真的等了：{}", lease.wait_microseconds);
    assert!(lease.outer_microseconds >= lease.wait_microseconds, "K8：外层包住 T_wait");
    assert!(lease.submit_to_poll_microseconds * 100 < work * COVERED_WORK_PERCENT, "K9：poll 开始时活还没做完");
}

#[test]
fn lease_that_is_already_void_does_not_sit_through_the_window() {
    let light_job = simulated_job(Group::MainGrid, 64, Liveness::InFlightLight, Method::OneBuild, Arm::WaitLatest, 1);
    let outcome = simulate_job(&light_job).0;
    assert_eq!(outcome.leases.len(), 1, "第 1 笔作废，这一轮不再起第 2、3 笔");
    assert!(outcome.leases[0].void_reasons.contains(&VoidReason::WorkNotInFlightAtPoll));
    assert!(outcome.leases[0].samples.is_empty(), "作废的借据不读窗口");
    let healthy_job = simulated_job(Group::MainGrid, 64, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest, 1);
    assert!(!first_lease_of(&healthy_job).samples.is_empty(), "没作废的借据读窗口");
}

#[test]
fn in_flight_work_is_not_waited_for_before_the_lease_drop() {
    for liveness in [Liveness::InFlightMedium, Liveness::InFlightHeavy] {
        let job = simulated_job(Group::MainGrid, 64, liveness, Method::OneBuild, Arm::Poll, 1);
        let lease = first_lease_of(&job);
        assert!(!lease.void_reasons.contains(&VoidReason::WorkNotInFlightAtPoll), "{liveness:?} 的活在 poll 之前就被等完了");
        assert!(lease.wait_microseconds <= POLL_NO_WAIT_LIMIT_MICROSECONDS, "不等待臂不等：{}", lease.wait_microseconds);
    }
}

#[test]
fn hold_control_still_holds_the_buffer_at_the_first_reading_after_the_drop() {
    for arm in [Arm::WaitLatest, Arm::Poll] {
        let job = simulated_job(Group::HoldControl, 2048, Liveness::InFlightMedium, Method::OneBuild, arm, 1);
        let lease = first_lease_of(&job);
        assert!(lease.void_reasons.is_empty(), "{arm:?}：{:?}", lease.void_reasons);
        let next = lease.next_remainder_mebibytes().expect("第一份读到了");
        assert!(next + signed(lease.tau_mebibytes()) >= 2048, "B4 前一半：R_next={next}");
    }
}

#[test]
fn time_to_clear_is_the_start_of_the_first_reading_after_which_every_reading_is_clean() {
    let settings = JudgeSettings { tau_override_mebibytes: None };
    let clean_from_the_start = lease_with_samples(100, 1, &[Some(100), Some(101), Some(103)]);
    assert_eq!(clean_from_the_start.time_to_clear(settings), TimeToClear::Next, "τ=3，103−100 不大于 τ");
    let clean_after_two = lease_with_samples(100, 1, &[Some(300), Some(200), Some(101), Some(100), Some(100)]);
    assert_eq!(clean_after_two.time_to_clear(settings), TimeToClear::After(2_000));
    let dirty_at_the_end = lease_with_samples(100, 1, &[Some(100), Some(100), Some(250)]);
    assert_eq!(dirty_at_the_end.time_to_clear(settings), TimeToClear::BeyondWindow);
    let unreadable_at_the_end = lease_with_samples(100, 1, &[Some(100), Some(100), None]);
    assert_eq!(unreadable_at_the_end.time_to_clear(settings), TimeToClear::BeyondWindow, "读不到不当 0");
    let dirty_in_the_middle = lease_with_samples(100, 1, &[Some(100), Some(300), Some(100), Some(100)]);
    assert_eq!(dirty_in_the_middle.time_to_clear(settings), TimeToClear::After(2_000), "中间含过一次，从之后算");
    assert_eq!(
        dirty_in_the_middle.positive_reading_counts(settings),
        (1, 4),
        "为正的份数与读得到的总份数并列"
    );
    assert_eq!(dirty_in_the_middle.peak_remainder_mebibytes(), Some(200));
    let moved = JudgeSettings { tau_override_mebibytes: Some(500) };
    assert_eq!(dirty_in_the_middle.time_to_clear(moved), TimeToClear::Next, "判别力自证：τ 挪到读数之上，判定翻面");
}

#[test]
fn presence_needs_all_rounds_on_one_side_of_their_own_tau() {
    assert_eq!(classify_presence(&[(2, 2), (0, 2), (-5, 2), (1, 3), (3, 3)]), Presence::Absent, "等于 τ 算不含");
    assert_eq!(classify_presence(&[(3, 2), (300, 2), (4, 3), (9, 3), (5, 3)]), Presence::Present);
    assert_eq!(classify_presence(&[(3, 2), (300, 2), (4, 3), (9, 3), (2, 3)]), Presence::Unstable);
    assert_eq!(classify_presence(&[]), Presence::Unstable, "没有轮不能说不含");
}

#[test]
fn wait_cost_tokens_follow_the_registered_limits() {
    let done_wait = CellKey {
        group: Group::MainGrid,
        card: Card::Rtx5090,
        size_mebibytes: 64,
        liveness: Liveness::Done,
        method: Method::OneBuild,
        arm: Arm::WaitLatest,
    };
    assert_eq!(wait_cost_token(done_wait, &[5_000, 0, 10, 3, 4_999], false), "zero_overhead");
    assert_eq!(wait_cost_token(done_wait, &[5_001, 9_000, 6_000, 7_000, 8_000], false), "fixed_overhead");
    assert_eq!(wait_cost_token(done_wait, &[5_001, 100, 6_000, 7_000, 8_000], false), "unstable");
    assert_eq!(wait_cost_token(done_wait, &[1, 1, 1, 1, 1], true), "wait_never_returned");
    let in_flight_wait = CellKey { liveness: Liveness::InFlightHeavy, ..done_wait };
    assert_eq!(wait_cost_token(in_flight_wait, &[20_000, -50, 0, 0, 0], false), "only_the_work_itself");
    assert_eq!(wait_cost_token(in_flight_wait, &[20_001, 30_000, 25_000, 90_000, 21_000], false), "longer_than_the_work");
    let poll = CellKey { arm: Arm::Poll, ..done_wait };
    assert_eq!(wait_cost_token(poll, &[1_000, 3, 0, 1, 2], false), "does_not_wait");
    assert_eq!(wait_cost_token(poll, &[1_001, 3, 0, 1, 2], false), "reports_numbers");
}

#[test]
fn round_is_complete_when_the_child_is_clean_and_the_needed_leases_are_valid() {
    let job = simulated_job(Group::MainGrid, 64, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest, 1);
    let valid = |lease_index: u32| LeaseRecord::new(job, lease_index);
    let void = |lease_index: u32| {
        let mut record = LeaseRecord::new(job, lease_index);
        record.void_reasons.push(VoidReason::ReadFailure);
        record
    };
    let round = |leases: Vec<LeaseRecord>, child_void: Vec<VoidReason>| RoundData {
        done: Some(ChildDoneRecord { child_void }),
        failure_detail: None,
        leases,
    };
    assert!(round(vec![valid(1), valid(2), valid(3)], vec![]).is_complete(3));
    assert!(round(vec![valid(1), void(2), valid(3)], vec![]).is_complete(3), "第 2、3 笔有一笔有效就够");
    assert!(!round(vec![valid(1), void(2), void(3)], vec![]).is_complete(3));
    assert!(!round(vec![void(1), valid(2), valid(3)], vec![]).is_complete(3), "第 1 笔作废，这一轮不进");
    assert!(!round(vec![valid(1), valid(2), valid(3)], vec![VoidReason::ForeignProcessListed]).is_complete(3));
    assert!(round(vec![valid(1)], vec![]).is_complete(1), "长窗只有一笔");
    let failed = RoundData { done: None, failure_detail: Some("exit".to_string()), leases: vec![] };
    assert!(!failed.is_complete(3));
}

#[test]
fn job_arguments_round_trip_and_unknown_flags_are_refused() {
    for cell in first_segment_cells().into_iter().chain(second_segment_cells()) {
        let job = ChildJob { cell, round: 4 };
        let arguments = job.child_arguments();
        assert_eq!(arguments[0], "child");
        assert_eq!(ChildJob::from_child_arguments(&arguments[1..]), Ok(job));
    }
    assert!(ChildJob::from_child_arguments(&["--bogus".to_string(), "1".to_string()]).is_err());
}

#[test]
fn shuffle_rank_is_seeded_and_does_not_depend_on_what_was_already_run() {
    assert_eq!(
        shuffle_rank("main 64 medium one_build round=1"),
        2_015_186_589_042_433_523,
        "种子 174 下这个键的位置钉死（独立算出）：种子或混合式变了这里先红"
    );
    assert_eq!(shuffle_rank("main 64 medium one_build round=1"), shuffle_rank("main 64 medium one_build round=1"));
    assert_ne!(shuffle_rank("main 64 medium one_build round=1"), shuffle_rank("main 64 medium one_build round=2"));
    let first = first_segment_cells();
    let full = pending_jobs(&first, &BTreeMap::new(), 2);
    let positions: BTreeMap<ChildJob, usize> = full.iter().enumerate().map(|(index, job)| (*job, index)).collect();
    let first_cell_rounds_one_to_three: Vec<ChildJob> =
        (1..=3).map(|round| ChildJob { cell: first[0], round }).collect();
    let done_so_far: String = first_cell_rounds_one_to_three.iter().map(|job| simulate_job(job).1).collect();
    let resumed = pending_jobs(&first, &parse_product(&done_so_far).expect("读得回来"), 2);
    assert_eq!(resumed.len(), full.len() - 3, "续跑只补没做完的");
    let resumed_positions: Vec<usize> = resumed.iter().map(|job| positions[job]).collect();
    assert!(resumed_positions.windows(2).all(|pair| pair[0] < pair[1]), "续跑的次序与整份清单的次序一致");
}

#[test]
fn two_arms_alternate_with_the_round_number_on_the_same_slot() {
    let cells = first_segment_cells();
    let mut leading_arm_by_round: BTreeMap<u32, Vec<Arm>> = BTreeMap::new();
    for round in 1..=4 {
        let key_of = |arm: Arm| {
            let cell = cells
                .iter()
                .find(|cell| {
                    cell.group == Group::MainGrid && cell.card == Card::Rtx5090 && cell.size_mebibytes == 64 && cell.liveness == Liveness::InFlightMedium && cell.arm == arm
                })
                .expect("格在清单里");
            job_order_key(&ChildJob { cell: *cell, round }, 2)
        };
        let leading = if key_of(Arm::WaitLatest) < key_of(Arm::Poll) { Arm::WaitLatest } else { Arm::Poll };
        leading_arm_by_round.entry(round).or_default().push(leading);
    }
    assert_ne!(leading_arm_by_round[&1], leading_arm_by_round[&2]);
    assert_eq!(leading_arm_by_round[&1], leading_arm_by_round[&3]);
}

#[test]
fn only_a_hold_control_child_that_lost_the_buffer_stops_the_whole_run() {
    let hold_job = simulated_job(Group::HoldControl, 2048, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest, 1);
    let main_job = ChildJob { cell: CellKey { group: Group::MainGrid, ..hold_job.cell }, round: 1 };
    let lost = format!("E7RESULT name=lease void={}", VoidReason::HoldControlDidNotHold.label());
    assert!(child_output_shows_hold_control_failure(&hold_job, &lost));
    assert!(!child_output_shows_hold_control_failure(&main_job, &lost), "别的组的子进程不触发 F1");
    assert!(!child_output_shows_hold_control_failure(&hold_job, "E7RESULT name=lease void=none"));
}

#[test]
fn tau_is_the_pre_reading_spread_plus_the_rounding_allowance() {
    let lease = lease_with_samples(100, 5, &[Some(100)]);
    assert_eq!(lease.tau_mebibytes(), 5 + 2);
    let mut device_lease = lease;
    device_lease.device_spread_mebibytes = 9;
    assert_eq!(device_lease.tau_device_mebibytes(), 9 + 2);
}

// ---------------------------------------------------------------------------------------------
// 续跑的轮库（登记修订 2 之六单测表；不起卡：子进程换成进程内的模型卡 simulate_job）
// ---------------------------------------------------------------------------------------------

fn resume_cell(arm: Arm) -> CellKey {
    CellKey { group: Group::MainGrid, card: Card::Rtx5090, size_mebibytes: 64, liveness: Liveness::InFlightMedium, method: Method::OneBuild, arm }
}

/// 单测自己的产物目录：临时目录下按测试名与进程号分开（轮库不在这里，在内存那一种里）。
fn test_directory(test_name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!("e174-rounds-{test_name}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("建得出测试的产物目录");
    directory
}

/// 单测自己的轮库：给的那一份内存库（几把代码键共用一份就传同一份），组成清单照 device-code-key.py 打的写（两行 code_key，`--simulated` 的入口）。
fn test_round_library(store: &DeviceLibrary, child_code_key: &str, arguments_code_key: &str) -> RoundLibrary {
    let composition = format!(
        "{COMPOSITION_FORM_LINE}\njudged\t2026-10-15\nhost\t本机\ncode_key\te7_index_bench::run_child_on_the_model\t{child_code_key}\n\
             code_key\te7_index_bench::ChildJob::child_arguments\t{arguments_code_key}\n\
             unit\te7_index_bench::run_child_on_the_model\tfn\te7_index_bench::<bin>::run_simulated_child\tscripts/experiments/e174.rs:1-2\t{child_code_key}\n"
    );
    round_library_from_composition(store.clone(), &composition, true).expect("单测写的组成清单读得出")
}

/// 在模型卡上跑一趟 `run`（进程内，不起子进程）：两格（等最新、轮询两条臂），第一段的两臂次序。
fn run_on_the_model(library: &RoundLibrary, output_path: &std::path::Path, limit_jobs: Option<usize>) -> RoundsOutcome {
    let cells = [resume_cell(Arm::WaitLatest), resume_cell(Arm::Poll)];
    let output = output_path.to_string_lossy().into_owned();
    let plan = RoundPlan { segment: Segment::First, cells: &cells, arm_count: 2, output_path: &output, skip_light: false, simulated: true, limit_jobs };
    let mut run_child = |job: &ChildJob| -> Result<String, String> { Ok(simulate_job(job).1) };
    let device_line_of = |_card: Card| -> Result<String, String> { Ok("simulated".to_string()) };
    run_rounds(&plan, library, &device_line_of, &mut run_child).expect("模型卡上的一趟 run 跑得完")
}

fn round_files_in(library: &RoundLibrary) -> usize {
    library.store.count_entries("", ".out")
}

#[test]
fn sha256_matches_the_standard_test_vectors() {
    assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

/// 「轮键随子进程代码键变」（M29 抓）：同一格同一轮、两个子进程代码键 → 轮键不等；轮键的绝对值另由 python 的 hashlib 算出钉住。
#[test]
fn round_key_changes_with_the_child_code_key() {
    let job = ChildJob { cell: resume_cell(Arm::WaitLatest), round: 1 };
    assert_eq!(round_key_of("A", "B", &job, "simulated"), "bbbb35f09d1e63698a0f6a591e11904367a09983dcc57f52feeb104d4ae47dfa");
    assert_ne!(round_key_of("A", "B", &job, "simulated"), round_key_of("C", "B", &job, "simulated"));
}

/// 「轮键随参数代码键变」（M30 抓）：同一格同一轮、两个参数代码键 → 轮键不等。
#[test]
fn round_key_changes_with_the_arguments_code_key() {
    let job = ChildJob { cell: resume_cell(Arm::WaitLatest), round: 1 };
    assert_ne!(round_key_of("A", "B", &job, "simulated"), round_key_of("A", "C", &job, "simulated"));
}

/// 「轮键随格的每个字段与轮号变」（M31 抓）：只差组、卡、S、活、借法、臂、轮号的两轮 → 轮键两两不等。
#[test]
fn round_key_changes_with_every_cell_field_and_the_round_number() {
    let cell = |group, card, size_mebibytes, liveness, method, arm| CellKey { group, card, size_mebibytes, liveness, method, arm };
    let (main, card, live, build, wait) = (Group::MainGrid, Card::Rtx5090, Liveness::InFlightMedium, Method::OneBuild, Arm::WaitLatest);
    let jobs = [
        ChildJob { cell: cell(main, card, 64, live, build, wait), round: 1 },
        ChildJob { cell: cell(Group::PassByPassGrid, card, 64, live, build, wait), round: 1 },
        ChildJob { cell: cell(main, Card::Rtx5060Ti, 64, live, build, wait), round: 1 },
        ChildJob { cell: cell(main, card, 512, live, build, wait), round: 1 },
        ChildJob { cell: cell(main, card, 64, Liveness::Done, build, wait), round: 1 },
        ChildJob { cell: cell(main, card, 64, live, Method::PassByPass, wait), round: 1 },
        ChildJob { cell: cell(main, card, 64, live, build, Arm::Poll), round: 1 },
        ChildJob { cell: cell(main, card, 64, live, build, wait), round: 2 },
    ];
    let keys: std::collections::BTreeSet<String> = jobs.iter().map(|job| round_key_of("A", "B", job, "simulated")).collect();
    assert_eq!(keys.len(), jobs.len(), "八轮只差一个字段或轮号，轮键要两两不等");
}

/// 「轮键随驱动与卡那一行变」（M32 抓）：驱动版本串改一位 → 轮键不等。
#[test]
fn round_key_changes_with_the_driver_and_card_line() {
    let job = ChildJob { cell: resume_cell(Arm::WaitLatest), round: 1 };
    let line = "00000000:01:00.0, NVIDIA GeForce RTX 5090, 32607 MiB, 595.84";
    assert_ne!(round_key_of("A", "B", &job, line), round_key_of("A", "B", &job, &line.replace("595.84", "595.85")));
}

/// 「轮键不看产物路径」（M33 抓）：模型卡上同一段先 `--output A` 跑完、再 `--output B` 起 → 第二趟现判 0、复用全部，B 与 A 内容相同。
#[test]
fn another_output_path_reuses_every_round() {
    let library = test_round_library(&DeviceLibrary::in_memory(), "a1", "b1");
    let directory = test_directory("other-output");
    let (first_output, second_output) = (directory.join("first.out"), directory.join("second.out"));
    let first = run_on_the_model(&library, &first_output, None);
    let second = run_on_the_model(&library, &second_output, None);
    let (first_text, second_text) = (std::fs::read_to_string(&first_output), std::fs::read_to_string(&second_output));
    std::fs::remove_dir_all(&directory).expect("删得掉测试自己建的产物目录");
    assert!(first.computed > 0 && first.reused == 0, "空库上第一趟全部现起：{} / {}", first.computed, first.reused);
    assert_eq!((second.computed, second.reused), (0, first.computed), "换一个 --output，轮键不变、全部读回");
    assert_eq!(first_text.expect("A 写出来了"), second_text.expect("B 写出来了"));
}

/// 「别的键写的轮不读回」（M34 抓）：子进程代码键 A 下跑完、B 下再起（同一份库） → 全部现起，A 写的轮留着；成功句键对不上 = 轮数，那一句点名变了的单元。
#[test]
fn rounds_written_under_another_child_code_key_are_not_read_back() {
    let store = DeviceLibrary::in_memory();
    let library_under_the_original_child_code_key = test_round_library(&store, "a1", "b1");
    let library_under_the_changed_child_code_key = test_round_library(&store, "a2", "b1");
    let directory = test_directory("other-child-key");
    let first = run_on_the_model(&library_under_the_original_child_code_key, &directory.join("first.out"), None);
    let second = run_on_the_model(&library_under_the_changed_child_code_key, &directory.join("second.out"), None);
    let files = round_files_in(&library_under_the_changed_child_code_key);
    std::fs::remove_dir_all(&directory).expect("删得掉测试自己建的产物目录");
    assert_eq!((second.computed, second.reused), (first.computed, 0), "B 下不读 A 写的轮");
    assert_eq!(second.mismatched, first.computed, "键对不上 = 上一趟的轮数");
    assert_eq!(files, 2 * usize::try_from(first.computed).expect("轮数装得进 usize"), "两趟各写的轮都在库里");
    let sentence = second.changed_units.expect("键对不上时有那一句");
    assert!(sentence.contains("改了 e7_index_bench::<bin>::run_simulated_child"), "{sentence}");
}

/// 「中途停下再起只补没做完的」（续跑本身）：`--limit-jobs 2` 停下、再不带它起 → 第二趟只起其余的轮、复用 2，拼出的产物与一次跑完的逐字节相同。
#[test]
fn stopping_at_the_limit_and_starting_again_only_runs_the_remaining_rounds() {
    let stopped_library = test_round_library(&DeviceLibrary::in_memory(), "a1", "b1");
    let whole_library = test_round_library(&DeviceLibrary::in_memory(), "a1", "b1");
    let (stopped_directory, whole_directory) = (test_directory("limit-stopped"), test_directory("limit-whole"));
    let stopped_output = stopped_directory.join("product.out");
    let stopped = run_on_the_model(&stopped_library, &stopped_output, Some(2));
    let output_after_stop = stopped_output.exists();
    let resumed = run_on_the_model(&stopped_library, &stopped_output, None);
    let whole_output = whole_directory.join("product.out");
    let whole = run_on_the_model(&whole_library, &whole_output, None);
    let (resumed_text, whole_text) = (std::fs::read(&stopped_output), std::fs::read(&whole_output));
    std::fs::remove_dir_all(&stopped_directory).expect("删得掉测试自己建的产物目录");
    std::fs::remove_dir_all(&whole_directory).expect("删得掉测试自己建的产物目录");
    assert_eq!(stopped.ending, RoundsEnding::StoppedAtTheLimit(2));
    assert!(!output_after_stop, "停在 --limit-jobs 时不写产物");
    assert_eq!((resumed.computed, resumed.reused), (whole.computed - 2, 2));
    assert_eq!(resumed_text.expect("续跑写出了产物"), whole_text.expect("一次跑完写出了产物"));
}

/// 「--output 已在而内容不同」：退 2 那一支（结局 OutputDiffers），原文件一个字节不动；内容相同的那一支报已经齐了。
#[test]
fn an_existing_output_with_other_content_is_left_alone() {
    let library = test_round_library(&DeviceLibrary::in_memory(), "a1", "b1");
    let directory = test_directory("existing-output");
    let output = directory.join("product.out");
    std::fs::write(&output, "E7RESULT name=config segment=first\n").expect("写得进测试的旧产物");
    let differs = run_on_the_model(&library, &output, None);
    let after = std::fs::read_to_string(&output);
    let complete_output = directory.join("complete.out");
    run_on_the_model(&library, &complete_output, None);
    let complete = run_on_the_model(&library, &complete_output, None);
    std::fs::remove_dir_all(&directory).expect("删得掉测试自己建的产物目录");
    assert_eq!(differs.ending, RoundsEnding::OutputDiffers);
    assert_eq!(after.expect("旧产物还在"), "E7RESULT name=config segment=first\n", "一个字节不动");
    assert_eq!(complete.ending, RoundsEnding::AlreadyComplete);
}

/// 组成清单要两行 code_key、入口与卡对得上。
#[test]
fn the_composition_needs_both_entries() {
    let card_entry = "code_key\te7_index_bench::run_child_on_the_card\ta1\ncode_key\te7_index_bench::ChildJob::child_arguments\tb1\n";
    let unused = DeviceLibrary::in_memory();
    assert!(round_library_from_composition(unused.clone(), &format!("{COMPOSITION_FORM_LINE}\n{card_entry}"), false).is_ok());
    assert!(round_library_from_composition(unused.clone(), &format!("{COMPOSITION_FORM_LINE}\n{card_entry}"), true).is_err(), "--simulated 要模型卡的入口");
    assert!(round_library_from_composition(unused.clone(), card_entry, false).is_err(), "缺 form 行");
    assert!(round_library_from_composition(unused, &format!("{COMPOSITION_FORM_LINE}\ncode_key\te7_index_bench::run_child_on_the_card\ta1\n"), false).is_err(), "缺参数入口");
}

#[test]
fn the_device_line_is_the_query_row_of_that_card_model() {
    let report = "NVIDIA GeForce RTX 5060 Ti, 16311 MiB, 595.84\nNVIDIA GeForce RTX 5090, 32607 MiB, 595.84\nNVIDIA GeForce RTX 5090, 32607 MiB, 595.84\n";
    let lines = device_lines_by_card(report).expect("同一型两张卡报的行相同");
    assert_eq!(lines.get(&Card::Rtx5090).map(String::as_str), Some("NVIDIA GeForce RTX 5090, 32607 MiB, 595.84"));
    assert_eq!(lines.get(&Card::Rtx5060Ti).map(String::as_str), Some("NVIDIA GeForce RTX 5060 Ti, 16311 MiB, 595.84"));
    assert!(device_lines_by_card("NVIDIA GeForce RTX 5090, 32607 MiB, 595.84\nNVIDIA GeForce RTX 5090, 32607 MiB, 595.85\n").is_err(), "同一型驱动不一判错");
    assert_eq!(device_lines_by_card("NVIDIA GeForce RTX 5060 Ti, 16311 MiB, 595.84\n").map(|lines| lines.contains_key(&Card::Rtx5090)), Ok(false), "没有这一型就没有它那一行");
}

// ---------- 取卡来路（不起真卡、不连服务） ----------

/// 单测的领卡通道：五个变量都给、值是名字本身。
fn card_lease_channel_for_the_unit_tests() -> CardLeaseChannel {
    card_lease_channel_of(&|name| Some(format!("value-of-{name}"))).expect("五个变量都给了")
}

#[test]
fn handed_over_cards_are_used_without_leasing() {
    let channel = card_lease_channel_for_the_unit_tests();
    let leases = std::cell::Cell::new(0_u32);
    let lease = |_channel: &CardLeaseChannel| -> Result<String, String> {
        leases.set(leases.get() + 1);
        Ok("10.0.0.1:1:8G".to_string())
    };
    assert_eq!(cards_handed_over_or_leased(Some("10.0.0.1:0:8G"), Some(&channel), &lease), Ok(Some("10.0.0.1:0:8G".to_string())));
    assert_eq!(leases.get(), 0, "环境里已有卡串就不领");
}

#[test]
fn without_handed_over_cards_a_lease_channel_leases_at_that_moment() {
    let channel = card_lease_channel_for_the_unit_tests();
    let leases = std::cell::Cell::new(0_u32);
    let lease = |_channel: &CardLeaseChannel| -> Result<String, String> {
        leases.set(leases.get() + 1);
        Ok("10.0.0.1:1:8G".to_string())
    };
    for handover_value in [None, Some(""), Some("  ")] {
        assert_eq!(cards_handed_over_or_leased(handover_value, Some(&channel), &lease), Ok(Some("10.0.0.1:1:8G".to_string())), "{handover_value:?} 当没交卡串");
    }
    assert_eq!(leases.get(), 3);
}

#[test]
fn without_cards_or_a_lease_channel_nothing_is_taken() {
    let lease = |_channel: &CardLeaseChannel| -> Result<String, String> { panic!("没有领卡通道就不该领") };
    assert_eq!(cards_handed_over_or_leased(None, None, &lease), Ok(None));
    let refused = |_channel: &CardLeaseChannel| -> Result<String, String> { Err("服务回 refused".to_string()) };
    assert_eq!(cards_handed_over_or_leased(None, Some(&card_lease_channel_for_the_unit_tests()), &refused), Err("服务回 refused".to_string()), "领卡被拒是错，不退回别的卡");
}

#[test]
fn a_lease_channel_needs_all_five_variables_present_and_not_empty() {
    for missing in CARD_LEASE_CHANNEL_VARIABLES {
        assert_eq!(card_lease_channel_of(&|name| (name != missing).then(|| format!("value-of-{name}"))), None, "缺 {missing} 不算有通道");
        assert_eq!(card_lease_channel_of(&|name| Some(if name == missing { String::new() } else { format!("value-of-{name}") })), None, "{missing} 是空的不算有通道");
    }
}

#[test]
fn leasing_through_a_fake_client_takes_then_returns_the_cards() {
    use std::os::unix::fs::PermissionsExt;
    let directory = std::env::temp_dir().join(format!("e174-lease-through-a-fake-client-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("建得了临时目录");
    let calls = directory.join("calls.log");
    let client = directory.join("fake-card-lease-client");
    std::fs::write(
        &client,
        format!(
            "#!/bin/sh\nprintf '%s|%s\\n' \"$*\" \"$TASK_SERVER_CARD_LEASE_TOKEN\" >> '{}'\n[ \"$1 $2\" = 'lease take' ] || exit 0\nprintf '10.0.0.1:0:8G\\n'\n",
            calls.display()
        ),
    )
    .expect("写得了假客户端");
    std::fs::set_permissions(&client, std::fs::Permissions::from_mode(0o755)).expect("设得了执行位");
    let leased = lease_cards_through_the_client(&client, &card_lease_channel_for_the_unit_tests()).expect("假客户端交了卡串");
    assert_eq!(leased.cards, "10.0.0.1:0:8G");
    assert!(leased.return_the_cards_now(), "还卡的客户端退 0");
    let logged = std::fs::read_to_string(&calls).expect("假客户端记了调用");
    assert_eq!(logged, "lease take|value-of-TASK_SERVER_CARD_LEASE_TOKEN\nlease return 10.0.0.1:0:8G|value-of-TASK_SERVER_CARD_LEASE_TOKEN\n");
    std::fs::remove_dir_all(&directory).expect("删得了临时目录");
}

#[test]
fn only_the_cards_of_this_host_in_the_handed_order_are_taken() {
    assert_eq!(handed_indices_of_this_host("10.0.0.1:2:8G 10.0.0.2:0:8G 10.0.0.1:0:8G", "10.0.0.1"), Ok(vec![2, 0]));
    assert_eq!(handed_indices_of_this_host("fe80::1:3:8G", "fe80::1"), Ok(vec![3]), "IPv6 地址自带冒号，从右边切两刀");
    assert!(handed_indices_of_this_host("10.0.0.2:0:8G", "10.0.0.1").is_err(), "没有这一台的");
    for unreadable in ["sample-card", "10.0.0.1:x:8G", "10.0.0.1:0:"] {
        assert!(handed_indices_of_this_host(unreadable, "10.0.0.1").is_err(), "「{unreadable}」读不成");
    }
    assert_eq!(this_host_address_emitted("HOSTS=(a)\nTHIS_HOST_ADDRESS=10.0.0.1\n"), Ok("10.0.0.1".to_string()));
    assert!(this_host_address_emitted("THIS_HOST_ADDRESS=''\n").is_err(), "空地址判错");
}

#[test]
fn the_cell_opens_the_first_handed_card_of_its_model() {
    let report = "0, 00000000:41:00.0, 16311\n1, 00000000:01:00.0, 32607\n2, 00000000:81:00.0, 32607\n";
    assert_eq!(bus_for_the_cell_among(Card::Rtx5090, &[2, 1], report), Ok("81:00.0".to_string()), "照给的次序取头一张 5090，不是总线号写死的那张");
    assert_eq!(bus_for_the_cell_among(Card::Rtx5060Ti, &[2, 0], report), Ok("41:00.0".to_string()));
    assert!(bus_for_the_cell_among(Card::Rtx5060Ti, &[1, 2], report).is_err(), "给的卡里没有这一型：判错，不开别的卡");
    assert!(bus_for_the_cell_among(Card::Rtx5090, &[3], report).is_err(), "nvidia-smi 没报的序号判错");
}

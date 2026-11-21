//! 起点那一段与建池：抽到的注入点落进起点那一段，起点那一段的每一项返回错误而不是 panic，建池做完才跑池级 checker，建池在第一道屏障失败、任何系统配置槽都没落盘时以已知红收尾。

use super::*;

/// 摆注入点的候选从第 0 段起算，起点段摆得到：`draw_faults` 的 `(0..marks.len())` 改回 `(1..marks.len())`，
/// 起点段的注入点恒为 0，这里必红（`crates/mutations.tsv` 同名的那一条）。
#[test]
fn the_drawn_injection_points_reach_the_starting_segment() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        8,
        &GenerationWeights::BROAD,
    );
    let injection = inject_faults_into_history(&history, HANDING_EVERY_STEP_IMAGE_OUT, 12);
    let rendered_faults: Vec<String> = injection
        .drawn_faults
        .iter()
        .map(DrawnFault::render)
        .collect();
    assert!(
        injection
            .drawn_faults
            .iter()
            .any(|drawn| drawn.segment == FaultedSegment::TheStartingPoint),
        "12 次抽里一个起点段的注入点都没摆出来：{rendered_faults:?}"
    );
    assert_eq!(
        injection.tally.faults_that_panicked,
        0,
        "起点段上注入打出了 panic\n{}",
        injection.tally.render()
    );
}

/// 只有起点段的一段历史：操作 0 步，跑起来只有 mkfs、取号、暖机、第一个文件。
/// 比重挑一律从第一个文件起的那一组，起点段四步才都走得到。
fn history_with_only_the_starting_point() -> GeneratedHistory {
    generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        0,
        &GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL,
    )
}

/// 在整池第 `ordinal` 次写上注一次写报错，跑那一段历史。`ordinal` 为 `None` 时不注入。
fn run_the_starting_point(
    history: &GeneratedHistory,
    failing_write_ordinal: Option<u64>,
) -> (HistoryRun, SharedFaultPlan) {
    let geometry = HANDING_EVERY_STEP_IMAGE_OUT.device_width.fixed_geometry();
    let plan = match failing_write_ordinal {
        None => SharedFaultPlan::unarmed(geometry),
        Some(ordinal) => SharedFaultPlan::armed(
            geometry,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, ordinal),
        ),
    };
    let run = execute_history_with_faults(
        history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        &SharedStream::new(),
        &plan,
        &mut |_| {},
    );
    (run, plan)
}

/// 起点段失败时那条观察：没 panic，而且停在「起点段的入口返回了错误」上，交回是哪一步。
fn the_starting_point_step_that_returned_an_error(
    run: &HistoryRun,
    ordinal: u64,
) -> Option<StartingPointStep> {
    let observation = match &run.ending {
        HistoryEnding::Completed => return None,
        HistoryEnding::KnownRed { observation, .. }
        | HistoryEnding::NewFinding { observation, .. } => observation,
    };
    assert!(
        observation.panic.is_none(),
        "起点段第 {ordinal} 次写上注入打出了 panic：{:?}",
        observation.panic
    );
    match &observation.harness_judgement {
        Some(HarnessJudgement::TheStartingPointReturnedAnError { step, .. }) => Some(*step),
        Some(_) | None => {
            panic!("起点段第 {ordinal} 次写上注入之后停在别的东西上：{observation:?}")
        }
    }
}

/// 起点段（mkfs、取号、暖机、第一个文件）的四个入口逐个注入：一次 panic 都没有，四步各至少有一次把错交回来。
///
/// 钉的就是这一格：这四步在执行器里若是 `expect`，注入在起点段只会撞出执行器自己的 panic
/// （全在 `history.rs` mkfs 那一处）。这条用例不抽样——
/// 起点段每一次写都注入一遍，四步谁少了一处 `expect` 没改，这里必红。
#[test]
fn every_entry_in_the_starting_segment_returns_an_error_instead_of_panicking() {
    let history = history_with_only_the_starting_point();
    // 一、不注入跑一遍，数起点段发了多少次读 / 写 / 刷盘（验收要的那个数，这里现量）。
    let (measured, measurement) = run_the_starting_point(&history, None);
    assert!(
        matches!(measured.ending, HistoryEnding::Completed),
        "不注入时起点段要跑完：{:?}",
        measured.ending
    );
    let calls = measurement.calls();
    print_uncaptured(&format!(
        "── 起点段逐次注入 ──\n起点段的设备调用：读 {}、写 {}、刷盘 {}，合计 {}\n",
        calls.reads,
        calls.writes,
        calls.barriers,
        calls.reads + calls.writes + calls.barriers
    ));
    // 二、起点段的每一次写各注入一遍：记下是哪一步把错交回来的，panic 一次都不许有。
    let mut failures_by_step: BTreeMap<&'static str, u64> = BTreeMap::new();
    let mut tolerated = 0u64;
    for ordinal in 1..=calls.writes {
        let (run, plan) = run_the_starting_point(&history, Some(ordinal));
        assert_eq!(plan.fired_count(), 1, "第 {ordinal} 次写没注上");
        match the_starting_point_step_that_returned_an_error(&run, ordinal) {
            None => tolerated += 1,
            Some(step) => *failures_by_step.entry(step.name()).or_insert(0) += 1,
        }
    }
    print_uncaptured(&format!(
        "把错交回来的：{failures_by_step:?}；报了成功、照样跑完的 {tolerated} 次\n"
    ));
    for step in [
        StartingPointStep::MakeFilesystem,
        StartingPointStep::AcquireInstance,
        StartingPointStep::WarmUp,
        StartingPointStep::PublishFirstFile,
    ] {
        assert!(
            failures_by_step.contains_key(step.name()),
            "起点段的{}一次都没把错交回来（这一处的 `expect` 还在？）：{failures_by_step:?}",
            step.name()
        );
    }
}

/// 起点段失败时池级 checker 要判的那张镜像交不交出去（harness 不跑 checker，那一遍归给定状态引擎两段，镜像经每步镜像那个钩子交出去）：
/// **mkfs 自己就报错那一次不交**（盘上没有一个做完了的文件系统，checker 判的每一条都没有对象），
/// **取号 / 暖机 / 第一个文件报错那几次照交**（mkfs 已经做完，盘上有池）。
///
/// 口径：分界与「重开走读失败算不算失败」同一条：mkfs 做没做完
/// （`history.rs` 的 `StartingPointStep::finished_filesystem_is_on_the_devices`）。
/// 起点段 44 次写逐次注入，两边都数出来——口径被改掉（两边都跑、或者两边都不跑），这里必红。
#[test]
fn the_pool_checker_runs_on_a_starting_point_failure_only_when_make_filesystem_finished() {
    let history = history_with_only_the_starting_point();
    // 不注入时起点段跑完，起点之后交一张镜像：下面那两格拿它当对照。
    let (measured, measurement) = run_the_starting_point(&history, None);
    assert!(
        matches!(measured.ending, HistoryEnding::Completed),
        "不注入时起点段要跑完：{:?}",
        measured.ending
    );
    assert_eq!(
        measured.tally.step_images_handed_out, 1,
        "起点段跑完之后该交一张镜像"
    );
    assert_eq!(measured.tally.checker_runs, 0, "harness 不跑池级 checker");
    let mut step_images_handed_out_by_step: BTreeMap<&'static str, BTreeSet<u64>> = BTreeMap::new();
    for ordinal in 1..=measurement.calls().writes {
        let (run, _plan) = run_the_starting_point(&history, Some(ordinal));
        let Some(step) = the_starting_point_step_that_returned_an_error(&run, ordinal) else {
            continue;
        };
        step_images_handed_out_by_step
            .entry(step.name())
            .or_default()
            .insert(run.tally.step_images_handed_out);
        let expected_step_images = u64::from(step.finished_filesystem_is_on_the_devices());
        assert_eq!(
            run.tally.step_images_handed_out,
            expected_step_images,
            "起点段第 {ordinal} 次写上注入、停在{}：mkfs 做完了才该交那张镜像",
            step.name()
        );
    }
    print_uncaptured(&format!(
        "── 起点段失败时交出去的镜像 ──\n每一步报错时交了几张：{step_images_handed_out_by_step:?}\n"
    ));
    // 两边都要有样本，不然上面那条断言是空过的。
    assert_eq!(
        step_images_handed_out_by_step.get(StartingPointStep::MakeFilesystem.name()),
        Some(&BTreeSet::from([0])),
        "mkfs 报错那几次要有样本，而且一张镜像都没交：{step_images_handed_out_by_step:?}"
    );
    for step in [
        StartingPointStep::AcquireInstance,
        StartingPointStep::WarmUp,
        StartingPointStep::PublishFirstFile,
    ] {
        assert_eq!(
            step_images_handed_out_by_step.get(step.name()),
            Some(&BTreeSet::from([1])),
            "{}报错那几次要有样本，而且各交过一张：{step_images_handed_out_by_step:?}",
            step.name()
        );
    }
}

/// ：mkfs 在写到系统配置之前失败的盘面就是全池皆废，I-8.5（tail 槽有效集非空）照判红，红是对的；
/// 故障注入的「已知红」清单加这一类。默认批在种子 7463871032432355120 上摆的那一个注入点（整池第 1 次 barrier 报错，落在 mkfs 第一道屏障上，
/// 那一刻 mkfs 只清过零、写过实例表与树表的创世单元，系统配置槽一次都没写）原样重跑：注入之后的镜像上只红 I-8.5、
/// 重开走读失败，判法记成那一条「已知红」，不记新发现。
///
/// 判别力自证：判法改回不认这一类（`known_red_form_only_after_a_fault` 恒交回 None，`crates/mutations.tsv` 同名那一条），
/// 这一次落成新发现 `CheckerViolations { invariants: ["I-8.5"] }`，同名的给定状态用例红（池级 checker 那一遍与归类在那里，
/// `crates/singlefs-verification/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）；这里只核注入的结局与交出去的盘。
#[test]
fn make_filesystem_failing_at_its_first_barrier_before_any_system_configuration_slot_lands_ends_known_red_not_a_new_finding(
) {
    let (injection, handed_out) = the_history_whose_make_filesystem_fails_at_its_first_barrier();
    let rendered = injection.tally.render();
    print_uncaptured(&format!(
        "── mkfs 第一道屏障报错：全池皆废、只红 I-8.5 ──\n{rendered}"
    ));
    let expected_drawn = DrawnFault {
        fault: InjectedFault::BarrierFails,
        call_ordinal: 1,
        segment: FaultedSegment::TheStartingPoint,
    };
    assert_eq!(
        injection.drawn_faults,
        vec![expected_drawn],
        "注入点是整池第 1 次 barrier、落在起点段\n{rendered}"
    );
    // 停下的是 mkfs 自己，重开走读失败而盘上本来就没有一版立得住：这一形的前两样事实真的在。
    assert_eq!(
        injection.tally.starting_point_failures,
        BTreeMap::from([(StartingPointStep::MakeFilesystem.name(), 1)]),
        "起点段里把错交回来的是 mkfs\n{rendered}"
    );
    assert_eq!(
        injection
            .tally
            .reopens_that_correctly_found_nothing_to_recover,
        1,
        "重开走读失败、盘上本来就没有一版立得住\n{rendered}"
    );
    assert!(
        injection.new_findings.is_empty() && injection.known_red_hits.is_empty(),
        "harness 那一截（模型、RefFS、执行器）既没有新发现、也没有已知红（I-8.5 那一判归池级 checker）：{:?}\n{rendered}",
        injection
            .new_findings
            .iter()
            .map(|finding| &finding.signature)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        injection.tally.checker_runs_on_the_image_after_the_fault, 0,
        "harness 不在注入之后的盘上跑池级 checker（那一遍归给定状态引擎两段）\n{rendered}"
    );
    let [images] = handed_out.as_slice() else {
        panic!(
            "一个注入点交出去一份盘：交了 {} 份\n{rendered}",
            handed_out.len()
        );
    };
    assert!(
        images.step_images_of_the_run_with_the_fault.is_empty(),
        "mkfs 自己报错，一张每步镜像都不交\n{rendered}"
    );
    assert_eq!(injection.tally.faults_that_panicked, 0, "{rendered}");
}

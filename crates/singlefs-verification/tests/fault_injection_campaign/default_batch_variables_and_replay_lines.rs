//! 故障注入的默认批与照变量跑的批：注入之后返回错误而不是 panic、各条路径都走到、变量为零时点名变量与取值 panic、重放行在入口上跑出同一批、一段写死的历史注入十二次、一个线程与四个线程报告逐字相同。

use super::*;

/// 故障注入这一批：不设变量是默认批（[`default_batch_campaign`]：这个测试周期的种子基起 24 段、每段 20 步、每段摆 4 个注入点）；
/// 设了 `SINGLEFS_FAULT_INJECTION_*` 的照变量跑（[`fault_injection_campaign_from`]，变量与判红时打的重放行同一张表，没设的那几项取默认批的）。
/// 每个注入点重跑一遍这段历史，
/// 判「返回错误而不是 panic」与「重开恢复到模型允许的版本」；「已知红」清单里的形态照记不停，清单外的一条都不许有。
///
/// 读出坏字节（`read_returns_corrupted_bytes`）只注入在一次读上：释放之前读盘核那一读撞上它时重读一次就对得上，不隔离
/// （D19（块指针的结构与宽度预算） 已定项 5：核出对不上也先重读一次）。只重读的是读不出那一半时，这一格在种子 7463871032432355113
/// 第 7 步（覆盖写）把一对好槽隔离掉、池级 checker 判 I-3.11 红。**每一读都给坏字节**（坏读一直在、盘上的字节是好的）时两次都对不上，
/// 照规则隔离那一对好槽——那是「两次都对不上才隔离」认下的形态，不是新发现；随机注入不摆这一形（一次注入只坏一次调用），
/// 它的样子钉在 `release_checksum_quarantine.rs` 的用例 13。
#[test]
fn fault_injection_returns_errors_instead_of_panicking() {
    let campaign = fault_injection_campaign_from(&process_environment_variable);
    let started = std::time::Instant::now();
    let report = run_fault_injection_campaign(&campaign);
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 故障注入 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert_eq!(
        report.tally.faults_that_panicked,
        0,
        "注入之后 core panic 了：这是被测的性质（返回错误而不是 panic）；{}\n{rendered}",
        how_to_replay(&report)
    );
    assert!(
        report.new_findings.is_empty(),
        "注入之后「已知红」清单外的失败；{}\n{rendered}",
        how_to_replay(&report)
    );
    // 各条路径都跑到了只在默认批上判：比重、规模换了的一批未必每条路径都摆得到，那一批只判不 panic 与清单外的失败。
    if default_batch_campaign(campaign.worker_threads) == campaign {
        assert_every_fault_injection_path_was_exercised(&report);
    }
}

/// 各条路径真的跑到了：六种注入都摆过、起点段与每一类会写盘的操作上都注入过、重开都跑过、四类写落点都注入过。
fn assert_every_fault_injection_path_was_exercised(report: &FaultInjectionReport) {
    let tally = &report.tally;
    let rendered = report.render();
    assert!(
        tally.faults >= DEFAULT_BATCH_SEEDS,
        "摆出来的注入点太少（{}）：每段至少该摆出一个\n{rendered}",
        tally.faults
    );
    // 六种全抽：前四种是里程碑逐字写的（写、读、刷盘返回块设备错，加上读回改坏的字节），
    // 后两种是设备说谎（`draw_injected_fault` 的 `below(6)` 钉在这一条上）。
    for fault in [
        InjectedFault::WriteFails,
        InjectedFault::ReadFails,
        InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FlippedBit {
                byte_index: 0,
                bit_index: 0,
            },
        },
        InjectedFault::BarrierFails,
        InjectedFault::WriteIsSwallowed,
        InjectedFault::BarrierIsSwallowed,
    ] {
        assert!(
            tally.faults_by_kind.contains_key(fault.name()),
            "一次都没注入过 {}：这一种注入在默认批里没跑到\n{rendered}",
            fault.name()
        );
    }
    // 起点段（mkfs、取号、暖机、第一个文件）上注入过：`draw_faults` 的候选从起点段起算时才不恒为 0。
    assert!(
        tally.faults_on_segment(FaultedSegmentKind::TheStartingPoint) > 0,
        "起点段上一次都没注入过：`draw_faults` 的候选区间又把第 0 段排掉了\n{rendered}"
    );
    // 说谎的设备真的丢掉过内容（盘面不一致只有池级 checker 看得见）：这一格归给定状态引擎两段，同一批种子在
    // `crates/singlefs-verification/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`
    // 里核说谎的设备留下的不一致至少一次（`mutation-sampling.md`「取样点不敏感」）。
    assert!(
        tally.reopens_reading_a_file > 0 && tally.reopens_without_a_file > 0,
        "重开的两种结局没都见过：读回文件 {}、没有文件 {}\n{rendered}",
        tally.reopens_reading_a_file,
        tally.reopens_without_a_file
    );
    assert!(
        tally
            .faults_by_outcome
            .contains_key(FaultOutcome::SurfacedAsAnError.name()),
        "一次都没有「注入那一步返回了错误」：这一件要的就是这一格\n{rendered}"
    );
    // 会写盘的四类操作：发布（覆盖写）、可写挂载、挂着时回退（一次向前发布）、抬 F。冷启动只读不写，不要求写落点上注入过。
    for operation in [
        HistoryOperationKind::PublishOverwrite,
        HistoryOperationKind::CloseAndMountWritable,
        HistoryOperationKind::RollBackWhileMounted,
        HistoryOperationKind::RaiseRollbackFloor,
    ] {
        assert!(
            tally.faults_on_segment(FaultedSegmentKind::Operation(operation)) > 0,
            "{operation:?} 上一次都没注入过（验收要「每个发布步骤与挂载步骤上各注入过至少一次」）\n{rendered}"
        );
    }
    assert!(
        tally.reopens >= tally.faults,
        "每次注入之后都要重开一次：重开 {} 次、注入 {} 次\n{rendered}",
        tally.reopens,
        tally.faults
    );
    assert_eq!(
        tally.reopens_outside_everything_the_model_allows,
        0,
        "重开走到了模型都不允许的版本：{}\n{rendered}",
        how_to_replay(report)
    );
    assert_eq!(
        tally.images_after_the_fault_handed_out, tally.faults,
        "每次注入之后重开的盘都要交出去一次（池级 checker 那一遍归给定状态引擎两段）\n{rendered}"
    );
    assert_eq!(
        tally.checker_runs_on_the_image_after_the_fault, 0,
        "harness 不在注入之后的盘上跑池级 checker\n{rendered}"
    );
}

/// 一个变量都没设读回默认批；设了的每一项照变量读回、没设的那几项仍取默认批的（设了变量也读回默认批时这一条红）。线程数不比。
#[test]
fn the_fault_injection_campaign_is_the_default_batch_without_variables_and_follows_the_variables_that_are_set(
) {
    let default_batch = default_batch_campaign(FaultInjectionWorkerThreads::GivenByCaller(1));
    let without_variables = fault_injection_campaign_from(&|_variable: &str| None);
    assert_eq!(
        FaultInjectionCampaign {
            worker_threads: default_batch.worker_threads,
            ..without_variables
        },
        default_batch,
        "一个 SINGLEFS_FAULT_INJECTION_* 都没设，跑的该是默认批"
    );
    let given = |variable: &str| {
        match variable {
            "SINGLEFS_FAULT_INJECTION_FIRST_SEED" => Some("11"),
            "SINGLEFS_FAULT_INJECTION_SEEDS" => Some("512"),
            "SINGLEFS_FAULT_INJECTION_OPERATIONS" => Some("30"),
            "SINGLEFS_FAULT_INJECTION_FAULTS" => Some("6"),
            _ => None,
        }
        .map(str::to_string)
    };
    assert_eq!(
        FaultInjectionCampaign {
            worker_threads: default_batch.worker_threads,
            ..fault_injection_campaign_from(&given)
        },
        FaultInjectionCampaign {
            first_seed: 11,
            seed_count: 512,
            operations_per_history: 30,
            faults_per_history: 6,
            ..default_batch
        },
        "设了的种子起点、种子数、步数、注入次数照变量读回，没设的比重与跑法取默认批的"
    );
}

/// 故障注入入口里种子数写 0：停下，消息带变量名与原值，不照收（一段都不跑的一批照「全部返回错误、不 panic」判成绿）。
#[test]
#[should_panic(expected = "SINGLEFS_FAULT_INJECTION_SEEDS=0：只认 正整数")]
fn a_fault_injection_seed_count_of_zero_panics_naming_the_variable_and_the_value() {
    let _campaign = fault_injection_campaign_from(&|variable: &str| {
        (variable == "SINGLEFS_FAULT_INJECTION_SEEDS").then(|| "0".to_string())
    });
}

/// 故障注入入口里每段注入次数写 0：停下，消息带变量名与原值，不照收（一次都不注入的一批照样判成绿）。
#[test]
#[should_panic(expected = "SINGLEFS_FAULT_INJECTION_FAULTS=0：只认 正整数")]
fn a_fault_injection_fault_count_of_zero_panics_naming_the_variable_and_the_value() {
    let _campaign = fault_injection_campaign_from(&|variable: &str| {
        (variable == "SINGLEFS_FAULT_INJECTION_FAULTS").then(|| "0".to_string())
    });
}

/// 判红时对一批故障注入会打的那一行重放命令（`how_to_replay`；造一份报告，不必真红）原样喂回故障注入那条入口
/// （`fault_injection_campaign_from`，入口用的同一个函数）：种子区间、每段步数、每段注入几次、比重、跑法与原来那一批逐项相同，
/// 第一段生成的历史逐步相同。不跑注入：同一组参数下摆出来的注入点由种子定，这里只核参数一项不漏地回来了
/// （重放行要带比重与跑法，否则照抄回不来）。
fn the_replay_line_printed_for_a_batch_runs_the_same_campaign_on_the_campaign_entry(
    campaign: &FaultInjectionCampaign,
) {
    let report = FaultInjectionReport {
        first_seed: campaign.first_seed,
        seed_count: campaign.seed_count,
        operations_per_history: campaign.operations_per_history,
        faults_per_history: campaign.faults_per_history,
        weights: campaign.weights,
        execution: campaign.execution,
        tally: FaultInjectionTally::default(),
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
    };
    let replay = how_to_replay(&report);
    let assignments = environment_assignments_in_line(&replay);
    let replayed =
        fault_injection_campaign_from(&|variable: &str| assignments.get(variable).cloned());
    assert_eq!(
        (
            replayed.first_seed,
            replayed.seed_count,
            replayed.operations_per_history,
            replayed.faults_per_history,
            replayed.weights,
            replayed.execution,
        ),
        (
            campaign.first_seed,
            campaign.seed_count,
            campaign.operations_per_history,
            campaign.faults_per_history,
            campaign.weights,
            campaign.execution,
        ),
        "照重放行跑故障注入入口，跑的不是原来那一批（种子、种子数、步数、注入次数、比重、跑法）：{replay}"
    );
    assert_eq!(
        generate_history_with_weights(
            HistorySeed(replayed.first_seed),
            replayed.operations_per_history,
            &replayed.weights
        ),
        generate_history_with_weights(
            HistorySeed(campaign.first_seed),
            campaign.operations_per_history,
            &campaign.weights
        ),
        "第一段生成的历史逐步相同"
    );
}

#[test]
fn replay_line_of_the_default_batch_runs_the_same_campaign_on_the_campaign_entry() {
    the_replay_line_printed_for_a_batch_runs_the_same_campaign_on_the_campaign_entry(
        &default_batch_campaign(FaultInjectionWorkerThreads::GivenByCaller(1)),
    );
}

/// 每一项都与默认批不同的一批：重放行漏写哪一项，故障注入入口就退回默认批的那一项，读回的不是这一批。
/// 前提断言先核「每一项都不同」，不然漏写那一项读回的恰好还是它。
#[test]
fn replay_line_of_a_batch_unlike_the_default_batch_runs_the_same_campaign_on_the_campaign_entry() {
    let campaign = FaultInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 7,
        seed_count: 3,
        operations_per_history: 11,
        faults_per_history: 2,
        weights: GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR,
        // 每一步的池级 checker 只剩不跑那一种（那一遍归给定状态引擎两段），那一项取不出与默认不同的值，前提断言不核它
        execution: HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::SkippedByTheTestOnlySwitch,
        },
        worker_threads: FaultInjectionWorkerThreads::GivenByCaller(1),
    };
    let defaults = fault_injection_campaign_from(&|_variable: &str| None);
    assert!(
        campaign.first_seed != defaults.first_seed
            && campaign.seed_count != defaults.seed_count
            && campaign.operations_per_history != defaults.operations_per_history
            && campaign.faults_per_history != defaults.faults_per_history
            && campaign.weights != defaults.weights
            && campaign.execution.device_width != defaults.execution.device_width
            && campaign.execution.space_admission != defaults.execution.space_admission,
        "前提：这一批除每一步的池级 checker 之外的每一项都与默认批不同：{campaign:?} 对 {defaults:?}"
    );
    the_replay_line_printed_for_a_batch_runs_the_same_campaign_on_the_campaign_entry(&campaign);
}

/// 一段写死的历史上把注入点摆满：同一段历史注入 12 次，逐次判「返回错误而不是 panic」。
#[test]
fn one_fixed_history_injects_twelve_faults_and_none_of_them_panics() {
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        24,
        &GenerationWeights::BROAD,
    );
    let injection = inject_faults_into_history(&history, HANDING_EVERY_STEP_IMAGE_OUT, 12);
    let rendered = injection.tally.render();
    print_uncaptured(&format!("── 故障注入：一段写死的历史 ──\n{rendered}"));
    assert!(
        injection.drawn_faults.len() >= 8,
        "12 次抽里摆出来的注入点太少（{}）",
        injection.drawn_faults.len()
    );
    assert_eq!(
        injection.tally.faults_that_panicked, 0,
        "注入之后 core panic 了\n{rendered}"
    );
    assert!(
        injection.new_findings.is_empty(),
        "「已知红」清单外的失败\n{rendered}"
    );
}

/// 线程数换了，报告逐字相同：计数按片的次序相加、新发现按种子从小到大留第一个
/// （`implementation-workflow.md`「测试与崩溃检测优先多线程」的「合并要确定」）。
#[test]
fn the_report_is_the_same_text_with_one_worker_thread_and_with_four() {
    let campaign = |threads| FaultInjectionCampaign {
        seed_count: 6,
        operations_per_history: 12,
        faults_per_history: 2,
        ..default_batch_campaign(FaultInjectionWorkerThreads::GivenByCaller(threads))
    };
    let one = run_fault_injection_campaign(&campaign(1));
    let four = run_fault_injection_campaign(&campaign(4));
    assert_eq!(
        one.render(),
        four.render(),
        "线程数变了报告就变了：并到一起的次序不确定"
    );
}

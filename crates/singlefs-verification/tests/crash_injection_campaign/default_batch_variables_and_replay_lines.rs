//! 崩溃注入的默认批、照变量跑的批与回退抽样：测试周期的种子基、默认批与照变量跑的批共用的入口与那一组断言（用例本身在根文件里：门禁按短名点它）、每一截都在每个崩溃状态上跑过、各条路径都走到、多线程与单线程报告逐字相同、抽崩溃点不改生成的历史、重放行在入口上跑出同一批。

use super::*;

/// 种子基是一个测试周期的常量：开一个周期抽一次，抽完在这个周期之内写死。周期从「开一个新里程碑」与
/// 「显式要求从零开始测试 / 重建测试」里先到的那一件开始。
///
/// 这一条钉两样：
/// 1. 常量就是当前这个周期（里程碑「覆盖写、释放、回退与复用」）抽出来的那个数。改成别的数 = 把这个周期的实验数据整批换掉，
///    而 `crates/mutations.tsv` 里名字以「增补 3 第 2 件」起头的那些「这条变异必须红」与里程碑那几条验收，量的都是这一批历史；
/// 2. 从它起跑出来的报告把它带了回来（报告第一行与重放那一行都带着它），判红了才重放得出来。
///
/// 随机历史那个二进制的五段用的是同一个常量——两边同基，那一半由它自己那条用例钉
/// （`random_histories.rs` 的 `the_five_samplings_start_from_the_test_cycle_seed_base`）。
#[test]
fn the_test_cycle_seed_base_is_the_number_drawn_for_this_cycle() {
    assert_eq!(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        7_463_871_032_432_355_113,
        "种子基不是 2026-09-20 抽的那个数：它永久固定，换了就让绑在这批历史上的判红、镜像与过程文件整批作废\
         （为什么固定、要多测历史怎么追加区间，写在 SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE 的文档注释里）"
    );
    let report = run_through_the_engine(&CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 2,
        operations_per_history: 8,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    });
    assert_eq!(
        report.first_seed, SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        "报告没把跑的那个种子基带回来"
    );
    let replay = how_to_replay(&report);
    assert!(
        replay.contains(&SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE.to_string()),
        "重放那一行里没有种子基，判红了就重放不出来：{replay}"
    );
    assert!(
        report.render().contains("这个测试周期写死的"),
        "报告第一行没说清种子基是这个测试周期写死的：\n{}",
        report.render()
    );
}

/// 照这一批的那一组断言判（[`CrashInjectionAssertions`]）：出红的那条用例与照它的重放行跑的崩溃注入入口都经这一个函数，
/// 所以照抄重放行跑，红在同一条断言上（代码三方 test-script-align-code-r1 判决 S2）。
pub(super) fn judge_crash_injection_report(report: &CrashInjectionReport) {
    assert!(
        report.new_findings.is_empty(),
        "崩溃状态上「已知红」清单外的失败；{}\n{}",
        how_to_replay(report),
        report.render()
    );
    match report.assertions {
        CrashInjectionAssertions::NewFindingsOnly => {
            assert_at_least_one_crash_state_was_judged(report);
        }
        CrashInjectionAssertions::DefaultBatch => {
            assert_each_stage_ran_on_every_crash_state(report);
            assert_every_crash_injection_path_was_exercised(report);
        }
        CrashInjectionAssertions::RollbackSampling => {
            assert_the_rollback_sampling_placed_crash_states_and_recovered_from_each(report);
        }
    }
}

/// 只判清单外失败的那一组也要判过至少一个崩溃状态：一个都没判过的一批没有清单外失败，只是因为什么都没判
/// （上游 `show-me-test.md`「扫到 0 项也不是通过」）。别的两组各自要的计数都不少于段数，已经含着这一条。
fn assert_at_least_one_crash_state_was_judged(report: &CrashInjectionReport) {
    assert!(
        report.tally.crash_points > 0,
        "这一批一个崩溃状态都没判过，扫到 0 项不算通过；{}",
        how_to_replay(report)
    );
}

/// 一段都不跑的一批照只判清单外失败的那一组判：判红，不因为没有清单外失败就判成绿。
// crash-case-check:not-a-crash-case 一段都不跑的一批判红：造的计数、不跑崩溃状态，核只判清单外失败那一组断言的判别力
#[test]
#[should_panic(expected = "这一批一个崩溃状态都没判过")]
fn a_batch_that_judged_no_crash_state_is_red_under_the_new_findings_only_assertions() {
    let report = run_through_the_engine(&CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 0,
        operations_per_history: 8,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(1),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    });
    judge_crash_injection_report(&report);
}

/// 默认批那一组的结构计数：三截每一截在每个崩溃状态上都真的跑了，二次崩溃摆出来过、每个都判过，恢复没失败过。
/// 先于各条路径那一半判：结构计数缺了，红在这里点名的那一截上。
fn assert_each_stage_ran_on_every_crash_state(report: &CrashInjectionReport) {
    let replay = how_to_replay(report);
    let tally = &report.tally;
    // 每段历史都摆出过崩溃状态（总数够放得过「几段一个都没有、别的段多摆」；
    // 为什么每段都摆得出见 [`assert_every_crash_injection_path_was_exercised`]）。
    assert_eq!(
        tally.histories_without_any_crash_point, 0,
        "有历史段一个崩溃状态都没摆出来；{replay}"
    );
    assert!(
        tally.crash_points >= report.seed_count,
        "平均每段连一个崩溃状态都没摆出来：{}；{replay}",
        tally.crash_points
    );
    for (stage, runs) in [
        ("问模型", tally.model_judgements),
        ("池级 checker", tally.checker_runs),
        ("记录核对器", tally.record_checks),
        (
            "崩溃后镜像上的可写挂载",
            tally.writable_mounts_after_the_crash,
        ),
        (
            "可写挂载之后的池上 checker",
            tally.checker_runs_after_the_writable_mount,
        ),
        (
            "可写挂载之后的池上记录核对器",
            tally.record_checks_after_the_writable_mount,
        ),
    ] {
        assert_eq!(
            runs, tally.crash_points,
            "每个崩溃状态都要跑过{stage}；{replay}"
        );
    }
    assert!(
        tally.second_crash_points >= 1,
        "挂载途中一个二次崩溃都没摆出来；{replay}"
    );
    for (stage, runs) in [
        ("问模型", tally.second_crash_model_judgements),
        ("池级 checker", tally.second_crash_checker_runs),
        ("记录核对器", tally.second_crash_record_checks),
    ] {
        assert_eq!(
            runs, tally.second_crash_points,
            "每个二次崩溃都要跑过{stage}；{replay}"
        );
    }
    assert_eq!(tally.recoveries_failed, 0, "崩溃之后恢复失败过；{replay}");
}

/// 默认批：这个测试周期的种子基起 [`DEFAULT_BATCH_SEEDS`] 段、每段 [`DEFAULT_BATCH_OPERATIONS_PER_HISTORY`] 步、[`DEFAULT_BATCH_DRAW`]、
/// 各类操作都抽。跑完把镜像目录删掉：临时目录里不留东西（门禁 77 号判跑完机器干不干净）。
fn default_batch_campaign(worker_threads: CrashInjectionWorkerThreads) -> CrashInjectionCampaign {
    CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: DEFAULT_BATCH_SEEDS,
        operations_per_history: DEFAULT_BATCH_OPERATIONS_PER_HISTORY,
        draw: DEFAULT_BATCH_DRAW,
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads,
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::DefaultBatch,
    }
}

/// 默认批那一组的覆盖：各条路径都跑到了（四种落点都扣下过、段内摆出过洞、起点那一段摆到过、截在发布中间过……）。
fn assert_every_crash_injection_path_was_exercised(report: &CrashInjectionReport) {
    let tally = &report.tally;
    let replay = how_to_replay(report);
    // 总数够不说明每段都摆到了：`crash_points >= seed_count` 放得过「几段一个都没有、别的段多摆」。
    // 从第一个文件起的历史，起点那一段（取号、暖机、
    // 第一个文件）就有写、都是候选段；从 mkfs 起的历史要等第一次做成的可写挂载，关着会话时只抽可写挂载与冷启动两种，
    // 一次可写挂载都没做成就收尾的（新发现或已知红收尾在那之前）这一条照判红。推的，没在默认批上跑过。
    assert_eq!(
        tally.histories_without_any_crash_point, 0,
        "有历史段一个崩溃状态都没摆出来；{replay}"
    );
    assert!(
        tally.crash_points >= report.seed_count,
        "平均每段连一个崩溃状态都没摆出来：{}；{replay}",
        tally.crash_points
    );
    assert_eq!(
        tally.model_judgements, tally.crash_points,
        "每个崩溃状态都要问过模型；{replay}"
    );
    assert_eq!(
        tally.checker_runs, tally.crash_points,
        "每个崩溃状态都要跑过池级 checker；{replay}"
    );
    assert_eq!(
        tally.record_checks, tally.crash_points,
        "每个崩溃状态都要跑过记录核对器；{replay}"
    );
    for kind in [
        StepKind::UnitWrite,
        StepKind::JournalRecord,
        StepKind::RootRecordFua,
        StepKind::SystemConfigurationSlot,
    ] {
        assert!(
            tally
                .crash_points_withholding_write_kind
                .get(kind.name())
                .copied()
                .unwrap_or(0)
                >= 1,
            "一次都没扣下 {}：{:?}；{replay}",
            kind.name(),
            tally.crash_points_withholding_write_kind
        );
    }
    for kind in [
        HistoryOperationKind::PublishOverwrite,
        HistoryOperationKind::CloseAndMountWritable,
    ] {
        assert!(
            tally
                .crash_points_by_operation_kind
                .get(&kind)
                .copied()
                .unwrap_or(0)
                >= 1,
            "一个崩溃状态都没落在 {kind:?} 里：{:?}；{replay}",
            tally.crash_points_by_operation_kind
        );
    }
    assert!(
        tally.crash_points_withholding_a_write_before_a_persisted_one >= 1,
        "一个段内有洞的状态都没摆出来（等于还是只截前缀）；{replay}"
    );
    assert!(
        tally.crash_points_inside_the_starting_point >= 1,
        "起点那一段里一个崩溃状态都没摆到（2026-09-20 才放开）；{replay}"
    );
    assert!(
        tally.crash_points_inside_an_unfinished_publish >= 1,
        "一次都没把发布截在它的根落盘之前；{replay}"
    );
    assert!(
        tally.crash_points_that_land_before_the_last_committed_version >= 1,
        "一个崩溃状态都没截回到最后一版之前（全落在流的尾巴上）；{replay}"
    );
    assert_eq!(tally.recoveries_failed, 0, "崩溃之后恢复失败过；{replay}");
    assert!(
        tally.recoveries_reading_a_file >= 1,
        "崩溃之后一次都没读回文件；{replay}"
    );
    assert!(
        tally.recoveries_without_a_file >= 1,
        "崩溃之后一次都没落在树表 0 条的一版上；{replay}"
    );
    assert!(
        tally.recoveries_that_applied_journal_records >= 1,
        "恢复一次都没施加过 journal 记录前缀（journal 在这一段里没承重）；{replay}"
    );
    assert!(
        tally.model_contents_compared >= 1,
        "模型一次都没比过崩溃之后读回的内容；{replay}"
    );
    assert!(
        tally.model_versions_without_a_file_matched >= 1,
        "模型一次都没对上「这一版树表 0 条」；{replay}"
    );
    for invariant in ["I-3.1", "I-5.4"] {
        assert!(
            tally.invariant_holds.get(invariant).copied().unwrap_or(0) >= 1,
            "崩溃状态上 checker 一次都没判过 {invariant}（全是不适用）；{replay}"
        );
    }
}

/// 多线程只影响跑得多快：同一批种子，1 个线程与 4 个线程跑出来的报告逐字相同
/// （`.claude/rules/implementation-workflow.md`「测试与崩溃检测优先多线程」的「合并要确定」）。
/// 计数按片的次序相加、新发现按种子留第一个，所以结论与线程数、调度次序无关；进度行按到达次序打，不带判定，不在比对之内；
/// 判红的崩溃镜像每次调用写进自己新建的目录，路径不在比对之内（比的是 `render_the_conclusions`）。
/// 两遍拿的是同一个种子基（这个测试周期写死的那一个），比的就是同一批历史上的同一批崩溃状态。
#[test]
fn one_thread_and_four_threads_render_the_same_report() {
    let first_seed = SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE;
    let campaign = |worker_threads: usize| {
        run_through_the_engine(&CrashInjectionCampaign {
            first_seed,
            seed_count: 8,
            operations_per_history: 16,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 3,
            },
            weights: GenerationWeights::BROAD,
            execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(worker_threads),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        })
    };
    let report_on_one_thread = campaign(1);
    let report_on_four_threads = campaign(4);
    for report in [&report_on_one_thread, &report_on_four_threads] {
        if printed_not_run_because_the_engine_gave_placeholder_conclusions(
            PlaceholderTablesOfTheRun::of_the_tally(&report.tally),
            "one_thread_and_four_threads_render_the_same_report",
        ) {
            return;
        }
    }
    let on_one_thread = report_on_one_thread.render_the_conclusions();
    let on_four_threads = report_on_four_threads.render_the_conclusions();
    assert_eq!(
        on_one_thread, on_four_threads,
        "线程数不改结论（种子基 {first_seed}）"
    );
    assert!(
        on_one_thread.contains("崩溃状态 "),
        "报告里有崩溃状态的计数：\n{on_one_thread}"
    );
}

/// 偏向抬 F 之后回退那一组比重的取样点：这个测试周期的种子基起 12 段、每段 20 步、每段摆 6 个崩溃状态，一条清单外的失败都不许有。
///
/// 这一段**不**断言「某一类操作里至少摆到一个崩溃状态」：那是抽样摆出来的形态，下一个测试周期重抽种子基就可能一次都没有
/// （前提不成立就不调入口；在随机种子基上实测撞到过：49 个崩溃状态里 `RaiseRollbackFloor` 是 0），
/// 那样的断言判的是抽样运气，不是代码。回退与抬 F 罩得到这一条由上面那条写死历史的用例钉着，那里是全枚举。
/// 各类操作各摆到几个照打进报告，给人看。
// crash-case-check:not-a-crash-case 偏向抬 F 之后回退的随机取样点（种子基起 12 段、每段 6 个崩溃状态），抽样摆点、不枚举；随机崩溃注入的默认批登记成 crash-case:crash-injection-default-batch
#[test]
fn crash_states_land_inside_rollbacks_and_floor_raises() {
    let report = run_through_the_engine(&rollback_sampling_campaign(
        CrashInjectionWorkerThreads::from_the_environment(),
    ));
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 崩溃注入：偏向抬 F 之后回退的取样点 ──\n{rendered}"
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&report.tally),
        "crash_states_land_inside_rollbacks_and_floor_raises",
    ) {
        return;
    }
    judge_crash_injection_report(&report);
}

/// 回退取样那一组：摆出过崩溃状态、恢复没失败过（不钉抽样摆出来的形态，见用到它的那条用例的文档注释）。
fn assert_the_rollback_sampling_placed_crash_states_and_recovered_from_each(
    report: &CrashInjectionReport,
) {
    assert!(
        report.tally.crash_points >= 1,
        "一个崩溃状态都没摆出来；{}",
        how_to_replay(report)
    );
    assert_eq!(
        report.tally.recoveries_failed,
        0,
        "崩溃之后恢复失败过；{}",
        how_to_replay(report)
    );
}

/// 偏向抬 F 之后回退的那一批：种子基起 12 段、每段 20 步、每段 6 个崩溃状态。
fn rollback_sampling_campaign(
    worker_threads: CrashInjectionWorkerThreads,
) -> CrashInjectionCampaign {
    CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 12,
        operations_per_history: 20,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 6,
        },
        weights: GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads,
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::RollbackSampling,
    }
}

/// 崩溃注入那条入口读环境：变量与判红时打的重放行同一张表（`crash_injection_campaign_from_environment`），没设的取默认批的
/// （[`default_batch_campaign`]，照默认批那一组断言判），线程数从 `SINGLEFS_CRASH_INJECTION_THREADS` 取。入口、「不设变量取默认批」那一格
/// 与「重放行喂回入口」那几条自证都经这一个函数。读回的不是默认批时（设变量照重放行复现、或换一批历史看现场）判红的崩溃镜像留着、
/// 路径打进报告；默认批跑完删掉，临时目录里不留东西。
pub(super) fn crash_injection_campaign_from(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> CrashInjectionCampaign {
    let default_batch = default_batch_campaign(CrashInjectionWorkerThreads::from_the_environment());
    let campaign = crash_injection_campaign_from_environment(lookup, &default_batch);
    if campaign == default_batch {
        campaign
    } else {
        CrashInjectionCampaign {
            image_retention: FailureImageRetention::KeepTheImageFiles,
            ..campaign
        }
    }
}

/// 一个变量都没设读回默认批；设了的每一项照变量读回、没设的那几项仍取默认批的（设了变量也读回默认批时这一条红）。线程数不比；
/// 读回的不是默认批时判红的崩溃镜像留着。
#[test]
fn the_crash_injection_campaign_is_the_default_batch_without_variables_and_follows_the_variables_that_are_set(
) {
    let default_batch = default_batch_campaign(CrashInjectionWorkerThreads::GivenByCaller(1));
    assert_eq!(
        CrashInjectionCampaign {
            worker_threads: default_batch.worker_threads,
            ..crash_injection_campaign_from(&|_variable: &str| None)
        },
        default_batch,
        "一个 SINGLEFS_CRASH_INJECTION_* 都没设，跑的该是默认批、照默认批那一组断言判、跑完删镜像"
    );
    let given = |variable: &str| {
        match variable {
            "SINGLEFS_CRASH_INJECTION_FIRST_SEED" => Some("11"),
            "SINGLEFS_CRASH_INJECTION_SEEDS" => Some("500"),
            "SINGLEFS_CRASH_INJECTION_OPERATIONS" => Some("40"),
            "SINGLEFS_CRASH_INJECTION_POINTS" => Some("8"),
            "SINGLEFS_CRASH_INJECTION_ASSERTIONS" => Some("new-findings"),
            _ => None,
        }
        .map(str::to_string)
    };
    assert_eq!(
        CrashInjectionCampaign {
            worker_threads: default_batch.worker_threads,
            ..crash_injection_campaign_from(&given)
        },
        CrashInjectionCampaign {
            first_seed: 11,
            seed_count: 500,
            operations_per_history: 40,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 8,
            },
            image_retention: FailureImageRetention::KeepTheImageFiles,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
            ..default_batch
        },
        "设了的种子起点、种子数、步数、崩溃点数、断言组照变量读回，没设的比重与跑法取默认批的，镜像留着"
    );
}

/// 生成器那一路没被这一件碰过：同一个种子生成的历史与随机历史那一段逐项相同（摆崩溃状态的随机源与它岔开）。
#[test]
fn drawing_crash_points_does_not_change_the_generated_history() {
    let history = generate_history(HistorySeed(5), 18);
    let without_any = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::Sampled {
            crash_points_per_history: 0,
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    let with_five = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::Sampled {
            crash_points_per_history: 5,
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    assert_eq!(with_five.seed, HistorySeed(5));
    assert_eq!(
        history,
        generate_history(HistorySeed(5), 18),
        "生成的历史逐项相同"
    );
    // 名副其实的那一半：摆崩溃状态不改变这段历史自己怎么跑。
    assert_eq!(
        without_any.tally.histories_completed, with_five.tally.histories_completed,
        "摆了崩溃状态之后这段历史的收尾变了"
    );
    assert_eq!(
        without_any.tally.histories_ended_known_red, with_five.tally.histories_ended_known_red,
        "摆了崩溃状态之后这段历史落到的已知红变了"
    );
    assert_eq!(
        without_any.tally.most_committed_versions_in_one_history,
        with_five.tally.most_committed_versions_in_one_history,
        "摆了崩溃状态之后模型提交过的版本数变了"
    );
    // 判别力：不摆就一个崩溃状态都没有，摆了就真摆出来了；两句都不成立时上面那三条恒真。
    assert_eq!(
        without_any.tally.crash_points, 0,
        "要 0 个崩溃状态却摆出了 {}",
        without_any.tally.crash_points
    );
    assert!(
        with_five.crash_points.len() <= 5 && !with_five.crash_points.is_empty(),
        "摆出来的崩溃状态要落在 1..=5：{}",
        with_five.crash_points.len()
    );
}

/// 判红时对一批崩溃注入会打的两行重放命令（`CrashInjectionReport::render` 里新发现之后的「复现这一段：」与末尾的「重放这一整批：」；
/// 造一个假发现，不必真红），原样喂回崩溃注入那条入口（`crash_injection_campaign_from`，入口用的同一个函数）：
/// 种子区间是那一段（种子数 1）或那一整批，崩溃状态怎么摆、每段步数、比重、跑法与原来那一批逐项相同，那一段生成的历史逐步相同；
/// 「重放这一整批」照原来那一批的那一组断言判，「复现这一段」只判清单外的失败（一段的新发现与整批的结构计数无关）。
/// 不跑崩溃状态：同一组参数下摆出来的崩溃状态由种子定（`crash_points_are_reproducible`），这里只核参数一项不漏地回来了。
fn the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_campaign_entry(
    campaign: &CrashInjectionCampaign,
) {
    let finding_seed = HistorySeed(campaign.first_seed + 5);
    let report = CrashInjectionReport {
        first_seed: campaign.first_seed,
        seed_count: campaign.seed_count,
        operations_per_history: campaign.operations_per_history,
        draw: campaign.draw,
        weights: campaign.weights,
        execution: campaign.execution,
        tally: CrashInjectionTally::default(),
        known_red_hits: Vec::new(),
        new_findings: vec![CrashPointFinding {
            signature: FailureSignature::HarnessJudgement {
                judgement: "自证用的假发现",
            },
            seed: finding_seed,
            crash_point: CrashPoint {
                segment_index: 3,
                persisted_within_the_segment: vec![true, false],
                step: StepPosition::Operation(2),
                operation_kind: None,
            },
            stage: CrashStateStage::CrashImage,
            observation: FailureObservation {
                position: StepPosition::Operation(2),
                operation_kind: None,
                violations: Vec::new(),
                panic: None,
                newest_ring_root_txg: None,
                root_ring_slot_count: None,
                harness_judgement: None,
                model_disagreement: None,
                raised_floor_lands_only_on_abandoned_roots: None,
                record_check: RecordCheck::default(),
            },
            read_back: "自证不读回".to_string(),
            image_files: None,
        }],
        image_directory: None,
        image_retention: campaign.image_retention,
        assertions: campaign.assertions,
        histories_left_by_the_stop_request: 0,
    };
    let rendered = report.render();
    let replay_lines: Vec<&str> = rendered
        .lines()
        .filter(|line| line.contains("SINGLEFS_CRASH_INJECTION_FIRST_SEED="))
        .collect();
    assert_eq!(
        replay_lines.len(),
        2,
        "判红的报告里有两行重放命令（那一段、那一整批）：\n{rendered}"
    );
    let replayed_parameters = |line: &str| {
        let assignments = environment_assignments_in_line(line);
        let replayed =
            crash_injection_campaign_from(&|variable: &str| assignments.get(variable).cloned());
        (
            replayed.first_seed,
            replayed.seed_count,
            replayed.operations_per_history,
            replayed.draw,
            replayed.weights,
            replayed.execution,
            replayed.assertions,
        )
    };
    let original_batch = (
        campaign.first_seed,
        campaign.seed_count,
        campaign.operations_per_history,
        campaign.draw,
        campaign.weights,
        campaign.execution,
        campaign.assertions,
    );
    let original_finding = (
        finding_seed.0,
        1,
        campaign.operations_per_history,
        campaign.draw,
        campaign.weights,
        campaign.execution,
        CrashInjectionAssertions::NewFindingsOnly,
    );
    let replayed_finding = replayed_parameters(replay_lines[0]);
    assert_eq!(
        replayed_finding, original_finding,
        "照「复现这一段」那一行跑崩溃注入入口，跑的不是原来那一段（种子、种子数、步数、摆法、比重、跑法、照哪一组断言判）：{}",
        replay_lines[0]
    );
    assert_eq!(
        replayed_parameters(replay_lines[1]),
        original_batch,
        "照「重放这一整批」那一行跑崩溃注入入口，跑的不是原来那一批、或不照原来那一组断言判：{}",
        replay_lines[1]
    );
    assert_eq!(
        generate_history_with_weights(
            HistorySeed(replayed_finding.0),
            replayed_finding.2,
            &replayed_finding.4
        ),
        generate_history_with_weights(
            finding_seed,
            campaign.operations_per_history,
            &campaign.weights
        ),
        "那一段生成的历史逐步相同"
    );
}

/// 照默认批的重放行跑的崩溃注入入口按默认批那一组断言判：一批二次崩溃一个都没摆出来的报告（造的计数，不跑崩溃状态；
/// 攻方的改坏就是这一形：`draw_second_crash_points` 一个都不摆），判红在默认批那一条「挂载途中一个二次崩溃都没摆出来」上，
/// 不是只判新发现而判绿（代码三方 test-script-align-code-r1 判决 S2）。
// crash-case-check:not-a-crash-case 造的计数（一批二次崩溃一个都没摆出来的报告）判红在默认批断言上，不跑崩溃状态，核默认批断言的判别力
#[test]
#[should_panic(expected = "挂载途中一个二次崩溃都没摆出来")]
fn a_default_batch_replay_without_second_crashes_is_judged_red_by_the_default_batch_assertions() {
    let campaign = default_batch_campaign(CrashInjectionWorkerThreads::GivenByCaller(1));
    let every_crash_state = DEFAULT_BATCH_SEEDS;
    let tally = CrashInjectionTally {
        crash_points: every_crash_state,
        model_judgements: every_crash_state,
        checker_runs: every_crash_state,
        record_checks: every_crash_state,
        writable_mounts_after_the_crash: every_crash_state,
        checker_runs_after_the_writable_mount: every_crash_state,
        record_checks_after_the_writable_mount: every_crash_state,
        second_crash_points: 0,
        // 计数袋没有不变量，其余计数从零起（与 `crash_injection.rs` 里汇总一段历史的计数同一种写法）。
        ..CrashInjectionTally::default()
    };
    let replay_line = CrashInjectionReport {
        first_seed: campaign.first_seed,
        seed_count: campaign.seed_count,
        operations_per_history: campaign.operations_per_history,
        draw: campaign.draw,
        weights: campaign.weights,
        execution: campaign.execution,
        tally: CrashInjectionTally::default(),
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
        image_directory: None,
        image_retention: campaign.image_retention,
        assertions: campaign.assertions,
        histories_left_by_the_stop_request: 0,
    }
    .replay_line();
    let assignments = environment_assignments_in_line(&replay_line);
    let replayed =
        crash_injection_campaign_from(&|variable: &str| assignments.get(variable).cloned());
    judge_crash_injection_report(&CrashInjectionReport {
        first_seed: replayed.first_seed,
        seed_count: replayed.seed_count,
        operations_per_history: replayed.operations_per_history,
        draw: replayed.draw,
        weights: replayed.weights,
        execution: replayed.execution,
        tally,
        known_red_hits: Vec::new(),
        new_findings: Vec::new(),
        image_directory: None,
        image_retention: replayed.image_retention,
        assertions: replayed.assertions,
        histories_left_by_the_stop_request: 0,
    });
}

#[test]
fn replay_lines_of_the_default_batch_run_the_same_campaign_on_the_campaign_entry() {
    the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_campaign_entry(
        &default_batch_campaign(CrashInjectionWorkerThreads::GivenByCaller(1)),
    );
}

#[test]
fn replay_lines_of_the_rollback_sampling_run_the_same_campaign_on_the_campaign_entry() {
    the_replay_lines_printed_for_a_finding_run_the_same_campaign_on_the_campaign_entry(
        &rollback_sampling_campaign(CrashInjectionWorkerThreads::GivenByCaller(1)),
    );
}

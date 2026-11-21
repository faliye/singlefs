//! GPU 一趟与 CPU 一趟、停止文件、几份分摊与别的主机：同一小批分开判报告相同、往 GPU 那一侧塞故障两趟不同，停止文件在时一片不取，两份分摊判进一份库每张表只判一次，GPU 记录核对器判非法复用盖掉的创世单元，留给别的主机的表计进报告，一个进程里同时起的两个引擎判进同一份库。

use super::*;

/// 开 GPU 的小规模一批（两段、每段 8 步、每段 2 个崩溃状态）：这一趟自己写一份开 GPU 的多机配置、在环境守卫里跑（清掉、跑完还原一趟读的变量）。
/// 库是临时 prune-server 里以用例标签命名的一段一次性名字空间、起跑是空的（全仓那一份里判过同一批表的话，这一趟就全复用、一张表都不判，
/// 塞在 GPU 那一侧的故障碰不到任何状态）。给定状态表只交 GPU 判器判；`fault` 塞在 GPU 那一侧。
#[cfg(feature = "gpu")]
fn small_campaign_judged_on_the_gpu(
    tag: &str,
    fault: FaultPutOnTheGpuSide,
) -> (CrashInjectionReport, GivenStatesRun) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    let directory = disposable_library_of_the_campaign(&format!("on-the-gpu-{tag}"));
    let configuration = directory.join("multi-host-with-gpu.env");
    write_a_configuration_with_the_gpu_enabled(&configuration);
    let settings = settings_of_an_engine_on_the_gpu(&configuration).to_vec();
    let _the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&settings);
    let cards_of_the_caller =
        common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run();
    let judged = campaign_through_the_given_states_engine(
        &CrashInjectionCampaign {
            first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
            seed_count: 2,
            operations_per_history: 8,
            draw: CrashPointDraw::Sampled {
                crash_points_per_history: 2,
            },
            weights: GenerationWeights::BROAD,
            execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
            worker_threads: CrashInjectionWorkerThreads::GivenByCaller(2),
            image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
            assertions: CrashInjectionAssertions::NewFindingsOnly,
        },
        fault,
    );
    drop(cards_of_the_caller);
    drop(environment);
    judged
}

/// 同一小批在 CPU 上经引擎判（这一趟自己写一份 `ENABLE_GPU=0` 的多机配置，库同 [`small_campaign_judged_on_the_gpu`]：用例标签命名的一次性库、起跑是空的）：
/// GPU 那一趟的对拍对象，两趟各判各的、各写各的库。
#[cfg(feature = "gpu")]
fn small_campaign_judged_on_the_cpu(tag: &str) -> (CrashInjectionReport, GivenStatesRun) {
    let (directory, configuration) =
        disposable_library_with_a_configuration_on_the_cpu(&format!("on-the-cpu-{tag}"));
    campaign_through_the_engine_with(
        &small_campaign_judged_on_the_cpu_through_the_engine(),
        &directory,
        &[("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str())],
    )
}

/// 验收第 1 条（第一截与第三截都上 GPU；GPU 与 CPU 的对拍是独立的两趟，一段里 CPU 不既跑数据又对拍）：
/// 同一小批一趟配了 GPU、第一截与第三截的给定状态表由 GPU 判器判，一趟没开 GPU、由 CPU 判器判，各写各的库；两趟的崩溃注入报告整份
/// 逐样相同（摆出的状态数、已知红、新发现，逐条不变量的判绿与没判到次数连同 I-1.13：GPU 扫描重建那一截补上之后内核不置位的表是空的，C821；
/// 判红的镜像只比落没落盘、不比路径，
/// `CrashInjectionReport::has_the_same_conclusions_as`），两趟都没有判不了的。要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
#[cfg(feature = "gpu")]
#[test]
fn a_small_campaign_judged_in_a_gpu_run_and_in_a_cpu_run_reports_the_same() {
    use singlefs_verification::crash_amplification::JudgedBy;
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    if gpu_card_pool_or_report_not_run(
        "a_small_campaign_judged_in_a_gpu_run_and_in_a_cpu_run_reports_the_same",
    )
    .is_none()
    {
        return;
    }
    let (gpu_report, gpu_run) =
        small_campaign_judged_on_the_gpu("agrees", FaultPutOnTheGpuSide::NoFault);
    let (cpu_report, cpu_run) = small_campaign_judged_on_the_cpu("agrees");
    assert_eq!(
        gpu_run.judged_by,
        JudgedBy::GpuJudge,
        "配了 GPU 的那一趟：GPU 判器判"
    );
    assert_eq!(
        cpu_run.judged_by,
        JudgedBy::CpuJudge,
        "没开 GPU 的那一趟：CPU 判器判"
    );
    assert!(
        gpu_report.tally.crash_points > 0 && gpu_report.tally.second_crash_points > 0,
        "两截都摆出了状态"
    );
    assert!(
        gpu_run.tally.states_judged_by_the_gpu_here > 0 && gpu_run.tally.states_reused == 0,
        "GPU 那一趟从空库起、每个状态都是这一趟在卡上判的，不是复用的：{:?}",
        gpu_run.tally
    );
    assert!(
        cpu_run.tally.states_judged_by_the_cpu_here > 0 && cpu_run.tally.states_reused == 0,
        "CPU 那一趟从空库起、每个状态都是这一趟判的，不是复用的：{:?}",
        cpu_run.tally
    );
    assert_eq!(
        gpu_run.red_items(),
        Vec::<String>::new(),
        "GPU 那一趟都判得了"
    );
    assert_eq!(cpu_run.red_items(), Vec::<String>::new());
    assert!(
        gpu_report.has_the_same_conclusions_as(&cpu_report),
        "GPU 那一趟与 CPU 那一趟的报告整份逐样相同（连同 I-1.13；镜像只比落没落盘、不比路径：两趟的镜像目录各是自己新建的那一份）：\nGPU 那一趟：\n{}\nCPU 那一趟：\n{}",
        gpu_report.render_the_conclusions(),
        cpu_report.render_the_conclusions()
    );
}

/// 上一条的判别力：往 GPU 那一侧塞一个已知故障（每张表第一个状态的第一条不变量违例位翻过来），GPU 那一趟的报告与 CPU 那一趟不同。
/// 两份报告同上一条一样整份比：两趟不塞故障时整份相同（上一条），塞了这条不绿，证得出上一条的比较抓得到 GPU 那一侧的错。
#[cfg(feature = "gpu")]
#[test]
fn a_fault_put_on_the_gpu_side_of_a_small_campaign_makes_the_gpu_run_differ_from_the_cpu_run() {
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    if gpu_card_pool_or_report_not_run(
        "a_fault_put_on_the_gpu_side_of_a_small_campaign_makes_the_gpu_run_differ_from_the_cpu_run",
    )
    .is_none()
    {
        return;
    }
    let (gpu_report, _gpu_run) = small_campaign_judged_on_the_gpu(
        "fault",
        FaultPutOnTheGpuSide::FirstInvariantViolatedBitFlippedOnTheFirstStateOfEachTable,
    );
    let (cpu_report, _cpu_run) = small_campaign_judged_on_the_cpu("fault");
    assert!(
        !gpu_report.has_the_same_conclusions_as(&cpu_report),
        "GPU 那一侧塞了故障，两趟的报告却整份相同（不比镜像路径：路径不同不算不同）：\n{}",
        gpu_report.render_the_conclusions()
    );
}

/// 走引擎、照本机配置在 CPU 上判（这一趟自己写一份 `ENABLE_GPU=0` 的多机配置）的小规模一批：两段、每段 8 步、每段 2 个崩溃状态，两条线程。
fn small_campaign_judged_on_the_cpu_through_the_engine() -> CrashInjectionCampaign {
    CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 2,
        operations_per_history: 8,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(2),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    }
}

/// 在 `library` 那一份一次性库上（设成这个进程开的那一份）、环境守卫里（清掉一趟读的变量、设 `settings`、跑完还原）经引擎跑一批。
fn campaign_through_the_engine_with(
    campaign: &CrashInjectionCampaign,
    library: &singlefs_verification::crash_library::DisposableLibrary,
    settings: &[(&str, &std::ffi::OsStr)],
) -> (CrashInjectionReport, GivenStatesRun) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    let _the_library_of_this_process = library.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(settings);
    let judged = campaign_through_the_given_states_engine(campaign, FaultPutOnTheGpuSide::NoFault);
    drop(environment);
    judged
}

/// 叫停（给定状态引擎补的第 ② 样，照流水线那一侧的叫停层）：叫停文件在，崩溃注入一片都不领，报告记两段历史都没领、引擎那一趟报叫停了；
/// 叫停文件删掉再跑同一批，两段都领了、没叫停。
/// 判别力：崩溃注入领片之前不问叫停（`crates/mutations.tsv` 那一行），叫停文件在也两段都跑了。
#[test]
fn a_campaign_whose_stop_file_is_there_takes_no_slice_and_reports_every_history_left() {
    let campaign = small_campaign_judged_on_the_cpu_through_the_engine();
    let (directory, configuration) = disposable_library_with_a_configuration_on_the_cpu("stop");
    let stop_file = directory.join("stop");
    std::fs::write(&stop_file, "").expect("写得了叫停文件");
    let settings = [
        ("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str()),
        (
            "SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE",
            stop_file.as_os_str(),
        ),
    ];
    let (stopped_report, stopped_run) =
        campaign_through_the_engine_with(&campaign, &directory, &settings);
    assert_eq!(
        stopped_run.library_name,
        directory
            .open_store(
                singlefs_verification::verdict_store::PruneServerFailureHandling::ReturnTheError
            )
            .expect("开得了一次性库")
            .library_name(),
        "引擎开的是这条用例的一次性库、不碰全仓那一份"
    );
    assert!(stopped_run.stopped_early, "叫停文件在：引擎那一趟报叫停了");
    assert_eq!(
        (
            stopped_report.histories_left_by_the_stop_request,
            stopped_report.tally.histories
        ),
        (campaign.seed_count, 0),
        "叫停文件在：一片都不领，两段历史都没领"
    );
    std::fs::remove_file(&stop_file).expect("删得了叫停文件");
    let (whole_report, whole_run) =
        campaign_through_the_engine_with(&campaign, &directory, &settings);
    assert!(!whole_run.stopped_early, "叫停文件不在：没叫停");
    assert_eq!(
        (
            whole_report.histories_left_by_the_stop_request,
            whole_report.tally.histories
        ),
        (0, campaign.seed_count),
        "叫停文件不在：两段都领了"
    );
    drop(directory);
}

/// 两台分片（给定状态引擎补的第 ① 样，H5「配了双机就两台分片」，照流水线那一侧的双机做法）：同一批先整份在一台上判一遍；
/// 再分两份各在一份空库上判（`SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2`、`1/2`），两份判的表加起来恰好是整份判的那么多，每一份判的加留给另一台的
/// 也是那么多；最后两份先后判进同一份库（两台都直接写服务那一份），第 0 份再跑一趟：一张都不现判、一张都不留，报告与整份那一趟逐字相同。
/// 判别力：每张表都归这台机器判（`crates/mutations.tsv` 那一行），两份各判整份，加起来是两倍。
#[test]
fn two_shares_judged_into_one_library_judge_every_table_once_and_render_the_report_of_one_host() {
    let campaign = small_campaign_judged_on_the_cpu_through_the_engine();
    let (_directory, configuration) = disposable_library_with_a_configuration_on_the_cpu("shares");
    let run_on = |library: &singlefs_verification::crash_library::DisposableLibrary,
                  share: Option<&str>| {
        let mut settings: Vec<(&str, &std::ffi::OsStr)> =
            vec![("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str())];
        if let Some(share) = share {
            settings.push((
                "SINGLEFS_CRASH_AMPLIFICATION_SHARE",
                std::ffi::OsStr::new(share),
            ));
        }
        campaign_through_the_engine_with(&campaign, library, &settings)
    };
    let (whole_report, whole_run) =
        run_on(&disposable_library_of_the_campaign("shares-whole"), None);
    // 整份：现判的加同一趟里读回的（同一批里内容相同的表，第二张照库里的复用）
    let every_table = whole_run.tally.tables_judged_here + whole_run.tally.tables_reused;
    assert!(
        whole_run.tally.tables_judged_here > 0,
        "前提：整份那一趟判了表"
    );
    let (_, first_share) = run_on(&disposable_library_of_the_campaign("shares-0"), Some("0/2"));
    let (_, second_share) = run_on(&disposable_library_of_the_campaign("shares-1"), Some("1/2"));
    assert_eq!(
        first_share.tally.tables_judged_here
            + first_share.tally.tables_reused
            + second_share.tally.tables_judged_here
            + second_share.tally.tables_reused,
        every_table,
        "两份判的（连同各自趟里读回的）加起来恰好是整份的：一张表只归一台"
    );
    for run in [&first_share, &second_share] {
        assert_eq!(
            run.tally.tables_judged_here
                + run.tally.tables_reused
                + run.tally.tables_left_to_the_other_hosts,
            every_table,
            "每一份判的加留给另一台的是整份的（分片 {}/{}）",
            run.share.index,
            run.share.count
        );
    }
    let one_library = disposable_library_of_the_campaign("shares-into-one-library");
    let _ = run_on(&one_library, Some("0/2"));
    let _ = run_on(&one_library, Some("1/2"));
    let (report_after_both_shares, run_after_both_shares) = run_on(&one_library, Some("0/2"));
    assert_eq!(
        (
            run_after_both_shares.tally.tables_judged_here,
            run_after_both_shares.tally.tables_left_to_the_other_hosts,
            run_after_both_shares.tally.tables_reused
        ),
        (0, 0, every_table),
        "两份都判进同一份库之后：一张都不现判、一张都不留，全照库里的"
    );
    assert_eq!(
        report_after_both_shares.render_the_conclusions(),
        whole_report.render_the_conclusions(),
        "两份都判进同一份库之后那一趟的报告与整份在一台上判的逐字相同"
    );
}

/// 创世发布上 GPU（先查 GPU 能不能核，不能就 CPU 核一遍；GPU 核得到）：一段历史跑完、每个写都落了的那一态，
/// 与同一张写表末尾再种两个盖掉创世树表单元两份副本的单元写、也都落了的那一态，交配了 GPU 的引擎判（只由 GPU 判器判）。
/// GPU 判器的记录核对器在种了的那一态上判「恢复自称的状态缺了一次发布的单元」，没种的那一态不红；两态 GPU 都判得了。
/// 要本机一张 NVIDIA 独显：一张适配器都没有时报「本次未跑」。
/// 判别力：给定状态表的写表把 mkfs 的写留在表外，创世发布不进 GPU 的发布表，种了也不红（`crates/mutations.tsv` 那一行）。
#[cfg(feature = "gpu")]
#[test]
fn a_genesis_unit_overwritten_by_an_illegal_reuse_is_red_on_the_gpu_record_checker() {
    use singlefs_verification::crash_amplification::{CrashAmplificationEnvironment, JudgedBy};
    use singlefs_verification::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
    use singlefs_verification::memory_pool::WrittenContents;
    const TEST_NAME: &str =
        "a_genesis_unit_overwritten_by_an_illegal_reuse_is_red_on_the_gpu_record_checker";
    if gpu_card_pool_or_report_not_run(TEST_NAME).is_none() {
        return;
    }
    let history = history_of_a_first_file_and_two_overwrites();
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "前提：这段历史跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        run.operations_written_by_make_filesystem,
        execution,
    );
    // 创世树表单元的两份副本：mkfs 单元写里偏移最大的那一组（实例表在单元区起点、树表紧跟它）
    let genesis_units: Vec<&RetainedWrite> = history_after_the_make_filesystem
        .writes_of_the_make_filesystem()
        .iter()
        .filter(|write| write.kind == StepKind::UnitWrite)
        .collect();
    let tree_table_offset = genesis_units
        .iter()
        .map(|write| write.offset)
        .max()
        .expect("mkfs 写了实例表与树表两个单元");
    let overwrites: Vec<RetainedWrite> = genesis_units
        .iter()
        .filter(|write| write.offset == tree_table_offset)
        .map(|copy| RetainedWrite {
            device: copy.device,
            kind: StepKind::UnitWrite,
            is_force_unit_access: false,
            offset: copy.offset,
            contents: WrittenContents::Bytes(vec![
                0xA5;
                usize::try_from(copy.length_in_bytes())
                    .expect("单元长装得进 usize")
            ]),
        })
        .collect();
    assert_eq!(overwrites.len(), 2, "前提：创世树表单元两块盘各一份副本");
    let healthy_write_count = history_after_the_make_filesystem.writes().len();
    let table_writes: Vec<RetainedWrite> = history_after_the_make_filesystem
        .writes()
        .iter()
        .chain(&overwrites)
        .cloned()
        .collect();
    let mut table_segments = history_after_the_make_filesystem.segments().to_vec();
    table_segments.push((healthy_write_count..table_writes.len()).collect());
    let mut healthy = vec![true; table_writes.len()];
    healthy[healthy_write_count..].fill(false);
    let planted = vec![true; table_writes.len()];
    let table = GivenStatesOfATable {
        table_name: "first-file-and-two-overwrites/genesis-tree-table-overwritten".to_string(),
        base: history_after_the_make_filesystem.base_pool(),
        writes: &table_writes,
        segments: &table_segments,
        seams: &[],
        persisted_sets: vec![healthy, planted],
    };
    // 库是这条用例自己在进程里起的临时 prune-server 里的一段一次性名字空间（设成这个进程开的那一份）：不连真服务
    let directory = disposable_library_of_the_campaign("genesis-publish-on-the-gpu");
    let configuration = directory.join("multi-host-with-gpu.env");
    write_a_configuration_with_the_gpu_enabled(&configuration);
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(
        &settings_of_an_engine_on_the_gpu(&configuration),
    );
    let cards_of_the_caller =
        common_gpu_cards_of_the_caller::cards_of_the_caller_handed_to_the_run();
    let engine = GivenStatesEngine::start_from_environment(
        GivenStatesItem {
            item_name: format!(
                "{}::history_of_a_first_file_and_two_overwrites genesis-tree-table-overwritten",
                CRASH_POINT_REPLAY_ITEM
            ),
        },
        &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
    )
    .expect("配了 GPU 的引擎起得来（卡用不上在这里红，不退回 CPU）");
    let conclusions = engine
        .judge_the_given_states(&table)
        .expect("配了 GPU 的引擎判得了这张表");
    let engine_run = engine.finish();
    drop(cards_of_the_caller);
    drop(environment);
    drop(the_library_of_this_process);
    drop(directory);
    assert_eq!(
        engine_run.judged_by,
        JudgedBy::GpuJudge,
        "配了 GPU：GPU 判器判"
    );
    assert_eq!(
        engine_run.red_items(),
        Vec::<String>::new(),
        "两态 GPU 都判得了"
    );
    assert_eq!(
        conclusions.states[0].record_check(),
        RecordCheck::default(),
        "没种的那一态：GPU 的记录核对器不红"
    );
    assert!(
        conclusions.states[1]
            .record_check()
            .claimed_state_missing_unit,
        "种了的那一态：GPU 的记录核对器判创世发布的单元缺了"
    );
}

/// 给定状态表的替身判器：照引擎判（表与结论进给定状态表那一类的判定库），每张表的结论都标成「两台分片里归别的机器判、交的占位结论」，数它判了几张表。
struct EveryTableLeftToAnotherHost {
    tables: std::sync::atomic::AtomicU64,
    engine: GivenStatesEngine,
}

impl EveryTableLeftToAnotherHost {
    fn judging_with(engine: GivenStatesEngine) -> Self {
        Self {
            tables: std::sync::atomic::AtomicU64::new(0),
            engine,
        }
    }
}

impl GivenStatesJudge for EveryTableLeftToAnotherHost {
    fn stop_is_requested(&self) -> bool {
        false
    }

    fn judge_the_given_states(
        &self,
        table: &GivenStatesOfATable<'_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        self.tables
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut conclusions = self.engine.judge_the_given_states(table)?;
        conclusions.source = GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost;
        Ok(conclusions)
    }

    /// 带两栏的表（第二截那张）同样照引擎判、标成占位的：引擎照两栏判，不走没重写的那一份。
    fn judge_the_given_states_with_its_columns(
        &self,
        table_with_its_columns: GivenStatesOfATableWithItsColumns<'_, '_>,
    ) -> Result<GivenStatesConclusionsOfATable, String> {
        self.tables
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut conclusions = self
            .engine
            .judge_the_given_states_with_its_columns(table_with_its_columns)?;
        conclusions.source = GivenStatesConclusionsSource::PlaceholderOfATableLeftToAnotherHost;
        Ok(conclusions)
    }
}

/// 占位结论在报告里标出来（E4 报告第三节第 5 条）：判器交了占位结论的表（两台分片、归别的机器判、库里又没有），崩溃注入数进报告的计数，
/// `render()` 多一行注明「这一批不作数」；就地 CPU 判的同一批一张不记、没有那一行。不要特性：判器是替身（照 CPU 就地判、每张都标成占位的）。
/// 两个引擎都在环境守卫里照一份单机 CPU 配置起，不跟调用方的分片走：崩溃点重放入口两台分片跑时调用方给了 `SINGLEFS_CRASH_AMPLIFICATION_SHARE`，
/// 照它起的引擎把归别的机器判的表也交成占位结论，就地那一趟就不是「一张都不是占位的」。
/// 判别力：崩溃注入不数占位结论的表（`crates/mutations.tsv` 那一行），记 0、`render()` 没那一行。
#[test]
fn tables_left_to_another_host_with_placeholder_conclusions_are_counted_in_the_report_and_noted_in_its_render(
) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    let (directory, configuration) =
        disposable_library_with_a_configuration_on_the_cpu("placeholder-note");
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let engine_judging_here =
        given_states_engine_under_the_callers_environment_guard("placeholder_note_judged_here");
    let judge = EveryTableLeftToAnotherHost::judging_with(
        given_states_engine_under_the_callers_environment_guard("placeholder_probe"),
    );
    drop(environment);
    drop(the_library_of_this_process);
    let campaign = CrashInjectionCampaign {
        first_seed: SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        seed_count: 2,
        operations_per_history: 8,
        draw: CrashPointDraw::Sampled {
            crash_points_per_history: 2,
        },
        weights: GenerationWeights::BROAD,
        execution: UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        worker_threads: CrashInjectionWorkerThreads::GivenByCaller(2),
        image_retention: FailureImageRetention::DeleteTheImageFilesWhenTheRunFinishes,
        assertions: CrashInjectionAssertions::NewFindingsOnly,
    };
    let judged_here =
        run_crash_injection_campaign_judging_the_given_states_with(&campaign, &engine_judging_here);
    assert_eq!(
        judged_here.tally.given_states_tables_left_to_other_hosts, 0,
        "就地 CPU 判的：一张都不是占位的"
    );
    assert!(
        !judged_here
            .render()
            .contains(GIVEN_STATES_TABLES_LEFT_TO_OTHER_HOSTS_NOTE),
        "就地 CPU 判的：render 里没有占位那一行"
    );
    let left = run_crash_injection_campaign_judging_the_given_states_with(&campaign, &judge);
    let tables = judge.tables.load(std::sync::atomic::Ordering::Relaxed);
    assert!(tables > 0, "前提：这一批建了给定状态表");
    assert_eq!(
        left.tally.given_states_tables_left_to_other_hosts, tables,
        "判器交了占位结论的每一张表都数进报告（第一截与第三截都算）"
    );
    assert!(
        left.render().contains(&format!(
            "{GIVEN_STATES_TABLES_LEFT_TO_OTHER_HOSTS_NOTE}：{tables} 张"
        )),
        "render 注明有几张表交的是占位结论：{}",
        left.render()
    );
    let library_of_the_engine_judging_here = engine_judging_here.finish().library_name;
    assert_eq!(
        library_of_the_engine_judging_here,
        directory
            .open_store(
                singlefs_verification::verdict_store::PruneServerFailureHandling::ReturnTheError
            )
            .expect("开得了一次性库")
            .library_name(),
        "就地判的那个引擎开的是这条用例的一次性库、不碰全仓那一份"
    );
}

/// 同一个进程里同时起两个引擎（两条用例并行跑撞同一份库）：两个都开这个进程的那一份（同一份一次性库，共用这个进程里开着的那一份连接），
/// 两个都起得来，先判的那个把库里还没有的表现判、后判的那个同一批全照库里的复用，两份报告相同。CPU 配置，不要卡。
/// 判别力：存表的结论不写进服务、只记在先判那个引擎那一端（后判的那个读不到），这条红在复用那一句。
#[test]
fn two_engines_started_at_once_in_one_process_both_judge_on_the_one_library_of_the_process() {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    let campaign = small_campaign_judged_on_the_cpu_through_the_engine();
    let (directory, configuration) =
        disposable_library_with_a_configuration_on_the_cpu("two-engines");
    let start = || {
        GivenStatesEngine::start_from_environment(
            GivenStatesItem {
                item_name: format!(
                    "{}::{}",
                    CRASH_POINT_REPLAY_ITEM,
                    seeds_and_scale_of(&campaign)
                ),
            },
            &CrashPointReplayToolchainIdentity::of_the_cargo_running_this_test(),
        )
    };
    let the_library_of_this_process = directory.make_it_the_library_of_this_process();
    let environment = CrashAmplificationEnvironment::cleared_then_set(&[(
        "SINGLEFS_MULTI_HOST_CONFIG",
        configuration.as_os_str(),
    )]);
    let first = start().unwrap_or_else(|refusal| panic!("第一个引擎起得来：{refusal}"));
    let second = start().unwrap_or_else(|refusal| {
        panic!("第一个还开着时第二个也起得来（共用这个进程里开着的那一份库）：{refusal}")
    });
    let first_report =
        run_crash_injection_campaign_judging_the_given_states_with(&campaign, &first);
    let second_report =
        run_crash_injection_campaign_judging_the_given_states_with(&campaign, &second);
    let first_run = first.finish();
    let second_run = second.finish();
    drop(environment);
    drop(the_library_of_this_process);
    assert_eq!(
        first_run.library_name, second_run.library_name,
        "两个引擎开的是同一份库"
    );
    let every_table = first_run.tally.tables_judged_here + first_run.tally.tables_reused;
    assert!(every_table > 0, "前提：这一批有表");
    assert_eq!(
        (
            second_run.tally.tables_judged_here,
            second_run.tally.tables_reused
        ),
        (0, every_table),
        "后判的那个全照同一份库里的复用"
    );
    assert_eq!(
        first_report.render_the_conclusions(),
        second_report.render_the_conclusions(),
        "复用的那一趟报告与现判的相同"
    );
}

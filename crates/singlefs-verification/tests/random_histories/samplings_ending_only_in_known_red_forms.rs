//! 默认批与复用、回退、分配记录墙、单元区墙几段取样：每段只以已知红的形态收尾，每种操作与路径都走到。

use super::*;

fn count_of(counts: &std::collections::BTreeMap<String, u64>, key: &str) -> u64 {
    counts.get(key).copied().unwrap_or(0)
}

/// 各条路径都真的跑到了：崩溃恢复抛弃根之外的每类操作至少一次 Ok、崩溃恢复抛弃根至少被乙拒过一次，回退的目标候选集里外都试过，
/// 抬 F 撞过上限也回收过落点，复用改写过已释放的记录（含跨度变了的与被删的），六种内容长度都进过入口，冷启动读回过文件，
/// checker 真的判过 I-3.1 与 I-5.4，至少一段历史转过根环。
fn assert_every_path_was_exercised(tally: &HistoryTally) {
    for kind in HistoryOperationKind::ALL {
        let operation = tally
            .operations_by_kind
            .get(&kind)
            .copied()
            .unwrap_or_default();
        match kind {
            // 那一步藏的是这个会话发的、系统配置见证过的最新那条根：C554 乙之后读阶段重读一次仍读不出、取号之前拒可写
            //，这一步跑到了是「拒过」，不再有做成的。
            HistoryOperationKind::CrashRecoveryAbandoningTheNewestRoot => {
                assert!(operation.refused >= 1, "{kind:?} 一次都没被拒过");
            }
            // broad 那组比重不抽它（种子钉着的取样点照旧）：它只在留孤儿那一组比重里抽，跑到与留下孤儿由 `singlefs_verification::history`
            // 的单元测试钉（`the_orphan_sampling_draws_the_overwrite_losing_power_and_the_broad_sampling_never_does` 与
            // `an_overwrite_losing_power_before_its_journal_record_leaves_orphans_the_sweep_erases_once_its_instance_left_the_ring`）。
            HistoryOperationKind::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable => {
                assert_eq!(
                    operation,
                    singlefs_verification::history::OperationTally::default(),
                    "{kind:?} 不在 broad 那组比重里，却跑到了"
                );
            }
            // 只藏第一块盘的那一种与每块盘都藏的那一种只进它们自己那一组比重
            // （`GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS`，C696），broad 那一组不抽；跑到了没有、
            // 每块盘都藏的那一种照乙入判不出拒过没有，在那一组的取样点那条用例里核
            // （`newest_system_configuration_slot_zeros_sampling_mounts_writable_and_ends_only_in_known_red_forms`）。
            HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros
            | HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "broad 那一组比重不抽 {kind:?}"
                );
            }
            // 建快照、读快照、删快照只进它们自己那一组比重（`GenerationWeights::SNAPSHOT_OPERATIONS`），broad 那一组不抽；跑到了没有在那一组的
            // 取样点那条用例里核（`snapshot_operations_in_random_histories.rs`）。
            HistoryOperationKind::CreateSnapshot
            | HistoryOperationKind::ReadSnapshot
            | HistoryOperationKind::DestroySnapshot => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "broad 那一组比重不抽 {kind:?}"
                );
            }
            // 挂着时整理只进它自己那一组比重（`GenerationWeights::COMPACTION_WHILE_MOUNTED`），broad 那一组不抽；跑到了没有在那一组的
            // 取样点那条用例里核（`compaction_while_mounted_in_random_histories.rs`）。
            HistoryOperationKind::CompactWhileMounted => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "broad 那一组比重不抽 {kind:?}"
                );
            }
            // 截断只进它自己那一组比重（`GenerationWeights::TRUNCATION`）与带整理的那一组，broad 那一组不抽；跑到了没有在那一组的
            // 取样点那条用例里核（`truncation_random_histories_against_the_model.rs`）。
            HistoryOperationKind::Truncate => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "broad 那一组比重不抽 {kind:?}"
                );
            }
            // 一次建几个 inode、按 inode 号写只进它们自己那一组比重（`GenerationWeights::WRITES_BY_INODE_NUMBER`），broad 那一组不抽；跑到了没有在那一组的
            // 取样点那条用例里核（`writes_by_inode_number_in_random_histories_judged_by_the_model.rs`）。
            HistoryOperationKind::CreateInodes | HistoryOperationKind::WriteOfAnInode => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "broad 那一组比重不抽 {kind:?}"
                );
            }
            // 按偏移写只进它自己那一组比重（`GenerationWeights::PARTIAL_WRITES`），broad 那一组不抽；跑到了没有在那一组的
            // 取样点那条用例里核（`partial_write_against_the_model.rs`）。
            HistoryOperationKind::WriteAtOffset => {
                assert_eq!(
                    (operation.applied, operation.refused, operation.not_applicable),
                    (0, 0, 0),
                    "broad 那一组比重不抽 {kind:?}"
                );
            }
            HistoryOperationKind::PublishFirstFile
            | HistoryOperationKind::PublishOverwrite
            | HistoryOperationKind::PublishWithoutUnits
            | HistoryOperationKind::CloseAndMountWritable
            | HistoryOperationKind::RollBackWhileMounted
            | HistoryOperationKind::RaiseRollbackFloor
            | HistoryOperationKind::ColdStartRecover
            | HistoryOperationKind::Unlink
            | HistoryOperationKind::CreateInode => {
                assert!(operation.applied >= 1, "{kind:?} 一次 Ok 都没有");
            }
        }
    }
    let refusals = &tally.refusals_by_member;
    for member in [
        "MountError::RollbackFloorAboveCeiling",
        "RollbackError::TargetNotACandidate(NotInRing)",
        "PublishError::ContentExceedsDataUnit",
        "PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile",
        "MountError::NewerStateStillUnreadableAfterOneReread(PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion)",
    ] {
        assert!(count_of(refusals, member) >= 1, "没见过 {member}");
    }
    assert!(
        count_of(
            refusals,
            "RollbackError::TargetNotACandidate(BelowEffectiveFloor)"
        ) >= 1,
        "回退的目标没落到过 F 之下的根"
    );
    // 回退的目标落到被抛弃的根上、按被抛弃的时间线拒，这一路随机历史不再要求跑到：被抛弃的根只由崩溃恢复造出
    // （D23（journal 的角色与格式） 已定项 14），生成器那一步 C554 乙之后一律被拒、造不出它；乙罩不到的「系统配置没见证到最新那条根」
    // 那一形由写死的用例 `rolling_back_to_a_root_crash_recovery_abandoned_without_a_system_configuration_witness_is_refused_on_the_abandoned_timeline` 钉。
    assert!(tally.raises_that_reclaimed >= 1, "抬 F 一次都没回收到落点");
    assert!(
        tally.records_rewritten_from_released >= 1,
        "一条已释放的记录都没被复用"
    );
    assert!(
        tally.records_rewritten_with_changed_span >= 1,
        "复用时跨度一次都没变过（Z1-d 那条路没跑到）"
    );
    assert!(
        tally.released_records_removed >= 1,
        "复用时一条被罩住的已释放记录都没删过"
    );
    for length in [
        ContentLength::Empty,
        ContentLength::InsideOneDataUnit { selector: 0 },
        ContentLength::OneByteBelowDataUnitPayloadCapacity,
        ContentLength::ExactlyDataUnitPayloadCapacity,
        ContentLength::OneByteAboveDataUnitPayloadCapacity,
        ContentLength::ExactlyDataUnitBytes,
    ] {
        assert!(
            tally
                .content_lengths_attempted
                .get(length.name())
                .copied()
                .unwrap_or(0)
                >= 1,
            "内容长度「{}」没进过入口",
            length.name()
        );
    }
    // 装得下的四种长度各至少发成一次：只看「进过入口」的话，全被拒也算进过（B1 / P3）。
    for length in [
        ContentLength::Empty,
        ContentLength::InsideOneDataUnit { selector: 0 },
        ContentLength::OneByteBelowDataUnitPayloadCapacity,
        ContentLength::ExactlyDataUnitPayloadCapacity,
    ] {
        assert!(
            tally
                .content_lengths_published
                .get(length.name())
                .copied()
                .unwrap_or(0)
                >= 1,
            "装得下的内容长度「{}」一次都没发成",
            length.name()
        );
    }
    assert!(
        count_of(&tally.recovery_outcomes, "FileRead") >= 1,
        "冷启动一次都没读回文件"
    );
    // 池级 checker 那一遍归给定状态引擎两段：harness 一条不变量都不判，每一步的镜像经钩子交出去。
    // I-3.1、I-5.4、I-3.11、I-7.9、I-9.15 在同一批历史上真被判成立过，由给定状态那一份用例核
    // （`crates/singlefs-verification/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs`）。
    assert_the_harness_ran_no_pool_checker_and_handed_every_step_image_out(tally);
    assert!(
        tally.histories_that_turned_the_root_ring >= 1,
        "没有一段历史转过根环"
    );
    assert_the_model_judged_every_kind_of_answer(tally);
}

/// harness 不逐步跑池级 checker（那一遍归给定状态引擎两段），每一步的镜像都交出去了：checker 一次都没跑，交出去的镜像至少一张。
fn assert_the_harness_ran_no_pool_checker_and_handed_every_step_image_out(tally: &HistoryTally) {
    assert_eq!(
        tally.checker_runs, 0,
        "harness 跑了池级 checker：每一步的池级 checker 归给定状态引擎两段，harness 只与理想模型比"
    );
    assert!(
        tally.step_images_handed_out >= tally.histories,
        "每段历史起点之后至少交出一张镜像：交了 {} 张、{} 段",
        tally.step_images_handed_out,
        tally.histories
    );
}

/// 每步镜像那个钩子（给定状态引擎两段的录入段照它把每一步的镜像作为给定状态写进判定库）：broad 那组比重、种子基那一段 12 步，
/// harness 一次池级 checker 都不跑；起点之后与每一步写过盘之后各交一张，交出去的张数与计数相同、录制流里的步数逐张变多，
/// 每一张都等于空盘叠上录制流头那么多步（给定状态引擎按录制流重建给定状态，判的就是这一张）。
/// 判别力：harness 偷回逐步池级 checker（执行器里每张交出去的镜像又就地判一遍、记进计数），checker 跑过的次数不再是 0；
/// 钩子收到了却不交出去，交出去的张数与计数对不上。
// harness-test-granularity:one-scenario 一段历史跑一遍，循环里逐张重建的是同一条录制流的前缀、拿来比这一段交出去的那几张，不是几个独立场景
#[test]
fn every_step_image_is_handed_out_as_the_recorded_stream_prefix_and_the_harness_runs_no_pool_checker(
) {
    let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
    let history = generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE),
        12,
        &DEFAULT_BATCH_SAMPLING.weights,
    );
    let stream = SharedStream::retaining_contents();
    let mut handed_out: Vec<(usize, MemoryPool)> = Vec::new();
    let run = execute_history_handing_out_every_step_image(
        &history,
        execution,
        &stream,
        &SharedFaultPlan::unarmed(execution.device_width.fixed_geometry()),
        &mut |_| {},
        &mut |step_image| {
            handed_out.push((step_image.operations_recorded, step_image.image.clone()));
        },
    );
    assert_eq!(run.tally.checker_runs, 0, "harness 不逐步跑池级 checker");
    assert_eq!(
        run.tally.step_images_handed_out,
        u64::try_from(handed_out.len()).expect("装得进 u64"),
        "计数与交出去的张数相同"
    );
    assert!(
        handed_out.len() >= 2,
        "起点之后与至少一步写过盘之后都交了：{}",
        handed_out.len()
    );
    assert!(
        handed_out.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "每一张都是写过盘之后交的，录制流里的步数逐张变多"
    );
    let operations = stream.retained_operations();
    for (index, (operations_recorded, image)) in handed_out.iter().enumerate() {
        let mut rebuilt = MemoryPool::with_devices(
            &[DeviceIdentity(0), DeviceIdentity(1)],
            execution.device_width.device_bytes(),
        );
        rebuilt.apply(&operations[..*operations_recorded]);
        assert!(
            rebuilt == *image,
            "交出去的第 {index} 张：空盘叠上录制流头 {operations_recorded} 步与交出去的镜像不同"
        );
    }
}

/// 理想模型真的判过：该拒而拒、该成而成都有，比过根、分配记录、冷启动读回的内容、抬 F 的上限（第 2 件；阴性结果要能和「模型没跑到」分开）。
fn assert_the_model_judged_every_kind_of_answer(tally: &HistoryTally) {
    let counts = &tally.model_counts;
    assert!(tally.model_judged_steps >= 1, "模型一步都没判");
    assert!(
        counts.required_refusals_matched >= 1,
        "模型一次「该拒而拒」都没判过"
    );
    assert!(
        counts.successes_matched >= 1,
        "模型一次「该成而成」都没判过"
    );
    assert!(counts.roots_compared >= 1, "模型一条根都没比过");
    assert!(
        counts.allocation_records_compared >= 1,
        "模型一条分配记录的分配代都没比过"
    );
    assert!(
        counts.cold_start_contents_compared >= 1,
        "模型一次冷启动读回的内容都没比过"
    );
    assert!(counts.ceilings_compared >= 1, "模型一次抬 F 的上限都没比过");
}

/// 随机历史这一批：不设变量是默认批（种子基起 96 段、每段 30 步、broad 比重，每一步的镜像交出去）；设了 `SINGLEFS_RANDOM_HISTORY_*`
/// 的照变量跑（[`random_history_batch_from`]：种子数、第一个种子、收不收缩、每段步数、比重、跑法；没设的那几项取默认批的；
/// 线程数从 `SINGLEFS_RANDOM_HISTORY_THREADS` 取）。入口的 `Err` 算合法结局，panic 与违例算失败，撞到「已知红」清单里的形态
/// 照记、那段到此为止，清单外的一条都不许有（有就按签名归类、报出种子与失败在哪一步；默认批不收缩，收缩交给「收缩一个种子」那个 example
/// 或设 `SINGLEFS_RANDOM_HISTORY_SHRINK=every`，debug 下收缩一类要两分多钟）。计数照打；跑的是默认批时再核各条路径真的跑到了
/// （比重、步数换了的一批未必每条路径都抽得到，那一项只判清单外的失败）。
#[test]
fn random_histories_end_only_in_known_red_forms_and_exercise_every_operation() {
    let batch = random_history_batch_from(&process_environment_variable);
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        batch.first_seed,
        batch.seed_count,
        batch.sampling.operations_per_history,
        &batch.sampling.weights,
        batch.sampling.execution,
        worker_threads_from_environment(),
        batch.shrinking,
    );
    let rendered = report.render();
    let every_new_finding_seed: Vec<(u64, &FailureSignature)> = report
        .new_findings
        .iter()
        .flat_map(|finding| {
            finding
                .seeds
                .iter()
                .map(move |seed| (seed.0, &finding.signature))
        })
        .collect();
    print_uncaptured(&format!(
        "── 随机历史 ──\n{rendered}每个新发现的种子：{every_new_finding_seed:?}\n用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    if batch == RANDOM_HISTORY_DEFAULT_BATCH {
        assert_every_path_was_exercised(&report.tally);
    }
}

/// 第 121 行那一类（复用时新记录罩住别的已回收记录）的专门取样点：比重取 `GenerationWeights::REUSE_AFTER_RAISING_THE_FLOOR`，
/// 种子 [0, 48)、每段 30 步。在 release 下量：第 121 行的变异施加之后，这组比重在种子
/// [0, 6000) × 30 步上 799 个种子判出（13.3%），按 48 个种子一窗切 125 窗、每窗至少 2 个；门禁这一窗 [0, 48) 判出 7 个。
/// 先判清单外的失败（变异下红在这一条），再核这一路真的跑到了：罩住别的已释放记录的起点、复用时跨度变了、抬 F 回收到落点——
/// 这几个数不看罩住的那条删没删，变异下照样成立，门禁 crates-mutation-replay 看到的红只会是分类判出来的。
#[test]
fn reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        REUSE_SAMPLING_FIRST_SEED,
        REUSE_SAMPLING_SEEDS,
        REUSE_SAMPLING.operations_per_history,
        &REUSE_SAMPLING.weights,
        REUSE_SAMPLING.execution,
        worker_threads_from_environment(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：偏向抬 F 之后复用的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    let tally = &report.tally;
    assert!(
        tally.released_records_covered >= 1,
        "新分配的记录一次都没罩住别的已释放记录的起点（第 121 行那条路没跑到）"
    );
    assert!(
        tally.records_rewritten_with_changed_span >= 1,
        "复用时跨度一次都没变过"
    );
    assert!(tally.raises_that_reclaimed >= 1, "抬 F 一次都没回收到落点");
}

/// 理想模型那一格 B2（回退到 txg = F_生效 的根被拒，`research/prompts/m2-supp3-item1-code-r1-opus-model/mutants.tsv`）的专门取样点：
/// 比重取 `GenerationWeights::ROLLBACK_AFTER_RAISING_THE_FLOOR`（回退目标一半取候选集的下沿），种子 [0, 48)、每段 30 步。broad 那组比重与抽法下
/// 回退落到 F 那条根上一次都没有。先判清单外的失败（模型对不上也在其内），再核这一格真的跑到了：回退到 txg = F_生效 > 0 的根做成过——
/// 这个数只在做成时加，变异下被拒、判红的是分类，不是这条计数。
#[test]
fn rollback_heavy_random_histories_reach_the_floor_root_and_end_only_in_known_red_forms() {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        ROLLBACK_SAMPLING_FIRST_SEED,
        ROLLBACK_SAMPLING_SEEDS,
        ROLLBACK_SAMPLING.operations_per_history,
        &ROLLBACK_SAMPLING.weights,
        ROLLBACK_SAMPLING.execution,
        worker_threads_from_environment(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：偏向抬 F 之后回退的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    assert!(
        report
            .tally
            .model_counts
            .rollbacks_accepted_at_the_effective_floor
            >= 1,
        "回退到 txg = F_生效 > 0 的根一次都没做成（B2 那一格没跑到）"
    );
}

/// 崩溃注入读故障那一支的取样点（C696（崩溃注入的读故障一支在随机历史里从没触发），候选「丁」）：比重取
/// `GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS`（broad 那一组、崩溃恢复抛弃根那几格换成「只有第一块盘最新那一槽
/// 系统配置读回全 0 时的可写挂载」），种子从这个测试周期的种子基起 48 段、每段 24 步，两块 384 槽单元区的窄盘（设计调查量的那一档），
/// 每一步之后跑池级 checker、模型照判。先判清单外的失败（checker 判红、模型对不上都在其内），再核这一步真的跑到了、可写挂载做成过——
/// 做成就是取号、写行、暖机都在读故障下发了写（D22（单元原子性怎么合成） 已定项 8：单故障在承诺里，挂载照常往下走）。
/// 同一组比重里另有小份数的「每块盘最新那一槽系统配置读回全 0 时的可写挂载」（C971（多处读坏时可写挂载写出倒挂的新实例根）乙入判不出）：
/// 再核它被抽到、跑到过、每次都被拒、一次都没做成；实现照乙入判不出拒、模型照条款答拒，对不上就是清单外的失败。
/// 两条「至少一次」（只藏第一块盘的做成过、每块盘都藏的被拒过）不靠运气：种子基 `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE` 永久固定，
/// 这 48 段历史的生成与执行只看种子与比重，汇总按种子排好、与线程数无关，同一份代码跑多少次计数都相同。
/// 判别力：这一步不只藏第一块盘、每块盘最新那一槽都藏（没有一块盘两槽都自证过，乙入判不出拒可写）时可写挂载被拒、一次都做不成，这一条红
/// （`crates/mutations.tsv` 那一行点名的是崩溃注入同一步上的靶向用例，跑得快）；模型那一步照普通可写挂载答时每块盘都藏的那一步报
/// 「模型说该成、实现拒了」，清单外的失败那一条红（`crates/mutations.tsv` 一行）。
#[test]
fn newest_system_configuration_slot_zeros_sampling_mounts_writable_and_ends_only_in_known_red_forms(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE,
        48,
        24,
        &GenerationWeights::NEWEST_SYSTEM_CONFIGURATION_SLOT_READING_ZEROS,
        HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        },
        worker_threads_from_environment(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：最新那一槽系统配置读回全 0 的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "「已知红」清单外的失败：\n{rendered}"
    );
    let step = report
        .tally
        .operations_by_kind
        .get(&HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros)
        .copied()
        .unwrap_or_default();
    assert!(
        step.applied >= 1,
        "只有第一块盘最新那一槽系统配置读回全 0 的可写挂载一次都没做成：{step:?}\n{rendered}"
    );
    assert_eq!(
        step.refused, 0,
        "单故障在承诺里，这一步不许被拒：{step:?}\n{rendered}"
    );
    let step_hiding_every_device = report
        .tally
        .operations_by_kind
        .get(&HistoryOperationKind::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros)
        .copied()
        .unwrap_or_default();
    assert!(
        step_hiding_every_device.refused >= 1,
        "每块盘最新那一槽系统配置读回全 0 的可写挂载一次都没被拒过（没抽到或前提一次都不满足）：{step_hiding_every_device:?}\n{rendered}"
    );
    assert_eq!(
        step_hiding_every_device.applied, 0,
        "没有一块盘两槽都自证过，乙入判不出拒可写，这一步不许做成：{step_hiding_every_device:?}\n{rendered}"
    );
}

/// 原分配记录墙那一格（增补 3 第 2 件）的取样点：比重取 `GenerationWeights::TOWARD_THE_ALLOCATION_RECORD_WALL`，
/// 种子与步数见常量。分配记录树按绝对槽号按位置寻址之后（D8（核心索引结构） 已定项 14，K1）没有「一个节点 812 条」那道墙，
/// 模型也没有那条拒绝理由：越过 812 条之后实现因为分配记录拒一次，模型就判对不上。每一步之后照跑池级 checker、判红就停（
/// 两条只有 checker 看得见的变异——根环转过之后回收门槛多一代、分配记录过 600 条之后「已分配」少记一槽——要 checker 才看得见）。
/// 先判没有新发现（checker 的判红、模型对不上、执行器判出的、panic 都算），再核这一路真的跑到了：checker 真的跑过、有一段历史的某一版
/// 分配记录多于 812 条（多叶的树真的写过、读过、判过）。
#[test]
fn allocation_record_sampling_with_the_checker_goes_past_the_812_records_of_the_former_one_node_wall(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        WALL_SAMPLING_FIRST_SEED,
        WALL_SAMPLING_SEEDS,
        WALL_SAMPLING.operations_per_history,
        &WALL_SAMPLING.weights,
        WALL_SAMPLING.execution,
        worker_threads_from_environment(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：越过原分配记录墙的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "新发现（checker 的判红、模型、执行器的判定与 panic）：\n{rendered}"
    );
    assert!(
        report.tally.step_images_handed_out > 0,
        "这一段每一步之后跑池级 checker"
    );
    assert!(
        report.tally.most_allocation_records_in_one_version > 812,
        "没有一段历史越过 812 条分配记录（原先那道墙那一格没跑到）：最多 {} 条",
        report.tally.most_allocation_records_in_one_version
    );
}

/// 单元区墙那一格空间准入关掉的取样点（两块单元区 256 槽的小盘，`HistoryDeviceWidth::UnitAreaOf256Slots`；比重
/// `GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`，每一步之后照跑池级 checker、判红就停，种子与步数见常量；
/// 空间准入关掉，只供测试的开关 `SpaceAdmission::SkippedByTheTestOnlySwitch`）跑到底、一次都没撞上落点墙：
/// 小盘为环窗口补足之后这一段的单元区比环窗口那一截宽，覆盖写与第一个文件一次都没被「每块盘上都没有」拒过，
/// 模型也一次都没在单元区墙的区间里放行过。种子与几何都不动，
/// 钉今天量到的那几个数——32 段全跑完、没有新发现，「每块盘上都没有」0 次，单元区墙区间里放行 0 次，池级 checker 跑 1141 次；
/// 落点墙那一格只靠写死的那 5 条用例罩着（`a_floor_raise_refused_for_space_counts_as_short_of_space.rs` 两条与
/// `admission_raises_the_floor_before_refusing.rs` 里删除类发布撞落得下那一判的三条）。
/// 判别力：这一段生成的历史或落点规则变了（比重里 unlink 那一格改成 0），这几个数跟着变。
#[test]
fn unit_area_wall_sampling_on_small_devices_with_the_space_admission_skipped_runs_to_the_end_without_reaching_the_placement_wall(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        UNIT_AREA_WALL_SAMPLING_FIRST_SEED,
        UNIT_AREA_WALL_SAMPLING_SEEDS,
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_SKIPPED.operations_per_history,
        &UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_SKIPPED.weights,
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_SKIPPED.execution,
        worker_threads_from_environment(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：小盘上逼近单元区墙的取样点 ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "新发现（checker 的判红、模型、执行器的判定与 panic）：\n{rendered}"
    );
    // 第一个文件直接调发布路径；覆盖写经挂着的会话（实七）：落点拒绝是「空间不够」的一种，会话先推抬 F、推满仍拒才交回，
    // 成员包在 `UserChangeRefused::NoSpaceAfterRaisingTheFloor` 里。
    let refused_on_every_device = count_of(
        &report.tally.refusals_by_member,
        "PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)",
    ) + count_of(
        &report.tally.refusals_by_member,
        "UserChangeRefused::NoSpaceAfterRaisingTheFloor(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))",
    );
    assert_eq!(
        (
            report.tally.histories_completed,
            refused_on_every_device,
            report
                .tally
                .model_counts
                .unit_area_wall_refusals_in_the_interval,
            report.tally.step_images_handed_out,
        ),
        (32, 0, 0, 1141),
        "32 段全跑完、「每块盘上都没有」0 次、单元区墙区间里放行 0 次、池级 checker 跑 1141 次（2026-10-03 量）：\n{rendered}"
    );
}

/// 同一段取样（两块单元区 256 槽的小盘、逼近单元区墙的比重、同一批种子与步数），空间准入判着（产品路径）：
/// D28（挂载期承诺量） 已定项 1 的式子接进发布与可写挂载之后（C363 (b)），这几块小盘上墙由式子先拒。
/// 覆盖写经挂着的会话发布之后（实七），式子一拒会话先推一串抬 F 再重判（D16（发布语义） 已定项 1「准入」那一行，C283）：
/// 这一批种子上推过之后每一次都发成了（推了 470 串、发成 470 次，最后被拒的 0 次），模型逐串比过推的每一串抬 F。
/// 推满仍不够时交回的 `UserChangeRefused::NoSpaceAfterRaisingTheFloor` 在这里一次都没出现，式子那一拒模型区间里放行的那一格
/// 在这批种子上看不到了（实七报告）。可写挂载准入不够时写行与暖机之后推抬 F 的空发布、再判，执行器把推的每一串照模型的抬 F 逐串比。
/// 先判没有新发现，再核这一路真的跑到了：覆盖写经会话推过抬 F 再发成、可写挂载写行之后推过抬 F、推的每一串模型都比过上限；
/// 落点那一道一次都没走到（式子先拒）。
#[test]
fn unit_area_wall_sampling_on_small_devices_with_the_space_admission_judged_is_refused_by_the_formula_inside_the_model_interval(
) {
    let started = std::time::Instant::now();
    let report = run_history_campaign(
        UNIT_AREA_WALL_SAMPLING_FIRST_SEED,
        UNIT_AREA_WALL_SAMPLING_SEEDS,
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED.operations_per_history,
        &UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED.weights,
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED.execution,
        worker_threads_from_environment(),
        FindingShrinking::ReportSeedsOnly,
    );
    let rendered = report.render();
    print_uncaptured(&format!(
        "── 随机历史：小盘上逼近单元区墙的取样点（空间准入判着） ──\n{rendered}用时 {:.1} 秒\n",
        started.elapsed().as_secs_f64()
    ));
    assert!(
        report.new_findings.is_empty(),
        "新发现（checker 的判红、模型、执行器的判定与 panic）：\n{rendered}"
    );
    assert!(
        report
            .tally
            .user_changes_published_after_the_session_raised_the_floor
            >= 1,
        "覆盖写一次都没经会话推过抬 F 再发成（发布路径那一处的推没跑到）"
    );
    assert!(
        report.tally.model_counts.ceilings_compared
            >= report
                .tally
                .floor_raises_pushed_by_the_session_before_a_published_change
                + report.tally.floor_raises_pushed_by_mounts,
        "会话与挂载推的每一串抬 F 模型都要比过上限"
    );
    assert!(
        report
            .tally
            .mounts_by_space_admission
            .get(MountSpaceAdmissionOutcome::AdmittedAfterTheFloorRaises { floor_raises: 0 }.name())
            .copied()
            .unwrap_or(0)
            >= 1
            && report.tally.floor_raises_pushed_by_mounts >= 1,
        "可写挂载一次都没在写行之后推过抬 F 再判够（可写挂载那一处没跑到）"
    );
    assert_eq!(
        count_of(
            &report.tally.refusals_by_member,
            "PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)"
        ) + count_of(
            &report.tally.refusals_by_member,
            "UserChangeRefused::NoSpaceAfterRaisingTheFloor(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))"
        ),
        0,
        "准入判着时式子先拒，落点那一道走不到"
    );
    assert!(
        report.tally.step_images_handed_out > 0,
        "这一段每一步之后跑池级 checker"
    );
}

/// 单元区墙取样点（空间准入判着）那一批的第一个种子只跑 77 步：第 76 步那次可写挂载接在 unlink 删完的一版后面
/// （有树而文件集合为空，D8（核心索引结构） 已定项 16 第 5 条），写行与两次暖机写在这一形上、之后推一串三次空发布的抬 F。
/// 执行器逐次比挂载写出的分配记录时，写行与暖机按「有树」挑进比较链：六次发布逐次比过、改写或新增的记录的代都等于各自那次的 txg，
/// 历史跑完、没有新发现。
/// 判别力：写行与暖机按「带文件」挑（`PoolVersion::file_version`）时这一形的三次被跳过，比较的上一版停在挂载之前那棵树上，
/// 推的第一次空发布把那三次改写的记录全当成自己改的、代却不是自己的 txg，执行器判「改写或新增的分配记录的代不是这次发布的 txg」。
#[test]
fn a_mount_on_the_emptied_version_in_the_unit_area_wall_sampling_compares_its_row_and_warm_up_publishes(
) {
    const OPERATIONS: usize = 77;
    const STEP_OF_THE_MOUNT: usize = 76;
    let history = generate_history_with_weights(
        HistorySeed(UNIT_AREA_WALL_SAMPLING_FIRST_SEED),
        OPERATIONS,
        &UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED.weights,
    );
    let run = execute_history_with(
        &history,
        UNIT_AREA_WALL_SAMPLING_WITH_THE_SPACE_ADMISSION_JUDGED.execution,
        &SharedStream::new(),
        &mut |_| {},
    );
    assert_eq!(
        run.ending,
        HistoryEnding::Completed,
        "{OPERATIONS} 步跑完、没有新发现"
    );
    assert_eq!(
        history.operations[STEP_OF_THE_MOUNT],
        HistoryOperation::CloseAndMountWritable,
        "第 {STEP_OF_THE_MOUNT} 步是可写挂载"
    );
    let StepOutcome::Applied(AppliedEffect::Mounted {
        allocation_records_compared,
        ..
    }) = &run.outcomes[STEP_OF_THE_MOUNT]
    else {
        panic!(
            "第 {STEP_OF_THE_MOUNT} 步挂载做成：{:?}",
            run.outcomes[STEP_OF_THE_MOUNT]
        );
    };
    assert_eq!(
        *allocation_records_compared,
        MountAllocationComparison::Compared { publishes: 6 },
        "写行、两次暖机（都在删完的一版上）与推的三次空发布逐次比过"
    );
}

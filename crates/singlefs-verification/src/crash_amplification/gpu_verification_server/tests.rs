use super::*;

/// 替身 GPU 批处理服务（不上卡）：库里的每一行待判行（服务不看判器的版本，照行的数据判），照块号用 CPU 判器判那一块（`check_blocks_where_until_with_threads`，判段判红块与
/// 强制现判的块同一个谓词）、删掉那一行，强制现判的另写一行强制现判凭据（结论摘要照库里判完的那一份）。核每一行指的录入块与态序号区间
/// 是计划里那个块号的那一块。
fn stand_in_service_judges_every_pending_row(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    flow: &FlowOfTheLibrary<'_>,
    plan: &CrashPlan,
    conclusions: &ConclusionsStoredUnder<'_>,
) {
    let rows = library.take().every_gpu_pending_row().expect("扫得了库");
    for (stream, block_number, row) in rows {
        let (point_index, local) = plan
            .points
            .iter()
            .enumerate()
            .flat_map(|(point_index, point)| {
                point
                    .blocks()
                    .into_iter()
                    .map(move |local| (point_index, local))
            })
            .nth(usize::try_from(block_number).expect("块号装得进 usize"))
            .expect("待判行的块号在计划里");
        let point = &plan.points[point_index];
        assert_eq!(
            (row.recorded_block.clone(), row.state_ordinals.clone()),
            (
                point.block_key(&local),
                point.ordinals.start + local.start..point.ordinals.start + local.end
            ),
            "块号 {block_number} 的待判行指的是计划里这一块：块键与态序号区间"
        );
        check_blocks_where_until_with_threads(
            library,
            flow,
            plan,
            &|candidate| candidate == block_number,
            conclusions,
            &|| false,
            1,
        )
        .expect("替身服务判得完");
        if row.forced == crate::verdict_store::ForcedRejudging::Forced {
            let store = library.take();
            let stored = store
                .read_conclusion_block(
                    &row.recorded_block,
                    &ConclusionKind::ImageJudgement,
                    &row.conclusion_version,
                )
                .expect("读得了库")
                .expect("刚判完");
            store
                .store_the_gpu_rejudged_row(
                    row.pool_entry_number,
                    &stream,
                    block_number,
                    &crate::verdict_store::GpuRejudgedRow {
                        holder_identity: b"stand-in".to_vec(),
                        conclusion_digest:
                            ConclusionSpelledOutInVerdictVectors::of_the_stored_block(
                                &store, &stored,
                            )
                            .conclusion_digest(),
                    },
                )
                .expect("写得进库");
        }
        library
            .take()
            .delete_the_gpu_pending_row(&stream, block_number)
            .expect("删得了");
    }
}

/// 场景：那条两个扇区的小流录进两份空库；一份由这个进程里的 CPU 判器照判段原来的样子判（`check_blocks_where_until`），
/// 另一份走待判行那一路（`judge_my_blocks_through_the_pending_rows`），由替身服务在两次现算之间把待判行判掉写回。
/// 预期：待判行那一路每一块都写了待判行、都等到判完（现判数等于块数），判完库里一行待判行都不剩；两份库逐块逐态是同一个结论
/// （按判定向量本身比，[`ConclusionSpelledOutInVerdictVectors::of_the_stored_block`]）。
/// 判别力（`crates/mutations.tsv`）：待判行的态序号区间不加点的起点（替身服务照行里那一段判的就不是这一块），这条红。
#[test]
fn the_pending_rows_judged_by_a_stand_in_service_read_back_the_conclusions_judged_in_this_process()
{
    let versions =
        vec![
            ConclusionVersion(sha256_digest(b"version of the pending rows unit tests\n"));
            plan_of_the_two_sector_flow_with_its_block_count()
                .0
                .points
                .len()
        ];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let (_in_process_directory, mut in_process_store) =
        an_empty_library_for("pending-rows-reference-in-process");
    let in_process_read_back = a_flow_of_two_writes_to_two_sectors().with_the_flow(|flow| {
        record_the_flow_and_read_it_back(&mut in_process_store, flow).expect("流写得进库、读得回")
    });
    let in_process_flow = in_process_read_back.flow();
    let plan = plan_crash_points(&in_process_flow);
    let total_blocks: usize = plan.points.iter().map(|point| point.blocks().len()).sum();
    assert!(
        plan.points.iter().any(|point| point.ordinals.start > 0),
        "前提：至少一个点的态序号不从 0 起（态序号区间要加点的起点才分得出）"
    );
    check_blocks_where_until(
        &mut in_process_store,
        &in_process_flow,
        &plan,
        &|_| true,
        &conclusions,
        &|| false,
    )
    .expect("这个进程里判得完");
    let (_pending_rows_directory, mut pending_rows_store) =
        an_empty_library_for("pending-rows-through-the-stand-in-service");
    let pending_rows_read_back = a_flow_of_two_writes_to_two_sectors().with_the_flow(|flow| {
        record_the_flow_and_read_it_back(&mut pending_rows_store, flow).expect("流写得进库、读得回")
    });
    let pending_rows_flow = pending_rows_read_back.flow();
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut pending_rows_store);
    let tally = judge_my_blocks_through_the_pending_rows(
        &library,
        &plan,
        &STAND_IN_RECORDED_FLOW_DIGEST,
        &STAND_IN_PACKED_JUDGE_TABLES_DIGEST,
        &|_| true,
        &conclusions,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|library_of_the_service| {
                stand_in_service_judges_every_pending_row(
                    library_of_the_service,
                    &pending_rows_flow,
                    &plan,
                    &conclusions,
                );
            },
        },
        &|| false,
    )
    .expect("替身服务判完、读得回");
    assert_eq!(
        usize::try_from(tally.blocks_checked).expect("装得进 usize"),
        total_blocks,
        "每一块都写了待判行、等到服务判完"
    );
    assert_eq!(
        library.take().every_gpu_pending_row().expect("扫得了库"),
        Vec::new(),
        "判完一行待判行都不剩"
    );
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            let key = point.block_key(&local);
            let read = |store: &VerdictStore| {
                let stored = store
                    .read_conclusion_block(
                        &key,
                        &ConclusionKind::ImageJudgement,
                        &versions[point_index],
                    )
                    .expect("读得了库")
                    .expect("每一块都有结论");
                ConclusionSpelledOutInVerdictVectors::of_the_stored_block(store, &stored)
            };
            assert_eq!(
                read(&library.take()),
                read(&in_process_store),
                "点 {point_index} 块 {local:?}：待判行那一路读回的与这个进程里判的逐态相同"
            );
        }
    }
}

/// 场景：同上录进一份空库、走待判行那一路；替身服务头两次现算之间不判，第三次才判；头一次现算之后就叫停。
/// 预期：叫停之后不再等（两次现算之间只走了一次），交回的账一块都没判，写下的待判行都还在库里（留给服务与下一趟）。
/// 判别力（`crates/mutations.tsv`）：等的那一圈不问叫停（等到第三次服务判完才出圈），这条红。
#[test]
fn a_stop_while_waiting_for_the_gpu_verification_server_leaves_the_pending_rows_and_returns() {
    let (_directory, mut store) = an_empty_library_for("pending-rows-stop");
    let read_back = a_flow_of_two_writes_to_two_sectors().with_the_flow(|flow| {
        record_the_flow_and_read_it_back(&mut store, flow).expect("流写得进库、读得回")
    });
    let flow = read_back.flow();
    let plan = plan_crash_points(&flow);
    let total_blocks: usize = plan.points.iter().map(|point| point.blocks().len()).sum();
    let versions = vec![
        ConclusionVersion(sha256_digest(
            b"version of the pending rows stop unit test\n"
        ));
        plan.points.len()
    ];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let reckonings_waited = std::sync::atomic::AtomicUsize::new(0);
    let ended = judge_my_blocks_through_the_pending_rows(
        &library,
        &plan,
        &STAND_IN_RECORDED_FLOW_DIGEST,
        &STAND_IN_PACKED_JUDGE_TABLES_DIGEST,
        &|_| true,
        &conclusions,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|library_of_the_service| {
                let waited =
                    reckonings_waited.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if waited >= 3 {
                    stand_in_service_judges_every_pending_row(
                        library_of_the_service,
                        &flow,
                        &plan,
                        &conclusions,
                    );
                }
            },
        },
        &|| reckonings_waited.load(std::sync::atomic::Ordering::SeqCst) >= 1,
    )
    .expect("叫停照常交回");
    assert_eq!(
        reckonings_waited.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "叫停之后不再等"
    );
    assert_eq!(ended, CheckingTally::default(), "一块都没判完");
    assert_eq!(
        library
            .take()
            .every_gpu_pending_row()
            .expect("扫得了库")
            .len(),
        total_blocks,
        "写下的待判行都还在"
    );
}

/// 场景：同上录进一份空库，每一块先存一份干净的判器结论（[`store_a_judge_conclusion_of_the_block`]，不红、连同逐盘自证槽数，
/// 与待判行那一路同一套结论）：复用口径（[`ConclusionsStoredUnder::is_stored_for_reuse`]）下每一块都已判。CPU 判器真判这条流会判出红块，
/// 记着红的块不复用、照旧要判，所以库况不用它造；再走第 ① 段写待判行那一路，等的那一圈一走到就当场失败（没有服务替它判，等下去不会结束）。
/// 预期：一行待判行都不写，这一趟判完（交回一块没现判的账），等的那一圈一次都没走；库里一行待判行都没有。
/// 判别力（`crates/mutations.tsv`）：复用口径一律答没判过（判过的块也写待判行），进了等的那一圈，这条当场红。
#[test]
fn a_run_whose_blocks_are_all_judged_already_writes_no_pending_row() {
    let (_disposable, mut store) = an_empty_library_for("pending-rows-all-judged-already");
    let read_back = a_flow_of_two_writes_to_two_sectors().with_the_flow(|flow| {
        record_the_flow_and_read_it_back(&mut store, flow).expect("流写得进库、读得回")
    });
    let flow = read_back.flow();
    let plan = plan_crash_points(&flow);
    let versions = vec![
        ConclusionVersion(sha256_digest(
            b"version of the pending rows all judged already unit test\n"
        ));
        plan.points.len()
    ];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    for (point_index, point) in plan.points.iter().enumerate() {
        for local in point.blocks() {
            store_a_judge_conclusion_of_the_block(
                &mut store,
                &plan,
                &conclusions,
                point_index,
                &local,
                None,
                "unit_test_publish",
            );
        }
    }
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    // 这里没有 GPU 批处理服务、替身也不判：走进等服务结论的那一圈就永远等不到，所以第一次走到就当场失败
    let tally = judge_my_blocks_through_the_pending_rows(
        &library,
        &plan,
        &STAND_IN_RECORDED_FLOW_DIGEST,
        &STAND_IN_PACKED_JUDGE_TABLES_DIGEST,
        &|_| true,
        &conclusions,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|_library_of_the_service| {
                panic!(
                    "每一块都判过：不该写待判行、不该进等 GPU 批处理服务结论的那一圈（走到这里就是写了待判行）"
                )
            },
        },
        &|| false,
    )
    .expect("一行都不用写：照常交回");
    assert_eq!(
        tally,
        CheckingTally::default(),
        "结论都在：这一趟判完、一块都没现判"
    );
    assert_eq!(
        library.take().every_gpu_pending_row().expect("扫得了库"),
        Vec::new(),
        "一行待判行都没写"
    );
}

/// 场景：等 GPU 批处理服务时打不打进度行：满间隔而还剩的块数没变、满间隔而块数变了、恰满间隔而块数变了、不满间隔而块数变了。
/// 预期：满间隔且块数变了才打；块数没变不打（服务一直不判时日志不涨）。
/// 判别力（`crates/mutations.tsv`）：不看块数变没变（满间隔就打），这条红。
#[test]
fn the_waiting_line_is_due_only_once_the_interval_passed_and_the_blocks_remaining_changed() {
    let progress_interval = std::time::Duration::from_secs(300);
    let past_the_interval = std::time::Duration::from_secs(301);
    assert!(
        !gpu_verification_server_waiting_line_is_due(past_the_interval, progress_interval, 4, 4),
        "满间隔而块数没变：不打"
    );
    assert!(
        gpu_verification_server_waiting_line_is_due(past_the_interval, progress_interval, 3, 4),
        "满间隔而块数变了：打"
    );
    assert!(
        gpu_verification_server_waiting_line_is_due(progress_interval, progress_interval, 3, 4),
        "恰满间隔而块数变了：打"
    );
    assert!(
        !gpu_verification_server_waiting_line_is_due(
            std::time::Duration::from_secs(299),
            progress_interval,
            3,
            4
        ),
        "不满间隔：不打"
    );
}

/// 场景：同上录进一份空库、走第 ① 段写待判行那一路；头一次现算之后就叫停。
/// 预期：交回判完的账（一块都还没判完），每一块都写了待判行、都还在库里，行里不带判器的版本；核对待判行那张表是空的。
/// 判别力（`crates/mutations.tsv`）：写待判行那一句删掉（一行都不写、直接等），这条红。
#[test]
fn pending_rows_are_written_without_asking_the_gpu_verification_server() {
    let (_disposable, mut store) = an_empty_library_for("pending-rows-written-without-asking");
    let read_back = a_flow_of_two_writes_to_two_sectors().with_the_flow(|flow| {
        record_the_flow_and_read_it_back(&mut store, flow).expect("流写得进库、读得回")
    });
    let flow = read_back.flow();
    let plan = plan_crash_points(&flow);
    let total_blocks: usize = plan.points.iter().map(|point| point.blocks().len()).sum();
    let versions = vec![
        ConclusionVersion(sha256_digest(
            b"version of the pending rows written without asking unit test\n"
        ));
        plan.points.len()
    ];
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &versions,
    };
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut store);
    let reckonings_waited = std::sync::atomic::AtomicUsize::new(0);
    let tally = judge_my_blocks_through_the_pending_rows(
        &library,
        &plan,
        &STAND_IN_RECORDED_FLOW_DIGEST,
        &STAND_IN_PACKED_JUDGE_TABLES_DIGEST,
        &|_| true,
        &conclusions,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|_library_of_the_service| {
                reckonings_waited.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            },
        },
        &|| reckonings_waited.load(std::sync::atomic::Ordering::SeqCst) >= 1,
    )
    .expect("叫停照常交回");
    assert_eq!(
        tally,
        CheckingTally::default(),
        "照常写待判行、等；叫停时一块都还没判完"
    );
    // 一条语句只取一次库：取到的要到语句末尾才放，同一条语句里再取等的是自己
    let gpu_pending_rows = library.take().every_gpu_pending_row().expect("扫得了库");
    let facts_verifier_pending_rows = library
        .take()
        .every_facts_verifier_pending_row()
        .expect("扫得了库");
    assert_eq!(
        (gpu_pending_rows.len(), facts_verifier_pending_rows.len()),
        (total_blocks, 0),
        "每一块都写了待判行；核对待判行一行都没写"
    );
    assert!(
        gpu_pending_rows
            .iter()
            .all(|(_stream, _block_number, row)| row.conclusion_version == versions[0]),
        "行里的结论版本是这一趟的那一把（行里不带判器的版本）"
    );
}

/// 场景：只排派活计划那一行怎么拼（几栏各取一个不同的数）。
/// 预期：逐字是钉住的这一句：行首、`judge=gpu`，再是第 ① 段要判的块与状态、复用数、判表大小、第 ② 段要核的块与状态，次序与字段名照钉住的。
/// 判别力（`crates/mutations.tsv`）：第 ② 段那两栏的字段名写错，这条红。
#[test]
fn the_placement_line_is_spelled_out_word_for_word() {
    assert_eq!(
            placement_line_of_the_run(&PlacementOfTheRun {
                pending_blocks: 3,
                pending_states: 700,
                states_reused: 11,
                tables_mebibytes: 42,
                verifier_pending_blocks: 5,
                verifier_pending_states: 900,
            }),
            "CRASH_AMPLIFICATION_PLACEMENT judge=gpu pending_blocks=3 pending_states=700 states_reused=11 tables_mebibytes=42 verifier_pending_blocks=5 verifier_pending_states=900",
            "只排派活计划那一行逐字是这一句"
        );
}

/// 三族待判行在临时库登记里的标签各是各的（服务的日志点名是哪一处登记的）。
/// 判别力（`crates/mutations.tsv`）：判表那一族的标签写成下划线连的，这条红。
#[test]
fn the_registration_labels_of_the_three_pending_row_families_take_their_names() {
    assert_eq!(
        [
            PendingRowFamilyOfTheGpuVerificationServer::ImageJudgement,
            PendingRowFamilyOfTheGpuVerificationServer::FactsVerifier,
            PendingRowFamilyOfTheGpuVerificationServer::PackedTables,
        ]
        .map(
            PendingRowFamilyOfTheGpuVerificationServer::label_of_the_temporary_library_registration
        ),
        ["image-judgement", "facts-verifier", "packed-tables"],
        "三族待判行的登记标签"
    );
}

/// 场景：判段的库不在临时 prune-server 里（真库，`VerdictStore::temporary_service_of_the_library` 交 `None`）；登记是替身，记下被调了几回。
/// 预期：交回「不在临时服务里」，一回都没登记。
/// 判别力（`crates/mutations.tsv`）：库不在临时服务里也照登记，这条红。
#[test]
fn a_judging_stage_on_a_library_kept_as_is_registers_nothing() {
    let registrations = std::sync::atomic::AtomicUsize::new(0);
    let outcome = temporary_library_registered_when_the_library_is_in_a_temporary_service(
        None,
        &|_service| {
            registrations.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err("真库不该登记".to_string())
        },
    )
    .unwrap_or_else(|reason| panic!("{reason}"));
    assert!(
        matches!(
            outcome,
            TemporaryLibraryOfTheJudgingStage::NotInATemporaryService
        ),
        "真库：交回不在临时服务里"
    );
    assert_eq!(
        registrations.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "一回都没登记"
    );
}

/// 场景：判段的库是一次性库（临时服务里的一段名字空间），另起一份一次性库替真服务；登记把判段那一份的临时服务登进替身真服务，
/// 不问 GPU 批处理服务收不收。
/// 预期：交回登记了的那一支，替身真服务里列得出恰好它那一把登记键、解出来连的是判段那一份临时服务；交回的结局丢掉之后列不出（放掉了）。
/// 判别力（`crates/mutations.tsv`）：在临时服务里也不登记（删掉登记那一句），这条红。
#[test]
fn a_judging_stage_on_a_disposable_library_registers_it_with_the_real_service() {
    let (_library_of_the_judging_stage, store) =
        an_empty_library_for("temporary-library-registered");
    // 一次性库留着：替真服务，判段的登记登进它里面
    let standing_in_for_the_real_service =
        crate::crash_library::DisposableLibrary::new("temporary-library-real-service")
            .unwrap_or_else(|refusal| panic!("{refusal}"));
    let temporary_service = store
        .temporary_service_of_the_library()
        .expect("前提：一次性库在临时服务里");
    let this_host = crate::gpu_card_configuration::multi_host_addresses_for_the_unit_tests()
        .expect("单测写死的判法输出认得出");
    let outcome = temporary_library_registered_when_the_library_is_in_a_temporary_service(
        Some(temporary_service.clone()),
        &|service| {
            crate::crash_library::register_the_temporary_library_with_the_service(
                standing_in_for_the_real_service.route(),
                crate::crash_library::TemporaryLibraryRegistration::of_this_process(
                    this_host.this_host(),
                    service.clone(),
                    "registered-by-the-judging-stage",
                )?,
                crate::crash_library::TEMPORARY_LIBRARY_REGISTRATION_LEASE,
                crate::crash_library::TEMPORARY_LIBRARY_REGISTRATION_RENEWAL_INTERVAL,
                crate::verdict_store::PruneServerFailureHandling::ReturnTheError,
            )
        },
    )
    .unwrap_or_else(|reason| panic!("{reason}"));
    let TemporaryLibraryOfTheJudgingStage::Registered(registered) = outcome else {
        panic!("库在临时服务里：交回登记了的那一支");
    };
    let listed = crate::crash_library::temporary_library_registrations_listed_by_the_service(
        standing_in_for_the_real_service.route(),
    )
    .unwrap_or_else(|reason| panic!("{reason}"));
    assert_eq!(
        listed
            .iter()
            .map(|claim| claim.key.clone())
            .collect::<Vec<_>>(),
        vec![registered.claim_key().to_vec()],
        "替身真服务里列得出恰好这一份登记"
    );
    assert_eq!(
        registered.registration().service,
        temporary_service,
        "登记连的是判段那一份临时服务"
    );
    drop(registered);
    assert_eq!(
        crate::crash_library::temporary_library_registrations_listed_by_the_service(
            standing_in_for_the_real_service.route(),
        )
        .unwrap_or_else(|reason| panic!("{reason}")),
        Vec::new(),
        "交回的结局丢掉之后列不出：放掉了"
    );
}

/// 强制现判那几条的一块：复用哨兵那几条的库里点 0 的头一块，写成强制现判的待判行的样子（入池序号写死）。
fn the_forced_block_waited_for_on_the_server(
    plan: &CrashPlan,
    versions: &[ConclusionVersion],
) -> BlockWaitedForOnTheServer {
    let local = plan.points[0].blocks()[0].clone();
    let stream = gpu_pending_stream_key(
        &STAND_IN_RECORDED_FLOW_DIGEST,
        &STAND_IN_PACKED_JUDGE_TABLES_DIGEST,
        &versions[0],
    );
    BlockWaitedForOnTheServer {
        point_index: 0,
        local: local.clone(),
        block_number: 0,
        stream,
        row: crate::verdict_store::GpuPendingRow {
            recorded_block: plan.points[0].block_key(&local),
            recorded_flow_digest: STAND_IN_RECORDED_FLOW_DIGEST,
            packed_judge_tables_digest: STAND_IN_PACKED_JUDGE_TABLES_DIGEST,
            state_ordinals: plan.points[0].ordinals.start + local.start
                ..plan.points[0].ordinals.start + local.end,
            conclusion_version: versions[0],
            forced: crate::verdict_store::ForcedRejudging::Forced,
            pool_entry_number: 41,
        },
    }
}

/// 场景：每一块都有不红结论的库里，强制现判那一块判完了，库里却没有它的强制现判凭据；再补上一行摘要对的凭据。
/// 预期：缺凭据报红、点名块号与「没有它的强制现判凭据」；补上摘要对的之后不报。
/// 判别力（`crates/mutations.tsv`）：缺凭据时当没事（交 `None`），这条红。
#[test]
fn a_forced_block_judged_by_the_server_without_its_rejudged_row_is_red() {
    let (_directory, store, plan, versions) =
        a_library_where_every_block_of_the_two_sector_flow_is_reusable(
            "forced-rejudged-row-missing",
        );
    let waited = the_forced_block_waited_for_on_the_server(&plan, &versions);
    let problem = problem_of_the_forced_rejudging_by_the_gpu_verification_server(
        &store,
        &waited,
        &ConclusionKind::ImageJudgement,
    )
    .expect("读得了库");
    assert!(
        problem.as_deref().is_some_and(|problem_text| problem_text
            .starts_with("复用哨兵：强制现判的块 0")
            && problem_text.contains("没有它的强制现判凭据")),
        "缺凭据报红：{problem:?}"
    );
    let stored = store
        .read_conclusion_block(
            &waited.row.recorded_block,
            &ConclusionKind::ImageJudgement,
            &versions[0],
        )
        .expect("读得了库")
        .expect("每一块都有结论");
    store
        .store_the_gpu_rejudged_row(
            waited.row.pool_entry_number,
            &waited.stream,
            waited.block_number,
            &crate::verdict_store::GpuRejudgedRow {
                holder_identity: b"stand-in".to_vec(),
                conclusion_digest: ConclusionSpelledOutInVerdictVectors::of_the_stored_block(
                    &store, &stored,
                )
                .conclusion_digest(),
            },
        )
        .expect("写得进库");
    assert_eq!(
        problem_of_the_forced_rejudging_by_the_gpu_verification_server(
            &store,
            &waited,
            &ConclusionKind::ImageJudgement,
        )
        .expect("读得了库"),
        None,
        "凭据在、摘要与库里那一份相同：不报"
    );
}

/// 场景：同上，强制现判凭据在，但里面的结论摘要是另一份结论的（头一个状态红的那一份）。
/// 预期：报红、点名块号与「结论摘要」「不同」。
/// 判别力（`crates/mutations.tsv`）：不比凭据里的摘要与库里那一份，这条红。
#[test]
fn a_forced_block_whose_rejudged_row_names_another_conclusion_digest_is_red() {
    let (_directory, store, plan, versions) =
        a_library_where_every_block_of_the_two_sector_flow_is_reusable(
            "forced-rejudged-row-differs",
        );
    let waited = the_forced_block_waited_for_on_the_server(&plan, &versions);
    let stored = store
        .read_conclusion_block(
            &waited.row.recorded_block,
            &ConclusionKind::ImageJudgement,
            &versions[0],
        )
        .expect("读得了库")
        .expect("每一块都有结论");
    let mut another_conclusion =
        ConclusionSpelledOutInVerdictVectors::of_the_stored_block(&store, &stored);
    another_conclusion.verdict_vector_of_each_state[0] =
        encoded_verdict_of_the_red_items(&["unit_test_red_item".to_string()]);
    assert_ne!(
        another_conclusion.conclusion_digest(),
        ConclusionSpelledOutInVerdictVectors::of_the_stored_block(&store, &stored)
            .conclusion_digest(),
        "前提：另一份结论的摘要不同"
    );
    store
        .store_the_gpu_rejudged_row(
            waited.row.pool_entry_number,
            &waited.stream,
            waited.block_number,
            &crate::verdict_store::GpuRejudgedRow {
                holder_identity: b"stand-in".to_vec(),
                conclusion_digest: another_conclusion.conclusion_digest(),
            },
        )
        .expect("写得进库");
    let problem = problem_of_the_forced_rejudging_by_the_gpu_verification_server(
        &store,
        &waited,
        &ConclusionKind::ImageJudgement,
    )
    .expect("读得了库");
    assert!(
        problem.as_deref().is_some_and(|problem_text| problem_text
            .starts_with("复用哨兵：强制现判的块 0")
            && problem_text.contains("结论摘要")
            && problem_text.contains("不同")),
        "摘要不同报红：{problem:?}"
    );
}

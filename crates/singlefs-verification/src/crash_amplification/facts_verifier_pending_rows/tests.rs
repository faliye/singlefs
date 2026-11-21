use super::*;

/// 核对待判行那几条用的库：两个扇区的小流（[`a_flow_of_two_writes_to_two_sectors`]）按摘要写进一份空库、读回，照录下的片排计划，
/// 事实表照计划抽、写进库读回；每一块存一份干净的判器结论（镜像判定那一种，[`store_a_judge_conclusion_of_the_block`]：判器那一侧块块都绿）。
struct LibraryOfTheTwoSectorFlowWithItsFacts {
    _disposable: crate::crash_library::DisposableLibrary,
    store: VerdictStore,
    plan: CrashPlan,
    facts: crate::crash_facts::FlowFactsReadBack,
    segment_expansions: Vec<SegmentExpansionFact>,
    recorded_flow_digest: [u8; DIGEST_BYTES],
    versions: Vec<ConclusionVersion>,
}

fn a_library_of_the_two_sector_flow_with_its_facts(
    test_name: &str,
) -> LibraryOfTheTwoSectorFlowWithItsFacts {
    let (disposable, mut store) = an_empty_library_for(test_name);
    let recorded = a_flow_of_two_writes_to_two_sectors();
    let recorded_flow_digest = recorded.digest();
    store
        .store_recorded_flow(&recorded_flow_digest, &recorded.to_bytes())
        .expect("录下的流程写得进库");
    let flow_read_back = RecordedFlowReadBack::read_from_the_library(&store, &recorded_flow_digest)
        .expect("录下的流程读得回");
    let flow = flow_read_back.flow();
    let plan = plan_crash_points_with_slices(&flow, &flow_read_back.recorded().segment_slices);
    let (facts_of_the_plan, segment_expansions) =
        facts_and_segment_expansions_of_the_plan(&flow, &plan);
    let facts =
        crate::crash_library::record_the_facts_and_read_them_back(&mut store, &facts_of_the_plan)
            .expect("事实表写得进库、读得回");
    let versions = vec![
        ConclusionVersion(sha256_digest(
            b"version of the facts verifier pending rows unit tests\n"
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
    LibraryOfTheTwoSectorFlowWithItsFacts {
        _disposable: disposable,
        store,
        plan,
        facts,
        segment_expansions,
        recorded_flow_digest,
        versions,
    }
}

/// 替身服务往核对那一侧塞的故障：没有，或每个状态的结论位都置上一位（核对红）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaultPutOnTheStandInFactsVerifier {
    NoFault,
    ARedBitOnEveryState,
}

/// 替身 GPU 批处理服务（不上卡）：库里的每一行核对待判行（服务不看核对器的版本，照行的数据核）照服务那三样做——在库上现算核没核完
/// （[`facts_verifier_pending_block_is_verified_in_the_library`]，前提：还没核）、读回核对材料（[`material_of_a_facts_verifier_pending_block`]）、
/// 照材料用 CPU 参照（[`verify_states_from_facts_by_the_segment_expansions`]；服务在卡上用核对内核）核那一段状态、
/// 照 [`facts_verifier_frame_of_a_pending_block`] 编成那一帧写回（写核对行、删核对待判行）。`fault` 塞在核出的结论位上。
fn stand_in_service_verifies_every_facts_verifier_pending_row(
    library: &LibraryTakenAtEachReadOrWrite<'_>,
    fault: FaultPutOnTheStandInFactsVerifier,
) {
    let rows = library
        .take()
        .every_facts_verifier_pending_row()
        .expect("扫得了库");
    for (stream, block_number, row) in rows {
        let store = library.take();
        assert!(
            !facts_verifier_pending_block_is_verified_in_the_library(&store, &row)
                .expect("读得了库"),
            "前提：块号 {block_number} 还没核"
        );
        let material =
            material_of_a_facts_verifier_pending_block(&store, &row).expect("读得回核对材料");
        let mut bits = verify_states_from_facts_by_the_segment_expansions(
            &material.facts,
            &material.segment_expansions,
            material.state_ordinals.clone(),
        );
        match fault {
            FaultPutOnTheStandInFactsVerifier::NoFault => {}
            FaultPutOnTheStandInFactsVerifier::ARedBitOnEveryState => {
                for bits_of_a_state in &mut bits {
                    *bits_of_a_state |= 1;
                }
            }
        }
        let frame = facts_verifier_frame_of_a_pending_block(
            &store,
            &material,
            &stream,
            block_number,
            &row,
            &bits,
        )
        .expect("编得出那一帧");
        store
            .write_the_facts_verifier_rows_of_one_frame(&frame)
            .expect("写得进库");
    }
}

/// 计划里全部块的状态数。
fn states_of_the_plan(plan: &CrashPlan) -> u64 {
    plan.points
        .iter()
        .flat_map(|point| point.blocks())
        .map(|local| local.end - local.start)
        .sum()
}

/// 场景：两个扇区的小流录进两份空库（判器结论块块都绿）；一份由这个进程里的 CPU 参照照判段没配 GPU 时的样子核（`verify_blocks_from_these_facts`），
/// 另一份走核对待判行那一路（[`verify_my_blocks_through_the_facts_verifier_pending_rows`]），由替身服务照服务那三样在两次现算之间核掉写回。
/// 预期：每一块都写了核对待判行、都等到核完（核了的块数与状态数是整条流的），核完库里一行核对待判行都不剩；
/// 两份库同一把核对键下逐块读回的核对行相同，第 ③ 段两份比出的也相同。
/// 判别力（`crates/mutations.tsv`）：核对待判行的态序号区间不加点的起点（替身服务读材料时对不上计划里那一块），这条红。
#[test]
fn the_facts_verifier_pending_rows_verified_by_a_stand_in_service_read_back_as_verified_in_this_process(
) {
    let mut in_process = a_library_of_the_two_sector_flow_with_its_facts(
        "facts-verifier-pending-rows-reference-in-process",
    );
    let mut pending_rows = a_library_of_the_two_sector_flow_with_its_facts(
        "facts-verifier-pending-rows-through-the-stand-in",
    );
    let plan = &pending_rows.plan;
    assert!(
        plan.points.iter().any(|point| point.ordinals.start > 0),
        "前提：至少一个点的态序号不从 0 起（态序号区间要加点的起点才分得出）"
    );
    let verifier_row_key = facts_verifier_row_key(&pending_rows.facts.digest());
    assert_eq!(
        verifier_row_key,
        facts_verifier_row_key(&in_process.facts.digest()),
        "前提：两份库的事实表相同，核对键相同"
    );
    verify_blocks_from_these_facts(
        &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut in_process.store),
        &in_process.plan,
        &in_process.facts,
        &in_process.segment_expansions,
        &|_| true,
        &FactsVerifierOn::Cpu,
    )
    .expect("这个进程里核得完");
    let recorded_flow_digest = pending_rows.recorded_flow_digest;
    let facts_digest = pending_rows.facts.digest();
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut pending_rows.store);
    let ended = verify_my_blocks_through_the_facts_verifier_pending_rows(
        &library,
        plan,
        &recorded_flow_digest,
        &facts_digest,
        &|_| true,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|library_of_the_service| {
                stand_in_service_verifies_every_facts_verifier_pending_row(
                    library_of_the_service,
                    FaultPutOnTheStandInFactsVerifier::NoFault,
                );
            },
        },
        &|| false,
    )
    .expect("替身服务核完、读得回");
    let tally = ended;
    let total_blocks: u64 = plan
        .points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("装得进 u64"))
        .sum();
    assert_eq!(
        (
            tally.blocks_verified,
            tally.blocks_already_verified,
            tally.states_verified
        ),
        (total_blocks, 0, states_of_the_plan(plan)),
        "每一块都写了核对待判行、等到核完"
    );
    assert_eq!(
        library
            .take()
            .every_facts_verifier_pending_row()
            .expect("扫得了库"),
        Vec::new(),
        "核完一行核对待判行都不剩"
    );
    for point in &plan.points {
        for local in point.blocks() {
            let key = point.block_key(&local);
            let read = |store: &VerdictStore| {
                store
                    .read_verifier_block(&key, &verifier_row_key)
                    .expect("读得了库")
                    .expect("每一块都有核对行")
            };
            assert_eq!(
                read(&library.take()),
                read(&in_process.store),
                "块 {local:?}：核对待判行那一路读回的与这个进程里核的相同"
            );
        }
    }
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &pending_rows.versions,
    };
    let compare = |store: &VerdictStore| {
        compare_the_conclusions_and_the_verifier_of_the_blocks(
            store,
            plan,
            &verifier_row_key,
            &conclusions,
            &|_| true,
        )
        .expect("读得了库")
    };
    assert_eq!(
        compare(&library.take()),
        compare(&in_process.store),
        "第 ③ 段两份比出的相同"
    );
}

/// 场景：同上走核对待判行那一路，判器结论块块都绿；替身服务往核对那一侧塞一个已知故障：每个状态的结论位都置上一位（核对红）。
/// 预期：核完读回的账里核对红的状态数是整条流的状态数；第 ③ 段比出核对红而判器绿的不一致，条数也是整条流的状态数，比的状态一个不少。
/// 判别力（`crates/mutations.tsv`）：服务那一帧编核对行时不照交来的结论位、换成全绿，这条红。
#[test]
fn a_fault_put_on_the_stand_in_facts_verifier_reads_back_as_a_disagreement_with_the_judge() {
    let mut pending_rows =
        a_library_of_the_two_sector_flow_with_its_facts("facts-verifier-pending-rows-with-a-fault");
    let plan = &pending_rows.plan;
    let recorded_flow_digest = pending_rows.recorded_flow_digest;
    let facts_digest = pending_rows.facts.digest();
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut pending_rows.store);
    let ended = verify_my_blocks_through_the_facts_verifier_pending_rows(
        &library,
        plan,
        &recorded_flow_digest,
        &facts_digest,
        &|_| true,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|library_of_the_service| {
                stand_in_service_verifies_every_facts_verifier_pending_row(
                    library_of_the_service,
                    FaultPutOnTheStandInFactsVerifier::ARedBitOnEveryState,
                );
            },
        },
        &|| false,
    )
    .expect("替身服务核完、读得回");
    let tally = ended;
    let states = states_of_the_plan(plan);
    assert_eq!(
        tally.verifier_red_states, states,
        "塞了故障：读回的核对行每个状态都核对红"
    );
    let conclusions = ConclusionsStoredUnder::TheConclusionKind {
        kind: ConclusionKind::ImageJudgement,
        version_of_each_point: &pending_rows.versions,
    };
    let comparison = compare_the_conclusions_and_the_verifier_of_the_blocks(
        &library.take(),
        plan,
        &facts_verifier_row_key(&facts_digest),
        &conclusions,
        &|_| true,
    )
    .expect("读得了库");
    assert_eq!(
        (
            comparison.states_compared,
            comparison.disagreements,
            comparison.blocks_without_verifier
        ),
        (states, states, 0),
        "第 ③ 段照读回的核对行比出核对红而判器绿：{comparison:?}"
    );
}

/// 场景：同上走核对待判行那一路；替身服务头两次现算之间不核，第三次才核；头一次现算之后就叫停。
/// 预期：叫停之后不再等（两次现算之间只走了一次），交回的账一块都没核，写下的核对待判行都还在库里（留给服务与下一趟）。
/// 判别力（`crates/mutations.tsv`）：等的那一圈不问叫停（等到第三次服务核完才出圈），这条红。
#[test]
fn a_stop_while_waiting_for_the_facts_verifier_leaves_the_facts_verifier_pending_rows_and_returns()
{
    let mut pending_rows =
        a_library_of_the_two_sector_flow_with_its_facts("facts-verifier-pending-rows-stop");
    let plan = &pending_rows.plan;
    let total_blocks: usize = plan.points.iter().map(|point| point.blocks().len()).sum();
    let recorded_flow_digest = pending_rows.recorded_flow_digest;
    let facts_digest = pending_rows.facts.digest();
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut pending_rows.store);
    let reckonings_waited = std::sync::atomic::AtomicUsize::new(0);
    let ended = verify_my_blocks_through_the_facts_verifier_pending_rows(
        &library,
        plan,
        &recorded_flow_digest,
        &facts_digest,
        &|_| true,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|library_of_the_service| {
                let waited =
                    reckonings_waited.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if waited >= 3 {
                    stand_in_service_verifies_every_facts_verifier_pending_row(
                        library_of_the_service,
                        FaultPutOnTheStandInFactsVerifier::NoFault,
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
    assert_eq!(ended, VerificationTally::default(), "一块都没核完");
    assert_eq!(
        library
            .take()
            .every_facts_verifier_pending_row()
            .expect("扫得了库")
            .len(),
        total_blocks,
        "写下的核对待判行都还在"
    );
}

/// 场景：同上走第 ② 段（判器结论块块都绿），先由这个进程里的 CPU 参照把每一块都核完、核对行记进库（`verify_blocks_from_these_facts`，
/// 与核对待判行那一路同一把核对键：核对器的代码不进键，CPU 核的与 GPU 核的同一把）；再走第 ② 段写核对待判行那一路，
/// 等的那一圈一走到就当场失败（没有服务替它核，等下去不会结束）。
/// 预期：一行核对待判行都不写，这一趟核完（交回每一块都开写之前就核过的账），等的那一圈一次都没走；库里一行核对待判行都没有。
/// 判别力（`crates/mutations.tsv`）：核对键里带上核对器的版本（CPU 参照核的那一行 GPU 那一路认不出），每一块都写成核对待判行、
/// 进了等的那一圈，这条当场红。
#[test]
fn a_run_whose_blocks_are_all_verified_already_writes_no_facts_verifier_pending_row() {
    let mut pending_rows = a_library_of_the_two_sector_flow_with_its_facts(
        "facts-verifier-pending-rows-all-verified-already",
    );
    let total_blocks: u64 = pending_rows
        .plan
        .points
        .iter()
        .map(|point| u64::try_from(point.blocks().len()).expect("装得进 u64"))
        .sum();
    verify_blocks_from_these_facts(
        &LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut pending_rows.store),
        &pending_rows.plan,
        &pending_rows.facts,
        &pending_rows.segment_expansions,
        &|_| true,
        &FactsVerifierOn::Cpu,
    )
    .expect("这个进程里核得完");
    let recorded_flow_digest = pending_rows.recorded_flow_digest;
    let facts_digest = pending_rows.facts.digest();
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut pending_rows.store);
    // 这里没有 GPU 批处理服务、替身也不核：走进等服务核对结论的那一圈就永远等不到，所以第一次走到就当场失败
    let tally = verify_my_blocks_through_the_facts_verifier_pending_rows(
        &library,
        &pending_rows.plan,
        &recorded_flow_digest,
        &facts_digest,
        &|_| true,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|_library_of_the_service| {
                panic!(
                    "每一块都核过：不该写核对待判行、不该进等 GPU 批处理服务核对结论的那一圈（走到这里就是写了核对待判行）"
                )
            },
        },
        &|| false,
    )
    .expect("一块都不用核：照常交回");
    assert_eq!(
        tally,
        VerificationTally {
            blocks_verified: 0,
            blocks_already_verified: total_blocks,
            states_verified: 0,
            verifier_red_states: 0,
        },
        "每一块都核过：这一趟核完、一块都没现核"
    );
    assert_eq!(
        library
            .take()
            .every_facts_verifier_pending_row()
            .expect("扫得了库"),
        Vec::new(),
        "一行核对待判行都没写"
    );
}

/// 场景：同上走第 ② 段写核对待判行那一路；头一次现算之后就叫停。
/// 预期：交回核完的账（一块都还没核完），每一块都写了核对待判行、都还在库里，行里不带核对器的版本；判器那一族的待判行表是空的。
/// 判别力（`crates/mutations.tsv`）：写核对待判行那一句删掉（一行都不写、直接等），这条红。
#[test]
fn facts_verifier_pending_rows_are_written_without_asking_the_gpu_verification_server() {
    let mut pending_rows =
        a_library_of_the_two_sector_flow_with_its_facts("facts-verifier-pending-rows-written");
    let total_blocks: usize = pending_rows
        .plan
        .points
        .iter()
        .map(|point| point.blocks().len())
        .sum();
    let recorded_flow_digest = pending_rows.recorded_flow_digest;
    let facts_digest = pending_rows.facts.digest();
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut pending_rows.store);
    let reckonings_waited = std::sync::atomic::AtomicUsize::new(0);
    let tally = verify_my_blocks_through_the_facts_verifier_pending_rows(
        &library,
        &pending_rows.plan,
        &recorded_flow_digest,
        &facts_digest,
        &|_| true,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|_library_of_the_service| {
                reckonings_waited.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            },
        },
        &|| reckonings_waited.load(std::sync::atomic::Ordering::SeqCst) >= 1,
    )
    .expect("照常写、叫停照常交回");
    assert_eq!(
        tally,
        VerificationTally::default(),
        "照常写核对待判行、等；叫停时一块都还没核完"
    );
    // 一条语句只取一次库：取到的要到语句末尾才放，同一条语句里再取等的是自己
    let facts_verifier_pending_rows = library
        .take()
        .every_facts_verifier_pending_row()
        .expect("扫得了库");
    let gpu_pending_rows = library.take().every_gpu_pending_row().expect("扫得了库");
    assert_eq!(
        (facts_verifier_pending_rows.len(), gpu_pending_rows.len()),
        (total_blocks, 0),
        "每一块都写了核对待判行；判器那一族的待判行一行都没写"
    );
}

/// 场景：同上，这一把核对键登在照判不入库的版本里（服务写不进这个进程里的行）。
/// 预期：交错、点名照判不入库，一行核对待判行都没写。
/// 判别力（`crates/mutations.tsv`）：不核核对键照不照判不入库，这条红。
#[test]
fn facts_verifier_pending_rows_are_not_written_under_a_verifier_key_kept_out_of_the_library() {
    let mut pending_rows =
        a_library_of_the_two_sector_flow_with_its_facts("facts-verifier-pending-rows-key-kept-out");
    let recorded_flow_digest = pending_rows.recorded_flow_digest;
    let facts_digest = pending_rows.facts.digest();
    pending_rows
        .store
        .keep_the_rows_of_these_versions_out_of_the_library([facts_verifier_row_key(
            &facts_digest,
        )]);
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut pending_rows.store);
    let refusal = verify_my_blocks_through_the_facts_verifier_pending_rows(
        &library,
        &pending_rows.plan,
        &recorded_flow_digest,
        &facts_digest,
        &|_| true,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|library_of_the_service| {
                stand_in_service_verifies_every_facts_verifier_pending_row(
                    library_of_the_service,
                    FaultPutOnTheStandInFactsVerifier::NoFault,
                );
            },
        },
        &|| false,
    )
    .expect_err("核对键照判不入库：不写核对待判行");
    assert!(
        refusal.contains("照判不入库"),
        "交的错点名照判不入库：{refusal}"
    );
    assert_eq!(
        library
            .take()
            .every_facts_verifier_pending_row()
            .expect("扫得了库"),
        Vec::new(),
        "一行核对待判行都没写"
    );
}

/// 两个扇区的小流那份库里，计划开头那一块（态序号从计划的起点起）的一行核对待判行（入池序号写死），连同它的流键。
fn the_facts_verifier_pending_row_of_the_block_opening_the_plan(
    pending_rows: &LibraryOfTheTwoSectorFlowWithItsFacts,
) -> (
    crate::verdict_store::FactsVerifierPendingStreamKey,
    crate::verdict_store::FactsVerifierPendingRow,
) {
    let point = &pending_rows.plan.points[0];
    let local = point.blocks()[0].clone();
    let facts_digest = pending_rows.facts.digest();
    (
        facts_verifier_pending_stream_key(&pending_rows.recorded_flow_digest, &facts_digest),
        crate::verdict_store::FactsVerifierPendingRow {
            recorded_block: point.block_key(&local),
            recorded_flow_digest: pending_rows.recorded_flow_digest,
            facts_digest,
            state_ordinals: point.ordinals.start + local.start..point.ordinals.start + local.end,
            pool_entry_number: 43,
        },
    )
}

/// 场景：计划开头那一块的核对待判行，态序号区间往后挪一个（块键还是那一块的）。
/// 预期：读核对材料交错，点名态序号区间对不上计划里块键相同的那一块；照原样的那一行读得出，领的就是行里的区间。
/// 判别力（`crates/mutations.tsv`）：读材料时不核行里的区间是不是计划里那一块，这条红。
#[test]
fn the_material_of_a_facts_verifier_pending_row_whose_states_are_not_its_block_is_refused() {
    let pending_rows =
        a_library_of_the_two_sector_flow_with_its_facts("facts-verifier-material-of-another-range");
    let (_stream, row) =
        the_facts_verifier_pending_row_of_the_block_opening_the_plan(&pending_rows);
    let material = material_of_a_facts_verifier_pending_block(&pending_rows.store, &row)
        .expect("照原样的那一行读得出材料");
    assert_eq!(
        material.state_ordinals, row.state_ordinals,
        "领的就是行里的区间"
    );
    let shifted = crate::verdict_store::FactsVerifierPendingRow {
        recorded_block: row.recorded_block.clone(),
        recorded_flow_digest: row.recorded_flow_digest,
        facts_digest: row.facts_digest,
        state_ordinals: row.state_ordinals.start + 1..row.state_ordinals.end + 1,
        pool_entry_number: row.pool_entry_number,
    };
    let Err(refusal) = material_of_a_facts_verifier_pending_block(&pending_rows.store, &shifted)
    else {
        panic!("区间挪过的那一行不该读得出材料");
    };
    assert!(
        refusal.contains("对不上这条流计划里块键相同的任何一块"),
        "点名区间对不上：{refusal}"
    );
}

/// 场景：计划开头那一块的核对待判行读出材料；拿它去编另一行的那一帧（另一行的事实表摘要不同）。
/// 预期：交错，点名材料不是这一行的；照原样的那一行、照原样的结论位编得出。
/// 判别力（`crates/mutations.tsv`）：编那一帧时不核材料是不是这一行的，这条红。
#[test]
fn a_facts_verifier_frame_refuses_the_material_of_another_row() {
    let pending_rows =
        a_library_of_the_two_sector_flow_with_its_facts("facts-verifier-frame-of-another-row");
    let (stream, row) = the_facts_verifier_pending_row_of_the_block_opening_the_plan(&pending_rows);
    let material =
        material_of_a_facts_verifier_pending_block(&pending_rows.store, &row).expect("读得出材料");
    let bits = verify_states_from_facts_by_the_segment_expansions(
        &material.facts,
        &material.segment_expansions,
        material.state_ordinals.clone(),
    );
    facts_verifier_frame_of_a_pending_block(
        &pending_rows.store,
        &material,
        &stream,
        0,
        &row,
        &bits,
    )
    .expect("照原样编得出");
    let another_row = crate::verdict_store::FactsVerifierPendingRow {
        recorded_block: row.recorded_block.clone(),
        recorded_flow_digest: row.recorded_flow_digest,
        facts_digest: [0x6A; DIGEST_BYTES],
        state_ordinals: row.state_ordinals.clone(),
        pool_entry_number: row.pool_entry_number,
    };
    let another_stream = facts_verifier_pending_stream_key(
        &another_row.recorded_flow_digest,
        &another_row.facts_digest,
    );
    let refusal = facts_verifier_frame_of_a_pending_block(
        &pending_rows.store,
        &material,
        &another_stream,
        0,
        &another_row,
        &bits,
    )
    .expect_err("材料不是这一行的");
    assert!(
        refusal.starts_with("材料是录下的流程"),
        "点名材料不是这一行的：{refusal}"
    );
}

/// 场景：计划开头那一块的核对待判行读出材料、照原样的结论位；交来的流键换成另一把。
/// 预期：交错，点名流键不是照这一行算出的那一把（删的就不是这一行）。
/// 判别力（`crates/mutations.tsv`）：编那一帧时不核流键，这条红。
#[test]
fn a_facts_verifier_frame_refuses_a_stream_key_not_of_its_row() {
    let pending_rows =
        a_library_of_the_two_sector_flow_with_its_facts("facts-verifier-frame-of-another-stream");
    let (_stream, row) =
        the_facts_verifier_pending_row_of_the_block_opening_the_plan(&pending_rows);
    let material =
        material_of_a_facts_verifier_pending_block(&pending_rows.store, &row).expect("读得出材料");
    let bits = verify_states_from_facts_by_the_segment_expansions(
        &material.facts,
        &material.segment_expansions,
        material.state_ordinals.clone(),
    );
    let refusal = facts_verifier_frame_of_a_pending_block(
        &pending_rows.store,
        &material,
        &crate::verdict_store::FactsVerifierPendingStreamKey([0x6B; DIGEST_BYTES]),
        0,
        &row,
        &bits,
    )
    .expect_err("流键不是这一行的");
    assert!(
        refusal.contains("交来的流键不是照这一行"),
        "点名流键对不上：{refusal}"
    );
}

/// 场景：计划开头那一块的核对待判行读出材料；交来的结论位少一个状态。
/// 预期：交错，点名结论位的条数与这一块的状态数不等。
/// 判别力（`crates/mutations.tsv`）：编那一帧时不核结论位的条数，这条红。
#[test]
fn a_facts_verifier_frame_refuses_bits_of_another_state_count() {
    let pending_rows =
        a_library_of_the_two_sector_flow_with_its_facts("facts-verifier-frame-of-another-count");
    let (stream, row) = the_facts_verifier_pending_row_of_the_block_opening_the_plan(&pending_rows);
    let material =
        material_of_a_facts_verifier_pending_block(&pending_rows.store, &row).expect("读得出材料");
    let mut bits = verify_states_from_facts_by_the_segment_expansions(
        &material.facts,
        &material.segment_expansions,
        material.state_ordinals.clone(),
    );
    assert!(!bits.is_empty(), "前提：这一块至少一个状态");
    bits.pop();
    let refusal = facts_verifier_frame_of_a_pending_block(
        &pending_rows.store,
        &material,
        &stream,
        0,
        &row,
        &bits,
    )
    .expect_err("结论位少一个状态");
    assert!(
        refusal.contains("个状态的结论位，这一块有"),
        "点名结论位的条数不等：{refusal}"
    );
}

/// 场景：同上走核对待判行那一路；替身服务不经那一帧，直接在核对键下写一行少一个结论位的核对行、删掉核对待判行（服务写回的不是这一块的样子）。
/// 预期：读回时交错，点名核对行的结论位数与这一块的状态数不等。
/// 判别力（`crates/mutations.tsv`）：读回时不核结论位的条数，这条红。
#[test]
fn a_verifier_row_of_another_state_count_read_back_after_the_wait_is_refused() {
    let mut pending_rows = a_library_of_the_two_sector_flow_with_its_facts(
        "facts-verifier-pending-rows-short-verifier-row",
    );
    let recorded_flow_digest = pending_rows.recorded_flow_digest;
    let facts_digest = pending_rows.facts.digest();
    let verifier_digest = facts_verifier_row_key(&facts_digest);
    let library = LibraryTakenAtEachReadOrWrite::held_by_this_caller_alone(&mut pending_rows.store);
    let refusal = verify_my_blocks_through_the_facts_verifier_pending_rows(
        &library,
        &pending_rows.plan,
        &recorded_flow_digest,
        &facts_digest,
        &|_| true,
        &WaitingForTheGpuVerificationServer {
            progress_interval: std::time::Duration::from_secs(3600),
            between_two_reckonings: &|library_of_the_service| {
                let rows = library_of_the_service
                    .take()
                    .every_facts_verifier_pending_row()
                    .expect("扫得了库");
                for (stream, block_number, row) in rows {
                    let states = usize::try_from(row.state_ordinals.end - row.state_ordinals.start)
                        .expect("装得进 usize");
                    library_of_the_service
                        .take()
                        .store_verifier_block(
                            &row.recorded_block,
                            &verifier_digest,
                            &row.facts_digest,
                            &vec![0; states - 1],
                        )
                        .expect("写得进库");
                    library_of_the_service
                        .take()
                        .delete_the_facts_verifier_pending_row(&stream, block_number)
                        .expect("删得了");
                }
            },
        },
        &|| false,
    )
    .expect_err("读回的核对行不是这一块的样子");
    assert!(
        refusal.contains("个结论位，这一块有"),
        "点名结论位的条数不等：{refusal}"
    );
}

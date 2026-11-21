//! 整理与覆盖写上的读错：瞬时读错落在整理的一步上，覆盖写期间读坏的一份让执行器与模型都认出一批搬迁。

use super::*;

/// 故障注入的段数上限：从这个测试周期的种子基起，在这么多段里找一段整理做成过一批的历史。
const SEEDS_SEARCHED_FOR_A_COMPACTION_BATCH: u64 = 64;

/// 瞬时读错撞上整理（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J5）：故障注入入口按比重变量点名「broad 加整理」那一组
/// （`SINGLEFS_FAULT_INJECTION_WEIGHTS=broad-compaction`，`GenerationWeights::BROAD_WITH_COMPACTION_WHILE_MOUNTED`），从种子基起找头一段
/// 整理做成过一批的历史（那一步读过单元头），在那一步发出的头一次读上摆一次读错（只坏这一次调用，下一次照常读得出），重跑这段历史：
/// 注入落在整理那一步上，返回错误而不是 panic。注入之后重开落不落在模型允许的版本、有没有「已知红」之外的新发现，归故障注入入口点名这一组比重
/// 跑（`fault_injection_returns_errors_instead_of_panicking` 设 `SINGLEFS_FAULT_INJECTION_WEIGHTS=broad-compaction`）；这里把计数与新发现打出来，不判。
#[test]
fn a_transient_read_error_lands_on_a_compaction_step_of_the_weights_the_campaign_entry_names_broad_compaction(
) {
    let campaign = fault_injection_campaign_from(&|variable: &str| {
        (variable == "SINGLEFS_FAULT_INJECTION_WEIGHTS").then(|| "broad-compaction".to_string())
    });
    let (history, compaction_step) = (0..SEEDS_SEARCHED_FOR_A_COMPACTION_BATCH)
        .find_map(|offset| {
            let history = generate_history_with_weights(
                HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + offset),
                DEFAULT_BATCH_OPERATIONS_PER_HISTORY,
                &campaign.weights,
            );
            let run = execute_history_with(
                &history,
                HANDING_EVERY_STEP_IMAGE_OUT,
                &SharedStream::new(),
                &mut |_| {},
            );
            let compaction_step = run.outcomes.iter().position(|outcome| {
                matches!(
                    outcome,
                    StepOutcome::Applied(AppliedEffect::Compacted {
                        batches_published,
                        ..
                    }) if *batches_published > 0
                )
            })?;
            Some((history, compaction_step))
        })
        .unwrap_or_else(|| {
            panic!(
                "比重 {} 在种子基起 {SEEDS_SEARCHED_FOR_A_COMPACTION_BATCH} 段里没有一段整理做成过一批：这一组比重不抽整理",
                campaign.weights.name
            )
        });
    let compaction_segment = FaultedSegment::Operation {
        step_index: compaction_step,
        operation_kind: HistoryOperationKind::CompactWhileMounted,
    };
    let injection = inject_one_fault_into_the_segment(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        InjectedFault::ReadFails,
        compaction_segment,
        1,
    );
    let rendered = injection.tally.render();
    print_uncaptured(&format!(
        "── 瞬时读错撞上整理（种子 {}、第 {compaction_step} 步）──\n{rendered}新发现 {} 条\n{}",
        history.seed.0,
        injection.new_findings.len(),
        injection
            .new_findings
            .iter()
            .map(|finding| finding.render())
            .collect::<String>()
    ));
    assert_eq!(
        injection
            .drawn_faults
            .iter()
            .map(|drawn| (drawn.fault, drawn.segment))
            .collect::<Vec<_>>(),
        vec![(InjectedFault::ReadFails, compaction_segment)],
        "注入点就是整理那一步的头一次读\n{rendered}"
    );
    assert_eq!(
        injection
            .tally
            .faults_on_segment(FaultedSegmentKind::Operation(
                HistoryOperationKind::CompactWhileMounted
            )),
        1,
        "这一次注入记在整理那一步上\n{rendered}"
    );
    assert_eq!(
        injection.tally.faults_that_panicked, 0,
        "注入之后 core panic 了：被测的性质是返回错误而不是 panic\n{rendered}"
    );
}

/// 默认批撞出的一段，单独跑：种子 7463871032432355126，规模与执行方式同默认批。抽出的注入点里有第 17 步（覆盖写）上一次读回坏字节：
/// 读到只坏一份，那次改动发成之后会话另发一批搬迁修它（D4（校验和位置） 已定项 4「可写挂载期间别的读」），这一步写两条根、实现走到比改动那一版高一个 txg 的那一版。
/// 执行器与模型把那一批当成改动之后的又一次发布认下（`history::settle_what_the_session_published_for_the_relocation_batch`）：不报新发现，模型比过那一批一次。
/// 判别力：执行器拿会话的现行那一版当改动那一版比时，这一段报「改写或新增的分配记录的代不是这次发布的 txg」；模型那一次比数为 0 就是这一段走的不是这条路。
#[test]
fn a_corrupted_read_during_an_overwrite_makes_a_relocation_batch_the_executor_and_the_model_recognize(
) {
    let history = generate_history_with_weights(
        HistorySeed(7_463_871_032_432_355_126),
        DEFAULT_BATCH_OPERATIONS_PER_HISTORY,
        &GenerationWeights::BROAD,
    );
    let injection = inject_faults_into_history(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        DEFAULT_BATCH_FAULTS_PER_HISTORY,
    );
    let rendered = injection.tally.render();
    assert_eq!(
        injection.tally.faults_that_panicked, 0,
        "注入之后 core panic 了\n{rendered}"
    );
    assert!(
        injection.new_findings.is_empty(),
        "「已知红」清单外的失败：{}\n{rendered}",
        injection
            .new_findings
            .iter()
            .map(|finding| finding.render())
            .collect::<String>()
    );
    let corrupted_read_on_the_overwrite = injection
        .drawn_faults
        .iter()
        .find(|drawn| {
            matches!(drawn.fault, InjectedFault::ReadReturnsCorruptedBytes { .. })
                && drawn.segment
                    == FaultedSegment::Operation {
                        step_index: 17,
                        operation_kind: HistoryOperationKind::PublishOverwrite,
                    }
        })
        .unwrap_or_else(|| {
            panic!(
                "这一段抽出的注入点里要有第 17 步覆盖写上那一次读回坏字节：{:?}",
                injection
                    .drawn_faults
                    .iter()
                    .map(DrawnFault::render)
                    .collect::<Vec<_>>()
            )
        });
    let plan = SharedFaultPlan::armed(
        HANDING_EVERY_STEP_IMAGE_OUT.device_width.fixed_geometry(),
        FaultSchedule::the_nth_call_across_the_pool(
            corrupted_read_on_the_overwrite.fault,
            corrupted_read_on_the_overwrite.call_ordinal,
        ),
    );
    let run = execute_history_with_faults(
        &history,
        HANDING_EVERY_STEP_IMAGE_OUT,
        &SharedStream::new(),
        &plan,
        &mut |_| {},
    );
    assert_eq!(
        run.tally
            .model_counts
            .relocation_batches_after_a_user_change_matched,
        1,
        "这一读坏了之后会话另发一批，模型比过它一次：{}；历史收尾 {:?}",
        corrupted_read_on_the_overwrite.render(),
        run.ending
    );
}

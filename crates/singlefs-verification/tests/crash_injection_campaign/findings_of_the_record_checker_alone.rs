//! 只由记录核对器判出的发现：有根而两条记录都没有、发布的数据单元没落盘、根已经离开根环的发布缺单元；另一台主机交占位结论时记录核对器的阳性对照不跑。

use super::*;

/// 崩溃镜像那一截（只读恢复、模型、池级 checker、记录核对器）判红记下的那一条新发现；没有时 panic，带上全部新发现。
fn finding_on_the_crash_image(injection: &HistoryCrashInjection) -> &CrashPointFinding {
    injection
        .new_findings
        .iter()
        .find(|finding| finding.stage == CrashStateStage::CrashImage)
        .unwrap_or_else(|| panic!("崩溃镜像那一截没有判红：{:#?}", injection.new_findings))
}

/// 记录核对器第一条判据（根在而那次发布的记录一条都不在）接进崩溃注入之后真的判得出东西（K3-④）：
/// 第 22 段起数 3 个写（(1, 5) 的两份 journal 记录与它的根槽写），两份记录扣下、根槽写持久——越过了记录与根之间那道屏障，
/// 屏障守住时摆不出来。恢复落到 (1, 5)、读回它的内容，模型认、池级 checker 不红（在 harness 侧照同一个持久集合现量的），
/// 这个崩溃状态只有记录核对器判得出。把崩溃镜像那一截通过条件里的 `record_check == RecordCheck::default()` 丢掉、
/// 计数照记（攻方变异 A，`crates/mutations.tsv`），这一截就没有新发现，这一条红。
#[test]
fn a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone()
{
    print_not_run_when_the_targeted_case_was_not_judged(
        the_root_without_either_of_its_records_is_found_by_the_record_checker_alone(
            given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
        ),
        "a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone",
    );
}

/// 同 [`a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone`]，那一个靶向状态交起在一份不开 GPU 的配置上的引擎
/// （[`given_states_engine_on_a_configuration_without_the_gpu`]：只由 CPU 判器判），收尾核这一趟是 CPU 判器判的。
/// 判别力（`crates/mutations.tsv`「实审 B3c-1：崩溃注入第一截交给记录核对器的写表少了崩溃窗口末尾那个写」）：CPU 判器交记录核对器的写表截掉落了盘的根槽写，
/// 「根在而记录一条都不在」判不出，这条红。
#[test]
fn on_a_configuration_without_the_gpu_a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone(
) {
    let engine_on_the_cpu = given_states_engine_on_a_configuration_without_the_gpu(
        "crash_injection_campaign_targeted_history_on_the_cpu",
    );
    let outcome = the_root_without_either_of_its_records_is_found_by_the_record_checker_alone(
        engine_on_the_cpu.engine(),
    );
    finish_the_engine_asserting_it_judged_on_the_cpu(engine_on_the_cpu);
    assert_eq!(
        outcome,
        TargetedCaseOutcome::Judged,
        "引擎起在清掉分片与选段变量的环境里，这个靶向状态这一趟就判"
    );
}

/// [`a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone`] 那一个靶向状态交 `judge` 判：
/// 引擎交了占位结论就不判、交回为什么（[`why_the_placeholder_conclusions_leave_this_run_unjudged`]）。
fn the_root_without_either_of_its_records_is_found_by_the_record_checker_alone(
    judge: &dyn GivenStatesJudge,
) -> TargetedCaseOutcome {
    let history = history_of_a_first_file_and_two_overwrites();
    let (writes, segments) = writes_and_segments_of(&history);
    let first_write = segments[23][0];
    assert_eq!(
        writes[first_write..first_write + 3]
            .iter()
            .map(|write| write.kind)
            .collect::<Vec<_>>(),
        vec![
            StepKind::JournalRecord,
            StepKind::JournalRecord,
            StepKind::RootRecordFua
        ],
        "第 22 段起的 3 个写是一次发布的两份 journal 记录与它的根槽写"
    );
    assert_eq!(
        root_identity_of_a_root_slot_write(&writes[first_write + 2]),
        Some((InstanceGeneration(1), CheckpointTxg(5))),
        "那个根槽写是第二次覆盖写 (1, 5) 的"
    );

    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: 23,
            writes_from_the_segment_on: 3,
            withheld_among_them: &[0, 1],
        },
        None,
        judge,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：记录核对器第一条判据的靶向崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    if let Some(reason) = why_the_placeholder_conclusions_leave_this_run_unjudged(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
    ) {
        return TargetedCaseOutcome::NotRunOnPlaceholderConclusions { reason };
    }
    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
    assert_eq!(
        injection.tally.record_root_without_record, 1,
        "记录核对器在这个崩溃状态上判出「根在而它那次发布的记录一条都不在」"
    );
    let finding = finding_on_the_crash_image(&injection);
    assert_eq!(
        finding.signature,
        FailureSignature::RecordCheck {
            aspects: vec!["root_without_record"]
        },
        "崩溃镜像那一截的新发现记在记录核对器头上：{finding:#?}"
    );
    assert!(
        finding.observation.violations.is_empty()
            && finding.observation.model_disagreement.is_none(),
        "池级 checker、RefFS 与模型都不红，这一条新发现只来自记录核对器：{finding:#?}"
    );
    TargetedCaseOutcome::Judged
}

/// 记录核对器第二条判据（恢复自称的那一版不早于某次发布、而那次发布写出的某个单元两份都不在）接进崩溃注入之后真的判得出东西（K3-⑨）：
/// 第 13 段起数 56 个写（(1, 3) 的单元写起、到 (1, 4) 的根槽写为止），扣下 (1, 3) 的数据单元两份、别的都持久——
/// 越过了 (1, 3) 自己单元与记录之间、记录与根之间两道屏障。恢复落到 (1, 4)（它的根槽写持久了），读回它的内容，模型认。
/// (1, 3) 的数据单元已被 (1, 4) 换下、还没被复用，盘上那两处一直是空的，记录核对器判「恢复自称新态而单元缺席」。
/// 照常点名验证的恢复下，只扣单元不越过根的状态落回上一版，这一条判据判不到（K3-⑨ 调查），所以这里让根也持久。
/// 池级 checker 也红（根环里还留着 (1, 3) 的根，从它出发的遍历读不到那个数据单元），签名照「记录核对器先于 checker」取记录核对器；
/// 只有记录核对器红的那一个状态是下一条用例。
/// 把崩溃镜像那一截交给记录核对器的「恢复落到的那一版」换成 None（`crates/mutations.tsv`），第二条判据的前置不成立，这一条红。
#[test]
fn a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker(
) {
    print_not_run_when_the_targeted_case_was_not_judged(
        the_claimed_state_missing_a_data_unit_is_found_by_the_record_checker(
            given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
        ),
        "a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker",
    );
}

/// 同 [`a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker`]，那一个靶向状态交起在一份不开 GPU 的配置上的引擎
/// （[`given_states_engine_on_a_configuration_without_the_gpu`]：只由 CPU 判器判），收尾核这一趟是 CPU 判器判的。
/// 判别力（`crates/mutations.tsv`「增补 3 第 3 件第 1 题：崩溃镜像那一截交给记录核对器的恢复落到的那一版换成 None」）：CPU 判器交记录核对器的那一版是 None，
/// 第二条判据的前置恒不成立，「恢复自称新态而单元缺席」判不出，这条红。
#[test]
fn on_a_configuration_without_the_gpu_a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker(
) {
    let engine_on_the_cpu = given_states_engine_on_a_configuration_without_the_gpu(
        "crash_injection_campaign_targeted_history_on_the_cpu",
    );
    let outcome = the_claimed_state_missing_a_data_unit_is_found_by_the_record_checker(
        engine_on_the_cpu.engine(),
    );
    finish_the_engine_asserting_it_judged_on_the_cpu(engine_on_the_cpu);
    assert_eq!(
        outcome,
        TargetedCaseOutcome::Judged,
        "引擎起在清掉分片与选段变量的环境里，这个靶向状态这一趟就判"
    );
}

/// [`a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker`] 那一个靶向状态交 `judge` 判：
/// 引擎交了占位结论就不判、交回为什么（[`why_the_placeholder_conclusions_leave_this_run_unjudged`]）。
fn the_claimed_state_missing_a_data_unit_is_found_by_the_record_checker(
    judge: &dyn GivenStatesJudge,
) -> TargetedCaseOutcome {
    let history = history_of_a_first_file_and_two_overwrites();
    let (writes, segments) = writes_and_segments_of(&history);
    let first_write = segments[14][0];
    let withheld_copies = &writes[first_write..first_write + 2];
    assert!(
        withheld_copies
            .iter()
            .all(|write| write.kind == StepKind::UnitWrite
                && write.length_in_bytes() == DATA_UNIT_BYTES
                && write.offset == withheld_copies[0].offset)
            && withheld_copies[0].device != withheld_copies[1].device,
        "第 13 段的头两个写是同一个数据单元在两块盘上的两份：{withheld_copies:?}"
    );
    assert_eq!(
        writes[first_write + 2..first_write + 56]
            .iter()
            .filter_map(root_identity_of_a_root_slot_write)
            .collect::<Vec<_>>(),
        vec![
            (InstanceGeneration(1), CheckpointTxg(3)),
            (InstanceGeneration(1), CheckpointTxg(4))
        ],
        "从第 13 段起数的 56 个写里有 (1, 3) 与 (1, 4) 的根槽写，最后一个是 (1, 4) 的"
    );
    assert_eq!(
        root_identity_of_a_root_slot_write(&writes[first_write + 55]),
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "从第 13 段起数的第 56 个写是 (1, 4) 的根槽写"
    );

    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: 14,
            writes_from_the_segment_on: 56,
            withheld_among_them: &[0, 1],
        },
        None,
        judge,
    );
    print_uncaptured(&format!(
        "── 崩溃注入：记录核对器第二条判据的靶向崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    if let Some(reason) = why_the_placeholder_conclusions_leave_this_run_unjudged(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
    ) {
        return TargetedCaseOutcome::NotRunOnPlaceholderConclusions { reason };
    }
    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
    assert_eq!(
        injection.tally.record_claimed_state_missing_unit, 1,
        "记录核对器在这个崩溃状态上判出「恢复自称新态而单元缺席」"
    );
    let finding = finding_on_the_crash_image(&injection);
    assert_eq!(
        finding.signature,
        FailureSignature::RecordCheck {
            aspects: vec!["claimed_state_missing_unit"]
        },
        "崩溃镜像那一截的新发现记在记录核对器第二条判据头上：{finding:#?}"
    );
    assert!(
        finding.observation.model_disagreement.is_none(),
        "恢复落到 (1, 4) 并读回它的内容，模型认：{finding:#?}"
    );
    TargetedCaseOutcome::Judged
}

/// 两台分片的那一趟（崩溃点重放入口双机跑时给本机设 `SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2`、给第二台设 `1/2`），照别的机器还没判完的样子造：
/// 环境守卫里照一份单机 CPU 配置、两份分片各起一个引擎（各一份起跑是空的一次性库，库里没有可复用的结论），各判记录核对器那两条阳性对照的
/// 靶向状态。每张给定状态表恰好归一台，所以每条阳性对照在两份里至少有一份拿到占位结论：那一份不判、交回为什么（用例自己跑时打「本次未跑」，
/// 这一条不打）；判了的那一份阳性照判（过了断言才交回「判了」）。
/// 判别力：断言阳性之前不核归别的机器判的占位（`crates/mutations.tsv` 那一行），首截表归另一台的那一份红在阳性对照的计数断言上（`left: 0 right: 1`）。
#[test]
fn the_record_checker_positive_controls_are_not_run_on_the_share_given_placeholder_conclusions_of_the_other_host(
) {
    use singlefs_verification::crash_amplification::CrashAmplificationEnvironment;
    /// 一条记录核对器阳性对照：把它那一个靶向状态交一个判器判。
    type RecordCheckerPositiveControl = fn(&dyn GivenStatesJudge) -> TargetedCaseOutcome;
    let (_directory, configuration) = disposable_library_with_a_configuration_on_the_cpu(
        "record-checker-positive-controls-under-two-shares",
    );
    let positive_controls: [(&str, RecordCheckerPositiveControl); 2] = [
        (
            "the_root_without_either_of_its_records_is_found_by_the_record_checker_alone",
            the_root_without_either_of_its_records_is_found_by_the_record_checker_alone,
        ),
        (
            "the_claimed_state_missing_a_data_unit_is_found_by_the_record_checker",
            the_claimed_state_missing_a_data_unit_is_found_by_the_record_checker,
        ),
    ];
    for (positive_control_name, positive_control) in positive_controls {
        let mut outcome_of_each_share = Vec::new();
        for share in ["0/2", "1/2"] {
            let library = disposable_library_of_the_campaign(&format!(
                "{positive_control_name}-share-{}",
                share.replace('/', "-of-")
            ));
            let _the_library_of_this_process = library.make_it_the_library_of_this_process();
            let environment = CrashAmplificationEnvironment::cleared_then_set(&[
                ("SINGLEFS_MULTI_HOST_CONFIG", configuration.as_os_str()),
                (
                    "SINGLEFS_CRASH_AMPLIFICATION_SHARE",
                    std::ffi::OsStr::new(share),
                ),
            ]);
            let engine = given_states_engine_under_the_callers_environment_guard(
                "crash_injection_campaign_targeted_history",
            );
            drop(environment);
            outcome_of_each_share.push((share, positive_control(&engine)));
        }
        // 不打「本次未跑」那一行：占位那一份是这一条要的样子，打了整项会被入口记成本次未跑
        assert!(
            outcome_of_each_share.iter().any(|(_, outcome)| matches!(
                outcome,
                TargetedCaseOutcome::NotRunOnPlaceholderConclusions { .. }
            )),
            "{positive_control_name}：每张表恰好归一台，两份里至少有一份拿到占位结论、不判：{outcome_of_each_share:?}"
        );
    }
}

/// 第一个文件之后接着覆盖写的次数：根环三个区域各 8 槽，第 24 次覆盖写 (1, 27) 的根槽写盖掉 (1, 3) 的那一槽。
const OVERWRITES_THAT_TURN_THE_ROOT_RING_PAST_THE_FIRST_FILE: u64 = 24;

/// 第一个文件 (1, 3) 之后覆盖写 24 次：根环转过一圈，(1, 3) 的根槽已被 (1, 27) 盖掉。
fn history_of_a_first_file_and_overwrites_that_turn_the_root_ring() -> GeneratedHistory {
    GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: (0..OVERWRITES_THAT_TURN_THE_ROOT_RING_PAST_THE_FIRST_FILE)
            .map(|overwrite_index| overwrite_choosing(5 + 2 * overwrite_index))
            .collect(),
    }
}

/// 记录核对器第二条判据只有它自己判得出的那一个状态（第二批报告 `research/prompts/m2-rest-batch2/report.md`「停下交主 agent 的设计问题」
/// 第 2 条说找不到：缺单元的发布只要它的根还在环里，从那条根出发的 checker 遍历就会红）。根环转过一圈之后 (1, 3) 的根不在环里了：
/// 第 13 段起到流尾的每一个写都持久，只扣下第 13 段段内第 4、5 个写——(1, 3) 那一片 inode 树叶容器在两块盘上的两份（它已被 (1, 4)
/// 换下，之后没有一个写落在那两处）。恢复落到 (1, 27)、读回它的内容，模型认，池级 checker 不红（在 harness 侧照同一个持久集合现量：
/// 覆盖写 16 次时 (1, 3) 的根还在环里，checker 红 I-2.1、I-4.8、I-7.4；24 次时不红）；记录核对器判 (1, 3) 那个单元两份都缺席。
/// RefFS 的崩溃时刻取持久窗口最后一个写所在那一段（流尾那一次覆盖写），读回最后一版的内容合它。
/// 把崩溃镜像那一截通过条件里的 `record_check == RecordCheck::default()` 丢掉（攻方变异 A，`crates/mutations.tsv`），这一截就没有新发现，这一条红。
#[test]
fn a_crash_state_missing_a_unit_of_a_publish_whose_root_left_the_ring_is_a_finding_of_the_record_checker_alone(
) {
    let history = history_of_a_first_file_and_overwrites_that_turn_the_root_ring();
    let (writes, segments) = writes_and_segments_of(&history);
    let first_write = segments[14][0];
    let withheld_copies = [&writes[first_write + 4], &writes[first_write + 5]];
    assert!(
        withheld_copies
            .iter()
            .all(|write| write.kind == StepKind::UnitWrite
                && write.offset == withheld_copies[0].offset)
            && withheld_copies[0].device != withheld_copies[1].device,
        "第 13 段段内第 4、5 个写是同一个单元在两块盘上的两份：{withheld_copies:?}"
    );
    let first_file_root = writes
        .iter()
        .position(|write| {
            root_identity_of_a_root_slot_write(write)
                == Some((InstanceGeneration(1), CheckpointTxg(3)))
        })
        .expect("流里有第一个文件 (1, 3) 的根槽写");
    assert!(
        first_write + 5 < first_file_root,
        "扣下的两份是 (1, 3) 那次发布的单元写（在它的根槽写之前）"
    );
    assert!(
        writes[first_file_root + 1..].iter().any(|write| {
            write.kind == StepKind::RootRecordFua
                && write.device == writes[first_file_root].device
                && write.offset == writes[first_file_root].offset
        }),
        "(1, 3) 的根槽之后被别的根盖掉了：根环转过一圈"
    );
    assert!(
        writes[first_write + 6..].iter().all(|later| withheld_copies
            .iter()
            .all(|withheld| later.device != withheld.device || later.offset != withheld.offset)),
        "扣下的那两处之后没有一个写落上去：缺席没有更晚的写来解释"
    );
    let writes_from_the_segment_to_the_end = writes.len() - first_write;
    assert_eq!(
        writes_from_the_segment_to_the_end, 793,
        "从第 13 段的第一个写数到流尾共 793 个写"
    );

    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        CrashPointDraw::ExactlyOneCrashState {
            segment_index: 14,
            writes_from_the_segment_on: 793,
            withheld_among_them: &[4, 5],
        },
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    print_uncaptured(&format!(
        "── 崩溃注入：只有记录核对器判得出的第二条判据靶向崩溃状态 ──\n{}",
        injection.tally.render()
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
        "a_crash_state_missing_a_unit_of_a_publish_whose_root_left_the_ring_is_a_finding_of_the_record_checker_alone",
    ) {
        return;
    }
    assert_eq!(injection.crash_points.len(), 1, "只摆写死的那一个崩溃状态");
    assert_eq!(
        injection.tally.record_claimed_state_missing_unit, 1,
        "记录核对器在这个崩溃状态上判出「恢复自称新态而单元缺席」"
    );
    let finding = finding_on_the_crash_image(&injection);
    assert_eq!(
        finding.signature,
        FailureSignature::RecordCheck {
            aspects: vec!["claimed_state_missing_unit"]
        },
        "崩溃镜像那一截的新发现记在记录核对器第二条判据头上：{finding:#?}"
    );
    assert!(
        finding.observation.violations.is_empty()
            && finding.observation.model_disagreement.is_none(),
        "池级 checker、RefFS 与模型都不红，这一条新发现只来自记录核对器：{finding:#?}"
    );
}

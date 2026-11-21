//! 从库里读回的结论重建崩溃点重放的计数与发现。

use super::*;

fn red_item_of(text: &str) -> Result<RedItem, String> {
    let (pass, rest) = text
        .split_once(':')
        .ok_or_else(|| format!("库里的红项 {text:?} 不是 <pass>:<名字> 的写法"))?;
    match pass {
        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_CONSULTED_ORACLE => {
            CrashPointReplayOracleViolationKind::from_name(rest)
                .map(RedItem::ConsultedOracle)
                .ok_or_else(|| format!("oracle 违例种类认不得：{text:?}"))
        }
        CRASH_POINT_REPLAY_RED_PASS_JOURNAL_IGNORED_ORACLE => {
            CrashPointReplayOracleViolationKind::from_name(rest)
                .map(RedItem::IgnoredOracle)
                .ok_or_else(|| format!("oracle 违例种类认不得：{text:?}"))
        }
        CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER => IMPLEMENTED_INVARIANTS
            .iter()
            .find(|invariant| **invariant == rest)
            .map(|invariant| RedItem::PoolChecker(invariant))
            .ok_or_else(|| format!("池级 checker 的不变量不在清单里：{text:?}")),
        CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER => match rest {
            RECORD_CHECKER_ROOT_WITHOUT_RECORD => Ok(RedItem::RecordRootWithoutRecord),
            RECORD_CHECKER_CLAIMED_STATE_MISSING_UNIT => Ok(RedItem::RecordClaimedStateMissingUnit),
            _ => Err(format!("记录核对器的判据认不得：{text:?}")),
        },
        "observer" => Ok(RedItem::Observer),
        _ => Err(format!("库里的红项 {text:?} 的 pass 认不得")),
    }
}

/// 从库里读回这条流每一块这一种结论（`kind`，每个点那一把 `version_of_each_point`）与各观察者的结论，重建崩溃点重放的计数：
/// 逐状态的红项给违例数、记录核对器两条、第一处与发现表（签名的样本原文是红项的名字：GPU 判的状态没有 CPU 那一句原文）；
/// 恢复字给两遍的结局（没有文件、读到文件、失败）与两遍不一致的状态数（按落到的根、结局码与失败点比）；计数向量给逐条不变量判到、
/// 违例、不适用与按发布分的态数；「根槽已持久」按被判根的位置事后数；观察者计数从挂在那个点上的观察者的结论块读（只挂点名段的观察者在它不挂的点上
/// 不读，[`observers_holding_the_point`]），看过的状态数是至少一个观察者挂着的点上的状态。整条流的块上观察者结论里存着的违例
/// 另交回（[`ObserverRedOfTheFlow`]）：别的机器写进服务的、库里早先存着的、这一趟看的块都读，别的机器领的块上的观察者红靠它报。
///
/// # Errors
/// 库读失败；某一块没有这一种结论（判段没写）；红项解不出；判定向量编号没登记。
pub(super) fn crash_point_replay_tally_from_the_library(
    store: &VerdictStore,
    flow: &CrashFlow<'_>,
    plan: &CrashPlan,
    kind: ConclusionKind,
    version_of_each_point: &[ConclusionVersion],
    observers: &[ObserverInTheBlockAccount],
) -> Result<(CrashPointReplayTally, ObserverRedOfTheFlow), String> {
    let positions = PositionsOfTheStates::of_the_plan(flow, plan);
    let mut tally = CrashPointReplayTally::default();
    let mut observer_red = ObserverRedOfTheFlow {
        red_states_of_each_observer: vec![0; observers.len()],
        first_red_block: None,
    };
    let invariant_named = |name: &str| -> Result<&'static str, String> {
        IMPLEMENTED_INVARIANTS
            .iter()
            .find(|invariant| **invariant == name)
            .copied()
            .ok_or_else(|| format!("计数向量里的不变量不在清单里：{name:?}"))
    };
    for (point_index, point) in plan.points.iter().enumerate() {
        let observers_of_the_point = observers_holding_the_point(observers, point_index);
        for local in point.blocks() {
            let key = point.block_key(&local);
            let stored: StoredConclusionBlock = store
                .read_conclusion_block(&key, &kind, &version_of_each_point[point_index])
                .map_err(|error| format!("读结论块失败：{error:?}"))?
                .ok_or_else(|| {
                    format!(
                        "点 {} 块 {} 在库里没有 {} 这一种结论（对账说有：库在判段里被改了）",
                        point.name,
                        local.start,
                        kind.version_text()
                    )
                })?;
            let numbers = stored.block.verdict_vector_number_of_each_state();
            let block_states = u64::try_from(numbers.len()).expect("一块的状态数装得进 u64");
            if block_states != local.end - local.start {
                return Err(format!(
                    "点 {} 块 {} 存着 {block_states} 个状态，计划里是 {}",
                    point.name,
                    local.start,
                    local.end - local.start
                ));
            }
            let recovery_words = stored.recovery_words.as_ref().ok_or_else(|| {
                format!(
                    "点 {} 块 {} 的结论没带恢复字（判器那一路存块时漏了）",
                    point.name, local.start
                )
            })?;
            if recovery_words.len() != numbers.len() {
                return Err(format!(
                    "点 {} 块 {} 的恢复字 {} 个、状态 {} 个",
                    point.name,
                    local.start,
                    recovery_words.len(),
                    numbers.len()
                ));
            }
            for (offset, (number, words)) in numbers.iter().zip(recovery_words).enumerate() {
                let ordinal =
                    point.ordinals.start + local.start + u64::try_from(offset).expect("块内序号");
                let vector = store.verdict_vector_of_number(*number).ok_or_else(|| {
                    format!("判定块里的编号 {} 没在这个库里登记过", number.as_u32())
                })?;
                let items = red_items_of_the_encoded_verdict(vector);
                tally.states += 1;
                let (segment, publish) = positions.position_of(ordinal);
                *tally.states_by_publish.entry(publish).or_insert(0) += 1;
                // 两遍各自的恢复字（落到的根连同完整的 checkpoint_txg、结局、失败点）整份比：与现判的
                // `crash_judge_conclusions::recovery_passes_differ` 同一组字
                let consulted_pass = words.consulted_pass();
                let ignored_pass = words.ignored_pass();
                match consulted_pass.outcome {
                    RECOVERY_OUTCOME_CODE_NO_FILE => tally.no_file_states += 1,
                    RECOVERY_OUTCOME_CODE_FILE_READ => tally.file_read_states += 1,
                    RECOVERY_OUTCOME_CODE_FAILED => tally.failed_states += 1,
                    other => return Err(format!("恢复字里的结局码认不得：{other}")),
                }
                if consulted_pass != ignored_pass {
                    tally.journal_differing_states += 1;
                }
                let mut is_red_state = false;
                let mut violated_invariants: Vec<&'static str> = Vec::new();
                let mut record_check = RecordCheck {
                    root_without_record: false,
                    claimed_state_missing_unit: false,
                };
                for item_text in &items {
                    match red_item_of(item_text)? {
                        RedItem::ConsultedOracle(kind_of_violation) => {
                            tally.violations += 1;
                            is_red_state = true;
                            if tally.first_violation.is_none() {
                                tally.first_violation = Some(item_text.clone());
                            }
                            tally.findings.record_red_state(
                                CrashPointReplayFindingSignature {
                                    red_pass: CrashPointReplayRedPass::JournalConsultedOracle(
                                        kind_of_violation,
                                    ),
                                    segment,
                                    publish,
                                },
                                ordinal,
                                || item_text.clone(),
                            );
                        }
                        RedItem::IgnoredOracle(kind_of_violation) => {
                            tally.ignored_violations += 1;
                            is_red_state = true;
                            if tally.first_ignored_violation.is_none() {
                                tally.first_ignored_violation = Some(item_text.clone());
                            }
                            tally.findings.record_red_state(
                                CrashPointReplayFindingSignature {
                                    red_pass: CrashPointReplayRedPass::JournalIgnoredOracle(
                                        kind_of_violation,
                                    ),
                                    segment,
                                    publish,
                                },
                                ordinal,
                                || item_text.clone(),
                            );
                        }
                        RedItem::PoolChecker(invariant) => {
                            violated_invariants.push(invariant);
                            tally
                                .checker_first_violation
                                .entry(invariant)
                                .or_insert_with(|| item_text.clone());
                        }
                        RedItem::RecordRootWithoutRecord => record_check.root_without_record = true,
                        RedItem::RecordClaimedStateMissingUnit => {
                            record_check.claimed_state_missing_unit = true;
                        }
                        RedItem::Observer => {
                            return Err(format!(
                                "判器那一种结论里出现了观察者的红项 {item_text:?}：观察者的结论另存，不该混进判器的块"
                            ));
                        }
                    }
                }
                if !violated_invariants.is_empty() {
                    is_red_state = true;
                    let text = format!(
                        "{CRASH_POINT_REPLAY_RED_PASS_POOL_CHECKER}:{}",
                        violated_invariants.join(",")
                    );
                    tally.findings.record_red_state(
                        CrashPointReplayFindingSignature {
                            red_pass: CrashPointReplayRedPass::PoolChecker(violated_invariants),
                            segment,
                            publish,
                        },
                        ordinal,
                        || text,
                    );
                }
                if record_check.root_without_record {
                    tally.record_root_without_record += 1;
                }
                if record_check.claimed_state_missing_unit {
                    tally.record_claimed_state_missing_unit += 1;
                }
                if record_check.root_without_record || record_check.claimed_state_missing_unit {
                    is_red_state = true;
                    let text = format!(
                        "{CRASH_POINT_REPLAY_RED_PASS_RECORD_CHECKER}:{}",
                        CrashPointReplayRedPass::RecordChecker(record_check)
                            .violated_names()
                            .join(",")
                    );
                    tally.findings.record_red_state(
                        CrashPointReplayFindingSignature {
                            red_pass: CrashPointReplayRedPass::RecordChecker(record_check),
                            segment,
                            publish,
                        },
                        ordinal,
                        || text,
                    );
                }
                if is_red_state {
                    tally.findings.red_states += 1;
                }
            }
            for (invariant_name, counts) in &stored.count_vector.per_invariant {
                let invariant = invariant_named(invariant_name)?;
                *tally.checker_evaluated_states.entry(invariant).or_insert(0) += counts.judged;
                *tally.checker_violated_states.entry(invariant).or_insert(0) += counts.violated;
                *tally
                    .checker_not_applicable_states
                    .entry(invariant)
                    .or_insert(0) += counts.not_applicable;
            }
            if !observers_of_the_point.is_empty() {
                tally.observed_states += block_states;
            }
            for (observer_position, (observer_kind, observer_version)) in &observers_of_the_point {
                let observer_position = *observer_position;
                let observed = store
                    .read_conclusion_block(&key, observer_kind, observer_version)
                    .map_err(|error| format!("读观察者的结论失败：{error:?}"))?
                    .ok_or_else(|| {
                        format!(
                            "点 {} 块 {} 在库里没有 {} 的结论（观察者那一路没写库）",
                            point.name,
                            local.start,
                            observer_kind.version_text()
                        )
                    })?;
                for (counter, count) in &observed.count_vector.per_observer {
                    tally.observer_counts.add(counter, *count);
                }
                let red_states_in_the_block =
                    u64::try_from(observed.violations.len()).expect("一块的违例数装得进 u64");
                observer_red.red_states_of_each_observer[observer_position] +=
                    red_states_in_the_block;
                if red_states_in_the_block > 0 && observer_red.first_red_block.is_none() {
                    observer_red.first_red_block = Some(FirstObserverRedBlock {
                        observer_position,
                        point_name: point.name.clone(),
                        block_start: local.start,
                    });
                }
            }
        }
    }
    tally.root_persisted_states = if flow
        .writes
        .get(flow.judged_root_index)
        .is_some_and(|write| write.kind == StepKind::RootRecordFua)
    {
        states_with_the_root_slot_write_persisted(flow, plan, flow.judged_root_index)
    } else {
        return Err(format!(
            "被判的第 {} 次写不是根槽写：崩溃点重放的「根槽已持久」只按根槽写两态数",
            flow.judged_root_index
        ));
    };
    if tally.states != positions.state_count {
        return Err(format!(
            "从库里读回 {} 个状态的结论，计划里整条流 {} 个",
            tally.states, positions.state_count
        ));
    }
    Ok((tally, observer_red))
}

/// 发现表：每个签名一行 `CRASH_POINT_REPLAY_FINDING event=new …`（按最先那个状态排号）、一行 `CRASH_POINT_REPLAY_FINDINGS`；设了发现日志就写一节定稿。
pub(super) fn report_the_findings(
    tally: &CrashPointReplayTally,
    item: CrashPointReplayItem<'_>,
    state_count: u64,
) {
    for (position, (signature, finding)) in tally
        .findings
        .signatures_in_first_state_order()
        .iter()
        .enumerate()
    {
        println!(
            "{}",
            findings_standard_output_new_signature_line(
                position + 1,
                signature,
                finding.first_sample()
            )
        );
    }
    println!(
        "{}",
        findings_standard_output_summary_line(&tally.findings, state_count)
    );
    let begin = CrashPointReplayFindingsLogBegin {
        stream_name: Some(item.name.to_string()),
        whole_stream_states: state_count,
    };
    if let CrashPointReplayFindingsLog::AppendedTo(path) =
        CrashPointReplayFindingsLog::from_environment()
    {
        let section = CrashPointReplayFindingsLogSection::begin(&path, &begin);
        section.replace_with_final_lines(&findings_log_final_lines(
            &begin,
            &tally.findings,
            state_count,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn red_items_stored_by_either_judge_are_read_back_into_their_passes() {
        assert!(matches!(
            red_item_of("journal_consulted_oracle:wrong_content"),
            Ok(RedItem::ConsultedOracle(_))
        ));
        assert!(matches!(
            red_item_of("journal_ignored_oracle:wrong_content"),
            Ok(RedItem::IgnoredOracle(_))
        ));
        assert!(matches!(
            red_item_of(&format!("pool_checker:{}", IMPLEMENTED_INVARIANTS[0])),
            Ok(RedItem::PoolChecker(invariant)) if invariant == IMPLEMENTED_INVARIANTS[0]
        ));
        assert!(matches!(
            red_item_of("record_checker:root_without_record"),
            Ok(RedItem::RecordRootWithoutRecord)
        ));
        assert!(matches!(
            red_item_of("record_checker:claimed_state_missing_unit"),
            Ok(RedItem::RecordClaimedStateMissingUnit)
        ));
        assert!(matches!(
            red_item_of("observer:f5:炸了"),
            Ok(RedItem::Observer)
        ));
        assert!(
            red_item_of("pool_checker:I-0.0").is_err(),
            "清单里没有的不变量要报错"
        );
        assert!(red_item_of("no_colon").is_err());
        assert!(red_item_of("unknown_pass:x").is_err());
    }
}

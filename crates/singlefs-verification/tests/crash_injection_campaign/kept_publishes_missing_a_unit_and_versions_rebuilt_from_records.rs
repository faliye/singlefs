//! 保留下来的发布缺单元：落地版本抛弃的发布不要求它的单元、保留的发布缺单元照红，崩溃恢复抛弃最新根之后整条流干净；从记录重建出来的版本免掉它的实例表抛弃的发布，缺单元的只在整条流上判、带校验的恢复落不到它上面。

use super::*;

/// 一段历史上写出的每次发布：它写出的单元写（写表下标）与它的根身份 (实例代号, checkpoint_txg)。
/// 归法与记录核对器的相同：一次根槽写之前、上一次根槽写之后的单元写都归这一次。
fn unit_writes_of_each_publish(writes: &[RetainedWrite]) -> Vec<((u32, u64), Vec<usize>)> {
    let mut publishes = Vec::new();
    let mut units = Vec::new();
    for (index, write) in writes.iter().enumerate() {
        match write.kind {
            StepKind::UnitWrite => units.push(index),
            StepKind::RootRecordFua => {
                let (instance, checkpoint_txg) =
                    root_identity_written_by(write.bytes().expect("根槽写带着字节"));
                publishes.push(((instance.0, checkpoint_txg.0), std::mem::take(&mut units)));
            }
            StepKind::JournalRecord
            | StepKind::ZeroFill
            | StepKind::Discard
            | StepKind::SystemConfigurationSlot
            | StepKind::Barrier => {}
        }
    }
    publishes
}

/// 这次单元写写下的字节还在不在 `image` 上。
fn unit_write_is_still_on_disk(image: &CrashImage<'_>, write: &RetainedWrite) -> bool {
    let length = usize::try_from(write.length_in_bytes()).expect("写长装得进 usize");
    PoolReader::read(image, write.device, write.offset, length).as_deref() == write.bytes()
}

/// 记录核对器不要求被抛弃时间线上那次发布的单元还在（实七报告 (c)，实八；复现的随机种子是崩溃注入种子基 + 2）：
/// 第一个文件（txg 3）之后覆盖写一次（txg 4），崩溃恢复抛弃它（新实例 2 写行 (1, 3, 0)），再覆盖写几次——实例 2 看不见 (1, 4)，
/// 它的数据单元那一对槽被后来的发布复用。整条流都落了盘的那个状态上，恢复落到最新那一版，它的实例表判 (1, 4) 被抛弃
/// （行 (1, 3) 且 4 > 3，D23（journal 的角色与格式） 已定项 14），(1, 4) 写出的单元不算「该在」，记录核对器不判红。
/// 判别力的两半：同一条流上把没被抛弃的发布的一个单元的两份都扣下，照判红——一次是 (1, 3)（行 (1, 3) 的边上，T = Ti 不算被抛弃），
/// 一次是最后那次覆盖写。
///
/// **C554 乙之后「崩溃恢复抛弃根」这一步造不出被抛弃的根**（改钉「这一形不可达」）：(1, 4) 是这个会话发的，
/// 系统配置见证过它；这一步让它的根槽与单元读回全 0 之后，可写挂载读阶段判据 N-配置 为真、重读一次仍为真，按 C554 乙拒
/// （`MountError::NewerStateStillUnreadableAfterOneReread`，`history.rs` 的 `apply_crash_recovery_abandoning_the_newest_root` 文档写着），
/// 不开新实例、不写抛弃行，后面三次覆盖写没有可写会话、都不适用。没有 C554 乙时这一条绿，
/// 今天的底座上把 C577 那道屏障去掉它照样造不出（探针量过，合入后验证一报告）：不可达是 C554 乙定的，不是 C577。
/// 这一条保留原来的构造（同一串步），改钉今天的样子：整条流都落了盘的那个状态上恢复落到 (1, 4)（实例 1），流里没有实例 2 的发布，
/// 记录核对器不判红；判别力那两次扣单元照旧，扣的是 (1, 3) 与流里最后一次发布 (1, 4)。把 C554 乙那一判关掉，挂载照旧做成、落到实例 2，
/// 红在「落到实例 1」那一条（`crates/mutations.tsv`）。三态（全落、扣 (1, 3) 的、扣 (1, 4) 的）一张给定状态表交崩溃点重放的引擎判
/// （两段：表先写进判定库、读回再判，配了 GPU 走 GPU）；「落到哪一版」那几条前提是实现的恢复，照旧在 CPU 上跑。
// crash-case-check:not-a-crash-case 一段写死的短历史上摆三个给定状态（全落、扣两个数据单元）作一张给定状态表交判器判，核记录核对器不要求被抛弃时间线上那次发布的单元，不调枚举一族函数
#[test]
fn units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red(
) {
    print_not_run_when_the_targeted_case_was_not_judged(
        the_whole_stream_is_clean_and_each_kept_publish_missing_a_unit_is_red_after_the_crash_recovery_abandoning_the_newest_root(
            given_states_engine_for("crash_recovery_abandoning_the_newest_root_then_three_overwrites")
                .engine(),
        ),
        "units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red",
    );
}

/// [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`] 那段历史与那三态
/// （全落、扣 (1, 3) 的数据单元、扣流里最后一次发布的数据单元）作一张给定状态表交 `judge` 判，断言照那条用例：
/// 全落那一态记录核对器一条都不红，扣单元的两态第二条判据红。引擎交了占位结论就不判、交回为什么。
fn the_whole_stream_is_clean_and_each_kept_publish_missing_a_unit_is_red_after_the_crash_recovery_abandoning_the_newest_root(
    judge: &dyn GivenStatesJudge,
) -> TargetedCaseOutcome {
    let overwrite = |fill_seed: u64| {
        HistoryOperation::PublishOverwrite(ContentChoice {
            length: ContentLength::InsideOneDataUnit { selector: 2999 },
            fill_seed,
        })
    };
    let history = GeneratedHistory {
        seed: HistorySeed(0),
        starting_point: HistoryStartingPoint::AfterFirstFile,
        operations: vec![
            overwrite(1),
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot,
            overwrite(2),
            overwrite(3),
            overwrite(4),
        ],
    };
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    let run = execute_history_with(&history, execution, &stream, &mut |_| {});
    assert_eq!(run.ending, HistoryEnding::Completed, "这段历史跑完");
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let base = MemoryPool::with_devices(
        &[DeviceIdentity(0), DeviceIdentity(1)],
        execution.device_width.device_bytes(),
    );
    let publishes = unit_writes_of_each_publish(&writes);
    let units_of = |identity: (u32, u64)| -> &Vec<usize> {
        &publishes
            .iter()
            .find(|(written, _)| *written == identity)
            .unwrap_or_else(|| panic!("流里有 {identity:?} 那次发布：{publishes:?}"))
            .1
    };
    let every_write_landed = CrashImage {
        base: &base,
        writes: &writes,
        persisted: vec![true; writes.len()],
    };
    let report = recover(&every_write_landed, JournalPolicy::Consult);
    assert_eq!(
        report.effective_root,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "C554 乙之后崩溃恢复抛弃根那一步的可写挂载被拒：落到实例 1 的 (1, 4)，没有实例 2（C554 乙之前落到实例 2、比 (1, 4) 新的一版）"
    );
    assert!(
        publishes.iter().all(|((instance, _), _)| *instance <= 1),
        "流里只有 mkfs（实例 0）与实例 1 的发布、没有实例 2 的：{publishes:?}"
    );

    let last_overwrite = publishes.last().expect("流里至少一次发布").0;
    let kept_publishes = [(1, 3), last_overwrite];
    // 整条流都落了盘的那一态，再接两态：没被抛弃的那两次发布各扣下一个数据单元的两份（两块盘各一份）
    let mut persisted_sets = vec![vec![true; writes.len()]];
    for kept in kept_publishes {
        let data_unit = *units_of(kept)
            .iter()
            .find(|unit| writes[**unit].length_in_bytes() == DATA_UNIT_BYTES)
            .unwrap_or_else(|| panic!("{kept:?} 那次发布写了数据单元"));
        let mut persisted = vec![true; writes.len()];
        for unit in units_of(kept) {
            if writes[*unit].offset == writes[data_unit].offset {
                persisted[*unit] = false;
            }
        }
        let recovery_with_the_unit_withheld = recover(
            &CrashImage {
                base: &base,
                writes: &writes,
                persisted: persisted.clone(),
            },
            JournalPolicy::Consult,
        );
        assert!(
            recovery_with_the_unit_withheld
                .effective_root
                .is_some_and(|(_, txg)| txg.0 >= kept.1),
            "恢复落到的那一版不早于 {kept:?}：{:?}",
            recovery_with_the_unit_withheld.effective_root
        );
        persisted_sets.push(persisted);
    }
    // 三态一张给定状态表交崩溃点重放的引擎判（两段：表先写进判定库、读回再判，配了 GPU 走 GPU；能走两段的都走两段）
    let history_after_the_make_filesystem = HistoryAfterTheMakeFilesystem::of(
        &writes,
        &segments,
        &stream_indexes,
        run.operations_written_by_make_filesystem,
        execution,
    );
    let conclusions = judge
        .judge_the_given_states(&GivenStatesOfATable {
            table_name: "every-write-landed-and-a-kept-unit-withheld".to_string(),
            base: history_after_the_make_filesystem.base_pool(),
            writes: history_after_the_make_filesystem.writes(),
            segments: history_after_the_make_filesystem.segments(),
            seams: &[],
            persisted_sets: persisted_sets
                .iter()
                .map(|persisted| {
                    history_after_the_make_filesystem.persisted_set_of_the_table(persisted)
                })
                .collect(),
        })
        .unwrap_or_else(|reason| panic!("引擎判得了这张表：{reason}"));
    if let Some(reason) = why_the_placeholder_conclusions_leave_this_run_unjudged(
        PlaceholderTablesOfTheRun::of_one_table(conclusions.source),
    ) {
        return TargetedCaseOutcome::NotRunOnPlaceholderConclusions { reason };
    }
    assert_eq!(
        conclusions.states.len(),
        1 + kept_publishes.len(),
        "判器逐状态交回结论"
    );
    assert_eq!(
        conclusions.states[0].record_check(),
        RecordCheck::default(),
        "整条流都落了盘：记录核对器一条都不判红"
    );
    for (kept, conclusion) in kept_publishes.iter().zip(&conclusions.states[1..]) {
        assert!(
            conclusion.record_check().claimed_state_missing_unit,
            "{kept:?} 没被抛弃，它的数据单元两份都没落，照判红"
        );
    }
    TargetedCaseOutcome::Judged
}

/// 同 [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`]，给定状态表交
/// 起在一份不开 GPU 的配置上的引擎（[`given_states_engine_on_a_configuration_without_the_gpu`]：只由 CPU 判器判），收尾核这一趟是 CPU 判器判的。
/// 判别力（`crates/mutations.tsv`「实审 B3c-1：记录核对器在实例表读得出时把每次发布都当被抛弃」）：恢复落到的 (1, 4) 的实例表读得出，
/// 每次发布都当被抛弃，扣单元的两态不再判红，这条红。
// crash-case-check:not-a-crash-case 与 units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red 那一条同样的三个给定状态，交起在不开 GPU 的配置上的引擎（只由 CPU 判器判），核这一趟是 CPU 判器判的；不调枚举一族函数
#[test]
fn on_a_configuration_without_the_gpu_the_whole_stream_is_clean_and_each_kept_publish_missing_a_unit_is_red_after_the_crash_recovery_abandoning_the_newest_root(
) {
    let engine_on_the_cpu = given_states_engine_on_a_configuration_without_the_gpu(
        "crash_recovery_abandoning_the_newest_root_then_three_overwrites_on_the_cpu",
    );
    let outcome =
        the_whole_stream_is_clean_and_each_kept_publish_missing_a_unit_is_red_after_the_crash_recovery_abandoning_the_newest_root(
            engine_on_the_cpu.engine(),
        );
    finish_the_engine_asserting_it_judged_on_the_cpu(engine_on_the_cpu);
    assert_eq!(
        outcome,
        TargetedCaseOutcome::Judged,
        "引擎起在清掉分片与选段变量的环境里，这张表这一趟就判"
    );
}

/// 调查报告 `research/prompts/m2-investigate-crash-injection-reds-report.md` 第 1 条那一形所在的历史：种子基 + 2，
/// 默认批的步数与比重（按 [`DEFAULT_BATCH_DRAW`] 摆的崩溃状态也与默认批相同）。
fn history_with_a_crash_state_landing_on_a_version_rebuilt_from_records() -> GeneratedHistory {
    generate_history_with_weights(
        HistorySeed(SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE + 2),
        DEFAULT_BATCH_OPERATIONS_PER_HISTORY,
        &GenerationWeights::BROAD,
    )
}

/// 与整条流（`write_count` 个写）逐条对应的持久集合：崩溃点所在段之前整段持久、
/// 这一段按崩溃点的子集、之后的一个都没持久——崩溃注入交给记录核对器的那一种。
fn persisted_over_the_whole_stream(
    write_count: usize,
    first_write_of_the_segment: usize,
    persisted_within_the_segment: &[bool],
) -> Vec<bool> {
    let mut persisted = vec![false; write_count];
    persisted[..first_write_of_the_segment].fill(true);
    persisted[first_write_of_the_segment
        ..first_write_of_the_segment + persisted_within_the_segment.len()]
        .copy_from_slice(persisted_within_the_segment);
    persisted
}

/// 两块空盘上叠 `writes_before_the_segment` 里在持久集合 `persisted`（整条流那一份，逐条对应）里的写。
fn pool_of_the_persisted_writes_before_the_segment(
    writes_before_the_segment: &[RetainedWrite],
    persisted: &[bool],
    device_bytes: u64,
) -> MemoryPool {
    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], device_bytes);
    for (write, is_persisted) in writes_before_the_segment.iter().zip(persisted) {
        if *is_persisted {
            base.apply_writes(std::slice::from_ref(write));
        }
    }
    base
}

/// 在 `history` 的整条录制流上，按 `crash_points`（这段历史上崩溃注入摆出来的那几个）逐个重建崩溃镜像、跑恢复，
/// 挑出第一个这样的崩溃状态：恢复施加过 journal 记录、落到的那一版的根槽写不在到这一段为止的前缀里（在更晚的段），
/// 那一版的实例表读得出、抛弃了更早的一次发布，而那次发布写在某个偏移上的单元每一份都已不在盘上（被后来的发布复用盖掉）。
/// 交回落到的那一版与被它的实例表抛弃的那次发布；一个这样的崩溃状态都没有时 `None`（C554 乙之后这一形不可达，见用到它的两条用例）。
fn crash_state_landing_on_a_version_rebuilt_from_records(
    history: &GeneratedHistory,
    crash_points: &[CrashPoint],
) -> Option<((InstanceGeneration, CheckpointTxg), (u32, u64))> {
    let execution = UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES;
    let stream = SharedStream::retaining_contents();
    execute_history_with(history, execution, &stream, &mut |_| {});
    let (writes, segments) = writes_and_segments(
        &stream.retained_operations(),
        &execution.device_width.fixed_geometry(),
    );
    let publishes = unit_writes_of_each_publish(&writes);
    let device_bytes = execution.device_width.device_bytes();
    crash_points.iter().find_map(|crash_point| {
        let segment = &segments[crash_point.segment_index];
        assert_eq!(
            segment.len(),
            crash_point.persisted_within_the_segment.len(),
            "这里切出的第 {} 段与崩溃注入那一路切出的写数相同",
            crash_point.segment_index
        );
        let first_write_of_the_segment = segment[0];
        let end_of_the_segment = first_write_of_the_segment + segment.len();
        let persisted = persisted_over_the_whole_stream(
            writes.len(),
            first_write_of_the_segment,
            &crash_point.persisted_within_the_segment,
        );
        let base = pool_of_the_persisted_writes_before_the_segment(
            &writes[..first_write_of_the_segment],
            &persisted,
            device_bytes,
        );
        let image = CrashImage {
            base: &base,
            writes: &writes[first_write_of_the_segment..end_of_the_segment],
            persisted: crash_point.persisted_within_the_segment.clone(),
        };
        let report = recover(&image, JournalPolicy::Consult);
        let landed_version = report.effective_root?;
        if report.journal.prefix_applied == 0
            || writes[..end_of_the_segment]
                .iter()
                .any(|write| root_identity_of_a_root_slot_write(write) == Some(landed_version))
        {
            return None;
        }
        let root_write_of_the_landed_version = writes
            .iter()
            .rposition(|write| root_identity_of_a_root_slot_write(write) == Some(landed_version))?;
        let table = InstanceTableOfRootRecord::read(
            &image,
            writes[root_write_of_the_landed_version]
                .bytes()
                .expect("根槽写带着字节"),
        )?;
        let (abandoned_publish_with_a_unit_overwritten, _) =
            publishes
                .iter()
                .find(|((instance, checkpoint_txg), units)| {
                    let mut copies_by_offset: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
                    for unit in units {
                        copies_by_offset
                            .entry(writes[*unit].offset.0)
                            .or_default()
                            .push(*unit);
                    }
                    *checkpoint_txg <= landed_version.1 .0
                        && table.abandons(*instance, *checkpoint_txg)
                        && units.iter().all(|unit| *unit < first_write_of_the_segment)
                        && copies_by_offset.values().any(|copies| {
                            copies
                                .iter()
                                .all(|copy| !unit_write_is_still_on_disk(&image, &writes[*copy]))
                        })
                })?;
        Some((landed_version, *abandoned_publish_with_a_unit_overwritten))
    })
}

/// 调查报告 `research/prompts/m2-investigate-crash-injection-reds-report.md` 第 1 条（装置错）那一形：种子基 + 2 那段历史上摆出来的一个崩溃状态，
/// 扣下一份 journal 记录、留下另一份，恢复施加记录落到由记录重建的那一版，那一版的根槽写在更晚的段里（D23（journal 的角色与格式） 已定项 15）；
/// 那一版的实例表抛弃了更早的一次发布，那次发布的单元已被后来的发布复用盖掉。那时记录核对器按落到的那一版的实例表豁免它（实八），
/// 前提是交给它的写表里找得到那一版的根槽写（崩溃注入交整条流）。
///
/// **C554 乙之后这一形不可达**（改钉「这一形不可达」）：实例表抛弃一次发布要崩溃恢复抛弃根
/// （`CrashRecoveryAbandoningTheNewestRoot`），而那条根是这个会话发的、系统配置见证过它，下一次可写挂载读不出它时按 C554 乙拒
/// （`MountError::NewerStateStillUnreadableAfterOneReread`），没有新实例、没有抛弃行。没有 C554 乙时这一条绿；
/// 今天的底座上把 C577 那道屏障去掉照样摆不出来，不可达是 C554 乙定的。
/// 这一条保留原来的构造（同一段历史、同一批崩溃状态），改钉：这批崩溃状态里一个这样的都没有，记录核对器一条都不判。
/// 把 C554 乙那一判关掉，这一形回来，第一条断言红（`crates/mutations.tsv`）。
#[test]
fn crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons(
) {
    let history = history_with_a_crash_state_landing_on_a_version_rebuilt_from_records();
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        DEFAULT_BATCH_DRAW,
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    print_uncaptured(&format!(
        "── 崩溃注入：种子基 + 2 那段历史，C554 乙之后找「恢复落到由记录重建的一版、它的实例表抛弃了单元被复用的发布」那一形 ──\n{}",
        injection.tally.render()
    ));
    if printed_not_run_because_the_engine_gave_placeholder_conclusions(
        PlaceholderTablesOfTheRun::of_the_tally(&injection.tally),
        "crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons",
    ) {
        return;
    }
    let crash_recovery_outcomes = outcomes_of_the_crash_recovery_steps(&history);
    assert!(
        !crash_recovery_outcomes.is_empty()
            && crash_recovery_outcomes.iter().all(|outcome| *outcome
                == StepOutcome::Refused {
                    member: CRASH_RECOVERY_REFUSED_BY_THE_WITNESS.to_string()
                }),
        "这段历史里的崩溃恢复抛弃根每一步都被 C554 乙拒、一个被抛弃的根都没造出来：{crash_recovery_outcomes:?}"
    );
    assert_eq!(
        crash_state_landing_on_a_version_rebuilt_from_records(&history, &injection.crash_points),
        None,
        "C554 乙之后崩溃恢复抛弃根那一步被拒、造不出被抛弃的根：这批崩溃状态里不该有落到由记录重建、实例表抛弃了单元被复用的发布的那一版"
    );
    assert_eq!(
        injection.tally.record_checks,
        u64::try_from(injection.crash_points.len()).expect("崩溃状态数装得进 u64"),
        "每个崩溃状态都跑过记录核对器"
    );
    assert_eq!(
        injection.tally.record_claimed_state_missing_unit, 0,
        "这段历史上的崩溃状态一条「恢复自称新态而单元缺席」都不判"
    );
    assert_eq!(
        injection.tally.record_root_without_record, 0,
        "这段历史上没有「根在而它那次发布的记录一条都不在」的崩溃状态"
    );
}

/// 判别力那一半（调查报告第 1 条「推翻条件与造它的那一次」第 2 次）原来在上一条那一形上做：落到的那一版在它实例里的前一版（没被抛弃）
/// 写在某个偏移上的单元两份都扣下，照判红。**C554 乙之后那一形不可达**（见上一条），这一条改钉：用默认批的崩溃状态与同一段历史，
/// 一个那样的崩溃状态都摆不出来。「没被抛弃的发布缺单元照样判红」那一半由
/// [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`] 的扣单元那两次守着。
/// 把 C554 乙那一判关掉，这一形回来，这一条红（`crates/mutations.tsv`）。
#[test]
fn on_the_whole_stream_a_kept_publish_missing_a_unit_under_a_version_rebuilt_from_records_is_still_red(
) {
    let history = history_with_a_crash_state_landing_on_a_version_rebuilt_from_records();
    let injection = inject_crashes_into_history(
        &history,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES,
        DEFAULT_BATCH_DRAW,
        None,
        given_states_engine_for("crash_injection_campaign_targeted_history").engine(),
    );
    let crash_recovery_outcomes = outcomes_of_the_crash_recovery_steps(&history);
    assert!(
        !crash_recovery_outcomes.is_empty()
            && crash_recovery_outcomes.iter().all(|outcome| *outcome
                == StepOutcome::Refused {
                    member: CRASH_RECOVERY_REFUSED_BY_THE_WITNESS.to_string()
                }),
        "这段历史里的崩溃恢复抛弃根每一步都被 C554 乙拒、一个被抛弃的根都没造出来：{crash_recovery_outcomes:?}"
    );
    assert_eq!(
        crash_state_landing_on_a_version_rebuilt_from_records(&history, &injection.crash_points),
        None,
        "C554 乙之后这一形不可达：判别力那一半没有可做的崩溃状态"
    );
}

/// 崩溃注入第一截交给记录核对器的写表取整条历史录制流（`crash_injection.rs` 第一截那一次 `check_records_against`），
/// 与只取到崩溃点所在段为止的前缀，判出来不同只在两种崩溃状态上（推的：前缀之后的写一个都没持久，缺席判定与回收谓词只用得到
/// 持久的后写与更早的写；整条流多出来的只有根槽写在前缀之后的那几次发布，与恢复落到的那一版的根槽写）：
/// - 恢复落到由记录重建的一版、它的实例表抛弃了某次单元已被复用的发布：前缀里没有那一版的根槽写，实例表读不出，豁免落空。
///   C554 乙之后这一形摆不出来（[`crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons`]、
///   [`units_of_a_publish_the_landed_version_abandons_are_not_required_while_a_kept_publish_missing_a_unit_is_still_red`] 钉着）。
/// - 恢复落到由记录重建的一版、那一版自己的某个单元两份都不在：前缀里没有它的根槽写，那次发布归不出来，第二条判据罩不到它。
///   这一条造的就是它：第二次覆盖写 (1, 5) 的第一个数据单元两份（第 21 段头两个写）扣下，第 21 段别的单元写与第 22 段两份 journal 记录都持久，
///   第 23 段 (1, 5) 的根槽写在窗口之后、没落。验点名单元的恢复（`JournalPolicy::Consult`，崩溃注入用的就是它）不施加那条记录、落回 (1, 4)，
///   两种交法同判；只有不验点名单元的恢复（只供测试强制进入的 `JournalPolicy::ConsultWithoutNamedVerification`）落到 (1, 5)，整条流判红、前缀不判。
///
/// 所以把第一截的写表退回前缀那一条变异不是等价变异：这一条自己就造出了两种交法不同值的输入（不验点名单元的恢复落到 (1, 5)，整条流判红、前缀不判）。
/// 它是第三类「取样点不敏感」（`.claude/rules/mutation-sampling.md`「没红的三类，判据不同」）：崩溃注入的取值域里摆不出能分辨两种交法的盘面——
/// 崩溃注入恢复只用验点名单元的那一种（`JournalPolicy::Consult`），C554 乙那一判又让另一形摆不出来；补不上敏感的取样点，是因为要给崩溃注入
/// 加换恢复策略的开关（改被测代码，判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第五节「崩溃注入换恢复策略的开关」那一行），
/// 不是因为两种交法在所有输入上同值。这一条钉住取值域外那一支的两种交法确实分开、取值域里那一形两种交法同判；C554 乙那一判或点名验证哪天不在了，
/// 崩溃注入里两种交法也会分开。
/// 第二条判据不罩恰好是恢复落到的那一版的那次发布（`crates/mutations.tsv`），整条流那一判不红，这一条红。
#[test]
fn a_version_rebuilt_from_records_missing_a_unit_is_judged_only_on_the_whole_stream_and_the_verifying_recovery_never_lands_on_it(
) {
    let history = history_of_a_first_file_and_two_overwrites();
    let (writes, segments) = writes_and_segments_of(&history);
    let root_of_the_second_overwrite = segments[24][0];
    assert_eq!(
        root_identity_of_a_root_slot_write(&writes[root_of_the_second_overwrite]),
        Some((InstanceGeneration(1), CheckpointTxg(5))),
        "第 23 段的头一个写是第二次覆盖写 (1, 5) 的根槽写"
    );
    assert_eq!(
        segments[23]
            .iter()
            .map(|index| writes[*index].kind)
            .collect::<Vec<_>>(),
        vec![StepKind::JournalRecord, StepKind::JournalRecord],
        "第 22 段是 (1, 5) 的两份 journal 记录"
    );
    let first_write = segments[22][0];
    let withheld_copies = &writes[first_write..first_write + 2];
    assert!(
        withheld_copies
            .iter()
            .all(|write| write.kind == StepKind::UnitWrite
                && write.length_in_bytes() == DATA_UNIT_BYTES
                && write.offset == withheld_copies[0].offset)
            && withheld_copies[0].device != withheld_copies[1].device,
        "第 21 段的头两个写是 (1, 5) 一个数据单元在两块盘上的两份：{withheld_copies:?}"
    );
    let end_of_the_window = segments[23][0] + segments[23].len();
    assert_eq!(
        end_of_the_window, root_of_the_second_overwrite,
        "窗口（第 21、22 段）截在 (1, 5) 的根槽写之前"
    );

    let persisted_within_the_window: Vec<bool> = (first_write..end_of_the_window)
        .map(|index| index >= first_write + 2)
        .collect();
    let persisted =
        persisted_over_the_whole_stream(writes.len(), first_write, &persisted_within_the_window);
    let base = pool_of_the_persisted_writes_before_the_segment(
        &writes[..first_write],
        &persisted,
        UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES
            .device_width
            .device_bytes(),
    );
    let image = CrashImage {
        base: &base,
        writes: &writes[first_write..end_of_the_window],
        persisted: persisted_within_the_window,
    };
    // 第一截的交法：两份镜像同一份、一条录制流（这段历史没掉过电），写表与持久集合取到 `end_of_table` 为止。
    let record_check_on_the_table_up_to =
        |end_of_table: usize, effective_root: Option<(InstanceGeneration, CheckpointTxg)>| {
            check_records_against(
                &image,
                &image,
                &writes[..end_of_table],
                &persisted[..end_of_table],
                RecordStreamContinuity::OneRecording,
                effective_root,
            )
        };

    let verifying = recover(&image, JournalPolicy::Consult).effective_root;
    assert_eq!(
        verifying,
        Some((InstanceGeneration(1), CheckpointTxg(4))),
        "验点名单元的恢复不施加 (1, 5) 的记录（它点名的数据单元两份都不在），落回 (1, 4)"
    );
    assert_eq!(
        record_check_on_the_table_up_to(writes.len(), verifying),
        RecordCheck::default(),
        "落回 (1, 4)：整条流上记录核对器一条都不判"
    );
    assert_eq!(
        record_check_on_the_table_up_to(end_of_the_window, verifying),
        RecordCheck::default(),
        "落回 (1, 4)：前缀上也一条都不判，两种交法同值"
    );

    let unverifying =
        recover(&image, JournalPolicy::ConsultWithoutNamedVerification).effective_root;
    assert_eq!(
        unverifying,
        Some((InstanceGeneration(1), CheckpointTxg(5))),
        "不验点名单元的恢复施加 (1, 5) 的记录，落到由记录重建的 (1, 5)"
    );
    assert!(
        record_check_on_the_table_up_to(writes.len(), unverifying).claimed_state_missing_unit,
        "整条流里有 (1, 5) 的根槽写，那次发布归得出来，它的数据单元两份都不在：判「恢复自称新态而单元缺席」"
    );
    assert_eq!(
        record_check_on_the_table_up_to(end_of_the_window, unverifying),
        RecordCheck::default(),
        "前缀里没有 (1, 5) 的根槽写，那次发布归不出来，第二条判据罩不到它"
    );
}

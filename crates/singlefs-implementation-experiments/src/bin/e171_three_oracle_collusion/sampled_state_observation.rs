//! 取样态的观测：录入段录表、判段判表。

use super::*;

/// 一张给定的盘上池级 checker 判红的不变量名：表名、那张盘。单测的 O2 两段替身（`PoolCheckerTablesKeptInMemory`）照它判读回的盘；
/// 产品的判段是 [`GivenStatesTablesInTheLibrary`]。
#[cfg(test)]
pub(super) type PoolCheckerRedNamesOfAGivenImage<'judge> =
    dyn Fn(&str, &MemoryPool) -> Vec<String> + Sync + 'judge;

/// 录入段录下一个取样态上 O2 要判的盘：流名、序号、镜像摘要、这一态的镜像、这一态是不是 O2 阳性对照改坏的那一张。
/// 产品路径是 [`pool_checker_table_of_a_sampled_state`]；由参数给，线程对拍能在单测里换别的录法。
pub(super) type PoolCheckerTableOfASampledState<'record> =
    dyn Fn(StreamName, u64, &str, &CrashImage<'_>, bool) -> RecordedTableReference + Sync + 'record;

/// 取样中途某个状态上实现 panic（实现自崩）：那一句（`<panic 的消息>:after_ordinal_<崩之前最后判完的序号>`），与崩之前判完的那几态（按序号升序）。
/// 那几态录下的盘判段照样判：引擎在其中一态上失败时，两段之前在取样回调里先撞上的是引擎那一次（[`judge_the_recorded_pool_checker_tables`]）。
pub(super) struct ObservationsUpToACrashInAState {
    pub(super) crash: String,
    pub(super) observations: Vec<StateObservation>,
}

/// 一个连续段判完的结局：判完的那几态（连续段中途 panic 时是 panic 之前判完的），与 `sample_crash_states_on_the_cpu` 交回的整条流状态数或 panic 的消息。
struct ObservationsOfARun {
    pub(super) observations: Vec<StateObservation>,
    pub(super) ending: Result<u64, String>,
}

/// 按序号升序把取样的状态切成连续的几段，每段调一次 `judge_crash_state_range`：回调里给的那个 `CrashImage` 折回落地向量、算摘要，
/// 再算 O1-崩溃判官，O2 只在 CPU 上计时、要判的盘交 `pool_checker_table` 录下（录入段：判归判段，[`judge_the_recorded_pool_checker_tables`]），
/// 并单独计恢复的耗时。回调的持久集合另与 `crash_facts` 按序号现算的掩码比。
/// 这几段分给 `threads` 条线程（[`outcomes_of_the_runs_in_their_order`]），每态各交各的表（表名、表的内容与一条线程时相同，判定库的键不变），
/// 结果按（批号、序号）排好：除了耗时几列，与 `threads` 是几无关。
/// 某个状态上实现 panic（实现自崩）：交回 `Err`，那一句带次序上第一个 panic 的态之前最后一个判完的序号，另带 panic 之前判完的那几态。
#[allow(
    clippy::too_many_arguments,
    reason = "前五个是这条流的几样输入，其余是开关、线程数与录 O2 要判的盘的办法"
)]
// gpu-exempt: experiment-measures-the-cpu-judge-itself
pub(super) fn observe_sampled_states(
    stream: &PreparedStream,
    tables: &CrashStateEnumerationTables,
    facts: &FlowFacts,
    torn_image_index_by_write: &BTreeMap<usize, usize>,
    sampled: &SampledOrdinals,
    switches: RunSwitches,
    threads: NonZeroUsize,
    pool_checker_table: &PoolCheckerTableOfASampledState<'_>,
) -> Result<Vec<StateObservation>, ObservationsUpToACrashInAState> {
    let mut batch_of_ordinal: BTreeMap<u64, u8> = BTreeMap::new();
    for (batch_offset, batch) in sampled.batches.iter().enumerate() {
        for ordinal in batch {
            let batch_number = u8::try_from(batch_offset + 1).expect("三批");
            assert!(
                batch_of_ordinal.insert(*ordinal, batch_number).is_none(),
                "三批互不重叠"
            );
        }
    }
    let ordinals: Vec<u64> = batch_of_ordinal.keys().copied().collect();
    let mut runs: Vec<std::ops::Range<u64>> = Vec::new();
    for ordinal in ordinals {
        match runs.last_mut() {
            Some(run) if run.end == ordinal => run.end += 1,
            Some(_) | None => runs.push(ordinal..ordinal + 1),
        }
    }
    let recorded_write_count = stream.writes.len();
    let call_of_write = call_index_of_each_write(stream);
    let model_is_enabled =
        switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge);
    let pool_checker_is_enabled = switches.disabled_oracle != Some(DisabledOracle::PoolChecker);
    let controls = ObservationControls {
        switches,
        crash_judge_control_pending: AtomicBool::new(
            switches.positive_control == Some(PositiveControl::CrashJudge),
        ),
    };
    let sampling_item_names: Vec<String> = runs
        .iter()
        .map(|_run| sampling_item_name_of_this_call())
        .collect();
    let observe_a_run = |run_position: usize| -> ObservationsOfARun {
        let committed_versions = committed_versions_for_this_thread(stream);
        let mut observations_of_the_run: Vec<StateObservation> = Vec::new();
        let judged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            sample_crash_states_on_the_cpu(
                CrashPointReplayItem {
                    name: &sampling_item_names[run_position],
                    setting_file: CRASH_POINT_REPLAY_SAMPLING_ITEM.setting_file,
                },
                &stream.base,
                &stream.writes,
                &stream.segments,
                stream.judged_root_index,
                &stream.versions,
                &full_expansion,
                runs[run_position].clone(),
                &mut |ordinal, image, judgement| {
                    let batch = batch_of_ordinal[&ordinal];
                    let landing_vector = landing_vector_of(
                        &image.persisted,
                        recorded_write_count,
                        torn_image_index_by_write,
                    );
                    let touched_call = landing_vector
                        .chars()
                        .zip(&call_of_write)
                        .filter(|(landing, _)| *landing != '0')
                        .filter_map(|(_, call)| *call)
                        .max();
                    let verdict = ImplementationVerdict::of(judgement);
                    let crash_judge_started = Instant::now();
                    let crash_judge = if !model_is_enabled {
                        CrashJudgeVerdict::Off
                    } else if !crash_judge_is_applicable(touched_call, stream.model_stopped_at_call)
                    {
                        CrashJudgeVerdict::NotApplicable
                    } else {
                        crash_judge_of_the_state(
                            stream,
                            &committed_versions,
                            image,
                            &judgement.consulted,
                            batch,
                            &controls,
                        )
                    };
                    let crash_judge_nanoseconds = crash_judge_started.elapsed().as_nanos();
                    let pool_checker_control = controls.switches.positive_control
                        == Some(PositiveControl::PoolChecker)
                        && stream.name == StreamName::NewPoolFileCreation
                        && ordinal == tables.state_count - 1;
                    // O2 逐状态计时量的是 CPU 池级 checker 本身（豁免截名 experiment-measures-the-cpu-judge-itself）：CPU 这一遍只计时；
                    // 判红的名单走两段（同一张盘作一张给定状态表录下、判段交引擎判，阳性对照那一态是改坏的那一张），不在计时里
                    let pool_checker_started = Instant::now();
                    let pool_checker_timed_on_the_cpu = pool_checker_is_enabled.then(|| {
                        if pool_checker_control {
                            pool_checker_on_the_first_known_bad_image(image)
                        } else {
                            check_pool_image(image)
                        }
                    });
                    let pool_checker_nanoseconds = pool_checker_started.elapsed().as_nanos();
                    let image_digest = sha256_text(&crash_image_digest(image));
                    let pool_checker = if pool_checker_timed_on_the_cpu.is_some() {
                        PoolCheckerOfAState::RecordedForTheJudgeStage(pool_checker_table(
                            stream.name,
                            ordinal,
                            &image_digest,
                            image,
                            pool_checker_control,
                        ))
                    } else {
                        PoolCheckerOfAState::Off
                    };
                    let recovery_started = Instant::now();
                    let _ = singlefs_core::recovery::recover(
                        image,
                        singlefs_core::recovery::JournalPolicy::Consult,
                    );
                    let recovery_nanoseconds = recovery_started.elapsed().as_nanos();
                    observations_of_the_run.push(StateObservation {
                        batch,
                        ordinal,
                        segment_index: tables
                            .state_ranges_by_segment
                            .partition_point(|state_range| state_range.end <= ordinal),
                        landing_vector,
                        image_digest,
                        verdict,
                        judge_nanoseconds: crash_judge_nanoseconds
                            + pool_checker_nanoseconds
                            + recovery_nanoseconds,
                        mask_matches_the_facts: facts.persisted_writes_of_state(ordinal)
                            == image.persisted,
                        touched_call,
                        crash_judge,
                        pool_checker,
                        consulted: RecoverySummary::of(&judgement.consulted),
                        ignored: RecoverySummary::of(&judgement.ignored),
                        crash_judge_nanoseconds,
                        pool_checker_nanoseconds,
                        recovery_nanoseconds,
                    });
                },
            )
            .judged()
            .expect("实验装置不分只录、只判两趟跑：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE")
        }));
        ObservationsOfARun {
            observations: observations_of_the_run,
            ending: judged.map_err(|payload| panic_text(&*payload)),
        }
    };
    eprintln!(
        "E171_SAMPLED_STATES stream={} runs={} threads={threads}",
        stream.name.directory_name(),
        runs.len()
    );
    let mut observations: Vec<StateObservation> = Vec::new();
    for observed in outcomes_of_the_runs_in_their_order(
        runs.len(),
        threads,
        &|observed: &ObservationsOfARun| observed.ending.is_err(),
        &observe_a_run,
    ) {
        observations.extend(observed.observations);
        match observed.ending {
            Ok(state_count_of_the_stream) => assert_eq!(
                state_count_of_the_stream, tables.state_count,
                "judge_crash_state_range 交回的整条流状态数与枚举表的相同"
            ),
            Err(what) => {
                let crash = format!(
                    "{what}:after_ordinal_{}",
                    observations
                        .last()
                        .map_or_else(|| "none".to_string(), |last| last.ordinal.to_string())
                );
                return Err(ObservationsUpToACrashInAState {
                    crash,
                    observations,
                });
            }
        }
    }
    observations.sort_by_key(|observation| (observation.batch, observation.ordinal));
    Ok(observations)
}

/// 判段判完一条流录下的盘。
pub(super) enum JudgedTablesOfAStream {
    /// 录下的盘都判完了（这条流没有录下的盘也算）。
    AllJudged,
    /// 引擎判某一态的盘时失败（panic）：`what` 与两段之前取样回调里接住它时的那一句相同（`<panic 的消息>:after_ordinal_<按序号它前面那一态>`），
    /// 这条流照两段之前记成某个状态上实现自崩（[`StreamExport::record_an_engine_failure_in_a_state`]）。
    EngineFailedInAState { what: String },
}

/// 判段：录入段录下了盘的每个取样态按序号升序（两段之前取样回调判它们的次序）交 `tables` 判，分给 `threads` 条线程
/// （[`outcomes_up_to_the_earliest_panic`]），换成判完的名单；S3 照判出的名单与 `judge_crash_image` 那一遍判红的不变量比
/// （O2 阳性对照那一趟不比，记相同）。关掉 O2 的态不动。引擎在序号上第一个失败的态停下，交回
/// [`JudgedTablesOfAStream::EngineFailedInAState`]（它之前的态判完了，它与之后的不再往下走）。
///
/// # Errors
/// 交接与录下的对不上（[`PoolCheckerTablesOfTheTwoStages::judge_a_recorded_table`] 的 `Err`）：那一句原样交回。
pub(super) fn judge_the_recorded_pool_checker_tables(
    observations: &mut [StateObservation],
    tables: &dyn PoolCheckerTablesOfTheTwoStages,
    threads: NonZeroUsize,
    switches: RunSwitches,
) -> Result<JudgedTablesOfAStream, String> {
    let mut positions_in_the_ordinal_order: Vec<usize> = (0..observations.len()).collect();
    positions_in_the_ordinal_order.sort_by_key(|position| observations[*position].ordinal);
    let recorded_tables: Vec<(usize, RecordedTableReference)> = positions_in_the_ordinal_order
        .into_iter()
        .filter_map(|position| match &observations[position].pool_checker {
            PoolCheckerOfAState::RecordedForTheJudgeStage(table) => Some((position, table.clone())),
            PoolCheckerOfAState::Off | PoolCheckerOfAState::Judged { .. } => None,
        })
        .collect();
    let judged_in_the_ordinal_order =
        outcomes_up_to_the_earliest_panic(recorded_tables.len(), threads, &|item| {
            tables.judge_a_recorded_table(&recorded_tables[item].1)
        });
    let pool_checker_control_is_on =
        switches.positive_control == Some(PositiveControl::PoolChecker);
    let mut ordinal_judged_last: Option<u64> = None;
    for ((position, _table), judged) in recorded_tables.iter().zip(judged_in_the_ordinal_order) {
        let red = match judged {
            Ok(red_or_refusal) => red_or_refusal?,
            Err(payload) => {
                return Ok(JudgedTablesOfAStream::EngineFailedInAState {
                    what: format!(
                        "{}:after_ordinal_{}",
                        panic_text(&*payload),
                        ordinal_judged_last
                            .map_or_else(|| "none".to_string(), |ordinal| ordinal.to_string())
                    ),
                });
            }
        };
        let observation = &mut observations[*position];
        let matches_the_judgement =
            red == observation.verdict.pool_checker_red || pool_checker_control_is_on;
        observation.pool_checker = PoolCheckerOfAState::Judged {
            red,
            matches_the_judgement,
        };
        ordinal_judged_last = Some(observation.ordinal);
    }
    Ok(JudgedTablesOfAStream::AllJudged)
}

#[cfg(test)]
mod tests;

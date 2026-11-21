//! 取样态的观测：一个态上判的各块、panic 的态与按序观测取样态。

use super::*;

// ───── 一个取样态上的 oracle 与它的记录 ─────

/// 一个取样状态上看到的：批号、崩溃状态序号、段号、落地向量、镜像摘要、`judge_crash_image` 的判定（`impl-verdicts.tsv`）、
/// O1 三个子列与 O2 的结论与各自耗时、触达的最后一次调用、可写挂载探针（开了才有）。看 journal 那一遍恢复 panic 的态 `recovery_panic` 有值
/// （修订 5 第 5.1 节第 5 条：含义收窄，回调链上别的件 panic 不算）；判态路上 panic 了的件记在 `piece_panics`。
pub(super) struct StateObservation {
    pub(super) batch: u8,
    pub(super) ordinal: u64,
    pub(super) segment_index: usize,
    pub(super) landing_vector: String,
    pub(super) image_digest: String,
    pub(super) verdict: ImplementationVerdict,
    pub(super) judge_nanoseconds: u128,
    pub(super) mask_matches_the_facts: bool,
    /// 这一态里持久或撕裂的写属于的最后一次调用（一个写都没落的态是 None）。
    pub(super) touched_call: Option<usize>,
    pub(super) crash_judge: CrashJudgeVerdict,
    pub(super) reference_filesystem: ReferenceFileSystemVoteOfAState,
    /// RefFS 的崩溃时刻（`call_<k>` 或 `after_every_step`）。
    pub(super) reference_filesystem_crash_moment: String,
    /// O2 直接调 `check_pool_image` 判红的不变量（关掉 O2 时 None）。
    pub(super) pool_checker_red: Option<Vec<String>>,
    /// S3：直接调的结果与 `judge_crash_image(...).pool_checker` 判红的不变量集合相同。
    pub(super) pool_checker_matches_the_judgement: bool,
    pub(super) consulted: RecoverySummary,
    pub(super) ignored: RecoverySummary,
    /// 看 journal 那一遍恢复落到的版本（实例代号, checkpoint_txg）；Q5 的「已确认的丢了」按它判。
    pub(super) consulted_root: Option<(InstanceGeneration, CheckpointTxg)>,
    pub(super) probe: Option<WritableMountProbe>,
    pub(super) recovery_panic: Option<String>,
    pub(super) crash_judge_nanoseconds: u128,
    pub(super) pool_checker_nanoseconds: u128,
    pub(super) recovery_nanoseconds: u128,
    /// 看 journal 那一遍的整份恢复报告：只在 Q5 主取样点按等价类判的那一趟留（类核对 ①、④ 与「等价类对拍」逐字段比它），
    /// 别的趟与恢复 panic 的态是 None。
    pub(super) consulted_report: Option<RecoveryReport>,
    /// 回调链 panic 了的态上，逐件重做时 panic 的件（[`JudgingPiece`] 的次序）；回调链没 panic 的态是空的（修订 5 第 5.1 节第 5 条）。
    pub(super) piece_panics: Vec<JudgingPiece>,
    /// 正常那一路（`judge_crash_state_range` 的回调链）panic 了的态是那一次 panic 的消息（不进产物），没 panic 的态是 None。
    /// 有值而 `piece_panics` 是空的态数进 `unattributed_panic_states=`（[`unattributed_panic_states_of`]）。
    pub(super) judging_path_panic: Option<String>,
}

/// 一次观测的状态：阳性对照里「批 1 第一个读回了文件的态」只翻一次（O1-崩溃判官与 RefFS 各一次）、「见证」只改一次。
struct ObservationControls {
    pub(super) switches: RunSwitches,
    crash_judge_control_pending: bool,
    reference_filesystem_control_pending: bool,
    witness_control_pending: bool,
    pub(super) consulted_reports: ConsultedReports,
}

/// 观测时留不留看 journal 那一遍的整份恢复报告（[`StateObservation::consulted_report`]）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ConsultedReports {
    Dropped,
    Kept,
}

/// 把写表每一项落在哪一次调用里算出来：写在录制流里的下标落在 `first_operation` 不超过它的最后一次调用里。
pub(super) fn call_index_of_each_write(stream: &PreparedStream) -> Vec<Option<usize>> {
    stream
        .stream_indexes
        .iter()
        .map(|operation_index| {
            stream
                .calls
                .iter()
                .rposition(|call| call.first_operation <= *operation_index)
        })
        .collect()
}

/// O1-模型崩溃判官在一个状态上的判定：交法照 `crash_injection.rs` 的第一截——「盘上已持久的最新根槽」由 `newest_persisted_root`
/// 从写流数出（只看写表里的原写，不含撕裂镜像），读回由 `observed_read_back_after_a_crash` 从看 journal 那一遍恢复报告造，
/// 模型那一侧给每一步之后并起来的版本目录，交给 `crash_recovery_disagreement`（S8：三处调用都在这个函数里）。
fn crash_judge_of_the_state(
    stream: &PreparedStream,
    image: &CrashImage<'_>,
    read_back: &ObservedReadBack,
) -> CrashJudgeVerdict {
    let newest_persisted_root_key =
        newest_persisted_root(&stream.writes, &image.persisted[..stream.writes.len()])
            .map(|(checkpoint_txg, instance)| model_root_key(instance, checkpoint_txg));
    crash_judge_verdict(
        &stream.committed_versions,
        newest_persisted_root_key,
        read_back,
    )
}

/// 模型的 `crash_recovery_disagreement` 判一态读回的版本：有不同记红（带实现那一侧说的话），没有不同记绿。
pub(super) fn crash_judge_verdict(
    committed_versions: &BTreeMap<ModelRootKey, Option<Rc<[u8]>>>,
    newest_persisted_root_key: Option<ModelRootKey>,
    read_back: &ObservedReadBack,
) -> CrashJudgeVerdict {
    match crash_recovery_disagreement(committed_versions, newest_persisted_root_key, read_back) {
        Some(disagreement) => CrashJudgeVerdict::Red {
            what: disagreement
                .implementation_answer
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("_"),
        },
        None => CrashJudgeVerdict::Green,
    }
}

/// 一态上 O1-崩溃判官适用不适用（读法 6）：这一态里有写落在模型停下的那次调用（含）之后的调用里，模型的版本目录里没有那些调用写出的版本，不适用。
pub(super) fn crash_judge_is_applicable(
    touched_call: Option<usize>,
    model_stopped_at_call: Option<usize>,
) -> bool {
    !touched_call.is_some_and(|call| model_stopped_at_call.is_some_and(|stopped| call >= stopped))
}

/// 读回内容末字节翻一位（阳性对照 O1-崩溃判官、RefFS）；读回里没有文件时交回 false、不动。
fn flip_the_last_byte_of_a_read_back(read_back: &mut ObservedReadBack) -> bool {
    if let ObservedReadBack::FileRead { content, .. } = read_back {
        if let Some(last_byte) = content.last_mut() {
            *last_byte ^= 1;
            return true;
        }
    }
    false
}

/// 按序号升序判一串连续的序号段；某一段判到一半 panic（`judge_run` 交回 panic 的那个序号）时，把那个序号记成恢复 panic 的态、
/// 从它的下一个接着判（态上的恢复 panic 不截断，读法 5：各态是互不依赖的镜像，截断会让判了几态取决于判的次序）。
/// 迭代上界：每一轮要么判完一段、要么至少吃掉一个序号；跨轮带的是还没判的那一截。
fn judge_runs_continuing_past_panicking_states(
    runs: &[std::ops::Range<u64>],
    judge_run: &mut dyn FnMut(std::ops::Range<u64>) -> Result<(), u64>,
    note_a_panicking_state: &mut dyn FnMut(u64),
) {
    for run in runs {
        let mut remaining = run.clone();
        while !remaining.is_empty() {
            match judge_run(remaining.clone()) {
                Ok(()) => break,
                Err(panicking_ordinal) => {
                    assert!(
                        remaining.contains(&panicking_ordinal),
                        "panic 的序号 {panicking_ordinal} 在这一段 {remaining:?} 里"
                    );
                    note_a_panicking_state(panicking_ordinal);
                    remaining = panicking_ordinal + 1..remaining.end;
                }
            }
        }
    }
}

/// 观测要的、与这一条流绑在一起的几样：流、枚举表、按序号现算掩码的事实表、撕裂镜像的下标、RefFS 的翻译。
pub(super) struct ObservationContext<'stream> {
    pub(super) stream: &'stream PreparedStream,
    pub(super) tables: &'stream CrashStateEnumerationTables,
    pub(super) facts: &'stream StateMasks,
    pub(super) torn_image_index_by_write: &'stream BTreeMap<usize, usize>,
    pub(super) reference_filesystem: &'stream ReferenceFileSystemStream,
    /// 写表每一项落在哪一次调用里（[`call_index_of_each_write`]，整条流算一次）。
    pub(super) call_of_write: Vec<Option<usize>>,
    /// 判态路 panic 的注入：跑法里只有阳性对照「判态路 panic 单列」那一遍有，别的都是 None；内联用例 U13–U16 另经
    /// [`ConfirmedLossPiecePanicInjections`] 交给 Q5 的对拍、阳性对照与取样点导出（登记修订 6 第 6.2 节第 3 条）。
    pub(super) piece_panic_injection: Option<&'stream PiecePanicInjection>,
}

impl ObservationContext<'_> {
    fn segment_of_ordinal(&self, ordinal: u64) -> usize {
        self.tables
            .state_ranges_by_segment
            .partition_point(|state_range| state_range.end <= ordinal)
    }
    /// RefFS 崩溃时刻要的段号：序号越过最后一段的那个全持久态交 `None`。
    pub(super) fn reference_filesystem_moment_of(
        &self,
        segment_index: usize,
    ) -> ReferenceFileSystemCrashMoment {
        let segment = (segment_index < self.stream.segments.len()).then_some(segment_index);
        reference_filesystem_crash_moment(self.reference_filesystem, segment)
    }
}

/// 一个判完了的态（`judge_crash_image` 交回了判定）上的全部 oracle（O1 三个子列、O2、探针）。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
fn observation_of_a_judged_state(
    context: &ObservationContext<'_>,
    batch: u8,
    ordinal: u64,
    image: &CrashImage<'_>,
    judgement: &CrashStateJudgement,
    controls: &mut ObservationControls,
) -> StateObservation {
    let stream = context.stream;
    let switches = controls.switches;
    let landing_vector = landing_vector_of(
        &image.persisted,
        stream.writes.len(),
        context.torn_image_index_by_write,
    );
    // 阳性对照「判态路 panic 单列」（修订 5 第 5.1 节）：选中的态在回调里先 panic，逼走逐件重做那一路。
    assert!(
        context
            .piece_panic_injection
            .and_then(|injection| injection.rule_of(&landing_vector))
            .is_none(),
        "修订 5 阳性对照「判态路 panic 单列」注入：正常那一路在这一态的回调里先 panic"
    );
    let touched_call = landing_vector
        .chars()
        .zip(&context.call_of_write)
        .filter(|(landing, _)| *landing != '0')
        .filter_map(|(_, call)| *call)
        .max();
    let verdict = ImplementationVerdict::of(judgement);
    let segment_index = context.segment_of_ordinal(ordinal);
    let model_is_enabled =
        switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge);
    let read_back = observed_read_back_after_a_crash(&judgement.consulted);
    let crash_judge_started = Instant::now();
    let crash_judge = if !model_is_enabled {
        CrashJudgeVerdict::Off
    } else if !crash_judge_is_applicable(touched_call, stream.model_stopped_at_call) {
        CrashJudgeVerdict::NotApplicable
    } else {
        let mut judged_read_back = read_back.clone();
        if controls.crash_judge_control_pending
            && batch == 1
            && flip_the_last_byte_of_a_read_back(&mut judged_read_back)
        {
            controls.crash_judge_control_pending = false;
        }
        crash_judge_of_the_state(stream, image, &judged_read_back)
    };
    let moment = context.reference_filesystem_moment_of(segment_index);
    let reference_filesystem = if model_is_enabled {
        let mut judged_read_back = read_back.clone();
        if controls.reference_filesystem_control_pending
            && batch == 1
            && flip_the_last_byte_of_a_read_back(&mut judged_read_back)
        {
            controls.reference_filesystem_control_pending = false;
        }
        reference_filesystem_vote_of_a_state(
            context.reference_filesystem,
            moment,
            &judged_read_back,
        )
    } else {
        ReferenceFileSystemVoteOfAState::Off
    };
    let crash_judge_nanoseconds = crash_judge_started.elapsed().as_nanos();
    let pool_checker_control = switches.positive_control == Some(PositiveControl::PoolChecker)
        && stream.name == StreamName::NewPoolFileCreation
        && ordinal == context.tables.state_count - 1;
    // O2 逐状态计时量的是 CPU 池级 checker 本身（豁免截名 experiment-measures-the-cpu-judge-itself）：CPU 这一遍只计时；
    // 判红的名单走两段（同一张盘作一张给定状态表交引擎，阳性对照那一态是改坏的那一张），不在计时里
    let pool_checker_started = Instant::now();
    let pool_checker_timed_on_the_cpu =
        (switches.disabled_oracle != Some(DisabledOracle::PoolChecker)).then(|| {
            if pool_checker_control {
                pool_checker_on_the_first_known_bad_image(image)
            } else {
                check_pool_image(image)
            }
        });
    let pool_checker_nanoseconds = pool_checker_started.elapsed().as_nanos();
    let image_digest = sha256_text(&crash_image_digest(image));
    // None 有两种：O2 这路 oracle 关着；或引擎这一趟只录没判（占位，判归编进 gpu 特性的那一趟）——两种都不与判决比
    let pool_checker_red = pool_checker_timed_on_the_cpu.and_then(|_| {
        pool_checker_red_names_of_a_sampled_state_through_the_given_states_engine(
            stream.name,
            ordinal,
            &image_digest,
            image,
            pool_checker_control,
        )
    });
    let pool_checker_matches_the_judgement = pool_checker_red
        .as_ref()
        .is_none_or(|direct| *direct == verdict.pool_checker_red)
        || pool_checker_control;
    let recovery_started = Instant::now();
    let _ =
        singlefs_core::recovery::recover(image, singlefs_core::recovery::JournalPolicy::Consult);
    let recovery_nanoseconds = recovery_started.elapsed().as_nanos();
    let probe = switches.writable_mount_probe.then(|| {
        let mut materialized = materialized_crash_image(image);
        let mut selected_version = judgement.consulted.effective_root;
        if controls.witness_control_pending
            && stream.name == StreamName::NewPoolFileCreation
            && ordinal == context.tables.state_count - 1
        {
            controls.witness_control_pending = false;
            raise_the_witnessed_counter_past_the_selected_version(
                &mut materialized,
                selected_version,
            );
            selected_version = singlefs_core::recovery::recover(
                &materialized,
                singlefs_core::recovery::JournalPolicy::Consult,
            )
            .effective_root;
        }
        writable_mount_probe_on(&materialized, selected_version, &crash_judge)
    });
    StateObservation {
        batch,
        ordinal,
        segment_index,
        landing_vector,
        image_digest,
        verdict,
        judge_nanoseconds: crash_judge_nanoseconds
            + pool_checker_nanoseconds
            + recovery_nanoseconds,
        mask_matches_the_facts: context.facts.persisted_writes_of_state(ordinal) == image.persisted,
        touched_call,
        crash_judge,
        reference_filesystem,
        reference_filesystem_crash_moment: crash_moment_text(moment),
        pool_checker_red,
        pool_checker_matches_the_judgement,
        consulted: RecoverySummary::of(&judgement.consulted),
        ignored: RecoverySummary::of(&judgement.ignored),
        consulted_root: judgement.consulted.effective_root,
        probe,
        recovery_panic: None,
        crash_judge_nanoseconds,
        pool_checker_nanoseconds,
        recovery_nanoseconds,
        consulted_report: match controls.consulted_reports {
            ConsultedReports::Dropped => None,
            ConsultedReports::Kept => Some(judgement.consulted.clone()),
        },
        piece_panics: Vec::new(),
        judging_path_panic: None,
    }
}

/// 回调链 panic 了的那一态（`judge_crash_image` 里 panic，或判这一态的 oracle 里 panic）：镜像按事实表的掩码现造，照修订 5 第 5.1 节第 2 条
/// 的次序逐件重做、每件各包一层 `catch_unwind`（[`JudgingPiece`]），panic 了记下件名、接着做下一件。O1 两个崩溃子列只随看 journal 那遍恢复：
/// 它 panic 两列都记恢复 panic（读法 5），没 panic 两列照正常那一路的函数从这一遍的恢复报告算，O1-模型崩溃判官或 RefFS 自己 panic 那一列记
/// `judge_panic`；别的件 panic 不改这两列（第 3 条）。O2 照判，panic 记 `pool_checker_panic`、按红计（第 4 条）。
/// 阳性对照的翻读回、见证改写与 O2 的坏镜像都只在正常那一路做，这一路不做。`judging_path_panic` 是正常那一路 panic 的消息，
/// 记进 [`StateObservation::judging_path_panic`]、不进产物。
// gpu-exempt: experiment-measures-the-cpu-judge-itself
fn observation_of_a_panicking_state(
    context: &ObservationContext<'_>,
    batch: u8,
    ordinal: u64,
    switches: RunSwitches,
    consulted_reports: ConsultedReports,
    judging_path_panic: String,
) -> StateObservation {
    let stream = context.stream;
    let persisted = context.facts.persisted_writes_of_state(ordinal);
    let image = CrashImage {
        base: &stream.base,
        writes: &context.tables.enumerated_writes,
        persisted,
    };
    let landing_vector = landing_vector_of(
        &image.persisted,
        stream.writes.len(),
        context.torn_image_index_by_write,
    );
    let touched_call = landing_vector
        .chars()
        .zip(&context.call_of_write)
        .filter(|(landing, _)| *landing != '0')
        .filter_map(|(_, call)| *call)
        .max();
    let segment_index = context.segment_of_ordinal(ordinal);
    let injected = context
        .piece_panic_injection
        .and_then(|injection| injection.rule_of(&landing_vector))
        .and_then(|rule| rule.piece);
    let mut piece_panics: Vec<JudgingPiece> = Vec::new();
    let consulted = judging_piece_outcome(JudgingPiece::ConsultedRecovery, injected, || {
        singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Consult)
    });
    note_a_piece_panic(
        &mut piece_panics,
        JudgingPiece::ConsultedRecovery,
        &consulted,
    );
    let ignored = judging_piece_outcome(JudgingPiece::IgnoredRecovery, injected, || {
        singlefs_core::recovery::recover(&image, singlefs_core::recovery::JournalPolicy::Ignore)
    });
    note_a_piece_panic(&mut piece_panics, JudgingPiece::IgnoredRecovery, &ignored);
    let root_persisted = judging_piece_outcome(JudgingPiece::RootPersisted, injected, || {
        image.persisted[stream.judged_root_index]
    });
    note_a_piece_panic(
        &mut piece_panics,
        JudgingPiece::RootPersisted,
        &root_persisted,
    );
    // 两份崩溃点重放 oracle 各吃自己那一遍恢复：那一遍 panic 了，那一份记 `not_run`；两遍都 panic 了这一件整个不跑。
    let crash_point_replay_oracles = if consulted.done().is_none() && ignored.done().is_none() {
        PieceOutcome::NotRun
    } else {
        judging_piece_outcome(JudgingPiece::CrashPointReplayOracles, injected, || {
            let newest_persisted = newest_persisted_root(image.writes, &image.persisted);
            let oracle_of = |report: &RecoveryReport| {
                classified_oracle_violation_for_versions(
                    &report.outcome,
                    report.effective_root,
                    newest_persisted,
                    &stream.versions,
                )
            };
            CrashPointReplayOraclesOfThePieces {
                consulted: consulted.done().map(oracle_of),
                ignored: ignored.done().map(oracle_of),
            }
        })
    };
    note_a_piece_panic(
        &mut piece_panics,
        JudgingPiece::CrashPointReplayOracles,
        &crash_point_replay_oracles,
    );
    let pool_checker_started = Instant::now();
    let pool_checker = judging_piece_outcome(JudgingPiece::PoolChecker, injected, || {
        check_pool_image(&image)
    });
    let pool_checker_nanoseconds = pool_checker_started.elapsed().as_nanos();
    note_a_piece_panic(&mut piece_panics, JudgingPiece::PoolChecker, &pool_checker);
    let record_checker = match consulted.done() {
        Some(report) => judging_piece_outcome(JudgingPiece::RecordChecker, injected, || {
            check_records(&image, report.effective_root)
        }),
        None => PieceOutcome::NotRun,
    };
    note_a_piece_panic(
        &mut piece_panics,
        JudgingPiece::RecordChecker,
        &record_checker,
    );
    // 两个崩溃子列只随看 journal 那遍恢复 panic（第 3 条）：别的件 panic 不改这两列。
    let columns_follow_a_recovery_panic = matches!(consulted, PieceOutcome::Panicked(_));
    let model_is_enabled =
        switches.disabled_oracle != Some(DisabledOracle::ComparisonAndCrashJudge);
    let moment = context.reference_filesystem_moment_of(segment_index);
    let crash_judge_started = Instant::now();
    let crash_judge = match consulted.done() {
        _ if columns_follow_a_recovery_panic => CrashJudgeVerdict::RecoveryPanicked,
        None => CrashJudgeVerdict::RecoveryPanicked,
        Some(_) if !model_is_enabled => CrashJudgeVerdict::Off,
        Some(_) if !crash_judge_is_applicable(touched_call, stream.model_stopped_at_call) => {
            CrashJudgeVerdict::NotApplicable
        }
        Some(report) => {
            let outcome = judging_piece_outcome(JudgingPiece::CrashJudge, injected, || {
                crash_judge_of_the_state(stream, &image, &observed_read_back_after_a_crash(report))
            });
            note_a_piece_panic(&mut piece_panics, JudgingPiece::CrashJudge, &outcome);
            match outcome {
                PieceOutcome::Done(verdict) => verdict,
                PieceOutcome::Panicked(_) | PieceOutcome::NotRun => {
                    CrashJudgeVerdict::JudgePanicked
                }
            }
        }
    };
    let reference_filesystem = match consulted.done() {
        _ if columns_follow_a_recovery_panic => ReferenceFileSystemVoteOfAState::RecoveryPanicked,
        None => ReferenceFileSystemVoteOfAState::RecoveryPanicked,
        Some(_) if !model_is_enabled => ReferenceFileSystemVoteOfAState::Off,
        Some(report) => {
            let outcome =
                judging_piece_outcome(JudgingPiece::ReferenceFileSystem, injected, || {
                    reference_filesystem_vote_of_a_state(
                        context.reference_filesystem,
                        moment,
                        &observed_read_back_after_a_crash(report),
                    )
                });
            note_a_piece_panic(
                &mut piece_panics,
                JudgingPiece::ReferenceFileSystem,
                &outcome,
            );
            match outcome {
                PieceOutcome::Done(vote) => vote,
                PieceOutcome::Panicked(_) | PieceOutcome::NotRun => {
                    ReferenceFileSystemVoteOfAState::JudgePanicked
                }
            }
        }
    };
    let crash_judge_nanoseconds = crash_judge_started.elapsed().as_nanos();
    let recovery_started = Instant::now();
    let timing_recovery = judging_piece_outcome(JudgingPiece::TimingRecovery, injected, || {
        let _ = singlefs_core::recovery::recover(
            &image,
            singlefs_core::recovery::JournalPolicy::Consult,
        );
    });
    let recovery_nanoseconds = recovery_started.elapsed().as_nanos();
    note_a_piece_panic(
        &mut piece_panics,
        JudgingPiece::TimingRecovery,
        &timing_recovery,
    );
    let probe = match (switches.writable_mount_probe, consulted.done()) {
        (true, Some(report)) => {
            let outcome = judging_piece_outcome(JudgingPiece::Probe, injected, || {
                writable_mount_probe_on(
                    &materialized_crash_image(&image),
                    report.effective_root,
                    &crash_judge,
                )
            });
            note_a_piece_panic(&mut piece_panics, JudgingPiece::Probe, &outcome);
            match outcome {
                PieceOutcome::Done(probe) => Some(probe),
                PieceOutcome::Panicked(_) | PieceOutcome::NotRun => None,
            }
        }
        (true, None) | (false, _) => None,
    };
    let pool_checker_red = (switches.disabled_oracle != Some(DisabledOracle::PoolChecker)).then(
        || match &pool_checker {
            PieceOutcome::Done(verdicts) => violated_invariant_names(verdicts),
            PieceOutcome::Panicked(_) | PieceOutcome::NotRun => {
                vec!["pool_checker_panic".to_string()]
            }
        },
    );
    let verdict = ImplementationVerdict::of_the_pieces(
        &consulted,
        &ignored,
        &crash_point_replay_oracles,
        &pool_checker,
        &record_checker,
    );
    StateObservation {
        batch,
        ordinal,
        segment_index,
        landing_vector,
        image_digest: sha256_text(&crash_image_digest(&image)),
        verdict,
        judge_nanoseconds: crash_judge_nanoseconds
            + pool_checker_nanoseconds
            + recovery_nanoseconds,
        mask_matches_the_facts: true,
        touched_call,
        crash_judge,
        reference_filesystem,
        reference_filesystem_crash_moment: crash_moment_text(moment),
        pool_checker_red,
        pool_checker_matches_the_judgement: true,
        consulted: consulted.done().map_or_else(
            RecoverySummary::of_a_panicking_recovery,
            RecoverySummary::of,
        ),
        ignored: ignored.done().map_or_else(
            RecoverySummary::of_a_panicking_recovery,
            RecoverySummary::of,
        ),
        consulted_root: consulted.done().and_then(|report| report.effective_root),
        probe,
        recovery_panic: match &consulted {
            PieceOutcome::Panicked(message) => Some(message.clone()),
            PieceOutcome::Done(_) | PieceOutcome::NotRun => None,
        },
        crash_judge_nanoseconds,
        pool_checker_nanoseconds,
        recovery_nanoseconds,
        consulted_report: match (consulted_reports, consulted.done()) {
            (ConsultedReports::Kept, Some(report)) => Some(report.clone()),
            (ConsultedReports::Kept, None) | (ConsultedReports::Dropped, _) => None,
        },
        piece_panics,
        judging_path_panic: Some(judging_path_panic),
    }
}

/// 按序号升序把取样的状态切成连续的几段，每段调一次 `judge_crash_state_range`：回调里给的那个 `CrashImage` 上算全部 oracle
/// （[`observation_of_a_judged_state`]）。某个状态上 panic：记成恢复 panic 的态、接着判它后面的（[`judge_runs_continuing_past_panicking_states`]）。
pub(super) fn observe_sampled_states(
    context: &ObservationContext<'_>,
    sampled: &SampledOrdinals,
    switches: RunSwitches,
    consulted_reports: ConsultedReports,
) -> Vec<StateObservation> {
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
    let mut runs: Vec<std::ops::Range<u64>> = Vec::new();
    for ordinal in batch_of_ordinal.keys().copied() {
        match runs.last_mut() {
            Some(run) if run.end == ordinal => run.end += 1,
            Some(_) | None => runs.push(ordinal..ordinal + 1),
        }
    }
    let stream = context.stream;
    let mut controls = ObservationControls {
        switches,
        crash_judge_control_pending: switches.positive_control == Some(PositiveControl::CrashJudge),
        reference_filesystem_control_pending: switches.positive_control
            == Some(PositiveControl::ReferenceFileSystem),
        witness_control_pending: switches.positive_control == Some(PositiveControl::Witness),
        consulted_reports,
    };
    let mut observations: Vec<StateObservation> = Vec::new();
    let mut panicking_states: Vec<(u64, String)> = Vec::new();
    // panic 的序号与消息在 `judge_run` 里记（它手里有 panic 的消息）；块之后只核每一个都记到了。`judge_run` 借着
    // `observations` 与 `panicking_states`，放进这一块，块结束借用就还回来。
    let mut panicking_ordinals_noted: Vec<u64> = Vec::new();
    {
        let mut judge_run = |run: std::ops::Range<u64>| -> Result<(), u64> {
            let mut next_ordinal = run.start;
            let judged =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    sample_crash_states_on_the_cpu(
                CrashPointReplayItem {
                    name: &sampling_item_name_of_this_call(),
                    setting_file: CRASH_POINT_REPLAY_SAMPLING_ITEM.setting_file,
                },
                    &stream.base,
                    &stream.writes,
                    &stream.segments,
                    stream.judged_root_index,
                    &stream.versions,
                    &full_expansion,
                    run.clone(),
                    &mut |ordinal, image, judgement| {
                        observations.push(observation_of_a_judged_state(
                            context,
                            batch_of_ordinal[&ordinal],
                            ordinal,
                            image,
                            judgement,
                            &mut controls,
                        ));
                        next_ordinal = ordinal + 1;
                    },
                )
        .judged()
        .expect("实验装置不分只录、只判两趟跑：起它时不设 SINGLEFS_CRASH_AMPLIFICATION_STAGE")
                }));
            match judged {
                Ok(state_count_of_the_stream) => {
                    assert_eq!(
                        state_count_of_the_stream, context.tables.state_count,
                        "judge_crash_state_range 交回的整条流状态数与枚举表的相同"
                    );
                    Ok(())
                }
                Err(payload) => {
                    panicking_states.push((next_ordinal, panic_text(&*payload)));
                    Err(next_ordinal)
                }
            }
        };
        judge_runs_continuing_past_panicking_states(&runs, &mut judge_run, &mut |ordinal| {
            panicking_ordinals_noted.push(ordinal);
        });
    }
    assert_eq!(
        panicking_ordinals_noted,
        panicking_states
            .iter()
            .map(|(ordinal, _)| *ordinal)
            .collect::<Vec<u64>>(),
        "panic 的态两处记的一样"
    );
    // 正常那一路 panic 的消息不进产物：逐件重做时各件自己的结局才是这一态的判定（修订 5 第 5.1 节第 2 条）。
    for (ordinal, message_of_the_judging_path) in panicking_states {
        observations.push(observation_of_a_panicking_state(
            context,
            batch_of_ordinal[&ordinal],
            ordinal,
            switches,
            consulted_reports,
            message_of_the_judging_path,
        ));
    }
    observations.sort_by_key(|observation| (observation.batch, observation.ordinal));
    observations
}

#[cfg(test)]
mod tests;

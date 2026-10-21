//! RefFS（`singlefs-reference-filesystem`）的调用方一侧：把一段历史、每步的结局、崩溃时刻翻成 RefFS 的五样输入
//! （`research/prompts/m3-formal-r6-main-verification.md` 第二节 X2 一行的白名单：操作结局、hi 与它的归属规则、回退目标 m、
//! 每次恢复读出的内容、显式发布点），再把 RefFS 的判定交回调用点。依赖只许这一个方向：harness 档依赖 RefFS，RefFS 不依赖 harness
//! （门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格 ⑤⑥⑦）。
//!
//! 照 r6 用户定案 H1-收严写：
//! - 崩溃时刻按「切开当前段的那道屏障（或那次 FUA 写）由哪一步发出」归属（[`segment_closing_stream_indexes`]、
//!   [`TranslatedHistory::crash_moment_in_segment`]）；只给 RefFS 用，O1 判官的归属照崩溃注入今天「段里第一个被扣下的写」不动。
//! - m 表（根 → 版本号）由这里攒：实现写下的每一条根（发布调用、回退、可写挂载的写行与暖机、抬 F、卸载）映到写它时的现行版本，
//!   现行版本含每次恢复读出、重新起头的那一份（冷启动恢复与抛弃最新根的那次挂载）。
//!
//! 一步历史在 RefFS 里是什么（第一版的操作集合：第一个文件、覆盖写、发布调用、回退；fsync 今天的历史里没有）：
//! 发布第一个文件 = 建第一个文件 + 发布调用；覆盖写 = 覆盖 + 发布调用；零单元发布 = 发布调用；管理员回退 = 回退到 m；
//! 可写挂载、抬 F 不改内容、不进 RefFS 的操作序列，它们写的根映到现行版本；冷启动 = 一次恢复（判读出的内容，以它重新起头）；
//! 抛弃最新根的那次挂载 = 一次带读故障的恢复（不判，以读出的内容重新起头，r6 判决第二节 U2 一行），连同被抛弃的那条根是不是写出
//! RefFS 最新那一版的那一条（[`AbandonedRoot`]，判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1）。
//! 原样重发这一种结局今天的历史走不出来（入口报错历史就停），这里不产出它。

use std::collections::BTreeMap;

use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_reference_filesystem::{
    AbandonedRoot, AbandonedRootRestartNotAccepted, AllowedState, CrashNotAccepted, FileContent,
    Operation, OperationNotAccepted, OperationNumber, OperationOutcome, ReferenceFileSystem,
    SnapshotName, SubmittedOperation, VersionNumber, VisibleState,
};

use crate::fault_injection::{FaultOutcome, InjectedFault};
use crate::history::{
    AppliedEffect, HarnessJudgement, HistoryEnding, HistoryOperation, HistoryRun,
    HistoryStartingPoint, SnapshotChange, StepObservation, StepOutcome, StepPosition,
};
use crate::memory_pool::root_identity_written_by;
use crate::model::{ModelDisagreement, ModelRootKey, ObservedReadBack, ObservedSnapshotReadBack};
use crate::model_comparison::model_root_key;
use crate::scenario::first_file_content;
use crate::segments::{FixedGeometry, SegmentAfterOperation, SegmentClosingRule, StepKind};
use crate::{RecordedOperationKind, RetainedOperation};

/// RefFS 判红时记进失败观察的那一项的名字（与挂载透明、I-8.4 一样，不是池级 checker 的不变量，借违例那一栏带着走）。
pub const REFERENCE_FILESYSTEM_VIOLATION_NAME: &str = "RefFS：恢复读出不在允许集合里";

/// 调用方看到的一步怎么收尾，按 RefFS 的结局白名单分。
#[derive(Clone, Debug, PartialEq, Eq)]
enum StepEndingSeenByTheTranslation {
    /// 入口返回 Ok；起点段没有 [`AppliedEffect`]。这一步里有恢复（冷启动、抛弃最新根的那次挂载）时带着它读出的内容。
    ReturnedOk {
        effect: Option<AppliedEffect>,
        read_out_of_the_recovery_in_the_step: Option<VisibleState>,
    },
    /// 入口在任何写之前拒（历史照判过的拒：模型核过拒之前一个写都没发）。
    RefusedBeforeAnyWrite,
    /// 入口返回了错误，写没写过不知道（故障注入那一步）：按「写了之后报错」算，两可。
    ReturnedAnErrorAfterItMayHaveWritten,
    /// 入口没返回（panic）：这一步在途。只在历史停下的那一步上有（[`ReferenceFileSystemTranslation::note_the_step_the_history_stopped_in`]），
    /// 它总是最后一步；交给 RefFS 时怎么写在途见 `events_of`。
    NeverReturned,
    /// 前提不成立，入口没调。
    NotCalled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StepSeenByTheTranslation {
    position: StepPosition,
    operation: Option<HistoryOperation>,
    ending: StepEndingSeenByTheTranslation,
    /// 这一步跑完时录制流里有几步（与崩溃注入登记的「每一步跑完时流里的步数」同一个数）。
    stream_length_after_the_step: usize,
    /// 这一步是截断时交给会话的目标 size（执行器交回的，`history::HistoryRun::sizes_requested_by_the_truncations`）：翻成 `Operation::Truncate(n)` 的 n。
    size_requested_by_the_truncation: Option<u64>,
}

/// 一段历史跑的过程中攒下的东西：起点、每一步的操作与结局、每一步跑完时流有多长。跑完之后 [`Self::translate`] 翻成 RefFS 的输入。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceFileSystemTranslation {
    starting_point: HistoryStartingPoint,
    steps: Vec<StepSeenByTheTranslation>,
}

/// 读出的内容按 RefFS 的可见状态写；恢复失败时没有状态。
fn visible_state_of(outcome: &RecoveryOutcome) -> Option<VisibleState> {
    match outcome {
        RecoveryOutcome::FileRead { content, .. } => {
            Some(VisibleState::OneFile(FileContent(content.clone())))
        }
        RecoveryOutcome::NoFile { .. } => Some(VisibleState::NoFile),
        RecoveryOutcome::Failed { .. } => None,
    }
}

impl ReferenceFileSystemTranslation {
    #[must_use]
    pub fn for_a_history_starting(starting_point: HistoryStartingPoint) -> Self {
        Self {
            starting_point,
            steps: Vec::new(),
        }
    }

    /// 历史执行器的观察者里每一步调一次（起点也调）。这一步里有恢复（冷启动、抛弃最新根的那次挂载）时，在这一步跑完的镜像上
    /// 再做一次只读恢复取读出的内容：这一步自己的那次恢复读的是同一份字节（冷启动只读不写；那次挂载之后最新的根是它暖机写的、
    /// 内容照抄它恢复到的那一版）。
    pub fn note_an_observed_step(
        &mut self,
        observation: &StepObservation<'_>,
        stream_length_after_the_step: usize,
    ) {
        let read_out_of_the_recovery_in_the_step =
            match (observation.operation, observation.outcome) {
                (
                    Some(HistoryOperation::ColdStartRecover),
                    Some(StepOutcome::Applied(AppliedEffect::Recovered { .. })),
                )
                | (
                    Some(HistoryOperation::CrashRecoveryAbandoningTheNewestRoot),
                    Some(StepOutcome::Applied(AppliedEffect::Mounted { .. })),
                ) => visible_state_of(&recover(observation.image, JournalPolicy::Consult).outcome),
                (Some(_) | None, Some(_) | None) => None,
            };
        let ending = match observation.outcome {
            None => StepEndingSeenByTheTranslation::ReturnedOk {
                effect: None,
                read_out_of_the_recovery_in_the_step,
            },
            Some(StepOutcome::Applied(effect)) => StepEndingSeenByTheTranslation::ReturnedOk {
                effect: Some(effect.clone()),
                read_out_of_the_recovery_in_the_step,
            },
            // 走到观察者的拒都是模型判过的拒：模型核过拒之前一个写都没发（`ModelDisagreementAspect::WroteBeforeRefusing`）。
            Some(StepOutcome::Refused { .. }) => {
                StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
            }
            Some(StepOutcome::NotApplicable(_)) => StepEndingSeenByTheTranslation::NotCalled,
        };
        self.steps.push(StepSeenByTheTranslation {
            position: observation.position,
            operation: observation.operation.copied(),
            ending,
            stream_length_after_the_step,
            size_requested_by_the_truncation: observation.size_requested_by_the_truncation,
        });
    }

    /// 历史停在某一步（判红、panic、入口报错）时，那一步观察者没调：照 `run` 补上它。历史跑完的不补。
    /// 入口返回 Ok 的照常翻（这一步里有恢复时读出的内容拿不到，那次恢复不进 RefFS）；入口报错的按「写了之后报错」算；
    /// 没有结局（panic）的按在途算。
    pub fn note_the_step_the_history_stopped_in(
        &mut self,
        history_operations: &[HistoryOperation],
        run: &HistoryRun,
        stream_length_at_the_end: usize,
    ) {
        let observation = match &run.ending {
            HistoryEnding::Completed => return,
            HistoryEnding::KnownRed { observation, .. }
            | HistoryEnding::NewFinding { observation, .. } => observation,
        };
        let size_requested_by_the_truncation = match observation.position {
            StepPosition::StartingPoint => None,
            StepPosition::Operation(step_index) => run
                .sizes_requested_by_the_truncations
                .get(&step_index)
                .copied(),
        };
        let (operation, ending) = match observation.position {
            StepPosition::StartingPoint => {
                let ending = match (&observation.panic, &observation.harness_judgement) {
                    (Some(_), Some(_) | None) => StepEndingSeenByTheTranslation::NeverReturned,
                    (None, Some(HarnessJudgement::TheStartingPointReturnedAnError { .. })) => {
                        StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                    }
                    (
                        None,
                        Some(
                            HarnessJudgement::AllocationGenerationIsNotThePublishTxg { .. }
                            | HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage { .. },
                        )
                        | None,
                    ) => StepEndingSeenByTheTranslation::ReturnedOk {
                        effect: None,
                        read_out_of_the_recovery_in_the_step: None,
                    },
                };
                (None, ending)
            }
            StepPosition::Operation(step_index) => {
                let ending = match (&observation.panic, run.outcomes.get(step_index)) {
                    (Some(_), Some(_) | None) | (None, None) => {
                        StepEndingSeenByTheTranslation::NeverReturned
                    }
                    (None, Some(StepOutcome::Applied(effect))) => {
                        StepEndingSeenByTheTranslation::ReturnedOk {
                            effect: Some(effect.clone()),
                            read_out_of_the_recovery_in_the_step: None,
                        }
                    }
                    (None, Some(StepOutcome::Refused { .. })) => {
                        StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                    }
                    (None, Some(StepOutcome::NotApplicable(_))) => {
                        StepEndingSeenByTheTranslation::NotCalled
                    }
                };
                (history_operations.get(step_index).copied(), ending)
            }
        };
        self.steps.push(StepSeenByTheTranslation {
            position: observation.position,
            operation,
            ending,
            stream_length_after_the_step: stream_length_at_the_end,
            size_requested_by_the_truncation,
        });
    }

    /// 翻成 RefFS 的输入：每一步的事件、m 表（根 → 版本号）、每一段录制流由哪一道屏障关上。
    /// `operations` 是整条录制流（开了内容保留，根槽写要字节），与攒步数时是同一条。
    #[must_use]
    pub fn translate(
        &self,
        operations: &[RetainedOperation],
        geometry: &FixedGeometry,
    ) -> TranslatedHistory {
        let root_writes: Vec<(usize, ModelRootKey)> = operations
            .iter()
            .enumerate()
            .filter_map(|(stream_index, retained)| {
                match retained.operation.kind {
                    RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {}
                    RecordedOperationKind::WriteZeroes
                    | RecordedOperationKind::Discard
                    | RecordedOperationKind::Barrier => return None,
                }
                if geometry.classify(&retained.operation) != StepKind::RootRecordFua {
                    return None;
                }
                let bytes = retained.contents.as_ref()?;
                let (instance, checkpoint_txg) = root_identity_written_by(bytes);
                Some((stream_index, model_root_key(instance, checkpoint_txg)))
            })
            .collect();
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        let mut versions_of_the_roots: BTreeMap<ModelRootKey, VersionNumber> = BTreeMap::new();
        let mut steps: Vec<TranslatedStep> = Vec::new();
        let mut stopped: Option<TranslationStopped> = None;
        let mut stream_length_before_the_step = 0usize;
        let mut roots_before_the_step = RootsBeforeTheStep::before_any_step();
        for (step_index, seen) in self.steps.iter().enumerate() {
            assert!(
                seen.ending != StepEndingSeenByTheTranslation::NeverReturned
                    || step_index + 1 == self.steps.len(),
                "没返回的那一步只由 note_the_step_the_history_stopped_in 补在历史停下之后，是最后一步：{:?}",
                seen.position
            );
            let version_before_the_step = model.newest_version();
            let events = match self.events_of(
                seen,
                &versions_of_the_roots,
                roots_before_the_step,
                version_before_the_step,
            ) {
                Ok(events) => events,
                Err(reason) => {
                    stopped = Some(TranslationStopped {
                        position: seen.position,
                        verdict: StoppedVerdict::Abstained(reason),
                    });
                    steps.push(TranslatedStep::without_events(seen.position));
                    break;
                }
            };
            if let Err(verdict) = apply_every_event(&mut model, &events.events) {
                stopped = Some(TranslationStopped {
                    position: seen.position,
                    verdict,
                });
                steps.push(TranslatedStep::without_events(seen.position));
                break;
            }
            let version_after_the_step = model.newest_version();
            let roots_of_the_step: Vec<ModelRootKey> = root_writes
                .iter()
                .filter(|(stream_index, _)| {
                    (stream_length_before_the_step..seen.stream_length_after_the_step)
                        .contains(stream_index)
                })
                .map(|(_, root)| *root)
                .collect();
            for (index_in_the_step, root) in roots_of_the_step.iter().enumerate() {
                let is_the_last_root_of_the_step = index_in_the_step + 1 == roots_of_the_step.len();
                let version = match events.new_version_is_written_by {
                    NewVersionWrittenBy::TheLastRootOfTheStep if is_the_last_root_of_the_step => {
                        version_after_the_step
                    }
                    NewVersionWrittenBy::TheLastRootOfTheStep => version_before_the_step,
                    NewVersionWrittenBy::EveryRootFromTheChangeOn {
                        roots_of_the_change,
                    } if index_in_the_step + roots_of_the_change >= roots_of_the_step.len() => {
                        version_after_the_step
                    }
                    NewVersionWrittenBy::EveryRootFromTheChangeOn { .. } => version_before_the_step,
                    NewVersionWrittenBy::EveryRootOfTheStep => version_after_the_step,
                };
                versions_of_the_roots.insert(*root, version);
            }
            roots_before_the_step = roots_before_the_step.after_the_step(
                &roots_of_the_step,
                events.new_version_is_written_by,
                version_before_the_step,
                version_after_the_step,
            );
            stream_length_before_the_step = seen.stream_length_after_the_step;
            steps.push(TranslatedStep {
                position: seen.position,
                events: events.events,
                crash_inside: events.crash_inside,
                stream_length_after_the_step: seen.stream_length_after_the_step,
            });
        }
        TranslatedHistory {
            steps,
            stopped,
            versions_of_the_roots,
            segment_closing_stream_indexes: segment_closing_stream_indexes(operations),
        }
    }

    /// 一步在 RefFS 里的事件。回退要的 m 在 `versions_of_the_roots` 里查（实现报的回退根写在它之前那几步里）。
    ///
    /// 没返回（panic）的发布按在途交：RefFS 的结局里没有「在途」这一种（`OperationOutcome` 四种：成、在任何写之前拒、
    /// 写了之后报错、原样重发），在途由 hi 表达——崩溃落在第 hi 个操作里，那一个不抬 lo、它的末尾仍在 Q 里、恢复之后不算已确认的版本。
    /// 这里交 `FailedAfterWriting`：没返回的那一步总是最后一步（`translate` 里断言），之后没有操作；最后一个操作按「写了之后报错」交时
    /// 同样不抬 lo（lo 只认 `Succeeded`）、末尾同样在 Q 里（`ends_a_publish` 不看结局）、恢复之后同样不算已确认（`confirmed_publishes_among`
    /// 只认 `Succeeded`），崩溃落在都返回过之后（重开）与落在它里面（hi = 它）两种时刻上允许集合都与在途相同。
    ///
    /// 抛弃最新根的那次挂载要交被抛弃的根与 RefFS 最新那一版（`newest_version`）的关系，由 `roots_before_the_step` 定。
    fn events_of(
        &self,
        seen: &StepSeenByTheTranslation,
        versions_of_the_roots: &BTreeMap<ModelRootKey, VersionNumber>,
        roots_before_the_step: RootsBeforeTheStep,
        newest_version: VersionNumber,
    ) -> Result<EventsOfAStep, TranslationStopReason> {
        let publish_with_content = |change: Option<Operation>| {
            let publish_outcome = match &seen.ending {
                StepEndingSeenByTheTranslation::ReturnedOk { .. } => OperationOutcome::Succeeded,
                StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                | StepEndingSeenByTheTranslation::NeverReturned => {
                    OperationOutcome::FailedAfterWriting
                }
                StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
                | StepEndingSeenByTheTranslation::NotCalled => {
                    return EventsOfAStep::nothing();
                }
            };
            let mut events: Vec<ReferenceFileSystemEvent> = change
                .into_iter()
                .map(|operation| ReferenceFileSystemEvent::Submit {
                    operation,
                    outcome: OperationOutcome::Succeeded,
                })
                .collect();
            events.push(ReferenceFileSystemEvent::Submit {
                operation: Operation::ExplicitPublish,
                outcome: publish_outcome,
            });
            EventsOfAStep {
                events,
                new_version_is_written_by: NewVersionWrittenBy::TheLastRootOfTheStep,
                crash_inside: CrashInsideTheStep::Judged,
            }
        };
        let Some(operation) = seen.operation else {
            return Ok(match self.starting_point {
                HistoryStartingPoint::AfterMakeFilesystem => EventsOfAStep::nothing(),
                HistoryStartingPoint::AfterFirstFile => publish_with_content(Some(
                    Operation::CreateTheFirstFile(FileContent(first_file_content())),
                )),
            });
        };
        Ok(match operation {
            HistoryOperation::PublishFirstFile(content) => publish_with_content(Some(
                Operation::CreateTheFirstFile(FileContent(content.bytes())),
            )),
            HistoryOperation::PublishOverwrite(content) => publish_with_content(Some(
                Operation::OverwriteTheFile(FileContent(content.bytes())),
            )),
            HistoryOperation::PublishWithoutUnits => publish_with_content(None),
            // 截断（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8）：发成的不论几个版本（意图、各批、最后一批）都交一个
            // `Truncate(n)` 连同它那次发布——任何写之前拒的交空、写了之后报错的交「写了之后报错」，同覆盖写（`publish_with_content`）。
            // n 是实现交回的这一版的 size：执行时按那一刻的 size 取（`history::TruncationChoice`），翻译拿不到，改 size 做成时由执行器交回。
            // 意图那一版起读出的就是截后的（K2「先落目标」），所以做成时这一步从意图那次起写的根都映到新版本（`roots_carrying_the_change` 条）。
            HistoryOperation::Truncate(_) => match (&seen.ending, seen.size_requested_by_the_truncation) {
                (
                    StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
                    | StepEndingSeenByTheTranslation::NotCalled,
                    Some(_) | None,
                ) => EventsOfAStep::nothing(),
                (
                    StepEndingSeenByTheTranslation::ReturnedOk {
                        effect:
                            Some(AppliedEffect::Published {
                                roots_carrying_the_change,
                                ..
                            }),
                        ..
                    },
                    Some(new_size),
                ) => {
                    let mut events = publish_with_content(Some(Operation::Truncate(new_size)));
                    events.new_version_is_written_by = NewVersionWrittenBy::EveryRootFromTheChangeOn {
                        roots_of_the_change: *roots_carrying_the_change,
                    };
                    events
                }
                // 写了之后报错、没返回：照覆盖写交（改 size 那一次的根之后有没有别的根写下，崩在里面的允许集合都是截前、截后两样）。
                (
                    StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                    | StepEndingSeenByTheTranslation::NeverReturned,
                    Some(new_size),
                ) => publish_with_content(Some(Operation::Truncate(new_size))),
                (
                    StepEndingSeenByTheTranslation::ReturnedOk { .. }
                    | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                    | StepEndingSeenByTheTranslation::NeverReturned,
                    Some(_) | None,
                ) => {
                    return Err(TranslationStopReason::TheSizeChangeDidNotReportItsSize);
                }
            },
            HistoryOperation::RollBackWhileMounted(_) => match &seen.ending {
                StepEndingSeenByTheTranslation::ReturnedOk {
                    effect: Some(AppliedEffect::RolledBack { target, .. }),
                    ..
                } => {
                    let target_root = model_root_key(target.instance, target.checkpoint_txg);
                    let Some(version) = versions_of_the_roots.get(&target_root) else {
                        return Err(TranslationStopReason::TheRollbackTargetRootHasNoVersion {
                            target: target_root,
                        });
                    };
                    EventsOfAStep {
                        events: vec![ReferenceFileSystemEvent::Submit {
                            operation: Operation::RollBackToVersion(*version),
                            outcome: OperationOutcome::Succeeded,
                        }],
                        new_version_is_written_by: NewVersionWrittenBy::TheLastRootOfTheStep,
                        crash_inside: CrashInsideTheStep::Judged,
                    }
                }
                StepEndingSeenByTheTranslation::ReturnedOk {
                    effect:
                        Some(
                            AppliedEffect::Published { .. }
                            | AppliedEffect::Mounted { .. }
                            | AppliedEffect::RaisedFloor { .. }
                            | AppliedEffect::Recovered { .. }
                            | AppliedEffect::SnapshotPublished { .. }
                            | AppliedEffect::SnapshotRead { .. }
                            | AppliedEffect::Compacted { .. },
                        )
                        | None,
                    ..
                }
                | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                | StepEndingSeenByTheTranslation::NeverReturned => {
                    // 实现只在 Ok 里报回退根（`AppliedEffect::RolledBack`）；没报就翻不出 m。
                    return Err(TranslationStopReason::TheRollbackDidNotReportItsTarget);
                }
                StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
                | StepEndingSeenByTheTranslation::NotCalled => EventsOfAStep::nothing(),
            },
            // 按 inode 号写（里程碑「删除与多次 COW」一·1）：RefFS 第一版只有第一个文件，写了盘就翻不出（点名的号执行时才取，写的可能是别的文件）。
            HistoryOperation::WriteOfAnInode(_) => match &seen.ending {
                StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
                | StepEndingSeenByTheTranslation::NotCalled => EventsOfAStep::nothing(),
                StepEndingSeenByTheTranslation::ReturnedOk { .. }
                | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                | StepEndingSeenByTheTranslation::NeverReturned => {
                    return Err(
                        TranslationStopReason::TheReferenceFileSystemHasNoFileOtherThanTheFirstInTheFirstVersion,
                    );
                }
            },
            HistoryOperation::Unlink => match &seen.ending {
                StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
                | StepEndingSeenByTheTranslation::NotCalled => EventsOfAStep::nothing(),
                StepEndingSeenByTheTranslation::ReturnedOk { .. }
                | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                | StepEndingSeenByTheTranslation::NeverReturned => {
                    return Err(
                        TranslationStopReason::TheReferenceFileSystemHasNoUnlinkInTheFirstVersion,
                    );
                }
            },
            // 可写挂载与抬 F 不改内容：不进 RefFS 的操作序列（H1 只让发布调用、fsync、回退抬 lo），它们写的根映到现行版本。
            // 建 inode 只接在第一个文件删完之后（`HistoryOperation::CreateInode`），翻译到 unlink 写了盘那一步就已经停了；
            // 它也不改第一个文件的内容（新建的 inode 长度 0），照可写挂载与抬 F 那样不进操作序列。
            // 掉电在第一个 journal 记录写之前的那次覆盖写一样都没发布成（盘上没有能让恢复读到它的记录与根），之后那次可写挂载照可写挂载。
            // 每块盘最新那一槽系统配置读回全 0 的可写挂载：根与单元照常读得出、落到的就是现行那一版，与可写挂载同样不进操作序列。
            // 挂着时整理（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7）只搬单元的落点、不改用户看得见的内容，
            // 先走空的删除续做只放已删的文件与已删快照的块；批间崩溃之后的可写挂载照可写挂载。不进操作序列，写的根映到现行版本。
            HistoryOperation::CloseAndMountWritable
            | HistoryOperation::RaiseRollbackFloor(_)
            | HistoryOperation::CreateInode
            // 一次建几个 inode 同样不改第一个文件的内容（新建的长度 0）。
            | HistoryOperation::CreateInodes(_)
            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_)
            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros
            | HistoryOperation::CompactWhileMounted(_) => {
                EventsOfAStep::nothing()
            }
            // 建快照、删快照（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S7：RefFS 加 CreateSnapshot、DestroySnapshot）：
            // 实现只在做成时报快照的树号（RefFS 拿它当快照名）；没报就翻不出名字（同回退没报目标根）。删快照一次调用写意图与全部批几条根，
            // 只有最后一条映到删完之后的版本（之前的是会话推的抬 F、意图与不是最后一批的批，可见的内容都没变）。
            HistoryOperation::CreateSnapshot | HistoryOperation::DestroySnapshot(_) => {
                match &seen.ending {
                    StepEndingSeenByTheTranslation::ReturnedOk {
                        effect:
                            Some(AppliedEffect::SnapshotPublished {
                                change, snapshot, ..
                            }),
                        ..
                    } => {
                        let name = SnapshotName(*snapshot);
                        let snapshot_operation = match change {
                            SnapshotChange::Created => Operation::CreateSnapshot(name),
                            SnapshotChange::Destroyed => Operation::DestroySnapshot(name),
                        };
                        EventsOfAStep {
                            events: vec![ReferenceFileSystemEvent::Submit {
                                operation: snapshot_operation,
                                outcome: OperationOutcome::Succeeded,
                            }],
                            new_version_is_written_by: NewVersionWrittenBy::TheLastRootOfTheStep,
                            crash_inside: CrashInsideTheStep::Judged,
                        }
                    }
                    StepEndingSeenByTheTranslation::ReturnedOk {
                        effect:
                            Some(
                                AppliedEffect::Published { .. }
                                | AppliedEffect::Mounted { .. }
                                | AppliedEffect::RaisedFloor { .. }
                                | AppliedEffect::RolledBack { .. }
                                | AppliedEffect::Recovered { .. }
                                | AppliedEffect::SnapshotRead { .. }
                                | AppliedEffect::Compacted { .. },
                            )
                            | None,
                        ..
                    }
                    | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                    | StepEndingSeenByTheTranslation::NeverReturned => {
                        return Err(TranslationStopReason::TheSnapshotChangeDidNotReportItsSnapshot);
                    }
                    StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
                    | StepEndingSeenByTheTranslation::NotCalled => EventsOfAStep::nothing(),
                }
            }
            // 读快照不写盘、不进操作序列：读出的内容拿 RefFS 记的那个快照比（判决 S7「读快照」）。
            HistoryOperation::ReadSnapshot(_) => match &seen.ending {
                StepEndingSeenByTheTranslation::ReturnedOk {
                    effect:
                        Some(AppliedEffect::SnapshotRead {
                            snapshot,
                            read_back,
                        }),
                    ..
                } => EventsOfAStep {
                    events: vec![ReferenceFileSystemEvent::ReadSnapshot {
                        name: SnapshotName(*snapshot),
                        read_out: snapshot_read_out_of(read_back),
                    }],
                    new_version_is_written_by: NewVersionWrittenBy::EveryRootOfTheStep,
                    crash_inside: CrashInsideTheStep::Judged,
                },
                StepEndingSeenByTheTranslation::ReturnedOk { .. }
                | StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
                | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                | StepEndingSeenByTheTranslation::NeverReturned
                | StepEndingSeenByTheTranslation::NotCalled => EventsOfAStep::nothing(),
            },
            HistoryOperation::ColdStartRecover => match &seen.ending {
                StepEndingSeenByTheTranslation::ReturnedOk {
                    read_out_of_the_recovery_in_the_step: Some(read_out),
                    ..
                } => EventsOfAStep {
                    events: vec![ReferenceFileSystemEvent::RecoverWithEveryOperationReturned(
                        read_out.clone(),
                    )],
                    new_version_is_written_by: NewVersionWrittenBy::EveryRootOfTheStep,
                    crash_inside: CrashInsideTheStep::Judged,
                },
                StepEndingSeenByTheTranslation::ReturnedOk {
                    read_out_of_the_recovery_in_the_step: None,
                    ..
                }
                | StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
                | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                | StepEndingSeenByTheTranslation::NeverReturned
                | StepEndingSeenByTheTranslation::NotCalled => EventsOfAStep::nothing(),
            },
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => {
                let events = match &seen.ending {
                    StepEndingSeenByTheTranslation::ReturnedOk {
                        read_out_of_the_recovery_in_the_step: Some(read_out),
                        ..
                    } => vec![
                        ReferenceFileSystemEvent::RestartAfterARecoveryThatAbandonedTheNewestRoot {
                            abandoned_root: roots_before_the_step
                                .relation_of_the_abandoned_root_to_the_newest_version(
                                    versions_of_the_roots,
                                    newest_version,
                                )?,
                            read_out: read_out.clone(),
                        },
                    ],
                    StepEndingSeenByTheTranslation::ReturnedOk {
                        read_out_of_the_recovery_in_the_step: None,
                        ..
                    }
                    | StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite
                    | StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten
                    | StepEndingSeenByTheTranslation::NeverReturned
                    | StepEndingSeenByTheTranslation::NotCalled => Vec::new(),
                };
                // 这一步的恢复带读故障（藏起来的那几处），崩溃镜像上读不到那份故障：崩在它里面的状态 RefFS 不判。
                EventsOfAStep {
                    events,
                    new_version_is_written_by: NewVersionWrittenBy::EveryRootOfTheStep,
                    crash_inside: CrashInsideTheStep::NotJudgedBecauseItsRecoveryHadReadFaults,
                }
            }
        })
    }
}

/// 一步翻出来的 RefFS 事件与它写的根怎么映版本。
struct EventsOfAStep {
    events: Vec<ReferenceFileSystemEvent>,
    new_version_is_written_by: NewVersionWrittenBy,
    crash_inside: CrashInsideTheStep,
}

impl EventsOfAStep {
    fn nothing() -> Self {
        Self {
            events: Vec::new(),
            new_version_is_written_by: NewVersionWrittenBy::EveryRootOfTheStep,
            crash_inside: CrashInsideTheStep::Judged,
        }
    }
}

/// 一步出了新版本时，这一步写的根里哪几条映到新版本：发布与回退只有最后一条根是它自己的（之前的是会话推的抬 F），
/// 恢复在这一步的写之前（之后的写都是以它重新起头的挂载写的）。没出新版本时全映到现行版本，两种一样。
/// 截断写的几条根（意图、各批、最后一批与其间推的抬 F）从意图那一条起读出的都是截后的内容（判决
/// `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K2「先落目标」）：这一步的最后 `roots_of_the_change` 条映到新版本，写出它的是其中第一条。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NewVersionWrittenBy {
    TheLastRootOfTheStep,
    EveryRootFromTheChangeOn { roots_of_the_change: usize },
    EveryRootOfTheStep,
}

/// 一步之前写下的根里，抛弃最新根的那次挂载要用的两样（判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1：
/// 翻译把「被抛弃的根是不是写出这一版的那一条」交进 RefFS）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RootsBeforeTheStep {
    /// 这一步之前写过的根里最新的那条（择根按 txg 为主、实例代号破平局，即 [`ModelRootKey`] 的次序）：根环里最新那条，
    /// 抛弃最新根的那次挂载抛弃的就是它。一条都没写过时没有。
    newest_root: Option<ModelRootKey>,
    /// 写出 RefFS 最新那一版的那条根：出新版本的那一步只有最后一条根映到新版本时（发布调用、回退、建快照、删快照）是它写的最后一条；
    /// 恢复出的版本（冷启动、抛弃最新根的那次挂载）没有——那一步写的根都在恢复之后，映到这一版却不是写出它的那一条。
    root_that_wrote_the_newest_version: Option<ModelRootKey>,
}

impl RootsBeforeTheStep {
    fn before_any_step() -> Self {
        Self {
            newest_root: None,
            root_that_wrote_the_newest_version: None,
        }
    }

    /// 一步写完它的根（`roots_of_the_step`，按流里的次序）之后：最新那条根并进这一步写的；这一步出了新版本时，写出它的那条根照
    /// `new_version_is_written_by` 换（与 `translate` 里给这一步的根映版本同一个分法），没出新版本时不变。
    fn after_the_step(
        self,
        roots_of_the_step: &[ModelRootKey],
        new_version_is_written_by: NewVersionWrittenBy,
        version_before_the_step: VersionNumber,
        version_after_the_step: VersionNumber,
    ) -> Self {
        let newest_root = roots_of_the_step
            .iter()
            .copied()
            .chain(self.newest_root)
            .max();
        let root_that_wrote_the_newest_version =
            if version_after_the_step == version_before_the_step {
                self.root_that_wrote_the_newest_version
            } else {
                match new_version_is_written_by {
                    NewVersionWrittenBy::TheLastRootOfTheStep => roots_of_the_step.last().copied(),
                    NewVersionWrittenBy::EveryRootFromTheChangeOn {
                        roots_of_the_change,
                    } => roots_of_the_step
                        .len()
                        .checked_sub(roots_of_the_change)
                        .and_then(|first_root_of_the_change| {
                            roots_of_the_step.get(first_root_of_the_change).copied()
                        }),
                    NewVersionWrittenBy::EveryRootOfTheStep => None,
                }
            };
        Self {
            newest_root,
            root_that_wrote_the_newest_version,
        }
    }

    /// 抛弃最新根的那次挂载抛弃的那条根与 RefFS 最新那一版（`newest_version`）是什么关系：是写出它的那一条，或映到它而不是写出它的那一条。
    ///
    /// # Errors
    /// 被抛弃的根不映到最新那一版、或这一步之前一条根都没写过：[`TranslationStopReason::TheAbandonedRootIsNotARootOfTheNewestVersion`]。
    fn relation_of_the_abandoned_root_to_the_newest_version(
        self,
        versions_of_the_roots: &BTreeMap<ModelRootKey, VersionNumber>,
        newest_version: VersionNumber,
    ) -> Result<AbandonedRoot, TranslationStopReason> {
        let not_a_root_of_the_newest_version =
            TranslationStopReason::TheAbandonedRootIsNotARootOfTheNewestVersion {
                abandoned: self.newest_root,
            };
        let Some(abandoned) = self.newest_root else {
            return Err(not_a_root_of_the_newest_version);
        };
        if self.root_that_wrote_the_newest_version == Some(abandoned) {
            return Ok(AbandonedRoot::WroteTheNewestVersion);
        }
        match versions_of_the_roots.get(&abandoned) {
            Some(version) if *version == newest_version => {
                Ok(AbandonedRoot::WrittenAfterTheNewestVersion)
            }
            Some(_) | None => Err(not_a_root_of_the_newest_version),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrashInsideTheStep {
    Judged,
    NotJudgedBecauseItsRecoveryHadReadFaults,
}

/// 交给 RefFS 的一件事。
#[derive(Clone, Debug, PartialEq, Eq)]
enum ReferenceFileSystemEvent {
    Submit {
        operation: Operation,
        outcome: OperationOutcome,
    },
    /// 序列里的操作都返回过之后的一次恢复（冷启动）：判读出的内容，以它重新起头。
    RecoverWithEveryOperationReturned(VisibleState),
    /// 抛弃最新根的那次恢复：不判，以读出的内容重新起头；快照集合按被抛弃的根与最新那一版的关系取。
    RestartAfterARecoveryThatAbandonedTheNewestRoot {
        abandoned_root: AbandonedRoot,
        read_out: VisibleState,
    },
    /// 读了一个快照：读出的要等于 RefFS 记的那个快照（建它那一刻的可见状态）；读失败时是失败那一句。
    ReadSnapshot {
        name: SnapshotName,
        read_out: Result<VisibleState, String>,
    },
}

/// 读快照读出的东西按 RefFS 的可见状态写；读不出、树表里没有那个快照的交回那一句。
fn snapshot_read_out_of(read_back: &ObservedSnapshotReadBack) -> Result<VisibleState, String> {
    match read_back {
        ObservedSnapshotReadBack::NoFile => Ok(VisibleState::NoFile),
        ObservedSnapshotReadBack::File(content) => {
            Ok(VisibleState::OneFile(FileContent(content.clone())))
        }
        ObservedSnapshotReadBack::NoSnapshotWithThisTree => Err("树表里没有这个快照".to_string()),
        ObservedSnapshotReadBack::Failed { what } => Err(format!("读失败：{what}")),
    }
}

/// 事件交进 RefFS 之后：进了序列的操作号，或者没有。
enum EventApplied {
    Entered(OperationNumber),
    NothingEntered,
}

// gpu-exempt: reference-filesystem
fn apply_event(
    model: &mut ReferenceFileSystem,
    event: &ReferenceFileSystemEvent,
) -> Result<EventApplied, StoppedVerdict> {
    match event {
        ReferenceFileSystemEvent::Submit { operation, outcome } => {
            match model.submit(operation.clone(), *outcome) {
                Ok(SubmittedOperation::Entered(number)) => Ok(EventApplied::Entered(number)),
                Ok(SubmittedOperation::DidNotHappen) => Ok(EventApplied::NothingEntered),
                Err(refusal) => Err(StoppedVerdict::Abstained(
                    TranslationStopReason::TheReferenceFileSystemDoesNotTakeTheOperation(refusal),
                )),
            }
        }
        ReferenceFileSystemEvent::RecoverWithEveryOperationReturned(read_out) => {
            let crash = model.crash_after_every_operation_returned();
            match model.crash_and_recover(crash, read_out.clone()) {
                Ok(_) => Ok(EventApplied::NothingEntered),
                Err(refusal) => Err(stopped_verdict_of_a_refused_recovery(refusal)),
            }
        }
        ReferenceFileSystemEvent::RestartAfterARecoveryThatAbandonedTheNewestRoot {
            abandoned_root,
            read_out,
        } => {
            let crash = model.crash_after_every_operation_returned();
            match model.restart_after_a_recovery_that_abandoned_the_newest_root(
                crash,
                *abandoned_root,
                read_out.clone(),
            ) {
                Ok(()) => Ok(EventApplied::NothingEntered),
                Err(AbandonedRootRestartNotAccepted::TheCrashIsNotAccepted(refusal)) => {
                    Err(stopped_verdict_of_a_refused_recovery(refusal))
                }
                Err(
                    refusal @ AbandonedRootRestartNotAccepted::TheRootThatWroteTheStateAtTheLastRecoveryIsNotInTheFirstVersion,
                ) => panic!(
                    "翻译只在出新版本的那一步只有最后一条根映到新版本时记写出它的根（发布调用、回退、建快照、删快照都是已确认的发布），\
                     恢复出的版本不记，所以交「写出最新那一版的那一条」时最新那一版是上一次恢复之后的一次已确认发布，允许集合里没有 S_0：{refusal:?}"
                ),
            }
        }
        // 恢复之后允许的快照集合可以不止一个（判决 `research/prompts/m4-segment-code-r1-main-verification.md` 第三节 F6）：
        // 读出的要是其中某一个里这个快照的内容，判过之后只留下读得出它的那几个。
        ReferenceFileSystemEvent::ReadSnapshot { name, read_out } => {
            let allowed_reads = model.allowed_reads_of_the_snapshot(*name);
            let read_out_is_the_kept_state = match read_out {
                Ok(read_state) => model
                    .judge_a_snapshot_read_and_keep_the_snapshot_sets_it_allows(*name, read_state)
                    .is_ok(),
                Err(_) => false,
            };
            if read_out_is_the_kept_state {
                Ok(EventApplied::NothingEntered)
            } else {
                let described_allowed_reads: Vec<String> = allowed_reads
                    .iter()
                    .map(|allowed_read| {
                        allowed_read
                            .as_ref()
                            .map_or_else(|| "没有这个快照".to_string(), describe_visible_state)
                    })
                    .collect();
                Err(StoppedVerdict::Red {
                    reason: format!(
                        "历史里读快照 {} 读出 {}，RefFS 允许的是 {{{}}}",
                        name.0,
                        read_out
                            .as_ref()
                            .map_or_else(Clone::clone, describe_visible_state),
                        described_allowed_reads.join("，")
                    ),
                })
            }
        }
    }
}

fn apply_every_event(
    model: &mut ReferenceFileSystem,
    events: &[ReferenceFileSystemEvent],
) -> Result<Option<OperationNumber>, StoppedVerdict> {
    let mut last_entered = None;
    for event in events {
        match apply_event(model, event)? {
            EventApplied::Entered(number) => last_entered = Some(number),
            EventApplied::NothingEntered => {}
        }
    }
    Ok(last_entered)
}

fn stopped_verdict_of_a_refused_recovery(refusal: CrashNotAccepted) -> StoppedVerdict {
    match refusal {
        CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { allowed, recovered } => {
            StoppedVerdict::Red {
                reason: format!(
                    "历史里一次恢复读出 {}，允许集合 {}",
                    describe_visible_state(&recovered),
                    describe_allowed_states(&allowed.allowed)
                ),
            }
        }
        CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion { .. } => {
            StoppedVerdict::Abstained(TranslationStopReason::AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion)
        }
        CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery { .. } => {
            panic!("历史里的恢复取的 hi 是「序列里的操作都返回过之后」那个号，不会越界：{refusal:?}")
        }
    }
}

/// 给人看：没有文件，或文件多少字节、末字节（内容可以有几万字节，不整份打）。
fn describe_visible_state(state: &VisibleState) -> String {
    match state {
        VisibleState::NoFile => "没有文件".to_string(),
        VisibleState::OneFile(FileContent(bytes)) => {
            format!("文件 {} 字节（末字节 {:?}）", bytes.len(), bytes.last())
        }
    }
}

fn describe_allowed_states(allowed: &[AllowedState]) -> String {
    let described: Vec<String> = allowed
        .iter()
        .map(|allowed_state| {
            format!(
                "S_{} = {}",
                allowed_state.index.0,
                describe_visible_state(&allowed_state.state)
            )
        })
        .collect();
    format!("{{{}}}", described.join("，"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TranslatedStep {
    position: StepPosition,
    events: Vec<ReferenceFileSystemEvent>,
    crash_inside: CrashInsideTheStep,
    stream_length_after_the_step: usize,
}

impl TranslatedStep {
    /// 翻译停在这一步：它与之后的步都不交给 RefFS。
    fn without_events(position: StepPosition) -> Self {
        Self {
            position,
            events: Vec::new(),
            crash_inside: CrashInsideTheStep::Judged,
            stream_length_after_the_step: usize::MAX,
        }
    }
}

/// 翻译为什么停在某一步。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TranslationStopReason {
    /// 回退报的目标根不在 m 表里（m 表收这一池里写过根的每一个入口；查不到说明表漏收了一种入口）。
    TheRollbackTargetRootHasNoVersion { target: ModelRootKey },
    /// 回退没返回 Ok（报错或没返回）：实现只在 Ok 里报回退根，翻不出 m。
    TheRollbackDidNotReportItsTarget,
    /// RefFS 不收这一步（第一版不支持的操作，或回退目标不是一个版本）。
    TheReferenceFileSystemDoesNotTakeTheOperation(OperationNotAccepted),
    /// 一个操作写了之后报错、恢复之前又起了操作（RefFS 第一版不支持）。
    AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion,
    /// unlink 写了盘（意图那次或之后几步落了盘）：RefFS 第一版没有删除操作，翻不出那几版。
    TheReferenceFileSystemHasNoUnlinkInTheFirstVersion,
    /// 按 inode 号写写了盘：RefFS 第一版只有第一个文件，翻不出写别的文件的那几版。
    TheReferenceFileSystemHasNoFileOtherThanTheFirstInTheFirstVersion,
    /// 建快照、删快照没返回 Ok（报错或没返回）：实现只在做成时报快照的树号，翻不出快照名。
    TheSnapshotChangeDidNotReportItsSnapshot,
    /// 抛弃最新根的那次挂载：被抛弃的根（这一步之前写过的最新那条）不映到 RefFS 最新那一版（例如冷启动恢复没写根就接着抛弃，
    /// 被抛弃的是恢复之前那一版的根），或这一步之前一条根都没写过。RefFS 只记上一次恢复之后的快照集合，判决
    /// `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1 只写了「是不是写出这一版的那一条」两种，这一支交什么没写，第一版不支持。
    TheAbandonedRootIsNotARootOfTheNewestVersion { abandoned: Option<ModelRootKey> },
    /// 截断调了入口、执行器却没交回它交给会话的目标 size（截断那一步的观察里没有 `size_requested_by_the_truncation`），
    /// 或做成了却不是 `AppliedEffect::Published`：翻不出 `Operation::Truncate(n)`。执行器每次调截断都记下目标 size，走到这里是执行器漏记了。
    TheSizeChangeDidNotReportItsSize,
}

impl TranslationStopReason {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            TranslationStopReason::TheSizeChangeDidNotReportItsSize => {
                "截断调了入口、执行器没交回目标 size"
            }
            TranslationStopReason::TheRollbackTargetRootHasNoVersion { .. } => {
                "回退目标根在 m 表里查不到"
            }
            TranslationStopReason::TheRollbackDidNotReportItsTarget => "回退没返回 Ok、没报目标根",
            TranslationStopReason::TheReferenceFileSystemDoesNotTakeTheOperation(_) => {
                "RefFS 不收这一步"
            }
            TranslationStopReason::AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion => {
                "写后报错、恢复之前又起了操作"
            }
            TranslationStopReason::TheReferenceFileSystemHasNoFileOtherThanTheFirstInTheFirstVersion => {
                "RefFS 第一版只有第一个文件"
            }
            TranslationStopReason::TheReferenceFileSystemHasNoUnlinkInTheFirstVersion => {
                "RefFS 第一版没有删除"
            }
            TranslationStopReason::TheSnapshotChangeDidNotReportItsSnapshot => {
                "建快照或删快照没返回 Ok、没报快照树号"
            }
            TranslationStopReason::TheAbandonedRootIsNotARootOfTheNewestVersion { .. } => {
                "被抛弃的根不在 RefFS 最新那一版上"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StoppedVerdict {
    Abstained(TranslationStopReason),
    /// 历史里一次恢复（冷启动）读出的内容 RefFS 就判红了：之后的崩溃都带着这一判。
    Red {
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TranslationStopped {
    position: StepPosition,
    verdict: StoppedVerdict,
}

/// 崩溃落在哪：切开当前段的那道屏障（或那次 FUA 写）由哪一步发出；当前段到流尾都没关上的，算在全部步都返回之后。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceFileSystemCrashMoment {
    InsideStep(StepPosition),
    AfterEveryTranslatedStep,
}

/// 每一段录制流由录制流里第几步关上（关上它的那道屏障或那次 FUA 写的下标）；流尾那一段没关上的是 `None`。
/// 切法与 `memory_pool::writes_and_segments_with_stream_indexes` 同一条规则（[`SegmentClosingRule`]），段号一一对应。
#[must_use]
pub fn segment_closing_stream_indexes(operations: &[RetainedOperation]) -> Vec<Option<usize>> {
    let mut closing_stream_indexes = Vec::new();
    let mut closing_rule = SegmentClosingRule::default();
    for (stream_index, retained) in operations.iter().enumerate() {
        match closing_rule.after(&retained.operation) {
            SegmentAfterOperation::Closes => closing_stream_indexes.push(Some(stream_index)),
            SegmentAfterOperation::StaysOpen => {}
        }
    }
    if closing_rule.current_segment_has_writes() {
        closing_stream_indexes.push(None);
    }
    closing_stream_indexes
}

/// 关上这一段的那一步：第一个「跑完时流里的步数」大于关段下标的那一条登记。登记里找不到（关段的那一步没登记，
/// 例如历史停下的那一步），或这一段没关上，都算在全部步都返回之后。
#[must_use]
pub fn crash_moment_of_a_segment_closed_at(
    closing_stream_index: Option<usize>,
    stream_length_after_each_step: &[(StepPosition, usize)],
) -> ReferenceFileSystemCrashMoment {
    let Some(closing_stream_index) = closing_stream_index else {
        return ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep;
    };
    stream_length_after_each_step
        .iter()
        .find(|(_, stream_length)| *stream_length > closing_stream_index)
        .map_or(
            ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
            |(position, _)| ReferenceFileSystemCrashMoment::InsideStep(*position),
        )
}

/// RefFS 对一个崩溃状态投的票。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceFileSystemVote {
    Green,
    Red { reason: String },
    Abstained(ReferenceFileSystemAbstention),
}

/// RefFS 为什么不投票。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceFileSystemAbstention {
    /// 故障注入之后、注入真落了的状态带故障（每一档都是：写报错、写被吞、读报错、读回翻位、刷盘报错、刷盘被吞）：U1 的射程句
    /// 「带故障的状态（说谎的设备、丢整块盘 C519）不适用」管全部带故障的状态，括号里是举例（调度记录
    /// `records/2026-10-01-里程碑三形式化接入.md` 第 12 批「12-2 交回」那一条）；O1 判官照旧。注入点没走到的（[`FaultOutcome::NeverFired`]）
    /// 不带故障，照判（代码轮第三轮判决 `research/prompts/m3-formal-code-r3-main-verification.md` 第三节第 1 条）。
    TheStateCarriesAnInjectedFault,
    /// 故障注入之后盘上本来就没有一版立得住（mkfs 自己报错），重开走读失败正是要的结局（`fault_injection::failed_reopen_is_the_right_answer`）。
    NoVersionStandsAfterTheInjectedFault,
    /// 翻译在崩溃之前（或崩溃落在的那一步）停了。
    TheTranslationStoppedBeforeTheCrash(TranslationStopReason),
    /// 崩溃落在一次带读故障的恢复那一步里（抛弃最新根的那次挂载）。
    TheCrashIsInsideAStepWhoseRecoveryHadReadFaults,
    /// 挂载途中的二次崩溃：第一次崩溃那一判 RefFS 没判绿，没有重新起头的起点。
    TheFirstCrashWasNotJudgedGreen,
    /// 写了之后报错的操作后面到崩溃为止又起了操作（RefFS 第一版不支持）。
    AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion,
}

impl ReferenceFileSystemAbstention {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            ReferenceFileSystemAbstention::TheStateCarriesAnInjectedFault => {
                "带故障的状态，U1 射程外"
            }
            ReferenceFileSystemAbstention::NoVersionStandsAfterTheInjectedFault => {
                "注入之后盘上没有一版立得住"
            }
            ReferenceFileSystemAbstention::TheTranslationStoppedBeforeTheCrash(reason) => {
                reason.name()
            }
            ReferenceFileSystemAbstention::TheCrashIsInsideAStepWhoseRecoveryHadReadFaults => {
                "崩在带读故障的那次恢复里"
            }
            ReferenceFileSystemAbstention::TheFirstCrashWasNotJudgedGreen => {
                "第一次崩溃 RefFS 没判绿"
            }
            ReferenceFileSystemAbstention::AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion => {
                "写后报错之后又起了操作"
            }
        }
    }
}

impl ReferenceFileSystemVote {
    /// 判红时记进失败观察违例那一栏的一项（[`REFERENCE_FILESYSTEM_VIOLATION_NAME`]）；别的票没有。
    #[must_use]
    pub fn violation(&self) -> Option<(&'static str, String)> {
        match self {
            ReferenceFileSystemVote::Red { reason } => {
                Some((REFERENCE_FILESYSTEM_VIOLATION_NAME, reason.clone()))
            }
            ReferenceFileSystemVote::Green | ReferenceFileSystemVote::Abstained(_) => None,
        }
    }
}

/// RefFS 与 O1 判官两票合起来（G1 用户定案：任一红即红），按哪一边红分开计。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JointVerdictOfTheReferenceFileSystemAndTheModel {
    BothGreen,
    OnlyTheReferenceFileSystemRed,
    OnlyTheModelRed,
    BothRed,
    ReferenceFileSystemAbstainedAndTheModelGreen,
    ReferenceFileSystemAbstainedAndTheModelRed,
}

impl JointVerdictOfTheReferenceFileSystemAndTheModel {
    #[must_use]
    pub fn of(vote: &ReferenceFileSystemVote, model: Option<&ModelDisagreement>) -> Self {
        match (vote, model) {
            (ReferenceFileSystemVote::Green, None) => Self::BothGreen,
            (ReferenceFileSystemVote::Red { .. }, None) => Self::OnlyTheReferenceFileSystemRed,
            (ReferenceFileSystemVote::Green, Some(_)) => Self::OnlyTheModelRed,
            (ReferenceFileSystemVote::Red { .. }, Some(_)) => Self::BothRed,
            (ReferenceFileSystemVote::Abstained(_), None) => {
                Self::ReferenceFileSystemAbstainedAndTheModelGreen
            }
            (ReferenceFileSystemVote::Abstained(_), Some(_)) => {
                Self::ReferenceFileSystemAbstainedAndTheModelRed
            }
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::BothGreen => "两者都绿",
            Self::OnlyTheReferenceFileSystemRed => "只 RefFS 红",
            Self::OnlyTheModelRed => "只 O1 红",
            Self::BothRed => "两者都红",
            Self::ReferenceFileSystemAbstainedAndTheModelGreen => "RefFS 弃权、O1 绿",
            Self::ReferenceFileSystemAbstainedAndTheModelRed => "RefFS 弃权、O1 红",
        }
    }
}

/// 故障注入之后的重开要读出什么（`fault_injection::failed_reopen_is_the_right_answer` 的两种答案）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhatTheReopenMustRecover {
    AVersionTheHistoryLeft,
    NothingBecauseNoVersionStands,
}

/// 一次崩溃 RefFS 的判定：票，判绿时连同以读出的内容重新起头之后的 RefFS（挂载途中再崩一次时接着判）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceFileSystemJudgementOfACrash {
    pub vote: ReferenceFileSystemVote,
    pub restarted: Option<ReferenceFileSystem>,
}

/// 翻好的一段历史：每一步的 RefFS 事件、翻译停在哪、m 表、每一段由哪一道屏障关上。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranslatedHistory {
    steps: Vec<TranslatedStep>,
    stopped: Option<TranslationStopped>,
    versions_of_the_roots: BTreeMap<ModelRootKey, VersionNumber>,
    segment_closing_stream_indexes: Vec<Option<usize>>,
}

/// 恢复读回的东西按 RefFS 的可见状态写；读失败的交回失败那一句。
fn read_out_of(read_back: &ObservedReadBack) -> Result<VisibleState, String> {
    match read_back {
        ObservedReadBack::FileRead { content, .. } => {
            Ok(VisibleState::OneFile(FileContent(content.clone())))
        }
        ObservedReadBack::NoFile { .. } => Ok(VisibleState::NoFile),
        ObservedReadBack::Failed { what } => Err(what.clone()),
    }
}

/// 在 `model` 上判一次崩溃（落在第 `crash_falls_in` 个操作里）之后恢复读回的东西。
// gpu-exempt: reference-filesystem
fn judge_the_recovery(
    mut model: ReferenceFileSystem,
    crash_falls_in: OperationNumber,
    read_back: &ObservedReadBack,
) -> ReferenceFileSystemJudgementOfACrash {
    let recovered = match read_out_of(read_back) {
        Ok(recovered) => recovered,
        Err(what) => {
            return ReferenceFileSystemJudgementOfACrash {
                vote: ReferenceFileSystemVote::Red {
                    reason: format!("恢复失败、没有读出的状态（{what}）"),
                },
                restarted: None,
            }
        }
    };
    match model.crash_and_recover(crash_falls_in, recovered) {
        Ok(_) => ReferenceFileSystemJudgementOfACrash {
            vote: ReferenceFileSystemVote::Green,
            restarted: Some(model),
        },
        Err(CrashNotAccepted::RecoveredStateOutsideTheAllowedSet { allowed, recovered }) => {
            ReferenceFileSystemJudgementOfACrash {
                vote: ReferenceFileSystemVote::Red {
                    reason: format!(
                        "崩溃落在第 {} 个操作里，读出 {}，允许集合 {}",
                        crash_falls_in.0,
                        describe_visible_state(&recovered),
                        describe_allowed_states(&allowed.allowed)
                    ),
                },
                restarted: None,
            }
        }
        Err(CrashNotAccepted::OperationAfterAFailureAfterWritingBeforeTheNextRecoveryIsNotInTheFirstVersion { .. }) => {
            ReferenceFileSystemJudgementOfACrash {
                vote: ReferenceFileSystemVote::Abstained(
                    ReferenceFileSystemAbstention::AnOperationAfterAFailureAfterWritingIsNotInTheFirstVersion,
                ),
                restarted: None,
            }
        }
        Err(refusal @ CrashNotAccepted::CrashOperationIsOutsideTheOperationsSinceTheLastRecovery { .. }) => {
            panic!("hi 取的是这一步最后进序列的操作号或「都返回过之后」那个号，不会越界：{refusal:?}")
        }
    }
}

/// 挂载途中的二次崩溃：第一次崩溃判绿、以它读出的内容重新起头之后，挂载不进 RefFS 的操作序列，二次崩溃落在「都返回过之后」。
/// 第一次没判绿（没有重新起头的 RefFS）的弃权。
#[must_use]
pub fn judge_a_second_crash_after_the_restart(
    first_crash: &ReferenceFileSystemJudgementOfACrash,
    read_back: &ObservedReadBack,
) -> ReferenceFileSystemVote {
    let Some(restarted) = &first_crash.restarted else {
        return ReferenceFileSystemVote::Abstained(
            ReferenceFileSystemAbstention::TheFirstCrashWasNotJudgedGreen,
        );
    };
    let crash = restarted.crash_after_every_operation_returned();
    judge_the_recovery(restarted.clone(), crash, read_back).vote
}

/// 这一档注入之后的状态在不在 RefFS 的射程外（[`ReferenceFileSystemAbstention::TheStateCarriesAnInjectedFault`]）。
/// 每一档单列一臂：加一档注入时编译器逼着在这里定它，不跟着别的档一起落进射程里。
const fn the_state_after_the_fault_is_outside_the_reference_filesystem_range(
    fault: InjectedFault,
) -> bool {
    match fault {
        InjectedFault::WriteFails => true,
        InjectedFault::WriteIsSwallowed => true,
        InjectedFault::ReadFails => true,
        InjectedFault::ReadReturnsCorruptedBytes { .. } => true,
        InjectedFault::BarrierFails => true,
        InjectedFault::BarrierIsSwallowed => true,
    }
}

impl TranslatedHistory {
    /// 录制流切出的段数：调用方核它与自己切的段数相同（同一条流、同一条规则）。
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segment_closing_stream_indexes.len()
    }

    /// 崩在第 `segment_index` 段里的崩溃落在哪一步：关上这一段的那一步。
    ///
    /// # Panics
    /// 段号超出这条流切出的段数（调用方的段与这里不是同一条流切的）。
    #[must_use]
    pub fn crash_moment_in_segment(&self, segment_index: usize) -> ReferenceFileSystemCrashMoment {
        let closing_stream_index = *self
            .segment_closing_stream_indexes
            .get(segment_index)
            .expect("段号来自同一条录制流、同一条切段规则");
        let stream_length_after_each_step: Vec<(StepPosition, usize)> = self
            .steps
            .iter()
            .map(|step| (step.position, step.stream_length_after_the_step))
            .collect();
        crash_moment_of_a_segment_closed_at(closing_stream_index, &stream_length_after_each_step)
    }

    /// m 表里这条根的版本号；不在表里是 None。
    #[must_use]
    pub fn version_of_the_root(&self, root: ModelRootKey) -> Option<VersionNumber> {
        self.versions_of_the_roots.get(&root).copied()
    }

    /// 从头把事件交给一个新的 RefFS，到崩溃为止；交回它与 hi。崩溃落在某一步里时 hi 是那一步最后进序列的操作，
    /// 那一步一个操作都没进序列（挂载、抬 F）时是「都返回过之后」那个号。
    fn replay_up_to(
        &self,
        moment: ReferenceFileSystemCrashMoment,
    ) -> Result<(ReferenceFileSystem, OperationNumber), ReferenceFileSystemVote> {
        let mut model = ReferenceFileSystem::on_a_freshly_formatted_pool();
        for step in &self.steps {
            if let Some(stopped) = self
                .stopped
                .as_ref()
                .filter(|stopped| stopped.position == step.position)
            {
                return Err(match &stopped.verdict {
                    StoppedVerdict::Abstained(reason) => ReferenceFileSystemVote::Abstained(
                        ReferenceFileSystemAbstention::TheTranslationStoppedBeforeTheCrash(
                            reason.clone(),
                        ),
                    ),
                    StoppedVerdict::Red { reason } => ReferenceFileSystemVote::Red {
                        reason: reason.clone(),
                    },
                });
            }
            let crash_is_inside_this_step =
                moment == ReferenceFileSystemCrashMoment::InsideStep(step.position);
            if crash_is_inside_this_step
                && step.crash_inside == CrashInsideTheStep::NotJudgedBecauseItsRecoveryHadReadFaults
            {
                return Err(ReferenceFileSystemVote::Abstained(
                    ReferenceFileSystemAbstention::TheCrashIsInsideAStepWhoseRecoveryHadReadFaults,
                ));
            }
            let last_entered = apply_every_event(&mut model, &step.events)
                .expect("翻译时同一串事件交给同一个起点的 RefFS 收过");
            if crash_is_inside_this_step {
                let crash_falls_in =
                    last_entered.unwrap_or_else(|| model.crash_after_every_operation_returned());
                return Ok((model, crash_falls_in));
            }
        }
        let crash_falls_in = model.crash_after_every_operation_returned();
        Ok((model, crash_falls_in))
    }

    /// 一次崩溃：照 `moment` 交事件到崩溃为止，判恢复读回的东西。
    #[must_use]
    pub fn judge_a_crash(
        &self,
        moment: ReferenceFileSystemCrashMoment,
        read_back: &ObservedReadBack,
    ) -> ReferenceFileSystemJudgementOfACrash {
        match self.replay_up_to(moment) {
            Ok((model, crash_falls_in)) => judge_the_recovery(model, crash_falls_in, read_back),
            Err(vote) => ReferenceFileSystemJudgementOfACrash {
                vote,
                restarted: None,
            },
        }
    }

    /// 故障注入之后的重开：历史停下之后崩（崩溃落在「都返回过之后」）。注入真落了（`fault_outcome` 不是
    /// [`FaultOutcome::NeverFired`]）而这一档注入之后的状态在 RefFS 射程外的弃权
    /// （[`the_state_after_the_fault_is_outside_the_reference_filesystem_range`]，今天每一档都是）；注入点没走到的不带故障、照判；
    /// 盘上本来没有一版立得住的弃权（重开走读失败是对的，O1 一侧同样不判）。
    #[must_use]
    pub fn judge_a_reopen_after_an_injected_fault(
        &self,
        fault: InjectedFault,
        fault_outcome: FaultOutcome,
        what_the_reopen_must_recover: WhatTheReopenMustRecover,
        read_back: &ObservedReadBack,
    ) -> ReferenceFileSystemVote {
        let the_state_carries_the_fault = match fault_outcome {
            FaultOutcome::NeverFired => false,
            FaultOutcome::SurfacedAsAnError
            | FaultOutcome::ToleratedAndTheHistoryFinished
            | FaultOutcome::StoppedOnSomethingOtherThanTheInjectedError => true,
        };
        if the_state_carries_the_fault
            && the_state_after_the_fault_is_outside_the_reference_filesystem_range(fault)
        {
            return ReferenceFileSystemVote::Abstained(
                ReferenceFileSystemAbstention::TheStateCarriesAnInjectedFault,
            );
        }
        match what_the_reopen_must_recover {
            WhatTheReopenMustRecover::NothingBecauseNoVersionStands => {
                ReferenceFileSystemVote::Abstained(
                    ReferenceFileSystemAbstention::NoVersionStandsAfterTheInjectedFault,
                )
            }
            WhatTheReopenMustRecover::AVersionTheHistoryLeft => {
                self.judge_a_crash(
                    ReferenceFileSystemCrashMoment::AfterEveryTranslatedStep,
                    read_back,
                )
                .vote
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelCheckpointTxg, ModelInstanceGeneration};

    fn root_with_txg(checkpoint_txg: u64) -> ModelRootKey {
        ModelRootKey {
            checkpoint_txg: ModelCheckpointTxg(checkpoint_txg),
            instance: ModelInstanceGeneration(1),
        }
    }

    /// mkfs 写第 1 版的根（txg 0），之后一次发布：会话先推一次抬 F（txg 1，映到第 1 版），再写发布那条根（txg 2，写出第 2 版）。
    /// 照 `translate` 给每条根映版本、记最新那条与写出最新那一版的那条；交回这一步之后的两样与 m 表。
    fn roots_after_the_make_filesystem_and_a_publish(
    ) -> (RootsBeforeTheStep, BTreeMap<ModelRootKey, VersionNumber>) {
        let mut versions_of_the_roots = BTreeMap::new();
        versions_of_the_roots.insert(root_with_txg(0), VersionNumber(1));
        let roots = RootsBeforeTheStep::before_any_step().after_the_step(
            &[root_with_txg(0)],
            NewVersionWrittenBy::EveryRootOfTheStep,
            VersionNumber(1),
            VersionNumber(1),
        );
        versions_of_the_roots.insert(root_with_txg(1), VersionNumber(1));
        versions_of_the_roots.insert(root_with_txg(2), VersionNumber(2));
        let roots = roots.after_the_step(
            &[root_with_txg(1), root_with_txg(2)],
            NewVersionWrittenBy::TheLastRootOfTheStep,
            VersionNumber(1),
            VersionNumber(2),
        );
        (roots, versions_of_the_roots)
    }

    /// 判决 `research/prompts/m4-segment-code-r3-main-verification.md` 第三节 J1：被抛弃的是发布写出第 2 版的那条根，交「写出最新那一版的那一条」。
    #[test]
    fn abandoning_the_root_a_publish_wrote_its_version_with_is_handed_as_the_root_that_wrote_the_newest_version(
    ) {
        let (roots, versions_of_the_roots) = roots_after_the_make_filesystem_and_a_publish();
        assert_eq!(
            roots.relation_of_the_abandoned_root_to_the_newest_version(
                &versions_of_the_roots,
                VersionNumber(2)
            ),
            Ok(AbandonedRoot::WroteTheNewestVersion),
            "最新那条根是发布写出第 2 版的那一条"
        );
    }

    /// 发布之后可写挂载一次，写行与暖机两条根（txg 3、4）都映到第 2 版、没出新版本：被抛弃的是暖机那条，交「映到最新那一版、不是写出它的那一条」
    /// （攻方第三轮 Q2 H5 那一格的第一格的形状，`research/prompts/m4-segment-code-r3-opus-output.md`）。
    #[test]
    fn abandoning_a_root_a_writable_mount_wrote_after_a_publish_is_handed_as_written_after_the_newest_version(
    ) {
        let (roots, mut versions_of_the_roots) = roots_after_the_make_filesystem_and_a_publish();
        versions_of_the_roots.insert(root_with_txg(3), VersionNumber(2));
        versions_of_the_roots.insert(root_with_txg(4), VersionNumber(2));
        let roots = roots.after_the_step(
            &[root_with_txg(3), root_with_txg(4)],
            NewVersionWrittenBy::EveryRootOfTheStep,
            VersionNumber(2),
            VersionNumber(2),
        );
        assert_eq!(
            roots.relation_of_the_abandoned_root_to_the_newest_version(
                &versions_of_the_roots,
                VersionNumber(2)
            ),
            Ok(AbandonedRoot::WrittenAfterTheNewestVersion),
            "最新那条根是暖机那一条，映到第 2 版、不是写出它的那一条"
        );
    }

    /// 发布之后冷启动恢复一次（只读不写，读出的那一份是第 3 版），接着抛弃最新根：被抛弃的是发布写出第 2 版的那条，不映到 RefFS 最新那一版，
    /// RefFS 不记第 3 版之前的快照集合——翻译停下、交「被抛弃的根不在 RefFS 最新那一版上」，点名那条根。
    #[test]
    fn abandoning_a_root_of_a_version_before_a_cold_start_recovery_that_wrote_no_root_stops_the_translation(
    ) {
        let (roots, versions_of_the_roots) = roots_after_the_make_filesystem_and_a_publish();
        let roots = roots.after_the_step(
            &[],
            NewVersionWrittenBy::EveryRootOfTheStep,
            VersionNumber(2),
            VersionNumber(3),
        );
        assert_eq!(
            roots.relation_of_the_abandoned_root_to_the_newest_version(
                &versions_of_the_roots,
                VersionNumber(3)
            ),
            Err(
                TranslationStopReason::TheAbandonedRootIsNotARootOfTheNewestVersion {
                    abandoned: Some(root_with_txg(2)),
                }
            ),
            "被抛弃的是第 2 版的根，RefFS 最新那一版是冷启动读出的第 3 版"
        );
    }

    /// 截断那一步的观察：第 `step_index` 步，结局 `ending`，执行器交给会话的目标 size 是 `new_size`。
    fn truncation_step_seen(
        ending: StepEndingSeenByTheTranslation,
        new_size: u64,
    ) -> StepSeenByTheTranslation {
        StepSeenByTheTranslation {
            position: StepPosition::Operation(0),
            operation: Some(HistoryOperation::Truncate(
                crate::history::TruncationChoice {
                    target: crate::history::TruncationTarget::IntoTheMiddleOfADataUnit,
                    selector: 0,
                },
            )),
            ending,
            stream_length_after_the_step: 0,
            size_requested_by_the_truncation: Some(new_size),
        }
    }

    fn events_of_a_truncation_step(seen: &StepSeenByTheTranslation) -> EventsOfAStep {
        let translation = ReferenceFileSystemTranslation::for_a_history_starting(
            HistoryStartingPoint::AfterFirstFile,
        );
        translation
            .events_of(
                seen,
                &BTreeMap::new(),
                RootsBeforeTheStep::before_any_step(),
                VersionNumber(1),
            )
            .unwrap_or_else(|reason| panic!("截断那一步翻得出：{reason:?}"))
    }

    /// 截断做成（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8「截断发成（不论几个版本）交一个 `Truncate(n)`」）：会话推了一串抬 F、
    /// 再写意图、一批、最后一批（三条根带着改动），翻成一个 `Truncate(100)` 连同它那次发布、都是成；这一步最后三条根映到新版本、写出它的是意图那一条
    /// （K2「先落目标」：意图那一版起读出的就是截后的）。
    #[test]
    fn a_truncation_published_in_several_versions_is_handed_as_one_truncate_and_every_root_from_the_intent_on_maps_to_the_new_version(
    ) {
        let seen = truncation_step_seen(
            StepEndingSeenByTheTranslation::ReturnedOk {
                effect: Some(AppliedEffect::Published {
                    reuse: crate::history::RecordReuse::default(),
                    floor_raises_pushed_by_the_session: 1,
                    roots_carrying_the_change: 3,
                }),
                read_out_of_the_recovery_in_the_step: None,
            },
            100,
        );
        let events = events_of_a_truncation_step(&seen);
        assert_eq!(
            events.events,
            vec![
                ReferenceFileSystemEvent::Submit {
                    operation: Operation::Truncate(100),
                    outcome: OperationOutcome::Succeeded,
                },
                ReferenceFileSystemEvent::Submit {
                    operation: Operation::ExplicitPublish,
                    outcome: OperationOutcome::Succeeded,
                },
            ]
        );
        assert_eq!(
            events.new_version_is_written_by,
            NewVersionWrittenBy::EveryRootFromTheChangeOn {
                roots_of_the_change: 3
            }
        );
        let roots = RootsBeforeTheStep::before_any_step().after_the_step(
            &[
                root_with_txg(1),
                root_with_txg(2),
                root_with_txg(3),
                root_with_txg(4),
            ],
            events.new_version_is_written_by,
            VersionNumber(1),
            VersionNumber(2),
        );
        assert_eq!(
            roots.root_that_wrote_the_newest_version,
            Some(root_with_txg(2)),
            "写出截后那一版的是意图那一条（抬 F 那一条之后）"
        );
    }

    /// 截断在任何写之前被拒（第一个文件之外的 inode、有树而文件集合为空）：交空，RefFS 的序列里没有它。
    #[test]
    fn a_truncation_refused_before_any_write_hands_nothing_to_the_reference_filesystem() {
        let seen = truncation_step_seen(StepEndingSeenByTheTranslation::RefusedBeforeAnyWrite, 7);
        assert!(events_of_a_truncation_step(&seen).events.is_empty());
    }

    /// 截断写了之后报错（故障注入那一步）：交一个 `Truncate(n)`，它那次发布交「写了之后报错」（两可，同覆盖写）。
    #[test]
    fn a_truncation_failing_after_writing_hands_the_truncate_and_a_publish_that_failed_after_writing(
    ) {
        let seen = truncation_step_seen(
            StepEndingSeenByTheTranslation::ReturnedAnErrorAfterItMayHaveWritten,
            100,
        );
        assert_eq!(
            events_of_a_truncation_step(&seen).events,
            vec![
                ReferenceFileSystemEvent::Submit {
                    operation: Operation::Truncate(100),
                    outcome: OperationOutcome::Succeeded,
                },
                ReferenceFileSystemEvent::Submit {
                    operation: Operation::ExplicitPublish,
                    outcome: OperationOutcome::FailedAfterWriting,
                },
            ]
        );
    }
}

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
//! 抛弃最新根的那次挂载 = 一次带读故障的恢复（不判，以读出的内容重新起头，r6 判决第二节 U2 一行）。
//! 原样重发这一种结局今天的历史走不出来（入口报错历史就停），这里不产出它。

use std::collections::BTreeMap;

use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_reference_filesystem::{
    AllowedState, CrashNotAccepted, FileContent, Operation, OperationNotAccepted, OperationNumber,
    OperationOutcome, ReferenceFileSystem, SubmittedOperation, VersionNumber, VisibleState,
};

use crate::fault_injection::{FaultOutcome, InjectedFault};
use crate::history::{
    AppliedEffect, HarnessJudgement, HistoryEnding, HistoryOperation, HistoryRun,
    HistoryStartingPoint, StepObservation, StepOutcome, StepPosition,
};
use crate::memory_pool::root_identity_written_by;
use crate::model::{ModelDisagreement, ModelRootKey, ObservedReadBack};
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
                    RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier => {
                        return None
                    }
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
        for (step_index, seen) in self.steps.iter().enumerate() {
            assert!(
                seen.ending != StepEndingSeenByTheTranslation::NeverReturned
                    || step_index + 1 == self.steps.len(),
                "没返回的那一步只由 note_the_step_the_history_stopped_in 补在历史停下之后，是最后一步：{:?}",
                seen.position
            );
            let version_before_the_step = model.newest_version();
            let events = match self.events_of(seen, &versions_of_the_roots) {
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
                    NewVersionWrittenBy::EveryRootOfTheStep => version_after_the_step,
                };
                versions_of_the_roots.insert(*root, version);
            }
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
    fn events_of(
        &self,
        seen: &StepSeenByTheTranslation,
        versions_of_the_roots: &BTreeMap<ModelRootKey, VersionNumber>,
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
                            | AppliedEffect::Recovered { .. },
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
            HistoryOperation::CloseAndMountWritable
            | HistoryOperation::RaiseRollbackFloor(_)
            | HistoryOperation::CreateInode
            | HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(_)
            | HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                EventsOfAStep::nothing()
            }
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
                        ReferenceFileSystemEvent::RestartAfterARecoveryThatAbandonedTheNewestRoot(
                            read_out.clone(),
                        ),
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NewVersionWrittenBy {
    TheLastRootOfTheStep,
    EveryRootOfTheStep,
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
    /// 抛弃最新根的那次恢复：不判，以读出的内容重新起头。
    RestartAfterARecoveryThatAbandonedTheNewestRoot(VisibleState),
}

/// 事件交进 RefFS 之后：进了序列的操作号，或者没有。
enum EventApplied {
    Entered(OperationNumber),
    NothingEntered,
}

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
        ReferenceFileSystemEvent::RestartAfterARecoveryThatAbandonedTheNewestRoot(read_out) => {
            let crash = model.crash_after_every_operation_returned();
            match model
                .restart_after_a_recovery_that_abandoned_the_newest_root(crash, read_out.clone())
            {
                Ok(()) => Ok(EventApplied::NothingEntered),
                Err(refusal) => Err(stopped_verdict_of_a_refused_recovery(refusal)),
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
}

impl TranslationStopReason {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
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
            TranslationStopReason::TheReferenceFileSystemHasNoUnlinkInTheFirstVersion => {
                "RefFS 第一版没有删除"
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

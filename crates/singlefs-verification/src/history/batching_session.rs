//! 攒批会话的随机历史：一串调用、按会话报告的发布点与模型对拍，根环的洞表。

use super::*;

/// 攒批会话随机历史里的一次调用（D16（发布语义） 已定项 2、10；判决 `research/prompts/m4-publish-trigger-r1-main-verification.md` 第二节「返回即落盘」
/// 那一格：随机历史的「返回即落盘」改成按会话报告的发布点判）。每一次调用之前调用方的钟往前走 `clock_advance_in_milliseconds`。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchingSessionCall {
    /// `accept` 覆盖写第一个文件（一个数据单元以内）。
    AcceptOverwriteOfTheFirstFile {
        length: usize,
        clock_advance_in_milliseconds: u64,
    },
    /// `accept` 顺序写第一个文件（可以跨几个数据单元）。
    AcceptSequentialWriteOfTheFirstFile {
        length: usize,
        clock_advance_in_milliseconds: u64,
    },
    /// `accept` 建 `count` 个空 inode。
    AcceptNewInodes {
        count: u64,
        clock_advance_in_milliseconds: u64,
    },
    /// `publish_when_due`。
    PublishWhenDue { clock_advance_in_milliseconds: u64 },
    /// `publish_pending_now`。
    PublishPendingNow { clock_advance_in_milliseconds: u64 },
    /// 放锁的后台入口（`run_compaction_in_the_background_for_at_most_one_publish`）一次：`foreground_fsync_is_waiting` 是这一次调用方答的前台 fsync。
    /// 起跑了就先排空待发，之后至多一次整理发布；三样交错都走得到：排空后接整理、整理发布推「上次发布的读数」、让路之后前台再发再接着推。
    RunCompactionInTheBackground {
        foreground_fsync_is_waiting: bool,
        clock_advance_in_milliseconds: u64,
    },
}

impl BatchingSessionCall {
    fn clock_advance_in_milliseconds(self) -> u64 {
        match self {
            BatchingSessionCall::AcceptOverwriteOfTheFirstFile {
                clock_advance_in_milliseconds,
                ..
            }
            | BatchingSessionCall::AcceptSequentialWriteOfTheFirstFile {
                clock_advance_in_milliseconds,
                ..
            }
            | BatchingSessionCall::AcceptNewInodes {
                clock_advance_in_milliseconds,
                ..
            }
            | BatchingSessionCall::PublishWhenDue {
                clock_advance_in_milliseconds,
            }
            | BatchingSessionCall::PublishPendingNow {
                clock_advance_in_milliseconds,
            }
            | BatchingSessionCall::RunCompactionInTheBackground {
                clock_advance_in_milliseconds,
                ..
            } => clock_advance_in_milliseconds,
        }
    }
}

/// 一段攒批会话随机历史：`seed` 定的 `calls` 次调用。钟每次往前走 0 到 `largest_clock_advance_in_milliseconds` 毫秒（含两端），内容长度在两个数据单元的
/// 净荷以内，建 inode 一次 1 到 3 个。
#[must_use]
pub fn random_batching_session_calls(
    seed: u64,
    calls: usize,
    largest_clock_advance_in_milliseconds: u64,
) -> Vec<BatchingSessionCall> {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let payload = usize::try_from(crate::model::data_unit_payload_capacity_in_bytes())
        .expect("一个数据单元的净荷装得进 usize");
    // 迭代上界是 `calls`；跨轮携带的只有随机数状态。
    (0..calls)
        .map(|_call_index| {
            let clock_advance_in_milliseconds =
                next() % (largest_clock_advance_in_milliseconds + 1);
            match next() % 6 {
                0 => BatchingSessionCall::AcceptOverwriteOfTheFirstFile {
                    length: usize::try_from(next() % u64::try_from(payload).expect("净荷"))
                        .expect("小于净荷")
                        + 1,
                    clock_advance_in_milliseconds,
                },
                1 => BatchingSessionCall::AcceptSequentialWriteOfTheFirstFile {
                    length: usize::try_from(next() % u64::try_from(2 * payload).expect("两份净荷"))
                        .expect("小于两份净荷")
                        + 1,
                    clock_advance_in_milliseconds,
                },
                2 => BatchingSessionCall::AcceptNewInodes {
                    count: next() % 3 + 1,
                    clock_advance_in_milliseconds,
                },
                3 => BatchingSessionCall::PublishWhenDue {
                    clock_advance_in_milliseconds,
                },
                4 => BatchingSessionCall::RunCompactionInTheBackground {
                    foreground_fsync_is_waiting: next() % 4 == 0,
                    clock_advance_in_milliseconds,
                },
                _ => BatchingSessionCall::PublishPendingNow {
                    clock_advance_in_milliseconds,
                },
            }
        })
        .collect()
}

/// 攒批会话随机历史里写的内容：长度与第几次调用定，逐字节确定。
#[must_use]
pub fn content_of_a_batching_session_call(length: usize, call_index: usize) -> Vec<u8> {
    let seed = u64::try_from(call_index).expect("调用序号装得进 u64");
    (0..length)
        .map(|position| {
            let position = u64::try_from(position).expect("长度装得进 u64");
            u8::try_from((position * 167 + seed * 31 + 7) % 253).expect("小于 256")
        })
        .collect()
}

/// 一次调用之后看到的：调用本身、这一刻的读数、收下的那一次是不是第一个文件的整份写（RefFS 翻译成一个内容改动）、会话报了几次发布（每一次翻成一个显式发布点）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchingSessionCallSeen {
    pub call: BatchingSessionCall,
    pub reading_in_milliseconds: u64,
    pub first_file_content_accepted: Option<Vec<u8>>,
    pub publishes_reported: usize,
}

/// 一段攒批会话历史跑完：每一次调用看到的、与模型比过的发布次数、按触发的哪一支数的发布次数。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchingSessionHistoryRun {
    pub calls_seen: Vec<BatchingSessionCallSeen>,
    pub publishes_compared_with_the_model: u64,
    pub publishes_triggered_by_time: u64,
    pub publishes_triggered_by_the_dirty_journal_bytes: u64,
    pub publishes_triggered_by_both: u64,
    pub publishes_on_request: u64,
    /// 后台入口发成的整理发布次数（`BatchingSessionCall::RunCompactionInTheBackground`）。
    pub compaction_publishes_in_the_background: u64,
}

/// [`run_batching_session_calls`] 取这一刻盘上镜像的那个函数：读带着身份的各块盘，交出整池的内存镜像。
/// `'captured` 是闭包借着的东西活多久（不写的话类型别名里的 `dyn` 默认要 `'static`，借着调用方局部量的闭包传不进来）。
pub type ImageOfTheDevicesTaker<'captured, Device> =
    dyn Fn(&[(DeviceIdentity, Device)]) -> MemoryPool + 'captured;

/// 攒批会话随机历史里后台入口的节奏：定时兜底间隔 1 毫秒（钟走了就查），写带宽不限。
const PACING_OF_THE_BACKGROUND_CALLS_OF_THE_BATCHING_HISTORY:
    singlefs_core::compaction_plan::CompactionPacing =
    singlefs_core::compaction_plan::CompactionPacing {
        timer_fallback: singlefs_core::compaction_plan::CompactionTimerFallback::EveryMilliseconds(
            std::num::NonZeroU64::MIN,
        ),
        write_bandwidth_limit:
            singlefs_core::compaction_plan::CompactionWriteBandwidthLimit::Unlimited,
    };

/// 攒批会话随机历史里后台入口问的两样：钟就是这一次调用的读数，前台 fsync 是生成的那一样。
struct CallerOfTheBatchingHistory {
    now: singlefs_core::compaction_plan::CallerMonotonicClockReading,
    foreground_fsync_is_waiting: bool,
}

impl singlefs_core::compaction_plan::CallerOfTheBackgroundCompaction
    for CallerOfTheBatchingHistory
{
    fn clock_reading(&self) -> singlefs_core::compaction_plan::CallerMonotonicClockReading {
        self.now
    }

    fn answer_before_the_next_batch(
        &self,
    ) -> singlefs_core::compaction_plan::AnswerOfTheCallerBeforeTheNextBatch {
        if self.foreground_fsync_is_waiting {
            singlefs_core::compaction_plan::AnswerOfTheCallerBeforeTheNextBatch::ForegroundFsyncIsWaiting
        } else {
            singlefs_core::compaction_plan::AnswerOfTheCallerBeforeTheNextBatch::NoForegroundFsyncWaiting
        }
    }
}

/// 实现报的触发那一支换成模型的写法（两边各自声明）。
fn model_cause_of(
    cause: singlefs_core::mounted_session::PublishTriggerCause,
) -> crate::model::ModelPublishTriggerCause {
    match cause {
        singlefs_core::mounted_session::PublishTriggerCause::TimeThreshold => {
            crate::model::ModelPublishTriggerCause::TimeThreshold
        }
        singlefs_core::mounted_session::PublishTriggerCause::DirtyThreshold => {
            crate::model::ModelPublishTriggerCause::DirtyThreshold
        }
        singlefs_core::mounted_session::PublishTriggerCause::BothThresholds => {
            crate::model::ModelPublishTriggerCause::BothThresholds
        }
    }
}

/// 在攒批的会话上逐次跑 `calls`，每一次调用之后判三样：会话报的发布与模型判的触发一致（该发的发了、哪一支先到对得上、不该发的没发、待发空时不推空发布）；
/// 报了发布就拿那一版与模型比（[`crate::model_comparison::compare_a_publish_of_the_pending_changes_with_the_model`]：模型对拍只放在会话报告的发布之后）；
/// 这一刻的盘过池级 checker。`image_of_the_devices` 取这一刻盘上的镜像（恢复读第一个文件、池级 checker 都读它）。
///
/// # Errors
/// 第一处对不上：哪一次调用、哪一样、两边各是什么。
// gpu-exempt: comparison-with-the-ideal-model
pub fn run_batching_session_calls<Device: singlefs_core::block_device::BlockDevice>(
    session: &mut singlefs_core::mounted_session::MountedSessionWithPendingChanges,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    image_of_the_devices: &ImageOfTheDevicesTaker<'_, Device>,
    calls: &[BatchingSessionCall],
    model: &mut crate::model::ModelOfThePendingChanges,
    write_time_seconds_of_the_first_call: u64,
) -> Result<BatchingSessionHistoryRun, String> {
    use singlefs_core::compaction_plan::CallerMonotonicClockReading;
    use singlefs_core::mounted_session::{HowTheChangeWasTaken, ReasonOfThePublish, UserChange};
    let mut run = BatchingSessionHistoryRun {
        calls_seen: Vec::new(),
        publishes_compared_with_the_model: 0,
        publishes_triggered_by_time: 0,
        publishes_triggered_by_the_dirty_journal_bytes: 0,
        publishes_triggered_by_both: 0,
        publishes_on_request: 0,
        compaction_publishes_in_the_background: 0,
    };
    let mut now_in_milliseconds = 0_u64;
    // 迭代上界是 `calls` 的条数；跨轮携带的是钟（只增）、模型与会话（各自只经自己的入口改），每一次调用都可能提前返回错误。
    for (call_index, call) in calls.iter().copied().enumerate() {
        now_in_milliseconds += call.clock_advance_in_milliseconds();
        let now =
            CallerMonotonicClockReading(std::time::Duration::from_millis(now_in_milliseconds));
        model.note_a_reading(now_in_milliseconds);
        let write_time_seconds = write_time_seconds_of_the_first_call
            + u64::try_from(call_index).expect("调用序号装得进 u64");
        let mut first_file_content_accepted = None;
        let mut is_a_compaction_published_by_this_call = false;
        let reported = match call {
            BatchingSessionCall::AcceptOverwriteOfTheFirstFile { length, .. }
            | BatchingSessionCall::AcceptSequentialWriteOfTheFirstFile { length, .. } => {
                let content = content_of_a_batching_session_call(length, call_index);
                let file = FirstFile {
                    content: &content,
                    write_time_seconds,
                };
                let change = match call {
                    BatchingSessionCall::AcceptOverwriteOfTheFirstFile { .. } => {
                        UserChange::Overwrite(file)
                    }
                    BatchingSessionCall::AcceptSequentialWriteOfTheFirstFile { .. }
                    | BatchingSessionCall::AcceptNewInodes { .. }
                    | BatchingSessionCall::PublishWhenDue { .. }
                    | BatchingSessionCall::PublishPendingNow { .. }
                    | BatchingSessionCall::RunCompactionInTheBackground { .. } => {
                        UserChange::SequentialWrite(file)
                    }
                };
                let accepted = session.accept(devices, change, now).map_err(|refusal| {
                    format!("第 {call_index} 次调用 {call:?}：accept 被拒 {refusal:?}")
                })?;
                let HowTheChangeWasTaken::KeptPending {
                    journal_bytes_of_this_change,
                } = accepted.taken
                else {
                    return Err(format!(
                        "第 {call_index} 次调用 {call:?}：整份写被当成屏障单发"
                    ));
                };
                model
                    .accept(
                        crate::model::ModelPendingChange::WholeWrite {
                            inode: singlefs_core::transaction::FIRST_INODE_NUMBER,
                            content: content.clone(),
                        },
                        journal_bytes_of_this_change.0,
                    )
                    .map_err(|below| format!("第 {call_index} 次调用 {call:?}：{below:?}"))?;
                first_file_content_accepted = Some(content);
                accepted.publishes
            }
            BatchingSessionCall::AcceptNewInodes { count, .. } => {
                let accepted = session
                    .accept(
                        devices,
                        UserChange::NewInodes {
                            count,
                            write_time_seconds,
                        },
                        now,
                    )
                    .map_err(|refusal| {
                        format!("第 {call_index} 次调用 {call:?}：accept 被拒 {refusal:?}")
                    })?;
                let HowTheChangeWasTaken::KeptPending {
                    journal_bytes_of_this_change,
                } = accepted.taken
                else {
                    return Err(format!(
                        "第 {call_index} 次调用 {call:?}：建 inode 被当成屏障单发"
                    ));
                };
                model
                    .accept(
                        crate::model::ModelPendingChange::NewInodes { count },
                        journal_bytes_of_this_change.0,
                    )
                    .map_err(|below| format!("第 {call_index} 次调用 {call:?}：{below:?}"))?;
                accepted.publishes
            }
            BatchingSessionCall::PublishWhenDue { .. } => session
                .publish_when_due(devices, now)
                .map_err(|refused| format!("第 {call_index} 次调用 {call:?}：{refused:?}"))?
                .into_iter()
                .collect(),
            BatchingSessionCall::PublishPendingNow { .. } => session
                .publish_pending_now(devices, now)
                .map_err(|refused| format!("第 {call_index} 次调用 {call:?}：{refused:?}"))?
                .into_iter()
                .collect(),
            BatchingSessionCall::RunCompactionInTheBackground {
                foreground_fsync_is_waiting,
                ..
            } => {
                let after_the_call = session
                    .run_compaction_in_the_background_for_at_most_one_publish(
                        devices,
                        PACING_OF_THE_BACKGROUND_CALLS_OF_THE_BATCHING_HISTORY,
                        &CallerOfTheBatchingHistory {
                            now,
                            foreground_fsync_is_waiting,
                        },
                    )
                    .map_err(|refused| format!("第 {call_index} 次调用 {call:?}：{refused:?}"))?;
                let compaction_publishes = match &after_the_call.call {
                    singlefs_core::mounted_session::BackgroundCompactionCall::NotStarted {
                        ..
                    } => 0,
                    singlefs_core::mounted_session::BackgroundCompactionCall::Ran {
                        run: compaction_run,
                        ..
                    } => compaction_run.versions_published.len(),
                };
                run.compaction_publishes_in_the_background += u64::try_from(compaction_publishes)
                    .expect("一次调用至多一次整理发布外加走空队列");
                is_a_compaction_published_by_this_call = compaction_publishes > 0;
                after_the_call
                    .pending_changes_published_before_the_run
                    .into_iter()
                    .collect()
            }
        };
        let expected_publish = match call {
            BatchingSessionCall::PublishPendingNow { .. } => {
                (!model.pending_is_empty()).then_some(None)
            }
            // 起不起跑看水位与备忘，模型不判：起跑了就排空，模型只判待发空时不许报排空。
            BatchingSessionCall::RunCompactionInTheBackground { .. } => {
                (!model.pending_is_empty() && !reported.is_empty()).then_some(None)
            }
            BatchingSessionCall::AcceptOverwriteOfTheFirstFile { .. }
            | BatchingSessionCall::AcceptSequentialWriteOfTheFirstFile { .. }
            | BatchingSessionCall::AcceptNewInodes { .. }
            | BatchingSessionCall::PublishWhenDue { .. } => {
                model.expected_trigger(now_in_milliseconds).map(Some)
            }
        };
        let reported_cause = match reported.as_slice() {
            [] => None,
            [one] => Some(match one.reason {
                ReasonOfThePublish::TriggeredAfterTheAccept(cause)
                | ReasonOfThePublish::TriggeredWhenDue(cause) => Some(model_cause_of(cause)),
                ReasonOfThePublish::PendingChangesPublishedOnRequest
                | ReasonOfThePublish::PendingChangesPublishedBeforeTheBackgroundCompaction => None,
                other @ (ReasonOfThePublish::PendingChangesPublishedBeforeABarrier
                | ReasonOfThePublish::BarrierPublishedAlone
                | ReasonOfThePublish::PendingChangesPublishedBeforeTheUserChange
                | ReasonOfThePublish::UserChangePublishedAlone
                | ReasonOfThePublish::PendingChangesPublishedBeforeHandingBackTheSession) => {
                    return Err(format!(
                        "第 {call_index} 次调用 {call:?}：会话报的发布理由 {other:?} 不是这几种入口报得出的"
                    ))
                }
            }),
            several => {
                return Err(format!(
                    "第 {call_index} 次调用 {call:?}：一次调用报了 {} 次发布（这几种入口至多排空一次）",
                    several.len()
                ))
            }
        };
        if reported_cause != expected_publish {
            return Err(format!(
                "第 {call_index} 次调用 {call:?}（读数 {now_in_milliseconds} ms，待发 journal 字节 {}）：模型判 {expected_publish:?}，会话报 {reported_cause:?}",
                model.journal_bytes_of_the_pending_changes()
            ));
        }
        let image = image_of_the_devices(devices);
        if let Some(cause) = reported_cause {
            model.publish_the_pending_changes(now_in_milliseconds);
            let version = session
                .session()
                .current
                .version_of_trees()
                .ok_or_else(|| format!("第 {call_index} 次调用：发布之后现行那一版没有树"))?;
            let first_file_read_back = match recover(&image, JournalPolicy::Consult).outcome {
                RecoveryOutcome::FileRead { content, .. } => Some(content),
                RecoveryOutcome::NoFile { .. } | RecoveryOutcome::Failed { .. } => None,
            };
            crate::model_comparison::compare_a_publish_of_the_pending_changes_with_the_model(
                version,
                first_file_read_back.as_deref(),
                model,
            )
            .map_err(|disagreement| format!("第 {call_index} 次调用 {call:?}：{disagreement}"))?;
            run.publishes_compared_with_the_model += 1;
            match cause {
                Some(crate::model::ModelPublishTriggerCause::TimeThreshold) => {
                    run.publishes_triggered_by_time += 1;
                }
                Some(crate::model::ModelPublishTriggerCause::DirtyThreshold) => {
                    run.publishes_triggered_by_the_dirty_journal_bytes += 1;
                }
                Some(crate::model::ModelPublishTriggerCause::BothThresholds) => {
                    run.publishes_triggered_by_both += 1;
                }
                None => run.publishes_on_request += 1,
            }
        }
        // 整理发布也把会话的「上次发布的读数」往前推（发布触发的时间那一支从它算）：模型跟着记，待发不动。
        if is_a_compaction_published_by_this_call {
            model.note_a_publish(now_in_milliseconds);
        }
        let violated: Vec<String> = singlefs_checker::walk::check_pool_image(&image)
            .into_iter()
            .filter_map(|(name, verdict)| match verdict {
                singlefs_checker::image::InvariantVerdict::Violated(detail) => {
                    Some(format!("{name}：{detail}"))
                }
                singlefs_checker::image::InvariantVerdict::Holds
                | singlefs_checker::image::InvariantVerdict::NotApplicable(_)
                | singlefs_checker::image::InvariantVerdict::PartiallyJudged(_) => None,
            })
            .collect();
        if !violated.is_empty() {
            return Err(format!(
                "第 {call_index} 次调用 {call:?} 之后池级 checker 红：{violated:?}"
            ));
        }
        run.calls_seen.push(BatchingSessionCallSeen {
            call,
            reading_in_milliseconds: now_in_milliseconds,
            first_file_content_accepted,
            publishes_reported: reported.len(),
        });
    }
    Ok(run)
}

/// 洞表里根环的一个位置：区域、区域里第几槽。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RootRingPositionOfTheHistory {
    pub region: u64,
    pub slot_in_region: u64,
}

/// 洞表里一个位置上历史留下了什么。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootRingPositionContent {
    /// 历史从没往这个位置写过根（mkfs 只种每个区域的槽 0）：一个洞，读槽层不许读出一条根。
    NeverWritten,
    /// 最后写进这个位置的那一条根。
    LastRootWritten { instance: u32, checkpoint_txg: u64 },
}

/// C700（三条判法各读各的根环没有跨装置检查）的洞表：历史知道每个位置实际写过什么，池级 checker 的 I-3.5、I-3.1、I-7.9 读的那一份根环
/// （`singlefs_checker::walk::root_ring_the_ring_judgements_read`）逐位置与它比——读槽层读松、读严时三条判法一起偏，单看判定看不出。
/// 只记写进去的根，不记写坏的（一次写撕了、盘读错了的位置这里不分，崩溃与读故障的流不拿这张表比）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootRingHoleTable {
    pub content_of_each_position: BTreeMap<RootRingPositionOfTheHistory, RootRingPositionContent>,
}

/// 按历史里依次写进根环的根（实例代号、checkpoint_txg，mkfs 种的第 0 代根也算）造洞表：第 n 次发布写区域 `n mod R` 的槽 `(n div R) mod S`，
/// n 取 checkpoint_txg（D22（单元原子性怎么合成） 已定项 2 / 已定项 16）；同一个位置写过几次的留最后那一次。式子在这里另写一份，不调实现的
/// `root_ring::target_for_publish`、也不调池级 checker 的读法（D13（验证路线） 已定项 5：几套装置各算各的才对得出读槽层偏没偏）。
///
/// # Panics
/// `regions` 或 `slots_per_region` 是 0（根环没有槽，调用方给错了几何）。
#[must_use]
pub fn root_ring_hole_table_of_the_roots_written(
    roots_written_in_order: &[(u32, u64)],
    regions: u64,
    slots_per_region: u64,
) -> RootRingHoleTable {
    assert!(
        regions > 0 && slots_per_region > 0,
        "根环至少一个区域、每区至少一槽：调用方给的几何 R = {regions}、S = {slots_per_region}"
    );
    let mut content_of_each_position: BTreeMap<
        RootRingPositionOfTheHistory,
        RootRingPositionContent,
    > = (0..regions)
        .flat_map(|region| {
            (0..slots_per_region).map(move |slot_in_region| {
                (
                    RootRingPositionOfTheHistory {
                        region,
                        slot_in_region,
                    },
                    RootRingPositionContent::NeverWritten,
                )
            })
        })
        .collect();
    for (instance, checkpoint_txg) in roots_written_in_order {
        content_of_each_position.insert(
            RootRingPositionOfTheHistory {
                region: checkpoint_txg % regions,
                slot_in_region: (checkpoint_txg / regions) % slots_per_region,
            },
            RootRingPositionContent::LastRootWritten {
                instance: *instance,
                checkpoint_txg: *checkpoint_txg,
            },
        );
    }
    RootRingHoleTable {
        content_of_each_position,
    }
}

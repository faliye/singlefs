//! 说谎的设备留下的：被吞的写能解释哪些失败、注入之后每一步的镜像与录制流、把被吞的写插回去的录制流，
//! 按池级 checker 的结论把注入之后的失败分成说谎的设备留下的与新发现。

use super::*;

/// 说谎的设备吞掉了几次写之后，拿来判「这条失败是不是它留下的」的这几样东西（按注入点认，实七）：
///
/// - `invariants_still_violated_if_the_swallowed_writes_had_landed`：录制流里把被吞的那几次写按原位插回去、重建镜像
///   （实现以为自己写下的那一份），在它上面 checker 判红的那几条。违例点名的（盘, 槽）是被吞那次写的、或红在被吞的根槽里留下的旧根上，
///   那几次写一落盘它就消失；插回去仍红的那一条不是注入点造成的。
/// - `ceiling_without_the_swallowed_roots`：被吞的写里有根槽写时，拿历史停下那一步之前的模型、去掉那几条根重算的抬 F 的上限两个数
///   （实现从盘上现读根环，看不见那几条根；它们留下的、这个会话挂载那一刻读回全零的槽，实现并进上限，D16（发布语义） 已定项 1
///   「抬 F 的上限」那一行「并」：[`crate::model::IdealModel::rollback_floor_ceilings_without_the_roots`]）；
///   没有被吞的根槽写、或没有那一步之前的模型时 None。
/// - `root_ring_slots_of_the_swallowed_root_slot_writes`：被吞的根槽写各落在根环的哪个槽（实八）。同一个进程写过、FUA 返回过的槽，
///   算抬 F 的上限、或挂着时回退判候选集读根环时读坏就重读一次、仍坏就拒（D16（发布语义） 已定项 1「根槽这一次读坏」那一行；
///   回退照同一个读法，`crates/singlefs-core/src/mount.rs` 的 `rollback_candidate`）；读坏的正是被吞那次写的槽，这个拒绝就是条款要的结局，是说谎的设备的投影。
/// - `instance_rows_with_the_swallowed_roots_rebuilt_from_their_records`：被吞的写里有根槽写时，拿历史停下那一步之前的模型答
///   「那几条根的根槽没落、记录与单元落了」时可写挂载写的行（实八，[`crate::model::IdealModel::answer_mount_writable_with_the_roots_rebuilt_from_their_records`]：
///   恢复施加它们的记录、W 取施加的记录的事务号，D23（journal 的角色与格式） 已定项 14 第 4 条）；没有被吞的根槽写、没有那一步之前的模型、
///   或模型答不了时 None。
/// - `byte_ranges_of_the_swallowed_writes`：每一次被吞的写落在哪块盘、哪一段字节（不分写的是什么结构）。管理员回退在任何写之前逐盘验
///   回退目标引用的单元（复活集里的、隔离的）每一份，读不出或整单元 CRC-32C 对不上就拒（D23（journal 的角色与格式） 已定项 14「在任何写之前拒」那一格）；
///   对不上的那一份正落在被吞的写上，这个拒绝就是条款要的结局，是说谎的设备的投影（`research/prompts/code-backlog-b2-code-r1-main-verification.md`
///   「更正」一节 A4：模型不知道那一份没落、要它成）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhatTheSwallowedWritesExplain {
    pub invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet<&'static str>,
    pub ceiling_without_the_swallowed_roots:
        Option<Option<ModelRollbackFloorCeilingsWithoutTheRoots>>,
    pub root_ring_slots_of_the_swallowed_root_slot_writes: BTreeSet<ModelRingPosition>,
    pub instance_rows_with_the_swallowed_roots_rebuilt_from_their_records:
        Option<Vec<ModelInstanceRow>>,
    pub byte_ranges_of_the_swallowed_writes: BTreeSet<SwallowedWriteByteRange>,
}

impl WhatTheSwallowedWritesExplain {
    /// 同一份，池级 checker 那一项（插回去之后仍红的不变量）换成 `invariants_still_violated`：交出去的那一份里那一项是空的，归类那一刻填。
    #[must_use]
    pub fn with_the_invariants_still_violated(
        self,
        invariants_still_violated: BTreeSet<&'static str>,
    ) -> Self {
        Self {
            invariants_still_violated_if_the_swallowed_writes_had_landed: invariants_still_violated,
            ceiling_without_the_swallowed_roots: self.ceiling_without_the_swallowed_roots,
            root_ring_slots_of_the_swallowed_root_slot_writes: self
                .root_ring_slots_of_the_swallowed_root_slot_writes,
            instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: self
                .instance_rows_with_the_swallowed_roots_rebuilt_from_their_records,
            byte_ranges_of_the_swallowed_writes: self.byte_ranges_of_the_swallowed_writes,
        }
    }
}

/// 一次被吞的写落在哪块盘的哪一段字节：[起, 止)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SwallowedWriteByteRange {
    pub device: ModelDeviceIdentity,
    pub start_in_bytes: u64,
    pub end_in_bytes: u64,
}

impl SwallowedWriteByteRange {
    /// 被吞的那一次写落的那一段。
    #[must_use]
    pub fn of(write: &SwallowedWrite) -> Self {
        let length = write.as_retained_operation().operation.length;
        Self {
            device: ModelDeviceIdentity(write.device.0),
            start_in_bytes: write.offset.0,
            end_in_bytes: write
                .offset
                .0
                .checked_add(length)
                .expect("一次写的末端装得进 u64：设备包装先按设备契约查过偏移与长度"),
        }
    }

    /// 这一段与一个单元在同一块盘上的那一份有没有重叠的字节。
    #[must_use]
    pub fn overlaps(self, unit_copy: ModelUnitCopyLocation) -> bool {
        let copy = unit_copy.byte_range_on_the_device();
        self.device == unit_copy.device
            && self.start_in_bytes < copy.end
            && copy.start < self.end_in_bytes
    }
}

/// 这条失败是不是说谎的设备留下的：没有 panic、没有执行器判出的、记录核对器没判红；checker 判红的每一条在「被吞的写都落了盘」的
/// 镜像上都不红；模型对不上的只许是这五格之一：
/// - 抬 F 的上限，而且实现报的上限等于去掉被吞的根、并进它们留下的全零槽之后模型重算的那一个（D16（发布语义） 已定项 1「抬 F 的上限」那一行，
///   读回全零的槽并进上限「并」；判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 4 问）；
/// - 推空（上限卡在挂载时读坏或读回全零的槽上推的不抬 F 的空发布，模型不答），而且实现报的上限等于模型去掉被吞的根、并进它们留下的全零槽之后
///   重算的那一个，并进来确实把上限压低了：卡住上限的是被吞的根槽写留下的全零槽（同一问）；
/// - 「模型说该成、实现拒了」或「拒绝的理由」，而实现拒的是「算抬 F 的上限、或挂着时回退判候选集时一个知道住着根的根环槽读坏、
///   重读仍坏」、拒之前一个写都没发、点名的那个槽正是一次被吞的根槽写的槽（实八：同一个进程里根槽写被吞之后再抬 F 或回退，
///   D16（发布语义） 已定项 1「根槽这一次读坏」那一行要的就是这个拒绝，模型不知道根槽没落、要它成或按上限拒）；
/// - 「模型说该成、实现拒了」或「拒绝的理由」，而实现拒的是管理员回退「回退目标引用的一个单元（复活集里的或隔离的）在一块盘上的那一份
///   读不出或对不上」、拒之前一个写都没发、点名的那一份与一次被吞的写有重叠的字节（A4：设备吞了那一份的写，回退照条款在任何写之前拒）；
/// - 写的实例表行，而且实现写的行等于模型按「被吞根槽的那几条根由记录重建」重算的行（实八：W 取施加的那条记录的事务号，
///   D23（journal 的角色与格式） 已定项 14 第 4 条；模型不知道根槽没落、答 W = 0）。
///
/// 一条不是注入点造成的违例（插回去仍红）、一次读坏的槽不是被吞那次写的、一行与重算的对不上，照报新发现——这是判别力那一半。
///
/// 它替下的是按签名写死两组的白名单（`records/2026-09-21-增补3第4件与checker补三条.md`「签名白名单写死两组、名单外照报」那一句）：
/// 白名单早于 I-3.10 的实现，设变量放大跑的那一批撞出的六组里有四组（I-3.10、I-9.1、旧根复活连带的五条、抬 F 的上限）都是说谎的设备丢一份内容的投影、
/// 却不在名单里（实四丙交回第二节）。
#[must_use]
pub fn left_by_the_lying_device(
    observation: &FailureObservation,
    explained: &WhatTheSwallowedWritesExplain,
) -> bool {
    if observation.panic.is_some()
        || observation.harness_judgement.is_some()
        || !crate::history::record_check_aspects(&observation.record_check).is_empty()
    {
        return false;
    }
    let every_violation_is_explained = observation.violations.iter().all(|(invariant, _)| {
        !explained
            .invariants_still_violated_if_the_swallowed_writes_had_landed
            .contains(invariant)
    });
    let disagreement_is_explained = match &observation.model_disagreement {
        None => true,
        Some(disagreement) => match disagreement.aspect {
            ModelDisagreementAspect::RollbackFloorCeiling => matches!(
                (
                    disagreement.implementation_reported_ceiling,
                    explained.ceiling_without_the_swallowed_roots
                ),
                (Some(reported), Some(Some(recomputed)))
                    if reported == recomputed.merging_the_ring_slots_left_all_zero
            ),
            ModelDisagreementAspect::EmptyPublishKeepingTheFloorWhileTheCeilingIsHeldIsNotModeled => {
                matches!(
                    (
                        disagreement.implementation_reported_ceiling,
                        explained.ceiling_without_the_swallowed_roots
                    ),
                    (Some(reported), Some(Some(recomputed)))
                        if reported == recomputed.merging_the_ring_slots_left_all_zero
                            && recomputed.are_lowered_by_the_ring_slots_left_all_zero()
                )
            }
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess
            | ModelDisagreementAspect::RefusalReason => {
                match disagreement.implementation_reported_unreadable_place {
                    Some(ModelReportedUnreadablePlace::RootRingSlotStillBadAfterOneReread(
                        ring_slot,
                    )) => explained
                        .root_ring_slots_of_the_swallowed_root_slot_writes
                        .contains(&ring_slot),
                    Some(ModelReportedUnreadablePlace::UnitCopyUnreadableOrMismatched(
                        unit_copy,
                    )) => explained
                        .byte_ranges_of_the_swallowed_writes
                        .iter()
                        .any(|swallowed| swallowed.overlaps(unit_copy)),
                    None => false,
                }
            }
            ModelDisagreementAspect::InstanceRows => matches!(
                (
                    &disagreement.implementation_rows_written,
                    &explained.instance_rows_with_the_swallowed_roots_rebuilt_from_their_records
                ),
                (Some(written), Some(recomputed)) if written == recomputed
            ),
            ModelDisagreementAspect::SessionState
            | ModelDisagreementAspect::SucceededWhenModelRequiresRefusal
            | ModelDisagreementAspect::WroteBeforeRefusing
            | ModelDisagreementAspect::PublishCount
            | ModelDisagreementAspect::RootIdentity
            | ModelDisagreementAspect::JournalCounter
            | ModelDisagreementAspect::RollbackFloor
            | ModelDisagreementAspect::FilePresence
            | ModelDisagreementAspect::FileContent
            | ModelDisagreementAspect::InstanceTableOfTheVersion
            | ModelDisagreementAspect::AllocationGeneration
            | ModelDisagreementAspect::InstanceGeneration
            | ModelDisagreementAspect::ColdStartReadBack
            | ModelDisagreementAspect::LiveHeadPreviousSnapshotTxg
            | ModelDisagreementAspect::Snapshots
            | ModelDisagreementAspect::DeadlistEntries
            | ModelDisagreementAspect::PinnedMappingEntries
            | ModelDisagreementAspect::SnapshotReadBack
            | ModelDisagreementAspect::CompactionIntentOfThePublish
            | ModelDisagreementAspect::LoggedCompactionIntent
            | ModelDisagreementAspect::FirstFileRecordTruncationField
            | ModelDisagreementAspect::InodesOtherThanTheFirst
            | ModelDisagreementAspect::EffectKindOfTheOperation
            | ModelDisagreementAspect::NotModeled => false,
        },
    };
    every_violation_is_explained && disagreement_is_explained
}

/// 一张重开的盘在录制流里的样子：空盘叠上 `operations` 就是它（录制流只记落了盘的写）。给定状态引擎照它建给定状态表：
/// mkfs 写的那几步进基线（`operations_written_by_make_filesystem`），掉电接缝照 `earliest_stream_index_after_each_power_loss` 断开记录核对器的发布表。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedStreamOfAnImage {
    pub operations: Vec<crate::RetainedOperation>,
    /// 录制流头上 mkfs 写的步数（[`HistoryRun::operations_written_by_make_filesystem`]）。
    pub operations_written_by_make_filesystem: usize,
    /// 每次掉电之后录制流里第一步的下标（[`HistoryRun::earliest_stream_index_after_each_power_loss`]）。
    pub earliest_stream_index_after_each_power_loss: Vec<usize>,
}

/// 一个注入点重开的盘经钩子交出去的那一份（[`inject_faults_into_history_handing_out_the_images`]）：池级 checker 那一遍归给定状态引擎两段
/// （镜像作为给定状态写进判定库、判段从库读、判在 GPU），判出的违例交回
/// [`classify_the_pool_checker_verdicts_of_the_images_after_a_fault`] 照这里带的事实归类（故障注入自己那条「已知红」、说谎的设备留下的不一致、新发现）。
/// 模型、RefFS 与执行器判出的失败 harness 自己归类过，不在这里。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImagesAfterAFault {
    pub seed: HistorySeed,
    pub drawn: DrawnFault,
    pub device_width: HistoryDeviceWidth,
    /// 注入之后重开的那张盘。
    pub image_after_the_fault: RecordedStreamOfAnImage,
    /// 带着注入跑的那一遍每一步的镜像（[`crate::history::StepImage`]，次序同历史）：都是 `image_after_the_fault` 录制流的前缀。
    pub step_images_of_the_run_with_the_fault: Vec<StepImageInTheRecordedStream>,
    /// 说谎的设备吞掉了写时：被吞的写按原位插回去之后的那张盘（实现以为自己写下的那一份；判「这条失败是不是它留下的」）。
    pub image_with_the_swallowed_writes_landed: Option<RecordedStreamOfAnImage>,
    /// 说谎的设备吞掉的每一次写，按发生的先后（每步镜像各记着它那一刻之前吞了几次，按步对照那张盘照它插：
    /// [`ImagesAfterAFault::image_at_the_step_image_with_the_swallowed_writes_before_it_landed`]）。
    pub swallowed_writes: Vec<SwallowedWrite>,
    /// 实现交回的每一份隔离，跨重挂累积（[`crate::history::HistoryRun::copies_quarantined_by_the_implementation`]；每步镜像各记着它那一刻之前交回了几份）：
    /// 归类器认「插回去那张盘上仍红的那一份是说谎的设备逼出来的隔离」（[`unit_start_slots_quarantined_over_a_swallowed_write`]）。
    pub copies_quarantined_by_the_implementation: Vec<CopyQuarantinedByTheImplementation>,
    /// 这个注入点上池级 checker 判红时那一条失败除违例之外的样子（位置、根环、抬 F 落没落在空档里），违例一栏是空的。
    pub observation_without_the_pool_checker_violations: FailureObservation,
    pub facts_after_the_fault: FactsAfterTheFault,
    /// 说谎的设备吞掉了写时判「这条失败是不是它留下的」要的那几样，池级 checker 那一项（插回去之后仍红的不变量）是空的、由归类那一刻填。
    pub what_the_swallowed_writes_explain_without_the_pool_checker:
        Option<WhatTheSwallowedWritesExplain>,
    /// 重开走到了哪一版（给人看的一句，新发现带着它）。
    pub reopen: String,
}

/// 带着注入跑的那一遍里一张每步镜像：空盘叠上重开那张盘录制流的头 `operations_recorded` 步，连同 harness 交出去那一刻读的根环
/// （池级 checker 在它上面判红时那一条失败要带）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepImageInTheRecordedStream {
    pub position: StepPosition,
    pub operation_kind: Option<HistoryOperationKind>,
    pub operations_recorded: usize,
    pub newest_ring_root_txg: Option<u64>,
    pub root_ring_slot_count: Option<u64>,
    /// 这一刻之前说谎的设备吞了几次写（[`ImagesAfterAFault::swallowed_writes`] 的头这么多次）。
    pub swallowed_writes_before_it: usize,
    /// 这一刻之前实现交回了几份隔离（[`ImagesAfterAFault::copies_quarantined_by_the_implementation`] 的头这么多份）。
    pub copies_quarantined_by_the_implementation_before_it: usize,
}

impl ImagesAfterAFault {
    /// 按步对照的那张盘（判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3：归类器对照第一张红的那一步镜像那一刻
    /// 插回去的盘）：带着注入跑的那一遍第 `step_image_index` 张每步镜像那一刻的录制流前缀，把那一刻之前被吞的写按原位插回去。
    /// 那一刻之前一次写都没吞时交 `None`（插回去的盘就是那张每步镜像本身，那一张上判红的没有一条是被吞的写留下的）。
    /// 掉电接缝只留落在前缀里的那几道。
    ///
    /// # Panics
    /// `step_image_index` 不是一张交出去的每步镜像。
    #[must_use]
    pub fn image_at_the_step_image_with_the_swallowed_writes_before_it_landed(
        &self,
        step_image_index: usize,
    ) -> Option<RecordedStreamOfAnImage> {
        let step_image = self
            .step_images_of_the_run_with_the_fault
            .get(step_image_index)
            .unwrap_or_else(|| {
                panic!(
                    "交出去的每步镜像只有 {} 张，没有第 {step_image_index} 张",
                    self.step_images_of_the_run_with_the_fault.len()
                )
            });
        let swallowed_before_it = &self.swallowed_writes[..step_image.swallowed_writes_before_it];
        if swallowed_before_it.is_empty() {
            return None;
        }
        let prefix = RecordedStreamOfAnImage {
            operations: self.image_after_the_fault.operations[..step_image.operations_recorded]
                .to_vec(),
            operations_written_by_make_filesystem: self
                .image_after_the_fault
                .operations_written_by_make_filesystem
                .min(step_image.operations_recorded),
            earliest_stream_index_after_each_power_loss: self
                .image_after_the_fault
                .earliest_stream_index_after_each_power_loss
                .iter()
                .copied()
                .filter(|earliest_stream_index| {
                    *earliest_stream_index < step_image.operations_recorded
                })
                .collect(),
        };
        Some(stream_with_the_swallowed_writes_landed(
            &prefix,
            swallowed_before_it,
        ))
    }
}

/// 一条注入之后的失败归到哪一格（[`classify_a_failure_after_a_fault`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureAfterAFault {
    KnownRed(KnownRedFormAtAFault),
    /// 说谎的设备吞掉写之后盘面不一致：设备丢的，不是实现的缺口（[`left_by_the_lying_device`]）；按签名计数、不算新发现。
    LeftByTheLyingDevice {
        signature: FailureSignature,
    },
    NewFinding {
        signature: FailureSignature,
        observation: Box<FailureObservation>,
    },
}

/// 一条注入之后的失败归类：故障注入自己那几条「已知红」先认，再对共用清单，清单外的说谎的设备留下的不一致单记，其余是新发现。
/// harness 就地判的失败与给定状态引擎两段判出的池级 checker 违例走同一个归类。
#[must_use]
pub fn classify_a_failure_after_a_fault(
    observation: FailureObservation,
    facts: &FactsAfterTheFault,
    fault: InjectedFault,
    what_the_swallowed_writes_explain: Option<&WhatTheSwallowedWritesExplain>,
) -> FailureAfterAFault {
    // 「已知红」先认：清单里那几条是认下来的形态（共用清单是实现与 checker 读法对不上的老账，增补 2 收口表第 ②、43 行；
    // 故障注入自己那一条是「mkfs 没写下系统配置槽，全池皆废判红是对的」），
    // 与设备说不说谎无关，说谎的设备把它撞出来时照样记进「已知红」那一格，不许被下面那一条接走。
    if let Some(form) = known_red_form_only_after_a_fault(&observation, facts) {
        return FailureAfterAFault::KnownRed(KnownRedFormAtAFault::OnlyAfterAFault(form));
    }
    match classify_failure(observation) {
        HistoryEnding::KnownRed { form, .. } => {
            FailureAfterAFault::KnownRed(KnownRedFormAtAFault::SharedWithRandomHistories(form))
        }
        HistoryEnding::NewFinding {
            signature,
            observation,
        } => {
            // 说谎的设备吞掉写之后盘面不一致：设备丢的，不是实现的缺口（`InjectedFault::the_device_lies`）。
            // 按注入点认（`left_by_the_lying_device`，实七）：被吞的写插回去就消失的违例、
            // 去掉被吞的根之后模型重算对得上的上限才豁免，别的照报成新发现。
            // panic 不走这一格——说谎的设备也不许把实现打 panic，那一条照样报成新发现。
            if fault.the_device_lies()
                && what_the_swallowed_writes_explain
                    .is_some_and(|explained| left_by_the_lying_device(&observation, explained))
            {
                FailureAfterAFault::LeftByTheLyingDevice { signature }
            } else {
                FailureAfterAFault::NewFinding {
                    signature,
                    observation: Box::new(observation),
                }
            }
        }
        HistoryEnding::Completed => {
            unreachable!("classify_failure 只交回「已知红」或新发现，不交回跑完")
        }
    }
}

/// 一张盘上池级 checker 判红的，连同违例点名的（不变量, 盘, 槽）（`singlefs_checker::walk::check_pool_image_naming_the_violation_sites` 交回的那两样；
/// 点名槽的只有 I-3.1、I-3.11、I-3.10 与 I-3.15，别的不变量判红时这里没有它的行）。违例细节可以是占位的一句：归类只看判红的是哪几条、点名了哪几处。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ViolationsNamingTheSites {
    pub violations: Vec<(&'static str, String)>,
    pub violation_sites: BTreeSet<ViolationSite>,
}

/// 给定状态引擎两段判回的、一个注入点交出去的那几张盘上池级 checker 判红的（[`classify_the_pool_checker_verdicts_of_the_images_after_a_fault`] 收它）。
/// 违例点名的（不变量, 盘, 槽）只在拿来比的那几张上要：第一张判红的每步镜像与它的按步对照盘、重开那张盘与被吞的写都插回去那张盘。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolCheckerVerdictsOfTheImagesAfterAFault {
    /// 与 [`ImagesAfterAFault::step_images_of_the_run_with_the_fault`] 逐张对应。
    pub violations_of_the_step_images: Vec<Vec<(&'static str, String)>>,
    /// 第一张判红的每步镜像上违例点名的（不变量, 盘, 槽）；一张都不红时空。
    pub violation_sites_of_the_first_red_step_image: BTreeSet<ViolationSite>,
    /// 重开那张盘（[`ImagesAfterAFault::image_after_the_fault`]）上判红的。
    pub after_the_fault: ViolationsNamingTheSites,
    /// 第一张判红的每步镜像的按步对照盘（[`ImagesAfterAFault::image_at_the_step_image_with_the_swallowed_writes_before_it_landed`]）上判红的：
    /// 那个方法交回 `Some` 时给，交回 `None`（一张都不红、或那一刻之前一次写都没吞）时是 `None`。
    pub at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed:
        Option<ViolationsNamingTheSites>,
    /// 被吞的写都插回去之后那张盘（[`ImagesAfterAFault::image_with_the_swallowed_writes_landed`]）上判红的：那张盘在时给，不在时是 `None`。
    pub with_the_swallowed_writes_landed: Option<ViolationsNamingTheSites>,
}

/// 隔离判据判的那三条（I-3.1、I-3.11 与随 I-3.1 那一份的 I-3.15，`crates/singlefs-checker/invariants/05-空间记账.md`）：
/// 对照盘上落在 [`unit_start_slots_quarantined_over_a_swallowed_write`] 那几个起点槽上的点名，归类器先减掉再比。
const INVARIANTS_JUDGED_BY_THE_QUARANTINE_CRITERION: [&str; 3] = [
    "I-3.1",
    "I-3.11",
    UNRELEASED_ALLOCATION_RECORD_COVERS_A_UNIT_THE_CENTRAL_MAPPING_PLACES_THERE,
];

/// 实现交回的隔离里，那一份与一次被吞的写有重叠字节的那几个单元的起点槽（两盘同槽，按槽认、不分盘）。被吞的写插回去之后这几个单元每一份都对得上、
/// 记录却留在已分配：那是实现在说谎的设备上读盘核出对不上才隔离的，不是实现的缺口（判决 `research/prompts/m4-i315-r2-main-verification.md`
/// 甲二「把实现交出、与被吞的写有重叠的单元当隔离单元」 收齐四条路的清单）。
#[must_use]
pub fn unit_start_slots_quarantined_over_a_swallowed_write(
    copies_quarantined_by_the_implementation: &[CopyQuarantinedByTheImplementation],
    swallowed_writes: &[SwallowedWrite],
) -> BTreeSet<u64> {
    let swallowed_ranges: Vec<SwallowedWriteByteRange> = swallowed_writes
        .iter()
        .map(SwallowedWriteByteRange::of)
        .collect();
    copies_quarantined_by_the_implementation
        .iter()
        .filter(|quarantined| {
            let copy = quarantined.copy;
            let start_in_bytes = copy.placement.slot.to_device_offset().0;
            let end_in_bytes = copy
                .placement
                .span
                .checked_mul(SLOT_BYTES)
                .and_then(|length_in_bytes| start_in_bytes.checked_add(length_in_bytes))
                .expect("一个落点的末端装得进 u64：分配器只在盘内取落点");
            swallowed_ranges.iter().any(|swallowed| {
                swallowed.device.0 == copy.device.0
                    && swallowed.start_in_bytes < end_in_bytes
                    && start_in_bytes < swallowed.end_in_bytes
            })
        })
        .map(|quarantined| quarantined.copy.placement.slot.0)
        .collect()
}

/// 被比的那一张上判红的每一条里，拿对照盘比之后仍红的那几条（判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3：按（不变量, 盘, 槽）比）：
/// - 被比那一张上这一条点了槽、对照盘上这一条也点了槽：被比那一张点名的（盘, 槽）里有一处在对照盘上这一条下仍被点名，才算仍红；
///   隔离判据那三条（[`INVARIANTS_JUDGED_BY_THE_QUARANTINE_CRITERION`]），对照盘上落在 `quarantined_over_a_swallowed_write` 那几个起点槽上的点名不算；
/// - 被比那一张上这一条没点槽（I-3.x 之外的不变量，或判红的不是哪一个单元），或对照盘上这一条一个槽都没点：按名比，对照盘上这一条红就算仍红。
///
/// `comparison` 是 `None`（这一刻之前一次写都没吞，插回去就是被比那一张本身）时每一条都算仍红。
#[must_use]
pub fn invariants_still_violated_on_the_comparison_image(
    observed: &ViolationsNamingTheSites,
    comparison: Option<&ViolationsNamingTheSites>,
    quarantined_over_a_swallowed_write: &BTreeSet<u64>,
) -> BTreeSet<&'static str> {
    let observed_invariants = observed.violations.iter().map(|(invariant, _)| *invariant);
    let Some(comparison) = comparison else {
        return observed_invariants.collect();
    };
    let violated_on_the_comparison: BTreeSet<&'static str> = comparison
        .violations
        .iter()
        .map(|(invariant, _)| *invariant)
        .collect();
    observed_invariants
        .filter(|invariant| {
            let sites_named_for = |sites: &BTreeSet<ViolationSite>| -> Vec<ViolationSite> {
                sites
                    .iter()
                    .filter(|site| site.invariant == *invariant)
                    .copied()
                    .collect()
            };
            let observed_sites = sites_named_for(&observed.violation_sites);
            let comparison_sites = sites_named_for(&comparison.violation_sites);
            if observed_sites.is_empty() || comparison_sites.is_empty() {
                return violated_on_the_comparison.contains(invariant);
            }
            let judged_by_the_quarantine_criterion =
                INVARIANTS_JUDGED_BY_THE_QUARANTINE_CRITERION.contains(invariant);
            observed_sites.iter().any(|site| {
                comparison_sites.contains(site)
                    && !(judged_by_the_quarantine_criterion
                        && quarantined_over_a_swallowed_write.contains(&site.slot))
            })
        })
        .collect()
}

/// 给定状态引擎两段判出的池级 checker 结论归类（harness 不判，那一遍的结论交回这里；要判红就停那一档口径的调用方改走
/// [`inject_faults_into_history_merging_the_pool_checker_verdicts_of_the_given_states_engine`]）。交回归好类的失败：每步镜像里第一张判红的
/// （harness 就地判时历史停在那一步，之后的步不跑）、重开那张判红的，各一条；都不红交回空。
///
/// 「这条失败是不是说谎的设备留下的」（[`left_by_the_lying_device`]）要的「插回去仍红的不变量」按步取对照盘、按（不变量, 盘, 槽）比
/// （判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C3，[`invariants_still_violated_on_the_comparison_image`]）：
/// 第一张判红的每步镜像拿它那一刻的按步对照盘比（那一刻之前被吞的写插回去），减掉那一刻之前实现交回的、压在那一刻之前被吞的写上的隔离；
/// 重开那张拿被吞的写都插回去那张比，减掉整段交回的、压在被吞的写上的隔离。
///
/// # Panics
/// `violations_of_the_step_images` 与每步镜像张数不同；按步对照盘该给没给、不该给给了；被吞的写都插回去那张盘在而没给结论、不在而给了。
#[must_use]
pub fn classify_the_pool_checker_verdicts_of_the_images_after_a_fault(
    images: &ImagesAfterAFault,
    verdicts: PoolCheckerVerdictsOfTheImagesAfterAFault,
) -> Vec<FailureAfterAFault> {
    let PoolCheckerVerdictsOfTheImagesAfterAFault {
        mut violations_of_the_step_images,
        violation_sites_of_the_first_red_step_image,
        after_the_fault,
        at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed,
        with_the_swallowed_writes_landed,
    } = verdicts;
    assert_eq!(
        violations_of_the_step_images.len(),
        images.step_images_of_the_run_with_the_fault.len(),
        "每步镜像的结论与交出去的每步镜像逐张对应"
    );
    let first_red_step_image = violations_of_the_step_images
        .iter()
        .position(|violations| !violations.is_empty());
    let write_was_swallowed_before_the_first_red_step_image =
        first_red_step_image.is_some_and(|index| {
            images.step_images_of_the_run_with_the_fault[index].swallowed_writes_before_it > 0
        });
    assert_eq!(
        at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed.is_some(),
        write_was_swallowed_before_the_first_red_step_image,
        "第一张判红的每步镜像那一刻之前吞过写，就要给它的按步对照盘的结论；没吞过就不给"
    );
    assert_eq!(
        with_the_swallowed_writes_landed.is_some(),
        images.image_with_the_swallowed_writes_landed.is_some(),
        "被吞的写都插回去那张盘在就要给它的结论；不在就不给"
    );
    let explained_with = |invariants_still_violated: BTreeSet<&'static str>| {
        images
            .what_the_swallowed_writes_explain_without_the_pool_checker
            .clone()
            .map(|explained| {
                explained.with_the_invariants_still_violated(invariants_still_violated)
            })
    };
    let mut failures = Vec::new();
    if let Some(index) = first_red_step_image {
        let step_image = &images.step_images_of_the_run_with_the_fault[index];
        let observed = ViolationsNamingTheSites {
            violations: std::mem::take(&mut violations_of_the_step_images[index]),
            violation_sites: violation_sites_of_the_first_red_step_image,
        };
        let quarantined_over_a_swallowed_write =
            unit_start_slots_quarantined_over_a_swallowed_write(
                &images.copies_quarantined_by_the_implementation
                    [..step_image.copies_quarantined_by_the_implementation_before_it],
                &images.swallowed_writes[..step_image.swallowed_writes_before_it],
            );
        let invariants_still_violated = invariants_still_violated_on_the_comparison_image(
            &observed,
            at_the_first_red_step_image_with_the_swallowed_writes_before_it_landed.as_ref(),
            &quarantined_over_a_swallowed_write,
        );
        let observation = FailureObservation {
            position: step_image.position,
            operation_kind: step_image.operation_kind,
            violations: observed.violations,
            panic: None,
            newest_ring_root_txg: step_image.newest_ring_root_txg,
            root_ring_slot_count: step_image.root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        failures.push(classify_a_failure_after_a_fault(
            observation,
            &images.facts_after_the_fault,
            images.drawn.fault,
            explained_with(invariants_still_violated).as_ref(),
        ));
    }
    if !after_the_fault.violations.is_empty() {
        let quarantined_over_a_swallowed_write =
            unit_start_slots_quarantined_over_a_swallowed_write(
                &images.copies_quarantined_by_the_implementation,
                &images.swallowed_writes,
            );
        let invariants_still_violated = invariants_still_violated_on_the_comparison_image(
            &after_the_fault,
            with_the_swallowed_writes_landed.as_ref(),
            &quarantined_over_a_swallowed_write,
        );
        let template = &images.observation_without_the_pool_checker_violations;
        let observation = FailureObservation {
            position: template.position,
            operation_kind: template.operation_kind,
            violations: after_the_fault.violations,
            panic: None,
            newest_ring_root_txg: template.newest_ring_root_txg,
            root_ring_slot_count: template.root_ring_slot_count,
            harness_judgement: None,
            model_disagreement: None,
            raised_floor_lands_only_on_abandoned_roots: template
                .raised_floor_lands_only_on_abandoned_roots,
            record_check: RecordCheck::default(),
        };
        failures.push(classify_a_failure_after_a_fault(
            observation,
            &images.facts_after_the_fault,
            images.drawn.fault,
            explained_with(invariants_still_violated).as_ref(),
        ));
    }
    failures
}

/// 被吞的那一次写要是落了盘，录制流里它排在第几步之前（插进去的下标）：录制流里第 `writes_recorded_before` 次写之后（屏障不改镜像，
/// 排在那几道屏障前后都一样）。
fn stream_index_the_swallowed_write_lands_at(
    operations: &[crate::RetainedOperation],
    write: &SwallowedWrite,
) -> usize {
    if write.writes_recorded_before == 0 {
        return 0;
    }
    operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| retained.operation.kind != RecordedOperationKind::Barrier)
        .nth(write.writes_recorded_before - 1)
        .map(|(index, _)| index + 1)
        .unwrap_or_else(|| {
            panic!(
                "被吞的写之前记着 {} 次写，录制流里的写不够这么多：注入层与录制器数的写对不上",
                write.writes_recorded_before
            )
        })
}

/// 被吞的那几次写要是都落了盘，录制流该是哪一串：各自插在 [`stream_index_the_swallowed_write_lands_at`] 那一处。
/// 从写数最大的那一次往前插：插进去的写只挪它后面的，排在前面的那几次的位置不变。
fn operations_with_the_swallowed_writes_landed(
    operations: &[crate::RetainedOperation],
    swallowed: &[SwallowedWrite],
) -> Vec<crate::RetainedOperation> {
    let mut landed = operations.to_vec();
    let mut by_position: Vec<&SwallowedWrite> = swallowed.iter().collect();
    by_position.sort_by_key(|write| write.writes_recorded_before);
    for write in by_position.into_iter().rev() {
        let stream_index = stream_index_the_swallowed_write_lands_at(operations, write);
        landed.insert(stream_index, write.as_retained_operation());
    }
    landed
}

/// 被吞的写插回去之后的那张盘（[`RecordedStreamOfAnImage`]）：mkfs 的步数与掉电之后第一步的下标跟着插进去的写往后挪。
/// 插在 mkfs 那几步之后第一处（恰好插在边界上）的算 mkfs 之后写的；插在掉电之后第一步之前（恰好插在接缝上）的算掉电之前写的。
/// 写死一段历史、照入口的样子拼 [`ImagesAfterAFault`] 的用例也经它插（同一份插法）。
///
/// # Panics
/// `swallowed` 里有一次写之前记着的写数比 `stream` 里的写多（注入层与录制器数的写对不上）。
#[must_use]
pub fn stream_with_the_swallowed_writes_landed(
    stream: &RecordedStreamOfAnImage,
    swallowed: &[SwallowedWrite],
) -> RecordedStreamOfAnImage {
    let landing_indexes: Vec<usize> = swallowed
        .iter()
        .map(|write| stream_index_the_swallowed_write_lands_at(&stream.operations, write))
        .collect();
    let landed_strictly_before = |boundary: usize| {
        landing_indexes
            .iter()
            .filter(|landing_index| **landing_index < boundary)
            .count()
    };
    let landed_at_or_before = |stream_index: usize| {
        landing_indexes
            .iter()
            .filter(|landing_index| **landing_index <= stream_index)
            .count()
    };
    RecordedStreamOfAnImage {
        operations: operations_with_the_swallowed_writes_landed(&stream.operations, swallowed),
        operations_written_by_make_filesystem: stream.operations_written_by_make_filesystem
            + landed_strictly_before(stream.operations_written_by_make_filesystem),
        earliest_stream_index_after_each_power_loss: stream
            .earliest_stream_index_after_each_power_loss
            .iter()
            .map(|earliest_stream_index| {
                earliest_stream_index + landed_at_or_before(*earliest_stream_index)
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::CapturedPanic;
    use crate::model::{ModelCheckpointTxg, ModelDisagreement};

    /// 去掉被吞的根之后重算的上限，被吞的根没留下读回全零的槽：并与不并两个数相同。
    fn ceilings_not_lowered_by_ring_slots_left_all_zero(
        recomputed_txg: u64,
    ) -> ModelRollbackFloorCeilingsWithoutTheRoots {
        ModelRollbackFloorCeilingsWithoutTheRoots {
            from_the_roots_read: ModelCheckpointTxg(recomputed_txg),
            merging_the_ring_slots_left_all_zero: ModelCheckpointTxg(recomputed_txg),
        }
    }

    /// 被吞的根槽写留下读回全零的槽、重挂之后实现把它并进准入抬 F 的上限（D16（发布语义） 已定项 1「抬 F 的上限」那一行「并」；
    /// 判决 `research/prompts/m3-close-b19-reason-r3-main-verification.md` 第二节 Q4 第 4 问）：
    /// 场景：模型去掉被吞的根之后重算，不并 9、并进那一槽 7。
    /// 预期：「抬 F 的上限」那一格实现报 7 才认，报 9（不并的那个）照报新发现；推空那一格（模型不答）实现报 7、并进来确实压低了上限才认，
    /// 报 9、或重算里并与不并一样（卡住上限的不是被吞的根留下的槽）照报新发现。
    /// 判别力：「抬 F 的上限」那一格比不并的那个数时第一条红（`crates/mutations.tsv`「故障注入认上限只比不并全零槽的那个数」）；
    /// 推空那一格一概不认时第三条红（「故障注入不认推空」）。
    #[test]
    fn ceiling_held_by_the_ring_slot_a_swallowed_root_left_all_zero_is_left_by_the_lying_device_only_at_the_merged_ceiling(
    ) {
        let observation = |disagreement: ModelDisagreement| FailureObservation {
            position: StepPosition::Operation(10),
            operation_kind: Some(HistoryOperationKind::RaiseRollbackFloor),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement: Some(disagreement),
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        let reported = |aspect: ModelDisagreementAspect, reported_ceiling: u64| ModelDisagreement {
            aspect,
            model_answer: "上限 Some(9)".to_string(),
            implementation_answer: format!("实现报上限 {reported_ceiling}"),
            implementation_reported_ceiling: Some(ModelCheckpointTxg(reported_ceiling)),
            implementation_reported_unreadable_place: None,
            implementation_rows_written: None,
        };
        let explained =
            |recomputed: ModelRollbackFloorCeilingsWithoutTheRoots| WhatTheSwallowedWritesExplain {
                invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet::new(),
                ceiling_without_the_swallowed_roots: Some(Some(recomputed)),
                root_ring_slots_of_the_swallowed_root_slot_writes: BTreeSet::new(),
                instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: None,
                byte_ranges_of_the_swallowed_writes: BTreeSet::new(),
            };
        let merged_at_the_all_zero_ring_slot = ModelRollbackFloorCeilingsWithoutTheRoots {
            from_the_roots_read: ModelCheckpointTxg(9),
            merging_the_ring_slots_left_all_zero: ModelCheckpointTxg(7),
        };
        let ceiling = ModelDisagreementAspect::RollbackFloorCeiling;
        let empty_publish_keeping_the_floor =
            ModelDisagreementAspect::EmptyPublishKeepingTheFloorWhileTheCeilingIsHeldIsNotModeled;
        assert!(
            left_by_the_lying_device(
                &observation(reported(ceiling, 7)),
                &explained(merged_at_the_all_zero_ring_slot)
            ),
            "实现报的 7 是并进被吞的根留下的全零槽之后的上限"
        );
        assert!(
            !left_by_the_lying_device(
                &observation(reported(ceiling, 9)),
                &explained(merged_at_the_all_zero_ring_slot)
            ),
            "实现报 9：没并那一槽，与 D16 不符，照报"
        );
        assert!(
            left_by_the_lying_device(
                &observation(reported(empty_publish_keeping_the_floor, 7)),
                &explained(merged_at_the_all_zero_ring_slot)
            ),
            "推空时实现报的 7 是并进被吞的根留下的全零槽之后的上限，卡住上限的是那一槽"
        );
        assert!(
            !left_by_the_lying_device(
                &observation(reported(empty_publish_keeping_the_floor, 9)),
                &explained(merged_at_the_all_zero_ring_slot)
            ),
            "推空时实现报 9：不是并进那一槽之后的上限，照报"
        );
        assert!(
            !left_by_the_lying_device(
                &observation(reported(empty_publish_keeping_the_floor, 7)),
                &explained(ceilings_not_lowered_by_ring_slots_left_all_zero(7))
            ),
            "被吞的根没留下压低上限的全零槽：卡住上限的不是说谎的设备留下的，照报"
        );
    }

    /// 按注入点认说谎的设备留下的失败（`left_by_the_lying_device`，实七）：被吞的写插回去就消失的违例、
    /// 去掉被吞的根之后模型重算对得上的上限才认；插回去仍红的违例、重算对不上的上限、别的模型格、panic、执行器判出的一概照报。
    #[test]
    fn only_failures_that_the_swallowed_writes_explain_are_left_by_the_lying_device() {
        let observation =
            |violations: &[&'static str],
             disagreement: Option<ModelDisagreement>,
             panic: Option<CapturedPanic>| FailureObservation {
                position: StepPosition::Operation(4),
                operation_kind: Some(HistoryOperationKind::PublishOverwrite),
                violations: violations
                    .iter()
                    .map(|invariant| (*invariant, "盘 0 槽 50180".to_string()))
                    .collect(),
                panic,
                newest_ring_root_txg: None,
                root_ring_slot_count: None,
                harness_judgement: None,
                model_disagreement: disagreement,
                raised_floor_lands_only_on_abandoned_roots: None,
                record_check: RecordCheck::default(),
            };
        let ceiling_disagreement = |reported: u64| ModelDisagreement {
            aspect: ModelDisagreementAspect::RollbackFloorCeiling,
            model_answer: "上限 Some(3)".to_string(),
            implementation_answer: format!("实现报上限 {reported}"),
            implementation_reported_ceiling: Some(ModelCheckpointTxg(reported)),
            implementation_reported_unreadable_place: None,
            implementation_rows_written: None,
        };
        let explained = |still_violated: &[&'static str], ceiling: Option<Option<u64>>| {
            WhatTheSwallowedWritesExplain {
                invariants_still_violated_if_the_swallowed_writes_had_landed: still_violated
                    .iter()
                    .copied()
                    .collect(),
                ceiling_without_the_swallowed_roots: ceiling.map(|recomputed| {
                    recomputed.map(ceilings_not_lowered_by_ring_slots_left_all_zero)
                }),
                root_ring_slots_of_the_swallowed_root_slot_writes: BTreeSet::new(),
                instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: None,
                byte_ranges_of_the_swallowed_writes: BTreeSet::new(),
            }
        };
        let lost_unit = ["I-2.1", "I-3.10", "I-4.8", "I-7.4"];
        assert!(left_by_the_lying_device(
            &observation(&lost_unit, None, None),
            &explained(&[], None)
        ));
        assert!(
            !left_by_the_lying_device(
                &observation(&lost_unit, None, None),
                &explained(&["I-3.10"], None)
            ),
            "被吞的写插回去 I-3.10 仍红：不是注入点造成的"
        );
        assert!(left_by_the_lying_device(
            &observation(&[], Some(ceiling_disagreement(0)), None),
            &explained(&[], Some(Some(0)))
        ));
        for (recomputed, why) in [
            (Some(Some(1)), "去掉被吞的根之后重算的上限不等于实现报的"),
            (Some(None), "去掉被吞的根之后模型算不出上限"),
            (None, "被吞的写里没有根槽写，或没有停下那一步之前的模型"),
        ] {
            assert!(
                !left_by_the_lying_device(
                    &observation(&[], Some(ceiling_disagreement(0)), None),
                    &explained(&[], recomputed)
                ),
                "{why}"
            );
        }
        let read_back_disagreement = ModelDisagreement {
            aspect: ModelDisagreementAspect::ColdStartReadBack,
            model_answer: "读回 (5,19)".to_string(),
            implementation_answer: "读回 (5,18)".to_string(),
            implementation_reported_ceiling: None,
            implementation_reported_unreadable_place: None,
            implementation_rows_written: None,
        };
        assert!(
            !left_by_the_lying_device(
                &observation(&[], Some(read_back_disagreement), None),
                &explained(&[], Some(Some(0)))
            ),
            "上限之外的模型格不认"
        );
        assert!(
            !left_by_the_lying_device(
                &observation(
                    &lost_unit,
                    None,
                    Some(CapturedPanic {
                        location: "crates/singlefs-core/src/transaction.rs:1".to_string(),
                        message: "断言".to_string(),
                    })
                ),
                &explained(&[], None)
            ),
            "说谎的设备也不许把实现打 panic"
        );
    }

    /// 按注入点认的另两格（实八）：实现拒抬 F、报读坏重读仍坏的那个根环槽正是被吞的根槽写的槽，才认——不管模型要它成还是按上限拒；
    /// 槽对不上、实现没报槽（别的拒绝，或拒之前写了盘）一概照报。实现写的实例表行等于「被吞的根由记录重建」时模型重算的行，才认；
    /// 行对不上、模型重算不出，照报。
    #[test]
    fn refused_floor_raise_and_the_rows_written_are_left_by_the_lying_device_only_when_the_swallowed_root_slot_write_explains_them(
    ) {
        let observation = |disagreement: ModelDisagreement| FailureObservation {
            position: StepPosition::Operation(5),
            operation_kind: Some(HistoryOperationKind::RaiseRollbackFloor),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement: Some(disagreement),
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        let swallowed_slot = ModelRingPosition {
            region: 1,
            slot_in_region: 1,
        };
        let another_slot = ModelRingPosition {
            region: 2,
            slot_in_region: 1,
        };
        let refusal = |aspect: ModelDisagreementAspect, reported: Option<ModelRingPosition>| {
            ModelDisagreement {
                aspect,
                model_answer: "RaiseRollbackFloor 该成".to_string(),
                implementation_answer:
                    "拒了：MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread"
                        .to_string(),
                implementation_reported_ceiling: None,
                implementation_reported_unreadable_place: reported
                    .map(ModelReportedUnreadablePlace::RootRingSlotStillBadAfterOneReread),
                implementation_rows_written: None,
            }
        };
        let row = |applied_transaction_high_water: u64| ModelInstanceRow {
            instance: crate::model::ModelInstanceGeneration(1),
            selected_root_txg: ModelCheckpointTxg(4),
            applied_transaction_high_water,
        };
        let rows_disagreement = |written: Option<Vec<ModelInstanceRow>>| ModelDisagreement {
            aspect: ModelDisagreementAspect::InstanceRows,
            model_answer: "写行 [(1, 4, 0)]".to_string(),
            implementation_answer: "写行 [(1, 4, 2)]".to_string(),
            implementation_reported_ceiling: None,
            implementation_reported_unreadable_place: None,
            implementation_rows_written: written,
        };
        let explained = |slots: &[ModelRingPosition], rows: Option<Vec<ModelInstanceRow>>| {
            WhatTheSwallowedWritesExplain {
                invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet::new(),
                ceiling_without_the_swallowed_roots: None,
                root_ring_slots_of_the_swallowed_root_slot_writes: slots.iter().copied().collect(),
                instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: rows,
                byte_ranges_of_the_swallowed_writes: BTreeSet::new(),
            }
        };
        for aspect in [
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
            ModelDisagreementAspect::RefusalReason,
        ] {
            assert!(
                left_by_the_lying_device(
                    &observation(refusal(aspect, Some(swallowed_slot))),
                    &explained(&[swallowed_slot], None)
                ),
                "{aspect:?}：读坏的正是被吞那次写的槽"
            );
            assert!(
                !left_by_the_lying_device(
                    &observation(refusal(aspect, Some(another_slot))),
                    &explained(&[swallowed_slot], None)
                ),
                "{aspect:?}：读坏的槽不是被吞那次写的"
            );
            assert!(
                !left_by_the_lying_device(
                    &observation(refusal(aspect, None)),
                    &explained(&[swallowed_slot], None)
                ),
                "{aspect:?}：实现没报读坏的槽（别的拒绝，或拒之前写了盘）"
            );
        }
        assert!(
            left_by_the_lying_device(
                &observation(rows_disagreement(Some(vec![row(2)]))),
                &explained(&[], Some(vec![row(2)]))
            ),
            "实现写的行等于被吞的根由记录重建之后重算的行"
        );
        for (written, recomputed, why) in [
            (
                Some(vec![row(2)]),
                Some(vec![row(0)]),
                "重算的行与实现写的不同",
            ),
            (Some(vec![row(2)]), None, "模型重算不出"),
            (None, Some(vec![row(2)]), "对不上的那一格没带实现写的行"),
        ] {
            assert!(
                !left_by_the_lying_device(
                    &observation(rows_disagreement(written)),
                    &explained(&[], recomputed)
                ),
                "{why}"
            );
        }
    }

    /// 按注入点认的回退那一格（`research/prompts/code-backlog-b2-code-r1-main-verification.md`「更正」一节 A4）：实现拒管理员回退、
    /// 点名的「回退目标引用的一个单元在一块盘上的那一份」与一次被吞的写有重叠的字节，才认——不管模型要它成还是要别的理由；
    /// 被吞的写落在另一块盘上、紧挨着那一份而不重叠、实现没点名那一份（别的拒绝，或拒之前写了盘），一概照报。
    #[test]
    fn refused_rollback_is_left_by_the_lying_device_only_when_the_unit_copy_it_names_overlaps_a_swallowed_write(
    ) {
        const UNIT_SLOT: u64 = 51_000;
        let observation = |disagreement: ModelDisagreement| FailureObservation {
            position: StepPosition::Operation(18),
            operation_kind: Some(HistoryOperationKind::RollBackWhileMounted),
            violations: Vec::new(),
            panic: None,
            newest_ring_root_txg: None,
            root_ring_slot_count: None,
            harness_judgement: None,
            model_disagreement: Some(disagreement),
            raised_floor_lands_only_on_abandoned_roots: None,
            record_check: RecordCheck::default(),
        };
        let unit_copy_on = |device: u32| ModelUnitCopyLocation {
            device: ModelDeviceIdentity(device),
            span_in_slots: 1,
            first_slot: UNIT_SLOT,
        };
        let refusal = |aspect: ModelDisagreementAspect, reported: Option<ModelUnitCopyLocation>| {
            let mut disagreement = ModelDisagreement::new(
                aspect,
                "RollbackWhileMounted 该成".to_string(),
                "拒了：RollbackError::IsolatedUnitCopyUnreadableOrMismatched".to_string(),
            );
            disagreement.note_the_unit_copy_a_rollback_refused_on(reported, true);
            disagreement
        };
        let swallowed_unit_write_on = |device: u32, first_slot: u64| {
            SwallowedWriteByteRange::of(&SwallowedWrite {
                device: DeviceIdentity(device),
                offset: DeviceOffsetInBytes(first_slot * singlefs_format::SLOT_BYTES),
                contents: SwallowedContents::Bytes(vec![
                    0xAB;
                    usize::try_from(
                        singlefs_format::SLOT_BYTES
                    )
                    .expect("槽宽装得进 usize")
                ]),
                durability: WriteDurability::Plain,
                writes_recorded_before: 146,
            })
        };
        let explained = |swallowed: &[SwallowedWriteByteRange]| WhatTheSwallowedWritesExplain {
            invariants_still_violated_if_the_swallowed_writes_had_landed: BTreeSet::new(),
            ceiling_without_the_swallowed_roots: None,
            root_ring_slots_of_the_swallowed_root_slot_writes: BTreeSet::new(),
            instance_rows_with_the_swallowed_roots_rebuilt_from_their_records: None,
            byte_ranges_of_the_swallowed_writes: swallowed.iter().copied().collect(),
        };
        let swallowed_on_the_unit = swallowed_unit_write_on(1, UNIT_SLOT);
        for aspect in [
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
            ModelDisagreementAspect::RefusalReason,
        ] {
            assert!(
                left_by_the_lying_device(
                    &observation(refusal(aspect, Some(unit_copy_on(1)))),
                    &explained(&[swallowed_on_the_unit])
                ),
                "{aspect:?}：对不上的那一份正是被吞那次写落的地方"
            );
            assert!(
                !left_by_the_lying_device(
                    &observation(refusal(aspect, Some(unit_copy_on(0)))),
                    &explained(&[swallowed_on_the_unit])
                ),
                "{aspect:?}：被吞的写在另一块盘上"
            );
            assert!(
                !left_by_the_lying_device(
                    &observation(refusal(aspect, Some(unit_copy_on(1)))),
                    &explained(&[
                        swallowed_unit_write_on(1, UNIT_SLOT - 1),
                        swallowed_unit_write_on(1, UNIT_SLOT + 1)
                    ])
                ),
                "{aspect:?}：被吞的写紧挨着那一份的前后、一个字节都不重叠"
            );
            assert!(
                !left_by_the_lying_device(
                    &observation(refusal(aspect, None)),
                    &explained(&[swallowed_on_the_unit])
                ),
                "{aspect:?}：实现没点名那一份（别的拒绝）"
            );
        }
        let mut written_before_refusing = ModelDisagreement::new(
            ModelDisagreementAspect::RefusedWhenModelRequiresSuccess,
            "RollbackWhileMounted 该成".to_string(),
            "拒了：RollbackError::IsolatedUnitCopyUnreadableOrMismatched".to_string(),
        );
        written_before_refusing
            .note_the_unit_copy_a_rollback_refused_on(Some(unit_copy_on(1)), false);
        assert_eq!(
            written_before_refusing.implementation_reported_unreadable_place, None,
            "拒之前写了盘：不记那一份"
        );
        assert!(!left_by_the_lying_device(
            &observation(written_before_refusing),
            &explained(&[swallowed_on_the_unit])
        ));
        let mut read_back = ModelDisagreement::new(
            ModelDisagreementAspect::ColdStartReadBack,
            "读回 (5,19)".to_string(),
            "读回 (5,18)".to_string(),
        );
        read_back.note_the_unit_copy_a_rollback_refused_on(Some(unit_copy_on(1)), true);
        assert_eq!(
            read_back.implementation_reported_unreadable_place, None,
            "拒绝之外的格不记那一份"
        );
        assert_eq!(
            SwallowedWriteByteRange::of(&SwallowedWrite {
                device: DeviceIdentity(0),
                offset: DeviceOffsetInBytes(8192),
                contents: SwallowedContents::Zeroes { length: 4096 },
                durability: WriteDurability::Plain,
                writes_recorded_before: 0,
            }),
            SwallowedWriteByteRange {
                device: ModelDeviceIdentity(0),
                start_in_bytes: 8192,
                end_in_bytes: 12_288,
            },
            "写零那一次照它的长度算"
        );
    }
}

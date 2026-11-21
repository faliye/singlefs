//! 施加可写挂载、崩溃恢复、掉电与系统配置槽读零之后的挂载、回退、抬 F 与冷启动。

use super::*;

/// 挂载写出的写行与暖机那几次发布，分配记录比过没有。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MountAllocationComparison {
    /// 有树的那几次发布（带文件的、有树而文件集合为空的）逐次比过：第一次比挂载之前的镜像上生效根那棵分配记录树，之后比前一次的输出。
    Compared { publishes: usize },
    /// 写行与暖机都是零单元发布（生效根那一版树表 0 条），不写分配记录，没东西可比。
    NoPublishWithFile,
    /// 挂载之前的镜像上生效根那棵分配记录树读不出（例如它是恢复施加 journal 重建出来的一版、树表单元没落盘）：没比。
    PreviousRecordsUnreadable,
}

/// 挂载写出的写行与暖机那几次（有树的）发布、推的抬 F 与续做的每一步：逐次拿上一版的分配记录比，改写或新增的记录代要等于那一次的 txg；顺带数复用。
/// 上一版：第一次是挂载之前的镜像上生效根（`MountOutput::effective_root`，可写挂载是施加前缀之后的根、回退是回退的目标根）
/// 那棵分配记录树（`recovery::allocation_records_under_root`，读盘），之后是前一次发布的输出。照代码三方攻方的改法写，
/// 被攻过零轮。
fn mount_publish_allocation_judgement(
    image_before_mount: &MemoryPool,
    mounted: &singlefs_core::mount::Mounted,
) -> (
    MountAllocationComparison,
    RecordReuse,
    Option<HarnessJudgement>,
) {
    let Ok(mut previous_records) =
        allocation_records_under_root(image_before_mount, &mounted.output.effective_root)
    else {
        return (
            MountAllocationComparison::PreviousRecordsUnreadable,
            RecordReuse::default(),
            None,
        );
    };
    let mut publishes_compared = 0;
    let mut reuse = RecordReuse::default();
    let mut harness_judgement = None;
    // 写行、暖机，与可写挂载准入不够时写行之后推的那几串抬 F 的空发布（D16（发布语义） 已定项 1「准入」那一行），按落盘的先后。
    // 写行与暖机按「有树」挑、不按「带文件」挑：有树而文件集合为空的一版（D8（核心索引结构） 已定项 16 第 5 条）照样写分配记录，
    // 跳过它们，比较的上一版就停在挂载之前那棵树上，下一次真比的发布把这几次改写的记录全当成自己改的、代却不是自己的 txg。
    let versions_of_trees_in_publish_order = std::iter::once(&mounted.output.row_publish)
        .chain(mounted.output.warm_up_publishes.iter())
        .filter_map(PoolVersion::version_of_trees)
        .chain(
            mounted
                .output
                .space_admission
                .floor_raises()
                .iter()
                .flat_map(|raised| raised.publishes.iter()),
        )
        .chain(
            floor_raises_and_versions_in_txg_order(
                &mounted.output.floor_raises_pushed_by_the_continuation,
                &mounted.output.deletion_queue_continuation,
            )
            .into_iter()
            .flat_map(|published| match published {
                FloorRaiseOrVersion::FloorRaise(raised) => raised.publishes.iter().collect(),
                FloorRaiseOrVersion::Version(version) => vec![version],
            }),
        );
    for output in versions_of_trees_in_publish_order {
        let publish_txg = output.root.checkpoint_txg;
        harness_judgement = harness_judgement.or_else(|| {
            allocation_generation_judgement(
                &previous_records,
                &output.allocation_records,
                publish_txg,
                publish_txg,
                &output.quarantined_after_release_checksum_mismatch,
            )
        });
        reuse = reuse.plus(RecordReuse::between(
            &previous_records,
            &output.allocation_records,
        ));
        previous_records.clone_from(&output.allocation_records);
        publishes_compared += 1;
    }
    let comparison = if publishes_compared == 0 {
        MountAllocationComparison::NoPublishWithFile
    } else {
        MountAllocationComparison::Compared {
            publishes: publishes_compared,
        }
    };
    (comparison, reuse, harness_judgement)
}

/// 挂载（可写挂载或回退）入口返回之后：做成的判分配代（第 1 件）、与模型比（第 2 件）、开会话；被拒的与模型比理由、写没写盘。
fn settle_mount(
    pool: &mut HistoryPool,
    mounted: Result<singlefs_core::mount::Mounted, MountError>,
    image_before_mount: &MemoryPool,
    answer: ModelAnswer,
    stream_length_before: usize,
) -> AppliedStep {
    match mounted {
        Ok(mounted) => {
            pool.copies_quarantined_in_this_step
                .extend(copies_quarantined_by_a_mount(&mounted));
            let (allocation_records_compared, reuse, harness_judgement) =
                mount_publish_allocation_judgement(image_before_mount, &mounted);
            // 暖机的零单元发布从这次挂载里写行那一版交回的那一份往下带着比（实审 B3c-4），带到最后的那一份交给会话。
            let (observed, carried_to_zero_unit_publishes) =
                observed_mount_carrying_to_zero_unit_publishes(&mounted);
            let mut verdict = judge_by_model(
                &mut pool.model,
                Ok(answer),
                &ObservedOutcome::Succeeded(observed),
            );
            // 可写挂载准入不够时写行之后推的那几串（D16（发布语义） 已定项 1「准入」那一行）：与会话推的同一个比法
            // （`judge_one_floor_raise_pushed_by_the_session`：抬到上限的照模型的抬 F 逐串比，上限卡在挂载时读坏、读回全零的槽上推的空发布模型不答），
            // 接在挂载那几条根后面；前面一处对不上就不再往下比。
            let floor_raises = mounted.output.space_admission.floor_raises();
            for raised in floor_raises {
                if verdict.disagreement.is_some() {
                    break;
                }
                let raise_verdict =
                    judge_one_floor_raise_pushed_by_the_session(&mut pool.model, raised);
                verdict.counts.add(&raise_verdict.counts);
                verdict.disagreement = raise_verdict.disagreement;
            }
            // 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）：接在推的抬 F 之后，每一步照模型的「走空的一步」比；
            // 走空里为空间推的那几串抬 F（「挂载续做被空间拒：同样推抬 F 再试」）按 txg 插在各步之间，照模型的抬 F 逐串比。
            // 推满仍放不下（`DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises`「照 D16 做成可写」）时挂载照样做成、
            // 队列留着：模型的现行那一版停在删除中，那一次被拒交给这次挂载之后会话里下一次用户改动的走空重判（会话先走空，`settle_user_change_comparing_every_version_published`）。
            for published in floor_raises_and_versions_in_txg_order(
                &mounted.output.floor_raises_pushed_by_the_continuation,
                &mounted.output.deletion_queue_continuation,
            ) {
                if verdict.disagreement.is_some() {
                    break;
                }
                let published_verdict = match published {
                    FloorRaiseOrVersion::FloorRaise(raised) => {
                        judge_one_floor_raise_pushed_by_the_session(&mut pool.model, raised)
                    }
                    FloorRaiseOrVersion::Version(continuation_step) => {
                        let step_answer = pool.model.answer_next_step_of_the_walk();
                        judge_by_model(
                            &mut pool.model,
                            step_answer,
                            &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                                roots: vec![observed_root_of_file_version(continuation_step)],
                                reported_ceiling: None,
                            }),
                        )
                    }
                };
                verdict.counts.add(&published_verdict.counts);
                verdict.disagreement = published_verdict.disagreement;
            }
            let publishes = 1
                + mounted.output.warm_up_publishes.len()
                + floor_raises
                    .iter()
                    .map(|raised| raised.publishes.len())
                    .sum::<usize>()
                + mounted.output.deletion_queue_continuation.len()
                + mounted
                    .output
                    .floor_raises_pushed_by_the_continuation
                    .iter()
                    .map(|raised| raised.publishes.len())
                    .sum::<usize>();
            // 执行器的可写挂载装 [`compaction_switches_of_the_mounts_of_the_executor`]：续做盘上那一条整理意图在发任何写之前停，
            // 上面逐版比的那一串里没有续做整理的发布。
            assert!(
                mounted
                    .output
                    .compaction_continuation
                    .run
                    .as_ref()
                    .is_none_or(|run| run.versions_published.is_empty()),
                "执行器的可写挂载叫停整理：挂载续做整理在任何写之前停"
            );
            let space_admission = MountSpaceAdmissionOutcome::of(&mounted.output.space_admission);
            let instance = mounted.output.instance;
            pool.successful_mounts += 1;
            pool.session = Some(WritableSession {
                allocator: mounted.allocator,
                current: mounted.current,
                instance,
                shadow_ledger: mounted.output.shadow_ledger,
                parameters_and_device_table: mounted.output.parameters_and_device_table,
                publishes_in_this_mount: publishes,
                carried_to_zero_unit_publishes,
                compaction: mounted.output.compaction_continuation.state_of_this_mount,
            });
            AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::Mounted {
                    instance,
                    publishes,
                    allocation_records_compared,
                    reuse,
                    space_admission,
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            }
        }
        // 挂载半路报错（写行之后的发布失败）：第 1 件不比分配代，入口没交回写出去的那几次发布；模型按「拒之前一个字节都不写」判。
        Err(error) => {
            let member = mount_error_member(&error);
            let reason = refusal_reason_of_mount_error(&error);
            let verdict = judge_by_model(
                &mut pool.model,
                Ok(answer),
                &observed_refusal(
                    member.clone(),
                    reason,
                    0,
                    stream_length_before,
                    &pool.stream,
                    None,
                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
                ),
            );
            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
        }
    }
}

pub(super) fn apply_mount_writable(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> AppliedStep {
    pool.session = None;
    pool.model.close_session();
    pool.mount_attempts += 1;
    let image_before_mount = pool.image();
    let answer = pool.model.answer_mount_writable();
    let stream_length_before = pool.stream.operation_count();
    let mounted = mount_writable_with_space_admission_and_compaction_switches(
        parameters,
        &mut pool.devices,
        pool.space_admission,
        compaction_switches_of_the_mounts_of_the_executor(),
    );
    settle_mount(
        pool,
        mounted,
        &image_before_mount,
        answer,
        stream_length_before,
    )
}

/// 盘上的一段 [起点, 起点 + 长度)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HiddenDeviceRange {
    offset_in_bytes: u64,
    length_in_bytes: u64,
}

impl HiddenDeviceRange {
    fn overlaps(self, offset_in_bytes: u64, length_in_bytes: u64) -> bool {
        self.offset_in_bytes < offset_in_bytes + length_in_bytes
            && offset_in_bytes < self.offset_in_bytes + self.length_in_bytes
    }
}

/// 一次挂载期间点名的那几段读回全 0 的设备（崩溃恢复抛弃根的造法：那几段暂时读不出）。读照样交给里面那块盘
/// （注入计划照样数到这次读，数法与不藏时相同），交回之前把落在藏起来那几段里的字节清零；写原样交下去，
/// 写到哪一段上那一段就不再藏（挂载自己写出的字节此后照常读得出）。
struct DeviceReadingZerosOverHiddenRanges<Device: singlefs_core::block_device::BlockDevice> {
    inner: Device,
    hidden_ranges: Vec<HiddenDeviceRange>,
}

impl<Device: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice
    for DeviceReadingZerosOverHiddenRanges<Device>
{
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)?;
        let buffer_length = u64::try_from(buffer.len()).expect("一次读的长度装得进 u64");
        for hidden in &self.hidden_ranges {
            if !hidden.overlaps(offset.0, buffer_length) {
                continue;
            }
            let start_in_buffer = hidden.offset_in_bytes.saturating_sub(offset.0);
            let end_in_buffer =
                (hidden.offset_in_bytes + hidden.length_in_bytes - offset.0).min(buffer_length);
            buffer[usize::try_from(start_in_buffer).expect("落在缓冲区里")
                ..usize::try_from(end_in_buffer).expect("落在缓冲区里")]
                .fill(0);
        }
        Ok(())
    }

    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        let length = u64::try_from(bytes.len()).expect("一次写的长度装得进 u64");
        self.hidden_ranges
            .retain(|hidden| !hidden.overlaps(offset.0, length));
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.hidden_ranges
            .retain(|hidden| !hidden.overlaps(offset.0, length));
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// 崩溃恢复要藏起来的那几段：`current`（这个会话的现行版本、根环里最新那条根）的根槽，与它那次发布每一条记录点名的每一份单元的
/// 第一个槽（单元头在那一槽里，读回全 0 整单元校验和就对不上，点名验证不过）。交回 (盘, 那一段)；那次发布一个单元都没点名时交回 None。
fn ranges_hidden_by_the_crash_recovery(
    parameters: &MakeFilesystemParameters,
    current: &PoolVersion,
) -> Option<Vec<(DeviceIdentity, HiddenDeviceRange)>> {
    let earlier_records = match current {
        PoolVersion::WithoutFile(version) => &version.earlier_records_of_this_publish,
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => {
            &version.earlier_records_of_this_publish
        }
    };
    let named_units: Vec<_> = earlier_records
        .iter()
        .map(|written| &written.record)
        .chain(std::iter::once(current.record()))
        .flat_map(|record| record.named.iter())
        .flat_map(|named| named.locations)
        .collect();
    if named_units.is_empty() {
        return None;
    }
    let root_slot = singlefs_core::root_ring::target_for_publish(
        current.root().checkpoint_txg,
        parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device =
        parameters.region_devices[usize::try_from(root_slot.region).expect("区域号小于 3")];
    let root_slot_offset = singlefs_core::root_ring::slot_offset(
        root_slot,
        parameters.geometry.fixed_structure_slot_spacing,
    );
    Some(
        std::iter::once((
            root_slot_device,
            HiddenDeviceRange {
                offset_in_bytes: root_slot_offset.0,
                length_in_bytes: u64::from(parameters.geometry.physical_block_size),
            },
        ))
        .chain(named_units.into_iter().map(|location| {
            (
                location.device,
                HiddenDeviceRange {
                    offset_in_bytes: location.slot.to_device_offset().0,
                    length_in_bytes: SLOT_BYTES,
                },
            )
        }))
        .collect(),
    )
}

/// 崩溃恢复抛弃根（`HistoryOperation::CrashRecoveryAbandoningTheNewestRoot`）：这个进程崩掉，下一次可写挂载时最新那条根的根槽与
/// 它那次发布点名的单元整次挂载读回全 0；挂载交回之后那几处照旧读得出。那条根是这个会话发的、系统配置见证过它，C554 乙之后
/// 读阶段重读一次仍读不出、取号之前拒可写（`MountError::NewerStateStillUnreadableAfterOneReread`），这一步因此造不出被抛弃的根。
/// 模型那一侧答必须拒（`IdealModel::answer_mount_writable_with_the_newest_root_unreadable`）；结局交给可写挂载那一处共用的
/// [`settle_mount`]：拒了就按成员映射出的理由与「录制流有没有多出写」同模型比，会话照旧关着（挂载之前就关了），
/// 后面的步与一次被拒的可写挂载之后一样按「没有可写会话」走。
pub(super) fn apply_crash_recovery_abandoning_the_newest_root(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> AppliedStep {
    let Some(session) = pool.session.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let image_before_mount = pool.image();
    let (roots, _) = ring_roots_newest_first(&image_before_mount);
    let current_root = session.current.root();
    if roots.first().copied() != Some((current_root.checkpoint_txg.0, current_root.instance.0)) {
        return AppliedStep::not_applicable(
            MissingPrecondition::NewestRingRootIsNotTheCurrentVersionOfThisSession,
        );
    }
    if roots.len() < 2 {
        return AppliedStep::not_applicable(MissingPrecondition::NoReadableRootBeforeTheNewest);
    }
    let Some(hidden) = ranges_hidden_by_the_crash_recovery(parameters, &session.current) else {
        return AppliedStep::not_applicable(MissingPrecondition::NewestPublishNamesNoUnit);
    };
    pool.session = None;
    pool.model.close_session();
    pool.mount_attempts += 1;
    let answer = match pool
        .model
        .answer_mount_writable_with_the_newest_root_unreadable()
    {
        Ok(answer) => answer,
        Err(disagreement) => {
            return AppliedStep {
                outcome: StepOutcome::NotApplicable(
                    MissingPrecondition::NoReadableRootBeforeTheNewest,
                ),
                harness_judgement: None,
                model_verdict: Some(ModelVerdict {
                    disagreement: Some(disagreement),
                    counts: ModelJudgementCounts::default(),
                }),
            }
        }
    };
    let stream_length_before = pool.stream.operation_count();
    let mut devices_with_hidden_ranges: Vec<(
        DeviceIdentity,
        DeviceReadingZerosOverHiddenRanges<HistoryDevice>,
    )> = std::mem::take(&mut pool.devices)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceReadingZerosOverHiddenRanges {
                    inner: device,
                    hidden_ranges: hidden
                        .iter()
                        .filter(|(device_of_the_range, _)| *device_of_the_range == identity)
                        .map(|(_, range)| *range)
                        .collect(),
                },
            )
        })
        .collect();
    let mounted = mount_writable_with_space_admission_and_compaction_switches(
        parameters,
        &mut devices_with_hidden_ranges,
        pool.space_admission,
        compaction_switches_of_the_mounts_of_the_executor(),
    );
    pool.devices = devices_with_hidden_ranges
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    settle_mount(
        pool,
        mounted,
        &image_before_mount,
        answer,
        stream_length_before,
    )
}

/// 几块盘此刻通没通电（[`DeviceLosingPowerAtTheFirstJournalRecordWrite`] 的几块盘共用一份）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PowerOfTheDevices {
    On,
    /// 某块盘上第一个落在 journal 环里的写到达时掉的电：那个写与之后每一个写、写零、屏障都不落盘。
    LostAtTheFirstJournalRecordWrite,
}

/// 第一个 journal 记录写到达时整池掉电的一块盘（[`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`] 的造法）：
/// 之前的写与屏障原样交给里面那块盘（录制流照常记下、落盘）；落在 journal 环里的第一个写把几块盘共用的那份电源状态翻成掉电，
/// 从它起每一个写、写零与屏障都不交下去、报 I/O 错——录制流里因此只有落了盘的那一串，与崩溃注入截在那个写之前的盘面相同。
/// 读照样交下去：掉电之后读到的就是落了盘的字节。
struct DeviceLosingPowerAtTheFirstJournalRecordWrite<
    Device: singlefs_core::block_device::BlockDevice,
> {
    inner: Device,
    /// journal 环在每块盘上的字节区间（`singlefs_format::JOURNAL_RING_START_SLOT` 起、`journal_ring_bytes` 长；与
    /// `crate::segments::FixedGeometry::classify` 认 journal 记录写的那一臂同一个区间）。
    journal_ring_offsets_in_bytes: std::ops::Range<u64>,
    power: Rc<Cell<PowerOfTheDevices>>,
}

impl<Device: singlefs_core::block_device::BlockDevice>
    DeviceLosingPowerAtTheFirstJournalRecordWrite<Device>
{
    fn error_after_the_power_loss() -> BlockDeviceError {
        BlockDeviceError::InputOutput(std::io::Error::other(
            "掉电：第一个 journal 记录写到达时整池断电，这个写与之后的写、屏障都没落盘",
        ))
    }

    fn refuse_once_the_power_is_lost(&self) -> Result<(), BlockDeviceError> {
        match self.power.get() {
            PowerOfTheDevices::On => Ok(()),
            PowerOfTheDevices::LostAtTheFirstJournalRecordWrite => {
                Err(Self::error_after_the_power_loss())
            }
        }
    }
}

impl<Device: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice
    for DeviceLosingPowerAtTheFirstJournalRecordWrite<Device>
{
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.refuse_once_the_power_is_lost()?;
        if self.journal_ring_offsets_in_bytes.contains(&offset.0) {
            self.power
                .set(PowerOfTheDevices::LostAtTheFirstJournalRecordWrite);
            return Err(Self::error_after_the_power_loss());
        }
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.refuse_once_the_power_is_lost()?;
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.refuse_once_the_power_is_lost()?;
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

/// [`HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`]：前提同覆盖写（[`apply_publish_overwrite`]），
/// 经挂着的会话发那次整份重写，盘在它的第一个 journal 记录写到达时掉电（[`DeviceLosingPowerAtTheFirstJournalRecordWrite`]）。
/// 会话的结局不判：进程已经在这次改动里崩掉，没有人收这个结局（掉电之前做完的那几步落了盘、录制流里有它们）。会话连同模型那一侧的会话
/// 一起丢掉，之后照可写挂载那一步（[`apply_mount_writable`]）挂一次、交模型判：那次改动一样都没发布，挂载恢复落到的就是改动之前那一版。
/// 模型只看盘上系统配置里的 F（[`IdealModel::settle_floor_left_in_the_system_configurations_by_a_lost_power`]）：那次改动被空间拒、先推抬 F 时，
/// 新 F 写进了系统配置、第一条根掉电没落，可写挂载的 F_生效取到它。那一格对不上时这一步报的是它，不是挂载对不上。
pub(super) fn apply_overwrite_losing_power_before_its_journal_record_then_mount_writable(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    content_choice: &ContentChoice,
    write_time_seconds: u64,
) -> AppliedStep {
    let Some(open_session) = pool.session.as_ref() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let Some(current_file_version) = open_session.current.file_version() else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    if current_file_version.inode_record.is_none() {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutTheFirstFile);
    }
    let WritableSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        compaction,
        ..
    } = pool.session.take().expect("上面判过这个进程的会话开着");
    let mut session_that_loses_power = MountedSession {
        allocator,
        current,
        instance,
        shadow_ledger,
        parameters_and_device_table,
        compaction,
    };
    let content = content_choice.bytes();
    let file = FirstFile {
        content: &content,
        write_time_seconds,
    };
    let power = Rc::new(Cell::new(PowerOfTheDevices::On));
    let journal_ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let journal_ring_offsets_in_bytes =
        journal_ring_start..journal_ring_start + parameters.geometry.journal_ring_bytes;
    let mut devices_losing_power: Vec<(
        DeviceIdentity,
        DeviceLosingPowerAtTheFirstJournalRecordWrite<HistoryDevice>,
    )> = std::mem::take(&mut pool.devices)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceLosingPowerAtTheFirstJournalRecordWrite {
                    inner: device,
                    journal_ring_offsets_in_bytes: journal_ring_offsets_in_bytes.clone(),
                    power: Rc::clone(&power),
                },
            )
        })
        .collect();
    let _outcome_nobody_receives_after_the_crash = session_that_loses_power.publish_user_change(
        &mut devices_losing_power,
        match content_choice.length.whole_file_write_entry() {
            WholeFileWriteEntry::Overwrite => UserChange::Overwrite(file),
            WholeFileWriteEntry::SequentialWrite => UserChange::SequentialWrite(file),
        },
    );
    pool.devices = devices_losing_power
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    // 这个进程到这里就没了（掉电截在第一个 journal 记录写上；那次改动在那之前就被拒的，进程同样不再往下写），
    // 之后录下的是可写挂载另起的实例写的流。
    pool.earliest_stream_index_after_each_power_loss
        .push(pool.stream.operation_count());
    let floor_read_from_the_system_configurations =
        highest_floor_in_the_self_verified_system_configuration_slots(pool, parameters);
    let floor_settled = pool
        .model
        .settle_floor_left_in_the_system_configurations_by_a_lost_power(
            floor_read_from_the_system_configurations,
        );
    pool.model.close_session();
    let mut step = apply_mount_writable(pool, parameters);
    let verdict = step.model_verdict.get_or_insert_with(ModelVerdict::default);
    match floor_settled {
        Ok(ModelFloorLeftByALostPower::KeptAtTheEffectiveFloor) => {}
        Ok(ModelFloorLeftByALostPower::RaisedToTheCeiling) => {
            verdict.counts.lost_powers_leaving_the_floor_at_the_ceiling += 1;
        }
        Err(disagreement) => verdict.disagreement = Some(disagreement),
    }
    step
}

/// 此刻盘上每块盘两槽里自证过（整槽校验和过、fsid 与本池相同）的系统配置带的 F 的最大值：恢复算 F_生效 的系统配置那一半同一个取法
/// （D16（发布语义） 已定项 1「生效」）。
fn highest_floor_in_the_self_verified_system_configuration_slots(
    pool: &HistoryPool,
    parameters: &MakeFilesystemParameters,
) -> ModelCheckpointTxg {
    let image = pool.image();
    let slot_spacing_in_bytes = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    pool.devices
        .iter()
        .flat_map(|(device, _)| {
            singlefs_core::recovery::verified_system_configuration_slots(
                &image,
                *device,
                slot_spacing_in_bytes,
                &parameters.filesystem_identifier,
            )
        })
        .map(|system_configuration| {
            ModelCheckpointTxg(system_configuration.quantities.rollback_floor.0)
        })
        .max()
        .expect("掉电设备只截 journal 环的写，系统配置槽的写整个落盘：池里至少一槽自证过")
}

/// 每块盘本池自证过（整槽校验和过、fsid 与本池相同）、世代号最大的那一槽系统配置在盘上的那一段：槽号是世代号对每盘槽数取余，
/// 与发布末尾的轮换、取号那一写取槽同一个取法（`crates/singlefs-core/src/transaction.rs` 取号写系统配置那一处）。交回 (盘, 那一段)，
/// 按 `devices` 的次序；有一块盘自证过的不到两份时交回 None（[`MissingPrecondition::SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots`]）。
fn newest_self_verified_system_configuration_slot_of_each_device(
    parameters: &MakeFilesystemParameters,
    image_before_mount: &MemoryPool,
    devices: &[DeviceIdentity],
) -> Option<Vec<(DeviceIdentity, HiddenDeviceRange)>> {
    let slot_spacing_in_bytes = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    devices
        .iter()
        .map(|device| {
            let self_verified = singlefs_core::recovery::verified_system_configuration_slots(
                image_before_mount,
                *device,
                slot_spacing_in_bytes,
                &parameters.filesystem_identifier,
            );
            if u64::try_from(self_verified.len()).expect("一块盘的系统配置槽数装得进 u64")
                < SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
            {
                return None;
            }
            let newest_slot_generation = self_verified
                .iter()
                .map(|system_configuration| system_configuration.quantities.slot_generation)
                .max()
                .expect("上面判过这块盘自证过的系统配置有两份");
            Some((
                *device,
                HiddenDeviceRange {
                    offset_in_bytes: (newest_slot_generation
                        % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                        * slot_spacing_in_bytes,
                    length_in_bytes: SYSTEM_CONFIGURATION_SLOT_BYTES,
                },
            ))
        })
        .collect()
}

/// 可写挂载期间哪几块盘自证过、世代号最大的那一槽系统配置读回全 0。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DevicesReadingTheirNewestSystemConfigurationSlotAsZeros {
    /// 每块盘（[`HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros`]）：没有一块盘两槽都自证过，
    /// 乙入判不出拒可写。
    EveryDevice,
    /// 只有按盘的次序排第一的那块盘（[`HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros`]）：
    /// 另一块盘两槽都自证过，R2 判得出，可写挂载照常往下走。
    OnlyTheFirstDevice,
}

/// 最新那一槽系统配置读回全 0 时的可写挂载（藏哪几块盘见 [`DevicesReadingTheirNewestSystemConfigurationSlotAsZeros`]）：关掉这个进程的会话，
/// 藏的那几块盘本池自证过、世代号最大的那一槽（[`newest_self_verified_system_configuration_slot_of_each_device`]）整次挂载读回全 0
/// （[`DeviceReadingZerosOverHiddenRanges`]：挂载写到它上面之后照常读得出）。读回全 0 的槽读得出而自证不过、设备不报读错。
/// 前提是每块盘两槽都自证过（藏之前），藏的那块盘因此还剩一槽自证过。
/// - 每块盘都藏：没有一块盘两槽都自证过，乙入判不出（C971（多处读坏时可写挂载写出倒挂的新实例根））并进 R2 当读错计、判据 N-配置 按真，
///   重读一次仍读回全 0，取号之前拒可写。模型照这一条答拒
///   （`IdealModel::answer_mount_writable_with_no_device_holding_two_self_verified_system_configuration_slots`）。
/// - 只藏第一块盘：另一块盘两槽都自证过，R2 与乙入判不出都判得出，读阶段的见证取自证过的槽，逐盘核与取号照「一槽自证不过、另一槽自证过」
///   往下走（D22（单元原子性怎么合成） 已定项 8 的 S-辨）。模型照一次普通可写挂载答（`IdealModel::answer_mount_writable`：模型不比系统配置
///   落在哪一槽、带的见证值）。
///
/// 结局交给与可写挂载共用的 [`settle_mount`]。
pub(super) fn apply_writable_mount_with_the_newest_system_configuration_slot_reading_zeros(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    devices_reading_zeros: DevicesReadingTheirNewestSystemConfigurationSlotAsZeros,
) -> AppliedStep {
    let image_before_mount = pool.image();
    let devices: Vec<DeviceIdentity> = pool.devices.iter().map(|(identity, _)| *identity).collect();
    let Some(mut hidden) = newest_self_verified_system_configuration_slot_of_each_device(
        parameters,
        &image_before_mount,
        &devices,
    ) else {
        return AppliedStep::not_applicable(
            MissingPrecondition::SomeDeviceWithoutTwoSelfVerifiedSystemConfigurationSlots,
        );
    };
    pool.session = None;
    pool.model.close_session();
    pool.mount_attempts += 1;
    let answer = match devices_reading_zeros {
        DevicesReadingTheirNewestSystemConfigurationSlotAsZeros::EveryDevice => pool
            .model
            .answer_mount_writable_with_no_device_holding_two_self_verified_system_configuration_slots(),
        DevicesReadingTheirNewestSystemConfigurationSlotAsZeros::OnlyTheFirstDevice => {
            hidden.truncate(1);
            pool.model.answer_mount_writable()
        }
    };
    let stream_length_before = pool.stream.operation_count();
    let mut devices_with_hidden_ranges: Vec<(
        DeviceIdentity,
        DeviceReadingZerosOverHiddenRanges<HistoryDevice>,
    )> = std::mem::take(&mut pool.devices)
        .into_iter()
        .map(|(identity, device)| {
            (
                identity,
                DeviceReadingZerosOverHiddenRanges {
                    inner: device,
                    hidden_ranges: hidden
                        .iter()
                        .filter(|(device_of_the_range, _)| *device_of_the_range == identity)
                        .map(|(_, range)| *range)
                        .collect(),
                },
            )
        })
        .collect();
    let mounted = mount_writable_with_space_admission_and_compaction_switches(
        parameters,
        &mut devices_with_hidden_ranges,
        pool.space_admission,
        compaction_switches_of_the_mounts_of_the_executor(),
    );
    pool.devices = devices_with_hidden_ranges
        .into_iter()
        .map(|(identity, device)| (identity, device.inner))
        .collect();
    settle_mount(
        pool,
        mounted,
        &image_before_mount,
        answer,
        stream_length_before,
    )
}

/// 回退那次发布（txg `rollback_txg`）前后的分配记录里，代不是这次的 txg 的那几条：复活的那几份例外——分配代写回回退目标那一版账里的
/// （D23（journal 的角色与格式） 已定项 14「复活」）。只改隔离记号的：置记号的只放过这一步交回的隔离（`copies_quarantined_in_the_step`：
/// 释放集里的与回退那次发布自己那一版的），清记号的放过（回退目标引用、现行账带着记号的那几份，D19（块指针的结构与宽度预算） 已定项 5
/// 「回退复活那一次，同一次发布清掉两块盘上的 0x4000」；执行器只在这一处放过清记号）。
pub(super) fn records_the_rollback_publish_changed_with_a_generation_outside_its_txg(
    before: &[AllocationRecord],
    after: &[AllocationRecord],
    rollback_txg: CheckpointTxg,
    resurrected_user_visible_copies: &[ResurrectedCopy],
    copies_quarantined_in_the_step: &[CopyQuarantinedAfterReleaseChecksumMismatch],
) -> Vec<AllocationRecord> {
    records_changed_with_a_generation_outside_the_publish_txgs(
        before,
        after,
        rollback_txg,
        rollback_txg,
        QuarantineMarkChangesVouchedFor {
            copies_quarantined_by_the_publishes: copies_quarantined_in_the_step,
            clearing: QuarantineMarkClearing::DoneByTheRollbackPublish,
        },
    )
    .into_iter()
    .filter(|record| {
        !resurrected_user_visible_copies.iter().any(|copy| {
            copy.device == record.device
                && copy.placement.slot == record.slot
                && copy.allocation_generation == record.generation
                && !record.is_released
        })
    })
    .collect()
}

pub(super) fn apply_rollback_while_mounted(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: RollbackTargetChoice,
) -> AppliedStep {
    let image_before_rollback = pool.image();
    let HistoryPool {
        devices,
        session,
        model,
        stream,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let WritableSession {
        allocator,
        current,
        publishes_in_this_mount,
        compaction,
        ..
    } = session;
    let PoolVersion::WithFile(current) = current else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    // 就地推进会话的现行那一版：还有别人指着它时先拷一份会话自己的（`PoolVersion` 有树的一版按引用计数共享）。
    let current = Arc::make_mut(current);
    let (roots, _) = ring_roots_newest_first(&image_before_rollback);
    let Some((newest_txg, newest_instance)) = roots.first().copied() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoReadableRootInRing);
    };
    let target = match choice {
        RollbackTargetChoice::RingRoot { index_from_newest } => {
            let root_count = u64::try_from(roots.len()).expect("根环至多几十条");
            let (txg, instance) =
                roots[usize::try_from(index_from_newest % root_count).expect("小于条数")];
            RollbackTarget {
                instance: InstanceGeneration(instance),
                checkpoint_txg: CheckpointTxg(txg),
            }
        }
        RollbackTargetChoice::BeyondNewestRoot { txg_beyond_newest } => RollbackTarget {
            instance: InstanceGeneration(newest_instance),
            checkpoint_txg: CheckpointTxg(newest_txg + 1 + txg_beyond_newest % 3),
        },
        RollbackTargetChoice::RingRootAtTheNewestFloor => {
            let floor = newest_ring_root_floor(&image_before_rollback);
            // 根环按 (txg, 实例) 从新到旧排，同一个 txg 上先碰到的就是实例最大的那条。
            let (txg, instance) = roots
                .iter()
                .copied()
                .find(|(txg, _)| Some(*txg) == floor)
                .unwrap_or((newest_txg, newest_instance));
            RollbackTarget {
                instance: InstanceGeneration(instance),
                checkpoint_txg: CheckpointTxg(txg),
            }
        }
    };
    let answer =
        model.answer_rollback_while_mounted(model_root_key(target.instance, target.checkpoint_txg));
    let records_on_disk_before = current.allocation_records.clone();
    let stream_length_before = stream.operation_count();
    // 会话内回滚：做成时清掉会话上的上一条完成的意图（`mount::roll_back_by_a_forward_publish_in_the_session`）。
    match roll_back_by_a_forward_publish_in_the_session(
        parameters, devices, allocator, current, compaction, target,
    ) {
        Ok(rolled_back) => {
            // 回退那条路交回两样：释放集里核出对不上的用户可见单元（`quarantined_user_visible_copies`），与回退那次发布换下的固定点单元里的
            // （那一版自己的 `quarantined_after_release_checksum_mismatch`）
            copies_quarantined_in_this_step
                .extend(rolled_back.quarantined_user_visible_copies.iter().copied());
            copies_quarantined_in_this_step.extend(copies_quarantined_by_the_versions([&*current]));
            *publishes_in_this_mount += 1;
            let verdict = judge_by_model(
                model,
                answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: vec![observed_root_of_file_version(current)],
                    reported_ceiling: None,
                }),
            );
            // 盘上的分配记录：改写或新增的记录代是这次的 txg，例外见 `records_the_rollback_publish_changed_with_a_generation_outside_its_txg`；
            // 这一步交回的隔离上面刚收齐（释放集里的与回退那次发布自己那一版的）。
            let rollback_txg = current.root.checkpoint_txg;
            let records = records_the_rollback_publish_changed_with_a_generation_outside_its_txg(
                &records_on_disk_before,
                &current.allocation_records,
                rollback_txg,
                &rolled_back.resurrected_user_visible_copies,
                copies_quarantined_in_this_step,
            );
            let harness_judgement = (!records.is_empty()).then_some(
                HarnessJudgement::AllocationGenerationIsNotThePublishTxg {
                    records,
                    first_publish_txg: rollback_txg,
                    last_publish_txg: rollback_txg,
                },
            );
            AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::RolledBack {
                    target,
                    released_user_visible_units: rolled_back.released_user_visible_units.len(),
                    resurrected_user_visible_copies: rolled_back
                        .resurrected_user_visible_copies
                        .len(),
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            }
        }
        Err(error) => {
            let member = rollback_error_member(&error);
            let reason = refusal_reason_of_rollback_error(&error);
            let mut verdict = judge_by_model(
                model,
                answer,
                &observed_refusal(
                    member.clone(),
                    reason,
                    0,
                    stream_length_before,
                    stream,
                    None,
                    root_ring_slot_still_bad_after_one_reread_of_rollback_error(&error),
                ),
            );
            // 拒在回退目标引用的一个单元的那一份上：把实现点名的那一份记进对不上的那一格（故障注入拿它与被吞的写落在哪比，A4）。
            if let Some(disagreement) = verdict.disagreement.as_mut() {
                disagreement.note_the_unit_copy_a_rollback_refused_on(
                    unit_copy_unreadable_or_mismatched_of_rollback_error(&error),
                    stream.operation_count() == stream_length_before,
                );
            }
            AppliedStep::judged_by_outcome_and_model(StepOutcome::Refused { member }, verdict)
        }
    }
}

pub(super) fn apply_raise_rollback_floor(
    pool: &mut HistoryPool,
    parameters: &MakeFilesystemParameters,
    choice: FloorTargetChoice,
) -> AppliedStep {
    let HistoryPool {
        devices,
        session,
        model,
        stream,
        copies_quarantined_in_this_step,
        ..
    } = pool;
    let Some(session) = session.as_mut() else {
        return AppliedStep::not_applicable(MissingPrecondition::NoWritableSession);
    };
    let WritableSession {
        allocator,
        current,
        publishes_in_this_mount,
        ..
    } = session;
    let PoolVersion::WithFile(current) = current else {
        return AppliedStep::not_applicable(MissingPrecondition::CurrentVersionWithoutFile);
    };
    // 就地推进会话的现行那一版：还有别人指着它时先拷一份会话自己的（`PoolVersion` 有树的一版按引用计数共享）。
    let current = Arc::make_mut(current);
    let current_floor = current.root.rollback_floor.0;
    let txg_before_raise = current.root.checkpoint_txg.0;
    // 前提：目标只取现行 F 及以上（出处见 `FloorTargetChoice::steps_above_current_floor` 的注释）。
    let choices = txg_before_raise.saturating_sub(current_floor) + 3;
    let new_floor = CheckpointTxg(current_floor + choice.steps_above_current_floor % choices);
    let records_before = allocator.records().to_vec();
    let records_on_disk_before = current.allocation_records.clone();
    let answer = model.answer_raise_rollback_floor(ModelCheckpointTxg(new_floor.0));
    let stream_length_before = stream.operation_count();
    let raised = raise_rollback_floor(
        parameters,
        devices,
        allocator,
        current,
        new_floor,
        ShadowLedger::On,
    );
    // 抬 F 的空发布逐次把现行版本往前推：半路报错时已经推出去的那几次也算这次挂载写出的根。
    let publishes_completed = usize::try_from(current.root.checkpoint_txg.0 - txg_before_raise)
        .expect("一次抬 F 至多推根环区域数那么多次");
    *publishes_in_this_mount += publishes_completed;
    match raised {
        Ok(raised) => {
            copies_quarantined_in_this_step
                .extend(copies_quarantined_by_the_versions(&raised.publishes));
            let verdict = judge_by_model(
                model,
                answer,
                &ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: raised
                        .publishes
                        .iter()
                        .map(observed_root_of_file_version)
                        .collect(),
                    reported_ceiling: Some(ModelCheckpointTxg(raised.ceiling.0)),
                }),
            );
            // 逐次发布比盘上的分配记录：每一次改写或新增的记录，代是那一次的 txg。
            let mut records_of_previous_publish = &records_on_disk_before;
            let mut harness_judgement = None;
            for publish in &raised.publishes {
                let publish_txg = publish.root.checkpoint_txg;
                harness_judgement = harness_judgement.or_else(|| {
                    allocation_generation_judgement(
                        records_of_previous_publish,
                        &publish.allocation_records,
                        publish_txg,
                        publish_txg,
                        &publish.quarantined_after_release_checksum_mismatch,
                    )
                });
                records_of_previous_publish = &publish.allocation_records;
            }
            AppliedStep {
                outcome: StepOutcome::Applied(AppliedEffect::RaisedFloor {
                    new_floor,
                    publishes: raised.publishes.len(),
                    reclaimed_placements: raised.reclaimed.len(),
                    reuse: RecordReuse::between(&records_before, allocator.records()),
                }),
                harness_judgement,
                model_verdict: Some(verdict),
            }
        }
        // 半路报错：推出去的那几次没有逐次的输出，现行版本是最后成功的那一次；改写或新增的记录的代要落在这几次的 txg 里。
        // 模型比理由（容量墙按做完的次数判区间）与上限。
        Err(error) => {
            let member = mount_error_member(&error);
            let reason = refusal_reason_of_mount_error(&error);
            let verdict = judge_by_model(
                model,
                answer,
                &observed_refusal(
                    member.clone(),
                    reason,
                    publishes_completed,
                    stream_length_before,
                    stream,
                    reported_ceiling_of_mount_error(&error),
                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
                ),
            );
            AppliedStep {
                outcome: StepOutcome::Refused { member },
                harness_judgement: allocation_generation_judgement(
                    &records_on_disk_before,
                    &current.allocation_records,
                    CheckpointTxg(txg_before_raise + 1),
                    current.root.checkpoint_txg,
                    &current.quarantined_after_release_checksum_mismatch,
                ),
                model_verdict: Some(verdict),
            }
        }
    }
}

pub(super) fn apply_cold_start_recover(pool: &mut HistoryPool) -> AppliedStep {
    pool.session = None;
    pool.model.close_session();
    let answer = pool.model.answer_cold_start_recover();
    let report = recover(&pool.devices, JournalPolicy::Consult);
    // 读回的根取施加 journal 记录前缀之后实际走的那条（`observed_read_back_after_a_crash`），不取施加之前所选的：
    // 两者只在「记录已持久、根槽没落」时不等，不建崩溃的历史里说谎的设备（根槽 FUA 被吞）照样造得出这个窗口
    // （实四丙交回第二节 ④ offset 330：读回的内容正是最新确认的那一版，对不上的只是根的 key）。
    let verdict = judge_by_model(
        &mut pool.model,
        Ok(answer),
        &ObservedOutcome::Succeeded(ObservedEffect::ColdStart {
            read_back: observed_read_back_after_a_crash(&report),
        }),
    );
    let read_back = match &report.outcome {
        RecoveryOutcome::NoFile { .. } => ColdStartReadBack::NoFile,
        RecoveryOutcome::FileRead { .. } => ColdStartReadBack::FileRead,
        RecoveryOutcome::Failed { .. } => ColdStartReadBack::Failed,
    };
    AppliedStep::judged_by_outcome_and_model(
        StepOutcome::Applied(AppliedEffect::Recovered {
            outcome: recovery_outcome_member(&report.outcome),
            read_back,
        }),
        verdict,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 掉电在 journal 记录之前的那次覆盖写留下实例 1 的孤儿（D18（块里携带什么信息） 已定项 11「回收」一条）：这段写死的历史
    /// （[`history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring`]）每一步都做成，每一步之后池级 checker 与模型都不红，
    /// 掉电之后那次可写挂载取到实例 2；之后在跑完的盘面上起一次有一行就扫的可写挂载，清扫删掉实例 1 那一行、抹了孤儿的头。
    /// 判别力：掉电那一截不截写（那次覆盖写整次发布成）时模型那一判先红（挂载落到的不是它答的那一版），盘上也没有孤儿、清扫一个头都不抹。
    #[test]
    fn an_overwrite_losing_power_before_its_journal_record_leaves_orphans_the_sweep_erases_once_its_instance_left_the_ring(
    ) {
        use singlefs_core::instance_table_sweep::{
            InstanceTableRowCountThreshold, InstanceTableSweepAtTheEndOfTheMount,
            InstanceTableSweepConfiguration, InstanceTableSweepOutcome, InstanceTableSweepSwitch,
            SweepPublishPlacements,
        };
        let history =
            history_leaving_orphans_of_the_first_instance_whose_roots_then_leave_the_ring();
        let execution = HistoryExecution::HANDING_EVERY_STEP_IMAGE_OUT_ON_FOUR_GIBIBYTE_DEVICES;
        let stream = SharedStream::retaining_contents();
        let mut image_after_the_last_step = None;
        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
            image_after_the_last_step = Some(observation.image.clone());
        });
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "这段历史每一步都做成、模型不红（每一步的池级 checker 归给定状态引擎两段）：{:?}",
            run.outcomes
        );
        assert!(
            matches!(
                run.outcomes.first(),
                Some(StepOutcome::Applied(AppliedEffect::Mounted {
                    instance: InstanceGeneration(2),
                    ..
                }))
            ),
            "掉电之后那次可写挂载做成、取到实例 2：{:?}",
            run.outcomes.first()
        );
        let image = image_after_the_last_step.expect("历史跑完，观察者至少被调过一次");
        let parameters = execution.device_width.parameters();
        let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = image
            .devices
            .iter()
            .map(|(identity, sectors)| {
                let mut device = SparseBlockDevice::new(
                    image.device_size_in_bytes,
                    PhysicalBlockSizeInBytes(parameters.geometry.physical_block_size),
                );
                device.image = sectors.clone();
                (*identity, device)
            })
            .collect();
        let mounted = singlefs_core::mount::mount_writable_with_instance_table_sweep_configuration(
            &parameters,
            &mut devices,
            SpaceAdmission::JudgedByTheFormula,
            ShadowLedger::On,
            InstanceTableSweepConfiguration {
                switch: InstanceTableSweepSwitch::SweptAtTheEndOfTheMountAboveTheThreshold,
                row_count_threshold: InstanceTableRowCountThreshold::ANY_ROW,
                sweep_publish_placements: SweepPublishPlacements::TakenFromTheAllocatorOfTheMount,
            },
        )
        .expect("跑完的盘面上可写挂载做得成");
        let InstanceTableSweepAtTheEndOfTheMount::Swept(InstanceTableSweepOutcome::RowsDeleted {
            rows_deleted,
            heads_erased,
        }) = &mounted.output.instance_table_sweep
        else {
            panic!("清扫删了行：{:?}", mounted.output.instance_table_sweep);
        };
        assert_eq!(
            rows_deleted
                .iter()
                .map(|row| row.instance)
                .collect::<Vec<_>>(),
            vec![InstanceGeneration(1)],
            "清扫删的是实例 1 那一行"
        );
        assert!(
            !heads_erased.is_empty(),
            "清扫抹了实例 1 的孤儿的头：{:?}",
            mounted.output.instance_table_sweep
        );
    }

    /// 每块盘最新那一槽系统配置读回全 0 时的可写挂载
    /// （[`HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros`]，起点是第一个文件之后）：
    /// 每块盘只剩一槽自证过、没有一块盘两槽都自证过，乙入判不出（C971）拒可写，拒因是「重读一次仍读不出」那一条，取号之前一个写都不发；
    /// 模型照条款答拒、对得上，这段历史跑完。
    /// 判别力（`crates/mutations.tsv` 各一行）：模型那一步照普通可写挂载答时报「模型说该成、实现拒了」，历史停在新发现上；
    /// 每块盘两槽都藏时系统配置一份都选不出、拒的不是这一条，模型判拒因对不上。
    #[test]
    fn writable_mount_with_the_newest_system_configuration_slot_of_each_device_reading_zeros_is_refused_as_the_model_answers(
    ) {
        let history = GeneratedHistory {
            seed: HistorySeed(0),
            starting_point: HistoryStartingPoint::AfterFirstFile,
            operations: vec![
                HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros,
            ],
        };
        let execution = HistoryExecution {
            per_step_checker: PerStepChecker::Skipped,
            device_width: HistoryDeviceWidth::UnitAreaOf384Slots,
            space_admission: SpaceAdmission::JudgedByTheFormula,
        };
        let stream = SharedStream::retaining_contents();
        let mut operations_after_each_step: Vec<(StepPosition, usize)> = Vec::new();
        let run = execute_history_with(&history, execution, &stream, &mut |observation| {
            operations_after_each_step.push((observation.position, stream.operation_count()));
        });
        assert_eq!(
            run.ending,
            HistoryEnding::Completed,
            "实现照乙入判不出拒、模型同判：{:?}",
            run.outcomes
        );
        assert!(
            matches!(
                run.outcomes.as_slice(),
                [StepOutcome::Refused { member }]
                    if member.contains("NewerStateStillUnreadableAfterOneReread")
                        && member.contains("PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion")
            ),
            "那一步拒可写，拒因是判据 N-配置 重读一次仍为真：{:?}",
            run.outcomes
        );
        let [(StepPosition::StartingPoint, after_the_starting_point), (StepPosition::Operation(0), after_the_mount)] =
            operations_after_each_step[..]
        else {
            panic!("起点与那一步各登记一次：{operations_after_each_step:?}");
        };
        assert_eq!(
            after_the_starting_point, after_the_mount,
            "拒在取号之前：录制流里一步不多"
        );
    }
}

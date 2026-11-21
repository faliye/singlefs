//! 发布被空间拒时推一串抬 F 腾地方再重发：抬 F 至多能放出多少、全空聚簇段合取下推不推、推完重发
//! （[`refusal_is_short_of_space`]、`MountedSession::publish_the_change_raising_the_floor_for_space`），连同推了还放不下时交回的拒绝。

use super::*;

/// 推满仍不够：最后一次被拒是 `last_refusal`（`SpaceAdmissionRefused` 或 `PlacementRefused`），已经推了 `floor_raises` 那几串
/// （每一串因为 `refusals_that_pushed_the_floor_raises` 里对应的那一次被拒才推），`stop` 是不再推的原因
/// （一次准入的发布数做满、F 已在上限、抬 F 那一串自己也取不到落点或被空间准入拒）。
#[derive(Debug)]
pub struct NoSpaceAfterRaisingTheFloor {
    pub last_refusal: PublishError,
    pub floor_raises: Vec<RaisedFloor>,
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    pub stop: FloorRaiseStop,
    /// 被拒之前发成的那几版（[`VersionsPublishedInThisChange`]）。
    pub versions_published: VersionsPublishedInThisChange,
    /// 这一次是不是被宽护卡住（[`WideProtectionAtTheCeiling`]）：F 已在上限、C 那一圈还隔离着槽时点名这一形、带上算出路要的东西
    /// （出路在调用方要的时候算，[`MountedSession::ways_out_of_the_wide_protection`]：推环那一条预演过放得下才列），
    /// 别的形是 [`WideProtectionAtTheCeiling::NotWhatStopsTheAdmission`]，照旧只是空间不够。
    pub wide_protection_at_the_ceiling: WideProtectionAtTheCeiling,
}

/// 抬 F 之前先算判出这一次不推（D16（发布语义） 已定项 10 射程「抬 F 之前先算」），两形：
/// 槽对与式子那两判（`devices_short`）是推抬 F 放得出的全放出来也放不下：每块列出的盘上界都小于需求；
/// 合取 7 那一形（`empty_cluster_segments_below_the_floor`）是这次调用还一串没推、被拒的是合取 7 判的前台写入类、写之前那一刻全空聚簇段已少于 `N₀`，不推。
/// 别的被拒成员一样带着这次调用已经落盘的：`floor_raises` 与 `refusals_that_pushed_the_floor_raises`
/// 一一对应，`versions_published` 是推的抬 F 之外发成的那几版（[`VersionsPublishedInThisChange`]）。
#[derive(Debug)]
pub struct NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
    /// 上界小于需求的盘；先判它们，判出了 `empty_cluster_segments_below_the_floor` 就是 `None`。
    pub devices_short: Vec<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease>,
    /// 没有上界小于需求的盘、合取 7 那一判在推串之前判出时：`N₀` 与全空聚簇段少于它的每一块盘（合取 7 自己报的那一份）；这时 `devices_short` 是空的。
    pub empty_cluster_segments_below_the_floor:
        Option<EmptyClusterSegmentsBelowTheFloorOnSomeDevices>,
    /// 判出这几块盘的那一次被拒（`SpaceAdmissionRefused`，合取 7 那一判判出时也可以是 `PlacementRefused`）：这一次发布没有推抬 F。
    pub last_refusal: PublishError,
    pub floor_raises: Vec<RaisedFloor>,
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    pub versions_published: VersionsPublishedInThisChange,
}

/// 一块放不下的盘：`judged_short_by` 是判它不够的那一判，`most_a_floor_raise_could_make_available` 与 `demand` 都按那一判自己的量
/// （[`JudgementThatFoundTheDeviceShort`] 各成员的文档）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
    pub device: DeviceIdentity,
    pub judged_short_by: JudgementThatFoundTheDeviceShort,
    pub most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice,
    pub demand: BytesOnOneDevice,
}

/// 为空间不够推抬 F 时抬 F 自己报了错：`cause` 是抬 F 的错原样（`mount::push_one_floor_raise_within_the_admission_budget` 的 `Err`），
/// 这一串是因为 `refusal_that_pushed_the_failed_raise` 那一次被拒才推的；之前推成的几串在 `floor_raises`（已落盘，
/// 每一串因为 `refusals_that_pushed_the_floor_raises` 里对应的那一次被拒才推）。报错那一串落盘途中失败时（`MountError::RaiseFloorSequencePublishFailed`），
/// 它前面已落盘的几次在会话的现行版本里、不在 `floor_raises` 里。
#[derive(Debug)]
pub struct FloorRaiseFailedWhilePushingForSpace {
    pub cause: MountError,
    pub refusal_that_pushed_the_failed_raise: PublishError,
    pub floor_raises: Vec<RaisedFloor>,
    pub refusals_that_pushed_the_floor_raises: Vec<PublishError>,
    /// 报错之前发成的那几版（[`VersionsPublishedInThisChange`]）。
    pub versions_published: VersionsPublishedInThisChange,
}

/// 这一次发布被拒是不是空间不够的那两种（D16（发布语义） 已定项 1「准入」那一行：准入不够，或准入放行而落点取不到）——
/// 是就推一串抬 F 再重判，不是就原样交回。
#[must_use]
pub fn refusal_is_short_of_space(refusal: &PublishError) -> bool {
    match refusal {
        PublishError::SpaceAdmissionRefused(_) | PublishError::PlacementRefused { .. } => true,
        // 合取 7（全空聚簇段少于 `N₀`）：全空段数按记录口径数（`allocator::DeviceFreeMap::empty_segments`，defer 里已释放的槽本来就不算占用），
        // 抬 F 把 defer 里的槽回收不改这个数；推串里的空发布会换下固定点单元，可能把全空段数推高，判的是被拒那一刻的数，
        // 推高之后越不越过 `N₀` 不另估，不为它推串。原样交回这个成员，调用方分得出「先整理」与「真没空间」。
        PublishError::EmptyClusterSegmentsBelowTheFloor(_)
        | PublishError::MultiLevelCodeTwoTreeRefused { .. }
        | PublishError::ReleaseNotInMapping { .. }
        | PublishError::ReleaseTargetNotAllocated { .. }
        | PublishError::ReleaseTargetAlreadyReleased { .. }
        | PublishError::ReleaseTargetLocationsOnDifferentSlots { .. }
        | PublishError::ReleaseSpanMismatch { .. }
        | PublishError::MappingEntryNarrowerThanItsFieldTable { .. }
        | PublishError::MappingEntryLocationOnADeviceOutsideThePool { .. }
        | PublishError::MappingEntryLocationsOnTheSameDevice { .. }
        | PublishError::ContentExceedsDataUnit { .. }
        | PublishError::AllocationRecordTreeRewriteSetDidNotSettle { .. }
        | PublishError::InodeTreeWriteRefused(_)
        | PublishError::FirstFileVersionDoesNotFollowTheVersionItBuildsOn { .. }
        | PublishError::FirstFileVersionOnAVersionThatAlreadyHasAFile { .. }
        | PublishError::TreeTableOfTheVersionToBuildOnUnreadable { .. }
        | PublishError::TreeIdentifierWatermarkLeavesNoRoomForTheFileVersionTrees(_)
        | PublishError::PublishFrozenAfterAWriteFailureIsNotResentYet { .. }
        // 一次发布切出来的记录多于上限：推抬 F 腾不出记录的位置，要把改动切小（代码审阅第 26 条）；txg 到顶是坏镜像（第 36 条）。
        | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
        | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        // 墓碑树、谱系树或退役树最右那片的容器号已到顶（C841）：坏镜像，推抬 F 不会让号变小。
        | PublishError::NextContainerNumberPastTheTopOfItsRange(_)
        // 写删除中的 inode、写已经删完的 inode、unlink 一个不是活记录的号：推抬 F 也不会让它复活。
        | PublishError::WriteToAnInodeBeingDeleted { .. }
        | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
        // 删完第一个文件之后给别的 inode 写数据（K-U5，第一版不支持）：推抬 F 也不会让第一个文件回来。
        | PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. }
        | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
        // `deleted_inodes` 装不下：类型化错误（D8（核心索引结构） 已定项 16 第 6 条），与空间无关。
        | PublishError::DeletedInodesQueueRefused(_)
        // 快照已到第一版的上限、树 ID 水位到顶：推抬 F 回收不出快照的名额，也抬不低水位。
        | PublishError::SnapshotCreationRefused(_)
        // 删快照点名的快照不在、正在删、已有别的在删：推抬 F 改变不了树表。deadlist 条目留存的映射条目没有、落点核不过：盘上的事，与空间无关。
        | PublishError::SnapshotDestructionRefused(_)
        | PublishError::DeadlistEntryNotReleasable { .. }
        // 改 size 的对象不是活记录、目标 size 溢出、条款没写的那一格、尾单元读不出：推抬 F 改变不了。
        | PublishError::TruncationRefused(_)
        | PublishError::BlockDevice(_)
        // 取号之前预演读错、读坏那两拒只在可写挂载取号之前那一判里造，发布路径不报它们。
        | PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_)
        | PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(_) => false,
    }
}

impl MountedSession {
    /// [`Self::publish_user_change`] 核过盘表之后那一半：先走空、再发这次改动（unlink 连同它之后的走空），被拒是空间不够就推抬 F 再判。
    /// 要出路时的预演在只进内存的盘与分配器的拷贝上走同一段（[`rehearse_the_push_then_the_remainder_of_the_change`]），
    /// 所以拆成不收 `self` 的一段。每次发这次改动的写入口上读回时只坏一份的账并进 `old_data_unit_reads_back`（发成、被拒都并）。
    #[allow(
        clippy::too_many_arguments,
        reason = "参数各是一样东西：参数、盘表、分配器、现行那一版、实例代号、影子账那一臂、这次改动、只坏一份的读回账"
    )]
    pub(super) fn publish_the_change_raising_the_floor_for_space<Device: BlockDevice>(
        parameters: &MakeFilesystemParameters,
        devices: &mut Vec<(DeviceIdentity, Device)>,
        allocator: &mut PoolAllocator,
        current: &mut PoolVersion,
        instance: InstanceGeneration,
        shadow_ledger: ShadowLedger,
        change: UserChange<'_>,
        old_data_unit_reads_back: &mut OldDataUnitReadsBackWithOneCopyFailingInThisChange,
    ) -> Result<UserChangePublished, UserChangeRefused> {
        let was_frozen_when_the_change_came_in = allocator.frozen_publish().is_some();
        let current_version_of_trees = match &*current {
            PoolVersion::WithFile(version_of_trees)
            | PoolVersion::WithTreesAndNoFile(version_of_trees) => version_of_trees,
            PoolVersion::WithoutFile(_) => {
                return Err(no_file_version_to_change(PublishedInThisChange {
                    floor_raises: Vec::new(),
                    refusals_that_pushed_the_floor_raises: Vec::new(),
                    versions_published: Vec::new(),
                }))
            }
        };
        let mut published_in_this_change = PublishedInThisChange {
            floor_raises: Vec::new(),
            refusals_that_pushed_the_floor_raises: Vec::new(),
            versions_published: Vec::new(),
        };
        // 队列空（任何一版都是常态）时一个写都不发、不复制；不空时在一份复制上走，走到哪一步（连同推的抬 F）`current` 就是哪一版。
        // 销毁中的快照（判决 S4：删快照照 unlink 的过程做，第七节岔路 4）、截断条目（判决 m4-trunc-r1 K1）与 `deleted_inodes` 一起走空。
        if !current_version_of_trees.queue_of_committed_intents_is_empty() {
            let mut walked = Arc::clone(current_version_of_trees);
            let walk = publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                parameters,
                devices,
                allocator,
                &mut walked,
                instance,
                shadow_ledger,
                RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
                published_in_this_change,
            );
            *current = PoolVersion::of_a_shared_version_of_trees(walked);
            published_in_this_change = walk?;
        }
        let mut publishes_pushed = 0;
        // 按成员分流（类型上放什么）：有树而文件集合为空的一版只接建 inode。走空把带文件的一版走成这一形时，走空的那几步与推的抬 F
        // 已落盘，带在这个拒绝里（`NoFileVersionToChange` 的三样）。
        let version_to_change = match current {
            PoolVersion::WithFile(file_version) => file_version,
            PoolVersion::WithTreesAndNoFile(emptied_version) => match change {
                UserChange::NewInodes { .. } | UserChange::DestroySnapshot { .. } => {
                    emptied_version
                }
                // 只建 inode 的那一批同建 inode；带文件写的那一批同写文件的改动。
                UserChange::DataChangesPublishedTogether(batch) if batch.whole_write.is_none() => {
                    emptied_version
                }
                UserChange::DataChangesPublishedTogether(_) => {
                    return Err(no_file_version_to_change(published_in_this_change));
                }
                // 文件集合为空时建快照放行、照常发布，读它读到空（用户 2026-10-16 定「放开」，判决 m4-segment-code-r1 第三节 F5）。
                UserChange::CreateSnapshot => emptied_version,
                // 改 size 同写文件的改动：删完之后只许建 inode、写行与空发布（D8（核心索引结构） 已定项 16 第 5 条）。
                // 按 inode 号写同写第一个文件：删空之后的一版上 inode 树一条记录都没有，没有可写的文件。这一判只罩删空那一版本身；
                // 之后建过 inode、归带文件的那几版上给删完之后才建的 inode 写数据，由写入口按 K-U5 的 β 读法拒
                // （`transaction::PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion`）。
                UserChange::Overwrite(_)
                | UserChange::SequentialWrite(_)
                | UserChange::OverwriteOfAnInode { .. }
                | UserChange::SequentialWriteOfAnInode { .. }
                | UserChange::WriteAtOffset { .. }
                | UserChange::Unlink { .. }
                | UserChange::Truncate { .. } => {
                    return Err(no_file_version_to_change(published_in_this_change));
                }
            },
            PoolVersion::WithoutFile(_) => {
                return Err(no_file_version_to_change(published_in_this_change))
            }
        };
        loop {
            let refusal = match publish_the_change_once(
                parameters,
                devices.as_mut_slice(),
                allocator,
                version_to_change,
                change,
                instance,
                old_data_unit_reads_back,
            ) {
                Ok(published) => {
                    // unlink：意图发成之后接着把全部批走完（一次调用 = 走空 + 意图 + 全部批）；别的改动之后队列本来就空，一个写都不发。
                    // 版本列表与会话的 `current` 指着同一份发成的那一版（不拷一整版）。
                    let mut published = Arc::new(published);
                    published_in_this_change
                        .versions_published
                        .push(Arc::clone(&published));
                    let walk = publish_the_queue_until_it_is_empty_raising_the_floor_for_space(
                        parameters,
                        devices,
                        allocator,
                        &mut published,
                        instance,
                        shadow_ledger,
                        RemainderOfTheUserChange::WalkTheQueue,
                        published_in_this_change,
                    );
                    // 这次改动已经发成，冻结的是它之后的删除步：重发成了这次改动的那份账照记。
                    if walk.is_err() {
                        note_the_kind_of_a_publish_frozen_by_this_call(
                            allocator,
                            was_frozen_when_the_change_came_in,
                            KindOfTheFrozenPublish::UserChange,
                        );
                    }
                    *current = PoolVersion::of_a_shared_version_of_trees(published);
                    let PublishedInThisChange {
                        floor_raises,
                        refusals_that_pushed_the_floor_raises,
                        versions_published,
                    } = walk?;
                    return Ok(UserChangePublished {
                        floor_raises,
                        refusals_that_pushed_the_floor_raises,
                        versions_published,
                        discards_after_the_change: DiscardsAfterThePublishes::default(),
                        // 另发的那一批由 `publish_user_change` 在这一段返回之后发（预演不发它），发了再换。
                        relocation_after_the_change_of_the_units_read_with_one_copy_failing:
                            RelocationAfterTheChangeOfTheUnitsReadWithOneCopyFailing::NotNeeded,
                    });
                }
                Err(refusal) => {
                    note_the_kind_of_a_publish_frozen_by_this_call(
                        allocator,
                        was_frozen_when_the_change_came_in,
                        KindOfTheFrozenPublish::UserChange,
                    );
                    refusal
                }
            };
            let publishes_of_this_raise;
            (published_in_this_change, publishes_of_this_raise) =
                push_one_floor_raise_for_a_refusal_for_space(
                    parameters,
                    devices,
                    allocator,
                    version_to_change,
                    shadow_ledger,
                    RemainderOfTheUserChange::WalkTheQueueThenPublishTheChange(change),
                    RefusedPublishUnderTheEmptyClusterSegmentsConjunct::of_the_user_change(change),
                    publishes_pushed,
                    published_in_this_change,
                    refusal,
                )?;
            publishes_pushed += publishes_of_this_raise;
        }
    }
}

/// 一块盘上抬 F 至多放得开的槽，都是分配器增量维护的数（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」：准入路径上不扫单元区）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SlotsAFloorRaiseCouldRelease {
    /// defer 窗口里全部已释放的槽：只算槽数、不问这一串的上限够不够得到它们。
    deferred: u64,
    /// 隔离着的槽（式子第九项）：被抛弃根离开根环、C 那一圈解开时回来，推环这条出路也靠它们（[`StuckBehindTheWideProtection`]）。
    isolated: u64,
    /// 这次挂载开过的聚簇段之外、两槽都不罩在未释放分配记录下的偶数起点槽对：defer、影子账隔离、抬 F 扣住三样全放开之后数据单元落得下的槽对，
    /// 现有没挡的槽对也在里面（`DeviceFreeMap::slot_pairs_without_an_unreleased_record_outside_the_cluster_segments`，分配器按段增量维护）。
    /// defer 里一个数据单元的两槽放开只成一对，另一半罩在未释放记录下的槽放开了也不成对。
    slot_pairs_once_everything_is_released: SlotPairCount,
}

impl SlotsAFloorRaiseCouldRelease {
    fn of_the_device(device_map: &DeviceFreeMap, cluster_segments: &BTreeSet<SlotNumber>) -> Self {
        Self {
            deferred: device_map.deferred_slots(),
            isolated: device_map.isolated_slots(),
            slot_pairs_once_everything_is_released: device_map
                .slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
                    cluster_segments,
                ),
        }
    }
}

/// 一块被判短的盘推抬 F 至多凑得出多少，按判它短的那一判自己的量（宁大勿小：上界算小了会误拒一个推了本来写得进的写）：
/// 式子那一判是可用加 defer 与隔离的字节；段外槽对那一判是三样全放开之后段外成的槽对（现有的在里面，不再另加可用），按一个数据单元的槽数换算。
/// 提交内生块那一判交 `None`：这一判数的槽抬 F 那一串自己也要占，不在这里算。
fn most_a_floor_raise_could_make_available(
    short: &DeviceShortOfDemand,
    slots: SlotsAFloorRaiseCouldRelease,
) -> Option<AvailableBytesOnOneDevice> {
    let bytes_of_slots = |count: u64| i128::from(BytesOnOneDevice::of_slots(count).0);
    match short.judged_short_by {
        JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand => {
            Some(AvailableBytesOnOneDevice(
                short.available.0 + bytes_of_slots(slots.deferred) + bytes_of_slots(slots.isolated),
            ))
        }
        JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits => {
            let slots_of_a_data_unit =
                TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots();
            Some(AvailableBytesOnOneDevice(bytes_of_slots(
                slots.slot_pairs_once_everything_is_released.0 * slots_of_a_data_unit,
            )))
        }
        JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits => None,
    }
}

/// 这块被判短的盘推抬 F 全放出来也放不下时交回它的上界与需求；上界恰好等于需求算放得下，判不准的那一判（[`most_a_floor_raise_could_make_available`] 交 `None`）也算放得下。
fn device_short_even_with_everything_a_floor_raise_could_release(
    short: &DeviceShortOfDemand,
    slots: SlotsAFloorRaiseCouldRelease,
) -> Option<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease> {
    let upper_bound = most_a_floor_raise_could_make_available(short, slots)?;
    (upper_bound.0 < i128::from(short.demand.0)).then_some(
        DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
            device: short.device,
            judged_short_by: short.judged_short_by,
            most_a_floor_raise_could_make_available: upper_bound,
            demand: short.demand,
        },
    )
}

/// 这一次被拒里全放出来也放不下的盘，按拒绝里的次序。只读分配器（内存）。只有 `SpaceAdmissionRefused` 判得出需求，`PlacementRefused` 交空的：这一判不拦它，接着判合取 7 那一形（[`empty_cluster_segments_below_the_floor_before_any_series`]），没判出才推。
fn devices_short_even_with_everything_a_floor_raise_could_release(
    allocator: &PoolAllocator,
    refusal: &PublishError,
) -> Vec<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease> {
    let PublishError::SpaceAdmissionRefused(refused) = refusal else {
        return Vec::new();
    };
    refused
        .short_devices
        .iter()
        .filter_map(|short| {
            let device_map = allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == short.device)
                .expect("拒绝里的盘来自这个分配器：准入的两份读数都按它的盘建");
            device_short_even_with_everything_a_floor_raise_could_release(
                short,
                SlotsAFloorRaiseCouldRelease::of_the_device(
                    device_map,
                    allocator.cluster_segments(),
                ),
            )
        })
        .collect()
}

/// 被拒的那一次发布是不是合取 7（`全空聚簇段数 ≥ N₀`，D3（空间分配） 已定项 7 登记表第 7 行）一定判的前台写入类：抬 F 之前先算的合取 7 那一判
/// （[`empty_cluster_segments_below_the_floor_before_any_series`]）只对这一种判。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RefusedPublishUnderTheEmptyClusterSegmentsConjunct {
    /// 写文件内容、建 inode、并进一次发布的数据写类：带普通分配的前台写入类，合取 7 判它（`transaction::reach_of_the_empty_cluster_segments_conjunct`）。
    ForegroundWriteTheConjunctJudges,
    /// 删除类（unlink、删快照的意图，走空里的每一步）免合取 7；改 size 发之前分不出是截断的意图（免）还是变长与只改记录（判），
    /// 建快照带不带普通分配发之前不知道：这几种都不先判，照推。
    ExemptOrNotKnownBeforeThePublish,
}

impl RefusedPublishUnderTheEmptyClusterSegmentsConjunct {
    /// 会话发这一个改动、被拒的就是这个改动自己那一次发布时（不是先走空里的一步）。
    pub(super) fn of_the_user_change(change: UserChange<'_>) -> Self {
        match change {
            UserChange::Overwrite(_)
            | UserChange::SequentialWrite(_)
            | UserChange::NewInodes { .. }
            | UserChange::OverwriteOfAnInode { .. }
            | UserChange::SequentialWriteOfAnInode { .. }
            | UserChange::WriteAtOffset { .. }
            | UserChange::DataChangesPublishedTogether(_) => Self::ForegroundWriteTheConjunctJudges,
            UserChange::Unlink { .. }
            | UserChange::DestroySnapshot { .. }
            | UserChange::Truncate { .. }
            | UserChange::CreateSnapshot => Self::ExemptOrNotKnownBeforeThePublish,
        }
    }
}

/// 抬 F 之前先算的合取 7 那一判（D16（发布语义） 已定项 10 射程「抬 F 之前先算」）：这次调用还一串都没推（`series_pushed_in_this_change` 是 0）、
/// 被拒是空间不够的那两种（[`refusal_is_short_of_space`]）、被拒的是合取 7 判的前台写入类、准入判着时，读每块盘此刻的全空聚簇段数
/// （`DeviceFreeMap::empty_segments`，合取 7 自己读的那一个）与整理开关里的 `N₀`，照合取 7 同一个判法
/// （[`admit_the_empty_cluster_segments_floor_on_every_device`]）判，有一块少于 `N₀` 就交回那几块盘。判的是写之前那一刻的数：
/// 抬 F 回收 defer 里的槽不改这个数（defer 里的已释放槽本来就不算占用）；推串里的空发布会换下固定点单元，可能把全空段数推高，
/// 推高之后越不越过 `N₀` 不另估。只读分配器（内存）。
fn empty_cluster_segments_below_the_floor_before_any_series(
    allocator: &PoolAllocator,
    refusal: &PublishError,
    refused_publish: RefusedPublishUnderTheEmptyClusterSegmentsConjunct,
    series_pushed_in_this_change: usize,
) -> Option<EmptyClusterSegmentsBelowTheFloorOnSomeDevices> {
    let is_judged_by_the_conjunct = match refused_publish {
        RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges => {
            true
        }
        RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish => {
            false
        }
    };
    let is_admission_judged = match allocator.space_admission() {
        SpaceAdmission::JudgedByTheFormula => true,
        SpaceAdmission::SkippedByTheTestOnlySwitch => false,
    };
    if series_pushed_in_this_change > 0
        || !refusal_is_short_of_space(refusal)
        || !is_judged_by_the_conjunct
        || !is_admission_judged
    {
        return None;
    }
    let empty_cluster_segments_on_each_device: Vec<(DeviceIdentity, u64)> = allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.empty_segments()))
        .collect();
    admit_the_empty_cluster_segments_floor_on_every_device(
        &empty_cluster_segments_on_each_device,
        allocator
            .compaction()
            .switches
            .empty_cluster_segments_floor
            .segments(),
    )
    .err()
}

/// 抬 F 之前先算（D16（发布语义） 已定项 10 射程「抬 F 之前先算」）：推串的三处（[`MountedSession::publish_user_change`]、
/// [`MountedSessionWithPendingChanges::accept`]、走空）都经 [`push_one_floor_raise_for_a_refusal_for_space`] 在推每一串之前走这一段。
/// 推抬 F 放得出的全放出来也放不下的盘（[`devices_short_even_with_everything_a_floor_raise_could_release`]）先判，判出就交回它们；
/// 没判出、这次调用还一串没推而全空聚簇段已少于 `N₀`、被拒的又是合取 7 判的前台写入（[`empty_cluster_segments_below_the_floor_before_any_series`]）的，
/// 交回那几块盘的全空段数。两种都是 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，带上这次调用已经落盘的；
/// 都没判出原样交回已落盘的与这一次被拒，由调用方接着推。只读分配器（内存），一个盘都不读、一个写都不发。
fn judge_before_pushing_a_floor_raise_for_a_refusal(
    allocator: &PoolAllocator,
    refused_publish: RefusedPublishUnderTheEmptyClusterSegmentsConjunct,
    published_in_this_change: PublishedInThisChange,
    refusal: PublishError,
) -> Result<(PublishedInThisChange, PublishError), UserChangeRefused> {
    let devices_short =
        devices_short_even_with_everything_a_floor_raise_could_release(allocator, &refusal);
    if !devices_short.is_empty() {
        return Err(no_space_even_with_everything_a_floor_raise_could_release(
            devices_short,
            None,
            refusal,
            published_in_this_change,
        ));
    }
    if let Some(below_the_floor) = empty_cluster_segments_below_the_floor_before_any_series(
        allocator,
        &refusal,
        refused_publish,
        published_in_this_change.floor_raises.len(),
    ) {
        return Err(no_space_even_with_everything_a_floor_raise_could_release(
            Vec::new(),
            Some(below_the_floor),
            refusal,
            published_in_this_change,
        ));
    }
    Ok((published_in_this_change, refusal))
}

/// 全放出来也放不下的那个成员，把这次调用已经落盘的都带上。判出它的是放不下的盘（`devices_short` 非空、`empty_cluster_segments_below_the_floor` 是 `None`）
/// 或推串之前合取 7 那一判（`devices_short` 空、`empty_cluster_segments_below_the_floor` 是 `Some`）。
fn no_space_even_with_everything_a_floor_raise_could_release(
    devices_short: Vec<DeviceShortEvenWithEverythingAFloorRaiseCouldRelease>,
    empty_cluster_segments_below_the_floor: Option<EmptyClusterSegmentsBelowTheFloorOnSomeDevices>,
    last_refusal: PublishError,
    published_in_this_change: PublishedInThisChange,
) -> UserChangeRefused {
    let PublishedInThisChange {
        floor_raises,
        refusals_that_pushed_the_floor_raises,
        versions_published,
    } = published_in_this_change;
    UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(Box::new(
        NoSpaceEvenWithEverythingAFloorRaiseCouldRelease {
            devices_short,
            empty_cluster_segments_below_the_floor,
            last_refusal,
            floor_raises,
            refusals_that_pushed_the_floor_raises,
            versions_published,
        },
    ))
}

/// 一次发布被拒之后会话怎么接（D16（发布语义） 已定项 1「准入」那一行）：不是空间不够的那两种（[`refusal_is_short_of_space`]）原样交回 `Publish`；
/// 抬 F 之前先算判出放不下（[`judge_before_pushing_a_floor_raise_for_a_refusal`]，`refused_publish` 说被拒的那一次归不归合取 7 管）交回
/// `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，一个写都不发；
/// 别的就为被拒的那一次发布推一串抬 F 的空发布（这次准入已推 `publishes_pushed_in_this_admission` 次，预算照
/// `mount::push_one_floor_raise_within_the_admission_budget`），推成交回记上这一串之后的 `published_in_this_change` 与这一串推了几次空发布，
/// 不再推交回 `NoSpaceAfterRaisingTheFloor`（报不报「被宽护卡住」，报的话记下这次改动剩下的 `remainder`，要出路时照它预演，
/// [`wide_protection_at_the_ceiling`]），抬 F 自己报错交回 `FloorRaiseFailedWhilePushingForSpace`。
/// 推的那一串就地推进 `current`（被拒的那一次要接在它后面的那一版）：`current` 还有别人指着（版本列表里那一份）时先拷一份再推
/// （[`Arc::make_mut`]），版本列表里那一份不动。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、影子账那一臂、这次改动还剩什么、被拒的那一次归不归合取 7 管、预算已用的数、已落盘的、这一次被拒，不成一个概念"
)]
pub(super) fn push_one_floor_raise_for_a_refusal_for_space<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut Arc<TransactionOutput>,
    shadow_ledger: ShadowLedger,
    remainder: RemainderOfTheUserChange<'_>,
    refused_publish: RefusedPublishUnderTheEmptyClusterSegmentsConjunct,
    publishes_pushed_in_this_admission: usize,
    published_in_this_change: PublishedInThisChange,
    refusal: PublishError,
) -> Result<(PublishedInThisChange, usize), UserChangeRefused> {
    let (published_in_this_change, refusal) = judge_before_pushing_a_floor_raise_for_a_refusal(
        allocator,
        refused_publish,
        published_in_this_change,
        refusal,
    )?;
    let PublishedInThisChange {
        mut floor_raises,
        mut refusals_that_pushed_the_floor_raises,
        versions_published,
    } = published_in_this_change;
    if !refusal_is_short_of_space(&refusal) {
        return Err(UserChangeRefused::Publish {
            cause: refusal,
            floor_raises,
            versions_published,
        });
    }
    match push_one_floor_raise_within_the_admission_budget(
        parameters,
        devices,
        allocator,
        Arc::make_mut(current),
        shadow_ledger,
        publishes_pushed_in_this_admission,
    ) {
        Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
            let publishes_of_this_raise = raised.publishes.len();
            floor_raises.push(raised);
            refusals_that_pushed_the_floor_raises.push(refusal);
            Ok((
                PublishedInThisChange {
                    floor_raises,
                    refusals_that_pushed_the_floor_raises,
                    versions_published,
                },
                publishes_of_this_raise,
            ))
        }
        Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => Err(
            UserChangeRefused::NoSpaceAfterRaisingTheFloor(Box::new(NoSpaceAfterRaisingTheFloor {
                last_refusal: refusal,
                floor_raises,
                refusals_that_pushed_the_floor_raises,
                // 先借 `stop` 判这一形，再把它挪进下一个字段（字段按写的次序求值）。
                wide_protection_at_the_ceiling: wide_protection_at_the_ceiling(
                    allocator, current, remainder, &stop,
                ),
                stop,
                versions_published,
            })),
        ),
        Err(cause) => Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(
            Box::new(FloorRaiseFailedWhilePushingForSpace {
                cause,
                refusal_that_pushed_the_failed_raise: refusal,
                floor_raises,
                refusals_that_pushed_the_floor_raises,
                versions_published,
            }),
        )),
    }
}

/// 把 `current` 的 `deleted_inodes` 走空（[`publish_the_queue_until_it_is_empty`]），走空里的删除类发布被拒是空间不够时
/// 推一串抬 F、回收之后接着走（用户 2026-10-02 定案 Q-V「推抬 F 再试」：删除类发布不判式子那一判，落得下那一判照判，
/// 被它以空间拒时推抬 F 腾得出槽）；推满仍放不下、抬 F 自己报错、被拒不是空间不够，交回会话要报的那一种拒绝。
/// 每一次被拒的删除类发布各算一次准入（与 unlink 意图那次、别的用户改动那次一样，预算数的是为被判的那一次发布推了几次空发布）：
/// 这一趟走空有一步落了盘，下一次被拒的就是后面那一次发布，这次准入已推的数从 0 数起。`current` 停在最后一次落盘的那一版上
/// （删除那一步或推的空发布）；发成的每一步记进 `versions_published`。`remainder` 是走空之后这次改动还剩什么（推满仍不够、被宽护卡住时
/// 记进报里，要出路时预演推环之后照它再发一次，[`wide_protection_at_the_ceiling`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各有名字与类型：写入口要的参数与盘表、分配器、现行那一版、实例代号、影子账那一臂、这次改动还剩什么、已落盘的，不成一个概念"
)]
pub(super) fn publish_the_queue_until_it_is_empty_raising_the_floor_for_space<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut Arc<TransactionOutput>,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    remainder: RemainderOfTheUserChange<'_>,
    published_in_this_change: PublishedInThisChange,
) -> Result<PublishedInThisChange, UserChangeRefused> {
    let mut published_in_this_change = published_in_this_change;
    let mut publishes_pushed_in_this_admission = 0;
    // 迭代上界：每一轮要么走空交回，要么交回拒绝，要么推成一串（推的数加进这次准入，同一次发布的准入在预算 B − 1 次空发布之内停），
    // 要么这一趟有一步落了盘（队首那个文件少了单元或删掉了，队列与单元数都有限）。跨轮携带的是 `current` 与已落盘的。
    loop {
        let DeletionQueueWalk {
            steps_published,
            stopped_by,
        } = publish_the_queue_until_it_is_empty(
            parameters,
            devices.as_mut_slice(),
            allocator,
            current,
            instance,
        );
        if !steps_published.is_empty() {
            publishes_pushed_in_this_admission = 0;
        }
        published_in_this_change
            .versions_published
            .extend(steps_published);
        let refusal = match stopped_by {
            Ok(()) => return Ok(published_in_this_change),
            Err(refusal) => refusal,
        };
        let publishes_of_this_raise;
        (published_in_this_change, publishes_of_this_raise) =
            push_one_floor_raise_for_a_refusal_for_space(
                parameters,
                devices,
                allocator,
                current,
                shadow_ledger,
                remainder,
                // 走空里被拒的是删除类的一步，合取 7 免。
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish,
                publishes_pushed_in_this_admission,
                published_in_this_change,
                refusal,
            )?;
        publishes_pushed_in_this_admission += publishes_of_this_raise;
    }
}

#[cfg(test)]
mod everything_a_floor_raise_could_release_tests {
    use super::*;
    use crate::admission::{
        admit_the_units_landing_on_every_device, PlacementRoomOnOneDevice, UnitsOfAPublishToLand,
    };
    use crate::allocator::ReclaimedReuse;
    use crate::mount::FloorOfThePushedPublishes;
    use crate::write_accounting::WritesByStructureKind;
    use singlefs_format::{SLOT_BYTES, UNIT_AREA_START_SLOT};

    const DEVICE_OF_THE_TEST: DeviceIdentity = DeviceIdentity(0);

    fn no_slots() -> SlotsAFloorRaiseCouldRelease {
        SlotsAFloorRaiseCouldRelease {
            deferred: 0,
            isolated: 0,
            slot_pairs_once_everything_is_released: SlotPairCount(0),
        }
    }

    fn short_by(
        judged_short_by: JudgementThatFoundTheDeviceShort,
        available_in_bytes: i128,
        demand_in_bytes: u64,
    ) -> DeviceShortOfDemand {
        DeviceShortOfDemand {
            device: DEVICE_OF_THE_TEST,
            judged_short_by,
            available: AvailableBytesOnOneDevice(available_in_bytes),
            demand: BytesOnOneDevice(demand_in_bytes),
        }
    }

    /// 式子那一判：可用是负的、defer 两槽、隔离一槽。需求恰好等于三样之和。
    const FORMULA_AVAILABLE_IN_BYTES: i128 = -100;

    fn formula_demand_equal_to_the_bound() -> u64 {
        3 * SLOT_BYTES - 100
    }

    fn formula_slots() -> SlotsAFloorRaiseCouldRelease {
        SlotsAFloorRaiseCouldRelease {
            deferred: 2,
            isolated: 1,
            slot_pairs_once_everything_is_released: SlotPairCount(0),
        }
    }

    #[test]
    fn formula_demand_exactly_covered_by_available_deferred_and_isolated_bytes_is_still_pushed_for()
    {
        assert_eq!(
            device_short_even_with_everything_a_floor_raise_could_release(
                &short_by(
                    JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
                    FORMULA_AVAILABLE_IN_BYTES,
                    formula_demand_equal_to_the_bound(),
                ),
                formula_slots(),
            ),
            None,
            "上界恰好等于需求：放得下，照推"
        );
    }

    #[test]
    fn formula_demand_one_byte_above_the_bound_is_named_with_the_bound_and_the_demand() {
        assert_eq!(
            device_short_even_with_everything_a_floor_raise_could_release(
                &short_by(
                    JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
                    FORMULA_AVAILABLE_IN_BYTES,
                    formula_demand_equal_to_the_bound() + 1,
                ),
                formula_slots(),
            ),
            Some(DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
                device: DEVICE_OF_THE_TEST,
                judged_short_by: JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
                most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(i128::from(
                    formula_demand_equal_to_the_bound()
                )),
                demand: BytesOnOneDevice(formula_demand_equal_to_the_bound() + 1),
            }),
            "差一个字节：点名这块盘，带上界与需求"
        );
    }

    #[test]
    fn the_formula_bound_does_not_count_slots_the_formula_never_deducted() {
        assert!(
            device_short_even_with_everything_a_floor_raise_could_release(
                &short_by(
                    JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
                    0,
                    1,
                ),
                SlotsAFloorRaiseCouldRelease {
                    deferred: 0,
                    isolated: 0,
                    slot_pairs_once_everything_is_released: SlotPairCount(8),
                },
            )
            .is_some(),
            "段外放开之后成的槽对是落得下那一判的量，式子不扣它：加进上界就多算"
        );
    }

    /// 单元区恰好一个 64 槽聚簇段的一块盘：32 个槽对。
    fn allocator_of_one_cluster_segment() -> PoolAllocator {
        PoolAllocator::new(vec![DeviceFreeMap::new(
            DEVICE_OF_THE_TEST,
            (UNIT_AREA_START_SLOT + CLUSTER_SEGMENT_SLOTS) * SLOT_BYTES,
        )])
    }

    /// 把这块盘的段外槽对分完（数据单元落点按次序取最低的偶数槽对），交回分到的落点，按槽号。
    fn allocate_every_slot_pair(allocator: &mut PoolAllocator) -> Vec<Placement> {
        // 迭代上界：每一轮分走一个槽对，单元区只有 32 对。
        let mut placements = Vec::new();
        while let Some(placement) = allocator.allocate_user_data(CheckpointTxg(1)) {
            placements.push(placement);
        }
        assert_eq!(placements.len(), 32, "一段 64 槽分成 32 个数据单元");
        placements
    }

    /// 落得下那一判（`admit_the_units_landing_on_every_device`，发布路径判的同一个函数）对这块盘此刻的分配器判 `data_units` 个数据单元、
    /// `commit_generated_slots` 个提交内生块槽，判不过交回发布路径交回的那一种被拒。
    fn refusal_of_the_landing_judgement(
        allocator: &PoolAllocator,
        data_units: u64,
        commit_generated_slots: u64,
    ) -> PublishError {
        PublishError::SpaceAdmissionRefused(
            admit_the_units_landing_on_every_device(
                &PlacementRoomOnOneDevice::of_every_device_of_the_allocator(allocator),
                UnitsOfAPublishToLand {
                    data_units,
                    commit_generated_slots,
                },
            )
            .expect_err("用例造的盘落不下这么多"),
        )
    }

    fn bytes_of_slot_pairs(pairs: u64) -> u64 {
        BytesOnOneDevice::of_slots(
            pairs * TransactionUnit::Data(DataUnitIndexInFile::FIRST).span_slots(),
        )
        .0
    }

    fn nothing_published_in_this_change() -> PublishedInThisChange {
        PublishedInThisChange {
            floor_raises: Vec::new(),
            refusals_that_pushed_the_floor_raises: Vec::new(),
            versions_published: Vec::new(),
        }
    }

    /// 段外槽对分完、一个数据单元（两槽）进了 defer：放开它只成一对。写两个数据单元，推了也只多一对，先算判放不下、一串都不推。
    /// 判别力：上界照每个放得开的槽各补一对算（defer 两槽算两对），恰好等于需求，照推。
    #[test]
    fn a_deferred_data_unit_completes_one_slot_pair_so_a_write_of_two_data_units_is_not_pushed_for()
    {
        let mut allocator = allocator_of_one_cluster_segment();
        let placements = allocate_every_slot_pair(&mut allocator);
        allocator.release(placements[7], CheckpointTxg(2));
        let refusal = refusal_of_the_landing_judgement(&allocator, 2, 0);
        let Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space)) =
            judge_before_pushing_a_floor_raise_for_a_refusal(
                &allocator,
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish,
                nothing_published_in_this_change(),
                refusal,
            )
        else {
            panic!("defer 里一个数据单元放开只成一对，两个数据单元推了也放不下：不推");
        };
        assert_eq!(
            (
                no_space.devices_short,
                no_space.empty_cluster_segments_below_the_floor.is_none()
            ),
            (
                vec![DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
                    device: DEVICE_OF_THE_TEST,
                    judged_short_by:
                        JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
                    most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(i128::from(
                        bytes_of_slot_pairs(1)
                    )),
                    demand: BytesOnOneDevice(bytes_of_slot_pairs(2)),
                }],
                true
            ),
            "点名这块盘：上界一对、需求两对，判出它的是槽对那一判"
        );
    }

    /// 段外槽对分完之后，一个数据单元隔离着（回收之后被影子账隔离）、一个抬 F 扣住、一个在 defer 里：三样全放开成三对。
    fn allocator_whose_slot_pairs_are_blocked_once_by_each_of_the_three() -> PoolAllocator {
        let mut allocator = allocator_of_one_cluster_segment();
        let placements = allocate_every_slot_pair(&mut allocator);
        allocator.release(placements[3], CheckpointTxg(2));
        allocator.reclaim_released_up_to(CheckpointTxg(2), ReclaimedReuse::Immediately);
        allocator.isolate_abandoned(DEVICE_OF_THE_TEST, placements[3].slot, placements[3].span);
        allocator.release(placements[11], CheckpointTxg(3));
        allocator
            .reclaim_released_up_to(CheckpointTxg(3), ReclaimedReuse::HeldUntilFloorTakesEffect);
        allocator.release(placements[20], CheckpointTxg(4));
        assert_eq!(
            allocator.devices[0]
                .unblocked_slot_pairs_outside_the_cluster_segments(allocator.cluster_segments()),
            SlotPairCount(0),
            "三对都挡着：落得下那一判此刻数到 0 对"
        );
        allocator
    }

    /// 上界恰好等于需求算放得下：三样各挡一对、写三个数据单元，照推。
    /// 判别力：上界只数此刻没挡的槽对（0 对），或漏数 defer、隔离、扣住任一样，都判放不下。
    #[test]
    fn slot_pairs_blocked_only_by_the_defer_queue_the_isolation_and_the_floor_hold_are_pushed_for()
    {
        let allocator = allocator_whose_slot_pairs_are_blocked_once_by_each_of_the_three();
        let devices_short = devices_short_even_with_everything_a_floor_raise_could_release(
            &allocator,
            &refusal_of_the_landing_judgement(&allocator, 3, 0),
        );
        assert!(
            devices_short.is_empty(),
            "三样全放开成三对，恰好够三个数据单元：照推，实际点名 {devices_short:?}"
        );
    }

    /// 同上，写四个数据单元：三对不够，点名这块盘，上界是三对。
    #[test]
    fn slot_pairs_blocked_by_the_three_fall_one_pair_short_of_four_data_units() {
        let allocator = allocator_whose_slot_pairs_are_blocked_once_by_each_of_the_three();
        assert_eq!(
            devices_short_even_with_everything_a_floor_raise_could_release(
                &allocator,
                &refusal_of_the_landing_judgement(&allocator, 4, 0),
            ),
            vec![DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
                device: DEVICE_OF_THE_TEST,
                judged_short_by:
                    JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
                most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(i128::from(
                    bytes_of_slot_pairs(3)
                )),
                demand: BytesOnOneDevice(bytes_of_slot_pairs(4)),
            }],
            "上界三对、需求四对"
        );
    }

    /// 此刻没挡的槽对已经在三样全放开之后的槽对里：段外只剩一对空着、别的都罩在未释放记录下，写两个数据单元，上界一对，不再另加此刻的可用。
    /// 判别力：上界再加一遍落得下那一判报的可用（一对），恰好等于需求，照推。
    #[test]
    fn slot_pairs_free_right_now_are_counted_once_in_the_bound() {
        let mut allocator = allocator_of_one_cluster_segment();
        let placements = allocate_every_slot_pair(&mut allocator);
        allocator.release(placements[5], CheckpointTxg(2));
        allocator.reclaim_released_up_to(CheckpointTxg(2), ReclaimedReuse::Immediately);
        assert_eq!(
            devices_short_even_with_everything_a_floor_raise_could_release(
                &allocator,
                &refusal_of_the_landing_judgement(&allocator, 2, 0),
            ),
            vec![DeviceShortEvenWithEverythingAFloorRaiseCouldRelease {
                device: DEVICE_OF_THE_TEST,
                judged_short_by:
                    JudgementThatFoundTheDeviceShort::SlotPairsOutsideTheClusterSegmentsForDataUnits,
                most_a_floor_raise_could_make_available: AvailableBytesOnOneDevice(i128::from(
                    bytes_of_slot_pairs(1)
                )),
                demand: BytesOnOneDevice(bytes_of_slot_pairs(2)),
            }],
            "上界一对（空着的那一对）、需求两对"
        );
    }

    /// 一个数据单元罩在未释放的记录下（全空聚簇段 0 个，`N₀` 是第一版的 1），被拒的是提交内生块那一判（这一判不算上界，盘那一判判不出）。
    fn allocator_without_an_empty_cluster_segment_and_its_refusal() -> (PoolAllocator, PublishError)
    {
        let mut allocator = allocator_of_one_cluster_segment();
        allocator
            .allocate_user_data(CheckpointTxg(1))
            .expect("空盘分得到一个数据单元");
        assert_eq!(
            allocator.devices[0].empty_segments(),
            0,
            "唯一那一段罩着一条未释放的记录"
        );
        let refusal = refusal_of_the_landing_judgement(&allocator, 0, CLUSTER_SEGMENT_SLOTS);
        (allocator, refusal)
    }

    /// 推串之前全空聚簇段已少于 `N₀`、被拒的是前台写入：先算判放不下、一串都不推，报合取 7 那一份。
    /// 判别力：关掉推串之前判合取 7 那一判，照推。
    #[test]
    fn a_foreground_write_refused_while_empty_cluster_segments_are_below_the_floor_is_not_pushed_for(
    ) {
        let (allocator, refusal) = allocator_without_an_empty_cluster_segment_and_its_refusal();
        let Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space)) =
            judge_before_pushing_a_floor_raise_for_a_refusal(
                &allocator,
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
                nothing_published_in_this_change(),
                refusal,
            )
        else {
            panic!("全空聚簇段在写之前已少于 N₀ 的前台写入：先算判出，不推");
        };
        assert_eq!(
            (
                no_space.devices_short.len(),
                no_space.empty_cluster_segments_below_the_floor,
                no_space.floor_raises.len(),
            ),
            (
                0,
                Some(EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
                    floor_in_segments: 1,
                    short_devices: vec![
                        crate::admission::DeviceWithEmptyClusterSegmentsBelowTheFloor {
                            device: DEVICE_OF_THE_TEST,
                            empty_cluster_segments: 0,
                        }
                    ],
                }),
                0,
            ),
            "没有上界小于需求的盘；合取 7 那一份：N₀ 是 1、这块盘 0 段；一串没推"
        );
    }

    /// 同上，被拒的是取落点那一种：唯一那一段的槽对全分出去（全空聚簇段 0 个，`N₀` 是第一版的 1），再取一个数据单元的落点被分配器拒，
    /// 照发布路径包成 `PlacementRefused`。槽对那一判对这一种交空的，合取 7 那一形照判：先算判出、一串不推，报合取 7 那一份，最后一次被拒原样是取落点那一种。
    /// 判别力：合取 7 那一形只认 `SpaceAdmissionRefused`，照推。
    #[test]
    fn a_foreground_write_refused_a_placement_while_empty_cluster_segments_are_below_the_floor_is_not_pushed_for(
    ) {
        let mut allocator = allocator_of_one_cluster_segment();
        allocate_every_slot_pair(&mut allocator);
        assert_eq!(
            allocator.devices[0].empty_segments(),
            0,
            "唯一那一段的槽对全分出去了"
        );
        let refusal = PublishError::PlacementRefused {
            unit: TransactionUnit::Data(DataUnitIndexInFile::FIRST),
            refusal: allocator
                .try_allocate_user_data(CheckpointTxg(1))
                .expect_err("段外槽对全分出去之后取不到数据单元的落点"),
        };
        let Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space)) =
            judge_before_pushing_a_floor_raise_for_a_refusal(
                &allocator,
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
                nothing_published_in_this_change(),
                refusal,
            )
        else {
            panic!("取落点被拒、全空聚簇段在写之前已少于 N₀ 的前台写入：先算判出，不推");
        };
        assert_eq!(
            (
                no_space.devices_short.len(),
                no_space.empty_cluster_segments_below_the_floor,
                no_space.floor_raises.len(),
                matches!(no_space.last_refusal, PublishError::PlacementRefused { .. }),
            ),
            (
                0,
                Some(EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
                    floor_in_segments: 1,
                    short_devices: vec![
                        crate::admission::DeviceWithEmptyClusterSegmentsBelowTheFloor {
                            device: DEVICE_OF_THE_TEST,
                            empty_cluster_segments: 0,
                        }
                    ],
                }),
                0,
                true,
            ),
            "槽对那一判对取落点被拒交空的；合取 7 那一份：N₀ 是 1、这块盘 0 段；一串没推；最后一次被拒是取落点那一种"
        );
    }

    /// 推串之前合取 7 那一判只拦合取 7 判的前台写入：被拒的是删除类、改 size 或建快照（发之前不知道合取 7 判不判）时照推。
    #[test]
    fn a_publish_exempt_from_the_empty_cluster_segments_conjunct_is_pushed_for_below_the_floor() {
        let (allocator, refusal) = allocator_without_an_empty_cluster_segment_and_its_refusal();
        assert!(
            judge_before_pushing_a_floor_raise_for_a_refusal(
                &allocator,
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish,
                nothing_published_in_this_change(),
                refusal,
            )
            .is_ok(),
            "合取 7 免的发布不先判它：照推"
        );
    }

    /// 推串之前合取 7 那一判只在这次调用还一串没推时判：推过一串之后照推（推完照旧由发布路径判合取 7）。
    #[test]
    fn after_a_series_was_pushed_the_empty_cluster_segments_conjunct_is_not_judged_before_the_next()
    {
        let (allocator, refusal) = allocator_without_an_empty_cluster_segment_and_its_refusal();
        assert!(
            judge_before_pushing_a_floor_raise_for_a_refusal(
                &allocator,
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
                PublishedInThisChange {
                    floor_raises: vec![raised_floor_without_publishes()],
                    refusals_that_pushed_the_floor_raises: vec![refusal_of_the_test()],
                    versions_published: Vec::new(),
                },
                refusal,
            )
            .is_ok(),
            "这次调用已推过一串：不先判合取 7"
        );
    }

    /// 推串之前合取 7 那一判只对空间不够的那两种被拒判：别的被拒原样交给调用方（它交回 `Publish`），不报空间不够。
    #[test]
    fn a_refusal_not_short_of_space_is_not_judged_against_the_empty_cluster_segments_floor() {
        let (allocator, _) = allocator_without_an_empty_cluster_segment_and_its_refusal();
        let refusal_not_short_of_space = PublishError::EmptyClusterSegmentsBelowTheFloor(
            EmptyClusterSegmentsBelowTheFloorOnSomeDevices {
                floor_in_segments: 1,
                short_devices: Vec::new(),
            },
        );
        assert!(
            judge_before_pushing_a_floor_raise_for_a_refusal(
                &allocator,
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
                nothing_published_in_this_change(),
                refusal_not_short_of_space,
            )
            .is_ok(),
            "不是空间不够：原样交回"
        );
    }

    /// 只供测试的开关关掉准入时合取 7 也不判（`transaction` 判它与式子那一判同一个开关）：推串之前同样不判。
    #[test]
    fn with_the_space_admission_switched_off_the_empty_cluster_segments_floor_is_not_judged_before_pushing(
    ) {
        let (mut allocator, refusal) = allocator_without_an_empty_cluster_segment_and_its_refusal();
        allocator.set_space_admission(SpaceAdmission::SkippedByTheTestOnlySwitch);
        assert!(
            judge_before_pushing_a_floor_raise_for_a_refusal(
                &allocator,
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
                nothing_published_in_this_change(),
                refusal,
            )
            .is_ok(),
            "准入关着：合取 7 不判，推串之前也不判"
        );
    }

    /// 被拒的那一次归不归合取 7 管，按会话发的改动分：写文件内容、建 inode 是前台写入；unlink、删快照、改 size、建快照不先判。
    #[test]
    fn data_writes_and_new_inodes_are_foreground_writes_and_the_other_changes_are_not_judged_before_pushing(
    ) {
        let content = [7_u8; 31];
        let foreground_writes = [
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds: 1_000_003,
            }),
            UserChange::SequentialWrite(FirstFile {
                content: &content,
                write_time_seconds: 1_000_007,
            }),
            UserChange::NewInodes {
                count: 2,
                write_time_seconds: 1_000_011,
            },
            UserChange::OverwriteOfAnInode {
                inode: InodeNumber(4_099),
                content: &content,
                write_time_seconds: 1_000_013,
            },
            UserChange::SequentialWriteOfAnInode {
                inode: InodeNumber(4_099),
                content: &content,
                write_time_seconds: 1_000_017,
            },
        ];
        let not_judged_before_pushing = [
            UserChange::Unlink {
                inode: InodeNumber(4_099),
                write_time_seconds: 1_000_019,
            },
            UserChange::CreateSnapshot,
            UserChange::DestroySnapshot {
                snapshot: TreeIdentifier(65_551),
            },
            UserChange::Truncate {
                inode: InodeNumber(4_099),
                new_size: 0,
                write_time_seconds: 1_000_023,
            },
        ];
        for change in foreground_writes {
            assert_eq!(
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::of_the_user_change(change),
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ForegroundWriteTheConjunctJudges,
                "{change:?}"
            );
        }
        for change in not_judged_before_pushing {
            assert_eq!(
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::of_the_user_change(change),
                RefusedPublishUnderTheEmptyClusterSegmentsConjunct::ExemptOrNotKnownBeforeThePublish,
                "{change:?}"
            );
        }
    }

    #[test]
    fn device_short_on_the_slots_for_commit_generated_units_is_always_pushed_for() {
        assert_eq!(
            device_short_even_with_everything_a_floor_raise_could_release(
                &short_by(
                    JudgementThatFoundTheDeviceShort::SlotsLeftForCommitGeneratedUnits,
                    0,
                    SLOT_BYTES,
                ),
                no_slots(),
            ),
            None,
            "这一判不算上界：照推"
        );
    }

    fn raised_floor_without_publishes() -> RaisedFloor {
        RaisedFloor {
            ceiling: CheckpointTxg(7),
            system_configuration_writes_before_the_first_publish:
                WritesByStructureKind::NOTHING_WRITTEN,
            publishes: Vec::new(),
            floor_of_the_publishes: FloorOfThePushedPublishes::RaisedToTheCeiling,
            reclaimed: Vec::new(),
            abandoned_roots_unreadable: 0,
        }
    }

    fn refusal_of_the_test() -> PublishError {
        PublishError::SpaceAdmissionRefused(AdmissionRefusedOnSomeDevices {
            short_devices: vec![short_by(
                JudgementThatFoundTheDeviceShort::FormulaAvailableAgainstDemand,
                0,
                SLOT_BYTES,
            )],
        })
    }

    fn refusal_after_one_series_was_pushed() -> UserChangeRefused {
        no_space_even_with_everything_a_floor_raise_could_release(
            Vec::new(),
            None,
            refusal_of_the_test(),
            PublishedInThisChange {
                floor_raises: vec![raised_floor_without_publishes()],
                refusals_that_pushed_the_floor_raises: vec![refusal_of_the_test()],
                versions_published: Vec::new(),
            },
        )
    }

    /// 第二轮才判到放不下时前面已推过串：丢了它们，模型对拍与调用方都少一串已落盘的。
    #[test]
    fn the_refusal_keeps_the_series_already_pushed_and_the_refusals_that_pushed_them() {
        let UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(no_space) =
            refusal_after_one_series_was_pushed()
        else {
            panic!("交回的就是全放出来也放不下那个成员");
        };
        assert_eq!(
            (
                no_space.floor_raises.len(),
                no_space.refusals_that_pushed_the_floor_raises.len(),
            ),
            (1, 1),
            "前面推成的那一串与推它的那一次被拒都带着"
        );
    }

    #[test]
    fn the_accept_entry_reports_the_series_kept_by_the_refusal() {
        let refused = AcceptRefused::RefusedAfterRaisingTheFloorForSpace {
            judgement: BatchJudgedAtTheAccept::ThisChangeAlone,
            refusal: Box::new(refusal_after_one_series_was_pushed()),
        };
        assert_eq!(
            refused.floor_raises_pushed_by_this_accept().len(),
            1,
            "`accept` 这个入口报拒绝里带的那一串"
        );
    }
}

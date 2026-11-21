//! 每个几何走的那一条历史：mkfs、可写挂载、写与覆盖写、卸载再挂载，每一次交回了输出的发布记一行观测。
use super::*;

/// 发布种类（登记第五节「每一次……的发布记一行：发布种类」），按装置从哪个入口拿到它分。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PublishKind {
    RowOfTheMount,
    WarmUpOfTheMount,
    FloorRaiseOfTheMount,
    DeletionContinuationOfTheMount,
    FloorRaiseOfTheMountsContinuation,
    CompactionContinuationOfTheMount,
    FirstFileVersion,
    SequentialWrite,
    Overwrite,
    FloorRaiseForAUserChange,
    UnmountChain,
}

impl PublishKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            PublishKind::RowOfTheMount => "row_of_the_mount",
            PublishKind::WarmUpOfTheMount => "warm_up_of_the_mount",
            PublishKind::FloorRaiseOfTheMount => "floor_raise_of_the_mount",
            PublishKind::DeletionContinuationOfTheMount => "deletion_continuation_of_the_mount",
            PublishKind::FloorRaiseOfTheMountsContinuation => {
                "floor_raise_of_the_mounts_continuation"
            }
            PublishKind::CompactionContinuationOfTheMount => "compaction_continuation_of_the_mount",
            PublishKind::FirstFileVersion => "first_file_version",
            PublishKind::SequentialWrite => "sequential_write",
            PublishKind::Overwrite => "overwrite",
            PublishKind::FloorRaiseForAUserChange => "floor_raise_for_a_user_change",
            PublishKind::UnmountChain => "unmount_chain",
        }
    }

    /// 登记第六节 X2 量 c实 的那两种：暖机与卸载那一串（登记第十一节「答不了的」：准入里抬 F 那一串与推空不在里面）。
    pub(super) fn counts_for_the_actual_empty_publish_cost(self) -> bool {
        match self {
            PublishKind::WarmUpOfTheMount | PublishKind::UnmountChain => true,
            PublishKind::RowOfTheMount
            | PublishKind::FloorRaiseOfTheMount
            | PublishKind::DeletionContinuationOfTheMount
            | PublishKind::FloorRaiseOfTheMountsContinuation
            | PublishKind::CompactionContinuationOfTheMount
            | PublishKind::FirstFileVersion
            | PublishKind::SequentialWrite
            | PublishKind::Overwrite
            | PublishKind::FloorRaiseForAUserChange => false,
        }
    }

    /// 没有普通分配的空发布：A6（c实 ≤ 那一次之前的 κ）与 PC7（空发布普通分配槽数 = 0）判它们。
    pub(super) fn is_an_empty_publish(self) -> bool {
        match self {
            PublishKind::WarmUpOfTheMount
            | PublishKind::FloorRaiseOfTheMount
            | PublishKind::FloorRaiseOfTheMountsContinuation
            | PublishKind::FloorRaiseForAUserChange
            | PublishKind::UnmountChain => true,
            PublishKind::RowOfTheMount
            | PublishKind::DeletionContinuationOfTheMount
            | PublishKind::CompactionContinuationOfTheMount
            | PublishKind::FirstFileVersion
            | PublishKind::SequentialWrite
            | PublishKind::Overwrite => false,
        }
    }
}

/// 一次发布交回来的那一版：有树的、或树表 0 条的。
#[allow(
    clippy::large_enum_variant,
    reason = "一次调用交回的至多几十版、按值搬一次就记完，装箱只多一层解引用（核心层 PoolVersion 同一个取舍）"
)]
enum PublishedVersion {
    WithTrees(TransactionOutput),
    WithoutFile(VersionWithoutFilePublishOutput),
}

impl PublishedVersion {
    fn of_a_pool_version(version: PoolVersion) -> Self {
        match version {
            PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
                PublishedVersion::WithTrees(std::sync::Arc::unwrap_or_clone(output))
            }
            PoolVersion::WithoutFile(output) => PublishedVersion::WithoutFile(output),
        }
    }

    pub(super) fn txg(&self) -> CheckpointTxg {
        match self {
            PublishedVersion::WithTrees(output) => output.root.checkpoint_txg,
            PublishedVersion::WithoutFile(output) => output.root.checkpoint_txg,
        }
    }

    /// 树表 0 条那一版上的这次发布写下了分配记录树（写行那次）：之后接在这一版后面的发布，本地 κ 带 K1 那一项。
    fn wrote_the_allocation_record_tree_on_a_version_without_file(&self) -> bool {
        match self {
            PublishedVersion::WithTrees(_) => false,
            PublishedVersion::WithoutFile(output) => match &output.allocation_records {
                AllocationRecordsOfTheVersionWithoutFile::WrittenIntoTheAllocationRecordTreeByThisPublish(_) => true,
                AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish => false,
            },
        }
    }

    fn fixed_point_account(
        &self,
        devices: &[DeviceIdentity],
        route_one_count: RouteOneCount,
    ) -> FixedPointAccount {
        match self {
            PublishedVersion::WithTrees(output) => fixed_point_account(
                &output.rewritten,
                Some(output.allocation_records.as_slice()),
                output.root.checkpoint_txg,
                devices,
                route_one_count,
            ),
            PublishedVersion::WithoutFile(output) => {
                let records = match &output.allocation_records {
                    AllocationRecordsOfTheVersionWithoutFile::WrittenIntoTheAllocationRecordTreeByThisPublish(records) => Some(records.as_slice()),
                    AllocationRecordsOfTheVersionWithoutFile::SameAsThePreviousVersionNotHandedInByAZeroUnitPublish => None,
                };
                fixed_point_account(
                    &output.rewritten,
                    records,
                    output.root.checkpoint_txg,
                    devices,
                    route_one_count,
                )
            }
        }
    }
}

impl PublishObservation {
    /// 取得出的空发布的 c实（两条路都在、逐盘相同时取路一）；路二取不出或两条路不同时 `None`。
    pub(super) fn actual_empty_publish_cost(&self) -> Option<u64> {
        let account = self.fixed_point.as_ref()?;
        match account.routes_agree_on_every_device() {
            Some(true) => Some(account.by_roles),
            Some(false) | None => None,
        }
    }
}

/// 接下来的发布要接在后面的那一版在不在装置手里。
#[allow(
    clippy::large_enum_variant,
    reason = "一条历史只有一个，按值换的次数就是发布次数，装箱只多一层解引用"
)]
enum PreviousVersionInHand {
    WithoutFile {
        allocation_record_tree_written: bool,
    },
    WithTrees(TransactionOutput),
    /// 中间有发布的输出没交回：接在它后面的那一次 κ 取不出。
    NotInHand,
}

/// 一条历史走下来的发布记录器：按 txg 接起每一次发布，缺了的 txg 记「取不出」。
struct HistoryRecorder {
    geometry: Geometry,
    devices: Vec<DeviceIdentity>,
    publishes: Vec<PublishObservation>,
    previous: PreviousVersionInHand,
    last_txg: u64,
    /// 每一次被准入或落点拒（推了抬 F 才发成的也算），按发生的次序：登记第五节 G-min「没有一次被准入或落点拒」。
    refusals: Vec<String>,
    /// 第一次覆盖写那一次 PC7 的读数。
    positive_control_of_the_routes: Option<PositiveControlOfTheRoutes>,
}

impl HistoryRecorder {
    /// mkfs 刚做完：接下来的发布接在 mkfs 那一版（树表 0 条、没写过行）后面。
    fn after_make_filesystem(geometry: Geometry, make_filesystem_txg: CheckpointTxg) -> Self {
        Self {
            geometry,
            devices: vec![DeviceIdentity(0), DeviceIdentity(1)],
            publishes: Vec::new(),
            previous: PreviousVersionInHand::WithoutFile {
                allocation_record_tree_written: false,
            },
            last_txg: make_filesystem_txg.0,
            refusals: Vec::new(),
            positive_control_of_the_routes: None,
        }
    }

    fn note_refusal(&mut self, what: String) {
        self.refusals.push(what);
    }

    /// 一次调用交回的几版，按 txg 排好接起来；txg 有缺口的，缺的那几次记「取不出」、之后那一次的 κ 也取不出。
    /// `allocator_before_the_call` 只给这次调用的第一次发布在树表 0 条那一版上算 κ（那时分配器还没被这次调用动过），
    /// `allocator_after_the_call` 给同一次调用里后面那几次，也给有树那几版提供几何。调用结束时会话的现行那一版是 `current`：
    /// 它比交回的最后一版新，中间的记「取不出」，再从 `current` 接上。
    fn record_the_versions_of_a_call(
        &mut self,
        call_kind: PublishKind,
        mut versions: Vec<(PublishKind, PublishedVersion)>,
        cycle: u64,
        allocator_before_the_call: &PoolAllocator,
        allocator_after_the_call: &PoolAllocator,
        current: &PoolVersion,
    ) {
        versions.sort_by_key(|(_kind, version)| version.txg().0);
        let mut first_of_the_call = true;
        for (kind, version) in versions {
            let txg = version.txg().0;
            if txg <= self.last_txg {
                // 同一版从两个入口各交回一次：只记第一次。
                continue;
            }
            self.record_the_publishes_not_handed_back_before(txg, call_kind, cycle);
            let allocator_of_a_version_without_file = if first_of_the_call {
                allocator_before_the_call
            } else {
                allocator_after_the_call
            };
            let checkpoint_cost = match &self.previous {
                PreviousVersionInHand::WithoutFile {
                    allocation_record_tree_written,
                } => Some(checkpoint_cost_reading(
                    self.geometry,
                    VersionToBuildOn::WithoutFile {
                        allocation_record_tree_written: *allocation_record_tree_written,
                        allocator: allocator_of_a_version_without_file,
                    },
                    allocator_after_the_call,
                )),
                PreviousVersionInHand::WithTrees(previous) => Some(checkpoint_cost_reading(
                    self.geometry,
                    VersionToBuildOn::WithTrees(previous),
                    allocator_after_the_call,
                )),
                PreviousVersionInHand::NotInHand => None,
            };
            let fixed_point =
                version.fixed_point_account(&self.devices, RouteOneCount::EachFixedPointRoleOnce);
            if kind == PublishKind::Overwrite && self.positive_control_of_the_routes.is_none() {
                let hook_on = version.fixed_point_account(
                    &self.devices,
                    RouteOneCount::EachFixedPointRoleTwiceForThePositiveControl,
                );
                self.positive_control_of_the_routes = Some(PositiveControlOfTheRoutes {
                    txg,
                    hook_on_routes_agree: hook_on.routes_agree_on_every_device(),
                    hook_off_routes_agree: fixed_point.routes_agree_on_every_device(),
                    ordinary_slots: fixed_point.ordinary_slots,
                });
            }
            self.publishes.push(PublishObservation {
                kind,
                cycle,
                txg,
                checkpoint_cost,
                fixed_point: Some(fixed_point),
            });
            let wrote_the_tree =
                version.wrote_the_allocation_record_tree_on_a_version_without_file();
            let written_before = matches!(
                self.previous,
                PreviousVersionInHand::WithoutFile {
                    allocation_record_tree_written: true
                }
            );
            self.previous = match version {
                PublishedVersion::WithTrees(output) => PreviousVersionInHand::WithTrees(output),
                PublishedVersion::WithoutFile(_) => PreviousVersionInHand::WithoutFile {
                    allocation_record_tree_written: written_before || wrote_the_tree,
                },
            };
            self.last_txg = txg;
            first_of_the_call = false;
        }
        let current_txg = current.root().checkpoint_txg.0;
        if current_txg > self.last_txg {
            self.record_the_publishes_not_handed_back_before(current_txg + 1, call_kind, cycle);
            self.previous = match current {
                PoolVersion::WithFile(output) | PoolVersion::WithTreesAndNoFile(output) => {
                    PreviousVersionInHand::WithTrees(TransactionOutput::clone(output))
                }
                PoolVersion::WithoutFile(_) => PreviousVersionInHand::NotInHand,
            };
        }
    }

    /// `last_txg` 与 `next_txg` 之间没交回的发布各记一行「取不出」（κ 与两条路都取不出），之后那一次的 κ 也取不出。
    fn record_the_publishes_not_handed_back_before(
        &mut self,
        next_txg: u64,
        call_kind: PublishKind,
        cycle: u64,
    ) {
        if next_txg <= self.last_txg + 1 {
            return;
        }
        for txg in self.last_txg + 1..next_txg {
            self.publishes.push(PublishObservation {
                kind: call_kind,
                cycle,
                txg,
                checkpoint_cost: None,
                fixed_point: None,
            });
        }
        self.last_txg = next_txg - 1;
        self.previous = PreviousVersionInHand::NotInHand;
    }
}

/// 一次可写挂载交回的全部发布记下来，交回接着它的会话。取号之前被准入拒、推了抬 F 才过的，记一次被拒（G-min 的「做得完」不收它）。
fn record_the_mount(
    recorder: &mut HistoryRecorder,
    mounted: Mounted,
    cycle: u64,
    allocator_before_the_mount: &PoolAllocator,
) -> MountedSession {
    let (output, session) = MountedSession::of_the_mount(mounted);
    let MountOutput {
        row_publish,
        warm_up_publishes,
        space_admission,
        deletion_queue_continuation,
        floor_raises_pushed_by_the_continuation,
        compaction_continuation,
        ..
    } = output;
    let mount_floor_raises = match space_admission {
        MountSpaceAdmission::AdmittedBeforeAcquisition => Vec::new(),
        MountSpaceAdmission::AdmittedAfterTheFloorRaises { floor_raises, .. } => {
            recorder.note_refusal(format!(
                "mount_cycle{cycle}:refused_before_acquisition_then_admitted_after_{}_floor_raises",
                floor_raises.len()
            ));
            floor_raises
        }
        MountSpaceAdmission::StillShortAfterTheFloorRaises { floor_raises, .. } => {
            recorder.note_refusal(format!(
                "mount_cycle{cycle}:still_short_after_{}_floor_raises",
                floor_raises.len()
            ));
            floor_raises
        }
        MountSpaceAdmission::NotJudgedByTheTestOnlySwitch => {
            recorder.note_refusal(format!("mount_cycle{cycle}:space_admission_not_judged"));
            Vec::new()
        }
    };
    if !floor_raises_pushed_by_the_continuation.is_empty() {
        recorder.note_refusal(format!(
            "mount_cycle{cycle}:continuation_pushed_{}_floor_raises",
            floor_raises_pushed_by_the_continuation.len()
        ));
    }
    let mut versions = vec![(
        PublishKind::RowOfTheMount,
        PublishedVersion::of_a_pool_version(row_publish),
    )];
    versions.extend(warm_up_publishes.into_iter().map(|version| {
        (
            PublishKind::WarmUpOfTheMount,
            PublishedVersion::of_a_pool_version(version),
        )
    }));
    for raised in mount_floor_raises {
        versions.extend(raised.publishes.into_iter().map(|published| {
            (
                PublishKind::FloorRaiseOfTheMount,
                PublishedVersion::WithTrees(published),
            )
        }));
    }
    versions.extend(deletion_queue_continuation.into_iter().map(|published| {
        (
            PublishKind::DeletionContinuationOfTheMount,
            PublishedVersion::WithTrees(std::sync::Arc::unwrap_or_clone(published)),
        )
    }));
    for raised in floor_raises_pushed_by_the_continuation {
        versions.extend(raised.publishes.into_iter().map(|published| {
            (
                PublishKind::FloorRaiseOfTheMountsContinuation,
                PublishedVersion::WithTrees(published),
            )
        }));
    }
    if let Some(run) = compaction_continuation.run {
        versions.extend(run.versions_published.into_iter().map(|published| {
            (
                PublishKind::CompactionContinuationOfTheMount,
                PublishedVersion::WithTrees(published),
            )
        }));
    }
    recorder.record_the_versions_of_a_call(
        PublishKind::RowOfTheMount,
        versions,
        cycle,
        allocator_before_the_mount,
        &session.allocator,
        &session.current,
    );
    session
}

/// 一次用户改动交回的全部发布记下来；为它推了抬 F 的，记一次被拒（推之前那一次是被准入或落点拒的）。
fn record_a_user_change(
    recorder: &mut HistoryRecorder,
    call_kind: PublishKind,
    published: UserChangePublished,
    cycle: u64,
    session: &MountedSession,
) {
    let UserChangePublished {
        floor_raises,
        refusals_that_pushed_the_floor_raises,
        versions_published,
        ..
    } = published;
    if let Some(first_refusal) = refusals_that_pushed_the_floor_raises.first() {
        recorder.note_refusal(format!(
            "{}_cycle{cycle}:{}_then_{}_floor_raises",
            call_kind.label(),
            variant_name_of(&format!("{first_refusal:?}")),
            floor_raises.len()
        ));
    } else if !floor_raises.is_empty() {
        recorder.note_refusal(format!(
            "{}_cycle{cycle}:{}_floor_raises",
            call_kind.label(),
            floor_raises.len()
        ));
    }
    let mut versions = Vec::new();
    for raised in floor_raises {
        versions.extend(raised.publishes.into_iter().map(|output| {
            (
                PublishKind::FloorRaiseForAUserChange,
                PublishedVersion::WithTrees(output),
            )
        }));
    }
    versions.extend(versions_published.into_iter().map(|output| {
        (
            call_kind,
            PublishedVersion::WithTrees(std::sync::Arc::unwrap_or_clone(output)),
        )
    }));
    recorder.record_the_versions_of_a_call(
        call_kind,
        versions,
        cycle,
        &session.allocator,
        &session.allocator,
        &session.current,
    );
}

/// 一条历史走多长（登记第五节第 1–4 步）：产物照 [`HistoryPlan::registered_for`]，单测取短的那一条走同一份代码。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HistoryPlan {
    sequentially_written_data_units: u64,
    overwrites_per_cycle: u64,
    cycles: u64,
}

impl HistoryPlan {
    pub(super) fn registered_for(geometry: Geometry) -> Self {
        Self {
            sequentially_written_data_units: SEQUENTIALLY_WRITTEN_DATA_UNITS,
            overwrites_per_cycle: geometry.overwrites_per_cycle(),
            cycles: CYCLES_OF_OVERWRITE_UNMOUNT_REMOUNT,
        }
    }
}

/// 一条历史停在哪：走完；哪一步没成（步名与原因，原因是错误成员名）；或核心层断言了（panic 的消息）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum HistoryEnd {
    Completed,
    StoppedAt { step: String, reason: String },
    Panicked { message: String },
}

impl HistoryEnd {
    pub(super) fn text(&self) -> String {
        match self {
            HistoryEnd::Completed => "completed".to_string(),
            HistoryEnd::StoppedAt { step, reason } => format!("stopped_at:{step}:{reason}"),
            HistoryEnd::Panicked { message } => format!(
                "panicked:{}",
                message.split_whitespace().collect::<Vec<_>>().join("_")
            ),
        }
    }
}

fn stopped_at(step: String, reason: String) -> HistoryEnd {
    HistoryEnd::StoppedAt { step, reason }
}

impl HistoryReport {
    /// 登记第五节 G-min「第 1–4 步三圈全部发成（没有一次被准入或落点拒）」。
    pub(super) fn completes_the_history(&self) -> bool {
        self.end == HistoryEnd::Completed && self.refusals.is_empty()
    }
}

/// 一条历史在走的时候攒下的、记录器之外的读数。
#[derive(Default)]
struct HistoryExtras {
    unit_areas_of_the_allocator: Vec<(u64, u64)>,
    allocation_record_tree_root_level: Option<u8>,
    final_checkpoint_cost: Option<CheckpointCostReading>,
}

/// 走一条历史；核心层断言了（小盘上可能）不让整个进程倒，记成 [`HistoryEnd::Panicked`]。
pub(super) fn run_the_history(geometry: Geometry, plan: HistoryPlan) -> HistoryReport {
    let mut recorder_slot: Option<HistoryRecorder> = None;
    let mut extras = HistoryExtras::default();
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
        drive_the_history(geometry, plan, &mut recorder_slot, &mut extras)
    }));
    let end = match outcome {
        Ok(Ok(())) => HistoryEnd::Completed,
        Ok(Err(end)) => end,
        Err(payload) => HistoryEnd::Panicked {
            message: payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|text| (*text).to_string())
                })
                .unwrap_or_else(|| "panic_without_a_text_message".to_string()),
        },
    };
    let (refusals, publishes, positive_control_of_the_routes) = match recorder_slot {
        Some(recorder) => (
            recorder.refusals,
            recorder.publishes,
            recorder.positive_control_of_the_routes,
        ),
        None => (Vec::new(), Vec::new(), None),
    };
    HistoryReport {
        geometry,
        end,
        refusals,
        publishes,
        unit_areas_of_the_allocator: extras.unit_areas_of_the_allocator,
        allocation_record_tree_root_level: extras.allocation_record_tree_root_level,
        positive_control_of_the_routes,
        final_checkpoint_cost: extras.final_checkpoint_cost,
    }
}

/// 登记第五节第 1–4 步：mkfs（这个 S）→ 可写挂载（写行 + 暖机）→ 第一个文件版本 → 顺序写 32 个数据单元 →
/// 三圈「覆盖写 4 × 3S 次（`UserChange::Overwrite`）→ 正常卸载 → 再可写挂载」。顺序写要接在带文件的一版后面
/// （`UserChangeRefused::NoFileVersionToChange`），所以先发第一个文件版本（一个数据单元），再顺序写把它换成 32 个数据单元。
/// 哪一步没成交回 `Err`（步名、错误成员名），后面不走；读数照记到停下那一刻。
fn drive_the_history(
    geometry: Geometry,
    plan: HistoryPlan,
    recorder_slot: &mut Option<HistoryRecorder>,
    extras: &mut HistoryExtras,
) -> Result<(), HistoryEnd> {
    let parameters = make_filesystem_parameters(geometry)
        .map_err(|reason| stopped_at("make_filesystem_parameters".to_string(), reason))?;
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    SparseBlockDevice::new(
                        geometry.device_bytes(),
                        PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES_LOCAL),
                    ),
                )
            })
            .collect();
    let genesis = make_filesystem(&parameters, devices.as_mut_slice()).map_err(|error| {
        stopped_at(
            "make_filesystem".to_string(),
            variant_name_of(&format!("{error:?}")),
        )
    })?;
    let allocator_after_mkfs = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    extras.unit_areas_of_the_allocator = allocator_after_mkfs
        .devices
        .iter()
        .map(|device_map| {
            let start = device_map.unit_area_start().slot().0;
            (start, start + device_map.unit_area_slots())
        })
        .collect();
    let recorder = recorder_slot.insert(HistoryRecorder::after_make_filesystem(
        geometry,
        genesis.root.checkpoint_txg,
    ));
    let mounted = mount_writable(&parameters, &mut devices).map_err(|error| {
        stopped_at(
            "first_mount".to_string(),
            variant_name_of(&format!("{error:?}")),
        )
    })?;
    let mut session = record_the_mount(recorder, mounted, 0, &allocator_after_mkfs);
    let allocator_before_the_first_file = session.allocator.clone();
    let first_file_content = one_data_unit_of_content(0);
    let first_file_version = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut session.allocator,
            session.current.root(),
            FirstFile {
                content: &first_file_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            session.instance,
            session.current.record_bytes(),
        )
    }
    .map_err(|error| {
        stopped_at(
            "first_file_version".to_string(),
            variant_name_of(&format!("{error:?}")),
        )
    })?;
    session.current = PoolVersion::of_a_version_of_trees(first_file_version.clone());
    recorder.record_the_versions_of_a_call(
        PublishKind::FirstFileVersion,
        vec![(
            PublishKind::FirstFileVersion,
            PublishedVersion::WithTrees(first_file_version),
        )],
        0,
        &allocator_before_the_first_file,
        &session.allocator,
        &session.current,
    );
    let sequential_content = sequential_write_content(plan.sequentially_written_data_units);
    let sequential = session
        .publish_user_change(
            &mut devices,
            UserChange::SequentialWrite(FirstFile {
                content: &sequential_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
            }),
        )
        .map_err(|refusal| {
            stopped_at(
                "sequential_write".to_string(),
                variant_name_of(&format!("{refusal:?}")),
            )
        })?;
    record_a_user_change(
        recorder,
        PublishKind::SequentialWrite,
        sequential,
        0,
        &session,
    );
    for cycle in 1..=plan.cycles {
        for overwrite_index in 0..plan.overwrites_per_cycle {
            let content = one_data_unit_of_content(cycle * 100_000 + overwrite_index + 1);
            let overwrite = session
                .publish_user_change(
                    &mut devices,
                    UserChange::Overwrite(FirstFile {
                        content: &content,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS
                            + cycle * 100_000
                            + overwrite_index
                            + 2,
                    }),
                )
                .map_err(|refusal| {
                    stopped_at(
                        format!("overwrite_cycle{cycle}_index{overwrite_index}"),
                        variant_name_of(&format!("{refusal:?}")),
                    )
                })?;
            record_a_user_change(recorder, PublishKind::Overwrite, overwrite, cycle, &session);
        }
        let unmounted = unmount(
            &parameters,
            &mut devices,
            &mut session.allocator,
            &mut session.current,
            session.shadow_ledger,
        )
        .map_err(|error| {
            stopped_at(
                format!("unmount_cycle{cycle}"),
                variant_name_of(&format!("{error:?}")),
            )
        })?;
        let unmount_chain = match unmounted {
            Unmounted::FloorRaisedToTheCurrentVersion(raised) => raised.publishes,
            Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => {
                return Err(stopped_at(
                    format!("unmount_cycle{cycle}"),
                    "nothing_written_on_a_version_without_file".to_string(),
                ))
            }
        };
        recorder.record_the_versions_of_a_call(
            PublishKind::UnmountChain,
            unmount_chain
                .into_iter()
                .map(|output| {
                    (
                        PublishKind::UnmountChain,
                        PublishedVersion::WithTrees(output),
                    )
                })
                .collect(),
            cycle,
            &session.allocator,
            &session.allocator,
            &session.current,
        );
        let remounted = mount_writable(&parameters, &mut devices).map_err(|error| {
            stopped_at(
                format!("remount_cycle{cycle}"),
                variant_name_of(&format!("{error:?}")),
            )
        })?;
        session = record_the_mount(recorder, remounted, cycle, &allocator_after_mkfs);
    }
    if let PoolVersion::WithFile(last) | PoolVersion::WithTreesAndNoFile(last) = &session.current {
        extras.allocation_record_tree_root_level = Some(
            u8::try_from(
                last.position_addressed_tree_heights_read_from_the_root_node_headers()
                    .allocation_record_tree
                    - 1,
            )
            .expect("根层级装得进 u8"),
        );
        extras.final_checkpoint_cost = Some(checkpoint_cost_reading(
            geometry,
            VersionToBuildOn::WithTrees(last),
            &session.allocator,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;

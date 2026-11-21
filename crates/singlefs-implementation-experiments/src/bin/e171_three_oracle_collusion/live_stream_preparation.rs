//! 跑着的流与备好的流：两条流从建池到收尾的驱动，与流的形状回比 crates。

use super::*;

/// 一条流驱动时的池：只做过 mkfs 的两块文件镜像（`FormattedPool` 管镜像文件，丢掉就删）、分配器、现行那一版、崩溃点重放手写 oracle 的版本表。
/// 流一照抄 `tests/common/mod.rs` 的 `build_pool` 的取号、暖机、新池新建文件三步，流二照抄
/// `tests/crash_enumeration_fixed_script_stream.rs` 的 `prepare` 里 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支；
/// 每一步都经 [`StreamDriver::call`]。
struct LiveStream {
    pub(super) driver: StreamDriver,
    formatted: experiment_pool::FormattedPool,
    pub(super) allocator: PoolAllocator,
    pub(super) instance: Option<InstanceGeneration>,
    pub(super) warm_up: Option<WarmUpOutput>,
    pub(super) output: Option<TransactionOutput>,
    pub(super) versions: Vec<PublishedVersion>,
    pub(super) checks: Vec<StopCheck>,
    /// 暖机返回时整条录制流（含 mkfs）的长度。
    warm_up_operation_count: usize,
}

impl LiveStream {
    pub(super) fn start(name: StreamName, tag: &str, switches: RunSwitches) -> Self {
        let formatted = experiment_pool::format_pool(tag);
        let mut allocator = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), LOCAL_DEVICE_BYTES),
            DeviceFreeMap::new(DeviceIdentity(1), LOCAL_DEVICE_BYTES),
        ]);
        allocator.mark_format_time_units(
            Placement {
                slot: INSTANCE_TABLE_SLOT,
                span: 2,
            },
            Placement {
                slot: TREE_TABLE_GENESIS_SLOT,
                span: 1,
            },
        );
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
            &experiment_pool::parameters(),
            &formatted.genesis,
        ));
        let driver = StreamDriver::new(
            name,
            formatted.stream.clone(),
            formatted.mkfs_operation_count,
            switches,
        );
        Self {
            driver,
            formatted,
            allocator,
            instance: None,
            warm_up: None,
            output: None,
            versions: Vec::new(),
            checks: Vec::new(),
            warm_up_operation_count: 0,
        }
    }

    /// 取号、暖机两次调用；两次都返回之后模型推一步。
    fn step_acquire_and_warm_up(&mut self) -> bool {
        let parameters = experiment_pool::parameters();
        let genesis = self.formatted.genesis.clone();
        let devices = self
            .formatted
            .devices
            .as_mut()
            .expect("format_pool 交回开着的设备");
        let mut pool = PoolWriter::new(&parameters, devices);
        let instance = match self.driver.call("acquire", || acquire_instance(&mut pool)) {
            Ok(instance) => instance,
            Err(failure) => {
                self.driver
                    .record_a_refusal_the_model_only_goes_forward_on(failure);
                return false;
            }
        };
        self.checks.push(StopCheck::new(
            "first_acquired_instance_is_one",
            InstanceGeneration(1),
            instance,
        ));
        let warmed = match self
            .driver
            .call("warm_up", || warm_up(&mut pool, &genesis.root, instance))
        {
            Ok(warmed) => warmed,
            Err(failure) => {
                self.driver
                    .record_a_refusal_the_model_only_goes_forward_on(failure);
                return false;
            }
        };
        self.warm_up_operation_count = self.driver.stream.operation_count();
        self.driver
            .advance_the_model_past_the_acquire_and_the_warm_up();
        self.instance = Some(instance);
        self.warm_up = Some(warmed);
        true
    }

    /// 新池新建文件（`publish_first_file`）：按模型的 `answer_publish_first_file` 比。
    fn step_publish_first_file(&mut self) -> bool {
        let parameters = experiment_pool::parameters();
        let content = local_first_file_content();
        let instance = self.instance.expect("取号成功才走到这一步");
        let warmed = self.warm_up.as_ref().expect("暖机成功才走到这一步");
        let devices = self
            .formatted
            .devices
            .as_mut()
            .expect("format_pool 交回开着的设备");
        let mut pool = PoolWriter::new(&parameters, devices);
        let stream_length_before = self.driver.stream.operation_count();
        let result = self.driver.call("publish", || {
            publish_first_file(
                &mut pool,
                &mut self.allocator,
                warmed.roots.last().expect("暖机两代根"),
                FirstFile {
                    content: &content,
                    write_time_seconds: LOCAL_FIXED_WRITE_TIME_SECONDS,
                },
                instance,
                &warmed.last_record_bytes,
            )
        });
        match result {
            Ok(output) => {
                self.driver.judge(
                    |model| model.answer_publish_first_file(&content),
                    ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                        roots: vec![observed_root_of_file_version(&output)],
                        reported_ceiling: None,
                    }),
                );
                self.output = Some(output);
                true
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_publish_error(&error),
                    0,
                    stream_length_before,
                    None,
                    None,
                );
                self.driver
                    .judge(|model| model.answer_publish_first_file(&content), observed);
                false
            }
            Err(None) => false,
        }
    }

    /// 覆盖写（`publish_overwrite`，写时刻取本地常量 + 60）：按模型的 `answer_publish_overwrite` 比。
    fn step_overwrite(&mut self, content: &[u8], instance: InstanceGeneration) -> bool {
        let parameters = experiment_pool::parameters();
        let previous = self.output.as_ref().expect("上一版带文件才走到这一步");
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let stream_length_before = self.driver.stream.operation_count();
        let result = self.driver.call("publish", || {
            publish_overwrite(
                &mut writer,
                &mut self.allocator,
                previous,
                FirstFile {
                    content,
                    write_time_seconds: LOCAL_FIXED_WRITE_TIME_SECONDS + 60,
                },
                instance,
            )
        });
        match result {
            Ok(output) => {
                self.driver.judge(
                    |model| model.answer_publish_overwrite(content),
                    ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                        roots: vec![observed_root_of_file_version(&output)],
                        reported_ceiling: None,
                    }),
                );
                self.output = Some(output);
                true
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_publish_error(&error),
                    0,
                    stream_length_before,
                    None,
                    None,
                );
                self.driver
                    .judge(|model| model.answer_publish_overwrite(content), observed);
                false
            }
            Err(None) => false,
        }
    }

    /// 进程重开之后的可写挂载（取号、写行、暖机）：按模型的 `answer_mount_writable` 比；准入不够时写行之后推的几串抬 F 逐串再比
    /// （`history.rs` 的 `settle_mount` 同一种比法）。
    fn step_mount_writable(&mut self) -> bool {
        if self.driver.model_is_enabled() {
            self.driver.model.close_session();
        }
        let parameters = experiment_pool::parameters();
        let stream_length_before = self.driver.stream.operation_count();
        let formatted = &mut self.formatted;
        let result = self.driver.call("mount_writable", || {
            let mut devices = formatted.reopen_recorded();
            let mounted = mount_writable(&parameters, &mut devices);
            formatted.devices = Some(devices);
            mounted
        });
        match result {
            Ok(mounted) => {
                let (observed, _carried_to_zero_unit_publishes) =
                    observed_mount_carrying_to_zero_unit_publishes(&mounted);
                self.driver.judge(
                    |model| Ok(model.answer_mount_writable()),
                    ObservedOutcome::Succeeded(observed),
                );
                for raised in mounted.output.space_admission.floor_raises() {
                    let raised_floor = raised
                        .publishes
                        .first()
                        .expect("抬 F 做成至少推一次空发布")
                        .root
                        .rollback_floor;
                    self.driver.judge(
                        |model| {
                            model.answer_raise_rollback_floor(
                                singlefs_verification::model::ModelCheckpointTxg(raised_floor.0),
                            )
                        },
                        ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                            roots: raised
                                .publishes
                                .iter()
                                .map(observed_root_of_file_version)
                                .collect(),
                            reported_ceiling: Some(
                                singlefs_verification::model::ModelCheckpointTxg(raised.ceiling.0),
                            ),
                        }),
                    );
                }
                self.allocator = mounted.allocator;
                self.output = mounted.current.into_file_version();
                if self.output.is_none() {
                    self.driver.halted = true;
                    self.driver.implementation_crash =
                        Some("mount_writable_returned_a_version_without_a_file".to_string());
                }
                self.output.is_some()
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_mount_error(&error),
                    0,
                    stream_length_before,
                    None,
                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
                );
                self.driver
                    .judge(|model| Ok(model.answer_mount_writable()), observed);
                false
            }
            Err(None) => false,
        }
    }

    /// 同一个会话里挂着的时候回退到 A 的根（一次向前发布）：按模型的 `answer_rollback_while_mounted` 比。
    fn step_roll_back(&mut self) -> bool {
        let target = RollbackTarget {
            instance: InstanceGeneration(1),
            checkpoint_txg: CheckpointTxg(3),
        };
        let target_key = model_root_key(target.instance, target.checkpoint_txg);
        let parameters = experiment_pool::parameters();
        let output = self.output.as_mut().expect("上一版带文件才走到这一步");
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let stream_length_before = self.driver.stream.operation_count();
        let result = self.driver.call("roll_back", || {
            roll_back_by_a_forward_publish(
                &parameters,
                devices,
                &mut self.allocator,
                &mut *output,
                target,
            )
        });
        match result {
            Ok(_rolled_back) => {
                let observed = ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: vec![observed_root_of_file_version(output)],
                    reported_ceiling: None,
                });
                self.driver.judge(
                    |model| model.answer_rollback_while_mounted(target_key),
                    observed,
                );
                true
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_rollback_error(&error),
                    0,
                    stream_length_before,
                    None,
                    root_ring_slot_still_bad_after_one_reread_of_rollback_error(&error),
                );
                self.driver.judge(
                    |model| model.answer_rollback_while_mounted(target_key),
                    observed,
                );
                false
            }
            Err(None) => false,
        }
    }

    /// 抬 F 到 txg 11（`raise_rollback_floor`，影子账开）：按模型的 `answer_raise_rollback_floor` 比（拒与成都要与实现报的上限相等）。
    fn step_raise_floor(&mut self) -> bool {
        let new_floor = CheckpointTxg(11);
        let parameters = experiment_pool::parameters();
        let mut current = self.output.clone().expect("上一版带文件才走到这一步");
        let txg_before_the_raise = current.root.checkpoint_txg.0;
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let stream_length_before = self.driver.stream.operation_count();
        let result = self.driver.call("raise_floor", || {
            raise_rollback_floor(
                &parameters,
                devices,
                &mut self.allocator,
                &mut current,
                new_floor,
                ShadowLedger::On,
            )
        });
        let publishes_completed = usize::try_from(
            current
                .root
                .checkpoint_txg
                .0
                .saturating_sub(txg_before_the_raise),
        )
        .expect("一次抬 F 至多推根环区域数那么多次");
        self.output = Some(current);
        let floor = singlefs_verification::model::ModelCheckpointTxg(new_floor.0);
        match result {
            Ok(raised) => {
                self.checks.push(StopCheck::new(
                    "second_stream_raise_floor_publishes",
                    LOCAL_SECOND_STREAM_RAISE_FLOOR_PUBLISHES,
                    raised.publishes.len(),
                ));
                let observed = ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                    roots: raised
                        .publishes
                        .iter()
                        .map(observed_root_of_file_version)
                        .collect(),
                    reported_ceiling: Some(singlefs_verification::model::ModelCheckpointTxg(
                        raised.ceiling.0,
                    )),
                });
                self.driver
                    .judge(|model| model.answer_raise_rollback_floor(floor), observed);
                true
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_mount_error(&error),
                    publishes_completed,
                    stream_length_before,
                    reported_ceiling_of_mount_error(&error),
                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
                );
                self.driver
                    .judge(|model| model.answer_raise_rollback_floor(floor), observed);
                false
            }
            Err(None) => false,
        }
    }

    /// 正常卸载（`unmount`，入口记在录制流里）：模型没有卸载的答，不比；模型停在这里。
    fn step_unmount(&mut self) -> bool {
        let mut current = PoolVersion::WithFile(std::sync::Arc::new(
            self.output.clone().expect("上一版带文件"),
        ));
        let stream = self.driver.stream.clone();
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let result = self.driver.call(UNMOUNT_CALL_LABEL, || {
            stream.record_entry(RecordedPublishEntry::Unmount, || {
                unmount(
                    &experiment_pool::parameters(),
                    devices,
                    &mut self.allocator,
                    &mut current,
                    ShadowLedger::On,
                )
            })
        });
        self.driver
            .note_that_the_model_has_no_answer_for_the_last_call();
        self.output = current.into_file_version();
        match result {
            Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
                self.checks.push(StopCheck::new(
                    "second_stream_unmount_txgs",
                    LOCAL_SECOND_STREAM_UNMOUNT_TXGS.to_vec(),
                    unmounted_txgs_of(&raised),
                ));
                true
            }
            Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) | Err(_) => false,
        }
    }
}

fn unmounted_txgs_of(raised: &UnmountRaisedTheFloor) -> Vec<u64> {
    raised
        .publishes
        .iter()
        .map(|publish| publish.root.checkpoint_txg.0)
        .collect()
}

impl LiveStream {
    /// 整条流的脚本：任一步出错或 panic 就停（这条流不再往下驱动）。流一是取号、暖机、新池新建文件；流二接着照
    /// `tests/crash_enumeration_fixed_script_stream.rs` 的脚本走到 E 再正常卸载。每一步成功之后才往崩溃点重放手写 oracle 的版本表里记一版。
    fn run_the_script(&mut self) {
        if !self.step_acquire_and_warm_up() || !self.step_publish_first_file() {
            return;
        }
        if self.driver.name == StreamName::NewPoolFileCreation {
            return;
        }
        self.versions
            .push(version(1, 3, local_first_file_content()));
        if !self.step_overwrite(&second_content(), InstanceGeneration(1)) {
            return;
        }
        self.versions.push(version(1, 4, second_content()));
        // 进程重开：取号 2、写行、暖机两次（txg 5 / 6 落盘 0、txg 7 落盘 1），文件还是第二次的内容。
        if !self.step_mount_writable() {
            return;
        }
        for checkpoint_txg in 5..=7 {
            self.versions
                .push(version(2, checkpoint_txg, second_content()));
        }
        if !self.step_overwrite(&third_content(), InstanceGeneration(2)) {
            return;
        }
        self.versions.push(version(2, 8, third_content()));
        // 同一个会话里挂着的时候回退到 A 的根：一次向前发布 D（txg 9 落盘 0），文件回到第一次的内容。
        if !self.step_roll_back() {
            return;
        }
        self.versions
            .push(version(2, 9, local_first_file_content()));
        let mut latest = Vec::new();
        for (checkpoint_txg, seed) in [(10u64, 13usize), (11, 17), (12, 19), (13, 23), (14, 29)] {
            latest = later_content(seed);
            if !self.step_overwrite(&latest, InstanceGeneration(2)) {
                return;
            }
            self.versions
                .push(version(2, checkpoint_txg, latest.clone()));
        }
        if !self.step_raise_floor() {
            return;
        }
        for checkpoint_txg in 15..=16 {
            self.versions
                .push(version(2, checkpoint_txg, latest.clone()));
        }
        if !self.step_overwrite(&later_content(31), InstanceGeneration(2)) {
            return;
        }
        let reuse_slot = self
            .output
            .as_ref()
            .and_then(|output| output.data_pointers.first())
            .and_then(|pointer| pointer.locations.first())
            .map(|location| location.slot.0);
        self.checks.push(StopCheck::new(
            "second_stream_e_data_unit_slot",
            Some(LOCAL_SECOND_STREAM_REUSED_VERSION_DATA_UNIT_SLOT),
            reuse_slot,
        ));
        self.versions.push(version(2, 17, later_content(31)));
        if !self.step_unmount() {
            return;
        }
        for checkpoint_txg in 18..=19 {
            self.versions
                .push(version(2, checkpoint_txg, later_content(31)));
        }
    }
}

/// 一条流驱动完之后的样子：mkfs 之后的基镜像、mkfs 之后的录制流、写表与段、写表每项在录制流里的下标、调用表、被判的根、版本表、
/// 模型提交过的版本目录、S1 回比，以及驱动怎么结束的。
pub(super) struct PreparedStream {
    pub(super) name: StreamName,
    pub(super) base: MemoryPool,
    pub(super) operations: Vec<RetainedOperation>,
    pub(super) writes: Vec<RetainedWrite>,
    pub(super) segments: Vec<Vec<usize>>,
    pub(super) stream_indexes: Vec<usize>,
    pub(super) calls: Vec<RecordedCall>,
    pub(super) judged_root_index: usize,
    pub(super) versions: Vec<PublishedVersion>,
    /// 模型的版本目录的内容：取样态分几条线程判，这份不用 `Rc`，每条线程照它造自己那一份（[`committed_versions_for_this_thread`]）。
    pub(super) committed_version_contents: BTreeMap<ModelRootKey, Option<Vec<u8>>>,
    /// 模型不再往前走的那次调用（`None`：整条流的每次调用都比过了）。
    pub(super) model_stopped_at_call: Option<usize>,
    pub(super) checks: Vec<StopCheck>,
    pub(super) ending: StreamEnding,
}

/// 驱动怎么结束的：每一步都成功；某一步交回错误（这条流在那里截断，仍照常判）；某一步 panic（实现自崩，这条流不导出）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum StreamEnding {
    Completed,
    StoppedByAnError { call_index: usize, member: String },
    ImplementationCrashed { what: String },
}

/// 驱动一条流并取出它的基镜像、写表与段。`check_against_the_crates` 只在基线那一趟开：流一整条流逐条与验证包库的新池新建文件那条路比（S1）。
pub(super) fn prepare_stream(
    name: StreamName,
    switches: RunSwitches,
    check_against_the_crates: bool,
) -> PreparedStream {
    let tag = format!("e171-{}", name.directory_name());
    let mut live = LiveStream::start(name, &tag, switches);
    live.run_the_script();
    let ending = match (&live.driver.implementation_crash, live.driver.halted) {
        (Some(what), _) => StreamEnding::ImplementationCrashed { what: what.clone() },
        (None, true) => {
            let call_index = live.driver.calls.len() - 1;
            StreamEnding::StoppedByAnError {
                call_index,
                member: live.driver.calls[call_index].outcome.outcome_word(),
            }
        }
        (None, false) => StreamEnding::Completed,
    };
    let all_operations = live.formatted.retained_operations();
    let mkfs_operation_count = live.formatted.mkfs_operation_count;
    // 照抄 `tests/crash_enumeration_fixed_script_stream.rs` 的 `writes_and_segments_after_mkfs`：入口段（卸载入口发的那几段）一起交给切段（C557）。
    let entry_spans: Vec<RecordedEntrySpan> = live
        .formatted
        .stream
        .entry_spans()
        .into_iter()
        .map(|span| RecordedEntrySpan {
            entry: span.entry,
            operations: span.operations.start - mkfs_operation_count
                ..span.operations.end - mkfs_operation_count,
        })
        .collect();
    let operations = all_operations[mkfs_operation_count..].to_vec();
    let (writes, segments, stream_indexes) = writes_and_segments_with_stream_indexes_and_entries(
        &operations,
        &entry_spans,
        &experiment_pool::geometry(),
    );
    let base = live.formatted.memory_pool_after_mkfs();
    let mut checks = std::mem::take(&mut live.checks);
    if check_against_the_crates {
        checks.extend(stream_checks_against_the_crates(
            name,
            &live,
            &all_operations,
            &operations,
            &writes,
            &segments,
            &stream_indexes,
            &base,
        ));
    }
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .unwrap_or(usize::MAX);
    let versions = if name == StreamName::NewPoolFileCreation && judged_root_index != usize::MAX {
        // 流一的版本表照 `tests/crash_enumeration_new_pool_file_creation_stream.rs` 的 `single_version`。
        let (instance, checkpoint_txg) =
            singlefs_verification::memory_pool::root_identity_of_write(&writes[judged_root_index]);
        vec![PublishedVersion {
            instance,
            checkpoint_txg,
            content: local_first_file_content(),
        }]
    } else {
        std::mem::take(&mut live.versions)
    };
    PreparedStream {
        name,
        base,
        operations,
        writes,
        segments,
        stream_indexes,
        calls: live.driver.calls.clone(),
        judged_root_index,
        versions,
        committed_version_contents: std::mem::take(&mut live.driver.committed_versions)
            .into_iter()
            .map(|(key, file)| (key, file.map(|content| content.to_vec())))
            .collect(),
        model_stopped_at_call: live.driver.model_stopped_at_call,
        checks,
        ending,
    }
}

/// S1 里跟流有关的几条（只在基线那一趟）：段长表、写数、根槽写数、盘数与盘字节数对本地常量；流一整条流逐条与验证包库的新池新建文件那条路比。
/// 与 `e170_specification_executor_export.rs` 的 `prepare_stream` 里那一批 S1 回比同一批。
#[allow(
    clippy::too_many_arguments,
    reason = "这几样是 prepare_stream 驱动完之后手里同时有的，拆成结构体只会多一层搬运"
)]
fn stream_checks_against_the_crates(
    name: StreamName,
    live: &LiveStream,
    all_operations: &[RetainedOperation],
    operations: &[RetainedOperation],
    writes: &[RetainedWrite],
    segments: &[Vec<usize>],
    stream_indexes: &[usize],
    base: &MemoryPool,
) -> Vec<StopCheck> {
    let mkfs_operation_count = live.formatted.mkfs_operation_count;
    let label = name.directory_name();
    let non_barrier_operations: Vec<usize> = operations
        .iter()
        .enumerate()
        .filter(|(_, retained)| retained.operation.kind != RecordedOperationKind::Barrier)
        .map(|(stream_index, _)| stream_index)
        .collect();
    let (local_segment_lengths, local_writes, local_root_writes): (&[usize], usize, usize) =
        match name {
            StreamName::NewPoolFileCreation => (
                &LOCAL_FIRST_STREAM_SEGMENT_LENGTHS,
                LOCAL_FIRST_STREAM_WRITES,
                LOCAL_FIRST_STREAM_ROOT_WRITES,
            ),
            StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => (
                &LOCAL_SECOND_STREAM_SEGMENT_LENGTHS,
                LOCAL_SECOND_STREAM_WRITES,
                LOCAL_SECOND_STREAM_ROOT_WRITES,
            ),
        };
    let root_write_count = writes
        .iter()
        .filter(|write| write.kind == StepKind::RootRecordFua)
        .count();
    let mut checks = vec![
        StopCheck::new(
            &format!("{label}_write_order_is_the_non_barrier_operation_order"),
            true,
            non_barrier_operations == stream_indexes,
        ),
        StopCheck::new(
            &format!("{label}_segment_lengths"),
            local_segment_lengths.to_vec(),
            segments.iter().map(Vec::len).collect::<Vec<usize>>(),
        ),
        StopCheck::new(&format!("{label}_writes"), local_writes, writes.len()),
        StopCheck::new(
            &format!("{label}_root_writes"),
            local_root_writes,
            root_write_count,
        ),
        StopCheck::new(
            &format!("{label}_device_count"),
            LOCAL_DEVICE_COUNT,
            base.devices.len(),
        ),
        StopCheck::new(
            &format!("{label}_device_bytes"),
            LOCAL_DEVICE_BYTES,
            base.device_size_in_bytes,
        ),
    ];
    if name == StreamName::NewPoolFileCreation {
        // 流一的驱动走的是新池新建文件那三步：整条流（含 mkfs）逐条与验证包库的新池新建文件那条路比（不拿实验包自己的造池比自己）。
        let reference = experiment_pool::new_pool_file_creation_of_the_verification_library();
        checks.push(StopCheck::new(
            "new_pool_file_creation_operations_equal_the_verification_scenario",
            true,
            reference.operations == all_operations,
        ));
        checks.push(StopCheck::new(
            "new_pool_file_creation_mkfs_operations_equal_the_verification_scenario",
            mkfs_operation_count,
            reference.mkfs_operation_count,
        ));
        checks.push(StopCheck::new(
            "new_pool_file_creation_warm_up_return_point_equal_the_verification_scenario",
            live.warm_up_operation_count,
            reference.warm_up_operation_count,
        ));
    }
    checks
}

#[cfg(test)]
mod tests;

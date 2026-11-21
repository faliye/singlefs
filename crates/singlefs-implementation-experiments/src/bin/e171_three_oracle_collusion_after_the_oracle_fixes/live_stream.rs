//! 跑着的流：三条流从建池到收尾的驱动。

use super::*;

/// 流三的一次发布（[`LiveStream::step_third_stream_publish`] 的 `publish`）：经 `PoolWriter` 调实现的发布函数，交回那一版或发布错误。
type ThirdStreamPublish<'closure> = dyn Fn(
        &mut PoolWriter<'_, experiment_pool::RecordedImageDevice>,
        &mut PoolAllocator,
        &TransactionOutput,
        InstanceGeneration,
    ) -> Result<TransactionOutput, singlefs_core::transaction::PublishError>
    + 'closure;

/// 一条流驱动时的池：只做过 mkfs 的两块文件镜像（`FormattedPool` 管镜像文件，丢掉就删）、分配器、现行那一版、崩溃点重放手写 oracle 的版本表。
/// 流一照抄 `tests/common/mod.rs` 的 `build_pool` 的取号、暖机、新池新建文件三步，流二照抄
/// `tests/crash_enumeration_fixed_script_stream.rs` 的 `prepare` 里 `Script::ReuseAfterRaisingFloorThenNormalUnmount` 那一支，
/// 流三接着流一删第一个文件、再一次建一个 inode 直到 inode 树的叶容器第一次变成两片、再多建一次（读法 2）；每一步都经 [`StreamDriver::call`]。
pub(super) struct LiveStream {
    pub(super) driver: StreamDriver,
    pub(super) formatted: experiment_pool::FormattedPool,
    pub(super) allocator: PoolAllocator,
    pub(super) instance: Option<InstanceGeneration>,
    pub(super) warm_up: Option<WarmUpOutput>,
    pub(super) output: Option<TransactionOutput>,
    /// 可写挂载交回的 `MountOutput` 的 `{:?}` 文本：算进之后每次调用的内存对象（读法 4）。
    pub(super) mount_output_text: Option<String>,
    pub(super) versions: Vec<PublishedVersion>,
    pub(super) checks: Vec<StopCheck>,
    /// 暖机返回时整条录制流（含 mkfs）的长度。
    pub(super) warm_up_operation_count: usize,
    /// 流三：删第一个文件那一段的调用数（意图一次加走空的每一步）、建 inode 发布的次数、叶容器第一次变成两片的那一次（从 1 数，K）。
    unlink_calls: usize,
    inode_creations: usize,
    split_publish: Option<usize>,
}

/// S1 启动核对 `<流>_allocator_carries_the_root_ring_of_make_filesystem`（登记修订 6 第 6.2 节第 1 条）：分配器装着的根环表与
/// `root_ring_occupancy_after_make_filesystem` 给这次 mkfs 算出的那一份 `{:?}` 文本相同。`RootRingOccupancy` 只派生 `Clone, Debug`，
/// 所以按文本比；两边记的是文本的 sha256（文本长、带空白，结果行里只放摘要），没装记 `None`。
fn allocator_carries_the_root_ring_of_make_filesystem_check(
    name: StreamName,
    allocator: &PoolAllocator,
    genesis: &MakeFilesystemOutput,
) -> StopCheck {
    let digest_of_the_debug_text =
        |occupancy: &RootRingOccupancy| sha256_hexadecimal(debug_text(occupancy).as_bytes());
    StopCheck::new(
        &format!(
            "{}_allocator_carries_the_root_ring_of_make_filesystem",
            name.directory_name()
        ),
        allocator
            .root_ring_occupancy()
            .map(digest_of_the_debug_text),
        Some(digest_of_the_debug_text(
            &root_ring_occupancy_after_make_filesystem(&experiment_pool::parameters(), genesis),
        )),
    )
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
        // 登记修订 6 第 6.2 节第 1 条：mkfs 同一个进程里接着写的会话照 core 部署的那一种，装上 mkfs 刚写下的根环表
        // （转过根环之后照 D16（发布语义） 已定项 1 的可再分配谓词回收）；空闲图照旧用 `DeviceFreeMap::new`（S1 回比验证包库的新池新建文件那条路、S2 比 E170）。
        allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(
            &experiment_pool::parameters(),
            &formatted.genesis,
        ));
        let root_ring_check = allocator_carries_the_root_ring_of_make_filesystem_check(
            name,
            &allocator,
            &formatted.genesis,
        );
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
            mount_output_text: None,
            versions: Vec::new(),
            checks: vec![root_ring_check],
            warm_up_operation_count: 0,
            unlink_calls: 0,
            inode_creations: 0,
            split_publish: None,
        }
    }

    /// 刚记的这次调用之后的内存对象（[`memory_objects_of`]）。阳性对照「内存对象」：流二第 2 次调用之后，分配器那一项换成在副本上多分配一槽的那份。
    fn record_memory_objects(&mut self) {
        let call_index = self.driver.calls.len() - 1;
        let mut objects = memory_objects_of(
            &self.allocator,
            self.instance,
            self.warm_up.as_ref(),
            self.output.as_ref(),
            self.mount_output_text.as_deref(),
        );
        if self.driver.switches.positive_control == Some(PositiveControl::MemoryObject)
            && self.driver.name == StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount
            && call_index == RETURN_VALUE_CONTROL_CALL
        {
            let mut allocator_with_one_more_slot = self.allocator.clone();
            let generation = self.output.as_ref().map_or(CheckpointTxg(1), |output| {
                CheckpointTxg(output.root.checkpoint_txg.0 + 1)
            });
            let _ = allocator_with_one_more_slot.allocate_user_data(generation);
            objects[0].1 = debug_text(&allocator_with_one_more_slot);
        }
        self.driver
            .record_the_memory_objects_after_the_last_call(objects);
    }

    /// 取号、暖机两次调用；两次都返回之后模型推一步。两次调用共用一个 `PoolWriter`（与验证包库的新池新建文件那条路同一条写流，S1 逐条比）。
    fn step_acquire_and_warm_up(&mut self) -> bool {
        let parameters = experiment_pool::parameters();
        let genesis = self.formatted.genesis.clone();
        let devices = self
            .formatted
            .devices
            .as_mut()
            .expect("format_pool 交回开着的设备");
        let mut pool = PoolWriter::new(&parameters, devices);
        let acquired = self
            .driver
            .call("acquire", debug_text::<InstanceGeneration>, || {
                acquire_instance(&mut pool)
            });
        let acquired_instance = acquired.as_ref().ok().copied();
        self.driver
            .record_the_memory_objects_after_the_last_call(memory_objects_of(
                &self.allocator,
                acquired_instance,
                None,
                None,
                None,
            ));
        let instance = match acquired {
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
        let warmed_result = self.driver.call("warm_up", debug_text::<WarmUpOutput>, || {
            warm_up(&mut pool, &genesis.root, instance)
        });
        self.driver
            .record_the_memory_objects_after_the_last_call(memory_objects_of(
                &self.allocator,
                Some(instance),
                warmed_result.as_ref().ok(),
                None,
                None,
            ));
        let warmed = match warmed_result {
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

    /// 新池新建文件（`publish_first_file`）：按模型的 `answer_publish_first_file` 比。阳性对照「交回值」改的就是流二这一次（第 2 次调用）。
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
        let result = self
            .driver
            .call("publish", debug_text::<TransactionOutput>, || {
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
        let call_index = self.driver.calls.len() - 1;
        self.driver
            .note_the_published_content_of_the_last_call(PublishedContent::FirstFile(
                content.clone(),
            ));
        if let Ok(output) = &result {
            if self.driver.switches.positive_control == Some(PositiveControl::ReturnValue)
                && self.driver.name == StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount
                && call_index == RETURN_VALUE_CONTROL_CALL
            {
                let mut recorded_copy = output.clone();
                recorded_copy.root.checkpoint_txg =
                    CheckpointTxg(recorded_copy.root.checkpoint_txg.0 + 1);
                self.driver
                    .replace_the_recorded_return_value_of_the_last_call(&debug_text(
                        &recorded_copy,
                    ));
            }
        }
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
                self.record_memory_objects();
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
                self.record_memory_objects();
                false
            }
            Err(None) => {
                self.record_memory_objects();
                false
            }
        }
    }

    /// 覆盖写（`publish_overwrite`，写时刻取本地常量 + 60）：按模型的 `answer_publish_overwrite` 比。
    fn step_overwrite(&mut self, content: &[u8], instance: InstanceGeneration) -> bool {
        let parameters = experiment_pool::parameters();
        let previous = self.output.as_ref().expect("上一版带文件才走到这一步");
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let stream_length_before = self.driver.stream.operation_count();
        let result = self
            .driver
            .call("publish", debug_text::<TransactionOutput>, || {
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
        self.driver
            .note_the_published_content_of_the_last_call(PublishedContent::Overwrite(
                content.to_vec(),
            ));
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
                self.record_memory_objects();
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
                self.record_memory_objects();
                false
            }
            Err(None) => {
                self.record_memory_objects();
                false
            }
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
        let result = self.driver.call(
            "mount_writable",
            debug_text::<singlefs_core::mount::Mounted>,
            || {
                let mut devices = formatted.reopen_recorded();
                let mounted = mount_writable(&parameters, &mut devices);
                formatted.devices = Some(devices);
                mounted
            },
        );
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
                self.mount_output_text = Some(debug_text(&mounted.output));
                self.allocator = mounted.allocator;
                self.output = mounted.current.into_file_version();
                self.record_memory_objects();
                if self.output.is_none() {
                    self.driver.halted = true;
                    self.driver.stopped_without_an_error =
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
                self.record_memory_objects();
                false
            }
            Err(None) => {
                self.record_memory_objects();
                false
            }
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
        let result = self.driver.call(
            "roll_back",
            debug_text::<singlefs_core::mount::RolledBack>,
            || {
                roll_back_by_a_forward_publish(
                    &parameters,
                    devices,
                    &mut self.allocator,
                    &mut *output,
                    target,
                )
            },
        );
        let observed = match &result {
            Ok(_rolled_back) => ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                roots: vec![observed_root_of_file_version(output)],
                reported_ceiling: None,
            }),
            Err(Some(error)) => self.driver.refusal(
                error,
                refusal_reason_of_rollback_error(error),
                0,
                stream_length_before,
                None,
                root_ring_slot_still_bad_after_one_reread_of_rollback_error(error),
            ),
            Err(None) => {
                self.record_memory_objects();
                return false;
            }
        };
        self.driver.judge(
            |model| model.answer_rollback_while_mounted(target_key),
            observed,
        );
        self.record_memory_objects();
        result.is_ok()
    }

    /// 抬 F 到 txg 11（`raise_rollback_floor`，影子账开）：按模型的 `answer_raise_rollback_floor` 比（拒与成都要与实现报的上限相等）。
    fn step_raise_floor(&mut self) -> bool {
        let new_floor = CheckpointTxg(11);
        let parameters = experiment_pool::parameters();
        let mut current = self.output.clone().expect("上一版带文件才走到这一步");
        let txg_before_the_raise = current.root.checkpoint_txg.0;
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let stream_length_before = self.driver.stream.operation_count();
        let result = self.driver.call(
            "raise_floor",
            debug_text::<singlefs_core::mount::RaisedFloor>,
            || {
                raise_rollback_floor(
                    &parameters,
                    devices,
                    &mut self.allocator,
                    &mut current,
                    new_floor,
                    ShadowLedger::On,
                )
            },
        );
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
        let raised_ok = match result {
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
        };
        self.record_memory_objects();
        raised_ok
    }

    /// 正常卸载（`unmount`，入口记在录制流里）：按模型的 `answer_unmount` 比（16-2 加的卸载答；观测 `observed_unmount` 是那几次带新 F
    /// 的空发布的根）。阳性对照「O1-对拍-卸载」把观测里第一条根的 F 加一。
    fn step_unmount(&mut self) -> bool {
        let mut current = PoolVersion::WithFile(std::sync::Arc::new(
            self.output.clone().expect("上一版带文件"),
        ));
        let stream = self.driver.stream.clone();
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let stream_length_before = self.driver.stream.operation_count();
        let result = self
            .driver
            .call(UNMOUNT_CALL_LABEL, unmounted_debug_text, || {
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
        self.output = current.into_file_version();
        let unmounted_ok = match result {
            Ok(unmounted) => {
                let mut observed = observed_unmount(&unmounted);
                if self.driver.switches.positive_control
                    == Some(PositiveControl::ComparisonOfTheUnmount)
                    && self.driver.name == StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount
                {
                    if let ObservedEffect::Publishes { roots, .. } = &mut observed {
                        if let Some(first_root) = roots.first_mut() {
                            first_root.rollback_floor =
                                singlefs_verification::model::ModelCheckpointTxg(
                                    first_root.rollback_floor.0 + 1,
                                );
                        }
                    }
                }
                self.driver.judge(
                    |model| model.answer_unmount(),
                    ObservedOutcome::Succeeded(observed),
                );
                match unmounted {
                    Unmounted::FloorRaisedToTheCurrentVersion(raised) => {
                        self.checks.push(StopCheck::new(
                            "second_stream_unmount_txgs",
                            LOCAL_SECOND_STREAM_UNMOUNT_TXGS.to_vec(),
                            unmounted_txgs_of(&raised),
                        ));
                        true
                    }
                    Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => false,
                }
            }
            Err(Some(error)) => {
                let observed = self.driver.refusal(
                    &error,
                    refusal_reason_of_mount_error(&error),
                    0,
                    stream_length_before,
                    reported_ceiling_of_mount_error(&error),
                    root_ring_slot_still_bad_after_one_reread_of_mount_error(&error),
                );
                self.driver.judge(|model| model.answer_unmount(), observed);
                false
            }
            Err(None) => false,
        };
        self.record_memory_objects();
        unmounted_ok
    }

    /// 流三的一次发布（意图、走空的一步、建一个 inode）：经 `PoolWriter` 调 `publish`，按模型的答比（观测与 `history.rs` 的
    /// `settle_user_change_comparing_every_version_published` 同一种：那一版的根 `observed_root_of_file_version`）。成功时换上现行那一版。
    fn step_third_stream_publish(
        &mut self,
        label: &'static str,
        answer: fn(&IdealModel) -> Result<ModelAnswer, ModelDisagreement>,
        publish: &ThirdStreamPublish<'_>,
    ) -> bool {
        let parameters = experiment_pool::parameters();
        let instance = self.instance.expect("取号成功才走到这一步");
        let previous = self.output.as_ref().expect("上一版接得上才走到这一步");
        let devices = self.formatted.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let stream_length_before = self.driver.stream.operation_count();
        let result = self
            .driver
            .call(label, debug_text::<TransactionOutput>, || {
                publish(&mut writer, &mut self.allocator, previous, instance)
            });
        let published = match result {
            Ok(output) => {
                // 胶水造观测时 panic（`model_comparison::model_unit_role` 里模型还不罩的几种单元：extent 上段根之下的节点、记账树根之下的节点、
                // 墓碑树第二片起的容器与根之下的节点）：接住，记「模型接不了这份观测」、模型停在这次调用。跨出一片 inode 叶容器不再走到这里
                // （理想模型那件修理之后胶水按片记角色，不再断言；登记修订 2 第 2.2 节）。
                let observed_root = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    observed_root_of_file_version(&output)
                }));
                match observed_root {
                    Ok(root) => self.driver.judge(
                        answer,
                        ObservedOutcome::Succeeded(ObservedEffect::Publishes {
                            roots: vec![root],
                            reported_ceiling: None,
                        }),
                    ),
                    Err(payload) => self
                        .driver
                        .note_that_the_model_cannot_take_the_observation(panic_text(&*payload)),
                }
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
                self.driver.judge(answer, observed);
                false
            }
            Err(None) => false,
        };
        self.record_memory_objects();
        published
    }

    /// 流三在第一个文件之后的那一段（读法 2）：删第一个文件——意图一次，再一次一调走空 `deleted_inodes`；之后一次建一个 inode，
    /// 直到 inode 树的叶容器第一次变成两片（那一次是第 K 次），再多建一次。叶容器数读交回值的 `inode_leaf_containers`。
    fn run_the_third_stream_after_the_first_file(&mut self) {
        let write_time_seconds = LOCAL_FIXED_WRITE_TIME_SECONDS + 60;
        let first_inode = InodeNumber(FIRST_INODE_NUMBER);
        if !self.step_third_stream_publish(
            "unlink_intent",
            IdealModel::answer_unlink_intent,
            &|writer, allocator, previous, instance| {
                publish_unlink_intent(
                    writer,
                    allocator,
                    previous,
                    first_inode,
                    write_time_seconds,
                    instance,
                )
            },
        ) {
            return;
        }
        self.unlink_calls = 1;
        // 迭代上界：第一个文件一个数据单元，意图之后一步就是最后一批；留 16 步的余量，到了还没走空就停下记 S1。
        for _ in 0..THIRD_STREAM_UNLINK_STEP_LIMIT {
            let Some(inode) = self
                .output
                .as_ref()
                .and_then(|output| output.deleted_inodes.first().copied())
            else {
                break;
            };
            if !self.step_third_stream_publish(
                "unlink_step",
                IdealModel::answer_deletion_step,
                &|writer, allocator, previous, instance| {
                    publish_unlink_step(
                        writer,
                        allocator,
                        previous,
                        inode,
                        UnlinkBatchSize::OneExtentLowerLeaf,
                        instance,
                    )
                },
            ) {
                return;
            }
            self.unlink_calls += 1;
        }
        // 迭代上界 THIRD_STREAM_INODE_CREATION_LIMIT；跨轮带的是建了几次与第一次变成两片是哪一次；出口：出错、或分裂之后多建了一次。
        while self.inode_creations < THIRD_STREAM_INODE_CREATION_LIMIT {
            if !self.step_third_stream_publish(
                "create_inode",
                IdealModel::answer_create_inode,
                &|writer, allocator, previous, instance| {
                    publish_new_inodes(writer, allocator, previous, 1, write_time_seconds, instance)
                },
            ) {
                return;
            }
            self.inode_creations += 1;
            let leaf_containers = self
                .output
                .as_ref()
                .map_or(0, |output| output.inode_leaf_containers.len());
            if self.split_publish.is_none() && leaf_containers >= 2 {
                self.split_publish = Some(self.inode_creations);
            }
            if self
                .split_publish
                .is_some_and(|split| self.inode_creations == split + 1)
            {
                break;
            }
        }
        let leaf_containers_at_the_end = self
            .output
            .as_ref()
            .map_or(0, |output| output.inode_leaf_containers.len());
        self.checks.extend([
            StopCheck::new(
                "third_stream_unlink_calls",
                LOCAL_THIRD_STREAM_UNLINK_CALLS,
                self.unlink_calls,
            ),
            StopCheck::new(
                "third_stream_split_publish",
                Some(LOCAL_THIRD_STREAM_SPLIT_PUBLISH),
                self.split_publish,
            ),
            StopCheck::new(
                "third_stream_inode_creations_are_the_split_publish_plus_one",
                self.split_publish.map(|split| split + 1),
                Some(self.inode_creations),
            ),
            StopCheck::new(
                "third_stream_leaf_containers_at_the_end",
                2usize,
                leaf_containers_at_the_end,
            ),
        ]);
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
    /// 整条流的脚本：任一步出错或 panic 就停（这条流不再往下驱动、截在那里照判）。流一是取号、暖机、新池新建文件；流二接着照
    /// `tests/crash_enumeration_fixed_script_stream.rs` 的脚本走到 E 再正常卸载；流三接着删第一个文件、建 inode 跨一片叶。
    /// 每一步成功之后才往崩溃点重放手写 oracle 的版本表里记一版（流三只记第一个文件那一版：之后的版本没有文件，手写版本表写不出，那张表只进附带的 `red_items`）。
    pub(super) fn run_the_script(&mut self) {
        if !self.step_acquire_and_warm_up() || !self.step_publish_first_file() {
            return;
        }
        match self.driver.name {
            StreamName::NewPoolFileCreation => return,
            StreamName::FirstFileUnlinkedThenInodesAcrossOneLeaf => {
                self.versions
                    .push(version(1, 3, local_first_file_content()));
                self.run_the_third_stream_after_the_first_file();
                return;
            }
            StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount => {}
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

    /// Q5 那一形（读法 7）要的流二前四次调用：取号、暖机、第一个文件（第 2 次调用，模型跟到这一次）、第一次覆盖写（第 3 次调用，
    /// 实现照常跑完、写流照录，模型不问——那一形里它在途）。与基线流二前四次调用逐条同一个驱动。
    pub(super) fn run_the_second_stream_through_the_first_overwrite(&mut self) {
        assert_eq!(
            self.driver.name,
            StreamName::FixedScriptReuseAfterRaisingFloorThenUnmount
        );
        if !self.step_acquire_and_warm_up() || !self.step_publish_first_file() {
            return;
        }
        self.versions
            .push(version(1, 3, local_first_file_content()));
        self.driver.model_not_asked_from_call = Some(CONFIRMED_LOSS_IN_FLIGHT_CALL);
        let _ = self.step_overwrite(&second_content(), InstanceGeneration(1));
    }
}

#[cfg(test)]
mod tests;

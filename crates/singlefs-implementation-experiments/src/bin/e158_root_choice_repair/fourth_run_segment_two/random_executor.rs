//! 装置自己的随机历史执行器：八种操作照 harness 执行器同一串入口、同一套前提各写一遍，checker 每一步的判定原样记下。

use super::*;

/// checker 在一步之后的判定：红的不变量与 I-7.4 被抛弃根那一半点名的根（去重的一份，与按违例细节里的次序、重复的照留的一份）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CheckerReading {
    pub(super) red_invariants: Vec<&'static str>,
    pub(super) abandoned_roots_named: Vec<TimelineRoot>,
    pub(super) abandoned_roots_named_in_the_order_of_the_detail: Vec<TimelineRoot>,
}

impl CheckerReading {
    pub(super) fn abandoned_half_is_red(&self) -> bool {
        !self.abandoned_roots_named.is_empty()
    }
}

/// 一张镜像上池级 checker 的判定，交崩溃点重放的给定状态引擎两段判（「GPU 判器加细节输出」：GPU 判器与判定库的结论
/// 带出 I-7.4 点名的被抛弃根，这一处就不再在 CPU 上就地跑池级 checker）。镜像建成一张给定状态表：基线就是这张镜像、写表为空、一个空的持久集合
/// （池级 checker 判的就是这张镜像本身，记录核对器在空写表上没有发布可判）；录入段写进家目录下给定状态表那一类的判定库、判段从库里读回判
/// （配了 GPU 且编进 `gpu` 特性在卡上判，否则 CPU 判器判），判过的复用。表名带镜像指纹（[`image_fingerprint`]）：同一趟里内容不同的两张镜像
/// 不同名（同名而内容不同的两张表，引擎录入段报错）。
/// 红的不变量按 `IMPLEMENTED_INVARIANTS` 的次序（与池级 checker 交回的次序相同）；点名的根取结论里 I-7.4 违例细节点名的那一栏
/// （`crash_given_states::GivenStateConclusion::abandoned_roots_named_by_the_pool_checker`，按细节里的次序、重复的照留），去重的那一份在这里去重。
///
/// # Panics
/// 同 [`super::given_states_judged_on_this_host_through_the_engine`]。
pub(super) fn checker_reading_of(engine: &GivenStatesEngine, image: &MemoryPool) -> CheckerReading {
    let table = GivenStatesOfATable {
        table_name: format!(
            "checker-reading/image-fingerprint={:016x}",
            image_fingerprint(image)
        ),
        base: image,
        writes: &[],
        segments: &[],
        seams: &[],
        persisted_sets: vec![Vec::new()],
    };
    let mut states = given_states_judged_on_this_host_through_the_engine(engine, &table);
    assert_eq!(states.len(), 1, "一张镜像一个状态，判器交回一份结论");
    checker_reading_of_the_conclusion(&states.pop().expect("上一句核过恰好一份结论"))
}

/// 一个状态的结论换成这一段记的判定（[`checker_reading_of`] 的后一半，纯函数）：红的不变量照结论的违例位按 `IMPLEMENTED_INVARIANTS` 的次序；
/// I-7.4 点名的根按细节里的次序、重复的照留一份，去重（第一次出现的次序）另一份。
fn checker_reading_of_the_conclusion(
    conclusion: &singlefs_verification::crash_given_states::GivenStateConclusion,
) -> CheckerReading {
    let red_invariants: Vec<&'static str> = conclusion
        .pool_checker_violations()
        .into_iter()
        .map(|(invariant, _detail)| invariant)
        .collect();
    let abandoned_roots_named_in_the_order_of_the_detail: Vec<TimelineRoot> = conclusion
        .abandoned_roots_named_by_the_pool_checker
        .iter()
        .map(|named| (named.instance.0, named.checkpoint_txg.0))
        .collect();
    let mut abandoned_roots_named: Vec<TimelineRoot> = Vec::new();
    for root in &abandoned_roots_named_in_the_order_of_the_detail {
        if !abandoned_roots_named.contains(root) {
            abandoned_roots_named.push(*root);
        }
    }
    CheckerReading {
        red_invariants,
        abandoned_roots_named,
        abandoned_roots_named_in_the_order_of_the_detail,
    }
}

/// 每一步写过盘之后池级 checker 那一遍怎么判（登记 5.3：要它的那几族每一步跑池级 checker，[`SegmentTwoFamily::checks_every_step`]）。
#[derive(Clone, Copy)]
pub(super) enum PoolCheckerOnEveryStep<'engine> {
    /// 这一族不逐步跑池级 checker（H-随全、注入分支）。
    Skipped,
    /// 每张镜像交给定状态引擎判（[`checker_reading_of`]）。
    ThroughTheGivenStatesEngine(&'engine GivenStatesEngine),
    /// 只供测试：判定换成测试给的函数（不跑池级 checker），钉与判法无关的执行器行为（「判红不截断」，变异 M22）。
    #[cfg(test)]
    ReadingsGivenByATest(fn(&MemoryPool) -> CheckerReading),
}

impl<'engine> PoolCheckerOnEveryStep<'engine> {
    /// 这一族照 [`SegmentTwoFamily::checks_every_step`]：要就交 `engine` 判，不要就不跑。
    pub(super) fn of_the_family(
        family: SegmentTwoFamily,
        engine: &'engine GivenStatesEngine,
    ) -> Self {
        if family.checks_every_step() {
            PoolCheckerOnEveryStep::ThroughTheGivenStatesEngine(engine)
        } else {
            PoolCheckerOnEveryStep::Skipped
        }
    }

    fn runs_on_every_step(self) -> bool {
        match self {
            PoolCheckerOnEveryStep::Skipped => false,
            PoolCheckerOnEveryStep::ThroughTheGivenStatesEngine(_) => true,
            #[cfg(test)]
            PoolCheckerOnEveryStep::ReadingsGivenByATest(_) => true,
        }
    }

    /// 这张镜像的判定；这一族不逐步跑池级 checker 时 None。
    fn reading_of(self, image: &MemoryPool) -> Option<CheckerReading> {
        match self {
            PoolCheckerOnEveryStep::Skipped => None,
            PoolCheckerOnEveryStep::ThroughTheGivenStatesEngine(engine) => {
                Some(checker_reading_of(engine, image))
            }
            #[cfg(test)]
            PoolCheckerOnEveryStep::ReadingsGivenByATest(reading_given_by_the_test) => {
                Some(reading_given_by_the_test(image))
            }
        }
    }
}

/// 一步的记录：操作、结局、这一步之后的镜像指纹、checker 的判定（这一族不跑 checker 或这一步没写盘时 None）。
pub(super) struct RandomStepRecord {
    pub(super) order: usize,
    pub(super) operation: String,
    pub(super) outcome: String,
    pub(super) fingerprint_after: u64,
    pub(super) checker: Option<CheckerReading>,
    pub(super) read_back: Option<RandomReadBack>,
}

/// 一次崩溃恢复抛弃根那一步（登记 5.3：V_LC、Q1、Q2 按每一次这一步各算一份）：被藏那条根、那一步开始与交回时的镜像。
pub(super) struct CrashRecoveryEvent {
    pub(super) order: usize,
    pub(super) hidden_root: TimelineRoot,
    pub(super) hidden_content_number: Option<u64>,
    pub(super) base: MemoryPool,
    pub(super) after: MemoryPool,
    pub(super) mount_index: usize,
}

/// 一段历史在一臂上的一次执行（一个分支：主路径、一个注入点、一个崩溃状态之前的主路径都是它）。
pub(super) struct RandomExecutor<'engine> {
    pub(super) arm: FourthRunArm,
    pub(super) parameters: MakeFilesystemParameters,
    pub(super) devices: Vec<(DeviceIdentity, HistoryDevice)>,
    pub(super) stream: SharedStream,
    pub(super) plan: SharedFaultPlan,
    session: Option<RandomSession>,
    pub(super) ledger: RandomLedger,
    pub(super) mounts: Vec<RandomMount>,
    pub(super) steps: Vec<RandomStepRecord>,
    /// 下标 = 步（起点 0）：那一步开始时录制流有几步。
    pub(super) stream_length_at_step_start: Vec<usize>,
    /// 下标 = 步：那一步做完时整池发出的读 / 写 / 屏障数（注入计划数的，与 harness 的测量跑同一数法）。
    pub(super) calls_after_step: Vec<FaultCallCounts>,
    pub(super) crash_recovery_events: Vec<CrashRecoveryEvent>,
    /// 跑 checker 的那几族：最后一次跑 checker 时录制流有几步（harness 只在这一步写过盘时跑）。
    checked_stream_length: usize,
    pub(super) pool_checker_on_every_step: PoolCheckerOnEveryStep<'engine>,
    pub(super) panicked: Option<(usize, String)>,
    pub(super) stopped_at_the_first_red: Option<usize>,
    pub(super) starting_point_error: Option<String>,
    /// 内容号 → 那一版引用的单元第一次被改的那一步（Q1 的时点栏，第十二节第二段修订）。
    pub(super) first_changes: BTreeMap<u64, FirstChange>,
    /// 要留下哪一步做完时的镜像（注入点那一步：重开读回在它上面做）。
    pub(super) capture_after_order: Option<usize>,
    pub(super) captured_image: Option<MemoryPool>,
    /// 冷启动那一步的读回（经这一叠盘读，调用数与 harness 同一数法）。
    pending_read_back: Option<RandomReadBack>,
}

/// 一版引用的单元第一次被改：那一步、那一刻它之后已经有几笔改内容的确认、它的根那一刻还在不在根环里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FirstChange {
    pub(super) order: usize,
    pub(super) later_confirmations: usize,
    pub(super) root_in_ring_then: bool,
}

/// 前提不满足的结局（harness `MissingPrecondition` 的成员名）。
fn not_applicable(precondition: &str) -> String {
    format!("not_applicable:{precondition}")
}

fn refused(error_debug: &str) -> String {
    format!("refused:{}", error_member_of_debug(error_debug))
}

impl<'engine> RandomExecutor<'engine> {
    /// 起点（harness `HistoryPool::start` 的同一串）：mkfs；`AfterFirstFile` 再在同一个进程里取号、暖机、发第一个文件（内容 `first_file_content`）。
    pub(super) fn start(
        arm: FourthRunArm,
        starting_point: HistoryStartingPoint,
        plan: SharedFaultPlan,
        pool_checker_on_every_step: PoolCheckerOnEveryStep<'engine>,
    ) -> Self {
        let parameters = HistoryDeviceWidth::FourGibibytes.parameters();
        let stream = SharedStream::retaining_contents();
        let mut devices: Vec<(DeviceIdentity, HistoryDevice)> =
            [DeviceIdentity(0), DeviceIdentity(1)]
                .into_iter()
                .map(|identity| {
                    (
                        identity,
                        FaultInjectingBlockDevice::new(
                            identity,
                            RecordingBlockDevice::with_shared_stream(
                                identity,
                                SparseBlockDevice::new(
                                    history_device_bytes(),
                                    PhysicalBlockSizeInBytes(512),
                                ),
                                stream.clone(),
                            ),
                            plan.clone(),
                        ),
                    )
                })
                .collect();
        let mut executor = RandomExecutor {
            arm,
            parameters: parameters.clone(),
            devices: Vec::new(),
            stream: stream.clone(),
            plan: plan.clone(),
            session: None,
            ledger: RandomLedger::default(),
            mounts: Vec::new(),
            steps: Vec::new(),
            stream_length_at_step_start: vec![0],
            calls_after_step: Vec::new(),
            crash_recovery_events: Vec::new(),
            checked_stream_length: 0,
            pool_checker_on_every_step,
            panicked: None,
            stopped_at_the_first_red: None,
            starting_point_error: None,
            first_changes: BTreeMap::new(),
            capture_after_order: None,
            captured_image: None,
            pending_read_back: None,
        };
        let genesis = match make_filesystem(&parameters, &mut devices) {
            Ok(genesis) => genesis,
            Err(error) => {
                executor.devices = devices;
                executor.starting_point_error = Some(format!("make_filesystem:{error:?}"));
                executor.finish_the_starting_point();
                return executor;
            }
        };
        executor.ledger = RandomLedger::after_make_filesystem(root_pair(&genesis.root));
        match starting_point {
            HistoryStartingPoint::AfterMakeFilesystem => {}
            HistoryStartingPoint::AfterFirstFile => {
                match Self::acquire_warm_up_and_publish_the_first_file(
                    &parameters,
                    &mut devices,
                    &genesis,
                ) {
                    Ok((session, warm_up_roots)) => {
                        for root in warm_up_roots {
                            executor.ledger.carry(root, 0);
                        }
                        let first_file_root = *session.current.root();
                        executor.devices = devices;
                        let image = executor.image();
                        executor
                            .ledger
                            .confirm(0, &first_file_root, first_file_content(), &image);
                        executor.session = Some(session);
                        executor.finish_the_starting_point();
                        return executor;
                    }
                    Err(error) => executor.starting_point_error = Some(error),
                }
            }
        }
        executor.devices = devices;
        executor.finish_the_starting_point();
        executor
    }

    fn acquire_warm_up_and_publish_the_first_file(
        parameters: &MakeFilesystemParameters,
        devices: &mut Vec<(DeviceIdentity, HistoryDevice)>,
        genesis: &singlefs_core::make_filesystem::MakeFilesystemOutput,
    ) -> Result<(RandomSession, Vec<TimelineRoot>), String> {
        let mut allocator = allocator_after_make_filesystem(parameters, devices, genesis);
        allocator.set_space_admission(SpaceAdmission::JudgedByTheFormula);
        let parameters_and_device_table =
            ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(devices)
                .map_err(|error| format!("read_the_pool_for_the_session:{error:?}"))?;
        let content = first_file_content();
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        let instance =
            acquire_instance(&mut writer).map_err(|error| format!("acquire_instance:{error:?}"))?;
        let warmed = warm_up(&mut writer, &genesis.root, instance)
            .map_err(|error| format!("warm_up:{error:?}"))?;
        let warm_up_roots: Vec<TimelineRoot> = warmed.roots.iter().map(root_pair).collect();
        let output = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &content,
                write_time_seconds: WRITE_TIME_SECONDS_OF_THE_HARNESS_HISTORIES,
            },
            instance,
            &warmed.last_record_bytes,
        )
        .map_err(|error| format!("publish_first_file:{error:?}"))?;
        Ok((
            RandomSession {
                allocator,
                current: PoolVersion::WithFile(std::sync::Arc::new(output)),
                instance,
                shadow_ledger: ShadowLedger::On,
                parameters_and_device_table,
            },
            warm_up_roots,
        ))
    }

    fn finish_the_starting_point(&mut self) {
        self.calls_after_step.push(self.plan.calls());
        let fingerprint_after = if self.devices.is_empty() {
            0
        } else {
            image_fingerprint(&self.image())
        };
        let checker =
            if self.pool_checker_on_every_step.runs_on_every_step() && !self.devices.is_empty() {
                self.checked_stream_length = self.stream.operation_count();
                self.pool_checker_on_every_step.reading_of(&self.image())
            } else {
                None
            };
        self.steps.push(RandomStepRecord {
            order: 0,
            operation: "starting_point".to_string(),
            outcome: self
                .starting_point_error
                .clone()
                .map_or("applied".to_string(), |error| format!("refused:{error}")),
            fingerprint_after,
            checker,
            read_back: None,
        });
    }

    pub(super) fn image(&self) -> MemoryPool {
        image_of_history_devices(&self.devices)
    }

    /// 现行那一版的内容号（没有会话时取字面读法的最后确认那一版）。
    fn carried_content_number(&self, order: usize) -> u64 {
        self.session
            .as_ref()
            .and_then(|session| {
                self.ledger
                    .content_number_of(root_pair(session.current.root()))
            })
            .unwrap_or_else(|| self.ledger.last_confirmed_number_before(order))
    }

    /// 一步做完之后根环里有、账里还没有的根（发布报错或挂载半路报错时已经落盘的那几条）按 `default_number` 记内容号。
    fn carry_new_ring_roots(&mut self, default_number: u64) {
        let image = self.image();
        let Ok(geometry) = independent_geometry(&image) else {
            return;
        };
        for root in readable_roots_independent(&image, &geometry) {
            if self.ledger.content_number_of(root).is_none() {
                self.ledger.carry(root, default_number);
            }
        }
    }

    /// 跑一段历史：逐步执行；某一步 panic ⇒ 到此为止（登记 5.3）；M22 那个开关开着时判红就停。
    pub(super) fn run_the_operations(&mut self, history: &GeneratedHistory) {
        if self.starting_point_error.is_some() {
            return;
        }
        for (step_index, operation) in history.operations.iter().enumerate() {
            let order = step_index + 1;
            self.stream_length_at_step_start
                .push(self.stream.operation_count());
            let applied = with_panic_capture(|| self.apply_operation(order, step_index, operation));
            let outcome = match applied {
                Ok(outcome) => outcome,
                Err(panic) => {
                    let text = format!("{}: {}", panic.location, panic.message);
                    self.panicked = Some((order, text.clone()));
                    self.calls_after_step.push(self.plan.calls());
                    self.steps.push(RandomStepRecord {
                        order,
                        operation: format!("{:?}", operation.kind()),
                        outcome: format!("panic:{text}"),
                        fingerprint_after: 0,
                        checker: None,
                        read_back: None,
                    });
                    return;
                }
            };
            self.calls_after_step.push(self.plan.calls());
            let default_number = self.carried_content_number(order);
            self.carry_new_ring_roots(default_number);
            let image = self.image();
            let stream_length = self.stream.operation_count();
            let checker = if self.pool_checker_on_every_step.runs_on_every_step()
                && stream_length != self.checked_stream_length
            {
                self.checked_stream_length = stream_length;
                self.pool_checker_on_every_step.reading_of(&image)
            } else {
                None
            };
            let red = checker
                .as_ref()
                .is_some_and(|reading| !reading.red_invariants.is_empty());
            let read_back = self.pending_read_back.take();
            self.after_a_step(order, &image);
            self.steps.push(RandomStepRecord {
                order,
                operation: format!("{:?}", operation.kind()),
                outcome,
                fingerprint_after: image_fingerprint(&image),
                checker,
                read_back,
            });
            if RANDOM_HISTORIES_STOP_AT_THE_FIRST_CHECKER_RED && red {
                self.stopped_at_the_first_red = Some(order);
                return;
            }
        }
    }

    fn apply_operation(
        &mut self,
        order: usize,
        step_index: usize,
        operation: &HistoryOperation,
    ) -> String {
        let write_time_seconds = WRITE_TIME_SECONDS_OF_THE_HARNESS_HISTORIES
            + u64::try_from(step_index).expect("步号装得进 u64");
        match operation {
            HistoryOperation::PublishFirstFile(content) => {
                self.apply_publish_first_file(order, content, write_time_seconds)
            }
            HistoryOperation::PublishOverwrite(content) => {
                self.apply_publish_overwrite(order, content, write_time_seconds)
            }
            HistoryOperation::PublishWithoutUnits => self.apply_publish_without_units(order),
            HistoryOperation::CloseAndMountWritable => {
                self.session = None;
                self.observed_mount(order, &order.to_string(), "mount", &[])
            }
            HistoryOperation::RollBackWhileMounted(choice) => {
                self.apply_rollback_while_mounted(order, *choice)
            }
            HistoryOperation::RaiseRollbackFloor(choice) => {
                self.apply_raise_rollback_floor(order, *choice)
            }
            HistoryOperation::ColdStartRecover => {
                self.session = None;
                let report = recover(&self.devices, JournalPolicy::Consult);
                self.pending_read_back = Some(classify_random_read_back(
                    &report,
                    &self.ledger,
                    self.ledger.last_confirmed_number_before(order),
                    self.ledger
                        .last_confirmed_number_before_rollback_aware(order),
                ));
                "applied:recovered".to_string()
            }
            HistoryOperation::CrashRecoveryAbandoningTheNewestRoot => {
                self.apply_crash_recovery_abandoning_the_newest_root(order)
            }
            // 这个装置的执行器不发 unlink（三·2 第 8 条把它加进随机历史的生成；这里的格子都是删除之前的那几种）：照前提不满足记。
            HistoryOperation::Unlink => not_applicable("UnlinkIsNotDrivenByThisExperiment"),
            // 建 inode 只接在 unlink 删完之后（三·2 后续把它加进随机历史的生成），这个装置不发 unlink，同样照前提不满足记。
            HistoryOperation::CreateInode => {
                not_applicable("CreateInodeIsNotDrivenByThisExperiment")
            }
            // 掉电在 journal 记录之前的那次覆盖写只在留孤儿那一组比重里抽（实例表清扫抹头那一段的取样点），这个装置的格子不用那组比重：
            // 照前提不满足记。
            HistoryOperation::OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable(
                _,
            ) => not_applicable("OverwriteLosingPowerIsNotDrivenByThisExperiment"),
            // 每块盘最新那一槽系统配置读回全 0 的可写挂载只进 C696 那一组比重，这个装置用的几组比重不抽它：照前提不满足记。
            HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros => {
                not_applicable(
                    "WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZerosIsNotDrivenByThisExperiment",
                )
            }
            // 只有第一块盘最新那一槽系统配置读回全 0 的可写挂载（C696）只进它自己那一组比重，这个装置用的几组比重不抽它：照前提不满足记。
            HistoryOperation::WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZeros => {
                not_applicable(
                    "WritableMountWithTheNewestSystemConfigurationSlotOfOneDeviceReadingZerosIsNotDrivenByThisExperiment",
                )
            }
            // 建快照、读快照、删快照只进它们自己那一组比重（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S7，
            // `GenerationWeights::SNAPSHOT_OPERATIONS`），这个装置用的几组比重不抽它们：照前提不满足记。
            HistoryOperation::CreateSnapshot => {
                not_applicable("CreateSnapshotIsNotDrivenByThisExperiment")
            }
            HistoryOperation::ReadSnapshot(_) => {
                not_applicable("ReadSnapshotIsNotDrivenByThisExperiment")
            }
            HistoryOperation::DestroySnapshot(_) => {
                not_applicable("DestroySnapshotIsNotDrivenByThisExperiment")
            }
            // 挂着时整理只进它自己那一组比重（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7，
            // `GenerationWeights::COMPACTION_WHILE_MOUNTED`），这个装置用的几组比重不抽它：照前提不满足记。
            HistoryOperation::CompactWhileMounted(_) => {
                not_applicable("CompactWhileMountedIsNotDrivenByThisExperiment")
            }
            // 截断只进它自己那一组比重与带整理的那一组（判决 `research/prompts/m4-trunc-r1-main-verification.md` 第三节 K8，
            // `GenerationWeights::TRUNCATION`），这个装置用的几组比重不抽它：照前提不满足记。
            HistoryOperation::Truncate(_) => {
                not_applicable("TruncateIsNotDrivenByThisExperiment")
            }
            // 建几个 inode、按 inode 号写只进它们自己那一组比重（里程碑「删除与多次 COW」一·1，
            // `GenerationWeights::WRITES_BY_INODE_NUMBER`），这个装置用的几组比重不抽它们：照前提不满足记。
            HistoryOperation::CreateInodes(_) => {
                not_applicable("CreateInodesIsNotDrivenByThisExperiment")
            }
            HistoryOperation::WriteOfAnInode(_) => {
                not_applicable("WriteOfAnInodeIsNotDrivenByThisExperiment")
            }
            // 按偏移写只进它自己那一组比重（判决 `research/prompts/m4-partial-write-r1-main-verification.md` 第二节，
            // `GenerationWeights::PARTIAL_WRITES`），这个装置用的几组比重不抽它：照前提不满足记。
            HistoryOperation::WriteAtOffset(_) => {
                not_applicable("WriteAtOffsetIsNotDrivenByThisExperiment")
            }
        }
    }

    fn apply_publish_first_file(
        &mut self,
        order: usize,
        choice: &ContentChoice,
        write_time_seconds: u64,
    ) -> String {
        let RandomExecutor {
            parameters,
            devices,
            session,
            ..
        } = self;
        let Some(session) = session.as_mut() else {
            return not_applicable("NoWritableSession");
        };
        let content = choice.bytes();
        let root_to_carry_instance_table_from = *session.current.root();
        let previous_record_bytes = session.current.record_bytes().to_vec();
        let published = {
            let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
            publish_first_file(
                &mut writer,
                &mut session.allocator,
                &root_to_carry_instance_table_from,
                FirstFile {
                    content: &content,
                    write_time_seconds,
                },
                session.instance,
                &previous_record_bytes,
            )
        };
        match published {
            Ok(output) => {
                let root = output.root;
                session.current = PoolVersion::WithFile(std::sync::Arc::new(output));
                let image = self.image();
                self.ledger.confirm(order, &root, content, &image);
                "applied:published".to_string()
            }
            Err(error) => {
                let attempted = self.ledger.record_attempt(content);
                self.carry_new_ring_roots(attempted);
                refused(&format!("{error:?}"))
            }
        }
    }

    fn apply_publish_overwrite(
        &mut self,
        order: usize,
        choice: &ContentChoice,
        write_time_seconds: u64,
    ) -> String {
        let Some(open_session) = self.session.as_ref() else {
            return not_applicable("NoWritableSession");
        };
        if open_session.current.file_version().is_none() {
            return not_applicable("CurrentVersionWithoutFile");
        }
        let carried = self
            .ledger
            .content_number_of(root_pair(open_session.current.root()))
            .unwrap_or_else(|| self.ledger.last_confirmed_number_before(order));
        let content = choice.bytes();
        let RandomSession {
            allocator,
            current,
            instance,
            shadow_ledger,
            parameters_and_device_table,
        } = self.session.take().expect("上面判过这个进程的会话开着");
        let mut mounted_session = MountedSession {
            allocator,
            current,
            instance,
            shadow_ledger,
            parameters_and_device_table,
            compaction: singlefs_core::compaction_plan::CompactionOfThisMount::of_a_new_mount(),
        };
        let change = mounted_session.publish_user_change(
            &mut self.devices,
            UserChange::Overwrite(FirstFile {
                content: &content,
                write_time_seconds,
            }),
        );
        let current_root = *mounted_session.current.root();
        self.session = Some(RandomSession {
            allocator: mounted_session.allocator,
            current: mounted_session.current,
            instance: mounted_session.instance,
            shadow_ledger: mounted_session.shadow_ledger,
            parameters_and_device_table: mounted_session.parameters_and_device_table,
        });
        let floor_raises: &[singlefs_core::mount::RaisedFloor] = match &change {
            Ok(published) => &published.floor_raises,
            Err(UserChangeRefused::NoFileVersionToChange { floor_raises, .. }) => floor_raises,
            Err(
                UserChangeRefused::DeviceTableOtherThanTheOneOfTheMount { .. }
                | UserChangeRefused::PhysicalBlockSizesProbedOnTheDevicesDifferFromTheSystemConfiguration {
                    ..
                }
                | UserChangeRefused::DevicesWithoutASelfVerifiedSystemConfiguration { .. }
                | UserChangeRefused::OwnDeviceNumbersDifferFromTheDeviceTable { .. }
                | UserChangeRefused::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot {
                    ..
                },
            ) => &[],
            Err(UserChangeRefused::Publish { floor_raises, .. }) => floor_raises,
            Err(UserChangeRefused::NoSpaceAfterRaisingTheFloor(refused_change)) => {
                &refused_change.floor_raises
            }
            Err(UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease(
                refused_change,
            )) => &refused_change.floor_raises,
            Err(UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(refused_change)) => {
                &refused_change.floor_raises
            }
        };
        let floor_raise_roots: Vec<TimelineRoot> = floor_raises
            .iter()
            .flat_map(|raised| {
                raised
                    .publishes
                    .iter()
                    .map(|publish| root_pair(&publish.root))
            })
            .collect();
        for root in floor_raise_roots {
            self.ledger.carry(root, carried);
        }
        match change {
            Ok(_) => {
                let image = self.image();
                self.ledger.confirm(order, &current_root, content, &image);
                "applied:published".to_string()
            }
            Err(error) => {
                self.carry_new_ring_roots(carried);
                let attempted = self.ledger.record_attempt(content);
                self.carry_new_ring_roots(attempted);
                refused(&format!("{error:?}"))
            }
        }
    }

    fn apply_publish_without_units(&mut self, order: usize) -> String {
        let carried = self.carried_content_number(order);
        let RandomExecutor {
            parameters,
            devices,
            session,
            ledger,
            ..
        } = self;
        let Some(session) = session.as_mut() else {
            return not_applicable("NoWritableSession");
        };
        let PoolVersion::WithoutFile(previous) = &session.current else {
            return not_applicable("CurrentVersionWithFile");
        };
        let plan = ZeroUnitPublishPlan {
            txg: CheckpointTxg(previous.root.checkpoint_txg.0 + 1),
            counter: previous.record.counter + 1,
            instance: session.instance,
            back_chain: back_chain_of(&previous.record_bytes),
            rollback_floor: previous.root.rollback_floor,
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
        };
        let previous_root = previous.root;
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        match publish_without_units(&mut writer, &previous_root, plan) {
            Ok(output) => {
                ledger.carry(root_pair(&output.root), carried);
                session.current = PoolVersion::WithoutFile(output);
                "applied:published_without_units".to_string()
            }
            Err(error) => refused(&format!("{error:?}")),
        }
    }

    fn apply_rollback_while_mounted(
        &mut self,
        order: usize,
        choice: RollbackTargetChoice,
    ) -> String {
        let image_before = self.image();
        let RandomExecutor {
            parameters,
            devices,
            session,
            ledger,
            ..
        } = self;
        let Some(session) = session.as_mut() else {
            return not_applicable("NoWritableSession");
        };
        let PoolVersion::WithFile(current) = &mut session.current else {
            return not_applicable("CurrentVersionWithoutFile");
        };
        let current = std::sync::Arc::make_mut(current);
        let roots = ring_roots_newest_first_of_the_image(&image_before);
        let Some((newest_txg, newest_instance)) = roots.first().copied() else {
            return not_applicable("NoReadableRootInRing");
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
                let floor = newest_ring_root_floor_of_the_image(&image_before);
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
        let target_key = (target.instance.0, target.checkpoint_txg.0);
        match roll_back_by_a_forward_publish(
            parameters,
            devices,
            &mut session.allocator,
            current,
            target,
        ) {
            Ok(_) => {
                let root = root_pair(&current.root);
                let number = ledger
                    .content_number_of(target_key)
                    .unwrap_or_else(|| ledger.last_confirmed_number_before(order));
                ledger.carry(root, number);
                ledger.rollback_confirmations.push((order, number, root));
                format!("applied:rolled_back_to:{}", key_text(Some(target_key)))
            }
            Err(error) => refused(&format!("{error:?}")),
        }
    }

    fn apply_raise_rollback_floor(&mut self, order: usize, choice: FloorTargetChoice) -> String {
        let carried = self.carried_content_number(order);
        let RandomExecutor {
            parameters,
            devices,
            session,
            ledger,
            ..
        } = self;
        let Some(session) = session.as_mut() else {
            return not_applicable("NoWritableSession");
        };
        let PoolVersion::WithFile(current) = &mut session.current else {
            return not_applicable("CurrentVersionWithoutFile");
        };
        let current = std::sync::Arc::make_mut(current);
        let current_floor = current.root.rollback_floor.0;
        let txg_before_raise = current.root.checkpoint_txg.0;
        let choices = txg_before_raise.saturating_sub(current_floor) + 3;
        let new_floor = CheckpointTxg(current_floor + choice.steps_above_current_floor % choices);
        match raise_rollback_floor(
            parameters,
            devices,
            &mut session.allocator,
            current,
            new_floor,
            ShadowLedger::On,
        ) {
            Ok(raised) => {
                for publish in &raised.publishes {
                    ledger.carry(root_pair(&publish.root), carried);
                }
                format!("applied:raised_floor_to:{}", new_floor.0)
            }
            Err(error) => refused(&format!("{error:?}")),
        }
    }

    fn apply_crash_recovery_abandoning_the_newest_root(&mut self, order: usize) -> String {
        let Some(session) = self.session.as_ref() else {
            return not_applicable("NoWritableSession");
        };
        let image_before = self.image();
        let roots = ring_roots_newest_first_of_the_image(&image_before);
        let current_root = session.current.root();
        if roots.first().copied() != Some((current_root.checkpoint_txg.0, current_root.instance.0))
        {
            return not_applicable("NewestRingRootIsNotTheCurrentVersionOfThisSession");
        }
        if roots.len() < 2 {
            return not_applicable("NoReadableRootBeforeTheNewest");
        }
        let Some(hidden) = ranges_hidden_by_the_crash_recovery(&self.parameters, &session.current)
        else {
            return not_applicable("NewestPublishNamesNoUnit");
        };
        let hidden_root = root_pair(current_root);
        let hidden_content_number = self.ledger.content_number_of(hidden_root);
        self.session = None;
        let outcome = self.observed_mount(order, &order.to_string(), "crash_recovery", &hidden);
        let after = self.image();
        self.crash_recovery_events.push(CrashRecoveryEvent {
            order,
            hidden_root,
            hidden_content_number,
            base: image_before,
            after,
            mount_index: self.mounts.len() - 1,
        });
        outcome
    }

    /// 这一次可写挂载落在哪一段（登记 5.3：第一次崩溃恢复抛弃根之前的挂载是合法状态，另算进 Q4、Q5-同成）。
    fn phase_of_a_mount(&self, operation: &'static str) -> &'static str {
        if operation == "crash_recovery" {
            "crash_recovery"
        } else if self.crash_recovery_events.is_empty() {
            "before_first_crash_recovery"
        } else {
            "after_crash_recovery"
        }
    }

    /// 一次受观测的可写挂载（`hidden` 那几段读回全 0）：记进 `mounts`，交回的是这一步的结局。
    fn observed_mount(
        &mut self,
        order: usize,
        step_label: &str,
        operation: &'static str,
        hidden: &[HiddenRange],
    ) -> String {
        let base = self.image();
        let parameters = self.parameters.clone();
        let (mounted, reading) = observed_call(&mut self.devices, hidden, true, |devices| {
            mount_writable_with_space_admission(
                &parameters,
                devices,
                SpaceAdmission::JudgedByTheFormula,
            )
        });
        let phase = self.phase_of_a_mount(operation);
        let record = evaluate_a_random_mount(
            self.arm,
            &self.ledger,
            &base,
            order,
            step_label,
            operation,
            phase,
            hidden,
            &mounted,
            &reading,
        );
        let outcome = match &mounted {
            Ok(_) => format!("applied:mounted:{}", record.summary.class),
            Err(error) => refused(&format!("{error:?}")),
        };
        let carried = record.effective_content.unwrap_or(record.literal_last);
        self.mounts.push(record);
        match mounted {
            Ok(mounted) => {
                let mut roots = vec![root_pair(mounted.output.row_publish.root())];
                roots.extend(
                    mounted
                        .output
                        .warm_up_publishes
                        .iter()
                        .map(|publish| root_pair(publish.root())),
                );
                for raised in mounted.output.space_admission.floor_raises() {
                    roots.extend(
                        raised
                            .publishes
                            .iter()
                            .map(|publish| root_pair(&publish.root)),
                    );
                }
                for root in roots {
                    self.ledger.carry(root, carried);
                }
                self.session = Some(RandomSession::of_the_mount(mounted));
            }
            Err(_) => {
                let device_carried = self
                    .mounts
                    .last()
                    .and_then(|last_mount| last_mount.device_effective_content)
                    .unwrap_or(carried);
                self.carry_new_ring_roots(device_carried);
            }
        }
        outcome
    }
}

fn summary_of_a_random_mount(
    mounted: &Result<Mounted, singlefs_core::mount::MountError>,
    reading: &ObservedCallReading,
) -> TestedMountSummary {
    let ok = mounted.as_ref().ok();
    TestedMountSummary {
        class: mount_outcome_class(mounted, reading.window.write_calls),
        member: mount_error_member_or_none(mounted),
        writes: reading.window.write_calls,
        reads: reading.window.read_calls,
        read_bytes: reading.window.read_bytes,
        waits: reading.waits,
        chosen: ok.map(|succeeded| root_pair(&succeeded.output.chosen_root)),
        effective: ok.map(|succeeded| root_pair(&succeeded.output.effective_root)),
        abandoned_roots_unreadable: ok.map(|succeeded| succeeded.output.abandoned_roots_unreadable),
        isolated: ok.map(|succeeded| {
            succeeded
                .output
                .isolated_slots_per_device
                .iter()
                .map(|(_, slots)| *slots)
                .sum()
        }),
        observation: reading.observation.clone(),
    }
}

/// 一次可写挂载的账（Q2 两种读法、7.3 第一二行的对拍、F6）。
#[allow(
    clippy::too_many_arguments,
    reason = "一次挂载的坐标与上下文原样进结果行，拆成结构体只多一层搬运"
)]
pub(super) fn evaluate_a_random_mount(
    arm: FourthRunArm,
    ledger: &RandomLedger,
    base: &MemoryPool,
    order: usize,
    step_label: &str,
    operation: &'static str,
    phase: &'static str,
    hidden: &[HiddenRange],
    mounted: &Result<Mounted, singlefs_core::mount::MountError>,
    reading: &ObservedCallReading,
) -> RandomMount {
    let summary = summary_of_a_random_mount(mounted, reading);
    let faults: Vec<ThirdRunFault> = hidden
        .iter()
        .map(|range| range.as_third_run_fault())
        .collect();
    let judgement = independent_geometry(base)
        .ok()
        .map(|geometry| device_judgement(base, &geometry, &faults));
    let literal_last = ledger.last_confirmed_number_before(order);
    let rollback_aware_last = ledger.last_confirmed_number_before_rollback_aware(order);
    let effective_content = summary
        .effective
        .and_then(|effective| ledger.content_number_of(effective));
    let device_effective_content = judgement
        .as_ref()
        .and_then(|judged| judged.effective)
        .and_then(|effective| ledger.content_number_of(effective));
    let older_version_returned = effective_content.is_some_and(|number| number < literal_last);
    let older_version_returned_rollback_aware = effective_content.is_some_and(|number| {
        number != rollback_aware_last && ledger.confirmed_numbers.contains(&number)
    });
    let mut stop_clause_four_findings = Vec::new();
    if let Some(judged) = &judgement {
        if arm.is_today() {
            if let Ok(result) = mounted {
                if judged.chosen != Some(root_pair(&result.output.chosen_root))
                    || judged.effective != Some(root_pair(&result.output.effective_root))
                {
                    stop_clause_four_findings.push(format!(
                        "7.3 第一行：装置择根 {} / E {}，crates {} / {}",
                        key_text(judged.chosen),
                        key_text(judged.effective),
                        key_text(Some(root_pair(&result.output.chosen_root))),
                        key_text(Some(root_pair(&result.output.effective_root)))
                    ));
                }
            }
        }
        let refused_by_the_arm = summary.member == arm.refusal_member();
        let device_witness = match arm.witness {
            FourthRunWitness::SystemConfiguration => Some(judged.configuration_newer),
            FourthRunWitness::UnreadableRootRingSlot => Some(judged.unreadable_root_slots > 0),
            FourthRunWitness::NotJudged | FourthRunWitness::SystemConfigurationCarried => None,
        };
        if arm.candidate == FourthRunCandidate::Jia {
            if let Some(witnessed) = device_witness {
                if witnessed != refused_by_the_arm {
                    stop_clause_four_findings.push(format!(
                        "7.3 第二行：装置算的 N={witnessed}，臂拒={refused_by_the_arm}（成员 {}）",
                        summary.member
                    ));
                }
            }
        }
    }
    let failure_clause_six = (operation == "crash_recovery").then(|| {
        judgement
            .as_ref()
            .is_some_and(|judged| judged.configuration_newer && judged.unreadable_root_slots == 0)
    });
    RandomMount {
        order,
        step_label: step_label.to_string(),
        operation,
        phase,
        base_fingerprint: image_fingerprint(base),
        summary,
        reading: reading.clone(),
        judgement,
        hidden_ranges: hidden.len(),
        literal_last,
        rollback_aware_last,
        effective_content,
        device_effective_content,
        older_version_returned,
        older_version_returned_rollback_aware,
        stop_clause_four_findings,
        failure_clause_six,
    }
}

impl RandomExecutor<'_> {
    /// 每一步做完：记各版单元第一次被改的那一步；要留的那一步的镜像留下。
    fn after_a_step(&mut self, order: usize, image: &MemoryPool) {
        let ring_roots = independent_geometry(image)
            .map(|geometry| readable_roots_independent(image, &geometry))
            .unwrap_or_default();
        for confirmation in &self.ledger.confirmations {
            if self
                .first_changes
                .contains_key(&confirmation.content_number)
                || confirmation.order >= order
            {
                continue;
            }
            let changed = confirmation
                .unit_bytes
                .iter()
                .any(|((device, slot, span), bytes)| {
                    let length = usize::try_from(span * SLOT_BYTES_LOCAL).expect("单元长");
                    image
                        .read(*device, slot * SLOT_BYTES_LOCAL, length)
                        .unwrap_or_default()
                        != *bytes
                });
            if changed {
                let later_confirmations = self
                    .ledger
                    .confirmations
                    .iter()
                    .filter(|later| later.content_number > confirmation.content_number)
                    .count();
                self.first_changes.insert(
                    confirmation.content_number,
                    FirstChange {
                        order,
                        later_confirmations,
                        root_in_ring_then: ring_roots.contains(&confirmation.root),
                    },
                );
            }
        }
        if self.capture_after_order == Some(order) {
            self.captured_image = Some(image.clone());
        }
    }

    /// 尾巴（登记 5.3 H-随：关闭 C → 可写挂载（不注入）→ 冷启动读回 → ③c）。交回尾巴那次读回、③c 的分类与尾巴之后的镜像。
    pub(super) fn run_the_tail(&mut self, order_of_the_tail: usize) -> TailReading {
        self.session = None;
        self.stream_length_at_step_start
            .push(self.stream.operation_count());
        let outcome = self.observed_mount(order_of_the_tail, "tail", "tail", &[]);
        self.calls_after_step.push(self.plan.calls());
        let end = self.image();
        self.after_a_step(order_of_the_tail, &end);
        let report = recover(&end, JournalPolicy::Consult);
        let read_back = classify_random_read_back(
            &report,
            &self.ledger,
            self.ledger
                .last_confirmed_number_before(order_of_the_tail + 1),
            self.ledger
                .last_confirmed_number_before_rollback_aware(order_of_the_tail + 1),
        );
        let first_hidden_instance = self
            .crash_recovery_events
            .first()
            .map(|event| event.hidden_root.0);
        let tail_class = third_tail_class(&end, &self.ledger, first_hidden_instance);
        self.steps.push(RandomStepRecord {
            order: order_of_the_tail,
            operation: "tail".to_string(),
            outcome,
            fingerprint_after: image_fingerprint(&end),
            checker: None,
            read_back: Some(read_back.clone()),
        });
        TailReading {
            read_back,
            tail_class,
            end,
        }
    }
}

#[cfg(test)]
mod tests;

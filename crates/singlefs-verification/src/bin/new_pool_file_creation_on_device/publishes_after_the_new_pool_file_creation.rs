//! 新池新建文件之后的那几次发布：覆盖写出第二版、换实例可写挂载后发布第三版、同一次挂载里再覆盖写与发布第四版、把回退下限抬到上限，
//! 每一次连同它的写账与设备一层的窗口比对一起打行。

use super::*;

/// 发布 B：同一个进程里再覆盖写一次，分配器从新池新建文件的分配记录重建（与可写挂载同一条路）、装上这个进程的根环
/// （`allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process`，只在内存里拼、不读盘）；两次快照之差就是这一次发布
/// 交给设备的调用数，挂钟只计发布本身。
///
/// # Errors
/// 发布 B 失败：交回失败账那几行与一句原因（增补 2 收口表第 58 行）。
pub(super) fn publish_the_second_version_and_describe<Inner: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, CountedDevice<Inner>)],
    plan: &SharedFaultPlan,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    first_run: &NewPoolFileCreationRun,
) -> Result<FileOverwriteRun, FailedRun> {
    let first_version = &first_run.output;
    let before = device_call_counts(devices, plan);
    let operations_before = stream.operations().len();
    let mut allocator = allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process(
        parameters, devices, first_run,
    );
    let started = Instant::now();
    let published = publish_the_second_version(parameters, devices, &mut allocator, first_version);
    let nanoseconds = started.elapsed().as_nanos();
    let after = device_call_counts(devices, plan);
    let second = match published {
        Ok(second) => second,
        Err(failed) => {
            let cause = format!("{:?}", failed.cause);
            let (lines, _) = describe_failed_window(
                "file_overwrite",
                &cause,
                &[],
                &failed.writes_of_failed_publishes,
                pool_writes_between(&before, &after),
            );
            return Err(FailedRun {
                lines,
                cause: format!("覆盖写、释放、回退与复用：{cause}"),
            });
        }
    };
    let after_operations = stream.operations();
    let second_segments = split_into_segments(&after_operations[operations_before..], geometry);
    let per_device: Vec<String> = after
        .iter()
        .enumerate()
        .map(|(index, later)| later.since(before[index]).describe(index))
        .collect();
    let mut lines = vec![
        format!(
            "name=file_overwrite root_txg={} transaction={} released={} nanoseconds={nanoseconds} operations={} segments={} closed_form={} {}",
            second.root.checkpoint_txg.0,
            second.record.transaction,
            second.released.len(),
            after_operations.len() - operations_before,
            segment_sizes_text(&second_segments),
            closed_form_state_count(&second_segments),
            per_device.join(" ")
        ),
        describe_publish_writes(
            "file_overwrite",
            second.root.checkpoint_txg.0,
            &second.writes,
        ),
    ];
    let (file_overwrite_line, file_overwrite_matches) = publish_writes_against_device(
        "file_overwrite",
        &[&second.writes],
        pool_writes_between(&before, &after),
    );
    lines.push(file_overwrite_line);
    Ok(FileOverwriteRun {
        lines,
        window_matches_device: file_overwrite_matches,
    })
}

/// 发布 B 之后：丢掉写的那一套句柄，同一对盘冷重开（`reopen` 拿到关掉的那几块盘、交回重新打开的），走可写挂载（恢复、取号、写行、暖机），
/// 再发布 C。重开是新一轮：计数从零重数，所以这一段自己开一个注入计划（不接着上一段那个）。
///
/// # Errors
/// 可写挂载失败、取号那道屏障没数到、挂载之后现行那一版没有文件、发布 C 失败：交回到那一刻为止的结果行与一句原因。
pub(super) fn switch_instance_and_publish_third_version<Inner, Reopen>(
    parameters: &MakeFilesystemParameters,
    devices_after_file_overwrite: Vec<(DeviceIdentity, CountedDevice<Inner>)>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    clock: &mut SegmentClock,
    reopen: Reopen,
) -> Result<SecondInstanceRun<Inner>, FailedRun>
where
    Inner: BlockDevice,
    Reopen: FnOnce(Vec<(DeviceIdentity, Inner)>) -> Vec<(DeviceIdentity, Inner)>,
{
    let mounted = reopen_and_mount_writable(
        parameters,
        devices_after_file_overwrite,
        stream,
        geometry,
        SharedFaultPlan::unarmed(*geometry),
        clock,
        reopen,
    )?;
    publish_the_third_version_and_describe(parameters, mounted, stream, geometry, clock)
}

/// 取号写几轮，每轮每块盘一写（D22（单元原子性怎么合成） 已定项 8「取号写两轮」；`singlefs-core` 的 `transaction.rs` 里
/// `ACQUISITION_WRITE_ROUNDS` 那一份不开放，这里照条款另写一份，两份对不上由挂载窗口那几条用例判红）。
const ACQUISITION_WRITE_ROUNDS: u64 = 2;

/// 新池上取号那一截在录制流里几步：两轮各两写（每轮两块盘各一写）、中间那道屏障两块盘各一步；最后一轮之后那道屏障归暖机那一截的
/// 第一段，与 `new_pool_file_creation_publish.rs` 的 `acquisition_operation_count: mkfs_operation_count + 6` 同一切法。
pub(super) const ACQUISITION_OPERATIONS_RECORDED_AFTER_MKFS: usize = 6;

/// 可写挂载里每块盘最后一轮取号写之后的那一道屏障（这块盘写数到了 [`ACQUISITION_WRITE_ROUNDS`] 之后进来的第一道；取号写之前那一道
/// 前面没有写，不算，代码审阅第 19 条；头一轮之后那一道在最后一轮那一写之前，也不算）那一刻的计数：最后一轮那道屏障完成才把新号交出去
/// （`singlefs-core` 的 `transaction.rs` 取号那一段），挂载窗口（成功与失败两条路）从这里算起。某块盘最后一轮写之后一道屏障都没收到
/// （取号那道屏障没发）：交回停下的那一行与原因。
fn counts_when_the_acquisition_barrier_arrived<Inner: BlockDevice>(
    devices: &[(DeviceIdentity, CountedDevice<Inner>)],
    plan: &SharedFaultPlan,
) -> Result<Vec<DeviceCallCounts>, FailedRun> {
    devices
        .iter()
        .map(|(identity, _)| {
            plan.counts_when_the_first_barrier_after_this_many_writes_arrived_at(
                *identity,
                ACQUISITION_WRITE_ROUNDS,
            )
            .map(DeviceCallCounts::of)
            .ok_or_else(|| {
                format!(
                    "盘 {} 在可写挂载里第二轮取号写之后一道屏障都没收到：取号那道屏障没发",
                    identity.0
                )
            })
        })
        .collect::<Result<_, _>>()
        .map_err(|cause: String| FailedRun {
            lines: vec![describe_run_failure("reopen_and_writable_mount", &cause)],
            cause,
        })
}

/// 冷重开、可写挂载。挂载那一段窗口从每块盘最后一轮取号写之后的那一道屏障算起、到挂载返回为止，按种类的合计是写行与每次暖机之和：
/// 取号两轮的系统配置槽写不是发布、不进按种类的账，与新池新建文件那条路上暖机窗口从取号之后算起同一个口径。
///
/// # Errors
/// 可写挂载失败（取号之后那一串发布里失败的，挂载随错交回写入口的账，照成功路径的窗口口径与设备一层判相等；
/// 别的错只打停下的原因）、取号那道屏障没数到：交回到那一刻为止的结果行与一句原因。
pub(super) fn reopen_and_mount_writable<Inner, Reopen>(
    parameters: &MakeFilesystemParameters,
    devices_after_file_overwrite: Vec<(DeviceIdentity, CountedDevice<Inner>)>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    plan: SharedFaultPlan,
    clock: &mut SegmentClock,
    reopen: Reopen,
) -> Result<WritableMountRun<Inner>, FailedRun>
where
    Inner: BlockDevice,
    Reopen: FnOnce(Vec<(DeviceIdentity, Inner)>) -> Vec<(DeviceIdentity, Inner)>,
{
    let closed_devices: Vec<(DeviceIdentity, Inner)> = devices_after_file_overwrite
        .into_iter()
        .map(|(identity, device)| (identity, device.into_inner_and_operations().0.into_inner()))
        .collect();
    let mut devices: Vec<(DeviceIdentity, CountedDevice<Inner>)> = reopen(closed_devices)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                RecordingBlockDevice::with_shared_stream(
                    identity,
                    FaultInjectingBlockDevice::new(identity, inner, plan.clone()),
                    stream.clone(),
                ),
            )
        })
        .collect();
    let mut lines = Vec::new();

    let operations_before_mount = stream.operations().len();
    let mount_started = Instant::now();
    let mounted = match mount_writable(parameters, &mut devices) {
        Ok(mounted) => mounted,
        Err(failure) => {
            let cause = format!("{failure:?}");
            let failure_lines = match &failure {
                // 取号之后那一串发布里有一次没做成：挂载把写入口的账随错交回（已经落盘的写行与前几次暖机、失败那一次已记的写），
                // 窗口与成功路径的挂载窗口同一个起点——最后一轮取号写之后那道屏障，取号两轮的系统配置槽写不是发布、不在账里。
                MountError::Publish(failed) => {
                    let counts_after_acquisition =
                        counts_when_the_acquisition_barrier_arrived(&devices, &plan)?;
                    describe_failed_window(
                        "reopen_and_writable_mount",
                        &cause,
                        &failed.writes_of_persisted_publishes,
                        &failed.writes_of_failed_publishes,
                        pool_writes_between(
                            &counts_after_acquisition,
                            &device_call_counts(&devices, &plan),
                        ),
                    )
                    .0
                }
                // 没走到取号之后的发布：恢复、准入、取号自己报的错（取号写的回卷在它自己的错里），这一段没有发布的账可比。
                MountError::Recovery(_)
                | MountError::FileVersionWithoutAnyJournalRecord
                | MountError::InstanceTableMalformed
                | MountError::InstanceTableUnreadable
                | MountError::Acquisition(_)
                | MountError::RaiseFloorSequencePublishFailed(_)
                | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
                | MountError::RollbackFloorAboveCeiling { .. }
                | MountError::VersionWithoutFileUnitPointerOfABadImage { .. }
                | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
                | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
                | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
                | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
                | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
                | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
                | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
                | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
                | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
                | MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(_)
                | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
                    ..
                }
                | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
                | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
                    ..
                }
                // 有盘落后于现行那一版、又缺它的单元或它的根（根写在这块盘上而那个根环槽里不是它）：挂着之后的入口在任何写之前拒，
                // 这个二进制不换盘，走不到。
                | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
                // 盘上的实例代号或 txg 到顶：取号之前拒，一个写都没发。
                | MountError::SequenceNumberPastTheTopOfItsRange(_)
                // 写行与暖机之后推的抬 F 报错（实审 A1b Q5，账在错里）：这个二进制每次都在新建的池上挂，取号之前的准入不会不够、走不到推抬 F；
                // 走到了也只报原样的错，窗口的账不比（推的，没在真设备上造过这一格）。
                | MountError::FloorRaiseFailedAfterTheMountsPublishes(_)
                // 读到更新的东西读不出、重读一次仍读不出（C554 乙）：取号之前拒，一个写都没发。
                | MountError::NewerStateStillUnreadableAfterOneReread(_)
                // 读阶段之后逐盘核或取号那一刻系统配置又读不出（C664）：第一个取号写之前拒，一个写都没发。
                | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
                // 重建分配器读根环重读那一遍仍一条有效根都没有（Q3）：取号之前拒，一个写都没发。
                | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {
                    vec![describe_run_failure("reopen_and_writable_mount", &cause)]
                }
            };
            return Err(FailedRun {
                lines: failure_lines,
                cause: format!("可写挂载：{cause}"),
            });
        }
    };
    let mount_nanoseconds = mount_started.elapsed().as_nanos();
    let counts_after_mount = device_call_counts(&devices, &plan);
    let counts_after_acquisition = counts_when_the_acquisition_barrier_arrived(&devices, &plan)?;
    let mount_operations = stream.operations();
    let mount_segments =
        split_into_segments(&mount_operations[operations_before_mount..], geometry);
    let output = &mounted.output;
    let warm_up_txgs: Vec<String> = output
        .warm_up_publishes
        .iter()
        .map(|publish| publish.root().checkpoint_txg.0.to_string())
        .collect();
    let per_device_mount: Vec<String> = counts_after_mount
        .iter()
        .enumerate()
        .map(|(index, counts)| counts.describe(index))
        .collect();
    lines.push(format!(
        "name=writable_mount instance={} chosen_root={}:{} rows_written={} row_publish_root={} warm_up_txgs={} nanoseconds={mount_nanoseconds} operations={} segments={} closed_form={} {}",
        output.instance.0,
        output.chosen_root.instance.0,
        output.chosen_root.checkpoint_txg.0,
        output.rows_written.len(),
        root_text(&output.row_publish),
        warm_up_txgs.join(","),
        mount_operations.len() - operations_before_mount,
        segment_sizes_text(&mount_segments),
        closed_form_state_count(&mount_segments),
        per_device_mount.join(" ")
    ));
    lines.push(describe_publish_writes(
        "instance_row",
        output.row_publish.root().checkpoint_txg.0,
        writes_of(&output.row_publish),
    ));
    let mut mount_publishes: Vec<&WritesByStructureKind> = vec![writes_of(&output.row_publish)];
    for warm_up_publish in &output.warm_up_publishes {
        lines.push(describe_publish_writes(
            "warm_up",
            warm_up_publish.root().checkpoint_txg.0,
            writes_of(warm_up_publish),
        ));
        mount_publishes.push(writes_of(warm_up_publish));
    }
    let (mount_window_line, mount_window_matches) = publish_writes_against_device(
        "writable_mount",
        &mount_publishes,
        pool_writes_between(&counts_after_acquisition, &counts_after_mount),
    );
    lines.push(mount_window_line);
    // 这一段从上一个标记算起，含冷重开那几块盘：名字照实写，不叫「可写挂载」。
    lines.push(clock.mark("reopen_and_writable_mount"));
    Ok(WritableMountRun {
        devices,
        mounted,
        plan,
        counts_after_mount,
        lines,
        mount_window_matches_device: mount_window_matches,
    })
}

/// 可写挂载之后发布 C，接在挂载交回的现行那一版上、用挂载取到的实例。
///
/// # Errors
/// 挂载之后现行那一版没有文件（发布 B 之后重开，上一版带文件）、发布 C 失败：交回到那一刻为止的结果行
/// （发布 C 失败时连同失败账那几行，增补 2 收口表第 58 行）与一句原因。
pub(super) fn publish_the_third_version_and_describe<Inner: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    mounted_run: WritableMountRun<Inner>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    clock: &mut SegmentClock,
) -> Result<SecondInstanceRun<Inner>, FailedRun> {
    let WritableMountRun {
        mut devices,
        mut mounted,
        plan,
        counts_after_mount,
        mut lines,
        mount_window_matches_device,
    } = mounted_run;
    let Some(current) = mounted.current.file_version().cloned() else {
        let cause = "可写挂载之后现行那一版没有文件：发布 B 之后重开，上一版带文件".to_string();
        lines.push(describe_run_failure("third_transaction", &cause));
        return Err(FailedRun { lines, cause });
    };
    let instance = mounted.output.instance;
    let third = match publish_an_overwrite_in_the_mounted_instance_and_describe(
        "third_transaction",
        &mut devices,
        &plan,
        &counts_after_mount,
        stream,
        geometry,
        |devices| {
            publish_the_third_version(
                parameters,
                devices,
                &mut mounted.allocator,
                &current,
                instance,
            )
        },
    ) {
        Ok(third) => third,
        Err(failed) => {
            lines.extend(failed.lines);
            return Err(FailedRun {
                lines,
                cause: format!("发布 C：{}", failed.cause),
            });
        }
    };
    lines.extend(third.lines);
    lines.push(clock.mark("third_transaction"));

    Ok(SecondInstanceRun {
        devices,
        allocator: mounted.allocator,
        current_version: third.version,
        plan,
        lines,
        every_window_matches_device: mount_window_matches_device && third.window_matches_device,
    })
}

/// 同一次挂载里的一次覆盖写（发布 C；`raise-rollback-floor` 还有发布 D）：`publish` 发这一次，挂钟只计它；
/// 窗口从 `counts_before` 那一刻算起、到它返回为止，按种类的合计与设备一层逐项比。结果行以 `window` 为名（`name=<window>` 一行
/// 与 `publish=<window>`、`window=<window>` 各一行）。
///
/// # Errors
/// 发布失败：交回失败账那几行（增补 2 收口表第 58 行）与失败原因的 Debug 文本，调用方在前面接上它前面几段的结果行。
fn publish_an_overwrite_in_the_mounted_instance_and_describe<Inner, Publish>(
    window: &str,
    devices: &mut [(DeviceIdentity, CountedDevice<Inner>)],
    plan: &SharedFaultPlan,
    counts_before: &[DeviceCallCounts],
    stream: &SharedStream,
    geometry: &FixedGeometry,
    publish: Publish,
) -> Result<OverwriteInTheMountedInstance, FailedRun>
where
    Inner: BlockDevice,
    Publish: FnOnce(
        &mut [(DeviceIdentity, CountedDevice<Inner>)],
    ) -> Result<TransactionOutput, FailedPublish>,
{
    let operations_before = stream.operations().len();
    let started = Instant::now();
    let published = publish(devices);
    let nanoseconds = started.elapsed().as_nanos();
    let counts_after = device_call_counts(devices, plan);
    let version = match published {
        Ok(version) => version,
        Err(failed) => {
            let cause = format!("{:?}", failed.cause);
            let (failure_lines, _) = describe_failed_window(
                window,
                &cause,
                &[],
                &failed.writes_of_failed_publishes,
                pool_writes_between(counts_before, &counts_after),
            );
            return Err(FailedRun {
                lines: failure_lines,
                cause,
            });
        }
    };
    let operations = stream.operations();
    let segments = split_into_segments(&operations[operations_before..], geometry);
    let per_device: Vec<String> = counts_after
        .iter()
        .enumerate()
        .map(|(index, later)| later.since(counts_before[index]).describe(index))
        .collect();
    let mut lines = vec![
        format!(
            "name={window} root_txg={} transaction={} released={} nanoseconds={nanoseconds} operations={} segments={} closed_form={} {}",
            version.root.checkpoint_txg.0,
            version.record.transaction,
            version.released.len(),
            operations.len() - operations_before,
            segment_sizes_text(&segments),
            closed_form_state_count(&segments),
            per_device.join(" ")
        ),
        describe_publish_writes(window, version.root.checkpoint_txg.0, &version.writes),
    ];
    let (window_line, window_matches_device) = publish_writes_against_device(
        window,
        &[&version.writes],
        pool_writes_between(counts_before, &counts_after),
    );
    lines.push(window_line);
    Ok(OverwriteInTheMountedInstance {
        version,
        lines,
        window_matches_device,
    })
}

/// `raise-rollback-floor` 的发布 D：发布 C 之后、同一次挂载里接在发布 C 那一版上再覆盖写一次（`publish_the_fourth_version`，
/// 宿主检查重跑同一份），这一段窗口从发布 C 返回那一刻算起。结果行与发布 C 同形，名字是 `fourth_transaction`。
///
/// # Errors
/// 发布 D 失败：接在前面几段的结果行后面交回失败账那几行（增补 2 收口表第 58 行）与一句原因。
pub(super) fn publish_the_fourth_version_and_describe<Inner: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    second_instance_run: SecondInstanceRun<Inner>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    clock: &mut SegmentClock,
) -> Result<SecondInstanceRun<Inner>, FailedRun> {
    let SecondInstanceRun {
        mut devices,
        mut allocator,
        current_version: third_version,
        plan,
        mut lines,
        every_window_matches_device,
    } = second_instance_run;
    let counts_after_third = device_call_counts(&devices, &plan);
    let fourth = match publish_an_overwrite_in_the_mounted_instance_and_describe(
        "fourth_transaction",
        &mut devices,
        &plan,
        &counts_after_third,
        stream,
        geometry,
        |devices| publish_the_fourth_version(parameters, devices, &mut allocator, &third_version),
    ) {
        Ok(fourth) => fourth,
        Err(failed) => {
            lines.extend(failed.lines);
            return Err(FailedRun {
                lines,
                cause: format!("发布 D：{}", failed.cause),
            });
        }
    };
    lines.extend(fourth.lines);
    lines.push(clock.mark("fourth_transaction"));

    Ok(SecondInstanceRun {
        devices,
        allocator,
        current_version: fourth.version,
        plan,
        lines,
        every_window_matches_device: every_window_matches_device && fourth.window_matches_device,
    })
}

/// `raise-rollback-floor` 在发布 C 之后的两段：同一次挂载里发布 D（第 4 新的非空有效根落到 A 上），再把 F 抬到上限。`main` 与用例走这同一个函数。
///
/// # Errors
/// 发布 D 或抬 F 失败：交回到那一刻为止的结果行（失败账那几行在内）与一句原因。
pub(super) fn publish_the_fourth_version_and_raise_the_rollback_floor<Inner: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    third_version_run: SecondInstanceRun<Inner>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    clock: &mut SegmentClock,
) -> Result<SecondInstanceRun<Inner>, FailedRun> {
    let fourth_version_run = publish_the_fourth_version_and_describe(
        parameters,
        third_version_run,
        stream,
        geometry,
        clock,
    )?;
    raise_the_rollback_floor_and_describe(parameters, fourth_version_run, stream, geometry, clock)
}

/// `raise-rollback-floor` 模式冷重开之后打的那一行：所选根从盘上读回的回退下界 F（Z10，
/// `research/prompts/m2-final-code-r2-main-verification.md` 第四节第 5 条）。所选根照恢复择根的同一套规则现读
/// （`choose_system_configuration` + `choose_root`：按 (txg, 实例) 择新），F 取那条根记录自己的字段——
/// 不取 `name=raise_rollback_floor` 那一行里内存那一版的 `requested_floor`。择不出系统配置或根时两段都写 `none`。
pub(super) fn rollback_floor_read_back_from_the_chosen_root_line(
    reader: &dyn PoolReader,
) -> String {
    let chosen_root = choose_system_configuration(reader)
        .ok()
        .and_then(|system_configuration| choose_root(reader, &system_configuration));
    match chosen_root {
        Some(root) => format!(
            "name=recover_cold_rollback_floor chosen_root={}:{} rollback_floor_on_disk={}",
            root.instance.0, root.checkpoint_txg.0, root.rollback_floor.0
        ),
        None => "name=recover_cold_rollback_floor chosen_root=none rollback_floor_on_disk=none"
            .to_string(),
    }
}

/// `raise-rollback-floor`：发布 D 之后、同一次挂载里把 F 抬到上限（`raise_the_rollback_floor_to_its_ceiling`，宿主检查重跑同一份）。
/// 这一段窗口从抬 F 之前那一刻算起、到它返回为止，按种类的合计是那一串空发布之和，与设备一层逐项比（增补 2 收口表第 58 行
/// 「二进制那一侧判相等」）。抬 F 只有测试入口（`raise_rollback_floor` 的文档注释），这一档就是那个入口在真设备上的一次。
/// 结果行里 `requested_floor` 是抬完之后最后一次空发布的根带的 F，`ceiling` 是 core 抬 F 时算的上限。
///
/// # Errors
/// 抬 F 失败：那一串空发布里有一次发不出去时，失败账那几行照可写挂载失败的判法打（已落盘的那几次各一行、失败那一次一行、
/// 两样相加与设备一层比一行）；别的错在任何写之前，只打停下的原因。都接在前面几段的结果行后面交回。
pub(super) fn raise_the_rollback_floor_and_describe<Inner: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    second_instance_run: SecondInstanceRun<Inner>,
    stream: &SharedStream,
    geometry: &FixedGeometry,
    clock: &mut SegmentClock,
) -> Result<SecondInstanceRun<Inner>, FailedRun> {
    let SecondInstanceRun {
        mut devices,
        mut allocator,
        mut current_version,
        plan,
        mut lines,
        every_window_matches_device,
    } = second_instance_run;
    let floor_before_raise = current_version.root.rollback_floor;
    let counts_before_raise = device_call_counts(&devices, &plan);
    let operations_before_raise = stream.operations().len();
    let raise_started = Instant::now();
    let raised = raise_the_rollback_floor_to_its_ceiling(
        parameters,
        &mut devices,
        &mut allocator,
        &mut current_version,
    );
    let raise_nanoseconds = raise_started.elapsed().as_nanos();
    let counts_after_raise = device_call_counts(&devices, &plan);
    let raised = match raised {
        Ok(raised) => raised,
        Err(failure) => {
            let cause = format!("{failure:?}");
            let failure_lines = match &failure {
                // 抬 F 的写入口随错丢掉，这一串的账随错交回（先写系统配置那一步、已经落盘的那几次空发布、失败那一次已记的写）。
                MountError::RaiseFloorSequencePublishFailed(failed) => {
                    let PublishSequenceFailed {
                        cause: _,
                        writes_before_the_first_publish,
                        writes_of_persisted_publishes,
                        writes_of_failed_publishes,
                    } = &**failed;
                    describe_failed_window_after_writes_outside_any_publish(
                        "raise_rollback_floor",
                        &cause,
                        Some(writes_before_the_first_publish),
                        writes_of_persisted_publishes,
                        writes_of_failed_publishes,
                        pool_writes_between(&counts_before_raise, &counts_after_raise),
                    )
                    .0
                }
                // 先写系统配置那一步报错：一次发布都没开始，那一步已记的写随错交回。
                MountError::RaiseFloorSystemConfigurationWriteFailedBeforeAnyRoot(failed) => {
                    describe_failed_window_after_writes_outside_any_publish(
                        "raise_rollback_floor",
                        &cause,
                        Some(&failed.writes),
                        &[],
                        &[],
                        pool_writes_between(&counts_before_raise, &counts_after_raise),
                    )
                    .0
                }
                // 在第一次空发布之前就停下的：选系统配置、读实例表、算上限、超上限、整串预演里有一次报错，这一段没有发布的账可比。
                // 可写挂载那几条（取号、写行、暖机）抬 F 走不到，照样列全，不写通配臂。
                MountError::Recovery(_)
                | MountError::FileVersionWithoutAnyJournalRecord
                | MountError::InstanceTableMalformed
                | MountError::InstanceTableUnreadable
                | MountError::Acquisition(_)
                | MountError::Publish(_)
                | MountError::RaiseFloorSequenceRefusedByTheRehearsalBeforeAnyWrite { .. }
                | MountError::RollbackFloorAboveCeiling { .. }
                | MountError::VersionWithoutFileUnitPointerOfABadImage { .. }
                | MountError::RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { .. }
                | MountError::RollbackFloorCeilingRootRingSlotStillBadAfterOneReread { .. }
                | MountError::InstanceGenerationChangedBeforeAcquisition { .. }
                | MountError::SpaceAdmissionRefusedBeforeAcquisition { .. }
                | MountError::RowPublishAdmissionRefusedBeforeAcquisition { .. }
                | MountError::WarmUpAdmissionRefusedBeforeAcquisition { .. }
                | MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided { .. }
                | MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion { .. }
                | MountError::WritableDeviceCountBelowTheStripeWidthLowerBound { .. }
                | MountError::RequestedFloorBelowTheEffectiveFloorWhoseRaiseIsUndecided {
                    ..
                }
                | MountError::DeviceIdentitiesHandedInMoreThanOnce { .. }
                | MountError::CallerParametersDisagreeWithTheSelectedSystemConfiguration {
                    ..
                }
                // 有盘落后于现行那一版、又缺它的单元或它的根（根写在这块盘上而那个根环槽里不是它）：挂着之后的入口在任何写之前拒，
                // 这个二进制不换盘，走不到。
                | MountError::DevicesBehindTheCurrentVersionAndMissingItsUnitsOrItsRoot { .. }
                // 现行那一版 txg 到顶：算这一串的 txg 时拒，一个写都没发；可写挂载那一处的抬 F 报错成员抬 F 自己报不出来。
                | MountError::SequenceNumberPastTheTopOfItsRange(_)
                | MountError::FloorRaiseFailedAfterTheMountsPublishes(_)
                // 重读一次仍读不出、读阶段之后系统配置又读不出、重建分配器读根环重读仍一条有效根都没有只由可写挂载交回，抬 F 报不出来。
                | MountError::NewerStateStillUnreadableAfterOneReread(_)
                | MountError::SystemConfigurationUnreadableAfterTheReadStage(_)
                | MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread => {
                    vec![describe_run_failure("raise_rollback_floor", &cause)]
                }
            };
            lines.extend(failure_lines);
            return Err(FailedRun {
                lines,
                cause: format!("抬 F：{cause}"),
            });
        }
    };
    let raise_operations = stream.operations();
    let raise_segments =
        split_into_segments(&raise_operations[operations_before_raise..], geometry);
    let per_device_raise: Vec<String> = counts_after_raise
        .iter()
        .enumerate()
        .map(|(index, later)| later.since(counts_before_raise[index]).describe(index))
        .collect();
    let raise_txgs: Vec<String> = raised
        .publishes
        .iter()
        .map(|publish| publish.root.checkpoint_txg.0.to_string())
        .collect();
    lines.push(format!(
        "name=raise_rollback_floor floor_before={} requested_floor={} ceiling={} publishes={} root_txgs={} reclaimed={} abandoned_roots_unreadable={} nanoseconds={raise_nanoseconds} operations={} segments={} closed_form={} {}",
        floor_before_raise.0,
        current_version.root.rollback_floor.0,
        raised.ceiling.0,
        raised.publishes.len(),
        raise_txgs.join(","),
        raised.reclaimed.len(),
        raised.abandoned_roots_unreadable,
        raise_operations.len() - operations_before_raise,
        segment_sizes_text(&raise_segments),
        closed_form_state_count(&raise_segments),
        per_device_raise.join(" ")
    ));
    let mut raise_publishes: Vec<&WritesByStructureKind> = Vec::new();
    for raise_publish in &raised.publishes {
        lines.push(describe_publish_writes(
            "raise_rollback_floor",
            raise_publish.root.checkpoint_txg.0,
            &raise_publish.writes,
        ));
        raise_publishes.push(&raise_publish.writes);
    }
    // 第一次发布之前先把新 F 写进每块盘系统配置那一步不属于任何一次发布（D16（发布语义） 已定项 1「抬 F 那一串」）：
    // 它单打一行，再并进窗口的合计与设备一层比。
    lines.push(format!(
        "name=writes_outside_any_publish window=raise_rollback_floor {}",
        describe_writes_by_kind(&raised.system_configuration_writes_before_the_first_publish)
    ));
    let (raise_window_line, raise_window_matches) =
        publish_writes_and_writes_outside_any_publish_against_device(
            "raise_rollback_floor",
            &raise_publishes,
            Some(&raised.system_configuration_writes_before_the_first_publish),
            pool_writes_between(&counts_before_raise, &counts_after_raise),
        );
    lines.push(raise_window_line);
    lines.push(clock.mark("raise_rollback_floor"));

    Ok(SecondInstanceRun {
        devices,
        allocator,
        current_version,
        plan,
        lines,
        every_window_matches_device: every_window_matches_device && raise_window_matches,
    })
}

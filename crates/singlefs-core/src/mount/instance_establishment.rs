//! 可写挂载的后半段（[`establish_instance`]）：取号之前的空间准入与逐盘核、预演，取号、写行、暖机，
//! 写行之后为空间推的抬 F。

use super::*;

/// 取号 → 写行发布 → 暖机到本实例的根覆盖每块盘 → 取号之前准入不够的，推抬 F 的空发布再判：可写挂载的后半段。
pub(super) fn establish_instance<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices_handed_in: &mut [(DeviceIdentity, Device)],
    mut allocator: PoolAllocator,
    start: InstanceStart,
) -> Result<Mounted, MountError> {
    // 建上一版读到只坏一份的那几组位置条目先记上（不加次数，次数另有一个数）：这一串的发布换下它们时照释放之前读盘核隔离
    // （`transaction::copies_of_the_exempt_units_read_with_one_copy_failing`），挂载交回之前按挂载之后那一版认。
    allocator.keep_the_locations_read_with_one_copy_failing_uncounted(
        start
            .previous
            .units_read_with_one_copy_failing_while_building_on_it(),
    );
    // 这次挂载往下的读，取号之前的几趟预演与取号之后的真做共用一份（岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一
    // 「读一次两边共用」那一行，用户定）：每一块第一次读时才穿到底下那块盘、读到的字节记下，之后谁再读都取记下的那一份；
    // 这次挂载写成的块读写下的。读阶段、恢复与重建分配器（这之前）读的不经它：它们自己的重读要打到盘上。
    // 逐盘核、真做取号写之前那几次读与实例表清扫三段照旧读盘（`ReadRouteOfThisMount::StraightToTheDisk`）：条款要它们在那一刻再读一次盘；
    // 取号写、轮换写与抬 F 先写系统配置那一步算世代号、覆写哪一槽那一读同样读盘（`BlockDevice::read_at_straight_from_the_disk`）。
    // 读坏之后当场重读的那一读（抬 F 算上限与重算影子账读根环、算 F 生效值重读根环与实例表、释放之前读盘核、被抛弃根的账）也穿到盘上，
    // 读到自证过的换掉记下的那一份（`recovery::PoolReaderOfOneReadPass`，`ReadRouteOfThisMount::RereadStraightToTheDiskKeepingWhatSelfVerifies`）。
    // 读成而读的一方核出自证不过的那几块从记下的那一份里丢掉、记进读坏账（岔路单 `research/prompts/m3-b44-code-r3-forks.md`
    // 岔路一「两样都做」那一行）；读错（盘报了错）的那几块本来不记、记进读错账。
    let reads_failed_on_the_disk: RefCell<Vec<ReadFailedOnTheDisk>> = RefCell::new(Vec::new());
    let reads_failing_self_verification: RefCell<Vec<ReadFailingSelfVerification>> =
        RefCell::new(Vec::new());
    let read_route = Cell::new(ReadRouteOfThisMount::SharedBetweenTheRehearsalAndTheRealRun);
    let mut devices_read_once_in_this_mount: Vec<(
        DeviceIdentity,
        DeviceReadOnceInThisMount<'_, '_, Device>,
    )> = devices_handed_in
        .iter_mut()
        .map(|(identity, device)| {
            (
                *identity,
                DeviceReadOnceInThisMount::over_the_disk(
                    *identity,
                    device,
                    &reads_failed_on_the_disk,
                    &reads_failing_self_verification,
                    &read_route,
                ),
            )
        })
        .collect();
    let devices = &mut devices_read_once_in_this_mount;
    let isolated_slots_per_device: Vec<(DeviceIdentity, u64)> = allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect();
    // 所选那一版的单元清单在开写入口之前取好（树表 0 条那一版要按位置条目读盘）：取号之前逐盘核拿它比。
    // 这一读是这次挂载共用的那一份第一次读这几个单元，之后写系统配置槽之前读实例表取记下的那一份，读不到盘；
    // 读到只坏一份进这次挂载别的读那一份账（D4（校验和位置） 已定项 4「可写挂载期间别的读」），建上一版已读到的那几组上面记过，不再加次数。
    let units_of_the_selected_version = allocator
        .reading_and_counting_the_other_unit_reads_with_one_copy_failing(&*devices, |reader| {
            start.previous.units(
                reader,
                unit_area_start_of_the_chosen_system_configuration(&start.system_configuration),
            )
        })?;
    let pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let all_devices: Vec<DeviceIdentity> =
        pool.devices.iter().map(|(identity, _)| *identity).collect();
    // 暖机推哪几次，取号之前先算出来：准入按这一份算，取号之后的循环也按这一份推。
    let warm_up_publishes_planned =
        warm_up_publish_txgs(parameters, &all_devices, start.first_txg)?;
    // 要取的号装不下（盘上最大的实例代号已是 u32::MAX，代码审阅第 36 条）就在取号之前拒，一个写都没发。
    let instance_to_acquire = next_instance_generation_to_acquire(&pool).map_err(
        |InstanceGenerationPastTheTopOfItsRange { highest_on_disk }| {
            MountError::SequenceNumberPastTheTopOfItsRange(
                SequenceNumberAtTheTopOfItsRange::InstanceGeneration(highest_on_disk),
            )
        },
    )?;
    // 要写的行按要取的号先列出来：实例表那一项准入按它算，取号之后写行也用这一份（号在写之前重算、不等就不写）。
    let rows_written = instance_rows_to_write(&start.previous_row, instance_to_acquire);
    let instance_table_rewrite = instance_table_rewrite_of_the_row_publish(
        start.previous.instance_table_chain(),
        &rows_written,
    );
    // 空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决第四节第 2 条接进可写挂载）：可写挂载的准入要
    // 「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13），预留是式子里的挂载期承诺量——每块盘上连它与别的几项都扣完，
    // 可用(d) ≥ 0（这一刻不另要需求：写行与暖机那一串的空间就在切换预留与 checkpoint 保留池里）。rows0 = 挂载时读到的行数加写行那次
    // 要写的行数（D28（挂载期承诺量） 已定项 3），挂载期间常量，记在分配器上给这次挂载之后的每次发布用（发布路径按它加一）。
    // 不够时（D16（发布语义） 已定项 1「准入」那一行，用户 2026-09-26 定）：写行那次发布照走切换预留，写完行（与暖机）之后推抬 F 的空发布、
    // 再判（`push_floor_raises_after_the_row_publish`）。上一版树表 0 条的不推：抬 F 要带文件的现行版本，这一版上没有可退的带文件版本、
    // 推不出空间（卸载在这一版上同样一个字节都不写），照旧在取号之前拒，盘上逐字节不变。
    // 只供测试的开关（`SpaceAdmission::SkippedByTheTestOnlySwitch`）关掉准入时不判，这次挂载之后的发布也不判（开关装在分配器上）。
    allocator.record_instance_rows_after_this_mounts_row_publish(
        u64::try_from(instance_table_rewrite.rows.len()).expect("实例表的行数装得进 u64"),
    );
    // 写行那次发布之后最新那条根带的就是这张表（池级 checker 按它判有效）：进程内重读挂载时读不出的槽（E2）从写行那次记根起按它判。
    allocator.note_the_instance_table_judging_the_rereads_of_root_ring_slots(
        InstanceTableRecords {
            rows: instance_table_rewrite.rows.clone(),
        },
    );
    allocator.set_space_admission(start.space_admission);
    let admission_before_acquisition = admission_of_the_writable_mount_before_acquisition(
        parameters,
        &pool,
        &allocator,
        &start,
        instance_to_acquire,
    )?;
    let PublishesRehearsedBeforeAcquisition {
        placements_taken_by_every_publish,
        version_after_the_last_publish,
        allocator_after_the_last_publish,
    } = publishes_after_acquisition_rehearsed(
        &pool,
        &allocator,
        &start,
        &instance_table_rewrite,
        rows_written.len(),
        &warm_up_publishes_planned,
        instance_to_acquire,
    )?;
    let placements_planned = Some(placements_taken_by_every_publish);
    // 丙'：过了补写闸的两槽全废的盘交给取号，取号写两轮把它两槽补写成新号（补写失败照甲' 每块盘回卷一次）。
    // 逐盘核排在续做预演之前（代码轮 `research/prompts/m3-close-b19-code-r1-main-verification.md` 第二节 T4 改法一）：预演里只进内存那一趟取号
    // 照真做那一份补写盘表认这几块盘；按空表走的话，那一趟取号读到两槽全废的盘一份自证过的都没有、停在取号，重走到上限在取号之前拒成预演读坏，
    // 补写闸与它的拒因载荷都走不到（攻方 `research/prompts/m3-close-b19-code-r1-opus-output.md` T4 一节）。逐盘核只读盘、不写，排在前面不改盘上的结局。
    // 逐盘核另交回每块盘每个系统配置槽读的那一刻的结论，续做预演判两本账时剔掉落在那一刻已读坏、已报读错的槽上的那几条（改法丙）。
    let DevicesPassingTheCheckBeforeTheAcquisition {
        devices_rewritten_by_the_acquisition,
        system_configuration_slots_as_checked,
    } = devices_passing_the_check_before_the_acquisition(
        &pool,
        &allocator,
        &start,
        &units_of_the_selected_version,
        &read_route,
    )?;
    // 取号之后那一串（取号、写行、暖机、写行之后推的抬 F）要的、取号之前定下来的那几样：真发与取号之前预演续做读同一份。
    let publishes_planned = PublishesPlannedAfterAcquisition {
        instance_to_acquire,
        instance_table_rewrite,
        warm_up_publishes_planned,
        placements_planned,
        admission_before_acquisition,
        devices_rewritten_by_the_acquisition: devices_rewritten_by_the_acquisition.clone(),
    };
    // 挂载续做接着写行与暖机预演到最后的那一版整个预演一遍（续做要推抬 F 时连同取号之后那一串在只进内存的盘上走）：
    // 有一步要开的容器号到顶（C841）就在取号之前拒，盘上逐字节不变；不预演的话这一错在写行那次发布落盘之后才在续做里交回。
    // 预演没走到续做的结局而读盘时有过读错（盘报了错，那一块没记进共用的那一份）或读坏（读成而自证不过，那几块已从共用的那一份里丢掉）：
    // 真做再读要穿到盘上、可能走出别的结局，从头重走，重走到上限仍是这样同样在取号之前拒。
    refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range(
        &pool,
        &version_after_the_last_publish,
        allocator_after_the_last_publish,
        &allocator,
        &start,
        &rows_written,
        &publishes_planned,
        &reads_failed_on_the_disk,
        &reads_failing_self_verification,
        &system_configuration_slots_as_checked,
    )?;
    let PublishedBetweenAcquisitionAndTheContinuation {
        instance,
        row_publish,
        warm_up_publishes,
        space_admission,
        mut current,
        mut allocator,
    } = acquire_then_publish_until_the_continuation(
        parameters,
        devices,
        allocator,
        &start,
        &rows_written,
        publishes_planned,
        ReadsOfTheAcquisition::StraightToTheDiskOfTheRealRun(&read_route),
    )?;

    let writes_of_the_publishes_before_the_continuation =
        writes_of_the_publishes_before_the_continuation(
            &row_publish,
            &warm_up_publishes,
            &space_admission,
        );
    let DeletionQueueContinuation {
        steps_published: deletion_queue_continuation,
        floor_raises: floor_raises_pushed_by_the_continuation,
        refusals_that_pushed_the_floor_raises: refusals_that_pushed_the_continuation_floor_raises,
        outcome: deletion_queue_continuation_outcome,
    } = continue_the_deletion_queue_until_it_is_empty(
        parameters,
        devices,
        &mut allocator,
        &mut current,
        instance,
        start.shadow_ledger,
        writes_of_the_publishes_before_the_continuation,
        space_admission.floor_raises(),
    )?;
    let mut background_compute_of_this_mount =
        BackgroundComputeOfThisMount::of_a_mount(start.background_compute);
    let mut compaction_continuation = compaction_intent_continued_at_the_mount(
        parameters,
        devices.as_mut_slice(),
        &mut allocator,
        &mut current,
        instance,
        &deletion_queue_continuation_outcome,
        &mut background_compute_of_this_mount,
    );
    // 建上一版读到只坏一份的次数记进这次挂载的状态（D4（校验和位置） 已定项 4「读到只坏一份时计数」），会话从它接着用。
    // 读到只坏一份的那几个单元要搬（用户 2026-11-05 定「可写挂载时搬，冷恢复只计数」）：挂载这一串里不另发——另发一批就是挂载的第七段写，
    // 挂载分段、二次崩溃的摆法、取号之前的续做预演都要跟着改（`crates/singlefs-verification/src/writable_mount_phases.rs` 那六段）；
    // 这里按挂载之后那一版把它们认成映射 key 记进挂载期状态，会话下一次用户改动发成之后照快照下另发那批同一条路径搬
    // （`mounted_session::MountedSession::publish_user_change`）。认没认出 key 的几组都留在账上（不加别的读那个数，等到那一批再去掉）：之后再读、那一批释放之前读盘核，
    // 同一份坏副本不再计进别的读，数据单元与映射树节点同一个口径。豁免映射的三类里挂载这一串没换下的（映射树根之下的节点）
    // 那一批换落点（`transaction::repair_of_the_exempt_units_at_these_locations`），挂载这一串已经重写掉的（树表、实例表片这类）到那一批认不出，照丢。
    compaction_continuation
        .state_of_this_mount
        .count_unit_reads_with_one_copy_failing_while_mounting(
            start.unit_reads_with_one_copy_failing_while_building_on_the_version,
        );
    let locations_read_with_one_copy_failing =
        allocator.locations_read_with_one_copy_failing_waiting_for_the_next_user_change();
    if let Some(version_after_the_mount) = current.version_of_trees() {
        let units_read_with_one_copy_failing = units_at_these_locations(
            version_after_the_mount,
            &locations_read_with_one_copy_failing,
        );
        compaction_continuation
            .state_of_this_mount
            .keep_the_mapping_keys_of_the_units_read_with_one_copy_failing_while_mounting(
                units_read_with_one_copy_failing.mapping_keys,
            );
    }
    // 实例表清扫（D18（块里携带什么信息） 已定项 11「回收」一条「谁跑、什么时候跑」，用户 2026-10-03 定）：写行、暖机、抬 F、删除续做之后，
    // 交回之前判一次。清扫自己不让挂载失败（落点被拒这一轮不删、挂载照样交回可写，Q4），走到哪儿记进交回值。
    let instance_table_sweep = with_reads_straight_to_the_disk(&read_route, || {
        sweep_the_instance_table_at_the_end_of_the_mount(
            parameters,
            devices,
            &mut allocator,
            &mut current,
            start.instance_table_sweep,
        )
    });

    Ok(Mounted {
        output: MountOutput {
            instance,
            chosen_root: start.chosen_root,
            effective_root: start.effective_root,
            journal: start.journal,
            rows_written,
            row_publish,
            warm_up_publishes,
            shadow_ledger: start.shadow_ledger,
            shadow_ledger_branch: start.shadow_ledger.branch_name(),
            isolated_slots_per_device,
            abandoned_roots_unreadable: start.abandoned_roots_unreadable,
            space_admission,
            // `parameters` 是可写挂载按盘上择到的系统配置建的那一份（`mount_writable_with_test_only_switches` 核过调用方的参数与盘表），
            // 盘表就是取号写过的这几块盘。
            parameters_and_device_table: ParametersAndDeviceTableOfTheMount {
                parameters_on_disk: parameters.clone(),
                device_identities_in_table_order: all_devices,
                background_compute_of_this_mount,
            },
            rereads: start.rereads,
            deletion_queue_continuation,
            floor_raises_pushed_by_the_continuation,
            refusals_that_pushed_the_continuation_floor_raises,
            deletion_queue_continuation_outcome,
            instance_table_sweep,
            compaction_continuation,
            devices_rewritten_by_the_acquisition,
        },
        allocator,
        current,
    })
}

/// 取号之前的空间准入（D28（挂载期承诺量） 已定项 1 的式子逐设备合取；C363 (b) 判决第四节第 2 条接进可写挂载）：可写挂载的准入要
/// 「实例切换的预留拿得到」（D2（RAID 条带策略） 已定项 13），预留是式子里的挂载期承诺量——每块盘上连它与别的几项都扣完，
/// 可用(d) ≥ 0（这一刻不另要需求：写行与暖机那一串的空间就在切换预留与 checkpoint 保留池里）。读数按分配器上记好的 rows0 算
/// （调用方先记，D28（挂载期承诺量） 已定项 3）。不够时（D16（发布语义） 已定项 1「准入」那一行，用户 2026-09-26 定）：写行那次发布照走切换预留，
/// 写完行（与暖机）之后推抬 F 的空发布、再判（`push_floor_raises_after_the_row_publish`）。上一版树表 0 条的不推：抬 F 要带文件的现行版本，
/// 这一版上没有可退的带文件版本、推不出空间（卸载在这一版上同样一个字节都不写），照旧在取号之前拒，盘上逐字节不变。
/// 只供测试的开关（`SpaceAdmission::SkippedByTheTestOnlySwitch`）关掉准入时不判。只读分配器，不读写盘。
///
/// # Errors
/// 上一版树表 0 条而准入不够 ⇒ `SpaceAdmissionRefusedBeforeAcquisition`。
fn admission_of_the_writable_mount_before_acquisition<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    start: &InstanceStart,
    instance_to_acquire: InstanceGeneration,
) -> Result<WritableMountAdmissionBeforeAcquisition, MountError> {
    match start.space_admission {
        SpaceAdmission::JudgedByTheFormula => match admit_on_every_device(
            &admission_reading_of_a_writable_mount_with_node_capacities(
                allocator,
                start.previous.version_of_trees(),
                pool.code_two_tree_node_capacities(),
                parameters.geometry.root_ring_slots_per_region,
            ),
            &no_demand_on_any_device_of(allocator),
        ) {
            Ok(()) => Ok(WritableMountAdmissionBeforeAcquisition::Admitted),
            Err(refusal) => match &start.previous {
                PreviousVersion::WithFile { .. } | PreviousVersion::WithTreesAndNoFile { .. } => Ok(
                    WritableMountAdmissionBeforeAcquisition::ShortUntilTheFloorIsRaisedAfterTheRowPublish(
                        refusal,
                    ),
                ),
                PreviousVersion::WithoutFile { .. } => {
                    Err(MountError::SpaceAdmissionRefusedBeforeAcquisition {
                        instance_to_acquire,
                        refusal,
                    })
                }
            },
        },
        SpaceAdmission::SkippedByTheTestOnlySwitch => {
            Ok(WritableMountAdmissionBeforeAcquisition::NotJudgedByTheTestOnlySwitch)
        }
    }
}

/// 取号之后要发的那一串（写行一次、暖机那几次）在取号之前整串预演一遍（[`dry_run_of_the_publishes_after_acquisition`]：分配器的拷贝上，
/// 走发布路径落盘之前那一段，与后面真发读同一个分配器——取号不碰它）：释放核验、树的形状、落点取不到，都在任何写之前返回，按被拒的是哪一次
/// 发布与为什么装成 `MountError`。落点取不到怎么算进取号之前的准入，条款没定（`PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 的文档注释）。
///
/// # Errors
/// 落点取不到 ⇒ `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided`；写行那一次被别的理由拒 ⇒ `RowPublishAdmissionRefusedBeforeAcquisition`；
/// 暖机那几次被别的理由拒 ⇒ `WarmUpAdmissionRefusedBeforeAcquisition`。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：写入口、分配器、挂载起点、实例表要写成什么样、要写几行、暖机推哪几次、要取的号"
)]
fn publishes_after_acquisition_rehearsed<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    start: &InstanceStart,
    instance_table_rewrite: &InstanceTableRewrite,
    rows_to_write: usize,
    warm_up_publishes_planned: &[CheckpointTxg],
    instance_to_acquire: InstanceGeneration,
) -> Result<PublishesRehearsedBeforeAcquisition, MountError> {
    dry_run_of_the_publishes_after_acquisition(
        pool,
        allocator,
        start,
        instance_table_rewrite,
        rows_to_write,
        warm_up_publishes_planned,
        instance_to_acquire,
    )
    .map_err(|refusal| match refusal {
        DryRunRefusal {
            publish_index,
            cause: PublishError::PlacementRefused { unit, refusal },
        } => MountError::PlacementRefusedBeforeAcquisitionMountAdmissionUndecided {
            instance_to_acquire,
            publish_index,
            warm_up_publishes_planned: warm_up_publishes_planned.len(),
            unit,
            refusal,
        },
        DryRunRefusal {
            publish_index: 0,
            cause,
        } => MountError::RowPublishAdmissionRefusedBeforeAcquisition {
            instance_to_acquire,
            cause,
        },
        DryRunRefusal {
            publish_index: warm_up_publish_index,
            cause,
        } => MountError::WarmUpAdmissionRefusedBeforeAcquisition {
            instance_to_acquire,
            warm_up_publish_index,
            warm_up_publishes_planned: warm_up_publishes_planned.len(),
            cause,
        },
    })
}

/// 可写挂载准入的逐盘核（D2（RAID 条带策略） 已定项 13、D18（块里携带什么信息） 已定项 11）：取号是这次挂载的第一个写，
/// 核在它之前、读的是紧接着要写的这批盘（读盘，不经这次挂载共用的那一份）；有一块不带所选那一版就不取号，盘上逐字节不变。
/// 落后支缺根那一半只核写进了根环的所选那一版：择到的那条根上没施加 journal 记录才是它（施加过的那一版是崩在根槽 FUA 之前的那次发布，
/// 哪块盘的根环里都没有它，那一形是正常的崩溃状态）。交回过了补写闸、两槽全废的盘（丙'，取号写两轮顺带补写它们），
/// 连同每块盘每个系统配置槽读的那一刻的结论（改法丙，取号之前预演续做判两本账时用）。
/// 落后的盘上窗口之外坏了的那几份（[`units_of_the_version_missing_on_a_device_behind_it`]）放行时记进 `allocator` 上别的读那一份账；拒了分配器随错丢掉。
///
/// # Errors
/// 一块盘的系统配置槽在读阶段之后读不出 ⇒ [`refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage`] 那一种；
/// 有盘不带所选那一版 ⇒ `WritableMountRefusedByDevicesWithoutTheSelectedVersion`。
fn devices_passing_the_check_before_the_acquisition<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    start: &InstanceStart,
    units_of_the_selected_version: &[PublishedUnit],
    read_route: &Cell<ReadRouteOfThisMount>,
) -> Result<DevicesPassingTheCheckBeforeTheAcquisition, MountError> {
    let root_of_the_selected_version = if start.effective_root.instance
        == start.chosen_root.instance
        && start.effective_root.checkpoint_txg == start.chosen_root.checkpoint_txg
    {
        RootOfTheVersionInTheRing::Written {
            instance: start.effective_root.instance,
            checkpoint_txg: start.effective_root.checkpoint_txg,
        }
    } else {
        RootOfTheVersionInTheRing::NotWritten
    };
    let reader_of_the_check =
        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(&*pool.devices);
    let DevicesCheckedBeforeTheAcquisition {
        devices_without,
        devices_rewritten_by_the_acquisition,
        system_configuration_slots_as_checked,
    } = with_reads_straight_to_the_disk(read_route, || {
        devices_without_the_selected_version(
            &reader_of_the_check,
            &start.system_configuration,
            SelectedVersionCheckedOnEachDevice {
                root: &start.effective_root,
                journal_position: start.selected_version_journal_position,
                units: units_of_the_selected_version,
                root_in_the_ring: root_of_the_selected_version,
            },
        )
    })
    .map_err(|(device, not_rewritten_because)| {
        refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
            start.rereads.read_stage,
            SystemConfigurationUnreadableWhen::CheckingEachDeviceBeforeTheAcquisition {
                device,
                not_rewritten_because,
            },
        )
    })?;
    if !devices_without.is_empty() {
        return Err(
            MountError::WritableMountRefusedByDevicesWithoutTheSelectedVersion {
                selected_version: RollbackTarget {
                    instance: start.effective_root.instance,
                    checkpoint_txg: start.effective_root.checkpoint_txg,
                },
                selected_version_journal_position: start.selected_version_journal_position,
                devices: devices_without,
            },
        );
    }
    allocator.count_the_other_unit_reads_with_one_copy_failing(
        reader_of_the_check.locations_of_the_units_read_with_one_copy_failing(),
    );
    Ok(DevicesPassingTheCheckBeforeTheAcquisition {
        devices_rewritten_by_the_acquisition,
        system_configuration_slots_as_checked,
    })
}

/// 取号之前逐盘核放行时交回的（[`devices_passing_the_check_before_the_acquisition`]）：过了补写闸的盘（交给取号补写，预演与真做照它走），
/// 与每块盘每个系统配置槽读的那一刻的结论（只交取号之前预演续做判两本账，[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）。
struct DevicesPassingTheCheckBeforeTheAcquisition {
    devices_rewritten_by_the_acquisition: Vec<DeviceIdentity>,
    system_configuration_slots_as_checked: Vec<SystemConfigurationSlotAsChecked>,
}

/// `body` 里的读穿到盘上（[`ReadRouteOfThisMount::StraightToTheDisk`]），做完切回进来之前那一路。
fn with_reads_straight_to_the_disk<Outcome>(
    read_route: &Cell<ReadRouteOfThisMount>,
    body: impl FnOnce() -> Outcome,
) -> Outcome {
    let route_before = read_route.replace(ReadRouteOfThisMount::StraightToTheDisk);
    let outcome = body();
    read_route.set(route_before);
    outcome
}

/// 取号那几次读（[`acquire_then_publish_until_the_continuation`]）走哪一路。
pub(super) enum ReadsOfTheAcquisition<'route> {
    /// 真做：取号写之前那几次读（重算要取的号、读上一个号、屏障前后各读一次见证值）穿到盘上；取号写起切回共用的那一份
    /// （写系统配置槽时算生效 F 读的根环与系统配置，预演那一趟读过、记下了）。取号写算世代号、覆写哪一槽那一读不归这一格切：
    /// 写系统配置槽自己这一刻读盘（`BlockDevice::read_at_straight_from_the_disk`），算进取号写之前那几次读。
    StraightToTheDiskOfTheRealRun(&'route Cell<ReadRouteOfThisMount>),
    /// 取号之前只进内存那一趟：读没写过的块照它底下那一份的路读（共用的那一份）。
    ThroughTheDevicesOfTheRehearsal,
}

/// 取号、写行、暖机，取号之前不够的再推抬 F（`push_floor_raises_after_the_row_publish`）：可写挂载取号之后、续做之前那一串。
/// 可写挂载真发走它（`establish_instance`）；取号之前预演续做要推抬 F 时，在只进内存的盘与分配器的一份拷贝上也走它
/// （[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]），两边是同一段代码。
/// 写入口在这里按 `devices` 开，与取号之前预演那一串的写入口同一份参数、同一档节点容量（都是 `PoolWriter::new` 的那一档）。
///
/// # Errors
/// 取号的错（号变了、到顶、系统配置读不出、取号自己的错）；写行与暖机发不出去（`MountError::Publish`，带这次挂载的写账）；
/// 写行之后推抬 F 时抬 F 自己报的错（[`push_floor_raises_after_the_row_publish`]）。
pub(super) fn acquire_then_publish_until_the_continuation<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    mut allocator: PoolAllocator,
    start: &InstanceStart,
    rows_written: &[InstanceRow],
    publishes_planned: PublishesPlannedAfterAcquisition,
    reads_of_the_acquisition: ReadsOfTheAcquisition<'_>,
) -> Result<PublishedBetweenAcquisitionAndTheContinuation, MountError> {
    let PublishesPlannedAfterAcquisition {
        instance_to_acquire,
        instance_table_rewrite,
        warm_up_publishes_planned,
        placements_planned,
        admission_before_acquisition,
        devices_rewritten_by_the_acquisition,
    } = publishes_planned;
    let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
    let witnessed = match reads_of_the_acquisition {
        ReadsOfTheAcquisition::StraightToTheDiskOfTheRealRun(read_route) => {
            with_reads_straight_to_the_disk(read_route, || {
                read_the_witness_before_the_expected_acquisition_writes(
                    &mut pool,
                    instance_to_acquire,
                    &devices_rewritten_by_the_acquisition,
                )
            })
        }
        ReadsOfTheAcquisition::ThroughTheDevicesOfTheRehearsal => {
            read_the_witness_before_the_expected_acquisition_writes(
                &mut pool,
                instance_to_acquire,
                &devices_rewritten_by_the_acquisition,
            )
        }
    };
    let acquired = witnessed
        .and_then(|witnessed| write_the_witnessed_expected_acquisition(&mut pool, witnessed));
    let instance = acquired.map_err(
        |failure| match failure {
            ExpectedInstanceAcquisitionFailed::InstanceGenerationChangedBeforeWrite {
                expected,
                recomputed,
            } => MountError::InstanceGenerationChangedBeforeAcquisition {
                expected,
                recomputed,
            },
            ExpectedInstanceAcquisitionFailed::InstanceGenerationPastTheTopOfItsRange(
                InstanceGenerationPastTheTopOfItsRange { highest_on_disk },
            ) => MountError::SequenceNumberPastTheTopOfItsRange(
                SequenceNumberAtTheTopOfItsRange::InstanceGeneration(highest_on_disk),
            ),
            // 读阶段的见证读判得出、取号那一刻读见证值时判不出（R2）或这块盘一份自证过的系统配置都没有（两次读之间的瞬时读错，
            // C554 乙-配置续 Q1）：拒可写，与读阶段判据 N-配置 那一拒分开报（C664）。一个字节都没写。
            ExpectedInstanceAcquisitionFailed::SystemConfigurationWitnessUndecidableWhenReadingTheWitness {
                first_device_reporting_a_read_error,
            } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
                start.rereads.read_stage,
                SystemConfigurationUnreadableWhen::AcquisitionReadsAnUndecidableWitness {
                    first_device_reporting_a_read_error,
                },
            ),
            ExpectedInstanceAcquisitionFailed::DeviceWithoutAnySelfVerifiedSystemConfigurationSlotWhenReadingTheWitness {
                device,
            } => refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage(
                start.rereads.read_stage,
                SystemConfigurationUnreadableWhen::AcquisitionFindsADeviceWithoutAnySelfVerifiedSlot {
                    device,
                },
            ),
            ExpectedInstanceAcquisitionFailed::Acquisition(acquisition) => {
                MountError::Acquisition(acquisition)
            }
        },
    )?;
    assert_eq!(
        instance, instance_to_acquire,
        "acquire_expected_instance 写之前重算、与判定时的号不等就不写：交回的就是列行与判准入用的那个号"
    );

    // 取号之后的每一次发布失败都带着这次挂载的写入口交得出的账返回（`PublishSequenceFailed`）：写入口随错一起丢掉。
    let row_publish = match &start.previous {
        PreviousVersion::WithFile { output, .. }
        | PreviousVersion::WithTreesAndNoFile { output, .. } => {
            let published = publish_rows_on_file_version(
                &mut pool,
                &mut allocator,
                output,
                instance_table_rewrite,
                RowPublishIdentity {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            );
            published.map(PoolVersion::of_a_version_of_trees)
        }
        // 树表 0 条的一版：要写的行为空（上一个实例是 0，第一次可写挂载）就走零单元发布，新池新建文件的字节不变
        // （D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」；layout/01-first-txn.md 八「第一次之后的可写挂载（写行）」那一行 2026-09-14 的改名注）；
        // 要写的行不为空就只重写实例表那一个单元——这一版没有记账树，已定项 9 的那五样一样都不写，而 D18（块里携带什么信息） 已定项 11
        // 要求每次可写挂载都写行。
        PreviousVersion::WithoutFile { root, .. } if rows_written.is_empty() => {
            publish_without_units(
                &mut pool,
                root,
                ZeroUnitPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            )
            .map(|published| {
                // 零单元发布不经分配器：根落盘之后在这里记它盖掉的根环槽（`PoolAllocator::record_root_written_by_this_process`），
                // 记之前照 E2 重读挂载时读不出的那一槽。
                record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount(
                    &*pool.devices,
                    pool.parameters,
                    &mut allocator,
                    published.root.checkpoint_txg,
                );
                PoolVersion::WithoutFile(published)
            })
        }
        PreviousVersion::WithoutFile { root, .. } => {
            publish_instance_table_on_version_without_file(
                &mut pool,
                &mut allocator,
                root,
                InstanceTableOnlyPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance,
                    // 新实例的第一条记录，反向链恒 0（D23（journal 的角色与格式） 已定项 19 ②）。
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    instance_table: &instance_table_rewrite,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            )
            .map(PoolVersion::WithoutFile)
        }
    }
    .map_err(|cause| publish_failed_after_acquisition(&pool, &[], cause))?;

    // 暖机（D16（发布语义） 已定项 8 戊）：本实例的根覆盖两块盘之前连推空发布，推哪几次按取号之前算好的那一份计划
    // （`warm_up_publishes_planned`），不在这里另算一遍——两处各算各的，只要有一处多算一次，取号之前那道准入就罩不住实际推的那几次。
    assert_eq!(
        row_publish.root().checkpoint_txg,
        start.first_txg,
        "写行那次发布的 txg 就是算暖机计划用的那个：两条路（带文件的写行、树表 0 条的零单元发布）交给发布路径的 txg 都取 start.first_txg"
    );
    let mut current = row_publish.clone();
    let mut warm_up_publishes = Vec::new();
    for planned_txg in &warm_up_publishes_planned {
        assert_eq!(
            checkpoint_txg_of_the_planned_publish_after(current.root().checkpoint_txg),
            *planned_txg,
            "这次空发布要用的 txg 与计划里的那一个相同：`publish_empty_after` 取的是现行那一版的 txg 加一，计划也是逐个加一"
        );
        let next = publish_empty_after(&mut pool, &mut allocator, &current, instance).map_err(
            |cause| {
                let persisted: Vec<&PoolVersion> = std::iter::once(&row_publish)
                    .chain(&warm_up_publishes)
                    .collect();
                publish_failed_after_acquisition(&pool, &persisted, cause)
            },
        )?;
        warm_up_publishes.push(next.clone());
        current = next;
    }
    // 暖机走完、本实例的根在每块盘上都落了（`warm_up_publish_txgs` 按根环落点公式推到覆盖每块盘为止），重建分配器时扣住的那一截放开
    // （`rebuilt_allocator`，用户 2026-10-04 选甲）：此后丢哪块盘，恢复择到的都不低于本实例的根，它们不引用这一截。
    // 放开的只有那一截：扣住位只有两处会置——重建分配器那一处与抬 F 那一串（`raise_rollback_floor`），这次挂载的抬 F 都排在这一句之后
    // （写行之后的准入 `push_floor_raises_after_the_row_publish`、续做里推的那几串），它们置的扣住位归它们自己放（`raise_rollback_floor`）；
    // 走到这里分配器上的扣住位全是重建那一处置的。暖机没走完（取号之前被拒、写行或暖机落盘途中报错）时挂载整次报错，分配器随错丢掉，
    // 扣住的那一截不交给任何会话。
    allocator.release_reclaim_holds();
    // 取号之前在拷贝上取的落点就是真发起来取的：同一个分配器、同一串分配动作。这一串里有一份换下的单元读盘核校验和对不上时不比——
    // 真发时那一份的分配记录留在已分配，拷贝上没做那一步、照常释放，那一槽若在这一串里被回收（崩溃恢复落到环里最旧的那条根、其余全被抛弃），两边可以不同
    // （`dry_run_of_the_publishes_after_acquisition` 的文档注释）；那一格取号之前判的与真发的不是同一串，是这一版留着的缺口。
    let placements_taken: Vec<PlacementsTakenByOnePublish> = std::iter::once(&row_publish)
        .chain(&warm_up_publishes)
        .map(placements_taken_by)
        .collect();
    let some_copy_was_quarantined = std::iter::once(&row_publish)
        .chain(&warm_up_publishes)
        .filter_map(PoolVersion::version_of_trees)
        .any(|version| {
            !version
                .quarantined_after_release_checksum_mismatch
                .is_empty()
        });
    if let (Some(planned), false) = (&placements_planned, some_copy_was_quarantined) {
        assert_eq!(
            &placements_taken, planned,
            "取号之前在分配器的拷贝上取的落点与真发起来取的逐次相同：同一个分配器、同一串分配动作，这一串里没有一份被隔离；\
             不同说明两处的角色、次序或记根分叉了，取号之前那一判判的不是真发的那一串"
        );
    }

    // 写入口上写系统配置槽之前读到只坏一份的那一份账到这里没有要记的：取号写读到的由接着那次经分配器的发布转交（`transaction::persist_the_publish_or_freeze_it`）；
    // 接着是零单元发布只有要写的行为空这一种，即盘上没有实例代号大于 0 的系统配置与根（`transaction::next_instance_generation_to_acquire`），最新那条根指着 mkfs 那张表，
    // 这一串读的就是取号之前取单元清单读过的那一片，读到只坏一份也是账上那一组；写过行之后的暖机读这次挂载自己写的那张表，读的是写下的那一份、不到盘。
    let space_admission = match admission_before_acquisition {
        WritableMountAdmissionBeforeAcquisition::Admitted => {
            MountSpaceAdmission::AdmittedBeforeAcquisition
        }
        WritableMountAdmissionBeforeAcquisition::NotJudgedByTheTestOnlySwitch => {
            MountSpaceAdmission::NotJudgedByTheTestOnlySwitch
        }
        WritableMountAdmissionBeforeAcquisition::ShortUntilTheFloorIsRaisedAfterTheRowPublish(
            refusal_before_acquisition,
        ) => {
            let current_file_version = match &mut current {
                PoolVersion::WithFile(current_file_version)
                | PoolVersion::WithTreesAndNoFile(current_file_version) => current_file_version,
                PoolVersion::WithoutFile(_) => panic!(
                    "取号之前不够、要写行之后推的只有上一版有树的那一格（树表 0 条的在取号之前拒了）：\
                     有树的一版上写行与暖机交回的都是有树的一版（`publish_rows_on_file_version`、`publish_empty_after`）"
                ),
            };
            // 这次挂载已落盘的发布（写行那次、暖机每次）各自的写：抬 F 报错时随错交出。
            let writes_of_persisted_publishes: Vec<WritesByStructureKind> =
                std::iter::once(&row_publish)
                    .chain(&warm_up_publishes)
                    .map(|version| version.writes().clone())
                    .collect();
            // 写行与暖机那几次的写入口装的节点容量：之后推抬 F、再判的读数按同一档算。
            let node_capacities_of_the_writer = pool.code_two_tree_node_capacities();
            // 抬 F 就地推进现行那一版：它与写行那一版或最后一次暖机指着同一份，先拷一份自己的再推，交回的那几版不动。
            push_floor_raises_after_the_row_publish(
                parameters,
                devices,
                &mut allocator,
                Arc::make_mut(current_file_version),
                start.shadow_ledger,
                node_capacities_of_the_writer,
                refusal_before_acquisition,
                writes_of_persisted_publishes,
            )?
        }
    };
    Ok(PublishedBetweenAcquisitionAndTheContinuation {
        instance,
        row_publish,
        warm_up_publishes,
        space_admission,
        current,
        allocator,
    })
}

/// 可写挂载取号之前那一判的结局（`establish_instance` 用，交回之前换成 [`MountSpaceAdmission`]）。
#[derive(Clone)]
pub(super) enum WritableMountAdmissionBeforeAcquisition {
    Admitted,
    /// 不够，上一版带文件：写行与暖机照做，之后推抬 F 的空发布再判（`push_floor_raises_after_the_row_publish`）。
    ShortUntilTheFloorIsRaisedAfterTheRowPublish(AdmissionRefusedOnSomeDevices),
    NotJudgedByTheTestOnlySwitch,
}

/// 每块盘的需求都是 0：可写挂载判「实例切换的预留拿得到」这一刻不另要需求。
fn no_demand_on_any_device_of(allocator: &PoolAllocator) -> Vec<DemandOnDevice> {
    allocator
        .devices
        .iter()
        .map(|device_map| DemandOnDevice {
            device: device_map.device,
            bytes: BytesOnOneDevice::ZERO,
        })
        .collect()
}

/// 可写挂载那一处推（D16（发布语义） 已定项 1「准入」那一行，用户 2026-09-26 定）：取号之前不够的挂载，写行与暖机之后按
/// 可写挂载的读数（这次挂载的 rows0，`admission::admission_reading_of_a_writable_mount_with_node_capacities`，节点容量取
/// `node_capacities`：写行与暖机那几次的写入口装的那一档）再判，不够就推一串抬 F 的空发布
/// （[`push_one_floor_raise_within_the_admission_budget`]，一次准入最多 [`PUBLISHES_PER_ADMISSION_AT_MOST`] 次发布，写行那次算一次）、
/// 再判，直到够了或不再推。推满仍不够时怎么收尾条款没定（C565（挂载处推满仍不够怎么收尾没定））：这里取实现员提的那一种——
/// 挂载照样交回做成（[`MountSpaceAdmission::StillShortAfterTheFloorRaises`]），实例已取、行已写，之后的发布照发布路径的准入判，
/// 正常卸载照常做成（它抬 F 到现行那一版、不判上限，预演算上新 F 回收出来、当场放开的那一截，[`unmount`]，用户 2026-10-03 定甲 GA）；交主 agent 定。
///
/// 循环：每一轮先判，够了就交回；不够就推一串，推成了（至少一次空发布、`publishes_pushed` 增加）进下一轮，推满了就交回
/// （抬 F 那一串自己也取不到落点、被空间准入拒，同样是推满了：[`FloorRaiseStop::FloorRaiseRefusedForSpace`]），
/// 抬 F 自己报别的错就把错原样交回（代码审阅第 23 条：块设备错不是「推满仍不够」，不走 C565 那一格，D2（RAID 条带策略） 已定项 13
/// 那一格的例外只管空间不够）。
/// 预算把推的次数压在 B − 1 以内 ⇒ 至多 7 轮；跨轮携带的是现行那一版（抬 F 就地推进它）与已推的几串。
///
/// # Errors
/// 抬 F 自己报的、不是空间不够的错（[`push_one_floor_raise_within_the_admission_budget`] 的 `Err`）：装进
/// `FloorRaiseFailedAfterTheMountsPublishes`，连同 `writes_of_persisted_publishes`（这次挂载写行与暖机那几次已落盘的写）
/// 与报错之前已推成的那几串。
#[allow(
    clippy::too_many_arguments,
    reason = "挂载推抬 F 要的：参数、盘、分配器、现行那一版、影子账那一臂、写入口的节点容量、取号之前那一判、已落盘的写，各自独立"
)]
fn push_floor_raises_after_the_row_publish<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut TransactionOutput,
    shadow_ledger: ShadowLedger,
    node_capacities: CodeTwoTreeNodeCapacities,
    refusal_before_acquisition: AdmissionRefusedOnSomeDevices,
    writes_of_persisted_publishes: Vec<WritesByStructureKind>,
) -> Result<MountSpaceAdmission, MountError> {
    let mut floor_raises: Vec<RaisedFloor> = Vec::new();
    let mut publishes_pushed = 0;
    loop {
        let refusal = match admit_on_every_device(
            &admission_reading_of_a_writable_mount_with_node_capacities(
                allocator,
                Some(&*current),
                node_capacities,
                parameters.geometry.root_ring_slots_per_region,
            ),
            &no_demand_on_any_device_of(allocator),
        ) {
            Ok(()) => {
                return Ok(MountSpaceAdmission::AdmittedAfterTheFloorRaises {
                    refusal_before_acquisition,
                    floor_raises,
                })
            }
            Err(refusal) => refusal,
        };
        match push_one_floor_raise_within_the_admission_budget(
            parameters,
            devices,
            allocator,
            current,
            shadow_ledger,
            publishes_pushed,
        ) {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
                publishes_pushed += raised.publishes.len();
                floor_raises.push(raised);
            }
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
                return Ok(MountSpaceAdmission::StillShortAfterTheFloorRaises {
                    refusal_before_acquisition,
                    floor_raises,
                    last_refusal: refusal,
                    stop,
                })
            }
            Err(cause) => {
                return Err(MountError::FloorRaiseFailedAfterTheMountsPublishes(
                    Box::new(FloorRaiseFailedAfterTheMountsPublishes {
                        cause,
                        writes_of_persisted_publishes,
                        floor_raises,
                    }),
                ))
            }
        }
    }
}

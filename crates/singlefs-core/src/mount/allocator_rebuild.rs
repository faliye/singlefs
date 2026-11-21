//! 可写挂载重建分配器：从上一版的账重建、按可再分配谓词的门槛回收已释放的槽、根环表里每一槽住着什么，
//! 连同进程内重读挂载那一刻读坏的根环槽、记下这个进程写的根与回收门槛。

use super::*;

/// 进程内重读一个挂载那一刻读坏的根环槽（D16（发布语义） 已定项 1 的 E2，用户 2026-10-03 定）：只读一次，不重试；读得出、自证过、
/// 按 `table` 仍然有效 ⇒ 照它的 txg；读得出、自证过而按 `table` 被抛弃 ⇒ 当没有根；读回全零 ⇒ 不算确认为空，照旧钉在它可能住过的最旧 txg
/// （用户 2026-10-05 定乙：读回全零的槽在回收门槛里照「自证不过」那一类钉住；准入抬 F 的上限也并它，用户 2026-11-02 定「并」）；设备报读错或读得出、不是全零而自证不过
/// ⇒ 照旧钉在它可能住过的最旧 txg（用户 2026-10-04 定改法 A，`research/prompts/m3-root-ring-code-r2-main-verification.md` 第六节 RS1：
/// 这一刻读到「自证不过」可能是一次读坏的有效根，当没有根在这上面让回收门槛越过盘上一条有效根）。
#[must_use]
pub fn reread_of_a_root_ring_slot_bad_at_mount<Reader: PoolReader + ?Sized>(
    reader: &Reader,
    parameters: &MakeFilesystemParameters,
    table: &InstanceTableRecords,
    ring_slot: RootRingSlot,
) -> RereadOfARootRingSlotBadAtMount {
    match read_root_ring_slot_telling_never_written_apart(
        reader,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
        ring_slot,
    ) {
        RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(root)
            if !abandoned_by_table(&root, table) =>
        {
            RereadOfARootRingSlotBadAtMount::ValidRoot {
                checkpoint_txg: root.checkpoint_txg,
            }
        }
        RootRingSlotReadingTellingNeverWrittenApart::SelfVerified(_) => {
            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable
        }
        RootRingSlotReadingTellingNeverWrittenApart::AllZeroNeverWritten => {
            RereadOfARootRingSlotBadAtMount::AllZero
        }
        RootRingSlotReadingTellingNeverWrittenApart::BadAndNotAllZero => {
            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
        }
    }
}

/// 这个进程写了一条根：记进分配器（[`PoolAllocator::record_root_written_by_this_process`]），记之前照 E2 重读挂载那一刻读坏、
/// 记了这条根之后会是环里最小的那个槽（[`PoolAllocator::record_root_written_by_this_process_rereading_slots_bad_at_mount`]；
/// D16（发布语义） 已定项 1，用户 2026-10-03 定；读坏的槽钉在它可能住过的最旧 txg，用户 2026-10-04 定）。名字里的「读不出」是读坏的两类
/// （读不出、不是全零而自证不过）合起来说的：`transaction` 里两处调用点按这个名字调，这一批不改那份文件。
/// 判「按实例表仍然有效」用分配器根环表上记着的那一张
/// （`RootRingOccupancy::instance_table_judging_the_rereads`）。产品路径上记根的每一处都走它：带单元的发布在取完落点之后
/// （`transaction::publish_admitted`、`transaction::prepare_the_row_publish_on_a_version_without_file`），零单元发布在根落盘之后。
/// 抬 F 那一串在分配器拷贝上的整串预演（`rehearse_the_publishes_raising_the_floor`）不重读：它按钉着的最旧 txg 算、回收得只会更少，
/// 预演过得去真发就过得去。
pub fn record_root_written_by_this_process_rereading_root_ring_slots_unreadable_at_mount<
    Reader: PoolReader + ?Sized,
>(
    reader: &Reader,
    parameters: &MakeFilesystemParameters,
    allocator: &mut PoolAllocator,
    checkpoint_txg: CheckpointTxg,
) {
    let table = allocator.root_ring_occupancy().map_or_else(
        || InstanceTableRecords { rows: Vec::new() },
        |occupancy| occupancy.instance_table_judging_the_rereads().clone(),
    );
    allocator.record_root_written_by_this_process_rereading_slots_bad_at_mount(
        checkpoint_txg,
        &mut |ring_slot| {
            reread_of_a_root_ring_slot_bad_at_mount(reader, parameters, &table, ring_slot)
        },
    );
}

/// 可再分配谓词的门槛（D16（发布语义） 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」，用户 2026-10-04 定 RGall：
/// 从一圈窗口退回整个环，`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1）：根环 R × S 槽（S 读自系统配置），第 0 代根被盖之前
/// 环里最旧有效根恒 0、门槛就是 F_生效；盖掉之后由环里最旧的有效根接管。调用方交进来的 `oldest_valid_root` 取的是环里全部有效根
/// （挂载时 `rebuilt_allocator`，进程内 `allocator::RootRingOccupancy`，抬 F 时 `raise_rollback_floor`），不按一圈窗口滤。
#[must_use]
pub fn reclaim_floor(
    effective_floor: CheckpointTxg,
    oldest_valid_root: Option<CheckpointTxg>,
) -> CheckpointTxg {
    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
}

/// 一条根按一张实例表判是不是被抛弃的（`recovery::root_is_abandoned_by_the_instance_table`，一处定义）。
pub(super) fn abandoned_by_table(root: &RootRecord, table: &InstanceTableRecords) -> bool {
    root_is_abandoned_by_the_instance_table(root, table)
}

/// 重建分配器判哪几条根被抛弃用的那张表：读阶段择到的那条根 `root_chosen_by_the_read_stage` 指着的实例表（代码审阅第 22 条）。
/// 判退那一形（读阶段择到的根走读失败、退到更旧的一版）照样用它：判退本身按所选根的实例表滤（`recovery::fall_back_judged_on_the_three_readers`），
/// 前后同一张。读阶段之后不另择根：读阶段之后读坏它的根槽时现择会落到更旧的一条根上，那条根的表可能还没有抛弃它自己那条时间线的行。
/// 这一次读不出（那张表沿链有一片读不出、解不开）就重读一次（D16（发布语义） 已定项 1「重读一次」；C554 乙，R = 1），重读之前调 `before_the_one_reread`。
///
/// # Errors
/// 重读仍读不出 ⇒ `NewerStateStillUnreadableAfterOneReread(InstanceTableOfTheNewestRootForTheShadowLedger)`，点名读阶段择到的那条根：
/// 不按「没有一条根被抛弃」往下走（那样一个槽都不隔离）。可写挂载在取号之前调它，拒的时候盘上逐字节不变。
fn instance_table_of_the_newest_root_read_at_most_twice(
    reader: &dyn PoolReader,
    root_chosen_by_the_read_stage: &RootRecord,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<(InstanceTableRecords, InstanceTableOfTheNewestRootRead), MountError> {
    if let Some(table) = instance_table_of_root(reader, root_chosen_by_the_read_stage) {
        return Ok((table, InstanceTableOfTheNewestRootRead::OnTheFirstRead));
    }
    before_the_one_reread.before_rereading();
    match instance_table_of_root(reader, root_chosen_by_the_read_stage) {
        Some(table) => Ok((table, InstanceTableOfTheNewestRootRead::OnTheOneReread)),
        None => Err(MountError::NewerStateStillUnreadableAfterOneReread(
            Box::new(
                StillUnreadableAfterOneReread::InstanceTableOfTheNewestRootForTheShadowLedger {
                    root_chosen_by_the_read_stage: RollbackTarget {
                        instance: root_chosen_by_the_read_stage.instance,
                        checkpoint_txg: root_chosen_by_the_read_stage.checkpoint_txg,
                    },
                },
            ),
        )),
    }
}

/// 重建分配器：从上一版的分配记录重建，按可再分配谓词的门槛回收（当场可用的只到每块盘上最新有效根那一截、某块盘一条有效根都读不出时
/// 只到环里最旧有效根（RGall），其余扣到这次挂载的暖机落满每块盘，甲），再把只被被抛弃根引用的槽隔离（影子账：被抛弃的根按最新根指着的
/// 实例表判，D23（journal 的角色与格式） 已定项 14 射程「影子账与按实例表判抛弃留着，理由是崩溃恢复」）。
/// 影子账只住内存，所以每次挂载都要重算（步 4 / 步 5 代码三方第一轮云端攻方腿打中：普通重开一次隔离就归零）。
/// 每块盘的空闲图从择到的系统配置按环长现算的单元区起点起（与偏移 417 那 8 字节相等，择的时候判过；C475（非默认环长下单元区起点取编译期常量））。
/// 生效的回退下界 F 判「有效根」用的也是影子账那一张表（读阶段择到的那条根 `root_chosen_by_the_read_stage` 的实例表，读不出重读一次）：
/// 不另读一遍、也就没有「读不出就不按表滤」那一支（C554 乙报告 Q6）。
///
/// # Errors
/// 读阶段择到的那条根的实例表读不出、重读一次仍读不出（[`instance_table_of_the_newest_root_read_at_most_twice`]）——在读阶段之后、取号之前；
/// 树表 0 条那一版的账重建不出（`allocator_of_version_without_file`）；
/// 读根环那一遍有一块盘一条不被抛弃的自证有效根都没读出、整环重读一次，两遍逐槽并起来整个环仍一条都没有 ⇒
/// `NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread`（用户 2026-10-05 定「重读」、Q3 另起拒因、Q4 rr2、R1 按槽并两遍）；
/// C 读一次没施加的发布或一条较新可读根（读阶段读到的根也算：读根环那一遍读它的根槽读坏时照读阶段读到的那一份）的账读不出、
/// 那次发布读得出的记录不齐（较新可读根那次发布一条记录都读不出、树表与中央映射树根
/// 两个指针都没有两份都是本池更新实例的单元的，也算这一支；别的池的单元头不算）、从中央映射也现算不成、重读一次仍不成
/// （[`placements_of_an_unapplied_publish_whose_account_is_unreadable`]）⇒
/// `NewerStateStillUnreadableAfterOneReread(AccountOfAnUnappliedPublishWithIncompleteReadableRecords)`。
/// 都在取号之前，盘上逐字节不变。
#[allow(
    clippy::ptr_arg,
    reason = "读盘的几处走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
#[allow(
    clippy::too_many_arguments,
    reason = "每个参数各是一样东西：设备表、系统配置、上一版、新实例第一次发布的 txg、读阶段扫出的记录、所选那一版、读阶段择到的根、读阶段读到的根、影子账开关、重读前的钩子"
)]
pub(super) fn rebuilt_allocator<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    system_configuration: &crate::system_configuration::SystemConfiguration,
    previous: &PreviousVersion,
    first_txg: CheckpointTxg,
    records: &BTreeMap<(InstanceGeneration, u64), crate::journal::JournalRecord>,
    selected_version: &RootRecord,
    root_chosen_by_the_read_stage: &RootRecord,
    roots_read_by_the_read_stage: &[RootRecord],
    shadow_ledger: ShadowLedger,
    before_the_one_reread: &mut BeforeTheOneReread<'_>,
) -> Result<RebuiltAllocator, MountError> {
    let unit_area_start = unit_area_start_of_the_chosen_system_configuration(system_configuration);
    let mut allocator = allocator_of_the_previous_version(devices, unit_area_start, previous)?;
    // 读读阶段择到的那条根的实例表与读各根的账（候选根、被抛弃根、没施加的发布、较新可读根、回退目标）读到只坏一份的进这次挂载别的读那一份账（D4（校验和位置） 已定项 4「可写挂载里的读都计数并搬」）：
    // 同一个单元建上一版已经读到的归挂载那个数，不再算；读的是别的版本的，下一次用户改动之后那一批认不出、只留下计数。
    let devices_counting_unit_reads_with_one_copy_failing =
        PoolReaderCountingUnitReadsWithOneCopyFailing::counting_from_zero(devices);
    let current_records = allocator.records().to_vec();
    let (newest_table, instance_table_of_the_newest_root) =
        instance_table_of_the_newest_root_read_at_most_twice(
            &devices_counting_unit_reads_with_one_copy_failing,
            root_chosen_by_the_read_stage,
            before_the_one_reread,
        )?;
    let effective_floor = effective_rollback_floor_under_the_newest_roots_table(
        devices,
        &system_configuration.immutable.region_devices,
        &system_configuration.immutable.sizes,
        &system_configuration.immutable.filesystem_identifier,
        &newest_table,
    );
    let slots_per_region = system_configuration
        .immutable
        .sizes
        .root_ring_slots_per_region;
    let root_ring = root_ring_read_once_or_twice_for_the_rebuild(
        devices,
        system_configuration,
        &newest_table,
        before_the_one_reread,
    )?;
    let sorted = RootRingReadingsSortedForTheRebuild::of(
        &root_ring.ring_slot_readings,
        root_ring.window_anchor,
        first_txg,
        slots_per_region,
    );
    let is_abandoned = |root: &RootRecord| abandoned_by_table(root, &newest_table);
    reclaim_the_released_slots_of_the_rebuilt_allocator(
        &mut allocator,
        &sorted,
        &root_ring.ring_slot_readings,
        effective_floor,
        &newest_table,
        devices,
        system_configuration,
    );
    // C694 的 C（判决 `research/prompts/m3-c694-r2-main-verification.md` 第三节第 1 条）：水位之上没施加的记录点名的落点与影子账同一次隔离、
    // 豁免同一个集合。影子账关着（只供测试）时它也不隔离：那一臂要的是「什么都不护」的对照。
    // C 并的较新可读根取这一遍读出的根并上读阶段读到的全部根（判退退掉的根与它和退到的那一版之间读阶段读得出的根）：这一遍读它们的根槽读坏时
    // 它们仍在回退候选集里，漏了哪一条，它的单元就不进 C、写行那次发布照样写掉（I-7.4，[`roots_of_the_rebuild_pass_with_the_roots_read_by_the_read_stage`]）。
    let roots_for_the_newer_readable_roots =
        roots_of_the_rebuild_pass_with_the_roots_read_by_the_read_stage(
            &sorted.roots,
            roots_read_by_the_read_stage,
        );
    let placements_named_by_unapplied_journal_records = match shadow_ledger {
        ShadowLedger::On => placements_of_unapplied_publishes_and_newer_readable_roots(
            &devices_counting_unit_reads_with_one_copy_failing,
            system_configuration,
            unit_area_start,
            records,
            selected_version,
            &roots_for_the_newer_readable_roots,
            &is_abandoned,
            &current_records,
            before_the_one_reread,
        )?,
        ShadowLedger::Off => BTreeMap::new(),
    };
    let shadow_ledger_computed = match shadow_ledger {
        ShadowLedger::On => isolate_slots_referenced_only_by_abandoned_roots(
            &devices_counting_unit_reads_with_one_copy_failing,
            unit_area_start,
            &mut allocator,
            &sorted.roots,
            &is_abandoned,
            effective_floor,
            &current_records,
            &placements_named_by_unapplied_journal_records
                .values()
                .flatten()
                .copied()
                .collect::<Vec<PlacementOnDevice>>(),
            PlacementsOfAbandonedRoots::ReadFromTheirAccounts,
            before_the_one_reread,
            &[],
        )?,
        ShadowLedger::Off => ShadowLedgerComputed {
            abandoned_roots_unreadable: 0,
            placements_referenced_by_abandoned_root: BTreeMap::new(),
        },
    };
    let units_read_while_building_the_previous_version =
        previous.units_read_with_one_copy_failing_while_building_on_it();
    allocator.count_the_other_unit_reads_with_one_copy_failing(
        devices_counting_unit_reads_with_one_copy_failing
            .locations_of_the_units_read_with_one_copy_failing()
            .into_iter()
            .filter(|locations| !units_read_while_building_the_previous_version.contains(locations))
            .collect(),
    );
    let ShadowLedgerComputed {
        abandoned_roots_unreadable,
        placements_referenced_by_abandoned_root,
    } = shadow_ledger_computed;
    let occupants = root_ring_occupants_of_the_rebuild(
        &sorted,
        &root_ring.ring_slots_self_verified_on_only_one_of_the_two_passes,
        &placements_referenced_by_abandoned_root,
        &is_abandoned,
    );
    allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(
        slots_per_region,
        occupants,
        effective_floor,
        CheckpointTxg(
            first_txg
                .0
                .checked_sub(1)
                .expect("新实例第一次发布的 txg = 环里最大 txg + 1 ≥ 1"),
        ),
    ));
    allocator.note_placements_named_by_unapplied_journal_records(
        placements_named_by_unapplied_journal_records,
    );
    Ok(RebuiltAllocator {
        allocator,
        effective_floor,
        abandoned_roots_unreadable,
        instance_table_of_the_newest_root,
        root_ring_of_the_rebuilt_allocator: root_ring.read,
    })
}

/// 从上一版的账重建的分配器：有树的一版从它的分配记录重建，树表 0 条那一版走 [`allocator_of_version_without_file`]。
/// 每块盘的空闲图从择到的系统配置按环长现算的单元区起点起（盘够不够长到单元区起点，可写挂载在读根环之前判过，
/// `every_device_reaches_the_unit_area_start`，同一个起点）。
///
/// # Errors
/// 树表 0 条那一版的账重建不出。
#[allow(
    clippy::ptr_arg,
    reason = "allocator_of_version_without_file 走 &Vec 那一份读者：Vec 实现了 PoolReader、切片不能转成 dyn"
)]
fn allocator_of_the_previous_version<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    previous: &PreviousVersion,
) -> Result<PoolAllocator, MountError> {
    let device_maps: Vec<DeviceFreeMap> = devices
        .iter()
        .map(|(identity, device)| {
            DeviceFreeMap::with_unit_area_start(*identity, device.size_in_bytes(), unit_area_start)
        })
        .collect();
    match previous {
        PreviousVersion::WithFile { output, .. }
        | PreviousVersion::WithTreesAndNoFile { output, .. } => Ok(
            PoolAllocator::rebuild_from_records(device_maps, output.allocation_records.clone()),
        ),
        PreviousVersion::WithoutFile { root, .. } => {
            allocator_of_version_without_file(devices, unit_area_start, device_maps, root)
        }
    }
}

/// 按可再分配谓词的门槛回收重建出来的分配器上已释放的槽。环里有效根的最小 txg（D16（发布语义） 已定项 1「环里最旧有效根」，
/// 用户 2026-10-04 定 RGall：不按一圈窗口滤）；挂载时读坏的槽钉在它可能住过的最旧 txg（用户 2026-10-04 定），读回全零的槽同样钉着（用户 2026-10-05 定乙）。
///
/// 卸载半途崩溃之后的下一次可写挂载只放开一截（D16（发布语义） 已定项 1「正常卸载」那一行，用户 2026-10-04 选甲，
/// `research/prompts/m3-batch14-code-r1-forks.md`「甲 再挂也只放开一截」那一行）：卸载那一串先写了系统配置里的新 F、带新 F 的根没落满每块盘就崩时，
/// F_生效 高于某块盘上最新的有效根；恢复择根不看 F，丢掉另一块盘之后择的就是那块盘上最新的根，它引用的单元释放代可以高于它、不高于 F_生效
/// （判决 `research/prompts/m3-batch14-code-r1-main-verification.md` 第二节「形 R」那一行）。于是当场可用的只到
/// reclaim_floor(min(F_生效, m), 环里最旧有效根)（RGall），m 是每块盘上最新有效根的 txg 取最小（被抛弃的根不算，与卸载那一格同一处定义
/// [`newest_valid_root_txg_on_every_device`]）；其余到原门槛为止的那一截扣住，这次挂载的暖机在每块盘上都落了根之后放开（`establish_instance`）。
/// 某块盘一条有效根都读不出（m 取不到）时当场只放到环里最旧有效根（用户 2026-10-04 选甲，`research/prompts/m3-batch16-code-r1-forks.md`
/// 第 1 行；判决 `research/prompts/m3-batch16-code-r1-main-verification.md` 第二节「再挂期间某块盘根环整段读坏」那一行；用户 2026-10-04 定 RGall
/// 之后取整个环里的有效根、不按一圈窗口滤）：那块盘上真正最新的根这一遍看不到，它引用的单元释放代可以高到 F_生效；它与回收门槛同一个读数
/// （挂载时读坏、读回全零的槽钉在它可能住过的最旧 txg 并进）。读根环那一步保证这一份读数至少有一条有效根，环里最旧有效根恒取得到。
/// m 与判要不要重读同一个归盘法（[`roots_on_the_devices_they_were_read_from`]）。只动分配器，不读盘。
fn reclaim_the_released_slots_of_the_rebuilt_allocator<Device: BlockDevice>(
    allocator: &mut PoolAllocator,
    sorted: &RootRingReadingsSortedForTheRebuild,
    ring_slot_readings: &[(RootRingSlot, RootRingSlotReadingTellingNeverWrittenApart)],
    effective_floor: CheckpointTxg,
    newest_table: &InstanceTableRecords,
    devices: &[(DeviceIdentity, Device)],
    system_configuration: &crate::system_configuration::SystemConfiguration,
) {
    let oldest_valid_root_in_the_ring = sorted
        .roots
        .iter()
        .filter(|root| !abandoned_by_table(root, newest_table))
        .map(|root| root.checkpoint_txg)
        .chain(
            sorted
                .ring_slots_bad_at_mount
                .iter()
                .map(|(_, txgs_it_can_hold)| txgs_it_can_hold.oldest_txg_it_can_have_held()),
        )
        .chain(
            sorted
                .ring_slots_reading_back_all_zero_at_mount
                .iter()
                .map(|(_, txgs_it_can_hold)| txgs_it_can_hold.oldest_txg_it_can_have_held()),
        )
        .min();
    let all_devices: Vec<DeviceIdentity> = devices.iter().map(|(identity, _)| *identity).collect();
    let floor_of_the_reclaim = reclaim_floor(effective_floor, oldest_valid_root_in_the_ring);
    let floor_released_immediately = match newest_valid_root_txg_on_every_device(
        &roots_on_the_devices_they_were_read_from(ring_slot_readings, system_configuration),
        newest_table,
        &all_devices,
    ) {
        Some(newest_valid_root_on_every_device) => Some(reclaim_floor(
            effective_floor.min(newest_valid_root_on_every_device),
            oldest_valid_root_in_the_ring,
        )),
        None => oldest_valid_root_in_the_ring,
    };
    if let Some(released_immediately_up_to) = floor_released_immediately {
        allocator.reclaim_released_up_to(released_immediately_up_to, ReclaimedReuse::Immediately);
    }
    allocator.reclaim_released_up_to(
        floor_of_the_reclaim,
        ReclaimedReuse::HeldUntilFloorTakesEffect,
    );
}

/// 根环表里每一槽住着什么：挂载时读坏的槽记成「挂载时读坏」，带它能住的 txg（回收门槛钉在它可能住过的最旧 txg、准入抬 F 的上限按它能住的
/// 最大 txg 并）；读回全零的槽记成「挂载时读回全零」，同样带它能住的 txg、同样两处都用（回收门槛那一半用户 2026-10-05 定乙，
/// 上限那一半用户 2026-11-02 定「并」）。两类都钉到这个进程
/// 写到那个槽、或进程内重读读出有效根或被抛弃的根为止（用户 2026-10-04 定）。同一个槽这一遍读出了根的，读出来的那条在后面、盖掉它
/// （这一遍读的是同一个槽，几样不会同时成立，留着次序只为不依赖这一点）。两遍里只有一遍自证过的槽另记读出来的那整条根记录：
/// 抬 F 那一串读它读坏、重读一次仍坏时拿它顶上（用户 2026-10-06 定 T1 甲）。
fn root_ring_occupants_of_the_rebuild(
    sorted: &RootRingReadingsSortedForTheRebuild,
    ring_slots_self_verified_on_only_one_of_the_two_passes: &BTreeSet<RootRingSlot>,
    placements_referenced_by_abandoned_root: &BTreeMap<
        (InstanceGeneration, CheckpointTxg),
        Vec<PlacementOnDevice>,
    >,
    is_abandoned: &dyn Fn(&RootRecord) -> bool,
) -> Vec<(RootRingSlot, RootRingOccupant)> {
    sorted
        .ring_slots_bad_at_mount
        .iter()
        .map(|(ring_slot, txgs_it_can_hold)| {
            (*ring_slot, RootRingOccupant::BadAtMount(*txgs_it_can_hold))
        })
        .chain(sorted.ring_slots_reading_back_all_zero_at_mount.iter().map(
            |(ring_slot, txgs_it_can_hold)| {
                (
                    *ring_slot,
                    RootRingOccupant::AllZeroAtMount {
                        txgs_it_can_hold: *txgs_it_can_hold,
                    },
                )
            },
        ))
        .chain(
            sorted
                .roots_with_ring_slots
                .iter()
                .map(|(ring_slot, root)| {
                    let root_read = if is_abandoned(root) {
                        RootReadOnOneRebuildPass::AbandonedRoot {
                            referenced_placements: placements_referenced_by_abandoned_root
                                .get(&(root.instance, root.checkpoint_txg))
                                .cloned()
                                .unwrap_or_default(),
                        }
                    } else {
                        RootReadOnOneRebuildPass::ValidRoot {
                            checkpoint_txg: root.checkpoint_txg,
                        }
                    };
                    let self_verified_on_only_one_of_the_two_passes =
                        ring_slots_self_verified_on_only_one_of_the_two_passes.contains(ring_slot);
                    let occupant = match (self_verified_on_only_one_of_the_two_passes, root_read) {
                        (true, root_read) => {
                            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                                root: root_read,
                                root_record_self_verified_on_one_rebuild_pass: Box::new(*root),
                            }
                        }
                        (false, RootReadOnOneRebuildPass::ValidRoot { checkpoint_txg }) => {
                            RootRingOccupant::ValidRoot { checkpoint_txg }
                        }
                        (
                            false,
                            RootReadOnOneRebuildPass::AbandonedRoot {
                                referenced_placements,
                            },
                        ) => RootRingOccupant::AbandonedRoot {
                            instance: root.instance,
                            checkpoint_txg: root.checkpoint_txg,
                            referenced_placements,
                        },
                    };
                    (*ring_slot, occupant)
                }),
        )
        .collect()
}

/// 树表 0 条的那一版的账从哪来，只有两条路：
///
/// - 根记录那一项（分配记录树根指针）不是全零 ⇒ 这一版写过行，写行那次发布把这一版的全部分配记录写进了那棵分配记录树
///   （C512（树表 0 条的一版上被换下的单元记在哪），2026-09-23 用户定案；按绝对槽号按位置寻址，D8（核心索引结构） 已定项 14）：
///   整棵读回来重建（挂载时分配记录树整棵读，用户 K4），被换下的实例表因此带着已释放标志回来，不会被当空闲槽发出去。
///   那棵树的节点与记录记进分配器——下一次发布要照抄或换下它们，而这一版没有上一版的内存态可查。
/// - 全零 ⇒ mkfs 的第 0 代（或照抄它的暖机根）：账由实例表与树表两条指针直接算（`format_time_allocator`）。
///
/// # Errors
/// 分配记录树读不出、解不开、位置对不上、条目宽或结构值判红 ⇒ `Recovery(...)`；某个节点两条位置条目不同槽，或全零那一条上
/// 两条指针任一条不是 mkfs 写的那一版 ⇒ `VersionWithoutFileUnitPointerOfABadImage`（见 `format_time_allocator`）。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_of_version_without_file 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn allocator_of_version_without_file<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    let Some(tree) = allocation_records_of_version_without_file_in_the_unit_area_starting_at(
        devices,
        unit_area_start,
        root,
    )
    .map_err(MountError::Recovery)?
    else {
        return format_time_allocator(devices, unit_area_start, device_maps, root);
    };
    // 每个节点两条位置条目同槽（第一版两盘同槽）：下一次发布照抄或换下它们时按一个落点释放。
    for (node, pointer) in &tree.version.nodes {
        slot_shared_by_both_location_entries(&pointer.locations).map_err(|disagreement| {
            MountError::VersionWithoutFileUnitPointerOfABadImage {
                unit: role_of_allocation_record_tree_node(*node),
                defect: VersionWithoutFileUnitPointerDefect::LocationEntriesOnDifferentSlots(
                    disagreement,
                ),
            }
        })?;
    }
    let mut allocator = PoolAllocator::rebuild_from_records(device_maps, tree.records.clone());
    allocator.note_allocation_record_tree_of_the_version_without_file(tree);
    Ok(allocator)
}

/// mkfs 写出的那一版（或照抄它的暖机根）的账：盘上没有分配记录树，这一版的全部落点就是根记录直接指着的实例表与树表两个单元
/// ——mkfs 写在单元区里的那两个（字节表五里它们两盘各一条、分配代 0）。与 mkfs 同一个进程里新池新建文件用的分配器同一个起点
/// （`PoolAllocator::mark_format_time_units`），第一个文件版本换下 mkfs 那片树表时照样把它释放。
///
/// # Errors
/// 根记录指着的实例表或树表不是 mkfs 写的那一版（诞生 txg 不是 0）⇒ `VersionWithoutFileUnitPointerOfABadImage`
/// （`BirthTxgNotZeroWithoutAnAllocationRecordTree`，先判实例表、再判树表）。这一判守的是分配器的账：这里只按两条指针把两个单元记成已占，
/// 根记录指着的不是 mkfs 那两片时照这样建分配器，别的活单元会被当空闲槽发出去。实现写不出这种根：树表 0 条的一版上写行那次发布
/// 建起这一版自己的分配记录树、根指针住根记录（C512（树表 0 条的一版上被换下的单元记在哪）），走 `allocator_of_version_without_file`
/// 的第一条路、到不了这里（`the_third_writable_mount_keeps_the_instance_table_that_the_row_publish_swapped_out_out_of_the_free_pool` 钉住）；
/// 用例要造它得手抹那一项（`plant_a_root_without_its_allocation_record_tree`）。
///
/// 两个指针里任一条的两条位置条目槽号不等 ⇒ `VersionWithoutFileUnitPointerOfABadImage`（`LocationEntriesOnDifferentSlots`）；两个落点在每块盘上记成的分配记录
/// 过不了盘上分配记录那一道几何判（`recovery::allocation_records_fit_the_pool_geometry`：槽号在单元区起点之下、跨度越过单元区末尾、
/// 两个单元罩住同一个槽）⇒ `Recovery(AllocationRecordOutsideThePoolGeometry)`（代码审阅第 32 条：槽号是根记录里盘上读来的 6 字节，
/// 改之前直接进 `mark_format_time_units`，在分配器的下标与断言上 panic）。三样都在动分配器之前返回。
#[allow(
    clippy::ptr_arg,
    reason = "allocation_records_fit_the_pool_geometry 走 &dyn PoolReader，要一个有大小的读者：Vec 实现了它、切片不能转成 dyn"
)]
fn format_time_allocator<Device: BlockDevice>(
    devices: &Vec<(DeviceIdentity, Device)>,
    unit_area_start: UnitAreaStart,
    device_maps: Vec<DeviceFreeMap>,
    root: &RootRecord,
) -> Result<PoolAllocator, MountError> {
    for (unit, pointer) in [
        (TransactionUnit::InstanceTable, &root.instance_table),
        (TransactionUnit::TreeTable, &root.tree_table),
    ] {
        if pointer.head.birth_txg != CheckpointTxg(0) {
            return Err(MountError::VersionWithoutFileUnitPointerOfABadImage {
                unit,
                defect:
                    VersionWithoutFileUnitPointerDefect::BirthTxgNotZeroWithoutAnAllocationRecordTree {
                        birth_txg: pointer.head.birth_txg,
                    },
            });
        }
    }
    let placement_of = |pointer: &crate::pointer::NodePointer, identity: TransactionUnit| {
        let slot =
            slot_shared_by_both_location_entries(&pointer.locations).map_err(|disagreement| {
                MountError::VersionWithoutFileUnitPointerOfABadImage {
                    unit: identity,
                    defect: VersionWithoutFileUnitPointerDefect::LocationEntriesOnDifferentSlots(
                        disagreement,
                    ),
                }
            })?;
        Ok::<Placement, MountError>(Placement {
            slot,
            span: identity.span_slots(),
        })
    };
    let instance_table_placement =
        placement_of(&root.instance_table, TransactionUnit::InstanceTable)?;
    let tree_table_placement = placement_of(&root.tree_table, TransactionUnit::TreeTable)?;
    // `mark_format_time_units` 在每块盘上各记这两条分配代 0 的记录：进分配器之前按盘上分配记录同一道几何判一遍。
    let records_the_format_time_units_become: Vec<AllocationRecord> = device_maps
        .iter()
        .flat_map(|device_map| {
            [instance_table_placement, tree_table_placement].map(|placement| AllocationRecord {
                device: device_map.device,
                slot: placement.slot,
                span_slots: u16::try_from(placement.span).expect("单元的跨度是 1 或 2 槽"),
                generation: CheckpointTxg(0),
                is_released: false,
                quarantine_mark: crate::allocator::AllocationRecordQuarantineMark::NotQuarantined,
            })
        })
        .collect();
    allocation_records_fit_the_pool_geometry(
        devices,
        unit_area_start,
        &records_the_format_time_units_become,
    )?;
    let mut allocator = PoolAllocator::new(device_maps);
    allocator.mark_format_time_units(instance_table_placement, tree_table_placement);
    Ok(allocator)
}

#[cfg(test)]
mod reclaim_floor_tests {
    use super::reclaim_floor;
    use crate::address::CheckpointTxg;

    /// 第 0 代根还在环里时门槛就是 F_生效；被盖之后环里最旧的有效根接管（比 F 大时取它）。
    #[test]
    fn reclaim_floor_takes_the_larger_of_the_effective_floor_and_the_oldest_valid_ring_root() {
        assert_eq!(
            reclaim_floor(CheckpointTxg(11), Some(CheckpointTxg(0))),
            CheckpointTxg(11)
        );
        assert_eq!(
            reclaim_floor(CheckpointTxg(0), Some(CheckpointTxg(25))),
            CheckpointTxg(25)
        );
        assert_eq!(
            reclaim_floor(CheckpointTxg(11), Some(CheckpointTxg(25))),
            CheckpointTxg(25)
        );
        assert_eq!(reclaim_floor(CheckpointTxg(11), None), CheckpointTxg(11));
    }
}

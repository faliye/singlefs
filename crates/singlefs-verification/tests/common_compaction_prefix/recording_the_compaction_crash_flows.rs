//! 整理搬迁、活意图期间写、暂停与出错停下删意图、搬共享块、生命周期闭环这几条流从 mkfs 录到底：实例一里的发布、重挂开会话、整理发到意图删掉、
//! 意图活着时插一次写，压准入余量与只剩 R 有空槽时的暂停、注入算着出错的停下，用户改动与抬 F，生命周期闭环那一条另录回收与再分配。

use super::*;

/// 有树的一版（会话与挂载交回的现行版本）。
pub fn version_of_trees(version: &PoolVersion) -> &TransactionOutput {
    version
        .version_of_trees()
        .expect("录流在重挂之前都已经发过第一个文件：挂载与会话交回的每一版都有树")
}

/// 准入读数里每块盘的可用(d)（字节），按分配器里盘的次序：会话现行那一版之上、下一次发布之前的那一份。
fn available_bytes_on_each_device_of_the_session(
    session: &MountedSession,
) -> Vec<(DeviceIdentity, i128)> {
    admission_reading_before_a_publish(&session.allocator, session.current.file_version())
        .available_on_each_device()
        .into_iter()
        .map(|(device, available)| (device, available.0))
        .collect()
}

/// 在每块盘单元区的最高处隔离槽（只动分配器的隔离位，不写盘、不进录制流），让每块盘的可用(d) 落进
/// [`lower_bound_in_bytes`, `lower_bound_in_bytes` + 一个槽宽)；交回每块盘隔离的那一截（盘、起始槽、槽数），解开时照它清。
/// 隔离的槽在单元区最高处、离这几条流取的落点远。
fn isolate_the_highest_slots_of_the_session_until_the_available_is_within_one_slot_above(
    session: &mut MountedSession,
    lower_bound_in_bytes: i128,
) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
    let slot_bytes = i128::from(SLOT_BYTES);
    let mut isolated = Vec::new();
    for (device, available) in available_bytes_on_each_device_of_the_session(session) {
        let excess_in_bytes = available - lower_bound_in_bytes;
        assert!(
            excess_in_bytes >= 0,
            "{device:?}：可用 {available} 本来就低于要压到的 {lower_bound_in_bytes}"
        );
        let slots_to_isolate =
            u64::try_from(excess_in_bytes / slot_bytes).expect("非负、装得进 u64");
        if slots_to_isolate == 0 {
            continue;
        }
        let device_map = session
            .allocator
            .devices
            .iter()
            .find(|device_map| device_map.device == device)
            .expect("读数里的盘就是分配器的盘");
        let unit_area_end = device_map.unit_area_start().slot().0 + device_map.unit_area_slots();
        let first_isolated_slot = SlotNumber(unit_area_end - slots_to_isolate);
        session
            .allocator
            .isolate_abandoned(device, first_isolated_slot, slots_to_isolate);
        isolated.push((device, first_isolated_slot, slots_to_isolate));
    }
    for (device, available) in available_bytes_on_each_device_of_the_session(session) {
        assert!(
            available >= lower_bound_in_bytes && available < lower_bound_in_bytes + slot_bytes,
            "{device:?}：隔离之后可用 {available} 落在 [{lower_bound_in_bytes}, +16384) 里"
        );
    }
    isolated
}

/// 解开 [`isolate_the_highest_slots_of_the_session_until_the_available_is_within_one_slot_above`] 或
/// [`isolate_every_free_slot_outside_the_region_of_the_crash_flows`] 隔离的那几截。
pub fn clear_the_isolation_of_the_session(
    session: &mut MountedSession,
    isolated: &[(DeviceIdentity, SlotNumber, u64)],
) {
    for (device, first_isolated_slot, slots) in isolated {
        let device_map = session
            .allocator
            .devices
            .iter_mut()
            .find(|device_map| device_map.device == *device)
            .expect("隔离时的盘就是分配器的盘");
        for slot in first_isolated_slot.0..first_isolated_slot.0 + slots {
            device_map.clear_isolation_of_slot(SlotNumber(slot));
        }
    }
}

/// 一次整理发布要新写的槽数（这次重写的角色的跨度之和，即准入的需求）与它搬到目的地的单元各占的槽数之和（批次最坏占用）。
fn demand_and_occupancy_in_slots_of_a_compaction_publish(
    version: &TransactionOutput,
) -> (u64, u64) {
    let demand_in_slots: u64 = version.rewritten.iter().map(|role| role.span_slots()).sum();
    let occupancy_in_slots: u64 = version
        .units_relocated_by_this_publish
        .iter()
        .map(|relocated| {
            let destination = relocated.destination[0];
            let record = version
                .allocation_records
                .iter()
                .find(|record| {
                    record.device == destination.device && record.slot == destination.slot
                })
                .expect("目的地记了一条分配记录");
            u64::from(record.span_slots)
        })
        .sum();
    (demand_in_slots, occupancy_in_slots)
}

/// 重挂之后的整理怎么走（[`record_the_compaction_crash_flow`]）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompactionAfterTheMount {
    /// 一路推到最后一批，第一批发成之后照 `WriteWhileTheIntentIsLive` 做不做一次写类用户改动。
    WritingOnceTheFirstBatchIsPublished(WriteWhileTheIntentIsLive),
    /// 最后一批被准入拒、暂停删意图，再从头签发推到最后一批。
    PausingShortOfSpaceBeforeTheFinalBatch,
    /// 下一批注入一次算着出错、放弃并删意图、整理停下，切换解除停下之后从头签发推到最后一批。
    StoppingAfterAComputeErrorBeforeTheFinalBatch,
    /// R 之外的空槽全隔离，最后一批取不到落点、暂停删意图（删意图那一次的新落点落在 R 里），解开隔离，到此为止。
    PausingWithOnlyTheRegionFreeBeforeTheFinalBatch,
}

/// 出错停下删意图那条流在重挂之后的整理：签发、一批（各记成一步）；下一趟经只供测试的钩子在那一批读完设置之后注入一次算着出错
/// （`BackgroundComputeOfThisMount::report_a_compute_error_in_the_batch_under_way_for_tests`）：那一批放弃、单发一次只删意图的发布
/// （记成 [`CompactionFlowStepKind::CompactionIntentDroppedOnAStop`]）、整理停下；选 `cpu` 的切换（不写盘）解除停下，
/// 再整理到最后一批同一次发布删了重新签发的那条意图。到删意图那一步为止，与暂停删意图那条流的录制流一样（被拒的那一批、注入出错的那一批都一个写都不发）。
///
/// # Panics
/// 签发、一批不是这几步，出错那一趟没停在放弃那一批、没恰好发一次只删意图的发布，切换被拒，或重新签发的那条意图删不了。
fn publish_the_compaction_stopped_after_a_compute_error_before_the_final_batch(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
) -> Vec<CompactionFlowStep> {
    let mut steps: Vec<CompactionFlowStep> = Vec::new();
    for expected_kind in [
        CompactionFlowStepKind::CompactionIssue,
        CompactionFlowStepKind::CompactionBatch,
    ] {
        let step = publish_the_compaction_once_as_a_step(pool, session, &steps);
        assert_eq!(step.kind, expected_kind, "出错之前的整理步：签发、一批");
        steps.push(step);
    }
    session.compaction.publishes_since_the_last_user_change =
        PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
    let start = pool.stream.operation_count();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let run = session
        .run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch(
            devices,
            &mut |background_compute| {
                background_compute.report_a_compute_error_in_the_batch_under_way_for_tests();
            },
        )
        .expect("整理一趟");
    let end = pool.stream.operation_count();
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(
                CompactionRunStop::BatchAbandonedAfterABackgroundComputeError {
                    task_after_the_error: BackgroundTaskAfterTheComputeError::LeftStopped
                }
            )
        ),
        "注入的出错：那一批放弃、整理停下，实际 {:?}",
        run.outcome
    );
    let [dropped] = <[TransactionOutput; 1]>::try_from(run.versions_published)
        .expect("出错停下那一趟恰好发一次只删意图的发布");
    assert!(
        dropped.logged_operations.is_empty()
            && dropped.units_relocated_by_this_publish.is_empty()
            && region_holds_an_unreleased_record(&dropped),
        "出错停下那一次只删意图、不搬单元，R 里还有未释放的记录"
    );
    steps.push(CompactionFlowStep {
        kind: CompactionFlowStepKind::CompactionIntentDroppedOnAStop,
        operations: start..end,
        versions: vec![dropped],
        discards: run.discards.applied.clone(),
    });
    session
        .switch_background_compute_from_the_next_batch(BackgroundComputeSettings::PRODUCT, None)
        .expect("选 cpu：切换收下，解除停下");
    steps.extend(publish_the_compaction_until_the_intent_is_deleted(
        pool, session,
    ));
    steps
}

/// 暂停删意图那条流在重挂之后的整理：签发、一批（各记成一步）；之后把每块盘的可用(d) 压到最后一批的「需求 + 批次最坏占用」之下一个槽以内
/// （两样在同一前缀的整理搬迁流上量出来：`demand_and_occupancy_of_the_final_batch`），最后一批被准入拒、暂停，那一趟单发一次只删意图的发布
/// （记成 [`CompactionFlowStepKind::CompactionIntentDroppedOnAStop`]）；解开压余量的隔离，再整理到最后一批同一次发布删了重新签发的那条意图。
///
/// # Panics
/// 签发、一批不是这几步，暂停那一趟没停在暂停、没恰好发一次只删意图的发布，或重新签发的那条意图删不了。
fn publish_the_compaction_pausing_short_of_space_before_the_final_batch(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    demand_and_occupancy_of_the_final_batch: (u64, u64),
) -> Vec<CompactionFlowStep> {
    let mut steps: Vec<CompactionFlowStep> = Vec::new();
    for expected_kind in [
        CompactionFlowStepKind::CompactionIssue,
        CompactionFlowStepKind::CompactionBatch,
    ] {
        let step = publish_the_compaction_once_as_a_step(pool, session, &steps);
        assert_eq!(step.kind, expected_kind, "暂停之前的整理步：签发、一批");
        steps.push(step);
    }
    let (demand_in_slots, occupancy_in_slots) = demand_and_occupancy_of_the_final_batch;
    let slot_bytes = i128::from(SLOT_BYTES);
    let isolated =
        isolate_the_highest_slots_of_the_session_until_the_available_is_within_one_slot_above(
            session,
            i128::from(demand_in_slots + occupancy_in_slots) * slot_bytes - slot_bytes,
        );
    session.compaction.publishes_since_the_last_user_change =
        PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
    let start = pool.stream.operation_count();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let run = session.run_compaction(devices).expect("整理一趟");
    let end = pool.stream.operation_count();
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PausedShortOfSpace)
        ),
        "最后一批被准入拒：暂停，实际 {:?}",
        run.outcome
    );
    let [dropped] = <[TransactionOutput; 1]>::try_from(run.versions_published)
        .expect("暂停那一趟恰好发一次只删意图的发布");
    assert!(
        dropped.logged_operations.is_empty()
            && dropped.units_relocated_by_this_publish.is_empty()
            && region_holds_an_unreleased_record(&dropped),
        "暂停那一次只删意图、不搬单元，R 里还有未释放的记录"
    );
    steps.push(CompactionFlowStep {
        kind: CompactionFlowStepKind::CompactionIntentDroppedOnAStop,
        operations: start..end,
        versions: vec![dropped],
        discards: run.discards.applied.clone(),
    });
    clear_the_isolation_of_the_session(session, &isolated);
    steps.extend(publish_the_compaction_until_the_intent_is_deleted(
        pool, session,
    ));
    steps
}

/// 每块盘 R 那一段（[`region_of_the_crash_flows`] 的槽，两块盘同槽）里此刻的空槽数（`DeviceFreeMap::is_free`：没分配、没隔离、没扣住），
/// 按分配器里盘的次序。
#[must_use]
pub fn free_slots_in_the_region_of_the_crash_flows_on_each_device(
    session: &MountedSession,
) -> Vec<(DeviceIdentity, u64)> {
    let region = region_of_the_crash_flows();
    session
        .allocator
        .devices
        .iter()
        .map(|device_map| {
            let free = (region.start_slot().0..region.end_slot().0)
                .filter(|slot| device_map.is_free(SlotNumber(*slot)))
                .count();
            (device_map.device, u64::try_from(free).expect("一段 64 槽"))
        })
        .collect()
}

/// 把每块盘单元区里 R 那一段之外的每一个空槽隔离（只动分配器的隔离位，不写盘、不进录制流）：之后 R 之外一个空槽都没有，
/// 取落点只剩 R 那一段。交回隔离的那几截（盘、起始槽、槽数），解开时照它清（[`clear_the_isolation_of_the_session`]）。
///
/// # Panics
/// 隔离之后 R 之外还有空槽。
pub fn isolate_every_free_slot_outside_the_region_of_the_crash_flows(
    session: &mut MountedSession,
) -> Vec<(DeviceIdentity, SlotNumber, u64)> {
    let region = region_of_the_crash_flows();
    let region_slots = region.start_slot().0..region.end_slot().0;
    let mut runs_to_isolate: Vec<(DeviceIdentity, SlotNumber, u64)> = Vec::new();
    // 迭代上界是每块盘单元区的槽数；跨轮只往隔离的清单里追加。
    for device_map in &session.allocator.devices {
        let start = device_map.unit_area_start().slot().0;
        let end = start + device_map.unit_area_slots();
        let is_free_outside_the_region =
            |slot: u64| !region_slots.contains(&slot) && device_map.is_free(SlotNumber(slot));
        let mut slot = start;
        while slot < end {
            if !is_free_outside_the_region(slot) {
                slot += 1;
                continue;
            }
            let run_start = slot;
            while slot < end && is_free_outside_the_region(slot) {
                slot += 1;
            }
            runs_to_isolate.push((device_map.device, SlotNumber(run_start), slot - run_start));
        }
    }
    for (device, run_start, slots) in &runs_to_isolate {
        session
            .allocator
            .isolate_abandoned(*device, *run_start, *slots);
    }
    for device_map in &session.allocator.devices {
        let start = device_map.unit_area_start().slot().0;
        let free_outside_the_region = (start..start + device_map.unit_area_slots())
            .filter(|slot| !region_slots.contains(slot) && device_map.is_free(SlotNumber(*slot)))
            .count();
        assert_eq!(
            free_outside_the_region, 0,
            "{:?}：隔离之后 R 之外一个空槽都没有",
            device_map.device
        );
    }
    runs_to_isolate
}

/// 这一版在 R 那一段里有一条分配代是它自己的 txg 的未释放记录：这次发布自己的新落点落在了 R 里。
#[must_use]
pub fn region_holds_a_placement_of_the_publish_of_the_version(version: &TransactionOutput) -> bool {
    let region = region_of_the_crash_flows();
    version.allocation_records.iter().any(|record| {
        !record.is_released
            && record.generation == version.root.checkpoint_txg
            && region.contains_slot(record.slot)
    })
}

/// 只剩 R 有空槽时暂停删意图那条流在重挂之后的整理：签发、一批（各记成一步）；再把 R 之外的每一个空槽隔离
/// （[`isolate_every_free_slot_outside_the_region_of_the_crash_flows`]），最后一批在 R 之外取不到落点、暂停，那一趟单发一次只删意图的发布
/// （记成 [`CompactionFlowStepKind::CompactionIntentDroppedOnAStop`]）：它取落点时放开 R，新落点落在 R 里；解开隔离，流到这里为止。
///
/// # Panics
/// 签发、一批不是这几步；隔离之前 R 里有一块盘一个空槽都没有；暂停那一趟没停在暂停、没恰好发一次只删意图的发布，或那一次的新落点没落在 R 里。
fn publish_the_compaction_pausing_with_only_the_region_free_before_the_final_batch(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
) -> Vec<CompactionFlowStep> {
    let mut steps: Vec<CompactionFlowStep> = Vec::new();
    for expected_kind in [
        CompactionFlowStepKind::CompactionIssue,
        CompactionFlowStepKind::CompactionBatch,
    ] {
        let step = publish_the_compaction_once_as_a_step(pool, session, &steps);
        assert_eq!(step.kind, expected_kind, "暂停之前的整理步：签发、一批");
        steps.push(step);
    }
    let free_in_the_region = free_slots_in_the_region_of_the_crash_flows_on_each_device(session);
    assert!(
        free_in_the_region.iter().all(|(_, free)| *free > 0),
        "前提：R 里每块盘都有空槽，删意图那一次落得下（盘、空槽数）{free_in_the_region:?}"
    );
    let isolated = isolate_every_free_slot_outside_the_region_of_the_crash_flows(session);
    session.compaction.publishes_since_the_last_user_change =
        PUBLISHES_ALREADY_PASSED_LEAVING_A_BUDGET_OF_ONE;
    let start = pool.stream.operation_count();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let run = session.run_compaction(devices).expect("整理一趟");
    let end = pool.stream.operation_count();
    assert!(
        matches!(
            run.outcome,
            CompactionRunOutcome::Stopped(CompactionRunStop::PausedShortOfSpace)
        ),
        "最后一批在 R 之外取不到落点：暂停，实际 {:?}",
        run.outcome
    );
    let [dropped] = <[TransactionOutput; 1]>::try_from(run.versions_published)
        .expect("暂停那一趟恰好发一次只删意图的发布");
    assert!(
        dropped.logged_operations.is_empty()
            && dropped.units_relocated_by_this_publish.is_empty()
            && region_holds_an_unreleased_record(&dropped)
            && region_holds_a_placement_of_the_publish_of_the_version(&dropped),
        "暂停那一次只删意图、不搬单元，R 里还有未释放的记录，它自己的新落点落在 R 里"
    );
    steps.push(CompactionFlowStep {
        kind: CompactionFlowStepKind::CompactionIntentDroppedOnAStop,
        operations: start..end,
        versions: vec![dropped],
        discards: run.discards.applied.clone(),
    });
    clear_the_isolation_of_the_session(session, &isolated);
    steps
}

/// 活意图期间一次覆盖写把 R 放空的会话（「R 已按记录全空，只删意图」那一形：下一次整理发布不带一批、只删意图）：
/// 整理搬迁流的前缀（A → B、C → 重挂），签发、一批（R 里搬走两个），意图活着时覆盖写第一个文件，重写的单元放掉 R 里剩下的那几个。
/// 交回池（录制流照录）与会话。用例在这之后隔离 R 之外的空槽，再推那一次只删意图的发布。
///
/// # Panics
/// 签发、一批不是这两步，或覆盖写之后 R 不按记录全空、意图不活着。
#[must_use]
pub fn session_with_the_region_emptied_by_an_overwrite_while_the_intent_is_live(
    tag: &str,
    contents_of_the_sequential_writes: &[Vec<u8>],
) -> (BuiltPool, MountedSession) {
    let mut pool = build_pool(tag);
    for content in contents_of_the_sequential_writes {
        record_one_publish_of_instance_one(
            &mut pool,
            CompactionFlowStepKind::SequentialWrite,
            |built_pool| {
                sequential_write_in_process_as_instance_one_and_advance_the_pool(
                    built_pool, content,
                )
            },
        );
    }
    let (_mount_step, mut session) = remount_and_open_the_session(&mut pool);
    let mut steps: Vec<CompactionFlowStep> = Vec::new();
    for expected_kind in [
        CompactionFlowStepKind::CompactionIssue,
        CompactionFlowStepKind::CompactionBatch,
    ] {
        let step = publish_the_compaction_once_as_a_step(&mut pool, &mut session, &steps);
        assert_eq!(step.kind, expected_kind, "覆盖写之前的整理步：签发、一批");
        steps.push(step);
    }
    let content = content_overwritten_while_the_intent_is_live();
    publish_a_user_change_through_the_session(
        &mut pool,
        &mut session,
        CompactionFlowStepKind::OverwriteWhileTheIntentIsLive,
        UserChange::Overwrite(FirstFile {
            content: &content,
            write_time_seconds: FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
        }),
    );
    let after_the_overwrite = version_of_trees(&session.current);
    assert!(
        !after_the_overwrite.logged_operations.is_empty()
            && !region_holds_an_unreleased_record(after_the_overwrite)
            && session.allocator.region_excluded_from_placement() == Some(region_of_the_crash_flows()),
        "前提：覆盖写之后意图还活着、R 排他着，R 按记录全空：R 上的记录（盘、槽、跨度、代、释放没有）{:?}",
        allocation_records_over_the_region(after_the_overwrite)
    );
    (pool, session)
}

/// 实例 1 里建空 inode，连同第一个文件恰好填满 [`INODE_LEAF_CONTAINERS_LEFT_IN_THE_REGION_BEHIND_THE_WRITE`] 个叶容器：
/// 这次发布的提交内生块接着 C 落在 R 里（重挂之后 R 不是本次挂载开过的段）。
fn publish_the_inode_leaf_containers_left_in_the_region(pool: &mut BuiltPool) -> TransactionOutput {
    let previous = pool.output.clone();
    let publish_parameters = parameters();
    let devices = pool.devices.as_mut().expect("镜像还开着");
    let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
    publish_new_inodes(
        &mut writer,
        &mut pool.allocator,
        &previous,
        INODE_LEAF_CONTAINERS_LEFT_IN_THE_REGION_BEHIND_THE_WRITE * INODE_LEAF_RECORDS - 1,
        FIXED_WRITE_TIME_SECONDS + SECONDS_FROM_THE_FIRST_WRITE_TO_THE_INODES_LEFT_IN_THE_REGION,
        INSTANCE_OF_THE_FLOWS,
    )
    .expect("重挂之前建空 inode：盘空得很、叶容器一个根装得下")
}

/// 录一条流（`tag` 是镜像文件名里那一段）。`contents_of_the_sequential_writes` 只给整理搬迁流与活意图期间写类改动流用（B、C 的内容，多记录流那两份），别的流给空。
///
/// # Panics
/// 任何一步发布被拒、整理停不下来、回收或再分配在上限之内做不成：这几条流的每一步按条款都做得成。
#[must_use]
pub fn record_the_compaction_crash_flow(
    tag: &str,
    flow: CompactionCrashFlow,
    contents_of_the_sequential_writes: &[Vec<u8>],
) -> RecordedCompactionFlow {
    let mut pool = build_pool(tag);
    let first_file = pool.output.clone();
    let mut steps = Vec::new();
    let mut snapshot = None;
    match flow {
        CompactionCrashFlow::CompactionRelocation
        | CompactionCrashFlow::CompactionWithAWriteWhileTheIntentIsLive
        | CompactionCrashFlow::CompactionPausedShortOfSpaceDroppingTheIntent
        | CompactionCrashFlow::CompactionStoppedAfterAComputeErrorDroppingTheIntent
        | CompactionCrashFlow::CompactionPausedWithOnlyTheRegionFreeDroppingTheIntentInTheRegion => {
            assert_eq!(
                contents_of_the_sequential_writes.len(),
                2,
                "整理搬迁流的前缀是多记录流：B、C 两次顺序写"
            );
            for content in contents_of_the_sequential_writes {
                steps.push(record_one_publish_of_instance_one(
                    &mut pool,
                    CompactionFlowStepKind::SequentialWrite,
                    |pool| {
                        sequential_write_in_process_as_instance_one_and_advance_the_pool(
                            pool, content,
                        )
                    },
                ));
            }
            if flow == CompactionCrashFlow::CompactionWithAWriteWhileTheIntentIsLive {
                steps.push(record_one_publish_of_instance_one(
                    &mut pool,
                    CompactionFlowStepKind::NewInodesThroughTheSession,
                    publish_the_inode_leaf_containers_left_in_the_region,
                ));
            }
        }
        CompactionCrashFlow::CompactionRelocatingABlockSharedWithASnapshot => {
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot),
                create_snapshot,
            ));
            snapshot = live_snapshot_trees_of(&steps[0].versions[0])
                .last()
                .copied();
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
                overwrite_under_the_snapshot,
            ));
        }
        CompactionCrashFlow::LifecycleClosure => {
            // 增量矩阵可在闭环前加入多单元顺序写；不加时传空切片，写流不变。
            for content in contents_of_the_sequential_writes {
                steps.push(record_one_publish_of_instance_one(
                    &mut pool,
                    CompactionFlowStepKind::SequentialWrite,
                    |pool| {
                        sequential_write_in_process_as_instance_one_and_advance_the_pool(
                            pool, content,
                        )
                    },
                ));
            }
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::OverwriteBeforeTheSnapshot,
                |pool| {
                    let previous = pool.output.clone();
                    publish_overwrite_in_process(
                        pool,
                        &previous,
                        &content_overwritten_before_the_snapshot(),
                        FIXED_WRITE_TIME_SECONDS + OVERWRITE_SECONDS_AFTER_THE_FIRST_WRITE,
                        INSTANCE_OF_THE_FLOWS,
                    )
                    .expect("建快照之前覆盖写")
                },
            ));
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::CreateSnapshot),
                create_snapshot,
            ));
            snapshot = live_snapshot_trees_of(&steps.last().expect("刚发布快照").versions[0])
                .last()
                .copied();
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::OverwriteUnderTheSnapshot),
                overwrite_under_the_snapshot,
            ));
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkIntent),
                unlink_intent,
            ));
            // 一个数据单元的文件、批大小 1：意图之后只有最后一批（`common_snapshot_prefix::SnapshotFlowStepKind::UnlinkFinalBatch`）
            steps.push(record_one_publish_of_instance_one(
                &mut pool,
                CompactionFlowStepKind::Snapshot(SnapshotFlowStepKind::UnlinkFinalBatch),
                unlink_step,
            ));
            assert!(
                pool.output.deleted_inodes.is_empty(),
                "一个数据单元的文件：意图之后一批就删完"
            );
        }
    }
    let (mount_step, mut session) = remount_and_open_the_session(&mut pool);
    steps.push(mount_step);
    let compaction_after_the_mount = match flow {
        CompactionCrashFlow::CompactionWithAWriteWhileTheIntentIsLive => {
            CompactionAfterTheMount::WritingOnceTheFirstBatchIsPublished(
                WriteWhileTheIntentIsLive::OverwriteTheFirstFileAfterTheFirstBatch,
            )
        }
        CompactionCrashFlow::CompactionRelocation
        | CompactionCrashFlow::CompactionRelocatingABlockSharedWithASnapshot
        | CompactionCrashFlow::LifecycleClosure => {
            CompactionAfterTheMount::WritingOnceTheFirstBatchIsPublished(
                WriteWhileTheIntentIsLive::NoWrite,
            )
        }
        CompactionCrashFlow::CompactionPausedShortOfSpaceDroppingTheIntent => {
            CompactionAfterTheMount::PausingShortOfSpaceBeforeTheFinalBatch
        }
        CompactionCrashFlow::CompactionStoppedAfterAComputeErrorDroppingTheIntent => {
            CompactionAfterTheMount::StoppingAfterAComputeErrorBeforeTheFinalBatch
        }
        CompactionCrashFlow::CompactionPausedWithOnlyTheRegionFreeDroppingTheIntentInTheRegion => {
            CompactionAfterTheMount::PausingWithOnlyTheRegionFreeBeforeTheFinalBatch
        }
    };
    match compaction_after_the_mount {
        CompactionAfterTheMount::WritingOnceTheFirstBatchIsPublished(write) => {
            steps.extend(publish_the_compaction_writing_after_the_first_batch(
                &mut pool,
                &mut session,
                write,
            ));
        }
        CompactionAfterTheMount::StoppingAfterAComputeErrorBeforeTheFinalBatch => {
            steps.extend(
                publish_the_compaction_stopped_after_a_compute_error_before_the_final_batch(
                    &mut pool,
                    &mut session,
                ),
            );
        }
        CompactionAfterTheMount::PausingWithOnlyTheRegionFreeBeforeTheFinalBatch => {
            steps.extend(
                publish_the_compaction_pausing_with_only_the_region_free_before_the_final_batch(
                    &mut pool,
                    &mut session,
                ),
            );
        }
        CompactionAfterTheMount::PausingShortOfSpaceBeforeTheFinalBatch => {
            // 最后一批的需求与批次最坏占用在同一前缀的整理搬迁流上量（两条流到最后一批之前走同一串操作，字节相同）。
            let measured = record_the_compaction_crash_flow(
                &format!("{tag}-measuring"),
                CompactionCrashFlow::CompactionRelocation,
                contents_of_the_sequential_writes,
            );
            let final_batches = measured.steps_of(CompactionFlowStepKind::CompactionFinalBatch);
            let [final_batch] = final_batches.as_slice() else {
                panic!("整理搬迁流恰好一次最后一批：{} 次", final_batches.len());
            };
            let demand_and_occupancy = demand_and_occupancy_in_slots_of_a_compaction_publish(
                final_batch.versions.last().expect("最后一批发成了一版"),
            );
            steps.extend(
                publish_the_compaction_pausing_short_of_space_before_the_final_batch(
                    &mut pool,
                    &mut session,
                    demand_and_occupancy,
                ),
            );
        }
    }
    if flow == CompactionCrashFlow::LifecycleClosure {
        let snapshot_to_destroy = snapshot.expect("生命周期闭环流建过快照");
        steps.push(publish_a_user_change_through_the_session(
            &mut pool,
            &mut session,
            CompactionFlowStepKind::DestroySnapshotThroughTheSession,
            UserChange::DestroySnapshot {
                snapshot: snapshot_to_destroy,
            },
        ));
        record_the_reclaim_and_the_reallocation(&mut pool, &mut session, &mut steps);
    }
    pool.output = version_of_trees(&session.current).clone();
    RecordedCompactionFlow {
        flow,
        pool,
        first_file,
        steps,
        snapshot,
    }
}

/// 生命周期闭环流的「回收」与「在搬出来的空段上再分配」：一轮抬 F 到准入上限、建一次 inode（建 inode 发成之后会话发 discard），
/// 直到会话对 R 整段发了 discard；再一次一次建 inode，直到 R 里有一条未释放的记录（分配器在最低的全空段开新段，D3（空间分配） 已定项 10 ①）。
/// 步数按实现的取落点走，用例只核「发了 discard、之后分配到 R 上」这两件事，不核轮数。
fn record_the_reclaim_and_the_reallocation(
    pool: &mut BuiltPool,
    session: &mut MountedSession,
    steps: &mut Vec<CompactionFlowStep>,
) {
    let mut new_inodes_published = 0_u64;
    let mut new_inodes_step =
        |pool_of_the_flow: &mut BuiltPool, session_of_the_flow: &mut MountedSession| {
            new_inodes_published += 1;
            publish_a_user_change_through_the_session(
                pool_of_the_flow,
                session_of_the_flow,
                CompactionFlowStepKind::NewInodesThroughTheSession,
                UserChange::NewInodes {
                    count: 1,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS
                        + LIFECYCLE_SECONDS_FROM_THE_FIRST_WRITE_TO_THE_NEW_INODES
                        + new_inodes_published,
                },
            )
        };
    let mut is_the_region_discarded = false;
    for _round_index in 0..LIFECYCLE_RECLAIM_ROUNDS_AT_MOST {
        if let Some(raised) = raise_the_floor_to_the_admission_ceiling(pool, session) {
            steps.push(raised);
        }
        let step = new_inodes_step(pool, session);
        is_the_region_discarded = the_step_discards_the_region_on_both_devices(&step);
        steps.push(step);
        if is_the_region_discarded {
            break;
        }
    }
    assert!(
        is_the_region_discarded,
        "{LIFECYCLE_RECLAIM_ROUNDS_AT_MOST} 轮抬 F 之内 R 整段回收、会话对它发了 discard"
    );
    for _publish_index in 0..LIFECYCLE_REALLOCATION_PUBLISHES_AT_MOST {
        let step = new_inodes_step(pool, session);
        let is_reallocated = step
            .versions
            .last()
            .is_some_and(region_holds_an_unreleased_record);
        steps.push(step);
        if is_reallocated {
            return;
        }
    }
    panic!("建 {LIFECYCLE_REALLOCATION_PUBLISHES_AT_MOST} 次 inode 之内分配到 R 上");
}

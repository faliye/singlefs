//! 取号之前的预演：取号之后那一串发布在分配器拷贝上整串预演，挂载续做在取号之前预演，
//! 续做里开容器越过编号上限的那一步在取号之前拒。

use super::*;

/// 取号之后那一串里一次发布取到的落点：这次重写的角色，按取落点的次序（bump 次序）各一个槽。零单元的发布一个都不取。
pub(super) type PlacementsTakenByOnePublish = Vec<(TransactionUnit, SlotNumber)>;

/// 取号之前把这次挂载取号之后要发的那一串——写行一次、暖机 `warm_up_publish_txgs` 那几次——在分配器的一份拷贝上整串预演一遍，
/// 交回每一次取到的落点、预演到最后的那一版与那一份分配器拷贝（[`PublishesRehearsedBeforeAcquisition`]：取号之前预演续做接着它们走）。
/// 只动拷贝、不写盘（读盘读的是这次挂载共用的那一份，`mounted_session::DeviceReadOnceInThisMount`）；**走的就是发布路径落盘之前那一段**（带文件的一版 `transaction::prepare_the_version_publish`，
/// 树表 0 条的一版上写行 `transaction::prepare_the_row_publish_on_a_version_without_file`，零单元发布
/// `transaction::prepare_the_publish_without_units`），计划也是取号之后那几次发布用的同一张（`row_publish_plan_on_file_version`、
/// `empty_publish_plan_after_file_version`、`empty_publish_plan_after_version_without_file`），不另推一份：
/// 这一串每一次重写哪些角色——分配记录树重写哪几个节点要在分配器上走到固定点才知道（D8（核心索引结构） 已定项 14）——
/// 与真发时是同一段代码算的。写行那次的释放核验、树的形状、落点取不到，都在这里先报出来，在取号之前拒（D18（块里携带什么信息） 已定项 11
/// 「可写挂载的顺序」第五个合取；增补 2 第 20a 行）。
///
/// 发布路径在取落点之前还读盘核换下的每一份的校验和，对不上（读不出也算）的那一份在它那块盘上的分配记录留在已分配、不释放
/// （D19（块指针的结构与宽度预算） 已定项 5），这里不读盘（`ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition`）、照常释放。
/// 两边只在那一槽这一串里被回收时分叉：拷贝上它回收了、可能再发出去，真发时它一直占着；这一串里回收它，得这一串自己的根把根环里
/// 比它旧的有效根全盖掉（崩溃恢复落到环里最旧的那条根、其余全被抛弃时走得到）。所以这一串里没有一份核出对不上时，这里取到的与真发起来取到的逐项相同
/// （`establish_instance` 在这一串发完之后断言）；有一份核出对不上时可能不同，那一格见 `establish_instance` 的注释。
///
/// # Errors
/// 第几次（从 0 数）的准备报了什么错（[`DryRunRefusal`]）。
pub(super) fn dry_run_of_the_publishes_after_acquisition<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    allocator: &PoolAllocator,
    start: &InstanceStart,
    instance_table_rewrite: &InstanceTableRewrite,
    rows_to_write: usize,
    warm_up_publish_txgs: &[CheckpointTxg],
    instance_to_acquire: InstanceGeneration,
) -> Result<PublishesRehearsedBeforeAcquisition, DryRunRefusal> {
    let mut copy = allocator.clone();
    let refused_at = |publish_index: usize| {
        move |cause: PublishError| DryRunRefusal {
            publish_index,
            cause,
        }
    };
    let row_publish = match &start.previous {
        PreviousVersion::WithFile { output, .. }
        | PreviousVersion::WithTreesAndNoFile { output, .. } => {
            let plan = row_publish_plan_on_file_version(
                output,
                instance_table_rewrite.clone(),
                &RowPublishIdentity {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance: instance_to_acquire,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            );
            prepare_the_version_publish(
                pool,
                &mut copy,
                &plan,
                Some(output),
                output.tree_identifiers,
                ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                UnmountMarker::NotWrittenByTheUnmountSequence,
                &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
            )
            .map(|(_, rehearsed)| PoolVersion::of_a_version_of_trees(rehearsed))
            .map_err(refused_at(0))?
        }
        PreviousVersion::WithoutFile { root, .. } if rows_to_write == 0 => {
            let (_, rehearsed) = prepare_the_publish_without_units(
                pool,
                root,
                ZeroUnitPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance: instance_to_acquire,
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            );
            copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);
            PoolVersion::WithoutFile(rehearsed)
        }
        PreviousVersion::WithoutFile { root, .. } => {
            prepare_the_row_publish_on_a_version_without_file(
                pool,
                &mut copy,
                root,
                InstanceTableOnlyPublishPlan {
                    txg: start.first_txg,
                    counter: start.next_counter,
                    instance: instance_to_acquire,
                    back_chain: 0,
                    rollback_floor: start.effective_floor,
                    instance_table: instance_table_rewrite,
                    tree_identifier_watermark: start.tree_identifier_watermark_of_the_ring,
                },
            )
            .map(|(_, rehearsed)| PoolVersion::WithoutFile(rehearsed))
            .map_err(refused_at(0))?
        }
    };
    let mut taken_by_every_publish = vec![placements_taken_by(&row_publish)];
    let mut current = row_publish;
    // 迭代上界是暖机的次数；跨轮携带的是预演出来的现行那一版（下一次的计划接着它）。
    for (warm_up_position, planned_txg) in warm_up_publish_txgs.iter().enumerate() {
        let publish_index = warm_up_position + 1;
        let next = match &current {
            PoolVersion::WithFile(current_file_version)
            | PoolVersion::WithTreesAndNoFile(current_file_version) => {
                let plan = empty_publish_plan_after_file_version(
                    current_file_version,
                    instance_to_acquire,
                );
                prepare_the_version_publish(
                    pool,
                    &mut copy,
                    &plan,
                    Some(Arc::as_ref(current_file_version)),
                    current_file_version.tree_identifiers,
                    ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,
                    UnmountMarker::NotWrittenByTheUnmountSequence,
                    &DeletionAndLineageOfThePublish::NEITHER_A_DELETION_NOR_A_ROLLBACK,
                )
                .map(|(_, rehearsed)| PoolVersion::of_a_version_of_trees(rehearsed))
                .map_err(refused_at(publish_index))?
            }
            PoolVersion::WithoutFile(current_version_without_file) => {
                let (_, rehearsed) = prepare_the_publish_without_units(
                    pool,
                    &current_version_without_file.root,
                    empty_publish_plan_after_version_without_file(
                        current_version_without_file,
                        instance_to_acquire,
                    ),
                );
                copy.record_root_written_by_this_process(rehearsed.root.checkpoint_txg);
                PoolVersion::WithoutFile(rehearsed)
            }
        };
        assert_eq!(
            next.root().checkpoint_txg,
            *planned_txg,
            "预演的暖机 txg 与计划里的那一个相同：计划逐个加一，计划接着现行那一版的 txg 加一"
        );
        taken_by_every_publish.push(placements_taken_by(&next));
        current = next;
    }
    Ok(PublishesRehearsedBeforeAcquisition {
        placements_taken_by_every_publish: taken_by_every_publish,
        version_after_the_last_publish: current,
        allocator_after_the_last_publish: copy,
    })
}

/// 取号之前预演续做至多走几趟（岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路二「没走完就重走」那一行，用户定：预演因读错没走完时
/// 从头再走一次，次数有上限，再没走完照「维持偏拒」）：头一趟加重走一趟。取值与 D19（块指针的结构与宽度预算） 已定项 5「只重读一次，
/// 不退避、不多次重试」同一个口径——一次瞬时读错、读坏给一次重来，两趟都被打断才拒。
const CONTINUATION_REHEARSALS_BEFORE_ACQUISITION_AT_MOST: usize = 2;

/// 挂载续做（[`continue_the_deletion_queue_until_it_is_empty`]）在取号之前整个预演一遍（岔路单 `research/prompts/m3-b44-code-r1-forks.md`
/// 那一行；代码轮 `research/prompts/m3-b44-code-r1-main-verification.md` 第二节 K6）：一趟怎么走见
/// [`rehearse_the_continuation_before_acquisition_once`]。
/// 有一步报 C841（容器树要开新容器而最右那片的号已到顶）就在取号之前拒：不拒的话这一错在续做里才交回，那时写行那次发布与续做里
/// 排在它前面的各步（连同推的抬 F）已落盘，之后每次可写挂载再取号、写行、再拒（攻方 `research/prompts/m3-b44-code-r1-opus-output.md` 第五节）。
/// 预演读的都是这次挂载共用的那一份（`mounted_session::DeviceReadOnceInThisMount`，真做之后读的也是它）。预演没走到续做的结局——
/// 续做里一步报了 C841 之外的错，或只进内存那一趟在续做之前（取号、写行、暖机、写行之后推的抬 F）报了错、续做里推的抬 F 自己报了错——
/// 而这一趟读盘时有过读错（盘报了错，`reads_failed_on_the_disk`）或读坏（读成了、读的一方核出自证不过，`reads_failing_self_verification`）：
/// 从头重走（岔路单 `research/prompts/m3-b44-code-r3-forks.md` 岔路二「没走完就重走」那一行），读错的块本来没记下、读坏的块读坏那一刻已从
/// 共用的那一份里丢掉，重走读到它们穿到盘上；走到 [`CONTINUATION_REHEARSALS_BEFORE_ACQUISITION_AT_MOST`] 趟仍是这样，在取号之前拒，
/// 交最后那一趟最先的那一次读错，没有读错交最先的那一次读坏（岔路一「两样都做」那一行：读错账与读坏账两本都判）。
/// 两本账里落在逐盘核那一刻已读坏、已报读错的系统配置槽上的那几条都不算进这一判（改法丙，代码轮
/// `research/prompts/m3-close-b19-code-r3-main-verification.md` 第二节 V1、V2 与第三节，用户 2026-10-29 选；
/// [`read_of_a_system_configuration_slot_already_bad_when_checked`]）：只进内存那一趟取号读每块盘的两槽，这几槽是重走治不好的旧故障，
/// 每一趟都读到、都记进账，算进去会把别处的读坏、读错或别的停法的拒因盖成这几槽——补写盘的两槽（逐盘核判过两槽全废、取号会补写）是其中一形，
/// 补写盘之外的成员盘一槽撕裂、一槽每次读报错（另一槽自证过，逐盘核放行）是另一形。
/// 逐盘核那一刻读得出、没报自证不过的槽之后在预演里读坏、读错的，照旧进账、照旧判（新故障）：读错的槽里装的是什么看不见，
/// 可能是一份更新的见证，不当没发生。
/// 两本账都空的照旧留给续做、真做时报它自己的（岔路单那一行只定了 C841，[`rehearsed_continuation_refusal_is_returned_before_acquisition`]）：
/// 只进内存那一趟与真做读同一份字节、走同一段，它报的错真做走到那一步照样报——当场重读那几读（预演与真做各自再读一次盘）
/// 与真做取号写之前那几次读、取号写与轮换写算世代号那一读（真做读盘、预演读共用的那一份）除外，它们这一刻读到的可以与预演不同；
/// 分配器拷贝上那一趟与真做差扣住位与校验和核验两样，它报的错真做不一定报。
/// 分配器拷贝上那一趟报了别的错（不是空间不够）的那一步之后接不上一版，不再预演。
///
/// # Errors
/// C841 ⇒ `MountError::Publish`：`cause` 是那个错；重走到上限仍没走到续做的结局而最后那一趟读盘时有过读错 ⇒ `MountError::Publish`：`cause` 是
/// `PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition`（最先的那一次读错）；只有读坏 ⇒ `cause` 是
/// `PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition`（最先的那一次读坏）。几种都是三份写账全空
/// （取号之前，一个写都没发到盘上）。
#[allow(
    clippy::too_many_arguments,
    reason = "取号之前预演续做要的：写入口、预演到最后的那一版与分配器拷贝、真分配器、起点、要写的行、取号之后的计划、读错账与读坏账、逐盘核读每个系统配置槽的结论，各自独立"
)]
pub(super) fn refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range<
    Device: BlockDevice,
>(
    pool: &PoolWriter<'_, Device>,
    version_after_the_publishes_rehearsed: &PoolVersion,
    allocator_after_the_publishes_rehearsed: PoolAllocator,
    allocator: &PoolAllocator,
    start: &InstanceStart,
    rows_written: &[InstanceRow],
    publishes_planned: &PublishesPlannedAfterAcquisition,
    reads_failed_on_the_disk: &RefCell<Vec<ReadFailedOnTheDisk>>,
    reads_failing_self_verification: &RefCell<Vec<ReadFailingSelfVerification>>,
    system_configuration_slots_as_checked: &[SystemConfigurationSlotAsChecked],
) -> Result<(), MountError> {
    let version_of_trees = match version_after_the_publishes_rehearsed {
        PoolVersion::WithoutFile(_) => return Ok(()),
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
    };
    // 迭代上界是 CONTINUATION_REHEARSALS_BEFORE_ACQUISITION_AT_MOST 趟；跨轮带着已走了几趟（两本账每一趟开头清空：只看这一趟记下的，
    // 之前读的——所选那一版的单元清单、写行与暖机那一串的预演、上一趟——有读错读坏也已各自判过）；
    // 出口：走到结局或两本账都空（照旧留给续做）、C841（拒）、到上限仍没走完而账不空（拒）。
    let mut rehearsals_walked: usize = 0;
    let refusal = loop {
        rehearsals_walked += 1;
        reads_failed_on_the_disk.borrow_mut().clear();
        reads_failing_self_verification.borrow_mut().clear();
        let ending = rehearse_the_continuation_before_acquisition_once(
            pool,
            version_of_trees,
            allocator_after_the_publishes_rehearsed.clone(),
            allocator,
            start,
            rows_written,
            publishes_planned,
        );
        match ending {
            ContinuationRehearsedBeforeAcquisition::WalkedToItsEnd => return Ok(()),
            ContinuationRehearsedBeforeAcquisition::StepRefused(refusal)
                if rehearsed_continuation_refusal_is_returned_before_acquisition(&refusal) =>
            {
                break refusal;
            }
            ContinuationRehearsedBeforeAcquisition::StepRefused(_)
            | ContinuationRehearsedBeforeAcquisition::StoppedBeforeTheEndOfTheContinuation => {}
        }
        // 两本账里落在逐盘核那一刻已读坏、已报读错的系统配置槽上的那几条不拿来判重走与拒（改法丙，代码轮
        // `research/prompts/m3-close-b19-code-r3-main-verification.md` 第三节，用户 2026-10-29 选）：只进内存那一趟取号读每块盘的两槽，
        // 这几槽重走一趟照样坏，拿它们判会让两本账在这几条之外本来是空的那一形重走、拒，或让拒因点名这几槽、盖住真出故障的那一读。
        // 逐盘核之后才坏的槽照旧拿来判：读错的槽里装的是什么看不见，可能是一份更新的见证，不当没发生。
        let first_read_failed_on_the_disk =
            reads_failed_on_the_disk
                .borrow_mut()
                .drain(..)
                .find(|read_failed_on_the_disk| {
                    !read_of_a_system_configuration_slot_already_bad_when_checked(
                        read_failed_on_the_disk.device,
                        read_failed_on_the_disk.offset,
                        read_failed_on_the_disk.length_in_bytes,
                        system_configuration_slots_as_checked,
                    )
                });
        let first_read_failing_self_verification = reads_failing_self_verification
            .borrow_mut()
            .drain(..)
            .find(|read_failing_self_verification| {
                !read_of_a_system_configuration_slot_already_bad_when_checked(
                    read_failing_self_verification.device,
                    read_failing_self_verification.offset,
                    read_failing_self_verification.length_in_bytes,
                    system_configuration_slots_as_checked,
                )
            });
        let is_the_last_rehearsal =
            rehearsals_walked >= CONTINUATION_REHEARSALS_BEFORE_ACQUISITION_AT_MOST;
        match (
            first_read_failed_on_the_disk,
            first_read_failing_self_verification,
            is_the_last_rehearsal,
        ) {
            // 逐盘核那一刻已坏的系统配置槽之外，这一趟读盘没出过读错、也没读坏：照旧留给续做（岔路单那一行只定了 C841）。
            (None, None, _) => return Ok(()),
            (Some(read_failed_on_the_disk), _, true) => {
                break PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(
                    Box::new(read_failed_on_the_disk),
                );
            }
            (None, Some(read_failing_self_verification), true) => {
                break PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(
                    read_failing_self_verification,
                );
            }
            // 没到上限：重走一趟。
            (Some(_), _, false) | (None, Some(_), false) => {}
        }
    };
    Err(MountError::Publish(Box::new(PublishSequenceFailed {
        cause: refusal,
        writes_before_the_first_publish: WritesByStructureKind::NOTHING_WRITTEN,
        writes_of_persisted_publishes: Vec::new(),
        writes_of_failed_publishes: Vec::new(),
    })))
}

/// 从 `device` 的 `offset` 起读 `length_in_bytes` 字节的那一读，是不是落在逐盘核那一刻已读坏或已报读错的系统配置槽上（改法丙）：
/// 逐盘核读过同一个落点（同一块盘、同一个偏移、同一个长度，取号读见证与逐盘核都经 `recovery::system_configuration_slots_of_one_device_read_once`
/// 读整槽），那一刻的结论是读坏或读错。取号之前预演续做判两本账时，读错账与读坏账都剔掉这几条
/// （[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）。
fn read_of_a_system_configuration_slot_already_bad_when_checked(
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length_in_bytes: u64,
    system_configuration_slots_as_checked: &[SystemConfigurationSlotAsChecked],
) -> bool {
    system_configuration_slots_as_checked.iter().any(|slot| {
        slot.device == device
            && slot.offset == offset
            && slot.length_in_bytes == length_in_bytes
            && match slot.conclusion {
                SystemConfigurationSlotConclusionWhenChecked::ReadWithoutFailingSelfVerification => {
                    false
                }
                SystemConfigurationSlotConclusionWhenChecked::ReadFailingSelfVerification
                | SystemConfigurationSlotConclusionWhenChecked::ReportingAReadError => true,
            }
    })
}

/// 取号之前预演续做走一趟（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）：
/// 在取号之前那一串预演到最后的那一版与那一份分配器拷贝上，
/// 照续做取的批大小与容器树容量（分配器上装的删除开关，续做读同一份）把删除队列走到空的每一步都只算不写
/// （`transaction::rehearse_every_step_of_the_deletion_queue_walk`），真分配器不动。
/// 这份拷贝是取号之前那一串（写行、暖机）预演到最后的那一份，重建分配器时扣住的那一截没放开（真做在暖机走完之后放开，
/// [`acquire_then_publish_until_the_continuation`]），不读盘核换下的每一份的校验和（`ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition`）。
/// 扣住位没放开那一差让这一趟的空间判得比真做紧：紧到有一步空间不够时转去下面只进内存那一趟，那一趟走真做那一段、放开扣住位，
/// 所以这一差只多花一趟、不多拒。
/// 那一趟有一步被拒是空间不够的那两种（[`refusal_is_short_of_space`]：续做真发时推一串抬 F 再试的就是这两种）时，抬 F 也预演进去：
/// 在只进内存的盘与 `allocator` 的一份拷贝上把取号之后那一串连同续做照真发走一遍
/// （[`walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory`]），看它走到哪儿。
fn rehearse_the_continuation_before_acquisition_once<Device: BlockDevice>(
    pool: &PoolWriter<'_, Device>,
    version_of_trees: &TransactionOutput,
    mut allocator_after_the_publishes_rehearsed: PoolAllocator,
    allocator: &PoolAllocator,
    start: &InstanceStart,
    rows_written: &[InstanceRow],
    publishes_planned: &PublishesPlannedAfterAcquisition,
) -> ContinuationRehearsedBeforeAcquisition {
    let deletion_switches_the_continuation_reads = allocator.deletion_switches();
    match rehearse_every_step_of_the_deletion_queue_walk(
        pool,
        &mut allocator_after_the_publishes_rehearsed,
        version_of_trees,
        deletion_switches_the_continuation_reads.batch_size,
        deletion_switches_the_continuation_reads.container_tree_capacities(),
        publishes_planned.instance_to_acquire,
    ) {
        Ok(()) => ContinuationRehearsedBeforeAcquisition::WalkedToItsEnd,
        Err(refusal) if refusal_is_short_of_space(&refusal) => {
            walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory(
                pool.parameters,
                &*pool.devices,
                allocator,
                start,
                rows_written,
                publishes_planned,
            )
        }
        Err(refusal) => ContinuationRehearsedBeforeAcquisition::StepRefused(refusal),
    }
}

/// 取号之前预演续做走到哪儿（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）。
pub(super) enum ContinuationRehearsedBeforeAcquisition {
    /// 续做走到空，或推满仍放不下、停在那里（真做时挂载照样做成可写、队列留着）。
    WalkedToItsEnd,
    /// 续做里一步报了错：分配器拷贝上那一趟报的不是空间不够的错，或只进内存那一趟里续做交的 `MountError::Publish` 的 `cause`。
    StepRefused(PublishError),
    /// 只进内存那一趟在续做之前（取号、写行、暖机、写行之后推的抬 F）报了错，或续做里推的抬 F 自己报了错：没走到续做的结局。
    StoppedBeforeTheEndOfTheContinuation,
}

/// 取号之前预演整个续做报的错里，哪几种不看读错账与读坏账就在取号之前拒（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）：
/// 只有 C841 那一种（岔路单那一行只定了这一错）；别的预演那一趟读盘没出过读错、读坏时照旧留给续做，续做里怎么处置见
/// [`continue_the_deletion_queue_until_it_is_empty`]，出过的重走、到上限由两本账那一判拒。
fn rehearsed_continuation_refusal_is_returned_before_acquisition(refusal: &PublishError) -> bool {
    match refusal {
        PublishError::NextContainerNumberPastTheTopOfItsRange(_) => true,
        // 预演读错、读坏那两拒是调用方按两本账自己造的，续做里的一步不报它们；走到这里也是取号之前拒的那一形。
        PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_)
        | PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(_) => true,
        PublishError::SpaceAdmissionRefused(_)
        | PublishError::PlacementRefused { .. }
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
        | PublishError::JournalRecordsOfThePublishExceedTheLimit { .. }
        // txg 到顶不拒：岔路单那一行只定了 C841，别的错不扩大，两趟预演都照这一条。分配器拷贝上那一趟另有一层：它不算写行之后为空间推的抬 F，
        // 续做各步的 txg 在那里算的不是真发的那一个；只进内存那一趟走真做那一段，txg 与真发相同。
        | PublishError::NextCheckpointTxgPastTheTopOfItsRange { .. }
        | PublishError::WriteToAnInodeBeingDeleted { .. }
        | PublishError::WriteToAnInodeThatIsNotInTheTree { .. }
        // 删完第一个文件之后给别的 inode 写数据（K-U5，第一版不支持）：推抬 F 也不会让第一个文件回来。
        | PublishError::DataWriteToAnInodeBornAfterTheFirstFileWasDeletedIsOutsideTheFirstVersion { .. }
        | PublishError::UnlinkOfAnInodeThatIsNotALiveRecord { .. }
        | PublishError::DeletedInodesQueueRefused(_)
        | PublishError::BlockDevice(_) | PublishError::SnapshotCreationRefused(_)
        | PublishError::SnapshotDestructionRefused(_)
        | PublishError::DeadlistEntryNotReleasable { .. }
        | PublishError::EmptyClusterSegmentsBelowTheFloor(_)
        | PublishError::TruncationRefused(_) => false,
    }
}

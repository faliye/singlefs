//! 挂载续做：删除队列走空（[`continue_the_deletion_queue_until_it_is_empty`]）与整理意图的续做，
//! 续做之前已落盘的写，取号之前在只进内存的盘上走续做。

use super::*;

/// 续做之前这次挂载已落盘的发布（写行那次、暖机每次、写行之后推的抬 F 每一次空发布）各自的写，按先后：续做里的一步没发成、
/// 不是空间不够时随错交出（`PublishSequenceFailed::writes_of_persisted_publishes` 的契约）。
pub(super) fn writes_of_the_publishes_before_the_continuation(
    row_publish: &PoolVersion,
    warm_up_publishes: &[PoolVersion],
    space_admission: &MountSpaceAdmission,
) -> Vec<WritesByStructureKind> {
    std::iter::once(row_publish)
        .chain(warm_up_publishes)
        .map(|version| version.writes().clone())
        .chain(
            space_admission
                .floor_raises()
                .iter()
                .flat_map(|raised| raised.publishes.iter())
                .map(|publish| publish.writes.clone()),
        )
        .collect()
}

/// 整理意图的续做（判决 G1；D8（核心索引结构） 已定项 5 第 2 条「续做是每次恢复都跑的常规动作」；判决 G8 次序：删除续做 → 抬 F → 整理）：
/// 现行那一版 logged_ops 里有一条未完成的意图、删除队列走空了，就照分配器上的整理开关推它，推到它删了、停机原因或预算用完，不签新的。
/// 整理自己不让挂载失败（提示类意图不挡挂载，D8 已定项 5），走到哪儿记进交回值。批大小取分配器上开关给的那一个（产品默认 16，挂载参数）。
/// 回退门不判这一趟：续做只推盘上那一条、不签新的，发布数有上界（1 + ⌈64 ÷ 批大小⌉）。
/// 删除续做没走空时不推（整理排在它后面），一个写都不发。续做的每一批照这次挂载给的后台计算设置记账（`background_compute`）。
pub(super) fn compaction_intent_continued_at_the_mount<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    deletion_queue_continuation_outcome: &DeletionQueueContinuationOutcome,
    background_compute: &mut BackgroundComputeOfThisMount,
) -> CompactionContinuationAtTheMount {
    let mut compaction_state_of_this_mount = CompactionOfThisMount::of_a_new_mount();
    let live_intent_to_continue = match deletion_queue_continuation_outcome {
        DeletionQueueContinuationOutcome::WalkedToEmpty => current
            .version_of_trees()
            .and_then(|version| live_compaction_intent_of_the_entries(&version.logged_operations)),
        DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises { .. } => None,
    };
    let run = if live_intent_to_continue.is_some() {
        Some(run_compaction_on(
            parameters,
            devices,
            allocator,
            &mut compaction_state_of_this_mount,
            current,
            instance,
            CompactionRunReach::ContinueTheLiveIntentAtTheMount,
            background_compute,
        ))
    } else {
        None
    };
    CompactionContinuationAtTheMount {
        run,
        state_of_this_mount: compaction_state_of_this_mount,
    }
}

impl DeletionQueueContinuation {
    /// 续做里已落盘的每一次发布（发成的每一步、推的每一串的每一次空发布）各自的写，按 txg 的先后。
    /// 推的那几串先写系统配置那一步（`RaisedFloor::system_configuration_writes_before_the_first_publish`）不是发布，不在这里。
    fn writes_of_the_persisted_publishes_in_txg_order(&self) -> Vec<WritesByStructureKind> {
        let mut persisted: Vec<&TransactionOutput> = self
            .steps_published
            .iter()
            .map(Arc::as_ref)
            .chain(
                self.floor_raises
                    .iter()
                    .flat_map(|raised| raised.publishes.iter()),
            )
            .collect();
        persisted.sort_by_key(|publish| publish.root.checkpoint_txg);
        persisted
            .into_iter()
            .map(|publish| publish.writes.clone())
            .collect()
    }
}

/// 挂载续做（D8（核心索引结构） 已定项 16 第 6 条）：写行与暖机（连同之后为空间推的抬 F）之后、交回之前，把现行那一版的 `deleted_inodes`
/// 走到空；与会话每次用户改动先走空共用同一个走空函数（`transaction::walk_the_deletion_queue_until_it_is_empty`）。取号之前把它的每一步
/// （为空间推的抬 F 在内）预演一遍、只拒 C841
/// （[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）。
/// 批大小与墓碑容器容量取分配器上装的开关（`mount_writable_with_test_only_switches_and_deletion_switches`）。队列空（常态）时一个写都不发；
/// 走完 `current` 按 `PoolVersion::of_a_version_of_trees` 重新归成员。交回发成的每一步（按先后）、为空间推的那几串抬 F 与停在哪儿。
///
/// 走空里的删除类发布（不判式子那一判、落得下那一判照判，D28（挂载期承诺量） 已定项 1）被拒是空间不够的那两种
/// （[`refusal_is_short_of_space`]）时，推一串抬 F 的空发布（[`push_one_floor_raise_within_the_admission_budget`]）、回收之后接着走
/// （用户 2026-10-03 定案「挂载续做被空间拒：同样推抬 F 再试」，与会话那一处 `mounted_session` 的走空同一个做法）：每一次被拒的删除类发布
/// 各算一次准入（用户 2026-10-03 定案 Q-V 预算「甲 每次被拒各算一次」：这一趟走空有一步落了盘，下一次被拒的就是后面那一次发布，
/// 这次准入已推的数从 0 数起）。推满仍放不下（一次准入的发布数做满、F 已在上限、抬 F 那一串自己也取不到落点或被空间准入拒）时
/// 挂载照样做成可写、队列留着（用户 2026-10-03 定「照 D16 做成可写」，D16（发布语义） 已定项 1「准入」那一行「挂载处推满仍不够时挂载照样做成可写」）：
/// 交回 [`DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises`]，`current` 是最后落盘的那一版；之后会话里每一次用户改动先走空、
/// 走不动报空间不够；正常卸载照常做成，靠的是卸载那一串的预演算上新 F 回收出来、当场放开的那一截（[`unmount`]，用户 2026-10-03 定甲 GA）。
/// 迭代上界：每一轮要么走空交回，要么推满仍放不下交回，要么交回错，要么推成一串（推的数加进这次准入，同一次发布的准入在预算 B − 1 次空发布之内停），
/// 要么这一趟有一步落了盘（队首那个文件少了单元或删掉了，队列与单元数都有限）。跨轮携带的是走到的那一版、已推的数与已落盘的。
///
/// # Errors
/// 走空里的一步没发成、不是空间不够：错连同这次挂载已落盘各次发布的写（`writes_of_the_publishes_before_the_continuation` 接上续做里已落盘的
/// 每一步与推的每一次空发布，按 txg 的先后）、这次挂载推成的每一串抬 F（`floor_raises_pushed_by_the_admission` 接上续做里推成的）
/// 先写系统配置那一步与这一趟走空的写入口的失败账（`MountError::Publish`，[`PublishSequenceFailed`] 的契约）。
/// 抬 F 自己报了不是空间不够的错：那个错装进 `MountError::FloorRaiseFailedAfterTheMountsPublishes`，
/// 连同同一张已落盘发布的写与这次挂载推成的每一串抬 F（准入推的接上续做推的）。
/// 两种都是已经发成的那几步与推成的那几串留在盘上。
#[allow(
    clippy::too_many_arguments,
    reason = "挂载续做要的：参数、盘、分配器、现行那一版、实例、影子账那一臂、续做之前已落盘的写、准入推成的那几串，各自独立"
)]
pub(super) fn continue_the_deletion_queue_until_it_is_empty<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut Vec<(DeviceIdentity, Device)>,
    allocator: &mut PoolAllocator,
    current: &mut PoolVersion,
    instance: InstanceGeneration,
    shadow_ledger: ShadowLedger,
    writes_of_the_publishes_before_the_continuation: Vec<WritesByStructureKind>,
    floor_raises_pushed_by_the_admission: &[RaisedFloor],
) -> Result<DeletionQueueContinuation, MountError> {
    let mut continuation = DeletionQueueContinuation {
        steps_published: Vec::new(),
        floor_raises: Vec::new(),
        refusals_that_pushed_the_floor_raises: Vec::new(),
        outcome: DeletionQueueContinuationOutcome::WalkedToEmpty,
    };
    let current_version_of_trees = match &*current {
        PoolVersion::WithoutFile(_) => return Ok(continuation),
        PoolVersion::WithFile(version) | PoolVersion::WithTreesAndNoFile(version) => version,
    };
    // 销毁中的快照（判决 S4：崩在意图与最后一批之间，下一次可写挂载续做）、截断条目（判决 m4-trunc-r1 K1）与 `deleted_inodes`
    // 同一个走空走它们。
    if current_version_of_trees.queue_of_committed_intents_is_empty() {
        return Ok(continuation);
    }
    let switches = allocator.deletion_switches();
    let mut walked = Arc::clone(current_version_of_trees);
    let mut publishes_pushed_in_this_admission = 0;
    loop {
        let (walk, writes_of_failed_publishes) = {
            let mut pool = PoolWriter::new(parameters, devices.as_mut_slice());
            pool.set_container_tree_capacities(switches.container_tree_capacities());
            let walk = walk_the_deletion_queue_until_it_is_empty(
                &mut pool,
                allocator,
                &mut walked,
                switches.batch_size,
                instance,
            );
            (walk, pool.writes_of_failed_publishes().to_vec())
        };
        let DeletionQueueWalk {
            steps_published,
            stopped_by,
        } = walk;
        if !steps_published.is_empty() {
            publishes_pushed_in_this_admission = 0;
        }
        continuation.steps_published.extend(steps_published);
        let refusal = match stopped_by {
            Ok(()) => {
                *current = PoolVersion::of_a_shared_version_of_trees(walked);
                return Ok(continuation);
            }
            Err(refusal) => refusal,
        };
        if !refusal_is_short_of_space(&refusal) {
            *current = PoolVersion::of_a_shared_version_of_trees(walked);
            let writes_of_persisted_publishes: Vec<WritesByStructureKind> =
                writes_of_the_publishes_before_the_continuation
                    .iter()
                    .cloned()
                    .chain(continuation.writes_of_the_persisted_publishes_in_txg_order())
                    .collect();
            return Err(MountError::Publish(Box::new(PublishSequenceFailed {
                cause: refusal,
                writes_before_the_first_publish: system_configuration_writes_of_the_floor_raises(
                    floor_raises_pushed_by_the_admission
                        .iter()
                        .chain(&continuation.floor_raises),
                ),
                writes_of_persisted_publishes,
                writes_of_failed_publishes,
            })));
        }
        // 抬 F 就地推进走到的那一版：它还与续做交回的最后一步指着同一份时先拷一份自己的再推，交回的那一步不动。
        match push_one_floor_raise_within_the_admission_budget(
            parameters,
            devices,
            allocator,
            Arc::make_mut(&mut walked),
            shadow_ledger,
            publishes_pushed_in_this_admission,
        ) {
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Raised(raised)) => {
                publishes_pushed_in_this_admission += raised.publishes.len();
                continuation.floor_raises.push(raised);
                continuation
                    .refusals_that_pushed_the_floor_raises
                    .push(refusal);
            }
            Ok(FloorRaisePushedWithinTheAdmissionBudget::Stopped(stop)) => {
                *current = PoolVersion::of_a_shared_version_of_trees(walked);
                continuation.outcome =
                    DeletionQueueContinuationOutcome::StillShortAfterTheFloorRaises {
                        last_refusal: refusal,
                        stop,
                    };
                return Ok(continuation);
            }
            Err(cause) => {
                *current = PoolVersion::of_a_shared_version_of_trees(walked);
                return Err(MountError::FloorRaiseFailedAfterTheMountsPublishes(
                    Box::new(FloorRaiseFailedAfterTheMountsPublishes {
                        cause,
                        writes_of_persisted_publishes: [
                            writes_of_the_publishes_before_the_continuation,
                            continuation.writes_of_the_persisted_publishes_in_txg_order(),
                        ]
                        .concat(),
                        floor_raises: floor_raises_pushed_by_the_admission
                            .iter()
                            .cloned()
                            .chain(continuation.floor_raises)
                            .collect(),
                    }),
                ));
            }
        }
    }
}

/// 几串抬 F 各自先写系统配置那一步的写（`RaisedFloor::system_configuration_writes_before_the_first_publish`）逐种相加：挂载续做那一格交
/// `Publish` 时装进 `PublishSequenceFailed::writes_before_the_first_publish`。`WritesByStructureKind` 每一种只记写调用数与字节数之和、
/// 没有加法，这里按种类把两样各自加起来、重放成同样多次写调用（这一种的字节全记在第一次里，其余几次 0 字节）：交出去的每一种的调用数与字节数
/// 与逐串相加相同；合计另核一次，`WrittenStructureKind::IN_REPORT_ORDER` 漏一种就对不上。
fn system_configuration_writes_of_the_floor_raises<'raised>(
    floor_raises: impl IntoIterator<Item = &'raised RaisedFloor>,
) -> WritesByStructureKind {
    let ledgers: Vec<&WritesByStructureKind> = floor_raises
        .into_iter()
        .map(|raised| &raised.system_configuration_writes_before_the_first_publish)
        .collect();
    let mut added_up = WritesByStructureKind::NOTHING_WRITTEN;
    for kind in WrittenStructureKind::IN_REPORT_ORDER {
        let of_this_kind = ledgers
            .iter()
            .fold(WriteCallsAndBytes::NONE, |sum, ledger| {
                sum.plus(ledger.of(kind))
            });
        // 迭代上界是这一种加起来的写调用数；跨轮只多记这一种的一次调用。
        for call_index in 0..of_this_kind.write_calls {
            let bytes_of_this_call = if call_index == 0 {
                usize::try_from(of_this_kind.written_bytes)
                    .expect("几串抬 F 先写系统配置那一步的字节数之和装得进 usize")
            } else {
                0
            };
            added_up.count_write_call(kind, &vec![0_u8; bytes_of_this_call]);
        }
    }
    assert_eq!(
        added_up.total(),
        ledgers
            .iter()
            .fold(WriteCallsAndBytes::NONE, |sum, ledger| sum.plus(ledger.total())),
        "逐种相加的合计与逐串合计之和相同：IN_REPORT_ORDER 罩着每一种，每一种有字节就至少有一次写调用"
    );
    added_up
}

/// 取号之前预演续做、续做要推抬 F 的那一格（[`refuse_before_acquisition_a_continuation_with_a_step_that_opens_a_container_past_the_top_of_its_range`]）：
/// 分配器拷贝上那一趟（`transaction::rehearse_every_step_of_the_deletion_queue_walk`）不读不写盘，推不了抬 F——抬 F 要读根环、实例表与
/// 被抛弃根的账，还要先写系统配置、再发空发布——空间不够的那一步之后就接不上一版。这里改在只进内存的盘上
/// （`mounted_session::DeviceKeepingWritesInMemory`：写只记进内存，读没写过的块穿到 `devices`）与 `allocator` 的一份拷贝上，把取号之后那一串
/// （[`acquire_then_publish_until_the_continuation`]）连同续做（[`continue_the_deletion_queue_until_it_is_empty`]：被拒是空间不够就推一串抬 F、
/// 再试，推满仍放不下就停）照真发走一遍：同一份计划、同一个推抬 F 的判法与同一段抬 F。`devices` 是这次挂载共用的那一份读
/// （`mounted_session::DeviceReadOnceInThisMount`）：这一趟读成而没核出读坏的块记在那里，真做再读同一块取的是同一份字节，所以读都读成、
/// 没有读坏时，续做里报的错就是真发走到那一步时会报的那一个——当场重读那几读、真做取号写之前那几次读与取号写、轮换写算世代号那一读除外：
/// 它们真做时再读一次盘（这一趟取号那几读走共用的那一份），这一刻读到的可以与这一趟不同。
/// 这一趟有一次读穿到盘上、盘报了错（读错，那一块没记下），或读成而读的一方核出自证不过（读坏，那几块已从记下的那一份里丢掉）的，
/// 真做那几块要再读一次盘，可能读得成、走出别的结局，这一格由调用方按读错账与读坏账判、重走。底下那几块盘一个写都不发，真分配器不动。
///
/// 交回走到哪儿：续做走到空或推满仍放不下（`WalkedToItsEnd`）；续做里一步报的错（`MountError::Publish` 的 `cause`，`StepRefused`）；
/// 取号之后那一串在续做之前报错、续做里推的抬 F 自己报错（`StoppedBeforeTheEndOfTheContinuation`）。
pub(super) fn walk_the_continuation_after_acquisition_on_devices_keeping_writes_in_memory<
    Device: BlockDevice,
>(
    parameters: &MakeFilesystemParameters,
    devices: &[(DeviceIdentity, Device)],
    allocator: &PoolAllocator,
    start: &InstanceStart,
    rows_written: &[InstanceRow],
    publishes_planned: &PublishesPlannedAfterAcquisition,
) -> ContinuationRehearsedBeforeAcquisition {
    let mut devices_keeping_writes_in_memory: Vec<(
        DeviceIdentity,
        DeviceKeepingWritesInMemory<'_>,
    )> = devices
        .iter()
        .map(|(identity, device)| {
            let disk: &dyn BlockDevice = device;
            (*identity, DeviceKeepingWritesInMemory::over_the_disk(disk))
        })
        .collect();
    let Ok(PublishedBetweenAcquisitionAndTheContinuation {
        instance,
        space_admission,
        mut current,
        mut allocator,
        ..
    }) = acquire_then_publish_until_the_continuation(
        parameters,
        &mut devices_keeping_writes_in_memory,
        allocator.clone(),
        start,
        rows_written,
        publishes_planned.clone(),
        ReadsOfTheAcquisition::ThroughTheDevicesOfTheRehearsal,
    )
    else {
        return ContinuationRehearsedBeforeAcquisition::StoppedBeforeTheEndOfTheContinuation;
    };
    let continuation = continue_the_deletion_queue_until_it_is_empty(
        parameters,
        &mut devices_keeping_writes_in_memory,
        &mut allocator,
        &mut current,
        instance,
        start.shadow_ledger,
        // 续做之前已落盘的写只随续做的错一起交出、不影响续做怎么走：这里只取那个错，不另算这一份。
        Vec::new(),
        space_admission.floor_raises(),
    );
    let Err(error_of_the_continuation) = continuation else {
        return ContinuationRehearsedBeforeAcquisition::WalkedToItsEnd;
    };
    if let MountError::Publish(failed) = error_of_the_continuation {
        ContinuationRehearsedBeforeAcquisition::StepRefused(failed.cause)
    } else {
        ContinuationRehearsedBeforeAcquisition::StoppedBeforeTheEndOfTheContinuation
    }
}

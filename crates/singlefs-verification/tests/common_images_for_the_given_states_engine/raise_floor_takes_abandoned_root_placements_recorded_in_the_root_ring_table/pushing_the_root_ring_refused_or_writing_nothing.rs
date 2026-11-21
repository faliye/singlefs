//! 推环入口拒或一槽不写的几形：块设备错之后再推拒成冻结、两块盘体对调、C 记着落点而一槽都没隔离时不报被宽护卡住。

use super::*;

/// 一份挂着的会话逐字段拷一份（同一刻的分配器、现行那一版、实例代号、影子账那一臂、认下的池）：同一个盘面上起几份池各推各的。
pub fn copy_of_the_session(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
        compaction: session.compaction.clone(),
    }
}

/// 整池的改盘调用（写、写零、屏障，几块盘共用一个数）数到第 `failing_call` 次（从 1 数）时报块设备错、不落盘，只这一次；`None` 只数不报。
/// 另数整池的读与探物理块大小（会话发布用户改动的入口、推抬 F 那一串核盘表、只进内存的盘一建，各探一次底下那块盘）。
pub struct ChangingCallsOfThePool {
    pub calls: Cell<u64>,
    pub failing_call: Option<u64>,
    pub reads: Cell<u64>,
    pub probes_of_the_physical_block_size: Cell<u64>,
}

pub struct DeviceFailingOneChangingCall {
    pub inner: SparseBlockDevice,
    pub calls_of_the_pool: Rc<ChangingCallsOfThePool>,
}

impl DeviceFailingOneChangingCall {
    pub fn count_one_changing_call(&self) -> Result<(), BlockDeviceError> {
        let call = self.calls_of_the_pool.calls.get() + 1;
        self.calls_of_the_pool.calls.set(call);
        if self.calls_of_the_pool.failing_call == Some(call) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "用例在这一次改盘调用上注入的块设备错",
            )));
        }
        Ok(())
    }
}

impl BlockDevice for DeviceFailingOneChangingCall {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let reads = &self.calls_of_the_pool.reads;
        reads.set(reads.get() + 1);
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.count_one_changing_call()?;
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.count_one_changing_call()?;
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.count_one_changing_call()?;
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        let probes = &self.calls_of_the_pool.probes_of_the_physical_block_size;
        probes.set(probes.get() + 1);
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

impl DeviceOverASparseImage for DeviceFailingOneChangingCall {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

/// `stuck_pool` 那一刻的盘面与会话拷一份，盘外面包上数改盘调用的那一层（第 `failing_call` 次报错，`None` 只数）。
pub fn pool_failing_one_changing_call_copied_from(
    stuck_pool: &PoolUnderTest<SparseBlockDevice>,
    failing_call: Option<u64>,
) -> (
    PoolUnderTest<DeviceFailingOneChangingCall>,
    Rc<ChangingCallsOfThePool>,
) {
    let calls_of_the_pool = Rc::new(ChangingCallsOfThePool {
        calls: Cell::new(0),
        failing_call,
        reads: Cell::new(0),
        probes_of_the_physical_block_size: Cell::new(0),
    });
    let devices = plain_devices_on(&stuck_pool.image())
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceFailingOneChangingCall {
                    inner,
                    calls_of_the_pool: Rc::clone(&calls_of_the_pool),
                },
            )
        })
        .collect();
    (
        PoolUnderTest {
            parameters: stuck_pool.parameters.clone(),
            device_bytes: stuck_pool.device_bytes,
            devices,
            session: Some(copy_of_the_session(stuck_pool.session())),
            write_time_seconds: stuck_pool.write_time_seconds,
            content_of_the_current_version: stuck_pool.content_of_the_current_version.clone(),
            content_of_each_root: stuck_pool.content_of_each_root.clone(),
        },
        calls_of_the_pool,
    )
}

/// 推环入口先判冻结：主形卡住之后推环，真发到半路一次改盘调用报块设备错——前面那几次已落盘，
/// 会话的现行那一版停在最后一次落盘的那一版上（攻方 M1），失败那一次冻结在分配器上。再推一次（盘表照挂载时那一份交，过得了入口比盘表次序那一道）：
/// 拒成点名冻结的那一种、带冻结那一次的 txg 与实例代号，在核盘表那一套与预演之前，两块盘逐字节不变、分配器与会话不动。预演若在冻结着的分配器拷贝上照样过，
/// 报成 `PublishFailedAfterTheRehearsal`、拒因是冻结。
pub fn run_the_scenario_where_pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        pool: stuck_pool, ..
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
    let (mut counting_pool, counted_calls) =
        pool_failing_one_changing_call_copied_from(&stuck_pool, None);
    let pushed_without_a_fault = push_the_root_ring_of(&mut counting_pool)
        .unwrap_or_else(|refusal| panic!("不注入故障时推环做得成：{refusal:?}"));
    assert!(
        pushed_without_a_fault.versions_published.len() >= 3,
        "推环要发好几次（阳性对照：半路失败时前面有几次已落盘）：{} 次",
        pushed_without_a_fault.versions_published.len()
    );
    let failing_call = counted_calls.calls.get() / 2;
    let (mut failing_pool, _) =
        pool_failing_one_changing_call_copied_from(&stuck_pool, Some(failing_call));
    let Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(failed)) =
        push_the_root_ring_of(&mut failing_pool)
    else {
        panic!("第 {failing_call} 次改盘调用报块设备错：推环真发到半路发不出去");
    };
    assert!(
        failed.failed_publish_in_the_push > 1
            && failed.failed_publish_in_the_push <= failed.publishes_in_the_push,
        "失败在半路（前面有几次已落盘）：第 {} 次，共 {} 次",
        failed.failed_publish_in_the_push,
        failed.publishes_in_the_push
    );
    assert_eq!(
        failed.versions_published.len(),
        failed.failed_publish_in_the_push - 1,
        "失败那一次之前的每一次都落了盘、交了回来"
    );
    let last_landed = failed
        .versions_published
        .last()
        .expect("失败在第二次及以后：前面至少一次已落盘");
    assert_eq!(
        *failing_pool.session().current.root(),
        last_landed.root,
        "真发半路失败：会话的现行那一版停在最后一次落盘的那一版上"
    );
    let frozen = failing_pool
        .session()
        .allocator
        .frozen_publish()
        .map(|frozen| (frozen.checkpoint_txg(), frozen.instance()))
        .expect("落盘途中失败的那一次冻结在分配器上（阳性对照）");
    assert_eq!(
        frozen.0,
        CheckpointTxg(last_landed.root.checkpoint_txg.0 + 1),
        "冻结的是最后一次落盘之后的那一次"
    );
    let image_before_pushing_again = failing_pool.image();
    let allocator_before_pushing_again = format!("{:?}", failing_pool.session().allocator);
    let current_before_pushing_again = *failing_pool.session().current.root();
    match push_the_root_ring_of(&mut failing_pool) {
        Err(RootRingPushRefused::PublishFrozenAfterAWriteFailureIsNotResentYet {
            checkpoint_txg,
            instance,
        }) => assert_eq!(
            (checkpoint_txg, instance),
            frozen,
            "拒成点名冻结的那一种，带冻结那一次的 txg 与实例代号"
        ),
        other => panic!("冻结着再推一次：在预演之前拒成点名冻结的那一种，而交回 {other:?}"),
    }
    assert!(
        failing_pool.image() == image_before_pushing_again,
        "冻结着再推一次：两块盘逐字节不变"
    );
    assert_eq!(
        format!("{:?}", failing_pool.session().allocator),
        allocator_before_pushing_again,
        "冻结着再推一次：分配器整份不变"
    );
    assert_eq!(
        *failing_pool.session().current.root(),
        current_before_pushing_again,
        "冻结着再推一次：会话的现行那一版不变"
    );
}

/// 推环入口核交进来的盘表（攻方 M2）：主形卡住之后两块盘的盘体对调（身份照旧、盘上的本盘设备号对不上），推环在任何写之前拒成盘表那一种，
/// 两块盘逐字节不变、分配器与会话不动。不核时推环照常发、本池的写落到对调了的盘上。
pub fn run_the_scenario_where_pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection { mut pool, .. } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
    {
        let (first, second) = pool.devices.split_at_mut(1);
        std::mem::swap(&mut first[0].1, &mut second[0].1);
    }
    let image_before = pool.image();
    let allocator_before = format!("{:?}", pool.session().allocator);
    let current_before = *pool.session().current.root();
    let pushed = push_the_root_ring_of(&mut pool);
    assert!(
        matches!(pushed, Err(RootRingPushRefused::DevicesHandedIn(_))),
        "盘体对调：推环拒成交进来的盘表没过核，而交回 {pushed:?}"
    );
    assert!(pool.image() == image_before, "推环被拒：两块盘逐字节不变");
    assert_eq!(
        format!("{:?}", pool.session().allocator),
        allocator_before,
        "推环被拒：分配器整份不变"
    );
    assert_eq!(
        *pool.session().current.root(),
        current_before,
        "推环被拒：会话的现行那一版不变"
    );
}

/// C 那一圈（这一刻记着落点的那几槽）罩着的槽里这一刻隔离着的槽数，两块盘合计。
pub fn isolated_slots_covered_by_the_ring_slots_holding_placements(
    allocator: &PoolAllocator,
) -> u64 {
    let placements: Vec<PlacementOnDevice> = allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .ring_slots_holding_placements_named_by_unapplied_journal_records()
        .into_values()
        .flatten()
        .collect();
    allocator
        .isolated_slots_covered_by(&placements)
        .iter()
        .map(|(_, isolated)| *isolated)
        .sum()
}

/// K1-1「三个量」注入级那一形：C 那一圈记着落点、它们罩着的槽却一个都没隔离。主形（建 inode、前缀 40 次）卡住之后，
/// 把 C 那几槽记着的落点换成现行账里仍分配着的那几个落点（分配位置着、不隔离；原来那一圈置的隔离位照旧留着、空间照旧不够）。
pub fn pool_stuck_while_the_ring_slots_hold_placements_none_of_which_is_isolated(
    images: &mut PoolCheckerImages,
) -> PoolUnderTest<SparseBlockDevice> {
    let PoolStuckBehindTheWideProtection { mut pool, .. } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
    let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
    let still_allocated: Vec<PlacementOnDevice> = allocator
        .records()
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| PlacementOnDevice {
            device: record.device,
            slot: record.slot,
            span: u64::from(record.span_slots),
        })
        .collect();
    assert!(
        !still_allocated.is_empty(),
        "现行账里有仍分配着的落点（阳性对照）"
    );
    let replaced: BTreeMap<RootRingSlot, Vec<PlacementOnDevice>> = allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .ring_slots_holding_placements_named_by_unapplied_journal_records()
        .into_keys()
        .map(|ring_slot| (ring_slot, still_allocated.clone()))
        .collect();
    assert!(
        !replaced.is_empty(),
        "主形卡住时 C 那一圈记着落点（阳性对照）"
    );
    allocator.note_placements_named_by_unapplied_journal_records(replaced);
    assert_eq!(
        isolated_slots_covered_by_the_ring_slots_holding_placements(allocator),
        0,
        "注入之后 C 那一圈罩着的槽一个都没隔离（阳性对照）"
    );
    assert!(
        isolated_slots_of(allocator) > 0,
        "原来那一圈置的隔离位照旧留着（空间照旧不够）"
    );
    pool
}

/// C 那一圈记着落点而一个槽都没隔离：推满仍不够、F 在上限时照旧只报空间不够，不报「被宽护卡住」（正推 `noIsoCheck`）。
pub fn run_the_scenario_where_no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection(
    images: &mut PoolCheckerImages,
) {
    let mut pool =
        pool_stuck_while_the_ring_slots_hold_placements_none_of_which_is_isolated(images);
    let refusal = pool
        .overwrite(OVERWRITE_BYTES)
        .expect_err("空间照旧不够：覆盖写被拒");
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
        ),
        "推抬 F 停在上限（阳性对照：与被宽护卡住同一种停法）：{:?}",
        no_space.stop
    );
    assert!(
        matches!(
            no_space.wide_protection_at_the_ceiling,
            WideProtectionAtTheCeiling::NotWhatStopsTheAdmission
        ),
        "C 那一圈罩着的槽一个都没隔离：不报被宽护卡住，而报 {:?}",
        no_space.wide_protection_at_the_ceiling
    );
}

/// C 那一圈记着落点而一个槽都没隔离：推环入口在任何写之前拒成「写掉哪一槽都放不出东西」那一种，两块盘逐字节不变、分配器不动。
pub fn run_the_scenario_where_pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing(
    images: &mut PoolCheckerImages,
) {
    let mut pool =
        pool_stuck_while_the_ring_slots_hold_placements_none_of_which_is_isolated(images);
    let image_before = pool.image();
    let allocator_before = format!("{:?}", pool.session().allocator);
    let pushed = push_the_root_ring_of(&mut pool);
    assert!(
        matches!(
            pushed,
            Err(RootRingPushRefused::NothingHeldUntilThisProcessWritesARingSlot)
        ),
        "C 那一圈罩着的槽一个都没隔离：不推，而交回 {pushed:?}"
    );
    assert!(pool.image() == image_before, "不推：两块盘逐字节不变");
    assert_eq!(
        format!("{:?}", pool.session().allocator),
        allocator_before,
        "不推：分配器整份不变"
    );
}

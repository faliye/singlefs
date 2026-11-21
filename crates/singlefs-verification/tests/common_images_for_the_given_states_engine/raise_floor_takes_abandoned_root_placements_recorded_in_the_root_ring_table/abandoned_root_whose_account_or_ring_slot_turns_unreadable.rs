//! 崩了再挂抛弃的根 C：挂上之后 C 的账或 C 的根槽读不出，卸载与会话写照根环表里记着的 C 的落点抬 F 的场景，与造这个池、注入读错的几样。

use super::*;

/// C 那一次覆盖写的内容长度（攻方原型的 2500 字节）：与之前之后的覆盖写长度不同，读回哪一版看得出。
pub const CONTENT_BYTES_OF_THE_ABANDONED_VERSION: usize = 2500;

/// 挂着时 C 的账读不出之后，会话至多覆盖写几次去等准入推出第一串抬 F。
pub const OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST: usize = 60;

/// 准入推过第一串抬 F 之后再覆盖写几次（推过之后的写也不拒）。
pub const OVERWRITES_AFTER_THE_FIRST_FLOOR_RAISE: usize = 3;

/// 一份崩溃恢复抛弃了最新那条根 C 之后、进程退出的盘面（还没再挂）。
pub struct PoolWithAnAbandonedRoot {
    pub parameters: MakeFilesystemParameters,
    pub device_bytes: u64,
    pub image: MemoryPool,
    pub write_time_seconds: u64,
    pub abandoned_root: RootRecord,
    /// 实例 3 退出时现行那一版的根（按实例 3 的表有效）：下一次可写挂载择到它，根环表把它记成有效根。
    pub newest_valid_root: RootRecord,
}

pub fn read_bytes_of(
    pool: &PoolUnderTest<SparseBlockDevice>,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length_in_bytes: usize,
) -> Vec<u8> {
    let (_, block_device) = pool
        .devices
        .iter()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    let mut bytes = vec![0u8; length_in_bytes];
    block_device
        .read_at(offset, &mut bytes)
        .expect("内存稀疏盘上的读做得成");
    bytes
}

pub fn write_bytes_to(
    pool: &mut PoolUnderTest<SparseBlockDevice>,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    bytes: &[u8],
) {
    let (_, block_device) = pool
        .devices
        .iter_mut()
        .find(|(identity, _)| *identity == device)
        .expect("池里有这块盘");
    block_device
        .write_at(offset, bytes, WriteDurability::Plain)
        .expect("内存稀疏盘上的写做得成");
}

/// 攻方原型 `abandoned_start` 照搬（之前覆盖写 1 次）：见文件头。
pub fn pool_whose_newest_root_was_abandoned_by_a_crash_recovery() -> PoolWithAnAbandonedRoot {
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        plain_device,
    );
    pool.overwrite(OVERWRITE_BYTES).expect("实例 1 覆盖写一次");
    pool.crash_and_mount_writable().expect("崩了再挂：取号 2");
    pool.overwrite(CONTENT_BYTES_OF_THE_ABANDONED_VERSION)
        .expect("实例 2 覆盖写一次：C");
    let abandoned_version = pool.current_file_version().clone();
    let parameters = pool.parameters.clone();
    let ring_slot_of_the_abandoned_root = target_for_publish(
        abandoned_version.root.checkpoint_txg,
        parameters.geometry.root_ring_slots_per_region,
    );
    let root_slot_device = parameters.region_devices
        [usize::try_from(ring_slot_of_the_abandoned_root.region).expect("区域号装得进 usize")];
    let root_slot_offset = slot_offset(
        ring_slot_of_the_abandoned_root,
        parameters.geometry.fixed_structure_slot_spacing,
    );
    let root_slot_bytes =
        usize::try_from(parameters.geometry.physical_block_size).expect("物理块装得进 usize");
    let data_unit_bytes =
        usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("数据单元装得进 usize");
    let mut hidden_places: Vec<(DeviceIdentity, DeviceOffsetInBytes, usize)> =
        vec![(root_slot_device, root_slot_offset, root_slot_bytes)];
    for pointer in &abandoned_version.data_pointers {
        for location in pointer.locations {
            hidden_places.push((
                location.device,
                location.slot.to_device_offset(),
                data_unit_bytes,
            ));
        }
    }
    let mut hidden_bytes = Vec::new();
    for (device, offset, length_in_bytes) in &hidden_places {
        let bytes = read_bytes_of(&pool, *device, *offset, *length_in_bytes);
        write_bytes_to(&mut pool, *device, *offset, &vec![0u8; *length_in_bytes]);
        hidden_bytes.push((*device, *offset, bytes));
    }
    let slot_spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let rotations_rolled_back: Vec<(DeviceIdentity, DeviceOffsetInBytes, Vec<u8>)> = pool
        .devices
        .iter()
        .map(|(identity, _)| {
            let mut verified = singlefs_core::recovery::verified_system_configuration_slots(
                pool.devices.as_slice(),
                *identity,
                slot_spacing,
                &parameters.filesystem_identifier,
            );
            verified.sort_by_key(|slot| slot.quantities.slot_generation);
            let [older, newest] =
                <[_; 2]>::try_from(verified).expect("每块盘上系统配置两槽都自证得过");
            let newest_generation = newest.quantities.slot_generation;
            let mut not_rotated = older;
            not_rotated.quantities.slot_generation = newest_generation
                .checked_sub(2)
                .expect("覆盖写过几次之后槽代号不小于 2");
            (
                *identity,
                DeviceOffsetInBytes(
                    (newest_generation % singlefs_format::SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE)
                        * slot_spacing,
                ),
                not_rotated.to_slot(),
            )
        })
        .collect();
    for (device, offset, bytes) in &rotations_rolled_back {
        write_bytes_to(&mut pool, *device, *offset, bytes);
    }
    let abandoning = pool
        .crash_and_mount_writable()
        .expect("崩了再挂：择根落到 C 之前那一条、取号 3");
    assert!(
        abandoning.effective_root.checkpoint_txg < abandoned_version.root.checkpoint_txg,
        "择根落到 C 之前那一条：{:?}",
        abandoning.effective_root
    );
    for (device, offset, bytes) in &hidden_bytes {
        write_bytes_to(&mut pool, *device, *offset, bytes);
    }
    let newest_valid_root = *pool.session().current.root();
    pool.session = None;
    PoolWithAnAbandonedRoot {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        image: pool.image(),
        write_time_seconds: pool.write_time_seconds,
        abandoned_root: abandoned_version.root,
        newest_valid_root,
    }
}

/// 挑出来的几段的读故障（一条根的账那几个单元，或一条根的根槽）：`is_active` 为真时，读到 `ranges` 里任一段（盘、起点字节、终点字节）
/// 就报错，盘上的字节不动。几块盘共用一份。
#[derive(Clone)]
pub struct ReadFaultOnChosenSlots {
    pub ranges: Rc<Vec<(DeviceIdentity, u64, u64)>>,
    pub is_active: Rc<Cell<bool>>,
}

impl ReadFaultOnChosenSlots {
    /// 这条根住的那个根环槽（它的 txg 在根环里对应的那一槽，一个物理块）。故障先关着。
    pub fn on_the_root_ring_slot_of(
        root: &RootRecord,
        parameters: &MakeFilesystemParameters,
    ) -> Self {
        let ring_slot = target_for_publish(
            root.checkpoint_txg,
            parameters.geometry.root_ring_slots_per_region,
        );
        let device = parameters.region_devices
            [usize::try_from(ring_slot.region).expect("区域号装得进 usize")];
        let start_in_bytes =
            slot_offset(ring_slot, parameters.geometry.fixed_structure_slot_spacing).0;
        Self {
            ranges: Rc::new(vec![(
                device,
                start_in_bytes,
                start_in_bytes + u64::from(parameters.geometry.physical_block_size),
            )]),
            is_active: Rc::new(Cell::new(false)),
        }
    }

    /// 这条根的树表、中央映射树根、分配记录树根（空指针不算）的每条位置条目各一个槽。故障先关着。
    pub fn on_the_account_of(root: &RootRecord) -> Self {
        let ranges = [
            root.tree_table,
            root.mapping_root,
            root.allocation_record_tree_root,
        ]
        .into_iter()
        .filter(|pointer| *pointer != NodePointer::empty_root())
        .flat_map(|pointer| pointer.locations)
        .map(|location| {
            let start_in_bytes = location.slot.to_device_offset().0;
            (
                location.device,
                start_in_bytes,
                start_in_bytes + singlefs_format::SLOT_BYTES,
            )
        })
        .collect();
        Self {
            ranges: Rc::new(ranges),
            is_active: Rc::new(Cell::new(false)),
        }
    }

    pub fn refuses(
        &self,
        device: DeviceIdentity,
        offset_in_bytes: u64,
        length_in_bytes: u64,
    ) -> bool {
        self.is_active.get()
            && self.ranges.iter().any(|(range_device, start, end)| {
                *range_device == device
                    && offset_in_bytes < *end
                    && offset_in_bytes + length_in_bytes > *start
            })
    }
}

pub struct DeviceWithReadFaultsOnChosenSlots {
    pub identity: DeviceIdentity,
    pub inner: SparseBlockDevice,
    pub fault: ReadFaultOnChosenSlots,
}

impl BlockDevice for DeviceWithReadFaultsOnChosenSlots {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
        if self.fault.refuses(self.identity, offset.0, length_in_bytes) {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other(
                "挑出来的那几段从这一刻起读不出",
            )));
        }
        self.inner.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

impl DeviceOverASparseImage for DeviceWithReadFaultsOnChosenSlots {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

pub fn devices_with_the_read_fault_on(
    image: &MemoryPool,
    fault: &ReadFaultOnChosenSlots,
) -> Vec<(DeviceIdentity, DeviceWithReadFaultsOnChosenSlots)> {
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceWithReadFaultsOnChosenSlots {
                    identity,
                    inner,
                    fault: fault.clone(),
                },
            )
        })
        .collect()
}

/// 挂着的池（盘上带着 `fault` 那一层）：会话是 `mounted` 那一次挂载的。
pub fn pool_under_test_after_the_mount(
    parameters: &MakeFilesystemParameters,
    device_bytes: u64,
    devices: Vec<(DeviceIdentity, DeviceWithReadFaultsOnChosenSlots)>,
    mounted: singlefs_core::mount::Mounted,
    write_time_seconds: u64,
) -> PoolUnderTest<DeviceWithReadFaultsOnChosenSlots> {
    let (_, session) = MountedSession::of_the_mount(mounted);
    PoolUnderTest {
        parameters: parameters.clone(),
        device_bytes,
        devices,
        session: Some(session),
        write_time_seconds,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    }
}

/// 可写挂载一次（实例 4，空间准入判着、影子账开着），`fault` 关着：C 的根槽与账都读得出、根环表把 C 记成被抛弃根。
/// 交回挂着的池与挂载那一刻根环表里记着的 C 的落点（非空）。
pub fn mounted_while_the_abandoned_root_reads(
    built: &PoolWithAnAbandonedRoot,
    fault: &ReadFaultOnChosenSlots,
) -> (
    PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
    Vec<PlacementOnDevice>,
) {
    let mut devices = devices_with_the_read_fault_on(&built.image, fault);
    let mounted = mount_writable_with_space_admission(
        &built.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("C 的根槽与账读得出：可写挂载做得成");
    let recorded = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_recorded_for_the_abandoned_root(
            built.abandoned_root.instance,
            built.abandoned_root.checkpoint_txg,
        )
        .filter(|placements| !placements.is_empty())
        .expect("挂载那一刻根环表把 C 记成被抛弃根、带着它引用的落点");
    let pool = pool_under_test_after_the_mount(
        &built.parameters,
        built.device_bytes,
        devices,
        mounted,
        built.write_time_seconds + 1000,
    );
    (pool, recorded)
}

/// 抬 F 之后 `placements` 里每个落点覆盖的每一槽（起点槽起、跨度那么多槽）在分配器里都不空闲：隔离着，或现行账里还分配着。
/// 被抛弃根离开根环之前它引用的单元不许发出去（D23（journal 的角色与格式） 已定项 14 回退段主句）；抬 F 越过一条候选根之后，
/// 之前靠它豁免、现行账里已释放的落点只靠影子账这一遍补的隔离护着。只核起点槽时，补隔离只隔起点那一槽（跨度给成 1）照绿
/// （正推 `research/prompts/m3-batch10z-d-code-r1-sonnet-output.md` K3 `loop-span1`）。
pub fn assert_none_of_the_placements_is_free(
    allocator: &PoolAllocator,
    placements: &[PlacementOnDevice],
    whose: &str,
) {
    let free_slots = free_slots_covered_by(allocator, placements);
    assert!(
        free_slots.is_empty(),
        "抬 F 之后{whose}的落点覆盖的槽一槽都不空闲（隔离着或现行账里还分配着）：空闲的（盘、槽）{free_slots:?}（共 {} 个落点）",
        placements.len()
    );
}

/// `placements` 里每个落点覆盖的槽（起点槽起、跨度那么多槽）里此刻在分配器里空闲的那几个（盘、槽），按落点的次序。
pub fn free_slots_covered_by(
    allocator: &PoolAllocator,
    placements: &[PlacementOnDevice],
) -> Vec<(DeviceIdentity, SlotNumber)> {
    placements
        .iter()
        .flat_map(|placement| {
            let device_map = allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == placement.device)
                .expect("池里有这块盘");
            (placement.slot.0..placement.slot.0 + placement.span)
                .map(SlotNumber)
                .filter(move |slot| device_map.is_free(*slot))
                .map(move |slot| (placement.device, slot))
        })
        .collect()
}

/// R 自己写出的每个单元的落点（盘、起点槽、跨度：写长按槽宽折成槽数）：从录下来的写算，不经被测的方法。
pub fn placements_written_by(recorded: &RecordedPublish) -> Vec<PlacementOnDevice> {
    recorded
        .unit_writes()
        .into_iter()
        .map(|write| {
            assert_eq!(
                write.offset.0 % singlefs_format::SLOT_BYTES,
                0,
                "单元写落在槽边界上：{write:?}"
            );
            PlacementOnDevice {
                device: write.device,
                slot: SlotNumber(write.offset.0 / singlefs_format::SLOT_BYTES),
                span: write
                    .length_in_bytes()
                    .div_ceil(singlefs_format::SLOT_BYTES),
            }
        })
        .collect()
}

/// 故障开着之后 C 的树表那一槽确实读不出（阳性对照：用例不是在读得出的盘上空转）。
pub fn assert_the_tree_table_of_the_abandoned_root_is_unreadable(
    pool: &PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
    abandoned_root: &RootRecord,
) {
    let location = abandoned_root.tree_table.locations[0];
    let (_, block_device) = pool
        .devices
        .iter()
        .find(|(identity, _)| *identity == location.device)
        .expect("C 的树表所在的盘");
    let mut bytes = vec![0u8; usize::try_from(singlefs_format::SLOT_BYTES).expect("槽宽")];
    assert!(
        block_device
            .read_at(location.slot.to_device_offset(), &mut bytes)
            .is_err(),
        "故障开着：C 的树表那一槽读不出"
    );
}

/// 交给定状态引擎时核的两条：I-7.4（近 K 代块未被复用）与 I-4.8（近 K 代根校验和自洽），被抛弃根引用的单元没被复用就不红。
pub const INVARIANTS_OF_UNIT_REUSE: [&str; 2] = ["I-7.4", "I-4.8"];

/// 正常卸载一次（影子账开着），要它做成（推一串抬 F 到现行那一版）。交回这一串的 `abandoned_roots_unreadable` 与卸载之后的分配器。
/// `why` 只用在断言消息里，说是哪一形。
pub fn unmount_expecting_the_floor_raised(
    pool: &mut PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
    why: &str,
) -> (u64, PoolAllocator) {
    let mut session = pool.session.take().expect("挂着的会话");
    let unmounted = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    match unmounted {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => {
            (raised.abandoned_roots_unreadable, session.allocator)
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("{why}：卸载照根环表里记着的落点做成，而被拒：{refusal:?}"),
    }
}

/// 挂上之后 C 的账读不出：正常卸载照根环表里记着的 C 的落点算影子账，做成；卸载之后 C 记着的落点一槽都不空闲，
/// 池级 checker 不红 I-7.4（近 K 代块未被复用）与 I-4.8。
/// 卸载不拒（拒则是 `NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`）。
pub fn run_the_scenario_where_unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
    images: &mut PoolCheckerImages,
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
    fault.is_active.set(true);
    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &built.abandoned_root);
    let (abandoned_roots_unreadable, allocator_after_the_unmount) =
        unmount_expecting_the_floor_raised(&mut pool, "C 的账读不出");
    assert_eq!(
        abandoned_roots_unreadable, 0,
        "C 的落点取根环表里记着的那一份，这一遍不读它的账"
    );
    assert_none_of_the_placements_is_free(&allocator_after_the_unmount, &recorded, "C 记着");
    images.hand_out(
        "卸载之后 C 引用的单元一个都没复用",
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

/// 这一刻根环表里还把 C 记成被抛弃根（这个进程还没写掉 C 那一槽）。
pub fn the_root_ring_table_still_records_the_abandoned_root(
    pool: &PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
    abandoned_root: &RootRecord,
) -> bool {
    pool.session()
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_recorded_for_the_abandoned_root(
            abandoned_root.instance,
            abandoned_root.checkpoint_txg,
        )
        .is_some()
}

/// 会话接着覆盖写，直到准入推出第一串抬 F（推的那一刻 C 还在根环里），一次都不许拒。`why` 只用在断言消息里。
pub fn overwrite_until_the_admission_pushes_the_first_floor_raise(
    pool: &mut PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
    abandoned_root: &RootRecord,
    why: &str,
) {
    let mut overwrites_before_the_first_floor_raise = 0;
    // 迭代上界 OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST；跨轮不带状态（计数之外）；出口：推出抬 F 的那一次 break，被拒就当场判红（panic）。
    loop {
        assert!(
            overwrites_before_the_first_floor_raise < OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST,
            "{OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST} 次覆盖写之内准入推出抬 F（384 槽小盘）"
        );
        let abandoned_root_was_in_the_ring =
            the_root_ring_table_still_records_the_abandoned_root(pool, abandoned_root);
        let published = pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!(
                "第 {} 次覆盖写：{why}，准入抬 F 照根环表里记着的落点做成，而被拒：{refusal:?}",
                overwrites_before_the_first_floor_raise + 1
            )
        });
        overwrites_before_the_first_floor_raise += 1;
        if !published.floor_raises.is_empty() {
            assert!(
                abandoned_root_was_in_the_ring,
                "准入推抬 F 的那一刻 C 还在根环里（不然这一串里没有被抛弃根要算，没走到要判的那一处）"
            );
            return;
        }
    }
}

/// 挂上之后 C 的账读不出：会话接着覆盖写到准入推出第一串抬 F（推的那一刻 C 还在根环里），再写几次，一次都不拒——准入推的抬 F
/// 照根环表里记着的 C 的落点算影子账；推过那一刻 C 记着的落点一槽都不空闲。写完池级 checker 不红 I-7.4 与 I-4.8。
/// 推抬 F 的那一次覆盖写不拒（拒则是 `FloorRaiseFailedWhilePushingForSpace`，拒因点名 C 的账）。
pub fn run_the_scenario_where_session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
    images: &mut PoolCheckerImages,
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
    fault.is_active.set(true);
    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &built.abandoned_root);
    overwrite_until_the_admission_pushes_the_first_floor_raise(
        &mut pool,
        &built.abandoned_root,
        "C 的账读不出",
    );
    assert_none_of_the_placements_is_free(&pool.session().allocator, &recorded, "C 记着");
    for overwrite_number in 1..=OVERWRITES_AFTER_THE_FIRST_FLOOR_RAISE {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("推过抬 F 之后第 {overwrite_number} 次覆盖写被拒：{refusal:?}")
        });
    }
    images.hand_out(
        "这一串覆盖写之后 C 引用的单元一个都没复用",
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

/// 故障开着之后这条根的根槽确实读不出（阳性对照）。
pub fn assert_the_root_ring_slot_is_unreadable(
    pool: &PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
    root: &RootRecord,
) {
    let ring_slot = target_for_publish(
        root.checkpoint_txg,
        pool.parameters.geometry.root_ring_slots_per_region,
    );
    let device = pool.parameters.region_devices
        [usize::try_from(ring_slot.region).expect("区域号装得进 usize")];
    let (_, block_device) = pool
        .devices
        .iter()
        .find(|(identity, _)| *identity == device)
        .expect("根槽所在的盘");
    let mut bytes = vec![
        0u8;
        usize::try_from(pool.parameters.geometry.physical_block_size)
            .expect("物理块装得进 usize")
    ];
    assert!(
        block_device
            .read_at(
                slot_offset(
                    ring_slot,
                    pool.parameters.geometry.fixed_structure_slot_spacing
                ),
                &mut bytes
            )
            .is_err(),
        "故障开着：({:?}, {:?}) 那一条根的根槽读不出",
        root.instance,
        root.checkpoint_txg
    );
}

/// H1（被抛弃根的槽不拒）：挂上之后 C 的根槽读报错、账读得出。正常卸载那一串重算影子账读根环时 C 那一槽读坏、
/// 重读仍坏不拒，照根环表里记着的 C 的落点进影子账：卸载做成、这一遍不读 C 的账（计数 0）、C 记着的落点一槽都不空闲，
/// 池级 checker 不红 I-7.4 与 I-4.8。那一槽若算「这个进程知道住着根的槽」，重读仍坏就拒
/// （`NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`，攻方 `research/prompts/m3-batch10z-b-code-r1-opus-output.md` K1-1）。
pub fn run_the_scenario_where_unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
    images: &mut PoolCheckerImages,
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault =
        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
    fault.is_active.set(true);
    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
    let (abandoned_roots_unreadable, allocator_after_the_unmount) =
        unmount_expecting_the_floor_raised(&mut pool, "C 的根槽读不出");
    assert_eq!(
        abandoned_roots_unreadable, 0,
        "C 的根槽读不出：照根环表里记着的落点算，不读它的账"
    );
    assert_none_of_the_placements_is_free(&allocator_after_the_unmount, &recorded, "C 记着");
    images.hand_out(
        "卸载之后 C 引用的单元一个都没复用",
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

/// H1：挂上之后 C 的根槽读报错，会话覆盖写到准入推出第一串抬 F（推的那一刻 C 还在根环里）：准入算上限那一遍与抬 F 重算影子账那一遍
/// 读到 C 那一槽读坏都不拒（上限本来不看被抛弃根；影子账照记着的落点算），推过那一刻 C 记着的落点一槽都不空闲；之后再写几次也不拒，
/// 池级 checker 不红 I-7.4 与 I-4.8。推抬 F 的那一次覆盖写不拒（拒则是 `FloorRaiseFailedWhilePushingForSpace`，拒因
/// `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，点名 C 那一槽）。
pub fn run_the_scenario_where_session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
    images: &mut PoolCheckerImages,
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault =
        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let (mut pool, recorded) = mounted_while_the_abandoned_root_reads(&built, &fault);
    fault.is_active.set(true);
    assert_the_root_ring_slot_is_unreadable(&pool, &built.abandoned_root);
    overwrite_until_the_admission_pushes_the_first_floor_raise(
        &mut pool,
        &built.abandoned_root,
        "C 的根槽读不出",
    );
    assert_none_of_the_placements_is_free(&pool.session().allocator, &recorded, "C 记着");
    images.hand_out(
        "推过抬 F 之后 C 引用的单元一个都没复用",
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

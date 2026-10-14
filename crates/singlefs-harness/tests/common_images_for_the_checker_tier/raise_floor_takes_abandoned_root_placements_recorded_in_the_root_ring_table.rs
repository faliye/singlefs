//! harness 测试文件 `crates/singlefs-harness/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use crate::common_unwitnessed_publish::{
    content_of, device_bytes_of_the_384_slot_small_devices, devices_reading_back_all_zero_on,
    devices_reading_back_all_zero_on_until_lifted, fixed_geometry_of_the_384_slot_small_devices,
    image_of_devices_reading_back_all_zero, parameters_of_the_384_slot_small_devices, plain_device,
    plain_devices_on, pool_after_the_mount, record_one_publish_after_overwrites,
    root_is_in_the_ring, units_of_r_overwritten_on, DeviceOverASparseImage,
    DeviceReadingBackAllZeroOnRanges, PoolUnderTest, PublishRecordedAsR, RangesReadingBackAllZero,
    ReadBackAllZeroLift, RecordedPublish, OVERWRITE_BYTES,
};
use singlefs_core::address::{
    CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration, SlotNumber,
};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{PlacementOnDevice, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable_with_space_admission, raise_rollback_floor,
    raise_rollback_floor_to_the_admission_ceiling,
    ring_slots_recorded_as_holding_an_abandoned_root_by, roll_back_by_a_forward_publish, unmount,
    FloorRaiseStop, RaiseToTheAdmissionCeiling, RollbackTarget, ShadowLedger, Unmounted,
};
use singlefs_core::mounted_session::{
    MountedSession, NoSpaceAfterRaisingTheFloor, RootRingPushRefused, RootRingPushRehearsal,
    RootRingPushed, StuckBehindTheWideProtection, UserChange, UserChangeRefused,
    WaysOutOfTheWideProtection, WaysOutOfTheWideProtectionRefused, WideProtectionAtTheCeiling,
};
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::recovery::choose_system_configuration;
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::{FirstFile, PoolVersion, PublishError};
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
    SparseDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// C 那一次覆盖写的内容长度（攻方原型的 2500 字节）：与之前之后的覆盖写长度不同，读回哪一版看得出。
pub const CONTENT_BYTES_OF_THE_ABANDONED_VERSION: usize = 2500;

/// 挂着时 C 的账读不出之后，会话至多覆盖写几次去等准入推出第一串抬 F（攻方原型覆盖写到 60 次；改之前推抬 F 的那一次写被拒）。
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
/// 此前靠它豁免、现行账里已释放的落点只靠影子账这一遍补的隔离护着。只核起点槽时，补隔离只隔起点那一槽（跨度给成 1）照绿
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

/// 交 checker 档时核的两条：I-7.4（近 K 代块未被复用）与 I-4.8（近 K 代根校验和自洽），被抛弃根引用的单元没被复用就不红。
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
/// 改之前这一次卸载拒（`NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`）。
pub fn unmount_after_the_account_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
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
/// 改之前推抬 F 的那一次覆盖写拒（`FloorRaiseFailedWhilePushingForSpace`，拒因点名 C 的账）。
pub fn session_writes_after_the_account_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
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

/// H1（用户 2026-10-08 定「被抛弃根的槽不拒」）：挂上之后 C 的根槽读报错、账读得出。正常卸载那一串重算影子账读根环时 C 那一槽读坏、
/// 重读仍坏不拒，照根环表里记着的 C 的落点进影子账：卸载做成、这一遍不读 C 的账（计数 0）、C 记着的落点一槽都不空闲，
/// 池级 checker 不红 I-7.4 与 I-4.8。改之前那一槽算「这个进程知道住着根的槽」，重读仍坏就拒
/// （`NewerStateStillUnreadableAfterOneReread(RootRingSlotKnownToHoldARoot)`，攻方 `research/prompts/m3-batch10z-b-code-r1-opus-output.md` K1-1）。
pub fn unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_takes_its_recorded_placements_and_succeeds(
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
/// 池级 checker 不红 I-7.4 与 I-4.8。改之前推抬 F 的那一次覆盖写拒（`FloorRaiseFailedWhilePushingForSpace`，拒因
/// `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`，点名 C 那一槽）。
pub fn session_writes_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_raise_the_floor_with_its_recorded_placements(
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

/// H2 那块盘之前覆盖写几次（攻方 `research/prompts/m3-batch10z-b-code-r1-opus-output.md` K2-2 原样那一格 k = 5）。
pub const OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH: u64 = 5;

/// H2（用户 2026-10-08 定「用 C 那一圈记着的落点」）：`tests/common_unwitnessed_publish/mod.rs` 那块盘（覆盖写 5 次之后录一次覆盖写 R，
/// 崩在 R 的根槽落盘之后、轮换之前），可写挂载那一刻 R 的账（树表、映射根、分配记录树根）之外写出的单元读回全零：R 走读不成、择根退到 P，
/// R 的根槽与账读得出，按最新那条根（R 自己）的表
/// 判有效、根环表记成有效根、不带落点；写行之后 R 按现行那一版的表被抛弃。挂上之后 R 的账（树表、映射根、分配记录树根）读报错：
/// 正常卸载那一串照 C 那一圈在 R 那一槽记着的落点算（那一槽记的是有效根、txg 与 R 相同），不读 R 的账、不拒（计数 0），那些落点一槽都不空闲，
/// 池级 checker 不红 I-7.4 与 I-4.8。改之前表里没记着 R 就读盘：账与映射都读不出、现行账里没有凭据，卸载拒
/// （`NewerStateStillUnreadableAfterOneReread(AccountOfAnAbandonedRoot)`，两腿都量到的白拒）。
pub fn unmount_after_the_account_of_a_root_abandoned_by_the_mounts_own_row_turns_unreadable_takes_what_the_unapplied_records_named_on_its_ring_slot(
    images: &mut PoolCheckerImages,
) {
    let recorded = record_one_publish_after_overwrites(
        OVERWRITES_BEFORE_THE_UNWITNESSED_PUBLISH,
        PublishRecordedAsR::Overwrite,
    );
    let root_of_r = recorded.root();
    let account_locations_of_r: Vec<LocationEntry> = [
        root_of_r.tree_table,
        root_of_r.mapping_root,
        root_of_r.allocation_record_tree_root,
    ]
    .into_iter()
    .filter(|pointer| *pointer != NodePointer::empty_root())
    .flat_map(|pointer| pointer.locations)
    .collect();
    let unit_writes_of_r_outside_its_account: Vec<&singlefs_harness::memory_pool::RetainedWrite> =
        recorded
            .unit_writes()
            .into_iter()
            .filter(|write| {
                !account_locations_of_r.iter().any(|location| {
                    location.device == write.device
                        && location.slot.to_device_offset() == write.offset
                })
            })
            .collect();
    assert!(
        !unit_writes_of_r_outside_its_account.is_empty(),
        "R 那次覆盖写写出了账之外的单元（数据单元、两棵树的节点）"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges =
        RangesReadingBackAllZero::default().with_the_writes(&unit_writes_of_r_outside_its_account);
    let mut mount_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut mount_devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账之外写出的单元读回全零：可写挂载做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P"
    );
    let root_ring_table = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表");
    assert_eq!(
        root_ring_table.placements_recorded_for_the_abandoned_root(
            root_of_r.instance,
            root_of_r.checkpoint_txg
        ),
        None,
        "挂载那一刻 R 记成有效根：根环表里没记着它的被抛弃根落点"
    );
    assert!(
        root_ring_table
            .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
                root_of_r.instance,
                root_of_r.checkpoint_txg,
            )
            .is_some_and(|placements| !placements.is_empty()),
        "C 那一圈在 R 那一槽记着 R（R 的记录点名的、R 的账里的落点）"
    );
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&mount_devices, recorded.pool.device_bytes);
    let fault = ReadFaultOnChosenSlots::on_the_account_of(&root_of_r);
    let devices = devices_with_the_read_fault_on(&image_after_the_mount, &fault);
    let mut pool = pool_under_test_after_the_mount(
        &recorded.pool.parameters,
        recorded.pool.device_bytes,
        devices,
        mounted,
        recorded.pool.write_time_seconds + 10_000,
    );
    fault.is_active.set(true);
    assert_the_tree_table_of_the_abandoned_root_is_unreadable(&pool, &root_of_r);
    let (abandoned_roots_unreadable, allocator_after_the_unmount) =
        unmount_expecting_the_floor_raised(&mut pool, "R 的账读不出");
    assert_eq!(
        abandoned_roots_unreadable, 0,
        "R 的落点借 C 那一圈在 R 那一槽记着的那一份，这一遍不读它的账"
    );
    // 期望从录下来的 R 那次发布的写独立算（正推 K1-2 末段：取自被测方法时，借来的那一份被削成一半照绿）。
    assert_none_of_the_placements_is_free(
        &allocator_after_the_unmount,
        &placements_written_by(&recorded),
        "R 自己写出的单元",
    );
    images.hand_out(
        "卸载之后 R 引用的单元一个都没复用",
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

/// 丙那一形之前覆盖写几次（攻方 `research/prompts/m3-batch10z-d-code-r1-opus-output.md` 第一节原样那一格 k = 3）。
pub const OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_R: u64 = 3;

/// 丙那一形推过第一串抬 F 之后再顺序写几次（每次两个数据单元；攻方原样那一格 `Wss` 的两个 s）。
pub const SEQUENTIAL_WRITES_AFTER_THE_FIRST_FLOOR_RAISE: usize = 2;

/// 照 `record_one_publish_after_overwrites` 的录法录 R（崩在 R 的根槽落盘之后、系统配置轮换之前那一份写表），R 换成建一个 inode：
/// 文件的数据单元不动，R 与它之前那一版 P 共用它们（攻方 `research/prompts/m3-batch10z-d-code-r1-opus-model/src/opus10zd-probe/tests/opus10zd_h2_borrowed.rs`
/// 的 `record_r` 那一种 `n`）。
pub fn record_one_inode_creation_after_overwrites(overwrites_before: u64) -> RecordedPublish {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
    }
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    pool.create_one_inode().expect("录的那次建 inode 落得下");
    let root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_indices.len(),
        1,
        "录的那一次建 inode 只写一条根（准入够，没推抬 F）"
    );
    RecordedPublish {
        content: pool.content_of_the_current_version.clone(),
        pool,
        base,
        writes,
        root_index: root_indices[0],
        key: (root.instance, root.checkpoint_txg),
        previous_key: (previous_root.instance, previous_root.checkpoint_txg),
        previous_content,
    }
}

/// R 的账（树表、中央映射树根、分配记录树根；空指针不算）那几个单元的写，每块盘一份。
pub fn account_writes_of(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    let root_of_r = recorded.root();
    [
        root_of_r.tree_table,
        root_of_r.mapping_root,
        root_of_r.allocation_record_tree_root,
    ]
    .into_iter()
    .filter(|pointer| *pointer != NodePointer::empty_root())
    .flat_map(|pointer| recorded.unit_writes_at(&pointer))
    .collect()
}

/// R 引用、R 自己没写的文件数据单元的落点（R 从所选那一版 P 继承的）：录下来的会话里 R 那一版的文件数据指针，
/// 每条位置条目里 R 一份都没写过的那几个。从录下来的会话与写表算，不经被测的方法。
pub fn data_unit_placements_r_shares_with_the_previous_version(
    recorded: &RecordedPublish,
) -> Vec<PlacementOnDevice> {
    let written_by_r = placements_written_by(recorded);
    recorded
        .pool
        .current_file_version()
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .map(|location| PlacementOnDevice {
            device: location.device,
            slot: location.slot,
            span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
        })
        .filter(|placement| {
            !written_by_r
                .iter()
                .any(|written| written.device == placement.device && written.slot == placement.slot)
        })
        .collect()
}

/// 丙那一形挂上之后，R 的账那几段还读不读回全零。
#[derive(Clone, Copy, Debug)]
pub enum AccountOfRAfterTheMount {
    /// 挂上之后撤掉读回全零（攻方 a2、a3 那几组）。
    ReadBackAllZeroLifted,
    /// 挂上之后照旧读回全零，写到哪一段那一段才照读（攻方 a4「故障不撤」）。
    StillReadingBackAllZero,
}

/// 丙那一形（攻方 `research/prompts/m3-batch10z-d-code-r1-opus-output.md` 第一节，原样那一格 k = 3、R 建一个 inode、故障 `acct`、动作 `Wss`）：
/// 第一个文件之后覆盖写 3 次得 P，录一次建 inode 得 R（R 与 P 共用文件的数据单元），崩在 R 的根槽落盘之后、轮换之前；可写挂载那一刻
/// R 的账（树表、中央映射树根、分配记录树根）读回全零：R 走读不成、择根退到 P，R 的记录齐，C 那两份账都不加，R 的根槽读得出、记成有效根，
/// 写行之后 R 按现行那一版的表被抛弃。挂上之后照 `after_the_mount` 撤或不撤故障；会话覆盖写到准入推出第一串抬 F（覆盖写释放了 R 与 P 共用的
/// 数据单元，推的那一刻 R 还在根环里），再顺序写两次。每一步之后池级 checker 不红 I-7.4（近 K 代块未被复用）与 I-4.8，R 写的单元一份都没被写掉；
/// 推过抬 F 那一刻 R 共用的数据单元与 R 写的单元覆盖的槽一槽都不空闲（期望从录下来的会话与写表独立算）；最后核挂载那一刻 C 那一槽借得到
/// R 共用的那几个数据单元（丙并上的那一截）。只有 H2、没有丙时抬 F 只借到 R 的记录点名的落点，共用的数据单元回收、被顺序写发出去，I-7.4 红。
pub fn the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor(
    images: &mut PoolCheckerImages,
    after_the_mount: AccountOfRAfterTheMount,
) {
    let recorded = record_one_inode_creation_after_overwrites(
        OVERWRITES_BEFORE_THE_INODE_CREATION_RECORDED_AS_R,
    );
    let root_of_r = recorded.root();
    let shared_with_the_previous_version =
        data_unit_placements_r_shares_with_the_previous_version(&recorded);
    assert!(
        !shared_with_the_previous_version.is_empty(),
        "R 建一个 inode、不写文件的数据单元：它与 P 共用文件的数据单元"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded));
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P"
    );
    let root_ring_table = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表");
    assert_eq!(
        root_ring_table.placements_recorded_for_the_abandoned_root(
            root_of_r.instance,
            root_of_r.checkpoint_txg
        ),
        None,
        "挂载那一刻 R 记成有效根：根环表里没记着它的被抛弃根落点（抬 F 那一串借 C 那一圈，H2）"
    );
    let borrowed_at_the_mount = root_ring_table
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着 R 的记录点名的落点");
    match after_the_mount {
        AccountOfRAfterTheMount::ReadBackAllZeroLifted => lift.lift(),
        AccountOfRAfterTheMount::StillReadingBackAllZero => {}
    }
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content_of_the_selected_version = recorded
        .pool
        .content_of_each_root
        .get(&(
            output.effective_root.instance,
            output.effective_root.checkpoint_txg,
        ))
        .cloned()
        .expect("挂载落到的那一版是录的池写过的一版");
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    // 同一个说明（「覆盖写之后」）在循环里交好几张，名字带上这是第几张，一个场景里不重名
    let mut images_handed_out_after_the_mount = 0_usize;
    let mut hand_out_expecting_nothing_of_r_reused =
        |pool_after_the_step: &PoolUnderTest<_>, after_what: &str| {
            images_handed_out_after_the_mount += 1;
            let image = pool_after_the_step.image();
            if root_is_in_the_ring(&image, recorded.key) {
                images.hand_out(
                &format!("第 {images_handed_out_after_the_mount} 张，{after_what}（{after_the_mount:?}），R 的根还在根环里：R 引用的单元一个都没复用"),
                || image.clone(),
                PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
            );
                assert_eq!(
                units_of_r_overwritten_on(&recorded, &image),
                Vec::<(DeviceIdentity, u64)>::new(),
                "{after_what}（{after_the_mount:?}），R 的根还在根环里：R 写的单元一份都没被写掉"
            );
            }
        };
    let mut overwrites_before_the_first_floor_raise = 0;
    // 迭代上界 OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST；跨轮不带状态（计数之外）；出口：推出抬 F 的那一次 break，被拒就当场判红（panic）。
    loop {
        assert!(
            overwrites_before_the_first_floor_raise < OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST,
            "{OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST} 次覆盖写之内准入推出抬 F（384 槽小盘）"
        );
        let root_of_r_was_in_the_ring = root_is_in_the_ring(&pool.image(), recorded.key);
        let published = pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!(
                "第 {} 次覆盖写（{after_the_mount:?}）被拒：{refusal:?}",
                overwrites_before_the_first_floor_raise + 1
            )
        });
        overwrites_before_the_first_floor_raise += 1;
        hand_out_expecting_nothing_of_r_reused(&pool, "覆盖写之后");
        if !published.floor_raises.is_empty() {
            assert!(
                root_of_r_was_in_the_ring,
                "准入推抬 F 的那一刻 R 还在根环里（不然这一串里没有被抛弃根要借 C，没走到要判的那一处）"
            );
            break;
        }
    }
    // 推过抬 F 那一刻的空闲槽先记下、顺序写之后再判：池级 checker 那一道先判（回收出来的槽被顺序写发出去才红得出），这一道接着判。
    let free_among_the_shared_after_the_raise =
        free_slots_covered_by(&pool.session().allocator, &shared_with_the_previous_version);
    let free_among_the_written_after_the_raise =
        free_slots_covered_by(&pool.session().allocator, &placements_written_by(&recorded));
    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_THE_FIRST_FLOOR_RAISE {
        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
            .unwrap_or_else(|refusal| {
                panic!(
                    "推过抬 F 之后第 {sequential_write_number} 次顺序写（{after_the_mount:?}）被拒：{refusal:?}"
                )
            });
        hand_out_expecting_nothing_of_r_reused(&pool, "推过抬 F 之后顺序写");
    }
    assert!(
        free_among_the_shared_after_the_raise.is_empty(),
        "推过抬 F 那一刻（{after_the_mount:?}）R 与 P 共用的数据单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_shared_after_the_raise:?}"
    );
    assert!(
        free_among_the_written_after_the_raise.is_empty(),
        "推过抬 F 那一刻（{after_the_mount:?}）R 自己写出的单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_written_after_the_raise:?}"
    );
    let shared_not_borrowed: Vec<&PlacementOnDevice> = shared_with_the_previous_version
        .iter()
        .filter(|shared| {
            !borrowed_at_the_mount
                .iter()
                .any(|borrowed| borrowed.device == shared.device && borrowed.slot == shared.slot)
        })
        .collect();
    assert!(
        shared_not_borrowed.is_empty(),
        "挂载那一刻 C 那一槽并上了现行账里仍分配着的落点（丙）：R 与 P 共用的数据单元都在借得到的那一份里，缺 {shared_not_borrowed:?}"
    );
}

/// 丙（用户 2026-10-08 定「改成丙」）：挂上之后撤掉 R 的账那几段的读回全零（攻方 a2、a3 那几组的取法），流程与断言见
/// [`the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor`]。
/// 判别力：撤掉丙（变异「第10z批第九块 丙 较新可读根账读不出而记录齐时 C 那一槽不并现行账里仍分配着的落点」）时 I-7.4 红。
pub fn units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_once_its_account_reads_again(
    images: &mut PoolCheckerImages,
) {
    the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor(images,
        AccountOfRAfterTheMount::ReadBackAllZeroLifted,
    );
}

/// 丙：挂上之后 R 的账那几段照旧读回全零（攻方 a4「故障不撤」），流程与断言同上。
/// 判别力：同上那一条变异时 I-7.4 红。
pub fn units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor_while_its_account_still_reads_back_zero(
    images: &mut PoolCheckerImages,
) {
    the_units_a_root_abandoned_by_the_mounts_own_row_shares_with_the_selected_version_stay_through_writes_raising_the_floor(images,
        AccountOfRAfterTheMount::StillReadingBackAllZero,
    );
}

/// 丙并中间那一形之前覆盖写几次（攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-output.md` 第一节原样那一格 k = 3）。
pub const OVERWRITES_BEFORE_THE_PUBLISHES_RECORDED_AS_Q_AND_R: u64 = 3;

/// 丙并中间那一形挂上之后，会话至多覆盖写几次去等写掉 R 那一槽（384 槽小盘一圈根环 24 次发布；攻方原样那一格第 20 步写掉）。
pub const OVERWRITES_UNTIL_THE_RING_SLOT_OF_R_IS_WRITTEN_AT_MOST: usize = 40;

/// 录下来的 Q、R 两次发布：Q 之前那一刻（所选那一版 P）的盘面、Q 与 R 两次发布的全部写（按落盘次序）、两条根槽写在写表里的下标、
/// P、Q、R 的 (实例, txg)，加起池那一份（会话停在 R 那一版）。
pub struct RecordedQAndR {
    pub pool: PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>,
    pub base: MemoryPool,
    pub writes: Vec<RetainedWrite>,
    pub root_index_of_q: usize,
    pub root_index_of_r: usize,
    pub key_of_p: (InstanceGeneration, CheckpointTxg),
    pub key_of_q: (InstanceGeneration, CheckpointTxg),
    pub key_of_r: (InstanceGeneration, CheckpointTxg),
}

/// 攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-model/src/opus10zghi-probe/tests/opus_k21_chain.rs` 的 `record_chain`（q = `s`、r = `n`）：
/// 起池（384 槽那一档、第一个文件），覆盖写 `overwrites_before` 次得 P，再录 Q（顺序写三个数据单元，三条记录）与 R（建一个 inode，
/// 文件的数据单元与 Q 共用）。
pub fn record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites(
    overwrites_before: u64,
) -> RecordedQAndR {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
    }
    let key_of = |pool_being_recorded: &PoolUnderTest<RecordingBlockDevice<SparseBlockDevice>>| {
        let root = pool_being_recorded.session().current.root();
        (root.instance, root.checkpoint_txg)
    };
    let key_of_p = key_of(&pool);
    let base = pool.image();
    let operations_before_q = stream.operation_count();
    pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity() + 1000)
        .expect("录的那次顺序写（Q）落得下");
    let key_of_q = key_of(&pool);
    pool.create_one_inode()
        .expect("录的那次建 inode（R）落得下");
    let key_of_r = key_of(&pool);
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_q..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_indices.len(),
        2,
        "录的 Q、R 两次发布各写一条根（准入够，没推抬 F）"
    );
    RecordedQAndR {
        pool,
        base,
        writes,
        root_index_of_q: root_indices[0],
        root_index_of_r: root_indices[1],
        key_of_p,
        key_of_q,
        key_of_r,
    }
}

impl RecordedQAndR {
    /// 崩在 R 的根槽落盘之后、R 那次系统配置轮换之前（Q 那次轮换已持久）。
    pub fn crash_image_with_the_root_of_r_persisted_and_its_rotation_lost(&self) -> MemoryPool {
        let mut image = self.base.clone();
        image.apply_writes(&self.writes[..=self.root_index_of_r]);
        image
    }

    pub fn root_of_r(&self) -> RootRecord {
        RootRecord::parse_slot(
            self.writes[self.root_index_of_r]
                .bytes()
                .expect("根槽写带字节"),
            &self.pool.parameters.filesystem_identifier,
        )
        .expect("R 的根自证")
    }

    /// Q 写出的单元写（每块盘一份）。
    pub fn unit_writes_of_q(&self) -> Vec<&RetainedWrite> {
        self.writes[..self.root_index_of_q]
            .iter()
            .filter(|write| write.kind == StepKind::UnitWrite)
            .collect()
    }

    /// 可写挂载那一刻读回全零的几段（攻方故障组 `cfgq,qroot,qlast,racct`）：每块盘上 Q 那次轮换写的系统配置槽、Q 的根槽、
    /// Q 带末条标志的那条记录、R 写的树表 / 中央映射树根 / 分配记录树根那几份。
    pub fn ranges_reading_back_all_zero_at_the_mount(
        &self,
        crash_image: &MemoryPool,
    ) -> RangesReadingBackAllZero {
        let root_of_r = self.root_of_r();
        let account_pointers_of_r = [
            root_of_r.tree_table,
            root_of_r.mapping_root,
            root_of_r.allocation_record_tree_root,
        ];
        let writes_in_between_the_two_roots =
            &self.writes[self.root_index_of_q..self.root_index_of_r];
        let rotation_of_q: Vec<&RetainedWrite> = writes_in_between_the_two_roots
            .iter()
            .filter(|write| write.kind == StepKind::SystemConfigurationSlot)
            .collect();
        assert!(!rotation_of_q.is_empty(), "Q 那次轮换写了系统配置槽");
        let account_of_r: Vec<&RetainedWrite> = writes_in_between_the_two_roots
            .iter()
            .filter(|write| write.kind == StepKind::UnitWrite)
            .filter(|write| {
                account_pointers_of_r.iter().any(|pointer| {
                    *pointer != NodePointer::empty_root()
                        && pointer.locations.iter().any(|location| {
                            location.device == write.device
                                && location.slot.to_device_offset() == write.offset
                        })
                })
            })
            .collect();
        assert!(!account_of_r.is_empty(), "R 自己写了它的账那几个单元");
        RangesReadingBackAllZero::default()
            .with_the_writes(&rotation_of_q)
            .with_the_writes(&[&self.writes[self.root_index_of_q]])
            .with_the_last_record_of_the_publish(crash_image, self.key_of_q)
            .with_the_writes(&account_of_r)
    }
}

/// `placements` 里有没有一个落点罩住这次单元写的起点（同一块盘、起点槽到跨度之内）。
pub fn some_placement_covers(placements: &[PlacementOnDevice], write: &RetainedWrite) -> bool {
    placements.iter().any(|placement| {
        let start_in_bytes = placement.slot.to_device_offset().0;
        placement.device == write.device
            && start_in_bytes <= write.offset.0
            && write.offset.0 < start_in_bytes + placement.span * singlefs_format::SLOT_BYTES
    })
}

/// 丙并中间（攻方 `research/prompts/m3-batch10z-ghi-code-r1-opus-output.md` 第一节，原样那一格 k = 3、Q 顺序写、R 建 inode、
/// 故障 `cfgq,qroot,qlast,racct`、动作全是覆盖写；判决 `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 攻方一形）：
/// 第一个文件之后覆盖写 3 次得 P，录 Q（顺序写）与 R（建 inode，数据单元与 Q 共用），崩在 R 的根槽落盘之后、R 那次轮换之前；
/// 可写挂载那一刻 Q 轮换写的系统配置槽、Q 的根槽、Q 的末条记录、R 的账读回全零：择根退到 P，R 的记录齐、账读不出，走丙。
/// C 里 R 那一槽并上 (P, R) 之间 Q 那一槽记着的落点，Q 写的单元全在里面。挂上之后撤故障，会话覆盖写到写掉 R 那一槽：
/// 每一步之后、每一步最后一条根槽写之前那个段边界状态（单元与记录都落了、这一步的根没落：攻方红的那一态）池级 checker 都不红
/// I-7.4（近 K 代块未被复用）与 I-4.8；其间有一步 Q 已离开根环而 R 还在（这一形的窗口确实走到了）。
/// 判别力：撤掉丙并中间（变异「第10z批第十块 丙并中间 走丙那一支的根那一槽不并所选那一版与它之间那几槽」）时，写掉 Q 那一槽之后、
/// 写掉 R 那一槽那一步根落之前那一态 I-7.4 红，违例的根是 R；挂载那一刻 C 那一槽缺 Q 的单元那一句先红。
pub fn units_a_root_inherits_from_a_publish_between_it_and_the_selected_version_stay_until_its_ring_slot_is_written(
    images: &mut PoolCheckerImages,
) {
    let recorded = record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites(
        OVERWRITES_BEFORE_THE_PUBLISHES_RECORDED_AS_Q_AND_R,
    );
    let crash_image = recorded.crash_image_with_the_root_of_r_persisted_and_its_rotation_lost();
    let ranges = recorded.ranges_reading_back_all_zero_at_the_mount(&crash_image);
    let mut faulted_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut faulted_devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("Q 的根与末条记录、R 的账读回全零：可写挂载退到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.key_of_p,
        "R 走读不成、Q 的根读不出：择根退到 P（阳性对照：故障打到了）"
    );
    let borrowed_for_r = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            recorded.key_of_r.0,
            recorded.key_of_r.1,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    let units_of_q_not_in_the_ring_slot_of_r: Vec<&RetainedWrite> = recorded
        .unit_writes_of_q()
        .into_iter()
        .filter(|write| !some_placement_covers(&borrowed_for_r, write))
        .collect();
    assert!(
        units_of_q_not_in_the_ring_slot_of_r.is_empty(),
        "挂载那一刻 C 里 R 那一槽并上了 Q 那一槽记着的落点（丙并中间）：Q 写的单元都在里面，缺 {} 份：{units_of_q_not_in_the_ring_slot_of_r:?}",
        units_of_q_not_in_the_ring_slot_of_r.len()
    );
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&faulted_devices, recorded.pool.device_bytes);
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content_of_the_selected_version = recorded
        .pool
        .content_of_each_root
        .get(&recorded.key_of_p)
        .cloned()
        .expect("P 是录的池写过的一版");
    let mut content_of_each_root = recorded.pool.content_of_each_root.clone();
    for version in std::iter::once(&output.row_publish).chain(&output.warm_up_publishes) {
        let root = version.root();
        content_of_each_root.insert(
            (root.instance, root.checkpoint_txg),
            content_of_the_selected_version.clone(),
        );
    }
    let session_stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices: plain_devices_on(&image_after_the_mount)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        device,
                        session_stream.clone(),
                    ),
                )
            })
            .collect(),
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root,
    };
    let mut q_left_the_ring_while_r_was_in_it = false;
    let mut overwrites = 0;
    // 迭代上界 OVERWRITES_UNTIL_THE_RING_SLOT_OF_R_IS_WRITTEN_AT_MOST；跨轮带着「Q 先离开根环、R 还在」那一个判断；
    // 出口：R 离开根环的那一步判完就 break，被拒就当场判红（panic）。
    loop {
        assert!(
            overwrites < OVERWRITES_UNTIL_THE_RING_SLOT_OF_R_IS_WRITTEN_AT_MOST,
            "{OVERWRITES_UNTIL_THE_RING_SLOT_OF_R_IS_WRITTEN_AT_MOST} 次覆盖写之内写掉 R 那一槽（384 槽小盘一圈根环 24 次发布）"
        );
        let image_before_the_step = pool.image();
        let operations_before_the_step = session_stream.operation_count();
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("挂上之后第 {} 次覆盖写被拒：{refusal:?}", overwrites + 1)
        });
        overwrites += 1;
        let (writes_of_the_step, _segments, _stream_indexes) =
            writes_and_segments_with_stream_indexes(
                &session_stream.retained_operations()[operations_before_the_step..],
                &fixed_geometry_of_the_384_slot_small_devices(),
            );
        let last_root_of_the_step = writes_of_the_step
            .iter()
            .rposition(|write| write.kind == StepKind::RootRecordFua)
            .expect("一次覆盖写至少写一条根");
        let mut state_before_the_last_root_of_the_step = image_before_the_step;
        state_before_the_last_root_of_the_step
            .apply_writes(&writes_of_the_step[..last_root_of_the_step]);
        images.hand_out(
            &format!(
                "第 {overwrites} 次覆盖写最后一条根槽写之前那一态：被抛弃根引用的单元一个都没复用"
            ),
            || state_before_the_last_root_of_the_step.clone(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
        let image_after_the_step = pool.image();
        images.hand_out(
            &format!("第 {overwrites} 次覆盖写之后：被抛弃根引用的单元一个都没复用"),
            || image_after_the_step.clone(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
        let r_is_in_the_ring = root_is_in_the_ring(&image_after_the_step, recorded.key_of_r);
        if r_is_in_the_ring && !root_is_in_the_ring(&image_after_the_step, recorded.key_of_q) {
            q_left_the_ring_while_r_was_in_it = true;
        }
        if !r_is_in_the_ring {
            break;
        }
    }
    assert!(
        q_left_the_ring_while_r_was_in_it,
        "有一步之后 Q 已离开根环而 R 还在（这一形的窗口走到了：Q 那一槽先写掉、解开，R 那一槽晚一个 txg）"
    );
}

/// 正推 `research/prompts/m3-batch10z-ghi-code-r1-sonnet-model/probe_rollback_revived.tail.rs` 那一形之前覆盖写几次（探针原样 4 次）。
pub const OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R: u64 = 4;

/// 回退复活那一形推过第一串抬 F 之后再顺序写几次（每次两个数据单元；探针原样 3 次）。
pub const SEQUENTIAL_WRITES_AFTER_THE_FLOOR_RAISE_OVER_THE_ROLLBACK: usize = 3;

/// 录下来的一次管理员回退 R（[`record_one_rollback_to_the_first_file_after_overwrites`]）。
pub struct RecordedRollback {
    pub recorded: RecordedPublish,
    /// R 复活的数据单元的落点：R_old 的文件数据指针里 P 不引用的那几条位置条目（跨度按数据单元宽折成槽数）。
    pub revived: Vec<PlacementOnDevice>,
    /// 回退目标 R_old（第一个文件那一版）的根。
    pub rollback_target: RootRecord,
    /// R_old 那一版的账里仍分配着的（盘、起点槽）：起池之后、覆盖写之前会话分配器里的记录（R_old 那次发布写下的账）。
    pub allocated_in_the_account_of_the_rollback_target: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 正推探针的 `record_one_rollback_after_overwrites`：起池（384 槽那一档、第一个文件，那一版是 R_old），覆盖写 `overwrites_before` 次得 P，
/// 再经会话做一次管理员回退到 R_old、录这一次发布得 R。交回录下来的 R、R 复活的单元的落点、R_old 的根与它账里仍分配着的落点
/// （都从录下来的会话算，不经被测的方法）。
pub fn record_one_rollback_to_the_first_file_after_overwrites(
    overwrites_before: u64,
) -> RecordedRollback {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    let root_of_the_first_file = *pool.session().current.root();
    let data_pointers_of_the_first_file = pool.current_file_version().data_pointers.clone();
    let allocated_in_the_account_of_the_rollback_target: Vec<(DeviceIdentity, SlotNumber)> = pool
        .session()
        .allocator
        .records()
        .iter()
        .filter(|record| !record.is_released)
        .map(|record| (record.device, record.slot))
        .collect();
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
    }
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let locations_referenced_by_p: Vec<(DeviceIdentity, SlotNumber)> = pool
        .current_file_version()
        .data_pointers
        .iter()
        .flat_map(|pointer| pointer.locations)
        .map(|location| (location.device, location.slot))
        .collect();
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    let mut session = pool.session.take().expect("会话");
    {
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("第一个文件之后现行那一版带文件");
        };
        roll_back_by_a_forward_publish(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            RollbackTarget {
                instance: root_of_the_first_file.instance,
                checkpoint_txg: root_of_the_first_file.checkpoint_txg,
            },
        )
        .expect("回退到第一个文件那一版做得成（它在根环里、按实例表有效）");
    }
    pool.session = Some(session);
    let root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indices.len(), 1, "回退那次发布只写一条根");
    let revived: Vec<PlacementOnDevice> = data_pointers_of_the_first_file
        .iter()
        .flat_map(|pointer| pointer.locations)
        .filter(|location| !locations_referenced_by_p.contains(&(location.device, location.slot)))
        .map(|location| PlacementOnDevice {
            device: location.device,
            slot: location.slot,
            span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
        })
        .collect();
    RecordedRollback {
        recorded: RecordedPublish {
            content: previous_content.clone(),
            pool,
            base,
            writes,
            root_index: root_indices[0],
            key: (root.instance, root.checkpoint_txg),
            previous_key: (previous_root.instance, previous_root.checkpoint_txg),
            previous_content,
        },
        revived,
        rollback_target: root_of_the_first_file,
        allocated_in_the_account_of_the_rollback_target,
    }
}

/// 回退复活（正推 `research/prompts/m3-batch10z-ghi-code-r1-sonnet-output.md` K2-1、探针 `probe_rollback_revived.tail.rs`；判决
/// `research/prompts/m3-batch10z-ghi-code-r1-main-verification.md` 第二节 K2-1 正推一形）：第一个文件之后覆盖写 4 次得 P，经会话管理员回退到
/// 第一个文件那一版（R_old）、录这一次发布得 R，崩在 R 的根槽落盘之后、轮换之前；可写挂载那一刻 R 的账（树表、中央映射树根、分配记录树根）
/// 读回全零而记录齐：择根退到 P，走丙。R 复活的数据单元在 P 的账里是已释放，丙那一截滤掉它们；补法认出 R 是回退（记录点名的谱系片里有
/// rollback_txg 等于 R 的那条谱系记录），把 R_old 账里仍分配、P 的账里已释放的落点并进 C 里 R 那一槽：复活的落点都在里面。
/// 挂上之后撤故障，会话覆盖写到准入推出第一串抬 F（推的那一刻 R 还在根环里），推过那一刻复活的落点一槽都不空闲；再顺序写 3 次，
/// 每一步之后池级 checker 不红 I-7.4 与 I-4.8。判别力：撤掉回退复活那一截（变异「第10z批第十块 回退复活 丙那一支不并回退目标账里仍分配、
/// 现行账里已释放的落点」）时 C 那一槽缺复活的落点那一句先红；去掉那一句，推过抬 F 复活的槽空闲、顺序写之后 I-7.4 红（正推探针原样）。
pub fn units_a_rollback_publish_revived_stay_through_writes_raising_the_floor_while_its_account_reads_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
) {
    let RecordedRollback {
        recorded, revived, ..
    } = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
    );
    assert!(
        !revived.is_empty(),
        "第一个文件那一版的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
    );
    let root_of_r = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let ranges = RangesReadingBackAllZero::default().with_the_writes(&account_writes_of(&recorded));
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let borrowed_for_r = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    let revived_not_borrowed: Vec<&PlacementOnDevice> = revived
        .iter()
        .filter(|revived_placement| {
            !borrowed_for_r.iter().any(|borrowed| {
                borrowed.device == revived_placement.device
                    && borrowed.slot == revived_placement.slot
            })
        })
        .collect();
    assert!(
        revived_not_borrowed.is_empty(),
        "挂载那一刻 C 里 R 那一槽并上了回退目标账里仍分配、P 的账里已释放的落点（回退复活）：复活的都在里面，缺 {revived_not_borrowed:?}"
    );
    lift.lift();
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content_of_the_selected_version = recorded
        .pool
        .content_of_each_root
        .get(&(
            output.effective_root.instance,
            output.effective_root.checkpoint_txg,
        ))
        .cloned()
        .expect("挂载落到的那一版是录的池写过的一版");
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    // 同一个说明（「覆盖写之后」）在循环里交好几张，名字带上这是第几张，一个场景里不重名
    let mut images_handed_out_after_the_mount = 0_usize;
    let mut hand_out_expecting_nothing_of_r_reused =
        |pool_after_the_step: &PoolUnderTest<_>, after_what: &str| {
            images_handed_out_after_the_mount += 1;
            images.hand_out(
                &format!("第 {images_handed_out_after_the_mount} 张，{after_what}：被抛弃根引用的单元一个都没复用"),
                || pool_after_the_step.image(),
                PoolCheckerExpectation::only_the_named_invariants()
                    .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
            );
        };
    let mut overwrites_before_the_first_floor_raise = 0;
    // 迭代上界 OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST；跨轮不带状态（计数之外）；出口：推出抬 F 的那一次 break，被拒就当场判红（panic）。
    loop {
        assert!(
            overwrites_before_the_first_floor_raise < OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST,
            "{OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST} 次覆盖写之内准入推出抬 F（384 槽小盘）"
        );
        let root_of_r_was_in_the_ring = root_is_in_the_ring(&pool.image(), recorded.key);
        let published = pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!(
                "第 {} 次覆盖写被拒：{refusal:?}",
                overwrites_before_the_first_floor_raise + 1
            )
        });
        overwrites_before_the_first_floor_raise += 1;
        hand_out_expecting_nothing_of_r_reused(&pool, "覆盖写之后");
        if !published.floor_raises.is_empty() {
            assert!(
                root_of_r_was_in_the_ring,
                "准入推抬 F 的那一刻 R 还在根环里（不然这一串里没有被抛弃根要借 C，没走到要判的那一处）"
            );
            break;
        }
    }
    let free_among_the_revived_after_the_raise =
        free_slots_covered_by(&pool.session().allocator, &revived);
    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_THE_FLOOR_RAISE_OVER_THE_ROLLBACK {
        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
            .unwrap_or_else(|refusal| {
                panic!("推过抬 F 之后第 {sequential_write_number} 次顺序写被拒：{refusal:?}")
            });
        hand_out_expecting_nothing_of_r_reused(&pool, "推过抬 F 之后顺序写");
    }
    assert!(
        free_among_the_revived_after_the_raise.is_empty(),
        "推过抬 F 那一刻 R 复活的数据单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_revived_after_the_raise:?}"
    );
}

/// 认不出时宽护（tri，用户 2026-10-08 弹窗定「认不出时宽护 tri」，判决 `research/prompts/m3-batch10z-jk-code-r1-main-verification.md` 第五节；
/// 判法照正推 tri 臂 `research/prompts/m3-batch10z-jk-code-r1-sonnet-model/src/probe-arms-on-mount.diff`，第 10z 批第十二块实现员定细节，被攻过零轮）
/// 那四形共用的走法：回退复活那一形（覆盖写 4 次得 P、回退到第一个文件那一版录 R），可写挂载那一刻 `ranges` 读回全零（R 的账另加一处，
/// 让认回退或算复活集那一步走不成）：择根退到 P，走丙。挂上之后撤故障，会话覆盖写到准入推出第一串抬 F（推的那一刻 R 还在根环里），
/// 再顺序写 3 次，每一步之后池级 checker 不红 I-7.4 与 I-4.8；全程过了之后再核 C 里 R 那一槽有复活的全部落点、推过抬 F 那一刻复活的槽不空闲。
/// 次序是故意的：撤掉宽护（这几形照丙原样交空）时先红的是写到半途的 I-7.4（复活的单元被回收、写掉），不是借到那一句。
pub fn units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
    recorded_rollback: &RecordedRollback,
    ranges: &RangesReadingBackAllZero,
) {
    let recorded = &recorded_rollback.recorded;
    let revived = &recorded_rollback.revived;
    assert!(
        !revived.is_empty(),
        "第一个文件那一版的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
    );
    let root_of_r = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, ranges, &lift);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的账读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let borrowed_for_r = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    lift.lift();
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content_of_the_selected_version = recorded
        .pool
        .content_of_each_root
        .get(&(
            output.effective_root.instance,
            output.effective_root.checkpoint_txg,
        ))
        .cloned()
        .expect("挂载落到的那一版是录的池写过的一版");
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    // 同一个说明（「覆盖写之后」）在循环里交好几张，名字带上这是第几张，一个场景里不重名
    let mut images_handed_out_after_the_mount = 0_usize;
    let mut hand_out_expecting_nothing_of_r_reused =
        |pool_after_the_step: &PoolUnderTest<_>, after_what: &str| {
            images_handed_out_after_the_mount += 1;
            images.hand_out(
                &format!("第 {images_handed_out_after_the_mount} 张，{after_what}：被抛弃根引用的单元一个都没复用"),
                || pool_after_the_step.image(),
                PoolCheckerExpectation::only_the_named_invariants()
                    .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
            );
        };
    let mut overwrites_before_the_first_floor_raise = 0;
    // 迭代上界 OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST；跨轮不带状态（计数之外）；出口：推出抬 F 的那一次 break，被拒就当场判红（panic）。
    loop {
        assert!(
            overwrites_before_the_first_floor_raise < OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST,
            "{OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST} 次覆盖写之内准入推出抬 F（384 槽小盘）"
        );
        let root_of_r_was_in_the_ring = root_is_in_the_ring(&pool.image(), recorded.key);
        let published = pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!(
                "第 {} 次覆盖写被拒：{refusal:?}",
                overwrites_before_the_first_floor_raise + 1
            )
        });
        overwrites_before_the_first_floor_raise += 1;
        hand_out_expecting_nothing_of_r_reused(&pool, "覆盖写之后");
        if !published.floor_raises.is_empty() {
            assert!(
                root_of_r_was_in_the_ring,
                "准入推抬 F 的那一刻 R 还在根环里（不然这一串里没有被抛弃根要借 C，没走到要判的那一处）"
            );
            break;
        }
    }
    let free_among_the_revived_after_the_raise =
        free_slots_covered_by(&pool.session().allocator, revived);
    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_THE_FLOOR_RAISE_OVER_THE_ROLLBACK {
        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
            .unwrap_or_else(|refusal| {
                panic!("推过抬 F 之后第 {sequential_write_number} 次顺序写被拒：{refusal:?}")
            });
        hand_out_expecting_nothing_of_r_reused(&pool, "推过抬 F 之后顺序写");
    }
    let revived_not_borrowed: Vec<&PlacementOnDevice> = revived
        .iter()
        .filter(|revived_placement| {
            !borrowed_for_r.iter().any(|borrowed| {
                borrowed.device == revived_placement.device
                    && borrowed.slot == revived_placement.slot
            })
        })
        .collect();
    assert!(
        revived_not_borrowed.is_empty(),
        "挂载那一刻 C 里 R 那一槽退到现行账里全部已释放的落点（宽护）：复活的都在里面，缺 {revived_not_borrowed:?}"
    );
    assert!(
        free_among_the_revived_after_the_raise.is_empty(),
        "推过抬 F 那一刻 R 复活的数据单元覆盖的槽一槽都不空闲：空闲的（盘、槽）{free_among_the_revived_after_the_raise:?}"
    );
}

/// R 写出的谱系片（打包记录类型 5 的打包单元）的写：从录下来的写的字节解，不经被测的方法。
pub fn lineage_unit_writes_of(recorded: &RecordedPublish) -> Vec<&RetainedWrite> {
    recorded
        .unit_writes()
        .into_iter()
        .filter(|write| {
            write
                .bytes()
                .and_then(|bytes| singlefs_core::unit::parse_packed_unit(bytes).ok())
                .is_some_and(|packed| {
                    packed.identity.record_type == singlefs_core::unit::PACKED_TYPE_LINEAGE
                })
        })
        .collect()
}

/// `image` 的根环里 (实例, txg) 为 `key`、自证过的那条根（盘上那一份：带着它的树表、分配记录树根等指针）。
///
/// # Panics
/// `image` 上择不到系统配置，或根环里没有自证过的这条根。
pub fn root_read_from_the_ring_of(
    image: &MemoryPool,
    key: (InstanceGeneration, CheckpointTxg),
) -> RootRecord {
    let configuration = choose_system_configuration(image).expect("镜像上择得到系统配置");
    let sizes = &configuration.immutable.sizes;
    singlefs_core::recovery::every_root_ring_slot(sizes)
        .into_iter()
        .find_map(|ring_slot| {
            if let singlefs_core::recovery::RootRingSlotReading::SelfVerified(root) =
                singlefs_core::recovery::read_root_ring_slot(
                    image,
                    &configuration.immutable.region_devices,
                    sizes,
                    &configuration.immutable.filesystem_identifier,
                    ring_slot,
                )
            {
                ((root.instance, root.checkpoint_txg) == key).then_some(root)
            } else {
                None
            }
        })
        .expect("根环里有自证过的这条根")
}

/// 挂载那一刻现行账（所选那一版 P 的账）里记成已释放、挂上之后还记着的（盘、起点槽）：可写挂载交回的分配器里释放代不高于 P 的 txg 的那几条
/// （释放代高于它的是这次挂载自己的写行、暖机释放的，挂载那一刻还分配着）。
pub fn released_in_the_account_of_the_selected_version(
    mounted: &singlefs_core::mount::Mounted,
) -> std::collections::BTreeSet<(DeviceIdentity, SlotNumber)> {
    let selected_version_txg = mounted.output.effective_root.checkpoint_txg;
    mounted
        .allocator
        .records()
        .iter()
        .filter(|record| record.is_released && record.generation <= selected_version_txg)
        .map(|record| (record.device, record.slot))
        .collect()
}

/// tri 第一种「不知道是不是回退」：R 的账与 R 写的谱系片都读回全零（正推 `rlin`）——R 的记录点名的打包单元有一片读不出，认不出是不是回退。
pub fn units_a_rollback_revived_stay_through_writes_while_its_lineage_unit_and_its_account_read_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
) {
    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
    );
    let recorded = &recorded_rollback.recorded;
    let lineage_unit_writes = lineage_unit_writes_of(recorded);
    assert!(
        !lineage_unit_writes.is_empty(),
        "回退那次发布写了谱系片（D18（块里携带什么信息） 已定项 24 第 5 条；阳性对照：故障有处可打）"
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&account_writes_of(recorded))
        .with_the_writes(&lineage_unit_writes);
    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
        images,
        &recorded_rollback,
        &ranges,
    );
}

/// tri 第一种的另一形：R 这次发布写的全部单元都读回全零（攻方 s2 `rall`），记录齐。
pub fn units_a_rollback_revived_stay_through_writes_while_every_unit_it_wrote_reads_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
) {
    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
    );
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&recorded_rollback.recorded.unit_writes());
    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
        images,
        &recorded_rollback,
        &ranges,
    );
}

/// tri 第二种「是回退、复活集算不出」：认得出回退（谱系片读得出），而 R_old 的根槽读回全零（正推 `rold`）——根环里没读出 R_old。
pub fn units_a_rollback_revived_stay_through_writes_while_its_account_and_the_root_slot_of_its_target_read_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
) {
    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
    );
    let recorded = &recorded_rollback.recorded;
    let rollback_target = recorded_rollback.rollback_target;
    let ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&account_writes_of(recorded))
        .with_the_root_slot_of(
            &recorded.crash_image_with_the_root_persisted_and_the_rotation_lost(),
            (rollback_target.instance, rollback_target.checkpoint_txg),
        );
    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
        images,
        &recorded_rollback,
        &ranges,
    );
}

/// tri 第二种的另一形：认得出回退、根环里读得出 R_old，而 R_old 的树表与分配记录树根读回全零（正推 `roldacct`）——R_old 的账读不出。
pub fn units_a_rollback_revived_stay_through_writes_while_its_account_and_the_account_of_its_target_read_back_zero_at_the_mount(
    images: &mut PoolCheckerImages,
) {
    let recorded_rollback = record_one_rollback_to_the_first_file_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_RECORDED_AS_R,
    );
    let rollback_target = root_read_from_the_ring_of(
        &recorded_rollback
            .recorded
            .crash_image_with_the_root_persisted_and_the_rotation_lost(),
        (
            recorded_rollback.rollback_target.instance,
            recorded_rollback.rollback_target.checkpoint_txg,
        ),
    );
    let mut ranges = RangesReadingBackAllZero::default()
        .with_the_writes(&account_writes_of(&recorded_rollback.recorded));
    for (pointer, unit) in [
        (
            rollback_target.tree_table,
            singlefs_core::transaction::TransactionUnit::TreeTable,
        ),
        (
            rollback_target.allocation_record_tree_root,
            singlefs_core::transaction::TransactionUnit::AllocationTree,
        ),
    ]
    .into_iter()
    .filter(|(pointer, _)| *pointer != NodePointer::empty_root())
    {
        for location in pointer.locations {
            ranges.0.push((
                location.device,
                location.slot.to_device_offset().0,
                unit.span_slots() * singlefs_format::SLOT_BYTES,
            ));
        }
    }
    assert!(
        ranges.0.len() > account_writes_of(&recorded_rollback.recorded).len(),
        "R_old（第一个文件那一版）的树表或分配记录树根有落点（阳性对照：故障有处可打）"
    );
    units_a_rollback_revived_stay_through_writes_while_these_read_back_zero_at_the_mount(
        images,
        &recorded_rollback,
        &ranges,
    );
}

/// 两块盘上分配位与隔离位同时置着的槽数之和：D28（挂载期承诺量） 已定项 1 第九项与「已分配」各扣一次的那几个。
pub fn slots_counted_both_as_allocated_and_as_isolated(allocator: &PoolAllocator) -> u64 {
    allocator
        .devices
        .iter()
        .map(singlefs_core::allocator::DeviceFreeMap::slots_counted_both_as_allocated_and_as_isolated)
        .sum()
}

/// 两块盘隔离的槽数之和（D28（挂载期承诺量） 已定项 1 第九项那一项）。
pub fn isolated_slots_of(allocator: &PoolAllocator) -> u64 {
    allocator
        .devices
        .iter()
        .map(singlefs_core::allocator::DeviceFreeMap::isolated_slots)
        .sum()
}

/// 正推 `research/prompts/m3-batch10z-lm-code-r1-sonnet-output.md` 第 1.3 节那一形之前覆盖写几次（探针 `S2_PREFIXES` 原样 40 次）。
pub const OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK: u64 = 40;

/// 那一形回退到倒数第几版（探针 `S2_RS=b6`：录的版本表里倒数第 1 + 6 版）。
pub const VERSIONS_BACK_OF_THE_ROLLBACK_TARGET: usize = 6;

/// 起池（384 槽那一档、第一个文件），覆盖写 `overwrites_before` 次得 P，经会话管理员回退到此前写过的倒数第 1 + `versions_back` 版（R_old），
/// 录这一次发布得 R（正推探针的 `record` 那一种 `b<数>`）。交回录下来的 R、R 复活的数据单元的落点（R_old 的文件数据指针里 P 不引用的那几条位置条目）、
/// R_old 的根与它账里仍分配着的落点（都从录下来的会话算，不经被测的方法）。
pub fn record_one_rollback_after_overwrites(
    overwrites_before: u64,
    versions_back: usize,
) -> RecordedRollback {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    // 每一版：根、文件数据指针的位置条目（盘、槽）、账里仍分配着的（盘、起点槽）。
    let version_of = |pool_at_this_version: &PoolUnderTest<
        RecordingBlockDevice<SparseBlockDevice>,
    >|
     -> RolledBackVersion {
        RolledBackVersion {
            root: *pool_at_this_version.session().current.root(),
            data_locations: pool_at_this_version
                .current_file_version()
                .data_pointers
                .iter()
                .flat_map(|pointer| pointer.locations)
                .map(|location| (location.device, location.slot))
                .collect(),
            allocated_in_its_account: pool_at_this_version
                .session()
                .allocator
                .records()
                .iter()
                .filter(|record| !record.is_released)
                .map(|record| (record.device, record.slot))
                .collect(),
        }
    };
    let mut versions_written = vec![version_of(&pool)];
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下（准入推抬 F 也做得成）");
        versions_written.push(version_of(&pool));
    }
    let target_version = &versions_written[versions_written.len() - 1 - versions_back];
    let rollback_target = target_version.root;
    let locations_referenced_by_p = &versions_written[versions_written.len() - 1].data_locations;
    let revived: Vec<PlacementOnDevice> = target_version
        .data_locations
        .iter()
        .filter(|location| !locations_referenced_by_p.contains(location))
        .map(|(device, slot)| PlacementOnDevice {
            device: *device,
            slot: *slot,
            span: singlefs_format::DATA_UNIT_BYTES.div_ceil(singlefs_format::SLOT_BYTES),
        })
        .collect();
    let allocated_in_the_account_of_the_rollback_target =
        target_version.allocated_in_its_account.clone();
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    let mut session = pool.session.take().expect("会话");
    {
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("第一个文件之后现行那一版带文件");
        };
        roll_back_by_a_forward_publish(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            RollbackTarget {
                instance: rollback_target.instance,
                checkpoint_txg: rollback_target.checkpoint_txg,
            },
        )
        .expect("回退到倒数第 7 版做得成（它在根环里、按实例表有效）");
    }
    pool.session = Some(session);
    let root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(root_indices.len(), 1, "回退那次发布只写一条根");
    RecordedRollback {
        recorded: RecordedPublish {
            content: previous_content.clone(),
            pool,
            base,
            writes,
            root_index: root_indices[0],
            key: (root.instance, root.checkpoint_txg),
            previous_key: (previous_root.instance, previous_root.checkpoint_txg),
            previous_content,
        },
        revived,
        rollback_target,
        allocated_in_the_account_of_the_rollback_target,
    }
}

/// [`record_one_rollback_after_overwrites`] 录的每一版。
pub struct RolledBackVersion {
    pub root: RootRecord,
    pub data_locations: Vec<(DeviceIdentity, SlotNumber)>,
    pub allocated_in_its_account: Vec<(DeviceIdentity, SlotNumber)>,
}

/// 单元区 1536 槽那一档每块盘的字节数：384 槽那一档的盘宽再加 1536 − 384 槽（攻方 `device_bytes_with_unit_area_slots(1536)` 同一个数）。
pub fn device_bytes_of_the_1536_slot_devices() -> u64 {
    const UNIT_AREA_SLOTS_OF_THE_1536_SLOT_DEVICES: u64 = 1536;
    const UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES: u64 = 384;
    device_bytes_of_the_384_slot_small_devices()
        + (UNIT_AREA_SLOTS_OF_THE_1536_SLOT_DEVICES - UNIT_AREA_SLOTS_OF_THE_384_SLOT_SMALL_DEVICES)
            * singlefs_format::SLOT_BYTES
}

/// 攻方 `research/prompts/m3-batch10z-lm-code-r1-opus-model/runs-summary/mount-counts-b11.txt` 那一形的前缀：顺序写几次（`S` 22 次）。
pub const SEQUENTIAL_WRITES_OF_TWENTY_DATA_UNITS_BEFORE_R: usize = 22;

/// 每次顺序写几个数据单元（攻方 `S`）。
pub const DATA_UNITS_OF_EACH_SEQUENTIAL_WRITE_BEFORE_R: usize = 20;

/// 那一形前缀末尾回退到倒数第几版（攻方 `b8`）。
pub const VERSIONS_BACK_OF_THE_ROLLBACK_BEFORE_R: usize = 8;

/// 挂上之后覆盖写几次、再顺序写几次（每次两个数据单元）：攻方 b9 那一串 30 步全被拒，这里 40 步。
pub const OVERWRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES: usize = 30;

pub const SEQUENTIAL_WRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES: usize = 10;

/// 挂上之后会话做的一步写。
#[derive(Clone, Copy, Debug)]
pub enum UserWriteAfterTheMount {
    Overwrite,
    SequentialWriteOfTwoDataUnits,
}

/// 攻方那一形（1536 槽两块盘）：起池、第一个文件之后顺序写 22 次（每次 20 个数据单元，先释放一大批），回退到倒数第 9 版，录一次建 inode 得 R。
pub fn record_one_inode_creation_after_releasing_a_large_batch_on_the_1536_slot_devices(
) -> RecordedPublish {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_1536_slot_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    let mut roots_written = vec![*pool.session().current.root()];
    for _sequential_write in 0..SEQUENTIAL_WRITES_OF_TWENTY_DATA_UNITS_BEFORE_R {
        pool.sequential_write(
            DATA_UNITS_OF_EACH_SEQUENTIAL_WRITE_BEFORE_R
                * singlefs_core::unit::data_unit_payload_capacity(),
        )
        .expect("1536 槽两块盘上这几次顺序写都落得下");
        roots_written.push(*pool.session().current.root());
    }
    let rollback_target =
        roots_written[roots_written.len() - 1 - VERSIONS_BACK_OF_THE_ROLLBACK_BEFORE_R];
    let mut session = pool.session.take().expect("会话");
    {
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("第一个文件之后现行那一版带文件");
        };
        roll_back_by_a_forward_publish(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            RollbackTarget {
                instance: rollback_target.instance,
                checkpoint_txg: rollback_target.checkpoint_txg,
            },
        )
        .expect("回退到倒数第 9 版做得成（它在根环里、按实例表有效）");
    }
    pool.session = Some(session);
    pool.content_of_the_current_version = pool
        .content_of_each_root
        .get(&(rollback_target.instance, rollback_target.checkpoint_txg))
        .cloned()
        .expect("回退目标是录的池写过的一版");
    let root_after_the_rollback = *pool.session().current.root();
    pool.note_root_of_the_current_content(&root_after_the_rollback);
    let previous_content = pool.content_of_the_current_version.clone();
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    pool.create_one_inode().expect("录的那次建 inode 落得下");
    let root = *pool.session().current.root();
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let root_indices: Vec<usize> = writes
        .iter()
        .enumerate()
        .filter(|(_, write)| write.kind == StepKind::RootRecordFua)
        .map(|(index, _)| index)
        .collect();
    assert_eq!(
        root_indices.len(),
        1,
        "录的那一次建 inode 只写一条根（准入够，没推抬 F）"
    );
    RecordedPublish {
        content: pool.content_of_the_current_version.clone(),
        pool,
        base,
        writes,
        root_index: root_indices[0],
        key: (root.instance, root.checkpoint_txg),
        previous_key: (
            root_after_the_rollback.instance,
            root_after_the_rollback.checkpoint_txg,
        ),
        previous_content,
    }
}

/// H2 双扣是 H1 卡死的主来源那一形（攻方 `research/prompts/m3-batch10z-lm-code-r1-opus-output.md`「K1-3 打中」两节，`mount-counts-b11.txt`、
/// `wedge-b6-b10.txt` 的 b9）：1536 槽两块盘、先顺序写释放一大批再回退，R 建一个 inode、R 写的全部单元挂载那一刻读回全零（不是回退而认不出，
/// 宽护并进现行账里全部已释放的落点）。改之前挂载那一刻隔离 2242 槽、其中 338 槽两项各扣一次，之后每次写都被准入拒、R 一直在根环里。
/// 改之后：挂载那一刻、每一步之后都没有一个槽同时算进「已分配」与第九项；覆盖写 30 次、顺序写 10 次一次都不拒；R 那一槽写掉（R 离开根环）；
/// R 还在根环里时 C 里 R 那一槽的槽一个都不空闲，每一步之后池级 checker 不红 I-7.4 与 I-4.8（回收之后补的隔离照罩着）。
pub fn wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_and_writes_go_through_on_the_1536_slot_devices(
    images: &mut PoolCheckerImages,
) {
    let recorded =
        record_one_inode_creation_after_releasing_a_large_batch_on_the_1536_slot_devices();
    assert!(
        lineage_unit_writes_of(&recorded).is_empty(),
        "建 inode 不写谱系片（R 不是回退：宽护只因为它写的单元读不出）"
    );
    let root_of_r = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(
        &crash_image,
        &RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes()),
        &lift,
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 写的单元读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 走读不成：择根退到 P（阳性对照：故障打到了）"
    );
    let released_in_the_current_account = released_in_the_account_of_the_selected_version(&mounted);
    let borrowed_for_r = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        )
        .expect("C 那一圈在 R 那一槽记着落点（R 记成有效根）");
    assert!(
        released_in_the_current_account
            .iter()
            .all(|(device, slot)| {
                borrowed_for_r
                    .iter()
                    .any(|borrowed| borrowed.device == *device && borrowed.slot == *slot)
            }),
        "认不出是不是回退：C 里 R 那一槽并进现行账里全部已释放的落点（阳性对照：宽护走到了）"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
        0,
        "挂载那一刻没有一个槽同时算进「已分配」与第九项（改之前 338 个）"
    );
    lift.lift();
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content_of_the_selected_version = recorded
        .pool
        .content_of_each_root
        .get(&(
            output.effective_root.instance,
            output.effective_root.checkpoint_txg,
        ))
        .cloned()
        .expect("挂载落到的那一版是录的池写过的一版");
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    let steps = std::iter::repeat_n(
        UserWriteAfterTheMount::Overwrite,
        OVERWRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES,
    )
    .chain(std::iter::repeat_n(
        UserWriteAfterTheMount::SequentialWriteOfTwoDataUnits,
        SEQUENTIAL_WRITES_AFTER_THE_MOUNT_ON_THE_1536_SLOT_DEVICES,
    ));
    let mut root_of_r_left_the_ring = false;
    // 迭代上界是步数（40）；跨轮带的是「R 离没离开根环」；出口：被拒、双扣、C 里 R 那一槽有空闲的槽、checker 红都当场判红（panic）。
    for (step_number, user_write) in (1..).zip(steps) {
        let root_of_r_was_in_the_ring = root_is_in_the_ring(&pool.image(), recorded.key);
        let published = match user_write {
            UserWriteAfterTheMount::Overwrite => pool.overwrite(OVERWRITE_BYTES),
            UserWriteAfterTheMount::SequentialWriteOfTwoDataUnits => {
                pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
            }
        };
        if let Err(refusal) = published {
            panic!("第 {step_number} 步被拒（改之前每一步都被准入拒）：{refusal:?}");
        }
        assert_eq!(
            slots_counted_both_as_allocated_and_as_isolated(&pool.session().allocator),
            0,
            "第 {step_number} 步之后没有一个槽同时算进「已分配」与第九项"
        );
        if root_of_r_was_in_the_ring && root_is_in_the_ring(&pool.image(), recorded.key) {
            let free_among_the_borrowed =
                free_slots_covered_by(&pool.session().allocator, &borrowed_for_r);
            assert!(
                free_among_the_borrowed.is_empty(),
                "第 {step_number} 步之后 R 还在根环里：C 里 R 那一槽的槽一个都不空闲（回收时补了隔离），空闲的 {free_among_the_borrowed:?}"
            );
        } else {
            root_of_r_left_the_ring = true;
        }
        images.hand_out(
            &format!("第 {step_number} 步之后被抛弃根引用的单元一个都没复用"),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
    assert!(
        root_of_r_left_the_ring,
        "40 步之内 R 那一槽写掉、R 离开根环（改之前 R 一直在环里、隔离到进程结束）"
    );
}

/// R 写出的打包单元里打包记录类型是 `record_type` 的那几份写：从录下来的写的字节解，不经被测的方法。
pub fn packed_unit_writes_of_type(
    recorded: &RecordedPublish,
    record_type: u16,
) -> Vec<&RetainedWrite> {
    recorded
        .unit_writes()
        .into_iter()
        .filter(|write| {
            write
                .bytes()
                .and_then(|bytes| singlefs_core::unit::parse_packed_unit(bytes).ok())
                .is_some_and(|packed| packed.identity.record_type == record_type)
        })
        .collect()
}

/// 起池（384 槽那一档、第一个文件），覆盖写 `overwrites_before` 次、建一个 inode，再经会话删掉它，录删除的第一次发布得 R
/// （崩在 R 的根槽落盘之后、轮换之前那一份写表只截到第一条根：删除那一次写两条根）。R 的记录点名它写的 inode 叶容器与墓碑片。
pub fn record_the_first_publish_of_one_deletion_after_overwrites(
    overwrites_before: u64,
) -> RecordedPublish {
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    for _overwrite in 0..overwrites_before {
        pool.overwrite(OVERWRITE_BYTES)
            .expect("384 槽小盘上第一个文件之后覆盖写这几次都落得下");
    }
    pool.create_one_inode().expect("建要删的那个 inode 落得下");
    let previous_root = *pool.session().current.root();
    let previous_content = pool.content_of_the_current_version.clone();
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    let inode = singlefs_core::address::InodeNumber(
        pool.current_file_version().inode_number_watermark() - 1,
    );
    pool.write_time_seconds += 1;
    let write_time_seconds = pool.write_time_seconds;
    pool.publish(
        singlefs_core::mounted_session::UserChange::Unlink {
            inode,
            write_time_seconds,
        },
        None,
    )
    .expect("删掉刚建的那个 inode 做得成");
    let operations = stream.retained_operations();
    let (mut writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let first_root_index = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("删除写了根");
    writes.truncate(first_root_index + 1);
    let root = RootRecord::parse_slot(
        writes[first_root_index].bytes().expect("根槽写带字节"),
        &pool.parameters.filesystem_identifier,
    )
    .expect("删除的第一条根自证");
    RecordedPublish {
        content: previous_content.clone(),
        pool,
        base,
        writes,
        root_index: first_root_index,
        key: (root.instance, root.checkpoint_txg),
        previous_key: (previous_root.instance, previous_root.checkpoint_txg),
        previous_content,
    }
}

/// 回退之前覆盖写几次（隔离量离整盘还远、写不被拒）。
pub const OVERWRITES_BEFORE_THE_ROLLBACK_WHOSE_PLACEMENTS_A_FLOOR_RAISE_RECLAIMS: u64 = 10;

/// 挂上之后至多覆盖写几次去等 R 那一槽写掉（R 离开根环）。
pub const OVERWRITES_UNTIL_R_LEAVES_THE_RING_AT_MOST: usize = 40;

/// H2 改法要回收时补隔离那一半（判决 `research/prompts/m3-batch10z-lm-code-r1-main-verification.md` 第三节第十四块第 1 条「I-7.4 照旧不红」）：
/// 覆盖写 10 次、回退到倒数第 7 版录 R，崩在 R 的根槽落盘之后、轮换之前，可写挂载那一刻 R 的根槽与 R 写的谱系片读回全零（C694 的 C：
/// 系统配置没见证 R、记录点名的谱系片验不过、记录不施加，择根落到 P）。R 的记录齐、用记录的新根段读得出 R 的账，C 那一圈把 R 的账里的落点
/// 隔离到这个进程写掉 R 那一槽；其中 R 复活的数据单元在 P 的账里已释放、
/// 还在 defer 窗口里（分配位置着），挂载那一刻不隔离（第九项不再算一次）。挂上之后先推一串抬 F 到准入上限：根环还没转，回收这几个复活单元的
/// 只有这一串；R 的根槽挂上之后照旧读回全零（到写掉那一槽为止），这一串读不出 R、不把它当被抛弃根算影子账，护着它们的只有 C 那一圈——分配器回收时照它补隔离
/// （`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）。推过之后复活的槽一个都不空闲；之后覆盖写到 R 离开根环，
/// 每一步之后没有一个槽同时算进「已分配」与第九项、R 还在根环里时复活的槽不空闲、池级 checker 不红 I-7.4 与 I-4.8、写一次都不拒。
/// 抬 F 那一串回收之后不补隔离时，推过抬 F 那一句先红。
pub fn revived_units_skipped_at_the_mount_are_isolated_when_the_floor_raise_reclaims_them_while_the_root_slot_of_the_rollback_read_back_zero(
    images: &mut PoolCheckerImages,
) {
    let RecordedRollback {
        recorded, revived, ..
    } = record_one_rollback_after_overwrites(
        OVERWRITES_BEFORE_THE_ROLLBACK_WHOSE_PLACEMENTS_A_FLOOR_RAISE_RECLAIMS,
        VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
    );
    assert!(
        !revived.is_empty(),
        "回退目标的数据单元 P 不引用：回退复活了它们（阳性对照：这一形走到了复活）"
    );
    let lineage_unit_writes = lineage_unit_writes_of(&recorded);
    assert!(
        !lineage_unit_writes.is_empty(),
        "回退那次发布写了谱系片（阳性对照：故障有处可打）"
    );
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut devices = devices_reading_back_all_zero_on(
        &crash_image,
        &RangesReadingBackAllZero::default()
            .with_the_root_slot_of(&crash_image, recorded.key)
            .with_the_writes(&lineage_unit_writes),
    );
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("R 的根槽与谱系片读回全零而记录齐：可写挂载落到 P、做得成");
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "R 的根槽读不出、系统配置没见证 R、记录不施加：择根落到 P（阳性对照：故障打到了）"
    );
    let isolated_by_the_unapplied_records = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_named_by_unapplied_journal_records();
    let revived_not_in_c: Vec<&PlacementOnDevice> = revived
        .iter()
        .filter(|revived_placement| {
            !isolated_by_the_unapplied_records.iter().any(|placement| {
                placement.device == revived_placement.device
                    && placement.slot == revived_placement.slot
            })
        })
        .collect();
    assert!(
        revived_not_in_c.is_empty(),
        "C 那一圈并进了 R 的账（记录的新根段读得出），R 复活的数据单元都在里面（阳性对照），缺 {revived_not_in_c:?}"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
        0,
        "挂载那一刻没有一个槽同时算进「已分配」与第九项"
    );
    // R 的根槽与谱系片挂上之后照旧读回全零，到这个进程写掉那几段为止（写过之后照读）：抬 F 那一串读不出 R，不把它当被抛弃根算影子账。
    let (output, session) = MountedSession::of_the_mount(mounted);
    let content_of_the_selected_version = recorded
        .pool
        .content_of_each_root
        .get(&(
            output.effective_root.instance,
            output.effective_root.checkpoint_txg,
        ))
        .cloned()
        .expect("挂载落到的那一版是录的池写过的一版");
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices,
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: content_of_the_selected_version,
        content_of_each_root: recorded.pool.content_of_each_root.clone(),
    };
    // 挂上之后先推一串抬 F 到准入上限（攻方动作 `F`）。
    let mut session_raising_the_floor = pool.session.take().expect("挂着的会话");
    let raised = {
        let PoolVersion::WithFile(current) = &mut session_raising_the_floor.current else {
            panic!("挂载落到的 P 带文件");
        };
        raise_rollback_floor_to_the_admission_ceiling(
            &pool.parameters,
            &mut pool.devices,
            &mut session_raising_the_floor.allocator,
            current,
            ShadowLedger::On,
        )
    };
    pool.session = Some(session_raising_the_floor);
    let reclaimed_by_the_raise = match raised {
        Ok(RaiseToTheAdmissionCeiling::Raised(raised)) => {
            pool.note_floor_raises(std::slice::from_ref(&raised));
            raised.reclaimed
        }
        Ok(RaiseToTheAdmissionCeiling::FloorAlreadyAtTheCeiling { floor, ceiling }) => {
            panic!(
                "挂上之后准入上限高于 F、推得出一串抬 F（阳性对照）：F {floor:?}、上限 {ceiling:?}"
            )
        }
        Err(refusal) => panic!("挂上之后抬 F 到准入上限做得成：{refusal:?}"),
    };
    assert!(
        revived.iter().any(|revived_placement| {
            reclaimed_by_the_raise
                .iter()
                .any(|reclaimed| reclaimed.slot == revived_placement.slot)
        }),
        "这一串抬 F 回收了复活的数据单元（阳性对照：回收时补隔离那一处走到了），回收的 {reclaimed_by_the_raise:?}"
    );
    assert!(
        root_is_in_the_ring(&pool.image(), recorded.key),
        "推过那一串抬 F 时 R 还在根环里"
    );
    let free_among_the_revived_after_the_raise =
        free_slots_covered_by(&pool.session().allocator, &revived);
    assert!(
        free_among_the_revived_after_the_raise.is_empty(),
        "推过抬 F 那一刻 R 还在根环里：复活的数据单元覆盖的槽一个都不空闲（那一串回收时照 C 那一圈补了隔离），空闲的 {free_among_the_revived_after_the_raise:?}"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&pool.session().allocator),
        0,
        "推过抬 F 之后没有一个槽同时算进「已分配」与第九项"
    );
    let mut overwrites = 0;
    // 迭代上界 OVERWRITES_UNTIL_R_LEAVES_THE_RING_AT_MOST；跨轮只带计数；出口：R 离开根环，
    // 被拒、双扣、复活的槽空闲、checker 红都当场判红（panic）。
    while root_is_in_the_ring(&pool.image(), recorded.key) {
        assert!(
            overwrites < OVERWRITES_UNTIL_R_LEAVES_THE_RING_AT_MOST,
            "{OVERWRITES_UNTIL_R_LEAVES_THE_RING_AT_MOST} 次覆盖写之内 R 那一槽写掉、R 离开根环"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {} 次覆盖写被拒：{refusal:?}", overwrites + 1));
        overwrites += 1;
        assert_eq!(
            slots_counted_both_as_allocated_and_as_isolated(&pool.session().allocator),
            0,
            "第 {overwrites} 次覆盖写之后没有一个槽同时算进「已分配」与第九项"
        );
        if root_is_in_the_ring(&pool.image(), recorded.key) {
            let free_among_the_revived = free_slots_covered_by(&pool.session().allocator, &revived);
            assert!(
                free_among_the_revived.is_empty(),
                "第 {overwrites} 次覆盖写之后 R 还在根环里：复活的数据单元覆盖的槽一个都不空闲，空闲的 {free_among_the_revived:?}"
            );
        }
        images.hand_out(
            &format!("第 {overwrites} 次覆盖写之后被抛弃根引用的单元一个都没复用"),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
}

/// `placements` 里每个落点覆盖的槽（起点槽起、跨度那么多槽），按落点的次序。
pub fn slots_covered_by(placements: &[PlacementOnDevice]) -> Vec<(DeviceIdentity, SlotNumber)> {
    placements
        .iter()
        .flat_map(|placement| {
            (placement.slot.0..placement.slot.0 + placement.span)
                .map(move |slot| (placement.device, SlotNumber(slot)))
        })
        .collect()
}

/// `placements` 里每个落点覆盖的槽里此刻隔离位置着的那几个（盘、槽），按落点的次序。
pub fn isolated_slots_covered_by(
    allocator: &PoolAllocator,
    placements: &[PlacementOnDevice],
) -> Vec<(DeviceIdentity, SlotNumber)> {
    slots_covered_by(placements)
        .into_iter()
        .filter(|(device, slot)| {
            allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == *device)
                .expect("池里有这块盘")
                .is_isolated(*slot)
        })
        .collect()
}

/// 第 10z 批第十四块实现员报告第六节 384 槽盘卡死的几形（调查报告 `~/.local/share/singlefs/work/m3-r1/investigate-10z-p/report.md` 第五节 22 形里取的）：
/// R 是哪一种、录在几次覆盖写之后，挂载那一刻读回全零的是哪几段（挂上就撤）。
#[derive(Clone, Copy, Debug)]
pub enum FormStuckBehindTheWideProtectionOnThe384SlotDevices {
    /// 覆盖写 `overwrites_before` 次、回退到倒数第 7 版录 R；R 的账与 R 写的谱系片读回全零（探针 `b6`、`racct,rlin`）。
    RollbackSixVersionsBack { overwrites_before: u64 },
    /// 覆盖写 `overwrites_before` 次、建一个 inode 录 R；R 写的全部单元读回全零（探针 `n`、`rall`）。
    InodeCreation { overwrites_before: u64 },
    /// 覆盖写 `overwrites_before` 次、建一个 inode、删掉它，录删除的第一次发布得 R；R 的账与 R 写的墓碑片读回全零（探针 `+n` `d`、`racct,rtomb`）。
    FirstPublishOfADeletion { overwrites_before: u64 },
}

/// 录下 `form` 的 R，交回录下来的 R 与挂载那一刻读回全零的那几段。
pub fn recorded_r_and_ranges_reading_back_zero_of(
    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
) -> (RecordedPublish, RangesReadingBackAllZero) {
    match form {
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before,
        } => {
            let recorded = record_one_rollback_after_overwrites(
                overwrites_before,
                VERSIONS_BACK_OF_THE_ROLLBACK_TARGET,
            )
            .recorded;
            let lineage_unit_writes = lineage_unit_writes_of(&recorded);
            assert!(
                !lineage_unit_writes.is_empty(),
                "回退那次发布写了谱系片（阳性对照：故障有处可打）"
            );
            let ranges = RangesReadingBackAllZero::default()
                .with_the_writes(&account_writes_of(&recorded))
                .with_the_writes(&lineage_unit_writes);
            (recorded, ranges)
        }
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before,
        } => {
            let recorded = record_one_inode_creation_after_overwrites(overwrites_before);
            let ranges =
                RangesReadingBackAllZero::default().with_the_writes(&recorded.unit_writes());
            (recorded, ranges)
        }
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
            overwrites_before,
        } => {
            let recorded =
                record_the_first_publish_of_one_deletion_after_overwrites(overwrites_before);
            let tombstone_unit_writes =
                packed_unit_writes_of_type(&recorded, singlefs_core::unit::PACKED_TYPE_TOMBSTONE);
            assert!(
                !tombstone_unit_writes.is_empty(),
                "删除的第一次发布写了墓碑片（阳性对照：故障有处可打）"
            );
            let ranges = RangesReadingBackAllZero::default()
                .with_the_writes(&account_writes_of(&recorded))
                .with_the_writes(&tombstone_unit_writes);
            (recorded, ranges)
        }
    }
}

/// 卡住的那一刻：挂着的池（读回全零已撤）、录下来的 R、准入第一次被拒时报的「被宽护卡住」。
pub struct PoolStuckBehindTheWideProtection {
    pub recorded: RecordedPublish,
    pub pool: PoolUnderTest<SparseBlockDevice>,
    pub stuck: StuckBehindTheWideProtection,
}

/// 录 `form` 的 R、崩在 R 的根槽落盘之后轮换之前，可写挂载那一刻那几段读回全零（挂上就撤），择根退到 P；挂上之后覆盖写一次：
/// 准入被拒、推抬 F 停在上限（`FloorAlreadyAtTheCeiling`），报「被宽护卡住」。挂上之后与被拒之后池级 checker 都不红 I-7.4、I-4.8。
pub fn pool_stuck_behind_the_wide_protection(
    images: &mut PoolCheckerImages,
    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
) -> PoolStuckBehindTheWideProtection {
    let (recorded, ranges) = recorded_r_and_ranges_reading_back_zero_of(form);
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let lift = ReadBackAllZeroLift::default();
    let mut devices = devices_reading_back_all_zero_on_until_lifted(&crash_image, &ranges, &lift);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .unwrap_or_else(|refusal| panic!("{form:?}：R 走读不成，可写挂载落到 P、做得成：{refusal:?}"));
    assert_eq!(
        (
            mounted.output.effective_root.instance,
            mounted.output.effective_root.checkpoint_txg
        ),
        recorded.previous_key,
        "{form:?}：择根退到 P（阳性对照：故障打到了）"
    );
    lift.lift();
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&devices, recorded.pool.device_bytes);
    let mut pool = pool_after_the_mount(&recorded, &image_after_the_mount, mounted);
    images.hand_out(
        &format!("{form:?}：挂上之后 I-7.4、I-4.8 不红"),
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
    let Err(refusal) = pool.overwrite(OVERWRITE_BYTES) else {
        panic!("{form:?}：挂上之后第一次覆盖写被宽护卡住（第 10z 批第十四块报告第六节：去掉双扣之后照旧全被拒），而它做成了");
    };
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("{form:?}：被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
        ),
        "{form:?}：推抬 F 停在上限（不再推的原因是 F 已在上限）：{:?}",
        no_space.stop
    );
    let NoSpaceAfterRaisingTheFloor {
        wide_protection_at_the_ceiling,
        ..
    } = *no_space;
    let WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) =
        wide_protection_at_the_ceiling
    else {
        panic!("{form:?}：F 在上限、C 那一圈还隔离着槽：报「被宽护卡住」，而报的是 {wide_protection_at_the_ceiling:?}");
    };
    images.hand_out(
        &format!("{form:?}：被拒之后 I-7.4、I-4.8 不红"),
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
    PoolStuckBehindTheWideProtection {
        recorded,
        pool,
        stuck: *stuck,
    }
}

/// 推环之后接着写几次覆盖写、几次顺序写（每次两个数据单元）。
pub const OVERWRITES_AFTER_PUSHING_THE_ROOT_RING: usize = 5;

pub const SEQUENTIAL_WRITES_AFTER_PUSHING_THE_ROOT_RING: usize = 2;

/// 被宽护卡住之后的报与推环（用户 2026-10-08 弹窗定「显式推环命令加带出路的错」）：报的点名 R 的 txg、宽护挡着的槽数、两条出路；
/// 调推环入口之后 R 那一槽写掉、C 那一圈一槽不剩、隔离回到不含宽护的数（这几形根环表里没有被抛弃根，就是 0），之后普通写放得下；
/// 推之前、推之后与之后每一次写，池级 checker 不红 I-7.4、I-4.8。
pub fn wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
    images: &mut PoolCheckerImages,
    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
) {
    let PoolStuckBehindTheWideProtection {
        recorded,
        mut pool,
        stuck,
    } = pool_stuck_behind_the_wide_protection(images, form);
    assert_eq!(
        stuck.checkpoint_txg_of_r,
        Some(recorded.key.1),
        "{form:?}：报的 R 是录下来的那一条（R 那一槽挂载那一刻记成有效根）"
    );
    assert_eq!(
        stuck.ring_slot_of_r,
        target_for_publish(
            recorded.key.1,
            recorded.pool.parameters.geometry.root_ring_slots_per_region
        ),
        "{form:?}：推环要写到的是 R 那一槽"
    );
    let isolated_by_the_wide_protection: u64 = stuck
        .isolated_slots_on_each_device
        .iter()
        .map(|(_, isolated)| *isolated)
        .sum();
    assert!(
        isolated_by_the_wide_protection > 0
            && isolated_by_the_wide_protection <= isolated_slots_of(&pool.session().allocator),
        "{form:?}：宽护挡着的槽数大于 0、不多于两块盘隔离的槽数：{isolated_by_the_wide_protection}"
    );
    let why = format!("{form:?}");
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, &why);
    assert_eq!(
        ways_out.len(),
        2,
        "{form:?}：预演过推完之后这次覆盖写放得下，两条出路都列：{ways_out:?}"
    );
    assert_the_way_out_by_unmounting_names_r_abandoned_and_warns_of_a_permanently_unreadable_account(
        &ways_out[0],
        recorded.key.1,
        &why,
    );
    let released_in_the_rehearsal = released_by_the_push_if_the_change_is_admitted_after_it(
        &pushing_the_root_ring,
    )
    .unwrap_or_else(|| {
        panic!("{form:?}：预演推完之后这次覆盖写放得下，而预演的结局是 {pushing_the_root_ring:?}")
    });
    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
        &stuck,
        &ways_out[1],
        &released_in_the_rehearsal,
        &why,
    );
    let isolated_before_the_push = isolated_slots_on_each_device_of(&pool.session().allocator);
    assert!(
        ring_slots_recorded_as_holding_an_abandoned_root_by(&pool.session().allocator).is_empty(),
        "{form:?}：这几形根环表里没有被抛弃根（隔离全是 C 那一圈的）"
    );
    let mut session = pool.session.take().expect("挂着的会话");
    let pushed = session
        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
            &mut pool.devices,
        );
    pool.session = Some(session);
    let pushed = pushed.unwrap_or_else(|refusal| panic!("{form:?}：推环做得成：{refusal:?}"));
    for version in &pushed.versions_published {
        pool.note_root_of_the_current_content(&version.root);
    }
    assert_eq!(
        u64::try_from(pushed.versions_published.len()).expect("次数"),
        stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
        "{form:?}：推环发的空发布次数就是报里写的那个数"
    );
    let image_after_the_push = pool.image();
    assert!(
        !root_is_in_the_ring(&image_after_the_push, recorded.key),
        "{form:?}：推环之后 R 那一槽被这个进程写掉、R 离开根环"
    );
    let allocator = &pool.session().allocator;
    assert!(
        allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records()
            .is_empty(),
        "{form:?}：推环之后 C 那一圈一槽不剩（宽护解开）"
    );
    assert_eq!(
        isolated_slots_of(allocator),
        0,
        "{form:?}：推环之后隔离回到不含宽护的数（根环表里没有被抛弃根）"
    );
    assert_eq!(
        released_in_the_rehearsal,
        isolated_slots_released_between(
            &isolated_before_the_push,
            &isolated_slots_on_each_device_of(allocator)
        ),
        "{form:?}：出路二说放出来的隔离槽数就是推完前后每块盘隔离数之差"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(allocator),
        0,
        "{form:?}：推环之后没有一个槽同时算进「已分配」与第九项"
    );
    images.hand_out(
        &format!("{form:?}：推环之后 I-7.4、I-4.8 不红"),
        || image_after_the_push.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
    for overwrite_number in 1..=OVERWRITES_AFTER_PUSHING_THE_ROOT_RING {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("{form:?}：推环之后第 {overwrite_number} 次覆盖写放得下：{refusal:?}")
        });
        images.hand_out(
            &format!("{form:?}：推环之后第 {overwrite_number} 次覆盖写之后 I-7.4、I-4.8 不红"),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
    for sequential_write_number in 1..=SEQUENTIAL_WRITES_AFTER_PUSHING_THE_ROOT_RING {
        pool.sequential_write(2 * singlefs_core::unit::data_unit_payload_capacity())
            .unwrap_or_else(|refusal| {
                panic!("{form:?}：推环之后第 {sequential_write_number} 次顺序写放得下：{refusal:?}")
            });
        images.hand_out(
            &format!(
                "{form:?}：推环之后第 {sequential_write_number} 次顺序写之后 I-7.4、I-4.8 不红"
            ),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
}

/// 前缀覆盖写 20 次、回退到倒数第 7 版（第十四块报告第六节 `o×20 | b6` 那一行）。
pub fn a_rollback_after_twenty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
    images: &mut PoolCheckerImages,
) {
    wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: 20,
        },
    );
}

/// 前缀覆盖写 40 次、建一个 inode（第十四块报告第六节 `o×40 | n | rall` 那一行）。
pub fn an_inode_creation_after_forty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
    images: &mut PoolCheckerImages,
) {
    wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
}

/// 前缀覆盖写 160 次、建一个 inode 再删掉它（第十四块报告第六节 `o×160+n | d` 那一行）。
pub fn a_deletion_after_one_hundred_sixty_overwrites_stuck_behind_the_wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
    images: &mut PoolCheckerImages,
) {
    wide_protection_is_reported_and_pushing_the_root_ring_lets_writes_through(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
            overwrites_before: 160,
        },
    );
}

/// 卸载重挂之后接着覆盖写几次。
pub const OVERWRITES_AFTER_UNMOUNTING_AND_MOUNTING_AGAIN: usize = 5;

/// 出路一（卸载重挂）走得通：主形（覆盖写 40 次、回退到倒数第 7 版，调查报告第一节复现那一形）被宽护卡住之后正常卸载做成，
/// 再可写挂载一次（读回全零早已撤：R 的账读得出）：R 按新实例的实例表判成被抛弃、根环表记着它自己账里的落点，宽护不再出现（C 那一圈一槽不剩），
/// 之后覆盖写一次都不拒，池级 checker 不红 I-7.4、I-4.8。
pub fn unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        recorded, mut pool, ..
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
        },
    );
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
            for version in &raised.publishes {
                pool.note_root_of_the_current_content(&version.root);
            }
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("被宽护卡住时正常卸载做得成：{refusal:?}"),
    }
    pool.crash_and_mount_writable()
        .expect("卸载之后再可写挂载做得成（R 的账读得出）");
    let occupancy = pool
        .session()
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表");
    let recorded_for_r = occupancy
        .placements_recorded_for_the_abandoned_root(recorded.key.0, recorded.key.1)
        .expect("再挂时根环表把 R 记成被抛弃根（按新实例的实例表）");
    assert!(
        !recorded_for_r.is_empty(),
        "再挂时 R 的账读得出：根环表记着它自己账里的落点"
    );
    assert!(
        occupancy
            .ring_slots_holding_placements_named_by_unapplied_journal_records()
            .is_empty(),
        "再挂时宽护不再出现：C 那一圈一槽不剩"
    );
    for overwrite_number in 1..=OVERWRITES_AFTER_UNMOUNTING_AND_MOUNTING_AGAIN {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("卸载重挂之后第 {overwrite_number} 次覆盖写放得下：{refusal:?}")
        });
        images.hand_out(
            &format!("卸载重挂之后第 {overwrite_number} 次覆盖写之后 I-7.4、I-4.8 不红"),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
}

/// 推到半路取不到落点那一形人为压到每块盘没挡的槽几个（调查报告第七节第 1 条 `PROBE_FILL_ISO_LEAVE=40`）。
pub const UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH: u64 = 40;

/// 主形卡住之后把每块盘没挡的槽隔离到只剩 [`UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH`] 个（注入级）。
pub fn pool_stuck_with_too_few_unblocked_slots_for_the_whole_push(
    images: &mut PoolCheckerImages,
) -> PoolStuckBehindTheWideProtection {
    let mut stuck_pool = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
        },
    );
    let session = stuck_pool.pool.session.as_mut().expect("挂着的会话");
    for device_map in &mut session.allocator.devices {
        let unit_area_start = device_map.unit_area_start().slot().0;
        // 迭代上界是单元区槽数；跨轮只带没挡的槽数；出口：只剩下要留的那几个。
        for slot in unit_area_start..unit_area_start + device_map.unit_area_slots() {
            if device_map.unblocked_slots() <= UNBLOCKED_SLOTS_LEFT_ON_EACH_DEVICE_BEFORE_THE_PUSH {
                break;
            }
            if !device_map.is_blocked_for_commit_generated(SlotNumber(slot)) {
                device_map.isolate(SlotNumber(slot), 1);
            }
        }
    }
    stuck_pool
}

/// 推到半路空发布取不到落点（调查报告第七节第 1 条造的那一次）：主形被宽护卡住之后，把每块盘没挡的槽隔离到只剩 40 个（人为的），
/// 推环要发的那一串在第 7 次左右取不到落点。推环入口在任何写之前拒（预演里被拒，`PlacementRefused`）：两块盘逐字节不变、
/// 分配器（`{:?}` 整份）与会话的现行那一版都不变——不留半截不一致的状态。
pub fn pushing_the_root_ring_without_placements_for_the_whole_push_is_refused_before_any_write(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        mut pool, stuck, ..
    } = pool_stuck_with_too_few_unblocked_slots_for_the_whole_push(images);
    let image_before = pool.image();
    let allocator_before = format!("{:?}", pool.session().allocator);
    let current_root_before = *pool.session().current.root();
    let mut session = pool.session.take().expect("挂着的会话");
    let pushed = session
        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
            &mut pool.devices,
        );
    pool.session = Some(session);
    match pushed {
        Err(RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite {
            refused_publish_in_the_push,
            publishes_in_the_push,
            cause: PublishError::PlacementRefused { .. },
        }) => {
            assert!(
                refused_publish_in_the_push > 1
                    && refused_publish_in_the_push <= publishes_in_the_push,
                "推到半路才取不到落点（前面几次放得下，阳性对照）：第 {refused_publish_in_the_push} 次，共 {publishes_in_the_push} 次"
            );
            assert_eq!(
                u64::try_from(publishes_in_the_push).expect("次数"),
                stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
                "这一串的次数就是报里写的那个数"
            );
        }
        other => panic!(
            "推到半路取不到落点：在任何写之前拒成预演被拒、拒因是取不到落点，而交回 {other:?}"
        ),
    }
    assert!(pool.image() == image_before, "推环被拒：两块盘逐字节不变");
    assert_eq!(
        format!("{:?}", pool.session().allocator),
        allocator_before,
        "推环被拒：分配器整份不变"
    );
    assert_eq!(
        *pool.session().current.root(),
        current_root_before,
        "推环被拒：会话的现行那一版不变"
    );
}

/// 要出路时预演里推环自己被拒（攻方 `research/prompts/m3-batch10z-tu-code-r1-opus-output.md` 第五节变异 K：自然历史里没走到这一支，注入级）：
/// 主形卡住之后把每块盘没挡的槽隔离到只剩 40 个，要出路：预演交回推环被拒（预演里那一串取不到落点）、出路只列卸载再挂，
/// 要出路那一次两块盘逐字节不变；照出路二真调推环入口，拒成同一种，`Debug` 逐字相同。
pub fn ways_out_when_the_push_finds_no_placements_in_the_rehearsal_list_only_unmounting_and_the_push_is_refused_the_same_way(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        mut pool, stuck, ..
    } = pool_stuck_with_too_few_unblocked_slots_for_the_whole_push(images);
    let image_before = pool.image();
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, "没挡的槽只剩 40 个");
    assert!(pool.image() == image_before, "要出路：两块盘逐字节不变");
    let RootRingPushRehearsal::PushRefused(rehearsed_push_refusal) = pushing_the_root_ring else {
        panic!("预演里推环那一串取不到落点：预演交回推环被拒，而交回 {pushing_the_root_ring:?}");
    };
    assert!(
        matches!(
            *rehearsed_push_refusal,
            RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite {
                cause: PublishError::PlacementRefused { .. },
                ..
            }
        ),
        "预演里推环拒成推环自己那一串预演取不到落点（阳性对照：注入打到了）：{rehearsed_push_refusal:?}"
    );
    assert_eq!(
        ways_out.len(),
        1,
        "推环在预演里被拒：只列卸载再挂：{ways_out:?}"
    );
    let real_push_refusal =
        push_the_root_ring_of(&mut pool).expect_err("照出路二真调推环入口：同样被拒");
    assert_eq!(
        format!("{real_push_refusal:?}"),
        format!("{rehearsed_push_refusal:?}"),
        "预演里推环的拒绝与真调推环入口的拒绝逐字相同"
    );
}

/// 实例 4 里覆盖写几次再把 F 抬到 C 的 txg：之后有 4 个以上不同的非空状态，准入上限高过 C 的 txg（抬 F 只供测试强制进入的那一档）。
pub const OVERWRITES_IN_THE_INSTANCE_RAISING_THE_FLOOR_TO_THE_ABANDONED_ROOT: usize = 4;

/// 挂上之后至多覆盖写几次去等 C 离开根环。
pub const OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST: usize = 40;

/// 被抛弃根那一圈的 defer 槽（第 10z 批第十四块实现员报告第七节第 2 条；派发第十六块第 2 条）的池：崩溃恢复抛弃了 C 的那份盘面
/// （[`pool_whose_newest_root_was_abandoned_by_a_crash_recovery`]：实例 3 择根退到 C 之前那一条 B、写行换下 C 与 B 共用的实例 2 那张实例表），
/// 实例 4 挂上、覆盖写 4 次、把 F 抬到 C 的 txg（B 不再是候选根），停在那里（进程退出）。交回实例 4 退出时的盘面。
/// 这之后的可写挂载里，C 引用、现行账里已释放（实例 3 的写行换下的）、释放代高于 F 的槽不再被候选根豁免，还在 defer 窗口里（分配位置着）。
pub fn image_after_raising_the_floor_to_the_txg_of_the_abandoned_root(
    built: &PoolWithAnAbandonedRoot,
) -> MemoryPool {
    let mut pool = PoolUnderTest {
        parameters: built.parameters.clone(),
        device_bytes: built.device_bytes,
        devices: plain_devices_on(&built.image),
        session: None,
        write_time_seconds: built.write_time_seconds + 1000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    pool.crash_and_mount_writable()
        .expect("实例 4 可写挂载做得成（C 的根槽与账读得出）");
    for overwrite_number in 1..=OVERWRITES_IN_THE_INSTANCE_RAISING_THE_FLOOR_TO_THE_ABANDONED_ROOT {
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("实例 4 第 {overwrite_number} 次覆盖写：{refusal:?}"));
    }
    let mut session = pool.session.take().expect("挂着的会话");
    let raised = {
        let PoolVersion::WithFile(current) = &mut session.current else {
            panic!("覆盖写之后现行那一版带文件");
        };
        raise_rollback_floor(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            built.abandoned_root.checkpoint_txg,
            ShadowLedger::On,
        )
    };
    raised.unwrap_or_else(|refusal| panic!("实例 4 把 F 抬到 C 的 txg 做得成：{refusal:?}"));
    pool.image()
}

/// `placements` 罩着的槽里，现行账（`allocator` 的记录）里记成已释放、释放代高于 `floor` 的（盘、槽）：挂载那一刻还在 defer 窗口里、
/// 不被候选根（txg ≥ F）豁免的那一截的候选。从记录算，不经被测的方法。
pub fn released_above_the_floor_and_covered_by(
    allocator: &PoolAllocator,
    placements: &[PlacementOnDevice],
    floor: CheckpointTxg,
) -> Vec<(DeviceIdentity, SlotNumber)> {
    allocator
        .records()
        .iter()
        .filter(|record| record.is_released && record.generation > floor)
        .flat_map(|record| {
            (record.slot.0..record.slot.0 + u64::from(record.span_slots))
                .map(move |slot| (record.device, SlotNumber(slot)))
        })
        .filter(|(device, slot)| {
            placements.iter().any(|placement| {
                placement.device == *device
                    && slot.0 >= placement.slot.0
                    && slot.0 < placement.slot.0 + placement.span
            })
        })
        .collect()
}

/// 被抛弃根那一圈挂载那一刻不双扣、回收之后照旧隔离（派发第十六块第 2 条）：实例 4 把 F 抬到 C 的 txg 之后崩了再挂（实例 5，F = C 的 txg）。
/// 挂载那一刻 C 记成被抛弃根，它引用的槽里有现行账已释放、释放代高于 F（还在 defer 窗口里、分配位置着）、不被候选根豁免的：
/// 两块盘没有一个槽同时算进「已分配」与第九项。之后覆盖写到 C 离开根环：C 还在环里时它引用的槽一个都不空闲，其中那几个 defer 槽
/// 至少有一个在 C 离环之前被回收、补上了隔离（回收时照根环表补，阳性对照：走到了回收那一处）；每一步池级 checker 不红 I-7.4、I-4.8。
/// 被抛弃根那一圈照旧整段隔离时挂载那一刻那一句红。
pub fn defer_slots_an_abandoned_root_references_are_not_counted_twice_at_the_mount_and_stay_isolated_once_reclaimed(
    images: &mut PoolCheckerImages,
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let image = image_after_raising_the_floor_to_the_txg_of_the_abandoned_root(&built);
    let fault =
        ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let mut devices = devices_with_the_read_fault_on(&image, &fault);
    // 空间准入关着（只供测试的开关）：会话不为空间推抬 F，一次覆盖写只发一次、txg 只走一格，看得到「C 那一槽写掉之前那一次发布」之后的那一刻。
    let mounted = mount_writable_with_space_admission(
        &built.parameters,
        &mut devices,
        SpaceAdmission::SkippedByTheTestOnlySwitch,
    )
    .expect("实例 5 可写挂载做得成");
    assert_eq!(
        mounted.output.effective_root.rollback_floor, built.abandoned_root.checkpoint_txg,
        "挂载那一刻 F 是 C 的 txg（阳性对照：B 不再是候选根）"
    );
    let recorded_for_c = mounted
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .placements_recorded_for_the_abandoned_root(
            built.abandoned_root.instance,
            built.abandoned_root.checkpoint_txg,
        )
        .filter(|placements| !placements.is_empty())
        .expect("挂载那一刻根环表把 C 记成被抛弃根、带着它引用的落点");
    let defer_slots_of_c = released_above_the_floor_and_covered_by(
        &mounted.allocator,
        &recorded_for_c,
        built.abandoned_root.checkpoint_txg,
    );
    assert!(
        !defer_slots_of_c.is_empty(),
        "C 引用的槽里有现行账已释放、释放代高于 F 的（阳性对照：这一形走到了被抛弃根引用的 defer 槽）"
    );
    let free_defer_slots_of_c_at_the_mount: Vec<&(DeviceIdentity, SlotNumber)> = defer_slots_of_c
        .iter()
        .filter(|(device, slot)| {
            mounted
                .allocator
                .devices
                .iter()
                .any(|device_map| device_map.device == *device && device_map.is_free(*slot))
        })
        .collect();
    assert!(
        free_defer_slots_of_c_at_the_mount.is_empty(),
        "挂载那一刻这几个 defer 槽分配位置着、不空闲：空闲的 {free_defer_slots_of_c_at_the_mount:?}"
    );
    assert_eq!(
        slots_counted_both_as_allocated_and_as_isolated(&mounted.allocator),
        0,
        "挂载那一刻没有一个槽同时算进「已分配」与第九项（D28（挂载期承诺量） 已定项 1 第九项：被抛弃根引用的 defer 槽算在「已分配」里）"
    );
    let mut pool = pool_under_test_after_the_mount(
        &built.parameters,
        built.device_bytes,
        devices,
        mounted,
        built.write_time_seconds + 3000,
    );
    let key_of_c = (
        built.abandoned_root.instance,
        built.abandoned_root.checkpoint_txg,
    );
    let mut a_defer_slot_was_reclaimed_and_isolated_while_c_was_in_the_ring = false;
    let mut overwrites = 0;
    // 迭代上界 OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST；跨轮带计数与「回收补隔离走到了没有」；出口：C 离开根环，
    // 被拒、双扣、C 引用的槽空闲、checker 红都当场判红（panic）。
    while root_is_in_the_ring(&pool.image(), key_of_c) {
        assert!(
            overwrites < OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST,
            "{OVERWRITES_UNTIL_THE_ABANDONED_ROOT_LEAVES_THE_RING_AT_MOST} 次覆盖写之内 C 那一槽写掉、C 离开根环"
        );
        pool.overwrite(OVERWRITE_BYTES)
            .unwrap_or_else(|refusal| panic!("第 {} 次覆盖写被拒：{refusal:?}", overwrites + 1));
        overwrites += 1;
        let allocator = &pool.session().allocator;
        assert_eq!(
            slots_counted_both_as_allocated_and_as_isolated(allocator),
            0,
            "第 {overwrites} 次覆盖写之后没有一个槽同时算进「已分配」与第九项"
        );
        if root_is_in_the_ring(&pool.image(), key_of_c) {
            let free_among_the_placements_of_c = free_slots_covered_by(allocator, &recorded_for_c);
            assert!(
                free_among_the_placements_of_c.is_empty(),
                "第 {overwrites} 次覆盖写之后 C 还在根环里：它引用的槽一个都不空闲，空闲的 {free_among_the_placements_of_c:?}"
            );
            a_defer_slot_was_reclaimed_and_isolated_while_c_was_in_the_ring |=
                defer_slots_of_c.iter().any(|(device, slot)| {
                    allocator.devices.iter().any(|device_map| {
                        device_map.device == *device && device_map.is_isolated(*slot)
                    })
                });
        }
        images.hand_out(
            &format!("第 {overwrites} 次覆盖写之后 C 引用的单元一个都没复用"),
            || pool.image(),
            PoolCheckerExpectation::only_the_named_invariants()
                .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
        );
    }
    assert!(
        a_defer_slot_was_reclaimed_and_isolated_while_c_was_in_the_ring,
        "C 离环之前有一个 defer 槽被回收、照根环表补上了隔离（阳性对照：走到了回收时补隔离那一处）"
    );
}

/// 推一次环（会话的推环入口），做成时把推出来的每条根记成现行内容。
pub fn push_the_root_ring_of<Device: DeviceOverASparseImage>(
    pool: &mut PoolUnderTest<Device>,
) -> Result<RootRingPushed, RootRingPushRefused> {
    let mut session = pool.session.take().expect("挂着的会话");
    let pushed = session
        .push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(
            &mut pool.devices,
        );
    pool.session = Some(session);
    if let Ok(pushed) = &pushed {
        for version in &pushed.versions_published {
            pool.note_root_of_the_current_content(&version.root);
        }
    }
    pushed
}

/// 每块盘隔离的槽数（按分配器里盘的次序）。
pub fn isolated_slots_on_each_device_of(allocator: &PoolAllocator) -> Vec<(DeviceIdentity, u64)> {
    allocator
        .devices
        .iter()
        .map(|device_map| (device_map.device, device_map.isolated_slots()))
        .collect()
}

/// 推环前后每块盘隔离的槽数少了几个（推完多了就当场判红：推环只写掉 C 那一圈的槽、放隔离）。
pub fn isolated_slots_released_between(
    before: &[(DeviceIdentity, u64)],
    after: &[(DeviceIdentity, u64)],
) -> Vec<(DeviceIdentity, u64)> {
    before
        .iter()
        .zip(after)
        .map(|((device, isolated_before), (device_after, isolated_after))| {
            assert_eq!(device, device_after, "前后数的是同一个分配器：盘的次序不变");
            (
                *device,
                isolated_before.checked_sub(*isolated_after).unwrap_or_else(|| {
                    panic!("盘 {} 推完隔离的槽数（{isolated_after}）不多于推之前（{isolated_before}）", device.0)
                }),
            )
        })
        .collect()
}

/// 要出路（会话的 `ways_out_of_the_wide_protection`，盘表交这一刻池里的那一份）；会话不在被拒的那一版上就当场判红（`why` 只用在断言消息里）。
pub fn ways_out_of<Device: DeviceOverASparseImage>(
    pool: &PoolUnderTest<Device>,
    stuck: &StuckBehindTheWideProtection,
    why: &str,
) -> WaysOutOfTheWideProtection {
    pool.session()
        .ways_out_of_the_wide_protection(&pool.devices, stuck)
        .unwrap_or_else(|refusal| {
            panic!("{why}：会话还在被拒的那一版上，要出路做得成：{refusal:?}")
        })
}

/// 照出路二真走一遍（这一刻盘面与会话的一份拷贝上推环、再发同一个改动 `change`），交回再发被拒的那一份；推环被拒或再发做成就当场判红。
pub fn refusal_of_the_same_change_after_pushing_the_root_ring_on_a_copy_of(
    pool: &PoolUnderTest<SparseBlockDevice>,
    change: UserChange<'_>,
    why: &str,
) -> UserChangeRefused {
    let (mut copy, _) = pool_failing_one_changing_call_copied_from(pool, None);
    push_the_root_ring_of(&mut copy)
        .unwrap_or_else(|push_refusal| panic!("{why}：照出路二推环做得成：{push_refusal:?}"));
    let mut session = copy.session.take().expect("挂着的会话");
    match session.publish_user_change(&mut copy.devices, change) {
        Ok(published) => panic!(
            "{why}：预演说推完之后这次改动照旧被拒，照出路二推完再发却做成了：{:?}",
            published.versions_published.len()
        ),
        Err(refusal) => refusal,
    }
}

/// 预演说推完照旧被拒时（`ChangeStillRefusedAfterThePush`），预演交回的拒绝与照出路二真推完、再发同一个改动交回的拒绝 `Debug` 逐字相同
/// （攻方 `research/prompts/m3-batch10z-tu-code-r1-opus-model/` 的 `atk_follow_way_two` 那一判 `dbg_same`：只进内存的盘读不回自己写过的块、
/// 写进内存时块号错一位，列不列推环不变，变的是这一份拒绝）。
pub fn assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
    pool: &PoolUnderTest<SparseBlockDevice>,
    pushing_the_root_ring: &RootRingPushRehearsal,
    change: UserChange<'_>,
    why: &str,
) {
    let RootRingPushRehearsal::ChangeStillRefusedAfterThePush {
        refusal: rehearsed_refusal,
        ..
    } = pushing_the_root_ring
    else {
        panic!("{why}：预演推环做得成、推完之后这次改动照旧被拒：{pushing_the_root_ring:?}");
    };
    let refusal_after_the_push =
        refusal_of_the_same_change_after_pushing_the_root_ring_on_a_copy_of(pool, change, why);
    assert_eq!(
        format!("{rehearsed_refusal:?}"),
        format!("{refusal_after_the_push:?}"),
        "{why}：预演交回的拒绝与推环之后真再发同一个改动交回的拒绝逐字相同"
    );
}

/// 出路里推环那一条预演的结局是「推完放得下」时，交回预演里每块盘放出来的隔离槽数；别的结局交 `None`。
pub fn released_by_the_push_if_the_change_is_admitted_after_it(
    pushing_the_root_ring: &RootRingPushRehearsal,
) -> Option<Vec<(DeviceIdentity, u64)>> {
    match pushing_the_root_ring {
        RootRingPushRehearsal::ChangeAdmittedAfterThePush {
            isolated_slots_released_by_the_push_on_each_device,
        } => Some(isolated_slots_released_by_the_push_on_each_device.clone()),
        RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
        | RootRingPushRehearsal::PushRefused(_) => None,
    }
}

/// 出路一那句话在 R 那一槽记着有效根时的说法：R（带 txg）再挂时判成被抛弃；R 的账读不出若是盘上永久坏，卸载之后再挂会被拒、会话已断开。
pub fn assert_the_way_out_by_unmounting_names_r_abandoned_and_warns_of_a_permanently_unreadable_account(
    way_out_by_unmounting: &str,
    checkpoint_txg_of_r: CheckpointTxg,
    why: &str,
) {
    for expected in [
        "正常卸载".to_string(),
        "再可写挂载".to_string(),
        format!(
            "R（txg {}）按新实例的实例表判成被抛弃",
            checkpoint_txg_of_r.0
        ),
        "R 的账读不出若是盘上永久坏，卸载之后再挂会被拒".to_string(),
        "会话在卸载时已断开".to_string(),
    ] {
        assert!(
            way_out_by_unmounting.contains(&expected),
            "{why}：出路一写明「{expected}」：{way_out_by_unmounting}"
        );
    }
}

/// 出路一那句话在 R 那一槽没有读得出的有效根时的说法：点名那一槽、说再挂之后 C 那一圈不再有这一槽，不说 R 判成被抛弃
/// （攻方 `research/prompts/m3-batch10z-pqrs-code-r1-opus-output.md` 第一节末：实测卸载再挂之后 C 那一圈直接没了）。
pub fn assert_the_way_out_by_unmounting_says_the_ring_slot_of_r_holds_no_readable_root(
    way_out_by_unmounting: &str,
    stuck: &StuckBehindTheWideProtection,
    why: &str,
) {
    for expected in [
        "正常卸载".to_string(),
        "再可写挂载".to_string(),
        format!(
            "R 那一槽（根环槽 {}.{}）没有读得出的有效根",
            stuck.ring_slot_of_r.region, stuck.ring_slot_of_r.slot
        ),
        "C 那一圈照它重算、不再有这一槽".to_string(),
    ] {
        assert!(
            way_out_by_unmounting.contains(&expected),
            "{why}：出路一写明「{expected}」：{way_out_by_unmounting}"
        );
    }
    assert!(
        !way_out_by_unmounting.contains("判成被抛弃"),
        "{why}：R 没有根，出路一不说它判成被抛弃：{way_out_by_unmounting}"
    );
}

/// 出路二那句话：推环入口的名字、推几次、写掉环里几槽、R 那一槽的槽号、放出来的隔离槽数（总数与逐盘，与预演的数相同）、回退候选的代价，
/// 不写「整环」。
pub fn assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
    stuck: &StuckBehindTheWideProtection,
    way_out_by_the_push: &str,
    released_on_each_device: &[(DeviceIdentity, u64)],
    why: &str,
) {
    let publishes = stuck.empty_publishes_until_the_ring_slot_of_r_is_written;
    let released: u64 = released_on_each_device
        .iter()
        .map(|(_, released_on_the_device)| *released_on_the_device)
        .sum();
    let mut expected_phrases = vec![
        "push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states"
            .to_string(),
        format!("发 {publishes} 次 F 不变的空发布"),
        format!("写掉根环里接下来轮到的 {publishes} 槽"),
        format!(
            "根环槽 {}.{}",
            stuck.ring_slot_of_r.region, stuck.ring_slot_of_r.slot
        ),
        format!("隔离的槽放出来 {released} 个"),
        "少于 4 个不同状态".to_string(),
    ];
    expected_phrases.extend(released_on_each_device.iter().map(
        |(device, released_on_the_device)| {
            format!("盘 {} 上 {released_on_the_device} 个", device.0)
        },
    ));
    for expected in &expected_phrases {
        assert!(
            way_out_by_the_push.contains(expected.as_str()),
            "{why}：出路二写明「{expected}」：{way_out_by_the_push}"
        );
    }
    assert!(
        !way_out_by_the_push.contains("整环"),
        "{why}：出路二写推几次、写掉几槽，不写「整环」：{way_out_by_the_push}"
    );
}

/// 一次写被拒、报「被宽护卡住」：这次调用里推了几串抬 F、发成了几版，与报的那一份。
pub struct WriteStuckBehindTheWideProtection {
    pub floor_raises_pushed: usize,
    pub versions_published: usize,
    pub stuck: StuckBehindTheWideProtection,
}

/// 被拒是推满仍不够、停在上限、报「被宽护卡住」（`why` 只用在断言消息里）。
pub fn stuck_behind_the_wide_protection_reported_by(
    refusal: UserChangeRefused,
    why: &str,
) -> WriteStuckBehindTheWideProtection {
    let UserChangeRefused::NoSpaceAfterRaisingTheFloor(no_space) = refusal else {
        panic!("{why}：被拒是推满仍不够：{refusal:?}");
    };
    assert!(
        matches!(
            no_space.stop,
            FloorRaiseStop::FloorAlreadyAtTheCeiling { .. }
        ),
        "{why}：推抬 F 停在上限：{:?}",
        no_space.stop
    );
    let NoSpaceAfterRaisingTheFloor {
        floor_raises,
        versions_published,
        wide_protection_at_the_ceiling,
        ..
    } = *no_space;
    let WideProtectionAtTheCeiling::StuckBehindTheWideProtection(stuck) =
        wide_protection_at_the_ceiling
    else {
        panic!("{why}：F 在上限、C 那一圈还隔离着槽：报「被宽护卡住」，而报的是 {wide_protection_at_the_ceiling:?}");
    };
    WriteStuckBehindTheWideProtection {
        floor_raises_pushed: floor_raises.len(),
        versions_published: versions_published.len(),
        stuck: *stuck,
    }
}

/// 攻方 K1 那一形（`research/prompts/m3-batch10z-pqrs-code-r1-opus-model/atk_part.rs` 的 `atk_k1_record`，`atk_k1_push_trace` 的 pre = 4、fill = 16）：
/// 第一个文件之后前缀覆盖写几次，文件每次长几个数据单元、长到几个，再录一次长几个单元的顺序写 R。
pub const OVERWRITES_BEFORE_GROWING_THE_FILE_UNTIL_THE_DEVICES_ARE_NEARLY_FULL: u64 = 4;

pub const DATA_UNITS_THE_FILE_GROWS_BY_EACH_TIME_BEFORE_R: usize = 5;

pub const DATA_UNITS_THE_FILE_GROWS_TO_BEFORE_R: usize = 16;

pub const DATA_UNITS_R_GROWS_THE_FILE_BY: usize = 3;

/// 攻方 K1 那一形崩在 R 第一条记录之后的盘面（不带任何读故障的自然崩溃点：可写挂载择 P，R 那几条记录在水位之上、没施加，
/// C 那一圈只有 R 那一槽，R 那一槽没有根）。
pub struct PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
    pub crashed: MemoryPool,
    pub content_of_each_root: BTreeMap<(InstanceGeneration, CheckpointTxg), Vec<u8>>,
    pub write_time_seconds: u64,
    pub data_units_of_the_file: usize,
}

pub fn pool_crashed_after_the_first_record_of_a_long_sequential_write(
) -> PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let stream = SharedStream::retaining_contents();
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes_of_the_384_slot_small_devices(),
        |identity, device| {
            RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())
        },
    );
    for overwrite_number in 1..=OVERWRITES_BEFORE_GROWING_THE_FILE_UNTIL_THE_DEVICES_ARE_NEARLY_FULL
    {
        pool.overwrite(OVERWRITE_BYTES).unwrap_or_else(|refusal| {
            panic!("前缀第 {overwrite_number} 次覆盖写落得下：{refusal:?}")
        });
    }
    let mut data_units_of_the_file = 1;
    // 迭代上界是长到的单元数；跨轮带着文件现在几个单元；出口：再长一步就越过长到的单元数。
    while data_units_of_the_file + DATA_UNITS_THE_FILE_GROWS_BY_EACH_TIME_BEFORE_R
        <= DATA_UNITS_THE_FILE_GROWS_TO_BEFORE_R
    {
        data_units_of_the_file += DATA_UNITS_THE_FILE_GROWS_BY_EACH_TIME_BEFORE_R;
        pool.sequential_write(data_units_of_the_file * unit_bytes)
            .unwrap_or_else(|refusal| {
                panic!("文件长到 {data_units_of_the_file} 个单元落得下：{refusal:?}")
            });
    }
    assert_eq!(
        data_units_of_the_file, DATA_UNITS_THE_FILE_GROWS_TO_BEFORE_R,
        "文件长到攻方那一形的单元数"
    );
    let base = pool.image();
    let operations_before_r = stream.operation_count();
    let published_r = pool
        .sequential_write((data_units_of_the_file + DATA_UNITS_R_GROWS_THE_FILE_BY) * unit_bytes)
        .unwrap_or_else(|refusal| panic!("录的那次顺序写（R）落得下：{refusal:?}"));
    assert!(
        published_r.floor_raises.is_empty(),
        "R 那次没推抬 F：录下来的写全是 R 自己的"
    );
    let operations = stream.retained_operations();
    let (writes, _segments, _stream_indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before_r..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let end_of_the_first_record = writes
        .iter()
        .position(|write| write.kind == StepKind::JournalRecord)
        .expect("R 写了记录")
        + 1;
    let mut crashed = base;
    crashed.apply_writes(&writes[..end_of_the_first_record]);
    PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
        crashed,
        content_of_each_root: pool.content_of_each_root.clone(),
        write_time_seconds: pool.write_time_seconds + 100,
        data_units_of_the_file,
    }
}

impl PoolCrashedAfterTheFirstRecordOfALongSequentialWrite {
    /// 拿崩溃那一刻的盘面起两块新盘、可写挂载一次（空间准入判着）。
    pub fn mounted(&self) -> PoolUnderTest<SparseBlockDevice> {
        let mut pool = PoolUnderTest {
            parameters: parameters_of_the_384_slot_small_devices(),
            device_bytes: device_bytes_of_the_384_slot_small_devices(),
            devices: plain_devices_on(&self.crashed),
            session: None,
            write_time_seconds: self.write_time_seconds,
            content_of_the_current_version: Vec::new(),
            content_of_each_root: self.content_of_each_root.clone(),
        };
        pool.crash_and_mount_writable()
            .expect("崩在 R 第一条记录之后：可写挂载择 P、做得成");
        let holding = pool
            .session()
            .allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        assert_eq!(
            holding.len(),
            1,
            "C 那一圈只有 R 那一槽（阳性对照）：{:?}",
            holding.keys().collect::<Vec<_>>()
        );
        pool
    }

    /// 挂上之后写一次长 `data_units_grown_by` 个单元的顺序写：被拒、报「被宽护卡住」（这一次调用里会话先推一串抬 F、把 F 抬到上限）。
    /// 交回被拒之后挂着的池：再试同一个写时 F 已在上限，那一次调用一个写都不推。
    pub fn mounted_and_stuck_once_by_a_write_grown_by(
        &self,
        data_units_grown_by: usize,
    ) -> PoolUnderTest<SparseBlockDevice> {
        let mut pool = self.mounted();
        let refusal = pool
            .sequential_write(self.bytes_of_the_file_grown_by(data_units_grown_by))
            .expect_err("挂上之后第一次长文件：宽护挡着准入");
        stuck_behind_the_wide_protection_reported_by(
            refusal,
            &format!("挂上之后第一次长 {data_units_grown_by} 个单元"),
        );
        pool
    }

    /// 把文件长到比 P 多 `data_units` 个数据单元的顺序写的字节数（攻方的 g）。
    pub fn bytes_of_the_file_grown_by(&self, data_units: usize) -> usize {
        (self.data_units_of_the_file + data_units)
            * singlefs_core::unit::data_unit_payload_capacity()
    }
}

/// 攻方 K1 那一形里长到这几个单元的写：宽护挡着、推完放得下的那一档（攻方第一节：g ≤ 19 推完第一次就放得下）；推完照旧被拒的那一档（g = 20）；
/// 没有 C 那一圈的对照臂同一个长度也被拒的那一档（g = 36，对照臂 g = 36 起被拒）。
pub const DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY: usize = 13;

pub const DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY: usize = 20;

pub const DATA_UNITS_A_WRITE_REFUSED_EVEN_WITHOUT_THE_WIDE_PROTECTION_GROWS_THE_FILE_BY: usize = 36;

/// 甲（攻方 K1 那一形 g = 13）：要出路时预演推环与推完之后这次改动、推完放得下——两条出路都列：出路一说 R 那一槽没有读得出的有效根，
/// 出路二写明推几次、写掉几槽、R 那一槽、放出来的隔离槽数。挂上之后第一次被拒时会话推了一串抬 F、把 F 抬到上限；再试同一个写，
/// 那一次调用一串抬 F 都不推。报「被宽护卡住」那一次不预演（用户 2026-10-09 弹窗定「改：预演延到要出路时才做」）：一块只进内存的盘都没建
/// （只进内存的盘一建就探底下那块盘的物理块大小，发布路径别处不探：数探的次数）；要出路那一次每块盘建一块、一个写都不发、会话的分配器不动。
/// 照推环走完，放出来的隔离槽数就是推完前后每块盘隔离数之差，同一个写放得下，池级 checker 不红 I-7.4、I-4.8。
pub fn a_write_stuck_behind_the_wide_protection_that_fits_after_the_push_lists_both_ways_out_and_goes_through_after_the_push(
    images: &mut PoolCheckerImages,
) {
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let stuck_once = crashed.mounted_and_stuck_once_by_a_write_grown_by(
        DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY,
    );
    let (mut pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&stuck_once, None);
    let length = crashed
        .bytes_of_the_file_grown_by(DATA_UNITS_A_WRITE_THE_PUSH_LETS_THROUGH_GROWS_THE_FILE_BY);
    let image_before_the_write = pool.image();
    let allocator_before_the_write = format!("{:?}", pool.session().allocator);
    let refusal = pool
        .sequential_write(length)
        .expect_err("长 13 个单元再试：宽护照旧挡着准入");
    let WriteStuckBehindTheWideProtection {
        floor_raises_pushed,
        versions_published,
        stuck,
    } = stuck_behind_the_wide_protection_reported_by(refusal, "长 13 个单元再试");
    assert_eq!(
        (floor_raises_pushed, versions_published),
        (0, 0),
        "第一次被拒时已把 F 抬到上限：再试这一次一串抬 F 都没推、一版都没发（阳性对照：盘上的改动只可能来自预演）"
    );
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0),
        "报「被宽护卡住」那一次不预演：一块只进内存的盘都没建、一个改盘调用都没有"
    );
    assert!(
        pool.image() == image_before_the_write,
        "报「被宽护卡住」那一次：两块盘逐字节不变"
    );
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, "长 13 个单元");
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.calls.get()
        ),
        (u64::try_from(pool.devices.len()).expect("盘数"), 0),
        "要出路那一次预演：每块盘建一块只进内存的盘，一个改盘调用都没发到底下的盘"
    );
    assert!(
        pool.image() == image_before_the_write,
        "要出路那一次预演推环与推完之后这次改动：两块盘逐字节不变"
    );
    assert_eq!(
        format!("{:?}", pool.session().allocator),
        allocator_before_the_write,
        "预演在分配器的拷贝上：会话的分配器整份不变"
    );
    assert_eq!(
        stuck.checkpoint_txg_of_r, None,
        "崩在 R 第一条记录之后：R 那一槽没有读得出的有效根（阳性对照：出路一要说的是这一种）"
    );
    assert_eq!(ways_out.len(), 2, "推完放得下：两条出路都列：{ways_out:?}");
    assert_the_way_out_by_unmounting_says_the_ring_slot_of_r_holds_no_readable_root(
        &ways_out[0],
        &stuck,
        "长 13 个单元",
    );
    let released_in_the_rehearsal =
        released_by_the_push_if_the_change_is_admitted_after_it(&pushing_the_root_ring)
            .unwrap_or_else(|| {
                panic!("预演推完之后这次改动放得下，而预演的结局是 {pushing_the_root_ring:?}")
            });
    assert_the_way_out_by_the_push_states_the_publishes_the_ring_slot_of_r_and_the_released_slots(
        &stuck,
        &ways_out[1],
        &released_in_the_rehearsal,
        "长 13 个单元",
    );
    let isolated_before_the_push = isolated_slots_on_each_device_of(&pool.session().allocator);
    let pushed = push_the_root_ring_of(&mut pool)
        .unwrap_or_else(|push_refusal| panic!("照出路二推环做得成：{push_refusal:?}"));
    assert_eq!(
        u64::try_from(pushed.versions_published.len()).expect("次数"),
        stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
        "推环发的空发布次数就是报里写的那个数"
    );
    let released_by_the_push = isolated_slots_released_between(
        &isolated_before_the_push,
        &isolated_slots_on_each_device_of(&pool.session().allocator),
    );
    assert_eq!(
        released_in_the_rehearsal, released_by_the_push,
        "出路二说放出来的隔离槽数就是推完前后每块盘隔离数之差"
    );
    pool.sequential_write(length)
        .unwrap_or_else(|refusal_after_the_push| {
            panic!("推完之后同一个写放得下：{refusal_after_the_push:?}")
        });
    images.hand_out(
        "推完写下之后 I-7.4、I-4.8 不红",
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

/// 甲（攻方 K1 那一形）：推完照旧放不下的写，被宽护卡住、要出路时只列卸载再挂（预演推完之后照旧被拒）；预演交回的拒绝与照出路二
/// 在一份拷贝上真推完、再发同一个写交回的拒绝逐字相同。照出路一走：正常卸载、再可写挂载，同一个写放得下，池级 checker 不红 I-7.4、I-4.8。
pub fn a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_only_unmounting_which_lets_it_through(
    images: &mut PoolCheckerImages,
    data_units_grown_by: usize,
) {
    let why = format!("长 {data_units_grown_by} 个单元");
    let crashed = pool_crashed_after_the_first_record_of_a_long_sequential_write();
    let mut pool = crashed.mounted();
    let length = crashed.bytes_of_the_file_grown_by(data_units_grown_by);
    let refusal = pool.sequential_write(length).expect_err("宽护挡着准入");
    let WriteStuckBehindTheWideProtection { stuck, .. } =
        stuck_behind_the_wide_protection_reported_by(refusal, &why);
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, &why);
    assert!(
        matches!(
            pushing_the_root_ring,
            RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
        ),
        "{why}：预演推环做得成、推完之后这次改动照旧被拒：{pushing_the_root_ring:?}"
    );
    assert_eq!(
        ways_out.len(),
        1,
        "{why}：推完放不下，只列卸载再挂：{ways_out:?}"
    );
    assert_the_way_out_by_unmounting_says_the_ring_slot_of_r_holds_no_readable_root(
        &ways_out[0],
        &stuck,
        &why,
    );
    let content_of_the_refused_write = content_of(length, pool.write_time_seconds);
    assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
        &pool,
        &pushing_the_root_ring,
        UserChange::SequentialWrite(FirstFile {
            content: &content_of_the_refused_write,
            write_time_seconds: pool.write_time_seconds,
        }),
        &why,
    );
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
            for version in &raised.publishes {
                pool.note_root_of_the_current_content(&version.root);
            }
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("{why}：现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(unmount_refusal) => panic!("{why}：照出路一正常卸载做得成：{unmount_refusal:?}"),
    }
    pool.crash_and_mount_writable()
        .unwrap_or_else(|mount_refusal| {
            panic!("{why}：卸载之后再可写挂载做得成：{mount_refusal:?}")
        });
    pool.sequential_write(length)
        .unwrap_or_else(|refusal_after_mounting_again| {
            panic!("{why}：卸载再挂之后同一个写放得下：{refusal_after_mounting_again:?}")
        });
    images.hand_out(
        &format!("{why}：卸载再挂写下之后 I-7.4、I-4.8 不红"),
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

/// 攻方 K1 那一形 g = 20：宽护确实挡着（对照臂同一个长度放得下），推完之后上限退到环里最旧的根，照旧被拒。
pub fn a_write_stuck_behind_the_wide_protection_still_refused_after_the_push_lists_only_unmounting_which_lets_it_through(
    images: &mut PoolCheckerImages,
) {
    a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_only_unmounting_which_lets_it_through(images,
        DATA_UNITS_A_WRITE_STILL_REFUSED_AFTER_THE_PUSH_GROWS_THE_FILE_BY,
    );
}

/// 攻方 K1 那一形 g = 36：没有 C 那一圈的对照臂同一个长度也被拒（宽护是挡着的之一、不是唯一），照样报「被宽护卡住」、只列卸载再挂。
pub fn a_write_refused_even_without_the_wide_protection_lists_only_unmounting_which_lets_it_through(
    images: &mut PoolCheckerImages,
) {
    a_write_stuck_behind_the_wide_protection_the_push_does_not_let_through_lists_only_unmounting_which_lets_it_through(images,
        DATA_UNITS_A_WRITE_REFUSED_EVEN_WITHOUT_THE_WIDE_PROTECTION_GROWS_THE_FILE_BY,
    );
}

/// 甲（攻方第一节「实现员自己的三形上……长到 60、80 个单元的推完照旧被拒」）：实现员三形卡住之后，把文件写成这么多个数据单元的顺序写
/// 被宽护卡住、要出路时只列卸载再挂；R 那一槽记着有效根，出路一照 R 的 txg 说它判成被抛弃、写明账永久读不出时的后果。
/// 预演交回的拒绝与照出路二在一份拷贝上真推完、再发同一个写交回的拒绝逐字相同。
pub fn a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
    form: FormStuckBehindTheWideProtectionOnThe384SlotDevices,
    data_units: usize,
) {
    let why = format!("{form:?} 写成 {data_units} 个单元");
    let PoolStuckBehindTheWideProtection {
        recorded, mut pool, ..
    } = pool_stuck_behind_the_wide_protection(images, form);
    let length = data_units * singlefs_core::unit::data_unit_payload_capacity();
    let refusal = pool.sequential_write(length).expect_err("宽护挡着准入");
    let WriteStuckBehindTheWideProtection { stuck, .. } =
        stuck_behind_the_wide_protection_reported_by(refusal, &why);
    let WaysOutOfTheWideProtection {
        pushing_the_root_ring,
        ways_out,
    } = ways_out_of(&pool, &stuck, &why);
    assert!(
        matches!(
            pushing_the_root_ring,
            RootRingPushRehearsal::ChangeStillRefusedAfterThePush { .. }
        ),
        "{why}：预演推环做得成、推完之后这次改动照旧被拒：{pushing_the_root_ring:?}"
    );
    let content_of_the_refused_write = content_of(length, pool.write_time_seconds);
    assert_the_rehearsed_refusal_is_the_refusal_after_pushing_the_root_ring(
        &pool,
        &pushing_the_root_ring,
        UserChange::SequentialWrite(FirstFile {
            content: &content_of_the_refused_write,
            write_time_seconds: pool.write_time_seconds,
        }),
        &why,
    );
    assert_eq!(
        ways_out.len(),
        1,
        "{why}：推完放不下，只列卸载再挂：{ways_out:?}"
    );
    assert_eq!(
        stuck.checkpoint_txg_of_r,
        Some(recorded.key.1),
        "{why}：R 那一槽记着录下来的 R（阳性对照：出路一要说的是这一种）"
    );
    assert_the_way_out_by_unmounting_names_r_abandoned_and_warns_of_a_permanently_unreadable_account(
        &ways_out[0],
        recorded.key.1,
        &why,
    );
}

/// 长 60 个数据单元、80 个数据单元的顺序写（攻方 `atk_k1_implementer_forms_sweep` 推完仍拒的那两档）。
pub const DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH: usize = 60;

pub const DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH: usize = 80;

pub fn a_sixty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: 20,
        },
        DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

pub fn an_eighty_unit_write_after_a_rollback_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: 20,
        },
        DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

pub fn a_sixty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
        DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

pub fn an_eighty_unit_write_after_an_inode_creation_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
        DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

pub fn a_sixty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
            overwrites_before: 160,
        },
        DATA_UNITS_OF_A_LONG_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

pub fn an_eighty_unit_write_after_a_deletion_stuck_behind_the_wide_protection_lists_only_unmounting(
    images: &mut PoolCheckerImages,
) {
    a_long_sequential_write_stuck_behind_the_wide_protection_lists_only_unmounting(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::FirstPublishOfADeletion {
            overwrites_before: 160,
        },
        DATA_UNITS_OF_A_LONGER_WRITE_STILL_REFUSED_AFTER_THE_PUSH,
    );
}

/// 一份挂着的会话逐字段拷一份（同一刻的分配器、现行那一版、实例代号、影子账那一臂、认下的池）：同一个盘面上起几份池各推各的。
pub fn copy_of_the_session(session: &MountedSession) -> MountedSession {
    MountedSession {
        allocator: session.allocator.clone(),
        current: session.current.clone(),
        instance: session.instance,
        shadow_ledger: session.shadow_ledger,
        parameters_and_device_table: session.parameters_and_device_table.clone(),
    }
}

/// 整池的改盘调用（写、写零、屏障，几块盘共用一个数）数到第 `failing_call` 次（从 1 数）时报块设备错、不落盘，只这一次；`None` 只数不报。
/// 另数整池的读与探物理块大小（只进内存的盘一建就探一次底下那块盘，发布路径别处不探）。
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

/// 推环入口先判冻结（判决第二节「K2-1 报法」；攻方第二节）：主形卡住之后推环，真发到半路一次改盘调用报块设备错——前面那几次已落盘，
/// 会话的现行那一版停在最后一次落盘的那一版上（攻方 M1），失败那一次冻结在分配器上。再推一次（盘表照挂载时那一份交，过得了入口比盘表次序那一道）：
/// 拒成点名冻结的那一种、带冻结那一次的 txg 与实例代号，在核盘表那一套与预演之前，两块盘逐字节不变、分配器与会话不动。改之前预演在冻结着的分配器拷贝上照样过，
/// 报成 `PublishFailedAfterTheRehearsal`、拒因是冻结。
pub fn pushing_the_root_ring_again_after_a_block_device_error_halfway_is_refused_as_frozen_before_any_write(
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
pub fn pushing_the_root_ring_with_the_two_device_bodies_swapped_is_refused_before_any_write(
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

/// 判决第二节「K1-1 三个量」正推注入级那一形：C 那一圈记着落点、它们罩着的槽却一个都没隔离。主形（建 inode、前缀 40 次）卡住之后，
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
pub fn no_space_while_the_ring_slots_hold_placements_none_of_which_is_isolated_is_not_reported_as_the_wide_protection(
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
pub fn pushing_the_root_ring_while_the_ring_slots_hold_placements_none_of_which_is_isolated_writes_nothing(
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

/// 记着空串的槽不算 C 的槽（正推 `emptyHeld`、`slotsWrittenEmpty`）：主形（建 inode、前缀 40 次）卡住之后，在现行那一版那一槽
/// （从现行那一版起最后轮到的一槽，满一环）记一个空串。再写一次照旧报「被宽护卡住」，R 那一槽与要推的次数都照旧是录下来的 R 那一槽；
/// 推环推到 R 那一槽为止，交回写掉的那几槽就是 C 那一圈原来那一槽。
pub fn a_ring_slot_recording_no_placement_is_not_the_ring_slot_the_push_writes_up_to(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        recorded,
        mut pool,
        stuck,
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
    let ring_slot_of_the_current_version = target_for_publish(
        pool.session().current.root().checkpoint_txg,
        pool.parameters.geometry.root_ring_slots_per_region,
    );
    {
        let allocator = &mut pool.session.as_mut().expect("挂着的会话").allocator;
        let mut holding = allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records();
        assert_eq!(
            holding.keys().copied().collect::<Vec<RootRingSlot>>(),
            vec![stuck.ring_slot_of_r],
            "主形 C 那一圈只有 R 那一槽（阳性对照）"
        );
        holding.insert(ring_slot_of_the_current_version, Vec::new());
        allocator.note_placements_named_by_unapplied_journal_records(holding);
    }
    let refusal = pool
        .overwrite(OVERWRITE_BYTES)
        .expect_err("宽护照旧挡着：覆盖写被拒");
    let WriteStuckBehindTheWideProtection {
        stuck: stuck_again, ..
    } = stuck_behind_the_wide_protection_reported_by(refusal, "多记一个空串之后再写");
    assert_eq!(
        (
            stuck_again.ring_slot_of_r,
            stuck_again.empty_publishes_until_the_ring_slot_of_r_is_written
        ),
        (
            target_for_publish(
                recorded.key.1,
                recorded.pool.parameters.geometry.root_ring_slots_per_region
            ),
            stuck.empty_publishes_until_the_ring_slot_of_r_is_written
        ),
        "记着空串的那一槽什么都不罩：R 那一槽与要推的次数照旧"
    );
    let pushed = push_the_root_ring_of(&mut pool)
        .unwrap_or_else(|push_refusal| panic!("推环做得成：{push_refusal:?}"));
    assert_eq!(
        u64::try_from(pushed.versions_published.len()).expect("次数"),
        stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
        "推到 R 那一槽为止，不推到记着空串的那一槽"
    );
    assert_eq!(
        pushed.ring_slots_written,
        vec![stuck.ring_slot_of_r],
        "交回写掉的那几槽：C 那一圈原来那一槽（记着空串的那一槽不算）"
    );
}

/// 两槽 C（攻方 M12，照攻方 `research/prompts/m3-batch10z-pqrs-code-r1-opus-model/atk_part3.rs` 的 `atk_k2_two_c_slots_push` 造法：丙并中间那一形，
/// Q 顺序写、R 建 inode，挂载那一刻 Q 的根与末条记录、R 的账读回全零，C 那一圈有 Q、R 两槽）：挂上就推环，推的次数是到较晚轮到的 R 那一槽的距离，
/// 交回写掉的两槽，推完 C 那一圈一槽不剩、隔离回 0，池级 checker 不红 I-7.4、I-4.8。R 那一槽取最先轮到的那一槽时推环停在 Q 那一槽、R 还记着。
pub fn pushing_the_root_ring_with_two_ring_slots_holding_placements_writes_up_to_the_later_one_and_releases_both(
    images: &mut PoolCheckerImages,
) {
    let recorded = record_a_sequential_write_as_q_and_an_inode_creation_as_r_after_overwrites(
        OVERWRITES_BEFORE_THE_PUBLISHES_RECORDED_AS_Q_AND_R,
    );
    let crash_image = recorded.crash_image_with_the_root_of_r_persisted_and_its_rotation_lost();
    let ranges = recorded.ranges_reading_back_all_zero_at_the_mount(&crash_image);
    let mut faulted_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut faulted_devices,
        SpaceAdmission::JudgedByTheFormula,
    )
    .expect("Q 的根与末条记录、R 的账读回全零：可写挂载退到 P、做得成");
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&faulted_devices, recorded.pool.device_bytes);
    let (_, session) = MountedSession::of_the_mount(mounted);
    let mut pool = PoolUnderTest {
        parameters: recorded.pool.parameters.clone(),
        device_bytes: recorded.pool.device_bytes,
        devices: plain_devices_on(&image_after_the_mount),
        session: Some(session),
        write_time_seconds: recorded.pool.write_time_seconds + 10_000,
        content_of_the_current_version: Vec::new(),
        content_of_each_root: BTreeMap::new(),
    };
    let slots_per_region = recorded.pool.parameters.geometry.root_ring_slots_per_region;
    let ring_slot_of_q = target_for_publish(recorded.key_of_q.1, slots_per_region);
    let ring_slot_of_r = target_for_publish(recorded.key_of_r.1, slots_per_region);
    let holding_before_the_push: Vec<RootRingSlot> = pool
        .session()
        .allocator
        .root_ring_occupancy()
        .expect("挂载建的分配器装着根环表")
        .ring_slots_holding_placements_named_by_unapplied_journal_records()
        .into_keys()
        .collect();
    let mut q_and_r = vec![ring_slot_of_q, ring_slot_of_r];
    q_and_r.sort();
    assert_eq!(
        holding_before_the_push, q_and_r,
        "C 那一圈有 Q、R 两槽（阳性对照）"
    );
    let current_txg = pool.session().current.root().checkpoint_txg;
    let publishes_until_the_ring_slot_of = |ring_slot: RootRingSlot| -> usize {
        (1..=singlefs_format::ROOT_RING_REGIONS * slots_per_region.count())
            .position(|publishes_ahead| {
                target_for_publish(
                    CheckpointTxg(current_txg.0 + publishes_ahead),
                    slots_per_region,
                ) == ring_slot
            })
            .expect("一环之内每一槽都轮到一次")
            + 1
    };
    assert!(
        publishes_until_the_ring_slot_of(ring_slot_of_q)
            < publishes_until_the_ring_slot_of(ring_slot_of_r),
        "Q 那一槽比 R 那一槽先轮到（阳性对照：取最先轮到的与最后轮到的推的次数不同）"
    );
    let pushed = push_the_root_ring_of(&mut pool)
        .unwrap_or_else(|refusal| panic!("两槽 C 推环做得成：{refusal:?}"));
    assert_eq!(
        pushed.versions_published.len(),
        publishes_until_the_ring_slot_of(ring_slot_of_r),
        "推的次数是到较晚轮到的 R 那一槽的距离"
    );
    assert_eq!(
        pushed.ring_slots_written, q_and_r,
        "交回写掉的那几槽：Q、R 两槽"
    );
    let allocator = &pool.session().allocator;
    assert!(
        allocator
            .root_ring_occupancy()
            .expect("挂载建的分配器装着根环表")
            .ring_slots_holding_placements_named_by_unapplied_journal_records()
            .is_empty(),
        "推完 C 那一圈一槽不剩"
    );
    assert_eq!(
        isolated_slots_of(allocator),
        0,
        "推完隔离回 0（这一形根环表里没有被抛弃根）"
    );
    images.hand_out(
        "推完之后 I-7.4、I-4.8 不红",
        || pool.image(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_none_of_these_violated(&INVARIANTS_OF_UNIT_REUSE),
    );
}

/// 要出路时会话已不在报「被宽护卡住」时被拒的那一版上（之后推过环）：按哪一刻算条款没写、第一版不支持，拒成点名这一形的那一种、
/// 带被拒那一版与现行那一版的实例代号与 txg；不预演：一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有，两块盘逐字节不变。
pub fn asking_for_the_ways_out_after_the_session_moved_past_the_version_refused_on_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        mut pool, stuck, ..
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
    let refused_on = *pool.session().current.root();
    push_the_root_ring_of(&mut pool).unwrap_or_else(|refusal| panic!("推环做得成：{refusal:?}"));
    let (counting_pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&pool, None);
    let current_after_the_push = *counting_pool.session().current.root();
    assert!(
        current_after_the_push.checkpoint_txg > refused_on.checkpoint_txg,
        "推环之后会话已在后面的一版上（阳性对照）"
    );
    let image_before = counting_pool.image();
    match counting_pool
        .session()
        .ways_out_of_the_wide_protection(&counting_pool.devices, &stuck)
    {
        Err(
            WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version,
            },
        ) => assert_eq!(
            (
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version
            ),
            (
                refused_on.instance,
                refused_on.checkpoint_txg,
                current_after_the_push.instance,
                current_after_the_push.checkpoint_txg
            ),
            "带被拒那一版与现行那一版的实例代号与 txg"
        ),
        other => panic!("会话已不在被拒的那一版上：不预演，而交回 {other:?}"),
    }
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.reads.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0, 0),
        "不预演：一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有"
    );
    assert!(counting_pool.image() == image_before, "两块盘逐字节不变");
}

/// 盘表怎么换（要出路时交进去的那一份）。
#[derive(Clone, Copy, Debug)]
pub enum DeviceTableChange {
    /// 两块盘次序对调（身份与盘不拆开）：攻方 `research/prompts/m3-batch10z-vw-code-r1-opus-model/atk_vw_part3.rs` 的 `table-reversed`。
    Reversed,
    /// 多交一块本池之外的空盘。
    OneDeviceMore,
    /// 少交最后一块盘。
    OneDeviceFewer,
}

/// 池里交进去的盘表的设备身份，按交进去的次序。
pub fn device_identities_in_the_order_handed_in(
    pool: &PoolUnderTest<DeviceFailingOneChangingCall>,
) -> Vec<DeviceIdentity> {
    pool.devices.iter().map(|(identity, _)| *identity).collect()
}

/// 照 `table_change` 换池里交进去的盘表（多交的那一块是本池之外的空盘，改盘调用一样数进 `calls_of_the_pool`），交回少交的那一块
/// （别的换法交 `None`）；换完的盘表与挂载时的不同（阳性对照，当场判）。
pub fn change_the_device_table_of(
    pool: &mut PoolUnderTest<DeviceFailingOneChangingCall>,
    table_change: DeviceTableChange,
    calls_of_the_pool: &Rc<ChangingCallsOfThePool>,
) -> Option<(DeviceIdentity, DeviceFailingOneChangingCall)> {
    let device_table_of_the_mount = device_identities_in_the_order_handed_in(pool);
    let mut device_left_out = None;
    match table_change {
        DeviceTableChange::Reversed => pool.devices.reverse(),
        DeviceTableChange::OneDeviceMore => pool.devices.push((
            DeviceIdentity(u32::try_from(device_table_of_the_mount.len()).expect("几块盘")),
            DeviceFailingOneChangingCall {
                inner: SparseBlockDevice::new(pool.device_bytes, PhysicalBlockSizeInBytes(512)),
                calls_of_the_pool: Rc::clone(calls_of_the_pool),
            },
        )),
        DeviceTableChange::OneDeviceFewer => device_left_out = pool.devices.pop(),
    }
    assert_ne!(
        device_identities_in_the_order_handed_in(pool),
        device_table_of_the_mount,
        "{table_change:?}：交进去的盘表与挂载时的不同（阳性对照）"
    );
    device_left_out
}

/// [`change_the_device_table_of`] 换回去：盘表回到挂载时那一份。
pub fn restore_the_device_table_of(
    pool: &mut PoolUnderTest<DeviceFailingOneChangingCall>,
    table_change: DeviceTableChange,
    device_left_out: Option<(DeviceIdentity, DeviceFailingOneChangingCall)>,
) {
    match table_change {
        DeviceTableChange::Reversed => pool.devices.reverse(),
        DeviceTableChange::OneDeviceMore => {
            pool.devices.pop();
        }
        DeviceTableChange::OneDeviceFewer => pool
            .devices
            .push(device_left_out.expect("少交的那一块留着")),
    }
}

/// 被宽护卡住（建 inode、前缀 40 次那一形）之后，在这一刻盘面与会话的一份拷贝上照 `table_change` 换盘表要出路：在任何读之前拒成
/// 盘表那一种、带两份盘表，一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有，盘逐字节不变；换回挂载时那一份盘表照常预演
/// （同一个会话原样照旧）。不核盘表次序时，次序对调的盘表在预演里推环那一串照样过（核盘表不看次序）、列推环，照出路二推完再发却被拒成盘表那一种
/// （攻方「打中二」）；多一块、少一块在预演里推环被拒，但已经读过盘。
pub fn assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
    table_change: DeviceTableChange,
) {
    let PoolStuckBehindTheWideProtection { pool, stuck, .. } =
        pool_stuck_behind_the_wide_protection(
            images,
            FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
                overwrites_before: 40,
            },
        );
    let (mut counting_pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&pool, None);
    let device_table_of_the_mount = device_identities_in_the_order_handed_in(&counting_pool);
    let device_left_out =
        change_the_device_table_of(&mut counting_pool, table_change, &calls_of_the_pool);
    let device_table_handed_in = device_identities_in_the_order_handed_in(&counting_pool);
    let image_before = counting_pool.image();
    match counting_pool
        .session()
        .ways_out_of_the_wide_protection(&counting_pool.devices, &stuck)
    {
        Err(WaysOutOfTheWideProtectionRefused::DeviceTableOtherThanTheOneOfTheMount {
            device_table_of_the_mount: refused_table_of_the_mount,
            device_table_handed_in: refused_table_handed_in,
        }) => assert_eq!(
            (refused_table_of_the_mount, refused_table_handed_in),
            (device_table_of_the_mount.clone(), device_table_handed_in),
            "{table_change:?}：带挂载时那一份盘表与交进去的那一份"
        ),
        other => panic!(
            "{table_change:?}：盘表与挂载时的不同：在预演之前拒成盘表那一种，而交回 {other:?}"
        ),
    }
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.reads.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0, 0),
        "{table_change:?}：不预演：一块只进内存的盘都没建、一个盘都没读、一个改盘调用都没有"
    );
    assert!(
        counting_pool.image() == image_before,
        "{table_change:?}：盘逐字节不变"
    );
    restore_the_device_table_of(&mut counting_pool, table_change, device_left_out);
    ways_out_of(
        &counting_pool,
        &stuck,
        &format!("{table_change:?} 之后换回挂载时那一份盘表"),
    );
}

pub fn asking_for_the_ways_out_with_the_two_devices_in_reversed_order_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
        images,
        DeviceTableChange::Reversed,
    );
}

pub fn asking_for_the_ways_out_with_one_device_more_than_the_mount_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
        images,
        DeviceTableChange::OneDeviceMore,
    );
}

pub fn asking_for_the_ways_out_with_one_device_fewer_than_the_mount_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_ways_out_with_the_device_table_changed_are_refused_without_reading_a_device(
        images,
        DeviceTableChange::OneDeviceFewer,
    );
}

/// 推环入口先核盘表次序（用户 2026-10-10 弹窗定「也先比次序」）：被宽护卡住（建 inode、前缀 40 次那一形）之后，在这一刻盘面与会话的
/// 一份拷贝上照 `table_change` 换盘表推环：在任何读写之前拒成盘表那一种、带挂载时那一份与交进去的那一份，物理块大小没探、一个盘都没读、
/// 一个改盘调用都没有，盘逐字节不变、分配器与会话不动；换回挂载时那一份盘表照常推环做成（推到 R 那一槽）。不核这一判时，
/// 次序对调的盘表照样推成（攻方 `research/prompts/m3-batch10z-xy-code-r1-opus-output.md` 第二节：写的先后跟着盘表次序变）；
/// 多一块、少一块在核盘表那一套里读过盘才拒成 `DevicesHandedIn`。
pub fn assert_the_push_with_the_device_table_changed_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
    table_change: DeviceTableChange,
) {
    let PoolStuckBehindTheWideProtection { pool, stuck, .. } =
        pool_stuck_behind_the_wide_protection(
            images,
            FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
                overwrites_before: 40,
            },
        );
    let (mut counting_pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&pool, None);
    let device_table_of_the_mount = device_identities_in_the_order_handed_in(&counting_pool);
    let device_left_out =
        change_the_device_table_of(&mut counting_pool, table_change, &calls_of_the_pool);
    let device_table_handed_in = device_identities_in_the_order_handed_in(&counting_pool);
    let image_before = counting_pool.image();
    let allocator_before = format!("{:?}", counting_pool.session().allocator);
    let current_before = *counting_pool.session().current.root();
    match push_the_root_ring_of(&mut counting_pool) {
        Err(RootRingPushRefused::DeviceTableOtherThanTheOneOfTheMount {
            device_table_of_the_mount: refused_table_of_the_mount,
            device_table_handed_in: refused_table_handed_in,
        }) => assert_eq!(
            (refused_table_of_the_mount, refused_table_handed_in),
            (device_table_of_the_mount, device_table_handed_in),
            "{table_change:?}：带挂载时那一份盘表与交进去的那一份"
        ),
        other => panic!(
            "{table_change:?}：盘表与挂载时的不同：推环在任何读写之前拒成盘表那一种，而交回 {other:?}"
        ),
    }
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.reads.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0, 0),
        "{table_change:?}：在任何读写之前拒：物理块大小没探、一个盘都没读、一个改盘调用都没有"
    );
    assert!(
        counting_pool.image() == image_before,
        "{table_change:?}：盘逐字节不变"
    );
    assert_eq!(
        format!("{:?}", counting_pool.session().allocator),
        allocator_before,
        "{table_change:?}：分配器整份不变"
    );
    assert_eq!(
        *counting_pool.session().current.root(),
        current_before,
        "{table_change:?}：会话的现行那一版不变"
    );
    restore_the_device_table_of(&mut counting_pool, table_change, device_left_out);
    let pushed = push_the_root_ring_of(&mut counting_pool).unwrap_or_else(|refusal| {
        panic!("{table_change:?} 之后换回挂载时那一份盘表：推环照常做成，而被拒：{refusal:?}")
    });
    assert_eq!(
        u64::try_from(pushed.versions_published.len()).expect("几次"),
        stuck.empty_publishes_until_the_ring_slot_of_r_is_written,
        "{table_change:?} 之后换回挂载时那一份盘表：推到 R 那一槽，发的次数是报里的"
    );
    assert!(
        pushed.ring_slots_written.contains(&stuck.ring_slot_of_r),
        "{table_change:?} 之后换回挂载时那一份盘表：写掉了 R 那一槽：{:?}",
        pushed.ring_slots_written
    );
}

pub fn pushing_the_root_ring_with_a_device_table_other_than_the_mount_in_reversed_order_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_push_with_the_device_table_changed_is_refused_without_reading_a_device(
        images,
        DeviceTableChange::Reversed,
    );
}

pub fn pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_more_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_push_with_the_device_table_changed_is_refused_without_reading_a_device(
        images,
        DeviceTableChange::OneDeviceMore,
    );
}

pub fn pushing_the_root_ring_with_a_device_table_other_than_the_mount_with_one_device_fewer_is_refused_without_reading_a_device(
    images: &mut PoolCheckerImages,
) {
    assert_the_push_with_the_device_table_changed_is_refused_without_reading_a_device(
        images,
        DeviceTableChange::OneDeviceFewer,
    );
}

/// 会话已动（推过环，不在被拒的那一版上）又交次序对调的盘表要出路：先比版本、再比盘表，拒成 `SessionMoved…`、带被拒那一版与现行那一版的
/// 实例代号与 txg，不拒成盘表那一种（攻方 N1 / 正推 S2：盘表那一道挪到版本那一道之前时拒成盘表那一种，调用方换回盘表再要一次才拿到
/// `SessionMoved…`）。不预演：物理块大小没探、一个盘都没读、一个改盘调用都没有，盘逐字节不变。
pub fn asking_for_the_ways_out_after_the_session_moved_with_the_two_devices_in_reversed_order_is_refused_as_the_session_moved(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        mut pool, stuck, ..
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::InodeCreation {
            overwrites_before: 40,
        },
    );
    let refused_on = *pool.session().current.root();
    push_the_root_ring_of(&mut pool).unwrap_or_else(|refusal| panic!("推环做得成：{refusal:?}"));
    let (mut counting_pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&pool, None);
    let current_after_the_push = *counting_pool.session().current.root();
    assert!(
        current_after_the_push.checkpoint_txg > refused_on.checkpoint_txg,
        "推环之后会话已在后面的一版上（阳性对照）"
    );
    let device_left_out = change_the_device_table_of(
        &mut counting_pool,
        DeviceTableChange::Reversed,
        &calls_of_the_pool,
    );
    assert!(device_left_out.is_none(), "次序对调不少交盘");
    let image_before = counting_pool.image();
    match counting_pool
        .session()
        .ways_out_of_the_wide_protection(&counting_pool.devices, &stuck)
    {
        Err(
            WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version,
            },
        ) => assert_eq!(
            (
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version
            ),
            (
                refused_on.instance,
                refused_on.checkpoint_txg,
                current_after_the_push.instance,
                current_after_the_push.checkpoint_txg
            ),
            "带被拒那一版与现行那一版的实例代号与 txg"
        ),
        other => panic!(
            "会话已不在被拒的那一版上、盘表又次序对调：先比版本，拒成 SessionMoved…，而交回 {other:?}"
        ),
    }
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.reads.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0, 0),
        "不预演：物理块大小没探、一个盘都没读、一个改盘调用都没有"
    );
    assert!(counting_pool.image() == image_before, "盘逐字节不变");
}

/// 卸载再挂之后拿再挂之前的报要出路（被宽护卡住的回退那一形，同 `unmounting_and_mounting_again_after_the_wide_protection_stops_writes_takes_the_account_of_r_and_lets_writes_through`）：
/// 再挂取新实例，拒成 `SessionMoved…`，被拒那一版的两个数是报里的，现行那一版的实例代号与 txg 是再挂之后那一版的（新实例）
/// （正推 S4：现行那一版的实例代号取成报里的时照绿，入库用例里只有这一形分得开）。不预演：物理块大小没探、一个盘都没读、一个改盘调用都没有，
/// 盘逐字节不变。
pub fn asking_for_the_ways_out_after_unmounting_and_mounting_again_with_the_report_from_before_is_refused_as_the_session_moved_naming_the_new_instance(
    images: &mut PoolCheckerImages,
) {
    let PoolStuckBehindTheWideProtection {
        mut pool, stuck, ..
    } = pool_stuck_behind_the_wide_protection(
        images,
        FormStuckBehindTheWideProtectionOnThe384SlotDevices::RollbackSixVersionsBack {
            overwrites_before: OVERWRITES_BEFORE_THE_ROLLBACK_SIX_VERSIONS_BACK,
        },
    );
    let refused_on = *pool.session().current.root();
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
            for version in &raised.publishes {
                pool.note_root_of_the_current_content(&version.root);
            }
        }
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { current }) => {
            panic!("现行那一版带文件，卸载要推一串抬 F：{current:?}")
        }
        Err(refusal) => panic!("被宽护卡住时正常卸载做得成：{refusal:?}"),
    }
    pool.crash_and_mount_writable()
        .expect("卸载之后再可写挂载做得成（R 的账读得出）");
    let (counting_pool, calls_of_the_pool) =
        pool_failing_one_changing_call_copied_from(&pool, None);
    let current_after_mounting_again = *counting_pool.session().current.root();
    assert_ne!(
        current_after_mounting_again.instance, refused_on.instance,
        "再挂取新实例（阳性对照）"
    );
    let image_before = counting_pool.image();
    match counting_pool
        .session()
        .ways_out_of_the_wide_protection(&counting_pool.devices, &stuck)
    {
        Err(
            WaysOutOfTheWideProtectionRefused::SessionMovedPastTheVersionRefusedOnNotSupportedInTheFirstVersion {
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version,
            },
        ) => assert_eq!(
            (
                instance_refused_on,
                checkpoint_txg_refused_on,
                instance_of_the_current_version,
                checkpoint_txg_of_the_current_version
            ),
            (
                refused_on.instance,
                refused_on.checkpoint_txg,
                current_after_mounting_again.instance,
                current_after_mounting_again.checkpoint_txg
            ),
            "被拒那一版是报里的、现行那一版是再挂之后那一版（实例代号是新实例的）"
        ),
        other => panic!("卸载再挂之后拿再挂之前的报：拒成 SessionMoved…，而交回 {other:?}"),
    }
    assert_eq!(
        (
            calls_of_the_pool.probes_of_the_physical_block_size.get(),
            calls_of_the_pool.reads.get(),
            calls_of_the_pool.calls.get()
        ),
        (0, 0, 0),
        "不预演：物理块大小没探、一个盘都没读、一个改盘调用都没有"
    );
    assert!(counting_pool.image() == image_before, "盘逐字节不变");
}

impl DeviceOverASparseImage for DeviceReadingBackAllZeroOnRanges<SparseBlockDevice> {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

//! 草稿探测（K2/K3 正推腿），不入库。
#![allow(dead_code, unused_imports, clippy::all)]
mod common_unwitnessed_publish;

use std::cell::Cell;
use std::rc::Rc;

use common_unwitnessed_publish::{
    checker_violations_on, device_bytes_of_the_384_slot_small_devices,
    devices_reading_back_all_zero_on, image_of_devices_reading_back_all_zero,
    parameters_of_the_384_slot_small_devices, plain_device, plain_devices_on, pool_after_the_mount,
    record_one_publish_after_overwrites, root_is_in_the_ring, units_of_r_overwritten_on, DeviceOverASparseImage, PoolUnderTest, PublishRecordedAsR,
    RangesReadingBackAllZero, OVERWRITE_BYTES,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, SlotNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::{AllocationRecord, PlacementOnDevice, PoolAllocator};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::make_filesystem::MakeFilesystemParameters;
use singlefs_core::mount::{
    mount_writable, mount_writable_with_space_admission, mount_writable_with_test_only_switches,
    unmount, MountError, RollbackTarget, ShadowLedger, StillUnreadableAfterOneReread, Unmounted,
};
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::pointer::{LocationEntry, NodePointer};
use singlefs_core::root_record::RootRecord;
use singlefs_core::root_ring::{slot_offset, target_for_publish, RootRingSlot};
use singlefs_core::transaction::PoolVersion;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use std::collections::BTreeMap;

/// C 那一次覆盖写的内容长度（攻方原型的 2500 字节）：与之前之后的覆盖写长度不同，读回哪一版看得出。
const CONTENT_BYTES_OF_THE_ABANDONED_VERSION: usize = 2500;

/// 挂着时 C 的账读不出之后，会话至多覆盖写几次去等准入推出第一串抬 F（攻方原型覆盖写到 60 次；改之前推抬 F 的那一次写被拒）。
const OVERWRITES_WHILE_THE_ACCOUNT_IS_UNREADABLE_AT_MOST: usize = 60;

/// 准入推过第一串抬 F 之后再覆盖写几次（推过之后的写也不拒）。
const OVERWRITES_AFTER_THE_FIRST_FLOOR_RAISE: usize = 3;

/// 一份崩溃恢复抛弃了最新那条根 C 之后、进程退出的盘面（还没再挂）。
struct PoolWithAnAbandonedRoot {
    parameters: MakeFilesystemParameters,
    device_bytes: u64,
    image: MemoryPool,
    write_time_seconds: u64,
    abandoned_root: RootRecord,
    /// 实例 3 退出时现行那一版的根（按实例 3 的表有效）：下一次可写挂载择到它，根环表把它记成有效根。
    newest_valid_root: RootRecord,
}

fn read_bytes_of(
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

fn write_bytes_to(
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
fn pool_whose_newest_root_was_abandoned_by_a_crash_recovery() -> PoolWithAnAbandonedRoot {
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
struct ReadFaultOnChosenSlots {
    ranges: Rc<Vec<(DeviceIdentity, u64, u64)>>,
    is_active: Rc<Cell<bool>>,
    zero_mode: Rc<Cell<bool>>,
}

impl ReadFaultOnChosenSlots {
    /// 这条根住的那个根环槽（它的 txg 在根环里对应的那一槽，一个物理块）。故障先关着。
    fn on_the_root_ring_slot_of(root: &RootRecord, parameters: &MakeFilesystemParameters) -> Self {
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
            zero_mode: Rc::new(Cell::new(false)),
        }
    }

    /// 这条根的树表、中央映射树根、分配记录树根（空指针不算）的每条位置条目各一个槽。故障先关着。
    fn on_the_account_of(root: &RootRecord) -> Self {
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
            zero_mode: Rc::new(Cell::new(false)),
        }
    }

    fn refuses(&self, device: DeviceIdentity, offset_in_bytes: u64, length_in_bytes: u64) -> bool {
        self.is_active.get()
            && self.ranges.iter().any(|(range_device, start, end)| {
                *range_device == device
                    && offset_in_bytes < *end
                    && offset_in_bytes + length_in_bytes > *start
            })
    }
}

struct DeviceWithReadFaultsOnChosenSlots {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    fault: ReadFaultOnChosenSlots,
}

impl BlockDevice for DeviceWithReadFaultsOnChosenSlots {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let length_in_bytes = u64::try_from(buffer.len()).expect("读长装得进 u64");
        if self.fault.refuses(self.identity, offset.0, length_in_bytes) {
            if self.fault.zero_mode.get() {
                buffer.fill(0);
                return Ok(());
            }
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

fn devices_with_the_read_fault_on(
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
fn pool_under_test_after_the_mount(
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
fn mounted_while_the_abandoned_root_reads(
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

/// 抬 F 之后 `placements` 里每个落点在分配器里都不空闲：隔离着，或现行账里还分配着。被抛弃根离开根环之前它引用的单元不许发出去
/// （D23（journal 的角色与格式） 已定项 14 回退段主句）；抬 F 越过一条候选根之后，此前靠它豁免、现行账里已释放的落点只靠影子账这一遍补的隔离护着。
fn assert_none_of_the_placements_is_free(
    allocator: &PoolAllocator,
    placements: &[PlacementOnDevice],
    whose: &str,
) {
    let free_placements: Vec<&PlacementOnDevice> = placements
        .iter()
        .filter(|placement| {
            allocator
                .devices
                .iter()
                .find(|device_map| device_map.device == placement.device)
                .expect("池里有这块盘")
                .is_free(placement.slot)
        })
        .collect();
    assert!(
        free_placements.is_empty(),
        "抬 F 之后{whose}的落点一槽都不空闲（隔离着或现行账里还分配着）：空闲的 {free_placements:?}（共 {} 个落点）",
        placements.len()
    );
}

/// 故障开着之后 C 的树表那一槽确实读不出（阳性对照：用例不是在读得出的盘上空转）。
fn assert_the_tree_table_of_the_abandoned_root_is_unreadable(
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

fn violations_of_unit_reuse_on(image: &MemoryPool) -> Vec<String> {
    checker_violations_on(image)
        .into_iter()
        .filter(|violation| violation.starts_with("I-7.4") || violation.starts_with("I-4.8"))
        .collect()
}

/// 正常卸载一次（影子账开着），要它做成（推一串抬 F 到现行那一版）。交回这一串的 `abandoned_roots_unreadable` 与卸载之后的分配器。
/// `why` 只用在断言消息里，说是哪一形。
fn unmount_expecting_the_floor_raised(
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

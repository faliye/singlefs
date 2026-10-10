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

// ---------------------------------------------------------------- 探测代码

fn free_among(allocator: &PoolAllocator, placements: &[PlacementOnDevice]) -> usize {
    placements
        .iter()
        .filter(|p| {
            let map = allocator
                .devices
                .iter()
                .find(|d| d.device == p.device)
                .expect("池里有这块盘");
            (0..p.span).any(|k| map.is_free(SlotNumber(p.slot.0 + k)))
        })
        .count()
}

fn merged(a: &ReadFaultOnChosenSlots, b: &ReadFaultOnChosenSlots) -> ReadFaultOnChosenSlots {
    let mut ranges: Vec<(DeviceIdentity, u64, u64)> = (*a.ranges).clone();
    ranges.extend((*b.ranges).iter().copied());
    ReadFaultOnChosenSlots {
        ranges: Rc::new(ranges),
        is_active: Rc::new(Cell::new(false)),
        zero_mode: Rc::new(Cell::new(false)),
    }
}

fn i74_i48(image: &MemoryPool) -> Vec<String> {
    violations_of_unit_reuse_on(image)
}

fn short(s: String) -> String {
    s.chars().take(300).collect()
}

fn unmount_result(
    pool: &mut PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>,
) -> (String, Option<PoolAllocator>) {
    let mut session = pool.session.take().expect("会话");
    let r = unmount(
        &pool.parameters,
        &mut pool.devices,
        &mut session.allocator,
        &mut session.current,
        ShadowLedger::On,
    );
    match r {
        Ok(Unmounted::FloorRaisedToTheCurrentVersion(raised)) => (
            format!("做成(计数{})", raised.abandoned_roots_unreadable),
            Some(session.allocator),
        ),
        Ok(Unmounted::NothingWrittenOnAVersionWithoutFile { .. }) => {
            ("无文件版本未写".to_string(), Some(session.allocator))
        }
        Err(e) => (short(format!("拒 {e:?}")), None),
    }
}

/// K2-1 / K2-3：被抛弃根 C 的 H1 / 全零 / 错误 各形。
/// kind: ring（根槽故障）/ acct（账故障）/ both
fn h1_run(
    label: &str,
    ring_slot_fault: bool,
    account_fault: bool,
    zero: bool,
    fault_at_mount: bool,
    lift_after_mount: bool,
    max_steps: usize,
    fill: bool,
) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    // 参照：无故障挂载记着的 C 的落点
    let reference: Vec<PlacementOnDevice> = {
        let f = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
        let (_p, rec) = mounted_while_the_abandoned_root_reads(&built, &f);
        rec
    };
    let ring = ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let acct = ReadFaultOnChosenSlots::on_the_account_of(&built.abandoned_root);
    let fault = match (ring_slot_fault, account_fault) {
        (true, true) => merged(&ring, &acct),
        (true, false) => ring,
        (false, true) => acct,
        (false, false) => merged(&ring, &acct),
    };
    fault.zero_mode.set(zero);
    fault.is_active.set(fault_at_mount);
    let mut devices = devices_with_the_read_fault_on(&built.image, &fault);
    let mounted = match mount_writable_with_space_admission(
        &built.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
    ) {
        Ok(m) => m,
        Err(e) => {
            println!("RESULT {label}: 可写挂载被拒: {}", short(format!("{e:?}")));
            return;
        }
    };
    let occ = mounted.allocator.root_ring_occupancy().expect("根环表");
    let recorded_len = occ
        .placements_recorded_for_the_abandoned_root(built.abandoned_root.instance, built.abandoned_root.checkpoint_txg)
        .map(|v| v.len());
    let free_after_mount = free_among(&mounted.allocator, &reference);
    let mut pool = pool_under_test_after_the_mount(
        &built.parameters,
        built.device_bytes,
        devices,
        mounted,
        built.write_time_seconds + 1000,
    );
    let any_fault = ring_slot_fault || account_fault;
    fault.is_active.set(any_fault && !lift_after_mount);
    let mut lifted_at: Option<usize> = None;
    let mut refused: Option<(usize, String)> = None;
    let mut raises = 0usize;
    let mut first_violation: Option<(usize, Vec<String>)> = None;
    let mut first_free: Option<(usize, usize)> = None;
    let mut steps_done = 0usize;
    let mut hazard_steps: Vec<(usize, usize)> = Vec::new();
    for index in 0..max_steps {
        if lifted_at.is_none() && fault.is_active.get() {
            let still = pool
                .session()
                .allocator
                .root_ring_occupancy()
                .map(|o| o.placements_recorded_for_the_abandoned_root(built.abandoned_root.instance, built.abandoned_root.checkpoint_txg).is_some())
                .unwrap_or(false);
            if !still {
                fault.is_active.set(false);
                lifted_at = Some(index);
            }
        }
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(p) => {
                steps_done = index + 1;
                raises += p.floor_raises.len();
                let fr = free_among(&pool.session().allocator, &reference);
                let still_now = pool
                    .session()
                    .allocator
                    .root_ring_occupancy()
                    .map(|o| o.placements_recorded_for_the_abandoned_root(built.abandoned_root.instance, built.abandoned_root.checkpoint_txg).is_some())
                    .unwrap_or(false);
                if still_now && fr > 0 {
                    hazard_steps.push((index + 1, fr));
                }
                if first_free.is_none() && fr > 0 && !p.floor_raises.is_empty() {
                    first_free = Some((index + 1, fr));
                }
                if first_violation.is_none() && index % 3 == 2 {
                    let v = i74_i48(&pool.image());
                    if !v.is_empty() {
                        first_violation = Some((index + 1, v));
                    }
                }
            }
            Err(e) => {
                refused = Some((index + 1, short(format!("{e:?}"))));
                break;
            }
        }
    }
    let mut fill_note = String::from("未做");
    if fill {
        let payload = singlefs_core::unit::data_unit_payload_capacity();
        let mut n = 2usize;
        let mut last_ok = 0usize;
        loop {
            match pool.sequential_write(n * payload) {
                Ok(_) => { last_ok = n; n += 2; }
                Err(e) => { fill_note = format!("顺序写{}个数据单元后第{}个被拒:{}", last_ok, n, short(format!("{e:?}"))); break; }
            }
            if n > 400 { fill_note = format!("到{}个数据单元仍未拒", last_ok); break; }
        }
    }
    let table_still = pool
        .session
        .as_ref()
        .and_then(|s| s.allocator.root_ring_occupancy())
        .map(|o| o.placements_recorded_for_the_abandoned_root(built.abandoned_root.instance, built.abandoned_root.checkpoint_txg).is_some());
    let v_writes = i74_i48(&pool.image());
    let free_after_writes = free_among(&pool.session().allocator, &reference);
    let (un, alloc_after) = unmount_result(&mut pool);
    let v_un = i74_i48(&pool.image());
    let free_after_un = alloc_after.map(|a| free_among(&a, &reference));
    println!(
        "RESULT {label}: 参照落点数={} 挂载记着={:?} 挂后空闲={free_after_mount} | 会话写至多{max_steps}:完成{steps_done}次 被拒={:?} 推抬F串={raises} 表里还记着C时空闲>0的步={hazard_steps:?} 写满={fill_note} 故障在C被覆写后撤掉于第{lifted_at:?}步 表里还记着C={:?} 首次推抬F后空闲={:?} 逐步首个违例={:?} 空闲(写后)={free_after_writes} 违例(写后)={:?} | 卸载={un} 空闲(卸载后)={:?} 违例(卸载后)={:?}",
        reference.len(), recorded_len, refused, table_still, first_free, first_violation, v_writes, free_after_un, v_un
    );
}

#[test]
fn h1_ring_error_after_mount() { h1_run("H1 根槽读错(挂后)", true, false, false, false, false, 60, true); }
#[test]
fn h1_ring_zero_after_mount() { h1_run("H1 根槽读回全零(挂后)", true, false, true, false, false, 60, true); }
#[test]
fn h1_ring_and_account_error_after_mount() { h1_run("H1 根槽+账读错(挂后)", true, true, false, false, false, 60, true); }
#[test]
fn h1_account_error_after_mount() { h1_run("账读错(挂后,今天已有的10y形)", false, true, false, false, false, 60, true); }
#[test]
fn k23_ring_zero_at_mount_then_lifted() { h1_run("K2-3 根槽挂载时读回全零(之后恢复)", true, false, true, true, true, 60, true); }
#[test]
fn k23_ring_zero_at_mount_stays() { h1_run("K2-3 根槽挂载时读回全零(一直全零)", true, false, true, true, false, 60, true); }
#[test]
fn k23_ring_error_at_mount_then_lifted() { h1_run("K2-3b 根槽挂载时读错(之后恢复)", true, false, false, true, true, 60, true); }
#[test]
fn k23_ring_error_at_mount_stays() { h1_run("K2-3b 根槽挂载时读错(一直读错)", true, false, false, true, false, 60, true); }
#[test]
fn k23_ring_and_account_zero_at_mount_then_lifted() { h1_run("K2-3c 根槽+账挂载时读回全零(之后恢复)", true, true, true, true, true, 60, true); }
#[test]
fn k23_baseline_no_fault() { h1_run("基线 无故障", false, false, false, false, false, 60, true); }

// ---------------------------------------------------------------- H2 扫描

use singlefs_harness::memory_pool::RetainedWrite;

#[derive(Clone, Copy, Debug)]
enum H2Fault {
    Nothing,
    Account,
    RootSlot,
}

fn cells_of_writes(writes: &[&RetainedWrite]) -> Vec<PlacementOnDevice> {
    writes
        .iter()
        .map(|w| PlacementOnDevice {
            device: w.device,
            slot: SlotNumber(w.offset.0 / singlefs_format::SLOT_BYTES),
            span: w.length_in_bytes().div_ceil(singlefs_format::SLOT_BYTES),
        })
        .collect()
}

fn covered_by(p: &PlacementOnDevice, set: &[PlacementOnDevice]) -> bool {
    set.iter().any(|q| q.device == p.device && p.slot.0 >= q.slot.0 && p.slot.0 < q.slot.0 + q.span)
}

fn h2_run(
    label: &str,
    kind: PublishRecordedAsR,
    overwrites_before: u64,
    zero_group: usize,
    fault_kind: H2Fault,
    max_steps: usize,
    fill: bool,
    extra: &str,
) {
    let recorded = record_one_publish_after_overwrites(overwrites_before, kind);
    let groups = recorded.unit_writes_grouped_by_unit();
    if zero_group >= groups.len() {
        println!("RESULT {label}: 组{zero_group} 超出(共{}组)", groups.len());
        return;
    }
    let root_of_r = recorded.root();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut ranges = RangesReadingBackAllZero::default().with_the_writes(&groups[zero_group]);
    if extra == "az" || extra == "bz" {
        let r = recorded.root();
        for pointer in [r.tree_table, r.mapping_root, r.allocation_record_tree_root] {
            if pointer != NodePointer::empty_root() {
                ranges = ranges.with_the_writes(&recorded.unit_writes_at(&pointer));
            }
        }
    }
    if extra == "rz" || extra == "bz" {
        for ordinal in 1..=recorded.record_count() {
            ranges = ranges.with_the_writes(&recorded.record_writes_of_the_record(ordinal));
        }
    }
    let r_cells = cells_of_writes(&recorded.unit_writes());
    let mut mount_devices = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = match mount_writable_with_space_admission(
        &recorded.pool.parameters,
        &mut mount_devices,
        SpaceAdmission::JudgedByTheFormula,
    ) {
        Ok(m) => m,
        Err(e) => {
            println!("RESULT {label}: M1 被拒 {}", short(format!("{e:?}")));
            return;
        }
    };
    let selected = (
        mounted.output.effective_root.instance,
        mounted.output.effective_root.checkpoint_txg,
    );
    let selected_is_p = selected == recorded.previous_key;
    let occ = mounted.allocator.root_ring_occupancy().expect("根环表");
    let rec_abandoned = occ
        .placements_recorded_for_the_abandoned_root(root_of_r.instance, root_of_r.checkpoint_txg)
        .map(|v| v.len());
    let c_valid = occ
        .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
            root_of_r.instance,
            root_of_r.checkpoint_txg,
        );
    let c_len = c_valid.as_ref().map(|v| v.len());
    let c_set = c_valid.clone().unwrap_or_default();
    let not_in_c: usize = r_cells.iter().filter(|p| !covered_by(p, &c_set)).count();
    let free_after_mount = free_among(&mounted.allocator, &r_cells);
    let image_after_the_mount =
        image_of_devices_reading_back_all_zero(&mount_devices, recorded.pool.device_bytes);
    let fault = match fault_kind {
        H2Fault::Nothing | H2Fault::Account => ReadFaultOnChosenSlots::on_the_account_of(&root_of_r),
        H2Fault::RootSlot => {
            ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&root_of_r, &recorded.pool.parameters)
        }
    };
    let devices = devices_with_the_read_fault_on(&image_after_the_mount, &fault);
    let mut pool = pool_under_test_after_the_mount(
        &recorded.pool.parameters,
        recorded.pool.device_bytes,
        devices,
        mounted,
        recorded.pool.write_time_seconds + 10_000,
    );
    fault.is_active.set(!matches!(fault_kind, H2Fault::Nothing));
    let mut refused: Option<(usize, String)> = None;
    let mut raises = 0usize;
    let mut first_violation: Option<(usize, Vec<String>)> = None;
    let mut hazard: Vec<(usize, usize, usize)> = Vec::new(); // (步, R 在根环里时空闲数, 被覆盖数)
    let mut steps_done = 0;
    for index in 0..max_steps {
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(p) => {
                steps_done = index + 1;
                raises += p.floor_raises.len();
                let fr = free_among(&pool.session().allocator, &r_cells);
                if index % 3 == 2 || fr > 0 {
                    let image = pool.image();
                    let in_ring = root_is_in_the_ring(&image, recorded.key);
                    if in_ring {
                        let over = units_of_r_overwritten_on(&recorded, &image).len();
                        if fr > 0 || over > 0 {
                            hazard.push((index + 1, fr, over));
                        }
                    }
                    if first_violation.is_none() && index % 3 == 2 {
                        let v = i74_i48(&image);
                        if !v.is_empty() {
                            first_violation = Some((index + 1, v));
                        }
                    }
                }
            }
            Err(e) => {
                refused = Some((index + 1, short(format!("{e:?}")).chars().take(150).collect::<String>()));
                break;
            }
        }
    }
    let mut fill_note = String::from("未做");
    if fill && refused.is_none() {
        let payload = singlefs_core::unit::data_unit_payload_capacity();
        let mut n = 2usize;
        let mut last_ok = 0usize;
        loop {
            match pool.sequential_write(n * payload) {
                Ok(_) => { last_ok = n; n += 2; }
                Err(e) => { fill_note = format!("顺序写{last_ok}个数据单元后第{n}个被拒:{}", short(format!("{e:?}")).chars().take(60).collect::<String>()); break; }
            }
            if n > 400 { fill_note = format!("到{last_ok}仍未拒"); break; }
        }
    }
    let image_w = pool.image();
    let in_ring_w = root_is_in_the_ring(&image_w, recorded.key);
    let over_w = units_of_r_overwritten_on(&recorded, &image_w).len();
    let v_w = i74_i48(&image_w);
    let (un, alloc_after) = unmount_result(&mut pool);
    let image_u = pool.image();
    let in_ring_u = root_is_in_the_ring(&image_u, recorded.key);
    let over_u = units_of_r_overwritten_on(&recorded, &image_u).len();
    let v_u = i74_i48(&image_u);
    let free_u = alloc_after.map(|a| free_among(&a, &r_cells));
    println!(
        "RESULT {label}[组{zero_group}/{}组,R单元{}个,不在C里{not_in_c}]: 择P={selected_is_p} 表记被抛弃落点={rec_abandoned:?} C借来={c_len:?} 挂后R单元空闲={free_after_mount} | 会话写{steps_done}/{max_steps} 被拒={refused:?} 推抬F串={raises} R在环里时空闲/被覆盖的步={hazard:?} 逐步首违例={first_violation:?} 写满={fill_note} R在环里(写后)={in_ring_w} 被覆盖(写后)={over_w} 违例(写后)={v_w:?} | 卸载={un} R在环里={in_ring_u} 空闲(卸载后)={free_u:?} 被覆盖={over_u} 违例(卸载后)={v_u:?}",
        groups.len(),
        r_cells.len()
    );
}

macro_rules! h2_tests {
    ($($name:ident: $kind:expr, $fault:expr, $ob:expr, $extra:expr;)*) => {$(
        mod $name {
            use super::*;
            pub fn go(g: usize) { h2_run(stringify!($name), $kind, $ob, g, $fault, 45, false, $extra); }
        }
    )*};
}
h2_tests! {
    ow_none: PublishRecordedAsR::Overwrite, H2Fault::Nothing, 5, "";
    ow_acct: PublishRecordedAsR::Overwrite, H2Fault::Account, 5, "";
    ow_root: PublishRecordedAsR::Overwrite, H2Fault::RootSlot, 5, "";
    sq_none: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::Nothing, 5, "";
    sq_acct: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::Account, 5, "";
    sq_root: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::RootSlot, 5, "";
    ow_az: PublishRecordedAsR::Overwrite, H2Fault::Account, 5, "az";
    sq_az: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::Account, 5, "az";
    ow_rz: PublishRecordedAsR::Overwrite, H2Fault::Account, 5, "rz";
    sq_rz: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::Account, 5, "rz";
    ow_bz: PublishRecordedAsR::Overwrite, H2Fault::Account, 5, "bz";
    sq_bz: PublishRecordedAsR::SequentialWriteOfThreeDataUnits, H2Fault::Account, 5, "bz";
}
#[test]
fn h2_ow_none_g00() { ow_none::go(0); }
#[test]
fn h2_ow_none_g01() { ow_none::go(1); }
#[test]
fn h2_ow_none_g02() { ow_none::go(2); }
#[test]
fn h2_ow_none_g03() { ow_none::go(3); }
#[test]
fn h2_ow_none_g04() { ow_none::go(4); }
#[test]
fn h2_ow_none_g05() { ow_none::go(5); }
#[test]
fn h2_ow_none_g06() { ow_none::go(6); }
#[test]
fn h2_ow_none_g07() { ow_none::go(7); }
#[test]
fn h2_ow_none_g08() { ow_none::go(8); }
#[test]
fn h2_ow_none_g09() { ow_none::go(9); }
#[test]
fn h2_ow_none_g10() { ow_none::go(10); }
#[test]
fn h2_ow_none_g11() { ow_none::go(11); }
#[test]
fn h2_ow_none_g12() { ow_none::go(12); }
#[test]
fn h2_ow_none_g13() { ow_none::go(13); }
#[test]
fn h2_ow_acct_g00() { ow_acct::go(0); }
#[test]
fn h2_ow_acct_g01() { ow_acct::go(1); }
#[test]
fn h2_ow_acct_g02() { ow_acct::go(2); }
#[test]
fn h2_ow_acct_g03() { ow_acct::go(3); }
#[test]
fn h2_ow_acct_g04() { ow_acct::go(4); }
#[test]
fn h2_ow_acct_g05() { ow_acct::go(5); }
#[test]
fn h2_ow_acct_g06() { ow_acct::go(6); }
#[test]
fn h2_ow_acct_g07() { ow_acct::go(7); }
#[test]
fn h2_ow_acct_g08() { ow_acct::go(8); }
#[test]
fn h2_ow_acct_g09() { ow_acct::go(9); }
#[test]
fn h2_ow_acct_g10() { ow_acct::go(10); }
#[test]
fn h2_ow_acct_g11() { ow_acct::go(11); }
#[test]
fn h2_ow_acct_g12() { ow_acct::go(12); }
#[test]
fn h2_ow_acct_g13() { ow_acct::go(13); }
#[test]
fn h2_ow_root_g00() { ow_root::go(0); }
#[test]
fn h2_ow_root_g01() { ow_root::go(1); }
#[test]
fn h2_ow_root_g02() { ow_root::go(2); }
#[test]
fn h2_ow_root_g03() { ow_root::go(3); }
#[test]
fn h2_ow_root_g04() { ow_root::go(4); }
#[test]
fn h2_ow_root_g05() { ow_root::go(5); }
#[test]
fn h2_ow_root_g06() { ow_root::go(6); }
#[test]
fn h2_ow_root_g07() { ow_root::go(7); }
#[test]
fn h2_ow_root_g08() { ow_root::go(8); }
#[test]
fn h2_ow_root_g09() { ow_root::go(9); }
#[test]
fn h2_ow_root_g10() { ow_root::go(10); }
#[test]
fn h2_ow_root_g11() { ow_root::go(11); }
#[test]
fn h2_ow_root_g12() { ow_root::go(12); }
#[test]
fn h2_ow_root_g13() { ow_root::go(13); }
#[test]
fn h2_sq_none_g00() { sq_none::go(0); }
#[test]
fn h2_sq_none_g01() { sq_none::go(1); }
#[test]
fn h2_sq_none_g02() { sq_none::go(2); }
#[test]
fn h2_sq_none_g03() { sq_none::go(3); }
#[test]
fn h2_sq_none_g04() { sq_none::go(4); }
#[test]
fn h2_sq_none_g05() { sq_none::go(5); }
#[test]
fn h2_sq_none_g06() { sq_none::go(6); }
#[test]
fn h2_sq_none_g07() { sq_none::go(7); }
#[test]
fn h2_sq_none_g08() { sq_none::go(8); }
#[test]
fn h2_sq_none_g09() { sq_none::go(9); }
#[test]
fn h2_sq_none_g10() { sq_none::go(10); }
#[test]
fn h2_sq_none_g11() { sq_none::go(11); }
#[test]
fn h2_sq_none_g12() { sq_none::go(12); }
#[test]
fn h2_sq_none_g13() { sq_none::go(13); }
#[test]
fn h2_sq_acct_g00() { sq_acct::go(0); }
#[test]
fn h2_sq_acct_g01() { sq_acct::go(1); }
#[test]
fn h2_sq_acct_g02() { sq_acct::go(2); }
#[test]
fn h2_sq_acct_g03() { sq_acct::go(3); }
#[test]
fn h2_sq_acct_g04() { sq_acct::go(4); }
#[test]
fn h2_sq_acct_g05() { sq_acct::go(5); }
#[test]
fn h2_sq_acct_g06() { sq_acct::go(6); }
#[test]
fn h2_sq_acct_g07() { sq_acct::go(7); }
#[test]
fn h2_sq_acct_g08() { sq_acct::go(8); }
#[test]
fn h2_sq_acct_g09() { sq_acct::go(9); }
#[test]
fn h2_sq_acct_g10() { sq_acct::go(10); }
#[test]
fn h2_sq_acct_g11() { sq_acct::go(11); }
#[test]
fn h2_sq_acct_g12() { sq_acct::go(12); }
#[test]
fn h2_sq_acct_g13() { sq_acct::go(13); }
#[test]
fn h2_sq_root_g00() { sq_root::go(0); }
#[test]
fn h2_sq_root_g01() { sq_root::go(1); }
#[test]
fn h2_sq_root_g02() { sq_root::go(2); }
#[test]
fn h2_sq_root_g03() { sq_root::go(3); }
#[test]
fn h2_sq_root_g04() { sq_root::go(4); }
#[test]
fn h2_sq_root_g05() { sq_root::go(5); }
#[test]
fn h2_sq_root_g06() { sq_root::go(6); }
#[test]
fn h2_sq_root_g07() { sq_root::go(7); }
#[test]
fn h2_sq_root_g08() { sq_root::go(8); }
#[test]
fn h2_sq_root_g09() { sq_root::go(9); }
#[test]
fn h2_sq_root_g10() { sq_root::go(10); }
#[test]
fn h2_sq_root_g11() { sq_root::go(11); }
#[test]
fn h2_sq_root_g12() { sq_root::go(12); }
#[test]
fn h2_sq_root_g13() { sq_root::go(13); }
#[test]
fn h2_ow_az_g00() { ow_az::go(0); }
#[test]
fn h2_ow_az_g01() { ow_az::go(1); }
#[test]
fn h2_ow_az_g02() { ow_az::go(2); }
#[test]
fn h2_ow_az_g03() { ow_az::go(3); }
#[test]
fn h2_ow_az_g04() { ow_az::go(4); }
#[test]
fn h2_ow_az_g05() { ow_az::go(5); }
#[test]
fn h2_ow_az_g06() { ow_az::go(6); }
#[test]
fn h2_ow_az_g07() { ow_az::go(7); }
#[test]
fn h2_ow_az_g08() { ow_az::go(8); }
#[test]
fn h2_ow_az_g09() { ow_az::go(9); }
#[test]
fn h2_ow_az_g10() { ow_az::go(10); }
#[test]
fn h2_ow_az_g11() { ow_az::go(11); }
#[test]
fn h2_ow_az_g12() { ow_az::go(12); }
#[test]
fn h2_ow_az_g13() { ow_az::go(13); }
#[test]
fn h2_sq_az_g00() { sq_az::go(0); }
#[test]
fn h2_sq_az_g01() { sq_az::go(1); }
#[test]
fn h2_sq_az_g02() { sq_az::go(2); }
#[test]
fn h2_sq_az_g03() { sq_az::go(3); }
#[test]
fn h2_sq_az_g04() { sq_az::go(4); }
#[test]
fn h2_sq_az_g05() { sq_az::go(5); }
#[test]
fn h2_sq_az_g06() { sq_az::go(6); }
#[test]
fn h2_sq_az_g07() { sq_az::go(7); }
#[test]
fn h2_sq_az_g08() { sq_az::go(8); }
#[test]
fn h2_sq_az_g09() { sq_az::go(9); }
#[test]
fn h2_sq_az_g10() { sq_az::go(10); }
#[test]
fn h2_sq_az_g11() { sq_az::go(11); }
#[test]
fn h2_sq_az_g12() { sq_az::go(12); }
#[test]
fn h2_sq_az_g13() { sq_az::go(13); }
#[test]
fn h2_ow_rz_g00() { ow_rz::go(0); }
#[test]
fn h2_ow_rz_g01() { ow_rz::go(1); }
#[test]
fn h2_ow_rz_g02() { ow_rz::go(2); }
#[test]
fn h2_ow_rz_g03() { ow_rz::go(3); }
#[test]
fn h2_ow_rz_g04() { ow_rz::go(4); }
#[test]
fn h2_ow_rz_g05() { ow_rz::go(5); }
#[test]
fn h2_ow_rz_g06() { ow_rz::go(6); }
#[test]
fn h2_ow_rz_g07() { ow_rz::go(7); }
#[test]
fn h2_ow_rz_g08() { ow_rz::go(8); }
#[test]
fn h2_ow_rz_g09() { ow_rz::go(9); }
#[test]
fn h2_ow_rz_g10() { ow_rz::go(10); }
#[test]
fn h2_ow_rz_g11() { ow_rz::go(11); }
#[test]
fn h2_ow_rz_g12() { ow_rz::go(12); }
#[test]
fn h2_ow_rz_g13() { ow_rz::go(13); }
#[test]
fn h2_sq_rz_g00() { sq_rz::go(0); }
#[test]
fn h2_sq_rz_g01() { sq_rz::go(1); }
#[test]
fn h2_sq_rz_g02() { sq_rz::go(2); }
#[test]
fn h2_sq_rz_g03() { sq_rz::go(3); }
#[test]
fn h2_sq_rz_g04() { sq_rz::go(4); }
#[test]
fn h2_sq_rz_g05() { sq_rz::go(5); }
#[test]
fn h2_sq_rz_g06() { sq_rz::go(6); }
#[test]
fn h2_sq_rz_g07() { sq_rz::go(7); }
#[test]
fn h2_sq_rz_g08() { sq_rz::go(8); }
#[test]
fn h2_sq_rz_g09() { sq_rz::go(9); }
#[test]
fn h2_sq_rz_g10() { sq_rz::go(10); }
#[test]
fn h2_sq_rz_g11() { sq_rz::go(11); }
#[test]
fn h2_sq_rz_g12() { sq_rz::go(12); }
#[test]
fn h2_sq_rz_g13() { sq_rz::go(13); }
#[test]
fn h2_ow_bz_g00() { ow_bz::go(0); }
#[test]
fn h2_ow_bz_g01() { ow_bz::go(1); }
#[test]
fn h2_ow_bz_g02() { ow_bz::go(2); }
#[test]
fn h2_ow_bz_g03() { ow_bz::go(3); }
#[test]
fn h2_ow_bz_g04() { ow_bz::go(4); }
#[test]
fn h2_ow_bz_g05() { ow_bz::go(5); }
#[test]
fn h2_ow_bz_g06() { ow_bz::go(6); }
#[test]
fn h2_ow_bz_g07() { ow_bz::go(7); }
#[test]
fn h2_ow_bz_g08() { ow_bz::go(8); }
#[test]
fn h2_ow_bz_g09() { ow_bz::go(9); }
#[test]
fn h2_ow_bz_g10() { ow_bz::go(10); }
#[test]
fn h2_ow_bz_g11() { ow_bz::go(11); }
#[test]
fn h2_ow_bz_g12() { ow_bz::go(12); }
#[test]
fn h2_ow_bz_g13() { ow_bz::go(13); }
#[test]
fn h2_sq_bz_g00() { sq_bz::go(0); }
#[test]
fn h2_sq_bz_g01() { sq_bz::go(1); }
#[test]
fn h2_sq_bz_g02() { sq_bz::go(2); }
#[test]
fn h2_sq_bz_g03() { sq_bz::go(3); }
#[test]
fn h2_sq_bz_g04() { sq_bz::go(4); }
#[test]
fn h2_sq_bz_g05() { sq_bz::go(5); }
#[test]
fn h2_sq_bz_g06() { sq_bz::go(6); }
#[test]
fn h2_sq_bz_g07() { sq_bz::go(7); }
#[test]
fn h2_sq_bz_g08() { sq_bz::go(8); }
#[test]
fn h2_sq_bz_g09() { sq_bz::go(9); }
#[test]
fn h2_sq_bz_g10() { sq_bz::go(10); }
#[test]
fn h2_sq_bz_g11() { sq_bz::go(11); }
#[test]
fn h2_sq_bz_g12() { sq_bz::go(12); }
#[test]
fn h2_sq_bz_g13() { sq_bz::go(13); }

// ---------------------------------------------------------------- K1-1：管理员回退 / 公开上限函数

fn k11_setup(fault_on: bool) -> (PoolWithAnAbandonedRoot, PoolUnderTest<DeviceWithReadFaultsOnChosenSlots>, ReadFaultOnChosenSlots) {
    let built = pool_whose_newest_root_was_abandoned_by_a_crash_recovery();
    let fault = ReadFaultOnChosenSlots::on_the_root_ring_slot_of(&built.abandoned_root, &built.parameters);
    let (pool, _rec) = mounted_while_the_abandoned_root_reads(&built, &fault);
    fault.is_active.set(fault_on);
    (built, pool, fault)
}

#[test]
fn k11_admin_rollback_with_abandoned_slot_bad() {
    for fault_on in [false, true] {
        let (built, mut pool, _fault) = k11_setup(fault_on);
        let mut session = pool.session.take().expect("会话");
        let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
        let target = singlefs_core::mount::RollbackTarget {
            instance: built.newest_valid_root.instance,
            checkpoint_txg: built.newest_valid_root.checkpoint_txg,
        };
        let r = singlefs_core::mount::roll_back_by_a_forward_publish(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            target,
        );
        println!("RESULT k11-admin-rollback fault_on={fault_on}: {}", short(format!("{:?}", r.map(|_| ()))));
    }
}

#[test]
fn k11_public_ceiling_function_with_abandoned_slot_bad() {
    for fault_on in [false, true] {
        let (_built, mut pool, _fault) = k11_setup(fault_on);
        let mut session = pool.session.take().expect("会话");
        let PoolVersion::WithFile(current) = &mut session.current else { panic!("带文件") };
        let configuration = singlefs_core::recovery::choose_system_configuration(&pool.devices).expect("系统配置");
        let table = singlefs_core::recovery::instance_table_chain_of_root(&pool.devices, &current.root)
            .expect("表")
            .records;
        let r = singlefs_core::mount::rollback_floor_ceiling(
            &pool.devices,
            &configuration,
            current.root.rollback_floor,
            &table,
            &singlefs_core::mount::ring_slots_known_to_hold_a_root_by(&session.allocator),
        );
        println!("RESULT k11-public-rollback_floor_ceiling(known slots, 与 checker 档 on_device_modes.rs 同一写法) fault_on={fault_on}: {}", short(format!("{r:?}")));
        let floor_now = current.root.rollback_floor;
        let r2 = singlefs_core::mount::raise_rollback_floor(
            &pool.parameters,
            &mut pool.devices,
            &mut session.allocator,
            current,
            r.as_ref().map(|c| *c).unwrap_or(floor_now),
            ShadowLedger::On,
        );
        println!("RESULT k11-raise_rollback_floor(到上面的上限) fault_on={fault_on}: {}", short(format!("{:?}", r2.map(|x| x.abandoned_roots_unreadable))));
    }
}

#[test]
fn print_keys_of_p_and_r() {
    for kind in [PublishRecordedAsR::Overwrite, PublishRecordedAsR::SequentialWriteOfThreeDataUnits] {
        let recorded = record_one_publish_after_overwrites(5, kind);
        println!("RESULT keys {kind:?}: P={:?} R={:?}", recorded.previous_key, recorded.key);
    }
}

#[test]
fn print_instance_table_after_m1_bz_and_az() {
    for extra in ["az", "bz"] {
        let recorded = record_one_publish_after_overwrites(5, PublishRecordedAsR::Overwrite);
        let groups = recorded.unit_writes_grouped_by_unit();
        let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
        let mut ranges = RangesReadingBackAllZero::default().with_the_writes(&groups[3]);
        let r = recorded.root();
        for pointer in [r.tree_table, r.mapping_root, r.allocation_record_tree_root] {
            if pointer != NodePointer::empty_root() {
                ranges = ranges.with_the_writes(&recorded.unit_writes_at(&pointer));
            }
        }
        if extra == "bz" {
            for ordinal in 1..=recorded.record_count() {
                ranges = ranges.with_the_writes(&recorded.record_writes_of_the_record(ordinal));
            }
        }
        let mut devs = devices_reading_back_all_zero_on(&crash_image, &ranges);
        let mounted = mount_writable_with_space_admission(&recorded.pool.parameters, &mut devs, SpaceAdmission::JudgedByTheFormula).expect("M1");
        let current_root = *mounted.current.root();
        let table = singlefs_core::recovery::instance_table_chain_of_root(&devs, &current_root).expect("表").records;
        println!("RESULT table {extra}: effective={:?} current_root=({:?},{:?}) rows={:?}", (mounted.output.effective_root.instance, mounted.output.effective_root.checkpoint_txg), current_root.instance, current_root.checkpoint_txg, table);
    }
}

#[test]
fn diag_bz_refusal() {
    let recorded = record_one_publish_after_overwrites(5, PublishRecordedAsR::SequentialWriteOfThreeDataUnits);
    let groups = recorded.unit_writes_grouped_by_unit();
    let crash_image = recorded.crash_image_with_the_root_persisted_and_the_rotation_lost();
    let mut ranges = RangesReadingBackAllZero::default().with_the_writes(&groups[3]);
    let r = recorded.root();
    for pointer in [r.tree_table, r.mapping_root, r.allocation_record_tree_root] {
        if pointer != NodePointer::empty_root() {
            ranges = ranges.with_the_writes(&recorded.unit_writes_at(&pointer));
        }
    }
    for ordinal in 1..=recorded.record_count() {
        ranges = ranges.with_the_writes(&recorded.record_writes_of_the_record(ordinal));
    }
    let mut devs = devices_reading_back_all_zero_on(&crash_image, &ranges);
    let mounted = mount_writable_with_space_admission(&recorded.pool.parameters, &mut devs, SpaceAdmission::JudgedByTheFormula).expect("M1");
    let image_after = image_of_devices_reading_back_all_zero(&devs, recorded.pool.device_bytes);
    let fault = ReadFaultOnChosenSlots::on_the_account_of(&r);
    let fdevs = devices_with_the_read_fault_on(&image_after, &fault);
    let mut pool = pool_under_test_after_the_mount(&recorded.pool.parameters, recorded.pool.device_bytes, fdevs, mounted, recorded.pool.write_time_seconds + 10_000);
    fault.is_active.set(true);
    for i in 0..30 {
        let before_root = *pool.session().current.root();
        match pool.overwrite(OVERWRITE_BYTES) {
            Ok(_) => {}
            Err(e) => {
                let image = pool.image();
                let in_ring = root_is_in_the_ring(&image, recorded.key);
                let mut devices = plain_devices_on(&image);
                let table = singlefs_core::recovery::instance_table_chain_of_root(&devices, &before_root).map(|t| t.records);
                println!("RESULT diag step {} refused; R_in_ring={in_ring} current_root=({:?},{:?}) table={:?}", i + 1, before_root.instance, before_root.checkpoint_txg, table);
                let cfg = singlefs_core::recovery::choose_system_configuration(&devices).expect("cfg");
                let roots = singlefs_core::recovery::readable_roots(&devices, &cfg.immutable.region_devices, &cfg.immutable.sizes, &cfg.immutable.filesystem_identifier);
                println!("RESULT diag roots_in_ring={:?}", roots.iter().map(|r| (r.instance.0, r.checkpoint_txg.0)).collect::<Vec<_>>());
                let _ = &mut devices;
                let session = pool.session.as_ref().expect("s");
                let table2 = singlefs_core::recovery::instance_table_chain_of_root(&pool.devices, session.current.root()).map(|t| t.records);
                println!("RESULT diag session.current.root=({:?},{:?}) floor={:?} table_from_session_current={:?}", session.current.root().instance, session.current.root().checkpoint_txg, session.current.root().rollback_floor, table2);
                let t2 = table2.expect("t");
                let cfg2 = singlefs_core::recovery::choose_system_configuration(&pool.devices).expect("cfg");
                let c = singlefs_core::mount::rollback_floor_ceiling(&pool.devices, &cfg2, session.current.root().rollback_floor, &t2, &singlefs_core::mount::ring_slots_known_to_hold_a_root_by(&session.allocator));
                println!("RESULT diag ceiling_with_session_table={}", short(format!("{c:?}")));
                let rr = singlefs_core::recovery::readable_roots(&pool.devices, &cfg2.immutable.region_devices, &cfg2.immutable.sizes, &cfg2.immutable.filesystem_identifier);
                let valid: Vec<(u32, u64)> = rr.iter().filter(|x| x.checkpoint_txg >= session.current.root().rollback_floor).filter(|x| !singlefs_core::recovery::root_is_abandoned_by_the_instance_table(x, &t2)).map(|x| (x.instance.0, x.checkpoint_txg.0)).collect();
                println!("RESULT diag valid_by_session_table={valid:?}");
                break;
            }
        }
    }
}

//! 可写挂载取号之前的预演与取号之后的真做读同一份（岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一「读一次两边共用」，用户定；
//! 代码轮 `research/prompts/m3-b44-code-r2-main-verification.md` 第二节 J2、J3）：这次挂载里每一块第一次读时才穿到底下那块盘，
//! 之后预演与真做都取记下的那一份；预演没走到续做的结局而读盘时有过读错，在取号之前拒（`MountError::Publish`，`cause` 是
//! `PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition`，三份写账全空、两块盘逐字节不变）。
//! 镜像照 `opening_a_container_past_the_largest_number_returns_an_error.rs` 窄盘那一条的造法（攻方原型
//! `research/prompts/m3-b44-code-r2-opus-model/attack_mount_rehearsal_probe.rs` 的 `narrow_image` 原样）：不补环窗口的窄盘 304 槽、
//! 第一个文件 16 个单元、删除队列 {1}、一批一个单元、退役容器一片 1 条；续做中途推抬 F，取号之前那一趟因此在只进内存的盘上走。
//! 改坏的那一份把退役树唯一那片的号改成 u64::MAX（续做最后一批报 C841）。
//! 读错窗口照攻方的「只进内存那一趟里连着两次读错」，落在只进内存那一趟头一次读、之前没记下的一个单元的两份上：续做推的抬 F 算上限要读
//! 每条有效根的树表（判非空，`mount::rollback_floor_ceiling` 一族），读不出就拒这一次抬 F（`RollbackFloorCeilingNeedsUnreadableValidRootTreeTable`），
//! 只进内存那一趟因此走不到续做的结局。不带故障先挂一遍、按读序数记下第一个写之前最后一次只读一份就读成的 16 KiB 单元（盘 0 上那一份），
//! 再挂一遍让这一次与紧接着读的另一份（盘 1 同一个偏移）都报读错。攻方原型落在盘 0 两个系统配置槽上的那一形今天造不出：系统配置槽在预演之前
//! （可写挂载算要取的号）就记进了共用的那一份，只进内存那一趟取号读的是记下的那一份；只进内存那一趟头一次读的另外两类读错不让预演停下——
//! 算生效 F 读实例表读不出照「不按表滤」往下走、释放之前读盘核读不出照隔离往下走。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use common::PoolImageGeometry;
use common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest,
};
use common_deletion_in_progress::{
    batches_of_one_data_unit, first_container_slot_of_the_tree_of_kind_of_the_newest_root,
    root_slot_of_the_tree_of_kind_of_the_newest_root,
};
use common_pointer_head_corruption::{
    change_the_unit_and_propagate_its_checksum_on, data_unit_bytes, node_bytes,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    DeletionQueueContinuationOutcome, MountError, Mounted, RollbackTarget, ShadowLedger,
};
use singlefs_core::mounted_session::UserChange;
use singlefs_core::records::TREE_KIND_RETIREMENT;
use singlefs_core::retirement_tree::{
    RetirementContainerCapacity, RetirementRecordsPerContainerCappedForTests,
};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{
    publish_unlink_intent, PoolWriter, PublishError, FIRST_INODE_NUMBER,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::DeletionSwitches;
use singlefs_harness::fault_injection::{
    injected_block_device_error, FaultInjectingBlockDevice, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};
use singlefs_harness::segments::FixedGeometry;

const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;
const CONTAINER_TREE_ROOT_SMALLEST_KEY_OFFSET: usize = 52;
const CONTAINER_TREE_ROOT_LARGEST_KEY_OFFSET: usize = 60;
const CONTAINER_TREE_ROOT_ENTRIES_START: usize = 131;
const CONTAINER_TREE_ENTRY_SEPARATOR_KEY_OFFSET: usize = 0;
const CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_NUMBER_OFFSET: usize = 18;
const UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES: u64 = 304;
const DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES: usize = 16;
/// 续做把 16 个单元一批一个地删完：16 步。
const CONTINUATION_STEPS_OF_THE_SIXTEEN_UNIT_FILE: usize = 16;
/// 读错窗口里先读的那一份在哪块盘上（按位置条目读一个单元，设备身份升序先读盘 0 那一份）。
const DEVICE_OF_THE_COPY_READ_FIRST: DeviceIdentity = DeviceIdentity(0);
/// 读错窗口里那份树表的长度：一个索引节点 16 KiB。
const BYTES_OF_A_TREE_TABLE: u64 = 16 * 1024;

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn set_u64_at(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn switches() -> DeletionSwitches {
    DeletionSwitches {
        batch_size: batches_of_one_data_unit(),
        tombstone_container_capacity: TombstoneContainerCapacity::FromTheContainerFormat,
        lineage_container_capacity: LineageContainerCapacity::FromTheContainerFormat,
        retirement_container_capacity: RetirementContainerCapacity::CappedForTests(
            RetirementRecordsPerContainerCappedForTests::new(1).expect("1..=959"),
        ),
        container_tree_node_entries: ContainerTreeNodeEntries::FromTheNodeFormat,
    }
}

/// 退役树唯一那片容器的号与根里指它的那几处都改成 u64::MAX：续做最后一批要开新容器时报 C841。
fn move_the_only_retirement_container_to_the_top_of_its_range(
    image: &mut MemoryPool,
    geometry: &PoolImageGeometry,
    number_before: u64,
) {
    let container_slot =
        first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_RETIREMENT);
    change_the_unit_and_propagate_its_checksum_on(
        image,
        geometry,
        container_slot,
        data_unit_bytes(),
        |container| {
            assert_eq!(
                u64_at(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET),
                number_before
            );
            set_u64_at(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET, u64::MAX);
        },
    );
    let root_slot = root_slot_of_the_tree_of_kind_of_the_newest_root(image, TREE_KIND_RETIREMENT);
    change_the_unit_and_propagate_its_checksum_on(
        image,
        geometry,
        root_slot,
        node_bytes(),
        |root| {
            let entry = CONTAINER_TREE_ROOT_ENTRIES_START;
            for offset in [
                CONTAINER_TREE_ROOT_SMALLEST_KEY_OFFSET,
                CONTAINER_TREE_ROOT_LARGEST_KEY_OFFSET,
                entry + CONTAINER_TREE_ENTRY_SEPARATOR_KEY_OFFSET,
                entry + CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_NUMBER_OFFSET,
            ] {
                assert_eq!(u64_at(root, offset), number_before);
                set_u64_at(root, offset, u64::MAX);
            }
        },
    );
}

/// 窄盘镜像：镜像、几何、退役树唯一那片的号。
fn narrow_image() -> (MemoryPool, PoolImageGeometry, u64) {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES,
        );
    pool.sequential_write(
        (DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES - 1) * data_unit_payload_capacity()
            + 100,
    )
    .expect("顺序写长到 16 个单元");
    let version_with_the_grown_file = pool.current_file_version().clone();
    pool.write_time_seconds += 1;
    let write_time_seconds_of_the_unlink = pool.write_time_seconds;
    pool.publish(
        UserChange::Unlink {
            inode: InodeNumber(FIRST_INODE_NUMBER),
            write_time_seconds: write_time_seconds_of_the_unlink,
        },
        None,
    )
    .expect("unlink 到底");
    let geometry = PoolImageGeometry {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
    };
    let PoolUnderTest {
        session,
        devices,
        parameters,
        write_time_seconds,
        ..
    } = &mut pool;
    let session = session.as_mut().expect("可写会话");
    let mut current = session.current.version_of_trees().expect("有树").clone();
    roll_back_by_a_forward_publish(
        parameters,
        devices,
        &mut session.allocator,
        &mut current,
        RollbackTarget {
            instance: version_with_the_grown_file.root.instance,
            checkpoint_txg: version_with_the_grown_file.root.checkpoint_txg,
        },
    )
    .expect("回退到 16 个单元那一版");
    let intent = {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            *write_time_seconds,
            session.instance,
        )
        .expect("意图")
    };
    let number = intent
        .retirement_containers
        .last()
        .expect("退役树一片")
        .contents
        .identity
        .container;
    pool.session = None;
    (pool.image(), geometry, number)
}

/// 改坏的那一份：退役树唯一那片的号在 u64::MAX。
fn narrow_image_with_the_retirement_container_at_the_top_of_its_range(
) -> (MemoryPool, PoolImageGeometry) {
    let (mut image, geometry, number) = narrow_image();
    move_the_only_retirement_container_to_the_top_of_its_range(&mut image, &geometry, number);
    (image, geometry)
}

/// 整池（两块盘一起数）读到盘上的一次读：第几次、哪块盘、从哪个偏移起读多长、这次挂载有没有发过写、在它之前发过几个写。
#[derive(Clone, Copy, Debug)]
struct ReadReachingTheDisk {
    ordinal_across_the_pool: u64,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length_in_bytes: u64,
    is_before_the_first_write_of_the_mount: bool,
    writes_of_the_mount_before_it: u64,
}

/// 读故障怎么摆。
#[derive(Clone, Debug)]
enum ReadFaults {
    None,
    /// 整池第这几次读报块设备错（读序数从 1 数）。
    AtTheOrdinals(BTreeSet<u64>),
    /// 这次挂载发过第一个写之后，每一次读都报块设备错。
    EveryReadAfterTheFirstWrite,
    /// 腿加的：整池第这几次读读成功、第一个字节取反（ReadReturnsCorruptedBytes 那一类）。
    CorruptTheBytesAtTheOrdinals(BTreeSet<u64>),
}

/// 几块盘共用的读账与读故障。
struct PoolReadLedger {
    faults: ReadFaults,
    reads_across_the_pool: u64,
    reads_reaching_the_disk: Vec<ReadReachingTheDisk>,
    writes_of_the_mount: u64,
    failed_reads: u64,
}

/// 内存稀疏盘外面包一层：记下每一次读到它上面的读，照 `ReadFaults` 让某几次读报错。
struct DeviceRecordingReads {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    ledger: Rc<RefCell<PoolReadLedger>>,
}

impl BlockDevice for DeviceRecordingReads {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        let mut ledger = self.ledger.borrow_mut();
        ledger.reads_across_the_pool += 1;
        let ordinal_across_the_pool = ledger.reads_across_the_pool;
        let writes_of_the_mount_before_it = ledger.writes_of_the_mount;
        let is_before_the_first_write_of_the_mount = writes_of_the_mount_before_it == 0;
        ledger.reads_reaching_the_disk.push(ReadReachingTheDisk {
            ordinal_across_the_pool,
            device: self.identity,
            offset,
            length_in_bytes: u64::try_from(buffer.len()).expect("读长装得进 u64"),
            is_before_the_first_write_of_the_mount,
            writes_of_the_mount_before_it,
        });
        let fails = match &ledger.faults {
            ReadFaults::None => false,
            ReadFaults::AtTheOrdinals(ordinals) => ordinals.contains(&ordinal_across_the_pool),
            ReadFaults::EveryReadAfterTheFirstWrite => !is_before_the_first_write_of_the_mount,
            ReadFaults::CorruptTheBytesAtTheOrdinals(_) => false,
        };
        let corrupts = match &ledger.faults {
            ReadFaults::CorruptTheBytesAtTheOrdinals(ordinals) => ordinals.contains(&ordinal_across_the_pool),
            _ => false,
        };
        if corrupts {
            ledger.failed_reads += 1;
        }
        if fails {
            ledger.failed_reads += 1;
            return Err(injected_block_device_error("读"));
        }
        drop(ledger);
        let result = self.inner.read_at(offset, buffer);
        if corrupts && result.is_ok() {
            buffer[0] ^= 0xFF;
        }
        result
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.ledger.borrow_mut().writes_of_the_mount += 1;
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.ledger.borrow_mut().writes_of_the_mount += 1;
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

    fn journal_record_offsets_hint(
        &self,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// 一次可写挂载的结局、挂完之后的两块盘与读账。
struct WritableMountOnRecordingDevices {
    result: Result<Mounted, MountError>,
    images_after: BTreeMap<DeviceIdentity, SparseDevice>,
    reads_reaching_the_disk: Vec<ReadReachingTheDisk>,
    writes_of_the_mount: u64,
    failed_reads: u64,
}

fn mount_writable_on_recording_devices(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    faults: ReadFaults,
) -> WritableMountOnRecordingDevices {
    let ledger = Rc::new(RefCell::new(PoolReadLedger {
        faults,
        reads_across_the_pool: 0,
        reads_reaching_the_disk: Vec::new(),
        writes_of_the_mount: 0,
        failed_reads: 0,
    }));
    let mut devices: Vec<(DeviceIdentity, DeviceRecordingReads)> = plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                DeviceRecordingReads {
                    identity,
                    inner,
                    ledger: Rc::clone(&ledger),
                },
            )
        })
        .collect();
    let result = mount_writable_with_test_only_switches_and_deletion_switches(
        &geometry.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(),
    );
    let images_after = devices
        .iter()
        .map(|(identity, device)| (*identity, device.inner.image.clone()))
        .collect();
    let ledger = ledger.borrow();
    WritableMountOnRecordingDevices {
        result,
        images_after,
        reads_reaching_the_disk: ledger.reads_reaching_the_disk.clone(),
        writes_of_the_mount: ledger.writes_of_the_mount,
        failed_reads: ledger.failed_reads,
    }
}

/// 不带故障挂一遍时，第一个写之前最后一次只读一份就读成的 16 KiB 单元：盘 0 上读它、紧接着的那一次读不是盘 1 同一个偏移（按位置条目读一个单元，
/// 读成一份就停；释放之前读盘核两份紧挨着各读一次），这次挂载第一个写之后两块盘上都不再读它。第一个写之前、只进内存那一趟之后只有逐盘核与
/// 真做取号写之前那几次读（系统配置槽、根环），所以它是只进内存那一趟读的：续做推的抬 F 算上限读一条有效根的树表（之前没记下，之后取记下的那一份）。
/// 交回这一次与紧接着读另一份那一次（再挂一遍时它读错，下一次读的就是盘 1 那一份）的读序数，与那一份的偏移。
fn ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
    reads_reaching_the_disk: &[ReadReachingTheDisk],
) -> (BTreeSet<u64>, DeviceOffsetInBytes) {
    let read_after_the_first_write = |device: DeviceIdentity, offset: DeviceOffsetInBytes| {
        reads_reaching_the_disk.iter().any(|read| {
            !read.is_before_the_first_write_of_the_mount
                && read.device == device
                && read.offset == offset
        })
    };
    let copy_read_first = reads_reaching_the_disk
        .windows(2)
        .filter_map(|consecutive| {
            let (read, read_right_after) = (consecutive[0], consecutive[1]);
            let read_right_after_is_the_other_copy =
                read_right_after.device != read.device && read_right_after.offset == read.offset;
            let read_alone_on_device_zero = read.device == DEVICE_OF_THE_COPY_READ_FIRST
                && read.length_in_bytes == BYTES_OF_A_TREE_TABLE
                && read.is_before_the_first_write_of_the_mount
                && !read_right_after_is_the_other_copy;
            (read_alone_on_device_zero
                && !read_after_the_first_write(DeviceIdentity(0), read.offset)
                && !read_after_the_first_write(DeviceIdentity(1), read.offset))
            .then_some(read)
        })
        .next_back()
        .expect("第一个写之前有一个 16 KiB 单元只读了盘 0 那一份、第一个写之后两块盘上都不再读");
    (
        [
            copy_read_first.ordinal_across_the_pool,
            copy_read_first.ordinal_across_the_pool + 1,
        ]
        .into_iter()
        .collect(),
        copy_read_first.offset,
    )
}

/// 读错窗口真注入上了：窗口里两次读是同一个偏移上先盘 0、后盘 1 那两份。
fn assert_the_window_failed_both_copies_of_one_unit(
    faulted: &WritableMountOnRecordingDevices,
    window: &BTreeSet<u64>,
    offset_of_the_copies: DeviceOffsetInBytes,
) {
    assert_eq!(
        faulted.failed_reads, 2,
        "窗口里两次读都真注入了：{window:?}"
    );
    let reads_in_the_window: Vec<(DeviceIdentity, DeviceOffsetInBytes)> = faulted
        .reads_reaching_the_disk
        .iter()
        .filter(|read| window.contains(&read.ordinal_across_the_pool))
        .map(|read| (read.device, read.offset))
        .collect();
    assert_eq!(
        reads_in_the_window,
        vec![
            (DeviceIdentity(0), offset_of_the_copies),
            (DeviceIdentity(1), offset_of_the_copies),
        ],
        "窗口里报读错的是同一个单元的两份（先盘 0、后盘 1）"
    );
}

/// 取号之前拒（`MountError::Publish`、三份写账全空）的那个错。
fn cause_of_the_refusal_before_acquisition(result: &Result<Mounted, MountError>) -> &PublishError {
    match result {
        Err(MountError::Publish(failed)) => {
            assert!(
                failed.writes_of_persisted_publishes.is_empty()
                    && failed.writes_of_failed_publishes.is_empty(),
                "取号之前拒：一次发布都没落盘（{:?}）",
                failed.cause
            );
            &failed.cause
        }
        Err(other) => {
            panic!("要的是取号之前那一拒（MountError::Publish、写账全空），挂载报的是：{other:?}")
        }
        Ok(mounted) => panic!(
            "要的是取号之前那一拒，挂载做成了：实例 {:?}",
            mounted.output.instance
        ),
    }
}

fn images_of(image: &MemoryPool) -> BTreeMap<DeviceIdentity, SparseDevice> {
    image.devices.clone()
}

/// 攻方 J2 那一形：改坏的镜像上，只进内存那一趟里连着两次读错——它头一次读的一份树表两份都读不出（窗口取法见文件头），续做推的抬 F
/// 算上限读不出它、拒这一次抬，那一趟走不到续做的结局。读错照旧吞成「放行」的话，真做取号、写行、推抬 F、续做落盘之后才交 C841
/// （真做再读那份树表读得出）；预演没走到结局而读过错就在取号之前拒、交最先那次读错（盘 0 那一份），盘不变。
/// 再挂一次不带故障：照常在取号之前交 C841，盘仍不变。
#[test]
fn both_copies_of_a_tree_table_failing_on_the_first_read_by_the_in_memory_rehearsal_refuse_the_mount_before_acquisition_then_a_plain_remount_refuses_on_the_container_number(
) {
    let (image, geometry) = narrow_image_with_the_retirement_container_at_the_top_of_its_range();
    let fault_free = mount_writable_on_recording_devices(&image, &geometry, ReadFaults::None);
    assert!(
        matches!(
            cause_of_the_refusal_before_acquisition(&fault_free.result),
            PublishError::NextContainerNumberPastTheTopOfItsRange(_)
        ),
        "不带故障时改坏的那一份在取号之前交 C841"
    );
    let (window, offset_of_the_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &fault_free.reads_reaching_the_disk,
        );

    let faulted = mount_writable_on_recording_devices(
        &image,
        &geometry,
        ReadFaults::AtTheOrdinals(window.clone()),
    );
    assert_the_window_failed_both_copies_of_one_unit(&faulted, &window, offset_of_the_copies);
    let cause = cause_of_the_refusal_before_acquisition(&faulted.result);
    let PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(read) = cause
    else {
        panic!("预演里的读错要在取号之前交出来，交的是：{cause:?}");
    };
    assert_eq!(
        (read.device, read.offset),
        (DEVICE_OF_THE_COPY_READ_FIRST, offset_of_the_copies),
        "交的是最先那次读错：盘 0 那一份树表"
    );
    assert!(
        faulted.images_after == images_of(&image),
        "取号之前拒：两块盘逐字节不变"
    );

    let remount = mount_writable_on_recording_devices(&image, &geometry, ReadFaults::None);
    assert!(
        matches!(
            cause_of_the_refusal_before_acquisition(&remount.result),
            PublishError::NextContainerNumberPastTheTopOfItsRange(_)
        ),
        "不带故障再挂一次：照常在取号之前交 C841"
    );
    assert!(
        remount.images_after == images_of(&image),
        "再挂一次也在取号之前拒：两块盘逐字节不变"
    );
}

/// 攻方 J3 那一形：没改坏的镜像上同一种读错窗口（只进内存那一趟头一次读的一份树表两份都读不出）。读错被只进内存那一趟吃掉的话，
/// 真做再读那份树表读得出、一次读错都没见，挂载照常做成；预演没走完而读过错，在取号之前拒、交那次读错，盘不变；
/// 再挂一次不带故障照常做成、续做走到空。
#[test]
fn the_same_read_fault_window_on_the_unchanged_image_is_refused_before_acquisition_instead_of_being_eaten_by_the_rehearsal(
) {
    let (image, geometry, _) = narrow_image();
    let fault_free = mount_writable_on_recording_devices(&image, &geometry, ReadFaults::None);
    assert!(
        fault_free.result.is_ok(),
        "不带故障时没改坏的那一份挂载做成"
    );
    let (window, offset_of_the_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &fault_free.reads_reaching_the_disk,
        );

    let faulted = mount_writable_on_recording_devices(
        &image,
        &geometry,
        ReadFaults::AtTheOrdinals(window.clone()),
    );
    assert_the_window_failed_both_copies_of_one_unit(&faulted, &window, offset_of_the_copies);
    assert!(
        matches!(
            cause_of_the_refusal_before_acquisition(&faulted.result),
            PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(_)
        ),
        "预演里的读错在取号之前交出来，不被预演吃掉"
    );
    assert!(
        faulted.images_after == images_of(&image),
        "取号之前拒：两块盘逐字节不变"
    );

    let remount = mount_writable_on_recording_devices(&image, &geometry, ReadFaults::None);
    let mounted = remount
        .result
        .expect("不带故障再挂一次：没改坏的那一份挂载做成");
    assert_eq!(
        mounted.output.deletion_queue_continuation.len(),
        CONTINUATION_STEPS_OF_THE_SIXTEEN_UNIT_FILE,
        "续做 16 步"
    );
    assert!(
        matches!(
            mounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "续做走到空"
    );
}

/// 真做读的就是预演读过、记下的那一份：没改坏的镜像上，这次挂载发过第一个写之后的每一次读都报错。只进内存那一趟把取号之后那一串
/// 连同续做照真发走过一遍，它读成的块都记下了、真做写成的块也记下了，取号写起到这次挂载最后一个写（续做最后一步）为止，真做一次都不再读到盘上，
/// 挂载照常做成、续做走到空。真做照旧穿到底下那块盘读的话，取号之后头一次读就报错。
/// 最后一个写之后的实例表清扫照旧读盘（条款要它抹之前再读一次盘，`instance_table_sweep_at_the_end_of_a_writable_mount.rs`）：
/// 它读实例表读错，清扫这一轮不判，挂载照样做成，不在这一判里。
#[test]
fn after_the_first_write_the_real_run_reads_only_what_the_rehearsal_read_or_the_mount_wrote() {
    let (image, geometry, _) = narrow_image();
    let mount = mount_writable_on_recording_devices(
        &image,
        &geometry,
        ReadFaults::EveryReadAfterTheFirstWrite,
    );
    let reads_between_the_first_and_the_last_write: Vec<&ReadReachingTheDisk> = mount
        .reads_reaching_the_disk
        .iter()
        .filter(|read| {
            !read.is_before_the_first_write_of_the_mount
                && read.writes_of_the_mount_before_it < mount.writes_of_the_mount
        })
        .collect();
    assert!(
        reads_between_the_first_and_the_last_write.is_empty(),
        "第一个写之后、这次挂载最后一个写之前没有一次读到盘上（读到盘上的那几次：{reads_between_the_first_and_the_last_write:?}）"
    );
    let mounted = mount
        .result
        .expect("取号写起到最后一个写一次都不读盘：挂载做成");
    assert_eq!(
        mounted.output.deletion_queue_continuation.len(),
        CONTINUATION_STEPS_OF_THE_SIXTEEN_UNIT_FILE,
        "续做 16 步"
    );
    assert!(
        matches!(
            mounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "续做走到空"
    );
}

fn fixed_geometry_of(geometry: &PoolImageGeometry) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: geometry.parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: geometry.parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: geometry.parameters.geometry.root_ring_slots_per_region,
    }
}

/// 无故障时真做看到的字节与改之前相同：没改坏的镜像上挂一次，续做每一步、续做推的抬 F 每一次空发布写的字节数与两块盘各写的字节数，
/// 逐项等于改之前在冻结副本上量的那一组（攻方 `research/prompts/m3-b44-code-r2-opus-model/out/run-bytes.log`：
/// 16 步合计 6594560 字节，主 agent 复跑逐字相同）。真做读到的哪一块与改之前不同，发布路径写出的单元就跟着变。
#[test]
fn without_faults_the_mount_writes_the_same_bytes_per_publish_as_before_the_reads_were_shared() {
    let (image, geometry, _) = narrow_image();
    let unarmed = SharedFaultPlan::unarmed(fixed_geometry_of(&geometry));
    let mut devices: Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&image)
            .into_iter()
            .map(|(identity, device)| {
                (
                    identity,
                    FaultInjectingBlockDevice::new(identity, device, unarmed.clone()),
                )
            })
            .collect();
    let mounted = mount_writable_with_test_only_switches_and_deletion_switches(
        &geometry.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(),
    )
    .expect("没改坏的那一份挂载做成");
    let bytes_per_continuation_step: Vec<u64> = mounted
        .output
        .deletion_queue_continuation
        .iter()
        .map(|step| step.writes.total().written_bytes)
        .collect();
    assert_eq!(
        bytes_per_continuation_step,
        vec![
            442_880, 442_880, 442_880, 442_880, 442_880, 377_344, 377_344, 377_344, 377_344,
            377_344, 377_344, 442_880, 442_880, 442_880, 410_112, 377_344,
        ],
        "续做每一步写的字节数与改之前相同"
    );
    let bytes_per_floor_raise_publish: Vec<u64> = mounted
        .output
        .floor_raises_pushed_by_the_continuation
        .iter()
        .flat_map(|raised| {
            raised
                .publishes
                .iter()
                .map(|publish| publish.writes.total().written_bytes)
        })
        .collect();
    assert_eq!(
        bytes_per_floor_raise_publish,
        vec![213_504, 213_504],
        "续做推的抬 F 每一次空发布写的字节数与改之前相同"
    );
    for (device, written_bytes_before_the_reads_were_shared) in [
        (DeviceIdentity(0), 3_766_784_u64),
        (DeviceIdentity(1), 3_763_712_u64),
    ] {
        assert_eq!(
            unarmed.counts_of_device(device).written_bytes,
            written_bytes_before_the_reads_were_shared,
            "盘 {device:?} 这次挂载写的字节数与改之前相同"
        );
    }
}

/// 腿的探针：J3 那一形，但读错换成「读成功、字节坏」（窗口里那两次读：盘 0 与盘 1 的同一份树表）。
#[test]
fn probe_corrupt_bytes_instead_of_io_errors_in_the_same_window() {
    let (image, geometry, _) = narrow_image();
    let fault_free = mount_writable_on_recording_devices(&image, &geometry, ReadFaults::None);
    assert!(fault_free.result.is_ok());
    let (window, offset_of_the_copies) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
            &fault_free.reads_reaching_the_disk,
        );
    let faulted = mount_writable_on_recording_devices(
        &image,
        &geometry,
        ReadFaults::CorruptTheBytesAtTheOrdinals(window.clone()),
    );
    let outcome = match &faulted.result {
        Ok(m) => format!("Ok instance={:?} steps={}", m.output.instance, m.output.deletion_queue_continuation.len()),
        Err(e) => format!("Err {e:?}").chars().take(400).collect::<String>(),
    };
    let disk_changed = faulted.images_after != images_of(&image);
    println!("PROBE2 window={window:?} offset_of_the_copies={offset_of_the_copies:?} corrupted_reads={}", faulted.failed_reads);
    println!("PROBE2 outcome={outcome}");
    println!("PROBE2 disk_changed={disk_changed} writes_of_the_mount={}", faulted.writes_of_the_mount);
    let remount = mount_writable_on_recording_devices(&image, &geometry, ReadFaults::None);
    println!("PROBE2 remount_without_fault_ok={}", remount.result.is_ok());
    // 对照：同一个窗口读错换成块设备错（W16 重造的那条断言的形）
    let io = mount_writable_on_recording_devices(&image, &geometry, ReadFaults::AtTheOrdinals(window.clone()));
    let io_outcome = match &io.result {
        Ok(_) => "Ok".to_string(),
        Err(e) => format!("Err {e:?}").chars().take(200).collect::<String>(),
    };
    println!("PROBE2 io_error_window_outcome={io_outcome} disk_changed={}", io.images_after != images_of(&image));
}

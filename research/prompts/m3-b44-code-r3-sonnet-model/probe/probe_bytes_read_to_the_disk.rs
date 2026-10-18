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
fn units_of_the_probe() -> usize {
    std::env::var("PROBE_UNITS").ok().and_then(|v| v.parse().ok()).unwrap_or(16)
}

fn narrow_image() -> (MemoryPool, PoolImageGeometry, u64) {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES,
        );
    pool.sequential_write(
        (units_of_the_probe() - 1) * data_unit_payload_capacity()
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
        };
        if fails {
            ledger.failed_reads += 1;
            return Err(injected_block_device_error("读"));
        }
        drop(ledger);
        self.inner.read_at(offset, buffer)
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


/// 探针（腿自己的量法，不进冻结副本）：不带故障挂一遍窄盘，数读到盘上的字节与 32 KiB 读。
#[test]
fn probe_bytes_read_to_the_disk_by_one_fault_free_writable_mount_with_a_sixteen_unit_deletion_queue() {
    let (image, geometry, _number) = narrow_image();
    let mounted = mount_writable_on_recording_devices(&image, &geometry, ReadFaults::None);
    let outcome = match &mounted.result {
        Ok(m) => format!("Ok instance={:?}", m.output.instance),
        Err(e) => format!("Err {e:?}"),
    };
    let reads = &mounted.reads_reaching_the_disk;
    let total_bytes: u64 = reads.iter().map(|r| r.length_in_bytes).sum();
    let bytes_32k: u64 = reads.iter().filter(|r| r.length_in_bytes == 32768).map(|r| r.length_in_bytes).sum();
    let n_32k = reads.iter().filter(|r| r.length_in_bytes == 32768).count();
    let n_16k = reads.iter().filter(|r| r.length_in_bytes == 16384).count();
    let before_first_write = reads.iter().filter(|r| r.is_before_the_first_write_of_the_mount).count();
    let after_first_write = reads.len() - before_first_write;
    let bytes_after_first_write: u64 = reads.iter().filter(|r| !r.is_before_the_first_write_of_the_mount).map(|r| r.length_in_bytes).sum();
    let mut distinct: BTreeSet<(u64, u64)> = BTreeSet::new();
    for r in reads { distinct.insert((r.device.0.into(), r.offset.0)); }
    println!("PROBE outcome={outcome}");
    println!("PROBE reads_reaching_the_disk={} total_bytes={} reads_32KiB={} bytes_32KiB={} reads_16KiB={}", reads.len(), total_bytes, n_32k, bytes_32k, n_16k);
    println!("PROBE reads_before_first_write={} reads_after_first_write={} bytes_after_first_write={} distinct_device_offset_starts={}", before_first_write, after_first_write, bytes_after_first_write, distinct.len());
    let distinct_32k: BTreeSet<(u64,u64)> = reads.iter().filter(|r| r.length_in_bytes==32768).map(|r| (u64::from(r.device.0), r.offset.0)).collect();
    println!("PROBE distinct_32KiB_device_offsets={}", distinct_32k.len());
    println!("PROBE writes_of_the_mount={}", mounted.writes_of_the_mount);
    println!("PROBE data_bytes_of_the_deleted_file={}", units_of_the_probe() as u64 * 32768);
}

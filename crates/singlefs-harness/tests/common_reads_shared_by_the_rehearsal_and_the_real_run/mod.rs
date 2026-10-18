//! 可写挂载取号之前的预演与取号之后的真做读同一份（`mounted_session::DeviceReadOnceInThisMount`）那几份用例共用的搭建：
//! 窄盘镜像（`writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs` 那一份的造法，攻方原型
//! `research/prompts/m3-b44-code-r2-opus-model/attack_mount_rehearsal_probe.rs` 的 `narrow_image` 原样：不补环窗口的窄盘 304 槽、
//! 第一个文件 16 个单元、删除队列 {1}、一批一个单元、退役容器一片 1 条；续做中途推抬 F，取号之前那一趟因此在只进内存的盘上走）与它改坏的那一份
//! （退役树唯一那片的号改成 u64::MAX，续做最后一批报 C841）、删除开关、记读写账并照计划让某几次读报块设备错或读回坏字节的盘、一次可写挂载。

#![allow(dead_code, reason = "几份用例各只用到一部分")]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::common::PoolImageGeometry;
use crate::common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest,
};
use crate::common_deletion_in_progress::{
    batches_of_one_data_unit, first_container_slot_of_the_tree_of_kind_of_the_newest_root,
    root_slot_of_the_tree_of_kind_of_the_newest_root,
};
use crate::common_pointer_head_corruption::{
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
use singlefs_format::SYSTEM_CONFIGURATION_SLOT_BYTES;
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

const PACKED_UNIT_CONTAINER_NUMBER_OFFSET: usize = 53;
const CONTAINER_TREE_ROOT_SMALLEST_KEY_OFFSET: usize = 52;
const CONTAINER_TREE_ROOT_LARGEST_KEY_OFFSET: usize = 60;
const CONTAINER_TREE_ROOT_ENTRIES_START: usize = 131;
const CONTAINER_TREE_ENTRY_SEPARATOR_KEY_OFFSET: usize = 0;
const CONTAINER_TREE_ENTRY_IDENTITY_CONTAINER_NUMBER_OFFSET: usize = 18;
const UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES: u64 = 304;
const DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES: usize = 16;
/// 续做把 16 个单元一批一个地删完：16 步。
pub const CONTINUATION_STEPS_OF_THE_SIXTEEN_UNIT_FILE: usize = 16;
/// 一份单元两块盘上的两份里先读的那一份在哪块盘上（按位置条目读一个单元，设备身份升序先读盘 0 那一份）。
pub const DEVICE_OF_THE_COPY_READ_FIRST: DeviceIdentity = DeviceIdentity(0);
/// 一份树表的长度：一个索引节点 16 KiB。
pub const BYTES_OF_A_TREE_TABLE: u64 = 16 * 1024;
/// 攻方量的（`research/prompts/m3-b44-code-r3-opus-model/out/trace-after-unchanged.txt` 与 `trace-after-corrupted.txt` 第 303、304 行）：
/// 两份窄盘可写挂载整池第 303 次读是盘 0 槽 0、第 304 次是盘 0 槽 1，是这次挂载头一次把它们读进预演与真做共用的那一份
/// （判号那一读，`transaction::next_instance_generation_to_acquire`）；之前那 302 次是读阶段、恢复与重建分配器的读与判号读根环。
/// 用例开头核这两次读的是哪一块。
pub const READ_OF_SLOT_ZERO_OF_DEVICE_ZERO_INTO_THE_SHARED_READS: u64 = 303;
/// 紧接着的那一次：盘 0 槽 1。
pub const READ_OF_SLOT_ONE_OF_DEVICE_ZERO_INTO_THE_SHARED_READS: u64 = 304;
/// 读回坏字节那一次把第一个字节按位取反：系统配置槽的整槽校验和、单元的 CRC-32C 都对不上。
const BITS_FLIPPED_IN_THE_FIRST_BYTE: u8 = 0xFF;

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 字节"))
}

fn set_u64_at(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

#[must_use]
pub fn switches() -> DeletionSwitches {
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
#[must_use]
pub fn narrow_image() -> (MemoryPool, PoolImageGeometry, u64) {
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
#[must_use]
pub fn narrow_image_with_the_retirement_container_at_the_top_of_its_range(
) -> (MemoryPool, PoolImageGeometry) {
    let (mut image, geometry, number) = narrow_image();
    move_the_only_retirement_container_to_the_top_of_its_range(&mut image, &geometry, number);
    (image, geometry)
}

/// 整池（两块盘一起数）读到盘上的一次读：第几次、哪块盘、从哪个偏移起读多长、这次挂载在它之前发过几个写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadReachingTheDisk {
    pub ordinal_across_the_pool: u64,
    pub device: DeviceIdentity,
    pub offset: DeviceOffsetInBytes,
    pub length_in_bytes: u64,
    pub writes_of_the_mount_before_it: u64,
}

impl ReadReachingTheDisk {
    #[must_use]
    pub fn is_before_the_first_write_of_the_mount(&self) -> bool {
        self.writes_of_the_mount_before_it == 0
    }

    #[must_use]
    pub fn reads_the_same_bytes_as(&self, other: &ReadReachingTheDisk) -> bool {
        self.device == other.device
            && self.offset == other.offset
            && self.length_in_bytes == other.length_in_bytes
    }

    /// 读的是这块盘两个系统配置槽之一（槽 0 在偏移 0、槽 1 在固定结构槽距处，整槽 4096 字节）。
    #[must_use]
    pub fn reads_a_system_configuration_slot(&self, geometry: &PoolImageGeometry) -> bool {
        let spacing = u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing);
        self.length_in_bytes == SYSTEM_CONFIGURATION_SLOT_BYTES
            && (self.offset == DeviceOffsetInBytes(0)
                || self.offset == DeviceOffsetInBytes(spacing))
    }
}

/// 这次挂载落到盘上的一个写（写零也算）：哪块盘、从哪个偏移起写多长。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteReachingTheDisk {
    pub device: DeviceIdentity,
    pub offset: DeviceOffsetInBytes,
    pub length_in_bytes: u64,
}

/// 带故障的那几次读怎么坏。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadFault {
    /// 读报块设备错，缓冲区不动（读错）。
    ReportsADeviceError,
    /// 读报成功、读回的第一个字节按位取反（读坏：读得出、自证不过）。
    ReadsBackCorruptedBytes,
}

/// 哪几次读带故障。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FaultedReads {
    Nothing,
    /// 整池第这几次读（读序数从 1 数）。
    AtTheOrdinals(BTreeSet<u64>),
    /// 整池第 `from_ordinal` 次读起，哪块盘上读 `offset` 起 `length_in_bytes` 字节都带故障（一份单元两块盘上的两份同偏移、同长度）。
    EveryReadOfTheRangeFromTheOrdinal {
        from_ordinal: u64,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    },
}

impl FaultedReads {
    fn covers(&self, read: &ReadReachingTheDisk) -> bool {
        match self {
            FaultedReads::Nothing => false,
            FaultedReads::AtTheOrdinals(ordinals) => {
                ordinals.contains(&read.ordinal_across_the_pool)
            }
            FaultedReads::EveryReadOfTheRangeFromTheOrdinal {
                from_ordinal,
                offset,
                length_in_bytes,
            } => {
                read.ordinal_across_the_pool >= *from_ordinal
                    && read.offset == *offset
                    && read.length_in_bytes == *length_in_bytes
            }
        }
    }
}

/// 几块盘共用的读写账与读故障的计划。
struct PoolReadLedger {
    fault: ReadFault,
    faulted_reads: FaultedReads,
    reads_across_the_pool: u64,
    reads_reaching_the_disk: Vec<ReadReachingTheDisk>,
    writes_reaching_the_disk: Vec<WriteReachingTheDisk>,
    reads_that_carried_the_fault: Vec<ReadReachingTheDisk>,
}

impl PoolReadLedger {
    fn count_a_write(&mut self, device: DeviceIdentity, offset: DeviceOffsetInBytes, length: u64) {
        self.writes_reaching_the_disk.push(WriteReachingTheDisk {
            device,
            offset,
            length_in_bytes: length,
        });
    }
}

/// 内存稀疏盘外面包一层：记下每一次读到它上面的读与每一个写，照计划让某几次读报块设备错或读回坏字节。
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
        let this_read = ReadReachingTheDisk {
            ordinal_across_the_pool: ledger.reads_across_the_pool,
            device: self.identity,
            offset,
            length_in_bytes: u64::try_from(buffer.len()).expect("读长装得进 u64"),
            writes_of_the_mount_before_it: u64::try_from(ledger.writes_reaching_the_disk.len())
                .expect("写的个数装得进 u64"),
        };
        ledger.reads_reaching_the_disk.push(this_read);
        let carries_the_fault = ledger.faulted_reads.covers(&this_read);
        if carries_the_fault {
            ledger.reads_that_carried_the_fault.push(this_read);
        }
        let fault = ledger.fault;
        drop(ledger);
        match (carries_the_fault, fault) {
            (false, ReadFault::ReportsADeviceError | ReadFault::ReadsBackCorruptedBytes) => {
                self.inner.read_at(offset, buffer)
            }
            (true, ReadFault::ReportsADeviceError) => Err(injected_block_device_error("读")),
            (true, ReadFault::ReadsBackCorruptedBytes) => {
                self.inner.read_at(offset, buffer)?;
                buffer[0] ^= BITS_FLIPPED_IN_THE_FIRST_BYTE;
                Ok(())
            }
        }
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.ledger.borrow_mut().count_a_write(
            self.identity,
            offset,
            u64::try_from(bytes.len()).expect("写长装得进 u64"),
        );
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.ledger
            .borrow_mut()
            .count_a_write(self.identity, offset, length);
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

/// 一次可写挂载的结局、挂完之后的两块盘、读写账与真带了故障的那几次读。
pub struct WritableMountOnRecordingDevices {
    pub result: Result<Mounted, MountError>,
    pub images_after: BTreeMap<DeviceIdentity, SparseDevice>,
    pub reads_reaching_the_disk: Vec<ReadReachingTheDisk>,
    pub writes_reaching_the_disk: Vec<WriteReachingTheDisk>,
    pub reads_that_carried_the_fault: Vec<ReadReachingTheDisk>,
}

/// 在 `image` 的一份拷贝上可写挂载一次（准入照式子判、影子账开、删除开关 [`switches`]），照 `fault` 与 `faulted_reads` 让某几次读带故障。
#[must_use]
pub fn mount_writable_on_recording_devices(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    fault: ReadFault,
    faulted_reads: FaultedReads,
) -> WritableMountOnRecordingDevices {
    let ledger = Rc::new(RefCell::new(PoolReadLedger {
        fault,
        faulted_reads,
        reads_across_the_pool: 0,
        reads_reaching_the_disk: Vec::new(),
        writes_reaching_the_disk: Vec::new(),
        reads_that_carried_the_fault: Vec::new(),
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
        writes_reaching_the_disk: ledger.writes_reaching_the_disk.clone(),
        reads_that_carried_the_fault: ledger.reads_that_carried_the_fault.clone(),
    }
}

/// 不带故障挂一次。
#[must_use]
pub fn mount_writable_without_faults(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
) -> WritableMountOnRecordingDevices {
    mount_writable_on_recording_devices(
        image,
        geometry,
        ReadFault::ReportsADeviceError,
        FaultedReads::Nothing,
    )
}

#[must_use]
pub fn images_of(image: &MemoryPool) -> BTreeMap<DeviceIdentity, SparseDevice> {
    image.devices.clone()
}

/// 取号之前拒（`MountError::Publish`、三份写账全空）的那个错。
pub fn cause_of_the_refusal_before_acquisition(
    result: &Result<Mounted, MountError>,
) -> &PublishError {
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

/// 挂载做成、续做 16 步走到空。
pub fn assert_the_mount_is_made_and_the_continuation_walks_to_empty(
    result: &Result<Mounted, MountError>,
    situation: &str,
) {
    let mounted = result
        .as_ref()
        .unwrap_or_else(|error| panic!("{situation}：挂载做成；挂载报的是 {error:?}"));
    assert_eq!(
        mounted.output.deletion_queue_continuation.len(),
        CONTINUATION_STEPS_OF_THE_SIXTEEN_UNIT_FILE,
        "{situation}：续做 16 步"
    );
    assert!(
        matches!(
            mounted.output.deletion_queue_continuation_outcome,
            DeletionQueueContinuationOutcome::WalkedToEmpty
        ),
        "{situation}：续做走到空"
    );
}

/// 不带故障挂一遍时，第一个写之前最后一次只读一份就读成的 16 KiB 单元：盘 0 上读它、紧接着的那一次读不是盘 1 同一个偏移（按位置条目读一个单元，
/// 读成一份就停；释放之前读盘核两份紧挨着各读一次），这次挂载第一个写之后两块盘上都不再读它。第一个写之前、只进内存那一趟之后只有逐盘核与
/// 真做取号写之前那几次读（系统配置槽、根环）与取号写算世代号那一读（系统配置槽），所以它是只进内存那一趟读的：续做推的抬 F 算上限读一条有效根的
/// 树表（之前没记下，之后取记下的那一份）。交回这一次与紧接着读另一份那一次（再挂一遍时它带了故障，下一次读的就是盘 1 那一份）的读序数，
/// 与那一份的偏移（`writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs` 同名那一个的取法）。
#[must_use]
pub fn ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(
    reads_reaching_the_disk: &[ReadReachingTheDisk],
) -> (BTreeSet<u64>, DeviceOffsetInBytes) {
    let read_after_the_first_write = |device: DeviceIdentity, offset: DeviceOffsetInBytes| {
        reads_reaching_the_disk.iter().any(|read| {
            !read.is_before_the_first_write_of_the_mount()
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
                && read.is_before_the_first_write_of_the_mount()
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

/// 带故障的那几次读真落在一份单元的两份上：同一个偏移、同一个长度，先盘 0、后盘 1，都在这次挂载第一个写之前。
pub fn assert_the_faulted_reads_are_both_copies_of_one_unit(
    faulted: &WritableMountOnRecordingDevices,
    offset_of_the_copies: DeviceOffsetInBytes,
) {
    let devices_and_offsets: Vec<(DeviceIdentity, DeviceOffsetInBytes, bool)> = faulted
        .reads_that_carried_the_fault
        .iter()
        .map(|read| {
            (
                read.device,
                read.offset,
                read.is_before_the_first_write_of_the_mount(),
            )
        })
        .collect();
    assert_eq!(
        devices_and_offsets,
        vec![
            (DeviceIdentity(0), offset_of_the_copies, true),
            (DeviceIdentity(1), offset_of_the_copies, true),
        ],
        "带故障的是同一个单元的两份（先盘 0、后盘 1），都在第一个写之前"
    );
}

//! 可写挂载里读坏之后当场重读的那一读绕过预演与真做共用的那一份、直接读底下那块盘，读到自证过的换掉记下的那一份
//! （主 agent 判定：岔路单 `research/prompts/m3-b44-code-r2-forks.md` 岔路一「读一次两边共用」那一行只管「抬 F 与续做要读的东西」，
//! 抬 F 与续做里「读坏当场重读」那一读不许被共用的那一份吃掉；预演里换了，真做读到的就是好的那一份）。
//! 释放之前读盘核那一处（D19（块指针的结构与宽度预算） 已定项 5：读不出、核出对不上都先重读一次，一次瞬时坏读不许让单元永久隔离）：
//! 续做里删一个单元要释放它两块盘上的两份，只进内存那一趟预演把续做照真发走一遍，头一次读到它们的就是那一趟。
//! 那一次读回坏字节（读成功、整单元 CRC-32C 与位置项对不上）记进共用的那一份；当场重读要再读一次盘，不取记下的坏字节。
//! 抬 F 算上限读根环那一处（D16（发布语义） 已定项 1「根槽这一次读坏」那一行：这个进程知道住着根的槽读坏就重读一次，仍坏就拒）：
//! 写行与暖机之后推的抬 F 读根环取的是共用的那一份，那一槽头一次读进共用的那一份时读回的坏字节记在那里；当场重读要再读一次盘。
//! 续做那一形的镜像照 `writable_mount_rehearsal_and_the_real_run_read_each_block_once.rs` 没改坏的那一份的造法（攻方原型
//! `research/prompts/m3-b44-code-r2-opus-model/attack_mount_rehearsal_probe.rs` 的 `narrow_image`）：不补环窗口的窄盘 304 槽、
//! 第一个文件 16 个单元、删除队列 {1}、一批一个单元；续做中途推抬 F，取号之前那一趟因此在只进内存的盘上走。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_images_for_the_checker_tier;
mod common_pointer_head_corruption;
#[path = "common_images_for_the_checker_tier/rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root.rs"]
mod scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use common::PoolImageGeometry;
use common_admission::{
    plain_devices_on,
    start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off,
    PoolUnderTest,
};
use common_deletion_in_progress::batches_of_one_data_unit;
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
use singlefs_core::recovery::choose_system_configuration;
use singlefs_core::retirement_tree::{
    RetirementContainerCapacity, RetirementRecordsPerContainerCappedForTests,
};
use singlefs_core::tombstone_tree::TombstoneContainerCapacity;
use singlefs_core::transaction::{publish_unlink_intent, PoolWriter, FIRST_INODE_NUMBER};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_core::unlink::DeletionSwitches;
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice, SparseDevice};

const UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES: u64 = 304;
const DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES: usize = 16;
/// 续做把 16 个单元一批一个地删完：16 步。
const CONTINUATION_STEPS_OF_THE_SIXTEEN_UNIT_FILE: usize = 16;
/// 读回坏字节那一次把第一个字节按位取反：整单元 CRC-32C 与位置项对不上。
const BITS_FLIPPED_IN_THE_FIRST_BYTE: u8 = 0xFF;

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

/// 窄盘镜像与它的几何：第一个文件长到 16 个单元那一版上发了 unlink 的意图，删除队列 {1}，一个单元都没删。
fn narrow_image_with_the_unlink_intent_of_a_sixteen_unit_file() -> (MemoryPool, PoolImageGeometry) {
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
    {
        let mut writer = PoolWriter::new(parameters, devices.as_mut_slice());
        publish_unlink_intent(
            &mut writer,
            &mut session.allocator,
            &current,
            InodeNumber(FIRST_INODE_NUMBER),
            *write_time_seconds,
            session.instance,
        )
        .expect("意图");
    }
    pool.session = None;
    (pool.image(), geometry)
}

/// 整池（两块盘一起数）读到盘上的一次读：第几次、哪块盘、从哪个偏移起读多长、这次挂载在它之前发过几个写。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReadReachingTheDisk {
    ordinal_across_the_pool: u64,
    device: DeviceIdentity,
    offset: DeviceOffsetInBytes,
    length_in_bytes: u64,
    writes_of_the_mount_before_it: u64,
}

impl ReadReachingTheDisk {
    fn is_before_the_first_write_of_the_mount(&self) -> bool {
        self.writes_of_the_mount_before_it == 0
    }

    fn reads_the_same_bytes_as(&self, other: &ReadReachingTheDisk) -> bool {
        self.device == other.device
            && self.offset == other.offset
            && self.length_in_bytes == other.length_in_bytes
    }
}

/// 几块盘共用的读账与读回坏字节的那一次（整池第几次读，从 1 数：读成功、第一个字节按位取反）。
struct PoolReadLedger {
    corrupted_read_ordinal: Option<u64>,
    reads_across_the_pool: u64,
    reads_reaching_the_disk: Vec<ReadReachingTheDisk>,
    writes_of_the_mount: u64,
    corrupted_reads: Vec<ReadReachingTheDisk>,
}

/// 内存稀疏盘外面包一层：记下每一次读到它上面的读，照 `corrupted_read_ordinal` 让那一次读读回坏字节。
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
        self.inner.read_at(offset, buffer)?;
        let mut ledger = self.ledger.borrow_mut();
        ledger.reads_across_the_pool += 1;
        let this_read = ReadReachingTheDisk {
            ordinal_across_the_pool: ledger.reads_across_the_pool,
            device: self.identity,
            offset,
            length_in_bytes: u64::try_from(buffer.len()).expect("读长装得进 u64"),
            writes_of_the_mount_before_it: ledger.writes_of_the_mount,
        };
        ledger.reads_reaching_the_disk.push(this_read);
        if ledger.corrupted_read_ordinal == Some(this_read.ordinal_across_the_pool) {
            ledger.corrupted_reads.push(this_read);
            buffer[0] ^= BITS_FLIPPED_IN_THE_FIRST_BYTE;
        }
        Ok(())
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

/// 一次可写挂载的结局、挂完之后的两块盘、读账与读回坏字节的那几次。
struct WritableMountOnRecordingDevices {
    result: Result<Mounted, MountError>,
    images_after: BTreeMap<DeviceIdentity, SparseDevice>,
    reads_reaching_the_disk: Vec<ReadReachingTheDisk>,
    corrupted_reads: Vec<ReadReachingTheDisk>,
}

fn mount_writable_on_recording_devices(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    corrupted_read_ordinal: Option<u64>,
) -> WritableMountOnRecordingDevices {
    let ledger = Rc::new(RefCell::new(PoolReadLedger {
        corrupted_read_ordinal,
        reads_across_the_pool: 0,
        reads_reaching_the_disk: Vec::new(),
        writes_of_the_mount: 0,
        corrupted_reads: Vec::new(),
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
        corrupted_reads: ledger.corrupted_reads.clone(),
    }
}

/// 不带故障挂一遍时，第一个写之前最后一对这样的读：盘 0 上读一个单元、紧接着读盘 1 同一个偏移同样长的那一份（释放之前读盘核按位置项逐份读，
/// 设备身份升序），这次挂载第一个写之后两块盘上都不再读它。第一个写之前、只进内存那一趟之后只有逐盘核与真做取号写之前那几次读（系统配置槽、
/// 根环），所以这一对是只进内存那一趟预演续做时释放之前读盘核读的：头一次读进共用的那一份，真做释放之前读盘核取记下的那一份。
/// 读阶段可能也读过这个单元（它读的不进共用的那一份），所以按读序数认那一次，不按偏移认。交回盘 0 那一次。
fn the_last_released_unit_copy_read_by_the_rehearsal_on_device_zero(
    reads_reaching_the_disk: &[ReadReachingTheDisk],
) -> ReadReachingTheDisk {
    let read_after_the_first_write = |read: &ReadReachingTheDisk| {
        reads_reaching_the_disk.iter().any(|other| {
            !other.is_before_the_first_write_of_the_mount()
                && other.offset == read.offset
                && other.length_in_bytes == read.length_in_bytes
        })
    };
    reads_reaching_the_disk
        .windows(2)
        .filter_map(|consecutive| {
            let (on_device_zero, on_device_one) = (consecutive[0], consecutive[1]);
            let is_one_released_unit_read_on_both_devices = on_device_zero.device
                == DeviceIdentity(0)
                && on_device_one.device == DeviceIdentity(1)
                && on_device_zero.offset == on_device_one.offset
                && on_device_zero.length_in_bytes == on_device_one.length_in_bytes;
            (is_one_released_unit_read_on_both_devices
                && on_device_one.is_before_the_first_write_of_the_mount()
                && !read_after_the_first_write(&on_device_zero))
            .then_some(on_device_zero)
        })
        .next_back()
        .expect("第一个写之前有一对两块盘同一个偏移紧挨着的读、第一个写之后不再读它")
}

/// 场景：没改坏的窄盘镜像上，只进内存那一趟预演续做、释放之前读盘核把盘 0 上那一份头一次读进共用的那一份时读回坏字节（只坏这一次；
/// 读成功、整单元 CRC-32C 与位置项对不上，坏字节记进预演与真做共用的那一份），盘上是好的。
/// 预期：预演当场重读那一读穿到盘上（紧挨着那一次坏读，仍在这次挂载第一个写之前），读到对得上的字节、换掉记下的那一份；这次挂载第一个写之后
/// 再没有一次读到盘上的那一份（真做释放之前读盘核取的是换上的那一份）。挂载做成、续做 16 步走到空，没有一份被隔离；
/// 两块盘与不带故障的那一次挂载之后逐字节相同（池级 checker 判这一份镜像与判不带故障那一份的结论因此相同）。
/// 判别力：重读也取记下的那一份（`crates/mutations.tsv`「4-4 修红 W16 读坏当场重读：重读也取共用的那一份」）时预演与真做都核出对不上、
/// 把这个单元两块盘上的两份都隔离，交回的续做步里有隔离、盘面不同；重读读到的不换掉记下的那一份
/// （「4-4 修红 W16 读坏当场重读：重读读到自证过的不换掉记下的那一份」）时真做又读到坏字节、再穿到盘上重读一次，第一个写之后读到了那一份。
#[test]
fn a_released_unit_copy_read_back_corrupted_once_by_the_rehearsal_is_reread_from_the_disk_and_the_real_run_takes_the_good_copy(
) {
    let (image, geometry) = narrow_image_with_the_unlink_intent_of_a_sixteen_unit_file();
    let fault_free = mount_writable_on_recording_devices(&image, &geometry, None);
    let fault_free_mounted = fault_free
        .result
        .as_ref()
        .expect("前提：不带故障时没改坏的那一份挂载做成");
    assert_eq!(
        fault_free_mounted.output.deletion_queue_continuation.len(),
        CONTINUATION_STEPS_OF_THE_SIXTEEN_UNIT_FILE,
        "前提：不带故障时续做 16 步"
    );
    let copy_read_by_the_rehearsal =
        the_last_released_unit_copy_read_by_the_rehearsal_on_device_zero(
            &fault_free.reads_reaching_the_disk,
        );

    let faulted = mount_writable_on_recording_devices(
        &image,
        &geometry,
        Some(copy_read_by_the_rehearsal.ordinal_across_the_pool),
    );
    let [corrupted_read] = faulted.corrupted_reads.as_slice() else {
        panic!(
            "那一份恰读回坏字节一次：{:?}（选中的是 {copy_read_by_the_rehearsal:?}）",
            faulted.corrupted_reads
        );
    };
    assert!(
        corrupted_read.is_before_the_first_write_of_the_mount(),
        "读回坏字节的是第一个写之前那一趟（预演）：{corrupted_read:?}"
    );
    let position_of_the_corrupted_read = faulted
        .reads_reaching_the_disk
        .iter()
        .position(|read| read == corrupted_read)
        .expect("读回坏字节的那一次在读账里");
    let read_right_after = faulted.reads_reaching_the_disk[position_of_the_corrupted_read + 1];
    assert!(
        read_right_after.reads_the_same_bytes_as(corrupted_read)
            && read_right_after.is_before_the_first_write_of_the_mount(),
        "紧挨着的下一次读到盘上的就是当场重读那一份，仍在第一个写之前：{read_right_after:?}"
    );
    let reads_of_the_copy_after_the_first_write: Vec<&ReadReachingTheDisk> = faulted
        .reads_reaching_the_disk
        .iter()
        .filter(|read| {
            read.reads_the_same_bytes_as(corrupted_read)
                && !read.is_before_the_first_write_of_the_mount()
        })
        .collect();
    assert!(
        reads_of_the_copy_after_the_first_write.is_empty(),
        "第一个写之后真做取的是重读换上的那一份、不再读到盘上：{reads_of_the_copy_after_the_first_write:?}"
    );
    let mounted = faulted
        .result
        .as_ref()
        .expect("预演当场重读读到对得上的那一份：挂载做成");
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
    let quarantined: Vec<_> = mounted
        .output
        .deletion_queue_continuation
        .iter()
        .flat_map(|step| step.quarantined_after_release_checksum_mismatch.iter())
        .collect();
    assert!(
        quarantined.is_empty(),
        "一次瞬时坏读不让单元隔离：续做各步一份都没隔离（{quarantined:?}）"
    );
    assert!(
        faulted.images_after == fault_free.images_after,
        "两块盘与不带故障的那一次挂载之后逐字节相同"
    );
}

/// `narrow-384-20-2`（写行与暖机之后推抬 F）上盘 0 第二新那条自证过的根住的槽：重建两遍都读得出、不是被抛弃的根，这个进程知道那里住着一条根
/// （算上限那一遍读它读坏、重读仍坏就拒的那一类；盘 0 最新那条根住的槽是另一类，读坏重读仍坏拿记着的根顶上、不拒）。
fn ring_slot_of_the_second_newest_root_on_device_zero(
    image: &MemoryPool,
) -> singlefs_core::root_ring::RootRingSlot {
    let region_devices = choose_system_configuration(image)
        .expect("系统配置")
        .immutable
        .region_devices;
    let mut roots_on_device_zero: Vec<_> =
        scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::self_verified_roots_in_the_ring(image)
            .into_iter()
            .filter(|(ring_slot, _)| {
                region_devices[usize::try_from(ring_slot.region).expect("区域号")]
                    == DeviceIdentity(0)
            })
            .collect();
    roots_on_device_zero
        .sort_by_key(|(_, root)| std::cmp::Reverse((root.checkpoint_txg, root.instance)));
    roots_on_device_zero
        .get(1)
        .map(|(ring_slot, _)| *ring_slot)
        .expect("盘 0 上有两条自证过的根")
}

/// 场景：`narrow-384-20-2`，重建那一遍盘 1 每个根环槽报读错（整环重读一遍），盘 0 第二新那条根住的槽重建两遍都读得出（这个进程知道那里住着
/// 一条根）；可写挂载头一次把它读进预演与真做共用的那一份时读回乱字节（读成功、自证不过，记下的是坏字节），盘上是好的。
/// 写行、暖机之后推的抬 F 算上限那一遍读它取到记下的坏字节（这一遍读坏），当场重读穿到盘上读得出。
/// 预期：挂载做成，与那一槽重建之后一次都不读坏的再挂同一份盘面；那一槽第
/// [`scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::READ_OF_THE_SLOT_REREAD_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE`]
/// 次读到盘上的是抬 F 那一段写之前唯一一次读到盘上的根环槽读、它上一次读到盘上在第一个写之前，挂载结束时一共读到盘上这么多次
/// （重读读到自证过的换掉记下的那一份，之后再算一次上限、重算影子账取它）。
/// 判别力：算上限那一遍的重读也取记下的那一份（`crates/mutations.tsv`「4-4 修红 W16 读坏当场重读：抬 F 算上限读坏的重读走第一次读」）时
/// 重读仍坏、照「知道住着根的槽重读仍坏就拒」拒这一次抬 F，挂载在写行、暖机之后报 `FloorRaiseFailedAfterTheMountsPublishes`。
#[test]
fn a_known_root_slot_read_back_garbage_into_the_shared_reads_is_reread_from_the_disk_by_the_floor_raise_and_the_mount_is_made(
) {
    use scenarios_of_rebuilding_the_allocator_rereads_the_root_ring_once_when_one_device_reads_no_valid_root::{
        pool_before_the_remount, read_is_the_floor_raises_reread_alone_on_the_disk_between_two_writes,
        remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it,
        NarrowRemountThroughFaultedReads, ReadsOfTheSlotFailingAfterTheRebuildPass, StartingPoint,
        READ_OF_THE_SLOT_REREAD_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE,
    };
    let before = pool_before_the_remount(
        StartingPoint::UnitAreaOf384SlotsAfterTwentyOverwritesGrownToTwoUnits,
    );
    let known_root_slot =
        ring_slot_of_the_second_newest_root_on_device_zero(&before.image_to_remount);
    let NarrowRemountThroughFaultedReads {
        mounted: mounted_without_the_fault,
        image_after_the_remount: image_after_the_remount_without_the_fault,
        ..
    } = remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
        &before,
        known_root_slot,
        ReadsOfTheSlotFailingAfterTheRebuildPass::NeverAfterTheRebuildPass,
    );
    let mounted_without_the_fault =
        mounted_without_the_fault.expect("前提：那一槽重建之后一次都不读坏的再挂做得成");
    assert!(
        !mounted_without_the_fault
            .output
            .space_admission
            .floor_raises()
            .is_empty(),
        "前提：这次挂载写行与暖机之后推了抬 F"
    );
    let NarrowRemountThroughFaultedReads {
        mounted,
        image_after_the_remount,
        reads_of_each_slot_by_the_end_of_the_remount,
        reads_and_writes_in_order,
        ..
    } = remount_of_the_narrow_pool_with_device_one_failing_the_rebuild_pass_and_a_slot_after_it(
        &before,
        known_root_slot,
        ReadsOfTheSlotFailingAfterTheRebuildPass::BackAsGarbageOnlyOnItsFirstReadIntoTheReadsSharedInThisMount,
    );
    let mounted = mounted.unwrap_or_else(|error| {
        panic!("抬 F 当场重读穿到盘上读得出：挂载做成；挂载报的是 {error:?}")
    });
    assert!(
        !mounted.output.space_admission.floor_raises().is_empty(),
        "这次挂载写行与暖机之后推了抬 F"
    );
    assert_eq!(
        (
            read_is_the_floor_raises_reread_alone_on_the_disk_between_two_writes(
                &reads_and_writes_in_order,
                known_root_slot,
                READ_OF_THE_SLOT_REREAD_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE,
            ),
            reads_of_each_slot_by_the_end_of_the_remount
                .get(&known_root_slot)
                .copied(),
        ),
        (
            true,
            Some(READ_OF_THE_SLOT_REREAD_BY_THE_FIRST_CEILING_OF_THE_MOUNTS_FLOOR_RAISE),
        ),
        "那一槽在抬 F 那一段写之前当场重读穿到盘上、之后不再到盘上（读写次序：{reads_and_writes_in_order:?}）"
    );
    assert!(
        image_after_the_remount == image_after_the_remount_without_the_fault,
        "与那一槽重建之后一次都不读坏的再挂同一份盘面"
    );
}

//! 攻方原型（m3-b44-code-r2 攻方腿，只在副本上）：取号之前预演整个续做（W8、W10、W11）在带读故障注入的盘上。
//! 镜像照 `opening_a_container_past_the_largest_number_returns_an_error.rs` 窄盘那一条的造法（不补环窗口的窄盘 304 槽、
//! 第一个文件 16 个单元、删除队列 {1}、退役树唯一那片号改成 u64::MAX；没改坏的那一份上续做推过抬 F）。
//! 扫：挂载开始之后整池第 n 次读报错（`FaultSchedule::the_nth_call_across_the_pool(ReadFails, n)`），n 从 1 到这次挂载的读总数，
//! 每个 n 用一份新拷贝的盘挂一次，按结局分类并比盘。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

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
use singlefs_core::address::{DeviceIdentity, InodeNumber};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::container_tree::ContainerTreeNodeEntries;
use singlefs_core::lineage_tree::LineageContainerCapacity;
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, roll_back_by_a_forward_publish,
    MountError, RollbackTarget, ShadowLedger,
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
    FaultInjectingBlockDevice, FaultSchedule, InjectedFault, SharedFaultPlan,
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

fn move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
    image: &mut MemoryPool,
    geometry: &PoolImageGeometry,
    kind: u16,
    number_before: u64,
) {
    let container_slot = first_container_slot_of_the_tree_of_kind_of_the_newest_root(image, kind);
    change_the_unit_and_propagate_its_checksum_on(
        image,
        geometry,
        container_slot,
        data_unit_bytes(),
        |container| {
            assert_eq!(u64_at(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET), number_before);
            set_u64_at(container, PACKED_UNIT_CONTAINER_NUMBER_OFFSET, u64::MAX);
        },
    );
    let root_slot = root_slot_of_the_tree_of_kind_of_the_newest_root(image, kind);
    change_the_unit_and_propagate_its_checksum_on(image, geometry, root_slot, node_bytes(), |root| {
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
    });
}

/// 窄盘镜像（造法照 `opening_a_container_past_the_largest_number_returns_an_error.rs` 的
/// `narrow_pool_with_one_leaf_retired_and_the_sixteen_unit_first_file_queued_for_deletion`）：镜像、几何、退役树唯一那片的号。
fn narrow_image() -> (MemoryPool, PoolImageGeometry, u64) {
    let mut pool =
        start_on_narrow_devices_without_the_ring_window_allowance_with_the_space_admission_switched_off(
            HistoryDeviceWidth::UnitAreaOf384Slots,
            UNIT_AREA_SLOTS_OF_THE_NARROW_DEVICES,
        );
    pool.sequential_write(
        (DATA_UNITS_OF_THE_FIRST_FILE_ON_THE_NARROW_DEVICES - 1) * data_unit_payload_capacity() + 100,
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

type FaultDevices = Vec<(DeviceIdentity, FaultInjectingBlockDevice<SparseBlockDevice>)>;

fn fault_devices_on(image: &MemoryPool, plan: &SharedFaultPlan) -> FaultDevices {
    plain_devices_on(image)
        .into_iter()
        .map(|(identity, device)| {
            (identity, FaultInjectingBlockDevice::new(identity, device, plan.clone()))
        })
        .collect()
}

fn images_of(devices: &FaultDevices) -> Vec<(DeviceIdentity, SparseDevice)> {
    devices
        .iter()
        .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
        .collect()
}

fn fixed_geometry_of(geometry: &PoolImageGeometry) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: geometry.parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: geometry.parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: geometry.parameters.geometry.root_ring_slots_per_region,
    }
}

/// 一次挂载的结局，分到几类。
fn outcome_name(
    result: &Result<singlefs_core::mount::Mounted, MountError>,
) -> String {
    match result {
        Ok(_) => "Ok".to_owned(),
        Err(MountError::Publish(failed)) => {
            let ledgers_empty = failed.writes_of_persisted_publishes.is_empty()
                && failed.writes_of_failed_publishes.is_empty();
            match (&failed.cause, ledgers_empty) {
                (PublishError::NextContainerNumberPastTheTopOfItsRange(_), true) => {
                    "C841-before-acquisition".to_owned()
                }
                (PublishError::NextContainerNumberPastTheTopOfItsRange(_), false) => format!(
                    "C841-after-writes(persisted={})",
                    failed.writes_of_persisted_publishes.len()
                ),
                (other, _) => format!("Publish({})", first_word(&format!("{other:?}"))),
            }
        }
        Err(other) => first_word(&format!("{other:?}")),
    }
}

fn first_word(text: &str) -> String {
    text.split(|character: char| !character.is_alphanumeric() && character != '_')
        .next()
        .unwrap_or("")
        .to_owned()
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .map_or(default, |text| text.parse().expect("十进制整数"))
}

fn mount_on(
    devices: &mut FaultDevices,
    geometry: &PoolImageGeometry,
) -> Result<singlefs_core::mount::Mounted, MountError> {
    mount_writable_with_test_only_switches_and_deletion_switches(
        &geometry.parameters,
        devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(),
    )
}

/// 挂载开始之后整池读了几次才发第一个写：第一个写注入写错，挂载停在那里，读数就是取号之前的读。
fn reads_before_the_first_write(image: &MemoryPool, geometry: &PoolImageGeometry) -> u64 {
    let plan = SharedFaultPlan::armed(
        fixed_geometry_of(geometry),
        FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, 1),
    );
    let mut devices = fault_devices_on(image, &plan);
    let result = mount_on(&mut devices, geometry);
    println!("first-write-fault outcome={}", outcome_name(&result));
    plan.fired()
        .first()
        .map_or(u64::MAX, |_| plan.calls().reads)
}

fn scan(image: &MemoryPool, geometry: &PoolImageGeometry, label: &str) {
    let fixed = fixed_geometry_of(geometry);
    let unarmed = SharedFaultPlan::unarmed(fixed);
    let mut clean_devices = fault_devices_on(image, &unarmed);
    let clean = mount_on(&mut clean_devices, geometry);
    let total_reads = unarmed.calls().reads;
    let clean_images = images_of(&clean_devices);
    let original_images: Vec<(DeviceIdentity, SparseDevice)> = image
        .devices
        .iter()
        .map(|(identity, device)| (*identity, device.clone()))
        .collect();
    let before_first_write = reads_before_the_first_write(image, geometry);
    println!(
        "{label} fault-free outcome={} total_reads={total_reads} reads_before_the_first_write={before_first_write} fault_free_changed_disk={}",
        outcome_name(&clean),
        clean_images != original_images
    );
    drop(clean);
    let from = env_u64("PROBE_FROM", 1);
    let to = env_u64("PROBE_TO", total_reads).min(total_reads);
    let step = env_u64("PROBE_STEP", 1).max(1);
    let mut histogram: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let started = std::time::Instant::now();
    let mut n = from;
    while n <= to {
        let plan = SharedFaultPlan::armed(
            fixed,
            FaultSchedule::the_nth_call_across_the_pool(InjectedFault::ReadFails, n),
        );
        let mut devices = fault_devices_on(image, &plan);
        let result = mount_on(&mut devices, geometry);
        let name = outcome_name(&result);
        drop(result);
        let after = images_of(&devices);
        let disk = if after == original_images {
            "disk-unchanged"
        } else if after == clean_images {
            "disk-as-fault-free"
        } else {
            "disk-other"
        };
        let phase = if n <= before_first_write { "before-first-write" } else { "after-first-write" };
        let key = format!("{name} {disk} {phase} fired={}", plan.fired_count());
        println!("{label} n={n} {key}");
        *histogram.entry(key).or_insert(0) += 1;
        n += step;
    }
    println!("{label} scanned n={from}..={to} step={step} in {:?}", started.elapsed());
    for (key, count) in &histogram {
        println!("{label} summary {count} × {key}");
    }
}

#[test]
fn scan_read_faults_on_the_image_with_the_retirement_container_at_the_top_of_its_range() {
    let (mut image, geometry, number) = narrow_image();
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut image,
        &geometry,
        TREE_KIND_RETIREMENT,
        number,
    );
    scan(&image, &geometry, "corrupted");
}

#[test]
fn scan_read_faults_on_the_image_not_changed() {
    let (image, geometry, _) = narrow_image();
    scan(&image, &geometry, "unchanged");
}

/// 没改坏的那一份上挂载一次：写行、暖机、续做每一步、续做推的抬 F 各写了多少字节（预演在只进内存的盘上照样写这一串，
/// 那几块盘的写全留在内存里）。
#[test]
fn bytes_written_by_the_mount_after_acquisition() {
    let (image, geometry, _) = narrow_image();
    let unarmed = SharedFaultPlan::unarmed(fixed_geometry_of(&geometry));
    let mut devices = fault_devices_on(&image, &unarmed);
    let mounted = mount_on(&mut devices, &geometry).expect("没改坏的那一份上挂载做成");
    let output = &mounted.output;
    let steps: Vec<u64> = output
        .deletion_queue_continuation
        .iter()
        .map(|step| step.writes.total().written_bytes)
        .collect();
    let floor_raise_publishes: Vec<u64> = output
        .floor_raises_pushed_by_the_continuation
        .iter()
        .flat_map(|raised| raised.publishes.iter().map(|publish| publish.writes.total().written_bytes))
        .collect();
    let mut by_device: std::collections::BTreeMap<DeviceIdentity, u64> = std::collections::BTreeMap::new();
    for (identity, _) in &devices {
        by_device.insert(*identity, unarmed.counts_of_device(*identity).written_bytes);
    }
    println!(
        "continuation steps={} bytes_per_step={steps:?} sum={} floor_raise_publishes={floor_raise_publishes:?} device_written_bytes={by_device:?} total_reads={}",
        steps.len(),
        steps.iter().sum::<u64>(),
        unarmed.calls().reads
    );
}

/// J1 原型：所选那一版的那条记录（同实例、同 txg、带本次发布末条标志，`mount::record_standing_for_the_version` 取的那一条）新根段里的
/// 树表指针两条位置条目对调（设备 1 在前），记录头的宽校验和重算（载荷校验和只罩点名项，不变）；根槽与单元一字不改。
/// 重建（`recovery::rebuild_version_in_the_unit_area_starting_at`）在判 I-2.5 之前先 `record.to_bytes()`，写者断言在那里。
fn swap_the_location_entries_of_the_tree_table_pointer_in_the_record_of_the_newest_root(
    image: &mut MemoryPool,
    geometry: &PoolImageGeometry,
) -> usize {
    use singlefs_core::checksum::wide_checksum_with_field_zeroed;
    use singlefs_core::journal::{
        record_offset, JournalRecord, JournalRecordPlaceInPublish, JOURNAL_HEADER_CHECKSUM_OFFSET,
    };
    use singlefs_core::recovery::PoolReader;
    use singlefs_core::unit::unit_filesystem_identifier;
    let root = common_pointer_head_corruption::newest_root(image);
    let pointer = common_pointer_head_corruption::node_pointer_bytes(&root.tree_table);
    let fsid = unit_filesystem_identifier(&geometry.parameters.filesystem_identifier);
    let ring = geometry.parameters.geometry.journal_ring_bytes;
    let record_length = 4096_usize;
    let mut changed = 0;
    for device in [DeviceIdentity(0), DeviceIdentity(1)] {
        for counter in 1..=ring / 4096 {
            let offset = record_offset(counter, ring);
            let bytes = PoolReader::read(&*image, device, offset, record_length).expect("环里读得到");
            let Some(record) = JournalRecord::parse(&bytes, fsid) else {
                break;
            };
            if record.instance != root.instance
                || record.checkpoint_txg != root.checkpoint_txg
                || record.place_in_publish != JournalRecordPlaceInPublish::LastRecordOfThePublish
            {
                continue;
            }
            let start = (0..487 - 86)
                .find(|&at| bytes[at..at + 86] == pointer[..])
                .expect("新根段里有树表指针那 86 字节");
            let mut changed_bytes = bytes.clone();
            let first = start + 50;
            let second = first + 14;
            let first_entry: Vec<u8> = bytes[first..first + 14].to_vec();
            changed_bytes[first..first + 14].copy_from_slice(&bytes[second..second + 14]);
            changed_bytes[second..second + 14].copy_from_slice(&first_entry);
            let digest = wide_checksum_with_field_zeroed(
                &changed_bytes,
                record_length,
                JOURNAL_HEADER_CHECKSUM_OFFSET,
            );
            changed_bytes[JOURNAL_HEADER_CHECKSUM_OFFSET..JOURNAL_HEADER_CHECKSUM_OFFSET + 32]
                .copy_from_slice(&digest);
            let reparsed = JournalRecord::parse(&changed_bytes, fsid);
            println!(
                "device {device:?} counter {counter}: pointer at {start}, reparsed={} devices_in_order={:?}",
                reparsed.is_some(),
                reparsed.map(|record| [record.new_tree_table.locations[0].device, record.new_tree_table.locations[1].device])
            );
            image.devices.get_mut(&device).expect("有这块盘").write(offset, &changed_bytes);
            changed += 1;
        }
    }
    changed
}

#[test]
fn journal_record_whose_new_root_segment_pointer_does_not_ascend() {
    use singlefs_core::mount::mount_writable;
    let pool = common::build_pool("attack-journal-new-root-segment-not-ascending");
    let mut image = pool.memory_pool();
    let geometry = PoolImageGeometry::of_four_gibibyte_devices_under_the_common_parameters();
    let changed =
        swap_the_location_entries_of_the_tree_table_pointer_in_the_record_of_the_newest_root(
            &mut image, &geometry,
        );
    println!("journal records changed: {changed}");
    assert!(changed > 0, "改到了所选那一版的那条记录");
    let image_for_the_writable_mount = image.clone();
    let writable = std::panic::catch_unwind(move || {
        let mut devices = common_pointer_head_corruption::writable_devices_of(&image_for_the_writable_mount);
        let result = mount_writable(&common::parameters(), &mut devices);
        format!("{:?}", result.map(|mounted| mounted.output.instance))
    });
    println!("writable mount: {writable:?}");
    let image_for_the_read_only_mount = image.clone();
    let read_only = std::panic::catch_unwind(move || {
        let result = singlefs_core::mounted_read::mount_read_only(&image_for_the_read_only_mount);
        format!("{:?}", result.map(|_| "mounted read-only"))
    });
    println!("read-only mount: {read_only:?}");
}

/// 读故障只在一个窗口里：挂载开始之后整池第 `window_start` 到第 `window_end − 1` 次读都报块设备错（两块盘一起数），窗口之后照常
/// （一阵子读不出、过后读得出，两份镜像都撞上）。
struct WindowedReadFailure {
    inner: SparseBlockDevice,
    reads_across_the_pool: std::rc::Rc<std::cell::Cell<u64>>,
    window_start: u64,
    window_end: u64,
    failed_reads: std::rc::Rc<std::cell::Cell<u64>>,
}

impl singlefs_core::block_device::BlockDevice for WindowedReadFailure {
    fn read_at(
        &self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        let ordinal = self.reads_across_the_pool.get() + 1;
        self.reads_across_the_pool.set(ordinal);
        if self.window_start <= ordinal && ordinal < self.window_end {
            self.failed_reads.set(self.failed_reads.get() + 1);
            return Err(singlefs_harness::fault_injection::injected_block_device_error("读"));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        bytes: &[u8],
        durability: singlefs_core::block_device::WriteDurability,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(
        &mut self,
        offset: singlefs_core::address::DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> singlefs_core::block_device::PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
    fn journal_record_offsets_hint(
        &self,
        ring_start: singlefs_core::address::DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<singlefs_core::address::DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

fn scan_windows(image: &MemoryPool, geometry: &PoolImageGeometry, label: &str) {
    let original_images: Vec<(DeviceIdentity, SparseDevice)> = image
        .devices
        .iter()
        .map(|(identity, device)| (*identity, device.clone()))
        .collect();
    let from = env_u64("PROBE_FROM", 1);
    let to = env_u64("PROBE_TO", 1070);
    let widths: Vec<u64> = std::env::var("PROBE_WIDTHS")
        .unwrap_or_else(|_| "2".to_owned())
        .split(',')
        .map(|text| text.parse().expect("窗口宽"))
        .collect();
    let mut histogram: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    let started = std::time::Instant::now();
    for width in &widths {
        for start in from..=to {
            let reads = std::rc::Rc::new(std::cell::Cell::new(0));
            let failed = std::rc::Rc::new(std::cell::Cell::new(0));
            let mut devices: Vec<(DeviceIdentity, WindowedReadFailure)> = plain_devices_on(image)
                .into_iter()
                .map(|(identity, inner)| {
                    (
                        identity,
                        WindowedReadFailure {
                            inner,
                            reads_across_the_pool: reads.clone(),
                            window_start: start,
                            window_end: start + width,
                            failed_reads: failed.clone(),
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
            let name = outcome_name(&result);
            drop(result);
            let after: Vec<(DeviceIdentity, SparseDevice)> = devices
                .iter()
                .map(|(identity, device)| (*identity, device.inner.image.clone()))
                .collect();
            let disk = if after == original_images { "disk-unchanged" } else { "disk-changed" };
            let key = format!("width={width} {name} {disk} failed_reads={}", failed.get());
            println!("{label} start={start} {key}");
            *histogram.entry(key).or_insert(0) += 1;
        }
    }
    println!("{label} scanned starts {from}..={to} widths {widths:?} in {:?}", started.elapsed());
    for (key, count) in &histogram {
        println!("{label} summary {count} × {key}");
    }
}

#[test]
fn scan_read_fault_windows_on_the_image_with_the_retirement_container_at_the_top_of_its_range() {
    let (mut image, geometry, number) = narrow_image();
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut image,
        &geometry,
        TREE_KIND_RETIREMENT,
        number,
    );
    scan_windows(&image, &geometry, "corrupted-window");
}

fn windowed_devices(
    image: &MemoryPool,
    start: u64,
    width: u64,
) -> (Vec<(DeviceIdentity, WindowedReadFailure)>, std::rc::Rc<std::cell::Cell<u64>>) {
    let reads = std::rc::Rc::new(std::cell::Cell::new(0));
    let failed = std::rc::Rc::new(std::cell::Cell::new(0));
    let devices = plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| {
            (
                identity,
                WindowedReadFailure {
                    inner,
                    reads_across_the_pool: reads.clone(),
                    window_start: start,
                    window_end: start + width,
                    failed_reads: failed.clone(),
                },
            )
        })
        .collect();
    (devices, failed)
}

fn image_of(devices: &[(DeviceIdentity, WindowedReadFailure)], device_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.inner.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    }
}

fn mount_windowed(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    start: u64,
    width: u64,
) -> (String, MemoryPool, u64) {
    let (mut devices, failed) = windowed_devices(image, start, width);
    let result = mount_writable_with_test_only_switches_and_deletion_switches(
        &geometry.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(),
    );
    let name = match &result {
        Err(MountError::Publish(failed)) => format!(
            "{} cause={:?}",
            outcome_name(&result),
            failed.cause
        ),
        Ok(mounted) => format!(
            "Ok instance={:?} continuation_steps={} outcome={:?}",
            mounted.output.instance,
            mounted.output.deletion_queue_continuation.len(),
            mounted.output.deletion_queue_continuation_outcome
        ),
        Err(_) => outcome_name(&result),
    };
    drop(result);
    (name, image_of(&devices, image.device_size_in_bytes), failed.get())
}

/// 读故障窗口落在取号之前预演里的一格：这一次挂载取号、写行、续做走到最后一批才交 C841；之后不带故障再挂两次。
/// 没改坏的那一份上同一个窗口：挂载结局与盘和不带故障的那一次比。
#[test]
fn a_read_fault_window_inside_the_rehearsal_then_two_plain_remounts() {
    let starts: Vec<u64> = std::env::var("PROBE_STARTS")
        .unwrap_or_else(|_| "340,960".to_owned())
        .split(',')
        .map(|text| text.parse().expect("起点"))
        .collect();
    let width = env_u64("PROBE_WIDTH", 2);
    let (unchanged_image, geometry, number) = narrow_image();
    let mut corrupted_image = unchanged_image.clone();
    move_the_only_container_of_the_tree_of_kind_to_the_top_of_its_range(
        &mut corrupted_image,
        &geometry,
        TREE_KIND_RETIREMENT,
        number,
    );
    let (fault_free_outcome, fault_free_after, _) =
        mount_windowed(&unchanged_image, &geometry, u64::MAX - 8, 2);
    println!("unchanged image, no fault: {fault_free_outcome}");
    for start in starts {
        let (first, after_first, failed) = mount_windowed(&corrupted_image, &geometry, start, width);
        println!(
            "corrupted start={start} width={width}: first mount {first} failed_reads={failed} disk_changed={}",
            after_first.devices != corrupted_image.devices
        );
        let (second, after_second, _) = mount_windowed(&after_first, &geometry, u64::MAX - 8, 2);
        println!(
            "corrupted start={start}: second mount without fault {second} disk_changed_by_it={}",
            after_second.devices != after_first.devices
        );
        let (third, after_third, _) = mount_windowed(&after_second, &geometry, u64::MAX - 8, 2);
        println!(
            "corrupted start={start}: third mount without fault {third} disk_changed_by_it={}",
            after_third.devices != after_second.devices
        );
        let (on_unchanged, after_on_unchanged, failed_on_unchanged) =
            mount_windowed(&unchanged_image, &geometry, start, width);
        println!(
            "unchanged start={start} width={width}: {on_unchanged} failed_reads={failed_on_unchanged} disk_same_as_fault_free={}",
            after_on_unchanged.devices == fault_free_after.devices
        );
    }
}

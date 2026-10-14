//! 攻方腿 m3-admission-r1-opus 的探针共用：读分配器计数、拷池、摘要。只在冻结副本的拷贝上跑，不进仓库的 crates/。
#![allow(dead_code)]

use std::collections::BTreeSet;

use singlefs_core::address::DeviceIdentity;
use singlefs_core::admission::admission_reading_before_a_publish;
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::mounted_session::MountedSession;
use singlefs_core::transaction::PoolVersion;
use singlefs_format::SLOT_BYTES;
use singlefs_harness::memory_pool::SparseBlockDevice;

use std::collections::BTreeMap;

use singlefs_core::admission::SpaceAdmission;
use singlefs_core::allocator::UnitAreaStart;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{allocator_after_make_filesystem, make_filesystem};
use singlefs_core::mount::{ParametersAndDeviceTableOfTheMount, ShadowLedger};
use singlefs_core::transaction::{acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter};

use super::common_admission::{
    content_of, parameters_of_the_384_slot_small_devices, root_key, DeviceOverASparseImage, PoolUnderTest,
    FIRST_WRITE_TIME_SECONDS,
};

/// 一块盘此刻的计数：容量、已分配、隔离、两者重叠、没挡的、只被扣住挡的（不在已分配与隔离里）、段外槽对、全部槽对。
#[derive(Clone, Copy, Debug)]
pub struct DeviceCounts {
    pub device: DeviceIdentity,
    pub capacity: u64,
    pub allocated: u64,
    pub isolated: u64,
    pub allocated_and_isolated: u64,
    pub unblocked: u64,
    pub held_only: u64,
    pub pairs_outside_segments: u64,
    pub pairs_total: u64,
    pub cluster_segments: usize,
    pub formula_available_slots: i128,
}

pub fn counts_of(allocator: &PoolAllocator, current: &PoolVersion) -> Vec<DeviceCounts> {
    let reading = admission_reading_before_a_publish(allocator, current.file_version());
    let available = reading.available_on_each_device();
    allocator
        .devices
        .iter()
        .map(|map| {
            let capacity = map.unit_area_slots();
            let allocated = map.allocated_slots();
            let isolated = map.isolated_slots();
            let both = map.slots_counted_both_as_allocated_and_as_isolated();
            let unblocked = map.unblocked_slots();
            let blocked = capacity - unblocked;
            let allocated_or_isolated = allocated + isolated - both;
            let available_bytes = available
                .iter()
                .find(|(device, _)| *device == map.device)
                .expect("读数里每块盘一条")
                .1
                 .0;
            DeviceCounts {
                device: map.device,
                capacity,
                allocated,
                isolated,
                allocated_and_isolated: both,
                unblocked,
                held_only: blocked - allocated_or_isolated,
                pairs_outside_segments: map
                    .unblocked_slot_pairs_outside_the_cluster_segments(allocator.cluster_segments()),
                pairs_total: map.unblocked_slot_pairs_outside_the_cluster_segments(&BTreeSet::new()),
                cluster_segments: allocator.cluster_segments().len(),
                formula_available_slots: available_bytes / i128::from(SLOT_BYTES),
            }
        })
        .collect()
}

pub fn line_of(tag: &str, counts: &[DeviceCounts]) -> String {
    let mut line = format!("PROBE {tag}");
    for c in counts {
        line.push_str(&format!(
            " | d{} cap={} alloc={} iso={} alloc&iso={} unblocked={} held_only={} pairs_out={} pairs_all={} segs={} formula_avail_slots={}",
            c.device.0,
            c.capacity,
            c.allocated,
            c.isolated,
            c.allocated_and_isolated,
            c.unblocked,
            c.held_only,
            c.pairs_outside_segments,
            c.pairs_total,
            c.cluster_segments,
            c.formula_available_slots
        ));
    }
    line
}

/// 拷一份池（盘面与会话），改拷贝不动原来那份。每块盘照它自己的字节数拷（盘可以不等大）。
pub fn clone_pool<D: DeviceOverASparseImage>(pool: &PoolUnderTest<D>) -> PoolUnderTest<SparseBlockDevice> {
    let session = pool.session();
    PoolUnderTest {
        parameters: pool.parameters.clone(),
        device_bytes: pool.device_bytes,
        devices: pool
            .devices
            .iter()
            .map(|(identity, device)| {
                let mut copy = SparseBlockDevice::new(device.size_in_bytes(), PhysicalBlockSizeInBytes(512));
                copy.image = device.sparse_image().clone();
                (*identity, copy)
            })
            .collect(),
        session: Some(MountedSession {
            allocator: session.allocator.clone(),
            current: session.current.clone(),
            instance: session.instance,
            shadow_ledger: session.shadow_ledger,
            parameters_and_device_table: session.parameters_and_device_table.clone(),
        }),
        write_time_seconds: pool.write_time_seconds,
        content_of_the_current_version: pool.content_of_the_current_version.clone(),
        content_of_each_root: pool.content_of_each_root.clone(),
    }
}

/// 两块盘各自的单元区槽数与 journal 环字节数（其余参数取 384 槽那一档）上起池：mkfs、取号、暖机、第一个文件（与
/// `common_admission_pool` 的 `start_after_the_first_file_on_devices_of` 同一套步骤，只是两块盘可以不等大）。
pub fn start_pool(unit_area_slots: [u64; 2], journal_ring_bytes: u64) -> PoolUnderTest<SparseBlockDevice> {
    let mut parameters = parameters_of_the_384_slot_small_devices();
    parameters.geometry.journal_ring_bytes = journal_ring_bytes;
    let start = UnitAreaStart::following_the_journal_ring(journal_ring_bytes)
        .expect("环末尾在段边界上")
        .slot()
        .0;
    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> = [0u32, 1]
        .into_iter()
        .map(|index| {
            let bytes = (start + unit_area_slots[index as usize]) * SLOT_BYTES;
            (DeviceIdentity(index), SparseBlockDevice::new(bytes, PhysicalBlockSizeInBytes(512)))
        })
        .collect();
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    allocator.set_space_admission(SpaceAdmission::JudgedByTheFormula);
    let first_content = content_of(3000, 0);
    let (instance, first_version) = {
        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
        let instance = acquire_instance(&mut writer).expect("取号 1");
        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
        let first_version = publish_first_file(
            &mut writer,
            &mut allocator,
            warmed.roots.last().expect("暖机写了两条根"),
            FirstFile { content: &first_content, write_time_seconds: FIRST_WRITE_TIME_SECONDS },
            instance,
            &warmed.last_record_bytes,
        )
        .expect("第一个文件");
        (instance, first_version)
    };
    let mut content_of_each_root = BTreeMap::new();
    content_of_each_root.insert(root_key(&first_version.root), first_content.clone());
    let parameters_and_device_table =
        ParametersAndDeviceTableOfTheMount::of_a_pool_on_disk(&devices).expect("盘表");
    PoolUnderTest {
        parameters,
        device_bytes: (start + unit_area_slots[0]) * SLOT_BYTES,
        devices,
        session: Some(MountedSession {
            allocator,
            current: PoolVersion::WithFile(first_version),
            instance,
            shadow_ledger: ShadowLedger::On,
            parameters_and_device_table,
        }),
        write_time_seconds: FIRST_WRITE_TIME_SECONDS,
        content_of_the_current_version: first_content,
        content_of_each_root,
    }
}


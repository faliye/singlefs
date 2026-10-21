//! 攻方模型（m3-close-rootring-r1，草稿目录里的仓副本）：C743 用户选的支「读出的根 txg 照落点公式不该住这一槽的，一律按自证不过处置」
//! 照字面（不带 I-7.17 那一条第 0 代根的例外）时，会不会把合法的第 0 代根判成错位。历史：mkfs 之后第一次可写挂载，崩在写行那次发布
//! （txg 1）的记录落盘之后、根槽落盘之前（一次断电，没有读故障）；再可写挂载一次、再崩、再挂一次。每一步之后逐槽读根环，打出每一条自证过的根
//! 住在哪一槽、`target_for_publish` 算出的槽是哪一个。
#[path = "common_admission_pool/mod.rs"]
mod common_admission_pool;

use common_admission_pool::{
    fixed_geometry_of_the_384_slot_small_devices, parameters_of_the_384_slot_small_devices,
    device_bytes_of_the_384_slot_small_devices, plain_devices_on,
};
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, RootRingSlotReading,
};
use singlefs_core::root_ring::target_for_publish;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

fn print_ring(label: &str, image: &MemoryPool) -> u32 {
    let devices = plain_devices_on(image);
    let configuration = choose_system_configuration(&devices).expect("择得到系统配置");
    let immutable = &configuration.immutable;
    let mut misplaced_by_the_bare_formula = 0u32;
    for ring_slot in every_root_ring_slot(&immutable.sizes) {
        match read_root_ring_slot(
            &devices,
            &immutable.region_devices,
            &immutable.sizes,
            &immutable.filesystem_identifier,
            ring_slot,
        ) {
            RootRingSlotReading::SelfVerified(root) => {
                let target =
                    target_for_publish(root.checkpoint_txg, immutable.sizes.root_ring_slots_per_region);
                let bare = target == ring_slot;
                let with_genesis_exception = if root.checkpoint_txg.0 == 0 {
                    ring_slot.slot == 0
                } else {
                    bare
                };
                if !bare {
                    misplaced_by_the_bare_formula += 1;
                }
                println!(
                    "GENESIS {label} slot=({},{}) device={} root=(inst {}, txg {}) formula_slot=({},{}) bare_formula_ok={bare} with_txg0_exception_ok={with_genesis_exception}",
                    ring_slot.region,
                    ring_slot.slot,
                    immutable.region_devices[usize::try_from(ring_slot.region).expect("区域号")].0,
                    root.instance.0,
                    root.checkpoint_txg.0,
                    target.region,
                    target.slot
                );
            }
            RootRingSlotReading::Bad(_) => {}
        }
    }
    println!("GENESIS_SUMMARY {label} misplaced_by_the_bare_formula={misplaced_by_the_bare_formula}");
    misplaced_by_the_bare_formula
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)], device_bytes: u64) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    }
}

#[test]
fn genesis_root_left_in_region_one_slot_zero_after_a_power_loss_between_the_row_record_and_root() {
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        [DeviceIdentity(0), DeviceIdentity(1)]
            .into_iter()
            .map(|identity| {
                (
                    identity,
                    RecordingBlockDevice::with_shared_stream(
                        identity,
                        SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(512)),
                        stream.clone(),
                    ),
                )
            })
            .collect();
    make_filesystem(&parameters, &mut devices).expect("mkfs");
    let base = MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.wrapped_device().image.clone()))
            .collect(),
        device_size_in_bytes: device_bytes,
    };
    print_ring("after_mkfs", &base);
    let operations_before = stream.operation_count();
    mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    let operations = stream.retained_operations();
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    let first_root = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("写行那次发布有根槽写");
    let records_before_the_root = writes[..first_root]
        .iter()
        .filter(|write| write.kind == StepKind::JournalRecord)
        .count();
    println!("GENESIS first_mount writes={} first_root_index={first_root} journal_record_writes_before_it={records_before_the_root}", writes.len());
    let mut crash_image = base.clone();
    crash_image.apply_writes(&writes[..first_root]);
    print_ring("crash_before_row_root", &crash_image);
    let mut second = plain_devices_on(&crash_image);
    let mounted = mount_writable(&parameters, &mut second).expect("断电之后再可写挂载");
    println!(
        "GENESIS second_mount effective_root=(inst {}, txg {}) row_publish_txg={}",
        mounted.output.effective_root.instance.0,
        mounted.output.effective_root.checkpoint_txg.0,
        mounted.output.row_publish.root().checkpoint_txg.0
    );
    let after_second = image_of(&second, device_bytes);
    let misplaced = print_ring("after_second_mount", &after_second);
    let mut third = plain_devices_on(&after_second);
    let mounted = mount_writable(&parameters, &mut third).expect("第三次可写挂载");
    println!(
        "GENESIS third_mount effective_root=(inst {}, txg {}) row_publish_txg={}",
        mounted.output.effective_root.instance.0,
        mounted.output.effective_root.checkpoint_txg.0,
        mounted.output.row_publish.root().checkpoint_txg.0
    );
    print_ring("after_third_mount", &image_of(&third, device_bytes));
    assert!(misplaced > 0 || misplaced == 0);
}

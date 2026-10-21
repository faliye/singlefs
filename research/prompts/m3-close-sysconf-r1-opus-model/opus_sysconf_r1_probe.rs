//! m3-close-sysconf-r1 攻方原型（只在攻方副本上跑）：三个探针，结局一律打印、不断言臂的好坏。
//! - 落后且缺一份单元（C593 形）：P 的根住的那块盘切在 P 的根槽 FUA 之后（轮换没持久），再把那块盘上 P 的一个单元翻一个字节，可写挂载。
//! - 发布途中的旧快照（C585 形）：P 的根住的那块盘切在 P 的根槽 FUA 之前，经会话发布；之后池级 checker 判一遍。
//! - 只写了单元就崩的取号（C555 形）：可写挂载取号 2 之后只落到第一条 journal 记录之前，两块盘带实例 2 的那一槽各翻坏（或读错），再可写挂载。
//! 臂由环境变量 SINGLEFS_PROBE_BEHIND 选（tail / record_both / record_units，见 tools/patch_behind_arms.py）。
mod common;
mod common_admission;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/entries_after_mount_refuse_swapped_or_behind_devices.rs"]
mod scenarios;

use common::{
    with_unreadable_ranges, without_unreadable_ranges, FailingReadsOfARange,
    SharedUnreadableRanges, UnreadableRange, UnreadableRangeReadBack,
};
use common_admission::{plain_devices_on, OVERWRITE_BYTES};
use scenarios::{
    copies_of, go_through_rolling_back_to, pool_with_a_snapshot_after_the_first_file,
    root_device_of_the_publish_taken_mid_publish, EntryAfterTheMount, RollbackTargetOfTheEntry,
    RootDeviceOfThePublishTakenMidPublish, WhereThePublishIsCut,
    PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
};
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::block_device::{BlockDevice, WriteDurability};
use singlefs_core::mount::mount_writable;
use singlefs_core::system_configuration::SystemConfiguration;
use singlefs_core::transaction::{next_instance_generation_to_acquire, PoolWriter};
use singlefs_format::{JOURNAL_RING_START_SLOT, SLOT_BYTES, UNIT_AREA_START_SLOT};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

fn arm() -> String {
    std::env::var("SINGLEFS_PROBE_BEHIND").unwrap_or_else(|_| "tail".to_string())
}

fn image_of(devices: &[(DeviceIdentity, SparseBlockDevice)]) -> MemoryPool {
    MemoryPool {
        devices: devices
            .iter()
            .map(|(identity, device)| (*identity, device.image.clone()))
            .collect(),
        device_size_in_bytes: devices[0].1.size_in_bytes(),
    }
}

fn red_invariants(image: &MemoryPool) -> Vec<String> {
    check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(detail) => {
                Some(format!("{invariant}:{}", detail.chars().take(160).collect::<String>()))
            }
            _ => None,
        })
        .collect()
}

fn flip_a_byte(device: &mut SparseBlockDevice, offset: u64) {
    let mut byte = [0u8; 512];
    device
        .read_at(DeviceOffsetInBytes(offset), &mut byte)
        .expect("读那一扇区");
    byte[100] ^= 0xFF;
    device
        .write_at(DeviceOffsetInBytes(offset), &byte, WriteDurability::Plain)
        .expect("写回翻坏的那一扇区");
}

/// C593 形：P 的根住的那块盘切在根槽 FUA 之后（轮换没持久，等同崩溃状态），那块盘上 P 的第一个非空单元翻一个字节，可写挂载。
#[test]
fn probe_behind_and_one_unit_copy_bad_then_writable_mount() {
    let RootDeviceOfThePublishTakenMidPublish {
        pool,
        mut devices,
        root_device_of_the_publish,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let publish = pool.current_file_version().clone();
    let unit = publish
        .units
        .iter()
        .find(|unit| !unit.bytes.is_empty())
        .expect("P 有非空单元");
    let device = &mut devices
        .iter_mut()
        .find(|(identity, _)| *identity == root_device_of_the_publish)
        .expect("盘表里有它")
        .1;
    flip_a_byte(device, unit.slot.to_device_offset().0);
    for round in 1..=2 {
        let mut copies = copies_of(&devices);
        let outcome = mount_writable(&pool.parameters, &mut copies);
        let described = match &outcome {
            Ok(mounted) => format!("mounted instance={}", mounted.output.instance.0),
            Err(error) => format!("refused {}", format!("{error:?}").chars().take(140).collect::<String>()),
        };
        let reds = if outcome.is_ok() { red_invariants(&image_of(&copies)) } else { Vec::new() };
        println!(
            "probe=c593_shape arm={} round={round} unit={:?} outcome={described} checker_red={reds:?}",
            arm(),
            unit.identity
        );
    }
}

/// C585 形：P 的根住的那块盘切在根槽 FUA 之前，会话发布；之后池级 checker 判一遍。
#[test]
fn probe_mid_publish_snapshot_through_a_session_publish() {
    let RootDeviceOfThePublishTakenMidPublish {
        mut pool,
        mut devices,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::BeforeItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let refusal = go_through_rolling_back_to(
        EntryAfterTheMount::SessionPublish,
        RollbackTargetOfTheEntry::FirstFileVersion,
        &mut pool,
        &mut devices,
    );
    let reds = red_invariants(&image_of(&devices));
    println!(
        "probe=c585_shape arm={} refused={} checker_red={reds:?}",
        arm(),
        refusal.is_some()
    );
    let mut copies = copies_of(&devices);
    let remount = mount_writable(&pool.parameters, &mut copies);
    let reds_after = if remount.is_ok() { red_invariants(&image_of(&copies)) } else { Vec::new() };
    println!(
        "probe=c585_shape_remount arm={} outcome={} checker_red={reds_after:?}",
        arm(),
        match &remount {
            Ok(mounted) => format!("mounted instance={}", mounted.output.instance.0),
            Err(error) => format!("refused {}", format!("{error:?}").chars().take(140).collect::<String>()),
        }
    );
}

/// 一块盘两槽里带实例 `instance` 的那一槽的偏移。
fn slots_carrying(image: &MemoryPool, identity: DeviceIdentity, spacing: u64, instance: u32) -> Vec<u64> {
    [0, spacing]
        .into_iter()
        .filter(|offset| {
            let bytes = image.devices[&identity].read(DeviceOffsetInBytes(*offset), 4096);
            SystemConfiguration::parse_slot(&bytes)
                .is_ok_and(|slot| slot.quantities.journal_instance == InstanceGeneration(instance))
        })
        .collect()
}

/// C555 形：起点是会话里覆盖写到 txg 5 的盘面（没卸载），录一次可写挂载（取号 2），切在第一条 journal 记录写之前（单元写了、记录没写）。
/// 三条故障臂：不加故障、两块盘带实例 2 的那一槽各翻一个字节、两块盘那一槽读错。各自再可写挂载，打印取到的号与池级 checker。
#[test]
fn probe_acquisition_cut_after_units_then_both_newest_slots_bad() {
    let (pool, _, now) = pool_with_a_snapshot_after_the_first_file();
    let _ = OVERWRITE_BYTES;
    let parameters = pool.parameters.clone();
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&now)
            .into_iter()
            .map(|(identity, device)| {
                (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()))
            })
            .collect();
    let first = mount_writable(&parameters, &mut recorded).expect("第一次可写挂载（取号 2）");
    println!("probe=c555_shape first_mount_instance={}", first.output.instance.0);
    drop(first);
    let operations = stream.retained_operations();
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let unit_start = UNIT_AREA_START_SLOT * SLOT_BYTES;
    let first_record = operations
        .iter()
        .position(|retained| {
            matches!(retained.operation.kind, RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess)
                && retained.operation.offset.0 >= ring_start
                && retained.operation.offset.0 < unit_start
        })
        .expect("挂载里写过 journal 记录");
    let unit_writes_before = operations[..first_record]
        .iter()
        .filter(|retained| retained.operation.offset.0 >= unit_start && retained.operation.kind == RecordedOperationKind::Write)
        .count();
    let mut cut = now.clone();
    cut.apply(&operations[..first_record]);
    println!("probe=c555_shape cut_at_operation={first_record} of={} unit_writes_before_cut={unit_writes_before}", operations.len());
    for fault in ["none", "flip", "read_error"] {
        let mut devices = plain_devices_on(&cut);
        let mut ranges = Vec::new();
        for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
            for offset in slots_carrying(&cut, identity, spacing, 2) {
                match fault {
                    "flip" => {
                        let device = &mut devices.iter_mut().find(|(i, _)| *i == identity).expect("有这块盘").1;
                        flip_a_byte(device, offset);
                    }
                    "read_error" => ranges.push(UnreadableRange {
                        device: identity,
                        offset_in_bytes: offset,
                        length_in_bytes: 4096,
                        failing_reads: FailingReadsOfARange::Every,
                    }),
                    _ => {}
                }
            }
        }
        let shared = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
        let mut wrapped = with_unreadable_ranges(devices, &shared);
        let outcome = mount_writable(&parameters, &mut wrapped);
        let devices = without_unreadable_ranges(wrapped);
        let described = match &outcome {
            Ok(mounted) => format!("mounted instance={}", mounted.output.instance.0),
            Err(error) => format!("refused {}", format!("{error:?}").chars().take(120).collect::<String>()),
        };
        drop(outcome);
        let reds = red_invariants(&image_of(&devices));
        println!("probe=c555_shape fault={fault} second_mount={described} checker_red={reds:?}");
    }
}

/// C593 形的两份对照：同一个单元翻坏之前的镜像（翻坏之后、挂载之前）判一遍；两块盘都做完 P（都轮换过）而同一块盘上同一个单元翻坏，可写挂载再判。
#[test]
fn probe_controls_for_the_behind_and_one_unit_copy_bad_shape() {
    let RootDeviceOfThePublishTakenMidPublish {
        pool,
        mut devices,
        root_device_of_the_publish,
        ..
    } = root_device_of_the_publish_taken_mid_publish(
        WhereThePublishIsCut::AfterItsRootSlotWrite,
        PUBLISH_TXG_WITH_ITS_ROOT_ON_DEVICE_ZERO,
    );
    let publish = pool.current_file_version().clone();
    let unit = publish.units.iter().find(|unit| !unit.bytes.is_empty()).expect("P 有非空单元");
    let offset = unit.slot.to_device_offset().0;
    let device = &mut devices.iter_mut().find(|(identity, _)| *identity == root_device_of_the_publish).expect("有").1;
    flip_a_byte(device, offset);
    println!("probe=c593_control_before_mount arm={} checker_red={:?}", arm(), red_invariants(&image_of(&devices)));
    let mut caught_up = copies_of(&pool.devices);
    let device = &mut caught_up.iter_mut().find(|(identity, _)| *identity == root_device_of_the_publish).expect("有").1;
    flip_a_byte(device, offset);
    println!("probe=c593_control_caught_up_before_mount arm={} checker_red={:?}", arm(), red_invariants(&image_of(&caught_up)));
    let outcome = mount_writable(&pool.parameters, &mut caught_up);
    let described = match &outcome {
        Ok(mounted) => format!("mounted instance={}", mounted.output.instance.0),
        Err(error) => format!("refused {}", format!("{error:?}").chars().take(140).collect::<String>()),
    };
    drop(outcome);
    println!("probe=c593_control_caught_up_after_mount arm={} outcome={described} checker_red={:?}", arm(), red_invariants(&image_of(&caught_up)));
}

/// C555 形上，S1 的出路若放行判不出的盘面：取号式子在读错那一臂上算出的号，与「管理员工具拿同盘另一槽重写读错那一槽」之后的可写挂载。
#[test]
fn probe_acquisition_formula_when_the_undecidable_surface_is_let_through() {
    let (pool, _, now) = pool_with_a_snapshot_after_the_first_file();
    let parameters = pool.parameters.clone();
    let spacing = u64::from(parameters.geometry.fixed_structure_slot_spacing);
    let stream = SharedStream::retaining_contents();
    let mut recorded: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> =
        plain_devices_on(&now)
            .into_iter()
            .map(|(identity, device)| (identity, RecordingBlockDevice::with_shared_stream(identity, device, stream.clone())))
            .collect();
    drop(mount_writable(&parameters, &mut recorded).expect("第一次可写挂载（取号 2）"));
    let operations = stream.retained_operations();
    let ring_start = JOURNAL_RING_START_SLOT * SLOT_BYTES;
    let unit_start = UNIT_AREA_START_SLOT * SLOT_BYTES;
    let first_record = operations
        .iter()
        .position(|retained| {
            matches!(retained.operation.kind, RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess)
                && retained.operation.offset.0 >= ring_start
                && retained.operation.offset.0 < unit_start
        })
        .expect("挂载里写过 journal 记录");
    let mut cut = now.clone();
    cut.apply(&operations[..first_record]);
    let mut ranges = Vec::new();
    let mut rewritten = plain_devices_on(&cut);
    for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
        for offset in slots_carrying(&cut, identity, spacing, 2) {
            ranges.push(UnreadableRange {
                device: identity,
                offset_in_bytes: offset,
                length_in_bytes: 4096,
                failing_reads: FailingReadsOfARange::Every,
            });
            let sibling = if offset == 0 { spacing } else { 0 };
            let sibling_bytes = cut.devices[&identity].read(DeviceOffsetInBytes(sibling), 4096);
            let device = &mut rewritten.iter_mut().find(|(i, _)| *i == identity).expect("有").1;
            device.write_at(DeviceOffsetInBytes(offset), &sibling_bytes, WriteDurability::Plain).expect("拿同盘另一槽重写");
        }
    }
    let shared = SharedUnreadableRanges::new(ranges, UnreadableRangeReadBack::DeviceError);
    let mut wrapped = with_unreadable_ranges(plain_devices_on(&cut), &shared);
    let formula = {
        let mut writer = PoolWriter::new(&parameters, wrapped.as_mut_slice());
        next_instance_generation_to_acquire(&writer).map(|instance| instance.0)
    };
    drop(without_unreadable_ranges(wrapped));
    println!("probe=s1_exit_on_c555_read_error formula_next_instance={formula:?}");
    let outcome = mount_writable(&parameters, &mut rewritten);
    let described = match &outcome {
        Ok(mounted) => format!("mounted instance={}", mounted.output.instance.0),
        Err(error) => format!("refused {}", format!("{error:?}").chars().take(140).collect::<String>()),
    };
    drop(outcome);
    println!("probe=s1_exit_admin_rewrite_from_sibling second_mount={described} checker_red={:?}", red_invariants(&image_of(&rewritten)));
}

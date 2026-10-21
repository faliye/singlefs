//! 正推腿模型（m3-close-rootring-r2，草稿目录里的仓副本，核心层带 `r2_switch` 开关）：Y1。C743 的形：读根环槽读出的根，txg 照落点公式不该住这一槽的按自证不过处置，
//! 第 0 代根只认槽 0（与 I-7.17 同一例外）。开关 `formula`：0 今天（不核），1 裸公式（第一轮用户那一支字面），2 带第 0 代例外（这一轮的形）。
//! 问两件事：（一）合法历史上的每一张镜像、每一个自证过的根，开关 1、2 各会不会把它判成错位（`rejected_valid`）；
//! （二）开关 2 下恢复、只读挂载、可写挂载的结局与今天（开关 0）逐镜像比，有没有变。再放两个对照：往合法镜像里手写一条错位的非第 0 代根
//! （开关 1、2 都得拒，证明核会红），往 (区域 1, 槽 0) 手写区域 0 槽 0 那份第 0 代根的字节（开关 2 放行，这是例外留下的盲区）。
mod common_unwitnessed_publish;

use common_unwitnessed_publish::*;
use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::make_filesystem;
use singlefs_core::mount::mount_writable;
use singlefs_core::mounted_read::mount_read_only;
use singlefs_core::r2_switch::set_formula;
use singlefs_core::recovery::{
    choose_system_configuration, every_root_ring_slot, read_root_ring_slot, recover, JournalPolicy,
    RecoveryOutcome, RootRingSlotReading,
};
use singlefs_core::root_ring::slot_offset;
use singlefs_harness::memory_pool::{
    writes_and_segments_with_stream_indexes, MemoryPool, RetainedWrite, SparseBlockDevice,
};
use singlefs_harness::segments::StepKind;
use singlefs_harness::{RecordingBlockDevice, SharedStream};

fn self_verified_slots(image: &MemoryPool) -> Vec<(u64, u64, u32, u64)> {
    let devices = plain_devices_on(image);
    let configuration = choose_system_configuration(&devices).expect("择得到系统配置");
    let immutable = &configuration.immutable;
    every_root_ring_slot(&immutable.sizes)
        .into_iter()
        .filter_map(|ring_slot| {
            match read_root_ring_slot(
                &devices,
                &immutable.region_devices,
                &immutable.sizes,
                &immutable.filesystem_identifier,
                ring_slot,
            ) {
                RootRingSlotReading::SelfVerified(root) => {
                    Some((ring_slot.region, ring_slot.slot, root.instance.0, root.checkpoint_txg.0))
                }
                RootRingSlotReading::Bad(_) => None,
            }
        })
        .collect()
}

fn outcomes(
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    image: &MemoryPool,
) -> String {
    let devices = plain_devices_on(image);
    let recovery = match recover(&devices, JournalPolicy::Consult).outcome {
        RecoveryOutcome::FileRead { root, .. } => format!("Ok({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::NoFile { root } => format!("NoFile({},{})", root.0 .0, root.1 .0),
        RecoveryOutcome::Failed { .. } => "Fail".to_string(),
    };
    let read_only = match mount_read_only(&devices) {
        Ok(mounted) => format!(
            "Ok({},{})",
            mounted.effective_root.instance.0, mounted.effective_root.checkpoint_txg.0
        ),
        Err(_) => "Fail".to_string(),
    };
    let mut writable_devices = plain_devices_on(image);
    let writable = match mount_writable(parameters, &mut writable_devices) {
        Ok(mounted) => format!(
            "Ok({},{})",
            mounted.output.effective_root.instance.0, mounted.output.effective_root.checkpoint_txg.0
        ),
        Err(_) => "Fail".to_string(),
    };
    format!("rec={recovery} ro={read_only} rw={writable}")
}

#[derive(Default)]
struct Tally {
    images: u32,
    roots: u32,
    rejected_by_bare: u32,
    rejected_by_exception: u32,
    images_with_a_difference_under_exception: u32,
    images_with_a_difference_under_bare: u32,
}

fn evaluate(
    tally: &mut Tally,
    label: &str,
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    image: &MemoryPool,
) {
    set_formula(0);
    let today_slots = self_verified_slots(image);
    let today = outcomes(parameters, image);
    set_formula(1);
    let bare_slots = self_verified_slots(image);
    let bare = outcomes(parameters, image);
    set_formula(2);
    let exception_slots = self_verified_slots(image);
    let exception = outcomes(parameters, image);
    set_formula(0);
    let rejected_bare = today_slots.len() - bare_slots.len();
    let rejected_exception = today_slots.len() - exception_slots.len();
    tally.images += 1;
    tally.roots += u32::try_from(today_slots.len()).expect("槽数");
    tally.rejected_by_bare += u32::try_from(rejected_bare).expect("槽数");
    tally.rejected_by_exception += u32::try_from(rejected_exception).expect("槽数");
    tally.images_with_a_difference_under_exception += u32::from(today != exception);
    tally.images_with_a_difference_under_bare += u32::from(today != bare);
    let rejected_roots: Vec<_> = today_slots
        .iter()
        .filter(|slot| !exception_slots.contains(slot))
        .collect();
    println!(
        "Y1 {label} roots={} rejected_bare={rejected_bare} rejected_with_exception={rejected_exception} today[{today}] bare[{}] exception[{}] rejected_by_exception={rejected_roots:?}",
        today_slots.len(),
        if bare == today { "same".to_string() } else { bare.clone() },
        if exception == today { "same".to_string() } else { exception.clone() },
    );
}

fn crash_prefix_sweep(
    tally: &mut Tally,
    label: &str,
    parameters: &singlefs_core::make_filesystem::MakeFilesystemParameters,
    base: &MemoryPool,
    writes: &[RetainedWrite],
) {
    for prefix in 0..=writes.len() {
        let mut image = base.clone();
        image.apply_writes(&writes[..prefix]);
        evaluate(tally, &format!("{label} prefix={prefix}/{}", writes.len()), parameters, &image);
    }
}

#[test]
fn legal_histories_never_trip_the_formula_check_with_the_genesis_exception() {
    let mut tally = Tally::default();
    let parameters = parameters_of_the_384_slot_small_devices();
    let device_bytes = device_bytes_of_the_384_slot_small_devices();

    // 历史一：mkfs 之后的第一次可写挂载，每个写前缀都断电一次；再挂两次。
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
    evaluate(&mut tally, "h1 after_mkfs", &parameters, &base);
    let operations_before = stream.operation_count();
    mount_writable(&parameters, &mut devices).expect("第一次可写挂载");
    let operations = stream.retained_operations();
    let (writes, _segments, _indexes) = writes_and_segments_with_stream_indexes(
        &operations[operations_before..],
        &fixed_geometry_of_the_384_slot_small_devices(),
    );
    crash_prefix_sweep(&mut tally, "h1 first_mount", &parameters, &base, &writes);
    let first_root = writes
        .iter()
        .position(|write| write.kind == StepKind::RootRecordFua)
        .expect("写行那次发布有根槽写");
    let mut crash_image = base.clone();
    crash_image.apply_writes(&writes[..first_root]);
    let mut second = plain_devices_on(&crash_image);
    mount_writable(&parameters, &mut second).expect("断电之后再可写挂载");
    let after_second = MemoryPool {
        devices: second.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: device_bytes,
    };
    evaluate(&mut tally, "h1 after_second_mount", &parameters, &after_second);
    let mut third = plain_devices_on(&after_second);
    mount_writable(&parameters, &mut third).expect("第三次可写挂载");
    let after_third = MemoryPool {
        devices: third.iter().map(|(identity, device)| (*identity, device.image.clone())).collect(),
        device_size_in_bytes: device_bytes,
    };
    evaluate(&mut tally, "h1 after_third_mount", &parameters, &after_third);

    // 历史二：一个实例里覆盖写 k 次（k = 3、30、60：一圈 24 个 txg，30、60 转过一圈、两圈多），再录一次发布 R，每个写前缀断电一次。
    for overwrites in [3u64, 30, 60] {
        let recorded = record_one_publish_after_overwrites(overwrites, PublishRecordedAsR::Overwrite);
        crash_prefix_sweep(
            &mut tally,
            &format!("h2 k={overwrites}"),
            &parameters,
            &recorded.base,
            &recorded.writes,
        );
    }

    // 历史三：覆盖写与断电再挂载交替（每次挂载取新实例、写行、暖机），转过三圈多。
    let mut pool = PoolUnderTest::start_after_the_first_file_on_devices_of(
        parameters_of_the_384_slot_small_devices(),
        device_bytes,
        plain_device,
    );
    for round in 0..14 {
        for _ in 0..3 {
            pool.overwrite(3000).expect("覆盖写");
        }
        evaluate(&mut tally, &format!("h3 round={round} before_crash"), &parameters, &pool.image());
        pool.crash_and_mount_writable().expect("断电后可写挂载");
        evaluate(&mut tally, &format!("h3 round={round} after_mount"), &parameters, &pool.image());
    }

    println!(
        "Y1_SUMMARY images={} self_verified_roots_seen={} rejected_by_bare_formula={} rejected_with_genesis_exception={} images_whose_outcome_differs_under_bare={} images_whose_outcome_differs_under_exception={}",
        tally.images,
        tally.roots,
        tally.rejected_by_bare,
        tally.rejected_by_exception,
        tally.images_with_a_difference_under_bare,
        tally.images_with_a_difference_under_exception
    );

    // 对照：手写错位。往 h3 最后一张镜像里，把 (区域 0, 槽 3) 的字节抄到 (区域 1, 槽 3)——一条非第 0 代根住到别的区域。
    let image = pool.image();
    let configuration = choose_system_configuration(&image).expect("择得到系统配置");
    let sizes = &configuration.immutable.sizes;
    let device_of = |region: u64| {
        configuration.immutable.region_devices[usize::try_from(region).expect("区域号")]
    };
    let width = usize::try_from(sizes.physical_block_size).expect("根槽宽");
    let read_slot = |image: &MemoryPool, region: u64, slot: u64| {
        image.devices[&device_of(region)].read(
            slot_offset(
                singlefs_core::root_ring::RootRingSlot { region, slot },
                sizes.fixed_structure_slot_spacing,
            ),
            width,
        )
    };
    let write_slot = |image: &mut MemoryPool, region: u64, slot: u64, bytes: &[u8]| {
        let offset = slot_offset(
            singlefs_core::root_ring::RootRingSlot { region, slot },
            sizes.fixed_structure_slot_spacing,
        );
        image
            .devices
            .get_mut(&device_of(region))
            .expect("盘在镜像里")
            .write(offset, bytes);
    };
    let mut misplaced = image.clone();
    let source = read_slot(&image, 0, 3);
    write_slot(&mut misplaced, 1, 3, &source);
    let mut control = Tally::default();
    evaluate(&mut control, "control misplaced_non_genesis_root_copied_to_(1,3)", &parameters, &misplaced);
    // 对照：第 0 代根的字节出现在 (区域 1, 槽 0)（mkfs 后的镜像里那一槽本来就住着 txg 0 的根；这里拿 h3 最后一张镜像，那一槽住的是 txg 更大的根）。
    let genesis_bytes = read_slot(&base, 0, 0);
    let mut genesis_misread = image.clone();
    write_slot(&mut genesis_misread, 1, 0, &genesis_bytes);
    evaluate(&mut control, "control genesis_root_bytes_copied_to_(1,0)", &parameters, &genesis_misread);
    // 对照：第 0 代根的字节出现在槽号不是 0 的槽 (区域 1, 槽 3)：例外只认槽 0，应当拒。
    let mut genesis_in_slot_three = image.clone();
    write_slot(&mut genesis_in_slot_three, 1, 3, &genesis_bytes);
    evaluate(&mut control, "control genesis_root_bytes_copied_to_(1,3)", &parameters, &genesis_in_slot_three);
}

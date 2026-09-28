//! E164（崩溃放量新设计的小范围功能测试）第二段的 harness 档探针：只录制、不造崩溃状态。
//! 装置把这份文本放进仓副本的 `crates/singlefs-harness/tests/e164_probe.rs`，在副本里 `cargo test -p singlefs-harness --test e164_probe` 起它；
//! 主仓里没有这份文件。每行 `E7RESULT probe=<哪条探针> name=<什么> …`，字段次序固定，装置逐行解析。
//! 装置自己独立算切段、状态数与写表哈希；这里打出的 harness 切段与闭式只拿来对拍（跑前登记第十一节 S-seg）。

use singlefs_core::address::{DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::DeviceFreeMap;
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{
    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, publish_sequential_write, warm_up,
    FirstFile, PoolWriter,
};
use singlefs_harness::history::with_panic_capture;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::scenario::{
    e142_parameters, first_file_content, run_new_pool_file_creation, ScenarioPoint,
    FIXED_WRITE_TIME_SECONDS,
};
use singlefs_harness::segments::{
    closed_form_state_count, segment_sizes_text, split_into_segments, FixedGeometry,
};
use singlefs_harness::sha256::sha256_hexadecimal;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, SharedStream};

/// 内存盘每块 4 GiB（与 harness 集成测试的镜像同大）。
const DEVICE_BYTES: u64 = 4 << 30;
const SECTOR_BYTES: usize = 512;
/// H-长接在新池新建文件之后的三次发布：内容长度（字节）与写入时间相对 E142 固定时间的偏移（秒）。
const FIRST_OVERWRITE_BYTES: usize = 2000;
const SEQUENTIAL_WRITE_BYTES: usize = 70000;
const SECOND_OVERWRITE_BYTES: usize = 1500;
const FIRST_OVERWRITE_SECONDS_AFTER: u64 = 60;
const SEQUENTIAL_WRITE_SECONDS_AFTER: u64 = 120;
const SECOND_OVERWRITE_SECONDS_AFTER: u64 = 180;
/// panic 探针建空闲图用的盘长：比单元区起点小。
const PANIC_PROBE_DEVICE_BYTES: u64 = 1 << 20;

type RecordedSparse = RecordingBlockDevice<SparseBlockDevice>;

fn first_overwrite_content() -> Vec<u8> {
    (0..FIRST_OVERWRITE_BYTES)
        .map(|index| u8::try_from((index * 7 + 3) % 253).expect("小于 256"))
        .collect()
}

fn sequential_write_content() -> Vec<u8> {
    (0..SEQUENTIAL_WRITE_BYTES)
        .map(|index| u8::try_from((index * 11 + 1) % 239).expect("小于 256"))
        .collect()
}

fn second_overwrite_content() -> Vec<u8> {
    (0..SECOND_OVERWRITE_BYTES)
        .map(|index| u8::try_from((index * 13 + 5) % 241).expect("小于 256"))
        .collect()
}

fn parameters() -> MakeFilesystemParameters {
    e142_parameters(512, 512)
}

fn geometry_of(parameters: &MakeFilesystemParameters) -> FixedGeometry {
    FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
        root_ring_slots_per_region: parameters.geometry.root_ring_slots_per_region,
    }
}

fn recorded_devices(stream: &SharedStream) -> Vec<(DeviceIdentity, RecordedSparse)> {
    (0..2u32)
        .map(|device_number| {
            (
                DeviceIdentity(device_number),
                RecordingBlockDevice::with_shared_stream(
                    DeviceIdentity(device_number),
                    SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                    stream.clone(),
                ),
            )
        })
        .collect()
}

/// 录制流按步切出的区间：（步名，起，止），起止是录制流下标。
fn print_history(
    probe: &str,
    stream: &SharedStream,
    step_bounds: &[(String, usize, usize)],
    mkfs_sector_tables: &[(u32, Vec<(u64, String)>)],
    geometry: &FixedGeometry,
) {
    let mut emitted = 0usize;
    for (label, start, end) in step_bounds {
        println!("E7RESULT probe={probe} name=step step={label} start={start} end={end}");
        emitted += 1;
    }
    let retained = stream.retained_operations();
    for (index, retained_operation) in retained.iter().enumerate() {
        let operation = &retained_operation.operation;
        let kind = match operation.kind {
            RecordedOperationKind::Write => "write",
            RecordedOperationKind::WriteForceUnitAccess => "write_fua",
            RecordedOperationKind::WriteZeroes => "write_zeroes",
            RecordedOperationKind::Barrier => "barrier",
        };
        let (content_digest, sector_digests) = match (&operation.kind, &retained_operation.contents) {
            (RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess, Some(bytes)) => {
                let sectors: Vec<String> = bytes
                    .chunks(SECTOR_BYTES)
                    .map(sha256_hexadecimal)
                    .collect();
                (sha256_hexadecimal(bytes), sectors.join(","))
            }
            (RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess, None) => {
                panic!("录制流开了内容保留，普通写没有字节：第 {index} 步")
            }
            (RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier, _) => {
                ("none".to_string(), "none".to_string())
            }
        };
        println!(
            "E7RESULT probe={probe} name=op index={index} device={} kind={kind} offset={} length={} fnv={:016x} sha256={content_digest} sectors={sector_digests}",
            operation.device.0, operation.offset.0, operation.length, operation.content_hash
        );
        emitted += 1;
    }
    for (device, table) in mkfs_sector_tables {
        for (sector, digest) in table {
            println!("E7RESULT probe={probe} name=mkfs_sector device={device} sector={sector} sha256={digest}");
            emitted += 1;
        }
    }
    let operations = stream.operations();
    for (label, start, end) in step_bounds {
        let segments = split_into_segments(&operations[*start..*end], geometry);
        let widest = segments
            .iter()
            .map(|segment| segment.iter().filter(|kind| kind.name() != "barrier").count())
            .max()
            .unwrap_or(0);
        let closed_form = if widest < 64 {
            closed_form_state_count(&segments).to_string()
        } else {
            "over64".to_string()
        };
        println!(
            "E7RESULT probe={probe} name=harness_segments step={label} sizes={} closed_form={closed_form}",
            if segments.is_empty() { "empty".to_string() } else { segment_sizes_text(&segments) }
        );
        emitted += 1;
    }
    let mkfs_end = step_bounds
        .iter()
        .find(|(label, _, _)| label == "base")
        .map(|(_, _, end)| *end)
        .expect("有 base 那一步");
    let after_mkfs = split_into_segments(&operations[mkfs_end..], geometry);
    println!(
        "E7RESULT probe={probe} name=harness_segments step=after_mkfs sizes={}",
        segment_sizes_text(&after_mkfs)
    );
    emitted += 1;
    println!(
        "E7RESULT probe={probe} name=probe_end operations={} emitted_before_this_line={emitted}",
        operations.len()
    );
}

fn sector_table_of(devices: &[(DeviceIdentity, RecordedSparse)]) -> Vec<(u32, Vec<(u64, String)>)> {
    devices
        .iter()
        .map(|(identity, device)| {
            let table = device
                .wrapped_device()
                .image
                .written_sectors()
                .iter()
                .map(|(sector, bytes)| (*sector, sha256_hexadecimal(bytes)))
                .collect();
            (identity.0, table)
        })
        .collect()
}

#[test]
fn records_h_short() {
    let parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices(&stream);
    let mut boundaries: Vec<usize> = Vec::new();
    let mut mkfs_sector_tables = Vec::new();
    run_new_pool_file_creation(&parameters, &mut devices, &stream, |point, devices_at_point| {
        boundaries.push(stream.operation_count());
        if matches!(point, ScenarioPoint::AfterMakeFilesystem) {
            mkfs_sector_tables = sector_table_of(devices_at_point);
        }
    })
    .expect("H-短整条路");
    let end = stream.operation_count();
    assert_eq!(boundaries.len(), 3, "scenario 的三处各叫一次");
    let step_bounds = vec![
        ("base".to_string(), 0, boundaries[0]),
        ("0".to_string(), boundaries[0], boundaries[1]),
        ("1".to_string(), boundaries[1], boundaries[2]),
        ("2".to_string(), boundaries[2], end),
    ];
    print_history(
        "h_short",
        &stream,
        &step_bounds,
        &mkfs_sector_tables,
        &geometry_of(&parameters),
    );
}

#[test]
fn records_h_long() {
    let parameters = parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices = recorded_devices(&stream);
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mkfs_end = stream.operation_count();
    let mkfs_sector_tables = sector_table_of(&devices);
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let first_content = first_file_content();
    let (instance, warm_up_output, acquisition_end) = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        let instance = acquire_instance(&mut pool).expect("取号");
        let acquisition_end = stream.operation_count();
        let warm = warm_up(&mut pool, &genesis.root, instance).expect("暖机");
        (instance, warm, acquisition_end)
    };
    assert_eq!(instance, InstanceGeneration(1));
    let warm_up_end = stream.operation_count();
    let created = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &first_content,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("新池新建文件")
    };
    let creation_end = stream.operation_count();
    let first_overwrite_bytes = first_overwrite_content();
    let first_overwrite = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &created,
            FirstFile {
                content: &first_overwrite_bytes,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + FIRST_OVERWRITE_SECONDS_AFTER,
            },
            instance,
        )
        .expect("第一次覆盖写")
    };
    let first_overwrite_end = stream.operation_count();
    let sequential_bytes = sequential_write_content();
    let sequential = {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        publish_sequential_write(
            &mut pool,
            &mut allocator,
            &first_overwrite,
            FirstFile {
                content: &sequential_bytes,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + SEQUENTIAL_WRITE_SECONDS_AFTER,
            },
            instance,
        )
        .expect("顺序写")
    };
    let sequential_end = stream.operation_count();
    let second_overwrite_bytes = second_overwrite_content();
    {
        let mut pool = PoolWriter::new(&parameters, &mut devices);
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &sequential,
            FirstFile {
                content: &second_overwrite_bytes,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + SECOND_OVERWRITE_SECONDS_AFTER,
            },
            instance,
        )
        .expect("第二次覆盖写");
    }
    let end = stream.operation_count();
    let step_bounds = vec![
        ("base".to_string(), 0, mkfs_end),
        ("0".to_string(), mkfs_end, acquisition_end),
        ("1".to_string(), acquisition_end, warm_up_end),
        ("2".to_string(), warm_up_end, creation_end),
        ("3".to_string(), creation_end, first_overwrite_end),
        ("4".to_string(), first_overwrite_end, sequential_end),
        ("5".to_string(), sequential_end, end),
    ];
    print_history(
        "h_long",
        &stream,
        &step_bounds,
        &mkfs_sector_tables,
        &geometry_of(&parameters),
    );
}

#[test]
fn panic_probe() {
    let captured = with_panic_capture(|| DeviceFreeMap::new(DeviceIdentity(0), PANIC_PROBE_DEVICE_BYTES));
    match captured {
        Ok(_) => println!("E7RESULT probe=panic name=no_panic"),
        Err(panic) => {
            let message_hex: String = panic
                .message
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            println!(
                "E7RESULT probe=panic name=panic location={} message_hex={message_hex}",
                panic.location
            );
        }
    }
}

#[test]
fn pid_probe() {
    println!("E7RESULT probe=pid name=pid pid={}", std::process::id());
}

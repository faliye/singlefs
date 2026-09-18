//! 宿主一侧：拿 QEMU blklogwrites 记下的两份设备侧日志，与宿主上重跑同一条写路（同参数同字节）得到的录制流逐盘逐项比。
//!
//!   first_transaction_device_log_check <log0.img> <log1.img> <设备字节数> <physical_block_size> <minimum_io> [<disk0.img> <disk1.img>]
//!
//! 三个数取虚机里那次跑的 `name=geometry` 行。判据：程序的事件是设备侧日志的逐项前缀，前缀之后至多一个 FLUSH
//! （虚机关机时补的那一个；2026-09-14 第一次跑 direct 时每块盘各多出这一个，见 records 十一·六）。
//! 给了两块数据盘就再在宿主上从盘上冷恢复一次。退出码：0 = 两块盘都对得上且（给了盘时）读回文件；
//! 1 = 有一块盘对不上（打印第一处）或读不回；2 = 用法或日志读不了。结果行同样以 `E7RESULT` 打头。

use singlefs_core::address::DeviceIdentity;
use singlefs_core::block_device::{
    DirectInputOutputBlockDevice, PageCachePolicy, PhysicalBlockSizeInBytes,
    PhysicalBlockSizeSource,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::crash::SparseBlockDevice;
use singlefs_harness::device_log::{
    compare_allowing_trailing_flushes, expected_device_events, parse_device_log, DeviceEvent,
};
use singlefs_harness::scenario::{e142_parameters, first_file_content, run_first_transaction};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

/// 前缀之后可以接受的 FLUSH 个数：虚机关机时每块盘至多补一个。
const ACCEPTED_TRAILING_FLUSHES: usize = 1;

fn parse_number(text: &str, what: &str) -> u64 {
    text.parse().unwrap_or_else(|_error| {
        eprintln!("{what} 不是数：{text}");
        std::process::exit(2)
    })
}

fn count_kinds(events: &[DeviceEvent]) -> (usize, usize) {
    let writes = events
        .iter()
        .filter(|event| matches!(event, DeviceEvent::Write { .. }))
        .count();
    let flushes = events
        .iter()
        .filter(|event| matches!(event, DeviceEvent::Flush))
        .count();
    (writes, flushes)
}

fn root_text(
    root: (
        singlefs_core::address::InstanceGeneration,
        singlefs_core::address::CheckpointTxg,
    ),
) -> String {
    format!("{}:{}", root.0 .0, root.1 .0)
}

/// 宿主从两块数据盘上冷恢复一次：读回文件且内容对才算读回。
fn recover_from_disk_images(paths: [&str; 2], physical_block_size: u32) -> (String, bool) {
    let disks: Vec<(DeviceIdentity, DirectInputOutputBlockDevice)> = paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let device = DirectInputOutputBlockDevice::open_existing(
                std::path::Path::new(path),
                PageCachePolicy::GoThroughPageCache,
                PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(physical_block_size)),
            )
            .unwrap_or_else(|error| {
                eprintln!("打不开 {path}：{error}");
                std::process::exit(2)
            });
            (
                DeviceIdentity(u32::try_from(index).expect("设备号")),
                device,
            )
        })
        .collect();
    let report = recover(&disks, JournalPolicy::Consult);
    let (outcome, root, recovered) = match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => (
            "file_read",
            root_text(*root),
            *content == first_file_content(),
        ),
        RecoveryOutcome::NoFile { root } => ("no_file", root_text(*root), false),
        RecoveryOutcome::Failed { root, .. } => (
            "failed",
            root.map_or_else(|| "none".to_string(), root_text),
            false,
        ),
    };
    (
        format!("name=host_recover outcome={outcome} root={root} content_matches={recovered} valid_records={}", report.journal.valid_records),
        recovered,
    )
}

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 6 && arguments.len() != 8 {
        eprintln!("用法：first_transaction_device_log_check <log0.img> <log1.img> <设备字节数> <physical_block_size> <minimum_io> [<disk0.img> <disk1.img>]");
        std::process::exit(2);
    }
    let device_bytes = parse_number(&arguments[3], "设备字节数");
    let physical_block_size = u32::try_from(parse_number(&arguments[4], "physical_block_size"))
        .expect("物理块大小装得进 u32");
    let minimum_input_output_bytes =
        u32::try_from(parse_number(&arguments[5], "minimum_io")).expect("io_min 装得进 u32");
    let parameters = e142_parameters(physical_block_size, minimum_input_output_bytes);
    let stream = SharedStream::new();
    let mut devices: Vec<(DeviceIdentity, RecordingBlockDevice<SparseBlockDevice>)> = (0..2u32)
        .map(|index| {
            let identity = DeviceIdentity(index);
            let device =
                SparseBlockDevice::new(device_bytes, PhysicalBlockSizeInBytes(physical_block_size));
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    run_first_transaction(&parameters, &mut devices, &stream, |_point, _devices| {})
        .unwrap_or_else(|reason| {
            eprintln!("宿主重跑写路失败：{reason}");
            std::process::exit(2)
        });
    let operations = stream.retained_operations();
    let mut every_device_matches = true;
    let mut emitted = 0u64;
    for index in 0..2usize {
        let log_bytes = std::fs::read(&arguments[1 + index]).unwrap_or_else(|error| {
            eprintln!("读不了 {}：{error}", arguments[1 + index]);
            std::process::exit(2)
        });
        let log = parse_device_log(&log_bytes).unwrap_or_else(|error| {
            eprintln!("{} 解析不了：{error:?}", arguments[1 + index]);
            std::process::exit(2)
        });
        let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
        let expected = expected_device_events(&operations, identity);
        let (expected_writes, expected_flushes) = count_kinds(&expected);
        let (observed_writes, observed_flushes) = count_kinds(&log.events);
        let comparison = compare_allowing_trailing_flushes(&expected, &log.events);
        let divergence = if comparison.divergence.is_none()
            && comparison.trailing_flushes > ACCEPTED_TRAILING_FLUSHES
        {
            Some((
                expected.len() + ACCEPTED_TRAILING_FLUSHES,
                None,
                Some(DeviceEvent::Flush),
            ))
        } else {
            comparison.divergence.clone()
        };
        let divergence_text = match &divergence {
            None => "none".to_string(),
            Some((position, program, device)) => {
                format!("at={position} program={program:?} device={device:?}").replace(' ', "")
            }
        };
        every_device_matches &= divergence.is_none();
        println!(
            "E7RESULT name=device_log device={} declared_entries={} expected_writes={expected_writes} expected_flushes={expected_flushes} observed_writes={observed_writes} observed_flushes={observed_flushes} trailing_flushes={} divergence={divergence_text}",
            identity.0,
            log.declared_entries.map_or_else(|| "NA".to_string(), |declared| declared.to_string()),
            comparison.trailing_flushes,
        );
        emitted += 1;
    }
    println!("E7RESULT name=device_log_verdict matches={every_device_matches}");
    emitted += 1;
    let mut recovered = true;
    if arguments.len() == 8 {
        let (line, read_back) =
            recover_from_disk_images([&arguments[6], &arguments[7]], physical_block_size);
        println!("E7RESULT {line}");
        emitted += 1;
        recovered = read_back;
    }
    println!("E7RESULT name=done emitted={}", emitted + 1);
    if !(every_device_matches && recovered) {
        std::process::exit(1);
    }
}

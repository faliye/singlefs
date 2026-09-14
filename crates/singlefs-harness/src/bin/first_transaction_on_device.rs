//! 虚机档（QEMU/KVM）：在两块真 virtio 盘上跑第一个事务的整条写路，冷重开再恢复读回文件。
//!
//!   first_transaction_on_device /dev/vda /dev/vdb <direct | page-cache | skip-first-transaction-barrier>
//!
//! 由 `research/scripts/vm-bench.sh` 送进虚机（`VM_DISKS=2`，设备路径排在参数前面）。结果行以 `E7RESULT` 打头、
//! 末行报条数（vm-bench.sh 的完整性闸）。这个二进制只判它自己判得了的（恢复读回文件、段序列）；
//! 「盘上实际收到的是不是程序以为的」由宿主上的 `first_transaction_device_log_check` 拿 QEMU 的设备侧日志判。
//! `skip-first-transaction-barrier`：第一个事务在盘 0 上漏掉「单元 → journal 记录」那道屏障，而录制器照样记下它——
//! 程序以为发了，盘上没收到，宿主那一侧必须判红（C6（块层语义假设写错） 的「故意去掉一次屏障」）。

use std::path::Path;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{
    probe_queue_number_under, probe_queue_text_under, BlockDevice, BlockDeviceError,
    DirectInputOutputBlockDevice, PageCachePolicy, PhysicalBlockSizeInBytes,
    PhysicalBlockSizeSource, WriteDurability, SYSFS_CLASS_BLOCK,
};
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_harness::scenario::{e142_parameters, first_file_content, run_first_transaction};
use singlefs_harness::segments::{
    closed_form_state_count, segment_kinds_text, segment_sizes_text, split_into_segments,
    FixedGeometry,
};
use singlefs_harness::{RecordedOperation, RecordingBlockDevice, SharedStream};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunMode {
    Direct,
    PageCache,
    SkipFirstTransactionBarrier,
}

/// 包在真设备外面、录制器里面：数程序真正交给设备的调用，按需吞掉一道屏障。
struct FaultInjectingDevice<Inner: BlockDevice> {
    inner: Inner,
    skip_next_barrier: bool,
    skipped_barriers: u64,
    barrier_calls: u64,
    write_calls: u64,
    written_bytes: u64,
    force_unit_access_writes: u64,
}

impl<Inner: BlockDevice> BlockDevice for FaultInjectingDevice<Inner> {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.write_calls += 1;
        self.written_bytes += u64::try_from(bytes.len()).expect("长度");
        if durability == WriteDurability::ForceUnitAccess {
            self.force_unit_access_writes += 1;
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        if self.skip_next_barrier {
            self.skip_next_barrier = false;
            self.skipped_barriers += 1;
            return Ok(());
        }
        self.barrier_calls += 1;
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

struct Emitter {
    emitted: u64,
}

impl Emitter {
    fn emit(&mut self, body: &str) {
        self.emitted += 1;
        println!("E7RESULT {body}");
    }
    fn finish(&mut self) {
        self.emitted += 1;
        println!("E7RESULT name=done emitted={}", self.emitted);
    }
}

/// 块层计数（`/sys/class/block/<名>/stat`，内核 `Documentation/block/stat.rst` 的字段序）：写请求数（下标 4）、写扇区数（下标 6）、
/// FLUSH 请求数（下标 15，5.5 起才有，老内核没有就是 None）。
fn block_layer_counters(device_path: &str) -> Option<(u64, u64, Option<u64>)> {
    let name = Path::new(device_path).file_name()?.to_str()?;
    let text =
        std::fs::read_to_string(Path::new(SYSFS_CLASS_BLOCK).join(name).join("stat")).ok()?;
    let fields: Vec<u64> = text
        .split_whitespace()
        .filter_map(|field| field.parse().ok())
        .collect();
    if fields.len() < 8 {
        return None;
    }
    Some((fields[4], fields[6], fields.get(15).copied()))
}

fn describe_counters(
    before: Option<(u64, u64, Option<u64>)>,
    after: Option<(u64, u64, Option<u64>)>,
) -> String {
    match (before, after) {
        (
            Some((write_ios_before, sectors_before, flush_before)),
            Some((write_ios_after, sectors_after, flush_after)),
        ) => {
            let flushes = match (flush_before, flush_after) {
                (Some(start), Some(end)) => (end - start).to_string(),
                (_, _) => "NA".to_string(),
            };
            format!(
                "write_ios={} write_sectors={} flush_ios={flushes}",
                write_ios_after - write_ios_before,
                sectors_after - sectors_before
            )
        }
        (_, _) => "write_ios=NA write_sectors=NA flush_ios=NA".to_string(),
    }
}

fn open(path: &str, policy: PageCachePolicy) -> DirectInputOutputBlockDevice {
    DirectInputOutputBlockDevice::open_existing(
        Path::new(path),
        policy,
        PhysicalBlockSizeSource::ProbeSysfs,
    )
    .unwrap_or_else(|error| {
        eprintln!("打不开 {path}：{error}");
        std::process::exit(3)
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "虚机里只跑这一件事：写、数、重开、读回，结果行按次序打"
)]
fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 4 {
        eprintln!("用法：first_transaction_on_device <盘 0> <盘 1> <direct | page-cache | skip-first-transaction-barrier>");
        std::process::exit(2);
    }
    let device_paths = [arguments[1].clone(), arguments[2].clone()];
    let mode = match arguments[3].as_str() {
        "direct" => RunMode::Direct,
        "page-cache" => RunMode::PageCache,
        "skip-first-transaction-barrier" => RunMode::SkipFirstTransactionBarrier,
        other => {
            eprintln!("不认识的模式 {other}");
            std::process::exit(2)
        }
    };
    let policy = match mode {
        RunMode::Direct | RunMode::SkipFirstTransactionBarrier => {
            PageCachePolicy::BypassWithDirectInputOutput
        }
        RunMode::PageCache => PageCachePolicy::GoThroughPageCache,
    };
    let mut emitter = Emitter { emitted: 0 };
    let probe = |attribute: &str| -> u32 {
        let values: Vec<Option<u32>> = device_paths
            .iter()
            .map(|path| {
                probe_queue_number_under(Path::new(SYSFS_CLASS_BLOCK), Path::new(path), attribute)
            })
            .collect();
        match (values[0], values[1]) {
            (Some(first), Some(second)) if first == second => first,
            (first, second) => {
                eprintln!("两块盘的 queue/{attribute} 读不到或不相等：{first:?} / {second:?}");
                std::process::exit(4)
            }
        }
    };
    let physical_block_size = probe("physical_block_size");
    let minimum_input_output_bytes = probe("minimum_io_size");
    let write_cache = probe_queue_text_under(
        Path::new(SYSFS_CLASS_BLOCK),
        Path::new(&device_paths[0]),
        "write_cache",
    )
    .unwrap_or_else(|| "NA".to_string());
    let parameters = e142_parameters(physical_block_size, minimum_input_output_bytes);
    let stream = SharedStream::new();
    let mut devices: Vec<(
        DeviceIdentity,
        RecordingBlockDevice<FaultInjectingDevice<DirectInputOutputBlockDevice>>,
    )> = device_paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let identity = DeviceIdentity(u32::try_from(index).expect("设备号"));
            let device = FaultInjectingDevice {
                inner: open(path, policy),
                skip_next_barrier: false,
                skipped_barriers: 0,
                barrier_calls: 0,
                write_calls: 0,
                written_bytes: 0,
                force_unit_access_writes: 0,
            };
            (
                identity,
                RecordingBlockDevice::with_shared_stream(identity, device, stream.clone()),
            )
        })
        .collect();
    let device_bytes = devices[0].1.size_in_bytes();
    emitter.emit(&format!(
        "name=geometry mode={} physical_block_size={physical_block_size} minimum_io={minimum_input_output_bytes} spacing={} write_cache={} device_bytes={device_bytes}",
        arguments[3],
        parameters.geometry.fixed_structure_slot_spacing,
        write_cache.replace(' ', "_")
    ));
    let counters_before: Vec<_> = device_paths
        .iter()
        .map(|path| block_layer_counters(path))
        .collect();
    let run = run_first_transaction(&parameters, &mut devices, &stream, |devices| {
        if mode == RunMode::SkipFirstTransactionBarrier {
            devices[0].1.inner_mut().skip_next_barrier = true;
        }
    })
    .unwrap_or_else(|reason| {
        eprintln!("写路径失败：{reason}");
        std::process::exit(5)
    });
    let counters_after: Vec<_> = device_paths
        .iter()
        .map(|path| block_layer_counters(path))
        .collect();

    let operations = stream.operations();
    let geometry = FixedGeometry {
        fixed_structure_slot_spacing: parameters.geometry.fixed_structure_slot_spacing,
        journal_ring_bytes: parameters.geometry.journal_ring_bytes,
    };
    let warm_up_end = operations.len() - 23;
    let paths: [(&str, &[RecordedOperation]); 5] = [
        ("mkfs", &operations[..run.mkfs_operation_count]),
        (
            "instance_acquisition",
            &operations[run.mkfs_operation_count..run.mkfs_operation_count + 2],
        ),
        (
            "warm_up",
            &operations[run.mkfs_operation_count + 2..warm_up_end],
        ),
        ("transaction", &operations[warm_up_end..]),
        ("post_mkfs_stream", &operations[run.mkfs_operation_count..]),
    ];
    for (name, slice) in paths {
        let segments = split_into_segments(slice, &geometry);
        emitter.emit(&format!(
            "name=segments path={name} operations={} segments={} closed_form={} kinds={}",
            slice.len(),
            segment_sizes_text(&segments),
            closed_form_state_count(&segments),
            segment_kinds_text(&segments)
        ));
    }
    for (index, (identity, device)) in devices.iter().enumerate() {
        let counted = device.inner();
        emitter.emit(&format!(
            "name=device_calls device={} path={} writes={} written_bytes={} force_unit_access_writes={} barriers={} skipped_barriers={} {}",
            identity.0,
            device_paths[index],
            counted.write_calls,
            counted.written_bytes,
            counted.force_unit_access_writes,
            counted.barrier_calls,
            counted.skipped_barriers,
            describe_counters(counters_before[index], counters_after[index])
        ));
    }
    emitter.emit(&format!(
        "name=transaction policy_mismatches={} key_order_mismatches={} root_txg={} back_chain={}",
        run.policy_mismatches,
        run.output.key_order_mismatches,
        run.output.root.checkpoint_txg.0,
        run.output.record.back_chain
    ));

    // 冷重开：丢掉写的那一套句柄，按路径重新打开，恢复只通过块设备读盘。
    drop(devices);
    let reopened: Vec<(DeviceIdentity, DirectInputOutputBlockDevice)> = device_paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            (
                DeviceIdentity(u32::try_from(index).expect("设备号")),
                open(path, policy),
            )
        })
        .collect();
    let report = recover(&reopened, JournalPolicy::Consult);
    let (outcome, root, content_matches) = match &report.outcome {
        RecoveryOutcome::FileRead { root, content } => (
            "file_read",
            format!("{}:{}", root.0 .0, root.1 .0),
            *content == first_file_content(),
        ),
        RecoveryOutcome::NoFile { root } => {
            ("no_file", format!("{}:{}", root.0 .0, root.1 .0), false)
        }
        RecoveryOutcome::Failed { root, failure } => {
            eprintln!("恢复失败：{failure:?}");
            (
                "failed",
                root.map_or_else(
                    || "none".to_string(),
                    |root| format!("{}:{}", root.0 .0, root.1 .0),
                ),
                false,
            )
        }
    };
    emitter.emit(&format!(
        "name=recover_cold outcome={outcome} root={root} content_matches={content_matches} valid_records={} above_water={} applied={} verification_passed={} mapping_fallbacks={}",
        report.journal.valid_records, report.journal.above_water, report.journal.prefix_applied, report.journal.verification_passed, report.mapping_fallbacks
    ));
    emitter.finish();
    if !(outcome == "file_read" && content_matches) {
        std::process::exit(1);
    }
}

//! e171-conclusion-r3 攻方腿的探针（只在腿的副本上编、跑；不进 crates）。由 e171-conclusion-r2 攻方的 opus_r2_drive_probe.rs 改出：
//! 驱动与写流记录逐行照抄；多记两样——每次调用成功时交回值的 Debug 文本的 sha256、调用之后分配器的 Debug 文本的 sha256，
//! 用来查 E171 判「未触达」的行里，有没有写流、结局都与基线相同而成功调用交回的值不同的（E171 装置不记交回值，登记读法 4 的缺口）。
mod common;

use std::io::Write;

use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
use singlefs_core::allocator::{DeviceFreeMap, Placement, PoolAllocator};
use singlefs_core::block_device::PhysicalBlockSizeInBytes;
use singlefs_core::make_filesystem::{make_filesystem, INSTANCE_TABLE_SLOT, TREE_TABLE_GENESIS_SLOT};
use singlefs_core::mount::{
    mount_writable, raise_rollback_floor, roll_back_by_a_forward_publish, unmount, RollbackTarget, ShadowLedger,
};
use singlefs_core::transaction::{
    acquire_instance, publish_first_file, publish_overwrite, warm_up, FirstFile, PoolVersion, PoolWriter,
    TransactionOutput,
};
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
use singlefs_harness::sha256::sha256_hexadecimal;
use singlefs_harness::{RecordedOperationKind, RecordedPublishEntry, RecordingBlockDevice, SharedStream};

type Dev = RecordingBlockDevice<SparseBlockDevice>;
const WRITE_TIME: u64 = 1_788_000_000;

fn first_content() -> Vec<u8> {
    (0..3000usize).map(|i| u8::try_from(i % 251).unwrap()).collect()
}
fn second_content() -> Vec<u8> {
    (0..4100usize).map(|i| u8::try_from((i * 7 + 3) % 253).unwrap()).collect()
}
fn third_content() -> Vec<u8> {
    (0..2500usize).map(|i| u8::try_from((i * 7 + 11) % 253).unwrap()).collect()
}
fn later_content(seed: usize) -> Vec<u8> {
    (0..3000 + seed).map(|i| u8::try_from((i * 5 + seed) % 251).unwrap()).collect()
}

struct Run {
    stream: SharedStream,
    mkfs_ops: usize,
    out: std::fs::File,
    label: String,
    call: usize,
}

impl Run {
    fn o2(&mut self) -> String {
        let mut pool = MemoryPool::with_devices(&[DeviceIdentity(0), DeviceIdentity(1)], common::IMAGE_BYTES);
        pool.apply(&self.stream.retained_operations());
        let red: Vec<String> = check_pool_image(&pool)
            .into_iter()
            .filter_map(|(name, v)| matches!(v, InvariantVerdict::Violated(_)).then(|| name.to_string()))
            .collect();
        if red.is_empty() { "-".into() } else { red.join(",") }
    }
    fn call<T: std::fmt::Debug>(&mut self, name: &str, f: impl FnOnce() -> Result<T, String>) -> Option<T> {
        let first = self.stream.operation_count() - self.mkfs_ops;
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        let last = self.stream.operation_count() - self.mkfs_ops;
        let (word, value) = match r {
            Ok(Ok(v)) => ("ok".to_string(), Some(v)),
            Ok(Err(e)) => (format!("err:{}", e.chars().take(120).collect::<String>().replace(['\t', '\n', ' '], "_")), None),
            Err(p) => {
                let m = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                (format!("panic:{}", m.replace(['\t', '\n', ' '], "_")), None)
            }
        };
        let o2 = if value.is_some() { self.o2() } else { "not_judged".into() };
        let returned = value.as_ref().map_or("-".to_string(), |v| sha256_hexadecimal(format!("{v:?}").as_bytes()));
        if let (Ok(dir), Some(v)) = (std::env::var("OPUS_DRIVE_DEBUG_DIR"), value.as_ref()) {
            std::fs::write(format!("{dir}/{}-call{}-{name}.txt", self.label, self.call), format!("{v:#?}")).unwrap();
        }
        writeln!(self.out, "{}\tcall\t{}\t{}\t{}\t{}\t{}\to2_full_persisted={}\treturned={}", self.label, self.call, name, first, last, word, o2, returned).unwrap();
        self.call += 1;
        value
    }
}

fn fresh(identity: u32, stream: &SharedStream) -> (DeviceIdentity, Dev) {
    let d = DeviceIdentity(identity);
    (d, RecordingBlockDevice::with_shared_stream(d, SparseBlockDevice::new(common::IMAGE_BYTES, PhysicalBlockSizeInBytes(512)), stream.clone()))
}

fn reopen(devices: Vec<(DeviceIdentity, Dev)>, stream: &SharedStream) -> Vec<(DeviceIdentity, Dev)> {
    devices
        .into_iter()
        .map(|(d, rec)| {
            let (inner, _) = rec.into_inner_and_operations();
            (d, RecordingBlockDevice::with_shared_stream(d, inner, stream.clone()))
        })
        .collect()
}

#[test]
fn opus_r3_returned_values_of_the_fixed_script_through_e_then_unmount() {
    let Ok(out_path) = std::env::var("OPUS_DRIVE_OUT") else {
        eprintln!("OPUS_DRIVE_OUT 没给，什么都不做");
        return;
    };
    let label = std::env::var("OPUS_DRIVE_LABEL").unwrap_or_else(|_| "run".into());
    let parameters = common::parameters();
    let stream = SharedStream::retaining_contents();
    let mut devices: Vec<(DeviceIdentity, Dev)> = vec![fresh(0, &stream), fresh(1, &stream)];
    let genesis = make_filesystem(&parameters, &mut devices).expect("mkfs");
    let mkfs_ops = stream.operation_count();
    let out = std::fs::OpenOptions::new().create(true).append(true).open(&out_path).unwrap();
    let mut run = Run { stream: stream.clone(), mkfs_ops, out, label: label.clone(), call: 0 };
    let mut allocator = PoolAllocator::new(vec![
        DeviceFreeMap::new(DeviceIdentity(0), common::IMAGE_BYTES),
        DeviceFreeMap::new(DeviceIdentity(1), common::IMAGE_BYTES),
    ]);
    allocator.mark_format_time_units(Placement { slot: INSTANCE_TABLE_SLOT, span: 2 }, Placement { slot: TREE_TABLE_GENESIS_SLOT, span: 1 });
    let mut output: Option<TransactionOutput> = None;
    'script: {
        let instance = {
            let mut pool = PoolWriter::new(&parameters, &mut devices);
            let Some(instance) = run.call("acquire", || acquire_instance(&mut pool).map_err(|e| format!("{e:?}"))) else { break 'script };
            let Some(warmed) = run.call("warm_up", || warm_up(&mut pool, &genesis.root, instance).map_err(|e| format!("{e:?}"))) else { break 'script };
            let content = first_content();
            let Some(o) = run.call("publish", || {
                publish_first_file(&mut pool, &mut allocator, warmed.roots.last().unwrap(), FirstFile { content: &content, write_time_seconds: WRITE_TIME }, instance, &warmed.last_record_bytes)
                    .map_err(|e| format!("{e:?}"))
            }) else { break 'script };
            output = Some(o);
            instance
        };
        let overwrite = |run: &mut Run, devices: &mut Vec<(DeviceIdentity, Dev)>, allocator: &mut PoolAllocator, output: &mut Option<TransactionOutput>, content: Vec<u8>, inst: InstanceGeneration| -> bool {
            let previous = output.clone().unwrap();
            let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
            match run.call("publish", || {
                publish_overwrite(&mut writer, allocator, &previous, FirstFile { content: &content, write_time_seconds: WRITE_TIME + 60 }, inst).map_err(|e| format!("{e:?}"))
            }) {
                Some(o) => { *output = Some(o); true }
                None => false,
            }
        };
        if !overwrite(&mut run, &mut devices, &mut allocator, &mut output, second_content(), instance) { break 'script }
        // 进程重开之后可写挂载
        devices = reopen(devices, &stream);
        let Some(mounted) = run.call("mount_writable", || mount_writable(&parameters, &mut devices).map_err(|e| format!("{e:?}"))) else { break 'script };
        allocator = mounted.allocator;
        output = mounted.current.into_file_version();
        if output.is_none() { break 'script }
        if !overwrite(&mut run, &mut devices, &mut allocator, &mut output, third_content(), InstanceGeneration(2)) { break 'script }
        {
            let target = RollbackTarget { instance: InstanceGeneration(1), checkpoint_txg: CheckpointTxg(3) };
            let mut o = output.clone().unwrap();
            let ok = run.call("roll_back", || roll_back_by_a_forward_publish(&parameters, &mut devices, &mut allocator, &mut o, target).map_err(|e| format!("{e:?}")));
            if ok.is_none() { break 'script }
            output = Some(o);
        }
        for seed in [13usize, 17, 19, 23, 29] {
            if !overwrite(&mut run, &mut devices, &mut allocator, &mut output, later_content(seed), InstanceGeneration(2)) { break 'script }
        }
        {
            let mut current = output.clone().unwrap();
            let ok = run.call("raise_floor", || raise_rollback_floor(&parameters, &mut devices, &mut allocator, &mut current, CheckpointTxg(11), ShadowLedger::On).map_err(|e| format!("{e:?}")));
            output = Some(current);
            if ok.is_none() { break 'script }
        }
        if !overwrite(&mut run, &mut devices, &mut allocator, &mut output, later_content(31), InstanceGeneration(2)) { break 'script }
        {
            let mut current = PoolVersion::WithFile(output.clone().unwrap());
            let s = stream.clone();
            let _ = run.call("unmount", || s.record_entry(RecordedPublishEntry::Unmount, || unmount(&parameters, &mut devices, &mut allocator, &mut current, ShadowLedger::On)).map(|u| match u {
                singlefs_core::mount::Unmounted::FloorRaisedToTheCurrentVersion(r) => format!("raised floor={:?} sc={:?} publishes={:?} reclaimed={:?} unreadable={}",
                    r.rollback_floor, r.system_configuration_writes_before_the_first_publish, r.publishes, r.reclaimed, r.abandoned_roots_unreadable),
                singlefs_core::mount::Unmounted::NothingWrittenOnAVersionWithoutFile { current } => format!("nothing current={current:?}"),
            }).map_err(|e| format!("{e:?}")));
        }
    }
    let mut out = std::fs::OpenOptions::new().append(true).open(&out_path).unwrap();
    if let Ok(dir) = std::env::var("OPUS_DRIVE_DEBUG_DIR") {
        std::fs::write(format!("{dir}/{label}-final-allocator.txt"), format!("{allocator:#?}")).unwrap();
        std::fs::write(format!("{dir}/{label}-final-output.txt"), format!("{output:#?}")).unwrap();
    }
    writeln!(out, "{label}\tfinal\tallocator={}\toutput={}", sha256_hexadecimal(format!("{allocator:?}").as_bytes()),
        sha256_hexadecimal(format!("{output:?}").as_bytes())).unwrap();
    if let Ok(payload_path) = std::env::var("OPUS_DRIVE_PAYLOAD") {
        let mut bytes = Vec::new();
        for op in stream.retained_operations().iter().skip(mkfs_ops) {
            if let Some(c) = &op.contents { bytes.extend_from_slice(c); }
        }
        std::fs::write(payload_path, bytes).unwrap();
    }
    for (index, op) in stream.retained_operations().iter().enumerate().skip(mkfs_ops) {
        let kind = match op.operation.kind {
            RecordedOperationKind::Write => "write",
            RecordedOperationKind::WriteForceUnitAccess => "write_fua",
            RecordedOperationKind::WriteZeroes => "write_zeroes",
            RecordedOperationKind::Barrier => "barrier",
        };
        let digest = op.contents.as_ref().map_or("-".to_string(), |c| sha256_hexadecimal(c));
        writeln!(out, "{label}\top\t{}\t{}\t{kind}\t{}\t{}\t{digest}", index - mkfs_ops, op.operation.device.0, op.operation.offset.0, op.operation.length).unwrap();
    }
}

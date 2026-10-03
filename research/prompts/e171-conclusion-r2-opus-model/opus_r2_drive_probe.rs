//! e171-conclusion-r2 攻方腿的探针（只在腿的副本上编、跑；不进 crates）。
//! 照 E171 装置流二（固定脚本到 E 再正常卸载）的调用序列，在内存稀疏盘上（不建镜像文件）驱动一遍：
//! 取号、暖机、新池新建文件、覆盖写、进程重开之后可写挂载、覆盖写、回退、五次覆盖写、抬 F 到 11、覆盖写、卸载。
//! 每次调用 catch_unwind；每次调用成功之后对「到这一刻的全持久态」跑一次池级 checker（O2）。
//! 输出（OPUS_DRIVE_OUT，追加）：mkfs 之后每一步录制流（下标、盘、种类、偏移、长度、内容 sha256），每次调用的起止与结局，每次调用之后 O2 红了哪几条。
//! 用哪份 core 由编译时的副本决定（基线 core 用来核这份驱动与 E171 导出的 ops.tsv 逐条相同；变异 core 用来看第 125 行流二被丢掉的前 13 次调用）。
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
    fn call<T>(&mut self, name: &str, f: impl FnOnce() -> Result<T, String>) -> Option<T> {
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
        writeln!(self.out, "{}\tcall\t{}\t{}\t{}\t{}\t{}\to2_full_persisted={}", self.label, self.call, name, first, last, word, o2).unwrap();
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
fn opus_r2_drive_the_fixed_script_through_e_then_unmount() {
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
            let ok = run.call("roll_back", || roll_back_by_a_forward_publish(&parameters, &mut devices, &mut allocator, &mut o, target).map(|_| ()).map_err(|e| format!("{e:?}")));
            if ok.is_none() { break 'script }
            output = Some(o);
        }
        for seed in [13usize, 17, 19, 23, 29] {
            if !overwrite(&mut run, &mut devices, &mut allocator, &mut output, later_content(seed), InstanceGeneration(2)) { break 'script }
        }
        {
            let mut current = output.clone().unwrap();
            let ok = run.call("raise_floor", || raise_rollback_floor(&parameters, &mut devices, &mut allocator, &mut current, CheckpointTxg(11), ShadowLedger::On).map(|_| ()).map_err(|e| format!("{e:?}")));
            output = Some(current);
            if ok.is_none() { break 'script }
        }
        if !overwrite(&mut run, &mut devices, &mut allocator, &mut output, later_content(31), InstanceGeneration(2)) { break 'script }
        {
            let mut current = PoolVersion::WithFile(output.clone().unwrap());
            let s = stream.clone();
            let _ = run.call("unmount", || s.record_entry(RecordedPublishEntry::Unmount, || unmount(&parameters, &mut devices, &mut allocator, &mut current, ShadowLedger::On)).map(|_| ()).map_err(|e| format!("{e:?}")));
        }
    }
    let mut out = std::fs::OpenOptions::new().append(true).open(&out_path).unwrap();
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

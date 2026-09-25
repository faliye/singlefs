//! m2-final-code-r2 云端攻方 Z8：发布失败原样重发。在冻结副本的拷贝上跑（不开容量开关）。
#![allow(clippy::all, clippy::pedantic, dead_code, unused_imports)]

mod common;

use std::cell::Cell;
use std::rc::Rc;

use common::{build_pool, crash_state_devices, file_content, parameters, FIXED_WRITE_TIME_SECONDS, IMAGE_BYTES};
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
use singlefs_core::allocator::PoolAllocator;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_core::mount::mount_writable;
use singlefs_core::recovery::{recover, JournalPolicy, RecoveryOutcome};
use singlefs_core::transaction::{
    publish_overwrite, publish_sequential_write, resend_the_frozen_publish, FirstFile, PoolWriter, PublishError,
    TransactionOutput,
};
use singlefs_core::unit::data_unit_payload_capacity;
use singlefs_harness::crash::{MemoryPool, SparseBlockDevice};
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::{RecordedOperationKind, RecordingBlockDevice, RetainedOperation, SharedStream};

/// 两块盘共用一个计数器：第 `fail_at` 次设备操作（写或屏障，从 0 数）报错、什么都不做；`fail_at` 为 None 时照转。
struct NthOperationFails<Inner: BlockDevice> {
    inner: Inner,
    counter: Rc<Cell<u64>>,
    fail_at: Rc<Cell<Option<u64>>>,
}

impl<Inner: BlockDevice> NthOperationFails<Inner> {
    fn step(&self) -> bool {
        let n = self.counter.get();
        self.counter.set(n + 1);
        self.fail_at.get() == Some(n)
    }
}

impl<Inner: BlockDevice> BlockDevice for NthOperationFails<Inner> {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        if self.step() {
            return Err(injected_block_device_error("写"));
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        if self.step() {
            return Err(injected_block_device_error("清零"));
        }
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        if self.step() {
            return Err(injected_block_device_error("屏障"));
        }
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

type Wrapped = NthOperationFails<RecordingBlockDevice<SparseBlockDevice>>;

fn wrap(image: &MemoryPool, stream: &SharedStream, counter: &Rc<Cell<u64>>, fail_at: &Rc<Cell<Option<u64>>>) -> Vec<(DeviceIdentity, Wrapped)> {
    crash_state_devices(image, &[], &[], stream)
        .into_iter()
        .map(|(identity, inner)| (identity, NthOperationFails { inner, counter: counter.clone(), fail_at: fail_at.clone() }))
        .collect()
}

fn content_needing(data_units: usize, seed: usize) -> Vec<u8> {
    let payload_capacity = data_unit_payload_capacity();
    (0..(data_units - 1) * payload_capacity + payload_capacity / 2)
        .map(|index| u8::try_from((index * 11 + seed * 3 + 1) % 241).expect("小于 256"))
        .collect()
}

fn key(op: &RetainedOperation) -> (u32, RecordedOperationKind, u64, u64, u64) {
    let o = &op.operation;
    (o.device.0, o.kind, o.offset.0, o.length, o.content_hash)
}

fn is_system_configuration(op: &RetainedOperation) -> bool {
    op.operation.kind == RecordedOperationKind::Write && op.operation.offset.0 < 8192
}

fn describe(outcome: &RecoveryOutcome) -> String {
    match outcome {
        RecoveryOutcome::FileRead { root, content } => format!("FileRead({},{}) len={}", root.0 .0, root.1 .0, content.len()),
        other => format!("{other:?}").chars().take(160).collect(),
    }
}

fn publish(
    writer: &mut PoolWriter<'_, Wrapped>,
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    data_units: usize,
) -> Result<TransactionOutput, PublishError> {
    let content = content_needing(data_units, 5);
    let file = FirstFile { content: &content, write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60 };
    if data_units == 1 {
        publish_overwrite(writer, allocator, previous, file, InstanceGeneration(1))
    } else {
        publish_sequential_write(writer, allocator, previous, file, InstanceGeneration(1))
    }
}

/// 失败点逐个扫：B（`data_units` 个数据单元）的第 i 次设备操作报错 ⇒ 冻结；
/// ① 原样重发：重发那一遍的每一步（系统配置轮换之外）与不失败的参照流逐项相同（设备、种类、偏移、长度、内容哈希）；
/// ② 不重发、进程退出后恢复：结局只能是 A 或 B，不撞「两个末条标志」；之后可写挂载照常；
/// ③ 重发那一遍在第 j 步再失败一次，再重发：最终的流同 ①；
/// ④ 重发那一遍逐前缀崩：恢复结局只能是 A 或 B。
fn sweep(data_units: usize) {
    let pool = build_pool(&format!("z8-{data_units}"));
    let image_a = pool.memory_pool();
    let allocator_a = pool.allocator.clone();
    let a = pool.output.clone();
    drop(pool);
    let publish_parameters = parameters();

    // 参照：不失败。
    let reference: Vec<RetainedOperation> = {
        let stream = SharedStream::retaining_contents();
        let counter = Rc::new(Cell::new(0));
        let fail_at = Rc::new(Cell::new(None));
        let mut devices = wrap(&image_a, &stream, &counter, &fail_at);
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        let mut allocator = allocator_a.clone();
        publish(&mut writer, &mut allocator, &a, data_units).expect("参照 B");
        stream.retained_operations()
    };
    let reference_len = reference.len() as u64;
    let mut tallies = std::collections::BTreeMap::<String, usize>::new();
    let mut mismatches = Vec::new();
    for fail in 0..reference_len * 2 {
        let stream = SharedStream::retaining_contents();
        let counter = Rc::new(Cell::new(0));
        let fail_at = Rc::new(Cell::new(Some(fail)));
        let mut devices = wrap(&image_a, &stream, &counter, &fail_at);
        let mut allocator = allocator_a.clone();
        let first = {
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish(&mut writer, &mut allocator, &a, data_units)
        };
        if first.is_ok() {
            *tallies.entry("first attempt did not fail".into()).or_default() += 1;
            continue;
        }
        let failed_ops = stream.retained_operations();
        // ② 不重发：此刻的盘面恢复。
        let mut after_failure = image_a.clone();
        after_failure.apply(&failed_ops);
        let report = recover(&after_failure, JournalPolicy::Consult);
        *tallies.entry(format!("no-resend recover: {}", describe(&report.outcome))).or_default() += 1;
        let _ = fail;
        {
            let s2 = SharedStream::retaining_contents();
            let mut devs = crash_state_devices(&after_failure, &[], &[], &s2);
            match mount_writable(&publish_parameters, &mut devs) {
                Ok(_) => *tallies.entry("no-resend remount ok".into()).or_default() += 1,
                Err(e) => *tallies.entry(format!("no-resend remount ERR {e:?}").chars().take(120).collect()).or_default() += 1,
            }
        }
        // ③ 重发那一遍在第 j 步再失败一次（j = None 即不再失败）。
        let resend_len_hint = reference_len * 2;
        for second in std::iter::once(None).chain((0..resend_len_hint).map(Some)) {
            let stream_b = SharedStream::retaining_contents();
            let counter_b = Rc::new(Cell::new(0));
            let fail_b = Rc::new(Cell::new(second));
            let mut devs = wrap(&after_failure, &stream_b, &counter_b, &fail_b);
            let mut alloc = allocator.clone();
            let mut writer = PoolWriter::new(&publish_parameters, devs.as_mut_slice());
            let mut result = resend_the_frozen_publish(&mut writer, &mut alloc);
            if second.is_some() {
                if result.is_ok() {
                    continue; // j 越过了重发的长度
                }
                fail_b.set(None);
                result = resend_the_frozen_publish(&mut writer, &mut alloc);
            }
            drop(writer);
            let resent = result.expect("最后一次重发成功").expect("有冻结的发布");
            let ops = stream_b.retained_operations();
            // 最后一遍重发 = ops 的最后 reference.len() 步（第一次重发失败的那半截在前面）。
            let last_pass = &ops[ops.len().saturating_sub(reference.len())..];
            let same = last_pass.len() == reference.len()
                && last_pass.iter().zip(&reference).all(|(x, r)| {
                    if is_system_configuration(r) {
                        x.operation.device == r.operation.device && x.operation.kind == r.operation.kind && x.operation.length == r.operation.length
                    } else {
                        key(x) == key(r)
                    }
                });
            let sc_same = last_pass.len() == reference.len()
                && last_pass.iter().zip(&reference).filter(|(_, r)| is_system_configuration(r)).all(|(x, r)| key(x) == key(r));
            *tallies
                .entry(format!(
                    "resend second_failure={} identical_except_sysconfig={same} sysconfig_bytes_identical={sc_same}",
                    second.is_some()
                ))
                .or_default() += 1;
            if !same {
                mismatches.push(format!("fail={fail} second={second:?}"));
            }
            if resent.root().checkpoint_txg != CheckpointTxg(4) {
                mismatches.push(format!("fail={fail} second={second:?} txg {:?}", resent.root().checkpoint_txg));
            }
            // ④ 只对不再失败的那一遍做：逐前缀崩。
            if second.is_none() {
                for prefix in 0..=ops.len() {
                    let mut cut = after_failure.clone();
                    cut.apply(&ops[..prefix]);
                    let r = recover(&cut, JournalPolicy::Consult);
                    let d = describe(&r.outcome);
                    let b_len = format!("len={}", content_needing(data_units, 5).len());
                    let class = if d.starts_with("FileRead(1,3) len=3000") {
                        "A"
                    } else if d.starts_with("FileRead(1,4)") && d.ends_with(&b_len) {
                        "B(root 4)"
                    } else if d.starts_with("FileRead(1,3)") && d.ends_with(&b_len) {
                        "B(root 3 + replayed record)"
                    } else {
                        "OTHER"
                    };
                    *tallies.entry(format!("resend-prefix recover {class}")).or_default() += 1;
                    if class == "OTHER" {
                        mismatches.push(format!("fail={fail} resend prefix {prefix}: {d}"));
                    }
                }
            }
        }
    }
    println!("=== Z8 data_units={data_units} reference_ops={reference_len}");
    for (k, v) in &tallies {
        println!("{v:6}  {k}");
    }
    println!("mismatches={} {:?}", mismatches.len(), mismatches.iter().take(8).collect::<Vec<_>>());
}

#[test]
fn z8_one_unit() {
    sweep(1);
}

#[test]
fn z8_two_units() {
    sweep(2);
}

/// 调试：每个失败点，打印失败那一遍与重发那一遍里系统配置写的 (盘, 偏移, 哈希)，以及参照流的。
#[test]
fn z8_debug_rotation() {
    let pool = build_pool("z8-dbg");
    let image_a = pool.memory_pool();
    let allocator_a = pool.allocator.clone();
    let a = pool.output.clone();
    drop(pool);
    let publish_parameters = parameters();
    let sc = |ops: &[RetainedOperation]| -> Vec<(u32, u64, u64)> {
        ops.iter().filter(|o| is_system_configuration(o)).map(|o| (o.operation.device.0, o.operation.offset.0, o.operation.content_hash)).collect()
    };
    for fail in [None, Some(20u64), Some(21), Some(22)] {
        let stream = SharedStream::retaining_contents();
        let counter = Rc::new(Cell::new(0));
        let fail_at = Rc::new(Cell::new(fail));
        let mut devices = wrap(&image_a, &stream, &counter, &fail_at);
        let mut allocator = allocator_a.clone();
        let first = {
            let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
            publish(&mut writer, &mut allocator, &a, 1)
        };
        let failed_ops = stream.retained_operations();
        let kinds: Vec<String> = failed_ops.iter().map(|o| format!("{}:{:?}@{}", o.operation.device.0, o.operation.kind, o.operation.offset.0)).collect();
        println!("fail={fail:?} first_ok={} ops={} last3={:?} sysconfig={:?}", first.is_ok(), failed_ops.len(), &kinds[kinds.len().saturating_sub(3)..], sc(&failed_ops));
        if first.is_err() {
            let mut after = image_a.clone();
            after.apply(&failed_ops);
            let s2 = SharedStream::retaining_contents();
            let c2 = Rc::new(Cell::new(0));
            let f2 = Rc::new(Cell::new(None));
            let mut devs = wrap(&after, &s2, &c2, &f2);
            let mut writer = PoolWriter::new(&publish_parameters, devs.as_mut_slice());
            resend_the_frozen_publish(&mut writer, &mut allocator).expect("重发");
            drop(writer);
            println!("   resend sysconfig={:?}", sc(&s2.retained_operations()));
        }
    }
}

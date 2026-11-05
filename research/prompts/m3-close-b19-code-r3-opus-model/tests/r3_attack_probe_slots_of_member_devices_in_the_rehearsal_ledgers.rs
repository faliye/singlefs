//! 第三轮攻方探针（m3-close-b19-code-r3-opus，副本，不进仓）：取号之前预演续做判两本账时，只剔补写盘系统配置槽的读坏
//! （`mount.rs` 的 `read_back_a_system_configuration_slot_of_a_device_rewritten_by_the_acquisition`）。
//! 这里看补写盘之外的成员盘：一槽读得出、自证不过（整槽写 0，撕裂轮换写的同一类），或一槽每次读都报块设备错，另一槽自证得过；
//! 只进内存那一趟取号读每块盘两槽（`transaction::self_verified_system_configurations_of_every_device`），这一槽必然再读坏、读错。
//! 镜像是 `common_reads_shared_by_the_rehearsal_and_the_real_run::narrow_image`（续做中途推抬 F，取号之前在只进内存的盘上走一遍）。
//! 每一格打一行 PROBE：挂载做成还是拒、拒因点名哪一读。

#![allow(dead_code, reason = "几份 common 模块各只用到一部分")]

mod common;
mod common_admission;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;
mod common_reads_shared_by_the_rehearsal_and_the_real_run;

use std::cell::RefCell;
use std::rc::Rc;

use common::PoolImageGeometry;
use common_admission::plain_devices_on;
use common_reads_shared_by_the_rehearsal_and_the_real_run::{
    narrow_image, ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy,
    switches, ReadReachingTheDisk, BYTES_OF_A_TREE_TABLE,
};
use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::admission::SpaceAdmission;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_core::mount::{
    mount_writable_with_test_only_switches_and_deletion_switches, MountError, Mounted,
    ShadowLedger,
};
use singlefs_core::transaction::PublishError;
use singlefs_format::{SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE, SYSTEM_CONFIGURATION_SLOT_BYTES};
use singlefs_harness::fault_injection::injected_block_device_error;
use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};

const MEMBER_DEVICE: DeviceIdentity = DeviceIdentity(0);
const DEVICE_REWRITTEN_BY_THE_ACQUISITION: DeviceIdentity = DeviceIdentity(1);

#[derive(Clone, Copy, Debug)]
enum FaultKind {
    ReportsADeviceError,
    ReadsBackCorruptedBytes,
}

/// 整池第 `from_ordinal` 次读起，`device`（None 是哪块盘都算）上读 `offset` 起 `length` 字节都带 `kind` 那一种故障。
#[derive(Clone, Copy, Debug)]
struct FaultRule {
    device: Option<DeviceIdentity>,
    offset: DeviceOffsetInBytes,
    length: u64,
    from_ordinal: u64,
    kind: FaultKind,
}

struct Ledger {
    rules: Vec<FaultRule>,
    reads: Vec<ReadReachingTheDisk>,
    writes: u64,
}

struct DeviceWithFaultRules {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    ledger: Rc<RefCell<Ledger>>,
}

impl BlockDevice for DeviceWithFaultRules {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let mut ledger = self.ledger.borrow_mut();
        let this_read = ReadReachingTheDisk {
            ordinal_across_the_pool: u64::try_from(ledger.reads.len()).expect("读数") + 1,
            device: self.identity,
            offset,
            length_in_bytes: u64::try_from(buffer.len()).expect("读长"),
            writes_of_the_mount_before_it: ledger.writes,
        };
        ledger.reads.push(this_read);
        let rule = ledger.rules.iter().copied().find(|rule| {
            this_read.ordinal_across_the_pool >= rule.from_ordinal
                && rule.device.is_none_or(|device| device == self.identity)
                && rule.offset == offset
                && rule.length == this_read.length_in_bytes
        });
        drop(ledger);
        match rule.map(|rule| rule.kind) {
            None => self.inner.read_at(offset, buffer),
            Some(FaultKind::ReportsADeviceError) => Err(injected_block_device_error("读")),
            Some(FaultKind::ReadsBackCorruptedBytes) => {
                self.inner.read_at(offset, buffer)?;
                buffer[0] ^= 0xFF;
                Ok(())
            }
        }
    }

    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.ledger.borrow_mut().writes += 1;
        self.inner.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.ledger.borrow_mut().writes += 1;
        self.inner.write_zeroes_at(offset, length)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }

    fn journal_record_offsets_hint(&self, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner.journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

/// 一次可写挂载（准入照式子判、影子账开、删除开关同 common 那一份）：结局、读到盘上的每一次读、写的个数。
fn mount_with_rules(
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    rules: Vec<FaultRule>,
) -> (Result<Mounted, MountError>, Vec<ReadReachingTheDisk>, u64) {
    let ledger = Rc::new(RefCell::new(Ledger { rules, reads: Vec::new(), writes: 0 }));
    let mut devices: Vec<(DeviceIdentity, DeviceWithFaultRules)> = plain_devices_on(image)
        .into_iter()
        .map(|(identity, inner)| (identity, DeviceWithFaultRules { identity, inner, ledger: Rc::clone(&ledger) }))
        .collect();
    let result = mount_writable_with_test_only_switches_and_deletion_switches(
        &geometry.parameters,
        &mut devices,
        SpaceAdmission::JudgedByTheFormula,
        ShadowLedger::On,
        switches(),
    );
    let ledger = ledger.borrow();
    (result, ledger.reads.clone(), ledger.writes)
}

fn slot_offset(geometry: &PoolImageGeometry, slot_index: u64) -> DeviceOffsetInBytes {
    DeviceOffsetInBytes(slot_index * u64::from(geometry.parameters.geometry.fixed_structure_slot_spacing))
}

/// `device` 上 `slots` 那几个系统配置槽整槽写 0（读得出、自证不过）。
fn with_slots_zeroed(image: &MemoryPool, geometry: &PoolImageGeometry, device: DeviceIdentity, slots: &[u64]) -> MemoryPool {
    let mut zeroed = image.clone();
    let slot_bytes = usize::try_from(SYSTEM_CONFIGURATION_SLOT_BYTES).expect("4096");
    let device_image = zeroed.devices.get_mut(&device).expect("池里有这块盘");
    for slot_index in slots {
        device_image.write(slot_offset(geometry, *slot_index), &vec![0u8; slot_bytes]);
    }
    zeroed
}

/// 只进内存那一趟头一次读的那份树表，从那一次读起两块盘上那两份每一次读都读回坏字节（U2 用例那一形）。
fn tree_table_reading_back_garbage_every_time(reads_without_it: &[ReadReachingTheDisk]) -> FaultRule {
    let (window, offset) =
        ordinals_of_a_tree_table_read_first_by_the_in_memory_rehearsal_and_of_its_other_copy(reads_without_it);
    FaultRule {
        device: None,
        offset,
        length: BYTES_OF_A_TREE_TABLE,
        from_ordinal: *window.first().expect("窗口两次读"),
        kind: FaultKind::ReadsBackCorruptedBytes,
    }
}

fn outcome(result: &Result<Mounted, MountError>, writes: u64) -> String {
    match result {
        Ok(mounted) => format!(
            "MountMade devices_rewritten={:?} continuation_steps={} writes={writes}",
            mounted.output.devices_rewritten_by_the_acquisition,
            mounted.output.deletion_queue_continuation.len()
        ),
        Err(MountError::Publish(failed)) => {
            let cause = match &failed.cause {
                PublishError::ReadFailedWhileRehearsingTheContinuationBeforeAcquisition(read) => format!(
                    "ReadFailedWhileRehearsing device={:?} offset={:?} length={}",
                    read.device, read.offset, read.length_in_bytes
                ),
                PublishError::ReadBackFailingSelfVerificationWhileRehearsingTheContinuationBeforeAcquisition(read) => format!(
                    "ReadBackFailingSelfVerificationWhileRehearsing device={:?} offset={:?} length={}",
                    read.device, read.offset, read.length_in_bytes
                ),
                other => format!("{other:?}"),
            };
            format!("Refused {cause} writes={writes}")
        }
        Err(other) => format!("Refused {other:?} writes={writes}"),
    }
}

/// 不带别的故障挂一次、再叠上树表每次读坏挂一次（不带别的故障那一次挂成了才叠）。
fn fault_free_then_with_the_tree_table_reading_back_garbage(
    name: &str,
    image: &MemoryPool,
    geometry: &PoolImageGeometry,
    rules_kept: &[FaultRule],
) {
    let (result, reads, writes) = mount_with_rules(image, geometry, rules_kept.to_vec());
    println!("PROBE name={name}_without_the_tree_table_fault result={}", outcome(&result, writes));
    if result.is_err() {
        return;
    }
    let mut rules = rules_kept.to_vec();
    rules.push(tree_table_reading_back_garbage_every_time(&reads));
    let tree_table_offset = rules.last().expect("刚加的").offset;
    let (result, _, writes) = mount_with_rules(image, geometry, rules);
    println!(
        "PROBE name={name}_with_the_tree_table_reading_back_garbage tree_table_offset={tree_table_offset:?} result={}",
        outcome(&result, writes)
    );
}

#[test]
fn a_member_slot_zeroed_on_a_device_not_rewritten_by_the_acquisition() {
    let (image, geometry, _) = narrow_image();
    fault_free_then_with_the_tree_table_reading_back_garbage("baseline", &image, &geometry, &[]);
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let torn = with_slots_zeroed(&image, &geometry, MEMBER_DEVICE, &[slot_index]);
        fault_free_then_with_the_tree_table_reading_back_garbage(
            &format!("member_device0_slot{slot_index}_zeroed"),
            &torn,
            &geometry,
            &[],
        );
    }
}

#[test]
fn a_device_rewritten_by_the_acquisition_and_a_member_slot_zeroed_on_the_other_device() {
    let (image, geometry, _) = narrow_image();
    let rewritten = with_slots_zeroed(&image, &geometry, DEVICE_REWRITTEN_BY_THE_ACQUISITION, &[0, 1]);
    fault_free_then_with_the_tree_table_reading_back_garbage("device1_both_zeroed", &rewritten, &geometry, &[]);
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let both = with_slots_zeroed(&rewritten, &geometry, MEMBER_DEVICE, &[slot_index]);
        fault_free_then_with_the_tree_table_reading_back_garbage(
            &format!("device1_both_zeroed_device0_slot{slot_index}_zeroed"),
            &both,
            &geometry,
            &[],
        );
    }
}

#[test]
fn a_member_slot_reporting_a_device_error_every_time() {
    let (image, geometry, _) = narrow_image();
    for slot_index in 0..SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE {
        let read_error_on_the_slot = FaultRule {
            device: Some(MEMBER_DEVICE),
            offset: slot_offset(&geometry, slot_index),
            length: SYSTEM_CONFIGURATION_SLOT_BYTES,
            from_ordinal: 1,
            kind: FaultKind::ReportsADeviceError,
        };
        fault_free_then_with_the_tree_table_reading_back_garbage(
            &format!("member_device0_slot{slot_index}_device_error_every_time"),
            &image,
            &geometry,
            &[read_error_on_the_slot],
        );
    }
}

//! 三个设备包装（读日志 `read_tally::ReadLoggingBlockDevice`、录制 `RecordingBlockDevice`、故障注入 `fault_injection::FaultInjectingBlockDevice`）
//! 把两种专用读（读坏之后的重读 `reread_at_after_a_bad_read`、这一刻读盘 `read_at_straight_from_the_disk`）交给里面那块盘的同名方法、
//! 把两个读缓存通知（`keep_the_self_verified_reread`、`note_the_read_failing_self_verification_and_forget_its_bytes`）原样转给它
//! （`singlefs_core::block_device::BlockDevice` 这四个方法的文档；`records/2026-10-24-singlefs-harness-恢复验证审计.md`
//! 「H02：三个设备包装器丢失专用读路由与缓存通知」）。
//!
//! 里面那块盘是一个测试设备：普通读读出 0x11、重读读出 0x22、这一刻读盘读出 0x33，四种调用各自记下收到的参数。
//! 记账一侧（读日志的读数与 journal 环偏移、录制流的读区间、故障计划的调用数与转发读数）与改前同数：改前两种专用读落回包装自己的
//! `read_at`，所以拿同样搭起来的另一份包装发普通读当改前的样子比。

use std::cell::RefCell;
use std::rc::Rc;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_harness::fault_injection::{
    FaultInjectingBlockDevice, FaultSchedule, FlippedBit, InjectedFault, SharedFaultPlan,
};
use singlefs_harness::history::HistoryDeviceWidth;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::read_tally::{JournalHintAnswer, JournalRingRegion, ReadLoggingBlockDevice};
use singlefs_harness::{RecordingBlockDevice, SharedStream};

const PLAIN_READ_BYTE: u8 = 0x11;
const REREAD_BYTE: u8 = 0x22;
const STRAIGHT_FROM_THE_DISK_BYTE: u8 = 0x33;
const TEST_DEVICE_BYTES: u64 = 1 << 20;
const READ_LENGTH: usize = 512;
const REREAD_OFFSET: DeviceOffsetInBytes = DeviceOffsetInBytes(4096);
const STRAIGHT_READ_OFFSET: DeviceOffsetInBytes = DeviceOffsetInBytes(16384);
const KEPT_REREAD_OFFSET: DeviceOffsetInBytes = DeviceOffsetInBytes(8192);
const FORGOTTEN_READ_OFFSET: DeviceOffsetInBytes = DeviceOffsetInBytes(12288);
const FORGOTTEN_READ_LENGTH: u64 = 1024;
const WRAPPED_DEVICE: DeviceIdentity = DeviceIdentity(0);

/// 里面那块测试设备收到过什么，按到达次序。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CallsReceived {
    plain_reads: Vec<DeviceOffsetInBytes>,
    rereads: Vec<DeviceOffsetInBytes>,
    reads_straight_from_the_disk: Vec<DeviceOffsetInBytes>,
    kept_rereads: Vec<(DeviceOffsetInBytes, Vec<u8>)>,
    forgotten_reads: Vec<(DeviceOffsetInBytes, u64)>,
}

/// 三种读各读出一种字节、两个通知各记参数的测试设备；记下的东西放在共用的 [`Rc`] 里，包在包装里面也看得见。
struct DeviceWithDedicatedReads {
    calls_received: Rc<RefCell<CallsReceived>>,
}

impl BlockDevice for DeviceWithDedicatedReads {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.calls_received.borrow_mut().plain_reads.push(offset);
        buffer.fill(PLAIN_READ_BYTE);
        Ok(())
    }

    fn write_at(
        &mut self,
        _offset: DeviceOffsetInBytes,
        _bytes: &[u8],
        _durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn write_zeroes_at(
        &mut self,
        _offset: DeviceOffsetInBytes,
        _length: u64,
    ) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        PhysicalBlockSizeInBytes(512)
    }

    fn size_in_bytes(&self) -> u64 {
        TEST_DEVICE_BYTES
    }

    fn reread_at_after_a_bad_read(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.calls_received.borrow_mut().rereads.push(offset);
        buffer.fill(REREAD_BYTE);
        Ok(())
    }

    fn keep_the_self_verified_reread(&self, offset: DeviceOffsetInBytes, bytes: &[u8]) {
        self.calls_received
            .borrow_mut()
            .kept_rereads
            .push((offset, bytes.to_vec()));
    }

    fn note_the_read_failing_self_verification_and_forget_its_bytes(
        &self,
        offset: DeviceOffsetInBytes,
        length_in_bytes: u64,
    ) {
        self.calls_received
            .borrow_mut()
            .forgotten_reads
            .push((offset, length_in_bytes));
    }

    fn read_at_straight_from_the_disk(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.calls_received
            .borrow_mut()
            .reads_straight_from_the_disk
            .push(offset);
        buffer.fill(STRAIGHT_FROM_THE_DISK_BYTE);
        Ok(())
    }
}

fn fresh_device_with_dedicated_reads() -> (DeviceWithDedicatedReads, Rc<RefCell<CallsReceived>>) {
    let calls_received = Rc::new(RefCell::new(CallsReceived::default()));
    (
        DeviceWithDedicatedReads {
            calls_received: Rc::clone(&calls_received),
        },
        calls_received,
    )
}

/// 一个设备上发一次重读、一次这一刻读盘、两个通知各一次之后看得见的：两种读读出的字节，与里面那块盘收到了什么。
#[derive(Debug, PartialEq, Eq)]
struct DedicatedCallsSeen {
    reread_bytes: Vec<u8>,
    straight_from_the_disk_bytes: Vec<u8>,
    calls_received: CallsReceived,
}

fn issue_the_dedicated_reads_and_the_notices<Device: BlockDevice>(
    device: &Device,
    calls_received: &RefCell<CallsReceived>,
) -> DedicatedCallsSeen {
    let mut reread_bytes = vec![0u8; READ_LENGTH];
    device
        .reread_at_after_a_bad_read(REREAD_OFFSET, &mut reread_bytes)
        .expect("重读");
    let mut straight_from_the_disk_bytes = vec![0u8; READ_LENGTH];
    device
        .read_at_straight_from_the_disk(STRAIGHT_READ_OFFSET, &mut straight_from_the_disk_bytes)
        .expect("这一刻读盘");
    device.keep_the_self_verified_reread(KEPT_REREAD_OFFSET, &[REREAD_BYTE; READ_LENGTH]);
    device.note_the_read_failing_self_verification_and_forget_its_bytes(
        FORGOTTEN_READ_OFFSET,
        FORGOTTEN_READ_LENGTH,
    );
    DedicatedCallsSeen {
        reread_bytes,
        straight_from_the_disk_bytes,
        calls_received: calls_received.borrow().clone(),
    }
}

/// 直接发给里面那块测试设备时看得见的：每个包装经它发都要与这一份逐项相同。
fn seen_directly_on_the_inner_device() -> DedicatedCallsSeen {
    let (inner, calls_received) = fresh_device_with_dedicated_reads();
    let seen = issue_the_dedicated_reads_and_the_notices(&inner, &calls_received);
    assert_eq!(
        seen,
        DedicatedCallsSeen {
            reread_bytes: vec![REREAD_BYTE; READ_LENGTH],
            straight_from_the_disk_bytes: vec![STRAIGHT_FROM_THE_DISK_BYTE; READ_LENGTH],
            calls_received: CallsReceived {
                plain_reads: Vec::new(),
                rereads: vec![REREAD_OFFSET],
                reads_straight_from_the_disk: vec![STRAIGHT_READ_OFFSET],
                kept_rereads: vec![(KEPT_REREAD_OFFSET, vec![REREAD_BYTE; READ_LENGTH])],
                forgotten_reads: vec![(FORGOTTEN_READ_OFFSET, FORGOTTEN_READ_LENGTH)],
            },
        },
        "直接发：测试设备的四个专用方法各到一次、普通读一次都没有"
    );
    seen
}

/// 读日志的测试用环：一段盖住重读那一读、不盖这一刻读盘那一读的窗口，两种读记不记环里起点分得开。
const TEST_JOURNAL_RING: JournalRingRegion = JournalRingRegion {
    start: DeviceOffsetInBytes(0),
    length_in_bytes: 8192,
};

fn read_logging_device_around<Inner: BlockDevice>(inner: Inner) -> ReadLoggingBlockDevice<Inner> {
    ReadLoggingBlockDevice::new(
        inner,
        TEST_JOURNAL_RING,
        JournalHintAnswer::PassTheInnerDeviceAnswerThrough,
    )
}

fn fault_plan_of_the_history_geometry() -> SharedFaultPlan {
    SharedFaultPlan::unarmed(HistoryDeviceWidth::FourGibibytes.fixed_geometry())
}

#[test]
fn read_logging_device_hands_the_dedicated_reads_and_the_notices_to_the_inner_device() {
    let direct = seen_directly_on_the_inner_device();
    let (inner, calls_received) = fresh_device_with_dedicated_reads();
    let wrapped = read_logging_device_around(inner);
    assert_eq!(
        issue_the_dedicated_reads_and_the_notices(&wrapped, &calls_received),
        direct,
        "经读日志包装：读出的字节、里面那块盘收到的调用与参数都与直接发相同"
    );
}

#[test]
fn recording_device_hands_the_dedicated_reads_and_the_notices_to_the_inner_device() {
    let direct = seen_directly_on_the_inner_device();
    let (inner, calls_received) = fresh_device_with_dedicated_reads();
    let wrapped = RecordingBlockDevice::with_shared_stream(
        WRAPPED_DEVICE,
        inner,
        SharedStream::retaining_contents_and_reads(),
    );
    assert_eq!(
        issue_the_dedicated_reads_and_the_notices(&wrapped, &calls_received),
        direct,
        "经录制包装：读出的字节、里面那块盘收到的调用与参数都与直接发相同"
    );
}

#[test]
fn fault_injecting_device_with_an_unarmed_plan_hands_the_dedicated_reads_and_the_notices_to_the_inner_device(
) {
    let direct = seen_directly_on_the_inner_device();
    let (inner, calls_received) = fresh_device_with_dedicated_reads();
    let wrapped =
        FaultInjectingBlockDevice::new(WRAPPED_DEVICE, inner, fault_plan_of_the_history_geometry());
    assert_eq!(
        issue_the_dedicated_reads_and_the_notices(&wrapped, &calls_received),
        direct,
        "经不武装的故障包装：读出的字节、里面那块盘收到的调用与参数都与直接发相同"
    );
}

/// 改前两种专用读落回包装自己的 `read_at`：同样搭起来的另一份包装在同两个偏移上发普通读，就是改前的记账。
fn issue_plain_reads_where_the_dedicated_reads_go<Device: BlockDevice>(device: &Device) {
    let mut buffer = vec![0u8; READ_LENGTH];
    device
        .read_at(REREAD_OFFSET, &mut buffer)
        .expect("改前重读那一读");
    device
        .read_at(STRAIGHT_READ_OFFSET, &mut buffer)
        .expect("改前这一刻读盘那一读");
}

#[test]
fn read_logging_device_logs_each_dedicated_read_as_the_plain_read_it_used_to_be() {
    let (dedicated_inner, dedicated_calls) = fresh_device_with_dedicated_reads();
    let dedicated = read_logging_device_around(dedicated_inner);
    issue_the_dedicated_reads_and_the_notices(&dedicated, &dedicated_calls);
    let (plain_inner, _plain_calls) = fresh_device_with_dedicated_reads();
    let plain = read_logging_device_around(plain_inner);
    issue_plain_reads_where_the_dedicated_reads_go(&plain);
    assert_eq!(
        dedicated.read_log(),
        plain.read_log(),
        "两种专用读各记一次读、环里起点照记，两个通知不记：与改前发普通读的读日志相同"
    );
    assert_eq!(dedicated.read_log().reads, 2, "两次专用读记两次读");
    assert_eq!(
        dedicated.read_log().journal_ring_read_offsets,
        vec![REREAD_OFFSET],
        "起点落在环里的只有重读那一读"
    );
}

#[test]
fn recording_device_records_each_dedicated_read_range_as_the_plain_read_it_used_to_be() {
    let dedicated_stream = SharedStream::retaining_contents_and_reads();
    let (dedicated_inner, dedicated_calls) = fresh_device_with_dedicated_reads();
    let dedicated = RecordingBlockDevice::with_shared_stream(
        WRAPPED_DEVICE,
        dedicated_inner,
        dedicated_stream.clone(),
    );
    issue_the_dedicated_reads_and_the_notices(&dedicated, &dedicated_calls);
    let plain_stream = SharedStream::retaining_contents_and_reads();
    let (plain_inner, _plain_calls) = fresh_device_with_dedicated_reads();
    let plain =
        RecordingBlockDevice::with_shared_stream(WRAPPED_DEVICE, plain_inner, plain_stream.clone());
    issue_plain_reads_where_the_dedicated_reads_go(&plain);
    assert_eq!(
        dedicated_stream.read_ranges(),
        plain_stream.read_ranges(),
        "两种专用读各记一段读、两个通知不记：与改前发普通读的读区间相同"
    );
    assert_eq!(dedicated_stream.read_ranges().len(), 2, "两段读");
    assert_eq!(
        dedicated_stream.operations(),
        plain_stream.operations(),
        "读与通知都不进录制流"
    );
}

#[test]
fn fault_injecting_device_counts_each_dedicated_read_as_the_plain_read_it_used_to_be() {
    let dedicated_plan = fault_plan_of_the_history_geometry();
    let (dedicated_inner, dedicated_calls) = fresh_device_with_dedicated_reads();
    let dedicated =
        FaultInjectingBlockDevice::new(WRAPPED_DEVICE, dedicated_inner, dedicated_plan.clone());
    issue_the_dedicated_reads_and_the_notices(&dedicated, &dedicated_calls);
    let plain_plan = fault_plan_of_the_history_geometry();
    let (plain_inner, _plain_calls) = fresh_device_with_dedicated_reads();
    let plain = FaultInjectingBlockDevice::new(WRAPPED_DEVICE, plain_inner, plain_plan.clone());
    issue_plain_reads_where_the_dedicated_reads_go(&plain);
    assert_eq!(
        dedicated_plan.calls(),
        plain_plan.calls(),
        "整池调用数：两种专用读各算一次读、两个通知不算"
    );
    assert_eq!(dedicated_plan.calls().reads, 2, "两次读");
    assert_eq!(
        dedicated_plan.counts_of_device(WRAPPED_DEVICE),
        plain_plan.counts_of_device(WRAPPED_DEVICE),
        "这块盘的转发读数与改前相同"
    );
    assert_eq!(
        dedicated_plan.counts_of_device(WRAPPED_DEVICE).reads,
        2,
        "两次读都交给了里面那块盘"
    );
}

/// 经故障包装发的一次专用读，故障计划在整池第 1 次读上注入：读出的结局（错误写成字）、注入记的调用序号、里面那块盘收到了什么、
/// 这块盘记下的转发读数。
struct DedicatedReadUnderAReadFault {
    result: Result<Vec<u8>, String>,
    fired_ordinals: Vec<u64>,
    received: CallsReceived,
    reads_forwarded: u64,
}

type DedicatedReadThroughTheFaultInjectingDevice = fn(
    &FaultInjectingBlockDevice<DeviceWithDedicatedReads>,
    &mut [u8],
) -> Result<(), BlockDeviceError>;

fn dedicated_read_under_a_read_fault(
    fault: InjectedFault,
    dedicated_read: DedicatedReadThroughTheFaultInjectingDevice,
) -> DedicatedReadUnderAReadFault {
    let plan = SharedFaultPlan::armed(
        HistoryDeviceWidth::FourGibibytes.fixed_geometry(),
        FaultSchedule::the_nth_call_across_the_pool(fault, 1),
    );
    let (inner, calls_received) = fresh_device_with_dedicated_reads();
    let wrapped = FaultInjectingBlockDevice::new(WRAPPED_DEVICE, inner, plan.clone());
    let mut buffer = vec![0u8; READ_LENGTH];
    let result = dedicated_read(&wrapped, &mut buffer)
        .map(|()| buffer)
        .map_err(|error| error.to_string());
    let fired_ordinals = plan
        .fired()
        .iter()
        .map(|fired| fired.call_ordinal_across_the_pool)
        .collect();
    let received = calls_received.borrow().clone();
    DedicatedReadUnderAReadFault {
        result,
        fired_ordinals,
        received,
        reads_forwarded: plan.counts_of_device(WRAPPED_DEVICE).reads,
    }
}

fn reread_through(
    device: &FaultInjectingBlockDevice<DeviceWithDedicatedReads>,
    buffer: &mut [u8],
) -> Result<(), BlockDeviceError> {
    device.reread_at_after_a_bad_read(REREAD_OFFSET, buffer)
}

fn read_straight_from_the_disk_through(
    device: &FaultInjectingBlockDevice<DeviceWithDedicatedReads>,
    buffer: &mut [u8],
) -> Result<(), BlockDeviceError> {
    device.read_at_straight_from_the_disk(STRAIGHT_READ_OFFSET, buffer)
}

const FLIPPED_BIT_ZERO_OF_BYTE_ZERO: FlippedBit = FlippedBit {
    byte_index: 0,
    bit_index: 0,
};

#[test]
fn fault_injecting_device_fails_a_reread_when_the_plan_fails_that_read() {
    let DedicatedReadUnderAReadFault {
        result,
        fired_ordinals,
        received,
        reads_forwarded,
    } = dedicated_read_under_a_read_fault(InjectedFault::ReadFails, reread_through);
    assert!(
        result
            .as_ref()
            .is_err_and(|error| error.contains("注入的读错")),
        "重读同样被注入读失败，实际 {result:?}"
    );
    assert_eq!(fired_ordinals, vec![1], "注入记在整池第 1 次读上");
    assert!(received.rereads.is_empty(), "读失败不交给里面那块盘");
    assert_eq!(reads_forwarded, 0, "没转发的读不记");
}

#[test]
fn fault_injecting_device_fails_a_read_straight_from_the_disk_when_the_plan_fails_that_read() {
    let DedicatedReadUnderAReadFault {
        result,
        fired_ordinals,
        received,
        reads_forwarded,
    } = dedicated_read_under_a_read_fault(
        InjectedFault::ReadFails,
        read_straight_from_the_disk_through,
    );
    assert!(
        result
            .as_ref()
            .is_err_and(|error| error.contains("注入的读错")),
        "这一刻读盘同样被注入读失败，实际 {result:?}"
    );
    assert_eq!(fired_ordinals, vec![1], "注入记在整池第 1 次读上");
    assert!(
        received.reads_straight_from_the_disk.is_empty(),
        "读失败不交给里面那块盘"
    );
    assert_eq!(reads_forwarded, 0, "没转发的读不记");
}

#[test]
fn fault_injecting_device_flips_a_bit_of_a_reread_when_the_plan_corrupts_that_read() {
    let DedicatedReadUnderAReadFault {
        result,
        fired_ordinals,
        received,
        reads_forwarded,
    } = dedicated_read_under_a_read_fault(
        InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FLIPPED_BIT_ZERO_OF_BYTE_ZERO,
        },
        reread_through,
    );
    let mut expected = vec![REREAD_BYTE; READ_LENGTH];
    expected[0] ^= 1;
    assert_eq!(
        result,
        Ok(expected),
        "重读交给里面那块盘的重读（0x22），再翻掉第 0 个字节的第 0 位"
    );
    assert_eq!(fired_ordinals, vec![1], "注入记在整池第 1 次读上");
    assert_eq!(received.rereads, vec![REREAD_OFFSET], "里面那块盘收到重读");
    assert_eq!(reads_forwarded, 1, "转发的读照记");
}

#[test]
fn fault_injecting_device_flips_a_bit_of_a_read_straight_from_the_disk_when_the_plan_corrupts_that_read(
) {
    let DedicatedReadUnderAReadFault {
        result,
        fired_ordinals,
        received,
        reads_forwarded,
    } = dedicated_read_under_a_read_fault(
        InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FLIPPED_BIT_ZERO_OF_BYTE_ZERO,
        },
        read_straight_from_the_disk_through,
    );
    let mut expected = vec![STRAIGHT_FROM_THE_DISK_BYTE; READ_LENGTH];
    expected[0] ^= 1;
    assert_eq!(
        result,
        Ok(expected),
        "这一刻读盘交给里面那块盘的同名方法（0x33），再翻掉第 0 个字节的第 0 位"
    );
    assert_eq!(fired_ordinals, vec![1], "注入记在整池第 1 次读上");
    assert_eq!(
        received.reads_straight_from_the_disk,
        vec![STRAIGHT_READ_OFFSET],
        "里面那块盘收到这一刻读盘"
    );
    assert_eq!(reads_forwarded, 1, "转发的读照记");
}

/// 随机历史用的那一层栈（`history::HistoryDevice`：故障注入包录制包稀疏内存盘，内存盘用默认的专用读与通知）上的一段操作，
/// 交回录制流、读区间、故障计划的调用数、转发计数、注入记录与读出的字节。
#[derive(Debug, PartialEq, Eq)]
struct HistoryStackSeen {
    stream_text: String,
    read_ranges: Vec<singlefs_harness::RecordedReadRange>,
    calls: singlefs_harness::fault_injection::FaultCallCounts,
    device_counts: singlefs_harness::fault_injection::FaultDeviceCounts,
    fired_ordinals: Vec<u64>,
    bytes_read: Vec<Vec<u8>>,
}

/// 写一段、屏障，再在写过的那一段上读两次（`DedicatedReads` 时是重读与这一刻读盘、各带一次通知，`PlainReads` 时是改前的两次普通读），
/// 故障计划在整池第 2 次读上翻一位。
#[derive(Clone, Copy, Debug)]
enum ReadsOfTheHistoryStackOperation {
    DedicatedReads,
    PlainReads,
}

fn run_the_history_stack_operation(reads: ReadsOfTheHistoryStackOperation) -> HistoryStackSeen {
    let device_width = HistoryDeviceWidth::FourGibibytes;
    let stream = SharedStream::retaining_contents_and_reads();
    let plan = SharedFaultPlan::unarmed(device_width.fixed_geometry());
    let mut device = FaultInjectingBlockDevice::new(
        WRAPPED_DEVICE,
        RecordingBlockDevice::with_shared_stream(
            WRAPPED_DEVICE,
            SparseBlockDevice::new(device_width.device_bytes(), PhysicalBlockSizeInBytes(512)),
            stream.clone(),
        ),
        plan.clone(),
    );
    device
        .write_at(
            REREAD_OFFSET,
            &[REREAD_BYTE; READ_LENGTH],
            WriteDurability::Plain,
        )
        .expect("写");
    device.barrier().expect("屏障");
    plan.arm(FaultSchedule::the_nth_call_across_the_pool(
        InjectedFault::ReadReturnsCorruptedBytes {
            flipped_bit: FLIPPED_BIT_ZERO_OF_BYTE_ZERO,
        },
        2,
    ));
    let mut bytes_read = Vec::new();
    let mut buffer_of_the_reread = vec![0u8; READ_LENGTH];
    let mut buffer_of_the_straight_read = vec![0u8; READ_LENGTH];
    match reads {
        ReadsOfTheHistoryStackOperation::DedicatedReads => {
            device
                .reread_at_after_a_bad_read(REREAD_OFFSET, &mut buffer_of_the_reread)
                .expect("重读");
            device.keep_the_self_verified_reread(REREAD_OFFSET, &buffer_of_the_reread);
            device
                .read_at_straight_from_the_disk(REREAD_OFFSET, &mut buffer_of_the_straight_read)
                .expect("这一刻读盘");
            device.note_the_read_failing_self_verification_and_forget_its_bytes(
                REREAD_OFFSET,
                u64::try_from(READ_LENGTH).expect("读长装得进 u64"),
            );
        }
        ReadsOfTheHistoryStackOperation::PlainReads => {
            device
                .read_at(REREAD_OFFSET, &mut buffer_of_the_reread)
                .expect("改前重读那一读");
            device
                .read_at(REREAD_OFFSET, &mut buffer_of_the_straight_read)
                .expect("改前这一刻读盘那一读");
        }
    }
    bytes_read.push(buffer_of_the_reread);
    bytes_read.push(buffer_of_the_straight_read);
    HistoryStackSeen {
        stream_text: stream.render(),
        read_ranges: stream.read_ranges(),
        calls: plan.calls(),
        device_counts: plan.counts_of_device(WRAPPED_DEVICE),
        fired_ordinals: plan
            .fired()
            .iter()
            .map(|fired| fired.call_ordinal_across_the_pool)
            .collect(),
        bytes_read,
    }
}

#[test]
fn history_stack_records_and_counts_the_dedicated_reads_as_the_plain_reads_they_used_to_be() {
    let dedicated =
        run_the_history_stack_operation(ReadsOfTheHistoryStackOperation::DedicatedReads);
    let plain = run_the_history_stack_operation(ReadsOfTheHistoryStackOperation::PlainReads);
    assert_eq!(
        dedicated, plain,
        "内存盘用默认的专用读：录制流、读区间、调用数、转发计数、注入序号与读出的字节都与改前发普通读相同"
    );
    assert_eq!(dedicated.read_ranges.len(), 2, "两段读都进了读区间");
    assert_eq!(dedicated.fired_ordinals, vec![2], "注入落在整池第 2 次读上");
    assert_eq!(
        dedicated.bytes_read[1][0],
        REREAD_BYTE ^ 1,
        "第 2 次读被翻了一位"
    );
}

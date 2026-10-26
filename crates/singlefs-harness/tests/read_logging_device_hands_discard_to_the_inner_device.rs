//! 读日志包装（`singlefs_harness::read_tally::ReadLoggingBlockDevice`）把 discard 原样转给里面那块盘、交回它的结局与错误
//! （`singlefs_core::block_device::BlockDevice::discard_at` 的文档：包在别的设备外面的包装要转给里面那块盘，不覆盖就把里面那块盘认得的
//! discard 吞成「不支持」；`records/2026-10-24-singlefs-harness-恢复验证审计.md`「H01：读日志包装器吞掉底层支持的 discard」）。
//!
//! 每条用例拿同一个请求各发一次：直接发给里面那块盘、经读日志包装发；两边交回的结局（含错误）逐字相同，里面那块盘收到的 discard 次数相同。
//! 里面那块盘是一块 1 MiB 的稀疏内存盘外面套一层数 discard 的测试设备：认 discard 的那一种照内存盘的判据核请求、整段清零，
//! 另两种只答「不支持」或报底层 I/O 错。

use std::cell::Cell;
use std::rc::Rc;

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
};
use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
use singlefs_harness::memory_pool::SparseBlockDevice;
use singlefs_harness::read_tally::{JournalHintAnswer, JournalRingRegion, ReadLoggingBlockDevice};

const DEVICE_BYTES: u64 = 1 << 20;
const PHYSICAL_BLOCK_BYTES: u32 = 512;
const PHYSICAL_BLOCK_LENGTH: u64 = 512;
const BYTE_WRITTEN_BEFORE_THE_DISCARD: u8 = 0x5a;

/// 里面那块盘收到 discard 之后怎么答。
#[derive(Clone, Copy, Debug)]
enum InnerDiscardAnswer {
    /// 交给稀疏内存盘：照它的判据核请求，合法的整段清零、答「交给了底层」。
    PassToTheSparseMemoryDisk,
    /// 不认 discard：一个字节不动，答「不支持」。
    AnswerUnsupported,
    /// 底层报 I/O 错。
    FailWithAnInputOutputError,
}

/// 数 discard 的测试设备：读写清零屏障都交给里面的稀疏内存盘，discard 按 [`InnerDiscardAnswer`] 答、每次都数。
struct DiscardCountingDevice {
    sparse_memory_disk: SparseBlockDevice,
    answer: InnerDiscardAnswer,
    discard_calls_received: Rc<Cell<u64>>,
}

impl BlockDevice for DiscardCountingDevice {
    fn read_at(
        &self,
        offset: DeviceOffsetInBytes,
        buffer: &mut [u8],
    ) -> Result<(), BlockDeviceError> {
        self.sparse_memory_disk.read_at(offset, buffer)
    }

    fn write_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        bytes: &[u8],
        durability: WriteDurability,
    ) -> Result<(), BlockDeviceError> {
        self.sparse_memory_disk.write_at(offset, bytes, durability)
    }

    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.sparse_memory_disk.write_zeroes_at(offset, length)
    }

    fn discard_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<DiscardOutcome, BlockDeviceError> {
        self.discard_calls_received
            .set(self.discard_calls_received.get() + 1);
        match self.answer {
            InnerDiscardAnswer::PassToTheSparseMemoryDisk => {
                self.sparse_memory_disk.discard_at(offset, length)
            }
            InnerDiscardAnswer::AnswerUnsupported => Ok(DiscardOutcome::Unsupported),
            InnerDiscardAnswer::FailWithAnInputOutputError => Err(BlockDeviceError::InputOutput(
                std::io::Error::other("里面那块盘的 discard 报底层 I/O 错"),
            )),
        }
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.sparse_memory_disk.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.sparse_memory_disk.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.sparse_memory_disk.size_in_bytes()
    }
}

/// 一次 discard 看得见的两样：交回的结局（`Debug` 写成字，错误也逐字比），里面那块盘收到了几次 discard。
#[derive(Debug, PartialEq, Eq)]
struct DiscardSeen {
    result: String,
    inner_discard_calls: u64,
}

fn fresh_inner_device(answer: InnerDiscardAnswer) -> (DiscardCountingDevice, Rc<Cell<u64>>) {
    let discard_calls_received = Rc::new(Cell::new(0));
    (
        DiscardCountingDevice {
            sparse_memory_disk: SparseBlockDevice::new(
                DEVICE_BYTES,
                PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
            ),
            answer,
            discard_calls_received: Rc::clone(&discard_calls_received),
        },
        discard_calls_received,
    )
}

fn wrapped_in_the_read_logging_device(
    inner: DiscardCountingDevice,
) -> ReadLoggingBlockDevice<DiscardCountingDevice> {
    ReadLoggingBlockDevice::new(
        inner,
        JournalRingRegion::starting_at_the_standard_slot(JOURNAL_RING_DEFAULT_BYTES),
        JournalHintAnswer::PassTheInnerDeviceAnswerThrough,
    )
}

fn discard_once_on<Device: BlockDevice>(
    device: &mut Device,
    offset: u64,
    length: u64,
    discard_calls_received: &Cell<u64>,
) -> DiscardSeen {
    let result = device.discard_at(DeviceOffsetInBytes(offset), length);
    DiscardSeen {
        result: format!("{result:?}"),
        inner_discard_calls: discard_calls_received.get(),
    }
}

/// 同一个请求直接发给里面那块盘与经读日志包装发各一次（各用一块新盘），交回两边看到的。
fn discard_directly_and_through_the_read_logging_device(
    answer: InnerDiscardAnswer,
    offset: u64,
    length: u64,
) -> (DiscardSeen, DiscardSeen) {
    let (mut direct_inner, direct_calls) = fresh_inner_device(answer);
    let direct = discard_once_on(&mut direct_inner, offset, length, &direct_calls);
    let (wrapped_inner, wrapped_calls) = fresh_inner_device(answer);
    let mut wrapped = wrapped_in_the_read_logging_device(wrapped_inner);
    let through_the_wrapper = discard_once_on(&mut wrapped, offset, length, &wrapped_calls);
    (direct, through_the_wrapper)
}

#[test]
fn read_logging_device_hands_back_applied_when_the_inner_device_applies_the_discard() {
    let (direct, through_the_wrapper) = discard_directly_and_through_the_read_logging_device(
        InnerDiscardAnswer::PassToTheSparseMemoryDisk,
        0,
        PHYSICAL_BLOCK_LENGTH,
    );
    assert_eq!(
        direct,
        DiscardSeen {
            result: format!("{:?}", Ok::<DiscardOutcome, ()>(DiscardOutcome::Applied)),
            inner_discard_calls: 1,
        },
        "直接发：认 discard 的内存盘答「交给了底层」、收到一次"
    );
    assert_eq!(
        through_the_wrapper, direct,
        "经读日志包装发：结局与里面那块盘收到的次数都与直接发相同（不吞成「不支持」）"
    );
}

#[test]
fn read_logging_device_hands_back_unsupported_when_the_inner_device_does_not_support_discard() {
    let (direct, through_the_wrapper) = discard_directly_and_through_the_read_logging_device(
        InnerDiscardAnswer::AnswerUnsupported,
        0,
        PHYSICAL_BLOCK_LENGTH,
    );
    assert_eq!(
        direct,
        DiscardSeen {
            result: format!(
                "{:?}",
                Ok::<DiscardOutcome, ()>(DiscardOutcome::Unsupported)
            ),
            inner_discard_calls: 1,
        },
        "直接发：不认 discard 的盘答「不支持」、收到一次"
    );
    assert_eq!(
        through_the_wrapper, direct,
        "经读日志包装发：「不支持」是里面那块盘答的，它也收到了一次"
    );
}

#[test]
fn read_logging_device_hands_back_the_input_output_error_of_the_inner_device() {
    let (direct, through_the_wrapper) = discard_directly_and_through_the_read_logging_device(
        InnerDiscardAnswer::FailWithAnInputOutputError,
        0,
        PHYSICAL_BLOCK_LENGTH,
    );
    assert!(
        direct.result.starts_with("Err(InputOutput("),
        "直接发：里面那块盘报底层 I/O 错，实际 {}",
        direct.result
    );
    assert_eq!(direct.inner_discard_calls, 1, "直接发：里面那块盘收到一次");
    assert_eq!(
        through_the_wrapper, direct,
        "经读日志包装发：交回里面那块盘的同一个错误"
    );
}

#[test]
fn read_logging_device_hands_a_zero_length_discard_to_the_inner_device() {
    let (direct, through_the_wrapper) = discard_directly_and_through_the_read_logging_device(
        InnerDiscardAnswer::PassToTheSparseMemoryDisk,
        PHYSICAL_BLOCK_LENGTH,
        0,
    );
    assert_eq!(
        direct,
        DiscardSeen {
            result: format!("{:?}", Ok::<DiscardOutcome, ()>(DiscardOutcome::Applied)),
            inner_discard_calls: 1,
        },
        "直接发：长度 0 核过请求、没有动作，答「交给了底层」"
    );
    assert_eq!(
        through_the_wrapper, direct,
        "经读日志包装发：长度 0 也交给里面那块盘核，结局相同"
    );
}

#[test]
fn read_logging_device_hands_back_the_unaligned_refusal_of_the_inner_device() {
    let (direct, through_the_wrapper) = discard_directly_and_through_the_read_logging_device(
        InnerDiscardAnswer::PassToTheSparseMemoryDisk,
        PHYSICAL_BLOCK_LENGTH / 2,
        PHYSICAL_BLOCK_LENGTH,
    );
    assert!(
        direct.result.starts_with("Err(Unaligned"),
        "直接发：偏移没按物理块对齐，实际 {}",
        direct.result
    );
    assert_eq!(direct.inner_discard_calls, 1, "直接发：里面那块盘收到一次");
    assert_eq!(
        through_the_wrapper, direct,
        "经读日志包装发：没对齐由里面那块盘核、交回同一个错误"
    );
}

#[test]
fn read_logging_device_hands_back_the_out_of_range_refusal_of_the_inner_device() {
    let (direct, through_the_wrapper) = discard_directly_and_through_the_read_logging_device(
        InnerDiscardAnswer::PassToTheSparseMemoryDisk,
        DEVICE_BYTES,
        PHYSICAL_BLOCK_LENGTH,
    );
    assert!(
        direct.result.starts_with("Err(OutOfRange"),
        "直接发：请求越过设备末尾，实际 {}",
        direct.result
    );
    assert_eq!(direct.inner_discard_calls, 1, "直接发：里面那块盘收到一次");
    assert_eq!(
        through_the_wrapper, direct,
        "经读日志包装发：越界由里面那块盘核、交回同一个错误"
    );
}

/// 同一块预先写了 0x5a 的稀疏内存盘，直接与经读日志包装各 discard 偏移 0 长度 512：之后读回的字节两边相同
/// （内存盘把 discard 当整段清零应用，读回全 0；包装吞成「不支持」的话那一段仍是 0x5a）。
#[test]
fn read_logging_device_leaves_the_sparse_memory_disk_with_the_bytes_a_direct_discard_leaves() {
    let written = vec![BYTE_WRITTEN_BEFORE_THE_DISCARD; 512];
    let mut direct =
        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES));
    direct
        .write_at(DeviceOffsetInBytes(0), &written, WriteDurability::Plain)
        .expect("直接写 0x5a");
    let direct_outcome = direct
        .discard_at(DeviceOffsetInBytes(0), PHYSICAL_BLOCK_LENGTH)
        .expect("直接 discard");
    let mut direct_read_back = vec![0xffu8; 512];
    direct
        .read_at(DeviceOffsetInBytes(0), &mut direct_read_back)
        .expect("直接读回");

    let mut wrapped = ReadLoggingBlockDevice::new(
        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES)),
        JournalRingRegion::starting_at_the_standard_slot(JOURNAL_RING_DEFAULT_BYTES),
        JournalHintAnswer::PassTheInnerDeviceAnswerThrough,
    );
    wrapped
        .write_at(DeviceOffsetInBytes(0), &written, WriteDurability::Plain)
        .expect("经包装写 0x5a");
    let wrapped_outcome = wrapped
        .discard_at(DeviceOffsetInBytes(0), PHYSICAL_BLOCK_LENGTH)
        .expect("经包装 discard");
    let mut wrapped_read_back = vec![0xffu8; 512];
    wrapped
        .read_at(DeviceOffsetInBytes(0), &mut wrapped_read_back)
        .expect("经包装读回");

    assert_eq!(direct_outcome, DiscardOutcome::Applied, "内存盘认 discard");
    assert!(
        direct_read_back.iter().all(|byte| *byte == 0),
        "内存盘把 discard 当整段清零应用"
    );
    assert_eq!(
        wrapped_outcome, direct_outcome,
        "经读日志包装的结局与直接发相同"
    );
    assert_eq!(
        wrapped_read_back, direct_read_back,
        "经读日志包装 discard 之后读回的字节与直接发相同（不是仍为 0x5a）"
    );
}

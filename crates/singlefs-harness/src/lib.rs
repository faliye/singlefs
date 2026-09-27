//! 录制器：包在块设备外面，把每个写请求与屏障记成一条流，落到文件给崩溃点重放用（里程碑步 0）。
//!
//! 每条记：设备身份、偏移、长度、内容哈希、种类（普通写 / FUA 写 / 屏障）。读不记——崩溃状态只由写与屏障切段（D13（验证路线） 已定项 4）。
#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::fmt::Write as _;
use std::path::Path;
use std::rc::Rc;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::block_device::{
    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
};

pub mod bad_disk_input;
pub mod crash;
pub mod crash_injection;
pub mod device_log;
pub mod fault_injection;
pub mod first_transaction_regions;
pub mod hexadecimal;
pub mod history;
pub mod layer0_progress;
pub mod model;
pub mod model_comparison;
pub mod on_device_modes;
pub mod read_tally;
pub mod scenario;
pub mod segments;
pub mod sha256;

/// 一条录制流里能出现的操作种类，封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordedOperationKind {
    Write,
    WriteForceUnitAccess,
    /// 整段清零：一次 [`BlockDevice::write_zeroes_at`] 记成**一步**，不是底下拆出来的那几次写
    /// （用户 2026-09-19 定案「录制流登记成一种新步骤、层 0 认它」）。内容由 `offset` 与 `length` 全定：整段全 0。
    WriteZeroes,
    Barrier,
}

/// 录制流的一条。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedOperation {
    pub device: DeviceIdentity,
    pub kind: RecordedOperationKind,
    /// 屏障没有偏移与长度，记 0。
    pub offset: DeviceOffsetInBytes,
    pub length: u64,
    /// 写内容的 FNV-1a 64 位哈希；屏障记 0。只用来认出「同一份内容」，不承担完整性。
    pub content_hash: u64,
}

impl RecordedOperation {
    /// 一行一条，机器能重新解析；字段顺序固定。
    #[must_use]
    pub fn to_stream_line(&self) -> String {
        let kind = match self.kind {
            RecordedOperationKind::Write => "write",
            RecordedOperationKind::WriteForceUnitAccess => "write_fua",
            RecordedOperationKind::WriteZeroes => "write_zeroes",
            RecordedOperationKind::Barrier => "barrier",
        };
        format!(
            "device={} kind={kind} offset={} length={} hash={:016x}",
            self.device.0, self.offset.0, self.length, self.content_hash
        )
    }
}

/// FNV-1a 64 位；本地实现，步 0 不引第三方 crate。
#[must_use]
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = FNV1A_64_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV1A_64_PRIME);
    }
    hash
}

const FNV1A_64_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV1A_64_PRIME: u64 = 0x0000_0100_0000_01b3;

/// `length` 个 0 字节的 FNV-1a 64 位哈希，不把那几百 MiB 全 0 真的摆出来。
///
/// 每吃一个 0 字节，`hash ^= 0` 什么都不做，只剩 `hash *= prime`，所以整串的哈希是
/// `offset_basis * prime^length`（模 2^64）。用平方乘算 `prime^length`，与
/// [`fnv1a_64`] 对同一串全 0 的结果逐位相同（本模块用例 `zero_hash_shortcut_agrees_with_hashing_real_zeros` 钉住）。
#[must_use]
pub fn fnv1a_64_of_zeros(length: u64) -> u64 {
    let mut result: u64 = FNV1A_64_OFFSET_BASIS;
    let mut factor: u64 = FNV1A_64_PRIME;
    let mut remaining = length;
    while remaining > 0 {
        if remaining % 2 == 1 {
            result = result.wrapping_mul(factor);
        }
        factor = factor.wrapping_mul(factor);
        remaining /= 2;
    }
    result
}

/// 一条录制流里的一步，带上写的内容（只在开了内容保留的流里有；屏障没有内容）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedOperation {
    pub operation: RecordedOperation,
    pub contents: Option<Vec<u8>>,
}

/// 录制流里一段操作是从哪个入口发出来的（C557（卸载记号只由卸载入口打没有检查）：录制流要记每次发布的入口）。
/// 块设备一层看不见入口，由驱动这条流的一方在调入口时记（[`SharedStream::record_entry`]）。
/// 只登记要核的那一个：正常卸载那一串；没记下入口的操作一律按「不是卸载入口发的」算。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordedPublishEntry {
    /// `singlefs_core::mount::unmount` 发的那一串：先写系统配置那一步与那几次带卸载记号的空发布。
    Unmount,
}

/// 录制流里一个入口发出的那一段：`operations` 是录制流下标的半开区间 [起, 止)（与 [`SharedStream::operations`] 同一套下标）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedEntrySpan {
    pub entry: RecordedPublishEntry,
    pub operations: std::ops::Range<usize>,
}

#[derive(Default)]
struct StreamState {
    operations: Vec<RecordedOperation>,
    contents: Vec<Option<Vec<u8>>>,
    retain_contents: bool,
    entry_spans: Vec<RecordedEntrySpan>,
}

/// 一个池里几块设备共用的录制流：按发出次序记，谁先发谁在前。
/// 屏障按设备记（代码审阅第 1 条，用户 2026-09-27 定「按设备记屏障」）：一块盘的屏障只让这块盘上它之前的写持久，
/// 不同设备的屏障各记一步，不并成一道池屏障——并了，一块盘漏发屏障时录下来的流与每盘都发的逐步相同，切段看不出来。
/// 只有同一块盘上连着的几道（一串连着的屏障里这块盘已经记过一道）记成一道：它们之间这块盘一个写都没有。
/// 开了内容保留的流把每次写的字节也留下来，崩溃点重放拿它在内存里重建镜像（步 7）。
#[derive(Clone, Default)]
pub struct SharedStream(Rc<RefCell<StreamState>>);

impl SharedStream {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn retaining_contents() -> Self {
        Self(Rc::new(RefCell::new(StreamState {
            operations: Vec::new(),
            contents: Vec::new(),
            retain_contents: true,
            entry_spans: Vec::new(),
        })))
    }

    /// 调 `run`（驱动这条流的一方调一个入口），把它期间录下的操作记成 `entry` 发的一段（[`RecordedEntrySpan`]）。
    /// `run` 报错也照记：报错之前已经发出去的写同样是这个入口发的。
    pub fn record_entry<Output>(
        &self,
        entry: RecordedPublishEntry,
        run: impl FnOnce() -> Output,
    ) -> Output {
        let start = self.operation_count();
        let output = run();
        let end = self.operation_count();
        self.0.borrow_mut().entry_spans.push(RecordedEntrySpan {
            entry,
            operations: start..end,
        });
        output
    }

    /// 记下来的每一段入口，按记的先后。
    #[must_use]
    pub fn entry_spans(&self) -> Vec<RecordedEntrySpan> {
        self.0.borrow().entry_spans.clone()
    }
    fn push(&self, operation: RecordedOperation, contents: Option<&[u8]>) {
        let mut state = self.0.borrow_mut();
        let this_device_already_barriered_in_the_trailing_run_of_barriers = state
            .operations
            .iter()
            .rev()
            .take_while(|previous| previous.kind == RecordedOperationKind::Barrier)
            .any(|previous| previous.device == operation.device);
        if operation.kind == RecordedOperationKind::Barrier
            && this_device_already_barriered_in_the_trailing_run_of_barriers
        {
            return;
        }
        let retained = if state.retain_contents {
            contents.map(<[u8]>::to_vec)
        } else {
            None
        };
        state.operations.push(operation);
        state.contents.push(retained);
    }
    #[must_use]
    pub fn operations(&self) -> Vec<RecordedOperation> {
        self.0.borrow().operations.clone()
    }
    /// 流里已有几步（不拷整条流）：随机历史拿它判「这一步发没发写」。
    #[must_use]
    pub fn operation_count(&self) -> usize {
        self.0.borrow().operations.len()
    }
    /// 每一步连同它的内容；没开内容保留的流里 `contents` 全是 None。
    #[must_use]
    pub fn retained_operations(&self) -> Vec<RetainedOperation> {
        let state = self.0.borrow();
        state
            .operations
            .iter()
            .zip(&state.contents)
            .map(|(operation, contents)| RetainedOperation {
                operation: operation.clone(),
                contents: contents.clone(),
            })
            .collect()
    }
    #[must_use]
    pub fn render(&self) -> String {
        render_stream(&self.0.borrow().operations)
    }
}

/// 包在一块设备外面的录制器；几块盘可以共用一条 [`SharedStream`]。
pub struct RecordingBlockDevice<Inner: BlockDevice> {
    inner: Inner,
    device: DeviceIdentity,
    stream: SharedStream,
}

impl<Inner: BlockDevice> RecordingBlockDevice<Inner> {
    pub fn new(device: DeviceIdentity, inner: Inner) -> Self {
        Self {
            inner,
            device,
            stream: SharedStream::new(),
        }
    }

    pub fn with_shared_stream(device: DeviceIdentity, inner: Inner, stream: SharedStream) -> Self {
        Self {
            inner,
            device,
            stream,
        }
    }

    #[must_use]
    pub fn recorded_operations(&self) -> Vec<RecordedOperation> {
        self.stream.operations()
    }

    #[must_use]
    pub fn into_inner_and_operations(self) -> (Inner, Vec<RecordedOperation>) {
        let operations = self.stream.operations();
        (self.inner, operations)
    }

    #[must_use]
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    /// 故障注入要在两次发布之间改里面那块设备的状态（虚机档的「漏一道屏障」）。
    pub fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }

    /// 整条流写成文本，末行报总条数：抓取方比对条数，对不上整轮作废（`command-safety.md` 的完整性闸）。
    #[must_use]
    pub fn render_stream(&self) -> String {
        self.stream.render()
    }

    pub fn write_stream_to(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.render_stream())
    }
}

/// 把一批操作渲染成流文本，末行 `count=N`。
#[must_use]
pub fn render_stream(operations: &[RecordedOperation]) -> String {
    let mut text = String::new();
    for operation in operations {
        text.push_str(&operation.to_stream_line());
        text.push('\n');
    }
    let _ = writeln!(text, "count={}", operations.len());
    text
}

/// 流文本的完整性闸：末行声明的条数与实际行数对不上就拒绝。
#[derive(Debug, PartialEq, Eq)]
pub enum StreamIntegrity {
    Consistent { operations: usize },
    Inconsistent { declared: usize, actual: usize },
    MissingCountLine,
}

#[must_use]
pub fn check_stream_integrity(stream_text: &str) -> StreamIntegrity {
    let lines: Vec<&str> = stream_text.lines().collect();
    let Some(last) = lines.last() else {
        return StreamIntegrity::MissingCountLine;
    };
    let Some(declared_text) = last.strip_prefix("count=") else {
        return StreamIntegrity::MissingCountLine;
    };
    let Ok(declared) = declared_text.parse::<usize>() else {
        return StreamIntegrity::MissingCountLine;
    };
    let actual = lines.len() - 1;
    if declared == actual {
        StreamIntegrity::Consistent { operations: actual }
    } else {
        StreamIntegrity::Inconsistent { declared, actual }
    }
}

impl<Inner: BlockDevice> BlockDevice for RecordingBlockDevice<Inner> {
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
        self.inner.write_at(offset, bytes, durability)?;
        let kind = match durability {
            WriteDurability::Plain => RecordedOperationKind::Write,
            WriteDurability::ForceUnitAccess => RecordedOperationKind::WriteForceUnitAccess,
        };
        self.stream.push(
            RecordedOperation {
                device: self.device,
                kind,
                offset,
                length: u64::try_from(bytes.len()).expect("写长度装得进 u64"),
                content_hash: fnv1a_64(bytes),
            },
            Some(bytes),
        );
        Ok(())
    }

    /// 整段清零记成**一步**（`RecordedOperationKind::WriteZeroes`）：底下拆成几次 I/O 是后端的事，上层看到的是一个动作。
    ///
    /// 内容不留（`contents: None`），即使这条流开了内容保留：整段全 0 由 `offset` 与 `length` 全定，
    /// 而 mkfs 一次清 768 MiB，留下来就是每条流多背 768 MiB 的 0。要重放它的人按长度铺 0
    /// （[`crash::SparseDevice::zero_fill`]），不从 `contents` 里拿。
    fn write_zeroes_at(
        &mut self,
        offset: DeviceOffsetInBytes,
        length: u64,
    ) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)?;
        self.stream.push(
            RecordedOperation {
                device: self.device,
                kind: RecordedOperationKind::WriteZeroes,
                offset,
                length,
                content_hash: fnv1a_64_of_zeros(length),
            },
            None,
        );
        Ok(())
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()?;
        self.stream.push(
            RecordedOperation {
                device: self.device,
                kind: RecordedOperationKind::Barrier,
                offset: DeviceOffsetInBytes(0),
                length: 0,
                content_hash: 0,
            },
            None,
        );
        Ok(())
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 内存里的假设备：正常那一种，每个写都做。
    struct MemoryDevice {
        bytes: Vec<u8>,
    }

    impl BlockDevice for MemoryDevice {
        fn read_at(
            &self,
            offset: DeviceOffsetInBytes,
            buffer: &mut [u8],
        ) -> Result<(), BlockDeviceError> {
            let start = usize::try_from(offset.0).expect("测试偏移装得进 usize");
            buffer.copy_from_slice(&self.bytes[start..start + buffer.len()]);
            Ok(())
        }
        fn write_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            bytes: &[u8],
            _durability: WriteDurability,
        ) -> Result<(), BlockDeviceError> {
            let start = usize::try_from(offset.0).expect("测试偏移装得进 usize");
            self.bytes[start..start + bytes.len()].copy_from_slice(bytes);
            Ok(())
        }
        fn write_zeroes_at(
            &mut self,
            offset: DeviceOffsetInBytes,
            length: u64,
        ) -> Result<(), BlockDeviceError> {
            let start = usize::try_from(offset.0).expect("测试偏移装得进 usize");
            let end = start + usize::try_from(length).expect("测试长度装得进 usize");
            self.bytes[start..end].fill(0);
            Ok(())
        }
        fn barrier(&mut self) -> Result<(), BlockDeviceError> {
            Ok(())
        }
        fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
            PhysicalBlockSizeInBytes(512)
        }
        fn size_in_bytes(&self) -> u64 {
            u64::try_from(self.bytes.len()).expect("测试镜像装得进 u64")
        }
    }

    fn drive_three_writes_and_a_barrier<Device: BlockDevice>(device: &mut Device) {
        device
            .write_at(DeviceOffsetInBytes(0), &[1u8; 512], WriteDurability::Plain)
            .expect("写 1");
        device
            .write_at(
                DeviceOffsetInBytes(512),
                &[2u8; 512],
                WriteDurability::Plain,
            )
            .expect("写 2");
        device.barrier().expect("屏障");
        device
            .write_at(
                DeviceOffsetInBytes(1024),
                &[3u8; 512],
                WriteDurability::ForceUnitAccess,
            )
            .expect("FUA 写");
    }

    #[test]
    fn recorder_keeps_every_write_and_barrier_in_issue_order_with_kinds() {
        let mut recorder = RecordingBlockDevice::new(
            DeviceIdentity(0),
            MemoryDevice {
                bytes: vec![0; 4096],
            },
        );
        drive_three_writes_and_a_barrier(&mut recorder);
        let operations = recorder.recorded_operations();
        let kinds: Vec<RecordedOperationKind> =
            operations.iter().map(|operation| operation.kind).collect();
        assert_eq!(
            kinds,
            [
                RecordedOperationKind::Write,
                RecordedOperationKind::Write,
                RecordedOperationKind::Barrier,
                RecordedOperationKind::WriteForceUnitAccess
            ]
        );
        assert_eq!(operations[1].offset, DeviceOffsetInBytes(512));
        assert_eq!(operations[1].content_hash, fnv1a_64(&[2u8; 512]));
        assert_eq!(operations[2].length, 0, "屏障没有长度");
        let mut read_back = [0u8; 512];
        recorder
            .read_at(DeviceOffsetInBytes(1024), &mut read_back)
            .expect("读");
        assert_eq!(read_back, [3u8; 512], "录制器把写透传给了里面的设备");
    }

    /// 录制器自证：喂一个少发一条写的假流，条数闸必须判红；正常的流判绿。
    #[test]
    fn stream_count_gate_rejects_a_stream_that_lost_one_write() {
        let mut recorder = RecordingBlockDevice::new(
            DeviceIdentity(1),
            MemoryDevice {
                bytes: vec![0; 4096],
            },
        );
        drive_three_writes_and_a_barrier(&mut recorder);
        let complete = recorder.render_stream();
        assert_eq!(
            check_stream_integrity(&complete),
            StreamIntegrity::Consistent { operations: 4 }
        );

        let (_, mut operations) = recorder.into_inner_and_operations();
        let lost_one = operations.remove(1);
        assert_eq!(lost_one.kind, RecordedOperationKind::Write);
        let declared_four = format!(
            "{}count=4\n",
            render_stream(&operations).trim_end_matches("count=3\n")
        );
        assert_eq!(
            check_stream_integrity(&declared_four),
            StreamIntegrity::Inconsistent {
                declared: 4,
                actual: 3
            }
        );
        assert_eq!(
            check_stream_integrity("device=0 kind=write offset=0 length=512 hash=0\n"),
            StreamIntegrity::MissingCountLine
        );
    }

    #[test]
    fn stream_lines_are_stable_text() {
        let operation = RecordedOperation {
            device: DeviceIdentity(1),
            kind: RecordedOperationKind::WriteForceUnitAccess,
            offset: DeviceOffsetInBytes(4096),
            length: 512,
            content_hash: 0x1234,
        };
        assert_eq!(
            operation.to_stream_line(),
            "device=1 kind=write_fua offset=4096 length=512 hash=0000000000001234"
        );
        assert_eq!(fnv1a_64(b""), 0xcbf2_9ce4_8422_2325, "FNV-1a 64 的空串值");
    }

    /// 整段清零记一步：录制流多的是一条 `write_zeroes`，不是底下拆出来的那几次写；
    /// 内层设备真的被清了；开了内容保留的流也不为它留字节（768 MiB 的 0 不进内存）。
    #[test]
    fn zero_fill_is_recorded_as_one_step_and_keeps_no_contents() {
        let stream = SharedStream::retaining_contents();
        let mut recorder = RecordingBlockDevice::with_shared_stream(
            DeviceIdentity(0),
            MemoryDevice {
                bytes: vec![0xD7; 4096],
            },
            stream.clone(),
        );
        recorder
            .write_at(DeviceOffsetInBytes(0), &[1u8; 512], WriteDurability::Plain)
            .expect("先写一扇区");
        recorder
            .write_zeroes_at(DeviceOffsetInBytes(512), 2048)
            .expect("写零");
        let operations = recorder.recorded_operations();
        assert_eq!(
            operations.len(),
            2,
            "一次写 + 一次清零 = 两步，清零不按块拆成多条"
        );
        assert_eq!(operations[1].kind, RecordedOperationKind::WriteZeroes);
        assert_eq!(operations[1].offset, DeviceOffsetInBytes(512));
        assert_eq!(operations[1].length, 2048);
        assert_eq!(operations[1].content_hash, fnv1a_64(&[0u8; 2048]));
        assert_eq!(
            operations[1].to_stream_line(),
            format!(
                "device=0 kind=write_zeroes offset=512 length=2048 hash={:016x}",
                fnv1a_64(&[0u8; 2048])
            )
        );
        let retained = stream.retained_operations();
        assert!(
            retained[1].contents.is_none(),
            "清零不留内容：整段全 0 由偏移与长度全定"
        );
        assert!(
            retained[0].contents.is_some(),
            "普通写照样留内容（这条流开了内容保留）"
        );
        let mut read_back = [0xFFu8; 2048];
        recorder
            .read_at(DeviceOffsetInBytes(512), &mut read_back)
            .expect("读");
        assert!(
            read_back.iter().all(|byte| *byte == 0),
            "内层设备那一段真的清了"
        );
        let mut before = [0u8; 512];
        recorder
            .read_at(DeviceOffsetInBytes(0), &mut before)
            .expect("读清零段之前");
        assert_eq!(before, [1u8; 512], "清零段之外一个字节不动");
    }

    /// 全 0 串的哈希走的是平方乘的捷径，必须与真的把那一串 0 喂给 [`fnv1a_64`] 逐位相同。
    /// 长度取 0、1、2、3、512、4096 与一个不是 2 的幂的奇数长（平方乘的两条臂都要走到）。
    #[test]
    fn zero_hash_shortcut_agrees_with_hashing_real_zeros() {
        for length in [0u64, 1, 2, 3, 512, 4096, 5001] {
            assert_eq!(
                fnv1a_64_of_zeros(length),
                fnv1a_64(&vec![0u8; usize::try_from(length).expect("测试长度")]),
                "长度 {length} 的全 0 串"
            );
        }
    }
}

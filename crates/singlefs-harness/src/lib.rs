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

pub mod crash;
pub mod device_log;
pub mod first_transaction_regions;
pub mod hexadecimal;
pub mod scenario;
pub mod segments;
pub mod sha256;

/// 一条录制流里能出现的操作种类，封闭集合，`match` 不写通配臂。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordedOperationKind {
    Write,
    WriteForceUnitAccess,
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
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// 一条录制流里的一步，带上写的内容（只在开了内容保留的流里有；屏障没有内容）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetainedOperation {
    pub operation: RecordedOperation,
    pub contents: Option<Vec<u8>>,
}

#[derive(Default)]
struct StreamState {
    operations: Vec<RecordedOperation>,
    contents: Vec<Option<Vec<u8>>>,
    retain_contents: bool,
}

/// 一个池里几块设备共用的录制流：按发出次序记，谁先发谁在前。
/// 连续几道屏障之间没有任何写 ⇒ 记成一道池屏障（mkfs 对每块盘各发一道屏障，段序列登记表按池算一道）。
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
        })))
    }
    fn push(&self, operation: RecordedOperation, contents: Option<&[u8]>) {
        let mut state = self.0.borrow_mut();
        let previous_is_barrier = state
            .operations
            .last()
            .is_some_and(|previous| previous.kind == RecordedOperationKind::Barrier);
        if operation.kind == RecordedOperationKind::Barrier && previous_is_barrier {
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
}

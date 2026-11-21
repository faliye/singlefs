//! 稀疏设备、读块的接口、池、写的种类与记录、录制池与崩溃镜像。

use e7_index_bench::{usize_from_u64, usize_from_u64_in_const};
use super::*;

// E142 第十五次跑步④：旧的「量 5 一个区域的比较结果」（`RegionComparison`/`compare_region`，按名字配对、
// 支持 head_and_tail 抽样）被 Q142.1 的新比对器取代，删除理由见文件末尾单测模块同一处的注释。

// ───────────────────────── 设备、录制器、崩溃镜像 ─────────────────────────

/// 稀疏镜像：按 512 字节扇区存，没写过的扇区读出来恒为 0（mkfs「整环写 0」在这个模型里等价于不写）。
#[derive(Clone, Default)]
pub(super) struct SparseDevice {
    pub(super) sectors: BTreeMap<u64, [u8; usize_from_u64_in_const(PHYSICAL_BLOCK_BYTES)]>,
}

impl SparseDevice {
    pub(super) fn read(&self, offset: DeviceOffset, length: usize) -> Vec<u8> {
        assert_eq!(offset.0 % PHYSICAL_BLOCK_BYTES, 0, "读要按扇区对齐");
        assert_eq!(length as u64 % PHYSICAL_BLOCK_BYTES, 0, "读长度要是整扇区");
        let mut out = vec![0u8; length];
        let first_sector = offset.0 / PHYSICAL_BLOCK_BYTES;
        for sector_index in 0..(length as u64 / PHYSICAL_BLOCK_BYTES) {
            if let Some(sector) = self.sectors.get(&(first_sector + sector_index)) {
                let start = usize_from_u64(sector_index * PHYSICAL_BLOCK_BYTES);
                out[start..start + usize_from_u64(PHYSICAL_BLOCK_BYTES)].copy_from_slice(sector);
            }
        }
        out
    }
    pub(super) fn write(&mut self, offset: DeviceOffset, bytes: &[u8]) {
        assert_eq!(offset.0 % PHYSICAL_BLOCK_BYTES, 0, "写要按扇区对齐");
        assert_eq!(bytes.len() as u64 % PHYSICAL_BLOCK_BYTES, 0, "写长度要是整扇区");
        let first_sector = offset.0 / PHYSICAL_BLOCK_BYTES;
        for (sector_index, chunk) in bytes.as_chunks::<{ usize_from_u64_in_const(PHYSICAL_BLOCK_BYTES) }>().0.iter().enumerate() {
            self.sectors.insert(first_sector + sector_index as u64, *chunk);
        }
    }
    fn written_sectors_in(&self, offset: DeviceOffset, length: u64) -> Vec<u64> {
        let first_sector = offset.0 / PHYSICAL_BLOCK_BYTES;
        let end_sector = (offset.0 + length) / PHYSICAL_BLOCK_BYTES;
        self.sectors.range(first_sector..end_sector).map(|(sector, _)| *sector).collect()
    }
}

/// 恢复路径只通过这个 trait 读盘。两个实现：完整的池，与层 0 枚举出的崩溃镜像。
pub(super) trait BlockReader {
    fn device_count(&self) -> usize;
    /// 每盘字节数：分配记录树的读路径要用它定根的格数（D8 已定项 14），同一个池的几块盘同构。
    fn device_bytes(&self) -> u64;
    fn read(&self, device: DeviceIdentity, offset: DeviceOffset, length: usize) -> Vec<u8>;
    /// 扫描环时的优化：范围内有过写入的扇区号。没写过的扇区读出来恒为 0，一条全 0 的记录 magic 必不匹配，
    /// 所以跳过它们与逐扇区读零判 magic 等价。
    fn written_sectors(&self, device: DeviceIdentity, offset: DeviceOffset, length: u64) -> Vec<u64>;
}

#[derive(Clone)]
pub(super) struct Pool {
    pub(super) devices: Vec<SparseDevice>,
    pub(super) device_bytes: u64,
}

impl BlockReader for Pool {
    fn device_count(&self) -> usize {
        self.devices.len()
    }
    fn device_bytes(&self) -> u64 {
        self.device_bytes
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffset, length: usize) -> Vec<u8> {
        self.devices[device.0 as usize].read(offset, length)
    }
    fn written_sectors(&self, device: DeviceIdentity, offset: DeviceOffset, length: u64) -> Vec<u64> {
        self.devices[device.0 as usize].written_sectors_in(offset, length)
    }
}

/// 录制器里一次**写**的步骤种类。D17（实现分层与第三方管道） 已定项 2 把结构等价类定成
/// 「段边界位置 + 每段步骤种类集合」，所以步骤种类要是机器可读的封闭枚举，不能是自由文本标签（C316 ②）。
/// 没有通配臂：以后多一种写步骤，每一处 match 都编译不过。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) enum StepKind {
    /// 单元区里一次整单元写（码 1 / 码 2 / 码 3 都是同一种步骤：一个 16 KiB 或 32 KiB 落点写满）。
    UnitWrite,
    /// journal 环里一条 4 KiB 记录。
    JournalRecord,
    /// 根环槽的一次 FUA 写（D16（发布语义） 已定项 7：只有这一步等落盘才发下一条）。
    RootRecordFua,
    /// 系统配置槽的一次写。
    SystemConfigurationSlot,
    /// mkfs 把每块盘的 [0, 根环基址)、根环三个区域与 journal 环各整段写 0（D22（单元原子性怎么合成） 已定项 8 第 3 条）：
    /// 三者都由 mkfs 自己经块设备的写零动作发出，每块盘上五段（[0, 根环基址) 一段 + 根环三个区域各一段 + journal 环一段）
    /// 各一次调用，区域归属只定根种在哪块盘、不定清哪块盘——两块盘各清全部五段，共 10 步；
    /// 录制流里各记一步整段清零，清零几段之间不加屏障。不是 FUA。
    ZeroFill,
}

impl StepKind {
    /// 发到输出里的名字，lowercase snake_case，一个种类一个。
    pub(super) fn tag(self) -> &'static str {
        match self {
            StepKind::UnitWrite => "unit_write",
            StepKind::JournalRecord => "journal_record",
            StepKind::RootRecordFua => "root_record_fua",
            StepKind::SystemConfigurationSlot => "system_configuration_slot",
            StepKind::ZeroFill => "zero_fill",
        }
    }
    /// FUA 由步骤种类决定，不再是调用点各传各的布尔：系统配置槽写不可能是 FUA，这样它写不出来。
    pub(super) fn is_fua(self) -> bool {
        match self {
            StepKind::RootRecordFua => true,
            StepKind::UnitWrite | StepKind::JournalRecord | StepKind::SystemConfigurationSlot | StepKind::ZeroFill => false,
        }
    }
}

/// 录制到的一步的种类：写请求带着它那四种之一，屏障自己是一种。
/// 段序列说的「步骤种类」是这个字母表——屏障是段边界，它也要有名字。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub(super) enum RecordedStepKind {
    Write(StepKind),
    Barrier,
}

impl RecordedStepKind {
    pub(super) fn of(operation: &RecordedOperation) -> Self {
        match operation {
            RecordedOperation::Write(write) => RecordedStepKind::Write(write.kind),
            RecordedOperation::Barrier(_) => RecordedStepKind::Barrier,
        }
    }
    pub(super) fn tag(self) -> &'static str {
        match self {
            RecordedStepKind::Write(kind) => kind.tag(),
            RecordedStepKind::Barrier => "barrier",
        }
    }
    /// 种类串的规范序（E142 第十九次跑臂 N19 的 ④）：`zero_fill` 排最前，其余照 `unit_write`、`journal_record`、
    /// `root_record_fua`、`system_configuration_slot`、`barrier`。依据是 `.claude/kb/layout/01-first-txn.md` 八
    /// 「mkfs 种根」那一行种类串原文 `[zero_fill×10,unit_write×4,barrier×2]`。只管种类串怎么排，不动 `StepKind` 的派生序。
    pub(super) fn canonical_rank_in_kinds_text(self) -> u8 {
        match self {
            RecordedStepKind::Write(StepKind::ZeroFill) => 0,
            RecordedStepKind::Write(StepKind::UnitWrite) => 1,
            RecordedStepKind::Write(StepKind::JournalRecord) => 2,
            RecordedStepKind::Write(StepKind::RootRecordFua) => 3,
            RecordedStepKind::Write(StepKind::SystemConfigurationSlot) => 4,
            RecordedStepKind::Barrier => 5,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct WriteRequest {
    pub(super) device: DeviceIdentity,
    pub(super) offset: DeviceOffset,
    pub(super) bytes: Vec<u8>,
    pub(super) kind: StepKind,
}

impl WriteRequest {
    pub(super) fn is_fua(&self) -> bool {
        self.kind.is_fua()
    }
}

/// 录下来的一步。屏障带设备号（E142 第十九次跑臂 N19 的 ①，D13（验证路线） 已定项 4「一次写只被它自己那块盘上
/// 之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面」）：一道池屏障按设备号升序每块盘记一步。
#[derive(Clone, Debug)]
pub(super) enum RecordedOperation {
    Write(WriteRequest),
    Barrier(DeviceIdentity),
}

/// 录制器包在池外面：每个写请求与屏障记成一条流，给层 0 枚举用。
pub(super) struct RecordingPool {
    pub(super) pool: Pool,
    pub(super) operations: Vec<RecordedOperation>,
}

impl RecordingPool {
    pub(super) fn write(&mut self, device: DeviceIdentity, offset: DeviceOffset, bytes: &[u8], kind: StepKind) {
        self.pool.devices[device.0 as usize].write(offset, bytes);
        self.operations.push(RecordedOperation::Write(WriteRequest { device, offset, bytes: bytes.to_vec(), kind }));
    }
    /// mkfs 的整段清零（D22（单元原子性怎么合成） 已定项 8 第 3 条）：只记一步操作，不真的把字节灌进
    /// `SparseDevice` 的扇区表。稀疏镜像没写过的扇区恒读 0（`SparseDevice` 的文档），[0, 根环基址)、根环区域与
    /// journal 环这三类清零都发在 mkfs 写任何字节之前、写的本来就是全 0 的字节，语义上跟真写一遍没有分别；journal 环有 768 MiB，
    /// 真按扇区写一遍会让 `mkfs()` 每次调用都往 `BTreeMap` 里插上百万条目——78 条变异 × 50 个单测
    /// 每条都要跑一次 `mkfs()`，会把这个装置拖到不可用，`mutate.sh` 单条变异 120 秒的超时也会被跑穿。
    pub(super) fn record_zero_fill(&mut self, device: DeviceIdentity, offset: DeviceOffset) {
        self.operations.push(RecordedOperation::Write(WriteRequest { device, offset, bytes: Vec::new(), kind: StepKind::ZeroFill }));
    }
    /// 一道池屏障：按设备号升序每块盘记一步，一块盘时恰一步（臂 N19 的 ①）。同一块盘上连着的屏障并成一道
    /// （臂 N19M 的 ⑫，登记 R46；D13（验证路线） 已定项 4 末句「录制流里一道屏障与这块盘上一道屏障之间
    /// 没有任何一次写（别的盘的屏障不算写）时，这一道不记」）：「任何一次写」取任何一块盘上的写。
    pub(super) fn barrier(&mut self) {
        for device_index in 0..self.pool.devices.len() {
            let device = DeviceIdentity(u32::try_from(device_index).expect("设备数"));
            if !self.device_has_a_barrier_since_the_last_write(device) {
                self.operations.push(RecordedOperation::Barrier(device));
            }
        }
    }
    /// 从流尾往回找：先碰到这块盘的屏障 ⇒ 真（它与这一道之间没有写）；先碰到任何一块盘上的写或流头 ⇒ 假。
    /// 别的盘的屏障跳过（它不算写）。
    fn device_has_a_barrier_since_the_last_write(&self, device: DeviceIdentity) -> bool {
        for operation in self.operations.iter().rev() {
            match operation {
                RecordedOperation::Write(_) => return false,
                RecordedOperation::Barrier(recorded_device) if *recorded_device == device => return true,
                RecordedOperation::Barrier(_) => {}
            }
        }
        false
    }
}

/// 一个崩溃状态的镜像：崩溃前的池 + 这次事务的写请求里被判持久的那个子集。没持久的位置读到旧字节（D13 已定项 4）。
pub(super) struct CrashImage<'base> {
    pub(super) base: &'base Pool,
    pub(super) writes: &'base [WriteRequest],
    pub(super) persisted: Vec<bool>,
}

impl BlockReader for CrashImage<'_> {
    fn device_count(&self) -> usize {
        self.base.devices.len()
    }
    fn device_bytes(&self) -> u64 {
        self.base.device_bytes
    }
    fn read(&self, device: DeviceIdentity, offset: DeviceOffset, length: usize) -> Vec<u8> {
        let mut out = self.base.read(device, offset, length);
        let read_start = offset.0;
        let read_end = offset.0 + length as u64;
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if !is_persisted || write.device != device {
                continue;
            }
            let write_start = write.offset.0;
            let write_end = write_start + write.bytes.len() as u64;
            let overlap_start = read_start.max(write_start);
            let overlap_end = read_end.min(write_end);
            if overlap_start >= overlap_end {
                continue;
            }
            let destination = usize_from_u64(overlap_start - read_start);
            let source = usize_from_u64(overlap_start - write_start);
            let overlap_length = usize_from_u64(overlap_end - overlap_start);
            out[destination..destination + overlap_length].copy_from_slice(&write.bytes[source..source + overlap_length]);
        }
        out
    }
    fn written_sectors(&self, device: DeviceIdentity, offset: DeviceOffset, length: u64) -> Vec<u64> {
        let mut sectors = self.base.written_sectors(device, offset, length);
        for (write, is_persisted) in self.writes.iter().zip(&self.persisted) {
            if !is_persisted || write.device != device {
                continue;
            }
            let first = write.offset.0 / PHYSICAL_BLOCK_BYTES;
            let count = write.bytes.len() as u64 / PHYSICAL_BLOCK_BYTES;
            for sector in first..first + count {
                if sector * PHYSICAL_BLOCK_BYTES >= offset.0 && sector * PHYSICAL_BLOCK_BYTES < offset.0 + length {
                    sectors.push(sector);
                }
            }
        }
        sectors.sort_unstable();
        sectors.dedup();
        sectors
    }
}

#[cfg(test)]
#[path = "devices_pools_and_recorded_writes_tests.rs"]
mod tests;

//! 块层读计数：包在 [`PoolReader`] 外面，把每次读按落点与长度分格数（里程碑「第二个事务」并行线二验收第 1 条：
//! 「K 次读期间 journal 环一次都不扫」要能观测，而录制器只录写与屏障、不录读）。
//!
//! 为什么数在这一层：D17（实现分层与第三方管道） 已定项 5 射程 ① 把录制钩子定在块设备接口上，不定在操作 API 上——
//! 实现自己报「这次 API 调用发了几个设备级操作」，装置在块层数，两个数对不上整轮作废。
//! 实现侧那几个数在 `singlefs_core::mounted_read::ReadPathObservation`。

use std::cell::RefCell;

use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
use singlefs_core::recovery::PoolReader;
use singlefs_format::{
    DATA_UNIT_BYTES, JOURNAL_RECORD_BYTES, JOURNAL_RING_START_SLOT, NODE_BYTES, SLOT_BYTES,
};

/// journal 环在一块盘上占的那一段：起点槽 1024 × 16384，长度取系统配置里的声明值
/// （D23（journal 的角色与格式）；`singlefs_core::recovery::scan_journal` 就是从这里逐条读的）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalRingRegion {
    pub start: DeviceOffsetInBytes,
    pub length_in_bytes: u64,
}

impl JournalRingRegion {
    /// 第一版的落点：起点是固定结构区之后的槽 1024，长度由调用方从系统配置里取。
    #[must_use]
    pub const fn starting_at_the_standard_slot(journal_ring_bytes: u64) -> Self {
        Self {
            start: DeviceOffsetInBytes(JOURNAL_RING_START_SLOT * SLOT_BYTES),
            length_in_bytes: journal_ring_bytes,
        }
    }

    /// 这次读的起点落不落在环里。
    #[must_use]
    pub const fn contains(self, offset: DeviceOffsetInBytes) -> bool {
        offset.0 >= self.start.0 && offset.0 < self.start.0 + self.length_in_bytes
    }

    /// 整扫一遍这个环在**一块盘**上是几次读：环字节数 ÷ 记录宽。默认几何 768 MiB ⇒ 196 608 次。
    /// 并行线二验收第 1 条不许读计数落进这一档。
    #[must_use]
    pub const fn full_scan_reads_per_device(self) -> u64 {
        self.length_in_bytes / JOURNAL_RECORD_BYTES
    }
}

/// 块层读的分格计数。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BlockLayerReadTally {
    /// 一共发了几次读（读不回字节的也算：请求已经发出去了）。
    pub reads: u64,
    pub bytes_read: u64,
    /// 起点落在 journal 环里的读。
    pub reads_inside_the_journal_ring: u64,
    /// 整单元宽（32768）的读：数据单元与码 3 打包记录单元。
    pub reads_of_a_whole_unit: u64,
    /// 整节点宽（16384）的读：码 2 索引节点。
    pub reads_of_a_whole_index_node: u64,
}

/// 包在一个 [`PoolReader`] 外面数读。读是 `&self`，所以计数走 [`RefCell`]（单线程用；跨线程的话每片各包一个）。
pub struct ReadCountingPoolReader<'inner, Inner: PoolReader + ?Sized> {
    inner: &'inner Inner,
    journal_ring: JournalRingRegion,
    tally: RefCell<BlockLayerReadTally>,
}

impl<'inner, Inner: PoolReader + ?Sized> ReadCountingPoolReader<'inner, Inner> {
    pub fn new(inner: &'inner Inner, journal_ring: JournalRingRegion) -> Self {
        Self {
            inner,
            journal_ring,
            tally: RefCell::new(BlockLayerReadTally::default()),
        }
    }

    /// 此刻的计数。
    #[must_use]
    pub fn tally(&self) -> BlockLayerReadTally {
        *self.tally.borrow()
    }

    /// 计数清零：打开挂载态花的那几次读与之后每次读要分开数，中间清一次。
    pub fn reset_tally(&self) {
        *self.tally.borrow_mut() = BlockLayerReadTally::default();
    }

    /// 这次跑里整扫一遍 journal 环会是几次读（判定行里要报出这个数，说明计数没落进那一档）。
    #[must_use]
    pub const fn journal_ring_full_scan_reads_per_device(&self) -> u64 {
        self.journal_ring.full_scan_reads_per_device()
    }
}

impl<Inner: PoolReader + ?Sized> PoolReader for ReadCountingPoolReader<'_, Inner> {
    fn device_identities(&self) -> Vec<DeviceIdentity> {
        self.inner.device_identities()
    }

    /// 盘有多大不走读，不计数。
    fn device_size_in_bytes(&self, device: DeviceIdentity) -> Option<u64> {
        self.inner.device_size_in_bytes(device)
    }

    fn read(
        &self,
        device: DeviceIdentity,
        offset: DeviceOffsetInBytes,
        length: usize,
    ) -> Option<Vec<u8>> {
        let length_in_bytes = u64::try_from(length).expect("读长度装得进 u64");
        {
            let mut tally = self.tally.borrow_mut();
            tally.reads += 1;
            tally.bytes_read += length_in_bytes;
            if self.journal_ring.contains(offset) {
                tally.reads_inside_the_journal_ring += 1;
            }
            if length_in_bytes == DATA_UNIT_BYTES {
                tally.reads_of_a_whole_unit += 1;
            }
            if length_in_bytes == NODE_BYTES {
                tally.reads_of_a_whole_index_node += 1;
            }
        }
        self.inner.read(device, offset, length)
    }

    fn journal_record_offsets_hint(
        &self,
        device: DeviceIdentity,
        ring_start: DeviceOffsetInBytes,
        ring_bytes: u64,
    ) -> Option<Vec<DeviceOffsetInBytes>> {
        self.inner
            .journal_record_offsets_hint(device, ring_start, ring_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use singlefs_format::JOURNAL_RING_DEFAULT_BYTES;

    /// 默认几何下整扫一遍环是 196 608 次读：并行线二验收第 1 条不许读计数落进这一档。
    #[test]
    fn a_full_journal_ring_scan_is_one_hundred_ninety_six_thousand_six_hundred_eight_reads() {
        let ring = JournalRingRegion::starting_at_the_standard_slot(JOURNAL_RING_DEFAULT_BYTES);
        assert_eq!(ring.full_scan_reads_per_device(), 196_608);
        assert_eq!(ring.start, DeviceOffsetInBytes(1024 * 16384));
        assert!(ring.contains(ring.start));
        assert!(!ring.contains(DeviceOffsetInBytes(ring.start.0 - 1)));
        assert!(!ring.contains(DeviceOffsetInBytes(
            ring.start.0 + JOURNAL_RING_DEFAULT_BYTES
        )));
    }
}

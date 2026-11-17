//! E207（B8严格判据下的步数）历史用例的紧凑盘：入口把这份拷进副本当 `crates/singlefs-verification/tests/e207_compact_device/mod.rs`，
//! 历史用例 `e207_strict_scan_steps_histories.rs` 经 `#[path]` 引它。只存一次的内存办法，不改 `crates/`。
//!
//! 为什么要它：稀疏盘（`SparseDevice`）每个写过的 512 字节扇区各占一个 `Vec<u8>`，加 `BTreeMap` 的项、堆块头，约 590 字节。
//! 一个数据单元是 32768 字节、两份，写一个单元落 128 个扇区；`files_1000_units_1000` 一百万个单元就是一亿两千八百万个扇区，约 75 GiB 以上
//! （任务 15482 里各片的内存峰值随历史推进，每个单元约 105 KiB；见 `task-server/task status 15488`）。数据单元的载荷由 `content_of` 造，
//! 一个扇区的内容只由（起点下标 × 131 + 种子 × 7 + 1）模 251 定，所以不同的扇区内容至多 251 种；每个单元只有头那一个扇区各不相同。
//! 紧凑盘把单元区里的扇区按内容只存一份，按槽（16 KiB = 32 个扇区）记每个扇区用哪一份；单元区以下（根环、journal 环）原样走稀疏盘。
//!
//! 读写的结果与稀疏盘逐字节相同：写进去什么读回什么，没写过的读出全 0，整段清零与 discard 把记着的扇区删掉。
//! 差别只在一点：紧凑盘不给 `sparse_image`（取镜像、算镜像哈希要整份稀疏盘），所以要取镜像的点（`base`、`files_1000`）不用紧凑盘。
#![allow(dead_code, reason = "历史用例只用到其中一部分，单测另用")]

use singlefs_core::address::DeviceOffsetInBytes;
use singlefs_core::block_device::{BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability};
use singlefs_verification::memory_pool::{SparseBlockDevice, SparseDevice};
use std::collections::{BTreeMap, HashMap};

/// 稀疏盘的扇区宽（`singlefs_verification::memory_pool::SECTOR_BYTES`）。
const SECTOR_BYTES: usize = 512;
/// 一个槽（16 KiB）里有几个扇区。
const SECTORS_PER_SLOT: usize = 32;
/// 同上，按 u64 用（扇区号、字节偏移都是 u64）。
const SECTORS_PER_SLOT_WIDE: u64 = 32;
const SECTOR_BYTES_WIDE: u64 = 512;
/// 没有后继的链尾。
const NO_NEXT: u32 = u32::MAX;

/// 内容相同的扇区只存一份：`sectors` 按序号放内容，`first_with_hash` 与 `next_with_the_same_hash` 串起哈希相同的几份，比字节才算相同。
#[derive(Default)]
struct SectorTable {
    sectors: Vec<Box<[u8]>>,
    first_with_hash: HashMap<u64, u32>,
    next_with_the_same_hash: Vec<u32>,
}

/// 扇区内容的哈希（FNV-1a，按 8 字节一字）：只用来分桶，相同与否照字节比。
fn hash_of(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for word in bytes.chunks_exact(8) {
        hash ^= u64::from_le_bytes(word.try_into().expect("八个字节"));
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        hash ^= hash >> 29;
    }
    hash
}

impl SectorTable {
    /// 这份内容的序号（没存过就存一份）。
    fn intern(&mut self, bytes: &[u8]) -> u32 {
        let hash = hash_of(bytes);
        let mut candidate = self.first_with_hash.get(&hash).copied();
        while let Some(index) = candidate {
            if *self.sectors[usize::try_from(index).expect("序号")] == *bytes {
                return index;
            }
            let next = self.next_with_the_same_hash[usize::try_from(index).expect("序号")];
            candidate = (next != NO_NEXT).then_some(next);
        }
        let index = u32::try_from(self.sectors.len()).expect("扇区内容不会超过四十亿种");
        self.sectors.push(bytes.into());
        self.next_with_the_same_hash.push(self.first_with_hash.insert(hash, index).unwrap_or(NO_NEXT));
        index
    }
}

/// 单元区的存法：槽号 → 这个槽 32 个扇区各用哪一份内容（序号加 1，0 是没写过）。
struct UnitAreaStore {
    first_sector: u64,
    slots: BTreeMap<u64, Box<[u32; SECTORS_PER_SLOT]>>,
    table: SectorTable,
}

impl UnitAreaStore {
    fn write(&mut self, first_sector: u64, bytes: &[u8]) {
        for (position, chunk) in bytes.chunks_exact(SECTOR_BYTES).enumerate() {
            let sector = first_sector + u64::try_from(position).expect("下标");
            let index = self.table.intern(chunk) + 1;
            let slot = self.slots.entry(sector / SECTORS_PER_SLOT_WIDE).or_insert_with(|| Box::new([0; SECTORS_PER_SLOT]));
            slot[usize::try_from(sector % SECTORS_PER_SLOT_WIDE).expect("槽内下标")] = index;
        }
    }

    fn read_into(&self, first_sector: u64, buffer: &mut [u8]) {
        buffer.fill(0);
        for (position, chunk) in buffer.chunks_exact_mut(SECTOR_BYTES).enumerate() {
            let sector = first_sector + u64::try_from(position).expect("下标");
            let Some(slot) = self.slots.get(&(sector / SECTORS_PER_SLOT_WIDE)) else { continue };
            let index = slot[usize::try_from(sector % SECTORS_PER_SLOT_WIDE).expect("槽内下标")];
            if index != 0 {
                chunk.copy_from_slice(&self.table.sectors[usize::try_from(index - 1).expect("序号")]);
            }
        }
    }

    /// 整段清零：记着的扇区删掉（同稀疏盘的 `zero_fill`）；一个槽全空了就把这个槽的项也删掉。
    fn zero(&mut self, first_sector: u64, sectors: u64) {
        let per_slot = SECTORS_PER_SLOT_WIDE;
        let (first_slot, last_slot) = (first_sector / per_slot, (first_sector + sectors).div_ceil(per_slot));
        let touched: Vec<u64> = self.slots.range(first_slot..last_slot).map(|(slot, _)| *slot).collect();
        for slot_number in touched {
            let slot = self.slots.get_mut(&slot_number).expect("刚列出来");
            for within in 0..per_slot {
                let sector = slot_number * per_slot + within;
                if sector >= first_sector && sector < first_sector + sectors {
                    slot[usize::try_from(within).expect("槽内下标")] = 0;
                }
            }
            if slot.iter().all(|index| *index == 0) {
                self.slots.remove(&slot_number);
            }
        }
    }

    fn written_sectors(&self) -> u64 {
        self.slots.values().map(|slot| u64::try_from(slot.iter().filter(|index| **index != 0).count()).expect("个数")).sum()
    }
}

/// 一块盘：单元区以下走稀疏盘，单元区（含）以上走紧凑存法；不给单元区起点就整块走稀疏盘（与 `SparseBlockDevice` 一个字节不差）。
pub struct CompactDevice {
    plain: SparseBlockDevice,
    unit_area: Option<UnitAreaStore>,
}

/// 这块盘存着多少：写过的扇区数、其中不同内容的份数（整块走稀疏盘时份数记 0）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stored {
    pub written_sectors: u64,
    pub distinct_contents: u64,
}

impl CompactDevice {
    /// `unit_area_start_slot` 是单元区的起点槽（`None` = 整块走稀疏盘）。
    pub fn new(plain: SparseBlockDevice, unit_area_start_slot: Option<u64>) -> Self {
        let unit_area = unit_area_start_slot.map(|slot| UnitAreaStore {
            first_sector: slot * SECTORS_PER_SLOT_WIDE,
            slots: BTreeMap::new(),
            table: SectorTable::default(),
        });
        Self { plain, unit_area }
    }

    pub fn stored(&self) -> Stored {
        let below = u64::try_from(self.plain.image.written_sectors().len()).expect("个数");
        match &self.unit_area {
            Some(store) => Stored { written_sectors: below + store.written_sectors(), distinct_contents: u64::try_from(store.table.sectors.len()).expect("个数") },
            None => Stored { written_sectors: below, distinct_contents: 0 },
        }
    }

    /// 一次请求在单元区起点处切成两段：交回（下面那段的字节数，上面那段的起点扇区号）；没有紧凑存法时全算下面。
    fn split(&self, offset: DeviceOffsetInBytes, length: usize) -> (usize, u64) {
        let Some(store) = &self.unit_area else { return (length, 0) };
        let boundary = store.first_sector * SECTOR_BYTES_WIDE;
        let below = if offset.0 >= boundary { 0 } else { usize::try_from(boundary - offset.0).map_or(length, |bytes| bytes.min(length)) };
        let first_above = (offset.0 + u64::try_from(below).expect("字节数")) / SECTOR_BYTES_WIDE;
        (below, first_above)
    }
}

impl BlockDevice for CompactDevice {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let (below, first_above) = self.split(offset, buffer.len());
        if below > 0 {
            self.plain.read_at(offset, &mut buffer[..below])?;
        }
        if below < buffer.len() {
            assert!((buffer.len() - below).is_multiple_of(SECTOR_BYTES), "读长度要是整扇区");
            self.unit_area.as_ref().expect("有上面那段就有紧凑存法").read_into(first_above, &mut buffer[below..]);
        }
        Ok(())
    }

    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        let (below, first_above) = self.split(offset, bytes.len());
        if below > 0 {
            self.plain.write_at(offset, &bytes[..below], durability)?;
        }
        if below < bytes.len() {
            assert!((bytes.len() - below).is_multiple_of(SECTOR_BYTES), "写长度要是整扇区");
            self.unit_area.as_mut().expect("有上面那段就有紧凑存法").write(first_above, &bytes[below..]);
        }
        Ok(())
    }

    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        let total = usize::try_from(length).expect("长度装得进 usize");
        let (below, first_above) = self.split(offset, total);
        if below > 0 {
            self.plain.write_zeroes_at(offset, u64::try_from(below).expect("字节数"))?;
        }
        if below < total {
            self.unit_area.as_mut().expect("有上面那段就有紧凑存法").zero(first_above, u64::try_from(total - below).expect("字节数") / SECTOR_BYTES_WIDE);
        }
        Ok(())
    }

    fn discard_at(&mut self, discarded_offset: DeviceOffsetInBytes, discarded_length: u64) -> Result<DiscardOutcome, BlockDeviceError> {
        self.write_zeroes_at(discarded_offset, discarded_length)?;
        Ok(DiscardOutcome::Applied)
    }

    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.plain.barrier()
    }

    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.plain.probe_physical_block_size()
    }

    fn size_in_bytes(&self) -> u64 {
        self.plain.size_in_bytes()
    }

    fn journal_record_offsets_hint(&self, ring_start: DeviceOffsetInBytes, ring_bytes: u64) -> Option<Vec<DeviceOffsetInBytes>> {
        self.plain.journal_record_offsets_hint(ring_start, ring_bytes)
    }
}

impl CompactDevice {
    /// 取镜像要整份稀疏盘：只在整块走稀疏盘的点上做（`unit_area` 是 `None`）；走了紧凑存法还来取，当场停，不交一份缺了单元区的镜像。
    pub fn sparse_image(&self) -> &SparseDevice {
        assert!(self.unit_area.is_none(), "紧凑盘不交镜像：要取镜像的点不该用紧凑盘（历史用例的 IMAGE_POINTS 与 PER_OPERATION_IMAGE_POINTS）");
        &self.plain.image
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEVICE_BYTES: u64 = 64 << 20;
    const UNIT_AREA_START_SLOT: u64 = 4;

    /// 两块盘做同一串操作：整块稀疏盘与紧凑盘，之后整盘逐扇区读回比。
    struct Pair {
        sparse: SparseBlockDevice,
        compact: CompactDevice,
    }

    impl Pair {
        fn new() -> Self {
            Self {
                sparse: SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
                compact: CompactDevice::new(SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)), Some(UNIT_AREA_START_SLOT)),
            }
        }

        fn write(&mut self, offset: u64, bytes: &[u8]) {
            self.sparse.write_at(DeviceOffsetInBytes(offset), bytes, WriteDurability::Plain).expect("稀疏盘写得进");
            self.compact.write_at(DeviceOffsetInBytes(offset), bytes, WriteDurability::Plain).expect("紧凑盘写得进");
        }

        fn zero(&mut self, offset: u64, length: u64) {
            self.sparse.write_zeroes_at(DeviceOffsetInBytes(offset), length).expect("稀疏盘清得掉");
            self.compact.write_zeroes_at(DeviceOffsetInBytes(offset), length).expect("紧凑盘清得掉");
        }

        fn discard(&mut self, offset: u64, length: u64) {
            let from_sparse = self.sparse.discard_at(DeviceOffsetInBytes(offset), length).expect("稀疏盘认 discard");
            let from_compact = self.compact.discard_at(DeviceOffsetInBytes(offset), length).expect("紧凑盘认 discard");
            assert!(matches!((from_sparse, from_compact), (DiscardOutcome::Applied, DiscardOutcome::Applied)));
        }

        /// 整盘（每 64 KiB 一读）两边读回逐字节相同。
        fn assert_the_same_everywhere(&self) {
            let step = 64 << 10;
            for offset in (0..DEVICE_BYTES).step_by(step) {
                let mut left = vec![0_u8; step];
                let mut right = vec![1_u8; step];
                self.sparse.read_at(DeviceOffsetInBytes(offset), &mut left).expect("读得出");
                self.compact.read_at(DeviceOffsetInBytes(offset), &mut right).expect("读得出");
                assert!(left == right, "偏移 {offset} 起 {step} 字节读回不同");
            }
        }
    }

    fn pattern(length: usize, seed: u8) -> Vec<u8> {
        (0..length).map(|index| u8::try_from((index * 131 + usize::from(seed) * 7 + 1) % 251).expect("小于 256")).collect()
    }

    #[test]
    fn writes_and_reads_match_the_sparse_device_across_the_unit_area_boundary() {
        let mut pair = Pair::new();
        let boundary = UNIT_AREA_START_SLOT * 16384;
        pair.write(0, &pattern(2048, 1));
        // 跨单元区起点的一次写：下面一半走稀疏盘、上面一半走紧凑存法
        pair.write(boundary - 2048, &pattern(4096, 2));
        pair.write(boundary + 32768, &pattern(32768, 3));
        pair.write(boundary + 32768 + 512, &pattern(512, 4));
        pair.assert_the_same_everywhere();
        let mut straddling_left = vec![0_u8; 4096];
        let mut straddling_right = vec![9_u8; 4096];
        pair.sparse.read_at(DeviceOffsetInBytes(boundary - 2048), &mut straddling_left).expect("读得出");
        pair.compact.read_at(DeviceOffsetInBytes(boundary - 2048), &mut straddling_right).expect("读得出");
        assert!(straddling_left == straddling_right);
    }

    #[test]
    fn zero_fill_and_discard_remove_the_written_sectors_like_the_sparse_device() {
        let mut pair = Pair::new();
        let boundary = UNIT_AREA_START_SLOT * 16384;
        pair.write(boundary, &pattern(16384 * 6, 5));
        pair.zero(boundary + 512, 1024);
        pair.assert_the_same_everywhere();
        pair.discard(boundary + 16384, 16384 * 2);
        pair.assert_the_same_everywhere();
        pair.zero(boundary - 1024, 4096);
        pair.assert_the_same_everywhere();
        pair.write(boundary + 16384, &pattern(512, 6));
        pair.assert_the_same_everywhere();
        assert_eq!(pair.compact.stored().written_sectors, u64::try_from(pair.sparse.image.written_sectors().len()).expect("个数"));
    }

    #[test]
    fn the_same_sector_contents_are_stored_once() {
        let mut pair = Pair::new();
        let boundary = UNIT_AREA_START_SLOT * 16384;
        // 两个单元的载荷相同、各只头一个扇区不同：不同内容的份数远小于写过的扇区数
        let mut unit = pattern(32768, 7);
        for number in 0..8_u8 {
            unit[0] = number;
            pair.write(boundary + u64::from(number) * 32768, &unit);
        }
        let stored = pair.compact.stored();
        assert_eq!(stored.written_sectors, 8 * 64);
        assert_eq!(stored.distinct_contents, 8 + 63);
        pair.assert_the_same_everywhere();
    }

    #[test]
    fn a_device_without_the_unit_area_start_is_the_sparse_device() {
        let mut compact = CompactDevice::new(SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)), None);
        compact.write_at(DeviceOffsetInBytes(1 << 20), &pattern(4096, 8), WriteDurability::Plain).expect("写得进");
        assert_eq!(compact.sparse_image().written_sectors().len(), 8);
        assert_eq!(compact.stored(), Stored { written_sectors: 8, distinct_contents: 0 });
    }
}

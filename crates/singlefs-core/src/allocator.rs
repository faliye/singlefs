//! 分配器（里程碑步 2 / 步 3）：内存里一张单元区的空闲图，两条落点政策，分配记录与记账增量在提交时增量维护。
//!
//! 用户数据落点（D3（空间分配） 已定项 8 / 已定项 10）：已选设备内起点槽号最小、起点 32768 对齐（偶数槽）、跨度内两槽都空、
//! 不在任何开放的聚簇段里、不在 `R` 保护期内（第一版 R 为空）。
//! 提交内生块（D3（空间分配） 已定项 5 / 已定项 10）：从开放聚簇段 bump，开放段 = 单元区内最低的 64 槽对齐全空段，bump 只在内存；
//! 码 3 容器按数据单元那一档取落点（起点 32768 对齐），码 2 节点取最低空槽。
//! 记账（D5（快照 / 空间记账机制） 已定项 7 / 已定项 8）：已分配 = 落点之和、容量 = 单元区、runs = 空闲槽的连续段数（逐设备）、
//! 全空聚簇段数逐设备——全部在分配那一刻增量维护，运行时不扫盘（`.claude/rules/fs-design.md` 第一格）。

use singlefs_format::{CLUSTER_SEGMENT_SLOTS, SLOT_BYTES, UNIT_AREA_START_SLOT};

use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use crate::bytes::ByteWriter;

/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备 4, 槽号 6) + value (跨度 2 含已释放标志, 分配代 8)。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllocationRecord {
    pub device: DeviceIdentity,
    pub slot: SlotNumber,
    pub span_slots: u16,
    pub generation: CheckpointTxg,
}

impl AllocationRecord {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new(20);
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u16(self.span_slots);
        writer.put_u64(self.generation.0);
        writer.assert_position(20, "分配记录");
        writer.into_bytes()
    }
    /// key = (设备 4, 槽号 6)。
    #[must_use]
    pub fn key_bytes(&self) -> Vec<u8> {
        self.to_bytes()[..10].to_vec()
    }
    /// 按字段比：设备身份，再槽号（D8（核心索引结构） 已定项 11）。
    #[must_use]
    pub fn sort_key(&self) -> (u32, u64) {
        (self.device.0, self.slot.0)
    }
    #[must_use]
    pub fn parse(bytes: &[u8]) -> Self {
        let mut reader = crate::bytes::ByteReader::at(bytes, 0);
        let device = DeviceIdentity(reader.get_u32());
        let slot = SlotNumber(reader.get_six_byte_unsigned());
        let span_slots = reader.get_u16();
        let generation = CheckpointTxg(reader.get_u64());
        Self {
            device,
            slot,
            span_slots,
            generation,
        }
    }
}

/// 一个单元占几个槽：数据单元与打包容器 2，索引节点 1。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitFootprint {
    TwoSlotsAligned,
    OneSlot,
}

impl UnitFootprint {
    const fn slots(self) -> u64 {
        match self {
            UnitFootprint::TwoSlotsAligned => 2,
            UnitFootprint::OneSlot => 1,
        }
    }
}

/// 一块设备的空闲图与它的记账增量。
#[derive(Clone, Debug)]
pub struct DeviceFreeMap {
    pub device: DeviceIdentity,
    unit_area_slots: u64,
    /// 每个 16 KiB 槽一位：true = 已分配。
    allocated: Vec<bool>,
    /// 每个 64 槽聚簇段里已分配的槽数（增量维护全空段数用）。
    used_per_segment: Vec<u64>,
    /// 空闲槽的连续段数（增量维护）。
    free_runs: u64,
    allocated_slots: u64,
}

impl DeviceFreeMap {
    /// 单元区从 50176 起到设备末尾；mkfs 占的槽由调用方标上。
    #[must_use]
    pub fn new(device: DeviceIdentity, device_bytes: u64) -> Self {
        let unit_area_slots = device_bytes / SLOT_BYTES - UNIT_AREA_START_SLOT;
        let segments = unit_area_slots.div_ceil(CLUSTER_SEGMENT_SLOTS);
        Self {
            device,
            unit_area_slots,
            allocated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
            used_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            free_runs: u64::from(unit_area_slots > 0),
            allocated_slots: 0,
        }
    }

    fn index(slot: SlotNumber) -> usize {
        usize::try_from(slot.0 - UNIT_AREA_START_SLOT).expect("槽号在单元区内")
    }

    #[must_use]
    pub fn is_free(&self, slot: SlotNumber) -> bool {
        slot.0 >= UNIT_AREA_START_SLOT
            && slot.0 < UNIT_AREA_START_SLOT + self.unit_area_slots
            && !self.allocated[Self::index(slot)]
    }

    #[must_use]
    pub fn unit_area_slots(&self) -> u64 {
        self.unit_area_slots
    }
    #[must_use]
    pub fn allocated_slots(&self) -> u64 {
        self.allocated_slots
    }
    #[must_use]
    pub fn free_slots(&self) -> u64 {
        self.unit_area_slots - self.allocated_slots
    }
    #[must_use]
    pub fn free_runs(&self) -> u64 {
        self.free_runs
    }
    #[must_use]
    pub fn empty_segments(&self) -> u64 {
        u64::try_from(
            self.used_per_segment
                .iter()
                .filter(|used| **used == 0)
                .count(),
        )
        .expect("段数")
    }

    /// 段内是不是整段空闲。
    #[must_use]
    pub fn segment_is_empty(&self, segment_index: usize) -> bool {
        self.used_per_segment[segment_index] == 0
    }

    /// 把 `[slot, slot + span)` 标成已分配，并增量更新 runs 与段计数。要求整个跨度此前都空闲。
    pub fn mark_allocated(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
        assert!(
            self.allocated[start..end].iter().all(|taken| !*taken),
            "跨度里有已分配的槽"
        );
        // runs 的增量：被切的那一段空闲 run 左右各剩不剩东西。
        let left_free = start > 0 && !self.allocated[start - 1];
        let right_free = end < self.allocated.len() && !self.allocated[end];
        let pieces_left = u64::from(left_free) + u64::from(right_free);
        self.free_runs = self.free_runs + pieces_left - 1;
        for index in start..end {
            self.allocated[index] = true;
            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
            self.used_per_segment[segment] += 1;
        }
        self.allocated_slots += span;
    }

    /// 用户数据落点：起点槽号最小、偶数槽（32768 对齐）、两槽都空、不在开放段里。
    #[must_use]
    pub fn lowest_user_data_slot(&self, open_segment: Option<SlotNumber>) -> Option<SlotNumber> {
        let mut candidate = UNIT_AREA_START_SLOT;
        let end = UNIT_AREA_START_SLOT + self.unit_area_slots;
        while candidate + 1 < end {
            let inside_open_segment = open_segment.is_some_and(|open| {
                candidate >= open.0 && candidate < open.0 + CLUSTER_SEGMENT_SLOTS
            });
            if !inside_open_segment
                && self.is_free(SlotNumber(candidate))
                && self.is_free(SlotNumber(candidate + 1))
            {
                return Some(SlotNumber(candidate));
            }
            candidate += 2;
        }
        None
    }

    /// 单元区内最低的 64 槽对齐全空段的起点。
    #[must_use]
    pub fn lowest_empty_segment(&self) -> Option<SlotNumber> {
        let per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        let full_segments = self.allocated.len() / per_segment;
        (0..full_segments)
            .find(|segment| self.used_per_segment[*segment] == 0)
            .map(|segment| {
                SlotNumber(
                    UNIT_AREA_START_SLOT
                        + u64::try_from(segment).expect("段号") * CLUSTER_SEGMENT_SLOTS,
                )
            })
    }
}

/// 一次发布里分配的一个落点，两盘同槽（D2（RAID 条带策略） 已定项 10：一个单元整个落在一列上、两盘各一份）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub slot: SlotNumber,
    pub span: u64,
}

/// 池级分配器：两盘同构、同一条规则给同一个号。
#[derive(Clone, Debug)]
pub struct PoolAllocator {
    pub devices: Vec<DeviceFreeMap>,
    /// 开放聚簇段的起点与 bump 游标（只在内存）。
    open_segment: Option<SlotNumber>,
    bump_cursor: u64,
    /// C146（无空段时的回落政策全仓无定义） ② 的运行时计数：实际落点与政策函数不一致的次数，第一个事务恒 0。
    pub policy_mismatches: u64,
    records: Vec<AllocationRecord>,
}

impl PoolAllocator {
    #[must_use]
    pub fn new(devices: Vec<DeviceFreeMap>) -> Self {
        Self {
            devices,
            open_segment: None,
            bump_cursor: 0,
            policy_mismatches: 0,
            records: Vec::new(),
        }
    }

    /// mkfs 写在单元区里的两个单元：每盘一条分配代 0 的分配记录（字节表五：20 条里 m1 / m2 各两条，分配代 0）。
    /// 单元区之外的固定结构（超级块、根环、journal 环）不写分配记录、分配器不下探（D3（空间分配） 已定项 10 ④）。
    pub fn mark_format_time_units(&mut self, placements: &[Placement]) {
        for placement in placements {
            self.record(*placement, CheckpointTxg(0));
        }
    }

    #[must_use]
    pub fn records(&self) -> &[AllocationRecord] {
        &self.records
    }

    fn record(&mut self, placement: Placement, generation: CheckpointTxg) {
        for device in &self.devices {
            self.records.push(AllocationRecord {
                device: device.device,
                slot: placement.slot,
                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),
                generation,
            });
        }
        for device in &mut self.devices {
            device.mark_allocated(placement.slot, placement.span);
        }
    }

    /// 用户数据：按政策函数取，并把「政策函数再算一遍」与实际落点比对（不一致计数）。
    pub fn allocate_user_data(&mut self, generation: CheckpointTxg) -> Option<Placement> {
        let open = self.open_segment;
        let chosen = self.devices[0].lowest_user_data_slot(open)?;
        let recomputed = self
            .devices
            .iter()
            .filter_map(|device| device.lowest_user_data_slot(open))
            .min()?;
        if recomputed != chosen {
            self.policy_mismatches += 1;
        }
        let placement = Placement {
            slot: chosen,
            span: 2,
        };
        self.record(placement, generation);
        Some(placement)
    }

    /// 提交内生块：从开放段 bump；容器按 32768 对齐档，节点取游标处最低空槽。开放段满了就开下一个全空段。
    pub fn allocate_commit_generated(
        &mut self,
        footprint: UnitFootprint,
        generation: CheckpointTxg,
    ) -> Option<Placement> {
        if self.open_segment.is_none() {
            let start = self.devices[0].lowest_empty_segment()?;
            self.open_segment = Some(start);
            self.bump_cursor = start.0;
        }
        let open = self.open_segment.expect("上面刚开");
        let segment_end = open.0 + CLUSTER_SEGMENT_SLOTS;
        let mut start = self.bump_cursor;
        if footprint == UnitFootprint::TwoSlotsAligned && !start.is_multiple_of(2) {
            start += 1;
        }
        if start + footprint.slots() > segment_end {
            self.open_segment = None;
            return self.allocate_commit_generated(footprint, generation);
        }
        let placement = Placement {
            slot: SlotNumber(start),
            span: footprint.slots(),
        };
        self.bump_cursor = start + footprint.slots();
        self.record(placement, generation);
        Some(placement)
    }

    #[must_use]
    pub fn open_segment(&self) -> Option<SlotNumber> {
        self.open_segment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool_after_mkfs() -> PoolAllocator {
        let device_bytes = 4u64 << 30;
        let mut pool = PoolAllocator::new(vec![
            DeviceFreeMap::new(DeviceIdentity(0), device_bytes),
            DeviceFreeMap::new(DeviceIdentity(1), device_bytes),
        ]);
        pool.mark_format_time_units(&[
            Placement {
                slot: SlotNumber(50176),
                span: 2,
            },
            Placement {
                slot: SlotNumber(50178),
                span: 1,
            },
        ]);
        pool
    }

    #[test]
    fn unit_area_of_a_4_gib_image_has_211968_slots_in_3312_segments() {
        let map = DeviceFreeMap::new(DeviceIdentity(0), 4 << 30);
        assert_eq!(map.unit_area_slots(), 211_968);
        assert_eq!(map.empty_segments(), 3312);
        assert_eq!(map.free_runs(), 1);
    }

    #[test]
    fn first_transaction_placements_follow_the_byte_table() {
        let mut pool = pool_after_mkfs();
        assert_eq!(
            pool.devices[0].free_runs(),
            1,
            "mkfs 占了单元区开头，空闲仍是一段"
        );
        assert_eq!(pool.devices[0].empty_segments(), 3311);
        let data = pool.allocate_user_data(CheckpointTxg(3)).expect("有空槽");
        assert_eq!(
            data,
            Placement {
                slot: SlotNumber(50180),
                span: 2
            },
            "t1@50180：最低的偶数空槽对"
        );
        assert_eq!(
            pool.records().len(),
            6,
            "mkfs 两个单元 × 2 盘（分配代 0）+ t1 × 2 盘"
        );
        assert_eq!(pool.policy_mismatches, 0);
        let extent_root = pool
            .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
            .expect("开放段");
        assert_eq!(
            extent_root.slot,
            SlotNumber(50240),
            "开放段 = 最低的 64 槽对齐全空段"
        );
        let inode_leaf = pool
            .allocate_commit_generated(UnitFootprint::TwoSlotsAligned, CheckpointTxg(3))
            .expect("开放段");
        assert_eq!(
            inode_leaf,
            Placement {
                slot: SlotNumber(50242),
                span: 2
            },
            "容器按 32768 对齐，槽 50241 空着"
        );
        for expected in [50244u64, 50245, 50246, 50247, 50248] {
            let node = pool
                .allocate_commit_generated(UnitFootprint::OneSlot, CheckpointTxg(3))
                .expect("开放段");
            assert_eq!(node.slot, SlotNumber(expected));
        }
        assert_eq!(
            pool.records().len(),
            20,
            "m1 m2 t1..t8 十个单元 × 2 盘（字节表五）"
        );
    }

    #[test]
    fn accounting_deltas_are_maintained_incrementally() {
        let mut pool = pool_after_mkfs();
        pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
        assert_eq!(pool.devices[0].allocated_slots(), 5, "mkfs 3 + t1 2");
        assert_eq!(pool.devices[0].free_slots(), 211_968 - 5);
        assert_eq!(pool.devices[0].free_runs(), 2, "[50179] 与 [50182, 末]");
        assert_eq!(pool.devices[0].empty_segments(), 3311);
        for footprint in [
            UnitFootprint::OneSlot,
            UnitFootprint::TwoSlotsAligned,
            UnitFootprint::OneSlot,
            UnitFootprint::OneSlot,
            UnitFootprint::OneSlot,
            UnitFootprint::OneSlot,
            UnitFootprint::OneSlot,
        ] {
            pool.allocate_commit_generated(footprint, CheckpointTxg(3))
                .expect("开放段");
        }
        assert_eq!(pool.devices[0].allocated_slots(), 13, "字节表五：13 个槽");
        assert_eq!(pool.devices[0].free_slots() * SLOT_BYTES, 3_472_670_720);
        assert_eq!(
            pool.devices[0].free_runs(),
            4,
            "[50179]、[50182, 50239]、[50241]、[50249, 末]"
        );
        assert_eq!(pool.devices[0].empty_segments(), 3310);
        assert_eq!(pool.devices[1].free_runs(), 4, "两盘同构");
    }
}

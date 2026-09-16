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

/// 跨度段的最高位借作已释放标志（D3（空间分配） 已定项 7 / 已定项 11）：0 = 仍分配、1 = 已释放，不占表达跨度值的位。
pub const ALLOCATION_RECORD_RELEASED_FLAG: u16 = 0x8000;

/// 分配记录 20（D3（空间分配） 已定项 7 / 已定项 11）：key (设备 4, 槽号 6) + value (跨度 2 含已释放标志, 分配代或释放代 8)。
/// 释放时条目不删、不点删：改写成已释放 + 释放代，留到该落点被重新分配时覆盖（覆盖是步 5 回收接上时的事：
/// 可再分配谓词今天没实现、同一个落点不会再分配到，所以 `record` 今天只追加）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AllocationRecord {
    pub device: DeviceIdentity,
    pub slot: SlotNumber,
    /// 纯跨度，不含标志位。
    pub span_slots: u16,
    /// 仍分配时是分配代；已释放时是释放代。
    pub generation: CheckpointTxg,
    pub is_released: bool,
}

impl AllocationRecord {
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        assert_eq!(
            self.span_slots & ALLOCATION_RECORD_RELEASED_FLAG,
            0,
            "跨度值不许占到标志位"
        );
        let span_field = if self.is_released {
            self.span_slots | ALLOCATION_RECORD_RELEASED_FLAG
        } else {
            self.span_slots
        };
        let mut writer = ByteWriter::new(20);
        writer.put_u32(self.device.0);
        writer.put_six_byte_unsigned(self.slot.0);
        writer.put_u16(span_field);
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
        let span_field = reader.get_u16();
        let generation = CheckpointTxg(reader.get_u64());
        Self {
            device,
            slot,
            span_slots: span_field & !ALLOCATION_RECORD_RELEASED_FLAG,
            generation,
            is_released: span_field & ALLOCATION_RECORD_RELEASED_FLAG != 0,
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
    /// 占着的槽数：仍分配的加上已释放、还在 defer 窗口里的——它们仍被根环里的有效根引用、仍占着空间
    /// （I-3.1（已分配统计对得上） 的读法 2026-09-14 用户定甲：按根环里全部有效根的引用取并集）。
    allocated_slots: u64,
    /// 空闲槽数，独立维护：分配时减、回收放回时加，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️：
    /// 那样 I-5.2（空闲统计对得上） 是恒真式；三方代码第一轮攻方腿打中）。已释放而还在 defer 窗口里的不算空闲。
    free_slots: u64,
    /// 其中已释放、还在 defer 窗口里的槽数（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
    deferred_slots: u64,
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
            free_slots: unit_area_slots,
            deferred_slots: 0,
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
    /// 独立维护的空闲槽数（I-5.2（空闲统计对得上） 拿它与「已分配」相加对容量），已释放而还在 defer 窗口里的不算空闲。
    #[must_use]
    pub fn free_slots(&self) -> u64 {
        self.free_slots
    }
    #[must_use]
    pub fn deferred_slots(&self) -> u64 {
        self.deferred_slots
    }

    /// 把一个仍分配的落点放进 defer 队列：槽仍占着（分配器不许再发它），只是记账上从「仍分配」挪到「待释放」。
    pub fn mark_released(&mut self, slot: SlotNumber, span: u64) {
        let start = Self::index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            self.allocated[start..end].iter().all(|taken| *taken),
            "释放的跨度里有没分配的槽"
        );
        self.deferred_slots += span;
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
        self.free_slots -= span;
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

    /// 某块盘上某个槽的分配记录；没分配过就 `None`。释放判定路径拿它核「映射查出来的落点真的在册、跨度对得上」。
    #[must_use]
    pub fn record_for(
        &self,
        device: DeviceIdentity,
        slot: SlotNumber,
    ) -> Option<&AllocationRecord> {
        self.records
            .iter()
            .find(|record| record.device == device && record.slot == slot)
    }

    fn record(&mut self, placement: Placement, generation: CheckpointTxg) {
        for device in &self.devices {
            self.records.push(AllocationRecord {
                device: device.device,
                slot: placement.slot,
                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),
                generation,
                is_released: false,
            });
        }
        for device in &mut self.devices {
            device.mark_allocated(placement.slot, placement.span);
        }
    }

    /// 释放一个落点（D3（空间分配） 已定项 7：「释放」= 放进 defer 队列那一刻）：每盘那条分配记录改写成已释放 + 释放代，
    /// 条目不删；槽仍占着，要等释放代 ≤ max(F_生效, 环里最旧有效根) 才可再分配（D16（发布语义） 已定项 1）。
    pub fn release(&mut self, placement: Placement, release_generation: CheckpointTxg) {
        for device in &mut self.devices {
            let record = self
                .records
                .iter_mut()
                .find(|record| record.device == device.device && record.slot == placement.slot)
                .expect("释放的落点要有分配记录");
            assert!(!record.is_released, "同一个落点释放了两次");
            assert_eq!(
                u64::from(record.span_slots),
                placement.span,
                "释放的跨度与分配记录不符"
            );
            record.is_released = true;
            record.generation = release_generation;
            device.mark_released(placement.slot, placement.span);
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

    #[test]
    fn releasing_a_placement_keeps_the_slots_occupied_and_moves_them_into_the_defer_queue() {
        let mut pool = pool_after_mkfs();
        let data = pool.allocate_user_data(CheckpointTxg(3)).expect("t1");
        pool.release(data, CheckpointTxg(4));
        assert_eq!(pool.devices[0].allocated_slots(), 5, "占着的槽数不变");
        assert_eq!(pool.devices[0].deferred_slots(), 2, "两槽进了 defer 队列");
        assert_eq!(pool.devices[0].free_slots(), 211_968 - 5, "空闲不变");
        let released: Vec<&AllocationRecord> = pool
            .records()
            .iter()
            .filter(|record| record.slot == data.slot)
            .collect();
        assert_eq!(released.len(), 2, "每盘一条");
        for record in released {
            assert!(record.is_released);
            assert_eq!(record.generation, CheckpointTxg(4), "释放代");
            assert_eq!(record.span_slots, 2, "跨度值不含标志位");
            let parsed = AllocationRecord::parse(&record.to_bytes());
            assert_eq!(parsed, *record, "标志位进跨度段最高位再读回来");
            assert_eq!(
                u16::from_le_bytes([record.to_bytes()[10], record.to_bytes()[11]]),
                2 | ALLOCATION_RECORD_RELEASED_FLAG
            );
        }
        assert_eq!(
            pool.allocate_user_data(CheckpointTxg(4))
                .expect("有空槽")
                .slot,
            SlotNumber(50182),
            "已释放的 50180 不许再发，下一个偶数空槽对是 50182"
        );
    }
}

//! 单元的足迹与放置规则、分配记录、根环的占用与表，以及分配器本身。

use super::*;

// ---------------------------------------------------------------------------------------------
// 分配器：W2–W6（G-环、G-宽）与 G-紧 的最低落点；根环表 W5
// ---------------------------------------------------------------------------------------------

pub(super) type UnitIdentifier = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UnitFootprint {
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

    const fn alignment_in_slots(self) -> u64 {
        match self {
            UnitFootprint::TwoSlotsAligned => 2,
            UnitFootprint::OneSlot => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlacementRule {
    UserData,
    CommitGenerated(UnitFootprint),
}

impl PlacementRule {
    pub(super) const fn span_slots(self) -> u64 {
        match self {
            PlacementRule::UserData => 2,
            PlacementRule::CommitGenerated(footprint) => footprint.slots(),
        }
    }
}

/// 一条分配记录（两块盘同槽、内容只差设备段，按槽号记一条，执行员修订 2 C1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AllocationRecord {
    pub(super) span_slots: u64,
    /// 仍分配时是分配代，已释放时是释放代。
    generation: u64,
    pub(super) is_released: bool,
    pub(super) unit: UnitIdentifier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootRingOccupant {
    ValidRoot(u64),
    /// 挂载时读回全零的槽，钉在它可能住过的最旧 txg（M4）。
    AllZeroPinnedAt(u64),
}

impl RootRingOccupant {
    pub(super) const fn txg(self) -> u64 {
        match self {
            RootRingOccupant::ValidRoot(txg) | RootRingOccupant::AllZeroPinnedAt(txg) => txg,
        }
    }
}

/// 根 t 写在哪个环槽：区 t mod 3、槽 ⌊t ÷ 3⌋ mod 8（`root_ring.rs:109-117`）。
pub(super) fn root_ring_slot_of(txg: u64) -> (u64, u64) {
    (
        txg % ROOT_RING_REGIONS,
        (txg / ROOT_RING_REGIONS) % ROOT_RING_SLOTS_PER_REGION,
    )
}

/// 分配器眼里的根环（W5）。
#[derive(Clone, Debug)]
pub(super) struct RootRingTable {
    latest_recorded_txg: u64,
    occupants: BTreeMap<(u64, u64), RootRingOccupant>,
}

/// F_生效：整段历史不抬 F，新池是 0（W7）；是模型的前提，不是读数。
const EFFECTIVE_FLOOR_OF_THE_WHOLE_HISTORY: u64 = 0;

impl RootRingTable {
    /// mkfs 那个进程：三个区的槽 0 各一条 txg 0（M3）。
    pub(super) fn after_make_filesystem() -> Self {
        Self {
            latest_recorded_txg: 0,
            occupants: (0..ROOT_RING_REGIONS)
                .map(|region| ((region, 0), RootRingOccupant::ValidRoot(0)))
                .collect(),
        }
    }

    /// 可写挂载从盘上读：有根的槽记它的 txg，读回全零的槽钉在 r + 3s（M4）。
    fn read_at_mount(roots_on_disk: &BTreeMap<(u64, u64), u64>, latest_recorded_txg: u64) -> Self {
        let mut occupants = BTreeMap::new();
        for region in 0..ROOT_RING_REGIONS {
            for slot in 0..ROOT_RING_SLOTS_PER_REGION {
                let occupant = match roots_on_disk.get(&(region, slot)) {
                    Some(txg) => RootRingOccupant::ValidRoot(*txg),
                    None => RootRingOccupant::AllZeroPinnedAt(region + ROOT_RING_REGIONS * slot),
                };
                occupants.insert((region, slot), occupant);
            }
        }
        Self {
            latest_recorded_txg,
            occupants,
        }
    }

    fn reclaim_floor(&self) -> u64 {
        let oldest = self
            .occupants
            .values()
            .map(|occupant| occupant.txg())
            .min()
            .unwrap_or(EFFECTIVE_FLOOR_OF_THE_WHOLE_HISTORY);
        oldest
    }

    /// 记一条这个进程写的根，交回此刻的回收门槛 max(F_生效, 环里最旧)。
    pub(super) fn record_root(&mut self, txg: u64) -> u64 {
        assert_eq!(txg, self.latest_recorded_txg + 1, "根的 txg 逐个加一，每一条都经分配器记");
        self.latest_recorded_txg = txg;
        self.occupants
            .insert(root_ring_slot_of(txg), RootRingOccupant::ValidRoot(txg));
        self.reclaim_floor()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PlacementRefused;

#[derive(Clone, Debug)]
pub(super) struct Allocator {
    pub(super) geometry: Geometry,
    /// 单元区槽数；G-宽 不设末端（`None`）。
    pub(super) unit_area_slots: Option<u64>,
    allocated: Vec<bool>,
    ever_allocated: Vec<bool>,
    used_per_segment: Vec<u64>,
    pub(super) allocated_slot_count: u64,
    /// 仍分配（没释放）的槽数：G-紧 的盘按 G-宽 下它的峰值定（C7）。
    pub(super) live_slot_count: u64,
    open_segment: Option<u64>,
    bump_cursor: u64,
    /// 这次挂载开过的聚簇段（W2）。
    pub(super) cluster_segments: BTreeSet<u64>,
    pub(super) records: BTreeMap<u64, AllocationRecord>,
    records_per_leaf: BTreeMap<u64, u64>,
    reclaimed: BTreeSet<u64>,
    pub(super) root_ring: Option<RootRingTable>,
    /// 这份分配器（固定点那一份拷贝）上动过记录的槽。
    pub(super) dirty_slots: BTreeSet<u64>,
    pub(super) reuse_placements: u64,
    pub(super) first_reuse_at: Option<u64>,
}

impl Allocator {
    pub(super) fn new(geometry: Geometry, unit_area_slots: Option<u64>) -> Self {
        let initial_slots = unit_area_slots.unwrap_or(0);
        Self {
            geometry,
            unit_area_slots,
            allocated: vec![false; u64_as_index(initial_slots)],
            ever_allocated: vec![false; u64_as_index(initial_slots)],
            used_per_segment: vec![0; u64_as_index(initial_slots.div_ceil(CLUSTER_SEGMENT_SLOTS))],
            allocated_slot_count: 0,
            live_slot_count: 0,
            open_segment: None,
            bump_cursor: 0,
            cluster_segments: BTreeSet::new(),
            records: BTreeMap::new(),
            records_per_leaf: BTreeMap::new(),
            reclaimed: BTreeSet::new(),
            root_ring: None,
            dirty_slots: BTreeSet::new(),
            reuse_placements: 0,
            first_reuse_at: None,
        }
    }

    pub(super) fn rehearsal_copy(&self) -> Self {
        let mut copy = self.clone();
        copy.dirty_slots.clear();
        copy
    }

    fn offset_of(slot: u64) -> usize {
        u64_as_index(slot.checked_sub(UNIT_AREA_START_SLOT).expect("槽号在单元区内"))
    }

    fn segment_start_of(slot: u64) -> u64 {
        UNIT_AREA_START_SLOT + ((slot - UNIT_AREA_START_SLOT) / CLUSTER_SEGMENT_SLOTS) * CLUSTER_SEGMENT_SLOTS
    }

    fn is_inside_the_unit_area(&self, slot: u64) -> bool {
        slot >= UNIT_AREA_START_SLOT
            && self
                .unit_area_slots
                .is_none_or(|slots| slot < UNIT_AREA_START_SLOT + slots)
    }

    pub(super) fn is_free(&self, slot: u64) -> bool {
        if !self.is_inside_the_unit_area(slot) {
            return false;
        }
        let offset = Self::offset_of(slot);
        offset >= self.allocated.len() || !self.allocated[offset]
    }

    fn grow_to_cover(&mut self, slot_end_exclusive: u64) {
        let needed = Self::offset_of(slot_end_exclusive);
        if self.allocated.len() < needed {
            assert_eq!(self.geometry, Geometry::Wide, "只有 G-宽 的单元区不设末端");
            let grown = needed.max(self.allocated.len() * 2).max(4096);
            self.allocated.resize(grown, false);
            self.ever_allocated.resize(grown, false);
            self.used_per_segment
                .resize(u64_as_index(length_as_u64(grown).div_ceil(CLUSTER_SEGMENT_SLOTS)), 0);
        }
    }

    fn mark_slot_allocated(&mut self, slot: u64) {
        let offset = Self::offset_of(slot);
        assert!(!self.allocated[offset], "跨度里有已分配的槽");
        self.allocated[offset] = true;
        self.ever_allocated[offset] = true;
        self.used_per_segment[offset / u64_as_index(CLUSTER_SEGMENT_SLOTS)] += 1;
        self.allocated_slot_count += 1;
    }

    fn mark_slot_free(&mut self, slot: u64) {
        let offset = Self::offset_of(slot);
        assert!(self.allocated[offset], "回收的跨度里有没分配的槽");
        self.allocated[offset] = false;
        self.used_per_segment[offset / u64_as_index(CLUSTER_SEGMENT_SLOTS)] -= 1;
        self.allocated_slot_count -= 1;
    }

    pub(super) fn leaf_of_slot(slot: u64) -> u64 {
        slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS
    }

    fn count_record_in_its_leaf(&mut self, slot: u64) {
        *self.records_per_leaf.entry(Self::leaf_of_slot(slot)).or_insert(0) += 1;
    }

    fn uncount_record_in_its_leaf(&mut self, slot: u64) {
        let leaf = Self::leaf_of_slot(slot);
        let count = self.records_per_leaf.get_mut(&leaf).expect("记录所在的叶有计数");
        *count -= 1;
        if *count == 0 {
            self.records_per_leaf.remove(&leaf);
        }
    }

    /// 用户数据：起点最小、偶数、两槽都空；G-环、G-宽 不落在这次挂载开过的任何聚簇段里（W2），G-紧 不模聚簇段。
    fn lowest_user_data_slot(&self) -> Option<u64> {
        let mut candidate = UNIT_AREA_START_SLOT;
        loop {
            if let Some(slots) = self.unit_area_slots {
                if candidate + 1 >= UNIT_AREA_START_SLOT + slots {
                    return None;
                }
            }
            if self.geometry != Geometry::Tight {
                let segment_start = Self::segment_start_of(candidate);
                if self.cluster_segments.contains(&segment_start) {
                    candidate = segment_start + CLUSTER_SEGMENT_SLOTS;
                    continue;
                }
            }
            if self.is_free(candidate) && self.is_free(candidate + 1) {
                return Some(candidate);
            }
            candidate += 2;
        }
    }

    /// 单元区里最低的整 64 槽全空段（W3）。
    fn lowest_empty_segment(&self) -> Option<u64> {
        let full_segments = match self.unit_area_slots {
            Some(slots) => slots / CLUSTER_SEGMENT_SLOTS,
            None => u64::MAX,
        };
        let mut segment = 0;
        while segment < full_segments {
            let position = u64_as_index(segment);
            if position >= self.used_per_segment.len() || self.used_per_segment[position] == 0 {
                return Some(UNIT_AREA_START_SLOT + segment * CLUSTER_SEGMENT_SLOTS);
            }
            segment += 1;
        }
        None
    }

    /// 最低的整个跨度都空的起点，按对齐步长走（W3 的回落；G-紧 的提交内生块）。
    fn lowest_fitting_commit_generated_slot(&self, footprint: UnitFootprint) -> Option<u64> {
        let mut candidate = UNIT_AREA_START_SLOT;
        loop {
            if let Some(slots) = self.unit_area_slots {
                if candidate + footprint.slots() > UNIT_AREA_START_SLOT + slots {
                    return None;
                }
            }
            if (candidate..candidate + footprint.slots()).all(|slot| self.is_free(slot)) {
                return Some(candidate);
            }
            candidate += footprint.alignment_in_slots();
        }
    }

    /// 开放段里从游标起第一个整跨度都空的对齐起点（W3）。
    fn bump_slot_in_open_segment(&self, open: u64, footprint: UnitFootprint) -> Option<u64> {
        let segment_end = open + CLUSTER_SEGMENT_SLOTS;
        let mut start = self.bump_cursor;
        if footprint == UnitFootprint::TwoSlotsAligned && !start.is_multiple_of(2) {
            start += 1;
        }
        while start + footprint.slots() <= segment_end
            && (start..start + footprint.slots()).any(|slot| !self.is_free(slot))
        {
            start += footprint.alignment_in_slots();
        }
        if start + footprint.slots() > segment_end {
            None
        } else {
            Some(start)
        }
    }

    fn commit_generated_slot(&mut self, footprint: UnitFootprint) -> Option<u64> {
        if let Some(open) = self.open_segment {
            if let Some(slot) = self.bump_slot_in_open_segment(open, footprint) {
                self.bump_cursor = slot + footprint.slots();
                return Some(slot);
            }
        }
        match self.lowest_empty_segment() {
            Some(segment_start) => {
                self.open_segment = Some(segment_start);
                self.bump_cursor = segment_start;
                self.cluster_segments.insert(segment_start);
                let slot = self
                    .bump_slot_in_open_segment(segment_start, footprint)
                    .expect("全空段装得下任何一个单元");
                self.bump_cursor = slot + footprint.slots();
                Some(slot)
            }
            None => {
                let slot = self.lowest_fitting_commit_generated_slot(footprint)?;
                self.open_segment = None;
                Some(slot)
            }
        }
    }

    fn lowest_fitting_slot_without_segments(&self, rule: PlacementRule) -> Option<u64> {
        match rule {
            PlacementRule::UserData => self.lowest_user_data_slot(),
            PlacementRule::CommitGenerated(footprint) => self.lowest_fitting_commit_generated_slot(footprint),
        }
    }

    /// G-紧：取不到落点时把「扣着」里除这次发布换下的之外全部放回空闲，再取（原文 5.1；C7）。
    fn tight_slot(&mut self, rule: PlacementRule, txg: u64) -> Option<u64> {
        if let Some(slot) = self.lowest_fitting_slot_without_segments(rule) {
            return Some(slot);
        }
        self.reclaim_released_records_where(|record| record.generation < txg);
        self.lowest_fitting_slot_without_segments(rule)
    }

    pub(super) fn allocate(&mut self, rule: PlacementRule, txg: u64, unit: UnitIdentifier) -> Result<u64, PlacementRefused> {
        let slot = match self.geometry {
            Geometry::Tight => self.tight_slot(rule, txg),
            Geometry::Ring | Geometry::Wide => match rule {
                PlacementRule::UserData => self.lowest_user_data_slot(),
                PlacementRule::CommitGenerated(footprint) => self.commit_generated_slot(footprint),
            },
        }
        .ok_or(PlacementRefused)?;
        self.record(slot, rule.span_slots(), txg, unit);
        Ok(slot)
    }

    /// 记下一次分配（W6）：起点就是这个槽的已回收记录改写成已分配，罩住的别的已回收记录删掉。
    pub(super) fn record(&mut self, slot: u64, span: u64, txg: u64, unit: UnitIdentifier) {
        let end = slot + span;
        self.grow_to_cover(end);
        let is_reuse = (slot..end).any(|each| self.ever_allocated[Self::offset_of(each)]);
        if is_reuse {
            self.reuse_placements += 1;
            if self.first_reuse_at.is_none() {
                self.first_reuse_at = Some(txg);
            }
        }
        let intersecting: Vec<u64> = self
            .records
            .range(slot.saturating_sub(1)..end)
            .filter(|(record_slot, record)| **record_slot + record.span_slots > slot)
            .map(|(record_slot, _)| *record_slot)
            .collect();
        let mut rewritten_in_place = false;
        for record_slot in intersecting {
            assert!(
                self.reclaimed.remove(&record_slot),
                "与新落点相交的记录是已回收的：位图与记录分叉了"
            );
            self.dirty_slots.insert(record_slot);
            if record_slot == slot {
                let existing = self.records.get_mut(&record_slot).expect("刚找到这条记录");
                assert!(existing.is_released, "回收过的落点的记录是已释放的");
                *existing = AllocationRecord {
                    span_slots: span,
                    generation: txg,
                    is_released: false,
                    unit,
                };
                rewritten_in_place = true;
            } else {
                self.records.remove(&record_slot);
                self.uncount_record_in_its_leaf(record_slot);
            }
        }
        if !rewritten_in_place {
            self.records.insert(
                slot,
                AllocationRecord {
                    span_slots: span,
                    generation: txg,
                    is_released: false,
                    unit,
                },
            );
            self.count_record_in_its_leaf(slot);
        }
        self.dirty_slots.insert(slot);
        for each in slot..end {
            self.mark_slot_allocated(each);
        }
        self.live_slot_count += span;
    }

    /// 释放（W4）：记录改成已释放 + 释放代，槽仍占着。
    pub(super) fn release(&mut self, slot: u64, span: u64, txg: u64) {
        let record = self.records.get_mut(&slot).expect("释放的落点在册");
        assert_eq!(record.span_slots, span, "释放的跨度与记录相同");
        assert!(!record.is_released, "同一个落点不释放两次");
        record.is_released = true;
        record.generation = txg;
        self.dirty_slots.insert(slot);
        self.live_slot_count -= span;
    }

    /// 回收满足条件的已释放记录：槽回到空闲，记录照旧写着已释放（W5、W6）。交回回收的落点 (槽, 跨度)。
    fn reclaim_released_records_where(&mut self, should_reclaim: impl Fn(&AllocationRecord) -> bool) -> Vec<(u64, u64)> {
        let candidates: Vec<(u64, u64)> = self
            .records
            .iter()
            .filter(|entry| {
                let (slot, record) = *entry;
                record.is_released && should_reclaim(record) && !self.reclaimed.contains(slot)
            })
            .map(|(slot, record)| (*slot, record.span_slots))
            .collect();
        for (slot, span) in &candidates {
            self.reclaimed.insert(*slot);
            for each in *slot..*slot + *span {
                self.mark_slot_free(each);
            }
        }
        candidates
    }

    /// 可写挂载重建分配器：没有开放段、聚簇段集合清空（M4）；G-环 装上从盘上读的根环表并按挂载那一刻的门槛回收。
    pub(super) fn reset_at_mount(&mut self, roots_on_disk: &BTreeMap<(u64, u64), u64>, latest_recorded_txg: u64) {
        self.open_segment = None;
        self.bump_cursor = 0;
        self.cluster_segments.clear();
        if self.geometry == Geometry::Ring {
            let table = RootRingTable::read_at_mount(roots_on_disk, latest_recorded_txg);
            let floor = table.reclaim_floor();
            self.root_ring = Some(table);
            self.reclaim_released_records_where(|record| record.generation <= floor);
        }
    }

    /// 这次发布的落点取完之后记根（W5）：G-环 按门槛回收，交回回收的落点；G-宽、G-紧 不按环回收。
    pub(super) fn record_root(&mut self, txg: u64) -> Vec<(u64, u64)> {
        match self.geometry {
            Geometry::Ring => {
                let floor = self
                    .root_ring
                    .as_mut()
                    .expect("G-环 的分配器装着根环表")
                    .record_root(txg);
                self.reclaim_released_records_where(|record| record.generation <= floor)
            }
            Geometry::Wide | Geometry::Tight => Vec::new(),
        }
    }

    /// 装着记录的叶（两块盘各一片）、它们的祖先与根（`allocation_record_tree.rs:335-363` 的口径）。
    pub(super) fn nodes_holding_records(&self) -> BTreeSet<AllocationNode> {
        allocation_nodes_of_leaves(self.records_per_leaf.keys().copied())
    }
}

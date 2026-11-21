//! 用户可见版本、被隔离的单元、池状态、放置的步行、写计数与一次发布要写的各部分。

use super::*;

/// 用户看得见的那一版：数据单元、extent 树、inode 树（回退取目标那一版的就是这几样，登记第五节「回退事件」）。
#[derive(Clone, Debug)]
pub(super) struct UserVisibleVersion {
    pub(super) data_units: Vec<UnitIdentifier>,
    pub(super) extent_leaves: Vec<UnitIdentifier>,
    pub(super) extent_root: Option<UnitIdentifier>,
    pub(super) inode_leaf_container: UnitIdentifier,
    pub(super) inode_root: UnitIdentifier,
}

impl UserVisibleVersion {
    pub(super) fn units(&self) -> BTreeSet<UnitIdentifier> {
        let mut units: BTreeSet<UnitIdentifier> = self.data_units.iter().chain(self.extent_leaves.iter()).copied().collect();
        units.extend(self.extent_root);
        units.insert(self.inode_leaf_container);
        units.insert(self.inode_root);
        units
    }
}

/// 一个被隔离、记录留在已分配的单元。
#[derive(Clone, Copy, Debug)]
pub(super) struct QuarantinedUnit {
    pub(super) slot: u64,
    pub(super) span_slots: u64,
}

/// 一条臂在一段历史里的池：分配器（位图、段、开放段、defer）、分配记录、全部单元、这一版的各棵树。
pub(super) struct PoolState {
    slots_per_device: u64,
    pub(super) root_level: u8,
    pub(super) candidate_window: u64,
    pub(super) txg: u64,
    occupied_slots: Vec<u64>,
    pub(super) used_slots_per_segment: Vec<u32>,
    pub(super) is_cluster_segment: Vec<bool>,
    pub(super) open_segment_start: Option<u64>,
    pub(super) bump_cursor: u64,
    pub(super) lowest_possibly_free_slot: u64,
    pub(super) lowest_possibly_empty_segment: usize,
    pub(super) records: BTreeMap<u64, AllocationRecord>,
    pub(super) leaves_with_records: BTreeSet<u64>,
    /// (释放代, 槽) → 跨度。
    pub(super) deferred: BTreeMap<(u64, u64), u64>,
    pub(super) units: Vec<Option<UnitInformation>>,
    pub(super) version: Option<UserVisibleVersion>,
    pub(super) allocation_nodes: BTreeMap<AllocationNodePosition, UnitIdentifier>,
    pub(super) accounting: Option<UnitIdentifier>,
    pub(super) tree_table: Option<UnitIdentifier>,
    pub(super) mapping: Option<CodeTwoNode<MappingKey>>,
    pub(super) quarantine_tree: Option<CodeTwoNode<QuarantineEntryKey>>,
    pub(super) quarantine_tree_registered: bool,
    pub(super) quarantined: BTreeMap<UnitIdentifier, QuarantinedUnit>,
    pub(super) keeps_versions_for_rollback: bool,
    pub(super) kept_versions: BTreeMap<u64, UserVisibleVersion>,
}

fn segment_of(slot: u64) -> usize {
    usize_of((slot - UNIT_AREA_START_SLOT) / CLUSTER_SEGMENT_SLOTS)
}

fn segment_start(segment: usize) -> u64 {
    UNIT_AREA_START_SLOT + u64_of(segment) * CLUSTER_SEGMENT_SLOTS
}

/// 回收门槛与 I-3.1 候选并集的下沿：最新 txg − K（登记第五节「释放与回收」，两处同一个函数）。
pub(super) fn oldest_candidate_txg(newest_txg: u64, candidate_window: u64) -> u64 {
    newest_txg.saturating_sub(candidate_window)
}

impl PoolState {
    /// mkfs 之后、新池新建文件之前的池：单元区起点那一段里 mkfs 留下的两条记录，别的全空；这次挂载还没开过聚簇段。
    pub(super) fn after_make_filesystem(geometry: PoolGeometry, candidate_window: u64, keeps_versions_for_rollback: bool) -> PoolState {
        let slots_per_device = geometry.slots_per_device();
        let segment_count = usize_of((slots_per_device - UNIT_AREA_START_SLOT) / CLUSTER_SEGMENT_SLOTS);
        let mut state = PoolState {
            slots_per_device,
            root_level: allocation_record_tree_root_level(slots_per_device),
            candidate_window,
            txg: 1,
            occupied_slots: vec![0; usize_of(slots_per_device.div_ceil(64))],
            used_slots_per_segment: vec![0; segment_count],
            is_cluster_segment: vec![false; segment_count],
            open_segment_start: None,
            bump_cursor: 0,
            lowest_possibly_free_slot: UNIT_AREA_START_SLOT,
            lowest_possibly_empty_segment: 0,
            records: BTreeMap::new(),
            leaves_with_records: BTreeSet::new(),
            deferred: BTreeMap::new(),
            units: Vec::new(),
            version: None,
            allocation_nodes: BTreeMap::new(),
            accounting: None,
            tree_table: None,
            mapping: None,
            quarantine_tree: None,
            quarantine_tree_registered: false,
            quarantined: BTreeMap::new(),
            keeps_versions_for_rollback,
            kept_versions: BTreeMap::new(),
        };
        for slot in MAKE_FILESYSTEM_RECORD_SLOTS {
            state.occupy_with_a_record_that_never_changes(slot, 1);
        }
        state
    }

    /// 一条从此不动的记录（mkfs 那两条；微历史里垫满一段槽位的填充记录）。
    pub(super) fn occupy_with_a_record_that_never_changes(&mut self, slot: u64, span_slots: u16) {
        for covered in slot..slot + u64::from(span_slots) {
            assert!(!self.is_occupied(covered), "填充记录落在空槽上");
            self.set_occupied(covered, true);
        }
        self.records.insert(slot, AllocationRecord { span_slots, is_released: false, generation: 1, carries_the_quarantine_mark: false });
        self.leaves_with_records.insert(leaf_of_slot(slot));
    }

    fn is_occupied(&self, slot: u64) -> bool {
        self.occupied_slots[usize_of(slot / 64)] & (1u64 << (slot % 64)) != 0
    }

    pub(super) fn set_occupied(&mut self, slot: u64, occupied: bool) {
        let word = &mut self.occupied_slots[usize_of(slot / 64)];
        if occupied {
            *word |= 1u64 << (slot % 64);
        } else {
            *word &= !(1u64 << (slot % 64));
        }
        let segment = segment_of(slot);
        if occupied {
            self.used_slots_per_segment[segment] += 1;
        } else {
            self.used_slots_per_segment[segment] -= 1;
            self.lowest_possibly_free_slot = self.lowest_possibly_free_slot.min(slot);
            if self.used_slots_per_segment[segment] == 0 {
                self.lowest_possibly_empty_segment = self.lowest_possibly_empty_segment.min(segment);
            }
        }
    }

    /// 从 `slot` 起第一个位图上空着的槽（只看位图，不看这次发布已经取走的）。
    pub(super) fn first_unoccupied_slot_from(&self, slot: u64) -> u64 {
        let mut word_index = usize_of(slot / 64);
        let mut word = self.occupied_slots[word_index] | ((1u64 << (slot % 64)) - 1);
        while word == u64::MAX {
            word_index += 1;
            assert!(word_index < self.occupied_slots.len(), "单元区满了：模型的工作负载装不下");
            word = self.occupied_slots[word_index];
        }
        u64_of(word_index) * 64 + u64::from(word.trailing_ones())
    }

    pub(super) fn unit(&self, unit: UnitIdentifier) -> UnitInformation {
        self.units[usize_of(unit.0)].expect("单元号指着一个写过的单元")
    }

    pub(super) fn next_unit(&self) -> u64 {
        u64_of(self.units.len())
    }

    pub(super) fn occupied_slot_count(&self) -> u64 {
        self.occupied_slots.iter().map(|word| u64::from(word.count_ones())).sum()
    }

    pub(super) fn deferred_slot_count(&self) -> u64 {
        self.deferred.values().sum()
    }
}

/// 单元占几槽、按什么对齐取起点：数据单元与码 3 容器两槽 32768 对齐，索引节点一槽。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Footprint {
    OneSlot,
    TwoSlotsAligned,
}

impl Footprint {
    pub(super) fn slots(self) -> u64 {
        match self {
            Footprint::OneSlot => 1,
            Footprint::TwoSlotsAligned => 2,
        }
    }
}

/// 一次发布在分配器上的取落点，只读池状态、自己记这次取走的槽（`allocator.rs` 的 `try_allocate_user_data`、
/// `try_allocate_commit_generated`、`bump_slot_in_open_segment`、`lowest_empty_segment`）：固定点每一轮从同一份池状态重走一遍。
pub(super) struct PlacementWalk<'state> {
    pub(super) state: &'state PoolState,
    pub(super) taken: BTreeSet<u64>,
    pub(super) open_segment_start: Option<u64>,
    pub(super) bump_cursor: u64,
    pub(super) opened_segments: BTreeSet<usize>,
    pub(super) placements_that_fell_back: u64,
}

impl<'state> PlacementWalk<'state> {
    pub(super) fn over(state: &'state PoolState) -> PlacementWalk<'state> {
        PlacementWalk {
            state,
            taken: BTreeSet::new(),
            open_segment_start: state.open_segment_start,
            bump_cursor: state.bump_cursor,
            opened_segments: BTreeSet::new(),
            placements_that_fell_back: 0,
        }
    }

    fn is_free(&self, slot: u64) -> bool {
        slot >= UNIT_AREA_START_SLOT && slot < self.state.slots_per_device && !self.state.is_occupied(slot) && !self.taken.contains(&slot)
    }

    /// 这次挂载开过的聚簇段（含这次发布新开的）：用户数据一个都不许落进去（`allocator.rs` 的 `cluster_segments`）。
    pub(super) fn is_cluster_segment(&self, segment: usize) -> bool {
        self.state.is_cluster_segment[segment] || self.opened_segments.contains(&segment)
    }

    fn take(&mut self, slot: u64, footprint: Footprint) {
        for covered in slot..slot + footprint.slots() {
            assert!(self.is_free(covered), "取的落点 {covered} 已经占着");
            self.taken.insert(covered);
        }
    }

    /// 第一个两槽都空、偶数起点、不在任何聚簇段里的落点（D3（空间分配） 已定项 10 ③ 与 ②）。
    fn lowest_free_pair_outside_the_cluster_segments(&self) -> u64 {
        let mut candidate = self.state.first_unoccupied_slot_from(self.state.lowest_possibly_free_slot);
        loop {
            candidate += candidate % 2;
            assert!(candidate + 1 < self.state.slots_per_device, "单元区满了：模型的工作负载装不下");
            let segment = segment_of(candidate);
            if self.is_cluster_segment(segment) {
                candidate = self.state.first_unoccupied_slot_from(segment_start(segment + 1));
            } else if self.is_free(candidate) && self.is_free(candidate + 1) {
                return candidate;
            } else {
                candidate = self.state.first_unoccupied_slot_from(candidate + 1);
            }
        }
    }

    pub(super) fn allocate_data_unit(&mut self) -> u64 {
        let slot = self.lowest_free_pair_outside_the_cluster_segments();
        self.take(slot, Footprint::TwoSlotsAligned);
        slot
    }

    /// 开放段里从游标起第一个整跨度都空的对齐起点；段里装不下就 `None`。
    fn bump_in_the_open_segment(&self, open_segment_start: u64, footprint: Footprint) -> Option<u64> {
        let segment_end = open_segment_start + CLUSTER_SEGMENT_SLOTS;
        let mut start = self.bump_cursor;
        if footprint == Footprint::TwoSlotsAligned && start % 2 == 1 {
            start += 1;
        }
        while start + footprint.slots() <= segment_end && (start..start + footprint.slots()).any(|slot| !self.is_free(slot)) {
            start += footprint.slots();
        }
        (start + footprint.slots() <= segment_end).then_some(start)
    }

    /// 最低的全空段：段里没有一个占着的槽（defer 里的、隔离的都算占着），这次发布也没在里面取过。
    fn lowest_empty_segment(&self) -> Option<usize> {
        (self.state.lowest_possibly_empty_segment..self.state.used_slots_per_segment.len()).find(|segment| {
            self.state.used_slots_per_segment[*segment] == 0
                && self.taken.range(segment_start(*segment)..segment_start(*segment + 1)).next().is_none()
        })
    }

    pub(super) fn allocate_commit_generated(&mut self, footprint: Footprint) -> u64 {
        if let Some(open_segment_start) = self.open_segment_start {
            if let Some(slot) = self.bump_in_the_open_segment(open_segment_start, footprint) {
                self.bump_cursor = slot + footprint.slots();
                self.take(slot, footprint);
                return slot;
            }
        }
        match self.lowest_empty_segment() {
            Some(segment) => {
                let start = segment_start(segment);
                self.open_segment_start = Some(start);
                self.bump_cursor = start;
                self.opened_segments.insert(segment);
                let slot = self.bump_in_the_open_segment(start, footprint).expect("全空段 64 槽装得下任何一个单元");
                self.bump_cursor = slot + footprint.slots();
                self.take(slot, footprint);
                slot
            }
            None => {
                // 没有全空段：回落，这会儿没有段可 bump（`allocator.rs` 的 `CommitGeneratedDeviceAnswer::FallBack`；产品池级政策先取重写集的叶、不够取空槽最多的叶，D3（空间分配） 已定项 8 第 2 条）；本装置自己的落点仍取槽号最小的空槽，算术不动。
                self.open_segment_start = None;
                self.placements_that_fell_back += 1;
                let mut candidate = self.state.first_unoccupied_slot_from(self.state.lowest_possibly_free_slot);
                while !(candidate.is_multiple_of(footprint.slots()) && (candidate..candidate + footprint.slots()).all(|slot| self.is_free(slot))) {
                    candidate = self.state.first_unoccupied_slot_from(candidate + 1);
                }
                self.take(candidate, footprint);
                candidate
            }
        }
    }

    pub(super) fn force(&mut self, slot: u64, footprint: Footprint) -> u64 {
        self.take(slot, footprint);
        slot
    }
}

/// 设备写次数与字节（两块盘相加；单盘口径 = 字节 ÷ 2，产物另报）。
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(super) struct WriteTally {
    pub(super) device_writes: u64,
    pub(super) bytes_on_both_devices: u64,
}

impl WriteTally {
    pub(super) fn add(&mut self, other: WriteTally) {
        self.device_writes += other.device_writes;
        self.bytes_on_both_devices += other.bytes_on_both_devices;
    }
}

/// 一个单元写到每块盘各一次（`transaction.rs` 的 `WriteUnitToEveryDevice`）。
fn tally_of_a_unit_write(span_slots: u64) -> WriteTally {
    WriteTally { device_writes: DEVICE_COUNT, bytes_on_both_devices: DEVICE_COUNT * span_slots * UNIT_WRITE_BYTES_PER_SLOT }
}

/// 一次发布的写，按结构分；journal、根槽、系统配置槽另列。
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub(super) struct PublishWrites {
    by_structure: [WriteTally; 8],
    pub(super) journal: WriteTally,
    root_slot: WriteTally,
    system_configuration: WriteTally,
    pub(super) journal_records: u64,
    pub(super) last_transaction_named_roles: u64,
    /// 树表多出来的那一条乙的条目（逻辑字节，只当钉值）。
    pub(super) logical_tree_table_bytes_added: u64,
}

impl PublishWrites {
    pub(super) fn add(&mut self, other: &PublishWrites) {
        for (mine, theirs) in self.by_structure.iter_mut().zip(other.by_structure.iter()) {
            mine.add(*theirs);
        }
        self.journal.add(other.journal);
        self.root_slot.add(other.root_slot);
        self.system_configuration.add(other.system_configuration);
        self.journal_records += other.journal_records;
        self.last_transaction_named_roles += other.last_transaction_named_roles;
        self.logical_tree_table_bytes_added += other.logical_tree_table_bytes_added;
    }

    pub(super) fn of(&self, structure: WrittenStructure) -> WriteTally {
        self.by_structure[structure.position()]
    }

    /// 不含根槽与系统配置槽（三臂相同的那两样，登记第一节「额外写」里不计）。
    pub(super) fn without_root_and_system_configuration(&self) -> WriteTally {
        let mut total = self.journal;
        for tally in &self.by_structure {
            total.add(*tally);
        }
        total
    }

    pub(super) fn with_root_and_system_configuration(&self) -> WriteTally {
        let mut total = self.without_root_and_system_configuration();
        total.add(self.root_slot);
        total.add(self.system_configuration);
        total
    }
}

/// 一次发布切成几条 journal 记录（`transaction.rs` 的 `roles_named_by_each_record_of_the_publish`）：写了 N ≥ 1 个数据单元时
/// 前 N − 1 个事务各一条只点名自己的数据单元，最后一个事务点名最后一个数据单元与全部别的重写角色、每条至多 64 个、装满再开；
/// 一个数据单元都没写时一个事务点名全部重写角色，一个都不点名也照样写一条。`padding_named_roles` 只给微历史把点名数垫到登记给的那个数。
/// 交回（记录条数、最后一个事务点名几个）。
pub(super) fn journal_records_of_the_publish(data_units_written: u64, other_rewritten_roles: u64, padding_named_roles: u64) -> (u64, u64) {
    if data_units_written >= 1 {
        let last_transaction_named = 1 + other_rewritten_roles + padding_named_roles;
        (data_units_written - 1 + last_transaction_named.div_ceil(JOURNAL_NAMED_ENTRIES_PER_RECORD), last_transaction_named)
    } else {
        let last_transaction_named = other_rewritten_roles + padding_named_roles;
        (last_transaction_named.div_ceil(JOURNAL_NAMED_ENTRIES_PER_RECORD).max(1), last_transaction_named)
    }
}

pub(super) fn tally_publish_writes(new_units: &[UnitInformation], padding_named_roles: u64, quarantine_tree_registered_by_this_publish: bool) -> PublishWrites {
    let mut writes = PublishWrites::default();
    for unit in new_units.iter() {
        writes.by_structure[unit.structure.position()].add(tally_of_a_unit_write(unit.span_slots));
    }
    let data_units_written = u64_of(new_units.iter().filter(|unit| unit.structure == WrittenStructure::DataUnit).count());
    let other_rewritten_roles = u64_of(new_units.len()) - data_units_written;
    let (journal_records, last_transaction_named_roles) =
        journal_records_of_the_publish(data_units_written, other_rewritten_roles, padding_named_roles);
    writes.journal = WriteTally {
        device_writes: DEVICE_COUNT * journal_records,
        bytes_on_both_devices: DEVICE_COUNT * journal_records * JOURNAL_RECORD_BYTES,
    };
    writes.root_slot = WriteTally {
        device_writes: ROOT_SLOT_WRITES_PER_PUBLISH,
        bytes_on_both_devices: ROOT_SLOT_WRITES_PER_PUBLISH * ROOT_SLOT_BYTES,
    };
    writes.system_configuration = WriteTally {
        device_writes: DEVICE_COUNT,
        bytes_on_both_devices: DEVICE_COUNT * SYSTEM_CONFIGURATION_SLOT_BYTES,
    };
    writes.journal_records = journal_records;
    writes.last_transaction_named_roles = last_transaction_named_roles;
    writes.logical_tree_table_bytes_added = if quarantine_tree_registered_by_this_publish { TREE_TABLE_ENTRY_BYTES } else { 0 };
    writes
}

/// 一次发布要做什么。
#[derive(Clone, Debug)]
pub(super) enum PublishKind {
    /// 新池新建文件那一次：U 个数据单元、extent 树、inode 树全新。只做历史的起点，不计进历史的写量。
    CreateFile { data_unit_count: u32 },
    /// 普通发布：覆盖写这几个数据单元；`quarantine_victim` 是这次释放读盘核没对上的那一个；
    /// `release_quarantined` 只给微历史造一次「释放隔离记录」（`crates/` 今天没有这条路径）。
    Overwrite { data_indices: Vec<u32>, quarantine_victim: Option<u32>, release_quarantined: Option<UnitIdentifier> },
    /// 管理员回退：挂着时的一次向前发布，文件回到目标那一版（`mount.rs` 的 `roll_back_by_a_forward_publish`）。
    RollBack { target_txg: u64 },
}

/// 微历史的构造：指定某个单元落在哪一槽（不走分配器、不挪 bump 游标）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum ForcedPlacement {
    DataUnit { index: u32, slot: u64 },
    NewQuarantineTreeNode { slot: u64 },
}

#[derive(Clone, Debug)]
pub(super) struct PublishRequest {
    pub(super) kind: PublishKind,
    pub(super) forced: Vec<ForcedPlacement>,
}

impl PublishRequest {
    pub(super) fn plain(kind: PublishKind) -> PublishRequest {
        PublishRequest { kind, forced: Vec::new() }
    }

    pub(super) fn forced_data_slot(&self, index: u32) -> Option<u64> {
        self.forced.iter().find_map(|forced| match forced {
            ForcedPlacement::DataUnit { index: forced_index, slot } => (*forced_index == index).then_some(*slot),
            ForcedPlacement::NewQuarantineTreeNode { .. } => None,
        })
    }

    pub(super) fn forced_quarantine_tree_node_slot(&self) -> Option<u64> {
        self.forced.iter().find_map(|forced| match forced {
            ForcedPlacement::DataUnit { .. } => None,
            ForcedPlacement::NewQuarantineTreeNode { slot } => Some(*slot),
        })
    }
}

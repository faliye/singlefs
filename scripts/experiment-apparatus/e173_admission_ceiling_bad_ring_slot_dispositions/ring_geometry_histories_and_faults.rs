//! 根环的槽与几何、各历史事件与历史族、读故障与故障时钟、槽读数与根环表。

use super::*;

pub(super) fn requests_per_mount(geometry: RingGeometry) -> usize {
    usize::try_from(2 * geometry.ring_length_in_txgs() + REQUESTS_PER_MOUNT_BASE).expect("请求数装得进 usize")
}

// ── 根环落点 ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct RingSlot {
    region: u64,
    index_in_region: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RingGeometry {
    pub(super) slots_per_region: u64,
}

impl RingGeometry {
    pub(super) fn ring_length_in_txgs(self) -> u64 {
        REGIONS_IN_THE_ROOT_RING * self.slots_per_region
    }

    /// 落点公式：区域 = txg mod 3，区内槽 = (txg ÷ 3) mod S。
    pub(super) fn slot_of_txg(self, checkpoint_txg: u64) -> RingSlot {
        RingSlot {
            region: checkpoint_txg % REGIONS_IN_THE_ROOT_RING,
            index_in_region: (checkpoint_txg / REGIONS_IN_THE_ROOT_RING) % self.slots_per_region,
        }
    }

    pub(super) fn flat_index(self, slot: RingSlot) -> usize {
        usize::try_from(slot.region * self.slots_per_region + slot.index_in_region).expect("槽号装得进 usize")
    }

    pub(super) fn every_slot(self) -> Vec<RingSlot> {
        let mut slots = Vec::new();
        for region in 0..REGIONS_IN_THE_ROOT_RING {
            for index_in_region in 0..self.slots_per_region {
                slots.push(RingSlot { region, index_in_region });
            }
        }
        slots
    }
}

pub(super) fn device_of_txg(checkpoint_txg: u64) -> usize {
    DEVICE_OF_EACH_REGION[usize::try_from(checkpoint_txg % REGIONS_IN_THE_ROOT_RING).expect("区域号")]
}

/// 抬 F 那一串（准入与卸载共用）的 txg：从现行那一版的下一个 txg 起逐个加一，直到每块盘上都落过一条，至多 R 次（登记第三节 `mount.rs:3362`；登记 A3、B1）。
pub(super) fn txgs_of_the_publishes_carrying_the_floor_to_every_device(current_txg: u64) -> Vec<u64> {
    let mut covered: BTreeSet<usize> = BTreeSet::new();
    let mut txgs: Vec<u64> = Vec::new();
    // 迭代上界 R；跨轮携带已落到的盘与上一个 txg。
    while covered.len() < DEVICES_IN_THE_POOL && txgs.len() < usize::try_from(REGIONS_IN_THE_ROOT_RING).expect("R") {
        let txg = txgs.last().map_or(current_txg, |previous| *previous) + 1;
        covered.insert(device_of_txg(txg));
        txgs.push(txg);
    }
    txgs
}

/// 暖机的 txg：写行那次已经覆盖了它落的那块盘，之后每次空发布加一，推到两块盘都有为止，至多 R 次（D16 已定项 8；登记 B2；`mount.rs:3818`）。
pub(super) fn warm_up_publish_txgs(row_txg: u64) -> Vec<u64> {
    let mut covered: BTreeSet<usize> = BTreeSet::new();
    covered.insert(device_of_txg(row_txg));
    let mut txgs: Vec<u64> = Vec::new();
    // 迭代上界 R；跨轮携带已落到的盘与上一个 txg。
    while covered.len() < DEVICES_IN_THE_POOL && txgs.len() < usize::try_from(REGIONS_IN_THE_ROOT_RING).expect("R") {
        let txg = txgs.last().map_or(row_txg, |previous| *previous) + 1;
        covered.insert(device_of_txg(txg));
        txgs.push(txg);
    }
    txgs
}

// ── 真值盘面与历史 ──────────────────────────────────────────────────────────────────────

/// 根环一个槽里真住着的一条根。`state_number` 是用户可见状态号：覆盖写发布换一个新号，空发布沿用前一版的号；
/// `STATE_WITHOUT_FILE` 是没有文件的那一版（登记第五节「模型·状态」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RootContent {
    pub(super) checkpoint_txg: u64,
    pub(super) state_number: u32,
    pub(super) carried_floor: u64,
}

/// 一次发布放掉的块：释放代 = 那次发布的 txg（登记第五节「模型·一次发布的空间」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Release {
    pub(super) released_at_txg: u64,
    pub(super) blocks: i64,
}

/// 被测挂载之前的历史做完之后真值的样子。
#[derive(Clone, Debug)]
pub(super) struct PoolAfterHistory {
    pub(super) geometry: RingGeometry,
    pub(super) metadata_blocks: i64,
    pub(super) ring: Vec<Option<RootContent>>,
    pub(super) highest_txg_used: u64,
    pub(super) newest_state_number: u32,
    pub(super) effective_floor: u64,
    pub(super) releases: Vec<Release>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HistoryEvent {
    /// 可写挂载：写行一次 + 暖机（登记第五节「历史族」的 M）。
    WritableMount,
    /// 覆盖写发布（O）：新状态号。
    Overwrite,
    /// 空发布（E）。
    EmptyPublish,
    /// 崩在写行根槽之前的一次可写挂载（C）：占一个 txg，根环不写。
    MountCrashedBeforeItsRowRoot,
    /// 正常卸载（U）：推空发布把 F 抬到现行那一版的 txg。
    NormalUnmount,
    /// 抬过 F 而那一串空发布不另建（H2 与 PC4 的做法，登记第五节 H2 行）。
    FloorRaisedWithoutBuildingTheSequence { new_floor: u64 },
}

impl PoolAfterHistory {
    /// mkfs：txg 0 的根，没有文件，F = 0。
    fn after_mkfs(geometry: RingGeometry, metadata_blocks: i64) -> Self {
        let slots = usize::try_from(geometry.ring_length_in_txgs()).expect("环长");
        let mut pool = Self {
            geometry,
            metadata_blocks,
            ring: vec![None; slots],
            highest_txg_used: 0,
            newest_state_number: STATE_WITHOUT_FILE,
            effective_floor: 0,
            releases: Vec::new(),
        };
        let slot = geometry.flat_index(geometry.slot_of_txg(0));
        pool.ring[slot] = Some(RootContent { checkpoint_txg: 0, state_number: STATE_WITHOUT_FILE, carried_floor: 0 });
        pool
    }

    fn write_root(&mut self, checkpoint_txg: u64, state_number: u32, released_blocks: i64) {
        assert_eq!(checkpoint_txg, self.highest_txg_used + 1, "历史里的根 txg 逐个加一（崩溃的挂载占的号也算）");
        let slot = self.geometry.flat_index(self.geometry.slot_of_txg(checkpoint_txg));
        self.ring[slot] = Some(RootContent { checkpoint_txg, state_number, carried_floor: self.effective_floor });
        self.highest_txg_used = checkpoint_txg;
        self.newest_state_number = state_number;
        self.releases.push(Release { released_at_txg: checkpoint_txg, blocks: released_blocks });
    }

    fn apply(&mut self, event: HistoryEvent) {
        let metadata_blocks = self.metadata_blocks;
        match event {
            HistoryEvent::WritableMount => {
                let row_txg = self.highest_txg_used + 1;
                self.write_root(row_txg, self.newest_state_number, metadata_blocks);
                for warm_up_txg in warm_up_publish_txgs(row_txg) {
                    self.write_root(warm_up_txg, self.newest_state_number, metadata_blocks);
                }
            }
            HistoryEvent::Overwrite => {
                let overwrite_txg = self.highest_txg_used + 1;
                self.write_root(overwrite_txg, self.newest_state_number + 1, DATA_BLOCKS_PER_OVERWRITE + metadata_blocks);
            }
            HistoryEvent::EmptyPublish => {
                let empty_txg = self.highest_txg_used + 1;
                self.write_root(empty_txg, self.newest_state_number, metadata_blocks);
            }
            HistoryEvent::MountCrashedBeforeItsRowRoot => {
                self.highest_txg_used += 1;
            }
            HistoryEvent::NormalUnmount => {
                let new_floor = self.highest_txg_used;
                self.effective_floor = new_floor;
                for unmount_txg in txgs_of_the_publishes_carrying_the_floor_to_every_device(new_floor) {
                    self.write_root(unmount_txg, self.newest_state_number, metadata_blocks);
                }
            }
            HistoryEvent::FloorRaisedWithoutBuildingTheSequence { new_floor } => {
                self.effective_floor = new_floor;
            }
        }
    }

    pub(super) fn from_events(geometry: RingGeometry, metadata_blocks: i64, events: &[HistoryEvent]) -> Self {
        let mut pool = Self::after_mkfs(geometry, metadata_blocks);
        for event in events {
            pool.apply(*event);
        }
        pool
    }
}

/// 历史族（登记第五节「历史族」表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum HistoryFamily {
    CrashedAfterOverwrites,
    FloorRaisedAfterOverwrites,
    UnmountedAfterOverwrites,
    OverwritesAndEmptiesAlternating,
    YoungRingWithCrashHoles,
}

impl HistoryFamily {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::CrashedAfterOverwrites => "H1",
            Self::FloorRaisedAfterOverwrites => "H2",
            Self::UnmountedAfterOverwrites => "H3",
            Self::OverwritesAndEmptiesAlternating => "H4",
            Self::YoungRingWithCrashHoles => "H5",
        }
    }

    pub(super) fn events(self, geometry: RingGeometry) -> Vec<HistoryEvent> {
        let ring_length = usize::try_from(geometry.ring_length_in_txgs()).expect("环长");
        let slots_per_region = usize::try_from(geometry.slots_per_region).expect("S");
        let mut events = vec![HistoryEvent::WritableMount];
        match self {
            Self::CrashedAfterOverwrites | Self::FloorRaisedAfterOverwrites | Self::UnmountedAfterOverwrites => {
                events.extend(std::iter::repeat_n(HistoryEvent::Overwrite, 4 * ring_length));
            }
            Self::OverwritesAndEmptiesAlternating => {
                for _ in 0..2 * ring_length {
                    events.push(HistoryEvent::Overwrite);
                    events.push(HistoryEvent::EmptyPublish);
                }
            }
            Self::YoungRingWithCrashHoles => {
                events.extend(std::iter::repeat_n(HistoryEvent::Overwrite, slots_per_region));
                events.extend(std::iter::repeat_n(HistoryEvent::MountCrashedBeforeItsRowRoot, 3));
            }
        }
        match self {
            Self::UnmountedAfterOverwrites => events.push(HistoryEvent::NormalUnmount),
            Self::FloorRaisedAfterOverwrites => {
                // F_生效 = N − 3，N 是这一段历史最后一次覆盖写的 txg（登记第五节 H2 行）。
                let newest_txg = 2 + 4 * geometry.ring_length_in_txgs();
                events.push(HistoryEvent::FloorRaisedWithoutBuildingTheSequence { new_floor: newest_txg - 3 });
            }
            Self::CrashedAfterOverwrites | Self::OverwritesAndEmptiesAlternating | Self::YoungRingWithCrashHoles => {}
        }
        events
    }

    pub(super) fn build_pool_after_history(self, geometry: RingGeometry, metadata_blocks: i64) -> PoolAfterHistory {
        PoolAfterHistory::from_events(geometry, metadata_blocks, &self.events(geometry))
    }
}

// ── 读故障 ───────────────────────────────────────────────────────────────────────────────

/// 读坏三种（登记第一节「读坏」）：设备报读错、读得出而不是全零自证不过、读成功而字节全零。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ReadFaultKind {
    DeviceError,
    GarbageBytes,
    AllZeroBytes,
}

impl ReadFaultKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::DeviceError => "e",
            Self::GarbageBytes => "g",
            Self::AllZeroBytes => "z",
        }
    }
}

/// 读故障的持续方式（登记第五节「读故障的持续方式」），每次读这一槽时判。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum FaultPersistence {
    /// P1：被测挂载里每次读都坏，直到这个进程写了这一槽，之后读得出写进去的内容。
    UntilTheProcessWritesTheSlot,
    /// P2：每次读都坏，写照样成功（坏扇区）。
    EvenAfterTheProcessWritesTheSlot,
    /// T1：从读阶段到暖机做完之间每次读都坏，之后读得好。
    DuringTheMountOnly,
    /// T2：坏到这次挂载第一次抬 F 那一串做完（含它的两遍算上限），之后读得好。
    UntilTheFirstFloorRaiseIsDone,
}

impl FaultPersistence {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::UntilTheProcessWritesTheSlot => "P1",
            Self::EvenAfterTheProcessWritesTheSlot => "P2",
            Self::DuringTheMountOnly => "T1",
            Self::UntilTheFirstFloorRaiseIsDone => "T2",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReadFault {
    pub(super) slot: RingSlot,
    pub(super) kind: ReadFaultKind,
    pub(super) persistence: FaultPersistence,
}

/// 读故障开窗用的时钟：这个进程写过读坏的那一槽没有、暖机做完没有、第一次抬 F 那一串做完没有。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FaultClock {
    pub(super) slot_written_by_the_process: bool,
    pub(super) warm_up_done: bool,
    pub(super) first_floor_raise_done: bool,
}

impl FaultClock {
    pub(super) fn at_the_start_of_the_mount() -> Self {
        Self { slot_written_by_the_process: false, warm_up_done: false, first_floor_raise_done: false }
    }
}

/// 读一个根环槽读到了什么（登记第五节「根环表」：读回全零单分一类）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SlotReading {
    Root(RootContent),
    DeviceError,
    NotSelfVerified,
    AllZero,
}

fn fault_is_active(fault: Option<ReadFault>, clock: FaultClock, slot: RingSlot) -> bool {
    match fault {
        None => false,
        Some(fault) => {
            fault.slot == slot
                && match fault.persistence {
                    FaultPersistence::UntilTheProcessWritesTheSlot => !clock.slot_written_by_the_process,
                    FaultPersistence::EvenAfterTheProcessWritesTheSlot => true,
                    FaultPersistence::DuringTheMountOnly => !clock.warm_up_done,
                    FaultPersistence::UntilTheFirstFloorRaiseIsDone => !clock.first_floor_raise_done,
                }
        }
    }
}

/// 一次读：故障在窗里就按种类坏，否则读真值（从没写过的槽读回全零）。
pub(super) fn read_slot(
    truth_ring: &[Option<RootContent>],
    geometry: RingGeometry,
    fault: Option<ReadFault>,
    clock: FaultClock,
    slot: RingSlot,
) -> SlotReading {
    if fault_is_active(fault, clock, slot) {
        let kind = fault.expect("故障在窗里").kind;
        return match kind {
            ReadFaultKind::DeviceError => SlotReading::DeviceError,
            ReadFaultKind::GarbageBytes => SlotReading::NotSelfVerified,
            ReadFaultKind::AllZeroBytes => SlotReading::AllZero,
        };
    }
    match truth_ring[geometry.flat_index(slot)] {
        Some(root) => SlotReading::Root(root),
        None => SlotReading::AllZero,
    }
}

// ── 根环表（分配器眼里的根环）与进程内重读 ───────────────────────────────────────────

/// 根环表里一个槽的记法（登记第五节「根环表」；冻结树 `allocator.rs:1067`、`1084`）。
/// 读回全零那一类也带一个「能住的最大 txg」：「全零不并」不用它，「全零并」臂拿它封顶（冻结树里没有这一项，登记第五节 Q2 行定义）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Occupant {
    ValidRoot { checkpoint_txg: u64 },
    ValidRootReadWhenRereading { checkpoint_txg: u64 },
    BadAtMount { oldest_txg_it_can_have_held: u64, highest_txg_it_can_hold: u64 },
    AllZeroAtMount { oldest_txg_it_can_have_held: u64, highest_txg_it_can_hold: u64 },
}

#[derive(Clone, Debug)]
pub(super) struct RootRingTable {
    pub(super) occupants: BTreeMap<RingSlot, Occupant>,
    /// 挂载时的 F_生效：进程内回收门槛一直用它（冻结树 `allocator.rs:1932` 取 `occupancy.effective_floor`，抬 F 之后不更新）。
    pub(super) effective_floor_at_the_mount: u64,
    pub(super) latest_recorded_txg: u64,
}

impl RootRingTable {
    /// 环里最旧有效根：有效根的 txg 与钉着的槽（挂载时读坏、读回全零）可能住过的最旧 txg 取最小。
    pub(super) fn oldest_valid_root_in_the_ring(&self) -> Option<u64> {
        self.occupants
            .values()
            .map(|occupant| match occupant {
                Occupant::ValidRoot { checkpoint_txg } | Occupant::ValidRootReadWhenRereading { checkpoint_txg } => *checkpoint_txg,
                Occupant::BadAtMount { oldest_txg_it_can_have_held, .. }
                | Occupant::AllZeroAtMount { oldest_txg_it_can_have_held, .. } => *oldest_txg_it_can_have_held,
            })
            .min()
    }

    /// 这个进程知道住着根的槽：挂载那一刻读得出的、这个进程写过的、进程内重读读出有效根的。
    pub(super) fn knows_a_root_in(&self, slot: RingSlot) -> bool {
        matches!(
            self.occupants.get(&slot),
            Some(Occupant::ValidRoot { .. } | Occupant::ValidRootReadWhenRereading { .. })
        )
    }

    /// 钉着的槽（挂载时读坏、读回全零），各按可能住过的最旧 txg。
    pub(super) fn pinned_oldest_txgs(&self) -> BTreeMap<RingSlot, u64> {
        self.occupants
            .iter()
            .filter_map(|(slot, occupant)| match occupant {
                Occupant::BadAtMount { oldest_txg_it_can_have_held, .. } | Occupant::AllZeroAtMount { oldest_txg_it_can_have_held, .. } => {
                    Some((*slot, *oldest_txg_it_can_have_held))
                }
                Occupant::ValidRoot { .. } | Occupant::ValidRootReadWhenRereading { .. } => None,
            })
            .collect()
    }

    /// 准入抬 F 的上限按它封顶的那个数：挂载时读坏的槽按能住的最大 txg；读回全零的槽只在 `merge_all_zero_slots` 时并。
    pub(super) fn highest_txg_the_cap_takes_from(&self, slot: RingSlot, merge_all_zero_slots: bool) -> Option<u64> {
        match self.occupants.get(&slot) {
            Some(Occupant::BadAtMount { highest_txg_it_can_hold, .. }) => Some(*highest_txg_it_can_hold),
            Some(Occupant::AllZeroAtMount { highest_txg_it_can_hold, .. }) if merge_all_zero_slots => Some(*highest_txg_it_can_hold),
            Some(Occupant::AllZeroAtMount { .. } | Occupant::ValidRoot { .. } | Occupant::ValidRootReadWhenRereading { .. }) | None => None,
        }
    }
}

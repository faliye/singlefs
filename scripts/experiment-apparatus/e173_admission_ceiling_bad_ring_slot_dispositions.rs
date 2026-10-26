//! E173（上限卡在读坏槽的三种处置）：准入抬 F 的上限遇到挂载那一刻读坏或读回全零的根槽时，四条 Q1 臂（照今天、推空B、推空W、不并）与
//! 两条 Q2 臂（全零不并、全零并）各付出什么代价：被拒次数、F 越过的非空根数、推空用掉的空间、回退候选数。
//!
//! 跑前登记 `research/prompts/e173-preregistration.md` 写死了这个装置：**独立手写的计数模型，不与 `crates/` 共用代码**（不是入库装置）。
//! 模型的口径只来自登记第二节抄下的 D16（发布语义）已定项 1、8、9 与 D22「每区槽数 S」那一行，以及登记第三节与第五节写明的「照今天」读法
//! （冻结树 `crates/singlefs-core/src/mount.rs`、`allocator.rs`、`mounted_session.rs` 的行为，只读了出处行，没有共用一行代码）：
//!
//! - 根环：R = 3 个区域、每区 S 个槽，txg t 落区域 t mod 3、区内槽 (t ÷ 3) mod S；区域归属 0 / 1 / 0。
//! - 空间：计数单位是块；覆盖写新占 d + c 块、放掉 d + c 块，空发布新占 c 块、放掉 c 块，放掉的块带释放代 = 这次发布的 txg，进待回收；
//!   每记一条根之后待回收里释放代不高于门槛 max(F_生效, 环里最旧有效根) 的转成空闲；抬 F 那一串回收出来的扣到这一串做完才放开。
//! - 准入：用户发布要求 空闲 − P保留 ≥ d + c；空发布（抬 F 那一串、推空）只要求 空闲 ≥ c；一次准入最多 B = 4 + 2 k_tol = 8 次发布。
//!
//! 子命令：`checks` 跑登记第十一节的对拍与第七节的锚点、第五节的阳性对照；`first` 是第一段（补 Q1）；`second` 是第二段（补 Q2）。

// admission: always 独立手写的计数模型，每次现算；不读 crates/，只读自己的常量
// run-condition: command python3
use e7_index_bench::Emitter;
use std::collections::{BTreeMap, BTreeSet};

// ── 条款与结构常量（出处：登记第二节抄的 D16 已定项 1、8、D22「每区槽数 S」那一行；`crates/singlefs-core/src/root_ring.rs:109` 落点公式） ──

/// 根环区域数 R（D16 已定项 8「至多 R = 3 次」，登记 A5）。
const REGIONS_IN_THE_ROOT_RING: u64 = 3;
/// 三个区域的设备归属 0 / 1 / 0（D16 已定项 8）。
const DEVICE_OF_EACH_REGION: [usize; 3] = [0, 1, 0];
const DEVICES_IN_THE_POOL: usize = 2;
/// k_tol = 2（D16 已定项 1 射程「按 k_tol」，「准入」那一行 B = 4 + 2 k_tol ⇒ 8，登记 A1）。
const TOLERATED_ROOT_SLOT_WRITE_FAILURES: usize = 2;
const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 4 + 2 * TOLERATED_ROOT_SLOT_WRITE_FAILURES;
/// 上限取「第 4 新的非空有效根」：最近 4 个不同状态（D16 已定项 1 定案首句）。
const NEWEST_DISTINCT_STATES_THE_CEILING_KEEPS: usize = 4;
/// 一个文件 64 块，覆盖写每次换 8 块（登记第五节「历史族」）。
const DATA_BLOCKS_PER_OVERWRITE: i64 = 8;
/// S 的取样点（登记第八节）：格式范围 4..16 的两端与 mkfs 取值 8（D22「每区槽数 S」那一行，`crates/singlefs-format/src/lib.rs:270`、`275`、`279`）。
const SLOTS_PER_REGION_CHOICES: [u64; 3] = [4, 8, 16];
/// c_max 的取样点（登记第八节：条款举例的两档）。
const METADATA_BLOCKS_CHOICES: [i64; 2] = [5, 10];
/// 没有文件的那一版（mkfs、写行、暖机）的状态号。
const STATE_WITHOUT_FILE: u32 = 0;
/// 池级读坏的「持久到被测挂载结束」的取样点上限：被测挂载的请求数 J = 2 × 3S + 8（登记第一节）。
const REQUESTS_PER_MOUNT_BASE: u64 = 8;

fn requests_per_mount(geometry: RingGeometry) -> usize {
    usize::try_from(2 * geometry.ring_length_in_txgs() + REQUESTS_PER_MOUNT_BASE).expect("请求数装得进 usize")
}

// ── 根环落点 ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RingSlot {
    region: u64,
    index_in_region: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RingGeometry {
    slots_per_region: u64,
}

impl RingGeometry {
    fn ring_length_in_txgs(self) -> u64 {
        REGIONS_IN_THE_ROOT_RING * self.slots_per_region
    }

    /// 落点公式：区域 = txg mod 3，区内槽 = (txg ÷ 3) mod S。
    fn slot_of_txg(self, checkpoint_txg: u64) -> RingSlot {
        RingSlot {
            region: checkpoint_txg % REGIONS_IN_THE_ROOT_RING,
            index_in_region: (checkpoint_txg / REGIONS_IN_THE_ROOT_RING) % self.slots_per_region,
        }
    }

    fn flat_index(self, slot: RingSlot) -> usize {
        usize::try_from(slot.region * self.slots_per_region + slot.index_in_region).expect("槽号装得进 usize")
    }

    fn every_slot(self) -> Vec<RingSlot> {
        let mut slots = Vec::new();
        for region in 0..REGIONS_IN_THE_ROOT_RING {
            for index_in_region in 0..self.slots_per_region {
                slots.push(RingSlot { region, index_in_region });
            }
        }
        slots
    }
}

fn device_of_txg(checkpoint_txg: u64) -> usize {
    DEVICE_OF_EACH_REGION[usize::try_from(checkpoint_txg % REGIONS_IN_THE_ROOT_RING).expect("区域号")]
}

/// 抬 F 那一串（准入与卸载共用）的 txg：从现行那一版的下一个 txg 起逐个加一，直到每块盘上都落过一条，至多 R 次（登记第三节 `mount.rs:3362`；登记 A3、B1）。
fn txgs_of_the_publishes_carrying_the_floor_to_every_device(current_txg: u64) -> Vec<u64> {
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
fn warm_up_publish_txgs(row_txg: u64) -> Vec<u64> {
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
struct RootContent {
    checkpoint_txg: u64,
    state_number: u32,
    carried_floor: u64,
}

/// 一次发布放掉的块：释放代 = 那次发布的 txg（登记第五节「模型·一次发布的空间」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Release {
    released_at_txg: u64,
    blocks: i64,
}

/// 被测挂载之前的历史做完之后真值的样子。
#[derive(Clone, Debug)]
struct PoolAfterHistory {
    geometry: RingGeometry,
    metadata_blocks: i64,
    ring: Vec<Option<RootContent>>,
    highest_txg_used: u64,
    newest_state_number: u32,
    effective_floor: u64,
    releases: Vec<Release>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HistoryEvent {
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

    fn from_events(geometry: RingGeometry, metadata_blocks: i64, events: &[HistoryEvent]) -> Self {
        let mut pool = Self::after_mkfs(geometry, metadata_blocks);
        for event in events {
            pool.apply(*event);
        }
        pool
    }
}

/// 历史族（登记第五节「历史族」表）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HistoryFamily {
    CrashedAfterOverwrites,
    FloorRaisedAfterOverwrites,
    UnmountedAfterOverwrites,
    OverwritesAndEmptiesAlternating,
    YoungRingWithCrashHoles,
}

impl HistoryFamily {
    fn label(self) -> &'static str {
        match self {
            Self::CrashedAfterOverwrites => "H1",
            Self::FloorRaisedAfterOverwrites => "H2",
            Self::UnmountedAfterOverwrites => "H3",
            Self::OverwritesAndEmptiesAlternating => "H4",
            Self::YoungRingWithCrashHoles => "H5",
        }
    }

    fn events(self, geometry: RingGeometry) -> Vec<HistoryEvent> {
        let ring_length = usize::try_from(geometry.ring_length_in_txgs()).expect("环长");
        let slots_per_region = usize::try_from(geometry.slots_per_region).expect("S");
        let mut events = vec![HistoryEvent::WritableMount];
        match self {
            Self::CrashedAfterOverwrites | Self::FloorRaisedAfterOverwrites | Self::UnmountedAfterOverwrites => {
                events.extend(std::iter::repeat(HistoryEvent::Overwrite).take(4 * ring_length));
            }
            Self::OverwritesAndEmptiesAlternating => {
                for _ in 0..2 * ring_length {
                    events.push(HistoryEvent::Overwrite);
                    events.push(HistoryEvent::EmptyPublish);
                }
            }
            Self::YoungRingWithCrashHoles => {
                events.extend(std::iter::repeat(HistoryEvent::Overwrite).take(slots_per_region));
                events.extend(std::iter::repeat(HistoryEvent::MountCrashedBeforeItsRowRoot).take(3));
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

    fn build_pool_after_history(self, geometry: RingGeometry, metadata_blocks: i64) -> PoolAfterHistory {
        PoolAfterHistory::from_events(geometry, metadata_blocks, &self.events(geometry))
    }
}

// ── 读故障 ───────────────────────────────────────────────────────────────────────────────

/// 读坏三种（登记第一节「读坏」）：设备报读错、读得出而不是全零自证不过、读成功而字节全零。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ReadFaultKind {
    DeviceError,
    GarbageBytes,
    AllZeroBytes,
}

impl ReadFaultKind {
    fn label(self) -> &'static str {
        match self {
            Self::DeviceError => "e",
            Self::GarbageBytes => "g",
            Self::AllZeroBytes => "z",
        }
    }
}

/// 读故障的持续方式（登记第五节「读故障的持续方式」），每次读这一槽时判。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FaultPersistence {
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
    fn label(self) -> &'static str {
        match self {
            Self::UntilTheProcessWritesTheSlot => "P1",
            Self::EvenAfterTheProcessWritesTheSlot => "P2",
            Self::DuringTheMountOnly => "T1",
            Self::UntilTheFirstFloorRaiseIsDone => "T2",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReadFault {
    slot: RingSlot,
    kind: ReadFaultKind,
    persistence: FaultPersistence,
}

/// 读故障开窗用的时钟：这个进程写过读坏的那一槽没有、暖机做完没有、第一次抬 F 那一串做完没有。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FaultClock {
    slot_written_by_the_process: bool,
    warm_up_done: bool,
    first_floor_raise_done: bool,
}

impl FaultClock {
    fn at_the_start_of_the_mount() -> Self {
        Self { slot_written_by_the_process: false, warm_up_done: false, first_floor_raise_done: false }
    }
}

/// 读一个根环槽读到了什么（登记第五节「根环表」：读回全零单分一类）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotReading {
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
fn read_slot(
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
enum Occupant {
    ValidRoot { checkpoint_txg: u64 },
    ValidRootReadWhenRereading { checkpoint_txg: u64 },
    BadAtMount { oldest_txg_it_can_have_held: u64, highest_txg_it_can_hold: u64 },
    AllZeroAtMount { oldest_txg_it_can_have_held: u64, highest_txg_it_can_hold: u64 },
}

#[derive(Clone, Debug)]
struct RootRingTable {
    occupants: BTreeMap<RingSlot, Occupant>,
    /// 挂载时的 F_生效：进程内回收门槛一直用它（冻结树 `allocator.rs:1932` 取 `occupancy.effective_floor`，抬 F 之后不更新）。
    effective_floor_at_the_mount: u64,
    latest_recorded_txg: u64,
}

impl RootRingTable {
    /// 环里最旧有效根：有效根的 txg 与钉着的槽（挂载时读坏、读回全零）可能住过的最旧 txg 取最小。
    fn oldest_valid_root_in_the_ring(&self) -> Option<u64> {
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
    fn knows_a_root_in(&self, slot: RingSlot) -> bool {
        matches!(
            self.occupants.get(&slot),
            Some(Occupant::ValidRoot { .. } | Occupant::ValidRootReadWhenRereading { .. })
        )
    }

    /// 钉着的槽（挂载时读坏、读回全零），各按可能住过的最旧 txg。
    fn pinned_oldest_txgs(&self) -> BTreeMap<RingSlot, u64> {
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
    fn highest_txg_the_cap_takes_from(&self, slot: RingSlot, merge_all_zero_slots: bool) -> Option<u64> {
        match self.occupants.get(&slot) {
            Some(Occupant::BadAtMount { highest_txg_it_can_hold, .. }) => Some(*highest_txg_it_can_hold),
            Some(Occupant::AllZeroAtMount { highest_txg_it_can_hold, .. }) if merge_all_zero_slots => Some(*highest_txg_it_can_hold),
            Some(Occupant::AllZeroAtMount { .. } | Occupant::ValidRoot { .. } | Occupant::ValidRootReadWhenRereading { .. }) | None => None,
        }
    }
}

/// 进程内重读一个钉着的槽读到了什么（冻结树 `allocator.rs:1193` 的四类，被抛弃那一类这个模型不建：单实例）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RereadOfAPinnedSlot {
    ValidRoot { checkpoint_txg: u64 },
    AllZero,
    StillUnreadableOrNotSelfVerified,
}

/// 环里最旧有效根，钉着的槽先重读：有效根与钉着的槽里最小的若是一个钉着的槽，重读那一槽，读出有效根就换成它的 txg，
/// 仍读不出或读回全零才按钉着的 txg 算（冻结树 `allocator.rs:1219` `oldest_txg_in_the_ring_rereading_slots_bad_at_mount`）。
/// 循环：每轮要么换掉一个钉着的槽、要么返回；轮数至多钉着的槽数加一。
fn oldest_txg_rereading_pinned_slots(
    valid_root_txgs: &[u64],
    mut pinned: BTreeMap<RingSlot, u64>,
    reread: &mut dyn FnMut(RingSlot) -> RereadOfAPinnedSlot,
) -> (Option<u64>, Vec<(RingSlot, RereadOfAPinnedSlot)>) {
    let mut valid_root_txgs: Vec<u64> = valid_root_txgs.to_vec();
    let mut rereads = Vec::new();
    loop {
        let oldest_valid_root = valid_root_txgs.iter().copied().min();
        let oldest_pinned_slot = pinned.iter().min_by_key(|(_, pinned_txg)| **pinned_txg).map(|(slot, pinned_txg)| (*slot, *pinned_txg));
        let Some((slot, pinned_txg)) = oldest_pinned_slot else {
            return (oldest_valid_root, rereads);
        };
        if oldest_valid_root.is_some_and(|oldest| oldest < pinned_txg) {
            return (oldest_valid_root, rereads);
        }
        let reread_of_the_slot = reread(slot);
        rereads.push((slot, reread_of_the_slot));
        match reread_of_the_slot {
            RereadOfAPinnedSlot::ValidRoot { checkpoint_txg } => {
                pinned.remove(&slot);
                valid_root_txgs.push(checkpoint_txg);
            }
            RereadOfAPinnedSlot::StillUnreadableOrNotSelfVerified | RereadOfAPinnedSlot::AllZero => {
                return (Some(pinned_txg), rereads);
            }
        }
    }
}

/// 挂载时读坏（或读回全零）的槽各能住哪几个 txg（冻结树 `mount.rs:1774` `txgs_ring_slots_bad_at_mount_can_hold`，登记第三节）：
/// 最大的取 (N − 3S, first_txg − 1] 里落到这一槽的最大 txg，最旧的取 [0, min(3S, first_txg)) 里落到这一槽的最小 txg；两个都没有的槽不交回。
/// 循环：最大 txg 只走区间尾 min(3S, 区间长) 个 txg，最旧 txg 只走 [0, min(3S, first_txg))，跨轮只往按槽的表里写。
fn txgs_ring_slots_bad_at_mount_can_hold(
    bad_slots: &BTreeSet<RingSlot>,
    window_anchor: u64,
    first_txg: u64,
    geometry: RingGeometry,
) -> BTreeMap<RingSlot, (u64, u64)> {
    let ring_length_in_txgs = geometry.ring_length_in_txgs();
    let lowest_txg_inside_the_window = window_anchor.checked_sub(ring_length_in_txgs).map_or(0, |below_the_window| below_the_window + 1);
    let interval_length_in_txgs = first_txg.saturating_sub(lowest_txg_inside_the_window);
    let txgs_walked_at_the_tail = interval_length_in_txgs.min(ring_length_in_txgs);
    let mut highest_txg_of_each_slot: BTreeMap<RingSlot, u64> = BTreeMap::new();
    for txg in first_txg - txgs_walked_at_the_tail..first_txg {
        let slot = geometry.slot_of_txg(txg);
        if bad_slots.contains(&slot) {
            highest_txg_of_each_slot.insert(slot, txg);
        }
    }
    let mut oldest_txg_of_each_slot: BTreeMap<RingSlot, u64> = BTreeMap::new();
    for txg in 0..first_txg.min(ring_length_in_txgs) {
        let slot = geometry.slot_of_txg(txg);
        if bad_slots.contains(&slot) {
            oldest_txg_of_each_slot.entry(slot).or_insert(txg);
        }
    }
    oldest_txg_of_each_slot
        .into_iter()
        .map(|(slot, oldest_txg)| {
            let highest_txg = *highest_txg_of_each_slot.get(&slot).expect("这个槽在 [0, first_txg) 里住得下一个 txg，区间尾也住得下一个");
            (slot, (oldest_txg, highest_txg))
        })
        .collect()
}

// ── 抬 F 的上限（冻结树 `mount.rs:2381` `rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`、`mount.rs:2518`） ──

/// 一次算上限读到的两个数：不并任何槽的 C0；并了读坏的槽之后的 C并（Q2 并时再加读回全零的槽）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CeilingReading {
    unmerged: u64,
    merged: u64,
}

/// 算上限那一遍读到「知道住着根的槽」重读仍坏：抬 F 报自己的错（冻结树 `RollbackFloorCeilingRootRingSlotStillBadAfterOneReread`）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SlotKnownToHoldARootStillBadAfterOneReread {
    slot: RingSlot,
}

/// 第 N 新的非空有效根与「4 个不同状态」：有效根（按 txg 排）里每一条与它前一条有效根的状态不同才算非空，
/// 最旧的一条拿「没有文件」比；从新到旧数，状态号去重（冻结树 `mount.rs:2255`、`2490`）。交回计入的那几条的 txg，从新到旧。
fn newest_first_distinct_non_empty_txgs(valid_roots_sorted_by_txg: &[RootContent]) -> Vec<u64> {
    let mut non_empty_roots: Vec<RootContent> = Vec::new();
    for (position, root) in valid_roots_sorted_by_txg.iter().enumerate() {
        let state_of_the_previous_root = if position == 0 { STATE_WITHOUT_FILE } else { valid_roots_sorted_by_txg[position - 1].state_number };
        if root.state_number != state_of_the_previous_root {
            non_empty_roots.push(*root);
        }
    }
    let mut counted_states: Vec<u32> = Vec::new();
    let mut distinct_txgs = Vec::new();
    for root in non_empty_roots.iter().rev() {
        if counted_states.contains(&root.state_number) {
            continue;
        }
        counted_states.push(root.state_number);
        distinct_txgs.push(root.checkpoint_txg);
    }
    distinct_txgs
}

/// 上限 = min(每块盘上最新的有效根, 第 4 新的不同状态的非空有效根)，去重之后不足 4 个取最旧的有效根（冻结树 `mount.rs:2518`；登记第二节「抬 F 的上限」）。
/// `valid_roots` 是已按「txg ≥ F」滤过的读得出的根。一条有效根都没有交回 `None`。
fn ceiling_of_the_valid_roots(valid_roots: &[RootContent]) -> Option<u64> {
    let mut sorted_by_txg: Vec<RootContent> = valid_roots.to_vec();
    sorted_by_txg.sort_by_key(|root| root.checkpoint_txg);
    let mut newest_per_device: [Option<u64>; DEVICES_IN_THE_POOL] = [None; DEVICES_IN_THE_POOL];
    for root in &sorted_by_txg {
        let device = device_of_txg(root.checkpoint_txg);
        newest_per_device[device] = Some(newest_per_device[device].map_or(root.checkpoint_txg, |newest| newest.max(root.checkpoint_txg)));
    }
    let newest_on_every_device = newest_per_device.iter().flatten().copied().min()?;
    let distinct_non_empty_txgs = newest_first_distinct_non_empty_txgs(&sorted_by_txg);
    let oldest_valid_root = sorted_by_txg.first().map(|root| root.checkpoint_txg);
    let fourth_newest = distinct_non_empty_txgs.get(NEWEST_DISTINCT_STATES_THE_CEILING_KEEPS - 1).copied().or(oldest_valid_root)?;
    Some(newest_on_every_device.min(fourth_newest))
}

/// 真值盘面上的回退候选数 K(F)：真值有效根（txg ≥ F）带的不同带文件状态数（登记第一节「F 越过的状态数」与「回退候选数」）。
fn candidate_states_at_the_floor(truth_ring: &[Option<RootContent>], floor: u64) -> usize {
    let states: BTreeSet<u32> = truth_ring
        .iter()
        .flatten()
        .filter(|root| root.checkpoint_txg >= floor && root.state_number != STATE_WITHOUT_FILE)
        .map(|root| root.state_number)
        .collect();
    states.len()
}

/// 真值盘面上、有效 = txg ≥ F 旧时的真值上限 C真，与 F 新越过它时越过的真值非空根数（登记第一节「F 越过的非空根数」）。
/// 交回 (C真, 越过的非空根数)；F 新不高于 C真 时越过数是 0。
fn truth_ceiling_and_crossed_roots(truth_ring: &[Option<RootContent>], old_floor: u64, new_floor: u64) -> (u64, u32) {
    let mut valid_truth_roots: Vec<RootContent> = truth_ring.iter().flatten().filter(|root| root.checkpoint_txg >= old_floor).copied().collect();
    valid_truth_roots.sort_by_key(|root| root.checkpoint_txg);
    let truth_ceiling = ceiling_of_the_valid_roots(&valid_truth_roots).expect("真值盘面上至少有一条有效根");
    if new_floor <= truth_ceiling {
        return (truth_ceiling, 0);
    }
    let mut crossed_non_empty_roots = 0;
    for (position, root) in valid_truth_roots.iter().enumerate() {
        let state_of_the_previous_root = if position == 0 { STATE_WITHOUT_FILE } else { valid_truth_roots[position - 1].state_number };
        let is_non_empty_in_the_truth = root.state_number != state_of_the_previous_root;
        if is_non_empty_in_the_truth && root.checkpoint_txg >= truth_ceiling && root.checkpoint_txg < new_floor {
            crossed_non_empty_roots += 1;
        }
    }
    (truth_ceiling, crossed_non_empty_roots)
}

// ── 空间：空闲、待回收、扣住 ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReclaimedReuse {
    Immediately,
    HeldUntilTheFloorTakesEffect,
}

/// 块的账：空闲（可发出去的）、扣住（回收了、抬 F 那一串做完才放开的）、待回收（释放代 → 块数）。
#[derive(Clone, Debug)]
struct SpaceBook {
    free_blocks: i64,
    held_blocks: i64,
    pending_by_release_txg: BTreeMap<u64, i64>,
    reclaim_disabled: bool,
}

impl SpaceBook {
    fn record_release(&mut self, released_at_txg: u64, blocks: i64) {
        *self.pending_by_release_txg.entry(released_at_txg).or_insert(0) += blocks;
    }

    /// 已释放 ∧ 释放代 ≤ 门槛 的转成空闲（或扣住）；交回这一次回收的块数。
    fn reclaim_up_to(&mut self, gate: u64, reuse: ReclaimedReuse) -> i64 {
        if self.reclaim_disabled {
            return 0;
        }
        let reclaimable_release_txgs: Vec<u64> = self.pending_by_release_txg.range(..=gate).map(|(release_txg, _)| *release_txg).collect();
        let mut reclaimed_blocks = 0;
        for release_txg in reclaimable_release_txgs {
            reclaimed_blocks += self.pending_by_release_txg.remove(&release_txg).expect("刚列出来的释放代");
        }
        match reuse {
            ReclaimedReuse::Immediately => self.free_blocks += reclaimed_blocks,
            ReclaimedReuse::HeldUntilTheFloorTakesEffect => self.held_blocks += reclaimed_blocks,
        }
        reclaimed_blocks
    }

    fn release_the_holds(&mut self) {
        self.free_blocks += self.held_blocks;
        self.held_blocks = 0;
    }

    fn pending_blocks(&self) -> i64 {
        self.pending_by_release_txg.values().sum()
    }
}

// ── 池的松紧（登记第五节「池」） ────────────────────────────────────────────────────────

/// 保留池的两种读法（登记第五节「准入」：代码读法 c，条款读法 10 + 7 c）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ReserveReading {
    CodeReadingOneEmptyPublish,
    ClauseReadingTenPlusSevenEmptyPublishes,
}

impl ReserveReading {
    fn label(self) -> &'static str {
        match self {
            Self::CodeReadingOneEmptyPublish => "code",
            Self::ClauseReadingTenPlusSevenEmptyPublishes => "clause",
        }
    }

    /// 条款读法：2 × 5 + (B − 1) × c_max（D16 已定项 1「`df`」那一行；登记 A2：c_max = 5 / 10 时 45 / 80）。
    fn reserve_blocks(self, metadata_blocks: i64) -> i64 {
        match self {
            Self::CodeReadingOneEmptyPublish => metadata_blocks,
            Self::ClauseReadingTenPlusSevenEmptyPublishes => {
                2 * 5 + i64::try_from(PUBLISHES_PER_ADMISSION_AT_MOST - 1).expect("B") * metadata_blocks
            }
        }
    }
}

/// 池的松紧：写行与暖机做完那一刻量（登记第五节「池」：近满 k = 0 与 k = 2，宽松按整条请求流不靠回收也发得完）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum PoolFullness {
    NearlyFullWithNoSpareOverwrite,
    NearlyFullWithTwoSpareOverwrites,
    Roomy,
    /// 只给阳性对照 PC1、PC3 用（登记第五节：近满 k = 8），不进扫描。
    NearlyFullWithEightSpareOverwritesForControlsOnly,
}

impl PoolFullness {
    fn label(self) -> &'static str {
        match self {
            Self::NearlyFullWithNoSpareOverwrite => "near0",
            Self::NearlyFullWithTwoSpareOverwrites => "near2",
            Self::NearlyFullWithEightSpareOverwritesForControlsOnly => "near8",
            Self::Roomy => "roomy",
        }
    }

    fn is_nearly_full(self) -> bool {
        match self {
            Self::NearlyFullWithNoSpareOverwrite
            | Self::NearlyFullWithTwoSpareOverwrites
            | Self::NearlyFullWithEightSpareOverwritesForControlsOnly => true,
            Self::Roomy => false,
        }
    }

    fn free_blocks_after_the_warm_up(self, geometry: RingGeometry, metadata_blocks: i64, reserve_blocks: i64) -> i64 {
        let blocks_per_overwrite = DATA_BLOCKS_PER_OVERWRITE + metadata_blocks;
        match self {
            Self::NearlyFullWithNoSpareOverwrite => reserve_blocks,
            Self::NearlyFullWithTwoSpareOverwrites => reserve_blocks + 2 * blocks_per_overwrite,
            Self::NearlyFullWithEightSpareOverwritesForControlsOnly => reserve_blocks + 8 * blocks_per_overwrite,
            Self::Roomy => {
                let requests = i64::try_from(requests_per_mount(geometry)).expect("请求数");
                let ring_length = i64::try_from(geometry.ring_length_in_txgs()).expect("环长");
                reserve_blocks + (requests + ring_length) * blocks_per_overwrite + 100
            }
        }
    }
}

// ── 臂与对照钩子 ─────────────────────────────────────────────────────────────────────────

/// Q1 四条臂（登记第五节「Q1 四条臂」）：上限卡在读坏槽上时怎么办。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum StuckCeilingDisposition {
    AsToday,
    PushEmptyWithinBudget,
    PushEmptyBeyondBudget,
    DoNotMergeTheBadSlots,
}

impl StuckCeilingDisposition {
    fn label(self) -> &'static str {
        match self {
            Self::AsToday => "today",
            Self::PushEmptyWithinBudget => "pushB",
            Self::PushEmptyBeyondBudget => "pushW",
            Self::DoNotMergeTheBadSlots => "nomerge",
        }
    }

    fn every_arm() -> [Self; 4] {
        [Self::AsToday, Self::PushEmptyWithinBudget, Self::PushEmptyBeyondBudget, Self::DoNotMergeTheBadSlots]
    }
}

/// Q2 两条臂（登记第五节「Q2 两条臂」）：读回全零的槽并不并进上限。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum AllZeroSlotDisposition {
    NotMerged,
    Merged,
}

impl AllZeroSlotDisposition {
    fn label(self) -> &'static str {
        match self {
            Self::NotMerged => "zeronot",
            Self::Merged => "zeromerge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Arm {
    stuck_ceiling: StuckCeilingDisposition,
    all_zero_slot: AllZeroSlotDisposition,
}

/// 阳性对照与单测用的钩子（登记第五节「阳性对照」）：种下一个已知效应，看计数器读得出它。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Controls {
    /// PC2：被测挂载里回收一次都不做。
    reclaim_disabled: bool,
    /// PC1：第一次抬 F 把目标改成「它算出的上限 + 这个数」。
    first_raise_target_above_the_ceiling_by: Option<u64>,
    /// PC3：第一次抬 F 把目标改成现行那一版的 txg。
    first_raise_target_is_the_current_txg: bool,
    /// PC4：第一次请求里每一次判够不够都当「不够」。
    first_request_is_judged_short_throughout: bool,
    /// PC4：第一次 `FloorAlreadyAtTheCeiling` 时先推一次空发布再判。
    push_one_empty_publish_on_the_first_floor_already_at_the_ceiling: bool,
    /// 一次准入最多推几次发布（产品值 B）；只有单测拿它找「B 差一」会改变读数的格。
    publishes_per_admission_at_most: usize,
}

impl Controls {
    fn none() -> Self {
        Self {
            reclaim_disabled: false,
            first_raise_target_above_the_ceiling_by: None,
            first_raise_target_is_the_current_txg: false,
            first_request_is_judged_short_throughout: false,
            push_one_empty_publish_on_the_first_floor_already_at_the_ceiling: false,
            publishes_per_admission_at_most: PUBLISHES_PER_ADMISSION_AT_MOST,
        }
    }
}

// ── 一次被测挂载里的池 ──────────────────────────────────────────────────────────────────

/// 可再分配谓词的门槛（D16 已定项 1「已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)」；冻结树 `mount.rs:1409` `reclaim_floor`）。
fn reclaim_floor(effective_floor: u64, oldest_valid_root: Option<u64>) -> u64 {
    oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
}

fn reread_of_a_pinned_slot(
    truth_ring: &[Option<RootContent>],
    geometry: RingGeometry,
    fault: Option<ReadFault>,
    clock: FaultClock,
    slot: RingSlot,
) -> RereadOfAPinnedSlot {
    match read_slot(truth_ring, geometry, fault, clock, slot) {
        SlotReading::Root(root) => RereadOfAPinnedSlot::ValidRoot { checkpoint_txg: root.checkpoint_txg },
        SlotReading::AllZero => RereadOfAPinnedSlot::AllZero,
        SlotReading::DeviceError | SlotReading::NotSelfVerified => RereadOfAPinnedSlot::StillUnreadableOrNotSelfVerified,
    }
}

#[derive(Clone, Debug)]
struct MountedPool {
    geometry: RingGeometry,
    metadata_blocks: i64,
    reserve_blocks: i64,
    truth_ring: Vec<Option<RootContent>>,
    table: RootRingTable,
    space: SpaceBook,
    current_txg: u64,
    current_state_number: u32,
    floor_in_the_current_version: u64,
    fault: Option<ReadFault>,
    clock: FaultClock,
}

impl MountedPool {
    /// 重建分配器（读根环、记根环表），再写行与暖机（登记第五节「模型·根环表」「模型·回收」；冻结树 `mount.rs:1858`）。
    /// 写行与暖机从切换预留出、不动空闲；做完那一刻空闲取 `free_blocks_after_the_warm_up`，待回收是释放代高于这一刻门槛的那些。
    fn mount(
        history: &PoolAfterHistory,
        fault: Option<ReadFault>,
        reserve_blocks: i64,
        free_blocks_after_the_warm_up: i64,
        reclaim_disabled: bool,
    ) -> Self {
        let geometry = history.geometry;
        let first_readings: Vec<(RingSlot, SlotReading)> = geometry
            .every_slot()
            .into_iter()
            .map(|slot| (slot, read_slot(&history.ring, geometry, fault, FaultClock::at_the_start_of_the_mount(), slot)))
            .collect();
        let window_anchor = first_readings
            .iter()
            .filter_map(|(_, reading)| match reading {
                SlotReading::Root(root) => Some(root.checkpoint_txg),
                SlotReading::DeviceError | SlotReading::NotSelfVerified | SlotReading::AllZero => None,
            })
            .max()
            .expect("读阶段至少读出一条有效根");
        let first_txg = history.highest_txg_used + 1;
        let slots_reading = |wanted: fn(&SlotReading) -> bool| -> BTreeSet<RingSlot> {
            first_readings.iter().filter(|(_, reading)| wanted(reading)).map(|(slot, _)| *slot).collect()
        };
        let bad_and_not_all_zero = slots_reading(|reading| matches!(reading, SlotReading::DeviceError | SlotReading::NotSelfVerified));
        let reading_back_all_zero = slots_reading(|reading| matches!(reading, SlotReading::AllZero));
        let mut occupants: BTreeMap<RingSlot, Occupant> = BTreeMap::new();
        for (slot, (oldest, highest)) in txgs_ring_slots_bad_at_mount_can_hold(&bad_and_not_all_zero, window_anchor, first_txg, geometry) {
            occupants.insert(slot, Occupant::BadAtMount { oldest_txg_it_can_have_held: oldest, highest_txg_it_can_hold: highest });
        }
        for (slot, (oldest, highest)) in txgs_ring_slots_bad_at_mount_can_hold(&reading_back_all_zero, window_anchor, first_txg, geometry) {
            occupants.insert(slot, Occupant::AllZeroAtMount { oldest_txg_it_can_have_held: oldest, highest_txg_it_can_hold: highest });
        }
        for (slot, reading) in &first_readings {
            if let SlotReading::Root(root) = reading {
                occupants.insert(*slot, Occupant::ValidRoot { checkpoint_txg: root.checkpoint_txg });
            }
        }
        let mut space = SpaceBook { free_blocks: 0, held_blocks: 0, pending_by_release_txg: BTreeMap::new(), reclaim_disabled };
        for release in &history.releases {
            space.record_release(release.released_at_txg, release.blocks);
        }
        let mut pool = Self {
            geometry,
            metadata_blocks: history.metadata_blocks,
            reserve_blocks,
            truth_ring: history.ring.clone(),
            table: RootRingTable {
                occupants,
                effective_floor_at_the_mount: history.effective_floor,
                latest_recorded_txg: first_txg - 1,
            },
            space,
            current_txg: history.highest_txg_used,
            current_state_number: history.newest_state_number,
            floor_in_the_current_version: history.effective_floor,
            fault,
            clock: FaultClock::at_the_start_of_the_mount(),
        };
        let mount_publish_txgs: Vec<u64> = std::iter::once(first_txg).chain(warm_up_publish_txgs(first_txg)).collect();
        for mount_publish_txg in mount_publish_txgs {
            let state_number = pool.current_state_number;
            let floor = pool.floor_in_the_current_version;
            let metadata_blocks = pool.metadata_blocks;
            pool.publish_root(mount_publish_txg, state_number, 0, metadata_blocks, floor, true);
        }
        pool.clock.warm_up_done = true;
        pool.space.free_blocks = free_blocks_after_the_warm_up;
        pool.space.held_blocks = 0;
        pool
    }

    /// 一次发布的根落盘：占 `placed_blocks`、放掉 `released_blocks`（释放代 = 这次发布的 txg），真值槽换成这条根，记进根环表。
    fn publish_root(&mut self, checkpoint_txg: u64, state_number: u32, placed_blocks: i64, released_blocks: i64, carried_floor: u64, rereading: bool) {
        self.space.free_blocks -= placed_blocks;
        assert!(self.space.free_blocks >= 0, "发布占的块取得到落点：调用方先判过空闲");
        self.space.record_release(checkpoint_txg, released_blocks);
        let written_slot = self.geometry.slot_of_txg(checkpoint_txg);
        let flat = self.geometry.flat_index(written_slot);
        self.truth_ring[flat] = Some(RootContent { checkpoint_txg, state_number, carried_floor });
        if self.fault.is_some_and(|fault| fault.slot == written_slot) {
            self.clock.slot_written_by_the_process = true;
        }
        self.record_root(checkpoint_txg, rereading);
        self.current_txg = checkpoint_txg;
        self.current_state_number = state_number;
    }

    /// 这个进程写了一条根，记进根环表（冻结树 `allocator.rs:1914`、`1966`）：记之前先照进程内重读那一遍重读钉着的槽
    /// （`rereading`，预演不重读），再把这条根记成它那一槽的有效根，按门槛 max(挂载时的 F_生效, 环里最旧有效根) 回收。
    fn record_root(&mut self, checkpoint_txg: u64, rereading: bool) {
        assert_eq!(checkpoint_txg, self.table.latest_recorded_txg + 1, "装了根环表的进程写的根 txg 逐个加一");
        let written_slot = self.geometry.slot_of_txg(checkpoint_txg);
        if rereading {
            let mut valid_root_txgs: Vec<u64> = self
                .table
                .occupants
                .iter()
                .filter(|(slot, _)| **slot != written_slot)
                .filter_map(|(_, occupant)| match occupant {
                    Occupant::ValidRoot { checkpoint_txg: txg } | Occupant::ValidRootReadWhenRereading { checkpoint_txg: txg } => Some(*txg),
                    Occupant::BadAtMount { .. } | Occupant::AllZeroAtMount { .. } => None,
                })
                .collect();
            valid_root_txgs.push(checkpoint_txg);
            let mut pinned = self.table.pinned_oldest_txgs();
            pinned.remove(&written_slot);
            let (truth_ring, geometry, fault, clock) = (&self.truth_ring, self.geometry, self.fault, self.clock);
            let (_oldest, rereads) = oldest_txg_rereading_pinned_slots(&valid_root_txgs, pinned, &mut |slot| {
                reread_of_a_pinned_slot(truth_ring, geometry, fault, clock, slot)
            });
            for (slot, outcome) in rereads {
                match outcome {
                    RereadOfAPinnedSlot::ValidRoot { checkpoint_txg: txg_read } => {
                        self.table.occupants.insert(slot, Occupant::ValidRootReadWhenRereading { checkpoint_txg: txg_read });
                    }
                    RereadOfAPinnedSlot::AllZero | RereadOfAPinnedSlot::StillUnreadableOrNotSelfVerified => {}
                }
            }
        }
        self.table.occupants.insert(written_slot, Occupant::ValidRoot { checkpoint_txg });
        self.table.latest_recorded_txg = checkpoint_txg;
        let gate = reclaim_floor(self.table.effective_floor_at_the_mount, self.table.oldest_valid_root_in_the_ring());
        self.space.reclaim_up_to(gate, ReclaimedReuse::Immediately);
    }

    fn read(&self, slot: RingSlot) -> SlotReading {
        read_slot(&self.truth_ring, self.geometry, self.fault, self.clock, slot)
    }

    /// 算上限那一遍读根环（冻结树 `mount.rs:2273` `roots_of_the_ring_for_the_ceiling`、`mount.rs:2381`）：每槽读一次；读坏而不是「这个进程知道住着根的槽」的，
    /// 记进「这一遍读坏、当没有根」；知道住着根的重读一次，仍坏就是抬 F 报自己的错。
    /// 有效 = 读得出 ∧ txg ≥ 现行那一版带的 F。C0 不并任何槽；C并 并上「这一遍读坏」∩ 根环表里能封顶的槽，只留能住的最大 txg ≥ F 的，取最小者封顶。
    fn ceiling_pass(&self, merge_all_zero_slots: bool) -> Result<CeilingReading, SlotKnownToHoldARootStillBadAfterOneReread> {
        let mut roots_read: Vec<RootContent> = Vec::new();
        let mut bad_slots_taken_as_holding_no_root: BTreeSet<RingSlot> = BTreeSet::new();
        for slot in self.geometry.every_slot() {
            match self.read(slot) {
                SlotReading::Root(root) => roots_read.push(root),
                SlotReading::DeviceError | SlotReading::NotSelfVerified | SlotReading::AllZero => {
                    if !self.table.knows_a_root_in(slot) {
                        bad_slots_taken_as_holding_no_root.insert(slot);
                        continue;
                    }
                    match self.read(slot) {
                        SlotReading::Root(root) => roots_read.push(root),
                        SlotReading::DeviceError | SlotReading::NotSelfVerified | SlotReading::AllZero => {
                            return Err(SlotKnownToHoldARootStillBadAfterOneReread { slot });
                        }
                    }
                }
            }
        }
        let valid_roots: Vec<RootContent> =
            roots_read.into_iter().filter(|root| root.checkpoint_txg >= self.floor_in_the_current_version).collect();
        let unmerged = ceiling_of_the_valid_roots(&valid_roots).expect("读得出的有效根至少一条");
        let lowest_highest_txg_a_bad_slot_can_hold = bad_slots_taken_as_holding_no_root
            .iter()
            .filter_map(|slot| self.table.highest_txg_the_cap_takes_from(*slot, merge_all_zero_slots))
            .filter(|highest_txg| *highest_txg >= self.floor_in_the_current_version)
            .min();
        let merged = lowest_highest_txg_a_bad_slot_can_hold.map_or(unmerged, |highest_txg| unmerged.min(highest_txg));
        Ok(CeilingReading { unmerged, merged })
    }

    /// 抬 F 那一遍的回收门槛里「环里最旧有效根」（冻结树 `mount.rs:3023`）：这一遍读出来的根并上根环表里钉着、这一遍也没在那一槽读出根的槽，
    /// 最小的是钉着的槽时先重读那一槽。只读，不改池。
    fn oldest_valid_root_for_raising_the_floor(&self) -> Option<u64> {
        let roots_read: Vec<RootContent> = self
            .geometry
            .every_slot()
            .into_iter()
            .filter_map(|slot| match self.read(slot) {
                SlotReading::Root(root) => Some(root),
                SlotReading::DeviceError | SlotReading::NotSelfVerified | SlotReading::AllZero => None,
            })
            .collect();
        let valid_root_txgs: Vec<u64> = roots_read.iter().map(|root| root.checkpoint_txg).collect();
        let slots_read: BTreeSet<RingSlot> = roots_read.iter().map(|root| self.geometry.slot_of_txg(root.checkpoint_txg)).collect();
        let pinned: BTreeMap<RingSlot, u64> =
            self.table.pinned_oldest_txgs().into_iter().filter(|(slot, _)| !slots_read.contains(slot)).collect();
        let (truth_ring, geometry, fault, clock) = (&self.truth_ring, self.geometry, self.fault, self.clock);
        oldest_txg_rereading_pinned_slots(&valid_root_txgs, pinned, &mut |slot| reread_of_a_pinned_slot(truth_ring, geometry, fault, clock, slot)).0
    }

    fn free_blocks_above_the_reserve(&self) -> i64 {
        self.space.free_blocks - self.reserve_blocks
    }
}

// ── 一次请求的准入循环与四条臂 ──────────────────────────────────────────────────────────

/// 一次请求被拒的五栏拒因（登记第一节「被拒次数」）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RejectionReason {
    AdmissionBudgetUsedUp,
    FloorAlreadyAtTheCeiling,
    FloorRaiseSequenceFoundNoPlacement,
    EmptyPublishFoundNoPlacement,
    FloorRaiseReportedItsOwnError,
}

const REJECTION_REASON_COUNT: usize = 5;

impl RejectionReason {
    fn index(self) -> usize {
        match self {
            Self::AdmissionBudgetUsedUp => 0,
            Self::FloorAlreadyAtTheCeiling => 1,
            Self::FloorRaiseSequenceFoundNoPlacement => 2,
            Self::EmptyPublishFoundNoPlacement => 3,
            Self::FloorRaiseReportedItsOwnError => 4,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::AdmissionBudgetUsedUp => "budget",
            Self::FloorAlreadyAtTheCeiling => "floor_at_ceiling",
            Self::FloorRaiseSequenceFoundNoPlacement => "raise_no_placement",
            Self::EmptyPublishFoundNoPlacement => "empty_no_placement",
            Self::FloorRaiseReportedItsOwnError => "own_error",
        }
    }
}

enum StepOutcome {
    Pushed,
    Stopped(RejectionReason),
}

/// 一次抬 F 的读数：F 旧 → F 新，真值上限 C真，越过的真值非空根数，少掉的状态数 L。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RaiseRecord {
    old_floor: u64,
    new_floor: u64,
    truth_ceiling: u64,
    crossed_non_empty_roots: u32,
    states_lost: usize,
    publishes: usize,
}

impl RaiseRecord {
    fn floor_is_above_the_truth_ceiling(self) -> bool {
        self.new_floor > self.truth_ceiling
    }
}

/// 一次请求结束时的读数（登记第八节「轨迹」：一步 = 一次请求）。
#[derive(Clone, Debug, PartialEq, Eq)]
struct RequestRecord {
    accepted: bool,
    rejection: Option<RejectionReason>,
    was_judged_short: bool,
    empty_publishes: u32,
    empty_publish_blocks: i64,
    publishes_in_the_admission: usize,
    crossed_non_empty_roots: u32,
    states_lost: usize,
    floor_after: u64,
    free_above_the_reserve_after: i64,
    candidate_states_after: usize,
}

/// 被测挂载的全部读数。
#[derive(Clone, Debug)]
struct MountRun {
    requests: Vec<RequestRecord>,
    raises: Vec<RaiseRecord>,
    candidate_states_at_the_start: usize,
    cap_active_passes: u32,
    cap_blocked_passes: u32,
    empty_publish_refusals: u32,
    raise_refused_after_an_empty_publish: u32,
    empty_publish_bound_exhausted: u32,
    lowest_free_blocks_in_the_first_raise: Option<i64>,
    trace: Vec<String>,
}

/// 一次请求里累计的东西，请求结束时折进 `RequestRecord`。
#[derive(Clone, Copy, Debug)]
struct RequestTally {
    pushed_within_the_budget: usize,
    empty_publishes: u32,
    empty_publish_blocks: i64,
    raise_publishes: usize,
    pushed_an_empty_publish: bool,
}

struct Session {
    pool: MountedPool,
    arm: Arm,
    controls: Controls,
    ledger: MountRun,
    request_index: usize,
    pushed_the_seeded_empty_publish: bool,
    tracing: bool,
}

#[derive(Clone, Copy, Debug)]
struct PoolSettings {
    geometry: RingGeometry,
    metadata_blocks: i64,
    reserve_reading: ReserveReading,
    fullness: PoolFullness,
}

impl PoolSettings {
    fn reserve_blocks(self) -> i64 {
        self.reserve_reading.reserve_blocks(self.metadata_blocks)
    }

    fn free_blocks_after_the_warm_up(self) -> i64 {
        self.fullness.free_blocks_after_the_warm_up(self.geometry, self.metadata_blocks, self.reserve_blocks())
    }
}

impl Session {
    fn merges_all_zero_slots(&self) -> bool {
        self.arm.all_zero_slot == AllZeroSlotDisposition::Merged
    }

    /// 这一臂的准入抬 F 取哪个上限：不并取 C0，其余取并了读坏槽的 C并。
    fn ceiling_of_the_arm(&self, reading: CeilingReading) -> u64 {
        match self.arm.stuck_ceiling {
            StuckCeilingDisposition::DoNotMergeTheBadSlots => reading.unmerged,
            StuckCeilingDisposition::AsToday | StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget => {
                reading.merged
            }
        }
    }

    fn trace_line(&mut self, text: String) {
        if self.tracing {
            self.ledger.trace.push(text);
        }
    }

    /// 封顶起作用与卡住的次数（登记第六节 Q1-6）：不并臂不封顶，不数。
    fn note_the_cap(&mut self, reading: CeilingReading) {
        let (txg, floor, free_blocks) = (self.pool.current_txg, self.pool.floor_in_the_current_version, self.pool.space.free_blocks);
        self.trace_line(format!(
            "ceiling_pass request={} txg={txg} floor={floor} c0={} c_merged={} free_blocks={free_blocks}",
            self.request_index, reading.unmerged, reading.merged
        ));
        if self.arm.stuck_ceiling == StuckCeilingDisposition::DoNotMergeTheBadSlots {
            return;
        }
        if reading.merged < reading.unmerged {
            self.ledger.cap_active_passes += 1;
        }
        if reading.merged <= self.pool.floor_in_the_current_version && reading.unmerged > self.pool.floor_in_the_current_version {
            self.ledger.cap_blocked_passes += 1;
        }
    }

    fn publish_an_empty_root(&mut self) {
        let (txg, state_number, floor, blocks) =
            (self.pool.current_txg + 1, self.pool.current_state_number, self.pool.floor_in_the_current_version, self.pool.metadata_blocks);
        self.pool.publish_root(txg, state_number, blocks, blocks, floor, true);
    }

    /// 推一次不抬 F 的空发布：带现行 F、txg 加一；空间从空闲块里取落点，不判空间准入式子（登记第五节「推空两条臂的空间从哪出」）。
    fn push_one_empty_publish(&mut self, tally: &mut RequestTally) -> Result<(), RejectionReason> {
        // 推空的空发布只要求空闲 ≥ c（取不到落点就被拒），与抬 F 那一串同一个来源。
        if self.pool.space.free_blocks < self.pool.metadata_blocks {
            self.ledger.empty_publish_refusals += 1;
            return Err(RejectionReason::EmptyPublishFoundNoPlacement);
        }
        self.publish_an_empty_root();
        tally.empty_publishes += 1;
        tally.empty_publish_blocks += self.pool.metadata_blocks;
        tally.pushed_an_empty_publish = true;
        Ok(())
    }

    /// 抬 F 那一串（登记第五节「模型·算上限那一遍」与冻结树 `mount.rs:3066`）：第二遍算上限、回收门槛里的环里最旧有效根、
    /// 回收扣到生效、整串预演、真发、放开。`enforce_the_ceiling` 为假时不判「新 F 不高于第二遍上限」（阳性对照把目标抬过上限）。
    fn raise_the_floor_to(&mut self, new_floor: u64, enforce_the_ceiling: bool) -> Result<usize, RejectionReason> {
        let second_reading = self.pool.ceiling_pass(self.merges_all_zero_slots()).map_err(|_| RejectionReason::FloorRaiseReportedItsOwnError)?;
        if enforce_the_ceiling && new_floor > self.ceiling_of_the_arm(second_reading) {
            return Err(RejectionReason::FloorRaiseReportedItsOwnError);
        }
        let oldest_valid_root = self.pool.oldest_valid_root_for_raising_the_floor();
        let planned_txgs = txgs_of_the_publishes_carrying_the_floor_to_every_device(self.pool.current_txg);
        let old_floor = self.pool.floor_in_the_current_version;
        let (truth_ceiling, crossed_non_empty_roots) = truth_ceiling_and_crossed_roots(&self.pool.truth_ring, old_floor, new_floor);
        let candidates_before = candidate_states_at_the_floor(&self.pool.truth_ring, old_floor);
        let candidates_after = candidate_states_at_the_floor(&self.pool.truth_ring, new_floor);
        let states_lost = candidates_before.min(NEWEST_DISTINCT_STATES_THE_CEILING_KEEPS).saturating_sub(candidates_after);
        let space_before_the_raise = self.pool.space.clone();
        self.pool.space.reclaim_up_to(reclaim_floor(new_floor, oldest_valid_root), ReclaimedReuse::HeldUntilTheFloorTakesEffect);
        let mut rehearsal = self.pool.clone();
        for planned_txg in &planned_txgs {
            if rehearsal.space.free_blocks < rehearsal.metadata_blocks {
                self.pool.space = space_before_the_raise;
                return Err(RejectionReason::FloorRaiseSequenceFoundNoPlacement);
            }
            let (state_number, blocks) = (rehearsal.current_state_number, rehearsal.metadata_blocks);
            rehearsal.publish_root(*planned_txg, state_number, blocks, blocks, new_floor, false);
        }
        let mut lowest_free_blocks = self.pool.space.free_blocks;
        for planned_txg in &planned_txgs {
            let (state_number, blocks) = (self.pool.current_state_number, self.pool.metadata_blocks);
            self.pool.publish_root(*planned_txg, state_number, blocks, blocks, new_floor, true);
            lowest_free_blocks = lowest_free_blocks.min(self.pool.space.free_blocks);
        }
        self.pool.space.release_the_holds();
        self.pool.floor_in_the_current_version = new_floor;
        self.pool.clock.first_floor_raise_done = true;
        if self.ledger.raises.is_empty() {
            self.ledger.lowest_free_blocks_in_the_first_raise = Some(lowest_free_blocks);
        }
        self.ledger.raises.push(RaiseRecord {
            old_floor,
            new_floor,
            truth_ceiling,
            crossed_non_empty_roots,
            states_lost,
            publishes: planned_txgs.len(),
        });
        Ok(planned_txgs.len())
    }

    /// 准入里再推一串抬 F（冻结树 `mount.rs:2958` `push_one_floor_raise_within_the_admission_budget`）：先核预算（已推 + 这一串次数 + 1 ≤ B），
    /// 再算上限，不高于现行 F 就交回 `FloorAlreadyAtTheCeiling`，高于就抬到它。`reading_already_taken` 是推空臂在分流之前已经算过的第一遍。
    fn push_one_floor_raise_within_the_budget(
        &mut self,
        tally: &mut RequestTally,
        reading_already_taken: Option<CeilingReading>,
    ) -> StepOutcome {
        let planned_publishes = txgs_of_the_publishes_carrying_the_floor_to_every_device(self.pool.current_txg).len();
        if tally.pushed_within_the_budget + planned_publishes + 1 > self.controls.publishes_per_admission_at_most {
            return StepOutcome::Stopped(RejectionReason::AdmissionBudgetUsedUp);
        }
        let reading = match reading_already_taken {
            Some(reading) => reading,
            None => match self.pool.ceiling_pass(self.merges_all_zero_slots()) {
                Ok(reading) => {
                    self.note_the_cap(reading);
                    reading
                }
                Err(_) => return StepOutcome::Stopped(RejectionReason::FloorRaiseReportedItsOwnError),
            },
        };
        let ceiling = self.ceiling_of_the_arm(reading);
        if ceiling <= self.pool.floor_in_the_current_version {
            return StepOutcome::Stopped(RejectionReason::FloorAlreadyAtTheCeiling);
        }
        let first_raise_of_the_mount = self.ledger.raises.is_empty();
        let (new_floor, enforce_the_ceiling) = if first_raise_of_the_mount && self.controls.first_raise_target_is_the_current_txg {
            (self.pool.current_txg, false)
        } else if let (true, Some(extra)) = (first_raise_of_the_mount, self.controls.first_raise_target_above_the_ceiling_by) {
            (ceiling + extra, false)
        } else {
            (ceiling, true)
        };
        let old_floor = self.pool.floor_in_the_current_version;
        match self.raise_the_floor_to(new_floor, enforce_the_ceiling) {
            Ok(publishes) => {
                tally.pushed_within_the_budget += publishes;
                tally.raise_publishes += publishes;
                self.trace_line(format!("raise {old_floor}->{new_floor} publishes={publishes} ceiling={ceiling}"));
                StepOutcome::Pushed
            }
            Err(RejectionReason::FloorRaiseSequenceFoundNoPlacement) => {
                if tally.pushed_an_empty_publish {
                    self.ledger.raise_refused_after_an_empty_publish += 1;
                }
                StepOutcome::Stopped(RejectionReason::FloorRaiseSequenceFoundNoPlacement)
            }
            Err(reason) => StepOutcome::Stopped(reason),
        }
    }

    /// 推空B：卡住时推一次不抬 F 的空发布，算进这次准入的已推次数；放不下这一次（已推 + 1 + 1 > B）或取不到落点就拒。
    fn push_one_empty_publish_within_the_budget(&mut self, tally: &mut RequestTally) -> StepOutcome {
        if tally.pushed_within_the_budget + 1 + 1 > self.controls.publishes_per_admission_at_most {
            return StepOutcome::Stopped(RejectionReason::AdmissionBudgetUsedUp);
        }
        match self.push_one_empty_publish(tally) {
            Ok(()) => {
                tally.pushed_within_the_budget += 1;
                self.trace_line(format!("empty push (within budget) txg={}", self.pool.current_txg));
                StepOutcome::Pushed
            }
            Err(reason) => StepOutcome::Stopped(reason),
        }
    }

    /// 推空W：推到卡住上限的那几个槽都被写掉为止，推空那几次不受 B 限、不算进已推次数；为了收得住，一次准入推空至多 3S 次。
    fn push_empty_publishes_until_the_ceiling_is_unstuck(&mut self, tally: &mut RequestTally) -> StepOutcome {
        let bound = usize::try_from(self.pool.geometry.ring_length_in_txgs()).expect("环长");
        let mut pushed_here = 0;
        loop {
            let reading = match self.pool.ceiling_pass(self.merges_all_zero_slots()) {
                Ok(reading) => reading,
                Err(_) => return StepOutcome::Stopped(RejectionReason::FloorRaiseReportedItsOwnError),
            };
            let floor = self.pool.floor_in_the_current_version;
            if !(reading.merged <= floor && reading.unmerged > floor) {
                return StepOutcome::Pushed;
            }
            if pushed_here >= bound {
                self.ledger.empty_publish_bound_exhausted += 1;
                return StepOutcome::Stopped(RejectionReason::AdmissionBudgetUsedUp);
            }
            if let Err(reason) = self.push_one_empty_publish(tally) {
                return StepOutcome::Stopped(reason);
            }
            pushed_here += 1;
            self.trace_line(format!("empty push (beyond budget) txg={}", self.pool.current_txg));
        }
    }

    /// 准入不够时按臂处置一步（登记第五节「Q1 四条臂」）。
    fn push_for_space(&mut self, tally: &mut RequestTally) -> StepOutcome {
        match self.arm.stuck_ceiling {
            StuckCeilingDisposition::AsToday | StuckCeilingDisposition::DoNotMergeTheBadSlots => self.push_one_floor_raise_within_the_budget(tally, None),
            StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget => {
                let reading = match self.pool.ceiling_pass(self.merges_all_zero_slots()) {
                    Ok(reading) => reading,
                    Err(_) => return StepOutcome::Stopped(RejectionReason::FloorRaiseReportedItsOwnError),
                };
                self.note_the_cap(reading);
                let floor = self.pool.floor_in_the_current_version;
                if reading.merged > floor {
                    return self.push_one_floor_raise_within_the_budget(tally, Some(reading));
                }
                if reading.unmerged > floor {
                    return match self.arm.stuck_ceiling {
                        StuckCeilingDisposition::PushEmptyBeyondBudget => self.push_empty_publishes_until_the_ceiling_is_unstuck(tally),
                        StuckCeilingDisposition::PushEmptyWithinBudget => self.push_one_empty_publish_within_the_budget(tally),
                        StuckCeilingDisposition::AsToday | StuckCeilingDisposition::DoNotMergeTheBadSlots => {
                            unreachable!("只有推空两臂走到这里")
                        }
                    };
                }
                StepOutcome::Stopped(RejectionReason::FloorAlreadyAtTheCeiling)
            }
        }
    }

    /// 一次用户覆盖写请求的准入循环（冻结树 `mounted_session.rs:423`）：发一次，被拒是空间不够就按臂推一步再判，推满或抬不动就交回。
    /// 循环：每一轮要么发成、要么推一步进下一轮、要么拒；推空B / 照今天 / 不并至多 B − 1 次推，推空W 的推空受 3S 的界；轮数有界。
    fn run_one_request(&mut self) -> RequestRecord {
        let blocks_for_the_overwrite = DATA_BLOCKS_PER_OVERWRITE + self.pool.metadata_blocks;
        let raises_before = self.ledger.raises.len();
        let mut tally = RequestTally {
            pushed_within_the_budget: 0,
            empty_publishes: 0,
            empty_publish_blocks: 0,
            raise_publishes: 0,
            pushed_an_empty_publish: false,
        };
        let judged_short_throughout = self.controls.first_request_is_judged_short_throughout && self.request_index == 0;
        let mut was_judged_short = false;
        let mut rejection: Option<RejectionReason> = None;
        let mut accepted = false;
        let mut rounds = 0;
        while rounds < 1000 {
            rounds += 1;
            let is_enough = !judged_short_throughout && self.pool.free_blocks_above_the_reserve() >= blocks_for_the_overwrite;
            if is_enough {
                let (txg, state_number, floor) =
                    (self.pool.current_txg + 1, self.pool.current_state_number + 1, self.pool.floor_in_the_current_version);
                self.pool.publish_root(txg, state_number, blocks_for_the_overwrite, blocks_for_the_overwrite, floor, true);
                accepted = true;
                break;
            }
            was_judged_short = true;
            match self.push_for_space(&mut tally) {
                StepOutcome::Pushed => {}
                StepOutcome::Stopped(reason) => {
                    if reason == RejectionReason::FloorAlreadyAtTheCeiling
                        && self.controls.push_one_empty_publish_on_the_first_floor_already_at_the_ceiling
                        && !self.pushed_the_seeded_empty_publish
                    {
                        self.pushed_the_seeded_empty_publish = true;
                        self.publish_an_empty_root();
                        tally.empty_publishes += 1;
                        tally.empty_publish_blocks += self.pool.metadata_blocks;
                        tally.pushed_an_empty_publish = true;
                        continue;
                    }
                    rejection = Some(reason);
                    break;
                }
            }
        }
        assert!(accepted || rejection.is_some(), "准入循环在 1000 轮之内收住");
        let raises_in_this_request = &self.ledger.raises[raises_before..];
        let record = RequestRecord {
            accepted,
            rejection,
            was_judged_short,
            empty_publishes: tally.empty_publishes,
            empty_publish_blocks: tally.empty_publish_blocks,
            publishes_in_the_admission: usize::try_from(tally.empty_publishes).expect("次数") + tally.raise_publishes + usize::from(accepted),
            crossed_non_empty_roots: raises_in_this_request.iter().map(|raise| raise.crossed_non_empty_roots).sum(),
            states_lost: raises_in_this_request.iter().map(|raise| raise.states_lost).sum(),
            floor_after: self.pool.floor_in_the_current_version,
            free_above_the_reserve_after: self.pool.free_blocks_above_the_reserve(),
            candidate_states_after: candidate_states_at_the_floor(&self.pool.truth_ring, self.pool.floor_in_the_current_version),
        };
        self.trace_line(format!(
            "request {} accepted={} rejection={} txg={} floor={} free_above_reserve={}",
            self.request_index,
            record.accepted,
            record.rejection.map_or("none", RejectionReason::label),
            self.pool.current_txg,
            record.floor_after,
            record.free_above_the_reserve_after
        ));
        self.request_index += 1;
        record
    }
}

/// 被测挂载：可写挂载（写行 + 暖机），然后 J 次覆盖写请求，不卸载（登记第一节「一次挂载」）。
fn run_a_mount(history: &PoolAfterHistory, fault: Option<ReadFault>, settings: PoolSettings, arm: Arm, controls: Controls, tracing: bool) -> MountRun {
    let mut session = start_a_session(history, fault, settings, arm, controls, tracing);
    for _ in 0..requests_per_mount(settings.geometry) {
        let record = session.run_one_request();
        session.ledger.requests.push(record);
    }
    session.ledger
}

/// 可写挂载做完（写行 + 暖机）、还没发第一次请求的那一刻。
fn start_a_session(history: &PoolAfterHistory, fault: Option<ReadFault>, settings: PoolSettings, arm: Arm, controls: Controls, tracing: bool) -> Session {
    let pool = MountedPool::mount(history, fault, settings.reserve_blocks(), settings.free_blocks_after_the_warm_up(), controls.reclaim_disabled);
    let candidate_states_at_the_start = candidate_states_at_the_floor(&pool.truth_ring, pool.floor_in_the_current_version);
    let ledger = MountRun {
        requests: Vec::new(),
        raises: Vec::new(),
        candidate_states_at_the_start,
        cap_active_passes: 0,
        cap_blocked_passes: 0,
        empty_publish_refusals: 0,
        raise_refused_after_an_empty_publish: 0,
        empty_publish_bound_exhausted: 0,
        lowest_free_blocks_in_the_first_raise: None,
        trace: Vec::new(),
    };
    Session { pool, arm, controls, ledger, request_index: 0, pushed_the_seeded_empty_publish: false, tracing }
}

// ── 读数的聚合：每个位置一组读数，每格按位置聚合 ──────────────────────────────────────

/// 登记第八节「轨迹」要的三样：吃光挂载时初始空闲之后（第一次「不够」之后）的峰值、为正的步数、期末值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Trajectory {
    peak_after_the_first_short: Option<i64>,
    positive_steps: i64,
    final_value: i64,
}

fn trajectory_of(series: &[i64], first_short_step: Option<usize>) -> Trajectory {
    Trajectory {
        peak_after_the_first_short: first_short_step.and_then(|step| series[step..].iter().copied().max()),
        positive_steps: i64::try_from(series.iter().filter(|value| **value > 0).count()).expect("步数"),
        final_value: *series.last().expect("至少一步"),
    }
}

const TRAJECTORY_SERIES: usize = 6;
const TRAJECTORY_NAMES: [&str; TRAJECTORY_SERIES] = ["rejected", "crossed", "lost", "empties", "floor", "free_above_reserve"];

/// 一个位置（一个读坏槽 j）上这一格的全部读数。整数一律 i64；没有的量用 −1 表示「没有」只出现在 `first_extra_step`。
#[derive(Clone, Debug, PartialEq, Eq)]
struct PositionMetrics {
    rejected: i64,
    extra_rejected: i64,
    all_enospc: i64,
    first_extra_step: i64,
    accepted_after_the_first_extra: i64,
    crossed: i64,
    crossed_peak_in_one_raise: i64,
    over_truth_steps: i64,
    lost_total: i64,
    most_states_lost_in_one_raise: i64,
    candidates_lowest: i64,
    candidates_final: i64,
    empties_total: i64,
    empty_blocks_total: i64,
    most_empties_in_one_admission: i64,
    empty_refusals: i64,
    raise_refused_after_an_empty_publish: i64,
    empty_bound_exhausted: i64,
    cap_active_passes: i64,
    cap_blocked_passes: i64,
    most_publishes_in_one_admission: i64,
    raise_sequences: i64,
    rejected_by_reason: [i64; REJECTION_REASON_COUNT],
    trajectories: [Trajectory; TRAJECTORY_SERIES],
}

fn count_as_i64(count: usize) -> i64 {
    i64::try_from(count).expect("计数装得进 i64")
}

fn rejected_count(run: &MountRun) -> usize {
    run.requests.iter().filter(|request| !request.accepted).count()
}

/// `run` 是注了读故障的那一次，`baseline` 是同一格去掉读故障的那一次（盘面、池、请求流逐项相同，登记第一节「因读坏多拒」）。
fn metrics_of(run: &MountRun, baseline: &MountRun) -> PositionMetrics {
    assert_eq!(run.requests.len(), baseline.requests.len(), "同一格的两次请求数相同");
    let rejected = count_as_i64(rejected_count(run));
    let extra_rejected = rejected - count_as_i64(rejected_count(baseline));
    let first_extra_step = run
        .requests
        .iter()
        .zip(&baseline.requests)
        .position(|(with_fault, without_fault)| !with_fault.accepted && without_fault.accepted)
        .or_else(|| if extra_rejected >= 1 { run.requests.iter().position(|request| !request.accepted) } else { None });
    let accepted_after_the_first_extra = first_extra_step.map_or(0, |step| run.requests[step..].iter().filter(|request| request.accepted).count());
    let all_enospc = extra_rejected >= 1 && first_extra_step.is_some() && accepted_after_the_first_extra == 0;
    let mut rejected_by_reason = [0i64; REJECTION_REASON_COUNT];
    for request in &run.requests {
        if let Some(reason) = request.rejection {
            rejected_by_reason[reason.index()] += 1;
        }
    }
    let first_short_step = run.requests.iter().position(|request| request.was_judged_short);
    let series_of = |value_of: &dyn Fn(&RequestRecord) -> i64| -> Vec<i64> { run.requests.iter().map(|request| value_of(request)).collect() };
    let trajectories = [
        trajectory_of(&series_of(&|request| i64::from(!request.accepted)), first_short_step),
        trajectory_of(&series_of(&|request| i64::from(request.crossed_non_empty_roots)), first_short_step),
        trajectory_of(&series_of(&|request| count_as_i64(request.states_lost)), first_short_step),
        trajectory_of(&series_of(&|request| i64::from(request.empty_publishes)), first_short_step),
        trajectory_of(&series_of(&|request| i64::try_from(request.floor_after).expect("F")), first_short_step),
        trajectory_of(&series_of(&|request| request.free_above_the_reserve_after), first_short_step),
    ];
    let candidates_lowest = run.requests.iter().map(|request| request.candidate_states_after).chain(std::iter::once(run.candidate_states_at_the_start)).min().expect("至少一项");
    PositionMetrics {
        rejected,
        extra_rejected,
        all_enospc: i64::from(all_enospc),
        first_extra_step: first_extra_step.map_or(-1, count_as_i64),
        accepted_after_the_first_extra: count_as_i64(accepted_after_the_first_extra),
        crossed: run.raises.iter().map(|raise| i64::from(raise.crossed_non_empty_roots)).sum(),
        crossed_peak_in_one_raise: run.raises.iter().map(|raise| i64::from(raise.crossed_non_empty_roots)).max().unwrap_or(0),
        over_truth_steps: count_as_i64(run.raises.iter().filter(|raise| raise.floor_is_above_the_truth_ceiling()).count()),
        lost_total: run.raises.iter().map(|raise| count_as_i64(raise.states_lost)).sum(),
        most_states_lost_in_one_raise: run.raises.iter().map(|raise| count_as_i64(raise.states_lost)).max().unwrap_or(0),
        candidates_lowest: count_as_i64(candidates_lowest),
        candidates_final: count_as_i64(run.requests.last().expect("至少一次请求").candidate_states_after),
        empties_total: run.requests.iter().map(|request| i64::from(request.empty_publishes)).sum(),
        empty_blocks_total: run.requests.iter().map(|request| request.empty_publish_blocks).sum(),
        most_empties_in_one_admission: run.requests.iter().map(|request| i64::from(request.empty_publishes)).max().unwrap_or(0),
        empty_refusals: i64::from(run.empty_publish_refusals),
        raise_refused_after_an_empty_publish: i64::from(run.raise_refused_after_an_empty_publish),
        empty_bound_exhausted: i64::from(run.empty_publish_bound_exhausted),
        cap_active_passes: i64::from(run.cap_active_passes),
        cap_blocked_passes: i64::from(run.cap_blocked_passes),
        most_publishes_in_one_admission: count_as_i64(run.requests.iter().map(|request| request.publishes_in_the_admission).max().unwrap_or(0)),
        raise_sequences: count_as_i64(run.raises.len()),
        rejected_by_reason,
        trajectories,
    }
}

/// 一格里一个量在全部位置上的聚合：最大、取到最大的位置、最小、量 > 0 的位置数、位置总数（登记第六节开头）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Aggregate {
    maximum: i64,
    maximum_at_position: usize,
    minimum: i64,
    positive_positions: usize,
    positions: usize,
}

impl Aggregate {
    fn empty() -> Self {
        Self { maximum: i64::MIN, maximum_at_position: 0, minimum: i64::MAX, positive_positions: 0, positions: 0 }
    }

    fn add(&mut self, position: usize, value: i64) {
        if value > self.maximum {
            self.maximum = value;
            self.maximum_at_position = position;
        }
        self.minimum = self.minimum.min(value);
        if value > 0 {
            self.positive_positions += 1;
        }
        self.positions += 1;
    }

    fn render(&self) -> String {
        if self.positions == 0 {
            return "none".to_string();
        }
        format!("{}@{}/{}/{}/{}", self.maximum, self.maximum_at_position, self.minimum, self.positive_positions, self.positions)
    }
}

/// 一格里「按位置聚合的轨迹三样」：峰值取最大与最小（没有第一次「不够」的位置数单报）、为正的步数取最大与最小、期末值取最大与最小。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TrajectoryAggregate {
    peak_maximum: Option<i64>,
    peak_minimum: Option<i64>,
    positions_never_short: usize,
    positive_steps_maximum: i64,
    positive_steps_minimum: i64,
    final_maximum: i64,
    final_minimum: i64,
}

impl TrajectoryAggregate {
    fn empty() -> Self {
        Self {
            peak_maximum: None,
            peak_minimum: None,
            positions_never_short: 0,
            positive_steps_maximum: i64::MIN,
            positive_steps_minimum: i64::MAX,
            final_maximum: i64::MIN,
            final_minimum: i64::MAX,
        }
    }

    fn add(&mut self, trajectory: Trajectory) {
        match trajectory.peak_after_the_first_short {
            Some(peak) => {
                self.peak_maximum = Some(self.peak_maximum.map_or(peak, |current| current.max(peak)));
                self.peak_minimum = Some(self.peak_minimum.map_or(peak, |current| current.min(peak)));
            }
            None => self.positions_never_short += 1,
        }
        self.positive_steps_maximum = self.positive_steps_maximum.max(trajectory.positive_steps);
        self.positive_steps_minimum = self.positive_steps_minimum.min(trajectory.positive_steps);
        self.final_maximum = self.final_maximum.max(trajectory.final_value);
        self.final_minimum = self.final_minimum.min(trajectory.final_value);
    }

    fn render(&self) -> String {
        let peak = |value: Option<i64>| value.map_or("none".to_string(), |number| number.to_string());
        format!(
            "peak:{}/{}/never_short={}|positive:{}/{}|final:{}/{}",
            peak(self.peak_maximum),
            peak(self.peak_minimum),
            self.positions_never_short,
            self.positive_steps_maximum,
            self.positive_steps_minimum,
            self.final_maximum,
            self.final_minimum
        )
    }
}

fn quantities_of(metrics: &PositionMetrics) -> Vec<(&'static str, i64)> {
    let mut quantities = vec![
        ("extra_rejected", metrics.extra_rejected),
        ("rejected", metrics.rejected),
        ("all_enospc", metrics.all_enospc),
        ("crossed", metrics.crossed),
        ("crossed_peak", metrics.crossed_peak_in_one_raise),
        ("over_truth", metrics.over_truth_steps),
        ("lost_total", metrics.lost_total),
        ("lost_max", metrics.most_states_lost_in_one_raise),
        ("candidates_lowest", metrics.candidates_lowest),
        ("candidates_final", metrics.candidates_final),
        ("empties", metrics.empties_total),
        ("empty_blocks", metrics.empty_blocks_total),
        ("empties_max_in_admission", metrics.most_empties_in_one_admission),
        ("empty_refusals", metrics.empty_refusals),
        ("raise_refused_after_empty", metrics.raise_refused_after_an_empty_publish),
        ("empty_bound_exhausted", metrics.empty_bound_exhausted),
        ("cap_active", metrics.cap_active_passes),
        ("cap_blocked", metrics.cap_blocked_passes),
        ("publishes_max_in_admission", metrics.most_publishes_in_one_admission),
        ("raise_sequences", metrics.raise_sequences),
    ];
    for reason in REJECTION_REASONS {
        quantities.push((reason.rejected_quantity_name(), metrics.rejected_by_reason[reason.index()]));
    }
    if metrics.first_extra_step >= 0 {
        quantities.push(("first_extra_step", metrics.first_extra_step));
        quantities.push(("accepted_after_first_extra", metrics.accepted_after_the_first_extra));
    }
    quantities
}

const REJECTION_REASONS: [RejectionReason; REJECTION_REASON_COUNT] = [
    RejectionReason::AdmissionBudgetUsedUp,
    RejectionReason::FloorAlreadyAtTheCeiling,
    RejectionReason::FloorRaiseSequenceFoundNoPlacement,
    RejectionReason::EmptyPublishFoundNoPlacement,
    RejectionReason::FloorRaiseReportedItsOwnError,
];

impl RejectionReason {
    fn rejected_quantity_name(self) -> &'static str {
        match self {
            Self::AdmissionBudgetUsedUp => "rej_budget",
            Self::FloorAlreadyAtTheCeiling => "rej_floor_at_ceiling",
            Self::FloorRaiseSequenceFoundNoPlacement => "rej_raise_no_placement",
            Self::EmptyPublishFoundNoPlacement => "rej_empty_no_placement",
            Self::FloorRaiseReportedItsOwnError => "rej_own_error",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CellKey {
    family: HistoryFamily,
    fault_kind: Option<ReadFaultKind>,
    persistence: Option<FaultPersistence>,
    slots_per_region: u64,
    metadata_blocks: i64,
    reserve_reading: ReserveReading,
    fullness: PoolFullness,
    arm: Arm,
}

#[derive(Clone, Debug)]
struct CellSummary {
    key: CellKey,
    positions: usize,
    expected_positions: usize,
    quantities: BTreeMap<&'static str, Aggregate>,
    trajectories: [TrajectoryAggregate; TRAJECTORY_SERIES],
    non_empty_slot_positions: usize,
    crossed_at_non_empty_slot_positions: Aggregate,
    baseline_rejected: i64,
}

impl CellSummary {
    fn new(key: CellKey, expected_positions: usize, baseline_rejected: i64) -> Self {
        Self {
            key,
            positions: 0,
            expected_positions,
            quantities: BTreeMap::new(),
            trajectories: [TrajectoryAggregate::empty(); TRAJECTORY_SERIES],
            non_empty_slot_positions: 0,
            crossed_at_non_empty_slot_positions: Aggregate::empty(),
            baseline_rejected,
        }
    }

    fn add_position(&mut self, position: usize, metrics: &PositionMetrics, slot_holds_a_non_empty_root: bool) {
        for (name, value) in quantities_of(metrics) {
            self.quantities.entry(name).or_insert_with(Aggregate::empty).add(position, value);
        }
        for (series, trajectory) in metrics.trajectories.iter().enumerate() {
            self.trajectories[series].add(*trajectory);
        }
        if slot_holds_a_non_empty_root {
            self.non_empty_slot_positions += 1;
            self.crossed_at_non_empty_slot_positions.add(position, metrics.crossed);
        }
        self.positions += 1;
    }

    fn quantity(&self, name: &str) -> Aggregate {
        self.quantities.get(name).copied().unwrap_or_else(Aggregate::empty)
    }
}

// ── 格与位置的枚举 ──────────────────────────────────────────────────────────────────────

/// 读坏槽 j 在这段历史里的真值是不是一条非空有效根（有效 = txg ≥ F_生效；非空 = 状态与它真值前一条有效根的不同，最旧的拿「没有文件」比）。
fn slot_holds_a_non_empty_valid_root(history: &PoolAfterHistory, root_txg: u64) -> bool {
    let mut valid_roots: Vec<RootContent> = history.ring.iter().flatten().filter(|root| root.checkpoint_txg >= history.effective_floor).copied().collect();
    valid_roots.sort_by_key(|root| root.checkpoint_txg);
    match valid_roots.iter().position(|root| root.checkpoint_txg == root_txg) {
        None => false,
        Some(position) => {
            let state_of_the_previous_root = if position == 0 { STATE_WITHOUT_FILE } else { valid_roots[position - 1].state_number };
            valid_roots[position].state_number != state_of_the_previous_root
        }
    }
}

/// 一格里要扫的读坏位置 j：H1–H4 是 1..3S − 1（N − j 是读坏槽里真住着的那条根的 txg）；H5 不注读故障，只有一个位置（记作 0）。
fn positions_of(family: HistoryFamily, geometry: RingGeometry) -> Vec<usize> {
    match family {
        HistoryFamily::YoungRingWithCrashHoles => vec![0],
        HistoryFamily::CrashedAfterOverwrites
        | HistoryFamily::FloorRaisedAfterOverwrites
        | HistoryFamily::UnmountedAfterOverwrites
        | HistoryFamily::OverwritesAndEmptiesAlternating => {
            (1..usize::try_from(geometry.ring_length_in_txgs()).expect("环长")).collect()
        }
    }
}

/// 同一 Q1 臂下两条 Q2 臂在每个位置上逐项比（登记第六节 Q2-1、Q2-2；F5 自检）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ZeroSlotPairComparison {
    positions: usize,
    identical_positions: usize,
    not_merged_crossed_more: usize,
    merged_crossed_more: usize,
    not_merged_lost_more: usize,
    merged_lost_more: usize,
    largest_crossed_excess_of_not_merged: i64,
    largest_lost_excess_of_not_merged: i64,
    positions_merged_rejects_more: usize,
    largest_rejected_excess_of_merged: i64,
    positions_merged_is_all_enospc_against_not_merged: usize,
}

#[derive(Clone, Debug)]
struct GroupSettings {
    family: HistoryFamily,
    fault_kind: Option<ReadFaultKind>,
    persistence: Option<FaultPersistence>,
    pool: PoolSettings,
    stuck_ceiling: StuckCeilingDisposition,
}

struct GroupResult {
    not_merged: CellSummary,
    merged: CellSummary,
    pair: ZeroSlotPairComparison,
    runs: usize,
    /// 去掉读故障的那一次的逐请求读数：接受与否、F、空闲 − P保留、回退候选数（H1–H4 上各臂必须逐项相同，登记第五节「真实基线」）。
    baseline_signature: Vec<(bool, u64, i64, usize)>,
    baselines_of_the_two_zero_slot_arms_are_identical: bool,
}

fn signature_of_a_run(run: &MountRun) -> Vec<(bool, u64, i64, usize)> {
    run.requests
        .iter()
        .map(|request| (request.accepted, request.floor_after, request.free_above_the_reserve_after, request.candidate_states_after))
        .collect()
}

/// 与「全零不并」那一次比：「全零并」第一次多拒（它拒而不并发成）之后一次请求都没发成。
fn merged_is_all_enospc_against(merged: &MountRun, not_merged: &MountRun) -> bool {
    let first_extra_step = merged.requests.iter().zip(&not_merged.requests).position(|(with_merge, without_merge)| !with_merge.accepted && without_merge.accepted);
    first_extra_step.is_some_and(|step| merged.requests[step..].iter().all(|request| !request.accepted))
}

fn run_a_group(group: &GroupSettings) -> GroupResult {
    let geometry = group.pool.geometry;
    let history = group.family.build_pool_after_history(geometry, group.pool.metadata_blocks);
    let positions = positions_of(group.family, geometry);
    let newest_txg = history.highest_txg_used;
    let arm_of = |zero: AllZeroSlotDisposition| Arm { stuck_ceiling: group.stuck_ceiling, all_zero_slot: zero };
    let key_of = |zero: AllZeroSlotDisposition| CellKey {
        family: group.family,
        fault_kind: group.fault_kind,
        persistence: group.persistence,
        slots_per_region: geometry.slots_per_region,
        metadata_blocks: group.pool.metadata_blocks,
        reserve_reading: group.pool.reserve_reading,
        fullness: group.pool.fullness,
        arm: arm_of(zero),
    };
    let baseline_not_merged = run_a_mount(&history, None, group.pool, arm_of(AllZeroSlotDisposition::NotMerged), Controls::none(), false);
    let baseline_merged = run_a_mount(&history, None, group.pool, arm_of(AllZeroSlotDisposition::Merged), Controls::none(), false);
    let mut not_merged = CellSummary::new(key_of(AllZeroSlotDisposition::NotMerged), positions.len(), count_as_i64(rejected_count(&baseline_not_merged)));
    let mut merged = CellSummary::new(key_of(AllZeroSlotDisposition::Merged), positions.len(), count_as_i64(rejected_count(&baseline_merged)));
    let mut pair = ZeroSlotPairComparison {
        positions: 0,
        identical_positions: 0,
        not_merged_crossed_more: 0,
        merged_crossed_more: 0,
        not_merged_lost_more: 0,
        merged_lost_more: 0,
        largest_crossed_excess_of_not_merged: i64::MIN,
        largest_lost_excess_of_not_merged: i64::MIN,
        positions_merged_rejects_more: 0,
        largest_rejected_excess_of_merged: i64::MIN,
        positions_merged_is_all_enospc_against_not_merged: 0,
    };
    let mut runs = 2;
    for position in &positions {
        let fault = group.fault_kind.zip(group.persistence).map(|(kind, persistence)| ReadFault {
            slot: geometry.slot_of_txg(newest_txg - u64::try_from(*position).expect("位置")),
            kind,
            persistence,
        });
        let holds_a_non_empty_root = fault.is_some() && slot_holds_a_non_empty_valid_root(&history, newest_txg - u64::try_from(*position).expect("位置"));
        let run_not_merged = run_a_mount(&history, fault, group.pool, arm_of(AllZeroSlotDisposition::NotMerged), Controls::none(), false);
        let run_merged = run_a_mount(&history, fault, group.pool, arm_of(AllZeroSlotDisposition::Merged), Controls::none(), false);
        runs += 2;
        let metrics_not_merged = metrics_of(&run_not_merged, &baseline_not_merged);
        let metrics_merged = metrics_of(&run_merged, &baseline_merged);
        not_merged.add_position(*position, &metrics_not_merged, holds_a_non_empty_root);
        merged.add_position(*position, &metrics_merged, holds_a_non_empty_root);
        pair.positions += 1;
        pair.identical_positions += usize::from(metrics_not_merged == metrics_merged);
        pair.not_merged_crossed_more += usize::from(metrics_not_merged.crossed > metrics_merged.crossed);
        pair.merged_crossed_more += usize::from(metrics_merged.crossed > metrics_not_merged.crossed);
        pair.not_merged_lost_more += usize::from(metrics_not_merged.lost_total > metrics_merged.lost_total);
        pair.merged_lost_more += usize::from(metrics_merged.lost_total > metrics_not_merged.lost_total);
        pair.largest_crossed_excess_of_not_merged = pair.largest_crossed_excess_of_not_merged.max(metrics_not_merged.crossed - metrics_merged.crossed);
        pair.largest_lost_excess_of_not_merged = pair.largest_lost_excess_of_not_merged.max(metrics_not_merged.lost_total - metrics_merged.lost_total);
        pair.positions_merged_rejects_more += usize::from(metrics_merged.rejected > metrics_not_merged.rejected);
        pair.largest_rejected_excess_of_merged = pair.largest_rejected_excess_of_merged.max(metrics_merged.rejected - metrics_not_merged.rejected);
        pair.positions_merged_is_all_enospc_against_not_merged += usize::from(merged_is_all_enospc_against(&run_merged, &run_not_merged));
    }
    let baseline_signature = signature_of_a_run(&baseline_not_merged);
    let baselines_of_the_two_zero_slot_arms_are_identical = baseline_signature == signature_of_a_run(&baseline_merged);
    GroupResult { not_merged, merged, pair, runs, baseline_signature, baselines_of_the_two_zero_slot_arms_are_identical }
}

const RESERVE_READINGS: [ReserveReading; 2] =
    [ReserveReading::CodeReadingOneEmptyPublish, ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes];
const POOL_FULLNESSES: [PoolFullness; 3] =
    [PoolFullness::NearlyFullWithNoSpareOverwrite, PoolFullness::NearlyFullWithTwoSpareOverwrites, PoolFullness::Roomy];
const FAULT_PERSISTENCES: [FaultPersistence; 4] = [
    FaultPersistence::UntilTheProcessWritesTheSlot,
    FaultPersistence::EvenAfterTheProcessWritesTheSlot,
    FaultPersistence::DuringTheMountOnly,
    FaultPersistence::UntilTheFirstFloorRaiseIsDone,
];
const FAMILIES_WITH_A_FAULT_TO_INJECT: [HistoryFamily; 4] = [
    HistoryFamily::CrashedAfterOverwrites,
    HistoryFamily::FloorRaisedAfterOverwrites,
    HistoryFamily::UnmountedAfterOverwrites,
    HistoryFamily::OverwritesAndEmptiesAlternating,
];

fn pool_settings_list() -> Vec<PoolSettings> {
    let mut specs = Vec::new();
    for slots_per_region in SLOTS_PER_REGION_CHOICES {
        for metadata_blocks in METADATA_BLOCKS_CHOICES {
            for reserve_reading in RESERVE_READINGS {
                for fullness in POOL_FULLNESSES {
                    specs.push(PoolSettings { geometry: RingGeometry { slots_per_region }, metadata_blocks, reserve_reading, fullness });
                }
            }
        }
    }
    specs
}

/// 第一段（补 Q1）：H1–H4 × e / g × P1 / P2 / T1 / T2 × 三种池 × 四条 Q1 臂，Q2 固定在全零不并（登记第五节「分段」）。
fn groups_of_the_first_segment() -> Vec<GroupSettings> {
    let mut groups = Vec::new();
    for family in FAMILIES_WITH_A_FAULT_TO_INJECT {
        for kind in [ReadFaultKind::DeviceError, ReadFaultKind::GarbageBytes] {
            for persistence in FAULT_PERSISTENCES {
                for pool in pool_settings_list() {
                    for stuck_ceiling in StuckCeilingDisposition::every_arm() {
                        groups.push(GroupSettings { family, fault_kind: Some(kind), persistence: Some(persistence), pool, stuck_ceiling });
                    }
                }
            }
        }
    }
    groups
}

/// 第二段（补 Q2）：z 与 H5 进来，两条 Q2 臂 × 四条 Q1 臂。H5 不注读故障（洞里真是全零）。
fn groups_of_the_second_segment() -> Vec<GroupSettings> {
    let mut groups = Vec::new();
    for family in FAMILIES_WITH_A_FAULT_TO_INJECT {
        for persistence in FAULT_PERSISTENCES {
            for pool in pool_settings_list() {
                for stuck_ceiling in StuckCeilingDisposition::every_arm() {
                    groups.push(GroupSettings {
                        family,
                        fault_kind: Some(ReadFaultKind::AllZeroBytes),
                        persistence: Some(persistence),
                        pool,
                        stuck_ceiling,
                    });
                }
            }
        }
    }
    for pool in pool_settings_list() {
        for stuck_ceiling in StuckCeilingDisposition::every_arm() {
            groups.push(GroupSettings { family: HistoryFamily::YoungRingWithCrashHoles, fault_kind: None, persistence: None, pool, stuck_ceiling });
        }
    }
    groups
}

include!("heavy_threads_default/threads.rs");

/// 线程数只影响挂钟、不影响任何一行输出（每组各算各的，结果按组的次序收）。`SINGLEFS_THREAD_CAP` 没设取登记的重活默认并行度（`.claude/heavy-thread-default`）。
fn worker_thread_count() -> usize {
    default_heavy_threads(include_str!("../../.claude/heavy-thread-default"))
}

fn run_groups_in_parallel(groups: &[GroupSettings]) -> Vec<GroupResult> {
    let next_group = std::sync::atomic::AtomicUsize::new(0);
    let results: Vec<std::sync::Mutex<Option<GroupResult>>> = groups.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|scope| {
        for _ in 0..worker_thread_count().min(groups.len().max(1)) {
            scope.spawn(|| loop {
                let group_index = next_group.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if group_index >= groups.len() {
                    break;
                }
                let result = run_a_group(&groups[group_index]);
                *results[group_index].lock().expect("结果槽没被毒化") = Some(result);
            });
        }
    });
    results.into_iter().map(|slot| slot.into_inner().expect("结果槽没被毒化").expect("每一组都算过")).collect()
}

// ── 产物行 ──────────────────────────────────────────────────────────────────────────────

fn emit(emitter: &mut Emitter, body: String) {
    println!("{}", emitter.emit_raw(&body));
}

fn yes_no(condition: bool) -> &'static str {
    if condition {
        "yes"
    } else {
        "no"
    }
}

fn label_of_the_persistence(persistence: Option<FaultPersistence>) -> &'static str {
    persistence.map_or("none", FaultPersistence::label)
}

fn label_of_the_fault_kind(kind: Option<ReadFaultKind>) -> &'static str {
    kind.map_or("none", ReadFaultKind::label)
}

fn render_the_key(key: &CellKey) -> String {
    format!(
        "family={} kind={} persistence={} S={} c_max={} reserve={} pool={} q1={} q2={}",
        key.family.label(),
        label_of_the_fault_kind(key.fault_kind),
        label_of_the_persistence(key.persistence),
        key.slots_per_region,
        key.metadata_blocks,
        key.reserve_reading.label(),
        key.fullness.label(),
        key.arm.stuck_ceiling.label(),
        key.arm.all_zero_slot.label()
    )
}

fn render_a_cell(segment: &str, summary: &CellSummary) -> String {
    let mut text = format!(
        "name=cell segment={segment} {} positions={} expected_positions={} non_empty_slot_positions={} baseline_rejected={} requests={} crossed_at_non_empty={}",
        render_the_key(&summary.key),
        summary.positions,
        summary.expected_positions,
        summary.non_empty_slot_positions,
        summary.baseline_rejected,
        requests_per_mount(RingGeometry { slots_per_region: summary.key.slots_per_region }),
        summary.crossed_at_non_empty_slot_positions.render(),
    );
    for (name, aggregate) in &summary.quantities {
        text.push_str(&format!(" {name}={}", aggregate.render()));
    }
    for (series, name) in TRAJECTORY_NAMES.iter().enumerate() {
        text.push_str(&format!(" traj_{name}={}", summary.trajectories[series].render()));
    }
    text
}

// ── 判定：每个量各占一行、各报各的判定（登记第六节），按 (臂, S, P保留, c_max) 各判一次（登记第八节「判决不稳定的记法」） ──

/// 作废条款（登记第十一节 V1–V6）与停机条款（装置与 `crates/` 对不上、条款与装置对不上）各记各的。
#[derive(Default)]
struct RunTally {
    void_conditions: Vec<String>,
    halt_conditions: Vec<String>,
    self_checks_passed: usize,
}

impl RunTally {
    fn self_check(&mut self, passed: bool, void_label: &str) {
        if passed {
            self.self_checks_passed += 1;
        } else {
            self.void_conditions.push(void_label.to_string());
        }
    }

    fn halt_check(&mut self, passed: bool, halt_label: &str) {
        if passed {
            self.self_checks_passed += 1;
        } else {
            self.halt_conditions.push(halt_label.to_string());
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PointKey {
    arm: &'static str,
    slots_per_region: u64,
    reserve: &'static str,
    metadata_blocks: i64,
}

#[derive(Clone, Copy, Debug, Default)]
struct PointTotals {
    observed_maximum: Option<i64>,
    cells_hit: usize,
    cells: usize,
    cells_not_applicable: usize,
    cells_with_path_evidence: usize,
}

/// 一条判定：谁的值、门槛 1（数 ≥ 1 才算打中）、路径证据（F6：阴性结果要能和「代码没跑到」分开）。
struct JudgmentSettings<'settings> {
    judgment_label: &'static str,
    scope: &'static str,
    value: &'settings dyn Fn(&CellSummary) -> Option<i64>,
    path_evidence: &'settings dyn Fn(&CellSummary) -> i64,
}

fn collect_the_points(cells: &[&CellSummary], settings: &JudgmentSettings<'_>) -> BTreeMap<PointKey, PointTotals> {
    let mut points: BTreeMap<PointKey, PointTotals> = BTreeMap::new();
    for cell in cells {
        let key = PointKey {
            arm: cell.key.arm.stuck_ceiling.label(),
            slots_per_region: cell.key.slots_per_region,
            reserve: cell.key.reserve_reading.label(),
            metadata_blocks: cell.key.metadata_blocks,
        };
        let totals = points.entry(key).or_default();
        totals.cells += 1;
        if (settings.path_evidence)(cell) > 0 {
            totals.cells_with_path_evidence += 1;
        }
        match (settings.value)(cell) {
            None => totals.cells_not_applicable += 1,
            Some(value) => {
                totals.observed_maximum = Some(totals.observed_maximum.map_or(value, |current| current.max(value)));
                if value >= 1 {
                    totals.cells_hit += 1;
                }
            }
        }
    }
    points
}

/// 发这一条判定的各点与每条臂的稳定性行；交回 (打中的点数, 点数) 按臂分的表，供判别力自证用。
fn emit_a_judgment(emitter: &mut Emitter, segment: &str, cells: &[&CellSummary], settings: &JudgmentSettings<'_>) -> BTreeMap<&'static str, (usize, usize, Option<i64>)> {
    let points = collect_the_points(cells, settings);
    let mut per_arm: BTreeMap<&'static str, (usize, usize, Option<i64>)> = BTreeMap::new();
    for (key, totals) in &points {
        let hit = totals.observed_maximum.is_some_and(|value| value >= 1);
        let no_hit_claim = if hit {
            "not_applicable"
        } else if totals.cells_with_path_evidence == 0 {
            "path_not_exercised"
        } else {
            "path_exercised"
        };
        emit(
            emitter,
            format!(
                "name=judgment segment={segment} id={} scope={} arm={} S={} reserve={} c_max={} observed_maximum={} cells_hit={} cells={} cells_not_applicable={} cells_with_path_evidence={} hit={} no_hit_claim={no_hit_claim}",
                settings.judgment_label,
                settings.scope,
                key.arm,
                key.slots_per_region,
                key.reserve,
                key.metadata_blocks,
                totals.observed_maximum.map_or("none".to_string(), |value| value.to_string()),
                totals.cells_hit,
                totals.cells,
                totals.cells_not_applicable,
                totals.cells_with_path_evidence,
                yes_no(hit)
            ),
        );
        let entry = per_arm.entry(key.arm).or_insert((0, 0, None));
        entry.0 += usize::from(hit);
        entry.1 += 1;
        entry.2 = match (entry.2, totals.observed_maximum) {
            (Some(left), Some(right)) => Some(left.max(right)),
            (Some(left), None) => Some(left),
            (None, right) => right,
        };
    }
    for (arm, (hit_points, total_points, observed_maximum)) in &per_arm {
        let stability = if *hit_points == *total_points {
            "stable_hit"
        } else if *hit_points == 0 {
            "stable_no_hit"
        } else {
            "unstable"
        };
        emit(
            emitter,
            format!(
                "name=stability segment={segment} id={} scope={} arm={arm} hit_points={hit_points} points={total_points} observed_maximum={} stability={stability}",
                settings.judgment_label,
                settings.scope,
                observed_maximum.map_or("none".to_string(), |value| value.to_string())
            ),
        );
    }
    per_arm
}

/// 判别力自证（登记第八节）：把门槛从 1 挪到「观测到的最大值 + 1」，判定必须由真转假；观测到的最大值是 0 的判定本来就是假，不适用。
fn emit_the_threshold_shift_check(emitter: &mut Emitter, tally: &mut RunTally, segment: &str, judgment_label: &str, per_arm: &BTreeMap<&'static str, (usize, usize, Option<i64>)>) {
    for (arm, (_, _, observed_maximum)) in per_arm {
        let Some(observed_maximum) = observed_maximum else {
            continue;
        };
        let hit_at_the_original_threshold = *observed_maximum >= 1;
        let shifted_threshold = observed_maximum + 1;
        let hit_at_the_shifted_threshold = *observed_maximum >= shifted_threshold;
        let applicable = hit_at_the_original_threshold;
        if applicable {
            tally.self_check(!hit_at_the_shifted_threshold, "threshold_shift_does_not_flip");
        }
        emit(
            emitter,
            format!(
                "name=threshold_shift segment={segment} id={judgment_label} arm={arm} observed_maximum={observed_maximum} hit_at_threshold_1={} shifted_threshold={shifted_threshold} hit_at_shifted_threshold={} flips={}",
                yes_no(hit_at_the_original_threshold),
                yes_no(hit_at_the_shifted_threshold),
                if applicable { yes_no(hit_at_the_original_threshold && !hit_at_the_shifted_threshold) } else { "not_applicable_observed_below_1" }
            ),
        );
    }
}

fn judge(
    emitter: &mut Emitter,
    tally: &mut RunTally,
    segment: &str,
    judgment_label: &'static str,
    scope: &'static str,
    cells: &[&CellSummary],
    value: &dyn Fn(&CellSummary) -> Option<i64>,
    path_evidence: &dyn Fn(&CellSummary) -> i64,
) {
    let settings = JudgmentSettings { judgment_label, scope, value, path_evidence };
    let per_arm = emit_a_judgment(emitter, segment, cells, &settings);
    emit_the_threshold_shift_check(emitter, tally, segment, judgment_label, &per_arm);
}

fn maximum_of(cell: &CellSummary, quantity_name: &str) -> i64 {
    let aggregate = cell.quantity(quantity_name);
    if aggregate.positions == 0 {
        0
    } else {
        aggregate.maximum
    }
}

fn positive_positions_of(cell: &CellSummary, quantity_name: &str) -> i64 {
    count_as_i64(cell.quantity(quantity_name).positive_positions)
}

fn crossed_at_non_empty_slots(cell: &CellSummary) -> Option<i64> {
    if cell.non_empty_slot_positions == 0 {
        None
    } else {
        Some(cell.crossed_at_non_empty_slot_positions.maximum)
    }
}

fn is_a_persistent_fault(cell: &CellSummary) -> bool {
    matches!(cell.key.persistence, Some(FaultPersistence::UntilTheProcessWritesTheSlot | FaultPersistence::EvenAfterTheProcessWritesTheSlot))
}

fn is_a_transient_fault(cell: &CellSummary) -> bool {
    matches!(cell.key.persistence, Some(FaultPersistence::DuringTheMountOnly | FaultPersistence::UntilTheFirstFloorRaiseIsDone))
}

fn is_an_empty_pushing_arm(cell: &CellSummary) -> bool {
    matches!(cell.key.arm.stuck_ceiling, StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget)
}

fn admission_path_evidence(cell: &CellSummary) -> i64 {
    maximum_of(cell, "raise_sequences").max(maximum_of(cell, "cap_blocked")).max(maximum_of(cell, "empties"))
}

/// 第一段的判定：Q1-1 到 Q1-6（登记第六节）。
fn emit_the_first_segment_judgments(emitter: &mut Emitter, tally: &mut RunTally, results: &[GroupResult]) {
    let segment = "first";
    let cells: Vec<&CellSummary> = results.iter().map(|result| &result.not_merged).collect();
    let select = |keep: &dyn Fn(&CellSummary) -> bool| -> Vec<&CellSummary> { cells.iter().copied().filter(|cell| keep(cell)).collect() };
    let persistent_near_full = select(&|cell| is_a_persistent_fault(cell) && cell.key.fullness.is_nearly_full());
    let transient_near_full = select(&|cell| is_a_transient_fault(cell) && cell.key.fullness.is_nearly_full());
    let transient_any_pool = select(&|cell| is_a_transient_fault(cell));
    let roomy = select(&|cell| cell.key.fullness == PoolFullness::Roomy);
    let pushing_near_full = select(&|cell| is_an_empty_pushing_arm(cell) && cell.key.fullness.is_nearly_full());
    let everything = select(&|_| true);
    let extra_rejected = |cell: &CellSummary| Some(maximum_of(cell, "extra_rejected"));
    let crossed_non_empty = |cell: &CellSummary| crossed_at_non_empty_slots(cell);
    let non_empty_slots = |cell: &CellSummary| count_as_i64(cell.non_empty_slot_positions);
    judge(emitter, tally, segment, "Q1-1", "persistent_near_full", &persistent_near_full, &extra_rejected, &admission_path_evidence);
    judge(emitter, tally, segment, "Q1-1b", "persistent_near_full", &persistent_near_full, &|cell| Some(positive_positions_of(cell, "all_enospc")), &admission_path_evidence);
    judge(emitter, tally, segment, "Q1-2", "every_persistence_every_pool", &everything, &crossed_non_empty, &non_empty_slots);
    judge(emitter, tally, segment, "Q1-2b", "every_persistence_every_pool", &everything, &|cell| Some(maximum_of(cell, "over_truth")), &|cell| maximum_of(cell, "raise_sequences"));
    judge(emitter, tally, segment, "Q1-3", "empty_pushing_arms_near_full", &pushing_near_full, &|cell| Some(maximum_of(cell, "empty_refusals")), &|cell| maximum_of(cell, "empties"));
    judge(emitter, tally, segment, "Q1-3-raise-refused-after-empty", "empty_pushing_arms_near_full", &pushing_near_full, &|cell| Some(maximum_of(cell, "raise_refused_after_empty")), &|cell| maximum_of(cell, "empties"));
    judge(emitter, tally, segment, "Q1-3-empties-in-one-admission", "empty_pushing_arms_near_full", &pushing_near_full, &|cell| Some(maximum_of(cell, "empties_max_in_admission")), &|cell| maximum_of(cell, "empties"));
    judge(emitter, tally, segment, "Q1-3-empties-in-one-mount", "empty_pushing_arms_near_full", &pushing_near_full, &|cell| Some(maximum_of(cell, "empties")), &|cell| maximum_of(cell, "empties"));
    judge(emitter, tally, segment, "Q1-3-empty-blocks-in-one-mount", "empty_pushing_arms_near_full", &pushing_near_full, &|cell| Some(maximum_of(cell, "empty_blocks")), &|cell| maximum_of(cell, "empties"));
    judge(emitter, tally, segment, "Q1-4-extra-rejected", "transient_near_full", &transient_near_full, &extra_rejected, &admission_path_evidence);
    judge(emitter, tally, segment, "Q1-4-crossed", "transient_every_pool", &transient_any_pool, &crossed_non_empty, &non_empty_slots);
    judge(emitter, tally, segment, "Q1-5-extra-rejected", "roomy", &roomy, &extra_rejected, &admission_path_evidence);
    judge(emitter, tally, segment, "Q1-5-crossed", "roomy", &roomy, &crossed_non_empty, &non_empty_slots);
    judge(emitter, tally, segment, "Q1-6-cap-active", "every_persistence_every_pool", &everything, &|cell| Some(maximum_of(cell, "cap_active")), &|cell| maximum_of(cell, "raise_sequences"));
    judge(emitter, tally, segment, "Q1-6-cap-blocked", "every_persistence_every_pool", &everything, &|cell| Some(maximum_of(cell, "cap_blocked")), &|cell| maximum_of(cell, "raise_sequences"));
}

#[derive(Clone, Copy, Debug)]
struct ZeroSlotTotals {
    cells: usize,
    cells_where_not_merged_has_more: usize,
    cells_where_merged_has_more: usize,
    largest_crossed_excess_of_not_merged: i64,
    largest_lost_excess_of_not_merged: i64,
    crossed_maximum_not_merged: i64,
    crossed_maximum_merged: i64,
    lost_maximum_not_merged: i64,
    lost_maximum_merged: i64,
    all_enospc_positions_not_merged: usize,
    all_enospc_positions_merged: usize,
    extra_rejected_maximum_not_merged: i64,
    extra_rejected_maximum_merged: i64,
    positions_merged_is_all_enospc_against_not_merged: usize,
    positions_merged_rejects_more: usize,
    largest_rejected_excess_of_merged: i64,
    lowest_candidates_not_merged: i64,
    lowest_candidates_merged: i64,
    lowest_final_candidates_not_merged: i64,
    lowest_final_candidates_merged: i64,
}

impl ZeroSlotTotals {
    fn start() -> Self {
        Self {
            cells: 0,
            cells_where_not_merged_has_more: 0,
            cells_where_merged_has_more: 0,
            largest_crossed_excess_of_not_merged: i64::MIN,
            largest_lost_excess_of_not_merged: i64::MIN,
            crossed_maximum_not_merged: 0,
            crossed_maximum_merged: 0,
            lost_maximum_not_merged: 0,
            lost_maximum_merged: 0,
            all_enospc_positions_not_merged: 0,
            all_enospc_positions_merged: 0,
            extra_rejected_maximum_not_merged: i64::MIN,
            extra_rejected_maximum_merged: i64::MIN,
            positions_merged_is_all_enospc_against_not_merged: 0,
            positions_merged_rejects_more: 0,
            largest_rejected_excess_of_merged: i64::MIN,
            lowest_candidates_not_merged: i64::MAX,
            lowest_candidates_merged: i64::MAX,
            lowest_final_candidates_not_merged: i64::MAX,
            lowest_final_candidates_merged: i64::MAX,
        }
    }

    fn add(&mut self, result: &GroupResult) {
        let (not_merged, merged, pair) = (&result.not_merged, &result.merged, &result.pair);
        self.cells += 1;
        self.cells_where_not_merged_has_more += usize::from(pair.not_merged_crossed_more + pair.not_merged_lost_more > 0);
        self.cells_where_merged_has_more += usize::from(pair.merged_crossed_more + pair.merged_lost_more > 0);
        self.largest_crossed_excess_of_not_merged = self.largest_crossed_excess_of_not_merged.max(pair.largest_crossed_excess_of_not_merged);
        self.largest_lost_excess_of_not_merged = self.largest_lost_excess_of_not_merged.max(pair.largest_lost_excess_of_not_merged);
        self.crossed_maximum_not_merged = self.crossed_maximum_not_merged.max(maximum_of(not_merged, "crossed"));
        self.crossed_maximum_merged = self.crossed_maximum_merged.max(maximum_of(merged, "crossed"));
        self.lost_maximum_not_merged = self.lost_maximum_not_merged.max(maximum_of(not_merged, "lost_total"));
        self.lost_maximum_merged = self.lost_maximum_merged.max(maximum_of(merged, "lost_total"));
        self.all_enospc_positions_not_merged += not_merged.quantity("all_enospc").positive_positions;
        self.all_enospc_positions_merged += merged.quantity("all_enospc").positive_positions;
        self.extra_rejected_maximum_not_merged = self.extra_rejected_maximum_not_merged.max(maximum_of(not_merged, "extra_rejected"));
        self.extra_rejected_maximum_merged = self.extra_rejected_maximum_merged.max(maximum_of(merged, "extra_rejected"));
        self.positions_merged_is_all_enospc_against_not_merged += pair.positions_merged_is_all_enospc_against_not_merged;
        self.positions_merged_rejects_more += pair.positions_merged_rejects_more;
        self.largest_rejected_excess_of_merged = self.largest_rejected_excess_of_merged.max(pair.largest_rejected_excess_of_merged);
        self.lowest_candidates_not_merged = self.lowest_candidates_not_merged.min(not_merged.quantity("candidates_lowest").minimum);
        self.lowest_candidates_merged = self.lowest_candidates_merged.min(merged.quantity("candidates_lowest").minimum);
        self.lowest_final_candidates_not_merged = self.lowest_final_candidates_not_merged.min(not_merged.quantity("candidates_final").minimum);
        self.lowest_final_candidates_merged = self.lowest_final_candidates_merged.min(merged.quantity("candidates_final").minimum);
    }
}

/// 第二段的判定：Q2-1（越过数与少掉的状态数，两条 Q2 臂比）、Q2-2（全零并的一次挂载全 ENOSPC，z 族与 H5 并排）、Q2-3（回退候选数）。
fn emit_the_second_segment_judgments(emitter: &mut Emitter, tally: &mut RunTally, results: &[GroupResult]) {
    let mut totals: BTreeMap<(&'static str, &'static str, u64, &'static str, i64), ZeroSlotTotals> = BTreeMap::new();
    for result in results {
        let key = &result.not_merged.key;
        let scope = if key.fault_kind.is_some() { "z_h1_to_h4" } else { "h5_holes" };
        totals
            .entry((scope, key.arm.stuck_ceiling.label(), key.slots_per_region, key.reserve_reading.label(), key.metadata_blocks))
            .or_insert_with(ZeroSlotTotals::start)
            .add(result);
    }
    const ZERO_SLOT_JUDGMENT_LABELS: [&str; 4] = ["Q2-1-not-merged-has-more", "Q2-1-merged-has-more", "Q2-2-merged-all-enospc", "Q2-2-merged-all-enospc-beyond-not-merged"];
    let mut hits_by_scope_and_arm: BTreeMap<(&'static str, &'static str), [(usize, usize); 4]> = BTreeMap::new();
    let mut observed_maxima: BTreeMap<(&'static str, &'static str), [i64; 4]> = BTreeMap::new();
    for ((scope, arm, slots_per_region, reserve, metadata_blocks), total) in &totals {
        let hit_not_merged_more = total.cells_where_not_merged_has_more >= 1;
        let hit_merged_more = total.cells_where_merged_has_more >= 1;
        // 登记第六节 Q2-2 的字面读法：全零并自己的一次挂载全 ENOSPC（对去掉读故障那一次）≥ 1 个位置；与全零不并并排看。
        let hit_merged_all_enospc = total.all_enospc_positions_merged >= 1;
        // 补一条（修订三）：全零并第一次多拒（全零不并那一次发成）之后一次都没发成的位置，两条 Q2 臂共有的卡死不算进来。
        let hit_merged_all_enospc_beyond_not_merged = total.positions_merged_is_all_enospc_against_not_merged >= 1;
        let maxima = observed_maxima.entry((*scope, *arm)).or_insert([0; 4]);
        for (index, value) in [
            count_as_i64(total.cells_where_not_merged_has_more),
            count_as_i64(total.cells_where_merged_has_more),
            count_as_i64(total.all_enospc_positions_merged),
            count_as_i64(total.positions_merged_is_all_enospc_against_not_merged),
        ]
        .into_iter()
        .enumerate()
        {
            maxima[index] = maxima[index].max(value);
        }
        let counters = hits_by_scope_and_arm.entry((*scope, *arm)).or_insert([(0, 0); 4]);
        for (index, hit) in [hit_not_merged_more, hit_merged_more, hit_merged_all_enospc, hit_merged_all_enospc_beyond_not_merged].into_iter().enumerate() {
            counters[index].0 += usize::from(hit);
            counters[index].1 += 1;
        }
        emit(
            emitter,
            format!(
                "name=q2_comparison segment=second scope={scope} arm={arm} S={slots_per_region} reserve={reserve} c_max={metadata_blocks} cells={} \
                 cells_where_not_merged_has_more={} cells_where_merged_has_more={} largest_crossed_excess_of_not_merged={} largest_lost_excess_of_not_merged={} \
                 crossed_maximum_not_merged={} crossed_maximum_merged={} lost_maximum_not_merged={} lost_maximum_merged={} \
                 all_enospc_positions_not_merged={} all_enospc_positions_merged={} extra_rejected_maximum_not_merged={} extra_rejected_maximum_merged={} \
                 positions_merged_is_all_enospc_against_not_merged={} positions_merged_rejects_more={} largest_rejected_excess_of_merged={} \
                 lowest_candidates_not_merged={} lowest_candidates_merged={} lowest_final_candidates_not_merged={} lowest_final_candidates_merged={} \
                 hit_not_merged_has_more={} hit_merged_has_more={} hit_merged_all_enospc={} hit_merged_all_enospc_beyond_not_merged={}",
                total.cells,
                total.cells_where_not_merged_has_more,
                total.cells_where_merged_has_more,
                total.largest_crossed_excess_of_not_merged,
                total.largest_lost_excess_of_not_merged,
                total.crossed_maximum_not_merged,
                total.crossed_maximum_merged,
                total.lost_maximum_not_merged,
                total.lost_maximum_merged,
                total.all_enospc_positions_not_merged,
                total.all_enospc_positions_merged,
                total.extra_rejected_maximum_not_merged,
                total.extra_rejected_maximum_merged,
                total.positions_merged_is_all_enospc_against_not_merged,
                total.positions_merged_rejects_more,
                total.largest_rejected_excess_of_merged,
                total.lowest_candidates_not_merged,
                total.lowest_candidates_merged,
                total.lowest_final_candidates_not_merged,
                total.lowest_final_candidates_merged,
                yes_no(hit_not_merged_more),
                yes_no(hit_merged_more),
                yes_no(hit_merged_all_enospc),
                yes_no(hit_merged_all_enospc_beyond_not_merged)
            ),
        );
    }
    // 判别力自证（登记第八节）：门槛从 1 挪到「观测到的最大值 + 1」，打中必须由真转假（按 (判定, 范围) 各一组，臂名当键）。
    for (index, judgment_label) in ZERO_SLOT_JUDGMENT_LABELS.iter().enumerate() {
        for scope in ["z_h1_to_h4", "h5_holes"] {
            let per_arm: BTreeMap<&'static str, (usize, usize, Option<i64>)> = hits_by_scope_and_arm
                .iter()
                .filter(|((hit_scope, _), _)| *hit_scope == scope)
                .map(|((_, arm), counters)| (*arm, (counters[index].0, counters[index].1, Some(observed_maxima[&(scope, *arm)][index]))))
                .collect();
            emit_the_threshold_shift_check(emitter, tally, "second", &format!("{judgment_label}@{scope}"), &per_arm);
        }
    }
    for ((scope, arm), counters) in &hits_by_scope_and_arm {
        for (index, judgment_label) in ZERO_SLOT_JUDGMENT_LABELS.iter().enumerate() {
            let (hit_points, points) = counters[index];
            let stability = if hit_points == points { "stable_hit" } else if hit_points == 0 { "stable_no_hit" } else { "unstable" };
            emit(
                emitter,
                format!("name=stability segment=second id={judgment_label} scope={scope} arm={arm} hit_points={hit_points} points={points} stability={stability}"),
            );
        }
    }
}

// ── 对拍锚点：装置与冻结树 `crates/` 逐格对（登记第十一节停机条款 K1–K6） ─────────────

const SLOTS_PER_REGION_OF_THE_CONTROL_POOLS: u64 = 4;
const METADATA_BLOCKS_OF_THE_CONTROL_POOLS: i64 = 5;

fn geometry_of_the_control_pools() -> RingGeometry {
    RingGeometry { slots_per_region: SLOTS_PER_REGION_OF_THE_CONTROL_POOLS }
}

fn control_pool_settings(reserve_reading: ReserveReading, fullness: PoolFullness) -> PoolSettings {
    PoolSettings {
        geometry: geometry_of_the_control_pools(),
        metadata_blocks: METADATA_BLOCKS_OF_THE_CONTROL_POOLS,
        reserve_reading,
        fullness,
    }
}

/// 没洞那一份（用例文件头）：实例 1 覆盖写到 txg 17（第一个文件在 3，之后 15 次覆盖写到 17）。
fn events_of_the_pool_without_holes() -> Vec<HistoryEvent> {
    let mut events = vec![HistoryEvent::WritableMount];
    events.extend(std::iter::repeat(HistoryEvent::Overwrite).take(15));
    events
}

/// 有洞那一份：实例 1 覆盖写到 17，崩三次留洞 18、19、20，再挂写 X = 21（写行）与暖机、再空发布两次，正常退出。
fn events_of_the_pool_over_the_holes() -> Vec<HistoryEvent> {
    let mut events = events_of_the_pool_without_holes();
    events.extend(std::iter::repeat(HistoryEvent::MountCrashedBeforeItsRowRoot).take(3));
    events.push(HistoryEvent::WritableMount);
    events.extend(std::iter::repeat(HistoryEvent::EmptyPublish).take(2));
    events
}

const FAULT_TXG_WITHOUT_HOLES: u64 = 17;
const ANCHOR_TXG_WITHOUT_HOLES: u64 = 28;
const FAULT_TXG_OVER_THE_HOLES: u64 = 21;
const ANCHOR_TXG_OVER_THE_HOLES: u64 = 29;

/// 在锚点上做一次准入抬 F 的读数：被测挂载起 X 那一槽按 `fault_kind` 读坏，空发布推到 `anchor_txg`，再调一次准入抬 F（宽松池，只看上限）。
#[derive(Clone, Debug)]
struct AnchorRaise {
    unmerged_ceiling: u64,
    merged_ceiling: u64,
    floor_after: u64,
    crossed_non_empty_roots: u32,
    pushed: bool,
    stop: Option<RejectionReason>,
    slot_entry_before_the_raise: Option<Occupant>,
    trace: Vec<String>,
}

fn raise_at_an_anchor(events: &[HistoryEvent], fault_txg: u64, fault_kind: Option<ReadFaultKind>, anchor_txg: u64, arm: Arm) -> AnchorRaise {
    let geometry = geometry_of_the_control_pools();
    let history = PoolAfterHistory::from_events(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS, events);
    let fault_slot = geometry.slot_of_txg(fault_txg);
    let fault = fault_kind.map(|kind| ReadFault { slot: fault_slot, kind, persistence: FaultPersistence::UntilTheProcessWritesTheSlot });
    let settings = control_pool_settings(ReserveReading::CodeReadingOneEmptyPublish, PoolFullness::Roomy);
    let mut session = start_a_session(&history, fault, settings, arm, Controls::none(), true);
    let slot_entry_before_the_raise = session.pool.table.occupants.get(&fault_slot).copied();
    while session.pool.current_txg < anchor_txg {
        session.publish_an_empty_root();
    }
    let reading = session.pool.ceiling_pass(session.merges_all_zero_slots()).expect("锚点上没有知道住着根的槽读坏");
    session.note_the_cap(reading);
    let mut tally = RequestTally { pushed_within_the_budget: 0, empty_publishes: 0, empty_publish_blocks: 0, raise_publishes: 0, pushed_an_empty_publish: false };
    let outcome = session.push_one_floor_raise_within_the_budget(&mut tally, Some(reading));
    let (pushed, stop) = match outcome {
        StepOutcome::Pushed => (true, None),
        StepOutcome::Stopped(reason) => (false, Some(reason)),
    };
    AnchorRaise {
        unmerged_ceiling: reading.unmerged,
        merged_ceiling: reading.merged,
        floor_after: session.pool.floor_in_the_current_version,
        crossed_non_empty_roots: session.ledger.raises.first().map_or(0, |raise| raise.crossed_non_empty_roots),
        pushed,
        stop,
        slot_entry_before_the_raise,
        trace: session.ledger.trace,
    }
}

fn arm_of(stuck_ceiling: StuckCeilingDisposition, all_zero_slot: AllZeroSlotDisposition) -> Arm {
    Arm { stuck_ceiling, all_zero_slot }
}

fn render_an_optional_occupant(occupant: Option<Occupant>) -> String {
    match occupant {
        None => "none".to_string(),
        Some(Occupant::ValidRoot { checkpoint_txg }) => format!("valid_root_{checkpoint_txg}"),
        Some(Occupant::ValidRootReadWhenRereading { checkpoint_txg }) => format!("valid_root_reread_{checkpoint_txg}"),
        Some(Occupant::BadAtMount { oldest_txg_it_can_have_held, highest_txg_it_can_hold }) => {
            format!("bad_at_mount_oldest_{oldest_txg_it_can_have_held}_highest_{highest_txg_it_can_hold}")
        }
        Some(Occupant::AllZeroAtMount { oldest_txg_it_can_have_held, highest_txg_it_can_hold }) => {
            format!("all_zero_oldest_{oldest_txg_it_can_have_held}_highest_{highest_txg_it_can_hold}")
        }
    }
}

/// 一格对拍锚点：期望值是冻结树上那份用例钉着的值（登记第十一节表），`expected_ceiling` 与 `expected_crossed` 为 `None` 时只报不判。
fn emit_an_anchor_cell(
    emitter: &mut Emitter,
    tally: &mut RunTally,
    anchor_label: &str,
    description: &str,
    outcome: &AnchorRaise,
    arm: Arm,
    expected_floor: Option<u64>,
    expected_crossed: Option<u32>,
) {
    let ceiling_of_this_arm = match arm.stuck_ceiling {
        StuckCeilingDisposition::DoNotMergeTheBadSlots => outcome.unmerged_ceiling,
        StuckCeilingDisposition::AsToday | StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget => {
            outcome.merged_ceiling
        }
    };
    let floor_matches = expected_floor.map_or(true, |expected| outcome.floor_after == expected && ceiling_of_this_arm == expected);
    let crossed_matches = expected_crossed.map_or(true, |expected| outcome.crossed_non_empty_roots == expected);
    tally.halt_check(floor_matches && crossed_matches && outcome.pushed, &format!("{anchor_label}_{}_{}", arm.stuck_ceiling.label(), arm.all_zero_slot.label()));
    emit(
        emitter,
        format!(
            "name=anchor id={anchor_label} description={description} q1={} q2={} c0={} c_merged={} floor_after={} crossed={} pushed={} stop={} expected_floor={} expected_crossed={} slot_entry={} matches={}",
            arm.stuck_ceiling.label(),
            arm.all_zero_slot.label(),
            outcome.unmerged_ceiling,
            outcome.merged_ceiling,
            outcome.floor_after,
            outcome.crossed_non_empty_roots,
            yes_no(outcome.pushed),
            outcome.stop.map_or("none", RejectionReason::label),
            expected_floor.map_or("none".to_string(), |value| value.to_string()),
            expected_crossed.map_or("none".to_string(), |value| value.to_string()),
            render_an_optional_occupant(outcome.slot_entry_before_the_raise),
            yes_no(floor_matches && crossed_matches && outcome.pushed),
        ),
    );
    for (step, line) in outcome.trace.iter().enumerate() {
        emit(emitter, format!("name=trace id={anchor_label} q1={} q2={} step={step} {line}", arm.stuck_ceiling.label(), arm.all_zero_slot.label()));
    }
}

fn emit_a_table_entry_anchor(emitter: &mut Emitter, tally: &mut RunTally, anchor_label: &str, entry: Option<Occupant>, expected_oldest: u64, expected_highest: Option<u64>) {
    let matches = match entry {
        Some(Occupant::BadAtMount { oldest_txg_it_can_have_held, highest_txg_it_can_hold }) => {
            oldest_txg_it_can_have_held == expected_oldest && Some(highest_txg_it_can_hold) == expected_highest
        }
        Some(Occupant::AllZeroAtMount { oldest_txg_it_can_have_held, .. }) => oldest_txg_it_can_have_held == expected_oldest && expected_highest.is_none(),
        Some(Occupant::ValidRoot { .. } | Occupant::ValidRootReadWhenRereading { .. }) | None => false,
    };
    tally.halt_check(matches, anchor_label);
    emit(
        emitter,
        format!(
            "name=anchor id={anchor_label} slot_entry={} expected_oldest={expected_oldest} expected_highest={} matches={}",
            render_an_optional_occupant(entry),
            expected_highest.map_or("none".to_string(), |value| value.to_string()),
            yes_no(matches)
        ),
    );
}

/// 登记第十一节 K1–K6：装置在冻结树那份用例的两份盘面上给的值与用例钉着的逐格对。K7（结构）由执行员读冻结树拷贝里的行逐项写进报告。
fn emit_the_frozen_tree_anchors(emitter: &mut Emitter, tally: &mut RunTally) {
    let holes = events_of_the_pool_over_the_holes();
    let no_holes = events_of_the_pool_without_holes();
    let not_merged = AllZeroSlotDisposition::NotMerged;
    let merged = AllZeroSlotDisposition::Merged;
    for stuck_ceiling in StuckCeilingDisposition::every_arm() {
        let arm = arm_of(stuck_ceiling, not_merged);
        let outcome = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, Some(ReadFaultKind::DeviceError), ANCHOR_TXG_OVER_THE_HOLES, arm);
        emit_an_anchor_cell(emitter, tally, "K1", "有洞那一份 X 读不出：上限 6、F 抬到 6", &outcome, arm, Some(6), None);
    }
    for kind in [ReadFaultKind::DeviceError, ReadFaultKind::GarbageBytes] {
        for stuck_ceiling in [StuckCeilingDisposition::AsToday, StuckCeilingDisposition::PushEmptyWithinBudget, StuckCeilingDisposition::PushEmptyBeyondBudget] {
            let arm = arm_of(stuck_ceiling, not_merged);
            let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(kind), ANCHOR_TXG_WITHOUT_HOLES, arm);
            emit_an_anchor_cell(emitter, tally, &format!("K2_{}", kind.label()), "没洞那一份 X 读不出：上限 17", &outcome, arm, Some(17), None);
        }
    }
    for stuck_ceiling in [StuckCeilingDisposition::AsToday, StuckCeilingDisposition::PushEmptyWithinBudget, StuckCeilingDisposition::PushEmptyBeyondBudget] {
        let arm = arm_of(stuck_ceiling, not_merged);
        let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_WITHOUT_HOLES, arm);
        emit_an_anchor_cell(emitter, tally, "K3", "没洞那一份 X 读回全零、全零不并：上限 18、越过 X（越过数 1）", &outcome, arm, Some(18), Some(1));
        let arm = arm_of(stuck_ceiling, merged);
        let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_WITHOUT_HOLES, arm);
        emit_an_anchor_cell(emitter, tally, "K3", "没洞那一份 X 读回全零、全零并：上限 17", &outcome, arm, Some(17), Some(0));
    }
    for stuck_ceiling in StuckCeilingDisposition::every_arm() {
        for zero in [not_merged, merged] {
            let arm = arm_of(stuck_ceiling, zero);
            for kind in [Some(ReadFaultKind::AllZeroBytes), None] {
                let outcome = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, kind, ANCHOR_TXG_OVER_THE_HOLES, arm);
                emit_an_anchor_cell(emitter, tally, "K4", "有洞那一份 X 读回全零或读得出：上限 6", &outcome, arm, Some(6), None);
            }
        }
    }
    for kind in [ReadFaultKind::DeviceError, ReadFaultKind::GarbageBytes, ReadFaultKind::AllZeroBytes] {
        for zero in [not_merged, merged] {
            let arm = arm_of(StuckCeilingDisposition::DoNotMergeTheBadSlots, zero);
            let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(kind), ANCHOR_TXG_WITHOUT_HOLES, arm);
            emit_an_anchor_cell(emitter, tally, &format!("K5_{}", kind.label()), "没洞那一份不并：上限 18", &outcome, arm, Some(18), None);
        }
    }
    let arm = arm_of(StuckCeilingDisposition::AsToday, not_merged);
    let zero_no_holes = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_WITHOUT_HOLES, arm);
    emit_a_table_entry_anchor(emitter, tally, "K6_no_holes", zero_no_holes.slot_entry_before_the_raise, 5, None);
    let zero_holes = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_OVER_THE_HOLES, arm);
    emit_a_table_entry_anchor(emitter, tally, "K6_holes", zero_holes.slot_entry_before_the_raise, 9, None);
    let bad_no_holes = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::DeviceError), ANCHOR_TXG_WITHOUT_HOLES, arm);
    emit_a_table_entry_anchor(emitter, tally, "K2_table_no_holes", bad_no_holes.slot_entry_before_the_raise, 5, Some(17));
    let bad_holes = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, Some(ReadFaultKind::DeviceError), ANCHOR_TXG_OVER_THE_HOLES, arm);
    emit_a_table_entry_anchor(emitter, tally, "K1_table_holes", bad_holes.slot_entry_before_the_raise, 9, Some(21));
}

// ── 第七节的锚点：条款本身（甲）与独立算出的（乙） ─────────────────────────────────────

fn emit_a_check(emitter: &mut Emitter, tally: &mut RunTally, is_clause_check: bool, check_label: &str, observed: String, expected: String) {
    let matches = observed == expected;
    if is_clause_check {
        tally.halt_check(matches, check_label);
    } else {
        tally.self_check(matches, check_label);
    }
    emit(emitter, format!("name=anchor id={check_label} observed={observed} expected={expected} matches={}", yes_no(matches)));
}

/// 甲类 A1–A5（出自条款，不符走 F3 停机）与乙类 B1、B2、B4（`anchors.py` 现算的数，不符作废 V4）。
fn emit_the_structural_anchors(emitter: &mut Emitter, tally: &mut RunTally) {
    emit_a_check(emitter, tally, true, "A1", PUBLISHES_PER_ADMISSION_AT_MOST.to_string(), "8".to_string());
    for (metadata_blocks, expected) in [(5, 45), (10, 80)] {
        emit_a_check(
            emitter,
            tally,
            true,
            &format!("A2_c{metadata_blocks}"),
            ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes.reserve_blocks(metadata_blocks).to_string(),
            expected.to_string(),
        );
    }
    let observed_raise_lengths: Vec<usize> = (0..3).map(|residue| txgs_of_the_publishes_carrying_the_floor_to_every_device(30 + residue).len()).collect();
    emit_a_check(emitter, tally, true, "A3_raise_sequence_by_txg_mod_3", format!("{observed_raise_lengths:?}"), "[2, 3, 2]".to_string());
    emit_a_check(emitter, tally, true, "B1_matches_A3", format!("{observed_raise_lengths:?}"), "[2, 3, 2]".to_string());
    let slots_are_in_the_format_range = SLOTS_PER_REGION_CHOICES.iter().all(|slots| (TOLERATED_ROOT_SLOT_WRITE_FAILURES as u64 + 2..=16).contains(slots));
    emit_a_check(emitter, tally, true, "A4_slots_per_region_in_4_to_16", yes_no(slots_are_in_the_format_range).to_string(), "yes".to_string());
    let warm_up_lengths: Vec<usize> = (0..3).map(|residue| warm_up_publish_txgs(60 + residue).len()).collect();
    let warm_up_is_at_most_the_regions = warm_up_lengths.iter().all(|length| *length <= 3);
    emit_a_check(emitter, tally, true, "A5_warm_up_at_most_three", yes_no(warm_up_is_at_most_the_regions).to_string(), "yes".to_string());
    emit_a_check(emitter, tally, false, "B2_warm_up_by_row_mod_3", format!("{warm_up_lengths:?}"), "[1, 1, 2]".to_string());
    for (metadata_blocks, expected_empties, expected_reserve) in [(5i64, 35i64, 45i64), (10, 70, 80)] {
        let empties = i64::try_from(PUBLISHES_PER_ADMISSION_AT_MOST - 1).expect("B") * metadata_blocks;
        emit_a_check(emitter, tally, false, &format!("B4_c{metadata_blocks}_seven_empties"), empties.to_string(), expected_empties.to_string());
        emit_a_check(
            emitter,
            tally,
            false,
            &format!("B4_c{metadata_blocks}_reserve"),
            ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes.reserve_blocks(metadata_blocks).to_string(),
            expected_reserve.to_string(),
        );
    }
}

fn every_arm_pair() -> Vec<Arm> {
    let mut arms = Vec::new();
    for stuck_ceiling in StuckCeilingDisposition::every_arm() {
        for all_zero_slot in [AllZeroSlotDisposition::NotMerged, AllZeroSlotDisposition::Merged] {
            arms.push(arm_of(stuck_ceiling, all_zero_slot));
        }
    }
    arms
}

fn label_of_the_arm(arm: Arm) -> String {
    format!("{}_{}", arm.stuck_ceiling.label(), arm.all_zero_slot.label())
}

/// B5：H1，S = 4，c = 5，P保留 = 45，近满 k = 0，不注读故障，第一次请求（八条臂都等于这几个数，逐步打印轨迹）。
fn emit_the_first_request_anchor(emitter: &mut Emitter, tally: &mut RunTally) {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS);
    let settings = control_pool_settings(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
    for arm in every_arm_pair() {
        let mut session = start_a_session(&history, None, settings, arm, Controls::none(), true);
        let ring_oldest = session.pool.truth_ring.iter().flatten().map(|root| root.checkpoint_txg).min().expect("环里有根");
        let facts = format!(
            "N={} row={} warmup={} ring=[{ring_oldest},{}] pending_total={}",
            history.highest_txg_used,
            history.highest_txg_used + 1,
            session.pool.current_txg - (history.highest_txg_used + 1),
            session.pool.current_txg,
            session.pool.space.pending_blocks()
        );
        emit_a_check(emitter, tally, false, &format!("B5_mount_facts_{}", label_of_the_arm(arm)), facts, "N=50 row=51 warmup=1 ring=[41,52] pending_total=127".to_string());
        let record = session.run_one_request();
        let raise = session.ledger.raises.first().copied();
        let after = format!(
            "accepted={} raises={} ceiling={} sequence_publishes={} floor={} lowest_free_in_the_raise={:?} free_above_reserve_after={} pending_after={} txg_after={}",
            record.accepted,
            session.ledger.raises.len(),
            raise.map_or(0, |raise| raise.truth_ceiling),
            raise.map_or(0, |raise| raise.publishes),
            record.floor_after,
            session.ledger.lowest_free_blocks_in_the_first_raise,
            record.free_above_the_reserve_after,
            session.pool.space.pending_blocks(),
            session.pool.current_txg
        );
        // 回收 78 块：127 − 78 + 抬 F 那一串 3 次 × 5 + 用户发布放掉的 13 = 77。
        emit_a_check(
            emitter,
            tally,
            false,
            &format!("B5_first_request_{}", label_of_the_arm(arm)),
            after,
            "accepted=true raises=1 ceiling=47 sequence_publishes=3 floor=47 lowest_free_in_the_raise=Some(30) free_above_reserve_after=50 pending_after=77 txg_after=56".to_string(),
        );
        for (step, line) in session.ledger.trace.iter().enumerate() {
            emit(emitter, format!("name=trace id=B5 arm={} step={step} {line}", label_of_the_arm(arm)));
        }
    }
}

/// B6：宽松池、不注读故障，每条臂被拒 0 次（H1–H4 × S × c_max × P保留的每一格）。
fn emit_the_roomy_pool_anchor(emitter: &mut Emitter, tally: &mut RunTally) {
    let mut cells_checked = 0usize;
    let mut cells_with_a_rejection = 0usize;
    for pool in pool_settings_list().into_iter().filter(|pool| pool.fullness == PoolFullness::Roomy) {
        for family in FAMILIES_WITH_A_FAULT_TO_INJECT {
            let history = family.build_pool_after_history(pool.geometry, pool.metadata_blocks);
            for arm in every_arm_pair() {
                let run = run_a_mount(&history, None, pool, arm, Controls::none(), false);
                cells_checked += 1;
                cells_with_a_rejection += usize::from(rejected_count(&run) > 0);
            }
        }
    }
    emit_a_check(emitter, tally, false, "B6_roomy_pool_no_fault_never_rejects", format!("cells={cells_checked} cells_with_a_rejection={cells_with_a_rejection}"), format!("cells={cells_checked} cells_with_a_rejection=0"));
}

// ── 阳性对照：对每一条臂各跑一次，期望值写死（登记第五节「阳性对照」，失败条款 F1） ────────────

/// PC4 推空两臂的盘面：H1，S = 4（N = 50，暖机 1 次），挂载之前 F 已抬到 N − 5，e，P1，j = 5（读坏槽里住着 txg 45），宽松池。
fn pc4_history_and_fault_for_the_empty_pushing_arms(floor_before_the_mount: u64, with_fault: bool) -> (PoolAfterHistory, Option<ReadFault>) {
    let geometry = geometry_of_the_control_pools();
    let mut events = HistoryFamily::CrashedAfterOverwrites.events(geometry);
    events.push(HistoryEvent::FloorRaisedWithoutBuildingTheSequence { new_floor: floor_before_the_mount });
    let history = PoolAfterHistory::from_events(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS, &events);
    let fault = with_fault.then(|| ReadFault {
        slot: geometry.slot_of_txg(history.highest_txg_used - 5),
        kind: ReadFaultKind::DeviceError,
        persistence: FaultPersistence::UntilTheProcessWritesTheSlot,
    });
    (history, fault)
}

fn emit_the_positive_controls(emitter: &mut Emitter, tally: &mut RunTally) {
    let geometry = geometry_of_the_control_pools();
    let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS);
    let requests = i64::try_from(requests_per_mount(geometry)).expect("请求数");
    for arm in every_arm_pair() {
        let label = label_of_the_arm(arm);
        // 登记写的是近满 k = 8；单测读数：k = 8 的池在 J 次请求里一次抬 F 都不走（回收随环转动补回每次请求花的块），对照打不出效应，
        // 所以 PC1、PC3 取第一次请求就走抬 F 的近满 k = 0（修订一），k = 8 那一格只报它走了几次抬 F。
        let eight_spare = control_pool_settings(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithEightSpareOverwritesForControlsOnly);
        let registered_pool_run = run_a_mount(&history, None, eight_spare, arm, Controls::none(), false);
        emit(emitter, format!("name=anchor id=PC1_PC3_registered_k8_pool_floor_raises arm={label} observed={} note=registered_pool_does_not_reach_the_raise_path", registered_pool_run.raises.len()));
        let no_spare = control_pool_settings(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
        let controls_pc1 = Controls { first_raise_target_above_the_ceiling_by: Some(2), ..Controls::none() };
        let pc1 = run_a_mount(&history, None, no_spare, arm, controls_pc1, false);
        emit_a_check(emitter, tally, false, &format!("PC1_crossed_by_two_{label}"), pc1.raises.first().map_or(-1, |raise| i64::from(raise.crossed_non_empty_roots)).to_string(), "2".to_string());
        let controls_pc3 = Controls { first_raise_target_is_the_current_txg: true, ..Controls::none() };
        let pc3 = run_a_mount(&history, None, no_spare, arm, controls_pc3, false);
        emit_a_check(emitter, tally, false, &format!("PC3_states_lost_{label}"), pc3.raises.first().map_or(-1, |raise| count_as_i64(raise.states_lost)).to_string(), "3".to_string());
        let two_spare_code_reserve = control_pool_settings(ReserveReading::CodeReadingOneEmptyPublish, PoolFullness::NearlyFullWithTwoSpareOverwrites);
        let controls_pc2 = Controls { reclaim_disabled: true, ..Controls::none() };
        let pc2 = run_a_mount(&history, None, two_spare_code_reserve, arm, controls_pc2, false);
        emit_a_check(emitter, tally, false, &format!("PC2_rejected_{label}"), count_as_i64(rejected_count(&pc2)).to_string(), (requests - 2).to_string());
        let pushing = matches!(arm.stuck_ceiling, StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget);
        let (history_pc4, fault_pc4) = if pushing {
            pc4_history_and_fault_for_the_empty_pushing_arms(45, true)
        } else {
            pc4_history_and_fault_for_the_empty_pushing_arms(47, false)
        };
        let roomy = control_pool_settings(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::Roomy);
        let controls_pc4 = Controls {
            first_request_is_judged_short_throughout: true,
            push_one_empty_publish_on_the_first_floor_already_at_the_ceiling: !pushing,
            ..Controls::none()
        };
        let mut session = start_a_session(&history_pc4, fault_pc4, roomy, arm, controls_pc4, true);
        let record = session.run_one_request();
        let (expected_empties, expected_blocks) = if pushing { (5, 25) } else { (1, 5) };
        emit_a_check(
            emitter,
            tally,
            false,
            &format!("PC4_empty_publishes_{label}"),
            format!("{} {}", record.empty_publishes, record.empty_publish_blocks),
            format!("{expected_empties} {expected_blocks}"),
        );
        for (step, line) in session.ledger.trace.iter().enumerate() {
            emit(emitter, format!("name=trace id=PC4 arm={label} step={step} {line}"));
        }
    }
}

/// B3 与 F4：单个读坏槽 j（能住的最大 txg N − j）暖机之后还要推 D = 3S − j − 1 − w 次才被写掉；推空W 在 PC4 那一形（F 先抬到 N − j）上推的次数 = max(D, 0)，
/// 推空B 受 B 限，推的次数 = min(max(D, 0), B − 1)。H1 的 w = 1（N = 2 + 12S，写行 txg ≡ 0 mod 3）。
fn emit_the_empty_push_count_anchor(emitter: &mut Emitter, tally: &mut RunTally) {
    for slots_per_region in SLOTS_PER_REGION_CHOICES {
        let geometry = RingGeometry { slots_per_region };
        let ring_length = i64::try_from(geometry.ring_length_in_txgs()).expect("环长");
        let base_events = HistoryFamily::CrashedAfterOverwrites.events(geometry);
        let mut mismatches_beyond_budget = Vec::new();
        let mut mismatches_within_budget = Vec::new();
        let mut positions = 0;
        for stuck_position in 4..ring_length {
            let probe_history = {
                let mut events = base_events.clone();
                let newest_txg = 2 + 4 * geometry.ring_length_in_txgs();
                events.push(HistoryEvent::FloorRaisedWithoutBuildingTheSequence { new_floor: newest_txg - u64::try_from(stuck_position).expect("j") });
                PoolAfterHistory::from_events(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS, &events)
            };
            let fault = ReadFault {
                slot: geometry.slot_of_txg(probe_history.highest_txg_used - u64::try_from(stuck_position).expect("j")),
                kind: ReadFaultKind::DeviceError,
                persistence: FaultPersistence::UntilTheProcessWritesTheSlot,
            };
            let roomy = PoolSettings {
                geometry,
                metadata_blocks: METADATA_BLOCKS_OF_THE_CONTROL_POOLS,
                reserve_reading: ReserveReading::CodeReadingOneEmptyPublish,
                fullness: PoolFullness::Roomy,
            };
            let controls = Controls { first_request_is_judged_short_throughout: true, ..Controls::none() };
            let pushes_still_needed = (ring_length - stuck_position - 1 - 1).max(0);
            for (stuck_ceiling, expected, mismatches) in [
                (StuckCeilingDisposition::PushEmptyBeyondBudget, pushes_still_needed, &mut mismatches_beyond_budget),
                (StuckCeilingDisposition::PushEmptyWithinBudget, pushes_still_needed.min(i64::try_from(PUBLISHES_PER_ADMISSION_AT_MOST - 1).expect("B")), &mut mismatches_within_budget),
            ] {
                let mut session = start_a_session(&probe_history, Some(fault), roomy, arm_of(stuck_ceiling, AllZeroSlotDisposition::NotMerged), controls, false);
                let observed = i64::from(session.run_one_request().empty_publishes);
                if observed != expected {
                    mismatches.push(format!("j{stuck_position}:{observed}vs{expected}"));
                }
            }
            positions += 1;
        }
        emit_a_check(
            emitter,
            tally,
            false,
            &format!("B3_F4_empty_pushes_S{slots_per_region}"),
            format!("positions={positions} beyond_budget_mismatches={mismatches_beyond_budget:?} within_budget_mismatches={mismatches_within_budget:?}"),
            format!("positions={positions} beyond_budget_mismatches=[] within_budget_mismatches=[]"),
        );
    }
}

// ── 每段的自检、判决行与入口 ─────────────────────────────────────────────────────────────

fn bool_text(condition: bool) -> &'static str {
    if condition {
        "true"
    } else {
        "false"
    }
}

/// 一段扫完之后的自检（登记第十一节 V2、V5，第七节 A1、互比旁边的钉死）。
#[derive(Clone, Copy, Debug)]
struct SegmentChecks {
    cells: usize,
    positions: usize,
    runs: usize,
    positions_match_the_expected_count: bool,
    publishes_within_the_budget_where_the_arm_keeps_it: bool,
    the_two_zero_slot_arms_agree_where_they_must: bool,
    baselines_agree_across_arms: bool,
}

fn check_the_segment(results: &[GroupResult], zero_slot_arms_must_agree_for: &dyn Fn(StuckCeilingDisposition) -> bool) -> SegmentChecks {
    let mut checks = SegmentChecks {
        cells: 0,
        positions: 0,
        runs: 0,
        positions_match_the_expected_count: true,
        publishes_within_the_budget_where_the_arm_keeps_it: true,
        the_two_zero_slot_arms_agree_where_they_must: true,
        baselines_agree_across_arms: true,
    };
    let mut baselines: BTreeMap<(HistoryFamily, u64, i64, &'static str, &'static str), Vec<(bool, u64, i64, usize)>> = BTreeMap::new();
    for result in results {
        for cell in [&result.not_merged, &result.merged] {
            checks.cells += 1;
            checks.positions += cell.positions;
            checks.positions_match_the_expected_count &= cell.positions == cell.expected_positions;
            let keeps_the_budget = match cell.key.arm.stuck_ceiling {
                StuckCeilingDisposition::AsToday | StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::DoNotMergeTheBadSlots => true,
                StuckCeilingDisposition::PushEmptyBeyondBudget => false,
            };
            if keeps_the_budget {
                checks.publishes_within_the_budget_where_the_arm_keeps_it &=
                    cell.quantity("publishes_max_in_admission").maximum <= i64::try_from(PUBLISHES_PER_ADMISSION_AT_MOST).expect("B");
            }
        }
        checks.runs += result.runs;
        if zero_slot_arms_must_agree_for(result.not_merged.key.arm.stuck_ceiling) {
            checks.the_two_zero_slot_arms_agree_where_they_must &= result.pair.identical_positions == result.pair.positions;
        }
        let key = result.not_merged.key;
        if key.family != HistoryFamily::YoungRingWithCrashHoles {
            checks.the_two_zero_slot_arms_agree_where_they_must &= result.baselines_of_the_two_zero_slot_arms_are_identical;
            let baseline_key = (key.family, key.slots_per_region, key.metadata_blocks, key.reserve_reading.label(), key.fullness.label());
            match baselines.get(&baseline_key) {
                None => {
                    baselines.insert(baseline_key, result.baseline_signature.clone());
                }
                Some(first_signature) => checks.baselines_agree_across_arms &= *first_signature == result.baseline_signature,
            }
        }
    }
    checks
}

fn emit_the_segment_verdict(emitter: &mut Emitter, tally: &mut RunTally, segment: &str, checks: SegmentChecks) {
    tally.self_check(checks.positions_match_the_expected_count, "V2_positions_do_not_match_the_expected_count");
    tally.self_check(checks.publishes_within_the_budget_where_the_arm_keeps_it, "A1_publishes_over_the_budget");
    tally.self_check(checks.the_two_zero_slot_arms_agree_where_they_must, "F5_q2_arms_differ_where_they_must_agree");
    tally.self_check(checks.baselines_agree_across_arms, "baseline_differs_across_arms");
    emit(
        emitter,
        format!(
            "name=verdict scope={segment} cells={} positions={} runs={} positions_match_expected={} publishes_within_budget_where_the_arm_keeps_it={} q2_arms_agree_where_they_must={} baselines_agree_across_arms={} void_conditions={} void_list={} halt_conditions={}",
            checks.cells,
            checks.positions,
            checks.runs,
            bool_text(checks.positions_match_the_expected_count),
            bool_text(checks.publishes_within_the_budget_where_the_arm_keeps_it),
            bool_text(checks.the_two_zero_slot_arms_agree_where_they_must),
            bool_text(checks.baselines_agree_across_arms),
            tally.void_conditions.len(),
            if tally.void_conditions.is_empty() { "none".to_string() } else { tally.void_conditions.join(",") },
            tally.halt_conditions.len(),
        ),
    );
}

fn emit_the_config(emitter: &mut Emitter, segment: &str) {
    emit(
        emitter,
        format!(
            "name=config segment={segment} slots_per_region={SLOTS_PER_REGION_CHOICES:?} metadata_blocks={METADATA_BLOCKS_CHOICES:?} data_blocks_per_overwrite={DATA_BLOCKS_PER_OVERWRITE} admission_budget={PUBLISHES_PER_ADMISSION_AT_MOST} requests_per_mount=2*3S+{REQUESTS_PER_MOUNT_BASE} reserve_readings=code:c,clause:10+7c pools=near0,near2,roomy"
        ),
    );
}

fn run_the_checks(emitter: &mut Emitter, tally: &mut RunTally) {
    emit_the_config(emitter, "checks");
    emit_the_frozen_tree_anchors(emitter, tally);
    emit_the_structural_anchors(emitter, tally);
    emit_the_first_request_anchor(emitter, tally);
    emit_the_roomy_pool_anchor(emitter, tally);
    emit_the_positive_controls(emitter, tally);
    emit_the_empty_push_count_anchor(emitter, tally);
    emit(
        emitter,
        format!(
            "name=verdict scope=checks self_checks_passed={} void_conditions={} void_list={} halt_conditions={} halt_list={}",
            tally.self_checks_passed,
            tally.void_conditions.len(),
            if tally.void_conditions.is_empty() { "none".to_string() } else { tally.void_conditions.join(",") },
            tally.halt_conditions.len(),
            if tally.halt_conditions.is_empty() { "none".to_string() } else { tally.halt_conditions.join(",") },
        ),
    );
}

fn run_the_first_segment(emitter: &mut Emitter, tally: &mut RunTally) {
    emit_the_config(emitter, "first");
    let groups = groups_of_the_first_segment();
    let results = run_groups_in_parallel(&groups);
    for result in &results {
        emit(emitter, render_a_cell("first", &result.not_merged));
    }
    // 第一段的盘面里没有读回全零的槽（e、g 两种读坏），两条 Q2 臂在每一条 Q1 臂上都必须逐项相同。
    let checks = check_the_segment(&results, &|_| true);
    emit_the_first_segment_judgments(emitter, tally, &results);
    emit_the_segment_verdict(emitter, tally, "first", checks);
}

fn run_the_second_segment(emitter: &mut Emitter, tally: &mut RunTally) {
    emit_the_config(emitter, "second");
    let groups = groups_of_the_second_segment();
    let results = run_groups_in_parallel(&groups);
    for result in &results {
        emit(emitter, render_a_cell("second", &result.not_merged));
        emit(emitter, render_a_cell("second", &result.merged));
        let pair = &result.pair;
        emit(
            emitter,
            format!(
                "name=q2_pair segment=second {} positions={} identical_positions={} not_merged_crossed_more={} merged_crossed_more={} not_merged_lost_more={} merged_lost_more={} largest_crossed_excess_of_not_merged={} largest_lost_excess_of_not_merged={} positions_merged_rejects_more={} largest_rejected_excess_of_merged={} positions_merged_is_all_enospc_against_not_merged={}",
                render_the_pair_key(&result.not_merged.key),
                pair.positions,
                pair.identical_positions,
                pair.not_merged_crossed_more,
                pair.merged_crossed_more,
                pair.not_merged_lost_more,
                pair.merged_lost_more,
                pair.largest_crossed_excess_of_not_merged,
                pair.largest_lost_excess_of_not_merged,
                pair.positions_merged_rejects_more,
                pair.largest_rejected_excess_of_merged,
                pair.positions_merged_is_all_enospc_against_not_merged
            ),
        );
    }
    // 不并什么都不并（登记第十节 F5）：Q1 不并臂上两条 Q2 臂在每个位置上逐项相同。
    let checks = check_the_segment(&results, &|stuck_ceiling| stuck_ceiling == StuckCeilingDisposition::DoNotMergeTheBadSlots);
    emit_the_second_segment_judgments(emitter, tally, &results);
    emit_the_segment_verdict(emitter, tally, "second", checks);
}

fn render_the_pair_key(key: &CellKey) -> String {
    format!(
        "family={} kind={} persistence={} S={} c_max={} reserve={} pool={} q1={}",
        key.family.label(),
        label_of_the_fault_kind(key.fault_kind),
        label_of_the_persistence(key.persistence),
        key.slots_per_region,
        key.metadata_blocks,
        key.reserve_reading.label(),
        key.fullness.label(),
        key.arm.stuck_ceiling.label()
    )
}

/// 准入用的仓根：环境变量 SINGLEFS_REPOSITORY_ROOT，没设取当前目录（replay.sh 与实验页的命令都在仓根起）；不取「清单目录/../..」（单元键把它当运行期路径拒算）。
fn repository_root() -> std::path::PathBuf {
    let root = std::env::var_os("SINGLEFS_REPOSITORY_ROOT")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    if !root.join(".claude/singlefs-ai-sop/scripts/preflight.py").is_file() {
        eprintln!("  ✗ {} 不是仓根（下面没有 .claude/singlefs-ai-sop/scripts/preflight.py），判不了准入", root.display());
        eprintln!("  → 怎么办：在仓根起这个装置，或设 SINGLEFS_REPOSITORY_ROOT=<仓根>");
        std::process::exit(78);
    }
    root
}

fn preflight() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let root = repository_root();
    let source = root.join("scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs");
    let script = root.join(".claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

fn main() {
    preflight();
    let mut emitter = Emitter::new();
    let mut tally = RunTally::default();
    match std::env::args().nth(1).as_deref() {
        Some("checks") => run_the_checks(&mut emitter, &mut tally),
        Some("first") => run_the_first_segment(&mut emitter, &mut tally),
        Some("second") => run_the_second_segment(&mut emitter, &mut tally),
        Some(other) => {
            eprintln!("不认识的子命令：{other}（可用：checks、first、second）");
            std::process::exit(2);
        }
        None => {
            eprintln!("要一个子命令：checks、first、second");
            std::process::exit(2);
        }
    }
    println!("{}", emitter.finish());
    if !tally.halt_conditions.is_empty() {
        std::process::exit(3);
    }
    if !tally.void_conditions.is_empty() {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod pinned_readings {
    use super::*;

    fn arm_of_the_pair(stuck_ceiling: StuckCeilingDisposition, all_zero_slot: AllZeroSlotDisposition) -> Arm {
        arm_of(stuck_ceiling, all_zero_slot)
    }

    fn fault_below_the_newest_root(history: &PoolAfterHistory, positions_below: u64, kind: ReadFaultKind, persistence: FaultPersistence) -> ReadFault {
        ReadFault { slot: history.geometry.slot_of_txg(history.highest_txg_used - positions_below), kind, persistence }
    }

    fn pool_of_the_control_cells(reserve_reading: ReserveReading, fullness: PoolFullness) -> PoolSettings {
        control_pool_settings(reserve_reading, fullness)
    }

    /// 场景：冻结树那份用例的两份盘面（S = 4）上，装置按每一条臂在锚点上做一次准入抬 F。
    /// 预期：有洞读不出 6（K1）；没洞读不出 17 而不并 18（K2、K5）；没洞读回全零 18 越过 1 个、全零并 17（K3）；有洞读回全零或读得出 6（K4）；
    /// 根环表里读回全零的槽钉在 5 与 9，读不出的槽能住的最大 txg 是 17 与 21（K6）。
    #[test]
    fn frozen_tree_use_case_cells_are_reproduced_on_every_arm() {
        let holes = events_of_the_pool_over_the_holes();
        let no_holes = events_of_the_pool_without_holes();
        for stuck_ceiling in StuckCeilingDisposition::every_arm() {
            let over_the_holes = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, Some(ReadFaultKind::DeviceError), ANCHOR_TXG_OVER_THE_HOLES, arm_of_the_pair(stuck_ceiling, AllZeroSlotDisposition::NotMerged));
            assert_eq!(over_the_holes.floor_after, 6, "K1 {stuck_ceiling:?}");
            assert_eq!(over_the_holes.slot_entry_before_the_raise, Some(Occupant::BadAtMount { oldest_txg_it_can_have_held: 9, highest_txg_it_can_hold: 21 }), "K1 的根环表");
            for zero in [AllZeroSlotDisposition::NotMerged, AllZeroSlotDisposition::Merged] {
                for kind in [Some(ReadFaultKind::AllZeroBytes), None] {
                    let outcome = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, kind, ANCHOR_TXG_OVER_THE_HOLES, arm_of_the_pair(stuck_ceiling, zero));
                    assert_eq!(outcome.floor_after, 6, "K4 {stuck_ceiling:?} {zero:?} {kind:?}");
                }
            }
        }
        for kind in [ReadFaultKind::DeviceError, ReadFaultKind::GarbageBytes] {
            for stuck_ceiling in [StuckCeilingDisposition::AsToday, StuckCeilingDisposition::PushEmptyWithinBudget, StuckCeilingDisposition::PushEmptyBeyondBudget] {
                let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(kind), ANCHOR_TXG_WITHOUT_HOLES, arm_of_the_pair(stuck_ceiling, AllZeroSlotDisposition::NotMerged));
                assert_eq!(outcome.floor_after, 17, "K2 {kind:?} {stuck_ceiling:?}");
                assert_eq!(outcome.slot_entry_before_the_raise, Some(Occupant::BadAtMount { oldest_txg_it_can_have_held: 5, highest_txg_it_can_hold: 17 }), "K2 的根环表");
            }
        }
        for stuck_ceiling in [StuckCeilingDisposition::AsToday, StuckCeilingDisposition::PushEmptyWithinBudget, StuckCeilingDisposition::PushEmptyBeyondBudget] {
            let not_merged = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_WITHOUT_HOLES, arm_of_the_pair(stuck_ceiling, AllZeroSlotDisposition::NotMerged));
            assert_eq!((not_merged.floor_after, not_merged.crossed_non_empty_roots), (18, 1), "K3 全零不并 {stuck_ceiling:?}");
            assert!(matches!(not_merged.slot_entry_before_the_raise, Some(Occupant::AllZeroAtMount { oldest_txg_it_can_have_held: 5, .. })), "K6 没洞读回全零的槽钉在 5");
            let merged = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_WITHOUT_HOLES, arm_of_the_pair(stuck_ceiling, AllZeroSlotDisposition::Merged));
            assert_eq!((merged.floor_after, merged.crossed_non_empty_roots), (17, 0), "K3 全零并 {stuck_ceiling:?}");
        }
        for kind in [ReadFaultKind::DeviceError, ReadFaultKind::GarbageBytes, ReadFaultKind::AllZeroBytes] {
            for zero in [AllZeroSlotDisposition::NotMerged, AllZeroSlotDisposition::Merged] {
                let outcome = raise_at_an_anchor(&no_holes, FAULT_TXG_WITHOUT_HOLES, Some(kind), ANCHOR_TXG_WITHOUT_HOLES, arm_of_the_pair(StuckCeilingDisposition::DoNotMergeTheBadSlots, zero));
                assert_eq!(outcome.floor_after, 18, "K5 {kind:?} {zero:?}");
            }
        }
        let zero_over_the_holes = raise_at_an_anchor(&holes, FAULT_TXG_OVER_THE_HOLES, Some(ReadFaultKind::AllZeroBytes), ANCHOR_TXG_OVER_THE_HOLES, arm_of_the_pair(StuckCeilingDisposition::AsToday, AllZeroSlotDisposition::NotMerged));
        assert!(matches!(zero_over_the_holes.slot_entry_before_the_raise, Some(Occupant::AllZeroAtMount { oldest_txg_it_can_have_held: 9, .. })), "K6 有洞读回全零的槽钉在 9");
    }

    /// 场景：B5 那一格（H1，S = 4，c = 5，P保留 = 45，近满 k = 0，不注读故障）挂载之后的第一次请求，八条臂各跑一次。
    /// 预期：N = 50、写行 51、暖机 1 次、环 [41, 52]、待回收 127 块；不够 ⇒ 上限 47、抬 F 那一串 3 次、F 抬到 47、抬 F 期间空闲最低 30、回收 78 块
    /// （待回收剩 127 − 78 + 15 + 13 = 77）、这一次请求在一串抬 F 之后发成、发成之后空闲 − P保留 = 50。
    #[test]
    fn first_request_of_the_b5_cell_follows_the_independent_arithmetic_on_every_arm() {
        let geometry = geometry_of_the_control_pools();
        let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS);
        let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
        for arm in every_arm_pair() {
            let mut session = start_a_session(&history, None, settings, arm, Controls::none(), false);
            assert_eq!((history.highest_txg_used, session.pool.current_txg, session.pool.space.pending_blocks()), (50, 52, 127), "B5 挂载之后 {arm:?}");
            assert_eq!(session.pool.truth_ring.iter().flatten().map(|root| root.checkpoint_txg).min(), Some(41), "环里最旧的根");
            let record = session.run_one_request();
            let raise = session.ledger.raises[0];
            assert!(record.accepted, "B5 发成 {arm:?}");
            assert_eq!((session.ledger.raises.len(), raise.truth_ceiling, raise.new_floor, raise.publishes), (1, 47, 47, 3), "B5 抬 F {arm:?}");
            assert_eq!(session.ledger.lowest_free_blocks_in_the_first_raise, Some(30), "B5 抬 F 期间空闲最低 {arm:?}");
            assert_eq!((record.free_above_the_reserve_after, session.pool.space.pending_blocks(), session.pool.current_txg), (50, 77, 56), "B5 发成之后 {arm:?}");
        }
    }

    /// 场景：抬 F 那一串与暖机的 txg 只由根环落点公式定。
    /// 预期：抬 F 那一串按现行 txg mod 3 = 0 / 1 / 2 是 2 / 3 / 2 次；暖机按写行 txg mod 3 = 0 / 1 / 2 是 1 / 1 / 2 次；保留池条款读法 c_max = 5 / 10 时 45 / 80。
    #[test]
    fn raise_sequence_and_warm_up_lengths_and_reserve_follow_the_region_ownership_and_the_clause() {
        let raise_lengths: Vec<usize> = (0..3).map(|residue| txgs_of_the_publishes_carrying_the_floor_to_every_device(30 + residue).len()).collect();
        assert_eq!(raise_lengths, vec![2, 3, 2]);
        let warm_up_lengths: Vec<usize> = (0..3).map(|residue| warm_up_publish_txgs(60 + residue).len()).collect();
        assert_eq!(warm_up_lengths, vec![1, 1, 2]);
        assert_eq!(txgs_of_the_publishes_carrying_the_floor_to_every_device(31), vec![32, 33, 34]);
        assert_eq!(warm_up_publish_txgs(62), vec![63, 64]);
        assert_eq!(PUBLISHES_PER_ADMISSION_AT_MOST, 8);
        assert_eq!(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes.reserve_blocks(5), 45);
        assert_eq!(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes.reserve_blocks(10), 80);
        assert_eq!(ReserveReading::CodeReadingOneEmptyPublish.reserve_blocks(10), 10);
    }

    /// 场景：阳性对照 PC1–PC4（登记第五节），每条臂各跑一次。
    /// 预期：PC1 把第一次抬 F 的目标抬到上限 + 2，越过 2 个非空根；PC3 把目标改成现行 txg，少掉 3 个状态；PC2 不回收，近满 k = 2、P保留 = c 时被拒 J − 2；
    /// PC4 推空两臂推 5 次 25 块（读坏槽 j = 5，F 先抬到 N − 5），照今天与不并在种下「FloorAlreadyAtTheCeiling 先推一次」之后推 1 次 5 块。
    #[test]
    fn positive_controls_read_their_planted_effect_on_every_arm() {
        let geometry = geometry_of_the_control_pools();
        let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, METADATA_BLOCKS_OF_THE_CONTROL_POOLS);
        let requests = i64::try_from(requests_per_mount(geometry)).expect("请求数");
        for arm in every_arm_pair() {
            let no_spare = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
            let pc1 = run_a_mount(&history, None, no_spare, arm, Controls { first_raise_target_above_the_ceiling_by: Some(2), ..Controls::none() }, false);
            assert_eq!(pc1.raises[0].crossed_non_empty_roots, 2, "PC1 {arm:?}");
            let pc3 = run_a_mount(&history, None, no_spare, arm, Controls { first_raise_target_is_the_current_txg: true, ..Controls::none() }, false);
            assert_eq!(pc3.raises[0].states_lost, 3, "PC3 {arm:?}");
            let two_spare = pool_of_the_control_cells(ReserveReading::CodeReadingOneEmptyPublish, PoolFullness::NearlyFullWithTwoSpareOverwrites);
            let pc2 = run_a_mount(&history, None, two_spare, arm, Controls { reclaim_disabled: true, ..Controls::none() }, false);
            assert_eq!(count_as_i64(rejected_count(&pc2)), requests - 2, "PC2 {arm:?}");
            let pushing = matches!(arm.stuck_ceiling, StuckCeilingDisposition::PushEmptyWithinBudget | StuckCeilingDisposition::PushEmptyBeyondBudget);
            let (history_pc4, fault_pc4) = pc4_history_and_fault_for_the_empty_pushing_arms(if pushing { 45 } else { 47 }, pushing);
            let roomy = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::Roomy);
            let controls = Controls {
                first_request_is_judged_short_throughout: true,
                push_one_empty_publish_on_the_first_floor_already_at_the_ceiling: !pushing,
                ..Controls::none()
            };
            let record = start_a_session(&history_pc4, fault_pc4, roomy, arm, controls, false).run_one_request();
            let expected = if pushing { (5, 25) } else { (1, 5) };
            assert_eq!((record.empty_publishes, record.empty_publish_blocks), expected, "PC4 {arm:?}");
        }
    }

    /// 场景：H1，宽松池，F 先抬到 N − j，读坏槽 j（能住的最大 txg N − j），第一次请求每一次判够不够都当「不够」。
    /// 预期：推空W 推的次数 = max(3S − j − 2, 0)（暖机 1 次）；推空B 受 B 限，= min(那个数, 7)；S = 4、8、16 的每个位置 j = 4..3S − 1 逐位置对上。
    #[test]
    fn empty_publish_counts_match_the_closed_form_at_every_position() {
        for slots_per_region in SLOTS_PER_REGION_CHOICES {
            let geometry = RingGeometry { slots_per_region };
            let ring_length = i64::try_from(geometry.ring_length_in_txgs()).expect("环长");
            let newest_txg = 2 + 4 * geometry.ring_length_in_txgs();
            for stuck_position in 4..ring_length {
                let mut events = HistoryFamily::CrashedAfterOverwrites.events(geometry);
                events.push(HistoryEvent::FloorRaisedWithoutBuildingTheSequence { new_floor: newest_txg - u64::try_from(stuck_position).expect("j") });
                let history = PoolAfterHistory::from_events(geometry, 5, &events);
                let fault = fault_below_the_newest_root(&history, u64::try_from(stuck_position).expect("j"), ReadFaultKind::DeviceError, FaultPersistence::UntilTheProcessWritesTheSlot);
                let roomy = PoolSettings { geometry, metadata_blocks: 5, reserve_reading: ReserveReading::CodeReadingOneEmptyPublish, fullness: PoolFullness::Roomy };
                let controls = Controls { first_request_is_judged_short_throughout: true, ..Controls::none() };
                let needed = u32::try_from((ring_length - stuck_position - 2).max(0)).expect("次数");
                let beyond = start_a_session(&history, Some(fault), roomy, arm_of(StuckCeilingDisposition::PushEmptyBeyondBudget, AllZeroSlotDisposition::NotMerged), controls, false).run_one_request();
                let within = start_a_session(&history, Some(fault), roomy, arm_of(StuckCeilingDisposition::PushEmptyWithinBudget, AllZeroSlotDisposition::NotMerged), controls, false).run_one_request();
                assert_eq!(beyond.empty_publishes, needed, "推空W S={slots_per_region} j={stuck_position}");
                assert_eq!(within.empty_publishes, needed.min(7), "推空B S={slots_per_region} j={stuck_position}");
            }
        }
    }

    /// 场景（M1）：H3（正常卸载，F = V = 50），读坏槽里住着 txg 50（j = 2，恰好是 V）。
    /// 预期：读坏槽能住的最大 txg 50 不低于 F 50，照今天封在 50；读得出的根算出的 C0 是 51。
    #[test]
    fn a_bad_slot_that_can_hold_exactly_the_floor_still_caps_the_ceiling() {
        let geometry = geometry_of_the_control_pools();
        let history = HistoryFamily::UnmountedAfterOverwrites.build_pool_after_history(geometry, 5);
        let fault = fault_below_the_newest_root(&history, 2, ReadFaultKind::DeviceError, FaultPersistence::UntilTheProcessWritesTheSlot);
        let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
        let session = start_a_session(&history, Some(fault), settings, arm_of(StuckCeilingDisposition::AsToday, AllZeroSlotDisposition::NotMerged), Controls::none(), false);
        assert_eq!((history.highest_txg_used, history.effective_floor), (52, 50));
        let reading = session.pool.ceiling_pass(false).expect("没有知道住着根的槽读坏");
        assert_eq!((reading.unmerged, reading.merged), (51, 50));
    }

    /// 场景（M5）：H2，S = 4，e，P1，条款读法的近满 k = 0，推空B，j = 3（读坏槽能住 47 = F）。
    /// 预期：B = 8 时第一次请求推 7 次空发布后被预算拒、第二次请求 F 抬到 50；B = 9 时第二次请求 F 抬到 53。
    #[test]
    fn the_admission_budget_of_eight_decides_how_far_the_floor_moves_in_the_second_request() {
        let geometry = geometry_of_the_control_pools();
        let history = HistoryFamily::FloorRaisedAfterOverwrites.build_pool_after_history(geometry, 5);
        let fault = fault_below_the_newest_root(&history, 3, ReadFaultKind::DeviceError, FaultPersistence::UntilTheProcessWritesTheSlot);
        let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
        let run = run_a_mount(&history, Some(fault), settings, arm_of(StuckCeilingDisposition::PushEmptyWithinBudget, AllZeroSlotDisposition::NotMerged), Controls::none(), false);
        assert_eq!((run.requests[0].empty_publishes, run.requests[0].rejection, run.requests[0].floor_after), (7, Some(RejectionReason::AdmissionBudgetUsedUp), 47));
        assert_eq!((run.requests[1].rejection, run.requests[1].floor_after, run.requests[1].publishes_in_the_admission), (Some(RejectionReason::AdmissionBudgetUsedUp), 50, 5));
        let at_nine = run_a_mount(&history, Some(fault), settings, arm_of(StuckCeilingDisposition::PushEmptyWithinBudget, AllZeroSlotDisposition::NotMerged), Controls { publishes_per_admission_at_most: 9, ..Controls::none() }, false);
        assert_eq!(at_nine.requests[1].floor_after, 53, "B = 9 时第二次请求 F 抬到 53，与 B = 8 不同");
    }

    /// 场景（M7）：推空B，H1 先抬 F 到 N − 5，读坏槽 j = 5（PC4 那一形），条款读法的近满 k = 0（空闲 = P保留 = 45）。
    /// 预期：推空的空发布只要求空闲 ≥ c，不判空间准入式子（空闲 − P保留 ≥ c），所以第一次请求推 5 次空发布（25 块），再被预算拒。
    #[test]
    fn empty_publishes_for_a_stuck_ceiling_are_not_judged_by_the_admission_formula() {
        let (history, fault) = pc4_history_and_fault_for_the_empty_pushing_arms(45, true);
        let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
        let run = run_a_mount(&history, fault, settings, arm_of(StuckCeilingDisposition::PushEmptyWithinBudget, AllZeroSlotDisposition::NotMerged), Controls::none(), false);
        let first = &run.requests[0];
        assert_eq!((first.empty_publishes, first.empty_publish_blocks, first.rejection), (5, 25, Some(RejectionReason::AdmissionBudgetUsedUp)));
        assert_eq!(run.empty_publish_refusals, 0);
    }

    /// 场景（M10）：H1，读坏槽 j = 10，持续读坏、写了也坏（P2），照今天，条款读法的近满 k = 0。
    /// 预期：这个进程在暖机之前就写过 X 那一槽（它成了「知道住着根的槽」），算上限那一遍重读仍坏，抬 F 报自己的错：每一次请求都被这个拒因拒、F 一直是 0。
    #[test]
    fn a_slot_this_process_wrote_that_still_reads_bad_stops_every_floor_raise_with_its_own_error() {
        let geometry = geometry_of_the_control_pools();
        let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, 5);
        let fault = fault_below_the_newest_root(&history, 10, ReadFaultKind::DeviceError, FaultPersistence::EvenAfterTheProcessWritesTheSlot);
        let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
        let run = run_a_mount(&history, Some(fault), settings, arm_of(StuckCeilingDisposition::AsToday, AllZeroSlotDisposition::NotMerged), Controls::none(), false);
        for request in run.requests.iter().take(3) {
            assert_eq!((request.accepted, request.rejection, request.floor_after), (false, Some(RejectionReason::FloorRaiseReportedItsOwnError), 0));
        }
        assert_eq!(run.raises.len(), 0);
    }

    /// 场景（M8）：H1，S = 4，e，读坏槽 j = 5 只在挂载里读坏（T1），条款读法的近满 k = 2，照今天。
    /// 预期：挂载做完时那一槽还钉着（可能住过的最旧 txg 9、能住的最大 txg 45），门槛压在 9、待回收 543 块；第一次发布记根时进程内重读读出有效根，
    /// 换成它的 txg 45，门槛升到环里最旧有效根，待回收收到 127 块、空闲 487 块。
    #[test]
    fn rereading_a_pinned_slot_after_a_transient_fault_lifts_the_reclaim_floor() {
        let geometry = geometry_of_the_control_pools();
        let history = HistoryFamily::CrashedAfterOverwrites.build_pool_after_history(geometry, 5);
        let fault = fault_below_the_newest_root(&history, 5, ReadFaultKind::DeviceError, FaultPersistence::DuringTheMountOnly);
        let settings = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithTwoSpareOverwrites);
        let mut session = start_a_session(&history, Some(fault), settings, arm_of(StuckCeilingDisposition::AsToday, AllZeroSlotDisposition::NotMerged), Controls::none(), false);
        assert_eq!(session.pool.table.occupants.get(&fault.slot), Some(&Occupant::BadAtMount { oldest_txg_it_can_have_held: 9, highest_txg_it_can_hold: 45 }));
        assert_eq!((session.pool.space.pending_blocks(), session.pool.space.free_blocks), (543, 71));
        let record = session.run_one_request();
        assert!(record.accepted);
        assert_eq!(session.pool.table.occupants.get(&fault.slot), Some(&Occupant::ValidRootReadWhenRereading { checkpoint_txg: 45 }));
        assert_eq!((session.pool.space.pending_blocks(), session.pool.space.free_blocks), (127, 487));
    }

    /// 场景：第二段 z 族的一组（H1，S = 4，c = 5，条款读法，近满 k = 0，P1，照今天）与 H5 的一组，两条 Q2 臂各跑。
    /// 预期：H1–H4 每格 3S − 1 = 11 个位置；H5 一个位置；Q1 不并臂上两条 Q2 臂逐位置相同；同一组算两遍结果逐项相同（确定性模型，没有隐藏状态）。
    #[test]
    fn positions_per_cell_and_the_q2_arms_agree_where_the_no_merge_arm_never_merges() {
        let pool = pool_of_the_control_cells(ReserveReading::ClauseReadingTenPlusSevenEmptyPublishes, PoolFullness::NearlyFullWithNoSpareOverwrite);
        let z_group = GroupSettings {
            family: HistoryFamily::CrashedAfterOverwrites,
            fault_kind: Some(ReadFaultKind::AllZeroBytes),
            persistence: Some(FaultPersistence::UntilTheProcessWritesTheSlot),
            pool,
            stuck_ceiling: StuckCeilingDisposition::DoNotMergeTheBadSlots,
        };
        let first = run_a_group(&z_group);
        let second = run_a_group(&z_group);
        assert_eq!((first.not_merged.positions, first.not_merged.expected_positions), (11, 11));
        assert_eq!(first.pair.identical_positions, first.pair.positions, "不并臂上两条 Q2 臂相同");
        assert_eq!(
            (first.pair.not_merged_crossed_more, first.pair.merged_crossed_more, first.pair.not_merged_lost_more, first.pair.merged_lost_more),
            (0, 0, 0, 0),
            "逐位置相同时谁都不比谁多"
        );
        assert_eq!(first.not_merged.quantities, second.not_merged.quantities);
        let holes_group = GroupSettings { family: HistoryFamily::YoungRingWithCrashHoles, fault_kind: None, persistence: None, pool, stuck_ceiling: StuckCeilingDisposition::DoNotMergeTheBadSlots };
        let holes = run_a_group(&holes_group);
        assert_eq!((holes.not_merged.positions, holes.not_merged.expected_positions), (1, 1));
        assert_eq!(holes.pair.identical_positions, holes.pair.positions, "H5 上不并臂也不并任何槽");
    }
}

#[cfg(test)]
mod measurement_readings {
    use super::*;

    fn request(accepted: bool, rejection: Option<RejectionReason>, was_judged_short: bool, floor_after: u64, candidates: usize) -> RequestRecord {
        RequestRecord {
            accepted,
            rejection,
            was_judged_short,
            empty_publishes: 0,
            empty_publish_blocks: 0,
            publishes_in_the_admission: usize::from(accepted),
            crossed_non_empty_roots: 0,
            states_lost: 0,
            floor_after,
            free_above_the_reserve_after: 7,
            candidate_states_after: candidates,
        }
    }

    fn run_of(requests: Vec<RequestRecord>, raises: Vec<RaiseRecord>) -> MountRun {
        MountRun {
            requests,
            raises,
            candidate_states_at_the_start: 4,
            cap_active_passes: 0,
            cap_blocked_passes: 0,
            empty_publish_refusals: 0,
            raise_refused_after_an_empty_publish: 0,
            empty_publish_bound_exhausted: 0,
            lowest_free_blocks_in_the_first_raise: None,
            trace: Vec::new(),
        }
    }

    /// 场景：基线 5 次请求全发成；注了读故障的那一次第 2 次起全被拒（第 2 次基线发成、这一次拒）。
    /// 预期：多拒 4、第一次多拒落在第 1 步（从 0 数）、之后发成 0 次、一次挂载全 ENOSPC 成立；轨迹的峰值取第一次「不够」之后的最大值、为正的步数数全程、期末值取最后一步。
    #[test]
    fn extra_rejections_the_first_extra_step_and_the_trajectory_are_read_from_the_two_runs() {
        let accepted = |floor| request(true, None, false, floor, 4);
        let rejected = |floor| request(false, Some(RejectionReason::FloorAlreadyAtTheCeiling), true, floor, 3);
        let baseline = run_of(vec![accepted(0), accepted(0), accepted(0), accepted(0), accepted(0)], Vec::new());
        let with_fault = run_of(vec![accepted(0), rejected(0), rejected(2), rejected(5), rejected(5)], Vec::new());
        let metrics = metrics_of(&with_fault, &baseline);
        assert_eq!((metrics.rejected, metrics.extra_rejected, metrics.first_extra_step, metrics.accepted_after_the_first_extra, metrics.all_enospc), (4, 4, 1, 0, 1));
        assert_eq!(metrics.rejected_by_reason[RejectionReason::FloorAlreadyAtTheCeiling.index()], 4);
        assert_eq!(metrics.trajectories[0], Trajectory { peak_after_the_first_short: Some(1), positive_steps: 4, final_value: 1 });
        assert_eq!(metrics.trajectories[4], Trajectory { peak_after_the_first_short: Some(5), positive_steps: 3, final_value: 5 });
        assert_eq!((metrics.candidates_lowest, metrics.candidates_final), (3, 3));
        let one_accepted_afterwards = run_of(vec![accepted(0), rejected(0), accepted(0), rejected(0), rejected(0)], Vec::new());
        let later = metrics_of(&one_accepted_afterwards, &baseline);
        assert_eq!((later.extra_rejected, later.first_extra_step, later.accepted_after_the_first_extra, later.all_enospc), (3, 1, 1, 0));
    }

    /// 场景：两次抬 F：一次 F 18 越过 1 个非空根、少掉 2 个状态；一次 F 50 越过 0 个、没越过真值上限。
    /// 预期：越过总数 1、一次抬 F 里越过的峰值 1、F 高于真值上限的步数 1、少掉的状态总数 2、一次抬 F 里少掉的最大 2。
    #[test]
    fn raise_records_fold_into_crossed_over_truth_and_lost_counts() {
        let raises = vec![
            RaiseRecord { old_floor: 0, new_floor: 18, truth_ceiling: 17, crossed_non_empty_roots: 1, states_lost: 2, publishes: 2 },
            RaiseRecord { old_floor: 18, new_floor: 50, truth_ceiling: 50, crossed_non_empty_roots: 0, states_lost: 0, publishes: 3 },
        ];
        let run = run_of(vec![request(true, None, false, 50, 4)], raises);
        let metrics = metrics_of(&run, &run_of(vec![request(true, None, false, 50, 4)], Vec::new()));
        assert_eq!((metrics.crossed, metrics.crossed_peak_in_one_raise, metrics.over_truth_steps), (1, 1, 1));
        assert_eq!((metrics.lost_total, metrics.most_states_lost_in_one_raise, metrics.raise_sequences), (2, 2, 2));
    }

    /// 场景：一格里 4 个位置上的同一个量取 3、0、7、7。
    /// 预期：最大 7（取到最大的位置按先到的 2）、最小 0、量 > 0 的位置 3、位置总数 4；渲染成「7@2/0/3/4」。
    #[test]
    fn aggregate_keeps_the_maximum_its_first_position_the_minimum_and_the_positive_count() {
        let mut aggregate = Aggregate::empty();
        for (position, value) in [(1usize, 3i64), (2, 0), (3, 7), (4, 7)] {
            aggregate.add(position, value);
        }
        assert_eq!((aggregate.maximum, aggregate.maximum_at_position, aggregate.minimum, aggregate.positive_positions, aggregate.positions), (7, 3, 0, 3, 4));
        assert_eq!(aggregate.render(), "7@3/0/3/4");
        assert_eq!(Aggregate::empty().render(), "none");
    }

    /// 场景：一条判定在两个点上各有两格：A 点两格的值是 0 和 1，B 点两格的值是 0 和 0。
    /// 预期：A 点打中（最大值 1、打中的格 1 / 2）、B 点没打中；门槛挪到最大值 + 1 之后 A 点由真转假。
    #[test]
    fn a_judgment_hits_at_one_and_the_shifted_threshold_flips_it() {
        let key = |slots_per_region: u64| CellKey {
            family: HistoryFamily::CrashedAfterOverwrites,
            fault_kind: Some(ReadFaultKind::DeviceError),
            persistence: Some(FaultPersistence::UntilTheProcessWritesTheSlot),
            slots_per_region,
            metadata_blocks: 5,
            reserve_reading: ReserveReading::CodeReadingOneEmptyPublish,
            fullness: PoolFullness::NearlyFullWithNoSpareOverwrite,
            arm: arm_of(StuckCeilingDisposition::AsToday, AllZeroSlotDisposition::NotMerged),
        };
        let cell_with = |slots_per_region: u64, value: i64| {
            let mut cell = CellSummary::new(key(slots_per_region), 1, 0);
            cell.quantities.insert("extra_rejected", {
                let mut aggregate = Aggregate::empty();
                aggregate.add(1, value);
                aggregate
            });
            cell
        };
        let cells = [cell_with(4, 0), cell_with(4, 1), cell_with(8, 0), cell_with(8, 0)];
        let references: Vec<&CellSummary> = cells.iter().collect();
        let settings = JudgmentSettings {
            judgment_label: "test",
            scope: "test",
            value: &|cell| Some(maximum_of(cell, "extra_rejected")),
            path_evidence: &|_| 1,
        };
        let points = collect_the_points(&references, &settings);
        let point_of = |slots_per_region: u64| points.iter().find(|(point, _)| point.slots_per_region == slots_per_region).map(|(_, totals)| *totals).expect("这个点有格");
        assert_eq!((point_of(4).observed_maximum, point_of(4).cells_hit, point_of(4).cells), (Some(1), 1, 2));
        assert_eq!((point_of(8).observed_maximum, point_of(8).cells_hit, point_of(8).cells), (Some(0), 0, 2));
        let mut emitter = Emitter::new();
        let per_arm = emit_a_judgment(&mut emitter, "test", &references, &settings);
        assert_eq!(per_arm.get("today"), Some(&(1, 2, Some(1))), "两个点里 A 点打中");
        let mut tally = RunTally::default();
        emit_the_threshold_shift_check(&mut emitter, &mut tally, "test", "test", &per_arm);
        assert_eq!((tally.self_checks_passed, tally.void_conditions.len()), (1, 0), "门槛挪到 2 之后由真转假");
    }
}

#[cfg(test)]
mod boundary_readings {
    use super::*;

    /// 场景：峰值取「第一次不够那一步起」的最大值，最大值恰好落在第一次不够那一步。
    /// 预期：序列 0、9、3、2 在第 1 步第一次不够时峰值 9、为正的步数 3、期末 2；从没不够过时峰值没有（none），不当 0 报。
    #[test]
    fn trajectory_peak_starts_at_the_first_short_step_and_is_none_when_never_short() {
        let series = [0i64, 9, 3, 2];
        assert_eq!(trajectory_of(&series, Some(1)), Trajectory { peak_after_the_first_short: Some(9), positive_steps: 3, final_value: 2 });
        assert_eq!(trajectory_of(&series, None).peak_after_the_first_short, None);
        let mut aggregate = TrajectoryAggregate::empty();
        aggregate.add(trajectory_of(&series, Some(1)));
        aggregate.add(trajectory_of(&series, None));
        assert_eq!((aggregate.peak_maximum, aggregate.peak_minimum, aggregate.positions_never_short), (Some(9), Some(9), 1));
    }

    /// 场景：注了读故障的那一次比基线恰好多拒 1 次，而且是最后一次请求。
    /// 预期：多拒 1 就够「一次挂载全 ENOSPC」成立（门槛是 ≥ 1，不是 > 1）：第一次多拒落在第 4 步、之后发成 0 次。
    #[test]
    fn exactly_one_extra_rejection_at_the_end_is_already_an_all_enospc_mount() {
        let accepted = RequestRecord {
            accepted: true,
            rejection: None,
            was_judged_short: false,
            empty_publishes: 0,
            empty_publish_blocks: 0,
            publishes_in_the_admission: 1,
            crossed_non_empty_roots: 0,
            states_lost: 0,
            floor_after: 0,
            free_above_the_reserve_after: 3,
            candidate_states_after: 4,
        };
        let rejected = RequestRecord { accepted: false, rejection: Some(RejectionReason::AdmissionBudgetUsedUp), was_judged_short: true, ..accepted.clone() };
        let make_run = |requests: Vec<RequestRecord>| MountRun {
            requests,
            raises: Vec::new(),
            candidate_states_at_the_start: 4,
            cap_active_passes: 0,
            cap_blocked_passes: 0,
            empty_publish_refusals: 0,
            raise_refused_after_an_empty_publish: 0,
            empty_publish_bound_exhausted: 0,
            lowest_free_blocks_in_the_first_raise: None,
            trace: Vec::new(),
        };
        let baseline = make_run(vec![accepted.clone(); 5]);
        let with_fault = make_run(vec![accepted.clone(), accepted.clone(), accepted.clone(), accepted.clone(), rejected]);
        let metrics = metrics_of(&with_fault, &baseline);
        assert_eq!((metrics.extra_rejected, metrics.first_extra_step, metrics.accepted_after_the_first_extra, metrics.all_enospc), (1, 4, 0, 1));
    }
}

//! 一块盘的空闲图 `DeviceFreeMap`：单元区起点、槽与槽对的计数、各段的空闲情况、回收复用的窗口与按段找用户数据槽。

use super::*;

/// 一块盘的单元区从哪个 16 KiB 槽起：journal 环末尾的下一个槽，随环长走（D23（journal 的角色与格式） 已定项 19 ③
/// 「单元区起始槽号随环长走，默认环下是 784 MiB（槽 50176）」；D3（空间分配） 已定项 10 ④「第一版 = journal 环末尾的下一个槽」；
/// 由环长算的只有 `journal::slot_after_the_journal_ring` 一处）。
///
/// **不变量：起点落在聚簇段边界上**（槽号是 [`CLUSTER_SEGMENT_SLOTS`] 的整数倍），只从 [`UnitAreaStart::following_the_journal_ring`] 进来。
/// 数据单元起点 32768 对齐（D3（空间分配） 已定项 10 ③：偶数槽）与聚簇段「64 槽对齐」（已定项 10 ①）说的都是设备内的槽号，
/// 而分配器从起点起按 2 槽、64 槽步进；起点在段边界上时两种对齐才都成立。起点不在段边界上时聚簇段按槽号对齐还是按单元区对齐、
/// 起点到下一个段边界那几槽算不算一段（「全空聚簇段数」那一行记账跟着变），条款没写 ⇒ 第一版不支持
/// （[`UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`]）。默认环（768 MiB）与环长是 1 MiB 整数倍的环都落在段边界上。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitAreaStart(SlotNumber);

/// 环长算出的单元区起点不落在聚簇段边界上（[`UnitAreaStart`] 的不变量）：条款没写这样的单元区怎么分段，第一版不支持。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitAreaStartOffTheClusterSegmentBoundaryUnsupported {
    pub journal_ring_bytes: u64,
    pub slot_after_the_journal_ring: SlotNumber,
}

impl UnitAreaStart {
    /// 环长 `journal_ring_bytes` 下的单元区起点：journal 环末尾的下一个槽（`journal::slot_after_the_journal_ring`）。
    ///
    /// # Errors
    /// 那个槽不落在聚簇段边界上 ⇒ [`UnitAreaStartOffTheClusterSegmentBoundaryUnsupported`]。
    pub fn following_the_journal_ring(
        journal_ring_bytes: u64,
    ) -> Result<Self, UnitAreaStartOffTheClusterSegmentBoundaryUnsupported> {
        let slot = slot_after_the_journal_ring(journal_ring_bytes);
        if !slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {
            return Err(UnitAreaStartOffTheClusterSegmentBoundaryUnsupported {
                journal_ring_bytes,
                slot_after_the_journal_ring: slot,
            });
        }
        Ok(Self(slot))
    }

    /// 默认环长（`JOURNAL_RING_DEFAULT_BYTES`，768 MiB）下的单元区起点：槽 50176（784 MiB），与格式常量 `UNIT_AREA_START_SLOT` 同一个数
    /// （有用例钉住）。
    #[must_use]
    pub fn of_the_default_journal_ring() -> Self {
        Self::following_the_journal_ring(JOURNAL_RING_DEFAULT_BYTES).expect(
            "默认环 768 MiB 从槽 1024 起、占 49152 槽，下一个槽 50176 = 784 × 64 落在聚簇段边界上",
        )
    }

    #[must_use]
    pub fn slot(self) -> SlotNumber {
        self.0
    }
}

/// 一块盘上的槽对数：从偶数下标起、不跨聚簇段的两槽算一对（[`DeviceFreeMap::lowest_user_data_slot`] 认的那一种），不是槽数。
/// 一个数据单元落一对；换算成字节先乘一个数据单元的槽数。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotPairCount(pub u64);

impl std::iter::Sum for SlotPairCount {
    fn sum<SlotPairCounts: Iterator<Item = Self>>(counts: SlotPairCounts) -> Self {
        SlotPairCount(counts.map(|count| count.0).sum())
    }
}

/// 一块设备的空闲图与它的记账增量。
#[derive(Clone, Debug)]
pub struct DeviceFreeMap {
    pub device: DeviceIdentity,
    /// 建这张空闲图时这块盘的字节数：单元区槽数由它算（[`unit_area_slots_of_device_starting_at`]），分配记录树写侧的几何也由它算
    /// （`AllocationRecordTreeGeometry::of_allocator`，与读侧按读者报的字节数走同一个函数）。
    device_size_in_bytes: u64,
    /// 单元区从哪个槽起：位图下标 0 对着它，三种落点（用户数据、开段、提交内生块的回落）都从它起扫。
    unit_area_start: UnitAreaStart,
    pub(super) unit_area_slots: u64,
    /// 每个 16 KiB 槽一位：true = 已分配。
    allocated: Vec<bool>,
    /// 每个 64 槽聚簇段里分配位置着的槽数（仍分配的与已释放、还在 defer 窗口里的都算）：开段、discard 判整段可再分配用。
    used_per_segment: Vec<u64>,
    /// 每个 64 槽聚簇段里罩在未释放分配记录下的槽数：仍分配的算，已释放、还在 defer 窗口里的不算（D3（空间分配） 已定项 10 ① 的
    /// 记录口径，C372（全空聚簇段数两份定义相反））。与 `used_per_segment` 只差 defer 里的那几槽。分配与复活时加、释放时减，
    /// 回收不动（回收的本来就是已释放的）。
    unreleased_per_segment: Vec<u64>,
    /// 单元区里凑满 64 槽的整段中 `unreleased_per_segment` 为 0 的段数：记账第 11 项「全空聚簇段数」（[`Self::empty_segments`]），
    /// 随 `unreleased_per_segment` 每变一次增量维护，不扫段（`.claude/rules/fs-design.md`「记账是事务的副产品」：合取 7 在准入路径上读它）。
    full_segments_without_an_unreleased_record: u64,
    /// 空闲槽的连续段数（增量维护）。
    pub(super) free_runs: u64,
    /// 占着的槽数：仍分配的加上已释放、还在 defer 窗口里的——它们仍被根环里的有效根引用、仍占着空间
    /// （I-3.1（已分配统计对得上） 的读法：按根环里全部有效根的引用取并集）。
    pub(super) allocated_slots: u64,
    /// 空闲槽数，独立维护：分配时减、回收放回时加，不由「容量 − 已分配」现算（D5（快照 / 空间记账机制） 已定项 4 的 ⚠️：
    /// 那样 I-5.2（空闲统计对得上） 是恒真式）。已释放而还在 defer 窗口里的不算空闲。
    pub(super) free_slots: u64,
    /// 影子账（D23（journal 的角色与格式） 已定项 14 回退段与「可写挂载时的预隔离」；D28（挂载期承诺量） 已定项 1 第九项）隔离的槽，两部分置同一位：
    /// 只被被抛弃根引用的槽，隔离到那条被抛弃的根离开根环；可写挂载那一刻水位之上没施加的 journal 记录点名的落点与那几次发布的账里的落点
    /// （C694 的 C，分配位置着的槽除外：现行账里仍分配的与已释放、还在 defer 窗口里的，它们在「已分配」里；回收时补隔离），
    /// 隔离到这个进程写掉那条记录的 txg 对应的根环槽（`RootRingOccupancy` 里记着的那一份）。
    /// 隔离位独立于分配位（[`Self::isolate`]）。只住内存、不进记账行。
    pub(super) isolated: Vec<bool>,
    isolated_per_segment: Vec<u64>,
    isolated_slots: u64,
    /// 其中已释放、还在 defer 窗口里的槽数（D5（快照 / 空间记账机制） 已定项 4 第 5 项）。
    pub(super) deferred_slots: u64,
    /// 抬 F 回收、但 F 还没在每块盘上生效的槽：记账已经算它空闲，分配器却不许发出去，直到带新 F 的根落满每块盘
    /// （D16（发布语义） 已定项 1「每块幸存盘上都有带新 F 的持久根才生效」；抬 F 自己的空发布开新段时
    /// 正好开到刚回收空的那一段，崩在两条带新 F 的根之间时 F 之下的根仍是候选、它们的单元已被盖）。只住内存。
    held_until_floor_takes_effect: Vec<bool>,
    held_per_segment: Vec<u64>,
    /// 已分配、影子账隔离、抬 F 扣住三种位一个都没置的槽数：提交内生块落得下的槽（开放段 bump、开全空段、回落挡的都是这三种位，
    /// [`Self::is_blocked_for_commit_generated`]），三种位每变一次跟着增量维护（[`Self::set_blocking_bit`]）。
    /// 发布路径在走固定点之前判「这次的单元落得下」读它（C545（空间准入罩不住分裂与聚簇段层））。
    unblocked_slots: u64,
    /// 每个 64 槽聚簇段里两槽都没挡（三种位都没置）的偶数起点槽对数：用户数据落得下的槽对（[`Self::lowest_user_data_slot`] 认的那一种，
    /// 段内外之分由读的一方减），同上增量维护。单元区起点在段边界上（[`UnitAreaStart`] 的不变量）、段长是偶数 ⇒ 一对不跨段；
    /// 单元区末尾凑不成一对的那一槽不算。
    unblocked_slot_pairs_per_segment: Vec<SlotPairCount>,
    /// 上一项全部段之和。
    unblocked_slot_pairs: SlotPairCount,
    /// `unblocked_slot_pairs_per_segment` 不是 0 的那几段的段号（从单元区起点起数）：用户数据落点按段号升序走它
    /// （[`Self::lowest_user_data_slot_outside_the_region`]），不从单元区起点逐个候选看。那一段的槽对数在 0 与非 0 之间翻过去时
    /// 跟着加进或摘掉（[`Self::set_blocking_bit`]），与那一项同一处维护。
    segments_with_an_unblocked_slot_pair: BTreeSet<usize>,
    /// 每一槽是不是罩在未释放的分配记录下（`unreleased_per_segment` 数的那几槽，逐槽一位）：分配与复活时置、释放时清，回收不动。
    /// 不罩在未释放记录下的槽只有四种：空闲的、defer 里的、影子账隔离的、抬 F 扣住的——抬 F 能放开的就是后三种。
    is_under_an_unreleased_record: Vec<bool>,
    /// 每个 64 槽聚簇段里两槽都不罩在未释放分配记录下的偶数起点槽对数：defer、影子账隔离、抬 F 扣住三样全放开之后用户数据落得下的槽对
    /// （抬 F 之前先算槽对那一判的上界读它，`mounted_session::most_a_floor_raise_could_make_available`），
    /// 随 `is_under_an_unreleased_record` 每变一槽增量维护。一对不跨段、单元区末尾凑不成一对的那一槽不算，同 `unblocked_slot_pairs_per_segment`。
    slot_pairs_without_an_unreleased_record_per_segment: Vec<SlotPairCount>,
    /// 上一项全部段之和。
    slot_pairs_without_an_unreleased_record: SlotPairCount,
    /// 分配记录树每片叶里三种位都没置的单元区槽数（叶序号 = 槽号 ÷ [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]，下标 = 叶序号 − 单元区起点所在那片叶的序号；
    /// 单元区首尾那两片叶只数单元区里的那一截）。回落取空槽最多的叶时拿它当每片叶「每块盘都空的槽数」的上界
    /// （[`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]），同上增量维护。
    unblocked_slots_per_allocation_record_tree_leaf: Vec<SlotCountInALeaf>,
    /// 回收过槽、还没发过 discard 的段（段号）：回收那一刻记（[`Self::mark_reclaimed`]），发布返回之后整段可再分配时发 discard 并摘掉
    /// 只住内存。
    segments_reclaimed_not_yet_discarded: BTreeSet<usize>,
}

/// 一段槽进出未释放分配记录的方向（[`DeviceFreeMap::empty_segments`] 的记录口径按它增量维护）。封闭集合。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnreleasedRecordTransition {
    /// 分配，或管理员回退把 defer 里的已释放记录复活成已分配。
    SlotsComeUnderAnUnreleasedRecord,
    /// 释放：记录改写成已释放、槽进 defer 队列。
    SlotsLeaveTheirUnreleasedRecord,
}

/// 挡住一个槽的三种位（[`DeviceFreeMap`] 的三张位图）：一个都没置的槽才发得出去。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotBlockingBit {
    Allocated,
    IsolatedByTheShadowLedger,
    HeldUntilFloorTakesEffect,
}

/// 一块盘上有几个完整的 16 KiB 槽（盘上绝对槽数，槽号从 0 起数到它为止）：字节数 ÷ 16384，盘尾不足一槽的零头不算。
/// **这是这个量唯一的一处定义**：单元区槽数（[`unit_area_slots_of_device`]）与分配记录树的根层级
/// （`crate::allocation_record_tree::AllocationRecordTreeGeometry`：写侧 `of_allocator`、读侧 `of_reader` 同走
/// `of_device_sizes_in_bytes`）都从它算，不各算一份：
/// 各算一份时，算的是同一个量、却没有东西把两份连起来。
#[must_use]
pub fn absolute_slot_count_of_device(device_bytes: u64) -> u64 {
    device_bytes / SLOT_BYTES
}

/// 一块盘的完整槽数不到单元区起点：盘末尾落在单元区起点之前，这块盘上没有单元区（代码审阅第 35 条）。
/// 盘的字节数是块设备报的、或崩溃镜像自述的，可以是任何值：挂载入口与恢复判分配记录落点那一道都按它报错，不拿它去减。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceEndsBeforeTheUnitAreaStart {
    pub device_bytes: u64,
    pub unit_area_start: SlotNumber,
}

/// 一块盘的单元区有几个 16 KiB 槽：从 `unit_area_start` 起到设备末尾（[`absolute_slot_count_of_device`]）。
/// **这是这个量唯一的一处定义**：[`DeviceFreeMap::with_unit_area_start`] 与恢复那一侧「分配记录的跨度在不在单元区内」
/// 那一道判（`recovery::allocation_records_fit_the_pool_geometry`）都读它，两处手抄会分叉。
///
/// # Errors
/// 设备末尾在单元区起点之前 ⇒ [`DeviceEndsBeforeTheUnitAreaStart`]。
pub fn unit_area_slots_of_device_starting_at(
    device_bytes: u64,
    unit_area_start: UnitAreaStart,
) -> Result<u64, DeviceEndsBeforeTheUnitAreaStart> {
    absolute_slot_count_of_device(device_bytes)
        .checked_sub(unit_area_start.slot().0)
        .ok_or(DeviceEndsBeforeTheUnitAreaStart {
            device_bytes,
            unit_area_start: unit_area_start.slot(),
        })
}

/// 默认环长下（[`UnitAreaStart::of_the_default_journal_ring`]，槽 50176 起）一块盘的单元区槽数：
/// [`unit_area_slots_of_device_starting_at`] 取默认环的起点。
///
/// # Panics
/// 设备末尾在单元区起点之前：调用方交进来的是判过的盘——mkfs 拒单元区在盘外的几何（`UnitAreaBeyondDevice`），
/// 可写挂载在读盘之前按 `recovery::every_device_reaches_the_unit_area_start` 拒（代码审阅第 35 条）。
#[must_use]
pub fn unit_area_slots_of_device(device_bytes: u64) -> u64 {
    unit_area_slots_of_device_starting_at(
        device_bytes,
        UnitAreaStart::of_the_default_journal_ring(),
    )
    .expect("盘末尾不在单元区起点之前：mkfs 判过 UnitAreaBeyondDevice，可写挂载在读盘之前判过 every_device_reaches_the_unit_area_start")
}

/// 按段维护的槽对数（`pairs_per_segment`，下标是从 `unit_area_start` 起的段号）里落在 `cluster_segments` 那几段的和：
/// 段外槽对数 = 总数减它（[`DeviceFreeMap::unblocked_slot_pairs_outside_the_cluster_segments`] 与
/// [`DeviceFreeMap::slot_pairs_without_an_unreleased_record_outside_the_cluster_segments`] 共用）。
///
/// # Panics
/// `cluster_segments` 里有一段的起点不在这块盘单元区里的段边界上：那几段都是各盘一致答出来的全空段起点
/// （[`DeviceFreeMap::lowest_empty_segment`]），每块盘上都是整段、在单元区里。
fn pairs_inside_the_cluster_segments(
    unit_area_start: UnitAreaStart,
    pairs_per_segment: &[SlotPairCount],
    cluster_segments: &BTreeSet<SlotNumber>,
) -> SlotPairCount {
    cluster_segments
        .iter()
        .map(|segment_start| {
            let offset_in_slots = segment_start
                .0
                .checked_sub(unit_area_start.slot().0)
                .expect("聚簇段在单元区里：它是各盘一致答出来的全空段起点");
            assert!(
                offset_in_slots.is_multiple_of(CLUSTER_SEGMENT_SLOTS),
                "聚簇段起点在段边界上：{segment_start:?}"
            );
            let segment =
                usize::try_from(offset_in_slots / CLUSTER_SEGMENT_SLOTS).expect("段号装得进 usize");
            *pairs_per_segment
                .get(segment)
                .expect("聚簇段在这块盘的单元区里：各盘一致答出来的整段")
        })
        .sum()
}

impl DeviceFreeMap {
    /// 单元区从默认环长下的起点（[`UnitAreaStart::of_the_default_journal_ring`]，槽 50176）起到设备末尾；mkfs 占的槽由调用方标上。
    /// 实例表、树表的落点（`make_filesystem::INSTANCE_TABLE_SLOT`）、恢复判分配记录落点、系统配置里写的单元区起点今天都还按这个数，
    /// 挂载与 mkfs 建空闲图都走这里（C475（非默认环长下单元区起点取编译期常量） 那一半还开着）。
    #[must_use]
    pub fn new(device: DeviceIdentity, device_bytes: u64) -> Self {
        Self::with_unit_area_start(
            device,
            device_bytes,
            UnitAreaStart::of_the_default_journal_ring(),
        )
    }

    /// 单元区从 `unit_area_start` 起到设备末尾的空闲图；mkfs 占的槽由调用方标上。
    ///
    /// # Panics
    /// 设备末尾在单元区起点之前（[`unit_area_slots_of_device_starting_at`] 交回 [`DeviceEndsBeforeTheUnitAreaStart`]）：
    /// 建空闲图的两处都先判过盘容量——mkfs 的 `UnitAreaBeyondDevice`，可写挂载读盘之前的
    /// `recovery::every_device_reaches_the_unit_area_start`（代码审阅第 35 条）。
    #[must_use]
    pub fn with_unit_area_start(
        device: DeviceIdentity,
        device_bytes: u64,
        unit_area_start: UnitAreaStart,
    ) -> Self {
        let unit_area_slots = unit_area_slots_of_device_starting_at(device_bytes, unit_area_start)
            .expect("建空闲图的盘末尾不在单元区起点之前：mkfs 判过 UnitAreaBeyondDevice，可写挂载读盘之前判过 every_device_reaches_the_unit_area_start");
        let segments = unit_area_slots.div_ceil(CLUSTER_SEGMENT_SLOTS);
        // 起步整个单元区都空：每段的槽对数是这一段的槽数 ÷ 2（末尾那一段可能不满 64 槽、槽数可能是奇数，多出的一槽不成对）。
        let unblocked_slot_pairs_per_segment: Vec<SlotPairCount> = (0..segments)
            .map(|segment_index| {
                let first_slot_of_the_segment = segment_index * CLUSTER_SEGMENT_SLOTS;
                SlotPairCount(
                    (unit_area_slots - first_slot_of_the_segment).min(CLUSTER_SEGMENT_SLOTS) / 2,
                )
            })
            .collect();
        // 起步整个单元区都空：每片叶的数是这片叶与单元区相交那一截的槽数。
        let unit_area_end_slot = unit_area_start.slot().0 + unit_area_slots;
        let unblocked_slots_per_allocation_record_tree_leaf: Vec<SlotCountInALeaf> =
            if unit_area_slots == 0 {
                Vec::new()
            } else {
                (AllocationRecordTreeLeafIndex::of_slot(unit_area_start.slot()).0
                    ..=AllocationRecordTreeLeafIndex::of_slot(SlotNumber(unit_area_end_slot - 1)).0)
                    .map(|leaf_number| {
                        let leaf_index = AllocationRecordTreeLeafIndex(leaf_number);
                        let past_the_leaf =
                            leaf_index.past_the_last_slot().0.min(unit_area_end_slot);
                        let first_slot_of_the_leaf_in_the_unit_area =
                            leaf_index.first_slot().0.max(unit_area_start.slot().0);
                        SlotCountInALeaf(past_the_leaf - first_slot_of_the_leaf_in_the_unit_area)
                    })
                    .collect()
            };
        // 起步整个单元区都空：有槽对的段就是槽对数不是 0 的段（末尾那一段只有一槽时没有）。
        let segments_with_an_unblocked_slot_pair: BTreeSet<usize> =
            unblocked_slot_pairs_per_segment
                .iter()
                .enumerate()
                .filter(|(_, pairs)| pairs.0 > 0)
                .map(|(segment_index, _)| segment_index)
                .collect();
        Self {
            device,
            device_size_in_bytes: device_bytes,
            unit_area_start,
            unit_area_slots,
            allocated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
            used_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            unreleased_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            full_segments_without_an_unreleased_record: unit_area_slots / CLUSTER_SEGMENT_SLOTS,
            free_runs: u64::from(unit_area_slots > 0),
            allocated_slots: 0,
            free_slots: unit_area_slots,
            deferred_slots: 0,
            isolated: vec![false; usize::try_from(unit_area_slots).expect("单元区槽数")],
            isolated_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            isolated_slots: 0,
            held_until_floor_takes_effect: vec![
                false;
                usize::try_from(unit_area_slots)
                    .expect("单元区槽数")
            ],
            held_per_segment: vec![0; usize::try_from(segments).expect("段数")],
            unblocked_slots: unit_area_slots,
            unblocked_slot_pairs: unblocked_slot_pairs_per_segment.iter().copied().sum(),
            // 起步一槽都不罩在未释放记录下：每段的数与没挡的槽对数相同。
            is_under_an_unreleased_record: vec![
                false;
                usize::try_from(unit_area_slots)
                    .expect("单元区槽数")
            ],
            slot_pairs_without_an_unreleased_record: unblocked_slot_pairs_per_segment
                .iter()
                .copied()
                .sum(),
            slot_pairs_without_an_unreleased_record_per_segment: unblocked_slot_pairs_per_segment
                .clone(),
            unblocked_slot_pairs_per_segment,
            segments_with_an_unblocked_slot_pair,
            unblocked_slots_per_allocation_record_tree_leaf,
            segments_reclaimed_not_yet_discarded: BTreeSet::new(),
        }
    }

    /// 槽号 → 位图下标。**槽号在单元区内是这里的前置条件，不是这里判的东西**：
    /// 落点由分配器自己发（`lowest_user_data_slot` / `lowest_empty_segment` / bump 都从这张图的单元区起点起），
    /// 从盘上读来的槽号在进分配器之前都判过：分配记录由 `recovery::allocation_records_fit_the_pool_geometry` 判
    /// （槽号 ≥ 单元区起点、跨度不越过单元区末尾、设备身份在池里、同一块盘上两条不罩同一个槽）；
    /// mkfs 那一版根记录里实例表与树表两条指针的槽号（`mount::format_time_allocator`）按同一道判；
    /// 影子账里被抛弃根的指针槽号（`mount::placements_referenced_by_root`）按 `recovery::placement_lies_in_the_unit_area_of_its_device` 判，
    /// 不在单元区里的那条根当「账读不出」只计数（代码审阅第 32 条）。
    /// 写成 `checked_sub(...).expect(...)` 而不是直接减：`Cargo.toml` 的 `overflow-checks = true` 下直接减也会炸，
    /// 但炸出来的是一句 `attempt to subtract with overflow`，说不出是哪条前置条件没守住。
    fn index(&self, slot: SlotNumber) -> usize {
        let offset_in_slots = slot.0.checked_sub(self.unit_area_start.slot().0).expect(
            "槽号在单元区内：盘上读来的分配记录与 mkfs 那一版两条指针在 recovery::allocation_records_fit_the_pool_geometry 判过，被抛弃根的指针槽号在 recovery::placement_lies_in_the_unit_area_of_its_device 判过，分配器自己发的落点都从单元区起点起",
        );
        usize::try_from(offset_in_slots).expect("单元区内的槽号装得进 usize")
    }

    /// 单元区末尾（不含）的槽号：起点加单元区槽数。
    fn unit_area_end_slot(&self) -> u64 {
        self.unit_area_start.slot().0 + self.unit_area_slots
    }

    /// 位图下标上的这一槽三种位（已分配、影子账隔离、抬 F 扣住）一个都没置。
    fn is_unblocked_at_index(&self, index: usize) -> bool {
        !self.allocated[index]
            && !self.isolated[index]
            && !self.held_until_floor_takes_effect[index]
    }

    /// 位图下标 `index` 所在的那一对（从偶数下标起：单元区起点在段边界上、是偶数槽，下标的奇偶就是槽号的奇偶）两槽都没挡；
    /// 单元区末尾凑不成一对的那一槽答否。
    fn is_unblocked_pair_at_index(&self, index: usize) -> bool {
        let first_index_of_the_pair = index - index % 2;
        first_index_of_the_pair + 1 < self.allocated.len()
            && self.is_unblocked_at_index(first_index_of_the_pair)
            && self.is_unblocked_at_index(first_index_of_the_pair + 1)
    }

    /// 改一槽的一种挡位，同时增量维护没挡的槽数（总数与每片叶一份）、没挡的槽对数（每段一份与总数）与有没挡槽对的段号集合：三张位图只经这里改。
    fn set_blocking_bit(&mut self, index: usize, bit: SlotBlockingBit, is_set: bool) {
        let slot_was_unblocked = self.is_unblocked_at_index(index);
        let pair_was_unblocked = self.is_unblocked_pair_at_index(index);
        match bit {
            SlotBlockingBit::Allocated => self.allocated[index] = is_set,
            SlotBlockingBit::IsolatedByTheShadowLedger => self.isolated[index] = is_set,
            SlotBlockingBit::HeldUntilFloorTakesEffect => {
                self.held_until_floor_takes_effect[index] = is_set;
            }
        }
        let leaf = self.allocation_record_tree_leaf_position_of_index(index);
        match (slot_was_unblocked, self.is_unblocked_at_index(index)) {
            (true, false) => {
                self.unblocked_slots -= 1;
                self.unblocked_slots_per_allocation_record_tree_leaf[leaf].0 -= 1;
            }
            (false, true) => {
                self.unblocked_slots += 1;
                self.unblocked_slots_per_allocation_record_tree_leaf[leaf].0 += 1;
            }
            (true, true) | (false, false) => {}
        }
        let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        match (pair_was_unblocked, self.is_unblocked_pair_at_index(index)) {
            (true, false) => {
                self.unblocked_slot_pairs_per_segment[segment].0 -= 1;
                self.unblocked_slot_pairs.0 -= 1;
                if self.unblocked_slot_pairs_per_segment[segment].0 == 0 {
                    self.segments_with_an_unblocked_slot_pair.remove(&segment);
                }
            }
            (false, true) => {
                self.unblocked_slot_pairs_per_segment[segment].0 += 1;
                self.unblocked_slot_pairs.0 += 1;
                self.segments_with_an_unblocked_slot_pair.insert(segment);
            }
            (true, true) | (false, false) => {}
        }
    }

    /// 位图下标 → 它所在那片分配记录树叶在 `unblocked_slots_per_allocation_record_tree_leaf` 里的下标。
    fn allocation_record_tree_leaf_position_of_index(&self, index: usize) -> usize {
        let slot = SlotNumber(
            self.unit_area_start.slot().0
                + u64::try_from(index)
                    .expect("位图下标小于单元区槽数，单元区槽数本来就是 u64（with_unit_area_start 由它定位图长度）"),
        );
        usize::try_from(
            AllocationRecordTreeLeafIndex::of_slot(slot).0
                - AllocationRecordTreeLeafIndex::of_slot(self.unit_area_start.slot()).0,
        )
        .expect("叶在单元区里的下标不大于位图下标（一片叶至少罩一个槽），位图下标本来就是 usize")
    }

    /// 分配记录树第 `leaf_index` 片叶（罩槽 `[leaf_index × W, (leaf_index + 1) × W)`，W = [`ALLOCATION_RECORD_TREE_LEAF_SLOTS`]）里
    /// 三种位都没置的单元区槽数；叶整片在单元区外是 0。增量维护，读一次 O(1)。
    #[must_use]
    pub fn unblocked_slots_in_allocation_record_tree_leaf(
        &self,
        leaf_index: AllocationRecordTreeLeafIndex,
    ) -> SlotCountInALeaf {
        let first_leaf_index = AllocationRecordTreeLeafIndex::of_slot(self.unit_area_start.slot());
        leaf_index
            .0
            .checked_sub(first_leaf_index.0)
            .and_then(|position| usize::try_from(position).ok())
            .and_then(|position| {
                self.unblocked_slots_per_allocation_record_tree_leaf
                    .get(position)
                    .copied()
            })
            .unwrap_or(SlotCountInALeaf(0))
    }

    /// 单元区罩到的分配记录树叶序号（首尾含）；单元区是空的交回 `None`。
    #[must_use]
    pub fn allocation_record_tree_leaves_of_the_unit_area(
        &self,
    ) -> Option<(AllocationRecordTreeLeafIndex, AllocationRecordTreeLeafIndex)> {
        if self.unit_area_slots == 0 {
            return None;
        }
        Some((
            AllocationRecordTreeLeafIndex::of_slot(self.unit_area_start.slot()),
            AllocationRecordTreeLeafIndex::of_slot(SlotNumber(self.unit_area_end_slot() - 1)),
        ))
    }

    /// 从 `candidate` 起的整个跨度在这块盘的单元区里、每一槽都不被挡（[`Self::is_blocked_for_commit_generated`]）、也不在活整理意图的 `R` 里：
    /// 回落在池里取落点时逐盘认一个落点的判法（槽号可以在这块盘的单元区外，那时答否）。
    #[must_use]
    pub(super) fn is_whole_span_open_for_commit_generated(
        &self,
        candidate: SlotNumber,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
    ) -> bool {
        let is_inside_the_unit_area = candidate.0 >= self.unit_area_start.slot().0
            && candidate.0 + footprint.slots() <= self.unit_area_end_slot();
        is_inside_the_unit_area
            && (candidate.0..candidate.0 + footprint.slots()).all(|slot_of_the_span| {
                let is_outside_the_excluded_region = !excluded_region
                    .is_some_and(|region| region.contains_slot(SlotNumber(slot_of_the_span)));
                is_outside_the_excluded_region
                    && !self.is_blocked_for_commit_generated(SlotNumber(slot_of_the_span))
            })
    }

    /// 三种位都没置的槽数（字段 `unblocked_slots`）：提交内生块落得下的槽，聚簇段内外都算。
    #[must_use]
    pub fn unblocked_slots(&self) -> u64 {
        self.unblocked_slots
    }

    /// 用户数据落得下的槽对数：两槽都没挡、起点偶数、不在 `cluster_segments` 里任何一段（[`Self::lowest_user_data_slot`] 认的那一种；
    /// D3（空间分配） 已定项 8 第 1、2 条，已定项 10 ②）。读时总数减那几段各自的数，O(段数)，不扫单元区
    /// （`.claude/rules/fs-design.md`「记账是事务的副产品」：准入是运行时决策路径，代价不许随盘容量增长）。
    ///
    /// # Panics
    /// `cluster_segments` 里有一段的起点不在这块盘单元区里的段边界上：那几段都是各盘一致答出来的全空段起点
    /// （[`Self::lowest_empty_segment`]），每块盘上都是整段、在单元区里。
    #[must_use]
    pub fn unblocked_slot_pairs_outside_the_cluster_segments(
        &self,
        cluster_segments: &BTreeSet<SlotNumber>,
    ) -> SlotPairCount {
        SlotPairCount(
            self.unblocked_slot_pairs.0
                - pairs_inside_the_cluster_segments(
                    self.unit_area_start,
                    &self.unblocked_slot_pairs_per_segment,
                    cluster_segments,
                )
                .0,
        )
    }

    /// defer、影子账隔离、抬 F 扣住三样全放开之后用户数据落得下的槽对数：两槽都不罩在未释放分配记录下、起点偶数、不在 `cluster_segments` 里任何一段
    /// （抬 F 之前先算槽对那一判的上界，D16（发布语义） 已定项 10 射程「抬 F 之前先算」）。不问这一串的上限够不够得到 defer 里的那几槽、
    /// 也不问推串时聚簇段会不会多开：只会比真放开之后成的对数多，不会少。读法同 [`Self::unblocked_slot_pairs_outside_the_cluster_segments`]，
    /// O(段数)、不扫单元区（准入是运行时决策路径）。
    ///
    /// # Panics
    /// 同 [`Self::unblocked_slot_pairs_outside_the_cluster_segments`]。
    #[must_use]
    pub fn slot_pairs_without_an_unreleased_record_outside_the_cluster_segments(
        &self,
        cluster_segments: &BTreeSet<SlotNumber>,
    ) -> SlotPairCount {
        SlotPairCount(
            self.slot_pairs_without_an_unreleased_record.0
                - pairs_inside_the_cluster_segments(
                    self.unit_area_start,
                    &self.slot_pairs_without_an_unreleased_record_per_segment,
                    cluster_segments,
                )
                .0,
        )
    }

    #[must_use]
    pub fn is_free(&self, slot: SlotNumber) -> bool {
        slot.0 >= self.unit_area_start.slot().0
            && slot.0 < self.unit_area_end_slot()
            && !self.allocated[self.index(slot)]
            && !self.isolated[self.index(slot)]
            && !self.held_until_floor_takes_effect[self.index(slot)]
    }

    #[must_use]
    pub fn unit_area_start(&self) -> UnitAreaStart {
        self.unit_area_start
    }
    #[must_use]
    pub fn unit_area_slots(&self) -> u64 {
        self.unit_area_slots
    }
    #[must_use]
    pub fn device_size_in_bytes(&self) -> u64 {
        self.device_size_in_bytes
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

    /// 位图下标 `[start, end)` 的槽进出未释放分配记录：逐槽改 `unreleased_per_segment`，一段的数在 0 与非 0 之间翻过去、
    /// 那一段又是凑满 64 槽的整段时，记录口径的全空整段数跟着减一或加一（[`Self::empty_segments`]）。
    fn note_slots_under_an_unreleased_record(
        &mut self,
        start: usize,
        end: usize,
        transition: UnreleasedRecordTransition,
    ) {
        let slots_per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        for index in start..end {
            let segment = index / slots_per_segment;
            let is_a_full_segment = (segment + 1) * slots_per_segment <= self.allocated.len();
            self.note_one_slot_under_an_unreleased_record_in_the_slot_pair_counts(
                index, segment, transition,
            );
            match transition {
                UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord => {
                    if self.unreleased_per_segment[segment] == 0 && is_a_full_segment {
                        self.full_segments_without_an_unreleased_record -= 1;
                    }
                    self.unreleased_per_segment[segment] += 1;
                }
                UnreleasedRecordTransition::SlotsLeaveTheirUnreleasedRecord => {
                    self.unreleased_per_segment[segment] = self.unreleased_per_segment[segment]
                        .checked_sub(1)
                        .expect("释放的槽此前罩在未释放的记录下：mark_released 先断言过这些槽分配位置着，分配与复活都给它们加过");
                    if self.unreleased_per_segment[segment] == 0 && is_a_full_segment {
                        self.full_segments_without_an_unreleased_record += 1;
                    }
                }
            }
        }
    }

    /// 位图下标 `index` 所在的那一对（从偶数下标起）两槽都不罩在未释放分配记录下；单元区末尾凑不成一对的那一槽答否。
    fn is_pair_without_an_unreleased_record_at_index(&self, index: usize) -> bool {
        let first_index_of_the_pair = index - index % 2;
        first_index_of_the_pair + 1 < self.is_under_an_unreleased_record.len()
            && !self.is_under_an_unreleased_record[first_index_of_the_pair]
            && !self.is_under_an_unreleased_record[first_index_of_the_pair + 1]
    }

    /// 一槽进出未释放分配记录：改它那一位，它所在那一对在「两槽都不罩在未释放记录下」上翻过去时，那一段的数与总数跟着加一或减一。
    fn note_one_slot_under_an_unreleased_record_in_the_slot_pair_counts(
        &mut self,
        index: usize,
        segment: usize,
        transition: UnreleasedRecordTransition,
    ) {
        let pair_was_without_an_unreleased_record =
            self.is_pair_without_an_unreleased_record_at_index(index);
        self.is_under_an_unreleased_record[index] = match transition {
            UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord => true,
            UnreleasedRecordTransition::SlotsLeaveTheirUnreleasedRecord => false,
        };
        match (
            pair_was_without_an_unreleased_record,
            self.is_pair_without_an_unreleased_record_at_index(index),
        ) {
            (true, false) => {
                self.slot_pairs_without_an_unreleased_record_per_segment[segment].0 -= 1;
                self.slot_pairs_without_an_unreleased_record.0 -= 1;
            }
            (false, true) => {
                self.slot_pairs_without_an_unreleased_record_per_segment[segment].0 += 1;
                self.slot_pairs_without_an_unreleased_record.0 += 1;
            }
            (true, true) | (false, false) => {}
        }
    }

    /// 把一个仍分配的落点放进 defer 队列：槽仍占着（分配器不许再发它），只是记账上从「仍分配」挪到「待释放」；
    /// 记录改写成已释放，按记录口径它的槽从此不罩在未释放的记录下（[`Self::empty_segments`]）。
    pub fn mark_released(&mut self, slot: SlotNumber, span: u64) {
        let start = self.index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            self.allocated[start..end].iter().all(|taken| *taken),
            "释放的跨度里有没分配的槽"
        );
        self.note_slots_under_an_unreleased_record(
            start,
            end,
            UnreleasedRecordTransition::SlotsLeaveTheirUnreleasedRecord,
        );
        self.deferred_slots += span;
    }
    /// 管理员回退复活一个还在 defer 队列里的落点（`PoolAllocator::resurrect_released_record`）：槽本来就占着，
    /// 记账上从「待释放」挪回「仍分配」——[`Self::mark_released`] 反过来。
    pub fn mark_resurrected_out_of_the_defer_queue(&mut self, slot: SlotNumber, span: u64) {
        let start = self.index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            self.allocated[start..end].iter().all(|taken| *taken),
            "复活的跨度里有没分配的槽：还在 defer 里的落点位图上恒占着"
        );
        self.deferred_slots = self
            .deferred_slots
            .checked_sub(span)
            .expect("复活的落点在 defer 里，defer 槽数不小于它的跨度");
        self.note_slots_under_an_unreleased_record(
            start,
            end,
            UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord,
        );
    }

    /// 隔离一个只被被抛弃根引用的落点：不进已分配、不进 defer、不动空闲计数，只让分配器绕开它（用户数据落点与开放段都不落在它上面）。
    /// 隔离位独立于分配位：一个槽在当前账里已分配、后来被释放、再被回收，隔离位照样让它发不出去——抬 F 之后候选集缩小，
    /// 一个先前靠候选根豁免的槽会在回收之前被补隔离（`mount::raise_rollback_floor`）。同一个槽隔离两次不重复计数。
    pub fn isolate(&mut self, slot: SlotNumber, span: u64) {
        let start = self.index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            end <= self.allocated.len(),
            "跨度越过单元区末尾：隔离的落点来自被抛弃根，它的槽与跨度在 recovery::placement_lies_in_the_unit_area_of_its_device / allocation_records_fit_the_pool_geometry 判过"
        );
        for index in start..end {
            self.isolate_slot_at_index(index);
        }
    }

    /// 同 [`Self::isolate`]，只隔离落点里分配位没置着的槽：分配位置着的（现行账里仍分配的，或已释放、还在 defer 窗口里的）算在
    /// 「已分配」里，D28（挂载期承诺量） 已定项 1 第九项「不在这里再算一次」；它们回收的那一刻由分配器照还没解开的那一份补隔离
    /// （`PoolAllocator::record_root_written_by_this_process`、`PoolAllocator::reclaim_released_up_to_isolating_what_the_isolation_still_protects`）。
    /// 给水位之上没施加的记录点名的落点（C694 的 C 那一圈）用。
    pub fn isolate_the_slots_not_allocated(&mut self, slot: SlotNumber, span: u64) {
        let start = self.index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            end <= self.allocated.len(),
            "跨度越过单元区末尾：C 那一圈的落点是盘上读来的记录点名的与账里的（mount::placements_named_by_unapplied_journal_records、placements_referenced_by_root 逐条经 recovery::placement_lies_in_the_unit_area_of_its_device 判过落在这块盘的单元区里），或现行账里的记录"
        );
        for index in start..end {
            if !self.allocated[index] {
                self.isolate_slot_at_index(index);
            }
        }
    }

    /// 置一槽的隔离位并维护两个隔离计数；已置着就什么都不动（同一个槽隔离两次不重复计数）。
    fn isolate_slot_at_index(&mut self, index: usize) {
        if !self.isolated[index] {
            self.set_blocking_bit(index, SlotBlockingBit::IsolatedByTheShadowLedger, true);
            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
            self.isolated_per_segment[segment] += 1;
            self.isolated_slots += 1;
        }
    }

    /// 分配位与隔离位同时置着的槽数：D28（挂载期承诺量） 已定项 1 第九项与「已分配」各扣一次的那几个。逐槽扫一遍单元区，
    /// 只给核对与用例读，运行时的判定不读它（`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」）。
    #[must_use]
    pub fn slots_counted_both_as_allocated_and_as_isolated(&self) -> u64 {
        u64::try_from(
            self.allocated
                .iter()
                .zip(&self.isolated)
                .filter(|(is_allocated, is_isolated)| **is_allocated && **is_isolated)
                .count(),
        )
        .expect("单元区槽数装得进 u64")
    }

    /// 这块盘上隔离的槽数（D28（挂载期承诺量） 已定项 1 第九项的这块盘那一项）：只被被抛弃根引用的，加水位之上没施加的 journal 记录点名、
    /// 还没解开的（C694 的 C）；两部分置的是同一个隔离位，同一个槽只数一次。
    #[must_use]
    pub fn isolated_slots(&self) -> u64 {
        self.isolated_slots
    }

    /// 这一槽的影子账隔离位（[`Self::isolate`] 那一套）置着；单元区之外的槽交 `false`。只给会话数「C 那一圈这一刻隔离着几个槽」读
    /// （`PoolAllocator::isolated_slots_covered_by`，推满仍不够时报不报「被宽护卡住」），分配与准入的判定不读它。
    #[must_use]
    pub fn is_isolated(&self, slot: SlotNumber) -> bool {
        let lies_in_the_unit_area =
            (self.unit_area_start.slot().0..self.unit_area_end_slot()).contains(&slot.0);
        lies_in_the_unit_area && self.isolated[self.index(slot)]
    }

    /// 清掉一个槽的影子账隔离位（`isolate` 置的那一套，D28（挂载期承诺量） 已定项 1 第九项「被抛弃的根被轮转覆写时清零」）：
    /// 只动这一位与它的两个计数，分配位与抬 F 扣住那一位照旧——清完之后它发不发得出去看别的位。这一位没置着就什么都不动。
    pub fn clear_isolation_of_slot(&mut self, slot: SlotNumber) {
        let index = self.index(slot);
        if self.isolated[index] {
            self.set_blocking_bit(index, SlotBlockingBit::IsolatedByTheShadowLedger, false);
            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
            self.isolated_per_segment[segment] -= 1;
            self.isolated_slots -= 1;
        }
    }

    /// 抬 F 回收的落点先扣住：空闲计数照加，位图照清，但 `is_free` 与开段都绕开它，直到 `release_holds`。
    pub fn hold_until_floor_takes_effect(&mut self, slot: SlotNumber, span: u64) {
        let start = self.index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(end <= self.allocated.len(), "跨度越过单元区末尾");
        for index in start..end {
            if !self.held_until_floor_takes_effect[index] {
                self.set_blocking_bit(index, SlotBlockingBit::HeldUntilFloorTakesEffect, true);
                let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
                self.held_per_segment[segment] += 1;
            }
        }
    }

    /// 提交内生块的 bump 游标要绕开的槽：已分配、影子账隔离、抬 F 扣住三种位任一为真
    /// （开段那一刻的条件担保不了开段之后才置的隔离位与扣住位）。
    /// 释放之前核出校验和对不上的那一份不另占一种位：它那块盘上的分配记录留在已分配（D19（块指针的结构与宽度预算） 已定项 5），
    /// 落在「已分配」那一位上。
    #[must_use]
    pub fn is_blocked_for_commit_generated(&self, slot: SlotNumber) -> bool {
        let index = self.index(slot);
        self.allocated[index] || self.isolated[index] || self.held_until_floor_takes_effect[index]
    }

    /// F 在每块盘上生效之后把扣住的槽放开。只走有扣住的那几段（`held_per_segment` 非 0），逐槽经 [`Self::set_blocking_bit`] 清，
    /// 没挡的槽数与槽对数跟着维护。
    pub fn release_holds(&mut self) {
        let segment_slots = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        // 迭代上界是段数；每一段里至多 64 槽。
        for segment in 0..self.held_per_segment.len() {
            if self.held_per_segment[segment] == 0 {
                continue;
            }
            let first_index = segment * segment_slots;
            let past_the_last_index = (first_index + segment_slots).min(self.allocated.len());
            for index in first_index..past_the_last_index {
                if self.held_until_floor_takes_effect[index] {
                    self.set_blocking_bit(index, SlotBlockingBit::HeldUntilFloorTakesEffect, false);
                }
            }
            self.held_per_segment[segment] = 0;
        }
    }

    /// 回收：一个已释放、释放代 ≤ max(F_生效, 环里最旧有效根) 的落点回到空闲（D16（发布语义） 已定项 1 的可再分配谓词）：
    /// 位图清掉、占着的槽数与 defer 队列各减、空闲加——记账的空闲字节到这一刻才动（里程碑「覆盖写、释放、回退与复用」步 5）。
    pub fn mark_reclaimed(&mut self, slot: SlotNumber, span: u64) {
        let start = self.index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            self.allocated[start..end].iter().all(|taken| *taken),
            "回收的跨度里有没分配的槽"
        );
        let left_free = start > 0 && !self.allocated[start - 1];
        let right_free = end < self.allocated.len() && !self.allocated[end];
        // runs 的增量：两边都空是把两段并成一段（−1），两边都占是新开一段（+1），一边空是接上去（不变）。
        self.free_runs = self.free_runs + 1 - u64::from(left_free) - u64::from(right_free);
        for index in start..end {
            self.set_blocking_bit(index, SlotBlockingBit::Allocated, false);
            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
            self.used_per_segment[segment] -= 1;
            self.segments_reclaimed_not_yet_discarded.insert(segment);
        }
        self.allocated_slots -= span;
        self.deferred_slots -= span;
        self.free_slots += span;
    }

    /// 段号 → 段起点的槽号。
    fn segment_start_slot(&self, segment_index: usize) -> SlotNumber {
        SlotNumber(
            self.unit_area_start.slot().0
                + u64::try_from(segment_index).expect("段号") * CLUSTER_SEGMENT_SLOTS,
        )
    }

    /// 这一段是不是整段都可再分配：没有已分配（仍分配与 defer 窗口里的都算占着）、没有影子账隔离、
    /// 没有抬 F 扣住（扣住 = F 还没在每块盘上生效）。单元区末尾凑不满 64 槽的那一截不是整段。
    #[must_use]
    pub fn segment_is_reusable_as_a_whole(&self, segment_index: usize) -> bool {
        let per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        let is_a_full_segment = (segment_index + 1) * per_segment <= self.allocated.len();
        is_a_full_segment
            && self.used_per_segment[segment_index] == 0
            && self.isolated_per_segment[segment_index] == 0
            && self.held_per_segment[segment_index] == 0
    }

    /// 发布返回之后要发 discard 的段起点：回收过槽、还没发过的段里，
    /// 整段可再分配的交出去并摘掉；段里又有槽被分配了的也摘掉（之后再回收会重新记上）；只差扣住或隔离的留着，等它们放开。
    /// 只看回收时记下的段，不扫单元区（`.claude/rules/fs-design.md`「记账是事务的副产品」：discard 不在决策路径上，也不为它遍历）。
    pub fn take_the_segments_to_discard(&mut self) -> Vec<SlotNumber> {
        let noted: Vec<usize> = self
            .segments_reclaimed_not_yet_discarded
            .iter()
            .copied()
            .collect();
        let mut segments_to_discard = Vec::new();
        for segment_index in noted {
            if self.segment_is_reusable_as_a_whole(segment_index) {
                segments_to_discard.push(self.segment_start_slot(segment_index));
                self.segments_reclaimed_not_yet_discarded
                    .remove(&segment_index);
            } else if self.used_per_segment[segment_index] > 0 {
                self.segments_reclaimed_not_yet_discarded
                    .remove(&segment_index);
            }
        }
        segments_to_discard
    }

    #[must_use]
    pub fn free_runs(&self) -> u64 {
        self.free_runs
    }
    /// 全空聚簇段数（记账第 11 项）：单元区里凑满 64 槽的整段中、段内 64 槽都没有未释放分配记录的段数（D3（空间分配） 已定项 10 ①、
    /// D26（后台整理与放置回收） 已定项 1 第 1 条收口的记录口径，C372（全空聚簇段数两份定义相反））。defer 窗口里的已释放槽不算占用：
    /// 一段里只剩 defer 槽时这里算它全空，[`Self::segment_is_empty`] 与开段（[`Self::lowest_empty_segment`]）照旧算它占着。
    /// 隔离位与抬 F 扣住位不看（「可开聚簇段」是另一个量，C379）。增量维护，读一次 O(1)。
    #[must_use]
    pub fn empty_segments(&self) -> u64 {
        self.full_segments_without_an_unreleased_record
    }

    /// 段内是不是整段空闲。
    #[must_use]
    pub fn segment_is_empty(&self, segment_index: usize) -> bool {
        self.used_per_segment[segment_index] == 0
    }

    /// 把 `[slot, slot + span)` 标成已分配，并增量更新 runs 与段计数。要求整个跨度事先都空闲。
    ///
    /// 两条断言都是**前置条件，不是这里判盘上内容的地方**：从盘上读来的分配记录在进分配器之前由
    /// `recovery::allocation_records_fit_the_pool_geometry` 判过跨度在单元区内、同一块盘上两条记录不罩同一个槽
    /// （I-5.4（分配记录罩住的槽互不相交） 的读路径形态）；分配器自己发的落点由落点政策保证两槽都空。
    pub fn mark_allocated(&mut self, slot: SlotNumber, span: u64) {
        let start = self.index(slot);
        let end = start + usize::try_from(span).expect("跨度");
        assert!(
            end <= self.allocated.len(),
            "跨度越过单元区末尾：盘上读来的记录在 recovery::allocation_records_fit_the_pool_geometry 判过跨度"
        );
        assert!(
            self.allocated[start..end].iter().all(|taken| !*taken),
            "跨度里有已分配的槽：盘上读来的记录在 recovery::allocation_records_fit_the_pool_geometry 判过 I-5.4（同一块盘上的记录罩住的槽互不相交）"
        );
        // runs 的增量：被切的那一段空闲 run 左右各剩不剩东西。
        let left_free = start > 0 && !self.allocated[start - 1];
        let right_free = end < self.allocated.len() && !self.allocated[end];
        let pieces_left = u64::from(left_free) + u64::from(right_free);
        self.free_runs = self.free_runs + pieces_left - 1;
        for index in start..end {
            self.set_blocking_bit(index, SlotBlockingBit::Allocated, true);
            let segment = index / usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
            self.used_per_segment[segment] += 1;
        }
        self.note_slots_under_an_unreleased_record(
            start,
            end,
            UnreleasedRecordTransition::SlotsComeUnderAnUnreleasedRecord,
        );
        self.allocated_slots += span;
        self.free_slots -= span;
    }

    /// 用户数据落点：起点槽号最小、偶数槽（32768 对齐）、两槽都空、不在任何一个聚簇段里（D3（空间分配） 已定项 10 ②
    /// 逐字「候选落点不在任何开放的聚簇段里」，量词是「任何」不是「当前那一个」）。
    #[must_use]
    pub fn lowest_user_data_slot(
        &self,
        cluster_segments: &BTreeSet<SlotNumber>,
    ) -> Option<SlotNumber> {
        self.lowest_user_data_slot_outside_the_region(cluster_segments, None)
    }

    /// 同 [`Self::lowest_user_data_slot`]，另加一条合取「候选落点不在活整理意图的 `R` 里」（D3（空间分配） 已定项 7 登记表第 5 行、
    /// D26（后台整理与放置回收） 已定项 1 第 3 条）。`excluded_region` 是 `None` 时与它相同。
    ///
    /// 取法：按段号升序走有没挡槽对的段（`segments_with_an_unblocked_slot_pair`，随三种挡位增量维护），跳过起点在 `cluster_segments` 里的段
    /// 与起点在 `R` 里的段，交回第一个没跳过的段里最低的那一对。与从单元区起点按 2 槽逐个看的取法答案相同，依据：一对不跨段
    /// （单元区起点在段边界上、段长是偶数）；聚簇段的起点是开段时各盘一致答出来的全空段起点（[`Self::lowest_empty_segment`]），`R` 的起点按类型不变量是
    /// 64 的倍数，单元区起点也是 ⇒ 三者都是同一套 64 槽格子上的整段，一个候选在不在聚簇段或 `R` 里等于它所在那一段的起点在不在。
    /// 代价：集合里每走过一段，那一段要么在 `cluster_segments` 里、要么是 `R` 那一段、要么就是答案所在的段 ⇒ 至多走过
    /// 「这次挂载开过的聚簇段数 + 2」段（每段一次集合点查），再在答案那一段里看至多 32 对；不随单元区的槽数、也不随已占的槽数长。
    #[must_use]
    pub fn lowest_user_data_slot_outside_the_region(
        &self,
        cluster_segments: &BTreeSet<SlotNumber>,
        excluded_region: Option<CompactionRegion>,
    ) -> Option<SlotNumber> {
        self.segments_with_an_unblocked_slot_pair
            .iter()
            .find_map(|segment_index| {
                let segment_start = self.segment_start_slot(*segment_index);
                let is_open_to_user_data = !cluster_segments.contains(&segment_start)
                    && !excluded_region.is_some_and(|region| region.contains_slot(segment_start));
                if is_open_to_user_data {
                    self.lowest_unblocked_slot_pair_in_the_segment(*segment_index)
                } else {
                    None
                }
            })
    }

    /// 第 `segment_index` 段里起点最低、两槽都没挡的那一对（[`Self::is_unblocked_pair_at_index`]，从段首按 2 槽步进；单元区末尾不成对的那一槽不算）；
    /// 没有就 `None`。至多看 32 对。
    fn lowest_unblocked_slot_pair_in_the_segment(
        &self,
        segment_index: usize,
    ) -> Option<SlotNumber> {
        let slots_per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect(
            "CLUSTER_SEGMENT_SLOTS 是 singlefs-format 定的格式常量 64，任何平台的 usize 都装得下",
        );
        let first_index_of_the_segment = segment_index * slots_per_segment;
        let past_the_segment =
            (first_index_of_the_segment + slots_per_segment).min(self.allocated.len());
        (first_index_of_the_segment..past_the_segment)
            .step_by(2)
            .find(|first_index_of_the_pair| {
                self.is_unblocked_pair_at_index(*first_index_of_the_pair)
            })
            .map(|first_index_of_the_pair| {
                SlotNumber(
                    self.unit_area_start.slot().0
                        + u64::try_from(first_index_of_the_pair)
                            .expect("位图下标小于单元区槽数，单元区槽数本来就是 u64"),
                )
            })
    }

    /// 用户数据落点的参照取法，只供测试与 [`Self::lowest_user_data_slot_outside_the_region`] 对拍：从单元区起点按 2 槽逐个候选看，
    /// 每个候选逐段问在不在聚簇段里、问在不在 `R` 里、问两槽空不空，第一个都过的就是答案。
    #[cfg(test)]
    fn lowest_user_data_slot_outside_the_region_by_stepping_through_every_candidate(
        &self,
        cluster_segments: &BTreeSet<SlotNumber>,
        excluded_region: Option<CompactionRegion>,
    ) -> Option<SlotNumber> {
        // 起点在聚簇段边界上（`UnitAreaStart` 的不变量）⇒ 是偶数槽，按 2 槽步进的候选都 32768 对齐。
        let mut candidate = self.unit_area_start.slot().0;
        let end = self.unit_area_end_slot();
        while candidate + 1 < end {
            let inside_cluster_segment = cluster_segments.iter().any(|segment| {
                candidate >= segment.0 && candidate < segment.0 + CLUSTER_SEGMENT_SLOTS
            });
            let inside_the_excluded_region =
                excluded_region.is_some_and(|region| region.contains_slot(SlotNumber(candidate)));
            if !inside_cluster_segment
                && !inside_the_excluded_region
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
        self.lowest_empty_segment_outside_the_region(None)
    }

    /// 同 [`Self::lowest_empty_segment`]，活整理意图的 `R` 那一段不开（D26（后台整理与放置回收） 已定项 1 第 3 条）。
    #[must_use]
    pub fn lowest_empty_segment_outside_the_region(
        &self,
        excluded_region: Option<CompactionRegion>,
    ) -> Option<SlotNumber> {
        let per_segment = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
        let full_segments = self.allocated.len() / per_segment;
        (0..full_segments)
            .find(|segment| {
                self.used_per_segment[*segment] == 0
                    && self.isolated_per_segment[*segment] == 0
                    && self.held_per_segment[*segment] == 0
                    && !excluded_region.is_some_and(|region| {
                        region.contains_slot(self.segment_start_slot(*segment))
                    })
            })
            .map(|segment| {
                SlotNumber(
                    self.unit_area_start.slot().0
                        + u64::try_from(segment).expect("段号") * CLUSTER_SEGMENT_SLOTS,
                )
            })
    }

    /// 这块盘按自己的空闲图答的回落落点（各盘只按自己的空闲图答时 [`CommitGeneratedDeviceAnswer::FallBack`] 带的那个槽；池里真取的回落落点见
    /// [`PoolAllocator::try_allocate_commit_generated_preferring_the_rewrite_set`]）：这块盘单元区里起点槽号最小、整个跨度都不被挡的落点——
    /// 挡的是 bump 游标绕开的同一套位（已分配、影子账隔离、抬 F 扣住），回落不是绕开影子账或扣住的第二条路；两槽的码 3 容器照数据单元那一档
    /// 起点 32768 对齐（D3（空间分配） 已定项 10 ⑤）。第一版 `R` 为空。
    #[must_use]
    pub fn lowest_commit_generated_fallback_slot(
        &self,
        footprint: UnitFootprint,
    ) -> Option<SlotNumber> {
        self.lowest_commit_generated_fallback_slot_outside_the_region(footprint, None)
    }

    /// 同 [`Self::lowest_commit_generated_fallback_slot`]，排除活整理意图的 `R`（D3（空间分配） 已定项 8 ②「同样排除 `R`」）。
    #[must_use]
    pub fn lowest_commit_generated_fallback_slot_outside_the_region(
        &self,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
    ) -> Option<SlotNumber> {
        let unit_area_end = self.unit_area_end_slot();
        let mut candidate = self.unit_area_start.slot().0;
        while candidate + footprint.slots() <= unit_area_end {
            let is_whole_span_unblocked = (candidate..candidate + footprint.slots()).all(|slot| {
                !self.is_blocked_for_commit_generated(SlotNumber(slot))
                    && !excluded_region.is_some_and(|region| region.contains_slot(SlotNumber(slot)))
            });
            if is_whole_span_unblocked {
                return Some(SlotNumber(candidate));
            }
            candidate += footprint.alignment_in_slots();
        }
        None
    }

    /// 开放段没有或装不下时这块盘给提交内生块的去处：最低的全空段；没有全空段就回落；连回落的空槽都没有就 `None`。
    #[must_use]
    pub fn commit_generated_answer_without_open_segment(
        &self,
        footprint: UnitFootprint,
    ) -> Option<CommitGeneratedDeviceAnswer> {
        self.commit_generated_answer_without_open_segment_outside_the_region(footprint, None)
    }

    /// 同 [`Self::commit_generated_answer_without_open_segment`]，开段与回落都排除活整理意图的 `R`。
    #[must_use]
    pub fn commit_generated_answer_without_open_segment_outside_the_region(
        &self,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
    ) -> Option<CommitGeneratedDeviceAnswer> {
        match self.lowest_empty_segment_outside_the_region(excluded_region) {
            Some(segment_start) => {
                Some(CommitGeneratedDeviceAnswer::OpenEmptySegment(segment_start))
            }
            None => self
                .lowest_commit_generated_fallback_slot_outside_the_region(
                    footprint,
                    excluded_region,
                )
                .map(CommitGeneratedDeviceAnswer::FallBack),
        }
    }
}

/// 回收的槽什么时候能再发出去：重建分配器时 F 已经生效，立刻；抬 F 时要等带新 F 的根落满每块盘。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReclaimedReuse {
    Immediately,
    HeldUntilFloorTakesEffect,
}

/// 复用窗口：一个落点从「释放」到「可再分配」之间隔多久。只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条：
/// 每条分支必须能被测试强制进入），产品路径恒 `GatedByTheRollbackFloor`。
///
/// 正常触发「释放的槽被再分配」要先抬回退下界 F 到释放代之上、再等带新 F 的根落满每块盘（`PoolAllocator::reclaim_released_up_to`
/// 与 `ReclaimedReuse::HeldUntilFloorTakesEffect`），一条固定脚本上要好几次空发布才进得去；`ForcedToZero` 让回收判定
/// 无视 F、也无视延迟窗口：一个落点在 `PoolAllocator::release` 返回时就已经回到空闲，下一次发布走正常的分配路径就把它再发出去。
/// 它造出来的正是 C22（刚释放的块立即重分配） 要的那一格：被复用的槽仍被一条候选根引用着。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReuseWindow {
    /// 产品路径：可再分配 ⟺ 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)（D16（发布语义） 已定项 1）。
    GatedByTheRollbackFloor,
    /// 只供测试：释放那一刻就回收、当场可再分配，F 与延迟窗口都不看。
    ForcedToZero,
}

impl ReuseWindow {
    /// 运行时报出走的是哪一条（`.claude/rules/fs-design.md` 五条硬要求第 4 条：分支必须可观测）。
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            ReuseWindow::GatedByTheRollbackFloor => "reuse_window_gated_by_the_rollback_floor",
            ReuseWindow::ForcedToZero => "reuse_window_forced_to_zero",
        }
    }
}

/// 一次发布里分配的一个落点，两盘同槽（D2（RAID 条带策略） 已定项 10：一个单元整个落在一列上、两盘各一份）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub slot: SlotNumber,
    pub span: u64,
}

/// 一块盘上、起点就是这次落点的那个槽的记录，让位之后是什么样子。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecordAtThePlacementSlot {
    /// 那里原来是一条已回收的记录，已经改写成这次的分配：不再追加。
    RewrittenFromReclaimed,
    /// 那里没有记录：由调用方追加一条。
    Absent,
}

/// 一块盘上，与新落点 [槽, 槽 + 跨度) 相交的已有记录给这次分配让位（I-5.4（分配记录罩住的槽互不相交））：
/// 起点就是这个槽的已回收记录改写成这次的分配（字节表五「已释放记录被重用之后再改写一次」），起点不是这个槽的已回收记录
/// 随这次分配一起删掉——留着它，下一次可写挂载重建分配器时两条记录罩住同一个槽，重建在 `mark_allocated` 的断言上 panic、
/// 池从此挂不上可写（两槽的数据单元复用一条跨 1 的已回收记录，下一槽另一条跨 1 的已回收记录留着）。
/// 删掉的记录罩住的槽已经回到空闲（`mark_reclaimed` 清了位图、减了计数），删它不动位图与计数；起点不在这次落点里的那一截槽
/// 删完之后就没有记录，与从没分配过的槽同样（D3（空间分配） 已定项 7：条目留到该落点被重新分配时覆盖，而它罩住的槽正被这次分配重新占用）。
///
/// # Panics
/// 相交的记录里有不是已回收的（仍分配的，或已释放还在 defer 窗口里的）：分配器只把位图上空闲的槽发出来，而那样的记录罩住的槽
/// 在位图上仍占着——走到这里说明位图与记录分叉了。
pub(super) fn make_room_for_record_on_device(
    records: &mut Vec<AllocationRecord>,
    reclaimed: &mut BTreeSet<(DeviceIdentity, SlotNumber)>,
    device: DeviceIdentity,
    placement: Placement,
    generation: CheckpointTxg,
) -> RecordAtThePlacementSlot {
    let placement_end = placement.slot.0 + placement.span;
    let intersecting_record_slots: Vec<SlotNumber> = records
        .iter()
        .filter(|record| {
            record.device == device
                && record.slot.0 < placement_end
                && placement.slot.0 < record.slot.0 + u64::from(record.span_slots)
        })
        .map(|record| record.slot)
        .collect();
    let placement_slot_was_reclaimed = reclaimed.contains(&(device, placement.slot));
    let mut record_at_the_placement_slot = RecordAtThePlacementSlot::Absent;
    for record_slot in intersecting_record_slots {
        let was_reclaimed = reclaimed.remove(&(device, record_slot));
        assert!(
            was_reclaimed,
            "盘 {device:?} 上与新落点（槽 {:?} 跨 {}）相交的记录（槽 {record_slot:?}）不是已回收的：分配器只发位图上空闲的槽，位图与记录分叉了",
            placement.slot,
            placement.span
        );
        if record_slot == placement.slot {
            let existing = records
                .iter_mut()
                .find(|record| record.device == device && record.slot == record_slot)
                .expect("上面刚从记录里找出这个槽");
            assert!(existing.is_released, "回收过的落点的记录该是已释放");
            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");
            existing.generation = generation;
            existing.is_released = false;
            record_at_the_placement_slot = RecordAtThePlacementSlot::RewrittenFromReclaimed;
        } else {
            records.retain(|record| !(record.device == device && record.slot == record_slot));
        }
    }
    assert!(
        !placement_slot_was_reclaimed
            || record_at_the_placement_slot == RecordAtThePlacementSlot::RewrittenFromReclaimed,
        "回收过的落点有它的已释放记录"
    );
    record_at_the_placement_slot
}

#[cfg(test)]
mod tests;

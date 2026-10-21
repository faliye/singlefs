//! 整理的选段与计划的 Verus 核与证明（里程碑四二·4；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7 的 V-1：
//! 放进这个 crate 的一个模块）。`singlefs-core` 的 `compaction_plan::choose_region` 与 `compaction_plan::plan_batch` 去掉读记录、
//! 读扫描结果之后剩下的判定写成两个可执行的核（[`choose_compaction_segment`]、[`plan_compaction_batch`]），证判决 G7 点名的四条：
//! - P1 选段健全（先不证最优）：选出的段合 G2 的资格——不是这次挂载开过的段、不是活意图的 R、没装钉住的槽、没被区域扫描拒过、
//!   扫过而且至少一个能搬的单元；「一段都没资格」时真没有一段是候选。
//! - P2 计划健全：一批至多 K 项、每一项是能搬的单元、各项互异；扫描结果里能搬的单元映射 key 互异时，一批里的 key 互异。
//!   目的地那一半（不在 R 内、两两不重叠、跨度相同）按判决 G2 是分配器的性质，计划函数不交目的地，这里只证 key 一侧。
//! - P3 进展：交「搬」时至少一项；不搬时分「搬空」（R 里一条未释放记录都没有）与「搬不动」（有记录、一个能搬的都没有）。
//! - P4 抽象恒等引理：映射（key → 落点）加落点上的字节是具体状态，每个 key 经映射读到的字节是抽象状态。按计划搬一批（源字节原样写到
//!   目的地、映射条目换到目的地）不改任何一个 key 读到的字节；在前提「R 内每条未释放记录都有映射 key 且不在隔离集里」（攻方 H-G7-1）下，
//!   一批批搬到 R 空也不改。
//!
//! 核心层不调这两个核（`crates/mutations.tsv` 里有十二行变异锚在 `compaction_plan` 现在的写法上，换成调核那几行原文就没了）：
//! `singlefs-core` 的 `compaction_plan` 单测在小输入上把它的 `choose_region`、`plan_batch` 与这两个核逐项对拍。
//! 执行层（发布路径是不是把字节原样写到目的地、写序、崩溃）不在 Verus 里，靠 N6、模型对拍与必出反例的世界（判决 G7）。
//! 必红世界与 `src/lib.rs` 的同登记在 `verus-must-red-worlds.tsv`。

use vstd::pervasive::unreached;
use vstd::prelude::*;

verus! {

// ===== 选段核的数据 =====
// 单元区里一个整段在选段那一刻的样子：胶水（singlefs-core 的 compaction_plan::choose_region 那一截）按分配记录、
// 这次挂载开过的段、活意图的 R、钉住的槽、被拒的段与扫描结果逐段算好，核只判。入参按段起点槽号升序排，下标就是段在单元区里的序。
// 扫过的段分两种：扫描结果里至少一个能搬的单元，与一个能搬的都没有。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionKernelSegmentScan {
    NotScanned,
    ScannedWithAMovableUnit,
    ScannedWithoutAMovableUnit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionKernelSegment {
    // greedy 的打分就是它（singlefs-core 的 compaction_plan::greedy_score）：段里未释放记录罩住的槽数。
    pub live_slots: u64,
    pub is_opened_in_this_mount: bool,
    pub is_the_live_intent_region: bool,
    pub holds_a_pinned_slot: bool,
    pub is_refused_by_the_region_scan: bool,
    pub scan: CompactionKernelSegmentScan,
}

// 交回的下标指入参里的段。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionKernelRegionChoice {
    Chosen(usize),
    ScanBeforeChoosing(usize),
    NoEligibleSegment,
}

// ===== 选段的 spec（可信：照判决 G2 的资格写）=====
// 候选：资格里除「扫过、至少一个能搬的单元」之外的四条都成立，而且不是「扫过而一个能搬的都没有」（没扫过的还不知道，留作候选）。
pub open spec fn segment_is_a_candidate(segment: CompactionKernelSegment) -> bool {
    &&& !segment.is_opened_in_this_mount
    &&& !segment.is_the_live_intent_region
    &&& !segment.holds_a_pinned_slot
    &&& !segment.is_refused_by_the_region_scan
    &&& !(segment.scan is ScannedWithoutAMovableUnit)
}

// P1：选中的段合资格五条；要先扫的段是候选、还没扫过；「一段都没资格」时没有一段是候选。
pub open spec fn region_choice_is_sound(
    segments: Seq<CompactionKernelSegment>,
    choice: CompactionKernelRegionChoice,
) -> bool {
    match choice {
        CompactionKernelRegionChoice::Chosen(chosen) => {
            &&& chosen < segments.len()
            &&& segment_is_a_candidate(segments[chosen as int])
            &&& segments[chosen as int].scan is ScannedWithAMovableUnit
        },
        CompactionKernelRegionChoice::ScanBeforeChoosing(to_scan) => {
            &&& to_scan < segments.len()
            &&& segment_is_a_candidate(segments[to_scan as int])
            &&& segments[to_scan as int].scan is NotScanned
        },
        CompactionKernelRegionChoice::NoEligibleSegment => forall|position: int|
            0 <= position < segments.len() ==> !segment_is_a_candidate(#[trigger] segments[position]),
    }
}

// singlefs-core 的 compaction_plan::choose_region 按记录与集合算完每段之后的那一截：候选里活槽数最少的先，同分取起点槽号最小的
// （入参里最前的）；排在最前的那一段扫过就选它，没扫过就要调用方先扫。
pub fn choose_compaction_segment(segments: &[CompactionKernelSegment]) -> (choice:
    CompactionKernelRegionChoice)
    ensures
        region_choice_is_sound(segments@, choice),
{
    let mut best: Option<usize> = None;
    let mut best_live_slots: u64 = 0;
    let mut position: usize = 0;
    while position < segments.len()
        invariant
            position <= segments.len(),
            best matches Some(best_position) ==> best_position < position
                && segment_is_a_candidate(segments@[best_position as int]),
            best is None ==> forall|earlier: int|
                0 <= earlier < position ==> !segment_is_a_candidate(#[trigger] segments@[earlier]),
        decreases segments.len() - position,
    {
        let segment = segments[position];
        let is_scanned_without_a_movable_unit = match segment.scan {
            CompactionKernelSegmentScan::ScannedWithoutAMovableUnit => true,
            CompactionKernelSegmentScan::NotScanned
            | CompactionKernelSegmentScan::ScannedWithAMovableUnit => false,
        };
        let is_a_candidate = !segment.is_opened_in_this_mount
            && !segment.is_the_live_intent_region
            && !segment.holds_a_pinned_slot
            && !segment.is_refused_by_the_region_scan
            && !is_scanned_without_a_movable_unit;
        if is_a_candidate {
            let scores_lower_than_the_best = match best {
                None => true,
                Some(_) => segment.live_slots < best_live_slots,
            };
            if scores_lower_than_the_best {
                best = Some(position);
                best_live_slots = segment.live_slots;
            }
        }
        position += 1;
    }
    match best {
        None => CompactionKernelRegionChoice::NoEligibleSegment,
        Some(best_position) => match segments[best_position].scan {
            CompactionKernelSegmentScan::ScannedWithAMovableUnit => CompactionKernelRegionChoice::Chosen(
                best_position,
            ),
            CompactionKernelSegmentScan::NotScanned => CompactionKernelRegionChoice::ScanBeforeChoosing(
                best_position,
            ),
            // 循环不变式：留下的那一段是候选，候选不是「扫过而一个能搬的都没有」（Verus 证这一臂走不到）。
            CompactionKernelSegmentScan::ScannedWithoutAMovableUnit => unreached(),
        },
    }
}

// ===== 计划核的数据 =====
// 区域扫描（singlefs-core 的 region_scan::scan_region）对 R 里一条未释放记录的分类。映射 key 的身份：胶水给每一把不同的 key
// 一个不同的数、同一把 key 同一个数，核只比相等。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompactionKernelAllocationClass {
    Movable { mapping_key_identity: u64 },
    Pinned,
    Quarantined,
    Dead,
    Inconsistent,
}

// R 里一条未释放记录：源落点的起点槽号与它的分类。入参按槽升序（区域扫描交回的次序）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompactionKernelScannedAllocation {
    pub source_slot: u64,
    pub class: CompactionKernelAllocationClass,
}

// 「搬」交的是要搬的那几项在入参里的下标，升序。
pub enum CompactionKernelPlannedBatch {
    Relocate(Vec<usize>),
    RegionEmptied,
    NothingMovableButTheRegionIsNotEmpty,
}

// ===== 计划的 spec（可信：照判决 G2 与 G7 的 P2、P3 写）=====
pub open spec fn allocation_is_movable(allocation: CompactionKernelScannedAllocation) -> bool {
    allocation.class is Movable
}

// 不能搬的几类没有映射 key；这里给 0 只为让函数是全的，用到它的地方都先判过能搬。
pub open spec fn mapping_key_identity_of(allocation: CompactionKernelScannedAllocation) -> u64 {
    match allocation.class {
        CompactionKernelAllocationClass::Movable { mapping_key_identity } => mapping_key_identity,
        CompactionKernelAllocationClass::Pinned
        | CompactionKernelAllocationClass::Quarantined
        | CompactionKernelAllocationClass::Dead
        | CompactionKernelAllocationClass::Inconsistent => 0,
    }
}

// 入参 [0, end) 里能搬的那几项的下标，升序。
pub open spec fn movable_positions_before(
    allocations: Seq<CompactionKernelScannedAllocation>,
    end: int,
) -> Seq<int>
    decreases end,
{
    if end <= 0 {
        Seq::empty()
    } else {
        let earlier = movable_positions_before(allocations, end - 1);
        if allocation_is_movable(allocations[end - 1]) {
            earlier.push(end - 1)
        } else {
            earlier
        }
    }
}

pub open spec fn movable_positions(allocations: Seq<CompactionKernelScannedAllocation>) -> Seq<
    int,
> {
    movable_positions_before(allocations, allocations.len() as int)
}

// 区域扫描判「可搬」时，头里的 key 在映射里、映射的位置就是这一槽；映射里同一把 key 只有一个位置，所以这一条在健康的映射上成立。
// 计划核不判它：坏盘上它不成立时计划怎么办，条款没写（报告交主 agent），这里只在它成立时证 key 互异。
pub open spec fn movable_mapping_keys_are_distinct(
    allocations: Seq<CompactionKernelScannedAllocation>,
) -> bool {
    forall|one: int, other: int|
        #![trigger allocations[one], allocations[other]]
        0 <= one < allocations.len() && 0 <= other < allocations.len() && one != other
            && allocation_is_movable(allocations[one]) && allocation_is_movable(allocations[other])
            ==> mapping_key_identity_of(allocations[one]) != mapping_key_identity_of(
            allocations[other],
        )
}

pub open spec fn positions_as_ints(positions: Seq<usize>) -> Seq<int> {
    Seq::new(positions.len(), |index: int| positions[index] as int)
}

pub open spec fn planned_positions(planned: CompactionKernelPlannedBatch) -> Seq<int> {
    match planned {
        CompactionKernelPlannedBatch::Relocate(positions) => positions_as_ints(positions@),
        CompactionKernelPlannedBatch::RegionEmptied
        | CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty => Seq::empty(),
    }
}

// 一批搬几个：能搬的不到 K 个就全搬，否则 K 个。
pub open spec fn units_in_the_batch(movable_units: int, batch_units: int) -> int {
    if movable_units < batch_units {
        movable_units
    } else {
        batch_units
    }
}

// P2：至多 K 项；每一项是入参里能搬的一项；下标严格升序（各项互异）；能搬的单元映射 key 互异时一批里的 key 互异。
pub open spec fn planned_batch_is_sound(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: usize,
    planned: CompactionKernelPlannedBatch,
) -> bool {
    let positions = planned_positions(planned);
    &&& positions.len() <= batch_units
    &&& forall|index: int|
        0 <= index < positions.len() ==> 0 <= #[trigger] positions[index] < allocations.len()
            && allocation_is_movable(allocations[positions[index]])
    &&& forall|earlier: int, later: int|
        #![trigger positions[earlier], positions[later]]
        0 <= earlier < later < positions.len() ==> positions[earlier] < positions[later]
    &&& movable_mapping_keys_are_distinct(allocations) ==> forall|earlier: int, later: int|
        #![trigger positions[earlier], positions[later]]
        0 <= earlier < later < positions.len() ==> mapping_key_identity_of(
            allocations[positions[earlier]],
        ) != mapping_key_identity_of(allocations[positions[later]])
}

// P3：「搬」至少一项；「搬空」时 R 里一条未释放记录都没有；「搬不动」时有记录、一个能搬的都没有。
// 合起来：R 里有能搬的单元，计划就是「搬」。
pub open spec fn planned_batch_makes_progress(
    allocations: Seq<CompactionKernelScannedAllocation>,
    planned: CompactionKernelPlannedBatch,
) -> bool {
    match planned {
        CompactionKernelPlannedBatch::Relocate(positions) => positions@.len() >= 1,
        CompactionKernelPlannedBatch::RegionEmptied => allocations.len() == 0,
        CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
            &&& allocations.len() > 0
            &&& forall|position: int|
                0 <= position < allocations.len() ==> !allocation_is_movable(
                    #[trigger] allocations[position],
                )
        },
    }
}

// 判决 G2 的 plan_batch：能搬的单元按槽升序取前 K 个（各批互不相交、并起来是 R 里能搬的全部，靠的就是每批都取最前的那几个）。
pub open spec fn planned_batch_takes_the_first_movable_units(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: usize,
    planned: CompactionKernelPlannedBatch,
) -> bool {
    let movable = movable_positions(allocations);
    planned_positions(planned) == movable.take(
        units_in_the_batch(movable.len() as int, batch_units as int),
    )
}

// ===== 计划的引理（不可信件：写错了只会证不出来）=====
proof fn lemma_movable_positions_are_the_movable_allocations_in_order(
    allocations: Seq<CompactionKernelScannedAllocation>,
    end: int,
)
    requires
        0 <= end <= allocations.len(),
    ensures
        ({
            let movable = movable_positions_before(allocations, end);
            &&& movable.len() <= end
            &&& forall|index: int|
                0 <= index < movable.len() ==> 0 <= #[trigger] movable[index] < end
                    && allocation_is_movable(allocations[movable[index]])
            &&& forall|earlier: int, later: int|
                #![trigger movable[earlier], movable[later]]
                0 <= earlier < later < movable.len() ==> movable[earlier] < movable[later]
            &&& forall|position: int|
                0 <= position < end && allocation_is_movable(#[trigger] allocations[position])
                    ==> exists|index: int| 0 <= index < movable.len() && movable[index] == position
        }),
    decreases end,
{
    if end > 0 {
        lemma_movable_positions_are_the_movable_allocations_in_order(allocations, end - 1);
        let earlier = movable_positions_before(allocations, end - 1);
        let movable = movable_positions_before(allocations, end);
        if allocation_is_movable(allocations[end - 1]) {
            assert(movable == earlier.push(end - 1));
            assert forall|position: int|
                0 <= position < end && allocation_is_movable(
                    #[trigger] allocations[position],
                ) implies exists|index: int|
                0 <= index < movable.len() && movable[index] == position by {
                if position == end - 1 {
                    assert(movable[earlier.len() as int] == position);
                } else {
                    let index = choose|index: int|
                        0 <= index < earlier.len() && earlier[index] == position;
                    assert(movable[index] == position);
                }
            }
        } else {
            assert(movable == earlier);
        }
    }
}

proof fn lemma_every_position_is_movable_when_every_allocation_is(
    allocations: Seq<CompactionKernelScannedAllocation>,
    end: int,
)
    requires
        0 <= end <= allocations.len(),
        forall|position: int|
            0 <= position < allocations.len() ==> allocation_is_movable(
                #[trigger] allocations[position],
            ),
    ensures
        movable_positions_before(allocations, end) == Seq::new(end as nat, |index: int| index),
    decreases end,
{
    if end > 0 {
        lemma_every_position_is_movable_when_every_allocation_is(allocations, end - 1);
        assert(movable_positions_before(allocations, end) =~= Seq::new(
            end as nat,
            |index: int| index,
        ));
    } else {
        assert(movable_positions_before(allocations, end) =~= Seq::new(
            end as nat,
            |index: int| index,
        ));
    }
}

// ===== 计划核 =====
// singlefs-core 的 compaction_plan::plan_batch：扫描结果里能搬的按槽升序取前 batch_units 个；一个都没取到时，R 里没有未释放记录
// 交「搬空」，还有交「搬不动」。
pub fn plan_compaction_batch(
    allocations: &[CompactionKernelScannedAllocation],
    batch_units: usize,
) -> (planned: CompactionKernelPlannedBatch)
    requires
        batch_units >= 1,
    ensures
        planned_batch_is_sound(allocations@, batch_units, planned),
        planned_batch_makes_progress(allocations@, planned),
        planned_batch_takes_the_first_movable_units(allocations@, batch_units, planned),
{
    let mut positions: Vec<usize> = Vec::new();
    let mut position: usize = 0;
    while position < allocations.len()
        invariant
            position <= allocations.len(),
            batch_units >= 1,
            positions_as_ints(positions@) == movable_positions_before(
                allocations@,
                position as int,
            ).take(
                units_in_the_batch(
                    movable_positions_before(allocations@, position as int).len() as int,
                    batch_units as int,
                ),
            ),
        decreases allocations.len() - position,
    {
        let ghost movable_before = movable_positions_before(allocations@, position as int);
        let ghost positions_before = positions@;
        proof {
            lemma_movable_positions_are_the_movable_allocations_in_order(
                allocations@,
                position as int,
            );
        }
        let is_movable = match allocations[position].class {
            CompactionKernelAllocationClass::Movable { .. } => true,
            CompactionKernelAllocationClass::Pinned
            | CompactionKernelAllocationClass::Quarantined
            | CompactionKernelAllocationClass::Dead
            | CompactionKernelAllocationClass::Inconsistent => false,
        };
        if is_movable && positions.len() < batch_units {
            positions.push(position);
        }
        proof {
            let movable_after = movable_positions_before(allocations@, position as int + 1);
            if is_movable {
                assert(movable_after == movable_before.push(position as int));
                if movable_before.len() < batch_units {
                    assert(movable_before.take(movable_before.len() as int) =~= movable_before);
                    assert(positions@ == positions_before.push(position));
                    assert(movable_after.take(movable_after.len() as int) =~= movable_after);
                    assert(positions_as_ints(positions@) =~= movable_after.take(
                        units_in_the_batch(movable_after.len() as int, batch_units as int),
                    ));
                } else {
                    assert(positions@ == positions_before);
                    assert(movable_after.take(batch_units as int) =~= movable_before.take(
                        batch_units as int,
                    ));
                }
            } else {
                assert(movable_after == movable_before);
            }
        }
        position += 1;
    }
    proof {
        lemma_movable_positions_are_the_movable_allocations_in_order(
            allocations@,
            allocations@.len() as int,
        );
        assert(allocations@.subrange(0, allocations@.len() as int) =~= allocations@);
    }
    if !positions.is_empty() {
        return CompactionKernelPlannedBatch::Relocate(positions);
    }
    proof {
        assert(movable_positions(allocations@).len() == 0);
        assert(movable_positions(allocations@).take(0) =~= Seq::<int>::empty());
    }
    if allocations.is_empty() {
        CompactionKernelPlannedBatch::RegionEmptied
    } else {
        CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty
    }
}

// ===== P4：抽象恒等引理（执行层的模型，不是执行层）=====
// 具体状态：映射（key 的身份 → 落点起点槽号），加上每个槽号上的字节。抽象状态：每个 key 经映射读到的字节。
pub open spec fn bytes_read_through_the_mapping(
    mapping: Map<u64, u64>,
    storage: spec_fn(u64) -> Seq<u8>,
    mapping_key_identity: u64,
) -> Seq<u8> {
    storage(mapping[mapping_key_identity])
}

// 搬一个单元的写：源落点上的字节原样写到目的落点（判决 G3：头不盖戳、birth 不变），别的槽不动。
pub open spec fn storage_after_copying(
    storage: spec_fn(u64) -> Seq<u8>,
    source_slot: u64,
    destination_slot: u64,
) -> spec_fn(u64) -> Seq<u8> {
    |slot: u64|
        if slot == destination_slot {
            storage(source_slot)
        } else {
            storage(slot)
        }
}

// 按次序搬前 count 项：第 j 项把 keys[j] 的映射条目换到 destinations[j]。
pub open spec fn mapping_after_relocating(
    mapping: Map<u64, u64>,
    keys: Seq<u64>,
    destinations: Seq<u64>,
    count: int,
) -> Map<u64, u64>
    decreases count,
{
    if count <= 0 {
        mapping
    } else {
        mapping_after_relocating(mapping, keys, destinations, count - 1).insert(
            keys[count - 1],
            destinations[count - 1],
        )
    }
}

// 按次序搬前 count 项：第 j 项把 sources[j] 上的字节写到 destinations[j]。
pub open spec fn storage_after_relocating(
    storage: spec_fn(u64) -> Seq<u8>,
    sources: Seq<u64>,
    destinations: Seq<u64>,
    count: int,
) -> spec_fn(u64) -> Seq<u8>
    decreases count,
{
    if count <= 0 {
        storage
    } else {
        storage_after_copying(
            storage_after_relocating(storage, sources, destinations, count - 1),
            sources[count - 1],
            destinations[count - 1],
        )
    }
}

// 目的地是分配器的性质（判决 G2）：分配器交的落点此刻没被映射里任何一个 key 占着（「目的地不在 R 内」是它的一个特例：R 里能搬的单元
// 都在映射里）。
pub open spec fn destination_is_free_in_the_mapping(mapping: Map<u64, u64>, destination: u64) -> bool {
    forall|mapping_key_identity: u64|
        #[trigger] mapping.contains_key(mapping_key_identity) ==> mapping[mapping_key_identity]
            != destination
}

// 前 count 个目的地两两不重叠、都不被映射占着。
pub open spec fn destinations_are_distinct_and_free(
    mapping: Map<u64, u64>,
    destinations: Seq<u64>,
    count: int,
) -> bool {
    &&& count <= destinations.len()
    &&& forall|earlier: int, later: int|
        #![trigger destinations[earlier], destinations[later]]
        0 <= earlier < later < count ==> destinations[earlier] != destinations[later]
    &&& forall|index: int|
        0 <= index < count ==> destination_is_free_in_the_mapping(
            mapping,
            #[trigger] destinations[index],
        )
}

// 区域扫描的「可搬」：key 在映射里、映射的位置就是这一项的源落点。
pub open spec fn scan_agrees_with_the_mapping(
    allocations: Seq<CompactionKernelScannedAllocation>,
    mapping: Map<u64, u64>,
) -> bool {
    forall|position: int|
        0 <= position < allocations.len() && allocation_is_movable(#[trigger] allocations[position])
            ==> mapping.contains_key(mapping_key_identity_of(allocations[position]))
            && mapping[mapping_key_identity_of(allocations[position])]
            == allocations[position].source_slot
}

pub open spec fn planned_mapping_keys(
    allocations: Seq<CompactionKernelScannedAllocation>,
    positions: Seq<int>,
) -> Seq<u64> {
    positions.map_values(|position: int| mapping_key_identity_of(allocations[position]))
}

pub open spec fn planned_sources(
    allocations: Seq<CompactionKernelScannedAllocation>,
    positions: Seq<int>,
) -> Seq<u64> {
    positions.map_values(|position: int| allocations[position].source_slot)
}

proof fn lemma_relocating_in_order_keeps_every_read(
    mapping: Map<u64, u64>,
    storage: spec_fn(u64) -> Seq<u8>,
    keys: Seq<u64>,
    sources: Seq<u64>,
    destinations: Seq<u64>,
    count: int,
)
    requires
        0 <= count <= keys.len(),
        count <= sources.len(),
        forall|earlier: int, later: int|
            #![trigger keys[earlier], keys[later]]
            0 <= earlier < later < count ==> keys[earlier] != keys[later],
        forall|index: int|
            0 <= index < count ==> mapping.contains_key(#[trigger] keys[index]) && mapping[keys[index]]
                == sources[index],
        destinations_are_distinct_and_free(mapping, destinations, count),
    ensures
        ({
            let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
            let moved_storage = storage_after_relocating(storage, sources, destinations, count);
            &&& moved_mapping.dom() == mapping.dom()
            &&& forall|mapping_key_identity: u64|
                #[trigger] mapping.contains_key(mapping_key_identity) ==> bytes_read_through_the_mapping(
                    moved_mapping,
                    moved_storage,
                    mapping_key_identity,
                ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity)
            &&& forall|index: int|
                0 <= index < count ==> moved_mapping[#[trigger] keys[index]] == destinations[index]
            &&& forall|mapping_key_identity: u64|
                #[trigger] mapping.contains_key(mapping_key_identity) && (forall|index: int|
                    0 <= index < count ==> keys[index] != mapping_key_identity)
                    ==> moved_mapping[mapping_key_identity] == mapping[mapping_key_identity]
            &&& forall|mapping_key_identity: u64|
                #[trigger] moved_mapping.contains_key(mapping_key_identity) ==> moved_mapping[mapping_key_identity]
                    == mapping[mapping_key_identity] || exists|index: int|
                    0 <= index < count && moved_mapping[mapping_key_identity] == destinations[index]
            &&& forall|slot: u64|
                (forall|index: int| 0 <= index < count ==> destinations[index] != slot) ==> #[trigger] moved_storage(slot)
                    == storage(slot)
        }),
    decreases count,
{
    if count > 0 {
        let previous_count = count - 1;
        lemma_relocating_in_order_keeps_every_read(
            mapping,
            storage,
            keys,
            sources,
            destinations,
            previous_count,
        );
        let previous_mapping = mapping_after_relocating(mapping, keys, destinations, previous_count);
        let previous_storage = storage_after_relocating(storage, sources, destinations, previous_count);
        let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
        let moved_storage = storage_after_relocating(storage, sources, destinations, count);
        let moved_key = keys[previous_count];
        let source = sources[previous_count];
        let destination = destinations[previous_count];
        assert(moved_mapping == previous_mapping.insert(moved_key, destination));
        assert(moved_storage == storage_after_copying(previous_storage, source, destination));
        assert(mapping.contains_key(moved_key) && mapping[moved_key] == source);
        assert forall|index: int| 0 <= index < previous_count implies keys[index] != moved_key by {
            assert(keys[index] != keys[previous_count]);
        }
        assert(previous_mapping[moved_key] == mapping[moved_key]);
        assert forall|index: int| 0 <= index < previous_count implies destinations[index] != source by {
            assert(destination_is_free_in_the_mapping(mapping, destinations[index]));
        }
        assert(previous_storage(source) == storage(source));
        assert(destination_is_free_in_the_mapping(mapping, destination));
        assert(moved_mapping.dom() =~= mapping.dom());
        assert forall|mapping_key_identity: u64| #[trigger]
            moved_mapping.contains_key(mapping_key_identity) implies moved_mapping[mapping_key_identity]
            == mapping[mapping_key_identity] || exists|index: int|
            0 <= index < count && moved_mapping[mapping_key_identity] == destinations[index] by {
            if mapping_key_identity == moved_key {
                assert(moved_mapping[mapping_key_identity] == destinations[previous_count]);
            } else {
                assert(previous_mapping.contains_key(mapping_key_identity));
            }
        }
        assert forall|mapping_key_identity: u64| #[trigger]
            mapping.contains_key(mapping_key_identity) implies bytes_read_through_the_mapping(
            moved_mapping,
            moved_storage,
            mapping_key_identity,
        ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity) by {
            if mapping_key_identity == moved_key {
                assert(moved_mapping[mapping_key_identity] == destination);
                assert(moved_storage(destination) == previous_storage(source));
            } else {
                assert(previous_mapping.contains_key(mapping_key_identity));
                let previous_slot = previous_mapping[mapping_key_identity];
                assert(moved_mapping[mapping_key_identity] == previous_slot);
                if previous_slot == mapping[mapping_key_identity] {
                    assert(previous_slot != destination);
                } else {
                    let index = choose|index: int|
                        0 <= index < previous_count && previous_slot == destinations[index];
                    assert(destinations[index] != destinations[previous_count]);
                }
                assert(moved_storage(previous_slot) == previous_storage(previous_slot));
                assert(bytes_read_through_the_mapping(
                    previous_mapping,
                    previous_storage,
                    mapping_key_identity,
                ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity));
            }
        }
        assert forall|index: int| 0 <= index < count implies moved_mapping[#[trigger] keys[index]]
            == destinations[index] by {
            if index < previous_count {
                assert(keys[index] != keys[previous_count]);
            }
        }
        assert forall|slot: u64|
            (forall|index: int| 0 <= index < count ==> destinations[index] != slot) implies #[trigger] moved_storage(
            slot,
        ) == storage(slot) by {
            assert(destinations[previous_count] != slot);
            assert forall|index: int| 0 <= index < previous_count implies destinations[index] != slot by {
                assert(0 <= index < count);
            }
        }
    }
}

// P4，一批：计划合 P2（key 互异来自扫描结果里能搬的 key 互异）、扫描的「可搬」与映射对得上、目的地两两不重叠且没被映射占着
// （分配器的性质），按计划搬完这一批，每个 key 读到的字节不变，映射里的 key 一个不多一个不少。
pub proof fn lemma_a_planned_batch_keeps_every_read(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: usize,
    planned: CompactionKernelPlannedBatch,
    mapping: Map<u64, u64>,
    storage: spec_fn(u64) -> Seq<u8>,
    destinations: Seq<u64>,
)
    requires
        planned_batch_is_sound(allocations, batch_units, planned),
        movable_mapping_keys_are_distinct(allocations),
        scan_agrees_with_the_mapping(allocations, mapping),
        destinations_are_distinct_and_free(
            mapping,
            destinations,
            planned_positions(planned).len() as int,
        ),
    ensures
        ({
            let positions = planned_positions(planned);
            let moved_mapping = mapping_after_relocating(
                mapping,
                planned_mapping_keys(allocations, positions),
                destinations,
                positions.len() as int,
            );
            let moved_storage = storage_after_relocating(
                storage,
                planned_sources(allocations, positions),
                destinations,
                positions.len() as int,
            );
            &&& moved_mapping.dom() == mapping.dom()
            &&& forall|mapping_key_identity: u64|
                #[trigger] mapping.contains_key(mapping_key_identity) ==> bytes_read_through_the_mapping(
                    moved_mapping,
                    moved_storage,
                    mapping_key_identity,
                ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity)
        }),
{
    let positions = planned_positions(planned);
    let keys = planned_mapping_keys(allocations, positions);
    let sources = planned_sources(allocations, positions);
    assert forall|earlier: int, later: int|
        #![trigger keys[earlier], keys[later]]
        0 <= earlier < later < positions.len() implies keys[earlier] != keys[later] by {
        assert(positions[earlier] < positions[later]);
        assert(allocation_is_movable(allocations[positions[earlier]]));
        assert(allocation_is_movable(allocations[positions[later]]));
    }
    assert forall|index: int| 0 <= index < positions.len() implies mapping.contains_key(
        #[trigger] keys[index],
    ) && mapping[keys[index]] == sources[index] by {
        assert(0 <= positions[index] < allocations.len());
        assert(allocation_is_movable(allocations[positions[index]]));
    }
    lemma_relocating_in_order_keeps_every_read(
        mapping,
        storage,
        keys,
        sources,
        destinations,
        positions.len() as int,
    );
}

// ===== P4，搬到 R 空 =====
// 前提「R 内每条未释放记录都有映射 key 且不在隔离集里」（攻方 H-G7-1）在扫描结果上就是每一项都能搬：钉住的豁免三类没有映射 key，
// 已死的映射不指它，隔离的在隔离集里，不一致的是映射与单元头对不上。
pub open spec fn every_allocation_in_the_region_is_movable(
    allocations: Seq<CompactionKernelScannedAllocation>,
) -> bool {
    forall|position: int|
        0 <= position < allocations.len() ==> allocation_is_movable(#[trigger] allocations[position])
}

pub open spec fn mapping_key_identities(allocations: Seq<CompactionKernelScannedAllocation>) -> Seq<
    u64,
> {
    allocations.map_values(
        |allocation: CompactionKernelScannedAllocation| mapping_key_identity_of(allocation),
    )
}

pub open spec fn source_slots(allocations: Seq<CompactionKernelScannedAllocation>) -> Seq<u64> {
    allocations.map_values(|allocation: CompactionKernelScannedAllocation| allocation.source_slot)
}

// 一批批搬到 R 空：每一批搬剩下的最前 min(K, 剩下几项) 项（前提下计划就交这几项，lemma_under_the_premise_a_plan_takes_the_leading_allocations），
// 目的地按次序从 destinations 里取；这一批发布之后它们离开 R，下一次扫描就是剩下的那几项（singlefs-core 的 compaction_plan::plan_batch 文档）。
pub open spec fn mapping_after_compacting_the_region(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: int,
    mapping: Map<u64, u64>,
    destinations: Seq<u64>,
) -> Map<u64, u64>
    decreases allocations.len(),
{
    if allocations.len() == 0 || batch_units <= 0 {
        mapping
    } else {
        let count = units_in_the_batch(allocations.len() as int, batch_units);
        mapping_after_compacting_the_region(
            allocations.skip(count),
            batch_units,
            mapping_after_relocating(mapping, mapping_key_identities(allocations), destinations, count),
            destinations.skip(count),
        )
    }
}

pub open spec fn storage_after_compacting_the_region(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: int,
    storage: spec_fn(u64) -> Seq<u8>,
    destinations: Seq<u64>,
) -> spec_fn(u64) -> Seq<u8>
    decreases allocations.len(),
{
    if allocations.len() == 0 || batch_units <= 0 {
        storage
    } else {
        let count = units_in_the_batch(allocations.len() as int, batch_units);
        storage_after_compacting_the_region(
            allocations.skip(count),
            batch_units,
            storage_after_relocating(storage, source_slots(allocations), destinations, count),
            destinations.skip(count),
        )
    }
}

// 前提下，计划交的就是剩下的最前 min(K, 剩下几项) 项，而且是「搬」（P3：R 不空就有进展）。
pub proof fn lemma_under_the_premise_a_plan_takes_the_leading_allocations(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: usize,
    planned: CompactionKernelPlannedBatch,
)
    requires
        batch_units >= 1,
        allocations.len() > 0,
        every_allocation_in_the_region_is_movable(allocations),
        planned_batch_makes_progress(allocations, planned),
        planned_batch_takes_the_first_movable_units(allocations, batch_units, planned),
    ensures
        planned is Relocate,
        planned_positions(planned) == Seq::new(
            units_in_the_batch(allocations.len() as int, batch_units as int) as nat,
            |index: int| index,
        ),
{
    lemma_every_position_is_movable_when_every_allocation_is(allocations, allocations.len() as int);
    let count = units_in_the_batch(allocations.len() as int, batch_units as int);
    assert(movable_positions(allocations).take(count) =~= Seq::new(
        count as nat,
        |index: int| index,
    ));
    assert(allocation_is_movable(allocations[0]));
}

// P4：前提下按计划一批批搬到 R 空，每个 key 读到的字节不变，映射里的 key 一个不多一个不少。
pub proof fn corollary_compacting_the_region_until_it_is_empty_keeps_every_read(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: int,
    mapping: Map<u64, u64>,
    storage: spec_fn(u64) -> Seq<u8>,
    destinations: Seq<u64>,
)
    requires
        batch_units >= 1,
        every_allocation_in_the_region_is_movable(allocations),
        movable_mapping_keys_are_distinct(allocations),
        scan_agrees_with_the_mapping(allocations, mapping),
        destinations_are_distinct_and_free(mapping, destinations, allocations.len() as int),
    ensures
        ({
            let moved_mapping = mapping_after_compacting_the_region(
                allocations,
                batch_units,
                mapping,
                destinations,
            );
            let moved_storage = storage_after_compacting_the_region(
                allocations,
                batch_units,
                storage,
                destinations,
            );
            &&& moved_mapping.dom() == mapping.dom()
            &&& forall|mapping_key_identity: u64|
                #[trigger] mapping.contains_key(mapping_key_identity) ==> bytes_read_through_the_mapping(
                    moved_mapping,
                    moved_storage,
                    mapping_key_identity,
                ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity)
        }),
    decreases allocations.len(),
{
    if allocations.len() > 0 {
        let count = units_in_the_batch(allocations.len() as int, batch_units);
        let keys = mapping_key_identities(allocations);
        let sources = source_slots(allocations);
        assert forall|earlier: int, later: int|
            #![trigger keys[earlier], keys[later]]
            0 <= earlier < later < count implies keys[earlier] != keys[later] by {
            assert(allocation_is_movable(allocations[earlier]));
            assert(allocation_is_movable(allocations[later]));
        }
        assert forall|index: int| 0 <= index < count implies mapping.contains_key(
            #[trigger] keys[index],
        ) && mapping[keys[index]] == sources[index] by {
            assert(allocation_is_movable(allocations[index]));
        }
        lemma_relocating_in_order_keeps_every_read(
            mapping,
            storage,
            keys,
            sources,
            destinations,
            count,
        );
        let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
        let moved_storage = storage_after_relocating(storage, sources, destinations, count);
        let remaining = allocations.skip(count);
        let remaining_destinations = destinations.skip(count);
        assert forall|position: int| 0 <= position < remaining.len() implies allocation_is_movable(
            #[trigger] remaining[position],
        ) by {
            assert(remaining[position] == allocations[position + count]);
        }
        assert forall|one: int, other: int|
            #![trigger remaining[one], remaining[other]]
            0 <= one < remaining.len() && 0 <= other < remaining.len() && one != other
                && allocation_is_movable(remaining[one]) && allocation_is_movable(remaining[other])
                implies mapping_key_identity_of(remaining[one]) != mapping_key_identity_of(
            remaining[other],
        ) by {
            assert(remaining[one] == allocations[one + count]);
            assert(remaining[other] == allocations[other + count]);
        }
        assert forall|position: int|
            0 <= position < remaining.len() && allocation_is_movable(
                #[trigger] remaining[position],
            ) implies moved_mapping.contains_key(mapping_key_identity_of(remaining[position]))
            && moved_mapping[mapping_key_identity_of(remaining[position])]
            == remaining[position].source_slot by {
            let original = position + count;
            assert(remaining[position] == allocations[original]);
            assert(allocation_is_movable(allocations[original]));
            let key = mapping_key_identity_of(allocations[original]);
            assert(mapping.contains_key(key));
            assert forall|index: int| 0 <= index < count implies keys[index] != key by {
                assert(allocation_is_movable(allocations[index]));
                assert(keys[index] == mapping_key_identity_of(allocations[index]));
            }
            assert(moved_mapping[key] == mapping[key]);
        }
        assert forall|earlier: int, later: int|
            #![trigger remaining_destinations[earlier], remaining_destinations[later]]
            0 <= earlier < later < remaining.len() implies remaining_destinations[earlier]
            != remaining_destinations[later] by {
            assert(remaining_destinations[earlier] == destinations[earlier + count]);
            assert(remaining_destinations[later] == destinations[later + count]);
        }
        assert forall|index: int| 0 <= index < remaining.len() implies destination_is_free_in_the_mapping(
            moved_mapping,
            #[trigger] remaining_destinations[index],
        ) by {
            let original = index + count;
            assert(remaining_destinations[index] == destinations[original]);
            assert(destination_is_free_in_the_mapping(mapping, destinations[original]));
            assert forall|mapping_key_identity: u64| #[trigger]
                moved_mapping.contains_key(mapping_key_identity) implies moved_mapping[mapping_key_identity]
                != destinations[original] by {
                if moved_mapping[mapping_key_identity] != mapping[mapping_key_identity] {
                    let moved_index = choose|moved_index: int|
                        0 <= moved_index < count && moved_mapping[mapping_key_identity]
                            == destinations[moved_index];
                    assert(destinations[moved_index] != destinations[original]);
                }
            }
        }
        corollary_compacting_the_region_until_it_is_empty_keeps_every_read(
            remaining,
            batch_units,
            moved_mapping,
            moved_storage,
            remaining_destinations,
        );
    }
}

// ===== spec 自己的冒烟（防 spec 恒真或恒假）=====
// 一项能搬、K = 1：交「搬」那一项合 P2、P3 与「取最前的」；交「搬空」不合 P3。
proof fn one_movable_allocation_is_planned_and_not_reported_emptied(
    only: CompactionKernelScannedAllocation,
    positions: Vec<usize>,
)
    requires
        allocation_is_movable(only),
        positions@ == seq![0usize],
    ensures
        planned_batch_takes_the_first_movable_units(
            seq![only],
            1,
            CompactionKernelPlannedBatch::Relocate(positions),
        ),
        !planned_batch_makes_progress(seq![only], CompactionKernelPlannedBatch::RegionEmptied),
{
    let allocations = seq![only];
    assert(allocations[0] == only);
    assert(movable_positions_before(allocations, 0) =~= Seq::<int>::empty());
    assert(movable_positions(allocations) =~= seq![0int]);
    assert(movable_positions(allocations).take(1) =~= seq![0int]);
    assert(positions_as_ints(positions@) =~= seq![0int]);
}

// 选段：唯一一段装着钉住的槽，「选中它」不合 P1。
proof fn a_segment_holding_a_pinned_slot_is_not_a_sound_choice(only: CompactionKernelSegment)
    requires
        only.holds_a_pinned_slot,
    ensures
        !region_choice_is_sound(seq![only], CompactionKernelRegionChoice::Chosen(0)),
{
}

} // verus!

// 平时的 cargo test 跑的几条：Verus 证的是 spec，这几条钉住编进这个 crate 的两个核在手算得出的小例子上交的答案
// （crates/mutations.tsv 里点名它们的变异改坏核，它们红）。核与 singlefs-core 的 compaction_plan 逐项对拍在那边的单测里。
#[cfg(test)]
mod tests {
    use super::{
        choose_compaction_segment, plan_compaction_batch, CompactionKernelAllocationClass,
        CompactionKernelPlannedBatch, CompactionKernelRegionChoice,
        CompactionKernelScannedAllocation, CompactionKernelSegment, CompactionKernelSegmentScan,
    };

    fn segment(live_slots: u64, scan: CompactionKernelSegmentScan) -> CompactionKernelSegment {
        CompactionKernelSegment {
            live_slots,
            is_opened_in_this_mount: false,
            is_the_live_intent_region: false,
            holds_a_pinned_slot: false,
            is_refused_by_the_region_scan: false,
            scan,
        }
    }

    fn allocation(
        source_slot: u64,
        class: CompactionKernelAllocationClass,
    ) -> CompactionKernelScannedAllocation {
        CompactionKernelScannedAllocation { source_slot, class }
    }

    fn movable(mapping_key_identity: u64) -> CompactionKernelAllocationClass {
        CompactionKernelAllocationClass::Movable {
            mapping_key_identity,
        }
    }

    fn relocated_positions(planned: CompactionKernelPlannedBatch) -> Vec<usize> {
        match planned {
            CompactionKernelPlannedBatch::Relocate(positions) => positions,
            CompactionKernelPlannedBatch::RegionEmptied => {
                panic!("期望「搬」，计划交了「搬空」")
            }
            CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty => {
                panic!("期望「搬」，计划交了「搬不动」")
            }
        }
    }

    /// 三段：活槽 5（扫过、有能搬的）、活槽 2（扫过、有能搬的）、活槽 2（没扫过）⇒ 选活槽最少、同分起点最前的那一段（下标 1）。
    #[test]
    fn the_fewest_live_slots_win_and_a_tie_goes_to_the_earlier_segment() {
        let segments = [
            segment(5, CompactionKernelSegmentScan::ScannedWithAMovableUnit),
            segment(2, CompactionKernelSegmentScan::ScannedWithAMovableUnit),
            segment(2, CompactionKernelSegmentScan::NotScanned),
        ];
        assert_eq!(
            choose_compaction_segment(&segments),
            CompactionKernelRegionChoice::Chosen(1),
            "活槽 2 的两段同分，取起点最前的下标 1；它扫过、有能搬的单元，直接选中"
        );
    }

    /// 活槽最少的那一段装着钉住的槽、次少的被这次挂载开过 ⇒ 都不是候选，剩下没扫过的那一段要先扫。
    #[test]
    fn excluded_segments_are_skipped_and_an_unscanned_candidate_is_scanned_first() {
        let mut pinned = segment(0, CompactionKernelSegmentScan::ScannedWithAMovableUnit);
        pinned.holds_a_pinned_slot = true;
        let mut opened = segment(1, CompactionKernelSegmentScan::ScannedWithAMovableUnit);
        opened.is_opened_in_this_mount = true;
        let segments = [
            pinned,
            opened,
            segment(9, CompactionKernelSegmentScan::NotScanned),
        ];
        assert_eq!(
            choose_compaction_segment(&segments),
            CompactionKernelRegionChoice::ScanBeforeChoosing(2),
            "钉住的段与这次挂载开过的段都不是候选，只剩没扫过的下标 2：先扫它"
        );
    }

    /// 扫过而一个能搬的都没有的段、活意图的 R、被扫描拒过的段都不是候选 ⇒ 一段都没资格。
    #[test]
    fn no_segment_is_eligible_when_each_one_is_excluded() {
        let mut live_intent = segment(1, CompactionKernelSegmentScan::ScannedWithAMovableUnit);
        live_intent.is_the_live_intent_region = true;
        let mut refused = segment(1, CompactionKernelSegmentScan::NotScanned);
        refused.is_refused_by_the_region_scan = true;
        let segments = [
            segment(0, CompactionKernelSegmentScan::ScannedWithoutAMovableUnit),
            live_intent,
            refused,
        ];
        assert_eq!(
            choose_compaction_segment(&segments),
            CompactionKernelRegionChoice::NoEligibleSegment,
            "三段各有一条不合资格，一段都不该选"
        );
    }

    /// 能搬、钉住、能搬、已死、能搬，K = 2 ⇒ 搬下标 0 与 2，跳过钉住与已死、不取第三个能搬的。
    #[test]
    fn a_batch_takes_the_first_movable_units_up_to_the_batch_size() {
        let allocations = [
            allocation(64, movable(10)),
            allocation(65, CompactionKernelAllocationClass::Pinned),
            allocation(66, movable(11)),
            allocation(68, CompactionKernelAllocationClass::Dead),
            allocation(69, movable(12)),
        ];
        assert_eq!(
            relocated_positions(plan_compaction_batch(&allocations, 2)),
            vec![0, 2],
            "K = 2：按槽升序取前两个能搬的（下标 0、2），钉住与已死的不进"
        );
    }

    /// R 里没有记录 ⇒ 搬空；只有隔离与不一致的 ⇒ 搬不动。
    #[test]
    fn an_empty_region_is_emptied_and_a_region_without_a_movable_unit_is_not() {
        assert!(
            matches!(
                plan_compaction_batch(&[], 1),
                CompactionKernelPlannedBatch::RegionEmptied
            ),
            "R 里一条未释放记录都没有：搬空"
        );
        let stuck = [
            allocation(64, CompactionKernelAllocationClass::Quarantined),
            allocation(66, CompactionKernelAllocationClass::Inconsistent),
        ];
        assert!(
            matches!(
                plan_compaction_batch(&stuck, 3),
                CompactionKernelPlannedBatch::NothingMovableButTheRegionIsNotEmpty
            ),
            "R 里有隔离与不一致的记录、一个能搬的都没有：搬不动，不是搬空"
        );
    }
}

//! 整理的选段与计划的 Verus 核与证明（里程碑四二·4；判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G7 的 V-1：
//! 放进这个 crate 的一个模块）。`singlefs-core` 的 `compaction_plan::choose_region` 与 `compaction_plan::plan_batch` 去掉读记录、
//! 读扫描结果之后剩下的判定写成两个可执行的核（[`choose_compaction_segment`]、[`plan_compaction_batch`]），证判决 G7 点名的四条：
//! - P1 选段健全与贪心选择（活槽数最少、平局取最前）：选出的段合 G2 的资格——不是这次挂载开过的段、不是活意图的 R、没装钉住的槽、没被区域扫描拒过、
//!   扫过而且至少一个能搬的单元；「一段都没资格」时真没有一段是候选。
//! - P2 计划健全：一批至多 K 项、每一项是能搬的单元、各项互异；扫描结果里能搬的单元映射 key 互异时，一批里的 key 互异。
//!   目的地那一半（不在 R 内、两两不重叠、跨度相同）按判决 G2 是分配器的性质，计划函数不交目的地，这里只证 key 一侧。
//! - P3 进展：交「搬」时至少一项；不搬时分「搬空」（R 里一条未释放记录都没有）与「搬不动」（有记录、一个能搬的都没有）。
//! - P4 抽象恒等引理：映射（key → 落点的槽区间，起点加跨度）加每个槽上的字节是具体状态，每个 key 经映射读到的单元是抽象状态。
//!   目的区间两两不重叠、不与现行映射的区间重叠、不压住回退候选与读者的区间、跨度等于源区间时，按计划搬一批（源区间的字节原样写到
//!   目的区间、映射条目换到目的区间）不改现行映射与回退候选、读者任何一个 key 读到的单元；在前提「R 内每条未释放记录都有映射 key 且
//!   不在隔离集里」（攻方 H-G7-1）下，一批批搬到 R 空也不改（判决 m4-exit5-relocation-proof-r1 第二节按区间、r3 第二节 Z1 回退候选进前提）。
//!
//! 核心层 `compaction_plan::choose_region` 与 `plan_batch` 直接调用这两个核；输入转换仍是普通 Rust，
//! 单测保留独立参考算法并对拍适配层。协议层的根环、回退候选、两层 F、读者与崩溃历史模型见 `gc_ring_interval_proofs`；
//! 设备屏障、字节解码与实际发布路径到该模型的精化仍是证明边界。
//! 必红世界与 `src/lib.rs` 的同登记在 `verus-must-red-worlds.tsv`。

#[allow(
    unused_imports,
    reason = "只有 Verus 编译时留下的证明代码用这些名字；普通构建擦掉证明代码之后没人用，擦掉的 spec fn 也不在，所以整模块导入"
)]
use crate::gc_ring_interval_proofs::*;
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

// 候选中活槽数最少，平局取输入中最前的一段；输入由胶水按槽号升序排列。
pub open spec fn region_choice_is_greedy(
    segments: Seq<CompactionKernelSegment>,
    choice: CompactionKernelRegionChoice,
) -> bool {
    match choice {
        CompactionKernelRegionChoice::Chosen(best)
        | CompactionKernelRegionChoice::ScanBeforeChoosing(best) => {
            &&& best < segments.len()
            &&& forall|other: int| 0 <= other < segments.len()
                && segment_is_a_candidate(#[trigger] segments[other]) ==>
                segments[best as int].live_slots < segments[other].live_slots
                || (segments[best as int].live_slots == segments[other].live_slots && best <= other)
        },
        CompactionKernelRegionChoice::NoEligibleSegment => true,
    }
}

// singlefs-core 的 compaction_plan::choose_region 按记录与集合算完每段之后的那一截：候选里活槽数最少的先，同分取起点槽号最小的
// （入参里最前的）；排在最前的那一段扫过就选它，没扫过就要调用方先扫。
pub fn choose_compaction_segment(segments: &[CompactionKernelSegment]) -> (choice:
    CompactionKernelRegionChoice)
    ensures
        region_choice_is_sound(segments@, choice),
        region_choice_is_greedy(segments@, choice),
{
    let mut best: Option<usize> = None;
    let mut best_live_slots: u64 = 0;
    let mut position: usize = 0;
    while position < segments.len()
        invariant
            position <= segments.len(),
            best matches Some(best_position) ==> best_position < position
                && segment_is_a_candidate(segments@[best_position as int]),
            best matches Some(best_position) ==> best_live_slots == segments@[best_position as int].live_slots,
            best matches Some(best_position) ==> forall|earlier: int|
                0 <= earlier < position && segment_is_a_candidate(#[trigger] segments@[earlier]) ==>
                best_live_slots < segments@[earlier].live_slots
                || (best_live_slots == segments@[earlier].live_slots && best_position <= earlier),
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
// 具体状态：映射（key 的身份 → 落点的槽区间，起点加跨度），加上每个槽号上的字节；抽象状态：每个 key 经映射读到的单元
// （区间里每个槽上的字节，按槽序）。目的地的前提按区间不重叠说，不只比起点（判决 m4-exit5-relocation-proof-r1 第二节）；
// 回退候选与读者还在读的区间也进前提（判决 m4-exit5-relocation-proof-r3 第二节 Z1：只看现行映射的区间版前提放得过
// 「目的地压住回退目标两槽单元的第二槽」）。槽号与跨度用 int，接 core 的 u64 时要另证起点加跨度不溢出。
pub open spec fn unit_read_through_the_storage(storage: spec_fn(int) -> Seq<u8>, run: SlotRun) -> Seq<Seq<u8>> {
    Seq::new(run.span as nat, |offset: int| storage(run.start + offset))
}

pub open spec fn bytes_read_through_the_mapping(
    mapping: Map<u64, SlotRun>,
    storage: spec_fn(int) -> Seq<u8>,
    mapping_key_identity: u64,
) -> Seq<Seq<u8>> {
    unit_read_through_the_storage(storage, mapping[mapping_key_identity])
}

// 搬一个单元的写：源区间每个槽上的字节原样写到目的区间对应的槽（判决 G3：头不盖戳、birth 不变），别的槽不动。
pub open spec fn storage_after_copying(
    storage: spec_fn(int) -> Seq<u8>,
    source: SlotRun,
    destination: SlotRun,
) -> spec_fn(int) -> Seq<u8> {
    |slot: int|
        if run_covers(destination, slot) {
            storage(source.start + (slot - destination.start))
        } else {
            storage(slot)
        }
}

// 按次序搬前 count 项：第 j 项把 keys[j] 的映射条目换到 destinations[j]。
pub open spec fn mapping_after_relocating(
    mapping: Map<u64, SlotRun>,
    keys: Seq<u64>,
    destinations: Seq<SlotRun>,
    count: int,
) -> Map<u64, SlotRun>
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

// 按次序搬前 count 项：第 j 项把 sources[j] 区间上的字节写到 destinations[j]。
pub open spec fn storage_after_relocating(
    storage: spec_fn(int) -> Seq<u8>,
    sources: Seq<SlotRun>,
    destinations: Seq<SlotRun>,
    count: int,
) -> spec_fn(int) -> Seq<u8>
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

// 每个 key 的区间至少一槽。
pub open spec fn every_run_of_the_mapping_has_a_slot(mapping: Map<u64, SlotRun>) -> bool {
    forall|mapping_key_identity: u64| #[trigger] mapping.contains_key(mapping_key_identity) ==> mapping[mapping_key_identity].span >= 1
}

// 目的地是分配器的性质（判决 G2）：分配器交的落点区间此刻不与映射里任何一个 key 的区间重叠（「目的地不在 R 内」是它的一个特例：
// R 里能搬的单元都在映射里）。
pub open spec fn destination_is_free_in_the_mapping(mapping: Map<u64, SlotRun>, destination: SlotRun) -> bool {
    forall|mapping_key_identity: u64|
        #[trigger] mapping.contains_key(mapping_key_identity) ==> !runs_overlap(mapping[mapping_key_identity], destination)
}

// 回退候选与读者的每个映射里，每个 key 的区间也至少一槽。
pub open spec fn every_protected_mapping_has_slots(protected_mappings: Seq<Map<u64, SlotRun>>) -> bool {
    forall|protected_index: int| 0 <= protected_index < protected_mappings.len() ==> every_run_of_the_mapping_has_a_slot(#[trigger] protected_mappings[protected_index])
}

// 回退候选与读者还在读的映射（gc_ring_interval_proofs 的保护范围：持久 F 之上的每个根、每个读者），目的地一个区间都不许压住。
pub open spec fn destination_spares_the_protected_mappings(
    protected_mappings: Seq<Map<u64, SlotRun>>,
    destination: SlotRun,
) -> bool {
    forall|protected_index: int|
        0 <= protected_index < protected_mappings.len() ==> destination_is_free_in_the_mapping(#[trigger] protected_mappings[protected_index], destination)
}

// 前 count 个目的区间两两不重叠。
pub open spec fn destinations_are_pairwise_apart(destinations: Seq<SlotRun>, count: int) -> bool {
    forall|earlier: int, later: int|
        #![trigger destinations[earlier], destinations[later]]
        0 <= earlier < later < count ==> !runs_overlap(destinations[earlier], destinations[later])
}

// 两两不重叠落到一对目的区间上；证明里每一处要用它都经这一条，两两不重叠那一条前提被弄坏时红在这里。
proof fn lemma_two_destinations_of_the_batch_are_apart(destinations: Seq<SlotRun>, count: int, earlier: int, later: int)
    requires
        destinations_are_pairwise_apart(destinations, count),
        0 <= earlier < later < count,
    ensures
        !runs_overlap(destinations[earlier], destinations[later]),
{
    assert(!runs_overlap(destinations[earlier], destinations[later]));
}

// 前 count 个目的区间两两不重叠、都不与现行映射的区间重叠、都不压住回退候选与读者的区间。
pub open spec fn destinations_are_distinct_and_free(
    mapping: Map<u64, SlotRun>,
    protected_mappings: Seq<Map<u64, SlotRun>>,
    destinations: Seq<SlotRun>,
    count: int,
) -> bool {
    &&& count <= destinations.len()
    &&& destinations_are_pairwise_apart(destinations, count)
    &&& forall|index: int| 0 <= index < count ==> destination_is_free_in_the_mapping(mapping, #[trigger] destinations[index])
    &&& forall|index: int|
        0 <= index < count ==> destination_spares_the_protected_mappings(protected_mappings, #[trigger] destinations[index])
}

// 目的区间的跨度等于它的源区间（被搬的 key 在映射里的区间）。
pub open spec fn destinations_take_the_spans_of_their_sources(
    mapping: Map<u64, SlotRun>,
    keys: Seq<u64>,
    destinations: Seq<SlotRun>,
    count: int,
) -> bool {
    forall|index: int| 0 <= index < count ==> (#[trigger] destinations[index]).span == mapping[keys[index]].span
}

// 区域扫描的「可搬」：key 在映射里、映射区间的起点就是这一项的源落点。
pub open spec fn scan_agrees_with_the_mapping(
    allocations: Seq<CompactionKernelScannedAllocation>,
    mapping: Map<u64, SlotRun>,
) -> bool {
    forall|position: int|
        0 <= position < allocations.len() && allocation_is_movable(#[trigger] allocations[position])
            ==> mapping.contains_key(mapping_key_identity_of(allocations[position]))
            && mapping[mapping_key_identity_of(allocations[position])].start == allocations[position].source_slot as int
}

pub open spec fn planned_mapping_keys(
    allocations: Seq<CompactionKernelScannedAllocation>,
    positions: Seq<int>,
) -> Seq<u64> {
    positions.map_values(|position: int| mapping_key_identity_of(allocations[position]))
}

// 被搬的每个 key 的源区间：它在映射里的区间。
pub open spec fn source_runs(mapping: Map<u64, SlotRun>, keys: Seq<u64>) -> Seq<SlotRun> {
    keys.map_values(|mapping_key_identity: u64| mapping[mapping_key_identity])
}

// 两个区间逐槽读出来相同，读出的单元就相同。
proof fn lemma_two_runs_read_the_same_unit_when_their_slots_agree(
    this_storage: spec_fn(int) -> Seq<u8>,
    this_run: SlotRun,
    that_storage: spec_fn(int) -> Seq<u8>,
    that_run: SlotRun,
)
    requires
        this_run.span == that_run.span,
        this_run.span >= 0,
        forall|offset: int|
            0 <= offset < this_run.span ==> #[trigger] this_storage(this_run.start + offset) == that_storage(that_run.start + offset),
    ensures
        unit_read_through_the_storage(this_storage, this_run) =~= unit_read_through_the_storage(that_storage, that_run),
{
    assert forall|offset: int| 0 <= offset < unit_read_through_the_storage(this_storage, this_run).len() implies #[trigger] unit_read_through_the_storage(
        this_storage,
        this_run,
    )[offset] == unit_read_through_the_storage(that_storage, that_run)[offset] by {
        assert(this_storage(this_run.start + offset) == that_storage(that_run.start + offset));
    }
}

// 按次序搬前 count 项之后的映射：定义域不变；被搬的 key 指到各自的目的区间；没被搬的 key 原样；每个区间要么是原来的、要么是某个目的区间。
proof fn lemma_mapping_after_relocating_has_the_moved_shape(
    mapping: Map<u64, SlotRun>,
    keys: Seq<u64>,
    destinations: Seq<SlotRun>,
    count: int,
)
    requires
        0 <= count <= keys.len(),
        forall|earlier: int, later: int|
            #![trigger keys[earlier], keys[later]]
            0 <= earlier < later < count ==> keys[earlier] != keys[later],
        forall|index: int| 0 <= index < count ==> mapping.contains_key(#[trigger] keys[index]),
    ensures
        ({
            let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
            &&& moved_mapping.dom() == mapping.dom()
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
        }),
    decreases count,
{
    if count > 0 {
        let previous_count = count - 1;
        lemma_mapping_after_relocating_has_the_moved_shape(mapping, keys, destinations, previous_count);
        let previous_mapping = mapping_after_relocating(mapping, keys, destinations, previous_count);
        let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
        let moved_key = keys[previous_count];
        assert(moved_mapping == previous_mapping.insert(moved_key, destinations[previous_count]));
        assert(mapping.contains_key(moved_key));
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
        assert forall|index: int| 0 <= index < count implies moved_mapping[#[trigger] keys[index]]
            == destinations[index] by {
            if index < previous_count {
                assert(keys[index] != keys[previous_count]);
            }
        }
    }
}

// 按次序搬前 count 项之后，现行映射每个 key 读到的单元不变；没被任何目的区间罩住的槽字节不变。
proof fn lemma_relocating_in_order_keeps_every_read(
    mapping: Map<u64, SlotRun>,
    protected_mappings: Seq<Map<u64, SlotRun>>,
    storage: spec_fn(int) -> Seq<u8>,
    keys: Seq<u64>,
    destinations: Seq<SlotRun>,
    count: int,
)
    requires
        0 <= count <= keys.len(),
        every_run_of_the_mapping_has_a_slot(mapping),
        forall|earlier: int, later: int|
            #![trigger keys[earlier], keys[later]]
            0 <= earlier < later < count ==> keys[earlier] != keys[later],
        forall|index: int| 0 <= index < count ==> mapping.contains_key(#[trigger] keys[index]),
        destinations_are_distinct_and_free(mapping, protected_mappings, destinations, count),
        destinations_take_the_spans_of_their_sources(mapping, keys, destinations, count),
    ensures
        ({
            let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
            let moved_storage = storage_after_relocating(storage, source_runs(mapping, keys), destinations, count);
            &&& forall|mapping_key_identity: u64|
                #[trigger] mapping.contains_key(mapping_key_identity) ==> bytes_read_through_the_mapping(
                    moved_mapping,
                    moved_storage,
                    mapping_key_identity,
                ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity)
            &&& forall|slot: int|
                (forall|index: int| 0 <= index < count ==> !run_covers(destinations[index], slot)) ==> #[trigger] moved_storage(slot)
                    == storage(slot)
        }),
    decreases count,
{
    if count > 0 {
        let previous_count = count - 1;
        let sources = source_runs(mapping, keys);
        lemma_relocating_in_order_keeps_every_read(
            mapping,
            protected_mappings,
            storage,
            keys,
            destinations,
            previous_count,
        );
        lemma_mapping_after_relocating_has_the_moved_shape(mapping, keys, destinations, previous_count);
        let previous_mapping = mapping_after_relocating(mapping, keys, destinations, previous_count);
        let previous_storage = storage_after_relocating(storage, sources, destinations, previous_count);
        let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
        let moved_storage = storage_after_relocating(storage, sources, destinations, count);
        let moved_key = keys[previous_count];
        let source = sources[previous_count];
        let destination = destinations[previous_count];
        assert(moved_mapping == previous_mapping.insert(moved_key, destination));
        assert(moved_storage == storage_after_copying(previous_storage, source, destination));
        assert(mapping.contains_key(moved_key));
        assert(source == mapping[moved_key]);
        assert(destination_is_free_in_the_mapping(mapping, destination));
        assert forall|mapping_key_identity: u64| #[trigger]
            mapping.contains_key(mapping_key_identity) implies bytes_read_through_the_mapping(
            moved_mapping,
            moved_storage,
            mapping_key_identity,
        ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity) by {
            if mapping_key_identity == moved_key {
                assert(moved_mapping[mapping_key_identity] == destination);
                assert forall|offset: int| 0 <= offset < destination.span implies #[trigger] moved_storage(
                    destination.start + offset,
                ) == storage(source.start + offset) by {
                    let source_slot = source.start + offset;
                    assert(run_covers(destination, destination.start + offset));
                    assert(moved_storage(destination.start + offset) == previous_storage(source_slot));
                    assert forall|index: int| 0 <= index < previous_count implies !run_covers(destinations[index], source_slot) by {
                        assert(destination_is_free_in_the_mapping(mapping, destinations[index]));
                        assert(!runs_overlap(mapping[moved_key], destinations[index]));
                    }
                    assert(previous_storage(source_slot) == storage(source_slot));
                }
                lemma_two_runs_read_the_same_unit_when_their_slots_agree(moved_storage, destination, storage, source);
            } else {
                assert(previous_mapping.contains_key(mapping_key_identity));
                let previous_run = previous_mapping[mapping_key_identity];
                assert(moved_mapping[mapping_key_identity] == previous_run);
                if previous_run == mapping[mapping_key_identity] {
                    assert(!runs_overlap(previous_run, destination));
                } else {
                    let index = choose|index: int|
                        0 <= index < previous_count && previous_run == destinations[index];
                    lemma_two_destinations_of_the_batch_are_apart(destinations, count, index, previous_count);
                }
                assert forall|offset: int| 0 <= offset < previous_run.span implies #[trigger] moved_storage(
                    previous_run.start + offset,
                ) == previous_storage(previous_run.start + offset) by {
                    assert(!run_covers(destination, previous_run.start + offset));
                }
                lemma_two_runs_read_the_same_unit_when_their_slots_agree(moved_storage, previous_run, previous_storage, previous_run);
                assert(bytes_read_through_the_mapping(
                    previous_mapping,
                    previous_storage,
                    mapping_key_identity,
                ) == bytes_read_through_the_mapping(mapping, storage, mapping_key_identity));
            }
        }
        assert forall|slot: int|
            (forall|index: int| 0 <= index < count ==> !run_covers(destinations[index], slot)) implies #[trigger] moved_storage(
            slot,
        ) == storage(slot) by {
            assert(!run_covers(destinations[previous_count], slot));
            assert forall|index: int| 0 <= index < previous_count implies !run_covers(destinations[index], slot) by {
                assert(0 <= index < count);
            }
        }
    }
}

// 目的地都不压住回退候选与读者的区间，只改目的地那几槽的写之后，它们每个 key 读到的单元不变。
proof fn lemma_storage_outside_the_destinations_keeps_the_protected_reads(
    protected_mappings: Seq<Map<u64, SlotRun>>,
    storage: spec_fn(int) -> Seq<u8>,
    moved_storage: spec_fn(int) -> Seq<u8>,
    destinations: Seq<SlotRun>,
    count: int,
)
    requires
        0 <= count <= destinations.len(),
        every_protected_mapping_has_slots(protected_mappings),
        forall|index: int|
            0 <= index < count ==> destination_spares_the_protected_mappings(protected_mappings, #[trigger] destinations[index]),
        forall|slot: int|
            (forall|index: int| 0 <= index < count ==> !run_covers(destinations[index], slot)) ==> #[trigger] moved_storage(slot)
                == storage(slot),
    ensures
        forall|protected_index: int, mapping_key_identity: u64|
            0 <= protected_index < protected_mappings.len() && #[trigger] protected_mappings[protected_index].contains_key(mapping_key_identity)
                ==> bytes_read_through_the_mapping(protected_mappings[protected_index], moved_storage, mapping_key_identity)
                == bytes_read_through_the_mapping(protected_mappings[protected_index], storage, mapping_key_identity),
{
    assert forall|protected_index: int, mapping_key_identity: u64|
        0 <= protected_index < protected_mappings.len() && #[trigger] protected_mappings[protected_index].contains_key(mapping_key_identity) implies bytes_read_through_the_mapping(
        protected_mappings[protected_index],
        moved_storage,
        mapping_key_identity,
    ) == bytes_read_through_the_mapping(protected_mappings[protected_index], storage, mapping_key_identity) by {
        let run = protected_mappings[protected_index][mapping_key_identity];
        assert(every_run_of_the_mapping_has_a_slot(protected_mappings[protected_index]));
        assert forall|offset: int| 0 <= offset < run.span implies #[trigger] moved_storage(run.start + offset) == storage(
            run.start + offset,
        ) by {
            assert forall|index: int| 0 <= index < count implies !run_covers(destinations[index], run.start + offset) by {
                assert(destination_spares_the_protected_mappings(protected_mappings, destinations[index]));
                assert(destination_is_free_in_the_mapping(protected_mappings[protected_index], destinations[index]));
                assert(!runs_overlap(protected_mappings[protected_index][mapping_key_identity], destinations[index]));
            }
        }
        lemma_two_runs_read_the_same_unit_when_their_slots_agree(moved_storage, run, storage, run);
    }
}

// P4，一批：计划合 P2（key 互异来自扫描结果里能搬的 key 互异）、扫描的「可搬」与映射对得上、目的区间两两不重叠、不与现行映射的区间重叠、
// 不压住回退候选与读者的区间、跨度等于源区间（分配器的性质），按计划搬完这一批，现行映射每个 key 读到的单元不变、映射里的 key 一个不多一个不少，
// 回退候选与读者每个 key 读到的单元也不变。
pub proof fn lemma_a_planned_batch_keeps_every_read(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: usize,
    planned: CompactionKernelPlannedBatch,
    mapping: Map<u64, SlotRun>,
    protected_mappings: Seq<Map<u64, SlotRun>>,
    storage: spec_fn(int) -> Seq<u8>,
    destinations: Seq<SlotRun>,
)
    requires
        planned_batch_is_sound(allocations, batch_units, planned),
        movable_mapping_keys_are_distinct(allocations),
        scan_agrees_with_the_mapping(allocations, mapping),
        every_run_of_the_mapping_has_a_slot(mapping),
        every_protected_mapping_has_slots(protected_mappings),
        destinations_are_distinct_and_free(
            mapping,
            protected_mappings,
            destinations,
            planned_positions(planned).len() as int,
        ),
        destinations_take_the_spans_of_their_sources(
            mapping,
            planned_mapping_keys(allocations, planned_positions(planned)),
            destinations,
            planned_positions(planned).len() as int,
        ),
    ensures
        ({
            let positions = planned_positions(planned);
            let keys = planned_mapping_keys(allocations, positions);
            let moved_mapping = mapping_after_relocating(mapping, keys, destinations, positions.len() as int);
            let moved_storage = storage_after_relocating(
                storage,
                source_runs(mapping, keys),
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
            &&& forall|protected_index: int, mapping_key_identity: u64|
                0 <= protected_index < protected_mappings.len() && #[trigger] protected_mappings[protected_index].contains_key(mapping_key_identity)
                    ==> bytes_read_through_the_mapping(protected_mappings[protected_index], moved_storage, mapping_key_identity)
                    == bytes_read_through_the_mapping(protected_mappings[protected_index], storage, mapping_key_identity)
        }),
{
    let positions = planned_positions(planned);
    let keys = planned_mapping_keys(allocations, positions);
    assert forall|earlier: int, later: int|
        #![trigger keys[earlier], keys[later]]
        0 <= earlier < later < positions.len() implies keys[earlier] != keys[later] by {
        assert(positions[earlier] < positions[later]);
        assert(allocation_is_movable(allocations[positions[earlier]]));
        assert(allocation_is_movable(allocations[positions[later]]));
    }
    assert forall|index: int| 0 <= index < positions.len() implies mapping.contains_key(#[trigger] keys[index]) by {
        assert(0 <= positions[index] < allocations.len());
        assert(allocation_is_movable(allocations[positions[index]]));
    }
    lemma_relocating_in_order_keeps_every_read(
        mapping,
        protected_mappings,
        storage,
        keys,
        destinations,
        positions.len() as int,
    );
    lemma_mapping_after_relocating_has_the_moved_shape(mapping, keys, destinations, positions.len() as int);
    lemma_storage_outside_the_destinations_keeps_the_protected_reads(
        protected_mappings,
        storage,
        storage_after_relocating(storage, source_runs(mapping, keys), destinations, positions.len() as int),
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

// 一批批搬到 R 空：每一批搬剩下的最前 min(K, 剩下几项) 项（前提下计划就交这几项，lemma_under_the_premise_a_plan_takes_the_leading_allocations），
// 目的地按次序从 destinations 里取；这一批发布之后它们离开 R，下一次扫描就是剩下的那几项（singlefs-core 的 compaction_plan::plan_batch 文档）。
pub open spec fn mapping_after_compacting_the_region(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: int,
    mapping: Map<u64, SlotRun>,
    destinations: Seq<SlotRun>,
) -> Map<u64, SlotRun>
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

// 源区间取这一批开搬时映射里的区间，所以按批推进时带着映射走。
pub open spec fn storage_after_compacting_the_region(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: int,
    mapping: Map<u64, SlotRun>,
    storage: spec_fn(int) -> Seq<u8>,
    destinations: Seq<SlotRun>,
) -> spec_fn(int) -> Seq<u8>
    decreases allocations.len(),
{
    if allocations.len() == 0 || batch_units <= 0 {
        storage
    } else {
        let count = units_in_the_batch(allocations.len() as int, batch_units);
        let keys = mapping_key_identities(allocations);
        storage_after_compacting_the_region(
            allocations.skip(count),
            batch_units,
            mapping_after_relocating(mapping, keys, destinations, count),
            storage_after_relocating(storage, source_runs(mapping, keys), destinations, count),
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

// 搬完一批之后剩下的那几项照样合前提：都能搬、key 互异、扫描与搬后的映射对得上（剩下的 key 这一批没搬）、搬后映射的每个区间至少一槽、
// 剩下的目的区间两两不重叠、不与搬后映射的区间重叠（搬后的区间要么是原来的、要么是这一批的目的区间）、不压住回退候选与读者、跨度对得上。
proof fn lemma_after_a_batch_the_remaining_region_meets_the_premise(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: int,
    mapping: Map<u64, SlotRun>,
    protected_mappings: Seq<Map<u64, SlotRun>>,
    destinations: Seq<SlotRun>,
)
    requires
        batch_units >= 1,
        allocations.len() > 0,
        every_allocation_in_the_region_is_movable(allocations),
        movable_mapping_keys_are_distinct(allocations),
        scan_agrees_with_the_mapping(allocations, mapping),
        every_run_of_the_mapping_has_a_slot(mapping),
        destinations_are_distinct_and_free(mapping, protected_mappings, destinations, allocations.len() as int),
        destinations_take_the_spans_of_their_sources(
            mapping,
            mapping_key_identities(allocations),
            destinations,
            allocations.len() as int,
        ),
    ensures
        ({
            let count = units_in_the_batch(allocations.len() as int, batch_units);
            let moved_mapping = mapping_after_relocating(mapping, mapping_key_identities(allocations), destinations, count);
            let remaining = allocations.skip(count);
            let remaining_destinations = destinations.skip(count);
            &&& every_allocation_in_the_region_is_movable(remaining)
            &&& movable_mapping_keys_are_distinct(remaining)
            &&& scan_agrees_with_the_mapping(remaining, moved_mapping)
            &&& every_run_of_the_mapping_has_a_slot(moved_mapping)
            &&& destinations_are_distinct_and_free(moved_mapping, protected_mappings, remaining_destinations, remaining.len() as int)
            &&& destinations_take_the_spans_of_their_sources(
                moved_mapping,
                mapping_key_identities(remaining),
                remaining_destinations,
                remaining.len() as int,
            )
        }),
{
    let count = units_in_the_batch(allocations.len() as int, batch_units);
    let keys = mapping_key_identities(allocations);
    assert forall|earlier: int, later: int|
        #![trigger keys[earlier], keys[later]]
        0 <= earlier < later < count implies keys[earlier] != keys[later] by {
        assert(allocation_is_movable(allocations[earlier]));
        assert(allocation_is_movable(allocations[later]));
    }
    assert forall|index: int| 0 <= index < count implies mapping.contains_key(#[trigger] keys[index]) by {
        assert(allocation_is_movable(allocations[index]));
    }
    lemma_mapping_after_relocating_has_the_moved_shape(mapping, keys, destinations, count);
    let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
    let remaining = allocations.skip(count);
    let remaining_destinations = destinations.skip(count);
    let remaining_keys = mapping_key_identities(remaining);
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
    // 剩下的 key 这一批没搬，它们在搬后的映射里还是原来的区间。
    assert forall|position: int| 0 <= position < remaining.len() implies moved_mapping.contains_key(
        #[trigger] remaining_keys[position],
    ) && moved_mapping[remaining_keys[position]] == mapping[keys[position + count]] by {
        let original = position + count;
        assert(remaining[position] == allocations[original]);
        assert(remaining_keys[position] == keys[original]);
        assert(allocation_is_movable(allocations[original]));
        let key = keys[original];
        assert(mapping.contains_key(key));
        assert forall|index: int| 0 <= index < count implies keys[index] != key by {
            assert(allocation_is_movable(allocations[index]));
        }
        assert(moved_mapping[key] == mapping[key]);
    }
    assert forall|position: int|
        0 <= position < remaining.len() && allocation_is_movable(
            #[trigger] remaining[position],
        ) implies moved_mapping.contains_key(mapping_key_identity_of(remaining[position]))
        && moved_mapping[mapping_key_identity_of(remaining[position])].start == remaining[position].source_slot as int by {
        let original = position + count;
        assert(remaining[position] == allocations[original]);
        assert(remaining_keys[position] == mapping_key_identity_of(remaining[position]));
        assert(allocation_is_movable(allocations[original]));
    }
    assert forall|mapping_key_identity: u64| #[trigger] moved_mapping.contains_key(mapping_key_identity) implies moved_mapping[mapping_key_identity].span
        >= 1 by {
        if moved_mapping[mapping_key_identity] != mapping[mapping_key_identity] {
            let moved_index = choose|moved_index: int|
                0 <= moved_index < count && moved_mapping[mapping_key_identity] == destinations[moved_index];
            assert(destinations[moved_index].span == mapping[keys[moved_index]].span);
        }
    }
    assert forall|earlier: int, later: int|
        #![trigger remaining_destinations[earlier], remaining_destinations[later]]
        0 <= earlier < later < remaining.len() implies !runs_overlap(
        remaining_destinations[earlier],
        remaining_destinations[later],
    ) by {
        assert(remaining_destinations[earlier] == destinations[earlier + count]);
        assert(remaining_destinations[later] == destinations[later + count]);
        lemma_two_destinations_of_the_batch_are_apart(destinations, allocations.len() as int, earlier + count, later + count);
    }
    assert forall|index: int| 0 <= index < remaining.len() implies destination_is_free_in_the_mapping(
        moved_mapping,
        #[trigger] remaining_destinations[index],
    ) && destination_spares_the_protected_mappings(protected_mappings, remaining_destinations[index]) by {
        let original = index + count;
        assert(remaining_destinations[index] == destinations[original]);
        assert(destination_is_free_in_the_mapping(mapping, destinations[original]));
        assert(destination_spares_the_protected_mappings(protected_mappings, destinations[original]));
        assert forall|mapping_key_identity: u64| #[trigger]
            moved_mapping.contains_key(mapping_key_identity) implies !runs_overlap(
            moved_mapping[mapping_key_identity],
            destinations[original],
        ) by {
            if moved_mapping[mapping_key_identity] != mapping[mapping_key_identity] {
                let moved_index = choose|moved_index: int|
                    0 <= moved_index < count && moved_mapping[mapping_key_identity]
                        == destinations[moved_index];
                lemma_two_destinations_of_the_batch_are_apart(destinations, allocations.len() as int, moved_index, original);
            }
        }
    }
    assert forall|index: int| 0 <= index < remaining.len() implies (#[trigger] remaining_destinations[index]).span
        == moved_mapping[remaining_keys[index]].span by {
        assert(remaining_destinations[index] == destinations[index + count]);
        assert(destinations[index + count].span == mapping[keys[index + count]].span);
    }
}

// P4：前提下按计划一批批搬到 R 空，现行映射每个 key 读到的单元不变、映射里的 key 一个不多一个不少，回退候选与读者每个 key 读到的单元也不变。
pub proof fn corollary_compacting_the_region_until_it_is_empty_keeps_every_read(
    allocations: Seq<CompactionKernelScannedAllocation>,
    batch_units: int,
    mapping: Map<u64, SlotRun>,
    protected_mappings: Seq<Map<u64, SlotRun>>,
    storage: spec_fn(int) -> Seq<u8>,
    destinations: Seq<SlotRun>,
)
    requires
        batch_units >= 1,
        every_allocation_in_the_region_is_movable(allocations),
        movable_mapping_keys_are_distinct(allocations),
        scan_agrees_with_the_mapping(allocations, mapping),
        every_run_of_the_mapping_has_a_slot(mapping),
        every_protected_mapping_has_slots(protected_mappings),
        destinations_are_distinct_and_free(mapping, protected_mappings, destinations, allocations.len() as int),
        destinations_take_the_spans_of_their_sources(
            mapping,
            mapping_key_identities(allocations),
            destinations,
            allocations.len() as int,
        ),
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
                mapping,
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
            &&& forall|protected_index: int, mapping_key_identity: u64|
                0 <= protected_index < protected_mappings.len() && #[trigger] protected_mappings[protected_index].contains_key(mapping_key_identity)
                    ==> bytes_read_through_the_mapping(protected_mappings[protected_index], moved_storage, mapping_key_identity)
                    == bytes_read_through_the_mapping(protected_mappings[protected_index], storage, mapping_key_identity)
        }),
    decreases allocations.len(),
{
    if allocations.len() > 0 {
        let count = units_in_the_batch(allocations.len() as int, batch_units);
        let keys = mapping_key_identities(allocations);
        assert forall|earlier: int, later: int|
            #![trigger keys[earlier], keys[later]]
            0 <= earlier < later < count implies keys[earlier] != keys[later] by {
            assert(allocation_is_movable(allocations[earlier]));
            assert(allocation_is_movable(allocations[later]));
        }
        assert forall|index: int| 0 <= index < count implies mapping.contains_key(#[trigger] keys[index]) by {
            assert(allocation_is_movable(allocations[index]));
        }
        lemma_relocating_in_order_keeps_every_read(mapping, protected_mappings, storage, keys, destinations, count);
        lemma_mapping_after_relocating_has_the_moved_shape(mapping, keys, destinations, count);
        let moved_mapping = mapping_after_relocating(mapping, keys, destinations, count);
        let moved_storage = storage_after_relocating(storage, source_runs(mapping, keys), destinations, count);
        lemma_storage_outside_the_destinations_keeps_the_protected_reads(
            protected_mappings,
            storage,
            moved_storage,
            destinations,
            count,
        );
        lemma_after_a_batch_the_remaining_region_meets_the_premise(
            allocations,
            batch_units,
            mapping,
            protected_mappings,
            destinations,
        );
        corollary_compacting_the_region_until_it_is_empty_keeps_every_read(
            allocations.skip(count),
            batch_units,
            moved_mapping,
            protected_mappings,
            moved_storage,
            destinations.skip(count),
        );
    }
}

// 第三轮攻方 A2（Z1 变体一）：现行映射只有 key 7 在 [10, 11)，回退目标里 key 4 是 [20, 22) 两槽单元，把 key 7 搬到 [21, 22)。
// 只看现行映射的区间前提放得过；回退候选进了前提，这一批的目的地就不合格。
pub proof fn lemma_a_destination_on_the_second_slot_of_a_rollback_target_unit_breaks_the_premise()
    ensures
        ({
            let mapping = Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 });
            let rollback_target = Map::<u64, SlotRun>::empty().insert(4u64, SlotRun { start: 20, span: 2 });
            let destinations = seq![SlotRun { start: 21, span: 1 }];
            &&& destinations_are_distinct_and_free(mapping, Seq::empty(), destinations, 1)
            &&& !destinations_are_distinct_and_free(mapping, seq![rollback_target], destinations, 1)
        }),
{
    let mapping = Map::<u64, SlotRun>::empty().insert(7u64, SlotRun { start: 10, span: 1 });
    let rollback_target = Map::<u64, SlotRun>::empty().insert(4u64, SlotRun { start: 20, span: 2 });
    let destinations = seq![SlotRun { start: 21, span: 1 }];
    assert(destination_is_free_in_the_mapping(mapping, destinations[0])) by {
        assert forall|mapping_key_identity: u64| #[trigger] mapping.contains_key(mapping_key_identity) implies !runs_overlap(
            mapping[mapping_key_identity],
            destinations[0],
        ) by {
            assert(mapping_key_identity == 7u64);
        }
    }
    let protected_mappings = seq![rollback_target];
    assert(protected_mappings[0].contains_key(4u64));
    assert(runs_overlap(protected_mappings[0][4u64], destinations[0]));
    assert(!destination_is_free_in_the_mapping(protected_mappings[0], destinations[0]));
    assert(!destination_spares_the_protected_mappings(protected_mappings, destinations[0]));
}

// 第一轮攻方的两槽目的地（只比起点那一版放得过）：映射 key 5 在 [3, 4)，key 9 在 [6, 8)，目的区间 [2, 4) 压住了槽 3。
pub proof fn lemma_a_two_slot_destination_over_a_one_slot_unit_breaks_the_premise()
    ensures
        ({
            let mapping = Map::<u64, SlotRun>::empty().insert(5u64, SlotRun { start: 3, span: 1 }).insert(
                9u64,
                SlotRun { start: 6, span: 2 },
            );
            !destinations_are_distinct_and_free(mapping, Seq::empty(), seq![SlotRun { start: 2, span: 2 }], 1)
        }),
{
    let mapping = Map::<u64, SlotRun>::empty().insert(5u64, SlotRun { start: 3, span: 1 }).insert(
        9u64,
        SlotRun { start: 6, span: 2 },
    );
    let destinations = seq![SlotRun { start: 2, span: 2 }];
    assert(mapping.contains_key(5u64));
    assert(mapping[5u64] == (SlotRun { start: 3, span: 1 }));
    assert(runs_overlap(mapping[5u64], destinations[0]));
}

// 无前提的见证（防空真）：两个单元（一槽、两槽）、一个回退目标拿着 [20, 22)，目的区间合格、跨度对得上。
pub proof fn witness_a_two_unit_batch_beside_a_rollback_target_satisfies_the_premise()
    ensures
        ({
            let mapping = Map::<u64, SlotRun>::empty().insert(5u64, SlotRun { start: 3, span: 1 }).insert(
                9u64,
                SlotRun { start: 6, span: 2 },
            );
            let protected_mappings = seq![Map::<u64, SlotRun>::empty().insert(4u64, SlotRun { start: 20, span: 2 })];
            let keys = seq![9u64, 5u64];
            let destinations = seq![SlotRun { start: 10, span: 2 }, SlotRun { start: 12, span: 1 }];
            &&& every_run_of_the_mapping_has_a_slot(mapping)
            &&& destinations_are_distinct_and_free(mapping, protected_mappings, destinations, 2)
            &&& destinations_take_the_spans_of_their_sources(mapping, keys, destinations, 2)
        }),
{
    let mapping = Map::<u64, SlotRun>::empty().insert(5u64, SlotRun { start: 3, span: 1 }).insert(
        9u64,
        SlotRun { start: 6, span: 2 },
    );
    let rollback_target = Map::<u64, SlotRun>::empty().insert(4u64, SlotRun { start: 20, span: 2 });
    let protected_mappings = seq![rollback_target];
    let keys = seq![9u64, 5u64];
    let destinations = seq![SlotRun { start: 10, span: 2 }, SlotRun { start: 12, span: 1 }];
    assert forall|mapping_key_identity: u64| #[trigger] mapping.contains_key(mapping_key_identity) implies mapping[mapping_key_identity].span
        >= 1 by {
        assert(mapping_key_identity == 5u64 || mapping_key_identity == 9u64);
    }
    assert forall|index: int| 0 <= index < 2 implies destination_is_free_in_the_mapping(mapping, #[trigger] destinations[index])
        && destination_spares_the_protected_mappings(protected_mappings, destinations[index]) by {
        assert forall|mapping_key_identity: u64| #[trigger] mapping.contains_key(mapping_key_identity) implies !runs_overlap(
            mapping[mapping_key_identity],
            destinations[index],
        ) by {
            assert(mapping_key_identity == 5u64 || mapping_key_identity == 9u64);
        }
        assert forall|protected_index: int| 0 <= protected_index < protected_mappings.len() implies destination_is_free_in_the_mapping(
            #[trigger] protected_mappings[protected_index],
            destinations[index],
        ) by {
            assert(protected_index == 0);
            assert forall|mapping_key_identity: u64| #[trigger] rollback_target.contains_key(mapping_key_identity) implies !runs_overlap(
                rollback_target[mapping_key_identity],
                destinations[index],
            ) by {
                assert(mapping_key_identity == 4u64);
            }
        }
    }
    assert(mapping[keys[0]] == (SlotRun { start: 6, span: 2 }));
    assert(mapping[keys[1]] == (SlotRun { start: 3, span: 1 }));
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

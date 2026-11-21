//! 两份 GPU 活槽打分算法的精确等价证明：二分边界、候选裁剪、逐槽计数与排列。
//! 这里验证算法模型；WGSL 到此模型的翻译、主机排序/打包与 GPU 执行仍是未证边界。
#[allow(
    unused_imports,
    reason = "只有 Verus 编译时留下的证明代码用这些名字；普通构建擦掉证明代码之后没人用，擦掉的 spec fn 也不在，所以整模块导入"
)]
use crate::gpu_gc_proofs::*;
use vstd::prelude::*;

verus! {

pub open spec fn record_key_before(record: ScoringRecord, device: int, slot: int) -> bool {
    record.device < device || (record.device == device && record.start < slot)
}

pub open spec fn records_are_sorted(records: Seq<ScoringRecord>) -> bool {
    forall|earlier: int, later: int| 0 <= earlier < later < records.len() ==>
        records[earlier].device < records[later].device
        || (records[earlier].device == records[later].device
            && records[earlier].start <= records[later].start)
}

pub open spec fn is_lower_bound(records: Seq<ScoringRecord>, device: int, slot: int, boundary: int) -> bool {
    0 <= boundary <= records.len()
        && (forall|position: int| 0 <= position < boundary ==>
            #[trigger] record_key_before(records[position], device, slot))
        && (forall|position: int| boundary <= position < records.len() ==>
            !#[trigger] record_key_before(records[position], device, slot))
}

/// 对 WGSL 二分循环的 (below, above) 状态归纳；不把“返回下界”作为假设。
pub proof fn lower_bound_of_sorted_records(records: Seq<ScoringRecord>, device: int, slot: int, below: int, above: int) -> (boundary: int)
    requires
        records_are_sorted(records),
        0 <= below <= above <= records.len() <= u32::MAX,
        forall|position: int| 0 <= position < below ==>
            #[trigger] record_key_before(records[position], device, slot),
        forall|position: int| above <= position < records.len() ==>
            !#[trigger] record_key_before(records[position], device, slot),
    ensures is_lower_bound(records, device, slot, boundary),
    decreases above - below,
{
    if below < above {
        let middle = below + (above - below) / 2;
        assert(below <= middle < above);
        // WGSL 里 below、above、middle 都是 u32，下一轮的 below 取 middle + 1u：requires 里记录数不超过 u32::MAX 这条界让它不回绕。
        assert(middle + 1 <= u32::MAX);
        if record_key_before(records[middle], device, slot) {
            assert forall|position: int| 0 <= position < middle + 1 implies
                #[trigger] record_key_before(records[position], device, slot) by {
                if position < middle {
                    assert(records[position].device < records[middle].device
                        || (records[position].device == records[middle].device
                            && records[position].start <= records[middle].start));
                }
            }
            lower_bound_of_sorted_records(records, device, slot, middle + 1, above)
        } else {
            assert forall|position: int| middle <= position < records.len() implies
                !#[trigger] record_key_before(records[position], device, slot) by {
                if middle < position {
                    assert(records[middle].device < records[position].device
                        || (records[middle].device == records[position].device
                            && records[middle].start <= records[position].start));
                }
            }
            lower_bound_of_sorted_records(records, device, slot, below, middle)
        }
    } else { below }
}

/// 逐槽核的内循环，按实际访问的每个槽定义；规格不借用区间交长度。
pub open spec fn slots_counted_one_by_one(first: int, span: nat, start: int, end: int) -> int
    decreases span,
{
    if span == 0 { 0 }
    else {
        slots_counted_one_by_one(first, (span - 1) as nat, start, end)
            + if start <= first + span - 1 < end { 1int } else { 0int }
    }
}

pub proof fn lemma_counting_slots_equals_interval_intersection(first: int, span: nat, start: int, end: int)
    requires start <= end,
    ensures slots_counted_one_by_one(first, span, start, end)
        == overlap_slots(first, first + span, start, end),
    decreases span,
{
    if span > 0 {
        lemma_counting_slots_equals_interval_intersection(first, (span - 1) as nat, start, end);
    }
}

/// 窗口下沿往前退 65535 个槽，比证明要的多一格：跨度不超过 65535 的记录，起点早于 start − 65534 的尾端不超过 start、贡献为零，
/// 所以退 65534 个已不漏，退 65533 个才漏（必红表 gpu-score-window-too-narrow）。
pub open spec fn earliest_candidate_start(start: int) -> int {
    if start >= 65535 { start - 65535 } else { 0 }
}

pub open spec fn valid_scoring_records(records: Seq<ScoringRecord>) -> bool {
    forall|position: int| 0 <= position < records.len() ==>
        0 <= records[position].start <= records[position].end <= u64::MAX
        && records[position].end - records[position].start <= 65535
}

/// 候选窗内不混入异盘；窗外记录无贡献，包括窗前跨段记录的最大跨度边界。
pub proof fn lemma_candidate_window_keeps_every_contribution(
    records: Seq<ScoringRecord>, device: int, start: int, end: int,
    first: int, past: int, position: int,
)
    requires
        valid_scoring_records(records), 0 <= start < end,
        is_lower_bound(records, device, earliest_candidate_start(start), first),
        is_lower_bound(records, device, end, past),
        0 <= position < records.len(),
    ensures
        first <= past,
        first <= position < past ==> records[position].device == device,
        !(first <= position < past) ==> record_contribution(records[position], device, start, end) == 0,
{
    if past < first {
        assert(record_key_before(records[past], device, earliest_candidate_start(start)));
        assert(!record_key_before(records[past], device, end));
    }
    let record = records[position];
    assert(0 <= record.start <= record.end <= u64::MAX);
    assert(record.end - record.start <= 65535);
    if position < first {
        assert(record_key_before(record, device, earliest_candidate_start(start)));
    } else {
        assert(!record_key_before(record, device, earliest_candidate_start(start)));
    }
    if position < past {
        assert(record_key_before(record, device, end));
    } else {
        assert(!record_key_before(record, device, end));
    }
}

/// 外循环只累加二分出的窗口，已释放记录跳过；重复记录不会被去重。
pub open spec fn searched_slot_sum(records: Seq<ScoringRecord>, start: int, end: int, first: int, past: int) -> int
    decreases past - first,
{
    if past <= first { 0 }
    else {
        let record = records[past - 1];
        searched_slot_sum(records, start, end, first, past - 1)
            + if record.released { 0int } else {
                slots_counted_one_by_one(record.start, (record.end - record.start) as nat, start, end)
            }
    }
}

pub proof fn lemma_search_prefix_matches_the_walk(
    records: Seq<ScoringRecord>, device: int, start: int, end: int,
    first: int, past: int, visited: int,
)
    requires
        valid_scoring_records(records), 0 <= start < end,
        is_lower_bound(records, device, earliest_candidate_start(start), first),
        is_lower_bound(records, device, end, past),
        first <= past, 0 <= visited <= records.len(),
    ensures
        record_slice_sum(records, device, start, end, 0, visited)
            == searched_slot_sum(records, start, end, first, if visited < past { visited } else { past }),
    decreases visited,
{
    if visited > 0 {
        lemma_search_prefix_matches_the_walk(records, device, start, end, first, past, visited - 1);
        lemma_candidate_window_keeps_every_contribution(records, device, start, end, first, past, visited - 1);
        if first < visited <= past {
            let record = records[visited - 1];
            lemma_counting_slots_equals_interval_intersection(record.start, (record.end - record.start) as nat, start, end);
        }
    }
}

pub proof fn lemma_record_sum_is_a_fold(records: Seq<ScoringRecord>, device: int, start: int, end: int, visited: int)
    requires 0 <= visited <= records.len(),
    ensures record_slice_sum(records, device, start, end, 0, visited)
        == records.take(visited).fold_left(0int, |total: int, record: ScoringRecord|
            total + record_contribution(record, device, start, end)),
    decreases visited,
{
    if visited > 0 {
        lemma_record_sum_is_a_fold(records, device, start, end, visited - 1);
        assert(records.take(visited).drop_last() =~= records.take(visited - 1));
    }
}

/// 排序只要求多重集合相同；允许重复、重叠记录，不能把排列假设弱化成集合相同。
pub proof fn lemma_permuting_records_preserves_the_walk(original: Seq<ScoringRecord>, sorted: Seq<ScoringRecord>, device: int, start: int, end: int)
    requires original.to_multiset() == sorted.to_multiset(),
    ensures record_slice_sum(original, device, start, end, 0, original.len() as int)
        == record_slice_sum(sorted, device, start, end, 0, sorted.len() as int),
{
    let add_contribution = |total: int, record: ScoringRecord| total + record_contribution(record, device, start, end);
    vstd::seq_lib::lemma_fold_left_permutation(original, sorted, add_contribution, 0int);
    lemma_record_sum_is_a_fold(original, device, start, end, original.len() as int);
    lemma_record_sum_is_a_fold(sorted, device, start, end, sorted.len() as int);
    assert(original.take(original.len() as int) =~= original);
    assert(sorted.take(sorted.len() as int) =~= sorted);
}

/// 任意有限表：实际二分控制流的两端之间逐槽计数，等于原始无序表逐记录求交。
pub proof fn theorem_sorted_search_equals_full_record_walk(
    original: Seq<ScoringRecord>, sorted: Seq<ScoringRecord>, device: int, start: int, end: int,
) -> (window: (int, int))
    requires
        original.to_multiset() == sorted.to_multiset(),
        records_are_sorted(sorted), valid_scoring_records(sorted),
        sorted.len() <= u32::MAX, 0 <= start < end <= u64::MAX,
    ensures
        is_lower_bound(sorted, device, earliest_candidate_start(start), window.0),
        is_lower_bound(sorted, device, end, window.1),
        window.0 <= window.1,
        searched_slot_sum(sorted, start, end, window.0, window.1)
            == record_slice_sum(original, device, start, end, 0, original.len() as int),
{
    let first = lower_bound_of_sorted_records(sorted, device, earliest_candidate_start(start), 0, sorted.len() as int);
    let past = lower_bound_of_sorted_records(sorted, device, end, 0, sorted.len() as int);
    if past < first {
        lemma_candidate_window_keeps_every_contribution(sorted, device, start, end, first, past, past);
    }
    lemma_search_prefix_matches_the_walk(sorted, device, start, end, first, past, sorted.len() as int);
    lemma_permuting_records_preserves_the_walk(original, sorted, device, start, end);
    (first, past)
}

/// 每个前缀都装得进 u64；允许重数，所以界限是记录数乘跨度，不是段长。
pub proof fn lemma_every_walk_prefix_fits_the_gpu_counter(records: Seq<ScoringRecord>, device: int, start: int, end: int, visited: int)
    requires valid_scoring_records(records), start <= end, 0 <= visited <= records.len() <= u32::MAX,
    ensures 0 <= record_slice_sum(records, device, start, end, 0, visited) <= visited * 65535 < u64::MAX,
    decreases visited,
{
    if visited > 0 {
        lemma_every_walk_prefix_fits_the_gpu_counter(records, device, start, end, visited - 1);
        let record = records[visited - 1];
        crate::gpu_gc_proofs::lemma_live_slot_intersection_bounds(record.start, record.end, start, end);
    }
}

/// 非空实例含异盘、释放、重复及从段前伸入的记录；另覆盖空表与饱和下界。
pub proof fn witness_scoring_search_with_duplicates_and_crossing_records() {
    broadcast use vstd::seq_lib::group_to_multiset_ensures;
    let crossing = ScoringRecord { device: 7, released: false, start: 9, end: 12 };
    let foreign = ScoringRecord { device: 6, released: false, start: 10, end: 11 };
    let released = ScoringRecord { device: 7, released: true, start: 10, end: 14 };
    let sorted = seq![foreign, crossing, crossing, released];
    let original = seq![crossing, foreign, released, crossing];
    // vstd 的 Seq::to_multiset 是 closed，本模块看不见它的定义；多于一个元素的 seq! 展开成数组视图，
    // group_to_multiset_ensures 里的 to_multiset_build 只认 s.push(a).to_multiset()。
    // 所以先把两张表写成从空表逐个 push 的形态，再逐步换成多重集合的 insert，两边的计数按 add、singleton 的公理逐值相等。
    let nothing_pushed = Seq::<ScoringRecord>::empty();
    assert(sorted =~= nothing_pushed.push(foreign).push(crossing).push(crossing).push(released));
    assert(original =~= nothing_pushed.push(crossing).push(foreign).push(released).push(crossing));
    assert(sorted.to_multiset() =~= nothing_pushed.to_multiset()
        .insert(foreign).insert(crossing).insert(crossing).insert(released));
    assert(original.to_multiset() =~= nothing_pushed.to_multiset()
        .insert(crossing).insert(foreign).insert(released).insert(crossing));
    assert(original.to_multiset() =~= sorted.to_multiset());
    assert(records_are_sorted(sorted));
    assert(valid_scoring_records(sorted));
    let window = theorem_sorted_search_equals_full_record_walk(original, sorted, 7, 10, 20);
    lemma_every_walk_prefix_fits_the_gpu_counter(sorted, 7, 10, 20, 4);
    reveal_with_fuel(record_slice_sum, 5);
    assert(searched_slot_sum(sorted, 10, 20, window.0, window.1) == 4);
    let empty = Seq::<ScoringRecord>::empty();
    theorem_sorted_search_equals_full_record_walk(empty, empty, 7, 0, 1);
    let longest_crossing = seq![ScoringRecord { device: 7, released: false, start: 1, end: 65536 }];
    let crossing_window = theorem_sorted_search_equals_full_record_walk(longest_crossing, longest_crossing, 7, 65535, 65537);
    assert(searched_slot_sum(longest_crossing, 65535, 65537, crossing_window.0, crossing_window.1) == 1);
}

} // verus!

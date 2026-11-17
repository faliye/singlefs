//! GPU GC 的主机端纯核与着色器算术模型。20 项核心性质见 src/README.md。
//! 仲裁、预算、分片、回读与区域准入的 exec 函数由产品直接调用；
//! Map/Seq 与双字算术证明不等于验证 WGSL 编译器、GPU 执行或设备 I/O。
use crate::file_range_proofs::units_for_sequential_write;
use vstd::prelude::*;

verus! {

pub const GPU_UNCERTAIN_ROUNDS_BEFORE_CPU: u32 = 2;
pub const GPU_INPUT_ALIGNMENT: u64 = 4;
pub const GPU_LIVE_SLOTS_COUNTED: u32 = 1;

pub struct ReconciliationStep {
    pub confirmed: bool,
    pub ask_cpu: bool,
    pub uncertain_rounds: u32,
}

pub open spec fn next_uncertain_count(previous: nat, agree: bool) -> nat {
    if agree || previous >= 1 { 0 } else { 1 }
}

pub open spec fn cpu_is_needed(previous: nat, agree: bool) -> bool {
    !agree && previous >= 1
}

/// G01：三种决策互斥且完备；第一次分歧不放行，第二次交 CPU；计数不溢出。
pub fn reconcile_gpu_entry(previous: u32, agree: bool) -> (step: ReconciliationStep)
    ensures
        step.confirmed == agree,
        step.ask_cpu == cpu_is_needed(previous as nat, agree),
        !(step.confirmed && step.ask_cpu),
        step.uncertain_rounds == next_uncertain_count(previous as nat, agree),
        step.uncertain_rounds <= 1,
        (!step.confirmed && !step.ask_cpu) <==> (!agree && previous == 0),
{
    if agree {
        ReconciliationStep { confirmed: true, ask_cpu: false, uncertain_rounds: 0 }
    } else if previous >= GPU_UNCERTAIN_ROUNDS_BEFORE_CPU - 1 {
        ReconciliationStep { confirmed: false, ask_cpu: true, uncertain_rounds: 0 }
    } else {
        ReconciliationStep { confirmed: false, ask_cpu: false, uncertain_rounds: 1 }
    }
}

/// G02：从零开始的两次分歧，恰好第二次调用 CPU，之后从零重新计数。
pub proof fn lemma_two_disagreements_trigger_exactly_one_cpu_judgment()
    ensures
        !cpu_is_needed(0, false),
        cpu_is_needed(next_uncertain_count(0, false), false),
        next_uncertain_count(next_uncertain_count(0, false), false) == 0,
        !cpu_is_needed(next_uncertain_count(next_uncertain_count(0, false), false), false),
{
}

/// G03：中间一次一致会打断连续分歧，不能把隔轮的分歧累加到 CPU 门槛。
pub proof fn lemma_agreement_restarts_the_disagreement_history(previous: nat)
    ensures
        next_uncertain_count(previous, true) == 0,
        !cpu_is_needed(next_uncertain_count(previous, true), false),
        next_uncertain_count(next_uncertain_count(previous, true), false) == 1,
{
}

pub open spec fn count_of(counts: Map<int, nat>, key: int) -> nat {
    if counts.dom().contains(key) { counts[key] } else { 0 }
}

pub open spec fn update_count(counts: Map<int, nat>, key: int, agree: bool) -> Map<int, nat> {
    if next_uncertain_count(count_of(counts, key), agree) == 0 { counts.remove(key) }
    else { counts.insert(key, 1) }
}

pub open spec fn round_state(counts: Map<int, nat>, cpu_calls: nat, key: int, agree: bool, succeeded: bool)
    -> (Map<int, nat>, nat)
{
    if succeeded {
        (update_count(counts, key, agree), cpu_calls + if cpu_is_needed(count_of(counts, key), agree) { 1nat } else { 0nat })
    } else { (counts, cpu_calls) }
}

/// G04：GPU 批失败不构成一轮；成功轮次恰按仲裁更新状态，并精确记录 CPU 调用。
pub proof fn lemma_failed_batch_keeps_all_counts_and_cpu_calls(counts: Map<int, nat>, cpu_calls: nat, key: int, agree: bool)
    ensures
        round_state(counts, cpu_calls, key, agree, false) == (counts, cpu_calls),
        round_state(counts, cpu_calls, key, agree, true).0 == update_count(counts, key, agree),
        round_state(counts, cpu_calls, key, agree, true).1
            == cpu_calls + if !agree && count_of(counts, key) >= 1 { 1nat } else { 0nat },
{
}

/// G05：不同条目的更新可交换；批内重排不串扰计数与 CPU 调用总数。
pub proof fn lemma_distinct_entry_updates_commute(counts: Map<int, nat>, first: int, second: int, first_agrees: bool, second_agrees: bool)
    requires first != second,
    ensures
        update_count(update_count(counts, first, first_agrees), second, second_agrees)
            == update_count(update_count(counts, second, second_agrees), first, first_agrees),
        count_of(update_count(counts, first, first_agrees), second) == count_of(counts, second),
        cpu_is_needed(count_of(update_count(counts, first, first_agrees), second), second_agrees)
            == cpu_is_needed(count_of(counts, second), second_agrees),
        round_state(
            round_state(counts, 0, first, first_agrees, true).0,
            round_state(counts, 0, first, first_agrees, true).1,
            second, second_agrees, true,
        ) == round_state(
            round_state(counts, 0, second, second_agrees, true).0,
            round_state(counts, 0, second, second_agrees, true).1,
            first, first_agrees, true,
        ),
{
    assert(update_count(update_count(counts, first, first_agrees), second, second_agrees)
        =~= update_count(update_count(counts, second, second_agrees), first, first_agrees));
}

/// G06：稀疏表只保存一个未决轮次；更新保持此不变量，未出现的条目保持原值。
pub proof fn lemma_count_updates_preserve_the_sparse_counter_invariant(counts: Map<int, nat>, key: int, agree: bool)
    requires forall|other: int| counts.dom().contains(other) ==> #[trigger] counts[other] == 1,
    ensures
        forall|other: int| update_count(counts, key, agree).dom().contains(other)
            ==> #[trigger] update_count(counts, key, agree)[other] == 1,
        forall|other: int| other != key ==> #[trigger] count_of(update_count(counts, key, agree), other) == count_of(counts, other),
        count_of(update_count(counts, key, agree), key) == next_uncertain_count(count_of(counts, key), agree),
{
}

pub open spec fn overlap_slots(left_start: int, left_end: int, right_start: int, right_end: int) -> int {
    let begin = if left_start > right_start { left_start } else { right_start };
    let end = if left_end < right_end { left_end } else { right_end };
    if begin < end { end - begin } else { 0 }
}

/// G07：逐记录求交的活槽贡献对称、非负，不超过任一跨度；相邻区间贡献零。
pub proof fn lemma_live_slot_intersection_bounds(left_start: int, left_end: int, right_start: int, right_end: int)
    requires left_start <= left_end, right_start <= right_end,
    ensures
        0 <= overlap_slots(left_start, left_end, right_start, right_end) <= left_end - left_start,
        overlap_slots(left_start, left_end, right_start, right_end) <= right_end - right_start,
        overlap_slots(left_start, left_end, right_start, right_end) == overlap_slots(right_start, right_end, left_start, left_end),
        (left_end <= right_start || right_end <= left_start) ==> overlap_slots(left_start, left_end, right_start, right_end) == 0,
{
}

/// G08：在任意槽界切开段，左右贡献之和等于原段；跨段记录不漏、不重算。
pub proof fn lemma_splitting_a_segment_preserves_live_slot_contributions(record_start: int, record_end: int, start: int, seam: int, end: int)
    requires record_start <= record_end, start <= seam <= end,
    ensures overlap_slots(record_start, record_end, start, end) == overlap_slots(record_start, record_end, start, seam) + overlap_slots(record_start, record_end, seam, end),
{
}

pub struct ScoringRecord {
    pub device: int,
    pub released: bool,
    pub start: int,
    pub end: int,
}

pub open spec fn record_contribution(record: ScoringRecord, device: int, start: int, end: int) -> int {
    if record.released || record.device != device { 0 }
    else { overlap_slots(record.start, record.end, start, end) }
}

pub open spec fn record_slice_sum(records: Seq<ScoringRecord>, device: int, start: int, end: int, first: int, past: int) -> int
    decreases past - first,
{
    if past <= first { 0 }
    else { record_slice_sum(records, device, start, end, first, past - 1)
        + record_contribution(records[past - 1], device, start, end) }
}

/// G09：任意长度记录表在任意切口分片后求和等于整表；空片、空表同样成立。
pub proof fn lemma_record_slices_preserve_the_score(records: Seq<ScoringRecord>, device: int, start: int, end: int, first: int, seam: int, past: int)
    requires 0 <= first <= seam <= past <= records.len(),
    ensures record_slice_sum(records, device, start, end, first, past)
        == record_slice_sum(records, device, start, end, first, seam) + record_slice_sum(records, device, start, end, seam, past),
    decreases past - seam,
{
    if past > seam {
        lemma_record_slices_preserve_the_score(records, device, start, end, first, seam, past - 1);
    }
}

/// G10：重复记录保留计数重数，不能误把求和改成槽集合的并集大小。
pub proof fn lemma_duplicate_records_keep_their_multiplicity(record: ScoringRecord, device: int, start: int, end: int)
    ensures record_slice_sum(seq![record, record], device, start, end, 0, 2)
        == 2 * record_contribution(record, device, start, end),
{
    reveal_with_fuel(record_slice_sum, 3);
}

/// G11：异盘或已释放记录贡献零；活记录贡献恰为交集长度，不能仅保持正负号。
pub proof fn lemma_released_and_foreign_records_do_not_score(record: ScoringRecord, device: int, start: int, end: int)
    requires record.start < record.end, start < end,
    ensures
        (record.released || record.device != device) ==> record_contribution(record, device, start, end) == 0,
        (record_contribution(record, device, start, end) > 0) <==>
            (!record.released && record.device == device && record.start < end && start < record.end),
        !record.released && record.device == device ==>
            record_contribution(record, device, start, end) == overlap_slots(record.start, record.end, start, end),
{
}

pub open spec fn words_value(low: u32, high: u32) -> int {
    low as int + 4294967296int * high as int
}

/// G12：WGSL 的高字优先比较等价于完整 64 位槽号比较，不截掉高字。
pub proof fn lemma_two_word_slot_order_is_u64_order(al: u32, ah: u32, bl: u32, bh: u32)
    ensures
        0 <= words_value(al, ah) <= u64::MAX,
        (words_value(al, ah) < words_value(bl, bh)) <==> (ah < bh || (ah == bh && al < bl)),
        (words_value(al, ah) == words_value(bl, bh)) <==> (al == bl && ah == bh),
{
}

/// G13：低字相加回绕后的进位恰为一次；全宽和有效时高字也不溢出。
pub proof fn lemma_two_word_span_addition_carries_exactly_once(low: u32, high: u32, span: u32)
    requires words_value(low, high) + span <= u64::MAX,
    ensures
        0 <= (low as int + span) % 4294967296 < 4294967296,
        high + (if low as int + span >= 4294967296 { 1int } else { 0int }) <= u32::MAX,
        (low as int + span) % 4294967296
            + 4294967296 * (high + if low as int + span >= 4294967296 { 1int } else { 0int })
            == words_value(low, high) + span,
        ((low as int + span) % 4294967296 < low) <==> (low as int + span >= 4294967296),
{
    if low as int + span >= 4294967296 {
        assert((low as int + span) % 4294967296 == low as int + span - 4294967296);
    } else {
        assert((low as int + span) % 4294967296 == low as int + span);
    }
}

/// G14：两份核分片累计各加各的，任一份溢出就拒绝，不混合或截断结果。
#[allow(clippy::manual_map, reason = "被验函数体照 ensures 的两层 match 逐支写，不用闭包")]
pub fn checked_live_slot_totals(walking: u64, searching: u64, next_walking: u64, next_searching: u64) -> (answer: Option<(u64, u64)>)
    ensures match answer {
        Some((walking_total, searching_total)) => walking_total == walking + next_walking && searching_total == searching + next_searching,
        None => walking + next_walking > u64::MAX || searching + next_searching > u64::MAX,
    },
{
    match walking.checked_add(next_walking) {
        None => None,
        Some(walking_total) => match searching.checked_add(next_searching) {
            None => None,
            Some(searching_total) => Some((walking_total, searching_total)),
        },
    }
}

pub open spec fn aligned_input_size(input: u64) -> int {
    if input == 0 { 4 } else { crate::file_range_proofs::covering_units(input, 4) * 4 }
}

/// G15：输入缓冲恰为最小非零四字节对齐长度；全宽向上取整溢出显式拒绝。
pub fn checked_gpu_input_bytes(input: u64) -> (answer: Option<u64>)
    ensures match answer {
        None => aligned_input_size(input) > u64::MAX,
        Some(bytes) => bytes == aligned_input_size(input) && bytes >= 4
            && bytes >= input && bytes % 4 == 0 && (input > 0 ==> bytes < input + 4),
    },
{
    let words = units_for_sequential_write(input, GPU_INPUT_ALIGNMENT);
    words.checked_mul(GPU_INPUT_ALIGNMENT)
}

/// G16：预算计算包含对齐输入、逐条固定占用和整批固定占用，乘加任一溢出都拒绝。
pub fn checked_gpu_chunk_bytes(input: u64, entries: u64, per_entry: u64, fixed: u64) -> (answer: Option<u64>)
    ensures match answer {
        None => aligned_input_size(input) + entries * per_entry + fixed > u64::MAX,
        Some(bytes) => bytes == aligned_input_size(input) + entries * per_entry + fixed,
    },
{
    match checked_gpu_input_bytes(input) {
        None => None,
        Some(aligned) => match entries.checked_mul(per_entry) {
            None => None,
            Some(overhead) => match aligned.checked_add(overhead) {
                None => None,
                Some(subtotal) => subtotal.checked_add(fixed),
            },
        },
    }
}

/// G17：各后台阶段共享显存预算；已占超限时不回绕，正的新增分配必在总限内。
#[allow(clippy::implicit_saturating_sub, reason = "被验函数体与 ensures 的分支逐字对照，不换成饱和减")]
pub fn remaining_gpu_memory(limit: u64, held: u64) -> (remaining: u64)
    ensures
        remaining == if held < limit { limit - held } else { 0int },
        remaining <= limit,
        held <= limit ==> held + remaining == limit,
{
    if held >= limit { 0 } else { limit - held }
}

pub open spec fn in_slice(index: int, start: u64, end: u64) -> bool {
    start <= index < end
}

/// G18：下一记录片有进展、不过界；减法取余量避免 start + capacity 溢出。
pub fn next_gpu_record_slice_end(start: u64, total: u64, capacity: u64) -> (end: u64)
    requires start <= total, capacity > 0,
    ensures
        start <= end <= total,
        end - start <= capacity,
        end == if total - start <= capacity { total as int } else { start + capacity },
        start < total ==> end > start,
        forall|index: int| #[trigger] in_slice(index, start, total)
            <==> (in_slice(index, start, end) || in_slice(index, end, total)),
        forall|index: int| !(#[trigger] in_slice(index, start, end) && in_slice(index, end, total)),
{
    if total - start <= capacity { total } else { start + capacity }
}

/// G19：只有同盘未释放、起点在区内且全跨度都在区内的记录才允许扫描。
pub fn gpu_record_is_in_scan_region(same_device: bool, released: bool, slot: u64, span: u64, start: u64, end: u64) -> (read: bool)
    ensures read == (same_device && !released && start <= slot < end && slot + span <= end),
{
    if !same_device || released || slot < start || slot >= end { return false; }
    match slot.checked_add(span) {
        None => false,
        Some(past) => past <= end,
    }
}

/// G20：判定字必须是 counted=1，填充字/未知字不成为活槽数；解码保留两个字。
pub fn decoded_gpu_live_slots(low: u32, high: u32, verdict: u32) -> (answer: Option<u64>)
    ensures match answer {
        None => verdict != 1,
        Some(value) => verdict == 1 && value == words_value(low, high),
    },
{
    if verdict != GPU_LIVE_SLOTS_COUNTED { None } else { Some(low as u64 + 4294967296u64 * high as u64) }
}

/// 非空协议与打分见证，不计入 G01–G20。
pub proof fn witness_gpu_gc_protocol_and_score_premises()
    ensures
        overlap_slots(9, 12, 10, 20) == 2,
        record_slice_sum(seq![ScoringRecord { device: 7, released: false, start: 9, end: 12 }], 7, 10, 20, 0, 1) == 2,
        count_of(update_count(Map::<int, nat>::empty(), 7, false), 7) == 1,
{
    let counts = Map::<int, nat>::empty().insert(7, 1nat);
    lemma_distinct_entry_updates_commute(counts, 7, 8, false, false);
    lemma_count_updates_preserve_the_sparse_counter_invariant(counts, 7, false);
    lemma_record_slices_preserve_the_score(seq![ScoringRecord { device: 7, released: false, start: 9, end: 12 }], 7, 10, 20, 0, 0, 1);
    lemma_live_slot_intersection_bounds(9, 12, 10, 20);
    lemma_splitting_a_segment_preserves_live_slot_contributions(9, 12, 10, 11, 20);
    lemma_released_and_foreign_records_do_not_score(ScoringRecord { device: 7, released: false, start: 9, end: 12 }, 7, 10, 20);
    lemma_two_word_span_addition_carries_exactly_once(u32::MAX, 0, 1);
}

/// 产品纯核的极值见证，不计入 G01–G20；普通 Rust 下也可以执行。
#[allow(unused_variables, reason = "assert 在普通 Rust 构建中擦除，执行调用保留")]
pub fn witness_gpu_gc_arithmetic_boundaries()
{
    let defer = reconcile_gpu_entry(0, false);
    assert(!defer.confirmed && !defer.ask_cpu && defer.uncertain_rounds == 1);
    let judge = reconcile_gpu_entry(defer.uncertain_rounds, false);
    assert(judge.ask_cpu && judge.uncertain_rounds == 0);
    let empty = checked_gpu_input_bytes(0);
    let too_large = checked_gpu_input_bytes(u64::MAX);
    let footprint = checked_gpu_chunk_bytes(1, 2, 40, 16);
    let overflow = checked_gpu_chunk_bytes(1, u64::MAX, 2, 0);
    let remaining = remaining_gpu_memory(1, u64::MAX);
    let past = next_gpu_record_slice_end(u64::MAX - 1, u64::MAX, u64::MAX);
    let scan = gpu_record_is_in_scan_region(true, false, u64::MAX - 1, 2, 0, u64::MAX);
    let decoded = decoded_gpu_live_slots(u32::MAX, u32::MAX, 1);
    let unwritten = decoded_gpu_live_slots(0, 0, u32::MAX);
    let sum = checked_live_slot_totals(1, 2, 3, 4);
    let sum_overflow = checked_live_slot_totals(0, u64::MAX, 1, 1);
    assert(empty == Some(4));
    assert(too_large == None);
    assert(footprint == Some(100));
    assert(overflow == None);
    assert(remaining == 0);
    assert(past == u64::MAX);
    assert(!scan);
    assert(decoded == Some(u64::MAX));
    assert(unwritten == None);
    assert(sum == Some((4, 6)));
    assert(sum_overflow == None);
}

} // verus!

// 选段打分里每个聚簇段的活槽数，两份独立核里逐条走遍分配记录、按区间相交算长度的那一份（D26（后台整理与放置回收） 已定项 13 第 8 条：两份核不共用核函数）。
// 每个段一个调用：从头到尾走一遍这一段的分配记录表，只看单元区那块盘上、未释放的记录，按「记录罩住的槽区间与这一段的槽区间相交多长」一次算出
// 它在这一段里罩住几个槽，加起来（几条记录罩住同一个槽各算一次，同核心层 compaction_plan::live_slots_per_cluster_segment_by_the_records）。
// 槽号是 64 位：两个 u32 拼成 vec2（x 是低 32 位、y 是高 32 位），比较与加法在这里自己写；活槽数也按 64 位累加。
// 另一份（live_slot_kernel_searching_sorted_records.wgsl）在按（盘、槽号）排好的序号上二分出可能落进这一段的记录、逐槽数；
// 两份只共用缓冲布局，布局由 src/live_slot_kernels.rs 定。

struct WalkingChunkParameters {
    number_of_segments: u32,
    number_of_records: u32,
    device_of_the_unit_area: u32,
    word_left_unused: u32,
}

struct SegmentToWalk {
    first_slot_low: u32,
    first_slot_high: u32,
    slot_past_the_segment_low: u32,
    slot_past_the_segment_high: u32,
}

struct RecordToWalk {
    device_of_the_record: u32,
    first_slot_low: u32,
    first_slot_high: u32,
    span_in_slots: u32,
    released_word: u32,
}

struct LiveSlotsOfTheWalk {
    live_slots_low: u32,
    live_slots_high: u32,
    verdict_word: u32,
}

@group(0) @binding(0) var<uniform> walking_parameters: WalkingChunkParameters;
@group(0) @binding(1) var<storage, read> segments_to_walk: array<SegmentToWalk>;
@group(0) @binding(2) var<storage, read> records_to_walk: array<RecordToWalk>;
@group(0) @binding(4) var<storage, read_write> live_slots_of_the_walk: array<LiveSlotsOfTheWalk>;

// 记录释放了的那一个字（src/live_slot_kernels.rs 打包时写 1，没释放写 0）。
const RELEASED_WORD_OF_A_RELEASED_RECORD: u32 = 1u;
// 判定字：这一段数完了。
const VERDICT_WORD_COUNTED: u32 = 1u;

fn slot_is_before(left: vec2<u32>, right: vec2<u32>) -> bool {
    return left.y < right.y || (left.y == right.y && left.x < right.x);
}

fn later_of_the_slots(left: vec2<u32>, right: vec2<u32>) -> vec2<u32> {
    if (slot_is_before(left, right)) {
        return right;
    }
    return left;
}

fn earlier_of_the_slots(left: vec2<u32>, right: vec2<u32>) -> vec2<u32> {
    if (slot_is_before(left, right)) {
        return left;
    }
    return right;
}

fn slot_past_the_span(first_slot: vec2<u32>, span_in_slots: u32) -> vec2<u32> {
    let low_word = first_slot.x + span_in_slots;
    let carry = select(0u, 1u, low_word < first_slot.x);
    return vec2<u32>(low_word, first_slot.y + carry);
}

@compute @workgroup_size(64)
fn count_live_slots_walking_every_record(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let segment_number = invocation.x;
    if (segment_number >= walking_parameters.number_of_segments) {
        return;
    }
    let segment = segments_to_walk[segment_number];
    let segment_first_slot = vec2<u32>(segment.first_slot_low, segment.first_slot_high);
    let slot_past_the_segment = vec2<u32>(segment.slot_past_the_segment_low, segment.slot_past_the_segment_high);
    var live_slots_low = 0u;
    var live_slots_high = 0u;
    for (var record_number = 0u; record_number < walking_parameters.number_of_records; record_number = record_number + 1u) {
        let record = records_to_walk[record_number];
        if (record.device_of_the_record != walking_parameters.device_of_the_unit_area) {
            continue;
        }
        if (record.released_word == RELEASED_WORD_OF_A_RELEASED_RECORD) {
            continue;
        }
        let record_first_slot = vec2<u32>(record.first_slot_low, record.first_slot_high);
        let slot_past_the_record = slot_past_the_span(record_first_slot, record.span_in_slots);
        let first_slot_of_both = later_of_the_slots(record_first_slot, segment_first_slot);
        let slot_past_both = earlier_of_the_slots(slot_past_the_record, slot_past_the_segment);
        if (slot_is_before(first_slot_of_both, slot_past_both)) {
            // 相交的那一截在这一段之内，比 2^32 短：低 32 位相减就是它的槽数
            let slots_in_both = slot_past_both.x - first_slot_of_both.x;
            let sum_low = live_slots_low + slots_in_both;
            live_slots_high = live_slots_high + select(0u, 1u, sum_low < live_slots_low);
            live_slots_low = sum_low;
        }
    }
    live_slots_of_the_walk[segment_number] = LiveSlotsOfTheWalk(live_slots_low, live_slots_high, VERDICT_WORD_COUNTED);
}

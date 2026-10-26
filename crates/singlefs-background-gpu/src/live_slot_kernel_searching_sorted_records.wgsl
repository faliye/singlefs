// 选段打分里每个聚簇段的活槽数，两份独立核里在按（盘、槽号）排好的记录序号上二分、逐槽数的那一份（D26（后台整理与放置回收） 已定项 13 第 8 条：两份核不共用核函数）。
// 每个段一个调用：分配记录的跨度是两字节无符号数，起点比这一段的头一个槽早 65535 槽以上的记录罩不到这一段；在排好的序号上二分出单元区那块盘上
// 起点不早于「这一段的头一个槽减 65535」的第一条、与起点不早于「这一段之后的第一个槽」的第一条，两者之间的每条未释放记录从它的头一个槽起
// 逐槽往后走、落进这一段的槽各数一次（几条记录罩住同一个槽各算一次，同核心层 compaction_plan::live_slots_per_cluster_segment_by_the_records）。
// 64 位的数在这里按（高 32 位、低 32 位）两个 u32 存，比较、加一、减法都在这里自己写。
// 另一份（live_slot_kernel_walking_every_record.wgsl）逐条走遍记录、按区间相交算长度；两份只共用缓冲布局，布局由 src/live_slot_kernels.rs 定。

struct SearchingParameters {
    segment_count: u32,
    record_count: u32,
    unit_area_device: u32,
    unused_word: u32,
}

struct BoundsOfTheSegment {
    start_slot_low: u32,
    start_slot_high: u32,
    end_slot_low: u32,
    end_slot_high: u32,
}

struct AllocationRecordToSearch {
    device: u32,
    slot_low: u32,
    slot_high: u32,
    span: u32,
    released: u32,
}

struct CountOfTheSearch {
    count_low: u32,
    count_high: u32,
    written_word: u32,
}

@group(0) @binding(0) var<uniform> searching_parameters: SearchingParameters;
@group(0) @binding(1) var<storage, read> bounds_of_the_segments: array<BoundsOfTheSegment>;
@group(0) @binding(2) var<storage, read> allocation_records_to_search: array<AllocationRecordToSearch>;
@group(0) @binding(3) var<storage, read> record_positions_by_device_and_slot: array<u32>;
@group(0) @binding(4) var<storage, read_write> counts_of_the_search: array<CountOfTheSearch>;

// 分配记录的跨度最多这么多个槽（两字节无符号数的最大值）。
const LONGEST_SPAN_OF_AN_ALLOCATION_RECORD: u32 = 65535u;
// 没释放的记录那一个字（src/live_slot_kernels.rs 打包时写 0，释放了写 1）。
const RELEASED_FLAG_CLEAR: u32 = 0u;
const COUNT_WRITTEN: u32 = 1u;

struct WideNumber {
    high: u32,
    low: u32,
}

fn wide_is_less(left: WideNumber, right: WideNumber) -> bool {
    if (left.high != right.high) {
        return left.high < right.high;
    }
    return left.low < right.low;
}

fn wide_plus_one(number: WideNumber) -> WideNumber {
    if (number.low == 0xFFFFFFFFu) {
        return WideNumber(number.high + 1u, 0u);
    }
    return WideNumber(number.high, number.low + 1u);
}

// 减去一个 32 位数，比 0 小就停在 0。
fn wide_minus_or_zero(number: WideNumber, amount: u32) -> WideNumber {
    if (number.low >= amount) {
        return WideNumber(number.high, number.low - amount);
    }
    if (number.high == 0u) {
        return WideNumber(0u, 0u);
    }
    // 借一位：低 32 位回绕
    return WideNumber(number.high - 1u, number.low - amount);
}

// 排在第 sorted_number 位的记录是不是排在（device, slot）之前：按（盘、槽号）比。
fn sorted_record_comes_before(sorted_number: u32, device: u32, slot: WideNumber) -> bool {
    let record = allocation_records_to_search[record_positions_by_device_and_slot[sorted_number]];
    if (record.device != device) {
        return record.device < device;
    }
    return wide_is_less(WideNumber(record.slot_high, record.slot_low), slot);
}

// 排好的序号里第一条不排在（device, slot）之前的位置；都排在前面时交回记录数。
fn first_sorted_number_from(device: u32, slot: WideNumber) -> u32 {
    var below = 0u;
    var above = searching_parameters.record_count;
    while (below < above) {
        let middle = below + (above - below) / 2u;
        if (sorted_record_comes_before(middle, device, slot)) {
            below = middle + 1u;
        } else {
            above = middle;
        }
    }
    return below;
}

@compute @workgroup_size(64)
fn count_live_slots_searching_the_sorted_records(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let segment_position = invocation.x;
    if (segment_position >= searching_parameters.segment_count) {
        return;
    }
    let bounds = bounds_of_the_segments[segment_position];
    let start_of_the_segment = WideNumber(bounds.start_slot_high, bounds.start_slot_low);
    let end_of_the_segment = WideNumber(bounds.end_slot_high, bounds.end_slot_low);
    let device = searching_parameters.unit_area_device;
    let earliest_start_reaching_the_segment = wide_minus_or_zero(start_of_the_segment, LONGEST_SPAN_OF_AN_ALLOCATION_RECORD);
    let past_the_candidates = first_sorted_number_from(device, end_of_the_segment);
    var count = WideNumber(0u, 0u);
    for (var sorted_number = first_sorted_number_from(device, earliest_start_reaching_the_segment); sorted_number < past_the_candidates; sorted_number = sorted_number + 1u) {
        let record = allocation_records_to_search[record_positions_by_device_and_slot[sorted_number]];
        if (record.released != RELEASED_FLAG_CLEAR) {
            continue;
        }
        var slot = WideNumber(record.slot_high, record.slot_low);
        for (var step = 0u; step < record.span; step = step + 1u) {
            if (!wide_is_less(slot, start_of_the_segment) && wide_is_less(slot, end_of_the_segment)) {
                count = wide_plus_one(count);
            }
            slot = wide_plus_one(slot);
        }
    }
    counts_of_the_search[segment_position] = CountOfTheSearch(count.low, count.high, COUNT_WRITTEN);
}

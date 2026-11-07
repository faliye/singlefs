//! 截断/删除的尾部分批与部分写交集：core 直接调用这里的可执行核。
//! 也验证 size 到单元数、整单元取整准入及部分写单元地址换算；发布 I/O 仍由调用方负责。
use vstd::prelude::*;

verus! {

pub open spec fn covering_units(size: u64, capacity: u64) -> int {
    if size == 0 { 0 } else { (size - 1) / (capacity as int) + 1 }
}

/// 非零容量下覆盖 size 个字节所需的最少单元数；不通过 size + capacity - 1 计算，避免中间和溢出。
pub fn units_covering_size(size: u64, capacity: u64) -> (count: u64)
    requires capacity > 0,
    ensures
        count == covering_units(size, capacity),
        count <= size,
        count == 0 <==> size == 0,
        size <= count * capacity,
        size > 0 ==> (count - 1) * capacity < size,
{
    if size == 0 {
        0
    } else {
        proof {
            vstd::arithmetic::div_mod::lemma_div_nonincreasing(size - 1, capacity as int);
            vstd::arithmetic::div_mod::lemma_fundamental_div_mod(size - 1, capacity as int);
        }
        let quotient = (size - 1) / capacity;
        proof {
            assert(quotient * capacity <= size - 1 < (quotient + 1) * capacity) by (nonlinear_arith)
                requires
                    capacity > 0,
                    size - 1 == quotient * capacity + (size - 1) % (capacity as int),
                    0 <= (size - 1) % (capacity as int) < capacity,
            {}
        }
        quotient + 1
    }
}

/// 顺序写（以及截断保留）的单元数：空内容仍保留一个声明长度为零的单元。
pub fn units_for_sequential_write(size: u64, capacity: u64) -> (count: u64)
    requires capacity > 0,
    ensures
        count == if size == 0 { 1 } else { covering_units(size, capacity) },
        count >= 1,
        size <= count * capacity,
        size > 0 ==> (count - 1) * capacity < size,
{
    let count = units_covering_size(size, capacity);
    if count == 0 { 1 } else { count }
}

/// 截断目标按整单元向上取整后是否仍在 u64 内；零大小按零个单元作准入计算。
pub fn rounded_size_fits(size: u64, capacity: u64) -> (fits: bool)
    requires capacity > 0,
    ensures
        fits == (covering_units(size, capacity) * capacity <= u64::MAX),
{
    let count = units_covering_size(size, capacity);
    count.checked_mul(capacity).is_some()
}

/// size 增大时覆盖它所需的单元数不会减少。
pub proof fn lemma_covering_units_are_monotone(smaller: u64, larger: u64, capacity: u64)
    requires
        capacity > 0,
        smaller <= larger,
    ensures covering_units(smaller, capacity) <= covering_units(larger, capacity),
{
    if smaller > 0 {
        vstd::arithmetic::div_mod::lemma_div_is_ordered(smaller - 1, larger - 1, capacity as int);
    }
}

/// 请求终点：数学和装得进 u64 当且仅当返回 Some，包括恰好到 u64::MAX 的边界。
pub fn checked_write_end(start: u64, length: u64) -> (answer: Option<u64>)
    ensures
        match answer {
            None => start + length > u64::MAX,
            Some(end) => end == start + length && start <= end,
        },
{
    start.checked_add(length)
}

pub open spec fn payload_length_for_size(start: u64, size: u64, capacity: u64) -> int {
    if size <= start { 0 }
    else if size - start < capacity { size - start }
    else { capacity as int }
}

pub open spec fn payload_byte_is_in_file(start: u64, size: u64, capacity: u64, offset: int) -> bool {
    0 <= offset < capacity && start + offset < size
}

/// 已知单元起点后，文件 size 以内的声明长度。允许起点在 EOF 之后，也允许零容量。
pub fn payload_length_from_start(start: u64, size: u64, capacity: u64) -> (length: u64)
    ensures
        length == payload_length_for_size(start, size, capacity),
        length <= capacity,
        length <= size,
        length > 0 ==> start + length <= size,
        forall|offset: int| #[trigger] payload_byte_is_in_file(start, size, capacity, offset)
            <==> 0 <= offset < length,
{
    if size <= start {
        0
    } else {
        let remaining = size - start;
        if remaining < capacity { remaining } else { capacity }
    }
}

/// 变长不缩短声明长度，新增声明字节数不超过文件增加的字节数。
pub proof fn lemma_payload_length_grows_by_at_most_the_size_change(start: u64, smaller: u64, larger: u64, capacity: u64)
    requires smaller <= larger,
    ensures
        payload_length_for_size(start, smaller, capacity) <= payload_length_for_size(start, larger, capacity),
        payload_length_for_size(start, larger, capacity) - payload_length_for_size(start, smaller, capacity) <= larger - smaller,
{
}

/// 截短后保留的字节恰好是旧载荷在新 EOF 以内的前缀；不是只证明长度上界。
pub proof fn lemma_truncation_keeps_exactly_the_payload_prefix(start: u64, old_size: u64, new_size: u64, capacity: u64, offset: int)
    requires new_size <= old_size,
    ensures
        (0 <= offset < payload_length_for_size(start, new_size, capacity)) <==>
            (0 <= offset < payload_length_for_size(start, old_size, capacity) && start + offset < new_size),
{
}

/// 返回（本批之后保留的单元数，是否最后一批）。desired 可以超过 existing（尾部有洞）。
pub fn next_tail_batch(existing: u64, desired: u64, batch: u64) -> (answer: (u64, bool))
    requires batch > 0,
    ensures
        answer.0 <= existing,
        answer.0 >= if desired < existing { desired } else { existing },
        existing - answer.0 <= batch,
        answer.1 == (existing <= desired + batch as int),
        answer.1 ==> answer.0 == if desired < existing { desired } else { existing },
        !answer.1 ==> answer.0 == existing - batch && answer.0 < existing,
{
    let target = if desired < existing { desired } else { existing };
    if existing - target <= batch {
        (target, true)
    } else {
        (existing - batch, false)
    }
}

pub open spec fn in_both_ranges(byte: int, write_start: u64, write_end: u64, unit_start: u64, unit_end: u64) -> bool {
    write_start <= byte < write_end && unit_start <= byte < unit_end
}

/// 返回（单元内偏移，请求内偏移，长度）；None 恰好表示交集为空。也接受空或倒置的区间。
pub fn bytes_in_unit(write_start: u64, write_end: u64, unit_start: u64, unit_end: u64)
    -> (answer: Option<(u64, u64, u64)>)
    ensures
        match answer {
            None => forall|byte: int| !in_both_ranges(byte, write_start, write_end, unit_start, unit_end),
            Some((in_unit, in_write, length)) => {
                &&& length > 0
                &&& unit_start + in_unit as int == write_start + in_write as int
                &&& in_unit + length as int <= unit_end - unit_start
                &&& in_write + length as int <= write_end - write_start
                &&& forall|byte: int| in_both_ranges(byte, write_start, write_end, unit_start, unit_end)
                    <==> unit_start + in_unit as int <= byte < unit_start + in_unit as int + length
            },
        },
{
    let from = if write_start > unit_start { write_start } else { unit_start };
    let until = if write_end < unit_end { write_end } else { unit_end };
    if from < until {
        Some((from - unit_start, from - write_start, until - from))
    } else {
        None
    }
}

/// None 精确表示完整单元的数学终点装不进 u64；零容量对应空区间。
pub fn checked_unit_byte_range(unit_index: u64, capacity: u64) -> (answer: Option<(u64, u64)>)
    ensures
        match answer {
            None => unit_index * capacity + capacity > u64::MAX,
            Some((start, end)) => start == unit_index * capacity && end == start + capacity,
        },
{
    let start = unit_index.checked_mul(capacity)?;
    let end = start.checked_add(capacity)?;
    Some((start, end))
}

pub open spec fn byte_in_write_and_indexed_unit(byte: int, write_start: u64, write_end: u64, unit_index: u64, capacity: u64) -> bool {
    write_start <= byte < write_end && unit_index * capacity <= byte < unit_index * capacity + capacity
}

/// 外层 None 是单元地址溢出；Some(None) 是可表示单元与写区间没有共同字节。
pub fn checked_bytes_in_indexed_unit(write_start: u64, write_end: u64, unit_index: u64, capacity: u64)
    -> (answer: Option<Option<(u64, u64, u64)>>)
    ensures
        match answer {
            None => unit_index * capacity + capacity > u64::MAX,
            Some(intersection) => {
                &&& unit_index * capacity + capacity <= u64::MAX
                &&& match intersection {
                    None => forall|byte: int| !byte_in_write_and_indexed_unit(byte, write_start, write_end, unit_index, capacity),
                    Some((in_unit, in_write, length)) => {
                        &&& length > 0
                        &&& unit_index * capacity + in_unit == write_start + in_write as int
                        &&& in_unit + length as int <= capacity
                        &&& in_write + length as int <= write_end - write_start
                        &&& forall|byte: int| byte_in_write_and_indexed_unit(byte, write_start, write_end, unit_index, capacity)
                            <==> unit_index * capacity + in_unit <= byte < unit_index * capacity + in_unit + length
                    },
                }
            },
        },
{
    match checked_unit_byte_range(unit_index, capacity) {
        None => None,
        Some((start, end)) => {
            let intersection = bytes_in_unit(write_start, write_end, start, end);
            proof {
                match intersection {
                    None => {
                        assert forall|byte: int| !byte_in_write_and_indexed_unit(byte, write_start, write_end, unit_index, capacity) by {
                            assert(!in_both_ranges(byte, write_start, write_end, start, end));
                        }
                    },
                    Some((in_unit, in_write, length)) => {
                        assert forall|byte: int| byte_in_write_and_indexed_unit(byte, write_start, write_end, unit_index, capacity)
                            <==> unit_index * capacity + in_unit <= byte < unit_index * capacity + in_unit + length by {
                            assert(in_both_ranges(byte, write_start, write_end, start, end)
                                <==> start + in_unit as int <= byte < start + in_unit as int + length);
                        }
                    },
                }
            }
            Some(intersection)
        },
    }
}

/// 整单元取整准入通过后，size 覆盖范围内每个单元的地址都可表示。
/// 不涵盖 size 之外另外保留的旧单元；那些单元的界限要由文件状态提供。
pub proof fn lemma_rounded_size_bounds_written_unit_addresses(size: u64, capacity: u64, unit_index: u64)
    requires
        capacity > 0,
        0 <= unit_index < covering_units(size, capacity),
        covering_units(size, capacity) * capacity <= u64::MAX,
    ensures
        unit_index * capacity + capacity <= u64::MAX,
{
    vstd::arithmetic::mul::lemma_mul_inequality(unit_index + 1, covering_units(size, capacity), capacity as int);
    assert(unit_index * capacity + capacity == (unit_index + 1) * capacity) by (nonlinear_arith);
}

/// 无前提边界见证：整批恰好完成、空文件、u64 极值、相邻区间不相交。
pub fn witness_file_range_boundaries() {
    let _last_representable_unit = checked_unit_byte_range(u64::MAX - 1, 1);
    assert(_last_representable_unit == Some(((u64::MAX - 1) as u64, u64::MAX)));
    let _unit_end_overflow = checked_unit_byte_range(u64::MAX, 1);
    assert(_unit_end_overflow is None);
    let _indexed_bytes = checked_bytes_in_indexed_unit(3, 7, 1, 4);
    assert(_indexed_bytes == Some(Some((0u64, 1u64, 3u64)))) by {
        assert(byte_in_write_and_indexed_unit(4, 3, 7, 1, 4));
        if let Some(Some((offset_in_unit, offset_in_write, length))) = _indexed_bytes {
            assert(byte_in_write_and_indexed_unit(6, 3, 7, 1, 4));
        }
    }
    proof { lemma_rounded_size_bounds_written_unit_addresses(1, 1, 0); }
    let _empty_units = units_for_sequential_write(0, 32_634);
    assert(_empty_units == 1);
    let _max_units = units_covering_size(u64::MAX, 1);
    assert(_max_units == u64::MAX);
    let _rounded_overflow = rounded_size_fits(u64::MAX, 2);
    assert(!_rounded_overflow);
    let _end = checked_write_end(u64::MAX - 3, 3);
    assert(_end == Some(u64::MAX));
    let _overflow = checked_write_end(u64::MAX, 1);
    assert(_overflow is None);
    let _length = payload_length_from_start(7, 10, 8);
    assert(_length == 3);
    let _past_eof = payload_length_from_start(u64::MAX, 1, u64::MAX);
    assert(_past_eof == 0);
    let _exact = next_tail_batch(9, 5, 4);
    assert(_exact == (5u64, true));
    let _empty = next_tail_batch(0, 1, 1);
    assert(_empty == (0u64, true));
    let _high = next_tail_batch(u64::MAX, u64::MAX - 1, 1);
    assert(_high.0 == u64::MAX - 1 && _high.1);
    let _adjacent = bytes_in_unit(0, 10, 10, 20);
    assert(_adjacent is None) by {
        if let Some((a, b, n)) = _adjacent {
            assert(in_both_ranges(10 + a as int, 0, 10, 10, 20));
        }
    }
}

/// 非最终批的剩余工作量严格下降，且本批与后续批释放的尾区间首尾相接。
pub proof fn lemma_tail_batches_partition_the_released_suffix(existing: u64, desired: u64, batch: u64, next: u64, unit: int)
    requires
        batch > 0,
        desired + (batch as int) < existing,
        desired <= next <= existing - batch,
    ensures
        existing - batch - desired < existing - desired,
        next <= unit < existing <==>
            (next <= unit < existing - batch || existing - batch <= unit < existing),
        !(next <= unit < existing - batch && existing - batch <= unit < existing),
{
}

/// 相邻单元上的交集互不重叠，合起来恰好覆盖合并区间内的写字节。
pub proof fn lemma_adjacent_units_partition_the_write(write_start: u64, write_end: u64, left: u64, seam: u64, right: u64)
    requires left <= seam <= right,
    ensures
        forall|byte: int| in_both_ranges(byte, write_start, write_end, left, right) <==>
            (in_both_ranges(byte, write_start, write_end, left, seam)
                || in_both_ranges(byte, write_start, write_end, seam, right)),
        forall|byte: int| !(in_both_ranges(byte, write_start, write_end, left, seam)
            && in_both_ranges(byte, write_start, write_end, seam, right)),
{
}

} // verus!

#[cfg(test)]
mod tests {
    use super::{
        bytes_in_unit, checked_write_end, next_tail_batch, payload_length_from_start,
        rounded_size_fits, units_covering_size, units_for_sequential_write,
        witness_file_range_boundaries,
    };
    use super::{checked_bytes_in_indexed_unit, checked_unit_byte_range};

    #[test]
    fn unit_count_matches_the_distinct_units_touched_by_each_byte() {
        for size in 0..96u64 {
            for capacity in 1..20u64 {
                let touched = (0..size)
                    .map(|byte| byte / capacity)
                    .collect::<std::collections::BTreeSet<_>>();
                assert_eq!(units_covering_size(size, capacity), touched.len() as u64);
                assert_eq!(
                    units_for_sequential_write(size, capacity),
                    (touched.len() as u64).max(1)
                );
            }
        }
    }

    #[test]
    fn indexed_unit_ranges_match_wide_arithmetic_at_both_overflow_steps() {
        let values = [0u64, 1, 2, 32_634, u64::MAX / 2, u64::MAX - 1, u64::MAX];
        for unit_index in values {
            for capacity in values {
                let start = u128::from(unit_index) * u128::from(capacity);
                let end = start + u128::from(capacity);
                let expected = u64::try_from(start).ok().zip(u64::try_from(end).ok());
                assert_eq!(checked_unit_byte_range(unit_index, capacity), expected);
                if expected.is_none() {
                    assert_eq!(
                        checked_bytes_in_indexed_unit(0, u64::MAX, unit_index, capacity),
                        None
                    );
                }
            }
        }
        assert_eq!(checked_unit_byte_range(u64::MAX, 0), Some((0, 0)));
        assert_eq!(
            checked_unit_byte_range(u64::MAX - 1, 1),
            Some((u64::MAX - 1, u64::MAX))
        );
        assert_eq!(checked_unit_byte_range(u64::MAX, 1), None);
        assert_eq!(checked_unit_byte_range(u64::MAX, 2), None);
        assert_eq!(
            checked_bytes_in_indexed_unit(u64::MAX - 1, u64::MAX, u64::MAX - 1, 1),
            Some(Some((0, 0, 1)))
        );
    }

    #[test]
    fn indexed_write_intersection_matches_enumerated_bytes() {
        for capacity in 0..6u64 {
            for unit_index in 0..6u64 {
                for write_start in 0..18u64 {
                    for write_end in 0..18u64 {
                        let start = unit_index * capacity;
                        let bytes = (start..start + capacity)
                            .filter(|byte| (write_start..write_end).contains(byte))
                            .collect::<Vec<_>>();
                        let actual = checked_bytes_in_indexed_unit(
                            write_start,
                            write_end,
                            unit_index,
                            capacity,
                        )
                        .expect("小范围完整单元地址可表示");
                        match actual {
                            None => assert!(bytes.is_empty()),
                            Some((in_unit, in_write, length)) => {
                                assert!(!bytes.is_empty());
                                assert_eq!(
                                    (start + in_unit..start + in_unit + length).collect::<Vec<_>>(),
                                    bytes
                                );
                                assert_eq!(
                                    (write_start + in_write..write_start + in_write + length)
                                        .collect::<Vec<_>>(),
                                    bytes
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn full_width_rounding_matches_wide_arithmetic_without_an_intermediate_overflow() {
        for capacity in [1u64, 2, 3, 32_634, u64::MAX / 2, u64::MAX] {
            let largest_multiple = u64::MAX / capacity * capacity;
            for size in [
                0,
                1,
                capacity - 1,
                capacity,
                largest_multiple - 1,
                largest_multiple,
                largest_multiple.saturating_add(1),
                u64::MAX,
            ] {
                let count = u128::from(size).div_ceil(u128::from(capacity));
                assert_eq!(u128::from(units_covering_size(size, capacity)), count);
                assert_eq!(
                    u128::from(units_for_sequential_write(size, capacity)),
                    count.max(1)
                );
                assert_eq!(
                    rounded_size_fits(size, capacity),
                    count * u128::from(capacity) <= u128::from(u64::MAX)
                );
            }
        }
        assert_eq!(units_for_sequential_write(0, 32_634), 1);
        assert!(rounded_size_fits(0, 32_634));
        assert!(!rounded_size_fits(u64::MAX, 2));
    }

    #[test]
    fn write_end_matches_wide_arithmetic_at_the_overflow_boundary() {
        let values = [0, 1, 2, 32_634, u64::MAX / 2, u64::MAX - 1, u64::MAX];
        for start in values {
            for length in values {
                let mathematical_end = u128::from(start) + u128::from(length);
                assert_eq!(
                    checked_write_end(start, length),
                    u64::try_from(mathematical_end).ok()
                );
            }
        }
        assert_eq!(checked_write_end(u64::MAX - 1, 1), Some(u64::MAX));
        assert_eq!(checked_write_end(u64::MAX, 1), None);
    }

    #[test]
    fn declared_length_counts_exactly_the_payload_bytes_before_eof() {
        for start in 0..12u64 {
            for size in 0..16u64 {
                for capacity in 0..12u64 {
                    let bytes = (0..capacity)
                        .filter(|offset| start + offset < size)
                        .collect::<Vec<_>>();
                    let length = payload_length_from_start(start, size, capacity);
                    assert_eq!((0..length).collect::<Vec<_>>(), bytes);
                }
            }
        }
        assert_eq!(payload_length_from_start(u64::MAX - 2, u64::MAX, 4), 2);
        assert_eq!(payload_length_from_start(0, u64::MAX, u64::MAX), u64::MAX);
        assert_eq!(payload_length_from_start(u64::MAX, u64::MAX, 10), 0);
    }

    #[test]
    fn tail_batches_release_each_suffix_unit_once_and_finish_in_the_expected_number_of_steps() {
        for existing in 0..20u64 {
            for desired in 0..24u64 {
                for batch in 1..8u64 {
                    let mut remaining = existing;
                    let mut released = Vec::new();
                    let mut steps = 0;
                    loop {
                        let (kept, final_batch) = next_tail_batch(remaining, desired, batch);
                        released.extend(kept..remaining);
                        steps += 1;
                        assert!(steps <= 20, "分批必须终止");
                        remaining = kept;
                        if final_batch {
                            break;
                        }
                    }
                    released.sort_unstable();
                    assert_eq!(
                        released,
                        (desired.min(existing)..existing).collect::<Vec<_>>()
                    );
                    assert_eq!(
                        steps,
                        existing.saturating_sub(desired).div_ceil(batch).max(1)
                    );
                }
            }
        }
    }

    #[test]
    fn intersections_match_enumerated_bytes_including_empty_and_reversed_ranges() {
        for write_start in 0..8 {
            for write_end in 0..8 {
                for unit_start in 0..8 {
                    for unit_end in 0..8 {
                        let expected = (write_start..write_end)
                            .filter(|byte| (unit_start..unit_end).contains(byte))
                            .collect::<Vec<_>>();
                        match bytes_in_unit(write_start, write_end, unit_start, unit_end) {
                            None => assert!(expected.is_empty()),
                            Some((in_unit, in_write, length)) => {
                                assert!(length > 0);
                                let actual = (0..length)
                                    .map(|i| unit_start + in_unit + i)
                                    .collect::<Vec<_>>();
                                assert_eq!(actual, expected);
                                assert_eq!(write_start + in_write, expected[0]);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn full_width_endpoints_do_not_wrap() {
        witness_file_range_boundaries();
        assert_eq!(next_tail_batch(u64::MAX, 0, u64::MAX), (0, true));
        assert_eq!(
            next_tail_batch(u64::MAX, u64::MAX, u64::MAX),
            (u64::MAX, true)
        );
        assert_eq!(
            bytes_in_unit(0, u64::MAX, 0, u64::MAX),
            Some((0, 0, u64::MAX))
        );
        assert_eq!(
            bytes_in_unit(u64::MAX - 1, u64::MAX, 1, u64::MAX),
            Some((u64::MAX - 2, 0, 1))
        );
    }
}

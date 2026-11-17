//! 部分写的纯计划核：单元来源、重写起点和写后单元数由 core 直接调用。
//! 证明计划决策与固定长度覆盖写的抽象字节规律；实际读写到字节模型的精化仍由调用方负责。
use crate::file_range_proofs::units_covering_size;
use vstd::prelude::*;

verus! {

/// 定位任意文件字节所在的单元；区间端点在规格中按数学整数计算，不要求尾端点装得进 u64。
pub fn unit_containing_byte(byte: u64, capacity: u64) -> (index: u64)
    requires capacity > 0,
    ensures
        index == byte / capacity,
        index <= byte,
        index * capacity <= byte < (index + 1) * capacity,
{
    proof {
        vstd::arithmetic::div_mod::lemma_div_nonincreasing(byte as int, capacity as int);
        vstd::arithmetic::div_mod::lemma_fundamental_div_mod(byte as int, capacity as int);
    }
    let index = byte / capacity;
    proof {
        vstd::arithmetic::mul::lemma_mul_is_commutative(index as int, capacity as int);
        assert(index * capacity <= byte < (index + 1) * capacity) by (nonlinear_arith)
            requires
                capacity > 0,
                byte == index * capacity + byte % capacity,
                0 <= byte % capacity < capacity,
        {}
    }
    index
}

pub enum PartialWriteUnitSource {
    OldReadBack,
    OldCoveredWhole,
    NewReadResidue,
    NewCoveredWhole,
    ZeroGap,
}

pub open spec fn byte_in_range(byte: int, start: u64, end: u64) -> bool {
    start <= byte < end
}

pub open spec fn unit_is_touched(write_start: u64, write_end: u64, unit_start: u64, unit_end: u64) -> bool {
    write_start < unit_end && unit_start < write_end
}

pub open spec fn unit_is_covered(write_start: u64, write_end: u64, unit_start: u64, unit_end: u64) -> bool {
    write_start <= unit_start && unit_end <= write_end
}

/// 旧单元没盖满就读旧数据；新单元只在被部分覆盖时读残留；未写到的新单元补零。
pub fn classify_partial_write_unit(has_old: bool, write_start: u64, write_end: u64, unit_start: u64, unit_end: u64)
    -> (answer: PartialWriteUnitSource)
    requires write_start < write_end, unit_start < unit_end,
    ensures
        answer is OldReadBack <==> has_old && !unit_is_covered(write_start, write_end, unit_start, unit_end),
        answer is OldCoveredWhole <==> has_old && unit_is_covered(write_start, write_end, unit_start, unit_end),
        answer is NewReadResidue <==> !has_old && unit_is_touched(write_start, write_end, unit_start, unit_end) && !unit_is_covered(write_start, write_end, unit_start, unit_end),
        answer is NewCoveredWhole <==> !has_old && unit_is_covered(write_start, write_end, unit_start, unit_end),
        answer is ZeroGap <==> !has_old && !unit_is_touched(write_start, write_end, unit_start, unit_end),
{
    let touched = write_start < unit_end && unit_start < write_end;
    let whole = write_start <= unit_start && unit_end <= write_end;
    if has_old {
        if whole { PartialWriteUnitSource::OldCoveredWhole } else { PartialWriteUnitSource::OldReadBack }
    } else if !touched {
        PartialWriteUnitSource::ZeroGap
    } else if whole {
        PartialWriteUnitSource::NewCoveredWhole
    } else {
        PartialWriteUnitSource::NewReadResidue
    }
}

/// 非空区间的端点判定等价于逐字节“全部覆盖”；部分覆盖不能被误判为免读旧数据。
pub proof fn lemma_whole_coverage_means_every_unit_byte_is_written(write_start: u64, write_end: u64, unit_start: u64, unit_end: u64)
    requires write_start < write_end, unit_start < unit_end,
    ensures
        unit_is_covered(write_start, write_end, unit_start, unit_end) <==>
            (forall|byte: int| #[trigger] byte_in_range(byte, unit_start, unit_end) ==> byte_in_range(byte, write_start, write_end)),
{
    if !unit_is_covered(write_start, write_end, unit_start, unit_end) {
        if write_start > unit_start {
            assert(byte_in_range(unit_start as int, unit_start, unit_end));
            assert(!byte_in_range(unit_start as int, write_start, write_end));
        } else {
            assert(byte_in_range(unit_end - 1, unit_start, unit_end));
            assert(!byte_in_range(unit_end - 1, write_start, write_end));
        }
    }
}

/// 相交判定恰好表示存在共同字节；相邻端点不算相交。
pub proof fn lemma_touched_unit_has_exactly_a_nonempty_intersection(write_start: u64, write_end: u64, unit_start: u64, unit_end: u64)
    requires write_start < write_end, unit_start < unit_end,
    ensures
        unit_is_touched(write_start, write_end, unit_start, unit_end) <==>
            (exists|byte: int| #[trigger] byte_in_range(byte, unit_start, unit_end) && byte_in_range(byte, write_start, write_end)),
{
    if unit_is_touched(write_start, write_end, unit_start, unit_end) {
        let byte = if write_start > unit_start { write_start as int } else { unit_start as int };
        assert(byte_in_range(byte, unit_start, unit_end) && byte_in_range(byte, write_start, write_end));
    }
}

/// 固定长度文件内的写入；扩展文件、补洞与 I/O 不在这个字节模型中。
pub open spec fn overwrite_fits(old: Seq<u8>, start: int, data: Seq<u8>) -> bool {
    0 <= start && start + data.len() <= old.len()
}

pub open spec fn overwritten_bytes(old: Seq<u8>, start: int, data: Seq<u8>) -> Seq<u8> {
    Seq::new(old.len(), |byte: int|
        if start <= byte < start + data.len() { data[byte - start] } else { old[byte] })
}

/// 写内每个字节来自请求，写外每个字节仍是原值；空写也包括在内。
pub proof fn lemma_overwrite_changes_exactly_the_requested_bytes(old: Seq<u8>, start: int, data: Seq<u8>)
    requires overwrite_fits(old, start, data),
    ensures
        overwritten_bytes(old, start, data).len() == old.len(),
        forall|byte: int| 0 <= byte < old.len() ==>
            #[trigger] overwritten_bytes(old, start, data)[byte] ==
                if start <= byte < start + data.len() { data[byte - start] } else { old[byte] },
        data.len() == 0 ==> overwritten_bytes(old, start, data) == old,
{
    if data.len() == 0 {
        assert(overwritten_bytes(old, start, data) =~= old);
    }
}

/// 同一段写两次，后一次完全取代前一次；不是只证明最后一个字节。
pub proof fn lemma_overwriting_the_same_range_keeps_only_the_last_write(
    old: Seq<u8>, start: int, first: Seq<u8>, last: Seq<u8>,
)
    requires
        overwrite_fits(old, start, first),
        first.len() == last.len(),
    ensures
        overwrite_fits(overwritten_bytes(old, start, first), start, last),
        overwritten_bytes(overwritten_bytes(old, start, first), start, last)
            == overwritten_bytes(old, start, last),
{
    assert(overwritten_bytes(overwritten_bytes(old, start, first), start, last)
        =~= overwritten_bytes(old, start, last));
}

/// 不相交的写可以交换顺序，包括相邻区间与空写；重叠写没有这个性质。
pub proof fn lemma_disjoint_overwrites_commute(
    old: Seq<u8>, first_start: int, first: Seq<u8>, second_start: int, second: Seq<u8>,
)
    requires
        overwrite_fits(old, first_start, first),
        overwrite_fits(old, second_start, second),
        first_start + first.len() <= second_start || second_start + second.len() <= first_start,
    ensures
        overwrite_fits(overwritten_bytes(old, first_start, first), second_start, second),
        overwrite_fits(overwritten_bytes(old, second_start, second), first_start, first),
        overwritten_bytes(overwritten_bytes(old, first_start, first), second_start, second)
            == overwritten_bytes(overwritten_bytes(old, second_start, second), first_start, first),
{
    assert(overwritten_bytes(overwritten_bytes(old, first_start, first), second_start, second)
        =~= overwritten_bytes(overwritten_bytes(old, second_start, second), first_start, first));
}

/// 分类核的“整单元覆盖”条件蕴涵该单元的结果不依赖旧内容。
pub proof fn lemma_whole_unit_overwrite_does_not_depend_on_old_bytes(
    left: Seq<u8>, right: Seq<u8>, data: Seq<u8>, write_start: u64, write_end: u64, unit_start: u64, unit_end: u64,
)
    requires
        write_start < write_end, unit_start < unit_end,
        left.len() == right.len(),
        overwrite_fits(left, write_start as int, data),
        data.len() == write_end - write_start,
        unit_is_covered(write_start, write_end, unit_start, unit_end),
    ensures
        forall|byte: int| #[trigger] byte_in_range(byte, unit_start, unit_end) ==>
            0 <= byte < left.len()
            && overwritten_bytes(left, write_start as int, data)[byte]
                == overwritten_bytes(right, write_start as int, data)[byte],
{
    lemma_whole_coverage_means_every_unit_byte_is_written(write_start, write_end, unit_start, unit_end);
}

/// 非空见证与重叠写反例：以上前提可满足，交换律确实需要不相交条件。
pub proof fn witness_overwrite_laws_and_overlapping_write_order()
    ensures
        overwrite_fits(seq![1u8, 2, 3, 4], 1, seq![8u8, 9]),
        overwritten_bytes(seq![1u8, 2, 3, 4], 1, seq![8u8, 9]) == seq![1u8, 8, 9, 4],
        overwritten_bytes(seq![1u8, 2, 3, 4], 4, Seq::<u8>::empty()) == seq![1u8, 2, 3, 4],
        overwritten_bytes(overwritten_bytes(seq![0u8, 0], 0, seq![1u8, 1]), 1, seq![2u8])
            != overwritten_bytes(overwritten_bytes(seq![0u8, 0], 1, seq![2u8]), 0, seq![1u8, 1]),
{
    let old = seq![1u8, 2, 3, 4];
    lemma_overwrite_changes_exactly_the_requested_bytes(old, 1, seq![8u8, 9]);
    lemma_overwrite_changes_exactly_the_requested_bytes(old, 4, Seq::<u8>::empty());
    lemma_overwriting_the_same_range_keeps_only_the_last_write(old, 1, seq![5u8, 6], seq![8u8, 9]);
    lemma_disjoint_overwrites_commute(old, 0, seq![7u8], 1, seq![8u8, 9]);
    lemma_whole_unit_overwrite_does_not_depend_on_old_bytes(old, seq![5u8, 6, 7, 8], seq![8u8, 9], 1, 3, 1, 3);
    assert(overwritten_bytes(old, 1, seq![8u8, 9]) =~= seq![1u8, 8, 9, 4]);
    assert(overwritten_bytes(overwritten_bytes(seq![0u8, 0], 0, seq![1u8, 1]), 1, seq![2u8])[1] == 2);
    assert(overwritten_bytes(overwritten_bytes(seq![0u8, 0], 1, seq![2u8]), 0, seq![1u8, 1])[1] == 1);
}

/// 分割点可落在请求的任一端；两段的边界字节既不能遗漏，也不能重复使用错误的偏移。
pub proof fn lemma_split_overwrite_equals_one_write(
    original_bytes: Seq<u8>, write_start: int, written_bytes: Seq<u8>, split_offset: int,
)
    requires
        overwrite_fits(original_bytes, write_start, written_bytes),
        0 <= split_offset <= written_bytes.len(),
    ensures
        overwrite_fits(original_bytes, write_start, written_bytes.subrange(0, split_offset)),
        overwrite_fits(
            overwritten_bytes(original_bytes, write_start, written_bytes.subrange(0, split_offset)),
            write_start + split_offset, written_bytes.subrange(split_offset, written_bytes.len() as int)),
        overwritten_bytes(
            overwritten_bytes(original_bytes, write_start, written_bytes.subrange(0, split_offset)),
            write_start + split_offset, written_bytes.subrange(split_offset, written_bytes.len() as int))
            == overwritten_bytes(original_bytes, write_start, written_bytes),
{
    let leading_bytes = written_bytes.subrange(0, split_offset);
    let trailing_bytes = written_bytes.subrange(split_offset, written_bytes.len() as int);
    let after_leading_write = overwritten_bytes(original_bytes, write_start, leading_bytes);
    let after_split_write = overwritten_bytes(after_leading_write, write_start + split_offset, trailing_bytes);
    let after_whole_write = overwritten_bytes(original_bytes, write_start, written_bytes);
    assert forall|byte: int| 0 <= byte < original_bytes.len() implies
        #[trigger] after_split_write[byte] == after_whole_write[byte] by {
        if write_start <= byte < write_start + split_offset {
            assert(leading_bytes[byte - write_start] == written_bytes[byte - write_start]);
        } else if write_start + split_offset <= byte < write_start + written_bytes.len() {
            assert(trailing_bytes[byte - (write_start + split_offset)] == written_bytes[byte - write_start]);
        }
    }
    assert(after_split_write =~= after_whole_write);
}

/// 不要求两段互斥或等长；交集取后写字节，只被先写覆盖的字节保留先写值，其余保留原值。
pub proof fn lemma_overlapping_overwrites_keep_the_latest_byte(
    original_bytes: Seq<u8>, earlier_start: int, earlier_bytes: Seq<u8>, later_start: int, later_bytes: Seq<u8>,
)
    requires
        overwrite_fits(original_bytes, earlier_start, earlier_bytes),
        overwrite_fits(original_bytes, later_start, later_bytes),
    ensures
        overwrite_fits(overwritten_bytes(original_bytes, earlier_start, earlier_bytes), later_start, later_bytes),
        overwritten_bytes(overwritten_bytes(original_bytes, earlier_start, earlier_bytes), later_start, later_bytes).len()
            == original_bytes.len(),
        forall|byte: int| 0 <= byte < original_bytes.len()
            && later_start <= byte < later_start + later_bytes.len() ==>
            #[trigger] overwritten_bytes(overwritten_bytes(original_bytes, earlier_start, earlier_bytes), later_start, later_bytes)[byte]
                == later_bytes[byte - later_start],
        forall|byte: int| 0 <= byte < original_bytes.len()
            && !(later_start <= byte < later_start + later_bytes.len())
            && earlier_start <= byte < earlier_start + earlier_bytes.len() ==>
            #[trigger] overwritten_bytes(overwritten_bytes(original_bytes, earlier_start, earlier_bytes), later_start, later_bytes)[byte]
                == earlier_bytes[byte - earlier_start],
        forall|byte: int| 0 <= byte < original_bytes.len()
            && !(later_start <= byte < later_start + later_bytes.len())
            && !(earlier_start <= byte < earlier_start + earlier_bytes.len()) ==>
            #[trigger] overwritten_bytes(overwritten_bytes(original_bytes, earlier_start, earlier_bytes), later_start, later_bytes)[byte]
                == original_bytes[byte],
{
    lemma_overwrite_changes_exactly_the_requested_bytes(original_bytes, earlier_start, earlier_bytes);
    lemma_overwrite_changes_exactly_the_requested_bytes(
        overwritten_bytes(original_bytes, earlier_start, earlier_bytes), later_start, later_bytes);
}

/// 用具体字节钉住分割端点和重叠优先级，避免只让两个抽象表达式互相作证。
pub proof fn witness_split_and_overlapping_overwrite_bytes()
    ensures
        overwrite_fits(seq![1u8, 2, 3, 4, 5, 6], 1, seq![7u8, 8, 9]),
        overwritten_bytes(overwritten_bytes(seq![1u8, 2, 3, 4, 5, 6], 1, seq![7u8]), 2, seq![8u8, 9])
            == seq![1u8, 7, 8, 9, 5, 6],
        overwritten_bytes(overwritten_bytes(seq![1u8, 2, 3, 4, 5, 6], 1, Seq::<u8>::empty()), 1, seq![7u8, 8, 9])
            == seq![1u8, 7, 8, 9, 5, 6],
        overwritten_bytes(overwritten_bytes(seq![1u8, 2, 3, 4, 5, 6], 1, seq![7u8, 8, 9]), 4, Seq::<u8>::empty())
            == seq![1u8, 7, 8, 9, 5, 6],
        overwritten_bytes(overwritten_bytes(seq![1u8, 2, 3, 4, 5, 6], 1, seq![7u8, 8, 9]), 2, seq![4u8, 5])
            == seq![1u8, 7, 4, 5, 5, 6],
        overwritten_bytes(overwritten_bytes(seq![1u8, 2, 3, 4, 5, 6], 1, seq![7u8]), 4, seq![9u8])
            == seq![1u8, 7, 3, 4, 9, 6],
{
    let original_bytes = seq![1u8, 2, 3, 4, 5, 6];
    let written_bytes = seq![7u8, 8, 9];
    lemma_split_overwrite_equals_one_write(original_bytes, 1, written_bytes, 1);
    lemma_split_overwrite_equals_one_write(original_bytes, 1, written_bytes, 0);
    lemma_split_overwrite_equals_one_write(original_bytes, 1, written_bytes, 3);
    lemma_overlapping_overwrites_keep_the_latest_byte(original_bytes, 1, written_bytes, 2, seq![4u8, 5]);
    lemma_overlapping_overwrites_keep_the_latest_byte(original_bytes, 1, seq![7u8], 4, seq![9u8]);
    assert(written_bytes.subrange(0, 1) =~= seq![7u8]);
    assert(written_bytes.subrange(1, 3) =~= seq![8u8, 9]);
    assert(written_bytes.subrange(0, 0) =~= Seq::<u8>::empty());
    assert(written_bytes.subrange(0, 3) =~= written_bytes);
    assert(written_bytes.subrange(3, 3) =~= Seq::<u8>::empty());
    assert(overwritten_bytes(original_bytes, 1, written_bytes) =~= seq![1u8, 7, 8, 9, 5, 6]);
    assert(overwritten_bytes(overwritten_bytes(original_bytes, 1, written_bytes), 2, seq![4u8, 5])
        =~= seq![1u8, 7, 4, 5, 5, 6]);
    assert(overwritten_bytes(overwritten_bytes(original_bytes, 1, seq![7u8]), 4, seq![9u8])
        =~= seq![1u8, 7, 3, 4, 9, 6]);
}

/// 重写从写入起点开始；跨过已有尾部时，未满的旧尾单元也必须重写。
pub fn first_unit_to_rewrite(first_written: u64, existing: u64, tail_is_partial: bool) -> (first: u64)
    ensures
        first <= existing,
        first <= first_written,
        first == if first_written < existing { first_written as int }
            else if existing > 0 && tail_is_partial { existing - 1 } else { existing as int },
{
    if first_written < existing { first_written }
    else if existing > 0 && tail_is_partial { existing - 1 }
    else { existing }
}

/// 写后的单元数同时保留已有单元并包含写入终点之前的全部单元。
pub fn units_after_partial_write(existing: u64, write_end: u64, capacity: u64) -> (after: u64)
    requires write_end > 0, capacity > 0,
    ensures
        after == if existing > crate::file_range_proofs::covering_units(write_end, capacity) { existing as int }
            else { crate::file_range_proofs::covering_units(write_end, capacity) },
        after >= existing,
        after >= 1,
        write_end <= after * capacity,
{
    let needed = units_covering_size(write_end, capacity);
    if existing > needed {
        proof {
            vstd::arithmetic::mul::lemma_mul_inequality(needed as int, existing as int, capacity as int);
        }
        existing
    } else { needed }
}

} // verus!

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn located_unit_contains_the_byte_even_when_its_end_exceeds_u64() {
        for capacity in [1u64, 2, 3, 32_634, u64::MAX] {
            for byte in [
                0,
                1,
                capacity - 1,
                capacity,
                capacity.saturating_add(1),
                u64::MAX - 1,
                u64::MAX,
            ] {
                let index = unit_containing_byte(byte, capacity);
                let start = u128::from(index) * u128::from(capacity);
                let end = start + u128::from(capacity);
                assert!(start <= u128::from(byte) && u128::from(byte) < end);
            }
        }
        assert_eq!(unit_containing_byte(32_633, 32_634), 0);
        assert_eq!(unit_containing_byte(32_634, 32_634), 1);
        assert_eq!(unit_containing_byte(u64::MAX, 1), u64::MAX);
        assert_eq!(unit_containing_byte(u64::MAX, u64::MAX), 1);
    }

    #[test]
    fn unit_sources_match_the_bytes_actually_overwritten() {
        let mut seen = [false; 5];
        for write_start in 0..7 {
            for write_end in write_start + 1..9 {
                for unit_start in 0..7 {
                    for unit_end in unit_start + 1..9 {
                        let written = (unit_start..unit_end)
                            .filter(|byte| (write_start..write_end).contains(byte))
                            .count() as u64;
                        for old in [false, true] {
                            let (index, reads_old, reads_residue, gap, whole) =
                                match classify_partial_write_unit(
                                    old,
                                    write_start,
                                    write_end,
                                    unit_start,
                                    unit_end,
                                ) {
                                    PartialWriteUnitSource::OldReadBack => {
                                        (0, true, false, false, false)
                                    }
                                    PartialWriteUnitSource::OldCoveredWhole => {
                                        (1, false, false, false, true)
                                    }
                                    PartialWriteUnitSource::NewReadResidue => {
                                        (2, false, true, false, false)
                                    }
                                    PartialWriteUnitSource::NewCoveredWhole => {
                                        (3, false, false, false, true)
                                    }
                                    PartialWriteUnitSource::ZeroGap => {
                                        (4, false, false, true, false)
                                    }
                                };
                            seen[index] = true;
                            assert_eq!(reads_old, old && written < unit_end - unit_start);
                            assert_eq!(
                                reads_residue,
                                !old && 0 < written && written < unit_end - unit_start
                            );
                            assert_eq!(gap, !old && written == 0);
                            assert_eq!(whole, written == unit_end - unit_start);
                        }
                    }
                }
            }
        }
        assert_eq!(seen, [true; 5]);
    }

    #[test]
    fn rewrite_start_includes_the_gap_and_the_partial_old_tail() {
        for existing in 0..12u64 {
            for first_written in 0..16u64 {
                for tail_partial in [false, true] {
                    let mut rewritten = vec![first_written];
                    rewritten.extend(existing..first_written);
                    if existing > 0 && tail_partial && first_written >= existing {
                        rewritten.push(existing - 1);
                    }
                    assert_eq!(
                        first_unit_to_rewrite(first_written, existing, tail_partial),
                        *rewritten.iter().min().unwrap()
                    );
                }
            }
        }
        assert_eq!(first_unit_to_rewrite(0, 0, true), 0);
        assert_eq!(
            first_unit_to_rewrite(u64::MAX, u64::MAX, true),
            u64::MAX - 1
        );
        assert_eq!(first_unit_to_rewrite(u64::MAX, u64::MAX, false), u64::MAX);
    }

    #[test]
    fn after_count_preserves_old_units_and_covers_the_write_at_full_width() {
        for existing in [0u64, 1, 10, u64::MAX] {
            for end in [1u64, 2, 32_634, 32_635, u64::MAX] {
                for capacity in [1u64, 2, 32_634, u64::MAX] {
                    let needed = u128::from(end).div_ceil(u128::from(capacity));
                    assert_eq!(
                        u128::from(units_after_partial_write(existing, end, capacity)),
                        u128::from(existing).max(needed)
                    );
                }
            }
        }
    }
}

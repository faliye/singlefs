//! 部分写的纯计划核：单元来源、重写起点和写后单元数由 core 直接调用。
//! 这里只证明决策；单元端点换算、旧尾单元是否不满的判定及实际读写由调用方负责。
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

pub open spec fn unit_is_touched(ws: u64, we: u64, us: u64, ue: u64) -> bool {
    ws < ue && us < we
}

pub open spec fn unit_is_covered(ws: u64, we: u64, us: u64, ue: u64) -> bool {
    ws <= us && ue <= we
}

/// 旧单元没盖满就读旧数据；新单元只在被部分覆盖时读残留；未写到的新单元补零。
pub fn classify_partial_write_unit(has_old: bool, ws: u64, we: u64, us: u64, ue: u64)
    -> (answer: PartialWriteUnitSource)
    requires ws < we, us < ue,
    ensures
        answer is OldReadBack <==> has_old && !unit_is_covered(ws, we, us, ue),
        answer is OldCoveredWhole <==> has_old && unit_is_covered(ws, we, us, ue),
        answer is NewReadResidue <==> !has_old && unit_is_touched(ws, we, us, ue) && !unit_is_covered(ws, we, us, ue),
        answer is NewCoveredWhole <==> !has_old && unit_is_covered(ws, we, us, ue),
        answer is ZeroGap <==> !has_old && !unit_is_touched(ws, we, us, ue),
{
    let touched = ws < ue && us < we;
    let whole = ws <= us && ue <= we;
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
pub proof fn lemma_whole_coverage_means_every_unit_byte_is_written(ws: u64, we: u64, us: u64, ue: u64)
    requires ws < we, us < ue,
    ensures
        unit_is_covered(ws, we, us, ue) <==>
            (forall|byte: int| #[trigger] byte_in_range(byte, us, ue) ==> byte_in_range(byte, ws, we)),
{
    if !unit_is_covered(ws, we, us, ue) {
        if ws > us {
            assert(byte_in_range(us as int, us, ue));
            assert(!byte_in_range(us as int, ws, we));
        } else {
            assert(byte_in_range(ue - 1, us, ue));
            assert(!byte_in_range(ue - 1, ws, we));
        }
    }
}

/// 相交判定恰好表示存在共同字节；相邻端点不算相交。
pub proof fn lemma_touched_unit_has_exactly_a_nonempty_intersection(ws: u64, we: u64, us: u64, ue: u64)
    requires ws < we, us < ue,
    ensures
        unit_is_touched(ws, we, us, ue) <==>
            (exists|byte: int| #[trigger] byte_in_range(byte, us, ue) && byte_in_range(byte, ws, we)),
{
    if unit_is_touched(ws, we, us, ue) {
        let byte = if ws > us { ws as int } else { us as int };
        assert(byte_in_range(byte, us, ue) && byte_in_range(byte, ws, we));
    }
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
        for ws in 0..7 {
            for we in ws + 1..9 {
                for us in 0..7 {
                    for ue in us + 1..9 {
                        let written =
                            (us..ue).filter(|byte| (ws..we).contains(byte)).count() as u64;
                        for old in [false, true] {
                            let (index, reads_old, reads_residue, gap, whole) =
                                match classify_partial_write_unit(old, ws, we, us, ue) {
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
                            assert_eq!(reads_old, old && written < ue - us);
                            assert_eq!(reads_residue, !old && 0 < written && written < ue - us);
                            assert_eq!(gap, !old && written == 0);
                            assert_eq!(whole, written == ue - us);
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

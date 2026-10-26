//! 搬迁的两道判定，写在 core 里、由 Verus 直接验（第二轮正推腿的第一阶段最小原型）。
//!
//! 与第一轮把核放在 `singlefs-replay-prefix` 里不同：这里的函数住在 `singlefs-core`，类型用 core 自己的 `Placement`、`SlotNumber`
//! （这两个类型的定义挪进了 `verus!`，字段与派生不变），调用方 `transaction::relocation_checked_against_the_version` 直接调这里的函数。
//! 目的地判定是 P4 区间版（`relocation_interval_proofs.rs`）在 u64 上的可执行形：区间 [起点, 起点 + 跨度) 两两不重叠、
//! 与受保护落点不重叠；前提要求每个落点的 `起点 + 跨度` 装得进 u64（接 core 的实际取值：槽号至多 6 字节）。
//! 条目判定把「两份副本都在计划源槽、跨度等于角色跨度」与「字节非空」两道判定写成可执行函数。
use vstd::prelude::*;

use crate::allocator::Placement;

verus! {

pub open spec fn placement_end(placement: Placement) -> int {
    placement.slot.0 as int + placement.span as int
}

pub open spec fn placements_overlap_spec(a: Placement, b: Placement) -> bool {
    a.slot.0 < placement_end(b) && b.slot.0 < placement_end(a)
}

pub open spec fn placement_is_bounded(placement: Placement) -> bool {
    placement.span >= 1 && placement_end(placement) <= u64::MAX
}

pub fn placements_overlap(a: &Placement, b: &Placement) -> (overlap: bool)
    requires
        placement_is_bounded(*a),
        placement_is_bounded(*b),
    ensures
        overlap == placements_overlap_spec(*a, *b),
{
    a.slot.0 < b.slot.0 + b.span && b.slot.0 < a.slot.0 + a.span
}

pub open spec fn disjoint_from_all(candidate: Placement, others: Seq<Placement>) -> bool {
    forall|k: int| 0 <= k < others.len() ==> !placements_overlap_spec(candidate, #[trigger] others[k])
}

pub open spec fn pairwise_disjoint(runs: Seq<Placement>) -> bool {
    forall|i: int, j: int|
        #![trigger runs[i], runs[j]]
        0 <= i < j < runs.len() ==> !placements_overlap_spec(runs[i], runs[j])
}

pub open spec fn all_bounded(runs: Seq<Placement>) -> bool {
    forall|i: int| 0 <= i < runs.len() ==> placement_is_bounded(#[trigger] runs[i])
}

pub fn disjoint_from_every(candidate: &Placement, others: &Vec<Placement>) -> (disjoint: bool)
    requires
        placement_is_bounded(*candidate),
        all_bounded(others@),
    ensures
        disjoint == disjoint_from_all(*candidate, others@),
{
    let mut k: usize = 0;
    while k < others.len()
        invariant
            0 <= k <= others.len(),
            placement_is_bounded(*candidate),
            all_bounded(others@),
            forall|m: int| 0 <= m < k ==> !placements_overlap_spec(*candidate, #[trigger] others@[m]),
        decreases others.len() - k,
    {
        if placements_overlap(candidate, &others[k]) {
            return false;
        }
        k += 1;
    }
    true
}

pub fn runs_are_pairwise_disjoint(runs: &Vec<Placement>) -> (disjoint: bool)
    requires
        all_bounded(runs@),
    ensures
        disjoint == pairwise_disjoint(runs@),
{
    let mut i: usize = 0;
    while i < runs.len()
        invariant
            0 <= i <= runs.len(),
            all_bounded(runs@),
            forall|a: int, b: int| #![trigger runs@[a], runs@[b]] 0 <= a < b < runs.len() && a < i ==> !placements_overlap_spec(runs@[a], runs@[b]),
        decreases runs.len() - i,
    {
        let mut j: usize = i + 1;
        while j < runs.len()
            invariant
                0 <= i < runs.len(),
                i + 1 <= j <= runs.len(),
                all_bounded(runs@),
                forall|a: int, b: int| #![trigger runs@[a], runs@[b]] 0 <= a < b < runs.len() && a < i ==> !placements_overlap_spec(runs@[a], runs@[b]),
                forall|b: int| #![trigger runs@[b]] i < b < j ==> !placements_overlap_spec(runs@[i as int], runs@[b]),
            decreases runs.len() - j,
        {
            if placements_overlap(&runs[i], &runs[j]) {
                return false;
            }
            j += 1;
        }
        i += 1;
    }
    true
}

// 一次发布的目的地判定：目的落点两两不重叠，且都不压住受保护的落点（映射里每个单元的落点、其余视图引用的落点）。
pub fn relocation_destinations_are_acceptable(
    destinations: &Vec<Placement>,
    protected: &Vec<Placement>,
) -> (acceptable: bool)
    requires
        all_bounded(destinations@),
        all_bounded(protected@),
    ensures
        acceptable == (pairwise_disjoint(destinations@) && forall|i: int|
            0 <= i < destinations.len() ==> disjoint_from_all(#[trigger] destinations@[i], protected@)),
{
    if !runs_are_pairwise_disjoint(destinations) {
        return false;
    }
    let mut i: usize = 0;
    while i < destinations.len()
        invariant
            0 <= i <= destinations.len(),
            all_bounded(destinations@),
            all_bounded(protected@),
            forall|m: int| 0 <= m < i ==> disjoint_from_all(#[trigger] destinations@[m], protected@),
        decreases destinations.len() - i,
    {
        if !disjoint_from_every(&destinations[i], protected) {
            return false;
        }
        i += 1;
    }
    true
}

// 条目判定的后半：两份副本的槽号都等于计划源槽，且计划源的跨度等于这个角色的跨度。
pub fn copies_sit_at_the_planned_source(
    first_copy_slot: u64,
    second_copy_slot: u64,
    planned_source_slot: u64,
    planned_source_span: u64,
    role_span: u64,
) -> (sits: bool)
    ensures
        sits == (first_copy_slot == planned_source_slot && second_copy_slot == planned_source_slot
            && planned_source_span == role_span),
{
    first_copy_slot == planned_source_slot && second_copy_slot == planned_source_slot
        && planned_source_span == role_span
}

// 条目判定的末一道：取到的字节非空（空 = 两份副本的校验和都对不上或根本读不出）。
pub fn unit_bytes_are_available(length: usize) -> (available: bool)
    ensures
        available == (length > 0),
{
    length > 0
}

// 无前提的见证：两个有界落点，不重叠，判定函数的前提都满足；防止上面的 requires 在空集上空真。
pub proof fn witness_two_bounded_placements_are_disjoint()
    ensures
        ({
            let a = Placement { slot: crate::address::SlotNumber(10), span: 2 };
            let b = Placement { slot: crate::address::SlotNumber(12), span: 1 };
            placement_is_bounded(a) && placement_is_bounded(b) && !placements_overlap_spec(a, b)
        }),
{
}

} // verus!

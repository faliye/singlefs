//! unlink 的分批计划（D8 已定项 16 第 2、6、7 条）：core 的 `unlink::next_unlink_step` 直接调用这里的可执行核。
//! 证明的是「还剩几个单元 → 下一步」这一条纯函数与它迭代出的整串计划：每批恰好截掉尾部一批、最后一批释放剩下全部、
//! 走到底的步数、每个数据单元恰好被释放一次、从任一中间版续做的计划与不崩溃一路做下来的后半截逐步相同（第 7 条 ①「计划相等」），
//! 以及删除批的下段节点三分（照抄 / 重写 / 换下）在每一层至多重写一个、就是罩着最后一个留下单元与第一个被截单元的那一个（K-R3-1）。
//! 不证明：单元数从 extent 树怎么数出来、净荷容量的取值、`plan_the_lower_segment_after_a_deletion_batch` 的 Vec 构造与发布 I/O；
//! 下段节点那条引理是按「第 index 个节点罩 [index × span, index × span + span)」的模型证的，跨度由调用方提供。
use vstd::prelude::*;

use crate::file_range_proofs::next_tail_batch;

verus! {

/// 计划里的一步（只在规格里用）：非最后一批记「这一批之前剩几个、之后留几个」，最后一批记「释放几个」。
pub enum UnlinkPlanStep {
    Batch { before: nat, kept: nat },
    Final { released: nat },
}

/// 这一步之前这个文件还剩几个单元。
pub open spec fn step_before(step: UnlinkPlanStep) -> nat {
    match step {
        UnlinkPlanStep::Batch { before, kept } => before,
        UnlinkPlanStep::Final { released } => released,
    }
}

/// 这一步之后还剩几个单元（最后一批之后是 0）。
pub open spec fn step_after(step: UnlinkPlanStep) -> nat {
    match step {
        UnlinkPlanStep::Batch { before, kept } => kept,
        UnlinkPlanStep::Final { released } => 0,
    }
}

/// 这一步释放了单元 `unit`：释放的恰好是 [之后剩下的, 之前剩下的)。
pub open spec fn released_by(step: UnlinkPlanStep, unit: nat) -> bool {
    step_after(step) <= unit < step_before(step)
}

/// 从还剩 `remaining` 个单元起走到底的整串计划：剩下的不多于一批 ⇒ 只剩最后一批；否则截掉尾部一批再从留下的起算。
/// 批大小是 0 时没有定义（交回空串），产品与测试的批大小都至少 1。
pub open spec fn unlink_plan(remaining: nat, batch: nat) -> Seq<UnlinkPlanStep>
    decreases remaining,
{
    if batch == 0 {
        Seq::empty()
    } else if remaining <= batch {
        seq![UnlinkPlanStep::Final { released: remaining }]
    } else {
        seq![UnlinkPlanStep::Batch { before: remaining, kept: (remaining - batch) as nat }]
            + unlink_plan((remaining - batch) as nat, batch)
    }
}

/// unlink 意图之后的下一步。返回（这一步之后留下的单元数，是不是最后一批）；就是整串计划的第一步。
pub fn next_unlink_batch(data_units_remaining: u64, batch: u64) -> (answer: (u64, bool))
    requires
        batch > 0,
    ensures
        answer.1 == (data_units_remaining <= batch),
        answer.1 ==> answer.0 == 0,
        !answer.1 ==> 1 <= answer.0 < data_units_remaining,
        !answer.1 ==> data_units_remaining - answer.0 == batch,
        unlink_plan(data_units_remaining as nat, batch as nat).len() >= 1,
        answer.1 ==> unlink_plan(data_units_remaining as nat, batch as nat)[0]
            == (UnlinkPlanStep::Final { released: data_units_remaining as nat }),
        !answer.1 ==> unlink_plan(data_units_remaining as nat, batch as nat)[0]
            == (UnlinkPlanStep::Batch { before: data_units_remaining as nat, kept: answer.0 as nat }),
{
    let (kept, final_batch) = next_tail_batch(data_units_remaining, 0, batch);
    (kept, final_batch)
}

/// 非最后一批之后 inode 记录的 size：留下的单元数 × 净荷容量；乘积装不进 u64 时交回 None（调用方据此停下，不回绕）。
pub fn size_after_an_unlink_batch(data_units_kept: u64, payload_in_bytes: u64) -> (answer: Option<u64>)
    ensures
        match answer {
            None => data_units_kept * payload_in_bytes > u64::MAX,
            Some(size) => size == data_units_kept * payload_in_bytes,
        },
{
    data_units_kept.checked_mul(payload_in_bytes)
}

/// 计划的形状：至少一步，最后一步恰是释放不多于一批的最后一批，前面每步都是截掉恰好一批、至少留一个的非最后一批，
/// 第一步从 `remaining` 起，每步从上一步留下的起，释放的单元都在起点之下。
pub proof fn lemma_unlink_plan_shape(remaining: nat, batch: nat)
    requires
        batch > 0,
    ensures
        unlink_plan(remaining, batch).len() >= 1,
        step_before(unlink_plan(remaining, batch)[0]) == remaining,
        unlink_plan(remaining, batch).last() is Final,
        step_before(unlink_plan(remaining, batch).last()) <= batch,
        forall|i: int| 0 <= i < unlink_plan(remaining, batch).len() - 1 ==> {
            &&& #[trigger] unlink_plan(remaining, batch)[i] is Batch
            &&& step_before(unlink_plan(remaining, batch)[i]) == step_after(unlink_plan(remaining, batch)[i]) + batch
            &&& step_after(unlink_plan(remaining, batch)[i]) >= 1
            &&& step_before(unlink_plan(remaining, batch)[i + 1]) == step_after(unlink_plan(remaining, batch)[i])
        },
        forall|i: int| 0 <= i < unlink_plan(remaining, batch).len() ==>
            step_before(#[trigger] unlink_plan(remaining, batch)[i]) <= remaining,
    decreases remaining,
{
    if remaining > batch {
        let rest = (remaining - batch) as nat;
        lemma_unlink_plan_shape(rest, batch);
        let head = seq![UnlinkPlanStep::Batch { before: remaining, kept: rest }];
        let tail = unlink_plan(rest, batch);
        let plan = unlink_plan(remaining, batch);
        assert(plan == head + tail);
        assert(plan.last() == tail.last());
        assert forall|i: int| 0 <= i < plan.len() - 1 implies {
            &&& #[trigger] plan[i] is Batch
            &&& step_before(plan[i]) == step_after(plan[i]) + batch
            &&& step_after(plan[i]) >= 1
            &&& step_before(plan[i + 1]) == step_after(plan[i])
        } by {
            if i > 0 {
                assert(plan[i] == tail[i - 1]);
                assert(plan[i + 1] == tail[i]);
            } else {
                assert(plan[1] == tail[0]);
            }
        }
        assert forall|i: int| 0 <= i < plan.len() implies step_before(#[trigger] plan[i]) <= remaining by {
            if i > 0 {
                assert(plan[i] == tail[i - 1]);
            }
        }
    }
}

/// 步数：没有数据单元的文件只有一步，否则恰好 ⌈remaining ÷ batch⌉ 步（`(步数 − 1) × batch < remaining ≤ 步数 × batch`）。
pub proof fn lemma_unlink_plan_length(remaining: nat, batch: nat)
    requires
        batch > 0,
    ensures
        remaining == 0 ==> unlink_plan(remaining, batch).len() == 1,
        remaining > 0 ==> (unlink_plan(remaining, batch).len() - 1) * batch < remaining,
        remaining > 0 ==> remaining <= unlink_plan(remaining, batch).len() * batch,
    decreases remaining,
{
    if remaining > batch {
        let rest = (remaining - batch) as nat;
        lemma_unlink_plan_length(rest, batch);
        let steps_after = unlink_plan(rest, batch).len();
        assert(unlink_plan(remaining, batch).len() == steps_after + 1);
        assert((steps_after + 1) * batch == steps_after * batch + batch) by (nonlinear_arith);
        // rest > 0，归纳前提给 (steps_after − 1) × batch < rest ≤ steps_after × batch；左式多一批就是 steps_after × batch < remaining。
        assert(steps_after >= 1);
        assert((steps_after - 1) * batch + batch == steps_after * batch) by (nonlinear_arith);
        assert(steps_after * batch < remaining);
        assert(unlink_plan(remaining, batch).len() - 1 == steps_after);
    } else if remaining > 0 {
        assert(unlink_plan(remaining, batch).len() == 1);
        assert(0 * batch == 0);
        assert(1 * batch == batch);
    }
}

/// 第 7 条 ①「计划相等」：做完第 i 批（不是最后一批）之后从它留下的单元数续做，算出的整串与不崩溃一路做下来的后半截逐步相同。
pub proof fn lemma_continued_plan_equals_the_rest(remaining: nat, batch: nat, i: int)
    requires
        batch > 0,
        0 <= i < unlink_plan(remaining, batch).len() - 1,
    ensures
        unlink_plan(remaining, batch).subrange(i + 1, unlink_plan(remaining, batch).len() as int)
            == unlink_plan(step_after(unlink_plan(remaining, batch)[i]), batch),
    decreases remaining,
{
    lemma_unlink_plan_shape(remaining, batch);
    let rest = (remaining - batch) as nat;
    let plan = unlink_plan(remaining, batch);
    let tail = unlink_plan(rest, batch);
    assert(remaining > batch);
    assert(plan == seq![UnlinkPlanStep::Batch { before: remaining, kept: rest }] + tail);
    if i == 0 {
        assert(plan.subrange(1, plan.len() as int) =~= tail);
    } else {
        lemma_continued_plan_equals_the_rest(rest, batch, i - 1);
        assert(plan[i] == tail[i - 1]);
        assert(plan.subrange(i + 1, plan.len() as int) =~= tail.subrange(i, tail.len() as int));
    }
}

/// 每个数据单元恰好被释放一次：单元号小于 `remaining` 的有且只有一步释放它，不小于的没有一步释放它。
pub proof fn lemma_each_data_unit_released_exactly_once(remaining: nat, batch: nat, unit: nat)
    requires
        batch > 0,
    ensures
        (unit < remaining) <==> exists|i: int| 0 <= i < unlink_plan(remaining, batch).len()
            && released_by(#[trigger] unlink_plan(remaining, batch)[i], unit),
        forall|i: int, j: int|
            0 <= i < unlink_plan(remaining, batch).len() && 0 <= j < unlink_plan(remaining, batch).len()
            && released_by(#[trigger] unlink_plan(remaining, batch)[i], unit)
            && released_by(#[trigger] unlink_plan(remaining, batch)[j], unit) ==> i == j,
    decreases remaining,
{
    let plan = unlink_plan(remaining, batch);
    if remaining <= batch {
        assert(plan.len() == 1);
        if unit < remaining {
            assert(released_by(plan[0], unit));
        }
    } else {
        let rest = (remaining - batch) as nat;
        let tail = unlink_plan(rest, batch);
        lemma_each_data_unit_released_exactly_once(rest, batch, unit);
        lemma_unlink_plan_shape(rest, batch);
        assert(plan == seq![UnlinkPlanStep::Batch { before: remaining, kept: rest }] + tail);
        assert forall|i: int| 1 <= i < plan.len() implies #[trigger] plan[i] == tail[i - 1] by {}
        // 后半截释放的单元都在 rest 之下，与第一批 [rest, remaining) 不交。
        assert forall|i: int| 0 <= i < tail.len() && released_by(#[trigger] tail[i], unit) implies unit < rest by {
            assert(step_before(tail[i]) <= rest);
        }
        if unit < remaining {
            if unit >= rest {
                assert(released_by(plan[0], unit));
            } else {
                let k = choose|k: int| 0 <= k < tail.len() && released_by(#[trigger] tail[k], unit);
                assert(plan[k + 1] == tail[k]);
                assert(released_by(plan[k + 1], unit));
            }
        } else {
            assert forall|i: int| 0 <= i < plan.len() implies !released_by(#[trigger] plan[i], unit) by {
                if i > 0 {
                    assert(plan[i] == tail[i - 1]);
                }
            }
        }
        assert forall|i: int, j: int|
            0 <= i < plan.len() && 0 <= j < plan.len()
            && released_by(#[trigger] plan[i], unit)
            && released_by(#[trigger] plan[j], unit) implies i == j by {
            if i > 0 {
                assert(plan[i] == tail[i - 1]);
            }
            if j > 0 {
                assert(plan[j] == tail[j - 1]);
            }
        }
    }
}

/// 下段某一层第 `index` 个节点（每个罩 `span` 个单元：[index × span, index × span + span)）在 `units` 个单元、没有洞的文件里存在。
pub open spec fn lower_node_exists(index: nat, span: nat, units: nat) -> bool {
    index * span < units
}

/// 罩的单元全在留下那一段里（core 的 `last_data_unit < data_units_kept`）：这个节点照抄。
pub open spec fn lower_node_entirely_kept(index: nat, span: nat, kept: nat) -> bool {
    index * span + span <= kept
}

/// 截完之后还在、但罩着被截掉那一段：这个节点重写（有同名后继）。
pub open spec fn lower_node_rewritten(index: nat, span: nat, kept: nat) -> bool {
    lower_node_exists(index, span, kept) && !lower_node_entirely_kept(index, span, kept)
}

/// 上一版有、截完之后不再出现：这个节点换下、没有同名后继。
pub open spec fn lower_node_dropped(index: nat, span: nat, before: nat, kept: nat) -> bool {
    lower_node_exists(index, span, before) && !lower_node_exists(index, span, kept)
}

/// K-R3-1：删除批每一层至多重写一个节点，就是罩着最后一个留下单元 kept − 1 与第一个被截单元 kept 的那一个（下标 kept ÷ span）；
/// 截在这一层的节点边界上（kept 是 span 的倍数）时这一层一个都不重写。照抄的与重写的上一版都有（同名前驱），
/// 换下的恰好是罩的单元全在被截掉那一段 [kept, before) 里的。
pub proof fn lemma_deletion_batch_rewrites_only_the_cut_path(index: nat, span: nat, before: nat, kept: nat)
    requires
        span > 0,
        0 < kept < before,
    ensures
        lower_node_rewritten(index, span, kept) <==> (index == kept / span && kept % span != 0),
        lower_node_rewritten(index, span, kept) ==> index * span <= kept - 1 && kept < index * span + span,
        lower_node_exists(index, span, kept) ==> lower_node_exists(index, span, before),
        lower_node_dropped(index, span, before, kept) <==> (kept <= index * span && index * span < before),
{
    let q = kept / span;
    let r = kept % span;
    assert(kept == q * span + r && 0 <= r < span) by {
        vstd::arithmetic::div_mod::lemma_fundamental_div_mod(kept as int, span as int);
        vstd::arithmetic::div_mod::lemma_mod_bound(kept as int, span as int);
        vstd::arithmetic::mul::lemma_mul_is_commutative(span as int, q as int);
    }
    if lower_node_rewritten(index, span, kept) {
        // index × span < kept < index × span + span 且 q × span ≤ kept < q × span + span ⇒ index == q、r ≠ 0。
        // index < q 时 (index + 1) × span ≤ q × span ≤ kept；index > q 时 (q + 1) × span ≤ index × span < kept：都矛盾。
        assert((index + 1) * span == index * span + span) by (nonlinear_arith);
        assert((q + 1) * span == q * span + span) by (nonlinear_arith);
        if index < q {
            vstd::arithmetic::mul::lemma_mul_inequality((index + 1) as int, q as int, span as int);
        }
        if index > q {
            vstd::arithmetic::mul::lemma_mul_inequality((q + 1) as int, index as int, span as int);
        }
        assert(index == q);
        assert(r != 0);
    }
    if index == q && r != 0 {
        assert(index * span < kept && kept < index * span + span);
    }
}

/// 无前提边界见证：不多于一批直接是最后一批、恰好一批也是最后一批、多一个先截一批；
/// 没有数据单元的文件只有释放 0 个的最后一批；u64 极值不回绕；size 乘法溢出交回 None。
pub fn witness_unlink_boundaries() {
    let _empty = next_unlink_batch(0, 144);
    assert(_empty == (0u64, true));
    let _exact = next_unlink_batch(144, 144);
    assert(_exact == (0u64, true));
    let _one_more = next_unlink_batch(145, 144);
    assert(_one_more == (1u64, false));
    let _high = next_unlink_batch(u64::MAX, 1);
    assert(_high == ((u64::MAX - 1) as u64, false));
    let _size = size_after_an_unlink_batch(2, 32634);
    assert(_size == Some(65268u64));
    let _overflow = size_after_an_unlink_batch(u64::MAX, 2);
    assert(_overflow is None);
    proof {
        // 7 个单元、每批 2 个：7 → 5 → 3 → 1 → 最后一批，共四步。
        assert(unlink_plan(1, 2) =~= seq![UnlinkPlanStep::Final { released: 1 }]);
        assert(unlink_plan(3, 2).len() == 2);
        assert(unlink_plan(5, 2).len() == 3);
        assert(unlink_plan(7, 2).len() == 4);
    }
}

} // verus!

#[cfg(test)]
mod tests {
    use super::{next_unlink_batch, size_after_an_unlink_batch, witness_unlink_boundaries};

    /// 逐个小输入走到底：每批截掉恰好一批、最后一批释放剩下全部，每个单元恰好释放一次，步数是 ⌈剩下 ÷ 批⌉.max(1)，
    /// 从任一中间版续做的后半截与一路做下来的逐步相同。
    #[test]
    fn unlink_batches_release_each_unit_once_and_any_continuation_equals_the_rest() {
        fn plan(remaining: u64, batch: u64) -> Vec<(u64, u64)> {
            let mut steps = Vec::new();
            let mut left = remaining;
            loop {
                let (kept, final_batch) = next_unlink_batch(left, batch);
                steps.push((left, kept));
                if final_batch {
                    assert_eq!(kept, 0);
                    return steps;
                }
                assert_eq!(left - kept, batch);
                assert!(kept >= 1);
                left = kept;
            }
        }
        for remaining in 0..40u64 {
            for batch in 1..10u64 {
                let steps = plan(remaining, batch);
                assert_eq!(steps.len() as u64, remaining.div_ceil(batch).max(1));
                let mut released: Vec<u64> = steps
                    .iter()
                    .flat_map(|&(before, after)| after..before)
                    .collect();
                released.sort_unstable();
                assert_eq!(released, (0..remaining).collect::<Vec<_>>());
                for (i, &(_, kept)) in steps.iter().enumerate().take(steps.len() - 1) {
                    assert_eq!(plan(kept, batch), steps[i + 1..].to_vec());
                }
            }
        }
    }

    #[test]
    fn unlink_boundaries_and_size_overflow() {
        witness_unlink_boundaries();
        assert_eq!(next_unlink_batch(u64::MAX, u64::MAX), (0, true));
        assert_eq!(
            size_after_an_unlink_batch(u64::MAX / 2, 2),
            Some(u64::MAX - 1)
        );
        assert_eq!(size_after_an_unlink_batch(u64::MAX / 2 + 1, 2), None);
        assert_eq!(size_after_an_unlink_batch(0, u64::MAX), Some(0));
    }
}

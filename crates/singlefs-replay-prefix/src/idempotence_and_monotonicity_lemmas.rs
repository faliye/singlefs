//! M3 的引理（不可信件）与两条推论 `corollary_the_landed_version_does_not_go_back`、
//! `corollary_recovery_replays_from_the_newest_self_verified_root`；三条 broadcast 引理给 `replay_core` 的 M3 ensures 用。
//! 记录级那一读的第一半 `lemma_applying_a_record_twice_in_a_row_is_applying_it_once` 在 crate 根里：
//! 单元图的测试 `strip_verus_specifications_from_judging_unit_keys` 按 crate 根的路径与 `src/lib.rs` 里的原文点名它。
//! 纯函数核 `pure_function_kernels` 是这个模块的子模块：`replay_core` 的 `broadcast use` 要看得见这里的三条 broadcast 引理，
//! 而这个模块只装证明，普通构建里 `verus!` 把它擦空，从这里 `use` 或再导出都成了没用到的 import；子模块经 `use super::*` 看得见它。
//! spec 与证明用的记法和引理留在 crate 根里是同一个缘故：这个模块、纯函数核与根里那条引理都要用它们，根里的条目子孙模块不经 `use` 也看得见。
use super::*;

pub(super) mod pure_function_kernels;

verus! {

// ===== M3 的引理（不可信件）=====
// 候选按切片次序严格递增。主循环不调它（上面 lemma_candidates_lie_below_their_end_and_above_the_chosen_root 的注：两两配对的触发会把 rlimit 用尽），只给 M3 用。
proof fn lemma_candidates_ascend_in_slice_order<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    end: int,
)
    requires
        0 <= end <= records.len(),
    ensures
        ({
            let candidates = candidates_before(records, root_instance, root_checkpoint_txg, end);
            forall|earlier: int, later: int|
                0 <= earlier < later < candidates.len() ==> candidates[earlier] < candidates[later]
        }),
    decreases end,
{
    if end > 0 {
        lemma_candidates_ascend_in_slice_order(records, root_instance, root_checkpoint_txg, end - 1);
        lemma_candidates_lie_below_their_end_and_above_the_chosen_root(
            records,
            root_instance,
            root_checkpoint_txg,
            end - 1,
        );
    }
}

// 在所选根之上的记录一定是候选：交回它在候选里的位置。
proof fn lemma_a_record_above_the_chosen_root_is_a_candidate<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    end: int,
    record_index: int,
) -> (position: int)
    requires
        0 <= record_index < end <= records.len(),
        above_the_chosen_root(records[record_index], root_instance, root_checkpoint_txg),
    ensures
        0 <= position < candidates_before(records, root_instance, root_checkpoint_txg, end).len(),
        candidates_before(records, root_instance, root_checkpoint_txg, end)[position]
            == record_index,
    decreases end,
{
    if record_index < end - 1 {
        lemma_a_record_above_the_chosen_root_is_a_candidate(
            records,
            root_instance,
            root_checkpoint_txg,
            end - 1,
            record_index,
        )
    } else {
        candidates_before(records, root_instance, root_checkpoint_txg, end).len() - 1
    }
}

proof fn lemma_no_record_above_the_chosen_root_means_no_candidate<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    end: int,
)
    requires
        0 <= end <= records.len(),
        forall|record_index: int|
            0 <= record_index < end ==> !#[trigger] above_the_chosen_root(
                records[record_index],
                root_instance,
                root_checkpoint_txg,
            ),
    ensures
        candidates_before(records, root_instance, root_checkpoint_txg, end).len() == 0,
    decreases end,
{
    if end > 0 {
        lemma_no_record_above_the_chosen_root_means_no_candidate(
            records,
            root_instance,
            root_checkpoint_txg,
            end - 1,
        );
        assert(!above_the_chosen_root(records[end - 1], root_instance, root_checkpoint_txg));
    }
}

// 落到的那一版之上的候选，就是原来的候选去掉 boundary 及其之前的那几个：boundary 及其之前没有一条在落到的那一版之上，
// 之后的在落到的那一版之上与在所选根之上是同一回事。
proof fn lemma_candidates_above_the_landed_root_are_the_tail<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    landed_txg: PrefixKernelCheckpointTxg,
    boundary: int,
    end: int,
)
    requires
        0 <= boundary < end <= records.len(),
        forall|record_index: int|
            0 <= record_index <= boundary ==> !#[trigger] above_the_chosen_root(
                records[record_index],
                root_instance,
                landed_txg,
            ),
        forall|record_index: int|
            boundary < record_index < records.len() ==> #[trigger] above_the_chosen_root(
                records[record_index],
                root_instance,
                landed_txg,
            ) == above_the_chosen_root(records[record_index], root_instance, root_checkpoint_txg),
    ensures
        ({
            let below = candidates_before(
                records,
                root_instance,
                root_checkpoint_txg,
                boundary + 1,
            ).len() as int;
            let all = candidates_before(records, root_instance, root_checkpoint_txg, end);
            &&& below <= all.len()
            &&& candidates_before(records, root_instance, landed_txg, end) == all.subrange(
                below,
                all.len() as int,
            )
        }),
    decreases end,
{
    lemma_candidates_before_a_shorter_end_are_a_prefix(
        records,
        root_instance,
        root_checkpoint_txg,
        boundary + 1,
        end,
    );
    let below = candidates_before(records, root_instance, root_checkpoint_txg, boundary + 1).len() as int;
    let all = candidates_before(records, root_instance, root_checkpoint_txg, end);
    if end == boundary + 1 {
        lemma_no_record_above_the_chosen_root_means_no_candidate(
            records,
            root_instance,
            landed_txg,
            end,
        );
        assert(candidates_before(records, root_instance, landed_txg, end) =~= all.subrange(
            below,
            all.len() as int,
        ));
    } else {
        lemma_candidates_above_the_landed_root_are_the_tail(
            records,
            root_instance,
            root_checkpoint_txg,
            landed_txg,
            boundary,
            end - 1,
        );
        lemma_candidates_before_a_shorter_end_are_a_prefix(
            records,
            root_instance,
            root_checkpoint_txg,
            boundary + 1,
            end - 1,
        );
        assert(above_the_chosen_root(records[end - 1], root_instance, landed_txg)
            == above_the_chosen_root(records[end - 1], root_instance, root_checkpoint_txg));
        assert(candidates_before(records, root_instance, landed_txg, end) =~= all.subrange(
            below,
            all.len() as int,
        ));
    }
}

// 一次发布的末条过了读者规则第四格（之后同一次发布没有读得出的记录），又有 I-8.12 ⇒ 同实例里切片在它之后的记录 txg 都比它大。
proof fn lemma_a_publish_end_that_passed_the_fourth_reader_rule_closes_its_txg<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    end_index: int,
)
    requires
        records_ascend_by_instance_and_counter(records),
        journal_txg_does_not_decrease_with_the_counter_within_an_instance(records),
        0 <= end_index < records.len(),
        !later_record_of_the_same_publish_is_readable(records, end_index),
    ensures
        forall|record_index: int|
            end_index < record_index < records.len() && (#[trigger] records[record_index]).instance
                == records[end_index].instance ==> records[record_index].checkpoint_txg.0
                > records[end_index].checkpoint_txg.0,
{
    assert forall|record_index: int|
        end_index < record_index < records.len() && (#[trigger] records[record_index]).instance
            == records[end_index].instance implies records[record_index].checkpoint_txg.0
        > records[end_index].checkpoint_txg.0 by {
        lemma_slice_order_within_an_instance_follows_the_counter(records, end_index, record_index);
        assert(is_a_later_record_of_the_same_instance(records, end_index, record_index));
        if records[record_index].checkpoint_txg.0 <= records[end_index].checkpoint_txg.0 {
            assert forall|between: int|
                0 <= between < records.len() && #[trigger] is_a_later_record_of_the_same_instance(
                    records,
                    end_index,
                    between,
                ) && records[between].counter.0 < records[record_index].counter.0
                    implies records[between].checkpoint_txg.0
                <= records[end_index].checkpoint_txg.0 by {
                assert(is_a_later_record_of_the_same_instance(records, between, record_index));
            }
            assert(records[record_index].checkpoint_txg == records[end_index].checkpoint_txg);
        }
    }
}

// 落到的那一版的发布里带末条标志的只有落点那一条 ⇒ 以它为所选根时，带末条标志的计数器就是落点那一条的。
proof fn lemma_the_flagged_counters_of_the_landed_publish<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    root_instance: PrefixKernelInstanceGeneration,
    landed_txg: PrefixKernelCheckpointTxg,
    landed_record_index: int,
    end: int,
)
    requires
        0 <= landed_record_index < records.len(),
        0 <= end <= records.len(),
        records[landed_record_index].instance == root_instance,
        records[landed_record_index].checkpoint_txg == landed_txg,
        is_last_record_of_its_publish(records[landed_record_index]),
        forall|record_index: int|
            0 <= record_index < records.len() && records[record_index].instance == root_instance
                && records[record_index].checkpoint_txg == landed_txg
                && #[trigger] is_last_record_of_its_publish(records[record_index]) ==> record_index
                == landed_record_index,
    ensures
        last_record_flag_counters_of_the_chosen_roots_publish(
            records,
            root_instance,
            landed_txg,
            end,
        ) == if end > landed_record_index {
            seq![records[landed_record_index].counter]
        } else {
            Seq::<JournalCounter>::empty()
        },
    decreases end,
{
    if end > 0 {
        lemma_the_flagged_counters_of_the_landed_publish(
            records,
            root_instance,
            landed_txg,
            landed_record_index,
            end - 1,
        );
        if end - 1 == landed_record_index {
            assert(Seq::<JournalCounter>::empty().push(records[landed_record_index].counter)
                =~= seq![records[landed_record_index].counter]);
        }
    }
}

// 同一个读者不会在同一个候选上既让一串停下、又让另一串接受它（两串在这一条上的非回调判定相同时）。
#[allow(clippy::too_many_arguments, reason = "停下那一串与接受那一串各三样判定输入，摊成参数才看得出哪一样两边相同")]
proof fn lemma_the_same_reader_does_not_both_stop_and_admit_a_candidate<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    records: Seq<CoreRecord<RootSource>>,
    next: int,
    stopping_previous: Option<CoreRecord<RootSource>>,
    stopping_anchor: Option<JournalCounter>,
    stopping_root_checkpoint_txg: PrefixKernelCheckpointTxg,
    admitting_previous: Option<CoreRecord<RootSource>>,
    admitting_anchor: Option<JournalCounter>,
    admitting_root_checkpoint_txg: PrefixKernelCheckpointTxg,
    root_instance: PrefixKernelInstanceGeneration,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    verification_failed: usize,
)
    requires
        the_reader_answers_each_position_one_way(back_chain_broken),
        the_reader_answers_each_position_one_way(named_units_verified),
        continues_the_jsn_chain(
            records[next],
            stopping_previous,
            stopping_anchor,
            root_instance,
            stopping_root_checkpoint_txg,
        ) == continues_the_jsn_chain(
            records[next],
            admitting_previous,
            admitting_anchor,
            root_instance,
            admitting_root_checkpoint_txg,
        ),
        obeys_the_reader_rules_and_the_commit_markers(records[next], stopping_previous)
            == obeys_the_reader_rules_and_the_commit_markers(records[next], admitting_previous),
    ensures
        !(((verification_failed == 0 && fails_a_check_before_the_named_units(
            records,
            next,
            stopping_previous,
            stopping_anchor,
            root_instance,
            stopping_root_checkpoint_txg,
            back_chain_broken,
        )) || (verification_failed == 1 && passes_every_check_before_the_named_units(
            records,
            next,
            stopping_previous,
            stopping_anchor,
            root_instance,
            stopping_root_checkpoint_txg,
            back_chain_broken,
        ) && named_units_reported_unverified(named_unit_verification, named_units_verified, next)))
            && passes_every_check_before_the_named_units(
            records,
            next,
            admitting_previous,
            admitting_anchor,
            root_instance,
            admitting_root_checkpoint_txg,
            back_chain_broken,
        ) && named_units_reported_verified(named_unit_verification, named_units_verified, next)),
{
    reveal(passes_every_check_before_the_named_units);
    reveal(fails_a_check_before_the_named_units);
}

// 同样的入参、同一个读者：规则容得下的两个结果里，过了逐条判定的那一串一样长（短的那一串停下的候选，长的那一串不会接受）。
#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
proof fn lemma_the_same_inputs_admit_no_longer_chain<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    chosen_root_source: RootSource,
    records: Seq<CoreRecord<RootSource>>,
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    shorter: CoreOutcome<RootSource>,
    longer: CoreOutcome<RootSource>,
)
    requires
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            Ok(shorter),
        ),
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            Ok(longer),
        ),
        the_reader_answers_each_position_one_way(back_chain_broken),
        the_reader_answers_each_position_one_way(named_units_verified),
    ensures
        shorter.admitted@.len() >= longer.admitted@.len(),
{
    let candidates = candidates_before(
        records,
        root_instance,
        root_checkpoint_txg,
        records.len() as int,
    );
    let anchor = anchor_of_the_chain(records, root_instance, root_checkpoint_txg);
    let short_chain = shorter.admitted@;
    let long_chain = longer.admitted@;
    if short_chain.len() < long_chain.len() {
        let stop_position = short_chain.len() as int;
        let next = candidates[stop_position];
        assert(long_chain[stop_position] == next);
        assert(admitted_record_passed_every_check(
            records,
            long_chain,
            stop_position,
            anchor,
            root_instance,
            root_checkpoint_txg,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
        ));
        reveal(admitted_record_passed_every_check);
        if stop_position > 0 {
            assert(short_chain[stop_position - 1] == candidates[stop_position - 1]);
            assert(long_chain[stop_position - 1] == candidates[stop_position - 1]);
        }
        assert(previous_in_the_chain(records, short_chain, stop_position) == previous_in_the_chain(
            records,
            long_chain,
            stop_position,
        ));
        lemma_the_same_reader_does_not_both_stop_and_admit_a_candidate(
            records,
            next,
            previous_in_the_chain(records, short_chain, stop_position),
            anchor,
            root_checkpoint_txg,
            previous_in_the_chain(records, long_chain, stop_position),
            anchor,
            root_checkpoint_txg,
            root_instance,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            shorter.report.verification_failed,
        );
    }
}

// 推论第 4 步「恢复是确定的函数」：同样的入参、同一个读者，规则容得下的两个结果过了逐条判定的、施加的、交回的根段都相同。
#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
proof fn lemma_the_rules_allow_one_outcome<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    chosen_root_source: RootSource,
    records: Seq<CoreRecord<RootSource>>,
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    outcome: CoreOutcome<RootSource>,
    again: CoreOutcome<RootSource>,
)
    requires
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            Ok(outcome),
        ),
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            Ok(again),
        ),
        the_reader_answers_each_position_one_way(back_chain_broken),
        the_reader_answers_each_position_one_way(named_units_verified),
    ensures
        again.admitted@ == outcome.admitted@,
        again.applied@ == outcome.applied@,
        again.root_source_after_replay == outcome.root_source_after_replay,
{
    lemma_the_same_inputs_admit_no_longer_chain(
        root_instance,
        root_checkpoint_txg,
        chosen_root_source,
        records,
        in_flight_limit,
        named_unit_verification,
        back_chain_broken,
        named_units_verified,
        outcome,
        again,
    );
    lemma_the_same_inputs_admit_no_longer_chain(
        root_instance,
        root_checkpoint_txg,
        chosen_root_source,
        records,
        in_flight_limit,
        named_unit_verification,
        back_chain_broken,
        named_units_verified,
        again,
        outcome,
    );
    let chain = outcome.admitted@;
    assert(again.admitted@ =~= chain);
    let first = outcome.applied@;
    let second = again.applied@;
    if first.len() < second.len() {
        assert(second[second.len() - 1] == chain[second.len() - 1]);
        assert(!is_last_record_of_its_publish(records[chain[second.len() - 1]]));
    } else if second.len() < first.len() {
        assert(first[first.len() - 1] == chain[first.len() - 1]);
        assert(!is_last_record_of_its_publish(records[chain[first.len() - 1]]));
    }
    assert forall|position: int| 0 <= position < first.len() implies first[position]
        == second[position] by {
        assert(first[position] == chain[position]);
        assert(second[position] == chain[position]);
    }
    assert(first =~= second);
}

// 落点（施加的最后一条）把它那个 txg 封住：落点及其之前没有记录在它之上，之后的在它之上与在所选根之上是同一回事，
// 它那次发布里带末条标志的只有它，所选根之上在它之前（含）的候选恰好是施加的那几条。
#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
proof fn lemma_the_landed_record_closes_its_txg<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    chosen_root_source: RootSource,
    records: Seq<CoreRecord<RootSource>>,
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    outcome: CoreOutcome<RootSource>,
)
    requires
        records_ascend_by_instance_and_counter(records),
        journal_txg_does_not_decrease_with_the_counter_within_an_instance(records),
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            Ok(outcome),
        ),
        outcome.applied@.len() > 0,
    ensures
        ({
            let applied = outcome.applied@;
            let landed_record_index = applied.last();
            let landed_txg = records[landed_record_index].checkpoint_txg;
            let candidates = candidates_before(
                records,
                root_instance,
                root_checkpoint_txg,
                records.len() as int,
            );
            &&& 0 <= landed_record_index < records.len()
            &&& records[landed_record_index].instance == root_instance
            &&& landed_txg.0 > root_checkpoint_txg.0
            &&& is_last_record_of_its_publish(records[landed_record_index])
            &&& landed_record_index == candidates[applied.len() - 1]
            &&& applied.len() <= outcome.admitted@.len() <= candidates.len()
            &&& candidates_before(records, root_instance, landed_txg, records.len() as int)
                == candidates.subrange(applied.len() as int, candidates.len() as int)
            &&& last_record_flag_counters_of_the_chosen_roots_publish(
                records,
                root_instance,
                landed_txg,
                records.len() as int,
            ) == seq![records[landed_record_index].counter]
        }),
{
    let candidates = candidates_before(
        records,
        root_instance,
        root_checkpoint_txg,
        records.len() as int,
    );
    let anchor = anchor_of_the_chain(records, root_instance, root_checkpoint_txg);
    let admitted = outcome.admitted@;
    let applied = outcome.applied@;
    let applied_count = applied.len() as int;
    let landed_record_index = applied.last();
    let landed_record = records[landed_record_index];
    let landed_txg = landed_record.checkpoint_txg;
    lemma_candidates_lie_below_their_end_and_above_the_chosen_root(
        records,
        root_instance,
        root_checkpoint_txg,
        records.len() as int,
    );
    lemma_candidates_ascend_in_slice_order(
        records,
        root_instance,
        root_checkpoint_txg,
        records.len() as int,
    );
    assert(applied[applied_count - 1] == admitted[applied_count - 1]);
    assert(admitted[applied_count - 1] == candidates[applied_count - 1]);
    assert(above_the_chosen_root(landed_record, root_instance, root_checkpoint_txg));
    // 过了逐条判定、带末条标志的每一条都过了读者规则第四格
    assert forall|position_in_chain: int|
        0 <= position_in_chain < admitted.len() && is_last_record_of_its_publish(
            records[#[trigger] admitted[position_in_chain]],
        ) implies !later_record_of_the_same_publish_is_readable(
        records,
        admitted[position_in_chain],
    ) by {
        assert(admitted_record_passed_every_check(
            records,
            admitted,
            position_in_chain,
            anchor,
            root_instance,
            root_checkpoint_txg,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
        ));
        reveal(admitted_record_passed_every_check);
        reveal(passes_every_check_before_the_named_units);
    }
    lemma_a_publish_end_that_passed_the_fourth_reader_rule_closes_its_txg(
        records,
        landed_record_index,
    );
    assert forall|record_index: int|
        0 <= record_index <= landed_record_index implies !#[trigger] above_the_chosen_root(
        records[record_index],
        root_instance,
        landed_txg,
    ) by {
        if records[record_index].instance == root_instance && record_index < landed_record_index {
            lemma_slice_order_within_an_instance_follows_the_counter(
                records,
                record_index,
                landed_record_index,
            );
            assert(is_a_later_record_of_the_same_instance(
                records,
                record_index,
                landed_record_index,
            ));
        }
    }
    assert forall|record_index: int|
        landed_record_index < record_index < records.len() implies #[trigger] above_the_chosen_root(
        records[record_index],
        root_instance,
        landed_txg,
    ) == above_the_chosen_root(records[record_index], root_instance, root_checkpoint_txg) by {
        if records[record_index].instance == root_instance {
            assert(records[record_index].checkpoint_txg.0 > landed_txg.0);
        }
    }
    // 所选根之上、切片在落点及其之前的候选恰好 applied_count 个
    lemma_candidates_before_a_shorter_end_are_a_prefix(
        records,
        root_instance,
        root_checkpoint_txg,
        landed_record_index + 1,
        records.len() as int,
    );
    let through_the_landed_record = candidates_before(
        records,
        root_instance,
        root_checkpoint_txg,
        landed_record_index + 1,
    );
    let below = through_the_landed_record.len() as int;
    assert(through_the_landed_record[below - 1] == landed_record_index);
    assert(candidates[below - 1] == landed_record_index);
    if below < applied_count {
        assert(candidates[below - 1] < candidates[applied_count - 1]);
    } else if below > applied_count {
        assert(candidates[applied_count - 1] < candidates[below - 1]);
    }
    lemma_candidates_above_the_landed_root_are_the_tail(
        records,
        root_instance,
        root_checkpoint_txg,
        landed_txg,
        landed_record_index,
        records.len() as int,
    );
    // 落到的那一版的发布里带末条标志的只有落点
    assert forall|record_index: int|
        0 <= record_index < records.len() && records[record_index].instance == root_instance
            && records[record_index].checkpoint_txg == landed_txg
            && #[trigger] is_last_record_of_its_publish(records[record_index]) implies record_index
        == landed_record_index by {
        if record_index < landed_record_index {
            let position = lemma_a_record_above_the_chosen_root_is_a_candidate(
                records,
                root_instance,
                root_checkpoint_txg,
                records.len() as int,
                record_index,
            );
            if position >= applied_count - 1 {
                assert(candidates[applied_count - 1] <= candidates[position]);
            }
            assert(admitted[position] == record_index);
            lemma_a_publish_end_that_passed_the_fourth_reader_rule_closes_its_txg(
                records,
                record_index,
            );
            assert(records[landed_record_index].instance == records[record_index].instance);
        } else if record_index > landed_record_index {
            assert(records[record_index].instance == records[landed_record_index].instance);
        }
    }
    lemma_the_flagged_counters_of_the_landed_publish(
        records,
        root_instance,
        landed_txg,
        landed_record_index,
        records.len() as int,
    );
}

// 不动点那一读：落到的那一版当所选根再取一次前缀，结局是 replaying_again_from_the_landed_root_stays_put。
// 前提：I-8.12、同一个读者、所选根之上的记录条数不超在飞上限（入参严格升序担保「同一 (实例代号, 计数器) 只对应一份内容」）。
#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
proof fn lemma_replaying_from_the_landed_root_stays_put<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    chosen_root_source: RootSource,
    records: Seq<CoreRecord<RootSource>>,
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    outcome: CoreOutcome<RootSource>,
    replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>,
)
    requires
        records_ascend_by_instance_and_counter(records),
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            Ok(outcome),
        ),
        // I-8.12
        journal_txg_does_not_decrease_with_the_counter_within_an_instance(records),
        // 同一个读者
        the_reader_answers_each_position_one_way(back_chain_broken),
        the_reader_answers_each_position_one_way(named_units_verified),
        // 所选根之上的记录条数不超在飞上限（D23 已定项 18）
        records_above_the_chosen_root_fit_the_in_flight_limit(
            records,
            root_instance,
            root_checkpoint_txg,
            in_flight_limit,
        ),
        replay_follows_the_prefix_rules(
            landed_instance(root_instance, records, outcome.applied@),
            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
            outcome.root_source_after_replay,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed_again,
        ),
    ensures
        replaying_again_from_the_landed_root_stays_put(
            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
            outcome.root_source_after_replay,
            replayed_again,
        ),
{
    let applied = outcome.applied@;
    if applied.len() == 0 {
        // 一条没施加：落到的就是所选根，第二次的入参与第一次相同
        match replayed_again {
            Ok(again) => {
                lemma_the_rules_allow_one_outcome(
                    root_instance,
                    root_checkpoint_txg,
                    chosen_root_source,
                    records,
                    in_flight_limit,
                    named_unit_verification,
                    back_chain_broken,
                    named_units_verified,
                    outcome,
                    again,
                );
            },
            Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
            Err(
                CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                    counters: _,
                },
            ) => {},
        }
    } else {
        lemma_the_landed_record_closes_its_txg(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            outcome,
        );
        let candidates = candidates_before(
            records,
            root_instance,
            root_checkpoint_txg,
            records.len() as int,
        );
        let anchor = anchor_of_the_chain(records, root_instance, root_checkpoint_txg);
        let admitted = outcome.admitted@;
        let applied_count = applied.len() as int;
        let landed_record_index = applied.last();
        let landed_record = records[landed_record_index];
        let landed_txg = landed_record.checkpoint_txg;
        let candidates_again = candidates.subrange(applied_count, candidates.len() as int);
        let anchor_again = anchor_of_the_chain(records, root_instance, landed_txg);
        assert(anchor_again == Some(landed_record.counter));
        match replayed_again {
            Ok(again) => {
                let chain_again = again.admitted@;
                let applied_again = again.applied@;
                if applied_again.len() > 0 {
                    // 第二次施加的最后一条带末条标志；它要么在第一次攒着没施加的尾巴里（那里一条都不带末条标志），
                    // 要么第二次接受了第一次停下的那个候选——同一个读者不会这样
                    let last_again = applied_again.len() - 1;
                    assert(applied_again[last_again] == chain_again[last_again]);
                    assert(chain_again[last_again] == candidates_again[last_again]);
                    if applied_count + last_again < admitted.len() {
                        assert(admitted[applied_count + last_again] == candidates[applied_count
                            + last_again]);
                    } else {
                        let position_again = admitted.len() - applied_count;
                        let next = candidates[admitted.len() as int];
                        assert(chain_again[position_again] == next);
                        assert(admitted_record_passed_every_check(
                            records,
                            chain_again,
                            position_again,
                            anchor_again,
                            root_instance,
                            landed_txg,
                            named_unit_verification,
                            back_chain_broken,
                            named_units_verified,
                        ));
                        reveal(admitted_record_passed_every_check);
                        if position_again > 0 {
                            assert(chain_again[position_again - 1] == candidates[admitted.len()
                                - 1]);
                            assert(admitted[admitted.len() - 1] == candidates[admitted.len() - 1]);
                        } else {
                            assert(admitted[admitted.len() - 1] == applied[applied_count - 1]);
                        }
                        lemma_the_same_reader_does_not_both_stop_and_admit_a_candidate(
                            records,
                            next,
                            previous_in_the_chain(records, admitted, admitted.len() as int),
                            anchor,
                            root_checkpoint_txg,
                            previous_in_the_chain(records, chain_again, position_again),
                            anchor_again,
                            landed_txg,
                            root_instance,
                            named_unit_verification,
                            back_chain_broken,
                            named_units_verified,
                            outcome.report.verification_failed,
                        );
                    }
                }
            },
            Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
            Err(
                CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                    counters: _,
                },
            ) => {},
        }
    }
}

// replay_core 的 M3 ensures 里三条各由下面一个 broadcast 引理从 replay_follows_the_prefix_rules 推出（replay_core 里 broadcast use），
// 那三个 spec 标 opaque、只在引理里 reveal：开着时它们的定义进了 replay_core 的整个上下文，主循环要 rlimit 10 才过（5 到 9 都报 rlimit 用尽）。
// 「落点不早于所选根」那一条开着、不 broadcast：Verus 把 replay_follows_the_prefix_rules 展开就得到，它的引理只给推论调。
#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
broadcast proof fn lemma_replay_core_result_replays_an_applied_record_twice_as_once<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    chosen_root_source: RootSource,
    records: Seq<CoreRecord<RootSource>>,
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
)
    requires
        records_ascend_by_instance_and_counter(records),
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
    ensures
        #[trigger] replaying_an_applied_record_twice_leaves_the_root_as_once(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
{
    reveal(replaying_an_applied_record_twice_leaves_the_root_as_once);
    match replayed {
        Ok(outcome) => {
            let applied = outcome.applied@;
            let applied_records = records_at(records, applied);
            if applied.len() > 0 {
                assert(applied_records.last() == records[applied.last()]);
                assert(applied[applied.len() - 1] == outcome.admitted@[applied.len() - 1]);
            }
            assert forall|position: int| 0 <= position < applied_records.len() implies #[trigger] root_source_after_applying_in_order(
                chosen_root_source,
                applied_records.insert(position + 1, applied_records[position]),
            ) == outcome.root_source_after_replay by {
                lemma_applying_a_record_twice_in_a_row_is_applying_it_once(
                    chosen_root_source,
                    applied_records,
                    position,
                );
            }
            if journal_txg_does_not_decrease_with_the_counter_within_an_instance(records) {
                assert forall|position: int| 0 <= position < applied.len() implies !above_the_chosen_root(
                    records[#[trigger] applied[position]],
                    landed_instance(root_instance, records, applied),
                    landed_checkpoint_txg(root_checkpoint_txg, records, applied),
                ) by {
                    if position < applied.len() - 1 {
                        assert(applied[position] < applied[applied.len() - 1]);
                        assert(above_the_chosen_root(
                            records[applied[position]],
                            root_instance,
                            root_checkpoint_txg,
                        ));
                        assert(above_the_chosen_root(
                            records[applied[applied.len() - 1]],
                            root_instance,
                            root_checkpoint_txg,
                        ));
                        lemma_slice_order_within_an_instance_follows_the_counter(
                            records,
                            applied[position],
                            applied.last(),
                        );
                        assert(is_a_later_record_of_the_same_instance(
                            records,
                            applied[position],
                            applied.last(),
                        ));
                    }
                }
            }
        },
        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
        Err(
            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                counters: _,
            },
        ) => {},
    }
}

#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
broadcast proof fn lemma_replay_core_result_is_a_fixed_point<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    chosen_root_source: RootSource,
    records: Seq<CoreRecord<RootSource>>,
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
)
    requires
        records_ascend_by_instance_and_counter(records),
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
    ensures
        #[trigger] replaying_from_the_landed_root_is_a_fixed_point(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
{
    reveal(replaying_from_the_landed_root_is_a_fixed_point);
    match replayed {
        Ok(outcome) => {
            if journal_txg_does_not_decrease_with_the_counter_within_an_instance(records)
                && the_reader_answers_each_position_one_way(back_chain_broken)
                && the_reader_answers_each_position_one_way(named_units_verified)
                && records_above_the_chosen_root_fit_the_in_flight_limit(
                records,
                root_instance,
                root_checkpoint_txg,
                in_flight_limit,
            ) {
                assert forall|replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>|
                    #[trigger] replay_follows_the_prefix_rules(
                        landed_instance(root_instance, records, outcome.applied@),
                        landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
                        outcome.root_source_after_replay,
                        records,
                        in_flight_limit,
                        named_unit_verification,
                        back_chain_broken,
                        named_units_verified,
                        replayed_again,
                    ) implies replaying_again_from_the_landed_root_stays_put(
                    landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
                    outcome.root_source_after_replay,
                    replayed_again,
                ) by {
                    lemma_replaying_from_the_landed_root_stays_put(
                        root_instance,
                        root_checkpoint_txg,
                        chosen_root_source,
                        records,
                        in_flight_limit,
                        named_unit_verification,
                        back_chain_broken,
                        named_units_verified,
                        outcome,
                        replayed_again,
                    );
                }
            }
        },
        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
        Err(
            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                counters: _,
            },
        ) => {},
    }
}

#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
proof fn lemma_replay_core_result_lands_no_earlier_than_the_chosen_root<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    chosen_root_source: RootSource,
    records: Seq<CoreRecord<RootSource>>,
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
)
    requires
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
    ensures
        replay_lands_no_earlier_than_the_chosen_root(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
{
    match replayed {
        Ok(outcome) => {
            let applied = outcome.applied@;
            if applied.len() > 0 {
                assert(above_the_chosen_root(
                    records[applied[applied.len() - 1]],
                    root_instance,
                    root_checkpoint_txg,
                ));
            }
        },
        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
        Err(
            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                counters: _,
            },
        ) => {},
    }
}

#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开")]
broadcast proof fn lemma_replay_core_result_is_the_only_one_the_rules_allow<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    chosen_root_source: RootSource,
    records: Seq<CoreRecord<RootSource>>,
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
)
    requires
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
    ensures
        #[trigger] replay_outcome_is_the_only_one_the_rules_allow(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
{
    reveal(replay_outcome_is_the_only_one_the_rules_allow);
    match replayed {
        Ok(outcome) => {
            if the_reader_answers_each_position_one_way(back_chain_broken)
                && the_reader_answers_each_position_one_way(named_units_verified) {
                assert forall|replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>|
                    #[trigger] replay_follows_the_prefix_rules(
                        root_instance,
                        root_checkpoint_txg,
                        chosen_root_source,
                        records,
                        in_flight_limit,
                        named_unit_verification,
                        back_chain_broken,
                        named_units_verified,
                        replayed_again,
                    ) implies (replayed_again matches Ok(again) && again.admitted@
                    == outcome.admitted@ && again.applied@ == outcome.applied@
                    && again.root_source_after_replay == outcome.root_source_after_replay) by {
                    match replayed_again {
                        Ok(again) => {
                            lemma_the_rules_allow_one_outcome(
                                root_instance,
                                root_checkpoint_txg,
                                chosen_root_source,
                                records,
                                in_flight_limit,
                                named_unit_verification,
                                back_chain_broken,
                                named_units_verified,
                                outcome,
                                again,
                            );
                        },
                        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
                        Err(
                            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                                counters: _,
                            },
                        ) => {},
                    }
                }
            }
        },
        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
        Err(
            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                counters: _,
            },
        ) => {},
    }
}

// M3 单调（推论；版本序取 (txg, 实例代号)）：同一次运行里两个崩溃状态 s ⊆ u，
// u 上恢复落到的那一版不早于 s 上的。核里证的只有两步：落点不早于所选根、恢复是确定的函数（上面两条 ensures 的引理）；
// 两条前提是择根、写侧的条款，核判不了，推出来的写成 requires：
// 所选根取根环里最新的有效根 ⇒ u 的所选根不早于 s 的；s 施加了一次发布时，串行提交与 D16（发布语义）已定项 7 的持久顺序
// ⇒ u 含那一版的根槽（u 的所选根不早于它），否则 u 与 s 的所选根、记录表、读者都相同（3.2 第 2、4 步）。开并行提交或把域放宽到带故障的状态时重开。
#[allow(clippy::too_many_arguments, reason = "两次恢复各一套入参，照 replay_follows_the_prefix_rules 原样摊开")]
pub proof fn corollary_the_landed_version_does_not_go_back<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    smaller_root_instance: PrefixKernelInstanceGeneration,
    smaller_root_checkpoint_txg: PrefixKernelCheckpointTxg,
    smaller_chosen_root_source: RootSource,
    smaller_records: Seq<CoreRecord<RootSource>>,
    smaller_back_chain_broken: BackChain,
    smaller_named_units_verified: NamedUnits,
    smaller_outcome: CoreOutcome<RootSource>,
    larger_root_instance: PrefixKernelInstanceGeneration,
    larger_root_checkpoint_txg: PrefixKernelCheckpointTxg,
    larger_chosen_root_source: RootSource,
    larger_records: Seq<CoreRecord<RootSource>>,
    larger_back_chain_broken: BackChain,
    larger_named_units_verified: NamedUnits,
    larger_replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
)
    requires
        replay_follows_the_prefix_rules(
            smaller_root_instance,
            smaller_root_checkpoint_txg,
            smaller_chosen_root_source,
            smaller_records,
            in_flight_limit,
            named_unit_verification,
            smaller_back_chain_broken,
            smaller_named_units_verified,
            Ok(smaller_outcome),
        ),
        replay_follows_the_prefix_rules(
            larger_root_instance,
            larger_root_checkpoint_txg,
            larger_chosen_root_source,
            larger_records,
            in_flight_limit,
            named_unit_verification,
            larger_back_chain_broken,
            larger_named_units_verified,
            larger_replayed,
        ),
        // 择根：所选根取根环里最新的有效根（3.2 第 1 步）
        version_is_not_later_than(
            smaller_root_checkpoint_txg,
            smaller_root_instance,
            larger_root_checkpoint_txg,
            larger_root_instance,
        ),
        // 串行提交与持久顺序（3.2 第 2、4 步）
        smaller_outcome.applied@.len() > 0 ==> version_is_not_later_than(
            landed_checkpoint_txg(
                smaller_root_checkpoint_txg,
                smaller_records,
                smaller_outcome.applied@,
            ),
            landed_instance(smaller_root_instance, smaller_records, smaller_outcome.applied@),
            larger_root_checkpoint_txg,
            larger_root_instance,
        ) || (larger_root_instance == smaller_root_instance && larger_root_checkpoint_txg
            == smaller_root_checkpoint_txg && larger_chosen_root_source
            == smaller_chosen_root_source && larger_records == smaller_records
            && larger_back_chain_broken == smaller_back_chain_broken && larger_named_units_verified
            == smaller_named_units_verified),
        // 同一个读者（只在 u 与 s 相同那一支用到）
        the_reader_answers_each_position_one_way(smaller_back_chain_broken),
        the_reader_answers_each_position_one_way(smaller_named_units_verified),
    ensures
        larger_replayed matches Ok(larger_outcome) ==> version_is_not_later_than(
            landed_checkpoint_txg(
                smaller_root_checkpoint_txg,
                smaller_records,
                smaller_outcome.applied@,
            ),
            landed_instance(smaller_root_instance, smaller_records, smaller_outcome.applied@),
            landed_checkpoint_txg(larger_root_checkpoint_txg, larger_records, larger_outcome.applied@),
            landed_instance(larger_root_instance, larger_records, larger_outcome.applied@),
        ),
{
    lemma_replay_core_result_lands_no_earlier_than_the_chosen_root(
        smaller_root_instance,
        smaller_root_checkpoint_txg,
        smaller_chosen_root_source,
        smaller_records,
        in_flight_limit,
        named_unit_verification,
        smaller_back_chain_broken,
        smaller_named_units_verified,
        Ok(smaller_outcome),
    );
    lemma_replay_core_result_lands_no_earlier_than_the_chosen_root(
        larger_root_instance,
        larger_root_checkpoint_txg,
        larger_chosen_root_source,
        larger_records,
        in_flight_limit,
        named_unit_verification,
        larger_back_chain_broken,
        larger_named_units_verified,
        larger_replayed,
    );
    match larger_replayed {
        Ok(larger_outcome) => {
            if smaller_outcome.applied@.len() > 0 && !version_is_not_later_than(
                landed_checkpoint_txg(
                    smaller_root_checkpoint_txg,
                    smaller_records,
                    smaller_outcome.applied@,
                ),
                landed_instance(smaller_root_instance, smaller_records, smaller_outcome.applied@),
                larger_root_checkpoint_txg,
                larger_root_instance,
            ) {
                lemma_the_rules_allow_one_outcome(
                    smaller_root_instance,
                    smaller_root_checkpoint_txg,
                    smaller_chosen_root_source,
                    smaller_records,
                    in_flight_limit,
                    named_unit_verification,
                    smaller_back_chain_broken,
                    smaller_named_units_verified,
                    smaller_outcome,
                    larger_outcome,
                );
            }
        },
        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
        Err(
            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                counters: _,
            },
        ) => {},
    }
}

// 择根与取前缀合判（C650 的 Verus 一半）：前提就是两个核交回的东西——choose_root_slot 交回 Some(chosen_slot) 时它的 ensures，
// 与拿那个槽的择新键当所选根调 replay_core 时它的五条 ensures；不加别的前提。胶水真的拿所选槽的根去调重放
// （recovery.rs 的 recover、mount.rs 与 mounted_read.rs 的读阶段）不在 Verus 里。
#[allow(clippy::too_many_arguments, reason = "入参照 replay_follows_the_prefix_rules 原样摊开，加根环与所选槽")]
pub proof fn corollary_recovery_replays_from_the_newest_self_verified_root<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    root_slots: Seq<Option<RootChoiceKey>>,
    chosen_slot: usize,
    chosen_root_source: RootSource,
    records: Seq<CoreRecord<RootSource>>,
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
)
    requires
        // choose_root_slot 的 ensures（交回 Some(chosen_slot) 那一支）
        root_choice_follows_the_newest_rule(root_slots, Some(chosen_slot)),
        // replay_core 的五条 ensures，所选根取所选槽的择新键
        replay_follows_the_prefix_rules(
            root_choice_key_of(root_slots[chosen_slot as int]).instance,
            root_choice_key_of(root_slots[chosen_slot as int]).checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
        replaying_an_applied_record_twice_leaves_the_root_as_once(
            root_choice_key_of(root_slots[chosen_slot as int]).instance,
            root_choice_key_of(root_slots[chosen_slot as int]).checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
        replaying_from_the_landed_root_is_a_fixed_point(
            root_choice_key_of(root_slots[chosen_slot as int]).instance,
            root_choice_key_of(root_slots[chosen_slot as int]).checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
        replay_lands_no_earlier_than_the_chosen_root(
            root_choice_key_of(root_slots[chosen_slot as int]).instance,
            root_choice_key_of(root_slots[chosen_slot as int]).checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
        replay_outcome_is_the_only_one_the_rules_allow(
            root_choice_key_of(root_slots[chosen_slot as int]).instance,
            root_choice_key_of(root_slots[chosen_slot as int]).checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
    ensures
        recovery_replays_from_the_newest_self_verified_root(
            root_slots,
            chosen_slot,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
{
    let chosen_key = root_choice_key_of(root_slots[chosen_slot as int]);
    match replayed {
        Ok(outcome) => {
            // 每一条自证过的根 ≤ 所选根（择根核）≤ 落点：择新序可传递。后一个 ≤ 由 requires 里 replay_core 的 M2
            // （replay_follows_the_prefix_rules）推得出（lemma_replay_core_result_lands_no_earlier_than_the_chosen_root 只凭 M2 证它），
            // requires 里另列的 J1（replay_lands_no_earlier_than_the_chosen_root）是多余前提，不是这一步非用不可的。
            assert forall|slot_index: int|
                0 <= slot_index < root_slots.len() && (#[trigger] root_slots[slot_index]) is Some
                    implies version_is_not_later_than(
                root_choice_key_of(root_slots[slot_index]).checkpoint_txg,
                root_choice_key_of(root_slots[slot_index]).instance,
                landed_checkpoint_txg(chosen_key.checkpoint_txg, records, outcome.applied@),
                landed_instance(chosen_key.instance, records, outcome.applied@),
            ) by {
                assert(root_choice_key_is_not_newer_than(
                    root_choice_key_of(root_slots[slot_index]),
                    chosen_key,
                ));
            }
        },
        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {},
        Err(
            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                counters: _,
            },
        ) => {},
    }
}

} // verus!

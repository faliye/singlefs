//! 编进产品的纯函数核：取前缀（`replay_core`）、择根（`choose_root_slot`）与扫环建表（`first_copy_of_each_record_key`），
//! 连同在手算得出的小例子上钉住这三个核的单测。
use super::*;

verus! {

// ===== 纯函数核 =====
// 记录标志位 0（D23（journal 的角色与格式） 已定项 14 第五条、D23（journal 的角色与格式） 已定项 17）。
fn record_ends_its_publish<RootSource>(record: &CoreRecord<RootSource>) -> (is_last_record: bool)
    ensures
        is_last_record == is_last_record_of_its_publish(*record),
{
    match record.place_in_publish {
        PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish => true,
        PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow => false,
    }
}

// 原文在 BTreeMap 上 range((instance, counter+1)..=(instance, u64::MAX))、take_while(txg <= 这一条的)、any(txg == 这一条的)；
// 这里从切片里这一条的下一个位置往后扫，靠入参按 (实例代号, 计数器) 严格升序与原文同义（requires 写着，证明用到它）。
fn readable_record_of_the_same_publish_follows<RootSource>(
    records: &[CoreRecord<RootSource>],
    index: usize,
) -> (follows: bool)
    requires
        index < records.len(),
        records_ascend_by_instance_and_counter(records@),
    ensures
        follows == later_record_of_the_same_publish_is_readable(records@, index as int),
{
    let record = &records[index];
    let mut later_index = index + 1;
    while later_index < records.len()
        invariant
            index < later_index <= records.len(),
            *record == records@[index as int],
            records_ascend_by_instance_and_counter(records@),
            forall|between: int|
                index < between < later_index && #[trigger] is_a_later_record_of_the_same_instance(
                    records@,
                    index as int,
                    between,
                ) ==> records@[between].checkpoint_txg.0 < records@[index as int].checkpoint_txg.0,
        decreases records.len() - later_index,
    {
        let later = &records[later_index];
        if later.instance.0 == record.instance.0 {
            // 入参按 (实例代号, 计数器) 严格升序：切片里在这一条之后的同实例记录，计数器一定更大，不用再比计数器。
            proof {
                lemma_slice_order_within_an_instance_follows_the_counter(
                    records@,
                    index as int,
                    later_index as int,
                );
            }
            if !(later.checkpoint_txg.0 <= record.checkpoint_txg.0) {
                proof {
                    assert forall|witness: int|
                        0 <= witness < records@.len() && #[trigger] is_a_later_record_of_the_same_instance(
                            records@,
                            index as int,
                            witness,
                        ) && records@[witness].checkpoint_txg == records@[index as int].checkpoint_txg
                            implies !(forall|between: int|
                        0 <= between < records@.len() && #[trigger] is_a_later_record_of_the_same_instance(
                            records@,
                            index as int,
                            between,
                        ) && records@[between].counter.0 < records@[witness].counter.0
                            ==> records@[between].checkpoint_txg.0
                            <= records@[index as int].checkpoint_txg.0) by {
                        assert(records@[later_index as int].instance == records@[index as int].instance);
                        lemma_slice_order_within_an_instance_follows_the_counter(
                            records@,
                            index as int,
                            witness,
                        );
                        lemma_slice_order_within_an_instance_follows_the_counter(
                            records@,
                            later_index as int,
                            witness,
                        );
                        assert(is_a_later_record_of_the_same_instance(
                            records@,
                            index as int,
                            later_index as int,
                        ));
                    }
                }
                return false;
            }
            if later.checkpoint_txg.0 == record.checkpoint_txg.0 {
                proof {
                    assert forall|between: int|
                        0 <= between < records@.len() && #[trigger] is_a_later_record_of_the_same_instance(
                            records@,
                            index as int,
                            between,
                        ) && records@[between].counter.0 < records@[later_index as int].counter.0
                            implies records@[between].checkpoint_txg.0
                        <= records@[index as int].checkpoint_txg.0 by {
                        lemma_slice_order_within_an_instance_follows_the_counter(
                            records@,
                            index as int,
                            between,
                        );
                        assert(records@[later_index as int].instance == records@[index as int].instance);
                        lemma_slice_order_within_an_instance_follows_the_counter(
                            records@,
                            between,
                            later_index as int,
                        );
                    }
                    assert(is_a_later_record_of_the_same_instance(
                        records@,
                        index as int,
                        later_index as int,
                    ));
                }
                return true;
            }
        }
        later_index += 1;
    }
    proof {
        assert forall|witness: int|
            0 <= witness < records@.len() && #[trigger] is_a_later_record_of_the_same_instance(
                records@,
                index as int,
                witness,
            ) implies records@[witness].checkpoint_txg != records@[index as int].checkpoint_txg by {
            lemma_slice_order_within_an_instance_follows_the_counter(records@, index as int, witness);
        }
    }
    false
}

// 所选根覆盖的最后一条记录的 jsn 计数器（D23（journal 的角色与格式） 已定项 14 注 1，读法乙）：所选根那次发布读得出的几条里带末条标志的那一条；
// 一条都不带 ⇒ Ok(None)；带标志的多于一条 ⇒ 认哪一条条款没写，报错并带出那几条的计数器。
fn counter_of_the_last_record_the_root_covers<RootSource>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    records: &[CoreRecord<RootSource>],
) -> (anchor: Result<Option<JournalCounter>, CoreFailure>)
    requires
        every_counter_has_a_successor(records@),
    ensures
        ({
            let flagged = last_record_flag_counters_of_the_chosen_roots_publish(
                records@,
                root_instance,
                root_checkpoint_txg,
                records@.len() as int,
            );
            match anchor {
                Ok(found) => flagged.len() < 2 && found == anchor_of_the_chain(
                    records@,
                    root_instance,
                    root_checkpoint_txg,
                ),
                Err(
                    CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                        counters,
                    },
                ) => flagged.len() >= 2 && counters@ == flagged,
                Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => false,
            }
        }),
        anchor matches Ok(Some(counter)) ==> counter.0 < u64::MAX,
{
    let mut counters_carrying_the_last_record_flag: Vec<JournalCounter> = Vec::new();
    let mut index = 0;
    while index < records.len()
        invariant
            index <= records.len(),
            every_counter_has_a_successor(records@),
            counters_carrying_the_last_record_flag@
                == last_record_flag_counters_of_the_chosen_roots_publish(
                records@,
                root_instance,
                root_checkpoint_txg,
                index as int,
            ),
            forall|position: int|
                0 <= position < counters_carrying_the_last_record_flag@.len()
                    ==> (#[trigger] counters_carrying_the_last_record_flag@[position]).0 < u64::MAX,
        decreases records.len() - index,
    {
        let record = &records[index];
        if record.instance.0 == root_instance.0 && record.checkpoint_txg.0 == root_checkpoint_txg.0
            && record_ends_its_publish(record) {
            counters_carrying_the_last_record_flag.push(record.counter);
        }
        index += 1;
    }
    if counters_carrying_the_last_record_flag.len() > 1 {
        return Err(
            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                counters: counters_carrying_the_last_record_flag,
            },
        );
    }
    if counters_carrying_the_last_record_flag.is_empty() {
        Ok(None)
    } else {
        Ok(Some(counters_carrying_the_last_record_flag[0]))
    }
}

// 前缀规则不跨实例边界（D23（journal 的角色与格式） 已定项 14 第 1 条），只有所选根自己那个实例、txg 比根大的记录是候选。
fn is_above_the_chosen_root<RootSource>(
    record: &CoreRecord<RootSource>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
) -> (is_above: bool)
    ensures
        is_above == above_the_chosen_root(*record, root_instance, root_checkpoint_txg),
{
    record.instance.0 == root_instance.0 && record.checkpoint_txg.0 > root_checkpoint_txg.0
}

// 攒着的这次发布里最大的事务号，与 start 取大。
fn maximum_transaction_of<RootSource>(
    records: &[CoreRecord<RootSource>],
    records_of_the_open_publish: &[usize],
    start: TransactionNumber,
) -> (maximum: TransactionNumber)
    requires
        forall|position: int|
            0 <= position < records_of_the_open_publish@.len()
                ==> (#[trigger] records_of_the_open_publish@[position]) < records.len(),
    ensures
        maximum.0 >= start.0,
        forall|position: int|
            0 <= position < records_of_the_open_publish@.len()
                ==> records@[#[trigger] records_of_the_open_publish@[position] as int].transaction.0
                <= maximum.0,
        maximum == start || exists|position: int|
            0 <= position < records_of_the_open_publish@.len()
                && records@[#[trigger] records_of_the_open_publish@[position] as int].transaction
                == maximum,
{
    let mut maximum = start;
    let mut position = 0;
    while position < records_of_the_open_publish.len()
        invariant
            position <= records_of_the_open_publish@.len(),
            forall|open_position: int|
                0 <= open_position < records_of_the_open_publish@.len()
                    ==> (#[trigger] records_of_the_open_publish@[open_position]) < records.len(),
            maximum.0 >= start.0,
            forall|open_position: int|
                0 <= open_position < position
                    ==> records@[#[trigger] records_of_the_open_publish@[open_position] as int].transaction.0
                    <= maximum.0,
            maximum == start || exists|open_position: int|
                0 <= open_position < position
                    && records@[#[trigger] records_of_the_open_publish@[open_position] as int].transaction
                    == maximum,
        decreases records_of_the_open_publish@.len() - position,
    {
        let transaction = records[records_of_the_open_publish[position]].transaction;
        if transaction.0 > maximum.0 {
            maximum = transaction;
        }
        position += 1;
    }
    maximum
}

// 一个候选过不过逐条判定。判的次序同原文，读盘次序跟着它：链首与 jsn 连续、反向链回调、读者规则与提交标记、末条之后同一次发布、点名单元回调；
// 前一判不过就不再往下判（也就不再调后面的回调）。
enum CandidateVerdict {
    Admitted,
    FailsACheckBeforeTheNamedUnits,
    NamedUnitsReportedUnverified,
}

#[allow(clippy::too_many_arguments, reason = "逐条判定要的输入各自独立；幽灵量五个只给证明用，收成结构体只会多一层没人验的名字")]
fn judge_the_candidate<RootSource, BackChain: Fn(usize) -> bool, NamedUnits: Fn(usize) -> bool>(
    records: &[CoreRecord<RootSource>],
    index: usize,
    records_of_the_open_publish: &[usize],
    expected_next: Option<(PrefixKernelInstanceGeneration, JournalCounter)>,
    chain_start_txg_without_anchor: PrefixKernelCheckpointTxg,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: &BackChain,
    named_units_verified: &NamedUnits,
    Ghost(admitted): Ghost<Seq<int>>,
    Ghost(applied): Ghost<Seq<int>>,
    Ghost(anchor): Ghost<Option<JournalCounter>>,
    Ghost(root_instance): Ghost<PrefixKernelInstanceGeneration>,
    Ghost(root_checkpoint_txg): Ghost<PrefixKernelCheckpointTxg>,
) -> (verdict: CandidateVerdict)
    requires
        index < records.len(),
        records_ascend_by_instance_and_counter(records@),
        every_counter_has_a_successor(records@),
        back_chain_broken.requires((index,)),
        named_units_verified.requires((index,)),
        root_checkpoint_txg.0 < u64::MAX,
        chain_start_txg_without_anchor.0 == root_checkpoint_txg.0 + 1,
        anchor matches Some(anchor_counter) ==> anchor_counter.0 < u64::MAX,
        expected_next == expected_next_after(records@, admitted, anchor, root_instance),
        forall|position: int|
            0 <= position < admitted.len() ==> 0 <= #[trigger] admitted[position] < records.len(),
        applied_is_the_chain_cut_at_its_last_publish_end(records@, admitted, applied),
        as_ints(records_of_the_open_publish@) == admitted.subrange(
            applied.len() as int,
            admitted.len() as int,
        ),
    ensures
        ({
            let previous = previous_in_the_chain(records@, admitted, admitted.len() as int);
            match verdict {
                CandidateVerdict::Admitted => passes_every_check_before_the_named_units(
                    records@,
                    index as int,
                    previous,
                    anchor,
                    root_instance,
                    root_checkpoint_txg,
                    *back_chain_broken,
                ) && obeys_the_reader_rules_and_the_commit_markers(records@[index as int], previous)
                    && named_units_reported_verified(
                    named_unit_verification,
                    *named_units_verified,
                    index as int,
                ),
                CandidateVerdict::FailsACheckBeforeTheNamedUnits => fails_a_check_before_the_named_units(
                    records@,
                    index as int,
                    previous,
                    anchor,
                    root_instance,
                    root_checkpoint_txg,
                    *back_chain_broken,
                ),
                CandidateVerdict::NamedUnitsReportedUnverified => passes_every_check_before_the_named_units(
                    records@,
                    index as int,
                    previous,
                    anchor,
                    root_instance,
                    root_checkpoint_txg,
                    *back_chain_broken,
                ) && named_units_reported_unverified(
                    named_unit_verification,
                    *named_units_verified,
                    index as int,
                ),
            }
        }),
{
    let record = &records[index];
    let ghost previous = previous_in_the_chain(records@, admitted, admitted.len() as int);
    proof {
        reveal(passes_every_check_before_the_named_units);
        reveal(fails_a_check_before_the_named_units);
        if as_ints(records_of_the_open_publish@).len() > 0 {
            assert(as_ints(records_of_the_open_publish@).last() == admitted.last());
        } else if admitted.len() > 0 {
            assert(applied.len() == admitted.len());
            assert(applied.last() == admitted.last());
        }
    }
    match expected_next {
        Some(expected_key) => {
            if record.instance.0 != expected_key.0.0 || record.counter.0 != expected_key.1.0 {
                return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
            }
        },
        // 锚点读不出：链首只能是 txg = 根的 txg + 1、本次发布内序号为 1 的那条。
        // 序号与 txg 一起在反向链回调之前判，次序同原文：链首序号不是 1 的不调反向链回调。下面「攒着的为空且序号不是 1」那一判
        // 管锚点读得出时接上的那一次与之后新开的每一次发布；它对这一支的链首也成立（走到这一支时攒着的一定是空的，下面这句由 Verus 证），
        // 这里先判掉，读盘次序才与原文相同。
        None => {
            proof {
                assert(records_of_the_open_publish@.len() == 0);
            }
            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0
                || record.ordinal_within_publish.0 != 1 {
                return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
            }
        },
    }
    // 反向链（D23（journal 的角色与格式） 已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断；三态由回调交成布尔。
    if back_chain_broken(index) {
        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
    }
    if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 {
        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
    }
    proof {
        if admitted.len() > 0 {
            assert(records@[admitted.last()].counter.0 < u64::MAX);
            assert(records@[index as int].instance == records@[admitted.last()].instance);
        } else if anchor is Some {
            assert(records@[index as int].instance == root_instance);
        }
        assert(continues_the_jsn_chain(
            records@[index as int],
            previous,
            anchor,
            root_instance,
            root_checkpoint_txg,
        ));
    }
    if !records_of_the_open_publish.is_empty() {
        let previous_record_of_the_open_publish = &records[records_of_the_open_publish[records_of_the_open_publish.len()
            - 1]];
        proof {
            assert(!is_last_record_of_its_publish(records@[admitted.last()]));
        }
        if record.checkpoint_txg.0 != previous_record_of_the_open_publish.checkpoint_txg.0 {
            return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
        }
        if u64::from(record.ordinal_within_publish.0) != u64::from(
            previous_record_of_the_open_publish.ordinal_within_publish.0,
        ) + 1 {
            return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
        }
        if !previous_record_of_the_open_publish.is_commit
            && record.transaction.0 != previous_record_of_the_open_publish.transaction.0 {
            return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
        }
    }
    if !record.is_commit && record_ends_its_publish(record) {
        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
    }
    if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(
        records,
        index,
    ) {
        return CandidateVerdict::FailsACheckBeforeTheNamedUnits;
    }
    let all_verified = match named_unit_verification {
        NamedUnitVerification::Verify => named_units_verified(index),
        NamedUnitVerification::Skip => true,
    };
    if !all_verified {
        return CandidateVerdict::NamedUnitsReportedUnverified;
    }
    CandidateVerdict::Admitted
}

// replay_journal 去掉读盘之后的那一截。
// 前置条件：入参按 (实例代号, 计数器) 严格升序（「末条之后同一次发布还有没有记录」那一判靠它与原文同义）；
// 计数器都不是 u64 的最大值（链上下一号要加一）；两个回调对每个下标都可调。条数不设界。
#[allow(clippy::too_many_arguments, reason = "两个回调是读盘的两处，摊成参数比包一层结构体好对照原文")]
pub fn replay_core<
    RootSource: Copy,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    chosen_root_source: RootSource,
    records: &[CoreRecord<RootSource>],
    in_flight_limit: usize,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
) -> (replayed: Result<CoreOutcome<RootSource>, CoreFailure>)
    requires
        records_ascend_by_instance_and_counter(records@),
        every_counter_has_a_successor(records@),
        forall|record_index: usize|
            record_index < records@.len() ==> #[trigger] back_chain_broken.requires((record_index,)),
        forall|record_index: usize|
            record_index < records@.len() ==> #[trigger] named_units_verified.requires(
                (record_index,),
            ),
    ensures
        replay_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records@,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
        // M3 幂等·记录级（「记录级」，I-8.4 的字面）
        replaying_an_applied_record_twice_leaves_the_root_as_once(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records@,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
        // M3 幂等·不动点（「不动点」）
        replaying_from_the_landed_root_is_a_fixed_point(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records@,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
        // M3 单调推论的 J1 那一步（版本序 (txg, 实例代号)）
        replay_lands_no_earlier_than_the_chosen_root(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records@,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
        // M3 单调推论第 4 步「恢复是确定的函数」
        replay_outcome_is_the_only_one_the_rules_allow(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records@,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            replayed,
        ),
{
    broadcast use {
        lemma_replay_core_result_replays_an_applied_record_twice_as_once,
        lemma_replay_core_result_is_a_fixed_point,
        lemma_replay_core_result_is_the_only_one_the_rules_allow,
    };
    let mut report = CoreReport {
        above_water: 0,
        prefix_applied: 0,
        verification_passed: 0,
        verification_failed: 0,
        maximum_applied_transaction: TransactionNumber(0),
    };
    let mut root_source_after_replay = chosen_root_source;
    // 这一遍只数条数，主循环里逐条再判同一个条件；原文 sort_by_key 没抄：入参已是那个次序。
    let mut counting_index = 0;
    while counting_index < records.len()
        invariant
            counting_index <= records.len(),
            report.above_water <= counting_index,
            report.above_water == candidates_before(
                records@,
                root_instance,
                root_checkpoint_txg,
                counting_index as int,
            ).len(),
        decreases records.len() - counting_index,
    {
        let record = &records[counting_index];
        if is_above_the_chosen_root(record, root_instance, root_checkpoint_txg) {
            report.above_water += 1;
        }
        counting_index += 1;
    }
    let root_own_record_counter = match counter_of_the_last_record_the_root_covers(
        root_instance,
        root_checkpoint_txg,
        records,
    ) {
        Ok(counter) => counter,
        Err(failure) => {
            return Err(failure);
        },
    };
    let mut expected_next: Option<(PrefixKernelInstanceGeneration, JournalCounter)> = None;
    if let Some(counter) = root_own_record_counter {
        expected_next = Some((root_instance, JournalCounter(counter.0 + 1)));
    }
    // 所选根的 checkpoint_txg 是盘上读来的 8 字节，可以是 u64 的最大值：加一溢出报损坏、不 panic（审阅第 36 条）。
    if root_checkpoint_txg.0 == u64::MAX {
        return Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor);
    }
    let chain_start_txg_without_anchor = PrefixKernelCheckpointTxg(root_checkpoint_txg.0 + 1);
    // 这次发布已经过了逐条判定、还没走到末条的记录的下标。
    let mut records_of_the_open_publish: Vec<usize> = Vec::new();
    let ghost mut admitted: Seq<int> = Seq::empty();
    let ghost mut applied: Seq<int> = Seq::empty();
    let ghost anchor = anchor_of_the_chain(records@, root_instance, root_checkpoint_txg);
    let mut records_taken_toward_the_in_flight_limit: usize = 0;
    let mut index = 0;
    proof {
        assert(expected_next == expected_next_after(records@, admitted, anchor, root_instance));
    }
    while index < records.len()
        invariant_except_break
            records_taken_toward_the_in_flight_limit == admitted.len(),
            report.verification_failed == 0,
            expected_next == expected_next_after(records@, admitted, anchor, root_instance),
        invariant
            index <= records.len(),
            root_checkpoint_txg.0 < u64::MAX,
            chain_start_txg_without_anchor.0 == root_checkpoint_txg.0 + 1,
            records_ascend_by_instance_and_counter(records@),
            every_counter_has_a_successor(records@),
            forall|record_index: usize|
                record_index < records@.len() ==> #[trigger] back_chain_broken.requires(
                    (record_index,),
                ),
            forall|record_index: usize|
                record_index < records@.len() ==> #[trigger] named_units_verified.requires(
                    (record_index,),
                ),
            anchor == anchor_of_the_chain(records@, root_instance, root_checkpoint_txg),
            anchor matches Some(anchor_counter) ==> anchor_counter.0 < u64::MAX,
            report.above_water == candidates_before(
                records@,
                root_instance,
                root_checkpoint_txg,
                records@.len() as int,
            ).len(),
            // 过了逐条判定的：到 index 为止的全部候选
            admitted == candidates_before(
                records@,
                root_instance,
                root_checkpoint_txg,
                index as int,
            ),
            admitted.len() <= in_flight_limit,
            forall|position_in_chain: int|
                0 <= position_in_chain < admitted.len() ==> #[trigger] admitted_record_passed_every_check(
                    records@,
                    admitted,
                    position_in_chain,
                    anchor,
                    root_instance,
                    root_checkpoint_txg,
                    named_unit_verification,
                    back_chain_broken,
                    named_units_verified,
                ),
            report.verification_passed == match named_unit_verification {
                NamedUnitVerification::Verify => admitted.len(),
                NamedUnitVerification::Skip => 0,
            },
            applied_is_the_chain_cut_at_its_last_publish_end(records@, admitted, applied),
            as_ints(records_of_the_open_publish@) == admitted.subrange(
                applied.len() as int,
                admitted.len() as int,
            ),
            maximum_transaction_is_the_largest_applied(
                records@,
                applied,
                report.maximum_applied_transaction,
            ),
            // 已施加的
            forall|position: int|
                0 <= position < applied.len() ==> 0 <= #[trigger] applied[position] < index,
            forall|earlier: int, later: int|
                0 <= earlier < later < applied.len() ==> applied[earlier] < applied[later],
            forall|position: int|
                0 <= position < applied.len() ==> above_the_chosen_root(
                    records@[#[trigger] applied[position]],
                    root_instance,
                    root_checkpoint_txg,
                ),
            is_concatenation_of_whole_publishes(records_at(records@, applied)),
            report.prefix_applied == applied.len(),
            root_source_after_replay == if applied.len() == 0 {
                chosen_root_source
            } else {
                records@[applied.last()].new_root_source
            },
            // 攒着的
            forall|position: int|
                0 <= position < records_of_the_open_publish@.len()
                    ==> (#[trigger] records_of_the_open_publish@[position]) < index,
            forall|earlier: int, later: int|
                0 <= earlier < later < records_of_the_open_publish@.len()
                    ==> records_of_the_open_publish@[earlier] < records_of_the_open_publish@[later],
            forall|applied_position: int, open_position: int|
                0 <= applied_position < applied.len() && 0 <= open_position
                    < records_of_the_open_publish@.len() ==> applied[applied_position]
                    < records_of_the_open_publish@[open_position],
            forall|position: int|
                0 <= position < records_of_the_open_publish@.len() ==> above_the_chosen_root(
                    records@[#[trigger] records_of_the_open_publish@[position] as int],
                    root_instance,
                    root_checkpoint_txg,
                ),
            is_open_publish(records_at(records@, as_ints(records_of_the_open_publish@))),
            applied.len() + records_of_the_open_publish@.len() <= index,
            // 还没接上链首（锚点读不出、一条都没收）时攒着的一定是空的：「锚点读不出时链首序号为 1」与「首条序号为 1」两判冗余靠这一条
            expected_next is None ==> records_of_the_open_publish@.len() == 0,
        ensures
            index <= records.len(),
            index == records.len() ==> report.verification_failed == 0,
            index < records.len() ==> above_the_chosen_root(
                records@[index as int],
                root_instance,
                root_checkpoint_txg,
            ) && the_candidate_stops_the_chain(
                records@,
                admitted,
                index as int,
                in_flight_limit,
                anchor,
                root_instance,
                root_checkpoint_txg,
                named_unit_verification,
                back_chain_broken,
                named_units_verified,
                report.verification_failed,
            ),
        decreases records.len() - index,
    {
        let record = &records[index];
        proof {
            lemma_candidates_lie_below_their_end_and_above_the_chosen_root(
                records@,
                root_instance,
                root_checkpoint_txg,
                index as int,
            );
        }
        if !is_above_the_chosen_root(record, root_instance, root_checkpoint_txg) {
            index += 1;
            continue ;
        }
        // 在飞上限（D23（journal 的角色与格式） 已定项 18）：一次恢复重放的前缀最多这么多条
        if records_taken_toward_the_in_flight_limit >= in_flight_limit {
            break ;
        }
        records_taken_toward_the_in_flight_limit += 1;
        match judge_the_candidate(
            records,
            index,
            records_of_the_open_publish.as_slice(),
            expected_next,
            chain_start_txg_without_anchor,
            named_unit_verification,
            &back_chain_broken,
            &named_units_verified,
            Ghost(admitted),
            Ghost(applied),
            Ghost(anchor),
            Ghost(root_instance),
            Ghost(root_checkpoint_txg),
        ) {
            CandidateVerdict::Admitted => {},
            CandidateVerdict::FailsACheckBeforeTheNamedUnits => {
                break ;
            },
            CandidateVerdict::NamedUnitsReportedUnverified => {
                report.verification_failed += 1;
                break ;
            },
        }
        let ghost previous = previous_in_the_chain(records@, admitted, admitted.len() as int);
        expected_next = Some((record.instance, JournalCounter(record.counter.0 + 1)));
        // 只数真验过的：关掉点名验证时这一格不加（fs-design.md 五条硬要求第 4 条：分支必须可观测）
        match named_unit_verification {
            NamedUnitVerification::Verify => {
                report.verification_passed += 1;
            },
            NamedUnitVerification::Skip => {},
        }
        let ghost open_before = records_of_the_open_publish@;
        let ghost admitted_before = admitted;
        records_of_the_open_publish.push(index);
        proof {
            admitted = admitted.push(index as int);
            lemma_candidates_lie_below_their_end_and_above_the_chosen_root(
                records@,
                root_instance,
                root_checkpoint_txg,
                index as int + 1,
            );
            assert(admitted == candidates_before(
                records@,
                root_instance,
                root_checkpoint_txg,
                index as int + 1,
            ));
            lemma_admitting_the_candidate_keeps_every_link(
                records@,
                admitted_before,
                index as int,
                anchor,
                root_instance,
                root_checkpoint_txg,
                named_unit_verification,
                back_chain_broken,
                named_units_verified,
            );
            assert(records_of_the_open_publish@ == open_before.push(index));
            lemma_admitting_the_candidate_extends_the_open_publish(
                records@,
                open_before,
                index,
                admitted_before,
                applied,
            );
        }
        // 末条没到就接着往下走，这次发布先不施加
        if !record_ends_its_publish(record) {
            index += 1;
            continue ;
        }
        report.prefix_applied += records_of_the_open_publish.len();
        let ghost maximum_before = report.maximum_applied_transaction;
        report.maximum_applied_transaction = maximum_transaction_of(
            records,
            records_of_the_open_publish.as_slice(),
            report.maximum_applied_transaction,
        );
        proof {
            let ghost applied_before = applied;
            let this_publish = as_ints(records_of_the_open_publish@);
            applied = applied + this_publish;
            lemma_appending_a_whole_publish_keeps_a_concatenation(records@, applied_before, this_publish);
            lemma_the_maximum_over_the_appended_publish(
                records@,
                applied_before,
                records_of_the_open_publish@,
                maximum_before,
                report.maximum_applied_transaction,
            );
            lemma_the_applied_head_and_the_open_tail_make_up_the_chain(
                admitted,
                applied_before,
                this_publish,
            );
            assert(applied.last() == index as int);
        }
        records_of_the_open_publish.clear();
        // 换根：核里只换「新根从哪来」这一项，RootRecord 的其余字段由调用方照原文拼
        root_source_after_replay = record.new_root_source;
        index += 1;
    }
    proof {
        lemma_candidates_before_a_shorter_end_are_a_prefix(
            records@,
            root_instance,
            root_checkpoint_txg,
            index as int,
            records@.len() as int,
        );
        if index < records.len() {
            lemma_candidates_before_a_shorter_end_are_a_prefix(
                records@,
                root_instance,
                root_checkpoint_txg,
                index as int + 1,
                records@.len() as int,
            );
            assert(candidates_before(records@, root_instance, root_checkpoint_txg, index as int + 1)
                == admitted.push(index as int));
            assert(candidates_before(
                records@,
                root_instance,
                root_checkpoint_txg,
                index as int + 1,
            )[admitted.len() as int] == index as int);
        }
        lemma_candidates_lie_below_their_end_and_above_the_chosen_root(
            records@,
            root_instance,
            root_checkpoint_txg,
            index as int,
        );
    }
    Ok(
        CoreOutcome {
            report,
            root_source_after_replay,
            admitted: Ghost(admitted),
            applied: Ghost(applied),
        },
    )
}

// recovery.rs 的 choose_root 去掉读槽之后的那一截：入参是根环三个区域全部槽按读根环的次序（区域号升序、区域内槽号升序）读出来的样子，
// 交回所选槽在入参里的下标。判的次序照提交 fcc4d340 的 choose_root：逐槽往后，比已择的严格新才换。
#[allow(clippy::manual_map, reason = "vstd 没给 Option::map 带闭包的规格，交回值写成 match 才验得了")]
pub fn choose_root_slot(root_slots: &[Option<RootChoiceKey>]) -> (chosen: Option<usize>)
    ensures
        root_choice_follows_the_newest_rule(root_slots@, chosen),
{
    let mut chosen: Option<(usize, RootChoiceKey)> = None;
    let mut slot_index: usize = 0;
    while slot_index < root_slots.len()
        invariant
            slot_index <= root_slots.len(),
            root_chosen_among_the_first_slots(
                root_slots@,
                slot_index as int,
                match chosen {
                    Some((chosen_slot, _)) => Some(chosen_slot),
                    None => None,
                },
            ),
            chosen matches Some((chosen_slot, chosen_key)) ==> chosen_key == root_choice_key_of(
                root_slots@[chosen_slot as int],
            ),
        decreases root_slots.len() - slot_index,
    {
        if let Some(candidate_key) = root_slots[slot_index] {
            let candidate_is_newer = match chosen {
                None => true,
                Some((_, chosen_key)) => candidate_key.checkpoint_txg.0 > chosen_key.checkpoint_txg.0
                    || (candidate_key.checkpoint_txg.0 == chosen_key.checkpoint_txg.0
                    && candidate_key.instance.0 > chosen_key.instance.0),
            };
            if candidate_is_newer {
                chosen = Some((slot_index, candidate_key));
            }
        }
        slot_index += 1;
    }
    match chosen {
        Some((chosen_slot, _)) => Some(chosen_slot),
        None => None,
    }
}

// recovery.rs 的 scan_journal 去掉读盘、解记录之后建表的那一截：入参是读到的记录按读到的次序摊成的 (实例代号, 计数器)，
// 交回每个键第一次读到的位置。原文在 BTreeMap 上 entry(key).or_insert(record)；vstd 的 BTreeMap 没有 entry / or_insert 的规格，
// 这里写成先查后插，与原文同义：键已在就不动，不在才插。
#[allow(clippy::map_entry, reason = "vstd 的 BTreeMap 没有 entry / or_insert 的规格，先查后插 Verus 才验得了")]
pub fn first_copy_of_each_record_key(scanned_keys: &[(u32, u64)]) -> (first_copies: BTreeMap<
    (u32, u64),
    usize,
>)
    ensures
        first_copies_follow_the_first_come_rule(
            scanned_keys@,
            scanned_keys@.len() as int,
            first_copies@,
        ),
        each_kept_copy_stands_for_one_record_key(first_copies@),
{
    broadcast use vstd::laws_cmp::group_laws_cmp;

    let mut first_copies: BTreeMap<(u32, u64), usize> = BTreeMap::new();
    let mut position: usize = 0;
    while position < scanned_keys.len()
        invariant
            position <= scanned_keys.len(),
            first_copies_follow_the_first_come_rule(scanned_keys@, position as int, first_copies@),
        decreases scanned_keys.len() - position,
    {
        let record_key = scanned_keys[position];
        if !first_copies.contains_key(&record_key) {
            first_copies.insert(record_key, position);
        }
        position += 1;
    }
    first_copies
}

// 构造见证：无 requires；确实施加一条完整发布，再从落点恢复保持不动。
// 联合证明 M3 蕴涵式的三个前件，避免空输入见证掩盖实际重放路径。
#[allow(clippy::vec_init_then_push, reason = "逐步构造非空记录序列，让见证直接使用 Vec::push 的规格")]
pub fn witness_nonempty_replay_and_its_fixed_point() {
    let back_chain = |_position: usize| -> (answer: bool)
        ensures answer == false,
    { false };
    let named_units = |_position: usize| -> (answer: bool)
        ensures answer == true,
    { true };
    let mut records = Vec::new();
    records.push(CoreRecord {
        instance: PrefixKernelInstanceGeneration(0),
        counter: JournalCounter(1),
        checkpoint_txg: PrefixKernelCheckpointTxg(1),
        transaction: TransactionNumber(1),
        is_commit: true,
        ordinal_within_publish: PrefixKernelJournalRecordOrdinalWithinPublish(1),
        place_in_publish: PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish,
        new_root_source: 1u64,
    });
    proof {
        reveal_with_fuel(candidates_before, 2);
        reveal_with_fuel(last_record_flag_counters_of_the_chosen_roots_publish, 2);
        assert(journal_txg_does_not_decrease_with_the_counter_within_an_instance(records@));
        assert(the_reader_answers_each_position_one_way(back_chain));
        assert(the_reader_answers_each_position_one_way(named_units));
        assert(records_above_the_chosen_root_fit_the_in_flight_limit(
            records@, PrefixKernelInstanceGeneration(0), PrefixKernelCheckpointTxg(0), 1));
    }
    let _first = replay_core(PrefixKernelInstanceGeneration(0), PrefixKernelCheckpointTxg(0),
        0u64, &records, 1, NamedUnitVerification::Verify, back_chain, named_units);
    proof {
        reveal_with_fuel(candidates_before, 2);
        reveal_with_fuel(last_record_flag_counters_of_the_chosen_roots_publish, 2);
        reveal(fails_a_check_before_the_named_units);
        reveal(passes_every_check_before_the_named_units);
        assert(_first is Ok);
        assert(_first->Ok_0.applied@.len() == 1);
        assert(_first->Ok_0.root_source_after_replay == 1);
    }
    let _again = replay_core(PrefixKernelInstanceGeneration(0), PrefixKernelCheckpointTxg(1),
        1u64, &records, 1, NamedUnitVerification::Verify, back_chain, named_units);
    proof {
        reveal_with_fuel(candidates_before, 2);
        reveal_with_fuel(last_record_flag_counters_of_the_chosen_roots_publish, 2);
        assert(_again is Ok);
        assert(_again->Ok_0.applied@.len() == 0);
        assert(_again->Ok_0.root_source_after_replay == 1);
    }
}

} // verus!

// 平时的 cargo test 跑的几条：Verus 证的是 spec，这几条钉住编进产品的那一份（取前缀、择根、扫环建表三个核）在手算得出的小例子上给出的数
// （crates/mutations.tsv 里点名它们的变异改坏核，它们红）。
#[cfg(test)]
mod tests {
    use super::{
        choose_root_slot, first_copy_of_each_record_key, replay_core, CoreRecord, JournalCounter,
        NamedUnitVerification, PrefixKernelCheckpointTxg, PrefixKernelInstanceGeneration,
        PrefixKernelJournalRecordOrdinalWithinPublish, PrefixKernelJournalRecordPlaceInPublish,
        RootChoiceKey, TransactionNumber,
    };

    const ROOT_INSTANCE: PrefixKernelInstanceGeneration = PrefixKernelInstanceGeneration(1);
    const ROOT_CHECKPOINT_TXG: PrefixKernelCheckpointTxg = PrefixKernelCheckpointTxg(1);
    const ROOT_SEGMENT: u64 = 0;
    const IN_FLIGHT_LIMIT: usize = 16;
    /// 每条记录的事务号取计数器加这个数：只为让事务号与计数器分得开，断言里的事务号从记录上取、不另算。
    const TRANSACTION_NUMBER_ABOVE_THE_COUNTER: u64 = 3;

    fn record(
        counter: u64,
        checkpoint_txg: u64,
        ordinal_within_publish: u32,
        place_in_publish: PrefixKernelJournalRecordPlaceInPublish,
        new_root_segment: u64,
    ) -> CoreRecord<u64> {
        CoreRecord {
            instance: ROOT_INSTANCE,
            counter: JournalCounter(counter),
            checkpoint_txg: PrefixKernelCheckpointTxg(checkpoint_txg),
            transaction: TransactionNumber(counter + TRANSACTION_NUMBER_ABOVE_THE_COUNTER),
            is_commit: true,
            ordinal_within_publish: PrefixKernelJournalRecordOrdinalWithinPublish(
                ordinal_within_publish,
            ),
            place_in_publish,
            new_root_source: new_root_segment,
        }
    }

    /// 所选根那次发布的末条（计数器 1）之后，txg 2 的发布两条都在、第二条带末条标志 ⇒ 整次施加，新根段取末条的。
    #[test]
    fn two_record_publish_after_the_anchor_is_applied_whole_and_the_new_root_segment_comes_from_its_last_record(
    ) {
        let records = [
            record(
                1,
                ROOT_CHECKPOINT_TXG.0,
                1,
                PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish,
                10,
            ),
            record(
                2,
                2,
                1,
                PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
                20,
            ),
            record(
                3,
                2,
                2,
                PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish,
                30,
            ),
        ];
        let Ok(outcome) = replay_core(
            ROOT_INSTANCE,
            ROOT_CHECKPOINT_TXG,
            ROOT_SEGMENT,
            &records,
            IN_FLIGHT_LIMIT,
            NamedUnitVerification::Skip,
            |_index: usize| false,
            |_index: usize| true,
        ) else {
            panic!("锚点只有一条、txg 不是最大值：核不该报错");
        };
        assert_eq!(outcome.report.above_water, 2, "水位之上是 txg 2 的两条");
        assert_eq!(outcome.report.prefix_applied, 2, "两条整次施加");
        assert_eq!(
            outcome.root_source_after_replay, 30,
            "新根段取那次发布末条的"
        );
        assert_eq!(
            outcome.report.maximum_applied_transaction, records[2].transaction,
            "事务号取施加的里最大的：那次发布末条的"
        );
    }

    /// 同样的锚点，txg 2 的发布只有第一条（不带末条标志）⇒ 末条没到，一条都不施加，新根段还是所选根的。
    #[test]
    fn publish_whose_last_record_is_missing_is_not_applied_and_the_root_segment_stays() {
        let records = [
            record(
                1,
                ROOT_CHECKPOINT_TXG.0,
                1,
                PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish,
                10,
            ),
            record(
                2,
                2,
                1,
                PrefixKernelJournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow,
                20,
            ),
        ];
        let Ok(outcome) = replay_core(
            ROOT_INSTANCE,
            ROOT_CHECKPOINT_TXG,
            ROOT_SEGMENT,
            &records,
            IN_FLIGHT_LIMIT,
            NamedUnitVerification::Skip,
            |_index: usize| false,
            |_index: usize| true,
        ) else {
            panic!("锚点只有一条、txg 不是最大值：核不该报错");
        };
        assert_eq!(outcome.report.above_water, 1, "水位之上是 txg 2 那一条");
        assert_eq!(outcome.report.prefix_applied, 0, "末条没到：一条都不施加");
        assert_eq!(
            outcome.root_source_after_replay, ROOT_SEGMENT,
            "新根段还是所选根的"
        );
    }

    fn root_choice_key(checkpoint_txg: u64, instance: u32) -> Option<RootChoiceKey> {
        Some(RootChoiceKey {
            checkpoint_txg: PrefixKernelCheckpointTxg(checkpoint_txg),
            instance: PrefixKernelInstanceGeneration(instance),
        })
    }

    /// 两个槽 checkpoint_txg 相同：实例代号高的那个在后面的槽里也择它（D22 已定项 7 平局实例代号高者赢）。
    #[test]
    fn of_two_slots_with_the_same_checkpoint_txg_the_higher_instance_in_the_later_slot_is_chosen() {
        let root_slots = [root_choice_key(4, 1), None, root_choice_key(4, 2)];
        assert_eq!(
            choose_root_slot(&root_slots),
            Some(2),
            "txg 平局，实例代号 2 高于 1：择槽 2"
        );
    }

    /// txg 大的赢，哪怕另一个槽的实例代号更高（D22 已定项 7 txg 为主、实例代号只破平局）。
    #[test]
    fn later_checkpoint_txg_is_chosen_over_a_higher_instance_with_an_earlier_checkpoint_txg() {
        let root_slots = [root_choice_key(2, 2), root_choice_key(3, 1)];
        assert_eq!(
            choose_root_slot(&root_slots),
            Some(1),
            "txg 3 新于 txg 2，实例代号 1 低于 2 也择槽 1"
        );
    }

    /// 两个槽择新键完全相同：取读根环次序里前面那个（与原来严格大于才换同义）。
    #[test]
    fn of_two_slots_with_the_same_choice_key_the_earlier_slot_is_chosen() {
        let root_slots = [None, root_choice_key(5, 3), root_choice_key(5, 3)];
        assert_eq!(
            choose_root_slot(&root_slots),
            Some(1),
            "键相同：留前面的槽 1"
        );
    }

    /// 第一个自证过的槽不是最新的：往后看完全部槽再择（D22 已定项 24 先验全部候选再择新）。
    #[test]
    fn newer_root_after_the_first_self_verified_slot_is_chosen_over_it() {
        let root_slots = [None, root_choice_key(2, 1), None, root_choice_key(3, 1)];
        assert_eq!(
            choose_root_slot(&root_slots),
            Some(3),
            "txg 3 新于 txg 2：择槽 3"
        );
    }

    /// 同一 (实例代号, 计数器) 读到两份：留第一份读到的位置，后读到的那一份不顶替它。
    #[test]
    fn of_two_scanned_copies_of_one_record_key_the_first_position_is_kept() {
        let scanned_keys = [(1, 7), (1, 8), (1, 7)];
        let first_copies = first_copy_of_each_record_key(&scanned_keys);
        assert_eq!(first_copies.get(&(1, 7)), Some(&0), "(1, 7) 第一份在位置 0");
        assert_eq!(
            first_copies.get(&(1, 8)),
            Some(&1),
            "(1, 8) 只有位置 1 那一份"
        );
        assert_eq!(first_copies.len(), 2, "两个键各留一份");
    }
}

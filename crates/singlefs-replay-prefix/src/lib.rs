//! 取 journal 前缀并施加的纯函数核：`singlefs-core` 的 `recovery::replay_journal` 取前缀那一截全部判定都在这里，
//! Verus 验的就是编进产品、被恢复路径调用的这一份（用户 2026-10-01 定「Verus 跑真实代码」；里程碑三七·3）。
//! 单独成一个 crate、只依赖 `vstd`：Verus 只编这一个文件，不编整个 `singlefs-core`（用户 2026-10-01 定拆）。
//!
//! 判定与判的次序照提交 fcc4d340 的 `crates/singlefs-core/src/recovery.rs` 里 `replay_journal` 搬过来：每一判断在哪、
//! 两个读盘回调在哪一判之前调，都与原文相同。
//! spec 照 D23（journal 的角色与格式）已定项 4、7、8、14、18、22 与 D13（验证路线）已定项 1 的 M2 那一行写（`replay_follows_the_prefix_rules`）：
//! 施加哪一截由逐条判定（链首与 jsn 严格连续、反向链、读者规则、提交标记、末条之后同一次发布、点名单元）与在飞上限一起定，
//! 报告的五个数与两种报错各有一句。
//! M3（幂等的记录级与不动点两读、单调推论里核自己的两步：落点不早于所选根、规则只容得下一个结果）写成 `replay_core` 的另四条 ensures，
//! 单调本身写成推论 `corollary_the_landed_version_does_not_go_back`，择根与写侧那两条前提写在它的 requires 里（D13 已定项 1 的 M3 一行）。
//! 读盘才判得出的两件（反向链、点名单元）由调用方给两个回调，核走到那一步才调。
//! spec 只说「核据以接受或断链的那一次回调交回了什么」，不管回调调了几次、按什么次序调；回调自己判得对不对也不在 Verus 里。
//! 块层读几次盘、按什么次序读，只在只读挂载（`mount_read_only`）的这几种形状上由 harness 用例
//! `crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs` 钉住：锚点读得出、两次单条发布都施加；
//! 锚点那条记录撕了；锚点读不出、链首序号不是 1；一次发布两条记录；候选在末条的提交标记上被断。
//! 别的形状（反向链断了、到了在飞上限、可写挂载走读阶段缓存、`recover` 不验点名单元那一臂）读几次盘没有用例钉。
//! 平时的 stable 编译用 crates.io 的 `vstd` 与 `verus_builtin_macros` 把 `verus!` 擦成普通 Rust；
//! 验的时候 Verus 用它自带的那一份，两边钉同一个发布（`Cargo.toml` 的 vstd 版本与 Verus 0.2026.09.20.aef82ed）。
//! 另两个纯函数核也在这里：择根（`choose_root_slot`，`recovery::choose_root` 取 `(checkpoint_txg, 实例代号)` 最大的那一截）与
//! 扫环建表（`first_copy_of_each_record_key`，`recovery::scan_journal` 同一 (实例代号, 计数器) 先到先得的那一截）；
//! 择根与取前缀的合判（C650 的 Verus 一半）写成推论 `corollary_recovery_replays_from_the_newest_self_verified_root`。
//! 整理的选段核与计划核及其四条性质（里程碑四二·4）在模块 [`compaction_plan_proofs`]：`singlefs-core` 不调它们，它的
//! `compaction_plan` 单测与它们逐项对拍。
//! Verus 的必红世界登记在同目录的 `verus-must-red-worlds.tsv`。
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use vstd::prelude::*;

pub mod compaction_plan_proofs;

verus! {

// ===== 核的数据 =====
// 核不依赖 singlefs-core 与 singlefs-format（Verus 只编这一个文件）。带 PrefixKernel 的四个类型与 singlefs-core 里不带它的那四个
// （address.rs 的 InstanceGeneration、CheckpointTxg，journal.rs 的 JournalRecordOrdinalWithinPublish、JournalRecordPlaceInPublish）
// 是同一个概念的两个类型，不是一处定义：胶水（recovery.rs 的 replay_journal）逐个手抄映射过来，两边加字段、加成员没有东西逼着同步。
// 没合成一处，是因为 core 在 journal.rs 里给后两个类型写了固有方法，外 crate 的类型上写不了；把那几个方法挪进本 crate 的 verus! 之外、
// core 改用这里的类型，是代码轮第三轮判决 U3 里用户没选的那一条（选的是维持改名、这里写实话）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrefixKernelInstanceGeneration(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalCounter(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrefixKernelCheckpointTxg(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransactionNumber(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrefixKernelJournalRecordOrdinalWithinPublish(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrefixKernelJournalRecordPlaceInPublish {
    LastRecordOfThePublish,
    MoreRecordsOfThePublishFollow,
}

// 关掉点名验证只供测试强制进入（.claude/rules/fs-design.md 五条硬要求第 2 条）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamedUnitVerification {
    Verify,
    Skip,
}

pub struct CoreRecord<RootSource> {
    pub instance: PrefixKernelInstanceGeneration,
    pub counter: JournalCounter,
    pub checkpoint_txg: PrefixKernelCheckpointTxg,
    pub transaction: TransactionNumber,
    pub is_commit: bool,
    pub ordinal_within_publish: PrefixKernelJournalRecordOrdinalWithinPublish,
    pub place_in_publish: PrefixKernelJournalRecordPlaceInPublish,
    // 核不看它：施加了一次发布之后把那次发布末条的这一项原样交回，调用方照它拼新根。
    pub new_root_source: RootSource,
}

pub struct CoreReport {
    pub above_water: usize,
    pub prefix_applied: usize,
    pub verification_passed: usize,
    pub verification_failed: usize,
    pub maximum_applied_transaction: TransactionNumber,
}

pub enum CoreFailure {
    RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided { counters: Vec<JournalCounter> },
    ChosenRootCheckpointTxgHasNoSuccessor,
}

// 根环一个自证过的槽的择新键（D22（单元原子性怎么合成）已定项 7）：择根核只看这两项，根记录别的字段留在胶水里。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootChoiceKey {
    pub checkpoint_txg: PrefixKernelCheckpointTxg,
    pub instance: PrefixKernelInstanceGeneration,
}

pub struct CoreOutcome<RootSource> {
    pub report: CoreReport,
    pub root_source_after_replay: RootSource,
    // 幽灵量，编译时擦掉。admitted：逐条判定都过了的记录在入参切片里的下标；applied：其中施加了的那一截。
    pub admitted: Ghost<Seq<int>>,
    pub applied: Ghost<Seq<int>>,
}

// ===== spec（可信：照条款文字写，没有别的东西核它）=====
pub open spec fn is_last_record_of_its_publish<RootSource>(record: CoreRecord<RootSource>) -> bool {
    record.place_in_publish is LastRecordOfThePublish
}

pub open spec fn key_is_below<RootSource>(
    lower: CoreRecord<RootSource>,
    upper: CoreRecord<RootSource>,
) -> bool {
    lower.instance.0 < upper.instance.0 || (lower.instance == upper.instance && lower.counter.0
        < upper.counter.0)
}

// 入参的次序：按 (实例代号, 计数器) 严格升序，与 BTreeMap<(PrefixKernelInstanceGeneration, u64), _> 的键序相同。
pub open spec fn records_ascend_by_instance_and_counter<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
) -> bool {
    forall|earlier: int, later: int|
        0 <= earlier < later < records.len() ==> #[trigger] key_is_below(
            records[earlier],
            records[later],
        )
}

// 计数器是盘上 6 字节读来的；加一不溢出（已定项 9 的 48 位）。
pub open spec fn every_counter_has_a_successor<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
) -> bool {
    forall|record_index: int|
        0 <= record_index < records.len() ==> (#[trigger] records[record_index]).counter.0
            < u64::MAX
}

// 已定项 14 定案句：只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录；注 1：实例代号与所选根不同即停。
pub open spec fn above_the_chosen_root<RootSource>(
    record: CoreRecord<RootSource>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
) -> bool {
    record.instance == root_instance && record.checkpoint_txg.0 > root_checkpoint_txg.0
}

// 切片里 [0, end) 之间在所选根之上的记录（候选）的下标，按切片次序。
pub open spec fn candidates_before<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    end: int,
) -> Seq<int>
    decreases end,
{
    if end <= 0 {
        Seq::empty()
    } else {
        let earlier = candidates_before(records, root_instance, root_checkpoint_txg, end - 1);
        if above_the_chosen_root(records[end - 1], root_instance, root_checkpoint_txg) {
            earlier.push(end - 1)
        } else {
            earlier
        }
    }
}

// 已定项 14 注 1（读法乙）：所选根那次发布（同实例、同 checkpoint_txg）读得出的记录里带末条标志的那几条的计数器。
pub open spec fn last_record_flag_counters_of_the_chosen_roots_publish<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    end: int,
) -> Seq<JournalCounter>
    decreases end,
{
    if end <= 0 {
        Seq::empty()
    } else {
        let earlier = last_record_flag_counters_of_the_chosen_roots_publish(
            records,
            root_instance,
            root_checkpoint_txg,
            end - 1,
        );
        let record = records[end - 1];
        if record.instance == root_instance && record.checkpoint_txg == root_checkpoint_txg
            && is_last_record_of_its_publish(record) {
            earlier.push(record.counter)
        } else {
            earlier
        }
    }
}

// 链首的锚点：带末条标志的恰好一条时是它；一条都没有就是「那条读不出」。
pub open spec fn anchor_of_the_chain<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
) -> Option<JournalCounter> {
    let flagged = last_record_flag_counters_of_the_chosen_roots_publish(
        records,
        root_instance,
        root_checkpoint_txg,
        records.len() as int,
    );
    if flagged.len() == 1 {
        Some(flagged[0])
    } else {
        None
    }
}

// 已定项 22 与已定项 14 注 1：jsn 严格连续（不等即止）；链首接在锚点之后，锚点读不出时链首的 txg = 所选根 + 1、本次发布内序号为 1。
pub open spec fn continues_the_jsn_chain<RootSource>(
    record: CoreRecord<RootSource>,
    previous: Option<CoreRecord<RootSource>>,
    anchor: Option<JournalCounter>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
) -> bool {
    match previous {
        Some(previous_record) => record.instance == previous_record.instance && record.counter.0
            == previous_record.counter.0 + 1,
        None => match anchor {
            Some(anchor_counter) => record.instance == root_instance && record.counter.0
                == anchor_counter.0 + 1,
            None => record.checkpoint_txg.0 == root_checkpoint_txg.0 + 1
                && record.ordinal_within_publish.0 == 1,
        },
    }
}

// 已定项 4 读者规则前三格（一次发布的首条序号是 1、一次发布之内不跳号、末条没到就换 txg 即缺了末条）与已定项 7
// （提交标记没出现就换了事务号 ⇒ 那个事务没写完；末条不带提交标记 ⇒ 这次发布最后一个事务没写完）。
pub open spec fn obeys_the_reader_rules_and_the_commit_markers<RootSource>(
    record: CoreRecord<RootSource>,
    previous: Option<CoreRecord<RootSource>>,
) -> bool {
    &&& match previous {
        Some(previous_record) => if is_last_record_of_its_publish(previous_record) {
            record.ordinal_within_publish.0 == 1
        } else {
            &&& record.checkpoint_txg == previous_record.checkpoint_txg
            &&& record.ordinal_within_publish.0 == previous_record.ordinal_within_publish.0 + 1
            &&& (previous_record.is_commit || record.transaction == previous_record.transaction)
        },
        None => record.ordinal_within_publish.0 == 1,
    }
    &&& (is_last_record_of_its_publish(record) ==> record.is_commit)
}

pub open spec fn is_a_later_record_of_the_same_instance<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    position: int,
    other: int,
) -> bool {
    records[other].instance == records[position].instance && records[other].counter.0
        > records[position].counter.0
}

// 已定项 4 读者规则第四格（C540）：同一 (实例代号, checkpoint_txg) 里这一条之后还有读得出的记录。
// 「之后」按计数器、不要求紧接着；往后看到 txg 越过这一条的就不再看——同实例 txg 随计数器不降（K10）时与「之后任一条同 txg」逐态相同
// （判决 research/prompts/m3-formal-r2-main-verification.md 第二节第 8 条）。
pub open spec fn a_later_record_of_the_same_publish_is_readable<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    position: int,
) -> bool {
    exists|later: int|
        0 <= later < records.len() && #[trigger] is_a_later_record_of_the_same_instance(
            records,
            position,
            later,
        ) && records[later].checkpoint_txg == records[position].checkpoint_txg && forall|
            between: int,
        |
            0 <= between < records.len() && #[trigger] is_a_later_record_of_the_same_instance(
                records,
                position,
                between,
            ) && records[between].counter.0 < records[later].counter.0
                ==> records[between].checkpoint_txg.0 <= records[position].checkpoint_txg.0
}

// 已定项 8 与 I-8.6：反向链对不上的那一条断。只说核据以接受或断链的那一次回调交回了什么。
pub open spec fn back_chain_reported_holding<BackChain: Fn(usize) -> bool>(
    back_chain_broken: BackChain,
    position: int,
) -> bool {
    back_chain_broken.ensures((position as usize,), false)
}

pub open spec fn back_chain_reported_broken<BackChain: Fn(usize) -> bool>(
    back_chain_broken: BackChain,
    position: int,
) -> bool {
    back_chain_broken.ensures((position as usize,), true)
}

// 已定项 14 五条口径第四条：一个点名单元的两份都读得出、校验和都对才施加。
pub open spec fn named_units_reported_verified<NamedUnits: Fn(usize) -> bool>(
    named_unit_verification: NamedUnitVerification,
    named_units_verified: NamedUnits,
    position: int,
) -> bool {
    match named_unit_verification {
        NamedUnitVerification::Verify => named_units_verified.ensures((position as usize,), true),
        NamedUnitVerification::Skip => true,
    }
}

pub open spec fn named_units_reported_unverified<NamedUnits: Fn(usize) -> bool>(
    named_unit_verification: NamedUnitVerification,
    named_units_verified: NamedUnits,
    position: int,
) -> bool {
    named_unit_verification is Verify && named_units_verified.ensures((position as usize,), false)
}

#[verifier::opaque]
pub open spec fn passes_every_check_before_the_named_units<
    RootSource,
    BackChain: Fn(usize) -> bool,
>(
    records: Seq<CoreRecord<RootSource>>,
    position: int,
    previous: Option<CoreRecord<RootSource>>,
    anchor: Option<JournalCounter>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    back_chain_broken: BackChain,
) -> bool {
    &&& continues_the_jsn_chain(
        records[position],
        previous,
        anchor,
        root_instance,
        root_checkpoint_txg,
    )
    &&& back_chain_reported_holding(back_chain_broken, position)
    &&& obeys_the_reader_rules_and_the_commit_markers(records[position], previous)
    &&& !(is_last_record_of_its_publish(records[position])
        && a_later_record_of_the_same_publish_is_readable(records, position))
}

#[verifier::opaque]
pub open spec fn fails_a_check_before_the_named_units<RootSource, BackChain: Fn(usize) -> bool>(
    records: Seq<CoreRecord<RootSource>>,
    position: int,
    previous: Option<CoreRecord<RootSource>>,
    anchor: Option<JournalCounter>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    back_chain_broken: BackChain,
) -> bool {
    ||| !continues_the_jsn_chain(
        records[position],
        previous,
        anchor,
        root_instance,
        root_checkpoint_txg,
    )
    ||| back_chain_reported_broken(back_chain_broken, position)
    ||| !obeys_the_reader_rules_and_the_commit_markers(records[position], previous)
    ||| (is_last_record_of_its_publish(records[position])
        && a_later_record_of_the_same_publish_is_readable(records, position))
}

pub open spec fn previous_in_the_chain<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    admitted: Seq<int>,
    position_in_chain: int,
) -> Option<CoreRecord<RootSource>> {
    if position_in_chain == 0 {
        None
    } else {
        Some(records[admitted[position_in_chain - 1]])
    }
}

#[verifier::opaque]
pub open spec fn admitted_record_passed_every_check<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    records: Seq<CoreRecord<RootSource>>,
    admitted: Seq<int>,
    position_in_chain: int,
    anchor: Option<JournalCounter>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
) -> bool {
    &&& passes_every_check_before_the_named_units(
        records,
        admitted[position_in_chain],
        previous_in_the_chain(records, admitted, position_in_chain),
        anchor,
        root_instance,
        root_checkpoint_txg,
        back_chain_broken,
    )
    &&& named_units_reported_verified(
        named_unit_verification,
        named_units_verified,
        admitted[position_in_chain],
    )
}

// 下一个候选 next 让这一串停下：到了在飞上限（已定项 18），或它有一判不过——点名单元那一判不过时 verification_failed 记 1。
pub open spec fn the_candidate_stops_the_chain<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    records: Seq<CoreRecord<RootSource>>,
    admitted: Seq<int>,
    next: int,
    in_flight_limit: usize,
    anchor: Option<JournalCounter>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
    verification_failed: usize,
) -> bool {
    let previous = previous_in_the_chain(records, admitted, admitted.len() as int);
    if admitted.len() >= in_flight_limit {
        verification_failed == 0
    } else {
        ||| verification_failed == 0 && fails_a_check_before_the_named_units(
            records,
            next,
            previous,
            anchor,
            root_instance,
            root_checkpoint_txg,
            back_chain_broken,
        )
        ||| verification_failed == 1 && passes_every_check_before_the_named_units(
            records,
            next,
            previous,
            anchor,
            root_instance,
            root_checkpoint_txg,
            back_chain_broken,
        ) && named_units_reported_unverified(named_unit_verification, named_units_verified, next)
    }
}

// 已定项 14 第五条：施加的单位是一次发布——施加的是过了逐条判定的那一串里到最后一个末条为止的那一截。
pub open spec fn applied_is_the_chain_cut_at_its_last_publish_end<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    admitted: Seq<int>,
    applied: Seq<int>,
) -> bool {
    &&& applied.len() <= admitted.len()
    &&& forall|position: int|
        0 <= position < applied.len() ==> #[trigger] applied[position] == admitted[position]
    &&& (applied.len() == 0 || is_last_record_of_its_publish(records[applied.last()]))
    &&& forall|position_in_chain: int|
        applied.len() <= position_in_chain < admitted.len() ==> !is_last_record_of_its_publish(
            records[#[trigger] admitted[position_in_chain]],
        )
}

// 一次整的发布：checkpoint_txg 相同、序号依次 1..k、只有第 k 条带末条标志（已定项 4 / 14 第五条 / 17）。
pub open spec fn is_whole_publish<RootSource>(publish: Seq<CoreRecord<RootSource>>) -> bool {
    &&& publish.len() >= 1
    &&& forall|position: int|
        0 <= position < publish.len() ==> (#[trigger] publish[position]).checkpoint_txg
            == publish[0].checkpoint_txg
    &&& forall|position: int|
        0 <= position < publish.len() ==> (#[trigger] publish[position]).ordinal_within_publish.0 as int
            == position + 1
    &&& forall|position: int|
        0 <= position < publish.len() ==> (is_last_record_of_its_publish(
            #[trigger] publish[position],
        ) <==> position == publish.len() - 1)
}

// 若干整次发布的拼接（M2：重放结果等于整数个发布，不存在被前缀切开的发布）。
pub open spec fn is_concatenation_of_whole_publishes<RootSource>(
    applied: Seq<CoreRecord<RootSource>>,
) -> bool
    decreases applied.len(),
{
    applied.len() == 0 || exists|cut: int|
        0 <= cut < applied.len() && is_whole_publish(
            #[trigger] applied.subrange(cut, applied.len() as int),
        ) && is_concatenation_of_whole_publishes(applied.subrange(0, cut))
}

pub open spec fn records_at<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    indices: Seq<int>,
) -> Seq<CoreRecord<RootSource>> {
    Seq::new(indices.len(), |position: int| records[indices[position]])
}

// D13 已定项 1 的 M2：施加的是若干整次发布、都在所选根之上；报告的条数与交回的根跟着施加的走。
pub open spec fn applied_records_are_whole_publishes_above_the_chosen_root<RootSource>(
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    chosen_root_source: RootSource,
    records: Seq<CoreRecord<RootSource>>,
    outcome: CoreOutcome<RootSource>,
) -> bool {
    let applied = outcome.applied@;
    &&& forall|position: int|
        0 <= position < applied.len() ==> 0 <= #[trigger] applied[position] < records.len()
    &&& forall|earlier: int, later: int|
        0 <= earlier < later < applied.len() ==> applied[earlier] < applied[later]
    &&& forall|position: int|
        0 <= position < applied.len() ==> above_the_chosen_root(
            records[#[trigger] applied[position]],
            root_instance,
            root_checkpoint_txg,
        )
    &&& is_concatenation_of_whole_publishes(records_at(records, applied))
    &&& outcome.report.prefix_applied == applied.len()
    &&& outcome.root_source_after_replay == if applied.len() == 0 {
        chosen_root_source
    } else {
        records[applied.last()].new_root_source
    }
}

// 报告的最大事务号（可写挂载算 W 的输入，已定项 14 注 4）：施加的记录里最大的那个，一条都没施加时 0。
pub open spec fn maximum_transaction_is_the_largest_applied<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    applied: Seq<int>,
    maximum: TransactionNumber,
) -> bool {
    &&& forall|position: int|
        0 <= position < applied.len() ==> (#[trigger] records[applied[position]]).transaction.0
            <= maximum.0
    &&& (maximum.0 == 0 || exists|position: int|
        0 <= position < applied.len() && (#[trigger] records[applied[position]]).transaction
            == maximum)
}

pub open spec fn outcome_follows_the_prefix_rules<
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
) -> bool {
    let candidates = candidates_before(
        records,
        root_instance,
        root_checkpoint_txg,
        records.len() as int,
    );
    let anchor = anchor_of_the_chain(records, root_instance, root_checkpoint_txg);
    let admitted = outcome.admitted@;
    let applied = outcome.applied@;
    // 过了逐条判定的是候选从头起的一段（不跳过候选）、不超过在飞上限
    &&& admitted.len() <= candidates.len()
    &&& forall|position_in_chain: int|
        0 <= position_in_chain < admitted.len() ==> #[trigger] admitted[position_in_chain]
            == candidates[position_in_chain]
    &&& admitted.len() <= in_flight_limit
    &&& forall|position_in_chain: int|
        0 <= position_in_chain < admitted.len() ==> #[trigger] admitted_record_passed_every_check(
            records,
            admitted,
            position_in_chain,
            anchor,
            root_instance,
            root_checkpoint_txg,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
        )
    // 停在哪有理由：候选取完，或下一个候选让它停下
    &&& if admitted.len() == candidates.len() {
        outcome.report.verification_failed == 0
    } else {
        the_candidate_stops_the_chain(
            records,
            admitted,
            candidates[admitted.len() as int],
            in_flight_limit,
            anchor,
            root_instance,
            root_checkpoint_txg,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            outcome.report.verification_failed,
        )
    }
    &&& applied_is_the_chain_cut_at_its_last_publish_end(records, admitted, applied)
    &&& applied_records_are_whole_publishes_above_the_chosen_root(
        root_instance,
        root_checkpoint_txg,
        chosen_root_source,
        records,
        outcome,
    )
    &&& outcome.report.above_water == candidates.len()
    &&& outcome.report.verification_passed == match named_unit_verification {
        NamedUnitVerification::Verify => admitted.len(),
        NamedUnitVerification::Skip => 0,
    }
    &&& maximum_transaction_is_the_largest_applied(
        records,
        applied,
        outcome.report.maximum_applied_transaction,
    )
}

// replay_core 的整个结论：两种报错各在什么时候报，不报错时施加的是哪一截、报告的每个数是什么。
pub open spec fn replay_follows_the_prefix_rules<
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
) -> bool {
    let flagged = last_record_flag_counters_of_the_chosen_roots_publish(
        records,
        root_instance,
        root_checkpoint_txg,
        records.len() as int,
    );
    match replayed {
        // 带末条标志的多于一条：锚点认哪一条条款没写，在施加任何一条之前停下（已定项 14「这一版的失败处置」）。
        Err(
            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                counters,
            },
        ) => flagged.len() >= 2 && counters@ == flagged,
        // 所选根的 checkpoint_txg 是盘上读来的 8 字节，最大值时接在它后面的发布取不到 txg（审阅第 36 条）。
        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => flagged.len() < 2
            && root_checkpoint_txg.0 == u64::MAX,
        Ok(outcome) => flagged.len() < 2 && root_checkpoint_txg.0 < u64::MAX
            && outcome_follows_the_prefix_rules(
            root_instance,
            root_checkpoint_txg,
            chosen_root_source,
            records,
            in_flight_limit,
            named_unit_verification,
            back_chain_broken,
            named_units_verified,
            outcome,
        ),
    }
}

// ===== M3 幂等与单调（spec，可信：照判决文字写）=====
// D13（验证路线）已定项 1 的 M3 一行；题面照第二轮判决 research/prompts/m3-formal-r2-main-verification.md 第二节第 5、6、7 行
// 与第三轮判决 research/prompts/m3-formal-r3-main-verification.md 第三节（用户 2026-10-01 定 S5 版本序、S6 txg 顶格）。
// 主语是 replay_core：singlefs-core 里以 verify_named_units = true 调 replay_journal 的三处走到的就是它（第二轮判决第 7 行）。
// 下面四条都写成 replay_core 的 ensures（证在 M3 的引理那一节），说的是编进产品的这一份交回的结果；单调本身是推论 corollary_the_landed_version_does_not_go_back。

// I-8.12（K10，判决 research/prompts/m3-inv-b-r1-main-verification.md 第二节第 9 行）：同一实例里 journal 记录的 txg 随计数器不减。
// 核不判它：记录级的第二半与不动点把它写成前提（核拿到的记录表满足它）。
pub open spec fn journal_txg_does_not_decrease_with_the_counter_within_an_instance<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
) -> bool {
    forall|earlier: int, later: int|
        0 <= earlier < records.len() && 0 <= later < records.len()
            && #[trigger] is_a_later_record_of_the_same_instance(records, earlier, later)
            ==> records[earlier].checkpoint_txg.0 <= records[later].checkpoint_txg.0
}

// 「同一个读者」（第二轮判决第 6 行不动点那一读的前提）：读盘回调在同一个下标上只交回一种答案。
pub open spec fn the_reader_answers_each_position_one_way<Reader: Fn(usize) -> bool>(
    reader: Reader,
) -> bool {
    forall|position: usize|
        #[trigger] reader.ensures((position,), true) ==> !reader.ensures((position,), false)
}

// D23 已定项 18「一次恢复重放的前缀最多 N 条」：不动点的前提「所选根之上的记录条数不超在飞上限」。
pub open spec fn records_above_the_chosen_root_fit_the_in_flight_limit<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    in_flight_limit: usize,
) -> bool {
    candidates_before(records, root_instance, root_checkpoint_txg, records.len() as int).len()
        <= in_flight_limit
}

// 落到的那一版：一条没施加时是所选根；施加了时是最后施加的那条记录的 (实例代号, checkpoint_txg)
// （D23 已定项 15：由记录重建的根取那次发布的 txg）。
pub open spec fn landed_checkpoint_txg<RootSource>(
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    records: Seq<CoreRecord<RootSource>>,
    applied: Seq<int>,
) -> PrefixKernelCheckpointTxg {
    if applied.len() == 0 {
        root_checkpoint_txg
    } else {
        records[applied.last()].checkpoint_txg
    }
}

pub open spec fn landed_instance<RootSource>(
    root_instance: PrefixKernelInstanceGeneration,
    records: Seq<CoreRecord<RootSource>>,
    applied: Seq<int>,
) -> PrefixKernelInstanceGeneration {
    if applied.len() == 0 {
        root_instance
    } else {
        records[applied.last()].instance
    }
}

// S5：版本序取 (txg, 实例代号)，txg 为主、实例代号破平局，与 crates/singlefs-core/src/recovery.rs 择根键
// (candidate.checkpoint_txg, candidate.instance) 同序。
pub open spec fn version_is_not_later_than(
    earlier_checkpoint_txg: PrefixKernelCheckpointTxg,
    earlier_instance: PrefixKernelInstanceGeneration,
    later_checkpoint_txg: PrefixKernelCheckpointTxg,
    later_instance: PrefixKernelInstanceGeneration,
) -> bool {
    earlier_checkpoint_txg.0 < later_checkpoint_txg.0 || (earlier_checkpoint_txg.0
        == later_checkpoint_txg.0 && earlier_instance.0 <= later_instance.0)
}

// 施加一条记录对根的作用：一次发布的末条把根换成它带的新根段，其余各条不动根（已定项 14 第五条：施加的单位是一次发布）。
pub open spec fn root_source_after_applying_the_record<RootSource>(
    root_source: RootSource,
    record: CoreRecord<RootSource>,
) -> RootSource {
    if is_last_record_of_its_publish(record) {
        record.new_root_source
    } else {
        root_source
    }
}

pub open spec fn root_source_after_applying_in_order<RootSource>(
    root_source: RootSource,
    applied_records: Seq<CoreRecord<RootSource>>,
) -> RootSource
    decreases applied_records.len(),
{
    if applied_records.len() == 0 {
        root_source
    } else {
        root_source_after_applying_the_record(
            root_source_after_applying_in_order(root_source, applied_records.drop_last()),
            applied_records.last(),
        )
    }
}

// M3 幂等·记录级（第二轮判决第二节第 6 行「记录级」，I-8.4（重放幂等）的字面「任一 journal 记录被重放两次与重放一次，产生的盘上状态相同」）：
// ① 交回的根就是按次序施加那几条的结果，其中任一条紧接着再施加一次，根与施加一次相同；
// ② 在 I-8.12 之下，施加过的每一条都不在落到的那一版之上：再取一次前缀时它不是候选，不会被第二次施加。
#[verifier::opaque]
pub open spec fn replaying_an_applied_record_twice_leaves_the_root_as_once<
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
) -> bool {
    replayed matches Ok(outcome) ==> {
        let applied = outcome.applied@;
        let applied_records = records_at(records, applied);
        &&& outcome.root_source_after_replay == root_source_after_applying_in_order(
            chosen_root_source,
            applied_records,
        )
        &&& forall|position: int|
            0 <= position < applied_records.len() ==> #[trigger] root_source_after_applying_in_order(
                chosen_root_source,
                applied_records.insert(position + 1, applied_records[position]),
            ) == outcome.root_source_after_replay
        &&& journal_txg_does_not_decrease_with_the_counter_within_an_instance(records)
            ==> forall|position: int|
            0 <= position < applied.len() ==> !above_the_chosen_root(
                records[#[trigger] applied[position]],
                landed_instance(root_instance, records, applied),
                landed_checkpoint_txg(root_checkpoint_txg, records, applied),
            )
    }
}

// 不动点那一次取前缀的结局（S6 取 Qn）：一条都不施加、交回的根段就是落到的那一版的；或者因落到的那一版 txg 顶格
// 报「所选根 txg 没有后继」——报错就一条都没施加，算相等。不许报别的错。
pub open spec fn replaying_again_from_the_landed_root_stays_put<RootSource>(
    landed_txg: PrefixKernelCheckpointTxg,
    landed_root_source: RootSource,
    replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>,
) -> bool {
    match replayed_again {
        Ok(again) => again.applied@.len() == 0 && again.root_source_after_replay
            == landed_root_source,
        Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => landed_txg.0 == u64::MAX,
        Err(
            CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {
                counters: _,
            },
        ) => false,
    }
}

// M3 幂等·不动点（第二轮判决第二节第 6 行「不动点」，第三轮 S6）：把落到的那一版当所选根，同一张记录表、同一个读者、同一个开关、
// 同一个在飞上限再取一次前缀（第二次取前缀的结果是规则容得下的任一个），根不动。
// 前提照第二轮判决第 6 行辩方那一句：I-8.12；同一个读者；所选根之上的记录条数不超在飞上限。
// 「同一 (实例代号, 计数器) 只对应一份内容」由 replay_core 的入参严格升序担保；txg 不顶格那一条照 S6 不当前提，写进结局。
#[verifier::opaque]
pub open spec fn replaying_from_the_landed_root_is_a_fixed_point<
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
) -> bool {
    replayed matches Ok(outcome) ==> (journal_txg_does_not_decrease_with_the_counter_within_an_instance(
        records,
    ) && the_reader_answers_each_position_one_way(back_chain_broken)
        && the_reader_answers_each_position_one_way(named_units_verified)
        && records_above_the_chosen_root_fit_the_in_flight_limit(
        records,
        root_instance,
        root_checkpoint_txg,
        in_flight_limit,
    ) ==> forall|replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>|
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
        ) ==> replaying_again_from_the_landed_root_stays_put(
            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
            outcome.root_source_after_replay,
            replayed_again,
        ))
}

// M3 单调（推论，第二轮判决第二节第 5 行、第三轮 S5）里核自己的两半，推论本身是下面的 corollary_the_landed_version_does_not_go_back。
// 一半（攻方 3.2 用的 J1）：落到的那一版不早于所选根，按 S5 的 (txg, 实例代号) 序。
pub open spec fn replay_lands_no_earlier_than_the_chosen_root<
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
) -> bool {
    replayed matches Ok(outcome) ==> version_is_not_later_than(
        root_checkpoint_txg,
        root_instance,
        landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
        landed_instance(root_instance, records, outcome.applied@),
    )
}

// 另一半（攻方 3.2 第 4 步「恢复是确定的函数」）：同样的入参、同一个读者，规则只容得下这一个结果。
#[verifier::opaque]
pub open spec fn replay_outcome_is_the_only_one_the_rules_allow<
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
) -> bool {
    replayed matches Ok(outcome) ==> (the_reader_answers_each_position_one_way(back_chain_broken)
        && the_reader_answers_each_position_one_way(named_units_verified) ==> forall|
        replayed_again: Result<CoreOutcome<RootSource>, CoreFailure>,
    |
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
        ) ==> (replayed_again matches Ok(again) && again.admitted@ == outcome.admitted@
            && again.applied@ == outcome.applied@ && again.root_source_after_replay
            == outcome.root_source_after_replay))
}

// ===== 择根（spec，可信：照条款文字写）=====
// D22（单元原子性怎么合成）已定项 7：择新按 (checkpoint_txg, 实例代号)，txg 为主、平局时实例代号高者赢；
// 已定项 24：先逐个验证全部候选、再在有效者中择新。核的入参是根环三个区域全部槽按读根环的次序读出来的样子：
// 自证过的槽给它的择新键，读不出、自证不过的槽给 None；读盘、自证校验和、fsid 由胶水（recovery.rs 的 choose_root）做，不在 Verus 里。
// 择新序就是上面 M3 单调用的 version_is_not_later_than（第三轮 S5 定它与择根同序），这里不另写一个序。
pub open spec fn root_choice_key_of(slot: Option<RootChoiceKey>) -> RootChoiceKey {
    match slot {
        Some(choice_key) => choice_key,
        None => RootChoiceKey {
            checkpoint_txg: PrefixKernelCheckpointTxg(0),
            instance: PrefixKernelInstanceGeneration(0),
        },
    }
}

pub open spec fn root_choice_key_is_not_newer_than(
    older: RootChoiceKey,
    newer: RootChoiceKey,
) -> bool {
    version_is_not_later_than(
        older.checkpoint_txg,
        older.instance,
        newer.checkpoint_txg,
        newer.instance,
    )
}

// 前 end 个槽里择出的那一个：一个自证过的都没有就不择；有就择择新序最大的；最大的不止一个槽（键相同）时取槽序最前的
// （条款没写键相同取哪一个，这一句照 recovery.rs 原来「严格大于才换」的行为写）。
pub open spec fn root_chosen_among_the_first_slots(
    root_slots: Seq<Option<RootChoiceKey>>,
    end: int,
    chosen: Option<usize>,
) -> bool {
    match chosen {
        None => forall|slot_index: int|
            0 <= slot_index < end ==> (#[trigger] root_slots[slot_index]) is None,
        Some(chosen_slot) => {
            &&& 0 <= chosen_slot < end
            &&& root_slots[chosen_slot as int] is Some
            &&& forall|slot_index: int|
                0 <= slot_index < end && (#[trigger] root_slots[slot_index]) is Some
                    ==> root_choice_key_is_not_newer_than(
                    root_choice_key_of(root_slots[slot_index]),
                    root_choice_key_of(root_slots[chosen_slot as int]),
                )
            &&& forall|slot_index: int|
                0 <= slot_index < chosen_slot && (#[trigger] root_slots[slot_index]) is Some
                    ==> !root_choice_key_is_not_newer_than(
                    root_choice_key_of(root_slots[chosen_slot as int]),
                    root_choice_key_of(root_slots[slot_index]),
                )
        },
    }
}

pub open spec fn root_choice_follows_the_newest_rule(
    root_slots: Seq<Option<RootChoiceKey>>,
    chosen: Option<usize>,
) -> bool {
    root_chosen_among_the_first_slots(root_slots, root_slots.len() as int, chosen)
}

// ===== 扫环建表（spec，可信）=====
// recovery.rs 的 scan_journal 按读者的盘次序、每块盘环内偏移次序读到的记录，同一 (实例代号, 计数器) 先到先得：第一份读到的留下
// （第三轮判决 research/prompts/m3-formal-r3-main-verification.md 第二节 S3、第四节第 10 条；replay_core 入参「严格升序」与 M2 语义层
// 「同一 (实例代号, 计数器) 只对应一份内容」用的就是这张表）。核只管「每个键留第几份」：入参是读到的记录按读到的次序摊成的键，
// 交回每个键第一次读到的位置；读盘、解记录由胶水做。
// 键摊成 (u32, u64) 两个无符号整数，不用 PrefixKernelInstanceGeneration 与 JournalCounter：vstd 给 BTreeMap 的规格只在键的比较守
// obeys_cmp 时成立，vstd 只为整数与整数的元组证了这一条（vstd::laws_cmp），这两个 newtype derive 出来的 Ord 没有。
pub open spec fn first_copies_follow_the_first_come_rule(
    scanned_keys: Seq<(u32, u64)>,
    end: int,
    first_copies: Map<(u32, u64), usize>,
) -> bool {
    // 表里的键正好是前 end 份读到的键（读到的每个键都在、没读到的一个不在）
    &&& forall|record_key: (u32, u64)|
        #[trigger] first_copies.contains_key(record_key) <==> exists|position: int|
            0 <= position < end && scanned_keys[position] == record_key
    // 先到先得：每个键记的是它第一次读到的那一份
    &&& forall|record_key: (u32, u64)|
        #[trigger] first_copies.contains_key(record_key) ==> {
            &&& 0 <= first_copies[record_key] < end
            &&& scanned_keys[first_copies[record_key] as int] == record_key
            &&& forall|earlier: int|
                0 <= earlier < first_copies[record_key] ==> #[trigger] scanned_keys[earlier]
                    != record_key
        }
}

// (实例代号, 计数器) 唯一：表里不同的键留的是不同的那一份（同一份读到的记录不会顶两个键）。
pub open spec fn each_kept_copy_stands_for_one_record_key(
    first_copies: Map<(u32, u64), usize>,
) -> bool {
    forall|one_key: (u32, u64), other_key: (u32, u64)|
        #[trigger] first_copies.contains_key(one_key) && #[trigger] first_copies.contains_key(
            other_key,
        ) && one_key != other_key ==> first_copies[one_key] != first_copies[other_key]
}

// ===== 择根与取前缀合判（C650 的 Verus 一半；用户 2026-10-02 定 Y6 两边都做）=====
// 落到的那一版不早于根环里任一自证过的根：择根取的是择新序最大的有效根，取前缀落点又不早于所选根（M3 的 J1）。
// 改坏择根序（平局取低、实例代号为主、不看实例、取第一个有效者）时，所选根之外会有一条自证过的根比落点新，这一条就不成立。
pub open spec fn replay_lands_no_earlier_than_any_self_verified_root<RootSource>(
    root_slots: Seq<Option<RootChoiceKey>>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    records: Seq<CoreRecord<RootSource>>,
    replayed: Result<CoreOutcome<RootSource>, CoreFailure>,
) -> bool {
    replayed matches Ok(outcome) ==> forall|slot_index: int|
        0 <= slot_index < root_slots.len() && (#[trigger] root_slots[slot_index]) is Some
            ==> version_is_not_later_than(
            root_choice_key_of(root_slots[slot_index]).checkpoint_txg,
            root_choice_key_of(root_slots[slot_index]).instance,
            landed_checkpoint_txg(root_checkpoint_txg, records, outcome.applied@),
            landed_instance(root_instance, records, outcome.applied@),
        )
}

// 合成的那一条：所选根是有效候选里择新序最大的那条，从它取出的前缀满足 replay_core 的 M2（replay_follows_the_prefix_rules）
// 与 M3 四条 ensures，落点不早于根环里任一自证过的根。
pub open spec fn recovery_replays_from_the_newest_self_verified_root<
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
) -> bool {
    let chosen_key = root_choice_key_of(root_slots[chosen_slot as int]);
    &&& root_choice_follows_the_newest_rule(root_slots, Some(chosen_slot))
    &&& replay_follows_the_prefix_rules(
        chosen_key.instance,
        chosen_key.checkpoint_txg,
        chosen_root_source,
        records,
        in_flight_limit,
        named_unit_verification,
        back_chain_broken,
        named_units_verified,
        replayed,
    )
    &&& replaying_an_applied_record_twice_leaves_the_root_as_once(
        chosen_key.instance,
        chosen_key.checkpoint_txg,
        chosen_root_source,
        records,
        in_flight_limit,
        named_unit_verification,
        back_chain_broken,
        named_units_verified,
        replayed,
    )
    &&& replaying_from_the_landed_root_is_a_fixed_point(
        chosen_key.instance,
        chosen_key.checkpoint_txg,
        chosen_root_source,
        records,
        in_flight_limit,
        named_unit_verification,
        back_chain_broken,
        named_units_verified,
        replayed,
    )
    &&& replay_lands_no_earlier_than_the_chosen_root(
        chosen_key.instance,
        chosen_key.checkpoint_txg,
        chosen_root_source,
        records,
        in_flight_limit,
        named_unit_verification,
        back_chain_broken,
        named_units_verified,
        replayed,
    )
    &&& replay_outcome_is_the_only_one_the_rules_allow(
        chosen_key.instance,
        chosen_key.checkpoint_txg,
        chosen_root_source,
        records,
        in_flight_limit,
        named_unit_verification,
        back_chain_broken,
        named_units_verified,
        replayed,
    )
    &&& replay_lands_no_earlier_than_any_self_verified_root(
        root_slots,
        chosen_key.instance,
        chosen_key.checkpoint_txg,
        records,
        replayed,
    )
}

// ===== 证明用的记法与引理（不可信件：写错了只会证不出来）=====
pub open spec fn as_ints(indices: Seq<usize>) -> Seq<int> {
    Seq::new(indices.len(), |position: int| indices[position] as int)
}

// 末条还没到的那一段：同 txg、序号 1..n、都不带末条标志。
pub open spec fn is_open_publish<RootSource>(publish: Seq<CoreRecord<RootSource>>) -> bool {
    &&& forall|position: int|
        0 <= position < publish.len() ==> (#[trigger] publish[position]).checkpoint_txg
            == publish[0].checkpoint_txg
    &&& forall|position: int|
        0 <= position < publish.len() ==> (#[trigger] publish[position]).ordinal_within_publish.0 as int
            == position + 1
    &&& forall|position: int|
        0 <= position < publish.len() ==> !is_last_record_of_its_publish(
            #[trigger] publish[position],
        )
}

// 「下一个该接的 jsn」：这一串还空着时由锚点给，否则是最后一条的下一号。
pub open spec fn expected_next_after<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    admitted: Seq<int>,
    anchor: Option<JournalCounter>,
    root_instance: PrefixKernelInstanceGeneration,
) -> Option<(PrefixKernelInstanceGeneration, JournalCounter)> {
    if admitted.len() == 0 {
        match anchor {
            Some(anchor_counter) => Some(
                (root_instance, JournalCounter((anchor_counter.0 + 1) as u64)),
            ),
            None => None,
        }
    } else {
        let last = records[admitted.last()];
        Some((last.instance, JournalCounter((last.counter.0 + 1) as u64)))
    }
}

// 不给「候选按切片次序递增」：那一条的触发是两两配对，主循环每轮调这个引理，它在主循环里实例化上万次、把 rlimit 用尽
// （H5-2 退回之后主循环要 rlimit 13 上下才过，去掉这一条后 6 就过）。已施加的、攒着的递增由主循环「都小于 index」那几条不变式自己保。
proof fn lemma_candidates_lie_below_their_end_and_above_the_chosen_root<RootSource>(
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
            &&& candidates.len() <= end
            &&& forall|position: int|
                0 <= position < candidates.len() ==> 0 <= #[trigger] candidates[position] < end
                    && above_the_chosen_root(
                    records[candidates[position]],
                    root_instance,
                    root_checkpoint_txg,
                )
        }),
    decreases end,
{
    if end > 0 {
        lemma_candidates_lie_below_their_end_and_above_the_chosen_root(
            records,
            root_instance,
            root_checkpoint_txg,
            end - 1,
        );
    }
}

proof fn lemma_candidates_before_a_shorter_end_are_a_prefix<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    shorter: int,
    longer: int,
)
    requires
        0 <= shorter <= longer <= records.len(),
    ensures
        ({
            let fewer = candidates_before(records, root_instance, root_checkpoint_txg, shorter);
            let more = candidates_before(records, root_instance, root_checkpoint_txg, longer);
            &&& fewer.len() <= more.len()
            &&& forall|position: int|
                0 <= position < fewer.len() ==> #[trigger] fewer[position] == more[position]
        }),
    decreases longer - shorter,
{
    if shorter < longer {
        lemma_candidates_before_a_shorter_end_are_a_prefix(
            records,
            root_instance,
            root_checkpoint_txg,
            shorter,
            longer - 1,
        );
    }
}

// 同一实例里切片次序就是计数器次序（入参按 (实例代号, 计数器) 严格升序）。
proof fn lemma_slice_order_within_an_instance_follows_the_counter<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    first: int,
    second: int,
)
    requires
        records_ascend_by_instance_and_counter(records),
        0 <= first < records.len(),
        0 <= second < records.len(),
        records[first].instance == records[second].instance,
    ensures
        first < second <==> records[first].counter.0 < records[second].counter.0,
{
    if first < second {
        assert(key_is_below(records[first], records[second]));
    } else if second < first {
        assert(key_is_below(records[second], records[first]));
    }
}

// 接受了下一个候选之后，已接受的每一条照旧过了逐条判定，新的一条也过了。
proof fn lemma_admitting_the_candidate_keeps_every_link<
    RootSource,
    BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize) -> bool,
>(
    records: Seq<CoreRecord<RootSource>>,
    admitted_before: Seq<int>,
    next: int,
    anchor: Option<JournalCounter>,
    root_instance: PrefixKernelInstanceGeneration,
    root_checkpoint_txg: PrefixKernelCheckpointTxg,
    named_unit_verification: NamedUnitVerification,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
)
    requires
        forall|position_in_chain: int|
            0 <= position_in_chain < admitted_before.len() ==> #[trigger] admitted_record_passed_every_check(
                records,
                admitted_before,
                position_in_chain,
                anchor,
                root_instance,
                root_checkpoint_txg,
                named_unit_verification,
                back_chain_broken,
                named_units_verified,
            ),
        passes_every_check_before_the_named_units(
            records,
            next,
            previous_in_the_chain(records, admitted_before, admitted_before.len() as int),
            anchor,
            root_instance,
            root_checkpoint_txg,
            back_chain_broken,
        ),
        named_units_reported_verified(named_unit_verification, named_units_verified, next),
    ensures
        forall|position_in_chain: int|
            0 <= position_in_chain < admitted_before.len() + 1 ==> #[trigger] admitted_record_passed_every_check(
                records,
                admitted_before.push(next),
                position_in_chain,
                anchor,
                root_instance,
                root_checkpoint_txg,
                named_unit_verification,
                back_chain_broken,
                named_units_verified,
            ),
{
    reveal(admitted_record_passed_every_check);
    let admitted = admitted_before.push(next);
    assert forall|position_in_chain: int|
        0 <= position_in_chain < admitted.len() implies #[trigger] admitted_record_passed_every_check(
        records,
        admitted,
        position_in_chain,
        anchor,
        root_instance,
        root_checkpoint_txg,
        named_unit_verification,
        back_chain_broken,
        named_units_verified,
    ) by {
        if position_in_chain < admitted_before.len() {
            assert(admitted[position_in_chain] == admitted_before[position_in_chain]);
            assert(previous_in_the_chain(records, admitted, position_in_chain)
                == previous_in_the_chain(records, admitted_before, position_in_chain));
            assert(admitted_record_passed_every_check(
                records,
                admitted_before,
                position_in_chain,
                anchor,
                root_instance,
                root_checkpoint_txg,
                named_unit_verification,
                back_chain_broken,
                named_units_verified,
            ));
        } else {
            assert(previous_in_the_chain(records, admitted, position_in_chain)
                == previous_in_the_chain(records, admitted_before, admitted_before.len() as int));
        }
    }
}

// 接受了下一个候选之后，攒着的这次发布仍是「同 txg、序号 1..n」，除新的一条外都不带末条标志，攒着的仍是这一串在已施加之后的尾巴。
proof fn lemma_admitting_the_candidate_extends_the_open_publish<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    open_before: Seq<usize>,
    index: usize,
    admitted_before: Seq<int>,
    applied: Seq<int>,
)
    requires
        0 <= index < records.len(),
        forall|position: int|
            0 <= position < open_before.len() ==> (#[trigger] open_before[position]) < records.len(),
        is_open_publish(records_at(records, as_ints(open_before))),
        as_ints(open_before) == admitted_before.subrange(
            applied.len() as int,
            admitted_before.len() as int,
        ),
        applied_is_the_chain_cut_at_its_last_publish_end(records, admitted_before, applied),
        obeys_the_reader_rules_and_the_commit_markers(
            records[index as int],
            previous_in_the_chain(records, admitted_before, admitted_before.len() as int),
        ),
    ensures
        as_ints(open_before.push(index)) == admitted_before.push(index as int).subrange(
            applied.len() as int,
            admitted_before.len() as int + 1,
        ),
        ({
            let after = records_at(records, as_ints(open_before.push(index)));
            &&& forall|position: int|
                0 <= position < after.len() ==> (#[trigger] after[position]).checkpoint_txg
                    == after[0].checkpoint_txg && after[position].ordinal_within_publish.0 as int
                    == position + 1
            &&& forall|position: int|
                0 <= position < open_before.len() ==> !is_last_record_of_its_publish(
                    #[trigger] after[position],
                )
            &&& after[after.len() - 1] == records[index as int]
            &&& is_last_record_of_its_publish(records[index as int]) ==> is_whole_publish(after)
            &&& !is_last_record_of_its_publish(records[index as int]) ==> is_open_publish(after)
        }),
{
    let open = open_before.push(index);
    let admitted = admitted_before.push(index as int);
    assert forall|position: int|
        0 <= position < as_ints(open).len() implies #[trigger] as_ints(open)[position]
        == admitted.subrange(applied.len() as int, admitted.len() as int)[position] by {
        if position < open_before.len() {
            assert(as_ints(open_before)[position] == admitted_before.subrange(
                applied.len() as int,
                admitted_before.len() as int,
            )[position]);
        }
    }
    assert(as_ints(open) =~= admitted.subrange(applied.len() as int, admitted.len() as int));
    if open_before.len() > 0 {
        assert(as_ints(open_before).last() == admitted_before.last());
    } else if admitted_before.len() > 0 {
        assert(applied.len() == admitted_before.len());
        assert(applied.last() == admitted_before.last());
    }
    let before = records_at(records, as_ints(open_before));
    let after = records_at(records, as_ints(open));
    assert forall|position: int| 0 <= position < after.len() implies (
    #[trigger] after[position]).checkpoint_txg == after[0].checkpoint_txg
        && after[position].ordinal_within_publish.0 as int == position + 1 by {
        if position < open_before.len() {
            assert(after[position] == before[position]);
            assert(after[0] == before[0]);
        } else {
            assert(after[position] == records[index as int]);
            if open_before.len() > 0 {
                assert(before[open_before.len() - 1] == records[open_before[open_before.len()
                    - 1] as int]);
                assert(after[0] == before[0]);
            }
        }
    }
    assert forall|position: int|
        0 <= position < open_before.len() implies !is_last_record_of_its_publish(
        #[trigger] after[position],
    ) by {
        assert(after[position] == before[position]);
    }
    assert(after[after.len() - 1] == records[index as int]);
}

// 已施加的是这一串的开头、攒着的是其余的尾巴 ⇒ 两段接起来就是整串。
proof fn lemma_the_applied_head_and_the_open_tail_make_up_the_chain(
    admitted: Seq<int>,
    applied: Seq<int>,
    open_tail: Seq<int>,
)
    requires
        applied.len() <= admitted.len(),
        forall|position: int|
            0 <= position < applied.len() ==> #[trigger] applied[position] == admitted[position],
        open_tail == admitted.subrange(applied.len() as int, admitted.len() as int),
    ensures
        applied + open_tail == admitted,
{
    assert(applied + open_tail =~= admitted);
}

proof fn lemma_appending_a_whole_publish_keeps_a_concatenation<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    applied_before: Seq<int>,
    this_publish: Seq<int>,
)
    requires
        is_concatenation_of_whole_publishes(records_at(records, applied_before)),
        is_whole_publish(records_at(records, this_publish)),
    ensures
        is_concatenation_of_whole_publishes(records_at(records, applied_before + this_publish)),
{
    let before = records_at(records, applied_before);
    let publish = records_at(records, this_publish);
    let after = records_at(records, applied_before + this_publish);
    assert(after =~= before + publish);
    assert(after.subrange(applied_before.len() as int, after.len() as int) =~= publish);
    assert(after.subrange(0, applied_before.len() as int) =~= before);
    assert(is_whole_publish(after.subrange(applied_before.len() as int, after.len() as int)));
}

proof fn lemma_the_maximum_over_the_appended_publish<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
    applied_before: Seq<int>,
    records_of_the_open_publish: Seq<usize>,
    maximum_before: TransactionNumber,
    maximum: TransactionNumber,
)
    requires
        maximum_transaction_is_the_largest_applied(records, applied_before, maximum_before),
        maximum.0 >= maximum_before.0,
        forall|position: int|
            0 <= position < records_of_the_open_publish.len()
                ==> records[#[trigger] records_of_the_open_publish[position] as int].transaction.0
                <= maximum.0,
        maximum == maximum_before || exists|position: int|
            0 <= position < records_of_the_open_publish.len()
                && records[#[trigger] records_of_the_open_publish[position] as int].transaction
                == maximum,
    ensures
        maximum_transaction_is_the_largest_applied(
            records,
            applied_before + as_ints(records_of_the_open_publish),
            maximum,
        ),
{
    let this_publish = as_ints(records_of_the_open_publish);
    let applied = applied_before + this_publish;
    assert forall|position: int|
        0 <= position < applied.len() implies (#[trigger] records[applied[position]]).transaction.0
        <= maximum.0 by {
        if position >= applied_before.len() {
            assert(applied[position] == this_publish[position - applied_before.len()]);
            assert(records[records_of_the_open_publish[position
                - applied_before.len()] as int].transaction.0 <= maximum.0);
        }
    }
    if maximum.0 != 0 {
        if maximum == maximum_before {
            let witness = choose|position: int|
                0 <= position < applied_before.len() && (#[trigger] records[applied_before[position]]).transaction
                    == maximum_before;
            assert(applied[witness] == applied_before[witness]);
        } else {
            let open_witness = choose|open_position: int|
                0 <= open_position < records_of_the_open_publish.len()
                    && records[#[trigger] records_of_the_open_publish[open_position] as int].transaction
                    == maximum;
            assert(applied[applied_before.len() + open_witness]
                == records_of_the_open_publish[open_witness] as int);
        }
    }
}

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
        !a_later_record_of_the_same_publish_is_readable(records, end_index),
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
        ) implies !a_later_record_of_the_same_publish_is_readable(
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

// 不动点那一读（第二轮判决第二节第 6 行；第三轮 S6 取 Qn）：落到的那一版当所选根再取一次前缀，结局是 replaying_again_from_the_landed_root_stays_put。
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

// 记录级那一读的第一半：同一条记录紧接着施加两次，与施加一次的根相同（I-8.4 的字面）。
proof fn lemma_applying_a_record_twice_in_a_row_is_applying_it_once<RootSource>(
    root_source: RootSource,
    applied_records: Seq<CoreRecord<RootSource>>,
    position: int,
)
    requires
        0 <= position < applied_records.len(),
    ensures
        root_source_after_applying_in_order(
            root_source,
            applied_records.insert(position + 1, applied_records[position]),
        ) == root_source_after_applying_in_order(root_source, applied_records),
    decreases applied_records.len(),
{
    let twice = applied_records.insert(position + 1, applied_records[position]);
    if position == applied_records.len() - 1 {
        assert(twice.drop_last() =~= applied_records);
        assert(twice.last() == applied_records.last());
    } else {
        lemma_applying_a_record_twice_in_a_row_is_applying_it_once(
            root_source,
            applied_records.drop_last(),
            position,
        );
        assert(twice.drop_last() =~= applied_records.drop_last().insert(
            position + 1,
            applied_records[position],
        ));
        assert(twice.last() == applied_records.last());
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

// M3 单调（推论；第二轮判决第二节第 5 行，第三轮 S5 版本序取 (txg, 实例代号)）：同一次运行里两个崩溃状态 s ⊆ u，
// u 上恢复落到的那一版不早于 s 上的。核里证的只有两步：落点不早于所选根、恢复是确定的函数（上面两条 ensures 的引理）；
// 两条前提是择根、写侧的条款，核判不了，照攻方 research/prompts/m3-formal-r2-opus-output.md 3.2 推的写成 requires：
// 所选根取根环里最新的有效根 ⇒ u 的所选根不早于 s 的（3.2 第 1 步）；s 施加了一次发布时，串行提交与 D16（发布语义）已定项 7 的持久顺序
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
            // requires 里另列的 J1（replay_lands_no_earlier_than_the_chosen_root）是多余前提，不是这一步非用不可的
            // （代码轮第一轮判决「改法落地」一节，research/prompts/m3-formal-code-r1-main-verification.md）。
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

// ===== 纯函数核 =====
// 记录标志位 0（已定项 14 第五条、已定项 17）。
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
        follows == a_later_record_of_the_same_publish_is_readable(records@, index as int),
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

// 所选根覆盖的最后一条记录的 jsn 计数器（已定项 14 注 1，读法乙）：所选根那次发布读得出的几条里带末条标志的那一条；
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

// 前缀规则不跨实例边界（已定项 14 第 1 条），只有所选根自己那个实例、txg 比根大的记录是候选。
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
        // 锚点读不出：链首只能是 txg = 根的 txg + 1、本次发布内序号为 1 的那条（三方第一轮 K4-b）。
        // 序号与 txg 一起在反向链回调之前判，次序同原文：链首序号不是 1 的不调反向链回调。下面「攒着的为空且序号不是 1」那一判
        // 管锚点读得出时接上的那一次与之后新开的每一次发布；它对这一支的链首也成立（走到这一支时攒着的一定是空的，下面这句由 Verus 证），
        // 这里先判掉，读盘次序才与原文相同（代码轮第三轮判决 J1、用户定案 U1）。
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
    // 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断；三态由回调交成布尔。
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
        // M3 幂等·记录级（第二轮判决第二节第 6 行「记录级」，I-8.4 的字面）
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
        // M3 幂等·不动点（第二轮判决第二节第 6 行「不动点」；第三轮 S6 取 Qn）
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
        // M3 单调推论的 J1 那一步（第二轮判决第二节第 5 行；第三轮 S5 版本序 (txg, 实例代号)）
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
        // M3 单调推论第 4 步「恢复是确定的函数」（第二轮判决第二节第 5 行）
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

// ===== spec 自己的冒烟（防 spec 恒真或恒假）=====
proof fn a_publish_of_two_records_is_whole(first: CoreRecord<u64>, second: CoreRecord<u64>)
    requires
        first.checkpoint_txg == second.checkpoint_txg,
        first.ordinal_within_publish.0 == 1,
        second.ordinal_within_publish.0 == 2,
        !is_last_record_of_its_publish(first),
        is_last_record_of_its_publish(second),
    ensures
        is_concatenation_of_whole_publishes(seq![first, second]),
{
    let both = seq![first, second];
    assert(both.subrange(0, 2) =~= both);
    assert(is_whole_publish(both.subrange(0, 2)));
    assert(both.subrange(0, 0).len() == 0);
    assert(is_concatenation_of_whole_publishes(both.subrange(0, 0)));
}

proof fn a_record_without_the_last_flag_alone_is_not_a_concatenation(only: CoreRecord<u64>)
    requires
        !is_last_record_of_its_publish(only),
    ensures
        !is_concatenation_of_whole_publishes(seq![only]),
{
    let one = seq![only];
    assert forall|cut: int| 0 <= cut < one.len() implies !is_whole_publish(
        #[trigger] one.subrange(cut, one.len() as int),
    ) by {
        assert(one.subrange(0, 1) =~= one);
        assert(one.subrange(0, 1)[0] == only);
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
    fn a_two_record_publish_after_the_anchor_is_applied_whole_and_the_new_root_segment_comes_from_its_last_record(
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
    fn a_publish_whose_last_record_is_missing_is_not_applied_and_the_root_segment_stays() {
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
    fn a_later_checkpoint_txg_is_chosen_over_a_higher_instance_with_an_earlier_checkpoint_txg() {
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
    fn a_newer_root_after_the_first_self_verified_slot_is_chosen_over_it() {
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

//! 取 journal 前缀并施加的纯函数核：`singlefs-core` 的 `recovery::replay_journal` 取前缀那一截全部判定都在这里，
//! Verus 验的就是编进产品、被恢复路径调用的这一份。
//! 单独成一个 crate、只依赖 `vstd`：Verus 验证这个 crate 及其模块，不编整个 `singlefs-core`。
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
//! `crates/singlefs-verification/tests/mounted_recovery_reads_each_candidate_once_in_order.rs` 钉住：锚点读得出、两次单条发布都施加；
//! 锚点那条记录撕了；锚点读不出、链首序号不是 1；一次发布两条记录；候选在末条的提交标记上被断。
//! 别的形状（反向链断了、到了在飞上限、可写挂载走读阶段缓存、`recover` 不验点名单元那一臂）读几次盘没有用例钉。
//! 平时的 stable 编译用 crates.io 的 `vstd` 与 `verus_builtin_macros` 把 `verus!` 擦成普通 Rust；
//! 验的时候 Verus 用它自带的那一份，两边钉同一个发布（`Cargo.toml` 的 vstd 版本与 Verus 0.2026.09.20.aef82ed）。
//! 另两个纯函数核也在这里：择根（`choose_root_slot`，`recovery::choose_root` 取 `(checkpoint_txg, 实例代号)` 最大的那一截）与
//! 扫环建表（`first_copy_of_each_record_key`，`recovery::scan_journal` 同一 (实例代号, 计数器) 先到先得的那一截）；
//! 择根与取前缀的合判（C650 的 Verus 一半）写成推论 `corollary_recovery_replays_from_the_newest_self_verified_root`。
//! 整理的选段核与计划核及其四条性质（里程碑四二·4）在模块 [`compaction_plan_proofs`]：`singlefs-core` 直接调用它们，
//! `compaction_plan` 单测以独立参考实现核对转换；GC 协议与旧读者证明在 [`gc_crash_proofs`]。
//! Verus 的必红世界登记在同目录的 `verus-must-red-worlds.tsv`。
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use vstd::prelude::*;

pub mod compaction_plan_proofs;
pub mod file_range_proofs;
pub mod gc_crash_proofs;
pub mod gc_ring_interval_proofs;
pub mod gpu_gc_capacity_proofs;
pub mod gpu_gc_proofs;
pub mod gpu_gc_scoring_proofs;
pub mod partial_write_proofs;
pub mod unlink_proofs;

// M3 的引理与纯函数核在子模块里，纯函数核挂在 M3 引理底下（缘故在 `idempotence_and_monotonicity_lemmas` 的模块文档里）。
mod idempotence_and_monotonicity_lemmas;

pub use idempotence_and_monotonicity_lemmas::pure_function_kernels::*;

verus! {

// ===== 核的数据 =====
// 核不依赖 singlefs-core 与 singlefs-format（Verus 只编这一个文件）。带 PrefixKernel 的四个类型与 singlefs-core 里不带它的那四个
// （address.rs 的 InstanceGeneration、CheckpointTxg，journal.rs 的 JournalRecordOrdinalWithinPublish、JournalRecordPlaceInPublish）
// 是同一个概念的两个类型，不是一处定义：胶水（recovery.rs 的 replay_journal）逐个手抄映射过来，两边加字段、加成员没有东西逼着同步。
// 没合成一处，是因为 core 在 journal.rs 里给后两个类型写了固有方法，外 crate 的类型上写不了；把那几个方法挪进本 crate 的 verus! 之外、
// core 改用这里的类型，这条路不取：维持改名、这里写实话。
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

// 已定项 14 的原句：只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录；注 1：实例代号与所选根不同即停。
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
pub open spec fn later_record_of_the_same_publish_is_readable<RootSource>(
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
        && later_record_of_the_same_publish_is_readable(records, position))
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
        && later_record_of_the_same_publish_is_readable(records, position))
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

// ===== M3 幂等与单调（spec，可信：照条款文字写）=====
// D13（验证路线）已定项 1 的 M3 一行。
// 主语是 replay_core：singlefs-core 里以 `NamedUnitVerification::Verify` 调 replay_journal 的三处走到的就是它。
// 下面四条都写成 replay_core 的 ensures（证在 M3 的引理那一节），说的是编进产品的这一份交回的结果；单调本身是推论 corollary_the_landed_version_does_not_go_back。

// I-8.12（同实例记录的 txg 不减），K10：同一实例里 journal 记录的 txg 随计数器不减。
// 核不判它：记录级的第二半与不动点把它写成前提（核拿到的记录表满足它）。
pub open spec fn journal_txg_does_not_decrease_with_the_counter_within_an_instance<RootSource>(
    records: Seq<CoreRecord<RootSource>>,
) -> bool {
    forall|earlier: int, later: int|
        0 <= earlier < records.len() && 0 <= later < records.len()
            && #[trigger] is_a_later_record_of_the_same_instance(records, earlier, later)
            ==> records[earlier].checkpoint_txg.0 <= records[later].checkpoint_txg.0
}

// 「同一个读者」（不动点那一读的前提）：读盘回调在同一个下标上只交回一种答案。
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

// M3 幂等·记录级（「记录级」，I-8.4（重放幂等）的字面「任一 journal 记录被重放两次与重放一次，产生的盘上状态相同」）：
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

// M3 幂等·不动点（「不动点」）：把落到的那一版当所选根，同一张记录表、同一个读者、同一个开关、
// 同一个在飞上限再取一次前缀（第二次取前缀的结果是规则容得下的任一个），根不动。
// 前提：I-8.12；同一个读者；所选根之上的记录条数不超在飞上限。
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

// M3 单调（推论）里核自己的两半，推论本身是下面的 corollary_the_landed_version_does_not_go_back。
// 一半（J1）：落到的那一版不早于所选根，按 S5 的 (txg, 实例代号) 序。
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

// 另一半（「恢复是确定的函数」）：同样的入参、同一个读者，规则只容得下这一个结果。
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
// 择新序就是上面 M3 单调用的 version_is_not_later_than（它与择根同序），这里不另写一个序。
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
// （replay_core 入参「严格升序」与 M2 语义层
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

// ===== 择根与取前缀合判（C650 的 Verus 一半）=====
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

// ===== spec 自己的冒烟（防 spec 恒真或恒假）=====
proof fn publish_of_two_records_is_whole(first: CoreRecord<u64>, second: CoreRecord<u64>)
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

proof fn record_without_the_last_flag_alone_is_not_a_concatenation(only: CoreRecord<u64>)
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

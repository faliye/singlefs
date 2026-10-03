//! 取 journal 前缀并施加的纯函数核：`singlefs-core` 的 `recovery::replay_journal` 取前缀那一截全部判定都在这里，
//! Verus 验的就是编进产品、被恢复路径调用的这一份（用户 2026-10-01 定「Verus 跑真实代码」；里程碑三七·3）。
//! 单独成一个 crate、只依赖 `vstd`：Verus 只编这一个文件，不编整个 `singlefs-core`（用户 2026-10-01 定拆）。
//!
//! 来历：判定照提交 fcc4d340 的 `crates/singlefs-core/src/recovery.rs` 里 `replay_journal`（第 3057–3229 行）搬过来，
//! 每个分支旁的 [原文 N] 是那一版 `recovery.rs` 的行号；[改写] 是写法换过、判定没换的。
//! spec 照 D23（journal 的角色与格式）已定项 14 与 D13（验证路线）已定项 1 的 M2 那一行写。
//! 读盘才判得出的两件（反向链、点名单元）由调用方给两个回调，核走到那一步才调，读盘次数与次序同原文。
//! 平时的 stable 编译用 crates.io 的 `vstd` 与 `verus_builtin_macros` 把 `verus!` 擦成普通 Rust；
//! 验的时候 Verus 用它自带的那一份，两边钉同一个发布（`Cargo.toml` 的 vstd 版本与 Verus 0.2026.09.20.aef82ed）。
//! Verus 的必红世界登记在同目录的 `verus-mutations.tsv`。
#![forbid(unsafe_code)]

use vstd::prelude::*;

verus! {

// ===== 核的数据 =====
// JournalRecord（singlefs-core 的 journal.rs）里取前缀用得到的字段；枚举与 newtype 摊成原始类型。
pub struct CoreRecord<Segment> {
    pub instance: u32,  // InstanceGeneration.0
    pub counter: u64,
    pub checkpoint_txg: u64,  // CheckpointTxg.0
    pub transaction: u64,
    pub is_commit: bool,
    pub ordinal_within_publish: u32,  // JournalRecordOrdinalWithinPublish.0
    pub ends_its_publish: bool,  // place_in_publish == LastRecordOfThePublish
    pub new_root_segment: Segment,  // 施加时换进根里的那几个字段（原文 3216–3222）
}

pub struct CoreReport {
    pub above_water: usize,
    pub prefix_applied: usize,
    pub verification_passed: usize,
    pub verification_failed: usize,
    pub maximum_applied_transaction: u64,
}

pub enum CoreFailure {
    RootPublishCarriesMoreThanOneLastRecordFlag,
    ChosenRootCheckpointTxgHasNoSuccessor,
}

pub struct CoreOutcome<Segment> {
    pub report: CoreReport,
    pub rebuilt: Segment,
    pub applied: Ghost<Seq<int>>,  // 施加了的记录在入参切片里的下标，幽灵量，编译时擦掉
}

// ===== spec（可信：照条款文字写，没有别的东西核它）=====
// 已定项 14 定案句：只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录；注 1：实例代号与所选根不同即停。
pub open spec fn above_the_chosen_root<Segment>(
    record: CoreRecord<Segment>,
    root_instance: u32,
    root_checkpoint_txg: u64,
) -> bool {
    record.instance == root_instance && record.checkpoint_txg > root_checkpoint_txg
}

// 一次整的发布：checkpoint_txg 相同、序号依次 1..k、只有第 k 条带末条标志（已定项 4 / 14 第五条 / 17）。
pub open spec fn is_whole_publish<Segment>(publish: Seq<CoreRecord<Segment>>) -> bool {
    &&& publish.len() >= 1
    &&& forall|position: int|
        0 <= position < publish.len() ==> (#[trigger] publish[position]).checkpoint_txg
            == publish[0].checkpoint_txg
    &&& forall|position: int|
        0 <= position < publish.len() ==> (#[trigger] publish[position]).ordinal_within_publish as int
            == position + 1
    &&& forall|position: int|
        0 <= position < publish.len() ==> ((#[trigger] publish[position]).ends_its_publish
            <==> position == publish.len() - 1)
}

// 若干整次发布的拼接（M2：重放结果等于整数个发布，不存在被前缀切开的发布）。
pub open spec fn is_concatenation_of_whole_publishes<Segment>(
    applied: Seq<CoreRecord<Segment>>,
) -> bool
    decreases applied.len(),
{
    applied.len() == 0 || exists|cut: int|
        0 <= cut < applied.len() && is_whole_publish(
            #[trigger] applied.subrange(cut, applied.len() as int),
        ) && is_concatenation_of_whole_publishes(applied.subrange(0, cut))
}

pub open spec fn records_at<Segment>(
    records: Seq<CoreRecord<Segment>>,
    indices: Seq<int>,
) -> Seq<CoreRecord<Segment>> {
    Seq::new(indices.len(), |position: int| records[indices[position]])
}

// 所选根那次发布读得出的记录里带末条标志的那一条（spec 层照 counter_of_the_last_record_the_root_covers 的字面）
pub open spec fn root_publish_last_flagged<Segment>(
    records: Seq<CoreRecord<Segment>>,
    root_instance: u32,
    root_checkpoint_txg: u64,
    position: int,
) -> bool {
    &&& 0 <= position < records.len()
    &&& records[position].instance == root_instance
    &&& records[position].checkpoint_txg == root_checkpoint_txg
    &&& records[position].ends_its_publish
}

// 锚点 = 那一条的计数器；一条都没有 = None；多于一条 = 没有锚点（核报错，走不到 Ok）
pub open spec fn anchor_is<Segment>(
    root_instance: u32,
    root_checkpoint_txg: u64,
    records: Seq<CoreRecord<Segment>>,
    anchor: Option<u64>,
) -> bool {
    match anchor {
        Some(counter) => exists|position: int|
            (#[trigger] root_publish_last_flagged(records, root_instance, root_checkpoint_txg, position))
                && records[position].counter == counter && (forall|other: int|
                (#[trigger] root_publish_last_flagged(records, root_instance, root_checkpoint_txg, other))
                    ==> other == position),
        None => forall|position: int|
            !(#[trigger] root_publish_last_flagged(records, root_instance, root_checkpoint_txg, position)),
    }
}

// 施加的计数器从链首起逐一连续：锚点读得出时首条 = 锚点 + 1；读不出时首条 txg = 根 + 1、序号 1；之后每条 = 前一条 + 1
pub open spec fn chain_from_the_head<Segment>(
    root_checkpoint_txg: u64,
    records: Seq<CoreRecord<Segment>>,
    anchor: Option<u64>,
    chain: Seq<int>,
) -> bool {
    &&& chain.len() > 0 ==> (match anchor {
        Some(counter) => records[chain[0]].counter == counter + 1,
        None => records[chain[0]].checkpoint_txg == root_checkpoint_txg + 1
            && records[chain[0]].ordinal_within_publish == 1,
    })
    &&& forall|position: int|
        0 <= position && position + 1 < chain.len() ==> records[(#[trigger] chain[position + 1])].counter
            == records[chain[position]].counter + 1
}

pub open spec fn m2_holds<Segment>(
    root_instance: u32,
    root_checkpoint_txg: u64,
    root_segment: Segment,
    records: Seq<CoreRecord<Segment>>,
    outcome: CoreOutcome<Segment>,
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
    &&& outcome.rebuilt == if applied.len() == 0 {
        root_segment
    } else {
        records[applied.last()].new_root_segment
    }
    &&& exists|anchor: Option<u64>|
        (#[trigger] anchor_is(root_instance, root_checkpoint_txg, records, anchor))
            && chain_from_the_head(root_checkpoint_txg, records, anchor, applied)
}

// ===== 证明用的记法（不可信件：写错了只会证不出来）=====
pub open spec fn as_ints(indices: Seq<usize>) -> Seq<int> {
    Seq::new(indices.len(), |position: int| indices[position] as int)
}

// 末条还没到的那一段：同 txg、序号 1..n、都不带末条标志。
pub open spec fn is_open_publish<Segment>(publish: Seq<CoreRecord<Segment>>) -> bool {
    &&& forall|position: int|
        0 <= position < publish.len() ==> (#[trigger] publish[position]).checkpoint_txg
            == publish[0].checkpoint_txg
    &&& forall|position: int|
        0 <= position < publish.len() ==> (#[trigger] publish[position]).ordinal_within_publish as int
            == position + 1
    &&& forall|position: int|
        0 <= position < publish.len() ==> !(#[trigger] publish[position]).ends_its_publish
}

// ===== 纯函数核 =====
// [原文 2930–2937] record_ends_its_publish：记录标志位 0（已定项 14 第五条、已定项 17）。[改写] 原文 match 两臂枚举，这里读布尔字段。
fn record_ends_its_publish<Segment>(record: &CoreRecord<Segment>) -> (is_last_record: bool)
    ensures
        is_last_record == record.ends_its_publish,
{
    record.ends_its_publish
}

// [原文 2939–2953] readable_record_of_the_same_publish_follows：同一 (实例代号, checkpoint_txg) 里这一条之后（计数器更大）
// 还有没有读得出的记录。同一实例里计数器与 checkpoint_txg 一起往上走，所以按计数器升序看到 txg 越过这一条的就停。
// [改写] 原文在 BTreeMap 上 range((instance, counter+1)..=(instance, u64::MAX))，这里从切片里这一条的下一个位置往后扫，
// 靠「切片按 (实例代号, 计数器) 严格升序」才与原文同义；这一条没有写成前置条件，也没有证。
fn readable_record_of_the_same_publish_follows<Segment>(
    records: &[CoreRecord<Segment>],
    index: usize,
) -> bool
    requires
        index < records.len(),
{
    let record = &records[index];
    // [原文 2945–2947] checked_add(1) 取不到就 false
    if record.counter == u64::MAX {
        return false;
    }
    let next_counter = record.counter + 1;
    let mut later_index = index + 1;
    while later_index < records.len()
        invariant
            index < later_index <= records.len(),
        decreases records.len() - later_index,
    {
        let later = &records[later_index];
        // [原文 2949] range 的上下界：同实例、计数器 ≥ next_counter
        if later.instance == record.instance && later.counter >= next_counter {
            // [原文 2951] take_while(later.checkpoint_txg <= record.checkpoint_txg)
            if !(later.checkpoint_txg <= record.checkpoint_txg) {
                return false;
            }
            // [原文 2952] any(later.checkpoint_txg == record.checkpoint_txg)
            if later.checkpoint_txg == record.checkpoint_txg {
                return true;
            }
        }
        later_index += 1;
    }
    false
}

// [原文 2955–2987] counter_of_the_last_record_the_root_covers：所选根覆盖的最后一条记录的 jsn 计数器（已定项 14 注 1，读法乙）——
// 所选根那次发布读得出的几条里带末条标志的那一条；一条都不带 ⇒ Ok(None)；带标志的多于一条 ⇒ 认哪一条条款没写，报错。
// [改写] 原文 filter + collect 成 Vec 再看 len() > 1、取 first()；这里一遍循环，第二条带标志的出现就报错。
// 报错里的 counters 清单没带（核不产 Vec），由调用方再调原来那个函数拿。
fn counter_of_the_last_record_the_root_covers<Segment>(
    root_instance: u32,
    root_checkpoint_txg: u64,
    records: &[CoreRecord<Segment>],
) -> (anchor: Result<Option<u64>, CoreFailure>)
    requires
        forall|record_index: int|
            0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter
                < u64::MAX,
    ensures
        anchor matches Ok(Some(counter)) ==> counter < u64::MAX,
        anchor matches Ok(found_anchor) ==> anchor_is(
            root_instance,
            root_checkpoint_txg,
            records@,
            found_anchor,
        ),
{
    let mut found: Option<u64> = None;
    let mut index = 0;
    while index < records.len()
        invariant
            index <= records.len(),
            forall|record_index: int|
                0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter
                    < u64::MAX,
            found matches Some(counter) ==> counter < u64::MAX,
            found is None ==> forall|position: int|
                0 <= position < index ==> !(#[trigger] root_publish_last_flagged(
                    records@,
                    root_instance,
                    root_checkpoint_txg,
                    position,
                )),
            found matches Some(counter) ==> exists|position: int|
                0 <= position < index
                    && (#[trigger] root_publish_last_flagged(
                    records@,
                    root_instance,
                    root_checkpoint_txg,
                    position,
                )) && records@[position].counter == counter && (forall|other: int|
                    0 <= other < index && (#[trigger] root_publish_last_flagged(
                        records@,
                        root_instance,
                        root_checkpoint_txg,
                        other,
                    )) ==> other == position),
        decreases records.len() - index,
    {
        let record = &records[index];
        // [原文 2970–2972]
        if record.instance == root_instance && record.checkpoint_txg == root_checkpoint_txg
            && record_ends_its_publish(record) {
            // [原文 2976–2984]
            if found.is_some() {
                return Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag);
            }
            found = Some(record.counter);
        }
        proof {
            // 本条是不是「那一次发布带末条标志的」：与 spec 谓词同义
            assert(root_publish_last_flagged(records@, root_instance, root_checkpoint_txg, index as int)
                == (record.instance == root_instance && record.checkpoint_txg == root_checkpoint_txg
                && record.ends_its_publish));
        }
        index += 1;
    }
    // [原文 2985–2986]
    Ok(found)
}

// [原文 3074–3080] above 的筛选条件：前缀规则不跨实例边界（已定项 14 第 1 条），只有所选根自己那个实例、txg 比根大的记录是候选。
// 数条数那一遍与主循环共用这一判，同原文只筛一次（crates/mutations.tsv 第 23、355 行改的就是它，两处一起变）。
// [改写] 原文比元组 (record.instance, record.checkpoint_txg) > water，这里把字典序摊开写。
fn is_above_the_chosen_root<Segment>(
    record: &CoreRecord<Segment>,
    root_instance: u32,
    root_checkpoint_txg: u64,
) -> (is_above: bool)
    ensures
        is_above == above_the_chosen_root(*record, root_instance, root_checkpoint_txg),
{
    record.instance == root_instance && (record.instance > root_instance || (record.instance
        == root_instance && record.checkpoint_txg > root_checkpoint_txg))
}

// [原文 3204–3207] 攒着的这次发布里最大的事务号。[改写] 原文 iter().map().fold(.., u64::max)。
fn maximum_transaction_of<Segment>(
    records: &[CoreRecord<Segment>],
    open: &[usize],
    start: u64,
) -> u64
    requires
        forall|position: int| 0 <= position < open@.len() ==> (#[trigger] open@[position]) < records.len(),
{
    let mut maximum = start;
    let mut position = 0;
    while position < open.len()
        invariant
            position <= open@.len(),
            forall|open_position: int|
                0 <= open_position < open@.len() ==> (#[trigger] open@[open_position]) < records.len(),
        decreases open@.len() - position,
    {
        let transaction = records[open[position]].transaction;
        if transaction > maximum {
            maximum = transaction;
        }
        position += 1;
    }
    maximum
}

// [原文 3057–3229] replay_journal 去掉读盘之后的那一截。
// records：全部自证过的记录，次序就是 BTreeMap<(InstanceGeneration, u64), _>::values() 的次序。
// 前置条件只有一条：计数器不是 u64 的最大值（原文 3096、3158 的 counter + 1 在那个值上溢出）。没有要求排序，条数不设界。
#[allow(clippy::too_many_arguments, reason = "两个回调是读盘的两处，摊成参数比包一层结构体好对照原文")]
pub fn replay_core<Segment: Copy, BackChain: Fn(usize) -> bool, NamedUnits: Fn(usize) -> bool>(
    root_instance: u32,
    root_checkpoint_txg: u64,
    root_segment: Segment,
    records: &[CoreRecord<Segment>],
    in_flight_limit: usize,
    verify_named_units: bool,
    back_chain_broken: BackChain,
    named_units_verified: NamedUnits,
) -> (replayed: Result<CoreOutcome<Segment>, CoreFailure>)
    requires
        forall|record_index: int|
            0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter
                < u64::MAX,
        forall|record_index: usize|
            record_index < records@.len() ==> #[trigger] back_chain_broken.requires((record_index,)),
        forall|record_index: usize|
            record_index < records@.len() ==> #[trigger] named_units_verified.requires(
                (record_index,),
            ),
    ensures
        match replayed {
            Ok(outcome) => m2_holds(
                root_instance,
                root_checkpoint_txg,
                root_segment,
                records@,
                outcome,
            ),
            Err(_) => true,
        },
{
    // [原文 3064–3071]（valid_records = records.len() 留给调用方）
    let mut report = CoreReport {
        above_water: 0,
        prefix_applied: 0,
        verification_passed: 0,
        verification_failed: 0,
        maximum_applied_transaction: 0,
    };
    // [原文 3073]
    let mut rebuilt = root_segment;
    // [原文 3074–3083] above 的条数。前缀规则不跨实例边界（已定项 14 第 1 条）：只有所选根自己那个实例的记录是候选。
    // [改写] 原文 filter + collect + sort_by_key 出一个 Vec，这里不建 Vec：
    // 这一遍只数条数，主循环里逐条再判同一个条件。sort_by_key（原文 3082）没抄：入参已是那个次序。
    let mut counting_index = 0;
    while counting_index < records.len()
        invariant
            counting_index <= records.len(),
            report.above_water <= counting_index,
        decreases records.len() - counting_index,
    {
        let record = &records[counting_index];
        // [原文 3078–3080]
        if is_above_the_chosen_root(record, root_instance, root_checkpoint_txg) {
            report.above_water += 1;
        }
        counting_index += 1;
    }
    // [原文 3086–3094] 链首锚点 = 所选根覆盖的最后一条：那次发布里带末条标志的那一条（已定项 14 注 1，读法乙；三方第一轮 K3）。
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
    // [原文 3095–3096] counter + 1 不溢出由前置条件（计数器不是最大值）保。
    let mut expected_next: Option<(u32, u64)> = match root_own_record_counter {
        Some(counter) => Some((root_instance, counter + 1)),
        None => None,
    };
    // [原文 3097–3106] 所选根的 checkpoint_txg 是盘上读来的 8 字节，可以是 u64 的最大值：加一溢出报损坏、不 panic（审阅第 36 条）。
    if root_checkpoint_txg == u64::MAX {
        return Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor);
    }
    let chain_start_txg_without_anchor = root_checkpoint_txg + 1;
    // [原文 3107–3108] 这次发布已经过了前四条、还没走到末条的记录。原文存 &JournalRecord，这里存下标
    let mut open: Vec<usize> = Vec::new();
    let ghost mut applied: Seq<int> = Seq::empty();
    // [原文 3109] take(in_flight_limit) 的计数
    let mut taken: usize = 0;
    let mut index = 0;
    while index < records.len()
        invariant_except_break
            match expected_next {
                Some(key) => key.0 == root_instance && ((applied + as_ints(open@)).len() == 0 ==> (
                root_own_record_counter matches Some(first_counter) && key.1 == first_counter + 1))
                    && ((applied + as_ints(open@)).len() > 0 ==> key.1 == records@[(applied
                    + as_ints(open@)).last()].counter + 1),
                None => root_own_record_counter is None && (applied + as_ints(open@)).len() == 0,
            },
            taken <= index,
            report.verification_passed <= index,
            report.verification_failed == 0,
        invariant
            index <= records.len(),
            root_checkpoint_txg < u64::MAX,
            chain_start_txg_without_anchor == root_checkpoint_txg + 1,
            forall|record_index: int|
                0 <= record_index < records@.len() ==> (#[trigger] records@[record_index]).counter
                    < u64::MAX,
            forall|record_index: usize|
                record_index < records@.len() ==> #[trigger] back_chain_broken.requires(
                    (record_index,),
                ),
            forall|record_index: usize|
                record_index < records@.len() ==> #[trigger] named_units_verified.requires(
                    (record_index,),
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
            rebuilt == if applied.len() == 0 {
                root_segment
            } else {
                records@[applied.last()].new_root_segment
            },
            // 攒着的
            forall|position: int| 0 <= position < open@.len() ==> (#[trigger] open@[position]) < index,
            forall|earlier: int, later: int|
                0 <= earlier < later < open@.len() ==> open@[earlier] < open@[later],
            forall|applied_position: int, open_position: int|
                0 <= applied_position < applied.len() && 0 <= open_position < open@.len()
                    ==> applied[applied_position] < open@[open_position],
            forall|position: int|
                0 <= position < open@.len() ==> above_the_chosen_root(
                    records@[#[trigger] open@[position] as int],
                    root_instance,
                    root_checkpoint_txg,
                ),
            is_open_publish(records_at(records@, as_ints(open@))),
            applied.len() + open@.len() <= index,
            // 还没接上链首（锚点读不出、一条都没收）时攒着的一定是空的：tsv-448 那一判冗余靠这一条
            expected_next is None ==> open@.len() == 0,
            // S2 补强：已施加加攒着的合起来就是从链首起逐一连续的一串
            anchor_is(root_instance, root_checkpoint_txg, records@, root_own_record_counter),
            chain_from_the_head(
                root_checkpoint_txg,
                records@,
                root_own_record_counter,
                applied + as_ints(open@),
            ),
        decreases records.len() - index,
    {
        let record = &records[index];
        let ghost expected_at_head = expected_next;
        // [原文 3078–3080] above 的筛选条件，同上面那一遍
        if !is_above_the_chosen_root(record, root_instance, root_checkpoint_txg) {
            index += 1;
            continue ;
        }
        // [原文 3109] take(in_flight_limit)
        if taken >= in_flight_limit {
            break ;
        }
        taken += 1;
        match expected_next {
            // [原文 3110–3113] [改写] 元组不等摊成两个字段
            Some(expected_key) => {
                if record.instance != expected_key.0 || record.counter != expected_key.1 {
                    break ;
                }
            },
            // [原文 3114–3118] 锚点读不出：链首只能是 txg = 根的 txg + 1、本次发布内序号为 1 的那条（三方第一轮 K4-b）。
            // 序号那一判与下面「攒着的为空且序号不是 1」那一判冗余（crates/mutations.tsv 第 448 行原来那条变异改的就是它）：
            // 走到这一支时攒着的一定是空的（下面这句由 Verus 证），两判之间只有反向链回调，它只会断、不改状态。
            None => {
                proof {
                    assert(open@.len() == 0);
                }
                if record.checkpoint_txg != chain_start_txg_without_anchor
                    || record.ordinal_within_publish != 1 {
                    break ;
                }
            },
        }
        // [原文 3119–3126] 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断。[改写] 三态由回调交成布尔
        if back_chain_broken(index) {
            break ;
        }
        // [原文 3127–3134] 一次发布的第一条序号是 1（已定项 4 读者规则；C539）。
        if open.is_empty() && record.ordinal_within_publish != 1 {
            break ;
        }
        // [原文 3135]
        if !open.is_empty() {
            let previous_record_of_the_open_publish = &records[open[open.len() - 1]];
            // [原文 3136–3140] 末条还没到、下一条已经换了 txg ⇒ 这次发布缺了末条（D16（发布语义）已定项 6）
            if record.checkpoint_txg != previous_record_of_the_open_publish.checkpoint_txg {
                break ;
            }
            // [原文 3141–3147] 一次发布之内跳号（已定项 4）
            if record.ordinal_within_publish as u64
                != previous_record_of_the_open_publish.ordinal_within_publish as u64 + 1 {
                break ;
            }
            // [原文 3148–3156] 上一条没带提交标记而这一条换了事务号 ⇒ 那个事务的提交标记没出现（已定项 7）
            if !previous_record_of_the_open_publish.is_commit && record.transaction
                != previous_record_of_the_open_publish.transaction {
                break ;
            }
        }
        // [原文 3158]
        expected_next = Some((record.instance, record.counter + 1));
        // [原文 3159–3163] 末条不带提交标记 ⇒ 这次发布最后一个事务没提交，整体不施加
        if !record.is_commit && record_ends_its_publish(record) {
            break ;
        }
        // [原文 3164–3172] 带末条标志的这一条之后同一 (实例代号, checkpoint_txg) 里还有读得出的记录（已定项 4 读者规则；C540）
        if record_ends_its_publish(record) && readable_record_of_the_same_publish_follows(
            records,
            index,
        ) {
            break ;
        }
        // [原文 3173–3187] [改写] .all(..) 那一串读盘由调用方的回调交成布尔；verify_named_units 关着时不调
        let all_verified = !verify_named_units || named_units_verified(index);
        // [原文 3188–3191]
        if !all_verified {
            report.verification_failed += 1;
            break ;
        }
        // [原文 3192–3197] 只数真验过的：关掉点名验证时这一格不加（fs-design.md 五条硬要求第 4 条：分支必须可观测）
        if verify_named_units {
            report.verification_passed += 1;
        }
        // [原文 3198]
        let ghost open_before = open@;
        let ghost consumed_before = applied + as_ints(open_before);
        proof {
            // 走到这里 expected_next 那一判没断：本条的计数器就是期望的（Some），或本条是无锚点时的链首（None）
            match expected_at_head {
                Some(expected_key) => {
                    assert(record.instance == expected_key.0 && record.counter == expected_key.1);
                },
                None => {
                    assert(record.checkpoint_txg == chain_start_txg_without_anchor && record.ordinal_within_publish == 1);
                },
            }
        }
        open.push(index);
        proof {
            assert(as_ints(open@) =~= as_ints(open_before).push(index as int));
            assert(applied + as_ints(open@) =~= consumed_before.push(index as int));
            let chain_after = consumed_before.push(index as int);
            assert(chain_from_the_head(
                root_checkpoint_txg,
                records@,
                root_own_record_counter,
                chain_after,
            )) by {
                assert forall|position: int|
                    0 <= position && position + 1 < chain_after.len() implies records@[(
                #[trigger] chain_after[position + 1])].counter == records@[chain_after[position]].counter + 1 by {
                    if position + 1 < consumed_before.len() {
                        assert(chain_after[position + 1] == consumed_before[position + 1]);
                        assert(chain_after[position] == consumed_before[position]);
                    }
                }
            }
        }
        proof {
            let before = records_at(records@, as_ints(open_before));
            let after = records_at(records@, as_ints(open@));
            assert(is_open_publish(before));
            assert forall|position: int| 0 <= position < after.len() implies (
            #[trigger] after[position]).checkpoint_txg == after[0].checkpoint_txg
                && after[position].ordinal_within_publish as int == position + 1 by {
                if position < open_before.len() {
                    assert(after[position] == before[position]);
                    assert(after[0] == before[0]);
                } else {
                    assert(after[position] == records@[index as int]);
                    if open_before.len() > 0 {
                        assert(before[open_before.len() - 1] == records@[open_before[open_before.len()
                            - 1] as int]);
                        assert(after[0] == before[0]);
                    }
                }
            }
            assert forall|position: int|
                0 <= position < open_before.len() implies !(#[trigger] after[position]).ends_its_publish by {
                assert(after[position] == before[position]);
            }
        }
        // [原文 3199–3202] 末条没到就接着往下走，这次发布先不施加
        if !record_ends_its_publish(record) {
            index += 1;
            continue ;
        }
        // [原文 3203]
        report.prefix_applied += open.len();
        // [原文 3204–3207]
        report.maximum_applied_transaction = maximum_transaction_of(
            records,
            open.as_slice(),
            report.maximum_applied_transaction,
        );
        proof {
            let ghost applied_before = applied;
            let this_publish = as_ints(open@);
            applied = applied + this_publish;
            let before = records_at(records@, applied_before);
            let publish = records_at(records@, this_publish);
            let after = records_at(records@, applied);
            assert(after =~= before + publish);
            assert(after.subrange(applied_before.len() as int, after.len() as int) =~= publish);
            assert(after.subrange(0, applied_before.len() as int) =~= before);
            assert(publish[publish.len() - 1] == records@[index as int]);
            assert(is_whole_publish(publish));
            assert(is_whole_publish(
                after.subrange(applied_before.len() as int, after.len() as int),
            ));
            assert(is_concatenation_of_whole_publishes(after));
            assert(applied.last() == index as int);
        }
        // [原文 3208]
        open.clear();
        // [原文 3209–3226] 换根：核里只换「新根段」这一块，RootRecord 的其余字段由调用方照原文拼
        rebuilt = record.new_root_segment;
        index += 1;
    }
    proof {
        let consumed = applied + as_ints(open@);
        assert(chain_from_the_head(root_checkpoint_txg, records@, root_own_record_counter, applied)) by {
            if applied.len() > 0 {
                assert(consumed[0] == applied[0]);
            }
            assert forall|position: int|
                0 <= position && position + 1 < applied.len() implies records@[(
            #[trigger] applied[position + 1])].counter == records@[applied[position]].counter + 1 by {
                assert(consumed[position + 1] == applied[position + 1]);
                assert(consumed[position] == applied[position]);
            }
        }
        assert(anchor_is(root_instance, root_checkpoint_txg, records@, root_own_record_counter));
    }
    // [原文 3228]
    Ok(CoreOutcome { report, rebuilt, applied: Ghost(applied) })
}

// ===== spec 自己的冒烟（防 spec 恒真或恒假）=====
proof fn a_publish_of_two_records_is_whole(first: CoreRecord<u64>, second: CoreRecord<u64>)
    requires
        first.checkpoint_txg == second.checkpoint_txg,
        first.ordinal_within_publish == 1,
        second.ordinal_within_publish == 2,
        !first.ends_its_publish,
        second.ends_its_publish,
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
        !only.ends_its_publish,
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

// 平时的 cargo test 跑的两条：Verus 证的是 spec，这两条钉住编进产品的那一份在两个手算得出的小例子上给出的数
// （crates/mutations.tsv 里点名它们的变异改坏核，它们红）。
#[cfg(test)]
mod tests {
    use super::{replay_core, CoreRecord};

    const ROOT_INSTANCE: u32 = 1;
    const ROOT_CHECKPOINT_TXG: u64 = 1;
    const ROOT_SEGMENT: u64 = 0;
    const IN_FLIGHT_LIMIT: usize = 16;

    fn record(
        counter: u64,
        checkpoint_txg: u64,
        ordinal_within_publish: u32,
        ends_its_publish: bool,
        new_root_segment: u64,
    ) -> CoreRecord<u64> {
        CoreRecord {
            instance: ROOT_INSTANCE,
            counter,
            checkpoint_txg,
            transaction: counter + 3,
            is_commit: true,
            ordinal_within_publish,
            ends_its_publish,
            new_root_segment,
        }
    }

    /// 所选根那次发布的末条（计数器 1）之后，txg 2 的发布两条都在、第二条带末条标志 ⇒ 整次施加，新根段取末条的。
    #[test]
    fn a_two_record_publish_after_the_anchor_is_applied_whole_and_the_new_root_segment_comes_from_its_last_record(
    ) {
        let records = [
            record(1, ROOT_CHECKPOINT_TXG, 1, true, 10),
            record(2, 2, 1, false, 20),
            record(3, 2, 2, true, 30),
        ];
        let Ok(outcome) = replay_core(
            ROOT_INSTANCE,
            ROOT_CHECKPOINT_TXG,
            ROOT_SEGMENT,
            &records,
            IN_FLIGHT_LIMIT,
            false,
            |_index: usize| false,
            |_index: usize| true,
        ) else {
            panic!("锚点只有一条、txg 不是最大值：核不该报错");
        };
        assert_eq!(outcome.report.above_water, 2, "水位之上是 txg 2 的两条");
        assert_eq!(outcome.report.prefix_applied, 2, "两条整次施加");
        assert_eq!(outcome.rebuilt, 30, "新根段取那次发布末条的");
        assert_eq!(
            outcome.report.maximum_applied_transaction, 6,
            "事务号取施加的里最大的"
        );
    }

    /// 同样的锚点，txg 2 的发布只有第一条（不带末条标志）⇒ 末条没到，一条都不施加，新根段还是所选根的。
    #[test]
    fn a_publish_whose_last_record_is_missing_is_not_applied_and_the_root_segment_stays() {
        let records = [
            record(1, ROOT_CHECKPOINT_TXG, 1, true, 10),
            record(2, 2, 1, false, 20),
        ];
        let Ok(outcome) = replay_core(
            ROOT_INSTANCE,
            ROOT_CHECKPOINT_TXG,
            ROOT_SEGMENT,
            &records,
            IN_FLIGHT_LIMIT,
            false,
            |_index: usize| false,
            |_index: usize| true,
        ) else {
            panic!("锚点只有一条、txg 不是最大值：核不该报错");
        };
        assert_eq!(outcome.report.above_water, 1, "水位之上是 txg 2 那一条");
        assert_eq!(outcome.report.prefix_applied, 0, "末条没到：一条都不施加");
        assert_eq!(outcome.rebuilt, ROOT_SEGMENT, "新根段还是所选根的");
    }
}

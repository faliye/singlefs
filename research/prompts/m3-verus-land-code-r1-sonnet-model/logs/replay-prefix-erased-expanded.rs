#![feature(prelude_import)]
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
extern crate std;
#[prelude_import]
use std::prelude::rust_2021::*;

use vstd::prelude::*;


// ===== 核的数据 =====
// JournalRecord（singlefs-core 的 journal.rs）里取前缀用得到的字段；枚举与 newtype 摊成原始类型。
pub struct CoreRecord<Segment> {
    pub instance: u32,
    // InstanceGeneration.0
    pub counter: u64,
    pub checkpoint_txg: u64,
    // CheckpointTxg.0
    pub transaction: u64,
    pub is_commit: bool,
    pub ordinal_within_publish: u32,
    // JournalRecordOrdinalWithinPublish.0
    pub ends_its_publish: bool,
    // place_in_publish == LastRecordOfThePublish
    pub new_root_segment: Segment, // 施加时换进根里的那几个字段（原文 3216–3222）
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
    pub applied: Ghost<Seq<int>>, // 施加了的记录在入参切片里的下标，幽灵量，编译时擦掉
}

// ===== spec（可信：照条款文字写，没有别的东西核它）=====
// 已定项 14 定案句：只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录；注 1：实例代号与所选根不同即停。

// 一次整的发布：checkpoint_txg 相同、序号依次 1..k、只有第 k 条带末条标志（已定项 4 / 14 第五条 / 17）。

// 若干整次发布的拼接（M2：重放结果等于整数个发布，不存在被前缀切开的发布）。



// ===== 证明用的记法（不可信件：写错了只会证不出来）=====

// 末条还没到的那一段：同 txg、序号 1..n、都不带末条标志。

// ===== 纯函数核 =====
// [原文 2930–2937] record_ends_its_publish：记录标志位 0（已定项 14 第五条、已定项 17）。[改写] 原文 match 两臂枚举，这里读布尔字段。
fn record_ends_its_publish<Segment>(record: &CoreRecord<Segment>) -> bool {
    record.ends_its_publish
}

// [原文 2939–2953] readable_record_of_the_same_publish_follows：同一 (实例代号, checkpoint_txg) 里这一条之后（计数器更大）
// 还有没有读得出的记录。同一实例里计数器与 checkpoint_txg 一起往上走，所以按计数器升序看到 txg 越过这一条的就停。
// [改写] 原文在 BTreeMap 上 range((instance, counter+1)..=(instance, u64::MAX))，这里从切片里这一条的下一个位置往后扫，
// 靠「切片按 (实例代号, 计数器) 严格升序」才与原文同义；这一条没有写成前置条件，也没有证。
fn readable_record_of_the_same_publish_follows<Segment>(records:
        &[CoreRecord<Segment>], index: usize) -> bool {
    let record = &records[index];
    // [原文 2945–2947] checked_add(1) 取不到就 false
    if record.counter == u64::MAX { return false; }
    let next_counter = record.counter + 1;
    let mut later_index = index + 1;
    while later_index < records.len() {
        let later = &records[later_index];
        // [原文 2949] range 的上下界：同实例、计数器 ≥ next_counter
        if later.instance == record.instance && later.counter >= next_counter
            {
            // [原文 2951] take_while(later.checkpoint_txg <= record.checkpoint_txg)
            if !(later.checkpoint_txg <= record.checkpoint_txg) {
                return false;
            }
            // [原文 2952] any(later.checkpoint_txg == record.checkpoint_txg)
            if later.checkpoint_txg == record.checkpoint_txg { return true; }
        }
        later_index += 1;
    }
    false
}

// [原文 2955–2987] counter_of_the_last_record_the_root_covers：所选根覆盖的最后一条记录的 jsn 计数器（已定项 14 注 1，读法乙）——
// 所选根那次发布读得出的几条里带末条标志的那一条；一条都不带 ⇒ Ok(None)；带标志的多于一条 ⇒ 认哪一条条款没写，报错。
// [改写] 原文 filter + collect 成 Vec 再看 len() > 1、取 first()；这里一遍循环，第二条带标志的出现就报错。
// 报错里的 counters 清单没带（核不产 Vec），由调用方再调原来那个函数拿。
fn counter_of_the_last_record_the_root_covers<Segment>(root_instance: u32,
    root_checkpoint_txg: u64, records: &[CoreRecord<Segment>])
    -> Result<Option<u64>, CoreFailure> {
    let mut found: Option<u64> = None;
    let mut index = 0;
    while index < records.len() {
        let record = &records[index];
        // [原文 2970–2972]
        if record.instance == root_instance &&
                    record.checkpoint_txg == root_checkpoint_txg &&
                record_ends_its_publish(record) {
            // [原文 2976–2984]
            if found.is_some() {
                return Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlag);
            }
            found = Some(record.counter);
        }
        index += 1;
    }
    // [原文 2985–2986]
    Ok(found)
}

// [原文 3074–3080] above 的筛选条件：前缀规则不跨实例边界（已定项 14 第 1 条），只有所选根自己那个实例、txg 比根大的记录是候选。
// 数条数那一遍与主循环共用这一判，同原文只筛一次（crates/mutations.tsv 第 23、355 行改的就是它，两处一起变）。
// [改写] 原文比元组 (record.instance, record.checkpoint_txg) > water，这里把字典序摊开写。
fn is_above_the_chosen_root<Segment>(record: &CoreRecord<Segment>,
    root_instance: u32, root_checkpoint_txg: u64) -> bool {
    record.instance == root_instance &&
        (record.instance > root_instance ||
                (record.instance == root_instance &&
                        record.checkpoint_txg > root_checkpoint_txg))
}

// [原文 3204–3207] 攒着的这次发布里最大的事务号。[改写] 原文 iter().map().fold(.., u64::max)。
fn maximum_transaction_of<Segment>(records: &[CoreRecord<Segment>],
    open: &[usize], start: u64) -> u64 {
    let mut maximum = start;
    let mut position = 0;
    while position < open.len() {
        let transaction = records[open[position]].transaction;
        if transaction > maximum { maximum = transaction; }
        position += 1;
    }
    maximum
}

// [原文 3057–3229] replay_journal 去掉读盘之后的那一截。
// records：全部自证过的记录，次序就是 BTreeMap<(InstanceGeneration, u64), _>::values() 的次序。
// 前置条件只有一条：计数器不是 u64 的最大值（原文 3096、3158 的 counter + 1 在那个值上溢出）。没有要求排序，条数不设界。
#[allow(clippy :: too_many_arguments, reason =
"两个回调是读盘的两处，摊成参数比包一层结构体好对照原文")]
pub fn replay_core<Segment: Copy, BackChain: Fn(usize) -> bool,
    NamedUnits: Fn(usize)
    ->
        bool>(root_instance: u32, root_checkpoint_txg: u64,
    root_segment: Segment, records: &[CoreRecord<Segment>],
    in_flight_limit: usize, verify_named_units: bool,
    back_chain_broken: BackChain, named_units_verified: NamedUnits)
    -> Result<CoreOutcome<Segment>, CoreFailure> {
    // [原文 3064–3071]（valid_records = records.len() 留给调用方）
    let mut report =
        CoreReport {
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
    while counting_index < records.len() {
        let record = &records[counting_index];
        // [原文 3078–3080]
        if is_above_the_chosen_root(record, root_instance,
                root_checkpoint_txg) {
            report.above_water += 1;
        }
        counting_index += 1;
    }
    // [原文 3086–3094] 链首锚点 = 所选根覆盖的最后一条：那次发布里带末条标志的那一条（已定项 14 注 1，读法乙；三方第一轮 K3）。
    let root_own_record_counter =
        match counter_of_the_last_record_the_root_covers(root_instance,
                root_checkpoint_txg, records) {
            Ok(counter) => counter,
            Err(failure) => { return Err(failure); }
        };
    // [原文 3095–3096] counter + 1 不溢出由前置条件（计数器不是最大值）保。
    let mut expected_next: Option<(u32, u64)> =
        root_own_record_counter.map(|counter: u64| -> (u32, u64)
                { (root_instance, counter + 1) });
    // [原文 3097–3106] 所选根的 checkpoint_txg 是盘上读来的 8 字节，可以是 u64 的最大值：加一溢出报损坏、不 panic（审阅第 36 条）。
    if root_checkpoint_txg == u64::MAX {
        return Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor);
    }
    let chain_start_txg_without_anchor = root_checkpoint_txg + 1;
    // [原文 3107–3108] 这次发布已经过了前四条、还没走到末条的记录。原文存 &JournalRecord，这里存下标
    let mut open: Vec<usize> = Vec::new();
    // [原文 3109] take(in_flight_limit) 的计数
    let mut taken: usize = 0;
    let mut index = 0;
    while index < records.len()
        // 已施加的
        // 攒着的
        // 还没接上链首（锚点读不出、一条都没收）时攒着的一定是空的：tsv-448 那一判冗余靠这一条
        {
        let record = &records[index];
        // [原文 3078–3080] above 的筛选条件，同上面那一遍
        if !is_above_the_chosen_root(record, root_instance,
                    root_checkpoint_txg) {
            index += 1;
            continue;
        }
        // [原文 3109] take(in_flight_limit)
        if taken >= in_flight_limit { break; }
        taken += 1;
        match expected_next {

                // [原文 3110–3113] [改写] 元组不等摊成两个字段
                Some(expected_key) => {
                if record.instance != expected_key.0 ||
                        record.counter != expected_key.1 {
                    break;
                }
            }

                // [原文 3114–3118] 锚点读不出：链首只能是 txg = 根的 txg + 1、本次发布内序号为 1 的那条（三方第一轮 K4-b）。
                // 序号那一判与下面「攒着的为空且序号不是 1」那一判冗余（crates/mutations.tsv 第 448 行原来那条变异改的就是它）：
                // 走到这一支时攒着的一定是空的（下面这句由 Verus 证），两判之间只有反向链回调，它只会断、不改状态。
                None => {
                {}
                if record.checkpoint_txg != chain_start_txg_without_anchor ||
                        record.ordinal_within_publish != 1 {
                    break;
                }
            }
        }
        // [原文 3119–3126] 反向链（已定项 7、I-8.6）：Holds | PreviousRecordNotOnDisk 接着走，Broken 断。[改写] 三态由回调交成布尔
        if back_chain_broken(index) { break; }
        // [原文 3127–3134] 一次发布的第一条序号是 1（已定项 4 读者规则；C539）。
        if open.is_empty() && record.ordinal_within_publish != 1 { break; }
        // [原文 3135]
        if !open.is_empty() {
            let previous_record_of_the_open_publish =
                &records[open[open.len() - 1]];
            // [原文 3136–3140] 末条还没到、下一条已经换了 txg ⇒ 这次发布缺了末条（D16（发布语义）已定项 6）
            if record.checkpoint_txg !=
                    previous_record_of_the_open_publish.checkpoint_txg {
                break;
            }
            // [原文 3141–3147] 一次发布之内跳号（已定项 4）
            if record.ordinal_within_publish as u64 !=
                    previous_record_of_the_open_publish.ordinal_within_publish
                            as u64 + 1 {
                break;
            }
            // [原文 3148–3156] 上一条没带提交标记而这一条换了事务号 ⇒ 那个事务的提交标记没出现（已定项 7）
            if !previous_record_of_the_open_publish.is_commit &&
                    record.transaction !=
                        previous_record_of_the_open_publish.transaction {
                break;
            }
        }
        // [原文 3158]
        expected_next = Some((record.instance, record.counter + 1));
        // [原文 3159–3163] 末条不带提交标记 ⇒ 这次发布最后一个事务没提交，整体不施加
        if !record.is_commit && record_ends_its_publish(record) { break; }
        // [原文 3164–3172] 带末条标志的这一条之后同一 (实例代号, checkpoint_txg) 里还有读得出的记录（已定项 4 读者规则；C540）
        if record_ends_its_publish(record) &&
                readable_record_of_the_same_publish_follows(records, index) {
            break;
        }
        // [原文 3173–3187] [改写] .all(..) 那一串读盘由调用方的回调交成布尔；verify_named_units 关着时不调
        let all_verified = !verify_named_units || named_units_verified(index);
        // [原文 3188–3191]
        if !all_verified { report.verification_failed += 1; break; }
        // [原文 3192–3197] 只数真验过的：关掉点名验证时这一格不加（fs-design.md 五条硬要求第 4 条：分支必须可观测）
        if verify_named_units { report.verification_passed += 1; }
        // [原文 3198]
        open.push(index);
        {}
        // [原文 3199–3202] 末条没到就接着往下走，这次发布先不施加
        if !record_ends_its_publish(record) { index += 1; continue; }
        // [原文 3203]
        report.prefix_applied += open.len();
        // [原文 3204–3207]
        report.maximum_applied_transaction =
            maximum_transaction_of(records, open.as_slice(),
                report.maximum_applied_transaction);
        {}
        // [原文 3208]
        open.clear();
        // [原文 3209–3226] 换根：核里只换「新根段」这一块，RootRecord 的其余字段由调用方照原文拼
        rebuilt = record.new_root_segment;
        index += 1;
    }
    // [原文 3228]
    Ok(CoreOutcome {
            report,
            rebuilt,
            applied: Ghost::assume_new_fallback(||
                    // ===== spec 自己的冒烟（防 spec 恒真或恒假）=====


                    // verus!

                    // 平时的 cargo test 跑的两条：Verus 证的是 spec，这两条钉住编进产品的那一份在两个手算得出的小例子上给出的数
                    // （crates/mutations.tsv 里点名它们的变异改坏核，它们红）。




                    ::core::panicking::panic("internal error: entered unreachable code")),
        })
}

//! journal 里的发布：事务号、提交标记、发布序号与末条标志、末条记录的新根段与根、点名项、在飞上限，
//! 以及下一次挂载的恢复会做的重放（I-4.1（重放后树可遍历））。

use super::*;

/// 不承载事务的空发布记录在记录头上写的事务号（D23（journal 的角色与格式） 已定项 19 ①）。
const TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH: u64 = 0;

/// I-8.7（实例内事务号不重号）：**同一实例**在环里写出的记录按 jsn 排下来，一个事务号只占一段。两条判据，
/// 红了在违例说明里标出是哪一条（「判据①」「判据②」）：
/// ① 按计数器相邻的两条非 0 记录，事务号**不减**（相等或严格递增：D23（journal 的角色与格式） 已定项 7
///    「一个事务可以跨多条记录」，同一事务的记录共享事务号）；
/// ② 一个事务号的记录在计数器上**连续**：被别的记录（事务号 0 的空发布也算）隔开之后不再出现。
/// 这是已定项 7「事务号按实例计数、从 1 起」「记录按事务号顺序追加……这是实例表行的 W 能当精确前缀的依据」的投影：
/// 同一实例里号不许重（一个号不许给两个隔开的事务用）、也不许往回走。
/// 逐盘判：环两盘互为镜像，一块盘上的记录坏了另一块不该跟着坏（与 I-8.6（反向链算法） 同一个口径）。
///
/// **射程**（照判据原样，多判一格少判一格都是错）：
/// ① **跨实例不判**：事务号各实例各算各的，两个实例各有一个 1 不是违例；这一条只比同一个实例代号的记录。
/// ② **事务号 0 不进 ① 的序列**：它是不承载事务的空发布记录（已定项 19 ①），夹在同一实例两个**不同**的非 0 号中间
///    （抬回退下界那两次空发布就是这个形态）照样合法；夹在两条**同号**记录中间就是把那个号隔开了，② 红
///    （一次发布的记录不会被另一次空发布插进来）。
/// ③ 两条同号记录之间只有读不出的槽、或别的实例的记录 ⇒ 看不到本实例在那几个计数器上写过什么，② 不判红。
/// ④ 同一块盘上同一实例非 0 的记录不足两条 ⇒ 没有一对号可比，整条报「不适用」并带理由。
///
/// 判的一格是「同一块盘、同一实例、按计数器排下来相邻的两条非 0 记录」：号相等时看它们之间有没有本实例别的记录（②），
/// 号不等时要求严格变大（①，号不等时「不减」就是严格变大）；这个号之前出现过（被别的非 0 号隔开之后又出现，②）
/// 一定伴着一处 ① 红，不另判，只在正好是同一对时写进 ① 那句说明。
/// `records` 按计数器索引，计数器与 jsn 在同一实例里同序（jsn = 实例代号 32 位 + 计数器 48 位，
/// D23（journal 的角色与格式） 已定项 9）⇒ 按计数器遍历就是按 jsn 排序。
///
/// ⚠️ 它钉的是「事务号计数只活在内存里」那个前提（`crates/singlefs-core/src/transaction.rs` 的
/// `publish_overwrite` 取这个实例见过的最大事务号加一、不取上一条记录上的事务号加一）：那条链式传递成立与否
/// 压在「每条恢复路径之后都取新实例代号」上，而在这一条之前没有任何东西判得出它失效。
/// 两个各自提交了的事务撞号而又紧挨着时，号不减、同号相连，这一条与「一事务两条」分不开——分得开的是提交标记，
/// 归 I-8.8（前缀里的事务不被切开）。
pub(super) fn judge_transaction_numbers_per_instance(
    records_by_device: &[(u32, JournalRecordsOfADevice)],
    judgements: &mut Judgements,
) -> ComparablePairsOfTransactionNumbers {
    let mut judged_pairs = 0u64;
    for (device, records) in records_by_device {
        let mut newest_non_empty_publish_by_instance: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
        // (实例代号, 事务号) → 这个号在这块盘上第一次出现的计数器。
        let mut first_counter_by_transaction: BTreeMap<(u32, u64), u64> = BTreeMap::new();
        for (JournalRecordKeyOnADevice { counter, .. }, record) in records {
            if record.transaction == TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH {
                continue;
            }
            let first_counter_of_this_number = *first_counter_by_transaction
                .entry((record.instance, record.transaction))
                .or_insert(*counter);
            let previous = newest_non_empty_publish_by_instance
                .insert(record.instance, (*counter, record.transaction));
            let Some((previous_counter, previous_transaction)) = previous else {
                continue;
            };
            judged_pairs += 1;
            if record.transaction == previous_transaction {
                let separating_record =
                    records_with_counters_between(records, previous_counter + 1, *counter)
                        .find(|(_, between)| between.instance == record.instance);
                judgements.judge("I-8.7", separating_record.is_none(), || {
                    let (separating_key, separating) =
                        separating_record.expect("判红的这一支里，隔开它们的那条记录找到了");
                    format!(
                        "判据②：盘 {device} 实例 {} 事务号 {} 的两条记录（计数器 {previous_counter}、{counter}）中间坐着计数器 {}（环槽 {}）上事务号 {} 的记录——这个号被隔开之后又出现",
                        record.instance, record.transaction, separating_key.counter, separating.slot, separating.transaction
                    )
                });
                continue;
            }
            // 号不等时，「被别的非 0 号隔开之后又出现」（②）一定伴着一处 ①：号先变走再变回来，中间必有一对是减的。
            // 所以 ② 的这一半不单独判，只在 ① 红的这一对上正好也是它时写进同一句说明。
            let reappears_after_another_number = first_counter_of_this_number != *counter;
            judgements.judge("I-8.7", record.transaction > previous_transaction, || {
                let decrease = format!(
                    "判据①：盘 {device} 实例 {} 的两条记录：计数器 {previous_counter} 上事务号 {previous_transaction}，计数器 {counter}（环槽 {}）上事务号 {}——同一实例按 jsn 排下来事务号减了",
                    record.instance, record.slot, record.transaction
                );
                if reappears_after_another_number {
                    format!(
                        "{decrease}；判据②：事务号 {} 在计数器 {first_counter_of_this_number} 出现过，被别的号隔开之后又在计数器 {counter} 出现",
                        record.transaction
                    )
                } else {
                    decrease
                }
            });
        }
    }
    if judged_pairs == 0 {
        ComparablePairsOfTransactionNumbers::NoPairJudged
    } else {
        ComparablePairsOfTransactionNumbers::SomePairJudged
    }
}

/// I-8.7 ① ② 判没判到一对可比的号（[`judge_transaction_numbers_per_instance`] 交回）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ComparablePairsOfTransactionNumbers {
    SomePairJudged,
    NoPairJudged,
}

/// I-8.7「从 1 起」那半句在哪个实例上判过且成立（[`judge_transaction_numbers_start_from_one_per_instance`] 交回）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TransactionNumbersFromOneJudged {
    HeldOnSomeInstance,
    HeldOnNoInstance,
}

/// I-8.7 ① ② 一对可比的号都凑不出来、「从 1 起」也没判过时的理由。
const NO_COMPARABLE_PAIR_OF_TRANSACTION_NUMBERS: &str =
    "环里没有哪个实例写出过两条非 0 事务号的记录：一对可比的号都凑不出来";
/// I-8.7 ① ② 一对可比的号都凑不出来、「从 1 起」判过且成立时的理由（C469 ②，判决
/// `research/prompts/m3-close-b19-design-r2-main-verification.md` 第二节 Q3 采甲加理由句：成立不单独算评估，理由句说真话）。
const TRANSACTION_NUMBERS_FROM_ONE_HELD_WITHOUT_A_COMPARABLE_PAIR: &str =
    "从 1 起判过且成立，凑不出一对可比的号：环里没有哪个实例写出过两条非 0 事务号的记录";

/// I-8.7 ① ② 一对都没判到时整条报不适用，理由按「从 1 起」那半句判没判过选：判过且成立的那一句不让理由说假话
/// （「从 1 起」在可达镜像上恒成立、判别力在坏镜像上，成立不单独算一次评估，同 I-8.8 的提交标记字节）。判到一对的不记。
pub(super) fn note_transaction_numbers_without_a_comparable_pair(
    comparable_pairs: ComparablePairsOfTransactionNumbers,
    from_one: TransactionNumbersFromOneJudged,
    judgements: &mut Judgements,
) {
    match (comparable_pairs, from_one) {
        (ComparablePairsOfTransactionNumbers::SomePairJudged, _) => {}
        (
            ComparablePairsOfTransactionNumbers::NoPairJudged,
            TransactionNumbersFromOneJudged::HeldOnSomeInstance,
        ) => judgements.not_applicable(
            "I-8.7",
            TRANSACTION_NUMBERS_FROM_ONE_HELD_WITHOUT_A_COMPARABLE_PAIR,
        ),
        (
            ComparablePairsOfTransactionNumbers::NoPairJudged,
            TransactionNumbersFromOneJudged::HeldOnNoInstance,
        ) => judgements.not_applicable("I-8.7", NO_COMPARABLE_PAIR_OF_TRANSACTION_NUMBERS),
    }
}

/// 事务号的第一个号（D23（journal 的角色与格式） 已定项 7「事务号按实例计数、从 1 起」）。
const NEW_POOL_FILE_CREATION_NUMBER_OF_AN_INSTANCE: u64 = 1;

/// 一个实例在一块盘上计数器最小的那条记录，是不是这个实例写出的第一条（C469 ②）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FirstRecordOfAnInstanceOnADevice {
    /// 计数器是 1，或计数器小 1 那一格坐着别的实例的自证过的记录：这一条就是这个实例的第一条。
    IsTheFirstRecordOfTheInstance,
    /// 计数器小 1 那一格坐着环转过一圈之后的记录（计数器大一个环长）：这个实例的第一条被盖掉了，射程外。
    OverwrittenByTheRingWrappingOverIt,
    /// 计数器小 1 那一格读不出（没有自证过的记录，也不是环转过一圈之后的）：判不了，报部分没判。
    PredecessorDoesNotReadBack,
}

/// I-8.7（实例内事务号不重号）D23（journal 的角色与格式） 已定项 7「事务号按实例计数、从 1 起」那半句（C469 ②）：
/// 逐盘（与 ① ② 同一个口径），每个在这块盘上写出过非 0 事务号记录的实例，它写出的第一条非 0 记录的事务号是 1。
/// 判得了要两件都在盘上：① 这个实例的第一条记录还在环里（[`FirstRecordOfAnInstanceOnADevice`]：计数器 1，或前一格是别的实例的记录）；
/// ② 从那一条到这个实例第一条非 0 记录之间每个计数器上都有自证过的记录（中间读不出的一格可能就是这个实例更早的非 0 记录）。
/// 判红只在第一条非 0 记录的号不是 1 时记；成立时不单独算一次评估（只有一条非 0 记录的镜像上照旧报 ① 的不适用）。
/// 环转过一圈盖掉了第一条的，这半句在这个实例上射程外，什么都不记（不记评估、不记不适用；判决
/// `research/prompts/m3-close-b19-checker-r1-main-verification.md` 第二节 K3 ①）：整条的判定由 ① ② 定。
/// 中间读不出的报部分没判、逐个列出（C701 那一条）。一个非 0 记录都没写的实例没有对象。
pub(super) fn judge_transaction_numbers_start_from_one_per_instance(
    records_by_device: &[(u32, JournalRecordsOfADevice)],
    ring_slots: u64,
    judgements: &mut Judgements,
) -> TransactionNumbersFromOneJudged {
    let mut unjudged_because_a_record_does_not_read_back: Vec<String> = Vec::new();
    let mut from_one = TransactionNumbersFromOneJudged::HeldOnNoInstance;
    for (device, records) in records_by_device {
        // 按计数器从小到大走（键是 (计数器, 实例代号)）：每个实例第一次碰到的就是它计数器最小的那条、第一条非 0 记录。
        let mut smallest_counter_by_instance: BTreeMap<u32, u64> = BTreeMap::new();
        let mut first_non_empty_publish_by_instance: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
        for (JournalRecordKeyOnADevice { counter, instance }, record) in records {
            smallest_counter_by_instance
                .entry(*instance)
                .or_insert(*counter);
            if record.transaction != TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH {
                first_non_empty_publish_by_instance
                    .entry(*instance)
                    .or_insert((*counter, record.transaction));
            }
        }
        for (instance, (non_empty_counter, transaction)) in &first_non_empty_publish_by_instance {
            let smallest_counter = *smallest_counter_by_instance
                .get(instance)
                .expect("有非 0 记录的实例在同一遍里记过计数器最小的那条");
            // 计数器 0 前面没有格（它本身归 I-8.6 判红），与计数器 1 一样当第一条。
            let first_record = match smallest_counter
                .checked_sub(FIRST_JOURNAL_COUNTER)
                .filter(|counter_before| *counter_before >= FIRST_JOURNAL_COUNTER)
            {
                None => FirstRecordOfAnInstanceOnADevice::IsTheFirstRecordOfTheInstance,
                Some(counter_before) => {
                    if records_at_counter(records, counter_before).next().is_some() {
                        FirstRecordOfAnInstanceOnADevice::IsTheFirstRecordOfTheInstance
                    } else if counter_before.checked_add(ring_slots).is_some_and(
                        |counter_one_lap_later| {
                            records_at_counter(records, counter_one_lap_later)
                                .next()
                                .is_some()
                        },
                    ) {
                        FirstRecordOfAnInstanceOnADevice::OverwrittenByTheRingWrappingOverIt
                    } else {
                        FirstRecordOfAnInstanceOnADevice::PredecessorDoesNotReadBack
                    }
                }
            };
            match first_record {
                FirstRecordOfAnInstanceOnADevice::IsTheFirstRecordOfTheInstance => {}
                // 记成射程里一部分没判（`Judgements::part_of_the_range_not_judged`）会让整条报不适用：`Judgements::into_report` 里它排在成立之前，
                // ① ② 已判成立的整条被压掉。射程外的这一形不留痕迹，整条照 ① ② 报。
                FirstRecordOfAnInstanceOnADevice::OverwrittenByTheRingWrappingOverIt => continue,
                FirstRecordOfAnInstanceOnADevice::PredecessorDoesNotReadBack => {
                    unjudged_because_a_record_does_not_read_back.push(format!(
                        "盘 {device} 实例 {instance}：计数器最小的那条是 {smallest_counter}，前一个计数器那一格读不出，认不出它是不是这个实例的第一条"
                    ));
                    continue;
                }
            }
            if let Some(counter_that_does_not_read_back) = (smallest_counter + 1
                ..*non_empty_counter)
                .find(|counter| records_at_counter(records, *counter).next().is_none())
            {
                unjudged_because_a_record_does_not_read_back.push(format!(
                    "盘 {device} 实例 {instance}：第一条记录（计数器 {smallest_counter}）到第一条非 0 记录（计数器 {non_empty_counter}）之间计数器 {counter_that_does_not_read_back} 读不出"
                ));
                continue;
            }
            // 只在不成立时记一次评估（判红）：成立时不单独把 I-8.7 从「一对可比的号都凑不出来」的不适用抬成成立，
            // 只记下判过且成立，不适用的理由句照它选。
            if *transaction != NEW_POOL_FILE_CREATION_NUMBER_OF_AN_INSTANCE {
                judgements.judge("I-8.7", false, || {
                    format!(
                        "从 1 起：盘 {device} 实例 {instance} 写出的第一条非 0 记录（计数器 {non_empty_counter}）事务号是 {transaction}，不是 {NEW_POOL_FILE_CREATION_NUMBER_OF_AN_INSTANCE}"
                    )
                });
            } else {
                from_one = TransactionNumbersFromOneJudged::HeldOnSomeInstance;
            }
        }
    }
    if !unjudged_because_a_record_does_not_read_back.is_empty() {
        judgements.partially_judged(
            "I-8.7",
            format!(
                "「事务号从 1 起」那半句没判：{}",
                unjudged_because_a_record_does_not_read_back.join("；")
            ),
        );
    }
    from_one
}

/// I-8.8（前缀里的事务不被切开）：同一实例在环里写出的记录，按事务看提交标记。判据各自单独判，
/// 红了在违例说明里逐条列出是哪几条（「判据③」「提交标记字节」「判据④」；① ② 是 I-8.7（实例内事务号不重号） 的）：
/// ③ 一组里带提交标记的至多一条，且它是这一组计数器最大的那一条；
/// 提交标记字节：那 1 字节只许 0 或 1，别的值判红（不许把它静默读成「不带」）；
/// ④ 一组里一条提交标记都没有时，它后面这个实例不许再有任何记录（事务号 0 的空发布也算）：
///    D23（journal 的角色与格式） 已定项 7「一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加」——
///    失败之后这个实例不再写任何记录（空发布也在内）。
/// 逐盘判（与 I-8.6（反向链算法）、I-8.7 同一个口径）。一组 = 同一块盘上实例代号与事务号都相同的非 0 记录：
/// 事务号按实例计数（已定项 7），两个实例各有一个 1 是两个事务。组里的次序不在这里判——同号记录连不连续归 I-8.7 的 ②。
///
/// **射程**：
/// 一、**事务号 0 不成组**：它是不承载事务的空发布记录（D23（journal 的角色与格式） 已定项 19 ①
///    「不进『丢掉提交标记没出现的尾巴』判定」）；它自己带提交标记 1，拿它成组 ③ 当场就红。它照样受「提交标记字节」管。
/// 二、④ 看的是同一实例在这一组之后的每一条记录，不分号大号小、不分是不是事务号 0：没提交的组后面跟着一条空发布也红；
///    别的实例的记录不算（新实例接着写是失败之后该有的样子）。
/// 三、**一组里一条提交标记都没有，本身合法**：那是提交标记还没落盘的尾巴，恢复要丢掉的正是它（④ 管它是不是尾巴）。
///
/// **不适用**：同一块盘上没有哪一组两条以上（③ 没有对象）、也没有哪一组一条提交标记都没有（④ 没有对象）
/// ⇒ 整条报「不适用」，不报成立。「提交标记字节」这时照样判、判出别的值照样红，只是它单独成立不算这一条成立——
/// 第一版一事务一条、每条都带提交标记，可达镜像上都是这个形态，这一条的判别力在坏镜像上
/// （`crates/singlefs-verification/tests/checker_known_bad_images.rs`）。
pub(super) fn judge_commit_markers_per_transaction(
    records_by_device: &[(u32, JournalRecordsOfADevice)],
    judgements: &mut Judgements,
) {
    let mut first_misplaced_commit_marker: Option<String> = None;
    let mut first_unrecognized_commit_marker: Option<String> = None;
    let mut first_record_after_an_uncommitted_transaction: Option<String> = None;
    let mut judged_transactions = 0u64;
    for (device, records) in records_by_device {
        for (JournalRecordKeyOnADevice { counter, .. }, record) in records {
            match record.commit_marker {
                CommitMarker::Absent | CommitMarker::Present => {}
                CommitMarker::Unrecognized(byte) => {
                    first_unrecognized_commit_marker.get_or_insert_with(|| {
                        format!(
                            "盘 {device} 实例 {} 计数器 {counter}（环槽 {}）那条记录的提交标记字节是 {byte}——只许 0 或 1",
                            record.instance, record.slot
                        )
                    });
                }
            }
        }
        let mut records_by_transaction: BTreeMap<(u32, u64), Vec<(u64, &ScannedJournalRecord)>> =
            BTreeMap::new();
        for (JournalRecordKeyOnADevice { counter, .. }, record) in records
            .iter()
            .filter(|(_, scanned)| scanned.transaction != TRANSACTION_NUMBER_OF_AN_EMPTY_PUBLISH)
        {
            records_by_transaction
                .entry((record.instance, record.transaction))
                .or_default()
                .push((*counter, record));
        }
        for ((instance, transaction), members) in &records_by_transaction {
            let (last_counter, _) = members
                .last()
                .expect("每一组至少一条：组是拿记录一条条追加出来的");
            let member_counters: Vec<u64> = members.iter().map(|(counter, _)| *counter).collect();
            let commit_marked_counters: Vec<u64> = members
                .iter()
                .filter(|(_, member)| member.commit_marker == CommitMarker::Present)
                .map(|(counter, _)| *counter)
                .collect();
            if members.len() >= 2 {
                judged_transactions += 1;
            }
            let commit_marker_misplaced = match commit_marked_counters.as_slice() {
                [] => false,
                [only_commit_marked] => only_commit_marked != last_counter,
                [_, _, ..] => true,
            };
            if commit_marker_misplaced {
                first_misplaced_commit_marker.get_or_insert_with(|| {
                    format!(
                        "盘 {device} 实例 {instance} 事务号 {transaction} 的记录（计数器 {member_counters:?}）里带提交标记的是计数器 {commit_marked_counters:?}——至多一条、且只许是这一组计数器最大的 {last_counter}"
                    )
                });
            }
            if !commit_marked_counters.is_empty() {
                continue;
            }
            judged_transactions += 1;
            if let Some((later_key, later)) = counter_after(*last_counter).and_then(|next| {
                records_from_counter(records, next).find(|(_, later)| later.instance == *instance)
            }) {
                first_record_after_an_uncommitted_transaction.get_or_insert_with(|| {
                    format!(
                        "盘 {device} 实例 {instance} 事务号 {transaction} 的记录（计数器 {member_counters:?}）一条提交标记都没有，后面计数器 {}（环槽 {}）上却还有同实例事务号 {} 的记录——没提交的事务之后这个实例不许再写任何记录",
                        later_key.counter, later.slot, later.transaction
                    )
                });
            }
        }
    }
    let violated_criteria: Vec<String> = [
        ("判据③", first_misplaced_commit_marker),
        ("提交标记字节", first_unrecognized_commit_marker),
        ("判据④", first_record_after_an_uncommitted_transaction),
    ]
    .into_iter()
    .filter_map(|(criterion, first_violation)| {
        first_violation.map(|detail| format!("{criterion}：{detail}"))
    })
    .collect();
    if !violated_criteria.is_empty() {
        judgements.judge("I-8.8", false, || violated_criteria.join("；"));
    } else if judged_transactions > 0 {
        judgements.judge("I-8.8", true, String::new);
    } else {
        judgements.not_applicable(
            "I-8.8",
            "环里没有哪一组（同一块盘上实例代号与事务号都相同的非 0 记录）落了两条以上、也没有哪一组一条提交标记都没有：③ ④ 没有判的对象，提交标记字节单独成立不算这一条成立",
        );
    }
}

/// 记录标志位 0：本次发布末条（D23（journal 的角色与格式） 已定项 17）。其余位只许 0（已定项 4）。
pub(super) const RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH: u8 = 0b0000_0001;

/// I-8.9 违例说明里的判据标签，次序照这里：红了在违例说明里逐条列出是哪几条（每条只留第一处）。
const PUBLISH_ORDINAL_CRITERIA: [&str; 7] = [
    "标志其余位",
    "序号 0",
    "跳号",
    "不从 1 起",
    "多于一条末条",
    "末条之后还有记录",
    "没有末条",
];

/// I-8.9（一次发布的记录序号连续且只有末条带标志）：同一实例、同一 checkpoint_txg 的记录（一次发布写出的全部记录），
/// 「本次发布内序号」依次是 1..N、与 jsn 同步，记录标志位 0 恰好只在序号 N 那一条上为 1，其余位为 0
/// （D23（journal 的角色与格式） 已定项 4 / 已定项 17）。逐盘判（与 I-8.6（反向链算法）、I-8.7、I-8.8 同一个口径）：
/// 一组 = 同一块盘上实例代号与 checkpoint_txg 都相同的自证过的记录，`records` 按计数器索引 ⇒ 组里按计数器升序。
/// 判据各自单独判，违例说明里写明红的是哪几条（标签见 `PUBLISH_ORDINAL_CRITERIA`）：
/// - 标志其余位：记录标志位 0 之外有位为 1；
/// - 序号 0；
/// - 跳号：组里两条记录的序号之差不等于计数器之差（与 jsn 同步：jsn 连号时序号也连号）；
/// - 不从 1 起：组里计数器最小那一条的前一个计数器上坐着一条自证过的、不属于这一组的记录 ⇒ 它是这次发布的第一条，序号要是 1；
/// - 多于一条末条：组里带末条标志的多于一条；
/// - 末条之后还有记录：带末条标志的那一条之后，这一组还有计数器更大的记录（它不是序号 N 那一条）；
/// - 没有末条：组里计数器最大那一条的下一个计数器上坐着**同一实例**、不属于这一组的记录（这个实例已经往下写了，
///   这次发布写完了），这一组却一条末条标志都没有。
///
/// **射程**：只判读得出的记录，读不出的那条不判（I-8.9 判据原句）——
/// 一、一组的末条可能还没落盘（崩在一次发布的记录之间）：计数器最大那一条的下一个计数器上没有自证过的记录、
///    或坐着别的实例的记录（崩溃之后新实例从读得出的最大号 + 1 接着写，D23（journal 的角色与格式） 已定项 14 注 3），
///    就不知道这一组写完没有，「没有末条」不判；
/// 二、组里夹着读不出的几条：「跳号」按序号之差与计数器之差比，隔着它们照样判得了；
/// 三、「不从 1 起」只在前一个计数器上有自证过的记录时判（环里第一条、前一条读不出，都判不了）。
///
/// **不适用**：环里一条自证过的记录都没有。
pub(super) fn judge_publish_ordinals_and_last_record_flags(
    records_by_device: &[(u32, JournalRecordsOfADevice)],
    judgements: &mut Judgements,
) {
    let mut first_violation_by_criterion: BTreeMap<&'static str, String> = BTreeMap::new();
    let mut note_violation = |criterion: &'static str, detail: String| {
        first_violation_by_criterion
            .entry(criterion)
            .or_insert(detail);
    };
    let mut judged_records = 0u64;
    for (device, records) in records_by_device {
        let mut records_by_publish: BTreeMap<(u32, u64), Vec<(u64, &ScannedJournalRecord)>> =
            BTreeMap::new();
        for (JournalRecordKeyOnADevice { counter, .. }, record) in records {
            judged_records += 1;
            if record.record_flags_byte & !RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0 {
                note_violation(
                    "标志其余位",
                    format!(
                        "盘 {device} 实例 {} 计数器 {counter}（环槽 {}）那条记录的记录标志是 {:#010b}——位 0 之外只许 0",
                        record.instance, record.slot, record.record_flags_byte
                    ),
                );
            }
            if record.ordinal_within_publish == 0 {
                note_violation(
                    "序号 0",
                    format!(
                        "盘 {device} 实例 {} 计数器 {counter}（环槽 {}）那条记录的本次发布内序号是 0——从 1 起",
                        record.instance, record.slot
                    ),
                );
            }
            records_by_publish
                .entry((record.instance, record.checkpoint_txg))
                .or_default()
                .push((*counter, record));
        }
        for ((instance, checkpoint_txg), members) in &records_by_publish {
            let member_counters: Vec<u64> = members.iter().map(|(counter, _)| *counter).collect();
            let member_ordinals: Vec<u32> = members
                .iter()
                .map(|(_, member)| member.ordinal_within_publish)
                .collect();
            let (first_counter, first_member) = *members
                .first()
                .expect("每一组至少一条：组是拿记录一条条追加出来的");
            let (last_counter, _) = *members.last().expect("同上");
            for (counter, member) in &members[1..] {
                let counter_distance = counter - first_counter;
                let ordinal_distance = i128::from(member.ordinal_within_publish)
                    - i128::from(first_member.ordinal_within_publish);
                if ordinal_distance != i128::from(counter_distance) {
                    note_violation(
                        "跳号",
                        format!(
                            "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布的记录（计数器 {member_counters:?}）序号是 {member_ordinals:?}——序号之差要等于计数器之差"
                        ),
                    );
                    break;
                }
            }
            // 前一个计数器上坐着的记录（哪个实例的都算）不在这一组里：这一组计数器最小的就是 first_counter，同实例同 txg 的那一条会在组里。
            let some_record_sits_before_the_first_member = first_counter
                .checked_sub(1)
                .is_some_and(|previous_counter| {
                    records_at_counter(records, previous_counter)
                        .next()
                        .is_some()
                });
            if some_record_sits_before_the_first_member && first_member.ordinal_within_publish != 1
            {
                note_violation(
                    "不从 1 起",
                    format!(
                        "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布的第一条（计数器 {first_counter}，前一个计数器上坐着别的发布的记录）序号是 {}——要从 1 起",
                        first_member.ordinal_within_publish
                    ),
                );
            }
            let flagged_counters: Vec<u64> = members
                .iter()
                .filter(|(_, member)| {
                    member.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0
                })
                .map(|(counter, _)| *counter)
                .collect();
            match flagged_counters.as_slice() {
                [] => {
                    let next_counter = counter_after(last_counter);
                    let this_instance_wrote_on = next_counter.is_some_and(|next| {
                        records.contains_key(&JournalRecordKeyOnADevice {
                            counter: next,
                            instance: *instance,
                        })
                    });
                    if this_instance_wrote_on {
                        note_violation(
                            "没有末条",
                            format!(
                                "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布的记录（计数器 {member_counters:?}）一条都不带末条标志，而同一实例在计数器 {} 上已经往下写了",
                                next_counter.expect("往下写了就有下一个计数器")
                            ),
                        );
                    }
                }
                [only_flagged] => {
                    if *only_flagged != last_counter {
                        note_violation(
                            "末条之后还有记录",
                            format!(
                                "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布带末条标志的是计数器 {only_flagged}，这一组后面还有计数器 {member_counters:?} 里更大的记录"
                            ),
                        );
                    }
                }
                [_, _, ..] => {
                    note_violation(
                        "多于一条末条",
                        format!(
                            "盘 {device} 实例 {instance} checkpoint_txg {checkpoint_txg} 那次发布带末条标志的有计数器 {flagged_counters:?} 这几条——只许一条"
                        ),
                    );
                }
            }
        }
    }
    let violated_criteria: Vec<String> = PUBLISH_ORDINAL_CRITERIA
        .into_iter()
        .filter_map(|criterion| {
            first_violation_by_criterion
                .get(criterion)
                .map(|detail| format!("{criterion}：{detail}"))
        })
        .collect();
    if !violated_criteria.is_empty() {
        judgements.judge("I-8.9", false, || violated_criteria.join("；"));
    } else if judged_records > 0 {
        judgements.judge("I-8.9", true, String::new);
    } else {
        judgements.not_applicable(
            "I-8.9",
            "环里一条自证过的记录都没有：没有哪一次发布的序号与末条标志可判",
        );
    }
}

/// 记录新根段九段的落点（D23（journal 的角色与格式） 已定项 4 / 已定项 15 的字段表：树表指针 86、映射根指针 86、树 ID 水位 8、F 8、
/// 实例表指针 86、分配记录树根指针 86、卸载记号 4、回收门槛 t 8、钉槽 p 1；末尾两段随 C773（读坏与全零根槽的回收门槛钉只在内存里） 加）。
pub(super) const NEW_ROOT_SEGMENT_TREE_TABLE_POINTER: std::ops::Range<usize> = 0..86;
pub(super) const NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 86..172;
const NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK: std::ops::Range<usize> = 172..180;
const NEW_ROOT_SEGMENT_ROLLBACK_FLOOR: std::ops::Range<usize> = 180..188;
pub(super) const NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER: std::ops::Range<usize> = 188..274;
pub(super) const NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER: std::ops::Range<usize> =
    274..360;
const NEW_ROOT_SEGMENT_UNMOUNT_MARKER: std::ops::Range<usize> = 360..364;
const NEW_ROOT_SEGMENT_RECLAIM_THRESHOLD: std::ops::Range<usize> = 364..372;
pub(super) const NEW_ROOT_SEGMENT_PINNED_RING_SLOT: std::ops::Range<usize> = 372..373;
/// 根记录里与新根段九段同名的九段（D22（单元原子性怎么合成） 已定项 7 的偏移表：flags 20（位 0 是卸载记号）、树表指针 36、
/// 树 ID 水位 122、F 130、实例表指针 170、映射根指针 256、树表 0 条那一版的分配记录树根 342、回收门槛 t 428、钉槽 p 436）。
const ROOT_RECORD_FLAGS: std::ops::Range<usize> = 20..24;
pub(super) const ROOT_RECORD_TREE_TABLE_POINTER: std::ops::Range<usize> = 36..122;
const ROOT_RECORD_TREE_IDENTIFIER_WATERMARK: std::ops::Range<usize> = 122..130;
const ROOT_RECORD_ROLLBACK_FLOOR: std::ops::Range<usize> = 130..138;
pub(super) const ROOT_RECORD_INSTANCE_TABLE_POINTER: std::ops::Range<usize> = 170..256;
pub(super) const ROOT_RECORD_MAPPING_ROOT_POINTER: std::ops::Range<usize> = 256..342;
const ROOT_RECORD_RECLAIM_THRESHOLD: std::ops::Range<usize> = 428..436;
const ROOT_RECORD_PINNED_RING_SLOT: std::ops::Range<usize> = 436..437;

/// I-8.10 逐字节比的九对：(这一段叫什么, 新根段里的落点, 根记录里的落点)。卸载记号对的是根记录 flags 整 4 字节：
/// 两边同一个编码（位 0），根记录 flags 其余位非 0 的根自证不过、进不了这里，新根段那 4 字节位 0 之外非 0 的记录也进不了这里（RR1）。
const NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT: &[(
    &str,
    std::ops::Range<usize>,
    std::ops::Range<usize>,
)] = &[
    (
        "树表指针",
        NEW_ROOT_SEGMENT_TREE_TABLE_POINTER,
        ROOT_RECORD_TREE_TABLE_POINTER,
    ),
    (
        "映射根指针",
        NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER,
        ROOT_RECORD_MAPPING_ROOT_POINTER,
    ),
    (
        "树 ID 水位",
        NEW_ROOT_SEGMENT_TREE_IDENTIFIER_WATERMARK,
        ROOT_RECORD_TREE_IDENTIFIER_WATERMARK,
    ),
    (
        "回退下界 F",
        NEW_ROOT_SEGMENT_ROLLBACK_FLOOR,
        ROOT_RECORD_ROLLBACK_FLOOR,
    ),
    (
        "实例表指针",
        NEW_ROOT_SEGMENT_INSTANCE_TABLE_POINTER,
        ROOT_RECORD_INSTANCE_TABLE_POINTER,
    ),
    (
        "分配记录树根指针",
        NEW_ROOT_SEGMENT_ALLOCATION_RECORD_TREE_ROOT_POINTER,
        ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT,
    ),
    (
        "卸载记号",
        NEW_ROOT_SEGMENT_UNMOUNT_MARKER,
        ROOT_RECORD_FLAGS,
    ),
    (
        "回收门槛 t",
        NEW_ROOT_SEGMENT_RECLAIM_THRESHOLD,
        ROOT_RECORD_RECLAIM_THRESHOLD,
    ),
    (
        "钉槽 p",
        NEW_ROOT_SEGMENT_PINNED_RING_SLOT,
        ROOT_RECORD_PINNED_RING_SLOT,
    ),
];
/// 根记录里实例代号 4 字节与 `checkpoint_txg` 8 字节的偏移（D22（单元原子性怎么合成） 已定项 7 的偏移表，与 `check_root_slot` 读的同一处）：
/// I-9.16 判红时拿它们说是哪一版根。
pub(super) const ROOT_RECORD_INSTANCE_OFFSET: usize = 24;
pub(super) const ROOT_RECORD_CHECKPOINT_TXG_OFFSET: usize = 28;
/// 根记录里「树表 0 条那一版的分配记录树的根」86 在 342（D22（单元原子性怎么合成） 已定项 7 的偏移表）。
pub(super) const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT: std::ops::Range<usize> = 342..428;

/// 一圈窗口：窗口 = txg > N − 3S（3S 是环长，根环槽数），锚 N 取镜像根环里择到的最新根的 txg。回收门槛、候选过滤、I-3.1（已分配统计对得上）
/// 丁-defer 门槛、抬 F 的上限等原来同取一圈窗口的六处 RGall 一起退回「环里最旧有效根」
/// （`research/prompts/m3-reclaim-window-r1-main-verification.md` 第六节 Q1），checker 里那两处不再用它。剩下的用处只有 I-7.9
/// （回退下界 F 不高于抬 F 的上限）在后来的镜像上重算上限时挡洞里留下的旧根（[`judge_rollback_floor_raises_against_their_ceilings`]）：
/// 镜像里 txg ≤ N − 3S 的根只能是洞那一槽留下的，比它新、抬 F 那一刻也在环里的根已被盖掉，拿它重算会把上限算低。
#[derive(Clone, Copy, Debug)]
pub(super) struct OneRevolutionWindow {
    /// 锚 N。
    pub(super) anchor_txg: u64,
    pub(super) root_ring_slot_count: u64,
}

impl OneRevolutionWindow {
    /// txg 落在窗口里：txg > N − 3S，写成 txg + 3S > N（N < 3S 时整个环都在窗口里，不减出负数）。
    pub(super) fn contains(self, checkpoint_txg: u64) -> bool {
        checkpoint_txg.saturating_add(self.root_ring_slot_count) > self.anchor_txg
    }
}

/// 由记录施加出来、根槽从没落盘的那一版：一条 journal 记录自带新根段（树表指针与映射根指针），恢复把它施加在所选根之上，
/// 这一版就成了现行版；之后写行那次发布把它换下的单元放进 defer——那些单元仍占着空间、仍算在「已分配」里，
/// 而根环里没有任何一条根引用它们（它自己的根槽一次都没写过）。
pub(super) struct VersionAppliedOnlyByRecords {
    pub(super) instance: u32,
    pub(super) checkpoint_txg: u64,
    pub(super) new_root_segment: Vec<u8>,
}

/// [`versions_applied_only_by_records`] 给一版取新根段时取的是哪一条记录的：带末条标志的那一条一到就换上它、之后不再换；
/// 在它之前只读到同一次发布别的记录时先占着先到的那一条。
pub(super) enum SegmentTakenForTheVersion {
    FromTheLastRecordOfThePublish(Vec<u8>),
    FromTheFirstRecordReadBecauseNoLastRecordWasSeen(Vec<u8>),
}

impl VersionAppliedOnlyByRecords {
    pub(super) fn tree_table_and_mapping_root_pointers(&self) -> (&[u8], &[u8]) {
        (
            &self.new_root_segment[NEW_ROOT_SEGMENT_TREE_TABLE_POINTER],
            &self.new_root_segment[NEW_ROOT_SEGMENT_MAPPING_ROOT_POINTER],
        )
    }
}

/// 回退候选集要补上的那几版：**由记录施加出来、根槽从没落盘的那一版**（增补 2 收口表第 54 行）。
/// 只看盘上这一份镜像能读出的东西，四条同时成立才算一版：
///
/// ① **它被施加过**：最新那条根指着的实例表里有这个实例的行 (i, Ti, Wi)，而这条记录的 checkpoint_txg ≤ Ti。
///    行里的 Ti 就是那次恢复停在哪一版（`crates/singlefs-core/src/mount.rs` 写行时取恢复之后的有效根的 txg），
///    施加出来的每一版的 txg 都不超过它；没有这个实例的行 ⇒ 还没有哪一次恢复把它施加出来，环里的记录只是残留
///    （崩溃点重放残留记录那条流里「链接得到残留记录」的 19 个状态就是这一格：施加要等下一次挂载，这一刻还不算）。
///    记录要带提交标记（D23（journal 的角色与格式） 已定项 7：不带的是恢复要丢的尾巴）。
/// ② **它的根槽读不出**：根环里自证过的根没有一条是 (i, 这个 txg)。读得出的那一版照常由根环那一路走。
/// ③ **不低于回退下界**：txg ≥ F 的生效值 F_生效（`rollback_floor`：调用处传的是 [`check_pool_image`] 现算的 `effective_rollback_floor`——
///    各块盘上最新的有效根所带 F 的最大值，再与池里自证过的系统配置槽带的 F 取大），与根环里的候选根同一条（D16（发布语义） 已定项 1「生效」）；
///    不是最新根自己带的那个 F。
/// ④ **它换下的单元还没到回收的时候**：根环里有一条没被实例表判抛弃的根，txg 比它小（RGall：原来只看一圈窗口里的根）。
///    理由：一个单元只被这一版引用时，它的释放代大于这一版的 txg；回收要「释放代 ≤ max(F_生效, 环里最旧有效根)」
///    （D3（空间分配） 已定项 7）。环里最旧的有效根比这一版老 ⇒ 最旧有效根 < 这一版 < 释放代，③ 又让 F ≤ 这一版 ⇒ 回收不了，
///    它们还算在已分配里；反过来环里最旧的有效根已经比这一版新，这一版里只有它引用的单元释放代都不超过那条根，已可回收复用，
///    再把它并进来就是拿已经不归它的槽去比。
///
/// 一版由那条记录的 (实例代号, checkpoint_txg) 认；两块盘各一份镜像记录，取先读到的那一份。
/// 一次发布切成几条记录时，新根段取带末条标志的那一条的（改法 FL
/// 第二节 P1-b 与第三节第 1 条）：core 的恢复走到带末条标志的那一条才整次施加，施加的是那一条的新根段
/// （`crates/singlefs-core/src/recovery.rs` 拼新根那一段「施加末条那一份」）。合法写者每条记录都带同一份新根段（D23（journal 的角色与格式）
/// 已定项 15），两种取法同一份字节；坏写者把前几条写成别的根时，取先到的那条就是拿 core 没施加的那一版去判。
/// 环里读不到带末条标志的那一条时退回先到的那一条（这一版仍被实例表的行判为施加过，只是末条这一刻读不出）。
pub(super) fn versions_applied_only_by_records(
    records_the_recovery_reader_reads_by_device: &[(u32, JournalRecordsOfADevice)],
    roots: &[(u64, u64, crate::RootView)],
    instance_table_rows: &[InstanceTableRow],
    rollback_floor: u64,
) -> Vec<VersionAppliedOnlyByRecords> {
    let is_abandoned = |instance: u32, checkpoint_txg: u64| {
        instance_table_rows
            .iter()
            .any(|row| row.instance == instance && checkpoint_txg > row.published_checkpoint_txg)
    };
    let Some(oldest_valid_root_txg) = roots
        .iter()
        .filter(|(_, _, root)| !is_abandoned(root.instance, root.checkpoint_txg))
        .map(|(_, _, root)| root.checkpoint_txg)
        .min()
    else {
        return Vec::new();
    };
    let mut versions: BTreeMap<(u32, u64), SegmentTakenForTheVersion> = BTreeMap::new();
    // 按恢复的读法（每个计数器取第一块盘上的那一份）：反向链对不上的记录不进重放前缀（I-8.6（反向链算法）），不算被哪一次恢复施加过。
    let records_as_the_recovery_reads_them =
        records_as_the_recovery_reads_them(records_the_recovery_reader_reads_by_device);
    for ((_, counter), record) in &records_as_the_recovery_reads_them {
        let applied_by_a_recovery = record.commit_marker == CommitMarker::Present
            && record_can_enter_a_replay_prefix(
                &records_as_the_recovery_reads_them,
                *counter,
                record,
            )
            && instance_table_rows.iter().any(|row| {
                row.instance == record.instance
                    && record.checkpoint_txg <= row.published_checkpoint_txg
            });
        let root_slot_is_readable = roots.iter().any(|(_, _, root)| {
            root.instance == record.instance && root.checkpoint_txg == record.checkpoint_txg
        });
        let at_or_above_the_floor = record.checkpoint_txg >= rollback_floor;
        let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;
        if applied_by_a_recovery
            && !root_slot_is_readable
            && at_or_above_the_floor
            && its_units_are_not_reclaimable_yet
        {
            let carries_the_last_record_flag =
                record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0;
            let taken_so_far = versions.get(&(record.instance, record.checkpoint_txg));
            let takes_this_record = match taken_so_far {
                None => true,
                Some(SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(_)) => false,
                Some(
                    SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(_),
                ) => carries_the_last_record_flag,
            };
            if takes_this_record {
                let new_root_segment = record.new_root_segment.clone();
                versions.insert(
                    (record.instance, record.checkpoint_txg),
                    if carries_the_last_record_flag {
                        SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(new_root_segment)
                    } else {
                        SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(
                            new_root_segment,
                        )
                    },
                );
            }
        }
    }
    versions
        .into_iter()
        .map(
            |((instance, checkpoint_txg), taken)| VersionAppliedOnlyByRecords {
                instance,
                checkpoint_txg,
                new_root_segment: match taken {
                    SegmentTakenForTheVersion::FromTheLastRecordOfThePublish(new_root_segment)
                    | SegmentTakenForTheVersion::FromTheFirstRecordReadBecauseNoLastRecordWasSeen(
                        new_root_segment,
                    ) => new_root_segment,
                },
            },
        )
        .collect()
}

/// I-7.4（近 K 代块未被复用） 被抛弃根那一半（代码审阅 6c 第 7 条）：
/// I-7.4 那一行「被抛弃时间线的根……引用的块在离开根环之前同样不许重新分配、不许抹头」（C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 取影子账）。
/// 根环里每条被最新根指着的实例表判抛弃的根（有行 (i, Ti, ·) 且 T > Ti，与回退候选集同一个判法）单独走一遍：用一份自己的走读，
/// 判定丢掉、引用不进 I-3.1（已分配统计对得上） 的并集（被抛弃的根引用的单元由影子账隔离、不在当前账里），只看它引用的块读回来对不对——
/// 有一个单元校验和对不上或头用不了，就是它指着的块被重新分配或抹头了，I-7.4 按这条根判一格红，说明里带着读回来对不上的是哪些。
/// 今天崩溃恢复抛弃时间线、那次挂载看不见被抛弃的根而复用了它的槽（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）） 那一形在这里照实红。
/// 根环有读不出的槽、最新根的实例表里又有行（有根可以被判抛弃）时，那个槽里可能坐着一条被抛弃的根：这一半判不了，
/// I-7.4 没有违例时整条报「不适用」、不报成立。
pub(super) fn judge_blocks_referenced_by_abandoned_roots(
    reader: &dyn ImageReader,
    geometry: &PoolGeometry,
    roots: &[(u64, u64, crate::RootView)],
    newest_index: usize,
    instance_table_rows: &[InstanceTableRow],
    filesystem_identifier_low: u64,
    judgements: &mut Judgements,
) {
    if instance_table_rows.is_empty() {
        return;
    }
    let root_slot_bytes = usize::try_from(geometry.physical_block_size).expect("根槽宽");
    let some_root_slot_is_unreadable = root_slot_positions(geometry)
        .into_iter()
        .any(|(_, _, device, offset)| reader.read(device, offset, root_slot_bytes).is_none());
    if some_root_slot_is_unreadable {
        judgements.part_of_the_range_left_unjudged(
            "I-7.4",
            "根环有读不出的槽、最新根的实例表里又有行：那个槽里可能坐着一条被抛弃的根，它引用的块有没有被重新分配或抹头没判".to_string(),
        );
    }
    let newest_root = &roots[newest_index].2;
    for (index, (_, _, root)) in roots.iter().enumerate() {
        let abandoned = instance_table_rows.iter().any(|row| {
            row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg
        });
        if index == newest_index || !abandoned {
            continue;
        }
        let mut walk_of_the_abandoned_root = Walk::starting_with(
            reader,
            Judgements::default(),
            filesystem_identifier_low,
            newest_root.instance,
            newest_root.checkpoint_txg,
        );
        walk_of_the_abandoned_root.walk_root(&root.record_bytes, false);
        let first_copy_that_does_not_match = walk_of_the_abandoned_root
            .judgements
            .first_violation_of("I-2.1")
            .map(str::to_string);
        let walk_failures = walk_of_the_abandoned_root.walk_failures;
        let walked_into_a_reallocated_or_erased_unit =
            first_copy_that_does_not_match.is_some() || !walk_failures.is_empty();
        let (instance, root_txg) = (root.instance, root.checkpoint_txg);
        // 树表指针整条全零的根：走读照 core 的根记录读者读盘 0 槽 0、读不出（`follow_an_all_zero_tree_table_pointer_like_the_root_record_reader_of_the_core`），
        // 红的是这条根自己坏了，说明不写成「引用的单元已被重新分配或抹头」
        // 判定不变。
        let tree_table_pointer_is_all_zero =
            parse_node_pointer(&root.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER]).all_zero;
        judgements.judge("I-7.4", !walked_into_a_reallocated_or_erased_unit, || {
            let read_back_wrong: Vec<String> = first_copy_that_does_not_match
                .into_iter()
                .chain(walk_failures)
                .collect();
            if tree_table_pointer_is_all_zero {
                format!(
                    "被抛弃的根（实例 {instance}、txg {root_txg}）走不完：根记录里树表指针全零，core 的根记录读者照它读盘 0 槽 0、读不出（这条根本身损坏，不只是它引用的单元被重新分配或抹头）：{}",
                    read_back_wrong.join("；")
                )
            } else {
                format!(
                    "被抛弃的根（实例 {instance}、txg {root_txg}）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：{}",
                    read_back_wrong.join("；")
                )
            }
        });
    }
}

/// I-8.10（记录新根段与同代根相同）：逐盘，按 (实例代号, `checkpoint_txg`) 分组，组里带末条标志（记录标志位 0）的恰一条时，
/// 它的新根段九段与根环里同 (实例代号, `checkpoint_txg`) 的每条自证过的根的同名九段逐字节相同（D23（journal 的角色与格式） 已定项 15；
/// 从四段扩到七段，C773 加回收门槛 t 与钉槽 p 两段）：新根段 [0..86) = 根 [36..122)、[86..172) = [256..342)、
/// [172..180) = [122..130)、[180..188) = [130..138)、[188..274) = [170..256)、[274..360) = [342..428)、[360..364) = flags [20..24)、
/// [364..372) = [428..436)、[372..373) = [436..437)（[`NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT`]）。
/// 末条标志不唯一的组归 I-8.9（一次发布的记录序号连续且只有末条带标志），不判；同 (实例, txg) 有两条以上不同内容的根不另分，逐条比。
/// 游标之后那一个计数器：计数器是盘上读来的 u64，到顶（`u64::MAX`）就没有「下一个」——不回绕成 0 去把整环当成「后面的记录」。
fn counter_after(last_counter: u64) -> Option<u64> {
    last_counter.checked_add(1)
}

/// I-8.10 两半：同一次发布的每条记录新根段相同（往非末条扩，C738（同一次发布几条记录的新根段一致没有不变量判）），与带末条标志那一条的新根段
/// 等于同代根（判法见上面 [`counter_after`] 之前那一段说明）。比到一对就算这一条有对象；一对都没比到报不适用。
pub(super) fn judge_new_root_segment_of_the_last_record_against_the_roots(
    records_by_device: &[(u32, JournalRecordsOfADevice)],
    roots: &[(u64, u64, crate::RootView)],
    judgements: &mut Judgements,
) {
    let mut pairs_compared: u64 = 0;
    for (device, records) in records_by_device {
        // 往非末条扩的那一半（C738（同一次发布几条记录的新根段一致没有不变量判），去向「I-8.10 往非末条扩」；D23（journal 的角色与格式）
        // 已定项 15 末句「一次发布切成几条记录时每条记录都带同一份新根段」）：同一块盘上同 (实例, txg) 的每条自证过的记录，新根段与这一组
        // 计数器最小的那一条逐字节相同。只认末条的那一判（下面）比不到非末条：写者把非末条的新根段写成上一版时 checker 与 core 都不看它。
        let mut records_of_each_publish: BTreeMap<(u32, u64), Vec<u64>> = BTreeMap::new();
        for (JournalRecordKeyOnADevice { counter, .. }, record) in records {
            records_of_each_publish
                .entry((record.instance, record.checkpoint_txg))
                .or_default()
                .push(*counter);
        }
        for ((instance, checkpoint_txg), counters) in &records_of_each_publish {
            let Some((first_counter, other_counters)) = counters.split_first() else {
                continue;
            };
            let first_segment = &records[&JournalRecordKeyOnADevice {
                counter: *first_counter,
                instance: *instance,
            }]
                .new_root_segment;
            for counter in other_counters {
                pairs_compared += 1;
                let segment = &records[&JournalRecordKeyOnADevice {
                    counter: *counter,
                    instance: *instance,
                }]
                    .new_root_segment;
                let segments_are_byte_identical = segment == first_segment;
                judgements.judge(
                    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
                    segments_are_byte_identical,
                    || {
                        let first_differing_offset = segment
                            .iter()
                            .zip(first_segment.iter())
                            .position(|(byte, byte_of_the_first)| byte != byte_of_the_first)
                            .unwrap_or(segment.len().min(first_segment.len()));
                        format!(
                            "盘 {device} 同一次发布（实例 {instance}、txg {checkpoint_txg}）的两条记录新根段不逐字节相同：计数器 {counter} 那一条与计数器 {first_counter} 那一条从段内偏移 {first_differing_offset} 起不同"
                        )
                    },
                );
            }
        }
        let mut last_records_of_each_publish: BTreeMap<(u32, u64), Vec<u64>> = BTreeMap::new();
        for (JournalRecordKeyOnADevice { counter, .. }, record) in records {
            if record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0 {
                last_records_of_each_publish
                    .entry((record.instance, record.checkpoint_txg))
                    .or_default()
                    .push(*counter);
            }
        }
        for ((instance, checkpoint_txg), counters) in last_records_of_each_publish {
            let [counter] = counters.as_slice() else {
                continue;
            };
            let segment = &records[&JournalRecordKeyOnADevice {
                counter: *counter,
                instance,
            }]
                .new_root_segment;
            for (region, slot, root) in roots {
                if (root.instance, root.checkpoint_txg) != (instance, checkpoint_txg) {
                    continue;
                }
                pairs_compared += 1;
                let root_bytes = &root.record_bytes;
                let differing_fields: Vec<&str> = NEW_ROOT_SEGMENT_FIELDS_COMPARED_WITH_THE_ROOT
                    .iter()
                    .filter(|(_, in_the_segment, in_the_root)| {
                        segment[in_the_segment.clone()] != root_bytes[in_the_root.clone()]
                    })
                    .map(|(name, _, _)| *name)
                    .collect();
                judgements.judge(
                    LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
                    differing_fields.is_empty(),
                    || {
                        format!(
                            "盘 {device} journal 计数器 {counter} 那条带末条标志的记录（实例 {instance}、txg {checkpoint_txg}）的新根段与区域 {region} 槽 {slot} 那条同代根这几段不逐字节相同：{}",
                            differing_fields.join("、")
                        )
                    },
                );
            }
        }
    }
    if pairs_compared == 0 {
        judgements.not_applicable(
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
            "环里没有哪条带末条标志、同 (实例, txg) 只此一条的记录在根环里有同代的自证根，也没有哪次发布在同一块盘上落了两条以上的记录：新根段没有可比的根与记录",
        );
    }
}

/// I-8.11（点名项与被指单元头相同）：环里每条自证过的记录（逐盘）的每个点名项的每条位置条目，按类读那一份（码 2 一个节点，码 1 / 码 3
/// 一个数据单元宽；类标签不在登记表里的不知道多长，不读），整单元校验和与位置条目里的相符时，单元头里的类标签、诞生代号、写序实例代号与尾段
/// （码 1 事务号 6 字节；码 2 / 码 3 出生序号 4 + 补零 2，按 6 字节整数比同时判了补零恒 0）与点名项逐字段相同；出生树只判码 1、码 3
/// （头偏移 43），码 2 随 C289（码 2 的出生树取哪个字段没写） 不判。校验和不相符的不判：落点被合法复用之后，那里就不是它点名的单元了。
pub(super) fn judge_named_entries_against_the_unit_headers(
    reader: &dyn ImageReader,
    records_by_device: &[(u32, JournalRecordsOfADevice)],
    judgements: &mut Judgements,
) {
    let mut locations_compared: u64 = 0;
    for (device, records) in records_by_device {
        for (JournalRecordKeyOnADevice { counter, .. }, record) in records {
            for (index, named) in record.named_entries.iter().enumerate() {
                let unit_bytes = match MappingKeyUnitClass::of(named.unit_class) {
                    MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
                        data_unit_bytes()
                    }
                    MappingKeyUnitClass::IndexNode => node_bytes(),
                    MappingKeyUnitClass::Unregistered(_) => continue,
                };
                for (location_device, location_slot, checksum) in named.locations {
                    let Some(unit) =
                        reader.read(location_device, location_slot * SLOT_BYTES, unit_bytes)
                    else {
                        continue;
                    };
                    if crc32_castagnoli_table(&unit) != checksum {
                        continue;
                    }
                    locations_compared += 1;
                    let named_instance = read_u32(&named.key_tail, 0);
                    let named_tail = read_six_byte_unsigned(&named.key_tail, 4);
                    let identity_matches =
                        birth_identity_in_the_header(&unit).is_some_and(|identity| {
                            identity.write_order.birth_txg() == named.birth_txg
                                && identity.write_order.instance() == named_instance
                                && identity.tail.birth_identity_tail_number() == named_tail
                        });
                    let birth_tree_matches =
                        match birth_tree_in_the_header_as_the_mapping_key_names_it(&unit) {
                            Some(BirthTreeInTheHeader::AtOffsetFortyThree(birth_tree)) => {
                                birth_tree == named.birth_tree
                            }
                            Some(BirthTreeInTheHeader::FieldStillOpenUnderC289) | None => true,
                        };
                    let holds =
                        unit[6] == named.unit_class && identity_matches && birth_tree_matches;
                    judgements.judge(NAMED_ENTRY_MATCHES_THE_UNIT_HEADER, holds, || {
                        format!(
                            "盘 {device} journal 计数器 {counter} 那条记录第 {index} 个点名项（类 {}、出生树 {}、诞生代号 {}、实例 {named_instance}、尾段 {named_tail}）与盘 {location_device} 槽 {location_slot} 那一份单元头（类 {}）不符",
                            named.unit_class, named.birth_tree, named.birth_txg, unit[6]
                        )
                    });
                }
            }
        }
    }
    if locations_compared == 0 {
        judgements.not_applicable(
            NAMED_ENTRY_MATCHES_THE_UNIT_HEADER,
            "环里没有哪个点名项的位置条目指着一份整单元校验和与它相符的单元：点名项没有可比的单元头",
        );
    }
}

/// I-8.12（同实例记录的 txg 不减）：逐盘，同一实例代号的自证记录按 jsn 计数器从小到大，`checkpoint_txg` 不减；
/// 只比同实例的前后两条，中间夹着别的实例的记录不管；换实例之后严格大于不判（设备失而复得下没核）。
pub(super) fn judge_journal_record_txg_per_instance(
    records_by_device: &[(u32, JournalRecordsOfADevice)],
    judgements: &mut Judgements,
) {
    for (device, records) in records_by_device {
        let mut previous_record_of_each_instance: BTreeMap<u32, (u64, u64)> = BTreeMap::new();
        for (JournalRecordKeyOnADevice { counter, .. }, record) in records {
            if let Some((previous_counter, previous_txg)) = previous_record_of_each_instance
                .insert(record.instance, (*counter, record.checkpoint_txg))
            {
                judgements.judge(
                    JOURNAL_RECORD_TXG_NOT_DECREASING_WITHIN_AN_INSTANCE,
                    record.checkpoint_txg >= previous_txg,
                    || {
                        format!(
                            "盘 {device} 实例 {} 的 journal 记录：计数器 {previous_counter} 的 txg {previous_txg} 之后，计数器 {counter} 的 txg {} 变小了",
                            record.instance, record.checkpoint_txg
                        )
                    },
                );
            }
        }
    }
}

/// I-3.12（实例表行 T 低于引用它的根的 txg）：根环里每条自证过的根，它自己指着的实例表（沿链读，只读不判）每一行 (i, T, W) 满足
/// T < 这条根的 `checkpoint_txg`（D18（块里携带什么信息） 已定项 11、D23（journal 的角色与格式） 已定项 14）。链上某一片读不出的根不判；
/// 一行都没判到时整条报不适用。
pub(super) fn judge_instance_table_rows_against_the_roots_referencing_them(
    reader: &dyn ImageReader,
    roots: &[(u64, u64, crate::RootView)],
    judgements: &mut Judgements,
) {
    let mut rows_judged: u64 = 0;
    for (_, _, root) in roots {
        let Some(rows) = instance_table_rows_of_root_without_judging(reader, &root.record_bytes)
        else {
            continue;
        };
        for row in rows {
            rows_judged += 1;
            judgements.judge(
                INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
                row.published_checkpoint_txg < root.checkpoint_txg,
                || {
                    format!(
                        "实例 {}、txg {} 的根指着的实例表里，实例 {} 那一行的 T {} 不小于这条根的 txg",
                        root.instance,
                        root.checkpoint_txg,
                        row.instance,
                        row.published_checkpoint_txg
                    )
                },
            );
        }
    }
    if rows_judged == 0 {
        judgements.not_applicable(
            INSTANCE_TABLE_ROWS_PUBLISHED_BEFORE_THE_ROOT_REFERENCING_THEM,
            "根环里没有哪条根指着一张读得出、有行的实例表",
        );
    }
}

/// [`JOURNAL_RING_GEOMETRY_FIELDS_AGREE`]（I-8.1）读 journal 环的那一格：逐盘，环里同一实例、同一 `checkpoint_txg` 的自证记录
/// （同一次发布）不多于系统配置里记的在飞记录数上限条，每个 (盘, 实例, txg) 判一格。上限取池里第一块择到系统配置的盘那一份
/// （各盘的几何本来是同一份，`check_pool_image` 走根环也只取它）。环里一条记录都没有时这一格没有对象，三个字段等式那一半照判。
pub(super) fn judge_records_of_one_publish_within_the_in_flight_record_limit(
    records_by_device: &[(u32, JournalRecordsOfADevice)],
    declared_in_flight_record_limit: u64,
    judgements: &mut Judgements,
) {
    for (device, records) in records_by_device {
        let mut records_per_publish: BTreeMap<(u32, u64), u64> = BTreeMap::new();
        for record in records.values() {
            *records_per_publish
                .entry((record.instance, record.checkpoint_txg))
                .or_insert(0) += 1;
        }
        for ((instance, checkpoint_txg), record_count) in records_per_publish {
            judgements.judge(
                JOURNAL_RING_GEOMETRY_FIELDS_AGREE,
                record_count <= declared_in_flight_record_limit,
                || {
                    format!(
                        "盘 {device} journal 环里实例 {instance}、txg {checkpoint_txg} 的自证记录 {record_count} 条，多于系统配置记的在飞上限 {declared_in_flight_record_limit}"
                    )
                },
            );
        }
    }
}

/// 恢复照 D23（journal 的角色与格式） 已定项 14 取重放前缀、施加之后的结局，checker 自己复刻一份（R-独，`research/prompts/m3-inv-a-r1-main-verification.md`
/// 第二节第 2 行）：只读，不与实现共用代码（D13（验证路线） 已定项 5）。
pub(super) enum ReplayTheNextMountWouldDo<'records> {
    /// 所选根那次发布读得出的记录里带末条标志的多于一条：锚点认哪一条条款没写，实现在任何写之前拒挂（已定项 14「这一版的失败处置」）。
    AnchorUndecided,
    /// 所选根的 `checkpoint_txg` 已是 u64 的最大值，接在它后面的发布取不到 txg：实现报损坏、一条都不施加。
    ChosenRootTxgHasNoSuccessor,
    /// 前缀停在一条点名单元两份没都验过的记录上（第四条）：起点含介质损坏。
    /// 停之前已经整次收下的发布 core 照施加（`crates/singlefs-core/src/recovery.rs` 的 `replay_journal`：前缀停在验不过的那一条之前，
    /// 施加到它之前最后一个末条为止）；`last_record_of_the_first_publish_applied_before_it` 是其中第一次发布的末条，一次都没收下时是 `None`。
    StoppedAtANamedUnitThatDoesNotVerify {
        last_record_of_the_first_publish_applied_before_it: Option<&'records ScannedJournalRecord>,
    },
    /// 一次发布都没施加：所选根那一版就是下一次挂载择到的那一版，它走不走得完归 I-7.2（最新根可遍历）。
    NothingApplied,
    /// 施加了 `publishes` 次发布、共 `records` 条记录；`last_record` 是最后一次发布的末条，施加出来的那一版照它的新根段
    /// （已定项 15：一次发布的每条记录带同一份新根段）。`last_record_of_the_first_publish` 是第一次发布的末条：
    /// 下一次挂载先施加的那一版照它的新根段（[`version_the_next_mount_applies_first`]，I-3.10 与 I-5.4 读它）。
    Applied {
        publishes: u64,
        records: u64,
        last_record_counter: u64,
        last_record: &'records ScannedJournalRecord,
        last_record_of_the_first_publish: &'records ScannedJournalRecord,
    },
}

/// 重放的在飞上限：按环长与格式常量 F 现算（D23（journal 的角色与格式） 已定项 18），I-4.1 与下一次挂载先施加的那一版用同一个数。
pub(super) fn in_flight_record_limit_of_the_journal_ring(journal_ring_bytes: u64) -> u64 {
    journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR
}

/// 重放前缀的反向链一关（I-8.6（反向链算法）「不等的记录不进重放前缀」）：本实例内逻辑前一条 = 同一实例、计数器小 1 的那一条，
/// 在 [`records_as_the_recovery_reads_them`] 里按 (实例代号, 计数器) 找；在、而它的头算出来的链值与这一条的反向链不等，这一条进不了前缀；
/// 不在不判（与实现 `judge_back_chain_against_the_previous_record_on_disk` 按 (实例, 计数器 − 1) 找、找不到不判同一个读法）。
/// 计数器 0 进不了前缀（计数器从 1 起，D23（journal 的角色与格式） 已定项 18）。
fn record_back_chain_lets_it_into_the_replay_prefix(
    records_the_replay_reads: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    counter: u64,
    record: &ScannedJournalRecord,
) -> bool {
    let Some(previous_counter) = counter.checked_sub(1) else {
        return false;
    };
    records_the_replay_reads
        .get(&(record.instance, previous_counter))
        .is_none_or(|previous| record.back_chain == previous.chain_value_the_next_record_must_carry)
}

/// 一条记录的点名单元是不是每一个都两份都读得出、整单元校验和都对（前缀第四条「一个点名单元的两份都读得出、校验和都对才施加」）。
/// 类标签不在登记表里的点名项不知道该读多长：当验不过。
fn named_units_of_the_record_verify_on_both_copies(
    reader: &dyn ImageReader,
    record: &ScannedJournalRecord,
) -> bool {
    record.named_entries.iter().all(|named| {
        let unit_bytes = match MappingKeyUnitClass::of(named.unit_class) {
            MappingKeyUnitClass::DataUnit | MappingKeyUnitClass::PackedRecordUnit => {
                data_unit_bytes()
            }
            MappingKeyUnitClass::IndexNode => node_bytes(),
            MappingKeyUnitClass::Unregistered(_) => return false,
        };
        named
            .locations
            .iter()
            .all(|(location_device, location_slot, checksum)| {
                reader
                    .read(*location_device, location_slot * SLOT_BYTES, unit_bytes)
                    .is_some_and(|unit| crc32_castagnoli_table(&unit) == *checksum)
            })
    })
}

/// 带末条标志的这一条之后，同一实例里计数器更大、读得出的记录有没有同一个 `checkpoint_txg` 的（已定项 4 读者规则「带标志的那条之后还有记录」：
/// 按计数器往后找，同实例里先遇到 txg 更大的就停——同一实例 txg 随计数器不减时与全表读法同一个结果）。
fn readable_record_of_the_same_publish_follows(
    records_the_replay_reads: &BTreeMap<(u32, u64), &ScannedJournalRecord>,
    counter: u64,
    record: &ScannedJournalRecord,
) -> bool {
    for later in records_the_replay_reads
        .range((record.instance, counter.saturating_add(1))..=(record.instance, u64::MAX))
        .map(|(_, later)| *later)
    {
        if later.checkpoint_txg > record.checkpoint_txg {
            return false;
        }
        if later.checkpoint_txg == record.checkpoint_txg {
            return true;
        }
    }
    false
}

/// R-独：照 D23（journal 的角色与格式） 已定项 14 的全部规则复刻下一次挂载会施加哪一段记录。
///
/// - **锚点**（注 1，读法乙）：所选根同实例、同 `checkpoint_txg`、带末条标志的记录。恰一条 ⇒ 链首是它的计数器 + 1；
///   零条 ⇒ 链首取水位之上第一条可读记录，它要序号 1、`checkpoint_txg` = 所选根 + 1；多于一条 ⇒ 锚点认不定。
/// - 只看「水位之上」的记录：同实例、`checkpoint_txg` 大于所选根（注 1「前缀规则不跨实例边界」，注 2 按实例为主比）。
///   在飞上限按环长与格式常量 F 现算（已定项 18），逐条数到上限就停。
/// - 逐条：计数器接得上（断号即止）、反向链对得上（I-8.6 的前缀口径，[`record_back_chain_lets_it_into_the_replay_prefix`]）、一次发布的首条序号是 1、
///   发布之内 txg 不变且序号逐一递增、前一条不带提交标记时事务号不变（第三条）、末条带提交标记、带末条标志的那条之后同一次发布没有可读记录；
///   这些有一样不成就停在这一条之前。
/// - 点名单元两份都验过才收（第四条）；验不过就停，结局是「起点含介质损坏」。
/// - 第五条：前缀按发布边界截，末条没到的那次发布整体不施加。
pub(super) fn replay_the_next_mount_would_do<'records>(
    reader: &dyn ImageReader,
    records_the_replay_reads: &BTreeMap<(u32, u64), &'records ScannedJournalRecord>,
    chosen_root: &crate::RootView,
    in_flight_record_limit: u64,
) -> ReplayTheNextMountWouldDo<'records> {
    let anchors: Vec<u64> = records_the_replay_reads
        .iter()
        .filter(|(_, record)| {
            record.instance == chosen_root.instance
                && record.checkpoint_txg == chosen_root.checkpoint_txg
                && record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0
        })
        .map(|((_, counter), _)| *counter)
        .collect();
    let mut expected_next_counter = match anchors.as_slice() {
        [] => None,
        [anchor] => Some(
            anchor
                .checked_add(1)
                .expect("计数器是记录头里 6 字节读来的，加一不会越过 u64"),
        ),
        [_, _, ..] => return ReplayTheNextMountWouldDo::AnchorUndecided,
    };
    let Some(chain_start_txg_without_anchor) = chosen_root.checkpoint_txg.checked_add(1) else {
        return ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor;
    };
    let mut open_publish: Vec<&ScannedJournalRecord> = Vec::new();
    let mut publishes_applied: u64 = 0;
    let mut records_applied: u64 = 0;
    let mut last_applied: Option<(u64, &ScannedJournalRecord)> = None;
    let mut last_record_of_the_first_publish_applied: Option<&ScannedJournalRecord> = None;
    let above_the_chosen_root = records_the_replay_reads.iter().filter(|(_, record)| {
        record.instance == chosen_root.instance
            && record.checkpoint_txg > chosen_root.checkpoint_txg
    });
    for (records_taken_before_this_one, ((_, counter), record)) in
        (0_u64..).zip(above_the_chosen_root)
    {
        let (counter, record) = (*counter, *record);
        if records_taken_before_this_one >= in_flight_record_limit {
            break;
        }
        let continues_the_chain = match expected_next_counter {
            Some(expected) => counter == expected,
            None => {
                record.checkpoint_txg == chain_start_txg_without_anchor
                    && record.ordinal_within_publish == 1
            }
        };
        if !continues_the_chain
            || !record_back_chain_lets_it_into_the_replay_prefix(
                records_the_replay_reads,
                counter,
                record,
            )
        {
            break;
        }
        let fits_the_open_publish = match open_publish.last() {
            None => record.ordinal_within_publish == 1,
            Some(previous) => {
                record.checkpoint_txg == previous.checkpoint_txg
                    && u64::from(record.ordinal_within_publish)
                        == u64::from(previous.ordinal_within_publish) + 1
                    && (previous.commit_marker == CommitMarker::Present
                        || record.transaction == previous.transaction)
            }
        };
        if !fits_the_open_publish {
            break;
        }
        let ends_its_publish =
            record.record_flags_byte & RECORD_FLAG_LAST_RECORD_OF_THE_PUBLISH != 0;
        if ends_its_publish
            && (record.commit_marker != CommitMarker::Present
                || readable_record_of_the_same_publish_follows(
                    records_the_replay_reads,
                    counter,
                    record,
                ))
        {
            break;
        }
        if !named_units_of_the_record_verify_on_both_copies(reader, record) {
            return ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {
                last_record_of_the_first_publish_applied_before_it:
                    last_record_of_the_first_publish_applied,
            };
        }
        expected_next_counter = Some(
            counter
                .checked_add(1)
                .expect("计数器是记录头里 6 字节读来的，加一不会越过 u64"),
        );
        open_publish.push(record);
        if ends_its_publish {
            publishes_applied += 1;
            records_applied += u64::try_from(open_publish.len()).expect("一次发布的记录条数");
            // 施加出来的那一版照末条那一份新根段（已定项 15），第一次发布的那一版同样取它的末条（改法 FL 的那一取法）。
            last_record_of_the_first_publish_applied.get_or_insert(record);
            open_publish.clear();
            last_applied = Some((counter, record));
        }
    }
    match last_applied {
        None => ReplayTheNextMountWouldDo::NothingApplied,
        Some((last_record_counter, last_record)) => ReplayTheNextMountWouldDo::Applied {
            publishes: publishes_applied,
            records: records_applied,
            last_record_counter,
            last_record,
            last_record_of_the_first_publish: last_record_of_the_first_publish_applied.expect(
                "一次发布收到末条时 last_applied 与它在同一处写进 Some、之后都不回 None：last_applied 是 Some，它就是 Some",
            ),
        },
    }
}

/// [`TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE`]（I-4.1）：所选根（重放之后那一版）指向的树全可遍历。
/// 所选根取根环里 (txg, 实例) 最大的那条（与 I-7.2 同一个择法）；重放照 [`replay_the_next_mount_would_do`]；施加出来的那一版 =
/// 所选根换上末条记录的新根段七段（实例表指针与树表 0 条那一版的分配记录树的根也从新根段来，已定项 15，与实现恢复拼新根同一个读法），
/// 用一份另起的走读走一遍（判定丢掉，那些归别的不变量），走读断了（任一单元两份都读不对、头用不了、条目解不开）判红。
/// 一条都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验不过（起点含介质损坏）都报不适用。
pub(super) fn judge_trees_of_the_replayed_version_are_traversable(
    reader: &dyn ImageReader,
    records_the_recovery_reader_reads_by_device: &[(u32, JournalRecordsOfADevice)],
    chosen_root: &crate::RootView,
    journal_ring_bytes: u64,
    filesystem_identifier_low: u64,
    judgements: &mut Judgements,
) {
    let records_the_replay_reads =
        records_as_the_recovery_reads_them(records_the_recovery_reader_reads_by_device);
    match replay_the_next_mount_would_do(
        reader,
        &records_the_replay_reads,
        chosen_root,
        in_flight_record_limit_of_the_journal_ring(journal_ring_bytes),
    ) {
        ReplayTheNextMountWouldDo::AnchorUndecided => judgements.not_applicable(
            TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
            "所选根那次发布读得出的记录里带末条标志的多于一条：锚点认哪一条条款没写，实现在任何写之前拒挂，重放不出一版",
        ),
        ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor => judgements.not_applicable(
            TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
            "所选根的 checkpoint_txg 是 u64 的最大值：接在它后面的发布取不到 txg，重放不出一版",
        ),
        ReplayTheNextMountWouldDo::StoppedAtANamedUnitThatDoesNotVerify {
            last_record_of_the_first_publish_applied_before_it: _,
        } => {
            judgements.not_applicable(
                TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
                "重放前缀停在一条点名单元两份没都验过的记录上：起点含介质损坏，不在 I-4.1 判的合法镜像里",
            );
        }
        ReplayTheNextMountWouldDo::NothingApplied => judgements.not_applicable(
            TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
            "所选根之上一次发布都不施加：重放之后那一版就是所选根，它可不可遍历归 I-7.2",
        ),
        ReplayTheNextMountWouldDo::Applied {
            publishes,
            records,
            last_record_counter,
            last_record,
            last_record_of_the_first_publish: _,
        } => {
            let replayed_version = VersionAppliedOnlyByRecords {
                instance: last_record.instance,
                checkpoint_txg: last_record.checkpoint_txg,
                new_root_segment: last_record.new_root_segment.clone(),
            };
            let mut walk_of_the_replayed_version = Walk::starting_with(
                reader,
                Judgements::default(),
                filesystem_identifier_low,
                chosen_root.instance,
                chosen_root.checkpoint_txg,
            );
            walk_of_the_replayed_version.walk_version_applied_only_by_records(&replayed_version);
            let walk_failures = walk_of_the_replayed_version.walk_failures;
            judgements.judge(
                TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
                walk_failures.is_empty(),
                || {
                    format!(
                        "所选根（实例 {}、txg {}）之上重放 {publishes} 次发布、{records} 条记录（末条计数器 {last_record_counter}）施加出的那一版走不完：{}",
                        chosen_root.instance,
                        chosen_root.checkpoint_txg,
                        walk_failures.join("；")
                    )
                },
            );
        }
    }
}

/// 走读记下的被引用落点（(设备, 起点槽, 跨度) → 第一次引用它的是谁）逐盘加起来是多少槽。
/// I-3.1（已分配统计对得上） 拿走完全部候选版本之后那一份算，I-3.11（已分配减 defer 等于最新根走读） 拿只走过最新根那一刻的那一份算：
/// 两条是同一个走法、同一个加法，差别只在取哪一刻。
pub(super) fn slots_referenced_per_device(
    references: &BTreeMap<(u32, u64, u64), String>,
) -> BTreeMap<u32, u64> {
    let mut slots_per_device: BTreeMap<u32, u64> = BTreeMap::new();
    for (device, _start_slot, span_slots) in references.keys() {
        *slots_per_device.entry(*device).or_insert(0) += span_slots;
    }
    slots_per_device
}

#[cfg(test)]
mod tests;

//! 日志记录的事务号、事务边界与发布序号：撕裂、重复、跨两条记录的事务与发布，以及新根段。

use super::*;

/// 两个各自提交了的事务撞号、而又紧挨着：从**发布 B 之后**的干净镜像出发（实例 1 有 A 的计数器 3、事务号 1，
/// B 的计数器 4、事务号 2），把 B 那一条的事务号改回 1、两盘各一份都改、头校验和重封。号不减、同号相连，
/// I-8.7（实例内事务号不重号） 分不出它与「一事务两条」；分得出的是提交标记——事务号 1 那一组两条都带 ⇒
/// 红的是 I-8.8（前缀里的事务不被切开） 的 ③。
fn two_committed_transactions_sharing_one_number_side_by_side(image: &mut MemoryPool) {
    set_transaction_number_of_journal_record(image, 4, 2, 1);
}

/// 记录头里提交标记那 1 字节的偏移：事务段 78 + 事务号 8（D23（journal 的角色与格式） 已定项 7）。
const JOURNAL_RECORD_COMMIT_MARKER_OFFSET: usize = 86;

/// I-8.8（前缀里的事务不被切开） 报不适用时的理由（照 checker 原样）：一事务一条、每条都带提交标记时 ③ ④ 没有对象。
pub(super) const COMMIT_MARKERS_NOT_APPLICABLE: &str = "环里没有哪一组（同一块盘上实例代号与事务号都相同的非 0 记录）落了两条以上、也没有哪一组一条提交标记都没有：③ ④ 没有判的对象，提交标记字节单独成立不算这一条成立";

/// 接在 B 那条记录之后的一条记录：它的事务号与提交标记那 1 字节（D23（journal 的角色与格式） 已定项 7 的两个事务边界字段）。
#[derive(Clone, Copy)]
struct AppendedRecord {
    transaction: u64,
    commit_marker_byte: u8,
}

const fn without_commit_marker(transaction: u64) -> AppendedRecord {
    AppendedRecord {
        transaction,
        commit_marker_byte: 0,
    }
}

const fn with_commit_marker(transaction: u64) -> AppendedRecord {
    AppendedRecord {
        transaction,
        commit_marker_byte: 1,
    }
}

/// 不承载事务的空发布记录：事务号 0、提交标记 1（D23（journal 的角色与格式） 已定项 19 ①），不点名任何单元。
const fn empty_publish() -> AppendedRecord {
    AppendedRecord {
        transaction: 0,
        commit_marker_byte: 1,
    }
}

/// 一事务跨两条记录的合法形态（D23（journal 的角色与格式） 已定项 7「一个事务可以跨多条记录」）：事务号 3 先落一条不带提交标记的，
/// 再落一条带的，接在 B 那条记录之后。I-8.7 与 I-8.8 共用的阳性对照；下面前四份坏镜像各自只在这个形态上改一处。
const ONE_TRANSACTION_OVER_TWO_RECORDS: [AppendedRecord; 2] =
    [without_commit_marker(3), with_commit_marker(3)];

/// I-8.7（实例内事务号不重号） 或 I-8.8（前缀里的事务不被切开） 的一份坏镜像：只该红 `violated_invariant` 这一条，
/// 违例说明里红的恰是 `registered_criteria` 这几条判据（次序照 `TRANSACTION_BOUNDARY_CRITERIA`）。
struct TransactionBoundaryBadImage {
    what_is_changed: &'static str,
    violated_invariant: &'static str,
    registered_criteria: &'static [&'static str],
    appended_records: &'static [AppendedRecord],
}

/// 两条不变量违例说明里的判据标签，次序照 checker：I-8.7 的 ① ②，I-8.8 的 ③、提交标记字节、④。
const TRANSACTION_BOUNDARY_CRITERIA: [&str; 5] =
    ["判据①", "判据②", "判据③", "提交标记字节", "判据④"];

/// I-8.7 与 I-8.8 的坏镜像。前两份是 C37（事务与记录的对应无定义） 点名的「交错」，第四份是它点名的「提交标记放错位置」。
const TRANSACTION_BOUNDARY_BAD_IMAGES: [TransactionBoundaryBadImage; 7] = [
    // 阳性对照那两条之间夹进事务 4 的一条。夹进来的是同一实例一个非 0 的号 ⇒ ① 一定跟着红：
    // 号不减又要把同号的两条隔开，夹进来的号只能等于它自己。
    TransactionBoundaryBadImage {
        what_is_changed: "交错：夹进另一个事务的记录",
        violated_invariant: "I-8.7",
        registered_criteria: &["判据①", "判据②"],
        appended_records: &[
            without_commit_marker(3),
            with_commit_marker(4),
            with_commit_marker(3),
        ],
    },
    // 阳性对照那两条之间夹进一次空发布（事务号 0）：号没有减，红的只有 ②。
    TransactionBoundaryBadImage {
        what_is_changed: "交错：夹进一条空发布",
        violated_invariant: "I-8.7",
        registered_criteria: &["判据②"],
        appended_records: &[
            without_commit_marker(3),
            empty_publish(),
            with_commit_marker(3),
        ],
    },
    // 两个各一条的事务，号先 4 后 3：没有同号的两条，红的只有 ①。
    TransactionBoundaryBadImage {
        what_is_changed: "事务号倒退",
        violated_invariant: "I-8.7",
        registered_criteria: &["判据①"],
        appended_records: &[with_commit_marker(4), with_commit_marker(3)],
    },
    // 阳性对照那两条的提交标记挪到前一条上：带提交标记的那条不是这一组计数器最大的。
    TransactionBoundaryBadImage {
        what_is_changed: "提交标记放错位置",
        violated_invariant: "I-8.8",
        registered_criteria: &["判据③"],
        appended_records: &[with_commit_marker(3), without_commit_marker(3)],
    },
    // 事务 3 一条提交标记都没落，同一实例却接着写了事务 4：失败之后这个实例还在写。
    TransactionBoundaryBadImage {
        what_is_changed: "没提交的事务后面还有号更大的记录",
        violated_invariant: "I-8.8",
        registered_criteria: &["判据④"],
        appended_records: &[without_commit_marker(3), with_commit_marker(4)],
    },
    // 事务 3 一条提交标记都没落，同一实例接着写了一条空发布：空发布也算「失败之后还在写」。
    // 号没有变大（事务号 0 不进 I-8.7 的序列），只看「号更大」的写法在这一份上判不出来。
    TransactionBoundaryBadImage {
        what_is_changed: "没提交的事务后面跟一条空发布",
        violated_invariant: "I-8.8",
        registered_criteria: &["判据④"],
        appended_records: &[without_commit_marker(3), empty_publish()],
    },
    // 事务 3 那一条的提交标记字节写成 2（头校验和照样重封）：不许静默读成「不带」。
    TransactionBoundaryBadImage {
        what_is_changed: "提交标记字节写成 2",
        violated_invariant: "I-8.8",
        registered_criteria: &["提交标记字节"],
        appended_records: &[AppendedRecord {
            transaction: 3,
            commit_marker_byte: 2,
        }],
    },
];

/// 在发布 B 那条记录之后接着写 `appended` 这几条记录，两盘各一份：每条照 B 那条记录改计数器（从 5 起）、代号（从 5 起，
/// 没有哪条不变量读记录头的代号）、事务号与提交标记（事务号 0 的空发布另把点名项清空），反向链接在前一条的头上，
/// 载荷校验和与头校验和由 `JournalRecord::to_bytes` 重封（它只写得出提交标记 0 或 1，别的值改完那一字节再重封头校验和）——
/// 记录自证与 I-8.6（反向链算法） 照旧成立，这几份镜像只在事务号与提交标记的排法上不同。
fn append_journal_records_after_the_overwrite(pool: &mut MemoryPool, appended: &[AppendedRecord]) {
    for device in DEVICES {
        let mut previous_bytes = read(
            pool,
            device,
            singlefs_core::journal::record_offset(
                LAST_COUNTER_AFTER_THE_OVERWRITE,
                singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            )
            .0,
            4096,
        );
        let template = singlefs_core::journal::JournalRecord::parse(
            &previous_bytes,
            get_u64(&previous_bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
        )
        .expect("B 那条记录自证过");
        assert_eq!(
            (template.counter, template.transaction),
            (LAST_COUNTER_AFTER_THE_OVERWRITE, 2),
            "盘 {device} 环里最后一条是 B 那条"
        );
        for (position, appended_record) in (1u64..).zip(appended) {
            let mut record = template.clone();
            record.counter = LAST_COUNTER_AFTER_THE_OVERWRITE + position;
            record.checkpoint_txg = CheckpointTxg(template.checkpoint_txg.0 + position);
            record.transaction = appended_record.transaction;
            record.is_commit = appended_record.commit_marker_byte == 1;
            record.back_chain = singlefs_core::journal::back_chain_of(&previous_bytes);
            if appended_record.transaction == 0 {
                record.named = Vec::new();
            }
            let mut bytes = record.to_bytes();
            if bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET] != appended_record.commit_marker_byte {
                bytes[JOURNAL_RECORD_COMMIT_MARKER_OFFSET] = appended_record.commit_marker_byte;
                let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 46);
                bytes[46..78].copy_from_slice(&digest);
            }
            write(
                pool,
                device,
                singlefs_core::journal::record_offset(
                    record.counter,
                    singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                )
                .0,
                &bytes,
            );
            previous_bytes = bytes;
        }
    }
}

/// 一块盘上计数器 `counter` 那条 journal 记录按 `change` 改、`JournalRecord::to_bytes` 重封，计数器 `counter + 1` 那条（在的话）的反向链
/// 接到改过的头上、同样重封：只在这一块盘上改，记录自证与 I-8.6（反向链算法） 照旧成立。交回改过之后计数器 `counter` 那条的字节。
pub(super) fn rewrite_journal_record_on_one_device_keeping_the_back_chain(
    pool: &mut MemoryPool,
    device: u32,
    counter: u64,
    change: impl FnOnce(&mut JournalRecord),
) -> Vec<u8> {
    let ring_bytes = singlefs_format::JOURNAL_RING_DEFAULT_BYTES;
    let offset = record_offset(counter, ring_bytes).0;
    let bytes = read(pool, device, offset, 4096);
    let mut record = JournalRecord::parse(
        &bytes,
        get_u64(&bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
    )
    .expect("改之前那条记录自证过");
    change(&mut record);
    let rewritten = record.to_bytes();
    write(pool, device, offset, &rewritten);
    let next_offset = record_offset(counter + 1, ring_bytes).0;
    let next_bytes = read(pool, device, next_offset, 4096);
    if let Some(mut next) = JournalRecord::parse(
        &next_bytes,
        get_u64(&next_bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
    ) {
        if next.counter == counter + 1 {
            next.back_chain = back_chain_of(&rewritten);
            write(pool, device, next_offset, &next.to_bytes());
        }
    }
    rewritten
}

/// C469（I-8.7 的两处射程今天判不了） ①，逐盘判：发布 B 之后接两次各一条的发布（事务号 3、4，两盘各一份），
/// 再只把盘 0 上事务号 3 那条（计数器 5）的事务号撕成 8 字节全 0xFF、重封，盘 1 那条好着。逐盘判在盘 0 上红 ①（全 0xFF 之后是 4）；
/// 两盘合判（取好的那块盘的那条）会被盘 1 盖过去、判成立。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：盘 0 不判（只判剩下那块盘）时 I-8.7 成立，这条红。
#[test]
fn transaction_number_torn_on_one_device_reddens_the_transaction_number_invariant_on_that_device() {
    let mut image = image_after_the_overwrite("known-bad-transaction-number-torn-on-one-device");
    append_journal_records_after_the_overwrite(
        &mut image,
        &[with_commit_marker(3), with_commit_marker(4)],
    );
    assert_eq!(
        verdict(&image, "I-8.7"),
        InvariantVerdict::Holds,
        "两盘都是 1、2、3、4：改之前 I-8.7 要真被评估过且成立"
    );
    rewrite_journal_record_on_one_device_keeping_the_back_chain(
        &mut image,
        0,
        LAST_COUNTER_AFTER_THE_OVERWRITE + 1,
        |record| {
            assert_eq!(record.transaction, 3, "计数器 5 那条是事务号 3");
            record.transaction = u64::MAX;
        },
    );
    for device in DEVICES {
        assert_eq!(
            journal_records_on_device(&image, device)
                .into_iter()
                .map(|(_, _, transaction)| transaction)
                .collect::<Vec<u64>>(),
            if device == 0 {
                vec![0, 0, 1, 2, u64::MAX, 4]
            } else {
                vec![0, 0, 1, 2, 3, 4]
            },
            "盘 {device} 环里的事务号"
        );
    }
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-8.7") else {
        panic!(
            "盘 0 上全 0xFF 之后是 4：逐盘判要在盘 0 上红，实际 {:?}",
            verdict(&image, "I-8.7")
        );
    };
    assert!(
        detail.starts_with("判据①：盘 0 实例 1 的两条记录"),
        "红在盘 0 的 ① 上：{detail}"
    );
}

/// C469 ②，回绕不判：
/// 发布 B 之后那份干净镜像上，先把 A、B 两条记录（计数器 3、4）的事务号改成 5、6（两盘各一份，反向链接上），再把计数器 1 那条记录
/// （写行，事务号 0）改成环转过一圈之后落在同一环槽的那一条（计数器 1 + 环槽数，两盘各一份、重封）。实例 1 计数器最小的那条成了计数器 2，
/// 计数器 1 那一格坐着计数器大一个环长的记录：实例 1 写出的第一条被盖掉了，「事务号从 1 起」那半句在这个实例上射程外、什么都不记，
/// 整条照 ① 报：① 判过 5 之后 6、成立。改回绕之前同一份镜像红在「从 1 起」（第一条非 0 记录的号是 5），所以成立只能是回绕那一支不判给的。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：环转过一圈当成判得了时，第一条非 0 记录事务号是 5 不是 1、判红；
/// 回绕那一支照记射程里一部分没判时，① 判过成立的整条被压成不适用：两样都让这条红。
#[test]
fn transaction_numbers_judged_in_order_hold_once_the_ring_wrapped_over_the_first_record_of_the_instance(
) {
    let mut image = image_after_the_overwrite("known-bad-transaction-number-ring-wrapped");
    for device in DEVICES {
        for (counter, transaction) in [(3, 5), (4, 6)] {
            rewrite_journal_record_on_one_device_keeping_the_back_chain(
                &mut image,
                device,
                counter,
                |record| record.transaction = transaction,
            );
        }
    }
    let InvariantVerdict::Violated(detail_before_the_wrap) = verdict(&image, "I-8.7") else {
        panic!(
            "回绕之前实例 1 的第一条记录（计数器 1）还在、第一条非 0 记录事务号是 5：I-8.7 要红在「从 1 起」，实际 {:?}",
            verdict(&image, "I-8.7")
        );
    };
    assert!(
        detail_before_the_wrap.starts_with("从 1 起："),
        "回绕之前红在「从 1 起」那半句上：{detail_before_the_wrap}"
    );
    let ring_slots = singlefs_format::JOURNAL_RING_DEFAULT_BYTES / 4096;
    for device in DEVICES {
        let offset = record_offset(1, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0;
        let bytes = read(&image, device, offset, 4096);
        let mut record = JournalRecord::parse(
            &bytes,
            get_u64(&bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
        )
        .expect("计数器 1 那条自证过");
        assert_eq!(
            (record.counter, record.transaction),
            (1, 0),
            "盘 {device} 计数器 1 那条是写行的空发布"
        );
        record.counter = 1 + ring_slots;
        write(
            &mut image,
            device,
            record_offset(record.counter, singlefs_format::JOURNAL_RING_DEFAULT_BYTES).0,
            &record.to_bytes(),
        );
    }
    assert_eq!(
        verdict(&image, "I-8.7"),
        InvariantVerdict::Holds,
        "实例 1 的第一条被环转过一圈之后的记录盖掉：「从 1 起」那半句射程外、什么都不记，① 判过 5 之后 6，整条报成立"
    );
}

/// C469 ② 判得了的那一形：发布 B 之后那份干净镜像上，A、B 两条记录（计数器 3、4）的事务号改成 5、6（两盘各一份，反向链接上）。
/// ① 成立（5 之后是 6），实例 1 的第一条记录（计数器 1）还在环里、到第一条非 0 记录之间每一格都读得出：第一条非 0 记录的号是 5 不是 1，红。
/// 判别力（`crates/mutations.tsv` 里点名这条的行）：「从 1 起」那一判恒成立时 I-8.7 成立，这条红。
#[test]
fn new_pool_file_creation_number_other_than_one_reddens_the_transaction_number_invariant() {
    let mut image = image_after_the_overwrite("known-bad-transaction-number-not-from-one");
    for device in DEVICES {
        for (counter, transaction) in [(3, 5), (4, 6)] {
            rewrite_journal_record_on_one_device_keeping_the_back_chain(
                &mut image,
                device,
                counter,
                |record| record.transaction = transaction,
            );
        }
    }
    let InvariantVerdict::Violated(detail) = verdict(&image, "I-8.7") else {
        panic!(
            "实例 1 写出的第一条非 0 记录事务号是 5：I-8.7 要红，实际 {:?}",
            verdict(&image, "I-8.7")
        );
    };
    assert!(
        detail.starts_with("从 1 起：盘 0 实例 1 写出的第一条非 0 记录（计数器 3）事务号是 5"),
        "红在「从 1 起」那半句上：{detail}"
    );
    assert!(
        matches!(verdict(&image, "I-8.6"), InvariantVerdict::Holds),
        "反向链接上了，I-8.6 照旧成立"
    );
}

/// I-8.7 与 I-8.8 的坏镜像，交给「每条不变量至少有一份坏镜像」那一处数清单；判定逐份在
/// `each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria` 里。
pub(super) fn known_bad_images_of_the_transaction_boundaries() -> Vec<(&'static str, Mutation)> {
    TRANSACTION_BOUNDARY_BAD_IMAGES
        .into_iter()
        .map(|bad_image| -> (&'static str, Mutation) {
            (
                bad_image.violated_invariant,
                Box::new(move |image: &mut MemoryPool| {
                    append_journal_records_after_the_overwrite(image, bad_image.appended_records);
                }),
            )
        })
        .collect()
}

/// 一事务一条（第一版可达的形态）、别处都干净的镜像上每一条该有的判定：I-8.8（前缀里的事务不被切开） 的 ③ ④ 没有对象、
/// 报不适用，别的每一条都要真被评估过且成立。I-8.8 的阳性对照与坏镜像在
/// `one_transaction_over_two_records_with_the_commit_marker_on_the_last_holds_both_transaction_boundary_invariants` 与后一条用例里。
/// 用它的镜像都没抬过 F（发布 B 之后、回退到 A 之后），I-7.9 报不适用；抬过 F 的镜像在
/// `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant` 里。
pub(super) fn expected_verdict_with_one_record_per_transaction(
    invariant: &str,
) -> InvariantVerdict {
    match invariant {
        "I-8.8" => InvariantVerdict::NotApplicable(COMMIT_MARKERS_NOT_APPLICABLE),
        "I-7.9" => InvariantVerdict::NotApplicable(NO_ROOT_RAISED_THE_FLOOR),
        // 最新根之上没有记录：重放之后那一版就是最新根。
        "I-4.1" => InvariantVerdict::NotApplicable(NOTHING_APPLIED_ABOVE_THE_CHOSEN_ROOT),
        // 用它的镜像上都没有删除：没有处于删除中的号；也没有整理意图，I-4.4 报不适用（logged_ops 里没有意图条目）。
        "I-4.7" => InvariantVerdict::NotApplicable(NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION),
        // I-4.6 自截断起每条记录都判（单元连号、单元数对标签，m4-trunc-r1 判决 K7 T-3）：文件的记录有单元，真被评估过且成立。
        "I-4.6" => InvariantVerdict::Holds,
        "I-4.4" => InvariantVerdict::NotApplicable(
            NO_INTENT_ENTRY_IN_THE_LOGGED_OPERATIONS_OF_ANY_WALKED_VERSION,
        ),
        // I-5.7（C706）：用它的镜像上根槽都读得出、最新根之上没有记录，没有由记录施加出来的那一版。
        // I-3.19、I-8.14、I-3.20 落到下面那一支的成立：发布 B 之后那一份只有实例 1、I-3.19 报不适用，在用它的那条用例里单列。
        ALLOCATION_RECORD_TREE_ROOT_OF_A_VERSION_APPLIED_ONLY_BY_RECORDS_IS_BORN_WITH_THE_VERSION => {
            InvariantVerdict::NotApplicable(
                NO_VERSION_APPLIED_ONLY_BY_RECORDS_REWRITING_ITS_ALLOCATION_RECORD_TREE,
            )
        }
        other_invariant => {
            expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
                other_invariant,
            )
        }
    }
}

/// 没有删除、没有回退的镜像上墓碑与谱系那九条（里程碑三·3，判决 `research/prompts/m3-delete-checker-r2-main-verification.md`）的判定：
/// 墓碑树、谱系树都是零根，I-9.17–I-9.23 没有对象、报不适用带理由（I-9.22 是旧入口当没装旋钮）；I-9.24（队列空的一版报成立）与
/// I-9.29（inode 记录各号只一个出生代）真被评估过且成立。别的不变量交回成立。
pub(super) fn expected_verdict_of_the_tombstone_and_lineage_invariants_without_deletion(
    invariant: &str,
) -> InvariantVerdict {
    match invariant {
        LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER => {
            InvariantVerdict::NotApplicable(NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION)
        }
        QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE => {
            InvariantVerdict::NotApplicable(NO_QUEUED_INODE_IN_ANY_WALKED_VERSION)
        }
        TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT => {
            InvariantVerdict::NotApplicable(NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD)
        }
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED => {
            InvariantVerdict::NotApplicable(NO_TOMBSTONE_IN_ANY_WALKED_VERSION)
        }
        NON_RIGHTMOST_TOMBSTONE_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE => {
            InvariantVerdict::NotApplicable(NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER)
        }
        TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE => {
            InvariantVerdict::NotApplicable(NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY)
        }
        TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER => {
            InvariantVerdict::NotApplicable(NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION)
        }
        // 退役树那几条：没有叶容器身份消失，退役树零根；I-9.35 在相邻两版上真被评估过且成立（叶身份集没变、没有退役记录）。
        RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER => {
            InvariantVerdict::NotApplicable(NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION)
        }
        EVERY_RETIREMENT_RECORD_IS_WELL_FORMED => {
            InvariantVerdict::NotApplicable(NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION)
        }
        RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT
        | RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF => InvariantVerdict::NotApplicable(
            NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION,
        ),
        // 谱系树与退役树的容器容量那两条：两棵都是零根，没有非最右的容器；旧入口没有谱系与退役的旋钮值。
        NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE => {
            InvariantVerdict::NotApplicable(NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER)
        }
        LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE => {
            InvariantVerdict::NotApplicable(NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB)
        }
        // 里程碑四自建 GC 那几条（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3）：logged_ops 树是零根，没有整理意图、没有 R；
        // 没有搬迁过的单元，没有一把映射 key 在两版里落在不同的位置。I-3.15（反查）与 I-3.16（runs 与全空段）真被评估过且成立。
        COMPACTION_REGION_HOLDS_NO_RECORD_ALLOCATED_AFTER_THE_INTENT_WAS_ISSUED => {
            InvariantVerdict::NotApplicable(
                NO_COMPACTION_INTENT_WITH_A_WELL_FORMED_REGION_IN_ANY_WALKED_VERSION,
            )
        }
        UNIT_BYTES_OF_ONE_MAPPING_KEY_AGREE_ACROSS_THE_VERSIONS => InvariantVerdict::NotApplicable(
            NO_MAPPING_KEY_PLACED_DIFFERENTLY_IN_TWO_WALKED_VERSIONS,
        ),
        // I-3.17（隔离记号与引用相符，D19（块指针的结构与宽度预算） 已定项 5）：这几份上没有隔离，最新根的分配记录一条都不带记号。
        // 四个世界在 `crates/singlefs-verification/tests/checker_quarantine_mark_agrees_with_the_references.rs`。
        QUARANTINE_MARK_AGREES_WITH_THE_REFERENCES => InvariantVerdict::NotApplicable(
            NO_RECORD_CARRYING_THE_QUARANTINE_MARK_UNDER_THE_NEWEST_ROOT,
        ),
        _ => InvariantVerdict::Holds,
    }
}

/// 谱系树与退役树的容器容量那两条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 末尾）。
const NO_NON_RIGHTMOST_LINEAGE_OR_RETIREMENT_CONTAINER: &str =
    "走读到的每一版里没装旋钮的谱系树与退役树都是零根或只一片容器：没有非最右的容器";

const NO_LINEAGE_OR_RETIREMENT_CONTAINER_CAPACITY_KNOB: &str =
    "池级 checker 没拿到只供测试的谱系与退役容量旋钮（旧入口，或新入口没给这两个旋钮值）：旋钮那一条不判";

/// 退役树那几条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 与 `check_pool_image` 末尾）。
const NO_RETIREMENT_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版退役树都是零根：没有退役树内部条目可判";

const NO_RETIREMENT_RECORD_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版退役树都是零根：没有退役记录可判";

const NO_RETIREMENT_RECORD_ON_THE_LINEAGE_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版里没有一条退役记录在谱系上：没有对象可判";

/// 墓碑与谱系那几条没有对象时报的理由（`walk` 的 `judge_tombstones_and_the_lineage_of_the_versions` 与 `check_pool_image` 末尾）。
const NO_LINEAGE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版谱系树都是零根：没有谱系树内部条目可判";

const NO_QUEUED_INODE_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版 deleted_inodes 里都没有号：没有队列里的号可判";

const NO_TOMBSTONE_ON_THE_LINEAGE_NAMING_A_RECORD: &str =
    "走读到的每一版里都没有在谱系上、而 inode 树里还有同号同对象出生代记录的墓碑";

const NO_TOMBSTONE_IN_ANY_WALKED_VERSION: &str = "走读到的每一版墓碑树都是零根：没有墓碑记录可判";

const NO_NON_RIGHTMOST_TOMBSTONE_CONTAINER: &str =
    "走读到的每一版墓碑树都是零根或只一片容器：没有非最右的容器";

const NO_TOMBSTONE_CONTAINER_CAPACITY_KNOB_ON_THE_OLD_ENTRY: &str =
    "池级 checker 的旧入口，当没装只供测试的墓碑容量旋钮：旋钮那一条不判";

const NO_TOMBSTONE_TREE_ENTRY_IN_ANY_WALKED_VERSION: &str =
    "走读到的每一版墓碑树都是零根：没有墓碑树内部条目可判";

/// 没抬过 F 的镜像上 I-7.9（回退下界 F 不高于抬 F 的上限） 报的不适用理由。
pub(super) const NO_ROOT_RAISED_THE_FLOOR: &str = "根环里没有抬 F 的根：每条根带的 F 都不高于同一实例里它前一条根带的（回退与新实例的第一条根不算抬）";

/// 一份改过的镜像上 I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 那一片判定，拿来与登记的整张表比。
#[derive(Debug, PartialEq, Eq)]
struct TransactionBoundaryJudgement {
    what_is_changed: &'static str,
    violated_invariants: Vec<&'static str>,
    /// 两条的违例说明里红了哪几条判据，次序照 `TRANSACTION_BOUNDARY_CRITERIA`。
    reddened_criteria: Vec<&'static str>,
    /// 除 I-8.7 与 I-8.8 之外，判定与干净那一份不同的不变量。
    other_verdicts_that_changed: Vec<&'static str>,
}

fn transaction_boundary_judgement(
    what_is_changed: &'static str,
    verdicts: &[(&'static str, InvariantVerdict)],
    verdicts_on_the_clean_image: &[(&'static str, InvariantVerdict)],
) -> TransactionBoundaryJudgement {
    let violation_details: Vec<&str> = verdicts
        .iter()
        .filter(|(name, _)| ["I-8.7", "I-8.8"].contains(name))
        .filter_map(|(_, found)| match found {
            InvariantVerdict::Violated(detail) => Some(detail.as_str()),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect();
    TransactionBoundaryJudgement {
        what_is_changed,
        violated_invariants: violated_invariants(verdicts),
        reddened_criteria: TRANSACTION_BOUNDARY_CRITERIA
            .into_iter()
            .filter(|criterion| {
                violation_details
                    .iter()
                    .any(|detail| detail.contains(criterion))
            })
            .collect(),
        other_verdicts_that_changed: verdicts_that_differ_outside(
            verdicts,
            verdicts_on_the_clean_image,
            &[
                "I-8.7",
                "I-8.8",
                // 接在 B 后面的记录在最新根之上：下一次挂载会重放它们，I-4.1（重放后树可遍历） 从「一次都不施加、不适用」变成真被评估过——
                // 判的是重放之后那一版，不是事务边界；它要是红了，上面 `violated_invariants` 那一格照样看得见。
                TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE,
            ],
        )
        .into_iter()
        .map(|(name, _, _)| name)
        .collect(),
    }
}

/// I-7.3（环健康性） 的坏镜像，从 **mkfs 刚写完**那份镜像出发：三个区域槽 0 上那三条第 0 代根的 `checkpoint_txg`
/// 一起改成 1、各自重封自证校验和——「第一次发布把三个区域全盖了一遍」（轮换键算错，每次发布盖满整个环）在盘上的样子。
/// S 里代号全是 1 ⇒ 除代号最大者外一条更早代号的记录都没有，而 S 又不全是第 0 代 ⇒ 例外那一支够不着 ⇒ 必须红。
/// **为什么不建在写完新池新建文件那份镜像上**：那一份上要让 S 只剩代号最大的一条，就得把 mkfs 那条第 0 代根也去掉，
/// 而 mkfs 种的树表单元（50178）只有它还引用着 ⇒ I-3.1（已分配统计对得上） 跟着红（遍历少算 16384），判别力算不到 I-7.3 头上。
/// 那一份的形态另由 `the_root_ring_health_invariant_reddens_...` 里第二段钉住，连它带的那一处一起钉。
pub(super) fn known_bad_images_of_the_root_ring_health() -> Vec<(&'static str, Mutation)> {
    vec![(
        "I-7.3",
        Box::new(|image: &mut MemoryPool| {
            let mut bumped_root_slots = 0;
            for (device, offset) in root_slot_offsets_of_both_devices() {
                let mut bytes = read(image, device, offset, 512);
                if &bytes[..4] != b"SFSR" {
                    continue;
                }
                assert_eq!(get_u64(&bytes, 28), 0, "mkfs 种下的是第 0 代");
                set_u64(&mut bytes, 28, 1);
                let digest = wide_checksum_with_field_zeroed(&bytes, 512, 138);
                bytes[138..170].copy_from_slice(&digest);
                write(image, device, offset, &bytes);
                bumped_root_slots += 1;
            }
            assert_eq!(
                bumped_root_slots, 3,
                "mkfs 在三个区域的槽 0 各种一条第 0 代根"
            );
        }),
    )]
}

/// I-5.4（分配记录罩住的槽互不相交） 的坏镜像，都从发布 B 之后的干净镜像出发、把 A 的 inode 树根那个落点（槽 50244、跨 1）的记录
/// 跨度改成 2（已释放标志照留），两盘各一条（两片叶各一条）——它罩住的 50245 上另有一条记录（A 的分配记录树盘 0 那片叶的落点）。
/// 50244 与 50245 都在叶 61 罩的 [49532, 50344) 里，改完的记录末槽 50245 仍在它那片叶里，按位置寻址那一道（I-1.1） 不跟着红。
/// 这是 Z1-d 在盘上的形状（复用时跨度变了、新记录罩住的旧记录留着）；树里的引用一个没动，I-5.1（物理范围不重叠） 看不见它。
pub(super) fn known_bad_images_of_overlapping_allocation_records() -> Vec<(&'static str, Mutation)>
{
    vec![
        // 改在 B 写的那棵账里（最新根下面）：两条都是已释放、释放代 4 的记录。
        (
            "I-5.4",
            Box::new(|image: &mut MemoryPool| {
                assert_eq!(
                    mutate_allocation_leaves(
                        image,
                        &UNITS_AFTER_OVERWRITE,
                        ALLOCATION_LEAVES_AFTER_OVERWRITE,
                        |bytes| set_allocation_record_span(bytes, INODE_ROOT, 2),
                    ),
                    2,
                    "A 的 inode 树根那一个落点，两盘各一条"
                );
            }),
        ),
        // 改在 A 写的那棵账里（只有 A 的根指着它，B 之后 A 的根仍在候选集里）：两条都是仍分配、分配代 3 的记录。
        // 「候选集里每条有效根」那个量词靠它：只判最新根那棵账的 checker 在这份镜像上判成立。
        (
            "I-5.4",
            Box::new(|image: &mut MemoryPool| {
                assert_eq!(
                    mutate_allocation_leaves(
                        image,
                        &UNITS_AFTER_OVERWRITE,
                        ALLOCATION_LEAVES,
                        |bytes| set_allocation_record_span(bytes, INODE_ROOT, 2),
                    ),
                    2,
                    "A 的 inode 树根那一个落点，两盘各一条"
                );
            }),
        ),
    ]
}

/// I-5.4 那份坏镜像**只**红在 I-5.4 上：别的不变量跟着红，就说明镜像改宽了，判别力算不到这一条头上。
/// 例外是改在 A 那棵账里的那一份：那两条记录未释放，罩住的是码 2 一槽的 inode 树根，跨度改成 2 同时也是 I-3.18（分配记录的跨度等于它罩住的
/// 单元那一类的槽数，C411） 的违例——两条一起红，别的不许红。改在 B 那棵账里的那两条已释放，I-3.18 不判已释放的记录。
#[test]
fn an_allocation_record_whose_span_covers_the_next_record_reddens_only_the_disjointness_invariant()
{
    let clean = image_after_the_overwrite("known-bad-overlapping-records");
    assert_eq!(
        verdict(&clean, "I-5.4"),
        InvariantVerdict::Holds,
        "发布 B 之后的干净镜像上 I-5.4 要真被评估过且成立"
    );
    let images_and_where_they_are_changed = [
        ("B 写的那棵账（最新根）", vec!["I-5.4"]),
        (
            "A 写的那棵账（更早的候选根）",
            vec![
                "I-5.4",
                ALLOCATION_RECORD_SPAN_EQUALS_THE_SLOTS_OF_THE_CLASS_OF_THE_UNIT_IT_COVERS,
            ],
        ),
    ];
    let bad_images = known_bad_images_of_overlapping_allocation_records();
    assert_eq!(bad_images.len(), images_and_where_they_are_changed.len());
    for ((invariant, mutation), (where_changed, expected_violated)) in bad_images
        .into_iter()
        .zip(images_and_where_they_are_changed)
    {
        let mut image = clean.clone();
        mutation(&mut image);
        let verdicts = check_pool_image(&image);
        let violated: Vec<&str> = verdicts
            .iter()
            .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(invariant, "I-5.4", "这一组坏镜像登记在 I-5.4 名下");
        assert_eq!(
            violated, expected_violated,
            "改在{where_changed}：判红的该恰是 {expected_violated:?}：{verdicts:?}"
        );
    }
}

/// 同一实例一个事务号用了两次：I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 各看到哪一面，
/// 都建在发布 B 之后那份镜像上：
/// ① 干净那一份：(计数器, 实例, 事务号) 整张表钉下来——写行与两次暖机空发布写 0，A 是 1、B 是 2——I-8.7 真被评估过且成立；
/// ② **阳性对照**：B 之后接一次空发布（事务号 0）再接事务 3。**事务号 0 夹在同一实例两个不同的非 0 号中间是合法的**
///    （抬回退下界那两次空发布就是这个形态）：I-8.7 成立、一条红都没有。把 0 也算进序列的写法在这一份上就红（2 之后是 0），
///    这一格是「事务号 0 不进序列」那条射程的阳性对照；
/// ③ B 那条的事务号改回 1（与 A 紧挨着、各带提交标记）：号不减、同号相连，I-8.7 分不出它与「一事务两条」，
///    分得出的是提交标记——事务号 1 那一组两条都带 ⇒ 只红 I-8.8 的 ③；
/// ④ 那个 bug 在盘上的样子：B 之后两次抬下界的空发布，下一次发布取「上一条记录的事务号 + 1」= 1
///    ⇒ I-8.7 红在 ①（2 之后是 1）与 ②（1 被隔开之后又出现），I-8.8 红在 ③（事务号 1 那一组两条都带提交标记）。
/// ③ ④ 两份的判定整张表比一次；每一份都要求除 I-8.7 与 I-8.8 之外每一条的判定与 ① 那一份逐项相同。
#[test]
fn repeated_transaction_number_in_one_instance_reddens_the_uncut_transaction_invariant_and_only_when_separated_the_transaction_number_invariant(
) {
    let clean = image_after_the_overwrite("known-bad-transaction-number");
    for device in DEVICES {
        assert_eq!(
            journal_records_on_device(&clean, device),
            vec![(1, 1, 0), (2, 1, 0), (3, 1, 1), (4, 1, 2)],
            "盘 {device} 环里的 (计数器, 实例, 事务号)：写行与两次暖机空发布写 0，A 是 1、B 是 2"
        );
    }
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-8.7"),
        InvariantVerdict::Holds,
        "环里有事务号 0 的空发布记录，I-8.7 要真被评估过且成立"
    );
    let mut empty_publish_between_two_numbers = clean.clone();
    append_journal_records_after_the_overwrite(
        &mut empty_publish_between_two_numbers,
        &[empty_publish(), with_commit_marker(3)],
    );
    assert_eq!(
        verdict(&empty_publish_between_two_numbers, "I-8.7"),
        InvariantVerdict::Holds,
        "事务号 0 夹在 2 与 3 中间：I-8.7 要真被评估过且成立"
    );
    let verdicts = check_pool_image(&empty_publish_between_two_numbers);
    assert_eq!(
        transaction_boundary_judgement(
            "空发布夹在 2 与 3 中间",
            &verdicts,
            &verdicts_on_the_clean_image
        ),
        TransactionBoundaryJudgement {
            what_is_changed: "空发布夹在 2 与 3 中间",
            violated_invariants: Vec::new(),
            reddened_criteria: Vec::new(),
            other_verdicts_that_changed: Vec::new(),
        },
        "一条红都不该有：{verdicts:?}"
    );
    let mut side_by_side = clean.clone();
    two_committed_transactions_sharing_one_number_side_by_side(&mut side_by_side);
    let mut after_raising_the_floor = clean.clone();
    append_journal_records_after_the_overwrite(
        &mut after_raising_the_floor,
        &[empty_publish(), empty_publish(), with_commit_marker(1)],
    );
    let judged: Vec<TransactionBoundaryJudgement> = [
        ("撞号而紧挨着", &side_by_side),
        ("撞号而中间隔着两次空发布", &after_raising_the_floor),
    ]
    .into_iter()
    .map(|(what_is_changed, image)| {
        transaction_boundary_judgement(
            what_is_changed,
            &check_pool_image(image),
            &verdicts_on_the_clean_image,
        )
    })
    .collect();
    assert_eq!(
        judged,
        vec![
            TransactionBoundaryJudgement {
                what_is_changed: "撞号而紧挨着",
                violated_invariants: vec!["I-8.8"],
                reddened_criteria: vec!["判据③"],
                other_verdicts_that_changed: Vec::new(),
            },
            TransactionBoundaryJudgement {
                what_is_changed: "撞号而中间隔着两次空发布",
                violated_invariants: vec!["I-8.7", "I-8.8"],
                reddened_criteria: vec!["判据①", "判据②", "判据③"],
                other_verdicts_that_changed: Vec::new(),
            },
        ],
        "两份撞号镜像各红在哪；违例说明：紧挨着那份 I-8.8 = {:?}，隔开那份 I-8.7 = {:?}、I-8.8 = {:?}",
        verdict(&side_by_side, "I-8.8"),
        verdict(&after_raising_the_floor, "I-8.7"),
        verdict(&after_raising_the_floor, "I-8.8")
    );
}

/// I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 共用的阳性对照，建在发布 B 之后那份镜像上：
/// ① 干净那一份一事务一条、每条都带提交标记 ⇒ I-8.8 的 ③ ④ 没有对象，报「不适用」，不报成立；
/// ② B 之后接着写事务 3 的两条记录，提交标记在计数器大的那一条（D23（journal 的角色与格式） 已定项 7 允许的形态），
///    两盘各一份、反向链接好 ⇒ 两条都要真被评估过且成立、一条红都没有，除 I-8.7 与 I-8.8 之外每一条的判定与 ① 那一份逐项相同。
///    见到两条同号就红的实现（I-8.7 还按严格递增判）、和从来报不出成立的 I-8.8 在这一格上红；
///    从来不红的实现由后一条用例的坏镜像挡。
#[test]
fn one_transaction_over_two_records_with_the_commit_marker_on_the_last_holds_both_transaction_boundary_invariants(
) {
    let clean = image_after_the_overwrite("known-bad-transaction-boundaries-positive");
    for device in DEVICES {
        assert_eq!(
            journal_records_on_device(&clean, device),
            vec![(1, 1, 0), (2, 1, 0), (3, 1, 1), (4, 1, 2)],
            "盘 {device} 环里的 (计数器, 实例, 事务号)：写行与两次暖机空发布写 0，A 是 1、B 是 2"
        );
    }
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-8.8"),
        InvariantVerdict::NotApplicable(COMMIT_MARKERS_NOT_APPLICABLE),
        "一事务一条：I-8.8 的 ③ ④ 没有对象，报不适用"
    );
    let mut image = clean.clone();
    append_journal_records_after_the_overwrite(&mut image, &ONE_TRANSACTION_OVER_TWO_RECORDS);
    for device in DEVICES {
        assert_eq!(
            journal_records_on_device(&image, device),
            vec![
                (1, 1, 0),
                (2, 1, 0),
                (3, 1, 1),
                (4, 1, 2),
                (5, 1, 3),
                (6, 1, 3)
            ],
            "盘 {device}：B 之后接着落事务 3 的两条"
        );
    }
    let verdicts = check_pool_image(&image);
    assert_eq!(
        (verdict(&image, "I-8.7"), verdict(&image, "I-8.8")),
        (InvariantVerdict::Holds, InvariantVerdict::Holds),
        "事务 3 跨两条、提交标记在计数器大的那条：I-8.7 与 I-8.8 都要真被评估过且成立"
    );
    assert_eq!(
        transaction_boundary_judgement("一事务两条", &verdicts, &verdicts_on_the_clean_image),
        TransactionBoundaryJudgement {
            what_is_changed: "一事务两条",
            violated_invariants: Vec::new(),
            reddened_criteria: Vec::new(),
            other_verdicts_that_changed: Vec::new(),
        },
        "一条红都不该有，别的判定一条不变：{verdicts:?}"
    );
}

/// I-8.7（实例内事务号不重号） 与 I-8.8（前缀里的事务不被切开） 的七份坏镜像（`TRANSACTION_BOUNDARY_BAD_IMAGES`：
/// 前四份各自只在阳性对照那个形态上改一处）：每份**只红登记的那一条不变量**，违例说明里红的判据恰是登记的那几条
/// （判别力要算到那一条判据头上，不只算到这条不变量头上），除 I-8.7 与 I-8.8 之外每一条的判定与干净那一份逐项相同。
/// 七份的判定先全部收齐、再整张表比一次，一处改坏时红的消息里看得到每一份各红在哪。
#[test]
fn each_transaction_boundary_bad_image_reddens_only_its_own_invariant_on_its_registered_criteria() {
    let clean = image_after_the_overwrite("known-bad-transaction-boundaries");
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-8.8"),
        InvariantVerdict::NotApplicable(COMMIT_MARKERS_NOT_APPLICABLE),
        "一事务一条：I-8.8 的 ③ ④ 没有对象，报不适用"
    );
    let mut judged_by_image: Vec<TransactionBoundaryJudgement> = Vec::new();
    let mut registered_by_image: Vec<TransactionBoundaryJudgement> = Vec::new();
    let mut details_by_image: Vec<(&str, InvariantVerdict, InvariantVerdict)> = Vec::new();
    for TransactionBoundaryBadImage {
        what_is_changed,
        violated_invariant,
        registered_criteria,
        appended_records,
    } in TRANSACTION_BOUNDARY_BAD_IMAGES
    {
        let mut image = clean.clone();
        append_journal_records_after_the_overwrite(&mut image, appended_records);
        let verdicts = check_pool_image(&image);
        judged_by_image.push(transaction_boundary_judgement(
            what_is_changed,
            &verdicts,
            &verdicts_on_the_clean_image,
        ));
        registered_by_image.push(TransactionBoundaryJudgement {
            what_is_changed,
            violated_invariants: vec![violated_invariant],
            reddened_criteria: registered_criteria.to_vec(),
            other_verdicts_that_changed: Vec::new(),
        });
        details_by_image.push((
            what_is_changed,
            verdict(&image, "I-8.7"),
            verdict(&image, "I-8.8"),
        ));
    }
    assert_eq!(
        judged_by_image, registered_by_image,
        "每份坏镜像只红登记的那一条、红在登记的那几条判据上、别的判定一条不变；(镜像, I-8.7, I-8.8) 逐份是 {details_by_image:?}"
    );
}

/// 记录头里记录标志那 1 字节的偏移：magic 4 + 类型 2 + 算法类型 1（D23（journal 的角色与格式） 已定项 4，原「填充 1」那个字节）。
const JOURNAL_RECORD_FLAGS_OFFSET: usize = 7;

/// 接在 B 那条记录之后的一条记录，I-8.9（一次发布的记录序号连续且只有末条带标志） 那几份镜像用：它属于哪次发布
/// （checkpoint_txg 比 B 大几）、事务号与提交标记、本次发布内序号、记录标志那 1 字节（D23（journal 的角色与格式） 已定项 4 / 已定项 17）。
#[derive(Clone, Copy)]
struct AppendedPublishRecord {
    txg_after_the_overwrite: u64,
    transaction: u64,
    is_commit: bool,
    ordinal_within_publish: u32,
    record_flags_byte: u8,
}

const fn publish_record(
    txg_after_the_overwrite: u64,
    transaction: u64,
    is_commit: bool,
    ordinal_within_publish: u32,
    record_flags_byte: u8,
) -> AppendedPublishRecord {
    AppendedPublishRecord {
        txg_after_the_overwrite,
        transaction,
        is_commit,
        ordinal_within_publish,
        record_flags_byte,
    }
}

/// 一次发布两条记录的合法形态（D23（journal 的角色与格式） 已定项 17：最后一个事务装不下一条记录时末条再跨记录）：
/// 发布 txg = B + 1，事务 3 跨两条，序号 1、2，提交标记与末条标志都只在第二条。I-8.9 的阳性对照；
/// 下面几份坏镜像各自只在这个形态上改一处。
const ONE_PUBLISH_OVER_TWO_RECORDS: [AppendedPublishRecord; 2] = [
    publish_record(1, 3, false, 1, 0),
    publish_record(1, 3, true, 2, 1),
];

/// I-8.9 违例说明里的判据标签，次序照 checker（`judge_publish_ordinals_and_last_record_flags`）。
const PUBLISH_ORDINAL_CRITERIA: [&str; 7] = [
    "标志其余位",
    "序号 0",
    "跳号",
    "不从 1 起",
    "多于一条末条",
    "末条之后还有记录",
    "没有末条",
];

/// I-8.9 的一份坏镜像：只该红 I-8.9，违例说明里红的恰是 `registered_criteria` 这几条判据（次序照 `PUBLISH_ORDINAL_CRITERIA`）。
struct PublishOrdinalBadImage {
    what_is_changed: &'static str,
    registered_criteria: &'static [&'static str],
    appended_records: &'static [AppendedPublishRecord],
}

/// I-8.9 的坏镜像，每份都从阳性对照那个形态改一处。前五份是派发点名的那五种（跳号、序号 0、没有末条、两条末条、
/// 标志其余位非 0），后两份是 checker 另两条判据（不从 1 起、末条之后同一次发布还有记录）各自的判别力。
const PUBLISH_ORDINAL_BAD_IMAGES: [PublishOrdinalBadImage; 7] = [
    // 第二条的序号 2 → 3：jsn 连号而序号跳了一格。
    PublishOrdinalBadImage {
        what_is_changed: "一次发布之内跳号",
        registered_criteria: &["跳号"],
        appended_records: &[
            publish_record(1, 3, false, 1, 0),
            publish_record(1, 3, true, 3, 1),
        ],
    },
    // 第一条的序号 1 → 0。它的前一个计数器上坐着 B 那条（别的发布）⇒ 它是这次发布的第一条，「不从 1 起」跟着红；
    // 序号 0、1 与计数器同步，不算跳号。
    PublishOrdinalBadImage {
        what_is_changed: "序号 0",
        registered_criteria: &["序号 0", "不从 1 起"],
        appended_records: &[
            publish_record(1, 3, false, 0, 0),
            publish_record(1, 3, true, 1, 1),
        ],
    },
    // 第二条的末条标志摘掉，同一实例接着写了下一次发布（txg = B + 2）：这次发布写完了，却一条末条都没有。
    PublishOrdinalBadImage {
        what_is_changed: "没有末条",
        registered_criteria: &["没有末条"],
        appended_records: &[
            publish_record(1, 3, false, 1, 0),
            publish_record(1, 3, true, 2, 0),
            publish_record(2, 4, true, 1, 1),
        ],
    },
    // 第一条也带末条标志。
    PublishOrdinalBadImage {
        what_is_changed: "两条末条",
        registered_criteria: &["多于一条末条"],
        appended_records: &[
            publish_record(1, 3, false, 1, 1),
            publish_record(1, 3, true, 2, 1),
        ],
    },
    // 末条的记录标志写成 0b11：位 0 对，位 1 不许为 1。
    PublishOrdinalBadImage {
        what_is_changed: "记录标志其余位非 0",
        registered_criteria: &["标志其余位"],
        appended_records: &[
            publish_record(1, 3, false, 1, 0),
            publish_record(1, 3, true, 2, 0b0000_0011),
        ],
    },
    // 两条的序号 2、3：与 jsn 同步、不跳号，可这次发布的第一条不是 1。
    PublishOrdinalBadImage {
        what_is_changed: "不从 1 起",
        registered_criteria: &["不从 1 起"],
        appended_records: &[
            publish_record(1, 3, false, 2, 0),
            publish_record(1, 3, true, 3, 1),
        ],
    },
    // 末条标志挪到第一条：带标志的那条后面同一次发布还有一条。
    PublishOrdinalBadImage {
        what_is_changed: "末条标志不在最后一条",
        registered_criteria: &["末条之后还有记录"],
        appended_records: &[
            publish_record(1, 3, false, 1, 1),
            publish_record(1, 3, true, 2, 0),
        ],
    },
];

/// 在发布 B 那条记录之后接着写 `appended` 这几条记录，两盘各一份：照 B 那条记录改计数器（从 5 起）、checkpoint_txg、
/// 事务号与提交标记、本次发布内序号与记录标志，反向链接在前一条的头上；载荷校验和与头校验和由 `JournalRecord::to_bytes` 重封，
/// 它写不出来的记录标志（其余位非 0）改完那一字节再重封头校验和——记录自证与 I-8.6（反向链算法） 照旧成立。
fn append_publish_records_after_the_overwrite(
    pool: &mut MemoryPool,
    appended: &[AppendedPublishRecord],
) {
    append_publish_records_after_the_overwrite_changing_each_record(pool, appended, |_, _, _| {});
}

/// 同 [`append_publish_records_after_the_overwrite`]，每条记录在写出之前再交给 `change` 改一处（第几条（从 0 起）、记录本身、
/// 它照着写的那块盘上 B 之前那几条记录读得到的镜像）；反向链与两道校验和照改过的那一份算。
fn append_publish_records_after_the_overwrite_changing_each_record(
    pool: &mut MemoryPool,
    appended: &[AppendedPublishRecord],
    change: impl Fn(usize, &mut singlefs_core::journal::JournalRecord, &MemoryPool),
) {
    for device in DEVICES {
        let mut previous_bytes = read(
            pool,
            device,
            singlefs_core::journal::record_offset(
                LAST_COUNTER_AFTER_THE_OVERWRITE,
                singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
            )
            .0,
            4096,
        );
        let template = singlefs_core::journal::JournalRecord::parse(
            &previous_bytes,
            get_u64(&previous_bytes, JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET),
        )
        .expect("B 那条记录自证过");
        assert_eq!(
            (template.counter, template.transaction),
            (LAST_COUNTER_AFTER_THE_OVERWRITE, 2),
            "盘 {device} 环里最后一条是 B 那条"
        );
        for (position, appended_record) in (1u64..).zip(appended) {
            let mut record = template.clone();
            record.counter = LAST_COUNTER_AFTER_THE_OVERWRITE + position;
            record.checkpoint_txg =
                CheckpointTxg(template.checkpoint_txg.0 + appended_record.txg_after_the_overwrite);
            record.transaction = appended_record.transaction;
            record.is_commit = appended_record.is_commit;
            record.ordinal_within_publish =
                singlefs_core::journal::JournalRecordOrdinalWithinPublish(
                    appended_record.ordinal_within_publish,
                );
            record.place_in_publish = if appended_record.record_flags_byte & 1 == 1 {
                singlefs_core::journal::JournalRecordPlaceInPublish::LastRecordOfThePublish
            } else {
                singlefs_core::journal::JournalRecordPlaceInPublish::MoreRecordsOfThePublishFollow
            };
            change(
                usize::try_from(position - 1).expect("接几条记录"),
                &mut record,
                pool,
            );
            record.back_chain = singlefs_core::journal::back_chain_of(&previous_bytes);
            let mut bytes = record.to_bytes();
            if bytes[JOURNAL_RECORD_FLAGS_OFFSET] != appended_record.record_flags_byte {
                bytes[JOURNAL_RECORD_FLAGS_OFFSET] = appended_record.record_flags_byte;
                let digest = wide_checksum_with_field_zeroed(&bytes, 4096, 46);
                bytes[46..78].copy_from_slice(&digest);
            }
            write(
                pool,
                device,
                singlefs_core::journal::record_offset(
                    record.counter,
                    singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                )
                .0,
                &bytes,
            );
            previous_bytes = bytes;
        }
    }
}

/// I-8.9 的坏镜像，交给「每条不变量至少有一份坏镜像」那一处数清单；判定逐份在
/// `each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria` 里。
pub(super) fn known_bad_images_of_the_publish_ordinals() -> Vec<(&'static str, Mutation)> {
    PUBLISH_ORDINAL_BAD_IMAGES
        .into_iter()
        .map(|bad_image| -> (&'static str, Mutation) {
            (
                "I-8.9",
                Box::new(move |image: &mut MemoryPool| {
                    append_publish_records_after_the_overwrite(image, bad_image.appended_records);
                }),
            )
        })
        .collect()
}

/// 一份改过的镜像上 I-8.9 那一片判定，拿来与登记的整张表比。
#[derive(Debug, PartialEq, Eq)]
struct PublishOrdinalJudgement {
    what_is_changed: &'static str,
    violated_invariants: Vec<&'static str>,
    /// I-8.9 的违例说明里红了哪几条判据，次序照 `PUBLISH_ORDINAL_CRITERIA`。
    reddened_criteria: Vec<&'static str>,
    /// 除 I-8.9 之外，判定与阳性对照那一份不同的不变量。
    other_verdicts_that_changed: Vec<&'static str>,
}

fn publish_ordinal_judgement(
    what_is_changed: &'static str,
    verdicts: &[(&'static str, InvariantVerdict)],
    verdicts_on_the_positive_control: &[(&'static str, InvariantVerdict)],
) -> PublishOrdinalJudgement {
    let violation_detail = verdicts
        .iter()
        .find(|(name, _)| *name == "I-8.9")
        .and_then(|(_, found)| match found {
            InvariantVerdict::Violated(detail) => Some(detail.as_str()),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .unwrap_or("");
    PublishOrdinalJudgement {
        what_is_changed,
        violated_invariants: violated_invariants(verdicts),
        reddened_criteria: PUBLISH_ORDINAL_CRITERIA
            .into_iter()
            .filter(|criterion| violation_detail.contains(&format!("{criterion}：")))
            .collect(),
        other_verdicts_that_changed: verdicts_that_differ_outside(
            verdicts,
            verdicts_on_the_positive_control,
            // I-4.1 不比的理由同 `transaction_boundary_judgement`：接在 B 后面的记录在最新根之上，重放到哪一条随序号与末条标志变。
            &["I-8.9", TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE],
        )
        .into_iter()
        .map(|(name, _, _)| name)
        .collect(),
    }
}

/// I-8.9（一次发布的记录序号连续且只有末条带标志） 的阳性对照，建在发布 B 之后那份镜像上：
/// ① 干净那一份每次发布一条记录（写行、两次暖机、A、B，序号都是 1、都带末条标志）⇒ I-8.9 真被评估过且成立；
/// ② B 之后接着写一次两条记录的发布（事务 3 跨两条，序号 1、2，提交标记与末条标志都只在第二条，D23（journal 的角色与格式）
///    已定项 17 末条再跨记录的形态）⇒ I-8.9 仍成立、一条红都没有；除 I-8.8（前缀里的事务不被切开） 那一格
///    （一事务两条，它从不适用变成真被评估过且成立）之外每一条的判定与 ① 那一份逐项相同。
///    把多条记录的发布一律判红的实现（例如「每条记录都要带末条标志」）在这一格上红；从来不红的实现由后一条用例的坏镜像挡。
#[test]
fn one_publish_over_two_records_with_the_last_record_flag_on_the_second_holds_the_publish_ordinal_invariant(
) {
    let clean = image_after_the_overwrite("known-bad-publish-ordinals-positive");
    let verdicts_on_the_clean_image = check_pool_image(&clean);
    assert_eq!(
        verdict(&clean, "I-8.9"),
        InvariantVerdict::Holds,
        "每次发布一条记录：I-8.9 真被评估过且成立"
    );
    let mut image = clean.clone();
    append_publish_records_after_the_overwrite(&mut image, &ONE_PUBLISH_OVER_TWO_RECORDS);
    let verdicts = check_pool_image(&image);
    assert_eq!(
        (verdict(&image, "I-8.8"), verdict(&image, "I-8.9")),
        (InvariantVerdict::Holds, InvariantVerdict::Holds),
        "一次发布两条记录：I-8.8 与 I-8.9 都要真被评估过且成立"
    );
    assert_eq!(
        verdict(&image, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
        InvariantVerdict::Holds,
        "这次两条记录的发布在最新根之上、末条带标志与提交标记：下一次挂载整次重放它，重放之后那一版走得完"
    );
    assert_eq!(
        publish_ordinal_judgement("一次发布两条记录", &verdicts, &verdicts_on_the_clean_image),
        PublishOrdinalJudgement {
            what_is_changed: "一次发布两条记录",
            violated_invariants: Vec::new(),
            reddened_criteria: Vec::new(),
            other_verdicts_that_changed: vec!["I-8.8"],
        },
        "一条红都不该有，除 I-8.8 从不适用变成成立之外别的判定一条不变：{verdicts:?}"
    );
}

/// A 那条记录（计数器 3、txg 3、事务 1）在发布 B 之后那份镜像上的计数器。
const COUNTER_OF_THE_RECORD_OF_THE_FILE_CREATION_PUBLISH: u64 = 3;

/// 发布 B 之后接一次两条记录的发布（`ONE_PUBLISH_OVER_TWO_RECORDS`），第一条（非末条）的新根段换成 A 那一版的（同一块盘上计数器 3 那条记录
/// 带的九段原样）：写者把非末条的新根段写成上一版（`research/prompts/m3-formal-code-b34-r1-main-verification.md` 第二节 P1-b 攻方 s1 那一形）。
/// 末条照阳性对照带 B 的那一份；反向链与两道校验和照改过的那一份算。
fn append_a_publish_over_two_records_whose_first_record_carries_the_new_root_segment_of_the_file_creation_publish(
    image: &mut MemoryPool,
) {
    append_publish_records_after_the_overwrite_changing_each_record(
        image,
        &ONE_PUBLISH_OVER_TWO_RECORDS,
        |index, record, pool| {
            if index != 0 {
                return;
            }
            // 两块盘上 A 那条记录的新根段是同一条根的九段（同一次发布写两盘），取盘 0 那一份。
            let bytes_of_the_file_creation_publish = read(
                pool,
                0,
                singlefs_core::journal::record_offset(
                    COUNTER_OF_THE_RECORD_OF_THE_FILE_CREATION_PUBLISH,
                    singlefs_format::JOURNAL_RING_DEFAULT_BYTES,
                )
                .0,
                4096,
            );
            let file_creation_publish = JournalRecord::parse(
                &bytes_of_the_file_creation_publish,
                get_u64(
                    &bytes_of_the_file_creation_publish,
                    JOURNAL_RECORD_FILESYSTEM_IDENTIFIER_OFFSET,
                ),
            )
            .expect("A 那条记录自证过");
            assert_eq!(
                (
                    file_creation_publish.counter,
                    file_creation_publish.transaction
                ),
                (COUNTER_OF_THE_RECORD_OF_THE_FILE_CREATION_PUBLISH, 1),
                "计数器 3 那条是 A 的记录"
            );
            record.new_tree_table = file_creation_publish.new_tree_table;
            record.new_mapping_root = file_creation_publish.new_mapping_root;
            record.new_tree_identifier_watermark =
                file_creation_publish.new_tree_identifier_watermark;
            record.new_rollback_floor = file_creation_publish.new_rollback_floor;
            record.new_instance_table = file_creation_publish.new_instance_table;
            record.new_allocation_record_tree_root =
                file_creation_publish.new_allocation_record_tree_root;
            record.new_unmount_marker = file_creation_publish.new_unmount_marker;
            record.new_reclaim_threshold = file_creation_publish.new_reclaim_threshold;
            record.new_pinned_ring_slot = file_creation_publish.new_pinned_ring_slot;
        },
    );
}

/// I-8.10（记录新根段与同代根相同） 往非末条扩的那一半（C738（同一次发布几条记录的新根段一致没有不变量判））的坏镜像：一次两条记录的发布，
/// 非末条带上一版（A）的新根段，**只**红 I-8.10，说明点名同一次发布的两条记录；别的每一条与阳性对照（两条都带 B 那一份）逐项相同。
/// 只比末条的判法在这一份上看不见非末条（0 条违例）。
#[test]
fn a_non_last_record_of_a_publish_carrying_the_new_root_segment_of_the_version_before_reddens_only_the_new_root_segment_invariant(
) {
    let clean = image_after_the_overwrite("known-bad-new-root-segment-of-a-non-last-record");
    let mut positive_control = clean.clone();
    append_publish_records_after_the_overwrite(
        &mut positive_control,
        &ONE_PUBLISH_OVER_TWO_RECORDS,
    );
    let verdicts_on_the_positive_control = check_pool_image(&positive_control);
    assert_eq!(
        verdict(
            &positive_control,
            LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH
        ),
        InvariantVerdict::Holds,
        "两条都带同一份新根段的阳性对照上 I-8.10 真被评估过且成立"
    );
    let mut image = clean.clone();
    append_a_publish_over_two_records_whose_first_record_carries_the_new_root_segment_of_the_file_creation_publish(
        &mut image,
    );
    let verdicts = check_pool_image(&image);
    assert_eq!(
        violated_invariants(&verdicts),
        vec![LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH],
        "只红 I-8.10：{verdicts:?}"
    );
    let InvariantVerdict::Violated(detail) = verdict(
        &image,
        LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH,
    ) else {
        panic!("上面判过 I-8.10 红");
    };
    assert!(
        detail.contains("同一次发布（实例 1、txg 5）的两条记录新根段不逐字节相同")
            && detail.contains("计数器 6 那一条与计数器 5 那一条"),
        "违例点名 B 之后那次发布（txg 5）的两条记录：{detail}"
    );
    assert_eq!(
        verdicts_that_differ_outside(
            &verdicts,
            &verdicts_on_the_positive_control,
            &[LAST_RECORD_OF_A_PUBLISH_CARRIES_THE_ROOT_OF_THAT_PUBLISH]
        ),
        Vec::new(),
        "除 I-8.10 之外每一条与阳性对照逐项相同"
    );
}

/// I-8.9（一次发布的记录序号连续且只有末条带标志） 的七份坏镜像（`PUBLISH_ORDINAL_BAD_IMAGES`，每份只在阳性对照那个形态上改一处）：
/// 每份**只红 I-8.9**，违例说明里红的判据恰是登记的那几条，除 I-8.9 之外每一条的判定与阳性对照那一份逐项相同。
/// 七份的判定先全部收齐、再整张表比一次，一处改坏时红的消息里看得到每一份各红在哪。
#[test]
fn each_publish_ordinal_bad_image_reddens_only_the_publish_ordinal_invariant_on_its_registered_criteria(
) {
    let clean = image_after_the_overwrite("known-bad-publish-ordinals");
    let mut positive_control = clean.clone();
    append_publish_records_after_the_overwrite(
        &mut positive_control,
        &ONE_PUBLISH_OVER_TWO_RECORDS,
    );
    let verdicts_on_the_positive_control = check_pool_image(&positive_control);
    let mut judged_by_image: Vec<PublishOrdinalJudgement> = Vec::new();
    let mut registered_by_image: Vec<PublishOrdinalJudgement> = Vec::new();
    let mut details_by_image: Vec<(&str, InvariantVerdict)> = Vec::new();
    for PublishOrdinalBadImage {
        what_is_changed,
        registered_criteria,
        appended_records,
    } in PUBLISH_ORDINAL_BAD_IMAGES
    {
        let mut image = clean.clone();
        append_publish_records_after_the_overwrite(&mut image, appended_records);
        let verdicts = check_pool_image(&image);
        judged_by_image.push(publish_ordinal_judgement(
            what_is_changed,
            &verdicts,
            &verdicts_on_the_positive_control,
        ));
        registered_by_image.push(PublishOrdinalJudgement {
            what_is_changed,
            violated_invariants: vec!["I-8.9"],
            reddened_criteria: registered_criteria.to_vec(),
            other_verdicts_that_changed: Vec::new(),
        });
        details_by_image.push((what_is_changed, verdict(&image, "I-8.9")));
    }
    assert_eq!(
        judged_by_image, registered_by_image,
        "每份坏镜像只红 I-8.9、红在登记的那几条判据上、别的判定一条不变；(镜像, I-8.9) 逐份是 {details_by_image:?}"
    );
}

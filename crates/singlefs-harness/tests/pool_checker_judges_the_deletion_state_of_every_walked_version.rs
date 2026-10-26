//! 池级 checker 的删除线（里程碑三·3）：I-9.5（删除队列里的号仍在树里，两个方向）、I-3.4（只落「记账标签 4 恒 0」）、
//! I-4.7（对象取丙）、I-4.6（取乙：nlink = 0 的记录单元数对 size）、I-4.4（第一版不适用）与 I-9.6 墓碑那一半；
//! 墓碑与谱系那几条（判决 `research/prompts/m3-delete-checker-r2-main-verification.md`）：I-9.18、I-9.19、I-9.20、I-9.24 与
//! I-9.10 墓碑那一个等号，「在谱系上」每一版从自己的 txg 与自己的谱系树起算（T3 取 a）。
//! 读法照判决 `research/prompts/m3-delete-checker-r1-main-verification.md` 第二节与岔路单 `research/prompts/m3-delete-checker-r1-forks.md`：
//! 判回退候选集里每一版（加由记录施加出来的几版），不只判最新根；读不全的那一版报射程里判不了、不判红。
//! 坏镜像各一，每份只红自己那一条；搭建与坏镜像的改法在 `common_deletion_in_progress`（`checker_known_bad_images.rs` 那一串 `.chain` 也登记它们）。

mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common::build_pool;
use common_deletion_in_progress::{
    accounting_tree_slot_of_the_newest_root,
    append_a_copy_of_the_first_tombstone_of_the_newest_root,
    change_the_first_tombstone_of_the_newest_root, change_the_inode_record,
    change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at,
    drop_the_pending_deletion_occupancy_row,
    drop_the_record_of_the_first_file_and_keep_its_data_units,
    drop_the_record_of_the_first_file_in_the_leaf_at_and_keep_its_data_units,
    first_tombstone_container_slot_of_the_newest_root,
    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at,
    image_after_the_first_batch_of_a_three_unit_file, image_after_the_intent_of_a_three_unit_file,
    image_after_the_intent_of_an_empty_inode,
    let_the_empty_livelist_entry_stand_in_for_the_extent_tree_in_the_tree_table_at,
    move_the_tombstone_container_to_an_unpublished_instance_and_raise_its_object_identifier,
    narrow_the_entries_of_the_tree_table_at_to_the_key_width, object_birth_of,
    pool_with_a_first_file_of_size_zero_and_one_data_unit, pool_with_a_three_unit_file,
    pool_with_inode_three_unlinked_and_the_root_ring_turned_over,
    pool_with_inode_two_unlinked_then_rolled_back_before_the_intent_and_the_root_ring_turned_over,
    pool_with_new_inodes, publish_new_inodes_and_advance_the_pool, publish_the_intent,
    publish_the_intent_of, publish_the_next_step_in_batches_of_one,
    put_a_second_inode_being_deleted_into_the_deletion_queue,
    raise_the_size_of_the_first_file_by_one_data_unit_payload,
    raise_the_size_of_the_first_file_in_the_leaf_at,
    raise_the_tombstone_object_identifier_to_the_inode_number_watermark,
    remove_the_deleted_inodes_entry_from_the_tree_table_at,
    replace_the_queued_inode_with_a_number_without_a_record, roll_back_to,
    root_slot_of_the_tree_of_kind_of_the_newest_root,
    set_the_head_identifier_of_the_tree_table_entry_of_kind_at,
    set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero,
    set_the_nlink_of_an_inode_record_to_zero, set_the_nlink_of_the_queued_first_file_back_to_one,
    set_the_pending_deletion_occupancy_row_to_one, set_the_smallest_key_of_the_inode_root_at,
    set_tombstone_field, tombstone_field, tree_table_slot_of_the_newest_root, unlink_to_the_end,
    zero_both_copies_of_the_unit_at, TOMBSTONE_RECORD_DEATH_TXG_OFFSET,
    TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET, TOMBSTONE_RECORD_RANGE_END_OFFSET,
};
use common_pointer_head_corruption::{data_unit_bytes, node_bytes, unit_bytes_at};
use singlefs_checker::image::{
    InvariantVerdict, DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY, INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,
    NO_INTENT_ENTRY_IN_THE_LOGGED_OPERATIONS_OF_ANY_WALKED_VERSION,
    ONE_OBJECT_BIRTH_PER_INODE_NUMBER, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
    QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
    TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT, TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
};
use singlefs_core::inode_tree::InodeLeafContainerIndexInTree;
use singlefs_core::records::{TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT};
use singlefs_core::transaction::{TransactionUnit, FIRST_INODE_NUMBER};
use singlefs_harness::memory_pool::MemoryPool;

/// 判违例的不变量，按 checker 报的次序（池级 checker 自己的单测，手造的镜像就地判：这份文件登记在 `cpu-judging-sites.tsv` 的 excluded 里；
/// 原先在 `common_deletion_in_progress/mod.rs`，那里还有别的产出再判的调用方，判法挪到 checker 档之后只剩这一份用）。
fn violated_invariants(image: &MemoryPool) -> Vec<&'static str> {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .filter_map(|(invariant, verdict)| match verdict {
            InvariantVerdict::Violated(_) => Some(invariant),
            InvariantVerdict::Holds
            | InvariantVerdict::NotApplicable(_)
            | InvariantVerdict::PartiallyJudged(_) => None,
        })
        .collect()
}

/// 一条不变量的判定。
fn verdict_of(image: &MemoryPool, invariant: &str) -> InvariantVerdict {
    singlefs_checker::walk::check_pool_image(image)
        .into_iter()
        .find(|(identifier, _)| *identifier == invariant)
        .expect("清单里有这一条")
        .1
}

/// 第一处违例的说明；这一条没判违例就当场失败。
fn first_violation_of(image: &MemoryPool, invariant: &str) -> String {
    match verdict_of(image, invariant) {
        InvariantVerdict::Violated(detail) => detail,
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::NotApplicable(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("{invariant} 应当判违例，实际 {other_verdict:?}")
        }
    }
}

/// 这一条报不适用时的理由（射程里判不了的那一版也报这个，`Judgements::part_of_the_range_not_judged`）；报了别的判定就当场失败。
fn reason_not_judged(image: &MemoryPool, invariant: &str) -> &'static str {
    match verdict_of(image, invariant) {
        InvariantVerdict::NotApplicable(reason) => reason,
        other_verdict @ (InvariantVerdict::Holds
        | InvariantVerdict::Violated(_)
        | InvariantVerdict::PartiallyJudged(_)) => {
            panic!("{invariant} 应当报射程里判不了（不适用带理由），实际 {other_verdict:?}")
        }
    }
}

/// 删除态扫描在一版上读 I-9.5、I-4.7、I-4.6 三条要的那一组（`walk.rs` 的 `DELETION_QUEUE_SCAN_INVARIANTS`）。
const DELETION_QUEUE_SCAN_INVARIANTS: [&str; 3] = [
    DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
];
/// 各种判不了的理由里点得出是哪一形的那一段（`walk.rs` 的 `DeletionScanGap::reason`）。
const NOT_JUDGED_INODE_TREE_NAMED_BY_NO_SINGLE_DELETED_INODES_ENTRY: &str =
    "头 ID 指着它的 deleted_inodes 条目不是恰好一条";
const NOT_JUDGED_DELETED_INODES_HEAD_IDENTIFIER_NAMES_NO_SINGLE_INODE_TREE: &str =
    "deleted_inodes 条目偏移 116 的头 ID 配不上同一张树表里恰好一条 inode 树条目";
const NOT_JUDGED_EXTENT_TREE_OF_THE_INODE_TREE_NOT_SINGLE: &str =
    "头 ID 指着这棵 inode 树的 extent 树条目不是恰好一条";
const NOT_JUDGED_EXTENT_TREE_HEAD_IDENTIFIER_NAMES_NO_SINGLE_INODE_TREE: &str =
    "有一棵 extent 树，偏移 116 的头 ID 配不上同一张表里恰好一条 inode 树条目";
const NOT_JUDGED_EXTENT_TREE_UNREADABLE: &str = "extent 树读不全：那一版每个号的数据单元数判不了";
const NOT_JUDGED_ACCOUNTING_TREE_UNREADABLE: &str = "记账树读不全";
const NOT_JUDGED_DELETED_INODES_ROOT_UNREADABLE: &str = "deleted_inodes 根读不出";
const NOT_JUDGED_TREE_TABLE_UNREADABLE: &str = "树表读不出";
const NOT_JUDGED_TREE_TABLE_ENTRY_SHORTER_THAN_THE_FIELD_TABLE: &str =
    "树表条目宽不到字段表的 200 字节";
const NOT_JUDGED_TOMBSTONE_TREE_UNREADABLE: &str = "所选根的树表或墓碑树读不全";
const NOT_JUDGED_QUEUED_INODE_WITHOUT_A_RECORD: &str =
    "deleted_inodes 里有一个号在同一版 inode 树里没有记录";

/// 干净的那一版上一条违例都没有：坏镜像「只红自己」从这里比起。
fn assert_the_base_image_is_clean(image: &MemoryPool, base: &str) {
    assert_eq!(
        violated_invariants(image),
        Vec::<&str>::new(),
        "{base}那一版没有违例"
    );
}

/// I-9.5 正方向，只红自己的那一形（判决第二节最后一行）：空 inode 2 发完意图之后队列 {2} 改成 {7}，7 号在 inode 树里没有记录、
/// extent 树里也没有单元（队列只一个号，I-9.24 照旧成立；2 号 nlink 0 而不在队列里，反方向也红在 I-9.5 上）。
/// I-9.18（删除队列里的号有谱系上的墓碑） 对 7 号判不了、不判红（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.18 那一行：这一形只让 I-9.5 红）。
#[test]
fn a_queue_number_without_an_inode_record_reddens_only_the_deletion_queue_invariant() {
    let clean = image_after_the_intent_of_an_empty_inode("deletion-checker-queue-without-record");
    assert_the_base_image_is_clean(&clean, "空 inode 2 发完意图之后");
    let mut image = clean.clone();
    replace_the_queued_inode_with_a_number_without_a_record(&mut image);
    assert_eq!(
        violated_invariants(&image),
        vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
        "只红 I-9.5"
    );
    let detail = first_violation_of(
        &image,
        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    );
    assert!(
        detail.contains("deleted_inodes 里的号 7") && detail.contains("没有记录"),
        "违例说明点名队列里那个没有记录的号：{detail}"
    );
    let reason = reason_not_judged(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE);
    assert!(
        reason.contains(NOT_JUDGED_QUEUED_INODE_WITHOUT_A_RECORD),
        "I-9.18 对队列里没有记录的号判不了、不判红：{reason}"
    );
}

/// I-9.5 反方向（C708 那一形，也是判决里说的僵尸记录：nlink 0、单元全放、不在队列）：第一个文件之后建一个空 inode（2 号，没有数据单元），
/// 它的记录 nlink 1 → 0，队列不动。I-4.7 对它成立（一个数据单元都不剩），I-4.6 也成立（size 0、0 个单元）。
#[test]
fn an_inode_record_with_nlink_zero_outside_the_queue_reddens_only_the_deletion_queue_invariant() {
    let clean = pool_with_new_inodes("deletion-checker-zombie-record", 1).memory_pool();
    assert_the_base_image_is_clean(&clean, "建了一个空 inode 之后");
    let mut image = clean.clone();
    set_the_nlink_of_an_inode_record_to_zero(&mut image, 2);
    assert_eq!(
        violated_invariants(&image),
        vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
        "只红 I-9.5"
    );
    let detail = first_violation_of(
        &image,
        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    );
    assert!(
        detail.contains("inode 2 的记录 nlink = 0") && detail.contains("不在"),
        "违例说明点名不在队列里的那条记录：{detail}"
    );
}

/// 形式化接入那条线攻方的 `being-deleted` 那一形（`research/prompts/m3-formal-code-b21-r1-opus-model/out/k2a.txt` 里 `variant=being-deleted` 那一行：
/// 一次建 inode 的发布写出一条新记录 (3, 9, nlink 0, size 0)、号不在队列里，那时池级 checker 一条都不红；C708 的验收）：
/// 一次发布建两个空 inode（2、3 号），把这次新写出的 3 号记录 nlink 改成 0。池级 checker 在这一形上红 I-9.5。
#[test]
fn a_new_inode_published_with_nlink_zero_as_in_the_formal_line_being_deleted_variant_reddens_the_deletion_queue_invariant(
) {
    let clean = pool_with_new_inodes("deletion-checker-being-deleted-variant", 2).memory_pool();
    assert_the_base_image_is_clean(&clean, "一次发布建了两个空 inode 之后");
    let mut image = clean.clone();
    set_the_nlink_of_an_inode_record_to_zero(&mut image, 3);
    assert_eq!(
        violated_invariants(&image),
        vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
        "k2a 那一形红 I-9.5，别的不红"
    );
    let detail = first_violation_of(
        &image,
        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    );
    assert!(
        detail.contains("inode 3 的记录 nlink = 0"),
        "违例说明点名新写出的那条记录：{detail}"
    );
}

/// I-4.7 只红自己的那一形（判决第二节 F1：孤儿单元，今天的实现走不到，在 checker 侧直接改字节造）：第一个文件（1 号，一个内联数据单元）
/// 的记录从叶里拿掉、extent 树不动，号也不在队列里。I-9.5 两个方向都成立（队列空、2 号 nlink 1），I-4.6 没有对象。
#[test]
fn data_units_of_an_inode_without_a_record_outside_the_queue_redden_only_the_inode_being_deleted_invariant(
) {
    let clean = pool_with_new_inodes("deletion-checker-orphan-data-units", 1).memory_pool();
    assert_the_base_image_is_clean(&clean, "建了一个空 inode 之后");
    let mut image = clean.clone();
    drop_the_record_of_the_first_file_and_keep_its_data_units(&mut image);
    assert_eq!(
        violated_invariants(&image),
        vec![INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT],
        "只红 I-4.7"
    );
    let detail = first_violation_of(
        &image,
        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    );
    assert!(
        detail.contains("inode 1 处于删除中（inode 树里没有它的记录）")
            && detail.contains("还有 1 个数据单元"),
        "违例说明点名那个号、为什么算删除中与剩几个单元：{detail}"
    );
}

/// I-4.6 只红自己的那一形：意图与第一批之后（inode 1 nlink 0、size 2 个净荷、剩两个单元）把 size 多加一个净荷容量、blocks 跟着改，
/// 单元数不动。I-9.5、I-4.7 照旧成立（队列 {1}、nlink 0）。
#[test]
fn an_inode_being_deleted_whose_size_asks_for_more_data_units_than_it_has_reddens_only_the_intermediate_state_invariant(
) {
    let clean =
        image_after_the_first_batch_of_a_three_unit_file("deletion-checker-size-ahead-of-units");
    assert_the_base_image_is_clean(&clean, "意图与第一批之后");
    let mut image = clean.clone();
    raise_the_size_of_the_first_file_by_one_data_unit_payload(&mut image);
    assert_eq!(
        violated_invariants(&image),
        vec![DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE],
        "只红 I-4.6"
    );
    let detail = first_violation_of(
        &image,
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    );
    assert!(
        detail.contains("要 3 个数据单元") && detail.contains("extent 树里有 2 个"),
        "违例说明写出 size 要几个、实有几个：{detail}"
    );
}

/// I-3.4：写完新池新建文件那一版记账树里待删占用那一行 0 → 1。别的条不读标签 4 的值（I-3.13 只比代段）。
#[test]
fn a_pending_deletion_occupancy_row_that_is_not_zero_reddens_only_the_pending_deletion_invariant() {
    let clean = build_pool("deletion-checker-pending-row-not-zero").memory_pool();
    assert_the_base_image_is_clean(&clean, "写完新池新建文件");
    let mut image = clean.clone();
    set_the_pending_deletion_occupancy_row_to_one(&mut image);
    assert_eq!(
        violated_invariants(&image),
        vec![PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO],
        "只红 I-3.4"
    );
    let detail = first_violation_of(&image, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO);
    assert!(
        detail.contains("的值是 1"),
        "违例说明写出那一行的值：{detail}"
    );
}

/// I-3.4 缺行判红（判决第二节 F5-b）：写完新池新建文件那一版记账树里拿掉待删占用那一行，别的行一个字节都不动。
#[test]
fn an_accounting_tree_without_the_pending_deletion_occupancy_row_reddens_only_the_pending_deletion_invariant(
) {
    let clean = build_pool("deletion-checker-pending-row-missing").memory_pool();
    assert_the_base_image_is_clean(&clean, "写完新池新建文件");
    let mut image = clean.clone();
    drop_the_pending_deletion_occupancy_row(&mut image);
    assert_eq!(
        violated_invariants(&image),
        vec![PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO],
        "只红 I-3.4"
    );
    let detail = first_violation_of(&image, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO);
    assert!(
        detail.contains("没有待删除但意图未完成占用"),
        "违例说明写出缺的那一行：{detail}"
    );
}

/// I-9.6 墓碑那一半（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.6 那一行：水位 > 所选根的墓碑树里全部已发布的墓碑记录的对象 ID）：
/// 3 号删完、根环转过一圈之后那条墓碑的对象 ID 3 → 记账里的 inode 号水位，inode 树里最大的 key 仍小于水位——只判「水位 > 树内最大 key」
/// 那一半时它全绿。在意图那一版上改会连带 I-9.18（意图那一版的队列 {3} 找不到 3 号的墓碑），所以等意图那一版转出候选集再改
/// （判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T9、攻方第五节的造法）。
#[test]
fn a_tombstone_whose_object_identifier_reaches_the_inode_number_watermark_reddens_only_the_watermark_invariant(
) {
    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
        "deletion-checker-tombstone-at-watermark",
    );
    let inode_number_watermark = pool.output.inode_number_watermark();
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
    let mut image = clean.clone();
    raise_the_tombstone_object_identifier_to_the_inode_number_watermark(
        &mut image,
        inode_number_watermark,
    );
    assert_eq!(violated_invariants(&image), vec!["I-9.6"], "只红 I-9.6");
    let detail = first_violation_of(&image, "I-9.6");
    assert!(
        detail.contains(&format!("墓碑记录的最大对象 ID {inode_number_watermark}")),
        "违例说明写出墓碑那一半：{detail}"
    );
}

/// 判决第二节 F4-a / F5-c：判回退候选集里每一版，不只判最新根。三个单元的文件删到「意图、留 2、留 1」，
/// 只把中间那一版（留 2 那一批）记账树里待删占用那一行改成 1；最新的根（留 1）那一版是对的——只判最新根会漏这一形（攻方量过）。
#[test]
fn an_intermediate_deletion_batch_with_a_non_zero_pending_deletion_row_reddens_while_the_newest_root_is_clean(
) {
    let mut pool = pool_with_a_three_unit_file("deletion-checker-intermediate-pending-row");
    publish_the_intent(&mut pool);
    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
    let newest = publish_the_next_step_in_batches_of_one(&mut pool);
    let intermediate_accounting_slot = intermediate.unit(TransactionUnit::AccountingTree).slot;
    assert_ne!(
        intermediate_accounting_slot,
        newest.unit(TransactionUnit::AccountingTree).slot,
        "每一版重写记账树：中间那一版的记账树只有它自己引用"
    );
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "删到留 1");
    let mut image = clean.clone();
    change_the_pending_deletion_occupancy_row_of_the_accounting_tree_at(
        &mut image,
        intermediate_accounting_slot,
        Some(1),
    );
    assert_eq!(
        violated_invariants(&image),
        vec![PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO],
        "中间那一版写坏、最新根写对：只红 I-3.4"
    );
    let detail = first_violation_of(&image, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO);
    assert!(
        detail.contains(&format!("txg {}）", intermediate.root.checkpoint_txg.0)),
        "违例说明点名中间那一版：{detail}"
    );
}

/// 同上一形，判的是 I-4.6：只把中间那一版（留 2 那一批）的 inode 叶里 1 号的 size 多加一个净荷容量，最新的根（留 1）那一版是对的。
#[test]
fn an_intermediate_deletion_batch_whose_size_asks_for_more_data_units_reddens_while_the_newest_root_is_clean(
) {
    let mut pool = pool_with_a_three_unit_file("deletion-checker-intermediate-size");
    publish_the_intent(&mut pool);
    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
    let newest = publish_the_next_step_in_batches_of_one(&mut pool);
    let leftmost_leaf =
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST);
    let intermediate_leaf_slot = intermediate.unit(leftmost_leaf).slot;
    assert_ne!(
        intermediate_leaf_slot,
        newest.unit(leftmost_leaf).slot,
        "每一批重写 inode 叶：中间那一版的叶只有它自己引用"
    );
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "删到留 1");
    let mut image = clean.clone();
    raise_the_size_of_the_first_file_in_the_leaf_at(&mut image, intermediate_leaf_slot);
    assert_eq!(
        violated_invariants(&image),
        vec![DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE],
        "中间那一版写坏、最新根写对：只红 I-4.6"
    );
    let detail = first_violation_of(
        &image,
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    );
    assert!(
        detail.contains(&format!("txg {}）", intermediate.root.checkpoint_txg.0)),
        "违例说明点名中间那一版：{detail}"
    );
}

/// 判决第二节 F4-c：读不全的那一版报射程里判不了（`part_of_the_range_not_judged`，没有违例时整条报不适用），不判红。
/// 中间那一版（留 2）的 inode 叶两份都盖坏、不补校验和：那一版的记录读不出，I-9.5、I-4.7、I-4.6 在那一版判不了；
/// 别的版本照判、都成立，整条报不适用并带「inode 树读不全」的理由。主走读那几条（I-2.1、I-7.4、I-4.8）照常因为这一片红。
#[test]
fn an_intermediate_version_whose_inode_leaf_does_not_read_back_is_not_judged_rather_than_red() {
    let mut pool = pool_with_a_three_unit_file("deletion-checker-intermediate-unreadable-leaf");
    publish_the_intent(&mut pool);
    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
    publish_the_next_step_in_batches_of_one(&mut pool);
    let leftmost_leaf =
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST);
    let intermediate_leaf_slot = intermediate.unit(leftmost_leaf).slot;
    let mut image = pool.memory_pool();
    let mut overwritten = unit_bytes_at(&image, intermediate_leaf_slot, data_unit_bytes());
    overwritten[1000] ^= 0xFF;
    for device in image.devices.values_mut() {
        device.write(intermediate_leaf_slot.to_device_offset(), &overwritten);
    }
    for invariant in [
        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    ] {
        let found = verdict_of(&image, invariant);
        assert!(
            matches!(&found, InvariantVerdict::NotApplicable(reason) if reason.contains("inode 树读不全")),
            "{invariant} 在读不全的那一版上判不了、不判红，实际 {found:?}"
        );
    }
}

/// 阳性对照：三个单元的文件按批大小 1 删完（意图、留 2、留 1、最后一批），每一步的镜像上一条违例都没有；
/// I-9.5、I-3.4、I-4.7、I-4.6 真被评估过且成立（最后一批之后，回退候选集里意图与各批那几版还在，删除中的号仍有对象），
/// 墓碑与谱系那几条（I-9.10 墓碑那一个等号、I-9.18、I-9.19、I-9.20、I-9.23、I-9.24、I-9.29）也真被评估过且成立，
/// I-4.4 报不适用、理由照 `crates/singlefs-checker/invariants/06-事务与崩溃一致性.md` I-4.4 那一行。
#[test]
fn every_published_step_of_unlinking_a_three_unit_file_holds_the_deletion_invariants() {
    let mut pool = pool_with_a_three_unit_file("deletion-checker-every-step-holds");
    publish_the_intent(&mut pool);
    let mut images_of_the_steps = vec![("意图", pool.memory_pool())];
    publish_the_next_step_in_batches_of_one(&mut pool);
    images_of_the_steps.push(("留 2", pool.memory_pool()));
    publish_the_next_step_in_batches_of_one(&mut pool);
    images_of_the_steps.push(("留 1", pool.memory_pool()));
    publish_the_next_step_in_batches_of_one(&mut pool);
    images_of_the_steps.push(("最后一批", pool.memory_pool()));
    for (step, image) in &images_of_the_steps {
        assert_eq!(
            violated_invariants(image),
            Vec::<&str>::new(),
            "{step}之后没有违例"
        );
        for invariant in [
            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
            PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO,
            INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
            "I-9.6",
            "I-9.10",
            QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
            TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
            TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
            TOMBSTONE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
            DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY,
            ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
        ] {
            assert_eq!(
                verdict_of(image, invariant),
                InvariantVerdict::Holds,
                "{step}之后 {invariant} 真被评估过且成立"
            );
        }
        assert_eq!(
            verdict_of(image, INTENT_DESCRIBES_ONLY_THE_TARGET_STATE),
            InvariantVerdict::NotApplicable(
                NO_INTENT_ENTRY_IN_THE_LOGGED_OPERATIONS_OF_ANY_WALKED_VERSION
            ),
            "{step}之后 I-4.4 报第一版不适用"
        );
    }
}

/// L1a（代码轮判决 `research/prompts/m3-batch18-code-r1-main-verification.md` 第二节「K2 漏判 L1」，攻方报告第二节）：意图之后那一版树表里
/// `deleted_inodes` 那一条的种类码 9 改成没登记的 16——那一版的 inode 树（1 号 nlink 0、三个单元）没有任何 `deleted_inodes` 条目指着它。
/// 改法 A 之前只从 `deleted_inodes` 条目一侧去配，这一版一次都不判、也不记判不了，I-9.5 报成立；现在三条在那一版判不了、不报成立。
/// 判红的只有 I-9.16（树表里十一种各恰好一条，④）。
#[test]
fn an_inode_tree_whose_deleted_inodes_entry_carries_an_unregistered_kind_leaves_the_deletion_queue_invariants_not_judged(
) {
    let clean =
        image_after_the_intent_of_a_three_unit_file("deletion-checker-l1a-unregistered-kind");
    assert_the_base_image_is_clean(&clean, "意图之后");
    let mut image = clean.clone();
    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(
        &mut image,
        tree_table_slot,
    );
    assert_eq!(
        violated_invariants(&image),
        vec!["I-9.16"],
        "判红的只有树表种类那一条"
    );
    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
        let reason = reason_not_judged(&image, invariant);
        assert!(
            reason.contains(NOT_JUDGED_INODE_TREE_NAMED_BY_NO_SINGLE_DELETED_INODES_ENTRY),
            "{invariant} 在没有 deleted_inodes 条目指着 inode 树的那一版判不了：{reason}"
        );
    }
}

/// L1b：同上，`deleted_inodes` 那一条整条从意图之后那一版的树表里拿掉（树表 10 条）。
#[test]
fn an_inode_tree_without_a_deleted_inodes_entry_in_the_tree_table_leaves_the_deletion_queue_invariants_not_judged(
) {
    let clean = image_after_the_intent_of_a_three_unit_file(
        "deletion-checker-l1b-deleted-inodes-entry-removed",
    );
    assert_the_base_image_is_clean(&clean, "意图之后");
    let mut image = clean.clone();
    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
    remove_the_deleted_inodes_entry_from_the_tree_table_at(&mut image, tree_table_slot);
    assert_eq!(
        violated_invariants(&image),
        vec!["I-9.16"],
        "判红的只有树表种类那一条"
    );
    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
        let reason = reason_not_judged(&image, invariant);
        assert!(
            reason.contains(NOT_JUDGED_INODE_TREE_NAMED_BY_NO_SINGLE_DELETED_INODES_ENTRY),
            "{invariant} 在没有 deleted_inodes 条目指着 inode 树的那一版判不了：{reason}"
        );
    }
}

/// L1c：三个单元的文件删到「意图、留 2、留 1」，只把中间那一版（留 2）树表里 `deleted_inodes` 那一条的种类码改成 13，最新的根写对。
/// 中间那一版 1 号 nlink 0、还有两个单元、不在任何队列里；改法 A 之前 I-9.5、I-4.7、I-4.6 都由别的版本判成立（攻方量过）。
#[test]
fn an_intermediate_version_without_a_deleted_inodes_entry_leaves_the_deletion_queue_invariants_not_judged_while_the_newest_root_is_clean(
) {
    let mut pool =
        pool_with_a_three_unit_file("deletion-checker-l1c-intermediate-unregistered-kind");
    publish_the_intent(&mut pool);
    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
    let newest = publish_the_next_step_in_batches_of_one(&mut pool);
    let intermediate_tree_table_slot = intermediate.unit(TransactionUnit::TreeTable).slot;
    assert_ne!(
        intermediate_tree_table_slot,
        newest.unit(TransactionUnit::TreeTable).slot,
        "每一版重写树表：中间那一版的树表只有它自己引用"
    );
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "删到留 1");
    let mut image = clean.clone();
    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(
        &mut image,
        intermediate_tree_table_slot,
    );
    assert_eq!(
        violated_invariants(&image),
        vec!["I-9.16"],
        "判红的只有树表种类那一条"
    );
    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
        let reason = reason_not_judged(&image, invariant);
        assert!(
            reason.contains(NOT_JUDGED_INODE_TREE_NAMED_BY_NO_SINGLE_DELETED_INODES_ENTRY),
            "{invariant} 在中间那一版判不了、不由别的版本判成立：{reason}"
        );
    }
}

/// L2（攻方报告第二节）：L1a 之上把 1 号的 size 多加一个净荷容量（要 4 个单元、有 3 个）。改法 A 之前 I-4.6 报不适用，理由
/// 「走读到的每一版 inode 树里都没有 nlink = 0 的记录」与镜像不符（那一版 1 号 nlink 0）；现在那一版判不了，理由写的是配不上。
#[test]
fn a_size_ahead_of_the_data_units_on_a_version_without_a_deleted_inodes_entry_is_not_judged_with_the_reason_of_the_missing_entry(
) {
    let clean = image_after_the_intent_of_a_three_unit_file("deletion-checker-l2-size-ahead");
    assert_the_base_image_is_clean(&clean, "意图之后");
    let mut image = clean.clone();
    raise_the_size_of_the_first_file_by_one_data_unit_payload(&mut image);
    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(
        &mut image,
        tree_table_slot,
    );
    let reason = reason_not_judged(
        &image,
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    );
    assert!(
        reason.contains(NOT_JUDGED_INODE_TREE_NAMED_BY_NO_SINGLE_DELETED_INODES_ENTRY),
        "I-4.6 在那一版判不了，理由写配不上：{reason}"
    );
    assert!(
        !reason.contains("没有 nlink = 0 的记录"),
        "理由不写镜像里不成立的事（那一版 1 号 nlink = 0）：{reason}"
    );
}

/// `DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree`（攻方 L1d 那一形）：意图之后那一版树表里 `deleted_inodes` 那一条的
/// 头 ID 从 inode 树的树 ID 改成 0，配不上任何一条 inode 树条目。三条在那一版判不了，理由先记头 ID 配不上那一句；I-9.27 判红。
#[test]
fn a_deleted_inodes_entry_whose_head_identifier_names_no_inode_tree_leaves_the_deletion_queue_invariants_not_judged(
) {
    let clean =
        image_after_the_intent_of_a_three_unit_file("deletion-checker-deleted-inodes-head-zero");
    assert_the_base_image_is_clean(&clean, "意图之后");
    let mut image = clean.clone();
    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
    set_the_head_identifier_of_the_tree_table_entry_of_kind_at(
        &mut image,
        tree_table_slot,
        TREE_KIND_DELETED_INODES,
        0,
    );
    assert_eq!(
        violated_invariants(&image),
        vec!["I-9.27"],
        "判红的只有头 ID 取值那一条"
    );
    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
        let reason = reason_not_judged(&image, invariant);
        assert!(
            reason.contains(NOT_JUDGED_DELETED_INODES_HEAD_IDENTIFIER_NAMES_NO_SINGLE_INODE_TREE),
            "{invariant} 在头 ID 配不上的那一版判不了：{reason}"
        );
    }
}

/// `DeletionScanGap::ExtentTreeOfTheInodeTreeNotSingle`：意图之后那一版树表里 extent 树那一条的头 ID 从 inode 树的树 ID 改成 0。
/// I-9.5 不读 extent 树，照判、成立；I-4.7、I-4.6 在那一版判不了。I-9.27 判红。
#[test]
fn an_extent_tree_entry_whose_head_identifier_names_no_inode_tree_leaves_the_data_unit_invariants_not_judged(
) {
    let clean = image_after_the_intent_of_a_three_unit_file("deletion-checker-extent-head-zero");
    assert_the_base_image_is_clean(&clean, "意图之后");
    let mut image = clean.clone();
    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
    set_the_head_identifier_of_the_tree_table_entry_of_kind_at(
        &mut image,
        tree_table_slot,
        TREE_KIND_EXTENT,
        0,
    );
    assert_eq!(
        violated_invariants(&image),
        vec!["I-9.27"],
        "判红的只有头 ID 取值那一条"
    );
    assert_eq!(
        verdict_of(
            &image,
            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES
        ),
        InvariantVerdict::Holds,
        "I-9.5 不读 extent 树，照判、成立"
    );
    for invariant in [
        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    ] {
        let reason = reason_not_judged(&image, invariant);
        assert!(
            reason.contains(NOT_JUDGED_EXTENT_TREE_OF_THE_INODE_TREE_NOT_SINGLE),
            "{invariant} 在 extent 树配不上的那一版判不了：{reason}"
        );
    }
}

/// S1（代码轮判决 `research/prompts/m3-batch20-code-r1-main-verification.md` 第二节，攻方报告第二节 2.2）：I-4.7 那一形坏镜像
/// （1 号记录拿掉、单元留着、号不在队列）之上，最新那一版树表里空根的 livelist 条目改成种类 1、头 ID 指 inode 树，真 extent 树头 ID 改 0。
/// 只从 inode 树一侧配 extent 树时配上的是那条空根，I-4.7 报「没有处于删除中的 inode」，而真 extent 树里 1 号还有一个单元；
/// extent 树也反方向配之后，真 extent 树配不上 inode 树，I-4.7、I-4.6 在那一版判不了。判红的只有 I-9.16（种类 1 两条）与 I-9.27（头 ID 取值）。
#[test]
fn an_empty_entry_standing_in_for_the_extent_tree_while_the_real_extent_tree_names_no_inode_tree_leaves_the_data_unit_invariants_not_judged(
) {
    let clean =
        pool_with_new_inodes("deletion-checker-s1-empty-entry-as-extent-tree", 1).memory_pool();
    assert_the_base_image_is_clean(&clean, "建了一个空 inode 之后");
    let reason_on_the_legal_image = reason_not_judged(
        &clean,
        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    );
    assert!(
        !reason_on_the_legal_image
            .contains(NOT_JUDGED_EXTENT_TREE_HEAD_IDENTIFIER_NAMES_NO_SINGLE_INODE_TREE),
        "I-4.7 在合法镜像上照旧报没有对象，不报 extent 树配不上：{reason_on_the_legal_image}"
    );
    // I-4.6 自截断起每条记录都判（m4-trunc-r1 判决 K7 T-3）：空 inode 的记录 size 0、没有单元，在合法镜像上真被评估过且成立，
    // 说明 extent 树从 inode 树一侧配上了。
    assert_eq!(
        verdict_of(
            &clean,
            DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE
        ),
        InvariantVerdict::Holds,
        "I-4.6 在合法镜像上成立，不报 extent 树配不上"
    );
    let mut image = clean.clone();
    drop_the_record_of_the_first_file_and_keep_its_data_units(&mut image);
    let tree_table_slot = tree_table_slot_of_the_newest_root(&image);
    let_the_empty_livelist_entry_stand_in_for_the_extent_tree_in_the_tree_table_at(
        &mut image,
        tree_table_slot,
    );
    assert_eq!(
        violated_invariants(&image),
        vec!["I-9.16", "I-9.27"],
        "判红的只有树表种类与头 ID 取值那两条"
    );
    assert_eq!(
        verdict_of(
            &image,
            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES
        ),
        InvariantVerdict::Holds,
        "I-9.5 不读 extent 树，照判、成立"
    );
    for invariant in [
        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    ] {
        let reason = reason_not_judged(&image, invariant);
        assert!(
            reason.contains(NOT_JUDGED_EXTENT_TREE_HEAD_IDENTIFIER_NAMES_NO_SINGLE_INODE_TREE),
            "{invariant} 在真 extent 树配不上 inode 树的那一版判不了，不报没有对象：{reason}"
        );
    }
}

/// `DeletionScanGap::AccountingTreeUnreadable`：写完新池新建文件那一版记账树根（根兼叶）两份全零（不补引用链）。
/// I-3.4 在那一版判不了：不判红、不报成立（更早的版本照判、成立，也压不过这一版的判不了）。
#[test]
fn an_accounting_tree_that_does_not_read_back_leaves_the_pending_deletion_invariant_not_judged() {
    let mut image = build_pool("deletion-checker-accounting-root-zeroed").memory_pool();
    let accounting_root_slot = accounting_tree_slot_of_the_newest_root(&image);
    zero_both_copies_of_the_unit_at(&mut image, accounting_root_slot, node_bytes());
    let reason = reason_not_judged(&image, PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO);
    assert!(
        reason.contains(NOT_JUDGED_ACCOUNTING_TREE_UNREADABLE),
        "I-3.4 在记账树读不全的那一版判不了、不判红：{reason}"
    );
}

/// `DeletionScanGap::ExtentTreeUnreadable`：意图之后那一版 extent 树根两份全零（不补引用链）。I-9.5 照判、成立；I-4.7、I-4.6 判不了。
#[test]
fn an_extent_tree_that_does_not_read_back_leaves_the_data_unit_invariants_of_the_deletion_not_judged(
) {
    let mut image =
        image_after_the_intent_of_a_three_unit_file("deletion-checker-extent-root-zeroed");
    let extent_root_slot =
        root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_EXTENT);
    zero_both_copies_of_the_unit_at(&mut image, extent_root_slot, node_bytes());
    assert_eq!(
        verdict_of(
            &image,
            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES
        ),
        InvariantVerdict::Holds,
        "I-9.5 不读 extent 树，照判、成立"
    );
    for invariant in [
        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
        DATA_UNIT_COUNT_OF_AN_INODE_BEING_DELETED_MATCHES_ITS_SIZE,
    ] {
        let reason = reason_not_judged(&image, invariant);
        assert!(
            reason.contains(NOT_JUDGED_EXTENT_TREE_UNREADABLE),
            "{invariant} 在 extent 树读不全的那一版判不了、不判红：{reason}"
        );
    }
}

/// `DeletionScanGap::DeletedInodesRootUnreadable`：意图之后那一版 `deleted_inodes` 根（队列 {1}）两份全零（不补引用链）。三条判不了、不判红。
#[test]
fn a_deleted_inodes_root_that_does_not_read_back_leaves_the_deletion_queue_invariants_not_judged() {
    let mut image =
        image_after_the_intent_of_a_three_unit_file("deletion-checker-deleted-inodes-root-zeroed");
    let deleted_inodes_root_slot =
        root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_DELETED_INODES);
    zero_both_copies_of_the_unit_at(&mut image, deleted_inodes_root_slot, node_bytes());
    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
        let reason = reason_not_judged(&image, invariant);
        assert!(
            reason.contains(NOT_JUDGED_DELETED_INODES_ROOT_UNREADABLE),
            "{invariant} 在队列读不出的那一版判不了、不判红：{reason}"
        );
    }
}

/// `DeletionScanGap::TreeTableUnreadable`：三个单元的文件删到「意图、留 2、留 1」，中间那一版（留 2）的树表两份全零（不补引用链）。
/// 那一版的删除态四条（I-9.5、I-4.7、I-4.6、I-3.4）判不了、不判红；别的版本照判。
#[test]
fn an_intermediate_tree_table_that_does_not_read_back_leaves_the_deletion_invariants_not_judged() {
    let mut pool = pool_with_a_three_unit_file("deletion-checker-intermediate-tree-table-zeroed");
    publish_the_intent(&mut pool);
    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
    publish_the_next_step_in_batches_of_one(&mut pool);
    let mut image = pool.memory_pool();
    zero_both_copies_of_the_unit_at(
        &mut image,
        intermediate.unit(TransactionUnit::TreeTable).slot,
        node_bytes(),
    );
    for invariant in DELETION_QUEUE_SCAN_INVARIANTS
        .into_iter()
        .chain([PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO])
    {
        let reason = reason_not_judged(&image, invariant);
        assert!(
            reason.contains(NOT_JUDGED_TREE_TABLE_UNREADABLE),
            "{invariant} 在树表读不出的那一版判不了、不判红：{reason}"
        );
    }
}

/// `DeletionScanGap::TreeTableEntryShorterThanTheFieldTable`：同上那三步，中间那一版（留 2）的树表条目宽收窄到 key 宽 8（只留第一条、
/// 别的自描述字段自洽，重封、补引用链）。那一版的删除态四条判不了、不判红。
#[test]
fn an_intermediate_tree_table_whose_entries_are_narrower_than_the_field_table_leaves_the_deletion_invariants_not_judged(
) {
    let mut pool = pool_with_a_three_unit_file("deletion-checker-intermediate-tree-table-narrow");
    publish_the_intent(&mut pool);
    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
    publish_the_next_step_in_batches_of_one(&mut pool);
    let mut image = pool.memory_pool();
    narrow_the_entries_of_the_tree_table_at_to_the_key_width(
        &mut image,
        intermediate.unit(TransactionUnit::TreeTable).slot,
    );
    for invariant in DELETION_QUEUE_SCAN_INVARIANTS
        .into_iter()
        .chain([PENDING_DELETION_OCCUPANCY_STATISTIC_IS_ZERO])
    {
        let reason = reason_not_judged(&image, invariant);
        assert!(
            reason.contains(NOT_JUDGED_TREE_TABLE_ENTRY_SHORTER_THAN_THE_FIELD_TABLE),
            "{invariant} 在树表条目窄的那一版判不了、不判红：{reason}"
        );
    }
}

/// `DeletionScanGap::TombstoneTreeUnreadable`：意图之后所选根墓碑树里那片墓碑容器两份全零（不补引用链）。I-9.6 前一半（水位 > 树内最大 key）
/// 照判、成立，墓碑那一半判不了：整条报不适用带墓碑树读不全的理由，不报成立。
#[test]
fn a_tombstone_container_that_does_not_read_back_leaves_the_watermark_invariant_not_judged() {
    let mut image =
        image_after_the_intent_of_a_three_unit_file("deletion-checker-tombstone-container-zeroed");
    let container_slot = first_tombstone_container_slot_of_the_newest_root(&image);
    zero_both_copies_of_the_unit_at(&mut image, container_slot, data_unit_bytes());
    let reason = reason_not_judged(&image, "I-9.6");
    assert!(
        reason.contains(NOT_JUDGED_TOMBSTONE_TREE_UNREADABLE),
        "I-9.6 墓碑那一半判不了、不报成立：{reason}"
    );
}

/// I-9.6 墓碑那一半「已发布」那一支（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.6 那一行：只算已发布的墓碑）：意图之后那片墓碑容器头里写序的实例代号
/// 1 → 2（大于挂载根的实例，已发布谓词判它不是已发布），墓碑对象 ID 也抬到水位 2。这片容器不算进墓碑那一半，I-9.6 照旧成立
/// （对照：只抬对象 ID、不改实例的那一份红 I-9.6，见 `a_tombstone_whose_object_identifier_reaches_the_inode_number_watermark_reddens_only_the_watermark_invariant`）。
#[test]
fn a_tombstone_container_outside_the_published_predicate_does_not_count_toward_the_watermark() {
    let clean =
        image_after_the_intent_of_a_three_unit_file("deletion-checker-tombstone-not-published");
    assert_the_base_image_is_clean(&clean, "意图之后");
    let mut image = clean.clone();
    move_the_tombstone_container_to_an_unpublished_instance_and_raise_its_object_identifier(
        &mut image,
    );
    assert_eq!(
        verdict_of(&image, "I-9.6"),
        InvariantVerdict::Holds,
        "没发布的墓碑容器不算进墓碑那一半：I-9.6 成立"
    );
}

/// I-4.6 size = 0 那一支（`crates/singlefs-checker/invariants/06-事务与崩溃一致性.md` I-4.6 那一行：size 为 0 时 0 或 1 个单元）：第一个文件顺序写成空内容（size 0、一个数据单元，
/// 长度 0 的顺序写照样写一个单元），再发 unlink 的意图——1 号 nlink 0、size 0、一个单元，是发布路径自己写出来的合法中间态。
#[test]
fn an_inode_being_deleted_with_size_zero_and_one_data_unit_holds_the_intermediate_state_invariant()
{
    let mut pool = pool_with_a_first_file_of_size_zero_and_one_data_unit(
        "deletion-checker-size-zero-one-unit",
    );
    publish_the_intent(&mut pool);
    let image = pool.memory_pool();
    assert_eq!(
        violated_invariants(&image),
        Vec::<&str>::new(),
        "size 0、一个单元的文件发完意图之后没有违例"
    );
    for invariant in DELETION_QUEUE_SCAN_INVARIANTS {
        assert_eq!(
            verdict_of(&image, invariant),
            InvariantVerdict::Holds,
            "{invariant} 真被评估过且成立"
        );
    }
}

/// 判决第二节 F4-a：判回退候选集里每一版，不只判最新根；判的是 I-9.5。第一个文件之后两次各建一个空 inode（2 号、3 号），
/// 只把中间那一版（建 2 号那次）inode 叶里 2 号的 nlink 改成 0，最新的根那一版是对的。
#[test]
fn an_intermediate_version_with_an_inode_record_of_nlink_zero_outside_the_queue_reddens_while_the_newest_root_is_clean(
) {
    let mut pool = build_pool("deletion-checker-intermediate-zombie-record");
    let intermediate = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let newest = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let leftmost_leaf =
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST);
    let intermediate_leaf_slot = intermediate.unit(leftmost_leaf).slot;
    assert_ne!(
        intermediate_leaf_slot,
        newest.unit(leftmost_leaf).slot,
        "每次建 inode 重写 inode 叶：中间那一版的叶只有它自己引用"
    );
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "建了两次空 inode");
    let mut image = clean.clone();
    set_the_nlink_of_an_inode_record_in_the_leaf_at_to_zero(&mut image, intermediate_leaf_slot, 2);
    assert_eq!(
        violated_invariants(&image),
        vec![DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES],
        "中间那一版写坏、最新根写对：只红 I-9.5"
    );
    let detail = first_violation_of(
        &image,
        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    );
    assert!(
        detail.contains(&format!("txg {}）", intermediate.root.checkpoint_txg.0))
            && detail.contains("inode 2 的记录 nlink = 0"),
        "违例说明点名中间那一版与那条记录：{detail}"
    );
}

/// 同上，判的是 I-4.7：只把中间那一版（建 2 号那次）inode 叶里 1 号的记录拿掉、extent 树不动，最新的根那一版是对的。
#[test]
fn an_intermediate_version_with_data_units_of_an_inode_without_a_record_reddens_while_the_newest_root_is_clean(
) {
    let mut pool = build_pool("deletion-checker-intermediate-orphan-data-units");
    let intermediate = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let newest = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    let leftmost_leaf =
        TransactionUnit::InodeLeafContainer(InodeLeafContainerIndexInTree::LEFTMOST);
    let intermediate_leaf_slot = intermediate.unit(leftmost_leaf).slot;
    assert_ne!(
        intermediate_leaf_slot,
        newest.unit(leftmost_leaf).slot,
        "每次建 inode 重写 inode 叶：中间那一版的叶只有它自己引用"
    );
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "建了两次空 inode");
    let mut image = clean.clone();
    drop_the_record_of_the_first_file_in_the_leaf_at_and_keep_its_data_units(
        &mut image,
        intermediate_leaf_slot,
    );
    // 中间那一版的 inode 树根头里的最小 key 跟着改成 2（子树覆盖区间，C478）：只红 I-4.7，不连带 I-1.1。
    set_the_smallest_key_of_the_inode_root_at(
        &mut image,
        intermediate.unit(TransactionUnit::InodeRoot).slot,
        FIRST_INODE_NUMBER + 1,
    );
    assert_eq!(
        violated_invariants(&image),
        vec![INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT],
        "中间那一版写坏、最新根写对：只红 I-4.7"
    );
    let detail = first_violation_of(
        &image,
        INODE_BEING_DELETED_IS_QUEUED_OR_HAS_NO_DATA_UNIT_LEFT,
    );
    assert!(
        detail.contains(&format!("txg {}）", intermediate.root.checkpoint_txg.0))
            && detail.contains("inode 1 处于删除中（inode 树里没有它的记录）"),
        "违例说明点名中间那一版与那个号：{detail}"
    );
}

/// C708（nlink 为 0 却不在删除队列，checker 判不出） 还差的那一形：I-9.5 正方向「队列里的号在树里 nlink ≠ 0」。
/// 意图之后队列 {1} 不动，1 号那条记录的 nlink 0 → 1。按条款同红 I-9.19（谱系上的墓碑不指活对象）：墓碑 (1, 1 号的出生代) 在谱系上而那条记录
/// nlink 1（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T9 那一行）；I-9.18 只要有那条墓碑、不看 nlink，照旧成立。
/// I-1.13 也红：删除中要两侧那条记录 nlink 都是 0（判决 `research/prompts/m4-c860-r1-main-verification.md` 第三节 C2，用户 2026-10-20 选丙），
/// 这条 nlink 1 的记录不算删除中，扫描重建按墓碑不认领 1 号的数据单元、遍历方向照收。
#[test]
fn a_queue_number_whose_inode_record_has_nlink_one_reddens_the_deletion_queue_invariant() {
    let clean =
        image_after_the_intent_of_a_three_unit_file("deletion-checker-queued-record-of-nlink-one");
    assert_the_base_image_is_clean(&clean, "意图之后");
    let mut image = clean.clone();
    set_the_nlink_of_the_queued_first_file_back_to_one(&mut image);
    let violated = violated_invariants(&image);
    assert!(
        violated.contains(&DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES),
        "I-9.5 在判红集合里：{violated:?}"
    );
    assert_eq!(
        violated,
        vec![
            DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
            TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT,
            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX
        ],
        "判红的是 I-9.5、I-9.19 与 I-1.13。原先按「只凭扫描一侧认删除中」写死的只有前两条：墓碑压着、记录还在就两侧都摘掉 1 号；\
         用户 2026-10-20 选丙（判决 `research/prompts/m4-c860-r1-main-verification.md` 第三节 C2）之后删除中要两侧那条记录 nlink 都是 0，\
         1 号这条 nlink 1 的活记录压着墓碑，不算删除中、两侧照比，它的三个数据单元扫描重建那一侧按墓碑不认领、遍历方向照收，I-1.13 跟着红"
    );
    let live_object_detail =
        first_violation_of(&image, TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT);
    assert!(
        live_object_detail.contains("号 1") && live_object_detail.contains("nlink 1 的记录"),
        "I-9.19 的说明点名那条墓碑与活着的记录：{live_object_detail}"
    );
    let detail = first_violation_of(
        &image,
        DELETION_QUEUE_AND_THE_NLINK_ZERO_RECORDS_NAME_THE_SAME_INODES,
    );
    assert!(
        detail.contains("deleted_inodes 里的号 1") && detail.contains("nlink = 1，不是 0"),
        "违例说明点名队列里那个号与它的 nlink：{detail}"
    );
}

/// I-9.24（删除队列至多一条 key） 只红自己的那一形（判决 `research/prompts/m3-delete-checker-r2-main-verification.md` 第二节 T8）：
/// 一次建了两个空 inode（2、3 号）、发完 2 号的意图之后，把 3 号也做成一个合法的删除中号——队列 {2, 3}、3 号记录 nlink 0、墓碑容器里
/// 追加 3 号的整对象墓碑（死于意图那次）。I-9.5 两个方向、I-4.7、I-4.6、I-9.18、I-9.19、I-9.20 都成立，只有队列多于一条 key。
/// 干净的那一版（队列 {2}）I-9.24 真被评估过且成立。
#[test]
fn a_deletion_queue_of_two_inodes_being_deleted_with_their_tombstones_reddens_only_the_at_most_one_key_invariant(
) {
    let mut pool = pool_with_new_inodes("deletion-checker-two-keys-in-the-queue", 2);
    publish_the_intent_of(&mut pool, 2);
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "一次建了两个空 inode、发完 2 号的意图之后");
    assert_eq!(
        verdict_of(&clean, DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY),
        InvariantVerdict::Holds,
        "队列 {{2}} 的那一版 I-9.24 真被评估过且成立"
    );
    let mut image = clean.clone();
    put_a_second_inode_being_deleted_into_the_deletion_queue(&mut image);
    assert_eq!(
        violated_invariants(&image),
        vec![DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY],
        "只红 I-9.24"
    );
    let detail = first_violation_of(&image, DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY);
    assert!(
        detail.contains("有 2 条 key"),
        "违例说明写出队列里有几条 key：{detail}"
    );
}

/// I-9.19（谱系上的墓碑不指活对象） 只红自己的那一形（攻方 `research/prompts/m3-delete-checker-r2-opus-output.md` 第五节第 7 行）：
/// 2、3 号分两次建（对象出生代不同），3 号删完、根环转过一圈之后，把 3 号那条墓碑的 (号, 对象出生代) 改成活着的 2 号的。
/// I-9.10 墓碑那一个等号按号认、出生代相同，成立；队列空，I-9.18 没有对象；I-9.20 不重号；2 < 水位。
#[test]
fn a_tombstone_on_the_lineage_naming_a_live_inode_reddens_only_the_live_object_invariant() {
    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
        "deletion-checker-tombstone-naming-a-live-inode",
    );
    let object_birth_of_inode_two = object_birth_of(&pool.output, 2).expect("2 号活着").0;
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
    let mut image = clean.clone();
    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
        assert_eq!(tombstone_field(tombstone, 0), 3, "那条墓碑的对象 ID 是 3");
        set_tombstone_field(tombstone, 0, 2);
        set_tombstone_field(
            tombstone,
            TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
            object_birth_of_inode_two,
        );
    });
    assert_eq!(
        violated_invariants(&image),
        vec![TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT],
        "只红 I-9.19"
    );
    let detail = first_violation_of(&image, TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT);
    assert!(
        detail.contains("号 2") && detail.contains("nlink 1 的记录"),
        "违例说明点名那条墓碑与活着的记录：{detail}"
    );
}

/// I-9.20（谱系上的墓碑不重号且记录合法） 区间那一句只红自己的那一形（攻方第五节第 8 行）：3 号删完、根环转过一圈之后，
/// 那条墓碑的区间止 2⁶⁴ − 1 → 5（第一版只许整对象）。
#[test]
fn a_tombstone_that_does_not_cover_the_whole_object_reddens_only_the_tombstone_record_invariant() {
    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
        "deletion-checker-tombstone-range-not-whole",
    );
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
    let mut image = clean.clone();
    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
        assert_eq!(
            tombstone_field(tombstone, TOMBSTONE_RECORD_RANGE_END_OFFSET),
            u64::MAX,
            "整对象的区间止是 2⁶⁴ − 1"
        );
        set_tombstone_field(tombstone, TOMBSTONE_RECORD_RANGE_END_OFFSET, 5);
    });
    assert_eq!(
        violated_invariants(&image),
        vec![TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED],
        "只红 I-9.20"
    );
    let detail = first_violation_of(
        &image,
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    );
    assert!(
        detail.contains("[0, 5]") && detail.contains("不是整对象"),
        "违例说明写出那条墓碑的区间：{detail}"
    );
}

/// I-9.20 死亡 txg 那一句（第四节用户定案「补」：对象出生代 ≤ 死亡 txg ≤ 这一版的 txg；攻方第五节「打中五」）只红自己的那一形：
/// 3 号删完、根环转过一圈之后，那条墓碑的死亡 txg 改成 0（早于对象出生代）。0 在每一版的父链上，I-9.18、I-9.19 都看不出这一形。
#[test]
fn a_tombstone_that_died_before_its_object_was_born_reddens_only_the_tombstone_record_invariant() {
    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
        "deletion-checker-tombstone-dies-before-birth",
    );
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
    let mut image = clean.clone();
    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
        set_tombstone_field(tombstone, TOMBSTONE_RECORD_DEATH_TXG_OFFSET, 0);
    });
    assert_eq!(
        violated_invariants(&image),
        vec![TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED],
        "只红 I-9.20"
    );
    let detail = first_violation_of(
        &image,
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    );
    assert!(
        detail.contains("死亡 txg 0 不在"),
        "违例说明写出那条墓碑的死亡 txg：{detail}"
    );
}

/// I-9.20 不重号那一句只红自己的那一形：3 号删完、根环转过一圈之后，墓碑容器里那条墓碑原样再追加一条（两条都在谱系上）。
#[test]
fn two_tombstones_of_one_object_on_the_lineage_redden_only_the_tombstone_record_invariant() {
    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
        "deletion-checker-tombstone-twice-on-the-lineage",
    );
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
    let mut image = clean.clone();
    append_a_copy_of_the_first_tombstone_of_the_newest_root(&mut image);
    assert_eq!(
        violated_invariants(&image),
        vec![TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED],
        "只红 I-9.20"
    );
    let detail = first_violation_of(
        &image,
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    );
    assert!(
        detail.contains("号 3") && detail.contains("不止一条"),
        "违例说明点名重复的那一对：{detail}"
    );
}

/// I-9.10（对象出生代三处一致） 墓碑那一个等号只红自己的那一形（判决第二节 T2 取 a：按号认、每版各比自己的）：2 号 unlink 到底、
/// 回退到发意图之前（2 号又活着），根环转过一圈之后，那条墓碑（死于被回退抛弃的那一段，谱系外）的对象出生代减一。
/// 只取谱系上的墓碑（T2 的 c）看不见它；墓碑在谱系外，I-9.29 不算它、I-9.19 不判它。
#[test]
fn a_tombstone_off_the_lineage_whose_object_birth_differs_from_the_live_record_reddens_only_the_object_birth_invariant(
) {
    let pool =
        pool_with_inode_two_unlinked_then_rolled_back_before_the_intent_and_the_root_ring_turned_over(
            "deletion-checker-tombstone-off-the-lineage-birth",
        );
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "2 号删完、回退到意图之前、根环转过一圈之后");
    assert_eq!(
        verdict_of(&clean, "I-9.10"),
        InvariantVerdict::Holds,
        "回退之后 2 号活着、谱系外那条墓碑的出生代与它相同：I-9.10 成立"
    );
    let mut image = clean.clone();
    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
        assert_eq!(tombstone_field(tombstone, 0), 2, "那条墓碑的对象 ID 是 2");
        let object_birth = tombstone_field(tombstone, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET);
        set_tombstone_field(
            tombstone,
            TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
            object_birth - 1,
        );
    });
    assert_eq!(violated_invariants(&image), vec!["I-9.10"], "只红 I-9.10");
    let detail = first_violation_of(&image, "I-9.10");
    assert!(
        detail.contains("墓碑（号 2"),
        "违例说明点名那条墓碑：{detail}"
    );
}

/// I-9.18（删除队列里的号有谱系上的墓碑） 只红自己的那一形（攻方第五节「打中五」之后的造法：死亡 txg 改成被回退抛弃那一段里的一个 txg，
/// 在 [对象出生代, 这一版的 txg] 里、不在谱系上）：建 2 号，再建 3 号（那一版 X），回退到建 3 号之前，发 2 号的意图；
/// 那条墓碑的死亡 txg 改成 X 的 txg。I-9.20 的「对象出生代 ≤ 死亡 txg ≤ 这一版的 txg」照旧成立，墓碑在谱系外、I-9.19 与 I-9.29 不看它。
#[test]
fn a_queued_inode_whose_tombstone_died_on_a_branch_a_rollback_left_reddens_only_the_queued_tombstone_invariant(
) {
    let mut pool = pool_with_new_inodes("deletion-checker-queued-tombstone-off-the-lineage", 1);
    let before_the_abandoned_branch = pool.output.clone();
    let on_the_abandoned_branch = publish_new_inodes_and_advance_the_pool(&mut pool, 1);
    roll_back_to(&mut pool, &before_the_abandoned_branch);
    publish_the_intent_of(&mut pool, 2);
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "回退之后发了 2 号的意图");
    assert_eq!(
        verdict_of(&clean, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE),
        InvariantVerdict::Holds,
        "队列 {{2}} 的那一版 I-9.18 真被评估过且成立"
    );
    let txg_on_the_abandoned_branch = on_the_abandoned_branch.root.checkpoint_txg.0;
    let mut image = clean.clone();
    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
        assert_eq!(tombstone_field(tombstone, 0), 2, "那条墓碑的对象 ID 是 2");
        set_tombstone_field(
            tombstone,
            TOMBSTONE_RECORD_DEATH_TXG_OFFSET,
            txg_on_the_abandoned_branch,
        );
    });
    assert_eq!(
        violated_invariants(&image),
        vec![QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE],
        "只红 I-9.18"
    );
    let detail = first_violation_of(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE);
    assert!(
        detail.contains("deleted_inodes 里的号 2") && detail.contains("没有在谱系上的整对象墓碑"),
        "违例说明点名队列里那个号：{detail}"
    );
}

/// 「在谱系上」每一版从自己的 txg 与自己的谱系树起算（判决第二节 T3 取 a；攻方第一节「打中一」「打中二」那两段合法历史）：
/// 2 号 unlink 到底，回退到发意图之前，再回退到删完那一版。第一次回退之后，意图那一版（队列 {2}）还在候选集里，它自己的谱系树里
/// 还没有这次回退那条记录，墓碑在它的链上，I-9.18 成立（一律从所选根起算会在这里误红）；第二次回退之后，第一次回退那一版（2 号活着）
/// 自己的父链跳过意图那次的 txg，I-9.19 不判它（一律从所选根起算、或不按谱系滤会在这里误红）。两步都一条违例都没有。
#[test]
fn unlinking_then_rolling_back_before_the_intent_and_back_to_the_emptied_version_holds_every_lineage_invariant(
) {
    let mut pool = pool_with_new_inodes("deletion-checker-unlink-and-two-rollbacks", 1);
    let before_the_intent = pool.output.clone();
    unlink_to_the_end(&mut pool, 2);
    let emptied = pool.output.clone();
    roll_back_to(&mut pool, &before_the_intent);
    let after_the_rollback_before_the_intent = pool.memory_pool();
    assert_eq!(
        violated_invariants(&after_the_rollback_before_the_intent),
        Vec::<&str>::new(),
        "回退到意图之前：没有违例"
    );
    assert_eq!(
        verdict_of(
            &after_the_rollback_before_the_intent,
            QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE
        ),
        InvariantVerdict::Holds,
        "意图那一版还在候选集里，I-9.18 真被评估过且成立"
    );
    roll_back_to(&mut pool, &emptied);
    let after_the_rollback_to_the_emptied_version = pool.memory_pool();
    assert_eq!(
        violated_invariants(&after_the_rollback_to_the_emptied_version),
        Vec::<&str>::new(),
        "再回退到删完那一版：没有违例"
    );
    assert_eq!(
        verdict_of(
            &after_the_rollback_to_the_emptied_version,
            TOMBSTONE_ON_THE_LINEAGE_NAMES_NO_LIVE_OBJECT
        ),
        InvariantVerdict::Holds,
        "意图那一版里墓碑指着 nlink 0 的记录，I-9.19 真被评估过且成立"
    );
}

/// 「删 → 回退到删之前 → 再删」的合法历史（`crates/singlefs-checker/invariants/09-inode树结构.md` I-9.20 那一行：不按谱系过滤的写法在这一段上判红）：墓碑树里有两条
/// (2, 2 号的出生代)，第一条死于被回退抛弃的那一段、不在谱系上，不重号那一句只判谱系上的，成立；一条违例都没有。
#[test]
fn unlinking_the_same_inode_again_after_a_rollback_before_its_intent_holds_the_tombstone_record_invariant(
) {
    let mut pool = pool_with_new_inodes("deletion-checker-unlink-rollback-unlink", 1);
    let before_the_intent = pool.output.clone();
    unlink_to_the_end(&mut pool, 2);
    roll_back_to(&mut pool, &before_the_intent);
    unlink_to_the_end(&mut pool, 2);
    let image = pool.memory_pool();
    assert_eq!(
        violated_invariants(&image),
        Vec::<&str>::new(),
        "删、回退、再删：没有违例"
    );
    for invariant in [
        "I-9.10",
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
        ONE_OBJECT_BIRTH_PER_INODE_NUMBER,
    ] {
        assert_eq!(
            verdict_of(&image, invariant),
            InvariantVerdict::Holds,
            "{invariant} 真被评估过且成立"
        );
    }
}

/// 判红的那几条，按名字排好（几条一起红时只比集合，不比 checker 报的次序）。
fn violated_invariants_in_name_order(image: &MemoryPool) -> Vec<&'static str> {
    let mut violated = violated_invariants(image);
    violated.sort_unstable();
    violated
}

/// M2（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K1 覆盖缺口那一行：I-9.29「与那一版谱系上墓碑的出生代都算」
/// 墓碑那一侧没有用例钉）：2、3 号分两次建（对象出生代不同），3 号删完、根环转过一圈之后，那条墓碑的号 3 → 2、出生代不动（还是 3 号的）。
/// 2 号的记录与谱系上的墓碑给出两个出生代：I-9.29 红；I-9.10 墓碑那一个等号按号认，同样红。墓碑的出生代与 2 号记录的不同，I-9.19 不判它。
/// 建 2 号那次留的台阶说 2 号在那之前就发过，谱系上这条墓碑（2 号、3 号的对象出生代）晚于它出生：I-9.30 ② 也红（第 49 批）。
#[test]
fn a_tombstone_on_the_lineage_carrying_another_object_birth_for_a_live_number_reddens_the_one_birth_per_number_invariant(
) {
    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
        "deletion-checker-tombstone-on-the-lineage-second-birth",
    );
    let object_birth_of_inode_two = object_birth_of(&pool.output, 2).expect("2 号活着").0;
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
    let mut image = clean.clone();
    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
        assert_eq!(tombstone_field(tombstone, 0), 3, "那条墓碑的对象 ID 是 3");
        assert_ne!(
            tombstone_field(tombstone, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET),
            object_birth_of_inode_two,
            "2、3 号分两次建，对象出生代不同"
        );
        set_tombstone_field(tombstone, 0, 2);
    });
    assert_eq!(
        violated_invariants_in_name_order(&image),
        vec!["I-9.10", ONE_OBJECT_BIRTH_PER_INODE_NUMBER, "I-9.30"],
        "2 号的记录与谱系上的墓碑两个出生代：红 I-9.29 与 I-9.10 墓碑那一个等号；墓碑晚于建 2 号那级台阶出生，红 I-9.30"
    );
    let detail = first_violation_of(&image, ONE_OBJECT_BIRTH_PER_INODE_NUMBER);
    assert!(
        detail.contains("inode 2 在回退候选集里对应不止一个对象出生代"),
        "违例说明点名两个出生代的那个号：{detail}"
    );
}

/// M3（覆盖缺口那一行：I-9.20「死亡 txg ≤ 这一版的 txg」上界没有用例钉）：3 号删完、根环转过一圈之后，那条墓碑的死亡 txg 改成最新根的 txg + 1。
/// 它晚于每一版的 txg、不在任何一版的谱系上：I-9.18（队列空）、I-9.19、I-9.29 都不看它，只红 I-9.20。
#[test]
fn a_tombstone_that_died_after_every_version_reddens_only_the_tombstone_record_invariant() {
    let pool = pool_with_inode_three_unlinked_and_the_root_ring_turned_over(
        "deletion-checker-tombstone-dies-after-every-version",
    );
    let after_the_newest_txg = pool.output.root.checkpoint_txg.0 + 1;
    let clean = pool.memory_pool();
    assert_the_base_image_is_clean(&clean, "3 号删完、根环转过一圈之后");
    let mut image = clean.clone();
    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
        set_tombstone_field(
            tombstone,
            TOMBSTONE_RECORD_DEATH_TXG_OFFSET,
            after_the_newest_txg,
        );
    });
    assert_eq!(
        violated_invariants(&image),
        vec![TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED],
        "只红 I-9.20"
    );
    let detail = first_violation_of(
        &image,
        TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
    );
    assert!(
        detail.contains(&format!("死亡 txg {after_the_newest_txg} 不在")),
        "违例说明写出那条墓碑的死亡 txg：{detail}"
    );
}

/// M4（覆盖缺口那一行：I-9.4 第四句「记录号沿记录序严格递增」只钉了同号两条、倒序没有钉）：一次建两个空 inode（叶 [1, 2, 3]），
/// 后两条的号对调成 [1, 3, 2]（两条都是同一次建的空 inode，除了号逐字节相同）。只红 I-9.4。
#[test]
fn a_leaf_holding_two_inode_numbers_in_descending_order_reddens_only_the_container_number_invariant(
) {
    let clean = pool_with_new_inodes("deletion-checker-leaf-in-descending-order", 2).memory_pool();
    assert_the_base_image_is_clean(&clean, "一次建两个空 inode 之后");
    let mut image = clean.clone();
    change_the_inode_record(&mut image, 3, |record| {
        record[..8].copy_from_slice(&2u64.to_le_bytes())
    });
    change_the_inode_record(&mut image, 2, |record| {
        record[..8].copy_from_slice(&3u64.to_le_bytes())
    });
    assert_eq!(violated_invariants(&image), vec!["I-9.4"], "只红 I-9.4");
    let detail = first_violation_of(&image, "I-9.4");
    assert!(
        detail.contains("inode 3 之后是 inode 2"),
        "违例说明写出倒序的那一对：{detail}"
    );
}

/// M6（覆盖缺口那一行：I-9.18 配墓碑时「N 的 inode 记录偏移 8 的对象出生代」没有用例钉）：空 inode 2 发完意图之后，那条墓碑的对象出生代减一。
/// 队列里的 2 号没有 (2, 它记录的出生代) 的墓碑：I-9.18 红；同号两个出生代、墓碑那一个等号不符，I-9.29、I-9.10 同样红。
#[test]
fn a_queued_inode_whose_tombstone_names_another_object_birth_reddens_the_queued_tombstone_invariant(
) {
    let clean = image_after_the_intent_of_an_empty_inode("deletion-checker-queued-tombstone-birth");
    assert_the_base_image_is_clean(&clean, "空 inode 2 发完意图之后");
    let mut image = clean.clone();
    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
        assert_eq!(tombstone_field(tombstone, 0), 2, "那条墓碑的对象 ID 是 2");
        let object_birth = tombstone_field(tombstone, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET);
        set_tombstone_field(
            tombstone,
            TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
            object_birth - 1,
        );
    });
    assert_eq!(
        violated_invariants_in_name_order(&image),
        vec![
            "I-9.10",
            QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
            ONE_OBJECT_BIRTH_PER_INODE_NUMBER
        ],
        "墓碑的出生代不是 2 号记录的：红 I-9.18，连带 I-9.29 与 I-9.10 墓碑那一个等号"
    );
    let detail = first_violation_of(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE);
    assert!(
        detail.contains("deleted_inodes 里的号 2"),
        "违例说明点名队列里那个号：{detail}"
    );
}

/// M7（覆盖缺口那一行：I-9.18 配墓碑时「[0, 2⁶⁴ − 1]」整对象区间没有用例钉）：空 inode 2 发完意图之后，那条墓碑的区间止改成 5。
/// 队列里的 2 号没有整对象墓碑：I-9.18 红；第一版只许整对象，I-9.20 同样红。
#[test]
fn a_queued_inode_whose_tombstone_does_not_cover_the_whole_object_reddens_the_queued_tombstone_invariant(
) {
    let clean = image_after_the_intent_of_an_empty_inode("deletion-checker-queued-tombstone-range");
    assert_the_base_image_is_clean(&clean, "空 inode 2 发完意图之后");
    let mut image = clean.clone();
    change_the_first_tombstone_of_the_newest_root(&mut image, |tombstone| {
        set_tombstone_field(tombstone, TOMBSTONE_RECORD_RANGE_END_OFFSET, 5);
    });
    assert_eq!(
        violated_invariants_in_name_order(&image),
        vec![
            QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE,
            TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED
        ],
        "墓碑不是整对象：红 I-9.18，连带 I-9.20"
    );
    let detail = first_violation_of(&image, QUEUED_INODE_HAS_A_TOMBSTONE_ON_THE_LINEAGE);
    assert!(
        detail.contains("没有在谱系上的整对象墓碑"),
        "违例说明写出没有整对象墓碑：{detail}"
    );
}

/// 码 2 节点头里条目数的偏移（82 + 2 × key 宽）、头里最小 key 的偏移（52）与声明长度（偏移 8，2 字节），D18（块里携带什么信息） 已定项 18。
fn index_node_entry_count_offset(key_width: usize) -> usize {
    82 + 2 * key_width
}
const UNIT_DECLARED_LENGTH_OFFSET: usize = 8;
const DELETED_INODES_ENTRY_BYTES: usize = 8;

/// 队列同号两条（判决 `research/prompts/m3-batch22-code-r1-main-verification.md` 第二节 K2 N7 那一行「同号两条归哪一条管，进下一批查清」）：
/// 空 inode 2 发完意图之后，`deleted_inodes` 根（码 2、层级 0、key 宽 8、条目宽 8）{2} 末尾再追加一条 2（条目数 1 → 2、声明长度 8 → 16，
/// 头里的 key 区间 [2, 2] 不变）。池级 checker 读队列时按号收成集合，I-9.24（删除队列至多一条 key） 看到的是一个号、照旧成立；
/// 管它的是 I-1.1（码 2 节点的条目按 key 严格递增）。只红 I-1.1。
#[test]
fn a_deletion_queue_holding_one_number_twice_reddens_only_the_key_order_invariant() {
    let clean = image_after_the_intent_of_an_empty_inode("deletion-checker-queue-one-number-twice");
    assert_the_base_image_is_clean(&clean, "空 inode 2 发完意图之后");
    let mut image = clean.clone();
    let queue_slot =
        root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_DELETED_INODES);
    common_deletion_in_progress::change_the_unit_and_propagate_its_checksum(
        &mut image,
        queue_slot,
        node_bytes(),
        |root| {
            let key_width = usize::from(root[51]);
            assert_eq!(
                key_width, DELETED_INODES_ENTRY_BYTES,
                "deleted_inodes 的 key 宽 8"
            );
            let count_offset = index_node_entry_count_offset(key_width);
            let count = usize::from(u16::from_le_bytes([
                root[count_offset],
                root[count_offset + 1],
            ]));
            assert_eq!(count, 1, "队列 {{2}} 一条");
            let first = 115 + 2 * key_width;
            let queued = root[first..first + DELETED_INODES_ENTRY_BYTES].to_vec();
            assert_eq!(queued, 2u64.to_le_bytes(), "队列里是 2 号");
            let second = first + DELETED_INODES_ENTRY_BYTES;
            root[second..second + DELETED_INODES_ENTRY_BYTES].copy_from_slice(&queued);
            root[count_offset..count_offset + 2].copy_from_slice(&2u16.to_le_bytes());
            root[UNIT_DECLARED_LENGTH_OFFSET..UNIT_DECLARED_LENGTH_OFFSET + 2].copy_from_slice(
                &u16::try_from(2 * DELETED_INODES_ENTRY_BYTES)
                    .expect("16")
                    .to_le_bytes(),
            );
        },
    );
    assert_eq!(
        violated_invariants(&image),
        vec!["I-1.1"],
        "队列同号两条：只红 I-1.1（I-9.24 按号收成一个）"
    );
    assert_eq!(
        verdict_of(&image, DELETION_QUEUE_HOLDS_AT_MOST_ONE_KEY),
        InvariantVerdict::Holds,
        "I-9.24 看到的是一个号：成立"
    );
}

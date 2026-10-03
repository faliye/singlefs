//! 新根段里的实例表指针与分配记录树根指针（D23（journal 的角色与格式） 已定项 15，用户 2026-10-02「回到甲」）在施加一条记录时取记录里的：
//! 实现恢复拼新根那一截（`recovery::replay_journal`）与池级 checker 重放那一版的走读（I-4.1（重放后树可遍历），
//! `walk::walk_version_applied_only_by_records`）都从新根段读这两项，不照抄所选根。
//!
//! 今天被施加的发布没有一种改实例表指针（改它的写行是新实例的第一次发布、从不施加；专用清扫发布还没实现），所以这里的盘面是改出来的：
//! 新池新建文件（txg 3）的根槽改坏、记录留着，再把那条记录新根段里的一项改成指着校验和对不上的落点（两份位置条目的整单元校验和各翻一位），
//! 重封记录头——恢复照样施加它（点名单元两份都验得过），施加出来的根带着改过的那一项；checker 重放同一条记录，沿改过的那一项走不下去。
mod common;

use common::{
    build_pool, rewrite_journal_record_and_rechain_the_records_after_it,
    tear_the_root_slot_written_for_txg,
};
use singlefs_checker::image::{InvariantVerdict, TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE};
use singlefs_checker::walk::check_pool_image;
use singlefs_core::address::CheckpointTxg;
use singlefs_core::journal::JournalRecord;
use singlefs_core::pointer::NodePointer;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, replay_journal, scan_journal,
};
use singlefs_harness::memory_pool::MemoryPool;

/// 新池新建文件那条记录的 jsn 计数器（暖机 1、2，新池新建文件 3）。
const COUNTER_OF_THE_NEW_POOL_FILE_CREATION_RECORD: u64 = 3;

/// 一条指着与 `pointer` 同一落点、两份整单元校验和都对不上的指针：读得到单元，校验和那一关两份都过不去。
fn pointer_whose_checksums_do_not_match(pointer: NodePointer) -> NodePointer {
    let mut mismatching = pointer;
    for location in &mut mismatching.locations {
        location.unit_checksum ^= 1;
    }
    mismatching
}

/// 新池新建文件（txg 3）的根槽改坏、记录留着；再按 `change` 改那条记录（两盘各一份，重封）。交回改好的镜像。
fn image_with_the_new_pool_file_creation_record_but_not_its_root(
    tag: &str,
    change: impl Fn(&mut JournalRecord),
) -> MemoryPool {
    let pool = build_pool(tag);
    assert_eq!(
        (pool.output.root.checkpoint_txg, pool.output.record.counter),
        (
            CheckpointTxg(3),
            COUNTER_OF_THE_NEW_POOL_FILE_CREATION_RECORD
        ),
        "新池新建文件：txg 3、jsn 3"
    );
    let mut image = pool.memory_pool();
    tear_the_root_slot_written_for_txg(&mut image, 3);
    rewrite_journal_record_and_rechain_the_records_after_it(
        &mut image,
        COUNTER_OF_THE_NEW_POOL_FILE_CREATION_RECORD,
        change,
    );
    image
}

fn verdict_of_the_replayed_version_invariant(image: &MemoryPool) -> InvariantVerdict {
    check_pool_image(image)
        .into_iter()
        .find(|(invariant, _)| *invariant == TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE)
        .map(|(_, verdict)| verdict)
        .expect("池级 checker 每条已实现的不变量都报")
}

/// 实现恢复：所选根是暖机第二条（txg 2），施加 txg 3 那条记录；施加出来的根的实例表指针是那条记录新根段里改过的那一条，
/// 不是所选根的（照抄所选根时两者相同、这一判红）。
#[test]
fn the_root_rebuilt_from_a_record_takes_the_instance_table_pointer_of_its_new_root_segment() {
    let image = image_with_the_new_pool_file_creation_record_but_not_its_root(
        "rebuilt-root-instance-table-from-the-record",
        |record| {
            record.new_instance_table = pointer_whose_checksums_do_not_match(record.new_tree_table);
        },
    );
    let system_configuration = choose_system_configuration(&image).expect("系统配置择得出");
    let chosen_root = choose_root(&image, &system_configuration).expect("择得出根");
    assert_eq!(
        chosen_root.checkpoint_txg,
        CheckpointTxg(2),
        "txg 3 的根槽坏了：所选根是暖机第二条"
    );
    let records = scan_journal(&image, &system_configuration);
    let record = records
        .values()
        .find(|record| record.counter == COUNTER_OF_THE_NEW_POOL_FILE_CREATION_RECORD)
        .expect("txg 3 那条记录改过之后照样自证得过");
    let (journal, rebuilt_root) = replay_journal(
        &image,
        &chosen_root,
        system_configuration.immutable.sizes.journal_ring_bytes,
        &records,
        true,
    )
    .expect("恢复施加 txg 3 那条记录");
    assert_eq!(journal.prefix_applied, 1, "施加了那一条");
    assert_ne!(
        record.new_instance_table, chosen_root.instance_table,
        "改过的那一项与所选根的实例表指针不同，两种取法分得开"
    );
    assert_eq!(
        rebuilt_root.instance_table, record.new_instance_table,
        "施加出来的根的实例表指针取记录新根段里的"
    );
}

/// 对照：同一段历史、记录不改，checker 重放施加出来那一版（新根段七项都是那次发布写的根上的）走得完，I-4.1 成立。
#[test]
fn the_replayed_version_invariant_holds_over_the_untouched_record_without_its_root() {
    let untouched = image_with_the_new_pool_file_creation_record_but_not_its_root(
        "replayed-version-untouched",
        |_record| {},
    );
    assert_eq!(
        verdict_of_the_replayed_version_invariant(&untouched),
        InvariantVerdict::Holds,
        "没改的那条记录施加出来那一版走得完"
    );
}

/// 池级 checker 重放改过的盘面（新根段里的实例表指针指着校验和对不上的落点）：施加出来那一版的实例表链从新根段读、走不下去 ⇒ I-4.1 判红。
/// 照抄所选根的实例表（或不走实例表）时走得完、判成立。
#[test]
fn the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment() {
    let image = image_with_the_new_pool_file_creation_record_but_not_its_root(
        "replayed-version-instance-table-from-the-segment",
        |record| {
            record.new_instance_table = pointer_whose_checksums_do_not_match(record.new_tree_table);
        },
    );
    assert!(
        matches!(
            verdict_of_the_replayed_version_invariant(&image),
            InvariantVerdict::Violated(_)
        ),
        "新根段里的实例表指针走不下去：I-4.1 判红"
    );
}

/// 同上，改的是新根段里的分配记录树根指针（这一版带文件，记录写的是空指针；改成指着校验和对不上的落点）：checker 从新根段读它、
/// 走不下去 ⇒ I-4.1 判红。照抄所选根的分配记录树根（暖机那一版，全零、不走）时判成立。
#[test]
fn the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment() {
    let image = image_with_the_new_pool_file_creation_record_but_not_its_root(
        "replayed-version-allocation-record-tree-from-the-segment",
        |record| {
            record.new_allocation_record_tree_root =
                pointer_whose_checksums_do_not_match(record.new_tree_table);
        },
    );
    assert!(
        matches!(
            verdict_of_the_replayed_version_invariant(&image),
            InvariantVerdict::Violated(_)
        ),
        "新根段里的分配记录树根指针走不下去：I-4.1 判红"
    );
}

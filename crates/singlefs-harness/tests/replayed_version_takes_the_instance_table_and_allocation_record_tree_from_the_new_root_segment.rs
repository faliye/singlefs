//! 新根段里的实例表指针与分配记录树根指针（D23（journal 的角色与格式） 已定项 15，用户 2026-10-02「回到甲」）在施加一条记录时取记录里的：
//! 实现恢复拼新根那一截（`recovery::replay_journal`）与池级 checker 重放那一版的走读（I-4.1（重放后树可遍历），
//! `walk::walk_version_applied_only_by_records`）都从新根段读这两项，不照抄所选根。
//!
//! 今天被施加的发布没有一种改实例表指针（改它的写行是新实例的第一次发布、从不施加；专用清扫发布还没实现），所以这里的盘面是改出来的：
//! 新池新建文件（txg 3）的根槽改坏、记录留着，再把那条记录新根段里的一项改成指着校验和对不上的落点（两份位置条目的整单元校验和各翻一位），
//! 重封记录头——恢复照样施加它（点名单元两份都验得过），施加出来的根带着改过的那一项；checker 重放同一条记录，沿改过的那一项走不下去。
mod common;
mod common_images_for_the_checker_tier;
#[path = "common_images_for_the_checker_tier/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs"]
mod scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment;

use common_images_for_the_checker_tier::PoolCheckerImages;
use scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::{COUNTER_OF_THE_NEW_POOL_FILE_CREATION_RECORD, image_with_the_new_pool_file_creation_record_but_not_its_root, pointer_whose_checksums_do_not_match};
use singlefs_core::address::CheckpointTxg;
use singlefs_core::recovery::{
    choose_root, choose_system_configuration, replay_journal, NamedUnitVerification, scan_journal,
};

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
        NamedUnitVerification::Verify,
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

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_holds_over_the_untouched_record_without_its_root`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_replayed_version_invariant_holds_over_the_untouched_record_without_its_root() {
    scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_holds_over_the_untouched_record_without_its_root(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment() {
    scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment(&mut PoolCheckerImages::left_to_the_checker_tier());
}

/// 场景与它交给池级 checker 的镜像见 `scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment`；池级 checker 那一遍在 checker 档同名用例（`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs`）。
#[test]
fn the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment() {
    scenarios_of_replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment::the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment(&mut PoolCheckerImages::left_to_the_checker_tier());
}

//! harness 测试文件 `crates/singlefs-harness/tests/replayed_version_takes_the_instance_table_and_allocation_record_tree_from_the_new_root_segment.rs` 的场景函数：搭场景、核实现的行为，池级 checker 要判的镜像经 [`PoolCheckerImages`] 交出去
//! （做法见 `common_images_for_the_checker_tier/mod.rs`）。harness 档那份测试文件与 checker 档的
//! `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 各经 `#[path]` 引这一份。
#![allow(dead_code, reason = "两档各只调其中一部分")]

use crate::common::{
    build_pool, rewrite_journal_record_and_rechain_the_records_after_it,
    tear_the_root_slot_written_for_txg,
};
use crate::common_images_for_the_checker_tier::{PoolCheckerExpectation, PoolCheckerImages};
use singlefs_checker::image::TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE;
use singlefs_core::address::CheckpointTxg;
use singlefs_core::journal::JournalRecord;
use singlefs_core::pointer::NodePointer;
use singlefs_harness::memory_pool::MemoryPool;

/// 新池新建文件那条记录的 jsn 计数器（暖机 1、2，新池新建文件 3）。
pub const COUNTER_OF_THE_NEW_POOL_FILE_CREATION_RECORD: u64 = 3;

/// 一条指着与 `pointer` 同一落点、两份整单元校验和都对不上的指针：读得到单元，校验和那一关两份都过不去。
pub fn pointer_whose_checksums_do_not_match(pointer: NodePointer) -> NodePointer {
    let mut mismatching = pointer;
    for location in &mut mismatching.locations {
        location.unit_checksum ^= 1;
    }
    mismatching
}

/// 新池新建文件（txg 3）的根槽改坏、记录留着；再按 `change` 改那条记录（两盘各一份，重封）。交回改好的镜像。
pub fn image_with_the_new_pool_file_creation_record_but_not_its_root(
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

/// 对照：同一段历史、记录不改，checker 重放施加出来那一版（新根段七项都是那次发布写的根上的）走得完，I-4.1 成立。
pub fn the_replayed_version_invariant_holds_over_the_untouched_record_without_its_root(
    images: &mut PoolCheckerImages,
) {
    let untouched = image_with_the_new_pool_file_creation_record_but_not_its_root(
        "replayed-version-untouched",
        |_record| {},
    );
    // 没改的那条记录施加出来那一版走得完
    images.hand_out(
        "记录不改、根槽改坏",
        || untouched.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_holds(TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
    );
}

/// 池级 checker 重放改过的盘面（新根段里的实例表指针指着校验和对不上的落点）：施加出来那一版的实例表链从新根段读、走不下去 ⇒ I-4.1 判红。
/// 照抄所选根的实例表（或不走实例表）时走得完、判成立。
pub fn the_replayed_version_invariant_walks_the_instance_table_of_the_new_root_segment(
    images: &mut PoolCheckerImages,
) {
    let image = image_with_the_new_pool_file_creation_record_but_not_its_root(
        "replayed-version-instance-table-from-the-segment",
        |record| {
            record.new_instance_table = pointer_whose_checksums_do_not_match(record.new_tree_table);
        },
    );
    // 新根段里的实例表指针走不下去：I-4.1 判红
    images.hand_out(
        "新根段里的实例表指针改成校验和对不上的落点",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated(TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
    );
}

/// 同上，改的是新根段里的分配记录树根指针（这一版带文件，记录写的是空指针；改成指着校验和对不上的落点）：checker 从新根段读它、
/// 走不下去 ⇒ I-4.1 判红。照抄所选根的分配记录树根（暖机那一版，全零、不走）时判成立。
pub fn the_replayed_version_invariant_walks_the_allocation_record_tree_root_of_the_new_root_segment(
    images: &mut PoolCheckerImages,
) {
    let image = image_with_the_new_pool_file_creation_record_but_not_its_root(
        "replayed-version-allocation-record-tree-from-the-segment",
        |record| {
            record.new_allocation_record_tree_root =
                pointer_whose_checksums_do_not_match(record.new_tree_table);
        },
    );
    // 新根段里的分配记录树根指针走不下去：I-4.1 判红
    images.hand_out(
        "新根段里的分配记录树根指针改成校验和对不上的落点",
        || image.clone(),
        PoolCheckerExpectation::only_the_named_invariants()
            .and_violated(TREES_OF_THE_REPLAYED_VERSION_ARE_TRAVERSABLE),
    );
}

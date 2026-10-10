#![allow(dead_code, unused_imports)]
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common_deletion_in_progress::{
    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at,
    image_after_the_intent_of_a_three_unit_file, pool_with_a_three_unit_file, publish_the_intent,
    publish_the_next_step_in_batches_of_one, remove_the_deleted_inodes_entry_from_the_tree_table_at,
    tree_table_slot_of_the_newest_root, verdict_of, violated_invariants,
};
use singlefs_core::transaction::TransactionUnit;

#[test]
fn probe() {
    let clean = image_after_the_intent_of_a_three_unit_file("probe-clean");
    println!("clean violated={:?}", violated_invariants(&clean));
    for inv in ["I-9.5", "I-4.7", "I-4.6", "I-4.4", "I-9.16"] {
        println!("clean {inv} = {:?}", verdict_of(&clean, inv));
    }
    let mut image = clean.clone();
    let slot = tree_table_slot_of_the_newest_root(&image);
    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(&mut image, slot);
    println!("L1a violated={:?}", violated_invariants(&image));
    for inv in ["I-9.5", "I-4.7", "I-4.6"] {
        println!("L1a {inv} = {:?}", verdict_of(&image, inv));
    }
    let mut pool = pool_with_a_three_unit_file("probe-l1c");
    publish_the_intent(&mut pool);
    let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
    publish_the_next_step_in_batches_of_one(&mut pool);
    let mut img = pool.memory_pool();
    println!("L1c-base violated={:?}", violated_invariants(&img));
    for inv in ["I-9.5", "I-4.7", "I-4.6"] {
        println!("L1c-base {inv} = {:?}", verdict_of(&img, inv));
    }
    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(
        &mut img,
        intermediate.unit(TransactionUnit::TreeTable).slot,
    );
    println!("L1c violated={:?}", violated_invariants(&img));
    for inv in ["I-9.5", "I-4.7", "I-4.6"] {
        println!("L1c {inv} = {:?}", verdict_of(&img, inv));
    }
}

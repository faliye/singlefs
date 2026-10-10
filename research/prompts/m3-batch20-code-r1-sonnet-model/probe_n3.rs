#![allow(dead_code, unused_imports)]
mod common;
mod common_deletion_in_progress;
mod common_pointer_head_corruption;

use common_deletion_in_progress::*;
use common_pointer_head_corruption::{data_unit_bytes, node_bytes};
use singlefs_core::records::{TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT};
use singlefs_core::transaction::TransactionUnit;

#[test]
fn probe() {
    // L2
    let mut image = image_after_the_intent_of_a_three_unit_file("p-l2");
    raise_the_size_of_the_first_file_by_one_data_unit_payload(&mut image);
    let s = tree_table_slot_of_the_newest_root(&image);
    give_the_deleted_inodes_entry_an_unregistered_kind_in_the_tree_table_at(&mut image, s);
    println!("P L2 violated={:?}", violated_invariants(&image));
    // extent root zero
    let mut image = image_after_the_intent_of_a_three_unit_file("p-ext");
    let slot = root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_EXTENT);
    zero_both_copies_of_the_unit_at(&mut image, slot, node_bytes());
    println!("P extent-root-zero violated={:?}", violated_invariants(&image));
    // deleted_inodes root zero
    let mut image = image_after_the_intent_of_a_three_unit_file("p-del");
    let slot = root_slot_of_the_tree_of_kind_of_the_newest_root(&image, TREE_KIND_DELETED_INODES);
    zero_both_copies_of_the_unit_at(&mut image, slot, node_bytes());
    println!("P deleted-root-zero violated={:?}", violated_invariants(&image));
    // tree table zero / narrow (intermediate)
    for narrow in [false, true] {
        let mut pool = pool_with_a_three_unit_file("p-tt");
        publish_the_intent(&mut pool);
        let intermediate = publish_the_next_step_in_batches_of_one(&mut pool);
        publish_the_next_step_in_batches_of_one(&mut pool);
        let mut image = pool.memory_pool();
        let slot = intermediate.unit(TransactionUnit::TreeTable).slot;
        if narrow {
            narrow_the_entries_of_the_tree_table_at_to_the_key_width(&mut image, slot);
        } else {
            zero_both_copies_of_the_unit_at(&mut image, slot, node_bytes());
        }
        println!("P tree-table narrow={narrow} violated={:?}", violated_invariants(&image));
    }
    // tombstone zero
    let mut image = image_after_the_intent_of_a_three_unit_file("p-tomb");
    let slot = first_tombstone_container_slot_of_the_newest_root(&image);
    zero_both_copies_of_the_unit_at(&mut image, slot, data_unit_bytes());
    println!("P tombstone-zero violated={:?}", violated_invariants(&image));
    // unpublished tombstone
    let mut image = image_after_the_intent_of_a_three_unit_file("p-tomb2");
    move_the_tombstone_container_to_an_unpublished_instance_and_raise_its_object_identifier(&mut image);
    println!("P tombstone-unpublished violated={:?}", violated_invariants(&image));
}

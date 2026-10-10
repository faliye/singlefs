// ───────── 正推腿探针（草稿副本，不入库）：K1 换实例旋钮各异的合法历史 ─────────
#[test]
fn sonnet_probe_k1_mixed_knob_history_tombstone() {
    let tree = ContainerTreeOfTheCapacityInvariants::Tombstone;
    let mut pool = common_deletion_in_progress::pool_with_new_inodes("sonnet-probe-k1-mixed", 5);
    for inode in 2..5 {
        unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, tree.write_capacities(2));
    }
    assert_eq!(tree.record_counts_of_each_container(&pool.output), vec![2, 1]);
    for knob in [1u64, 2, 3] {
        let v = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(&pool.memory_pool(), tree.checker_knobs(knob));
        eprintln!("PROBE_K1 stage=[2,1] judged_with_knob={knob} violated={:?}", violated_invariants(&v));
    }
    unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, 5, tree.write_capacities(3));
    assert_eq!(tree.record_counts_of_each_container(&pool.output), vec![2, 2]);
    for knob in [1u64, 2, 3] {
        let v = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(&pool.memory_pool(), tree.checker_knobs(knob));
        eprintln!("PROBE_K1 stage=[2,2](knob 2 then knob 3 write) judged_with_knob={knob} violated={:?}", violated_invariants(&v));
    }
}

#[test]
fn sonnet_probe_k1_knob_two_one_two_history_tombstone() {
    let tree = ContainerTreeOfTheCapacityInvariants::Tombstone;
    let mut pool = common_deletion_in_progress::pool_with_new_inodes("sonnet-probe-k1-212", 6);
    for inode in 2..5 {
        unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, inode, tree.write_capacities(2));
    }
    unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, 5, tree.write_capacities(1));
    eprintln!("PROBE_K1B after knob 2,2,2 then 1: {:?}", tree.record_counts_of_each_container(&pool.output));
    unlink_an_empty_inode_to_the_end_under_the_capacities(&mut pool, 6, tree.write_capacities(2));
    eprintln!("PROBE_K1B after knob 2 again: {:?}", tree.record_counts_of_each_container(&pool.output));
    for knob in [1u64, 2, 3] {
        let v = singlefs_checker::walk::check_pool_image_given_the_container_capacity_knobs(&pool.memory_pool(), tree.checker_knobs(knob));
        eprintln!("PROBE_K1B judged_with_knob={knob} violated={:?}", violated_invariants(&v));
    }
}

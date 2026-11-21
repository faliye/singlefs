use super::*;

#[test]
fn main_random_sequence_feeds_every_arm_the_same_operation_prefix() {
    let sequence = registered_sequences()[0];
    assert_eq!(sequence.name, "wr_p055_seed1");
    let result = run_sequence(sequence, &SEGMENT_ONE_ARMS, &SEGMENT_ONE_WINDOWS, OPERATIONS_PER_ARM_AT_MOST);
    assert_window_prefixes_agree(&result);
    let main_window = &result.windows[0];
    assert!(main_window.unlinks > 0, "主窗口里有删除（读得到的数，不是空窗口）");
    assert!(matches!(result.trajectories[0].stop, ArmStop::Refused { .. }), "W-R p = 0.55 上 M1 撞得到上限");
}

#[test]
fn sequential_workload_creates_a_round_then_deletes_the_oldest() {
    let mut generator = WorkloadGenerator::new(Workload::SequentialDelete { inserted_per_round: 3, deleted_per_round: 2, order: DeletionOrder::SmallestFirst }, 0);
    let operations: Vec<Operation> = (0..7).map(|_| generator.next_operation()).collect();
    assert_eq!(
        operations,
        vec![Operation::Create(1), Operation::Create(2), Operation::Create(3), Operation::Unlink(1), Operation::Unlink(2), Operation::Create(4), Operation::Create(5)]
    );
    let mut newest_first = WorkloadGenerator::new(Workload::SequentialDelete { inserted_per_round: 3, deleted_per_round: 2, order: DeletionOrder::LargestFirst }, 0);
    let newest: Vec<Operation> = (0..5).map(|_| newest_first.next_operation()).collect();
    assert_eq!(newest[3..], [Operation::Unlink(3), Operation::Unlink(2)]);
}

#[test]
fn alternate_workload_deletes_the_registered_share_of_each_batch() {
    let mut half = WorkloadGenerator::new(Workload::AlternateDelete { batch_size: 4, kept: KeptInEachBatch::OneIn(2) }, 0);
    let operations: Vec<Operation> = (0..8).map(|_| half.next_operation()).collect();
    assert_eq!(operations[4..], [Operation::Unlink(2), Operation::Unlink(4), Operation::Create(5), Operation::Create(6)]);
    let mut three_quarters = WorkloadGenerator::new(Workload::AlternateDelete { batch_size: 8, kept: KeptInEachBatch::ThreeInFour }, 0);
    let three_quarters_operations: Vec<Operation> = (0..11).map(|_| three_quarters.next_operation()).collect();
    assert_eq!(three_quarters_operations[8..], [Operation::Unlink(4), Operation::Unlink(8), Operation::Create(9)]);
}

#[test]
fn random_workload_starts_with_a_create_and_only_unlinks_live_inodes() {
    let mut generator = WorkloadGenerator::new(Workload::RandomInsertDelete { insert_probability_percent: 55 }, 1);
    let mut live = BTreeSet::new();
    for _ in 0..10_000 {
        match generator.next_operation() {
            Operation::Create(inode) => assert!(live.insert(inode), "号不复用"),
            Operation::Unlink(inode) => assert!(live.remove(&inode), "只删活着的号"),
        }
    }
    assert!(!live.is_empty());
}

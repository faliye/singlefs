//! 正推腿探针（T10）：logged_ops 根的读者对「多于一条整理意图」「不按 key 升序」「保留的种类码」「补零非零」各拒成什么。
use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
use singlefs_core::logged_operations::{
    logged_operation_entries_of_the_root_entries, CompactionIntent, CompactionRegion, LoggedOperation,
    LoggedOperationEntry,
};

fn entry(issued_txg: u64, start_slot: u64) -> LoggedOperationEntry {
    LoggedOperationEntry {
        issued_txg: CheckpointTxg(issued_txg),
        operation: LoggedOperation::Compaction(CompactionIntent {
            region: CompactionRegion::of_one_cluster_segment(DeviceIdentity(0), SlotNumber(start_slot)).unwrap(),
            allocation_generation_at_issue: CheckpointTxg(issued_txg),
            empty_cluster_segments_at_issue: 7,
        }),
    }
}

#[test]
fn probe_forward_logged_ops_root_decoding_refusals() {
    let one = vec![entry(5, 50240).to_bytes()];
    let two_ascending = vec![entry(5, 50240).to_bytes(), entry(6, 50304).to_bytes()];
    let two_descending = vec![entry(6, 50304).to_bytes(), entry(5, 50240).to_bytes()];
    let mut reserved_kind = entry(5, 50240).to_bytes();
    reserved_kind[0] = 2;
    let mut padding_not_zero = entry(5, 50240).to_bytes();
    let last = padding_not_zero.len() - 1;
    padding_not_zero[last] = 1;
    let mut slot_count_not_64 = entry(5, 50240).to_bytes();
    slot_count_not_64[20] = 32;
    println!("name=probe_forward_logged_ops_decode one={:?}", logged_operation_entries_of_the_root_entries(&one).map(|e| e.len()));
    println!("name=probe_forward_logged_ops_decode two_ascending={:?}", logged_operation_entries_of_the_root_entries(&two_ascending).map(|e| e.len()));
    println!("name=probe_forward_logged_ops_decode two_descending={:?}", logged_operation_entries_of_the_root_entries(&two_descending).map(|e| e.len()));
    println!("name=probe_forward_logged_ops_decode reserved_kind={:?}", logged_operation_entries_of_the_root_entries(&[reserved_kind]).map(|e| e.len()));
    println!("name=probe_forward_logged_ops_decode padding_not_zero={:?}", logged_operation_entries_of_the_root_entries(&[padding_not_zero]).map(|e| e.len()));
    println!("name=probe_forward_logged_ops_decode slot_count_not_64={:?}", logged_operation_entries_of_the_root_entries(&[slot_count_not_64]).map(|e| e.len()));
    println!("name=probe_forward_logged_ops_decode entry_bytes={}", one[0].len());
}

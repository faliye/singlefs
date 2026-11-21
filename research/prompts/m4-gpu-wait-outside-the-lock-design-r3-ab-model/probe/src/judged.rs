//! 池级 checker（`singlefs_checker::walk::check_pool_image`）对整个池跑一遍，交回红的不变量名，与几处共用的小函数。
use singlefs_checker::image::InvariantVerdict;
use singlefs_checker::walk::check_pool_image;
use singlefs_core::allocator::AllocationRecord;
use singlefs_core::logged_operations::CompactionRegion;
use singlefs_core::transaction::TransactionOutput;

use crate::pool::{memory_pool_of, Devices};

pub fn short(text: &str, limit: usize) -> String {
    text.split_whitespace().collect::<Vec<_>>().join("_").chars().take(limit).collect()
}

/// 红（`Violated`）的不变量名，按 checker 交回的次序。
pub fn violated_invariants(devices: &Devices) -> Vec<&'static str> {
    check_pool_image(&memory_pool_of(devices))
        .iter()
        .filter(|(_, verdict)| matches!(verdict, InvariantVerdict::Violated(_)))
        .map(|(name, _)| *name)
        .collect()
}

/// R 那块盘上起点在 R 里、未释放的记录。
pub fn unreleased_records_in(version: &TransactionOutput, region: CompactionRegion) -> Vec<&AllocationRecord> {
    version
        .allocation_records
        .iter()
        .filter(|record| {
            record.device == region.device()
                && !record.is_released
                && record.slot.0 >= region.start_slot().0
                && record.slot.0 < region.end_slot().0
        })
        .collect()
}

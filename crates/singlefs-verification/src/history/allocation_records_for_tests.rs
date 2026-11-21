//! 单元测试共用的分配记录样本。

use super::*;

pub(super) fn record(
    slot: u64,
    span_slots: u16,
    generation: u64,
    is_released: bool,
) -> AllocationRecord {
    AllocationRecord {
        device: DeviceIdentity(0),
        slot: singlefs_core::address::SlotNumber(slot),
        span_slots,
        generation: CheckpointTxg(generation),
        is_released,
        quarantine_mark: singlefs_core::allocator::AllocationRecordQuarantineMark::NotQuarantined,
    }
}

//! 空发布与工作负载发布，与一次发布自己释放的槽。

use super::*;

/// 空发布 E：`publish_version` 带 `file: None`、`new_inode_records` 为空（骨架发布 字面），与 `publish_overwrite`
/// 同一条发布路径，只是不带文件计划（形态照 `transaction.rs` 里 `publish_new_inodes` 构造 `PublishPlan` 的写法）。
fn publish_empty<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    instance: InstanceGeneration,
) -> Result<TransactionOutput, PublishError> {
    let txg = CheckpointTxg(previous.root.checkpoint_txg.0 + 1);
    let mut pool = PoolWriter::new(parameters, devices);
    publish_version(
        &mut pool,
        allocator,
        PublishPlan {
            txg,
            counter: previous.record.counter + 1,
            transaction: 0,
            highest_transaction_number_before_this_publish: previous
                .highest_transaction_number_in_this_instance,
            instance,
            back_chain: back_chain_of(&previous.record_bytes),
            file: None,
            new_inode_records: &[],
            instance_table: InstanceTablePlan::Carry(previous.root.instance_table),
            tree_birth_txg: previous.tree_birth_txg(),
            tree_identifier_watermark: previous.root.tree_identifier_watermark,
            rollback_floor: previous.root.rollback_floor,
        },
        Some(previous),
    )
}

/// 工作负载的一次发布是哪一种（骨架负载：段内第 k 次是 O ⟺ `k mod m == 0`，ρ = 1 ⇒ m = 1，ρ = 1/4 ⇒ m = 4，其余是 E）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum WorkloadPublishKind {
    Overwrite,
    EmptyPublish,
}

impl WorkloadPublishKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            WorkloadPublishKind::Overwrite => "overwrite",
            WorkloadPublishKind::EmptyPublish => "empty_publish",
        }
    }
}

/// 工作负载的一次发布：O 走 `publish_overwrite`，E 走 [`publish_empty`]。`content_salt` 同时定内容与写入时刻，逐次不同。
pub(super) fn publish_one_workload<Device: BlockDevice>(
    parameters: &MakeFilesystemParameters,
    devices: &mut [(DeviceIdentity, Device)],
    allocator: &mut PoolAllocator,
    previous: &TransactionOutput,
    instance: InstanceGeneration,
    kind: WorkloadPublishKind,
    content_salt: u64,
) -> Result<TransactionOutput, PublishError> {
    match kind {
        WorkloadPublishKind::Overwrite => {
            let content = overwrite_content(content_salt);
            let mut pool = PoolWriter::new(parameters, devices);
            publish_overwrite(
                &mut pool,
                allocator,
                previous,
                FirstFile {
                    content: &content,
                    write_time_seconds: FIXED_WRITE_TIME_SECONDS + content_salt,
                },
                instance,
            )
        }
        WorkloadPublishKind::EmptyPublish => {
            publish_empty(parameters, devices, allocator, previous, instance)
        }
    }
}

/// R8「这次发布自己的释放」：D0 上 `is_released` 且 `generation == txg` 的记录，跨度求和，单位槽（登记「六」R8 字面）。
pub(super) fn self_release_slots_of_this_publish(
    allocator: &PoolAllocator,
    device: DeviceIdentity,
    txg: CheckpointTxg,
) -> u64 {
    allocator
        .records()
        .iter()
        .filter(|record| record.device == device && record.is_released && record.generation == txg)
        .map(|record| u64::from(record.span_slots))
        .sum()
}

#[cfg(test)]
mod tests;

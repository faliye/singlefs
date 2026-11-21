//! U11、U13 的新形态（第 4 次重跑登记第一节 1.2）：管理员回退是挂着时的一次向前发布；回退到最旧候选与 deferred 为零的合成基底。

use super::*;

// ============================================================================================
// U11、U13 的新形态（第 4 次重跑登记第一节 1.2）：管理员回退是挂着时的一次向前发布（D23（journal 的角色与格式） 已定项 14）。
// ============================================================================================

pub(super) struct ForwardRollbackScenario {
    pub(super) txg_before_the_rollback: u64,
    pub(super) txg_of_the_rollback_publish: u64,
    pub(super) instance_before: u32,
    pub(super) instance_after: u32,
    pub(super) instance_table_rows_before: usize,
    pub(super) instance_table_rows_after: usize,
    pub(super) remount_instance: u32,
    pub(super) isolated_after_the_plain_remount: Vec<(DeviceIdentity, u64)>,
    pub(super) abandoned_roots_after_the_plain_remount: usize,
}

fn instance_table_row_count(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    root: &RootRecord,
) -> usize {
    instance_table_chain_of_root(devices, root)
        .expect("现行那一版的实例表读得出")
        .records
        .rows
        .len()
}

fn abandoned_root_count(
    devices: &[(DeviceIdentity, SparseBlockDevice)],
    parameters: &MakeFilesystemParameters,
) -> usize {
    let roots = readable_roots(
        devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let newest = roots
        .iter()
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .copied()
        .expect("环里有根");
    let table = instance_table_chain_of_root(devices, &newest)
        .expect("最新根的实例表读得出")
        .records;
    roots
        .iter()
        .filter(|root| root_is_abandoned_by_the_instance_table(root, &table))
        .count()
}

/// U11 的新形态（登记第一节 1.2 第二行）：A → B → 干净重开 → C → 同一次挂载里挂着回退到 A → 普通重开。
pub(super) fn run_forward_rollback_scenario() -> ForwardRollbackScenario {
    let parameters = parameters();
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("U11 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("U11 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("U11 暖机")
    };
    let first = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("U11 A")
    };
    let first_txg = first.root.checkpoint_txg;
    let first_instance = first.root.instance;
    {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut allocator,
            &first,
            FirstFile {
                content: &vec![0u8; 4100],
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 60,
            },
            instance,
        )
        .expect("U11 B");
    }
    let mounted = mount_writable(&parameters, &mut devices).expect("U11 干净重开");
    let mut reopened_allocator = mounted.allocator;
    let reopened_instance = mounted.output.instance;
    let reopened = mounted
        .current
        .into_file_version()
        .expect("U11 重开之后现行版本带文件");
    let mut current = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut pool,
            &mut reopened_allocator,
            &reopened,
            FirstFile {
                content: &vec![0u8; 2500],
                write_time_seconds: FIXED_WRITE_TIME_SECONDS + 120,
            },
            reopened_instance,
        )
        .expect("U11 C")
    };
    let txg_before_the_rollback = current.root.checkpoint_txg.0;
    let instance_before = current.root.instance.0;
    let instance_table_rows_before = instance_table_row_count(&devices, &current.root);
    roll_back_by_a_forward_publish(
        &parameters,
        &mut devices,
        &mut reopened_allocator,
        &mut current,
        RollbackTarget {
            instance: first_instance,
            checkpoint_txg: first_txg,
        },
    )
    .expect("U11 挂着回退到 A");
    let txg_of_the_rollback_publish = current.root.checkpoint_txg.0;
    let instance_after = current.root.instance.0;
    let instance_table_rows_after = instance_table_row_count(&devices, &current.root);
    let remounted = mount_writable(&parameters, &mut devices).expect("U11 回退之后普通重开");
    ForwardRollbackScenario {
        txg_before_the_rollback,
        txg_of_the_rollback_publish,
        instance_before,
        instance_after,
        instance_table_rows_before,
        instance_table_rows_after,
        remount_instance: remounted.output.instance.0,
        isolated_after_the_plain_remount: remounted.output.isolated_slots_per_device,
        abandoned_roots_after_the_plain_remount: abandoned_root_count(&devices, &parameters),
    }
}

pub(super) struct OldestCandidateRollback {
    pub(super) overwrites: u64,
    pub(super) target_txg: u64,
    pub(super) txg_before: u64,
    pub(super) txg_after: u64,
    pub(super) instance_before: u32,
    pub(super) instance_after: u32,
    pub(super) instance_table_rows_before: usize,
    pub(super) instance_table_rows_after: usize,
    pub(super) item1_slots: u64,
    pub(super) item5_slots: u64,
    pub(super) allocated_minus_deferred_verdict: InvariantVerdict,
}

/// U13 (a)（登记第一节 1.2 第一行）：mkfs 同一个进程、产品路径分配器、S = 8，覆盖写 3N = 72 次（N = 3S）→ 同一个进程里挂着回退到
/// 候选集里 txg 最小的根。第 5 项只报数（附带，不进判据）。
pub(super) fn run_rollback_to_the_oldest_candidate() -> OldestCandidateRollback {
    let parameters = parameters();
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("U13 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("U13 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("U13 暖机")
    };
    let mut current = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("U13 新池新建文件")
    };
    let overwrites = 3 * E156_ROOT_RING_REGIONS * E156_SLOTS_PER_REGION;
    for step in 1..=overwrites {
        current = publish_one_workload(
            &parameters,
            devices.as_mut_slice(),
            &mut allocator,
            &current,
            instance,
            WorkloadPublishKind::Overwrite,
            step,
        )
        .expect("U13 前缀覆盖写");
    }
    let effective_floor = effective_rollback_floor(
        &devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    );
    let current_table = instance_table_chain_of_root(&devices, &current.root)
        .expect("现行那一版的实例表读得出")
        .records;
    let oldest = readable_roots(
        &devices,
        &parameters.region_devices,
        &parameters.geometry,
        &parameters.filesystem_identifier,
    )
    .into_iter()
    .filter(|root| {
        root.checkpoint_txg >= effective_floor
            && root.checkpoint_txg.0 >= 3
            && !root_is_abandoned_by_the_instance_table(root, &current_table)
    })
    .min_by_key(|root| root.checkpoint_txg)
    .expect("回退候选集至少一个根");
    let txg_before = current.root.checkpoint_txg.0;
    let instance_before = current.root.instance.0;
    let instance_table_rows_before = instance_table_row_count(&devices, &current.root);
    roll_back_by_a_forward_publish(
        &parameters,
        &mut devices,
        &mut allocator,
        &mut current,
        RollbackTarget {
            instance: oldest.instance,
            checkpoint_txg: oldest.checkpoint_txg,
        },
    )
    .expect("U13 挂着回退到候选集里最旧的根");
    let pool = memory_pool_snapshot(&devices);
    let (item1_slots, _item2_slots, item5_slots) =
        mirror_accounting_row_slots(&pool, current.unit(TransactionUnit::AccountingTree).slot.0);
    OldestCandidateRollback {
        overwrites,
        target_txg: oldest.checkpoint_txg.0,
        txg_before,
        txg_after: current.root.checkpoint_txg.0,
        instance_before,
        instance_after: current.root.instance.0,
        instance_table_rows_before,
        instance_table_rows_after: instance_table_row_count(&devices, &current.root),
        item1_slots,
        item5_slots,
        allocated_minus_deferred_verdict: verdict(
            "oldest-candidate-rollback/allocated-minus-deferred",
            &pool,
            "I-3.11",
        ),
    }
}

/// β0（S = 8、新池新建文件之后）与 β_syn（β0 的镜像把 D0 第 1 项 − 1、第 5 项 − 1 成 0、第 2 项 + 1，重封）：U13 (b) 与 [`main`] 共用。
pub(super) struct SyntheticBase {
    pub(super) pool: MemoryPool,
    pub(super) accounting_slot: u64,
    pub(super) referenced: u64,
}

pub(super) fn synthetic_base_with_zero_deferred() -> SyntheticBase {
    let parameters = parameters();
    let mut devices = new_pool_devices();
    let genesis = make_filesystem(&parameters, &mut devices).expect("β0 mkfs");
    let mut allocator = allocator_after_make_filesystem(&parameters, &devices, &genesis);
    let instance = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        acquire_instance(&mut pool).expect("β0 取号")
    };
    let warm_up_output = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        warm_up(&mut pool, &genesis.root, instance).expect("β0 暖机")
    };
    let first = {
        let mut pool = PoolWriter::new(&parameters, devices.as_mut_slice());
        publish_first_file(
            &mut pool,
            &mut allocator,
            warm_up_output.roots.last().expect("暖机两代根"),
            FirstFile {
                content: &overwrite_content(0),
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            instance,
            &warm_up_output.last_record_bytes,
        )
        .expect("β0 新池新建文件")
    };
    let accounting_slot = first.unit(TransactionUnit::AccountingTree).slot.0;
    let tree_table_slot = first.unit(TransactionUnit::TreeTable).slot.0;
    let slot_bytes = i64::try_from(SLOT_BYTES).expect("16384");
    let pool = corrupt_device_zero_accounting(
        &memory_pool_snapshot(&devices),
        accounting_slot,
        tree_table_slot,
        &parameters.region_devices,
        parameters.geometry.fixed_structure_slot_spacing,
        &[(1u16, -slot_bytes), (5u16, -slot_bytes), (2u16, slot_bytes)],
    );
    SyntheticBase {
        pool,
        accounting_slot,
        referenced: referenced_slots(&first),
    }
}

#[cfg(test)]
mod tests;

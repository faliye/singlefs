//! 一个状态的读数：各盘的读数、残差、滞后与读出整个状态。

use super::*;

// ---------------------------------------------------------------------------------------------------------------------
// 一个状态上的读数（不含探测）
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(super) struct DeviceReading {
    pub(super) capacity_slots: u64,
    /// 现行账里仍分配的槽（从盘上读的现行那一版的分配记录）。
    pub(super) live_slots: u64,
    /// 同一个量从内存里的现行那一版取；两份对不上记 `records_agree_with_the_disk` 为假。
    pub(super) live_slots_in_memory_account: u64,
    pub(super) available_bytes: i128,
    /// 「挂载期承诺量 + 环窗口 + 不可回收 + 全池三项摊到这块盘的份」（登记 5.2 的 Cm），字节。
    pub(super) common_commitments_bytes: i128,
    pub(super) ring_window_slots: u64,
    pub(super) unblocked_slots: u64,
    pub(super) overlap_slots_allocated_and_isolated: u64,
    pub(super) isolated_slots: u64,
    pub(super) allocated_slots: u64,
    pub(super) deferred_slots: u64,
    /// 这次挂载开过的聚簇段里、三种挡位都没置的槽（登记「段内关着的空槽」）。
    pub(super) closed_free_slots_in_cluster_segments: u64,
}

impl DeviceReading {
    pub(super) fn available_slots_rounded_down(&self) -> i128 {
        self.available_bytes.div_euclid(i128::from(SLOT_BYTES))
    }

    /// (可用 + Cm) 换成槽：等于 容量 − 已分配 − 隔离（分解成立时，见 [`StateReading::decomposition_holds`]）。
    pub(super) fn available_plus_common_slots(&self) -> i128 {
        (self.available_bytes + self.common_commitments_bytes).div_euclid(i128::from(SLOT_BYTES))
    }
}

#[derive(Clone, Debug)]
pub(super) struct StateReading {
    pub(super) devices: Vec<DeviceReading>,
    pub(super) data_units: u64,
    pub(super) checkpoint_cost_blocks: u64,
    /// 滞后量（登记 5.1）：按全池口径的槽数。
    pub(super) lag_slots: u64,
    /// 释放代高于门槛、却既不在用户改动日志里也不在空发布日志里的已释放槽（该是 0）。
    pub(super) lag_unattributed_slots: u64,
    pub(super) oldest_valid_root_txg: u64,
    pub(super) fourth_newest_nonempty_valid_root_txg: Option<u64>,
    pub(super) cluster_segment_count: u64,
    pub(super) inode_tree_internal_levels: u64,
    pub(super) records_agree_with_the_disk: bool,
    /// 容量 − 已分配 − 隔离 = 可用 + Cm 在每块盘上都成立（Cm 的拆法没丢项）。
    pub(super) decomposition_holds: bool,
    pub(super) available_is_whole_slots: bool,
}

pub(super) fn floor_half(value: i128) -> i128 {
    value.div_euclid(2)
}

/// 现行账里的记录按盘汇总仍分配的槽数。
fn live_slots_per_device(
    records: &[singlefs_core::allocator::AllocationRecord],
    devices: usize,
) -> Vec<u64> {
    let mut live = vec![0u64; devices];
    for record in records.iter().filter(|record| !record.is_released) {
        let index = usize::try_from(record.device.0).expect("盘号");
        live[index] += u64::from(record.span_slots);
    }
    live
}

fn valid_root_txgs_in_the_ring(allocator: &PoolAllocator) -> Vec<u64> {
    let occupancy = allocator
        .root_ring_occupancy()
        .expect("分配器上装着根环：挂载与 mkfs 建的分配器都装");
    occupancy
        .ring_slots_known_to_hold_a_root()
        .into_iter()
        .filter_map(|ring_slot| occupancy.checkpoint_txg_of_the_valid_root_recorded_on(ring_slot))
        .map(|txg| txg.0)
        .collect()
}

/// 登记 5.2 的残差（单位槽，一块盘）：改写检查残差（登记记作 R2）= (可用 + Cm) − (没挡的 + 扣住账 − 重叠)；
/// 原式残差（登记记作 R3）= 可用 + Cm − 没挡的（欠账行 C356 原式）；阳性对照用的「改写检查残差去掉重叠项」。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Residuals {
    pub(super) rewritten_check_residual_with_the_held_and_overlap_terms: i128,
    pub(super) rewritten_check_residual_without_the_overlap_term: i128,
    pub(super) original_formula_residual: i128,
}

pub(super) fn residuals_of(
    available_plus_common_slots: i128,
    unblocked_slots: i128,
    overlap_allocated_and_isolated_slots: i128,
    held_ledger_slots: i128,
) -> Residuals {
    Residuals {
        rewritten_check_residual_with_the_held_and_overlap_terms: available_plus_common_slots
            - (unblocked_slots + held_ledger_slots - overlap_allocated_and_isolated_slots),
        rewritten_check_residual_without_the_overlap_term: available_plus_common_slots
            - (unblocked_slots + held_ledger_slots),
        original_formula_residual: available_plus_common_slots - unblocked_slots,
    }
}

/// 段内关着的空槽（登记第一节「段内关着的空槽」）：这次挂载开过的聚簇段里、三种挡位都没置的槽数；数据单元落不进去（D3 已定项 8 第 2 条）。
pub(super) fn closed_free_slots_in_segments(
    map: &singlefs_core::allocator::DeviceFreeMap,
    segments_opened_by_this_mount: &BTreeSet<SlotNumber>,
) -> u64 {
    let counted: usize = segments_opened_by_this_mount
        .iter()
        .map(|segment_start| {
            (0..CLUSTER_SEGMENT_SLOTS)
                .filter(|offset| map.is_free(SlotNumber(segment_start.0 + offset)))
                .count()
        })
        .sum();
    u64::try_from(counted).expect("槽数")
}

/// 滞后量那一项的读数（登记 5.1）：门槛 = max(第 4 新的非空有效根的 txg，环里最旧有效根的 txg)（非空有效根不足 4 个时第 4 新取最旧有效根）；
/// 滞后 = 现行账里已释放、释放代高于门槛、释放它的那次发布是用户改动（不是空发布）的槽，全池口径（各盘的跨度之和除以盘数）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LagReading {
    pub(super) lag_slots: u64,
    pub(super) unattributed_slots: u64,
    pub(super) oldest_valid_root_txg: u64,
    pub(super) fourth_newest_nonempty_valid_root_txg: Option<u64>,
}

pub(super) fn lag_reading_of(
    records: &[singlefs_core::allocator::AllocationRecord],
    valid_root_txgs: &[u64],
    log: &PublishLog,
    device_count: u64,
) -> LagReading {
    let oldest_valid_root_txg = valid_root_txgs.iter().copied().min().unwrap_or(0);
    let mut nonempty_valid: Vec<u64> = valid_root_txgs
        .iter()
        .copied()
        .filter(|txg| log.user_change_txgs.contains(txg))
        .collect();
    nonempty_valid.sort_unstable_by(|left, right| right.cmp(left));
    let fourth_newest_nonempty_valid_root_txg = nonempty_valid.get(3).copied();
    let threshold = fourth_newest_nonempty_valid_root_txg
        .unwrap_or(oldest_valid_root_txg)
        .max(oldest_valid_root_txg);
    let mut lagging_span_slots = 0u64;
    let mut unattributed_span_slots = 0u64;
    for record in records
        .iter()
        .filter(|record| record.is_released && record.generation.0 > threshold)
    {
        let txg = record.generation.0;
        if log.user_change_txgs.contains(&txg) {
            lagging_span_slots += u64::from(record.span_slots);
        } else if !log.empty_publish_txgs.contains(&txg) {
            unattributed_span_slots += u64::from(record.span_slots);
        }
    }
    LagReading {
        lag_slots: lagging_span_slots / device_count,
        unattributed_slots: unattributed_span_slots,
        oldest_valid_root_txg,
        fourth_newest_nonempty_valid_root_txg,
    }
}

pub(super) fn read_state(pool: &Pool) -> StateReading {
    let session = &pool.session;
    let version = current_version(&session.current);
    let allocator = &session.allocator;
    let reading: AdmissionReading = admission_reading_before_a_publish(allocator, Some(version));
    let replicas = reading.replicas();
    let pool_wide = reading.pool_wide();
    let shares: i128 = [
        pool_wide.pending_delete,
        pool_wide.committed_reservation,
        pool_wide.checkpoint_reserve_pool,
    ]
    .iter()
    .map(|commitment| i128::from(commitment.share_of_one_device(replicas).0))
    .sum();
    let available = reading.available_on_each_device();
    let disk_records = allocation_records_under_root(&pool.devices, &version.root);
    let device_count = allocator.devices.len();
    let live_in_memory = live_slots_per_device(&version.allocation_records, device_count);
    let (live_on_disk, records_agree_with_the_disk) = match &disk_records {
        Ok(records) => {
            let mut from_disk = records.clone();
            from_disk.sort();
            let mut from_memory = version.allocation_records.clone();
            from_memory.sort();
            (
                live_slots_per_device(records, device_count),
                from_disk == from_memory,
            )
        }
        Err(_) => (live_in_memory.clone(), false),
    };
    let lag = lag_reading_of(
        &version.allocation_records,
        &valid_root_txgs_in_the_ring(allocator),
        &pool.log,
        u64::try_from(device_count).expect("盘数"),
    );
    let cluster_segments = allocator.cluster_segments();
    let mut decomposition_holds = true;
    let mut available_is_whole_slots = true;
    let devices: Vec<DeviceReading> = allocator
        .devices
        .iter()
        .zip(reading.per_device())
        .zip(available.iter())
        .enumerate()
        .map(|(index, ((map, terms), (_, available_bytes)))| {
            let common_commitments_bytes = i128::from(terms.mount_time_commitment.0)
                + i128::from(terms.ring_window.0)
                + i128::from(terms.unreclaimable.0)
                + shares;
            let closed_free_slots_in_cluster_segments =
                closed_free_slots_in_segments(map, cluster_segments);
            let capacity_bytes = i128::from(terms.capacity.0);
            let by_decomposition = capacity_bytes
                - i128::from(terms.allocated.0)
                - i128::from(
                    terms
                        .isolated_for_abandoned_roots_and_unapplied_journal_records
                        .0,
                );
            if available_bytes.0 + common_commitments_bytes != by_decomposition {
                decomposition_holds = false;
            }
            if available_bytes.0.rem_euclid(i128::from(SLOT_BYTES)) != 0 {
                available_is_whole_slots = false;
            }
            DeviceReading {
                capacity_slots: map.unit_area_slots(),
                live_slots: live_on_disk[index],
                live_slots_in_memory_account: live_in_memory[index],
                available_bytes: available_bytes.0,
                common_commitments_bytes,
                ring_window_slots: terms.ring_window.0 / SLOT_BYTES,
                unblocked_slots: map.unblocked_slots(),
                overlap_slots_allocated_and_isolated: map
                    .slots_counted_both_as_allocated_and_as_isolated(),
                isolated_slots: map.isolated_slots(),
                allocated_slots: map.allocated_slots(),
                deferred_slots: map.deferred_slots(),
                closed_free_slots_in_cluster_segments,
            }
        })
        .collect();
    let inode_tree_internal_levels = match version.inode_leaf_containers.len() {
        0 | 1 => 0,
        containers => stop_with(
            "inode_tree_has_more_than_one_leaf_container",
            &format!("containers={containers}"),
        ),
    };
    StateReading {
        devices,
        data_units: u64::try_from(version.data_pointers.len()).expect("单元数"),
        checkpoint_cost_blocks: checkpoint_cost_of_the_version_to_build_on(
            Some(version),
            allocator,
        )
        .0,
        lag_slots: lag.lag_slots,
        lag_unattributed_slots: lag.unattributed_slots,
        oldest_valid_root_txg: lag.oldest_valid_root_txg,
        fourth_newest_nonempty_valid_root_txg: lag.fourth_newest_nonempty_valid_root_txg,
        cluster_segment_count: u64::try_from(cluster_segments.len()).expect("段数"),
        inode_tree_internal_levels,
        records_agree_with_the_disk,
        decomposition_holds,
        available_is_whole_slots,
    }
}

use super::*;

// ============================================================================================
// X15：准入读数逐项（D28 已定项 1 式子里除「容量」「已分配」「不可回收」之外的六项）。
// ============================================================================================

/// 两个读数入口（登记 X15）：可写挂载那一判（rows0 取这次挂载的）、下一次发布那一判（rows0 + 1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReadingEntry {
    OfAWritableMount,
    BeforeAPublish,
}

impl ReadingEntry {
    pub(super) const BOTH: [ReadingEntry; 2] =
        [ReadingEntry::OfAWritableMount, ReadingEntry::BeforeAPublish];

    pub(super) fn name(self) -> &'static str {
        match self {
            ReadingEntry::OfAWritableMount => "writable_mount",
            ReadingEntry::BeforeAPublish => "before_a_publish",
        }
    }

    /// D28 已定项 3：「发布路径用 rows0 + 1」，可写挂载自己判时照旧按这次的 rows0。
    pub(super) fn instance_rows_of_the_switch_reserve(
        self,
        instance_rows_of_this_mount: u64,
    ) -> u64 {
        match self {
            ReadingEntry::OfAWritableMount => instance_rows_of_this_mount,
            ReadingEntry::BeforeAPublish => instance_rows_of_this_mount + 1,
        }
    }
}

/// 一块盘上读数的各项，全按字节；全池三项已摊成「副本之和 ÷ 副本数」。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DeviceTerms {
    pub(super) device: u32,
    pub(super) capacity_bytes: u64,
    pub(super) allocated_bytes: u64,
    pub(super) unreclaimable_bytes: u64,
    pub(super) mount_time_commitment_bytes: u64,
    pub(super) abandoned_root_exclusive_bytes: u64,
    pub(super) pending_delete_share_bytes: u64,
    pub(super) committed_reservation_share_bytes: u64,
    pub(super) checkpoint_reserve_pool_share_bytes: u64,
    pub(super) ring_window_bytes: u64,
    pub(super) available_bytes: i128,
}

impl DeviceTerms {
    /// 准入预留的六项（名字，字节），次序固定。
    pub(super) fn admission_reserve_items(&self) -> [(&'static str, u64); 6] {
        [
            ("mount_time_commitment", self.mount_time_commitment_bytes),
            (
                "abandoned_root_exclusive",
                self.abandoned_root_exclusive_bytes,
            ),
            ("pending_delete_share", self.pending_delete_share_bytes),
            (
                "committed_reservation_share",
                self.committed_reservation_share_bytes,
            ),
            (
                "checkpoint_reserve_pool_share",
                self.checkpoint_reserve_pool_share_bytes,
            ),
            ("ring_window", self.ring_window_bytes),
        ]
    }

    /// 准入预留(d) = 六项之和（登记 X15）。
    pub(super) fn admission_reserve_bytes(&self) -> u64 {
        self.mount_time_commitment_bytes
            + self.abandoned_root_exclusive_bytes
            + self.pending_delete_share_bytes
            + self.committed_reservation_share_bytes
            + self.checkpoint_reserve_pool_share_bytes
            + self.ring_window_bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReadingSnapshot {
    pub(super) entry: ReadingEntry,
    /// 读数里切换预留按哪个行数算（这次挂载的 rows0，或它加一）。
    pub(super) instance_rows_of_the_switch_reserve: u64,
    pub(super) replicas: u64,
    pub(super) checkpoint_reserve_pool_bytes: u64,
    pub(super) pending_delete_bytes: u64,
    pub(super) committed_reservation_bytes: u64,
    /// 要接的那一版有树时它的退役树内部层数（环窗口 W 里的 H_退役）；树表 0 条的一版是 `None`。
    pub(super) retirement_tree_internal_levels: Option<u64>,
    pub(super) devices: Vec<DeviceTerms>,
}

fn snapshot_of_a_reading(
    entry: ReadingEntry,
    reading: &AdmissionReading,
    instance_rows_of_this_mount: u64,
    retirement_tree_internal_levels: Option<u64>,
) -> ReadingSnapshot {
    let replicas = reading.replicas();
    let pool_wide = reading.pool_wide();
    let available_by_device = reading.available_on_each_device();
    let devices = reading
        .per_device()
        .iter()
        .zip(available_by_device.iter())
        .map(|(terms, (available_device, available))| {
            assert_eq!(
                terms.device, *available_device,
                "可用(d) 与逐盘各项按同一个盘序"
            );
            DeviceTerms {
                device: terms.device.0,
                capacity_bytes: terms.capacity.0,
                allocated_bytes: terms.allocated.0,
                unreclaimable_bytes: terms.unreclaimable.0,
                mount_time_commitment_bytes: terms.mount_time_commitment.0,
                abandoned_root_exclusive_bytes: terms
                    .isolated_for_abandoned_roots_and_unapplied_journal_records
                    .0,
                pending_delete_share_bytes: pool_wide
                    .pending_delete
                    .share_of_one_device(replicas)
                    .0,
                committed_reservation_share_bytes: pool_wide
                    .committed_reservation
                    .share_of_one_device(replicas)
                    .0,
                checkpoint_reserve_pool_share_bytes: pool_wide
                    .checkpoint_reserve_pool
                    .share_of_one_device(replicas)
                    .0,
                ring_window_bytes: terms.ring_window.0,
                available_bytes: available.0,
            }
        })
        .collect();
    ReadingSnapshot {
        entry,
        instance_rows_of_the_switch_reserve: entry
            .instance_rows_of_the_switch_reserve(instance_rows_of_this_mount),
        replicas: replicas.replicas(),
        checkpoint_reserve_pool_bytes: pool_wide.checkpoint_reserve_pool.0,
        pending_delete_bytes: pool_wide.pending_delete.0,
        committed_reservation_bytes: pool_wide.committed_reservation.0,
        retirement_tree_internal_levels,
        devices,
    }
}

/// 两个入口各取一份读数：读数在 `mount_writable` 交回那一刻取。
pub(super) fn both_readings_of_a_mount(
    mounted: &Mounted,
    slots_per_region: RootRingSlotsPerRegion,
) -> Vec<ReadingSnapshot> {
    let version_of_trees = mounted.current.version_of_trees();
    let instance_rows_of_this_mount = mounted
        .allocator
        .instance_rows_after_this_mounts_row_publish();
    let retirement_tree_internal_levels =
        version_of_trees.map(|version| version.retirement_tree_internal_levels());
    let of_a_writable_mount = admission_reading_of_a_writable_mount(
        &mounted.allocator,
        version_of_trees,
        slots_per_region,
    );
    let before_a_publish = admission_reading_before_a_publish(&mounted.allocator, version_of_trees);
    vec![
        snapshot_of_a_reading(
            ReadingEntry::OfAWritableMount,
            &of_a_writable_mount,
            instance_rows_of_this_mount,
            retirement_tree_internal_levels,
        ),
        snapshot_of_a_reading(
            ReadingEntry::BeforeAPublish,
            &before_a_publish,
            instance_rows_of_this_mount,
            retirement_tree_internal_levels,
        ),
    ]
}

/// R4c（登记第六节）：准入预留六项各占总和的份额，最大一项份额 ≥ 50% 记「主导」，否则「没有一项过半」。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Dominance {
    Dominant {
        item: &'static str,
        share_permille: u64,
    },
    NoItemAboveHalf {
        largest_item: &'static str,
        share_permille: u64,
    },
}

impl Dominance {
    fn is_dominant(&self) -> bool {
        match self {
            Dominance::Dominant { .. } => true,
            Dominance::NoItemAboveHalf { .. } => false,
        }
    }

    pub(super) fn verdict_text(&self) -> String {
        match self {
            Dominance::Dominant {
                item,
                share_permille,
            } => {
                format!("dominant:{item}:{share_permille}permille")
            }
            Dominance::NoItemAboveHalf {
                largest_item,
                share_permille,
            } => format!("no_item_above_half:{largest_item}:{share_permille}permille"),
        }
    }
}

pub(super) fn share_permille(part: u64, total: u64) -> u64 {
    if total == 0 {
        return 0;
    }
    u64::try_from(u128::from(part) * 1000 / u128::from(total)).expect("千分比不超过 1000")
}

/// R4c 的门槛：份额 ≥ 50%（登记第六节），写成千分比。
const DOMINANCE_THRESHOLD_PERMILLE: u64 = 500;

pub(super) fn dominance_of(items: &[(&'static str, u64)]) -> Dominance {
    dominance_with_threshold(items, DOMINANCE_THRESHOLD_PERMILLE)
}

fn dominance_with_threshold(items: &[(&'static str, u64)], threshold_permille: u64) -> Dominance {
    let total: u64 = items.iter().map(|(_, bytes)| *bytes).sum();
    let (largest_item, largest_bytes) = items
        .iter()
        .copied()
        .max_by_key(|(_, bytes)| *bytes)
        .expect("六项不为空");
    let share_permille = share_permille(largest_bytes, total);
    if total > 0
        && 1000 * u128::from(largest_bytes) >= u128::from(threshold_permille) * u128::from(total)
    {
        Dominance::Dominant {
            item: largest_item,
            share_permille,
        }
    } else {
        Dominance::NoItemAboveHalf {
            largest_item,
            share_permille,
        }
    }
}

/// 判别力自证（登记第八节 ③）：把门槛挪到最大一项份额的另一侧——主导的挪到刚好高过最大份额，没过半的挪到最大份额的整千分点——判定必须转。
fn moved_threshold_permille(items: &[(&'static str, u64)]) -> u64 {
    let total: u64 = items.iter().map(|(_, bytes)| *bytes).sum();
    let largest_bytes = items
        .iter()
        .map(|(_, bytes)| *bytes)
        .max()
        .expect("六项不为空");
    let largest_share_permille = share_permille(largest_bytes, total);
    if dominance_of(items).is_dominant() {
        largest_share_permille + 1
    } else {
        largest_share_permille
    }
}

pub(super) fn dominance_flips_when_the_threshold_is_moved(items: &[(&'static str, u64)]) -> bool {
    dominance_of(items).is_dominant()
        != dominance_with_threshold(items, moved_threshold_permille(items)).is_dominant()
}

/// 对一份读数的 K30、K31、K32 与 G7（登记 7.1、第十节）。字段为 `false` 的格都在判决行点名。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ReadingChecks {
    /// K30：待删占用 = 0。
    pub(super) pending_delete_is_zero: bool,
    /// K31 用到的 c：checkpoint 保留池 ÷ (16384 × 副本数)，除不尽记 `None`。
    pub(super) checkpoint_cost_blocks_by_the_pool: Option<u64>,
    /// K31 反过来从挂载期承诺量解出的 c（每块盘一个），解不出整数记 `None`。
    pub(super) checkpoint_cost_blocks_by_the_commitment: Vec<Option<u64>>,
    /// K31：每块盘的挂载期承诺量 = 4 × (32768 × max(1, ⌈(rows + 3) ÷ 369⌉) + 3 × c × 16384)。
    pub(super) mount_time_commitment_by_the_clauses: bool,
    /// K32：环窗口(d) 与条款句现算的相同（树表 0 条的一版上必须是 0）。
    pub(super) ring_window_by_the_clauses: bool,
    /// 容量 = 本地几何算出的单元区槽数 × 16384。
    pub(super) capacity_by_the_local_geometry: bool,
}

impl ReadingChecks {
    pub(super) fn all_hold(&self) -> bool {
        self.pending_delete_is_zero
            && self.checkpoint_cost_blocks_by_the_pool.is_some()
            && self
                .checkpoint_cost_blocks_by_the_commitment
                .iter()
                .all(|blocks| {
                    blocks.is_some() && *blocks == self.checkpoint_cost_blocks_by_the_pool
                })
            && self.mount_time_commitment_by_the_clauses
            && self.ring_window_by_the_clauses
            && self.capacity_by_the_local_geometry
    }
}

/// 条款句现算的挂载期承诺量，每块盘（D28 已定项 3，c 是 checkpoint 保留池折成的块数）。
fn mount_time_commitment_by_the_clauses(
    instance_rows_of_the_switch_reserve: u64,
    checkpoint_cost_blocks: u64,
) -> u64 {
    (INSTANCE_SWITCHES_ALLOWED_PER_MOUNT + 1)
        * (INSTANCE_TABLE_PAGE_BYTES
            * instance_table_chain_pages(instance_rows_of_the_switch_reserve)
            + WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH
                * checkpoint_cost_blocks
                * SLOT_BYTES)
}

/// 从承诺量解出 c：先去掉链重写那一半，剩下的除以 (R × 16384)，除不尽记 `None`。
fn checkpoint_cost_blocks_solved_from_the_commitment(
    commitment_bytes: u64,
    instance_rows_of_the_switch_reserve: u64,
) -> Option<u64> {
    let per_switch = commitment_bytes.checked_div(INSTANCE_SWITCHES_ALLOWED_PER_MOUNT + 1)?;
    if per_switch * (INSTANCE_SWITCHES_ALLOWED_PER_MOUNT + 1) != commitment_bytes {
        return None;
    }
    let warm_up_bytes = per_switch.checked_sub(
        INSTANCE_TABLE_PAGE_BYTES * instance_table_chain_pages(instance_rows_of_the_switch_reserve),
    )?;
    let unit = WARM_UP_EMPTY_PUBLISHES_AT_MOST_PER_INSTANCE_SWITCH * SLOT_BYTES;
    (warm_up_bytes % unit == 0).then_some(warm_up_bytes / unit)
}

pub(super) fn checks_of_a_reading(
    snapshot: &ReadingSnapshot,
    device_bytes: u64,
    journal_ring_bytes: u64,
    slots_per_region: u64,
) -> ReadingChecks {
    let pool_bytes_per_block = SLOT_BYTES * snapshot.replicas;
    let checkpoint_cost_blocks_by_the_pool = (snapshot
        .checkpoint_reserve_pool_bytes
        .is_multiple_of(pool_bytes_per_block))
    .then_some(snapshot.checkpoint_reserve_pool_bytes / pool_bytes_per_block);
    let rows = snapshot.instance_rows_of_the_switch_reserve;
    let ring_window_expected_bytes = match snapshot.retirement_tree_internal_levels {
        None => 0,
        Some(levels) => {
            ring_window_by_the_clauses(device_bytes, journal_ring_bytes, slots_per_region, levels)
                .ring_window_bytes()
        }
    };
    ReadingChecks {
        pending_delete_is_zero: snapshot.pending_delete_bytes == 0
            && snapshot
                .devices
                .iter()
                .all(|terms| terms.pending_delete_share_bytes == 0),
        checkpoint_cost_blocks_by_the_pool,
        checkpoint_cost_blocks_by_the_commitment: snapshot
            .devices
            .iter()
            .map(|terms| {
                checkpoint_cost_blocks_solved_from_the_commitment(
                    terms.mount_time_commitment_bytes,
                    rows,
                )
            })
            .collect(),
        mount_time_commitment_by_the_clauses: checkpoint_cost_blocks_by_the_pool.is_some_and(
            |blocks| {
                snapshot.devices.iter().all(|terms| {
                    terms.mount_time_commitment_bytes
                        == mount_time_commitment_by_the_clauses(rows, blocks)
                })
            },
        ),
        ring_window_by_the_clauses: snapshot
            .devices
            .iter()
            .all(|terms| terms.ring_window_bytes == ring_window_expected_bytes),
        capacity_by_the_local_geometry: snapshot.devices.iter().all(|terms| {
            terms.capacity_bytes == unit_area_slots(device_bytes, journal_ring_bytes) * SLOT_BYTES
        }),
    }
}

#[cfg(test)]
mod tests;

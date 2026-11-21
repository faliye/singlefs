//! 岔路 1 的族 Hh：一版的事实、从盘上读的引用槽、门槛、洞数与逐盘多扣的槽，与一个状态的整份读数。

use super::*;

/// 一个 txg 的那一版在装置这边留下的事实（W9 的独立路径「装置自己为那个 txg 留的发布输出」）：它引用的槽（两盘同槽）与树表落点。
#[derive(Clone, Debug)]
pub(super) struct PublishedVersionFacts {
    pub(super) referenced_slots: BTreeSet<u64>,
    pub(super) tree_table_slot: u64,
}

pub(super) fn facts_of_published_version(output: &TransactionOutput) -> PublishedVersionFacts {
    let mut referenced_slots = BTreeSet::new();
    let mut has_instance_table = false;
    for unit in &output.units {
        if matches!(unit.identity, TransactionUnit::InstanceTable) {
            has_instance_table = true;
        }
        for slot in unit.slot.0..unit.slot.0 + unit.identity.span_slots() {
            referenced_slots.insert(slot);
        }
    }
    if !has_instance_table {
        let instance_table_slot =
            genesis_slot_on_device_zero(&output.root.instance_table.locations).0;
        for slot in instance_table_slot..instance_table_slot + INSTANCE_TABLE_SPAN_SLOTS {
            referenced_slots.insert(slot);
        }
    }
    PublishedVersionFacts {
        referenced_slots,
        tree_table_slot: output.unit(TransactionUnit::TreeTable).slot.0,
    }
}

/// 零单元那几版（mkfs 的第 0 代与之后两次暖机）：引用的是根记录两条指针直接指着的实例表与第 0 版树表。
pub(super) fn facts_of_zero_unit_version(root: &RootRecord) -> PublishedVersionFacts {
    let tree_table_slot = genesis_slot_on_device_zero(&root.tree_table.locations).0;
    let instance_table_slot = genesis_slot_on_device_zero(&root.instance_table.locations).0;
    let mut referenced_slots: BTreeSet<u64> =
        (instance_table_slot..instance_table_slot + INSTANCE_TABLE_SPAN_SLOTS).collect();
    for slot in tree_table_slot..tree_table_slot + TransactionUnit::TreeTable.span_slots() {
        referenced_slots.insert(slot);
    }
    PublishedVersionFacts {
        referenced_slots,
        tree_table_slot,
    }
}

/// W9 的盘上路径：同一条根，按 `recovery::allocation_records_under_root` 读出的仍分配记录，展开成这块盘上的槽。
/// 没有分配记录树的那几版（`allocation_records_under_root` 对第 0 代树表给空）按根记录里实例表与树表两条指针的这块盘那一份算。
#[allow(
    clippy::ptr_arg,
    reason = "`allocation_records_under_root` 收 `&dyn PoolReader`，切片没有大小不能转成它，要传 Vec 本身"
)]
pub(super) fn referenced_slots_read_from_disk(
    devices: &Vec<(DeviceIdentity, SparseBlockDevice)>,
    root: &RootRecord,
    device: DeviceIdentity,
) -> Result<BTreeSet<u64>, String> {
    let records =
        allocation_records_under_root(devices, root).map_err(|failure| format!("{failure:?}"))?;
    if records.is_empty() {
        let mut slots = BTreeSet::new();
        for (locations, span) in [
            (&root.instance_table.locations, INSTANCE_TABLE_SPAN_SLOTS),
            (
                &root.tree_table.locations,
                TransactionUnit::TreeTable.span_slots(),
            ),
        ] {
            for location in locations
                .iter()
                .filter(|location| location.device == device)
            {
                slots.extend(location.slot.0..location.slot.0 + span);
            }
        }
        return Ok(slots);
    }
    Ok(records
        .iter()
        .filter(|record| record.device == device && !record.is_released)
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect())
}

/// 回收门槛 `max(F_生效, 环里最旧有效根)`，从镜像的根环读（D16（发布语义） 已定项 1「可再分配」「环里最旧有效根」两行）。
/// 一条有效根都没有时按 0 算（Hh 里不出现）。
fn reclaim_threshold_read_from_the_ring_image(
    effective_floor: u64,
    valid_root_txgs: &BTreeSet<u64>,
) -> u64 {
    valid_root_txgs
        .iter()
        .min()
        .copied()
        .unwrap_or(0)
        .max(effective_floor)
}

/// h_缺：[环里最旧有效根.txg, 最新持久根.txg] 里环里没有自证合法根的 txg 个数（原登记「一」第 2 行字面）。
fn missing_txg_count(
    oldest_valid_root_txg: u64,
    newest_root_txg: u64,
    root_txgs: &BTreeSet<u64>,
) -> u64 {
    let missing = (oldest_valid_root_txg..=newest_root_txg)
        .filter(|candidate| !root_txgs.contains(candidate))
        .count();
    u64::try_from(missing).expect("落在 u64 内")
}

/// h_洞（r2 R2 第二读法）：注入过的洞里，落在 [环里最旧有效根, 最新根] 之间、环里没有它的根、且它的根环槽还没被更新的根盖掉的个数。
fn live_hole_count(
    injected_holes: &[u64],
    oldest_valid_root_txg: u64,
    newest_root_txg: u64,
    root_txgs: &BTreeSet<u64>,
    slot_occupant_txg: impl Fn(u64) -> Option<u64>,
) -> u64 {
    let live = injected_holes
        .iter()
        .copied()
        .filter(|hole| {
            let slot_not_overwritten_by_a_newer_root =
                slot_occupant_txg(*hole).is_none_or(|occupant| occupant < *hole);
            oldest_valid_root_txg <= *hole
                && *hole <= newest_root_txg
                && !root_txgs.contains(hole)
                && slot_not_overwritten_by_a_newer_root
        })
        .count();
    u64::try_from(live).expect("落在 u64 内")
}

/// G12 的谓词（原登记 5.3 第一张表）：已释放的记录只要环里有一条算数的根的 txg 落在它的寿命 [分配代, 释放代) 里就扣住。
fn lifetime_interval_rule_withholds(
    allocation_generation: u64,
    release_generation: u64,
    counted_root_txgs: &BTreeSet<u64>,
) -> bool {
    counted_root_txgs
        .iter()
        .any(|root_txg| *root_txg >= allocation_generation && *root_txg < release_generation)
}

pub(super) fn expanded_slots<'records>(
    records: impl Iterator<Item = &'records AllocationRecord>,
) -> BTreeSet<u64> {
    records
        .flat_map(|record| record.slot.0..record.slot.0 + u64::from(record.span_slots))
        .collect()
}

fn size_of_difference(first: &BTreeSet<u64>, second: &BTreeSet<u64>) -> u64 {
    u64::try_from(first.difference(second).count()).expect("落在 u64 内")
}

/// 一块盘上两条臂、两个回收时点的「不发」与多扣（Q1a、Q1b 与 Q1a⁺、Q1b⁺；登记第六节 6.2，r2 Q1a / Q1b 字面）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DeviceOverWithheld {
    pub(super) floor_rule_real: u64,
    pub(super) floor_rule_every_publish: u64,
    pub(super) interval_rule_real: u64,
    pub(super) interval_rule_every_publish: u64,
    pub(super) floor_rule_real_referenced_but_free: u64,
    pub(super) floor_rule_every_publish_referenced_but_free: u64,
    pub(super) interval_rule_referenced_but_free: u64,
    pub(super) no_reclaim_floor_rule_real: u64,
    pub(super) no_reclaim_floor_rule_every_publish: u64,
    pub(super) no_reclaim_interval_rule_real: u64,
    pub(super) no_reclaim_interval_rule_every_publish: u64,
    pub(super) reclaim_rule_violations: u64,
    pub(super) released_records_without_allocation_generation: u64,
}

struct DeviceOverWithheldInputs<'context> {
    pub(super) allocator: &'context PoolAllocator,
    pub(super) device: DeviceIdentity,
    pub(super) referenced: &'context BTreeSet<u64>,
    pub(super) ring_threshold: u64,
    pub(super) every_publish_floor: u64,
    interval_rule_root_txgs: &'context BTreeSet<u64>,
    pub(super) allocation_generations: &'context HashMap<u64, u64>,
}

fn over_withheld_on_device(inputs: &DeviceOverWithheldInputs<'_>) -> DeviceOverWithheld {
    let free_map = inputs
        .allocator
        .devices
        .iter()
        .find(|map| map.device == inputs.device)
        .expect("这块盘在池里");
    let records: Vec<&AllocationRecord> = inputs
        .allocator
        .records()
        .iter()
        .filter(|record| record.device == inputs.device)
        .collect();
    let still_allocated =
        expanded_slots(records.iter().copied().filter(|record| !record.is_released));
    let released_not_reclaimed = expanded_slots(
        records
            .iter()
            .copied()
            .filter(|record| record.is_released && !free_map.is_free(record.slot)),
    );
    let every_publish_withheld_released =
        expanded_slots(records.iter().copied().filter(|record| {
            record.is_released && record.generation.0 > inputs.every_publish_floor
        }));
    let allocation_generation_of =
        |record: &AllocationRecord| inputs.allocation_generations.get(&record.slot.0).copied();
    let interval_rule_withheld_released =
        expanded_slots(records.iter().copied().filter(|record| {
            record.is_released
                && lifetime_interval_rule_withholds(
                    allocation_generation_of(record).unwrap_or(0),
                    record.generation.0,
                    inputs.interval_rule_root_txgs,
                )
        }));
    let every_released =
        expanded_slots(records.iter().copied().filter(|record| record.is_released));
    let union = |extra: &BTreeSet<u64>| -> BTreeSet<u64> {
        still_allocated.union(extra).copied().collect()
    };
    let floor_rule_real_withheld = union(&released_not_reclaimed);
    let floor_rule_every_publish_withheld = union(&every_publish_withheld_released);
    let interval_rule_withheld = union(&interval_rule_withheld_released);
    let no_reclaim_withheld = union(&every_released);
    let reclaim_rule_violations = records
        .iter()
        .filter(|record| record.is_released)
        .filter(|record| {
            let reclaimed = free_map.is_free(record.slot);
            (reclaimed && record.generation.0 > inputs.ring_threshold)
                || (!reclaimed && record.generation.0 <= inputs.ring_threshold)
        })
        .count();
    let released_records_without_allocation_generation = records
        .iter()
        .filter(|record| record.is_released && allocation_generation_of(record).is_none())
        .count();
    DeviceOverWithheld {
        floor_rule_real: size_of_difference(&floor_rule_real_withheld, inputs.referenced),
        floor_rule_every_publish: size_of_difference(
            &floor_rule_every_publish_withheld,
            inputs.referenced,
        ),
        interval_rule_real: size_of_difference(&interval_rule_withheld, inputs.referenced),
        interval_rule_every_publish: size_of_difference(&interval_rule_withheld, inputs.referenced),
        floor_rule_real_referenced_but_free: size_of_difference(
            inputs.referenced,
            &floor_rule_real_withheld,
        ),
        floor_rule_every_publish_referenced_but_free: size_of_difference(
            inputs.referenced,
            &floor_rule_every_publish_withheld,
        ),
        interval_rule_referenced_but_free: size_of_difference(
            inputs.referenced,
            &interval_rule_withheld,
        ),
        no_reclaim_floor_rule_real: size_of_difference(&no_reclaim_withheld, inputs.referenced),
        no_reclaim_floor_rule_every_publish: size_of_difference(
            &no_reclaim_withheld,
            inputs.referenced,
        ),
        no_reclaim_interval_rule_real: size_of_difference(&no_reclaim_withheld, inputs.referenced),
        no_reclaim_interval_rule_every_publish: size_of_difference(
            &no_reclaim_withheld,
            inputs.referenced,
        ),
        reclaim_rule_violations: u64::try_from(reclaim_rule_violations).expect("落在 u64 内"),
        released_records_without_allocation_generation: u64::try_from(
            released_records_without_allocation_generation,
        )
        .expect("落在 u64 内"),
    }
}

/// 一个观测点上从镜像与分配器读到的全部量（登记第五节 5.3「逐发布一行的列」）。
#[derive(Clone, Debug)]
pub(super) struct HhStateReading {
    pub(super) newest_root_txg: u64,
    pub(super) missing_txgs: u64,
    pub(super) live_holes: u64,
    pub(super) ring_threshold: u64,
    pub(super) every_publish_floor: u64,
    pub(super) effective_floor: u64,
    pub(super) abandoned_roots: u64,
    pub(super) roots_without_version_facts: u64,
    pub(super) device_zero: DeviceOverWithheld,
    pub(super) devices_equal: bool,
    pub(super) allocated_slots: u64,
    pub(super) deferred_slots: u64,
    pub(super) isolated_slots: u64,
    pub(super) not_free_beyond_allocated: u64,
    pub(super) root_ring_installed: bool,
    /// W18 (c)(d)：到这个观测点还钉着的全零槽数。
    pub(super) all_zero_slots_still_pinned: u64,
    /// B 让门槛比只按 A（读得出的有效根）取更低（第五节 E10 的 `threshold_lowered_points`）。
    pub(super) threshold_lowered_by_all_zero_pins: bool,
    /// W18 (a)：这个观测点看到这个进程写掉了的钉住槽。
    pub(super) all_zero_pins_released_by_write: Vec<RootRingSlot>,
    /// W18 (b)：这个观测点看到读出了一条不是这个进程写的根的钉住槽（这一族推的是 0，不为 0 照 S18 停）。
    pub(super) all_zero_pins_released_by_reread_only: Vec<RootRingSlot>,
    /// W18 (d)：还钉着而读坏的槽数（这一族不造读故障，不为 0 照 S17 停）。
    pub(super) bad_slots_still_pinned: u64,
}

pub(super) struct HhStateInputs<'context> {
    pub(super) parameters: &'context MakeFilesystemParameters,
    pub(super) devices: &'context [(DeviceIdentity, SparseBlockDevice)],
    pub(super) allocator: &'context PoolAllocator,
    pub(super) versions: &'context BTreeMap<u64, PublishedVersionFacts>,
    pub(super) allocation_generations: &'context HashMap<u64, u64>,
    pub(super) injected_holes: &'context [u64],
    /// 第 6 次重跑登记 E7：这次挂载钉着的槽与它可能住过的最旧 txg；mkfs 那个进程为空。
    pub(super) all_zero_pins: &'context BTreeMap<RootRingSlot, u64>,
    /// 这次挂载新实例的第一个 txg；mkfs 那个进程为 `None`。
    pub(super) first_txg_of_this_mount: Option<u64>,
    /// PC-同槽：「每」列的旁路门槛去掉第一个洞的环槽里那条旧根再算。
    pub(super) treat_the_hole_ring_slot_as_rewritten: bool,
}

pub(super) fn read_hh_state(inputs: &HhStateInputs<'_>) -> HhStateReading {
    let geometry = &inputs.parameters.geometry;
    let region_devices = &inputs.parameters.region_devices;
    let filesystem_identifier = &inputs.parameters.filesystem_identifier;
    let ring = readable_roots_with_ring_slots(
        inputs.devices,
        region_devices,
        geometry,
        filesystem_identifier,
    );
    let ring_root_txgs: BTreeSet<u64> =
        ring.iter().map(|(_, root)| root.checkpoint_txg.0).collect();
    let newest_root = ring
        .iter()
        .map(|(_, root)| *root)
        .max_by_key(|root| (root.checkpoint_txg, root.instance))
        .expect("环里至少有一条根");
    let newest_root_table = instance_table_chain_of_root(inputs.devices, &newest_root)
        .ok()
        .map(|chain| chain.records);
    let abandoned_root_txgs: BTreeSet<u64> = ring
        .iter()
        .filter(|(_, root)| {
            newest_root_table
                .as_ref()
                .is_some_and(|table| root_is_abandoned_by_the_instance_table(root, table))
        })
        .map(|(_, root)| root.checkpoint_txg.0)
        .collect();
    let valid_root_txgs: BTreeSet<u64> = ring_root_txgs
        .difference(&abandoned_root_txgs)
        .copied()
        .collect();
    let effective_floor = effective_rollback_floor(
        inputs.devices,
        region_devices,
        geometry,
        filesystem_identifier,
    )
    .0;
    let newest_root_txg = newest_root.checkpoint_txg.0;
    let slots_per_region = geometry.root_ring_slots_per_region;
    let occupant_by_ring_slot: BTreeMap<RootRingSlot, u64> = ring
        .iter()
        .map(|(ring_slot, root)| (*ring_slot, root.checkpoint_txg.0))
        .collect();
    // W18：钉着的每一槽读这一刻那一槽里自证合法的根；「写掉」「重读读出」两句都不成立的仍钉着。
    let mut all_zero_slots_still_pinned: BTreeSet<u64> = BTreeSet::new();
    let mut all_zero_slots_still_pinned_count = 0u64;
    let mut all_zero_pins_released_by_write = Vec::new();
    let mut all_zero_pins_released_by_reread_only = Vec::new();
    let mut bad_slots_still_pinned = 0u64;
    if let Some(first_txg) = inputs.first_txg_of_this_mount {
        for (ring_slot, oldest_txg_it_can_have_held) in inputs.all_zero_pins {
            let occupant = occupant_by_ring_slot.get(ring_slot).copied();
            let released_by_write = occupant.is_some_and(|occupant_txg| occupant_txg >= first_txg);
            let released_by_reread = occupant.is_some_and(|occupant_txg| occupant_txg < first_txg);
            if released_by_write {
                all_zero_pins_released_by_write.push(*ring_slot);
            } else if released_by_reread {
                all_zero_pins_released_by_reread_only.push(*ring_slot);
            } else {
                all_zero_slots_still_pinned.insert(*oldest_txg_it_can_have_held);
                all_zero_slots_still_pinned_count += 1;
                if read_root_ring_slot_content(inputs.devices, inputs.parameters, *ring_slot)
                    != RootRingSlotContent::AllZero
                {
                    bad_slots_still_pinned += 1;
                }
            }
        }
    }
    let ring_threshold = reclaim_threshold_read_from_the_ring_image(
        effective_floor,
        &valid_root_txgs
            .union(&all_zero_slots_still_pinned)
            .copied()
            .collect(),
    );
    let threshold_lowered_by_all_zero_pins = ring_threshold
        < reclaim_threshold_read_from_the_ring_image(effective_floor, &valid_root_txgs);
    let oldest_valid_root_txg = valid_root_txgs.iter().min().copied().unwrap_or(0);
    let missing_txgs = missing_txg_count(oldest_valid_root_txg, newest_root_txg, &ring_root_txgs);
    let live_holes = live_hole_count(
        inputs.injected_holes,
        oldest_valid_root_txg,
        newest_root_txg,
        &ring_root_txgs,
        |hole| {
            occupant_by_ring_slot
                .get(&target_for_publish(CheckpointTxg(hole), slots_per_region))
                .copied()
        },
    );
    let every_publish_floor = ring_threshold;
    let every_publish_floor_read = if inputs.treat_the_hole_ring_slot_as_rewritten {
        let old_root_txg_in_the_hole_slot = inputs.injected_holes.first().and_then(|first_hole| {
            occupant_by_ring_slot
                .get(&target_for_publish(
                    CheckpointTxg(*first_hole),
                    slots_per_region,
                ))
                .copied()
                .filter(|occupant| occupant < first_hole)
        });
        let mut roots_still_counted = valid_root_txgs_without_the_old_root_in_the_slot(
            &valid_root_txgs,
            old_root_txg_in_the_hole_slot,
        );
        roots_still_counted.extend(all_zero_slots_still_pinned.iter().copied());
        reclaim_threshold_read_from_the_ring_image(effective_floor, &roots_still_counted)
    } else {
        every_publish_floor
    };
    let interval_rule_root_txgs: BTreeSet<u64> = ring_root_txgs
        .iter()
        .copied()
        .filter(|txg| abandoned_root_txgs.contains(txg) || *txg >= effective_floor)
        .collect();
    let mut referenced = BTreeSet::new();
    let mut roots_without_version_facts = 0u64;
    for txg in &ring_root_txgs {
        match inputs.versions.get(txg) {
            Some(facts) => referenced.extend(facts.referenced_slots.iter().copied()),
            None => roots_without_version_facts += 1,
        }
    }
    let reading_on = |device: DeviceIdentity| {
        over_withheld_on_device(&DeviceOverWithheldInputs {
            allocator: inputs.allocator,
            device,
            referenced: &referenced,
            ring_threshold,
            every_publish_floor: every_publish_floor_read,
            interval_rule_root_txgs: &interval_rule_root_txgs,
            allocation_generations: inputs.allocation_generations,
        })
    };
    let device_zero = reading_on(DeviceIdentity(0));
    let device_one = reading_on(DeviceIdentity(1));
    let free_map_zero = inputs
        .allocator
        .devices
        .iter()
        .find(|map| map.device == DeviceIdentity(0))
        .expect("盘 0 在池里");
    let not_free_record_slots = expanded_slots(
        inputs
            .allocator
            .records()
            .iter()
            .filter(|record| record.device == DeviceIdentity(0)),
    )
    .into_iter()
    .filter(|slot| !free_map_zero.is_free(SlotNumber(*slot)))
    .count();
    HhStateReading {
        newest_root_txg,
        missing_txgs,
        live_holes,
        ring_threshold,
        every_publish_floor: every_publish_floor_read,
        effective_floor,
        abandoned_roots: u64::try_from(abandoned_root_txgs.len()).expect("落在 u64 内"),
        roots_without_version_facts,
        device_zero,
        devices_equal: device_zero == device_one,
        allocated_slots: free_map_zero.allocated_slots(),
        deferred_slots: free_map_zero.deferred_slots(),
        isolated_slots: free_map_zero.isolated_slots(),
        not_free_beyond_allocated: u64::try_from(not_free_record_slots)
            .expect("落在 u64 内")
            .saturating_sub(free_map_zero.allocated_slots()),
        root_ring_installed: inputs.allocator.root_ring_occupancy().is_some(),
        all_zero_slots_still_pinned: all_zero_slots_still_pinned_count,
        threshold_lowered_by_all_zero_pins,
        all_zero_pins_released_by_write,
        all_zero_pins_released_by_reread_only,
        bad_slots_still_pinned,
    }
}

#[cfg(test)]
mod tests;

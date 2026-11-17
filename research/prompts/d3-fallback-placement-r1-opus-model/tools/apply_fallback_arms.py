#!/usr/bin/env python3
"""d3-fallback-placement-r1 攻方腿模型：往一棵仓副本的 singlefs-core 里打回落政策的几条臂（只在副本上，不进仓）。
用法：python3 apply_fallback_arms.py <副本根>。每处替换要求旧串命中次数与写明的一样，对不上退 3，一个文件都不写。
臂的选择、偏好集、计数都放线程局部：测试在同一个线程上换臂，不改 PoolAllocator 的字段与构造。"""
import pathlib
import sys

ARM_MODULE = r"""

/// d3-fallback-placement-r1 攻方腿模型：提交内生块回落政策的臂。只在副本上。
pub mod fallback_arm_model {
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeSet;

    use singlefs_format::{ALLOCATION_RECORD_TREE_LEAF_SLOTS, CLUSTER_SEGMENT_SLOTS};

    use super::{CommitGeneratedDeviceAnswer, DeviceFreeMap, UnitFootprint};
    use crate::address::{DeviceIdentity, SlotNumber};
    use crate::allocation_record_tree::AllocationRecordTreeNode;
    use crate::logged_operations::CompactionRegion;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum FallbackArm {
        /// 今天的实现：每块盘各答最低空槽。
        Today,
        /// 每块盘各答：先本盘重写集叶里最低的空槽，没有再本盘最低空槽。各盘答案不同就照今天的路拒。
        PreferThenLowestOnEachDevice,
        /// 池级：先每块盘都在重写集里的叶（按叶升序）里每块盘都空的最低槽，没有再每块盘都空的最低槽。
        PreferThenLowest,
        /// 池级：先重写集的叶，没有再去每块盘都空的槽最多的叶（一样多取低的）。
        PreferThenRichestLeaf,
        /// 池级：先重写集的叶，没有再去空槽多于 2 的最低的叶（一片叶重写要两个落点，多于 2 才净赚），都没有再最低空槽。
        PreferThenLowestLeafWithMoreThanTwoFree,
        /// 池级：同 `PreferThenLowest`，但两步都先绕开按记录全空的段（段内没有未释放记录），绕不开再取。
        PreferThenLowestSparingRecordEmptySegments,
    }

    impl FallbackArm {
        pub const ALL: [FallbackArm; 6] = [
            FallbackArm::Today,
            FallbackArm::PreferThenLowestOnEachDevice,
            FallbackArm::PreferThenLowest,
            FallbackArm::PreferThenRichestLeaf,
            FallbackArm::PreferThenLowestLeafWithMoreThanTwoFree,
            FallbackArm::PreferThenLowestSparingRecordEmptySegments,
        ];

        #[must_use]
        pub const fn name(self) -> &'static str {
            match self {
                FallbackArm::Today => "today",
                FallbackArm::PreferThenLowestOnEachDevice => "prefer_then_lowest_on_each_device",
                FallbackArm::PreferThenLowest => "prefer_then_lowest",
                FallbackArm::PreferThenRichestLeaf => "prefer_then_richest_leaf",
                FallbackArm::PreferThenLowestLeafWithMoreThanTwoFree => {
                    "prefer_then_lowest_leaf_with_more_than_two_free"
                }
                FallbackArm::PreferThenLowestSparingRecordEmptySegments => {
                    "prefer_then_lowest_sparing_record_empty_segments"
                }
            }
        }
    }

    /// 走到定下或轮数用完的固定点的累计。
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct SettlingTally {
        pub settled: u64,
        pub most_rounds: usize,
        pub rounds_sum: u64,
        pub roles_sum: u64,
        pub last_roles: usize,
        pub did_not_settle: u64,
    }

    thread_local! {
        static ARM: Cell<FallbackArm> = const { Cell::new(FallbackArm::Today) };
        static PREFERRED: RefCell<BTreeSet<(DeviceIdentity, u64)>> = const { RefCell::new(BTreeSet::new()) };
        /// 这一串取落点里已经扫空的叶（键的盘号是 None 时指每块盘都空的那一判）：一串里只取不放，扫空的不再扫，省的是模型的挂钟，不改答案。
        static LEAVES_WITHOUT_A_FREE_SLOT: RefCell<BTreeSet<(Option<DeviceIdentity>, u64)>> = const { RefCell::new(BTreeSet::new()) };
        static TALLY: Cell<SettlingTally> = const {
            Cell::new(SettlingTally {
                settled: 0,
                most_rounds: 0,
                rounds_sum: 0,
                roles_sum: 0,
                last_roles: 0,
                did_not_settle: 0,
            })
        };
    }

    pub fn set_the_arm(arm: FallbackArm) {
        ARM.with(|cell| cell.set(arm));
    }

    #[must_use]
    pub fn the_tally() -> SettlingTally {
        TALLY.with(Cell::get)
    }

    pub fn reset_the_tally() {
        TALLY.with(|cell| cell.set(SettlingTally::default()));
    }

    pub fn note_settled(rounds: usize, roles: usize) {
        TALLY.with(|cell| {
            let mut tally = cell.get();
            tally.settled += 1;
            tally.most_rounds = tally.most_rounds.max(rounds);
            tally.rounds_sum += u64::try_from(rounds).expect("轮数");
            tally.roles_sum += u64::try_from(roles).expect("角色数");
            tally.last_roles = roles;
            cell.set(tally);
        });
    }

    pub fn note_did_not_settle() {
        TALLY.with(|cell| {
            let mut tally = cell.get();
            tally.did_not_settle += 1;
            cell.set(tally);
        });
    }

    /// 这一轮（或真发）的重写集：只取叶。根与内部节点不罩槽。
    pub fn prefer_the_nodes(nodes: impl Iterator<Item = AllocationRecordTreeNode>) {
        let leaves: BTreeSet<(DeviceIdentity, u64)> = nodes
            .filter_map(|node| match node {
                AllocationRecordTreeNode::BelowTheRoot(position) if position.level == 0 => {
                    Some((position.device, position.index_in_device))
                }
                AllocationRecordTreeNode::BelowTheRoot(_) | AllocationRecordTreeNode::Root => None,
            })
            .collect();
        PREFERRED.with(|cell| *cell.borrow_mut() = leaves);
        LEAVES_WITHOUT_A_FREE_SLOT.with(|cell| cell.borrow_mut().clear());
    }

    fn aligned_up(slot: u64, alignment: u64) -> u64 {
        slot.div_ceil(alignment) * alignment
    }

    fn span_is_open_on_one_device(
        device: &DeviceFreeMap,
        candidate: u64,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
    ) -> bool {
        (candidate..candidate + footprint.slots()).all(|slot| {
            slot < device.unit_area_end_slot()
                && !device.is_blocked_for_commit_generated(SlotNumber(slot))
                && !excluded_region.is_some_and(|region| region.contains_slot(SlotNumber(slot)))
        })
    }

    fn span_is_open_on_every_device(
        devices: &[DeviceFreeMap],
        candidate: u64,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
    ) -> bool {
        devices.iter().all(|device| {
            span_is_open_on_one_device(device, candidate, footprint, excluded_region)
        })
    }

    /// 按段号看这一段有没有未释放记录：每块盘都没有才算按记录全空。整段之外（末尾那一截）不算。
    fn segment_is_empty_by_the_records(devices: &[DeviceFreeMap], slot: u64) -> bool {
        devices.iter().all(|device| {
            let offset = slot - device.unit_area_start.slot().0;
            let segment = usize::try_from(offset / CLUSTER_SEGMENT_SLOTS).expect("段号");
            let full_segments =
                usize::try_from(device.unit_area_slots / CLUSTER_SEGMENT_SLOTS).expect("段数");
            segment < full_segments && device.unreleased_per_segment[segment] == 0
        })
    }

    /// 每块盘各答（`FallbackArm::PreferThenLowestOnEachDevice`）；别的臂这里就是今天的函数，池级的臂在 `answers_after_the_pool_level_arm` 里换。
    pub(super) fn fallback_slot_on_one_device(
        device: &DeviceFreeMap,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
    ) -> Option<SlotNumber> {
        if ARM.with(Cell::get) == FallbackArm::PreferThenLowestOnEachDevice {
            let leaves: Vec<u64> = PREFERRED.with(|cell| {
                cell.borrow()
                    .iter()
                    .filter(|(identity, _)| *identity == device.device)
                    .map(|(_, leaf)| *leaf)
                    .collect()
            });
            for leaf in leaves {
                if LEAVES_WITHOUT_A_FREE_SLOT.with(|cell| cell.borrow().contains(&(Some(device.device), leaf))) {
                    continue;
                }
                let first = (leaf * ALLOCATION_RECORD_TREE_LEAF_SLOTS).max(device.unit_area_start.slot().0);
                let past = ((leaf + 1) * ALLOCATION_RECORD_TREE_LEAF_SLOTS).min(device.unit_area_end_slot());
                let mut candidate = aligned_up(first, footprint.alignment_in_slots());
                while candidate + footprint.slots() <= past {
                    if span_is_open_on_one_device(device, candidate, footprint, excluded_region) {
                        return Some(SlotNumber(candidate));
                    }
                    candidate += footprint.alignment_in_slots();
                }
                if footprint == UnitFootprint::OneSlot {
                    LEAVES_WITHOUT_A_FREE_SLOT.with(|cell| cell.borrow_mut().insert((Some(device.device), leaf)));
                }
            }
        }
        device.lowest_commit_generated_fallback_slot_outside_the_region(footprint, excluded_region)
    }

    /// 这一片叶里每块盘都放得下的最低落点。
    fn lowest_open_in_leaf(
        devices: &[DeviceFreeMap],
        leaf: u64,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
        spare_record_empty_segments: bool,
    ) -> Option<u64> {
        let area_start = devices[0].unit_area_start.slot().0;
        let area_end = devices.iter().map(DeviceFreeMap::unit_area_end_slot).min()?;
        let first = (leaf * ALLOCATION_RECORD_TREE_LEAF_SLOTS).max(area_start);
        let past = ((leaf + 1) * ALLOCATION_RECORD_TREE_LEAF_SLOTS).min(area_end);
        let mut candidate = aligned_up(first, footprint.alignment_in_slots());
        while candidate + footprint.slots() <= past {
            if span_is_open_on_every_device(devices, candidate, footprint, excluded_region)
                && !(spare_record_empty_segments
                    && segment_is_empty_by_the_records(devices, candidate))
            {
                return Some(candidate);
            }
            candidate += footprint.alignment_in_slots();
        }
        None
    }

    /// 重写集里每块盘都有的叶（升序）。
    fn leaves_preferred_on_every_device(devices: &[DeviceFreeMap]) -> Vec<u64> {
        PREFERRED.with(|cell| {
            let preferred = cell.borrow();
            let mut leaves: BTreeSet<u64> = preferred.iter().map(|(_, leaf)| *leaf).collect();
            leaves.retain(|leaf| {
                devices
                    .iter()
                    .all(|device| preferred.contains(&(device.device, *leaf)))
            });
            leaves.into_iter().collect()
        })
    }

    /// 每片叶每块盘都空的槽数（叶号升序，只数单元区内、两块盘都有的那一段）。
    fn free_slots_per_leaf_on_every_device(
        devices: &[DeviceFreeMap],
        excluded_region: Option<CompactionRegion>,
    ) -> Vec<(u64, u64)> {
        let area_start = devices[0].unit_area_start.slot().0;
        let area_end = devices
            .iter()
            .map(DeviceFreeMap::unit_area_end_slot)
            .min()
            .unwrap_or(area_start);
        let mut counts: std::collections::BTreeMap<u64, u64> = std::collections::BTreeMap::new();
        for slot in area_start..area_end {
            if span_is_open_on_every_device(devices, slot, UnitFootprint::OneSlot, excluded_region) {
                *counts.entry(slot / ALLOCATION_RECORD_TREE_LEAF_SLOTS).or_insert(0) += 1;
            }
        }
        counts.into_iter().collect()
    }

    fn pool_level_slot(
        arm: FallbackArm,
        devices: &[DeviceFreeMap],
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
    ) -> Option<u64> {
        let spare = arm == FallbackArm::PreferThenLowestSparingRecordEmptySegments;
        for leaf in leaves_preferred_on_every_device(devices) {
            if LEAVES_WITHOUT_A_FREE_SLOT.with(|cell| cell.borrow().contains(&(None, leaf))) {
                continue;
            }
            if let Some(slot) = lowest_open_in_leaf(devices, leaf, footprint, excluded_region, spare) {
                return Some(slot);
            }
            if !spare && footprint == UnitFootprint::OneSlot {
                LEAVES_WITHOUT_A_FREE_SLOT.with(|cell| cell.borrow_mut().insert((None, leaf)));
            }
        }
        let per_leaf = free_slots_per_leaf_on_every_device(devices, excluded_region);
        let in_leaf = |leaf: u64, spare_now: bool| {
            lowest_open_in_leaf(devices, leaf, footprint, excluded_region, spare_now)
        };
        match arm {
            FallbackArm::PreferThenRichestLeaf => {
                let mut leaves = per_leaf;
                leaves.sort_by_key(|(leaf, free)| (std::cmp::Reverse(*free), *leaf));
                leaves.into_iter().find_map(|(leaf, _)| in_leaf(leaf, false))
            }
            FallbackArm::PreferThenLowestLeafWithMoreThanTwoFree => per_leaf
                .iter()
                .filter(|(_, free)| *free > 2)
                .find_map(|(leaf, _)| in_leaf(*leaf, false))
                .or_else(|| per_leaf.iter().find_map(|(leaf, _)| in_leaf(*leaf, false))),
            FallbackArm::PreferThenLowestSparingRecordEmptySegments => per_leaf
                .iter()
                .find_map(|(leaf, _)| in_leaf(*leaf, true))
                .or_else(|| {
                    leaves_preferred_on_every_device(devices)
                        .into_iter()
                        .find_map(|leaf| in_leaf(leaf, false))
                })
                .or_else(|| per_leaf.iter().find_map(|(leaf, _)| in_leaf(*leaf, false))),
            FallbackArm::Today
            | FallbackArm::PreferThenLowestOnEachDevice
            | FallbackArm::PreferThenLowest => {
                per_leaf.iter().find_map(|(leaf, _)| in_leaf(*leaf, false))
            }
        }
    }

    /// 每块盘都答回落时，池级的臂把答案换成同一个槽；有一块盘答开段或答不出的不动（照今天的路判一致）。
    pub(super) fn answers_after_the_pool_level_arm(
        devices: &[DeviceFreeMap],
        answers: Vec<(DeviceIdentity, Option<CommitGeneratedDeviceAnswer>)>,
        footprint: UnitFootprint,
        excluded_region: Option<CompactionRegion>,
    ) -> Vec<(DeviceIdentity, Option<CommitGeneratedDeviceAnswer>)> {
        let arm = ARM.with(Cell::get);
        if matches!(arm, FallbackArm::Today | FallbackArm::PreferThenLowestOnEachDevice) {
            return answers;
        }
        let every_device_falls_back = answers.iter().all(|(_, answer)| {
            matches!(answer, Some(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(_)))
        });
        if !every_device_falls_back {
            return answers;
        }
        match pool_level_slot(arm, devices, footprint, excluded_region) {
            Some(slot) => answers
                .into_iter()
                .map(|(device, _)| {
                    (
                        device,
                        Some(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(SlotNumber(slot))),
                    )
                })
                .collect(),
            None => answers,
        }
    }
}
"""

ALLOCATOR_REPLACEMENTS = [
    (
        """            None => self
                .lowest_commit_generated_fallback_slot_outside_the_region(
                    footprint,
                    excluded_region,
                )
                .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),
""",
        """            None => fallback_arm_model::fallback_slot_on_one_device(self, footprint, excluded_region)
                .map(CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot),
""",
        1,
    ),
    (
        """        let answer = match agreement_across_devices(&answer_per_device) {
            DeviceAgreement::Agreed(answer) => answer,
            DeviceAgreement::NoAnswerOnAnyDevice => {
                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
""",
        """        let answer_per_device = fallback_arm_model::answers_after_the_pool_level_arm(
            &self.devices,
            answer_per_device,
            footprint,
            excluded_region,
        );
        let answer = match agreement_across_devices(&answer_per_device) {
            DeviceAgreement::Agreed(answer) => answer,
            DeviceAgreement::NoAnswerOnAnyDevice => {
                return Err(PlacementRefusal::NoFreeSlotOnAnyDevice);
""",
        1,
    ),
]
PREFER = "crate::allocator::fallback_arm_model::prefer_the_nodes"
TRANSACTION_REPLACEMENTS = [
    (
        """        for relocation in &relocations_in_effect {
            allocate_relocation_destination(&mut rehearsal, relocation)?;
""",
        f"""        {PREFER}(rewritten_by_the_plan.iter().copied());
        for relocation in &relocations_in_effect {{
            allocate_relocation_destination(&mut rehearsal, relocation)?;
""",
        1,
    ),
    (
        """        for identity in &rewritten_roles {
            allocate_placement_for_role(&mut rehearsal, *identity, txg)?;
""",
        f"""        {PREFER}(rewritten_by_the_plan.iter().copied());
        for identity in &rewritten_roles {{
            allocate_placement_for_role(&mut rehearsal, *identity, txg)?;
""",
        1,
    ),
    (
        """    let mut relocated_copies: Vec<(PublishedUnit, NamedUnit, UnitRelocatedByThisPublish)> =
""",
        f"""    {PREFER}(rewritten.iter().filter_map(|role| allocation_record_tree_node_of_role(*role)));
    let mut relocated_copies: Vec<(PublishedUnit, NamedUnit, UnitRelocatedByThisPublish)> =
""",
        1,
    ),
    (
        """    for identity in &settled.rewritten_roles {
        let placement = allocate_placement_for_role(allocator, *identity, txg)?;
""",
        f"""    {PREFER}(settled.rewritten_roles.iter().filter_map(|role| allocation_record_tree_node_of_role(*role)));
    for identity in &settled.rewritten_roles {{
        let placement = allocate_placement_for_role(allocator, *identity, txg)?;
""",
        1,
    ),
    (
        """    // 这次发布切出来的记录条数在动分配器之前判（代码审阅第 26 条）：点名的就是这次重写的角色（下面 `named_roles`）。
""",
        """    crate::allocator::fallback_arm_model::note_settled(settled.rounds_walked, settled.rewritten_roles.len());
    // 这次发布切出来的记录条数在动分配器之前判（代码审阅第 26 条）：点名的就是这次重写的角色（下面 `named_roles`）。
""",
        1,
    ),
    (
        """    // 这次发布切出来的记录条数（代码审阅第 26 条）：角色定下来就数得出，在准入、读盘核与动分配器之前判。
""",
        """    crate::allocator::fallback_arm_model::note_settled(settled.rounds_walked, settled.resolved.rewritten_roles.len());
    // 这次发布切出来的记录条数（代码审阅第 26 条）：角色定下来就数得出，在准入、读盘核与动分配器之前判。
""",
        1,
    ),
    (
        """    Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle {
        rounds: ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT,
    })
""",
        """    crate::allocator::fallback_arm_model::note_did_not_settle();
    Err(PublishError::AllocationRecordTreeRewriteSetDidNotSettle {
        rounds: ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT,
    })
""",
        2,
    ),
]


def patched(text, replacements, label):
    for old, new, expected in replacements:
        found = text.count(old)
        if found != expected:
            print(f"✗ {label}: 旧串命中 {found} 次，要 {expected} 次：{old[:80]!r}", file=sys.stderr)
            sys.exit(3)
        text = text.replace(old, new)
    return text


def main():
    if len(sys.argv) != 2:
        print("用法：apply_fallback_arms.py <副本根>", file=sys.stderr)
        sys.exit(2)
    root = pathlib.Path(sys.argv[1])
    allocator = root / "crates/singlefs-core/src/allocator.rs"
    transaction = root / "crates/singlefs-core/src/transaction.rs"
    new_allocator = patched(allocator.read_text(encoding="utf-8"), ALLOCATOR_REPLACEMENTS, "allocator.rs") + ARM_MODULE
    new_transaction = patched(transaction.read_text(encoding="utf-8"), TRANSACTION_REPLACEMENTS, "transaction.rs")
    allocator.write_text(new_allocator, encoding="utf-8")
    transaction.write_text(new_transaction, encoding="utf-8")
    print("name=apply_fallback_arms rc=0")


main()

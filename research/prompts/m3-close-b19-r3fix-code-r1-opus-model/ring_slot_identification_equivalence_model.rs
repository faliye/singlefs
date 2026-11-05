// 攻方腿模型（三方轮 m3-close-b19-r3fix-code-r1）：复跑时由 tools/rerun.sh 接在副本的 crates/singlefs-core/src/mount.rs 末尾，只在副本里，不进仓。
// 比的是抬 F 那一遍两处改动：环里最旧有效根里「这一遍在哪几槽读出了根」（按槽认，对照改前按 txg 推槽）、卸载那一格每块盘上最新有效根（按槽认盘，
// 对照改前按 txg 推盘）。按槽那一臂调实现里的函数；按 txg 那一臂照改前的几行原样写（补丁 crates.patch 里删掉的那几行）。
// 枚举的输入：三个区域槽 0 与区域 0 / 1 / 2 槽 1 六个槽，每槽读坏、读回全零、或住着一条合落点公式的根（槽 0 另可住第 0 代根）；
// 挂载时钉着的槽取四个槽的全部子集；实例表六种；E2 重读的读数三种口径；S 取 4 与 8。
// 另跑一臂破掉前提「读出的非第 0 代根都住在公式那一槽」：区域 0 槽 1 另可住一条 txg 4 的根（公式该在区域 1 槽 1），看两种认法分不分开。
#[cfg(test)]
mod ring_slot_identification_equivalence_model {
    use std::collections::{BTreeMap, BTreeSet};
    use std::time::Instant;

    use super::{
        abandoned_by_table, newest_valid_root_txg_on_every_device,
        oldest_txg_a_ring_slot_bad_at_mount_can_have_held,
        ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass,
        roots_on_the_devices_of_their_ring_slots,
    };
    use crate::address::{CheckpointTxg, DeviceIdentity, InstanceGeneration};
    use crate::allocator::{
        oldest_txg_in_the_ring_rereading_slots_bad_at_mount, RereadOfARootRingSlotBadAtMount,
    };
    use crate::instance_table::{InstanceRow, InstanceTableRecords};
    use crate::make_filesystem::GENESIS_CHECKPOINT_TXG;
    use crate::pointer::NodePointer;
    use crate::root_record::{RootRecord, UnmountMarker};
    use crate::root_ring::{
        root_sits_in_the_ring_slot_the_formula_gives, target_for_publish, RootRingSlot,
        RootRingSlotsPerRegion,
    };

    const REGION_DEVICES_OF_THE_FIRST_GEOMETRY: [DeviceIdentity; 3] =
        [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)];
    const BOTH_DEVICES: [DeviceIdentity; 2] = [DeviceIdentity(0), DeviceIdentity(1)];
    /// 卸载那一格的新 F 取得比枚举里每条根都高：当场放开的门槛 = min(回收门槛, 每块盘上最新有效根的最小者) 落在后者上。
    const FLOOR_ABOVE_EVERY_ROOT: u64 = 1_000;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SlotContent {
        Bad,
        AllZero,
        Root { instance: u32, txg: u64 },
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum RereadPolicy {
        /// 重读读到的就是盘上那一槽现有的：根按实例表判有效或被抛弃，读坏仍坏，全零仍全零。
        WhatTheSlotHolds,
        /// 重读一律仍读坏（这一遍读得出、重读那一下读坏）。
        AlwaysStillBad,
        /// 重读读到盘上那一槽现有的，只是读坏的槽重读读回全零。
        BadSlotsReadBackAllZero,
    }

    fn root_of(instance: u32, txg: u64) -> RootRecord {
        RootRecord {
            filesystem_identifier: [0; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(instance),
            checkpoint_txg: CheckpointTxg(txg),
            tree_table: NodePointer::empty_root(),
            tree_identifier_watermark: 0,
            rollback_floor: CheckpointTxg(0),
            instance_table: NodePointer::empty_root(),
            mapping_root: NodePointer::empty_root(),
            allocation_record_tree_root: NodePointer::empty_root(),
            reclaim_threshold: CheckpointTxg(0),
            pinned_ring_slot: crate::root_record::PinnedRingSlot::NO_SLOT_PINNED,
        }
    }

    fn slot_at(region: u64, slot: u64) -> RootRingSlot {
        RootRingSlot { region, slot }
    }

    /// 一个槽在枚举里能装的：读坏、读回全零、第 0 代根（只槽 0）、落点公式落到这一槽的第一个 txg（第 0 代那一格跳过）上的实例 1 与 2 的根；
    /// 槽 0 另加下一圈那个 txg（3S + 区域号）。`misplaced` 给区域 0 槽 1 另加一条 txg 4（公式落在区域 1 槽 1）的根，只给破前提那一臂。
    fn contents_of(ring_slot: RootRingSlot, slots_per_region: u64, misplaced: bool) -> Vec<SlotContent> {
        let mut contents = vec![SlotContent::Bad, SlotContent::AllZero];
        if ring_slot.slot == 0 {
            contents.push(SlotContent::Root { instance: 0, txg: GENESIS_CHECKPOINT_TXG.0 });
        }
        let first_txg = 3 * ring_slot.slot + ring_slot.region;
        let mut txgs = vec![first_txg];
        if ring_slot.slot == 0 {
            txgs.push(first_txg + 3 * slots_per_region);
        }
        for txg in txgs {
            if txg == GENESIS_CHECKPOINT_TXG.0 {
                continue;
            }
            for instance in [1, 2] {
                contents.push(SlotContent::Root { instance, txg });
            }
        }
        if misplaced && ring_slot == slot_at(0, 1) {
            contents.push(SlotContent::Root { instance: 1, txg: 4 });
        }
        contents
    }

    fn instance_tables() -> Vec<InstanceTableRecords> {
        let row = |instance: u32, selected: u64| InstanceRow {
            instance: InstanceGeneration(instance),
            selected_root_txg: CheckpointTxg(selected),
            applied_transaction_high_water: 0,
        };
        vec![
            InstanceTableRecords { rows: Vec::new() },
            InstanceTableRecords { rows: vec![row(1, 0)] },
            InstanceTableRecords { rows: vec![row(1, 3)] },
            InstanceTableRecords { rows: vec![row(2, 0)] },
            InstanceTableRecords { rows: vec![row(2, 4)] },
            InstanceTableRecords { rows: vec![row(0, 0), row(1, 0), row(2, 0)] },
        ]
    }

    /// 两种认法在一种输入上各算出的东西比出来的结局。
    struct Comparison {
        pin_maps_differ: bool,
        oldest_differs: bool,
        newest_raw_differs: bool,
        txg_arm_none_slot_arm_zero: bool,
        txg_arm_zero_slot_arm_none: bool,
        effective_differs: bool,
        genesis_root_read: bool,
    }

    #[derive(Default)]
    struct Tally {
        cases: u64,
        pin_maps_differ: u64,
        oldest_differs: u64,
        newest_raw_differs: u64,
        txg_arm_none_slot_arm_zero: u64,
        txg_arm_zero_slot_arm_none: u64,
        effective_differs: u64,
        differences_without_a_genesis_root_read: u64,
    }

    fn compare(
        contents: &BTreeMap<RootRingSlot, SlotContent>,
        pinned_slots: &BTreeSet<RootRingSlot>,
        table: &InstanceTableRecords,
        policy: RereadPolicy,
        slots_per_region: RootRingSlotsPerRegion,
        every_root_sits_where_the_formula_puts_it: bool,
    ) -> Comparison {
        let roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)> = contents
            .iter()
            .filter_map(|(slot, content)| match content {
                SlotContent::Root { instance, txg } => Some((*slot, root_of(*instance, *txg))),
                SlotContent::Bad | SlotContent::AllZero => None,
            })
            .collect();
        if every_root_sits_where_the_formula_puts_it {
            for (slot, root) in &roots_with_ring_slots {
                assert!(
                    root_sits_in_the_ring_slot_the_formula_gives(root.checkpoint_txg, *slot, slots_per_region),
                    "枚举里的根只取 read_root_ring_slot 交得出的：{slot:?} 上的 txg {:?}",
                    root.checkpoint_txg
                );
            }
        }
        let pinned: BTreeMap<RootRingSlot, CheckpointTxg> = pinned_slots
            .iter()
            .map(|slot| {
                (
                    *slot,
                    oldest_txg_a_ring_slot_bad_at_mount_can_have_held(
                        *slot,
                        CheckpointTxg(3 * slot.slot + slot.region),
                    ),
                )
            })
            .collect();
        let valid_root_txgs: Vec<CheckpointTxg> = roots_with_ring_slots
            .iter()
            .filter(|(_, root)| !abandoned_by_table(root, table))
            .map(|(_, root)| root.checkpoint_txg)
            .collect();
        // 按槽：实现里的函数。
        let pins_by_slot = ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass(
            pinned.clone(),
            &roots_with_ring_slots,
        );
        // 按 txg：补丁删掉的那几行原样（槽数取这个池的 S）。
        let ring_slots_read_by_txg: BTreeSet<RootRingSlot> = roots_with_ring_slots
            .iter()
            .map(|(_, root)| target_for_publish(root.checkpoint_txg, slots_per_region))
            .collect();
        let pins_by_txg: BTreeMap<RootRingSlot, CheckpointTxg> = pinned
            .into_iter()
            .filter(|(slot, _)| !ring_slots_read_by_txg.contains(slot))
            .collect();
        let mut reread = |slot: RootRingSlot| {
            let content = contents.get(&slot).copied().unwrap_or(SlotContent::Bad);
            match (content, policy) {
                (_, RereadPolicy::AlwaysStillBad) => {
                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
                }
                (SlotContent::Root { instance, txg }, _) => {
                    let root = root_of(instance, txg);
                    if abandoned_by_table(&root, table) {
                        RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable
                    } else {
                        RereadOfARootRingSlotBadAtMount::ValidRoot { checkpoint_txg: root.checkpoint_txg }
                    }
                }
                (SlotContent::AllZero, _)
                | (SlotContent::Bad, RereadPolicy::BadSlotsReadBackAllZero) => {
                    RereadOfARootRingSlotBadAtMount::AllZero
                }
                (SlotContent::Bad, RereadPolicy::WhatTheSlotHolds) => {
                    RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified
                }
            }
        };
        let oldest_by_slot =
            oldest_txg_in_the_ring_rereading_slots_bad_at_mount(&valid_root_txgs, pins_by_slot.clone(), &mut reread).0;
        let oldest_by_txg =
            oldest_txg_in_the_ring_rereading_slots_bad_at_mount(&valid_root_txgs, pins_by_txg.clone(), &mut reread).0;
        // 卸载那一格：按槽调实现里的函数；按 txg 照补丁删掉的 `device_of_txg` 那几行。
        let on_devices_by_slot = roots_on_the_devices_of_their_ring_slots(
            &roots_with_ring_slots,
            &REGION_DEVICES_OF_THE_FIRST_GEOMETRY,
        );
        let on_devices_by_txg: Vec<(DeviceIdentity, RootRecord)> = roots_with_ring_slots
            .iter()
            .map(|(_, root)| {
                let target = target_for_publish(root.checkpoint_txg, slots_per_region);
                (
                    REGION_DEVICES_OF_THE_FIRST_GEOMETRY[usize::try_from(target.region).expect("区域号")],
                    *root,
                )
            })
            .collect();
        let newest_by_slot =
            newest_valid_root_txg_on_every_device(&on_devices_by_slot, table, &BOTH_DEVICES);
        let newest_by_txg =
            newest_valid_root_txg_on_every_device(&on_devices_by_txg, table, &BOTH_DEVICES);
        // 当场放开的那一截：释放代 ≤ min(回收门槛, 每块盘上最新有效根)；交回 None 时一个都不当场放开。
        // 释放代至少是 1（释放发生在 mkfs 之后的某次发布里），None 与 Some(0) 放开的是同一截（空的）。
        let released_up_to =
            |newest: Option<CheckpointTxg>| newest.map_or(0, |newest| newest.0.min(FLOOR_ABOVE_EVERY_ROOT));
        Comparison {
            pin_maps_differ: pins_by_slot != pins_by_txg,
            oldest_differs: oldest_by_slot != oldest_by_txg,
            newest_raw_differs: newest_by_slot != newest_by_txg,
            txg_arm_none_slot_arm_zero: newest_by_txg.is_none()
                && newest_by_slot == Some(GENESIS_CHECKPOINT_TXG),
            txg_arm_zero_slot_arm_none: newest_by_slot.is_none()
                && newest_by_txg == Some(GENESIS_CHECKPOINT_TXG),
            effective_differs: released_up_to(newest_by_slot) != released_up_to(newest_by_txg),
            genesis_root_read: roots_with_ring_slots
                .iter()
                .any(|(_, root)| root.checkpoint_txg == GENESIS_CHECKPOINT_TXG),
        }
    }

    /// 六个槽各取一个读数的笛卡儿积 × 四个可钉槽的全部子集 × 六张实例表 × 三种重读口径，逐个比。
    fn run(declared_slots_per_region: u64, misplaced: bool) -> Tally {
        let slots_per_region = RootRingSlotsPerRegion::from_system_configuration_field(
            declared_slots_per_region,
        )
        .expect("S 在格式区间 [4, 16] 里");
        let slots = [
            slot_at(0, 0),
            slot_at(1, 0),
            slot_at(2, 0),
            slot_at(0, 1),
            slot_at(1, 1),
            slot_at(2, 1),
        ];
        let options: Vec<Vec<SlotContent>> = slots
            .iter()
            .map(|slot| contents_of(*slot, declared_slots_per_region, misplaced))
            .collect();
        let pinnable = [slot_at(0, 0), slot_at(1, 0), slot_at(2, 0), slot_at(0, 1)];
        let tables = instance_tables();
        let policies = [
            RereadPolicy::WhatTheSlotHolds,
            RereadPolicy::AlwaysStillBad,
            RereadPolicy::BadSlotsReadBackAllZero,
        ];
        let combinations: usize = options.iter().map(Vec::len).product();
        let mut tally = Tally::default();
        // 迭代上界是读数组合数 × 16 × 6 × 3；跨轮只累加计数；没有提前出口。
        for combination in 0..combinations {
            let mut rest = combination;
            let mut contents = BTreeMap::new();
            for (slot, contents_of_the_slot) in slots.iter().zip(&options) {
                contents.insert(*slot, contents_of_the_slot[rest % contents_of_the_slot.len()]);
                rest /= contents_of_the_slot.len();
            }
            for pin_mask in 0..(1u32 << pinnable.len()) {
                let pinned: BTreeSet<RootRingSlot> = pinnable
                    .iter()
                    .enumerate()
                    .filter(|(bit, _)| pin_mask & (1 << bit) != 0)
                    .map(|(_, slot)| *slot)
                    .collect();
                for table in &tables {
                    for policy in policies {
                        let comparison = compare(
                            &contents,
                            &pinned,
                            table,
                            policy,
                            slots_per_region,
                            !misplaced,
                        );
                        tally.cases += 1;
                        tally.pin_maps_differ += u64::from(comparison.pin_maps_differ);
                        tally.oldest_differs += u64::from(comparison.oldest_differs);
                        tally.newest_raw_differs += u64::from(comparison.newest_raw_differs);
                        tally.txg_arm_none_slot_arm_zero +=
                            u64::from(comparison.txg_arm_none_slot_arm_zero);
                        tally.txg_arm_zero_slot_arm_none +=
                            u64::from(comparison.txg_arm_zero_slot_arm_none);
                        tally.effective_differs += u64::from(comparison.effective_differs);
                        let any_difference = comparison.pin_maps_differ
                            || comparison.oldest_differs
                            || comparison.newest_raw_differs;
                        tally.differences_without_a_genesis_root_read +=
                            u64::from(any_difference && !comparison.genesis_root_read);
                    }
                }
            }
        }
        tally
    }

    fn print_tally(name: &str, declared_slots_per_region: u64, tally: &Tally, started: Instant) {
        println!(
            "MODELRESULT name={name} S={declared_slots_per_region} cases={} pin_maps_differ={} oldest_differs={} \
             unmount_newest_raw_differs={} unmount_txg_arm_none_slot_arm_zero={} unmount_txg_arm_zero_slot_arm_none={} \
             unmount_released_up_to_differs={} differences_without_a_genesis_root_read={} elapsed_seconds={}",
            tally.cases,
            tally.pin_maps_differ,
            tally.oldest_differs,
            tally.newest_raw_differs,
            tally.txg_arm_none_slot_arm_zero,
            tally.txg_arm_zero_slot_arm_none,
            tally.effective_differs,
            tally.differences_without_a_genesis_root_read,
            started.elapsed().as_secs()
        );
    }

    /// 跑前写下的预期（报告「模型」一节同样写着）：前提成立的两臂（S = 4、8）上，环里最旧有效根逐个相同、卸载当场放开的那一截逐个相同、
    /// 两种认法有任何不同的输入里都读出了第 0 代根、卸载那一格的原始读数只在 None 与 Some(0) 之间不同；钉表本身有不同（单测测得到）。
    /// 破前提那一臂（区域 0 槽 1 住着 txg 4 的根）上卸载当场放开的那一截分得开。断言在全部打印之后。
    #[test]
    fn slot_and_txg_identification_agree_on_every_output_unless_a_root_sits_off_its_formula_slot() {
        let started = Instant::now();
        let mut kept = Vec::new();
        for declared_slots_per_region in [4, 8] {
            let tally = run(declared_slots_per_region, false);
            print_tally("premises_hold", declared_slots_per_region, &tally, started);
            kept.push(tally);
        }
        let misplaced = run(8, true);
        print_tally("root_read_off_its_formula_slot", 8, &misplaced, started);
        for tally in &kept {
            assert_eq!(tally.oldest_differs, 0, "环里最旧有效根按槽与按 txg 分开了");
            assert_eq!(tally.effective_differs, 0, "卸载当场放开的那一截按槽与按 txg 分开了");
            assert_eq!(tally.differences_without_a_genesis_root_read, 0, "没读出第 0 代根时两种认法也不同");
            assert_eq!(
                tally.newest_raw_differs,
                tally.txg_arm_none_slot_arm_zero + tally.txg_arm_zero_slot_arm_none,
                "卸载那一格的原始读数在 None 与 Some(0) 之外也不同"
            );
            assert!(tally.pin_maps_differ > 0, "钉表一次都没分开：枚举没造到第 0 代根那几形");
            assert!(tally.newest_raw_differs > 0, "卸载原始读数一次都没分开：枚举没造到第 0 代根那几形");
        }
        assert!(misplaced.effective_differs > 0, "破前提那一臂也分不开：模型的判别力不够");
    }
}

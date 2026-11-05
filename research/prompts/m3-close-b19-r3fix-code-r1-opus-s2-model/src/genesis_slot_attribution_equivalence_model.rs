
/// 三方代码轮 m3-close-b19-r3fix-code-r1 攻方第二样本的模型（rerun.sh 把这一段拼到仓副本 mount.rs 的末尾再编，不进仓）：
/// 抬 F 那一遍三处按读出根的那一槽认盘、认槽（`ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass`、
/// `roots_on_the_devices_of_their_ring_slots`，调的是仓里的函数），与补丁删掉的按 txg 推槽、推盘那两种写法（这里照补丁的减号行重写），
/// 在第 0 代根三槽的每一种读数、四槽的钉与每一种重读结局上逐格比：中间量（钉表、每块盘上最新有效根）与结局量（回收门槛、
/// 卸载当场放开的释放代集合）。环里最旧有效根照仓里的 `oldest_txg_in_the_ring_rereading_slots_bad_at_mount` 算，回收门槛照 `reclaim_floor`。
/// 两条自证：释放代从 0 起、第 0 代根算被抛弃，各自应当比出结局不同（比法分得出才算数）。
#[cfg(test)]
mod genesis_slot_attribution_equivalence_model {
    use std::collections::{BTreeMap, BTreeSet};
    use std::time::Instant;

    use super::{
        abandoned_by_table, newest_valid_root_txg_on_every_device, reclaim_floor,
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
    use crate::root_ring::{target_for_publish, RootRingSlot, RootRingSlotsPerRegion};

    const VALID_INSTANCE: u32 = 1;
    /// 实例表里有一行 (7, 0)：实例 7 的 txg 高于 0 的根都被抛弃。
    const INSTANCE_ABANDONED_ABOVE_GENESIS: u32 = 7;
    /// 区域归属：第一版 0 / 1 / 0，另三种（三块盘、单盘、0 / 1 / 1）不把几何写死。
    const REGION_DEVICE_ASSIGNMENTS: [[u32; 3]; 4] = [[0, 1, 0], [0, 1, 2], [0, 0, 0], [0, 1, 1]];
    /// 池里不住根环区域的那块盘（`all_devices` 带上它的那一半）。
    const DEVICE_WITHOUT_A_REGION: u32 = 9;
    const EFFECTIVE_FLOORS: [u64; 2] = [0, 5];
    const HIGHEST_RELEASE_GENERATION: u64 = 64;
    /// 第 0 代根三槽之外钉着的那一槽：区域 0 的槽 2，钉在它可能住过的最旧 txg 6。
    const PINNED_SLOT_OUTSIDE_THE_GENESIS_SLOTS: RootRingSlot = RootRingSlot { region: 0, slot: 2 };
    const PIN_OF_THE_SLOT_OUTSIDE_THE_GENESIS_SLOTS: u64 = 6;

    /// 抬 F 这一遍在一个区域的槽 0 读到了什么（顶上的那条根与读出的同样算）。
    #[derive(Clone, Copy, Debug)]
    enum GenesisSlotOnThisPass {
        GenesisRootRead,
        LaterValidRootRead,
        LaterAbandonedRootRead,
        NoRootRead,
    }

    const GENESIS_SLOT_STATES: [GenesisSlotOnThisPass; 4] = [
        GenesisSlotOnThisPass::GenesisRootRead,
        GenesisSlotOnThisPass::LaterValidRootRead,
        GenesisSlotOnThisPass::LaterAbandonedRootRead,
        GenesisSlotOnThisPass::NoRootRead,
    ];

    fn slots_per_region() -> RootRingSlotsPerRegion {
        RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM
    }

    /// 区域 `region` 的槽 0 下一圈住的 txg：3S + region（落点公式 (txg mod 3, (txg div 3) mod S)）。
    fn later_txg_of_the_genesis_slot_of(region: u64) -> u64 {
        3 * slots_per_region().count() + region
    }

    fn root_of(instance: u32, checkpoint_txg: u64) -> RootRecord {
        RootRecord {
            filesystem_identifier: [0; 16],
            unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,
            instance: InstanceGeneration(instance),
            checkpoint_txg: CheckpointTxg(checkpoint_txg),
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

    /// 一遍读数：读出（或顶上）的根各带槽、挂载时钉着的槽、每个钉着的槽重读读到什么、区域归属、池里的盘、F_生效。
    struct OnePass {
        roots_with_ring_slots: Vec<(RootRingSlot, RootRecord)>,
        pinned_since_the_mount: BTreeMap<RootRingSlot, CheckpointTxg>,
        rereads: BTreeMap<RootRingSlot, RereadOfARootRingSlotBadAtMount>,
        region_devices: [DeviceIdentity; 3],
        all_devices: Vec<DeviceIdentity>,
        effective_floor: CheckpointTxg,
    }

    /// 补丁删掉的那一份：读出根的槽按 txg 推（第 0 代根三份都推到区域 0 的槽 0）。
    fn pinned_without_a_root_read_by_the_txgs(pass: &OnePass) -> BTreeMap<RootRingSlot, CheckpointTxg> {
        let read_by_the_txgs: BTreeSet<RootRingSlot> = pass
            .roots_with_ring_slots
            .iter()
            .map(|(_, root)| target_for_publish(root.checkpoint_txg, slots_per_region()))
            .collect();
        pass.pinned_since_the_mount
            .iter()
            .filter(|(ring_slot, _)| !read_by_the_txgs.contains(*ring_slot))
            .map(|(ring_slot, pin)| (*ring_slot, *pin))
            .collect()
    }

    /// 补丁删掉的那一份：根归哪块盘按 txg 推（`device_of_txg`）。
    fn roots_on_the_devices_of_their_txgs(pass: &OnePass) -> Vec<(DeviceIdentity, RootRecord)> {
        pass.roots_with_ring_slots
            .iter()
            .map(|(_, root)| {
                let region = target_for_publish(root.checkpoint_txg, slots_per_region()).region;
                (pass.region_devices[usize::try_from(region).expect("区域号小于 3")], *root)
            })
            .collect()
    }

    /// 环里最旧有效根：仓里那一份取法；`genesis_counted_abandoned` 只给自证用（第 0 代根当被抛弃滤掉）。
    fn oldest_valid_root(
        pass: &OnePass,
        bad_at_mount: BTreeMap<RootRingSlot, CheckpointTxg>,
        table: &InstanceTableRecords,
        genesis_counted_abandoned: bool,
    ) -> Option<CheckpointTxg> {
        let valid_root_txgs: Vec<CheckpointTxg> = pass
            .roots_with_ring_slots
            .iter()
            .filter(|(_, root)| !abandoned_by_table(root, table))
            .filter(|(_, root)| {
                !(genesis_counted_abandoned && root.checkpoint_txg == GENESIS_CHECKPOINT_TXG)
            })
            .map(|(_, root)| root.checkpoint_txg)
            .collect();
        let rereads = &pass.rereads;
        oldest_txg_in_the_ring_rereading_slots_bad_at_mount(
            &valid_root_txgs,
            bad_at_mount,
            &mut |ring_slot| *rereads.get(&ring_slot).expect("只重读钉着的槽"),
        )
        .0
    }

    /// 卸载那一格当场放开的释放代集合（释放代从 `lowest_release_generation` 起，到 HIGHEST_RELEASE_GENERATION 为止）：
    /// 每块盘上最新有效根取不到时为空，取到时放开到 min(回收门槛, 它)。
    fn reclaimed_immediately(
        lowest_release_generation: u64,
        floor_of_the_reclaim: CheckpointTxg,
        newest_on_every_device: Option<CheckpointTxg>,
    ) -> Vec<u64> {
        match newest_on_every_device {
            None => Vec::new(),
            Some(newest) => {
                let bound = floor_of_the_reclaim.min(newest).0.min(HIGHEST_RELEASE_GENERATION);
                (lowest_release_generation..=bound).collect()
            }
        }
    }

    #[derive(Default)]
    struct Tally {
        configurations: u64,
        pinned_sets_differ: u64,
        oldest_differs: u64,
        reclaim_floor_differs: u64,
        newest_on_every_device_differs: u64,
        immediate_reclaim_differs: u64,
        immediate_reclaim_differs_with_release_generation_zero: u64,
        oldest_differs_with_genesis_counted_abandoned: u64,
        example_of_a_pinned_difference: Option<String>,
        example_of_a_newest_difference: Option<String>,
    }

    fn described(pass: &OnePass) -> String {
        let roots: Vec<String> = pass
            .roots_with_ring_slots
            .iter()
            .map(|(ring_slot, root)| {
                format!(
                    "({},{})=i{}t{}",
                    ring_slot.region, ring_slot.slot, root.instance.0, root.checkpoint_txg.0
                )
            })
            .collect();
        let pins: Vec<String> = pass
            .pinned_since_the_mount
            .iter()
            .map(|(ring_slot, pin)| {
                format!("({},{})@{}->{:?}", ring_slot.region, ring_slot.slot, pin.0, pass.rereads[ring_slot])
            })
            .collect();
        let region_devices: Vec<u32> = pass.region_devices.iter().map(|device| device.0).collect();
        let all_devices: Vec<u32> = pass.all_devices.iter().map(|device| device.0).collect();
        format!(
            "region_devices={region_devices:?} all_devices={all_devices:?} F={} roots=[{}] pins=[{}]",
            pass.effective_floor.0,
            roots.join(" "),
            pins.join(" ")
        )
    }

    fn judge_one_pass(pass: &OnePass, table: &InstanceTableRecords, tally: &mut Tally) {
        tally.configurations += 1;
        let pinned_by_the_slots = ring_slots_pinned_since_the_mount_without_a_root_read_on_this_pass(
            pass.pinned_since_the_mount.clone(),
            &pass.roots_with_ring_slots,
        );
        let pinned_by_the_txgs = pinned_without_a_root_read_by_the_txgs(pass);
        if pinned_by_the_slots != pinned_by_the_txgs {
            tally.pinned_sets_differ += 1;
            if tally.example_of_a_pinned_difference.is_none() {
                tally.example_of_a_pinned_difference = Some(format!(
                    "{} by_the_slots={pinned_by_the_slots:?} by_the_txgs={pinned_by_the_txgs:?}",
                    described(pass)
                ));
            }
        }
        let oldest_by_the_slots = oldest_valid_root(pass, pinned_by_the_slots.clone(), table, false);
        let oldest_by_the_txgs = oldest_valid_root(pass, pinned_by_the_txgs.clone(), table, false);
        if oldest_by_the_slots != oldest_by_the_txgs {
            tally.oldest_differs += 1;
        }
        let floor_by_the_slots = reclaim_floor(pass.effective_floor, oldest_by_the_slots);
        let floor_by_the_txgs = reclaim_floor(pass.effective_floor, oldest_by_the_txgs);
        if floor_by_the_slots != floor_by_the_txgs {
            tally.reclaim_floor_differs += 1;
        }
        let newest_by_the_slots = newest_valid_root_txg_on_every_device(
            &roots_on_the_devices_of_their_ring_slots(&pass.roots_with_ring_slots, &pass.region_devices),
            table,
            &pass.all_devices,
        );
        let newest_by_the_txgs = newest_valid_root_txg_on_every_device(
            &roots_on_the_devices_of_their_txgs(pass),
            table,
            &pass.all_devices,
        );
        if newest_by_the_slots != newest_by_the_txgs {
            tally.newest_on_every_device_differs += 1;
            if tally.example_of_a_newest_difference.is_none() {
                tally.example_of_a_newest_difference = Some(format!(
                    "{} by_the_slots={newest_by_the_slots:?} by_the_txgs={newest_by_the_txgs:?}",
                    described(pass)
                ));
            }
        }
        if reclaimed_immediately(1, floor_by_the_slots, newest_by_the_slots)
            != reclaimed_immediately(1, floor_by_the_txgs, newest_by_the_txgs)
        {
            tally.immediate_reclaim_differs += 1;
        }
        if reclaimed_immediately(0, floor_by_the_slots, newest_by_the_slots)
            != reclaimed_immediately(0, floor_by_the_txgs, newest_by_the_txgs)
        {
            tally.immediate_reclaim_differs_with_release_generation_zero += 1;
        }
        if oldest_valid_root(pass, pinned_by_the_slots, table, true)
            != oldest_valid_root(pass, pinned_by_the_txgs, table, true)
        {
            tally.oldest_differs_with_genesis_counted_abandoned += 1;
        }
    }

    /// 一个钉着的槽重读可能读到的：第 0 代根三槽是 txg 0 或下一圈的有效根，槽 2 是 txg 6 的有效根，另加被抛弃、全零、仍坏三种。
    fn reread_outcomes_of(ring_slot: RootRingSlot) -> Vec<RereadOfARootRingSlotBadAtMount> {
        let mut outcomes = if ring_slot.slot == 0 {
            vec![
                RereadOfARootRingSlotBadAtMount::ValidRoot {
                    checkpoint_txg: GENESIS_CHECKPOINT_TXG,
                },
                RereadOfARootRingSlotBadAtMount::ValidRoot {
                    checkpoint_txg: CheckpointTxg(later_txg_of_the_genesis_slot_of(ring_slot.region)),
                },
            ]
        } else {
            vec![RereadOfARootRingSlotBadAtMount::ValidRoot {
                checkpoint_txg: CheckpointTxg(PIN_OF_THE_SLOT_OUTSIDE_THE_GENESIS_SLOTS),
            }]
        };
        outcomes.extend([
            RereadOfARootRingSlotBadAtMount::AbandonedByTheInstanceTable,
            RereadOfARootRingSlotBadAtMount::AllZero,
            RereadOfARootRingSlotBadAtMount::StillUnreadableOrNotSelfVerified,
        ]);
        outcomes
    }

    /// 这一遍读出（或顶上）的根：三个区域的槽 0 各按 `genesis_slot_states`，`later_root_mask` 的第 r 位开着时区域 r 的槽 1 读出 txg 3 + r 的有效根。
    /// 非第 0 代根都住在落点公式给的槽里（读者核落点公式，C743），第 0 代根只住三个区域的槽 0。
    fn roots_with_ring_slots_of(
        genesis_slot_states: [GenesisSlotOnThisPass; 3],
        later_root_mask: u8,
    ) -> Vec<(RootRingSlot, RootRecord)> {
        let mut roots = Vec::new();
        for (region, state) in (0u64..3).zip(genesis_slot_states) {
            let genesis_slot = RootRingSlot { region, slot: 0 };
            let later_txg = later_txg_of_the_genesis_slot_of(region);
            match state {
                GenesisSlotOnThisPass::GenesisRootRead => {
                    roots.push((genesis_slot, root_of(0, GENESIS_CHECKPOINT_TXG.0)))
                }
                GenesisSlotOnThisPass::LaterValidRootRead => {
                    roots.push((genesis_slot, root_of(VALID_INSTANCE, later_txg)))
                }
                GenesisSlotOnThisPass::LaterAbandonedRootRead => roots.push((
                    genesis_slot,
                    root_of(INSTANCE_ABANDONED_ABOVE_GENESIS, later_txg),
                )),
                GenesisSlotOnThisPass::NoRootRead => {}
            }
        }
        for region in 0u64..3 {
            if later_root_mask & (1 << region) != 0 {
                roots.push((RootRingSlot { region, slot: 1 }, root_of(VALID_INSTANCE, 3 + region)));
            }
        }
        roots
    }

    /// 挂载时钉着的槽：`pin_mask` 的第 r 位（r < 3）钉区域 r 的槽 0（钉值 0，第 0 代根），第 3 位钉区域 0 的槽 2（钉值 6）。
    fn pinned_since_the_mount_of(pin_mask: u8) -> BTreeMap<RootRingSlot, CheckpointTxg> {
        let mut pinned = BTreeMap::new();
        for region in 0u64..3 {
            if pin_mask & (1 << region) != 0 {
                pinned.insert(RootRingSlot { region, slot: 0 }, GENESIS_CHECKPOINT_TXG);
            }
        }
        if pin_mask & (1 << 3) != 0 {
            pinned.insert(
                PINNED_SLOT_OUTSIDE_THE_GENESIS_SLOTS,
                CheckpointTxg(PIN_OF_THE_SLOT_OUTSIDE_THE_GENESIS_SLOTS),
            );
        }
        pinned
    }

    /// 钉着的每一槽各取一种重读结局，全部组合（混合进制逐个数）。
    fn every_reread_assignment(
        pinned: &BTreeMap<RootRingSlot, CheckpointTxg>,
    ) -> Vec<BTreeMap<RootRingSlot, RereadOfARootRingSlotBadAtMount>> {
        let slots: Vec<RootRingSlot> = pinned.keys().copied().collect();
        let domains: Vec<Vec<RereadOfARootRingSlotBadAtMount>> =
            slots.iter().map(|ring_slot| reread_outcomes_of(*ring_slot)).collect();
        let combinations: usize = domains.iter().map(Vec::len).product();
        (0..combinations)
            .map(|mut index| {
                slots
                    .iter()
                    .zip(&domains)
                    .map(|(ring_slot, domain)| {
                        let chosen = domain[index % domain.len()];
                        index /= domain.len();
                        (*ring_slot, chosen)
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn genesis_slot_attribution_equivalence_model() {
        let started = Instant::now();
        let table = InstanceTableRecords {
            rows: vec![InstanceRow {
                instance: InstanceGeneration(INSTANCE_ABANDONED_ABOVE_GENESIS),
                selected_root_txg: GENESIS_CHECKPOINT_TXG,
                applied_transaction_high_water: 0,
            }],
        };
        let mut tally = Tally::default();
        for (assignment_index, assignment) in REGION_DEVICE_ASSIGNMENTS.into_iter().enumerate() {
            let region_devices = assignment.map(DeviceIdentity);
            let devices_of_the_regions: BTreeSet<DeviceIdentity> = region_devices.iter().copied().collect();
            for with_a_device_without_a_region in [false, true] {
                let mut all_devices: Vec<DeviceIdentity> = devices_of_the_regions.iter().copied().collect();
                if with_a_device_without_a_region {
                    all_devices.push(DeviceIdentity(DEVICE_WITHOUT_A_REGION));
                }
                for effective_floor in EFFECTIVE_FLOORS {
                    for state_of_region_zero in GENESIS_SLOT_STATES {
                        for state_of_region_one in GENESIS_SLOT_STATES {
                            for state_of_region_two in GENESIS_SLOT_STATES {
                                for later_root_mask in 0u8..8 {
                                    let roots_with_ring_slots = roots_with_ring_slots_of(
                                        [state_of_region_zero, state_of_region_one, state_of_region_two],
                                        later_root_mask,
                                    );
                                    for pin_mask in 0u8..16 {
                                        let pinned_since_the_mount = pinned_since_the_mount_of(pin_mask);
                                        for rereads in every_reread_assignment(&pinned_since_the_mount) {
                                            let pass = OnePass {
                                                roots_with_ring_slots: roots_with_ring_slots.clone(),
                                                pinned_since_the_mount: pinned_since_the_mount.clone(),
                                                rereads,
                                                region_devices,
                                                all_devices: all_devices.clone(),
                                                effective_floor: CheckpointTxg(effective_floor),
                                            };
                                            judge_one_pass(&pass, &table, &mut tally);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if assignment_index == 0 {
                let elapsed = started.elapsed().as_secs_f64();
                println!(
                    "E1ESTIMATE name=genesis_slot_attribution after_one_assignment_of={} configurations={} seconds={elapsed:.1} estimated_total_seconds={:.1}",
                    REGION_DEVICE_ASSIGNMENTS.len(),
                    tally.configurations,
                    elapsed * REGION_DEVICE_ASSIGNMENTS.len() as f64
                );
            }
        }
        println!(
            "E1RESULT name=genesis_slot_attribution configurations={} pinned_sets_differ={} oldest_differs={} reclaim_floor_differs={} newest_on_every_device_differs={} immediate_reclaim_differs={} seconds={:.1}",
            tally.configurations,
            tally.pinned_sets_differ,
            tally.oldest_differs,
            tally.reclaim_floor_differs,
            tally.newest_on_every_device_differs,
            tally.immediate_reclaim_differs,
            started.elapsed().as_secs_f64()
        );
        println!(
            "E1SELFPROOF name=release_generation_from_zero immediate_reclaim_differs={}",
            tally.immediate_reclaim_differs_with_release_generation_zero
        );
        println!(
            "E1SELFPROOF name=genesis_counted_abandoned oldest_differs={}",
            tally.oldest_differs_with_genesis_counted_abandoned
        );
        println!(
            "E1EXAMPLE name=pinned_difference {}",
            tally.example_of_a_pinned_difference.as_deref().unwrap_or("none")
        );
        println!(
            "E1EXAMPLE name=newest_difference {}",
            tally.example_of_a_newest_difference.as_deref().unwrap_or("none")
        );
        // 判据（跑之前写下）：结局量三格为 0 才支持「两种认法结局相同」；中间量两格大于 0 说明取样走到了两种认法不同的那几格；
        // 两条自证大于 0 说明比法分得出结局不同。结局量任一格大于 0，「造不出」那一句撤回，照那一格的例子去造历史。
        assert_eq!(tally.oldest_differs, 0, "环里最旧有效根两种认法不同");
        assert_eq!(tally.reclaim_floor_differs, 0, "回收门槛两种认法不同");
        assert_eq!(tally.immediate_reclaim_differs, 0, "卸载当场放开的释放代两种认法不同");
        assert!(tally.pinned_sets_differ > 0, "取样没走到钉表两种认法不同的格");
        assert!(tally.newest_on_every_device_differs > 0, "取样没走到每块盘上最新有效根两种认法不同的格");
        assert!(
            tally.immediate_reclaim_differs_with_release_generation_zero > 0,
            "自证：释放代从 0 起时比法应当比出不同"
        );
        assert!(
            tally.oldest_differs_with_genesis_counted_abandoned > 0,
            "自证：第 0 代根算被抛弃时比法应当比出不同"
        );
    }
}

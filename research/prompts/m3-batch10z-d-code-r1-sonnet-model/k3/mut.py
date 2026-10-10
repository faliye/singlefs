#!/usr/bin/env python3
"""K3 自造变异：python3 mut.py <名> [restore]；原文须在文件里恰好命中一次。路径写死在副本 repo 下。"""
import sys, os
R = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-d-code-r1-sonnet/repo/crates/singlefs-core/src/')
M = R + 'mount.rs'
A = R + 'allocator.rs'
CEIL_OLD = """        .into_iter()
        .filter(|ring_slot| {
            !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
        })
        .collect();
    rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount("""
MUT = {
 # 准入算上限那一遍
 'ceil-inverted': (M, CEIL_OLD, CEIL_OLD.replace("!ring_slots_recorded", "ring_slots_recorded")),
 'ceil-all-exempt': (M, CEIL_OLD, CEIL_OLD.replace("!ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)", "false")),
 # 抬 F 重算影子账那一遍的拒集
 'rr-all-exempt': (M, """            .filter(|ring_slot| {
                !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
            })
            .collect();
    let mut roots_with_ring_slots =""", """            .filter(|_ring_slot| false)
            .collect();
    let mut roots_with_ring_slots="""),
 # 新补隔离那一圈
 'loop-span1': (M, "allocator.isolate_abandoned(placement.device, placement.slot, placement.span);\n    }\n    // C 那一份不豁免", "allocator.isolate_abandoned(placement.device, placement.slot, 1);\n    }\n    // C 那一份不豁免"),
 'loop-no-exemption': (M, "        if referenced_by_candidates.contains(&key) || !isolated.insert(key) {\n            continue;\n        }\n        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);\n    }\n    // C 那一份不豁免", "        if !isolated.insert(key) {\n            continue;\n        }\n        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);\n    }\n    // C 那一份不豁免"),
 'loop-first-only': (M, "    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {\n        let key", "    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad.iter().take(1) {\n        let key"),
 'loop-skip-already-isolated-by-slot-only': (M, "    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {\n        let key = (placement.device.0, placement.slot.0);\n        if referenced_by_candidates", "    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {\n        let key = (placement.device.0, placement.slot.0);\n        if placement.span > 1 || referenced_by_candidates"),
 # 交回的落点
 'rr-placements-only-if-no-root-read': (M, "            .filter(|(ring_slot, _)| !ring_slots_read.contains(ring_slot))\n            .flat_map(|(_, recorded_placements)| recorded_placements)", "            .filter(|(ring_slot, _)| !ring_slots_read.contains(ring_slot))\n            .flat_map(|(_, recorded_placements)| recorded_placements.into_iter().take(1))"),
 'rr-placements-never-when-read-error-first-slot': (M, "            .filter(|(ring_slot, _)| !ring_slots_read.contains(ring_slot))\n            .flat_map(|(_, recorded_placements)| recorded_placements)", "            .filter(|(ring_slot, _)| !ring_slots_read.contains(ring_slot))\n            .filter(|(ring_slot, _)| ring_slot.region != 2)\n            .flat_map(|(_, recorded_placements)| recorded_placements)"),
 # H2 借用
 'h2-slot-off-by-one': (A, """        let ring_slot = target_for_publish(checkpoint_txg, self.slots_per_region);
        let is_the_same_root_recorded_valid = match""", """        let ring_slot = target_for_publish(CheckpointTxg(checkpoint_txg.0 + 1), self.slots_per_region);
        let is_the_same_root_recorded_valid = match"""),
 'h2-unrecorded-C-gives-empty': (A, """        self.placements_named_by_unapplied_journal_records
            .get(&ring_slot)
            .cloned()
    }

    /// 根环表里记成被抛弃根的槽""", """        self.placements_named_by_unapplied_journal_records
            .get(&ring_slot)
            .cloned()
            .or_else(|| Some(Vec::new()))
    }

    /// 根环表里记成被抛弃根的槽"""),
 'h2-union-of-all-slots': (A, """        self.placements_named_by_unapplied_journal_records
            .get(&ring_slot)
            .cloned()
    }

    /// 根环表里记成被抛弃根的槽""", """        self.placements_named_by_unapplied_journal_records
            .get(&ring_slot)
            .map(|_| self.placements_named_by_unapplied_journal_records())
    }

    /// 根环表里记成被抛弃根的槽"""),
 'h2-first-half-only': (A, """        self.placements_named_by_unapplied_journal_records
            .get(&ring_slot)
            .cloned()
    }

    /// 根环表里记成被抛弃根的槽""", """        self.placements_named_by_unapplied_journal_records
            .get(&ring_slot)
            .map(|placements| placements[..placements.len() / 2].to_vec())
    }

    /// 根环表里记成被抛弃根的槽"""),
 'h2-drop-valid-reread-class': (A, """            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread {
                checkpoint_txg: recorded_checkpoint_txg,
            } => *recorded_checkpoint_txg == checkpoint_txg,
            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::ValidRoot { .. },""", """            => *recorded_checkpoint_txg == checkpoint_txg,
            RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => false,
            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::ValidRoot { .. },"""),
 # 记成被抛弃根的槽集合
 'h1set-includes-valid': (A, """                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::BadAtMount(_)
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
            })
            .collect()""", """                RootRingOccupant::ValidRoot { .. } => Some((*ring_slot, Vec::new())),
                RootRingOccupant::BadAtMount(_)
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
            })
            .collect()"""),
 'h1set-includes-badatmount': (A, """                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::BadAtMount(_)
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
            })
            .collect()""", """                RootRingOccupant::BadAtMount(_) => Some((*ring_slot, Vec::new())),
                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
            })
            .collect()"""),
 'h1set-includes-reread-valid': (A, """                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::BadAtMount(_)
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
            })
            .collect()""", """                RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => Some((*ring_slot, Vec::new())),
                RootRingOccupant::ValidRoot { .. }
                | RootRingOccupant::BadAtMount(_)
                | RootRingOccupant::AllZeroAtMount { .. }
                | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses { .. } => None,
            })
            .collect()"""),
 # raise_the_floor_through 里只对 abandoned_by_table 的根借 C：改成对所有根借
 'raise-borrow-for-all-roots': (M, """            roots
                .iter()
                .filter(|root| abandoned_by_table(root, &table))
                .filter_map(|root| {
                    occupancy
                        .placements_recorded_for_the_abandoned_root(""", """            roots
                .iter()
                .filter(|_root| true)
                .filter_map(|root| {
                    occupancy
                        .placements_recorded_for_the_abandoned_root("""),
 'h2-allow-badatmount': (A, """            RootRingOccupant::AbandonedRoot { .. }
            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
                ..
            }
            | RootRingOccupant::BadAtMount(_)
            | RootRingOccupant::AllZeroAtMount { .. } => false,
        };
        if !is_the_same_root_recorded_valid {""", """            RootRingOccupant::AbandonedRoot { .. }
            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
                ..
            }
            | RootRingOccupant::AllZeroAtMount { .. } => false,
            RootRingOccupant::BadAtMount(_) => true,
        };
        if !is_the_same_root_recorded_valid {"""),
 'h2-allow-abandoned-occupant': (A, """            RootRingOccupant::AbandonedRoot { .. }
            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
                ..
            }
            | RootRingOccupant::BadAtMount(_)
            | RootRingOccupant::AllZeroAtMount { .. } => false,
        };
        if !is_the_same_root_recorded_valid {""", """            RootRingOccupant::AbandonedRoot { .. } => true,
            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::AbandonedRoot { .. },
                ..
            }
            | RootRingOccupant::BadAtMount(_)
            | RootRingOccupant::AllZeroAtMount { .. } => false,
        };
        if !is_the_same_root_recorded_valid {"""),
 'h2-onepass-compare-txg-only': (A, """                (
                    root_record_self_verified_on_one_rebuild_pass.instance,
                    root_record_self_verified_on_one_rebuild_pass.checkpoint_txg,
                ) == (instance, checkpoint_txg)
            }
            RootRingOccupant::AbandonedRoot { .. }""", """                root_record_self_verified_on_one_rebuild_pass.checkpoint_txg == checkpoint_txg
            }
            RootRingOccupant::AbandonedRoot { .. }"""),
 'h2-valid-compare-ignore-txg': (A, """            } => *recorded_checkpoint_txg == checkpoint_txg,
            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::ValidRoot { .. },""", """            } => *recorded_checkpoint_txg >= CheckpointTxg(0),
            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {
                root: RootReadOnOneRebuildPass::ValidRoot { .. },"""),
 'revert-ceil-filter': (M, CEIL_OLD, CEIL_OLD.replace("        .filter(|ring_slot| {\n            !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)\n        })\n","")),
 'revert-rr-filter': (M, """            .filter(|ring_slot| {
                !ring_slots_recorded_as_holding_an_abandoned_root.contains_key(ring_slot)
            })
            .collect();
    let mut roots_with_ring_slots =""", """            .collect();
    let mut roots_with_ring_slots ="""),
 'revert-new-loop': (M, "    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad {\n        let key", "    for placement in placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad.iter().take(0) {\n        let key"),
 'revert-h2-or-else': (M, """                        .or_else(|| {
                            occupancy
                                .placements_named_by_unapplied_journal_records_for_the_root_recorded_valid_on_its_ring_slot(
                                    root.instance,
                                    root.checkpoint_txg,
                                )
                        })
""", ""),
}
name = sys.argv[1]
restore = len(sys.argv) > 2 and sys.argv[2] == 'restore'
path, old, new = MUT[name]
s = open(path).read()
if restore:
    assert s.count(new) == 1, ('restore', name, s.count(new))
    s = s.replace(new, old)
else:
    assert s.count(old) == 1, ('apply', name, s.count(old))
    s = s.replace(old, new)
open(path, 'w').write(s)
print(('restored ' if restore else 'applied ') + name)

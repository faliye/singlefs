import sys, json, os
R = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-b-code-r1-sonnet/repo-k3/crates/singlefs-core/src/')
A = R + 'allocator.rs'
M = R + 'mount.rs'
MUTS = {
 'm1-recorded-empty': (M,
  "                    placements\n                        .iter()\n                        .map(|placement| (placement.device, placement.slot, placement.span))\n                        .collect()",
  "                    placements\n                        .iter()\n                        .map(|placement| (placement.device, placement.slot, placement.span))\n                        .take(0)\n                        .collect()"),
 'm2-recorded-first-only': (M,
  "                    placements\n                        .iter()\n                        .map(|placement| (placement.device, placement.slot, placement.span))\n                        .collect()",
  "                    placements\n                        .iter()\n                        .map(|placement| (placement.device, placement.slot, placement.span))\n                        .take(1)\n                        .collect()"),
 'm3-constructor-instance': (M,
  "                    instance: root.instance,\n                    checkpoint_txg: root.checkpoint_txg,\n                    referenced_placements,",
  "                    instance: InstanceGeneration(0),\n                    checkpoint_txg: root.checkpoint_txg,\n                    referenced_placements,"),
 'm4-constructor-txg': (M,
  "                    instance: root.instance,\n                    checkpoint_txg: root.checkpoint_txg,\n                    referenced_placements,",
  "                    instance: root.instance,\n                    checkpoint_txg: CheckpointTxg(root.checkpoint_txg.0 + 1),\n                    referenced_placements,"),
 'm5-badatmount-some-empty': (A,
  "            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,",
  "            RootRingOccupant::BadAtMount(_) => Some(Vec::new()),\n            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,"),
 'm6-allzero-some-empty': (A,
  "            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,",
  "            RootRingOccupant::AllZeroAtMount { .. } => Some(Vec::new()),\n            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,"),
 'm7-validroot-some-empty': (A,
  "            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,",
  "            RootRingOccupant::ValidRoot { .. } => Some(Vec::new()),\n            RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,"),
 'm8-onepass-valid-some-empty': (A,
  "            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,",
  "            RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => Some(Vec::new()),\n            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => None,"),
 'm9-abandoned-compare-txg-only': (A,
  "            } => ((*recorded_instance, *recorded_checkpoint_txg) == (instance, checkpoint_txg))",
  "            } => ((*recorded_checkpoint_txg) == (checkpoint_txg) && (*recorded_instance == *recorded_instance))"),
 'm10-abandoned-compare-instance-only': (A,
  "            } => ((*recorded_instance, *recorded_checkpoint_txg) == (instance, checkpoint_txg))",
  "            } => ((*recorded_instance) == (instance) && (*recorded_checkpoint_txg == *recorded_checkpoint_txg))"),
 'm11-onepass-compare-txg-only': (A,
  "            ) == (instance, checkpoint_txg))\n                .then(|| referenced_placements.clone()),\n            RootRingOccupant::ValidRoot { .. }",
  "            ).1 == checkpoint_txg)\n                .then(|| referenced_placements.clone()),\n            RootRingOccupant::ValidRoot { .. }"),
 'm12-onepass-compare-instance-only': (A,
  "            ) == (instance, checkpoint_txg))\n                .then(|| referenced_placements.clone()),\n            RootRingOccupant::ValidRoot { .. }",
  "            ).0 == instance)\n                .then(|| referenced_placements.clone()),\n            RootRingOccupant::ValidRoot { .. }"),
 'm13-accessor-slot-off-by-one': (A,
  "        let ring_slot = target_for_publish(checkpoint_txg, self.slots_per_region);\n        match self.occupants.get(&ring_slot)? {",
  "        let ring_slot = target_for_publish(CheckpointTxg(checkpoint_txg.0 + 1), self.slots_per_region);\n        match self.occupants.get(&ring_slot)? {"),
 'm14-key-txg-zero': (M,
  "                        .map(|recorded| ((root.instance, root.checkpoint_txg), recorded))",
  "                        .map(|recorded| ((root.instance, CheckpointTxg(0)), recorded))"),
 'm15-b-credential-always': (M,
  "    if the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {\n        return Ok(Vec::new());\n    }\n    Err(MountError::NewerStateStillUnreadableAfterOneReread(",
  "    if true || the_current_account_records_later_units_over_the_account_slots_of(root, current_records) {\n        return Ok(Vec::new());\n    }\n    Err(MountError::NewerStateStillUnreadableAfterOneReread("),
 'm16-count-recorded-as-unreadable': (M,
  "        let placements = match placements_of_abandoned_roots\n            .recorded_in_the_root_ring_table(root)\n            .or_else(|| placements_referenced_by_root(devices, unit_area_start, root))\n        {",
  "        if placements_of_abandoned_roots.recorded_in_the_root_ring_table(root).is_some() {\n            unreadable += 1;\n        }\n        let placements = match placements_of_abandoned_roots\n            .recorded_in_the_root_ring_table(root)\n            .or_else(|| placements_referenced_by_root(devices, unit_area_start, root))\n        {"),
 'm17-rebuilt-occupant-empty-placements': (M,
  "                ) => RootRingOccupant::AbandonedRoot {\n                    instance: root.instance,\n                    checkpoint_txg: root.checkpoint_txg,\n                    referenced_placements,\n                },",
  "                ) => RootRingOccupant::AbandonedRoot {\n                    instance: root.instance,\n                    checkpoint_txg: root.checkpoint_txg,\n                    referenced_placements: { let _ = referenced_placements; Vec::new() },\n                },"),
 'm18-raise-filter-removed': (M,
  "                .filter(|root| abandoned_by_table(root, &table))\n                .filter_map(|root| {\n                    occupancy",
  "                .filter_map(|root| {\n                    occupancy"),
 'm20-rereadvalid-some-empty': (A,
  "            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,",
  "            RootRingOccupant::ValidRootReadWhenTheSlotBadAtMountWasReread { .. } => Some(Vec::new()),\n            RootRingOccupant::ValidRoot { .. }\n            | RootRingOccupant::BadAtMount(_)\n            | RootRingOccupant::AllZeroAtMount { .. }\n            | RootRingOccupant::RootSelfVerifiedOnOnlyOneOfTheTwoRebuildPasses {\n                root: RootReadOnOneRebuildPass::ValidRoot { .. },\n                ..\n            } => None,"),
 'm21-onepass-abandoned-arm-none': (A,
  "            } => ((\n                root_record_self_verified_on_one_rebuild_pass.instance,\n                root_record_self_verified_on_one_rebuild_pass.checkpoint_txg,\n            ) == (instance, checkpoint_txg))\n                .then(|| referenced_placements.clone()),",
  "            } => ((\n                root_record_self_verified_on_one_rebuild_pass.instance,\n                root_record_self_verified_on_one_rebuild_pass.checkpoint_txg,\n            ) == (instance, checkpoint_txg))\n                .then(|| { let _ = referenced_placements; return Vec::new(); }),"),
 'm19-mapping-fallback-off-in-raise': [(M,
  "    if let Some(computed) =\n        placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)\n    {\n        return Ok(computed);\n    }\n    before_the_one_reread.before_rereading();",
  "    if let Some(computed) =\n        placements_referenced_by_root_computed_from_the_mapping(devices, unit_area_start, root)\n    {\n        if !IN_RAISE.with(|f| f.get()) {\n            return Ok(computed);\n        }\n    }\n    before_the_one_reread.before_rereading();"),
  (M, "enum PlacementsOfAbandonedRoots<'recorded> {", "thread_local! { static IN_RAISE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }\nenum PlacementsOfAbandonedRoots<'recorded> {"),
  (M, "    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {\n        match isolate_slots_referenced_only_by_abandoned_roots(", "    IN_RAISE.with(|f| f.set(true));\n    let abandoned_roots_unreadable = if shadow_ledger == ShadowLedger::On {\n        match isolate_slots_referenced_only_by_abandoned_roots("),
  (M, "            &mut BeforeTheOneReread::RereadImmediately,\n        ) {\n            Ok(computed) => computed.abandoned_roots_unreadable,\n            Err(error) => {\n                *allocator = allocator_before_the_raise;", "            &mut BeforeTheOneReread::RereadImmediately,\n        ) {\n            Ok(computed) => { IN_RAISE.with(|f| f.set(false)); computed.abandoned_roots_unreadable }\n            Err(error) => {\n                IN_RAISE.with(|f| f.set(false));\n                *allocator = allocator_before_the_raise;"),
 ],
}
MUTS['m22-prebatch-raise-reads-disk-and-mapping-off-in-raise'] = MUTS['m19-mapping-fallback-off-in-raise'] + [(M,
  "            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(\n                &placements_recorded_for_abandoned_roots,\n            ),\n            &mut BeforeTheOneReread::RereadImmediately,",
  "            PlacementsOfAbandonedRoots::ReadFromTheirAccounts,\n            &mut BeforeTheOneReread::RereadImmediately,")]
MUTS['m23-prebatch-raise-reads-disk'] = [(M,
  "            PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(\n                &placements_recorded_for_abandoned_roots,\n            ),\n            &mut BeforeTheOneReread::RereadImmediately,",
  "            PlacementsOfAbandonedRoots::ReadFromTheirAccounts,\n            &mut BeforeTheOneReread::RereadImmediately,")]
MUTS['m24-lr3-raise-no-restore'] = [(M,
  "            Ok(computed) => computed.abandoned_roots_unreadable,\n            Err(error) => {\n                *allocator = allocator_before_the_raise;\n                return Err(error);\n            }",
  "            Ok(computed) => computed.abandoned_roots_unreadable,\n            Err(error) => {\n                return Err(error);\n            }")]

def apply(name, restore=False):
    edits = MUTS[name]
    if isinstance(edits, tuple):
        edits = [edits]
    if restore:
        edits = list(reversed(edits))
    for f, old, new in edits:
        s = open(f).read()
        if restore:
            assert s.count(new) == 1, ('restore', name, s.count(new))
            open(f, 'w').write(s.replace(new, old))
        else:
            assert s.count(old) == 1, ('apply', name, s.count(old))
            open(f, 'w').write(s.replace(old, new))
if __name__ == '__main__':
    apply(sys.argv[1], restore=(len(sys.argv) > 2 and sys.argv[2] == 'restore'))
    print('ok', sys.argv[1:])

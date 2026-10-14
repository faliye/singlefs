"""Edit a copy of mount.rs: K2 instrumentation, B3 differential instrumentation, K1 direct tests.
Usage: python3 instrument.py <mount.rs> <k1-tests.rs>   (edits in place; only for scratch copies)"""
import sys
path, tests_path = sys.argv[1], sys.argv[2]
src = open(path, encoding="utf-8").read()

def once(old, new):
    global src
    count = src.count(old)
    if count != 1:
        sys.exit(f"anchor hit {count} times: {old[:80]!r}")
    src = src.replace(old, new)

# K2: log the isolated units every rollback computes.
once("    every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;\n",
     "    eprintln!(\n"
     "        \"OPUS-K2 rollback target={:?} resurrected_units={} isolated_units={:?} current_units={}\",\n"
     "        (target.instance.0, target.checkpoint_txg.0),\n"
     "        units_to_check_on_every_device.len(),\n"
     "        isolated_units_to_check_on_every_device\n"
     "            .iter()\n"
     "            .map(|(unit, locations)| (format!(\"{unit:?}\"), locations[0].slot.0, locations[1].slot.0))\n"
     "            .collect::<Vec<_>>(),\n"
     "        current.units.len(),\n"
     "    );\n"
     "    every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;\n")

# B3 differential: collect what the five places see, recompute the start-slot logic, print the difference.
once("        referenced_by_the_current_account.clone();\n",
     "        referenced_by_the_current_account.clone();\n"
     "    let mut opus_candidate_placements: Vec<(u32, u64, u64)> = Vec::new();\n")
once("        if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) {\n            referenced_by_candidates.extend(",
     "        if let Some(placements) = placements_referenced_by_root(devices, unit_area_start, root) {\n"
     "            opus_candidate_placements.extend(placements.iter().map(|(d, s, sp)| (d.0, s.0, *sp)));\n"
     "            referenced_by_candidates.extend(")
once("    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();\n",
     "    let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();\n"
     "    let mut opus_abandoned: Vec<(Vec<(u32, u64, u64)>, bool)> = Vec::new();\n"
     "    let mut opus_new_calls: BTreeSet<(u32, u64, bool)> = BTreeSet::new();\n")
once("        for (device, slot, span_slots) in placements.iter().copied() {\n",
     "        opus_abandoned.push((\n"
     "            placements.iter().map(|(d, s, sp)| (d.0, s.0, *sp)).collect(),\n"
     "            !reclaim_isolates_what_this_root_references,\n"
     "        ));\n"
     "        for (device, slot, span_slots) in placements.iter().copied() {\n")
once("            ) {\n                if reclaim_isolates_what_this_root_references {\n",
     "            ) {\n"
     "                opus_new_calls.insert((device.0, slot_to_isolate.0, !reclaim_isolates_what_this_root_references));\n"
     "                if reclaim_isolates_what_this_root_references {\n")
once("        ) {\n            allocator\n                .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n                    placement.device,",
     "        ) {\n"
     "            opus_new_calls.insert((placement.device.0, slot_to_isolate.0, false));\n"
     "            allocator\n                .isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n                    placement.device,")
once("        ) {\n            allocator\n                .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(",
     "        ) {\n"
     "            opus_new_calls.insert((placement.device.0, slot_to_isolate.0, false));\n"
     "            allocator\n                .isolate_the_slots_not_allocated_of_a_placement_named_by_unapplied_journal_records(")
once("    Ok(ShadowLedgerComputed {\n        abandoned_roots_unreadable: unreadable,\n",
     "    {\n"
     "        fn opus_start_slot_pass(\n"
     "            placements: &[(u32, u64, u64)],\n"
     "            exempt: &BTreeSet<(u32, u64)>,\n"
     "            strong: bool,\n"
     "            keys: &mut BTreeSet<(u32, u64)>,\n"
     "            calls: &mut BTreeSet<(u32, u64, bool)>,\n"
     "        ) {\n"
     "            for (device, slot, span) in placements {\n"
     "                let key = (*device, *slot);\n"
     "                if exempt.contains(&key) || !keys.insert(key) {\n"
     "                    continue;\n"
     "                }\n"
     "                for covered in *slot..*slot + *span {\n"
     "                    calls.insert((*device, covered, strong));\n"
     "                }\n"
     "            }\n"
     "        }\n"
     "        let old_current: BTreeSet<(u32, u64)> = current_records\n"
     "            .iter()\n"
     "            .filter(|record| !record.is_released)\n"
     "            .map(|record| (record.device.0, record.slot.0))\n"
     "            .collect();\n"
     "        let mut old_candidates = old_current.clone();\n"
     "        old_candidates.extend(opus_candidate_placements.iter().map(|(d, s, _)| (*d, *s)));\n"
     "        let mut old_keys: BTreeSet<(u32, u64)> = BTreeSet::new();\n"
     "        let mut old_calls: BTreeSet<(u32, u64, bool)> = BTreeSet::new();\n"
     "        for (placements, strong) in &opus_abandoned {\n"
     "            opus_start_slot_pass(placements, &old_candidates, *strong, &mut old_keys, &mut old_calls);\n"
     "        }\n"
     "        let ring_bad: Vec<(u32, u64, u64)> = placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad\n"
     "            .iter()\n"
     "            .map(|p| (p.device.0, p.slot.0, p.span))\n"
     "            .collect();\n"
     "        opus_start_slot_pass(&ring_bad, &old_candidates, false, &mut old_keys, &mut old_calls);\n"
     "        let named: Vec<(u32, u64, u64)> = placements_named_by_unapplied_journal_records\n"
     "            .iter()\n"
     "            .map(|p| (p.device.0, p.slot.0, p.span))\n"
     "            .collect();\n"
     "        opus_start_slot_pass(&named, &old_current, false, &mut old_keys, &mut old_calls);\n"
     "        let new_only: Vec<&(u32, u64, bool)> = opus_new_calls.difference(&old_calls).collect();\n"
     "        let old_only: Vec<&(u32, u64, bool)> = old_calls.difference(&opus_new_calls).collect();\n"
     "        let spans: BTreeSet<u64> = opus_abandoned.iter().flat_map(|(p, _)| p.iter().map(|x| x.2)).chain(named.iter().map(|x| x.2)).collect();\n"
     "        eprintln!(\n"
     "            \"OPUS-B3 abandoned_roots={} strong_roots={} named={} ring_bad={} spans={:?} calls_new={} calls_old={} new_only={:?} old_only={:?}\",\n"
     "            opus_abandoned.len(),\n"
     "            opus_abandoned.iter().filter(|(_, strong)| *strong).count(),\n"
     "            named.len(),\n"
     "            ring_bad.len(),\n"
     "            spans,\n"
     "            opus_new_calls.len(),\n"
     "            old_calls.len(),\n"
     "            new_only,\n"
     "            old_only,\n"
     "        );\n"
     "    }\n"
     "    Ok(ShadowLedgerComputed {\n        abandoned_roots_unreadable: unreadable,\n")

# K1 direct tests: append into the existing test module (before the file's last closing brace).
tests = open(tests_path, encoding="utf-8").read()
assert src.endswith("    }\n}\n"), "file tail changed"
src = src[: -len("}\n")] + tests + "}\n"
open(path, "w", encoding="utf-8").write(src)
print("edited", path)

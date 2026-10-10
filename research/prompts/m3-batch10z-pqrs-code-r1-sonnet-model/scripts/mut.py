#!/usr/bin/env python3
"""mut.py <名字...>  在 tree-m 上套一个变异、编、跑入库用例文件、还原。名字见 MUTANTS。"""
import os, subprocess, sys, shutil, re, time
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-sonnet')
T = W + '/tree-m'
S = T + '/crates/singlefs-core/src/'
R = '/home/fy5090/code/singlefs/research/scripts'
MS, AL, MT = 'mounted_session.rs', 'allocator.rs', 'mount.rs'
M = {}
def mut(name, f, old, new): M[name] = (f, old, new)
mut('maxmin', MS, '        .max()?;', '        .min()?;')
mut('noIsoCheck', MS, '.all(|(_, isolated)| *isolated == 0)', '.all(|(_, isolated)| *isolated == 1_000_000_000)')
mut('budgetAsStuck', MS, '        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}\n        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }\n        | FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {',
    '        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } | FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => {}\n        FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {')
mut('spaceAsStuck', MS, '        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}\n        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. }\n        | FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {',
    '        FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } | FloorRaiseStop::FloorRaiseRefusedForSpace { .. } => {}\n        FloorRaiseStop::PublishesPerAdmissionWouldBeExceeded { .. } => {')
mut('noRehearsedCarry', MS, '                rehearsed = next;', '                let _ = next;')
mut('failNoCurrent', MS, '                    *current = PoolVersion::of_a_version_of_trees(last_published);\n                    return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(',
    '                    return Err(RootRingPushRefused::PublishFailedAfterTheRehearsal(')
mut('slotsWrittenEmpty', MS, '            ring_slots_written: target.ring_slots,', '            ring_slots_written: Vec::new(),')
mut('textCountZero', MS, '                self.empty_publishes_until_the_ring_slot_of_r_is_written,\n', '                0,\n')
mut('textRegionSlot', MS, '                self.ring_slot_of_r.region,', '                self.ring_slot_of_r.slot,')
mut('ringLenMinus', MS, '        (1..=ring_length_in_txgs)', '        (1..ring_length_in_txgs)')
mut('skipDevCheck', MS, '        let parameters_of_the_pool = parameters_of_the_pool_agreeing_with_the_caller_inputs(\n            parameters_and_device_table.parameters_on_disk(),\n            devices,\n            &start,\n        )\n        .map_err(|refusal| RootRingPushRefused::DevicesHandedIn(Box::new(refusal)))?;',
    '        let parameters_of_the_pool = parameters_and_device_table.parameters_on_disk().clone();')
mut('isoAllSlots', AL, '.filter(|slot| device_map.is_isolated(SlotNumber(*slot)))', '.filter(|slot| { let _ = device_map.is_isolated(SlotNumber(*slot)); true })')
mut('emptyHeld', AL, '            .filter(|(_, placements)| !placements.is_empty())\n            .map(|(ring_slot, placements)| (*ring_slot, placements.clone()))', '            .map(|(ring_slot, placements)| (*ring_slot, placements.clone()))')
mut('diskReadWholeIso', MT, '                Some(_),\n            ) => true,', '                Some(_),\n            ) => false,')
mut('notRecordedAsNotAllocated', MT, '                None,\n            ) => false,\n        };', '                None,\n            ) => true,\n        };')
mut('badSlotLoopOld', MT, '        allocator.isolate_the_slots_not_allocated_of_a_placement_referenced_by_an_abandoned_root(\n            placement.device,\n            placement.slot,\n            placement.span,\n        );',
    '        allocator.isolate_abandoned(placement.device, placement.slot, placement.span);')
mut('txgOfRNone', AL, '            } => Some(*checkpoint_txg),\n            RootRingOccupant::AbandonedRoot { .. }', '            } => None,\n            RootRingOccupant::AbandonedRoot { .. }')
mut('countsDup', AL, 'let isolated_covered: BTreeSet<u64> = placements', 'let isolated_covered: Vec<u64> = placements')

mut('recordRootNoReisolate', AL, '            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);\n        self.isolate_reclaimed_slots_the_isolation_still_protects(\n            &reclaimed,\n            &still_protected_by_the_isolation,\n        );\n    }', '            self.reclaim_released_records_up_to(reclaim_floor, ReclaimedReuse::Immediately);\n        let _ = (&reclaimed, &still_protected_by_the_isolation);\n    }')
mut('raiseReclaimNoReisolate', AL, '            .unwrap_or_default();\n        self.isolate_reclaimed_slots_the_isolation_still_protects(\n            &reclaimed,\n            &still_protected_by_the_isolation,\n        );\n        reclaimed', '            .unwrap_or_default();\n        let _ = &still_protected_by_the_isolation;\n        reclaimed')
mut('protectOnlyAbandoned', AL, '        protected.extend(self.placements_named_by_unapplied_journal_records());', '        let _ = &mut protected;')
mut('protectOnlyC', AL, '        let mut protected = self.placements_referenced_by_abandoned_roots();', '        let mut protected: Vec<PlacementOnDevice> = Vec::new();')

mut('site1HeldAdmission', MT, '        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => {\n            reclaim_released_in_the_sequence_raising_the_floor_up_to(\n                allocator,\n                shadow_ledger,', '        ReuseOfTheSlotsReclaimedByTheSequence::HeldUntilFloorTakesEffect => {\n            reclaim_released_in_the_sequence_raising_the_floor_up_to(\n                allocator,\n                ShadowLedger::Off,')
mut('site2ImmediatelyUnmount', MT, '                    reclaim_released_in_the_sequence_raising_the_floor_up_to(\n                        allocator,\n                        shadow_ledger,\n                        floor_of_the_reclaim.min(newest_valid_root_on_every_device),', '                    reclaim_released_in_the_sequence_raising_the_floor_up_to(\n                        allocator,\n                        ShadowLedger::Off,\n                        floor_of_the_reclaim.min(newest_valid_root_on_every_device),')
mut('site3HeldUnmount', MT, '            reclaimed.extend(reclaim_released_in_the_sequence_raising_the_floor_up_to(\n                allocator,\n                shadow_ledger,', '            reclaimed.extend(reclaim_released_in_the_sequence_raising_the_floor_up_to(\n                allocator,\n                ShadowLedger::Off,')

def run(name):
    f, old, new = M[name]
    path = S + f
    src = open(path).read()
    if src.count(old) != 1:
        return f'{name}: 旧串命中 {src.count(old)} 次，跳过'
    shutil.copy(path, path + '.orig')
    try:
        open(path, 'w').write(src.replace(old, new, 1))
        env = dict(os.environ, CARGO_TARGET_DIR=W + '/target-m')
        b = subprocess.run(['nice', '-n', '19', 'bash', R + '/run-with-memory-cap.sh', '6G', 'bash', R + '/capped.sh', '8', 'cargo', 'test', '--release', '--offline', '-p', 'singlefs-harness', '--test', 'raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table', '--no-run'], cwd=T, env=env, capture_output=True, text=True)
        if b.returncode != 0:
            err = [l for l in b.stderr.splitlines() if l.startswith('error')][:2]
            return f'{name}: 编不过 {err}'
        exe = [l.split('(')[1].rstrip(')') for l in (b.stderr + b.stdout).splitlines() if 'Executable' in l and 'raise_floor' in l][0]
        r = subprocess.run(['nice', '-n', '19', 'bash', R + '/run-with-memory-cap.sh', '6G', 'bash', R + '/capped.sh', '6', os.path.join(T, exe) if not exe.startswith('/') else exe], cwd=T, capture_output=True, text=True)
        out = r.stdout + r.stderr
        failed = re.findall(r'^test (\S+) \.\.\. FAILED', out, re.M)
        tail = [l for l in out.splitlines() if l.startswith('test result')][-1:] 
        return f'{name}: rc={r.returncode} 红 {len(failed)} 条 {" ".join(x[:70] for x in failed[:3])} | {tail}'
    finally:
        shutil.move(path + '.orig', path)

if __name__ == '__main__':
    names = sys.argv[1:] or list(M)
    for n in names:
        t = time.time()
        print(run(n), f'({time.time()-t:.0f}s)', flush=True)

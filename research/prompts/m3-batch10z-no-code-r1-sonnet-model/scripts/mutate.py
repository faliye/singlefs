#!/usr/bin/env python3
"""mutate.py <名> : 把 tree-core 的文件拷到 tree-mut 再按名套一个变异（草稿目录内，只动 tree-mut）。"""
import os, sys, shutil
W = os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet')
CORE = W + '/tree-core/crates/singlefs-core/src/'
MUT = W + '/tree-mut/crates/singlefs-core/src/'
FILES = ['allocator.rs', 'mount.rs']
M = {}
# 每个变异: (文件, 旧串, 新串)
M['none'] = []
M['m1_raise_plain'] = [('mount.rs', '''        ShadowLedger::On => allocator
            .reclaim_released_up_to_isolating_what_the_isolation_still_protects(floor, reuse),''', '''        ShadowLedger::On => allocator.reclaim_released_up_to(floor, reuse),''')]
M['m2_newmethod_noiso'] = [('allocator.rs', '''        self.isolate_reclaimed_slots_the_isolation_still_protects(
            &reclaimed,
            &still_protected_by_the_isolation,
        );
        reclaimed
            .into_iter()''', '''        let _ = &still_protected_by_the_isolation;
        reclaimed
            .into_iter()''')]
M['m3_record_root_noiso'] = [('allocator.rs', '''        self.isolate_reclaimed_slots_the_isolation_still_protects(
            &reclaimed,
            &still_protected_by_the_isolation,
        );
    }

    /// 同 [`Self::record_root_written_by_this_process`]，记之前先照 E2''', '''        let _ = (&reclaimed, &still_protected_by_the_isolation);
    }

    /// 同 [`Self::record_root_written_by_this_process`]，记之前先照 E2''')]
M['m4_skip_alloc_check_removed'] = [('allocator.rs', '''            if !self.allocated[index] {
                self.isolate_slot_at_index(index);
            }''', '''            self.isolate_slot_at_index(index);''')]
M['m5_reiso_first_slot_only'] = [('allocator.rs', '''            for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
                let is_referenced_by_an_abandoned_root = still_referenced''', '''            for slot in record.slot.0..record.slot.0 + 1 {
                let is_referenced_by_an_abandoned_root = still_referenced''')]
M['m6_protect_abandoned_only'] = [('allocator.rs', '''        let still_protected_by_the_isolation = self
            .root_ring
            .as_ref()
            .map(RootRingOccupancy::placements_the_isolation_still_protects)
            .unwrap_or_default();''', '''        let still_protected_by_the_isolation = self
            .root_ring
            .as_ref()
            .map(RootRingOccupancy::placements_referenced_by_abandoned_roots)
            .unwrap_or_default();''')]
M['m7_immediately_arm_plain'] = [('mount.rs', '''                Some(newest_valid_root_on_every_device) => {
                    reclaim_released_in_the_sequence_raising_the_floor_up_to(
                        allocator,
                        shadow_ledger,
                        floor_of_the_reclaim.min(newest_valid_root_on_every_device),
                        ReclaimedReuse::Immediately,
                    )
                }''', '''                Some(newest_valid_root_on_every_device) => allocator.reclaim_released_up_to(
                    floor_of_the_reclaim.min(newest_valid_root_on_every_device),
                    ReclaimedReuse::Immediately,
                ),''')]
M['m8_extend_arm_plain'] = [('mount.rs', '''            reclaimed.extend(reclaim_released_in_the_sequence_raising_the_floor_up_to(
                allocator,
                shadow_ledger,
                floor_of_the_reclaim,
                ReclaimedReuse::HeldUntilFloorTakesEffect,
            ));''', '''            reclaimed.extend(allocator.reclaim_released_up_to(
                floor_of_the_reclaim,
                ReclaimedReuse::HeldUntilFloorTakesEffect,
            ));''')]
M['m9_clear_before_reclaim_all_c'] = [('allocator.rs', '''        let still_protected_by_the_isolation = occupancy.placements_the_isolation_still_protects();
        match displaced {''', '''        let still_protected_by_the_isolation: Vec<PlacementOnDevice> = Vec::new();
        match displaced {''')]
sys.path.insert(0, W + '/scripts')
import mutants_extra
M.update(mutants_extra.X)
name = sys.argv[1]
for f in FILES:
    shutil.copy(CORE + f, MUT + f)
for part in name.split('+'):
    for (f, old, new) in M[part]:
        s = open(MUT + f).read()
        assert s.count(old) == 1, (part, f, s.count(old))
        open(MUT + f, 'w').write(s.replace(old, new))
print('applied', name)

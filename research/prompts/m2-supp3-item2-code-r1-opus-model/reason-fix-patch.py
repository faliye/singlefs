#!/usr/bin/env python3
"""reason-fix-patch.py <copy>: attack-leg proposed glue change (measured only on this leg's copies, attacked zero rounds):
map RollbackTargetNotACandidate by its `reason` text to exactly one model reason (unknown text -> Unexplained)."""
import sys, pathlib
root = pathlib.Path(sys.argv[1])
p = root / 'crates/singlefs-harness/src/model_comparison.rs'
s = p.read_text()
old = ('        MountError::RollbackTargetNotACandidate { .. } => ObservedRefusalReason::Explained(vec![\n'
       '            ModelRefusalReason::RollbackTargetBelowEffectiveFloor,\n'
       '            ModelRefusalReason::RollbackTargetOnAbandonedTimeline,\n'
       '        ]),\n')
new = ('        MountError::RollbackTargetNotACandidate { reason, .. } => match *reason {\n'
       '            "txg 低于生效的回退下界 F" => explained(ModelRefusalReason::RollbackTargetBelowEffectiveFloor),\n'
       '            "实例表里那个实例的行 T 更小：这是被抛弃时间线的根" => explained(ModelRefusalReason::RollbackTargetOnAbandonedTimeline),\n'
       '            _ => ObservedRefusalReason::Unexplained,\n'
       '        },\n')
assert s.count(old) == 1, s.count(old)
p.write_text(s.replace(old, new)); print('reason fix applied', root)

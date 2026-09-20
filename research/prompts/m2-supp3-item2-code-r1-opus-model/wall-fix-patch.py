#!/usr/bin/env python3
"""wall-fix-patch.py <copy>: attack-leg proposed tightening (measured only on this leg's copies, attacked zero rounds):
a refusal by the allocation-record wall counts as the wall only when the member's own `records` exceeds the model's 812;
otherwise the glue maps it to Unexplained (so the model flags it)."""
import sys, pathlib
root = pathlib.Path(sys.argv[1])
def rep(path, old, new):
    p = root / path; s = p.read_text(); n = s.count(old)
    if n != 1: sys.exit(f'{path}: anchor count {n}')
    p.write_text(s.replace(old, new))
rep('crates/singlefs-harness/src/model.rs', '    fn allocation_record_node_capacity() -> u64 {', '    pub fn allocation_record_node_capacity() -> u64 {')
rep('crates/singlefs-harness/src/model_comparison.rs',
    '        PublishError::AllocationRecordsExceedOneNode { .. } => {\n',
    '        PublishError::AllocationRecordsExceedOneNode { records, .. }\n'
    '            if u64::try_from(*records).expect("条数") <= crate::model::IdealModel::allocation_record_node_capacity() =>\n'
    '        {\n            ObservedRefusalReason::Unexplained\n        }\n'
    '        PublishError::AllocationRecordsExceedOneNode { .. } => {\n')
print('wall fix applied', root)

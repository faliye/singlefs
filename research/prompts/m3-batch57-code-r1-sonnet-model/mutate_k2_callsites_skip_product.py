#!/usr/bin/env python3
# K2 变异：整镜像路径上三个调用点（墓碑 11496 行、谱系 11403 行、退役 11650 行）在没装旋钮（产品支）时不调容量判法
import sys
src, dst = sys.argv[1:3]
t = open(src, encoding='utf-8').read()
a = '        judge_tombstone_container_capacity_of_a_version(\n            &version.name,\n            tombstone_tree.tree,\n            containers,\n            capacity_knob,\n            judgements,\n        );'
assert t.count(a) == 1
t = t.replace(a, '        if capacity_knob.is_some() {\n' + a + '\n        }')
b = '        judge_lineage_or_retirement_container_capacity_of_a_version(\n            &version.name,\n            LineageOrRetirementTree::Retirement,'
assert t.count(b) == 1
i = t.index(b); j = t.index('            judgements,\n        );', i) + len('            judgements,\n        );')
blk = t[i:j]
t = t[:i] + '        if retirement_capacity_knob.is_some() {\n' + blk + '\n        }' + t[j:]
c = '            judge_lineage_or_retirement_container_capacity_of_a_version(\n                &version.name,\n                LineageOrRetirementTree::Lineage,'
assert t.count(c) == 1
i = t.index(c); j = t.index('                judgements,\n            );', i) + len('                judgements,\n            );')
blk = t[i:j]
t = t[:i] + '            if lineage_capacity_knob.is_some() {\n' + blk + '\n            }' + t[j:]
open(dst, 'w', encoding='utf-8').write(t); print('ok')

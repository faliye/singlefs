#!/usr/bin/env python3
# K3 变异：冷走读里把「分配记录逐条判代与跨度」从条目宽、逐条判代与 seq 之前挪到数记账条目的紧前面（仍在两个数之前）
import sys
src, dst = sys.argv[1:3]
t = open(src, encoding='utf-8').read()
gen = '    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;\n'
assert t.count(gen) == 1
t = t.replace(gen, '')
anchor = '    accounting_entry_count_is_three_plus_six_per_device(\n        &roots.accounting.leaf_entries_in_key_order,\n        root.checkpoint_txg,\n        device_count,\n    )?;\n'
assert t.count(anchor) == 1
t = t.replace(anchor, gen + anchor)
open(dst, 'w', encoding='utf-8').write(t); print('ok')

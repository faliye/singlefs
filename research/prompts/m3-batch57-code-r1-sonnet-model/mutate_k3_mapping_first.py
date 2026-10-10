#!/usr/bin/env python3
# K3 变异：冷走读（recovery.rs 里 5395 行那一段）把「数映射条目」整句挪到「分配记录逐条判代与跨度」之前；其余次序不动
import sys
src, dst = sys.argv[1:3]
text = open(src, encoding='utf-8').read()
call_start = text.index('    mapping_entry_count_is_one_per_mapped_unit(\n        roots.mapping.leaf_entries_in_key_order.len(),')
call_end = text.index('    )?;\n', call_start) + len('    )?;\n')
call = text[call_start:call_end]
assert text.count(call) == 1
text = text[:call_start] + text[call_end:]
anchor = '    allocation_record_generations_and_spans_are_judged(allocation_records, root)?;\n    every_accounting_entry_is_as_wide_as_its_field_table(\n        &roots.accounting.leaf_entries_in_key_order,\n    )?;'
assert text.count(anchor) == 1
text = text.replace(anchor, call + anchor)
open(dst, 'w', encoding='utf-8').write(text)
print('ok')

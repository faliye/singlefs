# 从 crates/mutations.tsv 今天的第 32、37、41、42、69、107 行算出改到今天写法的新行，写进草稿目录；不碰仓里的表。
import json, os, sys
root = "/home/fy5090/code/singlefs"
draft = "/tmp/claude-1000/m2-anchor-fix"
table_lines = open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")
def fields_of(line_number):
    fields = table_lines[line_number - 1].split("\t")
    assert len(fields) == 6, line_number
    return fields
def replace_exactly_once(text, old, new, what):
    assert text.count(old) == 1, (what, text.count(old))
    return text.replace(old, new)
new_rows = {}
# 第 32 行：写行那段从取号之后的 for 循环挪进 instance_rows_to_write，区间上界从 instance.0 换成 instance_to_acquire.0。
fields = fields_of(32)
fields[2] = "    (first_row_instance..instance_to_acquire.0)"
fields[3] = "    (first_row_instance..first_row_instance + 1)"
new_rows[32] = fields
# 第 37、42 行：候选集从 for 循环里的 if 换成 filter 闭包里的布尔式。
fields = fields_of(37)
fields[2] = "            *index == newest_index || (!abandoned && !below_floor)"
fields[3] = "            *index == newest_index || !below_floor"
new_rows[37] = fields
fields = fields_of(42)
fields[2] = "            *index == newest_index || (!abandoned && !below_floor)"
fields[3] = "            *index == newest_index || !abandoned"
new_rows[42] = fields
# 第 41 行：改写从 PoolAllocator::record 挪进 make_room_for_record_on_device；变异照旧是「不改写那条已回收记录、另追加一条」。
fields = fields_of(41)
fields[2] = "\\n".join([
    '            existing.span_slots = u16::try_from(placement.span).expect("跨度 2 字节");',
    "            existing.generation = generation;",
    "            existing.is_released = false;",
])
fields[3] = "\\n".join([
    "            records.push(AllocationRecord {",
    "                device,",
    "                slot: placement.slot,",
    '                span_slots: u16::try_from(placement.span).expect("跨度 2 字节"),',
    "                generation,",
    "                is_released: false,",
    "            });",
])
new_rows[41] = fields
# 第 69、107 行：refuse_publishes_before_acquisition_that_do_not_pass_admission 多了 rows_to_write 这个参数。
inserted_old = "        instance_to_acquire,\\n        warm_up_publishes_planned.len(),"
inserted_new = "        instance_to_acquire,\\n        rows_written.len(),\\n        warm_up_publishes_planned.len(),"
fields = fields_of(69)
fields[2] = replace_exactly_once(fields[2], inserted_old, inserted_new, "69 原文")
fields[3] = replace_exactly_once(fields[3], inserted_old, inserted_new, "69 替换文")
new_rows[69] = fields
fields = fields_of(107)
fields[2] = replace_exactly_once(fields[2], inserted_old, inserted_new, "107 原文")
new_rows[107] = fields
with open(os.path.join(draft, "new-rows.json"), "w", encoding="utf-8") as handle:
    json.dump({str(key): value for key, value in new_rows.items()}, handle, ensure_ascii=False, indent=1)
# 候选：第 41 行的另一种写法（调用方在「已改写」那一臂也追加），只用来对照跑，不进表。
alternative = fields_of(41)
alternative[2] = "                RecordAtThePlacementSlot::RewrittenFromReclaimed => {}\\n                RecordAtThePlacementSlot::Absent => {"
alternative[3] = "                RecordAtThePlacementSlot::RewrittenFromReclaimed | RecordAtThePlacementSlot::Absent => {"
with open(os.path.join(draft, "alternative-41.json"), "w", encoding="utf-8") as handle:
    json.dump(alternative, handle, ensure_ascii=False, indent=1)
# 按 59 号的读法（\n 当换行）在今天的源码里数命中
def unescape(text): return text.replace("\\n", "\n")
for line_number, fields in sorted(new_rows.items()):
    source = open(os.path.join(root, fields[1]), encoding="utf-8").read()
    print(f"第 {line_number} 行\t{fields[1]}\t新原文命中 {source.count(unescape(fields[2]))} 次\t替换文命中 {source.count(unescape(fields[3]))} 次")
source = open(os.path.join(root, alternative[1]), encoding="utf-8").read()
print(f"第 41 行候选写法\t{alternative[1]}\t原文命中 {source.count(unescape(alternative[2]))} 次")

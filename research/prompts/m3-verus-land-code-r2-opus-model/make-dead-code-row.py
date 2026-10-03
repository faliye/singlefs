#!/usr/bin/env python3
"""生成 dead-code-row.tsv：把 readable_record_of_the_same_publish_follows 里计数器是 u64 最大值就交回 false 那一支、
next_counter 与「计数器 ≥ 下一号」那一判一起删掉，进 if 时补一句引理调用（证明要「同实例、切片在后 ⇒ 计数器更大」）。
这是攻方的改法（零轮），用来量「删了之后 Verus 照样验得过」；原文从冻结副本里按行取。用法：make-dead-code-row.py <核 lib.rs> <出口>"""
import sys

source_lines = open(sys.argv[1], encoding="utf-8").read().split("\n")
original = "\n".join(source_lines[1068:1090])  # 第 1069–1090 行
assert original.startswith("    let record = &records[index];\n    if record.counter.0 == u64::MAX {"), original[:120]
assert original.endswith("        if later.instance.0 == record.instance.0 && later.counter.0 >= next_counter {"), original[-120:]
replacement = original
for removed in [
    "    if record.counter.0 == u64::MAX {\n        return false;\n    }\n",
    "    let next_counter = record.counter.0 + 1;\n",
    "            next_counter == records@[index as int].counter.0 + 1,\n",
]:
    assert replacement.count(removed) == 1, removed
    replacement = replacement.replace(removed, "")
old_condition = "        if later.instance.0 == record.instance.0 && later.counter.0 >= next_counter {"
new_condition = ("        if later.instance.0 == record.instance.0 {\n"
                 "            proof {\n"
                 "                lemma_slice_order_within_an_instance_follows_the_counter(records@, index as int, later_index as int);\n"
                 "            }")
replacement = replacement.replace(old_condition, new_condition)
with open(sys.argv[2], "x", encoding="utf-8") as output:
    output.write("# 攻方改法（零轮）：删掉 C540 扫描里的死分支与死子式；五段同 attack-rows.tsv。\n")
    output.write("\t".join(["D01 删掉 C540 扫描的「计数器是最大值」那一支、next_counter 与「≥ 下一号」那一判",
                            "crates/singlefs-replay-prefix/src/lib.rs",
                            original.replace("\n", "\\n"), replacement.replace("\n", "\\n"), "推：verified"]) + "\n")

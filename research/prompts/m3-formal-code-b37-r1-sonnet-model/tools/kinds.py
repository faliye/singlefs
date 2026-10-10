#!/usr/bin/env python3
"""B2 模型：nff 单测 publish_writes_line_... 期望串里的种类名与次序，对 write_accounting.rs 的 IN_REPORT_ORDER 与 report_name。
用法：kinds.py <树根>"""
import re, sys
root = sys.argv[1]
wa = open(root + "/crates/singlefs-core/src/write_accounting.rs", encoding="utf-8").read()
order = re.search(r"IN_REPORT_ORDER: \[WrittenStructureKind; (\d+)\] = \[(.*?)\];", wa, re.S)
declared_len = int(order.group(1))
kinds = re.findall(r"WrittenStructureKind::(\w+)", order.group(2))
names = dict(re.findall(r"WrittenStructureKind::(\w+) => \"(\w+)\"", wa))
expected_names = [names[k] for k in kinds]
nff = open(root + "/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs", encoding="utf-8").read()
start = nff.index("fn publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros")
end = nff.index("#[test]", start)
seg = nff[start:end]
got = re.findall(r"(\w+?)_write_calls=\d+ \1_written_bytes=\d+", seg)
print("IN_REPORT_ORDER 声明长度", declared_len, " 实列", len(kinds))
print("期望串里的种类（%d 个）：" % len(got), got)
print("IN_REPORT_ORDER 经 report_name：", expected_names)
print("逐个相同、次序相同：", got == expected_names)

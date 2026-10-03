"""e171-conclusion-r3 攻方：E171 的 281 行（编不过的取补位换上的那一行）里，哪几行的变异名出现在门禁 checker-tier-formal-verification 的 E165 红例钉表第 6 段。
用法：python3 e165_pin_overlap.py <E171 产物> <e165-pinned-red-examples.tsv>
"""
import re
import sys

product, pins = sys.argv[1], sys.argv[2]
rows = {}
for line in open(product, encoding="utf-8"):
    m = re.match(r"E7RESULT name=row row=(\d+) row_name=(\S+) category=([a-z_]+)", line)
    if m:
        rows[int(m.group(1))] = (m.group(2), m.group(3))
    m = re.match(r"E7RESULT name=substitution row=(\d+) .* substituted_by=(\S+) failed_attempts=.* category=([a-z_]+)", line)
    if m:
        rows[int(m.group(1))] = (m.group(2), m.group(3))
pinned = {}
for number, line in enumerate(open(pins, encoding="utf-8"), 1):
    if line.startswith("#") or not line.strip():
        continue
    name = line.split("\t")[5].split("#", 1)[1].replace(" ", "_")
    pinned.setdefault(name, []).append(number)
hits = [(n, c, pinned[name]) for n, (name, c) in sorted(rows.items()) if name in pinned]
for n, c, lines in hits:
    print(f"name=e165_pin_overlap row={n} category={c} pin_table_lines={','.join(map(str, lines))} row_name={rows[n][0]}")
print(f"name=e165_pin_overlap_summary rows={len(hits)} counted={sum(c == 'counted' for _, c, _ in hits)} untouched={sum(c == 'untouched' for _, c, _ in hits)}")

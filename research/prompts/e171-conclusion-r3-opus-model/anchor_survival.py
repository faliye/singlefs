"""e171-conclusion-r3 攻方：E171 的 281 行（6 行编不过的取补位换上的那一行）在一棵树上还打不打得上。
对每一行：在这棵树的 crates/mutations.tsv 里按变异名找行（名字里的 _ 是产物把空格换掉的，换回去比），找到之后原文在文件里恰好命中一次才算「打得上」。
用法：python3 anchor_survival.py <产物> <树根> [<树根> …]
输出：每棵树一行汇总，另外逐行列出 counted 行与「全放行」行在哪棵树上打不上。
"""
import os
import re
import sys

product, roots = sys.argv[1], sys.argv[2:]
rows, subs = {}, {}
for line in open(product, encoding="utf-8"):
    m = re.match(r"E7RESULT name=row row=(\d+) row_name=(\S+) category=([a-z_]+)", line)
    if m:
        rows[int(m.group(1))] = (m.group(2), m.group(3))
    m = re.match(r"E7RESULT name=substitution row=(\d+) .* substituted_by=(\S+) failed_attempts=.* category=([a-z_]+)", line)
    if m:
        subs[int(m.group(1))] = (m.group(2), m.group(3))
for n, v in subs.items():
    rows[n] = v
quiet = {int(x) for x in re.search(r"name=q1_alpha .*row_numbers=([0-9,]+)", open(product, encoding="utf-8").read()).group(1).split(",")}


def table_of(root):
    out = {}
    for line in open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        c = line.rstrip("\n").split("\t")
        out.setdefault(c[0].replace(" ", "_"), []).append(c)
    return out


for root in roots:
    table = table_of(root)
    status = {}
    for n, (name, category) in sorted(rows.items()):
        hits = table.get(name, [])
        if len(hits) != 1:
            status[n] = f"name_hits={len(hits)}"
            continue
        _, file, original = hits[0][:3]
        path = os.path.join(root, file)
        if not os.path.exists(path):
            status[n] = "file_missing"
            continue
        count = open(path, encoding="utf-8").read().count(original.replace("\\n", "\n"))
        status[n] = "ok" if count == 1 else f"anchor_hits={count}"
    bad = {n: s for n, s in status.items() if s != "ok"}
    by_category = {}
    for n in bad:
        by_category[rows[n][1]] = by_category.get(rows[n][1], 0) + 1
    print(f"name=anchor_survival root={root} rows={len(status)} applicable={len(status) - len(bad)} not_applicable={len(bad)} "
          f"by_category={','.join(f'{k}:{v}' for k, v in sorted(by_category.items())) or 'none'} "
          f"counted_not_applicable={','.join(str(n) for n in sorted(bad) if rows[n][1] == 'counted') or 'none'} "
          f"all_quiet_not_applicable={','.join(str(n) for n in sorted(bad) if n in quiet) or 'none'}")
    for n in sorted(bad):
        if rows[n][1] == "counted":
            print(f"name=anchor_lost root={root} row={n} status={bad[n]} row_name={rows[n][0]}")

#!/usr/bin/env python3
"""只读：对 281 行逐行、逐流，比 consulted 恢复结果与基线（同一态下标）：
   older = 实现恢复到的根 (实例,txg) 比基线旧的态数（M4 字面「少施加」的候选）
   newer = 比基线新；other = 根同而内容不同；failed = 实现走读失败而基线没有。
只在两边 states.tsv 的「落地向量」列逐行相同时才逐态比（写流变了的行下标对不上，标 stream_changed）。
用法：recovery_side.py <dumps根> <产物> """
import os, sys
dumps, prod = sys.argv[1], sys.argv[2]
def kv(line):
    d = {}
    for t in line.split(" ")[1:]:
        if "=" in t:
            k, v = t.split("=", 1); d.setdefault(k, v)
    return d
cat, touch, mat = {}, {}, {}
for line in open(prod, encoding="utf-8"):
    line = line.rstrip("\n")
    if line.startswith("E7RESULT name=row row="):
        d = kv(line); cat[int(d["row"])] = d["category"]
    elif line.startswith("E7RESULT name=row_touch row="):
        d = kv(line); touch[int(d["row"])] = d
    elif line.startswith("E7RESULT name=row_matrix row=") and " batch=1 " in line:
        d = kv(line); mat[int(d["row"])] = d
def red(d):
    return "".join(c for c, k in (("1", "O1-对拍"), ("C", "O1-崩溃判官"), ("2", "O2"), ("3", "O3甲")) if d.get(k) == "red") or "-"
def rd(path):
    return [l.rstrip("\n").split("\t") for l in open(path, encoding="utf-8")]
def root(t):
    if t in ("none", "-"): return None
    a, b = t.split(","); return (int(b), int(a))  # (txg, instance)
base = {}
for s in ("fixed_script_through_e_then_unmount", "new_pool_file_creation"):
    base[s] = (rd(f"{dumps}/baseline-run1/{s}/impl-verdicts.tsv"), rd(f"{dumps}/baseline-run1/{s}/states.tsv"))
print("row category red_cols(batch1) write_changed recovery_report_differs | per-stream older/newer/other/failed/diffstates | stream_changed")
for row in range(1, 282):
    top = "rows-s4" if row <= 117 else "rows-s4-f4"
    t = touch.get(row, {})
    out = []
    changed = False
    tot = [0, 0, 0, 0]
    for s, (bi, bs) in base.items():
        p = f"{dumps}/{top}/row-{row}/dump/{s}"
        if not os.path.exists(p + "/impl-verdicts.tsv"):
            out.append(f"{s[:5]}:nofile"); continue
        ri, rs = rd(p + "/impl-verdicts.tsv"), rd(p + "/states.tsv")
        bmap = {x[4]: v for x, v in zip(bs, bi)}
        pairs = [(bmap.get(x[4]), v) for x, v in zip(rs, ri)]
        miss = sum(1 for a, b in pairs if a is None)
        if miss:
            changed = True; out.append(f"{s[:5]}:stream_changed({miss}/{len(pairs)}unmatched)"); continue
        o = n = ot = f = 0
        for a, b in pairs:
            if a[1:3] == b[1:3]: continue
            ra, rb = root(a[1]), root(b[1])
            if b[2].startswith("failed") and not a[2].startswith("failed"): f += 1
            elif ra and rb and rb < ra: o += 1
            elif ra and rb and rb > ra: n += 1
            else: ot += 1
        out.append(f"{s[:5]}:{o}/{n}/{ot}/{f}")
        for i, v in enumerate((o, n, ot, f)): tot[i] += v
    if t.get("recovery_report_differs") == "true" or any(tot):
        print(row, cat.get(row), red(mat[row]) if row in mat else "-", t.get("write_stream_differs"), t.get("recovery_report_differs"), "|", " ".join(out))

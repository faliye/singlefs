#!/usr/bin/env python3
"""只读：对 recovery_side.py 里「恢复比基线旧」的行，数这些态里 O1-崩溃判官红 / O2 红 / 两者都不红的态数（M4 字面的违例态里，有几个态有一列红）。
用法：older_vs_red.py <dumps根> <行号>..."""
import sys, os
dumps = sys.argv[1]
def rd(p): return [l.rstrip("\n").split("\t") for l in open(p, encoding="utf-8")]
def root(t):
    if t in ("none", "-"): return None
    a, b = t.split(","); return (int(b), int(a))
for row in map(int, sys.argv[2:]):
    top = "rows-s4" if row <= 117 else "rows-s4-f4"
    for s in ("fixed_script_through_e_then_unmount", "new_pool_file_creation"):
        bi = rd(f"{dumps}/baseline-run1/{s}/impl-verdicts.tsv"); bs = rd(f"{dumps}/baseline-run1/{s}/states.tsv")
        p = f"{dumps}/{top}/row-{row}/dump/{s}"
        ri = rd(p + "/impl-verdicts.tsv"); rs = rd(p + "/states.tsv"); ro = rd(p + "/oracle-verdicts.tsv")
        bmap = {x[4]: v for x, v in zip(bs, bi)}
        older = c_red = o2_red = neither = 0
        for x, v, o in zip(rs, ri, ro):
            a = bmap.get(x[4])
            if a is None or a[1:3] == v[1:3]: continue
            ra, rb = root(a[1]), root(v[1])
            if ra and rb and rb < ra:
                older += 1
                c = o[5] == "red"; t = o[7] == "red"
                c_red += c; o2_red += t; neither += (not c and not t)
        print(f"row={row} stream={s[:5]} older={older} of_which_crash_judge_red={c_red} o2_red={o2_red} neither={neither}")

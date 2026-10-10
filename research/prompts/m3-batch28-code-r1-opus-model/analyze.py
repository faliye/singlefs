#!/usr/bin/env python3
"""m3-batch28-code-r1 攻方模型：把 hv-new / hv-old / hv-fix 三份日志里的 RESULT HV 行按形对齐，每形一行 RESULT HVSUM。
用法：python3 analyze.py <工作目录>，输出写到 <工作目录>/logs/hvsum.log。"""
import re, sys, pathlib
work = pathlib.Path(sys.argv[1])
rows = {}
pattern = re.compile(r"RESULT HV code=(\w+) tree=(\S+) violation=(\S+) entry=(\S+) other_class_copy=(\S+) target=(\S+) entry_invariant=(\S+) I-2.1:(\S+) core=(.*?) reds=(\[.*\])$")
for code in ("new", "old", "fix"):
    for line in (work / "logs" / f"hv-{code}.log").read_text(encoding="utf-8").splitlines():
        m = pattern.search(line)
        if not m:
            continue
        _, tree, violation, entry, hv, target, entry_inv, _i21, core, reds = m.groups()
        rows.setdefault((tree, violation, entry), {})[(code, hv)] = (target, entry_inv, core, reds)
out = []
for (tree, violation, entry), cells in sorted(rows.items()):
    parts = [f"tree={tree} violation={violation} entry={entry}"]
    target_name = cells[("new", "none")][0].split(":")[0]
    parts.append(f"target={target_name}")
    for code in ("new", "old", "fix"):
        tags = "/".join(cells[(code, hv)][0].split(":")[1] for hv in ("none", "loc0", "loc1"))
        entry_tags = "/".join(cells[(code, hv)][1].split(":")[1] for hv in ("none", "loc0", "loc1"))
        flip = "flip" if cells[(code, "loc0")][3] != cells[(code, "loc1")][3] else "same"
        parts.append(f"{code}[target none/loc0/loc1={tags} entry={entry_tags} loc0-vs-loc1={flip}]")
    parts.append(f"core_loc0={cells[('new', 'loc0')][2]} core_loc1={cells[('new', 'loc1')][2]}")
    out.append("RESULT HVSUM " + " ".join(parts))
(work / "logs" / "hvsum.log").write_text("\n".join(out) + "\n", encoding="utf-8")
print(f"hvsum: {len(out)} 行")

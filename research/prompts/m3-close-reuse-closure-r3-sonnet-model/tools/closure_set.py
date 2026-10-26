#!/usr/bin/env python3
"""closure_set.py <树> <入口> [--under 文件]… ：跑 lib_function_units.py closure（不带 --explain），自己解析输出。
打一行 JSON：units（去重后的 种类\t限定名\t文件 集合大小）、raw_unit_lines、refused、rc；
--dump <路径> 把单元集合（种类|限定名|文件|指纹）一行一个写出去，供 set_diff.py 比。"""
import json, subprocess, sys, os
tree, entry = os.path.abspath(sys.argv[1]), sys.argv[2]
rest = sys.argv[3:]
under, dump = [], None
i = 0
while i < len(rest):
    if rest[i] == "--under":
        under += ["--under", rest[i + 1]]; i += 2
    elif rest[i] == "--dump":
        dump = rest[i + 1]; i += 2
    else:
        raise SystemExit(f"认不出 {rest[i]}")
proc = subprocess.run(["nice", "-n", "19", sys.executable, os.path.join(tree, "scripts/gate/lib_function_units.py"), "closure", tree, entry, *under],
                      capture_output=True, text=True, cwd=tree)
units, refused, raw = set(), [], 0
for line in proc.stdout.splitlines():
    f = line.split("\t")
    if f[0] == "refused":
        refused.append(f[1] + " " + f[2]); continue
    if f[0] == "key" or len(f) < 4:
        continue
    raw += 1
    path = f[2].rsplit(":", 1)[0]
    units.add((f[0], f[1], path, f[3]))
if dump:
    with open(dump, "w", encoding="utf-8") as h:
        for u in sorted(units):
            h.write("|".join(u) + "\n")
files = sorted({u[2] for u in units})
print(json.dumps({"entry": entry, "rc": proc.returncode, "raw_lines": raw, "units_distinct": len(units), "files": len(files), "refused": refused}, ensure_ascii=False))

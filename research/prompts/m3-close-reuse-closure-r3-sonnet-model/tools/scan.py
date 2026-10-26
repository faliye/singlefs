#!/usr/bin/env python3
"""scan.py <树> <入口清单> <输出前缀>：每个入口收一次闭包（广谱变异那 22 个带它自己的 --under 三份，其余默认），
每个入口一行 TSV：入口、rc、原始行数、去重单元数、文件数、拒算、wrapper 单元数（run-with-memory-cap.sh）、host-resources 单元数、集合 sha256。集合另落 <输出前缀>.sets/<序号>.set。"""
import hashlib, json, os, subprocess, sys
tree = os.path.abspath(sys.argv[1]); entries = [l.strip() for l in open(sys.argv[2]) if l.strip()]; prefix = sys.argv[3]
os.makedirs(prefix + ".sets", exist_ok=True)
UNDER = ["scripts/mutation/broad-mutation.py", "scripts/mutation/mutation-prune.py", "scripts/gate/lib_libtest_result_lines.py"]
rows = []
for n, entry in enumerate(entries):
    cmd = ["nice", "-n", "19", sys.executable, os.path.join(tree, "scripts/gate/lib_function_units.py"), "closure", tree, entry]
    if entry.startswith("scripts/mutation/broad-mutation.py::"):
        for u in UNDER: cmd += ["--under", u]
    p = subprocess.run(cmd, capture_output=True, text=True, cwd=tree)
    units, refused, raw = set(), [], 0
    for line in p.stdout.splitlines():
        f = line.split("\t")
        if f[0] == "refused": refused.append(f[1] + "@" + f[2]); continue
        if f[0] == "key" or len(f) < 4: continue
        raw += 1; units.add((f[0], f[1], f[2].rsplit(":", 1)[0], f[3]))
    lines = sorted("|".join(u) for u in units)
    open(f"{prefix}.sets/{n:02d}.set", "w").write("\n".join(lines) + "\n")
    wrap = sum(1 for u in units if u[2] == "scripts/hosts/run-with-memory-cap.sh")
    hr = sum(1 for u in units if u[2] == "scripts/hosts/host-resources.py")
    digest = hashlib.sha256("\n".join(lines).encode()).hexdigest()[:12]
    rows.append((n, entry, p.returncode, raw, len(units), len({u[2] for u in units}), ";".join(refused), wrap, hr, digest))
with open(prefix + ".tsv", "w") as h:
    for r in rows: h.write("\t".join(str(x) for x in r) + "\n")
print(f"{prefix}: {len(rows)} 个入口")

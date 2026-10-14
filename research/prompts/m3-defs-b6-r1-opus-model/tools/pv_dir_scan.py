#!/usr/bin/env python3
"""数 <目录> 里文件内容与路径命中 private-values 取词的个数（取词照 <门禁根> doc-text.sh 那一段，配置取 <multi-host.env>）；只打个数，不打值。
用法：pv_dir_scan.py <门禁根> <multi-host.env> <目录>"""
import os, subprocess, sys
gate_root, configuration, tree = map(os.path.abspath, sys.argv[1:4])
lines = open(os.path.join(gate_root, ".claude/gate.d/doc-text.sh"), encoding="utf-8").read().split("\n")
start = next(i for i, l in enumerate(lines) if l.startswith('  python3 - "$base" "$configuration_file" "$hostname_file"'))
stop = next(i for i in range(start, len(lines)) if lines[i].startswith("def git("))
code = "\n".join(lines[start + 1:stop]) + r'''
hits = 0; files = 0
for root, dirs, names in os.walk(TREE):
    for name in names:
        p = os.path.join(root, name); files += 1
        try: text = open(p, encoding="utf-8", errors="replace").read()
        except OSError: continue
        if any(w in text or w in os.path.relpath(p, TREE) for _k, w in targets): hits += 1
print(f"扫了 {files} 份，内容或路径命中私有值的 {hits} 份（取词 {len(targets)} 个）")
'''.replace("TREE", repr(tree))
r = subprocess.run([sys.executable, "-", "x", configuration, "", "/dev/null", "", ""], input=code, text=True, capture_output=True)
sys.stdout.write(r.stdout); sys.stderr.write(r.stderr[-500:]); sys.exit(r.returncode)

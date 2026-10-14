#!/usr/bin/env python3
"""private-values 格的取词那一段（从 <门禁根>/.claude/gate.d/doc-text.sh 抽出，到 def git( 为止）原样跑，
再数 <被判树> 里每个文件的相对路径命中几个词：只打印键名与个数，不打印值、不打印命中的路径。
用法：pv_path_probe.py <门禁根> <multi-host.env> <被判树>"""
import os, subprocess, sys
gate_root, configuration, tree = map(os.path.abspath, sys.argv[1:4])
lines = open(os.path.join(gate_root, ".claude/gate.d/doc-text.sh"), encoding="utf-8").read().split("\n")
start = next(i for i, l in enumerate(lines) if l.startswith('  python3 - "$base" "$configuration_file" "$hostname_file"'))
stop = next(i for i in range(start, len(lines)) if lines[i].startswith("def git("))
code = "\n".join(lines[start + 1:stop]) + r'''
import collections
slash = sum(1 for _k, w in targets if "/" in w)
print(f"取到的词 {len(targets)} 个（含本机主机名）；带 / 的 {slash} 个")
hit_files, by_key, cross = 0, collections.Counter(), 0
total = 0
for root, dirs, files in os.walk(TREE):
    dirs[:] = [d for d in dirs if d not in (".git", "target")]
    for name in files:
        rel = os.path.relpath(os.path.join(root, name), TREE)
        if rel == "multi-host.env":
            continue
        total += 1
        keys = {k for k, w in targets if w in rel}
        if keys:
            hit_files += 1
            by_key.update(keys)
            if any(w in rel and not any(w in part for part in rel.split("/")) for _k, w in targets):
                cross += 1
print(f"树里 {total} 个文件（除 multi-host.env），相对路径命中的 {hit_files} 个；按键名：{dict(by_key)}；命中跨了 / 而逐段打码打不到的 {cross} 个")
'''
code = code.replace("TREE", repr(tree))
result = subprocess.run([sys.executable, "-", "x", configuration, "", "/dev/null", "", ""], input=code, text=True, capture_output=True)
sys.stdout.write(result.stdout); sys.stderr.write(result.stderr[-2000:])
sys.exit(result.returncode)

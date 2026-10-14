#!/usr/bin/env python3
"""private-values 格的取词那一段（从 <门禁根>/.claude/gate.d/doc-text.sh 抽出，到 def git( 为止）原样跑，
再数取到的长词里有几个是本机常见绝对路径（$HOME、$HOME/code、工作根、给定的仓根）的子串：只打印个数，不打印值与键名。
用法：pv_prefix_probe.py <门禁根> <multi-host.env> <仓根>"""
import os, subprocess, sys
gate_root, configuration, repo = map(os.path.abspath, sys.argv[1:4])
lines = open(os.path.join(gate_root, ".claude/gate.d/doc-text.sh"), encoding="utf-8").read().split("\n")
start = next(i for i, l in enumerate(lines) if l.startswith('  python3 - "$base" "$configuration_file" "$hostname_file"'))
stop = next(i for i in range(start, len(lines)) if lines[i].startswith("def git("))
home = os.path.expanduser("~")
probes = {"$HOME": home, "$HOME/code": os.path.join(home, "code"), "工作根": os.path.join(home, ".local/share/singlefs/work"), "仓根": repo,
          "本机仓根下 research/results 子目录": os.path.join(repo, "research/results/e999")}
code = "\n".join(lines[start + 1:stop]) + "\nPROBES = " + repr(probes) + r'''
print(f"取到的词 {len(targets)} 个（含本机主机名）")
for label, text in PROBES.items():
    print(f"{label}：是它子串的词 {sum(1 for _k, w in targets if w in text)} 个")
'''
result = subprocess.run([sys.executable, "-", "x", configuration, "", "/dev/null", "", ""], input=code, text=True, capture_output=True)
sys.stdout.write(result.stdout); sys.stderr.write(result.stderr[-2000:])
sys.exit(result.returncode)

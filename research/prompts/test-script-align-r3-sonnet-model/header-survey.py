#!/usr/bin/env python3
"""C625 翻面观测 1：含 _BREAK 的文件里，文件头（前 140 行）用的是哪种弄坏开关写法。启发式：
A = 文件头里有「第 N 格」（crash-amplification.sh 写法）；B = 文件头里有 `# <变量>_BREAK=<名>` 起头的行（mutate.sh 写法）。
只读，不跑任何脚本。用法：python3 header-survey.py（在仓里任意位置）"""
import glob, os, re, subprocess
root = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip()
os.chdir(root)
files = sorted({f for d in ("research/scripts", ".claude/hooks", ".claude/gate.d", ".claude/gate.d/lib") for f in glob.glob(os.path.join(d, "*"))
                if os.path.isfile(f) and not f.endswith((".tsv", ".md", ".json", ".pyc"))})
style_a = re.compile(r"第\s*\d+\s*格")
style_b = re.compile(r"^#\s+[A-Z][A-Z0-9_]*_BREAK=[A-Za-z0-9_-]+", re.M)
rows = []
for f in files:
    text = open(f, encoding="utf-8", errors="replace").read()
    if "_BREAK" not in text:
        continue
    head = "\n".join(text.splitlines()[:140])
    rows.append((f, "_BREAK" in head, bool(style_a.search(head)), len(style_b.findall(head))))
print("files with _BREAK:", len(rows))
print("BREAK mentioned in first 140 lines:", sum(1 for r in rows if r[1]))
print("style A:", sum(1 for r in rows if r[2]), [r[0] for r in rows if r[2]])
print("style B:", sum(1 for r in rows if r[3] > 0), [r[0] for r in rows if r[3] > 0])
print("neither:", sum(1 for r in rows if r[1] and not r[2] and r[3] == 0))
print("no mention in first 140 lines:", sum(1 for r in rows if not r[1]))

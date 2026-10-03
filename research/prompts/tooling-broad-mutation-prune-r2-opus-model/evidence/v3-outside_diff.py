#!/usr/bin/env python3
"""V3：一次提交里「函数外面的部分」变了的那几行（去注释、去空白之后逐行比）。用法：outside_diff.py <仓> <提交>"""
import difflib, os, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fnparse import outside_of_functions, strip
REPO, C = sys.argv[1:3]
git = lambda *a: subprocess.run(["git", "-C", REPO, *a], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True).stdout
for f in [f for f in git("diff", "--name-only", C + "^", C, "--", "crates/singlefs-core/src", "crates/singlefs-checker/src").split() if f.endswith(".rs")]:
    norm = lambda t: [l.strip() for l in strip(outside_of_functions(t), strings=False).splitlines() if l.strip()]
    d = [l for l in difflib.unified_diff(norm(git("show", f"{C}^:{f}")), norm(git("show", f"{C}:{f}")), lineterm="", n=0) if l[:1] in "+-" and not l.startswith(("+++", "---"))]
    if d:
        print(f, len(d), "|", " || ".join(x[:90] for x in d[:4]))

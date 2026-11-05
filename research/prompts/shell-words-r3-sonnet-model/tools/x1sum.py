"""x1sum.py : X1 各样本集在 r1 / r2 / now 上的迁移计数，与 now 上判错的条（按 tag 分组、列前几条）。"""
import subprocess, sys, glob, os
sets = sorted(os.path.basename(f)[len("cases-"):-5] for f in glob.glob("cases/cases-x1-*.json"))
for s in sets:
    for pair in (("r1", "r2"), ("r1", "now")):
        print(subprocess.run(["python3", "-I", "tools/trans.py", s, *pair], capture_output=True, text=True).stdout.strip())
    print(subprocess.run(["python3", "-I", "tools/show.py", s, "now", "", "5"], capture_output=True, text=True).stdout.rstrip())
    print()

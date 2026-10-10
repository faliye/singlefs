#!/usr/bin/env python3
"""用法：apply.py <workdir> <序号|restore>。把 mutations.py 里第 N 项落到 workdir 里的 walk.rs（原文必须恰好命中一次），或还原。"""
import sys, shutil, os
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
from mutations import MUTATIONS
work, which = sys.argv[1], sys.argv[2]
target = os.path.join(work, "crates/singlefs-checker/src/walk.rs")
pristine = target + ".pristine"
if not os.path.exists(pristine):
    shutil.copy2(target, pristine)
if which == "restore":
    shutil.copyfile(pristine, target)
    print("restored")
    sys.exit(0)
name, clause, old, new, _ = MUTATIONS[int(which)]
text = open(pristine, encoding="utf-8").read()
hits = text.count(old)
if hits != 1:
    print(f"ANCHOR_HITS={hits} for {name}")
    sys.exit(3)
open(target, "w", encoding="utf-8").write(text.replace(old, new))
print(f"applied {name}")

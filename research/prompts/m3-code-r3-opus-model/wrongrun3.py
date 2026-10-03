#!/usr/bin/env python3
"""wrongrun.py apply|restore <副本根> <冻结根> <错实现名> —— apply 逐条把原文（恰好命中一次）换成替换文；
restore 从冻结副本把那几份文件的原样内容写回（新写出的 mtime 是现在，cargo 会重编）。"""
import os, sys, importlib.util
action, work, frozen, name = sys.argv[1:5]
spec = importlib.util.spec_from_file_location("wrongs3", os.path.join(os.path.dirname(os.path.abspath(__file__)), "wrongs3.py"))
wrongs = importlib.util.module_from_spec(spec); spec.loader.exec_module(wrongs)
edits = wrongs.WRONGS[name]
files = sorted({f for f, _, _ in edits})
if action == "restore":
    for f in files:
        with open(os.path.join(frozen, f), encoding="utf-8") as src:
            text = src.read()
        with open(os.path.join(work, f), "w", encoding="utf-8") as dst:
            dst.write(text)
    print(f"RESTORED {name} {files}")
    sys.exit(0)
assert action == "apply"
for f in files:
    with open(os.path.join(work, f), encoding="utf-8") as src, open(os.path.join(frozen, f), encoding="utf-8") as orig:
        assert src.read() == orig.read(), f"{f} 不是原样，先 restore"
texts = {}
for f in files:
    with open(os.path.join(work, f), encoding="utf-8") as src:
        texts[f] = src.read()
for f, old, new in edits:
    hits = texts[f].count(old)
    assert hits == 1, f"{name}: {f} 原文命中 {hits} 次"
    texts[f] = texts[f].replace(old, new)
for f in files:
    with open(os.path.join(work, f), "w", encoding="utf-8") as dst:
        dst.write(texts[f])
print(f"APPLIED {name} {len(edits)} 处")

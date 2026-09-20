#!/usr/bin/env python3
"""把 old 换成 new（old 在文件里要恰好命中 1 次，或按 --nth 取第 n 次，从 1 数）；--restore 从 .orig 还原。只在仓副本里用。"""
import sys, pathlib, shutil, os
args = sys.argv[1:]
if args[0] == "--restore":
    p = pathlib.Path(args[1]); o = p.with_name(p.name + ".orig")
    if o.exists():
        shutil.move(str(o), str(p)); os.utime(str(p))  # 还原之后刷新 mtime，不然 cargo 按旧 mtime 当成新鲜、留着变异的编译产物
    sys.exit(0)
nth = None
if args[0] == "--nth":
    nth = int(args[1]); args = args[2:]
path, old_file, new_file = args
p = pathlib.Path(path)
text = p.read_text()
old = pathlib.Path(old_file).read_text()
new = pathlib.Path(new_file).read_text()
count = text.count(old)
if nth is None and count != 1:
    print(f"MUTATE-FAIL: old matched {count} times", file=sys.stderr); sys.exit(3)
if nth is not None and count < nth:
    print(f"MUTATE-FAIL: old matched {count} < nth {nth}", file=sys.stderr); sys.exit(3)
shutil.copy(str(p), str(p.with_name(p.name + ".orig")))
idx = -1
for _ in range(nth or 1):
    idx = text.index(old, idx + 1)
p.write_text(text[:idx] + new + text[idx + len(old):])
print(f"MUTATED {path} at offset {idx} (matches {count})")

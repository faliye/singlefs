#!/usr/bin/env python3
# 用法：patch.py apply <相对文件> <旧串文件> <新串文件> | restore <相对文件>
import sys, shutil, os
root = "/tmp/claude-1000/m3-root-ring-code-r2-sonnet/work/tree/"
cmd, rel = sys.argv[1], sys.argv[2]
path = root + rel
bak = path + ".orig-probe"
if cmd == "apply":
    old = open(sys.argv[3], encoding="utf-8").read().rstrip("\n")
    new = open(sys.argv[4], encoding="utf-8").read().rstrip("\n")
    text = open(path, encoding="utf-8").read()
    n = text.count(old)
    if n != 1:
        print(f"旧串命中 {n} 次，不改"); sys.exit(2)
    if not os.path.exists(bak):
        shutil.copy2(path, bak)
    open(path, "w", encoding="utf-8").write(text.replace(old, new))
    print("已改")
elif cmd == "restore":
    if os.path.exists(bak):
        shutil.copy2(bak, path); os.remove(bak); os.utime(path, None); print("已还原")
    else:
        print("没有备份")

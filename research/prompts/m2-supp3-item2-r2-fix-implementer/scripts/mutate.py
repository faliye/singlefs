#!/usr/bin/env python3
"""草稿用：在仓副本里施加 / 还原一条变异。
apply <root> <relfile> <old-text-file> <new-text-file> [nth]   第 nth 处（从 1 数；不给则要求恰好命中一次）
restore <root> <relfile>                                        从 .pristine 拷回并把 mtime 摸成现在（cargo 按 mtime 判新旧）
"""
import os, shutil, sys, time
mode, root, rel = sys.argv[1], sys.argv[2], sys.argv[3]
path = os.path.join(root, rel)
pristine = path + ".pristine"
if mode == "apply":
    old = open(sys.argv[4], encoding="utf-8").read().rstrip("\n")
    new = open(sys.argv[5], encoding="utf-8").read().rstrip("\n")
    nth = int(sys.argv[6]) if len(sys.argv) > 6 else 0
    if os.path.exists(pristine):
        sys.exit(f"{pristine} 已存在：上一条没还原")
    text = open(path, encoding="utf-8").read()
    count = text.count(old)
    if nth == 0:
        if count != 1:
            sys.exit(f"原文命中 {count} 次，要恰好 1 次")
        nth = 1
    if count < nth:
        sys.exit(f"原文只命中 {count} 次，要第 {nth} 处")
    shutil.copy2(path, pristine)
    start = -1
    for _ in range(nth):
        start = text.index(old, start + 1)
    mutated = text[:start] + new + text[start + len(old):]
    open(path, "w", encoding="utf-8").write(mutated)
    now = time.time(); os.utime(path, (now, now))
    print(f"施加：{rel} 第 {nth} 处（共 {count} 处）")
elif mode == "restore":
    shutil.copyfile(pristine, path)
    os.remove(pristine)
    now = time.time(); os.utime(path, (now, now))
    print(f"还原：{rel}")
else:
    sys.exit("apply | restore")

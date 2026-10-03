#!/usr/bin/env python3
"""照 stage-selftest.sh 的判法（退出码等于 expect 的 exit=，每条 want= 在输出里）判 out/<前缀>-<样本>.txt。"""
import os, sys
fixtures, out, prefix = sys.argv[1:4]
wrong, right = [], []
for name in sorted(os.listdir(fixtures)):
    expect = open(os.path.join(fixtures, name, "expect"), encoding="utf-8").read().splitlines()
    want_exit = next(line[5:] for line in expect if line.startswith("exit="))
    wants = [line[5:] for line in expect if line.startswith("want=") and line[5:]]
    text = open(os.path.join(out, f"{prefix}-{name}.txt"), encoding="utf-8", errors="replace").read()
    got = text.rstrip("\n").splitlines()[-1].removeprefix("exit=")
    missing = [w for w in wants if w not in text]
    if got != want_exit or missing:
        wrong.append(f"{name}(期望{want_exit}→实测{got}{'，缺 want ' + str(len(missing)) + ' 条' if missing else ''})")
    else:
        right.append(name)
print(f"{prefix} 判对 {len(right)} 个、判错 {len(wrong)} 个：{' '.join(wrong) if wrong else '无'}")

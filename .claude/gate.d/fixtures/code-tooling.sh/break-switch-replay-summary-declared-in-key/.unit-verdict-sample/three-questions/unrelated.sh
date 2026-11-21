#!/usr/bin/env bash
# 复用三问的闭包外改动：只改 alpha.py 第二行的说明（注释，不进单元指纹，也不是开关表里的哪一条），五条的键都不变
set -euo pipefail
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' scripts/gate/alpha.py "unrelated.sh 只改这一行说明" "unrelated.sh 只改这一行说明（改过了）"

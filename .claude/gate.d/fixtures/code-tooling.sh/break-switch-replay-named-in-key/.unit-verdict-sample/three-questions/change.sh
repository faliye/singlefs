#!/usr/bin/env bash
# 复用三问的「闭包里的输入」改动：drop 那一条点名的项改写成带 … 的写法（「✗ 自检：丢了的…那一格」照样照得出那一行），只有这一条的键变
set -euo pipefail
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' scripts/gate/alpha.py "alpha-drop（「✗ 自检：丢了的那一格」）" "alpha-drop（「✗ 自检：丢了的…那一格」）"

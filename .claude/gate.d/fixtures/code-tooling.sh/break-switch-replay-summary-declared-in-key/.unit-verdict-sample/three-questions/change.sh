#!/usr/bin/env bash
# 复用三问的「闭包里的输入」改动：alpha.py 文件头 break-summary-rows 那一行声明的写法改一下（「… 格不对」改成「…格不对」，照样照得出同一批行），alpha 的两条开关的键都变、基准趟与 beta 不变
set -euo pipefail
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' scripts/gate/alpha.py "「✗ 自检：… 格不对（查了 … 格）」" "「✗ 自检：…格不对（查了 … 格）」"

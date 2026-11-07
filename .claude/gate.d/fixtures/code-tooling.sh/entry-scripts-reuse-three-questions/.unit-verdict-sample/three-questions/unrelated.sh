#!/usr/bin/env bash
# 复用三问的闭包外改动：登记表 alpha 那一行只改理由那一列（这一格不读理由，不进键），两条的键都不变
set -euo pipefail
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' .claude/gate.d/entry-scripts.tsv '# 样本：alpha 带齐' '# 样本：alpha 带齐，只改了理由'

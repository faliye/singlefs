#!/usr/bin/env bash
# 复用三问的闭包内改动：alpha.py 的 expected_length 换个写法（恰好改中一次，改不中退 1），判出来的照旧；闭包含它的是 alpha 的两格
set -euo pipefail
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' scripts/gate/alpha.py "    return 2
" "    return 1 + 1
"

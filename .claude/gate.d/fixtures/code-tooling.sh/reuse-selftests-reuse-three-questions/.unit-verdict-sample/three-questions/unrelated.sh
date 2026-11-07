#!/usr/bin/env bash
# 复用三问的闭包外改动：登记表 gamma.sh 那一行加一列说明（audit 不读说明列，不进键），三行的键都不变
set -euo pipefail
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' .claude/gate.d/verdict-library-scripts.tsv \
  $'scripts/gate/gamma.sh\tbash scripts/gate/gamma.sh --selftest\t样本里的假库\treuse-three-questions\n' \
  $'scripts/gate/gamma.sh\tbash scripts/gate/gamma.sh --selftest\t样本里的假库\treuse-three-questions\t# 样本：只加了说明列\n'

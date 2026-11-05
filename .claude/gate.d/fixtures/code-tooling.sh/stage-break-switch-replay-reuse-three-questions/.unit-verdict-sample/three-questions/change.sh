#!/usr/bin/env bash
# 复用三问的闭包内改动：alpha-stage.sh 的 judged_verdict 换个打法（恰好改中一次，改不中退 1），打出来的照旧；闭包含它的是 alpha 的两条开关
set -euo pipefail
python3 -c '
import sys
path, old, new = sys.argv[1:4]
text = open(path, encoding="utf-8").read()
if text.count(old) != 1:
    sys.exit(f"样本坏了：{path} 里「{old}」命中 {text.count(old)} 次（要恰好 1 次）")
open(path, "w", encoding="utf-8").write(text.replace(old, new))' .claude/gate.d/alpha-stage.sh "printf '%s\n' \"\$verdict\"" "echo \"\$verdict\""

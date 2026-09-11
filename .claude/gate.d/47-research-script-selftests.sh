#!/usr/bin/env bash
# gate-stage: 三方论证那几个 research 脚本的自证还会红
#
# 判据：`research/scripts/ask-local-selftest.sh`、`checklist-specs.py --selftest`、`quote-kb.py --selftest`、
# `kb-sections.py --selftest` 四份都通过。
# 为什么：这几份自证此前都写着，却没有任何门禁阶段在跑（2026-09-12 现查 gate.d 与 .claude/scripts 零处调用）——自证只在写它的那天被跑过一次，
# 之后脚本改坏了也没人知道。2026-09-12 实测的两个坑都住在这里：
# ask-local.sh 判红时正文照样打到 stdout（一份作废输出顶着 -output-s1.md 落盘），
# 以及取法用不加引号的 $SPECS 传过 shell 被拆词（checklist-specs.py 就是为它写的）。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" || exit 1

failed=0
checked=0
for runner in "bash research/scripts/ask-local-selftest.sh" "python3 research/scripts/checklist-specs.py --selftest" \
              "python3 research/scripts/quote-kb.py --selftest" "python3 research/scripts/kb-sections.py --selftest"; do
  checked=$((checked + 1))
  output="$($runner 2>&1)"; rc=$?
  if [[ $rc -ne 0 ]]; then
    echo "  ✗ $runner 没过（退出码 $rc）："
    printf '%s\n' "$output" | sed 's/^/    /'   # gate-lint:detail
    echo "  → 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。"
    failed=1
  fi
done
((failed)) && exit 1
echo "  ✓ 三方论证脚本的自证都通过（查了 $checked 份：ask-local 判红分支、清单生成取法、机械整抄、小节清单）"

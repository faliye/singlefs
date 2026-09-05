#!/usr/bin/env bash
# gate-stage: 动了用户定案的条款有没有记一笔未还的账
#
# 判据（`.claude/singlefs-ai-sop/skills/decide/SKILL.md` 硬要求第 6 条）：
# 决策正文里凡标着「待用户复核」的条款，`checks-owed.md` 里必须有一笔**未还**的账点名它。
#
# ⚠️ **这条是实测出来的**：2026-09-05 C113 定案顺手改了 D16 已定项 5 的丢失窗口口径，
# 而那一句是 2026-08-31 由用户拍板的。正文里写了「待用户复核」，然后就没有任何东西
# 保证它会被看到——唯一提到它的那笔账（C113）当天就标了「已还」。
# 拍板的人不知道自己的定案变了，而门禁全绿。
#
# 判「未还」看账那一行有没有带日期的还清标记（`已还（2026-…`、`已还一半（…`）：
# 已还的账不会再被回看，等于没有账。要留着盯，就单立一笔。
# 不按「已还」两字判——那两个字出现在描述里就会把整笔账误判成还清（写这条时实测）。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
KB="$ROOT/.claude/kb"
OWED="$KB/checks-owed.md"
MARK='待用户复核\|等用户复核'

[[ -d "$KB/decisions" ]] || { echo "  ! 找不到 $KB/decisions，本阶段跳过"; exit 0; }
[[ -f "$OWED" ]] || { echo "  ✗ 缺 $OWED"
  echo "     → 怎么办： 待复核的条款要有账本盯着。先建 checks-owed.md，再把这一笔记进去。"; exit 1; }

fail=0; checked=0
while IFS= read -r f; do
  grep -q "$MARK" "$f" || continue
  # 决策号取自文件名：`16-发布语义.md` → D16
  d="D$(basename "$f" | sed 's/^0*//; s/-.*//')"
  checked=$((checked+1))
  # 未还的账：同一行里点名了这条决策、带着复核标记，且没标「已还」
  if grep "$MARK" "$OWED" | grep -F "$d" | grep -qv "已还[^ |]*（20"; then continue; fi
  echo "  ✗ $d 的正文标着待用户复核，checks-owed.md 里却没有一笔**未还**的账点名它"
  echo "        正文：$(basename "$f")"
  echo "     → 怎么办： 在 checks-owed.md 立一笔账，点名是 $d 的哪一项、原定案是哪天由谁拍的、"
  echo "                复核要回答什么问题；复核过了再标已还。"
  echo "                只在正文写一句「待用户复核」，没有任何东西保证它会被看到"
  echo "                （decide skill 硬要求第 6 条）。"
  fail=$((fail+1))
done < <(find "$KB/decisions" -name '*.md' | sort)

if [[ $fail -gt 0 ]]; then
  echo "  ✗ $fail 处待用户复核没有未还的账（共查 $checked 处）"
  exit 1
fi
echo "  ✓ 待用户复核的条款都有未还的账盯着（共 $checked 处）"

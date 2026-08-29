#!/usr/bin/env bash
# gate-stage: 项目规则清单一致
#
# 还 checks-owed.md C4（项目规则清单不一致）。
# 判据：`.claude/rules/` 下的文件集合，必须与 CLAUDE.md 里 `@.claude/rules/` 的引用集合逐项相等。
# 任一侧多出一项即判红——多出的那一份规则**不会被读进上下文**，等于没写。
#
# ⚠️ 上游 manifest.sh 只覆盖 SOP 包自己的 CLAUDE.md + rules/，
# 项目本地那一份此前没有任何检查在看（C4 的原文）。
set -uo pipefail
RULES=.claude/rules
MD=CLAUDE.md
[[ -d "$RULES" ]] || { echo "  ✓ 没有 $RULES，无对象可判"; exit 0; }
[[ -f "$MD" ]] || { echo "  ! 找不到 $MD，本阶段跳过"; exit 0; }

on_disk="$(find "$RULES" -maxdepth 1 -name '*.md' -printf '%f\n' | sort)"
referenced="$(grep -oE '@\.claude/rules/[A-Za-z0-9._-]+\.md' "$MD" | sed 's|.*/||' | sort -u)"

missing="$(comm -23 <(printf '%s\n' "$on_disk") <(printf '%s\n' "$referenced"))"
extra="$(comm -13 <(printf '%s\n' "$on_disk") <(printf '%s\n' "$referenced"))"

rc=0
if [[ -n "$missing" ]]; then
  echo "  ✗ 这些规则文件存在，但 $MD 没有 @ 引用它们——不会被读进上下文，等于没写："
  printf '%s\n' "$missing" | sed 's|^|     .claude/rules/|'
  rc=1
fi
if [[ -n "$extra" ]]; then
  echo "  ✗ $MD 引用了这些规则，但文件不存在——引用悬空："
  printf '%s\n' "$extra" | sed 's|^|     .claude/rules/|'
  rc=1
fi
if ((rc)); then
  echo "     → 怎么办：补上缺的 @ 引用，或删掉悬空引用；两侧必须逐项相等。"
  exit 1
fi
echo "  ✓ 项目规则清单一致（$(printf '%s\n' "$on_disk" | grep -c . ) 条，与 $MD 的引用逐项相符）"

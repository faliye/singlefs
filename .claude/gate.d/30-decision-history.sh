#!/usr/bin/env bash
# gate-stage: 决策变更留痕
#
# 改了 decisions.md 却没在文末「历史版本」留条目 ⇒ 判红。
#
# 为什么判红而不是提醒：`.claude/rules/format-evolution.md` 已定
# 「决策变更**必须记进** decisions.md，含推翻依据」；
# 而 kb-discipline 又要求正文只写现状 ⇒ **被推翻的那句话会被直接改掉**，
# 不留条目就等于它从未存在过，三个月后没人知道它为什么不算数了。
#
# 实测：这条检查是自查出来的——连续三个改动决策结论的提交（定案 D9 未答项 6、
# 定案 D23 轴二、收窄多流结论并改判 D17）**一条历史条目都没加**。
#
# ⚠️ 判据是「有没有新增条目」，不判「条目写得对不对」——后者只有人能判。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" || exit 2
KB=".claude/kb/decisions.md .claude/kb/decisions"
HIST=.claude/kb/decisions-history.md
bad() { printf '  ✗ %s\n' "$*"; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

if git diff --quiet HEAD -- "$KB" 2>/dev/null; then
  ok "决策正文与 HEAD 无差异，本阶段无对象可判"
  exit 0
fi

added=$(git diff HEAD -- "$KB" | grep -c '^+### 2026-' || true)
changed=$(git diff HEAD --numstat -- "$KB" | awk '{print $1+$2}')
if [[ "$added" -gt 0 ]]; then
  ok "决策正文改了 $changed 行，新增 $added 条历史版本条目"
  exit 0
fi
if [[ "${changed:-0}" -le 4 ]]; then
  ok "决策正文只改了 $changed 行，按小改动放行"
  exit 0
fi

bad "决策正文改了 $changed 行，却没有新增任何历史版本条目"
howto "若这次改动推翻或定下了任何结论，在文末「## 历史版本」加一条："
howto "  ### $(date +%F)  —— 曾经 X / 现在 Y / 依据 Z"
howto "纯排版改动可以拆成单独一个提交，那时这一项就无对象可判了。"
exit 1

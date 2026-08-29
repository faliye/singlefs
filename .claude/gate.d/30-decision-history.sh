#!/usr/bin/env bash
# gate-stage: 决策变更留痕
#
# 改了决策正文却没在 `kb/decisions-history.md` 留条目 ⇒ 判红。
#
# 为什么判红而不是提醒：`.claude/rules/format-evolution.md` 已定
# 「决策变更**必须记进** decisions-history.md，含推翻依据」；
# 而 kb-discipline 又要求正文只写现状 ⇒ **被推翻的那句话会被直接改掉**，
# 不留条目就等于它从未存在过，三个月后没人知道它为什么不算数了。
#
# ⚠️ 判据是「有没有新增条目」，不判「条目写得对不对」——后者只有人能判。
#
# ⚠️ **本阶段自己恒绿过一段时间**（2026-08-29 复跑复核轮查出）：两个路径被塞进一个变量再加引号，
# `git diff -- "$KB"` 于是拿一个「两条路径粘成的字符串」当单个 pathspec，**匹配不到任何文件**。
# 加上历史外置到 decisions-history.md 之后仍在决策正文里找 `### 2026-`，判据也早已错位。
# ⇒ 路径改成数组，历史条目改到正确的文件里数。**双向证过会红**，见文末注释。
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" || exit 2
KB=(.claude/kb/decisions.md .claude/kb/decisions)
HIST=.claude/kb/decisions-history.md
bad() { printf '  ✗ %s\n' "$*"; }
ok()  { printf '  ✓ %s\n' "$*"; }
howto() { printf '     → %s\n' "$*"; }

if git diff --quiet HEAD -- "${KB[@]}" 2>/dev/null; then
  ok "决策正文与 HEAD 无差异，本阶段无对象可判"
  exit 0
fi

# 决策正文改了多少行（增 + 删，各文件相加）
changed=$(git diff HEAD --numstat -- "${KB[@]}" | awk '{n+=$1+$2} END{print n+0}')
# 本次 diff 往变更史里加了几条日期标题
added=$(git diff HEAD -- "$HIST" | grep -c '^+### 20[0-9][0-9]-' || true)

if [[ "$added" -gt 0 ]]; then
  ok "决策正文改了 $changed 行，变更史新增 $added 条条目"
  exit 0
fi
if [[ "$changed" -le 4 ]]; then
  ok "决策正文只改了 $changed 行，按小改动放行"
  exit 0
fi

bad "决策正文改了 $changed 行，却没往 $HIST 新增任何条目"
howto "若这次改动推翻或定下了任何结论，在 $HIST 的「## 历史版本」下加一条："
howto "  ### $(date +%F)  —— 曾经 X / 现在 Y / 依据 Z"
howto "纯排版改动可以拆成单独一个提交，那时这一项就无对象可判了。"
exit 1

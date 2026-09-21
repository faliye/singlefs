#!/usr/bin/env bash
# gate-stage: 改过名的术语，全仓不再出现旧名（research/prompts/、research/results/、crates/ 都算）
#
# 规则见 .claude/kb/term-renames.md：每改一个全仓术语，往那张登记表加一行「旧 新 匹配」，这道阶段从此替你盯着。
# 实测（2026-09-21）：「超级块 → 系统配置」第一批只扫了现状类 96 份 1273 处，而全仓真实残余是 1137 份 22741 处——
# 中文旧名 10947 处、下划线与连字符变体、以及 `sb` 系缩写（`sb_mac`、`tail_sb`）一处都没动，当时没有任何检查报得出来。
# 同一天还查出：一类名字活在字符串与表格里（字段表标签、臂名、步骤种类串），naming-lint 的射程罩不到（C430）。
# 判别力：fixtures/90-term-renames.sh/red 放一份带旧名的文件，必须判红；green 只有新名，必须判绿。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
SCRIPT="$(cd "$(dirname "$0")/../.." && pwd)/research/scripts/sweep-term.py"
[[ -f "$ROOT/.claude/kb/term-renames.md" ]] || { echo "  ! 没有 .claude/kb/term-renames.md，本阶段无对象可判"; exit 77; }
if python3 "$SCRIPT" --check "$ROOT"; then
  exit 0
fi
echo "  ✗ 登记过的术语改名，仓里还有地方用着旧名（上面逐份列出份数与处数）"
echo "     → 怎么办：python3 research/scripts/sweep-term.py --apply 一次换完，源码与留存产物同改；"
echo "               换完跑 cargo test --workspace 与 bash research/scripts/replay.sh，产物必须仍然逐字节一致。"
echo "               确实该留旧名的（别家术语、搬迁登记表的输入列、对照表自己），登记进 .claude/term-rename-exempt 并写明为什么。"
exit 1

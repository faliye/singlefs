#!/usr/bin/env bash
# gate-stage: 改过名的术语，全仓不再出现旧名（research/prompts/、research/results/、crates/ 都算）
#
# 规则见 .claude/kb/term-renames.md：每改一个全仓术语，往那张登记表加一行「旧 新 匹配」，这道阶段从此替你盯着。
# 实测（2026-09-21）：「系统配置 → 系统配置」第一批只扫了现状类 96 份 1273 处，而全仓真实残余是 1137 份 22741 处——
# 中文旧名 10947 处、下划线与连字符变体、以及 `system_configuration` 系缩写（`system_configuration_mac`、`tail_system_configuration`）一处都没动，当时没有任何检查报得出来。
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
echo "     → 怎么办：python3 research/scripts/sweep-term.py --apply 一次换完。"
echo "               产物分两类，别一把梭：**跑得出来的**（装置源码在树里、replay.sh 判 exact）跟着源码一起换，"
echo "               换完跑 cargo test --workspace 与 bash research/scripts/replay.sh，同一份源码要仍然吐出逐字节相同的产物——"
echo "               那是重新生成，不是改证据；**跑不出来的**（已归档、要虚机或真设备、别人的产物）一个字节都不许动，"
echo "               连同引它的正文整段留旧名（evidence-discipline「原样保存的证据不许事后改」）。"
echo "               确实该留旧名的（别家术语、冻结证据目录、历史类文件、引文块、对照表自己），"
echo "               登记进 .claude/term-rename-exempt 并写明为什么。"
exit 1

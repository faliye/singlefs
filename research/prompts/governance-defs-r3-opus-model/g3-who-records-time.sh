#!/usr/bin/env bash
# G3：「腿开工时刻 / 派腿的时刻」在哪几份文件里被要求记；设计轮有没有人被要求给。
# 用法：bash g3-who-records-time.sh <仓根>
set -uo pipefail
cd "$1" || exit 1
for f in .claude/main-agent.md .claude/rules/three-way-inference.md .claude/rules/implementation-workflow.md .claude/agents/three-way-verifier.md .claude/agents/three-way-materials.md; do
  printf '%s\t「派腿的时刻」%s 处\t「腿开工时刻」%s 处\n' "$f" "$(grep -o '派腿的时刻' "$f" | wc -l)" "$(grep -o '腿开工时刻' "$f" | wc -l)"
done
echo "要求记时刻的那一节的标题："
grep -n '^## 代码轮派腿之前记一份开工快照' .claude/rules/implementation-workflow.md
echo "这一轮（governance-defs-r3）的快照目录里有什么："
ls research/prompts/governance-defs-r3-snapshot/
echo "上一轮核查员用的开工时刻、攻方腿自记的开工时刻、正推腿报告的提交时刻："
grep -o '腿开工时刻 2026-09-26T15:30:00Z 前后' research/prompts/governance-defs-r2-verifier-output.md
grep -o '开工 2026-09-26 15:15 UTC' research/prompts/governance-defs-r2-opus-output.md
git log -1 --format='%h %cI %s' 107b79f

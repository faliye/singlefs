#!/usr/bin/env bash
# batch-scope 格本该判绿：三个触发文件分三种形态出现（工作区改、暂存新增、未跟踪），三个都登记了、理由都写了；
# 另有四个不是触发文件的路径（kb、records、.claude/batch-scope 自己与同步记录）不用登记；注释行、空行、路径里的中文都不许把表读坏。
# 触发文件清单是真表的一个子集，够跑这几条就行——这份夹具验的是这一格的判定，不是真表的内容。
# 同一个阶段的另外几格在这份样本上不许红，所以另补两样：
#   knowledge-sync 格要一份点名这三个触发文件的同步记录（research/prompts/demo-sync.md，未跟踪），它点名的原始材料随基准提交；
#   agent-def-adversarial-review 格要改过的定义有去向：豁免表按改后那一版的哈希豁免 .claude/agents/demo.md，随基准提交。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/gate.d .claude/agents .claude/kb records research/scripts research/prompts
cat > .claude/gate.d/knowledge-sync-triggers.tsv <<'EOF'
# 子集：够跑这份夹具的几条
^\.claude/agents/	# agent 定义
^\.claude/gate\.d/[^/]+\.(sh|py|tsv)$	# 门禁阶段与它们读的数据表
^research/scripts/	# 研究与协作脚本
EOF
printf '# demo\n\n第一版\n' > .claude/agents/demo.md
printf '# 旧记录\n' > records/2026-09-16-拆分提案.md
printf '.claude/agents/demo.md\t%s\t样本：这份样本验 batch-scope 格，定义的三方不归它\n' \
  "$(printf '# demo\n\n第二版\n' | sha256sum | cut -d' ' -f1)" > .claude/agent-def-review-exempt
printf '甲\t改了\n乙\t不改\n' > research/prompts/demo-facts.tsv
git add -A && git commit -qm base
printf '# demo\n\n第二版\n' > .claude/agents/demo.md
printf 'print("demo")\n' > research/scripts/demo.py
git add research/scripts/demo.py
printf '# 门禁\nexit 0\n' > .claude/gate.d/77-样本阶段.sh
printf '# 坑\n' > .claude/kb/pitfalls.md
printf '# 记录\n\n改了一行\n' > records/2026-09-16-拆分提案.md
cat > .claude/batch-scope <<'EOF'
# 这一批要碰的触发文件，一行一条：<路径>	#<为什么这一批要碰它>

.claude/agents/demo.md	# 这一批改的就是它的定义
research/scripts/demo.py	# 新建：这一批要的那段脚本
.claude/gate.d/77-样本阶段.sh	# 新建：这一批立的那道闸，中文路径也要读得出来
EOF
# knowledge-sync 格要的同步记录（不是触发文件，batch-scope 格不要求登记它）
cat > research/prompts/demo-sync.md <<'EOF'
<!-- knowledge-sync -->
# demo 阶段同步

触发文件：.claude/agents/demo.md、research/scripts/demo.py、.claude/gate.d/77-样本阶段.sh

## 搜索

grep -rn "第一版" .claude records → 0

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 事实表 | research/prompts/demo-facts.tsv | 2 |

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
EOF

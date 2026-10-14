#!/usr/bin/env bash
# 本该判绿：基准提交里已有一份旧同步记录，它的「改了」「补了」两行点名的载体（records/untouched.md）是更早一轮改的，这一轮没动；
# 这一轮往旧记录末尾只追加了一行「不改：」（载体 records/notes.md 存在），旧记录因此进了改动范围、要被判，但旧行不是这一轮写的，不判载体在不在改动范围里。
# 触发文件 .claude/hooks/demo.sh 是这一轮新建的，由这一轮新写的记录点名，让这一格有对象可判。
# 弄坏开关 KNOWLEDGE_SYNC_BREAK=old-rows-rejudged 把旧行也当这一轮写的，两行旧处置都被报「载体不在改动范围里」。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/gate.d .claude/hooks records research/prompts
printf '#!/usr/bin/env bash\nexit 0\n' > .claude/hooks/old.sh
printf '# 记录\n\n第二行\n第三行\n' > records/notes.md
printf '# 没动的记录\n' > records/untouched.md
# 触发文件清单放在基准提交里（它自己也是触发文件，留在改动范围里会多算一个）。
cat > .claude/gate.d/knowledge-sync-triggers.tsv <<'TRIGGERS'
# 夹具用的清单，与真表同形
^\.claude/agents/	# agent 定义
^\.claude/agent-common\.md$	# agent 的共用约束
^\.claude/hooks/	# 钩子
^\.claude/settings\.json$	# 钩子的注册处
^\.claude/gate\.d/[^/]+\.(sh|py|tsv)$	# 门禁阶段与它们读的数据表
^\.claude/rules/	# 项目本地规则
^scripts/	# 研究与协作脚本
^crates/[^/]+/src/	# 实现
^CLAUDE\.md$	# 项目说明
TRIGGERS
cat > research/prompts/old-sync.md <<'OLD'
<!-- knowledge-sync -->
# old 阶段同步

触发文件：.claude/hooks/old.sh

## 搜索

grep -rn "没动" . → 1

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| records/untouched.md:1 | 没动的记录 | 改了：那一轮改的，载体今天不在改动范围里 |
| records/untouched.md:1 | 没动的记录 | 补了：同样是那一轮补的，载体今天不在改动范围里 |
OLD
git add -A && git commit -qm base
# 这一轮：旧记录末尾追加一行「不改：」；新建一个触发文件，由这一轮新写的记录点名
printf '| records/notes.md:3 | 第二行 | 不改：说的是那一次，冻结证据 |\n' >> research/prompts/old-sync.md
printf '#!/usr/bin/env bash\nexit 0\n' > .claude/hooks/demo.sh
printf '甲\n乙\n丙\n' > research/prompts/demo-candidates.tsv
cat > research/prompts/new-sync.md <<'EOF'
<!-- knowledge-sync -->
# new 阶段同步

触发文件：.claude/hooks/demo.sh

## 搜索

grep -rn "demo.sh" records .claude → 0

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 候选表 | research/prompts/demo-candidates.tsv | 3 |

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
EOF

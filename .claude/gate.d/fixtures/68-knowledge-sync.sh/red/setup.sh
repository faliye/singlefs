#!/usr/bin/env bash
# 本该判红，一份样本同时放十四种坏法：crates 源码没被任何记录点名；hook 只在不带标记的 *-sync.md 里点名；
# 处置写「改了」而载体没动；处置开头不合法；「不改：」没理由；载体没行号；载体指不到文件；
# 一份记录缺「## 搜索」且命中处置的表头写错。agent 定义被点名了，不该出现在漏点名清单里。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/hooks crates/demo/src records research/prompts
printf '# demo\n\n还没用过。\n' > .claude/agents/demo.md
printf '#!/usr/bin/env bash\nexit 0\n' > .claude/hooks/demo.sh
printf 'pub fn publish() -> u32 { 1 }\n' > crates/demo/src/lib.rs
printf '# 记录\n\n第二行\n第三行\n' > records/notes.md
printf '# 没动的记录\n' > records/untouched.md
git add -A && git commit -qm base
printf '# demo\n\n用过了。\n' > .claude/agents/demo.md
printf '#!/usr/bin/env bash\nexit 1\n' > .claude/hooks/demo.sh
printf 'pub fn publish() -> u32 { 2 }\n' > crates/demo/src/lib.rs
printf '# 记录\n\n第二行改了\n第三行\n' > records/notes.md
cat > research/prompts/nomarker-sync.md <<'EOF'
# 没有标记的同步记录

触发文件：.claude/hooks/demo.sh
EOF
cat > research/prompts/bad-sync.md <<'EOF'
<!-- knowledge-sync -->
# bad 阶段同步

触发文件：.claude/agents/demo.md

## 搜索

grep -rn "没用过" . → 5

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| records/untouched.md:1 | 没动的记录 | 改了：改成现状 |
| records/notes.md:3 | 第二行 | 看过了，没问题 |
| records/notes.md:4 | 第三行 | 不改： |
| notes.md | 第二行 | 不改：说的是那一次 |
| records/gone.md:4 | 不存在的 | 不改：冻结证据 |
EOF
cat > research/prompts/nosearch-sync.md <<'EOF'
<!-- knowledge-sync -->
# nosearch 阶段同步

触发文件：.claude/agents/demo.md

## 命中处置

| 位置 | 原句 | 处置 |
|---|---|---|
EOF

# ⑤ 原始证据的四种坏法，各一份记录：表头写错、表一行都没有、点名的材料不在仓里、行数与 wc -l 对不上。
# （bad-sync.md 与 nosearch-sync.md 整节都没有，那是第五种。）
printf '甲\n乙\n丙\n' > research/prompts/demo-candidates.tsv
cat > research/prompts/evidence-header-sync.md <<'EOF'
<!-- knowledge-sync -->
# 表头写错的阶段同步

触发文件：.claude/agents/demo.md

## 搜索

grep -rn "没用过" . → 5

## 原始证据

| 材料 | 文件 | 行数 |
|---|---|---|
| 候选表 | research/prompts/demo-candidates.tsv | 3 |

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
EOF
cat > research/prompts/evidence-empty-sync.md <<'EOF'
<!-- knowledge-sync -->
# 表一行都没有的阶段同步

触发文件：.claude/agents/demo.md

## 搜索

grep -rn "没用过" . → 5

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
EOF
cat > research/prompts/evidence-missing-sync.md <<'EOF'
<!-- knowledge-sync -->
# 材料不在仓里的阶段同步

触发文件：.claude/agents/demo.md

## 搜索

grep -rn "没用过" . → 5

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 候选表 | research/prompts/demo-gone-candidates.tsv | 3 |

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
EOF
cat > research/prompts/evidence-count-sync.md <<'EOF'
<!-- knowledge-sync -->
# 行数对不上的阶段同步

触发文件：.claude/agents/demo.md

## 搜索

grep -rn "没用过" . → 5

## 原始证据

| 材料 | 路径 | 行数 |
|---|---|---|
| 候选表 | research/prompts/demo-candidates.tsv | 138 |

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
EOF

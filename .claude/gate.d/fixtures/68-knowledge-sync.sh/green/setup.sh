#!/usr/bin/env bash
# 三个触发文件（agent 定义工作区改动、hook 暂存新增、research 脚本未跟踪）分在两份同步记录里点名全了；
# 第一份的命中处置表三种处置各一行（改了、补了的载体都在改动范围里，「改了」的载体是中文文件名，不改的载体没动；原句里带转义的 \|，表后代码围栏里有一行竖线行），
# 第二份表 0 行 ⇒ 本该判绿并报对数。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/hooks .claude/kb records research/prompts research/scripts
printf '# demo\n\n还没在真活里用过。\n' > .claude/agents/demo.md
printf '# 记录\n\n看门狗还没用过。\n' > records/2026-09-16-拆分提案.md
printf '# 坑\n' > .claude/kb/pitfalls.md
printf '# 旧判决\n\n第一行 | 带竖线\n' > research/prompts/old-r1-main-verification.md
git add -A && git commit -qm base
printf '# demo\n\n在 E153 派发里用过。\n' > .claude/agents/demo.md
printf '#!/usr/bin/env bash\nexit 0\n' > .claude/hooks/demo.sh
git add .claude/hooks/demo.sh
printf 'print("demo")\n' > research/scripts/demo.py
printf '# 记录\n\n看门狗在 E153 里用过。\n' > records/2026-09-16-拆分提案.md
printf '# 坑\n\n- 用过之后回头改「没用过」的句子。\n' > .claude/kb/pitfalls.md
cat > research/prompts/demo-sync.md <<'EOF'
<!-- knowledge-sync -->
# demo 阶段同步

触发文件：.claude/agents/demo.md、.claude/hooks/demo.sh

## 搜索

```bash
# 代码围栏里的井号行不算标题
grep -rn "没用过" .claude records research | wc -l
```
→ 2

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
| `records/2026-09-16-拆分提案.md:3` | 看门狗还没用过。 | 改了：E153 里用过，改成现状 |
| .claude/kb/pitfalls.md:3 | （新增） | 补了：用过之后回头改的坑 |
| research/prompts/old-r1-main-verification.md:3 | 第一行 \| 带竖线 | 不改：冻结证据，说的是那一次 |

复核时对过的原文：

```text
| 代码围栏里的竖线行不算第二张表 |
```
EOF
cat > research/prompts/second-sync.md <<'EOF'
<!-- knowledge-sync -->
# 第二批阶段同步

触发文件：research/scripts/demo.py

## 搜索

grep -rn "demo.py" records .claude → 0

## 命中处置

| 载体 | 原句 | 处置 |
|---|---|---|
EOF

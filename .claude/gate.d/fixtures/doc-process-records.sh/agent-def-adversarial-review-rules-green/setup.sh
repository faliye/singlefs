#!/usr/bin/env bash
# 改了一份项目规则（.claude/rules/ 顶层的 .md），另改了同目录里的一份 .txt（不归这一格管）；新写的判决在「## 这一轮被判过的文件」一节里点名了规则那一份 ⇒ 本该判绿、报项目规则 1 份。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/rules .claude/gate.d research/prompts
printf '# 规则\n\n- 只写现状。\n' > .claude/rules/demo.md
printf '草稿\n' > .claude/rules/notes.txt
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '# 规则\n\n- 只写现状与判据。\n' > .claude/rules/demo.md
printf '草稿二\n' > .claude/rules/notes.txt
cat > research/prompts/demo-r1-main-verification.md <<'MD'
# demo 第一轮：主 agent 核实

三条腿都没打中。

## 这一轮被判过的文件

- .claude/rules/demo.md
MD

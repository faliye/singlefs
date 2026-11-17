#!/usr/bin/env bash
# 改了一份项目规则（.claude/rules/ 顶层的 .md），另改了同目录里的一份 .txt（不是 .md，不归这一格管），没有判决 ⇒ 规则那一份没人点名，判红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/rules .claude/gate.d
printf '# 规则\n\n- 只写现状。\n' > .claude/rules/demo.md
printf '草稿\n' > .claude/rules/notes.txt
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '# 规则\n\n- 只写现状与判据。\n' > .claude/rules/demo.md
printf '草稿二\n' > .claude/rules/notes.txt

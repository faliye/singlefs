#!/usr/bin/env bash
# 改了一份技能（.claude/skills/ 下的 SKILL.md），另改了同目录里的一份 .txt（不是 .md，不归这一格管），没有判决 ⇒ 技能那一份没人点名，判红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/skills/demo .claude/gate.d
printf '# 技能\n\n1. 先读规则。\n' > .claude/skills/demo/SKILL.md
printf '草稿\n' > .claude/skills/demo/helper.txt
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '# 技能\n\n1. 先读规则，再读判据。\n' > .claude/skills/demo/SKILL.md
printf '草稿二\n' > .claude/skills/demo/helper.txt

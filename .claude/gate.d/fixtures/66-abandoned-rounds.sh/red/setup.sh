#!/usr/bin/env bash
# 建一个 git 仓，再删掉第一个提交的根树对象：git rev-parse 照样认得出这是仓，git log --diff-filter=D 却读不到
# ⇒ 归档进版本库的轮次读不到，本阶段要判红，不许退回只看树（那样归档的轮次整批消失）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
printf '# 已归档的一轮\n' > research/prompts/_archived-r1-body.md
git add -A && git commit -qm base
git rm -q research/prompts/_archived-r1-body.md && git commit -qm 归档
first_tree="$(git rev-parse HEAD~1^{tree})"
rm -f ".git/objects/${first_tree:0:2}/${first_tree:2}"

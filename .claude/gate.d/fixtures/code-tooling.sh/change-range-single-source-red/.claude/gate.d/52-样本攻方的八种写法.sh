#!/usr/bin/env bash
# 红样本：第二轮攻方给的漏判写法，每行一种
changed="$(git -c core.quotepath=false diff --name-only @{u}... --)" || exit 1
recent="$(git -c core.quotepath=false diff --name-only HEAD~1 --)" || exit 1
upstream_commit="$(git rev-parse @{u})" || exit 1
git -c core.quotepath=true ls-files --others --exclude-standard -- .claude/kb
[[ -z "${SKIP:-}" ]] && git ls-files --others --exclude-standard -- research > "$list"
printf '%s\n' "$(git ls-files --others --exclude-standard -- .claude/kb)" > "$list"

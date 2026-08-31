#!/usr/bin/env bash
# 造一段可控历史：先提交两条决策，再单独提交「乙样本新增了一个已定小节」。
# 于是被点名那条的状态行比点名它的未定项**更新** ⇒ 本该判红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
GIT_COMMITTER_DATE="2026-01-01T00:00:00" GIT_AUTHOR_DATE="2026-01-01T00:00:00" sh -c 'git add -A && git commit -qm base'
printf '\n### 已定项 1 —— 已定（2026-01-02）：取甲\n\n依据。\n' >> .claude/kb/decisions/96-样本.md
GIT_COMMITTER_DATE="2026-01-02T00:00:00" GIT_AUTHOR_DATE="2026-01-02T00:00:00" sh -c 'git add -A && git commit -qm settle'

#!/usr/bin/env bash
# 造一段可控历史：先提交决策 D95（未定项点名 D96）与 D96 的中文原文和一份译文，再单独提交「D96 新增了一个已定小节」⇒ D95 那条未定项本该判红。
# 译文 96-样本.zz.md 的首行也写 `## D96`，没有「—— 已定」标题；文件名排在中文原文之后（不论按字节排还是按 en_US.UTF-8 排）。
# 弄坏开关 DOC_DECISIONS_BREAK=stale-open-items-translations-read 连译文一起记每条决策的定案时间：译文后读、把中文原文那一条盖成 0，D95 那一条不比，这份样本判不红。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
GIT_COMMITTER_DATE="2026-08-31T00:00:00" GIT_AUTHOR_DATE="2026-08-31T00:00:00" sh -c 'git add -A && git commit -qm base'
printf '\n### 已定项 1 —— 已定（2026-09-01）：取甲\n\n依据。\n' >> .claude/kb/decisions/96-样本.md
GIT_COMMITTER_DATE="2026-09-01T00:00:00" GIT_AUTHOR_DATE="2026-09-01T00:00:00" sh -c 'git add -A && git commit -qm settle'

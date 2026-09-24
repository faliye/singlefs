#!/usr/bin/env bash
# 先提交一份带未定项的决策、把上游设在这个提交，再**另提交一次、不推**：加一个「已定」小节而不碰那条未定项；
# 工作区另改一个无关文件 ⇒ 本该判红：定了新东西却没回头看同文件还开着的那条。
# 定案已经提交在本地而没推，正是「在默认分支上 merge-base 就是 HEAD」那种基准会漏掉的形状（三方判决 gate-fix-forks-r1 的 T5）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master . && git add -A && git commit -qm base
git branch 上游 && git branch -q --set-upstream-to=上游 master
printf '\n### 已定项 2 —— 已定（2026-09-01）：取甲\n\n依据。\n' >> .claude/kb/decisions/97-样本.md
git add -A && git commit -qm '定案，没推'
printf '无关的改动\n' > 无关.md

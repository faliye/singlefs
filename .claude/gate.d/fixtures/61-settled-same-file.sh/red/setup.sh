#!/usr/bin/env bash
# 先提交一份带未定项的决策，再在**工作区**加一个「已定」小节而不碰那条未定项
# ⇒ 本该判红：定了新东西却没回头看同文件还开着的那条。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master . && git add -A && git commit -qm base
printf '\n### 已定项 2 —— 已定（2026-01-02）：取甲\n\n依据。\n' >> .claude/kb/decisions/97-样本.md

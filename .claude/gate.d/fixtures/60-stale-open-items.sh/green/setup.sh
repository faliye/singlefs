#!/usr/bin/env bash
# 同一段历史，但第二次提交**同时**回头改了那条点名它的未定项（补一句复核记录）
# ⇒ 未定项比对方的状态行更新，本该判绿。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
GIT_COMMITTER_DATE="2026-01-01T00:00:00" GIT_AUTHOR_DATE="2026-01-01T00:00:00" sh -c 'git add -A && git commit -qm base'
printf '\n### 已定项 1 —— 已定（2026-01-02）：取甲\n\n依据。\n' >> .claude/kb/decisions/99-样本.md
sed -i 's/那边不定它就定不了/那边不定它就定不了（2026-01-02 复核过，仍然开着）/' .claude/kb/decisions/98-样本.md
GIT_COMMITTER_DATE="2026-01-02T00:00:00" GIT_AUTHOR_DATE="2026-01-02T00:00:00" sh -c 'git add -A && git commit -qm settle'

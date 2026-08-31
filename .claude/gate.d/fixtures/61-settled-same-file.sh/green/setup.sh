#!/usr/bin/env bash
# 同样加一个「已定」小节，但**同时**回头在那条未定项上补了一句复核 ⇒ 本该判绿。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master . && git add -A && git commit -qm base
printf '\n### 已定项 2 —— 已定（2026-01-02）：取甲\n\n依据。\n' >> .claude/kb/decisions/100-样本.md
sed -i 's/两条出路都还没选/两条出路都还没选（2026-01-02 复核过，仍然开着）/' .claude/kb/decisions/100-样本.md

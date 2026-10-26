#!/usr/bin/env bash
# 基准里一份记录合法（28 字节三行）。这一轮：已跟踪的记录第 4 行是「- 加一行：」（14 字节）接一个被拦腰切断的汉字（「中」是 e4 b8 ad，只剩 e4 b8，后面接换行），
# 坏字节从偏移 28 + 14 = 42 起；未跟踪的 .claude/kb/坏页.md 第 1 行开头有孤立的 0xff（偏移 0，后面三个字节是「未」的 e6 9c aa）；另有一份合法的 .claude/kb/好页.md。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records .claude/kb
printf '# 样本调度\n\n- 起点。\n' > records/2026-09-24-样本调度.md
git add -A && git commit -qm base
printf '%s' '- 加一行：' >> records/2026-09-24-样本调度.md
printf '\xe4\xb8\n' >> records/2026-09-24-样本调度.md
printf '\xff未跟踪页的第一行\n第二行\n' > .claude/kb/坏页.md
printf '# 好页\n\n全部合法。\n' > .claude/kb/好页.md

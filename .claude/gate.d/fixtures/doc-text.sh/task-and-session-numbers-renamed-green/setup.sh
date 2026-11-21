#!/usr/bin/env bash
# 基准里一份记录带一行旧任务号、一份 kb 页带一行旧批号；这一轮把两份 git mv 到新名字：记录内容一字不改，kb 页末尾再加一行干净的话。
# 搬家、改名过来的两份按基准里的旧路径取基准（git diff -M 的改名对），旧号不报，判绿（成功句报放过 2 行、按旧路径取基准 2 份）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records .claude/kb
cat > records/2026-09-24-旧名.md <<'X'
# 样本调度（2026-09-24）

- 旧行：调度那一条跑完（任务 21036），基准里就有。
- 第二件：配置判法那一份补丁交回。
- 第三件：门禁那一格换上之后复跑。
- 第四件：两份记录对过一遍。
X
cat > .claude/kb/old-name.md <<'X'
# kb 页

- 它的批 105 在修，基准里就有。
- 第二条：判据库的文件头写全。
- 第三条：样本跟着搬。
- 第四条：搬完全仓改路径。
X
git add -A && git commit -qm base
git mv records/2026-09-24-旧名.md records/2026-09-26-新名.md
git mv .claude/kb/old-name.md .claude/kb/new-name.md
printf '%s\n' '- 一句干净的新话。' >> .claude/kb/new-name.md

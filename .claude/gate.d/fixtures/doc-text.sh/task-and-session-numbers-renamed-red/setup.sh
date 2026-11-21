#!/usr/bin/env bash
# 基准里一份记录带一行旧任务号；这一轮把它 git mv 到新名字，再在末尾加一行带任务号的：按基准里的旧路径取基准，旧号那一行不报，
# 只报改名之后新加的那一行，判红（报 1 行；改名不认、基准按此刻的路径取空串的话旧号那一行也报，成了 2 行）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p records
cat > records/2026-09-24-旧名.md <<'X'
# 样本调度（2026-09-24）

- 旧行：调度那一条跑完（任务 21036），基准里就有。
- 第二件：配置判法那一份补丁交回。
- 第三件：门禁那一格换上之后复跑。
- 第四件：两份记录对过一遍。
X
git add -A && git commit -qm base
git mv records/2026-09-24-旧名.md records/2026-09-26-新名.md
printf '%s\n' '- 新加的一行：复跑那一条跑完（任务 21037）。' >> records/2026-09-26-新名.md

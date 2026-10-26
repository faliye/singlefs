#!/usr/bin/env bash
# 没有第二台私有名单（替身 .out-of-project-private-names 不在）：别的来源照判，新加的一行干净，判绿并报第二台本次未查。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/.out-of-project-*' '/setup.sh' '/expect' > .gitignore   # 替身与样本自己的文件不进被判的改动，也不算 git 跟踪的脚本
mkdir -p records
printf '# 样本空间\n\n- 本项目的目录占用照常写。\n' > records/2026-10-21-样本空间.md
git add -A && git commit -qm base
printf '%s\n' '- 第二台跑了一半。' >> records/2026-10-21-样本空间.md

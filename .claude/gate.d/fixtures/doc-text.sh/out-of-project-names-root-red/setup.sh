#!/usr/bin/env bash
# 根下一层的系统目录（替身 .out-of-project-root 里的假目录名）：第 4 行反引号里的 /<名> 加容量、第 5 行括号里的容量与带收尾 / 的路径加容量，判红；
# 第 6 行 /<名>/log/… 不带容量、第 7 行本项目目录的容量，不报。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/.out-of-project-*' '/setup.sh' '/expect' > .gitignore   # 替身与样本自己的文件不进被判的改动，也不算 git 跟踪的脚本
mkdir -p records
printf '# 样本空间\n\n- 本项目的目录占用照常写。\n' > records/2026-10-21-样本空间.md
git add -A && git commit -qm base
cat >> records/2026-10-21-样本空间.md <<'Y'
- 根分区：`/sample-var` 50G。
- /sample-usr（12G）、/sample-opt/ 3.5 GiB。
- /sample-var/log/syslog 不带容量，不判。
- 本项目的 research/results 占 3G。
Y

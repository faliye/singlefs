#!/usr/bin/env bash
# 第二台私有名单（替身 .out-of-project-private-names）里的假名字：第 4 行路径段、第 5 行反引号，判红；第 6 行只说「第二台」，不报。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/.out-of-project-*' '/setup.sh' '/expect' > .gitignore   # 替身与样本自己的文件不进被判的改动，也不算 git 跟踪的脚本
mkdir -p records
printf '# 样本空间\n\n- 本项目的目录占用照常写。\n' > records/2026-10-21-样本空间.md
git add -A && git commit -qm base
cat >> records/2026-10-21-样本空间.md <<'Y'
- 第二台上的 ~/sample-peer-cache/blobs 不写。
- 也不写 `sample-peer-cache`。
- 第二台跑了一半。
Y

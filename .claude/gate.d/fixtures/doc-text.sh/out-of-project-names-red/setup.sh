#!/usr/bin/env bash
# 名字类各种上下文各一行（替身 .out-of-project-home / .out-of-project-parent 里的假条目名）：第 4 行反引号加容量、第 5 行只有反引号、第 6 行不带反引号加容量、
# 第 7、8 行家目录路径段（~/、$HOME/、/home/<用户>/）、第 9 行上一级目录路径段（../、~/<上一级>/）；第 10 行只说本项目的目录与总可用量，不报。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/.out-of-project-*' '/setup.sh' '/expect' > .gitignore   # 替身与样本自己的文件不进被判的改动，也不算 git 跟踪的脚本
mkdir -p records
printf '# 样本空间\n\n- 本项目的目录占用照常写。\n' > records/2026-10-21-样本空间.md
git add -A && git commit -qm base
cat >> records/2026-10-21-样本空间.md <<'Y'
- 根分区其余大头（`sample-workspace` 419G、`sample-kbuild`（82G））。
- 只写名字：`sample-workspace`。
- 不带反引号：sample-kbuild 80G 也算。
- 家目录路径：~/sample-workspace/data 与 $HOME/sample-kbuild/tree。
- 绝对路径：/home/sample-user/toolbox/bin。
- 仓的上一级：../sample-refs/docs 与 ~/sample-code/sample-other-project。
- 干净的一行：本项目的 research/results 占 3G，根分区总可用 1.4T。
Y

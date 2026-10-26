#!/usr/bin/env bash
# 放行表登记 ~/sample-kbuild/tree（scripts/fetch.sh 引到）：第 4、5 行这条路径的几种写法放过；第 6 行同名单独出现加容量、第 7 行同一目录下别的路径、
# 第 8 行多一个字的 tree2 都不在放行范围里，判红。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/.out-of-project-*' '/setup.sh' '/expect' > .gitignore   # 替身与样本自己的文件不进被判的改动，也不算 git 跟踪的脚本
mkdir -p scripts .claude
printf '%s\n' '#!/usr/bin/env bash' 'tree="${SAMPLE_TREE:-$HOME/sample-kbuild/tree}"' 'ls "$tree"' > scripts/fetch.sh
printf '%s\n' '# 样本放行表' '~/sample-kbuild/tree  # 样本：scripts/fetch.sh 读这棵假的内核树' > .claude/out-of-project-paths
mkdir -p records
printf '# 样本空间\n\n- 本项目的目录占用照常写。\n' > records/2026-10-21-样本空间.md
git add -A && git commit -qm base
cat >> records/2026-10-21-样本空间.md <<'Y'
- 读 ~/sample-kbuild/tree/arch 下的文件。
- 同一条路径写成 $HOME/sample-kbuild/tree、/home/sample-user/sample-kbuild/tree 也放过。
- 但 `sample-kbuild` 80G 照红。
- ~/sample-kbuild/other 也红。
- ~/sample-kbuild/tree2 不是放行的那条，也红。
Y

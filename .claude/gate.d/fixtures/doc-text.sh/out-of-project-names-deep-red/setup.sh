#!/usr/bin/env bash
# 深层段放过之后照红的（替身家目录里有小写词 samplebin 与 .local）：第 4 行这个名字单独带容量、第 5 行 ~/<名>、第 7 行 /home/<用户>/<名>、第 8 行 ~/<上一级>/<名> 带容量
# （第一段，照判带容量）；第 6 行 ~/<家目录条目名>/sub/<名>、第 9 行 ../<上一级条目名>/docs/<名> 带容量（深层段，不判带容量；第一段不归本项目，由路径段判红）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/.out-of-project-*' '/setup.sh' '/expect' > .gitignore   # 替身与样本自己的文件不进被判的改动，也不算 git 跟踪的脚本
mkdir -p scripts .claude records
printf '%s\n' '#!/usr/bin/env bash' 'work="$HOME/.local/share/singlefs/work"' 'ls "$work"' > scripts/work.sh
printf '%s\n' '# 样本放行表' '~/.local/share/singlefs  # 样本：scripts/work.sh 在这里放草稿目录' > .claude/out-of-project-paths
printf '# 样本空间\n\n- 本项目的目录占用照常写。\n' > records/2026-10-21-样本空间.md
git add -A && git commit -qm base
cat >> records/2026-10-21-样本空间.md <<'Y'
- `samplebin` 80G 单独写也算。
- `~/samplebin` 80G。
- `~/sample-workspace/sub/samplebin`：1.9G。
- /home/sample-user/samplebin（2G）。
- ~/sample-code/samplebin 4G。
- ../sample-refs/docs/samplebin 5M。
Y

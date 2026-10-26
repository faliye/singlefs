#!/usr/bin/env bash
# 路径的深层段带容量（替身家目录里有小写词 samplebin 与 .local）：第 4 行放行路径 ~/.local/share/singlefs 底下的 …/samplebin、第 5 行 singlefs 自己的路径底下的 …/samplebin、
# 第 6 行 ../singlefs… 底下的 …/samplebin、第 7 行根下系统目录底下的 …/samplebin，都带容量；深层段不按名字类判，第一段归本项目或是放行路径，判绿。
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
- `~/.local/share/singlefs/work/sample-round-defense/samplebin`：1.9G（各份探针二进制）。
- ~/singlefs-sample-runs/out/samplebin 3G 是本项目自己跑的。
- ../singlefs-ai-sop-zh/tools/samplebin（5M）在上游副本里。
- /sample-var/cache/samplebin 2G 是根下系统目录底下的深层段。
Y

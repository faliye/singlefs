#!/usr/bin/env bash
# 基准里的旧行带假名字加容量（只判新加的行，不报）；新加的行：放行路径的三种写法、singlefs 自己的路径与名字、正文里当普通词用的假名字、
# 仓里自己也有的顶层名（.sample-tool）在反引号里与带容量、根下系统目录不带容量的路径、只说本项目目录与总可用量的一句，判绿。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
printf '%s\n' '/.out-of-project-*' '/setup.sh' '/expect' > .gitignore   # 替身与样本自己的文件不进被判的改动，也不算 git 跟踪的脚本
mkdir -p scripts .claude
printf '%s\n' '#!/usr/bin/env bash' 'tree="${SAMPLE_TREE:-$HOME/sample-kbuild/tree}"' 'ls "$tree"' > scripts/fetch.sh
printf '%s\n' '# 样本放行表' '~/sample-kbuild/tree  # 样本：scripts/fetch.sh 读这棵假的内核树' > .claude/out-of-project-paths
mkdir -p records .sample-tool
printf '%s\n' '仓里自己的工具目录。' > .sample-tool/README.md
printf '# 样本空间\n\n- 旧行：`sample-workspace` 419G，早就写在这里了。\n' > records/2026-10-21-样本空间.md
git add -A && git commit -qm base
cat >> records/2026-10-21-样本空间.md <<'Y'
- 读 ~/sample-kbuild/tree/arch、$HOME/sample-kbuild/tree 与 /home/sample-user/sample-kbuild/tree 下的文件。
- 仓在 ~/sample-code/singlefs，上游副本在 /home/sample-user/sample-code/singlefs-ai-sop-zh 与 ../singlefs-ai-sop-zh，跑的记录在 ~/singlefs-sample-runs。
- `singlefs` 与 `singlefs-ai-sop-zh` 是本项目自己的。
- toolbox 与 datasets 在正文里当普通词用，不判。
- `.sample-tool` 是仓里自己的目录，`.sample-tool` 3G 也不判。
- /sample-var/log/syslog 与 /sample-usr/bin/env 不带容量，不判。
- 根分区其余不归 singlefs，总可用 1.4T；本项目的 research/results 占 3G。
Y

#!/usr/bin/env bash
# 本该判红：git 配置里的 diff.renames 写成 git 认不得的值，取这次改动碰了哪些路径的 git diff 一律失败。
# 取改动路径是阶段在父进程里预取的，红行之前要有 gate_changed_paths 写的哪一条 git、退几与 git 的报错原文
# （原文里的 diff.renames 是配置项名字，不随 git 的界面语言变）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/gate.d scripts
cat > .claude/gate.d/knowledge-sync-triggers.tsv <<'EOF'
# 子集：够跑这份夹具的几条
^scripts/	# 研究与协作脚本
EOF
printf 'print("old")\n' > scripts/kept.py
git add -A && git commit -qm base
printf 'print("new")\n' > scripts/kept.py
git config diff.renames foo

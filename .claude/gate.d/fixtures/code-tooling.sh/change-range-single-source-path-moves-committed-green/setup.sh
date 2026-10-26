#!/usr/bin/env bash
# 生效窗口那一支（这一批提交之后）：建 git 仓，先提交 scripts/old-rig 下一份脚本与点名它的说明；再 git mv 到 scripts/new-rig、说明里的路径一起换、
# 登记 scripts/old-rig → scripts/new-rig，连同登记表一起提交：基准里已经是新前缀，这一行不生效，实据不核，判绿。只起 git 与 printf。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/old-rig docs .claude/gate.d
printf 'echo 跑\n' > scripts/old-rig/run.sh
printf '复现：bash scripts/old-rig/run.sh\n' > docs/howto.md
git add -A scripts docs records
git commit -qm 更早
git mv scripts/old-rig scripts/new-rig
printf '复现：bash scripts/new-rig/run.sh\n' > docs/howto.md
printf '%s\n' '# 样本：搬迁登记' 'scripts/old-rig	scripts/new-rig	2026-10-01	records/样本搬迁.md' > .claude/gate.d/path-moves.tsv
git add -A scripts docs .claude
git commit -qm 搬迁那一次

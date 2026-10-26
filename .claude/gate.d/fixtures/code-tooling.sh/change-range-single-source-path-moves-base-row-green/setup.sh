#!/usr/bin/env bash
# 基准那一版登记表里已有的行不再生效（睡着的老行不醒来）：建 git 仓，scripts/old-rig 搬到 scripts/new-rig、登记，连同登记表一起提交；
# 之后 scripts/new-rig 又改名 scripts/newer-rig，旧名字 scripts/old-rig 下另起一份 fresh.sh，再提交一次。基准是最后这一次提交：基准树满足
# 「旧前缀下有文件、新前缀下没有文件」，可那一行已在基准那一版的登记表里，不生效：判绿，成功句写此刻生效的 0 行、点名「已在基准 HEAD 那一版的登记表里」。
# 只起 git 与 printf。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p scripts/old-rig .claude/gate.d
printf 'echo 跑\n' > scripts/old-rig/run.sh
git add -A scripts records
git commit -qm 更早
git mv scripts/old-rig scripts/new-rig
printf '%s\n' '# 样本：搬迁登记' 'scripts/old-rig	scripts/new-rig	2026-10-01	records/样本搬迁.md' > .claude/gate.d/path-moves.tsv
git add -A scripts .claude
git commit -qm 搬迁那一次
git mv scripts/new-rig scripts/newer-rig
mkdir -p scripts/old-rig
printf 'echo 旧名字上的新东西\n' > scripts/old-rig/fresh.sh
git add -A scripts
git commit -qm 新目录又改了名、旧名字另起一份新东西

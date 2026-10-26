#!/usr/bin/env bash
# 搬迁实据与不许串那一支：建 git 仓，把两份实现 tools/real-a、tools/real-b、起 A 的 ops/launch.sh、old/x/f.sh、old/y/g.sh、rig/run.sh 与写着 build/ 的 .gitignore 提交成基准；
# 再删掉 tools/real-a、ops/launch.sh 改成起 B；old/x/f.sh、old/y/g.sh 搬到 new/ 下，old/x/f.sh 另拷一份 other/f.sh；删掉 rig，build/rig/run.sh 放一份没进 git 的。
# 搬迁登记表五行：tools/real-a → tools/real-b（新前缀下基准里本来就有文件，不生效；又是不在基准登记表里的新行，判红「这一行不是一次搬迁」）、tools/real-a → tools/gone（生效，落点不在）、
# old → new 与 old/x → other（都生效、实据都齐，串了）、rig → build/rig（生效，落点被 .gitignore 忽略）。只起 git、printf、mkdir、rmdir、cp。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p tools/real-a tools/real-b ops old/x old/y rig .claude/gate.d
printf 'echo 走的是 A\n' > tools/real-a/run.sh
printf 'echo 走的是 B\n' > tools/real-b/run.sh
printf 'bash tools/real-a/run.sh\n' > ops/launch.sh
printf 'echo f\n' > old/x/f.sh
printf 'echo g\n' > old/y/g.sh
printf 'echo rig\n' > rig/run.sh
printf 'build/\n' > .gitignore
git add -A tools ops old rig records .gitignore
git commit -qm base
git rm -q -r tools/real-a
printf 'bash tools/real-b/run.sh\n' > ops/launch.sh
mkdir -p new/x new/y other build/rig
git mv old/y/g.sh new/y/g.sh
git mv old/x/f.sh new/x/f.sh
if [[ -d old ]]; then rmdir old/x old/y old; fi
cp new/x/f.sh other/f.sh
git add other/f.sh
git rm -q -r rig
printf 'echo 没进 git 的另一份\n' > build/rig/run.sh
printf '%s\n' '# 样本：五行登记，生效的四行里一行落点不在、两行串了、一行落点被忽略；不生效的一行是新行，判红「不是一次搬迁」' \
  'tools/real-a	tools/real-b	2026-10-01	records/样本搬迁.md' \
  'tools/real-a	tools/gone	2026-10-01	records/样本搬迁.md' \
  'old	new	2026-10-01	records/样本搬迁.md' \
  'old/x	other	2026-10-01	records/样本搬迁.md' \
  'rig	build/rig	2026-10-01	records/样本搬迁.md' > .claude/gate.d/path-moves.tsv

#!/usr/bin/env bash
# batch-scope-stale-red 的同一份登记（两个触发文件登记对了，另有三行悬空的），再照红样本出路里打出的那条删行命令跑一遍：悬空的三行没了、别的行不动，这一格转绿。
# 下面 prune_command 与 batch-scope-stale-red/expect 里钉的那一行逐字相同（改一处要同时改另一处）；要是命令漏了一行，回读断言当场失败，这份样本就判错。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/gate.d .claude/agents scripts
cat > .claude/gate.d/knowledge-sync-triggers.tsv <<'TSV'
# 子集：够跑这份夹具的几条
^\.claude/agents/	# agent 定义
^scripts/	# 研究与协作脚本
TSV
printf '# demo\n\n第一版\n' > .claude/agents/demo.md
printf 'print("old")\n' > scripts/kept.py
git add -A && git commit -qm base
printf '# demo\n\n第二版\n' > .claude/agents/demo.md
printf 'print("new")\n' > scripts/kept.py
printf '%s\n' \
  '# 这一批要碰的触发文件' \
  '' \
  $'.claude/agents/demo.md\t# 这一批改的就是它的定义' \
  $'.claude/agents/gone.md\t# 上一批留下的，这一批根本没碰它' \
  $'scripts/old-batch.py\t# 更早一批留下的' \
  $'.claude/agents/gone.md\t# 同一个路径的第二行' \
  $'scripts/kept.py\t# 这一批改的脚本' > .claude/batch-scope

prune_command=$(cat <<'CMD'
grep -vxF -e $'.claude/agents/gone.md\t# 上一批留下的，这一批根本没碰它' -e $'scripts/old-batch.py\t# 更早一批留下的' -e $'.claude/agents/gone.md\t# 同一个路径的第二行' .claude/batch-scope > .claude/batch-scope.pruned; [ $? -le 1 ] && mv .claude/batch-scope.pruned .claude/batch-scope && ! grep -qxF -e $'.claude/agents/gone.md\t# 上一批留下的，这一批根本没碰它' -e $'scripts/old-batch.py\t# 更早一批留下的' -e $'.claude/agents/gone.md\t# 同一个路径的第二行' .claude/batch-scope && echo '已删 3 行，回读确认它们已不在 .claude/batch-scope 里' || { rm -f .claude/batch-scope.pruned; echo '没删成，请手工核对 .claude/batch-scope'; }
CMD
)
bash -c "$prune_command"
# 回读：悬空的三行都没了，登记对的三行（注释、空行与两个触发文件）原样还在，没留下临时文件
! grep -qF -e 'gone.md' -e 'old-batch.py' .claude/batch-scope
grep -qxF $'.claude/agents/demo.md\t# 这一批改的就是它的定义' .claude/batch-scope
grep -qxF $'scripts/kept.py\t# 这一批改的脚本' .claude/batch-scope
test ! -e .claude/batch-scope.pruned

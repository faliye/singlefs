#!/usr/bin/env bash
# batch-scope 格只剩一种毛病：登记里有悬空的行（上一批留下的、这一批根本没碰的路径）。两个触发文件都登记对了、理由都写了。
# 悬空的三行：.claude/agents/gone.md 登记了两行（第二行理由不同）、scripts/old-batch.py 一行，路径之后都带制表符与理由。
# 红的出路里要打出一条能直接粘贴的删行命令（按整行内容匹配、删完回读确认）：expect 逐字钉那一条；
# batch-scope-stale-pruned-green 的 setup.sh 里照这条命令跑一遍之后这一格转绿，两处的命令逐字相同，改一处要同时改另一处。
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

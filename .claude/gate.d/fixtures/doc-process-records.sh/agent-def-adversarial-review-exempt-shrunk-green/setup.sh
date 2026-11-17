#!/usr/bin/env bash
# 基准里的豁免表有两行（other.md 一行、demo.md 一行，后一行按改后那一版的内容哈希登记）；工作区删了 other.md 那一行，只缩不涨不红，
# demo.md 改了、没有判决点名，按表里留着的那一行豁免 ⇒ 本该判绿、报 1 份豁免。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/gate.d
printf '# demo\n\n1. 跑完贴原样输出。\n' > .claude/agents/demo.md
printf '# other\n\n1. 只读。\n' > .claude/agents/other.md
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf '.claude/agents/other.md\t%s\t存量里的另一行豁免\n.claude/agents/demo.md\t%s\t存量里的一行豁免\n' \
  "$(sha256sum .claude/agents/other.md | cut -d' ' -f1)" \
  "$(printf '# demo\n\n1. 跑完贴原样输出与退出码。\n' | sha256sum | cut -d' ' -f1)" > .claude/agent-def-review-exempt
git add -A && git commit -qm base
printf '# demo\n\n1. 跑完贴原样输出与退出码。\n' > .claude/agents/demo.md
printf '.claude/agents/demo.md\t%s\t存量里的一行豁免\n' "$(sha256sum .claude/agents/demo.md | cut -d' ' -f1)" > .claude/agent-def-review-exempt

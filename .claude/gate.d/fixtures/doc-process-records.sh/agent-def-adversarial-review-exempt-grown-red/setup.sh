#!/usr/bin/env bash
# 基准里已有豁免表 .claude/agent-def-review-exempt（一行：other.md）；工作区又加了一行，按改后那一版的内容哈希豁免 demo.md，demo.md 没有判决点名
# ⇒ 豁免表只缩不涨，新加的一行判红（行本身的哈希对得上，所以靠它能放过去的路被堵死）。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/gate.d
printf '# demo\n\n1. 跑完贴原样输出。\n' > .claude/agents/demo.md
printf '# other\n\n1. 只读。\n' > .claude/agents/other.md
printf '^\\.claude/hooks/\t# 钩子（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf '.claude/agents/other.md\t%s\t存量里的一行豁免\n' "$(sha256sum .claude/agents/other.md | cut -d' ' -f1)" > .claude/agent-def-review-exempt
git add -A && git commit -qm base
printf '# demo\n\n1. 跑完贴原样输出与退出码。\n' > .claude/agents/demo.md
printf '.claude/agents/demo.md\t%s\t想靠新加一行豁免过去\n' "$(sha256sum .claude/agents/demo.md | cut -d' ' -f1)" >> .claude/agent-def-review-exempt

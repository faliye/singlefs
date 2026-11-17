#!/usr/bin/env bash
# 过渡挂账表在基准里已经有一行（一份钩子）；工作区又加了一行（一份脚本，路径合规、理由够长），钩子与脚本都改了、没有判决
# ⇒ 只缩不涨，新加的一行判红：改了治理文件不能靠往表里加行放过去。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/hooks scripts/demo .claude/gate.d
printf '#!/usr/bin/env bash\necho 1\n' > .claude/hooks/demo-guard.sh
printf '#!/usr/bin/env bash\necho 1\n' > scripts/demo/run.sh
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf '.claude/hooks/demo-guard.sh\t存量钩子还没补三方\n' > .claude/gate.d/governance-review-pending.tsv
git add -A && git commit -qm base
printf '#!/usr/bin/env bash\necho 2\n' > .claude/hooks/demo-guard.sh
printf '#!/usr/bin/env bash\necho 2\n' > scripts/demo/run.sh
printf '.claude/hooks/demo-guard.sh\t存量钩子还没补三方\nscripts/demo/run.sh\t新改的脚本想靠挂账表放过去\n' > .claude/gate.d/governance-review-pending.tsv

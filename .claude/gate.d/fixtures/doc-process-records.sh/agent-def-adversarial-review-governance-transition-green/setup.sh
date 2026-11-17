#!/usr/bin/env bash
# 过渡挂账表 .claude/gate.d/governance-review-pending.tsv 在基准里有三行（一份钩子、一份脚本、一份项目规则）；工作区删了项目规则那一行，
# 改了钩子与脚本两份、没有判决 ⇒ 登记的路径这一格不要求点名，只缩不涨不红，本该判绿、报 2 份挂账、按类报各几份。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/hooks .claude/rules scripts/demo .claude/gate.d
printf '#!/usr/bin/env bash\necho 1\n' > .claude/hooks/demo-guard.sh
printf '#!/usr/bin/env bash\necho 1\n' > scripts/demo/run.sh
printf '# 规则\n\n- 只写现状。\n' > .claude/rules/demo.md
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
printf '# 样本里的过渡挂账表\n.claude/hooks/demo-guard.sh\t存量钩子还没补三方\nscripts/demo/run.sh\t存量脚本还没补三方\n.claude/rules/demo.md\t存量规则还没补三方\n' > .claude/gate.d/governance-review-pending.tsv
git add -A && git commit -qm base
printf '#!/usr/bin/env bash\necho 2\n' > .claude/hooks/demo-guard.sh
printf '#!/usr/bin/env bash\necho 2\n' > scripts/demo/run.sh
printf '# 样本里的过渡挂账表\n.claude/hooks/demo-guard.sh\t存量钩子还没补三方\nscripts/demo/run.sh\t存量脚本还没补三方\n' > .claude/gate.d/governance-review-pending.tsv

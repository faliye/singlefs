#!/usr/bin/env bash
# 过渡挂账表写坏五行：理由太短（钩子）、路径是定义与共用约束（已在管，不进这张表）、脚本登记两次、只有一段没有理由（项目规则）；
# 另一行合规。改了的钩子与脚本都登记着、没有判决 ⇒ 表本身判红列四处，而不是靠登记放过去。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/hooks .claude/rules .claude/agents scripts/demo .claude/gate.d
printf '#!/usr/bin/env bash\necho 1\n' > .claude/hooks/demo-guard.sh
printf '#!/usr/bin/env bash\necho 1\n' > scripts/demo/run.sh
printf '# 规则\n\n- 只写现状。\n' > .claude/rules/demo.md
printf '# demo\n\n1. 只读。\n' > .claude/agents/demo.md
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '#!/usr/bin/env bash\necho 2\n' > .claude/hooks/demo-guard.sh
printf '#!/usr/bin/env bash\necho 2\n' > scripts/demo/run.sh
printf '.claude/hooks/demo-guard.sh\t短理由\n' > .claude/gate.d/governance-review-pending.tsv
printf '.claude/agents/demo.md\t定义与共用约束已经在管，不该进过渡表\n' >> .claude/gate.d/governance-review-pending.tsv
printf 'scripts/demo/run.sh\t脚本这一行的理由够长了\n' >> .claude/gate.d/governance-review-pending.tsv
printf 'scripts/demo/run.sh\t同一个脚本登记两次的理由\n' >> .claude/gate.d/governance-review-pending.tsv
printf '.claude/rules/demo.md\n' >> .claude/gate.d/governance-review-pending.tsv

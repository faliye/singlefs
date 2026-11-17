#!/usr/bin/env bash
# 改了一份门禁（.claude/gate.d/ 顶层的 .sh）与它的 lib/ 下的一份库（.py），另改了同目录里的登记表（.tsv）与样本目录 fixtures 里的一份脚本（都不归这一格管）；
# 新写的判决在「## 这一轮被判过的文件」一节里点名了门禁与库两份 ⇒ 本该判绿、报门禁 2 份。
set -e
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/gate.d/lib .claude/gate.d/fixtures/demo-stage.sh/case research/prompts
printf '#!/usr/bin/env bash\necho 1\n' > .claude/gate.d/demo-stage.sh
printf 'VALUE = 1\n' > .claude/gate.d/lib/demo-lib.py
printf 'demo\t1\n' > .claude/gate.d/demo-table.tsv
printf '#!/usr/bin/env bash\necho 1\n' > .claude/gate.d/fixtures/demo-stage.sh/case/setup.sh
printf '^\\.claude/agents/\t# 定义（这份样本不碰）\n' > .claude/gate.d/knowledge-sync-triggers.tsv
git add -A && git commit -qm base
printf '#!/usr/bin/env bash\necho 2\n' > .claude/gate.d/demo-stage.sh
printf 'VALUE = 2\n' > .claude/gate.d/lib/demo-lib.py
printf 'demo\t2\n' > .claude/gate.d/demo-table.tsv
printf '#!/usr/bin/env bash\necho 2\n' > .claude/gate.d/fixtures/demo-stage.sh/case/setup.sh
cat > research/prompts/demo-r1-main-verification.md <<'MD'
# demo 第一轮：主 agent 核实

三条腿都没打中。

## 这一轮被判过的文件

- .claude/gate.d/demo-stage.sh
- .claude/gate.d/lib/demo-lib.py
MD

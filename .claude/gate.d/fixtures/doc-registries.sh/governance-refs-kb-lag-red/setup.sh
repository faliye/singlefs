#!/usr/bin/env bash
# 滞后表 .claude/gate.d/dead-path-lag.tsv 的红样本：
#   基准那一版三行：scripts/kb/base-lagging.py（kb 里还引着、不在，对的）、scripts/kb/now-present.py（此刻在了 ⇒ 这一行该删）、
#                   scripts/kb/no-longer-cited.py（kb 里已经不引 ⇒ 这一行该删）
#   工作区另加：scripts/kb/new-lagging.py（kb 里引着、不在，但基准里没有这一行 ⇒ 只缩不涨判红），
#               一行只有两列、一行理由不到 4 个字、一行与基准那一行重复（各判写得不对）
# kb 里两处不在的路径都登记了，自己不出红行。
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/kb .claude/gate.d scripts/kb
printf -- '---\nname: sample\ndescription: 门禁 doc-registries 的 governance-refs 格的红样本，不派发。\n---\n# 样例定义\n\n这一份不点名路径。\n' > .claude/agents/sample.md
printf '# 样例工具页\n\n旧工具 `scripts/kb/base-lagging.py` 还没改。\n另一处 `scripts/kb/new-lagging.py` 也没改。\n判法在 `scripts/kb/now-present.py`。\n' > .claude/kb/tooling.md
printf 'print("在")\n' > scripts/kb/now-present.py
printf '# 样本滞后表\n.claude/kb/tooling.md\tscripts/kb/base-lagging.py\t# 样本：存量，下一批改\n.claude/kb/tooling.md\tscripts/kb/now-present.py\t# 样本：那时还不在\n.claude/kb/tooling.md\tscripts/kb/no-longer-cited.py\t# 样本：那时还引着\n' > .claude/gate.d/dead-path-lag.tsv
git add -A && git commit -qm 基准
printf '.claude/kb/tooling.md\tscripts/kb/new-lagging.py\t# 样本：这一轮新加的\n.claude/kb/tooling.md\tscripts/kb/two-columns.py\n.claude/kb/tooling.md\tscripts/kb/short-reason.py\t# 短\n.claude/kb/tooling.md\tscripts/kb/base-lagging.py\t# 样本：重复登记的一行\n' >> .claude/gate.d/dead-path-lag.tsv

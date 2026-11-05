#!/usr/bin/env bash
# 保留表 .claude/gate.d/dead-path-kept.tsv 的红样本：
#   第 2 行 scripts/kb/kept-fine.py       kb 里引着、不在、理由够长          ⇒ 不红，计保留
#   第 3 行 scripts/kb/both-tables.py     滞后表第 2 行也登了同一对           ⇒ 判红「两张表只登一张」
#   第 4 行 scripts/kb/short-reason.py    理由不到 8 个字                      ⇒ 判红、这一行不认，kb 里那一处跟着判红
#   第 5 行 scripts/kb/now-built.py       此刻在了                             ⇒ 判红「建好了或回来了，删这一行」
#   第 6 行 scripts/kb/no-longer-cited.py kb 里已经不引                         ⇒ 判红「这一行该删」
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/kb .claude/gate.d scripts/kb
printf -- '---\nname: sample\ndescription: 门禁 doc-registries 的 governance-refs 格的红样本，不派发。\n---\n# 样例定义\n\n这一份不点名路径。\n' > .claude/agents/sample.md
printf '# 样例工具页\n\n留着的 `scripts/kb/kept-fine.py` 登对了。\n两张表都登的 `scripts/kb/both-tables.py`。\n理由太短的 `scripts/kb/short-reason.py`。\n已经建好的 `scripts/kb/now-built.py`。\n' > .claude/kb/tooling.md
printf 'print("建好了")\n' > scripts/kb/now-built.py
printf '# 样本滞后表\n.claude/kb/tooling.md\tscripts/kb/both-tables.py\t# 样本：存量，下一批改\n' > .claude/gate.d/dead-path-lag.tsv
printf '# 样本保留表\n.claude/kb/tooling.md\tscripts/kb/kept-fine.py\t# 样本：句子本身说它不在，登对了的一行\n.claude/kb/tooling.md\tscripts/kb/both-tables.py\t# 样本：句子本身说它不在，滞后表也登了\n.claude/kb/tooling.md\tscripts/kb/short-reason.py\t# 短理由\n.claude/kb/tooling.md\tscripts/kb/now-built.py\t# 样本：决策定的将来落点，那时还没建\n.claude/kb/tooling.md\tscripts/kb/no-longer-cited.py\t# 样本：抄下来的当时的命令，载体那时还引着\n' > .claude/gate.d/dead-path-kept.tsv
git add -A && git commit -qm 基准

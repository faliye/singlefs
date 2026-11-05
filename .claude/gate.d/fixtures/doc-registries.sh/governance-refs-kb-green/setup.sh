#!/usr/bin/env bash
# kb 中文页里这几处都不红：
#   research/results/e1-old-round.out   基准里提交过、之后删掉（归档）      ⇒ research/ 下进过 git，放行、计 1
#   `layout/01`、`tests/x.rs`、`src/lib.rs`  相对简写                      ⇒ 不判、跳过 3
#   scripts/kb/live.py                   在                                 ⇒ 判过、指得到
#   scripts/kb/lagging-tool.py           不在、登记在滞后表（基准里就有这一行） ⇒ 滞后 1
#   「## 历史版本」一节里的 scripts/kb/history-only.py、译文 tooling.en.md 里的 scripts/kb/english-only.py ⇒ 不扫
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/kb .claude/gate.d scripts/kb research/results
printf -- '---\nname: sample\ndescription: 门禁 doc-registries 的 governance-refs 格的绿样本，不派发。\n---\n# 样例定义\n\n这一份不点名路径。\n' > .claude/agents/sample.md
printf '# 样例工具页\n\n上一轮产物 `research/results/e1-old-round.out` 已归档。\n相对简写 `layout/01`、`tests/x.rs`、`src/lib.rs` 不判。\n判法在 `scripts/kb/live.py`。\n旧工具 `scripts/kb/lagging-tool.py` 还没改。\n\n## 历史版本\n\n- 那时在 `scripts/kb/history-only.py`。\n' > .claude/kb/tooling.md
printf '# Sample tooling page\n\nThe old tool lived at `scripts/kb/english-only.py`.\n' > .claude/kb/tooling.en.md
printf '# 样本滞后表\n.claude/kb/tooling.md\tscripts/kb/lagging-tool.py\t# 样本：存量，下一批改\n' > .claude/gate.d/dead-path-lag.tsv
printf 'print("在")\n' > scripts/kb/live.py
printf 'name=e1 verdict=true\n' > research/results/e1-old-round.out
git add -A && git commit -qm 基准
git rm -q research/results/e1-old-round.out && git commit -qm '归档上一轮产物'

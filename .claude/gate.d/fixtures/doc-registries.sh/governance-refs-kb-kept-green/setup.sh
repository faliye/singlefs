#!/usr/bin/env bash
# kb 中文页里这几处都不红：
#   .claude/kb/future-list.toml   不在、登记在保留表（决策定的将来落点）                     ⇒ 保留 1
#   `git log --diff-filter=D -- scripts/kb/gone-tool.py` 里 -- 之后的路径（基准之后删掉了）  ⇒ git 历史命令跳过 1
#   `scripts/kb/old-name.py` → `new-name.py`：old-name.py 基准之后 git mv 成 scripts/kb/new-name.py，
#                                 后继的文件名在同一行里                                    ⇒ 说搬迁的行跳过 1
#   scripts/kb/live.py            在                                                         ⇒ 判过、指得到
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/kb .claude/gate.d scripts/kb
printf -- '---\nname: sample\ndescription: 门禁 doc-registries 的 governance-refs 格的绿样本，不派发。\n---\n# 样例定义\n\n这一份不点名路径。\n' > .claude/agents/sample.md
printf '# 样例工具页\n\n黄金镜像清单将来放 `.claude/kb/future-list.toml`，还没建。\n旧工具删了，要再用 `git log --diff-filter=D -- scripts/kb/gone-tool.py` 找回。\n跟着改的文件名：`scripts/kb/old-name.py` → `new-name.py`。\n判法在 `scripts/kb/live.py`。\n' > .claude/kb/tooling.md
printf '# 样本保留表\n.claude/kb/tooling.md\t.claude/kb/future-list.toml\t# 样本：决策定的将来落点，清单还没建\n' > .claude/gate.d/dead-path-kept.tsv
printf 'print("在")\n' > scripts/kb/live.py
printf 'print("旧判法：这一份改名之后内容不变，git 认得出是改名")\n' > scripts/kb/old-name.py
printf 'print("退役的工具")\n' > scripts/kb/gone-tool.py
git add -A && git commit -qm 基准
git mv scripts/kb/old-name.py scripts/kb/new-name.py && git commit -qm '改名 old-name'
git rm -q scripts/kb/gone-tool.py && git commit -qm '删 gone-tool'

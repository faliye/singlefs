#!/usr/bin/env bash
# kb 中文页里四处指不到的路径，红行各带后继结论：
#   scripts/kb/old-name.py   基准之后 git mv 成 scripts/kb/new-name.py        ⇒ 「改成 scripts/kb/new-name.py（改名：改名链 …）」
#   scripts/kb/retired-tool.py 基准之后删掉、没有后继                         ⇒ 「删于 <提交>（删 retired-tool：判法并进门禁）；…`<提交>^:scripts/kb/retired-tool.py`」
#   research/prompts/never-written-prereg.md 从没进过 git                      ⇒ research/ 下也判红「从没进过 git」
#   crates/sample/invariants/01-样例类.md 里的 crates/sample/src/gone.rs       ⇒ 登记的 kb 目录（不变量清单）同样扫，「从没进过 git」
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/kb scripts/kb crates/sample/invariants
printf -- '---\nname: sample\ndescription: 门禁 doc-registries 的 governance-refs 格的红样本，不派发。\n---\n# 样例定义\n\n这一份不点名路径。\n' > .claude/agents/sample.md
printf '.claude/kb  # 样本：kb 页\ncrates/sample/invariants  # 样本：不变量清单按类拆在这里\n' > .claude/kb-directories
printf '# 样例工具页\n\n旧判法在 `scripts/kb/old-name.py`。\n原来有一份 `scripts/kb/retired-tool.py`。\n跑前登记在 `research/prompts/never-written-prereg.md`。\n' > .claude/kb/tooling.md
printf '# 样例类\n\n| I-1.1 | 样例条 | 判法在 `crates/sample/src/gone.rs` |\n' > crates/sample/invariants/01-样例类.md
printf 'print("旧判法：这一份改名之后内容不变，git 认得出是改名")\n' > scripts/kb/old-name.py
printf 'print("退役的工具：删掉之后仓里没有后继")\n' > scripts/kb/retired-tool.py
git add -A && git commit -qm 基准
git mv scripts/kb/old-name.py scripts/kb/new-name.py && git commit -qm '改名 old-name'
git rm -q scripts/kb/retired-tool.py && git commit -qm '删 retired-tool：判法并进门禁'

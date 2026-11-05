#!/usr/bin/env bash
# 两种自动跳过之外的照判：
#   第 3 行 `git log scripts/kb/before-dash.py -- scripts/kb/after-dash.py`：-- 之前的 before-dash.py 照判（从没进过 git ⇒ 红），
#           -- 之后的 after-dash.py 跳过、计 1
#   第 4 行 scripts/kb/moved-away.py      基准之后 git mv 成 scripts/kb/moved-here.py，同一行没写后继的文件名 ⇒ 红「改成 …」
#   第 5 行 scripts/kb/fixtures/table.tsv 基准之后 git mv 成 scripts/kb/table.tsv（文件名相同），同一行里只有旧路径自己带着这个文件名 ⇒ 红
#   第 6 行 `scripts/kb/renamed-old.py` → `renamed-new.py`：后继的文件名在同一行里 ⇒ 说搬迁的行跳过、计 1
set -euo pipefail
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t
git init -q -b master .
mkdir -p .claude/agents .claude/kb scripts/kb/fixtures
printf -- '---\nname: sample\ndescription: 门禁 doc-registries 的 governance-refs 格的红样本，不派发。\n---\n# 样例定义\n\n这一份不点名路径。\n' > .claude/agents/sample.md
printf '# 样例工具页\n\n取回命令 `git log scripts/kb/before-dash.py -- scripts/kb/after-dash.py`。\n`scripts/kb/moved-away.py` 已经改了名。\n表在 `scripts/kb/fixtures/table.tsv`。\n跟着改的文件名：`scripts/kb/renamed-old.py` → `renamed-new.py`。\n' > .claude/kb/tooling.md
printf 'print("改名之后内容不变，git 认得出是改名：moved-away")\n' > scripts/kb/moved-away.py
printf 'name\tvalue\nsample\t1\n' > scripts/kb/fixtures/table.tsv
printf 'print("改名之后内容不变，git 认得出是改名：renamed-old")\n' > scripts/kb/renamed-old.py
git add -A && git commit -qm 基准
git mv scripts/kb/moved-away.py scripts/kb/moved-here.py
git mv scripts/kb/fixtures/table.tsv scripts/kb/table.tsv
git mv scripts/kb/renamed-old.py scripts/kb/renamed-new.py
git commit -qm '改名三份'

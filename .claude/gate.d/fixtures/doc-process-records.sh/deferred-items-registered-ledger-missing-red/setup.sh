#!/usr/bin/env bash
# 红样本（现行账本读不到）：基准里有欠账表（C10 开着）、拆分提案封存那份（九十节有 ②）与一份调度记录，没有现行账本；
# 提交之后往调度记录追加一行「九十节 ②」（封存那份里指得到）。现行账本读不到照实报红，不退回只认封存那份。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p .claude/kb records
cat > .claude/kb/checks-owed.md <<'OWED'
# 欠的检查清单

| 编号 | 欠什么 | 依据 | 去向 |
|---|---|---|---|
| C10 | 样本欠账 | 样本依据 | 下一批工具改动 |
OWED
cat > records/2026-09-16-subagent拆分提案.md <<'HISTORY'
# 拆分提案（样本，封存那份）

## 九十、带行首序号的一节

| 序 | 现象 | 要拦什么 | 去向 |
|---|---|---|---|
| ② | 样本现象二 | 样本检查二 | 工具改动 |
HISTORY
printf '# 样本调度记录\n\n- 基准里早就有的旧行。\n' > records/2026-10-22-样本调度.md
mkdir -p .claude/gate.d
printf 'current\trecords/2026-10-29-subagent拆分提案.md\t-\t样本的现行账本，新欠账往这一份末尾追加\n' > .claude/gate.d/split-proposal-ledgers.tsv
printf 'sealed\trecords/2026-09-16-subagent拆分提案.md\t%s\t样本里封存的那一份，当历史出处\n' "$(sha256sum < records/2026-09-16-subagent拆分提案.md | cut -d' ' -f1)" >> .claude/gate.d/split-proposal-ledgers.tsv
git add -A
git commit -qm '基准：欠账表、封存那份拆分提案、一份调度记录'
cat >> records/2026-10-22-样本调度.md <<'NEW'
- 延后：九十节 ② 那一条留到下一轮再做。
NEW

#!/usr/bin/env bash
# 红样本（两份拆分提案）：基准里有欠账表（C10 开着）、拆分提案历史那份（九十节有 ②）、照抄它又多一节一百零二（有 ①）的欠账那份与一份调度记录；
# 提交之后往调度记录追加三行：一行写 C10（指得到，不报），两行逐个报：「第一百零九节 ①」两份都没有那一节、「九十节 ⑦」两份的九十节都没有这一行，原因里点名两份。
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
# 拆分提案（样本，历史那份）

## 九十、带行首序号的一节

| 序 | 现象 | 要拦什么 | 去向 |
|---|---|---|---|
| ② | 样本现象二 | 样本检查二 | 工具改动 |
HISTORY
cp records/2026-09-16-subagent拆分提案.md records/2026-10-23-subagent拆分提案.md
cat >> records/2026-10-23-subagent拆分提案.md <<'LEDGER'

## 一百零二、新登记的一节

| 序 | 现象 | 要拦什么 | 去向 |
|---|---|---|---|
| ① | 新登的现象 | 新登的检查 | 工具改动 |
LEDGER
printf '# 样本调度记录\n\n- 基准里早就有的旧行。\n' > records/2026-10-22-样本调度.md
git add -A
git commit -qm '基准：欠账表、两份拆分提案、一份调度记录'
cat >> records/2026-10-22-样本调度.md <<'NEW'
- 延后：C10 那一条留到下一轮再做。
- 延后：记在第一百零九节 ①。
- 下一批：九十节 ⑦ 那一条。
NEW

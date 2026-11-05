#!/usr/bin/env bash
# 绿样本（节号只认紧跟「拆分提案」的那一个）：基准里有欠账表（没有开着的 C 号）、账本登记表（只有现行一份）、现行账本（一百零六节有 ①，没有第五节、第三节）与一份调度记录；
# 提交之后往调度记录追加三行，都只核一百零六节：
#   「登记的第五节」写在「拆分提案第一百零六节 ①」前面 16 个字以内（整行都照拆分提案认时，「第五节」把 ① 吞过去、指不到，这一行判红）；
#   「拆分提案第一百零六节 ①，登记第五节」；「拆分提案（2026-10-29 份）第一百零六节 ①，判决的第三节另说」（括注隔开的照认）。
# 判绿报「判了 3 行」。弄坏开关 DEFERRED_ITEMS_BREAK=split-proposal-section-anywhere 下第一行判红，整份退 1。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p .claude/kb .claude/gate.d records
cat > .claude/kb/checks-owed.md <<'OWED'
# 欠的检查清单

| 编号 | 欠什么 | 依据 | 去向 |
|---|---|---|---|
OWED
cat > records/2026-10-29-subagent拆分提案.md <<'LEDGER'
# 拆分提案（样本，现行账本）

## 索引

| 节号 | 判定 |
|---|---|
| 一百零六 | 欠着 |

## 一百零六、样本里登记的一节

| 序 | 现象 | 要拦什么 | 去向 |
|---|---|---|---|
| ① | 样本现象 | 样本检查 | 下一批工具改动 |
LEDGER
printf '# 样本调度记录\n\n- 基准里早就有的旧行。\n' > records/2026-10-22-样本调度.md
printf 'current\trecords/2026-10-29-subagent拆分提案.md\t-\t样本的现行账本，新欠账往这一份末尾追加\n' > .claude/gate.d/split-proposal-ledgers.tsv
git add -A
git commit -qm '基准：欠账表、现行账本、账本登记表、一份调度记录'
cat >> records/2026-10-22-样本调度.md <<'NEW'
- 延后：登记的第五节写完，拆分提案第一百零六节 ① 那一条照它补。
- 下一批：拆分提案第一百零六节 ①，登记第五节。
- 留给下一轮：拆分提案（2026-10-29 份）第一百零六节 ①，判决的第三节另说。
NEW

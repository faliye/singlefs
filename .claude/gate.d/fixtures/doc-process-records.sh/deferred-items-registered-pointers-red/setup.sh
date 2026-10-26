#!/usr/bin/env bash
# 红样本（登记指到不算数的地方）：基准与绿样本相同（欠账表 C10 开着、C11 已还清，拆分提案两份，research/notes.md 30 行，一份调度记录）；
# 提交之后往调度记录追加五行：三行逐个报——第 4 行写已还清的 C11、第 5 行的 文件:行指到同一份调度记录的第 3 行（records/ 下不是拆分提案）、
# 第 6 行的 文件:行指到它自己；第 7 行指欠账那份的 records/2026-10-23-subagent拆分提案.md:8、第 8 行写开着的 C10，这两行指得到、不报。
set -euo pipefail
git init -q .
git config user.email selftest@example.invalid
git config user.name selftest
mkdir -p .claude/kb records research
cat > .claude/kb/checks-owed.md <<'OWED'
# 欠的检查清单

| 编号 | 欠什么 | 依据 | 去向 |
|---|---|---|---|
| C10 | 样本欠账 | 样本依据 | 下一批工具改动 |

### 已还清

| 编号 | 欠什么 | 依据 | 怎么还的 |
|---|---|---|---|
| C11 | 已还清的样本欠账 | 样本依据 | 样本里还清 |
OWED
cat > records/2026-09-16-subagent拆分提案.md <<'SPLIT'
# 拆分提案（样本）

## 九十、带行首序号的一节

| 序 | 现象 | 要拦什么 | 去向 |
|---|---|---|---|
| ② | 样本现象二 | 样本检查二 | 下一批工具改动 |
| ③ | 样本现象三 | 样本检查三 | 下一批工具改动 |
SPLIT
cp records/2026-09-16-subagent拆分提案.md records/2026-10-23-subagent拆分提案.md   # 欠账那份：开着的节照原节号抄过去
seq 1 30 | sed 's/^/样本第 /; s/$/ 行/' > research/notes.md
printf '# 样本调度记录\n\n- 延后：基准里早就有的旧行，没写登记，这一次没改它，不判。\n' > records/2026-10-22-样本调度.md
git add -A
git commit -qm '基准：欠账表、拆分提案、一份调度记录'
cat >> records/2026-10-22-样本调度.md <<'NEW'
- 延后：C11 那一条留到下一轮再做。
- 下一批：照 records/2026-10-22-样本调度.md:3 那一行补上。
- 下一个题目：records/2026-10-22-样本调度.md:6 就是这一行。
- 留给下一轮：照 records/2026-10-23-subagent拆分提案.md:8 那一行做。
- 延后：C10 那一条留到下一轮再做。
NEW

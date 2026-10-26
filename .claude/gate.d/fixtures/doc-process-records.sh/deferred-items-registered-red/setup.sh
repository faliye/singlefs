#!/usr/bin/env bash
# 红样本：基准与绿样本相同（欠账表 C10 开着、C11 已还清，拆分提案历史那份九十节有 ② ③、九十四节只有正文，欠账那份照抄它，research/notes.md 30 行）；
# 提交之后往调度记录追加七行带延后类词的：一行写 C10（指得到，不报），六行逐个报：没写登记、C99 欠账表里没有、「九十节 ⑤」那一节没有这一行、
# 写了「拆分提案」的「第九十节」有行首序号却没写哪一行、research/notes.md:99 超过行数、gone.md:3 文件不在树里。
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

## 九十四、只有正文的一节

- 欠：样本里只写正文、没有行首序号的一节。
SPLIT
cp records/2026-09-16-subagent拆分提案.md records/2026-10-23-subagent拆分提案.md   # 欠账那份：开着的节照原节号抄过去
seq 1 30 | sed 's/^/样本第 /; s/$/ 行/' > research/notes.md
printf '# 样本调度记录\n\n- 延后：基准里早就有的旧行，没写登记，这一次没改它，不判。\n' > records/2026-10-22-样本调度.md
git add -A
git commit -qm '基准：欠账表、拆分提案、一份调度记录'
cat >> records/2026-10-22-样本调度.md <<'NEW'
- 延后：C10 那一条留到下一轮再做。
- 延后：这一行没写登记。
- 下一批：C99 那一条。
| ⑤ 的检查 | 下轮开 | 九十节 ⑤ |
- 留给以后：记在拆分提案第九十节。
- 下一个题目：research/notes.md:99 那一段。
- 延后：见 gone.md:3。
NEW

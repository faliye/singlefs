#!/usr/bin/env bash
# 绿样本：基准提交里有欠账表（C10 开着、C11 已还清）、拆分提案历史那份（九十节有 ② ③ 两行，九十四节只有正文）与照抄它的欠账那份、research/notes.md（30 行）与一份调度记录；
# 提交之后往调度记录追加：C10、欠账那份的 records/2026-10-23-subagent拆分提案.md:8、「九十节 ③」、写了「拆分提案」的「第九十四节」、research/notes.md:12 各指一行（判了 5 行，都指得到；
# 已还清的 C11 不再算登记，那一种在 deferred-items-registered-pointers-red 里）；
# 另有词只在「」里的一行、围栏里的一行、往拆分提案九十节加的新一行（登记处本身）不判，没带延后类词的一行与基准里的旧行不判。
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
- 下一批：照 records/2026-10-23-subagent拆分提案.md:8 那一行补上。
| ③ 的检查 | 下轮开 | 九十节 ③ |
- 留给下一轮：登在 `records/2026-09-16-subagent拆分提案.md` 第九十四节。
- 下一个题目：research/notes.md:12 那一段。
- 照规则「延后的当时就登记」写。
- 这一行没有那几个词，不判。

```
延后：围栏里举例的一行，不判
```
NEW
sed -i 's/^| ③ | 样本现象三 | 样本检查三 | 下一批工具改动 |$/&\n| ④ | 样本现象四 | 样本检查四 | 下一批工具改动 |/' records/2026-09-16-subagent拆分提案.md
grep -q '^| ④ | 样本现象四' records/2026-09-16-subagent拆分提案.md

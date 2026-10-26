#!/usr/bin/env bash
# 绿样本（「延后」只在点话题）：基准里有欠账表（C10 开着）、拆分提案与一份调度记录；提交之后往调度记录追加七行：
# 两行标题（词只在名词「延后项」里的一行、带「下一批」的一行）、两行只把「延后项」当话题名词、一行只在报统计数（「延后项无去向 8/54」「延后当时就登进欠账表的 6/54」），
# 都不判；一行写 C10 照判、指得到，判绿报「判了 1 行」，不判的里名词与统计数 4 行、标题 1 行。
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
OWED
cat > records/2026-09-16-subagent拆分提案.md <<'SPLIT'
# 拆分提案（样本）

## 九十、带行首序号的一节

| 序 | 现象 | 要拦什么 | 去向 |
|---|---|---|---|
| ② | 样本现象二 | 样本检查二 | 下一批工具改动 |
SPLIT
cp records/2026-09-16-subagent拆分提案.md records/2026-10-23-subagent拆分提案.md   # 欠账那份：开着的节照原节号抄过去
printf '# 样本调度记录\n\n- 基准里早就有的旧行。\n' > records/2026-10-22-样本调度.md
git add -A
git commit -qm '基准：欠账表、拆分提案、一份调度记录'
cat >> records/2026-10-22-样本调度.md <<'NEW'

### 延后项的登记

## 下一批要做的两件

| 延后项没进欠账表 | 真，普遍 | 只有少数当时就登记 |
- 交接类缺口有多普遍：延后项去向、回扫的行、载体搬家。
- 抽样：延后项无去向 8/54，延后当时就登进欠账表的 6/54。
- 延后：C10 那一条留到下一轮再做。
NEW

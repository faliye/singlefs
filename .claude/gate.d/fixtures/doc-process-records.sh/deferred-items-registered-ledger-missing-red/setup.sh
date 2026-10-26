#!/usr/bin/env bash
# 红样本（欠账那份读不到）：基准里有欠账表（C10 开着）、拆分提案历史那份（九十节有 ②）与一份调度记录，没有欠账那份；
# 提交之后往调度记录追加一行「九十节 ②」（历史那份里指得到）。欠账那份读不到照实报红，不退回只认历史那份。
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
printf '# 样本调度记录\n\n- 基准里早就有的旧行。\n' > records/2026-10-22-样本调度.md
git add -A
git commit -qm '基准：欠账表、历史那份拆分提案、一份调度记录'
cat >> records/2026-10-22-样本调度.md <<'NEW'
- 延后：九十节 ② 那一条留到下一轮再做。
NEW

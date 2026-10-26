#!/usr/bin/env bash
# 红样本（「延后」只在点话题的收窄不放过真延后）：基准同 topic-green；提交之后往调度记录追加六行，一行写 C10（指得到，不报），五行逐个报：
# 「进延后项」是把这一件延后、名词「延后项」旁边另有「下一批」、「延后到」不是名词、井号后面没有空格不是标题、统计数旁边的「下一批」不是「延后」。
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
- 另报两条，进延后项：没写登记。
- 延后项还有两件，下一批再做。
- 延后到下一轮：没写登记。
#延后：没有空格的井号不是标题。
- 下一批跑剩下的 3/10。
- 延后：C10 那一条留到下一轮再做。
NEW

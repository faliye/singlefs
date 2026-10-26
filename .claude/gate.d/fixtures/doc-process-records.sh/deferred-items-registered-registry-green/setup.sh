#!/usr/bin/env bash
# 绿样本（登记处）：基准里有欠账表、拆分提案、一份调度记录、两份流程文件与登记处表（两行都写对）；提交之后往调度记录追加三行：
# 两行表格的去向格（延后类词那一格之后）各点名一份登记处（一份在反引号里、一份不在），一行写 C10，判绿报「判了 3 行」。
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
mkdir -p .claude/gate.d .claude/skills/gate .claude/skills/crash-test
printf '# 提交前的整轮门禁（样本）\n' > .claude/skills/gate/staged-gate.md
printf '# 提交时的六道重阶段（样本）\n' > .claude/skills/crash-test/commit-stages.md
printf '# 别的流程（样本，没登记成登记处）\n' > .claude/skills/gate/other-process.md
printf '# 登记处表（样本）\n.claude/skills/gate/staged-gate.md\t提交前的整轮门禁由主 agent 每次提交时照它跑\n.claude/skills/crash-test/commit-stages.md\t提交时的六道重阶段由主 agent 每次提交时照它跑\n' > .claude/gate.d/deferral-registries.tsv
git add -A
git commit -qm '基准：欠账表、拆分提案、调度记录、流程文件与登记处表'
cat >> records/2026-10-22-样本调度.md <<'NEW'
| 提交前整轮（样本档、开关复跑格） | 下轮开 | 提交时照 `.claude/skills/gate/staged-gate.md` 跑 | — |
| 六道重阶段 | 延后 | 提交时主 agent 照 .claude/skills/crash-test/commit-stages.md 跑 |
- 延后：C10 那一条留到下一轮再做。
NEW

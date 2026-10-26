#!/usr/bin/env bash
# 红样本（登记处）：登记处表五行里一行写对（staged-gate.md），四行写坏逐行报（文件不在树里、理由不到 8 个字、只有一段、同一份登记两次）；
# 提交之后往调度记录追加八行，一行在去向格点名 staged-gate.md（不报），七行逐个报：只写「九十节各行」、点名没登记的流程文件、
# 登记处只在延后类词那一格之前（被延后的事本身）、不是表格行、staged-gate.md.old 不是 staged-gate.md、点名的两份在表里那一行写坏了。
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
printf '# 登记处表（样本）\n.claude/skills/gate/staged-gate.md\t提交前的整轮门禁由主 agent 每次提交时照它跑\n.claude/skills/gone.md\t这份文件不在树里，这一行该报\n.claude/skills/crash-test/commit-stages.md\t太短\n.claude/skills/gate/other-process.md\n.claude/skills/gate/staged-gate.md\t同一份文件登记第二次，这一行该报\n' > .claude/gate.d/deferral-registries.tsv
git add -A
git commit -qm '基准：欠账表、拆分提案、调度记录、流程文件与登记处表'
cat >> records/2026-10-22-样本调度.md <<'NEW'
| ㉕ ㉖ | 下轮开 | 九十节各行，下一批工具改动 |
| 提交前整轮 | 下轮开 | 照 `.claude/skills/gate/other-process.md` 跑 |
| 照 `.claude/skills/gate/staged-gate.md` 改那三行 | 下轮开 | — |
- 没改成：`.claude/skills/gate/staged-gate.md` 第 2b 步三行，并进下一批治理文件。
| 改名 | 下轮开 | 照 `.claude/skills/gate/staged-gate.md.old` 跑 |
| 六道重阶段 | 下轮开 | 照 `.claude/skills/crash-test/commit-stages.md` 跑 |
| 已删的流程 | 下轮开 | 照 `.claude/skills/gone.md` 跑 |
| 提交前整轮 | 下轮开 | 提交时照 `.claude/skills/gate/staged-gate.md` 跑 |
NEW

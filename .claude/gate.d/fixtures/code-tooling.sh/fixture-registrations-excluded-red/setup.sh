#!/usr/bin/env bash
# 登记标题与登记表标记现写：静态样本文件里放真的登记位，本仓的 doc-lint 扫到样本目录就多出登记位。
# 三道阶段：doc-sample.sh 有编号登记标题而排除表没有它，table-sample.sh 有登记表标记而排除表没有它，covered-sample.sh 有登记标题而排除表有它。
set -e
fixtures=.claude/gate.d/fixtures
mkdir -p "$fixtures/doc-sample.sh/red/.claude/kb/decisions" "$fixtures/table-sample.sh/red/.claude/kb" "$fixtures/covered-sample.sh/red/research/prompts"
printf '# 样本决策\n\n## D5 样本 —— 已定\n\n正文。\n' > "$fixtures/doc-sample.sh/red/.claude/kb/decisions/05-样本.md"
printf '# 样本欠账表\n\n<!-- doc-lint:registry name-col=2 -->\n| 编号 | 简称 |\n|---|---|\n| C1 | 样本欠账 |\n' > "$fixtures/table-sample.sh/red/.claude/kb/checks-owed.md"
printf '# 样本实验页\n\n## E7 样本 —— 已定\n\n正文。\n' > "$fixtures/covered-sample.sh/red/research/prompts/e7-样本.md"

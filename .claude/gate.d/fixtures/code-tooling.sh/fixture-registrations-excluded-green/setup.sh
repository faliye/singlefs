#!/usr/bin/env bash
# 登记标题现写（原因见 fixture-registrations-excluded-red/setup.sh）。四道阶段：
#   slash-sample.sh   有编号登记标题，表里一行带斜杠
#   bare-sample       有登记表标记，表里一行不带斜杠
#   plain-sample.sh   没有登记位，不在表里
#   fenced-sample.sh  只在代码围栏里写了登记标题（doc-lint 不认围栏里的），不在表里
set -e
fixtures=.claude/gate.d/fixtures
mkdir -p "$fixtures/slash-sample.sh/red/.claude/kb/decisions" "$fixtures/bare-sample/green/.claude/kb" "$fixtures/plain-sample.sh/red" "$fixtures/fenced-sample.sh/red"
printf '# 样本决策\n\n## D5 样本 —— 已定\n\n正文。\n' > "$fixtures/slash-sample.sh/red/.claude/kb/decisions/05-样本.md"
printf '# 样本欠账表\n\n<!-- doc-lint:registry name-col=2 -->\n| 编号 | 简称 |\n|---|---|\n| C1 | 样本欠账 |\n' > "$fixtures/bare-sample/green/.claude/kb/checks-owed.md"
printf '# 没有登记位的样本\n\n## 讨论 D5 的写法\n\n正文。\n' > "$fixtures/plain-sample.sh/red/note.md"
printf '# 围栏里的写法\n\n```\n## D5 样本 —— 已定\n```\n' > "$fixtures/fenced-sample.sh/red/fence.md"

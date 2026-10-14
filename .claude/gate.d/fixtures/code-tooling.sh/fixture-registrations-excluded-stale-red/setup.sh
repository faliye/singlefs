#!/usr/bin/env bash
# 登记标题现写（原因见 fixture-registrations-excluded-red/setup.sh）。live-sample.sh 有登记标题且在表里；表里另有一行指到不存在的 gone-sample.sh。
set -e
fixtures=.claude/gate.d/fixtures
mkdir -p "$fixtures/live-sample.sh/red/.claude/kb/decisions"
printf '# 样本决策\n\n## D5 样本 —— 已定\n\n正文。\n' > "$fixtures/live-sample.sh/red/.claude/kb/decisions/05-样本.md"

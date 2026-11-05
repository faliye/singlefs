#!/usr/bin/env bash
# 基准里三行带日期的现状句都登记在滞后表里；这一轮把里程碑那一行改成现值、删掉滞后表里它那一行（只缩）：判绿，报滞后 2 行、与基准比没涨。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/gate.d
{
  printf '%s\n' '# 样本：带日期的现状句的滞后表（doc-text 的 dated-status 格）。一行「<路径><TAB><行内锚><TAB># <为什么还没改>」，只缩不涨'
  printf '%s\t%s\t%s\n' '.claude/kb/milestone/02-样本里程碑.md' '样本写路径做出来了，崩溃点重放 12 段全过' '# 样本：回扫员逐行判过，下一批写回'
  printf '%s\t%s\t%s\n' '.claude/kb/experiments/01-样本实验.md' '样本装置跑完第一段，第二段还没跑' '# 样本：回扫员逐行判过，下一批写回'
  printf '%s\t%s\t%s\n' 'crates/sample-checker/invariants/I-1-样本.md' '只判了一半，另一半归增补收口表' '# 样本：回扫员逐行判过，下一批写回'
} > .claude/gate.d/dated-status-lag.tsv
git add -A && git commit -qm base
python3 - <<'PY'
milestone = '.claude/kb/milestone/02-样本里程碑.md'
text = open(milestone, encoding='utf-8').read()
text = text.replace('**现状（2026-09-17 落地）**：样本写路径做出来了，崩溃点重放 12 段全过。', '样本写路径做出来了，崩溃点重放 12 段全过。')
open(milestone, 'w', encoding='utf-8').write(text)
table = '.claude/gate.d/dated-status-lag.tsv'
rows = [row for row in open(table, encoding='utf-8').read().split('\n') if not row.startswith('.claude/kb/milestone/')]
open(table, 'w', encoding='utf-8').write('\n'.join(rows))
PY

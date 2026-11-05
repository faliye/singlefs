#!/usr/bin/env bash
# 基准里滞后表只登记里程碑页的一行；这一轮往滞后表加另一行：滞后表比基准那一版多 1 行，判红（两行命中都算滞后）。
set -e
git init -q .
git config user.email t@example.com; git config user.name t
mkdir -p .claude/gate.d
{
  printf '%s\n' '# 样本：带日期的现状句的滞后表（doc-text 的 dated-status 格）。一行「<路径><TAB><行内锚><TAB># <为什么还没改>」，只缩不涨'
  printf '%s\t%s\t%s\n' '.claude/kb/milestone/02-样本里程碑.md' '分配器加了释放，样本释放路径接上' '# 样本：回扫员逐行判过，下一批写回'
} > .claude/gate.d/dated-status-lag.tsv
git add -A && git commit -qm base
printf '%s\t%s\t%s\n' '.claude/kb/milestone/02-样本里程碑.md' '样本对比跑完一臂，另两臂还没跑' '# 样本：新写的句子不改，往表里加了一行' >> .claude/gate.d/dated-status-lag.tsv

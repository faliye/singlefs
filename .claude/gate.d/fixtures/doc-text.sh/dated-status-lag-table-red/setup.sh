#!/usr/bin/env bash
# 不建 git 仓（不比基准）：滞后表第 2 行写对，第 3 到第 9 行各错一处，逐行点名判红；写对的那一行照放过，页里那一行现状句不报。
set -e
mkdir -p .claude/gate.d
page='.claude/kb/milestone/01-样本里程碑.md'
{
  printf '%s\n' '# 样本：带日期的现状句的滞后表（doc-text 的 dated-status 格）。一行「<路径><TAB><行内锚><TAB># <为什么还没改>」，只缩不涨'
  printf '%s\t%s\t%s\n' "$page" '样本建池吃两块设备，按字节表写出实例' '# 样本：回扫员逐行判过，下一批写回'
  printf '%s\t%s\t%s\n' "$page" '这一段原文在页里找不到了一个字也没有' '# 样本：那一行早改好了没删'
  printf '%s\t%s\t%s\n' "$page" '验收：样本建池吃两块设备' '# 样本：锚在页里出现两次'
  printf '%s\t%s\t%s\n' '.claude/kb/milestone/09-不在的页.md' '这一页已经删掉了整份都不在' '# 样本：页删了没删行'
  printf '%s\t%s\t%s\n' "$page" '样本建池' '# 样本：锚太短认不准'
  printf '%s\t%s\n' "$page" '样本建池吃两块设备，按字节表写出实例'
  printf '%s\t%s\t%s\n' "$page" '按字节表写出实例，样本建池吃两块' '# 短'
  printf '%s\t%s\t%s\n' "$page" '样本建池吃两块设备，按字节表写出实例' '# 样本：与第 2 行重复登记'
} > .claude/gate.d/dated-status-lag.tsv

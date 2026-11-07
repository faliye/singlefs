#!/usr/bin/env bash
# 不建 git 仓：over.rs 命中 3 行而滞后表只登记 2 行，unlisted.rs 命中 1 行没登记：两份各把全部命中列出，判红。
set -e
mkdir -p .claude/gate.d
{
  printf '%s\n' '# 样本：注释历史写法的滞后表（doc-text 的 comment-history 格）。一行「<路径><TAB><命中行数><TAB># <为什么还没改>」，只缩不涨'
  printf '%s\t%s\t%s\n' 'crates/demo/src/over.rs' '2' '# 样本：登记了 2 行，此刻命中 3 行'
} > .claude/gate.d/comment-history-lag.tsv

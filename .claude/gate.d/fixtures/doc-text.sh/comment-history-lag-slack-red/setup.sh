#!/usr/bin/env bash
# 不建 git 仓：slack.rs 命中 2 行而滞后表登记 3 行：点名改小到 2，判红。
set -e
mkdir -p .claude/gate.d
{
  printf '%s\n' '# 样本：注释历史写法的滞后表（doc-text 的 comment-history 格）。一行「<路径><TAB><命中行数><TAB># <为什么还没改>」，只缩不涨'
  printf '%s\t%s\t%s\n' 'crates/demo/src/slack.rs' '3' '# 样本：登记了 3 行，此刻只命中 2 行'
} > .claude/gate.d/comment-history-lag.tsv

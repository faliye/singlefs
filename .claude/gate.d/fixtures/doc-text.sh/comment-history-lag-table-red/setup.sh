#!/usr/bin/env bash
# 不建 git 仓（不比基准）：滞后表第 2 行写对，第 3 到第 8 行各错一处（重复登记、只有两列、数不是整数、理由不到 4 个字、路径不在、数是 0），逐行点名判红；
# short.sh 与 short2.sh 的登记行写错了没生效，它们的命中一并判红，写对的 stock.sh 照放过。
set -e
mkdir -p .claude/gate.d
{
  printf '%s\n' '# 样本：滞后表写错的几种'
  printf '%s\t%s\t%s\n' 'scripts/demo/stock.sh' '2' '# 样本存量，等这一批改完'
  printf '%s\t%s\t%s\n' 'scripts/demo/stock.sh' '2' '# 样本存量，等这一批改完'
  printf '%s\t%s\n' 'scripts/demo/short.sh' '1'
  printf '%s\t%s\t%s\n' 'scripts/demo/clean.py' '二' '# 样本存量，等这一批改完'
  printf '%s\t%s\t%s\n' 'scripts/demo/short2.sh' '1' '# 短'
  printf '%s\t%s\t%s\n' 'scripts/demo/gone.sh' '1' '# 样本存量，等这一批改完'
  printf '%s\t%s\t%s\n' 'scripts/demo/clean.py' '0' '# 样本存量，等这一批改完'
} > .claude/gate.d/comment-history-lag.tsv

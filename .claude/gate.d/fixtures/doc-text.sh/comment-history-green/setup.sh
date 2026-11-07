#!/usr/bin/env bash
# 不建 git 仓：射程里各类文件都只写现状，三行带够长标记的放过，README 围栏里的词表词不判，豁免表里那份整份不判，射程外的不判：判绿。
set -e
mkdir -p .claude/gate.d
{
  printf '%s\n' '# 样本：整份不判的文件'
  printf '%s\t%s\n' 'scripts/demo/word-list.py' '# 样本里只讲这些词的词表文件'
} > .claude/gate.d/comment-history-exempt.tsv

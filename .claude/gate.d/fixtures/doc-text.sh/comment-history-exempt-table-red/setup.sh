#!/usr/bin/env bash
# 不建 git 仓：豁免表第 2 行写对，第 3 到第 6 行各错一处（重复登记、理由不到 8 个字、路径不在、三列），逐行点名判红；
# word-list.py 与 three.py 的登记行写错了没生效，它们的命中一并判红，写对的 ok-words.py 照放过。
set -e
mkdir -p .claude/gate.d
{
  printf '%s\n' '# 样本：豁免表写错的几种'
  printf '%s\t%s\n' 'scripts/demo/ok-words.py' '# 样本里只讲这些词的词表文件'
  printf '%s\t%s\n' 'scripts/demo/ok-words.py' '# 样本里只讲这些词的词表文件'
  printf '%s\t%s\n' 'scripts/demo/word-list.py' '# 词表'
  printf '%s\t%s\n' 'scripts/demo/gone.py' '# 样本里整份不判的词表文件'
  printf '%s\t%s\t%s\n' 'scripts/demo/three.py' 'a' '# 样本里整份不判的词表文件'
} > .claude/gate.d/comment-history-exempt.tsv

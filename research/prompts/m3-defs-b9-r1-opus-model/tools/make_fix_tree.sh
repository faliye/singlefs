#!/usr/bin/env bash
# 攻方试改的那一棵：冻结副本硬链接拷一份，三份被改的文件先换成新 inode，再打 <模型目录>/patches/ 下的补丁（F-Z1、F-Z4、F-Z5，只在本腿模型上量过、被攻过零轮）。
# 用法：make_fix_tree.sh <冻结副本根> <模型目录> <目标目录（不在）>
set -euo pipefail
frozen="$1" model="$2" tree="$3"
cp -al "$frozen" "$tree"
for path in .claude/hooks/runner-dispatch-guard.sh .claude/hooks/lib_abandoned_rounds.py .claude/gate.d/doc-text.sh; do
  cp --remove-destination "$frozen/$path" "$tree/$path"
  ( cd "$tree" && patch -s -p1 < "$model/patches/$(basename "$path").diff" )
done

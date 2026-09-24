#!/usr/bin/env bash
# 绿样本：清环境、核祖先、判了退出码的调用，都不算自己算基准
unset GATE_BASE GATE_STAGED_FROM
git merge-base --is-ancestor "$recorded_commit" HEAD || exit 1
if added="$(gate_added_lines "$base" .claude/kb)"; then :; fi

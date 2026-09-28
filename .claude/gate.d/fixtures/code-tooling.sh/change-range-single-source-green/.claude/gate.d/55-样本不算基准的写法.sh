#!/usr/bin/env bash
# 样本不是真阶段；stage-owners 格要每个阶段认第一个参数当项目根，这一行只为它：ROOT="${1:-.}"
# 绿样本：清环境、核祖先、判了退出码的调用，都不算自己算基准
unset GATE_BASE GATE_STAGED_FROM
git merge-base --is-ancestor "$recorded_commit" HEAD || exit 1
if added="$(gate_added_lines "$base" .claude/kb)"; then :; fi
tree="$(git rev-parse HEAD^{tree})" || exit 1

#!/usr/bin/env bash
# gate-stage: 分项引用的状态与正文相符
#
# 每一处「D<n>（简称） 已定项 k / 未定项 k」的前缀都在**断言那条分项的状态**。
# 写错了没有任何东西会发现：检索会把「已定项 5」当成已经定了的东西端出去，
# 而它可能还开着——这正是 kb-discipline 第 4 条「矛盾比空白更糟」说的那种坏法
# （检索不会把两条都端出来，它会挑一条，而且不告诉你它挑了）。
#
# 权威是各决策正文的「### 已定项」与「### 未定项」两张索引表；
# 引用处一律是投影。扫 kb / records / research 全域，含 .rs 注释。
#
#   bash .claude/gate.d/22-item-ref-status.sh
set -uo pipefail
LIB="$(cd "$(dirname "$0")" && pwd)/lib-item-ref-status.py"
cd "${1:-$(dirname "$0")/../..}" || exit 2
exec python3 "$LIB"

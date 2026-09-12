#!/usr/bin/env bash
# gate-stage: 决策变更史快查表与原文同步
#
# `decisions-history.md` 是决策变更史的快查表：每条变更一行，日期、改了什么、改前、改后。
# 原文按月住在 `decisions-history/<年-月>.md`。快查表是原文的投影，不是第二处权威记录——
# 手抄一份就会漂，而漂了没有任何东西会发现（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条）。
#
# 本阶段判四样：原文有条目而快查表没有那一行；快查表多出原文里没有的行；改前 / 改后还是「（待补）」或空着；
# 摘要里冒出那一条原文里没有的数字、D / E / C / I 编号或「已定项 k」。
# ⚠️ 它判不了摘要写得对不对——那要人拿原文对。它只拦「漏写」与「数和编号走样」这两种机器看得见的。
#
#   bash .claude/gate.d/49-history-brief.sh          只比对
#   bash .claude/gate.d/49-history-brief.sh --write  按原文重排快查表（已写的摘要保留，新条目填「（待补）」）
set -uo pipefail
LIB="$(cd "$(dirname "$0")" && pwd)/lib-history-brief.py"
if [[ "${1:-}" == "--write" ]]; then
  cd "$(dirname "$0")/../.." || exit 2
  python3 "$LIB" write
  exit $?
fi
cd "${1:-$(dirname "$0")/../..}" || exit 2
python3 "$LIB" check

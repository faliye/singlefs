#!/usr/bin/env bash
# gate-stage: 决策变更史的快查与原文同步
#
# 决策变更史的原文按月住在 `decisions-history/<年-月>.md`，每条原文标题下写两行快查（改前、改后各一句）；
# `decisions-history.md` 按决策汇总这些快查，整块由 `lib-history-brief.py` 生成，不是第二处权威记录——
# 手抄一份就会漂，而漂了没有任何东西会发现（`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条）。
#
# 本阶段判四样：条目没写快查；快查还是「（待补）」；快查里冒出那一条原文里没有的数字、D / E / C / I 编号或
# 「已定项 k」；decisions-history.md 与按原文重新生成的不一致。
# ⚠️ 它判不了快查写得对不对——那要人拿原文对。它只拦「漏写」与「数和编号走样」这两种机器看得见的。
#
#   bash .claude/gate.d/49-history-brief.sh          只比对
#   bash .claude/gate.d/49-history-brief.sh --write  给一行快查都没写的条目补「（待补）」，再重新生成 decisions-history.md
set -uo pipefail
LIB="$(cd "$(dirname "$0")" && pwd)/lib-history-brief.py"
if [[ "${1:-}" == "--write" ]]; then
  cd "$(dirname "$0")/../.." || exit 2
  python3 "$LIB" write
  exit $?
fi
cd "${1:-$(dirname "$0")/../..}" || exit 2
python3 "$LIB" check

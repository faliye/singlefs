#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 它调的 research/scripts/verify-citations.sh 要本机固定下来的外部文献树，缺了在那一步判红，不交给 gate.sh 预判
# gate-stage: 外部引用还核得动吗
#
# 还 checks-owed.md C38（外部文献不可复核）的源码那一半。
#
# **它要拦的是证据蒸发，不是 kb 写错。** 2026-08-29 的 find 发现一批标着
# 「本机 PDF 逐字核实」的文献已经不在本机了，而引用它们的结论仍标着「已核实」——
# 那批蒸发得**无声无息**。源码这一侧会以同样的方式蒸发：路径变了、版本升了、树被删了。
#
# ⚠️ **源码树不在也判红，不许当跳过**——「跳过」正是让上一批文献无声蒸发的那个行为。
# 红了不等于 kb 写错，处置见脚本自己打印的下一步。
#
# 判别力已双向证过（2026-08-29）：
#   FS_REFS=/nonexistent ⇒ 固定点三棵树（refs-linux、refs-zfs、refs-docs）上的断言成批未命中、rc=1
#   （条数随断言表增减，这里不写死：现跑一次看末行「N 条未命中」）；
#   把 spa.h 的 SPA_BLKPTRSHIFT 从 7 改成 9 ⇒ 该条未命中、rc=1。
#
# 样本：fixtures/70-citations.sh/red 是一个没有 verify-citations.sh 的仓，必须判红；green 放一份替身 verify-citations.sh
# （退 0、打一行 ✓），判的是本阶段自己那两件事：脚本在被判的仓里就去跑它，按它的退出码判。
# 真 verify-citations.sh 的判别力不在这里测：它读两个本机绝对路径下的源码树（FS_REFS、KERNEL_TREE 的默认值），装不进密封的样本目录；
# 它自己的 --selftest 合成两棵假树测，由 47 号跑。
#
#   bash .claude/gate.d/70-citations.sh [项目根]
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" || exit 2
S=research/scripts/verify-citations.sh
[[ -f "$S" ]] || {
  echo "  ✗ 找不到 $S"
  echo "     → 怎么办：这一阶段靠它逐条复核引文。文件被挪走就改这里的路径，"
  echo "               还没写就先写它——缺了它，引文一条都没被验过。"
  exit 1; }
bash "$S"

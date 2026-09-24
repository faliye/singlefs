#!/usr/bin/env bash
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
#   FS_REFS=/nonexistent ⇒ 32 条未命中、rc=1；
#   把 spa.h 的 SPA_BLKPTRSHIFT 从 7 改成 9 ⇒ 该条未命中、rc=1。
#
# 样本：fixtures/70-citations.sh/red 是一个没有 verify-citations.sh 的仓，必须判红。没有 green：
# verify-citations.sh 读的是两个本机绝对路径下的源码树（FS_REFS、KERNEL_TREE 的默认值），装不进密封的样本目录。
#
#   bash .claude/gate.d/70-citations.sh [项目根]
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
cd "$ROOT" || exit 2
S=research/scripts/verify-citations.sh
[[ -f "$S" ]] || {
  echo "  ✗ 找不到 $S"
  echo "     → 怎么办：这一阶段靠它逐条复核引文。文件被挪走就改这里的路径，"
  echo "               还没写就先写它——缺了它，引文一条都没被验过。"
  exit 1; }
bash "$S"

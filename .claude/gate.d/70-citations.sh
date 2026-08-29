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
set -uo pipefail
S=research/scripts/verify-citations.sh
[[ -f "$S" ]] || { echo "  ✗ 找不到 $S"; exit 1; }
bash "$S"

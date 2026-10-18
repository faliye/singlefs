#!/usr/bin/env bash
# K3 的全格扫：两个会话开跑 / 收尾的六种先后，四个事件前各插一步（- L U C G），共 6 × 5^4 = 3750 段；每段一个临时仓。
# 用法：k3-sweep.sh <冻结副本> <工具副本> <草稿根> <并行数> [前 N 段]
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"; FROZEN="$1"; TOOLS="$2"; WORK="$3"; JOBS="$4"; LIMIT="${5:-0}"
gen() {
  for order in "a A b B" "a b A B" "a b B A" "b a A B" "b a B A" "b B a A"; do
    read -r e1 e2 e3 e4 <<<"$order"
    for o1 in - L U C G; do for o2 in - L U C G; do for o3 in - L U C G; do for o4 in - L U C G; do
      echo "$o1.$e1 $o2.$e2 $o3.$e3 $o4.$e4"
    done; done; done; done
  done
}
if [[ "$LIMIT" -gt 0 ]]; then gen | head -n "$LIMIT"; else gen; fi \
  | xargs -P "$JOBS" -I{} bash "$HERE/k3-concurrent.sh" "$FROZEN" "$TOOLS" "$WORK" "{}" | sort

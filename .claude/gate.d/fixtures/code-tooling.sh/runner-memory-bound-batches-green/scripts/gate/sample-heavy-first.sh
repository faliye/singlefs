#!/usr/bin/env bash
# 样本：登记的大自证：开跑留一个记号、睡 1 秒，这期间样本根的 .running/ 下还有别的记号就退 1（与别的自证叠在一批了）
[[ "${1:-}" == --selftest ]] || exit 2
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 2
mkdir -p .running
: > .running/heavy-first
sleep 1
others="$(ls .running | grep -vx heavy-first)"
rm -f .running/heavy-first
if [[ -n "$others" ]]; then
  echo "  ✗ heavy-first 的自证判错：与 $others 叠在一批了"
  exit 1
fi
echo "  ✓ heavy-first 的自证 1 条都判对"
exit 0

#!/usr/bin/env bash
# 样本：没登记的小自证：开跑留一个记号、睡 1 秒，这期间样本根的 .running/ 下有大自证的记号就退 1（与大自证叠在一批了）
[[ "${1:-}" == --selftest ]] || exit 2
cd "$(dirname "${BASH_SOURCE[0]}")/../.." || exit 2
mkdir -p .running
: > .running/small-second
sleep 1
heavy="$(ls .running | grep '^heavy-')"
rm -f .running/small-second
if [[ -n "$heavy" ]]; then
  echo "  ✗ small-second 的自证判错：与 $heavy 叠在一批了"
  exit 1
fi
echo "  ✓ small-second 的自证 1 条都判对"
exit 0

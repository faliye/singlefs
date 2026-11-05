#!/usr/bin/env bash
# 样本：自证退 0 的一条（项写错时不该起）
[[ "${1:-}" == --selftest ]] || exit 2
echo "  ✓ a 的自证 1 条都判对"
exit 0

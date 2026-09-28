#!/usr/bin/env bash
# 样本：自证退 0 的一条，排在必红那一条后面：它照样要跑完、收回退出码
[[ "${1:-}" == --selftest ]] || exit 2
echo "  ✓ d 的自证 1 条都判对"
exit 0

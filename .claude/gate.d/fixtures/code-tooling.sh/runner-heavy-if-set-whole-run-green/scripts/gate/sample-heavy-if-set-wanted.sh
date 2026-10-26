#!/usr/bin/env bash
# 样本：登记成 heavy-if-set:SINGLEFS_HEAVY_TESTS 的自证。整轮里变量该原样交下来：设着退 0（真驱动那一支），没设退 1
[[ "${1:-}" == --selftest ]] || exit 2
if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  echo "  ✗ 样本自证没拿到 SINGLEFS_HEAVY_TESTS：整轮里这个变量该原样交给登记成 heavy-if-set 的自证"
  exit 1
fi
echo "  ✓ 样本自证拿到了 SINGLEFS_HEAVY_TESTS=${SINGLEFS_HEAVY_TESTS}，走真驱动那一支"
exit 0

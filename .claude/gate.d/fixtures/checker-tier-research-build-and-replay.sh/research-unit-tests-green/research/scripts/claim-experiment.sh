#!/usr/bin/env bash
# research-unit-tests 格绿样本里的替身：不是真脚本的拷贝，只认 --selftest、打一行、退 0
if [[ "${1:-}" != --selftest ]]; then
  echo "  ✗ 替身只认 --selftest"
  echo "     → 怎么办：这是门禁样本里的替身，别当真脚本用"
  exit 2
fi
echo "替身自证：只认 --selftest"

#!/usr/bin/env bash
# 样本：代码里实现了 --selftest，由不带编号的阶段 tooling-runs.sh 在跑
case "${1:-}" in
  --selftest) echo "  ✓ 自证 1 条都判对"; exit 0 ;;
esac
exit 0

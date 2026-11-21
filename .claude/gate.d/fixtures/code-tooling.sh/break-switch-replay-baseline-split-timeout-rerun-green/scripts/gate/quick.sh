#!/usr/bin/env bash
# 样本脚本：另一份，基准趟当场过（与 slow.sh 的基准趟同时起，基准趟那一段外层 2 条 × 里层 4）；一条开关弄坏一格。
# 弄坏开关：
#   QUICK_BREAK=suffix  判后缀那一处不判
#       quick-suffix（「✗ 自检：判后缀那一格」）
if [[ "${1:-}" == --selftest ]]; then
  case "${QUICK_BREAK:-}" in
    suffix) echo "  ✗ 自检：判后缀那一格：应当红，实际绿"; exit 1 ;;
  esac
  echo "  ✓ 自检通过（查了 1 格）"
fi

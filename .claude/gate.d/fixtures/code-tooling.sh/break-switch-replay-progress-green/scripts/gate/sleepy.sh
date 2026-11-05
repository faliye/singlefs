#!/usr/bin/env bash
# 样本脚本：每一趟自证（不设开关的基准趟与设上开关的开关趟）都先睡 3 秒，比样本的进度间隔（1 秒）长，复跑那一格中途要报进度
# 弄坏开关：
#   SLEEPY_BREAK=prefix  判前缀那一处不判
#       sleepy-prefix（「✗ 自检：判前缀那一格」）
#   SLEEPY_BREAK=suffix  判后缀那一处不判
#       sleepy-suffix（「✗ 自检：判后缀那一格」）
if [[ "${1:-}" == --selftest ]]; then
  sleep 3
  case "${SLEEPY_BREAK:-}" in
    prefix) echo "  ✗ 自检：判前缀那一格：应当红，实际绿"; exit 1 ;;
    suffix) echo "  ✗ 自检：判后缀那一格：应当红，实际绿"; exit 1 ;;
  esac
  echo "  ✓ 自检通过（查了 2 格）"
fi

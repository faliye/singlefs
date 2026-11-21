#!/usr/bin/env bash
# 样本脚本：基准趟（不设开关）头一趟在样本根留一个记号 .baseline-started 再睡过基准趟限时（3 秒）；再起一趟看到记号就当场过。
#   一份、线程 4：基准趟拿满 4 个线程还到点，门禁不重跑、这一份判红；重跑了就过、判成绿。一条开关弄坏一格。
# 弄坏开关：
#   STALL_BREAK=prefix  判前缀那一处不判
#       stall-prefix（「✗ 自检：判前缀那一格」）
if [[ "${1:-}" == --selftest ]]; then
  if [[ -z "${STALL_BREAK:-}" && ! -e .baseline-started ]]; then
    : > .baseline-started
    exec sleep 30
  fi
  case "${STALL_BREAK:-}" in
    prefix) echo "  ✗ 自检：判前缀那一格：应当红，实际绿"; exit 1 ;;
  esac
  echo "  ✓ 自检通过（查了 1 格）"
fi

#!/usr/bin/env bash
# 样本脚本：基准趟（不设开关）读拿到的里层线程数 SINGLEFS_THREAD_CAP：等于登记的 8 当场过；小于 8（两份基准趟同时起、各分到 4）就睡过基准趟限时（3 秒），
#   等门禁拿满 8 个线程重跑那一趟才过；读不到就打一行没点名的 ✗ 并退 1。开关趟不读它，一条开关弄坏一格。
# 弄坏开关：
#   SLOW_BREAK=prefix  判前缀那一处不判
#       slow-prefix（「✗ 自检：判前缀那一格」）
if [[ "${1:-}" == --selftest ]]; then
  if [[ -z "${SLOW_BREAK:-}" ]]; then
    case "${SINGLEFS_THREAD_CAP:-}" in
      8) ;;
      [1-7]) exec sleep 30 ;;
      *) echo "  ✗ 自检：里层线程数：应当是 1 到 8 之间的数，实际 ${SINGLEFS_THREAD_CAP:-没设}"; exit 1 ;;
    esac
  fi
  case "${SLOW_BREAK:-}" in
    prefix) echo "  ✗ 自检：判前缀那一格：应当红，实际绿"; exit 1 ;;
  esac
  echo "  ✓ 自检通过（查了 1 格）"
fi

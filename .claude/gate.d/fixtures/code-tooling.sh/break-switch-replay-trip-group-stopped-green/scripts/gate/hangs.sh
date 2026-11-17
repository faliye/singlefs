#!/usr/bin/env bash
# 样本脚本：开关的头一趟（样本根里还没有 .hung）派一个不理 TERM 的孙进程（进程号写进 grandchild.pid；样本根还在、又没满 20 秒就一直在），
# 自己睡 10 秒，比每条开关的限时（3 秒）长：开关复跑到点要停整组，TERM 停不掉孙进程、过宽限要 KILL。
# 再起的每一趟（重跑）起跑时看那个孙进程还活着没有：活着就多打一行没点名的失败行。
# 弄坏开关：
#   HANGS_BREAK=prefix  判前缀那一处不判
#       hangs-prefix（「✗ 自检：判前缀那一格」）
if [[ "${1:-}" == --selftest ]]; then
  if [[ "${HANGS_BREAK:-}" == prefix && ! -e .hung ]]; then
    : > .hung
    ( trap '' TERM; echo "$BASHPID" > grandchild.pid; while [[ -e .hung ]] && (( SECONDS < 20 )); do sleep 0.2; done ) &
    sleep 10
  fi
  if [[ -s grandchild.pid ]]; then
    state="$(awk '{print $3}' "/proc/$(<grandchild.pid)/stat" 2>/dev/null)"
    if [[ -n "$state" && "$state" != Z && "$state" != X ]]; then
      echo "  ✗ 自检：前一趟派的孙进程到这一趟起跑时还活着（进程号 $(<grandchild.pid)）"
    fi
  fi
  case "${HANGS_BREAK:-}" in
    prefix) echo "  ✗ 自检：判前缀那一格：应当红，实际绿"; exit 1 ;;
  esac
  echo "  ✓ 自检通过（查了 1 格）"
fi

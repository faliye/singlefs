#!/usr/bin/env bash
# 样本脚本：四条开关各弄坏一格。
# 被打断的那一趟（环境里有 CODE_TOOLING_SAMPLE_STOP_ARMED）里，每条开关趟像被内存上限杀掉：退 137、不打失败行；基准趟照常过。
# 别的趟里，开关趟起跑时在 running/ 下登一个以进程号为名的文件、退出前删掉，最多等 6 秒看有没有别的趟同时登着：看到了就再留 1 秒（让对方也看得到），
# 6 秒都没看到就多打一行带自己开关值的没点名的失败行（几条开关一起多红的不是同一行、不被认成汇总行）。
# 弄坏开关：
#   LANE_BREAK=prefix  判前缀那一处不判
#       lane-prefix（「✗ 自检：判前缀那一格」）
#   LANE_BREAK=suffix  判后缀那一处不判
#       lane-suffix（「✗ 自检：判后缀那一格」）
#   LANE_BREAK=infix  判中缀那一处不判
#       lane-infix（「✗ 自检：判中缀那一格」）
#   LANE_BREAK=stem  判词干那一处不判
#       lane-stem（「✗ 自检：判词干那一格」）
others_running() {
  local entry
  for entry in running/*; do
    [[ -e "$entry" && "${entry##*/}" != "$$" ]] && return 0
  done
  return 1
}
if [[ "${1:-}" == --selftest ]]; then
  if [[ -n "${LANE_BREAK:-}" ]]; then
    if [[ -n "${CODE_TOOLING_SAMPLE_STOP_ARMED:-}" ]]; then
      echo "  自证被杀：${LANE_BREAK}"
      exit 137
    fi
    mkdir -p running
    : > "running/$$"
    trap 'rm -f "running/$$"' EXIT
    seen=0
    for ((tick = 0; tick < 60; tick++)); do
      if others_running; then seen=1; break; fi
      sleep 0.1
    done
    if ((seen)); then
      sleep 1
    else
      echo "  ✗ 自检：这一趟起跑后 6 秒内没有别的趟同时在跑（这一趟：${LANE_BREAK}）"
    fi
  fi
  case "${LANE_BREAK:-}" in
    prefix) echo "  ✗ 自检：判前缀那一格：应当红，实际绿"; exit 1 ;;
    suffix) echo "  ✗ 自检：判后缀那一格：应当红，实际绿"; exit 1 ;;
    infix) echo "  ✗ 自检：判中缀那一格：应当红，实际绿"; exit 1 ;;
    stem) echo "  ✗ 自检：判词干那一格：应当红，实际绿"; exit 1 ;;
  esac
  echo "  ✓ 自检通过（查了 4 格）"
fi

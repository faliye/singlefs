#!/usr/bin/env bash
# 样本脚本：每条开关的头一趟（样本根里还没有 .seen-<开关值>）睡 1 秒、没红（退 0），开关复跑要重跑；重跑那一趟红在点名的失败行上。
# 每一趟起跑时在 running/ 下登一个以进程号为名的文件、退出前删掉；重跑那一趟起跑时与睡 1 秒之后各看一次 running/ 里有没有别的趟，
# 看到过就记下（.overlap-seen），此后每一趟都多打一行没点名的失败行（各趟写上自己的开关值，不被认成汇总行）。
# 弄坏开关：
#   FLAKY_BREAK=prefix  判前缀那一处不判
#       flaky-prefix（「✗ 自检：判前缀那一格」）
#   FLAKY_BREAK=suffix  判后缀那一处不判
#       flaky-suffix（「✗ 自检：判后缀那一格」）
others_running() {
  local entry
  for entry in running/*; do
    [[ -e "$entry" && "${entry##*/}" != "$$" ]] && return 0
  done
  return 1
}
if [[ "${1:-}" == --selftest ]]; then
  mkdir -p running
  : > "running/$$"
  trap 'rm -f "running/$$"' EXIT
  if [[ -n "${FLAKY_BREAK:-}" ]]; then
    if [[ ! -e ".seen-$FLAKY_BREAK" ]]; then
      : > ".seen-$FLAKY_BREAK"
      sleep 1
      echo "  ✓ 自检通过（查了 2 格）"
      exit 0
    fi
    if others_running; then : > .overlap-seen; fi
    sleep 1
    if others_running; then : > .overlap-seen; fi
  fi
  [[ ! -e .overlap-seen ]] || echo "  ✗ 自检：重跑时另有别的趟在跑（这一趟：${FLAKY_BREAK:-不设开关}）"
  case "${FLAKY_BREAK:-}" in
    prefix) echo "  ✗ 自检：判前缀那一格：应当红，实际绿"; exit 1 ;;
    suffix) echo "  ✗ 自检：判后缀那一格：应当红，实际绿"; exit 1 ;;
  esac
  [[ ! -e .overlap-seen ]] || exit 1
  echo "  ✓ 自检通过（查了 2 格）"
fi

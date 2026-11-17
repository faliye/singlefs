#!/usr/bin/env bash
# 样本脚本：开关趟睡 3 秒，比 flaky.sh 的头一趟长：要重跑的那几条在它收完之前起，就与它同时在跑。
# 每一趟起跑时在 running/ 下登一个以进程号为名的文件、退出前删掉；样本根里有 .overlap-seen 时多打一行没点名的失败行。
# 弄坏开关：
#   STEADY_BREAK=prefix  判前缀那一处不判
#       steady-prefix（「✗ 自检：判前缀那一格」）
if [[ "${1:-}" == --selftest ]]; then
  mkdir -p running
  : > "running/$$"
  trap 'rm -f "running/$$"' EXIT
  [[ -z "${STEADY_BREAK:-}" ]] || sleep 3
  [[ ! -e .overlap-seen ]] || echo "  ✗ 自检：重跑时另有别的趟在跑"
  case "${STEADY_BREAK:-}" in
    prefix) echo "  ✗ 自检：判前缀那一格：应当红，实际绿"; exit 1 ;;
  esac
  [[ ! -e .overlap-seen ]] || exit 1
  echo "  ✓ 自检通过（查了 1 格）"
fi

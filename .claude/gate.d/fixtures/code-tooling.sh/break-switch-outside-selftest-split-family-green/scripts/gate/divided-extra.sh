#!/usr/bin/env bash
# 样本脚本：名字与 divided.sh 起头相同，divided.sh 的代码里没有提它，是独立的脚本，自己守住自己的开关
# 弄坏开关：
#   EXTRA_BREAK=skip  判法整段跳过
#       extra-red（「✗ 自检：该红没红」）
if [[ -n "${EXTRA_BREAK:-}" && "${1:-}" != --selftest ]]; then
  echo "  ✗ EXTRA_BREAK 只给 --selftest 证红用，这一趟不是自证" >&2
  exit 2
fi
echo "样本：正常"

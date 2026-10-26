#!/usr/bin/env bash
# 样本脚本：关守卫项不叫 guard-off，说明里写着「平常跑」「不拒」；守卫按逗号拆开认它
# 弄坏开关：
#   DESCRIBED_BREAK=quiet  读到就多打一行
#       described-quiet（「✗ 自检：多打的一行」）
#   DESCRIBED_BREAK=open-door  平常跑设了也不拒（改前的样子）
#       described-door（「✗ 自检：平常跑没拒」）
if [[ -n "${DESCRIBED_BREAK:-}" && "${1:-}" != --selftest && ",${DESCRIBED_BREAK}," != *",open-door,"* ]]; then
  echo "  ✗ DESCRIBED_BREAK 只给 --selftest 证红用，这一趟不是自证" >&2
  echo "     → 怎么办：去掉 DESCRIBED_BREAK 再跑" >&2
  exit 2
fi
echo "样本：照常跑"

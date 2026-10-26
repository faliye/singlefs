#!/usr/bin/env python3
# 样本脚本：守卫按逗号拆开，里面有 guard-off 就放行（关守卫项与别的项同写也借道）
# 弄坏开关：
#   SPLIT_BREAK=skip  判法整段跳过
#       split-skip（「✗ 自检：该红没红」）
#   SPLIT_BREAK=guard-off  守卫整段不判
#       split-guard（「✗ 自检：守卫没拒」）
import os
import sys

value = os.environ.get("SPLIT_BREAK", "")
if value and sys.argv[1:] != ["--selftest"] and "guard-off" not in value.split(","):
    print("  ✗ SPLIT_BREAK 只给 --selftest 证红用，这一趟不是自证", file=sys.stderr)
    print("     → 怎么办：去掉 SPLIT_BREAK 再跑", file=sys.stderr)
    sys.exit(2)
if sys.argv[1:] and sys.argv[1:] != ["--selftest"]:
    print("用法错：认不得的参数", file=sys.stderr)
    sys.exit(2)
print("样本：正常")

#!/usr/bin/env python3
# 样本脚本：两条开关各红一格；复用三问的 change.sh 改 expected_cells 的写法（闭包里的一个函数单元，打出来的照旧）
# 弄坏开关：
#   ALPHA_BREAK=drop  丢掉一格
#       alpha-drop（「✗ 自检：丢了的那一格」）
#   ALPHA_BREAK=swap  两格换了位置
#       alpha-swap（「✗ 自检：换了位置的那一格」）
import os
import sys


def expected_cells():
    return 2


def selftest():
    broken = os.environ.get("ALPHA_BREAK")
    if broken == "drop":
        print(f"  ✗ 自检：丢了的那一格：应当 {expected_cells()}，实际 1")
        return 1
    if broken == "swap":
        print("  ✗ 自检：换了位置的那一格：应当 甲乙，实际 乙甲")
        return 1
    print(f"  ✓ 自检通过（查了 {expected_cells()} 格）")
    return 0


if sys.argv[1:] == ["--selftest"]:
    sys.exit(selftest())

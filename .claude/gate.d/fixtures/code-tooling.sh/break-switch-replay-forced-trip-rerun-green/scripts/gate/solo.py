#!/usr/bin/env python3
# 样本脚本：四条开关各红一格
# 弄坏开关：
#   SOLO_BREAK=sweep  整理的那一格
#       solo-sweep（「✗ 自检：整理的那一格」）
#   SOLO_BREAK=drop  丢掉的那一格
#       solo-drop（「✗ 自检：丢掉的那一格」）
#   SOLO_BREAK=swap  换位置的那一格
#       solo-swap（「✗ 自检：换位置的那一格」）
#   SOLO_BREAK=skip  跳过的那一格
#       solo-skip（「✗ 自检：跳过的那一格」）
import os
import sys

CELLS = {"sweep": "整理", "drop": "丢掉", "swap": "换位置", "skip": "跳过"}
if sys.argv[1:] == ["--selftest"]:
    broken = os.environ.get("SOLO_BREAK", "")
    if broken in CELLS:
        print(f"  ✗ 自检：{CELLS[broken]}的那一格")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

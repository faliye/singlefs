#!/usr/bin/env python3
# 样本脚本：开关设上之后自证照样退 0，开关没弄坏什么
# 弄坏开关：
#   SILENT_BREAK=noop  什么都不弄坏
#       silent-noop（「✗ 自检：多出来的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    os.environ.get("SILENT_BREAK")
    print("  ✓ 自检通过")
    sys.exit(0)

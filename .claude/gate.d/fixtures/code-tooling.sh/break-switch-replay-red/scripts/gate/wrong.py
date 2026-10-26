#!/usr/bin/env python3
# 样本脚本：开关打开后自证红了，红的失败行却不是文件头点名的那一句
# 弄坏开关：
#   WRONG_BREAK=swap  把两格对调
#       wrong-swap（「✗ 自检：本该红的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("WRONG_BREAK") == "swap":
        print("  ✗ 自检：红在了别处：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

#!/usr/bin/env python3
# 样本脚本：开关打开后红的两格里，一格接在单独的 \r 收尾的进度行后面（按文本收时 \r 是换行，那一格自成一行），文件头只点名另一格
# 弄坏开关：
#   CARRIAGE_BREAK=progress  两格一起弄坏，进度行用 \r 收尾
#       carriage-progress（「✗ 自检：点名的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("CARRIAGE_BREAK") == "progress":
        sys.stdout.write("  进度 1/2\r  ✗ 自检：回车后面那一格：应当 1，实际 0\n")
        sys.stdout.write("  ✗ 自检：点名的那一格：应当 1，实际 0\n")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

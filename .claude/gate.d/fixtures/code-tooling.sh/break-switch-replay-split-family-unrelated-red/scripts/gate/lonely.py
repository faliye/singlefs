#!/usr/bin/env python3
# 样本脚本：文件头写了一条开关，代码里不读它；同目录的 lonely_extra.py 读，但这份的代码没有载入它，是独立的脚本
# 弄坏开关：
#   LONELY_BREAK=drop  丢掉一格
#       lonely-drop（「✗ 自检：丢了的那一格」）
import sys

if sys.argv[1:] == ["--selftest"]:
    print("  ✓ 自检通过")
    sys.exit(0)

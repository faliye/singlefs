#!/usr/bin/env python3
# 样本脚本：点名的项只写了名字，没有「」括起来的失败行原文，复跑没有东西可比
# 弄坏开关：
#   NOMINAL_BREAK=bare  丢掉一格
#       nominal-bare 自证里红的是丢掉的那一格
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("NOMINAL_BREAK") == "bare":
        print("  ✗ 自检：丢掉的那一格")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

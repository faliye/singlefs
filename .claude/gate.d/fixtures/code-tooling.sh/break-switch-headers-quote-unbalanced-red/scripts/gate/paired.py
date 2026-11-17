#!/usr/bin/env python3
"""样本脚本：开关那一条点名的项里「」成对（里面套着一层成对的），这一格判它写全
  PAIRED_BREAK=skip  判法整段跳过
    「✗ 自检：判法那一格（含「嵌套」一层）」、
    「✗ 自检：另一格」
"""
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("PAIRED_BREAK") == "skip":
        print("  ✗ 自检：判法那一格：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

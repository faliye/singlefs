#!/usr/bin/env python3
# 样本脚本：开关打开后自证红了五格，文件头只点名其中一格、另点名了一格没红：多红四格（每行都超过八十个字）、点名的一格没红，日志里只列前三行并截断，完整的失败行与点名的项在明细文件里
# 弄坏开关：
#   CROWDED_BREAK=five  五格一起弄坏
#       crowded-five（「✗ 自检：点名的那一格」、「✗ 自检：点了没红的那一格」）
import os
import sys

TAILS = {"缓存": "尾段标记缓存", "索引": "尾段标记索引", "日志": "尾段标记日志", "校验": "尾段标记校验"}

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("CROWDED_BREAK") == "five":
        print("  ✗ 自检：点名的那一格：应当 1，实际 0")
        for name, tail in TAILS.items():
            print(f"  ✗ 自检：多红的{name}格：应当 1，实际 0；这一行故意写得很长，好让日志里按八十个字截断时把末尾那一段砍掉，而明细文件里要把整行原样留着，一个字都不能少，包括最后这一段的标记：{tail}")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

#!/usr/bin/env python3
# 样本脚本：不设开关时什么失败行都不打；三条开关各红一格、另带同一句汇总行（三条开关趟里都多红的那一句，不要求点名）
# 弄坏开关：
#   TALLY_BREAK=prefix  判前缀那一处不判
#       tally-prefix（「✗ 自检：判前缀那一格」）
#   TALLY_BREAK=suffix  判后缀那一处不判
#       tally-suffix（「✗ 自检：判后缀那一格」）
#   TALLY_BREAK=length  判长度那一处不判
#       tally-length（「✗ 自检：判长度那一格」）
import os
import sys

CELLS = {"prefix": "判前缀那一格", "suffix": "判后缀那一格", "length": "判长度那一格"}

if sys.argv[1:] == ["--selftest"]:
    broken = os.environ.get("TALLY_BREAK")
    if broken in CELLS:
        print(f"  ✗ 自检：{CELLS[broken]}：应当 1，实际 0")
        print("  ✗ 自检：1 格不对（查了 3 格）")
        sys.exit(1)
    print("  ✓ 自检通过（查了 3 格）")
    sys.exit(0)

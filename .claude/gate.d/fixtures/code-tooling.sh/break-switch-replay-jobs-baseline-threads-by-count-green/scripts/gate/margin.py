#!/usr/bin/env python3
# 样本脚本：并行度样本用的自证，不占内存、没有选格手段，一条开关弄坏一格；每一趟起跑先核拿到的里层线程数（SINGLEFS_THREAD_CAP）：
#   基准趟（不设开关）是 8 ÷ 两份同时起的基准趟 = 4，开关趟是分过的 2，不是就打一行没点名的 ✗ 并退 1（基准趟没过、或开关趟红在没点名的一格，整份判红）
# 弄坏开关：
#   MARGIN_BREAK=width  判边距那一处不判
#       margin-width（「✗ 自检：判边距那一格」）
import os
import sys

CELLS = {"width": "判边距那一格"}

if sys.argv[1:] == ["--selftest"]:
    broken = os.environ.get("MARGIN_BREAK", "")
    expected_thread_cap = "2" if broken else "4"
    if os.environ.get("SINGLEFS_THREAD_CAP") != expected_thread_cap:
        print(f"  ✗ 自检：里层线程数：应当 {expected_thread_cap}，实际 {os.environ.get('SINGLEFS_THREAD_CAP')}")
        sys.exit(1)
    if broken in CELLS:
        print(f"  ✗ 自检：{CELLS[broken]}：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过（查了 1 格）")
    sys.exit(0)

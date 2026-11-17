#!/usr/bin/env python3
# 样本脚本：一条开关红一格。跑过哪几趟记在样本根的 .trips-seen/ 下：同一趟起第二次就退 3、打一句没被点名的 ✗（落了盘的趟不该再起），这一格判红。
# 弄坏开关：
#   BETA_BREAK=skip  跳过一格
#       beta-skip（「✗ 自检：跳过的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    broken = os.environ.get("BETA_BREAK", "")
    os.makedirs(".trips-seen", exist_ok=True)
    marker = os.path.join(".trips-seen", "beta-" + (broken or "baseline"))
    if os.path.exists(marker):
        print(f"  ✗ 这一趟起了第二次：beta {broken or '基准趟'}")
        sys.exit(3)
    with open(marker, "w", encoding="utf-8"):
        pass
    if broken == "skip":
        print("  ✗ 自检：跳过的那一格：应当判过，实际没判")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

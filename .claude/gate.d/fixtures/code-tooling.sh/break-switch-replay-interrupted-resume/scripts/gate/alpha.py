#!/usr/bin/env python3
# 样本脚本：第一条开关退 2 并点名开关（守卫），另两条各红一格加一句汇总行。
# 跑过哪几趟记在样本根的 .trips-seen/ 下：同一趟起第二次就退 3、打一句没被点名的 ✗（落了盘的趟不该再起），这一格判红。
# 弄坏开关：
#   ALPHA_BREAK=guarded  外面设开关当场拒
#       alpha-guarded（退 2，判守卫）
#   ALPHA_BREAK=drop  丢掉一格
#       alpha-drop（「✗ 自检：丢了的那一格」）
#   ALPHA_BREAK=swap  两格换了位置
#       alpha-swap（「✗ 自检：换了位置的那一格」）
import os
import sys


def trip_ran_before(name):
    os.makedirs(".trips-seen", exist_ok=True)
    marker = os.path.join(".trips-seen", name)
    if os.path.exists(marker):
        return True
    with open(marker, "w", encoding="utf-8"):
        pass
    return False


def selftest():
    broken = os.environ.get("ALPHA_BREAK", "")
    if trip_ran_before("alpha-" + (broken or "baseline")):
        print(f"  ✗ 这一趟起了第二次：alpha {broken or '基准趟'}")
        return 3
    if broken == "guarded":
        print("  ALPHA_BREAK 只在自证自己的开关轮里认，外面设上当场拒")
        return 2
    if broken == "drop":
        print("  ✗ 自检：丢了的那一格：应当 2，实际 1")
        print("  ✗ 自检：1 格判错")
        return 1
    if broken == "swap":
        print("  ✗ 自检：换了位置的那一格：应当 甲乙，实际 乙甲")
        print("  ✗ 自检：1 格判错")
        return 1
    print("  ✓ 自检通过（查了 2 格）")
    return 0


if sys.argv[1:] == ["--selftest"]:
    sys.exit(selftest())

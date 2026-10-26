#!/usr/bin/env python3
# 样本脚本：读了弄坏开关；认不得的参数退 2，但没点名开关、没说只在自证里认
# 弄坏开关：
#   LOUD_BREAK=skip  读到就跳过判法
#       loud-red（「✗ 自检：跳过了判法」）
import os
import sys

if os.environ.get("LOUD_BREAK"):
    pass
if len(sys.argv) > 1:
    print("用法错：认不得的参数", file=sys.stderr)
    sys.exit(2)

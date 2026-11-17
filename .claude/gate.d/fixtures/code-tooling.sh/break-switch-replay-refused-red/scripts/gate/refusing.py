#!/usr/bin/env python3
# 样本脚本：自证里起的命令取自环境变量 REFUSING_COMMAND（没设是 true），单元闭包折不出那条命令、算键拒算；照跑的话一条开关红一格、点名对得上
# 弄坏开关：
#   REFUSING_BREAK=skip  跳过一格
#       refusing-skip（「✗ 自检：跳过的那一格」）
import os
import subprocess
import sys


def configured_command_exit():
    return subprocess.run(os.environ.get("REFUSING_COMMAND", "true").split(), stdin=subprocess.DEVNULL).returncode


if sys.argv[1:] == ["--selftest"]:
    if configured_command_exit() != 0:
        print("  ✗ 自检：起的命令没成：应当退 0")
        sys.exit(1)
    if os.environ.get("REFUSING_BREAK") == "skip":
        print("  ✗ 自检：跳过的那一格：应当判过，实际没判")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

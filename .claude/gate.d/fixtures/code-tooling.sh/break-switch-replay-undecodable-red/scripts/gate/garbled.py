#!/usr/bin/env python3
# 样本脚本：一条开关下自证吐出不合法的 UTF-8（失败行原文后面跟着坏字节），另一条开关照常红在点名的失败行上
# 弄坏开关：
#   GARBLED_BREAK=bytes  吐坏字节的那一格
#       garbled-bytes（「✗ 自检：吐了坏字节的那一格」）
#   GARBLED_BREAK=plain  照常丢掉一格
#       garbled-plain（「✗ 自检：照常红的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    mode = os.environ.get("GARBLED_BREAK")
    if mode == "bytes":
        sys.stdout.flush()
        sys.stdout.buffer.write("  ✗ 自检：吐了坏字节的那一格：".encode("utf-8") + b"\xff\xfe\xe2\x9c\n")
        sys.stdout.buffer.flush()
        sys.exit(1)
    if mode == "plain":
        print("  ✗ 自检：照常红的那一格：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

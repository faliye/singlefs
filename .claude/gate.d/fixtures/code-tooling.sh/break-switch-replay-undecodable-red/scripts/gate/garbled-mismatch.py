#!/usr/bin/env python3
# 样本脚本：开关打开后自证吐坏字节，红的那一格又不是文件头点名的那一格：坏字节与少红都要报出来（容错解码之后照比点名）
# 弄坏开关：
#   GARBLED_MISMATCH_BREAK=bytes  红在别处、带坏字节
#       garbled-mismatch-bytes（「✗ 自检：点名的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("GARBLED_MISMATCH_BREAK") == "bytes":
        sys.stdout.flush()
        sys.stdout.buffer.write("  ✗ 自检：红在别处的那一格：".encode("utf-8") + b"\xff\xfe\n")
        sys.stdout.buffer.flush()
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

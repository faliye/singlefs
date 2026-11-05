#!/usr/bin/env python3
# 样本脚本：自证只在临时 prune-server 上跑（样本根的要临时服务的表登记成 required；真脚本开跑前调客户端的核对，这里拿 SAMPLE_TEMPORARY_SERVICE 当替身），
# 不在包装底下（样本的假包装 scripts/prune-server/prune_server_client_selftest.py 设这个变量）就退 2、一格都不判
# 弄坏开关：
#   NEEDS_SERVICE_BREAK=drop  丢掉一格
#       needs-service-drop（「✗ 自检：丢了的那一格」）
import os
import sys

if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("SAMPLE_TEMPORARY_SERVICE") != "1":
        print("  ✗ needs-service.py 自证没跑：没有临时 prune-server")
        sys.exit(2)
    if os.environ.get("NEEDS_SERVICE_BREAK") == "drop":
        print("  ✗ 自检：丢了的那一格：应当 1，实际 0")
        sys.exit(1)
    print("  ✓ 自检通过")
    sys.exit(0)

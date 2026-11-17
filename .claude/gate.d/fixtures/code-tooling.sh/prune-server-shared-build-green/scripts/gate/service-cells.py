#!/usr/bin/env python3
# 样本脚本：自证只在临时 prune-server 上跑（样本根的要临时服务的表登记成 required；真脚本开跑前调客户端的核对，这里拿 SAMPLE_TEMPORARY_SERVICE 当替身），
# 不在包装底下（样本的假包装 scripts/prune-server/prune_server_client_selftest.py 设这个变量）就退 2、一格都不判。
# 自证分两格（prepare、consume）；选格：SERVICECELLS_SELFTEST_CELLS=<格名,…> 只跑点名的格，写错退 2 并列出认得的格。
# 弄坏开关：
#   SERVICECELLS_BREAK=drop  丢掉一格
#       service-cells-drop（「✗ 自检：丢了的那一格」）
import os
import sys

CELLS = ("prepare", "consume")
state = {}


def prepare():
    state["prepared"] = "甲"
    return True


def consume():
    prepare()
    return state.get("prepared") == "甲"


if sys.argv[1:] == ["--selftest"]:
    if os.environ.get("SAMPLE_TEMPORARY_SERVICE") != "1":
        print("  ✗ service-cells.py 自证没跑：没有临时 prune-server")
        sys.exit(2)
    selected = [name for name in os.environ.get("SERVICECELLS_SELFTEST_CELLS", "").split(",") if name] or list(CELLS)
    unknown = [name for name in selected if name not in CELLS]
    if unknown:
        print(f"  ✗ 自证：SERVICECELLS_SELFTEST_CELLS 里点名的 {'、'.join(unknown)} 不是这份自证的格")
        print(f"     → 怎么办：格名照失败行「✗ 自证：<格名>：」写，逗号隔开：{'、'.join(CELLS)}")
        sys.exit(2)
    if os.environ.get("SERVICECELLS_BREAK") == "drop":
        print("  ✗ 自检：丢了的那一格：应当 1，实际 0")
        sys.exit(1)
    checks = {"prepare": prepare, "consume": consume}
    failed = [name for name in CELLS if name in selected and not checks[name]()]
    for name in failed:
        print(f"  ✗ 自证：{name}：应当读到 甲，实际没读到")
    if failed:
        sys.exit(1)
    print(f"  ✓ 自证通过（查了 {len(selected)} 格）")
    sys.exit(0)

#!/usr/bin/env python3
# 样本脚本：自证分两格（prepare、consume）；选格：TWOCELLS_SELFTEST_CELLS=<格名,…> 只跑点名的格，写错退 2 并列出认得的格。
# consume 那一格自己先调 prepare 建它要读的全局变量：整份跑与单跑哪一格都绿。
# 并行度样本：每一趟（整份与单跑）起跑先核拿到的里层线程数（SINGLEFS_THREAD_CAP）是 2，不是就退 1（失败行不点名哪一格，整份那一趟比不了，每格判红）。
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
    selected = [name for name in os.environ.get("TWOCELLS_SELFTEST_CELLS", "").split(",") if name] or list(CELLS)
    unknown = [name for name in selected if name not in CELLS]
    if unknown:
        print(f"  ✗ 自证：TWOCELLS_SELFTEST_CELLS 里点名的 {'、'.join(unknown)} 不是这份自证的格")
        print(f"     → 怎么办：格名照失败行「✗ 自证：<格名>：」写，逗号隔开：{'、'.join(CELLS)}")
        sys.exit(2)
    if os.environ.get("SINGLEFS_THREAD_CAP") != "2":
        print(f"  ✗ 自证：里层线程数：应当 2，实际 {os.environ.get('SINGLEFS_THREAD_CAP')}")
        sys.exit(1)
    checks = {"prepare": prepare, "consume": consume}
    failed = [name for name in CELLS if name in selected and not checks[name]()]
    for name in failed:
        print(f"  ✗ 自证：{name}：应当读到 甲，实际没读到")
    if failed:
        sys.exit(1)
    print(f"  ✓ 自证通过（查了 {len(selected)} 格）")
    sys.exit(0)

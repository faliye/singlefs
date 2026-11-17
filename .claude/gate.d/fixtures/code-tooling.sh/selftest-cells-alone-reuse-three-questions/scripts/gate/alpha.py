#!/usr/bin/env python3
# 样本脚本：自证分两格（prefix、suffix），各自建自己的输入；选格：ALPHA_SELFTEST_CELLS=<格名,…> 只跑点名的格，写错退 2 并列出认得的格。
# 复用三问的 change.sh 改 expected_length 的写法（闭包里的一个函数单元，判出来的照旧）
import os
import sys

CELLS = ("prefix", "suffix")


def expected_length():
    return 2


def prefix():
    return len("甲乙") == expected_length()


def suffix():
    return "甲乙".endswith("乙")


if sys.argv[1:] == ["--selftest"]:
    selected = [name for name in os.environ.get("ALPHA_SELFTEST_CELLS", "").split(",") if name] or list(CELLS)
    unknown = [name for name in selected if name not in CELLS]
    if unknown:
        print(f"  ✗ 自证：ALPHA_SELFTEST_CELLS 里点名的 {'、'.join(unknown)} 不是这份自证的格")
        print(f"     → 怎么办：格名照失败行「✗ 自证：<格名>：」写，逗号隔开：{'、'.join(CELLS)}")
        sys.exit(2)
    checks = {"prefix": prefix, "suffix": suffix}
    failed = [name for name in CELLS if name in selected and not checks[name]()]
    for name in failed:
        print(f"  ✗ 自证：{name}：判错")
    if failed:
        sys.exit(1)
    print(f"  ✓ 自证通过（查了 {len(selected)} 格）")
    sys.exit(0)

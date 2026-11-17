#!/usr/bin/env python3
# 样本脚本：自证分两格（alpha、beta）；环境变量 TIDY_SELFTEST_CELLS=<格名,…> 只跑点名的格。
# 每条开关只弄坏点名的那一格，指定范围与不指定范围下都是绿。
# 弄坏开关：
#   TIDY_BREAK=alpha  alpha 那一格判错
#       tidy-alpha（「✗ 自证：alpha：判前缀」）
#   TIDY_BREAK=beta  beta 那一格判错
#       tidy-beta（「✗ 自证：beta：判后缀」）
import os
import sys

CELLS = {"alpha": "判前缀那一格", "beta": "判后缀那一格"}

if sys.argv[1:] == ["--selftest"]:
    selected = [name for name in os.environ.get("TIDY_SELFTEST_CELLS", "").split(",") if name] or list(CELLS)
    failed = [name for name in selected if name == os.environ.get("TIDY_BREAK", "")]
    for name in failed:
        print(f"  ✗ 自证：{name}：{CELLS[name]}：应当 1，实际 0")
    if failed:
        sys.exit(1)
    print(f"  ✓ 自证通过（查了 {len(selected)} 格）")
    sys.exit(0)

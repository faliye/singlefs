#!/usr/bin/env python3
# 样本脚本：自证分四格（alpha、beta、gamma、delta），失败行写成「✗ 自证 <格名>：」；环境变量 CELLS_SELFTEST_CELLS=<格名,…> 只跑点名的格（单跑一格与它在全部格里跑判定相同）。
# 两条开关各弄坏点名的一格，另带一格没点名的也弄坏：整份起才看得到没点名的那一格红了。
# 弄坏开关：
#   CELLS_BREAK=alpha  alpha 那一格判错，gamma 那一格跟着判错
#       cells-alpha（「✗ 自证 alpha：判前缀」）
#   CELLS_BREAK=beta  beta 那一格判错，delta 那一格跟着判错
#       cells-beta（「✗ 自证 beta：判后缀」）
import os
import sys

CELLS = {"alpha": "判前缀那一格", "beta": "判后缀那一格", "gamma": "判长度那一格", "delta": "判大小写那一格"}
BROKEN_CELLS = {"alpha": ("alpha", "gamma"), "beta": ("beta", "delta")}

if sys.argv[1:] == ["--selftest"]:
    selected = [name for name in os.environ.get("CELLS_SELFTEST_CELLS", "").split(",") if name] or list(CELLS)
    failed = [name for name in selected if name in BROKEN_CELLS.get(os.environ.get("CELLS_BREAK", ""), ())]
    for name in failed:
        print(f"  ✗ 自证 {name}：{CELLS[name]}：应当 1，实际 0")
    if failed:
        sys.exit(1)
    print(f"  ✓ 自证通过（查了 {len(selected)} 格）")
    sys.exit(0)

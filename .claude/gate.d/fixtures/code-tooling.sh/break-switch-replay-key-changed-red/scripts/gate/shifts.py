#!/usr/bin/env python3
# 样本脚本：自证读同目录的 shifted.tsv（进键），不设开关的那一趟读完往它末尾加一行；开关那一趟不加，只打一行文件头没点名的失败行：
# 基准趟跑完这一份的键就变了，开关那一条又判成不符
# 弄坏开关：
#   SHIFTS_BREAK=other  失败行换成别的一格
#       shifts-other（「✗ 自检：约定的那一格」）
import os
import sys

TABLE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "shifted.tsv")

if sys.argv[1:] == ["--selftest"]:
    with open(TABLE, encoding="utf-8") as handle:
        rows = handle.read().splitlines()
    if os.environ.get("SHIFTS_BREAK") == "other":
        print("  ✗ 自检：另一格出了错：应当 1，实际 0")
        sys.exit(1)
    with open(TABLE, "a", encoding="utf-8") as handle:
        handle.write("appended\n")
    print(f"  ✓ 自检通过（表里原有 {len(rows)} 行）")
    sys.exit(0)

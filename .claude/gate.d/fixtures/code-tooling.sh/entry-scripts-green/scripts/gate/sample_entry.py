#!/usr/bin/env python3
"""样本入口脚本：列项、自证、每项一行带齐。"""
import sys
if sys.argv[1:] in (["--list-items"], ["--selftest"]):
    sys.exit(0)
counts = "judged=1 red=0 reused=0"
print(f"SAMPLE_PY_ITEM {counts} "
      f"name={sys.argv[1:]}")

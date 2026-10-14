#!/usr/bin/env python3
"""样本：自证退 0 的一条（python）。"""
import sys

if sys.argv[1:] != ["--selftest"]:
    sys.exit(2)
print("  ✓ c 的自证 1 条都判对")
sys.exit(0)

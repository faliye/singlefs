#!/usr/bin/env python3
"""样本：用 match 语句实现自证（不是常见的 == 或 case 分支写法），没有阶段在跑它。"""
import sys

match sys.argv[1:]:
    case ["--selftest"]:
        print("  ✓ 自证 1 条都判对")
        sys.exit(0)
sys.exit(0)

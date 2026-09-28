#!/usr/bin/env python3
# research-unit-tests 格绿样本里的替身：不是真脚本的拷贝（拷来的副本会与真脚本分叉），只认 --selftest、打一行、退 0，
# 让这一格走完「研究脚本的自证」那一段；真脚本的判别力由它们自己的 --selftest 管
import sys
if sys.argv[1:] != ["--selftest"]:
    print("  ✗ 替身只认 --selftest")
    print("     → 怎么办：这是门禁样本里的替身，别当真脚本用")
    sys.exit(2)
print("替身自证：只认 --selftest")

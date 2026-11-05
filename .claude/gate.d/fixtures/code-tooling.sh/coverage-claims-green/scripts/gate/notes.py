#!/usr/bin/env python3
# 样本：只在注释、行尾注释与出路文字里提到 --selftest（例：单跑 python3 scripts/other.py --selftest），代码里没有实现它，不算声称有
import sys

QUIET = True  # 调用方的 --selftest 起它时同样安静（行尾注释，不算实现了）
print("  → 要看另一份的自证：python3 scripts/other.py --selftest")
sys.exit(0)

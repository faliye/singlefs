#!/usr/bin/env python3
"""样本：命令行认到 --selftest 只为退 2、指到另一份自证文件，自己没有自证；
登记在工具层自检那一道 research-script-selftest-coverage 格的 NOT_RUN_HERE 里（样本里缺了这份，那一行判成过期豁免）。"""
import sys
sys.exit(2 if sys.argv[1:2] == ["--selftest"] else 0)

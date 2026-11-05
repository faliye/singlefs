#!/usr/bin/env python3
"""样本：只被 import 的库，代码里的 --selftest 是认祖先进程命令行的那个串，没有自己的自证；
登记在工具层自检那一道 research-script-selftest-coverage 格的 NOT_RUN_HERE 里（样本里缺了这份，那一行判成过期豁免）。"""
import sys
INSIDE_SELFTEST_MARK = b"--selftest"
sys.exit(0)

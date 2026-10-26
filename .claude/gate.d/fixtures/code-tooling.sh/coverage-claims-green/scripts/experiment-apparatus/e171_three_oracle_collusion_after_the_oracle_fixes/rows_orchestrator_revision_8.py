#!/usr/bin/env python3
"""样本：冻结层，代码里实现了 --selftest，单跑按设计不过；
登记在工具层自检那一道 research-script-selftest-coverage 格的 NOT_RUN_HERE 里（样本里缺了这份，那一行判成过期豁免）。"""
import sys
arguments = sys.argv[1:]
if arguments[:1] == ["--selftest"]:
    sys.exit(3)
sys.exit(0)

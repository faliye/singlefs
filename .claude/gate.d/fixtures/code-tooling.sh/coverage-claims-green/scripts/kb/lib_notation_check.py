#!/usr/bin/env python3
"""样本：只被 import 的库，没有自己的 --selftest：调它的脚本各自的 --selftest 走一遍拒绝与放行；
登记在工具层自检那一道 research-script-selftest-coverage 格的 NOT_RUN_HERE 里（样本里缺了这份，那一行判成过期豁免）。"""
import sys
sys.exit(0)

#!/usr/bin/env python3
"""样本：只被 import、没有命令行入口的库，判法由调它的脚本的 --selftest 覆盖，它自己没有自证；
登记在工具层自检那一道 research-script-selftest-coverage 格的 NOT_RUN_HERE 里（样本里缺了这份，那一行判成过期豁免）。"""
import sys
sys.exit(0)

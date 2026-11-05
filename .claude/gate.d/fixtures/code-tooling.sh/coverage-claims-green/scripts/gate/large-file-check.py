#!/usr/bin/env python3
"""样本：文件头写着「这份脚本没有 --selftest」，照代码认成实现了；登记在工具层自检那一道 research-script-selftest-coverage 格的 NOT_RUN_HERE 里
（样本里缺了这份，那一行判成过期豁免）。"""
import sys
sys.exit(0)

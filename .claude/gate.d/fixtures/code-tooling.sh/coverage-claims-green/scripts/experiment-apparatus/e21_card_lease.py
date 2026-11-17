#!/usr/bin/env python3
"""样本：只被别的脚本与自证 import，直接执行只打用法；弄坏开关的守卫放行单独一个 --selftest（那一个是自证自己起的），自己没有自证；
登记在工具层自检那一道 research-script-selftest-coverage 格的 NOT_RUN_HERE 里（样本里缺了这份，那一行判成过期豁免）。"""
import sys
sys.exit(0 if sys.argv[1:] == ["--selftest"] else 2)

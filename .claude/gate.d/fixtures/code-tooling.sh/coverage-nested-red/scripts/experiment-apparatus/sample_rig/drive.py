#!/usr/bin/env python3
"""样本：装置目录那一层（scripts/<类>/<装置>/）的脚本，代码里实现了 --selftest，没有阶段在跑它。"""
import sys
arguments = sys.argv[1:]
if arguments[:1] == ["--selftest"]:
    sys.exit(0)
sys.exit(0)

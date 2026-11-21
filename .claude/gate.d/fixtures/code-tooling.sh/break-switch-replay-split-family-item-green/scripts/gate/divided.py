#!/usr/bin/env python3
# 样本脚本：入口，自己不读弄坏开关，文件头也没有开关表；读开关的代码与开关表都在 import 的 divided_core.py 里
import sys

import divided_core

if sys.argv[1:] == ["--selftest"]:
    sys.exit(divided_core.selftest())

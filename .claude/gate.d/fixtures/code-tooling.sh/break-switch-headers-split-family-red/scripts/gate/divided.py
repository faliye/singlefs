#!/usr/bin/env python3
# 样本脚本：入口，自己不读弄坏开关，读开关的代码在 import 的 divided_core.py 里，开关表写在这里，只有一条
# 弄坏开关：
#   DIVIDED_BREAK=drop  丢掉一格
#       divided-drop（「✗ 自检：丢了的那一格」）
import divided_core

divided_core.run()

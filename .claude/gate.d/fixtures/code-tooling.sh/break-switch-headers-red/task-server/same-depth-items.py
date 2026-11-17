#!/usr/bin/env python3
"""样本服务库：开关那一条的点名行与开关行一样深，读法认不出点名的项
  SAME_DEPTH_BREAK=skip  判法整段跳过
  「判法那一格」
"""
import os
BROKEN = os.environ.get("SAME_DEPTH_BREAK", "")

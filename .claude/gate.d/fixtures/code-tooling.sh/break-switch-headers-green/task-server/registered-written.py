#!/usr/bin/env python3
"""样本服务库：在复跑登记表里，开关表写全
  REGISTERED_WRITTEN_BREAK=skip  判法整段跳过
      「判法那一格」
"""
import os
BROKEN = os.environ.get("REGISTERED_WRITTEN_BREAK", "")

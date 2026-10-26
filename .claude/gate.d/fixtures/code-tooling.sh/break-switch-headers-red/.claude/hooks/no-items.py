#!/usr/bin/env python3
"""样本钩子库：开关那一条没写点名的项
  NO_ITEMS_BREAK=skip  判法整段跳过
"""
import os
BROKEN = os.environ.get("NO_ITEMS_BREAK", "")

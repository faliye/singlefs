#!/usr/bin/env python3
"""样本服务库：在复跑登记表里又登在豁免表里，点名行与开关行一样深
  REGISTERED_EXEMPT_BREAK=skip  判法整段跳过
  「判法那一格」
"""
import os
BROKEN = os.environ.get("REGISTERED_EXEMPT_BREAK", "")

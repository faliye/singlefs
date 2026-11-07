#!/usr/bin/env python3
"""样本入口脚本：文档串里写着 SAMPLE_PY_ITEM judged=1 red=0 reused=0，代码里那一句少 red=（文档串不当代码认）。"""
import sys
if sys.argv[1:] in (["--list-items"], ["--selftest"]):
    sys.exit(0)
print(f"SAMPLE_PY_ITEM judged=1 reused=0 name={sys.argv[1:]}")

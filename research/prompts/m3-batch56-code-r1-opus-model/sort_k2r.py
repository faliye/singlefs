#!/usr/bin/env python3
"""标准输入里以 `OPUS_K2R ` 开头的行按种子排好放在前面（多线程打印的次序不定），别的行照原次序接在后面。"""
import re
import sys
lines = sys.stdin.read().splitlines()
seeded = [line for line in lines if line.startswith("OPUS_K2R ")]
others = [line for line in lines if not line.startswith("OPUS_K2R ")]
seeded.sort(key=lambda line: int(re.search(r"seed=(\d+)", line).group(1)))
print("\n".join(seeded + others))

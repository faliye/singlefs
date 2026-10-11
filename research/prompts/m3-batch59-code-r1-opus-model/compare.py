#!/usr/bin/env python3
"""比 expected/ 与这一次的 out/：逐份逐行相同才算一致。用法：compare.py <expected 目录> <out 目录>。"""
import sys
from pathlib import Path
expected, out = Path(sys.argv[1]), Path(sys.argv[2])
names = sorted(path.name for path in expected.glob("*.txt"))
differing = 0
for name in names:
    produced = out / name
    same = produced.exists() and produced.read_text() == (expected / name).read_text()
    differing += 0 if same else 1
    print(f"{'一致' if same else '不一致'} {name}")
print(f"compare：{len(names)} 份，不一致 {differing} 份")
sys.exit(1 if differing else 0)

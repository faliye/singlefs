#!/usr/bin/env python3
"""一次 cargo test 的摘录排成确定的次序（线程先后不定）：OPUS_K1R 行按种子排，别的 OPUS 行与 panicked 行各按字面排，
其余行（failures、失败名、test result）原样、次序不动，依次接在后面。panicked 行里的线程号先去掉。"""
import re
import sys

lines = [re.sub(r"' \(\d+\) panicked", "' panicked", line) for line in sys.stdin.read().splitlines()]
random_history = sorted((line for line in lines if line.startswith("OPUS_K1R ")), key=lambda line: int(re.search(r"seed=(\d+)", line).group(1)))
other_probe = sorted(line for line in lines if line.startswith("OPUS") and not line.startswith("OPUS_K1R "))
panicked = sorted(line for line in lines if line.startswith("thread "))
rest = [line for line in lines if not line.startswith("OPUS") and not line.startswith("thread ")]
print("\n".join(random_history + other_probe + panicked + rest))

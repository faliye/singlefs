#!/usr/bin/env python3
"""在自己的副本里拼出探针测试文件（不进仓）：zz_probe_b36.rs = checker_known_bad_images.rs 整份 + probes/probe_b36_tail*.rs
用法 build_probes.py <副本根> <模型目录>"""
import sys
root, model = sys.argv[1], sys.argv[2]
tests = f"{root}/crates/singlefs-harness/tests"
src = open(f"{tests}/checker_known_bad_images.rs", encoding="utf-8").read()
names = ["probe_b36_tail.rs", "probe_b36_tail5.rs", "probe_b36_tail_a.rs", "probe_b36_tail_r3.rs", "probe_b36_tail_r2.rs"]
tails = "".join(open(f"{model}/probes/{n}", encoding="utf-8").read() for n in names)
open(f"{tests}/zz_probe_b36.rs", "w", encoding="utf-8").write(src + tails)
print("probes built")

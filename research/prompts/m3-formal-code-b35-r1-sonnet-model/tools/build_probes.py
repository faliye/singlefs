#!/usr/bin/env python3
"""在自己的副本里拼出探针测试文件（不进仓）：zz_probe_b35.rs = checker_known_bad_images.rs 整份 + probes/probe_b35_tail*.rs
用法 build_probes.py <副本根> <模型目录>"""
import sys
root, model = sys.argv[1], sys.argv[2]
tests = f"{root}/crates/singlefs-harness/tests"
src = open(f"{tests}/checker_known_bad_images.rs", encoding="utf-8").read()
names = ["probe_b35_tail.rs"] + [f"probe_b35_tail{i}.rs" for i in range(2, 10)]
tails = "".join(open(f"{model}/probes/{n}", encoding="utf-8").read() for n in names)
open(f"{tests}/zz_probe_b35.rs", "w", encoding="utf-8").write(src + tails)
print("probes built")

#!/usr/bin/env python3
"""把 33-b 改过的 checker 档用例文件整份拷进 harness 档的 tests/ 做探针（只在草稿副本里）：删掉唯一用到 checker 档包的 use 行与最后那条用到它的用例，
其余（含 33-b 新造法那条 rolling_back_below_a_floor_only_the_system_configuration_carries_is_refused_before_any_write）原样。
这不是 checker 档包自己的测试二进制：它在 singlefs-harness 包里编、跑；报告里只当「新造法在 harness 档上量到的」用，不当 checker 档用例本身的结果。
用法：make_probe_rollback_floor.py TREE"""
import sys
tree = sys.argv[1]
src = tree + "/crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs"
dst = tree + "/crates/singlefs-harness/tests/probe_b33_rollback_floor.rs"
lines = open(src, encoding="utf-8").read().split("\n")
last_test = max(i for i, line in enumerate(lines) if line.startswith("#[test]"))
start = last_test
while start > 0 and lines[start - 1].startswith("///"):
    start -= 1
body = [line for line in lines[:start] if not line.startswith("use singlefs_checker_tier::crash")]
open(dst, "w", encoding="utf-8").write("\n".join(body) + "\n")
print("WROTE", dst, len(body))

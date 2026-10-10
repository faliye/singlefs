#!/usr/bin/env python3
"""攻方副本：把 inode 树末尾分裂出的右半容器号写成「这条新记录的 inode 号 − 1」（坏实现，只改副本）。
用法：mutate_core_container_number.py <副本根> <apply|revert>"""
import sys, pathlib
p = pathlib.Path(sys.argv[1]) / "crates/singlefs-core/src/inode_tree.rs"
orig = pathlib.Path(str(p) + ".orig")
if not orig.exists():
    orig.write_bytes(p.read_bytes())
s = orig.read_text()
a = "                                container: inode.0,\n                                container_birth: checkpoint_txg,\n"
assert s.count(a) == 1
if sys.argv[2] == "apply":
    s = s.replace(a, "                                container: inode.0 - 1,\n                                container_birth: checkpoint_txg,\n")
tmp = pathlib.Path(str(p) + ".tmp"); tmp.write_text(s); tmp.replace(p)
print(sys.argv[2])

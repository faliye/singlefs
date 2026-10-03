#!/usr/bin/env python3
"""delta_by_ring_model.py 的判别力自证：把门槛改成「环里最新根」（错的定义），与 anchor_dump 逐行比必须出现不符行。"""
import re
import sys

import delta_by_ring_model as model

source = open(model.__file__, encoding="utf-8").read().replace("floor = oldest", "floor = roots[-1]")
namespace = {"__name__": "mutant"}
exec(compile(source, "mutant", "exec"), namespace)
text = open(sys.argv[1], encoding="utf-8").read().splitlines()
dump = {}
for line in text:
    m = re.match(r"E7RESULT name=anchor_dump S=(\d+) k=(\d+) pos=(\S+) fits=\S+ txg=(\d+) floor=(\d+) h_que=(\d+) h_dong=\d+ q1a_tree_table_lower_bound=(\d+)", line)
    if m:
        S, k, position, txg, floor, h_que, q1a = m.groups()
        dump.setdefault((int(S), int(k), position), {})[int(txg)] = (int(floor), int(h_que), int(q1a))
mismatched = 0
compared = 0
for S in (4, 8, 16):
    for position, hole in (("front", 5), ("back", 3 * S + 4)):
        for txg, floor, missing, delta in namespace["series"](S, hole, patched=False):
            if txg in dump[(S, 1, position)]:
                compared += 1
                mismatched += dump[(S, 1, position)][txg] != (floor, missing, delta)
print(f"门槛改成最新根：逐行比 {compared} 行、不符 {mismatched} 行（必须 > 0）")

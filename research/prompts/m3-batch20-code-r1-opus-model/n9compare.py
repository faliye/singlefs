#!/usr/bin/env python3
"""N9：原型在冻结树上导出的删除写字节，与入库的三份导出逐字节比；不同的行按 (行名, 偏移, 长度) 归类。
用法：python3 n9compare.py <仓根> <h1h2 导出> <h3 导出>"""
import collections
import hashlib
import pathlib
import re
import sys

repo, h1h2, h3 = (pathlib.Path(argument) for argument in sys.argv[1:4])
results = repo / "research/results"
first = results / "e172-deletion-write-dump-2026-10-05.out"
r2_h1h2 = results / "e172-r2-deletion-write-dump-h1h2-2026-10-05.out"
r2_h3 = results / "e172-r2-deletion-write-dump-h3-2026-10-05.out"


def sha(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


print(f"N9 dump=h1h2 sha256={sha(h1h2)} equals_r2_h1h2={h1h2.read_bytes() == r2_h1h2.read_bytes()} equals_first_segment_dump={h1h2.read_bytes() == first.read_bytes()}")
print(f"N9 dump=h3 sha256={sha(h3)} equals_r2_h3={h3.read_bytes() == r2_h3.read_bytes()}")
a = first.read_text().splitlines()
b = h1h2.read_text().splitlines()
groups = collections.Counter()
for x, y in zip(a, b):
    if x != y:
        fields = dict(re.findall(r"(\w+)=(\S+)", x[:400]))
        groups[(fields.get("name"), fields.get("offset"), fields.get("length"))] += 1
print(f"N9 h1h2_vs_first_segment lines={len(a)}/{len(b)} differing_lines={sum(groups.values())} groups={sorted(groups.items(), key=str)}")
config = [l for l in h3.read_text().splitlines() if "name=impl_config" in l][0]
print("N9 h3_impl_config " + " ".join(f for f in config.split() if f.startswith(("history=", "write_time_seconds=", "sequential_write_time_seconds=", "intent_write_time_seconds=", "file_bytes=", "batch_data_units="))))

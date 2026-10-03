"""e171-conclusion-r2 攻方腿：把单镜像谓词 W（witness_consistency.py 的逐态输出）与实现的两次可写挂载（判官探针输出）逐态并起来计数。
两条路子不共用代码：W 用冻结 O3 的 Python 读盘与重放；挂载是 core 的 `mount_writable`（Rust）。
用法：python3 join_witness_and_mount.py <W 逐态文件> <判官探针输出>...
"""
import sys
from collections import Counter

w = {}
for line in open(sys.argv[1]):
    label, state, value = line.rstrip("\n").split("\t")
    w[(label, state)] = value
joined = Counter()
for path in sys.argv[2:]:
    for line in open(path):
        c = line.rstrip("\n").split("\t")
        f = dict(x.split("=", 1) for x in c[2:] if "=" in x)
        joined[(c[0], w.get((c[0], c[1]), "missing"), f["mount1"], f["unchanged_after_mount1"], f["mount2"])] += 1
for (label, value, m1, unchanged, m2), n in sorted(joined.items()):
    print(f"name=witness_vs_mount label={label} w={value} mount1={m1} unchanged_after_mount1={unchanged} mount2={m2} count={n}")

"""e171-conclusion-r2 攻方腿：把驱动探针写出的录制流（mkfs 之后每一步的盘、种类、偏移、长度、内容 sha256）与一份 E171 导出目录的 ops.tsv 逐条比。
用法：python3 compare_ops.py <探针输出.tsv> <标签> <导出流目录> [只比前 N 条]
输出一行：两边条数、逐条相同的前缀长度、第一处不同的下标与两边那一条、前 N 条里不同的条数。
"""
import sys

probe, label, dump_dir = sys.argv[1], sys.argv[2], sys.argv[3]
limit = int(sys.argv[4]) if len(sys.argv) > 4 else None
mine = [l.rstrip("\n").split("\t") for l in open(probe) if l.startswith(label + "\top\t")]
mine = [(r[3], r[4], r[5], r[6], r[7]) for r in mine]
theirs = []
for line in open(f"{dump_dir}/ops.tsv"):
    c = line.rstrip("\n").split("\t")
    theirs.append((c[1], c[2], c[3], c[4], c[6]))
n = min(len(mine), len(theirs)) if limit is None else min(limit, len(mine), len(theirs))
same = 0
while same < n and mine[same] == theirs[same]:
    same += 1
differing = sum(1 for i in range(n) if mine[i] != theirs[i])
first = "-" if same == n else f"{same}:mine={'/'.join(mine[same][:4])}:theirs={'/'.join(theirs[same][:4])}"
print(f"name=compare_ops label={label} dump={dump_dir.split('e171-dumps/')[-1]} probe_ops={len(mine)} dump_ops={len(theirs)} "
      f"compared={n} identical_prefix={same} differing_in_compared={differing} first_difference={first}")

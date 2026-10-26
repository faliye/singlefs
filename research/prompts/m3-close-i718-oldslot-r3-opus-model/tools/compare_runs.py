"""比两份 ROW 输出（腿交的 out/raw 与判决复跑那一份）：逐 (S, 格, 镜像, 种类) 键比张数与逐候选的四类归类，只打不同的。"""
import collections, re, sys, glob
def load(directory):
    table = collections.defaultdict(list)
    for path in sorted(glob.glob(directory + "/*.out")):
        for line in open(path, encoding="utf-8", errors="replace"):
            if not line.startswith("ROW "):
                continue
            f = dict(re.findall(r"(\w+)=(\S+)", line))
            m = re.search(r" all=(\S+)", line)
            if not m or m.group(1) == "none":
                continue
            table[(int(f["S"]), f["cell"], re.sub(r"n\d+", "n*", f["image"]), f["kind"])].append(tuple(int(x) for x in m.group(1).split(",")))
    return table
def klass(vectors, column):
    seen = {v[column] for v in vectors}
    return {frozenset({1}): "H", frozenset({0}): "V", frozenset({-1}): "N"}.get(frozenset(seen), "M")
leg, rerun = load(sys.argv[1]), load(sys.argv[2])
print("keys leg", len(leg), "rerun", len(rerun), "only_leg", len(set(leg) - set(rerun)), "only_rerun", len(set(rerun) - set(leg)))
count_differs = class_differs = 0
for key in sorted(set(leg) & set(rerun)):
    a, b = leg[key], rerun[key]
    if len(a) != len(b):
        count_differs += 1
        print("count", key, len(a), len(b))
    for column in range(11):
        if klass(a, column) != klass(b, column):
            class_differs += 1
            print("class", key, "column", column, klass(a, column), klass(b, column))
print("keys_with_count_difference", count_differs, "key_column_class_differences", class_differs)
print("rows leg", sum(map(len, leg.values())), "rerun", sum(map(len, rerun.values())))

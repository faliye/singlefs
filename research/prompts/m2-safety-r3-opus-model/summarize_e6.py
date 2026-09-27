"""E6 汇总：最近 4 个不同状态里不再是回退候选的格，按「这一步是什么 : 为什么」分类求和（按臂）。
分类：before-last-umount(B1) = 正常卸载把 F 抬过了它（B1 定的代价）；evicted(ring-has-N-states) = 根环里已经没有它的根、环里只剩 N 个不同状态；
below-F = 环里有它的根而都低于 F；别的字串 = 在拷贝上回退它时报的错。"""
import re, sys, collections, ast
agg = collections.defaultdict(collections.Counter)
cells = collections.Counter()
for line in open(sys.argv[1], encoding="utf-8"):
    if not line.startswith("E6 "):
        continue
    d = dict(re.findall(r"(\w+)=([^\s{]+)", line))
    cells[d["arm"]] += 1
    body = "{" + line.split("lost={", 1)[1].split("}", 1)[0] + "}"
    for k, v in ast.literal_eval(body).items():
        agg[d["arm"]][re.sub(r"ring-has-(\d+)-states", lambda m: "ring-has-" + ("≤3" if int(m.group(1)) <= 3 else "≥4") + "-states", k)] += v
for arm in agg:
    print(arm, f"（{cells[arm]} 格）")
    for k, v in sorted(agg[arm].items()):
        print("   ", k, v)

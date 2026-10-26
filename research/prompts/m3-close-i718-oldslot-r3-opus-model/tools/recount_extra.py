"""第二轮辩方的几个数逐个重数（只读 out/raw 的 ROW 行，自己解析）：
① 只在部分 S 出现的键；② 写坏镜像（zero、torn_enum、torn_half）张数与红单分布；③ G 各 S 写坏镜像上丁甲同实例（第 8 列）判红几张；
④ BR oldest 各 S 张数与今天（第 1 列）、丁甲（第 7 列）、甲>=（第 3 列）、任一圈（第 6 列）的判定分布。"""
import collections, re, sys, glob
keys = collections.defaultdict(set)
bad = collections.Counter(); bad_total = 0
g = collections.defaultdict(collections.Counter)
br = collections.defaultdict(collections.Counter)
for path in sorted(glob.glob(sys.argv[1] + "/*.out")):
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("ROW "):
            continue
        f = dict(re.findall(r"(\w+)=(\S+)", line))
        m = re.search(r" all=(\S+)", line)
        if not m or m.group(1) == "none":
            continue
        v = [int(x) for x in m.group(1).split(",")]
        s = int(f["S"]); image = re.sub(r"n\d+", "n*", f["image"])
        keys[(f["cell"], image, f["kind"])].add(s)
        red = re.search(r" red=(\[.*\])", line).group(1)
        if image in ("zero", "torn_enum", "torn_half"):
            bad_total += 1; bad[red] += 1
        if f["cell"] == "G" and image in ("zero", "torn_enum", "torn_half"):
            g[s][(image, "ding_jia_same_instance", v[7])] += 1
        if f["cell"] == "BR" and "oldest" in image:
            br[s][("today", v[0])] += 1; br[s][("ding_jia", v[6])] += 1; br[s][("jia_seed", v[2])] += 1; br[s][("any_lap", v[5])] += 1
print("keys_not_at_all_four_S", sorted((k, sorted(s)) for k, s in keys.items() if s != {4, 5, 8, 16}))
print("written_bad_images", bad_total, dict(bad))
for s in sorted(g): print("G S=%d" % s, dict(sorted(g[s].items())))
for s in sorted(br): print("BR-oldest S=%d" % s, dict(sorted(br[s].items())))

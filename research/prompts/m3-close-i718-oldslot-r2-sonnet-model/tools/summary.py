"""按 S 与格汇总探针的 ROW 行（只读）：每个候选在「写坏」镜像（zero 一类）与「N 做完的健康镜像」等关键镜像上成立 / 判红 / 不适用各几张。
用法：summary.py <输出文件…>   候选顺序同 tally.py。零星同名格（同一 S 的几个起点分在几个文件）自动并在一起。
"""
import re, sys, collections
NAMES = ["today", "jia_gt", "jia_seed", "yi_literal", "yi_gate", "any_lap", "ding_jia", "ding_jia_same",
         "ding_jia_single", "ding_yi", "ding_yi_same"]
SHORT = ["今天/丙", "甲>", "甲>=", "乙字面", "乙加闸", "任一圈", "丁甲", "丁甲同实例", "丁甲单值", "丁乙", "丁乙同实例"]
table = collections.OrderedDict()
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("ROW "):
            continue
        f = dict(re.findall(r"(\w+)=(\S+)", line))
        m = re.search(r" all=(\S+)", line)
        if not m or m.group(1) == "none":
            continue
        vals = [int(x) for x in m.group(1).split(",")]
        img = re.sub(r"n\d+", "n*", f["image"])
        key = (int(f["S"]), f["cell"], img, f["kind"])
        c = table.setdefault(key, collections.Counter())
        c["n"] += 1
        for name, v in zip(NAMES, vals):
            c[name + ("H" if v == 1 else "V" if v == 0 else "N")] += 1
want_images = ("zero", "healthy_n", "before_root", "oldest", "middle", "bad_p_255", "bad_t_as_floor")
print("格式：成立H/判红V/不适用N；列 =", " | ".join(SHORT))
for key in sorted(table):
    S, cell, img, kind = key
    if not any(w in img for w in want_images):
        continue
    c = table[key]
    cols = " | ".join(f"{c[n+'H']}/{c[n+'V']}/{c[n+'N']}" for n in NAMES)
    print(f"S={S:<2} {cell:<3} {img:<34} {kind:<13} 张数={c['n']:<4} {cols}")

"""把探针的 ROW 行按（格、镜像名、kind）汇总：每个候选 成立(H) / 判红(V) / 不适用(N) 各几张。
用法：tally.py <输出文件…>   只读，打到标准输出。
"""
import re, sys, collections
NAMES = ["today", "jia_gt", "jia_seed", "yi_literal", "yi_gate", "any_lap", "ding_jia", "ding_jia_same",
         "ding_jia_single", "ding_yi", "ding_yi_same"]
table = collections.OrderedDict()
for path in sys.argv[1:]:
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.startswith("ROW "):
            continue
        f = dict(re.findall(r"(\w+)=(\S+)", line))
        m = re.search(r" all=(\S+)", line)
        if not m or m.group(1) == "none":
            key = (f["S"], f["cell"], f["image"], f["kind"], "I-7.18 不适用或没走到")
            table.setdefault(key, collections.Counter())["no_all"] += 1
            continue
        vals = [int(x) for x in m.group(1).split(",")]
        key = (f["S"], f["cell"], re.sub(r"n\d+", "n*", f["image"]), f["kind"], "")
        c = table.setdefault(key, collections.Counter())
        c["images"] += 1
        for name, v in zip(NAMES, vals):
            c[f"{name}:{'H' if v == 1 else 'V' if v == 0 else 'N'}"] += 1
for key, c in table.items():
    n = c["images"]
    cols = []
    for name in NAMES:
        cols.append(f"{name}=H{c[name+':H']}/V{c[name+':V']}/N{c[name+':N']}")
    print(f"S={key[0]} cell={key[1]} image={key[2]} kind={key[3]} images={n} " + " ".join(cols) + (f" no_all={c['no_all']}" if c["no_all"] else ""))

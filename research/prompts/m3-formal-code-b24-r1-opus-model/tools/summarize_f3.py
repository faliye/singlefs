#!/usr/bin/env python3
"""F3 变异矩阵汇总（m3-formal-code-b24-r1 opus）：读 rerun.sh f3 段的 f3.txt，每种种子算法一行：
入库四条（F3-c、A、B、C）与攻方 F3-e 三条（A2、B3、C2）的红绿，战役各档（快档、偏回退、线程那条、大档合起来）与稠密那一份上
同一段历史内撞种子的态数，以及战役各档合起来的不同种子数（入库那一种是基线）。"""
import re, sys
rows = {}; order = []
for line in open(sys.argv[1], encoding="utf-8"):
    m = re.match(r"(f3-tests|f3-proposed|harm) label=(\S+)", line)
    if not m:
        continue
    kind, label = m.groups()
    short = label if label == "pristine" else label.replace("mutations.tsv:", "tsv:").replace("opus-mutations.tsv:", "opus:")
    if short not in rows:
        rows[short] = {"name": ""}; order.append(short)
    r = rows[short]
    if kind == "f3-tests":
        name = re.search(r"label=\S+ (.*?) \|", line)
        r["name"] = (name.group(1) if name and label != "pristine" else "入库那一种").split("：")[0]
        r["tests"] = " ".join(re.findall(r"(?:F3c|A|B|C)=(?:green|red\(left=\d+\))", line))
    elif kind == "f3-proposed":
        r["proposed"] = " ".join(re.findall(r"(?:A2|B3|C2)=\w+\(\d+/\d+\)", line))
    else:
        this = line.split("| this: ")[1]
        sharing = re.search(r"states_sharing_a_seed_within_a_history=(\d+)", this).group(1)
        distinct = re.search(r"distinct_over_all_histories=(\d+/\d+)", this).group(1)
        states = int(re.search(r"states=(\d+)", line).group(1))
        if states < 10000:
            r["campaign"] = f"campaign_sharing={sharing} campaign_distinct={distinct}"
        else:
            r["dense"] = f"dense_sharing={sharing}"
for k in order:
    r = rows[k]
    print(f"{k} [{r['name']}] {r.get('tests','')} | {r.get('proposed','')} | {r.get('campaign','')} {r.get('dense','')}")

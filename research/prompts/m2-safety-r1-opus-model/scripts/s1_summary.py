#!/usr/bin/env python3
"""S1 各候选的汇总：读 logs/ 下 s1-<候选>-*.tsv 与 s1w-<候选>-*.tsv，按结局分类计数。用法：s1_summary.py <logs 目录> <候选名>……"""
import collections, glob, os, re, sys

def cls(text):
    text = re.sub(r"\[unit[^\]]*\]", "", text)
    text = re.sub(r" \{.*", "", text)
    if text.startswith("ok#"):
        return "ok"
    if text.startswith("AFTER:"):
        return "AFTER:" + text[6:].split("(")[0]
    return text

def fields(row):
    kv = {}
    for part in row:
        if "=" in part and not part.startswith("k="):
            key, value = part.split("=", 1)
            kv[key] = value
    return kv

logs = sys.argv[1]
for name in sys.argv[2:]:
    print(f"## {name}")
    for kind in ["skip-persistent", "skip-transient", "judged-persistent"]:
        path = os.path.join(logs, f"s1-{name}-{kind}.tsv")
        if not os.path.exists(path):
            continue
        counts = collections.Counter()
        for line in open(path):
            row = line.rstrip("\n").split("\t")
            mode = row[3]
            kv = fields(row[7:])
            if mode == "control":
                counts[("control", cls(row[6]), "ring=" + kv.get("target_in_ring", "?"))] += 1
            elif mode == "persistent":
                counts[("persistent", cls(row[6]), "ring=" + kv.get("target_in_ring", "?"),
                        "retry_faulted=" + cls(kv.get("retry_faulted", "?")),
                        "retry_cleared=" + cls(kv.get("retry_cleared", "?")),
                        "writable_cleared=" + cls(kv.get("writable_cleared", "?")),
                        "probe=" + kv.get("probe", "?").split(":")[0])] += 1
            else:
                counts[("transient", row[5], cls(row[6]), "ring=" + kv.get("target_in_ring", "?"),
                        "retry_cleared=" + cls(kv.get("retry_cleared", "?")),
                        "probe=" + kv.get("probe", "?").split(":")[0])] += 1
        print(f"### {kind}")
        for key, value in sorted(counts.items()):
            print(f"{value:6d}  " + " | ".join(key))
    for pattern in [f"s1w-{name}-4g-*.tsv", f"s1w-{name}-312skip-*.tsv"]:
        counts = collections.Counter()
        files = sorted(glob.glob(os.path.join(logs, pattern)))
        for path in files:
            tag = os.path.basename(path)[len(f"s1w-{name}-"):-4]
            for line in open(path):
                row = line.rstrip("\n").split("\t")
                if row[4] == "control" or (len(row) > 4 and row[3].startswith("target=")):
                    kv = fields(row[4:])
                    counts[(tag.rsplit("-", 1)[0] if "4g" in tag else tag, "control", cls(row[5]), "ring=" + kv.get("target_in_ring", "?"))] += 1
                    continue
                kv = fields(row[6:])
                counts[(tag.split("-t")[0], cls(row[5]), "burned=" + kv.get("burned", "?"), "ring=" + kv.get("target_in_ring", "?"),
                        "retry=" + cls(kv.get("retry", "?")), "writable=" + cls(kv.get("writable", "?")),
                        "probe=" + kv.get("probe", "?").split(":")[0])] += 1
        if files:
            print(f"### {pattern}")
            for key, value in sorted(counts.items()):
                print(f"{value:6d}  " + " | ".join(key))

#!/usr/bin/env python3
"""把几份副本的 c331grid / c331v4 行按键对齐：每个候选的丢写格、多拒格（今天可写且不丢、候选拒）、反例里故障段数最少的几格。
用法：summarize_c331.py <runs 目录> 今天的副本名 候选副本名...
"""
import re, sys, collections, pathlib
runs = pathlib.Path(sys.argv[1]); today = sys.argv[2]; candidates = sys.argv[3:]
SLOT_RANGES = {"None": 0, "AllFour": 4, "NewestOnBoth": 2, "NewestOnDevice0": 1, "OlderOnDevice0": 1, "BothOnDevice1": 2}
HIDDEN_RANGES_STANDARD = {"None": 0, "AllRootsAndRecordsOfTheSecondInstance": None, "AllRootsOfTheSecondInstance": None, "NewestRootAndRecord": 3, "NewestRootAndOneDataCopy": 2}

def load(copy):
    rows = {}
    path = runs / f"{copy}-c331_candidates_grid.log"
    for line in path.read_text().splitlines():
        if line.startswith("name=c331grid "):
            key, _, rest = line[len("name=c331grid "):].partition(" verdict=")
            rows[("grid", key)] = rest.split(" ")[0]
        elif line.startswith("name=c331v4 "):
            head, _, tail = line.partition(" || ")
            head_key = " ".join(part for part in head.split(" ") if part.startswith("mount_k_slots") or part.startswith("mount_k_slot_timing") or part.startswith("mount_k_read_back"))
            key, _, rest = tail[len("name=c331grid "):].partition(" verdict=")
            hidden = re.search(r"hidden=(\S+)", key).group(1)
            rows[("v4", head_key + " hidden=" + hidden)] = rest.split(" ")[0]
    return rows

def fault_ranges(kind, key):
    if kind == "v4":
        slots = re.search(r"mount_k_slots=(\S+)", key).group(1)
        hidden = re.search(r"hidden=(\S+)", key).group(1)
    else:
        slots = re.search(r"slots=(\S+)", key).group(1)
        hidden = re.search(r"hidden=(\S+)", key).group(1)
    prefix = re.search(r"prefix=(\S+)", key)
    prefix = prefix.group(1) if prefix else "standard"
    second_instance_publishes = {"standard": 4, "long": 6}.get(prefix, 5)
    hidden_count = {"None": 0, "NewestRootAndRecord": 3, "NewestRootAndOneDataCopy": 2,
                     "AllRootsOfTheSecondInstance": second_instance_publishes,
                     "AllRootsAndRecordsOfTheSecondInstance": 3 * second_instance_publishes}[hidden]
    return SLOT_RANGES[slots] + hidden_count

base = load(today)
print(f"today={today} scenarios={len(base)} verdicts={dict(collections.Counter(base.values()))}")
for candidate in candidates:
    rows = load(candidate)
    missing = set(base) ^ set(rows)
    lost = [(k, v) for k, v in rows.items() if "LOST" in v]
    over_refused = [(k, v) for k, v in rows.items() if v == "refused" and base.get(k) == "writable_no_loss"]
    newly_safe = [(k, v) for k, v in rows.items() if "LOST" in base.get(k, "") and "LOST" not in v]
    print(f"\n== candidate={candidate} scenarios={len(rows)} key_mismatch={len(missing)} verdicts={dict(collections.Counter(rows.values()))}")
    print(f"   lost_cells={len(lost)} over_refused_cells={len(over_refused)} today_lost_now_not_lost={len(newly_safe)}")
    for label, cells in (("lost", lost), ("over_refused", over_refused)):
        if not cells:
            continue
        smallest = min(fault_ranges(k[0], k[1]) for k, _ in cells)
        print(f"   {label}: fewest_fault_ranges={smallest}; cells with that many (up to 12):")
        for (kind, key), verdict in sorted(cells, key=lambda item: item[0][1]):
            if fault_ranges(kind, key) == smallest:
                print(f"     [{kind}] {key} -> {verdict} (today {base.get((kind, key))})")
        by_prefix = collections.Counter(re.search(r"prefix=(\S+)", k[1]).group(1) if k[0] == "grid" else "v4" for k, _ in cells)
        print(f"   {label} by prefix: {dict(by_prefix)}")

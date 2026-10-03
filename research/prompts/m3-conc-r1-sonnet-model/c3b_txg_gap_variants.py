#!/usr/bin/env python3
"""C3 补一格（接 c3_txg_gap_recount.py，不改它）：「施加的记录跳过 txg」换几个口径数，看数随口径变到什么程度。
只读 E169 的 replay-core-inputs 产物。四个口径（每个口径、每类输入各报 份数 / 状态数）：
  V1 相邻两条（含所选根）txg 之差 > 1                                         （c3 的口径）
  V2 相邻两条 txg 之差不是 0 或 1（含降序：后一条 txg 比前一条小）
  V3 施加的 txg 集合（含所选根）不是 root..最大 的连续整数
  V4 输入全部记录里有比最大施加 txg 小、却没被施加的 txg
"""
import os
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
PATH = os.path.join(ROOT, "research", "results",
                    "e169-parallel-commit-append-and-prefix-recovery-replay-core-inputs-2026-10-01.out")


def fields_of(line):
    return dict(f.split("=", 1) for f in line.rstrip("\n").split(" ")[1:] if "=" in f)


def verdicts(f):
    root, applied = int(f["root_txg"]), int(f["bing0_applied_records"])
    if applied == 0 or f["records"] == "none":
        return {}
    records = [r.split(":") for r in f["records"].split(",")]
    txgs = [int(r[1]) for r in records[:applied]]
    sequence = [root] + txgs
    steps = list(zip(sequence, sequence[1:]))
    every = {int(r[1]) for r in records}
    return {
        "V1 差>1": any(b - a > 1 for a, b in steps),
        "V2 差不是0或1": any(b - a not in (0, 1) for a, b in steps),
        "V3 集合不连续": sorted(set(sequence)) != list(range(root, max(sequence) + 1)),
        "V4 有更小的没施加": any(t < max(txgs) and t not in set(txgs) for t in every),
    }


def main():
    tally = defaultdict(lambda: [0, 0])
    with open(PATH, encoding="utf-8") as handle:
        for line in handle:
            if not line.startswith("E7RESULT name=replay_core_input "):
                continue
            f = fields_of(line)
            for name, hit in verdicts(f).items():
                if hit:
                    tally[(name, f["input"])][0] += 1
                    tally[(name, f["input"])][1] += int(f["states"])
    for name in ("V1 差>1", "V2 差不是0或1", "V3 集合不连续", "V4 有更小的没施加"):
        parts = [f"{kind} {tally[(name, kind)][0]} 份 / {tally[(name, kind)][1]} 态" for kind in ("all_durable", "strict_segment")]
        total = sum(tally[(name, k)][0] for k in ("all_durable", "strict_segment")), sum(tally[(name, k)][1] for k in ("all_durable", "strict_segment"))
        print(f"  {name:18s} " + "；".join(parts) + f"；合计 {total[0]} 份 / {total[1]} 态")


if __name__ == "__main__":
    main()

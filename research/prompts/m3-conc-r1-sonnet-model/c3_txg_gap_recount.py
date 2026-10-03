#!/usr/bin/env python3
"""C3：数一遍 E169 恢复输入产物里「今天的前缀核（丙0）施加的记录跳过了一个 txg」的输入与状态数，独立于 E169 页上抄的「10 份 / 208 个状态」。
只读 research/results/ 下 E169 的 replay-core-inputs 产物，不跑 crates/ 的代码。
记录七段：jsn:txg:事务号:提交标记:末条标志:序号:点名单元验过；施加的 = bing0_applied_records 条（输入里按 jsn 升序的前缀）。
一份输入里「跳 txg」= 施加的记录里相邻两条 txg 之差大于 1（含所选根 root_txg 到第一条施加的）。
另数：这些输入是不是都出在无纪律的对照丁历史（产物里没有标 Q2 臂，所以只数它们的记录形状：jsn 次序与 txg 次序是否一致）。
"""
import os
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
PATH = os.path.join(ROOT, "research", "results",
                    "e169-parallel-commit-append-and-prefix-recovery-replay-core-inputs-2026-10-01.out")


def fields_of(line):
    return dict(f.split("=", 1) for f in line.split(" ")[1:] if "=" in f)


def main():
    totals = defaultdict(lambda: [0, 0])      # input kind -> [份数, 状态数]
    gap = defaultdict(lambda: [0, 0])
    gap_examples = []
    with open(PATH, encoding="utf-8") as handle:
        for line in handle:
            if not line.startswith("E7RESULT name=replay_core_input "):
                continue
            f = fields_of(line.rstrip("\n"))
            kind, states, root = f["input"], int(f["states"]), int(f["root_txg"])
            totals[kind][0] += 1
            totals[kind][1] += states
            applied = int(f["bing0_applied_records"])
            if applied == 0 or f["records"] == "none":
                continue
            records = [r.split(":") for r in f["records"].split(",")]
            txgs = [int(r[1]) for r in records[:applied]]
            sequence = [root] + txgs
            jumps = [(a, b) for a, b in zip(sequence, sequence[1:]) if b - a > 1]
            if jumps:
                gap[kind][0] += 1
                gap[kind][1] += states
                if len(gap_examples) < 4:
                    gap_examples.append((kind, root, f["records"], applied, jumps))
    print(f"文件：{os.path.relpath(PATH, ROOT)}")
    for kind in sorted(totals):
        print(f"  输入类 {kind:15s} 份数 {totals[kind][0]:5d} 状态 {totals[kind][1]:6d} | 施加的记录跳过 txg 的：份数 {gap[kind][0]:3d} 状态 {gap[kind][1]:5d}")
    print(f"合计跳 txg 的输入（两类各算一次，E169 页的数法）：份数 {sum(v[0] for v in gap.values())}，状态 {sum(v[1] for v in gap.values())}")
    for kind, root, records, applied, jumps in gap_examples:
        print(f"  例（{kind}，root_txg={root}，施加 {applied} 条，跳点 {jumps}）：records={records}")


if __name__ == "__main__":
    main()

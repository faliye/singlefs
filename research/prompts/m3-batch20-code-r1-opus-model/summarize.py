#!/usr/bin/env python3
"""第 20 批代码轮攻方腿原型的摘要：把 out/ 下各步的输出汇成确定性的几行（不含挂钟），与 expected-summary.txt 逐字比。
用法：python3 summarize.py <out 目录>"""
import collections
import hashlib
import pathlib
import re
import sys

out = pathlib.Path(sys.argv[1])


def sweep_summary(prefix: str, label: str) -> list[str]:
    lines = []
    for path in sorted(out.glob(f"{prefix}*.out")):
        lines += [l for l in path.read_text().splitlines() if l.startswith("SWEEP_HISTORY")]
    lines.sort(key=lambda l: int(re.search(r"index=(\d+)", l).group(1)))
    ran = [l for l in lines if "outcome=ran" in l]
    unreachable = [l for l in lines if "outcome=not_reachable" in l]

    def num(line: str, key: str) -> int:
        return int(re.search(rf" {key}=(\d+)", line).group(1))

    watched = collections.Counter()
    outcomes = collections.Counter()
    divergent_kinds = collections.Counter()
    for line in ran:
        for key, value in re.findall(r'"(I-[0-9.]+=[A-Z])": (\d+)', line.split(" watched=")[1].split(" cpu_outcomes=")[0]):
            watched[key] += int(value)
        for key, value in re.findall(r'"([A-Z]{2})": (\d+)', line.split(" cpu_outcomes=")[1]):
            outcomes[key] += int(value)
        match = re.search(r"first_gpu_divergent=\[[^\]]*\] (cpu_consulted=\S+ cpu_ignored=\S+ extent_root=\S+ inode_root=level\d)", line)
        if match:
            divergent_kinds[match.group(1)] += 1
    reasons = collections.Counter(line.split(" reason=")[1][:40] for line in unreachable)
    digest = hashlib.sha256("\n".join(lines).encode()).hexdigest()[:16]
    result = [
        f"{label} histories={len(lines)} ran={len(ran)} not_reachable={len(unreachable)} images={sum(num(l, 'images') for l in ran)} "
        f"red_histories={sum(1 for l in ran if num(l, 'red'))} gap_histories={sum(1 for l in ran if num(l, 'gap'))} "
        f"gpu_divergent_histories={sum(1 for l in ran if num(l, 'gpu_divergent'))} gpu_divergent_images={sum(num(l, 'gpu_divergent') for l in ran)} "
        f"not_newest_images={sum(num(l, 'not_newest_effective_root') for l in ran)} history_lines_sha256_16={digest}",
        f"{label}_WATCHED {dict(sorted(watched.items()))}",
        f"{label}_CPU_OUTCOMES consulted_ignored={dict(sorted(outcomes.items()))}",
        f"{label}_FIRST_GPU_DIVERGENT_KINDS {dict(sorted(divergent_kinds.items()))}",
    ]
    result += [f"{label}_NOT_REACHABLE count={count} reason={reason}" for reason, count in sorted(reasons.items())]
    return result


def tagged(name: str, label: str, starts: tuple) -> list[str]:
    path = out / name
    if not path.exists():
        return [f"{label} 缺 {name}"]
    return [f"{label} {l}" for l in path.read_text().splitlines() if l.startswith(starts)]


summary = []
summary += sweep_summary("sweep-", "SWEEP")
summary += sweep_summary("fix-sweep-", "FIX_SWEEP")
summary += tagged("badimages.out", "TREE", ("BAD case", "BAD_SUMMARY", "BAD_VIOLATION", "BAD_NOTE"))
summary += tagged("shapes.out", "TREE", ("SHAPE",))
summary += tagged("fix-badimages.out", "FIX", ("BAD case", "BAD_SUMMARY"))
summary += tagged("fix-shapes.out", "FIX", ("SHAPE",))
summary += tagged("layer0.out", "TREE", ("L0",))
summary += tagged("n9.out", "TREE", ("N9",))
summary += [re.sub(r"finished in [0-9.]+s", "finished", l) for l in tagged("fix-tests.out", "FIX", ("test result",))]
print("\n".join(summary))

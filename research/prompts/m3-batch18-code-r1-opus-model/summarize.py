#!/usr/bin/env python3
"""把 rerun.sh 各步的输出并成一份与挂钟、线程数、分批方式无关的摘要（复跑与 expected-summary.txt 逐行比）。
用法：summarize.py <输出目录>"""
import hashlib
import os
import re
import sys

out = sys.argv[1]


def lines_of(prefix):
    found = []
    for name in sorted(os.listdir(out)):
        if name.startswith(prefix) and name.endswith(".out"):
            with open(os.path.join(out, name), encoding="utf-8") as handle:
                found.extend(handle.read().splitlines())
    return found


def digest(items):
    return hashlib.sha256("\n".join(items).encode()).hexdigest()[:16]


def sweep(group, prefix):
    histories = sorted({line for line in lines_of(prefix) if line.startswith("SWEEP_HISTORY ")},
                       key=lambda line: int(re.search(r"index=(\d+)", line).group(1)))
    ran = [line for line in histories if "outcome=ran" in line]
    red = [line for line in ran if " red=0 " not in line]
    images = sum(int(re.search(r" images=(\d+)", line).group(1)) for line in ran)
    kinds = {}
    for line in ran:
        for key, value in re.findall(r'"([^"]+)": (\d+)', line):
            kinds[key] = kinds.get(key, 0) + int(value)
    red_invariants = {}
    for line in red:
        match = re.search(r"first_red=\[[^\]]*\] (I-[0-9.]+)", line)
        if match:
            red_invariants[match.group(1)] = red_invariants.get(match.group(1), 0) + 1
    print(f"{group} histories={len(histories)} ran={len(ran)} not_reachable={len(histories) - len(ran)} images_judged={images} red_histories={len(red)} first_red_invariants={dict(sorted(red_invariants.items()))} history_lines_sha256_16={digest(histories)}")
    print(f"{group}_WATCHED {dict(sorted(kinds.items()))}")


def crash(group, prefix):
    totals = {}
    for line in lines_of(prefix):
        if not line.startswith("CRASH_HISTORY "):
            continue
        index = int(re.search(r"index=(\d+)", line).group(1))
        row = totals.setdefault(index, [0, 0, 0, 0, 0])
        for position, field in enumerate(["prefixes", "crash_image_red", "mount_refused", "after_mount_red", "deletion_scan_unreadable_verdicts"]):
            row[position] += int(re.search(field + r"=(\d+)", line).group(1))
    rows = [f"{index}:{','.join(map(str, row))}" for index, row in sorted(totals.items())]
    sums = [sum(row[position] for row in totals.values()) for position in range(5)]
    print(f"{group} histories={len(totals)} prefixes={sums[0]} crash_image_red={sums[1]} mount_refused={sums[2]} after_mount_red={sums[3]} deletion_scan_unreadable_verdicts={sums[4]} per_history_sha256_16={digest(rows)}")


def verbatim(group, prefix, starts):
    for line in lines_of(prefix):
        if line.startswith(starts):
            print(f"{group} {line}")


sweep("SWEEP", "sweep-")
crash("CRASH", "crash-")
verbatim("BAD", "badimages", ("BAD_IMAGE ", "BAD_IMAGE_SUMMARY"))
verbatim("GPUGAP", "gpugap", ("GPUGAP",))
verbatim("N9", "n9", ("N9 ",))
sweep("FIXA_SWEEP", "fixA-sweep-")
verbatim("FIXA_BAD", "fixA-badimages", ("BAD_IMAGE ", "BAD_IMAGE_SUMMARY"))
for line in lines_of("fixA-tests"):
    if line.startswith("test result:"):
        print("FIXA_TESTS " + re.sub(r"; finished in [0-9.]+s", "", line))
sweep("MUTC_SWEEP", "mutC-sweep")
crash("MUTC_CRASH", "mutC-crash")
sweep("SWEEPX", "sweepx-")
disk_loss = sorted(line for line in lines_of("diskloss") if line.startswith("DISKLOSS index="))
verbatim("DISKLOSS", "diskloss", ("DISKLOSS_SUMMARY",))
print(f"DISKLOSS_LINES count={len(disk_loss)} sha256_16={digest(disk_loss)}")

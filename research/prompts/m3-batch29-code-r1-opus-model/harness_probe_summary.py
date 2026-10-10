#!/usr/bin/env python3
"""rerun.sh harness-probe 段的摘要：每个测试目标的退出码、通过 / 失败的用例数、六格探针各被哪几个用例走到几次（不带计时）。
用法：python3 harness_probe_summary.py <工作目录/logs>
"""
import collections
import pathlib
import re
import sys

PROBE_NAMES = ["hv", "hb_inode", "hb_container", "unwalked", "unwalked_mismatch", "l1_type0"]


def main():
    logs = pathlib.Path(sys.argv[1])
    progress = {}
    for line in (logs / "harness-probe.progress").read_text(encoding="utf-8").splitlines():
        target, rc = line.split(" ")
        progress[target] = rc
    totals = collections.Counter()
    targets_with_hits = 0
    for target in sorted(progress):
        out = (logs / "harness-out" / f"{target}.log").read_text(encoding="utf-8", errors="replace")
        passed = sum(int(m.group(1)) for m in re.finditer(r"^test result: \w+\. (\d+) passed", out, re.M))
        failed = sum(int(m.group(1)) for m in re.finditer(r"^test result: \w+\. \d+ passed; (\d+) failed", out, re.M))
        totals["passed"] += passed
        totals["failed"] += failed
        hits = collections.Counter()
        probe_log = logs / "probe" / f"{target}.log"
        if probe_log.exists():
            for line in probe_log.read_text(encoding="utf-8", errors="replace").splitlines():
                parts = line.split(" ", 1)
                if len(parts) == 2 and parts[0].isdigit():
                    hits[(int(parts[0]), parts[1])] += 1
        if hits:
            targets_with_hits += 1
        print(f"RESULT harness target={target} {progress[target]} passed={passed} failed={failed} probe_hits={sum(hits.values())}")
        for (index, test), count in sorted(hits.items()):
            print(f"RESULT harness-hit target={target} probe={PROBE_NAMES[index]} test={test} hits={count}")
    print(f"RESULT harness-total targets={len(progress)} passed={totals['passed']} failed={totals['failed']} targets_with_probe_hits={targets_with_hits}")


if __name__ == "__main__":
    main()

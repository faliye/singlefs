#!/usr/bin/env python3
"""V2②：库目录这一版第一段的 nextest 日志里，每条测试 PASS 的耗时与基线同一条测试的耗时之比。
用法：per_test_timeout.py <库目录版本目录>
只读库目录。输出：按包分的比值分布；B1「限时 = k × 基线 + 下限」下，PASS 的测试有多少次会被判超时（即「抓到」），
以及因此整条变异由「没抓到」翻成「抓到」的条数（只数那一条变异日志里没有 FAIL 的）。"""
import csv, os, re, sys, statistics
D = sys.argv[1]
LINE = re.compile(r"^\s+(PASS|FAIL|SIGTERM)\s+\[\s*([0-9.]+)s\]\s+\(\s*[0-9─]+/\s*\d+\)\s+(\S+)\s+(\S+)")
def parse(path):
    out = []
    for line in open(path, encoding="utf-8", errors="replace"):
        m = LINE.match(line)
        if m:
            out.append((m.group(1), float(m.group(2)), m.group(3) + " " + m.group(4)))
    return out
base = {name: t for status, t, name in parse(os.path.join(D, "logs", "baseline.one-test.log")) if status == "PASS"}
rows = {r[0]: r for r in csv.reader((l for l in open(os.path.join(D, "results.tsv")) if not l.startswith("#")), delimiter="\t")}
print("baseline-tests", len(base), "baseline-sum-seconds", round(sum(base.values()), 1), "baseline-max", max(base.values()))
events = []  # (seq, outcome, name, t0, t, independent)
logs = sorted(f for f in os.listdir(os.path.join(D, "logs")) if re.match(r"^\d+\.one-test\.log$", f))
for f in logs:
    seq = f.split(".")[0]
    row = rows.get(seq)
    if not row or row[1] != "singlefs-core":
        continue
    parsed = parse(os.path.join(D, "logs", f))
    has_fail = any(s == "FAIL" for s, _, _ in parsed)
    for status, t, name in parsed:
        if status == "PASS" and name in base:
            independent = name.startswith("singlefs-checker") or name.startswith("singlefs-format")
            events.append((seq, row[3], has_fail, name, base[name], t, independent))
print("core-mutation-logs", len({e[0] for e in events}), "pass-events", len(events), "independent-pass-events", sum(e[6] for e in events))
for label, sel in (("independent(checker,format)", [e for e in events if e[6]]), ("dependent(core,harness)", [e for e in events if not e[6]])):
    ratios = sorted(e[5] / e[4] for e in sel if e[4] > 0)
    q = lambda p: ratios[min(len(ratios) - 1, int(p * len(ratios)))]
    print(f"ratio {label} n={len(ratios)} median={q(0.5):.2f} p90={q(0.9):.2f} p99={q(0.99):.2f} max={ratios[-1]:.2f}")
    big = sorted(sel, key=lambda e: e[5] - e[4], reverse=True)[:3]
    for e in big:
        print(f"  largest-excess {label} seq={e[0]} outcome={e[1]} test={e[3]} baseline={e[4]:.3f}s this-run={e[5]:.3f}s")
print("k floor_s | pass-events-over-limit (independent / all) | core mutations with no FAIL in log that B1 would turn into caught (by independent tests / by any test) of N")
no_fail = sorted({e[0] for e in events if not e[2]}, key=int)
for k in (1.5, 2, 3, 5, 10, 20):
    for floor in (0, 1, 5, 10, 30, 60):
        over = [e for e in events if e[5] > k * e[4] + floor]
        ind = [e for e in over if e[6]]
        flipped_ind = {e[0] for e in ind if not e[2]}
        flipped_all = {e[0] for e in over if not e[2]}
        print(f"k={k} floor={floor} | {len(ind)} / {len(over)} | {len(flipped_ind)} / {len(flipped_all)} of {len(no_fail)}")

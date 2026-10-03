#!/usr/bin/env python3
"""V2②续：只看结局是 caught 的变异日志（排掉 timeout），基线 ≥ 10 秒的测试在这些日志里 PASS 的耗时 / 基线耗时。
用法：long_tests.py <库目录版本目录>"""
import csv, os, re, sys
D = sys.argv[1]
LINE = re.compile(r"^\s+(PASS|FAIL|SIGTERM)\s+\[\s*([0-9.]+)s\]\s+\(\s*[0-9─]+/\s*\d+\)\s+(\S+)\s+(\S+)")
def parse(path):
    return [(m.group(1), float(m.group(2)), m.group(3) + " " + m.group(4)) for m in (LINE.match(l) for l in open(path, encoding="utf-8", errors="replace")) if m]
base = {n: t for s, t, n in parse(os.path.join(D, "logs", "baseline.one-test.log")) if s == "PASS"}
rows = {r[0]: r for r in csv.reader((l for l in open(os.path.join(D, "results.tsv")) if not l.startswith("#")), delimiter="\t")}
long = {n for n, t in base.items() if t >= 10}
print("baseline tests >=10s:", len(long), "of", len(base))
ev = []
for f in sorted(os.listdir(os.path.join(D, "logs"))):
    m = re.match(r"^(\d+)\.one-test\.log$", f)
    if not m or rows.get(m.group(1), [None] * 4)[3] != "caught":
        continue
    for s, t, n in parse(os.path.join(D, "logs", f)):
        if s == "PASS" and n in long:
            ev.append((m.group(1), n, base[n], t))
print("caught-logs pass events of long tests:", len(ev), "in", len({e[0] for e in ev}), "logs")
r = sorted(e[3] / e[2] for e in ev)
q = lambda p: r[min(len(r) - 1, int(p * len(r)))]
print(f"ratio median={q(.5):.2f} p90={q(.9):.2f} p99={q(.99):.2f} max={r[-1]:.2f} min={r[0]:.2f}")
for e in sorted(ev, key=lambda e: e[3] / e[2], reverse=True)[:5]:
    print(f"  top seq={e[0]} test={e[1]} baseline={e[2]:.3f}s this-run={e[3]:.3f}s ratio={e[3]/e[2]:.2f}")
for k in (1.5, 2, 3, 4, 5):
    for floor in (0, 10, 30, 60):
        over = [e for e in ev if e[3] > k * e[2] + floor]
        print(f"k={k} floor={floor}s over={len(over)} of {len(ev)} in-logs={len({e[0] for e in over})}")

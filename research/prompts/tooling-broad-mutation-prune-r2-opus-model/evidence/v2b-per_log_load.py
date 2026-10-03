#!/usr/bin/env python3
"""V2②续：每份 caught 日志里，长测试（基线 ≥10 秒）耗时比的中位数，与同一份日志里 singlefs-checker / singlefs-format 测试（core 的变异碰不到）耗时比的中位数。
两者在同一份日志里一起高，就是整趟变慢（负载），不是这条变异让某条测试变慢。用法：per_log_load.py <库目录版本目录>"""
import csv, os, re, sys, statistics
D = sys.argv[1]
LINE = re.compile(r"^\s+(PASS|FAIL|SIGTERM)\s+\[\s*([0-9.]+)s\]\s+\(\s*[0-9─]+/\s*\d+\)\s+(\S+)\s+(\S+)")
def parse(path):
    return [(m.group(1), float(m.group(2)), m.group(3) + " " + m.group(4)) for m in (LINE.match(l) for l in open(path, encoding="utf-8", errors="replace")) if m]
base = {n: t for s, t, n in parse(os.path.join(D, "logs", "baseline.one-test.log")) if s == "PASS"}
rows = {r[0]: r for r in csv.reader((l for l in open(os.path.join(D, "results.tsv")) if not l.startswith("#")), delimiter="\t")}
ind_base = sorted(t for n, t in base.items() if n.startswith(("singlefs-checker", "singlefs-format")))
print("independent tests in baseline:", len(ind_base), "max baseline seconds", ind_base[-1], "sum", round(sum(ind_base), 2))
out = []
for f in os.listdir(os.path.join(D, "logs")):
    m = re.match(r"^(\d+)\.one-test\.log$", f)
    if not m or rows.get(m.group(1), [None] * 4)[3] != "caught":
        continue
    p = parse(os.path.join(D, "logs", f))
    longr = [t / base[n] for s, t, n in p if s == "PASS" and n in base and base[n] >= 10]
    indr = [t / base[n] for s, t, n in p if s == "PASS" and n in base and n.startswith(("singlefs-checker", "singlefs-format")) and base[n] >= 0.01]
    if len(longr) >= 3:
        out.append((statistics.median(longr), len(longr), statistics.median(indr) if indr else None, len(indr), m.group(1), rows[m.group(1)][8]))
out.sort(reverse=True)
print("logs with >=3 long passing tests:", len(out))
print("median-long-ratio n-long median-independent-ratio(base>=0.01s) n-ind seq finished_at")
for o in out:
    print(f"{o[0]:.2f} {o[1]} {('%.2f' % o[2]) if o[2] is not None else '-'} {o[3]} {o[4]} {o[5]}")

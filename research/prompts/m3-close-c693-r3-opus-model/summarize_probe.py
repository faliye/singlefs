#!/usr/bin/env python3
"""攻方腿 m3-close-c693-r3 T2：把探针日志归并成「处 × 失败种类 × 草稿白名单退不退 × 今天退没退」，再列今天退了而草稿白名单不退的用例。
用法：python3 -I summarize_probe.py <探针日志>"""
import re, sys, collections
pat = re.compile(r"^C693F2 site=(\S+) thread=(\S+) failure=(.*) draft_whitelist_retreats=(true|false) judge=(.*) retreated=(.*)$")
kind = lambda f: re.sub(r"\s*\{.*", "", re.sub(r"^(Recovery|Walk)\((\w+).*", r"\1(\2)", f)) + (("{" + re.search(r'what: "([^"]*)"', f).group(1) + "}") if re.search(r'what: "([^"]*)"', f) else "")
rows = collections.Counter(); changed = collections.defaultdict(set); bad = 0
for line in open(sys.argv[1], encoding="utf-8"):
    m = pat.match(line.rstrip("\n"))
    if not m:
        bad += 1; continue
    site, thread, failure, draft, judge, retreated = m.groups()
    today = "none" if retreated in ("none",) else ("see_recovery" if retreated == "see_recovery" else "retreated")
    k = kind(failure)
    rows[(site, k, draft, today)] += 1
    if draft == "false" and today == "retreated":
        changed[(site, k)].add(thread)
print(f"name=probe_lines parsed={sum(rows.values())} unparsed={bad}")
for (site, k, draft, today), n in sorted(rows.items()):
    print(f"name=probe_cell site={site} failure_kind={k} draft_whitelist_retreats={draft} today={today} events={n}")
for (site, k), threads in sorted(changed.items()):
    for t in sorted(threads):
        print(f"name=today_retreats_but_draft_whitelist_does_not site={site} failure_kind={k} test={t}")

#!/usr/bin/env python3
import sys, os, json, warnings
warnings.filterwarnings("ignore")
here = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, here)
import feed, truth, cases_heavy, cases_heavy2, sandbox_reset
rows = []
for c in cases_heavy.CASES:
    if len(sys.argv) > 1 and c["id"] not in sys.argv[1:]: continue
    ran = None; calls = []
    sandbox_reset.reset()
    cwd = feed.TREE if c.get("cwd") == "TREE" else None
    rc, err, notes = feed.feed("heavy-test-guard.sh", c["cmd"], agent=c["agent"], env=c["env"], cwd=cwd)
    if c["truth"]:
        sandbox_reset.reset()
        trc, calls, tout = truth.truth(c["cmd"])
        ran = any(x.startswith("CARGO test") for x in calls) or ("CARGO" in " ".join(calls) and "test" in " ".join(calls))
        sandbox_reset.reset()
    first = err.strip().splitlines()[0] if err.strip() else ""
    klass = "符合" if rc == c["want"] else "不符"
    if ran and rc == 0: klass = "漏拒"
    elif ran is False and rc == 2 and c["want"] == 0: klass = "误拒"
    rows.append((c["id"], c["fam"], c["want"], rc, "" if ran is None else ("真起" if ran else "没起"), klass, first[:110], c["cmd"].replace("\n", "\\n")[:100], len(notes)))
for r in rows: print("\t".join(map(str, r)))

#!/usr/bin/env python3
import sys, os, warnings
warnings.filterwarnings("ignore")
here = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, here)
import feed, truth, cases_heavy, cases_fuzz, cases_fuzz2, sandbox_reset
rows = []
sel = sys.argv[1:]
for c in cases_heavy.CASES:
    if not c["id"][0] in "ZY": continue
    if sel and c["id"] not in sel: continue
    sandbox_reset.reset()
    rc, err, notes = feed.feed("heavy-test-guard.sh", c["cmd"], agent=c["agent"])
    sandbox_reset.reset()
    trc, calls, tout = truth.truth(c["cmd"])
    ran = any(x.startswith("CARGO test") for x in calls)
    sandbox_reset.reset()
    first = err.strip().splitlines()[0] if err.strip() else ""
    klass = "符合" if rc == 2 and ran else ("漏拒" if rc == 0 and ran else ("放过且没起" if rc == 0 else "拒了但没起"))
    rows.append((c["id"], rc, "真起" if ran else "没起", klass, first[7:70], c["cmd"].replace("\n", "\\n")[:90], len(notes)))
for r in rows: print("\t".join(map(str, r)))

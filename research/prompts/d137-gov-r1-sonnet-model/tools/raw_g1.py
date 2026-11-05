#!/usr/bin/env python3
"""打中清单里挑几条，贴钩子的原样 stderr 与真 bash 里 cargo 垫片收到的调用。"""
import sys, os, warnings
warnings.filterwarnings("ignore")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import feed, truth, sandbox_reset as R
from hits_g1 import HITS
pick = sys.argv[1:] or ["H1a", "H2a", "H3a", "H4c", "H5a", "H6a", "H7a", "H8a", "H10"]
H = dict(HITS)
for hid in pick:
    cmd = H[hid]
    R.reset()
    rc, err, notes = feed.feed("heavy-test-guard.sh", cmd, agent="implementation-writer")
    R.reset()
    trc, calls, tout = truth.truth(cmd)
    R.reset()
    print("=== %s  命令: %s" % (hid, cmd.replace("\n", "\\n")))
    print("钩子 退出码=%s" % rc)
    for l in err.strip().splitlines()[:2]: print("  stderr: " + l[:300])
    print("真 bash 里 cargo 垫片收到: %s" % (calls if calls else "没有调用"))

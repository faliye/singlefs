#!/usr/bin/env python3
"""把一批样本切给 N 个工人并行跑：工人 i 用自己的树 <work>/repo(i+1)（repo、repo2…），各写 <out>.part<i>，最后并成 <out>。
用法：run_shards.py --work D --corpus a,b --out out/x.jsonl [--only 前缀] [--workers 4]"""
import argparse, os, subprocess, sys
HERE = os.path.dirname(os.path.abspath(__file__))
ap = argparse.ArgumentParser()
ap.add_argument("--work", required=True); ap.add_argument("--corpus", required=True); ap.add_argument("--out", required=True)
ap.add_argument("--only", default=""); ap.add_argument("--stride", type=int, default=1); ap.add_argument("--ids", default=""); ap.add_argument("--slice", default="0:100000"); ap.add_argument("--workers", type=int, default=8)
a = ap.parse_args()
sys.path.insert(0, HERE)
import feed
for k in range(a.workers):
    repo = os.path.join(a.work, "repo" if k == 0 else f"repo{k + 1}")
    f = feed.feed(repo, a.work, "cargo test --workspace", role="sub", cwd=os.path.join(a.work, "wr", "copy"))
    if f["heavy-test-guard.sh"]["rc"] != 2:
        sys.exit(f"自检失败：{repo} 里的钩子对已知该拒的命令没拒（树坏了），停")
procs = []
for i in range(a.workers):
    repo = os.path.join(a.work, "repo" if i == 0 else f"repo{i + 1}")
    cmd = [sys.executable, os.path.join(HERE, "run_all.py"), "--repo", repo, "--copy", os.path.join(a.work, "wr", "copy"), "--work", a.work,
           "--corpus", a.corpus, "--out", f"{a.out}.part{i}", "--only", a.only, "--shard", f"{i}/{a.workers}", "--slice", a.slice, "--ids", a.ids, "--stride", str(a.stride)]
    procs.append(subprocess.Popen(cmd, stderr=open(f"{a.out}.err{i}", "w")))
rcs = [p.wait() for p in procs]
with open(a.out, "w") as fo:
    for i in range(a.workers):
        part = f"{a.out}.part{i}"
        if os.path.exists(part):
            fo.write(open(part).read().replace(os.path.join(a.work, "repo" + ("" if i == 0 else str(i + 1))), "{R}"))
            os.unlink(part)
print("workers rc:", rcs)

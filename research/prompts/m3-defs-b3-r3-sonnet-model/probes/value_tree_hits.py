#!/usr/bin/env python3 -I
"""整树数私有值命中份数（只打份数，不打值、不打文件名里含值的路径）。
用法：value_tree_hits.py <multi-host.env> <仓根> [--tracked-only]
扫 git ls-files（已跟踪，加未跟踪未忽略的）；二进制（前 8 KiB 带 NUL）跳过并报份数。
"""
import os, subprocess, sys, socket
cfg, root = sys.argv[1], sys.argv[2]
peer = None
for line in open(cfg, encoding="utf-8"):
    line = line.strip()
    if line.startswith("PEER_SSH_HOST="):
        peer = line.split("=", 1)[1].strip().strip("'\"")
host = socket.gethostname()
args = ["git", "-C", root, "ls-files", "-z", "--cached"] + ([] if "--tracked-only" in sys.argv else ["--others", "--exclude-standard"])
names = [n for n in subprocess.run(args, capture_output=True).stdout.decode("utf-8", "replace").split("\0") if n]
res = {"peer": 0, "host": 0}; nbin = 0; nread = 0; nmiss = 0
for n in names:
    p = os.path.join(root, n)
    try:
        b = open(p, "rb").read()
    except Exception:
        nmiss += 1; continue
    if b"\0" in b[:8192]:
        nbin += 1; continue
    nread += 1
    t = b.decode("utf-8", "replace")
    if peer and peer in t: res["peer"] += 1
    if host and host in t: res["host"] += 1
print("name=value_tree_hits files_listed=%d read=%d binary_skipped=%d unreadable=%d files_with_peer_value=%d files_with_hostname=%d" % (len(names), nread, nbin, nmiss, res["peer"], res["host"]))

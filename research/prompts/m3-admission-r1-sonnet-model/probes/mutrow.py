#!/usr/bin/env python3
"""草稿：在腿自己的仓副本上施一条 crates/mutations.tsv 的变异行，跑它点名的测试，还原。用法：mutrow.py <行号> [--baseline]"""
import os, shutil, subprocess, sys
root = os.environ["MODEL_TREE"]
table = os.path.join(os.environ["MODEL_TREE"], "crates", "mutations.tsv")
n = int(sys.argv[1]); baseline = "--baseline" in sys.argv
cols = open(table, encoding="utf-8").read().split("\n")[n - 1].split("\t")
path = os.path.join(root, cols[1]); old = cols[2].replace("\\n", "\n"); new = cols[3].replace("\\n", "\n")
src = open(path, encoding="utf-8").read()
hits = src.count(old)
print(f"row={n} file={cols[1]} anchor_hits={hits} baseline={baseline}", flush=True)
if hits != 1: sys.exit(3)
backup = path + ".orig"
shutil.copy(path, backup)
try:
    if not baseline:
        with open(path + ".tmp", "w", encoding="utf-8") as f: f.write(src.replace(old, new))
        os.replace(path + ".tmp", path)
    env = dict(os.environ, CARGO_TARGET_DIR=os.environ["MODEL_TARGET"])
    cmd = ["nice", "-n", "19", "cargo", "test", "--offline", "--manifest-path", root + "/Cargo.toml"] + os.environ["MUTROW_ARGS"].split()
    p = subprocess.run(cmd, env=env, capture_output=True, text=True)
    out = p.stdout + p.stderr
    print(f"exit={p.returncode}")
    for line in out.splitlines():
        if line.startswith("test ") or "test result" in line or line.startswith("error") or "panicked" in line or line.startswith("  left") or line.startswith(" right"):
            print(line[:240])
finally:
    os.replace(backup, path); os.utime(path)

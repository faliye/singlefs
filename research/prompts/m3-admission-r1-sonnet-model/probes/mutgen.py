#!/usr/bin/env python3
"""草稿：对腿自己的仓副本施一处临时改动（文件 旧 新 参数），跑 cargo test，还原并 touch。用法：mutgen.py <相对路径> <旧文本文件> <新文本文件> <cargo test 参数…>"""
import os, shutil, subprocess, sys
root = os.environ["MODEL_TREE"]
rel, oldf, newf, args = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4:]
old = open(oldf, encoding="utf-8").read(); new = open(newf, encoding="utf-8").read()
path = os.path.join(root, rel); src = open(path, encoding="utf-8").read()
hits = src.count(old); print(f"file={rel} anchor_hits={hits}", flush=True)
if hits != 1: sys.exit(3)
backup = path + ".orig"; shutil.copy(path, backup)
try:
    with open(path + ".tmp", "w", encoding="utf-8") as f: f.write(src.replace(old, new))
    os.replace(path + ".tmp", path)
    env = dict(os.environ, CARGO_TARGET_DIR=os.environ["MODEL_TARGET"])
    p = subprocess.run(["nice", "-n", "19", "cargo", "test", "--offline", "--manifest-path", root + "/Cargo.toml"] + args, env=env, capture_output=True, text=True)
    out = p.stdout + p.stderr
    print(f"exit={p.returncode}")
    for line in out.splitlines():
        if line.startswith("test ") and ("FAILED" in line) or "test result" in line or line.startswith("error"):
            print(line[:230])
finally:
    os.replace(backup, path); os.utime(path)

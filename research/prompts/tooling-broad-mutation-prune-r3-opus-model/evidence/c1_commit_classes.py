#!/usr/bin/env python3
"""最近 N 个动了导出范围（crates/ Cargo.toml Cargo.lock litmus/ .cargo/）的提交，按改动落在哪一类分：
  src  = 被变异的两个包（singlefs-core、singlefs-checker）src 下的 .rs（C1 看得见的那一类；含 src 里的 #[cfg(test)] 模块）
  test = 其余 crates 下的 .rs（harness、format、checker-tier、replay-prefix，以及 core / checker 的 tests/）
  data = 导出范围里不是 .rs 的（Cargo.toml、Cargo.lock、litmus/、.cargo/、crates 下的数据文件）
用法：c1_commit_classes.py <仓根> <N>
"""
import subprocess
import sys

root, count = sys.argv[1], int(sys.argv[2])
paths = ["crates", "Cargo.toml", "Cargo.lock", "litmus", ".cargo"]
commits = subprocess.run(["git", "log", f"-{count}", "--format=%h", "--", *paths], cwd=root, capture_output=True, text=True, check=True).stdout.split()
totals = {"src": 0, "test-only": 0, "data-only": 0, "test-or-data-without-src": 0}
for commit in commits:
    files = subprocess.run(["git", "show", "--name-only", "--format=", commit, "--", *paths], cwd=root, capture_output=True, text=True, check=True).stdout.split()
    src = [f for f in files if f.endswith(".rs") and (f.startswith("crates/singlefs-core/src/") or f.startswith("crates/singlefs-checker/src/"))]
    test = [f for f in files if f.endswith(".rs") and f not in src]
    data = [f for f in files if not f.endswith(".rs")]
    totals["src"] += bool(src)
    totals["test-only"] += bool(test) and not src and not data
    totals["data-only"] += bool(data) and not src and not test
    totals["test-or-data-without-src"] += bool(test or data) and not src
    print(f"C1-COMMIT {commit} src-files={len(src)} test-files={len(test)} data-files={len(data)} data={','.join(data)[:120]}")
print("C1-COMMIT-TOTALS commits=" + str(len(commits)) + " " + " ".join(f"{k}={v}" for k, v in totals.items()))

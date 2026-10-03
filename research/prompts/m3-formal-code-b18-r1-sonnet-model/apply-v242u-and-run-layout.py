#!/usr/bin/env python3
"""在副本 WS 的 transaction.rs 上套 V242u（行文取自 _m3-formal-code-b18-r1-diff.md 第 665 行的旧串与新串，TSV 里的 \\n 换成真换行），
跑草稿探针 scratch_layout（只量切段与挂载写行段的写布局，不判任何东西），还原。用法：apply-v242u-and-run-layout.py <副本工作区根> <仓根> <日志>"""
import os, subprocess, sys

ws, repo, log = sys.argv[1], sys.argv[2], sys.argv[3]
line = open(os.path.join(repo, "research/prompts/_m3-formal-code-b18-r1-diff.md"), encoding="utf-8").read().splitlines()[664]
fields = line.split("\t")
old, new = fields[2].replace("\\n", "\n"), fields[3].replace("\\n", "\n")
path = os.path.join(ws, "crates/singlefs-core/src/transaction.rs")
original = open(path, encoding="utf-8").read()
assert original.count(old) == 1, original.count(old)
open(path, "w", encoding="utf-8").write(original.replace(old, new))
try:
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(os.path.dirname(ws), "target"),
               RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同会话里别的腿各跑各的，我只跑一个小探针")
    build = subprocess.run(["nice", "-n", "19", "bash", f"{repo}/research/scripts/capped.sh", "6", "cargo", "test", "--offline", "-p", "singlefs-harness", "--test", "scratch_layout", "--no-run"], cwd=ws, env=env, capture_output=True, text=True)
    assert build.returncode == 0, build.stderr[-2000:]
    cmd = ["nice", "-n", "19", "bash", f"{repo}/research/scripts/capped.sh", "6", "bash",
           f"{repo}/research/scripts/run-with-memory-cap.sh", "auto", "cargo", "test", "--offline", "-p", "singlefs-harness",
           "--test", "scratch_layout", "--", "--nocapture"]
    out = subprocess.run(cmd, cwd=ws, env=env, capture_output=True, text=True)
    open(log, "w", encoding="utf-8").write(out.stdout + out.stderr)
    print("exit", out.returncode)
finally:
    open(path, "w", encoding="utf-8").write(original)

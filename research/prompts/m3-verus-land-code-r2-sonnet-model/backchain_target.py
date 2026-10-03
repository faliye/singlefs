#!/usr/bin/env python3
"""back_chain_inverted 改动在 crates/mutations.tsv 第 920 行点名的那个 harness 目标上红不红。"""
import subprocess, pathlib, os
ROOT = pathlib.Path("/tmp/claude-1000/m3-verus-land-code-r2-sonnet")
SRC = ROOT / "tree_g/crates/singlefs-core/src/recovery.rs"
p = SRC.read_text()
old, new = "BackChainJudgement::Broken => true,", "BackChainJudgement::Broken => false,"
assert p.count(old) == 1
env = {**os.environ, "RUN_WITH_MEMORY_CAP_WAIT_SECONDS": "3000", "RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON": "派发提示给定内存上限八G，胶水变异只跑点名的 harness 目标", "RUN_WITH_MEMORY_CAP_PARALLEL_REASON": "同会话几个腿各跑各的，这一条是胶水变异"}
try:
    SRC.write_text(p.replace(old, new))
    r = subprocess.run(["nice", "-n", "19", "bash", "/home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh", "8G", "bash", "/home/fy5090/code/singlefs/research/scripts/capped.sh", "6", "cargo", "test", "--release", "--offline", "-p", "singlefs-harness", "--test", "core_review_geometry_back_chain_and_empty_inode"], cwd=ROOT / "tree_g", env=env, capture_output=True, text=True)
    (ROOT / "glue_back_chain_inverted_row920.log").write_text(r.stdout + r.stderr)
    print("退出码", r.returncode)
finally:
    SRC.write_text(p)

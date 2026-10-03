#!/usr/bin/env python3
"""同一批核改动（mutate_core.M），只跑新 crate 自带的两条单测（cargo test -p singlefs-replay-prefix --lib），看红不红。用法：mutate_unit.py <草稿根>"""
import subprocess, sys, pathlib, os, re
sys.path.insert(0, str(pathlib.Path(__file__).parent))
ROOT = pathlib.Path(sys.argv[1])
sys.argv = [sys.argv[0], str(ROOT)]
import mutate_core
LIB = mutate_core.LIB
base = mutate_core.PRISTINE.read_text()
env = {**os.environ, "RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON": "派发提示给定内存上限八G，单测变异按提示上限跑",
       "RUN_WITH_MEMORY_CAP_PARALLEL_REASON": "同会话派出的几个腿各跑各的，这一条是两条单测"}
for name, old, new, prediction, why in mutate_core.M:
    assert base.count(old) == 1, name
    LIB.write_text(base.replace(old, new))
    r = subprocess.run(["bash", "/home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh", "8G", "bash", "/home/fy5090/code/singlefs/research/scripts/capped.sh", "10", "cargo", "test", "--release", "--offline", "-p", "singlefs-replay-prefix", "--lib"], cwd=ROOT / "tree_m", env=env, capture_output=True, text=True)
    out = r.stdout + r.stderr
    summary = re.findall(r"test result: [^\n]*", out)
    print(f"UNIT {name} | 退出码={r.returncode} | {summary[-1] if summary else '无 test result（' + out.strip().splitlines()[-1][:80] + '）'}", flush=True)
LIB.write_text(base)

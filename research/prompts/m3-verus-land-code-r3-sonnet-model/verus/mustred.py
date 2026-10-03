#!/usr/bin/env python3
"""辩方 J5：必红世界表逐行判（照 verus-proof-check.py 的口径：期望片段在 Verus 输出里出现，且除 rlimit 用尽与 aborting due to 之外至少还有一条 error）。
用法：mustred.py <冻结拷贝根（含 crates/singlefs-replay-prefix）> <表> <出口目录> [行号…]；每行在干净副本里套一次，跑完删副本。"""
import sys, os, shutil, subprocess, pathlib, re
root, table, out = map(pathlib.Path, sys.argv[1:4])
only = {int(x) for x in sys.argv[4:]}
out.mkdir(parents=True, exist_ok=True)
verus = os.path.expanduser("~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus")
env = {**os.environ, "RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON": "Verus 峰值几百 M，取 2G 在派发给的 12G 之内", "RUN_WITH_MEMORY_CAP_PARALLEL_REASON": "同会话几个腿各跑各的", "RUN_WITH_MEMORY_CAP_WAIT_SECONDS": "900"}
lines = table.read_text(encoding="utf-8").split("\n")
summary = out / "summary.tsv"
for number, line in enumerate(lines, 1):
    if not line.strip() or line.startswith("#"): continue
    if only and number not in only: continue
    cols = line.split("\t")
    name, rel, orig, repl, expected = cols[0], cols[1], cols[2].replace("\\n", "\n"), cols[3].replace("\\n", "\n"), cols[4]
    work = out / f"work-{number}"
    if work.exists(): shutil.rmtree(work)
    (work / "crates").mkdir(parents=True)
    shutil.copytree(root / "crates/singlefs-replay-prefix", work / "crates/singlefs-replay-prefix")
    target = work / rel
    text = target.read_text(encoding="utf-8")
    hits = text.count(orig)
    if hits != 1:
        with summary.open("a") as f: f.write(f"{number}\tANCHOR hits={hits}\t{name}\n")
        shutil.rmtree(work); continue
    target.write_text(text.replace(orig, repl, 1), encoding="utf-8")
    cp = subprocess.run(["nice", "-n", "19", "bash", "/home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh", "2G", verus, "--crate-type=lib", "--edition=2021", "--crate-name", "singlefs_replay_prefix", "--no-cheating", "--rlimit", "10", "src/lib.rs"], cwd=work / "crates/singlefs-replay-prefix", env=env, capture_output=True, text=True)
    log = cp.stdout + cp.stderr
    (out / f"verus-{number}.log").write_text(log)
    result = re.findall(r"verification results:: \d+ verified, \d+ errors", log)
    result = result[-1] if result else "NO-VERDICT"
    errors = [l for l in log.split("\n") if l.startswith("error")]
    proof_errors = [l for l in errors if "rlimit" not in l.lower() and "aborting due to" not in l]
    rlimit_lines = sum(1 for l in log.split("\n") if "rlimit" in l.lower() and "run-with-memory-cap" not in l)
    red = expected in log and len(proof_errors) >= 1
    with summary.open("a") as f:
        f.write(f"{number}\t{'RED' if red else 'NOT-RED'}\t{result}\texpected={expected}\tproof_errors={len(proof_errors)}\trlimit_lines={rlimit_lines}\t{name}\n")
    shutil.rmtree(work)

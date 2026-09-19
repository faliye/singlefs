import os, re, subprocess, sys, time
base = "/tmp/claude-1000/m2-supp3-item1/r3"
points = [("broad", 96, 30), ("reuse", 48, 30), ("broad", 96, 20), ("reuse", 48, 20), ("broad", 96, 40), ("reuse", 48, 40), ("reuse", 96, 30), ("broad", 48, 30)]
copies = sys.argv[1:]
out = open(f"{base}/sweep.tsv", "a", encoding="utf-8")
for name in copies:
    repo = f"{base}/{name}/repo"
    subprocess.run(["nice", "-n", "19", "cargo", "build", "--release", "-p", "singlefs-harness", "--tests"], cwd=repo, capture_output=True)
    for weights, seeds, operations in points:
        environment = dict(os.environ, SINGLEFS_RANDOM_HISTORY_SEEDS=str(seeds), SINGLEFS_RANDOM_HISTORY_OPERATIONS=str(operations), SINGLEFS_RANDOM_HISTORY_THREADS="32", SINGLEFS_RANDOM_HISTORY_WEIGHTS=weights, SINGLEFS_RANDOM_HISTORY_SHRINK="none", SINGLEFS_RANDOM_HISTORY_FIRST_SEED="0")
        started = time.time()
        run = subprocess.run(["nice", "-n", "19", "cargo", "test", "--release", "-p", "singlefs-harness", "--test", "second_transaction_supplement_three_random_history", "--", "--ignored", "random_histories_large_tier", "--nocapture"], cwd=repo, env=environment, capture_output=True, text=True)
        text = run.stdout + run.stderr
        label = f"{weights}-{seeds}x{operations}"
        open(f"{base}/{name}/sweep-{label}.log", "w", encoding="utf-8").write(text)
        summary = next((line for line in text.splitlines() if line.startswith("历史 ")), "（没有汇总行）")
        seeds_line = next((line for line in text.splitlines() if line.startswith("每个新发现的种子")), "")
        pairs = re.findall(r"\((\d+), (\w+) \{", seeds_line)
        red_seeds = sorted({int(seed) for seed, _ in pairs})
        signatures = sorted({kind for _, kind in pairs})
        capacity = next((line.strip() for line in text.splitlines() if "内容长度 载荷容量：" in line), "")
        out.write(f"{name}\t{label}\texit {run.returncode}\t{summary}\t红的种子 {len(red_seeds)} {red_seeds[:12]}\t{signatures}\t{capacity}\t{time.time() - started:.0f}s\n")
        out.flush()
out.write(f"== done {copies}\n")

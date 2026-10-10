#!/usr/bin/env python3
# 复跑变异行：在草稿目录的冻结副本拷贝上，逐行改坏一次、跑点名的测试、还原。用法 mut.py 行号...
import subprocess, sys, shutil, os, re
TREE = "/tmp/claude-1000/m3-batch20-code-r1-sonnet/work/tree"
PRIST = "/tmp/claude-1000/m3-batch20-code-r1/tree"
OUT = "/tmp/claude-1000/m3-batch20-code-r1-sonnet/mut"
rows = open(PRIST + "/crates/mutations.tsv", encoding="utf-8").read().split("\n")
env = dict(os.environ, CARGO_TARGET_DIR="/tmp/claude-1000/m3-batch20-code-r1-sonnet/target",
           RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方腿正推腿与攻方腿是各自派出的子 agent 各跑各的")
for n in map(int, sys.argv[1:]):
    f = rows[n - 1].split("\t")
    assert len(f) == 6, (n, len(f))
    name, path, old, new, args, test = f
    old = old.replace("\\n", "\n"); new = new.replace("\\n", "\n")
    full = os.path.join(TREE, path)
    src = open(os.path.join(PRIST, path), encoding="utf-8").read()
    cnt = src.count(old)
    if cnt != 1:
        print(f"row {n}: anchor hits {cnt}"); continue
    open(full, "w", encoding="utf-8").write(src.replace(old, new))
    cmd = ["nice", "-n", "19", "bash", "/home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh", "6G",
           "bash", "/home/fy5090/code/singlefs/research/scripts/capped.sh", "6", "cargo", "test", "--offline"] + args.split() + ["--", test]
    r = subprocess.run(cmd, cwd=TREE, env=env, capture_output=True, text=True)
    m = re.search(r"不超过 (\d+)M 的上限", r.stderr)
    if r.returncode == 251 and m:
        cmd[5] = m.group(1) + "M"
        r = subprocess.run(cmd, cwd=TREE, env=env, capture_output=True, text=True)
    open(f"{OUT}/row{n}.log", "w").write(r.stdout + "\n--stderr--\n" + r.stderr)
    open(full, "w", encoding="utf-8").write(src)
    red = re.search(r"test .*" + re.escape(test) + r" \.\.\. FAILED", r.stdout) is not None
    summ = [l for l in r.stdout.splitlines() if l.startswith("test result")]
    print(f"row {n}: exit={r.returncode} named_test_failed={red} {summ} err_has_error={'error[' in r.stderr}")

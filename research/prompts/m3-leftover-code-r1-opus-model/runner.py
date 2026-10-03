#!/usr/bin/env python3
"""攻方腿副本上的变异跑法：施加一个变异（每处替换恰好命中一次），编，按用例逐条并行跑（每条 1 线程），恢复原文件。
用法：runner.py <变异名> <变异表.json> <用例文件名…>；变异表里 {名: [[相对路径, 旧串, 新串], …]}；名为 base 时不改。"""
import json, os, subprocess, sys, concurrent.futures, re, time
D = os.environ.get("OPUS_DIR", "/tmp/claude-1000/m3-leftover-code-r1-opus")
WORK = D + "/work"
JOBS = int(os.environ.get("OPUS_JOBS", "6"))
name, table_path, *tests = sys.argv[1:]
table = json.load(open(table_path))
edits = table.get(name, [])
originals = {}
import shutil, signal
CLEAN = D + "/clean"
def restore_clean():
    for root, _, files in os.walk(CLEAN):
        for f in files:
            src = os.path.join(root, f)
            shutil.copyfile(src, WORK + src[len(CLEAN):])
def on_term(signum, frame):
    raise SystemExit(143)
signal.signal(signal.SIGTERM, on_term)
restore_clean()
out_dir = f"{D}/mut/out-{name}"
os.makedirs(out_dir, exist_ok=True)
try:
    for rel, old, new in edits:
        path = f"{WORK}/{rel}"
        text = originals.get(path) or open(path).read()
        originals.setdefault(path, text)
        current = open(path).read()
        count = current.count(old)
        if count != 1:
            print(f"MUTANT-APPLY-FAILED {name} {rel} hits={count}")
            sys.exit(3)
        open(path, "w").write(current.replace(old, new))
    env = dict(os.environ, CARGO_TARGET_DIR=D + "/target")
    args = ["cargo", "test", "--offline", "-p", "singlefs-harness", "--no-run"]
    for t in tests:
        args += ["--test", t]
    build = subprocess.run(args, cwd=WORK, env=env, capture_output=True, text=True)
    open(f"{out_dir}/build.log", "w").write(build.stderr)
    if build.returncode != 0:
        print(f"MUTANT-BUILD-FAILED {name} rc={build.returncode}")
        print("\n".join(l for l in build.stderr.splitlines() if l.startswith("error"))[:2000])
        sys.exit(4)
    exes = dict(re.findall(r"Executable tests/(\S+)\.rs \((\S+)\)", build.stderr))
    cases = []
    for t in tests:
        exe = exes[t]
        listed = subprocess.run([exe, "--list", "--format", "terse"], capture_output=True, text=True).stdout
        for line in listed.splitlines():
            if line.endswith(": test"):
                cases.append((t, exe, line[: -len(": test")]))
    def run(case):
        t, exe, case_name = case
        started = time.time()
        p = subprocess.run([exe, "--exact", case_name, "--test-threads", "1"], capture_output=True, text=True)
        return (t, case_name, p.returncode, round(time.time() - started, 1), p.stdout[-3000:] + p.stderr[-3000:])
    results = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=JOBS) as pool:
        for r in pool.map(run, cases):
            results.append(r)
    assert len(results) == len(cases), "派出去的与收回来的条数对不上"
    failed = [r for r in results if r[2] != 0]
    with open(f"{out_dir}/cases.tsv", "w") as f:
        for t, c, rc, sec, _ in results:
            f.write(f"{t}\t{c}\trc={rc}\t{sec}s\n")
    with open(f"{out_dir}/failed.log", "w") as f:
        for t, c, rc, sec, tail in failed:
            f.write(f"===== {t}::{c} rc={rc}\n{tail}\n")
    print(f"name={name} cases={len(results)} failed={len(failed)}")
    for t, c, rc, sec, _ in failed:
        print(f"  FAILED {t}::{c} rc={rc}")
finally:
    restore_clean()

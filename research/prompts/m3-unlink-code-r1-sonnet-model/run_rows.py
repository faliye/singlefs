#!/usr/bin/env python3
"""草稿副本里逐行套变异表的行、跑它点名的测试、看必须红的有没有红；每行跑完还原。输出追加到 rows-result.txt。"""
import os, shlex, subprocess, sys, time, shutil
WORK = "/tmp/claude-1000/m3-unlink-code-r1-sonnet/work"
OUT = "/tmp/claude-1000/m3-unlink-code-r1-sonnet/rows"
os.makedirs(OUT, exist_ok=True)
rows = open(f"{WORK}/crates/mutations.tsv", encoding="utf-8").read().split("\n")
env = dict(os.environ)
env.update({
    "CARGO_TARGET_DIR": "/tmp/claude-1000/m3-unlink-code-r1-sonnet/target-release",
    "RUN_WITH_MEMORY_CAP_PARALLEL_REASON": "三方腿并行派发：正推腿与攻方腿各跑各的探针",
    "RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON": "改过 core 要重编，峰值含 rustc",
})
for number in map(int, sys.argv[1:]):
    cols = rows[number - 1].split("\t")
    name, path, old, new, args, required = cols[:6]
    old, new = old.replace("\\n", "\n"), new.replace("\\n", "\n")
    full = f"{WORK}/{path}"
    original = open(full, encoding="utf-8").read()
    if original.count(old) != 1:
        line = f"行 {number}\t锚点命中 {original.count(old)} 次，没套\t{name[:60]}"
        open(f"{OUT}/../rows-result.txt", "a").write(line + "\n"); print(line); continue
    started = time.time()
    open(full, "w", encoding="utf-8").write(original.replace(old, new))
    try:
        command = ["nice", "-n", "19", "bash", "/home/fy5090/code/singlefs/scripts/run-with-memory-cap.sh", "8G",
                   "bash", "/home/fy5090/code/singlefs/scripts/capped.sh", "10",
                   "cargo", "test", "--release", "--offline"] + shlex.split(args)
        result = subprocess.run(command, cwd=WORK, env=env, capture_output=True, text=True)
        log = result.stdout + result.stderr
        open(f"{OUT}/row{number}.log", "w").write(log)
    finally:
        open(full, "w", encoding="utf-8").write(original)
    names = required.split()
    failed = [n for n in names if f"test {n} ... FAILED" in log]
    compile_error = "error[E" in log or "could not compile" in log
    verdict = "编不过" if compile_error else ("红" if len(failed) == len(names) and names else "没红")
    line = f"行 {number}\t{verdict}\t退出码 {result.returncode}\t用时 {time.time() - started:.0f} 秒\t{name[:70]}"
    open(f"{OUT}/../rows-result.txt", "a").write(line + "\n")
    print(line, flush=True)

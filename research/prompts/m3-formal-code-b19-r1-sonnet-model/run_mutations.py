#!/usr/bin/env python3
"""正推腿：把冻结副本 crates/mutations.tsv 第 2292-2303 行（19-1 追加的 12 条变异）逐条打在自己的 ws 副本 walk.rs 上，
跑那一行点名的测试，原样记退出码；每条跑完把 walk.rs 还原。K3 的 ⑤ 四条另跑 kbi_print 逐镜像打印版，看哪一张镜像的红灭了。
用法：python3 run_mutations.py <ws 根> <tsv> <起行> <止行> <日志目录>   （cargo 经 run-with-memory-cap 与 capped.sh）"""
import os, subprocess, sys, shutil
ws, tsv, first, last, logdir = sys.argv[1], sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), sys.argv[5]
os.makedirs(logdir, exist_ok=True)
rows = open(tsv, encoding="utf-8").read().split("\n")
env = dict(os.environ)
env["CARGO_TARGET_DIR"] = os.environ.get("CARGO_TARGET_DIR", "/tmp/claude-1000/m3-formal-code-b19-r1-sonnet/target")
env["RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON"] = "副本上的变异跑法，峰值表按原命令键登记、这里编的是副本"
env["RUN_WITH_MEMORY_CAP_PARALLEL_REASON"] = "云端正推腿与云端攻方腿各跑各的探针，并行派出"
REPO = "/home/fy5090/code/singlefs"
def run(args, log):
    cmd = ["nice", "-n", "19", "bash", f"{REPO}/research/scripts/run-with-memory-cap.sh", "4G", "bash", f"{REPO}/research/scripts/capped.sh", "4", "cargo", "test"] + args
    with open(log, "w") as f:
        p = subprocess.run(cmd, cwd=ws, env=env, stdout=f, stderr=subprocess.STDOUT, timeout=230)
    return p.returncode
for n in range(first, last + 1):
    desc, path, old, new, args, test = rows[n - 1].split("\t")[:6]
    old, new = old.replace("\\n", "\n"), new.replace("\\n", "\n")
    target = os.path.join(ws, path)
    original = open(target, encoding="utf-8").read()
    if original.count(old) != 1:
        print(f"row {n}: old 在 walk.rs 里命中 {original.count(old)} 次，跳过"); continue
    open(target, "w", encoding="utf-8").write(original.replace(old, new))
    try:
        argv = args.split()
        if test and "--" not in argv:
            argv += ["--", test]
        code = run(argv, f"{logdir}/row{n}.log")
        extra = ""
        if "checker_known_bad_images" in args and len(sys.argv) > 6 and sys.argv[6] == "print":
            code2 = run(["-p", "singlefs-harness", "--test", "kbi_print", "--", "--nocapture", "each_pointer"], f"{logdir}/row{n}-print.log")
            extra = f" print_exit={code2}"
        print(f"row {n}: exit={code}{extra} :: {desc[:60]}", flush=True)
    finally:
        open(target, "w", encoding="utf-8").write(original)

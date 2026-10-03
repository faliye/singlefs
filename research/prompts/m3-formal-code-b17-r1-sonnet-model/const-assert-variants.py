#!/usr/bin/env python3
"""K3：在草稿目录的副本上改 crash_judge_gpu.rs 的两张表，各跑一次 cargo check（只编 singlefs-checker-tier 的库、开 gpu 特性（crash_judge_gpu 模块在这个特性后面），不跑测试），看覆盖断言红不红。
用法：const-assert-variants.py <工作区> <日志目录>"""
import os, subprocess, sys, shutil
ws, logdir = sys.argv[1], sys.argv[2]
os.makedirs(logdir, exist_ok=True)
path = os.path.join(ws, "crates/singlefs-checker-tier/src/crash_judge_gpu.rs")
src = open(path, encoding="utf-8").read()
cov_old = "    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67,\n"
nj_old = "pub const POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET: &[usize] = &[65];"
assert src.count(cov_old) == 1 and src.count(nj_old) == 1
base = "    50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,"
def cov(tail): return base + tail + "\n"
def nj(items): return "pub const POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET: &[usize] = &[" + items + "];"
variants = [
    ("V0 原样（今天的两张表）", cov_old, nj_old),
    ("V1 覆盖表少 67（I-5.5 没登记）", cov(" 66,"), nj_old),
    ("V2 66 同时在两张表里", cov_old, nj("65, 66")),
    ("V3 覆盖表 66 写两遍、67 缺（条数对、升序破）", cov(" 66, 66,"), nj_old),
    ("V4 覆盖表 65 与还没判表重复、66 缺（条数对）", cov(" 65, 67,"), nj_old),
    ("V5 覆盖表 67 写成 68（条数对、67 缺）", cov(" 66, 68,"), nj_old),
    ("V6 还没判表点名 67、覆盖表不含 67（把 I-5.5 登记成内核没判，应当编得过）", cov(" 66,"), nj("65, 67")),
    ("V7 还没判表空、覆盖表 0..=67 全列（根环线第 9 批删表之后的形态，应当编得过）", cov(" 65, 66, 67,"), nj("")),
    ("V8 还没判表空、覆盖表 0..=67 少 65（删表而 I-3.5 没补，应当编不过）", cov(" 66, 67,"), nj("")),
]
scripts = "/home/fy5090/code/singlefs/research/scripts"
try:
    for name, cov_new, nj_new in variants:
        text = src.replace(cov_old, cov_new).replace(nj_old, nj_new)
        open(path, "w", encoding="utf-8").write(text)
        log = os.path.join(logdir, name.split()[0] + ".log")
        with open(log, "w") as f:
            p = subprocess.run(["nice", "-n", "19", "bash", f"{scripts}/capped.sh", "8", "cargo", "check", "--offline",
                                "-p", "singlefs-checker-tier", "--lib", "--features", "gpu"], cwd=ws, stdout=f, stderr=subprocess.STDOUT, timeout=230)
        out = open(log, encoding="utf-8", errors="replace").read().split("\n")
        errs = [l for l in out if "evaluation of constant value failed" in l or l.startswith("error")]
        msgs = [l.strip() for l in out if "the evaluated program panicked" in l or "panicked at" in l]
        print(f"{name}: cargo check 退出码 {p.returncode}")
        for l in (msgs or errs)[:2]:
            print("    " + l)
finally:
    open(path, "w", encoding="utf-8").write(src)
    os.utime(path, None)

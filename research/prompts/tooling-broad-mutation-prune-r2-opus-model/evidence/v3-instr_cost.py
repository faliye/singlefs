#!/usr/bin/env python3
"""V3：插桩的倍数与逐条覆盖的处理开销。用法：instr_cost.py <副本> <llvm 工具目录> <草稿目录> <harness 测试目标…>
core 的库测试与点名的 harness 测试目标各编一份不插桩、一份插桩（-C instrument-coverage）；
① 整份二进制一次跑全部测试（--test-threads=12）的墙钟；② 逐条测试单独起进程的墙钟合计；③ 插桩逐条之后 llvm-profdata merge 与 llvm-cov export 每条的耗时；
④ 每条的 profraw、profdata、export JSON 的大小，与「执行到的文件 + 函数」写成名字表的大小。"""
import json, os, statistics, subprocess, sys, time
REPO, LLVM, WORK = sys.argv[1:4]; TARGETS = sys.argv[4:]
def build(target_dir, instrumented, args):
    env = dict(os.environ, CARGO_TARGET_DIR=target_dir)
    if instrumented:
        env["RUSTFLAGS"] = "-C instrument-coverage"; env["LLVM_PROFILE_FILE"] = os.path.join(WORK, "junk-%p.profraw")
    t = time.time()
    r = subprocess.run(["cargo", "test", "--offline", *args, "--no-run", "--message-format=json"], cwd=REPO, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    exes = {}
    for l in r.stdout.splitlines():
        if l.startswith("{"):
            m = json.loads(l)
            if m.get("reason") == "compiler-artifact" and m.get("executable") and m["target"]["kind"] in (["test"], ["lib"]):
                exes[m["target"]["name"]] = m["executable"]
    return exes, time.time() - t, r.returncode
def tests_of(exe, cwd):
    out = subprocess.run([exe, "--list", "--format", "terse"], cwd=cwd, env=dict(os.environ, LLVM_PROFILE_FILE=os.path.join(WORK, "junk-%p.profraw")), stdout=subprocess.PIPE, text=True).stdout
    return [l.split(": ")[0] for l in out.splitlines() if l.endswith(": test")]
def timed(cmd, cwd, env=None):
    t = time.time(); r = subprocess.run(cmd, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True); return time.time() - t, r
def measure(label, plain, cov, cwd):
    tests = tests_of(plain, cwd)
    whole_plain, _ = timed([plain, "--test-threads=12"], cwd)
    whole_cov, _ = timed([cov, "--test-threads=12"], cwd, dict(os.environ, LLVM_PROFILE_FILE=os.path.join(WORK, "whole-%p.profraw")))
    per_plain = per_cov = merge = export = 0.0; sizes = []; files_n = []; funcs_n = []
    prof = os.path.join(WORK, "prof"); os.makedirs(prof, exist_ok=True)
    for t in tests:
        a, _ = timed([plain, "--exact", t], cwd); per_plain += a
        for f in os.listdir(prof): os.remove(os.path.join(prof, f))
        b, _ = timed([cov, "--exact", t], cwd, dict(os.environ, LLVM_PROFILE_FILE=os.path.join(prof, "t-%p.profraw"))); per_cov += b
        raws = [os.path.join(prof, f) for f in os.listdir(prof)]; data = os.path.join(WORK, "t.profdata")
        c, _ = timed([os.path.join(LLVM, "llvm-profdata"), "merge", "-sparse", *raws, "-o", data], cwd); merge += c
        d, r = timed([os.path.join(LLVM, "llvm-cov"), "export", cov, f"-instr-profile={data}"], cwd); export += d
        e = json.loads(r.stdout)["data"][0]
        fs = sorted(x["filename"] for x in e["files"] if x["summary"]["lines"]["covered"] > 0)
        fn = sorted(x["name"] for x in e["functions"] if x["count"] > 0)
        files_n.append(len(fs)); funcs_n.append(len(fn))
        SETS[f"{label} {t}"] = sorted({os.path.relpath(os.path.normpath(x), REPO) for x in fs})
        sizes.append((sum(os.path.getsize(x) for x in raws), os.path.getsize(data), len(r.stdout), len("\n".join(fs + fn))))
    n = len(tests); med = lambda i: statistics.median(s[i] for s in sizes)
    print(f"{label} tests={n} whole-run plain={whole_plain:.2f}s instrumented={whole_cov:.2f}s ratio={whole_cov / whole_plain:.2f} | "
          f"one-process-per-test plain-sum={per_plain:.2f}s instrumented-sum={per_cov:.2f}s ratio={per_cov / per_plain:.2f} | "
          f"per-test merge-sum={merge:.2f}s export-sum={export:.2f}s | median bytes profraw={med(0):.0f} profdata={med(1):.0f} export-json={med(2):.0f} names-list={med(3):.0f} | "
          f"median executed files={statistics.median(files_n)} functions={statistics.median(funcs_n)} max functions={max(funcs_n)}", flush=True)
SETS = {}
pe, ps_, pr = build(os.path.join(WORK, "t-plain"), False, ["-p", "singlefs-core", "--lib"])
ce, cs, cr = build(os.path.join(WORK, "t-cov"), True, ["-p", "singlefs-core", "--lib"])
print(f"BUILD core-lib plain={ps_:.1f}s rc={pr} instrumented={cs:.1f}s rc={cr}", flush=True)
measure("singlefs-core", pe["singlefs_core"], ce["singlefs_core"], REPO)
args = ["-p", "singlefs-harness"] + sum([["--test", t] for t in TARGETS], [])
pe, ps_, pr = build(os.path.join(WORK, "t-plain"), False, args)
ce, cs, cr = build(os.path.join(WORK, "t-cov"), True, args)
print(f"BUILD harness-targets={len(TARGETS)} plain={ps_:.1f}s rc={pr} instrumented={cs:.1f}s rc={cr}", flush=True)
for t in TARGETS:
    measure(f"singlefs-harness::{t}", pe[t], ce[t], os.path.join(REPO, "crates", "singlefs-harness"))
json.dump(SETS, open(os.path.join(WORK, "executed-files-per-test.json"), "w"), indent=0)

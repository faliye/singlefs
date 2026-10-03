#!/usr/bin/env python3
"""V2③：真 crates 副本上，抓到变异的那条 core 库测试 T，在没改过的树上与在变异体上各量一次覆盖，比两边执行到的文件与函数。
用法：mutant_path.py <副本> <库目录版本目录> <llvm 工具目录> <编译目录> <序号…>
每条：施加（照 r1 攻方 apply.py 的写法）→ 插桩编 core 的库测试 → 只跑 T → llvm-cov export → 拷回。"""
import csv, json, os, re, subprocess, sys, time
REPO, D, LLVM, TARGET = sys.argv[1:5]
SEQS = [int(x) for x in sys.argv[5:]]
APPLY = "/home/fy5090/code/singlefs/research/prompts/tooling-broad-mutation-prune-r1-opus-model/apply.py"
ENV = dict(os.environ, CARGO_TARGET_DIR=TARGET, RUSTFLAGS="-C instrument-coverage")
rows = {int(r[0]): r for r in csv.reader((l for l in open(os.path.join(D, "results.tsv")) if not l.startswith("#")), delimiter="\t")}
profiles = os.path.join(TARGET, "profiles"); os.makedirs(profiles, exist_ok=True)

def build():
    t = time.time()
    r = subprocess.run(["cargo", "test", "--offline", "-p", "singlefs-core", "--lib", "--no-run", "--message-format=json"], cwd=REPO,
                       env=dict(ENV, LLVM_PROFILE_FILE=os.path.join(profiles, "build-%p.profraw")), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    exe = [json.loads(l)["executable"] for l in r.stdout.splitlines() if l.startswith("{") and json.loads(l).get("reason") == "compiler-artifact"
           and json.loads(l).get("executable") and json.loads(l)["target"]["name"] == "singlefs_core"]
    return (exe[0] if exe else None), time.time() - t, r.returncode

def cover(exe, test):
    for f in os.listdir(profiles):
        os.remove(os.path.join(profiles, f))
    t = time.time()
    run = subprocess.run([exe, "--exact", test], cwd=REPO, env=dict(os.environ, LLVM_PROFILE_FILE=os.path.join(profiles, "t-%p.profraw")),
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, timeout=300)
    elapsed = time.time() - t
    data = os.path.join(TARGET, "t.profdata")
    subprocess.run([os.path.join(LLVM, "llvm-profdata"), "merge", "-sparse", *[os.path.join(profiles, f) for f in os.listdir(profiles)], "-o", data], check=True)
    export = json.loads(subprocess.run([os.path.join(LLVM, "llvm-cov"), "export", exe, f"-instr-profile={data}"], stdout=subprocess.PIPE, text=True, check=True).stdout)
    norm = lambda p: os.path.relpath(os.path.normpath(p), REPO)
    files = {norm(e["filename"]) for e in export["data"][0]["files"] if e["summary"]["lines"]["covered"] > 0}
    funcs = {(norm(f["filenames"][0]), f["name"]) for f in export["data"][0]["functions"] if f["count"] > 0}
    passed = "test result: ok" in run.stdout
    return files, funcs, passed, elapsed

exe, secs, rc = build()
print(f"BUILD baseline seconds={secs:.1f} rc={rc}", flush=True)
# 先在没改过的树上把每条 T 的覆盖都量完：cargo 每次把测试二进制写回同一个路径，施加过变异之后这个路径就是变异体了
baseline = {}
for seq in SEQS:
    test = rows[seq][5].split(" ", 1)[1]
    if test not in baseline:
        baseline[test] = cover(exe, test)
print(f"BASELINES {len(baseline)} distinct T, failing-on-baseline={sum(1 for v in baseline.values() if not v[2])}", flush=True)
for seq in SEQS:
    row = rows[seq]
    test = row[5].split(" ", 1)[1]
    bfiles, bfuncs, bpass, bsec = baseline[test]
    out = subprocess.run(["python3", APPLY, REPO, os.path.join(D, "list.json"), str(seq)], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        mexe, msecs, mrc = build()
        if not mexe or mrc != 0:
            print(f"SEQ {seq} build-failed rc={mrc}", flush=True); continue
        mfiles, mfuncs, mpass, msec = cover(mexe, test)
        new_files = sorted(mfiles - bfiles)
        new_funcs = sorted({f"{p}" for p, n in mfuncs - bfuncs})
        print(f"SEQ {seq} name={row[2]!r} T={test} baseline-pass={bpass} mutant-pass={mpass} build={msecs:.1f}s "
              f"baseline-files={len(bfiles)} mutant-files={len(mfiles)} files-only-in-mutant={','.join(new_files) or 'none'} "
              f"functions-only-in-mutant={len(mfuncs - bfuncs)} in-files={','.join(sorted(set(new_funcs))) or 'none'} functions-only-in-baseline={len(bfuncs - mfuncs)}", flush=True)
    finally:
        subprocess.run(["python3", APPLY, "restore", REPO], stdout=subprocess.DEVNULL)

#!/usr/bin/env python3
"""V2③（harness 那一份）：抓到变异的那条 harness 测试 T，在没改过的树上与在变异体上各量一次覆盖，比两边执行到的文件与函数。
用法：mutant_path_harness.py <副本> <库目录版本目录> <llvm 工具目录> <编译目录> <序号…>
先把每条 T 所在的测试目标在没改过的树上插桩编好、量完全部 T 的基线覆盖（cargo 每次把二进制写回同一路径，施加变异之后那个路径就是变异体了）；
再逐条：施加 → 只编 T 那个目标 → 只跑 T → llvm-cov export → 拷回。"""
import csv, json, os, subprocess, sys, time
REPO, D, LLVM, TARGET = sys.argv[1:5]
SEQS = [int(x) for x in sys.argv[5:]]
APPLY = "/home/fy5090/code/singlefs/research/prompts/tooling-broad-mutation-prune-r1-opus-model/apply.py"
ENV = dict(os.environ, CARGO_TARGET_DIR=TARGET, RUSTFLAGS="-C instrument-coverage")
rows = {int(r[0]): r for r in csv.reader((l for l in open(os.path.join(D, "results.tsv")) if not l.startswith("#")), delimiter="\t")}
profiles = os.path.join(TARGET, "profiles"); os.makedirs(profiles, exist_ok=True)
def target_and_test(seq):
    binary, test = rows[seq][5].split(" ", 1)
    return binary.split("::", 1)[1], test
def build(targets):
    t = time.time()
    cmd = ["cargo", "test", "--offline", "-p", "singlefs-harness", "--no-run", "--message-format=json"]
    for x in sorted(set(targets)):
        cmd += ["--test", x]
    r = subprocess.run(cmd, cwd=REPO, env=dict(ENV, LLVM_PROFILE_FILE=os.path.join(profiles, "build-%p.profraw")), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    exes = {}
    for l in r.stdout.splitlines():
        if l.startswith("{"):
            m = json.loads(l)
            if m.get("reason") == "compiler-artifact" and m.get("executable") and m["target"]["kind"] == ["test"]:
                exes[m["target"]["name"]] = m["executable"]
    return exes, time.time() - t, r.returncode
def cover(exe, test):
    for f in os.listdir(profiles):
        os.remove(os.path.join(profiles, f))
    t = time.time()
    run = subprocess.run([exe, "--exact", test], cwd=os.path.join(REPO, "crates", "singlefs-harness"), env=dict(os.environ, LLVM_PROFILE_FILE=os.path.join(profiles, "t-%p.profraw")),
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, timeout=900)
    elapsed = time.time() - t
    data = os.path.join(TARGET, "t.profdata")
    subprocess.run([os.path.join(LLVM, "llvm-profdata"), "merge", "-sparse", *[os.path.join(profiles, f) for f in os.listdir(profiles)], "-o", data], check=True)
    export = json.loads(subprocess.run([os.path.join(LLVM, "llvm-cov"), "export", exe, f"-instr-profile={data}"], stdout=subprocess.PIPE, text=True, check=True).stdout)
    norm = lambda p: os.path.relpath(os.path.normpath(p), REPO)
    files = {norm(e["filename"]) for e in export["data"][0]["files"] if e["summary"]["lines"]["covered"] > 0}
    funcs = {(norm(f["filenames"][0]), f["name"]) for f in export["data"][0]["functions"] if f["count"] > 0}
    return files, funcs, "test result: ok" in run.stdout, elapsed
exes, secs, rc = build([target_and_test(s)[0] for s in SEQS])
print(f"BUILD baseline targets={len(exes)} seconds={secs:.1f} rc={rc}", flush=True)
baseline = {}
for seq in SEQS:
    key = target_and_test(seq)
    if key not in baseline:
        baseline[key] = cover(exes[key[0]], key[1])
print(f"BASELINES {len(baseline)} distinct T, failing-on-baseline={sum(1 for v in baseline.values() if not v[2])}, "
      f"instrumented-seconds={sum(v[3] for v in baseline.values()):.1f}", flush=True)
for seq in SEQS:
    target, test = target_and_test(seq)
    bfiles, bfuncs, bpass, bsec = baseline[(target, test)]
    subprocess.run(["python3", APPLY, REPO, os.path.join(D, "list.json"), str(seq)], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        mexes, msecs, mrc = build([target])
        if mrc != 0 or target not in mexes:
            print(f"SEQ {seq} build-failed rc={mrc}", flush=True); continue
        mfiles, mfuncs, mpass, msec = cover(mexes[target], test)
        newf = sorted(mfiles - bfiles)
        print(f"SEQ {seq} name={rows[seq][2]!r} T={target} {test} baseline-pass={bpass} mutant-pass={mpass} build={msecs:.1f}s run={msec:.1f}s "
              f"baseline-files={len(bfiles)} mutant-files={len(mfiles)} files-only-in-mutant={','.join(newf) or 'none'} "
              f"functions-only-in-mutant={len(mfuncs - bfuncs)} in-files={','.join(sorted({p for p, n in mfuncs - bfuncs})) or 'none'} "
              f"functions-only-in-baseline={len(bfuncs - mfuncs)}", flush=True)
    finally:
        subprocess.run(["python3", APPLY, "restore", REPO], stdout=subprocess.DEVNULL)

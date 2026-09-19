#!/usr/bin/env python3
"""攻方腿（Opus）m2-supp3-item1-code-r1：在装了 opus_probe.rs 的仓副本上施加一条变异（或不施加：BASE），跑逐种子探针，存原样输出。
用法：python3 run-probe.py --slot DIR --target DIR --out FILE --id A2 [--env K=V ...] [--table mutants.tsv]
探针的环境变量：OPUS_FIRST、OPUS_SEEDS、OPUS_OPS、OPUS_THREADS；副本若打过 always-check 补丁，OPUS_ALWAYS_CHECK=1 让每一步都跑 checker。
"""
import argparse, os, subprocess, time

def load(table):
    rows = {}
    for line in open(table, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        ident, name, path, old, new = line.split("\t")
        rows[ident] = (name, path, old.replace("\\n", "\n"), new.replace("\\n", "\n"))
    return rows

parser = argparse.ArgumentParser()
parser.add_argument("--slot", required=True)
parser.add_argument("--target", required=True)
parser.add_argument("--out", required=True)
parser.add_argument("--id", required=True)
parser.add_argument("--env", action="append", default=[])
parser.add_argument("--table", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "mutants.tsv"))
args = parser.parse_args()
pristine = full = None
if args.id != "BASE":
    name, path, old, new = load(args.table)[args.id]
    full = os.path.join(args.slot, path)
    pristine = open(full, encoding="utf-8").read()
    assert pristine.count(old) == 1, (args.id, pristine.count(old))
    open(full, "w", encoding="utf-8").write(pristine.replace(old, new))
try:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = args.target
    for item in args.env:
        key, value = item.split("=", 1)
        env[key] = value
    started = time.time()
    run = subprocess.run(["nice", "-n", "19", "cargo", "test", "--offline", "--release", "-p", "singlefs-harness", "--test", "opus_probe",
                          "--", "--ignored", "--exact", "opus_probe_per_seed_endings", "--nocapture"],
                         cwd=args.slot, env=env, capture_output=True, text=True)
    output = run.stdout + run.stderr
    open(args.out, "w", encoding="utf-8").write(f"# id {args.id} env {args.env} exit {run.returncode} {time.time() - started:.1f}s\n" + output)
    summary = [l for l in output.splitlines() if l.startswith("OPUS-SUMMARY")]
    print(args.id, args.env, run.returncode, summary, flush=True)
finally:
    if pristine is not None:
        open(full, "w", encoding="utf-8").write(pristine)

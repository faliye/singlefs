#!/usr/bin/env python3
"""攻方腿（Opus）m2-supp3-item1-code-r2：在仓副本（槽目录）上逐条施加 mutants.tsv 的变异，编 opus_r2_gap 探针，按几组规模跑，记原样输出，跑完还原。
用法：python3 run-mutants.py --slot DIR --logs DIR --ids Y1a,A1 [--scales broad:0:96:30,reuse:0:48:30] [--table mutants.tsv]
  规模：比重:第一个种子:种子数:步数。BASE 表示不施加变异。
"""
import argparse, os, subprocess, re, time, json

def load(table):
    rows = {}
    for line in open(table, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        ident, name, path, old, new = line.split("\t")
        rows[ident] = (name, path, old.replace("\\n", "\n"), new.replace("\\n", "\n"))
    return rows

def build(slot):
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(slot, "target"))
    run = subprocess.run(["nice", "-n", "19", "cargo", "test", "--offline", "--release", "-p", "singlefs-harness",
                          "--test", "opus_r2_gap", "--no-run", "--message-format=json"], cwd=slot, env=env,
                         capture_output=True, text=True)
    exe = None
    for line in run.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get("reason") == "compiler-artifact" and message.get("executable") and "opus_r2_gap" in message["executable"]:
            exe = message["executable"]
    return run.returncode, exe, run.stderr

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--slot", required=True)
    parser.add_argument("--logs", required=True)
    parser.add_argument("--ids", required=True)
    parser.add_argument("--scales", default="broad:0:96:30,reuse:0:48:30")
    parser.add_argument("--threads", default="16")
    parser.add_argument("--table", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "mutants.tsv"))
    args = parser.parse_args()
    rows = load(args.table)
    os.makedirs(args.logs, exist_ok=True)
    summary = open(os.path.join(args.logs, "summary.tsv"), "a", encoding="utf-8")
    for ident in args.ids.split(","):
        full = pristine = None
        if ident != "BASE":
            name, path, old, new = rows[ident]
            full = os.path.join(args.slot, path)
            pristine = open(full, encoding="utf-8").read()
            if pristine.count(old) != 1:
                summary.write(f"{ident}\t-\t锚点命中 {pristine.count(old)} 次\n"); continue
            open(full, "w", encoding="utf-8").write(pristine.replace(old, new))
        try:
            code, exe, stderr = build(args.slot)
            if code != 0 or exe is None:
                open(os.path.join(args.logs, f"{ident}-build.log"), "w").write(stderr)
                summary.write(f"{ident}\tbuild\t{code}\n"); summary.flush(); continue
            for scale in args.scales.split(","):
                weights, first, seeds, ops = scale.split(":")
                env = dict(os.environ, OPUS_WEIGHTS=weights, OPUS_FIRST=first, OPUS_SEEDS=seeds, OPUS_OPS=ops, OPUS_THREADS=args.threads)
                started = time.time()
                run = subprocess.run(["nice", "-n", "19", exe, "--ignored", "--nocapture", "opus_r2_gap_probe"],
                                     env=env, capture_output=True, text=True)
                output = run.stdout + run.stderr
                tag = f"{weights}-{first}-{seeds}x{ops}"
                open(os.path.join(args.logs, f"{ident}-{tag}.log"), "w", encoding="utf-8").write(output)
                summ = [l for l in output.splitlines() if l.startswith("SUMMARY")]
                summary.write(f"{ident}\t{tag}\t{run.returncode}\t{summ[0] if summ else '-'}\t{time.time()-started:.1f}s\n")
                summary.flush()
                print(ident, tag, run.returncode, summ[0] if summ else "-", flush=True)
        finally:
            if full is not None:
                open(full, "w", encoding="utf-8").write(pristine)
main()

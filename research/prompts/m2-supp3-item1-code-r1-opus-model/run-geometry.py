#!/usr/bin/env python3
"""攻方腿（Opus）m2-supp3-item1-code-r1 · Z4：在打过 geometry-env-patch.py 的仓副本上，对（基线 / 一条变异）× 若干快档几何跑真快档（判定一字不动），
记退出码、「历史 … 段」那一行、新发现行、以及让测试失败的那一句 panic 信息。
用法：python3 run-geometry.py --slot DIR --target DIR --out FILE --ids BASE,R121 --points "seeds=96,ops=30;seeds=96,ops=20;..." [--table rows-41-121.tsv]
几何点的键：first、seeds、ops、wf（OPUS_WEIGHTS_WITH_FILE，用 / 分隔七个数）、wc（OPUS_WEIGHTS_CLOSED，三个数）。
"""
import argparse, os, re, subprocess, time
FAST = "random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation"

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
parser.add_argument("--ids", required=True)
parser.add_argument("--points", required=True)
parser.add_argument("--table", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "rows-41-121.tsv"))
args = parser.parse_args()
rows = load(args.table)
out = open(args.out, "a", encoding="utf-8")
for ident in args.ids.split(","):
    pristine = full = None
    if ident != "BASE":
        name, path, old, new = rows[ident]
        full = os.path.join(args.slot, path)
        pristine = open(full, encoding="utf-8").read()
        assert pristine.count(old) == 1, (ident, pristine.count(old))
        open(full, "w", encoding="utf-8").write(pristine.replace(old, new))
    try:
        for point in args.points.split(";"):
            env = dict(os.environ)
            env["CARGO_TARGET_DIR"] = args.target
            for item in point.split(","):
                key, value = item.split("=")
                name = {"first": "OPUS_FAST_FIRST", "seeds": "OPUS_FAST_SEEDS", "ops": "OPUS_FAST_OPS",
                        "wf": "OPUS_WEIGHTS_WITH_FILE", "wc": "OPUS_WEIGHTS_CLOSED"}[key]
                env[name] = value.replace("/", ",")
            started = time.time()
            run = subprocess.run(["nice", "-n", "19", "cargo", "test", "--offline", "--release", "-p", "singlefs-harness",
                                  "--test", "second_transaction_supplement_three_random_history", "--", "--exact", FAST],
                                 cwd=args.slot, env=env, capture_output=True, text=True)
            output = run.stdout + run.stderr
            lines = output.splitlines()
            head = next((l for l in lines if l.startswith("历史 ")), "")
            findings = [l for l in lines if l.startswith("新发现 ")]
            panic_index = next((i for i, l in enumerate(lines) if "panicked at" in l), None)
            reason = lines[panic_index + 1] if panic_index is not None and panic_index + 1 < len(lines) else ""
            if reason.startswith("「已知红」清单外的失败"):
                reason = "「已知红」清单外的失败（新发现）"
            removed = next((l.strip() for l in lines if "已释放的记录被删" in l), "")
            out.write(f"{ident}\t{point}\texit {run.returncode}\t{head}\t{' ; '.join(findings)}\t失败原因：{reason}\t{removed}\t{time.time() - started:.1f}s\n")
            out.flush()
            print(ident, point, run.returncode, head, len(findings), reason, flush=True)
    finally:
        if pristine is not None:
            open(full, "w", encoding="utf-8").write(pristine)

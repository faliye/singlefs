#!/usr/bin/env python3
"""攻方腿（Opus）m2-supp3-item1-code-r1：在仓副本（槽目录）上逐条施加 mutants.tsv 里的变异，跑随机历史快档与大档，记原样输出。

用法（副本上跑，主工作区不碰）：
  python3 run-mutants.py --slot DIR --target DIR --logs DIR --ids A1,A2 [--large 1000x60,3000x40] [--profile release|dev] [--table mutants.tsv]
  --slot   仓副本（至少有 Cargo.toml、Cargo.lock、crates/），每条变异施加前核原文恰好命中一次，跑完还原原文件内容。
  --ids    逗号分隔；BASE 表示不施加变异（基线）。
  --large  大档规模列表：种子数x步数，从种子 0 起、16 线程；空串表示不跑大档。
每条变异写 logs/<id>-fast.log、logs/<id>-large-<规模>.log，并往 logs/summary.tsv 追加一行：编号、档、退出码、「历史 … 段」那一行、已知红各条段数、新发现签名。
"""
import argparse, os, re, subprocess, sys, time

FAST = "random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation"
LARGE = "random_histories_large_tier_seeds_and_length_from_the_environment"
TEST = "second_transaction_supplement_three_random_history"

def load(table):
    rows = {}
    for line in open(table, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        assert len(fields) == 5, line[:80]
        ident, name, path, old, new = fields
        rows[ident] = (name, path, old.replace("\\n", "\n"), new.replace("\\n", "\n"))
    return rows

def digest(output):
    head = [l for l in output.splitlines() if l.startswith("历史 ")]
    forms = [l.split("：")[0] + "：" + l.split("：")[1].split("；")[0] for l in output.splitlines() if l.startswith("已知红第 ")]
    findings = [l for l in output.splitlines() if l.startswith("新发现 ")]
    return " | ".join(head[:1]), " ; ".join(forms), " ; ".join(findings)

def cargo_test(slot, target, profile, test_args, env_extra):
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = target
    env.update(env_extra)
    command = ["nice", "-n", "19", "cargo", "test", "--offline", "--profile", "release" if profile == "release" else "dev",
               "-p", "singlefs-harness", "--test", TEST, "--"] + test_args
    started = time.time()
    run = subprocess.run(command, cwd=slot, env=env, capture_output=True, text=True)
    return run.returncode, run.stdout + run.stderr, time.time() - started

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--slot", required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--logs", required=True)
    parser.add_argument("--ids", required=True)
    parser.add_argument("--large", default="")
    parser.add_argument("--profile", default="release")
    parser.add_argument("--table", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "mutants.tsv"))
    args = parser.parse_args()
    rows = load(args.table)
    os.makedirs(args.logs, exist_ok=True)
    summary = open(os.path.join(args.logs, "summary.tsv"), "a", encoding="utf-8")
    for ident in args.ids.split(","):
        pristine = None
        full = None
        if ident != "BASE":
            name, path, old, new = rows[ident]
            full = os.path.join(args.slot, path)
            pristine = open(full, encoding="utf-8").read()
            count = pristine.count(old)
            if count != 1:
                print(f"{ident}: 原文命中 {count} 次，跳过", flush=True)
                summary.write(f"{ident}\t-\t锚点命中 {count} 次\t\t\t\n")
                continue
            open(full, "w", encoding="utf-8").write(pristine.replace(old, new))
        try:
            code, output, seconds = cargo_test(args.slot, args.target, args.profile, ["--exact", FAST], {})
            open(os.path.join(args.logs, f"{ident}-fast.log"), "w", encoding="utf-8").write(output)
            head, forms, findings = digest(output)
            summary.write(f"{ident}\tfast\t{code}\t{head}\t{forms}\t{findings}\t{seconds:.1f}s\n")
            summary.flush()
            print(f"{ident} fast exit {code} {head} {forms} {findings}", flush=True)
            for scale in [s for s in args.large.split(",") if s]:
                seeds, operations = scale.split("x")
                env = {"SINGLEFS_RANDOM_HISTORY_SEEDS": seeds, "SINGLEFS_RANDOM_HISTORY_OPERATIONS": operations,
                       "SINGLEFS_RANDOM_HISTORY_FIRST_SEED": "0", "SINGLEFS_RANDOM_HISTORY_THREADS": "16"}
                code, output, seconds = cargo_test(args.slot, args.target, args.profile, ["--exact", LARGE, "--ignored"], env)
                open(os.path.join(args.logs, f"{ident}-large-{scale}.log"), "w", encoding="utf-8").write(output)
                head, forms, findings = digest(output)
                summary.write(f"{ident}\tlarge {scale}\t{code}\t{head}\t{forms}\t{findings}\t{seconds:.1f}s\n")
                summary.flush()
                print(f"{ident} large {scale} exit {code} {head} {forms} {findings}", flush=True)
        finally:
            if pristine is not None:
                open(full, "w", encoding="utf-8").write(pristine)

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""第 10z 批第五块代码轮第一轮攻方：实现员没证红的 harness 档变异行逐行在拷贝上证（照 crates/mutations.tsv 那一行的原文、替换文、
cargo test 参数与必须红的测试名）。用法（经内存包装）：python3 rows_10ze.py <冻结副本的拷贝> <编译目录> <行号…>"""
import os, subprocess, sys

def main():
    copy, target, rows = sys.argv[1], sys.argv[2], [int(r) for r in sys.argv[3:]]
    lines = open(os.path.join(copy, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")
    for row in rows:
        name, file, old, new, args, must_red = lines[row - 1].split("\t")
        old, new = old.replace("\\n", "\n"), new.replace("\\n", "\n")
        path = os.path.join(copy, file)
        original = open(path, encoding="utf-8").read()
        hits = original.count(old)
        if hits != 1:
            print(f"ROW {row} anchor_hits={hits} skipped: {name}")
            continue
        open(path, "w", encoding="utf-8").write(original.replace(old, new, 1))
        try:
            env = dict(os.environ, CARGO_TARGET_DIR=target)
            p = subprocess.run(["nice", "-n", "19", "cargo", "test", "--offline"] + args.split(), cwd=copy, env=env,
                               capture_output=True, text=True)
        finally:
            open(path, "w", encoding="utf-8").write(original)
        out = p.stdout + p.stderr
        red = f"{must_red} ... FAILED" in out
        summary = [l for l in out.splitlines() if l.startswith("test result:")]
        print(f"ROW {row} rc={p.returncode} named_test_red={red} {summary}: {name}")

if __name__ == "__main__":
    main()

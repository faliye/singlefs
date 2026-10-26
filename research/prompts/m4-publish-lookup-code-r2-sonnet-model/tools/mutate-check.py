#!/usr/bin/env python3
"""在一棵 crates 副本上逐行套变异表的一行（名字 TAB 文件 TAB 原文 TAB 替换 TAB cargo 参数 TAB 点名的测试，\\n 是换行），
跑那一行的 cargo 命令，打出 name=… exit=… 与第一条 panic 行；套完还原。
用法：mutate-check.py --tree <含 crates/ 的副本> --tsv <变异表> --target-dir <编译目录> --cap <内存上限> --threads <N> --repo <仓根> [--only <行序号,…>] [--skip-ignored-rows]
副本与编译目录是调用方给的草稿路径，脚本不碰别处。"""
import argparse, os, subprocess, sys, re

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tree", required=True)
    ap.add_argument("--tsv", required=True)
    ap.add_argument("--target-dir", required=True)
    ap.add_argument("--cap", required=True)
    ap.add_argument("--threads", required=True)
    ap.add_argument("--repo", required=True)
    ap.add_argument("--only", default="")
    ap.add_argument("--log-dir", default="")
    args = ap.parse_args()
    only = {int(x) for x in args.only.split(",") if x}
    rows = [l.rstrip("\n").split("\t") for l in open(args.tsv, encoding="utf-8") if l.strip()]
    for index, row in enumerate(rows, 1):
        if only and index not in only:
            continue
        name, path, original, replacement, cargo_args, _test = row
        original = original.replace("\\n", "\n")
        replacement = replacement.replace("\\n", "\n")
        file_path = os.path.join(args.tree, path)
        text = open(file_path, encoding="utf-8").read()
        hits = text.count(original)
        if hits != 1:
            print(f"name={name} anchor_hits={hits} (要恰好 1 处)")
            continue
        open(file_path, "w", encoding="utf-8").write(text.replace(original, replacement, 1))
        try:
            scope = ("SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的一个测试目标上的变异点名用例; 范围=1 个测试目标; "
                     "为什么=核第二批补丁的变异行点名的用例在变异下红、在原样下绿'")
            test_words = " ".join(cargo_args.split())
            command = (f"{scope} nice -n 19 bash {args.repo}/scripts/hosts/capped.sh {args.threads} "
                       f"bash {args.repo}/scripts/hosts/run-with-memory-cap.sh {args.cap} "
                       f"cargo test --offline {test_words}")
            env = dict(os.environ, CARGO_TARGET_DIR=args.target_dir)
            done = subprocess.run(["bash", "-c", command], cwd=args.tree, env=env,
                                  capture_output=True, text=True, timeout=1500)
            output = done.stdout + done.stderr
            if args.log_dir:
                os.makedirs(args.log_dir, exist_ok=True)
                with open(os.path.join(args.log_dir, f"row{index}.log"), "w", encoding="utf-8") as log:
                    log.write(output)
            panic = next((l.strip() for l in output.splitlines() if "panicked at" in l or "assertion" in l), "")
            result = next((l.strip() for l in output.splitlines() if l.startswith("test result")), "")
            print(f"name={name} exit={done.returncode} {result}")
            if panic:
                print(f"  first_panic_line={panic[:300]}")
            for l in output.splitlines():
                if re.match(r"^\s+(left|right):", l) or l.startswith("txg") or "整理读这一版" in l:
                    print("  " + l.strip()[:300])
                    break
        finally:
            open(file_path, "w", encoding="utf-8").write(text)
    return 0

sys.exit(main())

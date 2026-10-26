#!/usr/bin/env python3
"""mutate_run.py：在 --work 的仓副本里逐行施加变异表的改法，跑点名的测试，贴结果，还原。

用法：python3 -I mutate_run.py --repo <仓根> --pristine <未改副本根> --work <变异副本根> --target <编译目录> --out <输出目录> --tsv <变异表> --rows 1,2,3
变异表六段（制表符分隔）：名字、文件、原文、替换文、cargo test 参数、必须红的测试名；原文、替换文里的 \\n 是换行。
每行：未改副本里的该文件拷回变异副本 -> 原文必须恰好命中一次 -> 换成替换文 -> 经 capped.sh 与 run-with-memory-cap.sh 跑 cargo test -> 记 rc、test result 行、
panicked 行及其后至多 8 行 -> 还原。最后一行打 name=mutant ... 摘要。
"""
import argparse, os, shutil, subprocess, sys, pathlib

ap = argparse.ArgumentParser()
ap.add_argument("--repo", required=True); ap.add_argument("--pristine", required=True)
ap.add_argument("--work", required=True); ap.add_argument("--target", required=True)
ap.add_argument("--out", required=True); ap.add_argument("--tsv", required=True)
ap.add_argument("--rows", required=True); ap.add_argument("--threads", default="31")
ap.add_argument("--tag", default=""); ap.add_argument("--cap", default="8G"); ap.add_argument("--direct", action="store_true")
a = ap.parse_args()
out = pathlib.Path(a.out); out.mkdir(parents=True, exist_ok=True)
rows = [l.rstrip("\n").split("\t") for l in open(a.tsv, encoding="utf-8") if l.strip()]
for number in [int(x) for x in a.rows.split(",")]:
    name, rel, old, new, cargo_args, test = rows[number - 1]
    old = old.replace("\\n", "\n"); new = new.replace("\\n", "\n")
    src = pathlib.Path(a.pristine) / rel; dst = pathlib.Path(a.work) / rel
    shutil.copyfile(src, dst)
    text = dst.read_text(encoding="utf-8")
    hits = text.count(old)
    if hits != 1:
        print(f"name=mutant row={number} anchor_hits={hits} (not 1) skipped: {name}"); continue
    dst.write_text(text.replace(old, new), encoding="utf-8")
    env = dict(os.environ, CARGO_TARGET_DIR=a.target,
               SINGLEFS_RUN_SCOPE="种类=target; 跑什么=单条变异点名的测试目标; 范围=1 个测试目标; 为什么=第二轮正推腿逐条核变异改的是哪一行、点名的用例红在哪条断言")
    cmd = ["nice", "-n", "19", "bash", f"{a.repo}/scripts/hosts/capped.sh", a.threads, "bash",
           f"{a.repo}/scripts/hosts/run-with-memory-cap.sh", a.cap, "cargo", "test", "--offline"] + cargo_args.split()
    if a.direct:  # 第二台上外面已经套了内存包装
        cmd = ["nice", "-n", "19", "bash", f"{a.repo}/scripts/hosts/capped.sh", a.threads, "cargo", "test", "--offline"] + cargo_args.split()
    proc = subprocess.run(cmd, cwd=a.work, env=env, capture_output=True, text=True)
    log = (proc.stdout or "") + (proc.stderr or "")
    (out / f"{a.tag}row{number:02d}.log").write_text(log, encoding="utf-8")
    shutil.copyfile(src, dst)  # 还原
    lines = log.splitlines()
    result = next((l for l in lines if l.startswith("test result")), "no-test-result")
    result = result.split("; finished")[0]
    print(f"name=mutant row={number} rc={proc.returncode} {result} :: {name}")
    shown = 0
    for i, l in enumerate(lines):
        if "panicked at" in l and shown < 2:
            msg = " | ".join(x.strip() for x in lines[i + 1:i + 5] if x.strip())
            print(f"name=mutant_panic row={number} {l.strip()} => {msg[:700]}")
            shown += 1
    if proc.returncode != 0 and shown == 0:
        err = [l for l in lines if l.startswith("error")][:3]
        print(f"name=mutant_nopanic row={number} {err}")

#!/usr/bin/env python3
"""把变异表里的行逐条打在一棵源码副本上、跑点名的测试、记红不红。只读被判的树，只写 --tree（自己的副本）与 --out。
用法：
  run_mutants.py --repo <仓根> --tree <源码副本> --target <编译目录> --tsv <变异表> --out <结果文件> [--prefix 七·6 第三轮] [--from N] [--to M] [--baseline]
变异表格式同 crates/mutations.tsv：变异名 TAB 文件 TAB 原文 TAB 替换文 TAB cargo test 的参数 TAB 必须红的测试名（\\n 表示换行）。
每条：原文在文件里必须恰好命中一次；打上、跑（经 capped.sh 31 与 run-with-memory-cap.sh 16G、two-logs.sh）、还原。
结果行：idx<TAB>结局<TAB>名<TAB>测试结果行<TAB>第一条断言信息（截 300 字）。结局：RED（编译过、测试红）、GREEN（编译过、测试绿）、
APPLY_FAILED（原文命中次数不是 1）、COMPILE_ERROR（没编过）。"""
import argparse, os, re, subprocess, sys, pathlib, shlex

ap = argparse.ArgumentParser()
ap.add_argument("--repo", required=True)
ap.add_argument("--tree", required=True)
ap.add_argument("--target", required=True)
ap.add_argument("--tsv", required=True)
ap.add_argument("--out", required=True)
ap.add_argument("--prefix", default="")
ap.add_argument("--from", dest="lo", type=int, default=1)
ap.add_argument("--to", dest="hi", type=int, default=10**9)
ap.add_argument("--baseline", action="store_true")
a = ap.parse_args()

repo = pathlib.Path(a.repo)
tree = pathlib.Path(a.tree)
rows = [l.rstrip("\n").split("\t") for l in pathlib.Path(a.tsv).read_text().splitlines()
        if l and not l.startswith("#") and l.startswith(a.prefix)]
out = open(a.out, "a")


def run_test(args_text: str, label: str):
    log = pathlib.Path(a.out + f".{label}.log")
    cmd = ["bash", str(repo / "scripts/hosts/two-logs.sh"), "--quiet", str(log.with_suffix("")), "--",
           "bash", str(repo / "scripts/hosts/capped.sh"), "31",
           "bash", str(repo / "scripts/hosts/run-with-memory-cap.sh"), "16G",
           "cargo", "test", "--offline"] + shlex.split(args_text)
    env = dict(os.environ, CARGO_TARGET_DIR=a.target,
               SINGLEFS_RUN_SCOPE="种类=target; 跑什么=singlefs-harness 的测试目标 publish_lookup_by_identity 里点名的一条用例; "
                                  "范围=1 个测试目标; 为什么=把变异表里的一行打在自己的源码副本上看点名用例红不红，每次只点名一条")
    subprocess.run(cmd, cwd=tree, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    text = pathlib.Path(str(log.with_suffix("")) + ".log").read_text(errors="replace")
    return text


def summarize(text: str):
    if re.search(r"^error(\[E\d+\])?:", text, re.M) and "could not compile" in text and "test result" not in text:
        first = next((l for l in text.splitlines() if l.startswith("error")), "")
        return "COMPILE_ERROR", "", first[:300]
    m = re.findall(r"^test result: .*$", text, re.M)
    result = m[-1] if m else ""
    outcome = "GREEN" if result.startswith("test result: ok") else "RED" if result.startswith("test result: FAILED") else "NO_RESULT"
    panic = next((l for l in text.splitlines() if "panicked at" in l), "")
    detail_lines = text.splitlines()
    idx = next((i for i, l in enumerate(detail_lines) if "panicked at" in l), None)
    detail = (detail_lines[idx + 1] if idx is not None and idx + 1 < len(detail_lines) else panic)
    return outcome, result, detail[:300]


if a.baseline:
    text = run_test("-p singlefs-harness --test publish_lookup_by_identity -- --exact " + rows[0][5], "baseline")
    outcome, result, detail = summarize(text)
    out.write(f"0\t{outcome}\tbaseline\t{result}\t{detail}\n"); out.flush()

for idx, row in enumerate(rows, 1):
    if idx < a.lo or idx > a.hi:
        continue
    name, file, orig, repl, args_text, test = row
    path = tree / file
    original_text = path.read_text()
    o = orig.replace("\\n", "\n")
    n = repl.replace("\\n", "\n")
    hits = original_text.count(o)
    if hits != 1:
        out.write(f"{idx}\tAPPLY_FAILED\t{name}\thits={hits}\t\n"); out.flush()
        continue
    path.write_text(original_text.replace(o, n, 1))
    try:
        text = run_test(args_text, f"m{idx}")
    finally:
        path.write_text(original_text)
    outcome, result, detail = summarize(text)
    out.write(f"{idx}\t{outcome}\t{name}\t{result}\t{detail}\n"); out.flush()

#!/usr/bin/env python3
"""U3：最近 N 个动了 core / checker src 的提交，按 C1（读法 R2：词法单元比整个 fn 项；函数外部分变了算整个文件；新加的函数算）
数「这次改到的」变异（清单用库目录这一版的，按 HEAD 上的函数归属对过去），再按 C2 挑测试的耗时估这次提交要多久。
估法（推的）：每条 = 编译（库里这一版 caught 的编译秒数平均 20.4）+ 测试墙钟。测试墙钟 = 第一段整遍墙钟 278.5 秒 ×（挑出来的测试按基线日志的
耗时加起来 ÷ 全部测试的耗时和 7691.1）；零覆盖而落在 fn 外（常量初始化式）的跑全部（278.5 秒）；零覆盖而在 fn 里的记没红、不跑。
这是上界：遇第一个失败即停的那批（这一版 303 条里 242 条抓到，抓到的测试秒数中位 1.0）实际短得多。
用法：u3_commit_cost.py <模型目录> <HEAD 导出的副本根> <库目录这一版> <selection.tsv> <N> <主仓根（取 git 历史）>
"""
import json
import os
import subprocess
import sys

model, root, library, selection_path, count, repository = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]), sys.argv[6]
sys.argv = sys.argv[:1] + ["-", "-"]
sys.path.insert(0, model)
import run  # noqa: E402

BUILD, FULL_WALL, FULL_SUM = 20.4, 278.5, 7691.1
selected = {}
for line in open(selection_path):
    p = line.rstrip("\n").split("\t")
    if p[0].isdigit():
        selected[int(p[0])] = (int(p[4]), float(p[5]))
mutants = json.load(open(os.path.join(library, "list.json")))
head_text = {}
head_functions = {}


def cached_functions(path, text):
    if path not in head_functions:
        head_functions[path] = run.functions(text)
    return head_functions[path]


def key_of(mutant):
    text = head_text.setdefault(mutant["file"], open(os.path.join(root, mutant["file"])).read())
    lines = text.split("\n")
    offset = sum(len(l) + 1 for l in lines[: mutant["start"][0] - 1]) + mutant["start"][1] - 1
    best = None
    for key, item, body_start, body_end in cached_functions(mutant["file"], text):
        if item <= offset < body_end and (best is None or item > best[1]):
            best = (key, item)
    return best[0] if best else None


keys = {m["sequence"]: key_of(m) for m in mutants}


def cost(sequence):
    n, seconds = selected[sequence]
    if n == 0:
        return BUILD + (FULL_WALL if keys[sequence] is None else 0.0)
    return BUILD + FULL_WALL * seconds / FULL_SUM


def show(commit, path):
    out = subprocess.run(["git", "show", f"{commit}:{path}"], cwd=repository, capture_output=True, text=True)
    return out.stdout if out.returncode == 0 else ""


src = ["crates/singlefs-core/src", "crates/singlefs-checker/src"]
commits = subprocess.run(["git", "log", f"-{count}", "--format=%h", "--", *src], cwd=repository, capture_output=True, text=True, check=True).stdout.split()
totals = []
for commit in commits:
    files = [f for f in subprocess.run(["git", "show", "--name-only", "--format=", commit, "--", *src], cwd=repository, capture_output=True, text=True, check=True).stdout.split() if f.endswith(".rs")]
    whole_files, changed_keys = set(), set()
    for f in files:
        old, new = show(commit + "^", f), show(commit, f)
        t_old = {k: (i, e) for k, i, _, e in run.functions(old)}
        t_new = {k: (i, e) for k, i, _, e in run.functions(new)}
        if run.token_normal(run.outside(old, list(t_old.values()))) != run.token_normal(run.outside(new, list(t_new.values()))):
            whole_files.add(f)
        for k in set(t_old) | set(t_new):
            if k not in t_old or k not in t_new or run.token_normal(old[t_old[k][0]:t_old[k][1]]) != run.token_normal(new[t_new[k][0]:t_new[k][1]]):
                changed_keys.add((f, k))
    hit = [m["sequence"] for m in mutants if m["file"] in whole_files or (m["file"], keys[m["sequence"]]) in changed_keys]
    hours = sum(cost(s) for s in hit) / 3600
    totals.append((len(hit), hours))
    print(f"U3-COMMIT {commit} files={len(files)} whole-file-rerun={len(whole_files)} changed-functions={len(changed_keys)} mutants={len(hit)} hours-upper={hours:.1f}")
all_hours = sum(cost(m["sequence"]) for m in mutants) / 3600
by_mutants = sorted(t[0] for t in totals)
by_hours = sorted(t[1] for t in totals)
print(f"U3-TOTALS commits={len(totals)} mutants-median={by_mutants[len(by_mutants) // 2]} hours-median={by_hours[len(by_hours) // 2]:.1f} "
      f"hours-min={by_hours[0]:.1f} hours-max={by_hours[-1]:.1f} milestone-full-all-{len(mutants)}-mutants-hours-upper={all_hours:.1f} "
      f"same-without-c2-selection-hours={len(mutants) * (BUILD + FULL_WALL) / 3600:.1f}")

#!/usr/bin/env python3
"""广谱变异剪枝与复用：每次提交耗时的算术模型（只读 list.json / results.tsv / baseline.tsv 与 git log，不起 cargo、不起驱动）。
用法：python3 cost-model.py [版本目录]；默认是 19c45e02… 那一版。输出全部是从这些文件算出来的数。"""
import collections
import csv
import json
import os
import statistics as st
import subprocess
import sys

DEFAULT = "~/.local/share/singlefs/broad-mutation/versions/19c45e021b5638f68eac7c72dcc317a4574bdaa2b9f33a4a8855284827265cd4"
V = os.path.expanduser(sys.argv[1] if len(sys.argv) > 1 else DEFAULT)
L = json.load(open(V + "/list.json"))
N = len(L)
cnt = collections.Counter(x["file"] for x in L)
core_n = sum(v for f, v in cnt.items() if "singlefs-core" in f)
lines = open(V + "/results.tsv").read().splitlines()
hdr = lines[0].lstrip("# ").split("\t")
rows = [dict(zip(hdr, l.split("\t"))) for l in lines[1:]]
last = {}
for r in rows:
    last[r["sequence"]] = r
print(f"清单 {N} 条、{len(cnt)} 个文件；结果表 {len(rows)} 行、{len(last)} 个不同序号（重复的序号：{[k for k, v in collections.Counter(r['sequence'] for r in rows).items() if v > 1]}）")
print("结果表覆盖的文件：", dict(collections.Counter(next(x for x in L if str(x['sequence']) == s)['file'].split('/')[-1] for s in last)))
by = collections.defaultdict(list)
for r in last.values():
    by[r["outcome"]].append(float(r["build_seconds"]) + float(r["test_seconds"]))
n = len(last)
print("结局\t条数\t占比\t均值秒\t中位秒\t合计秒")
for o, v in by.items():
    print(f"{o}\t{len(v)}\t{len(v)/n:.3f}\t{st.mean(v):.1f}\t{st.median(v):.1f}\t{sum(v):.0f}")
total = sum(sum(v) for v in by.values())
mean_all = total / n
steady = by["caught"] + by["unviable"]
mean_steady = st.mean(steady)
mean_unviable = st.mean(by["unviable"])
frac_unviable = len(by["unviable"]) / n
slow_time = sum(by["stage-one-only"]) + sum(by["timeout"])
print(f"全混合每条均值 {mean_all:.1f} 秒；只含抓到与无效（可提交状态）每条均值 {mean_steady:.1f} 秒；两个慢档占采样时间 {slow_time/total:.3f}")
print(f"整张清单照这个混合外推：全混合 {mean_all*N/86400:.1f} 天；慢档条数外推 {(len(by['stage-one-only'])+len(by['timeout']))/n*N:.0f} 条、{slow_time/n*N/3600:.0f} 小时")
caught = [r for r in last.values() if r["outcome"] == "caught"]
bs = [float(r["build_seconds"]) for r in caught]
ts = [float(r["test_seconds"]) for r in caught]
print(f"抓到 {len(caught)} 条：编译均值 {st.mean(bs):.1f} 中位 {st.median(bs):.1f}；测试均值 {st.mean(ts):.1f} 中位 {st.median(ts):.1f}；测试超过 10 秒的 {sum(x > 10 for x in ts)} 条占测试总秒的 {sum(x for x in ts if x > 10)/sum(ts):.3f}")
kinds = {}
for k in ("singlefs-core", "singlefs-harness"):
    cc = [r for r in caught if r["first_failing_test"].startswith(k)]
    kinds[k] = (len(cc), st.mean(float(r["build_seconds"]) for r in cc), st.mean(float(r["test_seconds"]) for r in cc), st.median(float(r["test_seconds"]) for r in cc))
    print(f"  第一条失败的测试在 {k} 里：{len(cc)} 条，编译均值 {kinds[k][1]:.1f}，测试均值 {kinds[k][2]:.1f}，测试中位 {kinds[k][3]:.1f}")
mean_t_first = (kinds["singlefs-core"][0] * (kinds["singlefs-core"][1] + kinds["singlefs-core"][3]) + kinds["singlefs-harness"][0] * (kinds["singlefs-harness"][1] + 47.7)) / (kinds["singlefs-core"][0] + kinds["singlefs-harness"][0])
print(f"「先只跑上次抓到它的那条测试」的粗估：core 那类 编译均值+测试中位，harness 那类 编译均值+47.7（harness 类测试秒的中位数，当作该测试自身时长的替身）→ 每条约 {mean_t_first:.1f} 秒")
base = {}
for l in open(V + "/baseline.tsv").read().splitlines()[1:]:
    p = l.split("\t")
    base[p[0]] = (p[1], float(p[2]))
c0 = sum(s for _, s in base.values())
print("基线各步：", base)
print(f"覆盖那一遍的下限（基线各步秒数相加，未插桩、增量编译、含红的 two-test-checker-tier 只跑了 47.3 秒）= {c0:.1f} 秒 = {c0/60:.1f} 分钟")
recheck = frac_unviable * N * mean_unviable
print(f"无效（编不过）每次全部重判：{frac_unviable*N:.0f} 条 × {mean_unviable:.2f} 秒 = {recheck:.0f} 秒")

def scenario(name, files, per_item):
    own = sum(cnt[f] for f in files)
    hours_low = (own * per_item + (N - own) * frac_unviable * mean_unviable + c0) / 3600
    crate_wide_extra = 0
    has_core = any("singlefs-core" in f for f in files)
    has_checker = any("singlefs-checker/" in f for f in files)
    crate_items = (core_n if has_core else 0) + ((N - core_n) if has_checker else 0)
    hours_high = (max(own, crate_items) * per_item + (N - max(own, crate_items)) * frac_unviable * mean_unviable + c0) / 3600
    return own, hours_low, hours_high

fl = sorted(cnt.items(), key=lambda t: t[1])
print("\n三档（每档三种选文件的法子）：条数 = 被改文件自己名下的条数（P1「被改文件的 sha256」必重判）；")
print("下界 = 只重判这些条；上界 = 被改的是 core/checker 就把整个包的条数都重判（关联文件按包算的最坏）。per_item 取两种：整套重判 / 先只跑上次抓到它的测试（粗估）")
print("k\t选法\t自己名下条数\t占清单\t整套重判 下界h\t整套重判 上界h\t先跑T 下界h\t先跑T 上界h")
for k in (1, 5, 20):
    mid = len(fl) // 2
    lo = max(0, mid - k // 2)
    for label, files in (("最小", [f for f, _ in fl[:k]]), ("居中", [f for f, _ in fl[lo:lo + k]]), ("最大", [f for f, _ in fl[-k:]])):
        own, lowa, higha = scenario(label, files, mean_steady)
        _, lowb, highb = scenario(label, files, mean_t_first)
        print(f"{k}\t{label}\t{own}\t{own/N:.3f}\t{lowa:.1f}\t{higha:.1f}\t{lowb:.1f}\t{highb:.1f}")

out = subprocess.run(["git", "log", "-n", "150", "--no-merges", "--format=@@%h", "--name-only", "--", "crates"], capture_output=True, text=True, cwd=subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, cwd=os.path.dirname(os.path.abspath(__file__))).stdout.strip()).stdout
commits = []
for l in out.splitlines():
    if l.startswith("@@"):
        commits.append([l[2:], []])
    elif l.strip():
        commits[-1][1].append(l.strip())
touch = [(h, [f for f in fs if f in cnt]) for h, fs in commits]
nz = [(h, fs) for h, fs in touch if fs]
owns = [sum(cnt[f] for f in fs) for _, fs in nz]
print(f"\n最近 150 个非合并提交里动了 crates/ 的 {len(commits)} 个，其中动了清单里文件的 {len(nz)} 个；每个提交动的清单文件数：{sorted(len(fs) for _, fs in nz)}")
print(f"这些提交自己名下条数占清单：中位 {st.median(owns)/N:.3f}，最大 {max(owns)/N:.3f}；动了 core 或 checker 的提交整包条数占清单中位 {st.median([(core_n if any('singlefs-core' in f for f in fs) else 0) + ((N-core_n) if any('singlefs-checker/' in f for f in fs) else 0) for _, fs in nz])/N:.3f}")
med_own = st.median(owns)
print(f"按中位自己名下 {med_own:.0f} 条：整套重判 {(med_own*mean_steady + (N-med_own)*frac_unviable*mean_unviable + c0)/3600:.1f} 小时；先跑T {(med_own*mean_t_first + (N-med_own)*frac_unviable*mean_unviable + c0)/3600:.1f} 小时")

s1 = sorted(float(r["test_seconds"]) for r in last.values() if r["outcome"] == "stage-one-only")
print(f"只过了第一段的 {len(s1)} 条里，第一段整趟（nextest 通过）的测试秒数：最小 {s1[0]}、中位 {s1[len(s1)//2]}、最大 {s1[-1]}；基线 one-test = {base['one-test'][1]}")

# core 各模块文件的静态依赖闭包（只认 crate::<模块> 与 super::<模块>，粗）
import re
core_dir = os.path.join(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, cwd=os.path.dirname(os.path.abspath(__file__))).stdout.strip(), "crates/singlefs-core/src")
mods = [f[:-3] for f in os.listdir(core_dir) if f.endswith(".rs") and f != "lib.rs"]
graph = {}
for m in mods:
    text = open(f"{core_dir}/{m}.rs").read()
    graph[m] = {x for x in mods if x != m and re.search(r"\bcrate::" + x + r"\b|\bsuper::" + x + r"\b", text)}
def closure(m):
    seen, stack = set(), [m]
    while stack:
        a = stack.pop()
        for b in graph[a]:
            if b not in seen:
                seen.add(b)
                stack.append(b)
    return seen
sizes = collections.Counter(len(closure(m)) for m in mods)
print(f"core {len(mods)} 个模块文件的静态依赖闭包大小（含经环回到自己）分布：{sorted(sizes.items(), reverse=True)}")

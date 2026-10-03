#!/usr/bin/env python3
"""V3：最近 N 个动了 core / checker 的 src 的提交，按函数与按文件各要重判清单里多少条（只算「被改的那一份」，不算关联文件）。
用法：fn_changes.py <仓> <副本（库目录这一版的树）> <库目录版本目录> <N>
按函数：变异所在的最内层 fn（键 = impl 头、函数名、同名第几个）在那次提交里变了，或那份文件「函数外面的部分」变了，就重判。
按文件：变异所在文件变了就重判。清单取库目录这一版（今天的），提交是更早的：函数按键对上才算，对不上的不算（低估按函数一侧）。"""
import collections, json, os, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fnparse import items, outside_of_functions, digest
REPO, TREE, D, N = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
L = json.load(open(os.path.join(D, "list.json")))
def keyed(text):
    out, seen = {}, collections.Counter()
    for f in sorted(items(text), key=lambda f: f["start"]):
        k = (f["header"], f["name"]); seen[k] += 1
        out[k + (seen[k],)] = digest(f["header"] + text[f["start"]:f["end"]])
    return out
by_file = collections.defaultdict(list)
for m in L:
    by_file[m["file"]].append(m)
current_key = {}
for path, ms in by_file.items():
    text = open(os.path.join(TREE, path), encoding="utf-8").read()
    fns = sorted(items(text), key=lambda f: f["start"]); seen = collections.Counter(); keys = []
    for f in fns:
        k = (f["header"], f["name"]); seen[k] += 1; keys.append((f, k + (seen[k],)))
    starts = [0] + [i + 1 for i, c in enumerate(text) if c == "\n"]
    for m in ms:
        off = starts[m["start"][0] - 1] + m["start"][1] - 1
        around = [(f, k) for f, k in keys if f["start"] <= off < f["end"]]
        current_key[m["sequence"]] = min(around, key=lambda x: x[0]["end"] - x[0]["start"])[1] if around else None
git = lambda *a: subprocess.run(["git", "-C", REPO, *a], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True).stdout
commits = git("log", "--format=%H", "-n", str(N), "--", "crates/singlefs-core/src", "crates/singlefs-checker/src").split()
print("commit changed-fns outside-changed-files per-file-reruns per-function-reruns per-function-reruns-if-outside-not-counted list-size", len(L))
for c in commits:
    files = [f for f in git("diff", "--name-only", c + "^", c, "--", "crates/singlefs-core/src", "crates/singlefs-checker/src").split() if f.endswith(".rs")]
    changed_fns, per_file, per_fn, per_fn_no_out, outside_files = 0, 0, 0, 0, 0
    for f in files:
        old, new = git("show", f"{c}^:{f}"), git("show", f"{c}:{f}")
        ko, kn = keyed(old), keyed(new)
        changed = {k for k in set(ko) | set(kn) if ko.get(k) != kn.get(k)}
        changed_fns += len(changed)
        out_changed = digest(outside_of_functions(old)) != digest(outside_of_functions(new))
        outside_files += out_changed
        ms = by_file.get(f, [])
        per_file += len(ms)
        hit = [m for m in ms if current_key.get(m["sequence"]) in changed or current_key.get(m["sequence"]) is None]
        per_fn_no_out += len(hit)
        per_fn += len(ms) if out_changed else len(hit)
    print(c[:8], changed_fns, outside_files, per_file, per_fn, per_fn_no_out)

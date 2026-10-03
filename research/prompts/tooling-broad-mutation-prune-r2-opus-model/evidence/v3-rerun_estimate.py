#!/usr/bin/env python3
"""V3：这一版判过「抓到」的变异（T 已知、T 执行到的文件在 executed-files-per-test.json 里），拿最近 N 个提交逐个当「下一版」，数各种键下要重判几条。
用法：rerun_estimate.py <仓> <副本> <库目录版本目录> <executed-files-per-test.json> <N>
按文件（第一轮 P1+P2）：被改文件、T 的测试文件、T 执行到的文件有一个变了就重判。
读法甲（第二版字面）：被改函数或函数外面的部分变了、T 的测试文件变了、T 执行到的文件（含被改文件）有一个变了。
读法乙（第二版意图）：同甲，只是 T 执行到的文件里拿掉被改文件；T 的测试文件仍按文件（单元测试的测试文件就是被改文件）。
读法乙宽：同乙，T 的测试文件就是被改文件时，只比 T 那个测试函数。"""
import collections, csv, json, os, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from fnparse import items, outside_of_functions, digest
REPO, TREE, D, SETS, N = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5])
sets = json.load(open(SETS))
L = {m["sequence"]: m for m in json.load(open(os.path.join(D, "list.json")))}
rows = [r for r in csv.reader((l for l in open(os.path.join(D, "results.tsv")) if not l.startswith("#")), delimiter="\t") if r[3] == "caught"]
def keys_of(text):
    out, seen = {}, collections.Counter()
    for f in sorted(items(text), key=lambda f: f["start"]):
        k = (f["header"], f["name"]); seen[k] += 1
        out[k + (seen[k],)] = (f, digest(f["header"] + text[f["start"]:f["end"]]))
    return out
def fn_key_at(path, line, col):
    text = open(os.path.join(TREE, path), encoding="utf-8").read()
    starts = [0] + [i + 1 for i, c in enumerate(text) if c == "\n"]; off = starts[line - 1] + col - 1
    around = [(f, k) for k, (f, _) in keys_of(text).items() if f["start"] <= off < f["end"]]
    return min(around, key=lambda x: x[0]["end"] - x[0]["start"])[1] if around else None
sample = []
for r in rows:
    binary, test = r[5].split(" ", 1)
    key = f"{binary} {test}"
    if key not in sets:
        continue
    m = L[int(r[0])]
    test_file = m["file"] if binary == "singlefs-core" else f"crates/singlefs-harness/tests/{binary.split('::', 1)[1]}.rs"
    test_fn = ("", test.split("::")[-1], 1)
    sample.append((m["file"], fn_key_at(m["file"], *m["start"]), test_file, test_fn, set(sets[key])))
print(f"caught-in-this-version={len(rows)} with-executed-file-set={len(sample)} (unit-test T={sum(1 for s in sample if s[2] == s[0])})")
UNPARSED = []
def keys_of_safe(text):
    try:
        return keys_of(text)
    except SystemExit:
        return {}
git = lambda *a: subprocess.run(["git", "-C", REPO, *a], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True).stdout
print("commit changed-files changed-fns | reruns: per-file 甲 乙 乙宽 of", len(sample))
for c in git("log", "--format=%H", "-n", str(N), "--", "crates/singlefs-core/src", "crates/singlefs-checker/src").split():
    files = [f for f in git("diff", "--name-only", c + "^", c).split() if f.endswith(".rs") or f.endswith(".toml")]
    changed_fns, outside = {}, {}
    for f in files:
        old, new = git("show", f"{c}^:{f}"), git("show", f"{c}:{f}")
        if f.endswith(".rs"):
            try:
                ko, kn = keys_of(old), keys_of(new)
                changed_fns[f] = {k for k in set(ko) | set(kn) if (ko.get(k) or (None, None))[1] != (kn.get(k) or (None, None))[1]}
                outside[f] = digest(outside_of_functions(old)) != digest(outside_of_functions(new))
            except SystemExit:  # 切不开：照正文退回按文件
                UNPARSED.append(f"{c[:8]}:{f}")
                changed_fns[f] = {k for k in keys_of_safe(new)} | {None}
                outside[f] = True
    fs = set(files); counts = collections.Counter()
    for mfile, mkey, tfile, tfn, executed in sample:
        fn_or_out = mfile in fs and (mkey in changed_fns.get(mfile, set()) or mkey is None or outside.get(mfile, False))
        others = (executed - {mfile}) & fs
        counts["per-file"] += bool(({mfile, tfile} | executed) & fs)
        counts["甲"] += bool(fn_or_out or tfile in fs or executed & fs)
        counts["乙"] += bool(fn_or_out or tfile in fs or others)
        counts["乙宽"] += bool(fn_or_out or others or (tfile != mfile and tfile in fs) or (tfile == mfile and (tfn in changed_fns.get(mfile, set()) or None in changed_fns.get(mfile, set()))))
    print(c[:8], len(fs), sum(len(v) for v in changed_fns.values()), "|", counts["per-file"], counts["甲"], counts["乙"], counts["乙宽"])
print("files the syntax splitter could not parse (fell back to per-file):", len(UNPARSED), " ".join(UNPARSED))

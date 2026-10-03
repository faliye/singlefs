#!/usr/bin/env python3
"""apply.py <repo> <list.json> <sequence> | restore <repo> : 照驱动的 replace_span 写法把一条变异写进副本（加同样的标记），原件存 .orig。"""
import json, os, shutil, sys
MARK = " /* ~ changed by cargo-mutants ~ */"
def offset_of(lines, line, column):
    return sum(len(l) for l in lines[:line - 1]) + column - 1
def replace_span(text, start, end, replacement):
    lines = text.splitlines(keepends=True)
    a = offset_of(lines, *start); b = offset_of(lines, *end)
    return text[:a] + replacement + text[b:]
if sys.argv[1] == "restore":
    repo = sys.argv[2]
    for root, _, files in os.walk(repo):
        for f in files:
            if f.endswith(".orig"):
                p = os.path.join(root, f); shutil.move(p, p[:-5]); print("restored", p[:-5])
    sys.exit(0)
repo, lst, seq = sys.argv[1], sys.argv[2], int(sys.argv[3])
item = [i for i in json.load(open(lst)) if i["sequence"] == seq][0]
path = os.path.join(repo, item["file"])
if os.path.exists(path + ".orig"):
    sys.exit("already mutated: " + path)
text = open(path, encoding="utf-8").read()
new = replace_span(text, tuple(item["start"]), tuple(item["end"]), item["replacement"] + MARK)
assert new != text
shutil.copyfile(path, path + ".orig")
open(path, "w", encoding="utf-8").write(new)
print("applied", seq, item["name"])

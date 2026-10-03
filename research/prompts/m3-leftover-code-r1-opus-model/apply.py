#!/usr/bin/env python3
"""apply.py <变异名>：从 clean 恢复，再把这个变异施加到 work 上（每处恰好命中一次）。apply.py --restore：只恢复。"""
import json, os, shutil, sys
D = os.environ.get("OPUS_DIR", "/tmp/claude-1000/m3-leftover-code-r1-opus")
CLEAN = D + "/clean"
for root, _, files in os.walk(CLEAN):
    for f in files:
        src = os.path.join(root, f)
        shutil.copyfile(src, D + "/work" + src[len(CLEAN):])
if sys.argv[1] != "--restore":
    for rel, old, new in json.load(open(D + "/mut/mutants.json"))[sys.argv[1]]:
        path = D + "/work/" + rel
        text = open(path).read()
        assert text.count(old) == 1, (rel, text.count(old))
        open(path, "w").write(text.replace(old, new))
print("applied", sys.argv[1])

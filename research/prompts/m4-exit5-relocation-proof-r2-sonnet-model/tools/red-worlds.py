#!/usr/bin/env python3
"""单文件 Verus 模型的必红世界：逐行把「原文」换成「替换文」（原文须在文件里恰好命中一次），直接起 verus，期望报证明错。

用法：red-worlds.py <源码目录> <行表 tsv> <工作目录>
行表四列（制表符分隔）：文件名、世界名、原文、替换文。每行输出一行：世界名 rc errors verified 落点（最先报错的行）。
verus 参数同门禁 scripts/gate/verus-proof-check.py 的 verus_arguments（--no-cheating --crate-type=lib --edition=2021 --rlimit 10），没起门禁。
"""
import json
import os
import re
import shutil
import subprocess
import sys

src, table, work = sys.argv[1:4]
VERUS = os.path.expanduser("~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus")
os.makedirs(work, exist_ok=True)
for row in open(table, encoding="utf-8"):
    if not row.strip() or row.startswith("#"):
        continue
    fname, name, old, new = row.rstrip("\n").split("\t")
    text = open(os.path.join(src, fname), encoding="utf-8").read()
    old = old.replace("\\n", "\n")
    new = new.replace("\\n", "\n")
    hits = text.count(old)
    if hits != 1:
        print(f"{name}\t原文命中{hits}次，跳过")
        continue
    mutant = os.path.join(work, "mutant_" + fname)
    with open(mutant, "w", encoding="utf-8") as handle:
        handle.write(text.replace(old, new))
    ran = subprocess.run([VERUS, "--no-cheating", "--crate-type=lib", "--edition=2021", "--num-threads", os.environ.get("THREADS", "31"),
                          "--output-json", "--rlimit", "10", mutant], capture_output=True, text=True)
    out = ran.stdout.strip()
    errors = verified = None
    if out.startswith("{"):
        res = json.loads(out).get("verification-results", {})
        errors = res.get("errors")
        verified = res.get("verified")
    landing = ""
    lines = ran.stderr.splitlines()
    for i, line in enumerate(lines):
        if line.startswith("error") and "aborting" not in line:
            for follow in lines[i + 1:i + 4]:
                if "-->" in follow:
                    landing = re.sub(r"^.*mutant_", "", follow.split("-->")[1].strip())
                    break
            break
    print(f"{name}\trc={ran.returncode}\terrors={errors}\tverified={verified}\t落点={landing}")
    os.remove(mutant)

#!/usr/bin/env python3
"""在副本 $MUT_W 上套一条变异、跑它点名的测试、还原、核还原。
用法：mutrow.py ROW                                  （副本里 crates/mutations.tsv 的行号）
      mutrow.py --adhoc 源文件 原文文件 替换文件 cargo-test 参数…   （自己写的变异，两个文件里是原文与替换文）"""
import hashlib, os, shlex, subprocess, sys
W = os.environ["MUT_W"].rstrip("/") + "/"
def sha(path): return hashlib.sha256(open(path, "rb").read()).hexdigest()
if sys.argv[1] == "--adhoc":
    source = sys.argv[2]
    original = open(sys.argv[3], encoding="utf-8").read().rstrip("\n")
    replacement = open(sys.argv[4], encoding="utf-8").read().rstrip("\n")
    arguments = sys.argv[5:]
    label = "adhoc"
else:
    fields = open(W + "crates/mutations.tsv", encoding="utf-8").read().split("\n")[int(sys.argv[1]) - 1].split("\t")
    label, source, original, replacement, command, _test = fields[:6]
    original = original.replace("\\n", "\n"); replacement = replacement.replace("\\n", "\n")
    arguments = shlex.split(command)
path = W + source
text = open(path, encoding="utf-8").read()
hits = text.count(original)
print("label:", label[:100]); print("anchor hits:", hits)
if hits != 1:
    sys.exit(3)
before = sha(path)
open(path, "w", encoding="utf-8").write(text.replace(original, replacement, 1))
try:
    done = subprocess.run(["cargo", "test", "--offline"] + arguments, cwd=W, capture_output=True, text=True, timeout=220)
    output = done.stdout + done.stderr
    lines = output.split("\n")
    for line in lines:
        s = line.strip()
        if s.startswith("Running ") or line.startswith("test result") or line.endswith("FAILED") or line.startswith("error"):
            print(s)
    for index, line in enumerate(lines):
        if "panicked at" in line:
            print("  PANIC:", " | ".join(x.strip() for x in lines[index:index + 3])[:1100])
    print("exit:", done.returncode)
finally:
    open(path, "w", encoding="utf-8").write(text)
    print("restored:", sha(path) == before)

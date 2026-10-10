#!/usr/bin/env python3
"""N5：把 crates/mutations.tsv 里指定的行（行号）逐条在草稿副本上跑一遍：原文换成替换文、跑点名的测试、还原。
只跑 harness 档与 core 单测的行（checker 档的行 / 装置内联用例是重型，不跑）。
用法：run_mutation_rows.py <草稿副本根> <行号>...   （整条经 run-with-memory-cap.sh 跑）"""
import re
import shlex
import subprocess
import sys

tree = sys.argv[1]
numbers = [int(x) for x in sys.argv[2:]]
lines = open("/tmp/claude-1000/m3-formal-code-b31-r1/tree/crates/mutations.tsv", encoding="utf-8").read().split("\n")
for number in numbers:
    fields = lines[number - 1].split("\t")
    name, path, original, replacement, arguments, test = fields[:6]
    target = f"{tree}/{path}"
    saved = open(target, encoding="utf-8").read()
    old = original.replace("\\n", "\n")
    assert saved.count(old) == 1, (number, saved.count(old))
    open(target, "w", encoding="utf-8").write(saved.replace(old, replacement.replace("\\n", "\n")))
    try:
        command = ["cargo", "test", "--offline"] + shlex.split(arguments)
        result = subprocess.run(command, cwd=tree, capture_output=True, text=True)
        text = result.stdout + result.stderr
        verdict = "抓到(红)" if (result.returncode != 0) else "没红"
        compile_error = "error[" in text or "could not compile" in text
        summary = [l for l in text.split("\n") if re.match(r"test result|test .* (ok|FAILED)$", l)]
        print(f"ROW {number} {verdict} rc={result.returncode} compile_error={compile_error} :: {name[:40]}")
        for l in summary[:3]:
            print("   ", l[:200])
    finally:
        open(target, "w", encoding="utf-8").write(saved)

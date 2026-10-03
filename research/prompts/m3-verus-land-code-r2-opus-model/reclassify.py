#!/usr/bin/env python3
"""harness-rows.sh 第一版把 cargo 在测试红时打的「error: test failed」也当成编译错（BUILD_ERROR），这里从原日志重判：
编译错只认 error[E…] 与 could not compile / aborting；不是编译错时，点名的测试（第 6 段，ANY 表示任一）出现在 FAILED 里算 RED。
用法：reclassify.py <变异表> <出口目录（含 test-<行号>.log 与 summary.tsv）>；按 summary.tsv 的次序每行打一行。"""
import re
import sys

table_path, out = sys.argv[1], sys.argv[2]
table_lines = open(table_path, encoding="utf-8").read().split("\n")
for summary_line in open(f"{out}/summary.tsv", encoding="utf-8").read().splitlines():
    row = int(summary_line.split("\t")[0])
    required_red = table_lines[row - 1].split("\t")[5]
    log = open(f"{out}/test-{row}.log", encoding="utf-8", errors="replace").read()
    if re.search(r"^error\[E\d+\]|^error: (could not compile|aborting)", log, re.M):
        verdict = "BUILD_ERROR"
    else:
        failed = re.findall(r"^test (.*) \.\.\. FAILED$", log, re.M)
        hit = bool(failed) if required_red == "ANY" else any(required_red in name for name in failed)
        verdict = "RED" if hit else "GREEN"
    print(f"{row}\t{verdict}\trequired={required_red[:90]}")

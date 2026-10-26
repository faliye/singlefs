#!/usr/bin/env python3
"""summarize_mutant_logs.py <日志目录>：每份变异日志一行：文件名、test result 行、第一处 panicked 的位置与随后第一行消息（截 240 字）。"""
import sys, pathlib
for path in sorted(pathlib.Path(sys.argv[1]).glob("*.log")):
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    result = next((l.split("; finished")[0] for l in lines if l.startswith("test result")), "no-test-result")
    where, message = "-", "-"
    for i, l in enumerate(lines):
        if "panicked at" in l:
            where = l.split("panicked at")[1].strip().rstrip(":")
            message = " ".join(lines[i + 1].split())[:240] if i + 1 < len(lines) else "-"
            break
    print(f"{path.name}\t{result}\t{where}\t{message}")

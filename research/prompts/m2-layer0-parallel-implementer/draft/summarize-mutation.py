#!/usr/bin/env python3
"""summarize-mutation.py <日志>…：每条变异红了哪些测试、各自红在哪一处（panicked at 那一行与断言消息）。"""
import re
import sys

for path in sys.argv[1:]:
    lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    print(f"== {path.rsplit('/', 1)[-1]}")
    for line in lines:
        if line.startswith("MUTATION ") and " EXIT " in line:
            print("  " + line)
    failed = [line for line in lines if re.match(r"^test .* \.\.\. FAILED$", line)]
    for line in failed:
        print("  " + line)
    current = None
    for index, line in enumerate(lines):
        header = re.match(r"^---- (\S+) stdout ----$", line)
        if header:
            current = header.group(1)
            continue
        if current and "panicked at" in line:
            where = line.split("panicked at", 1)[1].strip()
            message = []
            for follow in lines[index + 1:index + 4]:
                if follow.startswith("  left") or follow.startswith("note:") or follow.startswith("----"):
                    break
                message.append(follow)
            print(f"  {current} → {where} {' '.join(message)[:220]}")
            current = None

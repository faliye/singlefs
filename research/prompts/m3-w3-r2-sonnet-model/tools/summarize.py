#!/usr/bin/env python3
"""把装置结果文件排成可比的摘要：agg.py 的合计表 + 事件行（去掉 elapsed_ns，排序）+ 逐步打印行（有就带）。
用法：summarize.py <结果文件…>"""
import re, subprocess, sys, os
here = os.path.dirname(os.path.abspath(__file__))
files = sys.argv[1:]
print(subprocess.run([sys.executable, os.path.join(here, "agg.py"), *files], capture_output=True, text=True, check=True).stdout, end="")
lines = []
for path in files:
    for line in open(path, encoding="utf-8"):
        if line.startswith("E7RESULT name=event") or line.startswith("E7W3TRACE"):
            lines.append(re.sub(r" elapsed_ns=\d+", "", line.rstrip("\n")))
for line in sorted(lines):
    print(line)

#!/usr/bin/env python3
"""把真实语料里「sed 脚本解析不下去」新拒的命令，按 sed 脚本里出现的写法分类，只打条数（不打命令，语料里有私有路径）。
用法：classify-sed.py <run-real.txt>"""
import ast, collections, re, sys
lines = open(sys.argv[1], encoding="utf-8").read().splitlines()
cats = collections.Counter(); total = 0
for i, line in enumerate(lines):
    m = re.match(r"now:rc=(\d+)/n=(\d+)/([\d.]+)s base:rc=(\d+)/n=(\d+)/([\d.]+)s.*?\| (\w+) (\S+) \| (\S+) \| (.*)$", line)
    if not m or not (m[4] == "0" and m[1] == "2"):
        continue
    if i + 1 >= len(lines) or "sed 脚本解析不下去" not in lines[i + 1]:
        continue
    try:
        command = ast.literal_eval(m[10])
    except (ValueError, SyntaxError):
        continue
    total += 1
    # 只看带 sed 的那几行
    text = "\n".join(found.group(1) for l in command.splitlines() for found in re.finditer(r"\bsed\b([^|;&\n]*)", l))
    if re.search(r"\$\(\(|\$\[", text) and not re.search(r"\$\((?!\()", text):
        cats["算术展开 $((…)) / $[…]（没有命令替换）"] += 1
    elif re.search(r"\$\(\(|\$\[", text):
        cats["算术展开与命令替换并存"] += 1
    elif re.search(r"\$\((?!\()", text) or "`" in text:
        cats["命令替换 $(…) / 反引号"] += 1
    elif re.search(r"[aic]\\", text):
        cats["a\\ i\\ c\\ 多行文本"] += 1
    else:
        cats["其他（要人看）"] += 1
print("sed 解析不下去的新拒总数", total)
for k, v in cats.most_common():
    print(v, k)

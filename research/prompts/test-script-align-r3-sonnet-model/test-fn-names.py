#!/usr/bin/env python3
"""L2：#[test] 函数名里带来历（欠账号 c<三位数>_、实验号 e<数>、里程碑样的 m<数>_）的有几条。只读。"""
import glob, os, re, subprocess
root = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip()
os.chdir(root)
pattern = re.compile(r'#\[test\][^\n]*\n(?:\s*#\[[^\n]*\n)*\s*(?:pub\s+)?fn\s+([A-Za-z0-9_]+)')
debt = re.compile(r'^c[0-9]{3}_')
other = re.compile(r'(^|_)(e[0-9]{2,3}|m[0-9])(_|$)')
total = 0; debt_hits = []; other_hits = []
for path in glob.glob('crates/**/*.rs', recursive=True):
    text = open(path, encoding='utf-8', errors='replace').read()
    for match in pattern.finditer(text):
        total += 1
        name = match.group(1)
        if debt.search(name): debt_hits.append((path, name))
        elif other.search(name): other_hits.append((path, name))
print("test fns:", total)
print("欠账号开头 c<三位数>_ 的:", len(debt_hits), "分在", len({p for p, _ in debt_hits}), "份文件")
for p, n in sorted(debt_hits)[:30]: print("  ", p, n[:70])
print("实验号或 m<数> 的（待判是不是来历）:", len(other_hits))
for p, n in sorted(other_hits): print("  ", p, n[:70])

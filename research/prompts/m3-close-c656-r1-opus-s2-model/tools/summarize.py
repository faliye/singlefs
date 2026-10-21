"""汇总原型日志：按（段、择到哪个池、判定）数态数。用法：python3 summarize.py <日志…>"""
import re, sys, collections
c = collections.Counter(); total = 0; done = []
for path in sys.argv[1:]:
    for line in open(path):
        if line.startswith("name=done"):
            done.append(line.strip())
        if not line.startswith("name=sig"):
            continue
        n = int(re.search(r" count=(\d+) ", line).group(1)); total += n
        phase = re.search(r" phase=(\S+)", line).group(1)
        choice = re.search(r" choice=(\S+?)(@|\s)", line).group(1)
        verdict = re.search(r" verdict=(.*)$", line).group(1)
        torn = re.search(r" torn=(\S+)", line).group(1)
        c[(phase, choice, verdict, "torn" if torn == "true" else "whole")] += n
for d in done: print(d)
print("total_states", total)
for k, v in sorted(c.items()): print(v, *k)

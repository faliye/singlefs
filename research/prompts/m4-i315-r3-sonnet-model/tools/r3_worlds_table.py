"""把 r3_worlds 日志压成表：一行一个（世界, 覆盖写次数），一列一个臂；格里 3 个字符依次是 I-3.1、I-3.11、I-3.15（o 成立、X 违例、? 别的），
后面带「其余违例」里出现过的名字。FACTS 行另列：回退点上每份的事实（自证 / 头里诞生代号 / 记录分配代 / gen / fits / cmp / journal）。
用法：python3 -I r3_worlds_table.py <日志> [--facts]"""
import re, sys
from collections import OrderedDict
rows = OrderedDict()
arms = []
facts = OrderedDict()
for line in open(sys.argv[1], encoding="utf-8"):
    m = re.match(r"SCEN arm=(\S+) (\w+) N=(\d+) : I-3.1=(\S+) I-3.11=(\S+) I-3.15=(\S+) 其余违例=(.*)$", line.strip())
    if m:
        arm, mode, n, a, b, c, rest = m.groups()
        if arm not in arms:
            arms.append(arm)
        sym = "".join("o" if v == "成立" else ("X" if v == "违例" else "?") for v in (a, b, c))
        others = re.findall(r"I-[0-9.]+", rest)
        rows.setdefault((mode, int(n)), {})[arm] = sym
        continue
    m = re.match(r"FACTS (\w+) N=(\d+) : (.*)$", line.strip())
    if m:
        facts[(m[1], int(m[2]))] = m[3]
print("世界 N | " + " ".join(f"{a}" for a in arms))
for (mode, n), cells in rows.items():
    if n < 24 and "--all" not in sys.argv:
        continue
    print(f"{mode} N={n} | " + " ".join(f"{cells.get(a, '-')}" for a in arms))
if "--facts" in sys.argv:
    for (mode, n), f in facts.items():
        print("FACTS", mode, n, f)

"""把 r3_fixtures 日志压成表：一行一张盘，一列一个臂；格里 3 个字符依次是 I-3.1、I-3.11、I-3.15（o 成立、X 违例、n 不适用、? 别的）。
最后一列「臂间一致」：九个臂的格逐字相同写 同，不同写 异。用法：python3 -I r3_fixtures_table.py <日志>"""
import re, sys
from collections import OrderedDict
rows = OrderedDict()
arms = []
def sym(v):
    return "o" if v.startswith("成立") else ("X" if v.startswith("违例") else ("n" if v.startswith("不适用") else "?"))
for line in open(sys.argv[1], encoding="utf-8"):
    m = re.match(r"SCEN arm=(\S+) (.*?) / (.*?) : I-3.1=(.*?) ; I-3.11=(.*?) ; I-3.15=(.*?) ; 全部违例=", line.strip())
    if not m:
        continue
    arm, scen, label, a, b, c = m.groups()
    if arm not in arms:
        arms.append(arm)
    rows.setdefault((scen, label), {})[arm] = sym(a) + sym(b) + sym(c)
print("盘 | " + " ".join(arms) + " | 臂间一致")
for (scen, label), cells in rows.items():
    values = [cells.get(a, "-") for a in arms]
    print(f"{scen} / {label} | " + " ".join(values) + " | " + ("同" if len(set(values)) == 1 else "异"))

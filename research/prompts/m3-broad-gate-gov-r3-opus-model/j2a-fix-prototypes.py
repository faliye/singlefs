#!/usr/bin/env python3
"""攻方 J2-a 改法的原型（只在攻方模型上量，零轮）：门禁两处改法各一份树拷贝，F1 超时基线认这一版 baseline.tsv 里这一步任何一行绿记录（登记的那一行还在就对上）；
F2 门禁不因被盖过的登记行判红，只要每条 ESCALATED 有一行复核得过的这一版登记行（复核不过的行只报不红）；F12 两样都改。
用法：j2a-fix-prototypes.py <原样树根> <放拷贝的目录>"""
import os, shutil, subprocess, sys
base, into = sys.argv[1:3]
STAGE = ".claude/gate.d/checker-tier-broad-mutation.sh"
F1 = ('                    if not line.startswith("#") and len(fields) == 4 and fields[0] == row["step"] and fields[1] == "ok":\n                        found = fields[2]\n',
      '                    if not line.startswith("#") and len(fields) == 4 and fields[0] == row["step"] and fields[1] == "ok":\n                        found = fields[2]\n'
      '                        if fields[2] == row["baseline"]:\n                            return fields[2]\n')
F2 = ('            found = row_problems(row)\n            if found:\n                registry_problems.append(',
      '            found = row_problems(row)\n            if found:\n                print("     （只报不红）登记表第 %d 行复核不过：%s" % (line_number, "；".join(found))); continue\n                registry_problems.append(')
for name, edits in (("F1", [F1]), ("F2", [F2]), ("F12", [F1, F2])):
    target = os.path.join(into, name); shutil.rmtree(target, ignore_errors=True)
    subprocess.run(["rsync", "-a", base.rstrip("/") + "/", target + "/"], check=True)
    path = os.path.join(target, STAGE); text = open(path, encoding="utf-8").read()
    for old, new in edits:
        if text.count(old) != 1:
            sys.exit(f"{name}：旧串命中 {text.count(old)} 次")
        text = text.replace(old, new)
    open(path, "w", encoding="utf-8").write(text)
    print(name)

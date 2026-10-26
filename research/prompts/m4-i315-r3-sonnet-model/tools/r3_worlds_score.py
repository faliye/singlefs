"""固定盘面记分：只看覆盖写 24 与 30 次（位置项已转出根环、走「没有位置项可比」那一格）的世界。
合法世界（实现隔离了、盘上留着旧住户或头写零，应当不红）按「不红」记；缺陷世界 LeakWithBothCopiesIntact（泄漏、两份完好，应当红）按「红」记；
LeakThenForeignUnit（泄漏之后槽被占，字节与合法同形，I-3.10 另红）单列。用法：python3 -I r3_worlds_score.py <日志>"""
import re, sys
LEGIT = ["ForeignUnitOnBothDevices", "ForeignUnitOnDeviceZero", "ZeroedHeaderOnDeviceZero", "DifferentForeignUnitsOnTheTwoDevices",
         "SameIdentityOnBothDevices", "SameIdentityOnDeviceZero", "RemadePoolSameBirthOnBothDevices", "RemadePoolSameBirthOnDeviceZero",
         "EarlierInstanceLeftoverOnBothDevices", "EarlierInstanceLeftoverOnDeviceZero"]
cells = {}
arms = []
for line in open(sys.argv[1], encoding="utf-8"):
    m = re.match(r"SCEN arm=(\S+) (\w+) N=(\d+) : I-3.1=(\S+) I-3.11=(\S+) I-3.15=(\S+) ", line)
    if not m:
        continue
    arm, mode, n, a, b, c = m.groups()
    if int(n) not in (24, 30):
        continue
    if arm not in arms:
        arms.append(arm)
    cells[(arm, mode, int(n))] = "ooo" if (a, b, c) == ("成立",) * 3 else ("XXX" if (a, b, c) == ("违例",) * 3 else "mixed")
print("arm | 合法世界不红(共 %d 个世界 x 2 档) | 缺陷世界红(1 x 2) | 泄漏之后槽被占不红(1 x 2) | 仍红的合法世界" % len(LEGIT))
for arm in arms:
    quiet = [(m, n) for m in LEGIT for n in (24, 30) if cells.get((arm, m, n)) == "ooo"]
    red = [n for n in (24, 30) if cells.get((arm, "LeakWithBothCopiesIntact", n)) == "XXX"]
    occupant = [n for n in (24, 30) if cells.get((arm, "LeakThenForeignUnit", n)) == "ooo"]
    bad = sorted({m for m in LEGIT for n in (24, 30) if cells.get((arm, m, n)) != "ooo"})
    print(f"{arm} | {len(quiet)}/{2 * len(LEGIT)} | {len(red)}/2 | {len(occupant)}/2 | {','.join(bad) if bad else '无'}")

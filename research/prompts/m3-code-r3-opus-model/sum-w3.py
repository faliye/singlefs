#!/usr/bin/env python3
"""汇总 W3 三盘扫描日志（logs/w3/<名>-012/sweep-*.log）：每个错实现 × txg 一行，五个用户动作的结局（做成 / 拒因成员名）、盘变没变与入口之后 checker 条数。
三盘副本上原样代码入口之后 checker 也报 2 条（盘 2 的 I-3.1，checker 不认三盘池），所以只比结局与「盘变没变」。"""
import glob, os, re, collections
D = "/tmp/claude-1000/m3-code-r3-opus"
for directory in sorted(glob.glob(os.path.join(D, "logs/w3/*-012"))):
    name = os.path.basename(directory)
    cells = collections.OrderedDict()
    for path in sorted(glob.glob(os.path.join(directory, "sweep-*.log"))):
        for line in open(path, encoding="utf-8"):
            m = re.search(r"W3SWEEP .* txg=(\d+) region=(\d) slot=(\d) root_dev=(\d) entry=(\w+) target=(\w+) outcome=(.*?) unchanged=(\w+) checker=(\d+)", line)
            if not m:
                continue
            txg, region, slot, dev, entry, target, outcome, unchanged, checker = m.groups()
            short = "ok" if outcome.startswith("做成了") else re.split(r"[ ({]", outcome.replace("CallerInputsDisagreeWithTheDisk(", ""), 1)[0]
            label = {"SessionPublish": "S", "NormalUnmount": "U", "FloorRaiseToTheAdmissionCeiling": "F"}.get(entry, "R1" if target == "FirstFileVersion" else "Rp")
            cells.setdefault((int(txg), region, dev), []).append(f"{label}:{short}{'' if unchanged == 'true' else '(changed,c' + checker + ')'}")
    for (txg, region, dev), outcomes in sorted(cells.items()):
        print(f"{name} txg={txg} region={region} root_dev={dev} " + " ".join(outcomes))

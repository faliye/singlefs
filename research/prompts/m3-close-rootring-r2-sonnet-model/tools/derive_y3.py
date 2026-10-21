#!/usr/bin/env python3
"""Y3 推算（不是真挂载跑出来的）：读第一轮攻方 union.out 的 100 格，逐格重算「触发取哪一种读者」之后三处落哪一版、成不成。

输入：union.out（每格：所选根上 走读 / 打开 / 建上一版 通不通，今天恢复 / 只读 / 可写各落哪一版）。
假设（只此一条）：退到的那一版 P 的三样读者，在 R 自己写的单元出了故障的格里全通；在「共用实例表读回全零」的格里 走读 / 建上一版 不通、打开 通。
校准：用「各处取自己的读者」重算，必须与 union.out 里今天的 rec / ro / rw 逐格相等，否则本脚本退 1。
用法：python3 derive_y3.py <union.out>
"""
import collections
import re
import sys

TRIGGERS = {
    "walk": lambda r: not r[0],
    "open": lambda r: not r[1],
    "rebuild": lambda r: not r[2],
    "any": lambda r: not all(r),
    "all": lambda r: not any(r),
}


def load(path):
    rows = []
    for line in open(path, encoding="utf-8"):
        if not line.startswith("UNION kind"):
            continue
        m = re.search(
            r"kind=(\w+) k=(\d) img=(\w+) fault=(\S+) chosen=Ok\((\d+),(\d+)\) "
            r"readers\(walk,open,rebuild\)=\((\w+),(\w+),(\w+)\) today rec=(\S+) ro=(\S+) rw=(\S+) ",
            line,
        )
        kind, k, img, fault, ci, ct, w, o, r, rec, ro, rw = m.groups()
        rows.append(
            dict(
                img=img,
                fault=fault,
                chosen="Ok(%s,%s)" % (ci, ct),
                target="Ok(%s,%d)" % (ci, int(ct) - 1),
                R=(w == "true", o == "true", r == "true"),
                P=(False, True, False) if fault.startswith("instance_table") else (True, True, True),
                today=(rec, ro, rw),
            )
        )
    return rows


def outcome(row, mount, trigger, lenient):
    R, P = row["R"], row["P"]
    if row["img"] == "confirmed" or not TRIGGERS[trigger](R):
        return row["chosen"] if R[mount] else "Fail"
    if P[mount]:
        return row["target"]
    if lenient and R[mount]:
        return row["chosen"]
    return "Fail"


def main(path):
    rows = load(path)
    own = {0: "walk", 1: "open", 2: "rebuild"}
    for row in rows:
        if tuple(outcome(row, m, own[m], False) for m in range(3)) != row["today"]:
            print("校准不过：", row)
            return 1
    print("校准：%d 格，各处取自己的读者与今天逐格相等" % len(rows))
    for img in ("confirmed", "unwitnessed"):
        sub = [r for r in rows if r["img"] == img]
        for label, trig, lenient in [("today", None, False)] + [
            (t, t, le) for t in TRIGGERS for le in (False, True)
        ]:
            same_ok = one_version = lost_mounts = lost_cells = moved = 0
            for row in sub:
                if trig is None:
                    t = row["today"]
                else:
                    t = tuple(outcome(row, m, trig, lenient) for m in range(3))
                ok = [x for x in t if x != "Fail"]
                same_ok += len(set(t)) == 1 and "Fail" not in t
                one_version += len(set(ok)) <= 1
                cell_lost = False
                for a, b in zip(row["today"], t):
                    if a != "Fail" and b == "Fail":
                        lost_mounts += 1
                        cell_lost = True
                    if a != "Fail" and b != "Fail" and a != b:
                        moved += 1
                lost_cells += cell_lost
            print(
                "%-11s %-8s %-8s 三处都成且同版=%d 成的几处同版=%d 今天成改后不成=%d处(%d格) 换版=%d处"
                % (img, label, "宽" if lenient else "严", same_ok, one_version, lost_mounts, lost_cells, moved)
            )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))

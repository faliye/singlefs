#!/usr/bin/env python3
"""汇总 grid 日志：每个 (宽, k, 臂) 只看根槽落盘、挂载做成的那几格（H1 行 root_landed=true 且有 step=），逐步列 卸载 / 再挂 / 探针 / 被写掉的落点数。"""
import re, sys, collections
rows = collections.defaultdict(list)
for path in sys.argv[1:]:
    for line in open(path):
        if not line.startswith("H1 ") or "root_landed=true" not in line:
            continue
        m = re.match(r"H1 W=(\S+) k=(\d+) arm=(\S+) .*?step=(\d+)", line)
        if not m:
            continue
        w, k, arm, step = m.group(1), int(m.group(2)), m.group(3), int(m.group(4))
        if "OVERWRITE_FAILED" in line:
            rows[(w, k, arm)].append((step, "OVERWRITE_FAILED " + line.split("OVERWRITE_FAILED ")[1].split(" states=")[0]))
            continue
        get = lambda key: re.search(r"(?<= )" + key + r"=(\S+)", line).group(1)
        rows[(w, k, arm)].append((step, f"dmg={get('damaged_before_unmount')}/{get('r_placements')} um={get('unmount')} re={get('remount')} probe={get('probe')} chk={get('checker')} n={get('states')}"))
order = {"384": 0, "256": 1, "240": 2, "4g": 3}
for key in sorted(rows, key=lambda x: (order[x[0]], x[1], x[2])):
    steps = sorted(set(rows[key]))
    print(f"== W={key[0]} k={key[1]} arm={key[2]}")
    prev = None
    for step, text in steps:
        if text != prev:
            print(f"  step {step}: {text}")
        prev = text

#!/usr/bin/env python3
"""把 out/run-*.log 里的 name=count 行按（段、结局类）汇总。用法：summarize.py <日志>...
键形：s<段>__whole|torn__<择到的系统配置>__<恢复读回>__green|red__ro_…__rw_…__mixed|notmixed[__oldintact]
结局类：
  old_intact     择到旧池系统配置、恢复读回与最后确认的一版逐字节相同（键里带 oldintact）
  old_other      择到旧池系统配置、读回不是「最后确认的一版且来自同一条根」或读不出
  mixed_last     新池系统配置 + 旧池的根，读回的恰是最后确认的一版（键里 mixed，恢复读回 oldlast*）
  mixed_older    新池系统配置 + 旧池的根，读回更早一版（恢复读回 oldolder*）
  mixed_other    其余 mixed
  new_empty      新池系统配置、读回第 0 代空池
  refused        整池拒（系统配置择不到、两盘系统配置对不上、新系统配置而没有自证的根）
  other          其余
"""
import re
import sys
from collections import defaultdict


def klass(parts, mixed):
    choice, rec = parts[2], parts[3]
    key = "__".join(parts)
    if choice.startswith("old"):
        return "old_intact" if "__oldintact" in key else "old_other"
    if mixed:
        if rec.startswith("oldlast"):
            return "mixed_last"
        if rec.startswith("oldolder"):
            return "mixed_older"
        return "mixed_other"
    if choice == "new" and rec == "newempty":
        return "new_empty"
    if choice.startswith("err_") or rec.startswith("fail_"):
        return "refused"
    return "other"


def main(paths):
    for path in paths:
        tables = defaultdict(lambda: defaultdict(int))
        totals = defaultdict(int)
        checkers = {}
        for line in open(path, encoding="utf-8"):
            m = re.match(r"name=count h=(\S+) order=(\S+) count=(\d+) (\S+)$", line.strip())
            if m:
                name = f"h={m.group(1)} order={m.group(2)}"
                count, key = int(m.group(3)), m.group(4)
                parts = key.split("__")
                mixed = "__mixed" in key
                red = "__red__" in key
                kl = klass(parts, mixed)
                tables[name][(parts[0], kl)] += count
                tables[name][("all", kl)] += count
                tables[name][("all", "checker_red" if red else "checker_green")] += count
                totals[name] += count
            elif line.startswith("name=checker"):
                mm = re.match(r"name=checker h=(\S+) order=(\S+)", line)
                checkers[f"h={mm.group(1)} order={mm.group(2)}"] = line.strip()
            elif line.startswith("name=done"):
                print(line.strip())
        for name, table in tables.items():
            print(f"== {name} 总态数 {totals[name]}")
            for (seg, kl), n in sorted(table.items(), key=lambda kv: (kv[0][0] != "all", kv[0])):
                print(f"  {seg:>4} {kl:<14} {n}")
            if name in checkers:
                print("  " + checkers[name])


if __name__ == "__main__":
    main(sys.argv[1:])

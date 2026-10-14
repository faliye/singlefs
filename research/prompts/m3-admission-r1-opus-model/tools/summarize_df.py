#!/usr/bin/env python3
"""汇总 df 探针的 DF 行：每个候选在多少个状态上多报（> 真值）、多报的最小与最大字节数、少报的最大字节数，与真值是否单调核过。"""
import re
import sys

LINE = re.compile(r"^DF (?P<tag>.*?) txg=(?P<txg>\d+) truth_units=(?P<tu>\d+) truth_bytes=(?P<tb>\d+) monotone_check=(?P<mc>\w+) .*?\| (?P<rest>.*)$")
CAND = re.compile(r"(?P<name>[A-Z]_[A-Za-z0-9_]+)=(?P<value>-?\d+)\((?P<dir>over|under)[+-](?P<amount>\d+)\)")

def main(paths):
    for path in paths:
        stats = {}
        states = 0
        monotone_false = 0
        with open(path, encoding="utf-8") as handle:
            for line in handle:
                match = LINE.match(line.strip())
                if not match:
                    continue
                states += 1
                if match.group("mc") != "true":
                    monotone_false += 1
                for cand in CAND.finditer(match.group("rest")):
                    entry = stats.setdefault(cand.group("name"), {"over": 0, "under": 0, "over_min": None, "over_max": 0, "under_max": 0})
                    amount = int(cand.group("amount"))
                    if cand.group("dir") == "over":
                        entry["over"] += 1
                        entry["over_max"] = max(entry["over_max"], amount)
                        entry["over_min"] = amount if entry["over_min"] is None else min(entry["over_min"], amount)
                    else:
                        entry["under"] += 1
                        entry["under_max"] = max(entry["under_max"], amount)
        print(f"file={path.split('/')[-1]} states={states} monotone_check_false={monotone_false}")
        for name, entry in stats.items():
            print(f"  {name}: over_states={entry['over']} under_states={entry['under']} over_min_bytes={entry['over_min']} over_max_bytes={entry['over_max']} under_max_bytes={entry['under_max']}")

if __name__ == "__main__":
    main(sys.argv[1:])

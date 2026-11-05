#!/usr/bin/env python3
"""Read probe_window lines from a task log; own parser (not the investigator's)."""
import re, sys

def parse(path):
    rows = []
    with open(path, errors="replace") as f:
        for line in f:
            if "name=probe_window" not in line:
                continue
            d = {}
            for tok in line.split():
                if "=" not in tok:
                    continue
                k, v = tok.split("=", 1)
                d[k] = v
            row = {"line": line.rstrip("\n")}
            for k, v in d.items():
                if v.count(":") == 2:
                    n, s, a = v.split(":")
                    row[k] = (int(n), float(s), int(a))
                else:
                    try:
                        row[k] = float(v) if "." in v else int(v)
                    except ValueError:
                        row[k] = v
            rows.append(row)
    return rows

if __name__ == "__main__":
    rows = parse(sys.argv[1])
    print(len(rows))

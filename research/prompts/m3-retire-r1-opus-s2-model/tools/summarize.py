"""汇总 sweep 输出：按 (族, 读法或判据) 合计判过的版本数、打中的版本数、打中的历史数、峰值；再按族合计计数行。
用法：python3 -I summarize.py <sweep 输出文件…>（只读参数给的文件，输出排好序、不带耗时）。"""
import sys
from collections import defaultdict

cells = defaultdict(lambda: [0, 0, 0, 0, 0, 0, 0])  # histories, scans, r_scans, m_scans, r_peak, m_peak, histories_hit
counts = defaultdict(lambda: defaultdict(int))
peaks = defaultdict(lambda: defaultdict(int))
histories = defaultdict(int)
for path in sys.argv[1:]:
    with open(path, encoding="utf-8") as f:
        for line in f:
            parts = line.split()
            if len(parts) < 2 or parts[0] != "S2":
                continue
            kv = dict(p.split("=", 1) for p in parts[1:] if "=" in p)
            fam = kv.get("family")
            if kv.get("name") == "cell":
                c = cells[(fam, kv["shape"] if fam == "up" else "all", kv["reader"])]
                c[0] += 1
                c[1] += int(kv["scans"]); c[2] += int(kv["r_scans"]); c[3] += int(kv["m_scans"])
                c[4] = max(c[4], int(kv["r_peak"])); c[5] = max(c[5], int(kv["m_peak"]))
                c[6] += 1 if int(kv["r_scans"]) + int(kv["m_scans"]) > 0 else 0
            elif kv.get("name") == "counts":
                histories[fam] += 1
                for k, v in kv.items():
                    if k in ("name", "family", "shape", "geometry", "rollback_pick", "seed"):
                        continue
                    if k.endswith("_peak"):
                        peaks[fam][k] = max(peaks[fam][k], int(v))
                    else:
                        counts[fam][k] += int(v)
for (fam, shape, reader), c in sorted(cells.items()):
    print(f"S2SUM family={fam} reader={reader} histories={c[0]} versions_judged={c[1]} r_versions={c[2]} m_versions={c[3]} r_peak={c[4]} m_peak={c[5]} histories_hit={c[6]}")
for fam in sorted(histories):
    items = " ".join(f"{k}={v}" for k, v in sorted(counts[fam].items()))
    pk = " ".join(f"{k}={v}" for k, v in sorted(peaks[fam].items()))
    print(f"S2SUM family={fam} histories={histories[fam]} {items} {pk}")

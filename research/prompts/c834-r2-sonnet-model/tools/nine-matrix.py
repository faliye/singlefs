"""把 out/nine-<候选>.log 里的 ROW 行汇成「历史 × 候选」表，每格 = 盘序 0,1 / 盘序 1,0 的结局；做成时带新写掉 R 单元的份数。"""
import re, sys, collections
out = sys.argv[1]
cands = sys.argv[2:]
table = collections.OrderedDict()
for c in cands:
    for line in open(f"{out}/nine-{c}.log", encoding="utf-8"):
        m = re.match(r"ROW variant=\S+ history=(\S+) 盘序=(\d,\d) => (\S+).*新写掉的 R 单元份数=(\S+?)（", line)
        if m:
            h, order, kind, n = m.groups()
            kind = "做成" if kind == "做成" else "拒"
            cell = kind + (f"[{n[5:-1]}]" if kind == "做成" and n.startswith("Some(") else "")
            table.setdefault(h, {}).setdefault(c, {})[order] = cell
print("历史 \\ 候选 : " + " | ".join(cands))
for h, row in table.items():
    cells = []
    for c in cands:
        d = row.get(c, {})
        a, b = d.get("0,1", "?"), d.get("1,0", "?")
        cells.append(a if a == b else f"{a}/{b}")
    print(f"{h} : " + " | ".join(cells))

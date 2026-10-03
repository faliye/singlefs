# 独立数 Σ_ord 的交错条数：只按登记第一节的读法（rA 先于 rB、RA 先于 RB、SA 先于 SB），两盘时记录与系统配置槽按盘各对各，根槽一份、与盘无关。
from math import comb
def chain(tag, units, disks):
    ev = []
    for u in range(1, units + 1):
        for d in range(disks):
            ev.append(("a" if tag == "A" else "b") + str(u) + "@" + str(d))
    for d in range(disks): ev.append(("barrier", d))
    for d in range(disks): ev.append("r" + tag + "@" + str(d))
    for d in range(disks): ev.append(("barrier", d))
    ev.append("R" + tag)
    for d in range(disks): ev.append("S" + tag + "@" + str(d))
    for d in range(disks): ev.append(("barrier", d))
    return ev
def count(units, disks, ordered):
    A, B = chain("A", units, disks), chain("B", units, disks)
    def need(e):
        if isinstance(e, tuple) or not ordered: return None
        if e.startswith("rB") or e.startswith("SB"): return A.index("rA" + e[2:] if e.startswith("rB") else "SA" + e[2:])
        if e == "RB": return A.index("RA")
        return None
    waits = [need(e) for e in B]
    ways = [[0] * (len(B) + 1) for _ in range(len(A) + 1)]
    ways[0][0] = 1
    for i in range(len(A) + 1):
        for j in range(len(B) + 1):
            w = ways[i][j]
            if not w: continue
            if i < len(A): ways[i + 1][j] += w
            if j < len(B) and (waits[j] is None or i > waits[j]): ways[i][j + 1] += w
    return ways[len(A)][len(B)]
for units, disks, name in [(2, 1, "G1"), (1, 1, "G3"), (2, 2, "G4"), (1, 2, "two_disk_u1")]:
    print(name, "free", count(units, disks, False), "ordered", count(units, disks, True), "comb", comb(len(chain("A", units, disks)) * 2, len(chain("A", units, disks))))

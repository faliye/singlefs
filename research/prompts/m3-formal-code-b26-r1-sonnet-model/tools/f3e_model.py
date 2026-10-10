#!/usr/bin/env python3
"""J2 F3-e：照 crates/singlefs-checker-tier/src/crash_injection.rs 的 fingerprint_of_the_persisted_subset 与 seed_of_the_second_crash_points
（冻结副本第 3062、3077 行）在 Python 里逐位复写，数「二次崩溃种子」在入库那三条取样（A2、B3、C2）与我自己加宽的取样上有没有撞车，
再对几种改坏变体各数撞车数：F3-e 三条取样下撞车数 > 0 的变体算被罩住，= 0 的是漏网。这是纯算术模型，不进仓里的任何判定。
"""
import itertools, sys
M = (1 << 64) - 1
BASIS, PRIME, SALT = 0xcbf29ce484222325, 0x100000001b3, 0x73_65_63_6f_6e_64_63_72

def fingerprint(bits, window=None):
    h = BASIS
    for b in (bits if window is None else window(bits)):
        h = ((h ^ int(b)) * PRIME) & M
    return h

def seed(hist, seg, bits, mut=None):
    mut = mut or {}
    hist2 = mut.get("hist", lambda x: x)(hist)
    seg2 = mut.get("seg", lambda x: x)(seg)
    return hist2 ^ SALT ^ seg2 ^ fingerprint(bits, mut.get("window"))

def end12(landed):  # F3-d 的搭法：前 landed 个写落了，段末 12 个写取扣下集合非空的 4095 种
    for mask in range(4095):
        yield [True] * landed + [((mask >> k) & 1) == 1 for k in range(12)]

def domain_f3d():
    return [(0, seg, bits) for seg in range(64) for bits in end12(0)]

def domain_f3e():
    a2 = [(0, seg, bits) for seg in list(range(64)) + [x for k in range(6, 21) for x in (1 << k, (1 << k) + 1)] for bits in end12(0)]
    b3 = []
    L = 32
    for first in range(L):
        for withheld in [[first]] + [[first, second] for second in range(first + 1, L)]:
            b3.append((0, 16, [i not in withheld for i in range(L)]))
    c2 = [(h, 16, bits) for h in (0, 1, 1 << 63) for bits in end12(0)]
    return a2 + b3 + c2

def wider_domain():
    """我加宽的取样：段长 40、扣下 1..3 个写落在任意位置；段号 0..235 与 2^k、历史种子取本测试周期种子基起 4 个、0、2^63。"""
    base = 7_463_871_032_432_355_113
    out = []
    L = 40
    sets = [[]] + [[i] for i in range(L)] + [[i, j] for i in range(L) for j in range(i + 1, L)]
    sets += [[i, j, k] for i, j, k in itertools.combinations(range(0, L, 2), 3)]
    for hist in (0, base, base + 1, base + 2, 1 << 63):
        for seg in (0, 1, 63, 64, 100, 210, 235, 1 << 10):
            for w in sets:
                out.append((hist, seg, [i not in w for i in range(L)]))
    return out

def collisions(domain, mut=None):
    seen = {}
    for hist, seg, bits in domain:
        key = (hist, seg, tuple(bits))
        s = seed(hist, seg, bits, mut)
        seen.setdefault(s, set()).add(key)
    return sum(len(v) - 1 for v in seen.values() if len(v) > 1), len(seen)

MUTANTS = {
    "无改坏（入库那一种）": {},
    "M7a 指纹只折段末 12 个写": {"window": lambda b: b[-12:]},
    "只折段末 24 个写": {"window": lambda b: b[-24:]},
    "只折段末 32 个写": {"window": lambda b: b[-32:]},
    "只折前 32 个写": {"window": lambda b: b[:32]},
    "只折前 36 个写": {"window": lambda b: b[:36]},
    "M7d 段号只取低 6 位": {"seg": lambda s: s & 63},
    "段号只取低 20 位": {"seg": lambda s: s & ((1 << 20) - 1)},
    "段号只取低 21 位": {"seg": lambda s: s & ((1 << 21) - 1)},
    "M7f 历史种子只取最低位": {"hist": lambda h: h & 1},
    "历史种子去掉最高位": {"hist": lambda h: h & ((1 << 63) - 1)},
    "历史种子去掉第 62 位": {"hist": lambda h: h & ~(1 << 62) & M},
}

def separate_tests():
    """入库的四条取样各自一个集合（各断言自己的种子两两不同）：旧 F3-d 跨段号、F3-e 的 A2、B3、C2。"""
    f3e = domain_f3e()
    a2_n = 94 * 4095
    return {
        "F3-d 跨段号（段号 0..=63、长 12）": domain_f3d(),
        "F3-e A2（段号 0..=63 与 2^k、2^k+1）": f3e[:a2_n],
        "F3-e B3（长 32、扣 1 或 2 个写）": f3e[a2_n:a2_n + 528],
        "F3-e C2（历史种子 0、1、2^63）": f3e[a2_n + 528:],
    }

if __name__ == "__main__":
    tests = separate_tests()
    print("各条取样规模：", {k: len(v) for k, v in tests.items()})
    wide = wider_domain()
    print(f"{'变体':30s} " + " ".join(f"{k[:9]:>10s}" for k in tests) + f" {'联合':>8s} {'加宽':>8s}  判定")
    base_union = None
    for name, mut in MUTANTS.items():
        per = [collisions(d, mut)[0] for d in tests.values()]
        union = collisions([x for d in tests.values() for x in d], mut)[0]
        wide_c = collisions(wide, mut)[0]
        if name.startswith("无改坏"):
            base_union, base_wide = union, wide_c
        caught = any(c > 0 for c in per)
        verdict = "基线" if name.startswith("无改坏") else ("四条取样罩住" if caught else ("漏网：加宽取样才多出撞车" if wide_c > base_wide else "漏网：加宽取样也无差别"))
        print(f"{name:30s} " + " ".join(f"{c:10d}" for c in per) + f" {union:8d} {wide_c:8d}  {verdict}")
    print("联合 / 加宽 两列基线非 0：同一子集在 (历史种子 h, 段号 s) 与 (h', s') 上 h^s = h'^s' 时种子相同，例：", end="")
    a = seed(0, 17, [False] * 12); b = seed(1, 16, [False] * 12)
    print("(0,17) 与 (1,16) 的种子相同 =", a == b)

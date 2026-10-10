#!/usr/bin/env python3
"""F3 第二条路子：不经 Rust，按冻结副本里的常量（盐、FNV 起始值与乘数）自己算种子，复算内联用例与改法 F3-d 的 A、B、C 三条。
常量从冻结副本 crash_injection.rs 里按名字读，读不到退 3。用法：f3_model.py <冻结副本根>"""
import re, sys
src = open(f"{sys.argv[1]}/crates/singlefs-checker-tier/src/crash_injection.rs", encoding="utf-8").read()
def const(name):
    m = re.search(rf"const {name}: u64 = (0x[0-9a-f_]+);", src)
    if not m:
        print(f"读不到 {name}", file=sys.stderr); sys.exit(3)
    return int(m.group(1).replace("_", ""), 16)
SALT, BASIS, PRIME = const("SECOND_CRASH_POINT_SEED_SALT"), const("PERSISTED_SUBSET_FINGERPRINT_OFFSET_BASIS"), const("PERSISTED_SUBSET_FINGERPRINT_PRIME")
M = (1 << 64) - 1
def fnv(bits):
    h = BASIS
    for b in bits:
        h = ((h ^ int(b)) * PRIME) & M
    return h
def bits(mask, n):
    return [(mask >> i) & 1 for i in range(n)]
V = {
    "entered(fnv1a)": lambda s, g, p: s ^ SALT ^ g ^ fnv(p),
    "take(12)": lambda s, g, p: s ^ SALT ^ g ^ fnv(p[:12]),
    "take(16)": lambda s, g, p: s ^ SALT ^ g ^ fnv(p[:16]),
    "bitmask-shl": lambda s, g, p: s ^ SALT ^ g ^ int("".join(str(b) for b in p) or "0", 2),
    "drop-history-seed": lambda s, g, p: SALT ^ g ^ fnv(p),
    "drop-segment": lambda s, g, p: s ^ SALT ^ fnv(p),
    "or-not-xor": lambda s, g, p: s | SALT | g | fnv(p),
    "take(1)": lambda s, g, p: s ^ SALT ^ g ^ fnv(p[:1]),
}
subsets12 = [bits(m, 12) for m in range((1 << 12) - 1)]
for name, f in V.items():
    inline = len({f(0, 16, p) for p in subsets12})
    a = len({f(0, g, p) for g in range(64) for p in subsets12})
    b = len({f(0, 16, [1] * 16 + p) for p in subsets12})
    c = len({f(s, 16, p) for s in (0, 1) for p in subsets12})
    print(f"f3-model variant={name} inline={'green' if inline == 4095 else 'red'}({inline}) A={a}/262080 B={b}/4095 C={c}/8190")

# S2 守卫的 u32 仿真：新旧两个守卫对「真值」（整数算术）逐点比。用法：python3 guard_model.py
M = (1 << 32) - 1
SLOT_SHIFT = 14
NODE = 1 << 14
def u32(v): return v & M

def guard_old(off, length, w9, w10):
    x, y = off & M, off >> 32
    first = u32((x >> SLOT_SHIFT) | u32(y << (32 - SLOT_SHIFT)))
    slots = (length + NODE - 1) >> SLOT_SHIFT
    if ((x & (NODE - 1)) | (y >> (32 - SLOT_SHIFT))) != 0 or first < w9 or w10 < u32(first + slots):
        return None
    return True

def guard_new(off, length, w9, w10):
    x, y = off & M, off >> 32
    first = u32((x >> SLOT_SHIFT) | u32(y << (32 - SLOT_SHIFT)))
    slots = (length + NODE - 1) >> SLOT_SHIFT
    if ((x & (NODE - 1)) | (y >> SLOT_SHIFT)) != 0 or first < w9 or first > w10 or u32(w10 - first) < slots:
        return None
    return True

def truth(S, length, w9, w10):
    n = (length + NODE - 1) >> SLOT_SHIFT
    return True if (S < (1 << 32) and w9 <= S and S + n <= w10) else None

edges = sorted({0,1,2,3,4,5,100,190000,190001,(1<<16),(1<<18)-1,(1<<18),(1<<18)+1,(1<<31)-1,(1<<31),(1<<32)-3,(1<<32)-2,(1<<32)-1,(1<<32),(1<<32)+1,(1<<32)+190000,(1<<33),(1<<36)-1,(1<<36),(1<<36)+1,(1<<40),(1<<46)-1,(1<<46),(1<<46)+5})
gaps_ends = [e for e in edges if e <= M]
bad = {"old": 0, "new": 0}; total = 0; first_bad = {}
for length in (NODE, 2 * NODE):
    for S in edges:
        off = S << SLOT_SHIFT
        for w9 in gaps_ends:
            for w10 in gaps_ends:
                if w9 > w10: continue
                total += 1
                t = truth(S, length, w9, w10)
                for name, g in (("old", guard_old), ("new", guard_new)):
                    if g(off, length, w9, w10) != t:
                        bad[name] += 1
                        first_bad.setdefault(name, (S, length, w9, w10, t))
print("cases", total, "disagree-with-truth", bad)
print("first old disagreement:", first_bad.get("old"))
print("first new disagreement:", first_bad.get("new"))
# 非对齐起点：低 14 位非零一律不认
assert guard_new(5, NODE, 0, 10) is None and guard_old(5, NODE, 0, 10) is None
print("misaligned rejected by both")

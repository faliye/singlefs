# L11 的独立再算：K1 对 T ∩ T_exact ∩ C 的跨事务漏。只用定义，不读 Rust 装置。一盘几何（G1 u=2、G3 u=1），P3 自由与 P3 有序。
# T      = 每条发出流 σ 的枚举域 Dom4(σ)（按设备关段，前面各段全持久 + 当前段任意组合，系统配置槽写取三态）的并；
# T_exact= 每条 σ 的偏序（屏障之前的写先于屏障之后发出的写，FUA 写先于它之后的每次写，传递闭包）的全部下闭集，
#          原地覆写（S*）在下闭集里是极大元时可取撕裂态；
# C      = A、B 两事务各至少一次写不是「没持久」的状态；
# 每条 σ 的 L11 = |(T ∩ T_exact ∩ C) \ Dom4(σ)|。
import itertools

def chain(tag, units):
    ev = []
    for u in range(1, units + 1):
        ev.append(('w', f'{tag.lower()}{u}'))
    ev += [('b',), ('w', f'r{tag}'), ('b',), ('f', f'R{tag}'), ('w', f'S{tag}'), ('b',)]
    return ev

def names_of(ev):
    return [e[1] for e in ev if e[0] != 'b']

def segments(stream):
    segs, cur = [], []
    for e in stream:
        if e[0] == 'w':
            cur.append(e[1])
        else:
            if e[0] == 'f':
                cur.append(e[1])
            if cur:
                segs.append(cur)
                cur = []
    if cur:
        segs.append(cur)
    return segs

def inplace(name):
    return name.startswith('S')

def dom4(stream, allnames):
    pos = {n: i for i, n in enumerate(allnames)}
    out = {tuple(0 for _ in allnames)}
    prefix = [0] * len(allnames)
    for seg in segments(stream):
        choices = [(0, 1, 2) if inplace(n) else (0, 1) for n in seg]
        for combo in itertools.product(*choices):
            v = list(prefix)
            for n, c in zip(seg, combo):
                v[pos[n]] = c
            out.add(tuple(v))
        for n in seg:
            prefix[pos[n]] = 1
    return out

def order_of(stream, allnames):
    # 一盘：任何屏障放行此前发出的全部写；FUA 写先于它之后发出的每次写
    pos = {n: i for i, n in enumerate(allnames)}
    before = {n: set() for n in allnames}
    issued = []  # (name, armed, is_fua)
    for e in stream:
        if e[0] == 'b':
            issued = [(n, True, f) for (n, _, f) in issued]
        else:
            name = e[1]
            for (n, armed, f) in issued:
                if armed or f:
                    before[name].add(n)
            issued.append((name, False, e[0] == 'f'))
    changed = True
    while changed:
        changed = False
        for n in allnames:
            ext = set(before[n])
            for m in before[n]:
                ext |= before[m]
            if ext != before[n]:
                before[n] = ext
                changed = True
    return before

def exact_states(stream, allnames):
    before = order_of(stream, allnames)
    pos = {n: i for i, n in enumerate(allnames)}
    out = set()
    for members in itertools.product((0, 1), repeat=len(allnames)):
        chosen = {n for n, m in zip(allnames, members) if m}
        if any(before[n] - chosen for n in chosen):
            continue
        maximal = [n for n in chosen if inplace(n) and not any(n in before[o] for o in chosen)]
        base = [1 if n in chosen else 0 for n in allnames]
        for torn in itertools.product((0, 1), repeat=len(maximal)):
            v = list(base)
            for n, t in zip(maximal, torn):
                if t:
                    v[pos[n]] = 2
            out.add(tuple(v))
    return out

def interleavings(a, b, ordered):
    res = []
    def rec(i, j, cur):
        if i == len(a) and j == len(b):
            res.append(list(cur))
            return
        if i < len(a):
            cur.append(a[i]); rec(i + 1, j, cur); cur.pop()
        if j < len(b):
            ev = b[j]
            ok = True
            if ordered and ev[0] != 'b' and ev[1] in ('rB', 'RB', 'SB'):
                need = ev[1].replace('B', 'A')
                ok = any(e[0] != 'b' and e[1] == need for e in a[:i])
            if ok:
                cur.append(ev); rec(i, j + 1, cur); cur.pop()
    rec(0, 0, [])
    return res

def run(units):
    A, B = chain('A', units), chain('B', units)
    allnames = names_of(A) + names_of(B)
    idx = {n: i for i, n in enumerate(allnames)}
    ANAMES, BNAMES = names_of(A), names_of(B)
    cross = lambda v: any(v[idx[n]] for n in ANAMES) and any(v[idx[n]] for n in BNAMES)
    print(f'--- units={units}')
    for label, ordered in (('P3free', False), ('P3ord', True)):
        sigmas = interleavings(A, B, ordered)
        doms = [dom4(s, allnames) for s in sigmas]
        T = set().union(*doms)
        E = set()
        distinct_orders = set()
        for s in sigmas:
            before = order_of(s, allnames)
            key = tuple(sorted((n, tuple(sorted(before[n]))) for n in allnames))
            if key in distinct_orders:
                continue
            distinct_orders.add(key)
            E |= exact_states(s, allnames)
        target = {t for t in T if cross(t) and t in E}
        miss = [len(target - d) for d in doms]
        print(f'{label} sigma {len(sigmas)} |T| {len(T)} |T_exact| {len(E)} |T∩C| {len({t for t in T if cross(t)})} |T∩T_exact∩C| {len(target)}')
        print(f'  L11 min {min(miss)} max {max(miss)} positive {sum(1 for m in miss if m > 0)} zero {sum(1 for m in miss if m == 0)}')
        ser = dom4(A + B, allnames)
        print(f'  L11 on serial stream {len(target - ser)}')

run(2)
run(1)

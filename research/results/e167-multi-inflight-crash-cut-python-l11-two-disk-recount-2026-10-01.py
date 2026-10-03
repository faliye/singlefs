# L11 的独立再算（两盘小几何）：只按登记定义写，不读 Rust 装置的算法部分。
# 用法：python3 two_disk_l11.py <units> <txg_of_a_mod3> <free|ordered> [进程数]
# 几何：两盘、每个事务 units 个单元（每个单元两盘各一份）、记录两盘各一份、根槽 FUA 落一块盘、系统配置槽写两盘各一份（三态）。
# 根盘：txg 对 3 取余落 0 / 1 / 0；A 的 txg 取余 = 参数，B 的 = 参数 + 1。
import itertools, sys
from multiprocessing import Pool

units, txg_a, mode = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
procs = int(sys.argv[4]) if len(sys.argv) > 4 else 4
ordered = mode == 'ordered'
ROOT_DISK = [0, 1, 0]
root_a, root_b = ROOT_DISK[txg_a % 3], ROOT_DISK[(txg_a + 1) % 3]

def chain(tag, root_disk):
    ev = []  # (类型 w/f/b, 名字, 盘)
    for u in range(1, units + 1):
        for d in (0, 1):
            ev.append(('w', f'{tag.lower()}{u}@{d}', d))
    ev += [('b', None, 0), ('b', None, 1)]
    for d in (0, 1):
        ev.append(('w', f'r{tag}@{d}', d))
    ev += [('b', None, 0), ('b', None, 1)]
    ev.append(('f', f'R{tag}@{root_disk}', root_disk))
    for d in (0, 1):
        ev.append(('w', f'S{tag}@{d}', d))
    ev += [('b', None, 0), ('b', None, 1)]
    return ev

A, B = chain('A', root_a), chain('B', root_b)
names = [e[1] for e in A + B if e[0] != 'b']
pos = {n: i for i, n in enumerate(names)}
inplace = lambda n: n.startswith('S')
A_names = [e[1] for e in A if e[0] != 'b']
B_names = [e[1] for e in B if e[0] != 'b']

def need_of(ev):
    # 有序约束：B 的记录、根、系统配置槽写之前，A 里对应的写已发出（记录与系统配置槽按盘对，根只按种类）
    kind = ev[1][0]
    if kind in 'rS':
        return ev[1].replace('B', 'A', 1)
    if kind == 'R':
        return [e[1] for e in A if e[0] == 'f'][0]
    return None

def segments_and_order(stream):
    segs, cur, nwrites, unreleased = [], [], 0, 0
    issued = []  # [名字, 盘, 已被屏障钉住, 是否 FUA]
    before = {}
    for kind, name, disk in stream:
        bit = 1 << disk
        if kind == 'b':
            unreleased &= ~bit
            for entry in issued:
                if entry[1] == disk:
                    entry[2] = True
            may_close = True
        else:
            cur.append(name)
            nwrites += 1
            before[name] = frozenset(e[0] for e in issued if e[2] or e[3])
            issued.append([name, disk, False, kind == 'f'])
            if kind == 'f':
                unreleased &= ~bit
                may_close = True
            else:
                unreleased |= bit
                may_close = False
        if may_close and nwrites > 0 and unreleased == 0:
            segs.append(tuple(cur))
            cur, nwrites = [], 0
    if nwrites > 0:
        segs.append(tuple(cur))
    return tuple(segs), tuple(sorted(before.items()))

def walk(prefix_path, i, j, cur, out_sigs, out_orders):
    stack = [(i, j, cur, prefix_path)]
    # 递归写成显式栈会改变访问次序；这里用递归，深度 ≤ 与链长之和
    def rec(i, j, cur):
        if i == len(A) and j == len(B):
            sig, order = segments_and_order(cur)
            out_sigs[sig] = out_sigs.get(sig, 0) + 1
            out_orders.add(order)
            return
        if i < len(A):
            cur.append(A[i]); rec(i + 1, j, cur); cur.pop()
        if j < len(B):
            ev = B[j]
            ok = True
            if ordered and ev[0] != 'b':
                need = need_of(ev)
                if need is not None:
                    ok = any(e[0] != 'b' and e[1] == need for e in A[:i])
            if ok:
                cur.append(ev); rec(i, j + 1, cur); cur.pop()
    rec(i, j, cur)

def prefixes(depth):
    # 前 depth 步的全部合法走法
    res = []
    def rec(i, j, cur, d):
        if d == depth:
            res.append((i, j, list(cur)))
            return
        if i < len(A):
            cur.append(A[i]); rec(i + 1, j, cur, d + 1); cur.pop()
        if j < len(B):
            ev = B[j]
            ok = True
            if ordered and ev[0] != 'b':
                need = need_of(ev)
                if need is not None:
                    ok = any(e[0] != 'b' and e[1] == need for e in A[:i])
            if ok:
                cur.append(ev); rec(i, j + 1, cur, d + 1); cur.pop()
    rec(0, 0, [], 0)
    return res

def task(item):
    i, j, cur = item
    sigs, orders = {}, set()
    walk(None, i, j, cur, sigs, orders)
    return sigs, orders

def dom4_of_segments(segs):
    out = {tuple(0 for _ in names)}
    prefix = [0] * len(names)
    for seg in segs:
        choices = [(0, 1, 2) if inplace(n) else (0, 1) for n in seg]
        for combo in itertools.product(*choices):
            v = list(prefix)
            for n, c in zip(seg, combo):
                v[pos[n]] = c
            out.add(tuple(v))
        for n in seg:
            prefix[pos[n]] = 1
    return out

def close(before_items):
    before = {n: set(b) for n, b in before_items}
    changed = True
    while changed:
        changed = False
        for n in before:
            ext = set(before[n])
            for m in before[n]:
                ext |= before[m]
            if ext != before[n]:
                before[n] = ext
                changed = True
    return before

def exact_of(before):
    out = set()
    for members in itertools.product((0, 1), repeat=len(names)):
        chosen = {n for n, m in zip(names, members) if m}
        if any(before[n] - chosen for n in chosen):
            continue
        maximal = [n for n in chosen if inplace(n) and not any(n in before[o] for o in chosen)]
        base = [1 if n in chosen else 0 for n in names]
        for torn in itertools.product((0, 1), repeat=len(maximal)):
            v = list(base)
            for n, t in zip(maximal, torn):
                if t:
                    v[pos[n]] = 2
            out.add(tuple(v))
    return out

if __name__ == '__main__':
    items = prefixes(8)
    sigs, orders = {}, set()
    with Pool(procs) as pool:
        for s, o in pool.imap_unordered(task, items):
            for k, v in s.items():
                sigs[k] = sigs.get(k, 0) + v
            orders |= o
    leaves = sum(sigs.values())
    print(f'units={units} txg_a%3={txg_a} mode={mode} root_a={root_a} root_b={root_b}')
    print(f'interleavings {leaves} distinct_segment_signatures {len(sigs)} distinct_orders {len(orders)}')
    doms = {s: dom4_of_segments(s) for s in sigs}
    T = set().union(*doms.values())
    E = set()
    for o in orders:
        E |= exact_of(close(o))
    cross = lambda v: any(v[pos[n]] for n in A_names) and any(v[pos[n]] for n in B_names)
    Tc = {t for t in T if cross(t)}
    target = {t for t in Tc if t in E}
    print(f'|T| {len(T)} |T_exact| {len(E)} |T∩T_exact| {len(T & E)} |T∩C| {len(Tc)} |T∩T_exact∩C| {len(target)}')
    miss = {s: len(target - d) for s, d in doms.items()}
    l2 = {s: len(Tc - d) for s, d in doms.items()}
    mn, mx = min(miss.values()), max(miss.values())
    pos_streams = sum(sigs[s] for s in sigs if miss[s] > 0)
    print(f'L11 min {mn} max {mx} streams_with_misses {pos_streams} streams_without {leaves - pos_streams} signatures_with_misses {sum(1 for s in sigs if miss[s] > 0)}')
    print(f'L2  min {min(l2.values())} max {max(l2.values())} streams_with_misses {sum(sigs[s] for s in sigs if l2[s] > 0)}')
    ser = segments_and_order(A + B)[0]
    print(f'L11 on serial stream {len(target - dom4_of_segments(ser))}')

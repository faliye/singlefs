"""Q3（C937）：一块盘上的小世界穷举。推的模型，照 crates/singlefs-checker/src/walk.rs 的
allocated_statistic_bounds_of_a_device（10279）、allocated_statistic_lies_within_its_bounds（10336）、
records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes（10360，改记条件 10372 的 `!=`；变体 `>` 只改窄的）、
allocated_statistic_lies_within_the_bounds_of_either_span_reading（10391）照搬；I-3.18、I-5.4、I-3.11 按条文判。
θ_high = t（I-7.18 成立的镜像），没有隔离豁免。
世界：槽 0..n-1；单元 (起点, 类槽数 1|2, 被谁引用 new=最新根 / old=只被更早的候选根引用、最新根里已释放)；单元区间互不相交（相交的 I-5.1 / I-2.1 已红，不进）。
记录：new 单元：无 | 未释放、跨 k；old 单元：无 | 已释放、释放代高于门槛（在 defer）、跨 k；另加至多一条游离的已释放记录（高于或低于门槛）。
未释放记录只起在 new 单元的起点（否则 I-3.15 已红）。记账 A 取 0..n+2；defer 行 D 两种口径：任取（I-3.11 能绿就绿）/ 写者自洽（=已释放高于门槛的记录罩住的槽数）。
「说全」：存在一种逐条读法（I-3.18 判红的每条各取自述或单元那一类），在它下面 (L)(U) 成立，且 (P) 不成立的槽都落在某条按自述读、比单元窄的
判红记录自己的单元里（那几槽正是 I-3.18 条文说的「单元后面那几槽在盘上没有归属」）。取宽成立而说不全、I-5.4 不红、I-3.11 不红 = 打中。"""
import itertools, sys
from collections import Counter


def cover(records, theta_high=True):
    s = set()
    for (st, sp, released, high) in records:
        if (not released) or high:
            s.update(range(st, st + sp))
    return s


def bounds(records, omega):
    E = cover(records)
    U = {x for (st, sp, rel, high) in records if rel and high for x in range(st, st + sp)} - omega
    return len(E), E, len(omega) + len(U)


def three(A, records, omega):
    nE, E, om_u = bounds(records, omega)
    return A >= nE and omega <= E and A <= min(nE, om_u)


def widened(A, rec, cls, omega):
    nr, Er, omu = bounds(rec, omega)
    nc, Ec, _ = bounds(cls, omega)
    P = all((x in Er) or (x in Ec) for x in omega - Er)
    return A >= min(nr, nc) and P and A <= min(max(nr, nc), omu)


def explained(A, records, red_idx, unit_class_at, omega):
    for choice in itertools.product((False, True), repeat=len(red_idx)):
        recs = list(records)
        excused = set()
        for flip, i in zip(choice, red_idx):
            st, sp, rel, high = recs[i]
            cl = unit_class_at[st]
            if flip:
                recs[i] = (st, cl, rel, high)
            elif sp < cl:
                excused.update(range(st, st + cl))
        nE, E, omu = bounds(recs, omega)
        if A >= nE and A <= min(nE, omu) and (omega - E) <= excused:
            return True
    return False


def worlds(n):
    for k in range(1, 4):
        for starts in itertools.combinations(range(n), k):
            for classes in itertools.product((1, 2), repeat=k):
                ext = [(s, c) for s, c in zip(starts, classes)]
                if any(s + c > n for s, c in ext):
                    continue
                if any(ext[i][0] + ext[i][1] > ext[i + 1][0] for i in range(k - 1)):
                    continue
                for refs in itertools.product(('new', 'old'), repeat=k):
                    yield ext, refs


def run(n, rewrite_rule):
    t = Counter(); examples = {}
    for ext, refs in worlds(n):
        unit_class_at = {s: c for s, c in ext}
        omega = {x for s, c in ext for x in range(s, s + c)}
        walked_new = {x for (s, c), rf in zip(ext, refs) if rf == 'new' for x in range(s, s + c)}
        opts = []
        for (s, c), rf in zip(ext, refs):
            if rf == 'new':
                opts.append([None] + [(s, k, False, False) for k in (1, 2, 3) if s + k <= n])
            else:
                opts.append([None] + [(s, k, True, True) for k in (1, 2, 3) if s + k <= n])
        extras = [None] + [(s, k, True, high) for s in range(n) for k in (1, 2) if s + k <= n for high in (True, False)]
        for chosen in itertools.product(*opts):
            for extra in extras:
                records = [r for r in chosen if r is not None] + ([extra] if extra else [])
                i54_red = any(set(range(a[0], a[0] + a[1])) & set(range(b[0], b[0] + b[1]))
                              for a, b in itertools.combinations(records, 2))
                red_idx = [i for i, r in enumerate(records) if not r[2] and r[0] in unit_class_at and r[1] != unit_class_at[r[0]]]
                if not red_idx:
                    continue
                dirs = {('narrow' if records[i][1] < unit_class_at[records[i][0]] else 'wide') for i in red_idx}
                direction = dirs.pop() if len(dirs) == 1 else 'mixed'
                def rewritten(i):
                    st, sp = records[i][0], records[i][1]
                    cl = unit_class_at[st]
                    if rewrite_rule == '!=':
                        return sp != cl
                    if rewrite_rule == '>':
                        return cl > sp
                    # '>|unwalked'（本腿提的收严，零轮）：窄的照改；宽的只在多罩的那几槽遍历一个都没走到时改
                    return cl > sp or not (set(range(st + cl, st + sp)) & omega)
                rw = [i for i in red_idx if rewritten(i)]
                cls = list(records)
                for i in rw:
                    cls[i] = (records[i][0], unit_class_at[records[i][0]], False, False)
                d_writer = len({x for (st, sp, rel, high) in records if rel and high for x in range(st, st + sp)})
                for A in range(0, n + 3):
                    if three(A, records, omega):
                        continue
                    t[(direction, 'rec_fails')] += 1
                    if not rw or not widened(A, records, cls, omega):
                        continue
                    t[(direction, 'widened_holds')] += 1
                    if explained(A, records, red_idx, unit_class_at, omega):
                        continue
                    i311_free = A >= len(walked_new)
                    i311_writer = A - d_writer == len(walked_new)
                    key = (direction, 'hit', 'i54_red' if i54_red else 'i54_green',
                           'i311_green_free_D' if i311_free else 'i311_red', 'i311_green_writer_D' if i311_writer else 'i311_writer_D_red')
                    t[key] += 1
                    if key not in examples or len(records) < len(examples[key][2]):
                        examples[key] = (ext, refs, records, A, d_writer)
    return t, examples


if __name__ == '__main__':
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 5
    for rule in ('!=', '>', '>|unwalked'):
        t, ex = run(n, rule)
        for k in sorted(t, key=str):
            print(f"name=q3 n={n} rewrite={rule} " + '/'.join(k) + f"={t[k]}")
        for k in sorted(ex, key=str):
            ext, refs, records, A, dw = ex[k]
            print(f"  example rewrite={rule} {'/'.join(k)} units={ext} refs={refs} records(start,span,released,gen_above)={records} A={A} D_writer={dw}")
        sys.stdout.flush()

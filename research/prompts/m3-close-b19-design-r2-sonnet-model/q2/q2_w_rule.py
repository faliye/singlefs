"""Q2（C740）小模型：实例表行 (i, T, W) 的 W 判法，在「挂载读故障」下谁误红、谁丢判别力。
一个旧实例发 txg 1..n 共 n 次发布（每次一条记录，事务号按 I-8.7 取），txg 0 是第 0 代根；
根槽落盘集合 L（真实形：1..n-1 全落，n 落或不落；宽形：任意子集）。
新实例挂载时 faults（读坏、挂载内不恢复的根槽集合）里的根读不出：所选根 c = 读得出的最大 txg；重放施加 (c, n] 的全部记录，
T = n，W = 施加的记录里最大事务号（没施加取 0）。镜像（判器看到的）里没有故障：L 全读得出，行 (T, W) 照写。
规则：
  forward  = 正推腿三支：T 在 L ⇒ W 必 0；T 不在 L ⇒ 从 L 里 txg 小于 T 的最大根 S 起重放比 W；
  set_rule = 攻方给的收窄：W ∈ {0（T 在 L 时）} ∪ {对每个 L 里 txg<T 的 c'：(c', T] 里最大事务号}；
  ideal    = 只看镜像能做到的最强：W ∈ 该镜像在 ≤kmax 个读故障下可达的 W 集合。
推的模型，不是入库装置；不模型环转圈、实例切换、F2 退回。"""
import itertools, sys

def txn_sequences(n, values=(0, 1, 2, 3)):
    """I-8.7：非 0 事务号第一个是 1，相邻非 0 不减；同号连续（被别的记录含 0 隔开后不再出现）；每次最多加 1。"""
    out = []
    for seq in itertools.product(values, repeat=n):
        nz = [v for v in seq if v]
        if nz and nz[0] != 1:
            continue
        ok = True
        last = 0
        closed = set()
        prev = None
        for v in seq:
            if v:
                if v < last or v > last + 1 and last != 0 or (last == 0 and v != 1):
                    ok = False; break
                if v in closed:
                    ok = False; break
                last = v
            if prev is not None and prev != v and prev:
                closed.add(prev)
            prev = v
        if ok:
            out.append(seq)
    return out

def max_txn(seq, lo, hi):
    """记录 txg 在 (lo, hi]：seq[txg-1]"""
    m = 0
    for t in range(lo + 1, hi + 1):
        m = max(m, seq[t - 1])
    return m

def mount_row(seq, n, L, faults):
    readable = [t for t in L if t not in faults]
    if not readable:
        return None
    c = max(readable)
    return (n, max_txn(seq, c, n) if c < n else 0)

def forward_ok(seq, n, L, row):
    T, W = row
    if T in L:
        return W == 0
    S = max(t for t in L if t < T)
    return W == max_txn(seq, S, T)

def set_allowed(seq, n, L, T):
    s = set()
    if T in L:
        s.add(0)
    for c in L:
        if c < T:
            s.add(max_txn(seq, c, T))
    return s

def run(mode, kmax, nmax):
    stats = dict(worlds=0, histories=0, images=0, fwd_false_red_hist=0, fwd_false_red_a=0, fwd_false_red_b=0,
                 set_false_red_hist=0, mut_total=0, mut_ideal_red=0, mut_fwd_red=0, mut_set_red=0)
    examples = []
    for n in range(1, nmax + 1):
        for seq in txn_sequences(n):
            if mode == "real":
                Ls = [frozenset(range(0, n + 1)), frozenset(range(0, n))]
            else:
                Ls = [frozenset([0] + [t for t in range(1, n + 1) if (mask >> (t - 1)) & 1]) for mask in range(1 << n)]
            for L in Ls:
                stats["worlds"] += 1
                # 该镜像可达的 (T=n 恒定) W 集合
                reach = {}
                for k in range(0, kmax + 1):
                    for faults in itertools.combinations(sorted(L), k):
                        row = mount_row(seq, n, L, set(faults))
                        if row is None:
                            continue
                        stats["histories"] += 1
                        reach.setdefault(row, faults)
                        if not forward_ok(seq, n, L, row):
                            stats["fwd_false_red_hist"] += 1
                            if n in L:
                                stats["fwd_false_red_a"] += 1
                            else:
                                stats["fwd_false_red_b"] += 1
                            if len(examples) < 4:
                                examples.append((n, seq, sorted(L), faults, row))
                        if row[1] not in set_allowed(seq, n, L, n):
                            stats["set_false_red_hist"] += 1
                stats["images"] += 1
                allowed_ideal = {r[1] for r in reach}
                # 变异：把行的 W 改成 0、W+1、W-1（≥0）、一个别的值
                for w_bad in {0, 1, 2, 3, 4}:
                    if w_bad in allowed_ideal:
                        continue
                    stats["mut_total"] += 1
                    stats["mut_ideal_red"] += 1
                    row = (n, w_bad)
                    if not forward_ok(seq, n, L, row):
                        stats["mut_fwd_red"] += 1
                    if w_bad not in set_allowed(seq, n, L, n):
                        stats["mut_set_red"] += 1
    return stats, examples

if __name__ == "__main__":
    nmax = int(sys.argv[1]) if len(sys.argv) > 1 else 6
    for mode in ("real", "wide"):
        for kmax in (1, 2):
            stats, examples = run(mode, kmax, nmax if mode == "real" else min(nmax, 5))
            print(f"mode={mode} nmax={nmax if mode == 'real' else min(nmax, 5)} kmax={kmax} " + " ".join(f"{k}={v}" for k, v in stats.items()))
            for e in examples[:2]:
                print("  example_false_red_of_forward (n, txn_seq, L, faults, row):", e)

def b35_shape(nmax=6):
    """C740 判别力自证那一形：T 的根槽没落（T 不在 L），T 这条记录事务号非 0，把 W 写成 0：各规则红不红。"""
    total = fwd = setr = ideal = 0
    for n in range(1, nmax + 1):
        for seq in txn_sequences(n):
            if seq[n - 1] == 0:
                continue
            L = frozenset(range(0, n))
            reach = set()
            for k in (0, 1):
                for faults in itertools.combinations(sorted(L), k):
                    row = mount_row(seq, n, L, set(faults))
                    if row:
                        reach.add(row[1])
            total += 1
            if 0 not in reach:
                ideal += 1
            if not forward_ok(seq, n, L, (n, 0)):
                fwd += 1
            if 0 not in set_allowed(seq, n, L, n):
                setr += 1
    return total, ideal, fwd, setr

if __name__ == "__main__" and len(sys.argv) > 2:
    print("b35_shape (worlds, ideal_red, forward_red, set_rule_red) =", b35_shape())

def k1_set_allowed(seq, n, L, T):
    """只许一个根槽读故障的收窄：T 在 L 时含 0 再加「L 里 txg<T 的最大一条根」的 (c,T] 最大事务号；T 不在 L 时加「最大两条」各自的"""
    s = set()
    if T in L:
        s.add(0)
    below = sorted(t for t in L if t < T)
    for c in (below[-1:] if T in L else below[-2:]):
        s.add(max_txn(seq, c, T))
    return s

def k1_rule(nmax=6):
    """k1 规则在 kmax=1 / 2 的真实形世界上：误红的 (世界,故障集) 数、与变异（W 改成不可达值）红的数"""
    out = {}
    for kmax in (1, 2):
        hist = false_red = mut_total = mut_red = 0
        for n in range(1, nmax + 1):
            for seq in txn_sequences(n):
                for L in (frozenset(range(0, n + 1)), frozenset(range(0, n))):
                    reach = set()
                    for k in range(0, kmax + 1):
                        for faults in itertools.combinations(sorted(L), k):
                            row = mount_row(seq, n, L, set(faults))
                            if row is None:
                                continue
                            hist += 1
                            reach.add(row[1])
                            if row[1] not in k1_set_allowed(seq, n, L, n):
                                false_red += 1
                    for w_bad in range(0, 5):
                        if w_bad in reach:
                            continue
                        mut_total += 1
                        if w_bad not in k1_set_allowed(seq, n, L, n):
                            mut_red += 1
        out[kmax] = (hist, false_red, mut_total, mut_red)
    return out

if __name__ == "__main__" and len(sys.argv) > 2:
    print("k1_set_rule {kmax: (histories, false_red, mutants_not_reachable_at_that_kmax, mutants_red)} =", k1_rule())

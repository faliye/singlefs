"""Q7（C717）两条路的代价模型：推的模型，不是入库装置。
根环 R 区域 x S 槽、txg、实例、F、树状态号（代表两棵树根指针）、实例表行、journal 环（只留最近 J 条记录）。
写者在抬 F 那一刻用整环算上限（照第一轮 k8_window_model.py 的 ceiling，D16 已定项 1 的准入式子）。
每次发布都写记录（带文件的版本，D16 已定项 9 只有树表 0 条才零单元；模型不模型树表 0 条那一段）。
每条根带回收门槛 t = max(F, 记这条根之后环里最旧有效根)（I-7.18 的写者取法，模型里没有读故障、没有钉）。
在最后那张镜像上，对每条抬 F 的根按五种判法判：
  window   现行 checker：锚在镜像最新根上的一圈窗口
  yi       乙（正推腿第一轮 K8 原文）：O = 前一条同实例根的 t；窗口红照红；F <= O 判成立；其余报部分没判
  yi_tight 乙的收紧（本腿提的，零轮）：窗口里 txg < r 的根每块盘都有、且窗口里看得见前一条根的换状态有 >= 4 个不同状态时窗口上限就是整环上限，照窗口判；
           否则照 yi
  ding     丁：按镜像根、journal 里还在的记录与实例表行重建抬 F 那一刻的根环；有歧义（行的 T 那一槽已被盖、根落没落不知道）
           两种都算，结论一致才判；要用的根既不在镜像也不在 journal 里报部分没判
真值：抬 F 那一刻整环上限下 F_new 合不合法（写者在模拟里记的）。"""
import random, sys
from collections import deque

R = 3

def slot_of(txg, S):
    return (txg % R, (txg // R) % S)

def ceiling(roots_before, devices_of_region):
    """同第一轮 k8_window_model.ceiling：roots_before = [(txg, inst, F, state)]，已滤好。"""
    if not roots_before:
        return None
    rs = sorted(roots_before)
    newest_per_device = {}
    for (t, i, f, st) in rs:
        d = devices_of_region[t % R]
        newest_per_device[d] = max(newest_per_device.get(d, -1), t)
    newest_on_every_device = min(newest_per_device.values())
    prev = None
    non_empty = []
    for (t, i, f, st) in rs:
        if st != prev:
            non_empty.append((t, st))
        prev = st
    counted, seen = [], []
    for (t, st) in sorted(non_empty, reverse=True):
        if st not in seen:
            seen.append(st); counted.append(t)
    fourth = counted[3] if len(counted) >= 4 else rs[0][0]
    return min(newest_on_every_device, fourth)

def distinct_non_empty(roots):
    """只数看得见前一条根的那几次换状态：最旧那条根算不算「非空」要看它前面那条，前面那条不在窗口里就不数。"""
    rs = sorted(roots)
    seen = set()
    for (a, b) in zip(rs, rs[1:]):
        if b[3] != a[3]:
            seen.add(b[3])
    return len(seen)

def abandoned(txg, inst, rows):
    return inst in rows and txg > rows[inst]

class History:
    def __init__(self, seed, S, J, devs, steps, p_nonempty, p_lost_root, p_clean_crash, p_raise, writer):
        rng = random.Random(seed)
        self.S, self.J, self.devs = S, J, devs
        self.versions = {}      # txg -> dict(inst, F, state, landed)
        self.ring = {}          # slot -> txg（盘上此刻）
        self.rows = {}          # inst -> T
        self.journal = deque(maxlen=J)  # 记录的 txg（计数器次序）
        self.t_of = {}          # 根 txg -> 它带的 t
        self.raises = {}        # 抬 F 的根 txg -> (F_new, F_old, 整环上限, 合法?)
        inst, F, state, nxt, txg = 1, 0, 0, 1, 0

        def land(txg_, inst_, F_, state_):
            self.versions[txg_] = dict(inst=inst_, F=F_, state=state_, landed=True)
            self.journal.append(txg_)
            self.ring[slot_of(txg_, S)] = txg_
            valid = [v for v in self.ring.values() if not abandoned(v, self.versions[v]['inst'], self.rows)]
            self.t_of[txg_] = max(F_, min(valid))

        for _ in range(steps):
            r = rng.random()
            if r < p_lost_root:
                txg += 1
                if rng.random() < p_nonempty:
                    state = nxt; nxt += 1
                self.versions[txg] = dict(inst=inst, F=F, state=state, landed=False)
                self.journal.append(txg)          # 记录落了、根槽没落
                self.rows[inst] = txg
                inst += 1
                txg += 1
                land(txg, inst, F, state)          # 新实例写行那条根
                continue
            r -= p_lost_root
            if r < p_clean_crash and txg > 0:
                self.rows[inst] = txg
                inst += 1
                txg += 1
                land(txg, inst, F, state)
                continue
            r -= p_clean_crash
            if r < p_raise and txg > 0:
                before = [v for v in self.ring.values()
                          if v >= F and not abandoned(v, self.versions[v]['inst'], self.rows)]
                c = ceiling([(v, self.versions[v]['inst'], self.versions[v]['F'], self.versions[v]['state'])
                             for v in before], devs)
                if c is not None and c > F:
                    if writer == 'admission':
                        new_f = c
                    elif writer == 'uniform':
                        new_f = rng.randint(F + 1, c)
                    else:  # bad：越过上限 1..3，不越过现行 txg
                        new_f = min(c + rng.randint(1, 3), txg)
                        if new_f <= c:
                            new_f = c
                    txg += 1
                    self.raises[txg] = (new_f, F, c, new_f <= c)
                    F = new_f
                    land(txg, inst, F, state)
                    continue
            txg += 1
            if rng.random() < p_nonempty:
                state = nxt; nxt += 1
            land(txg, inst, F, state)
        self.N = max(self.ring.values())

    def image_roots(self):
        return {v: self.versions[v] for v in self.ring.values()}

def tup(txg, v):
    return (txg, v['inst'], v['F'], v['state'])

def judge_all(h):
    """对镜像上每条抬 F 的根：返回 [(r, truth_legal, window, yi, yi_tight, ding, c717_condition)]。"""
    S, devs, rows = h.S, h.devs, h.rows
    img = h.image_roots()
    N = h.N
    journal = set(h.journal)
    out = []
    for r in sorted(img):
        if r not in h.raises:
            continue
        vr = img[r]
        prev = [x for x in img if img[x]['inst'] == vr['inst'] and x < r]
        if not prev:
            continue
        p = max(prev)
        fold = img[p]['F']
        f = vr['F']
        if f <= fold:
            continue
        legal = h.raises[r][3]
        cand = [tup(x, img[x]) for x in img if x < r and x >= fold and not abandoned(x, img[x]['inst'], rows)]
        win = [c for c in cand if c[0] > N - 3 * S]
        wc = ceiling(win, devs)
        window = 'none' if wc is None else ('red' if f > wc else 'holds')
        # c717 条件：洞里（窗口外）有有效根 X，F 落在 (X, W]
        hole = [c for c in cand if c[0] <= N - 3 * S]
        c717 = bool(win and hole and min(c[0] for c in hole) < f <= (wc if wc is not None else -1))
        # 乙
        O = h.t_of[p]
        if window == 'red':
            yi = 'red'
        elif f <= O:
            yi = 'holds'
        else:
            yi = 'partial'
        # 乙收紧
        devices_seen = {devs[c[0] % R] for c in win}
        complete = devices_seen == set(devs) and distinct_non_empty(win) >= 4
        if window == 'red':
            yi_tight = 'red'
        elif complete:
            yi_tight = window
        else:
            yi_tight = yi
        ding = judge_ding(h, r, fold, f, img, journal)
        out.append((r, legal, window, yi, yi_tight, ding, c717))
    return out

def judge_ding(h, r, fold, f, img, journal):
    S, devs, rows = h.S, h.devs, h.rows
    # 抬之前的 F 取紧挨着的上一版（r - 1）：镜像里或 journal 里有它才知道；镜像里同实例更早那条根的 F 可能更低，
    # 拿它当抬之前的 F 会把写者没算进去的根算进来（第一次跑出 1 个误红，就是这一处）。
    if r - 1 in img:
        fold = img[r - 1]['F']
    elif r - 1 in journal:
        fold = h.versions[r - 1]['F']
    else:
        return 'partial'
    period = 3 * S
    T_of_rows = set(rows.values())
    per_slot_options = []
    for reg in range(R):
        for s in range(S):
            # 公式落到这一槽、比 r 小的最大 txg
            base = s * R + reg
            if base >= r:
                per_slot_options.append([None]); continue
            u = base + ((r - 1 - base) // period) * period
            options = []
            pending = [u]
            ok = True
            while pending:
                u = pending.pop()
                if u <= 0:
                    options.append(None); continue
                if u in img:
                    options.append(u); continue
                occupant_now = h.ring.get(slot_of(u, S))
                if occupant_now is None or occupant_now < u:
                    pending.append(u - period); continue   # u 的根没落：那一槽此刻还住着更旧的
                # 此刻那一槽住着比 u 新的：u 被盖了，或 u 根本没落而之后被盖
                if u not in journal:
                    ok = False; break                       # 实例、状态都不知道
                if u in T_of_rows and rows.get(h.versions[u]['inst']) == u:
                    options.append(u); pending.append(u - period)   # 行的 T：落没落不知道，两种都算
                else:
                    options.append(u)                       # 实例中间的一版：根一定落了
            if not ok:
                return 'partial'
            per_slot_options.append(options)
    combos = [[]]
    for opts in per_slot_options:
        combos = [c + [o] for c in combos for o in opts]
        if len(combos) > 4096:
            return 'partial'
    verdicts = set()
    for combo in combos:
        roots = []
        for u in combo:
            if u is None:
                continue
            v = h.versions[u]
            if u >= fold and not abandoned(u, v['inst'], rows):
                roots.append(tup(u, v))
        c = ceiling(roots, devs)
        if c is None:
            verdicts.add('none')
        else:
            verdicts.add('red' if f > c else 'holds')
    if len(verdicts) == 1:
        return verdicts.pop()
    return 'partial'

def rand_steps(seed):
    return 20 + (seed * 7919) % 41

def main():
    seeds = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    devs = (0, 1, 0)
    for writer in ('admission', 'uniform', 'bad'):
        for S in (4, 8):
            for J in (8, 32, 128):
                tally = {}
                for seed in range(seeds):
                    h = History(seed, S, J, devs, rand_steps(seed), p_nonempty=0.35, p_lost_root=0.04,
                                p_clean_crash=0.03, p_raise=0.08, writer=writer)
                    for (r, legal, window, yi, yi_tight, ding, c717) in judge_all(h):
                        key = 'legal' if legal else 'illegal'
                        for name, v in (('window', window), ('yi', yi), ('yi_tight', yi_tight), ('ding', ding)):
                            tally[(key, name, v)] = tally.get((key, name, v), 0) + 1
                            if c717:
                                tally[(key + '_c717', name, v)] = tally.get((key + '_c717', name, v), 0) + 1
                        tally[(key, 'count', '')] = tally.get((key, 'count', ''), 0) + 1
                        if c717:
                            tally[(key + '_c717', 'count', '')] = tally.get((key + '_c717', 'count', ''), 0) + 1
                for key in ('legal', 'illegal', 'legal_c717', 'illegal_c717'):
                    n = tally.get((key, 'count', ''), 0)
                    if n == 0:
                        continue
                    parts = []
                    for name in ('window', 'yi', 'yi_tight', 'ding'):
                        vs = ' '.join(f"{v}={tally.get((key, name, v), 0)}" for v in ('holds', 'partial', 'red', 'none'))
                        parts.append(f"{name}[{vs}]")
                    print(f"writer={writer} S={S} J={J} {key}_raise_roots={n} " + ' '.join(parts))
                sys.stdout.flush()

if __name__ == '__main__':
    main()

"""Q2（C934）：丁（按 journal 重建抬 F 那一刻的根环）今天实现里三处保守简化各让「部分没判」多出多少。
推的模型，不是入库装置，不跑 crates。

口径：第二轮攻方模型 research/prompts/m3-close-b19-design-r2-opus-model/q7/q7_cost_model.py 的同一套历史（R=3，devs=(0,1,0)，
rand_steps、p_nonempty=0.35、p_lost_root=0.04、p_clean_crash=0.03、p_raise=0.08、种子 0..N-1、写者 admission 与 bad）。
judge_ding 重写成按槽的版本，对着 crates/singlefs-checker/src/walk.rs 的 root_in_the_ring_slot_at_the_raise（8986 起）：
  简化 A「只认一层，不往 u-3S 推」：放开程度 A = 0 不往下、1 往下一层、2 不限层；
  简化 B「u 是行 T、被抛弃、实例没有行都认不出」：放开 B = 1 时这几种按「两种都算（u 落了 / 没落），结论一致才判」；
          本模型里只出现「u 是行 T」一种（模型没有被抛弃时间线上的记录，也没有缺行的实例），另两种恒为 0，没量；
  简化 C「被盖掉的第 0 代根在抬之前 F = 0 时认不出」：放开 C = 1 时把第 0 代根当成已知的空树根。第二轮模型没有第 0 代根，
          只有 genesis 变体（mkfs 在三个区域的槽 0 各种一条 txg 0 的根）里 C 才有东西可量。
放开 B 而 A = 0（要「两种都算」却不许往下认那一种）等于不放开：本模型里认不出的写法只有往下一层才求得出另一种。
steps_scale：每段历史的步数乘几（1 是第二轮口径，20..60 步；4 是 80..240 步，让环转得更多圈，A 才有东西可量）。
输出每个 (变体, 写者, S, J, 模式) 一行：合法 / 非法抬 F 根的 holds / partial / red / none 计数。
"""
import random, sys
from collections import deque

R = 3
DEVS = (0, 1, 0)
PARTIAL = object()


def slot_of(txg, S):
    return (txg % R, (txg // R) % S)


def ceiling(roots_before):
    """roots_before = [(txg, inst, F, state, region)]，已滤好。同 walk.rs rollback_floor_ceiling_of_the_valid_roots 的上限（状态号已知）。"""
    if not roots_before:
        return None
    rs = sorted(roots_before)
    newest_per_device = {}
    for (t, i, f, st, reg) in rs:
        d = DEVS[reg]
        newest_per_device[d] = max(newest_per_device.get(d, -1), t)
    newest_on_every_device = min(newest_per_device.values())
    prev = None
    non_empty = []
    for (t, i, f, st, reg) in rs:
        if st != prev:
            non_empty.append((t, st))
        prev = st
    counted, seen = [], []
    for (t, st) in sorted(non_empty, reverse=True):
        if st not in seen:
            seen.append(st)
            counted.append(t)
    fourth = counted[3] if len(counted) >= 4 else rs[0][0]
    return min(newest_on_every_device, fourth)


def abandoned(txg, inst, rows):
    return inst in rows and txg > rows[inst]


class History:
    """同 opus 模型的 History；ring 存 槽 -> txg，genesis=True 时开头三个区域的槽 0 各种一条 txg 0 的根（实例 0、F=0、空树状态 0）。"""

    def __init__(self, seed, S, steps, p_nonempty, p_lost_root, p_clean_crash, p_raise, writer, genesis):
        rng = random.Random(seed)
        self.S = S
        self.versions = {}
        self.ring = {}
        self.rows = {}
        self.journal = []
        self.t_of = {}
        self.raises = {}
        self.genesis = genesis
        self.ring_then = {}      # 抬 F 的根 txg -> (写者算上限那一刻环里的根列表, 抬之前的 F)
        self.lost = {}           # 根没落的那一版 txg -> (实例, F, 状态)
        if genesis:
            self.versions[0] = dict(inst=0, F=0, state=0, landed=True)
            for reg in range(R):
                self.ring[(reg, 0)] = 0
        inst, F, state, nxt, txg = 1, 0, 0, 1, 0

        def root_list():
            return [(t, self.versions[t]['inst'], self.versions[t]['F'], self.versions[t]['state'], slot[0])
                    for slot, t in self.ring.items()]

        def land(txg_, inst_, F_, state_):
            self.versions[txg_] = dict(inst=inst_, F=F_, state=state_, landed=True)
            self.journal.append(txg_)
            self.ring[slot_of(txg_, S)] = txg_
            valid = [t for t in self.ring.values() if not abandoned(t, self.versions[t]['inst'], self.rows)]
            self.t_of[txg_] = max(F_, min(valid))

        for _ in range(steps):
            r = rng.random()
            if r < p_lost_root:
                txg += 1
                if rng.random() < p_nonempty:
                    state = nxt
                    nxt += 1
                self.versions[txg] = dict(inst=inst, F=F, state=state, landed=False)
                self.lost[txg] = (inst, F, state)
                self.journal.append(txg)
                self.rows[inst] = txg
                inst += 1
                txg += 1
                land(txg, inst, F, state)
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
                before = [x for x in root_list() if x[0] >= F and not abandoned(x[0], x[1], self.rows)]
                c = ceiling(before)
                if c is not None and c > F:
                    if writer == 'admission':
                        new_f = c
                    else:
                        new_f = min(c + rng.randint(1, 3), txg)
                        if new_f <= c:
                            new_f = c
                    txg += 1
                    self.ring_then[txg] = (list(root_list()), F)
                    self.raises[txg] = (new_f, F, c, new_f <= c)
                    F = new_f
                    land(txg, inst, F, state)
                    continue
            txg += 1
            if rng.random() < p_nonempty:
                state = nxt
                nxt += 1
            land(txg, inst, F, state)
        self.N = max(self.ring.values())
        self.roots = root_list()


def tup(txg, v, reg):
    return (txg, v['inst'], v['F'], v['state'], reg)


def judge(h, r, journal, mode):
    """抬 F 的根 r 的判定：'holds' / 'red' / 'none' / 'partial'。mode = (A, B, C)。对着 walk.rs rollback_floor_against_the_ring_reconstructed_at_the_raise。"""
    A, B, C = mode
    S, rows = h.S, h.rows
    img_by_txg = {t: h.versions[t] for (t, i, f, st, reg) in h.roots if t > 0}
    vr = img_by_txg[r]
    # 抬之前的 F：同 opus 模型，取紧挨着的上一版 r-1（镜像里或 journal 里有它才知道）
    if r - 1 in img_by_txg:
        fold = img_by_txg[r - 1]['F']
    elif r - 1 in journal:
        fold = h.versions[r - 1]['F']
    else:
        return 'partial'
    f = vr['F']
    period = 3 * S
    ring_now = h.ring
    per_slot = []

    def landed_root(u, reg):
        return tup(u, h.versions[u], reg)

    def genesis_case(s, reg):
        # 落点式在这一槽里没有比 r 小的正 txg：这一槽只住过第 0 代根或空着
        if not h.genesis:
            return [None]
        if s != 0 or fold > 0:
            return [None]
        if C:
            return [(0, 0, 0, 0, reg)]    # 放开：第 0 代根当已知的空树根
        return PARTIAL

    def descend(u, s, reg, depth):
        """u 的根没落时那一槽住着谁：往 u-3S 推。depth = 已经往下推了几层。"""
        if u <= 0:
            return genesis_case(s, reg)
        if A < 2 and depth > A:
            return PARTIAL
        if u not in journal:
            return PARTIAL
        inst_u = h.versions[u]['inst']
        ambiguous = (rows.get(inst_u) == u)
        if not ambiguous:
            return [landed_root(u, reg)]
        if not B:
            return PARTIAL
        deeper = descend(u - period, s, reg, depth + 1)
        if deeper is PARTIAL:
            return PARTIAL
        return [landed_root(u, reg)] + deeper

    for reg in range(R):
        for s in range(S):
            occ = ring_now.get((reg, s))
            if occ is None:
                per_slot.append([None])
                continue
            if occ < r:
                per_slot.append([tup(occ, h.versions[occ], reg)])
                continue
            base = s * R + reg
            if base >= r:
                u = 0
            else:
                u = base + ((r - 1 - base) // period) * period
            if u <= 0:
                o = genesis_case(s, reg)
                if o is PARTIAL:
                    return 'partial'
                per_slot.append(o)
                continue
            if u not in journal:
                return 'partial'
            inst_u = h.versions[u]['inst']
            ambiguous = (rows.get(inst_u) == u)
            if not ambiguous:
                per_slot.append([landed_root(u, reg)])
                continue
            if not B:
                return 'partial'
            deeper = descend(u - period, s, reg, 1)
            if deeper is PARTIAL:
                return 'partial'
            per_slot.append([landed_root(u, reg)] + deeper)
    combos = [[]]
    for opts in per_slot:
        combos = [c + [o] for c in combos for o in opts]
        if len(combos) > 4096:
            return 'partial'
    verdicts = set()
    for combo in combos:
        roots = [x for x in combo if x is not None and x[0] >= fold and not abandoned(x[0], x[1], rows)]
        c = ceiling(roots)
        verdicts.add('none' if c is None else ('red' if f > c else 'holds'))
    return verdicts.pop() if len(verdicts) == 1 else 'partial'


def raise_roots(h):
    out = []
    img_by_txg = {t: h.versions[t] for (t, i, f, st, reg) in h.roots if t > 0}
    for r in sorted(img_by_txg):
        if r not in h.raises:
            continue
        vr = img_by_txg[r]
        prev = [x for x in img_by_txg if img_by_txg[x]['inst'] == vr['inst'] and x < r]
        if not prev:
            continue
        p = max(prev)
        if vr['F'] <= img_by_txg[p]['F']:
            continue
        out.append((r, h.raises[r][3]))
    return out


MODES = [
    ('不放开（今天的实现）', (0, 0, 0)),
    ('只放开 A（B 不放）', (2, 0, 0)),
    ('只放开 B（A 不放）', (0, 1, 0)),
    ('放开 A 一层加 B', (1, 1, 0)),
    ('放开 A 不限层加 B', (2, 1, 0)),
    ('只放开 C', (0, 0, 1)),
    ('全放开 A 不限层加 B 加 C', (2, 1, 1)),
    ('只留 A 不放（放开 B、C）', (0, 1, 1)),
    ('只留 B 不放（放开 A、C）', (2, 0, 1)),
]
JS = (8, 32, 128)


def chunk_tally(args):
    genesis, writer, S, lo, hi, scale = args
    tally = {}
    for seed in range(lo, hi):
        steps = (20 + (seed * 7919) % 41) * scale
        h = History(seed, S, steps, 0.35, 0.04, 0.03, 0.08, writer, genesis)
        rr = raise_roots(h)
        for J in JS:
            jset = set(h.journal[-J:])
            for (r, legal) in rr:
                key = 'legal' if legal else 'illegal'
                tally[(J, key, 'count', '')] = tally.get((J, key, 'count', ''), 0) + 1
                for name, mode in MODES:
                    v = judge(h, r, jset, mode)
                    tally[(J, key, name, v)] = tally.get((J, key, name, v), 0) + 1
    return tally


def main():
    import multiprocessing, os
    seeds = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    scale = int(sys.argv[2]) if len(sys.argv) > 2 else 1
    workers = int(os.environ.get('SINGLEFS_THREAD_CAP', '4'))
    step = 250
    with multiprocessing.Pool(workers) as pool:
        for genesis in (False, True):
            for writer in ('admission', 'bad'):
                for S in (4, 8):
                    jobs = [(genesis, writer, S, lo, min(lo + step, seeds), scale) for lo in range(0, seeds, step)]
                    total = {}
                    for t in pool.imap_unordered(chunk_tally, jobs):
                        for k, v in t.items():
                            total[k] = total.get(k, 0) + v
                    for J in JS:
                        for key in ('legal', 'illegal'):
                            n = total.get((J, key, 'count', ''), 0)
                            if n == 0:
                                continue
                            for name, mode in MODES:
                                vs = ' '.join(f"{v}={total.get((J, key, name, v), 0)}" for v in ('holds', 'partial', 'red', 'none'))
                                print(f"steps_scale={scale} genesis={int(genesis)} writer={writer} S={S} J={J} {key}_raise_roots={n} mode={mode} [{name}] {vs}")
                    sys.stdout.flush()


if __name__ == '__main__':
    main()

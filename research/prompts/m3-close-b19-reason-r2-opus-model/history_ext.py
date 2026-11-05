"""合法 / 坏写者历史生成器：照搬第二轮攻方模型 research/prompts/m3-close-b19-design-r2-opus-model/q7/q7_cost_model.py 的 History
（同一 rng 消耗次序；扩展事件概率全为 0、gen0=False 时逐段与它相同，selftest 核这一点），另加三样它表示不了的态：
  gen0        mkfs 在每个区域槽 0 种第 0 代根 (txg 0, 实例 0, F 0, state 0)
  p_fallback  崩溃后恢复退回本实例更早一版（读故障让恢复择不到最新根）：行 T 取更早那一版，之后的根留在环里成被抛弃的根
  p_rootfail  根槽写失败而本实例接着发下一版（D23 已定项 14 写着「根槽写失败推进 txg」随后面的里程碑做；今天的写者冻结原样重发，
              走不到这一支）：记录进 journal、那一槽留旧内容
写者：admission（按真环算上限，写失败的槽按旧内容算）/ bad（第二轮的越界写者）/ believes_landed（把写失败的根当落了算上限，
F 可能落进重建上限与真上限之间那条缝）。"""
import random
from collections import deque

R = 3


def slot_of(txg, S):
    return (txg % R, (txg // R) % S)


def ceiling(roots_before, devices_of_region):
    """同第二轮攻方模型的 ceiling（写者一侧，D16 已定项 1 的准入式子）。"""
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
            seen.append(st)
            counted.append(t)
    fourth = counted[3] if len(counted) >= 4 else rs[0][0]
    return min(newest_on_every_device, fourth)


def abandoned(txg, inst, rows):
    return inst in rows and txg > rows[inst]


class History2:
    def __init__(self, seed, S, J, devs, steps, p_nonempty, p_lost_root, p_clean_crash, p_raise, writer,
                 gen0=False, p_fallback=0.0, p_rootfail=0.0):
        rng = random.Random(seed)
        self.S, self.J, self.devs = S, J, devs
        self.versions = {}
        self.ring = {}       # slot -> txg（盘上此刻）
        self.believed = {}   # slot -> txg（坏写者以为的环：写失败的也当落了）
        self.rows = {}
        self.journal = deque(maxlen=J)
        self.raises = {}     # r -> (F_new, F_old, 真上限, 合法?, 写者以为的上限)
        self.failed = set()
        if gen0:
            self.versions[0] = dict(inst=0, F=0, state=0, landed=True)
            for reg in range(R):
                self.ring[(reg, 0)] = 0
                self.believed[(reg, 0)] = 0
        inst, F, state, nxt, txg = 1, 0, 0, 1, 0
        landed_of_inst = []  # 本实例落了盘的 txg（退回用）

        def land(txg_, inst_, F_, state_):
            self.versions[txg_] = dict(inst=inst_, F=F_, state=state_, landed=True)
            self.journal.append(txg_)
            self.ring[slot_of(txg_, S)] = txg_
            self.believed[slot_of(txg_, S)] = txg_
            landed_of_inst.append(txg_)

        def ring_ceiling(ring_map):
            before = [v for v in ring_map.values()
                      if v >= F and not abandoned(v, self.versions[v]['inst'], self.rows)]
            return ceiling([(v, self.versions[v]['inst'], self.versions[v]['F'], self.versions[v]['state'])
                            for v in before], devs)

        for _ in range(steps):
            r = rng.random()
            if r < p_lost_root:
                txg += 1
                if rng.random() < p_nonempty:
                    state = nxt; nxt += 1
                self.versions[txg] = dict(inst=inst, F=F, state=state, landed=False)
                self.journal.append(txg)
                self.rows[inst] = txg
                inst += 1; landed_of_inst = []
                txg += 1
                land(txg, inst, F, state)
                continue
            r -= p_lost_root
            if r < p_clean_crash and txg > 0:
                self.rows[inst] = txg
                inst += 1; landed_of_inst = []
                txg += 1
                land(txg, inst, F, state)
                continue
            r -= p_clean_crash
            raise_drawn = r < p_raise
            if raise_drawn and txg > 0:
                c = ring_ceiling(self.ring)
                cb = ring_ceiling(self.believed)
                if c is not None and c > F:
                    if writer == 'admission':
                        new_f = c
                    elif writer == 'uniform':
                        new_f = rng.randint(F + 1, c)
                    elif writer == 'believes_landed':
                        new_f = max(cb, c) if cb is not None else c
                    else:
                        new_f = min(c + rng.randint(1, 3), txg)
                        if new_f <= c:
                            new_f = c
                    txg += 1
                    self.raises[txg] = (new_f, F, c, new_f <= c, cb)
                    F = new_f
                    land(txg, inst, F, state)
                    continue
            if not raise_drawn:
                r -= p_raise
                fallback_drawn = r < p_fallback
                if fallback_drawn and len(landed_of_inst) >= 2:
                    k = 1 + int(rng.random() * min(3, len(landed_of_inst) - 1))
                    self.rows[inst] = landed_of_inst[-1 - k]
                    state = self.versions[self.rows[inst]]['state']
                    F = self.versions[self.rows[inst]]['F']
                    inst += 1; landed_of_inst = []
                    txg += 1
                    land(txg, inst, F, state)
                    continue
                r -= p_fallback
                if not fallback_drawn and r < p_rootfail and txg > 0:
                    txg += 1
                    if rng.random() < p_nonempty:
                        state = nxt; nxt += 1
                    self.versions[txg] = dict(inst=inst, F=F, state=state, landed=False)
                    self.journal.append(txg)
                    self.believed[slot_of(txg, S)] = txg
                    self.failed.add(txg)
                    continue
            txg += 1
            if rng.random() < p_nonempty:
                state = nxt; nxt += 1
            land(txg, inst, F, state)
        self.N = max(self.ring.values())

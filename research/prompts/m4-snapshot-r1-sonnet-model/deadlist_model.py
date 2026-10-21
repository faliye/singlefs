#!/usr/bin/env python3
"""D5 已定项 12/13/14/16/19 的纪元式 deadlist 的小模型（三方正推腿，m4-snapshot-r1）。

只读快照、单个可写头、串行提交。每个操作是一次发布（txg 加一）；"写后建快照" 在同一次发布里（快照点在本 checkpoint
全部数据变更之后，D16 已定项 14）。块 = (birth, death)。判定与销毁照 D5 正文：

  杀块（死在发布 d）：birth > head_prev  ⇒ 立即释放；否则进 deadlist，纪元 = head_prev（已定项 12、19）
  建快照 S：S.txg = 这次发布的 txg；S.prev = head_prev；head_prev := S.txg（已定项 9、13）
  销毁 S 的意图：下一侧 N（按 (txg, id) 的下一个活快照，没有就是头）；N.prev := S.prev（已定项 13：prev 传递，同一次发布）；
                  S 从此不算活引用（已定项 12）；lo = S.txg，P = S.prev，hi = N 是快照 ? N.txg : 写意图这次发布的 txg
  销毁的一批：放掉纪元 ∈ [lo, hi) 且 birth > P 的至多 k 条（已定项 12：每个动作施加前核条目还在）；没有了就是最后一批，
              同一次发布去掉 S

真相（与上面的存储字段互不共享）：块 b 被活快照 S 引用 ⟺ birth ≤ S.txg < death（已定项 14）。

每次发布之后核：
  L 不丢：已释放的块不被任何活快照引用，也不在头里
  K 不漏记：死了、没释放的块恰好在 deadlist 里一次；deadlist 里的块都是死了没释放的
  T 紧界与不泄漏：每条条目（待放的除外）被某个活快照引用；条目所在份的 prev 满足 birth ≤ prev(份)（I-3.6）
  Q 未完成意图时：多出来的只能是 [lo, hi) 里 birth > P 的条目（待放的）

变体（--variant）：
  paper     照 D5 正文
  noprev    销毁时不传 prev（E94 变异 M5b）
  hiinf     意图的上界一律取 无穷（省掉 hi）
  hihead    下一侧是头时上界取 无穷，下一侧是快照时照常取它的 txg
  <变体>_serial  同上，但销毁意图发出之后、最后一批做完之前，别的改动（写、杀、建快照、再销毁）一律不接，只接这个销毁的批
  lt        判据写成 birth >= head_prev 才立即释放（边界取错，会丢数据的那个方向）
"""
import argparse
import random

INF = 10 ** 9


class World:
    def __init__(self, variant):
        self.v = variant
        self.t = 0
        self.next_id = 0
        self.blocks = {}      # id -> dict(birth, death, freed)
        self.live = set()
        self.head_prev = 0
        self.snaps = []       # dict(id, txg, prev, destroying, lo, hi, P)
        self.next_snap = 1
        self.dl = {}          # block id -> epoch

    # ---- 运行时存储（被测）----
    def kill(self, b):
        blk = self.blocks[b]
        blk['death'] = self.t
        self.live.discard(b)
        if self.v == 'lt':
            free_now = blk['birth'] >= self.head_prev
        else:
            free_now = blk['birth'] > self.head_prev
        if free_now:
            blk['freed'] = True
        else:
            assert b not in self.dl
            self.dl[b] = self.head_prev

    def live_snaps(self):
        return sorted([s for s in self.snaps if not s['destroying']], key=lambda s: (s['txg'], s['id']))

    def create_snapshot(self):
        s = dict(id=self.next_snap, txg=self.t, prev=self.head_prev, destroying=False, lo=0, hi=0, P=0)
        self.next_snap += 1
        self.snaps.append(s)
        self.head_prev = s['txg']

    def destroy_intent(self, s):
        order = self.live_snaps()
        i = order.index(s)
        nxt = order[i + 1] if i + 1 < len(order) else None
        if self.v != 'noprev':
            if nxt is None:
                self.head_prev = s['prev']
            else:
                nxt['prev'] = s['prev']
        s['destroying'] = True
        s['lo'] = s['txg']
        s['P'] = s['prev']
        if self.v == 'hiinf' or (self.v == 'hihead' and nxt is None):
            s['hi'] = INF
        else:
            s['hi'] = nxt['txg'] if nxt is not None else self.t

    def destroy_batch(self, s, k):
        todo = [b for b, e in sorted(self.dl.items())
                if s['lo'] <= e < s['hi'] and self.blocks[b]['birth'] > s['P']]
        for b in todo[:k]:
            del self.dl[b]
            self.blocks[b]['freed'] = True
        if len(todo) <= k:
            self.snaps.remove(s)
            return True
        return False

    # ---- 真相 ----
    def needed_by_a_live_snapshot(self, b):
        blk = self.blocks[b]
        d = blk['death'] if blk['death'] is not None else INF
        return any(blk['birth'] <= s['txg'] < d for s in self.live_snaps())

    def check(self):
        errs = []
        in_flight = [s for s in self.snaps if s['destroying']]
        for b, blk in self.blocks.items():
            dead = blk['death'] is not None
            if blk['freed']:
                if b in self.live:
                    errs.append(('L-live-freed', b))
                if self.needed_by_a_live_snapshot(b):
                    errs.append(('L-lost', b))
            elif dead and b not in self.dl:
                errs.append(('K-unrecorded', b))
        for b in self.dl:
            blk = self.blocks[b]
            if blk['death'] is None or blk['freed']:
                errs.append(('K-bad-entry', b))
        live = self.live_snaps()

        def share_prev_of(epoch):
            for s in live:
                if s['prev'] <= epoch < s['txg']:
                    return s['prev']
            return self.head_prev if epoch >= self.head_prev else None

        for b, e in self.dl.items():
            blk = self.blocks[b]
            if any(s['lo'] <= e < s['hi'] and blk['birth'] > s['P'] for s in in_flight):
                continue
            if not self.needed_by_a_live_snapshot(b):
                errs.append(('T-leak', b))
            pv = share_prev_of(e)
            if pv is None or blk['birth'] > pv:
                errs.append(('T-loose', b))
        return errs


def run(variant, seed, steps):
    rnd = random.Random(seed)
    serial = variant.endswith('_serial')   # 销毁意图发出之后、做完之前不接别的改动（会话里每次用户改动先走空的同一个纪律）
    w = World(variant.replace('_serial', ''))
    for _ in range(steps):
        w.t += 1
        op = rnd.choice(['write', 'write', 'kill', 'kill', 'snap', 'snap', 'destroy', 'batch', 'batch', 'piggy'])
        if serial and any(s['destroying'] for s in w.snaps):
            op = 'batch'
        if op in ('write', 'piggy'):
            for _i in range(rnd.randint(1, 3)):
                b = w.next_id
                w.next_id += 1
                w.blocks[b] = dict(birth=w.t, death=None, freed=False)
                w.live.add(b)
            if op == 'piggy':
                w.create_snapshot()
        elif op == 'kill' and w.live:
            for b in rnd.sample(sorted(w.live), rnd.randint(1, min(3, len(w.live)))):
                w.kill(b)
        elif op == 'snap':
            w.create_snapshot()
        elif op == 'destroy':
            cands = [s for s in w.snaps if not s['destroying']]
            if cands:
                w.destroy_intent(rnd.choice(cands))
        elif op == 'batch':
            flying = [s for s in w.snaps if s['destroying']]
            if flying:
                w.destroy_batch(rnd.choice(flying), rnd.randint(1, 2))
        errs = w.check()
        if errs:
            return errs
    # 收尾：把在飞的销毁做完，再核一遍
    for s in [s for s in w.snaps if s['destroying']]:
        while not w.destroy_batch(s, 2):
            pass
    return w.check()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--variant', default='paper')
    ap.add_argument('--histories', type=int, default=20000)
    ap.add_argument('--steps', type=int, default=40)
    a = ap.parse_args()
    kinds = {}
    bad = 0
    first = None
    for seed in range(a.histories):
        errs = run(a.variant, seed, a.steps)
        if errs:
            bad += 1
            k = errs[0][0]
            kinds[k] = kinds.get(k, 0) + 1
            if first is None:
                first = (seed, errs[0])
    print(f"name=deadlist_model variant={a.variant} histories={a.histories} steps={a.steps} "
          f"violated={bad} kinds={dict(sorted(kinds.items()))} first={first}")


if __name__ == '__main__':
    main()

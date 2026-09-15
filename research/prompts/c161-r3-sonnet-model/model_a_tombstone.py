#!/usr/bin/env python3
"""
C161 round 3, defense leg. Model A: does treating a delete (whiteout/tombstone)
message identically to a put message preserve G1 (reorg-invariant reads) under
the round-2 fixed forms T1 (fold_up pushes root buffer into children before
merging) and T2 (rebalance carries buffer content with the key)?

Two keys: 'a' always routes to child 0, 'b' can move between child 0 / child 1
via rebalance (matches c161-r2-opus-model/c161_r2_model.py's setup).

Candidates: jia_R4 (front-end flush -> leaf directly; node buffer only holds
direct/bypass writes), yi_D (front-end flush -> root buffer, cascades down,
single entry per buffer, newest replaces), yi_G (same cascade but a buffer
level may hold several messages for one key; ties at a level broken by a
pool-wide monotonic seq), bing (front-end flush -> leaf directly, no node
buffer at all for front-end trees).

BFS over operation sequences up to MAX_DEPTH, exact-state dedup by signature.
"""
import os
import sys
import itertools
from collections import deque

KEYS = ('a', 'b')

# self-test mutation switch: when set, a delete message is silently dropped
# instead of being written as a tombstone (models "detele isn't handled like
# any other message" -- the thing this script exists to rule out).
MUTATE_DROP_DEL = os.environ.get('MUTATE_DROP_DEL') == '1'


class Msg:
    __slots__ = ('kind', 'val', 'arr', 'seq')

    def __init__(self, kind, val, arr, seq):
        self.kind = kind    # 'put' or 'del'
        self.val = val      # int or None
        self.arr = arr      # global arrival counter (true temporal order)
        self.seq = seq      # write-buffer seq (yi_G only consumes this)

    def sig(self):
        return (self.kind, self.val, self.arr, self.seq)

    def __repr__(self):
        return f"{self.kind}{self.val if self.val is not None else ''}@{self.arr}/s{self.seq}"


def newer(m1, m2):
    """Return the message with the larger arrival order (None-safe)."""
    if m1 is None:
        return m2
    if m2 is None:
        return m1
    return m1 if m1.arr >= m2.arr else m2


class State:
    """Immutable-by-convention state; we always build fresh dict/list copies."""
    __slots__ = ('height', 'partition', 'fe', 'rb', 'cb', 'lf', 'arr_ctr', 'seq_ctr')

    def __init__(self):
        self.height = 2
        self.partition = 0          # which child currently holds 'b' (0 or 1)
        self.fe = {}                # key -> Msg (single; in-memory dedup)
        self.rb = {}                # key -> Msg or key -> [Msg] (yi_G)
        self.cb = [{}, {}]          # per-child-index: key -> Msg or [Msg]
        self.lf = {}                # key -> Msg or [Msg] (yi_G only)
        self.arr_ctr = 0
        self.seq_ctr = 0

    def clone(self):
        s = State()
        s.height = self.height
        s.partition = self.partition
        s.fe = dict(self.fe)
        s.rb = {k: (list(v) if isinstance(v, list) else v) for k, v in self.rb.items()}
        s.cb = [
            {k: (list(v) if isinstance(v, list) else v) for k, v in c.items()}
            for c in self.cb
        ]
        s.lf = {k: (list(v) if isinstance(v, list) else v) for k, v in self.lf.items()}
        s.arr_ctr = self.arr_ctr
        s.seq_ctr = self.seq_ctr
        return s

    def child_of(self, key):
        return 0 if key == 'a' else self.partition

    def sig(self):
        def lvl_sig(level):
            out = []
            for k in sorted(level.keys()):
                v = level[k]
                if isinstance(v, list):
                    out.append((k, tuple(sorted(m.sig() for m in v))))
                else:
                    out.append((k, v.sig()))
            return tuple(out)
        return (
            self.height, self.partition,
            lvl_sig(self.fe), lvl_sig(self.rb),
            lvl_sig(self.cb[0]), lvl_sig(self.cb[1]),
            lvl_sig(self.lf),
        )


def append_or_set(level, key, msg, use_list):
    if MUTATE_DROP_DEL and msg.kind == 'del':
        return
    if use_list:
        level.setdefault(key, []).append(msg)
    else:
        level[key] = msg


def dedupe_write(level, key, msg):
    """single-entry newest-wins write into a level (used by leaf writes for
    jia_R4/bing, and by buffer writes for jia_R4/yi_D)."""
    if MUTATE_DROP_DEL and msg.kind == 'del':
        return
    if key in level:
        level[key] = newer(level[key], msg)
    else:
        level[key] = msg


def best_of_list(msgs):
    """yi_G tie-break: pick the message with the largest seq."""
    best = msgs[0]
    for m in msgs[1:]:
        if m.seq > best.seq:
            best = m
    return best


# ---------- operations ----------

def op_fe_put(s, key, candidate):
    s2 = s.clone()
    s2.arr_ctr += 1
    s2.seq_ctr += 1
    s2.fe[key] = Msg('put', s2.arr_ctr, s2.arr_ctr, s2.seq_ctr)
    return s2


def op_fe_del(s, key, candidate):
    s2 = s.clone()
    s2.arr_ctr += 1
    s2.seq_ctr += 1
    s2.fe[key] = Msg('del', None, s2.arr_ctr, s2.seq_ctr)
    return s2


def op_direct(s, key, kind, candidate):
    # jia_R4 only: "不走前端的写...进根缓冲并逐出前端里同 key 的条目"
    s2 = s.clone()
    s2.arr_ctr += 1
    s2.seq_ctr += 1
    m = Msg(kind, s2.arr_ctr if kind == 'put' else None, s2.arr_ctr, s2.seq_ctr)
    s2.fe.pop(key, None)
    dedupe_write(s2.rb, key, m)
    return s2


def op_flush(s, candidate):
    s2 = s.clone()
    use_list = (candidate == 'yi_G')
    for key in list(s2.fe.keys()):
        m = s2.fe.pop(key)
        if candidate in ('jia_R4', 'bing'):
            # flush bypasses node buffers, goes straight to leaf; first clear
            # any path entries strictly older than m ("清掉路径上比它旧的同
            # key 消息", jia_R4 only -- bing has no node buffers at all).
            if candidate == 'jia_R4':
                ci = s2.child_of(key)
                if key in s2.rb and s2.rb[key].arr < m.arr:
                    del s2.rb[key]
                if key in s2.cb[ci] and s2.cb[ci][key].arr < m.arr:
                    del s2.cb[ci][key]
            dedupe_write(s2.lf, key, m)
        elif candidate == 'yi_D':
            dedupe_write(s2.rb, key, m)
        elif candidate == 'yi_G':
            append_or_set(s2.rb, key, m, True)
    return s2


def op_push_root(s, candidate):
    # yi_D / yi_G cascade root buffer -> child buffer (only meaningful at
    # height 3); jia_R4 also allowed for generality (its root buffer only
    # ever holds direct writes).
    if s.height != 3:
        return None
    s2 = s.clone()
    use_list = (candidate == 'yi_G')
    for key in list(s2.rb.keys()):
        ci = s2.child_of(key)
        if use_list:
            msgs = s2.rb.pop(key)
            s2.cb[ci].setdefault(key, []).extend(msgs)
        else:
            m = s2.rb.pop(key)
            dedupe_write(s2.cb[ci], key, m)
    return s2


def op_push_child(s, ci, candidate):
    if s.height != 3 or not s.cb[ci]:
        return None
    s2 = s.clone()
    use_list = (candidate == 'yi_G')
    for key in list(s2.cb[ci].keys()):
        if use_list:
            msgs = s2.cb[ci].pop(key)
            s2.lf.setdefault(key, []).extend(msgs)
        else:
            m = s2.cb[ci].pop(key)
            dedupe_write(s2.lf, key, m)
    return s2


def op_grow(s, candidate):
    if s.height != 2:
        return None
    if candidate == 'bing':
        return None  # bing's front-end trees never carry a node buffer
    s2 = s.clone()
    use_list = (candidate == 'yi_G')
    new_cb = [{}, {}]
    for key in list(s2.rb.keys()):
        ci = 0 if key == 'a' else 0  # partition resets on grow, 'b' starts at 0
        v = s2.rb.pop(key)
        new_cb[ci][key] = v
    s2.cb = new_cb
    s2.rb = {}
    s2.partition = 0
    s2.height = 3
    return s2


def op_shrink(s, candidate):
    # T1 fixed rule: push root buffer into children first, then merge.
    if s.height != 3:
        return None
    if candidate == 'bing':
        return None
    s2 = op_push_root(s, candidate)
    if s2 is None:
        s2 = s.clone()
    new_rb = {}
    use_list = (candidate == 'yi_G')
    for ci in (0, 1):
        for key, v in s2.cb[ci].items():
            if use_list:
                new_rb.setdefault(key, []).extend(v)
            else:
                if key in new_rb:
                    new_rb[key] = newer(new_rb[key], v)
                else:
                    new_rb[key] = v
    s2.rb = new_rb
    s2.cb = [{}, {}]
    s2.height = 2
    s2.partition = 0
    return s2


def op_rebalance(s, candidate):
    # T2 fixed rule: buffer content for 'b' travels with it.
    if s.height != 3:
        return None
    if candidate == 'bing':
        return None
    s2 = s.clone()
    old_ci = s2.partition
    new_ci = 1 - old_ci
    if 'b' in s2.cb[old_ci]:
        s2.cb[new_ci]['b'] = s2.cb[old_ci].pop('b')
    s2.partition = new_ci
    return s2


def ground_truth_update(gt, s, key):
    # ground truth = the temporally-last message ever created for `key`,
    # tracked independently of where the candidate currently stores it.
    pass  # handled inline at call sites (see build_ops)


def read(s, key, candidate):
    if candidate in ('jia_R4', 'yi_D'):
        ci = s.child_of(key)
        for level in (s.fe, s.rb, s.cb[ci], s.lf):
            if key in level:
                return level[key]
        return None
    if candidate == 'bing':
        for level in (s.fe, s.lf):
            if key in level:
                return level[key]
        return None
    if candidate == 'yi_G':
        ci = s.child_of(key)
        if key in s.fe:
            return s.fe[key]
        for level in (s.rb, s.cb[ci], s.lf):
            if key in level:
                v = level[key]
                return best_of_list(v) if isinstance(v, list) else v
        return None
    raise ValueError(candidate)


def run(candidate, max_depth):
    start = State()
    gt0 = {'a': None, 'b': None}
    start_sig = (start.sig(), tuple(sorted(gt0.items())))
    seen = {start_sig}
    queue = deque([(start, gt0, [])])
    violations = []
    states_explored = 0

    def try_op(name, s, gt, path, newstate, key_touched=None, kind=None):
        if newstate is None:
            return None
        gt2 = dict(gt)
        if key_touched is not None:
            gt2[key_touched] = (kind, newstate.arr_ctr)
        return (newstate, gt2, path + [name])

    while queue and len(path_ := []) == 0:  # placeholder to keep flake quiet
        break

    while queue:
        s, gt, path = queue.popleft()
        states_explored += 1

        # check G1 for this state
        for key in KEYS:
            r = read(s, key, candidate)
            truth = gt[key]
            if truth is None:
                if r is not None:
                    violations.append(('phantom', key, path, r, truth))
            else:
                tkind, tarr = truth
                if r is None or r.kind != tkind or r.arr != tarr:
                    violations.append(('stale', key, path, r, truth))

        if len(path) >= max_depth:
            continue

        nexts = []
        for key in KEYS:
            s2 = op_fe_put(s, key, candidate)
            nexts.append((f'fe_put_{key}', s2, key, 'put'))
            s2 = op_fe_del(s, key, candidate)
            nexts.append((f'fe_del_{key}', s2, key, 'del'))
        if candidate == 'jia_R4':
            for key in KEYS:
                s2 = op_direct(s, key, 'put', candidate)
                nexts.append((f'direct_put_{key}', s2, key, 'put'))
                s2 = op_direct(s, key, 'del', candidate)
                nexts.append((f'direct_del_{key}', s2, key, 'del'))
        nexts.append(('flush', op_flush(s, candidate), None, None))
        if candidate in ('yi_D', 'yi_G', 'jia_R4'):
            nexts.append(('push_root', op_push_root(s, candidate), None, None))
            nexts.append(('push_child0', op_push_child(s, 0, candidate), None, None))
            nexts.append(('push_child1', op_push_child(s, 1, candidate), None, None))
        if candidate != 'bing':
            nexts.append(('grow', op_grow(s, candidate), None, None))
            nexts.append(('shrink', op_shrink(s, candidate), None, None))
            nexts.append(('rebalance', op_rebalance(s, candidate), None, None))

        for name, s2, key_touched, kind in nexts:
            if s2 is None:
                continue
            gt2 = dict(gt)
            if key_touched is not None:
                gt2[key_touched] = (kind, s2.arr_ctr)
            sig = (s2.sig(), tuple(sorted(gt2.items())))
            if sig in seen:
                continue
            seen.add(sig)
            queue.append((s2, gt2, path + [name]))

    return states_explored, violations


def main():
    max_depth = int(sys.argv[1]) if len(sys.argv) > 1 else 8
    candidates = ['jia_R4', 'yi_D', 'yi_G', 'bing']
    total_violations = 0
    for c in candidates:
        n, viol = run(c, max_depth)
        print(f"{c}: states_explored={n} violations={len(viol)}")
        for kind, key, path, r, truth in viol[:3]:
            print(f"    {kind} key={key} path={path} read={r} truth={truth}")
        total_violations += len(viol)
    print(f"TOTAL violations across {len(candidates)} candidates: {total_violations}")
    print(f"emitted={len(candidates)}")


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""m4-gc-r1 攻方模型：抽象分配器 + 整理意图，广度优先全枚举（定义见同目录 prereg.md）。

用法：gc_model.py ARM NSEG SEGSZ RING MAXOBJ B DEPTH
      gc_model.py list
输出：一行 E name=... 计数；每个判据第一次打中时一行 W flag=... path=...（广度优先，最短见证）。
"""
import os
import sys

CAP_FACTOR = int(os.environ.get('GC_MODEL_CAP_FACTOR', '8'))
# 新写的单元 key 的写序取这个基数加 txg：规范化之后写序是名次（小整数），新 key 要与环里已有的各版都不相等
NEW_KEY_BASE = 10 ** 6

BASE = dict(move='rewrite', count='rec', dest='user', precond=False, select='safe',
            needmov=True, cont='recheck', rbint='old', plan='batch', inst=False, fault=False,
            bounded=False, n0=0, n0scope='all')
ARMS = dict(
    base=dict(),
    maponly_regen=dict(move='maponly_regen'),
    maponly_carry=dict(move='maponly_carry'),
    occ=dict(count='occ'),
    holes=dict(dest='holes'),
    holes_pre=dict(dest='holes', precond=True),
    occ_holes_pre=dict(count='occ', dest='holes', precond=True),
    cached=dict(plan='cached'),
    cur_skip=dict(rbint='cur', cont='skip'),
    cur_recheck=dict(rbint='cur'),
    old_skip=dict(cont='skip'),
    naive=dict(select='naive', needmov=False),
    inst_naive=dict(inst=True, select='naive'),
    inst_safe=dict(inst=True),
    fault_naive=dict(fault=True, select='naive'),
    fault_safe=dict(fault=True),
    fix_all=dict(dest='holes', precond=True, inst=True, fault=True),
    bounded=dict(bounded=True),
    holes_pre_bounded=dict(dest='holes', precond=True, bounded=True),
    n0_all=dict(n0=1, n0scope='all'),
    n0_fg=dict(n0=1, n0scope='fg'),
)
STATE_FLAGS = ['live_released', 'unreadable', 'dup_map', 'leak', 'hint_bad', 'gc_stuck', 'livelock',
               'frozen']
EDGE_FLAGS = ['unmet_del', 'false_enospc', 'release_bad', 'rb_refused', 'del_pred_true',
              'del_pred_false', 'n0_refused']


class Model:
    """根：(txg, objs, hint, mapping, recs, meta, inst, intent, quar)。
    objs[j] = 单元 key (j, 写序 txg) 或 None；hint[j] / mapping[j] = 槽或 None；recs[s] = None 或 (分配代, 释放代或 None)；
    intent = None 或 (R 段号, c0, segs0, 续做点)；quar = 隔离槽的有序元组。
    全局状态：(环 = 根的元组、旧在前；盘 = 每槽内容；会话 = (本次挂载开过的段, 开放段, 游标, 缓存计划)；已注入隔离)。"""

    def __init__(self, cfg, nseg, segsz, ring, maxobj, b):
        self.c = cfg
        self.NSEG, self.SZ, self.RING, self.MAXOBJ, self.B = nseg, segsz, ring, maxobj, b
        self.NSLOT = nseg * segsz
        self.last_n0 = False

    def mkfs(self):
        recs = [None] * self.NSLOT
        recs[0] = (0, None)
        disk = ['M'] + [None] * (self.NSLOT - 1)
        none = (None,) * self.MAXOBJ
        root = (0, none, none, none, tuple(recs), 0, None, None, ())
        st = ((root,), tuple(disk), (frozenset([0]), 0, 1, None), False)
        if self.c['inst']:
            st = self.op_mount(st)[0]
        return st

    def seg(self, s):
        return s // self.SZ

    def slots(self, g):
        return range(g * self.SZ, (g + 1) * self.SZ)

    @staticmethod
    def oldest(st):
        return st[0][0][0]

    def free(self, st, recs, s):
        r = recs[s]
        return r is None or (r[1] is not None and r[1] <= self.oldest(st))

    def seg_empty(self, st, recs, g, mode):
        for s in self.slots(g):
            r = recs[s]
            if mode == 'occ':
                if not self.free(st, recs, s):
                    return False
            elif r is not None and r[1] is None:
                return False
        return True

    def empty_count(self, st, recs, mode):
        return sum(1 for g in range(self.NSEG) if self.seg_empty(st, recs, g, mode))

    @staticmethod
    def r_of(root):
        return None if root[7] is None else root[7][0]

    def place_data(self, st, recs, sess, rseg, holes=False):
        for s in range(self.NSLOT):
            g = self.seg(s)
            if g in sess[0] or g == rseg:
                continue
            if holes and self.seg_empty(st, recs, g, 'rec'):
                continue
            if self.free(st, recs, s):
                return s
        return None

    def place_meta(self, st, recs, sess, rseg):
        opened, oseg, cur, plan = sess
        if oseg is not None and oseg != rseg:
            for s in range(max(cur, oseg * self.SZ), (oseg + 1) * self.SZ):
                if self.free(st, recs, s):
                    return s, (opened, oseg, s + 1, plan)
        for g in range(self.NSEG):
            if g != rseg and all(self.free(st, recs, s) for s in self.slots(g)):
                return g * self.SZ, (opened | frozenset([g]), g, g * self.SZ + 1, plan)
        for s in range(self.NSLOT):
            if self.seg(s) != rseg and self.free(st, recs, s):
                return s, sess
        return None, sess

    def commit(self, st, sess, objs, hint, mapping, recs, intent, quar, writes, new_inst=False,
               faulted=None, kind='sys'):
        """收尾一次发布：取这次的元数据块、释放上一版那一块，写盘，换根。落不了点交 None。
        kind：fg（写入类）、del（删除）、gc（整理）、sys（挂载、回退）；合取 7（全空段数 ≥ N₀）按 n0scope 管 all 或只管 fg，
        在这次发布动账之前按上一版的账判。"""
        ring, disk, _, was_faulted = st
        cur = ring[-1]
        if self.c['n0'] and (self.c['n0scope'] == 'all' or kind == 'fg'):
            if self.empty_count(st, cur[4], self.c['count']) < self.c['n0']:
                self.last_n0 = True
                return None
        T = cur[0] + 1
        recs = list(recs)
        rseg = None if intent is None else intent[0]
        if rseg is None and cur[7] is not None:
            rseg = cur[7][0]
        disk = list(disk)
        for s, content in writes:
            disk[s] = content
        inst_slot = cur[6]
        if new_inst:
            s, sess = self.place_meta(st, recs, sess, rseg)
            if s is None:
                return None
            if cur[6] is not None:
                recs[cur[6]] = (recs[cur[6]][0], T)
            recs[s] = (T, None)
            disk[s] = 'I'
            inst_slot = s
        m, sess = self.place_meta(st, recs, sess, rseg)
        if m is None:
            return None
        recs[cur[5]] = (recs[cur[5]][0], T)
        recs[m] = (T, None)
        disk[m] = 'M'
        root = (T, tuple(objs), tuple(hint), tuple(mapping), tuple(recs), m, inst_slot,
                intent, tuple(sorted(quar)))
        ring = (ring + (root,))[-self.RING:]
        return (ring, tuple(disk), sess, was_faulted if faulted is None else faulted)

    def regen(self, objs, hint, mapping):
        """非整理的发布：maponly_regen 臂照今天的代码，按引用方指针（提示）重算每条映射条目。"""
        if self.c['move'] == 'maponly_regen':
            return [hint[j] if objs[j] is not None else None for j in range(self.MAXOBJ)]
        return list(mapping)

    def would_place_without_r(self, st, recs, sess):
        return self.place_data(st, recs, sess, None) is not None

    # ---- 操作：交 (新状态或 None, 转移判据集合) ----
    def op_write(self, st, j):
        cur = st[0][-1]
        if cur[1][j] is not None:
            return None, set()
        recs = list(cur[4])
        rseg = self.r_of(cur)
        s = self.place_data(st, recs, st[2], rseg)
        if s is None:
            flags = set()
            if rseg is not None and self.would_place_without_r(st, recs, st[2]):
                flags.add('false_enospc')
            return None, flags
        T = cur[0] + 1
        key = (j, NEW_KEY_BASE + T)
        objs, hint = list(cur[1]), list(cur[2])
        mapping = self.regen(cur[1], cur[2], cur[3])
        objs[j], hint[j], mapping[j] = key, s, s
        recs[s] = (T, None)
        return self.commit(st, st[2], objs, hint, mapping, recs, cur[7], cur[8], [(s, key)],
                           kind='fg'), set()

    def release_via_mapping(self, recs, s, T, flags):
        r = recs[s]
        if r is None or r[1] is not None:
            flags.add('release_bad')
            return
        recs[s] = (r[0], T)

    def op_overwrite(self, st, j, fault=False):
        cur = st[0][-1]
        if cur[1][j] is None or (fault and st[3]):
            return None, set()
        recs = list(cur[4])
        rseg = self.r_of(cur)
        s = self.place_data(st, recs, st[2], rseg)
        flags = set()
        if s is None:
            if rseg is not None and self.would_place_without_r(st, recs, st[2]):
                flags.add('false_enospc')
            return None, flags
        T = cur[0] + 1
        old = cur[3][j]
        quar = list(cur[8])
        if fault:
            quar.append(old)
        else:
            self.release_via_mapping(recs, old, T, flags)
        key = (j, NEW_KEY_BASE + T)
        objs, hint = list(cur[1]), list(cur[2])
        mapping = self.regen(cur[1], cur[2], cur[3])
        objs[j], hint[j], mapping[j] = key, s, s
        recs[s] = (T, None)
        nst = self.commit(st, st[2], objs, hint, mapping, recs, cur[7], quar, [(s, key)],
                          faulted=True if fault else None, kind='fg')
        return nst, flags

    def op_delete(self, st, j):
        cur = st[0][-1]
        if cur[1][j] is None:
            return None, set()
        recs = list(cur[4])
        T = cur[0] + 1
        flags = set()
        self.release_via_mapping(recs, cur[3][j], T, flags)
        objs, hint = list(cur[1]), list(cur[2])
        mapping = self.regen(cur[1], cur[2], cur[3])
        objs[j], hint[j], mapping[j] = None, None, None
        return self.commit(st, st[2], objs, hint, mapping, recs, cur[7], cur[8], [],
                           kind='del'), flags

    def owners(self, root):
        return dict((root[3][j], j) for j in range(self.MAXOBJ) if root[1][j] is not None)

    def violators(self, recs, R, c0, start):
        return [s for s in self.slots(R) if s >= start and recs[s] is not None
                and recs[s][1] is None and recs[s][0] <= c0]

    def choose_r(self, st):
        cur = st[0][-1]
        recs = cur[4]
        own = self.owners(cur)
        best = None
        for g in range(self.NSEG):
            if self.seg_empty(st, recs, g, 'occ'):
                continue
            if self.c['select'] == 'safe':
                if g in st[2][0]:
                    continue
                if cur[6] is not None and self.seg(cur[6]) == g:
                    continue
                if any(self.seg(q) == g for q in cur[8]):
                    continue
            nfree = sum(1 for s in self.slots(g) if self.free(st, recs, s))
            movable = [s for s in self.violators(recs, g, cur[0], g * self.SZ) if s in own]
            if self.c['needmov'] and not movable:
                continue
            if self.c['precond']:
                holes = sum(1 for s in range(self.NSLOT) if self.seg(s) != g
                            and self.seg(s) not in st[2][0]
                            and not self.seg_empty(st, recs, self.seg(s), 'rec')
                            and self.free(st, recs, s))
                if not movable or holes < len(movable):
                    continue
            if best is None or nfree > best[0]:
                best = (nfree, g)
        return None if best is None else best[1]

    def op_issue(self, st):
        cur = st[0][-1]
        if cur[7] is not None:
            return None, set()
        R = self.choose_r(st)
        if R is None:
            return None, set()
        recs = cur[4]
        c0 = cur[0]
        segs0 = self.empty_count(st, recs, self.c['count'])
        sess = st[2]
        if self.c['plan'] == 'cached':
            own = self.owners(cur)
            sim = list(recs)
            plan = []
            for s in self.violators(recs, R, c0, R * self.SZ):
                if s not in own:
                    continue
                d = self.place_data(st, sim, sess, R, self.c['dest'] == 'holes')
                if d is None:
                    break
                sim[d] = (c0 + 1, None)
                plan.append((own[s], s, d))
            sess = (sess[0], sess[1], sess[2], tuple(plan))
        mapping = self.regen(cur[1], cur[2], cur[3])
        intent = (R, c0, segs0, R * self.SZ)
        return self.commit(st, sess, cur[1], cur[2], mapping, recs, intent, cur[8], [],
                           kind='gc'), set()

    def op_batch(self, st):
        """一批整理。self.info 记 (这一批删没删意图, 删那一刻停机谓词的值)。"""
        self.info = None
        self.stuck = False
        self.paused = False
        cur = st[0][-1]
        if cur[7] is None:
            return None, set()
        R, c0, segs0, cont = cur[7]
        recs = list(cur[4])
        T = cur[0] + 1
        flags = set()
        own = self.owners(cur)
        start = cont if self.c['cont'] == 'skip' else R * self.SZ
        v = self.violators(recs, R, c0, start)
        movable = [s for s in v if s in own]
        if v and not movable:
            self.stuck = True
            return None, flags
        objs, hint, mapping = list(cur[1]), list(cur[2]), list(cur[3])
        writes = []
        plan = st[2][3]
        new_cont = cont
        for s in movable[:self.B]:
            j = own[s]
            d = None
            if self.c['plan'] == 'cached' and plan is not None:
                for (pj, ps, pd) in plan:
                    if pj == j and ps == s:
                        d = pd
            if d is None:
                d = self.place_data(st, recs, st[2], R, self.c['dest'] == 'holes')
            if d is None:
                self.paused = True
                return None, flags
            writes.append((d, objs[j]))
            recs[d] = (T, None)
            recs[s] = (recs[s][0], T)
            mapping[j] = d
            if self.c['move'] == 'rewrite':
                hint[j] = d
            new_cont = s + 1
        rest = self.violators(recs, R, c0, new_cont if self.c['cont'] == 'skip' else R * self.SZ)
        intent = (R, c0, segs0, new_cont)
        deleted = False
        if not rest:
            intent = None
            deleted = True
            if self.violators(recs, R, c0, R * self.SZ):
                flags.add('unmet_del')
        nst = self.commit(st, st[2], objs, hint, mapping, recs, intent, cur[8], writes, kind='gc')
        if nst is not None and deleted:
            pred = self.empty_count(nst, nst[0][-1][4], self.c['count']) - segs0 >= 1
            self.info = (True, pred)
            flags.add('del_pred_true' if pred else 'del_pred_false')
        return nst, flags

    def op_mount(self, st):
        cur = st[0][-1]
        sess = (frozenset(), None, 0, None)
        mapping = self.regen(cur[1], cur[2], cur[3])
        nst = self.commit(st, sess, cur[1], cur[2], mapping, cur[4], cur[7], cur[8], [],
                          new_inst=self.c['inst'])
        return nst, set()

    def op_rollback(self, st, k):
        ring = st[0]
        if len(ring) <= k:
            return None, set()
        cur, old = ring[-1], ring[-1 - k]
        recs = list(cur[4])
        T = cur[0] + 1
        flags = set()
        target_slots = set()
        for j in range(self.MAXOBJ):
            if old[1][j] is None:
                continue
            s = old[3][j]
            target_slots.add(s)
            r, ro = recs[s], old[4][s]
            if r is not None and r[1] is None:
                if r[0] != ro[0]:
                    flags.add('rb_refused')
                    return None, flags
            elif r is not None and r[1] is not None and not self.free(st, recs, s):
                recs[s] = (ro[0], None)
            else:
                flags.add('rb_refused')
                return None, flags
        for j in range(self.MAXOBJ):
            if cur[1][j] is not None and cur[3][j] not in target_slots:
                self.release_via_mapping(recs, cur[3][j], T, flags)
        intent = cur[7] if self.c['rbint'] == 'cur' else old[7]
        mapping = self.regen(old[1], old[2], old[3])
        return self.commit(st, st[2], old[1], old[2], mapping, recs, intent, cur[8], []), flags

    # ---- 规范化（去重）：txg 整体减环里最旧根的 txg；已可再分配的记录记成 (-1, -1)；不是环里任何一条根的活 key 的盘内容记成 G；
    #      单元 key 的写序换成同一对象在环里出现过的各版里的名次（key 只拿来比相等与新旧，名次保住这两样） ----
    def norm(self, st):
        ring, disk, sess, faulted = st
        base = ring[0][0]
        keys = set()
        for root in ring:
            for k in root[1]:
                if k is not None:
                    keys.add(k)

        rank = dict()
        for j in range(self.MAXOBJ):
            for i, wo in enumerate(sorted(k[1] for k in keys if k[0] == j)):
                rank[(j, wo)] = i

        def nk(k):
            return None if k is None else (k[0], rank[k])

        def nr(r):
            if r is None:
                return None
            if r[1] is None:
                return (r[0] - base, None)
            if r[1] <= base:
                return (-1, -1)
            return (r[0] - base, r[1] - base)

        nring = []
        for root in ring:
            it = root[7]
            if it is not None:
                it = (it[0], it[1] - base, it[2], it[3])
            nring.append((root[0] - base, tuple(nk(k) for k in root[1]), root[2], root[3],
                          tuple(nr(r) for r in root[4]), root[5], root[6], it, root[8]))
        ndisk = tuple(nk(d) if d in keys else 'G' for d in disk)
        return (tuple(nring), ndisk, sess, faulted)

    def state_flags(self, st):
        cur = st[0][-1]
        objs, hint, mapping, recs, meta, inst = cur[1], cur[2], cur[3], cur[4], cur[5], cur[6]
        disk = st[1]
        out = set()
        live = [j for j in range(self.MAXOBJ) if objs[j] is not None]
        mslots = [mapping[j] for j in live]
        for j in live:
            m, h, key = mapping[j], hint[j], objs[j]
            r = recs[m]
            if r is None or r[1] is not None:
                out.add('live_released')
            if disk[h] != key and disk[m] != key:
                out.add('unreadable')
            if h != m:
                rh = recs[h]
                if rh is None or rh[1] is not None or disk[h] != key:
                    out.add('hint_bad')
        if len(set(mslots)) != len(mslots) or meta in mslots or (inst is not None and inst in mslots):
            out.add('dup_map')
        known = set(mslots) | set([meta]) | set(cur[8])
        if inst is not None:
            known.add(inst)
        for s in range(self.NSLOT):
            r = recs[s]
            if r is not None and r[1] is None and s not in known:
                out.add('leak')
        if cur[7] is not None:
            self.op_batch(st)
            if self.stuck:
                out.add('gc_stuck')
        if self.c['n0']:
            alive = False
            for label, (nst, _) in self.successors_raw(st):
                if nst is not None and label[0] != 'R':
                    alive = True
                    break
            if not alive:
                out.add('frozen')
        return out

    def gc_only(self, st, cache):
        """从 st 起只跑整理（G8 甲：一次调用跑到停机谓词为真或停下）：签发 → 各批 → 删意图那一刻判停机谓词，假就再签一条。
        交 done（谓词真）/ declined（选不出 R）/ stuck（R 里只剩搬不动的违例槽）/ paused（目的地落不了点或准入拒）/
        cycle（回到见过的状态，停不下）/ cap（CAP_FACTOR·NSEG 条意图之内没停）/ wasted（bounded 臂：一条意图完成而谓词假）。"""
        key0 = st
        if key0 in cache:
            return cache[key0]
        s = st
        seen = set()
        result = 'cap'
        cycles = 0
        while True:
            if s[0][-1][7] is None:
                if cycles >= CAP_FACTOR * self.NSEG:
                    break
                nst, _ = self.op_issue(s)
                if nst is None:
                    result = 'declined'
                    break
                s = self.norm(nst)
                cycles += 1
            nst, _ = self.op_batch(s)
            if nst is None:
                result = 'stuck' if self.stuck else 'paused'
                break
            info = self.info
            s = self.norm(nst)
            if s in seen:
                result = 'cycle'
                break
            seen.add(s)
            if info is not None and info[1]:
                result = 'done'
                break
            if info is not None and self.c['bounded']:
                result = 'wasted'
                break
        cache[key0] = result
        return result

    def successors(self, st):
        for label, res in self.successors_raw(st):
            if res[0] is None and self.last_n0:
                res = (None, res[1] | set(['n0_refused']))
            self.last_n0 = False
            yield label, res

    def successors_raw(self, st):
        self.last_n0 = False
        for j in range(self.MAXOBJ):
            yield 'W%d' % j, self.op_write(st, j)
            yield 'O%d' % j, self.op_overwrite(st, j)
            yield 'D%d' % j, self.op_delete(st, j)
            if self.c['fault']:
                yield 'Q%d' % j, self.op_overwrite(st, j, fault=True)
        yield 'GI', self.op_issue(st)
        yield 'GB', self.op_batch(st)
        yield 'M', self.op_mount(st)
        for k in range(1, self.RING):
            yield 'R%d' % k, self.op_rollback(st, k)


def path_of(parent, st):
    labels = []
    while parent[st] is not None:
        st, label = parent[st]
        labels.append(label)
    return '.'.join(reversed(labels)) or '(mkfs)'


def run(name, nseg, segsz, ring, maxobj, b, depth):
    cfg = dict(BASE)
    cfg.update(ARMS[name])
    m = Model(cfg, nseg, segsz, ring, maxobj, b)
    start = m.norm(m.mkfs())
    parent = dict()
    parent[start] = None
    counts = dict((f, 0) for f in STATE_FLAGS + EDGE_FLAGS)
    gc_results = dict(done=0, declined=0, stuck=0, paused=0, cycle=0, cap=0, wasted=0)
    first = dict()
    cache = dict()
    frontier = [start]
    for d in range(depth + 1):
        nxt = []
        for st in frontier:
            flags = m.state_flags(st)
            res = m.gc_only(st, cache)
            gc_results[res] += 1
            if res in ('cycle', 'cap'):
                flags.add('livelock')
            for f in flags:
                counts[f] += 1
                if f not in first:
                    first[f] = path_of(parent, st)
            if d == depth:
                continue
            for label, (nst, eflags) in m.successors(st):
                for f in eflags:
                    counts[f] += 1
                    if f not in first:
                        first[f] = path_of(parent, st) + '.' + label
                if nst is None:
                    continue
                nst = m.norm(nst)
                if nst in parent:
                    continue
                parent[nst] = (st, label)
                nxt.append(nst)
        frontier = nxt
    print('E name=%s nseg=%d segsz=%d ring=%d maxobj=%d b=%d depth=%d states=%d %s gc_done=%d '
          'gc_declined=%d gc_stuck=%d gc_paused=%d gc_cycle=%d gc_cap=%d gc_wasted=%d' % (
              name, nseg, segsz, ring, maxobj, b, depth, len(parent),
              ' '.join('%s=%d' % (f, counts[f]) for f in STATE_FLAGS + EDGE_FLAGS),
              gc_results['done'], gc_results['declined'], gc_results['stuck'],
              gc_results['paused'], gc_results['cycle'], gc_results['cap'],
              gc_results['wasted']))
    for f in STATE_FLAGS + EDGE_FLAGS:
        if f in first:
            print('W name=%s flag=%s path=%s' % (name, f, first[f]))
    sys.stdout.flush()


def main():
    if len(sys.argv) == 10 and sys.argv[1] == 'trace':
        trace(sys.argv[2], *[int(x) for x in sys.argv[3:8]], sys.argv[8], int(sys.argv[9]))
        return
    if len(sys.argv) == 2 and sys.argv[1] == 'list':
        print(' '.join(ARMS))
        return
    if len(sys.argv) != 8 or sys.argv[1] not in ARMS:
        sys.stderr.write(__doc__)
        sys.exit(2)
    run(sys.argv[1], *[int(x) for x in sys.argv[2:]])


def show(m, st, label):
    cur = st[0][-1]
    segs = []
    for g in range(m.NSEG):
        cells = []
        for s in m.slots(g):
            r = cur[4][s]
            own = [j for j in range(m.MAXOBJ) if cur[1][j] is not None and cur[3][j] == s]
            if r is None:
                cells.append('.')
            elif r[1] is not None:
                cells.append('f' if m.free(st, cur[4], s) else 'd')
            elif own:
                cells.append('u%d' % own[0])
            elif s == cur[5]:
                cells.append('M')
            elif s == cur[6]:
                cells.append('I')
            elif s in cur[8]:
                cells.append('Q')
            else:
                cells.append('?')
        segs.append(' '.join(cells))
    print('T %-6s txg=%d [%s] intent=%s open=%s empty_rec=%d empty_occ=%d' % (
        label, cur[0], ' | '.join(segs), cur[7], sorted(st[2][0]),
        m.empty_count(st, cur[4], 'rec'), m.empty_count(st, cur[4], 'occ')))


def trace(name, nseg, segsz, ring, maxobj, b, path, gcsteps):
    cfg = dict(BASE)
    cfg.update(ARMS[name])
    m = Model(cfg, nseg, segsz, ring, maxobj, b)
    st = m.norm(m.mkfs())
    show(m, st, 'mkfs')
    for label in [x for x in path.split('.') if x]:
        for lab, (nst, flags) in m.successors(st):
            if lab == label:
                if nst is None:
                    print('T %s refused %s' % (label, sorted(flags)))
                    return
                st = m.norm(nst)
                show(m, st, label)
                break
    for _ in range(gcsteps):
        if st[0][-1][7] is None:
            nst, _ = m.op_issue(st)
            if nst is None:
                print('T GI declined')
                return
            st = m.norm(nst)
            show(m, st, 'GI')
        nst, _ = m.op_batch(st)
        if nst is None:
            print('T GB refused stuck=%s paused=%s' % (m.stuck, m.paused))
            return
        info = m.info
        st = m.norm(nst)
        show(m, st, 'GB')
        if info is not None:
            print('T   deleted pred=%s' % info[1])
            if info[1]:
                return


if __name__ == '__main__':
    main()

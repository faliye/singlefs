#!/usr/bin/env python3
# C143 round 1, attack leg (Opus): admin rollback x inode-number watermark, three arms.
# stdlib only.  Run from this directory:  python3 c143_model.py > c143_model.out
# Rules and their kb file:line are listed in c143-r1-opus-output.md section 2.
import itertools
import sys

R = 3                        # root-ring regions
DISK_OF_REGION = (0, 1, 0)   # first-version region ownership (D22 file line 1005)
H0 = 12                      # inode tree id of the first head
CRIT = {"A": "num", "AJ": "num", "C": "numbg", "CJ": "numbg"}


class Root:
    __slots__ = ("txg", "inst", "heads", "tree_wm", "table")

    def __init__(self, txg, inst, heads, tree_wm, table):
        self.txg, self.inst, self.heads = txg, inst, heads
        self.tree_wm, self.table = tree_wm, table
        # heads: tid -> (inode watermark, tuple of (ino, birthgen, oid))
        # table: instance table, inst -> (T_pub, is_rollback_row)


def root_key(r):
    return (r.txg, r.inst)


class Sim:
    """One pool history.  arm: A (B identical for stat 12), AJ, C, CJ."""

    def __init__(self, arm, S=4, warm="strict", sw_read="keep"):
        self.arm, self.S, self.warm, self.sw_read = arm, S, warm, sw_read
        zero = Root(0, 0, {}, 19, {})
        self.ring = {(r, 0): zero for r in range(R)}  # mkfs seeds gen-0 root in every region
        self.journal = []        # (inst, txg, Root): record durable before its root
        self.sb_inst = 0         # superblock instance code, never rolled back
        self.oid = 0
        self.faults = 0
        self.log = []
        self.reg = {}            # oid -> (tid, ino, bg) at first root publication
        self.conf = set()        # oids published while the instance was warm (fsync returned)
        self.epoch = {}          # oid -> rollbacks performed before its creation
        self.tree_pub = {}       # tree id -> set of clone events whose roots carried it
        self.clone_ev = {}       # tid -> clone event that created it (0 = first head)
        self.n_clone = 0
        self.last_clone = None
        self.rb_count = 0
        self.mounted = False
        self.heads, self.table, self.covered = {}, {}, set()
        self.inst = self.txg = self.inst_last_pub = 0
        self.tree_wm = 19

    def slot(self, t):
        return (t % R, (t // R) % self.S)

    def disk(self, t):
        return DISK_OF_REGION[t % R]

    def wms(self):
        return ",".join(f"{t}:{h[0]}" for t, h in sorted(self.heads.items()))

    def snap(self, t):
        return Root(t, self.inst,
                    {tid: (h[0], tuple(h[1])) for tid, h in self.heads.items()},
                    self.tree_wm, dict(self.table))

    def load(self, root):
        self.heads = {tid: [wm, list(objs)] for tid, (wm, objs) in root.heads.items()}
        self.tree_wm = root.tree_wm

    def new_inst(self):
        n = max([self.sb_inst] + [r.inst for r in self.ring.values()]) + 1
        self.sb_inst = n
        return n

    def write_root(self, root):
        self.ring[self.slot(root.txg)] = root
        self.covered.add(self.disk(root.txg))
        self.inst_last_pub = root.txg
        warm = self.covered >= {0, 1}
        for tid, (_, objs) in root.heads.items():
            self.tree_pub.setdefault(tid, set()).add(self.clone_ev.get(tid, 0))
            for ino, bg, oid in objs:
                if oid not in self.reg:
                    self.reg[oid] = (tid, ino, bg)
                if warm:
                    self.conf.add(oid)

    def publish(self, create=0, tid=H0, fail=False, crash_after_record=False):
        t = self.txg + 1
        made = []
        for _ in range(create):
            h = self.heads[tid]
            self.oid += 1
            h[1].append((h[0], t, self.oid))
            made.append(f"({tid},{h[0]},bg{t})")
            self.epoch[self.oid] = self.rb_count
            h[0] += 1
        self.txg = t
        self.journal.append((self.inst, t, self.snap(t)))
        what = " creates " + " ".join(made) if made else ""
        if crash_after_record:
            self.log.append(f"txg{t} i{self.inst}{what}: record durable, CRASH before root")
            self.mounted = False
            return
        if fail:
            self.faults += 1
            self.log.append(f"txg{t} i{self.inst}{what}: root write to disk{self.disk(t)} FAILS")
            self.switch(None)
            t = self.txg = t + 1
            self.journal.append((self.inst, t, self.snap(t)))
            what = " (re-sent)" if made else ""
        self.write_root(self.snap(t))
        self.log.append(f"txg{t} i{self.inst} root->disk{self.disk(t)}{what} wm={self.wms()}")
        if fail:
            self.warmup()

    def switch(self, reload_from):
        old = self.inst
        if not (old in self.table and self.table[old][1]):
            self.table[old] = (self.inst_last_pub, False)
        self.inst = self.new_inst()
        self.covered, self.inst_last_pub = set(), 0
        if reload_from is not None:          # A/B weak reading: switch recovery reloads R_old
            for tid, h in self.heads.items():
                if tid in reload_from.heads:
                    h[0] = reload_from.heads[tid][0]
        self.log.append(f"  switch -> i{self.inst}"
                        + (" (watermark reloaded from R_old)" if reload_from else ""))

    def warmup(self):
        if self.warm != "strict":
            return
        n = 0
        while not self.covered >= {0, 1}:
            self.publish(0)
            n += 1
            assert n <= R, "warm-up exceeded R pushes"

    def first_mount(self):
        self.inst = self.new_inst()
        self.mounted = True
        self.publish(0)
        self.publish(0)                      # warm-up: 2 empty publishes, format constant
        self.heads[H0] = [1, []]
        self.clone_ev[H0] = 0
        self.publish(1)                      # first transaction, txg 3: inode 1, watermark 2

    def readable(self, u):
        items = list(self.ring.items())
        bad, nf = set(), 0
        if u.startswith("d"):                # dX / dXnY: disk X unreadable, then Y newest others
            bad = {s for s, _ in items if DISK_OF_REGION[s[0]] == int(u[1])}
            nf, k = 1, (int(u[3:]) if "n" in u else 0)
        else:                                # none / newK: slots of the K newest distinct roots
            k = 0 if u == "none" else int(u[3:])
        if k:
            rest = {id(r): r for s, r in items if s not in bad}
            order = sorted(rest.values(), key=root_key, reverse=True)
            victims = {id(r) for r in order[:k]}
            bad |= {s for s, r in items if id(r) in victims}
            nf += k
        self.faults += nf
        seen, out = set(), []
        for s, r in items:
            if s not in bad and id(r) not in seen:
                seen.add(id(r))
                out.append(r)
        return out

    def mount(self, u="none"):
        """normal recovery: newest readable root by (txg, inst); replay same-instance,
        txg-contiguous records; new instance; rows; row-writing publish; warm-up."""
        rs = self.readable(u)
        if not rs:
            return False                     # nothing readable: mount impossible
        sel = max(rs, key=root_key)
        st, t = sel, sel.txg
        while True:
            nxt = [s for (i, tt, s) in self.journal if i == sel.inst and tt == t + 1]
            if not nxt:
                break
            st, t = nxt[-1], t + 1
        self.load(st)
        new = self.new_inst()
        tb = dict(st.table)
        for i in range(max(sel.inst, 1), new):
            if not (i in tb and tb[i][1]):
                tb[i] = (st.txg, False) if i == sel.inst else (0, False)
        self.table, self.inst, self.txg = tb, new, st.txg
        self.covered, self.inst_last_pub, self.mounted = set(), 0, True
        self.log.append(f"MOUNT u={u}: selects txg{sel.txg} i{sel.inst}, replays to "
                        f"txg{st.txg}, new i{new}, wm={self.wms()}")
        self.publish(0)
        self.warmup()
        return True

    def rollback(self, pick, u="none", sw=False):
        rs = self.readable(u)
        if not rs:
            return False                     # nothing readable: no rollback possible
        tb = max(rs, key=root_key).table
        cands = sorted((r for r in rs if tb.get(r.inst) is None or r.txg <= tb[r.inst][0]),
                       key=root_key, reverse=True)
        ro = pick(cands)
        if ro is None:
            return False
        self.load(ro)
        self.tree_wm = max([ro.tree_wm] + [r.tree_wm for r in rs])   # tree-id watermark rule
        rmax = max(r.txg for r in rs)
        tfirst = rmax + 1                                               # ring max + 1
        if self.arm == "CJ":
            tfirst = max([tfirst] + [tt + 1 for (_, tt, _) in self.journal])
        if self.arm in ("A", "AJ"):
            for tid, h in self.heads.items():
                ws = [r.heads[tid][0] for r in rs if tid in r.heads]
                if self.arm == "AJ":
                    ws += [s.heads[tid][0] for (_, tt, s) in self.journal
                           if tt > ro.txg and tid in s.heads]
                h[0] = max([h[0]] + ws)
        new = self.new_inst()
        tbl = dict(ro.table)
        if ro.inst >= 1 and not (ro.inst in tbl and tbl[ro.inst][1]):
            tbl[ro.inst] = (ro.txg, True)
        for i in range(ro.inst + 1, new):
            if not (i in tbl and tbl[i][1]):
                tbl[i] = (0, False)
        self.table, self.inst, self.txg = tbl, new, tfirst - 1
        self.covered, self.inst_last_pub, self.mounted = set(), 0, True
        self.rb_count += 1
        self.log.append(f"ROLLBACK u={u}: R_old=txg{ro.txg} i{ro.inst}; readable ring max "
                        f"txg{rmax}; first new root txg{tfirst} i{new}; wm={self.wms()}")
        if sw:
            self.faults += 1
            t = self.txg = self.txg + 1
            self.journal.append((self.inst, t, self.snap(t)))
            self.log.append(f"txg{t} i{self.inst}: rollback root write FAILS")
            self.switch(ro if (self.sw_read == "reload" and self.arm in ("A", "AJ")) else None)
        self.publish(0)
        self.warmup()
        return True

    def clone(self, tid):
        new = self.tree_wm
        self.tree_wm += 1
        self.n_clone += 1
        self.clone_ev[new] = self.n_clone
        self.heads[new] = [self.heads[tid][0], []]
        self.last_clone = new
        self.log.append(f"clone of {tid} -> tree {new} (wm {self.heads[new][0]})")
        self.publish(0)

    def destroy(self, tid):
        del self.heads[tid]
        self.log.append(f"destroy head {tid}")
        self.publish(0)

    def user(self, n, tid=H0):
        self.warmup()
        for _ in range(n):
            self.publish(1, tid)

    def do(self, op):
        k = op[0]
        if k == "p":
            self.publish(op[1])
        elif k == "f":
            self.publish(op[1], fail=True)
        elif k == "cr":
            self.publish(op[1], crash_after_record=True)
        elif k == "crash":
            self.mounted = False
            self.log.append("CRASH (nothing in flight)")
        elif k == "mount":
            return self.mount(op[1])
        elif k in ("rb", "rbt"):
            self.mounted = False
            if k == "rb":
                pick = (lambda c, kk=op[1]: c[kk - 1]
                        if kk <= len(c) and H0 in c[kk - 1].heads else None)
            else:
                pick = (lambda c, t=op[1]: next((r for r in c if r.txg == t), None))
            return self.rollback(pick, op[2], op[3])
        elif k == "u":
            self.user(op[1])
        elif k == "clone":
            self.clone(H0)
        elif k == "uc":
            self.user(op[1], self.last_clone)
        elif k == "destroy":
            self.destroy(op[1])
        else:
            raise ValueError(op)
        return True


def violations(sim, crit):
    """identities (per criterion) carried by >= 2 distinct root-published objects, at least
    one created after a rollback.  crit 'num' = (tree, ino); 'numbg' = (tree, ino, bg)."""
    by = {}
    for oid, (tid, ino, bg) in sim.reg.items():
        key = (tid, ino) if crit == "num" else (tid, ino, bg)
        by.setdefault(key, []).append(oid)
    out = []
    for key, oids in sorted(by.items()):
        if len(oids) > 1 and max(sim.epoch[o] for o in oids) >= 1:
            oids.sort()
            out.append((key, oids, oids[0] in sim.conf))
    return out


def tree_reuse(sim):
    return sorted(t for t, evs in sim.tree_pub.items() if len(evs) > 1)


def run(arm, warm, sw_read, script, S=4):
    s = Sim(arm, S, warm, sw_read)
    s.first_mount()
    for op in script:
        if s.do(op) is False:
            return None
    return s


ARMS = [("A", "keep"), ("A", "reload"), ("AJ", "keep"), ("C", "keep"), ("CJ", "keep")]
PRE = [("p", 0), ("p", 1), ("f", 0), ("f", 1)]
USET = ["none", "new1", "new2", "new3", "d0", "d1", "d0n1", "d1n1"]


def label(arm, rd):
    return arm + ("/reload" if rd == "reload" else "")


def fmt(script):
    return " ".join("".join(str(x) for x in op) for op in script)


def sweep1(L1=4):
    """single rollback after a prefix; per (warm, arm): fewest faults to a violation."""
    best, hist, patt, c_not_a = {}, {}, {}, []
    for warm in ("strict", "lenient"):
        for n in range(L1 + 1):
            for pre in itertools.product(PRE, repeat=n):
                for cr, kk, u, sw in itertools.product((0, 1), (1, 2, 3), USET, (0, 1)):
                    script = list(pre) + ([("cr", 1)] if cr else []) \
                        + [("rb", kk, u, sw), ("u", 2)]
                    hit = set()
                    for arm, rd in ARMS:
                        if rd == "reload" and not sw:
                            continue
                        s = run(arm, warm, rd, script)
                        if s is None:
                            break
                        lab, key = label(arm, rd), (warm, label(arm, rd))
                        cell = hist.setdefault(key, {}).setdefault(s.faults, [0, 0])
                        cell[0] += 1
                        v = violations(s, CRIT[arm])
                        if v:
                            hit.add(lab)
                            cell[1] += 1
                            cand = (s.faults, len(script), fmt(script))
                            if key not in best or cand < best[key][0]:
                                best[key] = (cand, s.log, v)
                    else:
                        pk = (warm, frozenset(hit), s.faults)
                        patt[pk] = patt.get(pk, 0) + 1
                        if "C" in hit and "A" not in hit:
                            c_not_a.append((warm, s.faults, fmt(script)))
    return best, hist, patt, c_not_a


def sweep2():
    """after a clean rollback: crash + remount with a read fault, a second rollback, a
    clone; plus a clone made in the abandoned timeline (tree-id watermark path)."""
    fams = []
    pres = [[], [("p", 1)], [("p", 1), ("p", 1)], [("f", 1)], [("p", 1), ("f", 1)]]
    for pre in pres:
        for kk in (1, 2):
            rb = [("rb", kk, "none", 0), ("u", 2)]
            for u2 in USET:
                fams.append(("rb_then_crash_mount",
                             pre + rb + [("crash",), ("mount", u2), ("u", 1)]))
            for kk2, u2 in itertools.product((1, 2, 3), USET):
                fams.append(("two_rollbacks", pre + rb + [("rb", kk2, u2, 0), ("u", 1)]))
            fams.append(("clone_after_rb", pre + rb + [("clone",), ("uc", 2)]))
        for kk, u in itertools.product((1, 2, 3), USET):
            fams.append(("clone_in_abandoned",
                         pre + [("clone",), ("uc", 1), ("rb", kk, u, 0), ("clone",), ("uc", 1)]))
    res = {}
    for warm in ("strict", "lenient"):
        for fam, script in fams:
            for arm in ("A", "AJ", "C", "CJ"):
                s = run(arm, warm, "keep", script)
                if s is None:
                    continue
                r = res.setdefault((warm, fam, arm), [0, 0, 0, None])
                r[0] += 1
                tr = tree_reuse(s)
                r[2] += bool(tr)
                v = violations(s, CRIT[arm])
                if v:
                    r[1] += 1
                    cand = (s.faults, len(script), fmt(script))
                    if r[3] is None or cand < r[3][0]:
                        r[3] = (cand, s.log, v, tr)
    return res


def sweep3():
    """question 3 (ring gap): a failed root write on R_old's slot keeps R_old in the ring
    after every later abandoned root that still carried the destroyed head H2 left it."""
    out = []
    for S, skip, n_obj in itertools.product((4, 5), (True, False), (1, 2)):
        for arm in ("A", "AJ", "C", "CJ"):
            s = Sim(arm, S, "strict", "keep")
            s.first_mount()
            s.clone(H0)                        # head H2, tree 19
            h2 = s.last_clone
            s.publish(0)
            t_old = s.txg                      # R_old
            for _ in range(n_obj):
                s.publish(1, h2)               # H2 publishes numbers in the abandoned span
            s.destroy(h2)
            while s.txg + 1 < t_old + R * S:
                s.publish(0)
            s.publish(0, fail=skip)            # lands on R_old's slot; failed write keeps it
            while s.txg + 1 < t_old + 2 * R * S:
                s.publish(0)
            uniq = {id(r): r for r in s.ring.values()}.values()
            in_ring = any(r.txg == t_old for r in uniq)
            carriers = sorted(r.txg for r in uniq if h2 in r.heads)
            ok = s.do(("rbt", t_old, "none", 0))
            if ok:
                s.user(1, h2)
            v = violations(s, CRIT[arm]) if ok else None
            out.append((S, skip, n_obj, arm, t_old, in_ring, carriers, bool(ok), s.faults, v,
                        s.log if (ok and v) else None))
    return out


CAP = 3


def l1_rebuild(vers, units, i_now, join_bg=False):
    """level-1 scan rebuild with the instance table unreadable: every unit of an instance
    older than i_now counts as published (D18 file line 879); per container identity keep
    the newest version by (birth, inst); extents joined to records by inode number
    (join_bg False: the extent key carries no birth generation) or by (number, birthgen)."""
    cur = {}
    for ident, birth, inst, recs in vers:
        if inst < i_now and (ident not in cur or (birth, inst) > cur[ident][0]):
            cur[ident] = ((birth, inst), recs)
    leaves = sorted((ident[2], ident[3], recs) for ident, (_, recs) in cur.items())
    i94 = sum(1 for (c1, _, r1), (c2, _, _r2) in zip(leaves, leaves[1:])
              if not (c1 < c2 and max(i for i, _ in r1) < c2))
    rec = {}
    for _, _, recs in leaves:
        for ino, bg in recs:
            rec.setdefault(ino, []).append(bg)
    ext = {}
    for ino, bg, off, inst, n in units:
        k = (ino, bg, off) if join_bg else (ino, off)
        if inst < i_now and (k not in ext or (inst, n) > ext[k][0]):
            ext[k] = ((inst, n), bg)
    mixed = orphan = 0
    for k, (_, bg) in ext.items():
        if k[0] not in rec or (join_bg and bg not in rec[k[0]]):
            orphan += 1
        elif bg not in rec[k[0]]:
            mixed += 1
    return {"leaves(cno:[inos])": [(c, [i for i, _ in r]) for c, _, r in leaves],
            "I-9.4_violations": i94,
            "duplicate_keys": sum(1 for v in rec.values() if len(v) > 1),
            "mixed_extents": mixed, "orphan_extents": orphan}


def rebuild_case(kind, join_bg=False):
    """kind: rollback_C / rollback_AB (admin rollback to txg 4) or crash (plain crash at
    txg 5, any arm).  Container capacity CAP; rightmost-leaf insertion, split at the end."""
    vers, units, leaves, n = [], [], [], [0]

    def create(ino, bg, inst, offs):
        if leaves and len(leaves[-1][1]) < CAP:
            ident, recs = leaves[-1]
            leaves[-1] = (ident, recs + ((ino, bg),))
        else:
            leaves.append(((H0, 2, ino, bg), ((ino, bg),)))
        vers.append((leaves[-1][0], bg, inst, leaves[-1][1]))
        for off in offs:
            n[0] += 1
            units.append((ino, bg, off, inst, n[0]))

    create(1, 3, 1, (0,))
    create(2, 4, 1, (0,))                  # txg 4: R_old / last good root
    kept = list(leaves)
    create(3, 5, 1, (0, 1))                # txg 5: published (rollback) or lost (crash)
    if kind == "crash":
        leaves[:] = kept                   # recovery from txg 4, instance 2
        create(3, 8, 2, (0,))              # row publish 5, warm-up 6-7, user txg 8
    else:
        create(4, 6, 1, (0, 1))            # abandoned, published txg 6 (split)
        leaves[:] = kept                   # rollback to txg 4, instance 2
        base = 3 if kind == "rollback_C" else 5
        create(base, 9, 2, (0,))           # rollback root 7, warm-up 8, user txg 9
        create(base + 1, 10, 2, (0,))
    return l1_rebuild(vers, units, 3, join_bg)


def main():
    out = []
    p = out.append
    p("# c143_model.py output (stdlib only, deterministic)")
    p("# arm A == arm B for stat 12 in this model (same max over the same readable-root set)")
    best, hist, patt, c_not_a = sweep1(6)
    p("")
    p("## sweep1: one rollback after a prefix of <= 6 publishes (p0 p1 f0 f1), optional")
    p("##         crash-after-record, rollback kk in 1..3, read fault u, switch-in-rb sw")
    for key in sorted(hist):
        h = hist[key]
        minf = min((f for f, v in h.items() if v[1]), default=None)
        p(f"name=sweep1 warm={key[0]} arm={key[1]} runs={sum(v[0] for v in h.values())} "
          f"violating={sum(v[1] for v in h.values())} min_faults={minf} "
          "by_faults=" + ";".join(f"{f}:{v[1]}/{v[0]}" for f, v in sorted(h.items())))
    for key in sorted(best):
        (f, _ln, sc), log, v = best[key]
        p(f"example warm={key[0]} arm={key[1]} faults={f} script=[{sc}]")
        p(f"  violations={v}")
        for line in log:
            p("    " + line)
    p("")
    p("## sweep1 cross-arm pattern: arms violated (own criterion), by fault count")
    for (warm, hit, fl), cnt in sorted(patt.items(),
                                        key=lambda kv: (kv[0][0], kv[0][2], sorted(kv[0][1]))):
        p(f"name=pattern warm={warm} faults={fl} arms={'+'.join(sorted(hit)) or '-'} "
          f"scripts={cnt}")
    p(f"name=C_hit_but_A_not count={len(c_not_a)} first={c_not_a[:3]}")
    res2 = sweep2()
    p("")
    p("## sweep2: after a clean rollback, and clone made in the abandoned timeline")
    for key in sorted(res2):
        runs, vio, trr, ex = res2[key]
        p(f"name=sweep2 warm={key[0]} family={key[1]} arm={key[2]} runs={runs} "
          f"violating={vio} tree_id_reused={trr} min_example={ex[0] if ex else None}")
    for key in sorted(res2):
        ex = res2[key][3]
        if ex and key[0] == "strict":
            (f, _ln, sc), log, v, tr = ex
            p(f"example2 family={key[1]} arm={key[2]} faults={f} script=[{sc}] "
              f"tree_reuse={tr}")
            p(f"  violations={v}")
            for line in log:
                p("    " + line)
    p("")
    p("## sweep3 (question 3): R_old kept in the ring by a failed write on its own slot")
    for row in sweep3():
        S, skip, n_obj, arm, t_old, in_ring, carriers, ok, faults, v, log = row
        p(f"name=sweep3 S={S} skip={skip} n_obj={n_obj} arm={arm} R_old=txg{t_old} "
          f"R_old_in_ring={in_ring} ring_roots_carrying_H2={carriers} rollback_ok={ok} "
          f"faults={faults} violations={v}")
        if log and S == 4 and n_obj == 1:
            for line in log:
                p("    " + line)
    p("")
    p("## rebuild: level-1 scan rebuild with the instance table unreadable")
    for kind in ("crash", "rollback_AB", "rollback_C"):
        for jb in (False, True):
            p(f"name=rebuild case={kind} join_by_birthgen={jb} "
              f"result={rebuild_case(kind, jb)}")
    p(f"END c143_model lines_before_this={len(out)}")
    sys.stdout.write("\n".join(out) + "\n")


if __name__ == "__main__":
    main()

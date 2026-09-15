#!/usr/bin/env python3
# C143 round 2, attack leg (Opus): does the carrier of AJ / CJ itself fall back?
# stdlib only, deterministic.  Run from this directory:
#     python3 -B c143r2_model.py > c143r2_model.out
# Round-1 model (read only, not imported): ../c143-r1-opus-model/c143_model.py.  New here:
#   * journal ring of J record slots, one copy on each of the two disks (D22 file line 229);
#     the record with counter c sits in slot (c - 1) mod J (D23 file line 697); a new instance
#     writes from prefix end + 1 (D23 file line 1235), so a rollback writes over abandoned records;
#   * read faults on single record copies, on whole disks, on root slots; counted per slot (fs)
#     and per contiguous range (fr);
#   * the state a record's new-root segment points to can be gone: units of a publish that never
#     got a root, and of roots that left the ring or were unreadable at a rollback, are handed out
#     again at once (worst case for AJ; the shadow ledger only guards readable ring roots);
#   * rollback attempt that crashes after its record; mount / second rollback / switch after a
#     rollback with record faults (the AJ2 / CJ2 cell); a geometry table for the margin of C.
import itertools
import sys

R = 3
H0 = 12
BIG_J = 1 << 30
CRIT = {"A": "num", "AJ": "num", "AJ2": "num", "G": "num",
        "C": "numbg", "CJ": "numbg", "CJ2": "numbg"}
NO_FAULT = (None, 0, 0, (), ())   # (down disk, newest root slots, newest record copies,
#                                    root txgs to hide, record txgs to hide)


class Root:
    __slots__ = ("txg", "inst", "heads", "tree_wm", "table", "ctr", "sid", "gwm")

    def __init__(self, txg, inst, heads, tree_wm, table, ctr, sid, gwm):
        self.txg, self.inst, self.heads, self.tree_wm = txg, inst, heads, tree_wm
        self.table, self.ctr, self.sid, self.gwm = table, ctr, sid, gwm


class State:
    """what one publish's new-root segment points to (tree table + accounting tree)."""
    __slots__ = ("heads", "gwm", "tree_wm", "txg", "inst", "rooted", "alive")

    def __init__(self, heads, gwm, tree_wm, txg, inst):
        self.heads, self.gwm, self.tree_wm, self.txg, self.inst = heads, gwm, tree_wm, txg, inst
        self.rooted, self.alive = False, True


class Sim:
    """one pool history.  arm: A (= B for stat 12), AJ, AJ2, G, C, CJ, CJ2."""

    def __init__(self, arm, S=8, J=BIG_J, warm="strict", sw_read="keep", rbp="rold",
                 reuse=True, own=(0, 1, 0)):
        self.arm, self.S, self.J, self.warm, self.sw_read = arm, S, J, warm, sw_read
        self.rbp, self.reuse, self.own = rbp, reuse, own
        self.states = [State({}, 0, 19, 0, 0)]
        self.states[0].rooted = True
        zero = Root(0, 0, {}, 19, {}, 0, 0, 0)
        self.ring = {(r, 0): zero for r in range(R)}   # mkfs seeds gen-0 root in every region
        self.allroots = [zero]
        self.jr = {}             # journal slot -> (inst, ctr, txg, sid, last_of_publish, gwm)
        self.ctr = 1             # counter of the next record to write
        self.sb_inst = 0
        self.oid = 0
        self.fs = self.fr = self.wf = 0
        self.targets = set()     # distinct physical fault targets over the whole history
        self.log = []
        self.reg, self.conf, self.epoch = {}, set(), {}
        self.tree_pub, self.clone_ev = {}, {}
        self.n_clone, self.last_clone, self.rb_count = 0, None, 0
        self.abandoned = set()   # (txg, inst) of roots made invalid by a completed rollback
        self.c332 = False        # a normal mount selected a root on an abandoned timeline
        self.mark = None
        self.heads, self.table, self.covered = {}, {}, set()
        self.inst = self.txg = self.inst_last_pub = 0
        self.tree_wm, self.gwm = 19, 0

    def slot(self, t):
        return (t % R, (t // R) % self.S)

    def disk(self, t):
        return self.own[t % R]

    def wms(self):
        return ",".join(f"{t}:{h[0]}" for t, h in sorted(self.heads.items()))

    def new_state(self, t):
        self.gwm = max([self.gwm] + [h[0] for h in self.heads.values()])
        snap = {tid: (h[0], tuple(h[1])) for tid, h in self.heads.items()}
        self.states.append(State(snap, self.gwm, self.tree_wm, t, self.inst))
        return len(self.states) - 1

    def write_records(self, t, sid, nrec):
        for k in range(nrec):
            c = self.ctr
            self.ctr += 1
            self.jr[(c - 1) % self.J] = (self.inst, c, t, sid, k == nrec - 1, self.gwm)
        return self.ctr - 1

    @staticmethod
    def valid(root, table):
        row = table.get(root.inst)
        return row is None or root.txg <= row[0]

    def reuse_pass(self):
        """worst case for AJ: every unit the allocator may hand out is handed out now."""
        if not self.reuse:
            return
        in_ring = {r.sid for r in self.ring.values()}
        oldest = min((r.txg for r in self.ring.values() if self.valid(r, self.table)),
                     default=0)
        for sid, st in enumerate(self.states):
            if not st.alive or sid in in_ring:
                continue
            if not st.rooted:
                if st.inst < self.inst:
                    st.alive = False        # orphan of a publish that never got a root
            elif (st.txg, st.inst) in self.abandoned or st.txg < oldest:
                st.alive = False            # its root left the ring and nothing pins it

    def write_root(self, root):
        self.ring[self.slot(root.txg)] = root
        self.allroots.append(root)
        self.states[root.sid].rooted = True
        self.covered.add(self.disk(root.txg))
        self.inst_last_pub = root.txg
        warm = len(self.covered) >= 2
        for tid, (_, objs) in root.heads.items():
            self.tree_pub.setdefault(tid, set()).add(self.clone_ev.get(tid, 0))
            for ino, bg, oid in objs:
                if oid not in self.reg:
                    self.reg[oid] = (tid, ino, bg)
                if warm:
                    self.conf.add(oid)

    def publish(self, create=0, tid=H0, fail=False, crash_after_record=False, nrec=1):
        self.reuse_pass()
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
        sid = self.new_state(t)
        last = self.write_records(t, sid, nrec)
        what = " creates " + " ".join(made) if made else ""
        if crash_after_record:
            self.log.append(f"txg{t} i{self.inst}{what}: record c{last} durable, CRASH before root")
            return
        if fail:
            self.wf += 1
            self.recount()
            self.log.append(f"txg{t} i{self.inst}{what}: record c{last}; root write to "
                            f"disk{self.disk(t)} FAILS")
            self.switch(None)
            self.reuse_pass()
            t = self.txg = t + 1
            sid = self.new_state(t)
            last = self.write_records(t, sid, nrec)
            what = " (re-sent)" if made else ""
        self.write_root(Root(t, self.inst, self.states[sid].heads, self.tree_wm,
                             dict(self.table), last, sid, self.gwm))
        self.log.append(f"txg{t} i{self.inst} c{last} root->disk{self.disk(t)}{what} "
                        f"wm={self.wms()}")
        if fail:
            self.warmup()

    def switch(self, reload_from):
        old = self.inst
        if not (old in self.table and self.table[old][1]):
            self.table[old] = (self.inst_last_pub, False)
        self.inst = self.new_inst()
        self.covered, self.inst_last_pub = set(), 0
        if reload_from is not None:          # A weak reading: switch recovery reloads R_old
            for tid, h in self.heads.items():
                if tid in reload_from.heads:
                    h[0] = reload_from.heads[tid][0]
        self.log.append(f"  switch -> i{self.inst}"
                        + (" (watermark reloaded from R_old)" if reload_from else ""))

    def new_inst(self):
        n = max([self.sb_inst] + [r.inst for r in self.ring.values()]) + 1
        self.sb_inst = n
        return n

    def warmup(self):
        if self.warm != "strict":
            return
        n = 0
        while len(self.covered) < 2:
            self.publish(0)
            n += 1
            assert n <= R, "warm-up exceeded R pushes"

    def first_mount(self):
        self.inst = self.new_inst()
        self.publish(0)
        self.publish(0)                      # warm-up: 2 empty publishes, format constant
        self.heads[H0] = [1, []]
        self.clone_ev[H0] = 0
        self.publish(1)                      # first transaction, txg 3: inode 1, watermark 2

    def recount(self):
        """faults = root-write failures + distinct targets hidden in any scan of the history:
        a down disk, a root slot, a record slot on one disk (fs); record slots of one disk that
        are contiguous count once (fr)."""
        dr = sum(1 for t in self.targets if t[0] != "j")
        js = {(t[1], t[2]) for t in self.targets if t[0] == "j"}
        runs = sum(1 for slot, dk in js if ((slot - 1) % self.J, dk) not in js)
        self.fs = self.wf + dr + len(js)
        self.fr = self.wf + dr + runs

    def scan(self, u):
        """one read pass over the root ring and the journal ring."""
        down, kr, mr, rtx, ctx = u
        items = list(self.ring.items())
        bad = {s for s, _ in items if down is not None and self.own[s[0]] == down}
        rest = {id(r): r for s, r in items if s not in bad}
        order = sorted(rest.values(), key=lambda r: (r.txg, r.inst), reverse=True)
        victims = {id(r) for r in order[:kr]} | {id(r) for r in rest.values() if r.txg in rtx}
        bad |= {s for s, r in items if id(r) in victims}
        roots, seen = [], set()
        for s, r in items:
            if s not in bad and id(r) not in seen:
                seen.add(id(r))
                roots.append(r)
        if down is not None:
            self.targets.add(("d", down))
        for r in rest.values():
            if id(r) in victims:
                self.targets.add(("r", self.slot(r.txg)))
        other = (0, 1) if down is None else (1 - down,)   # copies still to hide per record
        recs = sorted(self.jr.items(), key=lambda kv: kv[1][1], reverse=True)
        hid = {js for js, _ in recs[:mr]} | {js for js, rec in recs if rec[2] in ctx}
        for js in hid:
            for dk in other:
                self.targets.add(("j", js, dk))
        self.recount()
        return roots, [rec for js, rec in recs if js not in hid]

    def mount(self, u=NO_FAULT):
        """normal recovery: newest readable root by (txg, inst); replay whole publishes of the
        same instance with contiguous counters whose units verify; new instance writes from
        prefix end + 1; rows; row-writing publish; warm-up."""
        roots, recs = self.scan(u)
        if not roots:
            return False
        sel = max(roots, key=lambda r: (r.txg, r.inst))
        if (sel.txg, sel.inst) in self.abandoned:
            self.c332 = True
        by = {(rec[0], rec[1]): rec for rec in recs}
        heads, gwm, tw, t, c = sel.heads, sel.gwm, sel.tree_wm, sel.txg, sel.ctr
        while True:
            k, last = c, None
            while True:
                rec = by.get((sel.inst, k + 1))
                if rec is None:
                    break
                k += 1
                if rec[4]:
                    last = rec
                    break
            if last is None or not self.states[last[3]].alive:
                break
            st = self.states[last[3]]
            heads, gwm, tw, t, c = st.heads, st.gwm, st.tree_wm, st.txg, k
        self.heads = {tid: [wm, list(objs)] for tid, (wm, objs) in heads.items()}
        self.gwm, self.tree_wm = gwm, tw
        if self.arm == "AJ2":                # attacker tightening: journal max at every mount
            for tid, h in self.heads.items():
                h[0] = max([h[0]] + [self.states[rec[3]].heads[tid][0] for rec in recs
                                     if self.states[rec[3]].alive
                                     and tid in self.states[rec[3]].heads])
        if self.arm == "G":                  # clue arm: pool-wide number watermark in headers
            g = max([r.gwm for r in roots] + [rec[5] for rec in recs])
            for h in self.heads.values():
                h[0] = max(h[0], g)
            self.gwm = max(self.gwm, g)
        start = t
        if self.arm == "CJ2":                # attacker tightening: journal txg at every mount
            start = max([t] + [r.txg for r in roots] + [rec[2] for rec in recs])
        new = self.new_inst()
        tb = dict(sel.table)
        for i in range(max(sel.inst, 1), new):
            if not (i in tb and tb[i][1]):
                tb[i] = (t, False) if i == sel.inst else (0, False)
        self.table, self.inst, self.txg, self.ctr = tb, new, start, c + 1
        self.covered, self.inst_last_pub = set(), 0
        self.log.append(f"MOUNT u={fmtu(u)}: selects txg{sel.txg} i{sel.inst}, replays to txg{t} "
                        f"(prefix end c{c}), new i{new} from txg{start + 1}, wm={self.wms()}"
                        + (" [selected root is on an abandoned timeline]"
                           if (sel.txg, sel.inst) in self.abandoned else ""))
        self.publish(0)
        self.warmup()
        return True

    def rollback(self, pick, u=NO_FAULT, sw=False, crash_attempt=False):
        roots, recs = self.scan(u)
        if not roots:
            return False
        tb = max(roots, key=lambda r: (r.txg, r.inst)).table
        cands = sorted((r for r in roots if self.valid(r, tb)),
                       key=lambda r: (r.txg, r.inst), reverse=True)
        ro = pick(cands)
        if ro is None:
            return False
        self.heads = {tid: [wm, list(objs)] for tid, (wm, objs) in ro.heads.items()}
        self.tree_wm = max([ro.tree_wm] + [r.tree_wm for r in roots])   # today's tree-id rule
        self.gwm = ro.gwm
        rmax = max(r.txg for r in roots)
        tfirst = rmax + 1                                                # ring max + 1
        if self.arm in ("CJ", "CJ2"):
            tfirst = max([tfirst] + [rec[2] + 1 for rec in recs])
        if self.arm in ("A", "AJ", "AJ2"):
            for tid, h in self.heads.items():
                ws = [r.heads[tid][0] for r in roots if tid in r.heads]
                if self.arm != "A":
                    ws += [self.states[rec[3]].heads[tid][0] for rec in recs
                           if rec[2] > ro.txg and self.states[rec[3]].alive
                           and tid in self.states[rec[3]].heads]
                h[0] = max([h[0]] + ws)
        if self.arm == "G":
            g = max([r.gwm for r in roots] + [rec[5] for rec in recs])
            for h in self.heads.values():
                h[0] = max(h[0], g)
            self.gwm = max(self.gwm, g)
        new = self.new_inst()
        tbl = dict(ro.table)
        if ro.inst >= 1 and not (ro.inst in tbl and tbl[ro.inst][1]):
            tbl[ro.inst] = (ro.txg, True)
        for i in range(ro.inst + 1, new):
            if not (i in tbl and tbl[i][1]):
                tbl[i] = (0, False)
        c = ro.ctr
        if self.rbp == "scan":               # other reading of "prefix end": whole readable chain
            have = {(rec[0], rec[1]) for rec in recs}
            while (ro.inst, c + 1) in have:
                c += 1
        if self.reuse:                       # shadow ledger guards readable ring roots only
            readable = {id(r) for r in roots}
            ring_ids = {id(r) for r in self.ring.values()}
            for r in self.allroots:
                if not self.valid(r, tbl) and (id(r) not in ring_ids or id(r) not in readable):
                    self.states[r.sid].alive = False
        self.table, self.inst, self.txg, self.ctr = tbl, new, tfirst - 1, c + 1
        self.covered, self.inst_last_pub = set(), 0
        self.log.append(f"ROLLBACK{' ATTEMPT' if crash_attempt else ''} u={fmtu(u)}: R_old=txg"
                        f"{ro.txg} i{ro.inst}; readable ring max txg{rmax}; first new root "
                        f"txg{tfirst} i{new}; records from c{c + 1}; wm={self.wms()}")
        if crash_attempt:
            self.publish(0, crash_after_record=True)
            return True
        self.rb_count += 1
        for r in self.allroots:
            if not self.valid(r, tbl):
                self.abandoned.add((r.txg, r.inst))
        if sw:
            self.wf += 1
            self.recount()
            t = self.txg = self.txg + 1
            sid = self.new_state(t)
            self.write_records(t, sid, 1)
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

    def user(self, n, tid=H0, nrec=1):
        if tid not in self.heads:          # the mounted root predates this head: script invalid
            return False
        self.warmup()
        for _ in range(n):
            self.publish(1, tid, nrec=nrec)
        return True

    def rel(self, u):
        down, kr, mr, rd, cd = u
        return (down, kr, mr, tuple(self.mark + d for d in rd), tuple(self.mark + d for d in cd))

    def do(self, op):
        k = op[0]
        if k == "p":
            self.publish(op[1])
        elif k == "pn":
            self.publish(op[1], nrec=op[2])
        elif k == "f":
            self.publish(op[1], fail=True)
        elif k == "cr":
            self.publish(op[1], crash_after_record=True)
        elif k == "crash":
            self.log.append("CRASH (nothing in flight)")
        elif k == "mount":
            return self.mount(op[1])
        elif k == "mark":
            self.mark = self.txg
        elif k == "rb":
            return self.rollback(lambda c, kk=op[1]: c[kk - 1]
                                 if kk <= len(c) and H0 in c[kk - 1].heads else None,
                                 op[2], op[3])
        elif k in ("rbm", "rbx"):
            pick = (lambda c, t=self.mark: next((r for r in c if r.txg == t), None))
            if k == "rbm":
                return self.rollback(pick, self.rel(op[1]), op[2])
            return self.rollback(pick, self.rel(op[1]), False, crash_attempt=True)
        elif k == "u":
            return self.user(op[1])
        elif k == "clone":
            if H0 not in self.heads:
                return False
            self.clone(H0)
        elif k == "ucx":
            return self.user(op[1], self.last_clone)
        elif k == "destroy":
            self.destroy(self.last_clone)
        else:
            raise ValueError(op)
        return True


def fmtu(u):
    down, kr, mr, rtx, ctx = u
    parts = ([f"d{down}"] if down is not None else []) + ([f"r{kr}"] if kr else []) \
        + ([f"j{mr}"] if mr else []) + (["rt" + "/".join(map(str, rtx))] if rtx else []) \
        + (["jt" + "/".join(map(str, ctx))] if ctx else [])
    return "+".join(parts) or "none"


def fmt(script):
    return " ".join(op[0] + ("(" + ",".join(fmtu(x) if isinstance(x, tuple) else str(x)
                                            for x in op[1:]) + ")" if len(op) > 1 else "")
                    for op in script)


def violations(sim, crit, need_rb=True):
    """identities (per criterion) carried by >= 2 distinct root-published objects; with need_rb,
    at least one of them created after a rollback.  crit num = (tree, ino); numbg adds bg."""
    by = {}
    for oid, (tid, ino, bg) in sim.reg.items():
        key = (tid, ino) if crit == "num" else (tid, ino, bg)
        by.setdefault(key, []).append(oid)
    out = []
    for key, oids in sorted(by.items()):
        if len(oids) > 1 and (not need_rb or max(sim.epoch[o] for o in oids) >= 1):
            oids.sort()
            out.append((key, oids, oids[0] in sim.conf))
    return out


def run(arm, script, **kw):
    s = Sim(arm, **kw)
    s.first_mount()
    for op in script:
        if s.do(op) is False:
            return None
    return s


KEY_PAIRS = [("C", "A"), ("CJ", "AJ"), ("CJ", "C"), ("AJ", "CJ"), ("AJ", "C"), ("A", "C"),
             ("CJ2", "AJ2"), ("AJ2", "CJ2"), ("CJ2", "C"), ("G", "CJ"), ("CJ", "G"),
             ("AJ", "G"), ("G", "AJ")]


class Tally:
    """per arm: runs, violating runs, fewest faults (per slot, per range), cross-arm pairs."""

    def __init__(self, arms):
        self.arms = arms
        self.runs = dict.fromkeys(arms, 0)
        self.viol = dict.fromkeys(arms, 0)
        self.byf = {a: {} for a in arms}
        self.best = {}
        self.pairs = {}
        self.c332 = {a: [0, 0] for a in arms}
        self.invalid = 0

    def add(self, results, script, need_rb=True):
        hit = set()
        for a, s in results.items():
            if s is None:
                continue
            v = violations(s, CRIT[a], need_rb)
            self.runs[a] += 1
            cell = self.byf[a].setdefault(s.fs, [0, 0])
            cell[0] += 1
            if not v:
                continue
            hit.add(a)
            self.viol[a] += 1
            cell[1] += 1
            self.c332[a][0 if s.c332 else 1] += 1
            for key, val in (("fs", s.fs), ("fr", s.fr)):
                cand = (val, s.fs + s.fr, len(script), fmt(script))
                b = self.best.get((a, key))
                if b is None or cand < b[0]:
                    self.best[(a, key)] = (cand, list(s.log), v)
        if all(s is None for s in results.values()):
            self.invalid += 1
        for x in results:
            for y in results:
                if x != y and results[x] is not None and results[y] is not None \
                        and x in hit and y not in hit:
                    self.pairs[(x, y)] = self.pairs.get((x, y), 0) + 1

    def dump(self, p, name, examples=True, max_log=24):
        p(f"name={name} scripts_invalid_for_every_arm={self.invalid}")
        for a in self.arms:
            f_fs = min((f for f, c in self.byf[a].items() if c[1]), default=None)
            b = self.best.get((a, "fr"))
            p(f"name={name} arm={a} runs={self.runs[a]} violating={self.viol[a]} "
              f"min_fs={f_fs} min_fr={b[0][0] if b else None} "
              f"viol_mount_on_abandoned={self.c332[a][0]} viol_other={self.c332[a][1]} "
              "by_fs=" + ";".join(f"{f}:{c[1]}/{c[0]}" for f, c in sorted(self.byf[a].items())))
        shown = set()
        for x, y in KEY_PAIRS:
            if x in self.arms and y in self.arms:
                shown.add((x, y))
                p(f"name={name} hit={x} not_hit={y} scripts={self.pairs.get((x, y), 0)}")
        for (x, y), n in sorted(self.pairs.items()):
            if (x, y) not in shown:
                p(f"name={name} hit={x} not_hit={y} scripts={n}")
        if not examples:
            return
        for a in self.arms:
            seen = set()
            for key in ("fs", "fr"):
                b = self.best.get((a, key))
                if b is None or b[0][3] in seen:
                    continue
                seen.add(b[0][3])
                (val, _t, _l, sc), log, v = b
                p(f"example name={name} arm={a} min_by={key} {key}={val} script=[{sc}]")
                p(f"  violations={v}")
                for line in log[-max_log:]:
                    p("    " + line)


ARMS_RB = ("A", "AJ", "G", "C", "CJ")


def s1_destroyed_head(p):
    """U1: a head destroyed in the abandoned timeline; hide roots / record copies after R_old."""
    offs = (1, 2, 3, 4)
    subsets = [()] + [(o,) for o in offs] + list(itertools.combinations(offs, 2))
    faults = [(down, 0, 0, rd, cd) for down in (None, 0, 1) for rd in subsets for cd in subsets]
    t = Tally(ARMS_RB)
    for a, b, c in itertools.product(range(3), range(3), range(4)):
        for u in faults:
            script = [("p", 0)] * a + [("clone",)] + [("p", 0)] * b \
                + [("mark",), ("ucx", 1), ("destroy",)] + [("p", 0)] * c \
                + [("rbm", u, 0), ("ucx", 1)]
            t.add({arm: run(arm, script) for arm in ARMS_RB}, script)
    t.dump(p, "s1_destroyed_head")


PRE = [("p", 0), ("p", 1), ("f", 0), ("f", 1)]


def s2_newest(p, L=5):
    """U1: r1's single-rollback space plus record faults: down disk, newest root slots, newest
    record copies on the other disk(s).  Strict warm-up for all arms; lenient only as control."""
    faults = [(down, kr, mr, (), ()) for down in (None, 0, 1) for kr in range(4)
              for mr in range(6)]
    t = Tally(ARMS_RB)
    tl = Tally(("A", "C", "CJ"))
    for n in range(L + 1):
        for pre in itertools.product(PRE, repeat=n):
            for kk in (1, 2):
                for u in faults:
                    script = list(pre) + [("rb", kk, u, 0), ("u", 2)]
                    t.add({arm: run(arm, script) for arm in ARMS_RB}, script)
                    if n <= 3:
                        tl.add({arm: run(arm, script, warm="lenient") for arm in tl.arms},
                               script)
    t.dump(p, "s2_newest_strict")
    tl.dump(p, "s2_newest_lenient")


def s3_wrap(p):
    """U1: journal ring overwritten inside one rollback window.  R_old kept by a failed root
    write on its own slot once per lap (H3); head destroyed after one object; no read fault."""
    S = 4
    for nrec in (1, 4):
        for J in (BIG_J, 12, 24, 48, 96, 192):
            for arm in ARMS_RB:
                first = None
                trail = []
                for laps in range(0, 10):
                    s = Sim(arm, S=S, J=J)
                    s.first_mount()
                    s.clone(H0)
                    s.publish(0)
                    s.mark = s.txg
                    s.user(1, s.last_clone)
                    c_h = s.ctr - 1
                    s.destroy(s.last_clone)
                    for _ in range(laps):
                        while (s.txg + 1) % (R * S) != s.mark % (R * S):
                            s.publish(0, nrec=nrec)
                        s.publish(0, fail=True, nrec=nrec)
                    ctr_rb = s.ctr
                    if not s.do(("rbm", NO_FAULT, 0)):
                        trail.append(f"{laps}:norb")
                        continue
                    s.user(1, s.last_clone)
                    v = violations(s, CRIT[arm])
                    trail.append(f"{laps}:{'V' if v else '-'}(c_rb={ctr_rb})")
                    if v and first is None:
                        first = (laps, s.fs, c_h, ctr_rb, v)
                p(f"name=s3_wrap S={S} nrec={nrec} J={'big' if J == BIG_J else J} arm={arm} "
                  f"first_violation={'none<=9' if first is None else first[:4]} "
                  f"record_of_H_at=c{c_h} trail={' '.join(trail)}")


def s4_crash_redo(p):
    """U1: the rollback attempt crashes after its record (it lands on prefix end + 1), a normal
    mount runs, then the rollback is redone.  A target hidden at both scans is counted once."""
    subsets1 = [(), (1,), (2,)]
    for rbp in ("rold", "scan"):
        for reuse in (True, False):
            for when in ("redo_only", "attempt_and_redo"):
                t = Tally(ARMS_RB)
                for a, b, c, d in itertools.product(range(2), range(2), range(3), range(3)):
                    for down in (None, 0, 1):
                        for rd in subsets1:
                            for cd in subsets1:
                                u = (down, 0, 0, rd, cd)
                                ua = u if when == "attempt_and_redo" else NO_FAULT
                                script = [("p", 0)] * a + [("clone",)] + [("p", 0)] * b \
                                    + [("mark",), ("ucx", 1), ("destroy",)] + [("p", 0)] * c \
                                    + [("rbx", ua), ("mount", NO_FAULT)] + [("p", 0)] * d \
                                    + [("rbm", u, 0), ("ucx", 1)]
                                t.add({arm: run(arm, script, rbp=rbp, reuse=reuse)
                                       for arm in ARMS_RB}, script)
                t.dump(p, f"s4_crash_redo rbp={rbp} reuse={reuse} faults={when}",
                       examples=(reuse and when == "redo_only"))


ARMS_ALL = ("A", "AJ", "AJ2", "G", "C", "CJ", "CJ2")
PRES5 = [[], [("p", 1)], [("p", 1), ("p", 1)], [("f", 1)], [("p", 1), ("f", 1)],
         [("pn", 1, 4)], [("pn", 1, 4), ("p", 1)]]
MIDS = [[("u", 1)], [("u", 2)], [("u", 3)], [("f", 1), ("u", 1)]]


def s5_after_rb(p):
    """U1 (the AJ2 / CJ2 cell): clean rollback, publishes (one variant switches), crash, then a
    normal mount with root and record faults.  The new timeline has written over the abandoned
    records from R_old's prefix end + 1 on."""
    t = Tally(ARMS_ALL)
    for pre in PRES5:
        for kk in (1, 2, 3):
            for mid in MIDS:
                for down, kr, mr in itertools.product((None, 0, 1), range(5), range(7)):
                    script = pre + [("rb", kk, NO_FAULT, 0)] + mid \
                        + [("crash",), ("mount", (down, kr, mr, (), ())), ("u", 1)]
                    t.add({arm: run(arm, script) for arm in ARMS_ALL}, script)
    t.dump(p, "s5_rb_then_crash_mount")


def s6_two_rb(p):
    """U1: a second rollback after a clean one, with root and record faults at the second."""
    t = Tally(ARMS_ALL)
    for pre in PRES5:
        for kk in (1, 2):
            for mid in MIDS[:2] + MIDS[3:]:
                for kk2 in (1, 2, 3):
                    for down, kr, mr in itertools.product((None, 0, 1), range(4), range(6)):
                        script = pre + [("rb", kk, NO_FAULT, 0)] + mid \
                            + [("rb", kk2, (down, kr, mr, (), ()), 0), ("u", 1)]
                        t.add({arm: run(arm, script) for arm in ARMS_ALL}, script)
    t.dump(p, "s6_two_rollbacks")


def s9_plain_crash(p):
    """side cell (not a rollback): plain crash, the newest root and both copies of its record
    unreadable; every published identity counts, not only those after a rollback."""
    for warm in ("strict", "lenient"):
        t = Tally(ARMS_ALL)
        for pre in ([("p", 1)], [("p", 1), ("p", 1)], [("p", 0), ("p", 1)],
                    [("p", 1), ("p", 0), ("p", 1)]):
            for down, kr, mr in itertools.product((None, 0, 1), range(4), range(5)):
                script = pre + [("crash",), ("mount", (down, kr, mr, (), ())), ("u", 2)]
                t.add({arm: run(arm, script, warm=warm) for arm in ARMS_ALL}, script,
                      need_rb=False)
        t.dump(p, f"s9_plain_crash_{warm}", max_log=16)


def w_after(own, devs, r0):
    """publishes a new instance makes from region r0 on until its roots cover two disks
    (two regions on a one-disk pool), D16 file line 207; the user object goes into the next."""
    dks, rgs = set(), set()
    for j in range(len(own) + 1):
        r = (r0 + j) % len(own)
        dks.add(own[r])
        rgs.add(r)
        if (devs >= 2 and len(dks) >= 2) or (devs == 1 and len(rgs) >= 2):
            return j + 1
    return None


def s8_geometry(p):
    """U1: margin of C / CJ against A / AJ over root-region ownerships.  One timeline; the L
    newest txgs are hidden (roots; for the J arms also both record copies).  C is hit iff
    L >= w + 1 (strict) or L >= 2 (lenient: user right after the rollback root); A, AJ iff
    L >= 1.  Cost: down disk 1, each root slot 1, record copies per slot (fs) or per range (fr)."""
    geos = [((0, 0, 0), 1), ((0, 1, 0), 2), ((0, 1, 0), 3), ((0, 1, 2), 3),
            ((0, 1, 0, 1), 2), ((0, 0, 1, 1), 2), ((0, 1, 2, 0), 3), ((0, 1, 2, 3), 4),
            ((0, 1, 0, 1, 0), 2), ((0, 1, 2, 0, 1), 3)]
    arms = (("A", 0, False), ("AJ", 0, True), ("C_strict", None, False),
            ("CJ_strict", None, True), ("C_lenient", 1, False), ("CJ_lenient", 1, True))
    for own, devs in geos:
        rg = len(own)
        jd = (0,) if devs == 1 else (0, 1)
        best = {}
        for a in range(rg):
            for L in range(1, 3 * rg + 4):
                w = w_after(own, devs, (a - L + 1) % rg)
                hidden = [(a - k) % rg for k in range(L)]
                for d in [None] + (sorted(set(own)) if devs >= 2 else []):
                    base = (0 if d is None else 1) + sum(1 for r in hidden if own[r] != d)
                    other = [x for x in jd if x != d]
                    for arm, m, rec in arms:
                        need = (w if m is None else m) + 1
                        if L < need:
                            continue
                        for key, val in (("fs", base + (L * len(other) if rec else 0)),
                                         ("fr", base + (len(other) if rec else 0))):
                            cand = (val, L, a, -1 if d is None else d)
                            if (arm, key) not in best or cand < best[(arm, key)]:
                                best[(arm, key)] = cand
        ws = sorted({w_after(own, devs, r) for r in range(rg)})
        p(f"name=s8_geometry own={'/'.join(map(str, own))} devs={devs} warmup={ws} "
          + " ".join(f"{arm}:fs={best[(arm, 'fs')][0]},fr={best[(arm, 'fr')][0]}"
                     f"[L={best[(arm, 'fs')][1]},d={best[(arm, 'fs')][3]}]"
                     for arm, _m, _r in arms))


def main():
    out = []
    p = out.append
    p("# c143r2_model.py output (stdlib only, deterministic)")
    p("# arms: A = arm jia / yi for stat 12 (max over readable roots); AJ / AJ2 = + journal")
    p("# records (at rollback / every mount); G = clue: pool-wide number watermark in root and")
    p("# record headers; C = arm bing; CJ / CJ2 = + journal checkpoint_txg (rollback / every mount)")
    p("# fs = faults per slot (down disk 1, root slot 1, record copy 1); fr = record copies per range")
    for title, fn in (("s1 destroyed head", s1_destroyed_head),
                      ("s2 newest roots and records", s2_newest),
                      ("s3 journal wrap inside the rollback window", s3_wrap),
                      ("s4 rollback attempt crashes, then redo", s4_crash_redo),
                      ("s5 after a rollback: crash and mount", s5_after_rb),
                      ("s6 second rollback", s6_two_rb),
                      ("s8 geometry margin", s8_geometry),
                      ("s9 side: plain crash, no rollback", s9_plain_crash)):
        p("")
        p(f"## {title}")
        fn(p)
    p(f"END c143r2_model lines_before_this={len(out)}")
    sys.stdout.write("\n".join(out) + "\n")


if __name__ == "__main__":
    main()

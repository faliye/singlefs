#!/usr/bin/env python3
# C143 round 3, attack leg (Opus).  stdlib only, deterministic.  Run from this directory:
#     nice -n 19 python3 -B c143r3_model.py > c143r3_model.out
# c143r2_base.py is a byte-for-byte copy of ../c143-r2-opus-model/c143r2_model.py (sha256
# 76fc64e1...a7e7); it is imported, never edited.  New here:
#   * Sim3.scan also counts the root slots that do not yield a readable root (k_bad);
#   * clue arms CK / CK2 (mine, attacked zero rounds): bing, first new root txg =
#     readable ring max + 1 + k_bad (at rollback; CK2 also at every normal mount) -- this jump
#     moves the ring rotation onto the newest roots (it breaks: see f5 inv_vs_C); CR / CR2
#     jump by exactly R * S instead, which writes the same slots as bing;
#   * pair tallies keep the fewest faults of the scripts where arm x is hit and arm y is not,
#     and a per-prefix comparison (same history, different faults);
#   * families: f1 (CJ shifts the landing, r2 s2 space), f5 (CJ2 at mounts, r2 s5 space),
#     f11 (clone heads, tree-id reissue, destroyed heads), f12 (attempt crash / redo under both
#     prefix-end readings, three rollbacks, back-to-back rollbacks, rollback-crash-mount-rollback);
#   * a slot-only fault count (no whole-disk fault at a writable scan) next to the r2 count.
import itertools
import sys

import c143r2_base as b

R = b.R
H0 = b.H0
NO_FAULT = b.NO_FAULT
b.CRIT.update({"CK": "numbg", "CK2": "numbg", "CR": "numbg", "CR2": "numbg"})


class Sim3(b.Sim):
    def __init__(self, arm, **kw):
        super().__init__(arm, **kw)
        self.k_bad = 0
        self.jumps = []          # extra txgs burnt by CK / CK2 beyond readable max + 1

    def scan(self, u):
        """r2 scan, plus k_bad = root slots of the ring that yield no readable root (faulted,
        or never written: the implementation cannot tell a never-written slot from a torn one)."""
        down, kr, mr, rtx, ctx = u
        items = list(self.ring.items())
        bad = {s for s, _ in items if down is not None and self.own[s[0]] == down}
        rest = {id(r): r for s, r in items if s not in bad}
        order = sorted(rest.values(), key=lambda r: (r.txg, r.inst), reverse=True)
        victims = {id(r) for r in order[:kr]} | {id(r) for r in rest.values() if r.txg in rtx}
        bad |= {s for s, r in items if id(r) in victims}
        self.k_bad = R * self.S - sum(1 for s in self.ring if s not in bad)
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
        other = (0, 1) if down is None else (1 - down,)
        recs = sorted(self.jr.items(), key=lambda kv: kv[1][1], reverse=True)
        hid = {js for js, _ in recs[:mr]} | {js for js, rec in recs if rec[2] in ctx}
        for js in hid:
            for dk in other:
                self.targets.add(("j", js, dk))
        self.recount()
        return roots, [rec for js, rec in recs if js not in hid]

    def mount(self, u=NO_FAULT):
        """r2 mount; CK2 starts the new instance at max(replayed txg, readable max + k_bad)."""
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
        if self.arm == "AJ2":
            for tid, h in self.heads.items():
                h[0] = max([h[0]] + [self.states[rec[3]].heads[tid][0] for rec in recs
                                     if self.states[rec[3]].alive
                                     and tid in self.states[rec[3]].heads])
        if self.arm == "G":
            g = max([r.gwm for r in roots] + [rec[5] for rec in recs])
            for h in self.heads.values():
                h[0] = max(h[0], g)
            self.gwm = max(self.gwm, g)
        start = t
        if self.arm == "CJ2":
            start = max([t] + [r.txg for r in roots] + [rec[2] for rec in recs])
        if self.arm == "CK2":
            start = max(t, max(r.txg for r in roots) + self.k_bad)
            self.jumps.append(start - t)
        if self.arm == "CR2":
            start = t + R * self.S
            self.jumps.append(R * self.S)
        new = self.new_inst()
        tb = dict(sel.table)
        for i in range(max(sel.inst, 1), new):
            if not (i in tb and tb[i][1]):
                tb[i] = (t, False) if i == sel.inst else (0, False)
        self.table, self.inst, self.txg, self.ctr = tb, new, start, c + 1
        self.covered, self.inst_last_pub = set(), 0
        self.log.append(f"MOUNT u={b.fmtu(u)}: selects txg{sel.txg} i{sel.inst}, replays to "
                        f"txg{t} (prefix end c{c}), new i{new} from txg{start + 1}, "
                        f"wm={self.wms()}" + (" [selected root is on an abandoned timeline]"
                                              if (sel.txg, sel.inst) in self.abandoned else ""))
        self.publish(0)
        self.warmup()
        return True

    def tree_reissued(self):
        """a tree id published for two different clone events (r1 H6)."""
        return any(len(ev) > 1 for ev in self.tree_pub.values())

    def rollback(self, pick, u=NO_FAULT, sw=False, crash_attempt=False):
        """r2 rollback; CK / CK2: first new root txg = readable ring max + 1 + k_bad."""
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
        self.tree_wm = max([ro.tree_wm] + [r.tree_wm for r in roots])
        self.gwm = ro.gwm
        rmax = max(r.txg for r in roots)
        tfirst = rmax + 1
        if self.arm in ("CJ", "CJ2"):
            tfirst = max([tfirst] + [rec[2] + 1 for rec in recs])
        if self.arm in ("CK", "CK2"):
            tfirst = rmax + 1 + self.k_bad
            self.jumps.append(self.k_bad)
        if self.arm in ("CR", "CR2"):
            tfirst = rmax + 1 + R * self.S
            self.jumps.append(R * self.S)
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
        if self.rbp == "scan":
            have = {(rec[0], rec[1]) for rec in recs}
            while (ro.inst, c + 1) in have:
                c += 1
        if self.reuse:
            readable = {id(r) for r in roots}
            ring_ids = {id(r) for r in self.ring.values()}
            for r in self.allroots:
                if not self.valid(r, tbl) and (id(r) not in ring_ids or id(r) not in readable):
                    self.states[r.sid].alive = False
        self.table, self.inst, self.txg, self.ctr = tbl, new, tfirst - 1, c + 1
        self.covered, self.inst_last_pub = set(), 0
        self.log.append(f"ROLLBACK{' ATTEMPT' if crash_attempt else ''} u={b.fmtu(u)}: R_old=txg"
                        f"{ro.txg} i{ro.inst}; readable ring max txg{rmax}; k_bad={self.k_bad}; "
                        f"first new root txg{tfirst} i{new}; records from c{c + 1}; "
                        f"wm={self.wms()}")
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
            self.switch(None)
        self.publish(0)
        self.warmup()
        return True

    def do(self, op):
        """r2 ops plus: ("rbxk", kk, u) rollback attempt to the kk-th newest candidate that
        crashes after its record; ("cl12",) clone head 12 even after a rollback."""
        if op[0] == "rbxk":
            return self.rollback(lambda c, kk=op[1]: c[kk - 1]
                                 if kk <= len(c) and H0 in c[kk - 1].heads else None,
                                 op[2], False, crash_attempt=True)
        return super().do(op)


def run3(arm, script, **kw):
    s = Sim3(arm, **kw)
    s.first_mount()
    for op in script:
        if s.do(op) is False:
            return None
    return s


PAIRS = [("CJ", "C"), ("CJ2", "C"), ("C", "A"), ("C", "CJ"), ("G", "C"), ("G", "AJ"),
         ("AJ", "G"), ("CK", "C"), ("C", "CK"), ("CK2", "C"), ("C", "CK2"), ("A", "G"),
         ("CR", "C"), ("C", "CR"), ("CR2", "C"), ("C", "CR2")]


class Tally3(b.Tally):
    """r2 tally, plus: fewest faults of x-hit-not-y scripts, per-prefix minima, CK jumps,
    tree-id reissue (r1 H6) per arm."""

    def __init__(self, arms, pairs=PAIRS):
        super().__init__(arms)
        self.tp = [(x, y) for x, y in pairs if x in arms and y in arms]
        self.pmin = {}
        self.pref = {}
        self.jump = {a: [0, 0, 0] for a in arms}      # runs with a jump, sum, max
        self.tree = {a: [0, None] for a in arms}      # runs with a reissued tree id, min fs
        self.inv_vs_c = dict.fromkeys(arms, 0)        # script invalid for the arm, valid for C

    def add(self, results, script, need_rb=True, prefix=None):
        hit, fsd = set(), {}
        for a, s in results.items():
            if s is None:
                if results.get("C") is not None:
                    self.inv_vs_c[a] += 1
                continue
            v = b.violations(s, b.CRIT[a], need_rb)
            self.runs[a] += 1
            fsd[a] = (s.fs, s.fr)
            cell = self.byf[a].setdefault(s.fs, [0, 0])
            cell[0] += 1
            if s.jumps:
                j = self.jump[a]
                j[0] += 1
                j[1] += sum(s.jumps)
                j[2] = max(j[2], max(s.jumps))
            if s.tree_reissued():
                tr = self.tree[a]
                tr[0] += 1
                tr[1] = s.fs if tr[1] is None else min(tr[1], s.fs)
            if not v:
                continue
            hit.add(a)
            self.viol[a] += 1
            cell[1] += 1
            self.c332[a][0 if s.c332 else 1] += 1
            for key, val in (("fs", s.fs), ("fr", s.fr)):
                cand = (val, s.fs + s.fr, len(script), b.fmt(script))
                bb = self.best.get((a, key))
                if bb is None or cand < bb[0]:
                    self.best[(a, key)] = (cand, list(s.log), v)
            if prefix is not None:
                pc = self.pref.setdefault(prefix, {})
                pc[a] = s.fs if a not in pc else min(pc[a], s.fs)
        if all(s is None for s in results.values()):
            self.invalid += 1
        for x in results:
            for y in results:
                if x != y and results[x] is not None and results[y] is not None \
                        and x in hit and y not in hit:
                    self.pairs[(x, y)] = self.pairs.get((x, y), 0) + 1
                    if (x, y) not in self.tp:
                        continue
                    s = results[x]
                    cand = (s.fs, s.fr, len(script), b.fmt(script))
                    bb = self.pmin.get((x, y))
                    if bb is None or cand < bb[0]:
                        self.pmin[(x, y)] = (cand, list(s.log), b.violations(s, b.CRIT[x]),
                                             fsd.get(y))
                    frb = self.pmin.get((x, y, "fr"))
                    cfr = (s.fr, s.fs, len(script), b.fmt(script))
                    if frb is None or cfr < frb:
                        self.pmin[(x, y, "fr")] = cfr
                    if prefix is not None:
                        pc = self.pref.setdefault(prefix, {})
                        k = ("pair", x, y)
                        pc[k] = s.fs if k not in pc else min(pc[k], s.fs)

    def dump3(self, p, name, examples=True, pair_examples=(), max_log=24):
        self.dump(p, name, examples=examples, max_log=max_log)
        for a in self.arms:
            j, tr = self.jump[a], self.tree[a]
            if j[0]:
                p(f"name={name} arm={a} jump_runs={j[0]} jump_mean={j[1] / j[0]:.2f} "
                  f"jump_max={j[2]}")
            p(f"name={name} arm={a} tree_id_reissued_runs={tr[0]} tree_id_reissued_min_fs={tr[1]} "
              f"invalid_where_C_valid={self.inv_vs_c[a]}")
        for x, y in self.tp:
            bb, frb = self.pmin.get((x, y)), self.pmin.get((x, y, "fr"))
            p(f"name={name} pair hit={x} not_hit={y} scripts={self.pairs.get((x, y), 0)} "
              f"min_fs={bb[0][0] if bb else None} min_fr={frb[0] if frb else None} "
              f"arm_{x}_min_fs={min((f for f, c in self.byf[x].items() if c[1]), default=None)}"
              f" arm_{y}_min_fs={min((f for f, c in self.byf[y].items() if c[1]), default=None)}")
            if bb and (x, y) in pair_examples:
                (fs, fr, _l, sc), log, v, yf = bb
                p(f"example name={name} pair hit={x} not_hit={y} fs={fs} fr={fr} script=[{sc}]")
                p(f"  violations={v}")
                for line in log[-max_log:]:
                    p("    " + line)
        for x, y in self.tp:
            k = ("pair", x, y)
            rows = [(pc[k], pc.get(y), pc.get(x)) for pc in self.pref.values() if k in pc]
            if not rows:
                continue
            fewer = sum(1 for pf, yf, _ in rows if yf is None or pf < yf)
            p(f"name={name} per_prefix hit={x} not_hit={y} prefixes={len(rows)} "
              f"prefixes_where_pair_needs_fewer_faults_than_{y}_on_same_prefix={fewer} "
              f"min_gap(pair_fs-{y}_fs)="
              f"{min((pf - yf for pf, yf, _ in rows if yf is not None), default=None)}")


ARMS1 = ("A", "AJ", "G", "C", "CJ", "CK", "CR")
ARMS5 = ("A", "AJ", "AJ2", "G", "C", "CJ", "CJ2", "CK", "CK2", "CR", "CR2")
ARMS8 = ("A", "AJ", "G", "C", "CJ", "CJ2", "CK", "CK2", "CR", "CR2")
F60 = [(d, kr, mr, (), ()) for d in (None, 0, 1) for kr in range(4) for mr in range(5)]
F18 = [(d, kr, mr, (), ()) for d in (None, 0, 1) for kr in (0, 1, 2) for mr in (0, 3)]


def slot_only(script):
    """True when no fault tuple of the script takes a whole disk away."""
    return all(not (isinstance(x, tuple) and len(x) == 5 and x[0] is not None)
               for op in script for x in op[1:])


class Both:
    """two tallies over one sweep: r2 count (whole-disk fault = 1) and slot-only scripts."""

    def __init__(self, arms):
        self.all, self.slot = Tally3(arms), Tally3(arms)

    def add(self, results, script, need_rb=True, prefix=None):
        self.all.add(results, script, need_rb, prefix)
        if slot_only(script):
            self.slot.add(results, script, need_rb, prefix)

    def dump(self, p, name, pair_examples=()):
        self.all.dump3(p, name, pair_examples=pair_examples)
        self.slot.dump3(p, name + "_slot_only", pair_examples=pair_examples)


def f1_cj_shift(p, L=5):
    """face 1: r2 s2 space (single rollback, newest roots / record copies / a disk hidden) with
    pair minima: where CJ is hit and C is not, how many faults, against C on the same prefix."""
    faults = [(d, kr, mr, (), ()) for d in (None, 0, 1) for kr in range(4) for mr in range(6)]
    t = Both(ARMS1)
    for n in range(L + 1):
        for pre in itertools.product(b.PRE, repeat=n):
            for kk in (1, 2):
                for u in faults:
                    script = list(pre) + [("rb", kk, u, 0), ("u", 2)]
                    t.add({arm: run3(arm, script) for arm in ARMS1}, script,
                          prefix=(pre, kk))
    t.dump(p, "f1_cj_shift", pair_examples=(("CJ", "C"), ("G", "C")))


def f5_after_rb(p):
    """face 1 at mounts: r2 s5 space (clean rollback, publishes, crash, mount with faults)."""
    t = Both(ARMS5)
    for pre in b.PRES5:
        for kk in (1, 2, 3):
            for mid in b.MIDS:
                for d, kr, mr in itertools.product((None, 0, 1), range(5), range(7)):
                    script = pre + [("rb", kk, NO_FAULT, 0)] + mid \
                        + [("crash",), ("mount", (d, kr, mr, (), ())), ("u", 1)]
                    t.add({arm: run3(arm, script) for arm in ARMS5}, script,
                          prefix=(tuple(map(tuple, pre)), kk, tuple(map(tuple, mid))))
    t.dump(p, "f5_rb_crash_mount", pair_examples=(("CJ2", "C"),))


def f11_clones(p):
    """face 2: clone heads and destroyed heads interleaved with a rollback.  clone_at = before
    R_old (no tree-id reissue possible) / after R_old (r1 H6 possible); the abandoned timeline
    makes n1 objects in the clone (and optionally destroys it); after the rollback a new clone
    of head 12 is made and gets 2 objects."""
    t = Both(ARMS8)
    for clone_at, a, n1, dest, c in itertools.product(("before", "after"), range(2), (1, 2),
                                                      (False, True), range(3)):
        for u in F60:
            head = [("p", 0)] * a
            if clone_at == "before":
                head += [("clone",), ("mark",)]
            else:
                head += [("mark",), ("clone",)]
            script = head + [("ucx", n1)] + ([("destroy",)] if dest else []) \
                + [("p", 0)] * c + [("rbm", (u[0], u[1], u[2], (), ()), 0), ("clone",),
                                    ("ucx", 2)]
            t.add({arm: run3(arm, script) for arm in ARMS8}, script,
                  prefix=(clone_at, a, n1, dest, c))
    t.dump(p, "f11_clones")


FA = [NO_FAULT, (0, 0, 0, (), ()), (1, 0, 0, (), ()), (None, 1, 0, (), ()),
      (None, 0, 1, (), ()), (0, 1, 2, (), ())]
FM = [NO_FAULT, (0, 0, 0, (), ()), (None, 1, 1, (), ()), (1, 1, 2, (), ())]
F36 = [(d, kr, mr, (), ()) for d in (None, 0, 1) for kr in range(4) for mr in (0, 2, 4)]


def f12_attempts(p):
    """face 3: (a) rollback attempt crashes after its record, a normal mount, publishes, then a
    rollback with faults, under both prefix-end readings (rold = R_old's last record + 1,
    scan = end of the whole readable chain + 1); (b) three rollbacks; (c) back-to-back
    rollbacks; (d) rollback, crash, mount with faults, rollback with faults."""
    for rbp in ("rold", "scan"):
        t = Both(ARMS8)
        for pre in ([("p", 1)], [("p", 1), ("p", 1)], [("p", 1), ("p", 0), ("p", 1)]):
            for kk, ua, um, d, kk2 in itertools.product((1, 2), FA, FM, (0, 1), (1, 2)):
                for u3 in F36:
                    script = pre + [("rbxk", kk, ua), ("mount", um)] + [("p", 0)] * d \
                        + [("rb", kk2, u3, 0), ("u", 2)]
                    t.add({arm: run3(arm, script, rbp=rbp) for arm in ARMS8}, script)
        t.dump(p, f"f12a_attempt_redo rbp={rbp}")
    t = Both(ARMS8)
    for pre in b.PRES5:
        for k1, k2, k3 in itertools.product((1, 2), (1, 2), (1, 2, 3)):
            for u in F60:
                script = pre + [("rb", k1, NO_FAULT, 0), ("u", 1), ("rb", k2, NO_FAULT, 0),
                                ("u", 1), ("rb", k3, u, 0), ("u", 2)]
                t.add({arm: run3(arm, script) for arm in ARMS8}, script)
    t.dump(p, "f12b_three_rollbacks")
    t = Both(ARMS8)
    for pre in b.PRES5:
        for k1, k2 in itertools.product((1, 2), (1, 2, 3)):
            for u in F60:
                script = pre + [("rb", k1, NO_FAULT, 0), ("rb", k2, u, 0), ("u", 2)]
                t.add({arm: run3(arm, script) for arm in ARMS8}, script)
    t.dump(p, "f12c_back_to_back")
    t = Both(ARMS8)
    for pre in b.PRES5:
        for k1, um, k2, u in itertools.product((1, 2), F18, (1, 2), F18):
            script = pre + [("rb", k1, NO_FAULT, 0), ("u", 1), ("crash",), ("mount", um),
                            ("rb", k2, u, 0), ("u", 1)]
            t.add({arm: run3(arm, script) for arm in ARMS8}, script)
    t.dump(p, "f12d_rb_crash_mount_rb")


def main():
    out = []
    p = out.append
    p("# c143r3_model.py output (stdlib only, deterministic); base = c143r2_base.py")
    p("# arms as in r2 (A, AJ, AJ2, G, C, CJ, CJ2); CK / CK2 = clue: bing with first new root")
    p("# txg = readable ring max + 1 + root slots yielding no readable root (rollback; CK2 also")
    p("# at every mount).  fs = faults per slot, fr = record copies per range.  *_slot_only =")
    p("# the same sweep restricted to scripts with no whole-disk fault at any scan")
    for title, fn in (("f1 CJ shifts the landing (r2 s2 space)", f1_cj_shift),
                      ("f1x the same history, wider fault space", f1x_same_history),
                      ("f5 after a rollback: crash and mount (r2 s5 space)", f5_after_rb),
                      ("f11 clone heads / destroyed heads / tree-id reissue", f11_clones),
                      ("f12 attempt crash-redo, repeated rollbacks", f12_attempts),
                      ("f1y details of f1x (printed last, f1x lines unchanged)", f1y_detail)):
        p("")
        p(f"## {title}")
        fn(p)
    p(f"END c143r3_model lines_before_this={len(out)}")
    sys.stdout.write("\n".join(out) + "\n")



F1X_ROWS = {}


def f1x_same_history(p, L=5):
    """face 1, finest reading of 'the same class': on every prefix of the r2 s2 space, the
    fewest faults C needs on that very history against the fewest faults of a script where CJ
    is hit and C is not.  Faults: a disk, the newest 0..10 root slots, the newest 0..10 record
    copies (a wider space than f1, so C's own minimum on a prefix is not cut off at 3 roots)."""
    faults = [(d, kr, mr, (), ()) for d in (None, 0, 1) for kr in range(11) for mr in range(11)]
    arms = ("C", "CJ", "CR")
    rows = {"all": [], "slot_only": []}
    cr_hits = dict.fromkeys(rows, 0)
    for n in range(L + 1):
        for pre in itertools.product(b.PRE, repeat=n):
            for kk in (1, 2):
                best = {reg: {"C": [None, None], "CJ": [None, None], "X": [None, None, None]}
                        for reg in rows}
                for u in faults:
                    script = list(pre) + [("rb", kk, u, 0), ("u", 2)]
                    res = {a: run3(a, script) for a in arms}
                    if any(s is None for s in res.values()):
                        continue
                    hit = {a: bool(b.violations(s, b.CRIT[a])) for a, s in res.items()}
                    for reg in (["all"] + (["slot_only"] if u[0] is None else [])):
                        bb = best[reg]
                        cr_hits[reg] += hit["CR"]
                        for a in ("C", "CJ"):
                            if hit[a]:
                                v = bb[a]
                                v[0] = res[a].fs if v[0] is None else min(v[0], res[a].fs)
                                v[1] = res[a].fr if v[1] is None else min(v[1], res[a].fr)
                        if hit["CJ"] and not hit["C"]:
                            x, s = bb["X"], res["CJ"]
                            if x[0] is None or s.fs < x[0]:
                                x[0], x[2] = s.fs, b.fmt(script)
                            x[1] = s.fr if x[1] is None else min(x[1], s.fr)
                for reg in rows:
                    if best[reg]["X"][0] is not None:
                        rows[reg].append((b.fmt(pre) or "-", kk, best[reg]))
    F1X_ROWS.update(rows)
    for reg, rs in rows.items():
        name = "f1x_same_history" + ("" if reg == "all" else "_slot_only")
        c_none = [r for r in rs if r[2]["C"][0] is None]
        fewer_fs = [r for r in rs if r[2]["C"][0] is not None and r[2]["X"][0] < r[2]["C"][0]]
        fewer_fr = [r for r in rs if r[2]["C"][1] is not None and r[2]["X"][1] < r[2]["C"][1]]
        p(f"name={name} prefixes_with_CJ_hit_C_not={len(rs)} C_never_hit_on_prefix={len(c_none)} "
          f"CJnotC_fewer_fs_than_C_on_prefix={len(fewer_fs)} "
          f"CJnotC_fewer_fr_than_C_on_prefix={len(fewer_fr)} "
          f"min_CJnotC_fs={min((r[2]['X'][0] for r in rs), default=None)} "
          f"min_CJnotC_fr={min((r[2]['X'][1] for r in rs), default=None)} CR_hits={cr_hits[reg]}")
        gaps = sorted((r[2]["X"][0] - r[2]["C"][0], r[2]["X"][1] - r[2]["C"][1])
                      for r in rs if r[2]["C"][0] is not None)
        p(f"name={name} gap(CJnotC - C on same prefix) fs_min={gaps[0][0] if gaps else None} "
          f"fr_min={min((g[1] for g in gaps), default=None)} "
          f"fs_hist={dict(sorted(__import__('collections').Counter(g[0] for g in gaps).items()))}")
        for r in (c_none + fewer_fs + fewer_fr)[:12]:
            p(f"name={name} row pre=[{r[0]}] kk={r[1]} C_min={r[2]['C']} CJ_min={r[2]['CJ']} "
              f"CJnotC_min=({r[2]['X'][0]}, {r[2]['X'][1]}) script=[{r[2]['X'][2]}]")


def f1y_detail(p):
    """details of f1x: every prefix where CJ needs fewer faults than C on the same history, how
    many prefixes where C is never hit contain a root-write failure, and the logs of one."""
    for reg, rs in F1X_ROWS.items():
        name = "f1y_detail" + ("" if reg == "all" else "_slot_only")
        c_none = [r for r in rs if r[2]["C"][0] is None]
        with_f = sum(1 for r in c_none if "f(" in r[0])
        p(f"name={name} C_never_hit_prefixes={len(c_none)} "
          f"of_which_prefix_has_a_root_write_failure={with_f}")
        for r in rs:
            if r[2]["C"][0] is not None and r[2]["X"][0] < r[2]["C"][0]:
                p(f"name={name} fewer_fs pre=[{r[0]}] kk={r[1]} C_min={r[2]['C']} "
                  f"CJnotC_min=({r[2]['X'][0]}, {r[2]['X'][1]}) script=[{r[2]['X'][2]}]")
        k = 0
        for r in rs:
            if r[2]["C"][1] is not None and r[2]["X"][1] < r[2]["C"][1] and k < 6:
                k += 1
                p(f"name={name} fewer_fr pre=[{r[0]}] kk={r[1]} C_min={r[2]['C']} "
                  f"CJnotC_min=({r[2]['X'][0]}, {r[2]['X'][1]}) script=[{r[2]['X'][2]}]")
    script = [("p", 0), ("p", 0), ("f", 0), ("p", 1), ("rb", 1, (0, 1, 3, (), ()), 0), ("u", 2)]
    for arm in ("C", "CJ"):
        s = run3(arm, script)
        p(f"example name=f1y_detail arm={arm} fs={s.fs} fr={s.fr} script=[{b.fmt(script)}] "
          f"violations={b.violations(s, b.CRIT[arm])}")
        for line in s.log[-11:]:
            p("    " + line)


if __name__ == "__main__":
    main()

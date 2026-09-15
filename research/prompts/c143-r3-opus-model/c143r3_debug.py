#!/usr/bin/env python3
# C143 round 3, attack leg: the single histories the report quotes that are not printed by
# c143r3_model.py (why a clue arm made a script invalid, one hit of each clue arm).
# stdlib only, deterministic.  Run from this directory:
#     nice -n 19 python3 -B c143r3_debug.py > c143r3_debug.out
import itertools

import c143r2_base as b
import c143r3_model as m


def show(p, arm, script, **kw):
    s = m.Sim3(arm, **kw)
    s.first_mount()
    failed = None
    for op in script:
        if s.do(op) is False:
            failed = op
            break
    p(f"arm={arm} kw={kw} script=[{b.fmt(script)}] failed_at={failed} fs={s.fs} fr={s.fr} "
      f"violations={b.violations(s, b.CRIT[arm]) if failed is None else None}")
    for line in s.log[-12:]:
        p("    " + line)
    p("    ring=" + " ".join(f"{sl}:txg{r.txg}/i{r.inst}{'/h12' if 12 in r.heads else ''}"
                          for sl, r in sorted(s.ring.items())))


def first(pred, scripts):
    for sc in scripts:
        if pred(sc):
            return sc
    return None


def f5_scripts():
    for pre in b.PRES5:
        for kk in (1, 2, 3):
            for mid in b.MIDS:
                for d, kr, mr in itertools.product((None, 0, 1), range(5), range(7)):
                    yield pre + [("rb", kk, b.NO_FAULT, 0)] + mid \
                        + [("crash",), ("mount", (d, kr, mr, (), ())), ("u", 1)]


def f12a_scripts():
    for pre in ([("p", 1)], [("p", 1), ("p", 1)], [("p", 1), ("p", 0), ("p", 1)]):
        for kk, ua, um, d, kk2 in itertools.product((1, 2), m.FA, m.FM, (0, 1), (1, 2)):
            for u3 in m.F36:
                yield pre + [("rbxk", kk, ua), ("mount", um)] + [("p", 0)] * d \
                    + [("rb", kk2, u3, 0), ("u", 2)]


def f12d_scripts():
    for pre in b.PRES5:
        for k1, um, k2, u in itertools.product((1, 2), m.F18, (1, 2), m.F18):
            yield pre + [("rb", k1, b.NO_FAULT, 0), ("u", 1), ("crash",), ("mount", um),
                         ("rb", k2, u, 0), ("u", 1)]


def hit(arm, sc, **kw):
    s = m.run3(arm, sc, **kw)
    return s is not None and bool(b.violations(s, b.CRIT[arm]))


def main():
    out = []
    p = out.append
    p("# c143r3_debug.py output")
    p("## d1 CK: first f5 script valid for C and invalid for CK")
    sc = first(lambda x: m.run3("C", x) is not None and m.run3("CK", x) is None, f5_scripts())
    show(p, "C", sc)
    show(p, "CK", sc)
    p("## d2 CJ2: first f12a (prefix end = R_old's) script valid for C and invalid for CJ2")
    sc = first(lambda x: m.run3("C", x, rbp="rold") is not None
               and m.run3("CJ2", x, rbp="rold") is None, f12a_scripts())
    show(p, "C", sc, rbp="rold")
    show(p, "CJ2", sc, rbp="rold")
    p("## d3 CR2 after a rollback with a jump: the jump leaves a txg gap, 4 root slots hide it")
    sc = [("rb", 1, b.NO_FAULT, 0), ("u", 1), ("crash",), ("mount", (None, 4, 0, (), ())),
          ("u", 1)]
    for arm in ("C", "CR", "CR2"):
        show(p, arm, sc)
    p("## d4 first f12d script where CR2 is hit and C is not")
    sc = first(lambda x: hit("CR2", x) and m.run3("C", x) is not None and not hit("C", x),
               f12d_scripts())
    show(p, "C", sc)
    show(p, "CR2", sc)
    p("## d5 first f12b-like script where CR is hit (three rollbacks, faults at the third)")
    scripts = (pre + [("rb", k1, b.NO_FAULT, 0), ("u", 1), ("rb", k2, b.NO_FAULT, 0), ("u", 1),
                      ("rb", k3, u, 0), ("u", 2)]
               for pre in b.PRES5 for k1, k2, k3 in itertools.product((1, 2), (1, 2), (1, 2, 3))
               for u in m.F60)
    sc = first(lambda x: hit("CR", x), scripts)
    show(p, "C", sc)
    show(p, "CR", sc)
    p(f"END c143r3_debug lines_before_this={len(out)}")
    print("\n".join(out))


if __name__ == "__main__":
    main()

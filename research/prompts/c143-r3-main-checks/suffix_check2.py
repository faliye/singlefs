"""Main-agent check, second half: the same suffix question as suffix_check.py for the families
where the leg reported CJ / CJ2 hit and C not hit after a later mount or repeated rollbacks
(f5, f12b, f12d). The scripts are built exactly as in the leg's c143r3_model.py; the only change
is that the final creating step u(k) may be preceded by d = 1..4 non-creating publishes.
Imports the leg's model read-only (-B)."""
import collections
import itertools
import sys

sys.path.insert(0, "/home/fy5090/code/singlefs/research/prompts/c143-r3-opus-model")
import c143r3_model as m  # noqa: E402

b = m.b
NO_FAULT = getattr(m, "NO_FAULT", None) or b.NO_FAULT


def fault_tuples(script):
    for op in script:
        if op[0] == "mount":
            yield op[1]
        elif op[0] == "rb":
            yield op[2]


def is_slot_only(script):
    return all(fault[0] is None for fault in fault_tuples(script))


def hit(arm, script):
    run = m.run3(arm, script)
    if run is None:
        return None, None
    return bool(b.violations(run, b.CRIT[arm])), run.fs


def delayed(script, delay):
    last = script[-1]
    assert last[0] == "u", last
    return script[:-1] + [("u", 0)] + [("p", 0)] * delay + [last]


def f5_scripts():
    for pre in b.PRES5:
        for kk in (1, 2, 3):
            for mid in b.MIDS:
                for d, kr, mr in itertools.product((None, 0, 1), range(5), range(7)):
                    yield pre + [("rb", kk, NO_FAULT, 0)] + mid \
                        + [("crash",), ("mount", (d, kr, mr, (), ())), ("u", 1)]


def f12b_scripts():
    for pre in b.PRES5:
        for k1, k2, k3 in itertools.product((1, 2), (1, 2), (1, 2, 3)):
            for u in m.F60:
                yield pre + [("rb", k1, NO_FAULT, 0), ("u", 1), ("rb", k2, NO_FAULT, 0),
                             ("u", 1), ("rb", k3, u, 0), ("u", 2)]


def f12d_scripts():
    for pre in b.PRES5:
        for k1, um, k2, u in itertools.product((1, 2), m.F18, (1, 2), m.F18):
            yield pre + [("rb", k1, NO_FAULT, 0), ("u", 1), ("crash",), ("mount", um),
                         ("rb", k2, u, 0), ("u", 1)]


def check(name, scripts, arms_x):
    for arm_x in arms_x:
        cases = collections.Counter()
        rescued = collections.Counter()
        by_delay = collections.Counter()
        not_above = collections.Counter()
        unrescued = []
        for script in scripts():
            c_hit, _ = hit("C", script)
            x_hit, x_fs = hit(arm_x, script)
            if c_hit is None or x_hit is None or not x_hit or c_hit:
                continue
            regimes = ["all"] + (["slot_only"] if is_slot_only(script) else [])
            for regime in regimes:
                cases[regime] += 1
            found = None
            for delay in (1, 2, 3, 4):
                c_hit_d, c_fs_d = hit("C", delayed(script, delay))
                if c_hit_d:
                    found = (delay, c_fs_d)
                    break
            if found:
                for regime in regimes:
                    rescued[regime] += 1
                    not_above[regime] += found[1] <= x_fs
                by_delay[found[0]] += 1
            else:
                unrescued.append((b.fmt(script), x_fs))
        for regime in ("all", "slot_only"):
            print(f"name={name} x={arm_x} regime={regime} x_hit_c_not={cases[regime]} "
                  f"c_hit_after_delay={rescued[regime]} with_fs_not_above_x={not_above[regime]}")
        print(f"name={name} x={arm_x} by_delay={dict(sorted(by_delay.items()))} "
              f"unrescued={len(unrescued)}")
        for script, x_fs in unrescued[:8]:
            print(f"name={name} x={arm_x} unrescued x_fs={x_fs} script=[{script}]")


def main():
    check("f5_rb_crash_mount", f5_scripts, ("CJ", "CJ2"))
    check("f12b_three_rollbacks", f12b_scripts, ("CJ", "CJ2"))
    check("f12d_rb_crash_mount_rb", f12d_scripts, ("CJ", "CJ2"))
    print("done")


if __name__ == "__main__":
    main()

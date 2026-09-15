"""Main-agent check for G (same suffix freedom as for CJ; delay 1..15) of the C143 round-3 attack leg's H15 (f1x): the leg fixed the post-rollback
suffix to u(2) (two creating publishes right after warm-up). Here, for every (prefix, kk, fault)
where CJ is hit and C is not under that suffix, C is rerun with d = 1..4 non-creating publishes
("p", 0) inserted after warm-up and before u(2). Same prefix, same faults, only the user's later
actions differ. Imports the leg's model read-only; writes nothing into its directory (-B)."""
import collections
import itertools
import sys

sys.path.insert(0, "/home/fy5090/code/singlefs/research/prompts/c143-r3-opus-model")
import c143r3_model as m  # noqa: E402

b = m.b
FAULTS = [(d, kr, mr, (), ()) for d in (None, 0, 1) for kr in range(11) for mr in range(11)]


def hit(arm, script):
    run = m.run3(arm, script)
    if run is None:
        return None, None
    return bool(b.violations(run, b.CRIT[arm])), run.fs


def main(max_prefix_len=5):
    for regime in ("all", "slot_only"):
        cases = 0
        rescued_by_delay = collections.Counter()
        rescued_same_fs = 0
        unrescued = []
        prefixes_c_never_hit_at_d0 = 0
        of_which_c_hit_with_delay = 0
        for n in range(max_prefix_len + 1):
            for prefix in itertools.product(b.PRE, repeat=n):
                for kk in (1, 2):
                    c_hit_d0 = False
                    c_hit_with_delay = False
                    had_case = False
                    for fault in FAULTS:
                        if regime == "slot_only" and fault[0] is not None:
                            continue
                        base = list(prefix) + [("rb", kk, fault, 0), ("u", 2)]
                        c_hit, _ = hit("C", base)
                        cj_hit, cj_fs = hit("G", base)
                        if c_hit is None or cj_hit is None:
                            continue
                        c_hit_d0 = c_hit_d0 or c_hit
                        if not (cj_hit and not c_hit):
                            continue
                        cases += 1
                        had_case = True
                        found = None
                        for delay in range(1, 16):
                            delayed = (list(prefix) + [("rb", kk, fault, 0), ("u", 0)]
                                       + [("p", 0)] * delay + [("u", 2)])
                            c_hit_delayed, c_fs_delayed = hit("C", delayed)
                            if c_hit_delayed:
                                found = (delay, c_fs_delayed)
                                break
                        if found:
                            rescued_by_delay[found[0]] += 1
                            rescued_same_fs += found[1] <= cj_fs
                            c_hit_with_delay = True
                        else:
                            unrescued.append((b.fmt(base), cj_fs))
                    if had_case and not c_hit_d0:
                        prefixes_c_never_hit_at_d0 += 1
                        of_which_c_hit_with_delay += c_hit_with_delay
        print(f"arm=G regime={regime} x_hit_c_not_cases={cases} "
              f"c_hit_after_delay={sum(rescued_by_delay.values())} "
              f"by_delay={dict(sorted(rescued_by_delay.items()))} "
              f"with_fs_not_above_cj={rescued_same_fs} unrescued={len(unrescued)}")
        print(f"regime={regime} prefixes_where_c_never_hit_with_u2={prefixes_c_never_hit_at_d0} "
              f"of_which_c_hit_once_delay_allowed={of_which_c_hit_with_delay}")
        for script, cj_fs in unrescued[:20]:
            print(f"regime={regime} unrescued cj_fs={cj_fs} script=[{script}]")
    print("done")


if __name__ == "__main__":
    main()

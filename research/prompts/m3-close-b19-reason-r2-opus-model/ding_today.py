"""I-7.9 不带卸载记号那一支今天的写法（丁全放开那一形）的 Python 转写，推的模型，不是编出来的二进制。
逐函数照开工快照 crates/singlefs-checker/src/walk.rs（sha256 b533596e…）：
  ceiling_two_bounds    <- rollback_floor_ceiling_of_the_valid_roots（8752 起）、rollback_floor_ceiling_from（8670）、
                           newest_first_distinct_state_txgs（8690）：树表读不出的（state 为 None）算空不空判不了，两沿分开
  window_bounds         <- window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise（8838）
  largest_below         <- largest_positive_txg_the_ring_slot_names_below（8967）
  possible_roots        <- possible_roots_in_the_ring_slot_at_the_raise（8994）
  ding                  <- rollback_floor_against_the_ring_reconstructed_at_the_raise（9091）
  judge_raise           <- judge_rollback_floor_raises_against_their_ceilings 不带卸载记号那一支（9499–9586）
根是 (txg, inst, F, state, dev)：dev 取它所在那一槽的区域的盘（walk.rs 8886、9004、9248；第 0 代根的几份各按自己那一槽的区域）。
state 是两棵用户可见树的根指针的代号，None 是树表读不出（第 0 代根被盖之后那一种）。
变体（只为量，不是改法）：
  max_rings            可能的环的上界（今天 4096；None 不设上界，另封顶 BIG 种，超了报 too_many）
  genesis_state        重建出的第 0 代根那一种的 state：None（今天）或给定值（按格式「mkfs 树表 0 条」认得出的那一种）
  genesis_device       'slot'（今天：按所在那一槽的区域的盘）或 'txg'（写者 mount.rs 3958–3968：按 target_for_publish(txg) 的区域，txg 0 落区域 0）
"""
ABSENT = ("ABSENT",)
BIG = 1 << 16


def _distinct(determinable, always):
    entries = [(t, i, st) for (t, i, st) in determinable] + [(t, i, None) for (t, i) in always]
    entries.sort(key=lambda e: (e[0], e[1]), reverse=True)
    seen, counted = [], []
    for (t, i, st) in entries:
        if st is None:
            counted.append(t)
        elif st not in seen:
            seen.append(st)
            counted.append(t)
    return counted


def _ceiling_from(newest_every, counted, oldest):
    c = sorted(counted, reverse=True)
    fourth = c[3] if len(c) >= 4 else oldest
    return min(newest_every, fourth)


def ceiling_two_bounds(valid):
    """valid 已按 (txg, inst) 升序、已滤有效。交回 None 或 dict(lowest, highest, per_device, distinct_newer)。"""
    if not valid:
        return None
    oldest = valid[0]
    per_device = {}
    for (t, i, f, st, d) in valid:
        per_device[d] = max(per_device.get(d, -1), t)
    newest_every = min(per_device.values())
    det, undet = [], []
    prev = ABSENT
    for (t, i, f, st, d) in valid:
        if st is not None and prev is not None:
            if st != prev:
                det.append((t, i, st))
        else:
            undet.append((t, i))
        prev = st
    newer = [e for e in det if (e[0], e[1]) != (oldest[0], oldest[1])]
    return dict(lowest=_ceiling_from(newest_every, _distinct(det, []), oldest[0]),
                highest=_ceiling_from(newest_every, _distinct(det, undet), oldest[0]),
                per_device=per_device, distinct_newer=len(_distinct(newer, [])))


def window_bounds(c, devs, R):
    return all(devs[reg] in c["per_device"] for reg in range(R)) and c["distinct_newer"] >= 4


def largest_below(region, slot, r, R, S):
    ring = R * S
    residue = region + R * slot
    below = r - 1
    if below < 0 or below < residue:
        return None
    largest = residue + ((below - residue) // ring) * ring
    return largest if largest > 0 else None


def is_valid(root, floor_before, rows):
    t, i = root[0], root[1]
    return t >= floor_before and not (i in rows and t > rows[i])


def possible_roots(region, slot, r, r_inst, rows, floor_before, journal, devs, R, S, genesis_state, genesis_device):
    ring = R * S
    dev = devs[region]
    out = []
    u = largest_below(region, slot, r, R, S)
    while True:
        if u is None:
            out.append(None)
            if slot == 0 and floor_before == 0:
                gdev = dev if genesis_device == "slot" else devs[0]
                out.append((0, 0, 0, genesis_state, gdev))
            return out
        if u < floor_before:
            out.append(None)
            return out
        versions = journal.get(u, [])
        if len(versions) != 1:
            return ("ERR", "journal%d" % len(versions))
        inst, f, st = versions[0]
        landed = inst == r_inst or (inst in rows and u < rows[inst])
        out.append((u, inst, f, st, dev))
        if landed:
            return out
        u = u - ring if u - ring > 0 else None


def ding(image, journal, r, r_inst, rows, floor_before, raised, devs, R, S, max_rings=4096,
         genesis_state=None, genesis_device="slot"):
    """image: {(region, slot): root 或 None(全零)}；root 是 (txg, inst, F, state, dev)。
    交回 ('holds'|'red'|'partial', 原因, 可能的环的种数)。"""
    per_slot, count = [], 1
    for region in range(R):
        for slot in range(S):
            occ = image.get((region, slot))
            if occ is None:
                per_slot.append([None])
            elif occ[0] < r:
                per_slot.append([occ])
            else:
                got = possible_roots(region, slot, r, r_inst, rows, floor_before, journal, devs, R, S,
                                     genesis_state, genesis_device)
                if isinstance(got, tuple):
                    return ("partial", got[1], 0)
                per_slot.append(got)
            count *= len(per_slot[-1])
    limit = max_rings if max_rings is not None else BIG
    if count > limit:
        return ("partial", "combos" if max_rings is not None else "too_many", count)
    kinds = set()
    for idx in range(count):
        left, valid = idx, []
        for opts in per_slot:
            x = opts[left % len(opts)]
            left //= len(opts)
            if x is not None and is_valid(x, floor_before, rows):
                valid.append(x)
        valid.sort(key=lambda x: (x[0], x[1]))
        c = ceiling_two_bounds(valid)
        if c is None:
            kinds.add("no_valid_root")
        elif raised > c["highest"]:
            kinds.add("red")
        elif raised <= c["lowest"]:
            kinds.add("holds")
        else:
            kinds.add("between")
    if len(kinds) == 1:
        k = kinds.pop()
        if k in ("holds", "red"):
            return (k, "-", count)
        return ("partial", k, count)
    return ("partial", "disagree", count)


def judge_raise(image, journal, rows_final, r_root, devs, R, S, **variant):
    """交回 None（不是抬 F 的根）或 dict(final=(判词, 原因), window=…, rings=种数)。"""
    r, r_inst, r_F = r_root[0], r_root[1], r_root[2]
    roots = [x for x in image.values() if x is not None]
    same = [x for x in roots if x[1] == r_inst and x[0] < r]
    if not same:
        return None
    floor_before = max(same, key=lambda x: x[0])[2]
    if r_F <= floor_before:
        return None
    N = max(x[0] for x in roots)
    ring = R * S
    rows = {i: T for (i, T) in rows_final.items() if i < r_inst}
    win = sorted([x for x in roots if x[0] < r and x[0] + ring > N and is_valid(x, floor_before, rows)],
                 key=lambda x: (x[0], x[1]))
    c = ceiling_two_bounds(win)
    if c is None:
        return dict(final=("not_judged", "no_window_root"), window="none", rings=0)
    if c["lowest"] < r_F <= c["highest"]:
        return dict(final=("not_judged", "window_between"), window="between", rings=0)
    if r_F <= c["lowest"] and not window_bounds(c, devs, R):
        v, why, n = ding(image, journal, r, r_inst, rows, floor_before, r_F, devs, R, S, **variant)
        return dict(final=(v, why), window="holds_undetermined", rings=n)
    return dict(final=("holds" if r_F <= c["lowest"] else "red", "-"), window="decided", rings=0)

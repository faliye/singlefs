"""C717 丁与窗口判法的 Python 照搬（推的模型，不是入库装置；只为在小模型上跑判法本身）。
逐函数照 crates/singlefs-checker/src/walk.rs（开工快照 sha256 b0dee5f2...）：
  ceiling_of_valid_roots      <- rollback_floor_ceiling_of_the_valid_roots（8770 起）+ rollback_floor_ceiling_from（8688）
                                 + newest_first_distinct_state_txgs（8708）；树表都读得出，两沿相等
  window_condition            <- window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise（8856）
  root_in_slot                <- root_in_the_ring_slot_at_the_raise（8986）；放开三处简化的开关 relax 见下
  largest_below               <- largest_positive_txg_the_ring_slot_names_below（8963）
  ding                        <- rollback_floor_against_the_ring_reconstructed_at_the_raise（9063）
  judge_raise                 <- judge_rollback_floor_raises_against_their_ceilings 不带卸载记号那一支（9396-9474）
根在模型里是 (txg, inst, F, state)；state 代表两棵用户可见树的根指针，ABSENT 与任何 state 都不同。
relax 是集合，元素：
  'T'     u 是行 T、被抛弃、实例没有行时不报认不出，两种都算：u 落了 / 没落（没落则那一槽住的是落点式的上一圈）
  'deep'  没落那一支往 u-RS 一直推（没有它只多看一层）
  'gen0'  第 0 代根被盖、抬之前 F=0 时认作第 0 代根（模型里是 (0, 0, 0, gen0_state)）
  'doubt' 本腿提的改法（零轮）：同实例、行 T > u 的也不当「一定落了」，一律两种都算（要与 'T'、'deep' 同开）
多种组合时照第二轮攻方模型「两种都算、结论一致才判」：每槽候选取笛卡尔积，判词一致才判，否则报部分没判。"""
ABSENT = ('ABSENT',)
MAX_COMBOS = 4096


def ceiling_of_valid_roots(valid, dev_of_region, R):
    """valid: [(txg, inst, F, state)] 已按 (txg, inst) 升序、已滤有效。交回 (ceiling, per_device, distinct_newer_than_oldest)。"""
    if not valid:
        return None
    per_device = {}
    for (t, i, f, st) in valid:
        d = dev_of_region[t % R]
        per_device[d] = max(per_device.get(d, -1), t)
    dev_min = min(per_device.values())
    non_empty = []
    prev = ABSENT
    for (t, i, f, st) in valid:
        if st != prev:
            non_empty.append((t, i, st))
        prev = st
    oldest = valid[0]

    def distinct(entries):
        counted, seen = [], []
        for (t, i, st) in sorted(entries, key=lambda e: (e[0], e[1]), reverse=True):
            if st not in seen:
                seen.append(st)
                counted.append(t)
        return counted

    counted = distinct(non_empty)
    fourth = counted[3] if len(counted) >= 4 else oldest[0]
    newer = [e for e in non_empty if (e[0], e[1]) != (oldest[0], oldest[1])]
    return min(dev_min, fourth), per_device, len(distinct(newer))


def window_condition(per_device, distinct_newer, dev_of_region, R):
    return all(dev_of_region[reg] in per_device for reg in range(R)) and distinct_newer >= 4


def largest_below(region, slot, r, R, S):
    ring = R * S
    residue = region + R * slot
    below = r - 1
    if below < 0 or below < residue:
        return None
    largest = residue + ((below - residue) // ring) * ring
    return largest if largest > 0 else None


def valid_filter(root, floor_before, rows):
    t, i = root[0], root[1]
    return t >= floor_before and not (i in rows and t > rows[i])


def root_in_slot(region, slot, start_txg, r, r_inst, rows, floor_before, journal_versions, R, S,
                 relax, gen0_state, depth=0):
    """从落点式 txg start_txg（None：落点式住进这一槽的正 txg 都不比它小）起认那一刻住着谁。交回候选列表或 ('ERR', 原因)。"""
    if start_txg is None:
        if slot != 0 or floor_before > 0:
            return [None]
        if 'gen0' in relax:
            return [(0, 0, 0, gen0_state)]
        return ('ERR', 'gen0')
    versions = journal_versions.get(start_txg, [])
    if len(versions) != 1:
        return ('ERR', 'journal%d' % len(versions))
    inst, F, st = versions[0]
    my_rows = [T for (i, T) in rows.items() if i == inst]
    landed_sure = (inst == r_inst or (my_rows and all(start_txg < T for T in my_rows))) and 'doubt' not in relax
    root = (start_txg, inst, F, st)
    if landed_sure:
        return [root]
    if 'T' not in relax:
        return ('ERR', 'T')
    if depth >= 1 and 'deep' not in relax:
        return ('ERR', 'depth')
    deeper_txg = start_txg - R * S
    if deeper_txg <= 0:
        deeper_txg = None
    deeper = root_in_slot(region, slot, deeper_txg, r, r_inst, rows, floor_before, journal_versions, R, S,
                          relax, gen0_state, depth + 1)
    if isinstance(deeper, tuple):
        return deeper
    return [root] + deeper


def ding(image_slots, journal_versions, r, r_inst, rows, floor_before, raised_F, dev_of_region, R, S,
         relax=frozenset(), gen0_state=0):
    """image_slots: {(region, slot): root 或 None(全零)}。交回 'holds' / 'red' / ('partial', 原因)。"""
    per_slot = []
    for region in range(R):
        for slot in range(S):
            occ = image_slots.get((region, slot))
            if occ is None:
                per_slot.append([None])
            elif occ[0] < r:
                per_slot.append([occ])
            else:
                got = root_in_slot(region, slot, largest_below(region, slot, r, R, S), r, r_inst, rows,
                                   floor_before, journal_versions, R, S, relax, gen0_state)
                if isinstance(got, tuple):
                    return ('partial', got[1])
                per_slot.append(got)
    combos = [[]]
    for opts in per_slot:
        combos = [c + [o] for c in combos for o in opts]
        if len(combos) > MAX_COMBOS:
            return ('partial', 'combos')
    verdicts = set()
    for combo in combos:
        valid = sorted([x for x in combo if x is not None and valid_filter(x, floor_before, rows)],
                       key=lambda x: (x[0], x[1]))
        c = ceiling_of_valid_roots(valid, dev_of_region, R)
        if c is None:
            verdicts.add(('partial', 'no_valid_root'))
        else:
            verdicts.add('red' if raised_F > c[0] else 'holds')
    if len(verdicts) == 1:
        return verdicts.pop()
    return ('partial', 'ambiguous')


def judge_raise(image_slots, journal_versions, rows_final, r_root, dev_of_region, R, S,
                relax=frozenset(), gen0_state=0, ding_always=False):
    """一条根 r_root 照 judge_rollback_floor_raises_against_their_ceilings（不带卸载记号那一支）判。
    交回 None（不是抬 F 的根）或 dict(final=..., window=..., ding=...)。final 是 I-7.9 这条根最终的判词：
    'holds' / 'red' / ('partial', 原因) / 'not_judged'（窗口里没有有效根，或 F 落在两沿之间——模型里两沿相等，不出现）。
    ding_always=True 时另把丁对每条抬 F 的根都跑一遍（第二轮攻方模型的口径：丁不受窗口条件门控）。"""
    r, r_inst, r_F, r_st = r_root
    roots = [x for x in image_slots.values() if x is not None]
    same = [x for x in roots if x[1] == r_inst and x[0] < r]
    if not same:
        return None
    floor_before = max(same, key=lambda x: x[0])[2]
    if r_F <= floor_before:
        return None
    N = max(x[0] for x in roots)
    ring = R * S
    rows = {i: T for (i, T) in rows_final.items() if i < r_inst}
    win = sorted([x for x in roots if x[0] < r and x[0] + ring > N and valid_filter(x, floor_before, rows)],
                 key=lambda x: (x[0], x[1]))
    out = {'floor_before': floor_before}
    c = ceiling_of_valid_roots(win, dev_of_region, R)
    d = None
    if ding_always:
        d = ding(image_slots, journal_versions, r, r_inst, rows, floor_before, r_F, dev_of_region, R, S, relax, gen0_state)
    if c is None:
        out.update(final='not_judged', window='none', ding=d)
        return out
    ceiling_w, per_device, distinct_newer = c
    if r_F > ceiling_w:
        out.update(final='red', window='red', ding=d)
        return out
    if not window_condition(per_device, distinct_newer, dev_of_region, R):
        if d is None:
            d = ding(image_slots, journal_versions, r, r_inst, rows, floor_before, r_F, dev_of_region, R, S, relax, gen0_state)
        out.update(final=d, window='holds_undetermined', ding=d)
        return out
    out.update(final='holds', window='holds_determined', ding=d)
    return out


def verdict_name(v):
    return v if isinstance(v, str) else v[0]

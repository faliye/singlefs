"""丁（C717）今天的实现在 Python 里的照搬（推的模型，不是入库装置）。第二轮辩方腿，在第一轮攻方模型 ding_port.py 的基础上按今天的 walk.rs 重写：
  possible_roots        <- possible_roots_in_the_ring_slot_at_the_raise（walk.rs 8994 起）
  ceiling_sides         <- rollback_floor_ceiling_of_the_valid_roots（8752 起）：带「树表读不出」一档（UNK），算 lowest / highest 两沿
  ding_today            <- rollback_floor_against_the_ring_reconstructed_at_the_raise（9091 起）
  judge_raise_today     <- judge_rollback_floor_raises_against_their_ceilings 不带卸载记号那一支（9437 起）
与 ding_port.py 的差别（逐条对 walk.rs）：
  1. 落点式正 txg 认完了：今天的代码一律先记一种「空着」，槽 0 且抬之前 F 为 0 时再加第 0 代根（ding_port 的 gen0 变体只记第 0 代根一种）。
  2. 认到的那一层 txg 低于抬之前的 F（窗口外）：记一种空着、不再往下（ding_port 没有这一停）。
  3. 第 0 代根的树表无从读：今天的代码记成「判不了」（UNK），它和它后面那一条有效根空不空判不了，上限落成 lowest / highest 两沿；
     ding_port 把它当一个已知的状态 0。
  4. 根带区域号：每一槽的盘按它所在的区域认（ding_port 用 txg mod R，第 0 代根一律算区域 0）。
cfg 的开关（逐项对应上面四条；'today' 全按今天的代码，'verdict' 是第一轮判决模型的口径）：
  none_with_genesis  True：认完了先记空着（今天）；False：槽 0 且 F=0 时只记第 0 代根（第一轮模型 gen0）
  genesis_state      UNK：判不了（今天）；0：已知的空树状态（第一轮模型的写法，历史生成器里初始状态就是 0，写者一侧的上限也这样数；
                     真池里 mkfs 第 0 版树表 0 条 = walk.rs 的 UserVisibleTreeRootPointers::ABSENT，对应「已知」这一档）
  window_out_stop    True：今天的窗口外一停；False：不停
根 = (txg, 实例, F, 状态, 区域)。状态 ABSENT 与任何别的状态都不同；UNK 表示树表读不出。
"""
ABSENT = ('ABSENT',)
UNK = ('UNK',)
MAX_COMBOS = 4096

CONFIGS = {
    'today': dict(none_with_genesis=True, genesis_state=UNK, window_out_stop=True),
    'verdict': dict(none_with_genesis=False, genesis_state=0, window_out_stop=False),
    'today_known_genesis': dict(none_with_genesis=True, genesis_state=0, window_out_stop=True),
    'today_genesis_only': dict(none_with_genesis=False, genesis_state=UNK, window_out_stop=True),
    'today_both_fixed': dict(none_with_genesis=False, genesis_state=0, window_out_stop=True),
    'today_no_stop': dict(none_with_genesis=True, genesis_state=UNK, window_out_stop=False),
}


def valid_filter(root, floor_before, rows):
    t, i = root[0], root[1]
    return t >= floor_before and not (i in rows and t > rows[i])


def largest_below(region, slot, r, R, S):
    ring = R * S
    residue = region + R * slot
    below = r - 1
    if below < 0 or below < residue:
        return None
    largest = residue + ((below - residue) // ring) * ring
    return largest if largest > 0 else None


def ceiling_sides(valid, devof):
    """valid：已按 (txg, 实例) 升序、已滤有效的根。交回 (lowest, highest, 每块盘最新的有效根, 去掉最旧那条后去重计入的非空条数)，一条都没有交回 None。"""
    if not valid:
        return None
    oldest = valid[0]
    per_device = {}
    for (t, i, f, st, reg) in valid:
        d = devof[reg]
        per_device[d] = max(per_device.get(d, -1), t)
    newest_everywhere = min(per_device.values())
    non_empty, undeterminable = [], []
    previous = ABSENT
    for (t, i, f, st, reg) in valid:
        if st == UNK or previous == UNK:
            undeterminable.append((t, i))
        elif st != previous:
            non_empty.append((t, i, st))
        previous = st

    def counted_txgs(determinable, always_counted):
        items = [(t, i, st) for (t, i, st) in determinable] + [(t, i, None) for (t, i) in always_counted]
        items.sort(key=lambda e: (e[0], e[1]), reverse=True)
        states, counted = [], []
        for (t, i, st) in items:
            if st is None:
                counted.append(t)
            elif st not in states:
                states.append(st)
                counted.append(t)
        return counted

    def ceiling_from(counted):
        newest_first = sorted(counted, reverse=True)
        fourth = newest_first[3] if len(newest_first) >= 4 else oldest[0]
        return min(newest_everywhere, fourth)

    lowest = ceiling_from(counted_txgs(non_empty, []))
    highest = ceiling_from(counted_txgs(non_empty, undeterminable))
    newer = [e for e in non_empty if (e[0], e[1]) != (oldest[0], oldest[1])]
    return lowest, highest, per_device, len(counted_txgs(newer, []))


def possible_roots(region, slot, start_txg, r, r_inst, rows, floor_before, jv, R, S, cfg):
    """一槽在抬 F 那一刻可能住的每一条根（None：空着或无效）。交回列表或 ('ERR', 原因)。"""
    out = []
    u = start_txg
    genesis_case = slot == 0 and floor_before == 0
    while True:
        if u is None:
            if cfg['none_with_genesis'] or not genesis_case:
                out.append(None)
            if genesis_case:
                out.append((0, 0, 0, cfg['genesis_state'], region))
            return out
        if cfg['window_out_stop'] and u < floor_before:
            out.append(None)
            return out
        versions = jv.get(u, [])
        if len(versions) != 1:
            return ('ERR', 'journal%d' % len(versions))
        inst, F, st = versions[0]
        my_rows = [T for (i, T) in rows.items() if i == inst]
        landed = inst == r_inst or (bool(my_rows) and all(u < T for T in my_rows))
        out.append((u, inst, F, st, region))
        if landed:
            return out
        u = u - R * S
        if u <= 0:
            u = None


def ding_today(image_slots, jv, r, r_inst, rows, floor_before, raised_F, devof, R, S, cfg):
    """交回 ('holds'|'red'|'partial', 原因)。image_slots：{(区域, 槽): (txg, 实例, F, 状态) 或 None(全零)}。"""
    per_slot = []
    count = 1
    for region in range(R):
        for slot in range(S):
            occ = image_slots.get((region, slot))
            if occ is None:
                per_slot.append([None])
            elif occ[0] < r:
                per_slot.append([occ + (region,)])
            else:
                got = possible_roots(region, slot, largest_below(region, slot, r, R, S), r, r_inst, rows,
                                     floor_before, jv, R, S, cfg)
                if isinstance(got, tuple):
                    return ('partial', got[1])
                per_slot.append(got)
                count *= len(got)
    if count > MAX_COMBOS:
        return ('partial', 'combos')
    combos = [[]]
    for options in per_slot:
        combos = [c + [o] for c in combos for o in options]
    kinds = []
    for combo in combos:
        valid = sorted([x for x in combo if x is not None and valid_filter(x, floor_before, rows)],
                       key=lambda x: (x[0], x[1]))
        sides = ceiling_sides(valid, devof)
        if sides is None:
            kinds.append('none')
        elif raised_F > sides[1]:
            kinds.append('red')
        elif raised_F <= sides[0]:
            kinds.append('holds')
        else:
            kinds.append('between')
    if len(set(kinds)) != 1:
        return ('partial', 'ambiguous')
    only = kinds[0]
    if only in ('holds', 'red'):
        return (only, '')
    return ('partial', 'between' if only == 'between' else 'no_valid_root')


def judge_raise_today(image_slots, jv, rows_final, r_root, devof, R, S, cfg):
    """一条根 r_root 照今天的 judge_rollback_floor_raises_against_their_ceilings（不带卸载记号那一支）判。
    交回 None（不是抬 F 的根）或 dict(final=(判词, 原因), window=…, floor_before=…)。"""
    r, r_inst, r_F, r_st = r_root
    roots = [(x, reg) for ((reg, _slot), x) in image_slots.items() if x is not None]
    same = [x for (x, _) in roots if x[1] == r_inst and x[0] < r]
    if not same:
        return None
    floor_before = max(same, key=lambda x: x[0])[2]
    if r_F <= floor_before:
        return None
    newest = max(x[0] for (x, _) in roots)
    ring = R * S
    rows = {i: T for (i, T) in rows_final.items() if i < r_inst}
    window = sorted([x + (reg,) for (x, reg) in roots
                     if x[0] < r and x[0] + ring > newest and valid_filter(x, floor_before, rows)],
                    key=lambda x: (x[0], x[1]))
    sides = ceiling_sides(window, devof)
    out = {'floor_before': floor_before}
    if sides is None:
        out.update(final=('not_judged', 'no_valid_root_in_window'), window='none')
        return out
    lowest, highest, per_device, distinct_newer = sides
    if lowest < r_F <= highest:
        out.update(final=('not_judged', 'window_between'), window='between')
        return out
    window_pins_the_ceiling = all(devof[reg] in per_device for reg in range(R)) and distinct_newer >= 4
    if r_F <= lowest and not window_pins_the_ceiling:
        out.update(final=ding_today(image_slots, jv, r, r_inst, rows, floor_before, r_F, devof, R, S, cfg),
                   window='holds_undetermined')
        return out
    out.update(final=('holds' if r_F <= lowest else 'red', ''), window='determined')
    return out


def true_occupant_is_enumerated(truth_txg, region, slot, r, r_inst, rows, floor_before, jv, R, S, cfg):
    """抬 F 那一刻这一槽真住的 txg（None：空着）在不在今天的枚举里。交回 True / False / None（这一槽认不出，不计）。
    真住户空着或 txg 低于抬之前的 F（无效）时，枚举里有一项「空着」或一项 txg 低于抬之前的 F 的根就算罩住。"""
    got = possible_roots(region, slot, largest_below(region, slot, r, R, S), r, r_inst, rows, floor_before, jv, R, S, cfg)
    if isinstance(got, tuple):
        return None
    if truth_txg is None or truth_txg < floor_before:
        return any(c is None or c[0] < floor_before for c in got)
    return any(c is not None and c[0] == truth_txg for c in got)

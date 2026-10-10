#!/usr/bin/env python3
# 用法：python3 -I f2_differential.py [种子]   —— 不依赖仓、不依赖 SINGLEFS_REPO。把 F2_NEWEST 的两份实现各逐支转写一遍（读码转写，不是调原代码），在同一批随机小状态上比：
#   cpu_f2：TREE/crates/singlefs-core/src/recovery.rs 的 newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to；
#   gpu_f2：TREE/crates/singlefs-checker-tier/src/crash_judge_recovery.wgsl 的同名函数。
# 状态：所选根 / 所选那一版、根环里读得出的根（带槽号，可重复 (txg, 实例)）、扫出的记录、每块盘两个系统配置槽（自证过的 tail 或没自证过、读错）、所选根的实例表（读不出 / 行表）。
# 输出（末尾几行）：CASES、MISMATCH、FALLBACK-FOUND、GPU-UNDECIDABLE-SKIPPED；再对 gpu_f2 逐个打进弄坏开关，每个必须 MISMATCH > 0（自证会红）。
import random, sys
seed = int(sys.argv[1]) if len(sys.argv) > 1 else 20261008

def cpu_f2(chosen, selected, roots, records, slots, table):
    """chosen/selected: (instance, txg)；roots: [(tag, instance, txg)]；records: [(instance, txg, counter, last)]；
    slots: 每块盘 [(self_verified_tail 或 None, read_error)]*2；table: None（读不出）或 [(instance, selected_root_txg)]。交回 None 或 root 三元组。"""
    if (selected[0], selected[1]) != (chosen[0], chosen[1]):
        return None
    # system_configuration_witness_is_undecidable：有槽报读错，且没有一块盘两槽都自证过
    any_read_error = any(err for dev in slots for (_, err) in dev)
    both_verified = any(all(tail is not None for (tail, _) in dev) for dev in slots)
    if any_read_error and not both_verified:
        return None
    tails = [tail for dev in slots for (tail, _) in dev if tail is not None]
    witnessed_counter = max(tails) if tails else 0
    if witnessed_counter == 0:
        return None
    order = lambda instance, txg: (txg, instance)
    last_counters = [r[2] for r in records if r[0] == selected[0] and r[1] == selected[1] and r[3]]
    last_counter = max(last_counters) if last_counters else None
    at_counter = [order(r[0], r[1]) for r in records if r[2] == witnessed_counter]
    record_at_counter = max(at_counter) if at_counter else None
    if last_counter is not None:
        is_witnessed = last_counter <= witnessed_counter
    elif record_at_counter is not None:
        is_witnessed = record_at_counter >= order(*selected)
    else:
        is_witnessed = True
    if is_witnessed:
        return None
    if record_at_counter is None:
        return None
    chosen_order = order(*chosen)
    def abandoned(instance, txg):
        return any(row[0] == instance and txg > row[1] for row in table)
    best = None
    for root in roots:
        o = order(root[1], root[2])
        kept = (not abandoned(root[1], root[2])) if table is not None else (o == record_at_counter)
        if record_at_counter <= o and o < chosen_order and kept:
            if best is None or order(best[1], best[2]) <= o:   # max_by_key：并列取最后
                best = root
    return best

def gpu_f2(chosen, selected, roots, records, slots, table, break_=None):
    """返回 ('undecidable',) 或 None 或 root。弄坏开关 break_ 只改这一份。"""
    def lt64(a, b): return a < b
    def precedes(et, ei, lt, li): return lt64(et, lt) or (et == lt and ei < li)
    if selected[0] != chosen[0] or selected[1] != chosen[1]:
        return None
    some_error = False
    some_device_both = False
    witnessed = 0
    for dev in slots:
        verified = 0
        for (tail, err) in dev:
            if err:
                some_error = True
                continue
            if tail is None:
                continue
            verified += 1
            if lt64(witnessed, tail):
                witnessed = tail
        if verified == 2:
            some_device_both = True
    if some_error and not some_device_both:
        return None
    has_last = False; last_counter = 0; has_w = False; w_txg = 0; w_inst = 0
    for (ri, rt, rc, rl) in records:
        if ri == selected[0] and rt == selected[1] and rl and (not has_last or lt64(last_counter, rc)):
            has_last = True; last_counter = rc
        if rc == witnessed and (not has_w or lt64(w_txg, rt) or (w_txg == rt and w_inst < ri)):
            has_w = True; w_txg = rt; w_inst = ri
    w_older = lt64(w_txg, selected[1]) or (w_txg == selected[1] and w_inst < selected[0])
    is_witnessed = True
    if has_last:
        is_witnessed = last_counter <= witnessed
    elif has_w:
        is_witnessed = not w_older
    zero_guard = (witnessed == 0) and break_ != 'no-zero-guard'
    if is_witnessed or zero_guard or not has_w:
        return None
    # load_rows_of_the_instance_table_chain：行多过 512 记判不了
    if table is not None and len(table) > 512:
        return ('undecidable',)
    table_readable = table is not None and break_ != 'ignore-table'
    out = None
    for root in roots:
        is_the_witnessed = (root[1] == w_inst and root[2] == w_txg)
        admissible = is_the_witnessed
        if table_readable:
            admissible = not any(row[0] == root[1] and lt64(row[1], root[2]) for row in table)
        low_ok = (not precedes(root[2], root[1], w_txg, w_inst)) if break_ != 'strict-lower' else precedes(w_txg, w_inst, root[2], root[1])
        in_range = low_ok and precedes(root[2], root[1], chosen[1], chosen[0])
        better = out is None or (not precedes(root[2], root[1], out[2], out[1]) if break_ != 'first-max' else precedes(out[2], out[1], root[2], root[1]))
        if in_range and admissible and better:
            out = root
    return out

def random_state(rng):
    instances, txgs = [0, 1, 2], [1, 2, 3, 4, 5]
    chosen = (rng.choice(instances), rng.choice(txgs[2:]))
    selected = chosen if rng.random() < 0.9 else (rng.choice(instances), rng.choice(txgs))
    roots = []
    for tag in range(rng.randint(0, 7)):
        roots.append((tag, rng.choice(instances), rng.choice(txgs)))
    records = []
    for _ in range(rng.randint(0, 6)):
        records.append((rng.choice(instances), rng.choice(txgs), rng.randint(0, 5), rng.random() < 0.5))
    slots = []
    for _ in range(rng.randint(1, 2)):
        dev = []
        for _ in range(2):
            kind = rng.random()
            dev.append((None, kind < 0.1) if kind < 0.25 else (rng.randint(0, 5), False))
        slots.append(dev)
    table = None if rng.random() < 0.25 else [(rng.choice(instances), rng.choice(txgs)) for _ in range(rng.randint(0, 3))]
    return chosen, selected, roots, records, slots, table

def run(break_=None, cases=400000):
    rng = random.Random(seed)
    mismatches = found = skipped = 0
    for _ in range(cases):
        state = random_state(rng)
        a = cpu_f2(*state)
        b = gpu_f2(*state, break_=break_)
        if b == ('undecidable',):
            skipped += 1
            continue
        found += a is not None
        mismatches += a != b
    return cases, mismatches, found, skipped

cases, mismatches, found, skipped = run()
print(f'CASES {cases}')
print(f'MISMATCH {mismatches}')
print(f'FALLBACK-FOUND {found}')
print(f'GPU-UNDECIDABLE-SKIPPED {skipped}')
for name in ('no-zero-guard', 'ignore-table', 'strict-lower', 'first-max'):
    _, m, _, _ = run(name)
    print(f'MUTANT {name} MISMATCH>0 {m > 0}')
sys.exit(0 if mismatches == 0 and found > 1000 else 1)

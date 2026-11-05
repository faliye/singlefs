"""Q1（C934）：「记录落了、根没落」一形在丁的重建里怎么走，以及假如它被认作那一刻的住户，重建出的上限与真上限谁高。推的小模型，不跑 crates。

一、场景（S = 4，三个区域，区域→盘 (0, 1, 0)）：移植 walk.rs root_in_the_ring_slot_at_the_raise（8986 起）的认槽规则，
    写者动作：实例 1 连发 txg 1..20（根都落了盘）；txg 21 的 journal 记录落了、根槽没落，进程崩；下一次挂载：择根 txg 20，
    施加记录 21，所选那一版 (1, 21)，写行 (1, T=21, W)（mount.rs 9230 的 previous_row.selected_root_txg = effective_root.checkpoint_txg），
    新实例 2 的第一个 txg = max(根环 20, 记录 21) + 1 = 22（mount.rs 1403 起 first_txg_of_new_instance）；实例 2 接着发 22..33；
    抬 F 的根是 r = 26。镜像：txg 33 盖掉了槽 (0, 3)（21 和 9 住的那一槽）。
    三种实例表里的行 T：T = 21（今天的写法）、T = 22（主 agent 题面里「T = u + 1」的读法，只当假设）。
二、方向：写者 admission 的 20 000 段合法历史（同 q2），在抬 F 的那一刻、根没落的版本 u 的槽还住着更旧的根 v 时，
    把 v 换成 u 重算上限（丁若错认 u 是住户会得到的），数重建出的上限比真上限高 / 相等 / 低。
"""
import os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)) + '/../q2')
import q2_three_simplifications as q2

R = 3
S = 4
RING = R * S


def largest_positive_txg_below(region, slot, raising_txg):
    residue = region + R * slot
    below = raising_txg - 1
    if below < residue:
        return None
    largest = residue + ((below - residue) // RING) * RING
    return largest if largest > 0 else None


def recognise(slot, image_slot, journal, rows, raising, floor_before):
    """walk.rs root_in_the_ring_slot_at_the_raise：返回 ('root', txg, inst) / ('empty',) / ('err', 原因)"""
    region, sl = slot
    kind = image_slot[0]
    if kind == 'older':
        return ('root', image_slot[1], image_slot[2])
    if kind == 'zero':
        return ('empty',)
    if kind == 'unreadable':
        return ('err', '读不出')
    raising_txg, raising_instance = raising
    u = largest_positive_txg_below(region, sl, raising_txg)
    if u is None:
        return ('empty',) if (sl != 0 or floor_before > 0) else ('err', '第 0 代根')
    versions = journal.get(u, [])
    if len(versions) != 1:
        return ('err', 'journal 里 txg %d 有 %d 个实例带提交标记的记录' % (u, len(versions)))
    inst = versions[0]
    my_rows = [t for (i, t) in rows if i == inst]
    landed = inst == raising_instance or (my_rows and all(u < t for t in my_rows))
    if not landed:
        return ('err', 'txg %d 那一版是实例 %d 的最后一版（行 T=%s），根落没落盘判不了' % (u, inst, my_rows))
    return ('root', u, inst)


STATE_SETS = {
    # 非空发布（树状态变了）的 txg 集合；三组各让 T = 22 假设下重建的上限 高于 / 等于 / 低于 真上限
    '高于': {3, 5, 6, 7, 9, 12, 13, 23, 25, 26, 29, 32},
    '等于': {2, 3, 5, 8, 14, 15, 18, 19, 23, 26, 27, 28, 30},
    '低于': {1, 4, 5, 9, 12, 15, 18, 19, 20, 21, 23, 24, 25, 28, 30, 31, 32},
}


def scene(nonempty_at):
    state = {}
    cur = 0
    for t in range(0, 34):
        if t in nonempty_at:
            cur += 1
        state[t] = cur
    inst_of = {t: (1 if t <= 21 else 2) for t in range(1, 34)}
    landed = {t for t in range(1, 34) if t != 21}
    # 镜像：每槽住的最新一条落了盘的根
    image = {}
    for t in sorted(landed):
        image[(t % R, (t // R) % S)] = t
    r, inst_r = 26, 2
    floor_before = 0
    journal = {t: [inst_of[t]] for t in range(1, 34)}     # journal 环 128 条，全部在
    print('镜像（槽 -> txg）：', {k: image[k] for k in sorted(image)})
    for T in (21, 22):
        rows = [(1, T)]
        print('--- 实例 1 在抬 F 那条根（txg %d，实例 %d）的实例表里的行 T = %d' % (r, inst_r, T))
        valid = []
        stuck = None
        per_slot = []
        for reg in range(R):
            for sl in range(S):
                t_img = image.get((reg, sl))
                if t_img is None:
                    seen = ('zero',)
                elif t_img < r:
                    seen = ('older', t_img, inst_of[t_img])
                else:
                    seen = ('overwritten',)
                res = recognise((reg, sl), seen, journal, rows, (r, inst_r), floor_before)
                per_slot.append(((reg, sl), seen[0], res))
                if res[0] == 'err':
                    stuck = stuck or ((reg, sl), res[1])
                elif res[0] == 'root':
                    t = res[1]
                    valid.append((t, res[2], 0, state[t], reg))
        print('   逐槽（槽 镜像上的样子 -> 认出）：' + '；'.join(
            '%s %s -> %s' % (slot, {'older': '比抬 F 那条小', 'overwritten': '被盖', 'zero': '全零'}[kind],
                             ('txg %d' % res[1]) if res[0] == 'root' else ('空' if res[0] == 'empty' else '认不出'))
            for (slot, kind, res) in per_slot))
        if stuck:
            print('   丁的结局：环重建不出（部分没判），第一处卡在槽 %s：%s' % stuck)
            continue
        c_hyp = q2.ceiling(valid)
        # 真环：u=21 没落，槽 (0,3) 在 r 之前住着 9
        true_roots = []
        for reg in range(R):
            for sl in range(S):
                cand = [t for t in landed if t < r and (t % R, (t // R) % S) == (reg, sl)]
                if cand:
                    t = max(cand)
                    true_roots.append((t, inst_of[t], 0, state[t], reg))
        c_true = q2.ceiling(true_roots)
        print('   丁的结局：认出住户，槽 (0,3) 认作 txg 21；重建出的上限 %d，真上限 %d' % (c_hyp, c_true))
        print('   重建的环 txg：', sorted(v[0] for v in valid), '真环 txg：', sorted(v[0] for v in true_roots))


def scenes():
    for name, ne in STATE_SETS.items():
        print('=== 状态组「%s」：非空发布的 txg = %s' % (name, sorted(ne)))
        scene(ne)


def direction(seeds):
    out = {}
    example = {}
    for Sv in (4, 8):
        tally = {'higher': 0, 'equal': 0, 'lower': 0, 'lower_makes_legal_raise_red': 0, 'lower_v_invalid': 0, 'lower_v_valid': 0, 'cases': 0}
        for seed in range(seeds):
            steps = 20 + (seed * 7919) % 41
            h = q2.History(seed, Sv, steps, 0.35, 0.04, 0.03, 0.08, 'admission', False)
            for r, (roots_then, fold) in h.ring_then.items():
                new_f, f_old, c_writer, legal = h.raises[r]
                valid_true = [x for x in roots_then if x[0] >= fold and not q2.abandoned(x[0], x[1], h.rows)]
                c_true = q2.ceiling(valid_true)
                assert c_true == c_writer
                occupied = {x[4]: x for x in roots_then}      # 仅作占位，下面按槽找
                for u, (inst_u, F_u, st_u) in h.lost.items():
                    if u >= r:
                        continue
                    slot_u = q2.slot_of(u, Sv)
                    v_roots = [x for x in roots_then if q2.slot_of(x[0], Sv) == slot_u and x[0] > 0]
                    if len(v_roots) != 1 or v_roots[0][0] > u:
                        continue      # 那一槽此刻不是住着比 u 更旧的根（空着或已被更新的盖掉）
                    v = v_roots[0]
                    hyp = [x for x in roots_then if x is not v] + [(u, inst_u, F_u, st_u, u % R)]
                    valid_hyp = [x for x in hyp if x[0] >= fold and not q2.abandoned(x[0], x[1], h.rows)]
                    c_hyp = q2.ceiling(valid_hyp)
                    tally['cases'] += 1
                    if c_hyp is None or c_true is None:
                        continue
                    if c_hyp > c_true:
                        tally['higher'] += 1
                    elif c_hyp == c_true:
                        tally['equal'] += 1
                    else:
                        tally['lower'] += 1
                        if v[0] < fold or q2.abandoned(v[0], v[1], h.rows):
                            tally['lower_v_invalid'] += 1
                        else:
                            tally['lower_v_valid'] += 1
                        if new_f > c_hyp:
                            tally['lower_makes_legal_raise_red'] += 1
                            key = 'lower'
                            if key not in example or len(roots_then) < len(example[key][0]):
                                example[key] = (roots_then, v, (u, inst_u, F_u, st_u), fold, c_true, c_hyp, new_f)
        out[Sv] = tally
    return out, example


def main():
    seeds = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    scenes()
    out, example = direction(seeds)
    for Sv, t in out.items():
        print('direction S=%d seeds=%d  根没落的版本 u 所在槽此刻住着更旧的根 v 的 (抬 F, u) 对数 = %d：重建上限高于真上限 %d，相等 %d，低于真上限 %d（其中让合法抬 F 被判红 %d；低于的里面真住户 v 本来就无效（低于抬之前的 F 或被抛弃）%d 个、v 有效 %d 个）'
              % (Sv, seeds, t['cases'], t['higher'], t['equal'], t['lower'], t['lower_makes_legal_raise_red'], t['lower_v_invalid'], t['lower_v_valid']))
    if 'lower' in example:
        roots_then, v, u, fold, c_true, c_hyp, new_f = example['lower']
        print('反例（重建上限低于真上限）：抬之前的 F = %d，真上限 = 抬到 %d，重建上限 = %d' % (fold, new_f, c_hyp))
        print('  真环里的根 (txg, 实例, F, 状态, 区域)：', sorted(roots_then))
        print('  被换掉的真住户 v：', v, ' 错认的 u (txg, 实例, F, 状态)：', u)


if __name__ == '__main__':
    main()

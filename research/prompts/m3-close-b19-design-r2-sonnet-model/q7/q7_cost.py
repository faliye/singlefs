"""Q7（C717）两条路的代价数：基于攻方腿 k8 小模型的同一套根环/抬 F 取样（自带一份，加记录抬 F 那一刻的环与非空版本表）。
乙（第一轮正推腿原文）= 抬 F 的根 r 的 F 越过抬 F 那一刻环里最旧有效根 O，且窗口口径判成立 ⇒ 部分没判（literal）；
乙收   = 乙再加一句「窗口里可见的非空状态不足 4 个」才报（只有这时窗口口径的上限才可能比当时整环的高，narrow）；
hole   = 攻方 k8 条件（洞里留旧根 X，F 在 (X, W]），只作对照；
丁     = 乙收落进的盲带里，若抬 F 那一刻环里「现在已被盖掉的非空版本」都还在 journal 里（只留最近 J 个非空发布的记录），
         重建当时的环、按整环口径判红绿，否则退回乙收的部分没判。
丁的重建假设：被盖掉的零单元发布按 txg 连号推得出、不影响非空计数。因此 J 够深时丁的红绿与写者当时的上限逐字相同——
这是构造出来的，不是证据；数里能当证据的只有「判得到的盘面比例」随 J 的变化。
推的模型：只有根环、txg、实例、F、树状态号；不模型单元、记账、journal 内容、读故障、真实 J 与每次发布的记录条数。"""
import random, sys

R = 3

def slot_of(txg, S):
    return (txg % R, (txg // R) % S)

def ceiling(roots_before, devices_of_region):
    if not roots_before:
        return None
    rs = sorted(roots_before)
    newest_per_device = {}
    for (t, i, f, st) in rs:
        d = devices_of_region[t % R]
        newest_per_device[d] = max(newest_per_device.get(d, -1), t)
    newest_on_every_device = min(newest_per_device.values())
    prev = None
    non_empty = []
    for (t, i, f, st) in rs:
        if st != prev:
            non_empty.append((t, st))
        prev = st
    counted, seen = [], []
    for (t, st) in sorted(non_empty, reverse=True):
        if st not in seen:
            seen.append(st); counted.append(t)
    fourth = counted[3] if len(counted) >= 4 else rs[0][0]
    return min(newest_on_every_device, fourth)

def count_nonempty_states(roots):
    prev = None; seen = set()
    for (t, i, f, st) in sorted(roots):
        if st != prev:
            seen.add(st)
        prev = st
    return len(seen)

def run(seed, S, devs, steps, p_nonempty, p_crash_lost_root, p_raise, writer_ok):
    rng = random.Random(seed)
    ring = {}; inst = 1; rows = {}; F = 0; state = 0; next_state = 1; txg = 0
    raises = []            # (txg, inst, newF, oldF, c_true, ring_at_raise)
    nonempty_txgs = []     # 所有发布（含根槽没落的）里树状态变了的 txg
    for _ in range(steps):
        r = rng.random()
        if r < p_crash_lost_root:
            txg += 1
            if rng.random() < p_nonempty:
                state = next_state; next_state += 1; nonempty_txgs.append(txg)
            rows[inst] = txg; inst += 1; txg += 1
            ring[slot_of(txg, S)] = (txg, inst, F, state)
            continue
        if r < p_crash_lost_root + p_raise:
            valid = [v for v in ring.values() if v[0] < txg + 1 and v[0] >= F and not (v[1] in rows and v[0] > rows[v[1]])]
            c = ceiling(valid, devs)
            if c is not None and c > F:
                newF = rng.randint(F + 1, c) if writer_ok else rng.randint(F + 1, c + 3)
                txg += 1
                raises.append((txg, inst, newF, F, c, dict(ring)))
                F = newF
                ring[slot_of(txg, S)] = (txg, inst, F, state)
                continue
        txg += 1
        if rng.random() < p_nonempty:
            state = next_state; next_state += 1; nonempty_txgs.append(txg)
        ring[slot_of(txg, S)] = (txg, inst, F, state)
    return ring, rows, raises, nonempty_txgs

def visible_raises(ring, rows, S, devs, raise_by_txg):
    roots = sorted(ring.values()); N = roots[-1][0]; out = []
    for (t, i, f, st) in roots:
        prev = [x for x in roots if x[1] == i and x[0] < t]
        if not prev:
            continue
        fold = max(prev)[2]
        if f <= fold:
            continue
        valid = [x for x in roots if x[0] < t and x[0] >= fold and not (x[1] in rows and x[0] > rows[x[1]])]
        win = [x for x in valid if x[0] > N - 3 * S]
        hole = [x for x in valid if x[0] <= N - 3 * S]
        cw = ceiling(win, devs)
        red_w = cw is not None and f > cw
        blind = bool(win and hole and cw is not None and min(x[0] for x in hole) < f <= cw)
        literal = narrow = False
        if t in raise_by_txg and cw is not None:
            ring_then = raise_by_txg[t][5]
            valid_then = [v for v in ring_then.values() if v[0] >= fold and not (v[1] in rows and v[0] > rows[v[1]])]
            O_then = min(v[0] for v in valid_then) if valid_then else None
            if O_then is not None and f > O_then and not red_w:
                literal = True
                narrow = count_nonempty_states(win) < 4
        out.append(dict(t=t, f=f, cw=cw, red_w=red_w, blind=blind, literal=literal, narrow=narrow))
    return out, N

def covered(raise_info, ring_final, nonempty_txgs, J):
    """抬 F 那一刻环里「现在已不在环里」的非空版本，是否都在最近 J 个非空发布里"""
    final_txgs = {v[0] for v in ring_final.values()}
    recent = set(nonempty_txgs[-J:]) if J > 0 else set()
    nonempty = set(nonempty_txgs)
    for v in raise_info[5].values():
        if v[0] in final_txgs:
            continue
        if v[0] in nonempty and v[0] not in recent:
            return False
    return True

def main():
    seeds = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    Js = (3, 6, 12, 24, 64)
    for S in (4, 8):
        devs = (0, 1, 0)
        n = ev = 0; ev_hole = ev_lit = ev_nar = 0; img_hole = img_lit = img_nar = img_vis = 0
        img_partial_ding = {J: 0 for J in Js}
        for seed in range(seeds):
            steps = 20 + (seed * 7919) % 41
            ring, rows, raises, ne = run(seed, S, devs, steps, 0.35, 0.04, 0.08, True)
            byt = {r[0]: r for r in raises}
            vis, N = visible_raises(ring, rows, S, devs, byt)
            n += 1; ev += len(vis); img_vis += bool(vis)
            hole = [v for v in vis if v["blind"]]; lit = [v for v in vis if v["literal"]]; nar = [v for v in vis if v["narrow"]]
            ev_hole += len(hole); ev_lit += len(lit); ev_nar += len(nar)
            img_hole += bool(hole); img_lit += bool(lit); img_nar += bool(nar)
            for J in Js:
                if any(not covered(byt[v["t"]], ring, ne, J) for v in nar):
                    img_partial_ding[J] += 1
        print(f"LEGAL S={S} images={n} images_with_a_visible_raise={img_vis} visible_raise_events={ev} "
              f"hole_form(opus k8 condition): events={ev_hole} images={img_hole} | "
              f"yi_literal(F>O): events={ev_lit} images={img_lit} | yi_narrow(F>O and window has <4 nonempty states): events={ev_nar} images={img_nar} | "
              + " ".join(f"ding_J{J}_images_still_partial={img_partial_ding[J]}" for J in Js))
        gt = win_red = win_false = 0
        yi_lit_red = yi_lit_part = yi_nar_red = yi_nar_part = missed_by_narrow = 0
        ding_red = {J: 0 for J in Js}; ding_part = {J: 0 for J in Js}
        for seed in range(seeds):
            steps = 20 + (seed * 7919) % 41
            ring, rows, raises, ne = run(seed, S, devs, steps, 0.35, 0.04, 0.08, False)
            byt = {r[0]: r for r in raises}
            vis, N = visible_raises(ring, rows, S, devs, byt)
            viol = [v for v in vis if v["t"] in byt and v["f"] > byt[v["t"]][4]]
            has_viol = bool(viol)
            gt += has_viol
            wr = any(v["red_w"] for v in vis)
            win_red += wr
            if wr and not has_viol:
                win_false += 1
            if wr:
                yi_lit_red += 1; yi_nar_red += 1
            else:
                yi_lit_part += any(v["literal"] for v in vis)
                yi_nar_part += any(v["narrow"] for v in vis)
                if has_viol and not any(v["narrow"] for v in vis):
                    missed_by_narrow += 1
            for J in Js:
                red = wr; part = False
                if not red:
                    for v in vis:
                        if v["narrow"] and v["t"] in byt:
                            if covered(byt[v["t"]], ring, ne, J):
                                if v["f"] > byt[v["t"]][4]:
                                    red = True
                            else:
                                part = True
                ding_red[J] += red
                ding_part[J] += (not red) and part
        print(f"BAD   S={S} images={seeds} images_with_a_true_violation_visible={gt} window_red={win_red} window_red_without_violation={win_false} "
              f"yi_literal: red={yi_lit_red} partial_not_red={yi_lit_part} | yi_narrow: red={yi_nar_red} partial_not_red={yi_nar_part} true_violations_neither_red_nor_flagged={missed_by_narrow} | "
              + " ".join(f"ding_J{J}: red={ding_red[J]} partial={ding_part[J]}" for J in Js))

if __name__ == "__main__":
    main()

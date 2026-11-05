"""Q2（C934）在辩方模型（第二轮 q7_cost.py）的历史与口径上量三处简化：同一批历史（run 抄过来、只多记每一版的 (实例, F, 状态, 落没落)，
先断言与原 run 逐字相同），同一个「图像里有没有抬 F 的违例被判红」的数法；丁的那一判换成 q2_three_simplifications.judge 的按槽版本。
辩方模型自己的丁（covered）假设「J 够深就逐字重建」，没有行 T、实例、落没落盘这些变量；换成按槽的认法之后才有三处简化可放可不放。
journal 口径照辩方模型：只留最近 J 个非空发布的记录，零单元发布（状态没变的）当推得出。
只在 window 判不出、且落进辩方模型 narrow 条件（F > 抬 F 那一刻整环最旧有效根且窗口里可见的非空状态不足 4 个）的抬 F 根上用丁，别的照辩方模型原样。
"""
import importlib.util, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import q2_three_simplifications as m

OPUS_DEFENSE = sys.argv[1]
spec = importlib.util.spec_from_file_location('defense_q7', OPUS_DEFENSE)
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)
R = 3


def run_with_versions(seed, S, devs, steps, p_nonempty, p_crash_lost_root, p_raise, writer_ok):
    import random
    rng = random.Random(seed)
    ring = {}; inst = 1; rows = {}; F = 0; state = 0; next_state = 1; txg = 0
    raises = []; nonempty_txgs = []; versions = {}
    for _ in range(steps):
        r = rng.random()
        if r < p_crash_lost_root:
            txg += 1
            if rng.random() < p_nonempty:
                state = next_state; next_state += 1; nonempty_txgs.append(txg)
            versions[txg] = dict(inst=inst, F=F, state=state, landed=False)
            rows[inst] = txg; inst += 1; txg += 1
            ring[d.slot_of(txg, S)] = (txg, inst, F, state)
            versions[txg] = dict(inst=inst, F=F, state=state, landed=True)
            continue
        if r < p_crash_lost_root + p_raise:
            valid = [v for v in ring.values() if v[0] < txg + 1 and v[0] >= F and not (v[1] in rows and v[0] > rows[v[1]])]
            c = d.ceiling(valid, devs)
            if c is not None and c > F:
                newF = rng.randint(F + 1, c) if writer_ok else rng.randint(F + 1, c + 3)
                txg += 1
                raises.append((txg, inst, newF, F, c, dict(ring)))
                F = newF
                ring[d.slot_of(txg, S)] = (txg, inst, F, state)
                versions[txg] = dict(inst=inst, F=F, state=state, landed=True)
                continue
        txg += 1
        if rng.random() < p_nonempty:
            state = next_state; next_state += 1; nonempty_txgs.append(txg)
        ring[d.slot_of(txg, S)] = (txg, inst, F, state)
        versions[txg] = dict(inst=inst, F=F, state=state, landed=True)
    return ring, rows, raises, nonempty_txgs, versions


class Shim:
    """q2_three_simplifications.judge 要的那几样。"""
    def __init__(self, S, ring, rows, versions):
        self.S = S
        self.genesis = False
        self.rows = rows
        self.versions = versions
        self.ring = {d.slot_of(v[0], S): v[0] for v in ring.values()}
        self.roots = [(v[0], v[1], v[2], v[3], d.slot_of(v[0], S)[0]) for v in ring.values()]


def journal_of(nonempty_txgs, J, top):
    recent = set(nonempty_txgs[-J:])
    ne = set(nonempty_txgs)
    return {t for t in range(1, top + 1) if (t not in ne) or (t in recent)}


def main():
    seeds = int(sys.argv[2]) if len(sys.argv) > 2 else 20000
    devs = (0, 1, 0)
    Js = (12, 24, 64)
    modes = m.MODES
    for S in (4, 8):
        for ok in (True, False):
            tally = {}
            n_images = n_viol_images = 0
            for seed in range(seeds):
                steps = 20 + (seed * 7919) % 41
                ring, rows, raises, ne = d.run(seed, S, devs, steps, 0.35, 0.04, 0.08, ok)
                ring2, rows2, raises2, ne2, versions = run_with_versions(seed, S, devs, steps, 0.35, 0.04, 0.08, ok)
                assert (ring, rows, raises, ne) == (ring2, rows2, raises2, ne2), seed
                byt = {r[0]: r for r in raises}
                vis, N = d.visible_raises(ring, rows, S, devs, byt)
                h = Shim(S, ring, rows, versions)
                viol = any(v['t'] in byt and v['f'] > byt[v['t']][4] for v in vis)
                n_images += 1
                n_viol_images += viol
                for J in Js:
                    jset = journal_of(ne, J, max(versions))
                    for name, mode in modes:
                        red = any(v['red_w'] for v in vis)
                        part = False
                        if not red:
                            for v in vis:
                                if v['narrow'] and v['t'] in byt:
                                    res = m.judge(h, v['t'], jset, mode)
                                    if res == 'red':
                                        red = True
                                    elif res == 'partial':
                                        part = True
                        key = ('bad' if not ok else 'legal')
                        if key == 'bad':
                            tally[(J, name, 'red_images')] = tally.get((J, name, 'red_images'), 0) + red
                            tally[(J, name, 'partial_images')] = tally.get((J, name, 'partial_images'), 0) + ((not red) and part)
                        else:
                            tally[(J, name, 'partial_images')] = tally.get((J, name, 'partial_images'), 0) + part
                            tally[(J, name, 'red_images')] = tally.get((J, name, 'red_images'), 0) + red
            if ok:
                print(f"defense LEGAL S={S} images={n_images}")
            else:
                print(f"defense BAD   S={S} images={n_images} images_with_a_true_violation_visible={n_viol_images}")
            for J in Js:
                for name, mode in modes:
                    print(f"  S={S} {'legal' if ok else 'bad'} J={J} mode={mode} [{name}] red_images={tally[(J, name, 'red_images')]} partial_images={tally[(J, name, 'partial_images')]}")
            sys.stdout.flush()


if __name__ == '__main__':
    main()

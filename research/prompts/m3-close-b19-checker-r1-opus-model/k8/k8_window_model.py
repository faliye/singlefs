"""K8（C717）小模型：根环 R 区域 × S 槽，按 D16 已定项 1 的准入抬 F 上限在抬 F 那一刻用整环算（分配器口径），
之后在最后那张镜像上按三种口径重算：锚在镜像最新根上的一圈窗口（现行 checker）、整环（看得见洞里 X）、
以及 C717 要求的「看得见洞里 X」那一格条件。只模型根环与 txg、实例、F、两棵树根指针（用一个状态号代表），
不模型单元、记账、journal 内容；读故障不模型。推的模型，不是入库装置。"""
import random, sys

R = 3

def slot_of(txg, S):
    return (txg % R, (txg // R) % S)

def ceiling(roots_before, devices_of_region):
    """roots_before: [(txg, instance, F, state)] 已滤好有效、txg 比抬 F 那条小。按 walk.rs rollback_floor_ceiling_from。"""
    if not roots_before:
        return None
    rs = sorted(roots_before)
    newest_per_device = {}
    for (t, i, f, st) in rs:
        d = devices_of_region[t % R]
        newest_per_device[d] = max(newest_per_device.get(d, -1), t)
    newest_on_every_device = min(newest_per_device.values())
    prev = None  # ABSENT
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

def run(seed, S, devices_of_region, steps, p_nonempty, p_crash_lost_root, p_raise, writer_ok=True):
    rng = random.Random(seed)
    ring = {}            # slot -> (txg, inst, F, state, marker)
    inst = 1
    rows = {}            # 实例 -> T（被抛弃判定用）
    F = 0
    state = 0
    next_state = 1
    txg = 0
    for s in range(S):   # mkfs 第 0 代种进每个区域槽 0
        pass
    raises = []          # (txg, inst, F_new, F_old, allocator_ceiling)
    last_applied = 0
    for _ in range(steps):
        r = rng.random()
        if r < p_crash_lost_root:
            # 这一版的记录落了、根槽没落：恢复由记录施加出这一版，新实例接在环里最大 + 1
            txg += 1
            if rng.random() < p_nonempty:
                state = next_state; next_state += 1
            rows[inst] = txg
            inst += 1
            txg += 1  # 新实例第一条根（写行，树不变 ⇒ 空）
            ring[slot_of(txg, S)] = (txg, inst, F, state)
            continue
        if r < p_crash_lost_root + p_raise:
            valid = [v for v in ring.values() if v[0] < txg + 1 and v[0] >= F and not (v[1] in rows and v[0] > rows[v[1]])]
            c = ceiling(valid, devices_of_region)
            if c is not None and c > F:
                if writer_ok:
                    newF = rng.randint(F + 1, c)
                else:
                    newF = rng.randint(F + 1, c + 3)
                txg += 1
                raises.append((txg, inst, newF, F, c))
                F = newF
                ring[slot_of(txg, S)] = (txg, inst, F, state)
                continue
        txg += 1
        if rng.random() < p_nonempty:
            state = next_state; next_state += 1
        ring[slot_of(txg, S)] = (txg, inst, F, state)
    return ring, rows, raises, F

def judge(ring, rows, S, devices_of_region, mode):
    """在镜像上判每条抬 F 的根；mode = window / whole。返回 [(txg, F, 上限, 红?)]"""
    roots = sorted(ring.values())
    N = roots[-1][0]
    out = []
    for (t, i, f, st) in roots:
        prev = [x for x in roots if x[1] == i and x[0] < t]
        if not prev:
            continue
        fold = max(prev)[2]
        if f <= fold:
            continue
        cand = [x for x in roots if x[0] < t and x[0] >= fold and not (x[1] in rows and x[0] > rows[x[1]])]
        if mode == "window":
            cand = [x for x in cand if x[0] > N - 3 * S]
        c = ceiling(cand, devices_of_region)
        if c is None:
            continue
        out.append((t, f, c, f > c))
    return out

def k8_condition(ring, rows, S, devices_of_region):
    """C717 要红的那一格：洞里（窗口外）留着有效根 X，窗口里非空有效根不足 4 个，F 落在 (X, W]（W = 窗口口径的上限）。"""
    roots = sorted(ring.values())
    N = roots[-1][0]
    hits = []
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
        if not win or not hole:
            continue
        W = ceiling(win, devices_of_region)
        X = min(x[0] for x in hole)
        if X < f <= W:
            hits.append((t, f, X, W))
    return hits

def main():
    seeds = int(sys.argv[1]) if len(sys.argv) > 1 else 20000
    totals = {}
    example = None
    for S in (4, 8):
        for devs in ((0, 1, 0), (0, 1, 1)):
            key = (S, devs)
            legal_images = whole_red = window_red = k8_hit = 0
            for seed in range(seeds):
                ring, rows, raises, F = run(seed, S, devs, steps=rand_steps(seed), p_nonempty=0.35,
                                            p_crash_lost_root=0.04, p_raise=0.08)
                legal_images += 1
                w = judge(ring, rows, S, devs, "window")
                a = judge(ring, rows, S, devs, "whole")
                if any(r[3] for r in w):
                    window_red += 1
                if any(r[3] for r in a):
                    whole_red += 1
                h = k8_condition(ring, rows, S, devs)
                if h:
                    k8_hit += 1
                    if example is None:
                        example = (S, devs, seed, h, sorted(ring.values()), rows, raises)
            totals[key] = (legal_images, window_red, whole_red, k8_hit)
            print(f"S={S} region_devices={devs} legal_images={legal_images} window_red={window_red} whole_ring_red={whole_red} legal_images_meeting_the_c717_must_red_condition={k8_hit}")
    if example:
        S, devs, seed, h, roots, rows, raises = example
        print(f"EXAMPLE S={S} region_devices={devs} seed={seed}")
        print(f"  c717_condition_hits(raise_txg, F, X, W)={h}")
        print(f"  raises_at_write_time(txg, inst, F_new, F_old, allocator_ceiling_whole_ring_at_raise)={raises}")
        print(f"  rows(instance->T)={rows}")
        print(f"  image_roots(txg, inst, F, state)={roots}")

def rand_steps(seed):
    return 20 + (seed * 7919) % 41

if __name__ == "__main__":
    main()

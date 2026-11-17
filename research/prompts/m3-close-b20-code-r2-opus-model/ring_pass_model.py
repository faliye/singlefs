#!/usr/bin/env python3
"""C973 第 0 代根判据：S 不是 2 的幂、环绕一圈以上、跳号、三槽同错、读回旧内容（攻方腿 m3-close-b20-code-r2 写的算术模型，不调实现）。

照 crates/singlefs-core/src/instance_table_sweep.rs 第 470-552 行与 root_ring.rs 第 167-193 行手抄，与判决引的上一轮模型同一种手抄：
它不是独立校验路子。上一轮取 S=4,5,8,16、错位读源是别的槽此刻的内容、至多两槽。这里多出来的：
  S 取 4..=16 全部；最大 txg 走到三圈多的每一个值；
  跳号（一次发布的记录落盘、根没写：下一个实例的 txg 取 max(环, 记录)+1，根环那一槽不写，mount.rs 第 1522-1534 行）；
  三个槽 0 同时错第一读；
  读回旧内容（同一槽上一次的内容，错位读之外的一种读坏）。
数的：hidden（接受的环里有真根被换掉）、stop:*、consistent；hidden 的态另数「按重读之后的最大 txg 再判一遍能不能拦下」。
用法：python3 ring_pass_model.py [--quick]
"""
import itertools
import sys
import time

GENESIS = (0, 0)


def target(txg, s):
    return (txg % 3, (txg // 3) % s)


def ring_slots(s):
    return [(region, slot) for region in range(3) for slot in range(s)]


def formula_ok(txg, ring_slot, s):
    if txg == 0:
        return ring_slot[1] == 0
    return target(txg, s) == ring_slot


def reached(root, ring_slot, highest, s):
    if root[0] != 0:
        return False
    return any(target(t, s) == ring_slot for t in itertools.islice(range(1, highest + 1), 3 * s))


def disk(s, highest_true, skipped):
    """交回 (每槽此刻的内容, 每槽此刻之前的那一份内容)。内容是 (txg, 实例) 或 None（全零）。每个 txg 当一个独立实例。"""
    now = {slot: None for slot in ring_slots(s)}
    before = {slot: None for slot in ring_slots(s)}
    for region in range(3):
        now[(region, 0)] = GENESIS
    for txg in range(1, highest_true + 1):
        if txg in skipped:
            continue
        slot = target(txg, s)
        before[slot] = now[slot]
        now[slot] = (txg, txg)
    return now, before


class Stop(Exception):
    pass


def judged_bytes(bytes_read, ring_slot, s):
    if bytes_read is None:
        raise Stop("zero")
    return bytes_read if formula_ok(bytes_read[0], ring_slot, s) else None


def sweep(now, s, first_reads, later_reads):
    """first_reads：槽 -> 第一读读回的内容；later_reads：槽 -> 第二读、判据重读读回的内容。不在表里的读此刻的内容。"""
    roots, first_ok = {}, {}
    for ring_slot in ring_slots(s):
        root = judged_bytes(first_reads.get(ring_slot, now[ring_slot]), ring_slot, s)
        first_ok[ring_slot] = root is not None
        if root is None:
            root = judged_bytes(later_reads.get(ring_slot, now[ring_slot]), ring_slot, s)
            if root is None:
                raise Stop("not-verified")
        roots[ring_slot] = root
    highest = max(root[0] for root in roots.values())
    after = dict(roots)
    for ring_slot, root in roots.items():
        if not reached(root, ring_slot, highest, s):
            continue
        if not first_ok[ring_slot]:
            raise Stop("genesis-after-reread")
        reread = judged_bytes(later_reads.get(ring_slot, now[ring_slot]), ring_slot, s)
        if reread is None or reached(reread, ring_slot, highest, s):
            raise Stop("genesis-still")
        after[ring_slot] = reread
    return after, highest


def hidden_slots(now, after):
    return [slot for slot, real in now.items() if real is not None and real[0] > 0 and after[slot] != real]


def tally_one(now, s, first_reads, later_reads, tally, examples, label):
    try:
        after, _highest = sweep(now, s, first_reads, later_reads)
    except Stop as stop:
        key = "stop:" + str(stop)
        tally[key] = tally.get(key, 0) + 1
        return
    hidden = hidden_slots(now, after)
    if not hidden:
        tally["consistent"] = tally.get("consistent", 0) + 1
        return
    tally["hidden"] = tally.get("hidden", 0) + 1
    highest_after = max(root[0] for root in after.values())
    caught = any(reached(after[slot], slot, highest_after, s) for slot in hidden)
    key = "hidden-caught-by-rejudging-after-rereads" if caught else "hidden-not-caught-by-rejudging"
    tally[key] = tally.get(key, 0) + 1
    if len(examples) < 3:
        examples.append(f"{label} first={sorted(first_reads.items())} hidden={hidden} highest_after={highest_after}")


def skip_sets(s):
    lap = 3 * s
    return [
        (),
        (1,),
        (2,),
        (1, 2),
        (lap,),
        (lap + 1,),
        (lap + 2,),
        (1, lap + 1),
        (2, lap + 2),
        (lap, 2 * lap),
        (1, lap + 1, 2 * lap + 1),
    ]


def misread_modes(s, slots_of_zero, slots, now):
    """错位读：三个槽 0 的第一读各取任一槽此刻的内容（S 小时全枚举；S 大时只取住着第 0 代根的槽与它自己，取别的槽只走到自证不过、重读读对）。"""
    genesis_holders = [slot for slot in slots if now[slot] == GENESIS]
    for victim_slots in (slots_of_zero[:1], slots_of_zero[1:2], slots_of_zero[2:], slots_of_zero):
        for sources in itertools.product(
            *[(slots if s <= 6 else sorted(set(genesis_holders + [victim]))) for victim in victim_slots]
        ):
            if all(source == victim for source, victim in zip(sources, victim_slots)):
                continue
            yield {victim: now[source] for victim, source in zip(victim_slots, sources)}


def stale_modes(slots_of_zero, now, before):
    """读回旧内容：三个槽 0 的任一子集第一读读回这一槽上一次的内容。"""
    for size in (1, 2, 3):
        for victims in itertools.combinations(slots_of_zero, size):
            if all(before[victim] is not None for victim in victims):
                yield {victim: before[victim] for victim in victims}


def publishes_until_the_sweep_goes_through(s, skipped):
    """这一组跳号都发生过之后，干净读（没有读坏）的清扫从第几个 txg 起不停；走到四圈还停记 None。"""
    for highest_true in range(max(max(skipped, default=0) + 1, 3 * s - 1), 12 * s + 1):
        now, _before = disk(s, highest_true, skipped)
        try:
            sweep(now, s, {}, {})
        except Stop:
            continue
        return highest_true
    return None


def run(sizes):
    for s in sizes:
        started = time.monotonic()
        slots = ring_slots(s)
        slots_of_zero = [(0, 0), (1, 0), (2, 0)]
        misread, stale, clean = {}, {}, {}
        examples_misread, examples_stale, examples_clean = [], [], []
        states = 0
        for skipped in skip_sets(s):
            for highest_true in range(3 * s - 1, 9 * s + 4):
                if highest_true in skipped:
                    continue
                now, before = disk(s, highest_true, set(skipped))
                label = f"S={s} H_true={highest_true} skipped={skipped}"
                tally_one(now, s, {}, {}, clean, examples_clean, label)
                states += 1
                for first_reads in misread_modes(s, slots_of_zero, slots, now):
                    tally_one(now, s, first_reads, {}, misread, examples_misread, label)
                    states += 1
                for first_reads in stale_modes(slots_of_zero, now, before):
                    tally_one(now, s, first_reads, {}, stale, examples_stale, label)
                    states += 1
        for name, tally in (("clean", clean), ("misread", misread), ("stale", stale)):
            print(f"name=ring_pass S={s} mode={name} " + " ".join(f"{k}={v}" for k, v in sorted(tally.items())))
        for example in examples_clean + examples_misread + examples_stale:
            print(f"example {example}")
        for skipped in skip_sets(s):
            print(
                f"name=sweep_goes_through S={s} skipped={list(skipped)} first_txg_clean_sweep_passes="
                f"{publishes_until_the_sweep_goes_through(s, set(skipped))}"
            )
        print(f"name=ring_pass_cost S={s} states={states} seconds={time.monotonic() - started:.1f}")


def main():
    if "--quick" in sys.argv[1:]:
        run([4, 5])
    else:
        run(list(range(4, 17)))


if __name__ == "__main__":
    main()

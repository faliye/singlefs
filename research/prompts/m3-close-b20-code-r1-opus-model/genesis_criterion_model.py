#!/usr/bin/env python3
"""C973 第 0 代根判据的算术模型（攻方腿自己写的，不调实现、不读仓里代码）。

照 crates/singlefs-core/src/instance_table_sweep.rs 的清扫读根环与 root_ring.rs 的落点公式手抄一份：
  落点 target(t) = (t mod 3, (t div 3) mod S)；第 0 代根（txg 0）只认三个区域的槽 0；
  第一遍逐槽读，全零即停；自证（这里只有落点核）不过重读一次，仍不过即停；
  H = 第一遍读出的根里最大的 txg；住着第 0 代根的槽若 1..=H（至多环长个）里有 txg 落进这一槽，算自证不过：
  第一遍没一次读过的即停，读过的再读一次，仍是第 0 代根（或不合）即停。
盘面：txg 1..H_true 连续发布，各写进落点那一槽；可选丢一次根槽写（说谎的设备）。每个 txg 当一个独立实例（最坏）。
故障：错位读 = 一次读交回另一槽此刻的内容。单槽三次读（第一读、重读、判据重读）各取任一槽；或两槽各错第一读。
数的：接受的环里有一槽的真根被换掉（hidden，等于有实例被当成环里没根）、停、接受且与盘一致；
另数 H 恰好等于落进住第 0 代根那一槽的某个 txg 的次数（equality），与按重读后的最大 txg 重判结局会变的次数（deflation）。
用法：python3 genesis_criterion_model.py [--quick]
"""
import itertools
import sys
import time

GENESIS = (0, 0)  # (txg, 实例)


def target(txg, s):
    return (txg % 3, (txg // 3) % s)


def ring_slots(s):
    return [(region, slot) for region in range(3) for slot in range(s)]


def formula_ok(txg, ring_slot, s):
    if txg == 0:
        return ring_slot[1] == 0
    return target(txg, s) == ring_slot


def genesis_reached(root, ring_slot, highest, s):
    if root[0] != 0:
        return False
    return any(target(t, s) == ring_slot for t in itertools.islice(range(1, highest + 1), 3 * s))


def disk_content(s, highest_true, lost_txg):
    content = {slot: None for slot in ring_slots(s)}
    for region in range(3):
        content[(region, 0)] = GENESIS
    for txg in range(1, highest_true + 1):
        if txg != lost_txg:
            content[target(txg, s)] = (txg, txg)
    return content


class Stop(Exception):
    pass


def read_once(content, ring_slot, source, s):
    bytes_read = content[source]
    if bytes_read is None:
        raise Stop("zero")
    return bytes_read if formula_ok(bytes_read[0], ring_slot, s) else None


def sweep(content, s, plan):
    """plan: 槽 -> (第一读源, 重读源, 判据重读源)，不在 plan 里的槽三读都读自己。交回 (接受的槽->根, H, 重读后最大 txg, equality 次数)。"""
    roots, first_ok = {}, {}
    for ring_slot in ring_slots(s):
        sources = plan.get(ring_slot, (ring_slot, ring_slot, ring_slot))
        root = read_once(content, ring_slot, sources[0], s)
        first_ok[ring_slot] = root is not None
        if root is None:
            root = read_once(content, ring_slot, sources[1], s)
            if root is None:
                raise Stop("not-verified")
        roots[ring_slot] = root
    highest = max(root[0] for root in roots.values())
    equality = 0
    for ring_slot, root in roots.items():
        if root[0] == 0 and target(highest, s) == ring_slot:
            equality += 1
    after = dict(roots)
    for ring_slot, root in roots.items():
        if not genesis_reached(root, ring_slot, highest, s):
            continue
        if not first_ok[ring_slot]:
            raise Stop("genesis-after-reread")
        sources = plan.get(ring_slot, (ring_slot, ring_slot, ring_slot))
        reread = read_once(content, ring_slot, sources[2], s)
        if reread is None or genesis_reached(reread, ring_slot, highest, s):
            raise Stop("genesis-still")
        after[ring_slot] = reread
    highest_after = max(root[0] for root in after.values())
    return after, highest, highest_after, equality


def sweep_judged_with(content, s, plan, highest_forced):
    """同 sweep，判据里的 H 换成给定的值（数 deflation 用）。"""
    roots, first_ok = {}, {}
    for ring_slot in ring_slots(s):
        sources = plan.get(ring_slot, (ring_slot, ring_slot, ring_slot))
        root = read_once(content, ring_slot, sources[0], s)
        first_ok[ring_slot] = root is not None
        if root is None:
            root = read_once(content, ring_slot, sources[1], s)
            if root is None:
                raise Stop("not-verified")
        roots[ring_slot] = root
    after = dict(roots)
    for ring_slot, root in roots.items():
        if not genesis_reached(root, ring_slot, highest_forced, s):
            continue
        if not first_ok[ring_slot]:
            raise Stop("genesis-after-reread")
        sources = plan.get(ring_slot, (ring_slot, ring_slot, ring_slot))
        reread = read_once(content, ring_slot, sources[2], s)
        if reread is None or genesis_reached(reread, ring_slot, highest_forced, s):
            raise Stop("genesis-still")
        after[ring_slot] = reread
    return after


def classify(content, after):
    hidden = [slot for slot, real in content.items() if real is not None and real[0] > 0 and after[slot] != real]
    if hidden:
        return "hidden"
    if any(after[slot] != content[slot] for slot in content):
        return "other-mismatch"
    return "consistent"


def judge(content, s, plan, tally, examples, label):
    try:
        after, highest, highest_after, equality = sweep(content, s, plan)
    except Stop as stop:
        tally["stop:" + str(stop)] = tally.get("stop:" + str(stop), 0) + 1
        return
    verdict = classify(content, after)
    tally[verdict] = tally.get(verdict, 0) + 1
    if equality:
        tally["equality"] = tally.get("equality", 0) + 1
    if highest_after != highest:
        try:
            rejudged = classify(content, sweep_judged_with(content, s, plan, highest_after))
        except Stop:
            rejudged = "stop"
        if rejudged != verdict:
            tally["deflation-changes-verdict"] = tally.get("deflation-changes-verdict", 0) + 1
    if verdict != "consistent" and len(examples) < 5:
        examples.append(f"{label} plan={sorted(plan.items())} verdict={verdict}")


def lost_candidates(s, highest_true):
    picks = {t for t in range(1, highest_true + 1) if t % (3 * s) in (0, 1, 2)}
    picks.add(highest_true)
    return [None] + sorted(picks)


def run(sizes, double_sizes, laps):
    for s in sizes:
        started = time.monotonic()
        slots = ring_slots(s)
        highs = sorted({h for lap in laps for h in (3 * s * lap - 1, 3 * s * lap, 3 * s * lap + 1, 3 * s * lap + 2, 3 * s * lap + 3)})
        tally, examples = {}, []
        sweeps = 0
        for highest_true in highs:
            for lost in lost_candidates(s, highest_true):
                content = disk_content(s, highest_true, lost)
                label = f"S={s} H_true={highest_true} lost={lost}"
                judge(content, s, {}, tally.setdefault("clean", {}), examples, label + " clean")
                sweeps += 1
                single = tally.setdefault("single", {})
                for victim in slots:
                    for first in slots:
                        rereads = slots if first != victim else [victim]
                        for reread in rereads:
                            root = None
                            if content[first] is not None and formula_ok(content[first][0], victim, s):
                                root = content[first]
                            elif content[reread] is not None and formula_ok(content[reread][0], victim, s):
                                root = content[reread]
                            genesis_sources = slots if root == GENESIS else [victim]
                            for genesis_source in genesis_sources:
                                plan = {victim: (first, reread, genesis_source)}
                                if plan[victim] == (victim, victim, victim):
                                    continue
                                judge(content, s, plan, single, examples, label + " single")
                                sweeps += 1
                if s in double_sizes:
                    double = tally.setdefault("double", {})
                    for victim_one, victim_two in itertools.combinations(slots, 2):
                        for first_one in slots:
                            for first_two in slots:
                                if first_one == victim_one and first_two == victim_two:
                                    continue
                                plan = {victim_one: (first_one, victim_one, victim_one), victim_two: (first_two, victim_two, victim_two)}
                                judge(content, s, plan, double, examples, label + " double")
                                sweeps += 1
        elapsed = time.monotonic() - started
        for mode in ("clean", "single", "double"):
            if mode in tally:
                print(f"name=genesis_criterion S={s} mode={mode} " + " ".join(f"{k}={v}" for k, v in sorted(tally[mode].items())))
        print(f"name=genesis_criterion_cost S={s} highs={highs} sweeps={sweeps} seconds={elapsed:.1f}")
        for example in examples:
            print(f"example {example}")


def main():
    quick = "--quick" in sys.argv[1:]
    if quick:
        run(sizes=[4], double_sizes=[4], laps=[1])
    else:
        run(sizes=[4, 5, 8], double_sizes=[4, 5], laps=[1, 2, 3])
        run(sizes=[16], double_sizes=[], laps=[1])


if __name__ == "__main__":
    main()

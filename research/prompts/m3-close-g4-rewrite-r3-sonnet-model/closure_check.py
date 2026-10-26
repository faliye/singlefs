"""可达盘面闭包：从 mkfs 返回后的盘面出发，反复「在任意崩溃状态（含撕裂）上再起一次可写挂载 / 抬 F / 发布」，
把每一次流的全部崩溃状态的盘面收进来，直到不再出新盘面；看收进来的盘面里有没有死盘（一块盘两槽都不自证）。
盘面按盘规范化（世代号减去偶数偏移，使最小的自证世代号落在 0 或 1），不改协议里用到的奇偶与大小关系。"""
import sys
sys.path.insert(0, '.')
import protocol_histories as P
from enumerate_states import states
from slot_flow_model import V, Z, T, valid, Recorder

FLOWS = {
    'acquisition_then_two_publishes': lambda rec: (P.acquisition(rec), P.publish(rec, 3), P.publish(rec, 2)),
    'acquisition_only': lambda rec: P.acquisition(rec),
    'publish_inside_a_mounted_session': lambda rec: (P.publish(rec, 2), P.publish(rec, 2)),
    'raise_floor_then_publish': lambda rec: (P.raise_floor(rec), P.publish(rec, 2)),
    'acquisition_then_raise_floor_then_publish': lambda rec: (P.acquisition(rec), P.raise_floor(rec), P.publish(rec, 2)),
}


def canon_dev(slots):
    gens = [c[1] for c in slots if valid(c)]
    shift = 2 * (min(gens) // 2) if gens else 0
    return tuple(('V', c[1] - shift) if valid(c) else c for c in slots)


def canon(img):
    return tuple(canon_dev(img[d]) for d in sorted(img))


def to_disk(c):
    return {d: list(c[d]) for d in range(len(c))}


def run(flows, start, max_rounds=12):
    seen, frontier, dead_found = {start}, {start}, []
    for rnd in range(max_rounds):
        nxt = set()
        for img in frontier:
            for name, build in flows.items():
                rec = Recorder(to_disk(img))
                build(rec)
                rec.finish()
                for si, dom, dsk, dead, wgt in states(rec):
                    c = canon({d: dsk[d] for d in dsk})
                    if dead:
                        dead_found.append((name, dom, c))
                    if c not in seen:
                        seen.add(c)
                        nxt.add(c)
        if not nxt:
            return seen, dead_found, rnd + 1
        frontier = nxt
    return seen, dead_found, -1


if __name__ == '__main__':
    start = canon({0: [V(0), V(0)], 1: [V(0), V(0)]})
    P.MUT.clear()
    seen, dead, rounds = run(FLOWS, start)
    print(f"name=closure_correct start=post_mkfs flows={len(FLOWS)} fixpoint_rounds={rounds} reachable_images={len(seen)} "
          f"images_with_a_dead_device={sum(1 for img in seen if any(not any(valid(c) for c in dv) for dv in img))} "
          f"dead_states_met={len(dead)}")
    print('torn_images_among_reachable=', sum(1 for img in seen if any(c == T for dv in img for c in dv)))
    for mut in ('B2', 'B1'):
        P.MUT.clear(); P.MUT.add(mut)
        seen_m, dead_m, rounds_m = run(FLOWS, start)
        print(f"name=closure_{mut} fixpoint_rounds={rounds_m} reachable_images={len(seen_m)} dead_states_met={len(dead_m)}")
        kinds = sorted({(n, d) for n, d, _ in dead_m})
        print('  first_dead_flow_kinds=', kinds[:6])
    P.MUT.clear()

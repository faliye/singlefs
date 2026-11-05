#!/usr/bin/env python3
"""Q3 第 2 问的计数模型（纯算术，不读 crates/、不读盘）：读回全零的根槽并进准入抬 F 上限（D16 发布语义 已定项 1 第 38 行）
之后，一个不并全零槽的写者（照旧式抬到 from_read）在池级 checker 的 I-7.9 上判不判得出。

历史形（合法：崩在记录之后、根槽之前，D16 已定项 7 的持久顺序允许）：
  mkfs 在三个区域的槽 0 种第 0 代根（txg 0）。实例 A 可写挂载：写行 txg 1（空）、暖机 txg 2（空），之后 txg 3..h-1 每次写一个文件（各一个新状态）；
  txg h 也是写文件，记录落了、根槽没落，崩溃。h < 3S，h 那一槽从没写过、读回全零。
  实例 B 可写挂载：恢复施加 h（实例表行 (A, h)），N_B = h-1，first_B = h+1；写行 h+1（空，状态承 h），暖机到两块盘都有 B 的根，
  之后每次写文件（新状态），写到 c 时准入不够、抬 F（F 抬之前为 0）。抬 F 那一串推到两块盘都有带新 F 的根；要求那一串的最后一个 txg < h + 3S，
  h 那一槽到判的那一刻仍全零。
写者两种：merge（照 D16 字面：全零槽按它在 (N_B-3S, first_B-1] 里能住的最大 txg 并，这里就是 h）与 drop（不并，抬到 from_read）。
checker：I-7.9 不带卸载记号那一支在这段历史上按抬 F 那一刻的环算（镜像上除 h 那一槽外每版都落了根；窗口外的槽按 journal 认、
同一实例的那一版认作落了盘，重建出的环与真环相同——推的，依据 walk.rs 的 possible_roots_in_the_ring_slot_at_the_raise），全零槽当空着、不并。
打出：每个 S 下历史条数、drop 越过字面上限（from_read > merge 的上限）的条数、其中 checker 仍判成立的条数，与第一条例子。
"""
import sys

R = 3
REGION_DEVICES = (0, 1, 0)


def slot_of(txg, s):
    return (txg % R, (txg // R) % s)


def device_of_slot(slot):
    return REGION_DEVICES[slot[0]]


def ceiling(roots, floor, s):
    """roots：{槽: (txg, 状态)}，抬 F 那一刻环里的根。D16 已定项 1 的准入式子：min(每块盘上最新有效根, 第 4 新的不同状态非空有效根)，不足 4 个取最旧有效根。"""
    valid = sorted((txg, state, slot) for slot, (txg, state) in roots.items() if txg >= floor)
    if not valid:
        return None
    newest = {}
    for txg, _, slot in valid:
        dev = device_of_slot(slot)
        newest[dev] = max(newest.get(dev, -1), txg)
    non_empty = []
    previous = 0  # 最旧那条跟「两棵都没有」比：状态 0 就是第 0 版那样一棵用户可见树都没有
    for txg, state, _ in valid:
        if state != previous:
            non_empty.append((txg, state))
        previous = state
    seen, distinct = set(), []
    for txg, state in sorted(non_empty, reverse=True):
        if state not in seen:
            seen.add(state)
            distinct.append(txg)
    fourth = distinct[3] if len(distinct) >= 4 else valid[0][0]
    return min(min(newest.values()), fourth)


def history(h, c, s):
    """交回 (抬 F 那一刻环里的根, 抬 F 那一串的 txg 表)；不合前提交 None。状态：空发布承上一版的状态，写文件那一版的状态是它自己的 txg。"""
    landed = {}  # txg -> 状态
    state = 0    # mkfs 第 0 版：没有用户可见树
    for txg in range(1, h):
        if txg >= 3:
            state = txg
        landed[txg] = state
    state = h  # h 那一版写了文件、被 B 的恢复施加，根没落
    # B：写行 h+1，暖机到两块盘都有 B 的根
    covered = set()
    txg = h + 1
    landed[txg] = state
    covered.add(device_of_slot(slot_of(txg, s)))
    while len(covered) < 2:
        txg += 1
        landed[txg] = state
        covered.add(device_of_slot(slot_of(txg, s)))
    if c <= txg:
        return None
    for t in range(txg + 1, c + 1):
        landed[t] = t  # 写文件
    raise_txgs, covered, t = [], set(), c
    while len(covered) < 2:
        t += 1
        raise_txgs.append(t)
        covered.add(device_of_slot(slot_of(t, s)))
    if raise_txgs[-1] >= h + R * s:
        return None  # h 那一槽在判之前被盖掉，不在这一形里
    ring = {}
    for r in range(R):
        ring[(r, 0)] = (0, 0)  # 第 0 代根
    for t in sorted(landed):
        ring[slot_of(t, s)] = (t, landed[t])
    return ring, raise_txgs


def main():
    first_example = None
    for s in (4, 8, 16):
        total = exceed = checker_holds = 0
        for h in range(3, R * s):
            if slot_of(h, s)[1] == 0:
                continue  # 第 0 代根那几槽不会读回全零
            for c in range(h + 2, h + R * s):
                built = history(h, c, s)
                if built is None:
                    continue
                ring, raise_txgs = built
                assert slot_of(h, s) not in ring
                floor = 0
                from_read = ceiling(ring, floor, s)
                h_can = h  # (N_B-3S, first_B-1] = (h-1-3S, h] 里落到这一槽的最大 txg
                merged = min(from_read, h_can) if h_can >= floor else from_read
                checker = ceiling(ring, floor, s)
                total += 1
                if from_read > merged and from_read > floor:
                    exceed += 1
                    if from_read <= checker:
                        checker_holds += 1
                        if first_example is None:
                            first_example = (s, h, c, from_read, merged, checker, raise_txgs)
        print(f"E7RESULT name=zero_slot_merge_blind S={s} histories={total} drop_writer_exceeds_the_literal_ceiling={exceed} checker_holds_on_those={checker_holds}")
    if first_example:
        s, h, c, fr, mg, ck, rt = first_example
        print(f"E7RESULT name=first_example S={s} hole_txg={h} raise_at={c} drop_writer_floor={fr} literal_ceiling={mg} checker_ceiling={ck} raise_sequence_txgs={rt}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

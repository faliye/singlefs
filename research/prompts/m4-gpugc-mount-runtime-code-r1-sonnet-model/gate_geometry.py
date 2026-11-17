#!/usr/bin/env python3
"""回退门在不同根环几何 S 上放几次连续的整理发布（纯算术模型，不起项目代码）。

照冻结副本 crates/singlefs-core/src/mount.rs 的 rollback_states_of_the_ring 与
crates/singlefs-core/src/mounted_session.rs 的 stop_of_the_rollback_gate 的判法写：
  根环 3S 槽；下一次发布与之后 MARGIN 次发布写的共 1 + MARGIN 槽里的根走掉，剩下的是 kept；
  状态数 = 把根按 txg 从旧到新排，每条根与前一条（kept 里最旧的一条跟「无」比）指针不同才算非空，
  再按指针去重；整理发布若没搬用户可见树的根单元，它的指针等于前一条（空发布，不算状态）。
  门关 当且仅当 剩下的状态数 < 4 且 < 此刻的状态数。
一条意图在默认批大小下至多 INTENT_PUBLISHES_AT_MOST 次发布（签发 1 次加 ceil(64 / 16) 批，删意图并进最后一批）。
"""
import sys

MARGIN = 8
KEPT_STATES = 4
INTENT_PUBLISHES_AT_MOST = 5


def distinct_states(labels):
    """labels 从旧到新；返回计入的不同状态数。"""
    previous = None
    seen = set()
    for label in labels:
        if label != previous:
            seen.add(label)
        previous = label
    return len(seen)


def gate_is_closed(ring):
    now = distinct_states(ring)
    after = distinct_states(ring[1 + MARGIN:])
    return after < KEPT_STATES and after < now


def publishes_until_the_gate_closes(ring, cap=200):
    ring = list(ring)
    published = 0
    while published < cap and not gate_is_closed(ring):
        ring.append(ring[-1])
        ring.pop(0)
        published += 1
    return published


def busy_ring(slots):
    return list(range(slots))


def ring_with_empties_after_each_state(slots, empties_between):
    ring = []
    label = 0
    while len(ring) < slots:
        ring.append(label)
        ring.extend([label] * empties_between)
        label += 1
    return ring[-slots:]


def verdict(allowed):
    return "yes" if allowed >= INTENT_PUBLISHES_AT_MOST else "no"


def main():
    print("name=gate_geometry margin=%d kept_states=%d intent_publishes_at_most=%d"
          % (MARGIN, KEPT_STATES, INTENT_PUBLISHES_AT_MOST))
    print("scenario=busy_ring")
    for slots_per_region in range(4, 17):
        slots = 3 * slots_per_region
        allowed = publishes_until_the_gate_closes(busy_ring(slots))
        print("S=%d ring=%d allowed=%d formula_3S_minus_12=%d finishes_one_intent=%s"
              % (slots_per_region, slots, allowed, max(0, slots - 12), verdict(allowed)))
    print("scenario=every_user_state_followed_by_j_empty_roots")
    for slots_per_region in (4, 5, 6, 8):
        slots = 3 * slots_per_region
        for empties in (0, 1, 2, 3):
            allowed = publishes_until_the_gate_closes(ring_with_empties_after_each_state(slots, empties))
            print("S=%d j=%d allowed=%d finishes_one_intent=%s"
                  % (slots_per_region, empties, allowed, verdict(allowed)))
    return 0


if __name__ == "__main__":
    sys.exit(main())

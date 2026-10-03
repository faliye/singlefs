#!/usr/bin/env python3
"""B7 / B5：墓碑预付（标签 6）的「每活对象 41 字节 + 全池常驻 3 槽」罩不罩得住墓碑树根长高那一步。

单位：一份副本（每块盘）的字节；一个槽 16384，一个墓碑容器 32 KiB = 2 槽，一个码 2 内部节点 1 槽。
墓碑树只追加、第一版不回收（D18 已定项 24 第 4 条），根满时长高、末尾分裂（D8 已定项 6）：
  m 条墓碑 ⇒ L = ceil(m / C) 个容器；第 1 层内部节点 ceil(L / F) 个，再往上每层 ceil(下一层 / F)，到 1 为止（L ≤ F 时只有根）。
永久占用 perm(m) = 2·L + 内部节点数（槽）。还有 n 个活对象将来要死时，最坏要再永久占 need(m, n) = perm(m + n) − perm(m)。
预付（照 m3-delete-r3 判决 G1-墓碑预付：每活对象 41 字节 + 全池常驻一份「一个开放墓碑容器 + 根」= 3 槽）是 n 的函数，不是余额：
  reserve(n) = 41·n 字节 + 3 槽。罩得住 ⟺ 对一切可达的 (m, n)，need(m, n)·16384 ≤ reserve(n)。
另量两个收严臂：常驻改成随墓碑树高 H(m) 变的 2 + H(m) + 1 槽（下一次长高要 H 个右半 + 1 个新根）；以及取整位置不同的两种 41。
"""
import math

SLOT = 16384

def internal_nodes(containers, fanout):
    if containers == 0:
        return 0, 0
    total, level, height = 0, containers, 0
    while True:
        level = math.ceil(level / fanout)
        total += level
        height += 1
        if level == 1:
            return total, height

def perm_slots(m, cap, fanout):
    containers = math.ceil(m / cap)
    nodes, _ = internal_nodes(containers, fanout)
    return 2 * containers + nodes

def height(m, cap, fanout):
    return internal_nodes(math.ceil(m / cap), fanout)[1]

def reserve_g1(n, m, cap, fanout):
    return 41 * n + 3 * SLOT

def reserve_by_height(n, m, cap, fanout):
    return 41 * n + (2 + height(m, cap, fanout) + 1) * SLOT

def critical_m(cap, fanout, levels):
    ms = set(range(0, 4 * cap + 3))
    for k in range(1, levels + 1):
        for containers in (fanout ** k - 1, fanout ** k, fanout ** k + 1):
            for extra in (-cap, -1, 0, 1, cap):
                m = containers * cap + extra
                if m >= 0:
                    ms.add(m)
    return sorted(ms)

def worst(reserve, cap, fanout, levels, n_max):
    best = None
    for m in critical_m(cap, fanout, levels):
        for n in range(1, n_max + 1):
            margin = reserve(n, m, cap, fanout) - perm_slots(m + n, cap, fanout) * SLOT + perm_slots(m, cap, fanout) * SLOT
            if best is None or margin < best[0]:
                best = (margin, m, n)
    return best

def main():
    product = (815, 135)
    for name, reserve in (("G1_41B_plus_3slots", reserve_g1), ("41B_plus_2_plus_height_plus_1", reserve_by_height)):
        for levels in (1, 2, 3):
            margin, m, n = worst(reserve, *product, levels, 1700)
            print(f"E name=tomb_reserve rule={name} cap=815 fanout=135 growth_levels_scanned={levels} n_max=1700 "
                  f"min_margin_bytes={margin} at_m={m} at_n={n} need_slots={perm_slots(m + n, *product) - perm_slots(m, *product)} "
                  f"height_before={height(m, *product)}")
    for name, reserve in (("G1_41B_plus_3slots", reserve_g1), ("41B_plus_2_plus_height_plus_1", reserve_by_height)):
        worst_large = None
        for m in critical_m(*product, 3):
            for n in (10 ** 4, 10 ** 5, 10 ** 6, 10 ** 7, 3 * 10 ** 7):
                margin = reserve(n, m, *product) - (perm_slots(m + n, *product) - perm_slots(m, *product)) * SLOT
                if worst_large is None or margin < worst_large[0]:
                    worst_large = (margin, m, n)
        print(f"E name=tomb_reserve_large_n rule={name} n_samples=1e4,1e5,1e6,1e7,3e7 min_margin_bytes={worst_large[0]} "
              f"at_m={worst_large[1]} at_n={worst_large[2]}")
    m = 135 * 815
    print(f"E name=tomb_root_full m={m} containers={math.ceil(m / 815)} perm_slots={perm_slots(m, 815, 135)} "
          f"next_one_need_slots={perm_slots(m + 1, 815, 135) - perm_slots(m, 815, 135)} reserve_g1_bytes_n1={reserve_g1(1, m, 815, 135)}")
    for cap, fanout in ((1, 2), (2, 3), (4, 4)):
        margin, m, n = worst(reserve_g1, cap, fanout, 3, 64)
        print(f"E name=tomb_reserve_knob rule=G1_41B_plus_3slots cap={cap} fanout={fanout} min_margin_bytes={margin} at_m={m} at_n={n}")
    exact = 2 * (1 + 1 / 135) * SLOT / 815
    first_diff = next(n for n in range(1, 10 ** 6) if 41 * n != math.ceil(n * 2 * 136 * SLOT / (135 * 815)))
    n6 = 10 ** 6
    print(f"E name=tomb_rounding per_object_exact={exact:.4f} ceil_per_object=41 first_n_where_41n_ne_ceil_total={first_diff} "
          f"at_1e6_41n={41 * n6} ceil_total={math.ceil(n6 * 2 * 136 * SLOT / (135 * 815))}")
    for replicas in (2,):
        n = n6
        label_sum = replicas * (41 * n + 3 * SLOT)
        label_one_copy = 41 * n + 3 * SLOT
        print(f"E name=tomb_units replicas={replicas} n={n} label6_sum_bytes={label_sum} per_device_deduct={math.ceil(label_sum / replicas)} "
              f"if_label_written_one_copy_per_device_deduct={math.ceil(label_one_copy / replicas)} "
              f"shortfall_per_device={math.ceil(label_sum / replicas) - math.ceil(label_one_copy / replicas)}")

if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""C5 补一刀（接 b3_c5_admission_overlap.py，不改它）：
b3 里 PO 变体（已发布统计 + 在飞预留，发布那一刻同步清零）在任何交错下都准，但代码今天读的不是「已发布统计」而是分配器此刻的计数
（crates/singlefs-core/src/admission.rs:318 of_allocator 的 allocated = device_map.allocated_slots()：落点一取就加）。
所以代码读法下合成值该是：分配器计数（已含在飞已取落点的 d）+ 已发布的已承诺预留 t + overlay。overlay 里该放什么、什么时候清？
本文件只加两个变体并且复跑 b3 的交错枚举（同一个 TXN、同一组容量、同一个 R）：
  LS  分配器计数 + 已发布的 t + overlay 分两段清：d 那份在取落点时清（分配器计数接手）、t 那份在发布时清（已发布统计接手）
  L0  LS 的 overlay 强制清零 = C87 欠的「可强制清零的测试开关」（等于 b3 的 L）
超卖 / 假拒的定义同 b3。
"""
from math import factorial

from b3_c5_admission_overlap import TXN, schedules


def run(schedule, variant, capacity, ring, txn):
    d, t = txn
    admitted, allocated, published = [], set(), set()
    overlay_d, overlay_t = set(), set()
    false_reject = False
    for i, event in schedule:
        if event == "A":
            counted = len(allocated) * d + len(published) * t
            if variant == "LS":
                counted += len(overlay_d) * d + len(overlay_t) * t
            if d + t <= capacity - ring - counted:
                admitted.append(i)
                overlay_d.add(i)
                overlay_t.add(i)
            elif len(admitted) * (d + t) + d + t + ring <= capacity:
                false_reject = True
        elif event == "L" and i in admitted:
            allocated.add(i)
            overlay_d.discard(i)
        elif event == "P" and i in admitted:
            published.add(i)
            overlay_t.discard(i)
    return len(admitted) * (d + t) + ring > capacity, false_reject


def main():
    ring, k = 2, 3
    print(f"d+t={TXN[0]}+{TXN[1]}，R={ring}，k={k}；每格「超卖的交错条数 / 假拒的交错条数」（LS 分两段清；L0 = LS 的 overlay 强制清零）")
    for atomic in (False, True):
        plans = list(schedules(k, atomic))
        label = "A+L 原子（整体独占）" if atomic else "A、L 任意交错"
        print(f"\n{label}：交错 {len(plans)} 条（全部交错数 {factorial(3 * k) // 6 ** k}）")
        print("  读数  " + "  ".join(f"C={c}:超卖/假拒" for c in (11, 12, 13, 14)))
        for variant in ("LS", "L0"):
            cells = []
            for capacity in (11, 12, 13, 14):
                over = fake = 0
                for plan in plans:
                    o, f = run(plan, "LS" if variant == "LS" else "L0", capacity, ring, TXN)
                    over += o
                    fake += f
                cells.append(f"{over:6d}/{fake:<6d}")
            print(f"  {variant:5s} " + "  ".join(cells))


if __name__ == "__main__":
    main()

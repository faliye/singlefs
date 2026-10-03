#!/usr/bin/env python3
"""C5（Q5，C87）：准入读数的几种合成写法，在 k 个事务的全部事件交错上数「超卖」与「假拒」，看环窗口 / 墓碑预付 / 在飞预留会不会重扣。

模型（纯算术，槽为单位，不与 crates/ 共用代码）：
  池容量 C 槽；环窗口 R 槽（D28 已定项 1 的常驻扣项：常量，不随在飞数变）；事务 i 的需求 d_i（这次新写的槽）与墓碑预付 t_i
  （创建 inode 的事务发布后把「已承诺预留」永久加 t_i，D3 已定项 2 / 标签 6）。每个事务三件事按序发生：
  准入 A_i（读数、判放不放）→ 取落点 L_i（分配器计数立刻加 d_i，admission.rs 的 of_allocator 读的就是这个计数）→ 发布 P_i（已发布统计加 d_i + t_i）。
  事件在事务之间任意交错；Q4 取整体独占时 A_i 与 L_i 在同一临界区（相邻）。
读数写法（avail = C − R − 已计 − 在飞）：
  P     已计 = 已发布的 d+t；在飞 = 0                              只读已发布统计（E82 的第一种直觉写法）
  L     已计 = 已取落点的 d + 已发布的 t；在飞 = 0                   读分配器此刻计数，墓碑只算已发布的
  L2    已计 = 已取落点的 d + 已取落点的 t；在飞 = 0                 读分配器计数，墓碑也在取落点那一刻计（链上现算）
  LO    L2 再扣在飞预留（准入时记、发布时清）                        在飞 = 已准入未发布的 d+t   ← 与分配器计数重扣
  PO    P 再扣在飞预留，在发布那一刻与已发布统计同步清零            在飞 = 已准入未发布的 d+t
  PO早  PO，但预留在取落点那一刻就清
  PO晚  PO，但预留在发布之后的下一个事件才清
超卖 = 被准入的 d+t 之和 + R > C；假拒 = 事务被拒、而「前面已准入的 + 它自己」放得下。
"""
from itertools import permutations
from math import factorial

TXN = (4, 1)    # (d, t)：创建 inode 的写，新写 4 槽、墓碑预付 1 槽


def schedules(k, atomic):
    base = [(i, e) for i in range(k) for e in "ALP"]
    for perm in sorted(set(permutations(base))):
        order = {event: pos for pos, event in enumerate(perm)}
        if all(order[(i, "A")] < order[(i, "L")] < order[(i, "P")] for i in range(k)) and \
                (not atomic or all(order[(i, "L")] == order[(i, "A")] + 1 for i in range(k))):
            yield perm


def run(schedule, variant, capacity, ring, txn):
    d, t = txn
    admitted, allocated, published, overlay = [], set(), set(), set()
    clear_next = []
    false_reject = False
    for i, event in schedule:
        clearing_now, clear_next = clear_next, []
        if event == "A":
            pub_dt = len(published) * (d + t)
            pub_t = len(published) * t
            live_d = len(allocated) * d
            live_t = len(allocated) * t
            in_flight = len(overlay) * (d + t)
            counted = {"P": pub_dt, "L": live_d + pub_t, "L2": live_d + live_t, "LO": live_d + live_t + in_flight,
                       "PO": pub_dt + in_flight, "PO早": pub_dt + in_flight, "PO晚": pub_dt + in_flight}[variant]
            if d + t <= capacity - ring - counted:
                admitted.append(i)
                overlay.add(i)
            elif len(admitted) * (d + t) + d + t + ring <= capacity:
                false_reject = True
        elif event == "L" and i in admitted:
            allocated.add(i)
            if variant == "PO早":
                overlay.discard(i)
        elif event == "P" and i in admitted:
            published.add(i)
            if variant in ("LO", "PO"):
                overlay.discard(i)
            elif variant == "PO晚":
                clear_next.append(i)
        for j in clearing_now:
            overlay.discard(j)
    return len(admitted) * (d + t) + ring > capacity, false_reject


def main():
    ring, k = 2, 3
    print(f"d+t={TXN[0]}+{TXN[1]}，R={ring}，k={k}；对每个容量 C 数「超卖的交错条数 / 假拒的交错条数」")
    print("  C 取 11..14：C−R 依次 9..12，串行能放下的事务数 = (C−R)÷5 取整 = 1、2、2、2；C=13 时三个事务的 d 之和 12 ≤ 11 不成立，d+t 之和 15 > 11")
    for atomic in (False, True):
        plans = list(schedules(k, atomic))
        label = "A+L 原子（整体独占）" if atomic else "A、L 任意交错"
        print(f"\n{label}：交错 {len(plans)} 条（全部交错数 {factorial(3 * k) // 6 ** k}）")
        print("  读数  " + "  ".join(f"C={c}:超卖/假拒" for c in (11, 12, 13, 14)))
        for variant in ("P", "L", "L2", "LO", "PO", "PO早", "PO晚"):
            cells = []
            for capacity in (11, 12, 13, 14):
                over = fake = 0
                for plan in plans:
                    o, f = run(plan, variant, capacity, ring, TXN)
                    over += o
                    fake += f
                cells.append(f"{over:6d}/{fake:<6d}")
            print(f"  {variant:5s} " + "  ".join(cells))


if __name__ == "__main__":
    main()

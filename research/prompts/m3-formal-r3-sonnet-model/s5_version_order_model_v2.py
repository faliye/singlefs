#!/usr/bin/env python3
"""S5 版本序的分辨观测：三种写法在三个选根器上对「s ⊆ u 时 land(s) 不晚于 land(u)」各判什么。

只译了一件事：选根器。真选根器是 crates/singlefs-core/src/recovery.rs 的 choose_root：
`let candidate_key = (candidate.checkpoint_txg, candidate.instance);` 取最大（txg 为主、实例代号破平局）。
不含施加记录那一段（那一段只在所选根自己的实例里往上走，不跨实例）。[译]：不是 Rust 上量的。
"""
import itertools, sys

def land_real(roots):            # (txg, instance) 取大
    return max(roots, key=lambda r: (r[1], r[0]))
def land_tie_low(roots):         # 变异 A：平局时实例代号低者赢
    return max(roots, key=lambda r: (r[1], -r[0]))
def land_instance_major(roots):  # 变异 B：实例代号为主
    return max(roots, key=lambda r: (r[0], r[1]))

SELECTORS = {"real(txg,inst)": land_real, "mutA_tie_low": land_tie_low, "mutB_instance_major": land_instance_major}

def later_txg_only(a, b):   return a[1] > b[1]
def later_inst_txg(a, b):   return (a[0], a[1]) > (b[0], b[1])
def later_txg_inst(a, b):   return (a[1], a[0]) > (b[1], b[0])
ORDERS = {"txg_only": later_txg_only, "(inst,txg)": later_inst_txg, "(txg,inst)": later_txg_inst}

# 根 = (实例代号, checkpoint_txg)。三个世界：
WORLDS = {
    # 注 3 世界：新实例第一次发布的 txg ≥ 环里一切 txg + 1；没有失而复得，没有倒挂
    "note3": [(1,100),(1,101),(1,102),(2,103),(2,104)],
    # 只有平局：丢盘回退 + 盘失而复得，新时间线复用了旧计数（D16 已定项 6 骑手 1、C88）；新实例的 txg 都不低于旧实例的
    "tie_only": [(1,100),(1,101),(2,101),(2,102)],
    # 倒挂：较旧实例的根 txg 更高，新实例从更旧的根往上数（C331）
    "inversion": [(1,100),(1,101),(1,102),(2,101),(2,102)],
}

def subsets(universe):
    for mask in range(1, 1 << len(universe)):
        yield frozenset(r for i, r in enumerate(universe) if mask >> i & 1)

def main():
    first_witness = {}
    print("world\tselector\torder\tpairs\tviolations")
    for world, universe in WORLDS.items():
        subs = list(subsets(universe))
        pairs = [(s, u) for u in subs for s in subs if s <= u]
        for sel_name, sel in SELECTORS.items():
            for order_name, later in ORDERS.items():
                bad = 0
                for s, u in pairs:
                    if later(sel(s), sel(u)):
                        bad += 1
                        first_witness.setdefault((world, sel_name, order_name), (sorted(s), sorted(u), sel(s), sel(u)))
                print(f"{world}\t{sel_name}\t{order_name}\t{len(pairs)}\t{bad}")
    print()
    print("见证（每格第一个；s、u 是读得出的根集合，land 是选根器的结果）：")
    for key, (s, u, ls, lu) in first_witness.items():
        print(f"{key[0]}\t{key[1]}\t{key[2]}\ts={s}\tu={u}\tland(s)={ls}\tland(u)={lu}")
    # 三种写法两两判定不同的对数（真选根器）：
    print()
    print("真选根器上两种写法判定不同的对数：")
    for world, universe in WORLDS.items():
        subs = list(subsets(universe))
        pairs = [(s, u) for u in subs for s in subs if s <= u]
        names = list(ORDERS)
        for a, b in itertools.combinations(names, 2):
            diff = sum(1 for s, u in pairs
                       if ORDERS[a](land_real(s), land_real(u)) != ORDERS[b](land_real(s), land_real(u)))
            print(f"{world}\t{a} vs {b}\tdiffer_pairs={diff}")

main()

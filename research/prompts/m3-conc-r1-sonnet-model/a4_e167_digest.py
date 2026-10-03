#!/usr/bin/env python3
"""E167 三段：K1（按根槽写切）与 K2（按事务切）在 P1 串行、P2 一次发布装多个事务、P3 多次发布同时在飞三个 Q1 候选上的
跨事务漏数与状态数，再算能力那一半的算术（段内序号 u32 / u64 装不装得下）。只读产物，能力数自己重算一遍对产物里的 anchor。"""
from a_common import E167_ONE, E167_TWO, E167_THREE, rows

U32 = 2**32 - 1
U64 = 2**64 - 1


def one(name, label):
    print(f"\n== {label}（{name}）==")
    truth = {f["cell"]: f for _, f in rows(name, "name=cell_truth")}
    k1 = {f["cell"]: f for _, f in rows(name, "name=arm_by_root_publish")}
    k2 = {f["cell"]: f for _, f in rows(name, "name=arm_by_transaction")}
    for cell in truth:
        t, a, b = truth[cell], k1[cell], k2.get(cell, {})
        print(f"  {cell:16s} 候选={t['candidate']:34s} 盘={t['disks']} 单元={t['units']} 真值={t['truth_by_enumeration']:>8s} "
              f"K1 漏数(最小..最大)={a['cross_misses_minimum']}..{a['cross_misses_maximum']} 漏的流={a['streams_with_cross_misses']}/{a['streams']} "
              f"K2 漏(对 T)={b.get('cross_misses', '-')} K2 真漏(对 T∩T_exact)={b.get('true_cross_misses', '-')}")


def main():
    one(E167_ONE, "第一段：一盘")
    one(E167_TWO, "第二段：两盘")
    print("\n第三段 L11（K1 对 T ∩ T_exact 的跨事务漏，两盘）：")
    for _, f in rows(E167_THREE, "name=arm_by_root_publish"):
        keys = [k for k in f if "true_cross" in k]
        print("  ", f.get("cell"), {k: f[k] for k in keys})
    print("\n能力算术（登记第三节：一段内序号 u32；一次提交最多 2048 个写；总状态数 u64）：")
    for _, f in rows(E167_ONE, "name=anchor"):
        if f.get("table") == "capacity" and "real_shape" in f.get("quantity", ""):
            value = int(f["actual"])
            print(f"  {f['quantity']} = {value}  > u32最大 {U32}: {value > U32}  <= u64最大: {value <= U64}  写数字段={f.get('cover_largest_segment_writes', '-')}")
    print(f"  独立重算：2^48 - 1 = {2**48 - 1}；2^54 * 3^4 - 1 = {2**54 * 3**4 - 1}（E167 页 L8 写的 1459166279268040703）")
    print(f"  2^54 * 3^4 - 1 与产物 real_shape_arm_by_transaction_cover_one_segment_states 1459166279268040704 差 "
          f"{1459166279268040704 - (2**54 * 3**4 - 1)}（产物那个数 = 2^54 * 3^4）")


if __name__ == "__main__":
    main()

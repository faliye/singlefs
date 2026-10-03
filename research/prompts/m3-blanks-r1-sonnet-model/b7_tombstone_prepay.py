#!/usr/bin/env python3
"""B7：墓碑预付摊分式子。单位字节、每块盘一份副本；记账行是全池一行，按全部副本之和记（C375 已还：admission.rs share_of_one_device）。
候选式子：预付(n) = 常驻 3 槽 + p × n，p 是每个活对象的字节数。验：对任意已写墓碑数 m 与将来要死的 n 个活对象，
 预付(n) ≥ 墓碑树这 n 条记录会新占的槽（含内部节点，含根长高）。结构：容器 2 槽（32 KiB）每容器 815 条；
 内部节点按 D8 已定项 6：每层 ⌈下层 ÷ 135⌉ 个，直到一个根；容器数为 0 时树是零根（0 个节点）。
"""
SLOT, REC_PER, F = 16384, 815, 135
def internal(c):
    if c == 0: return 0
    n, tot = c, 0
    while True:
        n = -(-n // F); tot += n
        if n == 1: return tot
def S(c): return 2 * c + internal(c)                # 槽
C_MAX = 30000
SA = [S(c) for c in range(C_MAX + 1300)]
from functools import lru_cache
@lru_cache(maxsize=None)
def need_for_new_containers(dc): return max(SA[c0 + dc] - SA[c0] for c0 in range(0, C_MAX))
def need_slots(n):                                  # 最坏：m 取容器刚满的那一格（开放容器已满），再取 c0 的最坏位置
    return need_for_new_containers(-(-n // REC_PER))
cands = {"α 每对象 ⌈32768÷815⌉=41 B": -(-2 * SLOT // REC_PER), "β 每对象 40 B（记录宽，不摊容器）": 40,
         "γ 每对象 1 整容器（2 槽）": 2 * SLOT, "δ 每对象 0 B（只留常驻）": 0}
print(f"E name=constants slot={SLOT} container_slots=2 records_per_container={REC_PER} fanout={F} alpha_bytes={cands['α 每对象 ⌈32768÷815⌉=41 B']}")
ns = list(range(1, 3000)) + list(range(3000, 130000, 997)) + [31455, 110025, 110026, 1_000_000]
need = {n: need_slots(n) if n < 130000 else None for n in ns}
need[1_000_000] = need_for_new_containers(1227)
for name, p in cands.items():
    worst = None; fails = 0
    for n in ns:
        margin = (3 * SLOT + p * n) - need[n] * SLOT          # 字节
        if margin < 0: fails += 1
        if worst is None or margin < worst[0]: worst = (margin, n)
    print(f"E name=cover cand={name} n_checked={len(ns)} n_failing={fails} min_margin_bytes={worst[0]} at_n={worst[1]}")
# 常驻项取几槽：α 的 n=1 就不过——容器刚好 135 个、根装满时，下一条墓碑要新开一个容器（2 槽）并让根长高（净增 2 个内部节点）
@lru_cache(maxsize=None)
def worst_start(dc): return max((SA[c0 + dc] - SA[c0], c0) for c0 in range(0, C_MAX))
for dc in (1, 2, 3):
    v, c0 = worst_start(dc)
    print(f"E name=worst_start new_containers={dc} worst_new_slots={v} at_existing_containers={c0} internal_before={internal(c0)} internal_after={internal(c0 + dc)}")
for const_slots in (3, 4, 5, 6):
    fails = [n for n in ns if (const_slots * SLOT + 41 * n) < need[n] * SLOT]
    print(f"E name=const_slots const_slots={const_slots} per_object_bytes=41 n_checked={len(ns)} n_failing={len(fails)} first_failing_n={fails[0] if fails else None} "
          f"last_failing_n={fails[-1] if fails else None}")
# 内部层数 h（根之下装容器的那一层算第 1 层）时，下一次长高的净增 = 2 + (h + 1) 槽；常驻 = 2 + (h + 1)：h = 1、2、3 取 4、5、6
for h in (1, 2, 3, 4):
    c_full = F ** h; print(f"E name=jump h={h} full_at_containers={c_full} full_at_unlinks={c_full * REC_PER} next_unlink_net_slots={S(c_full + 1) - S(c_full)} formula_2_plus_h_plus_1={2 + h + 1}")
# 取整落在哪一步：41 是每条记录向上取整一次，再乘 n_live 与副本数，之后不再取整；对比聚合取整
for n in (1, 815, 816, 31455, 110025, 1_000_000):
    per_rec = 41 * n; agg = -(-n * 2 * SLOT // REC_PER)
    print(f"E name=rounding n={n} per_record_ceil_bytes={per_rec} aggregate_ceil_bytes={agg} diff={per_rec - agg}")
# 取值序列：两块盘（副本数 2），3 单元文件 + K = 1 删除，对照辩方第三轮表 A（窗口项已按 K-R3-2 拿出标签 6）
rep = 2
CONST_SLOTS = 3
def tag6(n_live, started, const_slots=None):
    c = CONST_SLOTS if const_slots is None else const_slots
    return 0 if not started else rep * (c * SLOT + 41 * n_live)
rows = [("第一个文件版本之前（树表 0 条）", 0, False), ("第一个文件版本（n_live=1）", 1, True), ("顺序写成 3 单元之后", 1, True),
        ("意图之后（inode 记录 nlink=0，n_live=0）", 0, True), ("批 1", 0, True), ("批 2", 0, True), ("最后一批", 0, True),
        ("回退到删之前那一版（n_live 取 R_old 的 inode 树 =1）", 1, True)]
for const_slots in (3, 6):
  for label, n_live, started in rows:
    t = tag6(n_live, started, const_slots)
    print(f"E name=tag6 const_slots={const_slots} step={label} n_live={n_live} tag6_bytes={t} share_per_device={-(-t // rep)} divisible_by_replicas={t % rep == 0}")
print(f"E name=cross_check 辩方第三轮 3.1 的标签 6 含窗口：2×(9203×16384+41)=301563986；去掉窗口 9200 槽：{2*(9203*SLOT+41) - 2*9200*SLOT} 与本式 tag6(1)={tag6(1, True)} 相等={2*(9203*SLOT+41) - 2*9200*SLOT == tag6(1, True)}")
# 字节：新池新建文件那一版记账树叶 t10 里已承诺预留行（第 10 条，下标 9）的 value
hdr, ent, idx, val_off = 159, 34, 9, 22
for const_slots in (3, 6):
    print(f"E name=changed_bytes const_slots={const_slots} unit=t10 记账树根兼叶 entry_index={idx} value_offset_in_unit={hdr + idx * ent + val_off} width=8 old=0 new={tag6(1, True, const_slots)} hex_le={tag6(1, True, const_slots).to_bytes(8, 'little').hex()}")
# 每对象整容器（γ）与 41 字节在 4 GiB 池上的总价：单元区 211968 槽，最多 31455 个活 inode（一个 inode 根）
for n in (1, 815, 31455):
    print(f"E name=cost_on_4GiB n_live={n} alpha_bytes_per_device={41 * n} alpha_percent_of_unit_area={41 * n / (211968 * SLOT) * 100:.4f} gamma_bytes_per_device={2 * SLOT * n} gamma_percent={2 * SLOT * n / (211968 * SLOT) * 100:.2f}")
rows15 = [(1,0,0),(1,0,1),(2,0,0),(2,0,1),(3,0,0),(3,0,1),(4,0,0xFFFFFFFF),(5,0,0),(5,0,1),(6,0,0xFFFFFFFF),(10,0,0),(10,0,1),(11,0,0),(11,0,1),(12,12,0xFFFFFFFF)]
print(f"E name=row_order sorted_ok={rows15 == sorted(rows15)} index_of_stat6={[r[0] for r in rows15].index(6)} rows={len(rows15)}")

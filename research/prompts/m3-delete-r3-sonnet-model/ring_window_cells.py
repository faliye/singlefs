#!/usr/bin/env python3
"""m3-delete-r3 辩方腿：环窗口式子的纯算术与对拍（零依赖）。

输入：同目录 `run-final-r-lines.txt`（Rust 用例 `m3_delete_r3_defense_cells` 的 `R name=` 行，原样）。
输出：stdout（报告里抄的 `C name=` 行）。每个断言钉绝对值；断言失败就是这份模型或那份日志变了。
被攻过零轮：式子是我提的（环窗口落成式子），没在入库装置上量过。
"""
import ast
import math
import os
import random
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SLOT = 16384
LEAF = 812          # singlefs_format::ALLOCATION_RECORD_TREE_LEAF_SLOTS
FANOUT = 169        # singlefs_format::ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
UNIT_AREA_START = 50176  # 4 GiB 盘单元区 211968 槽 = 262144 − 50176（journal 环 768 MiB 默认 + 固定结构）
EXTRA_NODES_IN_CKPT_COST = 8  # 537 − 529：中央映射树 + 记账树 + 树表（4 GiB 上量到的差；别的盘大小上是推的）
TOMB_BYTES_PER_OBJECT = 41     # ⌈(2 槽 × (1 + 1/135)) × 16384 ÷ 815⌉
TOMB_SLACK_SLOTS = 3
O_UB, ROOTS_UB, MAP_UB = 16, 4, 4


def emit(**fields):
    print("C " + " ".join(f"{k}={v}" for k, v in fields.items()))


def level_counts(capacity_bytes, disks=2):
    slots = capacity_bytes // SLOT
    last = slots - 1

    def per_level(level):
        span = LEAF * FANOUT ** level
        return disks * (last // span - UNIT_AREA_START // span + 1)

    counts = [per_level(0)]
    while counts[-1] > FANOUT:
        counts.append(per_level(len(counts)))
    return counts


def w_ub(batch, counts):
    return O_UB + ROOTS_UB + MAP_UB + sum(min(c, 2 * (batch + 26)) for c in counts)


def ckpt_cost(counts):
    return sum(counts) + 1 + EXTRA_NODES_IN_CKPT_COST


def window(ring_roots, batch, counts):
    return (ring_roots + 1) * w_ub(batch, counts)


# ---------------------------------------------------------------------------------------------------------------------
# A. 几何表：窗口、保留量、「保留量罩得住的那部分」
# ---------------------------------------------------------------------------------------------------------------------
c4 = level_counts(4 << 30)
assert c4 == [524, 4], c4
assert ckpt_cost(c4) == 537
assert 13 * ckpt_cost(c4) + 8 == 6989
assert w_ub(144, c4) == 368 and window(24, 144, c4) == 9200 and window(48, 144, c4) == 18032
for gib in (3, 4, 8, 16, 64, 256, 1024):
    counts = level_counts(gib << 30)
    c = ckpt_cost(counts)
    reserve = 13 * c + 8
    unit_area = (gib << 30) // SLOT - UNIT_AREA_START
    for ring in (12, 24, 48):
        w = w_ub(144, counts)
        win = window(ring, 144, counts)
        # 保留量能借给删除的：13c + 8 − (c + O)（落点那一判要 c + 普通槽数空闲）；窗口里需要保留量罩不住的那一截
        need_beyond_reserve = max(0, ring * w + O_UB - (12 * c + 8))
        emit(name="geometry", gib=gib, ring_roots=ring, levels=counts, ckpt_cost=c, reserve_slots=reserve, w_ub_k144=w,
             window_slots=win, window_pct_of_unit_area=f"{100 * win / unit_area:.2f}", window_over_reserve=f"{win / reserve:.2f}",
             deficit_bound_ring_times_w=ring * w, beyond_reserve_slots=need_beyond_reserve)
for k in (1, 16, 48, 144):
    emit(name="window_by_batch", gib=4, ring_roots=24, k=k, w_ub=w_ub(k, c4), window_slots=window(24, k, c4),
         pct=f"{100 * window(24, k, c4) / 211968:.2f}")

# ---------------------------------------------------------------------------------------------------------------------
# B. 3 单元文件、K = 1 的逐格值（4 GiB × 2 盘、2 副本；每盘槽 16384 字节；标签按全部副本之和）
# ---------------------------------------------------------------------------------------------------------------------
log = [line for line in open(os.path.join(HERE, "run-final-r-lines.txt")).read().splitlines() if line.startswith("R name=")]


def field(line, key):
    match = re.search(rf"{key}=(\[[^\]]*\]|\S+)", line)
    return match.group(1) if match else None


def label6(live, win):
    return 2 * ((win + TOMB_SLACK_SLOTS) * SLOT + live * TOMB_BYTES_PER_OBJECT)


win24 = window(24, 144, c4)
assert label6(1, win24) == 301563986 and label6(0, win24) == 301563904
geometry_line = next(line for line in log if line.startswith("R name=geometry"))
assert int(field(geometry_line, "window_slots_per_disk")) == win24
assert int(field(geometry_line, "label6_one_live_object_all_replicas")) == label6(1, win24)

nominal = [line for line in log if line.startswith("R name=nominal_delete")]
cells = [line for line in log if line.startswith("R name=cell ")]
full_rows = ast.literal_eval(re.search(r"rows=(\[.*\])$", next(line for line in log if "variant=formula_skipped_landing_judged" in line)).group(1))
assert len(nominal) == 4 and len(cells) == 2 and len(full_rows) == 4
# S2 与原型的差（槽 / 盘）：意图那次多 deleted_inodes 根 1 + 墓碑容器 2 + 墓碑树根 1 = +4；最后一批不写 inode 叶容器（2 槽），E-b 下 deleted_inodes 不写根
s2_delta = {"intent": +4, "batch_1": 0, "batch_2": 0, "batch_3": -2}
cumulative = 0
emit(name="g1_cell", step="mkfs", label4=0, label6_all_replicas=0, note="树表 0 条的一版：窗口不计（没有可删的对象）")
for line in cells:
    emit(name="g1_cell", step=field(line, "step"), label4=0, label6_all_replicas=label6(1, win24),
         available_per_disk=ast.literal_eval(field(line, "available_with_window"))[0],
         available_zero_ledger=ast.literal_eval(field(line, "available_zero_ledger"))[0])
for line in nominal:
    step = field(line, "step")
    cumulative += s2_delta[step]
    raw = ast.literal_eval(field(line, "available"))[0]
    emit(name="g1_cell", step=step, label4=0, label6_all_replicas=label6(0, win24),
         new_slots_prototype=int(field(line, "new_slots")), new_slots_s2=int(field(line, "new_slots")) + s2_delta[step],
         available_per_disk_s2=raw - cumulative * SLOT)
cumulative = 0
for row in full_rows:
    step, new, available, _ = row.split(":")
    cumulative += s2_delta[step]
    raw = int(available.split("[")[1].split(",")[0])
    emit(name="g1_cell_full_pool", step=step, label4=0, label6_all_replicas=label6(0, win24),
         available_per_disk_s2=raw - cumulative * SLOT)
# R3 逐文件（辩方腿第二轮 cells.out 的策略 X）作对照：每盘可用恒 3356966912，标签 6 / 4 逐发布转
r3 = [("after_sequential_write", 1638400, 0), ("intent", 0, 1179648), ("batch_1", 0, 753664), ("batch_2", 0, 360448), ("batch_3", 0, 0)]
for step, l6, l4 in r3:
    emit(name="r3_cell_for_comparison", step=step, label4=l4, label6_all_replicas=l6, available_per_disk=3356966912)

# ---------------------------------------------------------------------------------------------------------------------
# C. R3 逐文件 与 环窗口：每盘槽数并排（R3 的数取辩方腿第二轮 cells.out 与这一轮量到的 W；窗口取 4 GiB、K = 144）
# ---------------------------------------------------------------------------------------------------------------------
prepay_per_file = {
    "3 单元 K=1": 14 + 13 + 12 + 11,                  # 辩方第二轮 cells.out：50
    "300 单元 K=144 连续": 15 + 18 + 15 + 13,         # 61
    "144 单元 K=144 一单元一叶": (16 + 4) + (307 - 2),  # 这一轮量到：意图 16 + S2 另加 4，批 307 − 最后一批不写 inode 叶 2
    "size 0 的空 inode": 22,                           # opus 第二轮代用量
}
data_slots = {"3 单元 K=1": 6, "300 单元 K=144 连续": 600, "144 单元 K=144 一单元一叶": 288, "size 0 的空 inode": 0}
for kind, slots in prepay_per_file.items():
    emit(name="r3_vs_window", kind=kind, r3_slots_per_file=slots, data_slots=data_slots[kind],
         breakeven_files=math.ceil(win24 / slots), window_slots=win24, window_per_object_bytes=TOMB_BYTES_PER_OBJECT)
emit(name="r3_vs_window_empty_inodes", r3_inodes_whole_unit_area=211968 // 22, window_plus_41B_inodes_per_unit_area=(211968 - win24) * SLOT // TOMB_BYTES_PER_OBJECT)

# 量到的每批新占槽数（散落造法）对上界：每行 max_batch_new_slots ≤ w_upper_bound，且 Python 算的上界等于 Rust 打印的
for line in log:
    if line.startswith("R name=w_scatter"):
        gib = int(field(line, "image_gib"))
        counts = level_counts(gib << 30)
        batch = int(field(line, "batch"))
        printed = int(field(line, "w_upper_bound"))
        measured = int(field(line, "max_batch_new_slots"))
        assert w_ub(batch, counts) == printed, (gib, batch, w_ub(batch, counts), printed)
        assert measured <= printed
        emit(name="w_vs_bound", image_gib=gib, units=field(line, "units"), batch=batch, max_batch_new_slots=measured,
             w_upper_bound=printed, slack=printed - measured, checkpoint_cost_before=field(line, "checkpoint_cost_before"),
             model_ckpt_cost=ckpt_cost(counts))
for line in log:
    if line.startswith("R name=ring_apex"):
        emit(name="ring_apex_check", ring_roots=field(line, "ring_roots"), units=field(line, "units"), with_window=field(line, "with_window"),
             apex_deficit_slots=field(line, "apex_deficit_slots"), window_slots=field(line, "window_slots"),
             min_free_minus_reserve=field(line, "min_free_minus_reserve"))
        assert int(field(line, "apex_deficit_slots")) <= int(field(line, "window_slots"))
        if field(line, "with_window") == "true":
            assert int(field(line, "min_free_minus_reserve")) > 0   # 窗口在账里时，删除全程没动到保留量
        else:
            assert int(field(line, "min_free_minus_reserve")) < 0   # 窗口不在账里时，删除借了保留量
redo = [line for line in log if line.startswith("R name=redo_from_the_same_image")]
assert len(redo) == 6 and all("images_byte_equal=true" in line and "outputs_equal=true" in line for line in redo)
emit(name="redo_samples", samples=len(redo), all_byte_equal=1)

# ---------------------------------------------------------------------------------------------------------------------
# D. 环引理：每盘亏空 ≤ L × W_max（释放 ≥ 新写时）。按 allocator.rs:1655 / mount.rs:1158 的次序模拟
#    发布 p：释放先于分配；分配之后记根；回收释放代 ≤ p − L + 1 的槽。亏空 = 基线空闲 − 此刻空闲；落点那一判要在分配前 空闲 ≥ W_p。
# ---------------------------------------------------------------------------------------------------------------------
def simulate(ring, writes, releases, pre_existing=()):
    pending = {}  # 释放代 → 槽数
    for offset, slots in enumerate(pre_existing):
        pending[-offset] = pending.get(-offset, 0) + slots
    deficit = 0
    worst_need = 0
    for p, (w, rel) in enumerate(zip(writes, releases), start=1):
        pending[p] = pending.get(p, 0) + rel     # 释放先于分配
        worst_need = max(worst_need, deficit + w)  # 分配那一刻要的空闲 = 已亏 + 这一次
        deficit += w
        floor = p - ring + 1
        for g in [g for g in pending if g <= floor]:
            deficit -= pending.pop(g)
    return worst_need


random.seed(20261001)
worst_ratio = 0.0
for trial in range(2000):
    ring = random.choice((12, 24, 48))
    n = random.randint(1, 300)
    wmax = random.randint(5, 400)
    writes = [random.randint(1, wmax) for _ in range(n)]
    releases = [w + random.choice((0, 0, 2 * random.randint(0, 144))) for w in writes]   # 释放 ≥ 新写（COW 换下的 + 被删的数据单元）
    need = simulate(ring, writes, releases)
    assert need <= ring * max(writes), (ring, need, max(writes))
    worst_ratio = max(worst_ratio, need / (ring * max(writes)))
emit(name="ring_lemma", trials=2000, claim="worst_need <= L * W_max (releases >= writes)", worst_need_over_bound=f"{worst_ratio:.3f}")
# 反例：释放 < 新写（意图那次的墓碑 4 槽是真增长）时亏空不回到 0：每次 +4 槽的永久增长
need = simulate(24, [16] * 100, [12] * 100)
emit(name="ring_lemma_permanent_growth", writes_per_publish=16, releases_per_publish=12, publishes=100, worst_need=need, grows_linearly=int(need > 24 * 16))
# 与第二轮攻方量到的顶点对拍：L=24, W=15（K=1、100 单元）与 L=48, W=19：上界 L × W
for ring, w, apex in ((24, 15, 258), (24, 19, 304), (48, 19, 712)):
    emit(name="apex_vs_bound", ring_roots=ring, batch_w=w, measured_apex=apex, ring_times_w=ring * w, within=int(apex <= ring * w))

# ---------------------------------------------------------------------------------------------------------------------
# E. G8：两轮判决里报出的数，在 S2 上重算
# ---------------------------------------------------------------------------------------------------------------------
cap40 = (32768 - 136) // 40
cap32 = (32768 - 136) // 32
emit(name="tombstone_capacity", record_bytes=40, per_container=cap40, record_bytes_32=cap32, old_header_91_32=(32768 - 91) // 32)
containers = math.ceil(10 ** 6 / cap40)
level_one = math.ceil(containers / 135)
emit(name="tombstone_growth_1e6", containers=containers, container_bytes_two_copies=containers * 32768 * 2, mib=f"{containers * 32768 * 2 / 2**20:.2f}",
     tomb_tree_nodes_below_root=level_one, tree_nodes_bytes_two_copies=(level_one + 1) * 16384 * 2,
     per_unlink_two_copies_bytes=f"{2 * 32768 / cap40:.1f}")


def two_state(units):
    return 2 ** (2 * units) + 11  # 一次带单元的发布（C577 之后）：n = 2 × 单元数


stream = 1_358_954_634
intent_units, b1_units, b2_units = 13, 12, 11
for form, last_units in (("E-c", 11), ("E-b", 10)):
    parts = [two_state(intent_units), two_state(b1_units), two_state(b2_units), two_state(last_units)]
    total = sum(parts)
    emit(name="f1_states", last_batch_form=form, per_publish=parts, delete_total=total, whole_domain=stream + total)
old_c = sum(two_state(u) for u in (12, 12, 11, 11))
old_b = sum(two_state(u) for u in (12, 12, 11, 10))
assert (old_c, old_b) == (41943084, 38797356)
emit(name="f1_states_previous_round", e_c=old_c, e_b=old_b, tomb_units_assumed=1)
rollback_segments = lambda writes: 1 + sum((1 << w) - 1 for w in writes)
assert rollback_segments([16, 2, 1, 2]) == 65543
emit(name="rollback_publish_states", base=rollback_segments([16, 2, 1, 2]),
     lineage_container_only=rollback_segments([18, 2, 1, 2]), lineage_container_plus_tomb_root=rollback_segments([20, 2, 1, 2]),
     retire_and_rewrite_k1=rollback_segments([18, 2, 1, 2]), retire_and_rewrite_k3=rollback_segments([22, 2, 1, 2]))
emit(name="ring_window_apex_reported", l24_k100=258, l24_k200=304, l48_k200=712,
     window_l24=win24, window_l48=window(48, 144, c4), reserve=13 * 537 + 8)
# 保留量自己罩得住最坏亏空的最小盘（单元区按分配记录树叶数线性外推；ckpt_cost 取模型值，推的）
for ring in (12, 24, 48):
    found = None
    gib = 3.0
    while gib < 64:
        counts = level_counts(int(gib * (1 << 30)))
        if 12 * ckpt_cost(counts) + 8 >= ring * w_ub(144, counts) + O_UB:
            found = round(gib, 1)
            break
        gib += 0.1
    emit(name="reserve_covers_from", ring_roots=ring, gib=found)
print("C name=done asserts=ok")

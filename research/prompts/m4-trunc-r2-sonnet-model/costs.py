#!/usr/bin/env python3
"""三条臂在「一次截断意图」上新写出的字节与长期占用的纯算术。输入全是 kb/crates 里已有的常量，没有拍的数。"""
import math
UNIT = 32768          # 码 1 / 码 3 单元 32 KiB（D18 已定项 9）
NODE = 16384          # 码 2 节点 16 KiB
HDR_CODE3 = 136       # 码 3 记录区起点（D18 已定项 24 第 2 条）
TOMB = 40             # 墓碑记录 40 字节
per_container = (UNIT - HDR_CODE3) // TOMB
print("一个墓碑容器装", per_container, "条（D18 已定项 24 第 2 条的 815）")
for n in (10**3, 10**6):
    containers = math.ceil(n / per_container)
    one_copy = containers * UNIT
    print("乙：%d 次截断各写一条区间墓碑 → %d 个容器，一份 %.2f MiB，两份 %.2f MiB（第一版不回收墓碑）" % (n, containers, one_copy / 2**20, 2 * one_copy / 2**20))
print("甲：每条 inode 记录仍 140 字节、一叶仍 233 条；新写的 16 字节在预留 20 字节里（偏移 120 起），长期占用增量 0；")
print("    意图那次发布本来就重写这条记录（size = N，K2 先落目标），所以甲新写的单元数 = 0、新占的槽数 = 0")
# 丙：边界截断意图那次比今天多重写一个尾单元 + 罩着它的下段叶到根 + 上段这个文件那一串
slot_unit = UNIT // NODE
for units in (3, 144, 145, 300, 144 * 147 + 1):
    lower_levels = 1
    cap = 144
    while cap < units:
        cap *= 147
        lower_levels += 1
    # 罩着单元 k 的下段叶到根：每层一个节点；上段这个文件那一串：至少 1 个节点（单根时）。上段层数这里取 1（一个文件的池，上段叶即根），多文件池更多（推的）
    extra_nodes = lower_levels + 1
    print("丙：文件 %d 个单元，边界截断多重写 1 个数据单元（%d 槽）+ 下段 %d 层 + 上段至少 1 个节点 = 每盘多新占约 %d 槽（节点层数是推的，没有量）" % (units, slot_unit, lower_levels, slot_unit + extra_nodes))
print("今天截到单元中间的意图（3 单元文件）每盘新占 11 槽，W 上界 24：边界那一格在丙下与它同形，不超 24（truncation_intent_batches_and_final_batch.rs 钉 11）")
print("乙：一次意图多写 1 个开放墓碑容器（2 槽）+ 墓碑树根（1 槽）= 每盘多新占约 3 槽（推的：开放容器 32 KiB、墓碑树根 16 KiB）；再加两份副本")

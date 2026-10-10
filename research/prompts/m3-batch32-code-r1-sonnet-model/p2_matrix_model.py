#!/usr/bin/env python3
"""P2 两份副本两两扫的预期（正推腿 Sonnet，m3-batch32-code-r1）：墓碑树、谱系树类型段 0 的条目（身份引用三段清零、层级 1 的根），
孩子两份副本各取六种状态之一，按 walk.rs walk_the_entries_of_a_container_tree_node 的读法（CPU 与 GPU 走读内核同一支）算「条目身份那一条（I-9.23 / I-9.17）红不红」。
读法照读码写：
  1. 自证过的副本 = 头里类标签是登记过的码（1 / 2 / 3）且按那一类的长度读回的整单元校验和与位置条目对得上。
  2. 有一份自证过的是码 2 之外的一类 → 再问有没有一份自证成码 2：有 → 条目判红（F-W，照节点那一支往下走）；没有 → 判红、不往下走。
  3. 否则（自证过的只有码 2、或哪一份都没自证过）：孩子的类 = 第一份自证过的副本的类标签；没有就取第一份「前 16 KiB 的校验和对得上」的副本头里的类标签（照改之前的读法）；
     再没有就读不出。红 = 孩子的类读得出且不是 2。
状态：S3 自证成码 3；S2 自证成码 2；C16 头里是码 3、校验和只对得上前 16 KiB；BAD 校验和翻位；UNREAD 读不出；ZERO 槽没写过、校验和是全零 16 KiB 的 CRC（F-零槽之后读出全零，头里类标签 0）。
输出 MODEL 行；与 probe.txt 里同一格的 identity_red 逐格比（rerun 里的 compare 一起排序，人工对）。"""
STATES = ["S3", "S2", "C16", "BAD", "UNREAD", "ZERO"]
SELF = {"S3": 3, "S2": 2}            # 自证过的类标签
SIXTEEN_KIB_MATCH = {"S2": 2, "C16": 3, "ZERO": 0}   # 前 16 KiB 的校验和对得上的副本头里的类标签

def identity_red(copies):
    certified = [SELF[c] for c in copies if c in SELF]
    other = next((tag for tag in certified if tag != 2), None)
    if other is not None:
        return True, "F-W 或另一类（红）" if 2 in certified else "另一类，不往下走（红）"
    cls = certified[0] if certified else None
    if cls is None:
        cls = next((SIXTEEN_KIB_MATCH[c] for c in copies if c in SIXTEEN_KIB_MATCH), None)
    return (cls is not None and cls != 2), f"类={cls}"

for first in STATES:
    for second in STATES:
        red, why = identity_red([first, second])
        print(f"MODEL p2 loc0={first} loc1={second} identity_red={red} ({why})")
# 盘序翻面：两份交换后判定不同的格
flips = [(a, b) for a in STATES for b in STATES if identity_red([a, b])[0] != identity_red([b, a])[0]]
print(f"MODEL p2 换盘序判定不同的格：{flips}")

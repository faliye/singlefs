#!/usr/bin/env python3
"""攻方 m3-batch33s-code-r1 S2 模型：crash_judge_common.wgsl 的 zero_unit_version_over_a_slot_nothing_wrote 守卫按 u32 算术逐句抄成 Python，
与「读的几槽都在空段 [w9, w10) 里、槽号装得进 32 位」的真值（大整数）比；另抄改之前那一版（判决第 32 批 P4 回绕那一行之前的写法）作自证。
取样：空段端点、槽号、长度、偏移高位各取边界附近的值（2³² 附近、2^(32-14) 附近、0、1）。纯算术，不读仓。
用法：python3 -I s2_guard_model.py → 打印 S2_ROW 行（无耗时字段）。"""
import itertools

M = 1 << 32
SLOT_SHIFT = 14
NODE = 16384


def u32(x):
    return x % M


def guard_now(w9, w10, w11, x, y, length):
    first_slot = u32((x >> SLOT_SHIFT) | u32(y << (32 - SLOT_SHIFT)))
    slots_read = u32(length + NODE - 1) >> SLOT_SHIFT
    if ((x & (NODE - 1)) | (y >> SLOT_SHIFT)) != 0 or first_slot < w9 or first_slot > w10 or u32(w10 - first_slot) < slots_read:
        return None
    return 1 if first_slot >= w11 else 0


def guard_before(w9, w10, w11, x, y, length):
    first_slot = u32((x >> SLOT_SHIFT) | u32(y << (32 - SLOT_SHIFT)))
    if ((x & (NODE - 1)) | (y >> (32 - SLOT_SHIFT))) != 0 or first_slot < w9 or w10 < u32(first_slot + u32((length + NODE - 1) >> SLOT_SHIFT)):
        return None
    return 1 if first_slot >= w11 else 0


def truth(w9, w10, w11, x, y, length):
    offset = (y << 32) | x
    if offset % NODE != 0:
        return None
    slot = offset >> SLOT_SHIFT
    if slot >= M:
        return None
    slots = (length + NODE - 1) // NODE
    if not (w9 <= slot and slot + slots <= w10):
        return None
    return 1 if slot >= w11 else 0


def main():
    edge = [0, 1, 2, 3, (1 << 18) - 1, 1 << 18, M - 3, M - 2, M - 1]
    gaps = [(0, 0), (5, 5), (5, 6), (5, 7), (0, M - 1), (M - 3, M - 1), (M - 2, M - 1), (7, 9), (1 << 18, (1 << 18) + 2)]
    lengths = [0, 1, NODE - 1, NODE, NODE + 1, 2 * NODE]
    ys = [0, 1, (1 << 14) - 1, 1 << 14, (1 << 14) + 1, 1 << 18, M - 1]
    cases = now_wrong = before_wrong = 0
    first_now = first_before = None
    for (w9, w10), w11, slot_low, y, length in itertools.product(gaps, [0, 6, M - 2], edge, ys, lengths):
        for misalign in (0, 512):
            x = u32((slot_low << SLOT_SHIFT) | misalign)
            y_eff = u32(y | (slot_low >> (32 - SLOT_SHIFT)))
            cases += 1
            t = truth(w9, w10, w11, x, y_eff, length)
            a = guard_now(w9, w10, w11, x, y_eff, length)
            b = guard_before(w9, w10, w11, x, y_eff, length)
            if a != t and length > 0:
                now_wrong += 1
                first_now = first_now or (w9, w10, w11, x, y_eff, length, a, t)
            if b != t and length > 0:
                before_wrong += 1
                first_before = first_before or (w9, w10, w11, x, y_eff, length, b, t)
    print(f"S2_ROW name=guard_now_vs_truth cases={cases} wrong={now_wrong} first={first_now}")
    print(f"S2_ROW name=selfproof_guard_before_vs_truth cases={cases} wrong={before_wrong} first={first_before}")
    # 长度 0（今天没有调用点按 0 读，推的）：起点恰在空段终点也放过
    print(f"S2_ROW name=length_zero_at_the_gap_end guard_now={guard_now(5, 7, 0, 7 << 14, 0, 0)} truth={truth(5, 7, 0, 7 << 14, 0, 0)}")
    # 空段右端那一槽按 32 KiB 读：守卫拒（下一槽有落点），CPU 稀疏读者照读「全零 16 KiB + 下一槽的 16 KiB」
    print(f"S2_ROW name=last_gap_slot_read_at_32k guard_now={guard_now(5, 7, 0, 6 << 14, 0, 2 * NODE)} cpu_reads=zeros_then_the_next_slot")


main()

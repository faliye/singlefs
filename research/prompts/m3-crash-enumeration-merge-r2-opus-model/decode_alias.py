#!/usr/bin/env python3
"""L1-G10 模型（攻方腿，m3-crash-enumeration-merge-r2）：GPU 判器内核与 CPU 事实表把「状态序号 → 持久掩码」怎么拆，逐行照抄成纯算术。

照抄的两处（行号是 2026-10-10 主工作区，开工快照 crates-sha256.txt 里的 sha256）：
- GPU 判器内核 crates/singlefs-checker-tier/src/crash_judge_common.wgsl 第 3497–3540 行：
  ordinal_lo = block_start_lo + local（u32 回绕），段内余数 remaining = ordinal_lo - 段起点低 32 位（u32 回绕），
  逐次写取 digit = remaining % choices；digit == choices - 1 落盘，digit == 1 撕裂（choices 是 2 或 3）。
- CPU 事实表 crates/singlefs-checker-tier/src/crash_facts.rs 第 556–567 行：remaining = ordinal - 段起点（u64），同一套进制。
判表打包 crates/singlefs-checker-tier/src/crash_judge_tables.rs 第 1262–1267 行不核段长；
第 ② 段核对器 crates/singlefs-checker-tier/src/crash_verify_gpu.rs 第 304 行核「段内状态数 < u32::MAX」、不满足就 panic。
不读仓、不起进程、不上卡。跑法：python3 decode_alias.py；末尾自检钉住几个绝对值，对不上退 1。
"""
import sys

U32 = 1 << 32


def gpu_decode(ordinal, segment_start, choices):
    """照 crash_judge_common.wgsl 第 3497–3540 行：只用低 32 位算段内余数。"""
    ordinal_lo = ordinal % U32
    remaining = (ordinal_lo - (segment_start % U32)) % U32
    landing = []
    for c in choices:
        digit = remaining % c
        remaining //= c
        landing.append(2 if digit == c - 1 else (1 if digit == 1 else 0))
    return tuple(landing)


def gpu_decode_with_64_bit_remainder(ordinal, segment_start, choices):
    """改法甲的样子（段内余数按 lo/hi 两个字算满 64 位）：只给判别力自证用。"""
    remaining = ordinal - segment_start
    landing = []
    for c in choices:
        digit = remaining % c
        remaining //= c
        landing.append(2 if digit == c - 1 else (1 if digit == 1 else 0))
    return tuple(landing)


def cpu_decode(ordinal, segment_start, choices):
    """照 crash_facts.rs 第 556–567 行：u64 余数，末尾断言余数归零。"""
    remaining = ordinal - segment_start
    landing = []
    for c in choices:
        digit = remaining % c
        remaining //= c
        landing.append({(2, 0): 0, (3, 0): 0, (2, 1): 2, (3, 2): 2, (3, 1): 1}[(c, digit)])
    assert remaining == 0, "段内序号超出组合数"
    return tuple(landing)


def cpu_decode_jia_er(local, segment_order, in_place, copy_on_write):
    """照 crates/singlefs-checker-tier/src/crash.rs 第 2389–2405 行（甲二）：原地写按 local / 2 拆（这里都取两态），local % 2 == 1 时单元写全落。"""
    landing = {w: 0 for w in segment_order}
    remaining = local // 2 if copy_on_write else local
    for w in in_place:
        digit = remaining % 2
        remaining //= 2
        landing[w] = 2 if digit == 1 else 0
    if copy_on_write and local % 2 == 1:
        for w in copy_on_write:
            landing[w] = 2
    return tuple(landing[w] for w in segment_order)


def jia_er_states_on_gpu(segment_order, in_place, copy_on_write):
    """甲二那一段的序号区间（2^(k+1) − 1 个）分别交 GPU 内核的拆法（只认每次写的两 / 三态、不认段怎么展开）与 CPU 甲二拆法。"""
    count = (1 << (len(in_place) + 1)) - 1 if copy_on_write else (1 << len(in_place)) - 1
    choices = [2] * len(segment_order)
    gpu = {gpu_decode(local, 0, choices) for local in range(count)}
    cpu = {cpu_decode_jia_er(local, segment_order, in_place, copy_on_write) for local in range(count)}
    position = {w: i for i, w in enumerate(segment_order)}
    partial = sum(1 for mask in gpu if 0 < sum(1 for w in copy_on_write if mask[position[w]] == 2) < len(copy_on_write))
    return count, len(gpu & cpu), len(gpu - cpu), partial


def first_alias(writes, tearable, segment_start):
    """一段 writes 次写（其中 tearable 次取三态）在全展开下：找第一个 GPU 拆法与 CPU 不同的段内序号。"""
    choices = [3] * tearable + [2] * (writes - tearable)
    total = 1
    for c in choices:
        total *= c
    states = total - 1  # 去掉整段全落那一个（D13 已定项 4）
    if states < U32:
        return states, None
    local = U32 + 5  # 第一个越过 2^32 的段内序号再往后 5 个，挑一个非平凡的
    ordinal = segment_start + local
    gpu = gpu_decode(ordinal, segment_start, choices)
    cpu = cpu_decode(ordinal, segment_start, choices)
    return states, (local, gpu == cpu, gpu == cpu_decode(segment_start + (local % U32), segment_start, choices))


def main():
    rows = []
    for writes, tearable in [(30, 0), (32, 0), (33, 0), (21, 2), (20, 8), (64, 0)]:
        # 段起点取 2^31 + 7：前面几段已经占掉一些序号，段起点不在 2^32 的整数倍上
        states, alias = first_alias(writes, tearable, (1 << 31) + 7)
        rows.append((writes, tearable, states, alias))
        if alias is None:
            print(f"DECODE writes={writes} tearable={tearable} states={states} fits_u32=true")
        else:
            local, same, same_as_wrapped = alias
            print(f"DECODE writes={writes} tearable={tearable} states={states} fits_u32=false "
                  f"local_ordinal={local} gpu_equals_cpu={str(same).lower()} gpu_equals_cpu_of_local_mod_2^32={str(same_as_wrapped).lower()}")
    jia_er = []
    for order in ([0, 1, 2, 3, 4], [2, 3, 4, 0, 1]):
        count, same, gpu_only, partial = jia_er_states_on_gpu(order, [0, 1], [2, 3, 4])
        jia_er.append((count, same, gpu_only, partial))
        print(f"JIA_ER segment_order={order} in_place=[0,1] copy_on_write=[2,3,4] states={count} states_in_both={same} gpu_only={gpu_only} gpu_partial_copy_on_write={partial}")
    skipped_322 = (1 << 322) - 1
    print(f"SKIPPED segment=322 states_skipped={skipped_322.bit_length()}-bit fits_u64={str(skipped_322 < (1 << 64)).lower()} fits_u128={str(skipped_322 < (1 << 128)).lower()}")
    checks = [
        (rows[0][3], None),            # 30 写（今天最长的段）：装得进 u32，GPU 照拆
        (rows[1][3], None),            # 32 写：2^32 − 1 个，恰好装得进
        (rows[2][2], (1 << 33) - 1),
        (rows[2][3][1], False),        # 33 写：GPU 与 CPU 拆出不同的掩码
        (rows[2][3][2], True),         # GPU 拆出的就是「序号 mod 2^32」那个状态：重判一个已判的、漏掉这一个
        (rows[3][2], 9 * (1 << 19) - 1),
        (rows[3][3], None),            # 21 写里 2 次三态：装得进
        (rows[4][2], 3 ** 8 * (1 << 12) - 1),
        (rows[4][3], None),            # 20 写里 8 次三态：装得进
        (rows[5][3][1], False),        # 64 写：仍装得进 u64（G10 不登记它），GPU 照样拆错
        (skipped_322.bit_length(), 322),
        (jia_er[0][0], 7),
        (jia_er[0][2] > 0, True),      # 原地写在前：GPU 判到的状态里有甲二没有的
        (jia_er[1][3] > 0, True),      # 单元写在前：GPU 判到只落一部分单元写的状态（甲二恰好不判这一类）
    ]
    # 判别力自证：把 GPU 那一份换成按 64 位算余数（改法甲），33 写那一格必须翻成相同；不翻就是这份模型比不出差
    choices_33 = [2] * 33
    start = (1 << 31) + 7
    fixed = gpu_decode_with_64_bit_remainder(start + U32 + 5, start, choices_33) == cpu_decode(start + U32 + 5, start, choices_33)
    print(f"MUTANT gpu_remainder_64_bit writes=33 gpu_equals_cpu={str(fixed).lower()}")
    checks.append((fixed, True))
    bad = [(i, got, want) for i, (got, want) in enumerate(checks) if got != want]
    print(f"MODEL_SELFCHECK compared={len(checks)} mismatched={len(bad)} {bad}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
